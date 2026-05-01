//! Column-level Arrow transformations used to produce ClickHouse-ready batches.
//!
//! This module implements the execution of per-column transformation operations over Arrow arrays
//! during the OTAP → ClickHouse export pipeline. It operates on a mutable map of output columns and
//! can reference results from “child” payload transformations (e.g., attribute payloads) to rewrite
//! IDs and/or inline child data into parent signal batches.
//!
//! Key pieces:
//!
//! - [`ColumnOpCtx`]: an operation context containing the current column set being built and a map
//!   of multi-payload transformation outputs (used for joins/remaps between parent and child
//!   payloads).
//!
//! - `apply_one_op`: dispatcher for [`ColumnTransformOp`] that applies a single transformation to
//!   the “current” column, supporting operations such as renaming, casting, adding offsets,
//!   dropping columns, flattening struct fields, adding synthetic columns (partition UUID bytes and
//!   insert timestamp), coercing complex body values into strings, reindexing lookup IDs, and
//!   inlining child payload data.
//!
//! - Struct and value coercions:
//!   - `flatten_struct` lifts selected `StructArray` fields into top-level columns.
//!   - `coerce_body_to_string` / `struct_column_to_string` converts OTLP/OTAP “AnyValue”-style
//!     structs into a single string column, including base64 encoding for bytes and CBOR→JSON
//!     transcoding for map/slice variants.
//!
//! - Parent/child inlining and remapping:
//!   - `reindex_attribute` rewrites parent-side foreign keys using a child payload’s remap table.
//!   - `inline_attribute` inlines attributes into the parent batch as either JSON (dictionary-encoded
//!     binary) or `Map(LowCardinality(String), String)`, depending on [`AttributeRepresentation`].
//!   - `inline_child_lists` and `inline_child_map` expand compact child arrays (indexed by ID) into
//!     parent-sized arrays, driven by parent IDs and a remap table.
//!
//! - Array expansion helpers:
//!   - `expand_list_array_to_parent_size` expands a compact `ListArray` into parent row cardinality,
//!     with null propagation and safe bounds handling.
//!   - `remap_array_column` and `remap_dict_keys` rebuild map/dictionary keys to align with remapped
//!     IDs (including handling of null IDs by appending a synthetic null/default entry).
//!
//! The output of these operations is a set of Arrow columns shaped and typed to match the
//! ClickHouse schema chosen by configuration (inline vs lookup attributes, JSON vs map encoding,
//! etc.).
use std::collections::HashMap;
use std::sync::Arc;

use std::ops::ControlFlow;

use arrow::array::{
    Array, ArrayBuilder, ArrayRef, BinaryArray, BinaryDictionaryBuilder, Int32Array, ListBuilder,
    MapBuilder, PrimitiveBuilder, StringBuilder, UInt8Array, make_builder,
};
use arrow::compute::{cast, max};
use arrow::datatypes::{DataType, Float64Type, UInt8Type, UInt16Type, UInt32Type};
use arrow_array::{
    DictionaryArray, ListArray, MapArray, PrimitiveArray, StringArray, StructArray, UInt16Array,
};
use base64::Engine;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::trace::v1::{
    span::SpanKind, status::StatusCode as SpanStatusCode,
};
use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

use crate::clickhouse_exporter::arrays::{NullableArrayAccessor, StructColumnAccessor};

use crate::clickhouse_exporter::config::AttributeRepresentation;
use crate::clickhouse_exporter::consts as ch_consts;
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::transform::transform_batch::{
    MultiColumnOpResult, append_list_value,
};
use crate::clickhouse_exporter::transform::transform_plan::{
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

/// Applies a single [`ColumnTransformOp`] to the column currently being processed
/// from the input [`RecordBatch`].
///
/// ## When This Is Called
///
/// This function is invoked during iteration over the *existing* columns of the
/// original input batch. As a result:
///
/// - Only columns physically present in the batch are processed.
/// - If a column has configured operations but is not present in the batch,
///   this function will never be called for it.
/// - Multi-column or synthetic operations must have been handled earlier in the
///   pipeline.
///
/// ## Context (`ctx`)
///
/// `ctx` represents the mutable working set of columns for the in-flight output
/// batch. It contains:
///
/// - The current state of all columns after previously applied transforms
/// - Results of any prior multi-column operations
/// - Temporary or derived columns inserted earlier in the pipeline
///
/// Most operations follow the pattern:
///
/// 1. `take(current_name)` — remove the column from the working set
/// 2. transform the underlying array
/// 3. `put(name, array)` — reinsert the transformed column
///
/// This ensures there is never more than one active version of a column in the
/// working set at a time.
///
/// ## `current_name`
///
/// `current_name` tracks the logical name of the column as it evolves through
/// transforms (e.g., after a rename). It must be kept in sync with the working
/// set to ensure subsequent operations reference the correct column.
///
/// ## Control Flow
///
/// The return type is `ControlFlow<()>`:
///
/// - `Continue(())` → continue applying additional operations to this column
/// - `Break(())` → stop applying further operations to this column
///
/// `Break` is typically returned by destructive operations such as `Drop`,
/// since no further transforms can be meaningfully applied.
///
/// ## Error Handling
///
/// Any failure in array extraction, casting, arithmetic, or derived column
/// creation results in a `ClickhouseExporterError` and aborts processing of the
/// current batch.
///
/// ## Invariants
///
/// - A column must exist in `ctx` before being transformed.
/// - After a successful non-`Drop` operation, the column must be present in `ctx`.
/// - `current_name` must always reflect the column’s latest name.
/// - No operation should leave the working set in a partially updated state.
///
/// This function forms the core primitive for per-column transformation in the
/// export pipeline and is intentionally side-effectful on `ctx`.
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

        ColumnTransformOp::InlineAttribute(child_pt, repr) => {
            inline_attribute(ctx, current_name, *child_pt, repr)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::InlineChildLists(child_pt) => {
            inline_child_lists(ctx, current_name, *child_pt)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::InlineChildMap(child_pt, from, to) => {
            inline_child_map(ctx, current_name, *child_pt, from, to)?;
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
fn fixed_binary_to_hex_string(arr: &ArrayRef) -> Result<ArrayRef, ClickhouseExporterError> {
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

    let byte_width = fsb.value_length() as usize;
    // Each byte becomes 2 hex chars.
    let mut builder = StringBuilder::with_capacity(fsb.len(), fsb.len() * byte_width * 2);

    for i in 0..fsb.len() {
        if fsb.is_null(i) {
            builder.append_null();
        } else {
            let bytes = fsb.value(i);
            let mut hex = String::with_capacity(byte_width * 2);
            for b in bytes {
                use std::fmt::Write;
                write!(hex, "{:02x}", b).expect("writing to String cannot fail");
            }
            builder.append_value(&hex);
        }
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
        DataType::Dictionary(_, value_type) if **value_type == DataType::Binary => {
            extract_map_value_from_json_dict(arr, key, default_value)?
        }
        _ => {
            return Err(ClickhouseExporterError::InvalidColumnType {
                name: current_name.to_string(),
                expected: "MapArray or Dictionary<*, Binary>".into(),
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

fn extract_map_value_from_json_dict(
    arr: &ArrayRef,
    key: &str,
    default_value: &str,
) -> Result<ArrayRef, ClickhouseExporterError> {
    let dict = arr
        .as_any()
        .downcast_ref::<DictionaryArray<UInt32Type>>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast attributes to DictionaryArray<UInt32Type>".into(),
        })?;
    let values = dict
        .values()
        .as_any()
        .downcast_ref::<BinaryArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast dictionary values to BinaryArray".into(),
        })?;

    let mut builder = StringBuilder::with_capacity(dict.len(), dict.len() * default_value.len());

    for row in 0..dict.len() {
        if dict.is_null(row) {
            builder.append_value(default_value);
            continue;
        }

        let dict_key = dict.keys().value(row) as usize;
        let bytes = values.value(dict_key);
        let extracted = serde_json::from_slice::<serde_json::Value>(bytes)
            .ok()
            .and_then(|json| json.get(key).cloned())
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| default_value.to_string());
        builder.append_value(extracted);
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
        // TODO: [Correctness] per @alockett:
        // In practice, what would happen here is that if a log/span had a resource/scope that was null,
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

/// Take a compact MapArray which may contain values for a given row in the parent batch,
/// expand it to the same size as the parent batch, and re-order the entries to match the parent row order.
/// This is used primarily for inlining SpanEventAttributes and SpanLinkAttributes to the Span signal batch.
fn inline_child_map(
    ctx: &mut ColumnOpCtx<'_>,
    parent_id_col: &str,
    child_payload: ArrowPayloadType,
    from_col: &str,
    to_col: &str,
) -> Result<(), ClickhouseExporterError> {
    let Some((parent_ids, remap, child)) =
        parent_ids_and_child_remap_owned(ctx, parent_id_col, child_payload)?
    else {
        return Ok(());
    };

    let values_arr =
        child
            .columns
            .get(from_col)
            .ok_or_else(|| ClickhouseExporterError::MissingColumn {
                name: from_col.to_string(),
            })?;
    let values = values_arr
        .as_any()
        .downcast_ref::<MapArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast child map column to MapArray".into(),
        })?;

    let new_map_array = remap_map_array_to_parent_order(&parent_ids, &remap, values)?;
    ctx.put(to_col.to_string(), Arc::new(new_map_array));
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
        parent_ids_and_child_remap_owned(ctx, parent_id_col, child_payload)?
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

        let expanded = remap_list_array_to_parent_order(&parent_ids, &remap, compact_list)?;
        ctx.put(name.clone(), expanded);
    }
    Ok(())
}
type Remap = Arc<HashMap<u32, u32>>;

fn parent_ids_and_child_remap_owned(
    ctx: &ColumnOpCtx<'_>,
    parent_id_col: &str,
    child_payload: ArrowPayloadType,
) -> Result<
    Option<(Arc<PrimitiveArray<UInt16Type>>, Remap, MultiColumnOpResult)>,
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

    let Some(child) = ctx.child(child_payload) else {
        return Ok(None);
    };

    let remap =
        child
            .remapped_ids
            .as_ref()
            .ok_or_else(|| ClickhouseExporterError::CoercionError {
                error: "Failed to find child batch inline key map".into(),
            })?;
    let remap = Arc::new(remap.clone()); // clone the HashMap so we can release the borrow

    // Also clone the child result you need so we can drop the immutable borrow of ctx.multi.
    Ok(Some((parent_ids, remap, child.clone())))
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
    // output list builder has same element type as compact.values()
    let values_builder: Box<dyn ArrayBuilder> = make_builder(compact.values().data_type(), 0);
    let mut out: ListBuilder<Box<dyn ArrayBuilder>> = ListBuilder::new(values_builder);

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

    let keys_builder = StringBuilder::new();
    let values_builder = StringBuilder::new();
    let mut map_builder = MapBuilder::new(None, keys_builder, values_builder);

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

/// Remap/reorder a compact `DictionaryArray<UInt32Type>` into parent row order.
///
/// Given:
/// - `parent_ids`: the per-parent-row id array (length = number of parent rows),
/// - `old_to_new`: a mapping from parent id (`u32`) -> row index in the compact child result,
/// - `compact`: a dictionary-encoded child column whose rows are in “compact” (deduplicated) order,
///
/// this function produces a new dictionary array whose **length matches `parent_ids.len()`** and
/// whose keys are reordered/expanded so that each parent row points at the dictionary key from the
/// corresponding compact row.
///
/// For each parent row `i`:
/// - if `parent_ids[i]` is null, the output row is null
/// - else if `old_to_new` contains no entry for `parent_ids[i]`, the output row is null
/// - else let `j = old_to_new[parent_ids[i]]`; the output key is `compact.keys()[j]` (or null if
///   `j` is out of bounds or `compact.keys()[j]` is null)
///
/// The returned array reuses `compact.values()` (dictionary values) and only rebuilds the key array.
///
/// # Errors
/// Returns an error if constructing the output `DictionaryArray` fails (e.g. due to Arrow
/// invariants/type mismatches).
pub(crate) fn remap_dict_array_to_parent_order(
    parent_ids: &PrimitiveArray<UInt16Type>,
    old_to_new: &HashMap<u32, u32>,
    compact: &DictionaryArray<UInt32Type>,
) -> Result<ArrayRef, ClickhouseExporterError> {
    // Keep the same dictionary values; only expand/remap the keys to parent size.
    let values = compact.values().clone();

    // DictArray key type is assumed to be u32 based on old_to_new signature.
    let compact_keys = compact.keys();

    // Build new keys of length == parent_ids.len(), inserting nulls when mapping missing or input null.
    let mut key_builder = PrimitiveBuilder::<UInt32Type>::with_capacity(parent_ids.len());

    for i in 0..parent_ids.len() {
        if parent_ids.is_null(i) {
            key_builder.append_null();
            continue;
        }

        let pid = parent_ids.value(i) as u32;

        match old_to_new.get(&pid) {
            None => key_builder.append_null(),
            Some(&new_row) => {
                // new_row refers to a row in `compact`; pick the compact key for that row.
                if (new_row as usize) >= compact_keys.len()
                    || compact_keys.is_null(new_row as usize)
                {
                    key_builder.append_null();
                } else {
                    key_builder.append_value(compact_keys.value(new_row as usize));
                }
            }
        }
    }

    let new_keys = key_builder.finish();

    // Recreate a dict array with expanded keys and same values.
    let expanded = DictionaryArray::try_new(new_keys, values)?;
    Ok(Arc::new(expanded))
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

    // TODO: [Optimization] per @a.lockett:
    // As a future optimization: most of the time the str, int, bytes, and ser column will likely be dictionary encoded.
    // When this is the case, we could basically just map the dictionary array's values to a stringified representation,
    // and then append them all to create new dictionary values. Then for each of these columns, we could just adjust the keys.
    // more details: https://gitlab.com/f5/observabilityhub/o11y-gateway/observability-gateway/-/merge_requests/90#note_3083198934
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
                        let mut buf = vec![];
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
/// - The requested representation cannot be inlined (OTAP array), or if underlying coercion/expansion
///   fails.
///
/// # Side effects
/// - On success, removes the parent id column and inserts the inlined attribute column.
/// - On no-op (missing child payload), leaves the context unchanged.
/// - On `OtapArray` error, restores the id column before returning.
fn inline_attribute(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &mut String,
    child_payload_type: ArrowPayloadType,
    representation: &AttributeRepresentation,
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

    // Find child payload results; if missing, just no-op.
    let Some(result) = ctx.child(child_payload_type) else {
        // removed the id column; put it back to preserve semantics
        ctx.put(current_name.clone(), id_arr);
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

    let new_column: ArrayRef = match representation {
        AttributeRepresentation::Json => inline_attr_json(id_arr_u16, remap, values_arr)?,
        AttributeRepresentation::StringMap => {
            inline_attr_string_map(id_arr_u16, remap, values_arr)?
        }
    };

    ctx.put(new_name.clone(), new_column);
    *current_name = new_name;

    Ok(())
}

/// Build a new column of DictArray<UInt32Type> with the same values as the input dict, but
/// with keys array re-mapped to the ordering of the parent ID column with nulls inserted for
/// empty attribute values.
fn inline_attr_json(
    parent_ids: &UInt16Array,
    remap: &HashMap<u32, u32>,
    values_arr: &ArrayRef,
) -> Result<ArrayRef, ClickhouseExporterError> {
    let values = values_arr
        .as_any()
        .downcast_ref::<DictionaryArray<UInt32Type>>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "Failed to downcast attributes to binary array".into(),
        })?;
    remap_dict_array_to_parent_order(parent_ids, remap, values)
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
    use arrow_array::{DurationNanosecondArray, UInt32Array, UInt64Array};

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

    fn make_json_attr_dict(rows: &[Option<&[u8]>]) -> ArrayRef {
        let mut builder = BinaryDictionaryBuilder::<UInt32Type>::new();
        for row in rows {
            match row {
                Some(bytes) => builder.append_value(*bytes),
                None => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn struct_body_array() -> ArrayRef {
        // struct { a: Utf8, b: UInt32 }
        let a: ArrayRef = Arc::new(StringArray::from(vec![Some("x"), None, Some("z")]));
        let b: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2, 3]));

        let fields = vec![
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
        inline_attribute(
            &mut ctx,
            &mut current,
            ArrowPayloadType::ResourceAttrs,
            &AttributeRepresentation::StringMap,
        )
        .unwrap();

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
        inline_attribute(
            &mut ctx,
            &mut current,
            ArrowPayloadType::ResourceAttrs,
            &AttributeRepresentation::StringMap,
        )
        .unwrap();

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

    #[test]
    fn extract_service_name_from_json_dict_and_null_row() {
        let arr = make_json_attr_dict(&[
            Some(br#"{"service.name":"payments","k":"v"}"#),
            Some(br#"{"other":"value"}"#),
            None,
        ]);

        let out = extract_map_value_from_json_dict(&arr, "service.name", "").unwrap();
        let out = out.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(out.value(0), "payments");
        assert_eq!(out.value(1), "");
        assert_eq!(out.value(2), "");
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
}
