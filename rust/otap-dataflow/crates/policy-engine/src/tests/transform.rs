// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{
    Array, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray, Float64Array,
    Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array, UInt16Array,
};
use arrow::buffer::{Buffer, NullBuffer, OffsetBuffer};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::filter::IdBitmapPool;
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::{FieldExt, consts};

use crate::bitset::DenseBitset;
use crate::model::{
    Action, AttributePath, LogAdd, LogAttributeSelector, LogFieldSelector, LogMatcher,
    LogRecordField, LogRedact, LogRemove, LogRename, LogTargetSelector, LogTransformPolicy, Policy,
    PolicyError, Predicate, ScalarValue,
};
use crate::otap_ops::{
    CompiledTransformPolicy, ExtractedValue, LogsBatchView, apply_spec_transforms_masked,
    extract_bytes_from_array, extract_str_from_array, filter_otap_logs_by_bitmap,
};
use crate::{EvaluationScratch, PolicyEngine, PolicyMatchCount};

use super::helpers::{
    build_attr_batch, build_custom_test_batch, compile_engine, decode_otlp_logs, encode_cbor,
    encode_otlp_logs,
};

/// Scenario: Run two transform policies spanning disjoint stages (`b_policy` Rename + Remove and `a_policy` Add `upsert=false` + regex Redact with `$1`).
/// Guarantees: Global Stage 1 Rename executes before Stage 2 Add so `b_policy` populates `b` before `a_policy` skips its `upsert=false` Add, Stage 3 removes `temp`, and Stage 4 expands `$1` in the body.
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
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("a")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("b")),
                    upsert: false,
                }],
                add: vec![],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single("temp")),
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
    let out = engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    let out_records = out.records.expect("row kept");
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();

    let body = view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0);
    assert_eq!(
        body,
        Some(ExtractedValue::Str("login user=[MASKED:alice] ok".into())),
        "PolicyEngine failed LogRedact capture template expansion"
    );

    let attr_a = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("a")),
        0,
    );
    assert_eq!(attr_a, None, "PolicyEngine should have renamed 'a'");

    let attr_b = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("b")),
        0,
    );
    assert_eq!(
        attr_b,
        Some(ExtractedValue::Str("orig_a".into())),
        "PolicyEngine violated global Stage 1 (Rename) before Stage 2 (Add) ordering or upsert=false"
    );

    let attr_temp = view.extract_field_at_row(
        &LogFieldSelector::LogAttribute(AttributePath::single("temp")),
        0,
    );
    assert_eq!(attr_temp, None, "PolicyEngine failed Stage 3 Remove");
}

/// Scenario: Pass two competing transform policies in reverse `Policy.id` order (`z_policy` before `a_policy`) targeting the same fields in Rename, Add, and Redact plus cross-policy Add->Remove and Remove->Redact.
/// Guarantees: Within every stage, ascending `Policy.id` order (`a_policy` before `z_policy`) determines `upsert=false`/`upsert=true` and regex-redaction winners, while global stage boundaries execute Stage 2 Add before Stage 3 Remove and Stage 3 Remove before Stage 4 Redact across policies.
#[test]
fn test_transform_policy_id_tiebreaking_and_cross_stage_ordering() {
    // Policies are intentionally passed in reverse Policy.id order (`z_policy` before `a_policy`)
    // so deleting `validated_transforms.sort_by` causes every same-stage tie-break assertion to fail.
    let batch = build_custom_test_batch(
        vec![Some("secret=abc")],
        vec![9],
        vec!["INFO"],
        vec![[1u8; 16]],
        vec![[1u8; 8]],
        vec![
            (0, "src_a", ScalarValue::String("from_a".into())),
            (0, "src_z", ScalarValue::String("from_z".into())),
            (
                0,
                "gone_before_redact",
                ScalarValue::String("pii=123".into()),
            ),
        ],
        vec![],
    );

    let policies = vec![
        Policy::log_transform(
            "z_policy",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("src_z")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("shared_rename")),
                    upsert: false,
                }],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "shared_add",
                        )),
                        value: ScalarValue::String("from_z".into()),
                        upsert: false,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "shared_upsert",
                        )),
                        value: ScalarValue::String("from_z".into()),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single("ephemeral")),
                        value: ScalarValue::String("temp".into()),
                        upsert: true,
                    },
                ],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "gone_before_redact",
                    )),
                }],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some("Z:$1".into()),
                    regex: Some("secret=([a-z]+)".into()),
                }],
            },
        ),
        Policy::log_transform(
            "a_policy",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("src_a")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("shared_rename")),
                    upsert: false,
                }],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "shared_add",
                        )),
                        value: ScalarValue::String("from_a".into()),
                        upsert: false,
                    },
                    LogAdd {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "shared_upsert",
                        )),
                        value: ScalarValue::String("from_a".into()),
                        upsert: true,
                    },
                ],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single("ephemeral")),
                }],
                redact: vec![
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::Body),
                        replacement: Some("A:$1".into()),
                        regex: Some("secret=([a-z]+)".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "gone_before_redact",
                        )),
                        replacement: None,
                        regex: None,
                    },
                ],
            },
        ),
    ];

    let engine = compile_engine(&policies);
    let mut stats = engine.new_match_stats();
    let out = engine
        .evaluate(&batch, &mut EvaluationScratch::new(), Some(&mut stats))
        .unwrap();
    assert_eq!(stats.evaluated_rows(), 1);
    assert_eq!(
        stats.iter(&engine).unwrap().collect::<Vec<_>>(),
        vec![
            ("a_policy", PolicyMatchCount { hits: 1, misses: 0 }),
            ("z_policy", PolicyMatchCount { hits: 1, misses: 0 }),
        ]
    );

    let out_records = out.records.expect("row kept");
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();

    // Stage 1 Rename tie-break: a_policy ran before z_policy, so shared_rename == "from_a" and src_z stays.
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("shared_rename")),
            0
        ),
        Some(ExtractedValue::Str("from_a".into()))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("src_a")),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("src_z")),
            0
        ),
        Some(ExtractedValue::Str("from_z".into()))
    );

    // Stage 2 Add tie-break: a_policy wins upsert=false; z_policy overwrites upsert=true.
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("shared_add")),
            0
        ),
        Some(ExtractedValue::Str("from_a".into()))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("shared_upsert")),
            0
        ),
        Some(ExtractedValue::Str("from_z".into()))
    );

    // Cross-policy Stage 2 Add (z_policy) -> Stage 3 Remove (a_policy).
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("ephemeral")),
            0
        ),
        None
    );

    // Cross-policy Stage 3 Remove (z_policy) -> Stage 4 Redact (a_policy).
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("gone_before_redact")),
            0
        ),
        None
    );

    // Stage 4 Redact tie-break: a_policy rewrites secret=abc -> A:abc before z_policy runs.
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 0),
        Some(ExtractedValue::Str("A:abc".into()))
    );
}

/// Scenario: Filter a batch with full-keep and partial-keep bitmaps via `filter_otap_logs_by_bitmap` and compile valid and invalid `LogRedact` patterns via `CompiledTransformPolicy::compile`.
/// Guarantees: Full-keep filtering short-circuits cleanly, partial filtering retains only selected rows, valid redaction regexes compile with default or custom replacements, and invalid regexes return `PolicyError::Compile`.
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
    let keep_all = DenseBitset::from_indices([0, 1, 2]);
    let all_out = filter_otap_logs_by_bitmap(&batch, &keep_all, 3, &mut pool).unwrap();
    assert_eq!(all_out.get(ArrowPayloadType::Logs).unwrap().num_rows(), 3);

    let partial = DenseBitset::from_indices([0, 2]);
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

/// Scenario: Invoke `apply_spec_transforms_masked` with empty transforms, empty row masks, missing or 0-row `Logs` batches, missing root/attribute columns, and `LogAttrs` containing `Empty` and CBOR `Map` entries.
/// Guarantees: No-op and empty inputs return `Ok(())` without corrupting the batch, missing columns are handled without errors, and untouched `Empty` and CBOR `Map` attributes survive sibling mutations.
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
    let bm0 = DenseBitset::from_indices([0]);

    // Empty transforms
    let mut empty_batch = OtapArrowRecords::Logs(Logs::default());
    apply_spec_transforms_masked(&mut empty_batch, &[], &[]).unwrap();
    // All-empty masks
    apply_spec_transforms_masked(
        &mut empty_batch,
        std::slice::from_ref(&dummy_tx),
        &[DenseBitset::new()],
    )
    .unwrap();
    // Missing Logs batch
    apply_spec_transforms_masked(
        &mut empty_batch,
        std::slice::from_ref(&dummy_tx),
        std::slice::from_ref(&bm0),
    )
    .unwrap();
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
    apply_spec_transforms_masked(
        &mut empty_batch,
        std::slice::from_ref(&dummy_tx),
        std::slice::from_ref(&bm0),
    )
    .unwrap();

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
            ],
            remove: vec![],
            redact: vec![],
        },
    )
    .unwrap();
    apply_spec_transforms_masked(
        &mut minimal_batch,
        &[touch_missing_tx],
        std::slice::from_ref(&bm0),
    )
    .unwrap();
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

    // LogAttrs with Empty value and CBOR map (both preserved through decode/encode)
    let cbor_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("k".into()),
        ciborium::Value::Integer(1.into()),
    )]));
    let sparse_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
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
    apply_spec_transforms_masked(&mut minimal_batch, &[dummy_tx], &[bm0]).unwrap();
    let view_sparse = LogsBatchView::try_new(&minimal_batch).unwrap().unwrap();
    assert_eq!(
        view_sparse.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["map_k", "k"])),
            0,
        ),
        Some(ExtractedValue::Int(1))
    );
    let rebuilt_attrs = minimal_batch.get(ArrowPayloadType::LogAttrs).unwrap();
    let rebuilt_keys = rebuilt_attrs.column_by_name(consts::ATTRIBUTE_KEY).unwrap();
    let rebuilt_types = rebuilt_attrs
        .column_by_name(consts::ATTRIBUTE_TYPE)
        .unwrap()
        .as_any()
        .downcast_ref::<UInt8Array>()
        .unwrap();
    let mut saw_empty_k = false;
    let mut saw_map_k = false;
    for r in 0..rebuilt_attrs.num_rows() {
        match extract_str_from_array(rebuilt_keys, r) {
            Some("empty_k") => {
                assert_eq!(rebuilt_types.value(r), AttributeValueType::Empty as u8);
                saw_empty_k = true;
            }
            Some("map_k") => {
                assert_eq!(rebuilt_types.value(r), AttributeValueType::Map as u8);
                saw_map_k = true;
            }
            _ => {}
        }
    }
    assert!(saw_empty_k, "empty_k attribute must survive transform");
    assert!(saw_map_k, "map_k attribute must survive transform");

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
    let both = DenseBitset::from_indices([0, 1]);
    apply_spec_transforms_masked(&mut body_only_batch, &[redact_body], &[both]).unwrap();
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

/// Scenario: Apply Rename, Add, Remove, and Redact across every `LogTargetSelector` and all 7 `AnyValue` body variants and 5 scalar attribute types, including `upsert=false`, type mismatches, and no-op self-renames.
/// Guarantees: Every supported target and scalar type round-trips through columnar rebuilds, `ResourceAttrs`/`ScopeAttrs` remain untouched by log transforms, and blocked/no-op operations do not inflate hit counts.
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

    // All 5 scalar types in LogAttrs (ResourceAttrs and ScopeAttrs remain untouched by log transforms)
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
            (0, "drop_me", ScalarValue::String("bye".into())),
            (0, "redact_default", ScalarValue::String("secret1".into())),
            (0, "redact_custom", ScalarValue::String("secret2".into())),
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
                from: LogAttributeSelector::LogAttribute(AttributePath::single("s")),
                to: LogAttributeSelector::LogAttribute(AttributePath::single("s2")),
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
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("s")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("s")),
                    upsert: true,
                },
                // Missing source skipped
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "does_not_exist",
                    )),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("dest_x")),
                    upsert: true,
                },
                // upsert=false when destination exists -> skipped, source kept intact
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("keep_src")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("existing_dst")),
                    upsert: false,
                },
                // upsert=true overwrites destination
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("over_src")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("over_dst")),
                    upsert: true,
                },
                // Absent nested LogAttribute source & non-map BodyField source yield no change (fail-open)
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("c")),
                    upsert: true,
                },
                LogRename {
                    from: LogAttributeSelector::BodyField(AttributePath::single("a")),
                    to: LogAttributeSelector::BodyField(AttributePath::single("b")),
                    upsert: true,
                },
            ],
            add: vec![
                // Valid writes to root fields & log attributes
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
                // Type mismatches, overflows, invalid TraceId hex, and non-map BodyField writes
                // yield no change (fail-open); nested LogAttribute ["a", "b"] is created then
                // removed in Stage 3 below.
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
                    target: LogTargetSelector::BodyField(AttributePath::single("k")),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
            ],
            remove: vec![
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single("drop_me")),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "missing_rem",
                    )),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested(["a", "b"])),
                },
                LogRemove {
                    target: LogAttributeSelector::BodyField(AttributePath::single("k")),
                },
            ],
            redact: vec![
                // Regex match on Body
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some("tok=[MASK]".into()),
                    regex: Some("tok=[a-z]+".into()),
                },
                // Whole-value default "[REDACTED]" and custom replacement on LogAttribute
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single(
                        "redact_default",
                    )),
                    replacement: None,
                    regex: None,
                },
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("redact_custom")),
                    replacement: Some("HIDDEN".into()),
                    regex: None,
                },
                // Whole-value on SeverityNumber yields no change per spec
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
                LogRedact {
                    target: LogTargetSelector::BodyField(AttributePath::single("k")),
                    replacement: None,
                    regex: None,
                },
            ],
        },
    )
    .unwrap();

    // Third policy clears string record fields on rows 5 and 6 via LogRedact(replacement: "")
    // per policy.md ("Remove": use LogRedact with replacement: "" to clear a string record field).
    let tx_clear_root = CompiledTransformPolicy::compile(
        "2_clear_root".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    replacement: Some(String::new()),
                    regex: None,
                },
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                    replacement: Some(String::new()),
                    regex: None,
                },
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::EventName),
                    replacement: Some(String::new()),
                    regex: None,
                },
            ],
        },
    )
    .unwrap();

    apply_spec_transforms_masked(
        &mut batch,
        &[tx_empty_mask, tx_full, tx_clear_root],
        &[
            DenseBitset::new(),
            DenseBitset::from_indices([0]),
            DenseBitset::from_indices([5, 6]),
        ],
    )
    .unwrap();

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
            &LogFieldSelector::LogAttribute(AttributePath::single("drop_me")),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("redact_default")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("redact_custom")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("HIDDEN")))
    );
    // Row 0's TraceId is read-only (untouched), and ["a", "b"] was added in Stage 2 then removed in Stage 3
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::TraceId), 0),
        Some(ExtractedValue::Str(Cow::Owned(
            "01010101010101010101010101010101".into()
        )))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["a", "b"])),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("a")),
            0
        ),
        Some(ExtractedValue::Map(vec![]))
    );
    // ResourceAttrs and ScopeAttrs remain completely untouched
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ResourceAttribute(AttributePath::single("rs")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("res_old")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::ScopeAttribute(AttributePath::single("ss")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("scope_old")))
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
    // Row 5 had string record fields cleared by tx_clear_root (Body = Str(""), SeverityText/EventName = None)
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 5),
        Some(ExtractedValue::Str(Cow::Borrowed("")))
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

/// Scenario: Mutate sibling flat log attributes and row 0 body on a batch containing CBOR `Map` and `Slice` values in both `Logs.body` and `LogAttrs`, plus rename and `upsert=false` checks targeting CBOR attributes.
/// Guarantees: Raw CBOR `Map`/`Slice` bytes in untouched rows and sibling attributes are preserved verbatim, `upsert=false` respects existing CBOR destinations, and renaming moves CBOR attributes intact.
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
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("src_blocked")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("cbor_map")),
                    upsert: false,
                },
                // Move CBOR map to another attribute key
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "cbor_rename_src",
                    )),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "cbor_rename_dst",
                    )),
                    upsert: false,
                },
            ],
            add: vec![
                // Overwrite row 0 Body so build_body_struct_array runs and preserves rows 1..=3 CBOR bytes
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::Body),
                    value: ScalarValue::String("updated_body".into()),
                    upsert: true,
                },
                // Blocked by existing CBOR attribute when upsert=false
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("cbor_slice")),
                    value: ScalarValue::String("should_not_overwrite".into()),
                    upsert: false,
                },
                // Mutate sibling flat attr in LogAttrs
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("flat_new")),
                    value: ScalarValue::Int(7),
                    upsert: true,
                },
            ],
            remove: vec![LogRemove {
                target: LogAttributeSelector::LogAttribute(AttributePath::single("cbor_remove_me")),
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

    let bm0 = DenseBitset::from_indices([0]);
    apply_spec_transforms_masked(&mut batch, &[tx], &[bm0]).unwrap();

    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();

    // Untouched CBOR body rows (1 = Map, 2 = Slice) preserved intact; row 0 received updated_body
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
        Some(ExtractedValue::Str(Cow::Borrowed("updated_body")))
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
    assert_eq!(extract_bytes_from_array(body_ser, 0), None);
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

    // ResourceAttrs & ScopeAttrs CBOR remain completely untouched
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

/// Scenario: Transform a 3-log batch with existing attributes (`parent_id` `[0, 1, 2]`) and export the rebuilt `OtapArrowRecords` back to OTLP via `otap_to_otlp`.
/// Guarantees: Rebuilt `LogAttrs.parent_id` carries plain encoding metadata so OTLP export does not delta-decode `[0, 1, 2]` into `[0, 1, 3]` and detach the third log record's attributes.
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
    let out = engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
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

/// Scenario: Add a log attribute to a row whose `logs.id` is null (alongside another row holding `id = 0`) and to a batch where `pdata` omitted the root `id` column entirely, then export to OTLP.
/// Guarantees: The engine allocates an unused `u16` ID without colliding with existing IDs, materializes a plain-encoded `logs.id` column when absent, and preserves every log's attributes through OTLP export.
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
    let out_mixed = engine_first
        .evaluate(&batch_mixed, &mut EvaluationScratch::new(), None)
        .unwrap();
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
    let out_no_id = engine_second
        .evaluate(&batch_no_id, &mut EvaluationScratch::new(), None)
        .unwrap();
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

/// Scenario: Encode identical multi-log OTLP batches in both plain and transport-optimized (delta/quasi-delta ID) OTAP forms and run both full-drop and partial-filter + transform policies on each.
/// Guarantees: Plain and transport-optimized OTAP batches produce byte-for-byte identical filter selections and OTLP-exported transformed log records.
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
        drop_engine
            .evaluate_filter(&plain_drop, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::new()
    );
    assert_eq!(
        drop_engine
            .evaluate_filter(&encoded_drop, &mut EvaluationScratch::new(), None)
            .unwrap()
            .into_bitmap(),
        DenseBitset::new()
    );
    let out_plain_drop = drop_engine
        .evaluate(&plain_drop, &mut EvaluationScratch::new(), None)
        .unwrap();
    let out_encoded_drop = drop_engine
        .evaluate(&encoded_drop, &mut EvaluationScratch::new(), None)
        .unwrap();
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
    let res_plain = mixed_engine
        .evaluate(&plain_mixed, &mut EvaluationScratch::new(), None)
        .unwrap();
    let res_encoded = mixed_engine
        .evaluate(&encoded_mixed, &mut EvaluationScratch::new(), None)
        .unwrap();
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

/// Scenario: Run no-op `LogAttrs` transforms on a batch without `logs.id`/`LogAttrs`, delete all attributes from a batch, preserve root schema metadata, and write empty `SeverityText`/`EventName` and zero `SeverityNumber` before follow-up `upsert=false` Add and Redact.
/// Guarantees: No-op and emptied transforms never attach a 0-row `LogAttrs` table (keeping downstream `filter_otap_batch` healthy), root schema metadata is preserved, and empty/zero default writes normalize to absent (`None`).
#[test]
fn test_transform_removes_empty_log_attrs_preserves_metadata_and_normalizes_defaults() {
    // 1. Batch with schema metadata, no `logs.id`, and no `LogAttrs`: a no-op remove/redact MUST NOT
    //    attach an empty 0-row `LogAttrs` batch (which would break downstream `filter_otap_batch`).
    let mut meta = std::collections::HashMap::new();
    let _ = meta.insert("otap.custom.meta".to_owned(), "keep-me".to_owned());
    let root_no_id = RecordBatch::try_new(
        Arc::new(Schema::new_with_metadata(
            vec![
                Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
                Field::new(consts::EVENT_NAME, DataType::Utf8, true),
                Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
            ],
            meta.clone(),
        )),
        vec![
            Arc::new(StringArray::from(vec![Some("WARN"), Some("INFO")])),
            Arc::new(StringArray::from(vec![Some("evt.a"), Some("evt.b")])),
            Arc::new(Int32Array::from(vec![Some(0), Some(9)])),
        ],
    )
    .unwrap();
    let mut batch_no_id = OtapArrowRecords::Logs(Logs::default());
    batch_no_id.set(ArrowPayloadType::Logs, root_no_id).unwrap();

    // No-op LogAttrs remove/redact + Add empty SeverityText/EventName
    // followed by upsert=false Add (including SeverityNumber on row 0 which starts at 0)
    // and whole-value Redact.
    let engine = compile_engine(&[
        Policy::log_transform(
            "a_clear_defaults_and_noop_attrs",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                        value: ScalarValue::String(String::new()),
                        upsert: true,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::EventName),
                        value: ScalarValue::String(String::new()),
                        upsert: true,
                    },
                ],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single(
                        "nonexistent",
                    )),
                }],
                redact: vec![
                    LogRedact {
                        target: LogTargetSelector::LogAttribute(AttributePath::single(
                            "nonexistent",
                        )),
                        replacement: None,
                        regex: None,
                    },
                    // EventName was normalized to None when set to "", so whole-value redact must NOT write "[REDACTED]"
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::EventName),
                        replacement: None,
                        regex: None,
                    },
                ],
            },
        ),
        Policy::log_transform(
            "b_upsert_false_after_default",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![
                    // Because a_policy normalized SeverityText="" to None and row 0 had SeverityNumber=0, upsert=false succeeds on row 0
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityText),
                        value: ScalarValue::String("ERROR".into()),
                        upsert: false,
                    },
                    LogAdd {
                        target: LogTargetSelector::RecordField(LogRecordField::SeverityNumber),
                        value: ScalarValue::Int(17),
                        upsert: false,
                    },
                ],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);

    let out = engine
        .evaluate(&batch_no_id, &mut EvaluationScratch::new(), None)
        .unwrap();
    let out_records = out.records.unwrap();
    // No-op LogAttrs transform must not attach a 0-row LogAttrs table
    assert!(out_records.get(ArrowPayloadType::LogAttrs).is_none());
    // Schema metadata on Logs root must survive column rebuild
    assert_eq!(
        out_records
            .get(ArrowPayloadType::Logs)
            .unwrap()
            .schema()
            .metadata(),
        &meta
    );
    // Downstream filter_otap_batch must succeed without ColumnNotFound("id")
    let mut pool = IdBitmapPool::default();
    let filtered =
        filter_otap_logs_by_bitmap(&out_records, &DenseBitset::from_indices([0]), 2, &mut pool)
            .unwrap();
    let view = LogsBatchView::try_new(&filtered).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityText),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("ERROR")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
            0
        ),
        Some(ExtractedValue::Int(17))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::EventName), 0),
        None
    );

    // 2. Deleting every attribute removes LogAttrs entirely instead of leaving a 0-row batch
    let batch_with_one_attr = build_custom_test_batch(
        vec![Some("msg")],
        vec![9],
        vec!["INFO"],
        vec![[1u8; 16]],
        vec![[1u8; 8]],
        vec![(0, "only_attr", ScalarValue::String("v".into()))],
        vec![],
    );
    let remove_all_engine = compile_engine(&[Policy::log_transform(
        "remove_only_attr",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![LogRemove {
                target: LogAttributeSelector::LogAttribute(AttributePath::single("only_attr")),
            }],
            redact: vec![],
        },
    )]);
    let removed_out = remove_all_engine
        .evaluate(&batch_with_one_attr, &mut EvaluationScratch::new(), None)
        .unwrap();
    let removed_records = removed_out.records.unwrap();
    assert!(removed_records.get(ArrowPayloadType::LogAttrs).is_none());
    let _ = filter_otap_logs_by_bitmap(
        &removed_records,
        &DenseBitset::from_indices([0]),
        1,
        &mut pool,
    )
    .unwrap();

    // 3. Mutating LogAttrs preserves custom schema metadata while stripping SORT_COLUMNS.
    let mut attr_meta = std::collections::HashMap::new();
    let _ = attr_meta.insert("otap.attr.custom".to_owned(), "keep-attr-meta".to_owned());
    let _ = attr_meta.insert(
        consts::metadata::SORT_COLUMNS.to_owned(),
        "parent_id,key".to_owned(),
    );
    let mut batch_meta = build_custom_test_batch(
        vec![Some("msg")],
        vec![9],
        vec!["INFO"],
        vec![[1u8; 16]],
        vec![[1u8; 8]],
        vec![(0, "k1", ScalarValue::String("v1".into()))],
        vec![],
    );
    let orig_attrs = batch_meta.get(ArrowPayloadType::LogAttrs).unwrap().clone();
    let attrs_with_meta = RecordBatch::try_new(
        Arc::new(Schema::new_with_metadata(
            orig_attrs
                .schema()
                .fields()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            attr_meta,
        )),
        orig_attrs.columns().to_vec(),
    )
    .unwrap();
    batch_meta
        .set(ArrowPayloadType::LogAttrs, attrs_with_meta)
        .unwrap();
    let add_attr_eng = compile_engine(&[Policy::log_transform(
        "add_k2",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("k2")),
                value: ScalarValue::String("v2".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let meta_out = add_attr_eng
        .evaluate(&batch_meta, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let rebuilt_attrs = meta_out.get(ArrowPayloadType::LogAttrs).unwrap();
    assert_eq!(
        rebuilt_attrs
            .schema()
            .metadata()
            .get("otap.attr.custom")
            .map(String::as_str),
        Some("keep-attr-meta")
    );
    assert!(
        !rebuilt_attrs
            .schema()
            .metadata()
            .contains_key(consts::metadata::SORT_COLUMNS),
        "SORT_COLUMNS must be stripped when LogAttrs is rebuilt"
    );
}

/// Scenario: Execute nested `LogAttribute` (`path.len() > 1`) and `BodyField` (`path.len() >= 1`) transforms across all 4 stages (`Rename` covering all 4 selector pairs, `Add`, `Remove`, `Redact`), including unset `Body` initialization, non-map fail-open handling, CBOR `Null` leaves, indefinite-length CBOR maps, non-text CBOR keys, and `ciborium`/`pdata` OTLP round-trips.
/// Guarantees: Nested CBOR mutations honor `policy.md` atomicity and fail-open rules, preserve untouched siblings byte-for-byte, and emit RFC 8949 CBOR valid for both `ciborium` and `pdata` OTLP export.
#[test]
fn test_nested_cbor_transforms_log_attrs_and_body_fields() {
    use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
    use otel_arrow_dfe_pdata::otlp::attributes::cbor::proto_encode_cbor_bytes;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    // 1. OTLP-encoded batch covering structured BodyField, unset Body, scalar Body, and nested LogAttrs.
    let logs_data = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![
                    // Row 0: Map body + nested Map/Slice LogAttrs + scalar non-map attr.
                    LogRecord::build()
                        .body(AnyValue::new_kvlist(vec![
                            KeyValue::new(
                                "user",
                                AnyValue::new_kvlist(vec![
                                    KeyValue::new("secret", AnyValue::new_string("tok=abc123")),
                                    KeyValue::new("keep", AnyValue::new_string("stay")),
                                    KeyValue::new("drop_me", AnyValue::new_int(7)),
                                    KeyValue::new(
                                        "sub",
                                        AnyValue::new_kvlist(vec![KeyValue::new(
                                            "k",
                                            AnyValue::new_string("deep"),
                                        )]),
                                    ),
                                ]),
                            ),
                            KeyValue::new(
                                "conflict",
                                AnyValue::new_kvlist(vec![KeyValue::new(
                                    "existing",
                                    AnyValue::new_string("orig_dst"),
                                )]),
                            ),
                            KeyValue::new("scalar_leaf", AnyValue::new_string("not_a_map")),
                        ]))
                        .attributes(vec![
                            KeyValue::new(
                                "http",
                                AnyValue::new_kvlist(vec![
                                    KeyValue::new("auth", AnyValue::new_string("Bearer s3cr3t")),
                                    KeyValue::new("old_key", AnyValue::new_string("moved_val")),
                                    KeyValue::new(
                                        "tags",
                                        AnyValue::new_array(vec![
                                            AnyValue::new_string("a"),
                                            AnyValue::new_string("b"),
                                        ]),
                                    ),
                                    KeyValue::new("rm_me", AnyValue::new_bool(true)),
                                    KeyValue::new("num_leaf", AnyValue::new_int(42)),
                                    KeyValue::new("empty_leaf", AnyValue::default()),
                                ]),
                            ),
                            KeyValue::new("flat_scalar", AnyValue::new_string("plain")),
                            KeyValue::new("from_fail", AnyValue::new_string("must_survive")),
                        ])
                        .finish(),
                    // Row 1: Unset body -> BodyField Add initializes Body as a Map.
                    LogRecord::build()
                        .attributes(vec![KeyValue::new(
                            "src_attr",
                            AnyValue::new_string("to_body"),
                        )])
                        .finish(),
                    // Row 2: Scalar string body -> BodyField Add/Rename/Remove/Redact must fail open.
                    LogRecord::build()
                        .body(AnyValue::new_string("plain_string_body"))
                        .attributes(vec![KeyValue::new(
                            "src_attr",
                            AnyValue::new_string("survives_non_map_body"),
                        )])
                        .finish(),
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let batch = encode_otlp_logs(&logs_data);

    let engine = compile_engine(&[Policy::log_transform(
        "nested_all_stages",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![
                // LogAttribute(nested) -> LogAttribute(nested) sibling rename
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "http", "old_key",
                    ])),
                    to: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "http", "new_key",
                    ])),
                    upsert: false,
                },
                // LogAttribute(nested Slice) -> BodyField(nested)
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "http", "tags",
                    ])),
                    to: LogAttributeSelector::BodyField(AttributePath::nested([
                        "meta",
                        "copied_tags",
                    ])),
                    upsert: true,
                },
                // BodyField(nested Map subtree) -> LogAttribute(nested)
                LogRename {
                    from: LogAttributeSelector::BodyField(AttributePath::nested(["user", "sub"])),
                    to: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "extracted",
                        "sub",
                    ])),
                    upsert: true,
                },
                // BodyField(nested) -> BodyField(nested)
                LogRename {
                    from: LogAttributeSelector::BodyField(AttributePath::nested(["user", "keep"])),
                    to: LogAttributeSelector::BodyField(AttributePath::nested(["moved", "kept"])),
                    upsert: true,
                },
                // Atomic fail-open: upsert=false conflict at BodyField must NOT remove LogAttribute
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("from_fail")),
                    to: LogAttributeSelector::BodyField(AttributePath::nested([
                        "conflict", "existing",
                    ])),
                    upsert: false,
                },
                // Atomic fail-open: non-map Body on row 2 must NOT remove `src_attr`; succeeds on row 1 (unset body)
                LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("src_attr")),
                    to: LogAttributeSelector::BodyField(AttributePath::nested(["init", "from"])),
                    upsert: false,
                },
            ],
            add: vec![
                // Create missing intermediate maps + all CBOR scalar types
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "net", "peer", "port",
                    ])),
                    value: ScalarValue::Int(-42),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "net", "peer", "min_i64",
                    ])),
                    value: ScalarValue::Int(i64::MIN),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "net", "peer", "ratio",
                    ])),
                    value: ScalarValue::Double(3.5),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "net", "peer", "tls",
                    ])),
                    value: ScalarValue::Bool(true),
                    upsert: false,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "net", "peer", "raw",
                    ])),
                    value: ScalarValue::Bytes(vec![0xca, 0xfe]),
                    upsert: false,
                },
                // Fail-open when top-level attribute or BodyField intermediate is not a Map
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "flat_scalar",
                        "child",
                    ])),
                    value: ScalarValue::String("ignored".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath::nested([
                        "scalar_leaf",
                        "child",
                    ])),
                    value: ScalarValue::String("ignored".into()),
                    upsert: true,
                },
                // Existing CBOR Null leaf: upsert=false yields no change
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "http",
                        "empty_leaf",
                    ])),
                    value: ScalarValue::String("blocked".into()),
                    upsert: false,
                },
                // BodyField Add creates intermediate map on row 0 and appends into row 1's map; fails open on row 2
                LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath::nested(["env", "tier"])),
                    value: ScalarValue::String("prod".into()),
                    upsert: true,
                },
            ],
            remove: vec![
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "http", "rm_me",
                    ])),
                },
                LogRemove {
                    target: LogAttributeSelector::BodyField(AttributePath::nested([
                        "user", "drop_me",
                    ])),
                },
            ],
            redact: vec![
                // Regex capture expansion on nested BodyField
                LogRedact {
                    target: LogTargetSelector::BodyField(AttributePath::nested(["user", "secret"])),
                    replacement: Some("tok=$$MASK:${1}".into()),
                    regex: Some("tok=([a-z]+)[0-9]+".into()),
                },
                // Whole-value default mask on nested LogAttribute
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "http", "auth",
                    ])),
                    replacement: None,
                    regex: None,
                },
                // Regex redact on non-string nested Int is a no-op
                LogRedact {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "http", "num_leaf",
                    ])),
                    replacement: Some("X".into()),
                    regex: Some("42".into()),
                },
            ],
        },
    )]);

    let out = engine
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap();
    let out_records = out.records.expect("logs kept");

    // Verify all transformed values through LogsBatchView
    let view = LogsBatchView::try_new(&out_records).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "new_key"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("moved_val")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "old_key"])),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["meta", "copied_tags"])),
            0
        ),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Str(Cow::Borrowed("a")),
            ExtractedValue::Str(Cow::Borrowed("b")),
        ]))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["extracted", "sub", "k"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("deep")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["moved", "kept"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("stay")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("from_fail")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("must_survive")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["conflict", "existing"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("orig_dst")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["net", "peer", "port"])),
            0
        ),
        Some(ExtractedValue::Int(-42))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["net", "peer", "min_i64"])),
            0
        ),
        Some(ExtractedValue::Int(i64::MIN))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["net", "peer", "ratio"])),
            0
        ),
        Some(ExtractedValue::Double(3.5))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["net", "peer", "tls"])),
            0
        ),
        Some(ExtractedValue::Bool(true))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["net", "peer", "raw"])),
            0
        ),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[0xca, 0xfe])))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("flat_scalar")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("plain")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("scalar_leaf")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("not_a_map")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "empty_leaf"])),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "rm_me"])),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["user", "drop_me"])),
            0
        ),
        None
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["user", "secret"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("tok=$MASK:abc")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "auth"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("[REDACTED]")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["http", "num_leaf"])),
            0
        ),
        Some(ExtractedValue::Int(42))
    );
    // Row 1: unset body was initialized as Map and received both Rename and Add
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["init", "from"])),
            1
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("to_body")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::nested(["env", "tier"])),
            1
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("prod")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("src_attr")),
            1
        ),
        None
    );
    // Row 2: plain string body stayed intact, and failed Rename preserved `src_attr`
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::Body), 2),
        Some(ExtractedValue::Str(Cow::Borrowed("plain_string_body")))
    );
    assert_eq!(
        view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("src_attr")),
            2
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("survives_non_map_body")))
    );

    // Full OTLP protobuf round-trip (invokes pdata's `proto_encode_cbor_bytes` on every edited map!)
    let exported = decode_otlp_logs(out_records);
    let recs = &exported.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(recs.len(), 3);
    assert_eq!(
        recs[2].body,
        Some(AnyValue::new_string("plain_string_body"))
    );

    // 2. Indefinite-length CBOR map + non-text key preservation verified via ciborium & proto_encode_cbor_bytes.
    // Raw CBOR: 0xbf (indef map), 0x01 0x62 "nt" (int key 1 -> "nt"), 0x61 "a" 0x61 "x", 0xff (break)
    let indef_cbor: Vec<u8> = vec![0xbf, 0x01, 0x62, b'n', b't', 0x61, b'a', 0x61, b'x', 0xff];
    let log_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
    ]));
    let log_rb =
        RecordBatch::try_new(log_schema, vec![Arc::new(UInt16Array::from(vec![0u16]))]).unwrap();
    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ]));
    let attr_rb = RecordBatch::try_new(
        attr_schema,
        vec![
            Arc::new(UInt16Array::from(vec![0u16])),
            Arc::new(StringArray::from(vec!["indef"])),
            Arc::new(UInt8Array::from(vec![AttributeValueType::Map as u8])),
            Arc::new(BinaryArray::from(vec![indef_cbor.as_slice()])),
        ],
    )
    .unwrap();
    let mut indef_records = OtapArrowRecords::Logs(Logs::default());
    indef_records.set(ArrowPayloadType::Logs, log_rb).unwrap();
    indef_records
        .set(ArrowPayloadType::LogAttrs, attr_rb)
        .unwrap();

    let indef_engine = compile_engine(&[Policy::log_transform(
        "edit_indef",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::nested(["indef", "b"])),
                value: ScalarValue::Int(9),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let indef_out = indef_engine
        .evaluate(&indef_records, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let out_attrs = indef_out.get(ArrowPayloadType::LogAttrs).unwrap();
    let raw_ser =
        extract_bytes_from_array(out_attrs.column_by_name(consts::ATTRIBUTE_SER).unwrap(), 0)
            .unwrap();
    let decoded: ciborium::Value = ciborium::from_reader(raw_ser).expect("valid RFC 8949 CBOR");
    let entries = decoded.as_map().expect("CBOR map");
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0],
        (
            ciborium::Value::Integer(1.into()),
            ciborium::Value::Text("nt".into())
        )
    );
    assert_eq!(
        entries[1],
        (
            ciborium::Value::Text("a".into()),
            ciborium::Value::Text("x".into())
        )
    );
    assert_eq!(
        entries[2],
        (
            ciborium::Value::Text("b".into()),
            ciborium::Value::Integer(9.into())
        )
    );
    // Also verify `proto_encode_cbor_bytes` accepts a valid text-keyed CBOR map built from scratch.
    let scratch_engine = compile_engine(&[Policy::log_transform(
        "create_nested",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::nested([
                    "fresh", "inner", "k",
                ])),
                value: ScalarValue::String("v".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let fresh_out = scratch_engine
        .evaluate(&indef_records, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let fresh_attrs = fresh_out.get(ArrowPayloadType::LogAttrs).unwrap();
    let key_col = fresh_attrs.column_by_name(consts::ATTRIBUTE_KEY).unwrap();
    let ser_col = fresh_attrs.column_by_name(consts::ATTRIBUTE_SER).unwrap();
    let fresh_row = (0..fresh_attrs.num_rows())
        .find(|&r| extract_str_from_array(key_col, r) == Some("fresh"))
        .expect("fresh attr row");
    let fresh_cbor = extract_bytes_from_array(ser_col, fresh_row).unwrap();
    let mut proto_buf = ProtoBuffer::default();
    proto_encode_cbor_bytes(fresh_cbor, &mut proto_buf)
        .expect("pdata proto_encode_cbor_bytes accepts raw-slice CBOR");
    assert!(!proto_buf.as_ref().is_empty());
}

/// Scenario: Compile and defensively execute `LogAdd` and `LogRedact` targeting read-only `TraceId` and `SpanId` fields across plain and dictionary-encoded batches.
/// Guarantees: `PolicyEngine::compile` rejects all `LogAdd`/`LogRedact` policies targeting `TraceId`/`SpanId` as compile errors (`policy.md`), and defensive direct calls to `apply_spec_transforms_masked` leave `TraceId`/`SpanId` columns completely untouched.
#[test]
fn test_trace_id_and_span_id_immutable_compile_errors_and_runtime_noop() {
    let tid_a = "4bf92f3577b34da6a3ce929d0e0e4736";
    let sid_a = "00f067aa0ba902b7";
    let tid_a_bytes: [u8; 16] = hex::decode(tid_a).unwrap().try_into().unwrap();
    let sid_a_bytes: [u8; 8] = hex::decode(sid_a).unwrap().try_into().unwrap();

    // 1. PolicyEngine::compile rejects LogAdd and LogRedact on TraceId / SpanId.
    let bad_policies = vec![
        Policy::log_transform(
            "add_trace_id",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    value: ScalarValue::String(tid_a.into()),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "add_span_id",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                    value: ScalarValue::String(sid_a.into()),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "redact_trace_id",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    replacement: None,
                    regex: None,
                }],
            },
        ),
        Policy::log_transform(
            "redact_span_id",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                    replacement: Some("MASK".into()),
                    regex: Some("[0-9a-f]+".into()),
                }],
            },
        ),
    ];
    let out = PolicyEngine::compile(&bad_policies).expect("compile");
    assert_eq!(out.errors.len(), 4);
    assert!(out.engine.policy_ids().is_empty());

    // 2. Defensive direct runtime call to apply_spec_transforms_masked leaves TraceId/SpanId intact.
    let mut batch = build_custom_test_batch(
        vec![Some("r0"), Some("r1")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![tid_a_bytes, [0u8; 16]],
        vec![sid_a_bytes, [0u8; 8]],
        vec![],
        vec![],
    );
    let tx = CompiledTransformPolicy::compile(
        "defensive_noop".into(),
        &LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    value: ScalarValue::String("0123456789abcdef0123456789abcdef".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                    value: ScalarValue::String("fedcba9876543210".into()),
                    upsert: true,
                },
            ],
            remove: vec![],
            redact: vec![
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::TraceId),
                    replacement: None,
                    regex: None,
                },
                LogRedact {
                    target: LogTargetSelector::RecordField(LogRecordField::SpanId),
                    replacement: Some("MASK".into()),
                    regex: Some("[0-9a-f]+".into()),
                },
            ],
        },
    )
    .unwrap();
    apply_spec_transforms_masked(&mut batch, &[tx], &[DenseBitset::from_indices([0, 1])]).unwrap();
    let view = LogsBatchView::try_new(&batch).unwrap().unwrap();
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::TraceId), 0),
        Some(ExtractedValue::Str(Cow::Borrowed(tid_a)))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::SpanId), 0),
        Some(ExtractedValue::Str(Cow::Borrowed(sid_a)))
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::TraceId), 1),
        None
    );
    assert_eq!(
        view.extract_field_at_row(&LogFieldSelector::RecordField(LogRecordField::SpanId), 1),
        None
    );
}

/// Scenario: Run `FlatAttrsEditor` direct Arrow builders over a 200-log x 10-attr dictionary-encoded `LogAttrs` batch with untouched rows across all `AnyValue` types, duplicate keys, a null dictionary key value, chained flat renames (including CBOR maps), and a null `log.id` allocation, followed by full-table deletion.
/// Guarantees: Untouched rows stream directly without losing data or colliding with newly allocated IDs, duplicate touched keys deduplicate last-write-wins, OTLP export reconstructs all 200 logs accurately, and deleting all attributes removes `LogAttrs`.
#[test]
fn test_flat_attrs_direct_builder_large_batch_preserves_untouched_and_dedups() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};

    let num_logs = 200usize;
    // Row 199 has null log.id so adding an attribute to it must allocate an unused ID (>= 200).
    let log_ids: Vec<Option<u16>> = (0..num_logs)
        .map(|i| if i == 199 { None } else { Some(i as u16) })
        .collect();
    let sev_texts: Vec<String> = (0..num_logs)
        .map(|i| {
            if i == 199 {
                "alloc_id".into()
            } else if i % 10 == 0 {
                "target".into()
            } else {
                "untouched".into()
            }
        })
        .collect();
    let root_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
    ]));
    let root_rb = RecordBatch::try_new(
        root_schema,
        vec![
            Arc::new(UInt16Array::from(log_ids)),
            Arc::new(StringArray::from(sev_texts)),
        ],
    )
    .unwrap();

    // Build 10 attributes per log for logs 0..199 (1990 rows) + duplicate key + null pid/key rows.
    let mut map_cbor = Vec::new();
    ciborium::into_writer(
        &ciborium::Value::Map(vec![(
            ciborium::Value::Text("k".into()),
            ciborium::Value::Text("v".into()),
        )]),
        &mut map_cbor,
    )
    .unwrap();
    let mut slice_cbor = Vec::new();
    ciborium::into_writer(
        &ciborium::Value::Array(vec![ciborium::Value::Integer(7.into())]),
        &mut slice_cbor,
    )
    .unwrap();

    let key_dict_vals = StringArray::from(vec![
        Some("u_str"),
        Some("u_int"),
        Some("u_dbl"),
        Some("u_bool"),
        Some("u_bytes"),
        Some("u_slice"),
        Some("u_empty"),
        Some("old_map"),
        Some("chain_a"),
        Some("drop_key"),
        None,
    ]);
    struct DictAttrRowBuilder {
        pids: Vec<u16>,
        key_idxs: Vec<u8>,
        types: Vec<u8>,
        strs: Vec<Option<String>>,
        ints: Vec<Option<i64>>,
        dbls: Vec<Option<f64>>,
        bools: Vec<Option<bool>>,
        bytes_v: Vec<Option<Vec<u8>>>,
        sers: Vec<Option<Vec<u8>>>,
    }
    impl DictAttrRowBuilder {
        fn with_capacity(cap: usize) -> Self {
            Self {
                pids: Vec::with_capacity(cap),
                key_idxs: Vec::with_capacity(cap),
                types: Vec::with_capacity(cap),
                strs: Vec::with_capacity(cap),
                ints: Vec::with_capacity(cap),
                dbls: Vec::with_capacity(cap),
                bools: Vec::with_capacity(cap),
                bytes_v: Vec::with_capacity(cap),
                sers: Vec::with_capacity(cap),
            }
        }
        fn push_empty(&mut self, pid: u16, kidx: u8) {
            self.pids.push(pid);
            self.key_idxs.push(kidx);
            self.types.push(AttributeValueType::Empty as u8);
            self.strs.push(None);
            self.ints.push(None);
            self.dbls.push(None);
            self.bools.push(None);
            self.bytes_v.push(None);
            self.sers.push(None);
        }
        fn push_str(&mut self, pid: u16, kidx: u8, s: String) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = AttributeValueType::Str as u8;
            *self.strs.last_mut().unwrap() = Some(s);
        }
        fn push_int(&mut self, pid: u16, kidx: u8, i: i64) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = AttributeValueType::Int as u8;
            *self.ints.last_mut().unwrap() = Some(i);
        }
        fn push_double(&mut self, pid: u16, kidx: u8, d: f64) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = AttributeValueType::Double as u8;
            *self.dbls.last_mut().unwrap() = Some(d);
        }
        fn push_bool(&mut self, pid: u16, kidx: u8, b: bool) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = AttributeValueType::Bool as u8;
            *self.bools.last_mut().unwrap() = Some(b);
        }
        fn push_bytes(&mut self, pid: u16, kidx: u8, by: Vec<u8>) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = AttributeValueType::Bytes as u8;
            *self.bytes_v.last_mut().unwrap() = Some(by);
        }
        fn push_cbor(&mut self, pid: u16, kidx: u8, vt: AttributeValueType, sr: Vec<u8>) {
            self.push_empty(pid, kidx);
            *self.types.last_mut().unwrap() = vt as u8;
            *self.sers.last_mut().unwrap() = Some(sr);
        }
        fn finish(self, key_dict_vals: StringArray) -> RecordBatch {
            let key_dict: DictionaryArray<UInt8Type> =
                DictionaryArray::new(UInt8Array::from(self.key_idxs), Arc::new(key_dict_vals));
            let attrs_schema = Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
                Field::new(consts::ATTRIBUTE_KEY, key_dict.data_type().clone(), false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ]));
            RecordBatch::try_new(
                attrs_schema,
                vec![
                    Arc::new(UInt16Array::from(self.pids)),
                    Arc::new(key_dict),
                    Arc::new(UInt8Array::from(self.types)),
                    Arc::new(StringArray::from(self.strs)),
                    Arc::new(Int64Array::from(self.ints)),
                    Arc::new(Float64Array::from(self.dbls)),
                    Arc::new(BooleanArray::from(self.bools)),
                    Arc::new(BinaryArray::from_opt_vec(
                        self.bytes_v.iter().map(|o| o.as_deref()).collect(),
                    )),
                    Arc::new(BinaryArray::from_opt_vec(
                        self.sers.iter().map(|o| o.as_deref()).collect(),
                    )),
                ],
            )
            .unwrap()
        }
    }

    let mut b = DictAttrRowBuilder::with_capacity(2005);
    // Earlier duplicate row for (pid=0, "chain_a") that must lose to the later row
    b.push_str(0, 8, "stale_dup".into());
    // Row referencing null dictionary key slot (index 10) that must be cleanly skipped
    b.push_str(0, 10, "bad_key".into());
    for i in 0..199u16 {
        b.push_str(i, 0, format!("s{i}"));
        b.push_int(i, 1, i64::from(i));
        b.push_double(i, 2, f64::from(i) + 0.5);
        b.push_bool(i, 3, i % 2 == 0);
        b.push_bytes(i, 4, vec![(i & 0xff) as u8]);
        b.push_cbor(i, 5, AttributeValueType::Slice, slice_cbor.clone());
        b.push_empty(i, 6);
        b.push_cbor(i, 7, AttributeValueType::Map, map_cbor.clone());
        b.push_str(i, 8, format!("fresh_{i}"));
        b.push_str(i, 9, "to_drop".into());
    }
    let attrs_rb = b.finish(key_dict_vals);

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records.set(ArrowPayloadType::Logs, root_rb).unwrap();
    records.set(ArrowPayloadType::LogAttrs, attrs_rb).unwrap();

    let engine = compile_engine(&[
        Policy::log_transform(
            "a_target_transforms",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("target".into())),
                    negate: false,
                }],
                rename: vec![
                    // Zero-copy flat rename of CBOR Map attribute
                    LogRename {
                        from: LogAttributeSelector::LogAttribute(AttributePath::single("old_map")),
                        to: LogAttributeSelector::LogAttribute(AttributePath::single("new_map")),
                        upsert: true,
                    },
                    // First hop of chained flat rename
                    LogRename {
                        from: LogAttributeSelector::LogAttribute(AttributePath::single("chain_a")),
                        to: LogAttributeSelector::LogAttribute(AttributePath::single("chain_b")),
                        upsert: true,
                    },
                ],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("added")),
                    value: ScalarValue::String("yes".into()),
                    upsert: false,
                }],
                remove: vec![LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::single("drop_key")),
                }],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "b_chain_second_hop",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("target".into())),
                    negate: false,
                }],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("chain_b")),
                    to: LogAttributeSelector::LogAttribute(AttributePath::single("chain_c")),
                    upsert: true,
                }],
                add: vec![],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "c_alloc_for_null_id",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("alloc_id".into())),
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::single("fresh_owner")),
                    value: ScalarValue::String("row199".into()),
                    upsert: true,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);

    let out = engine
        .evaluate(&records, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();

    // Verify full OTLP export reconstructs all 200 logs accurately despite unsorted overlay append.
    let exported = decode_otlp_logs(out);
    let recs = &exported.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(recs.len(), num_logs);

    let find_attr = |lr: &otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord,
                     k: &str|
     -> Option<AnyValue> {
        lr.attributes
            .iter()
            .find(|kv| kv.key == k)
            .and_then(|kv| kv.value.clone())
    };

    for (i, lr) in recs.iter().enumerate() {
        if i == 199 {
            assert_eq!(
                lr.attributes,
                vec![KeyValue::new("fresh_owner", AnyValue::new_string("row199"))]
            );
            continue;
        }
        // Every log 0..199 retains all 7 untouched attributes across every OTAP type
        assert_eq!(
            find_attr(lr, "u_str"),
            Some(AnyValue::new_string(format!("s{i}")))
        );
        assert_eq!(find_attr(lr, "u_int"), Some(AnyValue::new_int(i as i64)));
        assert_eq!(
            find_attr(lr, "u_dbl"),
            Some(AnyValue::new_double(i as f64 + 0.5))
        );
        assert_eq!(
            find_attr(lr, "u_bool"),
            Some(AnyValue::new_bool(i % 2 == 0))
        );
        assert_eq!(
            find_attr(lr, "u_bytes"),
            Some(AnyValue::new_bytes(vec![(i & 0xff) as u8]))
        );
        assert_eq!(
            find_attr(lr, "u_slice"),
            Some(AnyValue::new_array(vec![AnyValue::new_int(7)]))
        );
        assert!(lr.attributes.iter().any(|kv| kv.key == "u_empty"));

        if i % 10 == 0 {
            assert_eq!(find_attr(lr, "old_map"), None);
            assert_eq!(
                find_attr(lr, "new_map"),
                Some(AnyValue::new_kvlist(vec![KeyValue::new(
                    "k",
                    AnyValue::new_string("v"),
                )]))
            );
            assert_eq!(find_attr(lr, "chain_a"), None);
            assert_eq!(find_attr(lr, "chain_b"), None);
            // Row 0 also verifies duplicate key dedup (last-write-wins: "fresh_0", not "stale_dup")
            assert_eq!(
                find_attr(lr, "chain_c"),
                Some(AnyValue::new_string(format!("fresh_{i}")))
            );
            assert_eq!(find_attr(lr, "added"), Some(AnyValue::new_string("yes")));
            assert_eq!(find_attr(lr, "drop_key"), None);
        } else {
            assert_eq!(
                find_attr(lr, "old_map"),
                Some(AnyValue::new_kvlist(vec![KeyValue::new(
                    "k",
                    AnyValue::new_string("v"),
                )]))
            );
            assert_eq!(
                find_attr(lr, "chain_a"),
                Some(AnyValue::new_string(format!("fresh_{i}")))
            );
            assert_eq!(
                find_attr(lr, "drop_key"),
                Some(AnyValue::new_string("to_drop"))
            );
            assert_eq!(find_attr(lr, "added"), None);
        }
    }

    // Deleting the sole remaining attribute removes the LogAttrs table completely (no 0-row batch).
    let single = build_custom_test_batch(
        vec![Some("only")],
        vec![9],
        vec!["INFO"],
        vec![[0u8; 16]],
        vec![[0u8; 8]],
        vec![(0, "sole", ScalarValue::String("bye".into()))],
        vec![],
    );
    let rm_all = compile_engine(&[Policy::log_transform(
        "rm_sole",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![LogRemove {
                target: LogAttributeSelector::LogAttribute(AttributePath::single("sole")),
            }],
            redact: vec![],
        },
    )]);
    let empty_attrs_out = rm_all
        .evaluate(&single, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    assert!(empty_attrs_out.get(ArrowPayloadType::LogAttrs).is_none());
}

/// Scenario: Exercise exact `MAX_CBOR_DEPTH` write boundaries (scalar and moved `RawCbor` subtrees), `cbor_map_remove` on non-OTLP CBOR values (tag, `u64 > i64::MAX`, `undefined`), duplicate CBOR map keys on mutation, corrupt/truncated CBOR fail-open, null-`Bool`/null-`Slice` `Body` rename initialization, empty `0xa0` CBOR map OTLP export, and lazy `AnyValBuilders` sparse schema emission.
/// Guarantees: Depth `MAX_CBOR_DEPTH - 1` round-trips cleanly while depth `MAX_CBOR_DEPTH` and over-deep subtree renames fail open (preserving sources and avoiding `log.id` allocation), non-OTLP CBOR entries remove cleanly, duplicate keys collapse, corrupt CBOR fails open without corrupting siblings, empty maps export to OTLP, and lazy builders omit unused value columns.
#[test]
fn test_cbor_depth_boundaries_subtree_rename_non_otlp_remove_and_corrupt_cbor_fail_open() {
    // 1. Exact MAX_CBOR_DEPTH write boundaries for BodyField & LogAttribute:
    //    - 63 segments + scalar (val_depth=1) => total depth 64 <= MAX_CBOR_DEPTH -> succeeds & readable!
    //    - 64 segments + scalar (val_depth=1) => total depth 65 > MAX_CBOR_DEPTH -> fails open!
    let path_63: Vec<String> = (0..63).map(|i| format!("k{i}")).collect();
    let path_64: Vec<String> = (0..64).map(|i| format!("k{i}")).collect();
    let mut attr_path_64 = vec!["top_ok".to_owned()];
    attr_path_64.extend(path_63.iter().cloned());
    let mut attr_path_65 = vec!["top_fail".to_owned()];
    attr_path_65.extend(path_64.iter().cloned());

    let no_id_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::SEVERITY_TEXT,
            DataType::Utf8,
            true,
        )])),
        vec![Arc::new(StringArray::from(vec!["row0", "row1"]))],
    )
    .unwrap();
    let mut no_id_batch = OtapArrowRecords::Logs(Logs::default());
    no_id_batch.set(ArrowPayloadType::Logs, no_id_root).unwrap();

    // Writing an over-deep LogAttribute to a batch with no `id` column must fail open WITHOUT allocating `id`!
    let overdeep_only = compile_engine(&[Policy::log_transform(
        "overdeep_fail_open",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath {
                        path: path_64.clone(),
                    }),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath { path: attr_path_65 }),
                    value: ScalarValue::Int(2),
                    upsert: true,
                },
            ],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let over_out = overdeep_only
        .evaluate(&no_id_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    assert!(
        over_out
            .get(ArrowPayloadType::Logs)
            .unwrap()
            .column_by_name(consts::ID)
            .is_none()
    );
    assert!(over_out.get(ArrowPayloadType::LogAttrs).is_none());

    // Exact boundary depth 63 (BodyField) and 64 (LogAttribute: 1 flat key + 63 CBOR levels) succeed and read back!
    let at_limit = compile_engine(&[Policy::log_transform(
        "at_limit_ok",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath {
                        path: path_63.clone(),
                    }),
                    value: ScalarValue::String("deep_body_ok".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath {
                        path: attr_path_64.clone(),
                    }),
                    value: ScalarValue::String("deep_attr_ok".into()),
                    upsert: true,
                },
            ],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let ok_out = at_limit
        .evaluate(&no_id_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let ok_view = LogsBatchView::try_new(&ok_out).unwrap().unwrap();
    assert_eq!(
        ok_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath { path: path_63 }),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("deep_body_ok")))
    );
    assert_eq!(
        ok_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath { path: attr_path_64 }),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("deep_attr_ok")))
    );

    // 2. Deep RawCbor subtree rename: subtree {"inner": {"leaf": 9}} has cbor_item_depth == 3.
    //    Moving it to BodyField depth 62 (62 + 3 = 65 > 64) MUST fail open and preserve source!
    //    Moving it to BodyField depth 61 (61 + 3 = 64 <= 64) MUST succeed and round-trip!
    let subtree = ciborium::Value::Map(vec![(
        ciborium::Value::Text("inner".into()),
        ciborium::Value::Map(vec![(
            ciborium::Value::Text("leaf".into()),
            ciborium::Value::Integer(9.into()),
        )]),
    )]);
    let subtree_batch = build_custom_test_batch(
        vec![None, None],
        vec![9, 9],
        vec!["fail", "ok"],
        vec![[0u8; 16]; 2],
        vec![[0u8; 8]; 2],
        vec![],
        vec![(0, "sub", subtree.clone()), (1, "sub", subtree)],
    );
    let path_61: Vec<String> = (0..61).map(|i| format!("p{i}")).collect();
    let path_62: Vec<String> = (0..62).map(|i| format!("p{i}")).collect();
    let rename_subtree_eng = compile_engine(&[
        Policy::log_transform(
            "rename_overdeep",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("fail".into())),
                    negate: false,
                }],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("sub")),
                    to: LogAttributeSelector::BodyField(AttributePath { path: path_62 }),
                    upsert: true,
                }],
                add: vec![],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "rename_exact_fit",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
                    predicate: Predicate::Equals(ScalarValue::String("ok".into())),
                    negate: false,
                }],
                rename: vec![LogRename {
                    from: LogAttributeSelector::LogAttribute(AttributePath::single("sub")),
                    to: LogAttributeSelector::BodyField(AttributePath {
                        path: path_61.clone(),
                    }),
                    upsert: true,
                }],
                add: vec![],
                remove: vec![],
                redact: vec![],
            },
        ),
    ]);
    let sub_out = rename_subtree_eng
        .evaluate(&subtree_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let sub_view = LogsBatchView::try_new(&sub_out).unwrap().unwrap();
    // Row 0 failed open: source `sub.inner.leaf` is preserved!
    assert_eq!(
        sub_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["sub", "inner", "leaf"])),
            0
        ),
        Some(ExtractedValue::Int(9))
    );
    // Row 1 succeeded at exact depth 64: `sub` removed and destination `p0..p60.inner.leaf` readable!
    assert_eq!(
        sub_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("sub")),
            1
        ),
        None
    );
    let mut full_leaf_path = path_61;
    full_leaf_path.push("inner".into());
    full_leaf_path.push("leaf".into());
    assert_eq!(
        sub_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath {
                path: full_leaf_path,
            }),
            1
        ),
        Some(ExtractedValue::Int(9))
    );

    // 3. Raw CBOR map edge cases:
    //    - Removing non-OTLP CBOR values (tag 1 + u64::MAX + undefined 0xf7) succeeds while keeping sibling!
    //    - Duplicate text keys ("dup") collapse cleanly on Add(upsert=true) and Remove.
    //    - Corrupt/truncated CBOR fails open without corrupting sibling rows.
    //    - Removing the last key leaves empty CBOR map `0xa0` that exports cleanly to OTLP.
    let mut raw_special_map = vec![0xa6u8]; // 6 entries
    // "tag_k": tag(1, 42) => 0xc1 0x18 0x2a
    raw_special_map.extend_from_slice(&[0x65]);
    raw_special_map.extend_from_slice(b"tag_k");
    raw_special_map.extend_from_slice(&[0xc1, 0x18, 0x2a]);
    // "big_u": u64::MAX => 0x1b + 8 * 0xff
    raw_special_map.extend_from_slice(&[0x65]);
    raw_special_map.extend_from_slice(b"big_u");
    raw_special_map.extend_from_slice(&[0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    // "undef": undefined => 0xf7
    raw_special_map.extend_from_slice(&[0x65]);
    raw_special_map.extend_from_slice(b"undef");
    raw_special_map.push(0xf7);
    // duplicate key "dup": "first" and "dup": "second"
    raw_special_map.extend_from_slice(&[0x63]);
    raw_special_map.extend_from_slice(b"dup");
    raw_special_map.extend_from_slice(&[0x65]);
    raw_special_map.extend_from_slice(b"first");
    raw_special_map.extend_from_slice(&[0x63]);
    raw_special_map.extend_from_slice(b"dup");
    raw_special_map.extend_from_slice(&[0x66]);
    raw_special_map.extend_from_slice(b"second");
    // "keep": "ok"
    raw_special_map.extend_from_slice(&[0x64]);
    raw_special_map.extend_from_slice(b"keep");
    raw_special_map.extend_from_slice(&[0x62]);
    raw_special_map.extend_from_slice(b"ok");

    let one_key_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("only".into()),
        ciborium::Value::Integer(1.into()),
    )]));
    let corrupt_cbor: &[u8] = &[0xa2, 0x61, b'a', 0x01]; // truncated 2-entry map

    let raw_root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            // Row 0: null-Bool Body, Row 1: null-Slice Body (both absent -> Rename -> BodyField initializes Map!)
            Field::new(
                consts::BODY,
                DataType::Struct(
                    vec![
                        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
                    ]
                    .into(),
                ),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2])),
            Arc::new(StructArray::new(
                vec![
                    Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
                    Arc::new(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true)),
                    Arc::new(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true)),
                ]
                .into(),
                vec![
                    Arc::new(UInt8Array::from(vec![
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Slice as u8,
                        AttributeValueType::Empty as u8,
                    ])),
                    Arc::new(BooleanArray::from(vec![None, None, None])),
                    Arc::new(BinaryArray::from(vec![None::<&[u8]>, None, None])),
                ],
                None,
            )),
        ],
    )
    .unwrap();
    let raw_attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 0, 1, 1, 2])),
            Arc::new(StringArray::from(vec![
                "m", "mv", "mv", "corrupt", "emptying",
            ])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("from_null_bool"),
                Some("from_null_slice"),
                None,
                None,
            ])),
            Arc::new(BinaryArray::from(vec![
                Some(raw_special_map.as_slice()),
                None,
                None,
                Some(corrupt_cbor),
                Some(one_key_map.as_slice()),
            ])),
        ],
    )
    .unwrap();
    let mut raw_batch = OtapArrowRecords::Logs(Logs::default());
    raw_batch.set(ArrowPayloadType::Logs, raw_root).unwrap();
    raw_batch
        .set(ArrowPayloadType::LogAttrs, raw_attrs)
        .unwrap();

    let special_eng = compile_engine(&[Policy::log_transform(
        "special_cbor_ops",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![LogRename {
                from: LogAttributeSelector::LogAttribute(AttributePath::single("mv")),
                to: LogAttributeSelector::BodyField(AttributePath::single("moved")),
                upsert: false,
            }],
            add: vec![
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested(["m", "dup"])),
                    value: ScalarValue::String("deduped".into()),
                    upsert: true,
                },
                LogAdd {
                    target: LogTargetSelector::LogAttribute(AttributePath::nested([
                        "corrupt", "x",
                    ])),
                    value: ScalarValue::Int(1),
                    upsert: true,
                },
            ],
            remove: vec![
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "m", "tag_k",
                    ])),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "m", "big_u",
                    ])),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "m", "undef",
                    ])),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "corrupt", "a",
                    ])),
                },
                LogRemove {
                    target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                        "emptying", "only",
                    ])),
                },
            ],
            redact: vec![],
        },
    )]);
    let sp_out = special_eng
        .evaluate(&raw_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let sp_view = LogsBatchView::try_new(&sp_out).unwrap().unwrap();
    // Null-Bool (row 0) and null-Slice (row 1) Body initialized as Map by Rename -> BodyField!
    assert_eq!(
        sp_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("moved")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("from_null_bool")))
    );
    assert_eq!(
        sp_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("moved")),
            1
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("from_null_slice")))
    );
    // Non-OTLP CBOR entries removed, duplicate key collapsed, sibling "keep" intact!
    assert_eq!(
        sp_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["m", "dup"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("deduped")))
    );
    assert_eq!(
        sp_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["m", "keep"])),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("ok")))
    );
    let m_val = sp_view
        .extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("m")),
            0,
        )
        .unwrap();
    assert_eq!(
        m_val,
        ExtractedValue::Map(vec![
            (
                Cow::Borrowed("dup"),
                ExtractedValue::Str(Cow::Borrowed("deduped"))
            ),
            (
                Cow::Borrowed("keep"),
                ExtractedValue::Str(Cow::Borrowed("ok"))
            ),
        ])
    );
    // Empty `0xa0` CBOR map on row 2 survives and exports cleanly to OTLP!
    assert_eq!(
        sp_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("emptying")),
            2
        ),
        Some(ExtractedValue::Map(vec![]))
    );

    // 4. Empty `0xa0` map OTLP export + lazy AnyValBuilders sparse schema check (string-only LogAttrs omits unused columns).
    let str_only_batch = build_custom_test_batch(
        vec![Some("hi")],
        vec![9],
        vec!["INFO"],
        vec![[0u8; 16]],
        vec![[0u8; 8]],
        vec![(0, "a", ScalarValue::String("1".into()))],
        vec![],
    );
    let add_str_eng = compile_engine(&[Policy::log_transform(
        "add_str_only",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![LogAdd {
                target: LogTargetSelector::LogAttribute(AttributePath::single("b")),
                value: ScalarValue::String("2".into()),
                upsert: true,
            }],
            remove: vec![],
            redact: vec![],
        },
    )]);
    let str_out = add_str_eng
        .evaluate(&str_only_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let rebuilt_schema = str_out.get(ArrowPayloadType::LogAttrs).unwrap().schema();
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_STR).is_ok());
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_INT).is_err());
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_DOUBLE).is_err());
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_BOOL).is_err());
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_BYTES).is_err());
    assert!(rebuilt_schema.index_of(consts::ATTRIBUTE_SER).is_err());

    // Empty `0xa0` CBOR map OTLP round-trip on clean batch (without the intentionally corrupt row).
    let empty_map_batch = build_custom_test_batch(
        vec![Some("hi")],
        vec![9],
        vec!["INFO"],
        vec![[0u8; 16]],
        vec![[0u8; 8]],
        vec![],
        vec![(
            0,
            "emptying",
            ciborium::Value::Map(vec![(
                ciborium::Value::Text("only".into()),
                ciborium::Value::Integer(1.into()),
            )]),
        )],
    );
    let rm_only_eng = compile_engine(&[Policy::log_transform(
        "rm_nested_only",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![LogRemove {
                target: LogAttributeSelector::LogAttribute(AttributePath::nested([
                    "emptying", "only",
                ])),
            }],
            redact: vec![],
        },
    )]);
    let empty_map_out = rm_only_eng
        .evaluate(&empty_map_batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let exported = decode_otlp_logs(empty_map_out);
    let attrs = &exported.resource_logs[0].scope_logs[0].log_records[0].attributes;
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].key, "emptying");
}

/// Scenario: Run `LogAdd` (`upsert=false`, `Body` and `BodyField`) and `LogRedact` (whole-field `regex: None` and capture-expanding `regex: Some` across multiple rows) on a batch whose dictionary-encoded `ATTRIBUTE_SER`, `ATTRIBUTE_BYTES`, and `ATTRIBUTE_STR` columns hold non-empty masked payloads behind NULL dictionary slots.
/// Guarantees: Null-dict `Map`/`Slice`/`Str` bodies are treated as absent (not redacted by whole-field redact, writable by `upsert=false` `LogAdd`, and initializable by `BodyField` `LogAdd`); untouched null-dict `Bytes` and `Map`/`Slice` rows rebuild without leaking masked buffers; and capture-expanding regex redaction accurately rewrites multiple matching string rows while leaving non-strings untouched and handling nested CBOR null/string paths.
#[test]
fn test_null_dict_values_transform_safety_and_zero_copy_redaction() {
    let masked_map = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("leaked".into()),
        ciborium::Value::Text("yes".into()),
    )]));
    let valid_map = encode_cbor(&ciborium::Value::Map(vec![
        (
            ciborium::Value::Text("p".into()),
            ciborium::Value::Text("token=xyz999".into()),
        ),
        (
            ciborium::Value::Text("n".into()),
            ciborium::Value::Integer(42.into()),
        ),
        (ciborium::Value::Text("empty".into()), ciborium::Value::Null),
    ]));
    let mut ser_buf = Vec::new();
    ser_buf.extend_from_slice(&masked_map);
    ser_buf.extend_from_slice(&valid_map);
    let ser_dict_vals = BinaryArray::try_new(
        OffsetBuffer::from_lengths([masked_map.len(), valid_map.len()]),
        Buffer::from_slice_ref(&ser_buf),
        Some(NullBuffer::from(vec![false, true])),
    )
    .unwrap();

    let bytes_dict_vals = BinaryArray::try_new(
        OffsetBuffer::from_lengths([4, 2]),
        Buffer::from_slice_ref([0xdeu8, 0xad, 0xbe, 0xef, 0x01, 0x02]),
        Some(NullBuffer::from(vec![false, true])),
    )
    .unwrap();

    let str_parts = ["SECRET_MASKED", "token=abc123", "token=def456", "plain"];
    let str_joined: String = str_parts.concat();
    let str_dict_vals = StringArray::try_new(
        OffsetBuffer::from_lengths(str_parts.iter().map(|s| s.len())),
        Buffer::from_slice_ref(str_joined.as_bytes()),
        Some(NullBuffer::from(vec![false, true, true, true])),
    )
    .unwrap();

    // 6 log rows:
    // 0: Body Map -> null-dict slot 0 (absent)
    // 1: Body Slice -> null-dict slot 0 (absent)
    // 2: Body Bytes -> null-dict slot 0 (present, logical &[], must not leak 0xdeadbeef)
    // 3: Body Str -> valid "token=abc123"
    // 4: Body Str -> valid "token=def456"
    // 5: Body Map -> valid {"p": "token=xyz999", "n": 42, "empty": null}
    let body_struct = StructArray::try_new(
        vec![
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_BYTES,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ]
        .into(),
        vec![
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Slice as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, None, None, Some(1), Some(2), None]),
                    Arc::new(str_dict_vals.clone()),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, None, Some(0), None, None, None]),
                    Arc::new(bytes_dict_vals.clone()),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![Some(0), Some(0), None, None, None, Some(1)]),
                    Arc::new(ser_dict_vals.clone()),
                )
                .unwrap(),
            ),
        ],
        None,
    )
    .unwrap();

    let root = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::BODY, body_struct.data_type().clone(), true),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1, 2, 3, 4, 5])),
            Arc::new(body_struct),
        ],
    )
    .unwrap();

    // LogAttrs:
    // row 0: log 0, key="s", Str -> null-dict slot 0 (logical "", must not leak "SECRET_MASKED")
    // row 1: log 2, key="b", Bytes -> null-dict slot 0 (logical &[], must not leak 0xdeadbeef)
    // row 2: log 5, key="m", Map -> valid {"p": "token=xyz999", "n": 42, "empty": null}
    // row 3: log 1, key="bad_map", Map -> null-dict slot 0 (must rebuild as Empty, not masked CBOR)
    let attrs = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_BYTES,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ])),
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 2, 5, 1])),
            Arc::new(StringArray::from(vec!["s", "b", "m", "bad_map"])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Map as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![Some(0u16), None, None, None]),
                    Arc::new(str_dict_vals),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, Some(0u16), None, None]),
                    Arc::new(bytes_dict_vals),
                )
                .unwrap(),
            ),
            Arc::new(
                DictionaryArray::<UInt16Type>::try_new(
                    UInt16Array::from(vec![None, None, Some(1u16), Some(0u16)]),
                    Arc::new(ser_dict_vals),
                )
                .unwrap(),
            ),
        ],
    )
    .unwrap();

    let mut batch = OtapArrowRecords::Logs(Logs::default());
    batch.set(ArrowPayloadType::Logs, root).unwrap();
    batch.set(ArrowPayloadType::LogAttrs, attrs).unwrap();

    // Case A: Whole-field redact on Body must NOT touch null-dict Map (row 0) or null-dict Slice (row 1).
    let whole_redact_eng = compile_engine(&[Policy::log_transform(
        "redact_body_whole",
        LogTransformPolicy {
            matches: vec![],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                regex: None,
                replacement: Some("[MASKED]".into()),
            }],
        },
    )]);
    let whole_out = whole_redact_eng
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let whole_view = LogsBatchView::try_new(&whole_out).unwrap().unwrap();
    let body_sel = LogFieldSelector::RecordField(LogRecordField::Body);
    assert_eq!(whole_view.extract_field_at_row(&body_sel, 0), None);
    assert_eq!(whole_view.extract_field_at_row(&body_sel, 1), None);
    for row in 2..6 {
        assert_eq!(
            whole_view.extract_field_at_row(&body_sel, row),
            Some(ExtractedValue::Str(Cow::Borrowed("[MASKED]")))
        );
    }

    // Case B: BodyField Add on null-dict Map (row 0) initializes a new Map; multi-row capture
    // redaction reuses Captures across rows 3 & 4; nested CBOR string + null redaction works;
    // untouched null-dict Bytes/Str/Map rebuild cleanly without leaking masked payloads.
    let multi_tx_eng = compile_engine(&[
        Policy::log_transform(
            "init_body_field_on_row0",
            LogTransformPolicy {
                matches: vec![LogMatcher {
                    field: LogFieldSelector::LogAttribute(AttributePath::single("s")),
                    predicate: Predicate::Exists,
                    negate: false,
                }],
                rename: vec![],
                add: vec![LogAdd {
                    target: LogTargetSelector::BodyField(AttributePath::single("init")),
                    value: ScalarValue::String("created".into()),
                    upsert: false,
                }],
                remove: vec![],
                redact: vec![],
            },
        ),
        Policy::log_transform(
            "redact_regex_and_nested",
            LogTransformPolicy {
                matches: vec![],
                rename: vec![],
                add: vec![],
                remove: vec![],
                redact: vec![
                    LogRedact {
                        target: LogTargetSelector::RecordField(LogRecordField::Body),
                        regex: Some(r"token=([a-z0-9]+)".into()),
                        replacement: Some("tok=[$1]".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::BodyField(AttributePath::single("p")),
                        regex: Some(r"token=([a-z0-9]+)".into()),
                        replacement: Some("tok=[$1]".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::BodyField(AttributePath::single("n")),
                        regex: Some(r"\d+".into()),
                        replacement: Some("NOPE".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::BodyField(AttributePath::single("empty")),
                        regex: None,
                        replacement: Some("WAS_NULL".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::LogAttribute(AttributePath::nested(["m", "p"])),
                        regex: Some(r"token=([a-z0-9]+)".into()),
                        replacement: Some("attr_tok=[$1]".into()),
                    },
                    LogRedact {
                        target: LogTargetSelector::LogAttribute(AttributePath::nested([
                            "m", "empty",
                        ])),
                        regex: None,
                        replacement: Some("ATTR_WAS_NULL".into()),
                    },
                ],
            },
        ),
    ]);
    let tx_out = multi_tx_eng
        .evaluate(&batch, &mut EvaluationScratch::new(), None)
        .unwrap()
        .records
        .unwrap();
    let tx_view = LogsBatchView::try_new(&tx_out).unwrap().unwrap();
    // Row 0 initialized a Map body {"init": "created"} instead of failing open on null-dict Map.
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("init")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("created")))
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("leaked")),
            0
        ),
        None
    );
    // Row 1 (null-dict Slice body) rebuilt as absent (null StructArray entry).
    assert_eq!(tx_view.extract_field_at_row(&body_sel, 1), None);
    // Row 2 (null-dict Bytes body) preserved as logical `&[]`, not masked `0xdeadbeef`.
    assert_eq!(
        tx_view.extract_field_at_row(&body_sel, 2),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[])))
    );
    // Rows 3 & 4 both expanded `$1` cleanly with reused `Captures`.
    assert_eq!(
        tx_view.extract_field_at_row(&body_sel, 3),
        Some(ExtractedValue::Str(Cow::Borrowed("tok=[abc123]")))
    );
    assert_eq!(
        tx_view.extract_field_at_row(&body_sel, 4),
        Some(ExtractedValue::Str(Cow::Borrowed("tok=[def456]")))
    );
    // Row 5 nested CBOR string redacted, integer untouched, CBOR null replaced by whole-field redact.
    assert_eq!(
        tx_view.extract_field_at_row(&LogFieldSelector::BodyField(AttributePath::single("p")), 5),
        Some(ExtractedValue::Str(Cow::Borrowed("tok=[xyz999]")))
    );
    assert_eq!(
        tx_view.extract_field_at_row(&LogFieldSelector::BodyField(AttributePath::single("n")), 5),
        Some(ExtractedValue::Int(42))
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::BodyField(AttributePath::single("empty")),
            5
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("WAS_NULL")))
    );
    // LogAttrs: untouched null-dict `s` -> `""`, `b` -> `&[]`, `bad_map` -> absent (not masked CBOR),
    // and nested `m.p` / `m.empty` redacted.
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("s")),
            0
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("")))
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::single("b")),
            2
        ),
        Some(ExtractedValue::Bytes(Cow::Borrowed(&[])))
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["bad_map", "leaked"])),
            1
        ),
        None
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["m", "p"])),
            5
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("attr_tok=[xyz999]")))
    );
    assert_eq!(
        tx_view.extract_field_at_row(
            &LogFieldSelector::LogAttribute(AttributePath::nested(["m", "empty"])),
            5
        ),
        Some(ExtractedValue::Str(Cow::Borrowed("ATTR_WAS_NULL")))
    );
}

/// Scenario: Evaluate root-only transform policies (both transform-only and combined with a root-only filter that keeps all rows) against plain and `encode_transport_optimized()` OTAP batches carrying attributes.
/// Guarantees: Root-only transforms produce identical OTLP-exported logs on plain and transport-optimized inputs while skipping `decode_transport_optimized_ids()` and preserving `DELTA` encoding metadata on `Logs.id`.
#[test]
fn test_root_only_transform_preserves_transport_optimized_encoding() {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };

    let logs_data = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: (0..4)
                    .map(|i| {
                        LogRecord::build()
                            .severity_number(9)
                            .body(AnyValue::new_string(format!("msg secret=val{i} ok")))
                            .attributes(vec![KeyValue::new(
                                "env",
                                AnyValue::new_string(format!("e{i}")),
                            )])
                            .finish()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let plain = encode_otlp_logs(&logs_data);
    let mut transport = plain.clone();
    transport
        .encode_transport_optimized()
        .expect("encode_transport_optimized");
    // Sanity-check that the encoded input actually carries DELTA encoding on `Logs.id`.
    let orig_id_encoding = transport
        .get(ArrowPayloadType::Logs)
        .unwrap()
        .schema()
        .field_with_name(consts::ID)
        .unwrap()
        .metadata()
        .get(consts::metadata::COLUMN_ENCODING)
        .cloned();
    assert_eq!(
        orig_id_encoding.as_deref(),
        Some(consts::metadata::encodings::DELTA)
    );

    let redact_tx = Policy::log_transform(
        "redact_body_root_only",
        LogTransformPolicy {
            matches: vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::Body),
                predicate: Predicate::Contains(ScalarValue::String("secret=".into())),
                negate: false,
            }],
            rename: vec![],
            add: vec![],
            remove: vec![],
            redact: vec![LogRedact {
                target: LogTargetSelector::RecordField(LogRecordField::Body),
                regex: Some(r"secret=[a-z0-9]+".into()),
                replacement: Some("secret=[MASKED]".into()),
            }],
        },
    );

    let collect_logs = |d: &LogsData| -> Vec<(String, Vec<(String, String)>)> {
        let mut out: Vec<_> = d
            .resource_logs
            .iter()
            .flat_map(|rl| rl.scope_logs.iter())
            .flat_map(|sl| sl.log_records.iter())
            .map(|lr| {
                let body = format!("{:?}", lr.body);
                let mut attrs: Vec<_> = lr
                    .attributes
                    .iter()
                    .map(|kv| (kv.key.clone(), format!("{:?}", kv.value)))
                    .collect();
                attrs.sort();
                (body, attrs)
            })
            .collect();
        out.sort();
        out
    };

    // 1. Transform-only (`skip_filter` path).
    let tx_only_eng = compile_engine(std::slice::from_ref(&redact_tx));
    let mut scratch = EvaluationScratch::new();
    let plain_out = tx_only_eng
        .evaluate(&plain, &mut scratch, None)
        .unwrap()
        .records
        .unwrap();
    let trans_out = tx_only_eng
        .evaluate(&transport, &mut scratch, None)
        .unwrap()
        .records
        .unwrap();
    let trans_id_enc = trans_out
        .get(ArrowPayloadType::Logs)
        .unwrap()
        .schema()
        .field_with_name(consts::ID)
        .unwrap()
        .metadata()
        .get(consts::metadata::COLUMN_ENCODING)
        .cloned();
    assert_eq!(
        trans_id_enc.as_deref(),
        Some(consts::metadata::encodings::DELTA)
    );
    let plain_otlp = decode_otlp_logs(plain_out);
    let trans_otlp = decode_otlp_logs(trans_out);
    assert_eq!(collect_logs(&plain_otlp), collect_logs(&trans_otlp));
    assert!(
        collect_logs(&trans_otlp)
            .iter()
            .all(|(b, _)| b.contains("secret=[MASKED]"))
    );

    // 2. Root-only drop filter that keeps all rows + root-only transform (`kept_rows == total_rows`).
    let filter_and_tx_eng = compile_engine(&[
        Policy::log_filter(
            "drop_none_root_only",
            vec![LogMatcher {
                field: LogFieldSelector::RecordField(LogRecordField::SeverityNumber),
                predicate: Predicate::Equals(ScalarValue::Int(24)),
                negate: false,
            }],
            Action::Drop,
        ),
        redact_tx,
    ]);
    let plain_out2 = filter_and_tx_eng
        .evaluate(&plain, &mut scratch, None)
        .unwrap()
        .records
        .unwrap();
    let trans_out2 = filter_and_tx_eng
        .evaluate(&transport, &mut scratch, None)
        .unwrap()
        .records
        .unwrap();
    let trans_id_enc2 = trans_out2
        .get(ArrowPayloadType::Logs)
        .unwrap()
        .schema()
        .field_with_name(consts::ID)
        .unwrap()
        .metadata()
        .get(consts::metadata::COLUMN_ENCODING)
        .cloned();
    assert_eq!(
        trans_id_enc2.as_deref(),
        Some(consts::metadata::encodings::DELTA)
    );
    assert_eq!(
        collect_logs(&decode_otlp_logs(plain_out2)),
        collect_logs(&decode_otlp_logs(trans_out2))
    );
}
