// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};

use crate::EvaluationScratch;
use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::columnar::{
    Int64CmpOp, eval_anyval_bytes_eq_into, eval_anyval_double_pred_into, eval_f64_pred,
    eval_severity_number_columnar, eval_str_exists_bitmap,
};
use crate::model::{
    Action, AttributePath, LogFieldSelector, LogMatcher, LogRecordField, NumericValue, Policy,
    Predicate, ScalarValue, ScopeField,
};
use crate::otap_ops::ExtractedValue;

use super::helpers::{compile_engine, encode_cbor};

/// Scenario: Exhaustively reduce scalar equality and numeric range predicates (int and float
/// thresholds, including `NaN`, `+/-Inf`, `+/-0.0`, fractional values, `+/-2^53 +/- 1`, and `+/-2^63`) into
/// `Int64CmpOp` and `Int32CmpOp`, and evaluate `eval_f64_pred` across `NaN`/`-0.0`/`> 2^53`.
/// Guarantees: Integer kernel reductions match `cmp_i64_with_f64` / `eq_i64_f64` bit-for-bit on
/// every boundary value, and `matches_zero()` agrees with `eval_scalar(0)`.
#[test]
fn test_int_cmp_op_reduction_exact_parity_with_cmp_i64_with_f64() {
    let i64_candidates: &[i64] = &[
        i64::MIN,
        i64::MIN + 1,
        i64::from(i32::MIN) - 1,
        i64::from(i32::MIN),
        i64::from(i32::MIN) + 1,
        -100,
        -6,
        -5,
        -1,
        0,
        1,
        5,
        6,
        100,
        i64::from(i32::MAX) - 1,
        i64::from(i32::MAX),
        i64::from(i32::MAX) + 1,
        (1i64 << 53) - 1,
        1i64 << 53,
        (1i64 << 53) + 1,
        -(1i64 << 53) - 1,
        -(1i64 << 53),
        i64::MAX - 1024,
        i64::MAX - 1023,
        i64::MAX - 1,
        i64::MAX,
    ];
    let i32_candidates: &[i32] = &[
        i32::MIN,
        i32::MIN + 1,
        -6,
        -5,
        -1,
        0,
        1,
        5,
        6,
        i32::MAX - 1,
        i32::MAX,
    ];
    let f64_thresholds: &[f64] = &[
        f64::NAN,
        f64::NEG_INFINITY,
        f64::INFINITY,
        -1e30,
        1e30,
        -9_223_372_036_854_775_808.0_f64, // -2^63 == i64::MIN
        9_223_372_036_854_775_808.0_f64,  // +2^63 == i64::MAX + 1
        9_223_372_036_854_774_784.0_f64,  // 2^63 - 1024
        -((1i64 << 53) as f64),
        (1i64 << 53) as f64,
        f64::from(i32::MIN) - 1.0,
        f64::from(i32::MIN),
        f64::from(i32::MAX),
        f64::from(i32::MAX) + 1.0,
        -5.5,
        -5.0,
        -0.5,
        -0.0,
        0.0,
        0.5,
        5.0,
        5.5,
    ];
    let i64_thresholds: &[i64] = &[
        i64::MIN,
        i64::from(i32::MIN) - 1,
        i64::from(i32::MIN),
        -5,
        0,
        5,
        i64::from(i32::MAX),
        i64::from(i32::MAX) + 1,
        (1i64 << 53) + 1,
        i64::MAX,
    ];

    let mut preds: Vec<Predicate> = Vec::new();
    for &t in i64_thresholds {
        preds.push(Predicate::Equals(ScalarValue::Int(t)));
        preds.push(Predicate::Gt(NumericValue::Int(t)));
        preds.push(Predicate::Gte(NumericValue::Int(t)));
        preds.push(Predicate::Lt(NumericValue::Int(t)));
        preds.push(Predicate::Lte(NumericValue::Int(t)));
    }
    for &d in f64_thresholds {
        preds.push(Predicate::Equals(ScalarValue::Double(d)));
        preds.push(Predicate::Gt(NumericValue::Double(d)));
        preds.push(Predicate::Gte(NumericValue::Double(d)));
        preds.push(Predicate::Lt(NumericValue::Double(d)));
        preds.push(Predicate::Lte(NumericValue::Double(d)));
    }
    preds.push(Predicate::Exists);
    preds.push(Predicate::Equals(ScalarValue::Bool(true)));

    for pred in &preds {
        let op64 = Int64CmpOp::from_predicate(pred);
        let op32 = op64.narrow_to_i32();
        assert_eq!(op64.matches_zero(), op64.eval_scalar(0), "pred={pred:?}");
        assert_eq!(op32.matches_zero(), op32.eval_scalar(0), "pred={pred:?}");

        for &v in i64_candidates {
            let expected = match pred {
                Predicate::Equals(target) => ExtractedValue::Int(v).equals_scalar(target),
                Predicate::Gt(nv) | Predicate::Gte(nv) | Predicate::Lt(nv) | Predicate::Lte(nv) => {
                    pred.matches_ordering(nv.cmp_candidate_i64(v))
                }
                _ => false,
            };
            assert_eq!(
                op64.eval_scalar(v),
                expected,
                "i64 mismatch for pred={pred:?}, op64={op64:?}, v={v}"
            );
        }
        for &v in i32_candidates {
            assert_eq!(
                op32.eval_scalar(v),
                op64.eval_scalar(i64::from(v)),
                "i32 narrow mismatch for pred={pred:?}, op32={op32:?}, v={v}"
            );
        }
    }

    // Verify eval_f64_pred preserves IEEE-754 NaN, -0.0 == +0.0, and > 2^53 i64 precision.
    assert!(!eval_f64_pred(
        &Predicate::Equals(ScalarValue::Double(f64::NAN)),
        f64::NAN
    ));
    assert!(!eval_f64_pred(
        &Predicate::Gt(NumericValue::Double(0.0)),
        f64::NAN
    ));
    assert!(eval_f64_pred(
        &Predicate::Equals(ScalarValue::Double(0.0)),
        -0.0
    ));
    assert!(eval_f64_pred(&Predicate::Equals(ScalarValue::Int(0)), -0.0));
    let two_53 = (1i64 << 53) as f64;
    assert!(!eval_f64_pred(
        &Predicate::Equals(ScalarValue::Int((1i64 << 53) + 1)),
        two_53
    ));
    assert!(eval_f64_pred(
        &Predicate::Lt(NumericValue::Int((1i64 << 53) + 1)),
        two_53
    ));
}

/// Scenario: Evaluate columnar `SeverityNumber` (`Int32`, `Int64`, `Dict<U8, Int32>`,
/// `Dict<U16, Int64>`), `TraceId`/`SpanId` (`FixedSizeBinary` and `Dict<U8, FixedSizeBinary>`),
/// and root string `Exists` / small-`N` exact match fast paths (with and without `""` or null
/// dictionary entries and parent struct nulls).
/// Guarantees: `0` severity and all-zero trace/span IDs stay absent, float/infinity/`NaN`
/// thresholds work identically across native and dict columns, and validity-bitmap `Exists` agrees
/// with `skip_empty` semantics.
#[test]
fn test_columnar_severity_id_and_str_exists_fast_paths() {
    // 1. SeverityNumber across OTAP Int32, Dict<U8, Int32>, Dict<U16, Int32>
    // Rows: 0=0(absent), 1=null(absent), 2=5, 3=17, 4=21
    let sev_cols: Vec<ArrayRef> = vec![
        Arc::new(Int32Array::from(vec![
            Some(0),
            None,
            Some(5),
            Some(17),
            Some(21),
        ])),
        Arc::new(
            DictionaryArray::<UInt8Type>::try_new(
                UInt8Array::from(vec![Some(0), None, Some(1), Some(2), Some(3)]),
                Arc::new(Int32Array::from(vec![0, 5, 17, 21])),
            )
            .unwrap(),
        ),
        Arc::new(
            DictionaryArray::<UInt16Type>::try_new(
                UInt16Array::from(vec![Some(0), None, Some(1), Some(2), Some(3)]),
                Arc::new(Int32Array::from(vec![0, 5, 17, 21])),
            )
            .unwrap(),
        ),
    ];

    let sev_policies = vec![
        Policy::log_filter(
            "drop_lt17",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Lt(NumericValue::Int(17)),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_eq0",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Equals(ScalarValue::Int(0)),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_gte_16_5",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Gte(NumericValue::Double(16.5)),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_exists_and_lt_high",
            vec![
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                    predicate: Predicate::Exists,
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                    predicate: Predicate::Lt(NumericValue::Double(1e18)),
                    negate: false,
                },
                LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                    predicate: Predicate::Gt(NumericValue::Int(10)),
                    negate: false,
                },
            ],
            Action::Keep,
        ),
    ];
    let sev_eng = compile_engine(&sev_policies);
    let mut scratch = EvaluationScratch::new();
    for (idx, col) in sev_cols.into_iter().enumerate() {
        let root = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::SEVERITY_NUMBER,
                col.data_type().clone(),
                true,
            )])),
            vec![col],
        )
        .unwrap();
        let mut records = OtapArrowRecords::Logs(Logs::default());
        records.set(ArrowPayloadType::Logs, root).unwrap();
        // Row 2 (sev=5) is dropped by `drop_lt17`; rows 0 (sev=0) and 1 (null) are absent so NOT
        // dropped by `drop_lt17` or `drop_eq0`; rows 3 (17) and 4 (21) are kept.
        let kept = sev_eng
            .evaluate_filter(&records, &mut scratch, None)
            .unwrap()
            .into_bitmap();
        assert_eq!(
            kept,
            DenseBitset::from_indices([0, 1, 3, 4]),
            "SeverityNumber variant {idx} mismatch"
        );
    }

    // Direct Int64 / Dict<U16, Int64> defensive coverage for eval_severity_number_columnar
    let mut pool = BitsetBufferPool::default();
    let i64_sev_cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(vec![
            Some(0),
            None,
            Some(5),
            Some(17),
            Some(21),
        ])),
        Arc::new(
            DictionaryArray::<UInt16Type>::try_new(
                UInt16Array::from(vec![Some(0), None, Some(1), Some(2), Some(3)]),
                Arc::new(Int64Array::from(vec![0, 5, 17, 21])),
            )
            .unwrap(),
        ),
    ];
    for col in &i64_sev_cols {
        let mut bitmaps = vec![DenseBitset::new(); 4];
        eval_severity_number_columnar(
            col,
            Some(0),
            &[
                (1, Predicate::Lt(NumericValue::Int(17))),
                (2, Predicate::Gte(NumericValue::Double(16.5))),
                (3, Predicate::Lt(NumericValue::Double(f64::INFINITY))),
            ],
            None,
            &mut pool,
            &mut bitmaps,
        );
        assert_eq!(bitmaps[0], DenseBitset::from_indices([2, 3, 4]));
        assert_eq!(bitmaps[1], DenseBitset::from_indices([2]));
        assert_eq!(bitmaps[2], DenseBitset::from_indices([3, 4]));
        assert_eq!(bitmaps[3], DenseBitset::from_indices([2, 3, 4]));
        pool.recycle_vec(&mut bitmaps);
    }

    // 2. TraceId & SpanId with Dict<U8, FixedSizeBinary> covering Exists, small-N cmp::eq,
    //    large-N AHashMap (> 4), and hex regex.
    let tid_vals = FixedSizeBinaryArray::try_from_iter(
        [
            [0u8; 16], [0x11; 16], [0x22; 16], [0x33; 16], [0x44; 16], [0x55; 16],
        ]
        .into_iter(),
    )
    .unwrap();
    let tid_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1), Some(2), Some(5)]),
            Arc::new(tid_vals),
        )
        .unwrap(),
    );
    let sid_vals =
        FixedSizeBinaryArray::try_from_iter([[0u8; 8], [0xaa; 8], [0xbb; 8]].into_iter()).unwrap();
    let sid_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1), Some(2), Some(1)]),
            Arc::new(sid_vals),
        )
        .unwrap(),
    );
    let id_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::TRACE_ID, tid_dict.data_type().clone(), true),
            Field::new(consts::SPAN_ID, sid_dict.data_type().clone(), true),
        ])),
        vec![tid_dict, sid_dict],
    )
    .unwrap();
    let mut id_records = OtapArrowRecords::Logs(Logs::default());
    id_records.set(ArrowPayloadType::Logs, id_root).unwrap();

    // Small-N exact match + Exists (hits SIMD cmp::eq fast path)
    let id_small_eng = compile_engine(&[
        Policy::log_filter(
            "drop_has_tid",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_tid_11",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                predicate: Predicate::Equals(ScalarValue::String("11".repeat(16))),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_sid_bb",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SpanId),
                predicate: Predicate::Equals(ScalarValue::String("bb".repeat(8))),
                negate: false,
            }],
            Action::Keep,
        ),
    ]);
    // Rows 0 (all-zero) & 1 (null) not dropped; row 2 kept by tid_11; row 3 kept by sid_bb; row 4 dropped.
    assert_eq!(
        id_small_eng
            .evaluate_filter(&id_records, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 3])
    );

    // Large-N exact match (> 4 targets) + hex regex (hits non_zero.iter() + FixedBinaryCol path)
    let mut large_id_policies: Vec<Policy> = (1u8..=5)
        .map(|b| {
            Policy::log_filter(
                format!("drop_tid_{b}"),
                vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::TraceId),
                    predicate: Predicate::Equals(ScalarValue::String(
                        format!("{:02x}", b * 0x11).repeat(16),
                    )),
                    negate: false,
                }],
                Action::Drop,
            )
        })
        .collect();
    large_id_policies.push(Policy::log_filter(
        "keep_sid_hex_aa",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SpanId),
            predicate: Predicate::Regex("^aaaa".into()),
            negate: false,
        }],
        Action::Keep,
    ));
    let id_large_eng = compile_engine(&large_id_policies);
    // Rows 2,3,4 match drop_tid_{1,2,5}; rows 2 and 4 kept by keep_sid_hex_aa -> row 3 dropped.
    assert_eq!(
        id_large_eng
            .evaluate_filter(&id_records, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2, 4])
    );

    // 3. Root String Exists fast paths:
    //    - Dict without ""/nulls (O(N/64) validity bitmap)
    //    - Dict with "" and null dict value (cmp::neq(""))
    //    - Scope.Name with parent StructArray nulls + small-N cmp::eq
    //    - Plain StringArray Body with skip_empty=false ("" counts as existing)
    let sev_clean_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None, Some(1), Some(0)]),
            Arc::new(StringArray::from(vec!["INFO", "ERROR"])),
        )
        .unwrap(),
    );
    let event_dirty_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), Some(1), Some(2), None]),
            Arc::new(StringArray::from(vec![Some("login"), Some(""), None])),
        )
        .unwrap(),
    );
    let scope_name_dict: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), Some(0), Some(1), Some(0)]),
            Arc::new(StringArray::from(vec!["auth", "billing"])),
        )
        .unwrap(),
    );
    let scope_struct = StructArray::new(
        vec![Arc::new(Field::new(
            consts::NAME,
            scope_name_dict.data_type().clone(),
            true,
        ))]
        .into(),
        vec![scope_name_dict],
        Some(NullBuffer::from(vec![true, false, true, true])), // row 1 masked by parent null!
    );
    let schema_url_plain: ArrayRef = Arc::new(StringArray::from(vec![
        Some(""),
        None,
        Some("https://example.com/1"),
        Some("https://example.com/2"),
    ]));
    let str_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(
                consts::SEVERITY_TEXT,
                sev_clean_dict.data_type().clone(),
                true,
            ),
            Field::new(
                consts::EVENT_NAME,
                event_dirty_dict.data_type().clone(),
                true,
            ),
            Field::new(consts::SCOPE, scope_struct.data_type().clone(), true),
            Field::new(
                consts::SCHEMA_URL,
                schema_url_plain.data_type().clone(),
                true,
            ),
        ])),
        vec![
            sev_clean_dict,
            event_dirty_dict,
            Arc::new(scope_struct),
            schema_url_plain.clone(),
        ],
    )
    .unwrap();
    let mut str_records = OtapArrowRecords::Logs(Logs::default());
    str_records.set(ArrowPayloadType::Logs, str_root).unwrap();

    let str_eng = compile_engine(&[
        Policy::log_filter(
            "drop_sev_exists",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_event_exists",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::EventName),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_scope_billing",
            vec![LogMatcher {
                field: LogFieldSelector::ScopeField(ScopeField::Name),
                predicate: Predicate::Equals(ScalarValue::String("billing".into())),
                negate: false,
            }],
            Action::Keep,
        ),
    ]);
    // SeverityText exists on rows 0, 2, 3 (row 1 null).
    // EventName exists only on row 0 ("login"; row 1 is "", row 2 is null dict val, row 3 null key).
    // Scope.Name == "billing" on row 2. -> Row 3 dropped; rows 0, 1, 2 kept.
    assert_eq!(
        str_eng
            .evaluate_filter(&str_records, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1, 2])
    );

    // Scope.Name Exists must respect parent struct null on row 1 (even though child key is valid)
    let scope_exists_eng = compile_engine(&[Policy::log_filter(
        "drop_scope_exists",
        vec![LogMatcher {
            field: LogFieldSelector::ScopeField(ScopeField::Name),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        scope_exists_eng
            .evaluate_filter(&str_records, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([1])
    );

    // Plain SchemaUrl Exists (skip_empty=true) treats "" (row 0) and null (row 1) as absent
    let schema_url_exists_eng = compile_engine(&[Policy::log_filter(
        "drop_schema_url_exists",
        vec![LogMatcher {
            field: LogFieldSelector::ScopeField(ScopeField::SchemaUrl),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    assert_eq!(
        schema_url_exists_eng
            .evaluate_filter(&str_records, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::from_indices([0, 1])
    );

    // Direct eval_str_exists_bitmap with skip_empty=false treats "" (row 0) as existing
    let skip_false_bs = eval_str_exists_bitmap(&schema_url_plain, None, 4, None, false, &mut pool);
    assert_eq!(skip_false_bs, DenseBitset::from_indices([0, 2, 3]));
    pool.recycle(skip_false_bs);
}

/// Scenario: Evaluate columnar `Body` (`StructArray`) and selectivity-gated `LogAttrs` across
/// 0% (dict pre-filter skip), <= 20% (sparse high-selectivity), > 20% (low-selectivity SIMD),
/// multi-key dict lookup, and plain string keys across all OTAP value types (`Str`, `Int`,
/// `Double`, `Bool`, `Bytes`, `Empty`, `Slice`, nested `Map` CBOR).
/// Guarantees: Both selectivity regimes produce identical results, `default_values_optional`
/// defaults (`0`, `0.0`, `""`, `&[]`) and IEEE-754 `NaN`/`-0.0`/`> 2^53` hold everywhere, and
/// `Empty` attributes still satisfy `Exists`.
#[test]
fn test_columnar_body_and_selectivity_gated_attributes_all_types() {
    let mut scratch = EvaluationScratch::new();
    let num_logs = 80usize;

    let nested_map_hit = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("code".into()),
        ciborium::Value::Integer(503.into()),
    )]));
    let nested_map_miss = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("code".into()),
        ciborium::Value::Integer(200.into()),
    )]));
    let slice_hit = encode_cbor(&ciborium::Value::Array(vec![ciborium::Value::Text(
        "needle".into(),
    )]));

    // Build an 80-row Logs batch with mixed StructArray Body types (including defaults, NaN, -0.0).
    let mut b_types = Vec::with_capacity(num_logs);
    let mut b_strs: Vec<Option<&str>> = Vec::with_capacity(num_logs);
    let mut b_ints: Vec<Option<i64>> = Vec::with_capacity(num_logs);
    let mut b_doubles: Vec<Option<f64>> = Vec::with_capacity(num_logs);
    let mut b_bools: Vec<Option<bool>> = Vec::with_capacity(num_logs);
    let mut b_bytes: Vec<Option<&[u8]>> = Vec::with_capacity(num_logs);
    let mut b_ser: Vec<Option<&[u8]>> = Vec::with_capacity(num_logs);
    let mut b_valid = vec![true; num_logs];
    for (r, valid) in b_valid.iter_mut().enumerate() {
        match r % 10 {
            0 => {
                // Struct-null row (masked)
                *valid = false;
                b_types.push(AttributeValueType::Int as u8);
                b_strs.push(None);
                b_ints.push(Some(999));
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(None);
            }
            1 => {
                // Int with null slot -> default 0
                b_types.push(AttributeValueType::Int as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(None);
            }
            2 => {
                // Double: NaN on even tens, -0.0 on odd tens
                b_types.push(AttributeValueType::Double as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(Some(if (r / 10) % 2 == 0 { f64::NAN } else { -0.0 }));
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(None);
            }
            3 => {
                // Bool: false on even tens, null on odd tens
                b_types.push(AttributeValueType::Bool as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(if (r / 10) % 2 == 0 { Some(false) } else { None });
                b_bytes.push(None);
                b_ser.push(None);
            }
            4 => {
                // Bytes: b"abXYcd" on even tens, null (&[]) on odd tens
                b_types.push(AttributeValueType::Bytes as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(if (r / 10) % 2 == 0 {
                    Some(&b"abXYcd"[..])
                } else {
                    None
                });
                b_ser.push(None);
            }
            5 => {
                // Str: "hello" on even tens, null ("") on odd tens
                b_types.push(AttributeValueType::Str as u8);
                b_strs.push(if (r / 10) % 2 == 0 {
                    Some("hello")
                } else {
                    None
                });
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(None);
            }
            6 => {
                // CBOR Map: {code: 503} on even tens, {code: 200} on odd tens
                b_types.push(AttributeValueType::Map as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(Some(if (r / 10) % 2 == 0 {
                    nested_map_hit.as_slice()
                } else {
                    nested_map_miss.as_slice()
                }));
            }
            7 => {
                // CBOR Slice: ["needle"]
                b_types.push(AttributeValueType::Slice as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(Some(slice_hit.as_slice()));
            }
            _ => {
                // Empty
                b_types.push(AttributeValueType::Empty as u8);
                b_strs.push(None);
                b_ints.push(None);
                b_doubles.push(None);
                b_bools.push(None);
                b_bytes.push(None);
                b_ser.push(None);
            }
        }
    }

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
            Arc::new(UInt8Array::from(b_types.clone())),
            Arc::new(StringArray::from(b_strs.clone())),
            Arc::new(Int64Array::from(b_ints.clone())),
            Arc::new(Float64Array::from(b_doubles.clone())),
            Arc::new(BooleanArray::from(b_bools.clone())),
            Arc::new(BinaryArray::from_opt_vec(b_bytes.clone())),
            Arc::new(BinaryArray::from_opt_vec(b_ser.clone())),
        ],
        Some(NullBuffer::from(b_valid)),
    );
    let log_ids = UInt16Array::from_iter_values(0..num_logs as u16);
    let root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::BODY, body_struct.data_type().clone(), true),
        ])),
        vec![Arc::new(log_ids), Arc::new(body_struct)],
    )
    .unwrap();

    // Helper to build a LogAttrs table with the same 80 rows of values under controlled key
    // selectivity (`watched_rows` out of 80 have key "target", rest have "other_i").
    let build_records_with_attrs = |watched_count: usize, use_dict_keys: bool| {
        let keys_vec: Vec<String> = (0..num_logs)
            .map(|r| {
                if r < watched_count {
                    "target".to_string()
                } else {
                    format!("other_{}", r % 7)
                }
            })
            .collect();
        let key_col: ArrayRef = if use_dict_keys {
            let mut uniq: Vec<String> = vec!["target".into()];
            for i in 0..7 {
                uniq.push(format!("other_{i}"));
            }
            let dict_keys = UInt8Array::from_iter_values((0..num_logs).map(|r| {
                if r < watched_count {
                    0u8
                } else {
                    1u8 + (r % 7) as u8
                }
            }));
            Arc::new(
                DictionaryArray::<UInt8Type>::try_new(dict_keys, Arc::new(StringArray::from(uniq)))
                    .unwrap(),
            )
        } else {
            Arc::new(StringArray::from(keys_vec))
        };
        let attrs = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
                Field::new(consts::ATTRIBUTE_KEY, key_col.data_type().clone(), false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(0..num_logs as u16)),
                key_col,
                Arc::new(UInt8Array::from(b_types.clone())),
                Arc::new(StringArray::from(b_strs.clone())),
                Arc::new(Int64Array::from(b_ints.clone())),
                Arc::new(Float64Array::from(b_doubles.clone())),
                Arc::new(BooleanArray::from(b_bools.clone())),
                Arc::new(BinaryArray::from_opt_vec(b_bytes.clone())),
                Arc::new(BinaryArray::from_opt_vec(b_ser.clone())),
            ],
        )
        .unwrap();
        let mut records = OtapArrowRecords::Logs(Logs::default());
        records.set(ArrowPayloadType::Logs, root.clone()).unwrap();
        records.set(ArrowPayloadType::LogAttrs, attrs).unwrap();
        records
    };

    // 1. Body StructArray + BodyField columnar predicates
    let body_eng = compile_engine(&[
        Policy::log_filter(
            "drop_body_exists",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        // Matches default Int(0) (r%10==1) AND Double(-0.0) (r%10==2, odd tens), NOT NaN!
        Policy::log_filter(
            "keep_zero",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::Int(0)),
                negate: false,
            }],
            Action::Keep,
        ),
        // Matches Bool(false) (r%10==3, even tens), NOT null Bool!
        Policy::log_filter(
            "keep_false",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::Bool(false)),
                negate: false,
            }],
            Action::Keep,
        ),
        // Matches Bytes contains b"XY" (r%10==4, even tens) & Bytes equals &[] (odd tens)
        Policy::log_filter(
            "keep_bytes_xy",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::Bytes(b"XY".to_vec())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_bytes_empty",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::Bytes(Vec::new())),
                negate: false,
            }],
            Action::Keep,
        ),
        // Matches Str == "" (default on r%10==5, odd tens)
        Policy::log_filter(
            "keep_str_empty",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Equals(ScalarValue::String(String::new())),
                negate: false,
            }],
            Action::Keep,
        ),
        // Matches BodyField ["code"] == 503 (r%10==6, even tens)
        Policy::log_filter(
            "keep_map_503",
            vec![LogMatcher {
                field: LogFieldSelector::BodyField(AttributePath::single("code")),
                predicate: Predicate::Equals(ScalarValue::Int(503)),
                negate: false,
            }],
            Action::Keep,
        ),
        // Matches Slice contains "needle" (r%10==7)
        Policy::log_filter(
            "keep_slice",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("needle".into())),
                negate: false,
            }],
            Action::Keep,
        ),
    ]);
    let rec_body = build_records_with_attrs(0, true);
    let kept_body = body_eng
        .evaluate_filter(&rec_body, &mut scratch, None)
        .unwrap()
        .into_bitmap();
    let expected_body = DenseBitset::from_indices((0..num_logs as u32).filter(|&r| {
        let m = r % 10;
        let even_ten = (r / 10) % 2 == 0;
        match m {
            0 | 8 | 9 => true, // struct-null & Empty: not dropped by Exists
            1 | 4 | 7 => true, // Int(0), Bytes("abXYcd" or &[]), Slice("needle")
            2 => !even_ten,    // Double(-0.0) kept, NaN dropped!
            3 => true,         // Bool(false) kept; null Bool doesn't match Exists so not dropped!
            5 => !even_ten,    // Str("") kept, Str("hello") dropped
            6 => even_ten,     // Map{code:503} kept, Map{code:200} dropped
            _ => unreachable!(),
        }
    }));
    assert_eq!(kept_body, expected_body);

    // 2. Selectivity-gated LogAttrs: compare high selectivity (10/80 = 12.5% <= 20%),
    //    low selectivity (60/80 = 75% > 20%), dict keys vs plain keys, and 0 watched keys in dict!
    let attr_policies = vec![
        Policy::log_filter(
            "drop_attr_exists",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_attr_zero",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Equals(ScalarValue::Int(0)),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_false",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Equals(ScalarValue::Bool(false)),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_bytes_xy",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Contains(ScalarValue::Bytes(b"XY".to_vec())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_bytes_empty",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Equals(ScalarValue::Bytes(Vec::new())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_str_hello",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Equals(ScalarValue::String("hello".into())),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_nested_503",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested(["target", "code"])),
                predicate: Predicate::Equals(ScalarValue::Int(503)),
                negate: false,
            }],
            Action::Keep,
        ),
        Policy::log_filter(
            "keep_attr_slice",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
                predicate: Predicate::Contains(ScalarValue::String("needle".into())),
                negate: false,
            }],
            Action::Keep,
        ),
    ];
    let attr_eng = compile_engine(&attr_policies);

    for (watched_count, use_dict) in [(10usize, true), (60, true), (10, false), (60, false)] {
        let rec = build_records_with_attrs(watched_count, use_dict);
        let kept = attr_eng
            .evaluate_filter(&rec, &mut scratch, None)
            .unwrap()
            .into_bitmap();
        let expected = DenseBitset::from_indices((0..num_logs as u32).filter(|&r| {
            if (r as usize) >= watched_count {
                return true; // No "target" attribute on this log row
            }
            let m = r % 10;
            let even_ten = (r / 10) % 2 == 0;
            match m {
                0 => false,        // Int(999) (not struct-masked in LogAttrs!) -> dropped
                1 | 4 | 7 => true, // Int(0), Bytes("abXYcd" or &[]), Slice("needle")
                2 => !even_ten,    // Double(-0.0) kept, NaN dropped
                3 => even_ten, // Bool(false) kept; null Bool still matches attr Exists -> dropped!
                5 => even_ten, // Str("hello") kept, Str("") dropped
                6 => even_ten, // Map{code:503} kept, Map{code:200} dropped
                8 | 9 => false, // Empty attribute STILL satisfies top-level Exists -> dropped!
                _ => unreachable!(),
            }
        }));
        assert_eq!(
            kept, expected,
            "LogAttrs mismatch for watched_count={watched_count}, use_dict={use_dict}"
        );
    }

    // 3. Dictionary key pre-filter skip: 0 watched keys in dictionary skips table cleanly.
    let absent_key_eng = compile_engine(&[Policy::log_filter(
        "drop_missing_key",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("never.in.dict")),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    let rec_dict = build_records_with_attrs(60, true);
    assert_eq!(
        absent_key_eng
            .evaluate_filter(&rec_dict, &mut scratch, None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::full(num_logs)
    );

    // 4. Pure Exists-only attribute fast path (both high and low selectivity)
    let pure_exists_eng = compile_engine(&[Policy::log_filter(
        "drop_pure_exists",
        vec![LogMatcher {
            field: LogFieldSelector::LogAttribute(AttributePath::single("target")),
            predicate: Predicate::Exists,
            negate: false,
        }],
        Action::Drop,
    )]);
    for watched_count in [10usize, 60] {
        let rec = build_records_with_attrs(watched_count, true);
        assert_eq!(
            pure_exists_eng
                .evaluate_filter(&rec, &mut scratch, None)
                .unwrap()
                .into_bitmap(),
            DenseBitset::from_indices(watched_count as u32..num_logs as u32)
        );
    }
}

/// Scenario: Exercise `eval_str_exists_bitmap(..., skip_empty=false)` on a dictionary containing null dictionary values, sliced `Float64Array` in `eval_anyval_double_pred_into`, `FixedSizeBinary` `ATTRIBUTE_BYTES` equality/contains, and low-selectivity columnar dictionary `ATTRIBUTE_STR` regex/contains/glob evaluation.
/// Guarantees: Null dictionary values are excluded by logical-null checks, sliced `Float64Array` predicates respect array offsets and `NaN`, `FixedSizeBinary` bytes columns match accurately, and dictionary-encoded `ATTRIBUTE_STR` columnar evaluation matches plain strings.
#[test]
fn test_columnar_null_dict_exists_sliced_f64_fixed_bytes_and_dict_str_regex() {
    let mut pool = BitsetBufferPool::default();

    // 1. eval_str_exists_bitmap(..., skip_empty=false) on dictionary with a null value entry.
    let dict_with_null_val: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), Some(1), Some(2), None]),
            Arc::new(StringArray::from(vec![Some(""), None, Some("ok")])),
        )
        .unwrap(),
    );
    let exists_keep_empty =
        eval_str_exists_bitmap(&dict_with_null_val, None, 4, None, false, &mut pool);
    // Row 0 ("") and row 2 ("ok") exist; row 1 (points to null dict value) and row 3 (null key) do not.
    assert_eq!(exists_keep_empty, DenseBitset::from_indices([0, 2]));
    pool.recycle(exists_keep_empty);

    // 2. Sliced Float64Array in eval_anyval_double_pred_into (offset = 2, len = 5).
    let base_f64 = Float64Array::from(vec![
        Some(100.0),
        Some(200.0),
        Some(1.0),
        Some(f64::NAN),
        None,
        Some(5.0),
        Some(-0.0),
    ]);
    let sliced_f64 = base_f64.slice(2, 5);
    let mask = DenseBitset::full(5);
    let mut out_gt = DenseBitset::new();
    eval_anyval_double_pred_into(
        Some(&sliced_f64),
        &Predicate::Gt(NumericValue::Double(0.5)),
        &mask,
        &mut pool,
        &mut out_gt,
    );
    // Sliced values: [1.0, NaN, null(->0.0), 5.0, -0.0] -> rows 0 and 3 > 0.5.
    assert_eq!(out_gt, DenseBitset::from_indices([0, 3]));
    pool.recycle(out_gt);

    let mut out_eq0 = DenseBitset::new();
    eval_anyval_double_pred_into(
        Some(&sliced_f64),
        &Predicate::Equals(ScalarValue::Double(0.0)),
        &mask,
        &mut pool,
        &mut out_eq0,
    );
    // Null slot defaults to 0.0 (row 2) and -0.0 == 0.0 (row 4).
    assert_eq!(out_eq0, DenseBitset::from_indices([2, 4]));
    pool.recycle(out_eq0);
    pool.recycle(mask);

    // 3. Direct FixedSizeBinary(4) evaluation (matching width + null->empty default branch).
    let fsb_direct = FixedSizeBinaryArray::try_from_sparse_iter_with_size(
        [
            Some([0xde, 0xad, 0xbe, 0xef]),
            Some([0x01, 0x02, 0x03, 0x04]),
            None,
            Some([0xde, 0xad, 0xbe, 0xef]),
        ]
        .into_iter(),
        4,
    )
    .unwrap();
    let fsb_mask = DenseBitset::full(4);
    let mut out_fsb_eq = DenseBitset::new();
    eval_anyval_bytes_eq_into(
        Some(&fsb_direct),
        &[0xde, 0xad, 0xbe, 0xef],
        &fsb_mask,
        &mut pool,
        &mut out_fsb_eq,
    );
    assert_eq!(out_fsb_eq, DenseBitset::from_indices([0, 3]));
    pool.recycle(out_fsb_eq);

    let mut out_fsb_empty = DenseBitset::new();
    eval_anyval_bytes_eq_into(
        Some(&fsb_direct),
        &[],
        &fsb_mask,
        &mut pool,
        &mut out_fsb_empty,
    );
    assert_eq!(out_fsb_empty, DenseBitset::from_indices([2]));
    pool.recycle(out_fsb_empty);
    pool.recycle(fsb_mask);

    // 4. Dictionary<UInt16, Binary> ATTRIBUTE_BYTES + low-selectivity dictionary ATTRIBUTE_STR regex/glob.
    let n = 80usize;
    let log_ids = UInt16Array::from_iter_values(0..n as u16);
    let root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        ])),
        vec![Arc::new(log_ids)],
    )
    .unwrap();

    let bytes_dict = DictionaryArray::<UInt16Type>::try_new(
        UInt16Array::from_iter((0..n).map(|r| if r % 4 == 0 { Some(0u16) } else { Some(1u16) })),
        Arc::new(BinaryArray::from_vec(vec![
            &[0xde, 0xad, 0xbe, 0xef][..],
            &[0x01, 0x02, 0x03, 0x04][..],
        ])),
    )
    .unwrap();
    let str_dict = DictionaryArray::<UInt16Type>::try_new(
        UInt16Array::from_iter((0..n).map(|r| {
            if r % 4 == 1 {
                Some(0u16) // "err=timeout"
            } else if r % 4 == 2 {
                Some(1u16) // "ok"
            } else {
                None // null -> ""
            }
        })),
        Arc::new(StringArray::from(vec!["err=timeout", "ok"])),
    )
    .unwrap();
    let types: Vec<u8> = (0..n)
        .map(|r| {
            if r % 4 == 0 {
                AttributeValueType::Bytes as u8
            } else {
                AttributeValueType::Str as u8
            }
        })
        .collect();
    let attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, str_dict.data_type().clone(), true),
            Field::new(
                consts::ATTRIBUTE_BYTES,
                bytes_dict.data_type().clone(),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from_iter_values(0..n as u16)),
            Arc::new(StringArray::from(vec!["k"; n])),
            Arc::new(UInt8Array::from(types)),
            Arc::new(str_dict),
            Arc::new(bytes_dict),
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, attrs).unwrap();

    let eng = compile_engine(&[
        Policy::log_filter(
            "drop_bytes_eq",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("k")),
                predicate: Predicate::Equals(ScalarValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_dict_str_regex",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("k")),
                predicate: Predicate::Regex(r"^err=[a-z]+$".into()),
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "drop_dict_str_glob_empty",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("k")),
                predicate: Predicate::Glob(String::new()),
                negate: false,
            }],
            Action::Drop,
        ),
    ]);
    let kept = eng
        .evaluate_filter(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .into_bitmap();
    // r%4==0 dropped by dict Binary equals; r%4==1 dropped by dict Str regex;
    // r%4==3 (null attr_str -> "") dropped by Glob(""); only r%4==2 ("ok") survives.
    let expected = DenseBitset::from_indices((0..n as u32).filter(|r| r % 4 == 2));
    assert_eq!(kept, expected);
}
