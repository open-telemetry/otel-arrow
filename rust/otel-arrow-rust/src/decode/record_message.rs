// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use arrow::array::RecordBatch;
use arrow::compute::{sort_to_indices, take_record_batch};
use arrow::datatypes::DataType;

use crate::arrays::get_required_array;
use crate::error::{self, Result};
use crate::otlp::attributes::decoder::materialize_parent_id;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;

/// Wrapper for [RecordBatch].
pub struct RecordMessage {
    #[allow(unused)]
    pub(crate) batch_id: i64,
    #[allow(unused)]
    pub(crate) schema_id: String,
    pub(crate) payload_type: ArrowPayloadType,
    pub(crate) record: RecordBatch,

    sort_order: RecordMessageSort,
}

impl RecordMessage {
    pub fn new(
        batch_id: i64,
        schema_id: String,
        payload_type: ArrowPayloadType,
        record: RecordBatch,
        sort_order: RecordMessageSort,
    ) -> Self {
        // only set the sort order on this if there is a parent ID, otherwise we assume
        // it's the root record which is always sorted by ID?
        // TODO confirm this
        let sort_order = record
            .column_by_name(consts::PARENT_ID)
            .map(|_| sort_order)
            .unwrap_or(RecordMessageSort::Id);

        Self {
            batch_id,
            schema_id,
            payload_type,
            record,
            sort_order,
        }
    }

    pub(crate) fn sort_by_parent_id(&mut self) -> Result<()> {
        if self.sort_order != RecordMessageSort::TransportOptimized {
            // TODO should warn here if it's sorted by ID?
            return Ok(());
        }

        let parent_id_column = get_required_array(&self.record, consts::PARENT_ID)?;
        let record_batch = match parent_id_column.data_type() {
            DataType::UInt16 => materialize_parent_id::<u16>(&self.record),
            DataType::UInt32 => materialize_parent_id::<u32>(&self.record),
            d => error::UnsupportedParentIdTypeSnafu { actual: d.clone() }.fail(),
        }?;

        let parent_id_materialized = get_required_array(&record_batch, consts::PARENT_ID)?;
        // TODO comment about satety here
        let sort_indices = sort_to_indices(&parent_id_materialized, None, None)
            .expect("should be able to sort parent ids");
        // TODO comment about safety here
        self.record = take_record_batch(&record_batch, &sort_indices)
            .expect("should be able to take by sort indices");

        Ok(())
    }
}

#[derive(PartialEq)]
pub enum RecordMessageSort {
    TransportOptimized,
    ParentId,
    Id,
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::arrays::{get_string_array, get_u16_array};
    use crate::otlp::attributes::store::AttributeValueType;
    use arrow::array::{RecordBatch, StringArray, UInt8Array, UInt16Array};
    use arrow::datatypes::{Field, Schema};
    use std::sync::Arc;

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

        let mut message = RecordMessage {
            schema_id: "".to_string(),
            batch_id: 0,
            payload_type: ArrowPayloadType::LogAttrs,
            record: record_batch,
            sort_order: RecordMessageSort::TransportOptimized,
        };

        message.sort_by_parent_id().unwrap();

        let str_result = get_string_array(&message.record, consts::ATTRIBUTE_STR).unwrap();
        let parent_id_result = get_u16_array(&message.record, consts::PARENT_ID).unwrap();

        let expected_parent_ids = UInt16Array::from(vec![0, 1, 2, 2, 2, 5]);
        let expected_strs = StringArray::from(vec!["c", "a", "a", "b", "c", "a"]);

        assert_eq!(str_result, &expected_strs);
        assert_eq!(parent_id_result, &expected_parent_ids);
    }
}
