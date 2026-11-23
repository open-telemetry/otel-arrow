// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    UInt16Array, UInt32Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;

use crate::EvaluationScratch;
use crate::model::{
    Action, AttributePath, LogFieldSelector, LogMatcher, LogRecordField, Policy, Predicate,
    ScalarValue, ScopeField,
};
use crate::otap_ops::{
    CborPathResult, ExtractedValue, LogsBatchView, MAX_CBOR_DEPTH, cbor_item_depth,
    decode_cbor_value, extract_anyval_from_attr_batch, extract_anyval_from_struct,
    extract_bytes_from_array, extract_cbor_path, extract_cbor_path_for_eval,
    extract_cbor_path_status, extract_fixed_bytes_from_array, extract_i64_from_array,
    extract_parent_id_u16, extract_str_from_array, format_non_zero_hex, navigate_extracted_path,
};

use super::helpers::{build_attr_batch, build_custom_test_batch, compile_engine, encode_cbor};

/// Scenario: Test `ExtractedValue::into_owned`, `equals_scalar`, `contains_scalar` across all 7 value variants (including `Int`/`Double` cross-equality, `NaN`, bytes, and arrays) plus `format_non_zero_hex`.
/// Guarantees: Owned conversion preserves every variant, scalar equality and substring/element containment obey typed semantics (with `NaN != NaN`), and all-zero byte slices format as `None`.
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

/// Scenario: Decode CBOR payloads covering primitives, `u64::MAX` overflow, arrays/maps with null or non-text entries, invalid CBOR bytes, and navigate multi-segment paths.
/// Guarantees: Supported CBOR values decode to `ExtractedValue`, unsupported or out-of-range tags return `None` or are skipped inside containers, and path navigation resolves nested map keys while returning `None` on misses or non-map leaves.
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
    let map_bytes = encode_cbor(&map_val);
    let decoded_map = decode_cbor_value(&map_bytes).unwrap();

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

/// Scenario: Exercise string, `i64`/`i32`, binary, fixed-size binary, and `parent_id` extractors across native arrays, `UInt8`/`UInt16` dictionary arrays, nulls, `u32 -> u16` overflow, and unsupported Arrow types.
/// Guarantees: Every extractor decodes native and dictionary-encoded values consistently, respects null slots, rejects overflowing `u32` parent IDs, and returns `None` for unsupported types.
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

    // DictionaryArray<UInt16Type> parent_id (both UInt16 and UInt32 values)
    let dict16_u16 = DictionaryArray::<UInt16Type>::try_new(
        UInt16Array::from(vec![Some(0), None, Some(1)]),
        Arc::new(UInt16Array::from(vec![Some(7u16), None])),
    )
    .unwrap();
    let rb_dict16_u16 = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::PARENT_ID,
            dict16_u16.data_type().clone(),
            true,
        )])),
        vec![Arc::new(dict16_u16)],
    )
    .unwrap();
    assert_eq!(extract_parent_id_u16(&rb_dict16_u16, 0), Some(7));
    assert_eq!(extract_parent_id_u16(&rb_dict16_u16, 1), None);
    assert_eq!(extract_parent_id_u16(&rb_dict16_u16, 2), None);

    let dict16_u32 = DictionaryArray::<UInt16Type>::try_new(
        UInt16Array::from(vec![Some(0), Some(1)]),
        Arc::new(UInt32Array::from(vec![Some(42u32), Some(u32::MAX)])),
    )
    .unwrap();
    let rb_dict16_u32 = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::PARENT_ID,
            dict16_u32.data_type().clone(),
            true,
        )])),
        vec![Arc::new(dict16_u32)],
    )
    .unwrap();
    assert_eq!(extract_parent_id_u16(&rb_dict16_u32, 0), Some(42));
    assert_eq!(extract_parent_id_u16(&rb_dict16_u32, 1), None);
}

/// Scenario: Extract `AnyValue` entries from both `StructArray` (`Logs.body`) and attribute `RecordBatch` (`LogAttrs`) across valid values, null structs, null/invalid type discriminants, `Empty`, CBOR `Map`/`Slice`, and omitted default columns.
/// Guarantees: Valid and omitted-default slots resolve to their OTAP `ExtractedValue`, while null structs, invalid discriminants, `Empty`, and missing CBOR payloads resolve to `None`.
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
    assert_eq!(
        extract_anyval_from_struct(&sa, 7),
        Some(ExtractedValue::Double(0.0))
    );
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
    assert_eq!(
        extract_anyval_from_attr_batch(&rb, 7),
        Some(ExtractedValue::Double(0.0))
    );
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

/// Scenario: Construct `LogsBatchView` for missing, empty, minimal, and full OTAP log batches and read every `LogFieldSelector` variant by row index.
/// Guarantees: Missing or zero-row batches yield `Ok(None)`, ID lookup maps handle missing/nullable IDs without panicking, and `extract_field_at_row` resolves record, scope, body, and attribute fields accurately.
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

    // Minimal batch without ID/RESOURCE/SCOPE columns (no log IDs -> empty log_id_to_row)
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
    assert!(view.log_id_to_row().is_empty());
    assert_eq!(view.row_to_log_id(0), None);
    assert_eq!(view.row_to_log_id(1), None);
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

/// Scenario: Stream nested CBOR paths via `extract_cbor_path` and `extract_cbor_path_for_eval` across definite and indefinite containers/strings/bytes, half/single/double floats, `i64::MIN`/`i64::MAX`/overflow integers, skipped non-text keys and nested sibling subtrees, truncated/depth-bomb payloads, and end-to-end `PolicyEngine` `BodyField` + nested/exists-only CBOR `LogAttribute` evaluation.
/// Guarantees: Definite leaf text and bytes borrow zero-copy from the input slice, skipped siblings and post-match entries are not materialized, indefinite-length CBOR and binary16 floats decode accurately, malicious depth/length headers fail closed to `None`, and end-to-end policy evaluation matches nested/exists-only CBOR fields cleanly.
#[test]
fn test_streaming_cbor_path_traversal_and_edge_cases() {
    // 1. Definite map with non-text key, nested skipped sibling, and zero-copy borrowed leaves.
    let complex_map = encode_cbor(&ciborium::Value::Map(vec![
        (
            ciborium::Value::Integer(42.into()),
            ciborium::Value::Array(vec![
                ciborium::Value::Text("skipped_elem".into()),
                ciborium::Value::Map(vec![(
                    ciborium::Value::Text("deep".into()),
                    ciborium::Value::Bytes(vec![1, 2, 3, 4]),
                )]),
            ]),
        ),
        (
            ciborium::Value::Text("http".into()),
            ciborium::Value::Map(vec![
                (
                    ciborium::Value::Text("headers".into()),
                    ciborium::Value::Map(vec![(
                        ciborium::Value::Text("x-trace".into()),
                        ciborium::Value::Text("abc123".into()),
                    )]),
                ),
                (
                    ciborium::Value::Text("method".into()),
                    ciborium::Value::Text("POST".into()),
                ),
                (
                    ciborium::Value::Text("raw".into()),
                    ciborium::Value::Bytes(vec![0xde, 0xad]),
                ),
                (
                    ciborium::Value::Text("tags".into()),
                    ciborium::Value::Array(vec![
                        ciborium::Value::Text("canary".into()),
                        ciborium::Value::Integer(7.into()),
                    ]),
                ),
            ]),
        ),
    ]));

    let method = extract_cbor_path(&complex_map, &["http".into(), "method".into()]).unwrap();
    assert!(matches!(method, ExtractedValue::Str(Cow::Borrowed("POST"))));
    let raw_bytes = extract_cbor_path(&complex_map, &["http".into(), "raw".into()]).unwrap();
    assert!(matches!(
        raw_bytes,
        ExtractedValue::Bytes(Cow::Borrowed(&[0xde, 0xad]))
    ));
    // Eval mode skips map and (when decode_array_elements=false) array interiors without allocating.
    assert_eq!(
        extract_cbor_path_for_eval(&complex_map, &["http".into(), "headers".into()], false),
        Some(ExtractedValue::Map(vec![]))
    );
    assert_eq!(
        extract_cbor_path_for_eval(&complex_map, &["http".into(), "tags".into()], false),
        Some(ExtractedValue::Array(vec![]))
    );
    let tags =
        extract_cbor_path_for_eval(&complex_map, &["http".into(), "tags".into()], true).unwrap();
    assert!(matches!(
        tags,
        ExtractedValue::Array(ref elems)
            if elems.len() == 2
                && matches!(elems[0], ExtractedValue::Str(Cow::Borrowed("canary")))
                && elems[1] == ExtractedValue::Int(7)
    ));
    assert_eq!(
        extract_cbor_path(&complex_map, &["http".into(), "missing".into()]),
        None
    );

    // 2. Indefinite-length map, indefinite-length text key ("ab" = "a" + "b"), indefinite text/bytes values, and indefinite array.
    //    {_ "skip": [_ 1, 2], (_ "a", "b"): (_ "h", "i"), "raw": (_ h'01', h'02') }
    let mut indef: Vec<u8> = Vec::new();
    indef.push(0xbf); // indefinite map
    indef.extend_from_slice(&[0x64, b's', b'k', b'i', b'p']); // "skip"
    indef.extend_from_slice(&[0x9f, 0x01, 0x02, 0xff]); // [_ 1, 2]
    indef.extend_from_slice(&[0x7f, 0x61, b'a', 0x61, b'b', 0xff]); // key (_ "a", "b")
    indef.extend_from_slice(&[0x7f, 0x61, b'h', 0x61, b'i', 0xff]); // val (_ "h", "i")
    indef.extend_from_slice(&[0x63, b'r', b'a', b'w']); // "raw"
    indef.extend_from_slice(&[0x5f, 0x41, 0x01, 0x41, 0x02, 0xff]); // (_ h'01', h'02')
    indef.push(0xff); // end map
    assert_eq!(
        extract_cbor_path(&indef, &["ab".into()]),
        Some(ExtractedValue::Str(Cow::Owned("hi".into())))
    );
    assert_eq!(
        extract_cbor_path(&indef, &["raw".into()]),
        Some(ExtractedValue::Bytes(Cow::Owned(vec![0x01, 0x02])))
    );
    assert_eq!(
        extract_cbor_path(&indef, &["skip".into()]),
        Some(ExtractedValue::Array(vec![
            ExtractedValue::Int(1),
            ExtractedValue::Int(2),
        ]))
    );

    // 3. Exact i64 boundaries and overflows (i64::MAX, i64::MIN, i64::MAX + 1, i64::MIN - 1).
    let cbor_i64_max = [0x1bu8, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let cbor_i64_max_plus1 = [0x1bu8, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let cbor_i64_min = [0x3bu8, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let cbor_i64_min_minus1 = [0x3bu8, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(
        decode_cbor_value(&cbor_i64_max),
        Some(ExtractedValue::Int(i64::MAX))
    );
    assert_eq!(decode_cbor_value(&cbor_i64_max_plus1), None);
    assert_eq!(
        decode_cbor_value(&cbor_i64_min),
        Some(ExtractedValue::Int(i64::MIN))
    );
    assert_eq!(decode_cbor_value(&cbor_i64_min_minus1), None);

    // 4. IEEE 754 binary16 (f16) and binary32 (f32) decoding.
    //    f16 1.5 = 0x3e00, -0.0 = 0x8000, subnormal 0x0001 = 2^-24, +inf = 0x7c00, NaN = 0x7e00.
    assert_eq!(
        decode_cbor_value(&[0xf9, 0x3e, 0x00]),
        Some(ExtractedValue::Double(1.5))
    );
    match decode_cbor_value(&[0xf9, 0x80, 0x00]) {
        Some(ExtractedValue::Double(v)) => assert!(v == 0.0 && v.is_sign_negative()),
        other => panic!("expected -0.0, got {other:?}"),
    }
    assert_eq!(
        decode_cbor_value(&[0xf9, 0x00, 0x01]),
        Some(ExtractedValue::Double(2.0_f64.powi(-24)))
    );
    assert_eq!(
        decode_cbor_value(&[0xf9, 0x7c, 0x00]),
        Some(ExtractedValue::Double(f64::INFINITY))
    );
    match decode_cbor_value(&[0xf9, 0x7e, 0x00]) {
        Some(ExtractedValue::Double(v)) => assert!(v.is_nan()),
        other => panic!("expected NaN, got {other:?}"),
    }
    let f32_bytes = 3.25f32.to_be_bytes();
    assert_eq!(
        decode_cbor_value(&[0xfa, f32_bytes[0], f32_bytes[1], f32_bytes[2], f32_bytes[3]]),
        Some(ExtractedValue::Double(3.25))
    );

    // 5. Malicious depth bomb and forged huge length headers fail closed with None (zero panic/OOM).
    let depth_bomb = vec![0x9fu8; 128];
    assert_eq!(decode_cbor_value(&depth_bomb), None);
    let forged_array = [0x9bu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    assert_eq!(decode_cbor_value(&forged_array), None);
    let forged_bytes = [0x5bu8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    assert_eq!(decode_cbor_value(&forged_bytes), None);
    // Invalid UTF-8 in CBOR text string fails closed.
    assert_eq!(decode_cbor_value(&[0x62, 0xff, 0xfe]), None);

    // 6. Streaming stops after matching target key even if a later trailing sibling is corrupt.
    let mut map_with_trailing_corrupt = Vec::new();
    map_with_trailing_corrupt.push(0xa2); // 2-pair map
    map_with_trailing_corrupt.extend_from_slice(&[0x63, b'e', b'n', b'v']); // "env"
    map_with_trailing_corrupt.extend_from_slice(&[0x64, b'p', b'r', b'o', b'd']); // "prod"
    map_with_trailing_corrupt.extend_from_slice(&[0xff, 0xff]); // corrupt 2nd pair
    assert_eq!(
        extract_cbor_path(&map_with_trailing_corrupt, &["env".into()]),
        Some(ExtractedValue::Str(Cow::Borrowed("prod")))
    );

    // 7. End-to-end PolicyEngine: top-level Exists-only CBOR LogAttribute + nested CBOR LogAttribute
    //    (even when a top-level Exists-only CBOR attribute carries undecodable bytes).
    let mut batch = build_custom_test_batch(
        vec![Some("m0"), Some("m1")],
        vec![9, 9],
        vec!["INFO", "INFO"],
        vec![[1u8; 16]; 2],
        vec![[1u8; 8]; 2],
        vec![],
        vec![(
            0,
            "payload",
            ciborium::Value::Map(vec![(
                ciborium::Value::Text("status".into()),
                ciborium::Value::Integer(503.into()),
            )]),
        )],
    );
    // Add a second CBOR attr row for row 1 with raw invalid CBOR bytes to confirm Exists-only skips decode.
    let existing_attrs = batch.get(ArrowPayloadType::LogAttrs).unwrap().clone();
    let corrupt_attr_schema = existing_attrs.schema();
    let corrupt_row = RecordBatch::try_new(
        corrupt_attr_schema,
        vec![
            Arc::new(UInt16Array::from(vec![0u16, 1])),
            Arc::new(StringArray::from(vec!["payload", "payload"])),
            Arc::new(UInt8Array::from(vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Map as u8,
            ])),
            Arc::new(StringArray::from(vec![None::<&str>, None])),
            Arc::new(Int64Array::from(vec![None::<i64>, None])),
            Arc::new(Float64Array::from(vec![None::<f64>, None])),
            Arc::new(BooleanArray::from(vec![None::<bool>, None])),
            Arc::new(BinaryArray::from(vec![None::<&[u8]>, None])),
            Arc::new(BinaryArray::from(vec![
                Some(
                    encode_cbor(&ciborium::Value::Map(vec![(
                        ciborium::Value::Text("status".into()),
                        ciborium::Value::Integer(503.into()),
                    )]))
                    .as_slice(),
                ),
                Some(&[0xffu8, 0xff][..]),
            ])),
        ],
    )
    .unwrap();
    batch.set(ArrowPayloadType::LogAttrs, corrupt_row).unwrap();

    let eng = compile_engine(&[
        Policy::log_filter(
            "drop_payload_exists",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::single("payload")),
                predicate: Predicate::Exists,
                negate: false,
            }],
            Action::Drop,
        ),
        Policy::log_filter(
            "keep_503",
            vec![LogMatcher {
                field: LogFieldSelector::LogAttribute(AttributePath::nested(["payload", "status"])),
                predicate: Predicate::Equals(ScalarValue::Int(503)),
                negate: false,
            }],
            Action::Keep,
        ),
    ]);
    let mut scratch = EvaluationScratch::new();
    // Row 0 matches both payload Exists and payload.status == 503 -> Keep overrides Drop.
    // Row 1 matches payload Exists (without decoding invalid CBOR) and misses payload.status == 503 -> Dropped.
    let kept = eng.evaluate_filter(&batch, &mut scratch, None).unwrap();
    assert_eq!(kept.iter().collect::<Vec<_>>(), vec![0]);
}

/// Scenario: Measure CBOR nesting depths via `cbor_item_depth` and probe CBOR `null` (`0xf6`) vs absent keys via `extract_cbor_path_status`.
/// Guarantees: `cbor_item_depth` returns exact depths up to `MAX_CBOR_DEPTH` and `None` for over-deep or trailing-byte items, and `extract_cbor_path_status` distinguishes `PresentEmpty` (`0xf6`) from `Absent`.
#[test]
fn test_cbor_item_depth_and_null_path_status() {
    // Scalars and empty containers have depth 1.
    assert_eq!(cbor_item_depth(&[0x00]), Some(1));
    assert_eq!(cbor_item_depth(&[0xf6]), Some(1));
    assert_eq!(cbor_item_depth(&[0xa0]), Some(1));
    assert_eq!(cbor_item_depth(&[0x80]), Some(1));
    // Trailing garbage is rejected.
    assert_eq!(cbor_item_depth(&[0xa0, 0x00]), None);
    // Single-level map {"k": 1} has depth 2.
    let map_d2 = encode_cbor(&ciborium::Value::Map(vec![(
        ciborium::Value::Text("k".into()),
        ciborium::Value::Integer(1.into()),
    )]));
    assert_eq!(cbor_item_depth(&map_d2), Some(2));
    // Nested arrays at MAX_CBOR_DEPTH vs MAX_CBOR_DEPTH + 1.
    let mut at_limit = vec![0x81u8; MAX_CBOR_DEPTH - 1];
    at_limit.push(0x00);
    assert_eq!(cbor_item_depth(&at_limit), Some(MAX_CBOR_DEPTH));
    let mut over_limit = vec![0x81u8; MAX_CBOR_DEPTH];
    over_limit.push(0x00);
    assert_eq!(cbor_item_depth(&over_limit), None);

    // Nested CBOR null (`0xf6`) reports PresentEmpty whereas absent key reports Absent.
    let map_with_null = encode_cbor(&ciborium::Value::Map(vec![
        (
            ciborium::Value::Text("empty_key".into()),
            ciborium::Value::Null,
        ),
        (
            ciborium::Value::Text("real_key".into()),
            ciborium::Value::Text("ok".into()),
        ),
    ]));
    assert_eq!(
        extract_cbor_path_status(&map_with_null, &["empty_key".into()], false),
        CborPathResult::PresentEmpty
    );
    assert_eq!(
        extract_cbor_path_status(&map_with_null, &["missing_key".into()], false),
        CborPathResult::Absent
    );
    assert_eq!(
        extract_cbor_path_status(&map_with_null, &["real_key".into()], false),
        CborPathResult::Present(ExtractedValue::Str(Cow::Borrowed("ok")))
    );
}
