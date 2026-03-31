// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_recordset_otlp_bridge::*;
use otap_df_pdata::testing::fixtures::logs_with_varying_attributes_and_properties;
use prost::Message;

fn generate_logs_batch(batch_size: usize) -> Vec<u8> {
    let logs_data = logs_with_varying_attributes_and_properties(batch_size);
    logs_data.encode_to_vec()
}

fn bench_log_pipeline(
    c: &mut Criterion,
    batch_sizes: &[usize],
    bench_group_name: &str,
    bench_pipeline_kql: &str,
) {
    let mut group = c.benchmark_group(bench_group_name);
    for batch_size in batch_sizes {
        let benchmark_id = BenchmarkId::new("batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, batch_size| {
            let batch = generate_logs_batch(**batch_size);
            let pipeline = parse_kql_query_into_pipeline(bench_pipeline_kql, None)
                .expect("can parse pipeline");
            b.iter_with_setup(
                || batch.clone(),
                |batch| {
                    process_protobuf_otlp_export_logs_service_request_using_pipeline(
                        &pipeline,
                        RecordSetEngineDiagnosticLevel::Warn,
                        &batch,
                    )
                    .expect("doesn't fail")
                },
            );
        });
    }
    group.finish();
}

fn bench_extend_pipelines(c: &mut Criterion) {
    let batch_sizes = [32, 1024, 8192];

    // Note: toint() for an unknown field generates a warning
    bench_log_pipeline(
        c,
        &batch_sizes,
        "toint_unknown_field",
        "source | extend a = toint(unknown)",
    );

    // Note: coalesce with inner toint() for an unknown field generates an info
    bench_log_pipeline(
        c,
        &batch_sizes,
        "coalesce_toint_unknown_field",
        "source | extend a = coalesce(toint(unknown), null)",
    );
}

mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_extend_pipelines
    );
}

criterion_main!(benches::benches);
