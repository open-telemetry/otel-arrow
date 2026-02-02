// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains utilities for applying "transport optimized" encoding to Attributes
//! [`RecordBatch`]s.
//!
//! The goal of the encoding is to optimize the compression ratio of the IPC serialized record
//! batch when general compression is applied.
//!
//! In this encoding, the record batch sorted by the columns:
//! - type
//! - key
//! - value
//!
//! Delta encoding is then applied to some segments of the parent_id column.
//!
//! The attribute values are contained within multiple columns. There is one column per type of
//! attribute value, with the exception Map & Slice type attributes which share a single values
//! column called "ser".
//!
//! The "type" column identifies which values column contains the attribute value for the row. It
//! contains non-nullable u8 corresponding to the [`AttributeValueType`] enum.
//!
//! The "parent_id" column is delta-encoded for sequences of rows sharing the same type, key and
//! value, with two exceptions:
//! - parent_ids for Map & Slice types are not delta encoded
//! - attributes where the value is null do not have their parent_id column delta encoded.
//!
//! A row will be interpreted as containing a null attribute if:
//! - the type column = 0, which corresponds to [`AttributeValueType::Empty`]
//! - the value column contains a null
//! - the value column is not present indicating all rows for values of the type are null
//!
//! Note that the latter two cases (column contains null, values column missing), while valid arrow
//! data, are atypical ways to represent a null attribute. It is preferred to set
//! `AttributeValueType::Empty` in the type column. Not only does this more clearly express the
//! semantic of an absent value, but the process of adding this encoding is more optimal if there
//! are no nulls in the values column where type != `Empty`.
//!
//! Example:
//!
//!  type      | key    | str  | int   | parent_id
//! -----------|---- ---|------|-------|-----------
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 1
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 2 (delta encoded b/c type,key,val are equal to previous row)
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 3
//!  1 (str)   | "k1"   | "v2" | null  |  1     <- parent_id = 1 (value changed, breaks delta encoding sequence)
//!  1 (str)   | "k1"   | "v2" | null  |  1     <- parent_id = 2
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 1 (key changed, breaks delta encoding sequence)
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 2
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 3
//!  2 (int)   | null   | "v2" | 1     |  1     <- parent_id = 1 (type changed, breaks delta encoding sequence)
//!  2 (int)   | null   | "v2" | 1     |  1     <- parent_id = 2
//! ...
//!  0 (empty) | null   | null | null  |  1     <- parent_id = 1 (value null b/c type = empty, no delta encoding)
//! ...
//!  3 (float) | null   | null | null  |  1     <- parent_id = 1 (value null b/c value column missing, no delta encoding)
//! ...
//!  1 (str)   | null   | null | null  |  1     <- parent_id = 1 (value null b/c null in values column, no delta encoding)
//! ...
//!  5 (map)   | null   | null | null  |  1     <- parent_id = 1 (no delta encoding. not supported by type = map)
//! ...
//! 6 (slice)   | null   | null | null  |  1     <- parent_id = 1 (no delta encoding. not supported by type = slice)
//!

use std::{
    ops::{Range, Sub},
    sync::Arc,
};

use arrow::{
    array::{
        Array, ArrayRef, ArrowNativeTypeOp, ArrowPrimitiveType, BinaryArray, BooleanArray,
        BooleanBufferBuilder, DictionaryArray, Float64Array, Int64Array, NullBufferBuilder,
        PrimitiveArray, RecordBatch, StringArray, UInt8Array, UInt16Array, UInt32Array,
    },
    buffer::{BooleanBuffer, MutableBuffer, NullBuffer, OffsetBuffer, ScalarBuffer},
    compute::{SortOptions, cast, not, rank, take},
    datatypes::{ArrowNativeType, DataType, Schema, ToByteSlice, UInt8Type, UInt16Type},
    util::{bit_iterator::BitIndexIterator, bit_util},
};

use crate::{
    arrays::{get_required_array, get_u8_array},
    error::{Error, Result},
    otap::transform::create_next_element_equality_array,
    otlp::attributes::AttributeValueType,
    schema::{FieldExt, consts, update_field_metadata},
};

/// Applies the transport optimized encoding to the record batch (see module documentation for
/// description of the encoding).
///
/// This procedure first sorts the types and keys together. From this sorted array, it partitions
/// by type to select the correct values column for this segment of sorted rows, and subsequently
/// partitions by key within the type range to select segments of the values column for sorting.
/// It then sorts each segment of the values column, partitions this sorted values segment, and
/// optionally applies delta encoding to the parent_ids within each partition (according to the
/// rules for when to delta encode, which are outlined above).
///
/// The goal is to sort the rows as efficiently as possible, while collecting enough context during
/// the process so that we know: a) when to apply delta encoding to the parent IDs (without having
/// to do a second partition pass, and b) how to efficiently reconstruct the sorted values columns
/// at the end)
///
pub(crate) fn transport_optimize_encode_attrs<T: ArrowPrimitiveType>(
    record_batch: &RecordBatch,
) -> Result<RecordBatch>
where
    <T as ArrowPrimitiveType>::Native: Ord + Sub<Output = <T as ArrowPrimitiveType>::Native>,
{
    if record_batch.num_rows() <= 1 {
        // sorting & encoding rows would not change the data if there is 1 or fewer rows.
        // skip handling the data and just update the schema metadata
        let schema = update_field_metadata(
            record_batch.schema_ref(),
            consts::PARENT_ID,
            consts::metadata::COLUMN_ENCODING,
            consts::metadata::encodings::QUASI_DELTA,
        );

        // safety: we can expect this to succeed because we're reusing the same schema/columns from
        // the incoming batch, just with a single updated field metadata
        let result = RecordBatch::try_new(Arc::new(schema), record_batch.columns().to_vec())
            .expect("can create record batch");

        return Ok(result);
    }

    // initialize builders for each attribute value column, indexed by AttributeValueType.
    //
    // the final sorted layout for each column will be:
    // - rows grouped by attribute type (matching the type column)
    // - rows outside a column's type segment are set to null
    // - within the type segment, rows sorted by: key, then value
    //
    // example for ATTRIBUTE_STR (type=1):
    //   - Rows where type == 1: non-null, sorted by (key, value)
    //   - Rows where type != 1: null
    let mut sorted_val_columns: [Option<SortedValuesColumnBuilder>; 8] = [
        None, // empty - no values column for empty attrs
        record_batch
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(SortedValuesColumnBuilder::try_new)
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(SortedValuesColumnBuilder::try_new)
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_DOUBLE)
            .map(SortedValuesColumnBuilder::try_new)
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_BOOL)
            .map(SortedValuesColumnBuilder::try_new)
            .transpose()?,
        // map/slice are special case - see below
        None, // map
        None, // slice
        record_batch
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(SortedValuesColumnBuilder::try_new)
            .transpose()?,
    ];

    // ser column is special case because more than one attribute type is stored in this
    // column (both AttributeValueType::Map and AttributeValueType::Slice)
    let mut sorted_ser_column = record_batch
        .column_by_name(consts::ATTRIBUTE_SER)
        .map(SortedValuesColumnBuilder::try_new)
        .transpose()?;

    let parent_id_col = get_required_array(record_batch, consts::PARENT_ID)?;

    // if the parent ID column is a dictionary, we cast it to a primitive array so we can read
    // directly from the ScalarBuffer containing the values. We then cast it back to dictionary
    // array when reconstructing the final dataset.
    // TODO investigate if there's a more performant alternative to doing this cast
    let mut parent_dict_key_type = None;
    let parent_id_column_vals = match parent_id_col.data_type() {
        DataType::Dictionary(k, v) => {
            let as_prim = cast(parent_id_col, &T::DATA_TYPE).map_err(|_| {
                // cast would only fail here if the dictionary values were not something that can
                // be cast to <T as ArrowPrimitiveType>::DATA_TYPE (e.g. if the parent_id column
                // was completely the wrong type, indicating an invalid record batch was received).
                Error::ColumnDataTypeMismatch {
                    name: consts::PARENT_ID.into(),
                    actual: *v.clone(),
                    expect: T::DATA_TYPE.clone(),
                }
            })?;
            parent_dict_key_type = Some(*k.clone());
            as_prim
                .as_any()
                .downcast_ref::<PrimitiveArray<T>>()
                // safety: as_prim is created casting to the type we're expecting here
                .expect("cast to PrimitiveArray<T> produced correct type")
                .values()
                .clone()
        }
        other_dt => parent_id_col
            .as_any()
            .downcast_ref::<PrimitiveArray<T>>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::PARENT_ID.into(),
                actual: other_dt.clone(),
                expect: T::DATA_TYPE.clone(),
            })?
            .values()
            .clone(), // internally just an Arc<Bytes>
    };

    // this is the buffer into which we'll be appending the encoded parent IDs
    let mut encoded_parent_id_column = Vec::with_capacity(record_batch.num_rows());

    // first sort by type/key to indices. This will allow us to take new columns for type/key
    // and we'll use these indices later on to take values columns/parent_id column for each
    // partition of rows with equivalent type & key
    let type_col = get_u8_array(record_batch, consts::ATTRIBUTE_TYPE)?;
    let key_col = get_required_array(record_batch, consts::ATTRIBUTE_KEY)?;
    let type_and_key_indices = sort_attrs_type_and_keys_to_indices(type_col, key_col.clone())?;

    // safety: we can call "expect" here because the indices we're taking are computed from the
    // indices of original batch by the sort_attrs_type_and_keys_to_indices function
    let type_col_sorted =
        take(type_col, &type_and_key_indices, None).expect("can take sorted indices in bounds");
    let key_col_sorted =
        take(key_col, &type_and_key_indices, None).expect("can take sorted indices in bounds");

    // safety: `type_col_sorted` was computed just above by "taking" the type column, which we have
    // already checked is a UInt8Array by the call to `get_u8_array`
    let type_prim_arr = type_col_sorted
        .as_any()
        .downcast_ref::<UInt8Array>()
        .expect("type is UInt8Array");
    let type_col_sorted_bytes = type_prim_arr.values().inner().as_slice();

    let mut type_partitions = Vec::with_capacity(8); // 8 = number of possible value types
    collect_partition_from_array(&type_col_sorted, &mut type_partitions)?;

    // These `Vec`s are used farther below when handling the values w/in each range of sorted keys.
    // they're allocated ahead of time here so we can reuse the allocation for each range
    let mut key_ranges = Vec::new();
    let mut values_ranges = Vec::new();
    let mut key_range_indices_values_sorted = Vec::new();

    // iterates ranges of types in ascending order. For example, we iterate over type ranges
    // Empty, String, Int. and so on..
    for type_range in type_partitions {
        let type_range_attr_type =
            AttributeValueType::try_from(type_col_sorted_bytes[type_range.start])?;

        let is_ser_col_type = type_range_attr_type == AttributeValueType::Map
            || type_range_attr_type == AttributeValueType::Slice;

        let sorted_val_col_builder = if is_ser_col_type {
            sorted_ser_column.as_mut()
        } else {
            sorted_val_columns[type_range_attr_type as usize].as_mut()
        };

        // this contains indices of rows from the original record batch that are of the type for
        // the range we're currently handling, sorted by key. For example, if the current range is
        // for string type, this will contain all indices from the original record batch that are a
        // string type attribute, sorted attribute key.
        let type_range_indices_key_sorted =
            type_and_key_indices.slice(type_range.start, type_range.len());

        // sort the values columns for values of this type
        if let Some(sorted_val_col) = sorted_val_col_builder {
            // take the values at the indices for this value type. This produces an array, sorted
            // by key, for all the attribute values that have the type we're currently processing.
            //
            // Even though we eventually discard this Array, materializing it temporarily is worth
            // it for a couple reasons. First, this calls arrow's `take` compute kernel, which will
            // create a new null buffer and once we have this, counting the number of nulls is
            // cheap. There  are a few optimizations that happen later on if we know that none of
            // the values are null. Second, the values will be closer together in memory, so we get
            // better cache locality when we sort them.
            let values_type_range_by_key =
                sorted_val_col.take_source(&type_range_indices_key_sorted)?;

            // init struct that will help us sort the values w/in each range of equivalent keys
            let mut values_sorter = AttrValuesSorter::try_new(&values_type_range_by_key)?;

            // partition the keys column into ranges where all rows have the same key.
            // the indices in the ranges produced by this call will be relative to the type range.
            // they can be converted back to the range in the original record batch by indexing
            // type_range_indices_key_sorted
            key_ranges.clear();
            collect_partitions_for_range(&type_range, &key_col_sorted, &mut key_ranges)?;

            // iterate over contiguous ranges that have the same attribute key
            for key_range in &key_ranges {
                // this vec will contain a list of indices, relative to the key_range, sorted by
                // the values. Note: because these indices will be relative to the key_range,
                // adding key_range.start to the index will make it relative to the type_range
                key_range_indices_values_sorted.clear();
                key_range_indices_values_sorted.reserve(key_range.len());

                values_ranges.clear();
                values_sorter.sort_and_partition_range(
                    key_range,
                    sorted_val_col,
                    &mut key_range_indices_values_sorted,
                    &mut values_ranges,
                )?;

                // keep the current length of the encoded parent IDs for later. Below where we
                // iterate over the values ranges to sort and add delta encoding, and this will
                // make it easier to calculate the ranges containing parent IDs for each value
                let values_range_offset = encoded_parent_id_column.len();

                // push the values for parent IDs that have this key into the results column.
                // They may be inserted in the wrong order, but we're going to sort them afterward
                // for any non-null ranges for type that support quasi-delta encoding
                encoded_parent_id_column.extend(key_range_indices_values_sorted.iter().map(
                    |idx| {
                        let type_range_idx =
                            type_range_indices_key_sorted.value(key_range.start + *idx);
                        parent_id_column_vals[type_range_idx as usize]
                    },
                ));

                for values_range in &values_ranges {
                    let parent_ids_range = &mut encoded_parent_id_column[values_range.range.start
                        + values_range_offset
                        ..values_range.range.end + values_range_offset];

                    // nulls never count as equal for the purposes of delta encoding, and neither
                    // are Map & Slice types
                    if !values_range.is_null
                        && type_range_attr_type != AttributeValueType::Map
                        && type_range_attr_type != AttributeValueType::Slice
                    {
                        sort_and_delta_encode(parent_ids_range);
                    }
                }
            }
        } else {
            // The values column is missing - which either means that the column was all null, or the
            // attribute type is "empty". Either way, we interpret this as "null' attribute, which
            // means we should not delta encode the column. We just append the unsorted parent IDs
            // for the type range
            encoded_parent_id_column.extend(
                type_range_indices_key_sorted
                    .values()
                    .iter()
                    .map(|idx| parent_id_column_vals[*idx as usize]),
            );
        }

        // push null values into the all the values columns that were not of the type for this
        // range. This will fill in gaps in these columns.
        for attr_type in [
            AttributeValueType::Str,
            AttributeValueType::Int,
            AttributeValueType::Double,
            AttributeValueType::Bool,
            AttributeValueType::Bytes,
        ] {
            if attr_type == type_range_attr_type {
                // skip because we already pushed the sorted section for this type
                continue;
            }

            if let Some(sorted_val_col_builder) = sorted_val_columns[attr_type as usize].as_mut() {
                let len = type_range.len();
                // append the nulls to the null buffer
                sorted_val_col_builder.append_n_nulls(len);
                // fill in the data buffers with default values
                sorted_val_col_builder.append_n_default_values(len);
            }
        }

        // push unsorted values from slice/map type if not already pushed. This is handled as
        // special case from the loop above b/c both slice and map attrs types use the same column,
        // and we only want to append segment to the column builder once per value column per type
        if type_range_attr_type != AttributeValueType::Map
            && type_range_attr_type != AttributeValueType::Slice
        {
            if let Some(sorted_val_col_builder) = sorted_ser_column.as_mut() {
                let len = type_range.len();
                // append the nulls to the null buffer
                sorted_val_col_builder.append_n_nulls(len);
                // fill in the data buffers with default values
                sorted_val_col_builder.append_n_default_values(len);
            }
        }
    }

    // finalize the new parent_id column
    let mut parent_id_col = Arc::new(PrimitiveArray::<T>::new(
        ScalarBuffer::from(encoded_parent_id_column),
        None,
    )) as ArrayRef;

    // If the original Parent ID type was a dictionary, cast it back to a dictionary of this type.
    if let Some(dict_key_type) = parent_dict_key_type {
        parent_id_col = match cast(
            &parent_id_col,
            &DataType::Dictionary(Box::new(dict_key_type), Box::new(T::DATA_TYPE)),
        ) {
            Ok(as_dict) => as_dict,

            // If this cast failed, it means that the deltas have more unique values than would fit
            // into the original sized dictionary. Just return the native array as a fallback
            Err(_) => parent_id_col,
        }
    }

    // rebuild the record batch with all the sorted/encoded columns ...
    let mut fields = vec![];
    let mut columns = vec![];
    for field in record_batch.schema().fields() {
        let field_name = field.name();

        if field_name == consts::PARENT_ID {
            // add encoding the metadata to the parent_id column
            fields.push(
                field
                    .as_ref()
                    .clone()
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA)
                    .with_data_type(parent_id_col.data_type().clone()),
            );
            columns.push(parent_id_col.clone());
            continue;
        } else {
            fields.push(field.as_ref().clone());
        }

        if field_name == consts::ATTRIBUTE_TYPE {
            columns.push(type_col_sorted.clone());
            continue;
        }

        if field_name == consts::ATTRIBUTE_KEY {
            columns.push(key_col_sorted.clone());
            continue;
        }

        if field_name == consts::ATTRIBUTE_STR {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Str as usize]
                .take()
                .expect("str attr column present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_INT {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Int as usize]
                .take()
                .expect("int attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BOOL {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Bool as usize]
                .take()
                .expect("bool attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_DOUBLE {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Double as usize]
                .take()
                .expect("double attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BYTES {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Bytes as usize]
                .take()
                .expect("bytes attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_SER {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_ser_column
                .take()
                .expect("serialized attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        return Err(Error::UnexpectedRecordBatchState {
            reason: format!("unexpected column {field_name} found in record batch"),
        });
    }

    // safety: this should only fail if the columns don't match the schema or the arrays
    // have the wrong lengths, neither of which should be the case here
    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("can recreate record batch");

    Ok(batch)
}

/// Sort the slice in ascending order, and then apply delta encoding. We sort because
/// negative deltas are not allowed, since the parent_id column is always an unsigned type.
fn sort_and_delta_encode<T: Copy + Ord + Sub<Output = T>>(vals: &mut [T]) {
    vals.sort_unstable();
    let mut prev = vals[0];
    for val in vals.iter_mut().skip(1) {
        let curr = *val;
        *val = curr - prev;
        prev = curr;
    }
}

/// sort the type and key and return the indices of the sorted batch.
///
/// To improve sort performance, we use an approach similar to arrow's RowConverter, which
/// combines the columns into a row based byte array and sorts it.
///
/// However, this implementation is more optimal when "keys" are dictionary encoded, which they
/// usually will be. Unlike RowConverter, we don't expand the dictionary when creating the sorting
/// target. Instead, if keys is dictionary encoded, we rank the dictionary values, convert the
/// dictionary keys to their ranks, and sort that alongside type. The dictionary keys are normally
/// low cardinality, so ranking them is faster than expanding the dictionary.
fn sort_attrs_type_and_keys_to_indices(
    type_col: &UInt8Array,
    key_col: ArrayRef,
) -> Result<UInt32Array> {
    let len = type_col.len();
    let type_bytes = type_col.values().inner().as_slice();
    match key_col.data_type() {
        DataType::Dictionary(key, val) => match (*key.clone(), *val.clone()) {
            (DataType::UInt8, DataType::Utf8) => {
                // safety: we've just checked that the datatype is what we're casting to
                let dict_arr = key_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("can downcast to dict<u8>");
                let keys = dict_arr.keys();
                let keys_values = keys.values().inner().as_slice();

                // safety: we've checked the datatype for this values array is utf8 and rank will
                // will only return error for types that don't support rank, which utf8 does
                let val_ranks = rank(dict_arr.values(), None).expect("can rank string array");

                // concat the type and key rank into a a u16 vec where the higher end byte of each
                // element is the type, and the lower end is the key's rank
                let mut to_sort = vec![0u16; len];
                for i in 0..len {
                    to_sort[i] += (type_bytes[i] as u16) << 8u16;
                }
                for i in 0..len {
                    let rank = val_ranks[keys_values[i] as usize];
                    to_sort[i] += rank as u16;
                }

                let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
                with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

                Ok(PrimitiveArray::from_iter_values(
                    with_indices.into_iter().map(|(rank, _)| rank as u32),
                ))
            }
            (DataType::UInt16, DataType::Utf8) => {
                // safety: we've just checked that the datatype is what we're casting to
                let dict_arr = key_col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("can downcast to dict<u16>");
                let keys = dict_arr.keys();
                let keys_values = keys.values();

                // safety: we've checked the datatype for this values array is utf8 and rank will
                // will only return error for types that don't support rank, which utf8 does
                let val_ranks = rank(dict_arr.values(), None).expect("can rank string array");

                // concat the type and key rank into a u32 vec where the higher end byte of each
                // element is the type, and the lower bytes are the key's rank
                let mut to_sort = vec![0u32; len];
                for i in 0..len {
                    to_sort[i] += (type_bytes[i] as u32) << 16u32;
                }
                for i in 0..len {
                    let rank = val_ranks[keys_values[i] as usize];
                    to_sort[i] += rank;
                }

                let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
                with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

                Ok(PrimitiveArray::from_iter_values(
                    with_indices.into_iter().map(|(rank, _)| rank as u32),
                ))
            }
            (other_key, DataType::Utf8) => Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                actual: other_key,
            }),
            (_, other_value) => Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof: vec![DataType::Utf8],
                actual: other_value,
            }),
        },
        DataType::Utf8 => {
            // sort by native keys. It would be unusual if we ended up in this branch, which is
            // why it is not as well optimized as the cases above. Normally, we'd only end up with
            // non-dict encoded keys if a batch were received that had more than u16::MAX_VALUE
            // unique attribute keys.

            // safety: we've already checked the datatype is Utf8 just above
            let keys_as_str = key_col
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("can downcast to StringArray");

            // safety: we've checked the datatype for this values array is utf8 and rank will
            // will only return error for types that don't support rank, which utf8 doe
            let keys_rank = rank(keys_as_str, None).expect("can rank string array");

            // concat the type and key rank into a u64 vec where the higher end byte of each
            // element is the type, and the lower bytes are the key's rank
            let mut to_sort = vec![0u64; len];
            for i in 0..len {
                to_sort[i] += (type_bytes[i] as u64) << 32;
            }
            for i in 0..len {
                to_sort[i] += keys_rank[i] as u64;
            }

            let mut with_indices = to_sort.into_iter().enumerate().collect::<Vec<_>>();
            with_indices.sort_unstable_by(|a, b| a.1.cmp(&b.1));

            Ok(PrimitiveArray::from_iter_values(
                with_indices.into_iter().map(|(rank, _)| rank as u32),
            ))
        }
        other_data_type => Err(Error::UnexpectedRecordBatchState {
            reason: format!("found invalid type for attributes 'key' column {other_data_type:?}"),
        }),
    }
}

/// A builder for constructing sorted attribute values columns.
///
/// This builder is designed to efficiently reconstruct the values column from data that was
/// appended in a sorted order. It supports all the Arrow types which OTAP may use to contain
/// attribute values.
///
/// The builder maintains separate buffers for different data type components:
/// - `data`: holds the actual values for most types
/// - `bool_data`: specialized buffer for boolean values
/// - `offsets`: tracks variable-length data boundaries for Binary and Utf8 types
/// - `nulls`: tracks null values across all types
struct SortedValuesColumnBuilder {
    /// Reference to the original source array, used to determine data type and
    /// for taking values by index.
    source: ArrayRef,

    /// Buffer for storing the actual data values. Size and usage depends on data type:
    /// - Dictionary: stores u16 keys (2 bytes per element)
    /// - Int64/Float64: stores 8-byte values
    /// - Binary/Utf8: stores the actual byte content
    /// - Boolean: unused (uses `bool_data` instead)
    data: MutableBuffer,

    /// Specialized buffer builder for boolean values. Only used when the source
    /// array is of type Boolean, otherwise None.
    bool_data: Option<BooleanBufferBuilder>,

    /// Buffer for storing offsets into the `data` buffer. Only used for variable-length
    /// types (Binary and Utf8), where each offset marks the start position of a value.
    /// Contains (len + 1) offsets to define len value ranges.
    offsets: Option<MutableBuffer>,

    /// Builder for tracking null/non-null status of each element in the array.
    nulls: NullBufferBuilder,
}

impl SortedValuesColumnBuilder {
    fn try_new(arr: &ArrayRef) -> Result<Self> {
        let len = arr.len();
        let nulls = NullBufferBuilder::new(len);
        match arr.data_type() {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt16 => Ok(Self {
                    source: arr.clone(),
                    data: MutableBuffer::new(2 * len),
                    bool_data: None,
                    offsets: None,
                    nulls,
                }),
                other => Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::UInt16],
                    actual: other.clone(),
                }),
            },
            DataType::Int64 | DataType::Float64 => Ok(Self {
                source: arr.clone(),
                data: MutableBuffer::new(8 * len),
                bool_data: None,
                offsets: None,
                nulls,
            }),
            DataType::Binary => {
                let data_len = arr
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    // safety: we've checked the datatype is Binary
                    .expect("can downcast to Binary")
                    .values()
                    .len();
                Ok(Self {
                    source: arr.clone(),
                    data: MutableBuffer::new(data_len),
                    bool_data: None,
                    offsets: Some(MutableBuffer::new(4 * len)),
                    nulls,
                })
            }
            DataType::Utf8 => {
                let data_len = arr
                    .as_any()
                    .downcast_ref::<StringArray>()
                    // safety: we've checked the datatype is Binary
                    .expect("can downcast to Utf8")
                    .values()
                    .len();

                Ok(Self {
                    source: arr.clone(),
                    data: MutableBuffer::new(data_len),
                    bool_data: None,
                    offsets: Some(MutableBuffer::new(4 * len)),
                    nulls,
                })
            }

            DataType::Boolean => Ok(Self {
                source: arr.clone(),
                data: MutableBuffer::new(0),
                bool_data: Some(BooleanBufferBuilder::new(len)),
                offsets: None,
                nulls,
            }),
            other_data_type => Err(Error::UnexpectedRecordBatchState {
                reason: format!("found unexpected attribute value data type {other_data_type:?}"),
            }),
        }
    }

    /// Access some rows of the original values column by their indices
    ///
    /// This should only be called with indices computed from the original source. Calling this
    /// with out of bound indices will fail
    fn take_source(&self, indices: &UInt32Array) -> Result<Arc<dyn Array>> {
        // safety: take will only panic here if the indices are out of bounds, but this is only
        // being called with indices taken from the original record batch, so expect is safe
        Ok(take(self.source.as_ref(), indices, None).expect("indices out of bounds"))
    }

    fn append_n_default_values(&mut self, count: usize) {
        match self.source.data_type() {
            DataType::Dictionary(_, _) => {
                self.data.extend_zeros(count * 2);
            }
            DataType::Int64 | DataType::Float64 => {
                self.data.extend_zeros(count * 8);
            }
            DataType::Binary | DataType::Utf8 => {
                let curr_len = self.data.len();
                // safety: if the datatype is one of these types, we will have initialized
                // a builder for offsets in the constructor
                let offsets = self.offsets.as_mut().expect("offsets not None");
                offsets.extend(std::iter::repeat_n(curr_len as u32, count));
            }
            DataType::Boolean => {
                // safety: if the datatype is one of these types, we will have initialized
                // a builder for bool_data in the constructor
                let bool = self.bool_data.as_mut().expect("bool_data not None");
                bool.append_n(count, false);
            }
            _ => {}
        }
    }

    fn append_n_nulls(&mut self, count: usize) {
        self.nulls.append_n_nulls(count);
    }

    fn append_n_non_nulls(&mut self, count: usize) {
        self.nulls.append_n_non_nulls(count);
    }

    fn append_nulls(&mut self, nulls: &NullBuffer) {
        self.nulls.append_buffer(nulls);
    }

    fn append_data<T: ArrowNativeType>(&mut self, items: &[T]) {
        self.data.extend_from_slice(items);
    }

    fn append_offsets<T: ArrowNativeType>(&mut self, items: &[T]) {
        if let Some(offsets) = self.offsets.as_mut() {
            offsets.extend_from_slice(items);
        }
    }

    fn append_bools(&mut self, items: &BooleanBuffer) {
        if let Some(bool_data) = self.bool_data.as_mut() {
            bool_data.append_buffer(items);
        }
    }

    fn finish(mut self) -> Result<ArrayRef> {
        let len = self.source.len();
        let nulls = self.nulls.finish();
        match self.source.data_type() {
            DataType::Dictionary(k, _) => {
                match k.as_ref() {
                    DataType::UInt16 => {
                        let dict_source = self
                            .source
                            .as_any()
                            .downcast_ref::<DictionaryArray<UInt16Type>>()
                            // safety: we've checked the type is Dictionary
                            .expect("can downcast to DictionaryArray<u16>");
                        let keys_values = ScalarBuffer::new(self.data.into(), 0, len);
                        let keys = UInt16Array::new(keys_values, nulls);
                        Ok(Arc::new(DictionaryArray::new(
                            keys,
                            dict_source.values().clone(),
                        )))
                    }
                    other => Err(Error::UnsupportedDictionaryKeyType {
                        expect_oneof: vec![DataType::UInt16],
                        actual: other.clone(),
                    }),
                }
            }

            DataType::Int64 => Ok(Arc::new(Int64Array::new(
                ScalarBuffer::new(self.data.into(), 0, len),
                nulls,
            ))),
            DataType::Float64 => Ok(Arc::new(Float64Array::new(
                ScalarBuffer::new(self.data.into(), 0, len),
                nulls,
            ))),
            DataType::Binary => {
                // safety: if this is the datatype, we'll have initialized offsets in constructor
                let offsets_buffer = self.offsets.expect("offsets not None");
                let offsets_buffer = offsets_buffer.into();
                let offsets = OffsetBuffer::new(ScalarBuffer::new(offsets_buffer, 0, len + 1));
                Ok(Arc::new(BinaryArray::new(offsets, self.data.into(), nulls)))
            }
            DataType::Utf8 => {
                // safety: if this is the datatype, we'll have initialized offsets in constructor
                let offsets_buffer = self.offsets.expect("offsets not None");
                let offsets_buffer = offsets_buffer.into();
                let offsets = OffsetBuffer::new(ScalarBuffer::new(offsets_buffer, 0, len + 1));
                Ok(Arc::new(StringArray::new(offsets, self.data.into(), nulls)))
            }

            DataType::Boolean => Ok(Arc::new(BooleanArray::new(
                // safety: if this is the datatype, we'll have initialized bool_data in constructor
                self.bool_data.expect("bool_data not None").finish(),
                nulls,
            ))),
            _ => {
                // safety: the constructor would not allow this type to be created with
                // a DataType that is not one of the ones that were matched above.
                unreachable!("unknown values data type")
            }
        }
    }
}

/// This helper struct is used for sorting and partitioning segments of the values columns.
///
/// Because multiple ranges of the values array may need to be sorted and partitioned, this struct
/// keeps some internal state to avoid heap allocations for each sequence of attribute values.
struct AttrValuesSorter {
    /// this is the values array, or data extracted from it, used for sorting/partitioning
    inner: AttrsValueSorterInner,

    // The rest of the fields are temporary buffers used when sorting/partitioning to avoid heap
    // allocations on each method invocation
    array_partitions: Vec<Range<usize>>,
    partition_buffer: Vec<u8>,

    // reusable scratch buffers for sorting the keys of u16 coded dictionaries, which are used for
    // str, int, binary and ser columns when the column cardinality allows for it
    rank_sort: Vec<(usize, u16)>,
    key_partition: Vec<u16>,

    // reusable scratch buffers for sorting the float value column.
    float64_sort: Vec<(usize, f64)>,
    float64_partition: Vec<f64>,
}

/// References to the values array data kept for sorting & partitioning. In some cases, the
/// original Arrow array kept. For other cases we keep only some references to the original data
/// which can be handled in a more performant manner.
enum AttrsValueSorterInner {
    /// Inner references to values array dictionary keys, the rank of the keys, and the null buffer.
    /// this allows us to sort many segments of dictionary encoded values columns by the rank of the
    /// dictionary keys, while only having sorted the dictionary values one time.
    KeysAndRanks(AttrValueDictKeysAndRanks),

    // inner reference to values for sorting float64 arrays. We keep the scalar buffer so we can
    // slice it for ranges to sort without creating temporary values.
    Float64((ScalarBuffer<f64>, Option<NullBuffer>)),

    /// an arrow Array containing some segment of the values column
    // TODO - in the future we should pull out some internal state of boolean, int64, string and
    // binary array columns instead of using the ArrayRef to sort. However, these column types
    // would be less common, especially b/c all these types (except bool) would usually be
    // dictionary encoded.
    Array(ArrayRef),
}

/// Dictionary keys and the sorted rank of those keys for some segment of the values column.
struct AttrValueDictKeysAndRanks {
    keys: ScalarBuffer<u16>,
    ranks: Vec<u16>,

    /// validity bitmap for the keys field
    nulls: Option<NullBuffer>,
}

impl TryFrom<&ArrayRef> for AttrsValueSorterInner {
    type Error = Error;

    fn try_from(values_arr: &ArrayRef) -> Result<Self> {
        match values_arr.data_type() {
            DataType::Dictionary(k, _) => match k.as_ref() {
                &DataType::UInt16 => {
                    // safety: we can call expect below because we've checked the datatype just above
                    let dict_arr = values_arr
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can downcast to DictionaryArray<UInt16Type>");

                    let value_ranks = rank(
                        dict_arr.values(),
                        Some(SortOptions {
                            nulls_first: false,
                            ..Default::default()
                        }),
                    )
                    .map_err(|e| {
                        // this would be unusual - someone sent us an invalid batch...
                        Error::UnexpectedRecordBatchState {
                            reason: format!(
                                "found attributes value column that could not be ranked {e:?}"
                            ),
                        }
                    })?;

                    let key_ranks = dict_arr
                        .keys()
                        .values()
                        .iter()
                        .map(|k| value_ranks[*k as usize] as u16)
                        .collect::<Vec<_>>();

                    let rank_nulls = if dict_arr.keys().null_count() > 0 {
                        dict_arr.keys().nulls().cloned()
                    } else {
                        None
                    };

                    Ok(Self::KeysAndRanks(AttrValueDictKeysAndRanks {
                        ranks: key_ranks,
                        nulls: rank_nulls,
                        keys: dict_arr.keys().values().clone(),
                    }))
                }
                other_key_type => Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::UInt16],
                    actual: other_key_type.clone(),
                }),
            },
            DataType::Float64 => {
                // safety: we can expect here because we've checked the data type
                let float_arr = values_arr
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .expect("can downcast to Float64");
                Ok(Self::Float64((
                    float_arr.values().clone(),
                    float_arr.nulls().cloned(),
                )))
            }
            _ => Ok(Self::Array(values_arr.clone())),
        }
    }
}

/// Range returned by the partitioning methods on [`AttrValuesSorter`] which is either all valid
/// or entirely null
struct NullableRange {
    range: Range<usize>,
    is_null: bool,
}

impl AttrValuesSorter {
    fn try_new(values_arr: &ArrayRef) -> Result<Self> {
        Ok(Self {
            inner: values_arr.try_into()?,
            array_partitions: Vec::new(),
            rank_sort: Vec::new(),
            key_partition: Vec::new(),
            partition_buffer: Vec::new(),
            float64_partition: Vec::new(),
            float64_sort: Vec::new(),
        })
    }

    fn sort_and_partition_range(
        &mut self,
        range: &Range<usize>,
        value_col_builder: &mut SortedValuesColumnBuilder,
        key_range_sorted_result: &mut Vec<usize>,
        result: &mut Vec<NullableRange>,
    ) -> Result<()> {
        match &self.inner {
            AttrsValueSorterInner::Float64((floats, nulls)) => {
                let slice = floats.slice(range.start, range.len());
                self.float64_sort.clear();
                self.float64_sort.extend(slice.iter().copied().enumerate());

                if let Some(nulls) = nulls {
                    self.float64_sort.sort_unstable_by(|a, b| {
                        match (
                            nulls.is_null(range.start + a.0),
                            nulls.is_null(range.start + b.0),
                        ) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => a.1.total_cmp(&b.1),
                        }
                    });
                } else {
                    self.float64_sort
                        .sort_unstable_by(|a, b| a.1.total_cmp(&b.1));
                }

                // take the sorted values segment
                // let keys_range = &keys_and_ranks.keys[range.start..range.end];
                self.float64_partition.clear();
                self.float64_partition.reserve(range.len());
                self.float64_partition
                    .extend(self.float64_sort.iter().map(|(i, _)| slice[*i]));
                key_range_sorted_result.extend(self.float64_sort.iter().map(|(idx, _)| *idx));

                // append the sorted values to the builder
                value_col_builder.append_data(&self.float64_partition);
                if let Some(nulls) = &nulls {
                    let null_bits = nulls.inner().slice(range.start, range.len());
                    let sorted_range_null_bits =
                        BooleanBuffer::collect_bool(range.len(), |idx: usize| {
                            null_bits.value(key_range_sorted_result[idx])
                        });
                    value_col_builder.append_nulls(&NullBuffer::new(sorted_range_null_bits));
                } else {
                    value_col_builder.append_n_non_nulls(range.len());
                }

                // populate bitmap where a 1/true bit represents a partition boundary
                // (e.g. an index where one value is not equal to its neighbour)
                let boundaries_len = self.float64_partition.len() - 1;
                let left = &self.float64_partition[0..boundaries_len];
                let right = &self.float64_partition[1..boundaries_len + 1];
                collect_bool_inverted(
                    boundaries_len,
                    |i| left[i].is_eq(right[i]),
                    &mut self.partition_buffer,
                );

                // map the bitmap of partition boundaries to ranges
                let mut set_indices =
                    BitIndexIterator::new(&self.partition_buffer, 0, boundaries_len);
                self.array_partitions.clear();
                collect_partition_boundaries_to_ranges(
                    &mut set_indices,
                    boundaries_len,
                    &mut self.array_partitions,
                );
                result.extend(self.array_partitions.drain(..).map(|range| NullableRange {
                    range,
                    is_null: false,
                }));

                // if there were any nulls in the original values column, fill in any null segment
                // flags in the ranges result
                if let Some(nulls) = &nulls {
                    let nulls = nulls.slice(range.start, range.len());
                    for nullable_range in result {
                        let (start_idx, _) = self.float64_sort[nullable_range.range.start];
                        nullable_range.is_null = nulls.is_null(start_idx)
                    }
                }
            }
            AttrsValueSorterInner::KeysAndRanks(keys_and_ranks) => {
                self.rank_sort.clear();
                self.rank_sort.reserve(range.len());
                self.rank_sort.extend(
                    keys_and_ranks
                        .ranks
                        .iter()
                        .skip(range.start)
                        .take(range.len())
                        .copied()
                        .enumerate(),
                );

                // sort the ranks in the range:
                if let Some(nulls) = &keys_and_ranks.nulls {
                    // comparison may be slightly slower for nulls, but this OK as having null
                    // in the values column would not be a common way in OTAP
                    self.rank_sort.sort_unstable_by(|a, b| {
                        match (
                            nulls.is_null(range.start + a.0),
                            nulls.is_null(range.start + b.0),
                        ) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => a.1.cmp(&b.1),
                        }
                    });
                } else {
                    self.rank_sort.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                }

                // take the sorted values segment
                let keys_range = &keys_and_ranks.keys[range.start..range.end];
                self.key_partition.clear();
                self.key_partition.reserve(keys_range.len());
                self.key_partition
                    .extend(self.rank_sort.iter().map(|(i, _)| keys_range[*i]));
                key_range_sorted_result.extend(self.rank_sort.iter().map(|(idx, _)| *idx));

                // append the sorted values to the builder
                value_col_builder.append_data(&self.key_partition);
                if let Some(nulls) = &keys_and_ranks.nulls {
                    let null_bits = nulls.inner().slice(range.start, range.len());
                    let sorted_range_null_bits =
                        BooleanBuffer::collect_bool(range.len(), |idx: usize| {
                            null_bits.value(key_range_sorted_result[idx])
                        });
                    value_col_builder.append_nulls(&NullBuffer::new(sorted_range_null_bits));
                } else {
                    value_col_builder.append_n_non_nulls(range.len());
                }

                // populate bitmap where a 1/true bit represents a partition boundary
                // (e.g. an index where one value is not equal to its neighbour)
                let boundaries_len = self.key_partition.len() - 1;
                let left = &self.key_partition[0..boundaries_len];
                let right = &self.key_partition[1..boundaries_len + 1];
                collect_bool_inverted(
                    boundaries_len,
                    |i| left[i].is_eq(right[i]),
                    &mut self.partition_buffer,
                );

                // map the bitmap of partition boundaries to ranges
                let mut set_indices =
                    BitIndexIterator::new(&self.partition_buffer, 0, boundaries_len);
                self.array_partitions.clear();
                collect_partition_boundaries_to_ranges(
                    &mut set_indices,
                    boundaries_len,
                    &mut self.array_partitions,
                );
                result.extend(self.array_partitions.drain(..).map(|range| NullableRange {
                    range,
                    is_null: false,
                }));

                // if there were any nulls in the original values column, fill in any null segment/
                // flags in the ranges result
                if let Some(nulls) = &keys_and_ranks.nulls {
                    let nulls = nulls.slice(range.start, range.len());
                    for nullable_range in result {
                        let (start_idx, _) = self.rank_sort[nullable_range.range.start];
                        nullable_range.is_null = nulls.is_null(start_idx)
                    }
                }
            }

            // This is the fallback branch of types for which we haven't yet implemented a more
            // optimal way to sort and partition the range. The reason this is less optimal is
            // because each call to slice, sort, etc. allocate new Arc<dyn Array> which can be
            // expensive if it is happening a lot
            AttrsValueSorterInner::Array(arr) => {
                // slice the array and sort it to indices
                let slice = arr.slice(range.start, range.len());
                let sorted_indices = arrow::compute::sort_to_indices(
                    &slice,
                    Some(SortOptions {
                        nulls_first: false,
                        ..Default::default()
                    }),
                    None,
                )
                .map_err(|e| Error::UnexpectedRecordBatchState {
                    reason: format!(
                        "encountered a type of values column that could not be sorted {e:?}"
                    ),
                })?;

                // populate the list of sorted indices for the key range
                key_range_sorted_result.extend(sorted_indices.values().iter().map(|i| *i as usize));

                // sort the range, and append the sorted values to the column builder:
                // safety: the indices we've passed have been computed from sorting the array we'
                // re currently taking, so it will not be out of bounds
                let values_range_sorted =
                    take(&slice, &sorted_indices, None).expect("indices in bounds");
                match values_range_sorted.data_type() {
                    DataType::Binary => {
                        let binary_arr = values_range_sorted
                            .as_any()
                            .downcast_ref::<BinaryArray>()
                            // safety: we've checked the DataType
                            .expect("can downcast to BinaryArray");
                        value_col_builder.append_data(binary_arr.value_data());
                        value_col_builder
                            .append_offsets(binary_arr.offsets().inner().inner().as_slice());
                        if let Some(nulls) = binary_arr.nulls() {
                            value_col_builder.append_nulls(nulls);
                        } else {
                            value_col_builder.append_n_non_nulls(range.len());
                        }
                    }
                    DataType::Utf8 => {
                        let string_arr = values_range_sorted
                            .as_any()
                            .downcast_ref::<StringArray>()
                            // safety: we've checked the DataType
                            .expect("can downcast to StringArray");
                        value_col_builder.append_data(string_arr.value_data());
                        value_col_builder
                            .append_offsets(string_arr.offsets().inner().inner().as_slice());
                        if let Some(nulls) = string_arr.nulls() {
                            value_col_builder.append_nulls(nulls);
                        } else {
                            value_col_builder.append_n_non_nulls(range.len());
                        }
                    }
                    DataType::Int64 => {
                        let int_arr = values_range_sorted
                            .as_any()
                            .downcast_ref::<Int64Array>()
                            // safety: we've checked the DataType
                            .expect("can downcast to Int64Array");
                        value_col_builder.append_data(int_arr.values().inner().as_slice());
                        if let Some(nulls) = int_arr.nulls() {
                            value_col_builder.append_nulls(nulls);
                        } else {
                            value_col_builder.append_n_non_nulls(range.len());
                        }
                    }
                    DataType::Boolean => {
                        let bool_arr = values_range_sorted
                            .as_any()
                            .downcast_ref::<BooleanArray>()
                            // safety: we've checked the DataType
                            .expect("can downcast to BooleanArray");
                        value_col_builder.append_bools(bool_arr.values());
                        if let Some(nulls) = bool_arr.nulls() {
                            value_col_builder.append_nulls(nulls);
                        } else {
                            value_col_builder.append_n_non_nulls(range.len());
                        }
                    }
                    other_dt => {
                        return Err(Error::UnexpectedRecordBatchState {
                            reason: format!(
                                "encountered unexpected DataType for values column {other_dt:?}"
                            ),
                        });
                    }
                }

                // partition the sorted values and fill in the partition ranges:
                self.array_partitions.clear();
                collect_partition_from_array(&values_range_sorted, &mut self.array_partitions)?;

                result.extend(self.array_partitions.drain(..).map(|range| NullableRange {
                    range,
                    is_null: false,
                }));

                if arr.null_count() > 0 {
                    for nullable_range in result.iter_mut() {
                        if values_range_sorted.is_null(nullable_range.range.start) {
                            nullable_range.is_null = true
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Executes the function f for every index from `0..len` to create a bitmask, which is inverted
/// and then collected into the `result_buf`.
///
/// Generally, this function would be used to create a bitmap of partition boundaries, e.g. where
/// one value is not equal to its neighbour, by passing a function that checks if the values at
/// adjacent indices are equal.
///
/// This code is adapted from arrow's [`arrow::buffer::MutableBuffer::collect_bool`], however this
/// implementation allows reusing the `result_buf`, whereas the arrow version forces us to eventually
/// convert `MutableBuffer` into something that allocates an `Arc` before we can access the bytes.
///
/// Performance: Depending on the function that is passed, this can also be auto-vectorized.
/// For example, a simply comparison function like `left[i].is_eq(right[i])` can be compiled to use
/// SIMD to do the comparisons
fn collect_bool_inverted<F: Fn(usize) -> bool>(len: usize, f: F, result_buf: &mut Vec<u8>) {
    result_buf.clear();
    result_buf.reserve(bit_util::ceil(len, 64) * 8);

    let chunks = len / 64;
    let remainder = len % 64;
    for chunk in 0..chunks {
        let mut packed = 0;
        for bit_idx in 0..64 {
            let i = bit_idx + chunk * 64;
            packed |= (f(i) as u64) << bit_idx;
        }

        result_buf.extend_from_slice((!packed).to_byte_slice());
    }

    if remainder != 0 {
        let mut packed = 0;
        for bit_idx in 0..remainder {
            let i = bit_idx + chunks * 64;
            packed |= (f(i) as u64) << bit_idx;
        }

        result_buf.extend_from_slice((!packed).to_byte_slice());
    }

    result_buf.truncate(bit_util::ceil(len, 8));
}

/// Collect partitions of equivalent values in the given range of the source ID into the passed
/// results Vec. The indices in the result ranges will be relative to the passed range, NOT to the
/// indices of the passed source.
fn collect_partitions_for_range(
    range: &Range<usize>,
    source: &ArrayRef,
    result: &mut Vec<Range<usize>>,
) -> Result<()> {
    if range.len() <= 1 {
        result.push(0..range.len());
        return Ok(());
    }

    match source.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => {
                // safety: we've checked that the dict is this type
                let dict_arr = source
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("can downcast to DictionaryArray<u8>");
                let dict_key_range_bytes =
                    &dict_arr.keys().values().inner().as_slice()[range.start..range.end];

                let len = dict_key_range_bytes.len() - 1;
                let left = &dict_key_range_bytes[0..len];
                let right = &dict_key_range_bytes[1..len + 1];
                let mut partitions_buffer = Vec::new();
                collect_bool_inverted(len, |i| left[i].is_eq(right[i]), &mut partitions_buffer);

                let mut set_indices = BitIndexIterator::new(&partitions_buffer, 0, len);
                collect_partition_boundaries_to_ranges(&mut set_indices, len, result);

                Ok(())
            }
            DataType::UInt16 => {
                // safety: we've checked that the dict is this type
                let dict_arr = source
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("can downcast to DictionaryArray<u16>");
                let dict_key_range_vals = &dict_arr.keys().values()[range.start..range.end];

                let len = dict_key_range_vals.len() - 1;
                let left = &dict_key_range_vals[0..len];
                let right = &dict_key_range_vals[1..len + 1];
                let mut partitions_buffer = Vec::new();
                collect_bool_inverted(len, |i| left[i].is_eq(right[i]), &mut partitions_buffer);

                let mut set_indices = BitIndexIterator::new(&partitions_buffer, 0, len);
                collect_partition_boundaries_to_ranges(&mut set_indices, len, result);

                Ok(())
            }
            other_data_type => Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                actual: other_data_type.clone(),
            }),
        },
        _ => {
            // TODO - in the future we should implement custom behaviour for other array types
            // like what we've done above for dictionary arrays. Currently this creates many
            // new Arc<dyn Array>s, which can have noticeable overhead for small batches.
            let source_range = source.slice(range.start, range.len());
            collect_partition_from_array(&source_range, result)
        }
    }
}

/// Collect partitions of equal values from the passed array. This is very similar to arrow's
/// partition compute kernel, except that it allows reusing the vector of ranges.
///
/// Performance: internally this uses the arrow `eq` compute kernel to compute the partitions
/// which is generally fast from using SIMD. However, this call allocates a handful of temporary
/// Arrays. For large arrays, the performance overhead is usually worth it, but for small arrays
/// (dozens of elements) this does cause extra overhead and using arrow's `partition` kernel
///  may be faster.
fn collect_partition_from_array(source: &ArrayRef, result: &mut Vec<Range<usize>>) -> Result<()> {
    let next_eq_arr_key = create_next_element_equality_array(source)?;
    // safety: `not` is actually infallible, so we can expect here.
    let next_eq_inverted = not(&next_eq_arr_key).expect("can invert boolean array");
    let mut set_indices = next_eq_inverted.values().set_indices();
    collect_partition_boundaries_to_ranges(
        &mut set_indices,
        next_eq_inverted.values().len(),
        result,
    );

    Ok(())
}

/// Given a iterator of positions representing an index which is a boundary between a range
/// of partitions, fill in the result vec with partition ranges.
///
/// This is somewhat similar to what arrow's [`arrow::compute::Partitions::ranges`] method
/// does internally, however it allows reusing the result vec across subsequent calls
fn collect_partition_boundaries_to_ranges<I: Iterator<Item = usize>>(
    boundaries: &mut I,
    len: usize,
    result: &mut Vec<Range<usize>>,
) {
    let mut current = 0;
    for idx in boundaries {
        let t = current;
        current = idx + 1;
        result.push(t..current)
    }
    let last = len + 1;
    if current != last {
        result.push(current..last)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::datatypes::{Field, Schema, UInt32Type};

    #[test]
    fn test_transport_optimize_encode_attrs() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        Some(1),
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["va", "vb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(1),
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from_iter_values([0i64, 1i64])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(2.0),
                    Some(1.0),
                    None,
                    None,
                    Some(1.0),
                ])),
                Arc::new(BooleanArray::from_iter([
                    None,
                    None,
                    Some(true),
                    Some(false),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        None,
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    4, 2, 5, 4, 1, 0, 11, 8, 7, 3, 2, 10,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Bytes as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(0),
                        Some(1),
                        Some(1),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["va", "vb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        Some(1),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from_iter_values([0i64, 1i64])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.0),
                    Some(1.0),
                    Some(2.0),
                    None,
                    None,
                    None,
                ])),
                Arc::new(BooleanArray::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(false),
                    Some(true),
                    None,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_single_element() {
        let columns: Vec<ArrayRef> = vec![
            Arc::new(UInt16Array::from_iter_values([0])),
            Arc::new(UInt8Array::from_iter_values(
                [AttributeValueType::Str as u8],
            )),
            Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values([0]),
                Arc::new(StringArray::from_iter_values(["ka"])),
            )),
            Arc::new(DictionaryArray::new(
                UInt16Array::from_iter([Some(0)]),
                Arc::new(StringArray::from_iter_values(["va"])),
            )),
        ];

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            columns.clone(),
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            columns.clone(),
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_empty_batch() {
        let input = RecordBatch::new_empty(Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
        ])));

        let expected = RecordBatch::new_empty(Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false)
                .with_encoding(consts::metadata::encodings::QUASI_DELTA),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
        ])));

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_sorts_by_parent_id_u16() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    false,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([5, 4, 0, 3, 2, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([1, 1, 1, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 0, 1, 0, 1]),
                    Arc::new(StringArray::from_iter_values(["1", "2"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    false,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([2, 1, 2, 0, 5, 4])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 1, 1, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 1, 0, 0, 1]),
                    Arc::new(StringArray::from_iter_values(["1", "2"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_transport_optimize_encode_attrs_sorts_u32_parent_id() {
        let test_cases: &[(DataType, ArrayRef, ArrayRef)] = &[
            (
                DataType::UInt32,
                Arc::new(UInt32Array::from_iter_values([5, 4, 0, 3, 2, 1])) as ArrayRef,
                Arc::new(UInt32Array::from_iter_values([2, 1, 2, 0, 5, 4])) as ArrayRef,
            ),
            (
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([5, 4, 0, 3, 2, 1]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([2, 1, 2, 0, 5, 4]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
            ),
            (
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([5, 4, 0, 3, 2, 1]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([2, 1, 2, 0, 5, 4]),
                    Arc::new(UInt32Array::from_iter_values([0, 1, 2, 3, 4, 5])),
                )) as ArrayRef,
            ),
        ];

        for (data_type, input_parent_ids, expected_parent_ids) in test_cases {
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(
                        consts::ATTRIBUTE_KEY,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        false,
                    ),
                    Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                        false,
                    ),
                ])),
                vec![
                    input_parent_ids.clone(),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                    ])),
                    Arc::new(DictionaryArray::new(
                        UInt8Array::from_iter_values([1, 1, 1, 0, 0, 0]),
                        Arc::new(StringArray::from_iter_values(["a", "b"])),
                    )),
                    Arc::new(DictionaryArray::new(
                        UInt16Array::from_iter_values([0, 1, 0, 1, 0, 1]),
                        Arc::new(StringArray::from_iter_values(["1", "2"])),
                    )),
                ],
            )
            .unwrap();

            let expected = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, data_type.clone(), false)
                        .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(
                        consts::ATTRIBUTE_KEY,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        false,
                    ),
                    Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                        false,
                    ),
                ])),
                vec![
                    expected_parent_ids.clone(),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                        AttributeValueType::Str as u8,
                    ])),
                    Arc::new(DictionaryArray::new(
                        UInt8Array::from_iter_values([0, 0, 0, 1, 1, 1]),
                        Arc::new(StringArray::from_iter_values(["a", "b"])),
                    )),
                    Arc::new(DictionaryArray::new(
                        UInt16Array::from_iter_values([0, 1, 1, 0, 0, 1]),
                        Arc::new(StringArray::from_iter_values(["1", "2"])),
                    )),
                ],
            )
            .unwrap();

            let result = transport_optimize_encode_attrs::<UInt32Type>(&input).unwrap();
            pretty_assertions::assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_transport_optimize_encode_attrs_with_complex_value_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(
                consts::ATTRIBUTE_SER,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ]));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                // 7 and 6 are in descending here to ensure we don't bother sorting
                // segments that have the same key/value if the type is a complex type
                // b/c we don't need to delta encode it
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 7, 6, 8])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        Some(1),
                        None,
                        None,
                        Some(0),
                    ]),
                    Arc::new(StringArray::from_iter_values(["b", "a"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        Some(1),
                        Some(0),
                        None,
                        Some(2),
                        None,
                        Some(3),
                        Some(3),
                        None,
                    ]),
                    // Note - these aren't valid CBOR serialized attributes, but we're just testing
                    // that they sort in increasing byte order, so it's OK
                    Arc::new(BinaryArray::from_iter_values([b"a", b"b", b"c", b"d"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_SER,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([3, 2, 0, 8, 4, 7, 6, 2, 1])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Slice as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(1),
                        Some(1),
                        Some(0),
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["b", "a"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(2),
                        Some(3),
                        Some(3),
                        Some(0),
                        Some(1),
                    ]),
                    Arc::new(BinaryArray::from_iter_values([b"a", b"b", b"c", b"d"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_attr_empty_attrs() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 6, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 1, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        None,
                        Some(0),
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    Some(2.0),
                    Some(1.9999),
                    None,
                    None,
                ])),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([2, 7, 1, 0, 6, 3, 5, 4])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 1, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        Some(0),
                        Some(0),
                        Some(1),
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.9999),
                    Some(2.0),
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_attr_missing_values_column() {
        // here the `float` column is missing, even though the type column suggests there
        // should be attributes of this type. Ensure we still put these rows in the correct
        // place in the result, and that we don't delta encode the parent_ids which we consider
        // to be null b/c the values column is not present.
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 9, 5, 6, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 1, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        None,
                        None,
                        Some(1),
                        None,
                        None,
                        None,
                        Some(0),
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([2, 7, 1, 0, 6, 3, 4, 9, 5])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 1, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        Some(0),
                        Some(0),
                        Some(1),
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_sorts_by_keys_all_representations() {
        // test all combinations of how dictionary keys can be sorted
        let test_cases = &[
            (
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 2, 3, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 1, 2, 3]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
            ),
            (
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 2, 3, 1]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values([0, 1, 2, 3]),
                    Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
                )) as ArrayRef,
            ),
            (
                DataType::Utf8,
                Arc::new(StringArray::from_iter_values(["a", "c", "d", "b"])),
                Arc::new(StringArray::from_iter_values(["a", "b", "c", "d"])),
            ),
        ];

        for (key_data_type, input_key_col, expected_key_col) in test_cases {
            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt16, false),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_KEY, key_data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                ])),
                vec![
                    Arc::new(UInt16Array::from_iter_values([0, 2, 3, 1])),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                    ])),
                    input_key_col.clone(),
                    Arc::new(BooleanArray::from(vec![true, true, true, true])),
                ],
            )
            .unwrap();

            let expected = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt16, false)
                        .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_KEY, key_data_type.clone(), false),
                    Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                ])),
                vec![
                    Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3])),
                    Arc::new(UInt8Array::from_iter_values([
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                        AttributeValueType::Bool as u8,
                    ])),
                    expected_key_col.clone(),
                    Arc::new(BooleanArray::from(vec![true, true, true, true])),
                ],
            )
            .unwrap();

            let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
            pretty_assertions::assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_for_attr_null_attrs() {
        // create a record batch with some null attrs, both dict encoded and non-dict encoded
        // just to ensure that both are handled correctly.
        // - nulls are sorted last
        // - IDs for null values are not delta-encoded
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    0, 1, 2, 3, 4, 5, 6, 4, 7, 8, 9, 10, 11, 12, 13,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(1),
                        None,
                        None,
                        None, // null str attr (dict encoded)
                        None,
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None,
                        None, // null str attr (dict encoded) with different key
                        None, // null str attr (dict encoded) with different key
                        Some(1),
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    Some(2.0),
                    None, // null float attr (not dict encoded)
                    None,
                    Some(1.5),
                    None,
                    None,
                    None, // null float attr (not dict encoded)
                    None,
                    None,
                    None,
                    None, // null float attr (not dict encoded) with different key
                    None, // null float attr (not dict encoded) with different key
                    Some(2.0),
                ])),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([
                    0, 1, 4, 6, 4, 10, 8, 9, 5, 2, 3, 7, 13, 11, 12,
                ])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1]),
                    Arc::new(StringArray::from_iter_values(["ka", "kb"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(1),
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        Some(1),
                        None, // null str attr (dict encoded) with different key
                        None, // null str attr (dict encoded) with different key
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from_iter_values(["a", "b"])),
                )),
                Arc::new(Float64Array::from_iter([
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(1.5),
                    Some(2.0),
                    None, // null float attr (not dict encoded)
                    None, // null float attr (not dict encoded)
                    Some(2.0),
                    None, // null float attr (not dict encoded) with different key
                    None, // null float attr (not dict encoded) with different key
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_and_apply_transport_delta_encoding_optionally_non_dict_encoded_types() {
        // ensure we handle add the encoding correctly for types that are usually dict encoded,
        // but may be non-dict encoded (for example, if the dict overflowed due to cardinality).
        // This test includes types that aren't already tested elsewhere ..
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([1, 2, 3, 4, 5, 6])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Int as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Int as u8,
                ])),
                Arc::new(StringArray::from_iter_values([
                    "a", "a", "a", "a", "a", "a",
                ])),
                Arc::new(BinaryArray::from_iter([
                    None,
                    Some(b"a"),
                    None,
                    None,
                    Some(b"a"),
                    None,
                ])),
                Arc::new(Int64Array::from_iter([
                    Some(1),
                    None,
                    Some(3),
                    Some(2),
                    None,
                    Some(1),
                ])),
            ],
        )
        .unwrap();

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([1, 5, 4, 3, 2, 3])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Bytes as u8,
                ])),
                Arc::new(StringArray::from_iter_values([
                    "a", "a", "a", "a", "a", "a",
                ])),
                Arc::new(BinaryArray::from_iter([
                    None,
                    None,
                    None,
                    None,
                    Some(b"a"),
                    Some(b"a"),
                ])),
                Arc::new(Int64Array::from_iter([
                    Some(1),
                    Some(1),
                    Some(2),
                    Some(3),
                    None,
                    None,
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }

    #[test]
    fn test_delta_encoding_can_cause_dict_overflow_with_deltas() {
        // craft a sequence of values such that when delta encoding is applied
        // it will overflow dictionary, forcing us to handle this case

        let mut parent_ids = Vec::new();
        let mut parent_id_keys = Vec::new();
        let mut values = Vec::new();
        let mut current_id = 1u32;
        let mut gap = 1;
        for i in 0u8..=255u8 {
            parent_id_keys.push(i);
            parent_ids.push(current_id);
            current_id += gap;
            gap += 1;
            values.push("a".to_string());
        }
        values.push("b".into());
        values.push("b".into());
        values.push("c".into());
        values.push("c".into());
        parent_id_keys.push(0);
        parent_id_keys.push(254);
        parent_id_keys.push(0);
        parent_id_keys.push(255);

        println!("{:?}", parent_ids);

        println!("{:?}", parent_ids[255] - parent_ids[0]);
        println!("{:?}", parent_ids[254] - parent_ids[0]);

        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::PARENT_ID,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    false,
                ),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values(parent_id_keys),
                    Arc::new(UInt32Array::from_iter_values(parent_ids)),
                )),
                Arc::new(UInt8Array::from_iter_values(std::iter::repeat_n(
                    AttributeValueType::Str as u8,
                    260,
                ))),
                Arc::new(StringArray::from_iter_values(std::iter::repeat_n("a", 260))),
                Arc::new(StringArray::from_iter_values(values)),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt32Type>(&input).unwrap();

        let parent_id_column = result.column_by_name(consts::PARENT_ID).unwrap();
        // assert that since the original dict would have overflowed, we handle it by returning
        // a non-dict encoded ID column
        assert!(
            parent_id_column
                .as_any()
                .downcast_ref::<UInt32Array>()
                .is_some()
        )
    }
}
