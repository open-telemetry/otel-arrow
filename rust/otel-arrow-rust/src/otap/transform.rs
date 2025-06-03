// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::AddAssign;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BooleanArray, DictionaryArray, PrimitiveArray,
    PrimitiveBuilder, RecordBatch,
};
use arrow::compute::{and, partition};
use arrow::compute::{kernels::cmp::eq, sort_to_indices, take_record_batch};
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};
use snafu::{OptionExt, ResultExt};

use crate::arrays::{
    NullableArrayAccessor, get_f64_array, get_f64_array_opt, get_i64_array, get_i64_array_opt,
    get_required_array, get_u8_array,
};
use crate::error::{self, Result};
use crate::otlp::attributes::parent_id;
use crate::otlp::attributes::{parent_id::ParentId, store::AttributeValueType};
use crate::schema::{
    consts::{self, metadata},
    update_schema_metadata,
};
use crate::schema::{get_field_metadata, update_field_metadata};

pub fn sort_by_parent_id(record_batch: &RecordBatch) -> Result<RecordBatch> {
    let parent_id_column = record_batch.column_by_name(consts::PARENT_ID);
    if parent_id_column.is_none() {
        // nothing to do
        return Ok(record_batch.clone());
    }

    let schema = record_batch.schema_ref();
    if schema
        .metadata
        .get(metadata::SORT_COLUMNS)
        .map(|s| s.as_str())
        == Some(consts::PARENT_ID)
    {
        // nothing to do
        return Ok(record_batch.clone());
    }

    let parent_id_column = parent_id_column.expect("column is Some");
    let record_batch = match parent_id_column.data_type() {
        DataType::UInt16 => materialize_parent_id_for_attributes::<u16>(record_batch),
        DataType::UInt32 => materialize_parent_id_for_attributes::<u32>(record_batch),
        d => error::UnsupportedParentIdTypeSnafu { actual: d.clone() }.fail(),
    }?;

    let parent_id_materialized = get_required_array(&record_batch, consts::PARENT_ID)?;

    // safety: this should only fail if we've used some datatype that is not
    // supported by sort_to_indices (like RunEndEncoded), but we've already
    // checked the parent_id datatype just above so we can be sure this is ok
    let sort_indices = sort_to_indices(&parent_id_materialized, None, None)
        .expect("should be able to sort parent ids");

    // safety: take should only panic here if we have passed indices that are out
    // of bounds, or if the indices are not an int type, both of which shouldn't be
    // the case here based on us having used sort_to_indices to create indices
    let sorted_batch = take_record_batch(&record_batch, &sort_indices)
        .expect("should be able to take by sort indices");

    let result = update_schema_metadata(
        sorted_batch,
        metadata::SORT_COLUMNS.to_string(),
        consts::PARENT_ID.to_string(),
    );

    Ok(result)
}

pub fn remove_delta_encoding<T>(
    record_batch: &RecordBatch,
    column_name: &str,
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: AddAssign,
{
    let schema = record_batch.schema_ref();

    let column_index = schema.index_of(column_name);
    if column_index.is_err() {
        // column doesn't exist, nothing to do
        return Ok(record_batch.clone());
    }
    // safety: we've already returned if column_index is an error
    let column_index = column_index.expect("column_index should be Ok");

    // check that the column hasn't already been decoded
    let column_encoding = get_field_metadata(schema, column_name, metadata::COLUMN_ENCODING);
    if let Some(metadata::encodings::PLAIN) = column_encoding {
        // column already decoded, nothing to do
        return Ok(record_batch.clone());
    }

    let column = record_batch.column(column_index);
    let column = column
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .with_context(|| error::ColumnDataTypeMismatchSnafu {
            name: column_name,
            actual: column.data_type().clone(),
            expect: T::DATA_TYPE,
        })?;

    let new_column = Arc::new(remove_delta_encoding_from_column(column));
    let columns = record_batch
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| {
            if i == column_index {
                new_column.clone()
            } else {
                col.clone()
            }
        })
        .collect::<Vec<ArrayRef>>();

    let schema = update_field_metadata(
        schema,
        column_name,
        metadata::COLUMN_ENCODING,
        metadata::encodings::PLAIN,
    );

    // safety: this should only return an error if our schema, or column lengths don't match
    // but based on how we've constructed the batch, this shouldn't happen
    Ok(RecordBatch::try_new(Arc::new(schema), columns)
        .expect("should be able to create record batch"))
}

#[must_use]
pub fn remove_delta_encoding_from_column<T>(array: &PrimitiveArray<T>) -> PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: AddAssign,
{
    let mut result = PrimitiveBuilder::<T>::with_capacity(array.len());
    let mut acc: T::Native = T::Native::default(); // zero

    for i in 0..array.len() {
        if array.is_valid(i) {
            let delta = array.value(i);
            acc += delta;
            result.append_value(acc);
        } else {
            result.append_null();
        }
    }

    result.finish()
}

/// Decodes the parent IDs from their transport optimized encoding to the actual ID values.
///
/// In the transport optimized encoding, the record batch is sorted by value Type, then
/// attribute key, then attribute value. If subsequent rows have the same key & value, the
/// parent IDs are delta encoded.
///
/// There are additional exceptions that end the sequence of delta encoding:
/// - value types Empty, Slice & Map are never considered equal
/// - null values; we don't consider null values equal, even if current row & previous
///   row value are both null. (although in the future, we may want to revisit this
///   see: https://github.com/open-telemetry/otel-arrow/issues/463)
///
/// For example:
///
/// | attr key |  attr val  | parent_id |
/// |----------|------------| --------- |
/// |   "a1"   |  str("a")  |  0        | <-- parent id = 0
/// |   "a1"   |  str("a")  |  1        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 1 = 1
/// |   "a1"   |  str("a")  |  1        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 1 = 2
/// |   "a1"   |  str("a")  |  2        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 2 = 4
/// |   "a1"   |  str("b")  |  0        | <-- val != prev val -> no delta encoding -> parent id = 0
/// |   "a2"   |  str("b")  |  0        | <-- key != prev val -> no delta encoding -> parent id = 0
/// ...
/// |   "a3"   | slice([0]) |  0        | <-- parent id = 0
/// |   "a3"   | slice([0]) |  0        | <-- val type = slice -> no delta encoding -> parent id = 0
/// ...                                       (^ it would be same if value type = Map or Empty)
/// ...
/// |   "a4"   | str(null) |  0        | <-- parent id = 0
/// |   "a4"   | str(null) |  0        | <-- value is null -> no delta encoding even though key + value are same
///
/// This returns a new RecordBatch with the parent_id column replaced with the materialized id.
///
#[allow(unused)] // TODO -- remove allow(unused) when we use this to optimize decoding OTAP
pub fn materialize_parent_id_for_attributes<T>(record_batch: &RecordBatch) -> Result<RecordBatch>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: AddAssign,
{
    // if the batch is empty, just skip all this logic and return a batch
    if record_batch.num_rows() == 0 {
        return Ok(record_batch.clone());
    }

    // check if the column is already decoded
    let column_encoding = get_field_metadata(
        record_batch.schema_ref(),
        consts::PARENT_ID,
        metadata::COLUMN_ENCODING,
    );
    if let Some(metadata::encodings::PLAIN) = column_encoding {
        // column already decoded, nothing to do
        return Ok(record_batch.clone());
    }

    let keys_arr =
        record_batch
            .column_by_name(consts::ATTRIBUTE_KEY)
            .context(error::ColumnNotFoundSnafu {
                name: consts::ATTRIBUTE_KEY,
            })?;
    let key_eq_next = create_next_eq_array_for_array(keys_arr);

    let type_arr = record_batch
        .column_by_name(consts::ATTRIBUTE_TYPE)
        .context(error::ColumnNotFoundSnafu {
            name: consts::ATTRIBUTE_TYPE,
        })?;
    let types_eq_next = create_next_element_equality_array(type_arr)?;
    let type_arr = get_u8_array(record_batch, consts::ATTRIBUTE_TYPE)?;

    let val_str_arr = record_batch.column_by_name(consts::ATTRIBUTE_STR);
    let val_int_arr = record_batch.column_by_name(consts::ATTRIBUTE_INT);
    let val_double_arr = record_batch.column_by_name(consts::ATTRIBUTE_DOUBLE);
    let val_bool_arr = record_batch.column_by_name(consts::ATTRIBUTE_BOOL);
    let val_bytes_arr = record_batch.column_by_name(consts::ATTRIBUTE_BYTES);

    // downcast parent ID into an array of the primitive type
    let parent_id_arr = T::get_parent_id_column(record_batch)?;

    let mut materialized_parent_ids =
        PrimitiveArray::<T::ArrayType>::builder(record_batch.num_rows());

    // below we're iterating through the record batch and each time we find a contiguous range
    // where all the types & attribute keys are the same, we use the "eq" compute kernel to
    // compare all the values. Then we use the resulting next-element equality array for the
    // values to determine if there is delta encoding
    let mut curr_range_start = 0;
    for idx in 0..record_batch.num_rows() {
        // check if we've found the end of a range of where all the type & attribute are the same
        let found_range_end = if idx == types_eq_next.len() {
            true // end of list
        } else {
            !types_eq_next.value(idx) || !key_eq_next.value(idx)
        };

        // when we find the range end, decode the parent ID values
        if found_range_end {
            let value_type = AttributeValueType::try_from(type_arr.value(curr_range_start))
                .context(error::UnrecognizedAttributeValueTypeSnafu)?;
            let value_arr = match value_type {
                AttributeValueType::Str => val_str_arr,
                AttributeValueType::Int => val_int_arr,
                AttributeValueType::Bool => val_bool_arr,
                AttributeValueType::Bytes => val_bytes_arr,
                AttributeValueType::Double => val_double_arr,

                // These types are always considered not equal for purposes of determining
                // whether to delta encode parent ID
                AttributeValueType::Map | AttributeValueType::Slice | AttributeValueType::Empty => {
                    None
                }
            };

            // add the first value from this range to the parent IDs
            let mut curr_parent_id = parent_id_arr
                .value_at(curr_range_start)
                // safety: there's a check at the beginning of this function to ensure that
                // the batch is not empty
                .expect("expect the batch not to be empty");
            materialized_parent_ids.append_value(curr_parent_id);

            if let Some(value_arr) = value_arr {
                // if we have a value array here, we know the parent ID may be delta encoded
                let range_length = idx + 1 - curr_range_start;
                let values_range = value_arr.slice(curr_range_start, range_length);
                let values_eq_next = create_next_element_equality_array(&values_range)?;

                for batch_idx in (curr_range_start + 1)..=idx {
                    let delta_or_parent_id = parent_id_arr.value_at_or_default(batch_idx);
                    let prev_value_range_idx = batch_idx - 1 - curr_range_start;

                    if values_eq_next.value(prev_value_range_idx)
                        && !values_eq_next.is_null(prev_value_range_idx)
                    {
                        // value at current index equals previous so we're delta encoded
                        curr_parent_id += delta_or_parent_id;
                    } else {
                        // value change (or null) breaks the sequence of delta encoding
                        curr_parent_id = delta_or_parent_id;
                    }
                    materialized_parent_ids.append_value(curr_parent_id);
                }
            } else {
                // if we're here, we've determined that the parent ID values are not delta encoded
                // because the type doesn't support it
                for batch_idx in (curr_range_start + 1)..(idx + 1) {
                    materialized_parent_ids
                        .append_value(parent_id_arr.value_at_or_default(batch_idx));
                }
            }

            curr_range_start = idx + 1;
        }
    }
    let materialized_parent_ids = Arc::new(materialized_parent_ids.finish());

    // create new record batch but with parent column replaced
    replace_materialized_parent_id_column(record_batch, materialized_parent_ids)
}

// TODO write tests and rustdocs for this
pub fn materialize_parent_ids_by_columns<'a, T>(
    record_batch: &RecordBatch,
    equality_column_names: impl IntoIterator<Item = &'a str>,
) -> Result<RecordBatch>
where
    T: ParentId,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: AddAssign,
{
    // if the record batch is empty, nothing to decode so return early
    if record_batch.num_rows() == 0 {
        return Ok(record_batch.clone());
    }

    // check that the column hasn't already been decoded
    let column_encoding = get_field_metadata(
        record_batch.schema_ref(),
        consts::PARENT_ID,
        metadata::COLUMN_ENCODING,
    );
    if let Some(metadata::encodings::PLAIN) = column_encoding {
        // column already decoded, nothing to do
        return Ok(record_batch.clone());
    }

    // here we're building up the next-element equality array for multiple columns by 'and'ing
    // the equality array for each column together. This gives us an array that is true if all
    // the values in each column are equal (index offset by 1). If some column doesn't exist, in
    // this case assume this means null values which are equal
    let mut eq_next: Option<BooleanArray> = None;
    for column_name in equality_column_names.into_iter() {
        if let Some(column) = record_batch.column_by_name(column_name) {
            let eq_next_column = create_next_element_equality_array(column)?;
            eq_next = Some(match eq_next {
                Some(eq_next) => and(&eq_next_column, &eq_next)
                    .expect("can 'and' arrays together of same length"),
                None => eq_next_column,
            })
        }
    }

    let eq_next = match eq_next {
        Some(eq_next) => eq_next,
        // all the columns are null values, which means that the parent_id column will just
        // be effectively delta encoded since empty columns are always treated as equal for
        // purposes of checking equality to determine delta encoding sequence ends.
        None => return remove_delta_encoding::<T::ArrayType>(record_batch, consts::PARENT_ID),
    };

    let encoded_parent_ids = T::get_parent_id_column(record_batch)?;
    let mut materialized_parent_ids =
        PrimitiveBuilder::<T::ArrayType>::with_capacity(record_batch.num_rows());
    // safety: there's a check at the beginning of this method that the batch is not empty
    let mut curr_parent_id = encoded_parent_ids
        .value_at(0)
        .expect("expect batch not to be empty");
    materialized_parent_ids.append_value(curr_parent_id);

    for i in 1..record_batch.num_rows() {
        let delta_or_parent_id = encoded_parent_ids.value(i);
        if eq_next.value(i - 1) {
            curr_parent_id += delta_or_parent_id;
        } else {
            curr_parent_id = delta_or_parent_id;
        }
        materialized_parent_ids.append_value(curr_parent_id);
    }

    let materialized_parent_ids = Arc::new(materialized_parent_ids.finish());

    replace_materialized_parent_id_column(record_batch, materialized_parent_ids)
}

// TODO write tests and rustdocs for this
pub fn materialize_parent_id_for_exemplars<T>(record_batch: &RecordBatch) -> Result<RecordBatch>
where
    T: ParentId,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: AddAssign,
{
    materialize_parent_ids_by_columns::<T>(record_batch, [consts::INT_VALUE, consts::DOUBLE_VALUE])
}

fn replace_materialized_parent_id_column(
    record_batch: &RecordBatch,
    materialized_parent_ids: ArrayRef,
) -> Result<RecordBatch> {
    let schema = record_batch.schema();
    let parent_id_idx = schema
        .index_of(consts::PARENT_ID)
        .expect("parent_id should be in the schema");
    let columns = record_batch
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| {
            if i == parent_id_idx {
                materialized_parent_ids.clone()
            } else {
                col.clone()
            }
        })
        .collect::<Vec<ArrayRef>>();

    // update the field metadata for the parent_id column
    let schema = update_field_metadata(
        schema.as_ref(),
        consts::PARENT_ID,
        metadata::COLUMN_ENCODING,
        metadata::encodings::PLAIN,
    );

    Ok(
        RecordBatch::try_new(Arc::new(schema), columns).map_err(|e| {
            error::UnexpectedRecordBatchStateSnafu {
                reason: format!("could not replace parent id {}", e),
            }
            .build()
        })?,
    )
}

// Creates a boolean array where an element having value true means that the
// element at index i of the passed array equals the element at index i + 1.
// Nulls are always treated as not equal
//
// For example,
//    arr = [1, 1, 1, 2, 2,    null, null, 1, 1]
// result = [T, T, F, T, null, null, null, T]
//
pub(crate) fn create_next_element_equality_array(arr: &ArrayRef) -> Result<BooleanArray> {
    // if the array is a dictionary, we compare the dictionary keys
    if let DataType::Dictionary(dict_key_type, _) = arr.data_type() {
        match **dict_key_type {
            DataType::UInt8 => {
                let dict = arr
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("array can be downcast to DictionaryArray<UInt8Type>");

                Ok(create_next_eq_array_for_array(dict.keys()))
            }
            DataType::UInt16 => {
                let dict = arr
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("array can be downcast to DictionaryArray<UInt16Type>");
                Ok(create_next_eq_array_for_array(dict.keys()))
            }
            _ => error::UnsupportedDictionaryKeyTypeSnafu {
                expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                actual: (**dict_key_type).clone(),
            }
            .fail(),
        }
    } else {
        // array is not a dictionary, so just compare the values directly
        Ok(create_next_eq_array_for_array(arr))
    }
}

fn create_next_eq_array_for_array<T: Array>(arr: T) -> BooleanArray {
    // use the arrow compute kernel to compare one
    let lhs = arr.slice(0, arr.len() - 1);
    let rhs = arr.slice(1, arr.len() - 1);
    // safety: `eq` should only be returning an error if the types or lengths don't match
    // which because of what we're passing here, these conditions are satisfied
    eq(&lhs, &rhs).expect("should be able to compare slice with offset of 1")
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{
        BinaryArray, FixedSizeBinaryArray, Float64Array, Int64Array, StringArray, UInt8Array,
        UInt8DictionaryArray, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{
        ArrowDictionaryKeyType, DataType, Field, Schema, UInt8Type, UInt16Type,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::arrays::{get_string_array, get_u16_array, get_u32_array};
    use crate::error::Error;
    use crate::otlp::attributes::store::AttributeValueType;
    use crate::schema::{get_field_metadata, get_schema_metadata};

    #[test]
    fn test_materialize_parent_id_for_attributes_val_change() {
        let test_data = [
            // (key, str_val, int_val, parent_id, expected)
            ("attr1", Some("a"), None, 1, 1), //
            ("attr1", Some("a"), None, 1, 2), // delta = 1
            ("attr1", Some("a"), None, 1, 3), // delta = 1
            ("attr1", Some("b"), None, 1, 1), // not delta (val changed)
            ("attr1", Some("b"), None, 1, 2), // delta = 1
            ("attr2", Some("a"), None, 1, 1), // not delta (key changed)
            ("attr2", Some("a"), None, 1, 2), // delta = 1
            ("attr2", None, Some(1), 1, 1),   // not delta (type changed)
            ("attr2", None, Some(1), 1, 2),   // delta = 1
        ];

        let keys_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
        let parent_ids_arr = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));

        let strs: Vec<Option<&str>> = test_data.iter().map(|a| a.1).collect();
        let string_val_arr = StringArray::from(strs);

        let ints: Vec<Option<i64>> = test_data.iter().map(|a| a.2).collect();
        let int_arr = Int64Array::from(ints);

        let type_arr = UInt8Array::from_iter_values(test_data.iter().map(|a| match (a.1, a.2) {
            (Some(_), _) => AttributeValueType::Str,
            (_, Some(_)) => AttributeValueType::Int,
            _ => unreachable!(),
        } as u8));

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            ])),
            vec![
                Arc::new(parent_ids_arr),
                Arc::new(type_arr),
                Arc::new(keys_arr),
                Arc::new(string_val_arr),
                Arc::new(int_arr),
            ],
        )
        .unwrap();

        let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.4));
        assert_eq!(parent_ids, &expected)
    }

    #[test]
    fn test_materialize_parent_id_for_attributes_with_nulls() {
        let test_data = [
            // (key, string_val, parent id, expected parent id)
            ("attr1", Some("a"), 1, 1),
            ("attr1", Some("a"), 1, 2), // delta = 1
            // when the values are none, the encoder doesn't consider them equal so
            // we shouldn't treat the parent id as delta encoded
            ("attr1", None, 1, 1),
            ("attr1", None, 1, 1),
            ("attr1", None, 1, 1),
            ("attr1", Some("a"), 1, 1),
        ];
        let keys_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
        let parent_ids_arr = UInt16Array::from_iter_values(test_data.iter().map(|a| a.2));
        let string_val_arr = StringArray::from_iter(test_data.iter().map(|a| a.1));
        let type_arr =
            UInt8Array::from_iter_values(test_data.iter().map(|_| AttributeValueType::Str as u8));

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(parent_ids_arr),
                Arc::new(type_arr),
                Arc::new(keys_arr),
                Arc::new(string_val_arr),
            ],
        )
        .unwrap();

        let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
        assert_eq!(parent_ids, &expected)
    }

    #[test]
    fn test_materialize_parent_id_for_attributes_empty() {
        // test this special case of empty batch

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![])),
                Arc::new(UInt8Array::from_iter_values(vec![])),
                Arc::new(StringArray::from(Vec::<Option<&str>>::new())),
                Arc::new(StringArray::from(Vec::<Option<&str>>::new())),
            ],
        )
        .unwrap();

        let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![]);
        assert_eq!(parent_ids, &expected)
    }

    // test that the materialize_parent_id
    #[test]
    fn test_materialize_parent_id_for_attributes_dicts_values() {
        fn run_test_with_dict_key_type<K>()
        where
            K: ArrowDictionaryKeyType,
            K::Native: TryFrom<u8>,
            <<K as ArrowPrimitiveType>::Native as TryFrom<u8>>::Error: std::fmt::Debug,
        {
            let test_data = [
                // (key, dict_key, parent_id, expected)
                ("attr1", 0, 1, 1),
                ("attr1", 0, 1, 2), // delta = 1
                ("attr1", 0, 1, 3), // delta = 1
                ("attr1", 0, 1, 4), // delta = 1
                ("attr1", 0, 1, 5), // delta = 1
                ("attr1", 1, 1, 1), // not delta (val changed)
                ("attr1", 1, 1, 2), // delta = 1
            ];

            let dict_keys = PrimitiveArray::<K>::from_iter_values(
                test_data.iter().map(|a| K::Native::try_from(a.1).unwrap()),
            );
            let dict_values = StringArray::from_iter_values(vec!["a", "b"]);
            let string_val_arr = DictionaryArray::<K>::new(dict_keys, Arc::new(dict_values));

            let keys_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
            let parent_ids_arr = UInt16Array::from_iter_values(test_data.iter().map(|a| a.2));
            let type_arr = UInt8Array::from_iter_values(
                test_data.iter().map(|_| AttributeValueType::Str as u8),
            );

            let record_batch = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt16, false),
                    Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                    Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                    Field::new(
                        consts::ATTRIBUTE_STR,
                        DataType::Dictionary(Box::new(K::DATA_TYPE), Box::new(DataType::Utf8)),
                        true,
                    ),
                ])),
                vec![
                    Arc::new(parent_ids_arr),
                    Arc::new(type_arr),
                    Arc::new(keys_arr),
                    Arc::new(string_val_arr),
                ],
            )
            .unwrap();

            let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
            let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
            let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
            assert_eq!(parent_ids, &expected)
        }

        run_test_with_dict_key_type::<UInt8Type>();
        run_test_with_dict_key_type::<UInt16Type>();
    }

    #[test]
    fn test_materialize_parent_id_for_attributes_other_values_types() {
        let test_data = [
            ("attr1", AttributeValueType::Double, 1, 1),
            ("attr1", AttributeValueType::Double, 1, 2), // delta = 1
            ("attr1", AttributeValueType::Double, 2, 4), // delta = 2
            ("attr1", AttributeValueType::Bool, 1, 1),
            ("attr1", AttributeValueType::Bool, 1, 2), // delta = 1
            ("attr1", AttributeValueType::Bool, 4, 6), // delta = 4
            ("attr1", AttributeValueType::Bytes, 1, 1),
            ("attr1", AttributeValueType::Bytes, 2, 3), // delta = 2
            ("attr1", AttributeValueType::Bytes, 1, 4), // delta = 1
            // for maps, slices and empty the encoder doesn't consider values
            // of these types equal, so the parent ids for these types should
            // never be delta encoded
            ("attr1", AttributeValueType::Map, 1, 1),
            ("attr1", AttributeValueType::Map, 6, 6),
            ("attr1", AttributeValueType::Map, 2, 2),
            ("attr1", AttributeValueType::Slice, 1, 1),
            ("attr1", AttributeValueType::Slice, 6, 6),
            ("attr1", AttributeValueType::Slice, 2, 2),
            ("attr1", AttributeValueType::Empty, 1, 1),
            ("attr1", AttributeValueType::Empty, 6, 6),
            ("attr1", AttributeValueType::Empty, 2, 2),
        ];

        let mut double_builder = Float64Array::builder(test_data.len());
        let mut bool_builder = BooleanArray::builder(test_data.len());
        let mut bytes_builder = Vec::<Option<&[u8]>>::with_capacity(test_data.len());
        let mut ser_builder = Vec::<Option<&[u8]>>::with_capacity(test_data.len());

        for (_, val_type, _, _) in test_data.iter() {
            match val_type {
                AttributeValueType::Bool => {
                    bool_builder.append_value(true);
                    double_builder.append_null();
                    bytes_builder.push(None);
                    ser_builder.push(None);
                }
                AttributeValueType::Double => {
                    bool_builder.append_null();
                    double_builder.append_value(1.0);
                    bytes_builder.push(None);
                    ser_builder.push(None)
                }
                AttributeValueType::Bytes => {
                    bool_builder.append_null();
                    double_builder.append_null();
                    bytes_builder.push(Some(b"0"));
                    ser_builder.push(None)
                }
                AttributeValueType::Slice | AttributeValueType::Map => {
                    bool_builder.append_null();
                    double_builder.append_null();
                    bytes_builder.push(None);
                    ser_builder.push(Some(b"0"));
                }
                AttributeValueType::Empty => {
                    bool_builder.append_null();
                    double_builder.append_null();
                    bytes_builder.push(None);
                    ser_builder.push(None)
                }
                _ => unreachable!(),
            }
        }

        let keys_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
        let parent_ids_arr = UInt16Array::from_iter_values(test_data.iter().map(|a| a.2));
        let type_arr = UInt8Array::from_iter_values(test_data.iter().map(|a| a.1 as u8));
        let bytes_arr = BinaryArray::from_opt_vec(bytes_builder);
        let ser_arr = BinaryArray::from_opt_vec(ser_builder);

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ])),
            vec![
                Arc::new(parent_ids_arr),
                Arc::new(type_arr),
                Arc::new(keys_arr),
                Arc::new(double_builder.finish()),
                Arc::new(bool_builder.finish()),
                Arc::new(bytes_arr),
                Arc::new(ser_arr),
            ],
        )
        .unwrap();

        let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
        assert_eq!(parent_ids, &expected)
    }

    #[test]
    fn test_materialize_parent_id_for_attributes_noop() {
        // check that we don't re-decode the parent ID if the column is already decoded
        let test_data = [
            // (key, str_val, parent_id)
            ("attr1", Some("a"), 1),
            ("attr1", Some("a"), 1),
            ("attr1", Some("a"), 1),
            ("attr1", Some("b"), 1),
            ("attr1", Some("b"), 1),
            ("attr2", Some("a"), 1),
            ("attr2", Some("a"), 1),
            ("attr2", None, 1),
            ("attr2", None, 1),
        ];

        let keys_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
        let parent_ids_before = UInt16Array::from_iter_values(test_data.iter().map(|a| a.2));

        let strs: Vec<Option<&str>> = test_data.iter().map(|a| a.1).collect();
        let string_val_arr = StringArray::from(strs);

        let type_arr =
            UInt8Array::from_iter_values(test_data.iter().map(|_| AttributeValueType::Str as u8));

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false).with_metadata(
                    HashMap::from_iter(vec![(
                        metadata::COLUMN_ENCODING.into(),
                        metadata::encodings::PLAIN.into(),
                    )]),
                ),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(parent_ids_before.clone()),
                Arc::new(type_arr),
                Arc::new(keys_arr),
                Arc::new(string_val_arr),
            ],
        )
        .unwrap();

        let result_batch = materialize_parent_id_for_attributes::<u16>(&record_batch).unwrap();
        let parent_ids_after = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        assert_eq!(parent_ids_after, &parent_ids_before);
    }

    #[test]
    fn test_materialize_parent_id_by_columns() {
        let input = UInt16Array::from_iter_values(vec![1, 1, 1, 1, 1]);
        let column = StringArray::from_iter_values(vec!["a", "a", "b", "b", "c"]);
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::NAME, DataType::Utf8, false),
            ])),
            vec![Arc::new(input), Arc::new(column)],
        )
        .unwrap();
        let result =
            materialize_parent_ids_by_columns::<u16>(&record_batch, [consts::NAME]).unwrap();
        let result_ids = get_u16_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![1, 2, 1, 2, 1]);
        assert_eq!(&expected, result_ids)
    }

    #[test]
    fn test_materialize_parent_id_by_column_noop() {
        // check that we don't re-decode the column if it's already decoded
        let input = UInt16Array::from_iter_values(vec![1, 1, 1, 1, 1]);
        let column = StringArray::from_iter_values(vec!["a", "a", "b", "b", "c"]);
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false).with_metadata(
                    HashMap::from_iter(vec![(
                        metadata::COLUMN_ENCODING.into(),
                        metadata::encodings::PLAIN.into(),
                    )]),
                ),
                Field::new(consts::NAME, DataType::Utf8, false),
            ])),
            vec![Arc::new(input), Arc::new(column)],
        )
        .unwrap();
        let result =
            materialize_parent_ids_by_columns::<u16>(&record_batch, [consts::NAME]).unwrap();
        let result_ids = get_u16_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![1, 1, 1, 1, 1]);
        assert_eq!(&expected, result_ids)
    }

    #[test]
    fn test_materialize_parent_id_by_column_dict_fsb() {
        let input = UInt16Array::from_iter_values(vec![1, 1, 1, 1, 1]);
        let keys = UInt8Array::from_iter_values(vec![0, 0, 1, 1, 1]);
        let values = FixedSizeBinaryArray::try_from_iter(
            vec![
                (0u8..8u8).collect::<Vec<u8>>(),
                (24u8..32u8).collect::<Vec<u8>>(),
            ]
            .into_iter(),
        )
        .unwrap();
        let column = UInt8DictionaryArray::new(keys, Arc::new(values));
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(
                    consts::TRACE_ID,
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(8)),
                    ),
                    true,
                ),
            ])),
            vec![Arc::new(input), Arc::new(column)],
        )
        .unwrap();
        let result =
            materialize_parent_ids_by_columns::<u16>(&record_batch, [consts::TRACE_ID]).unwrap();
        let result_ids = get_u16_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![1, 2, 1, 2, 3]);
        assert_eq!(&expected, result_ids)
    }

    #[test]
    fn test_materialize_parent_id_by_column_empty_batch() {
        // check handling empty batch
        let input = UInt16Array::from_iter_values(vec![]);
        let column = StringArray::from_iter_values(Vec::<String>::new());
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::NAME, DataType::Utf8, false),
            ])),
            vec![Arc::new(input), Arc::new(column)],
        )
        .unwrap();
        let result =
            materialize_parent_ids_by_columns::<u16>(&record_batch, [consts::NAME]).unwrap();
        let result_ids = get_u16_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![]);
        assert_eq!(&expected, result_ids)
    }

    #[test]
    fn test_materialize_parent_id_for_exemplar() {
        let test_data = vec![
            // p.id, int, f64, expected
            (1, Some(1), None, 1),
            (1, Some(1), None, 2),   // delta = 1
            (1, Some(2), None, 1),   // val change breaks delta sequence, parent id = 1
            (1, None, Some(1.0), 1), // val change breaks delta sequence, parent id = 1
            (1, None, Some(1.0), 2), // delta = 1
            (1, None, None, 1),      // value change breaks delta sequence, parent id = 1
            (1, None, None, 2),      // delta = 1 (in this case, empty exemplar value is equal)
        ];

        let parent_ids = UInt32Array::from_iter_values(test_data.iter().map(|d| d.0));
        let int_values = Int64Array::from_iter(test_data.iter().map(|d| d.1));
        let double_values = Float64Array::from_iter(test_data.iter().map(|d| d.2));

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::INT_VALUE, DataType::Int64, true),
                Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(parent_ids),
                Arc::new(int_values),
                Arc::new(double_values),
            ],
        )
        .unwrap();

        let result = materialize_parent_id_for_exemplars::<u32>(&record_batch).unwrap();
        let result_parent_ids = get_u32_array(&result, consts::PARENT_ID).unwrap();

        let expected = UInt32Array::from_iter_values(test_data.iter().map(|d| d.3));
        assert_eq!(result_parent_ids, &expected);
    }

    #[test]
    fn test_materialize_parent_id_for_exemplar_empty_batch() {
        let parent_ids = UInt32Array::from_iter_values(Vec::new());
        let int_values = Int64Array::from_iter(Vec::<i64>::new());
        let double_values = Float64Array::from_iter(Vec::<f64>::new());

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::INT_VALUE, DataType::Int64, true),
                Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
            ])),
            vec![
                Arc::new(parent_ids),
                Arc::new(int_values),
                Arc::new(double_values),
            ],
        )
        .unwrap();

        let result = materialize_parent_id_for_exemplars::<u32>(&record_batch).unwrap();
        let result_parent_ids = get_u32_array(&result, consts::PARENT_ID).unwrap();

        let expected = UInt32Array::from_iter_values(Vec::new());
        assert_eq!(result_parent_ids, &expected);
    }

    #[test]
    fn test_materialize_parent_id_for_exemplar_optional_columns() {
        let test_data = vec![
            // p.id, int, f64
            (1, Some(1), None),
            (1, Some(1), None),
            (1, Some(2), None),
            (1, None, Some(1.0)),
            (1, None, Some(1.0)),
            (1, None, None),
            (1, None, None),
        ];

        let parent_ids = Arc::new(UInt32Array::from_iter_values(test_data.iter().map(|d| d.0)));
        let int_values = Arc::new(Int64Array::from_iter(test_data.iter().map(|d| d.1)));
        let double_values = Arc::new(Float64Array::from_iter(test_data.iter().map(|d| d.2)));

        // expect we handle the case there's no double_values column
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::INT_VALUE, DataType::Int64, true),
            ])),
            vec![parent_ids.clone(), int_values],
        )
        .unwrap();
        let result = materialize_parent_id_for_exemplars::<u32>(&record_batch).unwrap();
        let result_parent_ids = get_u32_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt32Array::from_iter_values(vec![1, 2, 1, 1, 2, 3, 4]);
        assert_eq!(result_parent_ids, &expected);

        // expect we handle the case there's no int_values column
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
            ])),
            vec![parent_ids.clone(), double_values],
        )
        .unwrap();
        let result = materialize_parent_id_for_exemplars::<u32>(&record_batch).unwrap();
        let result_parent_ids = get_u32_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt32Array::from_iter_values(vec![1, 2, 3, 1, 2, 1, 2]);
        assert_eq!(result_parent_ids, &expected);

        // expect we handle case where both types of value columns are missing

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::PARENT_ID,
                DataType::UInt32,
                false,
            )])),
            vec![parent_ids.clone()],
        )
        .unwrap();
        let result = materialize_parent_id_for_exemplars::<u32>(&record_batch).unwrap();
        let result_parent_ids = get_u32_array(&result, consts::PARENT_ID).unwrap();
        let expected = UInt32Array::from_iter_values(vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(result_parent_ids, &expected);
    }

    #[test]
    fn test_sort_by_parent_id() {
        let test_data = [
            ("a", 1), // parent id = 1
            ("a", 1), // delta = 1, parent id = 2
            ("a", 3), // delta = 3, parent id = 5
            ("b", 2), // parent id = 2
            ("c", 0), // parent id = 0
            ("c", 2), // delta = 2, parent id = 2
        ];

        let string_vals = StringArray::from_iter_values(test_data.iter().map(|a| a.0));
        let parent_ids = UInt16Array::from_iter_values(test_data.iter().map(|a| a.1));
        let keys = StringArray::from(vec!["attr1"; test_data.len()]);
        let types = UInt8Array::from(vec![AttributeValueType::Str as u8; test_data.len()]);

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(parent_ids),
                Arc::new(types),
                Arc::new(keys),
                Arc::new(string_vals),
            ],
        )
        .unwrap();

        let result = sort_by_parent_id(&record_batch).unwrap();

        let str_result = get_string_array(&result, consts::ATTRIBUTE_STR).unwrap();
        let parent_id_result = get_u16_array(&result, consts::PARENT_ID).unwrap();

        let expected_parent_ids = UInt16Array::from(vec![0, 1, 2, 2, 2, 5]);
        let expected_strs = StringArray::from(vec!["c", "a", "a", "b", "c", "a"]);

        assert_eq!(str_result, &expected_strs);
        assert_eq!(parent_id_result, &expected_parent_ids);

        // ensure it updated the metadata correctly
        assert_eq!(
            Some("parent_id"),
            get_schema_metadata(result.schema_ref(), metadata::SORT_COLUMNS)
        );
    }

    #[test]
    fn test_remove_delta_encoding() {
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "test",
                DataType::UInt16,
                true,
            )])),
            vec![Arc::new(UInt16Array::from(vec![
                Some(1),
                Some(1),
                None,
                Some(1),
                Some(1),
                None,
            ]))],
        )
        .unwrap();

        let result = remove_delta_encoding::<UInt16Type>(&record_batch, "test").unwrap();

        let transformed_column = result
            .column_by_name("test")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();

        let expected = UInt16Array::from(vec![Some(1), Some(2), None, Some(3), Some(4), None]);
        assert_eq!(transformed_column, &expected);

        // ensure it updates the metadata correctly
        assert_eq!(
            Some(metadata::encodings::PLAIN),
            get_field_metadata(result.schema_ref(), "test", metadata::COLUMN_ENCODING)
        );

        // check it doesn't error if we specify invalid column
        let result = remove_delta_encoding::<UInt16Type>(&record_batch, "badcol").unwrap();
        assert_eq!(result, record_batch);

        // check it returns an error if invoked for the wrong record type
        let result = remove_delta_encoding::<UInt8Type>(&record_batch, "test");
        assert!(matches!(result, Err(Error::ColumnDataTypeMismatch { .. })))
    }

    #[test]
    fn test_remove_delta_encoding_noop() {
        // check we don't remove delta encoding if the column metadata
        // already has encoding=plain
        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("test", DataType::UInt16, true).with_metadata(HashMap::from_iter(vec![
                    (
                        metadata::COLUMN_ENCODING.into(),
                        metadata::encodings::PLAIN.into(),
                    ),
                ])),
            ])),
            vec![Arc::new(UInt16Array::from(vec![
                Some(1),
                Some(1),
                None,
                Some(1),
                Some(1),
                None,
            ]))],
        )
        .unwrap();

        let result = remove_delta_encoding::<UInt16Type>(&record_batch, "test").unwrap();

        let transformed_column = result
            .column_by_name("test")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();

        let expected = UInt16Array::from(vec![Some(1), Some(1), None, Some(1), Some(1), None]);
        assert_eq!(transformed_column, &expected);
    }
}
