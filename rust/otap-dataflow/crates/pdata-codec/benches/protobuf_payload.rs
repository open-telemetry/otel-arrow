// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark for owned OTLP message encoding into `PdataPayload`.
//!
//! From `rust/otap-dataflow`:
//! ```sh
//! cargo bench -p otel-arrow-dfe-pdata-codec --bench protobuf_payload
//! ```

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
use otel_arrow_dfe_pdata::testing::fixtures::{
    logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
    traces_with_full_resource_and_scope,
};
use otel_arrow_dfe_pdata_codec::PdataPayload;

fn logs(count: usize) -> OtlpProtoMessage {
    if count == 0 {
        return OtlpProtoMessage::Logs(Default::default());
    }
    let mut logs = logs_with_full_resource_and_scope();
    logs.resource_logs[0].scope_logs.truncate(1);
    let records = &mut logs.resource_logs[0].scope_logs[0].log_records;
    *records = records.iter().cycle().take(count).cloned().collect();
    OtlpProtoMessage::Logs(logs)
}

fn traces(count: usize) -> OtlpProtoMessage {
    if count == 0 {
        return OtlpProtoMessage::Traces(Default::default());
    }
    let mut traces = traces_with_full_resource_and_scope();
    let spans = &mut traces.resource_spans[0].scope_spans[0].spans;
    *spans = spans.iter().cycle().take(count).cloned().collect();
    OtlpProtoMessage::Traces(traces)
}

fn metrics(count: usize, many_metrics: bool) -> OtlpProtoMessage {
    if count == 0 {
        return OtlpProtoMessage::Metrics(Default::default());
    }
    let mut metrics = metrics_sum_with_full_resource_and_scope();
    let series = &mut metrics.resource_metrics[0].scope_metrics[0].metrics;
    let Some(Data::Sum(sum)) = &mut series[0].data else {
        panic!("fixture must contain a sum");
    };
    let points = &mut sum.data_points;
    *points = points
        .iter()
        .cycle()
        .take(if many_metrics { 1 } else { count })
        .cloned()
        .collect();
    if many_metrics {
        *series = (0..count)
            .map(|index| {
                let mut metric = series[0].clone();
                metric.name = format!("counter.{index}");
                metric
            })
            .collect();
    }
    OtlpProtoMessage::Metrics(metrics)
}

fn bench_conversion(c: &mut Criterion, shape: &str, count: usize, message: OtlpProtoMessage) {
    // Validate the fixture and initialize the immutable registry outside timing.
    let mut payload = PdataPayload::try_from(message.clone()).expect("payload encoding");
    assert_eq!(payload.is_empty(), count == 0);
    assert_eq!(payload.num_items(), count);

    let mut group = c.benchmark_group(format!("protobuf_payload/{shape}/{count}"));
    let bytes = payload.encoded_bytes().expect("encoded payload").len();
    if bytes != 0 {
        let _ = group.throughput(Throughput::Bytes(bytes as u64));
    }

    // Fixture cloning is setup, not encoding work. Owned input destruction is
    // part of the conversion; Criterion drops returned payloads outside timing.
    // LargeInput limits the number of large fixtures retained in each batch.
    let _ = group.bench_with_input(
        BenchmarkId::new("owned_otlp_to_payload", bytes),
        &message,
        |b, message| {
            b.iter_batched(
                || message.clone(),
                |input| PdataPayload::try_from(black_box(input)).expect("payload encoding"),
                BatchSize::LargeInput,
            );
        },
    );
    group.finish();
}

fn protobuf_payload(c: &mut Criterion) {
    for count in [0, 1, 128, 8_192] {
        bench_conversion(c, "logs", count, logs(count));
        bench_conversion(c, "traces", count, traces(count));
        bench_conversion(c, "metrics_few", count, metrics(count, false));
        if count > 1 {
            bench_conversion(c, "metrics_many", count, metrics(count, true));
        }
    }
}

criterion_group!(benches, protobuf_payload);
criterion_main!(benches);
