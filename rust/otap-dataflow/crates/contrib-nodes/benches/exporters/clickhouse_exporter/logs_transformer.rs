// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compares generic and specialized OTAP logs transformation for the ClickHouse exporter.

use std::hint::black_box;

use arrow::ipc::writer::StreamWriter;
use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_contrib_nodes::exporters::clickhouse_exporter::bench_support::LogsTransformBenchmark;
use otap_df_pdata::testing::{fixtures, round_trip::encode_logs};

const LOGS_PER_BATCH: usize = 8192;

fn bench_logs_transform(c: &mut Criterion) {
    let mut records = encode_logs(&fixtures::logs_with_varying_attributes_and_properties(
        LOGS_PER_BATCH,
    ));
    records
        .decode_transport_optimized_ids()
        .expect("decode transport-optimized IDs");

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

    group.finish();
}

criterion_group!(benches, bench_logs_transform);
criterion_main!(benches);
