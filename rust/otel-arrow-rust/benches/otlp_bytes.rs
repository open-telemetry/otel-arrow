// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(missing_docs)]

//! This crate benchmarks OTLP and OTAP.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prost::Message;

use otel_arrow_rust::pdata::*;
use otel_arrow_rust::proto::opentelemetry::common::v1::*;
use otel_arrow_rust::proto::opentelemetry::logs::v1::*;
use otel_arrow_rust::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::*;
use otel_arrow_rust::proto::opentelemetry::metrics::v1::summary_data_point::*;
use otel_arrow_rust::proto::opentelemetry::metrics::v1::*;
use otel_arrow_rust::proto::opentelemetry::resource::v1::*;
use otel_arrow_rust::proto::opentelemetry::trace::v1::span::*;
use otel_arrow_rust::proto::opentelemetry::trace::v1::status::*;
use otel_arrow_rust::proto::opentelemetry::trace::v1::*;

fn create_traces_data() -> TracesData {
    let kv1 = KeyValue::new("k1", AnyValue::new_string("v1"));
    let kv2 = KeyValue::new("k2", AnyValue::new_int(2));
    let kvs = vec![kv1, kv2];

    let is1 = InstrumentationScope::new("library");

    let tid: TraceID = [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2].into();
    let sid: SpanID = [1, 2, 1, 2, 1, 2, 1, 2].into();
    let psid: SpanID = [2, 1, 2, 1, 2, 1, 2, 1].into();

    let s1 = Span::build(tid, sid, "myop", 123_000_000_000u64)
        .parent_span_id(psid)
        .attributes(kvs.clone())
        .flags(SpanFlags::ContextHasIsRemoteMask)
        .kind(SpanKind::Server)
        .trace_state("ot=th:0")
        .links(vec![Link::new(tid, sid)])
        .events(vec![Event::new("oops", 123_500_000_000u64)])
        .end_time_unix_nano(124_000_000_000u64)
        .status(Status::new("oh my!", StatusCode::Error))
        .dropped_attributes_count(1u32)
        .dropped_events_count(1u32)
        .dropped_links_count(1u32)
        .finish();

    let s2 = s1.clone();
    let sps = vec![s1, s2];

    let ss1 = ScopeSpans::build(is1.clone()).spans(sps.clone()).finish();
    let ss2 = ss1.clone();
    let sss = vec![ss1, ss2];

    let res = Resource::new(vec![]);

    let rs1 = ResourceSpans::build(res.clone())
        .scope_spans(sss.clone())
        .finish();
    let rs2 = rs1.clone();
    let rss = vec![rs1, rs2];

    TracesData::new(rss)
}

fn create_metrics_data() -> MetricsData {
    // Sum Metric
    let sum_metric = Metric::new_sum(
        "counter",
        Sum::new(
            AggregationTemporality::Delta,
            true,
            vec![
                NumberDataPoint::new_int(125_000_000_000u64, 123i64),
                NumberDataPoint::new_double(125_000_000_000u64, 123f64),
            ],
        ),
    );

    // Gauge Metric
    let gauge_metric = Metric::new_gauge(
        "gauge",
        Gauge::new(vec![
            NumberDataPoint::new_int(125_000_000_000u64, 123i64),
            NumberDataPoint::new_double(125_000_000_000u64, 123f64),
        ]),
    );

    // Histogram Metric
    let histogram_metric = Metric::new_histogram(
        "histogram",
        Histogram::new(
            AggregationTemporality::Delta,
            vec![
                HistogramDataPoint::build(125_000_000_000u64, [1u64, 2u64, 3u64], [1.0, 10.0])
                    .start_time_unix_nano(124_000_000_000u64)
                    .finish(),
                HistogramDataPoint::build(126_000_000_000u64, [3u64, 2u64, 1u64], [1.0, 10.0])
                    .start_time_unix_nano(125_000_000_000u64)
                    .count(100u64)
                    .sum(1000.0)
                    .min(0.1)
                    .max(10.1)
                    .finish(),
            ],
        ),
    );

    // Summary Metric
    let summary_metric = Metric::new_summary(
        "summary",
        Summary::new(vec![
            SummaryDataPoint::build(
                125_000_000_000u64,
                vec![
                    ValueAtQuantile::new(0.1, 0.1),
                    ValueAtQuantile::new(0.5, 2.1),
                    ValueAtQuantile::new(1.0, 10.1),
                ],
            )
            .start_time_unix_nano(124_000_000_000u64)
            .count(100u64)
            .sum(1000.0)
            .finish(),
            SummaryDataPoint::build(
                126_000_000_000u64,
                vec![
                    ValueAtQuantile::new(0.1, 0.5),
                    ValueAtQuantile::new(0.5, 2.5),
                    ValueAtQuantile::new(1.0, 10.5),
                ],
            )
            .start_time_unix_nano(124_000_000_000u64)
            .count(200u64)
            .sum(2000.0)
            .finish(),
        ]),
    );

    // Exponential Histogram Metric
    let exp_histogram_metric = Metric::new_exponential_histogram(
        "exp_histogram",
        ExponentialHistogram::new(
            AggregationTemporality::Delta,
            vec![
                ExponentialHistogramDataPoint::build(
                    125_000_000_000u64,
                    7,
                    Buckets::new(1, vec![3, 4, 5]),
                )
                .start_time_unix_nano(124_000_000_000u64)
                .count(17u64)
                .zero_count(2u64)
                .negative(Buckets::new(0, vec![1, 2]))
                .finish(),
            ],
        ),
    );

    let is1 = InstrumentationScope::new("library");
    let sm1 = ScopeMetrics::build(is1)
        .metrics(vec![
            sum_metric,
            gauge_metric,
            histogram_metric,
            summary_metric,
            exp_histogram_metric,
        ])
        .finish();

    let res = Resource::new(vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ]);

    let rm1 = ResourceMetrics::build(res)
        .scope_metrics(vec![sm1])
        .finish();

    MetricsData::new(vec![rm1])
}

fn create_logs_data() -> LogsData {
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ];
    let res = Resource::new(kvs.clone());

    let is1 = InstrumentationScope::new("library");

    let lr1 = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
        .attributes(kvs.clone())
        .finish();
    let lr2 = LogRecord::build(3_000_000_000u64, SeverityNumber::Info2, "event2")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("message text"))
        .severity_text("not on fire")
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let lr3 = LogRecord::build(3_000_000_000u64, SeverityNumber::Info2, "event3")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("here we go to 2us"))
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let lrs = vec![lr1, lr2, lr3];

    let sl1 = ScopeLogs::build(is1.clone())
        .log_records(lrs.clone())
        .schema_url("http://schema.opentelemetry.io")
        .finish();
    let sl2 = sl1.clone();
    let sls = vec![sl1, sl2];

    LogsData::new(vec![ResourceLogs::build(res).scope_logs(sls).finish()])
}

fn otlp_pdata_to_bytes_traces(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Traces Serialization");

    let traces_data = create_traces_data();

    _ = group.bench_function("TracesData", |b| {
        b.iter(|| {
            // Use black_box to prevent the compiler from optimizing away the results
            let mut buf = Vec::new();
            black_box(traces_data.encode(&mut buf)).expect("encoding success");
            black_box(buf)
        })
    });

    group.finish();
}

fn otlp_pdata_to_bytes_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Metrics Serialization");

    let metrics_data = create_metrics_data();

    _ = group.bench_function("MetricsData", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(metrics_data.encode(&mut buf)).expect("encoding success");
            black_box(buf)
        })
    });

    group.finish();
}

fn otlp_pdata_to_bytes_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs Serialization");

    let resource_logs = create_logs_data();

    _ = group.bench_function("ResourceLogs", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(resource_logs.encode(&mut buf)).expect("encoding success");
            black_box(buf)
        })
    });

    group.finish();
}

criterion_group!(
    otlp_bytes,
    otlp_pdata_to_bytes_traces,
    otlp_pdata_to_bytes_metrics,
    otlp_pdata_to_bytes_logs
);
criterion_main!(otlp_bytes);
