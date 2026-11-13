// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for [`MetricStartTimeProcessor`] (`subtract_initial_point`).
//!
//! Recommended CLI invocation (produces Go `benchstat`-compatible output via `benchfmt`):
//! ```sh
//! cargo bench -p otel-arrow-dfe-core-nodes --no-default-features --features bench \
//!   --bench metric_start_time_processor
//! benchfmt criterion target/criterion \
//!   --pkg otel_arrow_dfe_core_nodes::metric_start_time --cpu 2 | tee ${bench}.txt
//! ```

#![allow(missing_docs)]

use std::hint::black_box;
use std::sync::Arc;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_channel::mpsc;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::control::{RuntimeCtrlMsgReceiver, runtime_ctrl_msg_channel};
use otel_arrow_dfe_engine::local::processor as local;
use otel_arrow_dfe_engine::message::{Message, Receiver, Sender};
use otel_arrow_dfe_engine::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use otel_arrow_dfe_engine::processor::{ProcessorWrapper, ProcessorWrapperRuntime};
use otel_arrow_dfe_engine::testing::node::test_node;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords};
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
    Histogram, HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
    ScopeMetrics, Sum, Summary, SummaryDataPoint,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use serde_json::{Value, json};

use otel_arrow_dfe_core_nodes::processors::metric_start_time_processor::{
    METRIC_START_TIME_PROCESSOR_FACTORY, METRIC_START_TIME_PROCESSOR_URN,
};

criterion_group!(benches, bench_process_batch);
criterion_main!(benches);

struct ProcessorHarness {
    processor: Box<dyn local::Processor<OtapPdata>>,
    effect_handler: local::EffectHandler<OtapPdata>,
    output_receiver: Receiver<OtapPdata>,
    _ctrl_rx: RuntimeCtrlMsgReceiver<OtapPdata>,
}

fn create_harness(cfg: Value) -> ProcessorHarness {
    let telemetry = InternalTelemetrySystem::default();
    let registry = telemetry.registry();
    let reporter = telemetry.reporter();

    let controller_ctx = ControllerContext::new(registry);
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("bench_grp".into(), "bench_pipe".into(), 0, 1, 0);

    let mut node_config = NodeUserConfig::new_processor_config(METRIC_START_TIME_PROCESSOR_URN);
    node_config.config = cfg;

    let proc_cfg = otel_arrow_dfe_engine::config::ProcessorConfig::new("metric_start_time_bench");

    let mut wrapper: ProcessorWrapper<OtapPdata> = (METRIC_START_TIME_PROCESSOR_FACTORY.create)(
        pipeline_ctx,
        test_node("metric_start_time_bench"),
        Arc::new(node_config),
        &proc_cfg,
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    )
    .expect("create processor");

    let (out_tx, out_rx) = mpsc::Channel::new(64);
    let _ = wrapper.set_pdata_sender(
        test_node("metric_start_time_bench"),
        "default".into(),
        Sender::new_local_mpsc_sender(out_tx),
    );
    let (_, dummy_rx) = mpsc::Channel::<OtapPdata>::new(1);
    let _ = wrapper.set_pdata_receiver(
        test_node("metric_start_time_bench"),
        Receiver::new_local_mpsc_receiver(dummy_rx),
    );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("setup runtime");
    let runtime = rt
        .block_on(wrapper.prepare_runtime(
            reporter,
            Interests::empty(),
            otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
        ))
        .expect("prepare runtime");

    match runtime {
        ProcessorWrapperRuntime::Local {
            mut effect_handler,
            processor,
            ..
        } => {
            let (ctrl_tx, ctrl_rx) = runtime_ctrl_msg_channel(16);
            effect_handler.set_runtime_ctrl_msg_sender(ctrl_tx);
            ProcessorHarness {
                processor,
                effect_handler,
                output_receiver: Receiver::new_local_mpsc_receiver(out_rx),
                _ctrl_rx: ctrl_rx,
            }
        }
        ProcessorWrapperRuntime::Shared { .. } => unreachable!("always local"),
    }
}

#[inline]
fn run_one(rt: &tokio::runtime::Runtime, harness: &mut ProcessorHarness, pdata: OtapPdata) {
    rt.block_on(async {
        harness
            .processor
            .process(Message::PData(pdata), &mut harness.effect_handler)
            .await
            .expect("process");
    });
    while let Ok(out) = harness.output_receiver.try_recv() {
        let _ = black_box(out);
    }
}

fn kv_str(k: &str, v: &str) -> KeyValue {
    KeyValue::new(k, AnyValue::new_string(v))
}

fn dp_attrs(series_idx: usize) -> Vec<KeyValue> {
    const METHODS: [&str; 4] = ["GET", "POST", "PUT", "DELETE"];
    vec![
        kv_str("http.method", METHODS[series_idx % METHODS.len()]),
        KeyValue::new(
            "http.status_code",
            AnyValue::new_int(200 + (series_idx % 5) as i64),
        ),
        kv_str("endpoint", &format!("/api/v1/item/{series_idx}")),
    ]
}

fn make_otap_pdata(md: MetricsData) -> OtapPdata {
    OtapPdata::new_default(otlp_to_otap(&OtlpProtoMessage::Metrics(md)).into())
}

fn build_sum_batch(ts: u64, base_val: i64, start_ts: u64) -> OtapPdata {
    let metrics: Vec<_> = (0..10)
        .map(|m_idx| {
            let dps: Vec<_> = (0..100)
                .map(|s_idx| {
                    let mut b = NumberDataPoint::build()
                        .time_unix_nano(ts)
                        .attributes(dp_attrs(s_idx))
                        .value_int(base_val + s_idx as i64);
                    if start_ts != 0 {
                        b = b.start_time_unix_nano(start_ts);
                    }
                    b.finish()
                })
                .collect();
            Metric::build()
                .name(format!("http_requests_total_{m_idx}"))
                .data_sum(Sum::new(AggregationTemporality::Cumulative, true, dps))
                .finish()
        })
        .collect();

    make_otap_pdata(MetricsData::new(vec![ResourceMetrics::new(
        Resource::build()
            .attributes(vec![
                kv_str("service.name", "frontend"),
                kv_str("host.name", "node-01"),
            ])
            .finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().name("http").finish(),
            metrics,
        )],
    )]))
}

fn build_hist_batch(ts: u64, mult: u64) -> OtapPdata {
    let mut metrics = Vec::with_capacity(10);
    for m_idx in 0..5 {
        let h_dps: Vec<_> = (0..100)
            .map(|s_idx| {
                HistogramDataPoint::build()
                    .time_unix_nano(ts)
                    .attributes(dp_attrs(s_idx))
                    .count(10 * mult)
                    .sum(55.0 * mult as f64)
                    .explicit_bounds(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5])
                    .bucket_counts(vec![mult; 10])
                    .finish()
            })
            .collect();
        metrics.push(
            Metric::build()
                .name(format!("http_duration_seconds_{m_idx}"))
                .data_histogram(Histogram::new(AggregationTemporality::Cumulative, h_dps))
                .finish(),
        );

        let eh_dps: Vec<_> = (0..100)
            .map(|s_idx| {
                ExponentialHistogramDataPoint::build()
                    .time_unix_nano(ts)
                    .attributes(dp_attrs(s_idx))
                    .scale(3)
                    .count(20 * mult)
                    .sum(100.0 * mult as f64)
                    .zero_count(2 * mult)
                    .positive(Buckets::new(0, vec![mult; 12]))
                    .negative(Buckets::new(0, vec![mult; 6]))
                    .finish()
            })
            .collect();
        metrics.push(
            Metric::build()
                .name(format!("rpc_duration_exp_{m_idx}"))
                .data_exponential_histogram(ExponentialHistogram::new(
                    AggregationTemporality::Cumulative,
                    eh_dps,
                ))
                .finish(),
        );
    }

    make_otap_pdata(MetricsData::new(vec![ResourceMetrics::new(
        Resource::build()
            .attributes(vec![kv_str("service.name", "checkout")])
            .finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().name("rpc").finish(),
            metrics,
        )],
    )]))
}

fn build_mixed_batch(ts: u64, mult: u64, num_per_group: usize, with_exemplars: bool) -> OtapPdata {
    let make_ex = |v: i64| {
        if with_exemplars {
            vec![
                Exemplar::build()
                    .time_unix_nano(ts)
                    .value_int(v)
                    .filtered_attributes(vec![kv_str("trace_id", "abcd")])
                    .finish(),
            ]
        } else {
            vec![]
        }
    };

    let gauge_dps: Vec<_> = (0..num_per_group)
        .map(|i| {
            NumberDataPoint::build()
                .time_unix_nano(ts)
                .attributes(dp_attrs(i))
                .value_double(42.5)
                .finish()
        })
        .collect();
    let delta_dps: Vec<_> = (0..num_per_group)
        .map(|i| {
            NumberDataPoint::build()
                .time_unix_nano(ts)
                .attributes(dp_attrs(i))
                .value_int(5)
                .finish()
        })
        .collect();
    let cum_sum_dps: Vec<_> = (0..num_per_group)
        .map(|i| {
            NumberDataPoint::build()
                .time_unix_nano(ts)
                .attributes(dp_attrs(i))
                .value_int((100 * mult) as i64 + i as i64)
                .exemplars(make_ex(1))
                .finish()
        })
        .collect();
    let cum_hist_dps: Vec<_> = (0..num_per_group)
        .map(|i| {
            HistogramDataPoint::build()
                .time_unix_nano(ts)
                .attributes(dp_attrs(i))
                .count(8 * mult)
                .sum(32.0 * mult as f64)
                .explicit_bounds(vec![0.1, 0.5, 1.0])
                .bucket_counts(vec![2 * mult; 4])
                .exemplars(make_ex(1))
                .finish()
        })
        .collect();
    let summary_dps: Vec<_> = (0..num_per_group)
        .map(|i| {
            SummaryDataPoint::build()
                .time_unix_nano(ts)
                .attributes(dp_attrs(i))
                .count(10 * mult)
                .sum(50.0 * mult as f64)
                .quantile_values(vec![ValueAtQuantile::new(0.99, 0.8)])
                .finish()
        })
        .collect();

    let metrics = vec![
        Metric::build()
            .name("cpu_utilization")
            .data_gauge(Gauge::new(gauge_dps))
            .finish(),
        Metric::build()
            .name("delta_events")
            .data_sum(Sum::new(AggregationTemporality::Delta, true, delta_dps))
            .finish(),
        Metric::build()
            .name("requests_total")
            .data_sum(Sum::new(
                AggregationTemporality::Cumulative,
                true,
                cum_sum_dps,
            ))
            .finish(),
        Metric::build()
            .name("latency_seconds")
            .data_histogram(Histogram::new(
                AggregationTemporality::Cumulative,
                cum_hist_dps,
            ))
            .finish(),
        Metric::build()
            .name("legacy_summary")
            .data_summary(Summary::new(summary_dps))
            .finish(),
    ];

    make_otap_pdata(MetricsData::new(vec![ResourceMetrics::new(
        Resource::build()
            .attributes(vec![kv_str("service.name", "payments")])
            .finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().name("svc").finish(),
            metrics,
        )],
    )]))
}

fn bench_process_batch(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let sum_seed = build_sum_batch(10_000_000_000, 1_000, 0);
    let sum_steady = build_sum_batch(20_000_000_000, 2_000, 0);
    let mut sum_harness = create_harness(json!({}));
    run_one(&rt, &mut sum_harness, sum_seed);

    let hist_seed = build_hist_batch(10_000_000_000, 1);
    let hist_steady = build_hist_batch(20_000_000_000, 2);
    let mut hist_harness = create_harness(json!({}));
    run_one(&rt, &mut hist_harness, hist_seed);

    let mixed_seed = build_mixed_batch(10_000_000_000, 1, 200, false);
    let mixed_steady = build_mixed_batch(20_000_000_000, 2, 200, false);
    let mut mixed_harness = create_harness(json!({}));
    run_one(&rt, &mut mixed_harness, mixed_seed);

    let preset_batch = build_sum_batch(20_000_000_000, 2_000, 5_000_000_000);
    let mut preset_harness = create_harness(json!({}));

    let drop_batch = build_mixed_batch(10_000_000_000, 1, 50, true);
    let empty_batch = OtapPdata::new_default(OtapArrowRecords::Metrics(Metrics::default()).into());
    let mut drop_harness = create_harness(json!({ "gc_interval": "1ns" }));

    let mut group = c.benchmark_group("ProcessBatch");
    let _ = group.throughput(Throughput::Elements(1_000));

    let _ = group.bench_function("case=steady_sum_1000dp", |b| {
        b.iter(|| run_one(&rt, &mut sum_harness, sum_steady.clone()));
    });
    let _ = group.bench_function("case=steady_hist_1000dp", |b| {
        b.iter(|| run_one(&rt, &mut hist_harness, hist_steady.clone()));
    });
    let _ = group.bench_function("case=steady_mixed_1000dp", |b| {
        b.iter(|| run_one(&rt, &mut mixed_harness, mixed_steady.clone()));
    });
    let _ = group.bench_function("case=passthrough_preset_1000dp", |b| {
        b.iter(|| run_one(&rt, &mut preset_harness, preset_batch.clone()));
    });

    let _ = group.throughput(Throughput::Elements(250));
    let _ = group.bench_function("case=initial_drop_mixed_250dp", |b| {
        b.iter(|| {
            run_one(&rt, &mut drop_harness, drop_batch.clone());
            run_one(&rt, &mut drop_harness, empty_batch.clone());
            run_one(&rt, &mut drop_harness, empty_batch.clone());
        });
    });

    group.finish();
}
