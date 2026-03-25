// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Temporal reaggregation processor for OTAP metrics.
//!
//! This processor decreases telemetry volume by reaggregating metrics collected
//! at a higher frequency into a lower one. It collects metrics for a fixed
//! interval and emits a single data point per stream when the interval expires.

pub mod config;
mod metrics;

use self::config::Config;
use self::metrics::TemporalReaggregationMetrics;
use async_trait::async_trait;
use linkme::distributed_slice;
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
use otap_df_engine::process_duration::ComputeDuration;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::metrics::MetricSet;
use std::sync::Arc;
use std::time::Duration;

/// The URN for the temporal reaggregation processor.
pub const TEMPORAL_REAGGREGATION_PROCESSOR_URN: &str = "urn:otel:processor:temporal_reaggregation";

/// The temporal reaggregation processor.
///
/// Currently this is a passthrough implementation. Aggregation logic will be
/// added in a subsequent stage.
pub struct TemporalReaggregationProcessor {
    metrics: MetricSet<TemporalReaggregationMetrics>,
    compute_duration: ComputeDuration,
    collection_period: Duration,
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
        let compute_duration = ComputeDuration::new(&pipeline_ctx);
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Ok(Self {
            metrics,
            compute_duration,
            collection_period: config.period,
        })
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
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    _ = metrics_reporter.report(&mut self.metrics);
                    self.compute_duration.report(&mut metrics_reporter);
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. } => {
                    // Nothing buffered yet in this passthrough implementation.
                    Ok(())
                }
                _ => Ok(()),
            },
            Message::PData(pdata) => {
                // Passthrough: forward all data unchanged.
                // Aggregation logic will be added in a subsequent stage.
                effect_handler.send_message_with_source_node(pdata).await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_pdata::OtapPayload;
    use otap_df_pdata::testing::fixtures::DataGenerator;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;

    fn create_test_pipeline_context() -> PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
    }

    fn create_processor(
        config: serde_json::Value,
    ) -> (TestRuntime<OtapPdata>, ProcessorWrapper<OtapPdata>) {
        let pipeline_ctx = create_test_pipeline_context();
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("temporal-reaggregation-test");

        let mut node_config =
            NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
        node_config.config = config;

        let proc = create_temporal_reaggregation_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .expect("create processor");

        (rt, proc)
    }

    fn create_metrics_pdata() -> OtapPdata {
        let mut datagen = DataGenerator::new(3);
        let metrics_data = datagen.generate_metrics();
        let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
            metrics_data,
        ));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    #[test]
    fn test_default_config_parsing() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.period, std::time::Duration::from_secs(60));
    }

    #[test]
    fn test_custom_config_parsing() {
        let config: Config = serde_json::from_value(json!({
            "period": "30s",
        }))
        .unwrap();
        assert_eq!(config.period, std::time::Duration::from_secs(30));
    }

    #[test]
    fn test_passthrough_logs() {
        let (rt, proc) = create_processor(json!({}));

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
    fn test_passthrough_metrics() {
        let (rt, proc) = create_processor(json!({}));

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let pdata = create_metrics_pdata();

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
    fn test_passthrough_multiple_messages() {
        let (rt, proc) = create_processor(json!({}));

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                // Send logs
                let logs_pdata = create_test_pdata();
                ctx.process(Message::PData(logs_pdata))
                    .await
                    .expect("process logs");

                // Send metrics
                let metrics_pdata = create_metrics_pdata();
                ctx.process(Message::PData(metrics_pdata))
                    .await
                    .expect("process metrics");

                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 2, "expected two forwarded messages");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_shutdown_with_no_buffered_data() {
        let (rt, proc) = create_processor(json!({}));

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
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
    fn test_collect_telemetry() {
        let (rt, proc) = create_processor(json!({}));

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("timer tick should succeed");

                let output = ctx.drain_pdata().await;
                assert!(output.is_empty(), "timer tick should not emit data");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }

    #[test]
    fn test_factory_creation() {
        let pipeline_ctx = create_test_pipeline_context();
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("temporal-reaggregation-factory-test");

        let mut node_config =
            NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
        node_config.config = json!({
            "period": "5s"
        });

        let result = create_temporal_reaggregation_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        );
        assert!(
            result.is_ok(),
            "factory should create processor successfully"
        );
    }

    #[test]
    fn test_factory_invalid_config() {
        let pipeline_ctx = create_test_pipeline_context();
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("temporal-reaggregation-invalid-config");

        let mut node_config =
            NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
        node_config.config = json!({
            "period": 12345  // Wrong type, should be a string like "5s"
        });

        let result = create_temporal_reaggregation_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        );
        assert!(result.is_err(), "invalid config should fail");
    }
}
