//! RecordBatch transformation runner for OTAP → ClickHouse shaping.
//!
//! This module executes transformation plans that convert OTAP Arrow `RecordBatch`es into
//! ClickHouse-ready batches. It supports both multi-column operations (that need to look at or
//! restructure multiple columns together) and per-column operations (rename/cast/drop/inline, etc.)
//! while coordinating cross-payload dependencies such as attribute-table ID remapping.
//!
//! Key components:
//!
//! - [`BatchTransformer`]: the main orchestrator. It holds per-payload [`TransformationPlan`]s (built
//!   from exporter [`Config`]).
//!
//! - [`MultiColumnOpResult`]: the intermediate representation produced by the multi-column stage,
//!   containing the current column map plus an optional `old_id -> new_id` remap table used later to
//!   reindex foreign keys in parent payloads.
//!
//! Execution model:
//!
//! - **Multi-column stage** (`run_multi_column_stage`): runs `MultiColumnTransformOp`s that may:
//!   - drop all columns,
//!   - extract/group fields by an ID into list-typed columns (`build_list_arrays`),
//!   - normalize/dedupe OTLP attributes into either a string map or a JSON/dictionary form while
//!     producing an ID remap for downstream joins.
//!
//! - **Single-column stage** (`apply_column_ops`): applies `ColumnTransformOp`s to each original
//!   column name using [`apply_one_op`], allowing operations to rewrite, drop, or replace columns
//!   and to reference other payload results via the shared multi-column result map.
//!   Optionally recreates a new `RecordBatch` with a deterministic column order.
//!
//! Supporting helpers include:
//!
//! - `append_list_value`: type-aware appending into list/map builders when constructing grouped list
//!   arrays.
//! - `dictionary_to_key_value_columns`: utility for exposing dictionary-encoded values alongside
//!   their integer keys when producing normalized attribute tables.
//!
//! Together, these utilities implement the “shape the data to match the chosen ClickHouse schema”
//! step, including attribute normalization, ID remapping, and column-level coercions needed by the
//! exporter’s table layouts.
use std::ops::ControlFlow;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use arrow::array::{
    make_builder, Array, ArrayBuilder, ListBuilder, MapBuilder, PrimitiveArray, StringBuilder,
    TimestampNanosecondBuilder, UInt32Builder,
};
use arrow::datatypes::UInt16Type;
use arrow::{
    array::{ArrayRef, RecordBatch},
    datatypes::{Field, Schema},
};
use arrow_array::builder::FixedSizeBinaryBuilder;
use arrow_array::{MapArray, StringArray, TimestampNanosecondArray, UInt32Array};
use arrow_schema::{DataType, TimeUnit};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;
use otap_df_pdata::OtapArrowRecords;

use crate::clickhouse_exporter::arrays::get_u16_array_opt;
use crate::clickhouse_exporter::config::Config;
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::transform::build_payload_transform_map;
use crate::clickhouse_exporter::transform::transform_attributes::{
    group_attributes_to_json_ser, group_attributes_to_map_str, group_rows_by_id,
};
use crate::clickhouse_exporter::transform::transform_column::{apply_one_op, ColumnOpCtx};
use crate::clickhouse_exporter::transform::transform_plan::{
    ColumnOperations, ColumnTransformOp, MultiColumnTransformOp, TransformationPlan,
};

#[derive(Clone, Debug)]
pub(crate) struct MultiColumnOpResult {
    /// Map of column name -> ArrayRef
    pub columns: HashMap<String, ArrayRef>,
    /// Optional mapping of old parent_id -> new id value
    pub remapped_ids: Option<HashMap<u32, u32>>,
}

/// Executes the transformation plan to produce a new RecordBatch.
pub struct BatchTransformer {
    payload_transform_plans: HashMap<ArrowPayloadType, TransformationPlan>,
}
impl BatchTransformer {
    /// Create a new batch transformer from static payload transformation plans.
    pub fn new_from_config(config: &Config) -> Self {
        let payload_transform_plans = build_payload_transform_map(config);
        Self {
            payload_transform_plans,
        }
    }

    /// This method is the orchestration layer for the transformation engine.
    /// For each allowed `ArrowPayloadType`, it executes:
    ///
    /// 1. **Multi-column transformation stage**
    /// 2. **Single-column transformation stage**
    /// 3. **Optional RecordBatch reconstruction**
    ///
    /// The result is a map of transformed `RecordBatch`es ready for export.
    ///
    /// ---
    ///
    /// # Stage 1: Multi-Column Transformations
    ///
    /// For each allowed payload type:
    ///
    /// - Retrieves the input `RecordBatch`
    /// - Looks up its static transformation plan
    /// - Executes `run_multi_column_stage`
    ///
    /// This stage may:
    ///
    /// - Restructure entire column sets
    /// - Aggregate or reshape rows
    /// - Produce ID remapping (`old_id -> new_id`)
    /// - Replace or clear columns entirely
    ///
    /// Results are stored in `multi_col_results`.
    ///
    /// ---
    ///
    /// # Stage 2: Single-Column Transformations
    ///
    /// For each payload type:
    ///
    /// - `apply_column_ops` is executed against the corresponding
    ///   `MultiColumnOpResult`.
    ///
    /// This stage performs:
    ///
    /// - Per-column transformations
    /// - Renames, drops, or rewrites
    /// - Deterministic batch reconstruction (if configured)
    ///
    /// If `plan.recreate_batch == true` and columns remain,
    /// a new `RecordBatch` is produced and inserted into
    /// `writable_batches`.
    ///
    /// # Returns
    ///
    /// Returns a `HashMap<ArrowPayloadType, RecordBatch>` containing
    /// only payloads that:
    ///
    /// - Had an input batch
    /// - Had a transformation plan
    /// - Still contain columns after transformation
    /// - Requested batch reconstruction
    ///
    /// Returns `Err(ClickhouseExporterError)` if:
    ///
    /// - Any multi-column stage fails
    /// - Any single-column stage fails
    pub fn apply_plan(
        &mut self,
        arrow_records: OtapArrowRecords,
    ) -> Result<HashMap<ArrowPayloadType, RecordBatch>, ClickhouseExporterError> {
        let mut writable_batches: HashMap<ArrowPayloadType, RecordBatch> = HashMap::new();
        let mut multi_col_results: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut payload_order: Vec<ArrowPayloadType> = arrow_records
            .allowed_payload_types()
            .iter()
            .copied()
            .collect();

        payload_order.sort_by_key(|pt| match pt {
            // In the single-column stage, `apply_column_ops` removes each payload's entry
            // from `multi_col_results` via `.remove()`. Child attribute payloads (e.g.
            // ResourceAttrs) drop all their columns and never reinsert the entry. Parent
            // signal payloads (Logs/Spans) need those child entries for InlineAttribute /
            // InlineChildLists ops, so they must run first.
            ArrowPayloadType::Logs | ArrowPayloadType::Spans => 0,
            _ => 1,
        });

        // Multi-column stage
        for &payload_type in &payload_order {
            let Some(rb) = arrow_records.get(payload_type) else {
                continue;
            };
            let Some(transform_plan) = self.payload_transform_plans.get(&payload_type) else {
                continue;
            };

            let results = run_multi_column_stage(rb, &transform_plan.multi_column_ops)?;
            _ = multi_col_results.insert(payload_type, results);
        }

        // Single-column stage
        for &payload_type in &payload_order {
            let Some(plan) = self.payload_transform_plans.get(&payload_type) else {
                continue;
            };

            if let Some(final_batch) = apply_column_ops(
                &mut multi_col_results,
                payload_type,
                &plan.column_ops,
                plan.recreate_batch,
            )? {
                _ = writable_batches.insert(payload_type, final_batch);
            }
        }

        Ok(writable_batches)
    }
}

pub(crate) fn append_list_value(
    builder: &mut dyn ArrayBuilder,
    array: &ArrayRef,
    index: usize,
) -> Result<(), ClickhouseExporterError> {
    macro_rules! downcast_append_primitive {
        ($builder_ty:ty, $array_ty:ty) => {{
            let b = builder
                .as_any_mut()
                .downcast_mut::<$builder_ty>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast array builder.".into(),
                })?;
            let a = array.as_any().downcast_ref::<$array_ty>().ok_or_else(|| {
                ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast target array.".into(),
                }
            })?;
            if a.is_null(index) {
                b.append_null();
                return Ok(());
            }
            b.append_value(a.value(index));
        }};
    }

    match array.data_type() {
        DataType::Utf8 => downcast_append_primitive!(StringBuilder, StringArray),
        DataType::Dictionary(_, value_type) if **value_type == DataType::Utf8 => {
            let casted = arrow::compute::cast(array, &DataType::Utf8)?;
            append_list_value(builder, &casted, index)?;
            return Ok(());
        }
        DataType::Dictionary(_, value_type)
            if matches!(**value_type, DataType::FixedSizeBinary(_)) =>
        {
            let casted = arrow::compute::cast(array, value_type)?;
            append_list_value(builder, &casted, index)?;
            return Ok(());
        }
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            downcast_append_primitive!(TimestampNanosecondBuilder, TimestampNanosecondArray)
        }
        DataType::FixedSizeBinary(_) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<FixedSizeBinaryBuilder>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast fixed-size-binary builder.".into(),
                })?;
            let a = array
                .as_any()
                .downcast_ref::<arrow_array::FixedSizeBinaryArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast target array as FixedSizeBinaryArray.".into(),
                })?;

            if a.is_null(index) {
                b.append_null();
                return Ok(());
            }

            _ = b.append_value(a.value(index));
            return Ok(());
        }
        DataType::UInt32 => {
            downcast_append_primitive!(UInt32Builder, UInt32Array)
        }
        DataType::Map(_, _) => {
            // Only supports Map<Utf8, Utf8>
            let b = builder
                .as_any_mut()
                .downcast_mut::<MapBuilder<StringBuilder, StringBuilder>>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast map builder (expected MapBuilder<StringBuilder, StringBuilder>)."
                        .into(),
                })?;

            let a = array.as_any().downcast_ref::<MapArray>().ok_or_else(|| {
                ClickhouseExporterError::CoercionError {
                    error: "Failed to downcast target array as MapArray.".into(),
                }
            })?;

            if a.is_null(index) {
                _ = b.append(false);
                return Ok(());
            }

            // Determine range of entries belonging to this row
            let offsets = a.value_offsets();
            let start = offsets[index] as usize;
            let end = offsets[index + 1] as usize;

            // entries is a StructArray with fields ("key", "value")
            let entries = a.entries();
            let key_arr = entries
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Map key array is not Utf8.".into(),
                })?;
            let val_arr = entries
                .column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| ClickhouseExporterError::CoercionError {
                    error: "Map value array is not Utf8.".into(),
                })?;

            // Append each (key,value) entry
            for j in start..end {
                // keys in Arrow Map are usually non-null, but we’ll be defensive
                if key_arr.is_null(j) {
                    return Err(ClickhouseExporterError::CoercionError {
                        error: "Map contains null key (unsupported).".into(),
                    });
                }
                b.keys().append_value(key_arr.value(j));

                if val_arr.is_null(j) {
                    b.values().append_null();
                } else {
                    b.values().append_value(val_arr.value(j));
                }
            }
            _ = b.append(true);
            return Ok(());
        }
        _ => unimplemented!("type not supported {}", array.data_type()),
    }

    Ok(())
}

/// Group column entries by a group_ids array, producing a new hash of column names to list array.
fn build_list_arrays(
    id_col: &PrimitiveArray<UInt16Type>,
    targets: &HashMap<String, ArrayRef>,
) -> Result<(HashMap<u32, u32>, HashMap<String, ArrayRef>), ClickhouseExporterError> {
    let groups = group_rows_by_id(id_col);

    let mut old_to_new_map: HashMap<u32, u32> = HashMap::with_capacity(groups.len());

    // Create a ListBuilder per target column
    let mut builders: HashMap<String, ListBuilder<Box<dyn ArrayBuilder>>> = targets
        .iter()
        .map(|(name, array)| {
            let value_type = match array.data_type() {
                DataType::Dictionary(_, value_type) if **value_type == DataType::Utf8 => {
                    DataType::Utf8
                }
                DataType::Dictionary(_, value_type)
                    if matches!(**value_type, DataType::FixedSizeBinary(_)) =>
                {
                    (**value_type).clone()
                }
                _ => array.data_type().clone(),
            };
            let builder = ListBuilder::new(make_builder(&value_type, 0));
            (name.clone(), builder)
        })
        .collect();

    // TODO: [Optimization] per @a.lockett:
    // As a future optimization, if we know the target arrays are already in order as the list array,
    // we could probably significantly improve how quickly these can be constructed.
    // Much more detail: https://gitlab.com/f5/observabilityhub/o11y-gateway/observability-gateway/-/merge_requests/90#note_3083164311
    for (i, (id, rows)) in groups.into_iter().enumerate() {
        for row_u16 in rows {
            let row = row_u16 as usize;
            // Append values for each column
            for (name, target) in targets {
                let builder = builders.get_mut(name).expect("builder must exist");

                append_list_value(builder.values(), target, row)?;
            }
        }
        // New group - close previous list on all builders
        for builder in builders.values_mut() {
            builder.append(true);
        }
        _ = old_to_new_map.insert(id as u32, i as u32);
    }

    // Finish builders into ArrayRefs
    let result = builders
        .into_iter()
        .map(|(name, mut builder)| (name, Arc::new(builder.finish()) as ArrayRef))
        .collect();

    Ok((old_to_new_map, result))
}

/// Executes a sequence of multi-column transformation operations over an
/// input `RecordBatch`, producing a new column set and optional ID remapping.
///
/// This function initializes a mutable `HashMap<String, ArrayRef>` from the
/// input `RecordBatch`, then applies each `MultiColumnTransformOp` in order.
/// Each operation may:
///
/// - Mutate or completely replace the current column set.
/// - Derive new columns from existing ones.
/// - Clear all columns.
/// - Produce an ID remapping (`old_id -> new_id`) used to preserve
///   referential integrity after reshaping.
///
/// The operations are applied sequentially and are order-dependent.
///
/// # ID Remapping
///
/// Some operations restructure or deduplicate rows, requiring parent ID
/// remapping. When produced:
///
/// - `remapped_ids` stores a `HashMap<u32, u32>` mapping old IDs to new IDs.
/// - If multiple operations produce remappings, the *last one wins*.
/// - If no operation produces remapping, this field remains `None`.
///
/// # Returns
///
/// Returns a `MultiColumnOpResult` containing:
///
/// - `columns`: the final column set after all transformations.
/// - `remapped_ids`: optional ID remapping if produced by any operation.
///
/// Returns `Err(ClickhouseExporterError)` if:
///
/// - A required column is missing.
/// - Any helper transformation fails.
/// - Any internal array-building operation fails.
fn run_multi_column_stage(
    batch: &RecordBatch,
    ops: &[MultiColumnTransformOp],
) -> Result<MultiColumnOpResult, ClickhouseExporterError> {
    let mut columns: HashMap<String, ArrayRef> = HashMap::new();
    if ops.is_empty() {
        columns = batch
            .schema()
            .fields()
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name().clone(), batch.column(i).clone()))
            .collect();
    }

    let mut remapped_ids = None;

    for op in ops {
        match op {
            // Extract Struct child fields (e.g. resource.id, resource.schema_url) into a top level column in the final clickhouse table.
            MultiColumnTransformOp::ExtractFields(ef) => {
                let Some(id_col) = get_u16_array_opt(batch, &ef.id_field_name)? else {
                    return Err(ClickhouseExporterError::MissingColumn {
                        name: ef.id_field_name.clone(),
                    });
                };

                columns = batch
                    .schema()
                    .fields()
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.name().clone(), batch.column(i).clone()))
                    .collect();
                // Note: this no longer relies on a stable `order`.
                // It just uses whatever columns currently exist.
                let mut targets: HashMap<String, ArrayRef> = HashMap::new();
                for (col_name, arr) in &columns {
                    if let Some(final_name) = ef.field_mapping.get(col_name) {
                        _ = targets.insert(final_name.clone(), arr.clone());
                    }
                }

                let (old_to_new_map, new_cols) = build_list_arrays(id_col, &targets)?;
                remapped_ids = Some(old_to_new_map);
                columns = new_cols;
            }
            // This transforms an `Attribute` payload OTAP representation into a MapArray<String, String>. It's used when istring_map is
            // used for one of the primary attribute types (both inline or lookup storage), as well as for SpanLinkAttributes and SpanEventAttributes (always).
            MultiColumnTransformOp::AttributesToStringMap => {
                if let Some((deduped_parent_id_col, attr_col)) = group_attributes_to_map_str(batch)?
                {
                    columns.clear();

                    let mut old_to_new_map: HashMap<u32, u32> =
                        HashMap::with_capacity(deduped_parent_id_col.len());
                    for i in 0..deduped_parent_id_col.len() {
                        _ = old_to_new_map.insert(deduped_parent_id_col.value(i), i as u32);
                    }
                    remapped_ids = Some(old_to_new_map);
                    _ = columns.insert(consts::PARENT_ID.into(), Arc::new(deduped_parent_id_col));
                    _ = columns.insert(consts::ATTRIBUTES.into(), Arc::new(attr_col));
                }
            }
            // This transforms an `Attribute` payload OTAP representation into a DictionaryArray of Uint32Type to generic bytes. It's used when json is
            // used for one of the primary attributes columns (both inline or lookup storage).
            MultiColumnTransformOp::AttributesToJSONString => {
                if let Some((deduped_parent_id_col, attr_col)) =
                    group_attributes_to_json_ser(batch)?
                {
                    columns.clear();
                    // Build lookup table for more efficient inlining.
                    let mut old_to_new_map: HashMap<u32, u32> =
                        HashMap::with_capacity(deduped_parent_id_col.len());
                    for i in 0..deduped_parent_id_col.len() {
                        _ = old_to_new_map.insert(deduped_parent_id_col.value(i), i as u32);
                    }
                    remapped_ids = Some(old_to_new_map);
                    // deduped_parent_id_col is not necessarily in the same order as the original parent IDs, but the lookup table and inline method don't care.
                    _ = columns.insert(consts::PARENT_ID.into(), Arc::new(deduped_parent_id_col));
                    _ = columns.insert(consts::ATTRIBUTES.into(), Arc::new(attr_col));
                }
            }
        }
    }

    Ok(MultiColumnOpResult {
        columns,
        remapped_ids,
    })
}

/// Applies configured column transformation operations to all columns
/// belonging to a given `payload_type`.
///
/// This function:
///
/// 1. Removes the `MultiColumnOpResult` for `payload_type` from
///    `multi_col_op_results`.
/// 2. Iterates over a snapshot of the original column names.
/// 3. Applies the configured `ColumnTransformOp`s for each column
///    in declaration order.
/// 4. Optionally rebuilds and returns a new `RecordBatch` from the
///    transformed columns.
/// 5. Reinserts the (possibly modified) result back into
///    `multi_col_op_results`.
///
/// # Operation Semantics
///
/// - The iteration order is based on a snapshot of the original column names.
///   This ensures deterministic traversal even if operations rename or drop
///   columns.
/// - If a column has no explicitly configured operations in `ops.column_ops`,
///   it defaults to a single [`ColumnTransformOp::Drop`] operation.
/// - If an operation returns `ControlFlow::Break`, no further operations are
///   applied to that column.
/// - Columns removed by earlier operations are skipped if encountered later
///   in the snapshot iteration.
/// - The `ColumnOpCtx` provides access to:
///     - Mutable access to the current column set.
///     - Immutable access to other payload types' column results.
/// - The borrow of the context ends before batch reconstruction to satisfy
///   Rust's borrowing rules.
///
/// # RecordBatch Reconstruction
///
/// If `recreate_batch == true`:
///
/// - Remaining columns are sorted lexicographically by name to ensure
///   deterministic schema ordering.
/// - A new `Schema` is constructed from the columns' data types.
/// - A new `RecordBatch` is created and returned as `Ok(Some(batch))`.
///
/// If `recreate_batch == false`, the function only mutates internal state
/// and returns `Ok(None)`.
///
/// # Returns
///
/// - `Ok(Some(RecordBatch))` if `recreate_batch` is `true` and columns remain.
/// - `Ok(None)` if:
///     - No entry existed for `payload_type`, or
///     - All columns were dropped, or
///     - `recreate_batch` is `false`.
/// - `Err(ClickhouseExporterError)` if any operation or batch construction fails.
fn apply_column_ops(
    multi_col_op_results: &mut HashMap<ArrowPayloadType, MultiColumnOpResult>,
    payload_type: ArrowPayloadType,
    ops: &ColumnOperations,
    recreate_batch: bool,
) -> Result<Option<RecordBatch>, ClickhouseExporterError> {
    let mut these_results = match multi_col_op_results.remove(&payload_type) {
        Some(v) => v,
        None => return Ok(None),
    };
    if these_results.columns.is_empty() {
        return Ok(None);
    }

    // Snapshot of starting names.
    let original_names: Vec<String> = these_results.columns.keys().cloned().collect();

    {
        // Make a context for op application.
        let mut ctx = ColumnOpCtx {
            columns: &mut these_results.columns,
            multi: &*multi_col_op_results,
        };

        // Loop through the columns that were originally present in the batch, applying any Ops.
        for original_name in &original_names {
            if !ctx.contains(original_name) {
                continue; // dropped by earlier ops
            }

            let col_ops: &[ColumnTransformOp] = ops
                .column_ops
                .get(original_name.as_str())
                .map(|v| v.as_slice())
                .unwrap_or(&[ColumnTransformOp::Drop]);

            let mut current_name = original_name.clone();
            for op in col_ops {
                if let ControlFlow::Break(()) = apply_one_op(&mut ctx, &mut current_name, op)? {
                    break;
                }
            }
        }

        // Second pass: process ops for columns created during the first pass (for example,
        // flattened struct fields or columns produced by InlineAttribute / EnumToString).
        // We iterate until no new planned columns appear so chained synthetic-column ops run.
        let mut processed_second_pass: HashSet<String> = HashSet::new();
        loop {
            let pending: Vec<(String, Vec<ColumnTransformOp>)> = ops
                .column_ops
                .iter()
                .filter(|(col_name, _)| !original_names.contains(*col_name))
                .filter(|(col_name, _)| ctx.contains(col_name))
                .filter(|(col_name, _)| !processed_second_pass.contains(*col_name))
                .map(|(col_name, col_ops)| (col_name.clone(), col_ops.clone()))
                .collect();

            if pending.is_empty() {
                break;
            }

            for (col_name, col_ops) in pending {
                _ = processed_second_pass.insert(col_name.clone());
                let mut current_name = col_name;
                for op in &col_ops {
                    if let ControlFlow::Break(()) = apply_one_op(&mut ctx, &mut current_name, op)? {
                        break;
                    }
                }
            }
        }
    } // ctx borrow ends here

    if these_results.columns.is_empty() {
        return Ok(None);
    }

    if recreate_batch {
        let columns = &these_results.columns;

        // deterministic order
        let mut names: Vec<&String> = columns.keys().collect();
        names.sort();

        let fields: Vec<Field> = names
            .iter()
            .map(|name| {
                let arr = &columns[*name];
                Field::new(name.as_str(), arr.data_type().clone(), true)
            })
            .collect();

        let arrays: Vec<ArrayRef> = names.iter().map(|name| columns[*name].clone()).collect();

        let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)?;

        _ = multi_col_op_results.insert(payload_type, these_results);
        return Ok(Some(batch));
    }

    _ = multi_col_op_results.insert(payload_type, these_results);
    Ok(None)
}

#[cfg(test)]
mod runner_sequence_tests {
    #![allow(unused_results)]
    use super::*;
    use std::collections::HashMap;
    use std::ops::ControlFlow;
    use std::sync::Arc;

    use arrow::array::{ArrayRef, UInt32Array};

    fn ctx_with<'a>(
        columns: &'a mut HashMap<String, ArrayRef>,
        multi: &'a HashMap<ArrowPayloadType, MultiColumnOpResult>,
    ) -> ColumnOpCtx<'a> {
        ColumnOpCtx { columns, multi }
    }

    #[test]
    fn apply_one_op_sequence_rename_noop_drop_stops() {
        let mut cols: HashMap<String, ArrayRef> = HashMap::new();
        cols.insert("a".into(), Arc::new(UInt32Array::from(vec![1u32, 2, 3])));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut cols, &multi);

        let ops = vec![
            ColumnTransformOp::Rename("b".to_string()),
            ColumnTransformOp::NoOp,
            ColumnTransformOp::Drop,
            ColumnTransformOp::NoOp, // must NOT run
        ];

        let mut current = "a".to_string();
        for op in &ops {
            match apply_one_op(&mut ctx, &mut current, op).unwrap() {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(()) => break,
            }
        }

        // After rename+noop then drop: column is gone, and we never re-add.
        assert!(!ctx.columns.contains_key("a"));
        assert!(!ctx.columns.contains_key("b"));
    }
}

#[cfg(test)]
mod apply_column_ops_tests {
    #![allow(unused_results)]
    use crate::clickhouse_exporter::consts as ch_consts;
    use arrow::array::StringBuilder;
    use arrow_array::{MapArray, UInt16Array, UInt32Array};

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn col_ops(map: Vec<(&str, Vec<ColumnTransformOp>)>) -> ColumnOperations {
        ColumnOperations {
            column_ops: map.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }

    #[test]
    fn apply_column_ops_recreate_batch_sorts_and_applies_ops() {
        let payload_type = ArrowPayloadType::Logs; // pick any payload variant you have

        // Initial columns intentionally out of order: "b", "a"
        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert("b".into(), Arc::new(UInt32Array::from(vec![1u32, 2, 3])));
        columns.insert("a".into(), Arc::new(UInt32Array::from(vec![10u32, 20, 30])));

        let result = MultiColumnOpResult {
            columns,
            remapped_ids: None,
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(payload_type, result);

        // For "b": rename -> "z"
        // For "a": keep as-is (NoOp)
        // Any column not present in this map defaults to Drop in your code.
        let ops = col_ops(vec![
            ("b", vec![ColumnTransformOp::Rename("z".into())]),
            ("a", vec![ColumnTransformOp::NoOp]),
        ]);

        let out = apply_column_ops(&mut multi, payload_type, &ops, true)
            .unwrap()
            .expect("batch should be produced");

        // Output schema should be sorted by name: ["a", "z"]
        let binding = out.schema();
        let names: Vec<&str> = binding.fields().iter().map(|f| f.name().as_str()).collect();
        assert_eq!(names, vec!["a", "z"]);

        // Values preserved
        let a = out
            .column(0)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        let z = out
            .column(1)
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(a.values(), &[10, 20, 30]);
        assert_eq!(z.values(), &[1, 2, 3]);

        // And the transformed results should be reinserted for later ops
        assert!(multi.contains_key(&payload_type));
    }

    #[test]
    fn apply_column_ops_keeps_inlined_attribute_column_from_original_id_column() {
        let payload_type = ArrowPayloadType::Logs;

        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert(
            ch_consts::RESOURCE_ID.into(),
            Arc::new(UInt16Array::from(vec![10u16, 11u16])),
        );

        let result = MultiColumnOpResult {
            columns,
            remapped_ids: None,
        };

        let mut child_builder = MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        child_builder.keys().append_value("service.name");
        child_builder.values().append_value("checkout");
        child_builder.append(true).unwrap();
        child_builder.keys().append_value("service.name");
        child_builder.values().append_value("payments");
        child_builder.append(true).unwrap();

        let child_result = MultiColumnOpResult {
            columns: vec![(
                consts::ATTRIBUTES.to_string(),
                Arc::new(child_builder.finish()) as ArrayRef,
            )]
            .into_iter()
            .collect(),
            remapped_ids: Some(vec![(10u32, 0u32), (11u32, 1u32)].into_iter().collect()),
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(payload_type, result);
        multi.insert(ArrowPayloadType::ResourceAttrs, child_result);

        let ops = col_ops(vec![(
            ch_consts::RESOURCE_ID.into(),
            vec![ColumnTransformOp::InlineAttribute(
                ArrowPayloadType::ResourceAttrs,
                crate::clickhouse_exporter::config::AttributeRepresentation::StringMap,
            )],
        )]);

        let out = apply_column_ops(&mut multi, payload_type, &ops, true)
            .unwrap()
            .expect("batch should be produced");

        assert!(out
            .schema()
            .fields()
            .iter()
            .any(|field| field.name() == ch_consts::CH_RESOURCE_ATTRIBUTES));

        let attrs = out
            .column(
                out.schema()
                    .index_of(ch_consts::CH_RESOURCE_ATTRIBUTES)
                    .expect("resource attributes column"),
            )
            .as_any()
            .downcast_ref::<MapArray>()
            .expect("resource attributes should be map");
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn apply_column_ops_inlines_resource_and_scope_attributes_for_logs_parent_batch() {
        let payload_type = ArrowPayloadType::Logs;

        let mut columns: HashMap<String, ArrayRef> = HashMap::new();
        columns.insert(
            ch_consts::RESOURCE_ID.into(),
            Arc::new(UInt16Array::from(vec![10u16, 10u16])),
        );
        columns.insert(
            ch_consts::SCOPE_ID.into(),
            Arc::new(UInt16Array::from(vec![20u16, 20u16])),
        );

        let parent_result = MultiColumnOpResult {
            columns,
            remapped_ids: None,
        };

        let mut resource_builder =
            MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        resource_builder.keys().append_value("service.name");
        resource_builder.values().append_value("checkout");
        resource_builder
            .keys()
            .append_value("deployment.environment");
        resource_builder.values().append_value("prod");
        resource_builder.append(true).unwrap();

        let mut scope_builder = MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        scope_builder.keys().append_value("scope.attr");
        scope_builder.values().append_value("scope-value");
        scope_builder.append(true).unwrap();

        let resource_result = MultiColumnOpResult {
            columns: vec![(
                consts::ATTRIBUTES.to_string(),
                Arc::new(resource_builder.finish()) as ArrayRef,
            )]
            .into_iter()
            .collect(),
            remapped_ids: Some(vec![(10u32, 0u32)].into_iter().collect()),
        };
        let scope_result = MultiColumnOpResult {
            columns: vec![(
                consts::ATTRIBUTES.to_string(),
                Arc::new(scope_builder.finish()) as ArrayRef,
            )]
            .into_iter()
            .collect(),
            remapped_ids: Some(vec![(20u32, 0u32)].into_iter().collect()),
        };

        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(payload_type, parent_result);
        multi.insert(ArrowPayloadType::ResourceAttrs, resource_result);
        multi.insert(ArrowPayloadType::ScopeAttrs, scope_result);

        let ops = col_ops(vec![
            (
                ch_consts::RESOURCE_ID.into(),
                vec![ColumnTransformOp::InlineAttribute(
                    ArrowPayloadType::ResourceAttrs,
                    crate::clickhouse_exporter::config::AttributeRepresentation::StringMap,
                )],
            ),
            (
                ch_consts::SCOPE_ID.into(),
                vec![ColumnTransformOp::InlineAttribute(
                    ArrowPayloadType::ScopeAttrs,
                    crate::clickhouse_exporter::config::AttributeRepresentation::StringMap,
                )],
            ),
        ]);

        let out = apply_column_ops(&mut multi, payload_type, &ops, true)
            .unwrap()
            .expect("batch should be produced");

        assert!(out
            .schema()
            .fields()
            .iter()
            .any(|field| field.name() == ch_consts::CH_RESOURCE_ATTRIBUTES));
        assert!(out
            .schema()
            .fields()
            .iter()
            .any(|field| field.name() == ch_consts::CH_SCOPE_ATTRIBUTES));
    }
}

#[cfg(test)]
mod multi_plus_single_tests {
    #![allow(unused_results)]
    use crate::clickhouse_exporter::transform::transform_plan::ExtractGroupedFieldSpec;

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use arrow::array::*;
    use arrow::record_batch::RecordBatch;

    fn rb(schema: Schema, cols: Vec<ArrayRef>) -> RecordBatch {
        RecordBatch::try_new(Arc::new(schema), cols).unwrap()
    }

    #[test]
    fn extract_fields_then_inline_child_lists() {
        // ---------- Child batch (to be compacted into lists) ----------
        // rows:
        // parent_id=10 event="e1"
        // parent_id=10 event="e2"
        // parent_id=11 event="f1"
        let child_schema = Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, true),
            Field::new("event_name", DataType::Utf8, true),
        ]);

        let child_parent_id: ArrayRef = Arc::new(UInt16Array::from(vec![10u16, 10u16, 11u16]));
        let child_event: ArrayRef =
            Arc::new(StringArray::from(vec![Some("e1"), Some("e2"), Some("f1")]));

        let child_rb = rb(child_schema, vec![child_parent_id, child_event]);

        // Multi op: ExtractFields groups event_name by parent_id into ListArray
        let mut fm = HashMap::new();
        fm.insert("event_name".to_string(), "events".to_string());

        let ef = ExtractGroupedFieldSpec {
            id_field_name: "parent_id".to_string(),
            field_mapping: fm,
        };

        let child_multi =
            run_multi_column_stage(&child_rb, &[MultiColumnTransformOp::ExtractFields(ef)])
                .unwrap();

        // Multi results map contains child payload result
        let child_payload = ArrowPayloadType::SpanEvents; // choose your “child lists” payload
        let parent_payload = ArrowPayloadType::Spans; // choose your parent payload

        let mut multi_results: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi_results.insert(child_payload, child_multi);

        // ---------- Parent batch (single-column stage will inline child lists) ----------
        // parent rows: parent_id is unique and matches "old ids" used in child remap
        let parent_schema = Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, true),
            Field::new("other", DataType::UInt32, true),
        ]);

        let parent_ids: ArrayRef = Arc::new(UInt16Array::from(vec![10u16, 11u16]));
        let other: ArrayRef = Arc::new(UInt32Array::from(vec![1u32, 2u32]));
        let parent_rb = rb(parent_schema, vec![parent_ids.clone(), other]);

        // Seed parent result into multi_results so apply_column_ops can operate on it
        let parent_result = MultiColumnOpResult {
            columns: vec![
                ("parent_id".to_string(), parent_rb.column(0).clone()),
                ("other".to_string(), parent_rb.column(1).clone()),
            ]
            .into_iter()
            .collect(),
            remapped_ids: None,
        };
        multi_results.insert(parent_payload, parent_result);

        // Column ops: inline child lists using parent_id column
        let mut col_ops_map: HashMap<String, Vec<ColumnTransformOp>> = HashMap::new();
        col_ops_map.insert(
            "parent_id".to_string(),
            vec![ColumnTransformOp::InlineChildLists(child_payload)],
        );
        // keep "other" (NoOp), otherwise it defaults to Drop
        col_ops_map.insert("other".to_string(), vec![ColumnTransformOp::NoOp]);

        let ops = ColumnOperations {
            column_ops: col_ops_map,
        };

        // Apply single-column stage for parent payload
        let out = apply_column_ops(&mut multi_results, parent_payload, &ops, true)
            .unwrap()
            .unwrap();

        // Assert: new "events" column exists and is a ListArray
        let schema_fields: Vec<_> = out
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        assert!(schema_fields.iter().any(|n| n == "events"));

        let events_col = out
            .column(out.schema().index_of("events").unwrap())
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();

        assert_eq!(events_col.len(), 2);

        // Row0 events should be ["e1","e2"], row1 should be ["f1"]
        let values = events_col
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let offsets = events_col.offsets();

        // row0: offsets[0..1]
        assert_eq!((offsets[0] as usize, offsets[1] as usize), (0, 2));
        assert_eq!(values.value(0), "e1");
        assert_eq!(values.value(1), "e2");

        // row1: offsets[1..2]
        assert_eq!((offsets[1] as usize, offsets[2] as usize), (2, 3));
        assert_eq!(values.value(2), "f1");
    }
}

#[cfg(test)]
mod realistic_otap_tests {
    #![allow(unused_results)]

    use std::collections::HashMap;

    use arrow::array::{Array, ListArray, MapArray, RecordBatch, StringArray, UInt64Array};
    use arrow::datatypes::DataType;
    use bytes::Bytes;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{
        span::{Event, Link, SpanKind},
        status::StatusCode,
        ResourceSpans, ScopeSpans, Span, Status,
    };
    use otap_df_pdata::schema::consts;
    use otap_df_pdata::testing::fixtures;
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtlpProtoBytes};
    use prost::Message;

    use super::BatchTransformer;
    use crate::clickhouse_exporter::config::{Config, ConfigPatch};
    use crate::clickhouse_exporter::consts as ch_consts;
    use crate::clickhouse_exporter::transform::transform_batch::{
        apply_column_ops, run_multi_column_stage, MultiColumnOpResult,
    };
    use crate::clickhouse_exporter::transform::transform_plan::{
        MultiColumnTransformOp, TransformationPlan,
    };

    fn test_config() -> Config {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "user",
            "password": "pass"
        });
        let patch: ConfigPatch =
            serde_json::from_value(json).expect("valid clickhouse config patch");
        Config::from_patch(patch)
    }

    fn test_config_with_resource_json() -> Config {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "user",
            "password": "pass",
            "attributes": {
                "resource": { "representation": "json" }
            }
        });
        let patch: ConfigPatch =
            serde_json::from_value(json).expect("valid clickhouse config patch");
        Config::from_patch(patch)
    }

    fn logs_to_arrow_records(request: ExportLogsServiceRequest) -> OtapArrowRecords {
        let mut buf = Vec::new();
        request.encode(&mut buf).expect("encode OTLP logs request");
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(Bytes::from(buf)).into();
        payload.try_into().expect("convert OTLP logs to OTAP Arrow")
    }

    fn traces_to_arrow_records(request: ExportTraceServiceRequest) -> OtapArrowRecords {
        let mut buf = Vec::new();
        request
            .encode(&mut buf)
            .expect("encode OTLP traces request");
        let payload: OtapPayload = OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf)).into();
        payload
            .try_into()
            .expect("convert OTLP traces to OTAP Arrow")
    }

    fn column_names(batch: &RecordBatch) -> Vec<String> {
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect()
    }

    fn string_values(batch: &RecordBatch, name: &str) -> Vec<Option<String>> {
        let arr = batch.column(batch.schema().index_of(name).expect("column must exist"));
        let casted = arrow::compute::cast(arr, &DataType::Utf8).expect("column must cast to Utf8");
        let col = casted
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("column must be Utf8 after cast");

        (0..col.len())
            .map(|i| (!col.is_null(i)).then(|| col.value(i).to_string()))
            .collect()
    }

    fn list_len_at(batch: &RecordBatch, name: &str, row: usize) -> usize {
        let col = batch
            .column(batch.schema().index_of(name).expect("column must exist"))
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("column must be ListArray");
        col.value(row).len()
    }

    fn map_value_at(batch: &RecordBatch, name: &str, row: usize, key: &str) -> Option<String> {
        let col = batch
            .column(batch.schema().index_of(name).expect("column must exist"))
            .as_any()
            .downcast_ref::<MapArray>()
            .expect("column must be MapArray");

        if col.is_null(row) {
            return None;
        }

        let keys = col
            .keys()
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("map keys must be strings");
        let values = col
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("map values must be strings");
        let offsets = col.offsets();
        let start = offsets[row] as usize;
        let end = offsets[row + 1] as usize;

        for idx in start..end {
            if !keys.is_null(idx) && keys.value(idx) == key {
                return (!values.is_null(idx)).then(|| values.value(idx).to_string());
            }
        }

        None
    }

    fn build_logs_with_service_name() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(
                                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                        "checkout".to_string(),
                                    ),
                                ),
                            }),
                        },
                        KeyValue {
                            key: "deployment.environment".to_string(),
                            value: Some(AnyValue {
                                value: Some(
                                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                        "prod".to_string(),
                                    ),
                                ),
                            }),
                        },
                    ],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                schema_url: "https://resource.example/v1".to_string(),
                scope_logs: vec![
                    ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "gateway.clickhouse.tests".to_string(),
                            version: "1.2.3".to_string(),
                            attributes: vec![KeyValue {
                                key: "scope.attr".to_string(),
                                value: Some(AnyValue {
                                    value: Some(
                                        otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                            "scope-value".to_string(),
                                        ),
                                    ),
                                }),
                            }],
                            dropped_attributes_count: 0,
                        }),
                        schema_url: "https://scope.example/v1".to_string(),
                        log_records: vec![
                            LogRecord {
                                time_unix_nano: 1_736_937_000_000_000_000,
                                observed_time_unix_nano: 1_736_937_000_100_000_000,
                                severity_number: SeverityNumber::Info as i32,
                                severity_text: "INFO".to_string(),
                                event_name: "http.request".to_string(),
                                body: Some(AnyValue {
                                    value: Some(
                                        otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                            "request completed".to_string(),
                                        ),
                                    ),
                                }),
                                attributes: vec![KeyValue {
                                    key: "http.method".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(
                                            otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                                "GET".to_string(),
                                            ),
                                        ),
                                    }),
                                }],
                                ..Default::default()
                            },
                            LogRecord {
                                time_unix_nano: 1_736_937_001_000_000_000,
                                observed_time_unix_nano: 1_736_937_001_100_000_000,
                                severity_number: SeverityNumber::Warn as i32,
                                severity_text: "WARN".to_string(),
                                event_name: "cache.miss".to_string(),
                                body: Some(AnyValue {
                                    value: Some(
                                        otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                            "cache miss".to_string(),
                                        ),
                                    ),
                                }),
                                ..Default::default()
                            },
                        ],
                    },
                ],
            }],
        }
    }

    fn build_traces_with_children() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(
                                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                        "payments".to_string(),
                                    ),
                                ),
                            }),
                        },
                        KeyValue {
                            key: "cloud.region".to_string(),
                            value: Some(AnyValue {
                                value: Some(
                                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                        "us-east-1".to_string(),
                                    ),
                                ),
                            }),
                        },
                    ],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                schema_url: "https://resource.example/v1".to_string(),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "gateway.clickhouse.tests".to_string(),
                        version: "1.0.0".to_string(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                    }),
                    schema_url: String::new(),
                    spans: vec![Span {
                        trace_id: vec![0x11; 16],
                        span_id: vec![0x22; 8],
                        trace_state: "vendor=test".to_string(),
                        parent_span_id: vec![0x33; 8],
                        name: "POST /charge".to_string(),
                        kind: SpanKind::Server as i32,
                        start_time_unix_nano: 2_000,
                        end_time_unix_nano: 7_000,
                        attributes: vec![KeyValue {
                            key: "http.route".to_string(),
                            value: Some(AnyValue {
                                value: Some(
                                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                        "/charge".to_string(),
                                    ),
                                ),
                            }),
                        }],
                        events: vec![
                            Event {
                                time_unix_nano: 3_000,
                                name: "db.query.start".to_string(),
                                attributes: vec![KeyValue {
                                    key: "db.system".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(
                                            otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                                "clickhouse".to_string(),
                                            ),
                                        ),
                                    }),
                                }],
                                dropped_attributes_count: 0,
                            },
                            Event {
                                time_unix_nano: 4_000,
                                name: "db.query.end".to_string(),
                                attributes: vec![KeyValue {
                                    key: "db.statement".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(
                                            otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                                "INSERT INTO traces".to_string(),
                                            ),
                                        ),
                                    }),
                                }],
                                dropped_attributes_count: 0,
                            },
                        ],
                        links: vec![Link {
                            trace_id: vec![0x44; 16],
                            span_id: vec![0x55; 8],
                            trace_state: "linked=true".to_string(),
                            attributes: vec![KeyValue {
                                key: "link.kind".to_string(),
                                value: Some(AnyValue {
                                    value: Some(
                                        otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                                            "causal".to_string(),
                                        ),
                                    ),
                                }),
                            }],
                            dropped_attributes_count: 0,
                            flags: 0,
                        }],
                        status: Some(Status {
                            message: "ok".to_string(),
                            code: StatusCode::Ok as i32,
                        }),
                        flags: 0,
                        ..Default::default()
                    }],
                }],
            }],
        }
    }

    #[test]
    fn apply_plan_logs_fixture_produces_clickhouse_ready_logs_batch() {
        let mut transformer = BatchTransformer::new_from_config(&test_config());
        let arrow_records = logs_to_arrow_records(ExportLogsServiceRequest {
            resource_logs: fixtures::logs_with_full_resource_and_scope().resource_logs,
        });

        let results = transformer
            .apply_plan(arrow_records)
            .expect("transform logs");

        assert!(results.contains_key(&ArrowPayloadType::Logs));
        assert!(!results.contains_key(&ArrowPayloadType::ResourceAttrs));
        assert!(!results.contains_key(&ArrowPayloadType::ScopeAttrs));
        assert!(!results.contains_key(&ArrowPayloadType::LogAttrs));

        let batch = results
            .get(&ArrowPayloadType::Logs)
            .expect("logs batch must exist");
        let names = column_names(batch);
        assert!(batch.num_rows() > 0);
        for required in [
            ch_consts::CH_TIMESTAMP,
            ch_consts::CH_SEVERITY_NUMBER,
            ch_consts::CH_SEVERITY_TEXT,
            ch_consts::CH_BODY,
            ch_consts::CH_EVENT_NAME,
            ch_consts::CH_SCOPE_NAME,
            ch_consts::CH_SCOPE_VERSION,
        ] {
            assert!(
                names.iter().any(|name| name == required),
                "missing required column {required}"
            );
        }
    }

    #[test]
    fn apply_plan_logs_preserves_resource_and_log_shaping_with_realistic_fixture() {
        let mut transformer = BatchTransformer::new_from_config(&test_config());
        let arrow_records = logs_to_arrow_records(build_logs_with_service_name());

        let results = transformer
            .apply_plan(arrow_records)
            .expect("transform logs");
        let batch = results
            .get(&ArrowPayloadType::Logs)
            .expect("logs batch must exist");

        let names = column_names(batch);
        for required in [
            ch_consts::CH_BODY,
            ch_consts::CH_EVENT_NAME,
            ch_consts::CH_SCOPE_NAME,
            ch_consts::CH_SCOPE_VERSION,
            ch_consts::CH_RESOURCE_SCHEMA_URL,
            ch_consts::CH_LOG_ATTRIBUTES,
        ] {
            assert!(
                names.iter().any(|name| name == required),
                "missing required column {required}"
            );
        }

        assert_eq!(
            string_values(batch, ch_consts::CH_SCOPE_NAME),
            vec![
                Some("gateway.clickhouse.tests".to_string()),
                Some("gateway.clickhouse.tests".to_string())
            ]
        );
    }

    #[test]
    fn apply_plan_logs_inlines_resource_and_scope_attribute_values() {
        let mut transformer = BatchTransformer::new_from_config(&test_config());
        let arrow_records = logs_to_arrow_records(build_logs_with_service_name());

        let results = transformer
            .apply_plan(arrow_records)
            .expect("transform logs");
        let batch = results
            .get(&ArrowPayloadType::Logs)
            .expect("logs batch must exist");

        let names = column_names(batch);
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_RESOURCE_ATTRIBUTES),
            "missing resource attributes column"
        );
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_SCOPE_ATTRIBUTES),
            "missing scope attributes column"
        );
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_LOG_ATTRIBUTES),
            "missing log attributes column"
        );
        for row in 0..batch.num_rows() {
            assert_eq!(
                map_value_at(
                    batch,
                    ch_consts::CH_RESOURCE_ATTRIBUTES,
                    row,
                    "service.name"
                ),
                Some("checkout".to_string())
            );
            assert_eq!(
                map_value_at(
                    batch,
                    ch_consts::CH_RESOURCE_ATTRIBUTES,
                    row,
                    "deployment.environment",
                ),
                Some("prod".to_string())
            );
            assert_eq!(
                map_value_at(batch, ch_consts::CH_SCOPE_ATTRIBUTES, row, "scope.attr"),
                Some("scope-value".to_string())
            );
            if row == 0 {
                assert_eq!(
                    map_value_at(batch, ch_consts::CH_LOG_ATTRIBUTES, row, "http.method"),
                    Some("GET".to_string())
                );
            }
        }
    }

    #[test]
    fn realistic_logs_apply_column_ops_with_real_child_batches_inlines_all_attr_types() {
        let arrow_records = logs_to_arrow_records(build_logs_with_service_name());
        let logs_batch = arrow_records
            .get(ArrowPayloadType::Logs)
            .expect("logs payload must exist");
        let resource_attrs = arrow_records
            .get(ArrowPayloadType::ResourceAttrs)
            .expect("resource attrs payload must exist");
        let scope_attrs = arrow_records
            .get(ArrowPayloadType::ScopeAttrs)
            .expect("scope attrs payload must exist");
        let log_attrs = arrow_records
            .get(ArrowPayloadType::LogAttrs)
            .expect("log attrs payload must exist");

        let plan = TransformationPlan::from_config(&ArrowPayloadType::Logs, &test_config());
        let mut multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        multi.insert(
            ArrowPayloadType::Logs,
            run_multi_column_stage(logs_batch, &[]).expect("transform logs batch"),
        );
        multi.insert(
            ArrowPayloadType::ResourceAttrs,
            run_multi_column_stage(
                resource_attrs,
                &[MultiColumnTransformOp::AttributesToStringMap],
            )
            .expect("transform resource attrs"),
        );
        multi.insert(
            ArrowPayloadType::ScopeAttrs,
            run_multi_column_stage(
                scope_attrs,
                &[MultiColumnTransformOp::AttributesToStringMap],
            )
            .expect("transform scope attrs"),
        );
        multi.insert(
            ArrowPayloadType::LogAttrs,
            run_multi_column_stage(log_attrs, &[MultiColumnTransformOp::AttributesToStringMap])
                .expect("transform log attrs"),
        );

        let batch = apply_column_ops(&mut multi, ArrowPayloadType::Logs, &plan.column_ops, true)
            .expect("apply column ops")
            .expect("logs batch should be rebuilt");

        let names = column_names(&batch);
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_RESOURCE_ATTRIBUTES),
            "missing resource attributes column"
        );
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_SCOPE_ATTRIBUTES),
            "missing scope attributes column"
        );
        assert!(
            names
                .iter()
                .any(|name| name == ch_consts::CH_LOG_ATTRIBUTES),
            "missing log attributes column"
        );
    }

    #[test]
    fn apply_plan_spans_fixture_inlines_child_payloads_and_core_columns() {
        let mut transformer = BatchTransformer::new_from_config(&test_config());
        let arrow_records = traces_to_arrow_records(build_traces_with_children());

        let results = transformer
            .apply_plan(arrow_records)
            .expect("transform spans");

        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&ArrowPayloadType::Spans));

        let batch = results
            .get(&ArrowPayloadType::Spans)
            .expect("spans batch must exist");
        let names = column_names(batch);
        for required in [
            ch_consts::CH_TIMESTAMP,
            ch_consts::CH_TRACE_ID,
            ch_consts::CH_SPAN_ID,
            ch_consts::CH_PARENT_SPAN_ID,
            ch_consts::CH_TRACE_STATE,
            ch_consts::CH_SPAN_NAME,
            ch_consts::CH_SPAN_KIND,
            ch_consts::CH_DURATION,
            ch_consts::CH_STATUS_CODE,
            ch_consts::CH_STATUS_MESSAGE,
            ch_consts::CH_EVENTS_TIMESTAMP,
            ch_consts::CH_EVENTS_NAME,
            ch_consts::CH_LINKS_TRACE_ID,
            ch_consts::CH_LINKS_SPAN_ID,
            ch_consts::CH_LINKS_TRACE_STATE,
            ch_consts::CH_EVENTS_ATTRIBUTES,
            ch_consts::CH_LINKS_ATTRIBUTES,
        ] {
            assert!(
                names.iter().any(|name| name == required),
                "missing required column {required}"
            );
        }

        let duration = batch
            .column(
                batch
                    .schema()
                    .index_of(ch_consts::CH_DURATION)
                    .expect("duration column"),
            )
            .as_any()
            .downcast_ref::<UInt64Array>()
            .expect("duration must be UInt64");
        assert_eq!(duration.value(0), 5_000);

        assert_eq!(list_len_at(batch, ch_consts::CH_EVENTS_NAME, 0), 2);
        assert_eq!(list_len_at(batch, ch_consts::CH_LINKS_SPAN_ID, 0), 1);

        // Intermediate foreign-key columns must NOT leak into the final batch.
        for unwanted in [ch_consts::SCOPE_ID, ch_consts::RESOURCE_ID, consts::ID] {
            assert!(
                !names.iter().any(|name| name == unwanted),
                "intermediate column {unwanted} must not appear in the final spans batch"
            );
        }
    }

    #[test]
    fn apply_plan_logs_json_attribute_mode_keeps_realistic_batch_transformable() {
        let mut transformer = BatchTransformer::new_from_config(&test_config_with_resource_json());
        let arrow_records = logs_to_arrow_records(build_logs_with_service_name());

        let results = transformer
            .apply_plan(arrow_records)
            .expect("transform logs in json mode");
        let batch = results
            .get(&ArrowPayloadType::Logs)
            .expect("logs batch must exist");

        let names = column_names(batch);
        assert!(names.iter().any(|name| name == ch_consts::CH_BODY));
        assert!(names.iter().any(|name| name == ch_consts::CH_EVENT_NAME));
    }
}
