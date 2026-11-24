// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for `otel-arrow-dfe-policy-engine`.
//!
//! Covers filter scaling (`N = 1..10,000` mixed production policies on `dict256`,
//! `N = 1,000` shared-prefix literal/regex on `plain`, `N = 1,000` character-class
//! regex on `dict256`, and `Glob` patterns), compilation latency, end-to-end
//! filter + 4-stage transforms (`Rename -> Add -> Remove -> Redact`), and
//! root-only redaction over transport-optimized batches.

#![forbid(unsafe_code)]

use std::hint::black_box;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use otel_arrow_dfe_policy_engine::model::{
    Action, AttributePath, LogAdd, LogAttributeSelector, LogFieldSelector, LogMatcher,
    LogRecordField, LogRedact, LogRemove, LogRename, LogTargetSelector, LogTransformPolicy,
    NumericValue, Policy, Predicate, ScalarValue,
};
use otel_arrow_dfe_policy_engine::{EvaluationScratch, PolicyEngine};

const BATCH_SIZE: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DictCardinality {
    Medium256,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkloadProfile {
    W1ProductionMix,
    W2LiteralRegex,
    W3AdversarialRegex,
    W4bFullSpecTransforms,
    GlobMix,
}

const SERVICES: &[&str] = &[
    "checkout-service",
    "payment-gateway",
    "auth-service",
    "inventory-api",
    "recommendation-engine",
    "shipping-worker",
    "notification-hub",
    "fraud-detector",
];

const SEVERITIES: &[(&str, i32)] = &[
    ("TRACE", 1),
    ("DEBUG", 5),
    ("INFO", 9),
    ("WARN", 13),
    ("ERROR", 17),
    ("FATAL", 21),
];

const ROUTES: &[&str] = &[
    "/api/v1/orders/1042",
    "/api/v2/orders/9981",
    "/api/v1/checkout/session",
    "/api/v1/auth/token",
    "/healthz",
    "/metrics",
    "/api/v2/inventory/reserve",
    "/api/v1/payments/charge",
];

fn generate_logs_batch(num_rows: usize, cardinality: DictCardinality) -> OtapArrowRecords {
    let ids: Vec<u16> = (0..num_rows).map(|i| (i % 65535) as u16).collect();
    let res_ids: Vec<u16> = (0..num_rows).map(|i| (i % SERVICES.len()) as u16).collect();
    let scope_ids: Vec<u16> = (0..num_rows).map(|i| (i % 4) as u16).collect();

    let sev_texts: Vec<&str> = (0..num_rows)
        .map(|i| SEVERITIES[i % SEVERITIES.len()].0)
        .collect();
    let sev_nums: Vec<i32> = (0..num_rows)
        .map(|i| SEVERITIES[i % SEVERITIES.len()].1)
        .collect();
    let event_names: Vec<String> = (0..num_rows)
        .map(|i| format!("event.domain.action_{}", i % 32))
        .collect();
    let schema_urls: Vec<&str> = (0..num_rows)
        .map(|i| {
            if i % 2 == 0 {
                "https://opentelemetry.io/schemas/1.28.0"
            } else {
                "https://opentelemetry.io/schemas/1.27.0"
            }
        })
        .collect();

    let unique_count = match cardinality {
        DictCardinality::Medium256 => 256,
        DictCardinality::Plain => num_rows.max(1),
    };
    let unique_bodies: Vec<String> = (0..unique_count)
        .map(|u| match u % 8 {
            0 => format!(
                "POST /api/v1/orders/{} user_id=550e8400-e29b-41d4-a716-44665544{:04x} status=200",
                1000 + u,
                u
            ),
            1 => format!(
                "ERROR [auth] connection reset while verifying Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{:06x}",
                u
            ),
            2 => format!(
                "WARN deadline exceeded contacting upstream 10.{}.{}.{} after 250ms",
                (u >> 4) & 255,
                u & 255,
                (u * 7) & 255
            ),
            3 => format!(
                "DEBUG cache hit for session {:08x}-{:04x}-{:04x} in region us-east1",
                u * 17 + 0x1000_0000,
                u & 0xffff,
                (u * 3) & 0xffff
            ),
            4 => format!(
                "INFO processed payment charge ssn=123-45-{:04} amount={}.99",
                u % 10000,
                10 + (u % 500)
            ),
            5 => format!(
                "ERROR OutOfMemory in worker pool shard={} active_tasks={}",
                u % 16,
                64 + (u % 128)
            ),
            6 => format!(
                "POST /api/v2/orders/{} client_ip=192.168.{}.{} trace_ok=true",
                2000 + u,
                (u >> 3) & 255,
                u & 255
            ),
            _ => format!("INFO heartbeat ok seq={} node=worker-{}", u, u % 32),
        })
        .collect();

    let body_str_col: ArrayRef = match cardinality {
        DictCardinality::Medium256 => {
            let keys = UInt16Array::from_iter_values((0..num_rows).map(|i| (i % 256) as u16));
            let values =
                StringArray::from(unique_bodies.iter().map(String::as_str).collect::<Vec<_>>());
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(keys, Arc::new(values))
                    .expect("valid body string dictionary"),
            )
        }
        DictCardinality::Plain => {
            let values = StringArray::from(
                (0..num_rows)
                    .map(|i| unique_bodies[i % unique_bodies.len()].as_str())
                    .collect::<Vec<_>>(),
            );
            Arc::new(values)
        }
    };

    let sev_text_col: ArrayRef = match cardinality {
        DictCardinality::Medium256 => {
            let keys =
                UInt16Array::from_iter_values((0..num_rows).map(|i| (i % SEVERITIES.len()) as u16));
            let values = StringArray::from(SEVERITIES.iter().map(|(s, _)| *s).collect::<Vec<_>>());
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(keys, Arc::new(values))
                    .expect("valid severity_text dictionary"),
            )
        }
        DictCardinality::Plain => Arc::new(StringArray::from(sev_texts)),
    };

    let unique_events: Vec<String> = (0..32)
        .map(|i| format!("event.domain.action_{i}"))
        .collect();
    let event_name_col: ArrayRef = match cardinality {
        DictCardinality::Medium256 => {
            let keys = UInt16Array::from_iter_values((0..num_rows).map(|i| (i % 32) as u16));
            let values =
                StringArray::from(unique_events.iter().map(String::as_str).collect::<Vec<_>>());
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(keys, Arc::new(values))
                    .expect("valid event_name dictionary"),
            )
        }
        DictCardinality::Plain => Arc::new(StringArray::from(event_names)),
    };

    let body_types =
        UInt8Array::from_iter_values(std::iter::repeat_n(AttributeValueType::Str as u8, num_rows));
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_STR,
                body_str_col.data_type().clone(),
                true,
            )),
        ]
        .into(),
        vec![Arc::new(body_types), body_str_col],
        Some(NullBuffer::from(vec![true; num_rows])),
    );

    let trace_ids: Vec<[u8; 16]> = (0..num_rows)
        .map(|i| {
            let mut b = [0u8; 16];
            b[0] = 0x0a;
            b[14] = ((i >> 8) & 0xff) as u8;
            b[15] = ((i & 0xff) as u8).max(1);
            b
        })
        .collect();
    let span_ids: Vec<[u8; 8]> = (0..num_rows)
        .map(|i| {
            let mut b = [0u8; 8];
            b[0] = 0x0b;
            b[7] = ((i & 0xff) as u8).max(1);
            b
        })
        .collect();
    let trace_id_arr = FixedSizeBinaryArray::try_from_iter(trace_ids.iter().map(|b| b.as_slice()))
        .expect("valid trace_id array");
    let span_id_arr = FixedSizeBinaryArray::try_from_iter(span_ids.iter().map(|b| b.as_slice()))
        .expect("valid span_id array");

    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(res_ids))],
        None,
    );

    let scope_names: Vec<&str> = (0..num_rows)
        .map(|i| match i % 4 {
            0 => "otel.http.logger",
            1 => "otel.grpc.logger",
            2 => "otel.db.logger",
            _ => "otel.worker.logger",
        })
        .collect();
    let scope_versions: Vec<&str> = vec!["1.0.0"; num_rows];
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(scope_ids)),
            Arc::new(StringArray::from(scope_names)),
            Arc::new(StringArray::from(scope_versions)),
        ],
        None,
    );

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(
            consts::SEVERITY_TEXT,
            sev_text_col.data_type().clone(),
            true,
        ),
        Field::new(consts::EVENT_NAME, event_name_col.data_type().clone(), true),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(ids)),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(sev_nums)),
            sev_text_col,
            event_name_col,
            Arc::new(trace_id_arr),
            Arc::new(span_id_arr),
            Arc::new(StringArray::from(schema_urls)),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .expect("valid logs root RecordBatch");

    let attr_rows = num_rows * 4;
    let mut la_pids = Vec::with_capacity(attr_rows);
    let mut la_keys = Vec::with_capacity(attr_rows);
    let mut la_types = Vec::with_capacity(attr_rows);
    let mut la_strs: Vec<Option<String>> = Vec::with_capacity(attr_rows);
    let mut la_ints: Vec<Option<i64>> = Vec::with_capacity(attr_rows);
    let mut la_doubles: Vec<Option<f64>> = Vec::with_capacity(attr_rows);
    let mut la_bools: Vec<Option<bool>> = Vec::with_capacity(attr_rows);
    let mut la_bytes: Vec<Option<&[u8]>> = Vec::with_capacity(attr_rows);

    for r in 0..num_rows {
        let pid = (r % 65535) as u16;
        la_pids.push(pid);
        la_keys.push("http.route");
        la_types.push(AttributeValueType::Str as u8);
        la_strs.push(Some(ROUTES[r % ROUTES.len()].to_owned()));
        la_ints.push(None);
        la_doubles.push(None);
        la_bools.push(None);
        la_bytes.push(None);

        la_pids.push(pid);
        la_keys.push("env");
        la_types.push(AttributeValueType::Str as u8);
        la_strs.push(Some(if r % 3 == 0 { "prod" } else { "staging" }.to_owned()));
        la_ints.push(None);
        la_doubles.push(None);
        la_bools.push(None);
        la_bytes.push(None);

        la_pids.push(pid);
        la_keys.push("http.status_code");
        la_types.push(AttributeValueType::Int as u8);
        la_strs.push(None);
        la_ints.push(Some(match r % 5 {
            0 => 200,
            1 => 201,
            2 => 400,
            3 => 404,
            _ => 503,
        }));
        la_doubles.push(None);
        la_bools.push(None);
        la_bytes.push(None);

        la_pids.push(pid);
        la_keys.push("latency_ms");
        la_types.push(AttributeValueType::Double as u8);
        la_strs.push(None);
        la_ints.push(None);
        la_doubles.push(Some((r % 500) as f64 + 1.5));
        la_bools.push(None);
        la_bytes.push(None);
    }

    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
    ]));
    let log_attrs_rb = RecordBatch::try_new(
        attr_schema.clone(),
        vec![
            Arc::new(UInt16Array::from(la_pids)),
            Arc::new(StringArray::from(la_keys)),
            Arc::new(UInt8Array::from(la_types)),
            Arc::new(StringArray::from(la_strs)),
            Arc::new(Int64Array::from(la_ints)),
            Arc::new(Float64Array::from(la_doubles)),
            Arc::new(BooleanArray::from(la_bools)),
            Arc::new(BinaryArray::from_opt_vec(la_bytes)),
        ],
    )
    .expect("valid LogAttrs RecordBatch");

    let res_count = SERVICES.len();
    let res_pids: Vec<u16> = (0..res_count as u16).collect();
    let res_keys: Vec<&str> = vec!["service.name"; res_count];
    let res_types: Vec<u8> = vec![AttributeValueType::Str as u8; res_count];
    let res_strs: Vec<Option<&str>> = SERVICES.iter().map(|&s| Some(s)).collect();
    let res_attrs_rb = RecordBatch::try_new(
        attr_schema,
        vec![
            Arc::new(UInt16Array::from(res_pids)),
            Arc::new(StringArray::from(res_keys)),
            Arc::new(UInt8Array::from(res_types)),
            Arc::new(StringArray::from(res_strs)),
            Arc::new(Int64Array::from(vec![None; res_count])),
            Arc::new(Float64Array::from(vec![None; res_count])),
            Arc::new(BooleanArray::from(vec![None; res_count])),
            Arc::new(BinaryArray::from_opt_vec(vec![None; res_count])),
        ],
    )
    .expect("valid ResourceAttrs RecordBatch");

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records
        .set(ArrowPayloadType::Logs, root_rb)
        .expect("set Logs payload");
    records
        .set(ArrowPayloadType::LogAttrs, log_attrs_rb)
        .expect("set LogAttrs payload");
    records
        .set(ArrowPayloadType::ResourceAttrs, res_attrs_rb)
        .expect("set ResourceAttrs payload");
    records
}

fn generate_policies(num_policies: usize, profile: WorkloadProfile) -> Vec<Policy> {
    let mut policies = Vec::with_capacity(num_policies);
    for i in 0..num_policies {
        let id = format!("policy_{i:06}");
        match profile {
            WorkloadProfile::W1ProductionMix => {
                let bucket = i % 20;
                let action = if i % 7 == 0 {
                    Action::Keep
                } else {
                    Action::Drop
                };
                let matcher = if bucket < 6 {
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::single("http.route")),
                        predicate: Predicate::Equals(ScalarValue::String(format!(
                            "/api/v1/orders/{}",
                            1000 + i
                        ))),
                        negate: false,
                    }
                } else if bucket < 12 {
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::EventName),
                        predicate: Predicate::Equals(ScalarValue::String(format!(
                            "event.domain.action_{}",
                            i % 64
                        ))),
                        negate: false,
                    }
                } else if bucket < 15 {
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::single(
                            "http.status_code",
                        )),
                        predicate: Predicate::Gte(NumericValue::Int(500 + (i as i64 % 50))),
                        negate: false,
                    }
                } else if bucket < 18 {
                    let needle = match i % 3 {
                        0 => format!("connection reset #{}", i / 3),
                        1 => format!("deadline exceeded #{}", i / 3),
                        _ => format!("OutOfMemory shard={}", i % 32),
                    };
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Contains(ScalarValue::String(needle)),
                        negate: false,
                    }
                } else {
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Regex(format!(
                            r"POST /api/v[12]/orders/{}[0-9]*",
                            1000 + (i % 500)
                        )),
                        negate: false,
                    }
                };
                policies.push(Policy::log_filter(id, vec![matcher], action));
            }
            WorkloadProfile::W2LiteralRegex => {
                let matcher = if i % 2 == 0 {
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Contains(ScalarValue::String(format!(
                            "orders/{} user_id=",
                            1000 + i
                        ))),
                        negate: false,
                    }
                } else {
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Regex(format!(
                            r"ERROR \[auth\] connection reset.*{:04x}",
                            i % 4096
                        )),
                        negate: false,
                    }
                };
                policies.push(Policy::log_filter(id, vec![matcher], Action::Drop));
            }
            WorkloadProfile::W3AdversarialRegex => {
                let pat = format!(r"[a-f0-9]{{8}}-{:02x}[a-f0-9]{{2}}-[a-f0-9]{{4}}", i & 0xff);
                let matcher = LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Regex(pat),
                    negate: false,
                };
                policies.push(Policy::log_filter(id, vec![matcher], Action::Drop));
            }
            WorkloadProfile::GlobMix => {
                let matcher = if i % 2 == 0 {
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::Body),
                        predicate: Predicate::Glob(format!("POST /api/v?/orders/{}*", 1000 + i)),
                        negate: false,
                    }
                } else {
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::single("http.route")),
                        predicate: Predicate::Glob(format!("/api/v?/orders/{}*", 1000 + i)),
                        negate: false,
                    }
                };
                policies.push(Policy::log_filter(id, vec![matcher], Action::Drop));
            }
            WorkloadProfile::W4bFullSpecTransforms => {
                if i % 2 == 0 {
                    let action = if i % 6 == 0 {
                        Action::Keep
                    } else {
                        Action::Drop
                    };
                    let matcher = LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::single(
                            "http.status_code",
                        )),
                        predicate: Predicate::Equals(ScalarValue::Int(500 + (i as i64 % 20))),
                        negate: false,
                    };
                    policies.push(Policy::log_filter(id, vec![matcher], action));
                } else {
                    let matcher = LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                        predicate: Predicate::Equals(ScalarValue::String(
                            SEVERITIES[i % SEVERITIES.len()].0.to_owned(),
                        )),
                        negate: false,
                    };
                    let tx = LogTransformPolicy {
                        matches: vec![matcher],
                        rename: vec![LogRename {
                            from: LogAttributeSelector::LogAttribute(AttributePath::single("env")),
                            to: LogAttributeSelector::LogAttribute(AttributePath::single(
                                "deployment.environment",
                            )),
                            upsert: false,
                        }],
                        add: vec![LogAdd {
                            target: LogTargetSelector::LogAttribute(AttributePath::single(
                                "policy.applied",
                            )),
                            value: ScalarValue::String(id.clone()),
                            upsert: true,
                        }],
                        remove: vec![LogRemove {
                            target: LogAttributeSelector::LogAttribute(AttributePath::single(
                                "latency_ms",
                            )),
                        }],
                        redact: vec![LogRedact {
                            target: LogTargetSelector::RecordField(LogRecordField::Body),
                            replacement: Some("ssn=[REDACTED-$1]".to_owned()),
                            regex: Some(r"ssn=([0-9]{3})-[0-9]{2}-[0-9]{4}".to_owned()),
                        }],
                    };
                    policies.push(Policy::log_transform(id, tx));
                }
            }
        }
    }
    policies
}

fn compile_strict(policies: &[Policy]) -> PolicyEngine {
    let out = PolicyEngine::compile(policies).expect("policy compilation succeeded");
    assert!(
        out.errors.is_empty(),
        "unexpected compile errors: {:?}",
        out.errors
    );
    out.engine
}

fn bench_filter_w1_mixed(c: &mut Criterion) {
    let batch = generate_logs_batch(BATCH_SIZE, DictCardinality::Medium256);

    let mut group = c.benchmark_group("filter_w1_mixed_dict256");
    let _ = group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    for &n in &[1usize, 10, 100, 1_000, 10_000] {
        let policies = generate_policies(n, WorkloadProfile::W1ProductionMix);
        let engine = compile_strict(&policies);
        let mut scratch = EvaluationScratch::new();
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let sel = engine
                    .evaluate_filter(black_box(&batch), &mut scratch, None)
                    .expect("evaluate_filter succeeded");
                black_box(sel.len())
            });
        });
    }
    group.finish();

    // Exact per-policy stats mode disables Keep-all/Drop-all short-circuiting so both
    // Keep and Drop plans always execute across all rows (apples-to-apples with non-short-circuit baselines).
    let mut stats_group = c.benchmark_group("filter_w1_mixed_dict256_stats");
    let _ = stats_group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    for &n in &[1usize, 10, 100, 1_000, 10_000] {
        let policies = generate_policies(n, WorkloadProfile::W1ProductionMix);
        let engine = compile_strict(&policies);
        let mut scratch = EvaluationScratch::new();
        let mut stats = engine.new_match_stats();
        let _ = stats_group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                stats.reset();
                let sel = engine
                    .evaluate_filter(black_box(&batch), &mut scratch, Some(&mut stats))
                    .expect("evaluate_filter succeeded");
                black_box(sel.len())
            });
        });
    }
    stats_group.finish();
}

fn bench_filter_regex_and_glob(c: &mut Criterion) {
    let plain_batch = generate_logs_batch(BATCH_SIZE, DictCardinality::Plain);
    let dict_batch = generate_logs_batch(BATCH_SIZE, DictCardinality::Medium256);

    let mut group = c.benchmark_group("filter_string_workloads");
    let _ = group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    let w2_policies = generate_policies(1_000, WorkloadProfile::W2LiteralRegex);
    let w2_engine = compile_strict(&w2_policies);
    let mut scratch = EvaluationScratch::new();
    let _ = group.bench_function("w2_shared_prefix_plain_1000", |b| {
        b.iter(|| {
            let sel = w2_engine
                .evaluate_filter(black_box(&plain_batch), &mut scratch, None)
                .expect("evaluate_filter succeeded");
            black_box(sel.len())
        });
    });

    let w3_policies = generate_policies(1_000, WorkloadProfile::W3AdversarialRegex);
    let w3_engine = compile_strict(&w3_policies);
    let _ = group.bench_function("w3_char_class_dict256_1000", |b| {
        b.iter(|| {
            let sel = w3_engine
                .evaluate_filter(black_box(&dict_batch), &mut scratch, None)
                .expect("evaluate_filter succeeded");
            black_box(sel.len())
        });
    });

    for &n in &[10usize, 1_000] {
        let glob_policies = generate_policies(n, WorkloadProfile::GlobMix);
        let glob_engine = compile_strict(&glob_policies);
        let _ = group.bench_with_input(BenchmarkId::new("glob_dict256", n), &n, |b, _| {
            b.iter(|| {
                let sel = glob_engine
                    .evaluate_filter(black_box(&dict_batch), &mut scratch, None)
                    .expect("evaluate_filter succeeded");
                black_box(sel.len())
            });
        });
    }
    group.finish();
}

fn bench_compile(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_w1_mixed");
    for &n in &[10usize, 1_000, 10_000] {
        let policies = generate_policies(n, WorkloadProfile::W1ProductionMix);
        let _ = group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let out = PolicyEngine::compile(black_box(&policies)).expect("compile succeeded");
                black_box(out.engine)
            });
        });
    }
    group.finish();
}

fn bench_transforms(c: &mut Criterion) {
    let dict_batch = generate_logs_batch(BATCH_SIZE, DictCardinality::Medium256);
    let mut transport_batch = dict_batch.clone();
    transport_batch
        .encode_transport_optimized()
        .expect("encode_transport_optimized succeeded");

    let mut group = c.benchmark_group("eval_transforms_dict256");
    let _ = group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    let mut scratch = EvaluationScratch::new();
    for &n in &[10usize, 100, 1_000] {
        let policies = generate_policies(n, WorkloadProfile::W4bFullSpecTransforms);
        let engine = compile_strict(&policies);
        let _ = group.bench_with_input(BenchmarkId::new("w4b_4stage", n), &n, |b, _| {
            b.iter(|| {
                let out = engine
                    .evaluate(black_box(&dict_batch), &mut scratch, None)
                    .expect("evaluate succeeded");
                black_box(out.kept_rows)
            });
        });
    }

    let root_only_policies: Vec<Policy> = (0..10)
        .map(|i| {
            Policy::log_transform(
                format!("redact_ssn_{i:02}"),
                LogTransformPolicy {
                    matches: vec![LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                        predicate: Predicate::Equals(ScalarValue::String(
                            SEVERITIES[i % SEVERITIES.len()].0.to_owned(),
                        )),
                        negate: false,
                    }],
                    rename: Vec::new(),
                    add: Vec::new(),
                    remove: Vec::new(),
                    redact: vec![LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::Body),
                        replacement: Some("ssn=[REDACTED-$1]".to_owned()),
                        regex: Some(r"ssn=([0-9]{3})-[0-9]{2}-[0-9]{4}".to_owned()),
                    }],
                },
            )
        })
        .collect();
    let root_only_engine = compile_strict(&root_only_policies);
    let _ = group.bench_function("root_only_redact_transport_opt_10", |b| {
        b.iter(|| {
            let out = root_only_engine
                .evaluate(black_box(&transport_batch), &mut scratch, None)
                .expect("evaluate succeeded");
            black_box(out.kept_rows)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_filter_w1_mixed,
    bench_filter_regex_and_glob,
    bench_compile,
    bench_transforms,
);
criterion_main!(benches);
