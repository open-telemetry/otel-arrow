// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::RecordBatch;
use arrow::compute::{sort_to_indices, take_record_batch};
use arrow::datatypes::DataType;

use crate::arrays::get_required_array;
use crate::error::{self, Result};
use crate::otlp::attributes::decoder::materialize_parent_id;
use crate::schema::{
    consts::{self, metadata},
    update_schema_metadata,
};

pub fn sort_by_parent_id(record_batch: &RecordBatch) -> Result<RecordBatch> {
    let parent_id_column = record_batch.column_by_name(consts::PARENT_ID);
    if parent_id_column.is_none() {
        // nothing to do?
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

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{StringArray, UInt8Array, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    use crate::arrays::{get_string_array, get_u16_array};
    use crate::otlp::attributes::store::AttributeValueType;

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
    }
}
