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
//! Execution is in-core: `process` directly awaits the guest on the pipeline's
//! per-core runtime, while synchronous initialization and teardown reject
//! suspending clock waits. Store-owned state is never shared across threads.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::MessageSourceLocalEffectHandlerExtension;
use otel_arrow_dfe_engine::config::ProcessorConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::{Error as EngineError, ProcessorErrorKind};
use otel_arrow_dfe_engine::local::processor as local;
use otel_arrow_dfe_engine::message::Message;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::processor::ProcessorWrapper;
use otel_arrow_dfe_otap::OTAP_PROCESSOR_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::OtapPayload;
use serde::{Deserialize, Serialize};
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Config, Engine, Store};

use crate::bindings::KernelProcessor;
use crate::bridge;
use crate::host::{
    GuestCounterActivity, HostPdata, HostState, MAX_GUEST_HOSTCALL_BYTES, add_wasi_clocks_to_linker,
};
use crate::metrics::WasmProcessorAllMetrics;

/// URN identifying the WASM processor component.
pub const WASM_PROCESSOR_URN: &str = "urn:otel:processor:wasm_processor";

/// Wasmtime "fuel" budget granted before each guest entry point call
/// (`initialize`, `process`, `shutdown`). Wasmtime charges roughly one unit
/// of fuel per interpreted Wasm instruction (with some instructions costing
/// more), so this bounds guest Wasm instructions per call and turns
/// runaway/infinite guest loops into a catchable `Trap::OutOfFuel` instead of
/// hanging the host. Native host-kernel work has a separate invocation budget.
///
/// Exhausting the budget is *terminal for the plugin instance*, not a
/// per-call throttle: Wasmtime marks the instance unusable after any trap
/// (see [`WasmProcessor::poisoned`]), and the engine terminates a node whose
/// `process` returns an error. Size this generously; a batch that legitimately
/// needs more work than the budget kills the node rather than being retried.
///
/// TODO: This value is a placeholder, not derived from profiling or a documented
/// budget: it has not been benchmarked against real guest workloads (e.g.
/// the `severity-filter` reference plugin) and is not yet configurable per
/// plugin/pipeline. Resource limits and failure policy are being designed
/// properly as part of limits-and-cache follow-on work; revisit this
/// constant (and consider making it configurable) there.
const GUEST_FUEL_PER_CALL: u64 = 10_000_000;

/// Fold one `drain_counter_add_calls()` result into the processor's telemetry
/// counters.
///
/// `value` is derived from guest-supplied `counter-add` arguments, so it is
/// added with saturation: `Counter::add` is a checked `+=` by default, and
/// letting an unbounded guest total reach it would allow a guest to panic the
/// host (debug/test) or wrap the reported metric (release).
fn fold_guest_counter_metrics(
    metrics: &mut WasmProcessorAllMetrics,
    activity: GuestCounterActivity,
) {
    metrics.pdata.guest_counter_add_calls.add(activity.accepted);
    metrics
        .pdata
        .guest_counter_add_rejected_name_len
        .add(activity.rejected_name_len);
    metrics
        .pdata
        .guest_counter_add_rejected_cardinality
        .add(activity.rejected_cardinality);
    let headroom = u64::MAX - metrics.pdata.guest_counter_add_value.get();
    metrics
        .pdata
        .guest_counter_add_value
        .add(activity.value.min(headroom));
}

/// Fold one `drain_host_service_budget_calls()` result into the processor's
/// telemetry counters.
fn fold_guest_host_service_budget(
    metrics: &mut WasmProcessorAllMetrics,
    (calls_rejected, log_truncated): (u64, u64),
) {
    metrics
        .pdata
        .guest_host_service_calls_rejected
        .add(calls_rejected);
    metrics.pdata.guest_log_message_truncated.add(log_truncated);
}

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = WASM_PROCESSOR_URN,
    target = "otel.processor.wasm_processor",
);

/// Configuration for the WASM processor node.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmProcessorConfig {
    /// Filesystem path to the `.wasm` component plugin to load at startup.
    pub wasm_path: PathBuf,
    /// Optional freeform, plugin-specific configuration. Serialized to a
    /// string and served verbatim by `host-services.get-config`; guests
    /// fetch it from their own `initialize` implementation. Backward
    /// compatible: configs with only `wasm_path` remain valid (this field
    /// defaults to `None`, served as `null`).
    #[serde(default)]
    pub config: Option<serde_json::Value>,
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
    // Tracks whether the guest's `shutdown` export has run, so the teardown
    // path (see `Drop for WasmProcessor`) never double-calls the guest.
    shutdown_called: bool,
    // Set once any guest call traps. Wasmtime marks a store unusable after a
    // trap: every later entry into the instance fails with
    // `Trap::CannotEnterComponent` regardless of what it is asked to do. We
    // track it explicitly so teardown can skip a guest `shutdown` that could
    // only fail, rather than emitting a confusing second error for the same
    // underlying fault.
    poisoned: bool,
}

impl WasmProcessor {
    /// Compile and instantiate the plugin at `wasm_path`, serve `config` (if
    /// any) via `host-services.get-config`, and call the guest's `initialize`
    /// export exactly once before returning.
    ///
    /// This performs the one-time (per-core) compile + instantiate work.
    /// Initialization failures are propagated as `ConfigError::InvalidUserConfig`
    /// so a misconfigured plugin fails pipeline *startup*, not a later
    /// runtime `process` call.
    ///
    fn from_path(
        wasm_path: &PathBuf,
        plugin_config: Option<&serde_json::Value>,
        node_name: String,
        mut metrics: WasmProcessorAllMetrics,
    ) -> Result<Self, ConfigError> {
        let mut engine_config = Config::new();
        let _ = engine_config.consume_fuel(true);
        let _ = engine_config.wasm_component_model_async(true);
        let _ = engine_config.wasm_component_model_more_async_builtins(true);
        let _ = engine_config.wasm_component_model_async_stackful(true);
        let _ = engine_config.concurrency_support(true);
        let engine = Engine::new(&engine_config).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to configure wasm engine: {e}"),
        })?;
        let component = Component::from_file(&engine, wasm_path).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to load wasm component at {wasm_path:?}: {e:?}"),
            }
        })?;

        let mut linker: Linker<HostState> = Linker::new(&engine);
        // Link the plugin contract plus only the WASI CLI and clock interfaces
        // required by Rust `std`. This call site is the capability boundary:
        // filesystem, sockets, random, and other WASI interfaces remain
        // unavailable and fail component instantiation if imported.
        KernelProcessor::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |s| s)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("failed to link wasm host kernels: {e}"),
            })?;
        wasmtime_wasi::p3::cli::add_to_linker(&mut linker).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to link WASI CLI interfaces: {e}"),
            }
        })?;
        add_wasi_clocks_to_linker(&mut linker).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to link WASI clock interfaces: {e}"),
        })?;

        // Serve *only* the plugin-owned `config` sub-object. The host's own
        // fields (notably `wasm_path`) stay host-side: a guest in this world
        // has no filesystem capability, so it has no use for a host path, and
        // withholding it keeps `WasmProcessorConfig`'s shape from silently
        // becoming part of the plugin ABI.
        let config_blob =
            serde_json::to_string(&plugin_config).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("failed to serialize plugin config for the guest: {e}"),
            })?;

        let mut store = Store::new(
            &engine,
            HostState::with_config_for_node(config_blob, node_name),
        );
        // Limit canonical-ABI lifting before allocating guest-sized strings,
        // including imports during instantiation and exported error results.
        store.set_hostcall_fuel(MAX_GUEST_HOSTCALL_BYTES);
        store.limiter(|state| state as &mut dyn wasmtime::ResourceLimiter);
        store
            .set_fuel(GUEST_FUEL_PER_CALL)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("failed to configure wasm execution fuel: {e}"),
            })?;
        let instance = futures::executor::block_on(KernelProcessor::instantiate_async(
            &mut store, &component, &linker,
        ))
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to instantiate wasm plugin: {e}"),
        })?;

        // Reset fuel after instantiation: instantiation runs guest code
        // (component/core start functions), so without this `initialize`
        // would run on whatever budget instantiation happened to leave,
        // making startup failures depend on instantiation cost.
        store
            .set_fuel(GUEST_FUEL_PER_CALL)
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("failed to configure wasm execution fuel: {e}"),
            })?;
        // Call the guest's `initialize` export exactly once, before any
        // `process` call is ever attempted.
        store.data_mut().set_clock_waits_enabled(false);
        store.data_mut().begin_guest_call();
        let init_result = futures::executor::block_on(store.run_concurrent(async |accessor| {
            instance
                .otel_otap_dataflow_plugin_lifecycle()
                .call_initialize(accessor)
                .await
        }));

        // Fold whatever host-service activity `initialize` produced into the
        // metric set on every path. On the failure paths this is bookkeeping
        // only: no node is constructed, so nothing will ever deliver
        // `CollectTelemetry` to report it. The failure itself is surfaced by
        // the `otel_warn!` below plus the returned config error, which is
        // what an operator actually sees when a pipeline refuses to start.
        let drained = store.data_mut().drain_counter_add_calls();
        fold_guest_counter_metrics(&mut metrics, drained);
        let budget_drained = store.data_mut().drain_host_service_budget_calls();
        fold_guest_host_service_budget(&mut metrics, budget_drained);

        match init_result {
            Err(error) => {
                otel_warn!(
                    "wasm_processor.initialize_trap",
                    error = format!("{error:#}")
                );
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("wasm plugin initialize trapped: {error:#}"),
                });
            }
            Ok(Err(error)) => {
                otel_warn!(
                    "wasm_processor.initialize_trap",
                    error = format!("{error:#}")
                );
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("wasm plugin initialize trapped: {error:#}"),
                });
            }
            Ok(Ok(Err(init_error))) => {
                otel_warn!(
                    "wasm_processor.initialize_failed",
                    error = init_error.message.as_str()
                );
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("wasm plugin initialize failed: {}", init_error.message),
                });
            }
            Ok(Ok(Ok(()))) => {}
        }

        Ok(Self {
            store,
            instance,
            metrics,
            _engine: engine,
            _component: component,
            shutdown_called: false,
            poisoned: false,
        })
    }

    /// Call the guest's `shutdown` export exactly once. Safe to call multiple
    /// times; subsequent calls are no-ops.
    ///
    /// Teardown ordering: this is deliberately *not* called from the
    /// `NodeControlMsg::Shutdown` arm of `process` below. That control
    /// message begins a drain rather than ending the node -- the engine
    /// broadcasts it to every non-receiver node at once so upstream nodes can
    /// flush buffered data, and the node loop keeps delivering messages until
    /// the inbox closes. Buffering processors upstream (the batch and
    /// temporal-reaggregation processors, for example) emit pdata precisely
    /// on that signal, so a downstream plugin that shut down on `Shutdown`
    /// would then be asked to `process` data with its state already torn
    /// down. `wit/plugin.wit` promises `shutdown` runs after the last
    /// `process` call, so it runs from `Drop` instead: the engine drops the
    /// processor once its loop ends, which is the first point at which that
    /// promise actually holds.
    ///
    /// Skipped entirely if a previous guest call trapped: Wasmtime marks the
    /// instance permanently unusable after a trap, so calling in could only
    /// produce a second, misleading error for the same fault.
    fn shutdown_guest(&mut self) {
        if self.shutdown_called {
            return;
        }
        self.shutdown_called = true;
        if self.poisoned {
            otel_warn!("wasm_processor.shutdown_skipped_after_trap");
            return;
        }
        if let Err(e) = self.store.set_fuel(GUEST_FUEL_PER_CALL) {
            otel_warn!("wasm_processor.shutdown_fuel_setup_failed", error = %e);
            self.drain_host_service_counters();
            return;
        }
        self.store.data_mut().set_clock_waits_enabled(false);
        self.store.data_mut().begin_guest_call();
        if let Err(e) = futures::executor::block_on(self.store.run_concurrent(async |accessor| {
            self.instance
                .otel_otap_dataflow_plugin_lifecycle()
                .call_shutdown(accessor)
                .await
        }))
        .and_then(|result| result)
        {
            self.poisoned = true;
            otel_warn!("wasm_processor.shutdown_trap", error = format!("{e:#}"));
        }
        self.drain_host_service_counters();
    }

    fn drain_host_service_counters(&mut self) {
        let drained = self.store.data_mut().drain_counter_add_calls();
        fold_guest_counter_metrics(&mut self.metrics, drained);
        let budget_drained = self.store.data_mut().drain_host_service_budget_calls();
        fold_guest_host_service_budget(&mut self.metrics, budget_drained);
    }

    /// Push `batch` into the handle table, invoke the guest `process`, and
    /// return the resulting batch (or `None` when the guest dropped it).
    async fn run_guest(
        &mut self,
        otap_batch: OtapArrowRecords,
    ) -> wasmtime::Result<Option<OtapArrowRecords>> {
        self.store.set_fuel(GUEST_FUEL_PER_CALL)?;
        self.store.data_mut().begin_guest_call();
        let input = self.store.data_mut().table.push(HostPdata { otap_batch })?;
        let input_rep = input.rep();
        self.store.data_mut().set_clock_waits_enabled(true);

        let call_result = self
            .store
            .run_concurrent(async |accessor| {
                self.instance
                    .otel_otap_dataflow_plugin_processor()
                    .call_process(accessor, input)
                    .await
            })
            .await
            .and_then(|result| result);
        self.store.data_mut().set_clock_waits_enabled(false);

        let output = match call_result {
            Ok(output) => output,
            Err(err) => {
                // Any error out of `call_process` is a trap (fuel exhaustion,
                // a guest panic, or a kernel contract violation), and a trap
                // makes the whole instance permanently unusable. Record that
                // so teardown does not try to re-enter the guest.
                self.poisoned = true;
                // Best-effort cleanup: the guest may already have consumed or
                // dropped this handle before trapping.
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

impl Drop for WasmProcessor {
    /// Primary teardown path for the guest's `shutdown` export.
    ///
    /// The engine drops the processor after its message loop ends, which is
    /// the first moment `wit/plugin.wit`'s "after the last `process` call has
    /// returned" guarantee actually holds -- see [`WasmProcessor::shutdown_guest`]
    /// for why the `NodeControlMsg::Shutdown` control message is the wrong
    /// place for it.
    ///
    /// Skipped while the thread is already panicking. `shutdown_guest` runs
    /// guest code, and guest code can trap; if the host is unwinding from a
    /// panic, a panic raised here would be a panic-during-unwind and abort
    /// the process, replacing a recoverable node failure with a hard crash.
    /// `wit/plugin.wit` documents `shutdown` as best-effort for exactly this
    /// case.
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        self.shutdown_guest();
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
            // `Shutdown` starts a pipeline drain, not this node's teardown:
            // upstream nodes flush on the same signal, so more pdata can
            // still arrive. The guest's `shutdown` therefore runs from `Drop`
            // (see `shutdown_guest`), once the node loop has actually ended.
            Message::Control(NodeControlMsg::Shutdown { .. }) => Ok(()),
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                // Emit the guest's per-name counter totals on the metrics
                // tick rather than on every `counter-add` call, keeping guest
                // counting off the pdata hot path.
                self.store.data().report_guest_counters();
                let _ = self.metrics.report(&mut metrics_reporter);
                Ok(())
            }
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                let processor_id = effect_handler.processor_id();
                let (context, payload) = pdata.into_parts();
                let signal_type = payload.signal_type();
                let output = bridge::run_on_otap_records_async(
                    OtapPdata::new(context.clone(), payload),
                    |records| async {
                        self.metrics.pdata.guest_process_calls.add(1);
                        // Count rows entering the guest before consuming records.
                        let rows_in = records
                            .root_record_batch()
                            .map_or(0, |b| b.num_rows() as u64);

                        let result = self.run_guest(records).await.map_err(|e| {
                            self.metrics.pdata.guest_process_errors.add(1);
                            EngineError::ProcessorError {
                                processor: processor_id.clone(),
                                kind: ProcessorErrorKind::Other,
                                error: format!("wasm plugin process failed: {e:#}"),
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
                )
                .await;

                // Drain per-call kernel counters outside the closure so they
                // are captured for successful and error-return guest calls.
                let kc = self.store.data_mut().drain_kernel_counters();
                self.metrics.pdata.kernel_calls.add(kc);

                self.drain_host_service_counters();

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
    let processor = WasmProcessor::from_path(
        &config.wasm_path,
        config.config.as_ref(),
        node.name.to_string(),
        metrics,
    )?;

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

    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::ProducerEffectHandlerExtension;
    use otel_arrow_dfe_engine::config::ProcessorConfig;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::control::{
        CallData, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
    };
    use otel_arrow_dfe_engine::local::processor::Processor;
    use otel_arrow_dfe_engine::message::Message;
    use otel_arrow_dfe_engine::testing::node::test_node;
    use otel_arrow_dfe_engine::testing::processor::TestRuntime;
    use otel_arrow_dfe_otap::pdata::Context;
    use otel_arrow_dfe_pdata::OtapPayload;
    use tokio::time::timeout;

    /// Build a minimal `OtapArrowRecords::Logs` batch with a `severity_text`
    /// column, for exercising `WasmProcessor::run_guest` directly in tests.
    fn build_logs_batch(severities: &[&str]) -> OtapArrowRecords {
        use arrow::array::{RecordBatch, StringArray, UInt16Array};
        use arrow::datatypes::{DataType, Field, Schema};
        use otel_arrow_dfe_pdata::otap::Logs;
        use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

        let ids: Vec<u16> = (0..severities.len() as u16).collect();
        let schema = Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new("severity_text", DataType::Utf8, true),
        ]);
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt16Array::from(ids)),
                Arc::new(StringArray::from(severities.to_vec())),
            ],
        )
        .expect("valid record batch");

        let mut records = OtapArrowRecords::Logs(Logs::default());
        records
            .set(ArrowPayloadType::Logs, record_batch)
            .expect("valid logs batch");
        records
    }

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
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
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
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let result =
            create_wasm_processor(pipeline_ctx, node, Arc::new(node_config), &processor_config);
        assert!(
            matches!(result, Err(ConfigError::InvalidUserConfig { .. })),
            "missing wasm component file should map to InvalidUserConfig"
        );
    }

    /// Build the purpose-built test guest for lifecycle, failure-path, and
    /// resource-boundary tests.
    fn build_test_guest_wasm_for_unit_tests() -> PathBuf {
        let guest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins/test-plugin");
        let status = std::process::Command::new("rustup")
            .current_dir(&guest_dir)
            .args([
                "run",
                "nightly",
                "cargo",
                "build",
                "--release",
                "--target",
                "wasm32-wasip3",
            ])
            .status()
            .expect("failed to spawn rustup to build the guest plugin with nightly cargo");
        assert!(
            status.success(),
            "building the wasm-host test guest failed; ensure the \
             nightly toolchain and wasm32-wasip3 target are installed \
             (`rustup target add --toolchain nightly wasm32-wasip3`)"
        );

        let wasm = guest_dir.join("target/wasm32-wasip3/release/wasm_host_test_guest.wasm");
        assert!(wasm.exists(), "guest wasm not found at {wasm:?}");
        wasm
    }

    /// Scenario: A guest with valid configuration is instantiated via
    /// `WasmProcessor::from_path`. The guest's `initialize` checks the host's
    /// `host-services` ABI version, fetches its config through
    /// `host-services.get-config`, logs a line, and calls `counter-add` once.
    /// Guarantees: Construction succeeds and the guest's `counter-add` call
    /// is observable through the host's guest-counter tracking, proving the
    /// host-services import path works end to end from within `initialize`.
    #[test]
    fn initialize_calls_host_services() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let processor =
            WasmProcessor::from_path(&wasm_path, None, "test_wasm_node".to_string(), metrics)
                .expect("valid guest config should initialize successfully");

        assert_eq!(
            processor
                .store
                .data()
                .guest_counter("test_plugin.initialize"),
            Some(1),
            "guest's counter-add call during initialize should be observable"
        );
    }

    /// Scenario: A guest is constructed and asked for the configuration blob
    /// the host serves it, with a plugin-specific `config` object set.
    /// Guarantees: The guest is served exactly the plugin-owned `config`
    /// sub-object -- host-internal fields such as the `.wasm` file's path on
    /// the host filesystem never reach the guest, keeping host configuration
    /// internals out of the plugin ABI and out of reach of an untrusted
    /// plugin.
    #[test]
    fn guest_config_excludes_host_internal_fields() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let plugin_config = serde_json::json!({ "threshold": 7 });
        let processor = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "test_wasm_node".to_string(),
            metrics,
        )
        .expect("valid guest config");

        let served = processor.store.data().config_for_test();
        assert_eq!(served, r#"{"threshold":7}"#);
        assert!(
            !served.contains("wasm_path"),
            "the host's wasm_path must not be exposed to the guest: {served}"
        );
    }

    /// Scenario: A guest's `initialize` returns an `init-error` because its
    /// plugin-specific configuration requests it (`{"fail_init": true}`).
    /// Guarantees: `WasmProcessor::from_path` (and therefore the processor
    /// factory) surfaces this as `ConfigError::InvalidUserConfig` -- a
    /// pipeline *startup*-time configuration error -- not a panic or a trap
    /// that would only appear once `process` is called.
    #[test]
    fn init_error_prevents_processor_construction() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let plugin_config = serde_json::json!({ "fail_init": true });
        let result = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "test_wasm_node".to_string(),
            metrics,
        );

        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(
                    error.contains("fail_init"),
                    "error message should surface the guest's init-error reason: {error}"
                );
            }
            Ok(_) => panic!("expected InvalidUserConfig from a failed guest initialize, got Ok"),
            Err(other) => {
                panic!("expected InvalidUserConfig from a failed guest initialize, got {other:?}")
            }
        }
    }

    /// Scenario: A guest attempts `std::thread::sleep` from its synchronous
    /// `initialize` lifecycle hook.
    /// Guarantees: Construction fails immediately with a configuration error
    /// instead of blocking the pipeline thread on a Tokio clock wait.
    #[test]
    fn initialize_rejects_suspending_clock_waits() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let plugin_config = serde_json::json!({ "sleep_init": true });

        let result = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "sleep-init-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        );

        match result {
            Err(ConfigError::InvalidUserConfig { error }) => assert!(
                error.contains(crate::host::WASI_CLOCK_WAIT_UNSUPPORTED),
                "initialize clock wait should report the lifecycle restriction: {error}"
            ),
            Ok(_) => panic!("initialize clock wait must prevent processor construction"),
            Err(other) => panic!("expected InvalidUserConfig, got {other:?}"),
        }
    }

    /// Scenario: A guest calls `std::thread::sleep` while processing pdata on
    /// the engine's Tokio current-thread runtime.
    /// Guarantees: The guest wait yields to another task on that same runtime
    /// and processing resumes after the finite wait without deadlocking.
    #[test]
    fn process_clock_wait_yields_on_current_thread_runtime() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let plugin_config = serde_json::json!({ "sleep_process": true });
        let processor = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "sleep-process-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        )
        .expect("process clock waits are allowed after initialization");

        let runtime: TestRuntime<OtapPdata> = TestRuntime::new();
        let wrapper = ProcessorWrapper::local(
            processor,
            test_node("wasm-process-wait"),
            Arc::new(NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN)),
            runtime.config(),
        );
        let peer_task_ran = Arc::new(std::sync::atomic::AtomicBool::new(false));

        runtime
            .set_processor(wrapper)
            .run_test({
                let peer_task_ran = Arc::clone(&peer_task_ran);
                |mut ctx| async move {
                    let peer_task_flag = Arc::clone(&peer_task_ran);
                    let peer_task = tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                        peer_task_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                    });

                    ctx.process(Message::PData(OtapPdata::new(
                        Context::default(),
                        build_logs_batch(&["ERROR", "INFO"]).into(),
                    )))
                    .await
                    .expect("finite guest clock wait should complete");
                    assert!(
                        peer_task_ran.load(std::sync::atomic::Ordering::SeqCst),
                        "the peer task must run before guest processing returns"
                    );
                    peer_task.await.expect("peer task should complete");
                }
            })
            .validate(|_| async move {});

        assert!(
            peer_task_ran.load(std::sync::atomic::Ordering::SeqCst),
            "another task must run while the guest is suspended"
        );
    }

    /// Scenario: A successfully-constructed guest processes several pdata
    /// messages via `run_guest`.
    /// Guarantees: `initialize` (and therefore the test guest's
    /// `test_plugin.initialize` counter-add call) ran exactly once --
    /// its value does not grow across repeated `process` calls, because the
    /// host only invokes `initialize` once, in `from_path`, never from
    /// `run_guest`.
    #[test]
    fn initialize_is_called_exactly_once_per_instance() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let mut processor =
            WasmProcessor::from_path(&wasm_path, None, "test_wasm_node".to_string(), metrics)
                .expect("valid guest config");
        assert_eq!(
            processor
                .store
                .data()
                .guest_counter("test_plugin.initialize"),
            Some(1)
        );

        for _ in 0..5 {
            let batch = build_logs_batch(&["ERROR"]);
            let _ = futures::executor::block_on(processor.run_guest(batch))
                .expect("run_guest should succeed for valid input");
        }

        assert_eq!(
            processor
                .store
                .data()
                .guest_counter("test_plugin.initialize"),
            Some(1),
            "initialize must run exactly once per instance, not once per process() call"
        );
    }

    /// Scenario: A processor is torn down twice over -- `shutdown_guest` is
    /// invoked repeatedly, as happens when an explicit teardown is followed
    /// by the `Drop` impl running.
    /// Guarantees: The guest's `shutdown` export runs exactly once; the
    /// idempotency guard prevents a second invocation on the same store.
    #[test]
    fn shutdown_guest_is_idempotent() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let mut processor =
            WasmProcessor::from_path(&wasm_path, None, "test_wasm_node".to_string(), metrics)
                .expect("valid guest config");
        assert_eq!(
            processor.store.data().guest_counter("test_plugin.shutdown"),
            None,
            "shutdown must not run during construction"
        );

        processor.shutdown_guest();
        assert_eq!(
            processor.store.data().guest_counter("test_plugin.shutdown"),
            Some(1),
            "the first shutdown must invoke the guest export"
        );

        // Simulates `Drop` firing after an explicit teardown.
        processor.shutdown_guest();
        processor.shutdown_guest();
        assert_eq!(
            processor.store.data().guest_counter("test_plugin.shutdown"),
            Some(1),
            "subsequent shutdowns must be no-ops, never re-entering the guest"
        );
    }

    /// Scenario: A guest attempts `std::thread::sleep` from its synchronous
    /// `shutdown` lifecycle hook after the processor loop has ended.
    /// Guarantees: Teardown rejects the wait immediately, marks the instance
    /// poisoned, and does not execute guest code after the rejected wait.
    #[test]
    fn shutdown_rejects_suspending_clock_waits() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let plugin_config = serde_json::json!({ "sleep_shutdown": true });
        let mut processor = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "sleep-shutdown-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        )
        .expect("guest should initialize before its shutdown wait");

        processor.shutdown_guest();

        assert!(
            processor.poisoned,
            "a rejected shutdown wait must poison the trapped instance"
        );
        assert_eq!(
            processor.store.data().guest_counter("test_plugin.shutdown"),
            None,
            "guest code after the rejected shutdown wait must not run"
        );
    }

    /// Scenario: A running plugin node receives the pipeline's `Shutdown`
    /// control message and is then handed more pdata, which is exactly what
    /// the engine does: `Shutdown` starts a *drain*, is broadcast to every
    /// non-receiver node at once so buffering upstream processors can flush,
    /// and the node loop keeps delivering messages until its inbox closes.
    /// Guarantees: The guest's `shutdown` export has NOT run when that later
    /// pdata arrives, and the pdata is still processed correctly. This is the
    /// regression guard for `wit/plugin.wit`'s promise that `shutdown` is
    /// called only after the last `process` call has returned -- calling it
    /// from the `Shutdown` arm would hand post-teardown data to the guest.
    #[test]
    fn shutdown_control_message_does_not_tear_down_the_guest_mid_drain() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);
        let processor =
            WasmProcessor::from_path(&wasm_path, None, "test_wasm_node".to_string(), metrics)
                .expect("valid guest config");

        let runtime: TestRuntime<OtapPdata> = TestRuntime::new();
        let wrapper = ProcessorWrapper::local(
            processor,
            test_node("wasm-drain"),
            Arc::new(NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN)),
            runtime.config(),
        );

        runtime
            .set_processor(wrapper)
            .run_test(|mut ctx| async move {
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: std::time::Instant::now() + Duration::from_secs(5),
                    reason: "pipeline drain".to_string(),
                }))
                .await
                .expect("shutdown control message is accepted");

                // Upstream buffering nodes flush on the same signal, so this
                // is ordinary in-drain traffic, not a protocol violation.
                let records = build_logs_batch(&["ERROR", "INFO", "ERROR"]);
                ctx.process(Message::PData(OtapPdata::new(
                    Context::default(),
                    records.into(),
                )))
                .await
                .expect("pdata arriving during the drain must still be processed");

                let out = ctx.drain_pdata().await;
                assert_eq!(
                    out.len(),
                    1,
                    "the guest must still produce output after the drain signal"
                );
            })
            .validate(|_| async move {});
    }

    /// Scenario: A guest's `initialize` traps (the test plugin panics
    /// when its config sets `panic_init`), rather than returning a structured
    /// `init-error`.
    /// Guarantees: The trap is caught and surfaced as a pipeline *startup*
    /// configuration error -- the same class of failure as a returned
    /// `init-error` -- instead of unwinding into the host or deferring the
    /// failure to the first `process` call.
    #[test]
    fn initialize_trap_prevents_processor_construction() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let plugin_config = serde_json::json!({ "panic_init": true });
        let result = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "test_wasm_node".to_string(),
            metrics,
        );

        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(
                    error.contains("trapped"),
                    "a trapping initialize should be reported as a trap: {error}"
                );
            }
            Ok(_) => panic!("a trapping initialize must not produce a processor"),
            Err(other) => panic!("expected InvalidUserConfig, got {other:?}"),
        }
    }

    /// Scenario: A guest asks its allocator for more linear memory than the
    /// host's per-instance cap allows (the test plugin does this when
    /// its config sets `balloon`, reporting back through an `init-error`
    /// whether the request was refused).
    /// Guarantees: The host's `ResourceLimiter` is actually installed on the
    /// store and refuses growth beyond the guest's memory allowance,
    /// independently of the host-side string-copy limits.
    #[test]
    fn guest_memory_growth_is_capped() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let plugin_config = serde_json::json!({ "balloon": true });
        let result = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "test_wasm_node".to_string(),
            metrics,
        );

        match result {
            Err(ConfigError::InvalidUserConfig { error }) => assert!(
                error.contains("host refused an over-cap allocation"),
                "the guest memory cap must be enforced, got: {error}"
            ),
            Ok(_) => panic!("the balloon plugin always fails initialize by design"),
            Err(other) => panic!("expected InvalidUserConfig, got {other:?}"),
        }
    }

    /// Scenario: Real guest log and counter imports lift strings at the
    /// 16 KiB limit and one byte beyond it.
    /// Guarantees: Boundary calls reach the host, but oversized arguments
    /// trap in canonical lifting rather than being copied then rejected.
    #[test]
    fn guest_string_lifting_is_capped_before_host_imports() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        for flag in ["large_log", "large_counter"] {
            for at_limit in [true, false] {
                let controller_ctx = ControllerContext::new(
                    otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
                );
                let pipeline_ctx =
                    controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
                let config = serde_json::json!({ flag: true, "at_copy_limit": at_limit });
                let result = WasmProcessor::from_path(
                    &wasm_path,
                    Some(&config),
                    "copy-limit-test".to_string(),
                    WasmProcessorAllMetrics::new(&pipeline_ctx),
                );
                if at_limit {
                    let processor = result.expect("16 KiB strings fit the lifting budget");
                    assert_eq!(processor.store.hostcall_fuel(), MAX_GUEST_HOSTCALL_BYTES);
                    assert_eq!(
                        processor
                            .store
                            .data()
                            .guest_counter("test_plugin.initialize"),
                        Some(1),
                    );
                } else {
                    match result {
                        Err(ConfigError::InvalidUserConfig { error }) => assert!(
                            error.contains("fuel allocated for hostcalls has been exhausted"),
                            "{flag} should fail during lifting, got: {error}",
                        ),
                        _ => panic!("{flag} must trap for a 16 KiB + 1 byte string"),
                    }
                }
            }
        }
    }

    /// Scenario: A real guest drains the call-rate bucket and repeatedly
    /// sends 16 KiB counter names without exceeding the per-lift limit.
    /// Guarantees: Aggregate copy work traps on the byte budget, even when
    /// calls would otherwise be discarded by rate or name-length checks.
    #[test]
    fn guest_copy_flood_traps_on_byte_budget() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let config = serde_json::json!({ "copy_flood": true });
        match WasmProcessor::from_path(
            &wasm_path,
            Some(&config),
            "copy-flood-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        ) {
            Err(ConfigError::InvalidUserConfig { error }) => assert!(
                error.contains("string-copy byte budget exceeded"),
                "copy flood must exhaust bytes rather than guest instruction fuel: {error}",
            ),
            _ => panic!("copy flood must prevent processor construction"),
        }
    }

    /// Scenario: A guest loops without terminating inside `process` (the
    /// test plugin does this when its config sets `spin`), consuming its
    /// whole `GUEST_FUEL_PER_CALL` budget.
    /// Guarantees: The call traps and returns an error in bounded time
    /// instead of hanging the pipeline thread forever, and the instance is
    /// marked poisoned -- Wasmtime refuses every later entry into a trapped
    /// instance, so teardown must not try to call the guest's `shutdown`
    /// again and produce a second, misleading error for the same fault.
    #[test]
    fn runaway_guest_exhausts_its_fuel_budget_and_poisons_the_instance() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);

        let plugin_config = serde_json::json!({ "spin": true });
        let mut processor = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "test_wasm_node".to_string(),
            metrics,
        )
        .expect("a spinning guest still initializes cleanly");
        assert!(!processor.poisoned);

        let error = futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR"])))
            .expect_err("an endless guest loop must trap rather than hang");
        assert_eq!(
            error.downcast_ref::<wasmtime::Trap>(),
            Some(&wasmtime::Trap::OutOfFuel),
            "the guest should be stopped by the fuel budget, got: {error:#}"
        );
        assert!(
            processor.poisoned,
            "a trapped instance must be marked unusable"
        );

        // Teardown must not re-enter the trapped instance.
        processor.shutdown_guest();
        assert_eq!(
            processor.store.data().guest_counter("test_plugin.shutdown"),
            None,
            "shutdown must be skipped for a poisoned instance"
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
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
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

    /// Scenario: The `kernel-processor` world's WIT source is inspected for
    /// its declared imports.
    /// Guarantees: The plugin contract itself imports only `otel-kernels` and
    /// `host-services`; ambient WASI support remains an explicit host linker
    /// decision and cannot silently enter the project-owned WIT world.
    #[test]
    fn kernel_processor_world_imports_only_otel_kernels_and_host_services() {
        let wit_source = include_str!("../wit/plugin.wit");
        let world_start = wit_source
            .find("world kernel-processor")
            .expect("kernel-processor world must be declared in plugin.wit");
        let world_block = &wit_source[world_start..];
        let world_end = world_block
            .find('}')
            .expect("kernel-processor world block must be closed");
        let world_block = &world_block[..world_end];

        let imports: Vec<&str> = world_block
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("import "))
            .collect();

        assert_eq!(
            imports,
            vec!["import otel-kernels;", "import host-services;"],
            "kernel-processor must import exactly otel-kernels and host-services, \
             no other ambient capability (no wasi:http, wasi:sockets, filesystem)"
        );

        // No wasi capability strings should appear anywhere in the world
        // block (imports or otherwise), guarding against a future import
        // being added without also updating the assertion above.
        assert!(
            !world_block.contains("wasi:"),
            "kernel-processor world must not import any wasi:* capability"
        );
    }
}

/// Register [`WasmProcessor`] as an OTAP processor factory.
#[otel_arrow_dfe_engine::component_inventory(category = Processor)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static WASM_PROCESSOR_FACTORY: otel_arrow_dfe_engine::ProcessorFactory<OtapPdata> =
    otel_arrow_dfe_engine::ProcessorFactory {
        name: WASM_PROCESSOR_URN,
        create:
            |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig,
             _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
                create_wasm_processor(pipeline, node, node_config, proc_cfg)
            },
        context_declarations: None,
        wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
            WasmProcessorConfig,
        >,
    };
