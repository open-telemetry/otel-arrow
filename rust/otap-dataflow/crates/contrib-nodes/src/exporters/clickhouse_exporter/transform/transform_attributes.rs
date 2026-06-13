//! Attribute row grouping and serialization helpers for ClickHouse attribute columns.
//!
//! This module contains utilities for converting OTAP “attribute row” payloads (key/type/value
//! columns keyed by `parent_id`) into the denormalized `Map(LowCardinality(String), String)`
//! representation used by ClickHouse.
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
//! These helpers are typically used as part of the multi-column transformation stage that
//! normalizes/deduplicates attribute rows into per-parent attribute sets and produces an ID mapping
//! for rewriting foreign keys in parent signal batches.
use std::collections::HashMap;

use arrow::array::{Array, MapBuilder, StringBuilder, UInt16Array, UInt32Builder};
use arrow::array::{RecordBatch, UInt32Array};
use arrow::compute::cast;
use arrow::datatypes::DataType;
use arrow_array::MapArray;
use base64::Engine;
use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

use crate::exporters::clickhouse_exporter::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    get_array_op, get_binary_array_opt, get_bool_array_opt, get_f64_array_opt, get_u8_array,
};

use crate::exporters::clickhouse_exporter::error::ClickhouseExporterError;
use crate::exporters::clickhouse_exporter::transform::transform_column::append_cbor_as_json;

/// The set of row indices belonging to a single parent-id group.
///
/// The fast path produces a contiguous [`RowRun::Range`] (no per-group allocation); the unsorted
/// fallback produces an explicit [`RowRun::Indices`] list. Both iterate as `usize` row indices via
/// [`RowRun::iter`].
pub(crate) enum RowRun {
    /// Contiguous `start..end` range of row indices (sorted/grouped fast path).
    Range(std::ops::Range<usize>),
    /// Explicit list of row indices (unsorted fallback).
    Indices(Vec<u16>),
}

impl RowRun {
    pub(crate) fn iter(&self) -> RowRunIter<'_> {
        match self {
            RowRun::Range(r) => RowRunIter::Range(r.clone()),
            RowRun::Indices(v) => RowRunIter::Indices(v.iter()),
        }
    }
}

pub(crate) enum RowRunIter<'a> {
    Range(std::ops::Range<usize>),
    Indices(std::slice::Iter<'a, u16>),
}

impl Iterator for RowRunIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            RowRunIter::Range(r) => r.next(),
            RowRunIter::Indices(it) => it.next().map(|&x| x as usize),
        }
    }
}

/// Group row indices by their parent ID (so we can convert to a map representation).
///
/// OTAP attribute rows arrive sorted ascending by `parent_id`, so equal ids are contiguous. The
/// fast path is a single linear pass that emits `(id, start..end)` runs, no hashing and no
/// per-group `Vec` allocations, with deterministic ordering. If the "sorted & contiguous" invariant
/// is violated (a previously-seen id reappears, or ids are out of order, e.g. when nulls split a
/// group), it falls back to the order-independent `HashMap` grouping.
pub(crate) fn group_rows_by_id(ids: &UInt16Array) -> Vec<(u16, RowRun)> {
    let len = ids.len();
    let mut runs: Vec<(u16, RowRun)> = Vec::new();
    // Start index of the run currently being accumulated.
    let mut cur: Option<(u16, usize)> = None;
    // Largest id finalized into a run so far; ids must strictly increase across runs.
    let mut max_finalized: Option<u16> = None;

    for i in 0..len {
        if ids.is_null(i) {
            // A null splits any in-progress contiguous run.
            if let Some((id, start)) = cur.take() {
                runs.push((id, RowRun::Range(start..i)));
                max_finalized = Some(id);
            }
            continue;
        }

        let id = ids.value(i);
        match cur {
            Some((cur_id, _)) if cur_id == id => {} // extend current run
            Some((cur_id, start)) => {
                runs.push((cur_id, RowRun::Range(start..i)));
                max_finalized = Some(cur_id);
                if max_finalized.is_some_and(|m| id <= m) {
                    return group_rows_by_id_unsorted(ids);
                }
                cur = Some((id, i));
            }
            None => {
                if max_finalized.is_some_and(|m| id <= m) {
                    // id resumed after a null-split run, or is otherwise out of order.
                    return group_rows_by_id_unsorted(ids);
                }
                cur = Some((id, i));
            }
        }
    }

    if let Some((id, start)) = cur.take() {
        runs.push((id, RowRun::Range(start..len)));
    }

    runs
}

/// Order-independent fallback grouping for the rare case where parent ids are not sorted/contiguous.
fn group_rows_by_id_unsorted(ids: &UInt16Array) -> Vec<(u16, RowRun)> {
    let mut groups: HashMap<u16, Vec<u16>> = HashMap::with_capacity(ids.len() / 4);

    for i in 0..ids.len() {
        if ids.is_null(i) {
            continue;
        }
        let id = ids.value(i);
        groups.entry(id).or_default().push(i as u16);
    }

    groups
        .into_iter()
        .map(|(id, rows)| (id, RowRun::Indices(rows)))
        .collect()
}

fn parent_ids_as_u16(batch: &RecordBatch) -> Result<Option<UInt16Array>, ClickhouseExporterError> {
    let Some(id_arr) = batch.column_by_name(consts::PARENT_ID) else {
        return Ok(None);
    };

    let casted = match id_arr.data_type() {
        DataType::UInt16 => id_arr.clone(),
        DataType::Dictionary(_, value_type) if **value_type == DataType::UInt32 => {
            cast(id_arr, &DataType::UInt16)?
        }
        other => {
            return Err(ClickhouseExporterError::Child(
                otap_df_pdata::error::Error::ColumnDataTypeMismatch {
                    name: consts::PARENT_ID.into(),
                    expect: DataType::UInt16,
                    actual: other.clone(),
                },
            ));
        }
    };

    let ids = casted
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast casted parent_id column to UInt16Array".into(),
        })?
        .clone();

    Ok(Some(ids))
}

/// For Clickhouse Map(str, str) attribute columns, we group the rows by parent_id and represent the set as an array of dicts.
pub(crate) fn group_attributes_to_map_str(
    batch: &RecordBatch,
) -> Result<Option<(UInt32Array, MapArray)>, ClickhouseExporterError> {
    let Some(id_col) = parent_ids_as_u16(batch)? else {
        // No ID column, nothing to do.
        return Ok(None);
    };
    let groups = group_rows_by_id(&id_col);
    let types = get_u8_array(batch, consts::ATTRIBUTE_TYPE)?;
    let key_accessor = StringArrayAccessor::try_new_for_column(batch, consts::ATTRIBUTE_KEY)?;

    // Build all value accessors once up front rather than per row. The accessors borrow the batch
    // columns, so they are valid for the whole grouping loop below.
    let str_accessor = match get_array_op(batch, consts::ATTRIBUTE_STR) {
        Some(col) => Some(StringArrayAccessor::try_new(col)?),
        None => None,
    };
    let int_accessor = match get_array_op(batch, consts::ATTRIBUTE_INT) {
        Some(col) => Some(Int64ArrayAccessor::try_new(col)?),
        None => None,
    };
    let double_col = get_f64_array_opt(batch, consts::ATTRIBUTE_DOUBLE)?;
    let bool_col = get_bool_array_opt(batch, consts::ATTRIBUTE_BOOL)?;
    let bytes_col = get_binary_array_opt(batch, consts::ATTRIBUTE_BYTES)?;
    let ser_accessor = match get_array_op(batch, consts::ATTRIBUTE_SER) {
        Some(col) => Some(ByteArrayAccessor::try_new(col)?),
        None => None,
    };

    let keys_builder = StringBuilder::new();
    let values_builder = StringBuilder::new();
    let mut map_builder = MapBuilder::new(None, keys_builder, values_builder);
    let mut id_builder = UInt32Builder::with_capacity(groups.len());

    for (id, rows) in groups {
        for row in rows.iter() {
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
                    if let Some(str_accessor) = &str_accessor {
                        if let Some(v) = str_accessor.str_at(row) {
                            map_builder.values().append_value(v);
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Int as u8 => {
                    if let Some(int_accessor) = &int_accessor {
                        if let Some(v) = int_accessor.value_at(row) {
                            let mut itoa_buf = itoa::Buffer::new();
                            map_builder.values().append_value(itoa_buf.format(v));
                            continue;
                        }
                    }
                    map_builder.values().append_value("");
                }

                t if t == AttributeValueType::Double as u8 => {
                    if let Some(col) = double_col {
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
                    if let Some(col) = bool_col {
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
                    if let Some(col) = bytes_col {
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
                    if let Some(byte_accessor) = &ser_accessor {
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

#[cfg(test)]
mod tests_group_attributes {
    use super::*;
    use std::sync::Arc;

    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, MapArray,
        PrimitiveDictionaryBuilder, StringArray, UInt8Array, UInt16Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt32Type};
    use arrow::record_batch::RecordBatch;

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
    fn map_groups_dictionary_encoded_u32_parent_ids() {
        let fields = vec![
            Field::new(
                consts::PARENT_ID,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                true,
            ),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ];

        let mut parent_ids = PrimitiveDictionaryBuilder::<UInt8Type, UInt32Type>::new();
        let _ = parent_ids.append(42_u32).unwrap();
        let _ = parent_ids.append(42_u32).unwrap();
        let _ = parent_ids.append(99_u32).unwrap();
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            vec![
                Arc::new(parent_ids.finish()) as ArrayRef,
                Arc::new(UInt8Array::from(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])) as ArrayRef,
                Arc::new(StringArray::from(vec![Some("a"), Some("b"), Some("c")])) as ArrayRef,
                Arc::new(StringArray::from(vec![Some("x"), Some("y"), Some("z")])) as ArrayRef,
            ],
        )
        .unwrap();

        let (ids, map) = group_attributes_to_map_str(&batch).unwrap().unwrap();
        let rows = map_to_vec(&map);
        let by_id: HashMap<u32, Vec<(String, String)>> = (0..ids.len())
            .map(|i| (ids.value(i), rows[i].clone()))
            .collect();

        assert_eq!(
            by_id[&42],
            vec![("a".into(), "x".into()), ("b".into(), "y".into())]
        );
        assert_eq!(by_id[&99], vec![("c".into(), "z".into())]);
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
}
