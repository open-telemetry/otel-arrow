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

use arrow::array::{Array, BinaryDictionaryBuilder, PrimitiveBuilder};
use arrow::array::{
    ArrayBuilder, ListBuilder, MapBuilder, StringBuilder, TimestampNanosecondBuilder, UInt8Array,
    make_builder,
};
use arrow::buffer::MutableBuffer;
use arrow::compute::kernels::numeric::add;

use arrow::array::ArrayRef;
use arrow::compute::{cast, max};
use arrow::datatypes::{Float64Type, UInt8Type, UInt16Type, UInt32Type};
use arrow_array::{
    DictionaryArray, FixedSizeBinaryArray, ListArray, MapArray, PrimitiveArray, StringArray,
    StructArray, UInt16Array, UInt32Array,
};
use base64::Engine;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::{otlp::attributes::AttributeValueType, schema::consts};

use crate::clickhouse_exporter::arrays::{NullableArrayAccessor, StructColumnAccessor};

use crate::clickhouse_exporter::config::AttributeRepresentation;
use crate::clickhouse_exporter::consts as ch_consts;
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::transform::transform_batch::{
    MultiColumnOpResult, append_list_value,
};
use crate::clickhouse_exporter::transform::transform_plan::{
    CoerceStructStringSpec, ColumnTransformOp, FlattenStructSpec,
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

    fn len_of(&self, name: &str) -> Result<usize, ClickhouseExporterError> {
        Ok(self.get(name)?.len())
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

        ColumnTransformOp::CastTo(new_type) => {
            let arr = ctx.take(current_name)?;
            let arr = if arr.data_type() == new_type {
                arr
            } else {
                cast(&arr, new_type)?
            };
            ctx.put(current_name.clone(), arr);
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::AddOffset(offset) => {
            let arr = ctx.take(current_name)?;
            let new_column = add(&UInt32Array::new_scalar(*offset), &arr)?;
            ctx.put(current_name.clone(), new_column);
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

        ColumnTransformOp::AddUUIDBytesColumn(bytes) => {
            add_uuid_col(ctx, current_name, bytes)?;
            Ok(ControlFlow::Continue(()))
        }

        ColumnTransformOp::AddInsertTimestampColumn(ts) => {
            add_insert_ts_col(ctx, current_name, *ts)?;
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
    }
}

// Add a new column with the current 'partition' uuid (from idgen).
fn add_uuid_col(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &str,
    bytes: &[u8; 32],
) -> Result<(), ClickhouseExporterError> {
    let len = ctx.len_of(current_name)?;
    let mut buffer = MutableBuffer::with_capacity(len * 32);
    buffer.repeat_slice_n_times(bytes, len);
    let uuid_col = FixedSizeBinaryArray::new(32, buffer.into(), None);
    ctx.put(ch_consts::PART_ID, Arc::new(uuid_col) as ArrayRef);

    Ok(())
}

/// insert_time is used on some versions of some clickhouse tables for indexing / ttl calculations. This inserts
/// a static timestamp value (calculated at idgen time) into the column context.
fn add_insert_ts_col(
    ctx: &mut ColumnOpCtx<'_>,
    current_name: &str,
    ts: i64,
) -> Result<(), ClickhouseExporterError> {
    let len = ctx.len_of(current_name)?;
    let mut builder = TimestampNanosecondBuilder::with_capacity(len);
    builder.append_value_n(ts, len);
    ctx.put(
        ch_consts::INSERT_TIME,
        Arc::new(builder.finish()) as ArrayRef,
    );
    Ok(())
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

        let src_row = *old_to_new.get(&(parent_ids.value(i) as u32)).ok_or(
            ClickhouseExporterError::CoercionError {
                error: format!("Unknown src index while remapping array values: {}", i),
            },
        )? as usize;

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
        AttributeRepresentation::OtapArray => {
            // put back the id column since we already took it
            ctx.put(current_name.clone(), id_arr);
            return Err(ClickhouseExporterError::CoercionError {
                error: "Otap Array can't be inline".into(),
            });
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
}

#[cfg(test)]
mod remap_array_column_tests {
    #![allow(unused_results)]
    use super::*;
    use std::collections::HashMap;

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
    fn remap_array_column_errors_on_unknown_parent_id() {
        let parent_ids = UInt16Array::from(vec![10u16, 99u16]); // 99 missing

        let mut remap: HashMap<u32, u32> = HashMap::new();
        remap.insert(10, 0);

        let values = make_map_array_two_rows();

        let err = remap_map_array_to_parent_order(&parent_ids, &remap, &values).unwrap_err();
        match err {
            ClickhouseExporterError::CoercionError { .. } => {}
            other => panic!("expected CoercionError, got {other:?}"),
        }
    }
}
