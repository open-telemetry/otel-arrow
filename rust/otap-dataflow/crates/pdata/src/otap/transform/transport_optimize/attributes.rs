// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This code utilities for applying "transport optimized" encoding to Attributes record batches.
//!
//! The goal of the transport optimized encoding optimize the compression ratio of the IPC
//! serialized record batch when general compression is applied.
//!
//! In this encoding, the record batch sorted by the columns:
//! - type
//! - key
//! - value*
//! - parent_id
//! Then delta encoding is applied to segments of the parent_id column
//!
//! Note that the values are contained in multiple columns. For the most part, there is one column
//! per type of attribute, however Map & Slice type attributes are a special case where these two
//! types of attributes share a values column called "ser".
//!
//! In this encoding, the parent_id column is delta-encoded for sequences of rows sharing the same
//! type, key and value, with two exceptions:
//! - parent_ids for Map & Slice types are not delta encoded
//! - attributes where the value is null do not have their parent_id column delta encoded.
//!
//! An row is considered to contain a null attribute if:
//! - the type column = 0, which corresponds to [`AttributeValueType::Empty`]
//! - the value column contains a null
//! - the value column is not present
//!
//! Note that the latter two cases (column contains null, values column missing), would be unusual.
//! Typically to represent a null value, it is best to set `AttributeValueType::Empty` in the type
//! column. Not only does this more clearly express the semantic of no value, but there are the
//! process of adding this encoding is more optimal if there are no nulls in the values column
//! where type != `Empty.
//!
//! Example:
//!
//!  type      | key    | str  | int   | parent_id
//! -----------|---- ---|------|-------|-----------
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 0
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 2 (delta encoded b/c type,key,val are equal to previous row)
//!  1 (str)   | "k1"   | "v1" | null  |  1     <- parent_id = 3
//!  1 (str)   | "k1"   | "v2" | null  |  1     <- parent_id = 1 (value changed, broke delta encoding sequence)
//!  1 (str)   | "k1"   | "v2" | null  |  1     <- parent_id = 2
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 1 (key changed, broke delta encoding sequence)
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 2
//!  1 (str)   | "k2"   | "v2" | null  |  1     <- parent_id = 3
//!  2 (int)   | null   | "v2" | 1     |  1     <- parent_id = 1 (type changed, broke delta encoding sequence)
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
        DictionaryArray, Float64Array, Int64Array, NullBufferBuilder, PrimitiveArray, RecordBatch,
        StringArray, UInt8Array, UInt16Array, UInt32Array,
    },
    buffer::{NullBuffer, ScalarBuffer},
    compute::{SortOptions, cast, concat, not, partition, rank, take},
    datatypes::{DataType, Schema, ToByteSlice, UInt8Type, UInt16Type},
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
/// This function first sorts the types and keys together. Then from this sorted array, it
/// partitions by type to select the correct values column for this sorted segment and subsequently
/// partitions by key within this type range to select segments of the values column for sorting.
/// It then sorts each segment of the values column, partitions this sorted segment, and maybe
/// applies sorting/delta encoding to the parent_ids within each partition (depending on the rules
/// for when to delta encode, which are spelled out above).
///
/// The idea is to sort as efficiently as possible, while collecting enough context along the way
/// that we know: a) when to apply delta encoding to the parent IDs without having to do a second
/// partition pass, and b) how to efficiently reconstruct the sorted values columns at the end
///
pub(crate) fn transport_optimize_encode_attrs<T: ArrowPrimitiveType>(
    record_batch: &RecordBatch,
) -> Result<RecordBatch>
where
    <T as ArrowPrimitiveType>::Native: Ord + Sub<Output = <T as ArrowPrimitiveType>::Native>,
{
    if record_batch.num_rows() <= 1 {
        // sorting & encoding rows would not change the data if there is 1 or fewer rows, so we skip
        // handling the data and just update the schema metadata
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

    // builders for each attribute value column, indexed by AttributeValueType.
    //
    // the final sorted layout for each column will be:
    // - rows grouped by attribute type (matching the type column)
    // - within each type segment, rows sorted by: key, then value
    // - rows outside a column's type segment are set to null
    //
    // example for ATTRIBUTE_STR (type=1):
    //   - Rows where type == 1: non-null, sorted by (key, value)
    //   - Rows where type != 1: null
    let mut sorted_val_columns: [Option<SortedValuesColumnBuilder>; 8] = [
        None, // empty - no values column for empty attrs
        record_batch
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_DOUBLE)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        record_batch
            .column_by_name(consts::ATTRIBUTE_BOOL)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
        // map/slice are special case - see below
        None, // map
        None, // slice
        record_batch
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(|col| SortedValuesColumnBuilder::try_new(col))
            .transpose()?,
    ];

    // ser column is special case because more than one attribute type is stored in this
    // column (both AttributeValueType::Map and AttributeValueType::Slice)
    let mut sorted_ser_column = record_batch
        .column_by_name(consts::ATTRIBUTE_SER)
        .map(|col| SortedValuesColumnBuilder::try_new(col))
        .transpose()?;

    let parent_id_col = get_required_array(record_batch, consts::PARENT_ID)?;

    // if the parent ID column is a dictionary, we cast it to a primitive array so we can work
    // directly with the ScalarBuffer containing the values. We then cast it back to dictionary
    // array when reconstructing the final dataset.
    //
    // TODO investigate if there's a more performant alternative to doing this cast
    let mut parent_dict_key_type = None;
    let parent_id_column_vals = match parent_id_col.data_type() {
        DataType::Dictionary(k, v) => {
            let as_prim = cast(parent_id_col, &T::DATA_TYPE).map_err(|_| {
                // cast would only fail here if the dictionary values were not something that can
                // be cast to <T as ArrowPrimitiveType>::DATA_TYPE (e.g. if the parent_id column
                // was completely the wong type).
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

    // this is the buffer into which we'll be appending sorted + quasi-delta encoded parent_ids
    let mut encoded_parent_id_column = Vec::with_capacity(record_batch.num_rows());

    // first sort by type/key to indices. This will allow us to take new columns for type/key
    // and we'll use these indices later on to take values columns/parent_id column for each
    // partition of rows with equivalent type/value
    let type_col = get_u8_array(record_batch, consts::ATTRIBUTE_TYPE)?;
    let key_col = get_required_array(record_batch, consts::ATTRIBUTE_KEY)?;
    let type_and_key_indices = sort_attrs_type_and_keys_to_indices(type_col, key_col.clone())?;

    // safety: we can call "expect" here because the indices we're taking are computed from the
    // indices of the array we're taking by the sort_attrs_type_and_keys_to_indices function
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

    // safety: partition can only be expected to fail if multiple arrays are passed that have
    // different lengths, or if an array type is passed for which the values cannot be compared.
    // neither of these failing criteria are met for a single uint8 column
    let mut type_partitions = Vec::with_capacity(8);
    collect_partition_from_array(&type_col_sorted, &mut type_partitions)?;

    // These `Vec`s are used farther below when handling the values w/in each range of sorted keys.
    // they're allocated ahead of time here so we can reuse the allocation for each range
    let mut key_ranges = Vec::new();
    let mut values_ranges = Vec::new();
    let mut key_range_indices_values_sorted = Vec::new();

    // iterates of ranges of of value types in ascending order. For example, we iterate over type
    // ranges Empty, String, Int. and so on..
    for type_range in type_partitions {
        // the byte
        let type_range_attr_type = type_col_sorted_bytes[type_range.start];

        let is_ser_col_type = type_range_attr_type == AttributeValueType::Map as u8
            || type_range_attr_type == AttributeValueType::Slice as u8;

        let sorted_val_col_builder = if is_ser_col_type {
            sorted_ser_column.as_mut()
        } else {
            sorted_val_columns[type_range_attr_type as usize].as_mut()
        };

        // this contains indices of rows from the original record batch that are of the type for
        // the range we're currently handling sorted by key. For example, if the current range is
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

            // hint for the number of nulls to the builder
            sorted_val_col.set_null_count_hint(values_type_range_by_key.null_count());

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
                values_sorter
                    .sort_range_to_indices(key_range, &mut key_range_indices_values_sorted)?;

                // map the sorted indices back to original source indices, and add them to the
                // list of indices to take when materializing the sorted values column

                // TODO - internally to `append_indices_to_take`, we actually materialize
                // the values array IF the underlying values array is dictionary encoded.
                // This means that when we eventually construct the values array, we don't
                // actually need to `take` indices from the key buffer. We should find a way
                // to reuse this ...
                sorted_val_col.append_indices_to_take(
                    key_range_indices_values_sorted.iter().map(|&i| {
                        type_range_indices_key_sorted.value(key_range.start + i as usize)
                    }),
                );

                // partition the sorted values column into ranges that all contain the same
                // attribute value and store the results in values_ranges.
                values_ranges.clear();
                values_sorter.take_and_partition_range(
                    key_range,
                    &key_range_indices_values_sorted,
                    &mut values_ranges,
                )?;

                // keep the current length of the encoded parent IDs for later, when we need to
                // iterate over the values ranges to sort and add delta encoding, and this will
                // make it easier to calculate the ranges containing parent IDs for each value
                let values_range_offset = encoded_parent_id_column.len();

                // push the values for parent IDs that have this key into the results column.
                // They'll be inserted in the wrong order, but we're going to sort them afterward
                // for any non-null ranges for type that support quasi-delta encoding
                encoded_parent_id_column.extend(key_range_indices_values_sorted.iter().map(
                    |idx| {
                        let type_range_idx =
                            type_range_indices_key_sorted.value(key_range.start + *idx as usize);
                        parent_id_column_vals[type_range_idx as usize]
                    },
                ));

                for values_range in &values_ranges {
                    let parent_ids_range = &mut encoded_parent_id_column[values_range.range.start
                        + values_range_offset
                        ..values_range.range.end + values_range_offset];

                    // nulls never count as equal for the purposes of delta encoding.
                    // Map & Slice type are never considered "equal" for the purposes of delta
                    // encoding so we skip adding quasi-delta encoding for this range, which means
                    // the parent_id segment also does not need to be sorted
                    if !values_range.is_null
                        && type_range_attr_type != AttributeValueType::Map as u8
                        && type_range_attr_type != AttributeValueType::Slice as u8
                    {
                        sort_and_delta_encode(parent_ids_range);
                    }
                }
            }
        } else {
            // The values column is missing - which either means that the column was all null, or the
            // attribute type is "empty". Either way, we interpret this as "null' attribute, which
            // and nulls are not equal for the purposes of whether we should delta encode the column
            // so we can just append the unsorted parent IDs for the type range
            encoded_parent_id_column.extend(
                type_range_indices_key_sorted
                    .values()
                    .iter()
                    .map(|idx| parent_id_column_vals[*idx as usize]),
            );
        }

        // push the unsorted values for columns not of this type to fill in gaps
        for attr_type in [
            AttributeValueType::Str as u8,
            AttributeValueType::Int as u8,
            AttributeValueType::Double as u8,
            AttributeValueType::Bool as u8,
            AttributeValueType::Bytes as u8,
        ] {
            if attr_type == type_range_attr_type {
                // skip because we already pushed the sorted section for this type
                continue;
            }

            if let Some(sorted_val_col_builder) = sorted_val_columns[attr_type as usize].as_mut() {
                sorted_val_col_builder.append_nulls(type_range.len());
            }
        }

        // push unsorted values from slice/map type if not already pushed. This is handled as
        // special case from the loop above b/c both slice and map attrs types use the same column,
        // and we only want to append segment to the column builder once per value column per type
        if type_range_attr_type != AttributeValueType::Map as u8
            && type_range_attr_type != AttributeValueType::Slice as u8
        {
            if let Some(sorted_val_col_builder) = sorted_ser_column.as_mut() {
                sorted_val_col_builder.append_nulls(type_range.len());
            }
        }
    }

    // finalize the new parent_id column
    let mut parent_id_col = Arc::new(PrimitiveArray::<T>::new(
        ScalarBuffer::from(encoded_parent_id_column),
        None,
    )) as ArrayRef;

    // If the original Parent ID type was a dictionary, cast it back to a dictionary of this type.
    //
    // TODO investigate if there's a more performant alternative than casting back and forth.
    if let Some(dict_key_type) = parent_dict_key_type {
        parent_id_col = match cast(
            &parent_id_col.clone(),
            &DataType::Dictionary(Box::new(dict_key_type), Box::new(T::DATA_TYPE)),
        ) {
            Ok(as_dict) => as_dict,

            // TODO - verify if this could fail? I think conceivably the only reason it could fail
            // is if the dictionary overflowed because we had too many new values after adding
            // delta encoding.
            // TODO - test for this case
            Err(_) => parent_id_col,
        }
    }

    // rebuild the record batch with all the sorted/encoded columns ...
    let mut fields = vec![];
    let mut columns = vec![];
    for field in record_batch.schema().fields() {
        let field_name = field.name();

        if field.name() == consts::PARENT_ID {
            // add encoding the metadata to the parent_id column
            fields.push(
                field
                    .as_ref()
                    .clone()
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
            );
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
                .as_ref()
                .expect("str attr column present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_INT {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Int as usize]
                .as_ref()
                .expect("int attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BOOL {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Bool as usize]
                .as_ref()
                .expect("bool attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_DOUBLE {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Double as usize]
                .as_ref()
                .expect("double attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_BYTES {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_val_columns[AttributeValueType::Bytes as usize]
                .as_ref()
                .expect("bytes attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::ATTRIBUTE_SER {
            // safety: we initialized this element of sorted_val_column only if the column was
            // present in the original record batch. Since the column is present, this is `Some`
            let sorted_col = sorted_ser_column
                .as_ref()
                .expect("serialized attr column is present");
            columns.push(sorted_col.finish()?);
            continue;
        }

        if field_name == consts::PARENT_ID {
            columns.push(parent_id_col.clone());
            continue;
        }

        return Err(Error::UnexpectedRecordBatchState {
            reason: format!("unexpected column {field_name} found in record batch"),
        });
    }

    let schema = Schema::new(fields);
    let batch = RecordBatch::try_new(Arc::new(schema), columns).unwrap();

    Ok(batch)
}

/// Sort the slice in ascending order, and then apply delta encoding
fn sort_and_delta_encode<T: Copy + Ord + Sub<Output = T>>(vals: &mut [T]) {
    vals.sort_unstable();
    let mut prev = vals[0];
    for i in 1..vals.len() {
        let curr = vals[i];
        vals[i] = curr - prev;
        prev = curr;
    }
}

/// sort the type and key and return the indices of the sorted batch.
///
/// To improve sort performance, does something similar to what would be done by arrow's
/// RowConverter, which is to combine the columns into a row based byte array and then sort that.
///
/// However, this implementation is more optimal when keys are dictionary encoded, which they
/// usually will be. Unlike RowConverter, we don't expand the dictionary when creating the sorting
/// target. Instead, if keys is dictionary encoded, we rank the dictionary values, convert the
/// dictionary keys to their ranks, and sort that alongside type. The dictionary keys are normally
/// low cardinality, so ranking is fast.
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
            (other_key, DataType::Utf8) => {
                return Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                    actual: other_key,
                });
            }
            (_, other_value) => {
                return Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::Utf8],
                    actual: other_value,
                });
            }
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
        other_data_type => {
            return Err(Error::UnexpectedRecordBatchState {
                reason: format!(
                    "found invalid type for attributes 'key' column {other_data_type:?}"
                ),
            });
        }
    }
}

/// This is a helper struct for building the values column when converting it to transport a
/// optimized encoding.
///
/// During this encoding process, we sort by type, key, and value. In the attribute's arrow
/// schema, there is one column per type of attribute, but all columns have the same length.
/// Attribute value's column is identified by the value in the type column, and the other columns
/// in the batch will be null at this position.
///
/// This builder helps to track segments of the resulting values array that should either be null,
/// or have sorted values taken from the incoming unencoded source column.
struct SortedValuesColumnBuilder {
    /// the original source column for the sorted value column
    source: Arc<dyn Array>,

    /// segments to take from the source column when constructing the sorted values column.
    /// these segments will either be all null, or identify sorted indices in the values column
    /// to take when [`Self::finish`] is called
    sorted_segments: Vec<SortedValuesColumnSegment>,

    /// hint for the count of nulls in the segment of the record batch containing values of the
    /// type for this column. The column will be nullable, but the segment containing the values
    /// may have no nulls, in which case we can optimize the final construction by ignoring the
    /// null buffer on the source array. In MOST cases, there will be no nulls in this segment
    /// as null attributes should actually in a row with type AttributeValueType::Empty
    null_count_hint: Option<usize>,
}

#[derive(Debug)]
enum SortedValuesColumnSegment {
    Nulls(usize),
    NonNull(Vec<u32>), // indices into the original source array
}

impl SortedValuesColumnBuilder {
    fn try_new(source: &Arc<dyn Array>) -> Result<Self> {
        Ok(Self {
            sorted_segments: Vec::new(),
            source: Arc::clone(source),
            null_count_hint: None,
        })
    }

    /// access some rows of the original values column by their indices
    ///
    /// this should only be called with indices computed from the original source. Calling this
    /// with out of bound indices will fail
    fn take_source(&self, indices: &UInt32Array) -> Result<Arc<dyn Array>> {
        // safety: take will only panic here if the indices are out of bounds, but this is only
        // being called with indices taken from the original record batch, so expect is safe
        Ok(take(self.source.as_ref(), indices, None).expect("indices out of bounds"))
    }

    /// set the hint for the number of nulls in the segment of the attribute's record batch
    /// containing values of the type being constructed by this instance.
    fn set_null_count_hint(&mut self, null_count: usize) {
        self.null_count_hint = Some(null_count);
    }

    /// append a list of sorted indices to the list that will eventually be taken to form
    /// the final sorted values column
    fn append_indices_to_take<I: IntoIterator<Item = u32>>(&mut self, new_indices: I) {
        // extends to the current segment of non-null indices if one is available, otherwise
        // just creates a new segment of non-null indices.
        //
        // It would be typical that we are appending multiple non-null segments in a row, because
        // this will be called from a place where we've sorted by type, and are inserting all the
        // indices for rows having this type. appending to the end of the exiting vec saves us from
        // allocating a new vec for each segment.
        if let Some(SortedValuesColumnSegment::NonNull(non_null_indices)) =
            self.sorted_segments.last_mut()
        {
            non_null_indices.extend(new_indices)
        } else {
            self.sorted_segments
                .push(SortedValuesColumnSegment::NonNull(
                    new_indices.into_iter().collect(),
                ));
        }
    }

    /// append a segment of all null values
    fn append_nulls(&mut self, count: usize) {
        if let Some(SortedValuesColumnSegment::Nulls(null_count)) = self.sorted_segments.last_mut()
        {
            *null_count += count
        } else {
            self.sorted_segments
                .push(SortedValuesColumnSegment::Nulls(count))
        }
    }

    fn finish(&self) -> Result<Arc<dyn Array>> {
        // there should always be at least one segment appended, otherwise it would have meant
        // the batch was empty but we have a check for this at the very start of
        // transport_optimize_encode_attrs
        debug_assert!(self.sorted_segments.len() > 0);

        // debug assert that the segments are actually coalesced by type ...
        // because we're appending to this while iterating over ranges of value rows that all have
        // the same type, and any null segments should be at the end, and because of the way the
        // append_* methods work where they coalesce subsequent segments of the same type, we
        // should have a segment list looks at most like: [Nulls, NonNulls, Nulls] (each item could
        // be missing under certain conditions). If we didn't have the segments coalesced like this
        // we'd do extra work when we construct the final batch.
        debug_assert!(self.sorted_segments.len() <= 3, "segments not coalesced");

        // For dictionary arrays, build directly to avoid concat overhead
        if let DataType::Dictionary(key_type, _) = self.source.data_type() {
            return self.finish_dictionary(key_type.as_ref());
        }

        // Fall back to using the `concat` compute kernel for non dict encoded types.
        // This is less optimal for a couple reasons:
        // - we allocate new all-null arrays for the all null segments
        // - arrow's `concat` concatenates the null buffers of the arrays together, and the code
        //   it uses for this isn't optimal in the case where the are long sequences of all nulls
        //
        // TODO: write optimized method to concat the segments of non-dict encoded sources

        let mut result_segments = Vec::with_capacity(self.sorted_segments.len());
        for segment in &self.sorted_segments {
            match segment {
                SortedValuesColumnSegment::Nulls(count) => {
                    result_segments.push(self.gen_source_nulls(*count)?);
                }
                SortedValuesColumnSegment::NonNull(indices) => {
                    let segment_indices = UInt32Array::from(indices.clone());
                    // safety: take will only fail here if we called it with indices out of bounds,
                    // but all the indices we're using have been computed from the original source
                    // array, so it is safe to expect here
                    let sorted_segment = take(self.source.as_ref(), &segment_indices, None)
                        .expect("indices in bounds");
                    result_segments.push(sorted_segment);
                }
            }
        }

        if result_segments.len() == 1 {
            // don't bother invoking concat, which saves a couple allocations
            // safety: we can call expect here b/c we've checked the vec is non-empty
            return Ok(result_segments
                .into_iter()
                .take(1)
                .next()
                .expect("non empty"));
        }

        let result_segment_refs = result_segments
            .iter()
            .map(|k| k.as_ref())
            .collect::<Vec<_>>();

        // safety: concat will only fail if we are have arrays that are different types, but in
        // this case all our types will be the same so it's OK to expect here
        Ok(concat(result_segment_refs.as_ref()).expect("can concat"))
    }

    // create the final sorted values column for case when the source column is a dictionary.
    fn finish_dictionary(&self, key_type: &DataType) -> Result<Arc<dyn Array>> {
        // Calculate total length
        let total_len: usize = self
            .sorted_segments
            .iter()
            .map(|seg| match seg {
                SortedValuesColumnSegment::Nulls(count) => *count,
                SortedValuesColumnSegment::NonNull(indices) => indices.len(),
            })
            .sum();

        match key_type {
            DataType::UInt16 => {
                // safety: we can call expect here because the caller should have only called this
                // if the self.source is indeed a dictionary array with this key type
                let source_dict = self
                    .source
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("can downcast to DictionaryArray<u16>");
                let source_keys = source_dict.keys().values().iter().as_slice();

                // initiate buffers for building new dictionary keys
                let mut keys_builder = Vec::with_capacity(total_len);
                let mut null_buffer_builder = NullBufferBuilder::new(total_len);

                for segment in &self.sorted_segments {
                    match segment {
                        SortedValuesColumnSegment::Nulls(count) => {
                            keys_builder.resize(keys_builder.len() + count, 0u16);
                            null_buffer_builder.append_n_nulls(*count);
                        }
                        SortedValuesColumnSegment::NonNull(indices) => {
                            // take keys from source dictionary using indices
                            keys_builder
                                .extend(indices.iter().map(|idx| source_keys[*idx as usize]));

                            // check if any of the source positions were null - if not, we can
                            // completely skip handling the null buffer for the source. In most
                            // cases we'd be able to make this skip b/c null attribute values
                            // should actually have type AttributeValuesType::Empty
                            let null_hint_zero = self.null_count_hint == Some(0);
                            if let Some(source_nulls) = source_dict.nulls()
                                && !null_hint_zero
                            {
                                // batch consecutive valid/null runs to reduce append overhead
                                let mut i = 0;
                                while i < indices.len() {
                                    let idx = indices[i] as usize;
                                    let is_valid = source_nulls.is_valid(idx);

                                    // count consecutive runs of same validity
                                    let mut run_len = 1;
                                    while i + run_len < indices.len() {
                                        let next_idx = indices[i + run_len] as usize;
                                        if source_nulls.is_valid(next_idx) == is_valid {
                                            run_len += 1;
                                        } else {
                                            break;
                                        }
                                    }

                                    // append the run
                                    if is_valid {
                                        null_buffer_builder.append_n_non_nulls(run_len);
                                    } else {
                                        null_buffer_builder.append_n_nulls(run_len);
                                    }

                                    i += run_len;
                                }
                            } else {
                                null_buffer_builder.append_n_non_nulls(indices.len());
                            }
                        }
                    }
                }

                let null_buffer = null_buffer_builder.finish();
                let keys_array = UInt16Array::new(ScalarBuffer::from(keys_builder), null_buffer);

                // safety: calling new_unchecked here to avoid having the constructor iterate the
                // keys column to ensure each key has an associated value. We know this to be the
                // case already because we've taken the keys directly from the source array.
                #[allow(unsafe_code)]
                let result = unsafe {
                    DictionaryArray::new_unchecked(keys_array, source_dict.values().clone())
                };

                Ok(Arc::new(result))
            }

            // only u16 dictionary key is supported for attribute values
            key_type => Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof: vec![DataType::UInt16],
                actual: key_type.clone(),
            }),
        }
    }

    fn gen_source_nulls(&self, count: usize) -> Result<Arc<dyn Array>> {
        Ok(match self.source.data_type() {
            DataType::Binary => Arc::new(BinaryArray::new_null(count)),
            DataType::Boolean => Arc::new(BooleanArray::new_null(count)),
            DataType::Int64 => Arc::new(Int64Array::new_null(count)),
            DataType::Float64 => Arc::new(Float64Array::new_null(count)),
            DataType::Utf8 => Arc::new(StringArray::new_null(count)),
            other_data_type => {
                return Err(Error::UnexpectedRecordBatchState {
                    reason: format!(
                        "found unexpected datatype for attribute column {other_data_type:?}"
                    ),
                });
            }
        })
    }
}

/// This helper struct is used for sorting and partitioning segments of the values columns.
///
/// Because multiple ranges of the values array may need to be sorted and partitioned, this struct
/// keeps some internal state to avoid heap allocations for each sequence of attributes.
struct AttrValuesSorter {
    /// this is the values array, or data extracted from it, used for sorting/partitioning
    inner: AttrsValueSorterInner,

    // The rest of temporary buffers used when sorting/partitioning to avoid heap allocations on
    // each method invocation
    array_partitions_scratch: Vec<Range<usize>>,
    rank_sort_scratch: Vec<(usize, u16)>,
    key_partition_scratch: Vec<u16>,
    partition_buffer: Vec<u8>,
}

/// References to the values array data kept for sorting & partitioning. In some cases, the
/// original Arrow array kept. For other cases we keep only some references to the original data
/// which can be handled in a more performant manner. Generally this means not using the arrow
/// compute kernels for sorting & partitioning to avoid the `Arc`s they would force us to allocate.
enum AttrsValueSorterInner {
    /// inner references to values array dictionary keys, the rank of the keys, and the null buffer.
    /// this allows us to sort many segments of dictionary encoded values columns by the rank of the
    /// dictionary keys, while only having sorted the dictionary values one time.
    KeysAndRanks(AttrValueDictKeysAndRanks),

    /// an arrow Array containing some segment of the values column
    Array(ArrayRef),
}

/// Dictionary keys and the sorted rank of those keys for some segment of the values column.
struct AttrValueDictKeysAndRanks {
    keys: Vec<u16>,
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
                        keys: dict_arr.keys().values().to_vec(),
                    }))
                }
                other_key_type => {
                    return Err(Error::UnsupportedDictionaryKeyType {
                        expect_oneof: vec![DataType::UInt16],
                        actual: other_key_type.clone(),
                    });
                }
            },
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
            array_partitions_scratch: Vec::new(),
            rank_sort_scratch: Vec::new(),
            key_partition_scratch: Vec::new(),
            partition_buffer: Vec::new(),
        })
    }

    /// sort the given range of the contained segment of the values column, and collect the sorted
    /// indices into the result vec
    fn sort_range_to_indices(&mut self, range: &Range<usize>, result: &mut Vec<u32>) -> Result<()> {
        match &self.inner {
            AttrsValueSorterInner::KeysAndRanks(ranks) => {
                // take the range from the key ranks, and map to (idx, rank)
                self.rank_sort_scratch.clear();
                self.rank_sort_scratch.reserve(range.len());
                self.rank_sort_scratch.extend(
                    ranks
                        .ranks
                        .iter()
                        .skip(range.start)
                        .take(range.len())
                        .copied()
                        .enumerate(),
                );

                // sort the ranks in the range:
                if let Some(nulls) = &ranks.nulls {
                    // comparison may be slightly slower for nulls, but this OK as having null
                    // in the values column would not be a common way in OTAP
                    self.rank_sort_scratch.sort_by(|a, b| {
                        match (nulls.is_valid(a.0), nulls.is_valid(b.0)) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            (false, false) => a.1.cmp(&b.1),
                        }
                    });
                } else {
                    self.rank_sort_scratch.sort_by(|a, b| a.1.cmp(&b.1));
                }

                // map sorted (idx, rank) to idx
                result.extend(self.rank_sort_scratch.iter().map(|(idx, _)| *idx as u32));
            }

            AttrsValueSorterInner::Array(arr) => {
                // This is the less optimized path, which uses the arrow compute kernels. The
                // reason this is less optimal is that we allocate new Arcs/Arrays and then
                // dispose of them
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

                result.extend(sorted_indices.values());
            }
        }

        Ok(())
    }

    /// Slices the contained values column for the given range, takes the values from the range at
    /// the passed indices and then collects partitions of equivalent values into the results vec.
    ///
    /// For the purposes of computing range nullability, this method assumes that the passed
    /// indices are arranged in such a way that all null values will be grouped into together
    /// into a single partition. (e.g. the indices represent a sorted segment of the values buffer,
    /// and this sorting has been done in such a way that takes nulls into account)
    fn take_and_partition_range(
        &mut self,
        range: &Range<usize>,
        indices: &Vec<u32>,
        result: &mut Vec<NullableRange>,
    ) -> Result<()> {
        match &self.inner {
            AttrsValueSorterInner::KeysAndRanks(keys) => {
                // slice values keys by range, and take rows for the desired indices
                let keys_range = &keys.keys[range.start..range.end];
                self.key_partition_scratch.clear();
                self.key_partition_scratch.reserve(keys_range.len());
                self.key_partition_scratch
                    .extend(indices.iter().map(|i| keys_range[*i as usize]));

                // populate bitmap where a 1/true bit represents a partition boundary
                // (e.g. an index where one value is not equal to its neighbour)
                let boundaries_len = self.key_partition_scratch.len() - 1;
                let left = &self.key_partition_scratch[0..boundaries_len];
                let right = &self.key_partition_scratch[1..boundaries_len + 1];
                collect_bool_inverted(
                    boundaries_len,
                    |i| left[i].is_eq(right[i]),
                    &mut self.partition_buffer,
                );

                // map the bitmap of partition boundaries to ranges
                let mut set_indices =
                    BitIndexIterator::new(&self.partition_buffer, 0, boundaries_len);
                self.array_partitions_scratch.clear();
                collect_partition_boundaries_to_ranges(
                    &mut set_indices,
                    boundaries_len,
                    &mut self.array_partitions_scratch,
                );
                result.extend(
                    self.array_partitions_scratch
                        .drain(..)
                        .map(|range| NullableRange {
                            range,
                            is_null: false,
                        }),
                );

                // TODO -- Should we be coalescing null ranges here?
                // thinking if indices groups nulls into a single partition, but we're partitioning
                // on values, null rows are together but may have different values which means
                // different partitions which is just extra stuff to handle.

                // if there were any nulls in the original values column, fill in any null segments
                // in the ranges result
                if let Some(nulls) = &keys.nulls {
                    for nullable_range in result {
                        nullable_range.is_null =
                            nulls.is_null(indices[nullable_range.range.start] as usize)
                    }
                }
            }

            AttrsValueSorterInner::Array(arr) => {
                // this is a less optimized version of the block above, because it allocates
                // many Arcs & Arrays, but it is a fallback for types for which we haven't yet
                // written an  optimized implementation such as non-dict encoded str/byte and
                // boolean columns
                let indices = UInt32Array::from_iter_values(indices.iter().copied());
                let values_range = arr.slice(range.start, range.len());
                let values_range_sorted = take(&values_range, &indices, None).unwrap();
                self.array_partitions_scratch.clear();
                collect_partition_from_array(
                    &values_range_sorted,
                    &mut self.array_partitions_scratch,
                )?;

                result.extend(
                    self.array_partitions_scratch
                        .drain(..)
                        .map(|range| NullableRange {
                            range,
                            is_null: false,
                        }),
                );

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
/// code allows reusing the `result_buf`, whereas the arrow version forces us to eventually convert
/// `MutableBuffer` into something that allocates an `Arc` before we can access the bytes.
///
/// Depending on the function that is passed, this will also auto-vectorize decently. For example,
/// a simply comparison function like `left[i].is_eq(right[i])` can be compiled to use SIMD to do
/// the comparison
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

/// Collect partitions of equivalent values in the given range of the source ID into the
/// passed results Vec. The indices in the ranges will be relative to the passed range, not
/// to the indices of the passed source.
fn collect_partitions_for_range(
    range: &Range<usize>,
    source: &ArrayRef,
    result: &mut Vec<Range<usize>>,
) -> Result<()> {
    // TODO - if len = 0 or len = 1, just return one range
    // TODO - lots of code duplicated with other methods in this funciton
    // TODO - see if using get_unchecked really makes a difference in perf

    match source.data_type() {
        DataType::Dictionary(k, _) => match **k {
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

                let set_indices = BitIndexIterator::new(&partitions_buffer, 0, len);
                let mut current = 0;
                for idx in set_indices {
                    let t = current;
                    current = idx + 1;
                    result.push(t..current)
                }
                let last = len + 1;
                if current != last {
                    result.push(current..last)
                }

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

                let set_indices = BitIndexIterator::new(&partitions_buffer, 0, len);
                let mut current = 0;
                for idx in set_indices {
                    let t = current;
                    current = idx + 1;
                    result.push(t..current)
                }
                let last = len + 1;
                if current != last {
                    result.push(current..last)
                }

                Ok(())
            }
            _ => {
                todo!()
            }
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
/// Performance: this uses the arrow `eq` compute kernel to compute the partitions which is
/// generally fast from using SIMD. However, this call allocates a handful of temporary Arrays.
/// For large arrays, the performance overhead is usually worth it, but for small arrays
/// (dozens of elements) this does cause extra overhead and using arrow's `partition` kernel
///  may be faster.
fn collect_partition_from_array(source: &ArrayRef, result: &mut Vec<Range<usize>>) -> Result<()> {
    let next_eq_arr_key = create_next_element_equality_array(&source)?;
    // safety: `not` is actually infallible
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
/// This is somewhat similar to what arrow's `partition` kernel does internally, however it
/// allows for reuse of the vec that contains the ranges
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
                Arc::new(UInt16Array::from_iter_values([0, 1, 2, 3, 4, 5, 6, 4, 7])),
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
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
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
                Arc::new(UInt16Array::from_iter_values([0, 1, 4, 6, 4, 5, 2, 3, 7])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Double as u8,
                ])),
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values([0, 0, 0, 0, 0, 0, 0, 0, 0]),
                    Arc::new(StringArray::from_iter_values(["ka"])),
                )),
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter([
                        Some(0),
                        Some(1),
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
                        None, // null str attr (dict encoded)
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
                    Some(1.5),
                    Some(2.0),
                    None, // null float attr (not dict encoded)
                    None, // null float attr (not dict encoded)
                ])),
            ],
        )
        .unwrap();

        let result = transport_optimize_encode_attrs::<UInt16Type>(&input).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }
}
