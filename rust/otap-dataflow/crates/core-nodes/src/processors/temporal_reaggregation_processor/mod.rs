// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Temporal reaggregation processor for OTAP metrics.
//!
//! This processor decreases telemetry volume by reaggregating metrics collected
//! at a higher frequency into a lower one.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::views::otap::OtapMetricsView;
use otap_df_pdata::views::otap::common::OtapAttributeView;
use otap_df_telemetry::metrics::MetricSet;

mod accumulator;
mod config;
mod data_points;
mod identity;
mod metrics;

use self::accumulator::MetricAggregator;
use self::config::Config;
use self::identity::{
    AttributeHashBuffer, DataPointIdAssigner, MetricId, MetricIdAssigner, ResourceId,
    ResourceIdAssigner, ScopeId, ScopeIdAssigner, StreamId, U16IdAssigner, U32IdAssigner,
};
use self::metrics::TemporalReaggregationMetrics;

/// The URN for the temporal reaggregation processor.
pub const TEMPORAL_REAGGREGATION_PROCESSOR_URN: &str = "urn:otel:processor:temporal_reaggregation";

/// The temporal reaggregation processor.
pub struct TemporalReaggregationProcessor {
    metrics: MetricSet<TemporalReaggregationMetrics>,
    collection_period: Duration,
    /// Whether the periodic flush timer has been started.
    timer_started: bool,
    /// Reusable buffer for attribute hashing. Stored as `'static` between
    /// calls and recycled to the local lifetime via [`AttributeHashBuffer::recycle`]
    /// at the start of each `process()` invocation.
    attr_buf: AttributeHashBuffer<OtapAttributeView<'static>>,
    /// Accumulates metrics data over the collection interval.
    accumulator: MetricAggregator,
    // resource_ids: ResourceIdAssigner,
    // scope_ids: ScopeIdAssigner,
    // metric_ids: MetricIdAssigner,
    // number_ids: DataPointIdAssigner,
    // histogram_ids: DataPointIdAssigner,
    // exp_histogram_ids: DataPointIdAssigner,
    // summary_ids: DataPointIdAssigner,
}

/// Factory function to create a [`TemporalReaggregationProcessor`].
pub fn create_temporal_reaggregation_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        TemporalReaggregationProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register the temporal reaggregation processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: TEMPORAL_REAGGREGATION_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_temporal_reaggregation_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

impl TemporalReaggregationProcessor {
    /// Creates a new processor from a configuration JSON value.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<TemporalReaggregationMetrics>();
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;
        Ok(Self {
            metrics,
            collection_period: config.period,
            timer_started: false,
            attr_buf: AttributeHashBuffer::new(),
            accumulator: MetricAggregator::new(),
            // resource_ids: ResourceIdAssigner::new(),
            // scope_ids: ScopeIdAssigner::new(),
            // metric_ids: MetricIdAssigner::new(),
            // number_ids: DataPointIdAssigner::new(),
            // histogram_ids: DataPointIdAssigner::new(),
            // exp_histogram_ids: DataPointIdAssigner::new(),
            // summary_ids: DataPointIdAssigner::new(),
        })
    }

    /// Starts the periodic flush timer if it has not already been started.
    async fn ensure_timer_started(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if !self.timer_started {
            let _handle = effect_handler
                .start_periodic_timer(self.collection_period)
                .await?;
            self.timer_started = true;
        }
        Ok(())
    }

    /// Flush accumulated metrics and send downstream.
    async fn flush(
        &mut self,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if let Some(batch) = self.accumulator.flush() {
            let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(batch));
            effect_handler.send_message_with_source_node(pdata).await?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for TemporalReaggregationProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::PData(pdata) => {
                self.ensure_timer_started(effect_handler).await?;

                match pdata.signal_type() {
                    SignalType::Metrics => {
                        // Recycle the attribute hash buffer for this call.
                        let attr_buf =
                            std::mem::replace(&mut self.attr_buf, AttributeHashBuffer::new());
                        let mut attr_buf: AttributeHashBuffer<OtapAttributeView<'_>> =
                            attr_buf.recycle();

                        // Build a view over the incoming metrics and ingest.
                        if let OtapPayload::OtapArrowRecords(ref records) = *pdata.payload_ref() {
                            if let Ok(view) = OtapMetricsView::try_from(records) {
                                self.accumulator.ingest(view, &mut attr_buf);
                            }
                        }

                        // Recycle back to 'static for storage.
                        self.attr_buf = attr_buf.recycle();

                        // TODO (Stage 3b): Check stream overflow and flush if needed.
                    }
                    // Non-metrics signals pass through unchanged.
                    SignalType::Logs | SignalType::Traces => {
                        effect_handler.send_message_with_source_node(pdata).await?;
                    }
                }
                Ok(())
            }
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::TimerTick {} => {
                    self.flush(effect_handler).await?;
                    self.metrics.flushes_timer.add(1);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. } => {
                    self.flush(effect_handler).await?;
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    _ = metrics_reporter.report(&mut self.metrics);
                    Ok(())
                }
                _ => Ok(()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::error::Error as ConfigError;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_pdata::testing::fixtures::DataGenerator;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;

    #[test]
    fn test_default_config_parsing() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.period, Duration::from_secs(60));
    }

    #[test]
    fn test_custom_config_parsing() {
        let config: Config = serde_json::from_value(json!({
            "period": "30s",
        }))
        .unwrap();
        assert_eq!(config.period, Duration::from_secs(30));
    }

    #[test]
    fn test_passthrough_logs() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_test_pdata();

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process message");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "expected exactly one forwarded message");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_passthrough_traces() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_traces_pdata();

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process message");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "expected exactly one forwarded message");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_passthrough_multiple_non_metrics() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let logs_pdata = create_test_pdata();
                ctx.process(Message::PData(logs_pdata))
                    .await
                    .expect("process logs");

                let traces_pdata = create_traces_pdata();
                ctx.process(Message::PData(traces_pdata))
                    .await
                    .expect("process traces");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 2, "expected two forwarded messages");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_metrics_are_buffered_not_forwarded() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_metrics_pdata();
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process metrics");

                let output = ctx.drain_pdata().await;
                assert!(
                    output.is_empty(),
                    "metrics should be buffered, not forwarded immediately"
                );
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_shutdown_with_no_buffered_data() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let deadline = std::time::Instant::now() + Duration::from_secs(1);
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline,
                    reason: "test".into(),
                }))
                .await
                .expect("shutdown should succeed");

                let output = ctx.drain_pdata().await;
                assert!(output.is_empty(), "no data should be emitted on shutdown");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_timer_tick_with_no_data() {
        let (rt, proc) = try_create_processor(json!({})).unwrap();

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("timer tick should succeed");

                let output = ctx.drain_pdata().await;
                assert!(
                    output.is_empty(),
                    "timer tick with no data should emit nothing"
                );
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_factory_creation() {
        test_config(json!({ "period": "5s" }), |result| {
            assert!(
                result.is_ok(),
                "factory should create processor successfully"
            );
        });
    }

    #[test]
    fn test_factory_invalid_config() {
        test_config(json!({ "period": 12345 }), |result| {
            assert!(result.is_err(), "invalid config should fail");
        });
    }

    #[test]
    fn test_factory_rejects_period_below_minimum() {
        test_config(json!({ "period": "0s" }), |result| {
            assert!(result.is_err(), "zero period should fail validation");
        });
        test_config(json!({ "period": "50ms" }), |result| {
            assert!(result.is_err(), "sub-100ms period should fail validation");
        });
        test_config(json!({ "period": "100ms" }), |result| {
            assert!(result.is_ok(), "100ms period should pass validation");
        });
    }

    // --- Test Helpers ---

    fn test_config<F>(config: serde_json::Value, assert_fn: F)
    where
        F: FnOnce(Result<ProcessorWrapper<OtapPdata>, ConfigError>),
    {
        let res = try_create_processor(config).map(|(_, proc)| proc);
        assert_fn(res);
    }

    fn try_create_processor(
        config: serde_json::Value,
    ) -> Result<(TestRuntime<OtapPdata>, ProcessorWrapper<OtapPdata>), ConfigError> {
        let pipeline_ctx = create_test_pipeline_context();
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("temporal-reaggregation-config-test");

        let mut node_config =
            NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
        node_config.config = config;

        (TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY.create)(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .map(|proc| (rt, proc))
    }

    fn create_metrics_pdata() -> OtapPdata {
        let mut datagen = DataGenerator::new(3);
        let metrics_data = datagen.generate_metrics();
        let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
            metrics_data,
        ));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_traces_pdata() -> OtapPdata {
        let mut datagen = DataGenerator::new(3);
        let traces_data = datagen.generate_traces();
        let otap_records =
            otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Traces(traces_data));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_test_pipeline_context() -> PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
    }
}
