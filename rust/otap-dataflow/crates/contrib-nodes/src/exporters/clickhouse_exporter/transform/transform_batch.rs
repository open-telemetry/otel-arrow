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
//!   from exporter [`Config`]) and a [`PartitionSequenceIdGenerator`] used to synthesize/assign
//!   partition-scoped IDs for payloads that require them.
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
use std::{collections::HashMap, sync::Arc};

use arrow::array::{
    Array, ArrayBuilder, ListBuilder, MapBuilder, PrimitiveArray, StringBuilder,
    TimestampNanosecondBuilder, make_builder,
};
use arrow::datatypes::UInt16Type;
use arrow::{
    array::{ArrayRef, RecordBatch},
    datatypes::{Field, Schema},
};
use arrow_array::{MapArray, StringArray, TimestampNanosecondArray};
use arrow_schema::{DataType, TimeUnit};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::clickhouse_exporter::arrays::get_u16_array_opt;
use crate::clickhouse_exporter::config::Config;
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::idgen::PartitionSequenceIdGenerator;
use crate::clickhouse_exporter::tables::build_payloads_with_id_map;
use crate::clickhouse_exporter::transform::build_payload_transform_map;
use crate::clickhouse_exporter::transform::transform_attributes::{
    group_attributes_to_json_ser, group_attributes_to_map_str, group_rows_by_id,
};
use crate::clickhouse_exporter::transform::transform_column::{ColumnOpCtx, apply_one_op};
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
    id_generator: PartitionSequenceIdGenerator,
}
impl BatchTransformer {
    /// Create a new batch transformer from static payload transformation plans.
    pub fn new_from_config(config: &Config) -> Self {
        let payload_transform_plans = build_payload_transform_map(config);
        let payloads_with_ids = build_payloads_with_id_map(config);
        let id_generator = PartitionSequenceIdGenerator::new(payloads_with_ids);
        Self {
            payload_transform_plans,
            id_generator,
        }
    }

    /// This method is the orchestration layer for the transformation engine.
    /// For each allowed `ArrowPayloadType`, it executes:
    ///
    /// 1. **ID planning stage** (via `PartitionSequenceIdGenerator`)
    /// 2. **Multi-column transformation stage**
    /// 3. **Single-column transformation stage**
    /// 4. **Optional RecordBatch reconstruction**
    ///
    /// The result is a map of transformed `RecordBatch`es ready for export.
    ///
    /// ---
    ///
    /// # Stage 1: ID Plan Generation
    ///
    /// Before transforming columns, the transformer computes any required
    /// ID generation or remapping plans using:
    ///
    /// - `self.id_generator.generate_id_transform_plan`
    ///
    /// These dynamic plans are later merged with the static
    /// `TransformationPlan` for each payload.
    ///
    /// This plan results in:
    /// 1. A new "partition" UUID column (partID) which will be used for up to MAX(U32) batch scoped IDs.
    /// 2. A re-mapping of original batch scoped ID to a new value with the current offset of the partID U32 counter.
    ///
    /// So, if the original "PARENT_ID" is 5, and the PARTITION offset is 100, the remapped partition scoped ID will be 105.
    ///
    /// ---
    ///
    /// # Stage 2: Multi-Column Transformations
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
    /// # Stage 3: Single-Column Transformations
    ///
    /// For each payload type:
    ///
    /// - The static plan is optionally merged with a dynamic ID plan.
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
    /// # Plan Merging Semantics
    ///
    /// When an ID-generation plan exists for a payload type:
    ///
    /// - It is merged with the static `TransformationPlan`
    /// - The merged plan is used only for that invocation
    /// - Static plans stored in the transformer remain unchanged
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
    /// - ID plan generation fails
    /// - Any multi-column stage fails
    /// - Any single-column stage fails
    pub fn apply_plan(
        &mut self,
        mut arrow_records: OtapArrowRecords,
    ) -> Result<HashMap<ArrowPayloadType, RecordBatch>, ClickhouseExporterError> {
        let mut writable_batches: HashMap<ArrowPayloadType, RecordBatch> = HashMap::new();
        let mut multi_col_results: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();

        let id_gen_plans = self
            .id_generator
            .generate_id_transform_plan(&mut arrow_records)?;

        // Multi-column stage
        for &payload_type in arrow_records.allowed_payload_types() {
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
        for &payload_type in arrow_records.allowed_payload_types() {
            let id_gen_plan = id_gen_plans.get(&payload_type);
            let Some(static_plan) = self.payload_transform_plans.get(&payload_type) else {
                continue;
            };

            let merged_plan;
            let plan = match id_gen_plan {
                Some(idp) => {
                    merged_plan = static_plan.merged(idp);
                    &merged_plan
                }
                None => static_plan,
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
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            downcast_append_primitive!(TimestampNanosecondBuilder, TimestampNanosecondArray)
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
        // TODO: [Correctness] per @a.lockett:
        // I think the intention is to use this for SpanLink & SpanEvent, right? We'll also need to support UInt32 and FixedSizeBinary I'm pretty sure
        // https://github.com/open-telemetry/otel-arrow/blob/cbc03d838832e2dedba932c899b95cdf95b07594/go/pkg/otel/traces/arrow/event.go#L35-L40
        // https://github.com/open-telemetry/otel-arrow/blob/cbc03d838832e2dedba932c899b95cdf95b07594/go/pkg/otel/traces/arrow/link.go#L36-L44
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
            let builder = ListBuilder::new(make_builder(array.data_type(), 0));
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
        for original_name in original_names {
            if !ctx.contains(&original_name) {
                continue; // dropped by earlier ops
            }

            let col_ops: &[ColumnTransformOp] = ops
                .column_ops
                .get(&original_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[ColumnTransformOp::Drop]);

            let mut current_name = original_name.clone();
            for op in col_ops {
                if let ControlFlow::Break(()) = apply_one_op(&mut ctx, &mut current_name, op)? {
                    break;
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
    fn apply_one_op_sequence_rename_addoffset_drop_stops() {
        let mut cols: HashMap<String, ArrayRef> = HashMap::new();
        cols.insert("a".into(), Arc::new(UInt32Array::from(vec![1u32, 2, 3])));

        let multi: HashMap<ArrowPayloadType, MultiColumnOpResult> = HashMap::new();
        let mut ctx = ctx_with(&mut cols, &multi);

        let ops = vec![
            ColumnTransformOp::Rename("b".to_string()),
            ColumnTransformOp::AddOffset(10),
            ColumnTransformOp::Drop,
            ColumnTransformOp::AddOffset(999), // must NOT run
        ];

        let mut current = "a".to_string();
        for op in &ops {
            match apply_one_op(&mut ctx, &mut current, op).unwrap() {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(()) => break,
            }
        }

        // After rename+offset then drop: column is gone, and we never re-add.
        assert!(!ctx.columns.contains_key("a"));
        assert!(!ctx.columns.contains_key("b"));
    }
}

#[cfg(test)]
mod apply_column_ops_tests {
    #![allow(unused_results)]
    use arrow_array::UInt32Array;

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
