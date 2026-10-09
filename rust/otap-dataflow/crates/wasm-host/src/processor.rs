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
//! per-core runtime. Clock reads are available, but suspending clock waits are
//! rejected in every phase. Store-owned state is never shared across threads.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::{Either, select};
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
/// (`initialize` and `process`). Wasmtime charges roughly one unit
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
/// Provisional deadline for one guest lifecycle or process call.
/// Cannot preempt synchronous native host kernels; revisit with resource limits.
const GUEST_CALL_TIMEOUT: Duration = Duration::from_secs(30);
/// Error returned when a caller attempts to re-enter a terminal instance.
const WASM_INSTANCE_POISONED: &str =
    "WASM plugin instance is unavailable after a previous terminal process failure";

async fn with_guest_call_timeout<F: Future>(future: F, timeout: Duration) -> Result<F::Output, ()> {
    // The synchronous factory blocks the pipeline runtime during initialization.
    // futures-timer's independent timer driver keeps this deadline advancing
    // without moving guest execution or store state off the pipeline thread.
    match select(
        Box::pin(future),
        Box::pin(futures_timer::Delay::new(timeout)),
    )
    .await
    {
        Either::Left((output, _)) => Ok(output),
        Either::Right(((), _)) => Err(()),
    }
}

/// Fold one `drain_counter_add_calls()` result into the processor's telemetry
/// counters.
///
/// `value` is guest-supplied, so the cumulative total saturates instead of
/// overflowing. Observing that total also prevents independent report
/// intervals from overflowing a downstream delta accumulator.
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
    metrics.guest_counter_add_value_total = metrics
        .guest_counter_add_value_total
        .saturating_add(activity.value);
    metrics
        .pdata
        .guest_counter_add_value
        .observe(metrics.guest_counter_add_value_total);
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
    guest_call_timeout: Duration,
    // Terminal after a guest-call or output-cleanup failure, including
    // host-detected violations that do not themselves trap Wasmtime.
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
        metrics: WasmProcessorAllMetrics,
    ) -> Result<Self, ConfigError> {
        Self::from_path_with_timeout(
            wasm_path,
            plugin_config,
            node_name,
            metrics,
            GUEST_CALL_TIMEOUT,
        )
    }

    fn from_path_with_timeout(
        wasm_path: &PathBuf,
        plugin_config: Option<&serde_json::Value>,
        node_name: String,
        mut metrics: WasmProcessorAllMetrics,
        guest_call_timeout: Duration,
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
        store.data_mut().begin_guest_call();
        let init_result = futures::executor::block_on(with_guest_call_timeout(
            store.run_concurrent(async |accessor| {
                instance
                    .otel_otap_dataflow_plugin_lifecycle()
                    .call_initialize(accessor)
                    .await
            }),
            guest_call_timeout,
        ));

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

        let init_result = match init_result {
            Ok(result) => result,
            Err(()) => {
                let error =
                    format!("WASM plugin initialization timed out after {guest_call_timeout:?}");
                otel_warn!("wasm_processor.initialize_timeout", error = error.as_str());
                return Err(ConfigError::InvalidUserConfig { error });
            }
        };

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
            guest_call_timeout,
            poisoned: false,
        })
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
        if self.poisoned {
            return Err(wasmtime::Error::msg(WASM_INSTANCE_POISONED));
        }
        self.store.set_fuel(GUEST_FUEL_PER_CALL)?;
        self.store.data_mut().begin_guest_call();
        let input = self.store.data_mut().table.push(HostPdata { otap_batch })?;
        let input_rep = input.rep();

        let call_result = with_guest_call_timeout(
            self.store.run_concurrent(async |accessor| {
                self.instance
                    .otel_otap_dataflow_plugin_processor()
                    .call_process(accessor, input)
                    .await
            }),
            self.guest_call_timeout,
        )
        .await
        .map_err(|()| {
            wasmtime::Error::msg(format!(
                "WASM plugin process timed out after {:?}",
                self.guest_call_timeout
            ))
        })
        .and_then(|result| result)
        .and_then(|result| result);

        let output = match call_result {
            Ok(output) => output,
            Err(err) => {
                // Guest traps and host deadline cancellation are both terminal.
                // A cancelled guest may still have suspended tasks in the store,
                // so the host must never re-enter this instance.
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

        self.finish_guest_call(output)
    }

    fn finish_guest_call(
        &mut self,
        output: Option<wasmtime::component::Resource<HostPdata>>,
    ) -> wasmtime::Result<Option<OtapArrowRecords>> {
        let result = match output {
            Some(handle) => {
                let data = self
                    .store
                    .data_mut()
                    .table
                    .delete(handle)
                    .map_err(|error| {
                        self.poisoned = true;
                        wasmtime::Error::from(error).context("failed to reclaim guest output pdata")
                    })?;
                Some(data.otap_batch)
            }
            None => None,
        };

        if !self.store.data().table.is_empty() {
            self.poisoned = true;
            return Err(wasmtime::Error::msg(
                "guest retained host resources after process returned",
            ));
        }

        Ok(result)
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

    /// Scenario: The host deadline races an operation that never completes,
    /// while initialization is driven without a Tokio runtime.
    /// Guarantees: The host-owned timer expires under the synchronous factory
    /// executor instead of relying on Tokio to advance the deadline.
    #[test]
    fn guest_call_timeout_expires_without_tokio_runtime() {
        let result = futures::executor::block_on(with_guest_call_timeout(
            futures::future::pending::<()>(),
            Duration::from_millis(10),
        ));
        assert!(result.is_err(), "the host deadline must end a pending call");
    }

    fn empty_waitable_plugin_fixture(wait_in_initialize: bool) -> tempfile::NamedTempFile {
        use std::io::Write;

        const EMPTY_WAITABLE_SET_COMPONENT: &str = r#"
            (component
              (import "otel:otap-dataflow-plugin/otel-kernels@0.1.0"
                (instance $kernels
                  (export "pdata" (type (sub resource)))))
              (alias export $kernels "pdata" (type $pdata))
              (core module $libc
                (memory (export "memory") 1))
              (core instance $libc (instantiate $libc))
              (core func $new (canon waitable-set.new))
              (core func $wait
                (canon waitable-set.wait
                  (memory (core memory $libc "memory"))))
              (core module $guest
                (import "" "waitable-set.new" (func $new (result i32)))
                (import "" "waitable-set.wait"
                  (func $wait (param i32 i32) (result i32)))
                (import "libc" "memory" (memory 1))
                (func (export "initialize") (result i32)
                  i32.const 16)
                (func (export "process") (param i32) (result i32)
                  (local $set i32)
                  call $new
                  local.set $set
                  local.get $set
                  i32.const 0
                  call $wait
                  drop
                  unreachable))
              (core instance $guest
                (instantiate $guest
                  (with ""
                    (instance
                      (export "waitable-set.new" (func $new))
                      (export "waitable-set.wait" (func $wait))))
                  (with "libc" (instance $libc))))
              (type $init-error (record (field "message" string)))
              (func $initialize async
                (result (result (error $init-error)))
                (canon lift
                  (core func $guest "initialize")
                  (memory (core memory $libc "memory"))))
              (func $process async (param "data" (own $pdata))
                (result (option (own $pdata)))
                (canon lift
                  (core func $guest "process")
                  (memory (core memory $libc "memory"))))
              (instance $lifecycle
                (export "init-error" (type $init-error))
                (export "initialize" (func $initialize)))
              (instance $processor
                (export "pdata" (type $pdata))
                (export "process" (func $process)))
              (export "otel:otap-dataflow-plugin/lifecycle@0.1.0"
                (instance $lifecycle))
              (export "otel:otap-dataflow-plugin/processor@0.1.0"
                (instance $processor)))
        "#;

        let component = if wait_in_initialize {
            EMPTY_WAITABLE_SET_COMPONENT.replace(
                "i32.const 16",
                "call $new\n i32.const 0\n call $wait\n drop\n unreachable",
            )
        } else {
            EMPTY_WAITABLE_SET_COMPONENT.to_string()
        };
        let mut fixture = tempfile::NamedTempFile::new().expect("create component fixture file");
        fixture
            .write_all(&wat::parse_str(component).expect("valid component WAT"))
            .expect("write component fixture");
        fixture
    }

    /// Scenario: Plugin initialization waits on an empty component-model
    /// waitable set while the synchronous factory runs without a Tokio runtime.
    /// Guarantees: The real initialization call ends with a startup timeout
    /// configuration error, not a guest trap or a successfully constructed node.
    #[test]
    fn initialize_empty_waitable_set_returns_startup_timeout() {
        let fixture = empty_waitable_plugin_fixture(true);
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let result = WasmProcessor::from_path_with_timeout(
            &fixture.path().to_path_buf(),
            None,
            "empty-waitable-initialize-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
            Duration::from_millis(50),
        );
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => assert!(
                error.contains("WASM plugin initialization timed out"),
                "expected the host initialization deadline, got: {error}"
            ),
            Ok(_) => panic!("an indefinitely waiting initializer must not construct a node"),
            Err(error) => panic!("expected a startup configuration error, got: {error}"),
        }
    }

    /// Scenario: A plugin initializes successfully, then its `process` export
    /// waits on an empty component-model waitable set without calling a clock.
    /// Guarantees: `run_guest` reports a process timeout, poisons the instance,
    /// reclaims the input resource, and rejects subsequent processing calls.
    #[tokio::test(flavor = "current_thread")]
    async fn process_empty_waitable_set_times_out_and_poisons_instance() {
        let fixture = empty_waitable_plugin_fixture(false);
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut processor = WasmProcessor::from_path_with_timeout(
            &fixture.path().to_path_buf(),
            None,
            "empty-waitable-process-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
            Duration::from_millis(50),
        )
        .expect("plugin should initialize successfully before processing");

        let error = timeout(
            Duration::from_secs(5),
            processor.run_guest(build_logs_batch(&["ERROR"])),
        )
        .await
        .expect("host deadline must resolve processing before the test watchdog")
        .expect_err("guest process must time out while waiting on an empty set");
        assert!(
            error.to_string().contains("WASM plugin process timed out"),
            "expected a host process timeout rather than a guest trap: {error:#}"
        );
        assert!(
            processor.poisoned,
            "process timeout must poison the instance"
        );
        assert!(
            processor.store.get_fuel().expect("remaining guest fuel") > 0,
            "the host deadline must stop the suspended guest before fuel exhaustion"
        );
        assert!(
            processor.store.data().table.is_empty(),
            "timeout must reclaim the input pdata resource"
        );

        let retry_error = timeout(
            Duration::from_secs(5),
            processor.run_guest(build_logs_batch(&["ERROR"])),
        )
        .await
        .expect("a poisoned instance must reject processing before the test watchdog")
        .expect_err("a timed-out instance must reject subsequent process calls");
        assert_eq!(retry_error.to_string(), WASM_INSTANCE_POISONED);
        assert!(
            processor.store.data().table.is_empty(),
            "a rejected retry must not insert another input resource"
        );
    }

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

    /// Scenario: A guest calls `std::thread::sleep` while processing pdata.
    /// Guarantees: The host rejects the future clock wait immediately and
    /// poisons the instance instead of allowing an unbounded suspension.
    #[test]
    fn process_rejects_suspending_clock_waits() {
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
        .expect("guest should initialize before its process wait");
        let mut processor = processor;
        let error =
            futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR", "INFO"])))
                .expect_err("process clock wait must fail");
        assert!(
            format!("{error:#}").contains(crate::host::WASI_CLOCK_WAIT_UNSUPPORTED),
            "process clock wait should report the capability restriction: {error:#}"
        );
        assert!(
            processor.poisoned,
            "a rejected process wait must poison the trapped instance"
        );
    }

    /// Scenario: The guest reports `u64::MAX`, then adds 1 after reporting;
    /// both snapshots are aggregated before the registry is drained.
    /// Guarantees: Reporting preserves a saturated cumulative total and
    /// registry aggregation neither panics nor wraps across reporting intervals.
    #[test]
    fn guest_counter_value_saturates_across_reporting_snapshots() {
        use otel_arrow_dfe_telemetry::metrics::MetricValue;
        use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

        let registry = otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = WasmProcessorAllMetrics::new(&pipeline_ctx);
        let (snapshots, mut reporter) = MetricsReporter::create_new_and_receiver(2);

        fold_guest_counter_metrics(
            &mut metrics,
            GuestCounterActivity {
                value: u64::MAX,
                ..GuestCounterActivity::default()
            },
        );
        reporter
            .report(&mut metrics.pdata)
            .expect("report first interval");
        fold_guest_counter_metrics(
            &mut metrics,
            GuestCounterActivity {
                value: 1,
                ..GuestCounterActivity::default()
            },
        );
        reporter
            .report(&mut metrics.pdata)
            .expect("report second interval");
        for _ in 0..2 {
            let snapshot = snapshots.try_recv().expect("reported snapshot");
            registry.accumulate_metric_set_snapshot(
                snapshot.key(),
                snapshot.bucket(),
                snapshot.get_metrics(),
            );
        }
        let batch = registry.drain_metric_export_batch();
        let exported = batch
            .metric_sets
            .iter()
            .find(|set| set.descriptor.name == "processor.wasm_processor.pdata")
            .expect("exported processor metrics");
        let value_index = exported
            .descriptor
            .metrics
            .iter()
            .position(|field| field.name == "guest.counter.add.value")
            .expect("guest counter value field");
        assert_eq!(exported.values[value_index], MetricValue::U64(u64::MAX));
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
    /// instead of hanging the pipeline thread forever; the instance is marked
    /// poisoned and the host rejects a later call before re-entering Wasmtime.
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

        let retry_error =
            futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR"])))
                .expect_err("a poisoned instance must reject later process calls");
        assert_eq!(retry_error.to_string(), WASM_INSTANCE_POISONED);
    }

    /// Scenario: A guest stores its owned pdata resource instead of returning
    /// or dropping it before `process` completes.
    /// Guarantees: The host rejects cross-call resource retention, poisons the
    /// instance, and rejects a later call before inserting another batch.
    #[test]
    fn guest_cannot_retain_pdata_across_process_calls() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let plugin_config = serde_json::json!({ "retain_pdata": true });
        let mut processor = WasmProcessor::from_path(
            &wasm_path,
            Some(&plugin_config),
            "retain-pdata-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        )
        .expect("retaining guest should initialize");

        let error =
            futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR", "INFO"])))
                .expect_err("retaining pdata across calls must fail");

        assert!(
            error
                .to_string()
                .contains("retained host resources after process returned"),
            "unexpected retained-resource error: {error:#}"
        );
        assert!(
            processor.poisoned,
            "a retained host resource must make the instance terminal"
        );
        assert!(
            !processor.store.data().table.is_empty(),
            "the test guest must retain the first pdata resource"
        );

        let retry_error =
            futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR"])))
                .expect_err("a poisoned instance must reject later process calls");
        assert_eq!(retry_error.to_string(), WASM_INSTANCE_POISONED);
    }

    /// Scenario: Host-side fault injection supplies a missing output resource
    /// after a guest call; this does not demonstrate a guest-reachable failure.
    /// Guarantees: Cleanup preserves the underlying error, poisons the instance,
    /// and a retry neither resets fuel/budgets nor inserts another pdata.
    #[test]
    fn output_cleanup_failure_poisons_instance() {
        let wasm_path = build_test_guest_wasm_for_unit_tests();
        let controller_ctx = ControllerContext::new(
            otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle::new(),
        );
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut processor = WasmProcessor::from_path(
            &wasm_path,
            None,
            "output-cleanup-test".to_string(),
            WasmProcessorAllMetrics::new(&pipeline_ctx),
        )
        .expect("guest initializes");
        let invalid = wasmtime::component::Resource::<HostPdata>::new_own(u32::MAX);
        let error = processor
            .finish_guest_call(Some(invalid))
            .expect_err("missing output must fail cleanup");
        assert!(
            error
                .to_string()
                .contains("failed to reclaim guest output pdata")
        );
        assert!(matches!(
            error.downcast_ref::<wasmtime::component::ResourceTableError>(),
            Some(wasmtime::component::ResourceTableError::NotPresent)
        ));
        assert!(processor.poisoned);
        processor.store.set_fuel(123).expect("set sentinel fuel");
        processor.store.data_mut().kernel_calls = 7;

        let retry = futures::executor::block_on(processor.run_guest(build_logs_batch(&["ERROR"])))
            .expect_err("terminal instance must reject retry");
        assert_eq!(retry.to_string(), WASM_INSTANCE_POISONED);
        assert_eq!(processor.store.get_fuel().unwrap(), 123);
        assert_eq!(processor.store.data().kernel_calls, 7);
        assert!(processor.store.data().table.is_empty());
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
