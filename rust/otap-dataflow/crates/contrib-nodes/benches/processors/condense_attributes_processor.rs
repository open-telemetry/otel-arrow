// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the Condense Attributes processor.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_contrib_nodes::processors::condense_attributes_processor::CondenseAttributesProcessor;
use otap_df_engine::testing::test_pipeline_ctx;
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::proto::opentelemetry::{
    collector::logs::v1::ExportLogsServiceRequest,
    common::v1::{AnyValue, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
};
use otap_df_pdata::{OtapArrowRecords, OtlpProtoBytes};
use prost::Message as _;
use serde_json::json;
use std::hint::black_box;

fn generate_records(log_count: usize) -> OtapArrowRecords {
    let log_records = (0..log_count)
        .map(|index| LogRecord {
            attributes: vec![
                KeyValue::new("service.name", AnyValue::new_string("checkout")),
                KeyValue::new("host.name", AnyValue::new_string("host-01")),
                KeyValue::new("http.method", AnyValue::new_string("GET")),
                KeyValue::new("http.status_code", AnyValue::new_int(200)),
                KeyValue::new("payload", AnyValue::new_bytes(b"preserved bytes")),
                KeyValue::new(
                    "labels",
                    AnyValue::new_kvlist(vec![KeyValue::new(
                        "region",
                        AnyValue::new_string("us-east"),
                    )]),
                ),
            ],
            time_unix_nano: index as u64,
            ..Default::default()
        })
        .collect();
    let request = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };

    OtlpProtoBytes::ExportLogsRequest(request.encode_to_vec().into())
        .try_into_with_default()
        .expect("convert generated logs to Arrow records")
}

fn bench_condense(c: &mut Criterion) {
    let (pipeline_ctx, _) = test_pipeline_ctx();
    let processor = CondenseAttributesProcessor::from_config(
        pipeline_ctx,
        &json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["service.name", "host.name", "http.method", "http.status_code"]
        }),
    )
    .expect("create processor");
    let mut group = c.benchmark_group("condense_attributes");

    for log_count in [32, 1024, 8192] {
        let records = generate_records(log_count);
        let _ = group.bench_with_input(
            BenchmarkId::new("logs", log_count),
            &records,
            |b, records| {
                b.iter_batched_ref(
                    || records.clone(),
                    |records| {
                        let condensed = processor
                            .condense(records)
                            .expect("condense generated records");
                        let _ = black_box(condensed);
                        let _ = black_box(records);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_condense);
criterion_main!(benches);
