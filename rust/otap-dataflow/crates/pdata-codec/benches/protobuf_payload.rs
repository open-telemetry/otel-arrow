// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Before/after benchmark for owned OTLP message encoding into `PdataPayload`.

use std::hint::black_box;

use bytes::BytesMut;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
use otel_arrow_dfe_pdata::testing::fixtures::{
    logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
    traces_with_full_resource_and_scope,
};
use otel_arrow_dfe_pdata_codec::PdataPayload;
use prost::Message;

// Keep the pre-optimization conversion here so both versions run with the same
// compiler, allocator, fixtures, and registry. Do not preallocate or reuse bytes.
fn before(message: OtlpProtoMessage) -> Result<PdataPayload, prost::EncodeError> {
    let mut bytes = BytesMut::new();
    Ok(match message {
        OtlpProtoMessage::Logs(logs) => {
            logs.encode(&mut bytes)?;
            OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into()
        }
        OtlpProtoMessage::Metrics(metrics) => {
            metrics.encode(&mut bytes)?;
            OtlpProtoBytes::ExportMetricsRequest(bytes.freeze()).into()
        }
        OtlpProtoMessage::Traces(traces) => {
            traces.encode(&mut bytes)?;
            OtlpProtoBytes::ExportTracesRequest(bytes.freeze()).into()
        }
    })
}

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
    // Validate equivalence and initialize the immutable registry outside timing.
    let old = before(message.clone()).expect("baseline encoding");
    let new = PdataPayload::try_from(message.clone()).expect("preallocated encoding");
    assert_eq!(old.signal_type(), new.signal_type());
    assert_eq!(old.encoding(), new.encoding());
    assert_eq!(old.encoded_bytes(), new.encoded_bytes());
    assert_eq!(old.is_empty(), count == 0);
    let mut counted = new.clone();
    assert_eq!(counted.num_items(), count);

    let mut group = c.benchmark_group(format!("protobuf_payload/{shape}/{count}"));
    let bytes = old.encoded_bytes().expect("encoded payload").len();
    if bytes != 0 {
        let _ = group.throughput(Throughput::Bytes(bytes as u64));
    }

    // Fixture cloning is setup, not encoding work. Owned input destruction is
    // part of both conversions; Criterion drops returned payloads outside timing.
    // LargeInput limits the number of large fixtures retained in each batch.
    let _ = group.bench_with_input(BenchmarkId::new("before", bytes), &message, |b, message| {
        b.iter_batched(
            || message.clone(),
            |input| before(black_box(input)).expect("baseline encoding"),
            BatchSize::LargeInput,
        );
    });
    let _ = group.bench_with_input(BenchmarkId::new("after", bytes), &message, |b, message| {
        b.iter_batched(
            || message.clone(),
            |input| PdataPayload::try_from(black_box(input)).expect("preallocated encoding"),
            BatchSize::LargeInput,
        );
    });
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
