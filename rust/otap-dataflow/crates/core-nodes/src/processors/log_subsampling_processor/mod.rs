//! Log subsampling processor.
//!
//! Reduces log volume by discarding a portion of incoming log records
//! according to a configurable subsampling policy. Non-log signals
//! (metrics and traces) pass through unchanged.
//!
//! See `README.md` in this module for full design documentation.

mod config;
mod metrics;
mod policy;

use self::config::Config;
use self::metrics::LogSubsamplingMetrics;
use self::policy::SubsamplingPolicy;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap::OTAP_PROCESSOR_FACTORIES;
use otap::pdata::OtapPdata;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::{Error as EngineError, ProcessorErrorKind, format_error_sources};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// URN for the log subsampling processor.
const LOG_SUBSAMPLING_PROCESSOR_URN: &str = "urn:otel:log_subsampling:processor";

/// Log subsampling processor.
///
/// Reduces log volume by discarding a portion of incoming log records
/// according to a configurable subsampling policy (zip or ratio).
struct LogSubsamplingProcessor {
    /// Runtime subsampling policy state.
    policy: SubsamplingPolicy,
    /// Telemetry metrics.
    metrics: MetricSet<LogSubsamplingMetrics>,
    /// Timer interval for zip sampling (None for ratio).
    timer_interval: Option<Duration>,
    /// Whether the periodic timer has been started.
    timer_started: bool,
}

impl LogSubsamplingProcessor {
    /// Creates a new processor from configuration.
    fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;

        let timer_interval = match &config.policy {
            config::Policy::Zip(zip) => Some(zip.interval),
            config::Policy::Ratio(_) => None,
        };
        let policy = SubsamplingPolicy::from_config(&config.policy);
        let metrics = pipeline_ctx.register_metrics::<LogSubsamplingMetrics>();

        Ok(Self {
            policy,
            metrics,
            timer_interval,
            timer_started: false,
        })
    }

    /// Ensures the periodic timer is started for zip sampling.
    async fn ensure_timer(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        if self.timer_started {
            return Ok(());
        }
        if let Some(interval) = self.timer_interval {
            let _handle = effect_handler.start_periodic_timer(interval).await?;
            self.timer_started = true;
        }
        Ok(())
    }

    /// Processes a log payload: compute to_keep, slice or ack.
    async fn process_logs(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        let total = pdata.num_items();

        // Update consumed metric
        self.metrics.log_signals_consumed.add(total as u64);

        // Handle empty batch
        if total == 0 {
            self.metrics.batches_fully_dropped.inc();
            effect_handler.notify_ack(AckMsg::new(pdata)).await?;
            return Ok(());
        }

        let to_keep = self.policy.compute_to_keep(total);
        let dropped = total - to_keep;
        self.metrics.log_signals_dropped.add(dropped as u64);

        if to_keep == 0 {
            // All records dropped, ack immediately
            self.metrics.batches_fully_dropped.inc();
            effect_handler.notify_ack(AckMsg::new(pdata)).await?;
            return Ok(());
        }

        if to_keep == total {
            // All records kept, forward unchanged
            effect_handler.send_message_with_source_node(pdata).await?;
            return Ok(());
        }

        // Slice the root Logs record batch
        let (context, payload) = pdata.into_parts();
        let mut records: OtapArrowRecords =
            payload
                .try_into()
                .map_err(|e: otap_df_pdata::error::Error| {
                    let source_detail = format_error_sources(&e);
                    EngineError::ProcessorError {
                        processor: effect_handler.processor_id(),
                        kind: ProcessorErrorKind::Other,
                        error: format!("failed to convert payload to arrow records: {e}"),
                        source_detail,
                    }
                })?;

        slice_logs(&mut records, to_keep);

        let sliced_pdata = OtapPdata::new(context, OtapPayload::OtapArrowRecords(records));
        effect_handler
            .send_message_with_source_node(sliced_pdata)
            .await?;

        Ok(())
    }
}

/// Slices the root `Logs` record batch to keep only the first `to_keep` rows.
///
/// Child batches (ResourceAttrs, ScopeAttrs, LogAttrs) are left unchanged.
/// This is a zero-copy operation (RecordBatch::slice adjusts buffer offsets).
fn slice_logs(records: &mut OtapArrowRecords, to_keep: usize) {
    if let Some(batch) = records.get(ArrowPayloadType::Logs) {
        let sliced = batch.slice(0, to_keep);
        records.set(ArrowPayloadType::Logs, sliced);
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for LogSubsamplingProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        // Ensure timer is started on first message (for zip sampling)
        self.ensure_timer(effect_handler).await?;

        match msg {
            Message::PData(pdata) => {
                match pdata.signal_type() {
                    SignalType::Logs => self.process_logs(pdata, effect_handler).await,
                    // Metrics and traces pass through unchanged
                    SignalType::Metrics | SignalType::Traces => {
                        effect_handler.send_message_with_source_node(pdata).await?;
                        Ok(())
                    }
                }
            }
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::TimerTick {} => {
                    self.policy.reset();
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    let _ = metrics_reporter.report(&mut self.metrics);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. }
                | NodeControlMsg::Config { .. }
                | NodeControlMsg::Ack(_)
                | NodeControlMsg::Nack(_)
                | NodeControlMsg::DelayedData { .. } => Ok(()),
            },
        }
    }
}

// ==================== Factory Registration ====================

/// Creates a new [`LogSubsamplingProcessor`] from pipeline configuration.
fn create_log_subsampling_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        LogSubsamplingProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register the log subsampling processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
static LOG_SUBSAMPLING_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: LOG_SUBSAMPLING_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_log_subsampling_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    };

#[cfg(test)]
mod tests {
    use super::*;
    use otap::pdata::Context;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime};
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::encode::encode_logs_otap_batch;
    use otap_df_pdata::testing::fixtures::logs_with_varying_attributes_and_properties;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message as _;
    use std::future::Future;

    /// Helper: create a processor wrapped in TestRuntime, run a scenario, validate.
    fn run_processor_test<F, Fut>(config_json: Value, scenario: F)
    where
        F: FnOnce(TestContext<OtapPdata>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            LOG_SUBSAMPLING_PROCESSOR_URN,
        ));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let processor = ProcessorWrapper::local(
            LogSubsamplingProcessor::from_config(pipeline_ctx, &config_json).expect("valid config"),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario)
            .validate(|_ctx| async {});
    }

    /// Helper: create OtapPdata with N log records (Arrow format).
    fn make_log_pdata_arrow(n: usize) -> OtapPdata {
        let logs_data = logs_with_varying_attributes_and_properties(n);
        let records = encode_logs_otap_batch(&logs_data).expect("encode");
        OtapPdata::new(Context::default(), OtapPayload::OtapArrowRecords(records))
    }

    /// Helper: create OtapPdata with traces (OTLP bytes).
    fn make_trace_pdata() -> OtapPdata {
        use otap_df_pdata::proto::opentelemetry::trace::v1::{
            ResourceSpans, ScopeSpans, Span, TracesData,
        };
        let traces_data = TracesData {
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut bytes = vec![];
        traces_data.encode(&mut bytes).expect("encode");
        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportTracesRequest(bytes.into()).into(),
        )
    }

    /// Helper: create OtapPdata with metrics (OTLP bytes).
    fn make_metrics_pdata() -> OtapPdata {
        use otap_df_pdata::proto::opentelemetry::metrics::v1::{
            Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        };
        let metrics_data = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "test".to_string(),
                        data: Some(
                            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Gauge(
                                Gauge {
                                    data_points: vec![NumberDataPoint::default()],
                                },
                            ),
                        ),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut bytes = vec![];
        metrics_data.encode(&mut bytes).expect("encode");
        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportMetricsRequest(bytes.into()).into(),
        )
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_zip_basic_flow() {
        let config = serde_json::json!({
            "policy": {
                "zip": {
                    "interval": "60s",
                    "max_items": 20
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Send 10 logs (within budget of 20)
                let pdata = make_log_pdata_arrow(10);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "all 10 should be forwarded");
                assert_eq!(msgs[0].num_items(), 10);

                // Send 15 more logs (exceeds remaining budget of 10)
                let pdata = make_log_pdata_arrow(15);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "partial batch should be forwarded");
                assert_eq!(msgs[0].num_items(), 10, "only 10 remaining budget");

                // Send 5 more (budget exhausted, should be acked/dropped)
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 0, "budget exhausted, nothing forwarded");
            })
        });
    }

    #[test]
    fn test_ratio_basic_flow() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Send 100 logs, expect 10 (1:10 ratio)
                let pdata = make_log_pdata_arrow(100);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].num_items(), 10);
            })
        });
    }

    #[test]
    fn test_pass_through_traces() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = make_trace_pdata();
                let original_items = pdata.num_items();
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "traces should pass through");
                assert_eq!(msgs[0].num_items(), original_items);
            })
        });
    }

    #[test]
    fn test_pass_through_metrics() {
        let config = serde_json::json!({
            "policy": {
                "ratio": {
                    "emit": 1,
                    "out_of": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let pdata = make_metrics_pdata();
                let original_items = pdata.num_items();
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "metrics should pass through");
                assert_eq!(msgs[0].num_items(), original_items);
            })
        });
    }

    #[test]
    fn test_timer_tick_resets_zip_counter() {
        let config = serde_json::json!({
            "policy": {
                "zip": {
                    "interval": "60s",
                    "max_items": 10
                }
            }
        });

        run_processor_test(config, |mut ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Fill the budget
                let pdata = make_log_pdata_arrow(10);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0].num_items(), 10);

                // Budget exhausted
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 0, "budget exhausted");

                // Timer tick resets the counter
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("timer tick");

                // Now we have budget again
                let pdata = make_log_pdata_arrow(5);
                ctx.process(Message::PData(pdata)).await.expect("process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1, "budget restored after timer tick");
                assert_eq!(msgs[0].num_items(), 5);
            })
        });
    }
}
