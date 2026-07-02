// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-column Arrow transformations that shape OTAP columns into the ClickHouse schema.
//!
//! [`apply_one_op`] dispatches each [`ColumnTransformOp`] against [`ColumnOpCtx`] — the mutable
//! output-column set plus the other payloads' multi-column results (used to join/remap between
//! parent and child payloads). The notable helpers:
//!
//! - `flatten_struct` lifts struct fields (resource/scope) into top-level columns.
//!   `coerce_body_to_string` / `struct_column_to_string` collapse an "AnyValue" struct into one
//!   string column (base64 for bytes, CBOR→JSON for map/slice).
//! - `inline_attribute` inlines a child attribute payload into the parent as a
//!   `Map(LowCardinality(String), String)`; `inline_child_lists` / `build_child_attr_list` expand
//!   compact, id-indexed child arrays back to parent row order via the child's remap table.
use std::collections::HashMap;
use std::sync::Arc;

use std::ops::ControlFlow;

use arrow::array::{
    Array, ArrayBuilder, ArrayRef, BinaryDictionaryBuilder, Int32Array, ListBuilder, MapBuilder,
    StringBuilder, UInt8Array, make_builder,
};
use arrow::compute::{cast, max};
use arrow::datatypes::{DataType, Float64Type, UInt8Type, UInt16Type, UInt32Type};
use arrow_array::{
    ListArray, MapArray, PrimitiveArray, StringArray, StructArray, UInt16Array, UInt32Array,
};
use base64::Engine;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::trace::v1::{
    span::SpanKind, status::StatusCode as SpanStatusCode,
};
use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

use crate::exporters::clickhouse_exporter::arrays::{NullableArrayAccessor, StructColumnAccessor};

use crate::exporters::clickhouse_exporter::consts as ch_consts;
use crate::exporters::clickhouse_exporter::error::ClickhouseExporterError;
use crate::exporters::clickhouse_exporter::transform::transform_batch::{
    MultiColumnOpResult, append_list_value,
};
use crate::exporters::clickhouse_exporter::transform::transform_plan::{
    CoerceStructStringSpec, ColumnTransformOp, EnumStringMapper, FlattenStructSpec,
};
use serde_cbor::Deserializer as CborDeserializer;
use serde_json::Serializer as JSONSerializer;
use serde_transcode::transcode;

pub(crate) struct ColumnOpCtx<'a> {
    pub columns: &'a mut HashMap<String, ArrayRef>,
    pub multi: &'a HashMap<ArrowPayloadType, MultiColumnOpResult>,
}

impl<'a> ColumnOpCtx<'a> {
    pub fn contains(&self, name: &str) -> bool {
        self.columns.contains_key(name)
    }

    fn get(&self, name: &str) -> Result<&ArrayRef, ClickhouseExporterError> {
        self.columns
            .get(name)
            .ok_or_else(|| ClickhouseExporterError::MissingColumn {
                name: name.to_string(),
            })
    }

    fn take(&mut self, name: &str) -> Result<ArrayRef, ClickhouseExporterError> {
        self.columns
            .remove(name)
            .ok_or_else(|| ClickhouseExporterError::MissingColumn {
                name: name.to_string(),
            })
    }

    fn put(&mut self, name: impl Into<String>, arr: ArrayRef) {
        _ = self.columns.insert(name.into(), arr);
    }

    fn drop_col(&mut self, name: &str) {
        _ = self.columns.remove(name);
    }

    fn child(&self, pt: ArrowPayloadType) -> Option<&MultiColumnOpResult> {
        self.multi.get(&pt)
    }
}

/// Apply a single [`ColumnTransformOp`] to the column named `current_name` in `ctx`.
///
/// Most ops follow take -> transform -> put, so a column has only one active version at a time;
/// `current_name` is updated to track renames. Returns `ControlFlow::Break` for terminal ops (e.g.
/// `Drop`) to stop further ops on that column, otherwise `Continue`.
pub(crate) fn apply_one_op(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    op: &ColumnTransformOp,
) -> Result<ControlFlow<()>, ClickhouseExporterError> {
    match op {
        ColumnTransformOp::NoOp => Ok(ControlFlow::Continue(())),

        ColumnTransformOp::Rename(new_name) => {
            let arr = ctx.take(current_name)?;
            ctx.put(new_name.clone(), arr);
            *current_name = new_name.clone();
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::CastAndRename(new_name, target_type) => {
            cast_and_rename(ctx, current_name, new_name, target_type)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::Drop => {
            ctx.drop_col(current_name);
            Ok(ControlFlow::Break(()))
        }

        ColumnTransformOp::FlattenStructField(spec) => {
            flatten_struct(ctx, current_name, spec)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::CoerceBodyToString(spec) => {
            coerce_body_to_string(ctx, current_name, spec)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::InlineAttribute(child_pt) => {
            inline_attribute(ctx, current_name, *child_pt)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::InlineChildLists(child_pt) => {
            inline_child_lists(ctx, current_name, *child_pt)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::InlineChildAttrList(child_pt, to_col) => {
            build_child_attr_list(ctx, current_name, *child_pt, to_col)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::ExtractMapValue {
            key,
            output_column,
            default_value,
        } => {
            extract_map_value(ctx, current_name, key, output_column, default_value)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::EnumToString {
            output_name,
            mapper,
        } => {
            let arr = ctx.take(current_name)?;
            let string_arr = enum_to_string_array(&arr, mapper)?;
            ctx.put(output_name.clone(), string_arr);
            *current_name = output_name.clone();
            Ok(ControlFlow::Continue(()))
        }
    }
}

fn cast_and_rename(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    new_name: &str,
    target_type: &DataType,
) -> Result<(), ClickhouseExporterError> {
    let arr = ctx.take(current_name)?;
    let casted = cast_array(&arr, target_type)?;
    ctx.put(new_name.to_string(), casted);
    *current_name = new_name.to_string();
    Ok(())
}

/// Dispatch casting based on target type.
///
/// For [`DataType::Utf8`] the source array is expected to be a `FixedSizeBinary`
/// (possibly dictionary-encoded) and will be hex-encoded into a `StringArray`.
/// All other target types delegate to Arrow's built-in [`cast`].
fn cast_array(arr: &ArrayRef, target: &DataType) -> Result<ArrayRef, ClickhouseExporterError> {
    match target {
        DataType::Utf8 => fixed_binary_to_hex_string(arr),
        _ => Ok(cast(arr, target)?),
    }
}

/// Hex-encode a `FixedSizeBinary` array (plain or dictionary-encoded) into a
/// `StringArray` of lowercase hex strings.
///
/// Dictionary-encoded arrays are first unpacked to a plain `FixedSizeBinary`
/// array via Arrow's [`cast`] before conversion.
pub(crate) fn fixed_binary_to_hex_string(
    arr: &ArrayRef,
) -> Result<ArrayRef, ClickhouseExporterError> {
    use arrow::array::FixedSizeBinaryArray;

    // Unpack dictionary encoding if present.
    let plain: ArrayRef = match arr.data_type() {
        DataType::Dictionary(_, value_type) => match value_type.as_ref() {
            DataType::FixedSizeBinary(n) => cast(arr, &DataType::FixedSizeBinary(*n))?,
            other => {
                return Err(ClickhouseExporterError::InvalidColumnType {
                    name: "fixed_binary_to_hex_string".into(),
                    expected: "Dictionary with FixedSizeBinary values".into(),
                    found: format!("{:?}", other),
                });
            }
        },
        DataType::FixedSizeBinary(_) => arr.clone(),
        other => {
            return Err(ClickhouseExporterError::InvalidColumnType {
                name: "fixed_binary_to_hex_string".into(),
                expected: "FixedSizeBinary or Dictionary<*, FixedSizeBinary>".into(),
                found: format!("{:?}", other),
            });
        }
    };

    let fsb = plain
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast to FixedSizeBinaryArray".into(),
        })?;

    const HEX: &[u8; 16] = b"0123456789abcdef";

    let byte_width = fsb.value_length() as usize;
    // Each byte becomes 2 hex chars.
    let mut builder = StringBuilder::with_capacity(fsb.len(), fsb.len() * byte_width * 2);
    // Scratch buffer reused across rows to avoid a per-row allocation.
    let mut hex = String::with_capacity(byte_width * 2);

    for i in 0..fsb.len() {
        if fsb.is_null(i) {
            builder.append_null();
            continue;
        }
        hex.clear();
        for &b in fsb.value(i) {
            hex.push(HEX[(b >> 4) as usize] as char);
            hex.push(HEX[(b & 0x0f) as usize] as char);
        }
        builder.append_value(&hex);
    }

    Ok(Arc::new(builder.finish()))
}

/// Convert an `Int32` column (plain or dictionary-encoded) to a `StringArray`
/// by mapping each integer value through a proto enum's `as_str_name()`.
///
/// Handles `Int32`, `Dict(UInt8, Int32)`, and `Dict(UInt16, Int32)` inputs per
/// the OTAP spec's optimized encoding allowances. Dictionary encoding is
/// unpacked before conversion.
///
/// Unknown or out-of-range values are mapped to an empty string. Null values
/// produce null entries in the output.
fn enum_to_string_array(
    arr: &ArrayRef,
    mapper: &EnumStringMapper,
) -> Result<ArrayRef, ClickhouseExporterError> {
    // Unpack dictionary encoding if present.
    let plain: ArrayRef = match arr.data_type() {
        DataType::Dictionary(_, value_type) if value_type.as_ref() == &DataType::Int32 => {
            cast(arr, &DataType::Int32)?
        }
        DataType::Int32 => arr.clone(),
        other => {
            return Err(ClickhouseExporterError::InvalidColumnType {
                name: "enum_to_string_array".into(),
                expected: "Int32 or Dictionary<*, Int32>".into(),
                found: format!("{:?}", other),
            });
        }
    };

    let int_arr = plain.as_any().downcast_ref::<Int32Array>().ok_or_else(|| {
        ClickhouseExporterError::CoercionError {
            error: "Failed to downcast to Int32Array in enum_to_string_array".into(),
        }
    })?;

    let mut builder = StringBuilder::with_capacity(int_arr.len(), int_arr.len() * 20);

    for i in 0..int_arr.len() {
        if int_arr.is_null(i) {
            builder.append_null();
        } else {
            let val = int_arr.value(i);
            let name = match mapper {
                EnumStringMapper::SpanKind => SpanKind::try_from(val)
                    .map(|v| v.as_str_name())
                    .unwrap_or(""),
                EnumStringMapper::StatusCode => SpanStatusCode::try_from(val)
                    .map(|v| v.as_str_name())
                    .unwrap_or(""),
            };
            builder.append_value(name);
        }
    }

    Ok(Arc::new(builder.finish()))
}

fn extract_map_value(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &str,
    key: &str,
    output_column: &str,
    default_value: &str,
) -> Result<(), ClickhouseExporterError> {
    let arr = ctx.get(current_name)?;

    let output = match arr.data_type() {
        DataType::Map(_, _) => extract_map_value_from_map_array(arr, key, default_value)?,
        _ => {
            return Err(ClickhouseExporterError::InvalidColumnType {
                name: current_name.to_string(),
                expected: "MapArray".into(),
                found: format!("{:?}", arr.data_type()),
            });
        }
    };

    ctx.put(output_column.to_string(), output);
    Ok(())
}

fn extract_map_value_from_map_array(
    arr: &ArrayRef,
    key: &str,
    default_value: &str,
) -> Result<ArrayRef, ClickhouseExporterError> {
    let map = arr.as_any().downcast_ref::<MapArray>().ok_or_else(|| {
        ClickhouseExporterError::CoercionError {
            error: "Failed to downcast attributes to MapArray".into(),
        }
    })?;

    let keys = map
        .keys()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast map keys to StringArray".into(),
        })?;
    let vals = map
        .values()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast map values to StringArray".into(),
        })?;

    let offsets = map.offsets();
    let mut builder = StringBuilder::with_capacity(map.len(), map.len() * default_value.len());

    for row in 0..map.len() {
        if map.is_null(row) {
            builder.append_value(default_value);
            continue;
        }

        let start = offsets[row] as usize;
        let end = offsets[row + 1] as usize;
        let mut found = None;

        for idx in start..end {
            if !keys.is_null(idx) && keys.value(idx) == key {
                found = Some(if vals.is_null(idx) {
                    default_value
                } else {
                    vals.value(idx)
                });
                break;
            }
        }

        builder.append_value(found.unwrap_or(default_value));
    }

    Ok(Arc::new(builder.finish()))
}

/// "Flatten" a struct column, extracting key fields from it into their own column in the batch.
/// This takes a Struct column (e.g. scope, resource) and pulls child fields (e.g. resource.id, resource.schema_url)
/// into their own column in the batch (and resulting clickhouse table).
fn flatten_struct(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &str,
    spec: &FlattenStructSpec,
) -> Result<(), ClickhouseExporterError> {
    // Take the struct column out so we can optionally re-insert it.
    let arr = ctx.take(current_name)?;

    let struct_array = arr.as_any().downcast_ref::<StructArray>().ok_or_else(|| {
        ClickhouseExporterError::InvalidColumnType {
            name: current_name.to_string(),
            expected: "StructArray".into(),
            found: format!("{:?}", arr.data_type()),
        }
    })?;

    // Add requested child fields as new top-level columns.
    for (field_name, new_name) in &spec.field_mapping {
        // TODO(correctness): if a log/span had a resource/scope that was null,
        // we'd flatten up the name/version/schema_id fields into the log/span which would most likely have
        // just default values in those positions. e.g. "" for text columns or 0 for numeric columns (although
        // we're not guaranteed to have these values in these columns; the arrow spec allows anything to be in
        // null positions, but I don't think we actually ever put anything non-default there in otel-arrow).
        // Semantically, these empty values are basically equivalent to null as far as the OTel data model is concerned.
        if let Some(child) = struct_array.column_by_name(field_name) {
            ctx.put(new_name.clone(), child.clone());
        }
    }

    // Optionally keep the original struct column.
    if !spec.remove_struct_col {
        ctx.put(current_name.to_string(), arr);
    }

    Ok(())
}

/// Clickhouse Exporter table expects body to be a String column. OTAP body column is potentially not, so this converts
/// non-string representations to string.
fn coerce_body_to_string(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    spec: &CoerceStructStringSpec,
) -> Result<(), ClickhouseExporterError> {
    // Remove the source struct column.
    let arr = ctx.take(current_name)?;

    // Downcast to StructArray with a good error.
    let struct_array = arr.as_any().downcast_ref::<StructArray>().ok_or_else(|| {
        ClickhouseExporterError::InvalidColumnType {
            name: current_name.clone(),
            expected: "StructArray".into(),
            found: format!("{:?}", arr.data_type()),
        }
    })?;

    // Use your accessor logic.
    let struct_accessor = StructColumnAccessor::new(struct_array);

    let type_arr: &PrimitiveArray<UInt8Type> =
        struct_accessor.primitive_column(&spec.type_field)?;

    let output_arr = struct_column_to_string(type_arr, &struct_accessor)?;

    // Insert output and advance the "current column name" pointer.
    ctx.put(spec.output_column.clone(), output_arr);
    *current_name = spec.output_column.clone();

    Ok(())
}

/// Build an `Array(Map(Utf8, Utf8))` column from a per-parent `List<UInt32>` of child ids and a
/// child attribute payload's compact map.
///
/// For each parent (span) row, the output array holds one map per child id, in list order — keeping
/// it index-aligned with the sibling `Events.*` / `Links.*` list columns. A child id with no
/// attributes (or a null id) yields an empty map. The id-list column named by `current_name` is
/// consumed; the result is stored under `to_col`. Child ids are looked up in the same u16 id-space
/// the attribute grouping uses (see `group_attributes_to_map_str`).
fn build_child_attr_list(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    child_payload: ArrowPayloadType,
    to_col: &str,
) -> Result<(), ClickhouseExporterError> {
    let multi = ctx.multi;
    let child = multi.get(&child_payload);

    // Compact map (one row per unique child id) + its key/value string arrays, if the child
    // attribute payload is present. Absent payload => every map is empty.
    let compact: Option<(&MapArray, &StringArray, &StringArray)> = match child {
        Some(c) => {
            let map = c
                .columns
                .get(consts::ATTRIBUTES)
                .ok_or_else(|| ClickhouseExporterError::MissingColumn {
                    name: consts::ATTRIBUTES.into(),
                })?
                .as_any()
                .downcast_ref::<MapArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "child attributes column is not a MapArray".into(),
                })?;
            let keys = map
                .keys()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "child map keys are not Utf8".into(),
                })?;
            let vals = map
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "child map values are not Utf8".into(),
                })?;
            Some((map, keys, vals))
        }
        None => None,
    };
    let remap = child.and_then(|c| c.remapped_ids.as_ref());

    // The id-list column: List<UInt32> of child ids per parent row.
    let id_list_arr = ctx.take(current_name)?;
    let id_lists = id_list_arr
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: format!("expected ListArray of child ids for {current_name}"),
        })?;

    let mut out = ListBuilder::with_capacity(
        MapBuilder::new(None, StringBuilder::new(), StringBuilder::new()),
        id_lists.len(),
    );

    for i in 0..id_lists.len() {
        if id_lists.is_null(i) {
            out.append_null();
            continue;
        }

        let sub = id_lists.value(i);
        let ids = sub.as_any().downcast_ref::<UInt32Array>().ok_or_else(|| {
            ClickhouseExporterError::CoercionError {
                error: "child id list values are not UInt32".into(),
            }
        })?;

        for j in 0..ids.len() {
            if !ids.is_null(j) {
                if let (Some((map, keys, vals)), Some(remap)) = (compact, remap) {
                    if let Some(&src) = remap.get(&ids.value(j)) {
                        let src = src as usize;
                        if src < map.len() && !map.is_null(src) {
                            let offsets = map.offsets();
                            let start = offsets[src] as usize;
                            let end = offsets[src + 1] as usize;
                            for k in start..end {
                                out.values().keys().append_value(keys.value(k));
                                if vals.is_null(k) {
                                    out.values().values().append_null();
                                } else {
                                    out.values().values().append_value(vals.value(k));
                                }
                            }
                        }
                    }
                }
            }

            out.values().append(true)?; // close this (possibly empty) map element
        }

        out.append(true); // close the array for this parent row
    }

    ctx.put(to_col.to_string(), Arc::new(out.finish()));
    *current_name = to_col.to_string();
    Ok(())
}

/// Take a compact ListArray which may contain values for a given row in the parent batch,
/// expand it to the same size as the parent batch, and re-order the entries to match the parent row order.
/// This is used for inlining SpanLinks and SpanEvents columns into the main Spans record batch.
fn inline_child_lists(
    ctx: &mut ColumnOpCtx<'_>,
    parent_id_col: &str,
    child_payload: ArrowPayloadType,
) -> Result<(), ClickhouseExporterError> {
    let Some((parent_ids, remap, child)) =
        parent_ids_and_child_remap(ctx, parent_id_col, child_payload)?
    else {
        return Ok(());
    };

    for (name, compact_arr) in &child.columns {
        let compact_list = compact_arr
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| ClickhouseExporterError::CoercionError {
                error: format!("Expected ListArray for {}", name),
            })?;

        let expanded = remap_list_array_to_parent_order(&parent_ids, remap, compact_list)?;
        ctx.put(name.clone(), expanded);
    }
    Ok(())
}

/// Resolve the parent id column and the child payload's remap table for an inline op.
///
/// Returns:
/// - `parent_ids`: an owned `UInt16` array (only a cheap Arc/buffer-sharing clone, plus a dictionary
///   cast when needed) so the caller does not borrow `ctx.columns`.
/// - `remap` and `child`: borrows into `ctx.multi`'s underlying data (`'b`), NOT into the `&ctx`
///   borrow. We copy the `multi` shared reference out of `ctx` first, so these borrows are
///   independent of `ctx` and the caller can still mutate `ctx.columns` (e.g. `ctx.put`) while
///   holding them without deep-copying the `HashMap` remap or the whole `MultiColumnOpResult`.
fn parent_ids_and_child_remap<'b>(
    ctx: &ColumnOpCtx<'b>,
    parent_id_col: &str,
    child_payload: ArrowPayloadType,
) -> Result<
    Option<(
        Arc<PrimitiveArray<UInt16Type>>,
        &'b HashMap<u32, u32>,
        &'b MultiColumnOpResult,
    )>,
    ClickhouseExporterError,
> {
    // Own the parent_ids Arc so we can return it safely.
    let parent_ids_arr = ctx.get(parent_id_col)?.clone();
    let parent_ids_arr = if matches!(
        parent_ids_arr.data_type(),
        DataType::Dictionary(_, value_type) if **value_type == DataType::UInt32
    ) {
        cast(&parent_ids_arr, &DataType::UInt16)?
    } else {
        parent_ids_arr
    };

    let parent_ids = parent_ids_arr
        .as_any()
        .downcast_ref::<PrimitiveArray<UInt16Type>>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Parent ID column must be UInt16".into(),
        })?;
    let parent_ids = Arc::new(parent_ids.clone()); // cheap-ish clone of the array struct; buffers are shared

    // Copy the `multi` shared reference out of `ctx` so the returned borrows are tied to `'b`
    // (the underlying data) rather than to this `&ctx` borrow.
    let multi: &'b HashMap<ArrowPayloadType, MultiColumnOpResult> = ctx.multi;
    let Some(child) = multi.get(&child_payload) else {
        return Ok(None);
    };

    let remap =
        child
            .remapped_ids
            .as_ref()
            .ok_or_else(|| ClickhouseExporterError::CoercionError {
                error: "Failed to find child batch inline key map".into(),
            })?;

    Ok(Some((parent_ids, remap, child)))
}

/// Switch cbor encoded field to JSON
pub(crate) fn append_cbor_as_json(
    buf: &mut Vec<u8>,
    cbor_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cbor = CborDeserializer::from_slice(cbor_bytes);

    // Serialize JSON directly into `buf`
    let mut json_ser = JSONSerializer::new(buf);

    transcode(&mut cbor, &mut json_ser)?;

    Ok(())
}

/// Remap/reorder a compact `ListArray` into parent row order.
///
/// This expands a child `ListArray` (`compact`) whose rows are stored in a compact/deduplicated
/// order into a new list column whose **length matches `parent_ids.len()`**, using `old_to_new` to
/// map each parent id to the appropriate compact row.
///
/// Given:
/// - `parent_ids`: per-parent-row ids (UInt16) identifying which compact row to use,
/// - `old_to_new`: mapping from parent id (`u32`) -> row index in `compact`,
/// - `compact`: the compact list column (any element type),
///
/// For each parent row `i`:
/// - if `parent_ids[i]` is null, the output row is null
/// - else if there is no mapping for that id in `old_to_new`, the output row is null
/// - else let `j = old_to_new[parent_ids[i]]`:
///   - if `j` is out of bounds or `compact[j]` is null, the output row is null
///   - otherwise, the output row is a list containing a copy of `compact[j]`'s elements
///     (including element-level nulls)
///
/// The output list’s element type is the same as `compact.values().data_type()`. Elements are copied
/// using [`append_list_value`], which handles the concrete element array types and preserves nulls.
///
/// # Errors
/// Returns an error if copying an element via [`append_list_value`] fails, or if the underlying
/// Arrow builders fail to append.
///
/// # Returns
/// An `ArrayRef` holding the newly built `ListArray` aligned to parent row order.
pub(crate) fn remap_list_array_to_parent_order(
    parent_ids: &PrimitiveArray<UInt16Type>,
    old_to_new: &HashMap<u32, u32>,
    compact: &ListArray,
) -> Result<ArrayRef, ClickhouseExporterError> {
    // output list builder has same element type as compact.values(), one list row per parent
    let values_builder: Box<dyn ArrayBuilder> = make_builder(compact.values().data_type(), 0);
    let mut out: ListBuilder<Box<dyn ArrayBuilder>> =
        ListBuilder::with_capacity(values_builder, parent_ids.len());

    let offsets = compact.value_offsets(); // &[i32] for ListArray

    for i in 0..parent_ids.len() {
        if parent_ids.is_null(i) {
            out.append_null();
            continue;
        }

        let pid = parent_ids.value(i) as u32;

        let Some(&new_idx_u32) = old_to_new.get(&pid) else {
            out.append_null();
            continue;
        };

        let new_idx = new_idx_u32 as usize;
        if new_idx >= compact.len() {
            out.append_null();
            continue;
        }

        if compact.is_null(new_idx) {
            out.append_null();
            continue;
        }

        let start = offsets[new_idx] as usize;
        let end = offsets[new_idx + 1] as usize;

        // copy elements (including nulls) from compact.values()[start..end)
        let values = compact.values();
        for j in start..end {
            append_list_value(out.values(), values, j)?;
        }
        // close the non-null list slot
        out.append(true);
    }

    Ok(Arc::new(out.finish()) as ArrayRef)
}

/// Remap/reorder a compact `MapArray` into parent row order.
///
/// This takes a child `MapArray` (`compact`) whose rows are in a compact/deduplicated order, and
/// produces a new `MapArray` whose **length matches `parent_ids.len()`**, by copying the map
/// entries from the appropriate compact row for each parent row.
///
/// Given:
/// - `parent_ids`: per-parent-row indices (UInt16) identifying which compact row to use
///   (null indicates the parent row has no map),
/// - `old_to_new`: mapping from the `parent_ids` id (`u32`) -> row index in the compact `compact`,
/// - `compact`: the compact `MapArray` (assumed `map<string, string>`),
///
/// For each parent row `i`:
/// - if `parent_ids[i]` is null, an **empty map** is appended to the output
/// - otherwise, let `src_row = old_to_new[parent_ids[i]]`
///   - the key/value pairs for `compact[src_row]` are appended into the output map
///
/// Internally this function:
/// - downcasts `compact.keys()` and `compact.values()` to `StringArray`s (so the map must be
///   `Map<Utf8, Utf8>`),
/// - uses the source `compact.offsets()` to find the contiguous range of entries for each `src_row`,
/// - rebuilds the output with an Arrow `MapBuilder`.
///
/// # Errors
/// Returns an error if:
/// - `compact` is not a `map<string, string>` (keys/values are not `StringArray`),
/// - a non-null `parent_ids[i]` has no entry in `old_to_new`,
/// - the underlying `MapBuilder` append operation fails.
///
/// # Note
/// Null `parent_ids` rows become **empty maps**, not null maps.
fn remap_map_array_to_parent_order(
    parent_ids: &PrimitiveArray<UInt16Type>,
    old_to_new: &HashMap<u32, u32>,
    compact: &MapArray,
) -> Result<MapArray, ClickhouseExporterError> {
    let keys = compact
        .keys()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to find attribute key array".into(),
        })?;

    let vals = compact
        .values()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to find attribute value array".into(),
        })?;

    // The offsets buffer for the original MapArray
    let offsets = compact.offsets();

    let mut map_builder = MapBuilder::with_capacity(
        None,
        StringBuilder::new(),
        StringBuilder::new(),
        parent_ids.len(),
    );

    for i in 0..parent_ids.len() {
        if !parent_ids.is_valid(i) {
            // This adds an empty map
            map_builder.append(true)?;
            continue;
        }

        let Some(src_row) = old_to_new.get(&(parent_ids.value(i) as u32)) else {
            // Parent rows without a child attr group should produce an empty map.
            map_builder.append(true)?;
            continue;
        };
        let src_row = *src_row as usize;

        let start = offsets[src_row] as usize;
        let end = offsets[src_row + 1] as usize;

        for j in start..end {
            map_builder.keys().append_value(keys.value(j));
            map_builder.values().append_value(vals.value(j));
        }

        map_builder.append(true)?;
    }

    Ok(map_builder.finish())
}

/// Take body struct and transform to a single string column.
pub fn struct_column_to_string(
    type_arr: &UInt8Array,
    struct_accessor: &StructColumnAccessor<'_>,
) -> Result<ArrayRef, ClickhouseExporterError> {
    let len = type_arr.len();

    // FAST PATH: check the maximum variant to see if it's all string or null
    let max_variant = max(type_arr).unwrap_or(0); // Treat None as all empty values

    // Variant 0 = empty, Variant 1 = string
    if max_variant <= 1 {
        let string_col = match struct_accessor.get_inner_array_op(consts::ATTRIBUTE_STR) {
            Some(col) => col.clone(),
            None => {
                // create a new array of nulls with length `len`
                let mut builder = BinaryDictionaryBuilder::<UInt32Type>::new();
                for _ in 0..len {
                    builder.append_null();
                }
                Arc::new(builder.finish())
            }
        };
        return Ok(string_col);
    }

    // TODO(optimization): the str/int/bytes/ser columns are usually dictionary-encoded. When so, we
    // could stringify the dictionary values once, build new dictionary values from those, and then
    // only remap each column's keys instead of going row-by-row.
    //
    // A kernel-based (take/cast) rewrite is avoided here: this slow path is a per-row *union*
    // dispatch (Empty/Str/Int/Double/Bool/Bytes/Map/Slice), not a single sorted-dictionary column.
    // It would have to stringify each variant column independently and then merge them by the
    // per-row `type_arr` selector while preserving the exact null/empty semantics below.
    // SLOW PATH: build string column row-by-row
    let mut builder = BinaryDictionaryBuilder::<UInt32Type>::new();
    let string_accessor = struct_accessor.string_column_op(consts::ATTRIBUTE_STR)?;
    let int_accessor = struct_accessor.int64_column_op(consts::ATTRIBUTE_INT)?;
    let float_accessor =
        struct_accessor.primitive_column_op::<Float64Type>(consts::ATTRIBUTE_DOUBLE)?;
    let bool_accessor = struct_accessor.bool_column_op(consts::ATTRIBUTE_BOOL)?;
    let bytes_accessor = struct_accessor.byte_array_column_op(consts::ATTRIBUTE_BYTES)?;
    let ser_accessor = struct_accessor.byte_array_column_op(consts::ATTRIBUTE_SER)?;
    for i in 0..len {
        if type_arr.is_null(i) {
            builder.append_null();
            continue;
        }

        let variant = type_arr.value(i);
        match variant {
            t if t == AttributeValueType::Empty as u8 => {
                builder.append_null();
            }

            t if t == AttributeValueType::Str as u8 => {
                if let Some(string_accessor) = &string_accessor {
                    if let Some(v) = string_accessor.str_at(i) {
                        builder.append_value(v);
                        continue;
                    }
                };
                builder.append_null();
            }

            t if t == AttributeValueType::Int as u8 => {
                if let Some(int_accessor) = &int_accessor {
                    let v = int_accessor.value_at_or_default(i);
                    let mut itoa_buf = itoa::Buffer::new();
                    builder.append_value(itoa_buf.format(v));
                    continue;
                };
                builder.append_null();
            }

            t if t == AttributeValueType::Double as u8 => {
                if let Some(float_accessor) = float_accessor {
                    let v = float_accessor.value_at_or_default(i);
                    let mut r_buf = ryu::Buffer::new();
                    builder.append_value(r_buf.format(v));
                    continue;
                };
                builder.append_null();
            }

            t if t == AttributeValueType::Bool as u8 => {
                if let Some(bool_accessor) = bool_accessor {
                    if let Some(v) = bool_accessor.value_at(i) {
                        if v {
                            builder.append_value("true");
                        } else {
                            builder.append_value("false");
                        }
                        continue;
                    }
                };
                builder.append_null();
            }

            t if t == AttributeValueType::Bytes as u8 => {
                if let Some(bytes_accessor) = &bytes_accessor {
                    if let Some(v) = bytes_accessor.slice_at(i) {
                        let mut buf = String::new();
                        base64::engine::general_purpose::STANDARD
                            .encode_string::<&[u8]>(v, &mut buf);
                        builder.append_value(buf);
                        continue;
                    }
                    builder.append_null();
                };
            }

            t if t == AttributeValueType::Map as u8 || t == AttributeValueType::Slice as u8 => {
                if let Some(ser_accessor) = &ser_accessor {
                    if let Some(v) = ser_accessor.slice_at(i) {
                        let mut buf = Vec::with_capacity(v.len() * 2);
                        if append_cbor_as_json(&mut buf, v).is_err() {
                            builder.append_null();
                        } else {
                            builder.append_value(buf);
                            continue;
                        }
                    }
                }
                builder.append_null();
            }

            _ => builder.append_null(),
        }
    }

    Ok(Arc::new(builder.finish()))
}

/// Map an arrow payload type to a corresponding output column name (i.e. the name of the column in clickhouse).
/// This is used for "Inlining" attribute columns into the main signal batch as a new column.
fn attr_output_name(pt: ArrowPayloadType) -> Result<&'static str, ClickhouseExporterError> {
    match pt {
        ArrowPayloadType::ResourceAttrs => Ok(ch_consts::CH_RESOURCE_ATTRIBUTES),
        ArrowPayloadType::ScopeAttrs => Ok(ch_consts::CH_SCOPE_ATTRIBUTES),
        ArrowPayloadType::LogAttrs => Ok(ch_consts::CH_LOG_ATTRIBUTES),
        ArrowPayloadType::SpanAttrs => Ok(ch_consts::CH_SPAN_ATTRIBUTES),
        _ => Err(ClickhouseExporterError::UnsupportedPayload {
            error: format!("failed to inline attribute, unknown payload: {:?}", pt),
        }),
    }
}

/// Inlines a child “attributes” payload column into the parent row-set.
///
/// This operator consumes the current parent  attribute id column (expected to be a `UInt16Array`)
/// and replaces it with a new column containing the *materialized attribute value per parent row*.
///
/// # How it works
/// - `current_name` is the name of the parent id column (e.g. `"scope_attributes"`).
/// - The function removes that column from `ctx` via `ctx.take(current_name)` and downcasts it to
///   `UInt16Array`.
/// - It then looks up the corresponding multi-column operation payload results for `child_payload_type`.
///   - Attribute grouping to json or map(string,string) is handled as a multi-column op.
///   - If the child payload is missing, the function restores the id column back into `ctx` and
///     returns `Ok(())` (no-op).
/// - Using `result.remapped_ids` (a mapping from parent id -> compact child row index) and the
///   child `ATTRIBUTES` column, it expands/materializes attribute values so the output column has
///   the same length and order as the parent row-set.
/// - The output column name is derived from `child_payload_type` via [`attr_output_name`], stored
///   back into `ctx`, and `current_name` is updated to that new column name.
///
/// # Errors
/// Returns an error if:
/// - The id column is present but not a `UInt16Array`.
/// - The child payload exists but is missing `remapped_ids` or the `ATTRIBUTES` column.
/// - The attributes cannot be inlined, or if underlying coercion/expansion fails.
///
/// # Side effects
/// - On success, removes the parent id column and inserts the inlined attribute column.
/// - On no-op (missing child payload), leaves the context unchanged.
/// - On `OtapArray` error, restores the id column before returning.
fn inline_attribute(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    child_payload_type: ArrowPayloadType,
) -> Result<(), ClickhouseExporterError> {
    // Take parent IDs (u16).
    let id_arr = ctx.take(current_name)?;
    let id_arr = if matches!(
        id_arr.data_type(),
        DataType::Dictionary(_, value_type) if **value_type == DataType::UInt32
    ) {
        cast(&id_arr, &DataType::UInt16)?
    } else {
        id_arr
    };
    let id_arr_u16 = id_arr
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| ClickhouseExporterError::InvalidColumnType {
            name: current_name.clone(),
            expected: "UInt16Array".into(),
            found: format!("{:?}", id_arr.data_type()),
        })?;

    // No attribute payload for this group in this batch (e.g. every scope/resource/log had empty
    // attributes). Drop the foreign-key id column entirely — it is NOT a ClickHouse column, and
    // since inserts bind by column name, leaking it (`scope_id`/`resource_id`/`id`) makes the
    // INSERT reference a column the table doesn't have (NO_SUCH_COLUMN_IN_TABLE). The attribute
    // column is simply omitted; ClickHouse fills its default (empty map). `id_arr` was already
    // taken above, so returning here leaves the column dropped.
    let Some(result) = ctx.child(child_payload_type) else {
        return Ok(());
    };

    let new_name = attr_output_name(child_payload_type)?.to_string();

    let remap =
        result
            .remapped_ids
            .as_ref()
            .ok_or_else(|| ClickhouseExporterError::CoercionError {
                error: "Failed to find attribute key map".into(),
            })?;
    let values_arr = result.columns.get(consts::ATTRIBUTES).ok_or_else(|| {
        ClickhouseExporterError::MissingColumn {
            name: consts::ATTRIBUTES.into(),
        }
    })?;

    let new_column: ArrayRef = inline_attr_string_map(id_arr_u16, remap, values_arr)?;

    ctx.put(new_name.clone(), new_column);
    *current_name = new_name;

    Ok(())
}

/// Build a new column of MapArray<String, String> with the order updated to match the parent_id column.
/// Nulls are inserted for rows where parent_id doesn't have a corresponding MapArray entry (e.g. there were no attributes).
fn inline_attr_string_map(
    parent_ids: &UInt16Array,
    remap: &HashMap<u32, u32>,
    values_arr: &ArrayRef,
) -> Result<ArrayRef, ClickhouseExporterError> {
    let values = values_arr
        .as_any()
        .downcast_ref::<MapArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast attributes to MapArray".into(),
        })?;

    let new_map_array = remap_map_array_to_parent_order(parent_ids, remap, values)?;
    Ok(Arc::new(new_map_array))
}

#[cfg(test)]
mod tests {
    #![allow(unused_results)]
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::buffer::OffsetBuffer;
    use arrow::datatypes::*;
    use arrow_array::{DictionaryArray, DurationNanosecondArray, UInt32Array, UInt64Array};

    fn ctx_with<'a>(
        columns: &'a mut HashMap<String, ArrayRef>,
        multi: &'a HashMap<ArrowPayloadType, MultiColumnOpResult>,
    ) -> ColumnOpCtx<'a> {
        ColumnOpCtx { columns, multi }
    }

    fn u16_array(values: &[u16]) -> ArrayRef {
        Arc::new(UInt16Array::from(values.to_vec()))
    }

    fn u16_nullable(values: Vec<Option<u16>>) -> ArrayRef {
        Arc::new(UInt16Array::from(values))
    }

    fn make_child_attr_map_2rows() -> MapArray {
        let keys = StringBuilder::new();
        let vals = StringBuilder::new();
        let mut b = MapBuilder::new(None, keys, vals);

        // row0: {"k1":"v1"}
        b.keys().append_value("k1");
        b.values().append_value("v1");
        b.append(true).unwrap();

        // row1: {"a":"b","c":"d"}
        b.keys().append_value("a");
        b.values().append_value("b");
        b.keys().append_value("c");
        b.values().append_value("d");
        b.append(true).unwrap();

        b.finish()
    }

    fn struct_body_array() -> ArrayRef {
        // struct { a: Utf8, b: UInt32 }
        let a: ArrayRef = Arc::new(StringArray::from(vec![Some("x"), None, Some("z")]));
        let b: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2, 3]));

        let fields = [
            Field::new("a", DataType::Utf8, true),
            Field::new("b", DataType::UInt32, false),
        ];
        let struct_arr = StructArray::from(vec![
            (Arc::new(fields[0].clone()), a),
            (Arc::new(fields[1].clone()), b),
        ]);
        Arc::new(struct_arr)
    }

    #[test]
    fn rename_moves_column_and_updates_current_name() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("old".into(), Arc::new(UInt32Array::from(vec![1, 2, 3])));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let mut current = "old".to_string();
        let op = ColumnTransformOp::Rename("new".to_string());

        let r = apply_one_op(&mut ctx, &mut current, &op).unwrap();
        assert!(matches!(r, ControlFlow::Continue(())));

        assert!(!ctx.columns.contains_key("old"));
        assert!(ctx.columns.contains_key("new"));
        assert_eq!(current, "new");
    }

    #[test]
    fn flatten_struct_inserts_child_columns_and_optionally_removes_struct() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("s".into(), struct_body_array());

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let spec = FlattenStructSpec {
            field_mapping: vec![
                ("a".to_string(), "a_out".to_string()),
                ("b".to_string(), "b_out".to_string()),
            ]
            .into_iter()
            .collect(),
            remove_struct_col: true,
        };

        flatten_struct(&mut ctx, "s", &spec).unwrap();

        // struct removed
        assert!(!ctx.columns.contains_key("s"));

        // children present
        assert!(ctx.columns.contains_key("a_out"));
        assert!(ctx.columns.contains_key("b_out"));

        let a_out = ctx.columns["a_out"]
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(a_out.len(), 3);
        assert_eq!(a_out.value(0), "x");
        assert!(a_out.is_null(1));
    }

    #[test]
    fn inline_child_lists_inserts_expanded_columns() {
        // Parent has 3 rows and a parent_id column (u16)
        let mut parent_cols: HashMap<String, ArrayRef> = HashMap::new();
        parent_cols.insert("parent_id".into(), u16_array(&[10, 11, 12]));

        // Child compact list has 3 rows (after compaction), mapped from old parent ids:
        // old 10 -> new 0
        // old 11 -> new 1
        // old 12 -> new 2
        let mut remap = HashMap::new();
        remap.insert(10u32, 0u32);
        remap.insert(11u32, 1u32);
        remap.insert(12u32, 2u32);

        // compact list column: row0=[1,2], row1=[9]
        let values = StringArray::from(vec!["a", "b", "c"]);
        let offsets: OffsetBuffer<i32> = OffsetBuffer::new(vec![0i32, 2, 3, 3].into());
        let compact_list = ListArray::try_new(
            Arc::new(Field::new("item", DataType::Utf8, false)),
            offsets,
            Arc::new(values),
            None,
        )
        .unwrap();

        let mut child_cols: HashMap<String, ArrayRef> = HashMap::new();
        child_cols.insert("events".into(), Arc::new(compact_list));

        let child_result = MultiColumnOpResult {
            columns: child_cols,
            remapped_ids: Some(remap),
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(ArrowPayloadType::SpanEvents, child_result);

        let mut ctx = ctx_with(&mut parent_cols, &multi);

        inline_child_lists(&mut ctx, "parent_id", ArrowPayloadType::SpanEvents).unwrap();

        // Now parent should have "events" inserted
        assert!(ctx.columns.contains_key("events"));

        let out = ctx.columns["events"]
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();

        assert_eq!(out.len(), 3);
    }

    #[test]
    fn inline_attribute_string_map_happy_path() {
        // Parent columns: id column that selects child compact rows (via remap)
        // parent rows: [10, 11, 99]  where 99 is missing => empty map row
        let mut parent_cols: HashMap<String, ArrayRef> = HashMap::new();
        parent_cols.insert("attr_id".into(), u16_nullable(vec![Some(10), Some(11)]));

        // Child: compact attributes map with 2 rows
        let child_map = make_child_attr_map_2rows();
        let mut child_cols: HashMap<String, ArrayRef> = HashMap::new();
        child_cols.insert(consts::ATTRIBUTES.into(), Arc::new(child_map));

        // Remap: old parent id -> compact row index
        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 0);
        remap.insert(11, 1);

        let child_result = MultiColumnOpResult {
            columns: child_cols,
            remapped_ids: Some(remap),
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(ArrowPayloadType::ResourceAttrs, child_result);

        let mut ctx = ctx_with(&mut parent_cols, &multi);

        // Apply
        let mut current = "attr_id".to_string();
        inline_attribute(&mut ctx, &mut current, ArrowPayloadType::ResourceAttrs).unwrap();

        // Output column name for ResourceAttrs
        assert_eq!(current, ch_consts::CH_RESOURCE_ATTRIBUTES);

        let out = ctx.columns[ch_consts::CH_RESOURCE_ATTRIBUTES]
            .as_any()
            .downcast_ref::<MapArray>()
            .unwrap();

        assert_eq!(out.len(), 2);

        // Validate row0 == child row0, row1 == child row1, row2 empty (since missing id 99)
        let keys = out.keys().as_any().downcast_ref::<StringArray>().unwrap();
        let vals = out.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offsets = out.offsets();

        // row0 offsets [0..1]
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 1);
        assert_eq!(keys.value(0), "k1");
        assert_eq!(vals.value(0), "v1");

        // row1 offsets [1..3]
        assert_eq!(offsets[1], 1);
        assert_eq!(offsets[2], 3);
        assert_eq!(keys.value(1), "a");
        assert_eq!(vals.value(1), "b");
        assert_eq!(keys.value(2), "c");
        assert_eq!(vals.value(2), "d");
    }

    #[test]
    fn inline_attribute_string_map_casts_dictionary_encoded_parent_ids() {
        let dict_values = Arc::new(UInt32Array::from(vec![10u32, 11u32]));
        let dict_keys = PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1u8)]);
        let dict_ids = DictionaryArray::try_new(dict_keys, dict_values).unwrap();

        let mut parent_cols: HashMap<String, ArrayRef> = HashMap::new();
        parent_cols.insert("attr_id".into(), Arc::new(dict_ids));

        let child_map = make_child_attr_map_2rows();
        let mut child_cols: HashMap<String, ArrayRef> = HashMap::new();
        child_cols.insert(consts::ATTRIBUTES.into(), Arc::new(child_map));

        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 0);
        remap.insert(11, 1);

        let child_result = MultiColumnOpResult {
            columns: child_cols,
            remapped_ids: Some(remap),
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(ArrowPayloadType::ResourceAttrs, child_result);

        let mut ctx = ctx_with(&mut parent_cols, &multi);
        let mut current = "attr_id".to_string();
        inline_attribute(&mut ctx, &mut current, ArrowPayloadType::ResourceAttrs).unwrap();

        assert_eq!(current, ch_consts::CH_RESOURCE_ATTRIBUTES);

        let out = ctx.columns[ch_consts::CH_RESOURCE_ATTRIBUTES]
            .as_any()
            .downcast_ref::<MapArray>()
            .unwrap();

        assert_eq!(out.len(), 2);

        let keys = out.keys().as_any().downcast_ref::<StringArray>().unwrap();
        let vals = out.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offsets = out.offsets();

        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 1);
        assert_eq!(keys.value(0), "k1");
        assert_eq!(vals.value(0), "v1");

        assert_eq!(offsets[1], 1);
        assert_eq!(offsets[2], 3);
        assert_eq!(keys.value(1), "a");
        assert_eq!(vals.value(1), "b");
        assert_eq!(keys.value(2), "c");
        assert_eq!(vals.value(2), "d");
    }

    #[test]
    fn cast_and_rename_duration_dictionary_to_u64() {
        let values = Arc::new(DurationNanosecondArray::from(vec![100_i64, 200, 300]));
        let keys = PrimitiveArray::<UInt8Type>::from(vec![Some(0_u8), Some(2_u8), Some(1_u8)]);
        let dict = DictionaryArray::try_new(keys, values).unwrap();

        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert(consts::DURATION_TIME_UNIX_NANO.into(), Arc::new(dict));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);
        let mut current = consts::DURATION_TIME_UNIX_NANO.to_string();

        cast_and_rename(
            &mut ctx,
            &mut current,
            ch_consts::CH_DURATION,
            &DataType::UInt64,
        )
        .unwrap();

        assert_eq!(current, ch_consts::CH_DURATION);
        let out = ctx.columns[ch_consts::CH_DURATION]
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        assert_eq!(out.value(0), 100);
        assert_eq!(out.value(1), 300);
        assert_eq!(out.value(2), 200);
    }

    #[test]
    fn cast_and_rename_duration_dictionary_preserves_nulls() {
        let values = Arc::new(DurationNanosecondArray::from(vec![100_i64, 200]));
        let keys = PrimitiveArray::<UInt8Type>::from(vec![Some(0_u8), None, Some(1_u8)]);
        let dict = DictionaryArray::try_new(keys, values).unwrap();

        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert(consts::DURATION_TIME_UNIX_NANO.into(), Arc::new(dict));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);
        let mut current = consts::DURATION_TIME_UNIX_NANO.to_string();

        cast_and_rename(
            &mut ctx,
            &mut current,
            ch_consts::CH_DURATION,
            &DataType::UInt64,
        )
        .unwrap();

        let out = ctx.columns[ch_consts::CH_DURATION]
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        assert_eq!(out.value(0), 100);
        assert!(out.is_null(1));
        assert_eq!(out.value(2), 200);
    }

    #[test]
    fn extract_service_name_from_map_array_when_present_and_missing() {
        let mut builder = MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        builder.keys().append_value("service.name");
        builder.values().append_value("checkout");
        builder.keys().append_value("k");
        builder.values().append_value("v");
        builder.append(true).unwrap();
        builder.keys().append_value("other");
        builder.values().append_value("x");
        builder.append(true).unwrap();
        let map = builder.finish();

        let map: ArrayRef = Arc::new(map);
        let out = extract_map_value_from_map_array(&map, "service.name", "").unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(out.value(0), "checkout");
        assert_eq!(out.value(1), "");
    }

    // --- FixedSizeBinary -> hex string cast tests ---

    use arrow::array::FixedSizeBinaryArray;

    #[test]
    fn cast_fixed_binary_16_to_hex_string() {
        // 16-byte trace ID → 32-char lowercase hex
        let bytes: Vec<Option<&[u8]>> = vec![
            Some(&[
                0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82, 0x93, 0xa4, 0xb5, 0xc6, 0xd7,
                0xe8, 0xf9,
            ]),
            Some(&[0x00; 16]),
            Some(&[0xff; 16]),
        ];
        let fsb =
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(bytes.into_iter(), 16).unwrap();
        let arr: ArrayRef = Arc::new(fsb);

        let out = fixed_binary_to_hex_string(&arr).unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(out.len(), 3);
        assert_eq!(out.value(0), "0a1b2c3d4e5f60718293a4b5c6d7e8f9");
        assert_eq!(out.value(1), "00000000000000000000000000000000");
        assert_eq!(out.value(2), "ffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn cast_fixed_binary_8_to_hex_string() {
        // 8-byte span ID → 16-char lowercase hex
        let bytes: Vec<Option<&[u8]>> = vec![
            Some(&[0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe]),
            Some(&[0x00; 8]),
        ];
        let fsb =
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(bytes.into_iter(), 8).unwrap();
        let arr: ArrayRef = Arc::new(fsb);

        let out = fixed_binary_to_hex_string(&arr).unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(out.len(), 2);
        assert_eq!(out.value(0), "deadbeefcafebabe");
        assert_eq!(out.value(1), "0000000000000000");
    }

    #[test]
    fn cast_fixed_binary_null_handling() {
        let bytes: Vec<Option<&[u8]>> = vec![
            Some(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            None,
            Some(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11]),
        ];
        let fsb =
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(bytes.into_iter(), 8).unwrap();
        let arr: ArrayRef = Arc::new(fsb);

        let out = fixed_binary_to_hex_string(&arr).unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(out.len(), 3);
        assert_eq!(out.value(0), "0102030405060708");
        assert!(out.is_null(1));
        assert_eq!(out.value(2), "aabbccddeeff0011");
    }

    #[test]
    fn cast_fixed_binary_dict_encoded_to_hex_string() {
        // Build a dictionary-encoded FixedSizeBinary(8) array.
        let values = FixedSizeBinaryArray::try_from_sparse_iter_with_size(
            vec![
                Some(&[0xaa_u8, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11] as &[u8]),
                Some(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
            ]
            .into_iter(),
            8,
        )
        .unwrap();

        let keys = PrimitiveArray::<UInt8Type>::from(vec![Some(1u8), Some(0u8), None, Some(0u8)]);
        let dict = DictionaryArray::try_new(keys, Arc::new(values)).unwrap();
        let arr: ArrayRef = Arc::new(dict);

        let out = fixed_binary_to_hex_string(&arr).unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(out.len(), 4);
        assert_eq!(out.value(0), "0102030405060708"); // key 1 -> values[1]
        assert_eq!(out.value(1), "aabbccddeeff0011"); // key 0 -> values[0]
        assert!(out.is_null(2)); // null key
        assert_eq!(out.value(3), "aabbccddeeff0011"); // key 0 -> values[0]
    }

    #[test]
    fn cast_and_rename_via_apply_one_op_with_string_target() {
        let bytes: Vec<Option<&[u8]>> =
            vec![Some(&[0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe])];
        let fsb =
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(bytes.into_iter(), 8).unwrap();

        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("span_id".into(), Arc::new(fsb));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);
        let mut current = "span_id".to_string();

        let op = ColumnTransformOp::CastAndRename("SpanId".to_string(), DataType::Utf8);
        let r = apply_one_op(&mut ctx, &mut current, &op).unwrap();
        assert!(matches!(r, ControlFlow::Continue(())));

        assert_eq!(current, "SpanId");
        assert!(!ctx.columns.contains_key("span_id"));

        let out = ctx.columns["SpanId"]
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(out.value(0), "deadbeefcafebabe");
    }

    // remap_array_column_tests

    fn make_map_array_two_rows() -> MapArray {
        let keys = StringBuilder::new();
        let vals = StringBuilder::new();
        let mut b = MapBuilder::new(None, keys, vals);

        // row0: {"k":"v"}
        b.keys().append_value("k");
        b.values().append_value("v");
        b.append(true).unwrap();

        // row1: {"a":"b","c":"d"}
        b.keys().append_value("a");
        b.values().append_value("b");
        b.keys().append_value("c");
        b.values().append_value("d");
        b.append(true).unwrap();

        b.finish()
    }

    #[test]
    fn remap_array_column_reorders_rows() {
        // "current_indexes" are old ids; remap maps old id -> src_row in values
        let parent_ids = UInt16Array::from(vec![10u16, 11u16]);

        // swap: old 10 should take row1, old 11 should take row0
        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 1);
        remap.insert(11, 0);

        let values = make_map_array_two_rows();
        let out = remap_map_array_to_parent_order(&parent_ids, &remap, &values).unwrap();

        let keys = out.keys().as_any().downcast_ref::<StringArray>().unwrap();
        let vals = out.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offsets = out.offsets();

        // row0 should match original row1 (2 entries)
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 2);
        assert_eq!(keys.value(0), "a");
        assert_eq!(vals.value(0), "b");
        assert_eq!(keys.value(1), "c");
        assert_eq!(vals.value(1), "d");

        // row1 should match original row0 (1 entry)
        assert_eq!(offsets[1], 2);
        assert_eq!(offsets[2], 3);
        assert_eq!(keys.value(2), "k");
        assert_eq!(vals.value(2), "v");
    }

    #[test]
    fn remap_array_column_null_parent_id_produces_empty_map_row() {
        let parent_ids = UInt16Array::from(vec![Some(10u16), None, Some(11u16)]);

        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 0);
        remap.insert(11, 1);

        let values = make_map_array_two_rows();
        let out = remap_map_array_to_parent_order(&parent_ids, &remap, &values).unwrap();

        let offsets = out.offsets();

        // row0: takes row0 => 1 item => offsets[1]=1
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 1);

        // row1: NULL parent => empty map => offsets doesn't advance
        assert_eq!(offsets[1], 1);
        assert_eq!(offsets[2], 1);

        // row2: takes row1 => 2 items => offsets[3]=3
        assert_eq!(offsets[2], 1);
        assert_eq!(offsets[3], 3);
    }

    #[test]
    fn remap_array_column_unknown_parent_id_produces_empty_map_row() {
        let parent_ids = UInt16Array::from(vec![10u16, 99u16]); // 99 missing

        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 0);

        let values = make_map_array_two_rows();

        let out = remap_map_array_to_parent_order(&parent_ids, &remap, &values).unwrap();
        let offsets = out.offsets();

        // row0 maps to compact row0 => 1 item
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 1);

        // row1 has no remap entry => empty map
        assert_eq!(offsets[1], 1);
        assert_eq!(offsets[2], 1);
    }

    // ── EnumToString tests ──────────────────────────────────────────────

    #[test]
    fn enum_to_string_span_kind_plain_int32() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![
            Some(0), // SPAN_KIND_UNSPECIFIED
            Some(1), // SPAN_KIND_INTERNAL
            Some(2), // SPAN_KIND_SERVER
            Some(3), // SPAN_KIND_CLIENT
            Some(4), // SPAN_KIND_PRODUCER
            Some(5), // SPAN_KIND_CONSUMER
            None,    // null
        ]));

        let result = enum_to_string_array(&arr, &EnumStringMapper::SpanKind).unwrap();
        let strings = result.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(strings.value(0), "SPAN_KIND_UNSPECIFIED");
        assert_eq!(strings.value(1), "SPAN_KIND_INTERNAL");
        assert_eq!(strings.value(2), "SPAN_KIND_SERVER");
        assert_eq!(strings.value(3), "SPAN_KIND_CLIENT");
        assert_eq!(strings.value(4), "SPAN_KIND_PRODUCER");
        assert_eq!(strings.value(5), "SPAN_KIND_CONSUMER");
        assert!(strings.is_null(6));
    }

    #[test]
    fn enum_to_string_status_code_plain_int32() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![
            Some(0), // STATUS_CODE_UNSET
            Some(1), // STATUS_CODE_OK
            Some(2), // STATUS_CODE_ERROR
            None,    // null
        ]));

        let result = enum_to_string_array(&arr, &EnumStringMapper::StatusCode).unwrap();
        let strings = result.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(strings.value(0), "STATUS_CODE_UNSET");
        assert_eq!(strings.value(1), "STATUS_CODE_OK");
        assert_eq!(strings.value(2), "STATUS_CODE_ERROR");
        assert!(strings.is_null(3));
    }

    #[test]
    fn enum_to_string_unknown_value_produces_empty_string() {
        let arr: ArrayRef = Arc::new(Int32Array::from(vec![Some(99), Some(-1)]));

        let result = enum_to_string_array(&arr, &EnumStringMapper::SpanKind).unwrap();
        let strings = result.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(strings.value(0), "");
        assert_eq!(strings.value(1), "");
    }

    #[test]
    fn enum_to_string_dict_encoded_uint8_key() {
        use arrow::array::PrimitiveDictionaryBuilder;

        let mut builder = PrimitiveDictionaryBuilder::<UInt8Type, Int32Type>::new();
        builder.append(2_i32).unwrap(); // SPAN_KIND_SERVER
        builder.append(3_i32).unwrap(); // SPAN_KIND_CLIENT
        builder.append(2_i32).unwrap(); // SPAN_KIND_SERVER (reuse dict entry)
        builder.append_null();
        let arr: ArrayRef = Arc::new(builder.finish());

        let result = enum_to_string_array(&arr, &EnumStringMapper::SpanKind).unwrap();
        let strings = result.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(strings.value(0), "SPAN_KIND_SERVER");
        assert_eq!(strings.value(1), "SPAN_KIND_CLIENT");
        assert_eq!(strings.value(2), "SPAN_KIND_SERVER");
        assert!(strings.is_null(3));
    }

    #[test]
    fn enum_to_string_dict_encoded_uint16_key() {
        use arrow::array::PrimitiveDictionaryBuilder;

        let mut builder = PrimitiveDictionaryBuilder::<UInt16Type, Int32Type>::new();
        builder.append(1_i32).unwrap(); // STATUS_CODE_OK
        builder.append(2_i32).unwrap(); // STATUS_CODE_ERROR
        builder.append_null();
        let arr: ArrayRef = Arc::new(builder.finish());

        let result = enum_to_string_array(&arr, &EnumStringMapper::StatusCode).unwrap();
        let strings = result.as_any().downcast_ref::<StringArray>().unwrap();

        assert_eq!(strings.value(0), "STATUS_CODE_OK");
        assert_eq!(strings.value(1), "STATUS_CODE_ERROR");
        assert!(strings.is_null(2));
    }

    #[test]
    fn enum_to_string_rejects_wrong_type() {
        let arr: ArrayRef = Arc::new(StringArray::from(vec!["not", "an", "int"]));
        let err = enum_to_string_array(&arr, &EnumStringMapper::SpanKind).unwrap_err();
        match err {
            ClickhouseExporterError::InvalidColumnType { .. } => {}
            other => panic!("expected InvalidColumnType, got {other:?}"),
        }
    }

    #[test]
    fn enum_to_string_via_apply_one_op() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert(
            "kind".into(),
            Arc::new(Int32Array::from(vec![Some(2), Some(0)])),
        );

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let mut current = "kind".to_string();
        let op = ColumnTransformOp::EnumToString {
            output_name: "SpanKind".to_string(),
            mapper: EnumStringMapper::SpanKind,
        };

        let r = apply_one_op(&mut ctx, &mut current, &op).unwrap();
        assert!(matches!(r, ControlFlow::Continue(())));

        assert!(!ctx.columns.contains_key("kind"));
        assert!(ctx.columns.contains_key("SpanKind"));
        assert_eq!(current, "SpanKind");

        let out = ctx.columns["SpanKind"]
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(out.value(0), "SPAN_KIND_SERVER");
        assert_eq!(out.value(1), "SPAN_KIND_UNSPECIFIED");
    }

    #[test]
    fn noop_leaves_column_unchanged() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("col".into(), Arc::new(UInt32Array::from(vec![1, 2, 3])));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let mut current = "col".to_string();
        let r = apply_one_op(&mut ctx, &mut current, &ColumnTransformOp::NoOp).unwrap();
        assert!(matches!(r, ControlFlow::Continue(())));
        assert!(ctx.columns.contains_key("col"));
        assert_eq!(current, "col");
    }

    #[test]
    fn drop_removes_column_and_returns_break() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("col".into(), Arc::new(UInt32Array::from(vec![1, 2, 3])));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let mut current = "col".to_string();
        let r = apply_one_op(&mut ctx, &mut current, &ColumnTransformOp::Drop).unwrap();
        assert!(matches!(r, ControlFlow::Break(())));
        assert!(!ctx.columns.contains_key("col"));
    }

    #[test]
    fn flatten_struct_keeps_struct_col_when_remove_is_false() {
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("s".into(), struct_body_array());

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut columns, &multi);

        let spec = FlattenStructSpec {
            field_mapping: vec![("a".to_string(), "a_out".to_string())]
                .into_iter()
                .collect(),
            remove_struct_col: false,
        };

        flatten_struct(&mut ctx, "s", &spec).unwrap();

        // struct is NOT removed
        assert!(ctx.columns.contains_key("s"));
        // child is also present
        assert!(ctx.columns.contains_key("a_out"));
    }

    #[test]
    fn append_cbor_as_json_simple_map() {
        let cbor_bytes = serde_cbor::to_vec(&serde_json::json!({"key": "value"})).unwrap();
        let mut buf = Vec::new();
        append_cbor_as_json(&mut buf, &cbor_bytes).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn append_cbor_as_json_nested_structure() {
        let cbor_bytes =
            serde_cbor::to_vec(&serde_json::json!({"a": [1, 2, 3], "b": true})).unwrap();
        let mut buf = Vec::new();
        append_cbor_as_json(&mut buf, &cbor_bytes).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["a"], serde_json::json!([1, 2, 3]));
        assert_eq!(parsed["b"], serde_json::json!(true));
    }

    #[test]
    fn append_cbor_as_json_empty_object() {
        let cbor_bytes = serde_cbor::to_vec(&serde_json::json!({})).unwrap();
        let mut buf = Vec::new();
        append_cbor_as_json(&mut buf, &cbor_bytes).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed, serde_json::json!({}));
    }

    #[test]
    fn append_cbor_as_json_invalid_cbor_errors() {
        let mut buf = Vec::new();
        let result = append_cbor_as_json(&mut buf, &[0xFF, 0xFE, 0xFD]);
        assert!(result.is_err());
    }

    #[test]
    fn inline_attribute_missing_child_payload_drops_id_column() {
        let mut parent_cols: HashMap<String, ArrayRef> = HashMap::new();
        parent_cols.insert("attr_id".into(), u16_array(&[10, 11]));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut parent_cols, &multi);

        let mut current = "attr_id".to_string();
        inline_attribute(&mut ctx, &mut current, ArrowPayloadType::ResourceAttrs).unwrap();

        // The foreign-key id column must be DROPPED when the child attribute payload is absent —
        // it is not a ClickHouse column, and leaking it breaks the bind-by-name INSERT.
        assert!(!ctx.columns.contains_key("attr_id"));
        // No attribute column is emitted either; ClickHouse fills the default empty map.
        assert!(!ctx.columns.contains_key(ch_consts::CH_RESOURCE_ATTRIBUTES));
    }
}
