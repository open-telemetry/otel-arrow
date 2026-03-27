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
        // TODO: Track some kind of failure metric here
        if let Ok(Some(batch)) = self.accumulator.finish() {
            let pdata = OtapPdata::new_todo_context(OtapPayload::OtapArrowRecords(batch));
            effect_handler.send_message_with_source_node(pdata).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    use otap_df_config::error::Error as ConfigError;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::{TestContext, TestRuntime};
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_pdata::otap::{OtapArrowRecords, OtapBatchStore};
    use otap_df_pdata::otlp::metrics::MetricType;
    use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Histogram,
        HistogramDataPoint, Metric, MetricsData, ResourceMetrics, ScopeMetrics, Summary,
        SummaryDataPoint,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use otap_df_pdata::testing::fixtures::DataGenerator;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
    use otap_df_pdata::{metrics, record_batch};
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
    fn test_gauge_correlation() {
        // Two batches with the same gauge stream. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            #[rustfmt::skip]
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("name", Utf8, ["cpu"])),
                (NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [1000i64]),
                    ("double_value", Float64, [10.0]))
            ));

            #[rustfmt::skip]
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("name", Utf8, ["cpu"])),
                (NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [2000i64]),
                    ("double_value", Float64, [20.0]))
            ));
            let expected = batch2.clone();

            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .unwrap();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .unwrap();
            ctx.process(Message::timer_tick_ctrl_msg()).await.unwrap();

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &expected);
        });
    }

    #[test]
    fn test_cumulative_sum_correlation() {
        // Two batches with the same cumulative monotonic sum. The later
        // timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("metric_type", UInt8, [MetricType::Sum as u8]),
                    ("name", Utf8, ["requests"]),
                    ("aggregation_temporality", Int32, [2i32]),
                    ("is_monotonic", Boolean, [true])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [1000i64]),
                    ("int_value", Int64, [100i64])
                )
            ));
            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .expect("batch1");

            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("metric_type", UInt8, [MetricType::Sum as u8]),
                    ("name", Utf8, ["requests"]),
                    ("aggregation_temporality", Int32, [2i32]),
                    ("is_monotonic", Boolean, [true])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [2000i64]),
                    ("int_value", Int64, [200i64])
                )
            ));
            let expected = batch2.clone();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &expected);
        });
    }

    #[test]
    fn test_cumulative_histogram_correlation() {
        // Two batches with the same cumulative histogram. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .count(10u64)
                                        .sum(100.0f64)
                                        .bucket_counts(vec![2, 3, 5])
                                        .explicit_bounds(vec![10.0, 50.0])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]));
            ctx.process(Message::PData(batch1)).await.expect("batch1");

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency")
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    HistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(20u64)
                                        .sum(200.0f64)
                                        .bucket_counts(vec![4, 6, 10])
                                        .explicit_bounds(vec![10.0, 50.0])
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(batch2)).await.expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_cumulative_exp_histogram_correlation() {
        // Two batches with the same cumulative exp histogram. The later
        // timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency.exp")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(1000u64)
                                        .count(5u64)
                                        .scale(2i32)
                                        .zero_count(1u64)
                                        .positive(Buckets::new(0, vec![1, 2]))
                                        .negative(Buckets::new(0, vec![1, 1]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]));
            ctx.process(Message::PData(batch1)).await.expect("batch1");

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("latency.exp")
                            .data_exponential_histogram(ExponentialHistogram::new(
                                AggregationTemporality::Cumulative,
                                vec![
                                    ExponentialHistogramDataPoint::build()
                                        .time_unix_nano(2000u64)
                                        .count(10u64)
                                        .scale(2i32)
                                        .zero_count(2u64)
                                        .positive(Buckets::new(0, vec![3, 4]))
                                        .negative(Buckets::new(0, vec![2, 1]))
                                        .finish(),
                                ],
                            ))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(batch2)).await.expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_summary_correlation() {
        // Two batches with the same summary stream. The later timestamp wins.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = make_otlp_pdata(MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(1000u64)
                                    .count(10u64)
                                    .sum(500.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 45.0),
                                        ValueAtQuantile::new(0.99, 95.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]));
            ctx.process(Message::PData(batch1)).await.expect("batch1");

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("request.duration")
                            .data_summary(Summary::new(vec![
                                SummaryDataPoint::build()
                                    .time_unix_nano(2000u64)
                                    .count(20u64)
                                    .sum(1000.0f64)
                                    .quantile_values(vec![
                                        ValueAtQuantile::new(0.5, 50.0),
                                        ValueAtQuantile::new(0.99, 99.0),
                                    ])
                                    .finish(),
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let batch2 = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(batch2)).await.expect("batch2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    // ==================== Hierarchy Differentiation Tests ====================

    #[test]
    fn test_different_resources_preserved() {
        // Two gauges with different resource attributes should be treated as
        // separate streams and both preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{Gauge, NumberDataPoint};

            let expected_data = MetricsData::new(vec![
                ResourceMetrics::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("prod"))])
                        .finish(),
                    vec![ScopeMetrics::new(
                        InstrumentationScope::build().finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(10.0).finish(),
                                ]))
                                .finish(),
                        ],
                    )],
                ),
                ResourceMetrics::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("staging"))])
                        .finish(),
                    vec![ScopeMetrics::new(
                        InstrumentationScope::build().finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(20.0).finish(),
                                ]))
                                .finish(),
                        ],
                    )],
                ),
            ]);
            let pdata = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_different_scope_attributes_preserved() {
        // Two gauges with the same resource but different scope attributes
        // should both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{Gauge, NumberDataPoint};

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![
                    ScopeMetrics::new(
                        InstrumentationScope::build()
                            .attributes(vec![KeyValue::new(
                                "lib",
                                AnyValue::new_string("opentelemetry"),
                            )])
                            .finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(10.0).finish(),
                                ]))
                                .finish(),
                        ],
                    ),
                    ScopeMetrics::new(
                        InstrumentationScope::build()
                            .attributes(vec![KeyValue::new(
                                "lib",
                                AnyValue::new_string("prometheus"),
                            )])
                            .finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(20.0).finish(),
                                ]))
                                .finish(),
                        ],
                    ),
                ],
            )]);
            let pdata = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_different_scope_name_preserved() {
        // Two gauges with the same resource but different scope names should
        // both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{Gauge, NumberDataPoint};

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![
                    ScopeMetrics::new(
                        InstrumentationScope::build().name("scope-a").finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(10.0).finish(),
                                ]))
                                .finish(),
                        ],
                    ),
                    ScopeMetrics::new(
                        InstrumentationScope::build().name("scope-b").finish(),
                        vec![
                            Metric::build()
                                .name("cpu")
                                .data_gauge(Gauge::new(vec![
                                    NumberDataPoint::build().value_double(20.0).finish(),
                                ]))
                                .finish(),
                        ],
                    ),
                ],
            )]);
            let pdata = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    #[test]
    fn test_different_metric_name_preserved() {
        // Two gauges with the same resource/scope but different metric names
        // should both be preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            let input = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16, 1]),
                    ("name", Utf8, ["cpu", "memory"])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32, 1]),
                    ("parent_id", UInt16, [0u16, 1]),
                    ("double_value", Float64, [10.0, 20.0])
                )
            ));
            let expected = input.clone();
            ctx.process(Message::PData(make_pdata(input)))
                .await
                .expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &expected);
        });
    }

    #[test]
    fn test_different_metric_type_preserved() {
        // A gauge and a cumulative sum with the same name should be treated as
        // different metrics and both preserved.
        run_processor_test(json!({}), |mut ctx| async move {
            let input = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16, 1]),
                    ("name", Utf8, ["cpu", "cpu"]),
                    (
                        "metric_type",
                        UInt8,
                        [MetricType::Gauge as u8, MetricType::Sum as u8]
                    ),
                    ("aggregation_temporality", Int32, [0i32, 2]),
                    ("is_monotonic", Boolean, [false, true])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32, 1]),
                    ("parent_id", UInt16, [0u16, 1]),
                    ("double_value", Float64, [10.0, 20.0])
                )
            ));
            let expected = input.clone();
            ctx.process(Message::PData(make_pdata(input)))
                .await
                .expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &expected);
        });
    }

    #[test]
    fn test_different_dp_attributes_preserved() {
        // One gauge with two data points that have different DP attributes
        // should treat them as distinct streams and preserve both. We use
        // the OTLP builder path since constructing DP attributes with proper
        // dictionary encoding at the OTAP level is complex.
        run_processor_test(json!({}), |mut ctx| async move {
            use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
            use otap_df_pdata::proto::opentelemetry::metrics::v1::{Gauge, NumberDataPoint};

            let expected_data = MetricsData::new(vec![ResourceMetrics::new(
                Resource::build().finish(),
                vec![ScopeMetrics::new(
                    InstrumentationScope::build().finish(),
                    vec![
                        Metric::build()
                            .name("cpu")
                            .data_gauge(Gauge::new(vec![
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
                            ]))
                            .finish(),
                    ],
                )],
            )]);
            let pdata = make_otlp_pdata(expected_data.clone());
            ctx.process(Message::PData(pdata)).await.expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_otlp_equivalent(&output[0], expected_data);
        });
    }

    // ==================== Flush Behavior Tests ====================

    #[test]
    fn test_multiple_flush_cycles() {
        // After a flush the accumulator should be clean. Each cycle should
        // produce independent output.
        run_processor_test(json!({}), |mut ctx| async move {
            let batch1 = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("name", Utf8, ["cpu"])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [1000i64]),
                    ("double_value", Float64, [10.0])
                )
            ));
            let expected1 = batch1.clone();
            ctx.process(Message::PData(make_pdata(batch1)))
                .await
                .expect("cycle1");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick1");

            let output1 = ctx.drain_pdata().await;
            assert_eq!(output1.len(), 1);
            assert_output_equivalent(&output1[0], &expected1);

            // Cycle 2 — new data, should not contain cycle 1 state
            let batch2 = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("name", Utf8, ["cpu"])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [3000i64]),
                    ("double_value", Float64, [30.0])
                )
            ));
            let expected2 = batch2.clone();
            ctx.process(Message::PData(make_pdata(batch2)))
                .await
                .expect("cycle2");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick2");

            let output2 = ctx.drain_pdata().await;
            assert_eq!(output2.len(), 1);
            assert_output_equivalent(&output2[0], &expected2);
        });
    }

    #[test]
    fn test_shutdown_flushes_accumulated_metrics() {
        // Shutdown should flush any buffered data, just like a timer tick.
        run_processor_test(json!({}), |mut ctx| async move {
            let input = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16]),
                    ("name", Utf8, ["cpu"])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32]),
                    ("parent_id", UInt16, [0u16]),
                    ("time_unix_nano", TimestampNs, [1000i64]),
                    ("double_value", Float64, [42.0])
                )
            ));
            let expected = input.clone();
            ctx.process(Message::PData(make_pdata(input)))
                .await
                .expect("process");

            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline,
                reason: "test".into(),
            }))
            .await
            .expect("shutdown");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1, "shutdown should flush buffered data");
            assert_output_equivalent(&output[0], &expected);
        });
    }

    #[test]
    fn test_mixed_metric_types_in_single_batch() {
        // A batch containing both a gauge and a cumulative sum should preserve
        // both in the output.
        run_processor_test(json!({}), |mut ctx| async move {
            let input = OtapArrowRecords::Metrics(metrics!(
                (
                    UnivariateMetrics,
                    ("id", UInt16, [0u16, 1]),
                    ("name", Utf8, ["temperature", "requests"]),
                    (
                        "metric_type",
                        UInt8,
                        [MetricType::Gauge as u8, MetricType::Sum as u8]
                    ),
                    ("aggregation_temporality", Int32, [0i32, 2]),
                    ("is_monotonic", Boolean, [false, true])
                ),
                (
                    NumberDataPoints,
                    ("id", UInt32, [0u32, 1]),
                    ("parent_id", UInt16, [0u16, 1]),
                    ("double_value", Float64, [22.5, 1000.0])
                )
            ));
            let expected = input.clone();
            ctx.process(Message::PData(make_pdata(input)))
                .await
                .expect("process");

            ctx.process(Message::timer_tick_ctrl_msg())
                .await
                .expect("tick");

            let output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            assert_output_equivalent(&output[0], &expected);
        });
    }

    // ==================== Config Tests ====================

    // ==================== Helpers ====================

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

    /// Wrap [`OtapArrowRecords`] in an [`OtapPdata`].
    fn make_pdata(records: OtapArrowRecords) -> OtapPdata {
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(records))
    }

    /// Convert OTLP [`MetricsData`] into an [`OtapPdata`] via OTAP encoding.
    fn make_otlp_pdata(metrics_data: MetricsData) -> OtapPdata {
        let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
            metrics_data,
        ));
        OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
    }

    fn create_metrics_pdata() -> OtapPdata {
        let mut datagen = DataGenerator::new(3);
        let metrics_data = datagen.generate_metrics();
        make_otlp_pdata(metrics_data)
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

    /// Assert that the processor output is semantically equivalent to the
    /// expected [`OtapArrowRecords`].
    fn assert_output_equivalent(output: &OtapPdata, expected: &OtapArrowRecords) {
        let actual = match output.payload_ref() {
            OtapPayload::OtapArrowRecords(r) => r,
            _ => panic!("expected OtapArrowRecords payload"),
        };
        assert_equivalent(&[otap_to_otlp(actual)], &[otap_to_otlp(expected)]);
    }

    /// Assert that the processor output is semantically equivalent to the
    /// expected OTLP [`MetricsData`].
    fn assert_output_otlp_equivalent(output: &OtapPdata, expected: MetricsData) {
        let actual = match output.payload_ref() {
            OtapPayload::OtapArrowRecords(r) => r,
            _ => panic!("expected OtapArrowRecords payload"),
        };
        assert_equivalent(
            &[otap_to_otlp(actual)],
            &[otap_df_pdata::proto::OtlpProtoMessage::Metrics(expected)],
        );
    }
}
