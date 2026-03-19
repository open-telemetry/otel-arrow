// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark for Azure Monitor Transformer: OTLP logs -> JSON conversion.
//!
//! MacOS M4 Pro baseline results (1000 records):
//!   Original (per-record resource/scope mapping):  1.60ms (~625K records/s)
//!   Hoisted (resource/scope computed once):        1.36ms (~735K records/s)  +17%
//!   Hoisted + Direct Serialization (current):      425µs  (~2.35M records/s) +275%

#![allow(unused_results)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_contrib_nodes::exporters::azure_monitor_exporter::{Config, Transformer};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use prost::Message;
use serde_json::json;
use std::collections::HashMap;

use otap_df_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value::Value as OtelAnyValueEnum,
};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;

use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;

fn create_config() -> Config {
    use otap_df_contrib_nodes::exporters::azure_monitor_exporter::config::{
        ApiConfig, AuthConfig, SchemaConfig,
    };

    Config {
        api: ApiConfig {
            dcr_endpoint: "https://test.ingest.monitor.azure.com".into(),
            stream_name: "Custom-TestTable".into(),
            dcr: "dcr-test-rule-id".into(),
            schema: SchemaConfig {
                resource_mapping: HashMap::from([
                    ("service.name".into(), "ServiceName".into()),
                    ("service.version".into(), "ServiceVersion".into()),
                    ("host.name".into(), "HostName".into()),
                ]),
                scope_mapping: HashMap::from([
                    ("scope.name".into(), "ScopeName".into()),
                    ("scope.version".into(), "ScopeVersion".into()),
                ]),
                log_record_mapping: HashMap::from([
                    ("time_unix_nano".into(), json!("TimeGenerated")),
                    ("severity_text".into(), json!("SeverityText")),
                    ("severity_number".into(), json!("SeverityNumber")),
                    ("body".into(), json!("Body")),
                    ("trace_id".into(), json!("TraceId")),
                    ("span_id".into(), json!("SpanId")),
                    (
                        "attributes".into(),
                        json!({
                            "env": "Environment",
                            "request.id": "RequestId",
                            "user.id": "UserId"
                        }),
                    ),
                ]),
            },
            azure_monitor_source_resourceid: None,
        },
        auth: AuthConfig::default(),
    }
}

fn make_log_record(i: usize) -> LogRecord {
    LogRecord {
        time_unix_nano: 1_700_000_000_000_000_000 + (i as u64) * 1_000_000,
        observed_time_unix_nano: 1_700_000_000_000_000_000 + (i as u64) * 1_000_000,
        severity_number: 9, // INFO
        severity_text: "INFO".into(),
        body: Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue(format!(
                "Log message number {i}"
            ))),
        }),
        attributes: vec![
            KeyValue {
                key: "env".into(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("production".into())),
                }),
            },
            KeyValue {
                key: "request.id".into(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue(format!("req-{i:06}"))),
                }),
            },
            KeyValue {
                key: "user.id".into(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("user-42".into())),
                }),
            },
        ],
        trace_id: vec![
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef,
        ],
        span_id: vec![0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10],
        flags: 1,
        ..Default::default()
    }
}

fn make_request(
    num_resource_logs: usize,
    num_scope_logs: usize,
    records_per_scope: usize,
) -> Vec<u8> {
    let resource_logs: Vec<ResourceLogs> = (0..num_resource_logs)
        .map(|_| ResourceLogs {
            resource: Some(Resource {
                attributes: vec![
                    KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                    },
                    KeyValue {
                        key: "service.version".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("1.2.3".into())),
                        }),
                    },
                    KeyValue {
                        key: "host.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("host-01.prod".into())),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_logs: (0..num_scope_logs)
                .map(|_| ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "my-library".into(),
                        version: "0.1.0".into(),
                        attributes: vec![
                            KeyValue {
                                key: "scope.name".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("my-library".into())),
                                }),
                            },
                            KeyValue {
                                key: "scope.version".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("0.1.0".into())),
                                }),
                            },
                        ],
                        dropped_attributes_count: 0,
                    }),
                    log_records: (0..records_per_scope).map(make_log_record).collect(),
                    schema_url: String::new(),
                })
                .collect(),
            schema_url: String::new(),
        })
        .collect();

    let request = ExportLogsServiceRequest { resource_logs };
    request.encode_to_vec()
}

fn bench_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("transformer");

    let config = create_config();
    let transformer = Transformer::new(&config);

    // Varying record counts: 1 ResourceLogs, 1 ScopeLogs, N records
    for num_records in [10, 100, 1000] {
        let bytes = make_request(1, 1, num_records);
        let total_records = num_records;

        group.throughput(criterion::Throughput::Elements(total_records as u64));
        group.bench_with_input(
            BenchmarkId::new("1r_1s", total_records),
            &bytes,
            |b, bytes| {
                b.iter(|| {
                    let view = RawLogsData::new(bytes);
                    let result = transformer.convert_to_log_analytics(&view);
                    assert_eq!(result.len(), total_records);
                });
            },
        );
    }

    // Many scopes: 1 ResourceLogs, 10 ScopeLogs, 100 records each
    {
        let bytes = make_request(1, 10, 100);
        let total_records = 1000;

        group.throughput(criterion::Throughput::Elements(total_records as u64));
        group.bench_with_input(
            BenchmarkId::new("1r_10s", total_records),
            &bytes,
            |b, bytes| {
                b.iter(|| {
                    let view = RawLogsData::new(bytes);
                    let result = transformer.convert_to_log_analytics(&view);
                    assert_eq!(result.len(), total_records);
                });
            },
        );
    }

    // Many resources: 10 ResourceLogs, 1 ScopeLogs, 100 records each
    {
        let bytes = make_request(10, 1, 100);
        let total_records = 1000;

        group.throughput(criterion::Throughput::Elements(total_records as u64));
        group.bench_with_input(
            BenchmarkId::new("10r_1s", total_records),
            &bytes,
            |b, bytes| {
                b.iter(|| {
                    let view = RawLogsData::new(bytes);
                    let result = transformer.convert_to_log_analytics(&view);
                    assert_eq!(result.len(), total_records);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_transform);
criterion_main!(benches);
