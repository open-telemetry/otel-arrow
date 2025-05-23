// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::AddAssign;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, PrimitiveArray, PrimitiveBuilder, RecordBatch,
};
use arrow::compute::{sort_to_indices, take_record_batch};
use arrow::datatypes::DataType;
use snafu::OptionExt;

use crate::arrays::get_required_array;
use crate::error::{self, Result};
use crate::otlp::attributes::decoder::materialize_parent_id;
use crate::schema::update_field_metadata;
use crate::schema::{
    consts::{self, metadata},
    update_schema_metadata,
};

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
        DataType::UInt16 => materialize_parent_id::<u16>(record_batch),
        DataType::UInt32 => materialize_parent_id::<u32>(record_batch),
        d => error::UnsupportedParentIdTypeSnafu { actual: d.clone() }.fail(),
    }?;

    let parent_id_materialized = get_required_array(&record_batch, consts::PARENT_ID)?;
    // TODO comment about satety here
    let sort_indices = sort_to_indices(&parent_id_materialized, None, None)
        .expect("should be able to sort parent ids");
    // TODO comment about safety here

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

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{StringArray, UInt8Array, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
    use std::sync::Arc;

    use crate::arrays::{get_string_array, get_u16_array};
    use crate::error::Error;
    use crate::otlp::attributes::store::AttributeValueType;
    use crate::schema::{get_field_metadata, get_schema_metadata};

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
}
