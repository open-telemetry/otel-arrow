// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projection utilities for extracting required columns from RecordBatches

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, RecordBatch, RecordBatchOptions, StructArray};
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Schema};
use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion, TreeNodeVisitor};
use datafusion::common::{HashMap, HashSet};
use datafusion::error::DataFusionError;
use datafusion::functions::core::getfield::GetFieldFunc;
use datafusion::logical_expr::Expr;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};

#[derive(Default)]
pub struct ProjectionOptions {
    /// Whether or not to downcast dictionary arrays to the native type. Some types of expressions,
    /// arithmetic operations for example, do not work on dictionary encoded columns.
    pub downcast_dicts: bool,
}

/// Projection helper that can project a RecordBatch to only the columns needed by an expression
#[derive(Debug)]
pub struct Projection {
    pub schema: ProjectedSchema,
}

impl From<Vec<String>> for Projection {
    fn from(columns: Vec<String>) -> Self {
        Self {
            schema: columns
                .into_iter()
                .map(ProjectedSchemaColumn::Root)
                .collect(),
        }
    }
}

impl Projection {
    /// Attempt to create a new instance of [`FilterProjection`]. It will return an error if
    /// there is some form of [`Expr`] tree which is not recognized
    pub(crate) fn try_new(logical_expr: &Expr) -> Result<Self> {
        let mut visitor = ProjectedSchemaExprVisitor::default();
        _ = logical_expr.visit(&mut visitor)?;
        Ok(Self {
            schema: visitor.into(),
        })
    }

    /// Project the record batch to the expected schema. If there are some expected columns in
    /// the passed [`RecordBatch`] which are missing, this will return `None`.
    pub fn project(&self, record_batch: &RecordBatch) -> Result<Option<RecordBatch>> {
        self.project_with_options(record_batch, &ProjectionOptions::default())
    }

    pub fn project_with_options(
        &self,
        record_batch: &RecordBatch,
        options: &ProjectionOptions,
    ) -> Result<Option<RecordBatch>> {
        let (mut fields, mut columns) = match self.project_columns(record_batch) {
            Some(projection) => projection,
            None => return Ok(None),
        };

        if options.downcast_dicts {
            Self::try_downcast_dicts(&mut fields, &mut columns)?;
        }

        // safety: `try_new` should not return an error here unless the columns do not match the
        // fields in the schema, or if the columns are different lengths. Based on how we've
        // constructed the inputs, this should not happen because we've taken them from the input
        let rb = RecordBatch::try_new_with_options(
            Arc::new(Schema::new(fields)),
            columns,
            &RecordBatchOptions::new().with_row_count(Some(record_batch.num_rows())),
        )
        .expect("can project record batch");

        Ok(Some(rb))
    }

    fn project_columns(
        &self,
        record_batch: &RecordBatch,
    ) -> Option<(Vec<Arc<Field>>, Vec<ArrayRef>)> {
        let original_schema = record_batch.schema_ref();

        // TODO - if the heap allocations here have significant perf overhead, we could try reusing
        // these arrays between batches.
        let mut columns = Vec::new();
        let mut fields = Vec::new();

        for projected_col in &self.schema {
            match projected_col {
                ProjectedSchemaColumn::Root(desired_col_name) => {
                    let index = original_schema.index_of(desired_col_name).ok()?;
                    let column = record_batch.column(index).clone();
                    let field = original_schema.fields[index].clone();
                    columns.push(column);
                    fields.push(field)
                }
                ProjectedSchemaColumn::Struct(desired_struct_name, desired_struct_fields) => {
                    let struct_index = original_schema.index_of(desired_struct_name).ok()?;
                    let column = record_batch.column(struct_index);
                    let col_as_struct = column.as_any().downcast_ref::<StructArray>()?;

                    let mut struct_fields = Vec::new();
                    let mut struct_field_defs = Vec::new();

                    for field_name in desired_struct_fields {
                        let (field_index, field) = col_as_struct.fields().find(field_name)?;
                        struct_fields.push(col_as_struct.column(field_index).clone());
                        struct_field_defs.push(field.clone());
                    }

                    // safety: `try_new` will return an error here if the types of arrays we pass
                    // for the fields do not match the field definitions, or if the arrays have
                    // different lengths. Based on the way we've constructed inputs, this should
                    // not happen because we've taken them from the input struct column in order
                    let projected_struct_arr = StructArray::try_new(
                        struct_field_defs.into(),
                        struct_fields,
                        col_as_struct.nulls().cloned(),
                    )
                    .expect("can init StructArray");

                    let projected_field = original_schema.fields[struct_index]
                        .as_ref()
                        .clone()
                        .with_data_type(projected_struct_arr.data_type().clone());
                    fields.push(Arc::new(projected_field));
                    columns.push(Arc::new(projected_struct_arr));
                }
            }
        }

        Some((fields, columns))
    }

    pub fn try_downcast_dicts(fields: &mut [Arc<Field>], columns: &mut [ArrayRef]) -> Result<()> {
        for i in 0..fields.len() {
            let field = &fields[i];
            if let DataType::Dictionary(_, v) = field.data_type() {
                let new_field = Arc::new(field.as_ref().clone().with_data_type(v.as_ref().clone()));
                let new_column = cast(&columns[i], v.as_ref())?;
                fields[i] = new_field;
                columns[i] = new_column;
            }
        }

        Ok(())
    }
}

/// Defines that the record batch should be projected as when the filter is applied.
///
/// Note that the only thing that matters when applying the filter's `PhysicalExpr` is that the
/// columns are all present and in the correct order, which is why this is implemented as a lists
/// of column names without regard to types.
type ProjectedSchema = Vec<ProjectedSchemaColumn>;

/// Definition of column in the projected schema
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd)]
pub(crate) enum ProjectedSchemaColumn {
    /// Simply column in the [`RecordBatch`] being filtered that should be in the projected schema
    Root(String),

    /// Columns that should be projected from a nested struct. For example on a Logs record batch
    /// this could be things like `resource.name`, or `body.str`.
    Struct(String, Vec<String>),
}

/// Implementation of [`TreeNodeVisitor`] that will visit the [`Expr`] defining the filter
/// predicate to determine which columns are referenced in the filter predicate. This information
/// can then be used to determine how to project the input batches before evaluating the filter's
/// [`PhysicalExpr`]
#[derive(Debug, Default)]
struct ProjectedSchemaExprVisitor {
    root_columns: HashSet<String>,

    // this is used to keep track of fields in some nested struct which are referenced by the expr.
    // the map is keyed by struct name, and the set contains the fields within the struct.
    struct_columns: HashMap<String, HashSet<String>>,
}

impl<'a> TreeNodeVisitor<'a> for ProjectedSchemaExprVisitor {
    type Node = Expr;

    fn f_down(&mut self, node: &'a Self::Node) -> datafusion::error::Result<TreeNodeRecursion> {
        if let Expr::Column(col) = node {
            _ = self.root_columns.insert(col.name.clone());
        }

        // here we're checking if the expression we're visiting references a field within a struct
        // column. The way we reference these in the plans we build is using an expression like
        // `col("scope").field("name")` which produces a ScalarFunction expression invoking the
        // `GetFieldFunc` function with arguments ("scope", "name").
        if let Expr::ScalarFunction(scalar_udf) = node {
            if scalar_udf
                .func
                .as_ref()
                .inner()
                .as_any()
                .is::<GetFieldFunc>()
            {
                let source = scalar_udf.args.first();
                let field = scalar_udf.args.get(1);
                match (source, field) {
                    (
                        Some(Expr::Column(col)),
                        Some(Expr::Literal(ScalarValue::Utf8(Some(nested_col)), _)),
                    ) => {
                        let struct_fields = self
                            .struct_columns
                            .entry(col.name.clone())
                            .or_insert(HashSet::new());
                        _ = struct_fields.insert(nested_col.clone());

                        // don't continue as we've found a column. Otherwise this will continue
                        // down the expression tree and we'll visit the Column expression twice.
                        return Ok(TreeNodeRecursion::Jump);
                    }
                    unexpected_args => {
                        let err_msg = format!(
                            "Found unexpected arguments to `GetFieldFunc`. Expected (Col, Literal(Utf8)) found {:?}",
                            unexpected_args
                        );
                        return Err(DataFusionError::Plan(err_msg));
                    }
                }
            }
        }

        Ok(TreeNodeRecursion::Continue)
    }
}

impl From<ProjectedSchemaExprVisitor> for ProjectedSchema {
    fn from(visitor: ProjectedSchemaExprVisitor) -> Self {
        let num_cols = visitor.root_columns.len()
            + visitor
                .struct_columns
                .values()
                .map(|cols| cols.len())
                .sum::<usize>();
        let mut schema = Vec::with_capacity(num_cols);

        for col in visitor.root_columns {
            schema.push(ProjectedSchemaColumn::Root(col))
        }

        for (struct_name, cols) in visitor.struct_columns {
            schema.push(ProjectedSchemaColumn::Struct(
                struct_name,
                cols.into_iter().collect(),
            ));
        }

        schema
    }
}

// =============================================================================
// AnyValue column split/stitch support
//
// AnyValue columns are struct columns tagged with `ANY_VALUE_METADATA_KEY` metadata.
// They contain a `type` discriminant (UInt8) and multiple typed value sub-columns.
// The routines below split a RecordBatch by AnyValue type signatures so that each
// partition has concrete (non-union) typed columns, allowing standard expression
// evaluation. Results are then stitched back to original row order.
// =============================================================================

use std::ops::Range;

use arrow::array::UInt8Array;
use arrow::compute::take;
use smallvec::SmallVec;

/// Contiguous row ranges sharing an AnyValue type. `SmallVec` avoids heap allocation
/// for the common case of one or two ranges per type.
pub(crate) type RowRanges = SmallVec<[Range<usize>; 2]>;

/// A partition of the original batch where all AnyValue columns have been resolved to
/// concrete typed columns.
pub(crate) struct AnyValuePartitionedBatch {
    pub batch: RecordBatch,
    /// Row ranges in the *original* (pre-split) batch that this partition covers.
    pub original_row_ranges: RowRanges,
}

/// Result of [`project_any_value_columns`].
pub(crate) struct AnyValueProjectionResult {
    pub partitions: Vec<AnyValuePartitionedBatch>,
}

/// Per-column analysis of the type distribution within an AnyValue column.
enum AnyValueTypeDistribution {
    /// Every row has the same type value.
    Uniform(AttributeValueType),
    /// Multiple distinct type values are present. Each entry contains the type and the
    /// contiguous row ranges where that type appears.
    Mixed(Vec<(AttributeValueType, RowRanges)>),
}

/// Detect AnyValue columns by structural shape: a struct field containing a sub-field
/// named `consts::ATTRIBUTE_TYPE` (`"type"`) with `DataType::UInt8`.
///
/// This avoids any explicit metadata bookkeeping — AnyValue columns are recognized
/// by their characteristic layout (type discriminant + value sub-columns).
pub(crate) fn find_any_value_columns(schema: &Schema) -> Vec<usize> {
    schema
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| is_any_value_field(f))
        .map(|(i, _)| i)
        .collect()
}

/// Returns `true` if a field has the shape of an AnyValue column: a struct containing
/// a `"type"` sub-field of `UInt8`.
fn is_any_value_field(field: &Field) -> bool {
    if let DataType::Struct(sub_fields) = field.data_type() {
        sub_fields
            .iter()
            .any(|sf| sf.name() == consts::ATTRIBUTE_TYPE && *sf.data_type() == DataType::UInt8)
    } else {
        false
    }
}

/// Extract the `type` sub-column from an AnyValue struct column.
fn extract_type_from_any_value_struct(column: &ArrayRef) -> Result<UInt8Array> {
    let struct_arr = column
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected AnyValue column to be a Struct, got {:?}",
                column.data_type()
            ),
        })?;

    let type_col = struct_arr
        .column_by_name(consts::ATTRIBUTE_TYPE)
        .ok_or_else(|| Error::ExecutionError {
            cause: "AnyValue struct column is missing the 'type' sub-column".into(),
        })?;

    type_col
        .as_any()
        .downcast_ref::<UInt8Array>()
        .cloned()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected AnyValue 'type' sub-column to be UInt8, got {:?}",
                type_col.data_type()
            ),
        })
}

/// Analyze the type distribution of a single AnyValue column in a single pass.
///
/// Walks the type array linearly, tracking contiguous runs. If all values are the same,
/// returns [`AnyValueTypeDistribution::Uniform`] without allocating. Otherwise, returns
/// [`AnyValueTypeDistribution::Mixed`] with run ranges coalesced per type.
fn compute_type_distribution(type_array: &UInt8Array) -> Result<AnyValueTypeDistribution> {
    if type_array.is_empty() {
        return Ok(AnyValueTypeDistribution::Uniform(AttributeValueType::Empty));
    }

    let first = type_array.value(0);
    let mut run_start: usize = 0;
    let mut current_type = first;
    let mut is_uniform = true;

    // We defer building the groups vec until we know we're mixed, to avoid allocating in
    // the uniform (common) case.
    let mut groups: Option<Vec<(u8, RowRanges)>> = None;

    for i in 1..type_array.len() {
        let t = type_array.value(i);
        if t != current_type {
            if is_uniform {
                // First time we see a different type — retroactively start collecting groups
                is_uniform = false;
                let mut g = Vec::new();
                push_range(&mut g, current_type, run_start..i);
                groups = Some(g);
            } else {
                push_range(
                    groups.as_mut().expect("initialized"),
                    current_type,
                    run_start..i,
                );
            }
            current_type = t;
            run_start = i;
        }
    }

    if is_uniform {
        let type_val = AttributeValueType::try_from(first).map_err(|_| Error::ExecutionError {
            cause: format!("invalid AnyValue type discriminant: {first}"),
        })?;
        return Ok(AnyValueTypeDistribution::Uniform(type_val));
    }

    // Flush the last run
    let g = groups.as_mut().expect("initialized");
    push_range(g, current_type, run_start..type_array.len());

    let mixed = groups
        .expect("initialized")
        .into_iter()
        .map(|(t, ranges)| {
            let type_val = AttributeValueType::try_from(t).map_err(|_| Error::ExecutionError {
                cause: format!("invalid AnyValue type discriminant: {t}"),
            })?;
            Ok((type_val, ranges))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AnyValueTypeDistribution::Mixed(mixed))
}

/// Push a range into the groups vec, coalescing with an existing entry for the same type.
fn push_range(groups: &mut Vec<(u8, RowRanges)>, type_val: u8, range: Range<usize>) {
    if let Some((_, ranges)) = groups.iter_mut().find(|(t, _)| *t == type_val) {
        ranges.push(range);
    } else {
        let mut ranges = SmallVec::new();
        ranges.push(range);
        groups.push((type_val, ranges));
    }
}

/// Map the name of an [`AttributeValueType`] to the column name within the AnyValue struct.
fn any_value_type_to_column_name(type_val: AttributeValueType) -> Result<&'static str> {
    match type_val {
        AttributeValueType::Str => Ok(consts::ATTRIBUTE_STR),
        AttributeValueType::Int => Ok(consts::ATTRIBUTE_INT),
        AttributeValueType::Double => Ok(consts::ATTRIBUTE_DOUBLE),
        AttributeValueType::Bool => Ok(consts::ATTRIBUTE_BOOL),
        AttributeValueType::Bytes => Ok(consts::ATTRIBUTE_BYTES),
        AttributeValueType::Empty => Err(Error::ExecutionError {
            cause: "cannot resolve a concrete column for AnyValue type Empty".into(),
        }),
        AttributeValueType::Map | AttributeValueType::Slice => Err(Error::NotYetSupportedError {
            message:
                "expression evaluation on non-scalar AnyValue types (Map/Slice) not yet supported"
                    .into(),
        }),
    }
}

/// Replace a single AnyValue struct column in a batch with its concrete typed sub-column.
///
/// The resulting column keeps the same name as the struct column but has the concrete Arrow
/// type of the selected sub-field. The AnyValue metadata is stripped from the field.
fn replace_single_any_value_with_concrete(
    batch: &RecordBatch,
    col_idx: usize,
    type_val: AttributeValueType,
) -> Result<RecordBatch> {
    let schema = batch.schema();
    let struct_field = schema.field(col_idx);
    let struct_col = batch.column(col_idx);

    let struct_arr = struct_col
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected AnyValue column '{}' to be a Struct, got {:?}",
                struct_field.name(),
                struct_col.data_type()
            ),
        })?;

    let sub_col_name = any_value_type_to_column_name(type_val)?;
    let concrete_col =
        struct_arr
            .column_by_name(sub_col_name)
            .ok_or_else(|| Error::ExecutionError {
                cause: format!(
                    "AnyValue struct column '{}' is missing sub-column '{}'",
                    struct_field.name(),
                    sub_col_name,
                ),
            })?;

    // Build new fields/columns, replacing the struct column at col_idx
    let mut new_fields: Vec<Arc<Field>> = Vec::with_capacity(schema.fields().len());
    let mut new_columns: Vec<ArrayRef> = Vec::with_capacity(batch.num_columns());

    for (i, field) in schema.fields().iter().enumerate() {
        if i == col_idx {
            // Replace the struct field with a concrete field: same name, concrete type, no
            // AnyValue metadata
            let concrete_field = Field::new(
                struct_field.name(),
                concrete_col.data_type().clone(),
                true, // value columns are nullable
            );
            new_fields.push(Arc::new(concrete_field));
            new_columns.push(concrete_col.clone());
        } else {
            new_fields.push(field.clone());
            new_columns.push(batch.column(i).clone());
        }
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(new_fields)),
        new_columns,
    )?)
}

/// Slice a [`RecordBatch`] to the rows described by `ranges`.
///
/// If the ranges form a single contiguous region, uses the zero-copy `RecordBatch::slice`.
/// Otherwise, builds an indices array and uses `arrow::compute::take`.
fn slice_batch_by_ranges(batch: &RecordBatch, ranges: &RowRanges) -> Result<RecordBatch> {
    if ranges.len() == 1 {
        let range = &ranges[0];
        return Ok(batch.slice(range.start, range.end - range.start));
    }

    // Build a UInt32 indices array from the ranges
    let total_rows: usize = ranges.iter().map(|r| r.len()).sum();
    let mut indices = Vec::with_capacity(total_rows);
    for range in ranges {
        for i in range.clone() {
            indices.push(i as u32);
        }
    }
    let indices_arr = arrow::array::UInt32Array::from(indices);

    // Take from each column
    let schema = batch.schema();
    let new_columns: Vec<ArrayRef> = batch
        .columns()
        .iter()
        .map(|col| take(col.as_ref(), &indices_arr, None).map_err(Error::from))
        .collect::<Result<_>>()?;

    Ok(RecordBatch::try_new(schema, new_columns)?)
}

/// Map local row ranges within a partition back to the original batch's row indices.
///
/// `partition_ranges` are the ranges this partition covers in the original batch.
/// `local_ranges` are ranges within this partition's (sub-)batch.
/// The result is the corresponding ranges in the original batch.
fn map_local_ranges_to_original(
    partition_ranges: &RowRanges,
    local_ranges: &RowRanges,
) -> RowRanges {
    // Build a flat mapping from local row index -> original row index.
    // This is necessary because the partition_ranges may not be contiguous, so we can't
    // just do simple offset arithmetic.
    //
    // Optimization: if partition_ranges is a single contiguous range, we can just add the
    // offset.
    if partition_ranges.len() == 1 {
        let offset = partition_ranges[0].start;
        return local_ranges
            .iter()
            .map(|r| (r.start + offset)..(r.end + offset))
            .collect();
    }

    // General case: build the full mapping
    let total_partition_rows: usize = partition_ranges.iter().map(|r| r.len()).sum();
    let mut local_to_original = Vec::with_capacity(total_partition_rows);
    for range in partition_ranges {
        for i in range.clone() {
            local_to_original.push(i);
        }
    }

    // Now map each local range to original indices. The result ranges may not be contiguous
    // in the original batch, so we need to coalesce adjacent indices into ranges.
    let mut result = RowRanges::new();
    for local_range in local_ranges {
        for local_idx in local_range.clone() {
            let original_idx = local_to_original[local_idx];
            // Try to extend the last range if this index is adjacent
            if let Some(last) = result.last_mut() {
                if last.end == original_idx {
                    last.end = original_idx + 1;
                    continue;
                }
            }
            result.push(original_idx..original_idx + 1);
        }
    }

    result
}

/// Split a [`RecordBatch`] by AnyValue type signatures, resolving each AnyValue struct
/// column to its concrete typed sub-column.
///
/// Processes AnyValue columns one at a time: for each column, uniform partitions get the
/// struct replaced inline; mixed partitions get split into sub-partitions per type. This
/// iterative approach avoids complex multi-column intersection logic.
///
/// If all AnyValue columns are uniform across all rows, returns a single partition (the
/// fast path).
pub(crate) fn project_any_value_columns(
    batch: &RecordBatch,
    any_value_indices: &[usize],
) -> Result<AnyValueProjectionResult> {
    if any_value_indices.is_empty() {
        return Ok(AnyValueProjectionResult {
            partitions: vec![AnyValuePartitionedBatch {
                batch: batch.clone(),
                original_row_ranges: SmallVec::from_elem(0..batch.num_rows(), 1),
            }],
        });
    }

    struct PartitionInProgress {
        batch: RecordBatch,
        original_row_ranges: RowRanges,
    }

    let mut current_partitions = vec![PartitionInProgress {
        batch: batch.clone(),
        original_row_ranges: SmallVec::from_elem(0..batch.num_rows(), 1),
    }];

    for &col_idx in any_value_indices {
        let mut next_partitions = Vec::new();

        for partition in current_partitions {
            if partition.batch.num_rows() == 0 {
                next_partitions.push(partition);
                continue;
            }

            let type_array = extract_type_from_any_value_struct(partition.batch.column(col_idx))?;
            let distribution = compute_type_distribution(&type_array)?;

            match distribution {
                AnyValueTypeDistribution::Uniform(type_val) => {
                    if matches!(type_val, AttributeValueType::Empty) {
                        // All rows are empty — skip this partition
                        continue;
                    }
                    let updated = replace_single_any_value_with_concrete(
                        &partition.batch,
                        col_idx,
                        type_val,
                    )?;
                    next_partitions.push(PartitionInProgress {
                        batch: updated,
                        original_row_ranges: partition.original_row_ranges,
                    });
                }
                AnyValueTypeDistribution::Mixed(type_groups) => {
                    for (type_val, local_ranges) in type_groups {
                        if matches!(type_val, AttributeValueType::Empty) {
                            continue;
                        }
                        let sub_batch = slice_batch_by_ranges(&partition.batch, &local_ranges)?;
                        let concrete =
                            replace_single_any_value_with_concrete(&sub_batch, col_idx, type_val)?;
                        let original_ranges = map_local_ranges_to_original(
                            &partition.original_row_ranges,
                            &local_ranges,
                        );
                        next_partitions.push(PartitionInProgress {
                            batch: concrete,
                            original_row_ranges: original_ranges,
                        });
                    }
                }
            }
        }

        current_partitions = next_partitions;
    }

    Ok(AnyValueProjectionResult {
        partitions: current_partitions
            .into_iter()
            .map(|p| AnyValuePartitionedBatch {
                batch: p.batch,
                original_row_ranges: p.original_row_ranges,
            })
            .collect(),
    })
}

/// Stitch partitioned expression evaluation results back into the original row order.
///
/// Each entry in `partition_results` is a `(result_array, original_row_ranges)` pair.
/// The result array contains the values for the rows indicated by the ranges.
///
/// If there is a single partition covering all rows, returns its array directly (fast path).
/// Otherwise, scatters each partition's values back to their original positions.
pub(crate) fn stitch_partitioned_results(
    partition_results: Vec<(ArrayRef, RowRanges)>,
    total_rows: usize,
) -> Result<ArrayRef> {
    debug_assert!(!partition_results.is_empty());

    // Fast path: single partition covering the full batch
    if partition_results.len() == 1 {
        let (ref arr, ref ranges) = partition_results[0];
        if ranges.len() == 1 && ranges[0] == (0..total_rows) {
            return Ok(Arc::clone(arr));
        }
    }

    // Collect all (original_row_index, partition_index, offset_within_partition) triples,
    // sorted by original row index, then build a take-indices array to interleave.
    //
    // We build a mapping: for each original row, which partition and which offset within
    // that partition's result array does it come from.
    let mut row_assignments: Vec<(usize, usize, usize)> = Vec::with_capacity(total_rows);
    for (part_idx, (_, ranges)) in partition_results.iter().enumerate() {
        let mut offset = 0usize;
        for range in ranges {
            for original_idx in range.clone() {
                row_assignments.push((original_idx, part_idx, offset));
                offset += 1;
            }
        }
    }
    row_assignments.sort_unstable_by_key(|(original_idx, _, _)| *original_idx);

    // All partitions must have the same data type for stitching to work. If they don't,
    // the expression evaluation has already produced the wrong types and we should error.
    let first_type = partition_results[0].0.data_type().clone();
    for (arr, _) in &partition_results[1..] {
        if *arr.data_type() != first_type {
            return Err(Error::ExecutionError {
                cause: format!(
                    "cannot stitch AnyValue partitions with different result types: {:?} vs {:?}",
                    first_type,
                    arr.data_type()
                ),
            });
        }
    }

    // Concatenate all partition arrays, then take in the correct order
    let partition_arrays: Vec<&dyn Array> = partition_results
        .iter()
        .map(|(arr, _)| arr.as_ref())
        .collect();
    let concatenated = arrow::compute::concat(&partition_arrays)?;

    // Build the take indices: for each original row (in sorted order), compute the global
    // index into the concatenated array.
    //
    // First, compute the starting offset of each partition within the concatenated array.
    let mut partition_offsets = Vec::with_capacity(partition_results.len());
    let mut offset = 0u32;
    for (arr, _) in &partition_results {
        partition_offsets.push(offset);
        offset += arr.len() as u32;
    }

    let take_indices: Vec<u32> = row_assignments
        .iter()
        .map(|(_, part_idx, offset_in_part)| partition_offsets[*part_idx] + *offset_in_part as u32)
        .collect();
    let take_indices_arr = arrow::array::UInt32Array::from(take_indices);

    let result = take(concatenated.as_ref(), &take_indices_arr, None)?;
    Ok(result)
}
