// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compares generic and specialized OTAP logs transformation for the ClickHouse exporter.

use std::hint::black_box;

use arrow::ipc::writer::StreamWriter;
use bytes::Bytes;
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_contrib_nodes::exporters::clickhouse_exporter::bench_support::LogsTransformBenchmark;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::testing::{fixtures, round_trip::encode_logs};
use prost::Message;

const LOGS_PER_BATCH: usize = 8192;

fn bench_logs_transform(c: &mut Criterion) {
    let logs = fixtures::logs_with_varying_attributes_and_properties(LOGS_PER_BATCH);
    let mut records = encode_logs(&logs);
    records
        .decode_transport_optimized_ids()
        .expect("decode transport-optimized IDs");
    let request = Bytes::from(
        ExportLogsServiceRequest {
            resource_logs: logs.resource_logs,
        }
        .encode_to_vec(),
    );

    let mut group = c.benchmark_group("clickhouse_logs_transform");
    _ = group.throughput(Throughput::Elements(LOGS_PER_BATCH as u64));

    let mut generic = LogsTransformBenchmark::default();
    _ = group.bench_function("generic/8192", |b| {
        b.iter_batched(
            || records.clone(),
            |input| black_box(generic.transform_generic(input)),
            BatchSize::SmallInput,
        );
    });

    let mut fast = LogsTransformBenchmark::default();
    _ = group.bench_function("specialized/8192", |b| {
        b.iter(|| black_box(fast.transform_fast(black_box(&records))));
    });

    let output = fast.transform_fast(&records);
    _ = group.bench_function("arrow_stream/8192", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            let mut writer = StreamWriter::try_new(&mut bytes, output.schema_ref())
                .expect("create ArrowStream writer");
            writer.write(&output).expect("encode ClickHouse batch");
            writer.finish().expect("finish ArrowStream");
            black_box(bytes)
        });
    });

    let mut otlp_legacy = LogsTransformBenchmark::default();
    _ = group.bench_function("otlp_legacy/8192", |b| {
        b.iter_batched(
            || request.clone(),
            |input| black_box(otlp_legacy.transform_otlp_legacy(input)),
            BatchSize::SmallInput,
        );
    });

    let mut otlp_direct = LogsTransformBenchmark::default();
    _ = group.bench_function("otlp_direct/8192", |b| {
        b.iter(|| black_box(otlp_direct.transform_otlp_direct(black_box(&request))));
    });

    let direct_output = otlp_direct.transform_otlp_direct(&request);
    _ = group.bench_function("otlp_direct_arrow_stream/8192", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            let mut writer = StreamWriter::try_new(&mut bytes, direct_output.schema_ref())
                .expect("create direct ArrowStream writer");
            writer
                .write(&direct_output)
                .expect("encode direct ClickHouse batch");
            writer.finish().expect("finish direct ArrowStream");
            black_box(bytes)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_logs_transform);
criterion_main!(benches);
