// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Signal type router processor for OTAP pipelines.
//!
//! Simplest behavior: pass-through using engine wiring.
//! All signals are forwarded unchanged via the engine-provided default output port
//! (or error if multiple ports are connected without a default).

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{MessageSourceLocalEffectHandlerExtension, ProcessorFactory};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// URN for the SignalTypeRouter processor
pub const SIGNAL_TYPE_ROUTER_URN: &str = "urn:otel:type_router:processor";

/// Well-known output port names for type-based routing
/// Name of the output port used for trace signals
pub const PORT_TRACES: &str = "traces";
/// Name of the output port used for metric signals
pub const PORT_METRICS: &str = "metrics";
/// Name of the output port used for log signals
pub const PORT_LOGS: &str = "logs";

/// Metrics for the SignalTypeRouter processor.
#[metric_set(name = "signal_type_router.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct SignalTypeRouterMetrics {
    /// Number of log messages received by the router.
    #[metric(unit = "{msg}")]
    pub signals_received_logs: Counter<u64>,
    /// Number of metric messages received by the router.
    #[metric(unit = "{msg}")]
    pub signals_received_metrics: Counter<u64>,
    /// Number of trace messages received by the router.
    #[metric(unit = "{msg}")]
    pub signals_received_traces: Counter<u64>,

    /// Number of log messages routed to a named port.
    #[metric(unit = "{msg}")]
    pub signals_routed_named_logs: Counter<u64>,
    /// Number of metric messages routed to a named port.
    #[metric(unit = "{msg}")]
    pub signals_routed_named_metrics: Counter<u64>,
    /// Number of trace messages routed to a named port.
    #[metric(unit = "{msg}")]
    pub signals_routed_named_traces: Counter<u64>,

    /// Number of log messages routed via the default port.
    #[metric(unit = "{msg}")]
    pub signals_routed_default_logs: Counter<u64>,
    /// Number of metric messages routed via the default port.
    #[metric(unit = "{msg}")]
    pub signals_routed_default_metrics: Counter<u64>,
    /// Number of trace messages routed via the default port.
    #[metric(unit = "{msg}")]
    pub signals_routed_default_traces: Counter<u64>,

    /// Number of log messages dropped due to routing failure.
    #[metric(unit = "{msg}")]
    pub signals_dropped_logs: Counter<u64>,
    /// Number of metric messages dropped due to routing failure.
    #[metric(unit = "{msg}")]
    pub signals_dropped_metrics: Counter<u64>,
    /// Number of trace messages dropped due to routing failure.
    #[metric(unit = "{msg}")]
    pub signals_dropped_traces: Counter<u64>,
}

impl SignalTypeRouterMetrics {
    const fn inc_received(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_received_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_received_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_received_traces.inc(),
        }
    }
    const fn inc_routed_named(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_routed_named_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_routed_named_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_routed_named_traces.inc(),
        }
    }
    const fn inc_routed_default(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_routed_default_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_routed_default_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_routed_default_traces.inc(),
        }
    }
    const fn inc_dropped(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_dropped_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_dropped_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_dropped_traces.inc(),
        }
    }
}

/// Minimal configuration for the SignalTypeRouter processor
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalTypeRouterConfig {}

/// The SignalTypeRouter processor (local, !Send)
pub struct SignalTypeRouter {
    /// Router configuration (currently unused, kept for forward compatibility)
    #[allow(dead_code)]
    config: SignalTypeRouterConfig,
    /// Telemetry metrics for this router (optional when constructed without PipelineContext)
    metrics: Option<MetricSet<SignalTypeRouterMetrics>>,
}

impl SignalTypeRouter {
    /// Creates a new SignalTypeRouter with the given configuration
    #[must_use]
    pub const fn new(config: SignalTypeRouterConfig) -> Self {
        Self {
            config,
            metrics: None,
        }
    }

    /// Creates a new SignalTypeRouter with metrics registered via PipelineContext
    #[must_use]
    pub fn with_pipeline_ctx(
        pipeline_ctx: PipelineContext,
        config: SignalTypeRouterConfig,
    ) -> Self {
        let metrics = pipeline_ctx.register_metrics::<SignalTypeRouterMetrics>();
        Self {
            config,
            metrics: Some(metrics),
        }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(ctrl) => {
                if let NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } = ctrl
                {
                    if let Some(m) = self.metrics.as_mut() {
                        let _ = metrics_reporter.report(m);
                    }
                }
                Ok(())
            }
            Message::PData(data) => {
                let st = data.signal_type();
                if let Some(m) = self.metrics.as_mut() {
                    m.inc_received(st);
                }

                // Determine desired output port by signal type
                let desired_port = match st {
                    otap_df_config::SignalType::Traces => PORT_TRACES,
                    otap_df_config::SignalType::Metrics => PORT_METRICS,
                    otap_df_config::SignalType::Logs => PORT_LOGS,
                };

                // Probe connections to decide if named route exists (avoid falling back on unrelated errors)
                let has_port = effect_handler
                    .connected_ports()
                    .iter()
                    .any(|p| p.as_ref() == desired_port);

                if has_port {
                    match effect_handler
                        .send_message_with_source_node_to(desired_port, data)
                        .await
                    {
                        Ok(()) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_routed_named(st);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_dropped(st);
                            }
                            Err(e.into())
                        }
                    }
                } else {
                    match effect_handler.send_message_with_source_node(data).await {
                        Ok(()) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_routed_default(st);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_dropped(st);
                            }
                            Err(e.into())
                        }
                    }
                }
            }
        }
    }
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_signal_type_router(
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    // Deserialize the (currently empty) router configuration
    let router_config: SignalTypeRouterConfig = serde_json::from_value(node_config.config.clone())
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse SignalTypeRouter configuration: {e}"),
        })?;

    // Create the router processor
    let router = SignalTypeRouter::new(router_config);

    // Create NodeUserConfig and wrap as local processor
    let user_config = Arc::new(NodeUserConfig::new_processor_config(SIGNAL_TYPE_ROUTER_URN));

    Ok(ProcessorWrapper::local(
        router,
        node,
        user_config,
        processor_config,
    ))
}

/// Register SignalTypeRouter as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static SIGNAL_TYPE_ROUTER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: SIGNAL_TYPE_ROUTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig| {
        // Deserialize the (currently empty) router configuration
        let router_config: SignalTypeRouterConfig =
            serde_json::from_value(node_config.config.clone()).map_err(|e| {
                ConfigError::InvalidUserConfig {
                    error: format!("Failed to parse SignalTypeRouter configuration: {e}"),
                }
            })?;

        // Create the router with metrics registered via PipelineContext
        let router = SignalTypeRouter::with_pipeline_ctx(pipeline, router_config);

        Ok(ProcessorWrapper::local(router, node, node_config, proc_cfg))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<SignalTypeRouterConfig>,
};

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::testing::{processor::TestRuntime, test_node};
    use otap_df_pdata::otap::{Logs, OtapArrowRecords};
    use serde_json::json;

    #[test]
    fn test_config_deserialization_defaults() {
        let config_json = json!({});
        let _cfg: SignalTypeRouterConfig = serde_json::from_value(config_json).unwrap();
    }

    #[test]
    fn test_factory_creation_ok() {
        let config = json!({});
        let processor_config = ProcessorConfig::new("test_router");
        let mut node_config = NodeUserConfig::new_processor_config(SIGNAL_TYPE_ROUTER_URN);
        node_config.config = config;
        let result = create_signal_type_router(
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
            &processor_config,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_creation_bad_config() {
        // An invalid type (e.g., number instead of object) should error
        let config = json!(42);
        let processor_config = ProcessorConfig::new("test_router");
        let mut node_config = NodeUserConfig::new_processor_config(SIGNAL_TYPE_ROUTER_URN);
        node_config.config = config;
        let result = create_signal_type_router(
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
            &processor_config,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_process_messages_pass_through() {
        use otap_df_config::node::NodeUserConfig;
        use std::sync::Arc;

        let test_runtime = TestRuntime::new();
        let user_cfg = Arc::new(NodeUserConfig::new_processor_config("sig_router_test"));
        let wrapper = ProcessorWrapper::local(
            SignalTypeRouter::new(SignalTypeRouterConfig::default()),
            test_node(test_runtime.config().name.clone()),
            user_cfg,
            test_runtime.config(),
        );

        let validation = test_runtime.set_processor(wrapper).run_test(|mut ctx| {
            Box::pin(async move {
                // Control message is handled and produces no output
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("control processing failed");
                assert!(ctx.drain_pdata().await.is_empty());

                // Data message is forwarded
                let data = OtapArrowRecords::Logs(Logs::default());
                ctx.process(Message::data_msg(OtapPdata::new_default(data.into())))
                    .await
                    .expect("data processing failed");
                let forwarded = ctx.drain_pdata().await;
                assert_eq!(forwarded.len(), 1);
            })
        });

        // No-op validation closure
        validation.validate(|_| async {});
    }

    // -----------------------
    // Telemetry metrics tests
    // -----------------------

    mod telemetry {
        use super::*;
        use crate::pdata::OtapPdata;
        use otap_df_channel::mpsc;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::NodeControlMsg;
        use otap_df_engine::local::message::LocalSender;
        use otap_df_engine::local::processor::{
            EffectHandler as LocalEffectHandler, Processor as _,
        };
        use otap_df_engine::message::{Message, Sender};
        use otap_df_engine::testing::setup_test_runtime;
        use otap_df_pdata::otap::{Logs, OtapArrowRecords};
        use otap_df_telemetry::InternalTelemetrySystem;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;
        use std::collections::HashMap;
        use std::time::Duration;
        use tokio::task::JoinHandle;

        fn collect_metrics_map(
            telemetry_registry: &TelemetryRegistryHandle,
        ) -> HashMap<String, u64> {
            let mut out = HashMap::new();
            telemetry_registry.visit_current_metrics(|_desc, _attrs, iter| {
                for (field, value) in iter {
                    let _ = out.insert(field.name.to_string(), value.to_u64_lossy());
                }
            });
            out
        }

        // Helper to start/stop telemetry collection on the local task set.
        // Returns the telemetry registry, a cloneable reporter, and the spawned collector task handle.
        fn start_telemetry() -> (TelemetryRegistryHandle, MetricsReporter, JoinHandle<()>) {
            let telemetry = InternalTelemetrySystem::default();
            let telemetry_registry = telemetry.registry();
            let reporter = telemetry.reporter();
            let collector_task = tokio::task::spawn_local(async move {
                let collector = telemetry.collector();
                let _ = collector.run_collection_loop().await;
            });
            (telemetry_registry, reporter, collector_task)
        }

        // Stops telemetry collection by dropping the reporter and aborting the collector task.
        fn stop_telemetry(reporter: MetricsReporter, collector_task: JoinHandle<()>) {
            drop(reporter);
            collector_task.abort();
        }

        #[test]
        fn test_metrics_named_logs_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                // Telemetry setup (telemetry registry + collector + reporter)
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                // Pipeline + node context
                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_success");

                // Router with metrics
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                // Effect handler with a logs named port
                let (tx_logs, rx_logs) = mpsc::Channel::new(4);
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_LOGS.into(), Sender::Local(LocalSender::mpsc(tx_logs)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                // Send a logs pdata -> should route to named port
                let pdata = OtapPdata::new_default(OtapArrowRecords::Logs(Logs::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed processing logs");

                // Ensure message reached the named port
                let _received = rx_logs.recv().await.expect("no message on logs port");

                // Flush metrics via CollectTelemetry
                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");

                // Allow collector to accumulate snapshot
                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(
                    metrics.get("signals.received.logs").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.named.logs")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.default.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 0);

                // shutdown collector
                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_named_logs_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_failure");

                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                // Named logs port but drop receiver to force send error
                let (tx_logs, rx_logs) = mpsc::Channel::new(1);
                drop(rx_logs); // close to trigger SendError::Closed
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_LOGS.into(), Sender::Local(LocalSender::mpsc(tx_logs)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata = OtapPdata::new_default(OtapArrowRecords::Logs(Logs::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(res.is_err(), "expected send failure on closed named port");

                // Flush metrics
                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(
                    metrics.get("signals.received.logs").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.named.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.default.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_logs_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_success");

                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                // Only a single output port (non-named for logs); default path should be used
                let (tx_out, rx_out) = mpsc::Channel::new(2);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx_out)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata = OtapPdata::new_default(OtapArrowRecords::Logs(Logs::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed processing logs on default path");

                // Ensure message was sent on default port
                let _ = rx_out.recv().await.expect("no message on default port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(
                    metrics.get("signals.received.logs").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.named.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.default.logs")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_logs_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_failure");

                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                // Single default output port but drop receiver to force send error
                let (tx_out, rx_out) = mpsc::Channel::new(1);
                drop(rx_out);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx_out)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata = OtapPdata::new_default(OtapArrowRecords::Logs(Logs::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(
                    res.is_err(),
                    "expected failure on default route when receiver closed"
                );

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(
                    metrics.get("signals.received.logs").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.named.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    metrics
                        .get("signals.routed.default.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }

        // -------- Traces (spans) tests --------
        #[test]
        fn test_metrics_named_traces_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_traces_success");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(2);
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_TRACES.into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Traces(Default::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed processing traces");
                let _ = rx.recv().await.expect("no message on traces port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.traces").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    m.get("signals.routed.default.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_named_traces_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_traces_failure");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(1);
                drop(rx);
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_TRACES.into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Traces(Default::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(res.is_err(), "expected failure on named traces port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_traces_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_traces_success");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(2);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Traces(Default::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed on default traces path");
                let _ = rx.recv().await.expect("no message on default port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.traces").copied().unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_traces_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_traces_failure");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(1);
                drop(rx);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Traces(Default::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(res.is_err(), "expected failure on default traces route");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.traces").copied().unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }

        // -------- Metrics tests --------
        #[test]
        fn test_metrics_named_metrics_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_metrics_success");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(2);
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_METRICS.into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Metrics(Default::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed processing metrics");
                let _ = rx.recv().await.expect("no message on metrics port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.metrics").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    m.get("signals.routed.default.metrics")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_named_metrics_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_named_metrics_failure");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(1);
                drop(rx);
                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_METRICS.into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Metrics(Default::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(res.is_err(), "expected failure on named metrics port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.metrics").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.metrics")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_metrics_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_metrics_success");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(2);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Metrics(Default::default()).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed on default metrics path");
                let _ = rx.recv().await.expect("no message on default port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.metrics").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.metrics")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_default_metrics_failure() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_default_metrics_failure");
                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx, rx) = mpsc::Channel::new(1);
                drop(rx);
                let mut senders = HashMap::new();
                let _ = senders.insert("out".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let pdata =
                    OtapPdata::new_default(OtapArrowRecords::Metrics(Default::default()).into());
                let res = router.process(Message::PData(pdata), &mut eh).await;
                assert!(res.is_err(), "expected failure on default metrics route");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");
                tokio::time::sleep(Duration::from_millis(50)).await;

                let m = collect_metrics_map(&telemetry_registry);
                assert_eq!(m.get("signals.received.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.routed.named.metrics").copied().unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.routed.default.metrics")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 1);

                stop_telemetry(reporter, collector_task);
            }));
        }
    }
}
