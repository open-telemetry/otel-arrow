// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and conformance tests for `PolicyEngine` (`open-telemetry/otel-arrow#4726`).

#![allow(clippy::unwrap_used)]

use std::borrow::Cow;
use std::cmp::Ordering;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array, UInt32Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::filter::IdBitmapPool;
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};
use regex::Regex;
use regex_syntax::ParserBuilder;
use regex_syntax::hir::Hir;
use roaring::RoaringBitmap;

use crate::model::{
    Action, AttributePath, LogAdd, LogFieldSelector, LogFilterPolicy, LogMatcher, LogRecordField,
    LogRedact, LogRemove, LogRename, LogTargetSelector, LogTransformPolicy, MetricFilterPolicy,
    NumericValue, Policy, PolicyDefinition, PolicyError, Predicate, ScalarValue, ScopeField,
    TraceSamplingPolicy, apply_regex_redaction, cmp_i64_with_f64, eq_i64_f64,
};
use crate::otap_ops::{
    CompiledTransformPolicy, ExtractedValue, LogsBatchView, apply_spec_transforms_masked,
    decode_cbor_value, extract_anyval_from_attr_batch, extract_anyval_from_struct,
    extract_bytes_from_array, extract_fixed_bytes_from_array, extract_i64_from_array,
    extract_parent_id_u16, extract_str_from_array, filter_otap_logs_by_bitmap, format_non_zero_hex,
    navigate_extracted_path,
};
use crate::{PolicyEngine, PolicyEvaluationEngine, extract_required_ngrams};

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

    let trace_arr =
        FixedSizeBinaryArray::try_from_iter(trace_ids.iter().map(|b| b.as_slice())).unwrap();
    let span_arr =
        FixedSizeBinaryArray::try_from_iter(span_ids.iter().map(|b| b.as_slice())).unwrap();

    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(vec![0u16; n]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
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
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
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
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
            (
                2,
                "raw_bytes",
                ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef]),
            ),
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

    let body = view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0);
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
    assert_eq!(
        attr_a,
        None,
        "Engine {} should have renamed 'a'",
        engine.name()
    );

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

fn encode_cbor(val: &ciborium::Value) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(val, &mut buf).unwrap();
    buf
}

fn build_attr_batch(
    attr_rows: Vec<(u16, &str, ScalarValue)>,
    cbor_attr_rows: Vec<(u16, &str, ciborium::Value)>,
) -> RecordBatch {
    let total = attr_rows.len() + cbor_attr_rows.len();
    let mut pids = Vec::with_capacity(total);
    let mut keys = Vec::with_capacity(total);
    let mut types = Vec::with_capacity(total);
    let mut strs: Vec<Option<String>> = Vec::with_capacity(total);
    let mut ints: Vec<Option<i64>> = Vec::with_capacity(total);
    let mut doubles: Vec<Option<f64>> = Vec::with_capacity(total);
    let mut bools: Vec<Option<bool>> = Vec::with_capacity(total);
    let mut bytes_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);
    let mut ser_vec: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);

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
        ser_vec.push(Some(encode_cbor(&cval)));
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ]));
    RecordBatch::try_new(
        schema,
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
    .unwrap()
}

fn parse_hir(pattern: &str) -> Hir {
    ParserBuilder::new()
        .multi_line(false)
        .build()
        .parse(pattern)
        .unwrap()
}

// ============================================================================
// model.rs unit tests
// ============================================================================

#[test]
fn test_cmp_i64_with_f64_and_numeric_value() {
    // Special floating-point values
    assert_eq!(cmp_i64_with_f64(0, f64::NAN), None);
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, f64::INFINITY),
        Some(Ordering::Less)
    );
    assert_eq!(
        cmp_i64_with_f64(i64::MIN, f64::NEG_INFINITY),
        Some(Ordering::Greater)
    );

    // Bounds around 2^63 and -2^63
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, 9_223_372_036_854_775_808.0),
        Some(Ordering::Less)
    );
    assert_eq!(cmp_i64_with_f64(42, 1e30), Some(Ordering::Less));
    assert_eq!(
        cmp_i64_with_f64(i64::MIN, -9_223_372_036_854_775_808.0),
        Some(Ordering::Equal)
    );
    assert_eq!(cmp_i64_with_f64(i64::MIN, -1e30), Some(Ordering::Greater));
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, 9_223_372_036_854_774_784.0),
        Some(Ordering::Greater)
    );

    // Integral comparisons
    assert_eq!(cmp_i64_with_f64(10, 20.0), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(20, 10.0), Some(Ordering::Greater));
    assert_eq!(cmp_i64_with_f64(10, 10.0), Some(Ordering::Equal));
    assert_eq!(cmp_i64_with_f64(0, -0.0), Some(Ordering::Equal));

    // Fractional positive and negative remainders when truncated parts match
    assert_eq!(cmp_i64_with_f64(5, 5.25), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(0, 0.5), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(-5, -5.25), Some(Ordering::Greater));
    assert_eq!(cmp_i64_with_f64(0, -0.5), Some(Ordering::Greater));

    // Exact 64-bit precision above 2^53
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_991, 9_007_199_254_740_992.0),
        Some(Ordering::Less)
    );
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_992, 9_007_199_254_740_992.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_993, 9_007_199_254_740_992.0),
        Some(Ordering::Greater)
    );

    // eq_i64_f64 helper
    assert!(eq_i64_f64(42, 42.0));
    assert!(eq_i64_f64(i64::MIN, -9_223_372_036_854_775_808.0));
    assert!(!eq_i64_f64(42, 42.5));
    assert!(!eq_i64_f64(42, f64::NAN));
    assert!(!eq_i64_f64(9_007_199_254_740_993, 9_007_199_254_740_992.0));

    // NumericValue::cmp_candidate_i64
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(5),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(10),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(15),
        Some(Ordering::Greater)
    );
    assert_eq!(NumericValue::Double(f64::NAN).cmp_candidate_i64(10), None);
    assert_eq!(
        NumericValue::Double(10.5).cmp_candidate_i64(10),
        Some(Ordering::Less)
    );

    // NumericValue::cmp_candidate_f64 (NaN, Int threshold, Double threshold)
    assert_eq!(NumericValue::Int(10).cmp_candidate_f64(f64::NAN), None);
    assert_eq!(NumericValue::Double(10.0).cmp_candidate_f64(f64::NAN), None);
    assert_eq!(NumericValue::Double(f64::NAN).cmp_candidate_f64(10.0), None);
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(9.5),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(10.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(10.5),
        Some(Ordering::Greater)
    );
    assert_eq!(
        NumericValue::Int(i64::MAX).cmp_candidate_f64(f64::INFINITY),
        Some(Ordering::Greater)
    );
    assert_eq!(
        NumericValue::Int(i64::MIN).cmp_candidate_f64(f64::NEG_INFINITY),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(9_007_199_254_740_993).cmp_candidate_f64(9_007_199_254_740_992.0),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(9.25),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(10.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(10.25),
        Some(Ordering::Greater)
    );
}

#[test]
fn test_predicate_matches_ordering() {
    let gt = Predicate::Gt(NumericValue::Int(0));
    let gte = Predicate::Gte(NumericValue::Int(0));
    let lt = Predicate::Lt(NumericValue::Int(0));
    let lte = Predicate::Lte(NumericValue::Int(0));

    assert!(!gt.matches_ordering(None));
    assert!(!gte.matches_ordering(None));
    assert!(!lt.matches_ordering(None));
    assert!(!lte.matches_ordering(None));

    assert!(!gt.matches_ordering(Some(Ordering::Less)));
    assert!(!gt.matches_ordering(Some(Ordering::Equal)));
    assert!(gt.matches_ordering(Some(Ordering::Greater)));

    assert!(!gte.matches_ordering(Some(Ordering::Less)));
    assert!(gte.matches_ordering(Some(Ordering::Equal)));
    assert!(gte.matches_ordering(Some(Ordering::Greater)));

    assert!(lt.matches_ordering(Some(Ordering::Less)));
    assert!(!lt.matches_ordering(Some(Ordering::Equal)));
    assert!(!lt.matches_ordering(Some(Ordering::Greater)));

    assert!(lte.matches_ordering(Some(Ordering::Less)));
    assert!(lte.matches_ordering(Some(Ordering::Equal)));
    assert!(!lte.matches_ordering(Some(Ordering::Greater)));

    for non_range in [
        Predicate::Exists,
        Predicate::Equals(ScalarValue::Int(0)),
        Predicate::Regex("a".into()),
        Predicate::Contains(ScalarValue::String("a".into())),
    ] {
        assert!(!non_range.matches_ordering(Some(Ordering::Equal)));
        assert!(!non_range.matches_ordering(Some(Ordering::Greater)));
        assert!(!non_range.matches_ordering(Some(Ordering::Less)));
    }
}

#[test]
fn test_regex_redaction_and_capture_templates() {
    // No match returns None
    let re_user = Regex::new(r"user=(?P<name>[a-z]+)").unwrap();
    assert_eq!(apply_regex_redaction(&re_user, "no match here", "X"), None);

    // Literal replacement across multiple matches
    assert_eq!(
        apply_regex_redaction(&re_user, "a user=alice and user=bob!", "REDACTED"),
        Some("a REDACTED and REDACTED!".into())
    );

    // Escaped $$, $0, $1, ${1}, ${name}, missing captures, trailing $, unclosed ${, unknown $_
    assert_eq!(
        apply_regex_redaction(
            &re_user,
            "user=alice",
            "$$0=$0,${1},${name},${99},${missing}"
        ),
        Some("$0=user=alice,alice,alice,,".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user, "user=alice", "tail$"),
        Some("tail$".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user, "user=alice", "open${unclosed"),
        Some("open${unclosed".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user, "user=alice", "$_keep$x"),
        Some("$_keep$x".into())
    );

    // Two-digit capture indexes ($10, $12) and missing capture ($20, unmatched optional)
    let re_twelve = Regex::new(r"(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)(m)?").unwrap();
    assert_eq!(
        apply_regex_redaction(&re_twelve, "abcdefghijkl", "$10-$12-$13-$20-${11}"),
        Some("j-l---k".into())
    );

    // LogRedact::effective_replacement default vs explicit
    let default_redact = LogRedact {
        target: LogTargetSelector::RecordField(LogRecordField::Body),
        replacement: None,
        regex: None,
    };
    assert_eq!(default_redact.effective_replacement(), "[REDACTED]");
    let custom_redact = LogRedact {
        target: LogTargetSelector::RecordField(LogRecordField::Body),
        replacement: Some("MASK".into()),
        regex: Some("secret".into()),
    };
    assert_eq!(custom_redact.effective_replacement(), "MASK");

    // PolicyError Display formatting
    let compile_err = PolicyError::Compile("bad".into());
    assert!(compile_err.to_string().contains("bad"));
    let eval_err = PolicyError::Evaluation("oops".into());
    assert!(eval_err.to_string().contains("oops"));
}

// ============================================================================
// otap_ops.rs unit tests
// ============================================================================

#[test]
fn test_extracted_value_into_owned_equals_and_contains() {
    // into_owned across all 7 variants
    let s_ref = "hello";
    let b_ref: &[u8] = &[1, 2, 3];
    assert_eq!(
        ExtractedValue::Str(Cow::Borrowed(s_ref)).into_owned(),
        ExtractedValue::Str(Cow::Owned("hello".into()))
    );
    assert_eq!(ExtractedValue::Int(7).into_owned(), ExtractedValue::Int(7));
    assert_eq!(
        ExtractedValue::Double(2.5).into_owned(),
        ExtractedValue::Double(2.5)
    );
    assert_eq!(
        ExtractedValue::Bool(true).into_owned(),
        ExtractedValue::Bool(true)
    );
    assert_eq!(
        ExtractedValue::Bytes(Cow::Borrowed(b_ref)).into_owned(),
        ExtractedValue::Bytes(Cow::Owned(vec![1, 2, 3]))
    );
    let arr = ExtractedValue::Array(vec![ExtractedValue::Int(1)]);
    assert_eq!(arr.clone().into_owned(), arr);
    let map = ExtractedValue::Map(vec![("k".into(), ExtractedValue::Bool(false))]);
    assert_eq!(map.clone().into_owned(), map);

    // equals_scalar across all arms
    let ev_abc = ExtractedValue::Str(Cow::Borrowed("abc"));
    assert!(ev_abc.equals_scalar(&ScalarValue::String("abc".into())));
    assert!(!ev_abc.equals_scalar(&ScalarValue::String("xyz".into())));
    assert!(ExtractedValue::Int(42).equals_scalar(&ScalarValue::Int(42)));
    assert!(!ExtractedValue::Int(42).equals_scalar(&ScalarValue::Int(43)));
    assert!(ExtractedValue::Double(1.5).equals_scalar(&ScalarValue::Double(1.5)));
    assert!(!ExtractedValue::Double(1.5).equals_scalar(&ScalarValue::Double(2.5)));
    assert!(!ExtractedValue::Double(f64::NAN).equals_scalar(&ScalarValue::Double(f64::NAN)));
    assert!(!ExtractedValue::Double(f64::NAN).equals_scalar(&ScalarValue::Double(1.0)));
    assert!(!ExtractedValue::Double(1.0).equals_scalar(&ScalarValue::Double(f64::NAN)));
    assert!(ExtractedValue::Int(42).equals_scalar(&ScalarValue::Double(42.0)));
    assert!(!ExtractedValue::Int(42).equals_scalar(&ScalarValue::Double(42.5)));
    assert!(
        !ExtractedValue::Int(9_007_199_254_740_993)
            .equals_scalar(&ScalarValue::Double(9_007_199_254_740_992.0))
    );
    assert!(ExtractedValue::Double(42.0).equals_scalar(&ScalarValue::Int(42)));
    assert!(!ExtractedValue::Double(42.5).equals_scalar(&ScalarValue::Int(42)));
    assert!(!ExtractedValue::Double(f64::NAN).equals_scalar(&ScalarValue::Int(42)));
    assert!(ExtractedValue::Bool(true).equals_scalar(&ScalarValue::Bool(true)));
    assert!(!ExtractedValue::Bool(true).equals_scalar(&ScalarValue::Bool(false)));
    let ev_bytes = ExtractedValue::Bytes(Cow::Borrowed(&[1, 2]));
    assert!(ev_bytes.equals_scalar(&ScalarValue::Bytes(vec![1, 2])));
    assert!(!ev_bytes.equals_scalar(&ScalarValue::Bytes(vec![1, 3])));
    // Cross-type mismatches
    assert!(!ExtractedValue::Str(Cow::Borrowed("42")).equals_scalar(&ScalarValue::Int(42)));
    assert!(!ExtractedValue::Bool(true).equals_scalar(&ScalarValue::Int(1)));
    assert!(!arr.equals_scalar(&ScalarValue::Int(1)));
    assert!(!map.equals_scalar(&ScalarValue::String("k".into())));

    // contains_scalar across all arms
    let ev_hello = ExtractedValue::Str(Cow::Borrowed("hello world"));
    assert!(ev_hello.contains_scalar(&ScalarValue::String("world".into())));
    assert!(!ev_hello.contains_scalar(&ScalarValue::String("mars".into())));
    assert!(ev_hello.contains_scalar(&ScalarValue::String(String::new())));
    let ev_deadbeef = ExtractedValue::Bytes(Cow::Borrowed(&[0xde, 0xad, 0xbe, 0xef]));
    assert!(ev_deadbeef.contains_scalar(&ScalarValue::Bytes(vec![])));
    assert!(ev_deadbeef.contains_scalar(&ScalarValue::Bytes(vec![0xad, 0xbe])));
    assert!(!ev_deadbeef.contains_scalar(&ScalarValue::Bytes(vec![0xaa, 0xbb])));

    let mixed_arr = ExtractedValue::Array(vec![
        ExtractedValue::Str(Cow::Owned("tag".into())),
        ExtractedValue::Int(10),
        ExtractedValue::Double(2.5),
        ExtractedValue::Bool(true),
        ExtractedValue::Bytes(Cow::Owned(vec![0x01, 0x02])),
    ]);
    assert!(mixed_arr.contains_scalar(&ScalarValue::String("tag".into())));
    assert!(mixed_arr.contains_scalar(&ScalarValue::Int(10)));
    assert!(mixed_arr.contains_scalar(&ScalarValue::Double(10.0)));
    assert!(mixed_arr.contains_scalar(&ScalarValue::Double(2.5)));
    assert!(mixed_arr.contains_scalar(&ScalarValue::Bool(true)));
    assert!(mixed_arr.contains_scalar(&ScalarValue::Bytes(vec![0x01, 0x02])));
    assert!(!mixed_arr.contains_scalar(&ScalarValue::String("missing".into())));

    // Mismatched contains_scalar
    assert!(!ExtractedValue::Int(10).contains_scalar(&ScalarValue::Int(10)));
    let ev_ab = ExtractedValue::Str(Cow::Borrowed("ab"));
    assert!(!ev_ab.contains_scalar(&ScalarValue::Bytes(vec![b'a'])));
    let ev_ab_bytes = ExtractedValue::Bytes(Cow::Borrowed(b"ab"));
    assert!(!ev_ab_bytes.contains_scalar(&ScalarValue::String("a".into())));
    assert!(!map.contains_scalar(&ScalarValue::String("k".into())));

    // format_non_zero_hex
    assert_eq!(format_non_zero_hex(&[0u8; 8]), None);
    assert_eq!(format_non_zero_hex(&[]), None);
    assert_eq!(format_non_zero_hex(&[0x0a, 0xbc]), Some("0abc".into()));
}

#[test]
fn test_decode_cbor_and_navigate_path() {
    // Text, Integer, Float, Bool, Bytes
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Text("hi".into()))),
        Some(ExtractedValue::Str(Cow::Owned("hi".into())))
    );
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Integer((-42).into()))),
        Some(ExtractedValue::Int(-42))
    );
    // Unsigned integer exceeding i64::MAX (CBOR major 0, 8-byte u64::MAX) returns None
    let cbor_u64_max = [0x1bu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    assert_eq!(decode_cbor_value(&cbor_u64_max), None);
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Float(1.25))),
        Some(ExtractedValue::Double(1.25))
    );
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Bool(true))),
        Some(ExtractedValue::Bool(true))
    );
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Bytes(vec![0xca, 0xfe]))),
        Some(ExtractedValue::Bytes(Cow::Owned(vec![0xca, 0xfe])))
    );

    // Array with supported items plus Null (skipped)
    let arr_val = ciborium::Value::Array(vec![
        ciborium::Value::Integer(1.into()),
        ciborium::Value::Null,
        ciborium::Value::Text("ok".into()),
    ]);
    assert_eq!(
        decode_cbor_value(&encode_cbor(&arr_val)),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Int(1),
            ExtractedValue::Str(Cow::Owned("ok".into())),
        ]))
    );

    // Map with text key, non-text key (skipped), and Null value (skipped)
    let map_val = ciborium::Value::Map(vec![
        (
            ciborium::Value::Text("nested".into()),
            ciborium::Value::Map(vec![(
                ciborium::Value::Text("leaf".into()),
                ciborium::Value::Integer(99.into()),
            )]),
        ),
        (
            ciborium::Value::Integer(7.into()),
            ciborium::Value::Text("ignored_key".into()),
        ),
        (
            ciborium::Value::Text("null_val".into()),
            ciborium::Value::Null,
        ),
    ]);
    let decoded_map = decode_cbor_value(&encode_cbor(&map_val)).unwrap();

    // Unsupported top-level CBOR Null, Tag, and invalid CBOR bytes
    assert_eq!(
        decode_cbor_value(&encode_cbor(&ciborium::Value::Null)),
        None
    );
    let tagged = ciborium::Value::Tag(1, Box::new(ciborium::Value::Integer(1.into())));
    assert_eq!(decode_cbor_value(&encode_cbor(&tagged)), None);
    assert_eq!(decode_cbor_value(&[0xff, 0xff]), None);

    // navigate_extracted_path: empty, hit, miss, non-map leaf
    assert_eq!(
        navigate_extracted_path(decoded_map.clone(), &[]),
        Some(decoded_map.clone())
    );
    assert_eq!(
        navigate_extracted_path(decoded_map.clone(), &["nested".into(), "leaf".into()]),
        Some(ExtractedValue::Int(99))
    );
    assert_eq!(
        navigate_extracted_path(decoded_map.clone(), &["nested".into(), "missing".into()]),
        None
    );
    assert_eq!(
        navigate_extracted_path(
            decoded_map,
            &["nested".into(), "leaf".into(), "deeper".into()]
        ),
        None
    );
    assert_eq!(
        navigate_extracted_path(ExtractedValue::Str(Cow::Borrowed("x")), &["k".into()]),
        None
    );
}

#[test]
fn test_array_extractors_and_dictionary_encoding() {
    // extract_str_from_array: StringArray, Dict<UInt8>, Dict<UInt16>, null, unsupported
    let str_col: ArrayRef = Arc::new(StringArray::from(vec![Some("alpha"), None]));
    assert_eq!(extract_str_from_array(&str_col, 0), Some("alpha"));
    assert_eq!(extract_str_from_array(&str_col, 1), None);

    let str_dict_vals: ArrayRef = Arc::new(StringArray::from(vec![Some("d0"), None]));
    let dict8_str: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1)]),
            str_dict_vals.clone(),
        )
        .unwrap(),
    );
    assert_eq!(extract_str_from_array(&dict8_str, 0), Some("d0"));
    assert_eq!(extract_str_from_array(&dict8_str, 1), None);
    assert_eq!(extract_str_from_array(&dict8_str, 2), None);

    let dict16_str: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), None, Some(1)]),
            str_dict_vals,
        )
        .unwrap(),
    );
    assert_eq!(extract_str_from_array(&dict16_str, 0), Some("d0"));
    assert_eq!(extract_str_from_array(&dict16_str, 1), None);
    assert_eq!(extract_str_from_array(&dict16_str, 2), None);

    let int_col: ArrayRef = Arc::new(Int64Array::from(vec![Some(10), None]));
    assert_eq!(extract_str_from_array(&int_col, 0), None);
    let dict8_bad_str: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(UInt8Array::from(vec![0]), int_col.clone()).unwrap(),
    );
    assert_eq!(extract_str_from_array(&dict8_bad_str, 0), None);

    // extract_i64_from_array: Int64, Int32, Dict<UInt8, Int64|Int32>, Dict<UInt16, Int64|Int32>
    assert_eq!(extract_i64_from_array(&int_col, 0), Some(10));
    assert_eq!(extract_i64_from_array(&int_col, 1), None);
    let i32_col: ArrayRef = Arc::new(Int32Array::from(vec![Some(-5), None]));
    assert_eq!(extract_i64_from_array(&i32_col, 0), Some(-5));
    assert_eq!(extract_i64_from_array(&i32_col, 1), None);

    let dict8_i64: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1)]),
            int_col.clone(),
        )
        .unwrap(),
    );
    assert_eq!(extract_i64_from_array(&dict8_i64, 0), Some(10));
    assert_eq!(extract_i64_from_array(&dict8_i64, 1), None);
    assert_eq!(extract_i64_from_array(&dict8_i64, 2), None);

    let dict8_i32: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), Some(1)]),
            i32_col.clone(),
        )
        .unwrap(),
    );
    assert_eq!(extract_i64_from_array(&dict8_i32, 0), Some(-5));
    assert_eq!(extract_i64_from_array(&dict8_i32, 1), None);

    let dict16_i64: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), None, Some(1)]),
            int_col,
        )
        .unwrap(),
    );
    assert_eq!(extract_i64_from_array(&dict16_i64, 0), Some(10));
    assert_eq!(extract_i64_from_array(&dict16_i64, 1), None);
    assert_eq!(extract_i64_from_array(&dict16_i64, 2), None);

    let dict16_i32: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(UInt16Array::from(vec![Some(0), Some(1)]), i32_col)
            .unwrap(),
    );
    assert_eq!(extract_i64_from_array(&dict16_i32, 0), Some(-5));
    assert_eq!(extract_i64_from_array(&dict16_i32, 1), None);
    assert_eq!(extract_i64_from_array(&str_col, 0), None);
    assert_eq!(extract_i64_from_array(&dict8_str, 0), None);
    assert_eq!(extract_i64_from_array(&dict16_str, 0), None);

    // extract_bytes_from_array: BinaryArray, Dict<UInt8>, Dict<UInt16>, unsupported
    let bin_col: ArrayRef = Arc::new(BinaryArray::from_opt_vec(vec![Some(b"xyz"), None]));
    assert_eq!(extract_bytes_from_array(&bin_col, 0), Some(&b"xyz"[..]));
    assert_eq!(extract_bytes_from_array(&bin_col, 1), None);

    let dict8_bin: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1)]),
            bin_col.clone(),
        )
        .unwrap(),
    );
    assert_eq!(extract_bytes_from_array(&dict8_bin, 0), Some(&b"xyz"[..]));
    assert_eq!(extract_bytes_from_array(&dict8_bin, 1), None);
    assert_eq!(extract_bytes_from_array(&dict8_bin, 2), None);

    let dict16_bin: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), None, Some(1)]),
            bin_col.clone(),
        )
        .unwrap(),
    );
    assert_eq!(extract_bytes_from_array(&dict16_bin, 0), Some(&b"xyz"[..]));
    assert_eq!(extract_bytes_from_array(&dict16_bin, 1), None);
    assert_eq!(extract_bytes_from_array(&dict16_bin, 2), None);
    assert_eq!(extract_bytes_from_array(&str_col, 0), None);

    // extract_fixed_bytes_from_array: FixedSizeBinaryArray, Dict<UInt8>, Dict<UInt16>, unsupported
    let fsb_vals = FixedSizeBinaryArray::try_from_sparse_iter_with_size(
        vec![Some([0x11u8, 0x22, 0x33, 0x44]), None].into_iter(),
        4,
    )
    .unwrap();
    let fsb_col: ArrayRef = Arc::new(fsb_vals);
    assert_eq!(
        extract_fixed_bytes_from_array(&fsb_col, 0),
        Some(&[0x11u8, 0x22, 0x33, 0x44][..])
    );
    assert_eq!(extract_fixed_bytes_from_array(&fsb_col, 1), None);

    let dict8_fsb: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1)]),
            fsb_col.clone(),
        )
        .unwrap(),
    );
    assert_eq!(
        extract_fixed_bytes_from_array(&dict8_fsb, 0),
        Some(&[0x11u8, 0x22, 0x33, 0x44][..])
    );
    assert_eq!(extract_fixed_bytes_from_array(&dict8_fsb, 1), None);
    assert_eq!(extract_fixed_bytes_from_array(&dict8_fsb, 2), None);

    let dict16_fsb: ArrayRef = Arc::new(
        DictionaryArray::<UInt16Type>::try_new(
            UInt16Array::from(vec![Some(0), None, Some(1)]),
            fsb_col,
        )
        .unwrap(),
    );
    assert_eq!(
        extract_fixed_bytes_from_array(&dict16_fsb, 0),
        Some(&[0x11u8, 0x22, 0x33, 0x44][..])
    );
    assert_eq!(extract_fixed_bytes_from_array(&dict16_fsb, 1), None);
    assert_eq!(extract_fixed_bytes_from_array(&dict16_fsb, 2), None);
    assert_eq!(extract_fixed_bytes_from_array(&bin_col, 0), None);

    // extract_parent_id_u16: missing, UInt16, UInt32 (fit & overflow), Dict<UInt8, UInt16|UInt32>
    let make_pid_batch = |col: ArrayRef| {
        let schema = Arc::new(Schema::new(vec![Field::new(
            consts::PARENT_ID,
            col.data_type().clone(),
            true,
        )]));
        RecordBatch::try_new(schema, vec![col]).unwrap()
    };

    let empty_schema_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "other",
            DataType::UInt8,
            false,
        )])),
        vec![Arc::new(UInt8Array::from(vec![1]))],
    )
    .unwrap();
    assert_eq!(extract_parent_id_u16(&empty_schema_rb, 0), None);

    let rb_u16 = make_pid_batch(Arc::new(UInt16Array::from(vec![Some(7), None])));
    assert_eq!(extract_parent_id_u16(&rb_u16, 0), Some(7));
    assert_eq!(extract_parent_id_u16(&rb_u16, 1), None);

    let rb_u32 = make_pid_batch(Arc::new(UInt32Array::from(vec![
        Some(42),
        Some(70_000),
        None,
    ])));
    assert_eq!(extract_parent_id_u16(&rb_u32, 0), Some(42));
    assert_eq!(extract_parent_id_u16(&rb_u32, 1), None);
    assert_eq!(extract_parent_id_u16(&rb_u32, 2), None);

    let rb_dict_u16 = make_pid_batch(Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1)]),
            Arc::new(UInt16Array::from(vec![Some(9), None])),
        )
        .unwrap(),
    ));
    assert_eq!(extract_parent_id_u16(&rb_dict_u16, 0), Some(9));
    assert_eq!(extract_parent_id_u16(&rb_dict_u16, 1), None);
    assert_eq!(extract_parent_id_u16(&rb_dict_u16, 2), None);

    let rb_dict_u32 = make_pid_batch(Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), Some(1), Some(2)]),
            Arc::new(UInt32Array::from(vec![Some(11), Some(70_000), None])),
        )
        .unwrap(),
    ));
    assert_eq!(extract_parent_id_u16(&rb_dict_u32, 0), Some(11));
    assert_eq!(extract_parent_id_u16(&rb_dict_u32, 1), None);
    assert_eq!(extract_parent_id_u16(&rb_dict_u32, 2), None);

    let rb_unsupported = make_pid_batch(Arc::new(StringArray::from(vec!["bad"])));
    assert_eq!(extract_parent_id_u16(&rb_unsupported, 0), None);
    let rb_dict_unsupported = make_pid_batch(Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![0]),
            Arc::new(StringArray::from(vec!["bad"])),
        )
        .unwrap(),
    ));
    assert_eq!(extract_parent_id_u16(&rb_dict_unsupported, 0), None);
}

#[test]
fn test_extract_anyval_struct_and_attr_batch() {
    let map_bytes = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("k".into()),
        ciborium::Value::Text("v".into()),
    )]));
    let slice_bytes = encode_cbor(&ciborium::Value::Array(vec![ciborium::Value::Integer(
        5.into(),
    )]));

    // Rows:
    // 0: struct-null (for struct array; Str for attr batch)
    // 1: type_arr null
    // 2: invalid type discriminant (255)
    // 3: Empty
    // 4: Str valid
    // 5: Int valid
    // 6: Double valid
    // 7: Double null
    // 8: Bool valid
    // 9: Bool null
    // 10: Bytes valid
    // 11: Map valid
    // 12: Slice valid
    let types = UInt8Array::from(vec![
        Some(AttributeValueType::Str as u8),
        None,
        Some(255),
        Some(AttributeValueType::Empty as u8),
        Some(AttributeValueType::Str as u8),
        Some(AttributeValueType::Int as u8),
        Some(AttributeValueType::Double as u8),
        Some(AttributeValueType::Double as u8),
        Some(AttributeValueType::Bool as u8),
        Some(AttributeValueType::Bool as u8),
        Some(AttributeValueType::Bytes as u8),
        Some(AttributeValueType::Map as u8),
        Some(AttributeValueType::Slice as u8),
    ]);
    let strs = StringArray::from(vec![
        Some("masked"),
        None,
        None,
        None,
        Some("hello"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]);
    let ints = Int64Array::from(vec![
        None,
        None,
        None,
        None,
        None,
        Some(123),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]);
    let doubles = Float64Array::from(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        Some(4.5),
        None,
        None,
        None,
        None,
        None,
        None,
    ]);
    let bools = BooleanArray::from(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
    ]);
    let bytes = BinaryArray::from_opt_vec(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xaa, 0xbb]),
        None,
        None,
    ]);
    let ser = BinaryArray::from_opt_vec(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(map_bytes.as_slice()),
        Some(slice_bytes.as_slice()),
    ]);

    let fields = vec![
        Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
        Arc::new(Field::new(
            consts::ATTRIBUTE_DOUBLE,
            DataType::Float64,
            true,
        )),
        Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
        Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
    ];
    let cols: Vec<ArrayRef> = vec![
        Arc::new(types),
        Arc::new(strs),
        Arc::new(ints),
        Arc::new(doubles),
        Arc::new(bools),
        Arc::new(bytes),
        Arc::new(ser),
    ];

    let mut validity = vec![true; 13];
    validity[0] = false;
    let sa = StructArray::new(
        fields.clone().into(),
        cols.clone(),
        Some(NullBuffer::from(validity)),
    );

    assert_eq!(extract_anyval_from_struct(&sa, 0), None);
    assert_eq!(extract_anyval_from_struct(&sa, 1), None);
    assert_eq!(extract_anyval_from_struct(&sa, 2), None);
    assert_eq!(extract_anyval_from_struct(&sa, 3), None);
    assert_eq!(
        extract_anyval_from_struct(&sa, 4),
        Some(ExtractedValue::Str(Cow::Borrowed("hello")))
    );
    assert_eq!(
        extract_anyval_from_struct(&sa, 5),
        Some(ExtractedValue::Int(123))
    );
    assert_eq!(
        extract_anyval_from_struct(&sa, 6),
        Some(ExtractedValue::Double(4.5))
    );
    assert_eq!(extract_anyval_from_struct(&sa, 7), None);
    assert_eq!(
        extract_anyval_from_struct(&sa, 8),
        Some(ExtractedValue::Bool(true))
    );
    assert_eq!(extract_anyval_from_struct(&sa, 9), None);
    assert_eq!(
        extract_anyval_from_struct(&sa, 10),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[0xaa, 0xbb])))
    );
    assert_eq!(
        extract_anyval_from_struct(&sa, 11),
        Some(ExtractedValue::Map(vec![(
            "k".into(),
            ExtractedValue::Str(Cow::Owned("v".into()))
        )]))
    );
    assert_eq!(
        extract_anyval_from_struct(&sa, 12),
        Some(ExtractedValue::Array(vec![ExtractedValue::Int(5)]))
    );

    // Equivalent RecordBatch via extract_anyval_from_attr_batch
    let rb = RecordBatch::try_new(Arc::new(Schema::new(fields)), cols).unwrap();
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 0),
        Some(ExtractedValue::Str(Cow::Borrowed("masked")))
    );
    assert_eq!(extract_anyval_from_attr_batch(&rb, 1), None);
    assert_eq!(extract_anyval_from_attr_batch(&rb, 2), None);
    assert_eq!(extract_anyval_from_attr_batch(&rb, 3), None);
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 4),
        Some(ExtractedValue::Str(Cow::Borrowed("hello")))
    );
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 5),
        Some(ExtractedValue::Int(123))
    );
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 6),
        Some(ExtractedValue::Double(4.5))
    );
    assert_eq!(extract_anyval_from_attr_batch(&rb, 7), None);
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 8),
        Some(ExtractedValue::Bool(true))
    );
    assert_eq!(extract_anyval_from_attr_batch(&rb, 9), None);
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 10),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[0xaa, 0xbb])))
    );
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 11),
        Some(ExtractedValue::Map(vec![(
            "k".into(),
            ExtractedValue::Str(Cow::Owned("v".into()))
        )]))
    );
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 12),
        Some(ExtractedValue::Array(vec![ExtractedValue::Int(5)]))
    );
    // Missing ATTRIBUTE_TYPE column returns None
    let no_type_sa = StructArray::new(
        vec![Arc::new(Field::new("other", DataType::UInt8, false))].into(),
        vec![Arc::new(UInt8Array::from(vec![1]))],
        None,
    );
    assert_eq!(extract_anyval_from_struct(&no_type_sa, 0), None);
    let no_type_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "other",
            DataType::UInt8,
            false,
        )])),
        vec![Arc::new(UInt8Array::from(vec![1]))],
    )
    .unwrap();
    assert_eq!(extract_anyval_from_attr_batch(&no_type_rb, 0), None);
}

#[test]
fn test_logs_batch_view_and_extract_field_at_row() {
    // Missing Logs payload -> Ok(None)
    let empty_records = OtapArrowRecords::Logs(Logs::default());
    assert!(LogsBatchView::try_new(&empty_records).unwrap().is_none());

    // Zero-row Logs RecordBatch -> Ok(None)
    let zero_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_NUMBER,
            DataType::Int32,
            true,
        )])),
        vec![Arc::new(Int32Array::from(Vec::<i32>::new()))],
    )
    .unwrap();
    let mut zero_records = OtapArrowRecords::Logs(Logs::default());
    zero_records.set(ArrowPayloadType::Logs, zero_rb).unwrap();
    assert!(LogsBatchView::try_new(&zero_records).unwrap().is_none());

    // Minimal batch without ID/RESOURCE/SCOPE columns (identity log ID mapping)
    let minimal_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            true,
        )])),
        vec![Arc::new(StringArray::from(vec![Some("INFO"), None]))],
    )
    .unwrap();
    let mut minimal_records = OtapArrowRecords::Logs(Logs::default());
    minimal_records
        .set(ArrowPayloadType::Logs, minimal_rb)
        .unwrap();
    let view = LogsBatchView::try_new(&minimal_records).unwrap().unwrap();
    assert_eq!(view.log_id_to_row, vec![0, 1]);
    assert_eq!(view.row_to_log_id(0), Some(0));
    assert_eq!(view.row_to_log_id(1), Some(1));
    assert_eq!(view.row_to_resource_id(0), None);
    assert_eq!(view.row_to_scope_id(0), None);
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("INFO")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            1
        ),
        None
    );

    // Rich batch with nullable ID, RESOURCE.ID, SCOPE.ID, Map body, nested attributes, empty fields
    let body_map_bytes = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("req".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("path".into()),
            ciborium::Value::Text("/v1/logs".into()),
        )]),
    )]));
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Str as u8,
            ])),
            Arc::new(StringArray::from(vec![None, Some("row1")])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                Some(body_map_bytes.as_slice()),
                None,
            ])),
        ],
        Some(NullBuffer::from(vec![true, false])),
    );

    let res_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, true))].into(),
        vec![Arc::new(UInt16Array::from(vec![Some(10), None]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, true)),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(vec![Some(20), None])),
            Arc::new(StringArray::from(vec!["my.scope", ""])),
            Arc::new(StringArray::from(vec!["2.3.4", ""])),
        ],
        None,
    );

    let mut trace_nz = [0u8; 16];
    trace_nz[0] = 0xde;
    trace_nz[15] = 0xad;
    let mut span_nz = [0u8; 8];
    span_nz[0] = 0xbe;
    span_nz[7] = 0xef;

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true),
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
            Arc::new(UInt16Array::from(vec![Some(5), None])),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(vec![17, 0])),
            Arc::new(StringArray::from(vec!["ERROR", ""])),
            Arc::new(StringArray::from(vec!["order.failed", ""])),
            Arc::new(
                FixedSizeBinaryArray::try_from_iter(
                    [trace_nz.as_slice(), [0u8; 16].as_slice()].into_iter(),
                )
                .unwrap(),
            ),
            Arc::new(
                FixedSizeBinaryArray::try_from_iter(
                    [span_nz.as_slice(), [0u8; 8].as_slice()].into_iter(),
                )
                .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["https://schema.test", ""])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let nested_cbor = ciborium::Value::Map(vec![(
        ciborium::Value::Text("sub".into()),
        ciborium::Value::Text("deep".into()),
    )]);
    let log_attrs = build_attr_batch(
        vec![(5, "lk", ScalarValue::String("lv".into()))],
        vec![(5, "lmap", nested_cbor.clone())],
    );
    let res_attrs = build_attr_batch(
        vec![(10, "service.name", ScalarValue::String("checkout".into()))],
        vec![(10, "rmap", nested_cbor.clone())],
    );
    let scope_attrs = build_attr_batch(
        vec![(20, "lib.lang", ScalarValue::String("rust".into()))],
        vec![(20, "smap", nested_cbor)],
    );

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records.set(ArrowPayloadType::Logs, root_rb).unwrap();
    records.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
    records
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    records
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    let view = LogsBatchView::try_new(&records).unwrap().unwrap();
    assert_eq!(view.row_to_log_id(0), Some(5));
    assert_eq!(view.row_to_log_id(1), None);
    assert_eq!(view.row_to_resource_id(0), Some(10));
    assert_eq!(view.row_to_resource_id(1), None);
    assert_eq!(view.row_to_scope_id(0), Some(20));
    assert_eq!(view.row_to_scope_id(1), None);

    // RecordField variants
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::Unspecified),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("ERROR")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            1
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("order.failed")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            0
        ),
        Some(ExtractedValue::Int(17))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            1
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::TraceId), 0),
        Some(ExtractedValue::Str(Cow::Owned(hex::encode(trace_nz))))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::TraceId), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::SpanId), 0),
        Some(ExtractedValue::Str(Cow::Owned(hex::encode(span_nz))))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::SpanId), 1),
        None
    );

    // ScopeField variants
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Unspecified), 0),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::SchemaUrl), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("https://schema.test")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::SchemaUrl), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Name), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("my.scope")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Name), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Version), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("2.3.4")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::ScopeField(ScopeField::Version), 1),
        None
    );

    // BodyField (empty path, valid nested path, missing key)
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath { path: vec![] }),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["req", "path"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Owned("/v1/logs".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["req", "missing"])),
            0
        ),
        None
    );

    // LogAttribute, ResourceAttribute, ScopeAttribute (flat & nested CBOR)
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("lk")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("lv")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["lmap", "sub"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Owned("deep".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("checkout")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::nested(["rmap", "sub"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Owned("deep".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::single("lib.lang")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("rust")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::nested(["smap", "sub"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Owned("deep".into())))
    );
}

#[test]
fn test_filter_otap_logs_by_bitmap_and_transform_compile() {
    let batch = build_custom_test_batch(
        vec![Some("r0"), Some("r1"), Some("r2")],
        vec![5, 9, 17],
        vec!["DEBUG", "INFO", "ERROR"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![(0, "k", ScalarValue::Int(0)), (2, "k", ScalarValue::Int(2))],
        vec![],
    );

    let mut pool = IdBitmapPool::default();
    let keep_all = RoaringBitmap::from_sorted_iter([0, 1, 2]).unwrap();
    let all_out = filter_otap_logs_by_bitmap(&batch, &keep_all, 3, &mut pool).unwrap();
    assert_eq!(all_out.get(ArrowPayloadType::Logs).unwrap().num_rows(), 3);

    let partial = RoaringBitmap::from_sorted_iter([0, 2]).unwrap();
    let partial_out = filter_otap_logs_by_bitmap(&batch, &partial, 3, &mut pool).unwrap();
    assert_eq!(
        partial_out.get(ArrowPayloadType::Logs).unwrap().num_rows(),
        2
    );

    // CompiledTransformPolicy::compile: valid None regex, valid Some regex, invalid regex error
    let valid_tx = LogTransformPolicy {
        matches: vec![],
        rename: vec![],
        add: vec![],
        remove: vec![],
        redact: vec![
            LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: None,
                regex: None,
            },
            LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: Some("X".into()),
                regex: Some(r"secret=\w+".into()),
            },
        ],
    };
    let compiled = CompiledTransformPolicy::compile("p1".into(), &valid_tx).unwrap();
    assert_eq!(compiled.redact[0].replacement, "[REDACTED]");
    assert!(compiled.redact[0].regex.is_none());
    assert!(compiled.redact[1].regex.is_some());

    let bad_tx = LogTransformPolicy {
        matches: vec![],
        rename: vec![],
        add: vec![],
        remove: vec![],
        redact: vec![LogRedact {
            target: LogTargetSelector::RecordField(LogRecordField::Body),
            replacement: None,
            regex: Some("[unclosed".into()),
        }],
    };
    let err = CompiledTransformPolicy::compile("bad".into(), &bad_tx).unwrap_err();
    assert!(matches!(err, PolicyError::Compile(_)));
}

#[test]
fn test_apply_spec_transforms_masked_edge_cases() {
    let dummy_tx = CompiledTransformPolicy::compile(
        "p0".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("x")),
                value: ScalarValue::Int(1),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )
    .unwrap();
    let bm0 = RoaringBitmap::from_sorted_iter([0]).unwrap();

    // Empty transforms
    let mut empty_batch = OtapArrowRecords::Logs(Logs::default());
    assert_eq!(
        apply_spec_transforms_masked(&mut empty_batch, &[], &[]).unwrap(),
        0
    );
    // All-empty masks
    assert_eq!(
        apply_spec_transforms_masked(
            &mut empty_batch,
            std::slice::from_ref(&dummy_tx),
            &[RoaringBitmap::new()]
        )
        .unwrap(),
        0
    );
    // Missing Logs batch
    assert_eq!(
        apply_spec_transforms_masked(
            &mut empty_batch,
            std::slice::from_ref(&dummy_tx),
            std::slice::from_ref(&bm0)
        )
        .unwrap(),
        0
    );
    // Zero-row Logs batch
    let zero_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::ID,
            DataType::UInt16,
            false,
        )])),
        vec![Arc::new(UInt16Array::from(Vec::<u16>::new()))],
    )
    .unwrap();
    empty_batch.set(ArrowPayloadType::Logs, zero_rb).unwrap();
    assert_eq!(
        apply_spec_transforms_masked(
            &mut empty_batch,
            std::slice::from_ref(&dummy_tx),
            std::slice::from_ref(&bm0)
        )
        .unwrap(),
        0
    );

    // Missing root columns (BODY, SEVERITY_TEXT, SEVERITY_NUMBER, EVENT_NAME) & missing attr tables
    let minimal_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::ID,
            DataType::UInt16,
            false,
        )])),
        vec![Arc::new(UInt16Array::from(vec![0u16]))],
    )
    .unwrap();
    let mut minimal_batch = OtapArrowRecords::Logs(Logs::default());
    minimal_batch
        .set(ArrowPayloadType::Logs, minimal_root)
        .unwrap();
    let touch_missing_tx = CompiledTransformPolicy::compile(
        "missing".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    value: ScalarValue::String("b".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    value: ScalarValue::String("WARN".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    value: ScalarValue::Int(13),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::EventName),
                    value: ScalarValue::String("ev".into()),
                    upsert: true,
                },
                // Resource/Scope with missing RESOURCE/SCOPE structs hit row_to_res_id == None
                LogAdd {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("rk")),
                    value: ScalarValue::String("rv".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("sk")),
                    value: ScalarValue::String("sv".into()),
                    upsert: true,
                },
            ],
            remove: vec![],
            redact: vec![],
        },
    )
    .unwrap();
    let hits = apply_spec_transforms_masked(
        &mut minimal_batch,
        &[touch_missing_tx],
        std::slice::from_ref(&bm0),
    )
    .unwrap();
    assert_eq!(hits, 4);
    let view_added_cols = LogsBatchView::try_new(&minimal_batch).unwrap().unwrap();
    assert_eq!(
        view_added_cols
            .extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("b")))
    );
    assert_eq!(
        view_added_cols.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("WARN")))
    );
    assert_eq!(
        view_added_cols.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            0
        ),
        Some(ExtractedValue::Int(13))
    );
    assert_eq!(
        view_added_cols
            .extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("ev")))
    );

    // LogAttrs with Empty value (skipped) and CBOR map (preserved byte-for-byte in ATTRIBUTE_SER)
    let cbor_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("k".into()),
        ciborium::Value::Integer(1.into()),
    )]));
    let sparse_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 0])),
            Arc::new(StringArray::from(vec!["empty_k", "map_k"])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Empty as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                Some(cbor_map.as_slice()),
            ])),
        ],
    )
    .unwrap();
    minimal_batch
        .set(ArrowPayloadType::LogAttrs, sparse_attrs)
        .unwrap();
    assert_eq!(
        apply_spec_transforms_masked(&mut minimal_batch, &[dummy_tx], &[bm0]).unwrap(),
        1
    );
    let view_sparse = LogsBatchView::try_new(&minimal_batch).unwrap().unwrap();
    assert_eq!(
        view_sparse.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["map_k", "k"])),
            0,
        ),
        Some(ExtractedValue::Int(1))
    );

    // Whole-value default redaction on Body when LogAttrs is not touched (touches_log_attrs false)
    let mut body_only_batch = build_custom_test_batch(
        vec![Some("secret=42"), None],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![],
        vec![],
    );
    let redact_body = CompiledTransformPolicy::compile(
        "p_body_only".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: None,
                regex: None,
            }],
        },
    )
    .unwrap();
    let both = RoaringBitmap::from_sorted_iter([0, 1]).unwrap();
    let body_hits =
        apply_spec_transforms_masked(&mut body_only_batch, &[redact_body], &[both]).unwrap();
    assert_eq!(body_hits, 1);
    let view = LogsBatchView::try_new(&body_only_batch).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]")))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
        None
    );
}

#[test]
fn test_apply_spec_transforms_all_targets_and_scalar_variants() {
    // Build a 7-row batch whose BODY struct holds all 7 variants (Str, Int, Double, Bool, Bytes,
    // Map, Empty) so both extract_root_body_vec and build_body_struct_array hit every variant.
    let map_bytes = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("m".into()),
        ciborium::Value::Integer(1.into()),
    )]));
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )),
            Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Int as u8,
                AttributeValueType::Double as u8,
                AttributeValueType::Bool as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Empty as u8,
            ])),
            Arc::new(StringArray::from(vec![
                Some("tok=abc"),
                None,
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Int64Array::from(vec![
                None,
                Some(77),
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Float64Array::from(vec![
                None,
                None,
                Some(2.25),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                None,
                None,
                Some(true),
                None,
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                Some(&[0xde, 0xad]),
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                None,
                Some(map_bytes.as_slice()),
                None,
            ])),
        ],
        None,
    );

    let res_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, true))].into(),
        vec![Arc::new(UInt16Array::from(vec![
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            None,
        ]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, true))].into(),
        vec![Arc::new(UInt16Array::from(vec![
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            None,
        ]))],
        None,
    );

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::EVENT_NAME, DataType::Utf8, true),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3, 4, 5, 6])),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(vec![9, 17, 0, 13, 5, 1, 0])),
            Arc::new(StringArray::from(vec![
                "INFO", "ERROR", "", "WARN", "DEBUG", "TRACE", "",
            ])),
            Arc::new(StringArray::from(vec![
                "evt0", "evt1", "", "evt3", "evt4", "evt5", "",
            ])),
            Arc::new(
                FixedSizeBinaryArray::try_from_iter([[1u8; 16].as_slice(); 7].into_iter()).unwrap(),
            ),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    // All 5 scalar types in LogAttrs, ResourceAttrs, and ScopeAttrs
    let log_attrs = build_attr_batch(
        vec![
            (0, "s", ScalarValue::String("val".into())),
            (0, "i", ScalarValue::Int(12)),
            (0, "d", ScalarValue::Double(1.5)),
            (0, "b", ScalarValue::Bool(true)),
            (0, "raw", ScalarValue::Bytes(vec![0x01, 0x02])),
            (0, "keep_src", ScalarValue::String("src".into())),
            (0, "existing_dst", ScalarValue::String("dst".into())),
            (0, "over_src", ScalarValue::String("new_dst".into())),
            (0, "over_dst", ScalarValue::String("old_dst".into())),
            (0, "no_match", ScalarValue::String("clean".into())),
            (0, "num_attr", ScalarValue::Int(99)),
        ],
        vec![],
    );
    let res_attrs = build_attr_batch(
        vec![
            (0, "rs", ScalarValue::String("res_old".into())),
            (0, "ri", ScalarValue::Int(5)),
            (0, "rd", ScalarValue::Double(2.5)),
            (0, "rb", ScalarValue::Bool(false)),
            (0, "rbytes", ScalarValue::Bytes(vec![0xaa])),
            (0, "r_drop", ScalarValue::String("bye".into())),
        ],
        vec![],
    );
    let scope_attrs = build_attr_batch(
        vec![
            (0, "ss", ScalarValue::String("scope_old".into())),
            (0, "s_drop", ScalarValue::String("bye".into())),
        ],
        vec![],
    );

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
    batch
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    batch
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    // First policy has an empty mask to exercise `if mask.is_empty() { continue; }`
    let tx_empty_mask = CompiledTransformPolicy::compile(
        "0_skipped".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![LogRename {
                from: LogTargetSelector::LogAttribute(AttributePath::single("s")),
                to: LogTargetSelector::LogAttribute(AttributePath::single("s2")),
                upsert: true,
            }],
            add: vec![],
            remove: vec![],
            redact: vec![],
        },
    )
    .unwrap();

    // Second policy exercises every selector, macro arm, type mismatch, and multi-segment no-op
    let tx_full = CompiledTransformPolicy::compile(
        "1_full".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![
                // Same target (from == to) skipped
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("s")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("s")),
                    upsert: true,
                },
                // Missing source skipped
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("does_not_exist")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("dest_x")),
                    upsert: true,
                },
                // upsert=false when destination exists -> skipped, source kept intact
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("keep_src")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("existing_dst")),
                    upsert: false,
                },
                // upsert=true overwrites destination
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("over_src")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("over_dst")),
                    upsert: true,
                },
                // Rename resource & scope attributes
                LogRename {
                    from: LogTargetSelector::ResourceAttribute(AttributePath::single("rs")),
                    to: LogTargetSelector::ResourceAttribute(AttributePath::single("rs_new")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::ScopeAttribute(AttributePath::single("ss")),
                    to: LogTargetSelector::ScopeAttribute(AttributePath::single("ss_new")),
                    upsert: true,
                },
                // Rename SeverityNumber -> LogAttribute and EventName -> LogAttribute
                LogRename {
                    from: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("prev_sev")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("prev_sev_text")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::RecordField(LogRecordField::EventName),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("prev_event")),
                    upsert: true,
                },
                // Unsupported target (TraceId) & multi-segment paths (no-ops)
                LogRename {
                    from: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("trace_copy")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("c")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::ResourceAttribute(AttributePath::nested(["a", "b"])),
                    to: LogTargetSelector::ResourceAttribute(AttributePath::single("c")),
                    upsert: true,
                },
                LogRename {
                    from: LogTargetSelector::ScopeAttribute(AttributePath::nested(["a", "b"])),
                    to: LogTargetSelector::ScopeAttribute(AttributePath::single("c")),
                    upsert: true,
                },
            ],
            add: vec![
                // Valid writes to root fields & attributes
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    value: ScalarValue::String("CRITICAL".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    value: ScalarValue::Int(21),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::EventName),
                    value: ScalarValue::String("new.evt".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("added_attr")),
                    value: ScalarValue::String("added".into()),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("r_added")),
                    value: ScalarValue::Int(7),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("s_added")),
                    value: ScalarValue::Bool(true),
                    upsert: false,
                },
                // Type mismatches & overflows (no-ops, hit false branches in set_target!)
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    value: ScalarValue::Int(i64::MAX),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    value: ScalarValue::String("bad".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::EventName),
                    value: ScalarValue::Bool(false),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    value: ScalarValue::String("ignored".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::nested(["a", "b"])),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::nested(["a", "b"])),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
            ],
            remove: vec![
                LogRemove {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("r_drop")),
                },
                LogRemove {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("s_drop")),
                },
                LogRemove {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("missing_rem")),
                },
                LogRemove {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                },
                LogRemove {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                },
                LogRemove {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::nested(["a", "b"])),
                },
                LogRemove {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::nested(["a", "b"])),
                },
            ],
            redact: vec![
                // Regex match on Body
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some("tok=[MASK]".into()),
                    regex: Some("tok=[a-z]+".into()),
                },
                // Whole-value default "[REDACTED]" on ResourceAttribute and ScopeAttribute
                LogRedact {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("rs_new")),
                    replacement: None,
                    regex: None,
                },
                LogRedact {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("ss_new")),
                    replacement: Some("HIDDEN".into()),
                    regex: None,
                },
                // Whole-value on SeverityNumber fails type check (String cannot write to i32)
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                    replacement: None,
                    regex: None,
                },
                // Regex non-match on String attr & non-string Int attr & missing attr
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("no_match")),
                    replacement: Some("X".into()),
                    regex: Some("zzz".into()),
                },
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("num_attr")),
                    replacement: Some("X".into()),
                    regex: Some("99".into()),
                },
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("absent_key")),
                    replacement: None,
                    regex: None,
                },
            ],
        },
    )
    .unwrap();

    // Third policy removes Body on row 6 and SeverityText on row 6 (null res_id/scope_id too)
    let tx_remove_body = CompiledTransformPolicy::compile(
        "2_remove_root".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("no_res")),
                    value: ScalarValue::Int(1),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("no_scope")),
                    value: ScalarValue::Int(1),
                    upsert: false,
                },
            ],
            remove: vec![
                LogRemove {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                },
                LogRemove {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                },
                LogRemove {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                },
                LogRemove {
                    target: LogTargetSelector::RecordField(LogRecordField::EventName),
                },
                LogRemove {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("no_res")),
                },
                LogRemove {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("no_scope")),
                },
            ],
            redact: vec![],
        },
    )
    .unwrap();

    let hits = apply_spec_transforms_masked(
        &mut batch,
        &[tx_empty_mask, tx_full, tx_remove_body],
        &[
            RoaringBitmap::new(),
            RoaringBitmap::from_sorted_iter([0]).unwrap(),
            RoaringBitmap::from_sorted_iter([5, 6]).unwrap(),
        ],
    )
    .unwrap();
    assert!(hits > 10);

    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();
    // Row 0 checks
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("tok=[MASK]")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("CRITICAL")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            0
        ),
        Some(ExtractedValue::Int(21))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 0),
        Some(ExtractedValue::Str(Cow::Borrowed("new.evt")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("keep_src")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("src")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("over_dst")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("new_dst")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("added_attr")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("added")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::single("rs_new")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::single("r_drop")),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::single("ss_new")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("HIDDEN")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::single("s_drop")),
            0
        ),
        None
    );

    // Rows 1..=4 preserved Int, Double, Bool, Bytes body values through build_body_struct_array
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 1),
        Some(ExtractedValue::Int(77))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 2),
        Some(ExtractedValue::Double(2.25))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 3),
        Some(ExtractedValue::Bool(true))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 4),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[0xde, 0xad])))
    );
    // Row 5 had root fields removed by tx_remove_body
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 5),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            5
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            5
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 5),
        None
    );
}

// ============================================================================
// engine.rs unit tests
// ============================================================================

#[test]
fn test_extract_required_ngrams_all_hir_kinds() {
    // Pure literal >= 2 bytes and < 2 bytes
    assert_eq!(
        extract_required_ngrams(&parse_hir("hello")),
        vec![b"hello".to_vec()]
    );
    assert!(extract_required_ngrams(&parse_hir("a")).is_empty());

    // Pure literal inside Capture and Concat of pure literals
    assert_eq!(
        extract_required_ngrams(&parse_hir("(abcd)")),
        vec![b"abcd".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab)(cd)")),
        vec![b"abcd".to_vec()]
    );

    // Duplicate n-grams are deduplicated
    assert_eq!(
        extract_required_ngrams(&parse_hir("tok.*tok")),
        vec![b"tok".to_vec()]
    );

    // Empty, Class, and Look produce no n-grams
    assert!(extract_required_ngrams(&parse_hir("")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("[a-z0-9]+")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("^$")).is_empty());
    assert!(extract_required_ngrams(&parse_hir(r"\b")).is_empty());

    // Capture wrapping non-pure HIR
    assert_eq!(
        extract_required_ngrams(&parse_hir("(alpha.*beta)")),
        vec![b"alpha".to_vec(), b"beta".to_vec()]
    );

    // Repetition: min == 0, min >= 1 pure literal (short, normal, clamped > 16), non-pure literal
    assert!(extract_required_ngrams(&parse_hir("(abc)*")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("(abc)?")).is_empty());
    assert!(extract_required_ngrams(&parse_hir("x{1,3}")).is_empty());
    assert_eq!(
        extract_required_ngrams(&parse_hir("a{2,5}")),
        vec![b"aa".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab){2,}")),
        vec![b"abab".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("a{20,30}")),
        vec![vec![b'a'; 16]]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("(ab){20,}")),
        vec![b"ab".repeat(16)]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir(r"(err\d+)+")),
        vec![b"err".to_vec()]
    );

    // Concat: short literal cleared before wildcard, trailing literal >= 2 and < 2
    assert_eq!(
        extract_required_ngrams(&parse_hir("x.*tail")),
        vec![b"tail".to_vec()]
    );
    assert_eq!(
        extract_required_ngrams(&parse_hir("head.*x")),
        vec![b"head".to_vec()]
    );

    // Alternation: common n-gram across non-prefix-factorable branches, disjoint 3-branch break
    assert_eq!(
        extract_required_ngrams(&parse_hir("(a.*tok|b.*tok)")),
        vec![b"tok".to_vec()]
    );
    assert!(extract_required_ngrams(&parse_hir("(aaa.*1|bbb.*2|ccc.*3)")).is_empty());
    assert!(extract_required_ngrams(&Hir::alternation(vec![])).is_empty());
}

#[test]
fn test_string_index_multigram_fallback_dedup_and_heap_bytes() {
    let batch = build_custom_test_batch(
        vec![
            // repeated n-gram "err", hits "err.*timeout" & contains("timeout")
            Some("err err timeout here"),
            Some("alpha only"),             // 1/3 required n-grams
            Some("alpha beta"),             // 2/3 required n-grams
            Some("gamma beta alpha"),       // 3/3 n-grams in wrong order -> MetaRegex rejects
            Some("alpha 1 beta 2 gamma"),   // 3/3 n-grams in order -> matches
            Some("98765"),                  // matches zero-ngram fallback regex ^\d+$
            Some("has z char"),             // matches 1-byte fallback contains("z")
            Some("err connection refused"), // shared n-gram "err" matches second policy
            Some("clean log"),              // matches nothing
        ],
        vec![9; 9],
        vec!["INFO"; 9],
        vec![[1u8; 16]; 9],
        vec![[1u8; 8]; 9],
        vec![],
        vec![],
    );

    let policies = vec![
        // Multi-ngram regex (3 required n-grams)
        Policy::log_filter(
            "drop_alpha_beta_gamma",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("alpha.*beta.*gamma".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Two regexes sharing the same n-gram "err" (tests intern_ngram cache + posting list)
        Policy::log_filter(
            "drop_err_timeout",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("err.*timeout".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_err_refused",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("err.*refused".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Indexed Contains (>= 2 bytes) that actually matches
        Policy::log_filter(
            "drop_contains_timeout",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("timeout".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // Fallback Contains (< 2 bytes)
        Policy::log_filter(
            "drop_contains_z",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("z".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        // Fallback Regex (0 required n-grams)
        Policy::log_filter(
            "drop_digits_only",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex(r"^\d+$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
    ];

    let engine = compile_engine(&policies);
    assert_eq!(engine.name(), "PolicyEngine");

    // Verify heap estimation covers empty vs populated engines (indexed + fallback Contains/Regex)
    let empty_engine = compile_engine(&[]);
    assert!(engine.estimated_heap_bytes() > empty_engine.estimated_heap_bytes());

    let fallback_only_engine = compile_engine(&[Policy::log_filter(
        "fb_only",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Contains(ScalarValue::String("z".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert!(fallback_only_engine.estimated_heap_bytes() > empty_engine.estimated_heap_bytes());

    // Rows 0, 4, 5, 6, 7 are dropped; rows 1, 2, 3, 8 are kept
    let kept = engine.evaluate_filter(&batch).unwrap();
    assert_eq!(kept, RoaringBitmap::from_sorted_iter([1, 2, 3, 8]).unwrap());
}

#[test]
fn test_engine_compile_edge_cases_and_errors() {
    let disabled = Policy {
        id: "disabled".into(),
        name: "disabled".into(),
        enabled: false,
        definition: PolicyDefinition::LogFilter(LogFilterPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("[unclosed_ignored".into()),
                negate: false,
            }],
            action: Action::Drop,
        }),
    };
    let unspecified = Policy::log_filter(
        "unspec",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Unspecified,
    );
    let empty_matches = Policy::log_filter("empty", vec![], Action::Drop);
    let metric = Policy {
        id: "metric".into(),
        name: "metric".into(),
        enabled: true,
        definition: PolicyDefinition::MetricFilter(MetricFilterPolicy {
            action: Action::Drop,
        }),
    };
    let trace = Policy {
        id: "trace".into(),
        name: "trace".into(),
        enabled: true,
        definition: PolicyDefinition::TraceSampling(TraceSamplingPolicy { probability: 0.5 }),
    };

    // Identical predicate across filter & transform rules exercises pred_dedup & field_index_map
    let shared_matcher = LogMatcher {
        field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
        predicate: Predicate::Equals(ScalarValue::String("prod".into())),
        negate: false,
    };
    let f1 = Policy::log_filter("f1", vec![shared_matcher.clone()], Action::Drop);
    let f2 = Policy::log_filter(
        "f2",
        vec![LogMatcher {
            negate: true,
            ..shared_matcher.clone()
        }],
        Action::Keep,
    );
    let tx = Policy::log_transform(
        "tx1",
        LogTransformPolicy {
            matches: vec![shared_matcher],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("tagged")),
                value: ScalarValue::Bool(true),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    );
    // Empty attribute paths should compile without panic
    let empty_paths = Policy::log_filter(
        "empty_paths",
        vec![
            LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ResourceAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
            LogMatcher {
                field: LogFieldSelector::ScopeAttribute(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            },
        ],
        Action::Drop,
    );

    let engine = compile_engine(&[
        disabled,
        unspecified,
        empty_matches,
        metric,
        trace,
        f1,
        f2,
        tx,
        empty_paths,
    ]);
    assert!(engine.estimated_heap_bytes() > 0);

    // Invalid regex in filter matcher
    let bad_filter = Policy::log_filter(
        "bad_f",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Regex("[bad".into()),
            negate: false,
        }],
        Action::Drop,
    );
    assert!(matches!(
        PolicyEngine::compile(&[bad_filter]),
        Err(PolicyError::Compile(_))
    ));

    // Invalid regex in transform matcher
    let bad_tx_matcher = Policy::log_transform(
        "bad_tx_m",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Regex("[bad".into()),
                negate: false,
            }],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![],
        },
    );
    assert!(matches!(
        PolicyEngine::compile(&[bad_tx_matcher]),
        Err(PolicyError::Compile(_))
    ));

    // Invalid regex in transform redact
    let bad_tx_redact = Policy::log_transform(
        "bad_tx_r",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                replacement: None,
                regex: Some("[bad".into()),
            }],
        },
    );
    assert!(matches!(
        PolicyEngine::compile(&[bad_tx_redact]),
        Err(PolicyError::Compile(_))
    ));
}

#[test]
fn test_engine_all_selectors_types_and_missing_columns() {
    // 1. Minimal batch missing BODY, TraceId, SpanId, EventName, SeverityNumber columns
    let minimal_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2])),
            Arc::new(StringArray::from(vec![
                Some("hello world"),
                Some("exact"),
                None,
            ])),
        ],
    )
    .unwrap();
    let mut minimal_batch = OtapArrowRecords::Logs(Logs::default());
    minimal_batch
        .set(ArrowPayloadType::Logs, minimal_root)
        .unwrap();

    let missing_cols_engine = compile_engine(&[
        // Exercises eval_str_only (Exists + str_equals_map + string_index) on SeverityText
        Policy::log_filter(
            "eq_exact",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("exact".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "contains_world",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Contains(ScalarValue::String("world".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Missing BODY column for both Body and BodyField selectors
        Policy::log_filter(
            "body_exists",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "bf_exists",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath::single("x")),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Empty BodyField path
        Policy::log_filter(
            "body_field_empty",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath { path: vec![] }),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Unspecified RecordField & ScopeField
        Policy::log_filter(
            "unspec_fields",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Unspecified),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "unspec_scope",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Unspecified),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Missing TraceId column
        Policy::log_filter(
            "missing_trace",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        missing_cols_engine.evaluate_filter(&minimal_batch).unwrap(),
        RoaringBitmap::from_sorted_iter([2]).unwrap()
    );

    // 2. Rich batch testing AnyValColumns (null struct, null type, Empty, Str, Int, Double, Bool,
    //    Bytes, Map, Slice), ScopeField (Name, Version, SchemaUrl), SpanId, SeverityText,
    //    EventName, SeverityNumber (0 absent), BodyField nested map, ResourceAttrs & ScopeAttrs
    //    shared across multiple rows!
    let body_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("req".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("code".into()),
            ciborium::Value::Integer(500.into()),
        )]),
    )]));
    let body_slice = encode_cbor(&ciborium::Value::Array(vec![
        ciborium::Value::Text("a".into()),
        ciborium::Value::Integer(42.into()),
    ]));

    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )),
            Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,    // row 0: masked by struct validity
                255,                              // row 1: invalid type discriminant
                AttributeValueType::Empty as u8,  // row 2: Empty
                AttributeValueType::Int as u8,    // row 3: Int(100)
                AttributeValueType::Double as u8, // row 4: Double(12.5)
                AttributeValueType::Double as u8, // row 5: Double(NaN)
                AttributeValueType::Bool as u8,   // row 6: Bool(true)
                AttributeValueType::Bytes as u8,  // row 7: Bytes([0xde, 0xad, 0xbe, 0xef])
                AttributeValueType::Map as u8,    // row 8: CBOR Map {req: {code: 500}}
                AttributeValueType::Slice as u8,  // row 9: CBOR Slice ["a", 42]
            ])),
            Arc::new(StringArray::from(vec![
                Some("hidden"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Int64Array::from(vec![
                None,
                None,
                None,
                Some(100),
                None,
                None,
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Float64Array::from(vec![
                None,
                None,
                None,
                None,
                Some(12.5),
                Some(f64::NAN),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(true),
                None,
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(&[0xde, 0xad, 0xbe, 0xef]),
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(body_map.as_slice()),
                Some(body_slice.as_slice()),
            ])),
        ],
        Some(NullBuffer::from(vec![
            false, true, true, true, true, true, true, true, true, true,
        ])),
    );

    // Rows 0, 1 share resource_id=0 and scope_id=0; rows 2..10 have resource_id=1, scope_id=1
    let res_ids = vec![0u16, 0, 1, 1, 1, 1, 1, 1, 1, 1];
    let scope_ids = vec![0u16, 0, 1, 1, 1, 1, 1, 1, 1, 1];
    let res_struct = StructArray::new(
        vec![Arc::new(
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        )]
        .into(),
        vec![Arc::new(UInt16Array::from(res_ids))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()),
            Arc::new(Field::new(consts::NAME, DataType::Utf8, true)),
            Arc::new(Field::new(consts::VERSION, DataType::Utf8, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt16Array::from(scope_ids)),
            Arc::new(StringArray::from(vec![
                "scope.alpha",
                "",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
                "scope.beta",
            ])),
            Arc::new(StringArray::from(vec![
                "1.0", "", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0", "2.0",
            ])),
        ],
        None,
    );

    let mut span_nz = [0u8; 8];
    span_nz[0] = 0x12;
    span_nz[7] = 0xef;
    let span_zero = [0u8; 8];
    let span_slices = [
        span_nz.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
        span_zero.as_slice(),
    ];

    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::EVENT_NAME, DataType::Utf8, true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from((0..10u16).collect::<Vec<_>>())),
            Arc::new(body_struct),
            Arc::new(Int32Array::from(vec![0, 5, 9, 13, 17, 21, 0, 9, 9, 9])),
            Arc::new(StringArray::from(vec![
                "", "DEBUG", "INFO", "WARN", "ERROR", "FATAL", "", "INFO", "INFO", "INFO",
            ])),
            Arc::new(StringArray::from(vec![
                "",
                "evt.login",
                "evt.ok",
                "evt.ok",
                "evt.ok",
                "evt.ok",
                "",
                "evt.ok",
                "evt.ok",
                "evt.ok",
            ])),
            Arc::new(FixedSizeBinaryArray::try_from_iter(span_slices.into_iter()).unwrap()),
            Arc::new(StringArray::from(vec![
                "https://schema.v1",
                "",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
                "https://schema.v2",
            ])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let res_attrs = build_attr_batch(
        vec![(0, "service.name", ScalarValue::String("billing".into()))],
        vec![],
    );
    let scope_attrs = build_attr_batch(
        vec![(0, "instr.lib", ScalarValue::String("otel-rs".into()))],
        vec![],
    );
    // LogAttrs with Empty value, out-of-range parent_id, nested CBOR, and flat string
    let nested_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("inner".into()),
        ciborium::Value::Text("hit".into()),
    )]));
    let log_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![
                0u16, // Empty value
                99,   // out-of-range parent_id
                2,    // CBOR Map
                3,    // flat string
            ])),
            Arc::new(StringArray::from(vec![
                "shared", "shared", "shared", "shared",
            ])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Empty as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Str as u8,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("ignored"),
                None,
                Some("flat"),
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                None,
                Some(nested_map.as_slice()),
                None,
            ])),
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
    batch
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    batch
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    // Evaluate every Body ExtractedValue variant (Int, Double, NaN, Bool, Bytes, Map, Slice)
    let body_preds = vec![
        // Row 3: Body == Int(100) and Gte(100), Lte(100)
        Policy::log_filter(
            "int_eq",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Int(100)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gte(NumericValue::Int(100)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lte(NumericValue::Int(100)),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 4: Double(12.5) with Gt/Gte/Lt/Lte; row 5 is NaN (must not match)
        Policy::log_filter(
            "dbl_range",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Double(12.5)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gt(NumericValue::Double(10.0)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Gte(NumericValue::Double(12.5)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lt(NumericValue::Double(13.0)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Lte(NumericValue::Double(12.5)),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 6: Body == Bool(true)
        Policy::log_filter(
            "bool_eq",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::Bool(true)),
                negate: false,
            }],
            Action::Drop,
        ),
        // Row 7: Body == Bytes and Contains(Bytes)
        Policy::log_filter(
            "bytes_match",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Contains(ScalarValue::Bytes(vec![0xad, 0xbe])),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 8: Body is Map -> BodyField(["req", "code"]) == 500 & Body Exists
        Policy::log_filter(
            "body_map_nested",
            vec![
                LogMatcher {
                    field: LogFieldSelector::BodyField(AttributePath::nested(["req", "code"])),
                    predicate: Predicate::Equals(ScalarValue::Int(500)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Row 9: Body is Slice -> Contains(Int(42))
        Policy::log_filter(
            "body_slice",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::Int(42)),
                negate: false,
            }],
            Action::Drop,
        ),
    ];
    let eng_body = compile_engine(&body_preds);
    // Rows 3, 4, 6, 7, 8, 9 dropped; rows 0 (null struct), 1 (null type), 2 (Empty), 5 (NaN) kept
    assert_eq!(
        eng_body.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 1, 2, 5]).unwrap()
    );

    // Evaluate SeverityText, EventName, SpanId, ScopeField::{Name, Version, SchemaUrl}
    let root_eng = compile_engine(&[
        Policy::log_filter(
            "sev_text",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "evt_name",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::EventName),
                predicate: Predicate::Contains(ScalarValue::String("login".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "span_id",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Equals(ScalarValue::String(hex::encode(span_nz))),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "scope_name",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Name),
                predicate: Predicate::Equals(ScalarValue::String("scope.alpha".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "scope_ver",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Version),
                predicate: Predicate::Equals(ScalarValue::String("1.0".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "schema_url",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::SchemaUrl),
                predicate: Predicate::Regex(r"schema\.v1$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        // Exercises SeverityNumber == 0 absent semantics and exact Int equality on root field
        Policy::log_filter(
            "sev_num_fatal",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Equals(ScalarValue::Int(21)),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    // Drops rows 0 (span_id/scope), 1 (evt.login), 3 (WARN), 5 (sev=21); rows 0 and 6 have sev=0
    assert_eq!(
        root_eng.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([2, 4, 6, 7, 8, 9]).unwrap()
    );

    // Evaluate ResourceAttrs & ScopeAttrs matching BOTH rows 0 and 1 sharing resource_id/scope_id=0
    let res_eng = compile_engine(&[Policy::log_filter(
        "res_billing",
        vec![LogMatcher {
            field: LogFieldSelector::ResourceAttribute(AttributePath::single("service.name")),
            predicate: Predicate::Equals(ScalarValue::String("billing".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        res_eng.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([2, 3, 4, 5, 6, 7, 8, 9]).unwrap()
    );

    let scope_eng = compile_engine(&[Policy::log_filter(
        "scope_lib",
        vec![LogMatcher {
            field: LogFieldSelector::ScopeAttribute(AttributePath::single("instr.lib")),
            predicate: Predicate::Equals(ScalarValue::String("otel-rs".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        scope_eng.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([2, 3, 4, 5, 6, 7, 8, 9]).unwrap()
    );

    // Same top-level key "shared" split across a filter-only evaluator (["shared"]) and a
    // transform-only evaluator (["shared", "inner"]) + filter-only root vs transform-only root
    // exercises should_run() skipping inside both evaluate_predicates and evaluate_attr_table!
    let split_eng = compile_engine(&[
        Policy::log_filter(
            "drop_shared_flat",
            vec![
                LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("shared")),
                    predicate: Predicate::Equals(ScalarValue::String("flat".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("WARN".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_nested_shared",
            LogTransformPolicy {
                matches: vec![
                    LogMatcher {
                        field: LogFieldSelector::LogAttribute(AttributePath::nested([
                            "shared", "inner",
                        ])),
                        predicate: Predicate::Equals(ScalarValue::String("hit".into())),
                        negate: false,
                    },
                    LogMatcher {
                        field: LogFieldSelector::RecordField(LogRecordField::EventName),
                        predicate: Predicate::Equals(ScalarValue::String("evt.ok".into())),
                        negate: false,
                    },
                ],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("tx_hit")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    // Drops only row 3 ("flat" + "WARN"); transform tags row 2 (CBOR {"inner": "hit"} + "evt.ok")
    assert_eq!(
        split_eng.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 1, 2, 4, 5, 6, 7, 8, 9]).unwrap()
    );
    let out = split_eng.evaluate(batch).unwrap();
    assert_eq!(out.kept_rows, 9);
    assert_eq!(out.dropped_rows, 1);
    assert_eq!(out.transform_hits, 1);
}

#[test]
fn test_engine_negation_conjunction_and_evaluate_lifecycle() {
    let batch = build_custom_test_batch(
        vec![Some("a"), Some("b"), Some("c")],
        vec![5, 9, 17],
        vec!["DEBUG", "INFO", "ERROR"],
        vec![[1u8; 16]; 3],
        vec![[1u8; 8]; 3],
        vec![
            (0, "keep_me", ScalarValue::Bool(true)),
            (1, "keep_me", ScalarValue::Bool(false)),
        ],
        vec![],
    );

    // 1. Empty / missing Logs batch in evaluate_filter and evaluate
    let empty_batch = OtapArrowRecords::Logs(Logs::default());
    let any_engine = compile_engine(&[Policy::log_filter(
        "drop_info",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("INFO".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert!(any_engine.evaluate_filter(&empty_batch).unwrap().is_empty());
    let empty_out = any_engine.evaluate(empty_batch).unwrap();
    assert!(empty_out.records.is_some());
    assert_eq!(empty_out.kept_rows, 0);
    assert_eq!(empty_out.dropped_rows, 0);
    assert_eq!(empty_out.transform_hits, 0);

    // 2. Empty engine (no filters, no transforms): fast path in evaluate_filter and evaluate
    let noop_engine = compile_engine(&[]);
    assert_eq!(
        noop_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 1, 2]).unwrap()
    );
    let noop_out = noop_engine.evaluate(batch.clone()).unwrap();
    assert_eq!(noop_out.kept_rows, 3);
    assert_eq!(noop_out.dropped_rows, 0);
    assert_eq!(noop_out.transform_hits, 0);

    // 3. No filters + conditional transform (fast path with predicates!)
    let tx_only = compile_engine(&[Policy::log_transform(
        "add_tag_on_error",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                negate: false,
            }],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("alert")),
                value: ScalarValue::Bool(true),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let tx_out = tx_only.evaluate(batch.clone()).unwrap();
    assert_eq!(tx_out.kept_rows, 3);
    assert_eq!(tx_out.dropped_rows, 0);
    assert_eq!(tx_out.transform_hits, 1);

    // 4. Filter dropping ALL rows -> records == None
    let drop_all = compile_engine(&[Policy::log_filter(
        "drop_everything",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::Body),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    let dropped_out = drop_all.evaluate(batch.clone()).unwrap();
    assert!(dropped_out.records.is_none());
    assert_eq!(dropped_out.kept_rows, 0);
    assert_eq!(dropped_out.dropped_rows, 3);
    assert_eq!(dropped_out.transform_hits, 0);

    // 5. Partial drop WITHOUT transforms
    let partial_out = any_engine.evaluate(batch.clone()).unwrap();
    assert!(partial_out.records.is_some());
    assert_eq!(partial_out.kept_rows, 2);
    assert_eq!(partial_out.dropped_rows, 1);
    assert_eq!(partial_out.transform_hits, 0);

    // 6. Negation (first matcher negated, subsequent matcher negated, subsequent positive, and
    //    empty-accumulator short-circuit break)
    let neg_engine = compile_engine(&[
        // Drop rows where NOT ERROR AND NOT DEBUG -> drops row 1 (INFO)
        Policy::log_filter(
            "neg_first_and_second",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("ERROR".into())),
                    negate: true,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
                    negate: true,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Equals(ScalarValue::String("b".into())),
                    negate: false,
                },
            ],
            Action::Drop,
        ),
        // Short-circuit: first matcher yields empty bitmap so loop breaks on 2nd matcher
        Policy::log_filter(
            "short_circuit",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("NONEXISTENT".into())),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::Body),
                    predicate: Predicate::Exists,
                    negate: false,
                },
            ],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        neg_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 2]).unwrap()
    );

    // 7. Filter keeping all rows + conditional transform (filter_otap_logs_by_bitmap shortcut)
    let keep_all_with_tx = compile_engine(&[
        Policy::log_filter(
            "drop_none",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Equals(ScalarValue::String("FATAL".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "tag_keep_me",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("keep_me")),
                    predicate: Predicate::Equals(ScalarValue::Bool(true)),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("marked")),
                    value: ScalarValue::Int(1),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let kept_all_out = keep_all_with_tx.evaluate(batch).unwrap();
    assert_eq!(kept_all_out.kept_rows, 3);
    assert_eq!(kept_all_out.dropped_rows, 0);
    assert_eq!(kept_all_out.transform_hits, 1);
}

#[test]
fn test_canonical_predicate_key_dedup_all_variants_and_floats() {
    let field = LogFieldSelector::LogAttribute(AttributePath::single("x"));
    let quiet_nan_alt = f64::from_bits(0x7ff8_0000_0000_0001);

    let mk = |id: &str, pred: Predicate| {
        Policy::log_filter(
            id,
            vec![LogMatcher {
                field: field.clone(),
                predicate: pred,
                negate: false,
            }],
            Action::Drop,
        )
    };

    // Each unique predicate appears twice (with -0.0/+0.0 and distinct NaN payloads canonicalized)
    let policies = vec![
        mk("p_exists_1", Predicate::Exists),
        mk("p_exists_2", Predicate::Exists),
        mk(
            "p_eq_str_1",
            Predicate::Equals(ScalarValue::String("a".into())),
        ),
        mk(
            "p_eq_str_2",
            Predicate::Equals(ScalarValue::String("a".into())),
        ),
        mk("p_eq_int_1", Predicate::Equals(ScalarValue::Int(0))),
        mk("p_eq_int_2", Predicate::Equals(ScalarValue::Int(0))),
        mk("p_eq_bool_1", Predicate::Equals(ScalarValue::Bool(true))),
        mk("p_eq_bool_2", Predicate::Equals(ScalarValue::Bool(true))),
        mk("p_eq_bool_f", Predicate::Equals(ScalarValue::Bool(false))),
        mk(
            "p_eq_bytes_1",
            Predicate::Equals(ScalarValue::Bytes(vec![1, 2])),
        ),
        mk(
            "p_eq_bytes_2",
            Predicate::Equals(ScalarValue::Bytes(vec![1, 2])),
        ),
        // +0.0 and -0.0 collapse to the same canonical key
        mk("p_eq_pos_zero", Predicate::Equals(ScalarValue::Double(0.0))),
        mk(
            "p_eq_neg_zero",
            Predicate::Equals(ScalarValue::Double(-0.0)),
        ),
        // Distinct NaN bit patterns collapse to the same canonical key
        mk(
            "p_eq_nan_1",
            Predicate::Equals(ScalarValue::Double(f64::NAN)),
        ),
        mk(
            "p_eq_nan_2",
            Predicate::Equals(ScalarValue::Double(-f64::NAN)),
        ),
        mk(
            "p_eq_nan_3",
            Predicate::Equals(ScalarValue::Double(quiet_nan_alt)),
        ),
        // Non-zero finite float stays distinct from 0.0 and NaN
        mk("p_eq_dbl_1", Predicate::Equals(ScalarValue::Double(1.5))),
        mk("p_eq_dbl_2", Predicate::Equals(ScalarValue::Double(1.5))),
        // Regex dedup
        mk("p_re_1", Predicate::Regex("^foo.*bar$".into())),
        mk("p_re_2", Predicate::Regex("^foo.*bar$".into())),
        // Range operators (Int and Double, including -0.0/+0.0 and NaN canonicalization)
        mk("p_gt_int_1", Predicate::Gt(NumericValue::Int(10))),
        mk("p_gt_int_2", Predicate::Gt(NumericValue::Int(10))),
        mk("p_gt_zero_p", Predicate::Gt(NumericValue::Double(0.0))),
        mk("p_gt_zero_n", Predicate::Gt(NumericValue::Double(-0.0))),
        mk("p_gt_nan_1", Predicate::Gt(NumericValue::Double(f64::NAN))),
        mk(
            "p_gt_nan_2",
            Predicate::Gt(NumericValue::Double(quiet_nan_alt)),
        ),
        mk("p_gte_1", Predicate::Gte(NumericValue::Double(2.5))),
        mk("p_gte_2", Predicate::Gte(NumericValue::Double(2.5))),
        mk("p_lt_1", Predicate::Lt(NumericValue::Int(5))),
        mk("p_lt_2", Predicate::Lt(NumericValue::Int(5))),
        mk("p_lte_1", Predicate::Lte(NumericValue::Double(-3.5))),
        mk("p_lte_2", Predicate::Lte(NumericValue::Double(-3.5))),
        // Contains dedup
        mk(
            "p_con_str_1",
            Predicate::Contains(ScalarValue::String("sub".into())),
        ),
        mk(
            "p_con_str_2",
            Predicate::Contains(ScalarValue::String("sub".into())),
        ),
        mk(
            "p_con_bytes_1",
            Predicate::Contains(ScalarValue::Bytes(vec![0xaa])),
        ),
        mk(
            "p_con_bytes_2",
            Predicate::Contains(ScalarValue::Bytes(vec![0xaa])),
        ),
    ];

    let engine = compile_engine(&policies);
    // 18 unique canonical predicates:
    // Exists, Equals(Str, Int(0), Bool(true), Bool(false), Bytes, Double(0.0), Double(NaN), Double(1.5)),
    // Regex, Gt(Int(10), Double(0.0), Double(NaN)), Gte(2.5), Lt(5), Lte(-3.5),
    // Contains(Str("sub")), Contains(Bytes([0xaa]))
    assert_eq!(engine.num_predicates(), 18);

    // Distinct NaN / -0.0 float evaluation semantics remain exact
    let nan_engine = compile_engine(&[
        mk("nan_a", Predicate::Equals(ScalarValue::Double(f64::NAN))),
        mk(
            "nan_b",
            Predicate::Equals(ScalarValue::Double(quiet_nan_alt)),
        ),
    ]);
    assert_eq!(nan_engine.num_predicates(), 1);
    let batch = build_custom_test_batch(
        vec![Some("row0"), Some("row1")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![
            (0, "x", ScalarValue::Double(f64::NAN)),
            (1, "x", ScalarValue::Double(0.0)),
        ],
        vec![],
    );
    // NaN never equals NaN, so neither row is dropped
    assert_eq!(
        nan_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 1]).unwrap()
    );

    let zero_engine = compile_engine(&[
        mk("zero_p", Predicate::Equals(ScalarValue::Double(0.0))),
        mk("zero_n", Predicate::Equals(ScalarValue::Double(-0.0))),
    ]);
    assert_eq!(zero_engine.num_predicates(), 1);
    // Row 1 has x == 0.0 -> dropped; row 0 has NaN -> kept
    assert_eq!(
        zero_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0]).unwrap()
    );
}

#[test]
fn test_transforms_preserve_cbor_map_and_slice_attributes_and_body() {
    let map_val = ciborium::Value::Map(vec![(
        ciborium::Value::Text("nested".into()),
        ciborium::Value::Text("keep_me".into()),
    )]);
    let slice_val = ciborium::Value::Array(vec![
        ciborium::Value::Text("tag_a".into()),
        ciborium::Value::Integer(42.into()),
    ]);
    let map_bytes = encode_cbor(&map_val);
    let slice_bytes = encode_cbor(&slice_val);

    // Build 4-row batch with CBOR Map body (row 1), CBOR Slice body (row 2), and null-ser Map body (row 3)
    let body_struct = StructArray::new(
        vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Slice as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(StringArray::from(vec![
                Some("secret=abc"),
                None,
                None,
                None,
            ])),
            Arc::new(BinaryArray::from_opt_vec(vec![
                None,
                Some(map_bytes.as_slice()),
                Some(slice_bytes.as_slice()),
                None,
            ])),
        ],
        None,
    );
    let res_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, false))].into(),
        vec![Arc::new(UInt16Array::from(vec![0u16; 4]))],
        None,
    );
    let scope_struct = StructArray::new(
        vec![Arc::new(Field::new(consts::ID, DataType::UInt16, false))].into(),
        vec![Arc::new(UInt16Array::from(vec![0u16; 4]))],
        None,
    );
    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false),
        Field::new(consts::BODY, body_struct.data_type().clone(), true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(consts::RESOURCE, res_struct.data_type().clone(), true),
        Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3])),
            Arc::new(body_struct),
            Arc::new(StringArray::from(vec!["INFO"; 4])),
            Arc::new(res_struct),
            Arc::new(scope_struct),
        ],
    )
    .unwrap();

    let log_attrs = build_attr_batch(
        vec![
            (0, "flat", ScalarValue::String("orig".into())),
            (0, "src_blocked", ScalarValue::String("stay".into())),
        ],
        vec![
            (0, "cbor_map", map_val.clone()),
            (0, "cbor_slice", slice_val.clone()),
            (0, "cbor_rename_src", map_val.clone()),
            (0, "cbor_bad_rename", map_val.clone()),
            (0, "cbor_to_body", slice_val.clone()),
            (0, "cbor_remove_me", map_val.clone()),
            (0, "cbor_redact_whole", slice_val.clone()),
            (0, "cbor_redact_regex", map_val.clone()),
        ],
    );
    let res_attrs = build_attr_batch(
        vec![(0, "r_flat", ScalarValue::String("r_orig".into()))],
        vec![(0, "r_cbor_map", map_val.clone())],
    );
    let scope_attrs = build_attr_batch(
        vec![(0, "s_flat", ScalarValue::String("s_orig".into()))],
        vec![(0, "s_cbor_slice", slice_val.clone())],
    );

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, log_attrs).unwrap();
    batch
        .set(ArrowPayloadType::ResourceAttrs, res_attrs)
        .unwrap();
    batch
        .set(ArrowPayloadType::ScopeAttrs, scope_attrs)
        .unwrap();

    let tx = CompiledTransformPolicy::compile(
        "preserve_cbor".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![
                // Blocked by existing CBOR destination when upsert=false
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("src_blocked")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("cbor_map")),
                    upsert: false,
                },
                // Move CBOR map to another attribute key
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("cbor_rename_src")),
                    to: LogTargetSelector::LogAttribute(AttributePath::single("cbor_rename_dst")),
                    upsert: false,
                },
                // Incompatible rename of CBOR attribute into SeverityText MUST NOT drop source
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("cbor_bad_rename")),
                    to: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    upsert: true,
                },
                // Move CBOR slice into Body on row 0 (overwriting row 0 string after upsert=true)
                LogRename {
                    from: LogTargetSelector::LogAttribute(AttributePath::single("cbor_to_body")),
                    to: LogTargetSelector::RecordField(LogRecordField::Body),
                    upsert: true,
                },
            ],
            add: vec![
                // Blocked by existing CBOR attribute when upsert=false
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("cbor_slice")),
                    value: ScalarValue::String("should_not_overwrite".into()),
                    upsert: false,
                },
                // Mutate sibling flat attrs in all 3 attribute tables
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("flat_new")),
                    value: ScalarValue::Int(7),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::ResourceAttribute(AttributePath::single("r_new")),
                    value: ScalarValue::String("added".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::ScopeAttribute(AttributePath::single("s_new")),
                    value: ScalarValue::Bool(true),
                    upsert: true,
                },
            ],
            remove: vec![LogRemove {
                target: LogTargetSelector::LogAttribute(AttributePath::single("cbor_remove_me")),
            }],
            redact: vec![
                // Whole-value mask replaces CBOR with replacement string
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single(
                        "cbor_redact_whole",
                    )),
                    replacement: None,
                    regex: None,
                },
                // Regex redact on CBOR is a no-op (non-string value)
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single(
                        "cbor_redact_regex",
                    )),
                    replacement: Some("MASK".into()),
                    regex: Some("keep_me".into()),
                },
            ],
        },
    )
    .unwrap();

    let bm0 = RoaringBitmap::from_sorted_iter([0]).unwrap();
    let hits = apply_spec_transforms_masked(&mut batch, &[tx], &[bm0]).unwrap();
    // Hits: cbor_rename_src->dst (1), cbor_to_body->body (1), flat_new (1), r_new (1), s_new (1),
    // cbor_remove_me (1), cbor_redact_whole (1) = 7
    assert_eq!(hits, 7);

    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();

    // Untouched CBOR body rows (1 = Map, 2 = Slice) preserved intact; row 0 received cbor_to_body Slice
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("nested")),
            1
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 2),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Str(Cow::Owned("tag_a".into())),
            ExtractedValue::Int(42),
        ]))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Str(Cow::Owned("tag_a".into())),
            ExtractedValue::Int(42),
        ]))
    );
    // Row 3 had Map type with null ATTRIBUTE_SER -> remains None
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 3),
        None
    );

    // Verify raw ATTRIBUTE_SER bytes on logs.body match byte-for-byte
    let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
    let body_sa = logs_rb
        .column_by_name(consts::BODY)
        .unwrap()
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    let body_ser = body_sa.column_by_name(consts::ATTRIBUTE_SER).unwrap();
    assert_eq!(
        extract_bytes_from_array(body_ser, 0),
        Some(slice_bytes.as_slice())
    );
    assert_eq!(
        extract_bytes_from_array(body_ser, 1),
        Some(map_bytes.as_slice())
    );
    assert_eq!(
        extract_bytes_from_array(body_ser, 2),
        Some(slice_bytes.as_slice())
    );
    assert_eq!(extract_bytes_from_array(body_ser, 3), None);

    // LogAttrs CBOR checks
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["cbor_map", "nested"])),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("src_blocked")),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("stay")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("cbor_slice")),
            0,
        ),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Str(Cow::Owned("tag_a".into())),
            ExtractedValue::Int(42),
        ]))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("cbor_rename_src")),
            0,
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["cbor_rename_dst", "nested"])),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );
    // Incompatible rename into SeverityText preserved source CBOR attribute and left SeverityText untouched
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["cbor_bad_rename", "nested"])),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("INFO")))
    );
    // Removed CBOR attr is gone; whole-value redacted CBOR attr became "[REDACTED]"
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("cbor_remove_me")),
            0,
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("cbor_redact_whole")),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]")))
    );
    // Regex redact left CBOR map untouched
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["cbor_redact_regex", "nested"])),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );

    // ResourceAttrs & ScopeAttrs CBOR preserved byte-for-byte alongside newly added flat attrs
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::nested(["r_cbor_map", "nested"])),
            0,
        ),
        Some(ExtractedValue::Str(Cow::Owned("keep_me".into())))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::single("s_cbor_slice")),
            0,
        ),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Str(Cow::Owned("tag_a".into())),
            ExtractedValue::Int(42),
        ]))
    );
    let out_log_attrs = batch.get(ArrowPayloadType::LogAttrs).unwrap();
    let key_col = out_log_attrs.column_by_name(consts::ATTRIBUTE_KEY).unwrap();
    let ser_col = out_log_attrs.column_by_name(consts::ATTRIBUTE_SER).unwrap();
    let mut found_map_raw = false;
    for r in 0..out_log_attrs.num_rows() {
        if extract_str_from_array(key_col, r) == Some("cbor_map") {
            assert_eq!(
                extract_bytes_from_array(ser_col, r),
                Some(map_bytes.as_slice())
            );
            found_map_raw = true;
        }
    }
    assert!(found_map_raw);
}

fn encode_otlp_logs(
    logs: &otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData,
) -> OtapArrowRecords {
    otel_arrow_dfe_pdata::encode::encode_logs_otap_batch(logs).expect("encode_logs_otap_batch")
}

fn decode_otlp_logs(
    records: OtapArrowRecords,
) -> otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData {
    use otel_arrow_dfe_pdata::TryIntoWithOptions;
    use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;

    let bytes: OtlpProtoBytes = records.try_into_with_default().expect("OTAP to OTLP bytes");
    match OtlpProtoMessage::try_from(bytes).expect("OTLP bytes to message") {
        OtlpProtoMessage::Logs(logs) => logs,
        other => panic!("expected LogsData, got {other:?}"),
    }
}

#[test]
fn test_encode_attrs_table_sets_plain_encoding_for_otlp_export() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    // 3 logs each with an existing attribute -> parent IDs [0, 1, 2].
    // Without `.with_plain_encoding()` on `parent_id`, OTLP export delta-decodes
    // `[0, 1, 2]` into `[0, 1, 3]` and detaches the third log's attributes.
    let logs_data = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord::build()
                        .body(AnyValue::new_string("log-0"))
                        .attributes(vec![KeyValue::new("idx", AnyValue::new_string("0"))])
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("log-1"))
                        .attributes(vec![KeyValue::new("idx", AnyValue::new_string("1"))])
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("log-2"))
                        .attributes(vec![KeyValue::new("idx", AnyValue::new_string("2"))])
                        .finish(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let batch = encode_otlp_logs(&logs_data);
    let engine = compile_engine(&[Policy::log_transform(
        "tag_all",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("env")),
                value: ScalarValue::String("prod".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let out = engine.evaluate(batch).unwrap();
    let out_records = out.records.expect("logs kept");
    let exported = decode_otlp_logs(out_records);
    let records = &exported.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), 3);
    for (i, lr) in records.iter().enumerate() {
        let idx_val = lr
            .attributes
            .iter()
            .find(|kv| kv.key == "idx")
            .and_then(|kv| kv.value.as_ref());
        assert_eq!(
            idx_val,
            Some(&AnyValue::new_string(i.to_string())),
            "log {i} lost its idx attribute after OTLP export"
        );
        let env_val = lr
            .attributes
            .iter()
            .find(|kv| kv.key == "env")
            .and_then(|kv| kv.value.as_ref());
        assert_eq!(
            env_val,
            Some(&AnyValue::new_string("prod")),
            "log {i} lost its added env attribute after OTLP export"
        );
    }
}

#[test]
fn test_add_log_attribute_allocates_unused_id_for_null_and_missing_id_column() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    // Case 1: First log has no attributes (pdata encodes log.id = null), second log has
    // attributes (pdata encodes log.id = 0). Adding an attribute to the first log must
    // allocate an unused ID (1) and write it to root.id rather than colliding with ID 0.
    let logs_mixed = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord::build()
                        .body(AnyValue::new_string("first-no-attrs"))
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("second-has-attrs"))
                        .attributes(vec![KeyValue::new("orig", AnyValue::new_string("keep"))])
                        .finish(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let batch_mixed = encode_otlp_logs(&logs_mixed);
    let engine_first = compile_engine(&[Policy::log_transform(
        "add_to_first",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("first-no-attrs".into())),
                negate: false,
            }],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("added")),
                value: ScalarValue::String("for-first".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let out_mixed = engine_first.evaluate(batch_mixed).unwrap();
    let exported_mixed = decode_otlp_logs(out_mixed.records.expect("logs kept"));
    let recs = &exported_mixed.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(recs.len(), 2);
    let first_rec = recs
        .iter()
        .find(|r| r.body == Some(AnyValue::new_string("first-no-attrs")))
        .expect("first-no-attrs record present");
    let second_rec = recs
        .iter()
        .find(|r| r.body == Some(AnyValue::new_string("second-has-attrs")))
        .expect("second-has-attrs record present");
    assert_eq!(
        first_rec.attributes,
        vec![KeyValue::new("added", AnyValue::new_string("for-first"))]
    );
    assert_eq!(
        second_rec.attributes,
        vec![KeyValue::new("orig", AnyValue::new_string("keep"))]
    );

    // Case 2: No log has attributes initially -> pdata omits root `id` column entirely.
    // Adding an attribute to the second log must materialize a plain-encoded `id` column.
    let logs_no_id = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord::build()
                        .body(AnyValue::new_string("untouched"))
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("target"))
                        .finish(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let batch_no_id = encode_otlp_logs(&logs_no_id);
    assert!(
        batch_no_id
            .get(ArrowPayloadType::Logs)
            .unwrap()
            .column_by_name(consts::ID)
            .is_none()
    );
    let engine_second = compile_engine(&[Policy::log_transform(
        "add_to_target",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String("target".into())),
                negate: false,
            }],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("tag")),
                value: ScalarValue::String("ok".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let out_no_id = engine_second.evaluate(batch_no_id).unwrap();
    let exported_no_id = decode_otlp_logs(out_no_id.records.expect("logs kept"));
    let recs2 = &exported_no_id.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(recs2.len(), 2);
    let untouched_rec = recs2
        .iter()
        .find(|r| r.body == Some(AnyValue::new_string("untouched")))
        .expect("untouched record present");
    let target_rec = recs2
        .iter()
        .find(|r| r.body == Some(AnyValue::new_string("target")))
        .expect("target record present");
    assert!(untouched_rec.attributes.is_empty());
    assert_eq!(
        target_rec.attributes,
        vec![KeyValue::new("tag", AnyValue::new_string("ok"))]
    );
}

#[test]
fn test_transport_optimized_batch_matches_plain_filter_and_transform() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    // 4 logs sharing the same attribute value -> transport encoding delta-encodes log IDs and
    // quasi-delta-encodes LogAttrs.parent_id. Both plain and transport-encoded batches must
    // yield identical filter & transform results.
    let drop_all_logs = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: (0..4)
                    .map(|i| {
                        LogRecord::build()
                            .body(AnyValue::new_string(format!("log-{i}")))
                            .attributes(vec![KeyValue::new("drop_me", AnyValue::new_string("yes"))])
                            .finish()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let plain_drop = encode_otlp_logs(&drop_all_logs);
    let mut encoded_drop = plain_drop.clone();
    encoded_drop
        .encode_transport_optimized()
        .expect("encode_transport_optimized");

    let drop_engine = compile_engine(&[Policy::log_filter(
        "drop_by_attr",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("drop_me")),
            predicate: Predicate::Equals(ScalarValue::String("yes".into())),
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        drop_engine.evaluate_filter(&plain_drop).unwrap(),
        RoaringBitmap::new()
    );
    assert_eq!(
        drop_engine.evaluate_filter(&encoded_drop).unwrap(),
        RoaringBitmap::new()
    );
    let out_plain_drop = drop_engine.evaluate(plain_drop).unwrap();
    let out_encoded_drop = drop_engine.evaluate(encoded_drop).unwrap();
    assert_eq!(out_plain_drop.dropped_rows, 4);
    assert_eq!(out_encoded_drop.dropped_rows, 4);
    assert!(out_encoded_drop.records.is_none());

    // Partial filter + transform round-trip comparing plain vs transport-encoded OTLP export.
    let mixed_logs = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    LogRecord::build()
                        .body(AnyValue::new_string("keep-0"))
                        .attributes(vec![
                            KeyValue::new("env", AnyValue::new_string("prod")),
                            KeyValue::new("secret", AnyValue::new_string("s0")),
                        ])
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("drop-1"))
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("debug"))])
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("keep-2"))
                        .attributes(vec![
                            KeyValue::new("env", AnyValue::new_string("prod")),
                            KeyValue::new("secret", AnyValue::new_string("s2")),
                        ])
                        .finish(),
                    LogRecord::build()
                        .body(AnyValue::new_string("drop-3"))
                        .attributes(vec![KeyValue::new("env", AnyValue::new_string("debug"))])
                        .finish(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let plain_mixed = encode_otlp_logs(&mixed_logs);
    let mut encoded_mixed = plain_mixed.clone();
    encoded_mixed
        .encode_transport_optimized()
        .expect("encode_transport_optimized");

    let mixed_engine = compile_engine(&[
        Policy::log_filter(
            "drop_debug",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("env")),
                predicate: Predicate::Equals(ScalarValue::String("debug".into())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_transform(
            "redact_secret",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("secret")),
                    replacement: None,
                    regex: None,
                }],
            },
        ),
    ]);
    let res_plain = mixed_engine.evaluate(plain_mixed).unwrap();
    let res_encoded = mixed_engine.evaluate(encoded_mixed).unwrap();
    assert_eq!(res_plain.kept_rows, 2);
    assert_eq!(res_encoded.kept_rows, 2);
    assert_eq!(res_plain.dropped_rows, 2);
    assert_eq!(res_encoded.dropped_rows, 2);

    let otlp_plain = decode_otlp_logs(res_plain.records.expect("kept"));
    let otlp_encoded = decode_otlp_logs(res_encoded.records.expect("kept"));
    let mut plain_bodies: Vec<_> = otlp_plain.resource_logs[0].scope_logs[0]
        .log_records
        .iter()
        .map(|lr| (lr.body.clone(), lr.attributes.clone()))
        .collect();
    let mut enc_bodies: Vec<_> = otlp_encoded.resource_logs[0].scope_logs[0]
        .log_records
        .iter()
        .map(|lr| (lr.body.clone(), lr.attributes.clone()))
        .collect();
    plain_bodies.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    enc_bodies.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    assert_eq!(plain_bodies, enc_bodies);
}

#[test]
fn test_empty_attribute_matches_exists_not_equals() {
    let root_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1])),
            Arc::new(StringArray::from(vec!["INFO", "INFO"])),
        ],
    )
    .unwrap();
    let attrs_rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16])),
            Arc::new(StringArray::from(vec!["present_empty"])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Empty as u8])),
        ],
    )
    .unwrap();
    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root_rb).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, attrs_rb).unwrap();

    // Top-level Exists matches row 0 (key present with Empty AnyValue), keeps row 1.
    let exists_engine = compile_engine(&[Policy::log_filter(
        "drop_exists",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("present_empty")),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        exists_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([1]).unwrap()
    );

    // Value predicates (Equals) and nested path Exists must NOT match an Empty attribute.
    let non_match_engine = compile_engine(&[
        Policy::log_filter(
            "drop_eq_empty_str",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("present_empty")),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_nested_exists",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested([
                    "present_empty",
                    "child",
                ])),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    assert_eq!(
        non_match_engine.evaluate_filter(&batch).unwrap(),
        RoaringBitmap::from_sorted_iter([0, 1]).unwrap()
    );
}
