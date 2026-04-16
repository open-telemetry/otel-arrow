// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use std::hint::black_box;
use std::sync::Arc;

use criterion::measurement::WallTime;
use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use otap_df_channel::mpsc;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::Interests;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{NodeControlMsg, RuntimeCtrlMsgReceiver, runtime_ctrl_msg_channel};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::{Message, Receiver, Sender};
use otap_df_engine::node::NodeWithPDataReceiver;
use otap_df_engine::node::NodeWithPDataSender;
use otap_df_engine::processor::{ProcessorWrapper, ProcessorWrapperRuntime};
use otap_df_engine::testing::node::test_node;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otap_df_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
    NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};
use otap_df_pdata::{OtapPayload, proto::OtlpProtoMessage};
use otap_df_telemetry::InternalTelemetrySystem;
use serde_json::json;

use otap_df_core_nodes::processors::temporal_reaggregation_processor::{
    TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY, TEMPORAL_REAGGREGATION_PROCESSOR_URN,
};

const NUM_BATCHES: usize = 50;
const METRICS_PER_BATCH: usize = 100;

/// Fraction of metrics per batch that are aggregatable (0.0–1.0).
/// The remainder are non-aggregatable (passthrough).
const AGGREGATABLE_FRACTION: f64 = 0.5;

/// Number of distinct aggregatable metric types (gauge, cumulative sum,
/// cumulative histogram, summary).
const NUM_AGGREGATABLE_TYPES: usize = 4;

/// Number of distinct non-aggregatable metric types (delta sum, delta
/// histogram, non-monotonic cumulative sum).
const NUM_NON_AGGREGATABLE_TYPES: usize = 3;

/// Output channel capacity. Must hold at least NUM_BATCHES messages since every
/// mixed batch emits its non-aggregatable portion immediately, plus one more for
/// the final flush.
const OUTPUT_CHANNEL_CAPACITY: usize = NUM_BATCHES + 16;

criterion_group!(benches, bench_temporal_reaggregation);
criterion_main!(benches);

fn bench_temporal_reaggregation(c: &mut Criterion) {
    // Single-threaded tokio runtime used for all benchmark iterations.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Generate data once, outside the benchmark loop.
    let (otlp_messages, otap_messages) = generate_bench_data();

    let mut group = c.benchmark_group("temporal_reaggregation");
    let _ = group.throughput(Throughput::Elements(
        (NUM_BATCHES * METRICS_PER_BATCH) as u64,
    ));

    bench_scenario(&mut group, &rt, "otlp", &otlp_messages);
    bench_scenario(&mut group, &rt, "otap", &otap_messages);

    group.finish();
}

/// Run a single named benchmark scenario within the group.
fn bench_scenario(
    group: &mut BenchmarkGroup<'_, WallTime>,
    rt: &tokio::runtime::Runtime,
    label: &str,
    messages: &[OtapPdata],
) {
    let _ = group.bench_function(BenchmarkId::new(label, METRICS_PER_BATCH), |b| {
        b.iter_batched(
            || {
                let state = create_processor();
                (messages.to_vec(), state)
            },
            |(msgs, mut state)| {
                rt.block_on(run_scenario(msgs, &mut state));
            },
            BatchSize::LargeInput,
        );
    });
}

/// All state needed to drive the processor for a single benchmark iteration.
///
/// The `_ctrl_rx` field is never read from, but must be kept alive so the
/// runtime control channel remains open - the processor sends a
/// `StartTimer` message on the first `process()` call.
struct ProcessorState {
    processor: Box<dyn local::Processor<OtapPdata>>,
    effect_handler: local::EffectHandler<OtapPdata>,
    output_receiver: Receiver<OtapPdata>,
    _ctrl_rx: RuntimeCtrlMsgReceiver<OtapPdata>,
}

/// Create a fresh processor instance with all wiring needed for direct
/// `process()` calls.
fn create_processor() -> ProcessorState {
    // Pipeline context with telemetry.
    let telemetry = InternalTelemetrySystem::default();
    let registry = telemetry.registry();
    let reporter = telemetry.reporter();

    let controller_ctx = ControllerContext::new(registry);
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("bench_grp".into(), "bench_pipe".into(), 0, 1, 0);

    // Build the processor via the registered factory.
    let mut node_config =
        NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
    node_config.config = json!({});

    let config = otap_df_engine::config::ProcessorConfig::new("temporal_reaggregation_bench");

    let mut wrapper: ProcessorWrapper<OtapPdata> = (TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY
        .create)(
        pipeline_ctx,
        test_node("temporal_reaggregation_bench"),
        Arc::new(node_config),
        &config,
    )
    .expect("failed to create processor");

    // Wire up the output channel
    let (out_tx, out_rx) = mpsc::Channel::new(OUTPUT_CHANNEL_CAPACITY);
    let out_sender = Sender::new_local_mpsc_sender(out_tx);
    let _ = wrapper.set_pdata_sender(
        test_node("temporal_reaggregation_bench"),
        "default".into(),
        out_sender,
    );

    // Wire up a dummy input receiver
    let (_, dummy_rx) = mpsc::Channel::<OtapPdata>::new(1);
    let dummy_receiver = Receiver::new_local_mpsc_receiver(dummy_rx);
    let _ = wrapper.set_pdata_receiver(test_node("temporal_reaggregation_bench"), dummy_receiver);

    // Prepare the runtime to get the processor + effect handler.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build setup runtime");

    let runtime = rt
        .block_on(wrapper.prepare_runtime(reporter, Interests::empty()))
        .expect("failed to prepare runtime");

    match runtime {
        ProcessorWrapperRuntime::Local {
            mut effect_handler,
            processor,
            ..
        } => {
            // The runtime control channel is required by the effect handler.
            // The receiver must stay alive for the duration of the iteration
            // so the channel remains open.
            let (ctrl_tx, ctrl_rx) = runtime_ctrl_msg_channel(10);
            effect_handler.set_runtime_ctrl_msg_sender(ctrl_tx);

            let output_receiver = Receiver::new_local_mpsc_receiver(out_rx);
            ProcessorState {
                processor,
                effect_handler,
                output_receiver,
                _ctrl_rx: ctrl_rx,
            }
        }
        ProcessorWrapperRuntime::Shared { .. } => {
            unreachable!("temporal reaggregation processor is always local")
        }
    }
}

/// Run one complete benchmark iteration: process all messages then flush.
async fn run_scenario(messages: Vec<OtapPdata>, state: &mut ProcessorState) {
    // Process all data messages.
    for msg in messages {
        state
            .processor
            .process(Message::PData(msg), &mut state.effect_handler)
            .await
            .expect("process failed");
    }

    // Flush via wakeup: pop the scheduled wakeup from the local scheduler
    // and deliver it as a control message.
    if let Some((slot, when, revision)) = state.effect_handler.pop_wakeup() {
        state
            .processor
            .process(
                Message::Control(NodeControlMsg::Wakeup {
                    slot,
                    when,
                    revision,
                }),
                &mut state.effect_handler,
            )
            .await
            .expect("wakeup flush failed");
    }

    // Drain the output channel to prevent backpressure.
    let mut output = Vec::new();
    while let Ok(pdata) = state.output_receiver.try_recv() {
        output.push(pdata);
    }
    let _ = black_box(output);
}

/// Generate all benchmark data.
///
/// Returns `(otlp_messages, otap_messages)` — the same logical data encoded as
/// OTLP protobuf bytes and OTAP Arrow record batches respectively.
fn generate_bench_data() -> (Vec<OtapPdata>, Vec<OtapPdata>) {
    let otlp_data: Vec<MetricsData> = (0..NUM_BATCHES).map(build_batch_metrics_data).collect();

    let otlp_messages: Vec<OtapPdata> = otlp_data
        .iter()
        .map(|md| {
            let msg = OtlpProtoMessage::Metrics(md.clone());
            let otlp_bytes = otlp_message_to_bytes(&msg);
            OtapPdata::new_default(OtapPayload::OtlpBytes(otlp_bytes))
        })
        .collect();

    let otap_messages: Vec<OtapPdata> = otlp_data
        .iter()
        .map(|md| {
            let msg = OtlpProtoMessage::Metrics(md.clone());
            let otap_records = otlp_to_otap(&msg);
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
        })
        .collect();

    (otlp_messages, otap_messages)
}

/// Build a single batch's worth of metrics as OTLP [`MetricsData`].
///
/// The aggregatable portion uses the same stream identities across all batches
/// (controlled by the metric name + dp attribute) so the processor exercises
/// its dedup / latest-value-wins logic. Timestamps increment per batch.
fn build_batch_metrics_data(batch_idx: usize) -> MetricsData {
    let shape = BatchShape::new();
    let mut metrics = Vec::with_capacity(METRICS_PER_BATCH);

    // Each type gets a contiguous range of dp_attr stream IDs so every
    // aggregatable metric maps to a unique stream.
    let mut offset = 0;

    for i in 0..shape.n_gauges {
        metrics.push(make_agg_gauge(i, batch_idx, offset));
    }
    offset += shape.n_gauges;

    for i in 0..shape.n_cumulative_sums {
        metrics.push(make_agg_cumulative_sum(i, batch_idx, offset));
    }
    offset += shape.n_cumulative_sums;

    for i in 0..shape.n_cumulative_histograms {
        metrics.push(make_agg_cumulative_histogram(i, batch_idx, offset));
    }
    offset += shape.n_cumulative_histograms;

    for i in 0..shape.n_summaries {
        metrics.push(make_agg_summary(i, batch_idx, offset));
    }

    for i in 0..shape.n_delta_sums {
        metrics.push(make_delta_sum(i, batch_idx));
    }
    for i in 0..shape.n_delta_histograms {
        metrics.push(make_delta_histogram(i, batch_idx));
    }
    for i in 0..shape.n_non_monotonic_sums {
        metrics.push(make_nonmono_cumulative_sum(i, batch_idx));
    }

    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().attributes(resource_attrs()).finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("bench_scope")
                .attributes(scope_attrs())
                .finish(),
            metrics,
        )],
    )])
}

/// Per-batch counts for each metric type, derived from [`METRICS_PER_BATCH`]
/// and [`AGGREGATABLE_FRACTION`].
struct BatchShape {
    // Aggregatable types
    n_gauges: usize,
    n_cumulative_sums: usize,
    n_cumulative_histograms: usize,
    n_summaries: usize,
    // Non-aggregatable types
    n_delta_sums: usize,
    n_delta_histograms: usize,
    n_non_monotonic_sums: usize,
}

impl BatchShape {
    fn new() -> Self {
        let agg_total = (METRICS_PER_BATCH as f64 * AGGREGATABLE_FRACTION).round() as usize;
        let nonagg_total = METRICS_PER_BATCH - agg_total;

        // Evenly distribute across types; remainder goes to the first types.
        let agg_base = agg_total / NUM_AGGREGATABLE_TYPES;
        let agg_rem = agg_total % NUM_AGGREGATABLE_TYPES;

        let nonagg_base = nonagg_total / NUM_NON_AGGREGATABLE_TYPES;
        let nonagg_rem = nonagg_total % NUM_NON_AGGREGATABLE_TYPES;

        let shape = Self {
            n_gauges: agg_base + usize::from(agg_rem > 0),
            n_cumulative_sums: agg_base + usize::from(agg_rem > 1),
            n_cumulative_histograms: agg_base + usize::from(agg_rem > 2),
            n_summaries: agg_base,
            n_delta_sums: nonagg_base + usize::from(nonagg_rem > 0),
            n_delta_histograms: nonagg_base + usize::from(nonagg_rem > 1),
            n_non_monotonic_sums: nonagg_base,
        };

        debug_assert_eq!(
            shape.aggregatable_total() + shape.non_aggregatable_total(),
            METRICS_PER_BATCH
        );
        shape
    }

    fn aggregatable_total(&self) -> usize {
        self.n_gauges + self.n_cumulative_sums + self.n_cumulative_histograms + self.n_summaries
    }

    fn non_aggregatable_total(&self) -> usize {
        self.n_delta_sums + self.n_delta_histograms + self.n_non_monotonic_sums
    }
}

fn make_agg_gauge(idx: usize, batch_idx: usize, dp_offset: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("agg_gauge_{idx}"))
        .data_gauge(Gauge::new(vec![
            NumberDataPoint::build()
                .time_unix_nano(ts)
                .value_double(idx as f64 + batch_idx as f64 * 0.1)
                .attributes(dp_attr(dp_offset + idx))
                .finish(),
        ]))
        .finish()
}

fn make_agg_cumulative_sum(idx: usize, batch_idx: usize, dp_offset: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("agg_cum_sum_{idx}"))
        .data_sum(Sum::new(
            AggregationTemporality::Cumulative,
            true,
            vec![
                NumberDataPoint::build()
                    .time_unix_nano(ts)
                    .value_int((idx as i64 + 1) * (batch_idx as i64 + 1))
                    .attributes(dp_attr(dp_offset + idx))
                    .finish(),
            ],
        ))
        .finish()
}

fn make_agg_cumulative_histogram(idx: usize, batch_idx: usize, dp_offset: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("agg_cum_hist_{idx}"))
        .data_histogram(Histogram::new(
            AggregationTemporality::Cumulative,
            vec![
                HistogramDataPoint::build()
                    .time_unix_nano(ts)
                    .count((10 + batch_idx) as u64)
                    .sum(100.0 + batch_idx as f64)
                    .bucket_counts(vec![2, 3, 5])
                    .explicit_bounds(vec![10.0, 50.0])
                    .attributes(dp_attr(dp_offset + idx))
                    .finish(),
            ],
        ))
        .finish()
}

fn make_agg_summary(idx: usize, batch_idx: usize, dp_offset: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("agg_summary_{idx}"))
        .data_summary(Summary::new(vec![
            SummaryDataPoint::build()
                .time_unix_nano(ts)
                .count((10 + batch_idx) as u64)
                .sum(500.0 + batch_idx as f64)
                .quantile_values(vec![
                    ValueAtQuantile::new(0.5, 45.0),
                    ValueAtQuantile::new(0.99, 95.0),
                ])
                .attributes(dp_attr(dp_offset + idx))
                .finish(),
        ]))
        .finish()
}

fn make_delta_sum(idx: usize, batch_idx: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("nonagg_delta_sum_{idx}"))
        .data_sum(Sum::new(
            AggregationTemporality::Delta,
            true,
            vec![
                NumberDataPoint::build()
                    .time_unix_nano(ts)
                    .value_int(idx as i64 + 1)
                    .finish(),
            ],
        ))
        .finish()
}

fn make_delta_histogram(idx: usize, batch_idx: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("nonagg_delta_hist_{idx}"))
        .data_histogram(Histogram::new(
            AggregationTemporality::Delta,
            vec![
                HistogramDataPoint::build()
                    .time_unix_nano(ts)
                    .count(10u64)
                    .sum(100.0)
                    .bucket_counts(vec![2, 3, 5])
                    .explicit_bounds(vec![10.0, 50.0])
                    .finish(),
            ],
        ))
        .finish()
}

fn make_nonmono_cumulative_sum(idx: usize, batch_idx: usize) -> Metric {
    let ts = ((batch_idx + 1) * 1000) as u64;
    Metric::build()
        .name(format!("nonagg_nonmono_sum_{idx}"))
        .data_sum(Sum::new(
            AggregationTemporality::Cumulative,
            false, // non-monotonic => not aggregatable
            vec![
                NumberDataPoint::build()
                    .time_unix_nano(ts)
                    .value_int(idx as i64 + 1)
                    .finish(),
            ],
        ))
        .finish()
}

/// Build the shared resource attributes (String + Int).
fn resource_attrs() -> Vec<KeyValue> {
    vec![
        KeyValue::new("res_attr_str", AnyValue::new_string("resource_val")),
        KeyValue::new("res_attr_int", AnyValue::new_int(42i64)),
    ]
}

/// Build the shared scope attributes (Double + Bool).
fn scope_attrs() -> Vec<KeyValue> {
    vec![
        KeyValue::new("scope_attr_dbl", AnyValue::new_double(9.81)),
        KeyValue::new("scope_attr_bool", AnyValue::new_bool(true)),
    ]
}

/// Build a data-point attribute that varies per metric (Bytes).
/// `stream_id` must be globally unique across all aggregatable metrics in a
/// batch so each metric maps to a distinct stream.
fn dp_attr(stream_id: usize) -> Vec<KeyValue> {
    vec![KeyValue::new(
        "dp_attr_bytes",
        AnyValue::new_bytes(format!("stream_{stream_id}").into_bytes()),
    )]
}
