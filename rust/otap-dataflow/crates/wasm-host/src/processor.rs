// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The WASM processor node and its engine factory registration.
//!
//! A [`WasmProcessor`] owns a wasmtime [`Store`] with per-instance
//! [`HostState`] and a long-running instantiation of the `kernel-processor`
//! world. The component is compiled once when the factory creates the node
//! (at pipeline startup, per core); there is no compile or instantiate step in
//! the hot path.
//!
//! Execution is synchronous and in-core: `process` runs on the pipeline's
//! per-core thread and the store-owned state is never shared across threads
//! (the processor is a `!Send` local node).

use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::RecordBatch;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::{Error as EngineError, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use serde::{Deserialize, Serialize};
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Engine, Store};

use crate::bindings::KernelProcessor;
use crate::bridge;
use crate::host::{HostBatchData, HostState};

/// URN identifying the WASM processor component.
pub const WASM_PROCESSOR_URN: &str = "urn:otel:processor:wasm_processor";

/// Configuration for the WASM processor node.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmProcessorConfig {
    /// Filesystem path to the `.wasm` component plugin to load at startup.
    pub wasm_path: PathBuf,
}

/// A processor node that delegates `process` to a WASM guest plugin driving
/// native host kernels.
///
/// The wasmtime types are `!Send`/`!Sync`; the node is therefore a local
/// (single-threaded) processor confined to one pipeline/core thread.
pub struct WasmProcessor {
    store: Store<HostState>,
    instance: KernelProcessor,
    // Kept alive for the lifetime of the node; the compiled component and
    // engine are the once-at-startup artifacts we deliberately do not rebuild
    // in the hot path.
    _engine: Engine,
    _component: Component,
}

impl WasmProcessor {
    /// Compile and instantiate the plugin at `wasm_path`.
    ///
    /// This performs the one-time (per-core) compile + instantiate work.
    ///
    /// TODO: add an AOT module cache keyed on wasm content hash,
    /// wasmtime version and target triple, and epoch-interruption limits.
    fn from_path(wasm_path: &PathBuf) -> Result<Self, ConfigError> {
        let engine = Engine::default();
        let component = Component::from_file(&engine, wasm_path).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to load wasm component at {wasm_path:?}: {e}"),
            }
        })?;

        let mut linker: Linker<HostState> = Linker::new(&engine);
        KernelProcessor::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |s| s)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("failed to link wasm host kernels: {e}"),
            })?;

        let mut store = Store::new(&engine, HostState::new());
        let instance =
            KernelProcessor::instantiate(&mut store, &component, &linker).map_err(|e| {
                ConfigError::InvalidUserConfig {
                    error: format!("failed to instantiate wasm plugin: {e}"),
                }
            })?;

        Ok(Self {
            store,
            instance,
            _engine: engine,
            _component: component,
        })
    }

    /// Push `batch` into the handle table, invoke the guest `process`, and
    /// return the resulting batch (or `None` when the guest dropped it).
    fn run_guest(&mut self, batch: RecordBatch) -> wasmtime::Result<Option<RecordBatch>> {
        let input = self.store.data_mut().table.push(HostBatchData {
            record_batch: batch,
        })?;
        let input_rep = input.rep();

        let output = match self
            .instance
            .otel_otap_dataflow_plugin_processor()
            .call_process(&mut self.store, input)
        {
            Ok(output) => output,
            Err(err) => {
                // Best-effort cleanup: the guest may already have consumed or
                // dropped this handle before trapping/returning an error.
                let _ = self
                    .store
                    .data_mut()
                    .table
                    .delete(wasmtime::component::Resource::<HostBatchData>::new_own(
                        input_rep,
                    ));
                return Err(err);
            }
        };

        match output {
            Some(handle) => {
                let data = self.store.data_mut().table.delete(handle)?;
                Ok(Some(data.record_batch))
            }
            None => Ok(None),
        }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for WasmProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            // TODO: report plugin telemetry on CollectTelemetry.
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                let processor_id = effect_handler.processor_id();
                let output = bridge::run_on_root_batch(pdata, |batch| {
                    self.run_guest(batch)
                        .map_err(|e| EngineError::ProcessorError {
                            processor: processor_id.clone(),
                            kind: ProcessorErrorKind::Other,
                            error: format!("wasm plugin process failed: {e}"),
                            source_detail: String::new(),
                        })
                })?;

                match output {
                    Some(pdata) => effect_handler
                        .send_message_with_source_node(pdata)
                        .await
                        .map_err(Into::into),
                    // Guest returned `none`: drop the batch.
                    None => Ok(()),
                }
            }
        }
    }
}

/// Factory function to create a [`WasmProcessor`] node.
fn create_wasm_processor(
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: WasmProcessorConfig =
        serde_json::from_value(node_config.config.clone()).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to parse WasmProcessor configuration: {e}"),
            }
        })?;

    let processor = WasmProcessor::from_path(&config.wasm_path)?;

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::testing::node::test_node;

    #[test]
    fn create_wasm_processor_rejects_invalid_config_shape() {
        let node = test_node("wasm-test");
        let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
        node_config.config = serde_json::json!("not an object");
        let processor_config = ProcessorConfig::new("wasm-test");

        let result = create_wasm_processor(node, Arc::new(node_config), &processor_config);
        assert!(
            matches!(result, Err(ConfigError::InvalidUserConfig { .. })),
            "invalid user config JSON should be rejected"
        );
    }

    #[test]
    fn create_wasm_processor_rejects_missing_wasm_file() {
        let node = test_node("wasm-test");
        let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
        node_config.config = serde_json::json!({
            "wasm_path": "/definitely/missing/wasm-host-plugin.wasm"
        });
        let processor_config = ProcessorConfig::new("wasm-test");

        let result = create_wasm_processor(node, Arc::new(node_config), &processor_config);
        assert!(
            matches!(result, Err(ConfigError::InvalidUserConfig { .. })),
            "missing wasm component file should map to InvalidUserConfig"
        );
    }
}

/// Register [`WasmProcessor`] as an OTAP processor factory.
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static WASM_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: WASM_PROCESSOR_URN,
        create:
            |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
                create_wasm_processor(node, node_config, proc_cfg)
            },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<WasmProcessorConfig>,
    };
