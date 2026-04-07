// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_recordset_otlp_bridge::*;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use prost::Message;

/// Generate a batch of synthetic log records with varying attributes and
/// severity levels.  This is a self-contained replacement for the fixture that
/// previously lived in `otap-df-pdata` so that the benchmark does not introduce
/// a cross-workspace dependency.
fn generate_logs_batch(batch_size: usize) -> Vec<u8> {
    let severity_levels: [(i32, &str); 4] = [
        (1, "TRACE"), // SeverityNumber::Trace
        (5, "DEBUG"), // SeverityNumber::Debug
        (9, "INFO"),  // SeverityNumber::Info
        (13, "WARN"), // SeverityNumber::Warn
    ];

    let log_records: Vec<LogRecord> = (0..batch_size)
        .map(|i| {
            let namespace = match i % 3 {
                0 => "main",
                1 => "otap_dataflow_engine",
                _ => "arrow::array",
            };

            let attributes = vec![
                KeyValue {
                    key: "code.namespace".into(),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue(namespace.into())),
                    }),
                },
                KeyValue {
                    key: "code.line.number".into(),
                    value: Some(AnyValue {
                        value: Some(Value::IntValue((i % 5) as i64)),
                    }),
                },
            ];

            let (severity_number, severity_text) = severity_levels[i % 4];

            LogRecord {
                time_unix_nano: i as u64,
                severity_number,
                severity_text: severity_text.into(),
                event_name: format!("event {i}"),
                attributes,
                ..Default::default()
            }
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

    request.encode_to_vec()
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
        let _ = group.bench_with_input(benchmark_id, batch_size, |b, batch_size| {
            let batch = generate_logs_batch(*batch_size);
            let pipeline = parse_kql_query_into_pipeline(bench_pipeline_kql, None)
                .expect("can parse pipeline");
            b.iter(|| {
                process_protobuf_otlp_export_logs_service_request_using_pipeline(
                    &pipeline,
                    RecordSetEngineDiagnosticLevel::Warn,
                    &batch,
                )
                .expect("doesn't fail")
            });
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
