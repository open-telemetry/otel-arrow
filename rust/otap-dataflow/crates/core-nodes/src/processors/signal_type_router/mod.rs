// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Signal type router processor for OTAP pipelines.
//!
//! Routes OTAP payloads to well-known named output ports based on signal type:
//! `logs`, `metrics`, and `traces`.
//!
//! The router prefers the signal-type-specific named output when that port is
//! connected. If the named port is not wired, it falls back to the node default
//! output when one exists.
//!
//! # Selected-Route Admission
//!
//! After the router picks a named or default route, admission to that selected
//! route is non-blocking:
//!
//! - a writable selected route is forwarded normally
//! - a selected route that is full is rejected immediately with a retryable
//!   route-local NACK
//! - a selected route that is closed is rejected immediately with a retryable
//!   route-local NACK
//!
//! The processor still returns `Ok(())` after those route-local rejections so a
//! blocked signal-specific route does not stall traffic for other signal types.
//!
//! If neither a signal-specific named route nor a default output exists, the
//! message is dropped with the historical routing-failure behavior.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, MessageSourceLocalEffectHandlerExtension, ProcessorFactory,
    RouteAdmission,
};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// URN for the SignalTypeRouter processor
pub const SIGNAL_TYPE_ROUTER_URN: &str = "urn:otel:processor:type_router";

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

    /// Number of log messages NACKed due to route-local rejection.
    #[metric(unit = "{msg}")]
    pub signals_nacked_logs: Counter<u64>,
    /// Number of metric messages NACKed due to route-local rejection.
    #[metric(unit = "{msg}")]
    pub signals_nacked_metrics: Counter<u64>,
    /// Number of trace messages NACKed due to route-local rejection.
    #[metric(unit = "{msg}")]
    pub signals_nacked_traces: Counter<u64>,

    /// Number of log messages rejected because the selected route was full.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_full_logs: Counter<u64>,
    /// Number of metric messages rejected because the selected route was full.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_full_metrics: Counter<u64>,
    /// Number of trace messages rejected because the selected route was full.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_full_traces: Counter<u64>,

    /// Number of log messages rejected because the selected route was closed.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_closed_logs: Counter<u64>,
    /// Number of metric messages rejected because the selected route was closed.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_closed_metrics: Counter<u64>,
    /// Number of trace messages rejected because the selected route was closed.
    #[metric(unit = "{msg}")]
    pub signals_rejected_route_closed_traces: Counter<u64>,

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
    const fn inc_nacked(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_nacked_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_nacked_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_nacked_traces.inc(),
        }
    }
    const fn inc_rejected_route_full(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_rejected_route_full_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_rejected_route_full_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_rejected_route_full_traces.inc(),
        }
    }
    const fn inc_rejected_route_closed(&mut self, st: otap_df_config::SignalType) {
        match st {
            otap_df_config::SignalType::Logs => self.signals_rejected_route_closed_logs.inc(),
            otap_df_config::SignalType::Metrics => self.signals_rejected_route_closed_metrics.inc(),
            otap_df_config::SignalType::Traces => self.signals_rejected_route_closed_traces.inc(),
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

    async fn handle_rejected_route(
        &mut self,
        signal_type: otap_df_config::SignalType,
        port: &str,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
        admission: RouteAdmission<OtapPdata>,
    ) -> Result<(), EngineError> {
        match admission {
            RouteAdmission::Accepted => Ok(()),
            RouteAdmission::RejectedFull(data) => {
                if let Some(m) = self.metrics.as_mut() {
                    m.inc_nacked(signal_type);
                    m.inc_rejected_route_full(signal_type);
                }
                // Selected-route overload is isolated to that one message. The
                // router emits a retryable local NACK and returns success so
                // other signal types can continue flowing.
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("signal_type_router route overload: output port '{port}' is full"),
                        data,
                    ))
                    .await?;
                Ok(())
            }
            RouteAdmission::RejectedClosed(data) => {
                if let Some(m) = self.metrics.as_mut() {
                    m.inc_nacked(signal_type);
                    m.inc_rejected_route_closed(signal_type);
                }
                // A closed selected route follows the same isolation policy as
                // overload: refuse that message locally without stalling the
                // router task.
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!(
                            "signal_type_router route unavailable: output port '{port}' is closed"
                        ),
                        data,
                    ))
                    .await?;
                Ok(())
            }
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

                // Resolve the preferred named route first. Falling back is only
                // allowed when the named port is not wired at all, not when the
                // named route exists but is blocked or closed.
                let desired_port = match st {
                    otap_df_config::SignalType::Traces => PORT_TRACES,
                    otap_df_config::SignalType::Metrics => PORT_METRICS,
                    otap_df_config::SignalType::Logs => PORT_LOGS,
                };

                // Probe wiring first so send failures on the named port stay
                // route-local instead of being misinterpreted as a signal to
                // fall back to the default output.
                let has_port = effect_handler
                    .connected_ports()
                    .iter()
                    .any(|p| p.as_ref() == desired_port);

                if has_port {
                    // Named-route admission is non-blocking for the same
                    // liveness reason as content_router.
                    let admission = effect_handler
                        .try_admit_message_with_source_node_to(desired_port, data)
                        .map_err(EngineError::from)?;
                    match admission {
                        RouteAdmission::Accepted => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_routed_named(st);
                            }
                            Ok(())
                        }
                        rejected => {
                            self.handle_rejected_route(st, desired_port, effect_handler, rejected)
                                .await
                        }
                    }
                } else if let Some(default_port) = effect_handler.default_port() {
                    // Once the default route has been selected, it uses the
                    // same non-blocking admission and route-local rejection
                    // policy as the named route.
                    let admission = effect_handler
                        .try_admit_message_with_source_node_to(default_port.clone(), data)
                        .map_err(EngineError::from)?;
                    match admission {
                        RouteAdmission::Accepted => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.inc_routed_default(st);
                            }
                            Ok(())
                        }
                        rejected => {
                            self.handle_rejected_route(
                                st,
                                default_port.as_ref(),
                                effect_handler,
                                rejected,
                            )
                            .await
                        }
                    }
                } else {
                    // Preserve the historical "no selected route exists"
                    // behavior. This path is distinct from selected-route
                    // overload/closed handling above.
                    match effect_handler.send_message_with_source_node(data).await {
                        Ok(()) => unreachable!(
                            "default route send should not succeed when no default output is selected"
                        ),
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
        use otap_df_channel::error::RecvError;
        use otap_df_channel::mpsc;
        use otap_df_engine::Interests;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::{
            NodeControlMsg, PipelineCompletionMsg, PipelineCompletionMsgReceiver,
            pipeline_completion_msg_channel,
        };
        use otap_df_engine::local::message::LocalSender;
        use otap_df_engine::local::processor::{
            EffectHandler as LocalEffectHandler, Processor as _,
        };
        use otap_df_engine::message::{Message, Sender};
        use otap_df_engine::testing::setup_test_runtime;
        use otap_df_otap::pdata::OtapPdata;
        use otap_df_otap::testing::{TestCallData, next_nack};
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

        fn subscribed_pdata(payload: OtapArrowRecords, upstream_node_id: usize) -> OtapPdata {
            OtapPdata::new_default(payload.into()).test_subscribe_to(
                Interests::NACKS,
                TestCallData::default().into(),
                upstream_node_id,
            )
        }

        async fn expect_nack(
            completion_rx: &mut PipelineCompletionMsgReceiver<OtapPdata>,
            upstream_node_id: usize,
        ) -> NackMsg<OtapPdata> {
            match completion_rx
                .recv()
                .await
                .expect("pipeline-completion channel closed unexpectedly")
            {
                PipelineCompletionMsg::DeliverNack { nack } => {
                    let (node_id, nack) = next_nack(nack).expect("expected nack subscriber");
                    assert_eq!(node_id, upstream_node_id);
                    nack
                }
                other => panic!("expected DeliverNack, got {other:?}"),
            }
        }

        fn signal_name(st: otap_df_config::SignalType) -> &'static str {
            match st {
                otap_df_config::SignalType::Logs => "logs",
                otap_df_config::SignalType::Metrics => "metrics",
                otap_df_config::SignalType::Traces => "traces",
            }
        }

        fn signal_payload(st: otap_df_config::SignalType) -> OtapArrowRecords {
            match st {
                otap_df_config::SignalType::Logs => OtapArrowRecords::Logs(Logs::default()),
                otap_df_config::SignalType::Metrics => {
                    OtapArrowRecords::Metrics(Default::default())
                }
                otap_df_config::SignalType::Traces => OtapArrowRecords::Traces(Default::default()),
            }
        }

        fn signal_named_port(st: otap_df_config::SignalType) -> &'static str {
            match st {
                otap_df_config::SignalType::Logs => PORT_LOGS,
                otap_df_config::SignalType::Metrics => PORT_METRICS,
                otap_df_config::SignalType::Traces => PORT_TRACES,
            }
        }

        async fn flush_metrics(
            router: &mut SignalTypeRouter,
            effect_handler: &mut LocalEffectHandler<OtapPdata>,
            reporter: MetricsReporter,
            telemetry_registry: &TelemetryRegistryHandle,
        ) -> HashMap<String, u64> {
            router
                .process(
                    Message::Control(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: reporter,
                    }),
                    effect_handler,
                )
                .await
                .expect("collect telemetry failed");
            tokio::time::sleep(Duration::from_millis(50)).await;
            collect_metrics_map(telemetry_registry)
        }

        async fn assert_full_rejection_for_route(
            st: otap_df_config::SignalType,
            route_port: &'static str,
            test_name: &'static str,
            upstream_node_id: usize,
        ) {
            let (telemetry_registry, reporter, collector_task) = start_telemetry();

            let controller = ControllerContext::new(telemetry_registry.clone());
            let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
            let node_id = test_node(test_name);

            let mut router =
                SignalTypeRouter::with_pipeline_ctx(pipeline, SignalTypeRouterConfig::default());

            let (tx, _rx) = mpsc::Channel::new(1);
            tx.send(OtapPdata::new_default(signal_payload(st).into()))
                .expect("prefill should occupy the selected route");

            let mut senders = HashMap::new();
            let _ = senders.insert(route_port.into(), Sender::Local(LocalSender::mpsc(tx)));
            let mut eh = LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());
            let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
            eh.set_pipeline_completion_msg_sender(completion_tx);

            router
                .process(
                    Message::PData(subscribed_pdata(signal_payload(st), upstream_node_id)),
                    &mut eh,
                )
                .await
                .expect("full selected route should nack locally");

            let nack = expect_nack(&mut completion_rx, upstream_node_id).await;
            assert_eq!(
                nack.reason,
                format!("signal_type_router route overload: output port '{route_port}' is full")
            );
            assert!(
                !nack.permanent,
                "route-full rejection should remain retryable"
            );
            assert!(matches!(completion_rx.try_recv(), Err(RecvError::Empty)));

            let metrics =
                flush_metrics(&mut router, &mut eh, reporter.clone(), &telemetry_registry).await;
            let signal = signal_name(st);
            assert_eq!(
                metrics
                    .get(format!("signals.received.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                1
            );
            assert_eq!(
                metrics
                    .get(format!("signals.routed.named.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(
                metrics
                    .get(format!("signals.routed.default.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(
                metrics
                    .get(format!("signals.nacked.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                1
            );
            assert_eq!(
                metrics
                    .get(format!("signals.rejected.route.full.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                1
            );
            assert_eq!(
                metrics
                    .get(format!("signals.rejected.route.closed.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(
                metrics
                    .get(format!("signals.dropped.{signal}").as_str())
                    .copied()
                    .unwrap_or(0),
                0
            );

            stop_telemetry(reporter, collector_task);
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

        /// Scenario: the signal-type-specific `logs` route is wired, selected,
        /// and already closed.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-closed telemetry for logs, and does not fall back.
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
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
                eh.set_pipeline_completion_msg_sender(completion_tx);

                let upstream_node_id = 81usize;
                let pdata =
                    subscribed_pdata(OtapArrowRecords::Logs(Logs::default()), upstream_node_id);
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed named route should nack locally");

                let nack = expect_nack(&mut completion_rx, upstream_node_id).await;
                assert_eq!(
                    nack.reason,
                    "signal_type_router route unavailable: output port 'logs' is closed"
                );
                assert!(
                    !nack.permanent,
                    "route-closed rejection should remain retryable"
                );
                assert!(matches!(completion_rx.try_recv(), Err(RecvError::Empty)));

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
                assert_eq!(metrics.get("signals.nacked.logs").copied().unwrap_or(0), 1);
                assert_eq!(
                    metrics
                        .get("signals.rejected.route.full.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    metrics
                        .get("signals.rejected.route.closed.logs")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the signal-type-specific `logs` route is wired, selected,
        /// and already full.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-full telemetry for logs, and does not fall back.
        #[test]
        fn test_metrics_named_logs_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Logs,
                signal_named_port(otap_df_config::SignalType::Logs),
                "signal_router_named_logs_full",
                83,
            )));
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

        /// Scenario: the `logs` named route is absent, the default route is
        /// selected, and that default route is already closed.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
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
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
                eh.set_pipeline_completion_msg_sender(completion_tx);

                let upstream_node_id = 82usize;
                let pdata =
                    subscribed_pdata(OtapArrowRecords::Logs(Logs::default()), upstream_node_id);
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed default route should nack locally");

                let nack = expect_nack(&mut completion_rx, upstream_node_id).await;
                assert_eq!(
                    nack.reason,
                    "signal_type_router route unavailable: output port 'out' is closed"
                );
                assert!(
                    !nack.permanent,
                    "route-closed rejection should remain retryable"
                );
                assert!(matches!(completion_rx.try_recv(), Err(RecvError::Empty)));

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
                assert_eq!(metrics.get("signals.nacked.logs").copied().unwrap_or(0), 1);
                assert_eq!(
                    metrics
                        .get("signals.rejected.route.full.logs")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    metrics
                        .get("signals.rejected.route.closed.logs")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the `logs` named route is absent, the default route is
        /// selected, and that default route is already full.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
        #[test]
        fn test_metrics_default_logs_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Logs,
                "out",
                "signal_router_default_logs_full",
                84,
            )));
        }

        /// Scenario: the selected `logs` route is full while the selected
        /// `metrics` route remains healthy.
        /// Guarantees: route-local rejection for logs does not stall metrics
        /// traffic, and the router stays live for other signal types.
        #[test]
        fn test_full_named_logs_route_does_not_block_healthy_named_metrics_route() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("signal_router_isolation_test");

                let mut router = SignalTypeRouter::with_pipeline_ctx(
                    pipeline,
                    SignalTypeRouterConfig::default(),
                );

                let (tx_logs, _rx_logs) = mpsc::Channel::new(1);
                tx_logs
                    .send(OtapPdata::new_default(
                        signal_payload(otap_df_config::SignalType::Logs).into(),
                    ))
                    .expect("prefill should occupy the blocked logs route");
                let (tx_metrics, rx_metrics) = mpsc::Channel::new(1);

                let mut senders = HashMap::new();
                let _ = senders.insert(PORT_LOGS.into(), Sender::Local(LocalSender::mpsc(tx_logs)));
                let _ = senders.insert(
                    PORT_METRICS.into(),
                    Sender::Local(LocalSender::mpsc(tx_metrics)),
                );
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(4);
                eh.set_pipeline_completion_msg_sender(completion_tx);

                let upstream_node_id = 85usize;
                router
                    .process(
                        Message::PData(subscribed_pdata(
                            signal_payload(otap_df_config::SignalType::Logs),
                            upstream_node_id,
                        )),
                        &mut eh,
                    )
                    .await
                    .expect("blocked logs route should nack locally without failing the router");

                router
                    .process(
                        Message::PData(OtapPdata::new_default(
                            signal_payload(otap_df_config::SignalType::Metrics).into(),
                        )),
                        &mut eh,
                    )
                    .await
                    .expect("healthy metrics route should still be admitted");

                let _healthy = rx_metrics
                    .recv()
                    .await
                    .expect("healthy metrics route should continue receiving traffic");

                let nack = expect_nack(&mut completion_rx, upstream_node_id).await;
                assert_eq!(
                    nack.reason,
                    "signal_type_router route overload: output port 'logs' is full"
                );
                assert!(
                    !nack.permanent,
                    "route-full rejection should remain retryable"
                );
                assert!(matches!(completion_rx.try_recv(), Err(RecvError::Empty)));

                let metrics =
                    flush_metrics(&mut router, &mut eh, reporter.clone(), &telemetry_registry)
                        .await;
                assert_eq!(
                    metrics.get("signals.received.logs").copied().unwrap_or(0),
                    1
                );
                assert_eq!(
                    metrics
                        .get("signals.received.metrics")
                        .copied()
                        .unwrap_or(0),
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
                        .get("signals.routed.named.metrics")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.nacked.logs").copied().unwrap_or(0), 1);
                assert_eq!(
                    metrics
                        .get("signals.rejected.route.full.logs")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.dropped.logs").copied().unwrap_or(0), 0);
                assert_eq!(
                    metrics.get("signals.dropped.metrics").copied().unwrap_or(0),
                    0
                );

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

        /// Scenario: the signal-type-specific `traces` route is wired,
        /// selected, and already closed.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-closed telemetry for traces, and does not fall back.
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
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed named traces route should nack locally");

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
                assert_eq!(m.get("signals.nacked.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.rejected.route.full.traces")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.rejected.route.closed.traces")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the signal-type-specific `traces` route is wired,
        /// selected, and already full.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-full telemetry for traces, and does not fall back.
        #[test]
        fn test_metrics_named_traces_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Traces,
                signal_named_port(otap_df_config::SignalType::Traces),
                "signal_router_named_traces_full",
                86,
            )));
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

        /// Scenario: the `traces` named route is absent, the default route is
        /// selected, and that default route is already closed.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
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
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed default traces route should nack locally");

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
                assert_eq!(m.get("signals.nacked.traces").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.rejected.route.full.traces")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.rejected.route.closed.traces")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.traces").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the `traces` named route is absent, the default route is
        /// selected, and that default route is already full.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
        #[test]
        fn test_metrics_default_traces_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Traces,
                "out",
                "signal_router_default_traces_full",
                87,
            )));
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

        /// Scenario: the signal-type-specific `metrics` route is wired,
        /// selected, and already closed.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-closed telemetry for metrics, and does not fall back.
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
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed named metrics route should nack locally");

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
                assert_eq!(m.get("signals.nacked.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.rejected.route.full.metrics")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.rejected.route.closed.metrics")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the signal-type-specific `metrics` route is wired,
        /// selected, and already full.
        /// Guarantees: the router emits a retryable local NACK, records
        /// route-full telemetry for metrics, and does not fall back.
        #[test]
        fn test_metrics_named_metrics_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Metrics,
                signal_named_port(otap_df_config::SignalType::Metrics),
                "signal_router_named_metrics_full",
                88,
            )));
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

        /// Scenario: the `metrics` named route is absent, the default route is
        /// selected, and that default route is already closed.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
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
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("closed default metrics route should nack locally");

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
                assert_eq!(m.get("signals.nacked.metrics").copied().unwrap_or(0), 1);
                assert_eq!(
                    m.get("signals.rejected.route.full.metrics")
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(
                    m.get("signals.rejected.route.closed.metrics")
                        .copied()
                        .unwrap_or(0),
                    1
                );
                assert_eq!(m.get("signals.dropped.metrics").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        /// Scenario: the `metrics` named route is absent, the default route is
        /// selected, and that default route is already full.
        /// Guarantees: default-route admission follows the same retryable
        /// route-local NACK contract as named-route admission.
        #[test]
        fn test_metrics_default_metrics_full() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(assert_full_rejection_for_route(
                otap_df_config::SignalType::Metrics,
                "out",
                "signal_router_default_metrics_full",
                89,
            )));
        }
    }
}
