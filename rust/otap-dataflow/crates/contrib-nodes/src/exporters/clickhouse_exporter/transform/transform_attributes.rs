//! Attribute row grouping and serialization helpers for ClickHouse attribute columns.
//!
//! This module contains utilities for converting OTAP “attribute row” payloads (key/type/value
//! columns keyed by `parent_id`) into the denormalized representations used by ClickHouse:
//!
//! - `Map(String, String)` for string-map attribute columns, and
//! - JSON object strings for JSON attribute columns.
//!
//! Core functionality:
//!
//! - `group_rows_by_id`: scans a `UInt16Array` of parent IDs and groups row indexes by ID, ignoring
//!   null IDs. This is the common grouping primitive used by the serializers.
//!
//! - `group_attributes_to_map_str`: for each `parent_id` group, builds a ClickHouse-compatible
//!   Arrow `MapArray` where each entry is `key -> stringified value`. Values are coerced from the
//!   OTLP attribute variants (string/int/double/bool/bytes/map/slice), including base64 encoding for
//!   bytes and CBOR→JSON transcoding for map/slice serialized values. Returns the deduped parent ID
//!   column alongside the map column.
//!
//! - `group_attributes_to_json_ser`: for each `parent_id` group, builds a JSON object encoded as
//!   bytes and stores it as a dictionary-encoded binary array (to reduce duplication across
//!   identical objects). Values follow the same coercion rules as above, emitting JSON `null` when a
//!   value is missing or cannot be represented.
//!
//! These helpers are typically used as part of the multi-column transformation stage that
//! normalizes/deduplicates attribute rows into per-parent attribute sets and produces an ID mapping
//! for rewriting foreign keys in parent signal batches.
use std::collections::HashMap;

use arrow::array::{
    Array, BinaryDictionaryBuilder, DictionaryArray, MapBuilder, StringBuilder, UInt16Array,
    UInt32Builder,
};
use arrow::array::{RecordBatch, UInt32Array};
use arrow::datatypes::UInt32Type;
use arrow_array::MapArray;
use base64::Engine;
use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

use crate::clickhouse_exporter::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    get_array_op, get_binary_array_opt, get_bool_array_opt, get_f64_array_opt, get_u8_array,
    get_u16_array_opt,
};

use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::transform::transform_column::append_cbor_as_json;
use serde::ser::Serializer;

/// Iterate through the ID column and group rows by the parent ID (so we can convert to map or json representation).
pub(crate) fn group_rows_by_id(ids: &UInt16Array) -> HashMap<u16, Vec<u16>> {
    let mut groups: HashMap<u16, Vec<u16>> = HashMap::with_capacity(ids.len() / 4);

    for i in 0..ids.len() {
        if ids.is_null(i) {
            continue;
        }
        let id = ids.value(i);
        groups.entry(id).or_default().push(i as u16);
    }

    groups
}

/// For Clickhouse Map(str, str) attribute columns, we group the rows by parent_id and represent the set as an array of dicts.
pub(crate) fn group_attributes_to_map_str(
    batch: &RecordBatch,
) -> Result<Option<(UInt32Array, MapArray)>, ClickhouseExporterError> {
    // TODO: [Correctness] per @a.lockett:
    // When we support span event/span link attributes, or metric datapoint attributes,
    // we'll probably need to make this generic over the type of parent_id column as these attributes have a u32 parent_id column.
    // Also, to complicate matters, the parent_id column may also be dictionary encoded when it is a u32 type!
    let Some(id_col) = get_u16_array_opt(batch, consts::PARENT_ID)? else {
        // No ID column, nothing to do.
        return Ok(None);
    };
    let groups = group_rows_by_id(id_col);
    let types = get_u8_array(batch, consts::ATTRIBUTE_TYPE)?;
    let key_accessor = StringArrayAccessor::try_new_for_column(batch, consts::ATTRIBUTE_KEY)?;

    let keys_builder = StringBuilder::new();
    let values_builder = StringBuilder::new();
    let mut map_builder = MapBuilder::new(None, keys_builder, values_builder);
    let mut id_builder = UInt32Builder::with_capacity(groups.len());

    for (id, rows) in groups {
        for row_u16 in rows {
            let row = row_u16 as usize;
            let Some(key) = key_accessor.str_at(row) else {
                // No key, skip it
                continue;
            };
            map_builder.keys().append_value(key);

            let t = types.value(row);
            match t {
                t if t == AttributeValueType::Empty as u8 => {
                    // No value, default for type
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Str as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_STR) {
                        // TODO: [Optimization] per @a.lockett:
                        // Rather than pulling out the columnand creating a new StringArrayAccessor for each element, it might be faster to create all the accessors up front.
                        // We have a type that can be used for this in otel-arrow, but unfortunately it's not public 😠 !
                        // For now, you could maybe copy it into arrays.rs, and we'll add exposing this to the list of improvements that can be made to the OSS crate
                        // https://github.com/open-telemetry/otel-arrow/blob/cbc03d838832e2dedba932c899b95cdf95b07594/rust/otap-dataflow/crates/pdata/src/otlp/common.rs#L283-L296
                        let str_accessor = StringArrayAccessor::try_new(col)?;
                        if let Some(v) = str_accessor.str_at(row) {
                            map_builder.values().append_value(v);
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Int as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_INT) {
                        let int_accessor = Int64ArrayAccessor::try_new(col)?;
                        if let Some(v) = int_accessor.value_at(row) {
                            let mut itoa_buf = itoa::Buffer::new();
                            map_builder.values().append_value(itoa_buf.format(v));
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Double as u8 => {
                    if let Some(col) = get_f64_array_opt(batch, consts::ATTRIBUTE_DOUBLE)? {
                        if !col.is_null(row) {
                            let mut r_buf = ryu::Buffer::new();
                            map_builder
                                .values()
                                .append_value(r_buf.format(col.value(row)));
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Bool as u8 => {
                    if let Some(col) = get_bool_array_opt(batch, consts::ATTRIBUTE_BOOL)? {
                        if !col.is_null(row) {
                            if col.value(row) {
                                map_builder.values().append_value("true");
                                continue;
                            } else {
                                map_builder.values().append_value("false");
                                continue;
                            }
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Bytes as u8 => {
                    if let Some(col) = get_binary_array_opt(batch, consts::ATTRIBUTE_BYTES)? {
                        if !col.is_null(row) {
                            let bytes = col.value(row);
                            let v = base64::engine::general_purpose::STANDARD.encode(bytes);
                            map_builder.values().append_value(v);
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Map as u8 || t == AttributeValueType::Slice as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_SER) {
                        let byte_accessor = ByteArrayAccessor::try_new(col)?;
                        if let Some(v) = byte_accessor.slice_at(row) {
                            let mut buf = Vec::with_capacity(v.len() * 2);
                            if append_cbor_as_json(&mut buf, v).is_ok() {
                                if let Ok(json) = String::from_utf8(buf) {
                                    map_builder.values().append_value(json);
                                    continue;
                                }
                            }
                        }
                    }
                    map_builder.values().append_value("");
                }
                _ => map_builder.values().append_value(""),
            };
        }
        map_builder.append(true)?; // true = row is not null
        id_builder.append_value(id as u32);
    }

    Ok(Some((id_builder.finish(), map_builder.finish())))
}

/// For Clickhouse JSON attribute columns, we group the rows by parent_id and represent the set as a json string.
pub(crate) fn group_attributes_to_json_ser(
    batch: &RecordBatch,
) -> Result<Option<(UInt32Array, DictionaryArray<UInt32Type>)>, ClickhouseExporterError> {
    let Some(id_col) = get_u16_array_opt(batch, consts::PARENT_ID)? else {
        // No ID column, nothing to do.
        return Ok(None);
    };
    let groups = group_rows_by_id(id_col);

    let mut data_col_builder = BinaryDictionaryBuilder::<UInt32Type>::new();
    let mut id_builder: arrow::array::PrimitiveBuilder<UInt32Type> =
        UInt32Builder::with_capacity(groups.len());

    let types = get_u8_array(batch, consts::ATTRIBUTE_TYPE)?;
    let key_accessor = StringArrayAccessor::try_new_for_column(batch, consts::ATTRIBUTE_KEY)?;

    for (id, rows) in groups {
        // TODO: [Optimization] per @a.lockett:
        // this implementation would be a good use case for serde_json::Serializer
        // https://docs.rs/serde_json/latest/serde_json/struct.Serializer.html
        // If we wanted to be really fancy, we could also refactor the code we use to serialize attributes as CBOR to accept any serializer, and just use this helper method directly
        // https://github.com/open-telemetry/otel-arrow/blob/cbc03d838832e2dedba932c899b95cdf95b07594/rust/otap-dataflow/crates/pdata/src/encode/cbor.rs#L100-L120
        // The view that this method takes is implemented in this module:
        // https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otap/logs.rs
        // Again, this would require changes to the OSS crate, so we can probably leave this for the future

        // Reserve enough space for a medium-sized JSON object
        let mut buf = Vec::with_capacity(rows.len() * 48);
        buf.push(b'{');

        let mut first = true;

        for row_u16 in rows {
            let row = row_u16 as usize;

            let Some(key) = key_accessor.str_at(row) else {
                // No key somehow, just skip it.
                continue;
            };

            // ---------- Write `"key":` ----------
            if !first {
                buf.push(b',');
            } else {
                first = false;
            }

            // Write JSON key: "foo":
            buf.push(b'"');
            buf.extend_from_slice(key.as_bytes());
            buf.push(b'"');
            buf.push(b':');

            // ---------- Write value ----------
            let t = types.value(row);

            match t {
                t if t == AttributeValueType::Empty as u8 => {
                    buf.extend_from_slice(b"null");
                }

                t if t == AttributeValueType::Str as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_STR) {
                        let str_accessor = StringArrayAccessor::try_new(col)?;
                        if let Some(v) = str_accessor.str_at(row) {
                            let mut ser = serde_json::Serializer::new(&mut buf);
                            ser.serialize_str(v).map_err(|e| {
                                ClickhouseExporterError::SerializationError {
                                    error: format!("Failed to serialize string attribute: {}", e),
                                }
                            })?;
                            continue;
                        }
                    }
                    buf.extend_from_slice(b"null");
                }

                t if t == AttributeValueType::Int as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_INT) {
                        let int_accessor = Int64ArrayAccessor::try_new(col)?;
                        let v = int_accessor.value_at_or_default(row);
                        let mut itoa_buf = itoa::Buffer::new();
                        buf.extend_from_slice(itoa_buf.format(v).as_bytes());
                        continue;
                    }
                    buf.extend_from_slice(b"null");
                }

                t if t == AttributeValueType::Double as u8 => {
                    if let Some(col) = get_f64_array_opt(batch, consts::ATTRIBUTE_DOUBLE)? {
                        if col.is_null(row) {
                            buf.extend_from_slice(b"null");
                        } else {
                            let mut r_buf = ryu::Buffer::new();
                            buf.extend_from_slice(r_buf.format(col.value(row)).as_bytes());
                        }
                    } else {
                        buf.extend_from_slice(b"null");
                    }
                }

                t if t == AttributeValueType::Bool as u8 => {
                    if let Some(col) = get_bool_array_opt(batch, consts::ATTRIBUTE_BOOL)? {
                        if col.is_null(row) {
                            buf.extend_from_slice(b"null");
                        } else if col.value(row) {
                            buf.extend_from_slice(b"true");
                        } else {
                            buf.extend_from_slice(b"false");
                        }
                    } else {
                        buf.extend_from_slice(b"null");
                    }
                }

                t if t == AttributeValueType::Bytes as u8 => {
                    if let Some(col) = get_binary_array_opt(batch, consts::ATTRIBUTE_BYTES)? {
                        if col.is_null(row) {
                            buf.extend_from_slice(b"null");
                        } else {
                            let bytes = col.value(row);
                            let v = base64::engine::general_purpose::STANDARD.encode(bytes);
                            buf.push(b'"');
                            buf.extend_from_slice(v.as_bytes());
                            buf.push(b'"');
                        }
                    } else {
                        buf.extend_from_slice(b"null");
                    }
                }

                t if t == AttributeValueType::Map as u8 || t == AttributeValueType::Slice as u8 => {
                    if let Some(col) = get_array_op(batch, consts::ATTRIBUTE_SER) {
                        let byte_accessor = ByteArrayAccessor::try_new(col)?;
                        if let Some(v) = byte_accessor.slice_at(row) {
                            if append_cbor_as_json(&mut buf, v).is_err() {
                                buf.extend_from_slice(b"null");
                            }
                            continue;
                        }
                    }
                    buf.extend_from_slice(b"null");
                }

                _ => buf.extend_from_slice(b"null"),
            };
        }

        buf.push(b'}');

        data_col_builder.append_value(&buf);
        id_builder.append_value(id as u32);
    }

    Ok(Some((id_builder.finish(), data_col_builder.finish())))
}

#[cfg(test)]
mod tests_group_attributes {
    use super::*;
    use std::sync::Arc;

    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, MapArray, StringArray,
        UInt8Array, UInt16Array,
    };
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    fn dict_to_strings(dict: &DictionaryArray<UInt32Type>) -> Vec<String> {
        // DictionaryArray<K>: keys are indices into values()
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<BinaryArray>()
            .unwrap();
        (0..dict.len())
            .map(|i| {
                if dict.is_null(i) {
                    return "null".to_string();
                }
                let key = dict.keys().value(i) as usize;
                let bytes = values.value(key);
                String::from_utf8(bytes.to_vec()).unwrap()
            })
            .collect()
    }

    fn by_id(ids: &UInt32Array, dict: &DictionaryArray<UInt32Type>) -> HashMap<u32, String> {
        let ss = dict_to_strings(dict);
        (0..ids.len())
            .map(|i| (ids.value(i), ss[i].clone()))
            .collect()
    }

    fn build_batch(
        parent_id: Option<Vec<Option<u16>>>,
        attr_type: Vec<u8>,
        attr_key: Vec<Option<&'static str>>,
        attr_str: Option<Vec<Option<&'static str>>>,
        attr_int: Option<Vec<Option<i64>>>,
        attr_double: Option<Vec<Option<f64>>>,
        attr_bool: Option<Vec<Option<bool>>>,
        attr_bytes: Option<Vec<Option<&'static [u8]>>>,
        // for ATTRIBUTE_SER: use BinaryArray with CBOR bytes per row (or None)
        attr_ser: Option<Vec<Option<&'static [u8]>>>,
    ) -> RecordBatch {
        let mut fields: Vec<Field> = vec![];
        let mut cols: Vec<ArrayRef> = vec![];

        if let Some(parent_id) = parent_id {
            fields.push(Field::new(consts::PARENT_ID, DataType::UInt16, true));
            cols.push(Arc::new(UInt16Array::from(parent_id)) as ArrayRef);
        }

        fields.push(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false));
        cols.push(Arc::new(UInt8Array::from(attr_type)) as ArrayRef);

        fields.push(Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, true));
        cols.push(Arc::new(StringArray::from(attr_key)) as ArrayRef);

        if let Some(v) = attr_str {
            fields.push(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true));
            cols.push(Arc::new(StringArray::from(v)) as ArrayRef);
        }
        if let Some(v) = attr_int {
            fields.push(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true));
            cols.push(Arc::new(Int64Array::from(v)) as ArrayRef);
        }
        if let Some(v) = attr_double {
            fields.push(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            ));
            cols.push(Arc::new(Float64Array::from(v)) as ArrayRef);
        }
        if let Some(v) = attr_bool {
            fields.push(Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true));
            cols.push(Arc::new(BooleanArray::from(v)) as ArrayRef);
        }
        if let Some(v) = attr_bytes {
            fields.push(Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true));
            cols.push(Arc::new(BinaryArray::from(v)) as ArrayRef);
        }
        if let Some(v) = attr_ser {
            fields.push(Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true));
            cols.push(Arc::new(BinaryArray::from(v)) as ArrayRef);
        }

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, cols).unwrap()
    }

    /// Convert MapArray output to Vec<Vec<(String,String)>> for easy assertions.
    fn map_to_vec(map: &MapArray) -> Vec<Vec<(String, String)>> {
        let keys = map.keys().as_any().downcast_ref::<StringArray>().unwrap();
        let values = map.values().as_any().downcast_ref::<StringArray>().unwrap();

        let mut out = Vec::with_capacity(map.len());
        for i in 0..map.len() {
            let start = map.value_offsets()[i] as usize;
            let end = map.value_offsets()[i + 1] as usize;
            let mut row = Vec::with_capacity(end - start);
            for j in start..end {
                row.push((keys.value(j).to_string(), values.value(j).to_string()));
            }
            out.push(row);
        }
        out
    }

    #[test]
    fn map_returns_none_when_parent_id_missing() {
        let batch = build_batch(
            None,
            vec![AttributeValueType::Str as u8],
            vec![Some("k")],
            Some(vec![Some("v")]),
            None,
            None,
            None,
            None,
            None,
        );

        let got = group_attributes_to_map_str(&batch).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn map_groups_by_parent_id_and_skips_null_keys() {
        // rows:
        // id=10: ("a","x"), (null key -> skipped)
        // id=20: ("b","y")
        let batch = build_batch(
            Some(vec![Some(10), Some(10), Some(20)]),
            vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
            ],
            vec![Some("a"), None, Some("b")],
            Some(vec![Some("x"), Some("SHOULDNT_APPEAR"), Some("y")]),
            None,
            None,
            None,
            None,
            None,
        );

        let (ids, map) = group_attributes_to_map_str(&batch).unwrap().unwrap();
        let rows = map_to_vec(&map);

        let by_id: HashMap<u32, Vec<(String, String)>> = (0..ids.len())
            .map(|i| (ids.value(i), rows[i].clone()))
            .collect();

        assert_eq!(by_id[&10], vec![("a".into(), "x".into())]);
        assert_eq!(by_id[&20], vec![("b".into(), "y".into())]);
    }

    #[test]
    fn map_formats_non_string_types_to_string_and_defaults_to_empty_on_null_or_missing_column() {
        // 6 rows for same parent, each different type.
        // Int is null -> ""
        // Double present -> "1.5"
        // Bool false -> "false"
        // Bytes b"\x01\x02" -> base64 "AQI="
        // Str column is entirely missing -> should still default to ""
        // Empty type -> ""
        let batch = build_batch(
            Some(vec![Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)]),
            vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Int as u8,
                AttributeValueType::Double as u8,
                AttributeValueType::Bool as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Empty as u8,
            ],
            vec![
                Some("s"),
                Some("i"),
                Some("d"),
                Some("b"),
                Some("bin"),
                Some("e"),
            ],
            None,                                           // ATTRIBUTE_STR missing
            Some(vec![None, None, None, None, None, None]), // Int column present but null at row 1
            Some(vec![None, None, Some(1.5), None, None, None]),
            Some(vec![None, None, None, Some(false), None, None]),
            Some(vec![None, None, None, None, Some(&[1u8, 2u8][..]), None]),
            None,
        );

        let (_ids, map) = group_attributes_to_map_str(&batch).unwrap().unwrap();
        let got = map_to_vec(&map);

        // one parent group
        assert_eq!(got.len(), 1);
        assert_eq!(
            got[0],
            vec![
                ("s".into(), "".into()),       // missing ATTRIBUTE_STR => ""
                ("i".into(), "".into()),       // null int => ""
                ("d".into(), "1.5".into()),    // formatted float
                ("b".into(), "false".into()),  // formatted bool
                ("bin".into(), "AQI=".into()), // base64
                ("e".into(), "".into()),       // empty type => ""
            ]
        );
    }

    #[test]
    fn map_and_slice_types_use_attribute_ser_and_default_to_empty_on_decode_failure() {
        // If you can easily generate real CBOR bytes used by append_cbor_as_json,
        // put them in ser_ok. Otherwise this test still exercises the failure path.
        let ser_bad: &'static [u8] = &[0xff, 0x00, 0x01];

        let batch = build_batch(
            Some(vec![Some(7), Some(7)]),
            vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Slice as u8,
            ],
            vec![Some("m"), Some("s")],
            None,
            None,
            None,
            None,
            None,
            Some(vec![Some(ser_bad), None]),
        );

        let (_ids, map) = group_attributes_to_map_str(&batch).unwrap().unwrap();
        let got = map_to_vec(&map);
        assert_eq!(got.len(), 1);

        // both should fall back to empty string in this setup
        assert_eq!(
            got[0],
            vec![("m".into(), "".into()), ("s".into(), "".into())]
        );
    }

    #[test]
    fn json_returns_none_when_parent_id_missing() {
        let batch = build_batch(
            None,
            vec![AttributeValueType::Str as u8],
            vec![Some("k")],
            Some(vec![Some("v")]),
            None,
            None,
            None,
            None,
            None,
        );

        let got = group_attributes_to_json_ser(&batch).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn groups_by_parent_id_skips_null_keys_and_emits_valid_json_objects() {
        // id=10: a="x", (null key skipped)
        // id=20: b="y"
        let batch = build_batch(
            Some(vec![Some(10), Some(10), Some(20)]),
            vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
                AttributeValueType::Str as u8,
            ],
            vec![Some("a"), None, Some("b")],
            Some(vec![Some("x"), Some("SHOULDNT_APPEAR"), Some("y")]),
            None,
            None,
            None,
            None,
            None,
        );

        let (ids, dict) = group_attributes_to_json_ser(&batch).unwrap().unwrap();
        let m = by_id(&ids, &dict);

        // Order-independent assertions:
        let v10: serde_json::Value = serde_json::from_str(&m[&10]).unwrap();
        let v20: serde_json::Value = serde_json::from_str(&m[&20]).unwrap();

        assert_eq!(v10, serde_json::json!({"a": "x"}));
        assert_eq!(v20, serde_json::json!({"b": "y"}));
    }

    #[test]
    fn serializes_mixed_types_and_null_fallbacks() {
        // One parent with many keys/types.
        // - Str is present, but value is null => null
        // - Int column present => uses default 0 when null (value_at_or_default)
        // - Double null => null
        // - Bool true => true
        // - Bytes b"\x01\x02" => "AQI="
        // - Empty => null
        // - Unknown type => null
        let batch = build_batch(
            Some(vec![
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
            ]),
            vec![
                AttributeValueType::Str as u8,
                AttributeValueType::Int as u8,
                AttributeValueType::Double as u8,
                AttributeValueType::Bool as u8,
                AttributeValueType::Bytes as u8,
                AttributeValueType::Empty as u8,
                255u8, // unknown
            ],
            vec![
                Some("s"),
                Some("i"),
                Some("d"),
                Some("b"),
                Some("bin"),
                Some("e"),
                Some("u"),
            ],
            Some(vec![None, None, None, None, None, None, None]), // Str null at row0
            Some(vec![None, None, None, None, None, None, None]), // Int null -> default 0
            Some(vec![None, None, None, None, None, None, None]), // Double null
            Some(vec![None, None, None, Some(true), None, None, None]),
            Some(vec![
                None,
                None,
                None,
                None,
                Some(&[1u8, 2u8][..]),
                None,
                None,
            ]),
            None,
        );

        let (ids, dict) = group_attributes_to_json_ser(&batch).unwrap().unwrap();
        let m = by_id(&ids, &dict);

        let v: serde_json::Value = serde_json::from_str(&m[&1]).unwrap();

        // Note: Int null => 0 (because value_at_or_default), not null.
        assert_eq!(
            v,
            serde_json::json!({
                "s": null,
                "i": 0,
                "d": null,
                "b": true,
                "bin": "AQI=",
                "e": null,
                "u": null
            })
        );
    }

    #[test]
    fn map_slice_use_attribute_ser_and_on_decode_failure_write_null() {
        // We don't depend on CBOR success here; we just ensure failure -> null.
        let ser_bad: &'static [u8] = &[0xff, 0x00, 0x01];

        let batch = build_batch(
            Some(vec![Some(7), Some(7)]),
            vec![
                AttributeValueType::Map as u8,
                AttributeValueType::Slice as u8,
            ],
            vec![Some("m"), Some("s")],
            None,
            None,
            None,
            None,
            None,
            Some(vec![Some(ser_bad), None]),
        );

        let (ids, dict) = group_attributes_to_json_ser(&batch).unwrap().unwrap();
        let m = by_id(&ids, &dict);

        let v: serde_json::Value = serde_json::from_str(&m[&7]).unwrap();
        assert_eq!(v, serde_json::json!({"m": null, "s": null}));
    }

    #[test]
    fn properly_escapes_string_values_as_json_strings() {
        // value contains characters that must be escaped in JSON.
        let batch = build_batch(
            Some(vec![Some(1)]),
            vec![AttributeValueType::Str as u8],
            vec![Some("k")],
            Some(vec![Some("quote: \" backslash: \\ newline:\n")]),
            None,
            None,
            None,
            None,
            None,
        );

        let (ids, dict) = group_attributes_to_json_ser(&batch).unwrap().unwrap();
        let m = by_id(&ids, &dict);

        // Must parse as JSON; serializer is used in the implementation.
        let v: serde_json::Value = serde_json::from_str(&m[&1]).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"k": "quote: \" backslash: \\ newline:\n"})
        );
    }
}
