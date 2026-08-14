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

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::{Error as EngineError, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::OtapPayload;
use serde::{Deserialize, Serialize};
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Engine, Store};

use crate::bindings::KernelProcessor;
use crate::bridge;
use crate::host::{HostPdata, HostState};
use crate::metrics::WasmProcessorAllMetrics;

/// URN identifying the WASM processor component.
pub const WASM_PROCESSOR_URN: &str = "urn:otel:processor:wasm_processor";

otap_df_telemetry::otel_component_scope!(
    urn = WASM_PROCESSOR_URN,
    target = "otel.processor.wasm_processor",
);

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
    metrics: WasmProcessorAllMetrics,
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
    fn from_path(
        wasm_path: &PathBuf,
        metrics: WasmProcessorAllMetrics,
    ) -> Result<Self, ConfigError> {
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
            metrics,
            _engine: engine,
            _component: component,
        })
    }

    /// Push `batch` into the handle table, invoke the guest `process`, and
    /// return the resulting batch (or `None` when the guest dropped it).
    fn run_guest(
        &mut self,
        otap_batch: OtapArrowRecords,
    ) -> wasmtime::Result<Option<OtapArrowRecords>> {
        let input = self.store.data_mut().table.push(HostPdata { otap_batch })?;
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
                let _ =
                    self.store.data_mut().table.delete(
                        wasmtime::component::Resource::<HostPdata>::new_own(input_rep),
                    );
                return Err(err);
            }
        };

        match output {
            Some(handle) => {
                let data = self.store.data_mut().table.delete(handle)?;
                Ok(Some(data.otap_batch))
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
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                let _ = self.metrics.report(&mut metrics_reporter);
                Ok(())
            }
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                let processor_id = effect_handler.processor_id();
                let (context, payload) = pdata.into_parts();
                let signal_type = payload.signal_type();
                let output = bridge::run_on_otap_records(
                    OtapPdata::new(context.clone(), payload),
                    |records| {
                        self.metrics.pdata.guest_process_calls.add(1);
                        // Count rows entering the guest before consuming records.
                        let rows_in = records
                            .root_record_batch()
                            .map_or(0, |b| b.num_rows() as u64);

                        let result = self.run_guest(records).map_err(|e| {
                            self.metrics.pdata.guest_process_errors.add(1);
                            EngineError::ProcessorError {
                                processor: processor_id.clone(),
                                kind: ProcessorErrorKind::Other,
                                error: format!("wasm plugin process failed: {e}"),
                                source_detail: String::new(),
                            }
                        });

                        // Record rows-in by signal type.
                        self.metrics
                            .records_for(signal_type)
                            .records_in
                            .add(rows_in);

                        // Record rows-out if the guest returned a batch.
                        if let Ok(Some(ref out)) = result {
                            let rows_out =
                                out.root_record_batch().map_or(0, |b| b.num_rows() as u64);
                            self.metrics
                                .records_for(signal_type)
                                .records_out
                                .add(rows_out);
                        }

                        result
                    },
                );

                // Drain per-call kernel counters outside the closure so they
                // are captured for successful and error-return guest calls.
                let kc = self.store.data_mut().drain_kernel_counters();
                self.metrics.pdata.kernel_calls.add(kc);

                let output = output?;

                match output {
                    Some(pdata) => effect_handler
                        .send_message_with_source_node(pdata)
                        .await
                        .map_err(Into::into),
                    // Guest returned `none`: intentionally drop this pdata and
                    // ack upstream so context unwinding follows normal
                    // processor drop semantics.
                    None => {
                        self.metrics.pdata.pdata_dropped.add(1);
                        let dropped = OtapPdata::new(context, OtapPayload::empty(signal_type));
                        effect_handler.notify_ack(AckMsg::new(dropped)).await
                    }
                }
            }
        }
    }
}

/// Factory function to create a [`WasmProcessor`] node.
fn create_wasm_processor(
    pipeline_ctx: PipelineContext,
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

    let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);
    let processor = WasmProcessor::from_path(&config.wasm_path, metrics)?;

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
    use std::sync::Arc;
    use std::time::Duration;

    use otap_df_config::SignalType;
    use otap_df_engine::Interests;
    use otap_df_engine::ProducerEffectHandlerExtension;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{
        CallData, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
    };
    use otap_df_engine::local::processor::Processor;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_otap::pdata::Context;
    use otap_df_pdata::OtapPayload;
    use tokio::time::timeout;

    struct DropAllProcessor;

    #[async_trait(?Send)]
    impl Processor<OtapPdata> for DropAllProcessor {
        async fn process(
            &mut self,
            msg: Message<OtapPdata>,
            effect_handler: &mut local::EffectHandler<OtapPdata>,
        ) -> Result<(), EngineError> {
            match msg {
                Message::Control(NodeControlMsg::CollectTelemetry { .. }) => Ok(()),
                Message::Control(_) => Ok(()),
                Message::PData(mut pdata) => {
                    effect_handler.subscribe_to(Interests::ACKS, CallData::default(), &mut pdata);
                    let (context, payload) = pdata.into_parts();
                    let dropped =
                        OtapPdata::new(context, OtapPayload::empty(payload.signal_type()));
                    effect_handler.notify_ack(AckMsg::new(dropped)).await
                }
            }
        }
    }

    /// Scenario: Processor config JSON is not an object.
    /// Guarantees: Factory rejects malformed config with InvalidUserConfig.
    #[test]
    fn create_wasm_processor_rejects_invalid_config_shape() {
        let node = test_node("wasm-test");
        let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
        node_config.config = serde_json::json!("not an object");
        let processor_config = ProcessorConfig::new("wasm-test");
        let controller_ctx =
            ControllerContext::new(otap_df_telemetry::registry::TelemetryRegistryHandle::new());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let result =
            create_wasm_processor(pipeline_ctx, node, Arc::new(node_config), &processor_config);
        assert!(
            matches!(result, Err(ConfigError::InvalidUserConfig { .. })),
            "invalid user config JSON should be rejected"
        );
    }

    /// Scenario: Processor config points to a missing wasm file.
    /// Guarantees: Factory maps missing component file to InvalidUserConfig.
    #[test]
    fn create_wasm_processor_rejects_missing_wasm_file() {
        let node = test_node("wasm-test");
        let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
        node_config.config = serde_json::json!({
            "wasm_path": "/definitely/missing/wasm-host-plugin.wasm"
        });
        let processor_config = ProcessorConfig::new("wasm-test");
        let controller_ctx =
            ControllerContext::new(otap_df_telemetry::registry::TelemetryRegistryHandle::new());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let result =
            create_wasm_processor(pipeline_ctx, node, Arc::new(node_config), &processor_config);
        assert!(
            matches!(result, Err(ConfigError::InvalidUserConfig { .. })),
            "missing wasm component file should map to InvalidUserConfig"
        );
    }

    /// Scenario: A processor intentionally drops a pdata item.
    /// Guarantees: The drop path emits an Ack completion and does not forward output pdata.
    #[test]
    fn dropping_pdata_routes_ack_completion() {
        let runtime = TestRuntime::new();
        let node = test_node("drop-all");
        let node_config = Arc::new(NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN));
        let wrapper =
            ProcessorWrapper::local(DropAllProcessor, node, node_config, runtime.config());

        let phase = runtime.set_processor(wrapper);
        phase
            .run_test(|mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(8);
                ctx.set_pipeline_completion_sender(completion_tx);

                let input =
                    OtapPdata::new(Context::default(), OtapPayload::empty(SignalType::Logs));

                ctx.process(Message::PData(input))
                    .await
                    .expect("drop process should succeed");

                let emitted = ctx.drain_pdata().await;
                assert!(
                    emitted.is_empty(),
                    "drop path must not forward pdata downstream"
                );

                let completion = timeout(Duration::from_secs(1), completion_rx.recv())
                    .await
                    .expect("ack completion should arrive before timeout")
                    .expect("completion channel should have ack");
                match completion {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        assert!(
                            ack.accepted.is_empty(),
                            "drop ack should carry an empty payload"
                        );
                        assert_eq!(ack.accepted.signal_type(), SignalType::Logs);
                    }
                    other => panic!("expected DeliverAck, got {other:?}"),
                }
            })
            .validate(|_ctx| async {});
    }

    /// Scenario: `WasmProcessorAllMetrics` is constructed and record
    /// throughput counters are accessed per signal type.
    /// Guarantees: `records_for` partitions counters by signal; each signal
    /// type accumulates independently and increments are observable.
    #[test]
    fn metrics_records_for_partitions_by_signal_type() {
        let controller_ctx =
            ControllerContext::new(otap_df_telemetry::registry::TelemetryRegistryHandle::new());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        metrics.records_for(SignalType::Logs).records_in.add(10);
        metrics.records_for(SignalType::Logs).records_out.add(7);
        metrics.records_for(SignalType::Metrics).records_in.add(5);
        metrics.records_for(SignalType::Traces).records_in.add(3);

        assert_eq!(
            metrics.records_for(SignalType::Logs).records_in.get(),
            10,
            "logs records_in should be 10"
        );
        assert_eq!(
            metrics.records_for(SignalType::Logs).records_out.get(),
            7,
            "logs records_out should be 7"
        );
        assert_eq!(
            metrics.records_for(SignalType::Metrics).records_in.get(),
            5,
            "metrics records_in should be 5 independently of logs"
        );
        assert_eq!(
            metrics.records_for(SignalType::Traces).records_in.get(),
            3,
            "traces records_in should be 3"
        );
        // Metrics records_out was never incremented -- should remain zero.
        assert_eq!(
            metrics.records_for(SignalType::Metrics).records_out.get(),
            0,
            "unincremented records_out should be zero"
        );
    }
}

/// Register [`WasmProcessor`] as an OTAP processor factory.
#[otap_df_engine::component_inventory(category = Processor)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static WASM_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: WASM_PROCESSOR_URN,
        create:
            |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
                create_wasm_processor(pipeline, node, node_config, proc_cfg)
            },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<WasmProcessorConfig>,
    };
