// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `AnyValue` column type partitioned projection stitching support.
//!
//! When evaluating expressions involving `AnyValue` types, some expressions may need to treat the
//! column as the concrete type by evaluating on the value column. This module contains utilities
//! projecting the any value into partitions of value columns where each partition has represents
//! a single value type. It also contains utilities for stitching the partitions back together into
//! a struct column representing the AnyValue.

use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, MutableArrayData, NullArray, RecordBatch, StructArray, UInt8Array,
    UInt8Builder, make_array,
};
use arrow::datatypes::{DataType, Field, Schema};
use otap_df_pdata::otap::transform::util::take_record_batch_ranges;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;
use smallvec::SmallVec;

use crate::error::{Error, Result};

/// Detect AnyValue columns by structural shape: a struct field containing a sub-field
/// named `consts::ATTRIBUTE_TYPE` (`"type"`) with `DataType::UInt8`.
///
/// This avoids any explicit metadata bookkeeping — AnyValue columns are recognized
/// by their characteristic layout (type discriminant + value fields).
pub(crate) fn find_any_value_columns(schema: &Schema) -> Vec<usize> {
    schema
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| is_any_value_field(f))
        .map(|(i, _)| i)
        .collect()
}

/// Returns `true` if a field has the shape of an AnyValue column.
///
/// Delegates to [`is_any_value_data_type`].
pub(crate) fn is_any_value_field(field: &Field) -> bool {
    is_any_value_data_type(field.data_type())
}

/// Returns `true` if the given [`DataType`] has the shape of an AnyValue: a struct containing
/// a `"type"` field of `UInt8`.
pub(crate) fn is_any_value_data_type(dt: &DataType) -> bool {
    if let DataType::Struct(fields) = dt {
        fields
            .iter()
            .any(|f| f.name() == consts::ATTRIBUTE_TYPE && *f.data_type() == DataType::UInt8)
    } else {
        false
    }
}

/// Extract the `type` field from an AnyValue struct column.
pub(crate) fn extract_type_from_any_value_struct(column: &ArrayRef) -> Result<UInt8Array> {
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
            cause: "AnyValue struct is missing the 'type' field".into(),
        })?;

    type_col
        .as_any()
        .downcast_ref::<UInt8Array>()
        .cloned()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected AnyValue 'type' field to be UInt8, got {:?}",
                type_col.data_type()
            ),
        })
}

pub(crate) fn fill_null_type_as_empty(column: &ArrayRef) -> Result<ArrayRef> {
    let type_col = extract_type_from_any_value_struct(column)?;
    if type_col.null_count() == 0 {
        // nothing to do
        return Ok(Arc::clone(column));
    }

    // safety: this will be Some because if we're here it means null_count != 0
    let type_nulls = type_col.nulls().expect("there are nulls");

    // set AttributeValueType::Empty at all the null indices
    let mut new_types = type_col.values().to_vec();
    let mut last_valid_end = 0;
    for (start, end) in type_nulls.valid_slices() {
        new_types[last_valid_end..start].fill(AttributeValueType::Empty as u8);
        last_valid_end = end;
    }
    new_types[last_valid_end..column.len()].fill(AttributeValueType::Empty as u8);

    let struct_arr = column
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected AnyValue column to be a Struct, got {:?}",
                column.data_type()
            ),
        })?;

    let (fields, mut columns, nulls) = struct_arr.clone().into_parts();
    // safety: if the fields didn't contain type col, we would have already returned an error
    let (field_index, _) = fields
        .find(consts::ATTRIBUTE_TYPE)
        .expect("contains type col");
    // fields[field_index]
    columns[field_index] = Arc::new(UInt8Array::from(new_types));

    Ok(Arc::new(StructArray::new(fields, columns, nulls)))
}

/// Contiguous row ranges sharing an AnyValue type.
///
/// `SmallVec` is used here to avoid the heap allocation in the case where the AnyValue is
/// entirely one uniform type, which would be common when we're working with attributes all
/// having the same key.
pub(crate) type RowRanges = SmallVec<[Range<usize>; 1]>;

/// Type distribution within an AnyValue column.
enum AnyValueTypeDistribution {
    /// Every row has the same type value.
    Uniform(AttributeValueType),
    /// Multiple distinct type values are present. Each entry contains the type and the
    /// contiguous row ranges where that type appears.
    Mixed(Vec<(AttributeValueType, RowRanges)>),
}

/// Analyze the type distribution of a single AnyValue column.
///
/// Finds contiguous runs of the same type value, then groups those runs by type. If all values
/// share the same type, returns [`AnyValueTypeDistribution::Uniform`]. Otherwise, returns
/// [`AnyValueTypeDistribution::Mixed`] with run ranges coalesced per type.
fn compute_type_distribution(type_array: &UInt8Array) -> Result<AnyValueTypeDistribution> {
    if type_array.is_empty() {
        return Ok(AnyValueTypeDistribution::Uniform(AttributeValueType::Empty));
    }

    let type_col: ArrayRef = Arc::new(type_array.clone());
    let partitions = arrow::compute::partition(&[type_col])?;
    let ranges = partitions.ranges();

    // Single partition → all rows have the same type value
    if ranges.len() <= 1 {
        let first = type_array.value(0);
        let type_val = AttributeValueType::try_from(first).map_err(|_| Error::ExecutionError {
            cause: format!("invalid AnyValue type discriminant: {first}"),
        })?;
        return Ok(AnyValueTypeDistribution::Uniform(type_val));
    }

    // Multiple partitions → group contiguous runs by type value, coalescing non-adjacent
    // runs that share the same type.
    let mut groups: Vec<(u8, RowRanges)> = Vec::new();
    for range in ranges {
        let type_val = type_array.value(range.start);
        if let Some((_, row_ranges)) = groups.iter_mut().find(|(t, _)| *t == type_val) {
            row_ranges.push(range);
        } else {
            groups.push((type_val, SmallVec::from_elem(range, 1)));
        }
    }

    let mixed = groups
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

/// Map an Arrow [`DataType`] to the corresponding [`AttributeValueType`] and AnyValue field name.
///
/// This is the reverse of [`any_value_type_to_column_name`] — given an expression result's
/// Arrow type, determine which AnyValue field it belongs in.
pub(crate) fn arrow_type_to_any_value_type(
    dt: &DataType,
) -> Result<(AttributeValueType, Option<&'static str>)> {
    match dt {
        DataType::Dictionary(_, v) => arrow_type_to_any_value_type(v.as_ref()),
        DataType::Utf8 => Ok((AttributeValueType::Str, Some(consts::ATTRIBUTE_STR))),
        DataType::Int64 => Ok((AttributeValueType::Int, Some(consts::ATTRIBUTE_INT))),
        DataType::Binary => Ok((AttributeValueType::Bytes, Some(consts::ATTRIBUTE_BYTES))),
        DataType::Boolean => Ok((AttributeValueType::Bool, Some(consts::ATTRIBUTE_BOOL))),
        DataType::Float64 => Ok((AttributeValueType::Double, Some(consts::ATTRIBUTE_DOUBLE))),
        DataType::Null => Ok((AttributeValueType::Empty, None)),
        other => Err(Error::ExecutionError {
            cause: format!("cannot map Arrow type {:?} to an AnyValue field", other),
        }),
    }
}

/// Wrap a concrete typed array into an AnyValue struct column.
///
/// Produces a [`StructArray`] with:
/// - A `type` field (UInt8) with a uniform discriminant for every row
/// - A single value field (named per the AnyValue convention, e.g. `"str"`, `"int"`) holding
///   the original values
///
/// This is the inverse of [`replace_single_any_value_with_concrete`].
pub(crate) fn wrap_as_any_value_struct(values: &ArrayRef) -> Result<ArrayRef> {
    let (type_val, field_name) = arrow_type_to_any_value_type(values.data_type())?;
    let num_rows = values.len();

    // lift the nulls off the original column for use in the struct
    let nulls = values.nulls().cloned();

    // Build uniform type discriminant column
    let mut types = vec![type_val as u8; num_rows];
    if let Some(nulls) = &nulls {
        let mut last_valid_end = 0;
        for (start, end) in nulls.valid_slices() {
            types[last_valid_end..start].fill(AttributeValueType::Empty as u8);
            last_valid_end = end;
        }
        // Fill trailing nulls after the last valid slice
        types[last_valid_end..num_rows].fill(AttributeValueType::Empty as u8);
    }

    let type_arr: ArrayRef = Arc::new(UInt8Array::from(types));

    let mut fields = vec![Arc::new(Field::new(
        consts::ATTRIBUTE_TYPE,
        DataType::UInt8,
        false,
    ))];
    let mut columns = vec![type_arr];

    // if this attribute type represents a column (e.g. if it's not a Null array representing
    // empty attributes), push the values field and column
    if let Some(field_name) = field_name {
        fields.push(Arc::new(Field::new(
            field_name,
            values.data_type().clone(),
            true,
        )));
        columns.push(Arc::clone(values))
    }

    let struct_arr = StructArray::try_new(fields.into(), columns, nulls)?;
    Ok(Arc::new(struct_arr))
}

/// Attempt to coerce the values array (a struct array representing an AnyValue) into the concrete
/// values type. If the passed array contains multiple types, the original array is returned
/// because coercion was not successful.
pub(crate) fn attempt_coerce_value_column_from_any_value_struct_column(
    values: &ArrayRef,
) -> Result<ArrayRef> {
    // build a temporary record batch containing the column to maybe partition by type
    let rb = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "",
            values.data_type().clone(),
            true,
        )])),
        vec![Arc::clone(values)],
    )?;

    // attempt to partition the AnyValue column by type
    let partitions = project_any_value_columns(&rb, &[0])?;
    let result = if partitions.len() == 1 {
        partitions[0].batch.column(0)
    } else {
        // just return the original array
        values
    };

    Ok(Arc::clone(result))
}

/// Replace a single AnyValue struct column in a batch with its concrete typed field.
///
/// The resulting column keeps the same name as the struct column but has the concrete Arrow
/// type of the selected field.
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

    let concrete_col = if type_val == AttributeValueType::Empty {
        Arc::new(NullArray::new(batch.num_rows()))
    } else {
        let sub_col_name = any_value_type_to_column_name(type_val)?;
        struct_arr
            .column_by_name(sub_col_name)
            .ok_or_else(|| Error::ExecutionError {
                cause: format!(
                    "AnyValue struct column '{}' is missing field '{}'",
                    struct_field.name(),
                    sub_col_name,
                ),
            })?
            .clone()
    };

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
/// Otherwise, delegates to [`take_record_batch_ranges`] which uses `MutableArrayData::extend`
/// to copy ranges directly without building intermediate index vectors.
fn slice_batch_by_ranges(batch: &RecordBatch, ranges: &RowRanges) -> Result<RecordBatch> {
    if ranges.len() == 1 {
        let range = &ranges[0];
        return Ok(batch.slice(range.start, range.end - range.start));
    }

    Ok(take_record_batch_ranges(batch, ranges.as_slice())?)
}

/// Map local row ranges within a partition back to the original batch's row indices.
///
/// `partition_ranges` are the ranges this partition covers in the original batch.
/// `local_ranges` are ranges within this partition's (sub-)batch.
/// The result is the corresponding ranges in the original batch.
///
/// Operates in chunks over the ranges directly — no per-row iteration or intermediate
/// allocation beyond a small cumulative-length array.
fn map_local_ranges_to_original(
    partition_ranges: &RowRanges,
    local_ranges: &RowRanges,
) -> RowRanges {
    if partition_ranges.len() == 1 {
        let offset = partition_ranges[0].start;
        return local_ranges
            .iter()
            .map(|r| (r.start + offset)..(r.end + offset))
            .collect();
    }

    // Precompute cumulative row counts so we can locate which partition range a local
    // index falls into. For partition_ranges [5..8, 12..15] this gives [0, 3, 6].
    let mut cumulative_lens: SmallVec<[usize; 3]> =
        SmallVec::with_capacity(partition_ranges.len() + 1);
    cumulative_lens.push(0);
    for range in partition_ranges {
        cumulative_lens.push(cumulative_lens.last().expect("non-empty") + range.len());
    }

    let mut result = RowRanges::new();

    for local_range in local_ranges {
        // Find which partition range the start of this local range falls into.
        let mut part_idx = cumulative_lens
            .partition_point(|&cum| cum <= local_range.start)
            .saturating_sub(1);

        let mut local_pos = local_range.start;
        while local_pos < local_range.end {
            let offset_in_part = local_pos - cumulative_lens[part_idx];
            let part_range = &partition_ranges[part_idx];

            // How many consecutive local indices we can map from this partition range
            let remaining_in_part = part_range.len() - offset_in_part;
            let remaining_in_local = local_range.end - local_pos;
            let chunk_len = remaining_in_part.min(remaining_in_local);

            let orig_start = part_range.start + offset_in_part;
            let orig_end = orig_start + chunk_len;

            // Coalesce with the previous range if adjacent in the original batch
            if let Some(last) = result.last_mut() {
                if last.end == orig_start {
                    last.end = orig_end;
                } else {
                    result.push(orig_start..orig_end);
                }
            } else {
                result.push(orig_start..orig_end);
            }

            local_pos += chunk_len;
            if chunk_len == remaining_in_part {
                part_idx += 1;
            }
        }
    }

    result
}

/// A partition of the original batch where all AnyValue columns have been resolved to
/// concrete typed columns.
#[derive(Debug)]
pub(crate) struct AnyValuePartitionedBatch {
    pub batch: RecordBatch,
    /// Row ranges in the *original* (pre-split) batch that this partition covers.
    pub original_row_ranges: RowRanges,
}

/// Split a [`RecordBatch`] by AnyValue type signatures, resolving each AnyValue struct
/// column to its concrete typed field.
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
) -> Result<Vec<AnyValuePartitionedBatch>> {
    if any_value_indices.is_empty() {
        return Ok(vec![AnyValuePartitionedBatch {
            batch: batch.clone(),
            original_row_ranges: SmallVec::from_elem(0..batch.num_rows(), 1),
        }]);
    }

    let mut current_partitions = vec![AnyValuePartitionedBatch {
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
                    let updated = replace_single_any_value_with_concrete(
                        &partition.batch,
                        col_idx,
                        type_val,
                    )?;
                    next_partitions.push(AnyValuePartitionedBatch {
                        batch: updated,
                        original_row_ranges: partition.original_row_ranges,
                    });
                }
                AnyValueTypeDistribution::Mixed(type_groups) => {
                    for (type_val, local_ranges) in type_groups {
                        let sub_batch = slice_batch_by_ranges(&partition.batch, &local_ranges)?;
                        let concrete =
                            replace_single_any_value_with_concrete(&sub_batch, col_idx, type_val)?;
                        let original_ranges = map_local_ranges_to_original(
                            &partition.original_row_ranges,
                            &local_ranges,
                        );
                        next_partitions.push(AnyValuePartitionedBatch {
                            batch: concrete,
                            original_row_ranges: original_ranges,
                        });
                    }
                }
            }
        }

        current_partitions = next_partitions;
    }

    Ok(current_partitions)
}

/// Stitch partitioned expression evaluation results back into the original row order.
///
/// Each entry in `partition_results` is a `(result_array, original_row_ranges)` pair.
/// The result array contains the values for the rows indicated by the ranges.
///
/// If there is a single partition covering all rows, returns its array directly (fast path).
/// If all partitions produce the same type, concatenates and reorders them.
/// If partitions produce different types, builds an AnyValue struct column with a `type`
/// discriminant and one value field per distinct type.
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

    // Check whether all partitions share the same type.
    let all_same_type = partition_results
        .iter()
        .skip(1)
        .all(|(arr, _)| arr.data_type() == partition_results[0].0.data_type());

    if all_same_type {
        stitch_same_type(&partition_results, total_rows)
    } else {
        stitch_as_any_value_struct(&partition_results, total_rows)
    }
}

/// Stitch partitions that all share the same Arrow data type into a single array in
/// original row order.
///
/// Builds the output array directly via [`MutableArrayData`] in one pass — no intermediate
/// concatenation or take-indices allocation.
fn stitch_same_type(
    partition_results: &[(ArrayRef, RowRanges)],
    total_rows: usize,
) -> Result<ArrayRef> {
    // Build (original_row_idx, partition_idx) pairs sorted by original row order.
    let mut merged_rows: Vec<(usize, usize)> = Vec::with_capacity(total_rows);
    for (part_idx, (_, ranges)) in partition_results.iter().enumerate() {
        for range in ranges {
            for orig_idx in range.clone() {
                merged_rows.push((orig_idx, part_idx));
            }
        }
    }
    merged_rows.sort_unstable_by_key(|(orig_idx, _)| *orig_idx);

    // Track per-partition cursors (offset within each partition's array).
    let mut part_cursors = vec![0usize; partition_results.len()];

    // Set up MutableArrayData with all partition arrays as sources.
    let source_arrays: Vec<&dyn Array> = partition_results
        .iter()
        .map(|(arr, _)| arr.as_ref())
        .collect();
    let source_data: Vec<_> = source_arrays.iter().map(|a| a.to_data()).collect();
    let source_data_refs: Vec<_> = source_data.iter().collect();
    let mut mutable = MutableArrayData::new(source_data_refs, false, total_rows);

    // Walk merged_rows, batching consecutive rows from the same partition into single
    // extend calls.
    let mut i = 0;
    while i < merged_rows.len() {
        let (_, part_idx) = merged_rows[i];
        let start_offset = part_cursors[part_idx];
        let mut count = 1;
        // Batch consecutive rows that come from the same partition with consecutive offsets.
        while i + count < merged_rows.len() {
            let (_, next_part) = merged_rows[i + count];
            if next_part == part_idx && part_cursors[part_idx] + count == start_offset + count {
                count += 1;
            } else {
                break;
            }
        }
        mutable.extend(part_idx, start_offset, start_offset + count);
        part_cursors[part_idx] += count;
        i += count;
    }

    Ok(make_array(mutable.freeze()))
}

/// Stitch partitions with different Arrow data types into an AnyValue struct column.
///
/// The struct has:
/// - A `type` field (UInt8): the [`AttributeValueType`] discriminant per row
/// - One value field per distinct result type, using standard AnyValue field names
///   (`str`, `int`, `double`, `bool`, `bytes`)
///
/// Each row has its value in the matching field; all other fields are null for that row.
fn stitch_as_any_value_struct(
    partition_results: &[(ArrayRef, RowRanges)],
    total_rows: usize,
) -> Result<ArrayRef> {
    // Map each partition to its AnyValue type info.
    let partition_type_info: Vec<(AttributeValueType, Option<&'static str>)> = partition_results
        .iter()
        .map(|(arr, _)| arrow_type_to_any_value_type(arr.data_type()))
        .collect::<Result<_>>()?;

    // Collect the distinct (field_name, AttributeValueType) pairs in insertion order.
    // We'll create one value column per distinct field name.
    let mut distinct_fields: Vec<(&'static str, AttributeValueType)> = Vec::new();
    for &(type_val, field_name) in &partition_type_info {
        if let Some(field_name) = field_name {
            if !distinct_fields.iter().any(|(n, _)| *n == field_name) {
                distinct_fields.push((field_name, type_val));
            }
        }
    }

    // Build a merged list of (original_row_idx, partition_idx) sorted by original row order.
    // Also track per-partition row cursors so we know the offset within each partition's array.
    let mut merged_rows: Vec<(usize, usize)> = Vec::with_capacity(total_rows);
    for (part_idx, (_, ranges)) in partition_results.iter().enumerate() {
        for range in ranges {
            for orig_idx in range.clone() {
                merged_rows.push((orig_idx, part_idx));
            }
        }
    }
    merged_rows.sort_unstable_by_key(|(orig_idx, _)| *orig_idx);

    // Compute the offset within each partition's array for each merged row.
    // merged_offsets[i] = offset into partition_results[merged_rows[i].1].0
    let mut part_cursors = vec![0usize; partition_results.len()];
    let mut merged_offsets: Vec<usize> = Vec::with_capacity(total_rows);
    for &(_, part_idx) in &merged_rows {
        merged_offsets.push(part_cursors[part_idx]);
        part_cursors[part_idx] += 1;
    }

    // 1. Build the `type` UInt8 column.
    let mut type_builder = UInt8Builder::with_capacity(total_rows);
    for &(_, part_idx) in &merged_rows {
        type_builder.append_value(partition_type_info[part_idx].0 as u8);
    }
    let type_array: ArrayRef = Arc::new(type_builder.finish());

    // 2. Build each value field column using MutableArrayData.
    //
    // For each distinct field, we iterate through merged_rows in order. For rows whose
    // partition maps to this field, we extend from the partition's array. For rows that
    // don't match, we extend_nulls.
    let mut value_fields: Vec<Arc<Field>> = Vec::with_capacity(distinct_fields.len());
    let mut value_columns: Vec<ArrayRef> = Vec::with_capacity(distinct_fields.len());

    for (field_name, _) in &distinct_fields {
        // Collect the partition indices that map to this field
        let matching_partitions: SmallVec<[usize; 4]> = partition_type_info
            .iter()
            .enumerate()
            .filter(|(_, (_, name))| name == &Some(*field_name))
            .map(|(idx, _)| idx)
            .collect();

        // All matching partitions should have the same data type for this field.
        // Use the first matching partition's array as the reference type.
        let first_matching = matching_partitions[0];
        let field_data_type = partition_results[first_matching].0.data_type().clone();

        // Prepare MutableArrayData source arrays — one per matching partition.
        let source_arrays: Vec<&dyn Array> = matching_partitions
            .iter()
            .map(|&pi| partition_results[pi].0.as_ref())
            .collect();
        let source_data: Vec<_> = source_arrays.iter().map(|a| a.to_data()).collect();
        let source_data_refs: Vec<_> = source_data.iter().collect();
        let mut mutable = MutableArrayData::new(source_data_refs, true, total_rows);

        // Map partition_idx -> index into the source_arrays vec (for MutableArrayData's
        // source index parameter).
        let mut part_to_source: Vec<Option<usize>> = vec![None; partition_results.len()];
        for (source_idx, &part_idx) in matching_partitions.iter().enumerate() {
            part_to_source[part_idx] = Some(source_idx);
        }

        // Walk merged_rows in order, batching consecutive extends/nulls.
        let mut i = 0;
        while i < merged_rows.len() {
            let (_, part_idx) = merged_rows[i];
            if let Some(source_idx) = part_to_source[part_idx] {
                // This row matches this field — find how many consecutive rows also match
                // the same source.
                let start = i;
                let start_offset = merged_offsets[i];
                i += 1;
                while i < merged_rows.len() {
                    let (_, next_part_idx) = merged_rows[i];
                    if part_to_source[next_part_idx] == Some(source_idx)
                        && merged_offsets[i] == start_offset + (i - start)
                    {
                        i += 1;
                    } else {
                        break;
                    }
                }
                mutable.extend(source_idx, start_offset, start_offset + (i - start));
            } else {
                // This row doesn't match — count consecutive non-matching rows.
                let start = i;
                i += 1;
                while i < merged_rows.len() {
                    let (_, next_part_idx) = merged_rows[i];
                    if part_to_source[next_part_idx].is_none() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                mutable.extend_nulls(i - start);
            }
        }

        value_fields.push(Arc::new(Field::new(*field_name, field_data_type, true)));
        value_columns.push(make_array(mutable.freeze()));
    }

    // 3. Assemble the struct.
    let mut all_fields = vec![Arc::new(Field::new(
        consts::ATTRIBUTE_TYPE,
        DataType::UInt8,
        false,
    ))];
    all_fields.extend(value_fields);

    let mut all_columns = vec![type_array];
    all_columns.extend(value_columns);

    let struct_array = StructArray::try_new(all_fields.into(), all_columns, None)?;
    Ok(Arc::new(struct_array))
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray, UInt8Array};
    use smallvec::smallvec;

    /// Helper: build a simple AnyValue struct column from type + value arrays.
    /// Produces a struct with fields: type (UInt8), str (Utf8, nullable),
    /// int (Int64, nullable), double (Float64, nullable).
    fn make_any_value_struct(
        types: &[u8],
        str_vals: Vec<Option<&str>>,
        int_vals: Vec<Option<i64>>,
        double_vals: Vec<Option<f64>>,
    ) -> StructArray {
        let type_arr: ArrayRef = Arc::new(UInt8Array::from(types.to_vec()));
        let str_arr: ArrayRef = Arc::new(StringArray::from(str_vals));
        let int_arr: ArrayRef = Arc::new(Int64Array::from(int_vals));
        let double_arr: ArrayRef = Arc::new(Float64Array::from(double_vals));

        let fields = vec![
            Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
            Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
            Arc::new(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true)),
            Arc::new(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                DataType::Float64,
                true,
            )),
        ];

        StructArray::try_new(
            fields.into(),
            vec![type_arr, str_arr, int_arr, double_arr],
            None,
        )
        .unwrap()
    }

    /// Helper: build a RecordBatch with a single AnyValue struct column named "value".
    fn make_any_value_batch(any_value: StructArray) -> RecordBatch {
        let field = Arc::new(Field::new("value", any_value.data_type().clone(), true));
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![field])),
            vec![Arc::new(any_value)],
        )
        .unwrap()
    }

    // =========================================================================
    // compute_type_distribution
    // =========================================================================

    #[test]
    fn test_compute_type_distribution_empty() {
        let arr = UInt8Array::from(Vec::<u8>::new());
        let dist = compute_type_distribution(&arr).unwrap();
        assert!(matches!(
            dist,
            AnyValueTypeDistribution::Uniform(AttributeValueType::Empty)
        ));
    }

    #[test]
    fn test_compute_type_distribution_single_element() {
        let arr = UInt8Array::from(vec![2u8]); // Int
        let dist = compute_type_distribution(&arr).unwrap();
        assert!(matches!(
            dist,
            AnyValueTypeDistribution::Uniform(AttributeValueType::Int)
        ));
    }

    #[test]
    fn test_compute_type_distribution_uniform() {
        let arr = UInt8Array::from(vec![1u8, 1, 1, 1]); // all Str
        let dist = compute_type_distribution(&arr).unwrap();
        assert!(matches!(
            dist,
            AnyValueTypeDistribution::Uniform(AttributeValueType::Str)
        ));
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_compute_type_distribution_contiguous_mixed() {
        // [Str, Str, Int, Int]
        let arr = UInt8Array::from(vec![1u8, 1, 2, 2]);
        let dist = compute_type_distribution(&arr).unwrap();
        match dist {
            AnyValueTypeDistribution::Mixed(groups) => {
                assert_eq!(groups.len(), 2);

                let (str_type, str_ranges) = &groups[0];
                assert_eq!(*str_type, AttributeValueType::Str);
                assert_eq!(str_ranges.as_slice(), &[0..2]);

                let (int_type, int_ranges) = &groups[1];
                assert_eq!(*int_type, AttributeValueType::Int);
                assert_eq!(int_ranges.as_slice(), &[2..4]);
            }
            _ => panic!("expected Mixed"),
        }
    }

    #[test]
    fn test_compute_type_distribution_alternating_mixed() {
        // [Str, Int, Str, Int]
        let arr = UInt8Array::from(vec![1u8, 2, 1, 2]);
        let dist = compute_type_distribution(&arr).unwrap();
        match dist {
            AnyValueTypeDistribution::Mixed(groups) => {
                assert_eq!(groups.len(), 2);

                let (str_type, str_ranges) = &groups[0];
                assert_eq!(*str_type, AttributeValueType::Str);
                assert_eq!(str_ranges.as_slice(), &[0..1, 2..3]);

                let (int_type, int_ranges) = &groups[1];
                assert_eq!(*int_type, AttributeValueType::Int);
                assert_eq!(int_ranges.as_slice(), &[1..2, 3..4]);
            }
            _ => panic!("expected Mixed"),
        }
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_compute_type_distribution_three_types_with_runs() {
        // [Int, Int, Int, Double, Double, Int]
        let arr = UInt8Array::from(vec![2u8, 2, 2, 3, 3, 2]);
        let dist = compute_type_distribution(&arr).unwrap();
        match dist {
            AnyValueTypeDistribution::Mixed(groups) => {
                assert_eq!(groups.len(), 2);

                let (int_type, int_ranges) = &groups[0];
                assert_eq!(*int_type, AttributeValueType::Int);
                assert_eq!(int_ranges.as_slice(), &[0..3, 5..6]);

                let (dbl_type, dbl_ranges) = &groups[1];
                assert_eq!(*dbl_type, AttributeValueType::Double);
                assert_eq!(dbl_ranges.as_slice(), &[3..5]);
            }
            _ => panic!("expected Mixed"),
        }
    }

    // =========================================================================
    // is_any_value_field
    // =========================================================================

    #[test]
    fn test_is_any_value_field_valid_struct() {
        let field = Field::new(
            "value",
            DataType::Struct(
                vec![
                    Arc::new(Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false)),
                    Arc::new(Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true)),
                ]
                .into(),
            ),
            true,
        );
        assert!(is_any_value_field(&field));
    }

    #[test]
    fn test_is_any_value_field_no_type_field() {
        let field = Field::new(
            "value",
            DataType::Struct(
                vec![Arc::new(Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Utf8,
                    true,
                ))]
                .into(),
            ),
            true,
        );
        assert!(!is_any_value_field(&field));
    }

    #[test]
    fn test_is_any_value_field_wrong_type_datatype() {
        let field = Field::new(
            "value",
            DataType::Struct(
                vec![Arc::new(Field::new(
                    consts::ATTRIBUTE_TYPE,
                    DataType::Int32,
                    false,
                ))]
                .into(),
            ),
            true,
        );
        assert!(!is_any_value_field(&field));
    }

    #[test]
    fn test_is_any_value_field_non_struct() {
        let field = Field::new("value", DataType::Utf8, true);
        assert!(!is_any_value_field(&field));
    }

    // =========================================================================
    // map_local_ranges_to_original
    // =========================================================================

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_map_local_ranges_single_partition_range() {
        // Partition covers original rows 5..10. Local range 1..4 → original 6..9.
        let partition_ranges: RowRanges = smallvec![5..10];
        let local_ranges: RowRanges = smallvec![1..4];
        let result = map_local_ranges_to_original(&partition_ranges, &local_ranges);
        assert_eq!(result.as_slice(), &[6..9]);
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_map_local_ranges_multi_partition_within_one() {
        // Partition covers [5..8, 12..15]. Local range 0..2 → original 5..7 (within first range).
        let partition_ranges: RowRanges = smallvec![5..8, 12..15];
        let local_ranges: RowRanges = smallvec![0..2];
        let result = map_local_ranges_to_original(&partition_ranges, &local_ranges);
        assert_eq!(result.as_slice(), &[5..7]);
    }

    #[test]
    fn test_map_local_ranges_spanning_boundary() {
        // Partition covers [5..8, 12..15]. Local indices 0..6 = all rows.
        // Local 0..3 → original 5..8, local 3..6 → original 12..15.
        // These are non-contiguous in original space, so we get two ranges.
        let partition_ranges: RowRanges = smallvec![5..8, 12..15];
        let local_ranges: RowRanges = smallvec![0..6];
        let result = map_local_ranges_to_original(&partition_ranges, &local_ranges);
        assert_eq!(result.as_slice(), &[5..8, 12..15]);
    }

    #[test]
    fn test_map_local_ranges_spanning_partial() {
        // Partition covers [5..8, 12..15]. Local range 2..5 spans the boundary:
        // local 2 → original 7, local 3 → original 12, local 4 → original 13.
        let partition_ranges: RowRanges = smallvec![5..8, 12..15];
        let local_ranges: RowRanges = smallvec![2..5];
        let result = map_local_ranges_to_original(&partition_ranges, &local_ranges);
        assert_eq!(result.as_slice(), &[7..8, 12..14]);
    }

    #[test]
    fn test_map_local_ranges_multiple_local_ranges() {
        // Partition covers [0..5]. Local ranges [1..3, 4..5].
        let partition_ranges: RowRanges = smallvec![0..5];
        let local_ranges: RowRanges = smallvec![1..3, 4..5];
        let result = map_local_ranges_to_original(&partition_ranges, &local_ranges);
        assert_eq!(result.as_slice(), &[1..3, 4..5]);
    }

    // =========================================================================
    // replace_single_any_value_with_concrete
    // =========================================================================

    #[test]
    fn test_replace_any_value_with_str() {
        let struct_col = make_any_value_struct(
            &[1, 1],                            // type: Str, Str
            vec![Some("hello"), Some("world")], // str values
            vec![None, None],                   // int: null
            vec![None, None],                   // double: null
        );
        let batch = make_any_value_batch(struct_col);

        let result =
            replace_single_any_value_with_concrete(&batch, 0, AttributeValueType::Str).unwrap();

        assert_eq!(result.num_columns(), 1);
        assert_eq!(result.schema().field(0).name(), "value");
        assert_eq!(*result.schema().field(0).data_type(), DataType::Utf8);

        let str_col = result
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(str_col.value(0), "hello");
        assert_eq!(str_col.value(1), "world");
    }

    #[test]
    fn test_replace_any_value_with_int() {
        let struct_col = make_any_value_struct(
            &[2, 2],                  // type: Int, Int
            vec![None, None],         // str: null
            vec![Some(42), Some(99)], // int values
            vec![None, None],         // double: null
        );
        let batch = make_any_value_batch(struct_col);

        let result =
            replace_single_any_value_with_concrete(&batch, 0, AttributeValueType::Int).unwrap();

        assert_eq!(*result.schema().field(0).data_type(), DataType::Int64);
        let int_col = result
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(int_col.value(0), 42);
        assert_eq!(int_col.value(1), 99);
    }

    // =========================================================================
    // project_any_value_columns
    // =========================================================================

    #[test]
    fn test_project_any_value_no_any_value_columns() {
        // A batch with no AnyValue struct columns — should pass through as single partition.
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Arc::new(Field::new(
                "x",
                DataType::Int64,
                false,
            ))])),
            vec![Arc::new(Int64Array::from(vec![1, 2, 3]))],
        )
        .unwrap();

        let partitions = project_any_value_columns(&batch, &[]).unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].batch.num_rows(), 3);
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_project_any_value_uniform_type() {
        // All rows are Str — should produce single partition with concrete Utf8 column.
        let struct_col = make_any_value_struct(
            &[1, 1, 1],
            vec![Some("a"), Some("b"), Some("c")],
            vec![None, None, None],
            vec![None, None, None],
        );
        let batch = make_any_value_batch(struct_col);

        let partitions = project_any_value_columns(&batch, &[0]).unwrap();
        assert_eq!(partitions.len(), 1);

        let result = &partitions[0].batch;
        assert_eq!(result.num_rows(), 3);
        assert_eq!(*result.schema().field(0).data_type(), DataType::Utf8);
        assert_eq!(partitions[0].original_row_ranges.as_slice(), &[0..3]);
    }

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn test_project_any_value_mixed_types() {
        // [Str, Str, Int, Int] — should produce two partitions.
        let struct_col = make_any_value_struct(
            &[1, 1, 2, 2],
            vec![Some("a"), Some("b"), None, None],
            vec![None, None, Some(10), Some(20)],
            vec![None, None, None, None],
        );
        let batch = make_any_value_batch(struct_col);

        let partitions = project_any_value_columns(&batch, &[0]).unwrap();
        assert_eq!(partitions.len(), 2);

        // First partition: Str rows
        let str_part = &partitions[0];
        assert_eq!(str_part.batch.num_rows(), 2);
        assert_eq!(
            *str_part.batch.schema().field(0).data_type(),
            DataType::Utf8
        );
        assert_eq!(str_part.original_row_ranges.as_slice(), &[0..2]);

        // Second partition: Int rows
        let int_part = &partitions[1];
        assert_eq!(int_part.batch.num_rows(), 2);
        assert_eq!(
            *int_part.batch.schema().field(0).data_type(),
            DataType::Int64
        );
        assert_eq!(int_part.original_row_ranges.as_slice(), &[(2..4)]);
    }

    #[test]
    fn test_project_any_value_all_empty() {
        // All rows have type=Empty — should produce zero partitions.
        let struct_col = make_any_value_struct(
            &[0, 0, 0],
            vec![None, None, None],
            vec![None, None, None],
            vec![None, None, None],
        );
        let batch = make_any_value_batch(struct_col);

        let partitions = project_any_value_columns(&batch, &[0]).unwrap();
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0].batch.column(0).data_type(), &DataType::Null);
    }

    #[test]
    fn test_project_any_value_mixed_with_empty() {
        // [Int, Empty, Int, Empty] — Empty rows skipped, Int rows kept.
        let struct_col = make_any_value_struct(
            &[2, 0, 2, 0],
            vec![None, None, None, None],
            vec![Some(10), None, Some(30), None],
            vec![None, None, None, None],
        );
        let batch = make_any_value_batch(struct_col);

        let partitions = project_any_value_columns(&batch, &[0]).unwrap();
        assert_eq!(partitions.len(), 2);

        let part = &partitions[0];
        assert_eq!(part.batch.num_rows(), 2);
        assert_eq!(*part.batch.schema().field(0).data_type(), DataType::Int64);
        // Original rows 0 and 2 (Empty rows 1 and 3 skipped).
        assert_eq!(part.original_row_ranges.as_slice(), &[0..1, 2..3]);
        // the 2nd projection should be placeholder for the empty attrs
        assert_eq!(partitions[1].batch.column(0).data_type(), &DataType::Null);
    }

    // =========================================================================
    // stitch_partitioned_results
    // =========================================================================

    #[test]
    fn test_stitch_single_partition_full_range() {
        let arr: ArrayRef = Arc::new(Int64Array::from(vec![10, 20, 30]));
        let ranges: RowRanges = smallvec![0..3];
        let result = stitch_partitioned_results(vec![(arr.clone(), ranges)], 3).unwrap();
        // Should return the same array (fast path).
        assert_eq!(result.len(), 3);
        let result_arr = result.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(result_arr.values(), &[10, 20, 30]);
    }

    #[test]
    fn test_stitch_same_type_interleaved() {
        // Partition 0 covers original rows 0, 2 with values [10, 30].
        // Partition 1 covers original rows 1, 3 with values [20, 40].
        // Stitched result should be [10, 20, 30, 40].
        let arr0: ArrayRef = Arc::new(Int64Array::from(vec![10, 30]));
        let arr1: ArrayRef = Arc::new(Int64Array::from(vec![20, 40]));
        let ranges0: RowRanges = smallvec![0..1, 2..3];
        let ranges1: RowRanges = smallvec![1..2, 3..4];

        let result = stitch_partitioned_results(vec![(arr0, ranges0), (arr1, ranges1)], 4).unwrap();

        assert_eq!(result.len(), 4);
        let result_arr = result.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(result_arr.values(), &[10, 20, 30, 40]);
    }

    #[test]
    fn test_stitch_same_type_contiguous_partitions() {
        // Partition 0: rows 0..2, values [1, 2]
        // Partition 1: rows 2..4, values [3, 4]
        let arr0: ArrayRef = Arc::new(Int64Array::from(vec![1, 2]));
        let arr1: ArrayRef = Arc::new(Int64Array::from(vec![3, 4]));
        let ranges0: RowRanges = smallvec![0..2];
        let ranges1: RowRanges = smallvec![2..4];

        let result = stitch_partitioned_results(vec![(arr0, ranges0), (arr1, ranges1)], 4).unwrap();

        assert_eq!(result.len(), 4);
        let result_arr = result.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(result_arr.values(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_stitch_mixed_types_produces_any_value_struct() {
        // Partition 0: rows 0, 2 with Int64 values [10, 30]
        // Partition 1: rows 1, 3 with Utf8 values ["b", "d"]
        // Result should be a struct with type, int, str fields.
        let arr0: ArrayRef = Arc::new(Int64Array::from(vec![10, 30]));
        let arr1: ArrayRef = Arc::new(StringArray::from(vec!["b", "d"]));
        let ranges0: RowRanges = smallvec![0..1, 2..3];
        let ranges1: RowRanges = smallvec![1..2, 3..4];

        let result = stitch_partitioned_results(vec![(arr0, ranges0), (arr1, ranges1)], 4).unwrap();

        // Result should be a struct array
        let struct_arr = result.as_any().downcast_ref::<StructArray>().unwrap();
        assert_eq!(struct_arr.len(), 4);

        // Check the type discriminant column
        let type_col = struct_arr
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(type_col.value(0), AttributeValueType::Int as u8);
        assert_eq!(type_col.value(1), AttributeValueType::Str as u8);
        assert_eq!(type_col.value(2), AttributeValueType::Int as u8);
        assert_eq!(type_col.value(3), AttributeValueType::Str as u8);

        // Check the int field: values at rows 0, 2; null at rows 1, 3
        let int_col = struct_arr
            .column_by_name(consts::ATTRIBUTE_INT)
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(int_col.value(0), 10);
        assert!(int_col.is_null(1));
        assert_eq!(int_col.value(2), 30);
        assert!(int_col.is_null(3));

        // Check the str field: null at rows 0, 2; values at rows 1, 3
        let str_col = struct_arr
            .column_by_name(consts::ATTRIBUTE_STR)
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert!(str_col.is_null(0));
        assert_eq!(str_col.value(1), "b");
        assert!(str_col.is_null(2));
        assert_eq!(str_col.value(3), "d");
    }

    // =========================================================================
    // arrow_type_to_any_value_type
    // =========================================================================

    #[test]
    fn test_arrow_type_to_any_value_type_mappings() {
        let cases = vec![
            (
                DataType::Utf8,
                AttributeValueType::Str,
                consts::ATTRIBUTE_STR,
            ),
            (
                DataType::Int64,
                AttributeValueType::Int,
                consts::ATTRIBUTE_INT,
            ),
            (
                DataType::Float64,
                AttributeValueType::Double,
                consts::ATTRIBUTE_DOUBLE,
            ),
            (
                DataType::Boolean,
                AttributeValueType::Bool,
                consts::ATTRIBUTE_BOOL,
            ),
            (
                DataType::Binary,
                AttributeValueType::Bytes,
                consts::ATTRIBUTE_BYTES,
            ),
        ];

        for (dt, expected_type, expected_name) in cases {
            let (type_val, name) = arrow_type_to_any_value_type(&dt).unwrap();
            assert_eq!(type_val, expected_type, "failed for {:?}", dt);
            assert_eq!(name, Some(expected_name), "failed for {:?}", dt);
        }
    }

    #[test]
    fn test_arrow_type_to_any_value_type_unsupported() {
        let result = arrow_type_to_any_value_type(&DataType::Date32);
        assert!(result.is_err());
    }
}
