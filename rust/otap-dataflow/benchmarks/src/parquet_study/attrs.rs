// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared machinery for the flattened-Parquet contenders.
//!
//! OTAP logs are stored as four interleaved record batches: the root `Logs`
//! batch plus three attribute batches (`ResourceAttrs`, `ScopeAttrs`,
//! `LogAttrs`) linked to it by parent id. Each attribute batch has the columns
//! `parent_id, key, type, str, int, double, bool, bytes, ser` where `type`
//! selects which value column is populated.
//!
//! "Flattening" denormalizes those attribute batches onto the log rows. This
//! module provides the pieces every flattened layout needs:
//!
//! - [`extract_attr_value_arrays`] reads the eight value columns from a source
//!   attribute batch (synthesizing null columns for any the encoder omitted and
//!   normalizing dictionary-encoded keys to plain UTF-8).
//! - [`gather_by_parent`] computes, for each log row, the source attribute rows
//!   that belong to it (joining on `resource.id` / `scope.id` / log `id`).
//! - [`build_attr_list_column`] / [`rebuild_attr_batch`] move attributes into a
//!   `List<Struct>` column and back into an OTAP attribute batch.
//! - [`logs_resource_id`], [`logs_scope_id`], [`logs_id`] expose the join keys.
//! - [`assert_logs_equivalent`] checks a round-tripped batch structurally.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanArray, ListArray, RecordBatch, StructArray, UInt16Array, UInt32Array,
    new_empty_array, new_null_array,
};
use arrow::buffer::{OffsetBuffer, ScalarBuffer};
use arrow::compute::{cast, take};
use arrow::datatypes::{DataType, Field, Fields, Schema};

use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use super::StudyResult;

/// Flat-table column holding each log row's denormalized resource attributes.
pub const RESOURCE_ATTRS_COL: &str = "resource_attributes";
/// Flat-table column holding each log row's denormalized scope attributes.
pub const SCOPE_ATTRS_COL: &str = "scope_attributes";
/// Flat-table column holding each log row's own attributes.
pub const LOG_ATTRS_COL: &str = "log_attributes";

/// The eight value columns of an attribute batch, in canonical order, with the
/// fixed Arrow types used by the flattened representation. `key` and `type` are
/// required; the typed value columns are nullable.
#[must_use]
pub fn attr_struct_fields() -> Fields {
    Fields::from(vec![
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ])
}

/// The list element field used for the `List<Struct>` attribute columns.
#[must_use]
pub fn attr_list_element_field() -> Arc<Field> {
    Arc::new(Field::new(
        "item",
        DataType::Struct(attr_struct_fields()),
        false,
    ))
}

/// The seven value fields of an attribute (everything except `key`): used as the
/// `value` struct of the Map attribute layout.
#[must_use]
pub fn attr_value_struct_fields() -> Fields {
    Fields::from(
        attr_struct_fields()
            .iter()
            .skip(1)
            .map(|f| f.as_ref().clone())
            .collect::<Vec<_>>(),
    )
}

/// The flat-table field for one attribute-container column.
#[must_use]
pub fn attr_list_column_field(name: &str) -> Field {
    Field::new(name, DataType::List(attr_list_element_field()), false)
}

fn normalize(array: &ArrayRef, want: &DataType) -> StudyResult<ArrayRef> {
    if array.data_type() == want {
        Ok(array.clone())
    } else {
        Ok(cast(array, want)?)
    }
}

/// Read the eight value columns (`key, type, str, int, double, bool, bytes,
/// ser`) of an attribute batch in canonical order. Columns the encoder omitted
/// (because no attribute used that value type) are materialized as all-null
/// arrays so the flattened schema is stable, and dictionary-encoded keys are
/// normalized to plain UTF-8.
pub fn extract_attr_value_arrays(attr_batch: &RecordBatch) -> StudyResult<Vec<ArrayRef>> {
    let len = attr_batch.num_rows();
    let fields = attr_struct_fields();
    let mut out = Vec::with_capacity(fields.len());
    for field in fields.iter() {
        let array = match attr_batch.column_by_name(field.name()) {
            Some(col) => normalize(col, field.data_type())?,
            None => new_null_array(field.data_type(), len),
        };
        out.push(array);
    }
    Ok(out)
}

fn downcast_u16(array: &ArrayRef) -> StudyResult<UInt16Array> {
    if array.data_type() == &DataType::UInt16 {
        Ok(array
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("checked UInt16")
            .clone())
    } else {
        let cast = cast(array, &DataType::UInt16)?;
        Ok(cast
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("cast to UInt16")
            .clone())
    }
}

/// The `id` column of the root `Logs` batch (parent of `LogAttrs`).
pub fn logs_id(logs: &RecordBatch) -> StudyResult<UInt16Array> {
    let col = logs
        .column_by_name(consts::ID)
        .ok_or("Logs batch missing `id` column")?;
    downcast_u16(col)
}

fn struct_child_u16(logs: &RecordBatch, struct_col: &str) -> StudyResult<UInt16Array> {
    let col = logs
        .column_by_name(struct_col)
        .ok_or_else(|| format!("Logs batch missing `{struct_col}` column"))?;
    let st = col
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| format!("`{struct_col}` is not a struct"))?;
    let id = st
        .column_by_name(consts::ID)
        .ok_or_else(|| format!("`{struct_col}` struct missing `id`"))?;
    downcast_u16(id)
}

/// The `resource.id` column of the root `Logs` batch (parent of `ResourceAttrs`).
pub fn logs_resource_id(logs: &RecordBatch) -> StudyResult<UInt16Array> {
    struct_child_u16(logs, consts::RESOURCE)
}

/// The `scope.id` column of the root `Logs` batch (parent of `ScopeAttrs`).
pub fn logs_scope_id(logs: &RecordBatch) -> StudyResult<UInt16Array> {
    struct_child_u16(logs, consts::SCOPE)
}

/// For each value of `parents` (one per log row), the source attribute rows that
/// share that parent id, as `take` indices plus per-log-row list offsets.
pub struct Gathered {
    /// `take` indices into the source attribute batch, concatenated by log row.
    pub indices: UInt32Array,
    /// List offsets, length `parents.len() + 1`.
    pub offsets: Vec<i32>,
}

/// Join a source attribute batch onto the log rows by parent id.
pub fn gather_by_parent(attr_batch: &RecordBatch, parents: &UInt16Array) -> StudyResult<Gathered> {
    let parent_col = attr_batch
        .column_by_name(consts::PARENT_ID)
        .ok_or("attribute batch missing `parent_id`")?;
    let parent_id = downcast_u16(parent_col)?;

    let mut by_parent: HashMap<u16, Vec<u32>> = HashMap::new();
    for row in 0..parent_id.len() {
        if parent_id.is_valid(row) {
            by_parent
                .entry(parent_id.value(row))
                .or_default()
                .push(row as u32);
        }
    }

    let mut indices: Vec<u32> = Vec::new();
    let mut offsets: Vec<i32> = Vec::with_capacity(parents.len() + 1);
    offsets.push(0);
    for i in 0..parents.len() {
        if parents.is_valid(i) {
            if let Some(rows) = by_parent.get(&parents.value(i)) {
                indices.extend_from_slice(rows);
            }
        }
        offsets.push(i32::try_from(indices.len()).expect("offset fits i32"));
    }

    Ok(Gathered {
        indices: UInt32Array::from(indices),
        offsets,
    })
}

/// Take the gathered attribute rows out of `attr_batch` as an eight-field
/// `Struct{key,type,str,int,double,bool,bytes,ser}` array (one struct row per
/// gathered attribute, concatenated across log rows).
pub fn taken_attr_struct(
    attr_batch: &RecordBatch,
    gathered: &Gathered,
) -> StudyResult<StructArray> {
    let value_arrays = extract_attr_value_arrays(attr_batch)?;
    let taken: Vec<ArrayRef> = value_arrays
        .iter()
        .map(|a| take(a, &gathered.indices, None))
        .collect::<Result<_, _>>()?;
    Ok(StructArray::new(attr_struct_fields(), taken, None))
}

/// Build a `List<Struct{key,type,str,...}>` column denormalizing `attr_batch`
/// onto the log rows described by `gathered`.
pub fn build_attr_list_column(
    attr_batch: &RecordBatch,
    gathered: &Gathered,
) -> StudyResult<ArrayRef> {
    let struct_array = taken_attr_struct(attr_batch, gathered)?;
    let offsets = OffsetBuffer::new(ScalarBuffer::from(gathered.offsets.clone()));
    let list = ListArray::new(
        attr_list_element_field(),
        offsets,
        Arc::new(struct_array),
        None,
    );
    Ok(Arc::new(list))
}

/// Rebuild an OTAP attribute batch (`parent_id` + the eight value columns) from
/// a flattened `List<Struct>` column. `entries` selects which log rows to emit
/// and the parent id to stamp on each emitted attribute row.
pub fn rebuild_attr_batch(list: &ListArray, entries: &[(usize, u16)]) -> StudyResult<RecordBatch> {
    let values = list
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or("attribute list values are not a struct")?;
    rebuild_attr_batch_from_parts(values, list.value_offsets(), entries)
}

/// Rebuild an OTAP attribute batch from an eight-field `Struct` of attribute
/// values plus per-log-row `offsets`. Used by both the nested (`List<Struct>`)
/// and map (`Map`) layouts; the map layout reassembles its `keys`/`values`
/// children into the eight-field struct first.
pub fn rebuild_attr_batch_from_parts(
    values: &StructArray,
    offsets: &[i32],
    entries: &[(usize, u16)],
) -> StudyResult<RecordBatch> {
    let mut child_indices: Vec<u32> = Vec::new();
    let mut parent_ids: Vec<u16> = Vec::new();
    for (row, pid) in entries {
        let start = offsets[*row] as usize;
        let end = offsets[*row + 1] as usize;
        for child in start..end {
            child_indices.push(u32::try_from(child).expect("index fits u32"));
            parent_ids.push(*pid);
        }
    }
    let idx = UInt32Array::from(child_indices);

    let mut fields: Vec<Field> = Vec::with_capacity(9);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(9);
    fields.push(Field::new(consts::PARENT_ID, DataType::UInt16, false));
    columns.push(Arc::new(UInt16Array::from(parent_ids)) as ArrayRef);
    for (i, field) in attr_struct_fields().iter().enumerate() {
        fields.push(field.as_ref().clone());
        columns.push(take(values.column(i), &idx, None)?);
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

/// Entries for a child attribute group (`LogAttrs`): every log row with a valid
/// id, stamped with its own id.
#[must_use]
pub fn entries_per_row(id_arr: &UInt16Array) -> Vec<(usize, u16)> {
    (0..id_arr.len())
        .filter(|&i| id_arr.is_valid(i))
        .map(|i| (i, id_arr.value(i)))
        .collect()
}

/// Entries for a parent attribute group (`ResourceAttrs` / `ScopeAttrs`): the
/// first log row observed for each distinct id, stamped with that id. This
/// re-normalizes the denormalized resource/scope attributes back to one set per
/// resource/scope, using the join id preserved in the flat table.
#[must_use]
pub fn entries_dedup(id_arr: &UInt16Array) -> Vec<(usize, u16)> {
    let mut seen: HashSet<u16> = HashSet::new();
    let mut entries = Vec::new();
    for i in 0..id_arr.len() {
        if id_arr.is_valid(i) && seen.insert(id_arr.value(i)) {
            entries.push((i, id_arr.value(i)));
        }
    }
    entries
}

/// An attribute-container column where every log row has an empty attribute
/// list (used when an attribute payload is absent from the source batch).
#[must_use]
pub fn empty_attr_list_column(num_rows: usize) -> ArrayRef {
    let children: Vec<ArrayRef> = attr_struct_fields()
        .iter()
        .map(|f| new_empty_array(f.data_type()))
        .collect();
    let struct_array = StructArray::new(attr_struct_fields(), children, None);
    let offsets = OffsetBuffer::new(ScalarBuffer::from(vec![0i32; num_rows + 1]));
    Arc::new(ListArray::new(
        attr_list_element_field(),
        offsets,
        Arc::new(struct_array),
        None,
    ))
}

/// Downcast a flat-table column to `ListArray`.
pub fn as_list<'a>(batch: &'a RecordBatch, name: &str) -> StudyResult<&'a ListArray> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("flat batch missing `{name}` column"))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| format!("`{name}` is not a list").into())
}

/// The OTAP `Logs` payload batch out of an [`OtapArrowRecords`].
pub fn logs_batch(otap: &OtapArrowRecords) -> StudyResult<&RecordBatch> {
    otap.get(ArrowPayloadType::Logs)
        .ok_or_else(|| "missing Logs payload".into())
}

/// Convenience helper used by the boolean attribute path in tests.
#[must_use]
pub fn bool_value(array: &ArrayRef, row: usize) -> Option<bool> {
    let b = array.as_any().downcast_ref::<BooleanArray>()?;
    b.is_valid(row).then(|| b.value(row))
}

/// Structurally compare a round-tripped logs batch against the original: equal
/// log-record count and equal per-payload attribute-row counts.
pub fn assert_logs_equivalent(
    original: &OtapArrowRecords,
    decoded: &OtapArrowRecords,
    codec: &str,
    compressor: &str,
) {
    let orig_logs = logs_batch(original).expect("original logs");
    let dec_logs = logs_batch(decoded).expect("decoded logs");
    assert_eq!(
        orig_logs.num_rows(),
        dec_logs.num_rows(),
        "{codec}/{compressor}: log record count differs"
    );

    for pt in [
        ArrowPayloadType::ResourceAttrs,
        ArrowPayloadType::ScopeAttrs,
        ArrowPayloadType::LogAttrs,
    ] {
        let orig = original.get(pt).map_or(0, RecordBatch::num_rows);
        let dec = decoded.get(pt).map_or(0, RecordBatch::num_rows);
        assert_eq!(
            orig, dec,
            "{codec}/{compressor}: {pt:?} attribute count differs"
        );
    }
}
