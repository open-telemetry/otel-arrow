// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains an optimized function for upserting attribute values
//! [`upsert_attributes`], which merges new attribute values into an existing attributes
//! [`RecordBatch`].

use std::borrow::Cow;
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, LazyLock};

use arrow::util::bit_iterator::BitSliceIterator;
use smallvec::{SmallVec, smallvec};

use crate::otlp::attributes::AttributeValueType;
use crate::schema::consts;
use arrow::array::{
    Array, ArrayData, ArrayRef, ArrowPrimitiveType, BooleanArray, BooleanBufferBuilder,
    DictionaryArray, MutableArrayData, PrimitiveArray, RecordBatch, StringArray, UInt8Array,
    UInt16Array, UInt32Array, make_array,
};
use arrow::buffer::{MutableBuffer, NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::compute::{SlicesIterator, cast};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use datafusion::logical_expr::ColumnarValue;
use datafusion::scalar::ScalarValue;

use crate::error::{Error, Result};

/// Size of [`SmallVec`] instances.
///
/// There are several places within this code that we allocate vectors of some where the length is
/// capped at the number of unique keys being inserted. Ideally, we'd like to avoid the allocation
/// of all these small heap allocated vectors, which is why we use small vec. This number seems
/// like probably a reasonable number of insertions to accommodate before the underlying
/// implementation spills to something heap allocated.
const SMALLVEC_SIZE: usize = 16;

/// Empty placeholder record batch used when assigning attributes in cases where there is not a
/// pre-existing attributes record batch
pub static EMPTY_U16_ATTRS_RECORD_BATCH: LazyLock<RecordBatch> = LazyLock::new(|| {
    RecordBatch::new_empty(Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(
            consts::ATTRIBUTE_KEY,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        ),
    ])))
});

pub static EMPTY_U32_ATTRS_RECORD_BATCH: LazyLock<RecordBatch> = LazyLock::new(|| {
    RecordBatch::new_empty(Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt32, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(
            consts::ATTRIBUTE_KEY,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        ),
    ])))
});

/// A single attribute upsert specification for use with [`upsert_attributes`].
///
/// Each upsert targets a single attribute key and provides
/// - the mask identifying which existing rows have that key (e.g. a mask of which rows to update
///   values for this instance)
/// - the new values to assign
/// - and the ordered parent IDs (updates first, then inserts).
///
/// The specific ordering of parent_ids + the mask are build the new attribute record batch
/// efficiently by quickly merging the values with the existing columns and and appending new rows
/// for the inserts.
///
/// When multiple `AttributeUpsert`s are passed to [`upsert_attributes`], their keys **must be
/// distinct**. This invariant is not checked inside `upsert_attributes` — callers are responsible
/// for enforcing it (typically at the planner level). Passing duplicate keys results in undefined
/// behavior and may result in duplicate attributes.
pub struct AttributeUpsert<'a, T: ArrowPrimitiveType> {
    /// The attribute key being upserted (e.g., "http.method").
    pub attrs_key: &'a str,
    /// Boolean mask over existing rows: `true` where key matches `attrs_key`.
    pub existing_key_mask: BooleanArray,
    /// New values to assign (scalar broadcasts or per-parent array).
    pub new_values: ColumnarValue,
    /// Parent IDs: updates first, then inserts. Length = num_updates + num_inserts.
    pub upsert_parent_ids: PrimitiveArray<T>,
}

/// Pre-computed per-upsert state derived from an [`AttributeUpsert`].
struct ResolvedUpsert<'a, T: ArrowPrimitiveType> {
    attrs_key: &'a str,
    mask: &'a BooleanArray,
    target_col_name: Option<&'static str>,
    new_values_array: Option<ArrayRef>,
    new_values_scalar: Option<&'a ScalarValue>,
    upsert_parent_ids: &'a PrimitiveArray<T>,
    num_updates: usize,
    num_inserts: usize,

    /// the value that should be in the "type" column for non-null rows
    attr_value_type: AttributeValueType,
}

/// Upsert one or more attributes in an existing attributes record batch.
///
/// This function merges new attribute values into an existing attributes batch in a **single
/// pass** over the columns. It handles both updating existing attribute rows and inserting new
/// ones, while preserving dictionary encoding semantics and choosing optimal dictionary key
/// sizes.
///
/// # Arguments
///
/// * `existing_attrs` - The full attributes record batch containing all keys and parent IDs.
///   Columns: parent_id (UInt16), type (UInt8), key (Utf8/Dict), and optional value columns
///   (str, int, double, bool, bytes, ser).
///
/// * `upserts` - One or more [`AttributeUpsert`] specifications. Keys must be distinct across
///   all upserts.
///
/// # Returns
///
/// A new `RecordBatch` with the same schema as `existing_attrs` (potentially with new value
/// columns added), containing all non-updated (passthrough) rows, updated rows, and newly
/// inserted rows. Insert rows are appended in upsert order at the tail of the [`RecordBatch`]
pub fn upsert_attributes<T: ArrowPrimitiveType>(
    existing_attrs: &RecordBatch,
    upserts: &[AttributeUpsert<'_, T>],
) -> Result<RecordBatch> {
    let num_existing = existing_attrs.num_rows();

    // Resolve each upsert: determine type, target column, extract values, compute counts, and
    // other properties used to create the final result.
    let resolved: SmallVec<[ResolvedUpsert<'_, T>; SMALLVEC_SIZE]> = upserts
        .iter()
        .map(|u| {
            let data_type = u.new_values.data_type();
            let (attr_value_type, target_col_name) = resolve_attr_type_and_col_name(&data_type)?;
            let num_updates = u.existing_key_mask.true_count();
            let num_inserts = u.upsert_parent_ids.len() - num_updates;
            let new_values_array = match &u.new_values {
                ColumnarValue::Array(arr) => Some(Arc::clone(arr)),
                ColumnarValue::Scalar(_) => None,
            };
            let new_values_scalar = match &u.new_values {
                ColumnarValue::Scalar(s) => Some(s),
                ColumnarValue::Array(_) => None,
            };
            Ok(ResolvedUpsert {
                attrs_key: u.attrs_key,
                mask: &u.existing_key_mask,
                attr_value_type,
                target_col_name,
                new_values_array,
                new_values_scalar,
                upsert_parent_ids: &u.upsert_parent_ids,
                num_updates,
                num_inserts,
            })
        })
        .collect::<Result<_>>()?;

    let total_num_inserts: usize = resolved.iter().map(|r| r.num_inserts).sum();
    let total_output_rows = num_existing + total_num_inserts;

    // Build the row ownership index which indicates, for each existing range of existing values,
    // which upsert (if any) should write to it.
    let row_owners = build_row_owners(num_existing, &resolved);

    let existing_schema = existing_attrs.schema();

    // combined update row mask for all inserts. This is only used if we need to insert nulls into
    // some columns that have had their type changed and some rows must become null. It is
    // initialized lazily one time, if/when it is needed
    let mut combined_mask = None;

    let mut output_fields: Vec<Arc<Field>> =
        Vec::with_capacity(existing_schema.fields().len() + resolved.len());
    let mut output_columns: Vec<ArrayRef> =
        Vec::with_capacity(existing_schema.fields().len() + resolved.len());

    // Track which upserts were written to columns that already existed. This information will be
    // used to indicate whether we need to create new values columns for some attribute type that
    // may not have been present in the original schema.
    let mut written_target_cols: SmallVec<[bool; SMALLVEC_SIZE]> = smallvec![false; resolved.len()];

    // Build each output column by merging existing rows with new rows.
    for (col_idx, field) in existing_schema.fields().iter().enumerate() {
        let col_name = field.name().as_str();
        let existing_col = existing_attrs.column(col_idx);

        let (merged_field, merged_col) = match col_name {
            consts::PARENT_ID => merge_parent_id_column(
                field,
                Arc::clone(existing_col),
                &resolved,
                total_num_inserts,
            )?,

            consts::ATTRIBUTE_TYPE => merge_type_column(
                field,
                existing_col,
                &resolved,
                &row_owners,
                total_output_rows,
            )?,

            consts::ATTRIBUTE_KEY => {
                merge_key_column(field, existing_col, &resolved, total_output_rows)?
            }

            _ => {
                // Mark any upserts that target this column as "written".
                for (i, r) in resolved.iter().enumerate() {
                    if !written_target_cols[i] && r.target_col_name == Some(col_name) {
                        written_target_cols[i] = true;
                    }
                }
                merge_value_column(
                    field,
                    existing_col,
                    &resolved,
                    &row_owners,
                    &mut combined_mask,
                    col_name,
                    total_output_rows,
                )?
            }
        };

        let keep_column = match col_name {
            // for required columns always keep it
            consts::PARENT_ID | consts::ATTRIBUTE_TYPE | consts::ATTRIBUTE_KEY => true,
            // for non-required columns, only keep it if it's not all null
            _ => merged_col.null_count() < merged_col.len(),
        };

        if keep_column {
            output_fields.push(Arc::new(merged_field));
            output_columns.push(merged_col);
        }
    }

    // For any upserts targeting column that doesn't exist in the current schema create new column
    let mut created_cols: SmallVec<[&str; SMALLVEC_SIZE]> = SmallVec::new();
    for (i, r) in resolved.iter().enumerate() {
        if written_target_cols[i] {
            continue;
        }
        if let Some(col_name) = r.target_col_name {
            if created_cols.contains(&col_name) {
                // column already created in previous iteration of loop via some other upsert
                continue;
            }
            let (field, col) = create_new_value_column_batched(
                col_name,
                &resolved,
                &row_owners,
                total_output_rows,
            )?;
            output_fields.push(Arc::new(field));
            output_columns.push(col);
            created_cols.push(col_name);
        }
    }

    let output_schema = Arc::new(Schema::new(output_fields));
    // TODO no unwrap (I think we can expect here)
    Ok(RecordBatch::try_new(output_schema, output_columns).unwrap())
}

/// Resolve the OTAP attribute value type and target value column name from a data type.
///
/// Unwraps dictionary encoding to get the logical type, then maps to the appropriate
/// [`AttributeValueType`] and column name.
fn resolve_attr_type_and_col_name(
    data_type: &DataType,
) -> Result<(AttributeValueType, Option<&'static str>)> {
    // unwrap dictionary encoding to get the logical value type
    let logical_type = match data_type {
        DataType::Dictionary(_, v) => v.as_ref(),
        other => other,
    };

    match logical_type {
        DataType::Null => Ok((AttributeValueType::Empty, None)),
        DataType::Utf8 => Ok((AttributeValueType::Str, Some(consts::ATTRIBUTE_STR))),
        DataType::Boolean => Ok((AttributeValueType::Bool, Some(consts::ATTRIBUTE_BOOL))),
        DataType::Int64 => Ok((AttributeValueType::Int, Some(consts::ATTRIBUTE_INT))),
        DataType::Float64 => Ok((AttributeValueType::Double, Some(consts::ATTRIBUTE_DOUBLE))),
        DataType::Binary => Ok((AttributeValueType::Bytes, Some(consts::ATTRIBUTE_BYTES))),
        other => {
            // TODO - need an error for this
            todo!("TODO")
        } // other => Err(Error:: {
          //     cause: format!("unsupported attribute value type: {other:?}"),
          // }),
    }
}

/// A contiguous run of rows with the same ownership."Ownership" meaning the source of the rows
/// for each column in the final result.
#[derive(Debug)]
struct OwnershipRun {
    start: usize,
    end: usize,

    /// `None` for passthrough rows, meaning the rows were not updated and we should take the
    /// values for these rows from the original record batch.
    ///
    /// `Some(upsert_index)` for rows owned by an upsert, meaning we will calculate new row for
    /// each column using the resolved upsert at this index.
    owner: Option<usize>,
}

/// Build the row ownership index as a sorted list of contiguous runs.
///
/// Since upsert masks are mutually exclusive (each row has one key), at most one upsert can
/// own any given row.
fn build_row_owners<T: ArrowPrimitiveType>(
    num_existing: usize,
    resolved: &[ResolvedUpsert<'_, T>],
) -> Vec<OwnershipRun> {
    if num_existing == 0 {
        return Vec::new();
    }

    // K-way sorted merge of owned ranges from each upsert's mask.
    // Each SlicesIterator yields (start, end) ranges in sorted order. Since masks are
    // mutually exclusive, ranges from different upserts never overlap. We merge them
    // by always picking the iterator with the smallest next `start`, filling passthrough
    // gaps inline.
    let mut iters: SmallVec<[_; SMALLVEC_SIZE]> = resolved
        .iter()
        .map(|r| SlicesIterator::new(r.mask).peekable())
        .collect();

    let mut runs = Vec::new();
    let mut pos = 0;

    loop {
        // Find the iterator with the smallest next start.
        let mut best: Option<(usize, usize, usize)> = None; // (iter_idx, start, end)
        for (i, iter) in iters.iter_mut().enumerate() {
            if let Some(&(start, end)) = iter.peek() {
                // safety: we can expect because we've checked it's None in LHS of ||
                if best.is_none() || start < best.expect("best set").1 {
                    best = Some((i, start, end));
                }
            }
        }

        let Some((iter_idx, start, end)) = best else {
            break; // all iterators exhausted
        };

        // Advance the winning iterator.
        let _ = iters[iter_idx].next();

        debug_assert!(
            start >= pos,
            "overlapping ownership ranges at position {start} (previous end was {pos})"
        );

        // Fill passthrough gap before this range.
        if pos < start {
            runs.push(OwnershipRun {
                start: pos,
                end: start,
                owner: None,
            });
        }

        // Emit the owned range.
        runs.push(OwnershipRun {
            start,
            end,
            owner: Some(iter_idx),
        });
        pos = end;
    }

    // Trailing passthrough existing rows
    if pos < num_existing {
        runs.push(OwnershipRun {
            start: pos,
            end: num_existing,
            owner: None,
        });
    }

    runs
}

/// Build a combined boolean mask that is the OR of all upsert masks.
fn build_combined_mask<T: ArrowPrimitiveType>(
    resolved: &[ResolvedUpsert<'_, T>],
) -> Result<BooleanArray> {
    let mut combined = resolved[0].mask.clone();
    for upsert in &resolved[1..] {
        // TODO no unwrap (I think we can expect here)
        combined = arrow::compute::or(&combined, upsert.mask).unwrap();
    }
    Ok(combined)
}

/// Merge the `parent_id` column: existing rows unchanged, append insert parent IDs from each
/// upsert in order.
///
/// Uses `MutableArrayData` to copy the existing column and each upsert's insert portion
/// directly into a single output buffer — no intermediate slices or concat allocations.
fn merge_parent_id_column<T: ArrowPrimitiveType>(
    field: &Field,
    mut existing_col: ArrayRef,
    resolved: &[ResolvedUpsert<'_, T>],
    total_num_inserts: usize,
) -> Result<(Field, ArrayRef)> {
    if total_num_inserts == 0 {
        return Ok((field.as_ref().clone(), Arc::clone(&existing_col)));
    }

    // merge as dict if type is dict and all the values would fit
    if matches!(existing_col.data_type(), DataType::Dictionary(_, _)) {
        if let Some(merged_dict_column) =
            merge_parent_id_column_dict(&existing_col, resolved, total_num_inserts)?
        {
            return Ok((
                Field::new(
                    consts::PARENT_ID,
                    merged_dict_column.data_type().clone(),
                    false,
                ),
                merged_dict_column,
            ));
        }

        // fall back to native parent ID
        // safety: if we're here, we know the existing col must be a dictionary array with
        // the correct value type (u32), otherwise merge_parent_id_column_dict would have
        // returned an error. Casting Dict<_, u32> to u32 is will succeed
        existing_col = cast(&existing_col, &DataType::UInt32).expect("can safely cast to native")
    }

    let num_existing = existing_col.len();
    let total_output_rows = num_existing + total_num_inserts;

    // Source 0 = existing column, sources 1..N = each upsert's parent_ids array.
    let existing_data = existing_col.to_data();
    let upsert_data: SmallVec<[_; SMALLVEC_SIZE]> = resolved
        .iter()
        .map(|r| r.upsert_parent_ids.to_data())
        .collect();

    let mut sources = Vec::with_capacity(1 + resolved.len());
    sources.push(&existing_data);
    for d in &upsert_data {
        sources.push(d);
    }
    let mut mutable = MutableArrayData::new(sources, false, total_output_rows);

    // Bulk copy existing rows.
    mutable.extend(0, 0, num_existing);

    // Append insert portions from each upsert.
    for (i, r) in resolved.iter().enumerate() {
        if r.num_inserts > 0 {
            mutable.extend(i + 1, r.num_updates, r.num_updates + r.num_inserts);
        }
    }

    let result = make_array(mutable.freeze());

    Ok((
        field
            .as_ref()
            .clone()
            .with_data_type(existing_col.data_type().clone()),
        result,
    ))
}

fn merge_parent_id_column_dict<T: ArrowPrimitiveType>(
    existing_col: &ArrayRef,
    resolved: &[ResolvedUpsert<'_, T>],
    total_num_inserts: usize,
) -> Result<Option<ArrayRef>> {
    let existing_dict_values = match existing_col.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => {
                let dict_arr = existing_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("can downcast to dict");
                dict_arr.values()
            }
            DataType::UInt16 => {
                let dict_arr = existing_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("can downcast to dict");
                dict_arr.values()
            }
            _ => {
                todo!("invalid type")
            }
        },
        _ => {
            todo!("return error b/c we shouldn't have been called if not dict")
        }
    };

    let existing_dict_values_primitive = existing_dict_values
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::InvalidIdColumnType {
            data_type: existing_col.data_type().clone(),
        })?;

    // build a mapping of the existing dictionary keys
    let mut value_to_index = HashMap::with_capacity(existing_dict_values_primitive.len());
    for (i, val) in existing_dict_values_primitive.iter().enumerate() {
        if let Some(val) = val {
            _ = value_to_index.insert(val, i);
        }
    }

    let mut new_keys = Vec::with_capacity(total_num_inserts);
    let mut new_values = Vec::new();
    for r in resolved.iter() {
        let inserts = r
            .upsert_parent_ids
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap() // TODO need return error here that passed invalid arg
            .slice(r.num_updates, r.num_inserts);

        for val in inserts.iter() {
            if let Some(val) = val {
                let key = match value_to_index.get(&val) {
                    Some(key) => *key,
                    None => {
                        new_values.push(val);
                        let new_key = value_to_index.len();
                        if new_key >= 65535 {
                            return Ok(None);
                        }
                        _ = value_to_index.insert(val, new_key);
                        new_key
                    }
                };
                new_keys.push(key)
            } else {
                return Err(Error::NullParentId);
            }
        }
    }

    let result_values = if new_values.len() > 0 {
        let mut new_dict_values = Vec::with_capacity(existing_dict_values.len() + new_values.len());
        new_dict_values.extend_from_slice(existing_dict_values_primitive.values());
        new_dict_values.extend_from_slice(&new_values);
        Arc::new(UInt32Array::new(ScalarBuffer::from(new_dict_values), None))
    } else {
        Arc::clone(existing_dict_values)
    };

    let result = if value_to_index.len() <= 256 {
        let mut new_dict_keys = Vec::with_capacity(existing_col.len() + total_num_inserts);
        match existing_col.data_type() {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => {
                    let dict_arr = existing_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("can downcast to dict");
                    new_dict_keys.extend_from_slice(dict_arr.keys().values());
                }
                DataType::UInt16 => {
                    let dict_arr = existing_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can downcast to dict");
                    new_dict_keys.extend(dict_arr.keys().values().iter().map(|k| *k as u8));
                }
                _ => {
                    // safety: we've checked the type of existing column above and we know it is
                    // not any other type but those matched above
                    unreachable!("unreachable type")
                }
            },
            _ => {
                // safety: we've checked the type of existing column above and we know it is not
                // any other type but those matched above
                unreachable!("unreachable type")
            }
        };

        new_dict_keys.extend(new_keys.into_iter().map(|k| k as u8));
        Arc::new(DictionaryArray::new(
            UInt8Array::from(new_dict_keys),
            result_values,
        )) as ArrayRef
    } else {
        let mut new_dict_keys = Vec::with_capacity(existing_col.len() + total_num_inserts);
        match existing_col.data_type() {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => {
                    let dict_arr = existing_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("can downcast to dict");
                    new_dict_keys.extend(dict_arr.keys().values().iter().map(|k| *k as u16));
                }
                DataType::UInt16 => {
                    let dict_arr = existing_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can downcast to dict");
                    new_dict_keys.extend_from_slice(dict_arr.keys().values());
                }
                _ => {
                    // safety: we've checked the type of existing column above and we know it is
                    // not any other type but those matched above
                    unreachable!("unreachable type")
                }
            },
            _ => {
                // safety: we've checked the type of existing column above and we know it is not
                // any other type but those matched above
                unreachable!("unreachable type")
            }
        };

        new_dict_keys.extend(new_keys.into_iter().map(|k| k as u16));
        Arc::new(DictionaryArray::new(
            UInt16Array::from(new_dict_keys),
            result_values,
        )) as ArrayRef
    };

    Ok(Some(result))
}

/// Merge the `type` column for a batched upsert.
///
/// For passthrough runs: copy from existing in bulk.
/// For owned runs: fill with the owning upsert's type discriminant.
/// For insert rows: append each upsert's type discriminant
/// For any null values in the type data being updated/inserted, we set the type to Empty.
fn merge_type_column<T: ArrowPrimitiveType>(
    field: &Field,
    existing_col: &ArrayRef,
    resolved: &[ResolvedUpsert<'_, T>],
    row_owners: &[OwnershipRun],
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    let existing_types = existing_col
        .as_any()
        .downcast_ref::<UInt8Array>()
        .expect("type column is UInt8Array");

    let mut output = Vec::with_capacity(total_output_rows);

    // Per-upsert counters tracking how many update rows have been consumed from the values
    // array. Used to index into the null bitmap when the values array has nulls.
    let mut update_counters: SmallVec<[usize; SMALLVEC_SIZE]> = smallvec![0; resolved.len()];

    // Existing rows: copy contiguous runs while updating type to Empty for null rows
    for run in row_owners {
        match run.owner {
            None => {
                // Passthrough: bulk copy from existing.
                output.extend_from_slice(&existing_types.values()[run.start..run.end]);
            }
            Some(idx) => {
                let resolved_upsert = &resolved[idx];
                let count = run.end - run.start;
                let attr_type = resolved_upsert.attr_value_type as u8;

                if let Some(arr) = &resolved_upsert.new_values_array {
                    if let Some(nulls) = arr.nulls() {
                        // Array with nulls: write Empty for null values, attr_type for
                        // non-null.
                        let counter = &mut update_counters[idx];
                        let validity_iter =
                            BitSliceIterator::new(nulls.buffer().as_slice(), *counter, count);
                        let mut last_valid_range_end = 0;
                        for (start, end) in validity_iter {
                            if start != last_valid_range_end {
                                output.extend(std::iter::repeat_n(
                                    AttributeValueType::Empty as u8,
                                    start - last_valid_range_end,
                                ));
                            }
                            output.extend(std::iter::repeat_n(attr_type, end - start));
                            last_valid_range_end = end;
                        }
                        if last_valid_range_end != count {
                            output.extend(std::iter::repeat_n(
                                AttributeValueType::Empty as u8,
                                count - last_valid_range_end,
                            ));
                        }
                        *counter += count;
                        continue;
                    }
                }

                // No nulls (or scalar): fill with this upsert's type discriminant.
                output.extend(std::iter::repeat_n(attr_type, count));
            }
        }
    }

    // Insert rows: each upsert's type discriminant
    for resolved_upsert in resolved {
        let attr_type = resolved_upsert.attr_value_type as u8;
        if let Some(arr) = &resolved_upsert.new_values_array {
            if let Some(nulls) = arr.nulls() {
                let validity_slice_iter = BitSliceIterator::new(
                    nulls.buffer().as_slice(),
                    resolved_upsert.num_updates,
                    resolved_upsert.num_inserts,
                );
                let mut last_valid_range_end = 0;
                for (start, end) in validity_slice_iter {
                    // put the null range that came before the last valid range
                    if start != last_valid_range_end {
                        output.extend(std::iter::repeat_n(
                            AttributeValueType::Empty as u8,
                            start - last_valid_range_end,
                        ));
                    }
                    output.extend(std::iter::repeat_n(attr_type, end - start));

                    last_valid_range_end = end;
                }

                // put the remaining nulls
                if last_valid_range_end != resolved_upsert.num_inserts {
                    output.extend(std::iter::repeat_n(
                        AttributeValueType::Empty as u8,
                        resolved_upsert.num_inserts - last_valid_range_end,
                    ));
                }

                continue;
            }
        }

        output.extend(std::iter::repeat_n(attr_type, resolved_upsert.num_inserts));
    }

    let result: ArrayRef = Arc::new(UInt8Array::from(output));
    Ok((field.as_ref().clone(), result))
}

/// Merge inserted rows into the `key` column
///
/// Existing rows are unchanged (updates already have the correct key). Insert rows append each
/// upsert's key. This handles converting the key to correctly sized dictionary/native array if
/// any new keys would cause overflow.
fn merge_key_column<T: ArrowPrimitiveType>(
    field: &Field,
    existing_col: &ArrayRef,
    resolved: &[ResolvedUpsert<'_, T>],
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    let total_num_inserts: usize = resolved.iter().map(|r| r.num_inserts).sum();
    if total_num_inserts == 0 {
        return Ok((field.as_ref().clone(), Arc::clone(existing_col)));
    }

    // Collect (key, insert_count) pairs for all upserts that have inserts.
    let insert_keys: SmallVec<[(&str, usize); SMALLVEC_SIZE]> = resolved
        .iter()
        .filter(|r| r.num_inserts > 0)
        .map(|r| (r.attrs_key, r.num_inserts))
        .collect();

    match existing_col.data_type() {
        DataType::Dictionary(key_type, _) => merge_key_column_dict(
            field,
            existing_col,
            &insert_keys,
            total_output_rows,
            key_type,
        ),
        DataType::Utf8 => {
            let existing_str = existing_col
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("key column is StringArray");
            merge_key_column_plain(field, existing_str, &insert_keys, total_output_rows)
        }
        other => {
            // TODO need return the error for this
            todo!("TODO")
        } // other => Err(Error::ExecutionError {
          //     cause: format!("unexpected key column type: {other:?}"),
          // }),
    }
}

fn merge_key_column_dict(
    field: &Field,
    existing_col: &ArrayRef,
    insert_keys: &[(&str, usize)],
    total_output_rows: usize,
    key_type: &DataType,
) -> Result<(Field, ArrayRef)> {
    // Extract a reference to the dict values (StringArray).
    let dict_values: &StringArray = match key_type {
        DataType::UInt8 => {
            let dict = existing_col
                .as_any()
                // safety: save to expect at this point because we've already checked the type of
                // both the array and dictionary key type
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                .expect("key column is DictionaryArray<UInt8Type>");
            dict.values()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::ColumnDataTypeMismatch {
                    name: consts::ATTRIBUTE_KEY.into(),
                    expect: DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::Utf8),
                    ),
                    actual: dict.values().data_type().clone(),
                })?
        }
        DataType::UInt16 => {
            let dict = existing_col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                // safety: save to expect at this point because we've already checked the type of
                // both the array and dictionary key type
                .expect("key column is DictionaryArray<UInt16Type>");
            dict.values()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::ColumnDataTypeMismatch {
                    name: consts::ATTRIBUTE_KEY.into(),
                    expect: DataType::Dictionary(
                        Box::new(DataType::UInt16),
                        Box::new(DataType::Utf8),
                    ),
                    actual: dict.values().data_type().clone(),
                })?
        }
        other => {
            return Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                actual: key_type.clone(),
            });
        }
    };

    // Phase 1: For each insert key, find its existing dict index or assign a new one.
    // We don't rebuild the dict values array in the loop — just track novel keys.
    let existing_cardinality = dict_values.len();
    let mut novel_keys: SmallVec<[&str; SMALLVEC_SIZE]> = SmallVec::new();
    let mut key_indices: SmallVec<[u16; SMALLVEC_SIZE]> =
        SmallVec::with_capacity(insert_keys.len());

    for &(key_str, _count) in insert_keys {
        match index_of(dict_values, key_str) {
            Some(idx) => key_indices.push(idx as u16),
            None => {
                let new_idx = existing_cardinality + novel_keys.len();
                if new_idx >= 65535 {
                    // Overflow u16: fall back to plain Utf8.
                    // TODO - NO UNWRAP NEED ERROR
                    let decoded = cast(existing_col, &DataType::Utf8).unwrap();
                    let decoded_str = decoded
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect("cast to Utf8 yields StringArray");
                    return merge_key_column_plain(
                        field,
                        decoded_str,
                        insert_keys,
                        total_output_rows,
                    );
                }
                novel_keys.push(key_str);
                key_indices.push(new_idx as u16);
            }
        }
    }

    let final_cardinality = existing_cardinality + novel_keys.len();

    // Phase 2: Build new dict values if we have novel keys. Batch-append all at once.
    let final_values: ArrayRef = if novel_keys.is_empty() {
        Arc::new(dict_values.clone()) as ArrayRef
    } else {
        append_strings(dict_values, &novel_keys)
    };

    // Phase 3: build dictionary from new keys:
    match key_type {
        DataType::UInt8 if final_cardinality <= 255 => {
            // Stay u8. Copy existing keys buffer, append new u8 indices.
            let dict = existing_col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                // safety: save to expect at this point because we've already checked the type of
                // both the array and dictionary key type
                .expect("can downcast to Dict<u8>");

            // copy existing keys and append new ones
            let existing_keys_buf = dict.keys().values().inner();
            let mut keys_buf = MutableBuffer::with_capacity(total_output_rows);
            keys_buf.extend_from_slice(existing_keys_buf.as_slice());
            for (i, &(_key_str, count)) in insert_keys.iter().enumerate() {
                let idx = key_indices[i] as u8;
                for _ in 0..count {
                    keys_buf.push(idx);
                }
            }
            let keys = UInt8Array::new(
                ScalarBuffer::new(keys_buf.into(), 0, total_output_rows),
                None,
            );

            let result: ArrayRef = Arc::new(DictionaryArray::new(keys, final_values));
            let new_field = field
                .as_ref()
                .clone()
                .with_data_type(result.data_type().clone());
            Ok((new_field, result))
        }
        DataType::UInt8 => {
            // Overflow u8 → u16. Widen existing u8 keys to u16, then append new u16 indices.
            let dict = existing_col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                // safety: save to expect at this point because we've already checked the type of
                // both the array and dictionary key type
                .expect("can downcast to Dict<u8>");

            // copy existing keys and append new ones while converting everything to u16
            // because we've added new keys and overflowed the u8 dict
            let existing_u8_keys = dict.keys().values();
            let mut keys_buf = MutableBuffer::with_capacity(total_output_rows * size_of::<u16>());
            for &k in existing_u8_keys.as_ref() {
                keys_buf.push(k as u16);
            }
            for (i, &(_key_str, count)) in insert_keys.iter().enumerate() {
                let idx = key_indices[i];
                for _ in 0..count {
                    keys_buf.push(idx);
                }
            }
            let keys = UInt16Array::new(
                ScalarBuffer::new(keys_buf.into(), 0, total_output_rows),
                None,
            );

            let result: ArrayRef = Arc::new(DictionaryArray::new(keys, final_values));
            let new_field = field
                .as_ref()
                .clone()
                .with_data_type(result.data_type().clone());
            Ok((new_field, result))
        }
        DataType::UInt16 => {
            // Stay u16. Copy existing keys buffer, append new u16 indices.
            let dict = existing_col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                // safety: save to expect at this point because we've already checked the type of
                // both the array and dictionary key type
                .expect("can downcast to Dict<16>");

            // copy existing keys and append new ones
            let existing_keys_buf = dict.keys().values().inner();
            let mut keys_buf = MutableBuffer::with_capacity(total_output_rows * size_of::<u16>());
            keys_buf.extend_from_slice(existing_keys_buf.as_slice());
            for (i, &(_key_str, count)) in insert_keys.iter().enumerate() {
                let idx = key_indices[i];
                for _ in 0..count {
                    keys_buf.push(idx);
                }
            }
            let keys = UInt16Array::new(
                ScalarBuffer::new(keys_buf.into(), 0, total_output_rows),
                None,
            );

            let result: ArrayRef = Arc::new(DictionaryArray::new(keys, final_values));
            let new_field = field
                .as_ref()
                .clone()
                .with_data_type(result.data_type().clone());
            Ok((new_field, result))
        }
        _ => unreachable!("key_type already validated above"),
    }
}

/// Build a plain Utf8 key column with newly insert keys
fn merge_key_column_plain(
    field: &Field,
    existing_str: &StringArray,
    insert_keys: &[(&str, usize)],
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    let existing_offsets = existing_str.offsets();
    let existing_values = existing_str.values();

    // Compute total insert bytes needed.
    let total_insert_bytes: usize = insert_keys.iter().map(|(k, count)| k.len() * count).sum();

    // Build the values buffer: existing bytes + insert key bytes.
    let new_values_len = existing_values.len() + total_insert_bytes;
    let mut values_buf = MutableBuffer::with_capacity(new_values_len);
    values_buf.extend_from_slice(existing_values);
    for &(key_str, count) in insert_keys {
        let key_bytes = key_str.as_bytes();
        for _ in 0..count {
            values_buf.extend_from_slice(key_bytes);
        }
    }

    // Build the offsets buffer: existing offsets + insert offsets.
    let mut offsets_buf = MutableBuffer::with_capacity((total_output_rows + 1) * size_of::<i32>());
    offsets_buf.extend_from_slice(existing_offsets.inner().as_ref());
    let mut offset = *existing_offsets.last().unwrap_or(&0);
    for &(key_str, count) in insert_keys {
        let key_len = key_str.len() as i32;
        for _ in 0..count {
            offset += key_len;
            offsets_buf.push(offset);
        }
    }

    let offsets_len = total_output_rows + 1;
    let offsets = OffsetBuffer::new(ScalarBuffer::<i32>::new(offsets_buf.into(), 0, offsets_len));
    let result_arr: ArrayRef = Arc::new(StringArray::new(offsets, values_buf.into(), None));
    let new_field = field.as_ref().clone().with_data_type(DataType::Utf8);
    Ok((new_field, result_arr))
}

/// Search for a value in the string array. Returns the index if found.
fn index_of(string_arr: &StringArray, value: &str) -> Option<usize> {
    (0..string_arr.len()).find(|&i| string_arr.value(i) == value)
}

/// Append a strings to a StringArray's underlying buffers, returning a new ArrayRef.
fn append_strings(existing: &StringArray, new_values: &[&str]) -> ArrayRef {
    let existing_offsets = existing.offsets();
    let existing_values_buf = existing.values();

    // values buffer: existing bytes + new string bytes
    let new_bytes_total: usize = new_values.iter().map(|new_val| new_val.len()).sum();
    let new_values_len = existing_values_buf.len() + new_bytes_total;
    let mut values_buf = MutableBuffer::with_capacity(new_values_len);
    values_buf.extend_from_slice(existing_values_buf);
    for new_value in new_values {
        values_buf.extend_from_slice(new_value.as_bytes());
    }

    // offsets buffer: existing offsets + one new offset
    let new_num_values = existing.len() + new_values.len();
    let mut offsets_buf = MutableBuffer::with_capacity((new_num_values + 1) * size_of::<i32>());
    offsets_buf.extend_from_slice(existing_offsets.inner().as_ref());
    let mut prev_offset = *existing_offsets.last().unwrap_or(&0);
    for new_value in new_values {
        let next_offset = prev_offset + new_value.len() as i32;
        offsets_buf.push(next_offset);
        prev_offset = next_offset;
    }

    let offsets = OffsetBuffer::new(ScalarBuffer::<i32>::new(
        offsets_buf.into(),
        0,
        new_num_values + 1,
    ));
    Arc::new(StringArray::new(offsets, values_buf.into(), None))
}

/// Merge an existing value column.
fn merge_value_column<T: ArrowPrimitiveType>(
    field: &Field,
    existing_col: &ArrayRef,
    resolved: &[ResolvedUpsert<'_, T>],
    row_owners: &[OwnershipRun],
    combined_mask: &mut Option<BooleanArray>,
    col_name: &str,
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    // Determine which upserts are making modifications to this column.
    let active_indices: SmallVec<[usize; SMALLVEC_SIZE]> = resolved
        .iter()
        .enumerate()
        .filter(|(_, r)| r.target_col_name == Some(col_name))
        .map(|(i, _)| i)
        .collect();

    // If no upserts are active, this is purely a "passthrough column", where we're placing nulls
    // in the rows where there were updated/inserted values in some other column.
    if active_indices.is_empty() {
        if combined_mask.is_none() {
            // lazily initialize the null mask
            *combined_mask = Some(build_combined_mask(resolved)?);
        }
        // safety: we can expect here because we've initialized the null mask on the line above
        let combined_mask = combined_mask.as_ref().expect("initialized");
        let total_num_inserts: usize = resolved.iter().map(|r| r.num_inserts).sum();
        return merge_passthrough_column(
            field,
            existing_col,
            combined_mask,
            total_num_inserts,
            total_output_rows,
        );
    }

    // try to merge into a dict column if supported and new values will fit
    let dict_encoding_supported = values_column_supports_dictionary_encoding(col_name);
    if dict_encoding_supported
        && let Some(unified) = try_build_unified_dict_multi(Some(existing_col), resolved, col_name)?
    {
        return merge_values_with_unified_dict(
            field,
            row_owners,
            &unified,
            resolved,
            total_output_rows,
        );
    }

    // dict encoding is either not supported, or the new column has too high cardinality so
    // fallback to plain encoded column
    let existing_plain = decode_to_plain(existing_col)?;
    let mut source_arrays: SmallVec<[ArrayRef; SMALLVEC_SIZE]> =
        SmallVec::with_capacity(1 + active_indices.len());
    source_arrays.push(existing_plain);

    // Direct-indexed by upsert_idx: None for inactive upserts.
    let mut active_sources: SmallVec<[Option<ActiveSource>; SMALLVEC_SIZE]> =
        smallvec![None; resolved.len()];

    for &active_idx in &active_indices {
        let resolved_upsert = &resolved[active_idx];
        let arr = match &resolved_upsert.new_values_array {
            Some(arr) => Arc::clone(arr),
            // safety: OK to expect here because when we construct resolved upsert, we set either
            // the new_values_array as Some or new_values_scalar to Some, never neither
            None => resolved_upsert
                .new_values_scalar
                .expect("is scalar")
                .to_array()
                // TODO NO UNWRAP NEED ERROR
                .unwrap(),
        };
        let is_scalar = resolved_upsert.new_values_array.is_none();
        let arr = decode_to_plain(&arr)?;

        let source_idx = source_arrays.len();
        source_arrays.push(arr);
        active_sources[active_idx] = Some(ActiveSource {
            source_idx,
            is_scalar,
            num_updates: resolved_upsert.num_updates,
            num_inserts: resolved_upsert.num_inserts,
        });
    }

    let source_data: SmallVec<[_; SMALLVEC_SIZE]> =
        source_arrays.iter().map(|a| a.to_data()).collect();
    let source_refs: Vec<_> = source_data.iter().collect();
    let mut mutable = MutableArrayData::new(source_refs, true, total_output_rows);

    // Per-upsert update counters, indexed by upsert position. Inactive slots stay at 0.
    let mut update_counters: SmallVec<[usize; SMALLVEC_SIZE]> = smallvec![0; resolved.len()];

    // Existing rows: iterate ownership runs.
    for run in row_owners {
        let count = run.end - run.start;
        match run.owner {
            None => {
                mutable.extend(0, run.start, run.end);
            }
            Some(owner_idx) => {
                if let Some(active) = &active_sources[owner_idx] {
                    if active.is_scalar {
                        for _ in 0..count {
                            mutable.extend(active.source_idx, 0, 1);
                        }
                    } else {
                        let counter = &mut update_counters[owner_idx];
                        mutable.extend(active.source_idx, *counter, *counter + count);
                        *counter += count;
                    }
                } else {
                    mutable.extend_nulls(count);
                }
            }
        }
    }

    // Insert rows.
    for (upsert_idx, r) in resolved.iter().enumerate() {
        if r.num_inserts == 0 {
            continue;
        }
        if let Some(active) = &active_sources[upsert_idx] {
            if active.is_scalar {
                for _ in 0..r.num_inserts {
                    mutable.extend(active.source_idx, 0, 1);
                }
            } else {
                mutable.extend(
                    active.source_idx,
                    active.num_updates,
                    active.num_updates + active.num_inserts,
                );
            }
        } else {
            mutable.extend_nulls(r.num_inserts);
        }
    }

    let result = make_array(mutable.freeze());
    let new_field = field
        .as_ref()
        .clone()
        .with_data_type(result.data_type().clone())
        .with_nullable(true);

    Ok((new_field, result))
}

/// Decode a potentially dict-encoded column to its plain (non-dict) representation.
/// If the column is already plain, returns a clone of the Arc.
fn decode_to_plain(col: &ArrayRef) -> Result<ArrayRef> {
    match col.data_type() {
        // TODO NO UNWRAP NEED ERROR
        DataType::Dictionary(_, value_type) => Ok(cast(col, value_type.as_ref()).unwrap()),
        _ => Ok(Arc::clone(col)),
    }
}

/// Per-active-upsert key mapping into a unified dictionary.
enum ActiveKeys {
    /// A scalar value: broadcast this single key for all update/insert rows.
    Scalar(u16),
    /// Per-row key remap for an array-valued upsert.
    Array(Vec<u16>),
}

/// Identifier of a upsert that is being used to merge new values to some particular
/// values column. Used internally when merging new values into existing values column
/// or when creating a new values column.
#[derive(Clone, Copy, Debug)]
struct ActiveSource {
    source_idx: usize,
    is_scalar: bool,
    num_updates: usize,
    num_inserts: usize,
}

/// Result of building a unified dictionary across the existing column and multiple new value
/// sources (one per active upsert).
struct UnifiedDictMulti {
    /// The deduplicated unified dictionary values array.
    values: ArrayRef,
    /// Per-row key mapping for existing column rows → unified dict keys.
    /// `None` when there is no existing column (creating a new column).
    existing_keys: Option<Vec<u16>>,

    /// The null buffer from the existing dictionary keys
    existing_nulls: Option<NullBuffer>,

    /// Per-upsert key mappings, indexed by position in `resolved`. `None` for upserts that
    /// are inactive (don't target this column).
    new_keys: SmallVec<[Option<ActiveKeys>; SMALLVEC_SIZE]>,
}

/// Attempt to build a unified dictionary from the existing column (if any) and the resolved
/// upserts.
///
/// When `existing_col` is `Some`, the existing column's values are included in the unified
/// dictionary and `existing_keys` in the result will be `Some`. When `existing_col` is `None`
/// (creating a new column), `existing_keys` will be `None`.
///
/// Only upserts whose `target_col_name` matches `col_name` are considered active. Inactive
/// upserts get `None` in the returned `new_keys` array.
///
/// Returns `Ok(None)` if the combined cardinality exceeds u16 max (65535), indicating the
/// caller should fall back to the primitive merge path.
///
/// Scalars are handled efficiently — only one value is added to the dict, producing an
/// `ActiveKeys::Scalar(key)` that the merge phase can broadcast without expansion.
fn try_build_unified_dict_multi<T: ArrowPrimitiveType>(
    existing_col: Option<&ArrayRef>,
    resolved: &[ResolvedUpsert<'_, T>],
    col_name: &str,
) -> Result<Option<UnifiedDictMulti>> {
    const MAX_DICT_CARDINALITY: usize = 65535;

    // Map from value (bytes) to unified dict index
    let mut value_to_idx: HashMap<Option<&[u8]>, u16> = HashMap::new();
    let mut num_distinct: usize = 0;

    // extract either reference to the array (if source is array), otherwise convert the
    // scalar into a unit length array. We keep them in this vec so that they live long enough
    // that we can borrow the dict keys later on if they're already the correct type.
    let mut maybe_scalar_arrays: SmallVec<[_; SMALLVEC_SIZE]> = smallvec![None; resolved.len()];
    for (i, r) in resolved.iter().enumerate() {
        if r.target_col_name != Some(col_name) {
            continue;
        }
        let (arr, is_scalar) = match &r.new_values_array {
            Some(arr) => (Arc::clone(arr), false),
            // safety: OK to expect here because when we construct resolved upsert, we set either
            // the new_values_array as Some or new_values_scalar to Some, never neither
            // TODO NO UNWRAP NEED ERROR
            None => (
                r.new_values_scalar.expect("is scalar").to_array().unwrap(),
                true,
            ),
        };
        maybe_scalar_arrays[i] = Some((arr, is_scalar));
    }

    // Each entry is `Some(...)` for active upserts (targeting this column), `None` for inactive.
    struct NewSource<'a> {
        values_arr: ArrayRef,
        dict_keys: Option<Cow<'a, [u16]>>,
        is_scalar: bool,
    }
    let mut new_sources: SmallVec<[Option<NewSource<'_>>; SMALLVEC_SIZE]> =
        (0..resolved.len()).map(|_| None).collect();
    for (i, maybe_scalar_arr) in maybe_scalar_arrays.iter().enumerate() {
        let Some((arr, is_scalar)) = maybe_scalar_arr else {
            continue;
        };
        let (values_arr, dict_keys) = match arr.data_type() {
            DataType::Dictionary(key_type, _) => {
                let (vals, keys) = extract_dict_any_info(arr, key_type)?;
                (vals, Some(keys))
            }
            _ => (Arc::clone(arr), None),
        };
        new_sources[i] = Some(NewSource {
            values_arr,
            dict_keys,
            is_scalar: *is_scalar,
        });
    }

    // Extract existing dict values and per-row keys (if existing column provided).
    let existing_info: Option<(ArrayRef, Option<Cow<'_, [u16]>>)> = match existing_col {
        Some(col) => {
            let (values_arr, dict_keys) = match col.data_type() {
                DataType::Dictionary(key_type, _) => {
                    let (vals, keys) = extract_dict_any_info(col, key_type)?;
                    (vals, Some(keys))
                }
                _ => (Arc::clone(col), None),
            };
            Some((values_arr, dict_keys))
        }
        None => None,
    };

    // Build a bitset of "live" dict value indices — values still referenced by at least one
    // passthrough (non-updated) row. This prevents dead values from being included in the
    // unified dictionary, which is important for data redaction scenarios.
    //
    // The bitset covers all 65536 possible dict value indices (1024 * 64 bits = 8KB).
    let mut live_values = [0u64; 1024];
    if let Some(existing_col) = existing_col {
        let num_existing = existing_col.len();

        // Build combined mask (OR of all upsert masks). Rows where mask=true are being
        // updated/nulled; rows where mask=false are passthrough and keep their values.
        let combined_mask = build_combined_mask(resolved)?;

        // Get reference to existing dict keys for lookup.
        let existing_dict_keys_ref = existing_info
            .as_ref()
            .and_then(|(_, dk)| dk.as_ref().map(|k| k.as_ref()));

        // Iterate passthrough rows (gaps between true ranges in combined_mask) and mark
        // their dict value indices as live.
        let mut pos = 0;
        for (start, end) in SlicesIterator::new(&combined_mask) {
            // Gap before this true range = passthrough rows
            for row in pos..start {
                let value_idx = match existing_dict_keys_ref {
                    Some(keys) => keys[row] as usize,
                    None => row, // non-dict: row index IS value index
                };
                live_values[value_idx / 64] |= 1u64 << (value_idx % 64);
            }
            pos = end;
        }
        // Trailing passthrough rows
        for row in pos..num_existing {
            let value_idx = match existing_dict_keys_ref {
                Some(keys) => keys[row] as usize,
                None => row,
            };
            live_values[value_idx / 64] |= 1u64 << (value_idx % 64);
        }
    }

    // Build source list for MutableArrayData.
    // If existing_col is Some: source 0 = existing, sources 1..N = new sources
    // If existing_col is None: sources 0..N = new sources only
    let existing_data: Option<ArrayData> = existing_info.as_ref().map(|(arr, _)| arr.to_data());

    let mut mutable_source_map: Vec<usize> = vec![0; resolved.len()];
    let mut source_data_vecs: Vec<ArrayData> = Vec::new();
    let source_offset = if existing_data.is_some() { 1 } else { 0 };
    let mut next_source_idx: usize = source_offset;
    for (i, ns) in new_sources.iter().enumerate() {
        if let Some(ns) = ns {
            source_data_vecs.push(ns.values_arr.to_data());
            mutable_source_map[i] = next_source_idx;
            next_source_idx += 1;
        }
    }

    let mut all_sources: Vec<&ArrayData> =
        Vec::with_capacity(source_offset + source_data_vecs.len());
    if let Some(ref existing_data) = existing_data {
        all_sources.push(existing_data);
    }
    for d in &source_data_vecs {
        all_sources.push(d);
    }

    let total_capacity = existing_info
        .as_ref()
        .map(|(arr, _)| arr.len())
        .unwrap_or(0)
        + new_sources
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.values_arr.len())
            .sum::<usize>();
    let mut unified_builder =
        MutableArrayData::new(all_sources, false, MAX_DICT_CARDINALITY.min(total_capacity));

    // Step 1: Seed HashMap from existing column's live values only (if any).
    // Dead values (only referenced by updated rows) are excluded to support data redaction.
    let existing_keys: Option<Vec<u16>> = match &existing_info {
        Some((existing_values_arr, existing_dict_keys)) => {
            let keys = if let Some(dict_keys) = existing_dict_keys {
                // Dict-encoded existing column: only add live values to the unified dict,
                // then remap existing keys through the filtered mapping.
                let n = existing_values_arr.len();

                // Build old_to_new remap: maps old value index → new value index.
                // Dead values get placeholder 0 (updated rows will be overwritten/nulled anyway).
                let mut old_to_new: Vec<u16> = vec![0; n];
                for i in 0..n {
                    let is_live = (live_values[i / 64] >> (i % 64)) & 1 == 1;
                    if !is_live {
                        continue;
                    }
                    if num_distinct >= MAX_DICT_CARDINALITY {
                        return Ok(None);
                    }
                    let key = array_value_as_bytes(existing_values_arr, i)?;
                    let idx = num_distinct as u16;
                    let _ = value_to_idx.insert(key, idx);
                    unified_builder.extend(0, i, i + 1);
                    old_to_new[i] = idx;
                    num_distinct += 1;
                }

                // Remap existing keys. Updated rows get placeholder 0 (will be overwritten).
                dict_keys
                    .iter()
                    .map(|&old_key| old_to_new[old_key as usize])
                    .collect()
            } else {
                // Non-dict existing column: only add live (passthrough) rows to the unified
                // dict, deduplicating as we go.
                let n = existing_values_arr.len();
                let mut remap: Vec<u16> = vec![0; n]; // placeholder for updated rows

                for row in 0..n {
                    let is_live = (live_values[row / 64] >> (row % 64)) & 1 == 1;
                    if !is_live {
                        continue; // Updated row — skip, leave remap as 0
                    }
                    let key = array_value_as_bytes(existing_values_arr, row)?;
                    if let Some(&idx) = value_to_idx.get(&key) {
                        remap[row] = idx;
                    } else {
                        if num_distinct >= MAX_DICT_CARDINALITY {
                            return Ok(None);
                        }
                        let idx = num_distinct as u16;
                        let _ = value_to_idx.insert(key, idx);
                        unified_builder.extend(0, row, row + 1);
                        num_distinct += 1;
                        remap[row] = idx;
                    }
                }

                remap
            };
            Some(keys)
        }
        None => None,
    };

    // Step 2: For each active upsert, build key mappings and append values to deduped dict values.
    // Inactive upserts are left as None in new_keys.
    let mut new_keys: SmallVec<[Option<ActiveKeys>; SMALLVEC_SIZE]> =
        (0..resolved.len()).map(|_| None).collect();

    for (upsert_idx, new_source) in new_sources.iter().enumerate() {
        let Some(new_source) = new_source else {
            continue;
        };
        let mutable_source_idx = mutable_source_map[upsert_idx];

        if new_source.is_scalar {
            // Scalar: just one value to lookup/add.
            let key_byte = array_value_as_bytes(&new_source.values_arr, 0)?;
            let unified_key = if let Some(&idx) = value_to_idx.get(&key_byte) {
                idx
            } else {
                if num_distinct >= MAX_DICT_CARDINALITY {
                    return Ok(None);
                }
                let idx = num_distinct as u16;
                let _ = value_to_idx.insert(key_byte, idx);
                unified_builder.extend(mutable_source_idx, 0, 1);
                num_distinct += 1;
                idx
            };
            new_keys[upsert_idx] = Some(ActiveKeys::Scalar(unified_key));
        } else {
            // Array: build full remap.
            let remap = build_values_remap(
                &new_source.values_arr,
                mutable_source_idx,
                &mut value_to_idx,
                &mut unified_builder,
                &mut num_distinct,
                MAX_DICT_CARDINALITY,
            )?;
            let remap = match remap {
                Some(r) => r,
                None => return Ok(None),
            };
            // If the new source was dict-encoded, remap through its dict keys.
            let final_keys = match &new_source.dict_keys {
                Some(dk) => dk.iter().map(|&k| remap[k as usize]).collect(),
                None => remap,
            };
            new_keys[upsert_idx] = Some(ActiveKeys::Array(final_keys));
        }
    }

    let unified_values = make_array(unified_builder.freeze());

    Ok(Some(UnifiedDictMulti {
        values: unified_values,
        existing_keys,
        existing_nulls: existing_col.map(|col| col.nulls().cloned()).flatten(),
        new_keys,
    }))
}

/// Extract dict values as ArrayRef and per-row keys from a dict column (any value type).
fn extract_dict_any_info<'a>(
    col: &'a ArrayRef,
    key_type: &DataType,
) -> Result<(ArrayRef, Cow<'a, [u16]>)> {
    match key_type {
        DataType::UInt8 => {
            let dict = col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                .expect("dict<u8>");
            let keys: Vec<u16> = dict.keys().values().iter().map(|k| *k as u16).collect();
            Ok((Arc::clone(dict.values()), Cow::Owned(keys)))
        }
        DataType::UInt16 => {
            let dict = col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .expect("dict<u16>");
            // Borrow directly from the key array's underlying buffer -- zero copy.
            let keys: &[u16] = dict.keys().values();
            Ok((Arc::clone(dict.values()), Cow::Borrowed(keys)))
        }
        other => Err(Error::UnsupportedDictionaryKeyType {
            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
            actual: key_type.clone(),
        }),
    }
}

/// Iterate over all values in `values_arr`, deduplicating via `value_to_idx` and appending
/// novel values to `unified_builder`. Returns a remap vector mapping each value index to its
/// unified dict index, or `None` if cardinality would exceed `max_cardinality`.
fn build_values_remap<'a>(
    values_arr: &'a ArrayRef,
    source_idx: usize,
    value_to_idx: &mut HashMap<Option<&'a [u8]>, u16>,
    unified_builder: &mut MutableArrayData<'_>,
    num_distinct: &mut usize,
    max_cardinality: usize,
) -> Result<Option<Vec<u16>>> {
    let mut remap: Vec<u16> = Vec::with_capacity(values_arr.len());
    for i in 0..values_arr.len() {
        let key = array_value_as_bytes(values_arr, i)?;
        if let Some(&idx) = value_to_idx.get(&key) {
            remap.push(idx);
            continue;
        }
        if *num_distinct >= max_cardinality {
            return Ok(None);
        }
        let idx = *num_distinct as u16;
        let _ = value_to_idx.insert(key, idx);
        unified_builder.extend(source_idx, i, i + 1);
        *num_distinct += 1;
        remap.push(idx);
    }
    Ok(Some(remap))
}

/// Extract a borrowed byte representation of an array element.
///
/// Returns `None` for null values.
///
/// Handles the types that can be dictionary-encoded in OTAP attribute value columns:
/// Utf8 (str), Int64 (int), and Binary (bytes, ser). Returns an error if any other
/// type is passed.
fn array_value_as_bytes(arr: &ArrayRef, idx: usize) -> Result<Option<&[u8]>> {
    if arr.is_null(idx) {
        return Ok(None);
    }
    match arr.data_type() {
        DataType::Utf8 => {
            let str_arr = arr
                .as_any()
                .downcast_ref::<StringArray>()
                // safety: we've checked the data type as part of match statement
                .expect("is utf8");
            Ok(Some(str_arr.value(idx).as_bytes()))
        }
        DataType::Int64 => {
            let int_arr = arr
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
                // safety: we've checked the data type as part of match statement
                .expect("is int64");
            let buf: &[u8] = int_arr.values().inner().as_slice();
            let offset = idx * size_of::<i64>();
            Ok(Some(&buf[offset..offset + size_of::<i64>()]))
        }
        DataType::Binary => {
            let bin_arr = arr
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                // safety: we've checked the data type as part of match statement
                .expect("is binary");
            Ok(Some(bin_arr.value(idx)))
        }
        other => {
            // TODO need error
            todo!("need error")
        } // other => Err(Error::ExecutionError {
          //     cause: format!("Invalid type for byte conversion {other:?}"),
          // }),
    }
}

/// Merge keys from a [`UnifiedDictMulti`] using ownership runs, producing new dictionary keys,
/// then create the new values column as a dict encoded array.
fn merge_values_with_unified_dict<T: ArrowPrimitiveType>(
    field: &Field,
    row_owners: &[OwnershipRun],
    unified: &UnifiedDictMulti,
    resolved: &[ResolvedUpsert<'_, T>],
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    let mut output_keys: Vec<u16> = Vec::with_capacity(total_output_rows);

    // Lazily initialized null bitmap — only allocated when the first inactive row is encountered.
    let mut nulls: Option<BooleanBufferBuilder> = None;

    /// Helper to mark a range of output positions as null.
    #[inline]
    fn mark_nulls(nulls: &mut Option<BooleanBufferBuilder>, current_len: usize, count: usize) {
        let builder = nulls.get_or_insert_with(|| {
            // First null encountered: allocate and backfill all prior positions as valid.
            let mut b = BooleanBufferBuilder::new(current_len + count);
            b.append_n(current_len, true);
            b
        });
        builder.append_n(count, false);
    }

    /// Helper to mark a range of output positions as valid (non-null).
    #[inline]
    fn mark_valid(nulls: &mut Option<BooleanBufferBuilder>, count: usize) {
        if let Some(builder) = nulls.as_mut() {
            builder.append_n(count, true);
        }
    }

    #[inline]
    fn mark_existing_nulls(
        nulls: &mut Option<BooleanBufferBuilder>,
        existing: &NullBuffer,
        start: usize,
        end: usize,
        current_len: usize,
        count: usize,
    ) {
        // let buff = existing.slice(start, end - start);

        let builder = nulls.get_or_insert_with(|| {
            // First null encountered: allocate and backfill all prior positions as valid.
            let mut b = BooleanBufferBuilder::new(current_len + count);
            b.append_n(current_len, true);
            b
        });
        let nulls_for_range = existing.inner().slice(start, end - start);
        builder.append_buffer(&nulls_for_range);
    }

    // Per-upsert update counters, indexed by upsert position. Inactive slots stay at 0.
    let mut update_counters: Vec<usize> = vec![0; resolved.len()];

    // Existing rows via ownership runs.
    for run in row_owners {
        let count = run.end - run.start;
        match run.owner {
            None => {
                // Passthrough: copy existing keys if we have an existing column,
                // otherwise emit nulls (new column case).
                match &unified.existing_keys {
                    Some(existing_keys) => {
                        if let Some(existing_nulls) = &unified.existing_nulls {
                            // TODO will this make things slow? We should rerun the bench
                            mark_existing_nulls(
                                &mut nulls,
                                existing_nulls,
                                run.start,
                                run.end,
                                output_keys.len(),
                                count,
                            );
                        } else {
                            mark_valid(&mut nulls, count);
                        }
                        output_keys.extend_from_slice(&existing_keys[run.start..run.end]);
                    }
                    None => {
                        mark_nulls(&mut nulls, output_keys.len(), count);
                        output_keys.extend(std::iter::repeat_n(0u16, count));
                    }
                }
            }
            Some(owner_idx) => {
                if let Some(active_keys) = &unified.new_keys[owner_idx] {
                    // Active: write new keys.
                    let counter = &mut update_counters[owner_idx];
                    match active_keys {
                        ActiveKeys::Scalar(key) => {
                            output_keys.extend(std::iter::repeat_n(*key, count));
                        }
                        ActiveKeys::Array(keys) => {
                            output_keys.extend_from_slice(&keys[*counter..*counter + count]);
                        }
                    }
                    *counter += count;
                    mark_valid(&mut nulls, count);
                } else {
                    // these rows have been updated, but the value for this column is null.
                    // put 0 in the keys column as a placeholder, and update the null buffer.
                    mark_nulls(&mut nulls, output_keys.len(), count);
                    output_keys.extend(std::iter::repeat_n(0u16, count));
                }
            }
        }
    }

    // Insert rows: iterate upserts in order.
    for (upsert_idx, r) in resolved.iter().enumerate() {
        if r.num_inserts == 0 {
            continue;
        }
        if let Some(active_keys) = &unified.new_keys[upsert_idx] {
            match active_keys {
                ActiveKeys::Scalar(key) => {
                    output_keys.extend(std::iter::repeat_n(*key, r.num_inserts));
                }
                ActiveKeys::Array(keys) => {
                    let start = r.num_updates;
                    let end = start + r.num_inserts;
                    output_keys.extend_from_slice(&keys[start..end]);
                }
            }
            mark_valid(&mut nulls, r.num_inserts);
        } else {
            // these rows have been updated, but the value for this column is null.
            // put 0 in the keys column as a placeholder, and update the null buffer.
            mark_nulls(&mut nulls, output_keys.len(), r.num_inserts);
            output_keys.extend(std::iter::repeat_n(0u16, r.num_inserts));
        }
    }

    // Build the output dict array.
    let null_buffer = nulls.map(|mut b| b.finish());

    let keys = UInt16Array::new(
        ScalarBuffer::from(output_keys),
        null_buffer.map(|b| b.into()),
    );
    let dict: ArrayRef = Arc::new(DictionaryArray::new(keys, Arc::clone(&unified.values)));
    let new_field = field
        .as_ref()
        .clone()
        .with_data_type(dict.data_type().clone())
        .with_nullable(true);

    Ok((new_field, dict))
}

/// Merge a non-target value column (passthrough for existing, null for updates/inserts).
///
/// For passthrough rows (mask=false): keep existing value.
/// For update rows (mask=true): set to null (the value for this column is no longer relevant
///   since the attribute type may have changed).
/// For insert rows: set to null.
///
/// TODO: If the column is dict-encoded, nulling out update rows may leave dictionary values
/// that are no longer referenced by any key. We should scan the remaining keys to detect
/// unreferenced dict values and either compact the dictionary or fall back to plain encoding.
/// Without this, the dictionary accumulates dead values over repeated upserts.
/// Fix this when handling https://github.com/open-telemetry/otel-arrow/issues/2313
fn merge_passthrough_column(
    field: &Field,
    existing_col: &ArrayRef,
    mask: &BooleanArray,
    num_inserts: usize,
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    // check if we even have any updates or inserts that would require nulls
    let num_updates = mask.true_count();
    if num_updates == 0 && num_inserts == 0 {
        return Ok((field.as_ref().clone(), Arc::clone(existing_col)));
    }

    let existing_data = existing_col.to_data();
    let mut mutable = MutableArrayData::new(vec![&existing_data], true, total_output_rows);

    // Iterate the mask: false ranges are passthrough (copy from existing),
    // true ranges are updates (null out).
    let mut pos = 0;
    for (start, end) in SlicesIterator::new(mask) {
        if pos < start {
            mutable.extend(0, pos, start);
        }
        mutable.extend_nulls(end - start);
        pos = end;
    }
    if pos < existing_col.len() {
        mutable.extend(0, pos, existing_col.len());
    }

    // append nulls for inserts
    if num_inserts > 0 {
        mutable.extend_nulls(num_inserts);
    }

    let result = make_array(mutable.freeze());
    let new_field = field
        .as_ref()
        .clone()
        .with_data_type(result.data_type().clone())
        .with_nullable(true);
    Ok((new_field, result))
}

/// returns an indication of whether the values column supports dictionary encoding
///
/// it is expected that the argument is a valid values column name. If it receives an invalid
/// column name, is will not return an error but the result is meaningless.
fn values_column_supports_dictionary_encoding(col_name: &str) -> bool {
    !matches!(col_name, consts::ATTRIBUTE_BOOL | consts::ATTRIBUTE_DOUBLE)
}

/// Create a new value column for a batched upsert (column didn't exist in the existing batch).
///
/// All existing rows are null. Insert rows are interleaved: active upserts contribute values,
/// inactive upserts contribute nulls.
///
/// Strategy: if dictionary encoding is supported for the column, try to build a unified
/// dictionary first. If that succeeds (cardinality fits in u16), use the dict path. If it
/// fails (overflow) or dict encoding is not supported, fall back to primitive merge.
fn create_new_value_column_batched<T: ArrowPrimitiveType>(
    target_col_name: &str,
    resolved: &[ResolvedUpsert<'_, T>],
    row_owners: &[OwnershipRun],
    total_output_rows: usize,
) -> Result<(Field, ArrayRef)> {
    let supports_dict_encoding = values_column_supports_dictionary_encoding(target_col_name);

    // Try dict encoding path first if supported.
    if supports_dict_encoding {
        if let Some(unified) = try_build_unified_dict_multi(None, resolved, target_col_name)? {
            let field = Field::new(
                target_col_name,
                DataType::Dictionary(
                    Box::new(DataType::UInt16),
                    Box::new(unified.values.data_type().clone()),
                ),
                true,
            );
            return merge_values_with_unified_dict(
                &field,
                row_owners,
                &unified,
                resolved,
                total_output_rows,
            );
        }
    }

    // Fallback: primitive merge with decoded-to-plain sources.
    let mut source_arrays: Vec<ArrayRef> = Vec::new();
    let mut active_sources: Vec<Option<ActiveSource>> = vec![None; resolved.len()];

    for (i, r) in resolved.iter().enumerate() {
        if r.target_col_name != Some(target_col_name) {
            continue;
        }
        let arr = match &r.new_values_array {
            Some(arr) => Arc::clone(arr),
            // safety: OK to expect here because when we construct resolved upsert, we set either
            // the new_values_array as Some or new_values_scalar to Some, never neither
            // TODO need error no unwrap
            None => r.new_values_scalar.expect("scalar").to_array().unwrap(),
        };
        let is_scalar = r.new_values_array.is_none();
        // Decode to plain (non-dict) for primitive merge.
        let arr = decode_to_plain(&arr)?;

        let source_idx = source_arrays.len();
        source_arrays.push(arr);
        active_sources[i] = Some(ActiveSource {
            source_idx,
            is_scalar,
            num_updates: r.num_updates,
            num_inserts: r.num_inserts,
        });
    }

    let source_data: Vec<_> = source_arrays.iter().map(|a| a.to_data()).collect();
    let source_refs: Vec<_> = source_data.iter().collect();
    let mut mutable = MutableArrayData::new(source_refs, true, total_output_rows);

    // Per-upsert update counters, indexed by upsert position. Inactive slots stay at 0.
    let mut update_counters: Vec<usize> = vec![0; resolved.len()];

    // Existing rows: null for passthrough, values for active upsert's updates, null for inactive.
    for run in row_owners {
        let count = run.end - run.start;
        match run.owner {
            None => {
                // For ranges we are not updating, insert nulls (column didn't exist before).
                mutable.extend_nulls(count);
            }
            Some(owner_idx) => {
                if let Some(active) = &active_sources[owner_idx] {
                    if active.is_scalar {
                        for _ in 0..count {
                            mutable.extend(active.source_idx, 0, 1);
                        }
                    } else {
                        let counter = &mut update_counters[owner_idx];
                        mutable.extend(active.source_idx, *counter, *counter + count);
                        *counter += count;
                    }
                } else {
                    // inactive: update must be for a different column. write nulls in this column
                    mutable.extend_nulls(count);
                }
            }
        }
    }

    // Insert rows: iterate upserts in order.
    for (upsert_idx, r) in resolved.iter().enumerate() {
        if r.num_inserts == 0 {
            continue;
        }
        if let Some(active) = &active_sources[upsert_idx] {
            if active.is_scalar {
                for _ in 0..r.num_inserts {
                    mutable.extend(active.source_idx, 0, 1);
                }
            } else {
                mutable.extend(
                    active.source_idx,
                    active.num_updates,
                    active.num_updates + active.num_inserts,
                );
            }
        } else {
            // inactive: insert must be for a different column. write nulls in this column
            mutable.extend_nulls(r.num_inserts);
        }
    }

    let result = make_array(mutable.freeze());
    let field = Field::new(target_col_name, result.data_type().clone(), true);
    Ok((field, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::UInt32Type;

    /// Helper to build a Dict(u16, Utf8) array from a slice of strings.
    fn dict_utf8_u16(values: &[&str]) -> ArrayRef {
        let plain = Arc::new(StringArray::from_iter_values(values.iter().copied())) as ArrayRef;
        let dict_type = DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8));
        cast(&plain, &dict_type).unwrap()
    }

    /// Helper to build a Dict(u8, Utf8) array from a slice of strings.
    fn dict_utf8_u8(values: &[&str]) -> ArrayRef {
        let plain = Arc::new(StringArray::from_iter_values(values.iter().copied())) as ArrayRef;
        let dict_type = DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8));
        cast(&plain, &dict_type).unwrap()
    }

    /// Helper to build a Dict(u16, Int64) array from a slice of i64.
    fn dict_int64_u16(values: &[i64]) -> ArrayRef {
        let plain = Arc::new(arrow::array::Int64Array::from_iter_values(
            values.iter().copied(),
        )) as ArrayRef;
        let dict_type = DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64));
        cast(&plain, &dict_type).unwrap()
    }

    /// Helper to call merge_value_column_batched with a single upsert for unit testing.
    fn merge_value_column_single(
        field: &Field,
        existing_col: &ArrayRef,
        mask: &BooleanArray,
        new_values: ColumnarValue,
        col_name: &str,
        target_col_name: &'static str,
        num_updates: usize,
        num_inserts: usize,
        total_output_rows: usize,
    ) -> Result<(Field, ArrayRef)> {
        let parent_ids = UInt16Array::from(vec![0u16; num_updates + num_inserts]);
        let resolved = vec![ResolvedUpsert {
            attrs_key: "test",
            mask,
            attr_value_type: AttributeValueType::Str,
            target_col_name: Some(target_col_name),
            new_values_array: match &new_values {
                ColumnarValue::Array(a) => Some(Arc::clone(a)),
                ColumnarValue::Scalar(_) => None,
            },
            new_values_scalar: match &new_values {
                ColumnarValue::Scalar(s) => Some(s),
                ColumnarValue::Array(_) => None,
            },
            upsert_parent_ids: &parent_ids,
            num_updates,
            num_inserts,
        }];
        let row_owners = build_row_owners(existing_col.len(), &resolved);
        let mut combined_mask = None;
        merge_value_column(
            field,
            existing_col,
            &resolved,
            &row_owners,
            &mut combined_mask,
            col_name,
            total_output_rows,
        )
    }

    #[test]
    fn test_merge_dict_utf8_with_scalar() {
        // Existing: Dict(u16, Utf8) ["a", "b", "a"] (3 rows, dict values ["a", "b"])
        // Mask: [false, true, false] — row 1 is an update
        // Scalar: "hello"
        // Inserts: 1
        // Expected output: ["a", "hello", "a", "hello"] as Dict(u16)
        let existing = dict_utf8_u16(&["a", "b", "a"]);
        let field = Field::new("str", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, true, false]);

        let (result_field, result_col) = merge_value_column_single(
            &field,
            &existing,
            &mask,
            ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
            consts::ATTRIBUTE_STR,
            consts::ATTRIBUTE_STR,
            1,
            1,
            4,
        )
        .unwrap();

        // Output should be dict-encoded.
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(_, _)),
            "expected dict-encoded output, got {:?}",
            result_field.data_type()
        );

        // Decode to check logical values.
        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let strs = plain.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(strs.len(), 4);
        assert_eq!(strs.value(0), "a"); // passthrough
        assert_eq!(strs.value(1), "hello"); // update
        assert_eq!(strs.value(2), "a"); // passthrough
        assert_eq!(strs.value(3), "hello"); // insert
    }

    #[test]
    fn test_merge_dict_utf8_with_dict_utf8_overlapping() {
        // Existing: Dict(u16, Utf8) ["x", "y", "x"] (dict values ["x", "y"])
        // New array: Dict(u16, Utf8) ["y", "z"] (1 update + 1 insert, dict values ["y", "z"])
        // Mask: [false, true, false] — row 1 updated
        // Expected output: ["x", "y", "x", "z"] as Dict
        let existing = dict_utf8_u16(&["x", "y", "x"]);
        let new_arr = dict_utf8_u16(&["y", "z"]);
        let field = Field::new("str", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, true, false]);

        let (result_field, result_col) = merge_value_column_single(
            &field,
            &existing,
            &mask,
            ColumnarValue::Array(new_arr),
            consts::ATTRIBUTE_STR,
            consts::ATTRIBUTE_STR,
            1,
            1,
            4,
        )
        .unwrap();

        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(_, _)),
            "expected dict-encoded output, got {:?}",
            result_field.data_type()
        );

        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let strs = plain.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(strs.len(), 4);
        assert_eq!(strs.value(0), "x"); // passthrough
        assert_eq!(strs.value(1), "y"); // update
        assert_eq!(strs.value(2), "x"); // passthrough
        assert_eq!(strs.value(3), "z"); // insert
    }

    #[test]
    fn test_merge_plain_utf8_with_plain_utf8() {
        // Existing: plain Utf8 ["a", "b", "c", "c"]
        // New: plain Utf8 ["d", "e"] (1 update + 1 insert)
        // Mask: [true, false, false, false] — row 0 updated
        // Expected output: ["d", "b", "c", "c", "e"] (dict-encoded since Utf8 is dict-eligible)
        let existing = Arc::new(StringArray::from_iter_values(["a", "b", "c", "c"])) as ArrayRef;
        let new_arr = Arc::new(StringArray::from_iter_values(["d", "e"])) as ArrayRef;
        let field = Field::new("str", DataType::Utf8, true);
        let mask = BooleanArray::from(vec![true, false, false]);

        let (result_field, result_col) = merge_value_column_single(
            &field,
            &existing,
            &mask,
            ColumnarValue::Array(new_arr),
            consts::ATTRIBUTE_STR,
            consts::ATTRIBUTE_STR,
            1,
            1,
            5,
        )
        .unwrap();

        // Output gets dict-encoded as Dict(u16) per the OTAP spec since Utf8 is dict-eligible.
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(_, _)),
            "expected dict-encoded output, got {:?}",
            result_field.data_type()
        );

        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let strs = plain.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(strs.len(), 5);
        assert_eq!(strs.value(0), "d"); // update
        assert_eq!(strs.value(1), "b"); // passthrough
        assert_eq!(strs.value(2), "c"); // passthrough
        assert_eq!(strs.value(3), "c"); // passthrough
        assert_eq!(strs.value(4), "e"); // insert
    }

    #[test]
    fn test_merge_dict_int64_with_scalar() {
        // Existing: Dict(u16, Int64) [10, 20, 10] (dict values [10, 20])
        // Mask: [false, true, false] — row 1 updated
        // Scalar: 42i64
        // Inserts: 1
        // Expected output: [10, 42, 10, 42] as Dict(u16)
        let existing = dict_int64_u16(&[10, 20, 10]);
        let field = Field::new("int", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, true, false]);

        let (result_field, result_col) = merge_value_column_single(
            &field,
            &existing,
            &mask,
            ColumnarValue::Scalar(ScalarValue::Int64(Some(42))),
            consts::ATTRIBUTE_INT,
            consts::ATTRIBUTE_INT,
            1,
            1,
            4,
        )
        .unwrap();

        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(_, _)),
            "expected dict-encoded output, got {:?}",
            result_field.data_type()
        );

        let plain = cast(&result_col, &DataType::Int64).unwrap();
        let ints = plain
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap();
        assert_eq!(ints.len(), 4);
        assert_eq!(ints.value(0), 10); // passthrough
        assert_eq!(ints.value(1), 42); // update
        assert_eq!(ints.value(2), 10); // passthrough
        assert_eq!(ints.value(3), 42); // insert
    }

    // ---- merge_key_column_batched tests ----

    /// Helper to build a minimal ResolvedUpsert for key column tests.
    fn key_test_resolved<'a>(
        attrs_key: &'a str,
        mask: &'a BooleanArray,
        parent_ids: &'a UInt16Array,
        num_updates: usize,
        num_inserts: usize,
    ) -> ResolvedUpsert<'a, UInt16Type> {
        ResolvedUpsert {
            attrs_key,
            mask,
            attr_value_type: AttributeValueType::Str,
            target_col_name: Some(consts::ATTRIBUTE_STR),
            new_values_array: None,
            new_values_scalar: None,
            upsert_parent_ids: parent_ids,
            num_updates,
            num_inserts,
        }
    }

    #[test]
    fn test_merge_key_column_dict_existing_key() {
        // Existing: Dict(u16, Utf8) ["x", "y", "x"] (dict values ["x", "y"])
        // Insert key "x" (already in dict) — 2 inserts
        // Expected: 5 rows ["x", "y", "x", "x", "x"], dict values unchanged ["x", "y"]
        let existing = dict_utf8_u16(&["x", "y", "x"]);
        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, false, false]);
        let parent_ids = UInt16Array::from(vec![0u16, 1]);
        let resolved = vec![key_test_resolved("x", &mask, &parent_ids, 0, 2)];

        let (result_field, result_col) = merge_key_column(&field, &existing, &resolved, 5).unwrap();

        assert!(matches!(
            result_field.data_type(),
            DataType::Dictionary(_, _)
        ));
        // Decode to check per-row values regardless of key type.
        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let str_arr = plain
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray after cast");
        assert_eq!(str_arr.len(), 5);
        assert_eq!(str_arr.value(0), "x");
        assert_eq!(str_arr.value(1), "y");
        assert_eq!(str_arr.value(2), "x");
        assert_eq!(str_arr.value(3), "x"); // insert
        assert_eq!(str_arr.value(4), "x"); // insert
    }

    #[test]
    fn test_merge_key_column_dict_novel_key() {
        // Existing: Dict(u16, Utf8) ["x", "y", "x"] (dict values ["x", "y"])
        // Insert key "z" (novel) — 1 insert
        // Expected: 4 rows ["x", "y", "x", "z"], dict values ["x", "y", "z"]
        let existing = dict_utf8_u16(&["x", "y", "x"]);
        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, false, false]);
        let parent_ids = UInt16Array::from(vec![0u16]);
        let resolved = vec![key_test_resolved("z", &mask, &parent_ids, 0, 1)];

        let (result_field, result_col) = merge_key_column(&field, &existing, &resolved, 4).unwrap();

        assert!(matches!(
            result_field.data_type(),
            DataType::Dictionary(_, _)
        ));
        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let str_arr = plain
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray after cast");
        assert_eq!(str_arr.len(), 4);
        assert_eq!(str_arr.value(0), "x");
        assert_eq!(str_arr.value(1), "y");
        assert_eq!(str_arr.value(2), "x");
        assert_eq!(str_arr.value(3), "z"); // insert
    }

    #[test]
    fn test_merge_key_column_no_inserts() {
        // Existing: Dict(u16, Utf8) ["x", "y", "x"]
        // No inserts — column should be returned unchanged.
        let existing = dict_utf8_u16(&["x", "y", "x"]);
        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, false, false]);
        let parent_ids = UInt16Array::from(Vec::<u16>::new());
        let resolved = vec![key_test_resolved("z", &mask, &parent_ids, 0, 0)];

        let (result_field, result_col) = merge_key_column(&field, &existing, &resolved, 3).unwrap();

        assert_eq!(result_field.data_type(), existing.data_type());
        assert_eq!(result_col.len(), 3);
    }

    #[test]
    fn test_merge_key_column_plain_utf8() {
        // Existing: plain Utf8 ["x", "y", "x"]
        // Insert key "z" — 2 inserts
        // Expected: 5 rows ["x", "y", "x", "z", "z"]
        let existing = Arc::new(StringArray::from_iter_values(["x", "y", "x"])) as ArrayRef;
        let field = Field::new("key", DataType::Utf8, true);
        let mask = BooleanArray::from(vec![false, false, false]);
        let parent_ids = UInt16Array::from(vec![0u16, 1]);
        let resolved = vec![key_test_resolved("z", &mask, &parent_ids, 0, 2)];

        let (_, result_col) = merge_key_column(&field, &existing, &resolved, 5).unwrap();

        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let str_arr = plain
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray after cast");

        assert_eq!(str_arr.len(), 5);
        assert_eq!(str_arr.value(0), "x");
        assert_eq!(str_arr.value(1), "y");
        assert_eq!(str_arr.value(2), "x");
        assert_eq!(str_arr.value(3), "z"); // insert
        assert_eq!(str_arr.value(4), "z"); // insert
    }

    #[test]
    fn test_merge_key_column_dict_u8_stays_u8() {
        // Existing: Dict(u8, Utf8) ["x", "y", "x"] (dict values ["x", "y"], cardinality=2)
        // Insert key "z" (novel) — 1 insert
        // Final cardinality = 3, which fits in u8
        // Expected: Dict(u8, Utf8) with 4 rows ["x", "y", "x", "z"]
        let existing = dict_utf8_u8(&["x", "y", "x"]);
        assert!(
            matches!(existing.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "precondition: existing should be Dict(u8, Utf8)"
        );

        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, false, false]);
        let parent_ids = UInt16Array::from(vec![0u16]);
        let resolved = vec![key_test_resolved("z", &mask, &parent_ids, 0, 1)];

        let (result_field, result_col) = merge_key_column(&field, &existing, &resolved, 4).unwrap();

        // Should stay as Dict(u8, Utf8) since cardinality (3) fits in u8.
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "expected Dict(u8, Utf8) output, got {:?}",
            result_field.data_type()
        );

        // Verify logical values.
        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let str_arr = plain
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray after cast");
        assert_eq!(str_arr.len(), 4);
        assert_eq!(str_arr.value(0), "x");
        assert_eq!(str_arr.value(1), "y");
        assert_eq!(str_arr.value(2), "x");
        assert_eq!(str_arr.value(3), "z"); // insert
    }

    #[test]
    fn test_merge_key_column_dict_u8_overflows_to_u16() {
        // Existing: Dict(u8, Utf8) with 255 distinct keys (max for u8)
        // Insert a novel key → cardinality becomes 256, which exceeds u8
        // Expected: Output widens to Dict(u16, Utf8)

        // Build 255 distinct keys: "k000", "k001", ..., "k254"
        let distinct_keys: Vec<String> = (0..255).map(|i| format!("k{i:03}")).collect();
        let distinct_refs: Vec<&str> = distinct_keys.iter().map(|s| s.as_str()).collect();

        // Create a column with these 255 distinct values (one row each)
        let existing = dict_utf8_u8(&distinct_refs);
        assert!(
            matches!(existing.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "precondition: existing should be Dict(u8, Utf8)"
        );
        assert_eq!(existing.len(), 255);

        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false; 255]);
        let parent_ids = UInt16Array::from(vec![0u16]);
        // Insert a novel key "overflow" which will be the 256th distinct value
        let resolved = vec![key_test_resolved("overflow", &mask, &parent_ids, 0, 1)];

        let (result_field, result_col) =
            merge_key_column(&field, &existing, &resolved, 256).unwrap();

        // Should widen to Dict(u16, Utf8) since cardinality (256) exceeds u8 max (255).
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt16),
            "expected Dict(u16, Utf8) output after overflow, got {:?}",
            result_field.data_type()
        );

        // Verify we have 256 rows.
        assert_eq!(result_col.len(), 256);

        // Verify first and last values.
        let plain = cast(&result_col, &DataType::Utf8).unwrap();
        let str_arr = plain
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray after cast");
        assert_eq!(str_arr.value(0), "k000"); // first existing
        assert_eq!(str_arr.value(254), "k254"); // last existing
        assert_eq!(str_arr.value(255), "overflow"); // inserted novel key
    }

    #[test]
    fn test_merge_key_column_dict_u16_overflows_to_plain_utf8() {
        // Existing: Dict(u16, Utf8) with 65535 distinct keys (max for u16)
        // Insert a novel key → cardinality becomes 65536, which exceeds u16
        // Expected: Output falls back to plain Utf8

        // Build 65535 distinct keys: "k00000", "k00001", ..., "k65534"
        let distinct_keys: Vec<String> = (0..65535u32).map(|i| format!("k{i:05}")).collect();
        let distinct_refs: Vec<&str> = distinct_keys.iter().map(|s| s.as_str()).collect();

        let existing = dict_utf8_u16(&distinct_refs);
        assert!(
            matches!(existing.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt16),
            "precondition: existing should be Dict(u16, Utf8)"
        );
        assert_eq!(existing.len(), 65535);

        let field = Field::new("key", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false; 65535]);
        let parent_ids = UInt16Array::from(vec![0u16]);
        // Insert a novel key "overflow" which will be the 65536th distinct value.
        let resolved = vec![key_test_resolved("overflow", &mask, &parent_ids, 0, 1)];

        let (result_field, result_col) =
            merge_key_column(&field, &existing, &resolved, 65536).unwrap();

        // Should fall back to plain Utf8 since cardinality (65536) exceeds u16 max (65535).
        assert_eq!(
            result_field.data_type(),
            &DataType::Utf8,
            "expected plain Utf8 output after overflow, got {:?}",
            result_field.data_type()
        );

        // Verify we have 65536 rows.
        assert_eq!(result_col.len(), 65536);

        // Verify first, last existing, and inserted values.
        let str_arr = result_col
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("expected StringArray (plain Utf8)");
        assert_eq!(str_arr.value(0), "k00000"); // first existing
        assert_eq!(str_arr.value(65534), "k65534"); // last existing
        assert_eq!(str_arr.value(65535), "overflow"); // inserted novel key
    }

    // ---- merge_type_column_batched tests ----

    #[test]
    fn test_merge_type_column_updates_and_inserts() {
        // Existing type column: [Str=1, Int=2, Str=1]
        // Mask: [false, true, false] — row 1 is an update
        // New type: Str (1)
        // Inserts: 1
        // Expected: [1, 1, 1, 1] — row 1 updated from Int to Str, insert gets Str
        let existing = Arc::new(UInt8Array::from(vec![1u8, 2, 1])) as ArrayRef;
        let field = Field::new("type", DataType::UInt8, false);
        let mask = BooleanArray::from(vec![false, true, false]);
        let parent_ids = UInt16Array::from(vec![0u16, 1]);

        let scalar = ScalarValue::Utf8(Some("x".into()));
        let resolved = vec![ResolvedUpsert {
            attrs_key: "test",
            mask: &mask,
            attr_value_type: AttributeValueType::Str,
            target_col_name: Some(consts::ATTRIBUTE_STR),
            new_values_array: None,
            new_values_scalar: Some(&scalar),
            upsert_parent_ids: &parent_ids,
            num_updates: 1,
            num_inserts: 1,
        }];
        let row_owners = build_row_owners(3, &resolved);

        let (_, result_col) = merge_type_column(
            &field,
            &existing,
            &resolved,
            &row_owners,
            4, // total_output_rows
        )
        .unwrap();

        let arr = result_col
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("expected UInt8Array");
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.value(0), 1); // passthrough (Str)
        assert_eq!(arr.value(1), 1); // update: was Int(2), now Str(1)
        assert_eq!(arr.value(2), 1); // passthrough (Str)
        assert_eq!(arr.value(3), 1); // insert (Str)
    }

    #[test]
    fn test_merge_type_column_inserts_only() {
        // Existing type column: [Str=1, Str=1]
        // Mask: [false, false] — no updates
        // New type: Int (2)
        // Inserts: 2
        // Expected: [1, 1, 2, 2] — existing unchanged, inserts get Int
        let existing = Arc::new(UInt8Array::from(vec![1u8, 1])) as ArrayRef;
        let field = Field::new("type", DataType::UInt8, false);
        let mask = BooleanArray::from(vec![false, false]);
        let parent_ids = UInt16Array::from(vec![0u16, 1]);

        let scalar = ScalarValue::Int64(Some(42));
        let resolved = vec![ResolvedUpsert {
            attrs_key: "test",
            mask: &mask,
            attr_value_type: AttributeValueType::Int,
            target_col_name: Some(consts::ATTRIBUTE_INT),
            new_values_array: None,
            new_values_scalar: Some(&scalar),
            upsert_parent_ids: &parent_ids,
            num_updates: 0,
            num_inserts: 2,
        }];
        let row_owners = build_row_owners(2, &resolved);

        let (_, result_col) = merge_type_column(
            &field,
            &existing,
            &resolved,
            &row_owners,
            4, // total_output_rows
        )
        .unwrap();

        let arr = result_col
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("expected UInt8Array");
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.value(0), 1); // passthrough (Str)
        assert_eq!(arr.value(1), 1); // passthrough (Str)
        assert_eq!(arr.value(2), 2); // insert (Int)
        assert_eq!(arr.value(3), 2); // insert (Int)
    }

    #[test]
    fn test_merge_type_column_all_updates() {
        // Existing type column: [Str=1, Int=2, Double=3]
        // Mask: [true, true, true] — all rows updated
        // New type: Bool (4)
        // Inserts: 0
        // Expected: [4, 4, 4] — all replaced
        let existing = Arc::new(UInt8Array::from(vec![1u8, 2, 3])) as ArrayRef;
        let field = Field::new("type", DataType::UInt8, false);
        let mask = BooleanArray::from(vec![true, true, true]);
        let parent_ids = UInt16Array::from(vec![0u16, 1, 2]);

        let scalar = ScalarValue::Boolean(Some(true));
        let resolved = vec![ResolvedUpsert {
            attrs_key: "test",
            mask: &mask,
            attr_value_type: AttributeValueType::Bool,
            target_col_name: Some(consts::ATTRIBUTE_BOOL),
            new_values_array: None,
            new_values_scalar: Some(&scalar),
            upsert_parent_ids: &parent_ids,
            num_updates: 3,
            num_inserts: 0,
        }];
        let row_owners = build_row_owners(3, &resolved);

        let (_, result_col) = merge_type_column(
            &field,
            &existing,
            &resolved,
            &row_owners,
            3, // total_output_rows
        )
        .unwrap();

        let arr = result_col
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("expected UInt8Array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(0), 4); // update: was Str(1), now Bool(4)
        assert_eq!(arr.value(1), 4); // update: was Int(2), now Bool(4)
        assert_eq!(arr.value(2), 4); // update: was Double(3), now Bool(4)
    }

    // ---- merge_passthrough_column tests ----

    #[test]
    fn test_merge_passthrough_column_updates_and_inserts() {
        // Existing: Dict(u16, Int64) [10, 20, 30]
        // Mask: [false, true, false] — row 1 is an update
        // Inserts: 1
        // Expected: 4 rows [10, null, 30, null] — update and insert rows nulled out
        let existing = dict_int64_u16(&[10, 20, 30]);
        let field = Field::new("int", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, true, false]);

        let (result_field, result_col) =
            merge_passthrough_column(&field, &existing, &mask, 1, 4).unwrap();

        assert!(result_field.is_nullable());
        assert_eq!(result_col.len(), 4);
        assert_eq!(result_col.null_count(), 2);

        // Decode to plain Int64 to check values including nulls.
        let plain = cast(&result_col, &DataType::Int64).unwrap();
        let arr = plain
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap();
        assert!(!arr.is_null(0));
        assert_eq!(arr.value(0), 10); // passthrough
        assert!(arr.is_null(1)); // update — nulled
        assert!(!arr.is_null(2));
        assert_eq!(arr.value(2), 30); // passthrough
        assert!(arr.is_null(3)); // insert — nulled
    }

    #[test]
    fn test_merge_passthrough_column_no_changes() {
        // Existing: Dict(u16, Utf8) ["a", "b", "c"]
        // Mask: [false, false, false] — no updates
        // Inserts: 0
        // Expected: column returned unchanged (early-out path)
        let existing = dict_utf8_u16(&["a", "b", "c"]);
        let field = Field::new("str", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![false, false, false]);

        let (_, result_col) = merge_passthrough_column(&field, &existing, &mask, 0, 3).unwrap();

        assert_eq!(result_col.len(), 3);
        assert_eq!(result_col.null_count(), 0);

        // Should be the exact same array (early-out returns Arc::clone).
        let dict = result_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .expect("expected Dict(u16) output");
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(values.value(dict.keys().value(0) as usize), "a");
        assert_eq!(values.value(dict.keys().value(1) as usize), "b");
        assert_eq!(values.value(dict.keys().value(2) as usize), "c");
    }

    #[test]
    fn test_merge_passthrough_column_all_updates() {
        // Existing: Dict(u16, Utf8) ["a", "b", "c"]
        // Mask: [true, true, true] — all rows updated
        // Inserts: 0
        // Expected: 3 rows [null, null, null] — all nulled out
        let existing = dict_utf8_u16(&["a", "b", "c"]);
        let field = Field::new("str", existing.data_type().clone(), true);
        let mask = BooleanArray::from(vec![true, true, true]);

        let (result_field, result_col) =
            merge_passthrough_column(&field, &existing, &mask, 0, 3).unwrap();

        assert!(result_field.is_nullable());
        assert_eq!(result_col.len(), 3);
        assert_eq!(result_col.null_count(), 3);

        for i in 0..3 {
            assert!(result_col.is_null(i), "row {i} should be null");
        }
    }

    // ---- upsert_attributes tests ----

    // TODO: If all rows in a passthrough value column have been replaced with nulls (e.g.,
    // when every existing row is updated to a different type), the column should be removed
    // from the output batch entirely rather than kept as an all-null column.

    /// Build a minimal attrs RecordBatch with columns [parent_id, key, type, str].
    /// `rows` is a slice of (parent_id, key, type_discriminant, str_value).
    fn build_attrs_batch(rows: &[(u16, &str, u8, &str)]) -> RecordBatch {
        let parent_ids = UInt16Array::from_iter_values(rows.iter().map(|(pid, _, _, _)| *pid));
        let keys = dict_utf8_u16(&rows.iter().map(|(_, k, _, _)| *k).collect::<Vec<_>>());
        let types = UInt8Array::from_iter_values(rows.iter().map(|(_, _, t, _)| *t));
        let strs = dict_utf8_u16(&rows.iter().map(|(_, _, _, s)| *s).collect::<Vec<_>>());

        let schema = Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(consts::ATTRIBUTE_KEY, keys.data_type().clone(), true),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, strs.data_type().clone(), true),
        ]);

        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(parent_ids) as ArrayRef,
                keys,
                Arc::new(types) as ArrayRef,
                strs,
            ],
        )
        .unwrap()
    }

    /// Helper to decode a column to plain Utf8, handling both dict-encoded and plain.
    fn decode_to_utf8(col: &ArrayRef) -> StringArray {
        let plain = cast(col, &DataType::Utf8).unwrap();
        plain
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .clone()
    }

    #[test]
    fn test_upsert_attributes_basic_update_and_insert() {
        // Existing attrs batch:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=0, key="y", type=Str(1), str="b"
        //   Row 2: parent_id=1, key="x", type=Str(1), str="c"
        //
        // Upsert: attributes["y"] = "hello"
        //   Parent 0 has key "y" (row 1) → update
        //   Parent 1 does not have key "y" → insert
        //
        // Expected output (4 rows):
        //   Row 0: parent_id=0, key="x", type=1, str="a"   (passthrough)
        //   Row 1: parent_id=0, key="y", type=1, str="hello" (update)
        //   Row 2: parent_id=1, key="x", type=1, str="c"   (passthrough)
        //   Row 3: parent_id=1, key="y", type=1, str="hello" (insert)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (0, "y", 1, "b"), (1, "x", 1, "c")]);
        let mask = BooleanArray::from(vec![false, true, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16, 1]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into())));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "y",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 4);

        // Check parent_id column.
        let parent_ids = result
            .column_by_name(consts::PARENT_ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        assert_eq!(parent_ids.value(0), 0);
        assert_eq!(parent_ids.value(1), 0);
        assert_eq!(parent_ids.value(2), 1);
        assert_eq!(parent_ids.value(3), 1);

        // Check key column.
        let keys = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_KEY).unwrap());
        assert_eq!(keys.value(0), "x");
        assert_eq!(keys.value(1), "y");
        assert_eq!(keys.value(2), "x");
        assert_eq!(keys.value(3), "y");

        // Check type column.
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1); // Str
        assert_eq!(types.value(1), 1); // Str
        assert_eq!(types.value(2), 1); // Str
        assert_eq!(types.value(3), 1); // Str

        // Check str column.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "a"); // passthrough
        assert_eq!(strs.value(1), "hello"); // update
        assert_eq!(strs.value(2), "c"); // passthrough
        assert_eq!(strs.value(3), "hello"); // insert
    }

    #[test]
    fn test_upsert_redacts_dead_dict_values() {
        // When a value is updated, the old value should NOT persist in the output dict's
        // values array. This is important for data redaction: if someone upserts to replace
        // sensitive data, the old value must not leak through the dictionary.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="secret"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="public"
        //
        // Upsert: attributes["x"] = "REDACTED" for parent 0 only
        //   Row 0 matches → update (replaces "secret" with "REDACTED")
        //   Row 1 does not match → passthrough (keeps "public")
        //   Parent 2 doesn't have key "x" → insert ("REDACTED")
        //
        // Expected: the output str column's dictionary values should contain "public" and
        // "REDACTED" but NOT "secret".
        let existing = build_attrs_batch(&[(0, "x", 1, "secret"), (1, "x", 1, "public")]);
        let mask = BooleanArray::from(vec![true, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16, 2]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Utf8(Some("REDACTED".into())));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "x",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 3);

        // Check logical values are correct.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "REDACTED"); // update
        assert_eq!(strs.value(1), "public"); // passthrough
        assert_eq!(strs.value(2), "REDACTED"); // insert

        // The critical check: inspect the dictionary values array to ensure "secret" is NOT
        // present. If the column is dict-encoded, the values array should only contain values
        // that are actually referenced by live keys.
        let str_col = result.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        if let DataType::Dictionary(_, _) = str_col.data_type() {
            let dict = str_col
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .expect("expected Dict(u16, Utf8)");
            let values = dict
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("expected Utf8 values");
            let dict_values: Vec<&str> = (0..values.len()).map(|i| values.value(i)).collect();
            assert!(
                !dict_values.contains(&"secret"),
                "dead value 'secret' should not appear in dict values, but found: {dict_values:?}"
            );
        }
    }

    #[test]
    fn test_upsert_attributes_target_column_does_not_exist() {
        // Existing attrs batch has only a "str" value column.
        // Upsert assigns an Int64 value → "int" column doesn't exist yet and must be created.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upsert: attributes["z"] = 42i64
        //   Neither parent has key "z" → both are inserts.
        //
        // Expected output (4 rows):
        //   Row 0: parent_id=0, key="x", type=1, str="a",  int=null  (passthrough)
        //   Row 1: parent_id=1, key="x", type=1, str="b",  int=null  (passthrough)
        //   Row 2: parent_id=0, key="z", type=2, str=null,  int=42   (insert)
        //   Row 3: parent_id=1, key="z", type=2, str=null,  int=42   (insert)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);
        let mask = BooleanArray::from(vec![false, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16, 1]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Int64(Some(42)));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "z",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 4);

        // The "int" column should now exist in the output schema.
        let int_col = result.column_by_name(consts::ATTRIBUTE_INT);
        assert!(int_col.is_some(), "expected 'int' column to be created");

        // Check type column: existing rows keep Str(1), inserts get Int(2).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1); // Str (passthrough)
        assert_eq!(types.value(1), 1); // Str (passthrough)
        assert_eq!(types.value(2), 2); // Int (insert)
        assert_eq!(types.value(3), 2); // Int (insert)

        // Check str column: passthrough for existing, null for inserts.
        let str_col = result.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let strs = decode_to_utf8(str_col);
        assert_eq!(strs.value(0), "a");
        assert_eq!(strs.value(1), "b");
        assert!(str_col.is_null(2));
        assert!(str_col.is_null(3));

        // Check int column: must be Dict(u16, Int64) per OTAP spec.
        // TODO: Add tests for dict overflow in the new column case (cardinality > 65535).
        let int_col = int_col.unwrap();
        let expected_type =
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64));
        assert_eq!(
            int_col.data_type(),
            &expected_type,
            "expected Dict(u16, Int64) for new int column, got {:?}",
            int_col.data_type()
        );

        // Decode to plain to check values including nulls.
        let ints = cast(int_col, &DataType::Int64).unwrap();
        let ints = ints
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap();
        assert!(ints.is_null(0));
        assert!(ints.is_null(1));
        assert_eq!(ints.value(2), 42);
        assert_eq!(ints.value(3), 42);

        // Check key column: existing keep their keys, inserts get "z".
        let keys = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_KEY).unwrap());
        assert_eq!(keys.value(0), "x");
        assert_eq!(keys.value(1), "x");
        assert_eq!(keys.value(2), "z");
        assert_eq!(keys.value(3), "z");
    }

    #[test]
    fn test_upsert_attributes_null_scalar() {
        // Upsert with a null scalar — the attribute type becomes Empty(0) and no target
        // value column is written. All value columns become passthrough (nulled for updates).
        //
        // Existing:
        //   Row 0: parent_id=0, key="y", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upsert: attributes["y"] = null
        //   Row 0 matches key "y" → update
        //   No inserts (parent 1 already doesn't have "y", and we only upsert parent 0)
        //
        // Expected output (2 rows):
        //   Row 0: parent_id=0, key="y", type=0(Empty), str=null  (update — type cleared)
        //   Row 1: parent_id=1, key="x", type=1(Str),   str="b"  (passthrough)
        let existing = build_attrs_batch(&[(0, "y", 1, "a"), (1, "x", 1, "b")]);
        let mask = BooleanArray::from(vec![true, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Null);

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "y",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 2);

        // Check type column: row 0 updated to Empty(0), row 1 passthrough Str(1).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 0); // Empty
        assert_eq!(types.value(1), 1); // Str (passthrough)

        // Check str column: row 0 nulled (passthrough nulls update rows), row 1 kept.
        let str_col = result.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        assert!(str_col.is_null(0)); // nulled by passthrough
        let strs = decode_to_utf8(str_col);
        assert_eq!(strs.value(1), "b"); // passthrough kept
    }

    #[test]
    fn test_upsert_attributes_inserts_only() {
        // All upserts are inserts — no existing rows match the target key.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upsert: attributes["z"] = "new"
        //   Neither row matches key "z" → both parent 2 and 3 are inserts.
        //
        // Expected output (4 rows):
        //   Row 0: parent_id=0, key="x", type=1, str="a"   (passthrough)
        //   Row 1: parent_id=1, key="x", type=1, str="b"   (passthrough)
        //   Row 2: parent_id=2, key="z", type=1, str="new" (insert)
        //   Row 3: parent_id=3, key="z", type=1, str="new" (insert)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);
        let mask = BooleanArray::from(vec![false, false]);
        let upsert_parent_ids = UInt16Array::from(vec![2u16, 3]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Utf8(Some("new".into())));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "z",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 4);

        // Check parent_id column.
        let parent_ids = result
            .column_by_name(consts::PARENT_ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        assert_eq!(parent_ids.value(0), 0);
        assert_eq!(parent_ids.value(1), 1);
        assert_eq!(parent_ids.value(2), 2);
        assert_eq!(parent_ids.value(3), 3);

        // Check key column.
        let keys = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_KEY).unwrap());
        assert_eq!(keys.value(0), "x");
        assert_eq!(keys.value(1), "x");
        assert_eq!(keys.value(2), "z");
        assert_eq!(keys.value(3), "z");

        // Check type column: all Str(1).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1);
        assert_eq!(types.value(1), 1);
        assert_eq!(types.value(2), 1);
        assert_eq!(types.value(3), 1);

        // Check str column.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "a"); // passthrough
        assert_eq!(strs.value(1), "b"); // passthrough
        assert_eq!(strs.value(2), "new"); // insert
        assert_eq!(strs.value(3), "new"); // insert
    }

    #[test]
    fn test_upsert_attributes_new_double_column_not_dict_encoded() {
        // Per OTAP spec section 5.4.2, Float64 (`double`) has no optimized encoding —
        // it must remain plain Float64, never dict-encoded.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upsert: attributes["temp"] = 3.00f64
        //   Neither parent has key "temp" → both are inserts.
        //
        // Expected output (4 rows):
        //   Row 0: parent_id=0, key="x", type=1(Str),    str="a",  double=null  (passthrough)
        //   Row 1: parent_id=1, key="x", type=1(Str),    str="b",  double=null  (passthrough)
        //   Row 2: parent_id=0, key="temp", type=3(Double), str=null, double=3.14  (insert)
        //   Row 3: parent_id=1, key="temp", type=3(Double), str=null, double=3.14  (insert)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);
        let mask = BooleanArray::from(vec![false, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16, 1]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Float64(Some(3.00)));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "temp",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 4);

        // The "double" column must be plain Float64 — NOT dict-encoded.
        let double_col = result.column_by_name(consts::ATTRIBUTE_DOUBLE).unwrap();
        assert_eq!(
            double_col.data_type(),
            &DataType::Float64,
            "expected plain Float64 for double column per OTAP spec, got {:?}",
            double_col.data_type()
        );

        let doubles = double_col
            .as_any()
            .downcast_ref::<arrow::array::Float64Array>()
            .unwrap();
        assert!(doubles.is_null(0)); // passthrough
        assert!(doubles.is_null(1)); // passthrough
        assert!((doubles.value(2) - 3.00).abs() < f64::EPSILON); // insert
        assert!((doubles.value(3) - 3.00).abs() < f64::EPSILON); // insert

        // Check type column: existing keep Str(1), inserts get Double(3).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1); // Str
        assert_eq!(types.value(1), 1); // Str
        assert_eq!(types.value(2), 3); // Double
        assert_eq!(types.value(3), 3); // Double
    }

    #[test]
    fn test_upsert_attributes_new_bool_column_not_dict_encoded() {
        // Per OTAP spec section 5.4.2, Boolean (`bool`) has no optimized encoding —
        // it must remain plain Boolean, never dict-encoded.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upsert: attributes["flag"] = true
        //   Neither parent has key "flag" → both are inserts.
        //
        // Expected output (4 rows):
        //   Row 0: parent_id=0, key="x",    type=1(Str),  str="a", bool=null  (passthrough)
        //   Row 1: parent_id=1, key="x",    type=1(Str),  str="b", bool=null  (passthrough)
        //   Row 2: parent_id=0, key="flag", type=4(Bool), str=null, bool=true  (insert)
        //   Row 3: parent_id=1, key="flag", type=4(Bool), str=null, bool=true  (insert)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);
        let mask = BooleanArray::from(vec![false, false]);
        let upsert_parent_ids = UInt16Array::from(vec![0u16, 1]);
        let new_values = ColumnarValue::Scalar(ScalarValue::Boolean(Some(true)));

        let result = upsert_attributes(
            &existing,
            &[AttributeUpsert {
                attrs_key: "flag",
                existing_key_mask: mask,
                new_values,
                upsert_parent_ids,
            }],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 4);

        // The "bool" column must be plain Boolean — NOT dict-encoded.
        let bool_col = result.column_by_name(consts::ATTRIBUTE_BOOL).unwrap();
        assert_eq!(
            bool_col.data_type(),
            &DataType::Boolean,
            "expected plain Boolean for bool column per OTAP spec, got {:?}",
            bool_col.data_type()
        );

        let bools = bool_col.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(bool_col.is_null(0)); // passthrough
        assert!(bool_col.is_null(1)); // passthrough
        assert!(bools.value(2)); // insert: true
        assert!(bools.value(3)); // insert: true

        // Check type column: existing keep Str(1), inserts get Bool(4).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1); // Str
        assert_eq!(types.value(1), 1); // Str
        assert_eq!(types.value(2), 4); // Bool
        assert_eq!(types.value(3), 4); // Bool
    }

    // ---- batched upsert tests ----

    #[test]
    fn test_upsert_attributes_batched_two_new_str_keys() {
        // Test batched upsert of two new string keys in a single call.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upserts:
        //   attributes["y"] = "hello"  (neither parent has "y" → 2 inserts)
        //   attributes["z"] = "world"  (neither parent has "z" → 2 inserts)
        //
        // Expected output (6 rows):
        //   Row 0: parent_id=0, key="x", type=1, str="a"      (passthrough)
        //   Row 1: parent_id=1, key="x", type=1, str="b"      (passthrough)
        //   Row 2: parent_id=0, key="y", type=1, str="hello"  (insert from upsert 0)
        //   Row 3: parent_id=1, key="y", type=1, str="hello"  (insert from upsert 0)
        //   Row 4: parent_id=0, key="z", type=1, str="world"  (insert from upsert 1)
        //   Row 5: parent_id=1, key="z", type=1, str="world"  (insert from upsert 1)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);

        let mask_y = BooleanArray::from(vec![false, false]);
        let mask_z = BooleanArray::from(vec![false, false]);
        let parent_ids_y = UInt16Array::from(vec![0u16, 1]);
        let parent_ids_z = UInt16Array::from(vec![0u16, 1]);

        let result = upsert_attributes(
            &existing,
            &[
                AttributeUpsert {
                    attrs_key: "y",
                    existing_key_mask: mask_y,
                    new_values: ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
                    upsert_parent_ids: parent_ids_y,
                },
                AttributeUpsert {
                    attrs_key: "z",
                    existing_key_mask: mask_z,
                    new_values: ColumnarValue::Scalar(ScalarValue::Utf8(Some("world".into()))),
                    upsert_parent_ids: parent_ids_z,
                },
            ],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 6);

        // Check parent_id.
        let pids = result
            .column_by_name(consts::PARENT_ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        assert_eq!(pids.values(), &[0, 1, 0, 1, 0, 1]);

        // Check key column.
        let keys = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_KEY).unwrap());
        assert_eq!(keys.value(0), "x");
        assert_eq!(keys.value(1), "x");
        assert_eq!(keys.value(2), "y");
        assert_eq!(keys.value(3), "y");
        assert_eq!(keys.value(4), "z");
        assert_eq!(keys.value(5), "z");

        // Check type column: all Str(1).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        for i in 0..6 {
            assert_eq!(types.value(i), 1, "type at row {i} should be Str(1)");
        }

        // Check str column.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "a"); // passthrough
        assert_eq!(strs.value(1), "b"); // passthrough
        assert_eq!(strs.value(2), "hello"); // insert y
        assert_eq!(strs.value(3), "hello"); // insert y
        assert_eq!(strs.value(4), "world"); // insert z
        assert_eq!(strs.value(5), "world"); // insert z
    }

    #[test]
    fn test_upsert_attributes_batched_different_types() {
        // Test batched upsert where each upsert targets a different value column.
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=1, key="x", type=Str(1), str="b"
        //
        // Upserts:
        //   attributes["y"] = "hello"  (Str, inserts 2)
        //   attributes["z"] = 42       (Int, inserts 2)
        //
        // Expected output (6 rows):
        //   Row 0-1: passthrough (str="a"/"b", int=null)
        //   Row 2-3: inserts for y (str="hello", int=null)
        //   Row 4-5: inserts for z (str=null, int=42)
        let existing = build_attrs_batch(&[(0, "x", 1, "a"), (1, "x", 1, "b")]);

        let mask_y = BooleanArray::from(vec![false, false]);
        let mask_z = BooleanArray::from(vec![false, false]);
        let parent_ids_y = UInt16Array::from(vec![0u16, 1]);
        let parent_ids_z = UInt16Array::from(vec![0u16, 1]);

        let result = upsert_attributes(
            &existing,
            &[
                AttributeUpsert {
                    attrs_key: "y",
                    existing_key_mask: mask_y,
                    new_values: ColumnarValue::Scalar(ScalarValue::Utf8(Some("hello".into()))),
                    upsert_parent_ids: parent_ids_y,
                },
                AttributeUpsert {
                    attrs_key: "z",
                    existing_key_mask: mask_z,
                    new_values: ColumnarValue::Scalar(ScalarValue::Int64(Some(42))),
                    upsert_parent_ids: parent_ids_z,
                },
            ],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 6);

        // Check type column: rows 0-1 Str(1), rows 2-3 Str(1), rows 4-5 Int(2).
        let types = result
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(types.value(0), 1); // passthrough Str
        assert_eq!(types.value(1), 1); // passthrough Str
        assert_eq!(types.value(2), 1); // insert y: Str
        assert_eq!(types.value(3), 1); // insert y: Str
        assert_eq!(types.value(4), 2); // insert z: Int
        assert_eq!(types.value(5), 2); // insert z: Int

        // Check str column: rows 0-3 have values, rows 4-5 null.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "a"); // passthrough
        assert_eq!(strs.value(1), "b"); // passthrough
        assert_eq!(strs.value(2), "hello"); // insert y
        assert_eq!(strs.value(3), "hello"); // insert y
        assert!(
            result
                .column_by_name(consts::ATTRIBUTE_STR)
                .unwrap()
                .is_null(4)
        ); // insert z
        assert!(
            result
                .column_by_name(consts::ATTRIBUTE_STR)
                .unwrap()
                .is_null(5)
        ); // insert z

        // Check int column exists and has correct values.
        let int_col = result.column_by_name(consts::ATTRIBUTE_INT).unwrap();
        assert!(int_col.is_null(0)); // passthrough (no int in existing)
        assert!(int_col.is_null(1)); // passthrough
        assert!(int_col.is_null(2)); // insert y (str type, not int)
        assert!(int_col.is_null(3)); // insert y
        // Decode int column to check insert z values.
        let int_plain = cast(int_col, &DataType::Int64).unwrap();
        let ints = int_plain
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap();
        assert_eq!(ints.value(4), 42); // insert z
        assert_eq!(ints.value(5), 42); // insert z
    }

    #[test]
    fn test_upsert_attributes_batched_mixed_update_and_insert() {
        // Test batched upsert with one update (existing key) and one insert (new key).
        //
        // Existing:
        //   Row 0: parent_id=0, key="x", type=Str(1), str="a"
        //   Row 1: parent_id=0, key="y", type=Str(1), str="b"
        //   Row 2: parent_id=1, key="x", type=Str(1), str="c"
        //   Row 3: parent_id=1, key="y", type=Str(1), str="d"
        //
        // Upserts:
        //   attributes["x"] = "updated" (both parents have "x" → 2 updates, 0 inserts)
        //   attributes["z"] = "new"     (neither parent has "z" → 0 updates, 2 inserts)
        //
        // Expected output (6 rows):
        //   Row 0: parent_id=0, key="x", str="updated"  (update from upsert 0)
        //   Row 1: parent_id=0, key="y", str="b"        (passthrough)
        //   Row 2: parent_id=1, key="x", str="updated"  (update from upsert 0)
        //   Row 3: parent_id=1, key="y", str="d"        (passthrough)
        //   Row 4: parent_id=0, key="z", str="new"      (insert from upsert 1)
        //   Row 5: parent_id=1, key="z", str="new"      (insert from upsert 1)
        let existing = build_attrs_batch(&[
            (0, "x", 1, "a"),
            (0, "y", 1, "b"),
            (1, "x", 1, "c"),
            (1, "y", 1, "d"),
        ]);

        // Mask for "x": rows 0 and 2 match.
        let mask_x = BooleanArray::from(vec![true, false, true, false]);
        // Mask for "z": no rows match.
        let mask_z = BooleanArray::from(vec![false, false, false, false]);

        // For "x": both parents have it → 2 updates, 0 inserts.
        let parent_ids_x = UInt16Array::from(vec![0u16, 1]);
        // For "z": neither parent has it → 0 updates, 2 inserts.
        let parent_ids_z = UInt16Array::from(vec![0u16, 1]);

        let result = upsert_attributes(
            &existing,
            &[
                AttributeUpsert {
                    attrs_key: "x",
                    existing_key_mask: mask_x,
                    new_values: ColumnarValue::Scalar(ScalarValue::Utf8(Some("updated".into()))),
                    upsert_parent_ids: parent_ids_x,
                },
                AttributeUpsert {
                    attrs_key: "z",
                    existing_key_mask: mask_z,
                    new_values: ColumnarValue::Scalar(ScalarValue::Utf8(Some("new".into()))),
                    upsert_parent_ids: parent_ids_z,
                },
            ],
        )
        .unwrap();

        assert_eq!(result.num_rows(), 6);

        // Check parent_id.
        let pids = result
            .column_by_name(consts::PARENT_ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();
        assert_eq!(pids.values(), &[0, 0, 1, 1, 0, 1]);

        // Check key column.
        let keys = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_KEY).unwrap());
        assert_eq!(keys.value(0), "x"); // update
        assert_eq!(keys.value(1), "y"); // passthrough
        assert_eq!(keys.value(2), "x"); // update
        assert_eq!(keys.value(3), "y"); // passthrough
        assert_eq!(keys.value(4), "z"); // insert
        assert_eq!(keys.value(5), "z"); // insert

        // Check str column.
        let strs = decode_to_utf8(result.column_by_name(consts::ATTRIBUTE_STR).unwrap());
        assert_eq!(strs.value(0), "updated"); // update from upsert 0
        assert_eq!(strs.value(1), "b"); // passthrough
        assert_eq!(strs.value(2), "updated"); // update from upsert 0
        assert_eq!(strs.value(3), "d"); // passthrough
        assert_eq!(strs.value(4), "new"); // insert from upsert 1
        assert_eq!(strs.value(5), "new"); // insert from upsert 1
    }

    // ---- merge_parent_id_column dict tests ----

    /// Helper to build a Dict(u8, UInt32) array from a slice of u32 values.
    fn dict_u32_u8(values: &[u32]) -> ArrayRef {
        let plain = Arc::new(UInt32Array::from(values.to_vec())) as ArrayRef;
        let dict_type = DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32));
        cast(&plain, &dict_type).unwrap()
    }

    /// Helper to build a Dict(u16, UInt32) array from a slice of u32 values.
    fn dict_u32_u16(values: &[u32]) -> ArrayRef {
        let plain = Arc::new(UInt32Array::from(values.to_vec())) as ArrayRef;
        let dict_type =
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32));
        cast(&plain, &dict_type).unwrap()
    }

    /// Helper to build a minimal ResolvedUpsert<UInt32Type> for parent_id column tests.
    fn parent_id_test_resolved<'a>(
        parent_ids: &'a UInt32Array,
        mask: &'a BooleanArray,
        num_updates: usize,
        num_inserts: usize,
    ) -> ResolvedUpsert<'a, UInt32Type> {
        ResolvedUpsert {
            attrs_key: "test",
            mask,
            attr_value_type: AttributeValueType::Str,
            target_col_name: Some(consts::ATTRIBUTE_STR),
            new_values_array: None,
            new_values_scalar: None,
            upsert_parent_ids: parent_ids,
            num_updates,
            num_inserts,
        }
    }

    /// Decode a potentially dict-encoded UInt32 column to a plain UInt32Array for assertions.
    fn decode_parent_ids(col: &ArrayRef) -> Vec<u32> {
        let plain = cast(col, &DataType::UInt32).unwrap();
        let arr = plain
            .as_any()
            .downcast_ref::<UInt32Array>()
            .expect("expected UInt32Array after cast");
        arr.values().to_vec()
    }

    #[test]
    fn test_merge_parent_id_dict_no_inserts() {
        // No inserts — column should be returned unchanged (early-out path).
        let existing = dict_u32_u8(&[0, 1, 0]);
        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        let parent_ids = UInt32Array::from(Vec::<u32>::new());
        let mask = BooleanArray::from(vec![false, false, false]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 0)];

        let (_, result_col) = merge_parent_id_column(&field, existing, &resolved, 0).unwrap();

        assert_eq!(result_col.len(), 3);
        assert_eq!(decode_parent_ids(&result_col), vec![0, 1, 0]);
    }

    #[test]
    fn test_merge_parent_id_dict_u8_inserts_existing_values() {
        // Existing: Dict(u8, u32) [0, 1, 0] (dict values [0, 1])
        // Insert parent IDs: [0, 1] — all values already in dict
        // Expected: 5 rows [0, 1, 0, 0, 1], output stays Dict(u8, _)
        let existing = dict_u32_u8(&[0, 1, 0]);
        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        let parent_ids = UInt32Array::from(vec![0u32, 1]);
        let mask = BooleanArray::from(vec![false, false, false]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 2)];

        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, 2).unwrap();

        assert_eq!(result_col.len(), 5);
        assert_eq!(decode_parent_ids(&result_col), vec![0, 1, 0, 0, 1]);
        // Should stay as Dict(u8, _) since no new values were added.
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "expected Dict(u8, UInt32) output, got {:?}",
            result_field.data_type()
        );
    }

    #[test]
    fn test_merge_parent_id_dict_u8_inserts_novel_values() {
        // Existing: Dict(u8, u32) [0, 1, 0] (dict values [0, 1])
        // Insert parent IDs: [2, 3] — novel values
        // Expected: 5 rows [0, 1, 0, 2, 3], dict values extended, output stays Dict(u8, _)
        let existing = dict_u32_u8(&[0, 1, 0]);
        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        let parent_ids = UInt32Array::from(vec![2u32, 3]);
        let mask = BooleanArray::from(vec![false, false, false]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 2)];

        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, 2).unwrap();

        assert_eq!(result_col.len(), 5);
        assert_eq!(decode_parent_ids(&result_col), vec![0, 1, 0, 2, 3]);
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "expected Dict(u8, UInt32) output, got {:?}",
            result_field.data_type()
        );
    }

    #[test]
    fn test_merge_parent_id_dict_u16_inserts() {
        // Existing: Dict(u16, u32) [10, 20, 10] (dict values [10, 20])
        // Insert parent IDs: [30, 10] — one novel, one existing
        // Expected: 5 rows [10, 20, 10, 30, 10]
        let existing = dict_u32_u16(&[10, 20, 10]);
        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        let parent_ids = UInt32Array::from(vec![30u32, 10]);
        let mask = BooleanArray::from(vec![false, false, false]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 2)];

        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, 2).unwrap();

        assert_eq!(result_col.len(), 5);
        assert_eq!(decode_parent_ids(&result_col), vec![10, 20, 10, 30, 10]);
        // Cardinality is 3, fits in u8
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "expected Dict(u8, UInt32) output, got {:?}",
            result_field.data_type()
        );
    }

    #[test]
    fn test_merge_parent_id_dict_u8_overflows_to_u16() {
        // Existing: Dict(u8, u32) with 256 distinct values (maxed out u8)
        // Insert a novel value → cardinality becomes 257, exceeds u8
        // Expected: output widens to Dict(u16, _)
        let distinct_values: Vec<u32> = (0..256u32).collect();
        let existing = dict_u32_u8(&distinct_values);
        assert!(
            matches!(existing.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "precondition: existing should be Dict(u8, UInt32)"
        );

        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        // Insert value 999, which is not in the existing dict
        let parent_ids = UInt32Array::from(vec![999u32]);
        let mask = BooleanArray::from(vec![false; 256]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 1)];

        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, 1).unwrap();

        assert_eq!(result_col.len(), 257);
        // Should widen to Dict(u16, _) since cardinality (257) exceeds u8 max (256).
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt16),
            "expected Dict(u16, UInt32) output after overflow, got {:?}",
            result_field.data_type()
        );
        // Verify first, last existing, and inserted value.
        let decoded = decode_parent_ids(&result_col);
        assert_eq!(decoded[0], 0);
        assert_eq!(decoded[255], 255);
        assert_eq!(decoded[256], 999);
    }

    #[test]
    fn test_merge_parent_id_dict_batched_two_upserts() {
        // Existing: Dict(u8, u32) [0, 1, 0, 1] (dict values [0, 1])
        // Upsert 0: 1 update + 1 insert, parent_ids [0, 5]
        // Upsert 1: 0 updates + 2 inserts, parent_ids [1, 6]
        // Expected: 7 rows [0, 1, 0, 1, 5, 1, 6]
        let existing = dict_u32_u8(&[0, 1, 0, 1]);
        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);

        let parent_ids_0 = UInt32Array::from(vec![0u32, 5]);
        let mask_0 = BooleanArray::from(vec![true, false, false, false]);

        let parent_ids_1 = UInt32Array::from(vec![1u32, 6]);
        let mask_1 = BooleanArray::from(vec![false, false, false, false]);

        let resolved = vec![
            parent_id_test_resolved(&parent_ids_0, &mask_0, 1, 1),
            parent_id_test_resolved(&parent_ids_1, &mask_1, 0, 2),
        ];

        let total_num_inserts = 3; // 1 from upsert 0 + 2 from upsert 1
        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, total_num_inserts).unwrap();

        assert_eq!(result_col.len(), 7);
        assert_eq!(decode_parent_ids(&result_col), vec![0, 1, 0, 1, 5, 1, 6]);
        assert!(
            matches!(result_field.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt8),
            "expected Dict(u8, UInt32) output, got {:?}",
            result_field.data_type()
        );
    }

    #[test]
    fn test_merge_parent_id_dict_overflow_falls_back_to_plain() {
        // Existing: Dict(u16, u32) with 65535 distinct values (one less than max u16 cardinality).
        // Insert two novel values → total cardinality becomes 65537, which exceeds the 65535
        // limit in merge_parent_id_column_dict, causing it to return None and fall back to plain.
        let distinct_values: Vec<u32> = (0..65535u32).collect();
        let existing = dict_u32_u16(&distinct_values);
        assert!(
            matches!(existing.data_type(), DataType::Dictionary(k, _) if **k == DataType::UInt16),
            "precondition: existing should be Dict(u16, UInt32)"
        );
        assert_eq!(existing.len(), 65535);

        let field = Field::new(consts::PARENT_ID, existing.data_type().clone(), false);
        // Insert two novel values. The first (99999) brings cardinality to 65536 which still
        // fits in u16 (key=65535). The second (100000) would need key=65536, exceeding u16,
        // so the dict path bails out.
        let parent_ids = UInt32Array::from(vec![99999u32, 100000]);
        let mask = BooleanArray::from(vec![false; 65535]);
        let resolved = vec![parent_id_test_resolved(&parent_ids, &mask, 0, 2)];

        let (result_field, result_col) =
            merge_parent_id_column(&field, existing, &resolved, 2).unwrap();

        assert_eq!(result_col.len(), 65537);
        // Should fall back to plain UInt32 since dict cardinality would exceed u16 max.
        assert_eq!(
            result_field.data_type(),
            &DataType::UInt32,
            "expected plain UInt32 output after overflow, got {:?}",
            result_field.data_type()
        );
        // Verify values.
        let decoded = decode_parent_ids(&result_col);
        assert_eq!(decoded[0], 0);
        assert_eq!(decoded[65534], 65534);
        assert_eq!(decoded[65535], 99999);
        assert_eq!(decoded[65536], 100000);
    }
}
