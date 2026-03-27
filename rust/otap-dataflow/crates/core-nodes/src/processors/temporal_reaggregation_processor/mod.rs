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
use otap_df_telemetry::metrics::MetricSet;

mod aggregate;
mod config;
mod data_point_builder;
mod identity;
mod metrics;

use self::aggregate::MetricAggregator;
use self::config::Config;
use self::metrics::TemporalReaggregationMetrics;

/// The URN for the temporal reaggregation processor.
pub const TEMPORAL_REAGGREGATION_PROCESSOR_URN: &str = "urn:otel:processor:temporal_reaggregation";

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

/// The temporal reaggregation processor.
pub struct TemporalReaggregationProcessor {
    metrics: MetricSet<TemporalReaggregationMetrics>,
    collection_period: Duration,
    /// Whether the periodic flush timer has been started.
    timer_started: bool,
    /// Accumulates metrics data over the collection interval.
    accumulator: MetricAggregator,
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
                    // TODO: Support for both view types
                    SignalType::Metrics => {
                        if let OtapPayload::OtapArrowRecords(ref records) = *pdata.payload_ref() {
                            if let Ok(view) = OtapMetricsView::try_from(records) {
                                self.accumulator.ingest(view);
                            }
                        }
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
            accumulator: MetricAggregator::new(),
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
        if let Some(batch) = self.accumulator.finish() {
            let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(batch));
            effect_handler.send_message_with_source_node(pdata).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::future::Future;

    use otap_df_config::error::Error as ConfigError;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime};
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::fixtures::DataGenerator;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_pdata::views::otap::OtapMetricsView;
    use otap_df_pdata_views::views::metrics::{
        DataType as MetricDataType, DataView, GaugeView, MetricView, MetricsView,
        NumberDataPointView, ResourceMetricsView, ScopeMetricsView, Value as DpValue,
    };
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

    // ==================== Reaggregation Tests ====================

    #[test]
    fn test_timer_tick_flushes_gauge() {
        // A gauge with two data points on the same stream (no DP attrs) should
        // collapse to just the latest data point on flush.
        run_processor_test(json!({}), |mut ctx| async move {
            let pdata = make_gauge_pdata(
                "cpu.usage",
                vec![
                    NumberDataPoint::build()
                        .time_unix_nano(1000u64)
                        .value_double(10.0)
                        .finish(),
                    NumberDataPoint::build()
                        .time_unix_nano(2000u64)
                        .value_double(20.0)
                        .finish(),
                ],
            );
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("timer tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "expected one output batch");

            let values = collect_gauge_values(&output[0]);
            let cpu = values.get("cpu.usage").expect("cpu.usage metric");
            assert_eq!(cpu.len(), 1, "same-stream points should collapse to one");
            assert_eq!(cpu[0], (2000, 20.0), "latest point should win");
        });
    }

    #[test]
    fn test_correlation_across_batches() {
        // Two separate batches with the same stream identity should be
        // correlated — only the latest data point survives.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_gauge_pdata(
                "temperature",
                vec![NumberDataPoint::build()
                    .time_unix_nano(1000u64)
                    .value_double(25.0)
                    .finish()],
            );
            ctx.process(Message::PData(batch1)).await.expect("batch1");

            let batch2 = make_gauge_pdata(
                "temperature",
                vec![NumberDataPoint::build()
                    .time_unix_nano(2000u64)
                    .value_double(30.0)
                    .finish()],
            );
            ctx.process(Message::PData(batch2)).await.expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("timer tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);

            let values = collect_gauge_values(&output[0]);
            let temp = values.get("temperature").expect("temperature metric");
            assert_eq!(temp.len(), 1, "same-stream points should collapse");
            assert_eq!(temp[0], (2000, 30.0), "latest point should win");
        });
    }

    #[test]
    fn test_distinct_streams_preserved() {
        // Data points with different attributes are different streams and
        // should both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            let pdata = make_gauge_pdata(
                "cpu.usage",
                vec![
                    NumberDataPoint::build()
                        .time_unix_nano(1000u64)
                        .value_double(10.0)
                        .attributes(vec![KeyValue::new(
                            "host",
                            AnyValue::new_string("host-a"),
                        )])
                        .finish(),
                    NumberDataPoint::build()
                        .time_unix_nano(1000u64)
                        .value_double(20.0)
                        .attributes(vec![KeyValue::new(
                            "host",
                            AnyValue::new_string("host-b"),
                        )])
                        .finish(),
                ],
            );
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("timer tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);

            let values = collect_gauge_values(&output[0]);
            let cpu = values.get("cpu.usage").expect("cpu.usage metric");
            assert_eq!(cpu.len(), 2, "distinct streams should both be preserved");
        });
    }

    #[test]
    fn test_mixed_correlation_and_new_streams() {
        // Batch 1: "cpu" and "memory" each with one data point.
        // Batch 2: "cpu" with a newer data point (correlates with batch 1).
        // After flush: "cpu" should have 1 DP (latest), "memory" should have
        // 1 DP (unchanged).
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_metrics_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .unit("1")
                            .data_gauge(Gauge::new(vec![NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_double(10.0)
                                .finish()]))
                            .finish(),
                        Metric::build()
                            .name("memory")
                            .unit("By")
                            .data_gauge(Gauge::new(vec![NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_double(50.0)
                                .finish()]))
                            .finish(),
                    ],
                )],
            )]));
            ctx.process(Message::PData(batch1)).await.expect("batch1");

            let batch2 = make_gauge_pdata(
                "cpu",
                vec![NumberDataPoint::build()
                    .time_unix_nano(2000u64)
                    .value_double(20.0)
                    .finish()],
            );
            ctx.process(Message::PData(batch2)).await.expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("timer tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);

            let values = collect_gauge_values(&output[0]);

            let cpu = values.get("cpu").expect("cpu metric");
            assert_eq!(cpu.len(), 1, "cpu streams should collapse");
            assert_eq!(cpu[0], (2000, 20.0), "cpu should have latest value");

            let mem = values.get("memory").expect("memory metric");
            assert_eq!(mem.len(), 1, "memory should have one data point");
            assert_eq!(mem[0], (1000, 50.0), "memory should be unchanged");
        });
    }

    #[test]
    fn test_multiple_flush_cycles() {
        // After a flush the accumulator should be clean. Each cycle should
        // produce independent output.
        run_processor_test(json!({}), |mut ctx| async move {
            // Cycle 1
            let pdata = make_gauge_pdata(
                "cpu",
                vec![NumberDataPoint::build()
                    .time_unix_nano(1000u64)
                    .value_double(10.0)
                    .finish()],
            );
            ctx.process(Message::PData(pdata)).await.expect("cycle1");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick1");

            let output1 = ctx.drain_pdata().await;
            assert_eq!(output1.len(), 1);
            let vals1 = collect_gauge_values(&output1[0]);
            assert_eq!(vals1["cpu"][0], (1000, 10.0));

            // Cycle 2 — new data, should not contain cycle 1 state
            let pdata = make_gauge_pdata(
                "cpu",
                vec![NumberDataPoint::build()
                    .time_unix_nano(3000u64)
                    .value_double(30.0)
                    .finish()],
            );
            ctx.process(Message::PData(pdata)).await.expect("cycle2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick2");

            let output2 = ctx.drain_pdata().await;
            assert_eq!(output2.len(), 1);
            let vals2 = collect_gauge_values(&output2[0]);
            assert_eq!(
                vals2["cpu"][0],
                (3000, 30.0),
                "cycle 2 should only contain cycle 2 data"
            );
        });
    }

    #[test]
    fn test_shutdown_flushes_accumulated_metrics() {
        // Shutdown should flush any buffered data, just like a timer tick.
        run_processor_test(json!({}), |mut ctx| async move {
            let pdata = make_gauge_pdata(
                "cpu",
                vec![NumberDataPoint::build()
                    .time_unix_nano(1000u64)
                    .value_double(42.0)
                    .finish()],
            );
            ctx.process(Message::PData(pdata)).await.expect("process");

            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline,
                reason: "test".into(),
            }))
            .await
            .expect("shutdown");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "shutdown should flush buffered data");

            let values = collect_gauge_values(&output[0]);
            let cpu = values.get("cpu").expect("cpu metric");
            assert_eq!(cpu[0], (1000, 42.0));
        });
    }

    // ==================== Config Tests ====================

    // --- Test Helpers ---

    /// Create a processor wrapped in TestRuntime, run a scenario, validate.
    fn run_processor_test<F, Fut>(config_json: serde_json::Value, scenario: F)
    where
        F: FnOnce(TestContext<OtapPdata>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let (rt, proc) = try_create_processor(config_json).expect("valid config");
        rt.set_processor(proc)
            .run_test(scenario)
            .validate(|_ctx| async {});
    }

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

    /// Build an [`OtapPdata`] containing a single gauge metric with the given
    /// name and data points, wrapped in a default resource and scope.
    fn make_gauge_pdata(name: &str, data_points: Vec<NumberDataPoint>) -> OtapPdata {
        make_metrics_pdata(MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                vec![Metric::build()
                    .name(name)
                    .unit("1")
                    .data_gauge(Gauge::new(data_points))
                    .finish()],
            )],
        )]))
    }

    /// Convert OTLP [`MetricsData`] into an [`OtapPdata`] via OTAP encoding.
    fn make_metrics_pdata(metrics_data: MetricsData) -> OtapPdata {
        let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
            metrics_data,
        ));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_metrics_pdata() -> OtapPdata {
        let mut datagen = DataGenerator::new(3);
        let metrics_data = datagen.generate_metrics();
        make_metrics_pdata(metrics_data)
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

    /// Extract gauge metric data from an output [`OtapPdata`].
    ///
    /// Returns a map of metric name to a sorted list of `(time_unix_nano, value)`
    /// tuples for all gauge data points in the output.
    fn collect_gauge_values(pdata: &OtapPdata) -> BTreeMap<String, Vec<(u64, f64)>> {
        let records = match pdata.payload_ref() {
            OtapPayload::OtapArrowRecords(r) => r,
            _ => panic!("expected OtapArrowRecords payload"),
        };
        let view = OtapMetricsView::try_from(records).expect("valid metrics view");

        let mut result: BTreeMap<String, Vec<(u64, f64)>> = BTreeMap::new();
        for resource in view.resources() {
            for scope in resource.scopes() {
                for metric in scope.metrics() {
                    let name =
                        String::from_utf8(metric.name().to_vec()).expect("valid UTF-8 name");
                    let data = metric.data().expect("metric should have data");
                    if data.value_type() != MetricDataType::Gauge {
                        continue;
                    }
                    let gauge = data.as_gauge().expect("gauge data");
                    let entry = result.entry(name).or_default();
                    for dp in gauge.data_points() {
                        let value: f64 = match dp.value() {
                            Some(DpValue::Double(v)) => v,
                            Some(DpValue::Integer(v)) => v as f64,
                            None => panic!("data point has no value"),
                        };
                        entry.push((dp.time_unix_nano(), value));
                    }
                }
            }
        }
        // Sort each metric's data points by time for deterministic assertions.
        for entries in result.values_mut() {
            entries.sort_by_key(|(t, _)| *t);
        }
        result
    }
}
