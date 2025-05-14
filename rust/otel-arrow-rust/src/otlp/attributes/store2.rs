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

use crate::arrays::{
    ByteArrayAccessor, Int64ArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor,
    StringArrayAccessor, get_bool_array_opt, get_f64_array_opt, get_u8_array,
};
use crate::error;
use crate::otlp::attributes::parent_id::ParentId;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use crate::schema::consts;
use arrow::array::{ArrowPrimitiveType, Array, PrimitiveArray, RecordBatch};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use super::store::AttributeValueType;

/// A reference-based attribute store that directly accesses the RecordBatch.
/// This avoids copying data into a HashMap and creating new KeyValue objects.
#[derive(Default)]
pub struct AttributeStore2<T> {
    /// The RecordBatch containing attribute data
    pub record_batch: Option<Arc<RecordBatch>>,
    /// The last ID used in delta ID lookups
    pub last_id: T,
    /// Maps parent_id -> index of first attribute with that parent_id
    pub first_index_by_id: HashMap<T, usize>,
    /// For each row, contains index of the next row with the same parent_id, or None
    pub next_indices: Vec<Option<usize>>,
}

impl<T> AttributeStore2<T>
where
    T: ParentId + Clone,
{
    /// Retrieve attributes by ID with delta encoding (increments last_id by delta).
    /// Returns an iterator over KeyValue pairs with the given parent ID.
    pub fn attributes_by_delta_id(&mut self, delta: T) -> Option<AttributeIterator<T>> {
        self.last_id += delta;
        self.attributes_by_id(&self.last_id)
    }

    /// Retrieve attributes by ID without delta encoding.
    /// Returns an iterator over KeyValue pairs with the given parent ID.
    pub fn attributes_by_id(&self, id: &T) -> Option<AttributeIterator<T>> {
        self.first_index_by_id.get(id).map(|&first_idx| AttributeIterator {
            store: self,
            current_idx: Some(first_idx),
            id: id.clone(),
        })
    }
}

/// Iterator that yields KeyValue pairs from an AttributeStore2.
pub struct AttributeIterator<'a, T> {
    store: &'a AttributeStore2<T>,
    current_idx: Option<usize>,
    id: T,
}

impl<'a, T> Iterator for AttributeIterator<'a, T>
where
    T: ParentId + Clone,
{
    type Item = KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.current_idx?;
        let rb = self.store.record_batch.as_ref()?;
        
        // Update for the next iteration
        self.current_idx = self.store.next_indices.get(idx).and_then(|&next_idx| next_idx);
        
        // Extract key and value from the record batch at the current index
        let key_arr = rb.column_by_name(consts::ATTRIBUTE_KEY)
            .and_then(|arr| StringArrayAccessor::try_new(arr).ok());
        let value_type_arr = get_u8_array(rb, consts::ATTRIBUTE_TYPE).ok()?;
        
        let key = key_arr.and_then(|arr| arr.value_at(idx)).unwrap_or_default();
        let value_type = AttributeValueType::try_from(value_type_arr.value_at_or_default(idx)).ok()?;
        
        let value_str_arr = StringArrayAccessor::try_new_for_column(rb, consts::ATTRIBUTE_STR).ok()?;
        let value_int_arr = rb.column_by_name(consts::ATTRIBUTE_INT)
            .and_then(|arr| Int64ArrayAccessor::try_new(arr).ok());
        let value_double_arr = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE).ok()?;
        let value_bool_arr = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL).ok()?;
        let value_bytes_arr = rb.column_by_name(consts::ATTRIBUTE_BYTES)
            .and_then(|arr| ByteArrayAccessor::try_new(arr).ok());

        let value = match value_type {
            AttributeValueType::Str => {
                Value::StringValue(value_str_arr.value_at(idx).unwrap_or_default())
            }
            AttributeValueType::Int => {
                Value::IntValue(value_int_arr.and_then(|arr| arr.value_at_or_default(idx)).unwrap_or_default())
            }
            AttributeValueType::Double => {
                Value::DoubleValue(value_double_arr.value_at_or_default(idx))
            }
            AttributeValueType::Bool => {
                Value::BoolValue(value_bool_arr.value_at_or_default(idx))
            }
            AttributeValueType::Bytes => {
                Value::BytesValue(value_bytes_arr.and_then(|arr| arr.value_at_or_default(idx)).unwrap_or_default())
            }
            AttributeValueType::Slice | AttributeValueType::Map | AttributeValueType::Empty => {
                // Skip unsupported or empty attributes
                return self.next();
            }
        };

        Some(KeyValue {
            key,
            value: Some(AnyValue { value: Some(value) }),
        })
    }
}

impl<T> TryFrom<&RecordBatch> for AttributeStore2<T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: Into<T>,
{
    type Error = error::Error;

    fn try_from(rb: &RecordBatch) -> Result<Self, Self::Error> {
        let num_rows = rb.num_rows();
        let mut store = Self::default();
        
        if num_rows == 0 {
            // Return empty store for empty record batch
            return Ok(store);
        }
        
        // Create a clone of the record batch to store in the AttributeStore2
        store.record_batch = Some(Arc::new(rb.clone()));
        store.next_indices = vec![None; num_rows];
        
        // Get parent_id array
        let parent_id_arr = rb
            .column_by_name(consts::PARENT_ID)
            .context(error::ColumnNotFoundSnafu {
                name: consts::PARENT_ID,
            })?;
            
        let parent_id_arr =
            MaybeDictArrayAccessor::<PrimitiveArray<<T as ParentId>::ArrayType>>::try_new(
                parent_id_arr,
            )?;
            
        // Keep track of parent IDs for setting up linked lists
        let mut parent_id_decoder = T::new_decoder();
        
        // Get key and value arrays for decoding parent IDs
        let key_arr = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .map(StringArrayAccessor::try_new)
            .transpose()?
            .ok_or(error::ColumnNotFoundSnafu {
                name: consts::ATTRIBUTE_KEY,
            })?;
            
        let value_type_arr = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;
        let value_str_arr = StringArrayAccessor::try_new_for_column(rb, consts::ATTRIBUTE_STR)?;
        let value_int_arr = rb
            .column_by_name(consts::ATTRIBUTE_INT)
            .map(Int64ArrayAccessor::try_new)
            .transpose()?;
        let value_double_arr = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?;
        let value_bool_arr = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?;
        let value_bytes_arr = rb
            .column_by_name(consts::ATTRIBUTE_BYTES)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;

        // Last seen index for each parent_id
        let mut last_idx_by_id: HashMap<T, usize> = HashMap::new();

        // Build linked lists of attributes with the same parent_id
        for idx in 0..num_rows {
            let key = key_arr.value_at_or_default(idx);
            
            // Process value to get parent ID
            let value_type = AttributeValueType::try_from(value_type_arr.value_at_or_default(idx))
                .context(error::UnrecognizedAttributeValueTypeSnafu)?;
                
            let value = match value_type {
                AttributeValueType::Str => {
                    Value::StringValue(value_str_arr.value_at(idx).unwrap_or_default())
                }
                AttributeValueType::Int => Value::IntValue(value_int_arr.value_at_or_default(idx)),
                AttributeValueType::Double => {
                    Value::DoubleValue(value_double_arr.value_at_or_default(idx))
                }
                AttributeValueType::Bool => {
                    Value::BoolValue(value_bool_arr.value_at_or_default(idx))
                }
                AttributeValueType::Bytes => {
                    Value::BytesValue(value_bytes_arr.value_at_or_default(idx))
                }
                AttributeValueType::Slice | AttributeValueType::Map | AttributeValueType::Empty => {
                    // Skip unsupported or empty attributes
                    continue;
                }
            };
            
            // Decode parent ID
            let parent_id = parent_id_decoder.decode(
                parent_id_arr.value_at_or_default(idx).into(),
                &key,
                &value,
            );
            
            // Update first_index_by_id if this is the first time we see this parent_id
            if !store.first_index_by_id.contains_key(&parent_id) {
                store.first_index_by_id.insert(parent_id.clone(), idx);
            }
            
            // Update the next_indices for the previous entry with the same parent_id
            if let Some(&prev_idx) = last_idx_by_id.get(&parent_id) {
                store.next_indices[prev_idx] = Some(idx);
            }
            
            // Remember this as the last seen index for this parent_id
            last_idx_by_id.insert(parent_id, idx);
        }
        
        Ok(store)
    }
}

pub type Attribute32Store2 = AttributeStore2<u32>;
pub type Attribute16Store2 = AttributeStore2<u16>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::create_array;
    use arrow::array::{StringArray, UInt8Array, Int64Array, Float64Array, BooleanArray, BinaryArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    
    fn create_test_batch() -> RecordBatch {
        // Create a test record batch with attribute data
        let schema = Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt32, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
            Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
            Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
            Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        ]);
        
        // 2 attributes with parent_id=1, 3 attributes with parent_id=2, etc.
        let parent_id = Arc::new(PrimitiveArray::<arrow::datatypes::UInt32Type>::from(vec![
            1, 1, 2, 2, 2, 3, 3, 4
        ]));
        
        let keys = Arc::new(StringArray::from(vec![
            "key1", "key2", "key1", "key2", "key3", "key1", "key2", "key1"
        ]));
        
        // 1=string, 2=int, 3=double, 4=bool
        let types = Arc::new(UInt8Array::from(vec![
            1, 2, 1, 3, 4, 2, 1, 1
        ]));
        
        let string_values = Arc::new(StringArray::from(vec![
            Some("value1"), None, Some("value3"), None, None, None, Some("value7"), Some("value8")
        ]));
        
        let int_values = Arc::new(Int64Array::from(vec![
            None, Some(42), None, None, None, Some(100), None, None
        ]));
        
        let double_values = Arc::new(Float64Array::from(vec![
            None, None, None, Some(3.14), None, None, None, None
        ]));
        
        let bool_values = Arc::new(BooleanArray::from(vec![
            None, None, None, None, Some(true), None, None, None
        ]));
        
        let bytes_values = Arc::new(BinaryArray::from_opt_vec(vec![
            None, None, None, None, None, None, None, None
        ]));
        
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                parent_id, 
                keys, 
                types, 
                string_values, 
                int_values,
                double_values,
                bool_values,
                bytes_values,
            ],
        ).unwrap()
    }
    
    #[test]
    fn test_attribute_store2_creation() {
        let rb = create_test_batch();
        let store = AttributeStore2::<u32>::try_from(&rb).unwrap();
        
        // Check first indices
        assert_eq!(store.first_index_by_id.get(&1), Some(&0));
        assert_eq!(store.first_index_by_id.get(&2), Some(&2));
        assert_eq!(store.first_index_by_id.get(&3), Some(&5));
        assert_eq!(store.first_index_by_id.get(&4), Some(&7));
        
        // Check linked list structure
        assert_eq!(store.next_indices[0], Some(1));
        assert_eq!(store.next_indices[1], None);
        assert_eq!(store.next_indices[2], Some(3));
        assert_eq!(store.next_indices[3], Some(4));
        assert_eq!(store.next_indices[4], None);
        assert_eq!(store.next_indices[5], Some(6));
        assert_eq!(store.next_indices[6], None);
        assert_eq!(store.next_indices[7], None);
    }
    
    #[test]
    fn test_attribute_iterator() {
        let rb = create_test_batch();
        let store = AttributeStore2::<u32>::try_from(&rb).unwrap();
        
        // Test parent_id 1
        let attr_iter = store.attributes_by_id(&1).unwrap();
        let attrs: Vec<_> = attr_iter.collect();
        
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key, "key1");
        assert_eq!(attrs[1].key, "key2");
        
        assert_eq!(
            attrs[0].value.as_ref().and_then(|av| av.value.as_ref()).unwrap(),
            &Value::StringValue("value1".to_string())
        );
        assert_eq!(
            attrs[1].value.as_ref().and_then(|av| av.value.as_ref()).unwrap(),
            &Value::IntValue(42)
        );
        
        // Test parent_id 2
        let attr_iter = store.attributes_by_id(&2).unwrap();
        let attrs: Vec<_> = attr_iter.collect();
        
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs[0].key, "key1");
        assert_eq!(attrs[1].key, "key2");
        assert_eq!(attrs[2].key, "key3");
        
        assert_eq!(
            attrs[0].value.as_ref().and_then(|av| av.value.as_ref()).unwrap(),
            &Value::StringValue("value3".to_string())
        );
        assert_eq!(
            attrs[1].value.as_ref().and_then(|av| av.value.as_ref()).unwrap(),
            &Value::DoubleValue(3.14)
        );
        assert_eq!(
            attrs[2].value.as_ref().and_then(|av| av.value.as_ref()).unwrap(),
            &Value::BoolValue(true)
        );
    }
    
    #[test]
    fn test_delta_id() {
        let rb = create_test_batch();
        let mut store = AttributeStore2::<u32>::try_from(&rb).unwrap();
        
        // Check delta ID functionality
        let attrs1: Vec<_> = store.attributes_by_delta_id(1).unwrap().collect();
        assert_eq!(attrs1.len(), 2);
        assert_eq!(attrs1[0].key, "key1");
        
        let attrs2: Vec<_> = store.attributes_by_delta_id(1).unwrap().collect();
        assert_eq!(attrs2.len(), 3);
        assert_eq!(attrs2[0].key, "key1");
        assert_eq!(attrs2[1].key, "key2");
        assert_eq!(attrs2[2].key, "key3");
    }
}
