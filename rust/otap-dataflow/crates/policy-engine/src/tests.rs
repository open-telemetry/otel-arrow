// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and conformance tests for `PolicyEngine` (`open-telemetry/otel-arrow#4726`).

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use arrow::array::{
    Array, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float64Array, Int32Array, Int64Array,
    RecordBatch, StringArray, StructArray, UInt8Array, UInt16Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use roaring::RoaringBitmap;

use crate::model::{
    Action, AttributePath, LogAdd, LogFieldSelector, LogMatcher, LogRecordField, LogRedact,
    LogRemove, LogRename, LogTargetSelector, LogTransformPolicy, NumericValue, Policy, Predicate,
    ScalarValue,
};
use crate::otap_ops::{ExtractedValue, LogsBatchView};
use crate::{PolicyEngine, PolicyEvaluationEngine};

fn compile_engine(policies: &[Policy]) -> PolicyEngine {
    PolicyEngine::compile(policies).expect("PolicyEngine compile")
}

fn build_custom_test_batch(
    bodies: Vec<Option<&str>>,
    sev_nums: Vec<i32>,
    sev_texts: Vec<&str>,
    trace_ids: Vec<[u8; 16]>,
    span_ids: Vec<[u8; 8]>,
    attr_rows: Vec<(u16, &str, ScalarValue)>,
    cbor_attr_rows: Vec<(u16, &str, ciborium::Value)>,
) -> OtapArrowRecords {
    let n = bodies.len();
    let ids: Vec<u16> = (0..n as u16).collect();
    let body_types: Vec<u8> = bodies
        .iter()
        .map(|b| {
            if b.is_some() {
                AttributeValueType::Str as u8
            } else {
                AttributeValueType::Empty as u8
            }
        })
        .collect();
    let body_validity: Vec<bool> = bodies.iter().map(Option::is_some).collect();
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(body_types)),
            Arc::new(StringArray::from(bodies)),
        ],
        Some(NullBuffer::from(body_validity)),
    );

    let trace_arr = FixedSizeBinaryArray::try_from_iter(trace_ids.iter().map(|b| b.as_slice())).unwrap();
    let span_arr = FixedSizeBinaryArray::try_from_iter(span_ids.iter().map(|b| b.as_slice())).unwrap();

    let res_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, false))].into(),
        vec![Arc::new(UInt16Array::from(vec![0u16; n]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false)),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(vec![0u16; n])),
            Arc::new(StringArray::from(vec!["test.scope"; n])),
            Arc::new(StringArray::from(vec!["1.0"; n])),
        ],
        None,
    );

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::EVENT_NAME, DataType::Utf8, true),
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
            Arc::new(StringArray::from(sev_texts)),
            Arc::new(StringArray::from(vec!["evt"; n])),
            Arc::new(trace_arr),
            Arc::new(span_arr),
            Arc::new(StringArray::from(vec!["https://otel.io/schemas/1.28.0"; n])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let total_attrs = attr_rows.len() + cbor_attr_rows.len();
    let mut pids = Vec::with_capacity(total_attrs);
    let mut keys = Vec::with_capacity(total_attrs);
    let mut types = Vec::with_capacity(total_attrs);
    let mut strs: Vec<Option<String>> = Vec::with_capacity(total_attrs);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(total_attrs);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(total_attrs);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(total_attrs);
    let mut bytes_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total_attrs);
    let mut ser_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total_attrs);

    for (pid, k, v) in attr_rows {
        pids.push(pid);
        keys.push(k);
        ser_vec.push(None);
        match v {
            ScalarValue::String(s) => {
                types.push(AttributeValueType::Str as u8);
                strs.push(Some(s));
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Int(i) => {
                types.push(AttributeValueType::Int as u8);
                strs.push(None);
                ints.push(Some(i));
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Double(d) => {
                types.push(AttributeValueType::Double as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(Some(d));
                bools.push(None);
                bytes_vec.push(None);
            }
            ScalarValue::Bool(b) => {
                types.push(AttributeValueType::Bool as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(Some(b));
                bytes_vec.push(None);
            }
            ScalarValue::Bytes(b) => {
                types.push(AttributeValueType::Bytes as u8);
                strs.push(None);
                ints.push(None);
                doubles.push(None);
                bools.push(None);
                bytes_vec.push(Some(b));
            }
        }
    }

    for (pid, k, cval) in cbor_attr_rows {
        pids.push(pid);
        keys.push(k);
        let vtype = match &cval {
            ciborium::Value::Array(_) => AttributeValueType::Slice as u8,
            _ => AttributeValueType::Map as u8,
        };
        types.push(vtype);
        strs.push(None);
        ints.push(None);
        doubles.push(None);
        bools.push(None);
        bytes_vec.push(None);
        let mut buf = Vec::new();
        ciborium::into_writer(&cval, &mut buf).unwrap();
        ser_vec.push(Some(buf));
    }

    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ]));
    let attr_rb = RecordBatch::try_new(
        attr_schema,
        vec![
            Arc::new(UInt16Array::from(pids)),
            Arc::new(StringArray::from(keys)),
            Arc::new(UInt8Array::from(types)),
            Arc::new(StringArray::from(strs)),
            Arc::new(Int64Array::from(ints)),
            Arc::new(Float64Array::from(doubles)),
            Arc::new(BooleanArray::from(bools)),
            Arc::new(BinaryArray::from_opt_vec(
                bytes_vec.iter().map(|b| b.as_deref()).collect(),
            )),
            Arc::new(BinaryArray::from_opt_vec(
                ser_vec.iter().map(|b| b.as_deref()).collect(),
            )),
        ],
    )
    .unwrap();

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records.set(ArrowPayloadType::Logs, root_rb).unwrap();
    records.set(ArrowPayloadType::LogAttrs, attr_rb).unwrap();
    records
}

#[test]
fn test_keep_overrides_drop_precedence() {
    // Row 0: DEBUG (sev=5), service="payment" -> matches both Drop(sev < 9) and Keep(service == "payment") -> MUST BE KEPT
    // Row 1: DEBUG (sev=5), service="cart"    -> matches Drop(sev < 9) only -> MUST BE DROPPED
    // Row 2: INFO  (sev=9), service="cart"    -> matches neither -> MUST BE KEPT
    let batch = build_custom_test_batch(
        vec![Some("debug payment"), Some("debug cart"), Some("info cart")],
        vec![5, 5, 9],
        vec!["DEBUG", "DEBUG", "INFO"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![
            (0, "service.name", ScalarValue::String("payment".into())),
            (1, "service.name", ScalarValue::String("cart".into())),
            (2, "service.name", ScalarValue::String("cart".into())),
        ],
        vec![],
    );

    let policies = vec![
        Policy::log_filter(
            "drop_low_severity",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Lt(NumericValue::Int(9)),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_payment_service",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("service.name")),
                predicate: Predicate::Equals(ScalarValue::String("payment".into())),
                negate: false,
            }],
            Action::Keep,
        ),
    ];

    let expected = RoaringBitmap::from_sorted_iter([0, 2]).unwrap();
    let engine = compile_engine(&policies);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        expected,
        "Engine {} failed ACTION_KEEP > ACTION_DROP precedence",
        engine.name()
    );
}

#[test]
fn test_all_8_predicates_and_edge_cases() {
    // Tests:
    // 1. RE2 `$` vs trailing `\n`: `^ERROR$` matches `"ERROR"` (row 0) but MUST NOT match `"ERROR\n"` (row 1).
    // 2. Non-dictionary StringArray row boundary isolation: row 0 `"foo"` and row 1 `"bar"` must NOT match `contains: "oba"`.
    // 3. Exact 64-bit integer precision above 2^53: 9_007_199_254_740_993 (2^53 + 1) > 9_007_199_254_740_992.0 (2^53 as f64).
    // 4. trace_id / span_id lowercase hex and all-zero byte absence (`[0u8; 16]` has exists == false).
    // 5. CBOR nested AttributePath (`["http", "request", "method"]`) and array `contains`.
    let mut trace0 = [0u8; 16];
    trace0[0] = 0xab;
    trace0[15] = 0xcd;
    let trace_zero = [0u8; 16];

    let nested_map = ciborium::Value::Map(vec![(
        ciborium::Value::Text("request".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("method".into()),
            ciborium::Value::Text("POST".into()),
        )]),
    )]);
    let tags_array = ciborium::Value::Array(vec![
        ciborium::Value::Text("canary".into()),
        ciborium::Value::Text("pci-dss".into()),
    ]);

    let batch = build_custom_test_batch(
        vec![Some("foo"), Some("bar"), Some("ERROR\n"), Some("ERROR")],
        vec![9, 9, 17, 17],
        vec!["INFO", "INFO", "ERROR", "ERROR"],
        vec![trace0, trace_zero, trace_zero, trace0],
        vec![[0u8; 8]; 4],
        vec![
            (0, "big_int", ScalarValue::Int(9_007_199_254_740_993)), // 2^53 + 1
            (1, "big_int", ScalarValue::Int(9_007_199_254_740_992)), // 2^53
            (2, "raw_bytes", ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
        ],
        vec![(0, "http", nested_map), (3, "tags", tags_array)],
    );

    // 1. `^ERROR$` must drop row 3 ("ERROR") and keep row 2 ("ERROR\n")
    let p_re_dollar = vec![Policy::log_filter(
        "drop_exact_error_line",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("^ERROR$".into()),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_re_dollar);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        RoaringBitmap::from_sorted_iter([0, 1, 2]).unwrap(),
        "Engine {} failed RE2 $ vs trailing newline",
        engine.name()
    );

    // 2. Row boundary isolation: `contains("oba")` on ["foo", "bar", ...] must match 0 rows (keep all 4 rows)
    let p_boundary = vec![Policy::log_filter(
        "drop_cross_row_oba",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Contains(ScalarValue::String("oba".into())),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_boundary);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        RoaringBitmap::from_sorted_iter([0, 1, 2, 3]).unwrap(),
        "Engine {} falsely matched across adjacent StringArray rows",
        engine.name()
    );

    // 3. Cross-type numeric precision above 2^53: Gt(Double(9_007_199_254_740_992.0)) matches only row 0 (2^53 + 1), NOT row 1 (2^53)
    let p_precision = vec![Policy::log_filter(
        "drop_above_2_53",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("big_int")),
            predicate: Predicate::Gt(NumericValue::Double(9_007_199_254_740_992.0)),
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_precision);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        RoaringBitmap::from_sorted_iter([1, 2, 3]).unwrap(),
        "Engine {} failed 2^53 cross-type i64/f64 precision test",
        engine.name()
    );

    // 4. trace_id exists & hex match: rows 0 and 3 have non-zero trace0, rows 1 and 2 have all-zero trace_id
    let p_trace_exists = vec![Policy::log_filter(
        "drop_when_trace_id_exists",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::TraceId),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )];
    let engine = compile_engine(&p_trace_exists);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        RoaringBitmap::from_sorted_iter([1, 2]).unwrap(),
        "Engine {} failed all-zero trace_id absence semantics",
        engine.name()
    );

    // 5. CBOR nested map path ["http", "request", "method"] == "POST" (drops row 0) and array contains "pci-dss" (drops row 3)
    let p_cbor = vec![
        Policy::log_filter(
            "drop_nested_post",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested([
                    "http", "request", "method",
                ])),
                predicate: Predicate::Equals(ScalarValue::String("POST".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_pci_tag",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("tags")),
                predicate: Predicate::Contains(ScalarValue::String("pci-dss".into())),
                negate: false,
            }],
            Action::Drop,
        ),
    ];
    let engine = compile_engine(&p_cbor);
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(
        kept,
        RoaringBitmap::from_sorted_iter([1, 2]).unwrap(),
        "Engine {} failed CBOR nested path or array contains",
        engine.name()
    );
}

#[test]
fn test_global_4stage_transform_order_and_redact_captures() {
    // Verifies:
    // 1. Global stage order is `1. Rename -> 2. Add -> 3. Remove -> 4. Redact` across policies sorted by ascending `Policy.id`.
    //    Suppose Policy "b_policy" has Rename(a -> b, upsert=false) and Remove(temp),
    //    and Policy "a_policy" has Add(b = "added_b", upsert=false) and Redact(body, regex="user=([a-z]+)", replacement="user=[MASKED:$1]").
    //    Because ALL Stage 1 (Rename) operations run BEFORE Stage 2 (Add):
    //    - In Stage 1, `b` does not exist yet, so `b_policy`'s Rename(`a` -> `b`, upsert=false) succeeds (`b` becomes `"orig_a"`).
    //    - In Stage 2, `b` already exists (`"orig_a"`), so `a_policy`'s Add(`b` = `"added_b"`, upsert=false) does NOT overwrite `b`!
    //    (Under legacy per-policy `Remove -> Redact -> Rename -> Add` or `a_policy` before `b_policy`, `a_policy` would set `b = "added_b"` first and `b_policy`'s Rename(upsert=false) would be blocked!)
    let batch = build_custom_test_batch(
        vec![Some("login user=alice ok")],
        vec![9],
        vec!["INFO"],
        vec![[1u8; 16]],
        vec![[1u8; 8]],
        vec![
            (0, "a", ScalarValue::String("orig_a".into())),
            (0, "temp", ScalarValue::String("to_delete".into())),
        ],
        vec![],
    );

    let policies = vec![
        Policy::log_transform(
            "b_policy",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("a")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("b")),
                    upsert: false,
                }],
                add: vec![],
                remove: vec![LogRemove {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("temp")),
                }],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "a_policy",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("b")),
                    value: ScalarValue::String("added_b".into()),
                    upsert: false,
                }],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some("user=[MASKED:$1]".into()),
                    regex: Some("user=([a-z]+)".into()),
                }],
            },
        ),
    ];

    let engine = compile_engine(&policies);
    let out = engine.evaluate(batch.clone()).unwrap();
    let out_records = out.records.expect("row kept");
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();

    let body = view.extract_field_at_row(
        &LogFieldSelector::RecordField(LogRecordField::Body),
        0,
    );
    assert_eq!(
        body,
        Some(ExtractedValue::Str("login user=[MASKED:alice] ok".into())),
        "Engine {} failed LogRedact capture template expansion",
        engine.name()
    );

    let attr_a = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("a")),
        0,
    );
    assert_eq!(attr_a, None, "Engine {} should have renamed 'a'", engine.name());

    let attr_b = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("b")),
        0,
    );
    assert_eq!(
        attr_b,
        Some(ExtractedValue::Str("orig_a".into())),
        "Engine {} violated global Stage 1 (Rename) before Stage 2 (Add) ordering or upsert=false",
        engine.name()
    );

    let attr_temp = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("temp")),
        0,
    );
    assert_eq!(
        attr_temp,
        None,
        "Engine {} failed Stage 3 Remove",
        engine.name()
    );
}
