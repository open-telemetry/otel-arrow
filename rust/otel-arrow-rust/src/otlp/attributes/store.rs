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
use arrow::array::{
    ArrowPrimitiveType, BinaryArray, BooleanArray, DictionaryArray, Float64Array, PrimitiveArray,
    RecordBatch, UInt8Array, UInt64Array,
};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter;

use super::cbor;

#[derive(Copy, Clone, Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum AttributeValueType {
    Empty = 0,
    Str = 1,
    Int = 2,
    Double = 3,
    Bool = 4,
    Map = 5,
    Slice = 6,
    Bytes = 7,
}

pub type Attribute32Store = AttributeStore<u32>;
pub type Attribute16Store = AttributeStore<u16>;

#[derive(Default)]
pub struct AttributeStore<T> {
    last_id: T,
    attribute_by_ids: HashMap<T, Vec<KeyValue>>,
}

impl<T> AttributeStore<T>
where
    T: ParentId,
{
    pub fn attribute_by_delta_id(&mut self, delta: T) -> Option<&[KeyValue]> {
        self.last_id += delta;
        self.attribute_by_ids
            .get(&self.last_id)
            .map(|r| r.as_slice())
    }

    pub fn attribute_by_id(&self, id: T) -> Option<&[KeyValue]> {
        self.attribute_by_ids.get(&id).map(|r| r.as_slice())
    }
}

impl<T> TryFrom<&RecordBatch> for AttributeStore<T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: Into<T>,
{
    type Error = error::Error;

    fn try_from(rb: &RecordBatch) -> Result<Self, Self::Error> {
        let mut store = Self::default();

        let key_arr = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
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
        let value_ser_arr = rb
            .column_by_name(consts::ATTRIBUTE_SER)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;

        for idx in 0..rb.num_rows() {
            let key = key_arr.value_at_or_default(idx);
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
                AttributeValueType::Slice | AttributeValueType::Map => {
                    let bytes = value_ser_arr.value_at(idx);
                    if bytes.is_none() {
                        continue;
                    }

                    let decoded_result = cbor::decode_pcommon_val(&bytes.expect("expected Some"))?;
                    match decoded_result {
                        Some(value) => value,
                        None => continue,
                    }
                }
                AttributeValueType::Empty => {
                    // should warn here.
                    continue;
                }
            };

            // Parse potentially delta encoded parent id field.
            let parent_id_arr =
                rb.column_by_name(consts::PARENT_ID)
                    .context(error::ColumnNotFoundSnafu {
                        name: consts::PARENT_ID,
                    })?;
            let parent_id_arr =
                MaybeDictArrayAccessor::<PrimitiveArray<<T as ParentId>::ArrayType>>::try_new(
                    parent_id_arr,
                )?;
            let mut parent_id_decoder = T::new_decoder();

            let parent_id = parent_id_decoder.decode(
                parent_id_arr.value_at_or_default(idx).into(),
                &key,
                &value,
            );
            let attributes = store.attribute_by_ids.entry(parent_id).or_default();
            //todo: support assigning ArrayValue and KvListValue by deep copy as in https://github.com/open-telemetry/opentelemetry-collector/blob/fbf6d103eea79e72ff6b2cc3a2a18fc98a836281/pdata/pcommon/value.go#L323
            *attributes.find_or_append(&key) = Some(AnyValue { value: Some(value) });
        }

        Ok(store)
    }
}

trait FindOrAppendValue<V> {
    /// Finds a value with given key and returns the mutable reference to that value.
    /// Appends a new value if not found and return mutable reference to that newly created value.
    fn find_or_append(&mut self, key: &str) -> &mut V;
}

impl FindOrAppendValue<Option<AnyValue>> for Vec<KeyValue> {
    fn find_or_append(&mut self, key: &str) -> &mut Option<AnyValue> {
        // It's a workaround for https://github.com/rust-lang/rust/issues/51545
        if let Some((idx, _)) = self.iter().enumerate().find(|(_, kv)| kv.key == key) {
            return &mut self[idx].value;
        }

        self.push(KeyValue {
            key: key.to_string(),
            value: None,
        });
        &mut self.last_mut().expect("vec is not empty").value
    }
}

#[allow(unused_qualifications)]
pub struct AttributeStoreV2<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
{
    last_id: T,

    curr_idx: usize,
    parent_id_offsets: UInt64Array,

    // TODO fix type names and put in a better order
    parent_ids: MaybeDictArrayAccessor<'a, PrimitiveArray<T::ArrayType>>,
    keys: StringArrayAccessor<'a>,
    value_type_arr: &'a UInt8Array,
    value_str_arr: Option<StringArrayAccessor<'a>>,
    value_int_arr: Option<Int64ArrayAccessor<'a>>,
    value_double_arr: Option<&'a Float64Array>,
    value_bool_arr: Option<&'a BooleanArray>,
    value_bytes_arr: Option<ByteArrayAccessor<'a>>,
    value_ser_arr: Option<ByteArrayAccessor<'a>>,
}

impl<'a, T> AttributeStoreV2<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: Into<T>,
{
    pub fn try_from(rb: &'a RecordBatch) -> error::Result<Self> {
        let key_arr = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .map(StringArrayAccessor::try_new)
            .transpose()?
            .unwrap(); // TODO no unwrap
        let value_type_arr = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;

        let value_str_arr = rb
            .column_by_name(consts::ATTRIBUTE_STR)
            .map(StringArrayAccessor::try_new)
            .transpose()?;
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
        let value_ser_arr = rb
            .column_by_name(consts::ATTRIBUTE_SER)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;

        let parent_id_arr =
            rb.column_by_name(consts::PARENT_ID)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::PARENT_ID,
                })?;
        let parent_id_arr =
            MaybeDictArrayAccessor::<PrimitiveArray<<T as ParentId>::ArrayType>>::try_new(
                parent_id_arr,
            )?;

        let parent_id_eq_next = super::decoder::create_next_element_equality_array(
            rb.column_by_name(consts::PARENT_ID).unwrap(),
        )?;

        let parent_id_offsets = UInt64Array::from_iter_values(
            iter::once(0).chain(
                parent_id_eq_next
                    .into_iter()
                    .enumerate()
                    // TODO safe to unwrap?
                    .filter_map(|(i, val)| {
                        if !val.unwrap() {
                            Some((i + 1) as u64)
                        } else {
                            None
                        }
                    }),
            ),
        );

        Ok(Self {
            last_id: T::default(),
            curr_idx: 0,
            parent_id_offsets,
            keys: key_arr,
            value_type_arr,
            value_str_arr,
            value_int_arr,
            value_double_arr,
            value_bool_arr,
            value_bytes_arr,
            value_ser_arr,
            parent_ids: parent_id_arr,
        })
    }

    pub fn attribute_by_delta_id(&'_ mut self, delta: T) -> Option<AttributeIterator<'_, T>> {
        self.last_id += delta;

        let parent_ids_start = self.parent_id_offsets.value(self.curr_idx) as usize;
        let expected_parent_id = self.parent_ids.value_at(parent_ids_start).unwrap();
        if self.last_id == expected_parent_id.into() {
            self.curr_idx += 1;
            Some(AttributeIterator {
                store: self,
                curr_idx: self.parent_id_offsets.value(self.curr_idx - 1) as usize,
                end_idx: if self.curr_idx < self.parent_id_offsets.len() {
                    self.parent_id_offsets.value(self.curr_idx) as usize
                } else {
                    // iterate to end
                    self.value_type_arr.len()
                },
            })
        } else {
            None
        }
    }
}

pub struct AttributeIterator<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
{
    store: &'a AttributeStoreV2<'a, T>,
    curr_idx: usize,
    end_idx: usize,
}

impl<'a, T> AttributeIterator<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
{
    fn value_type(&self, idx: usize) -> error::Result<AttributeValueType> {
        AttributeValueType::try_from(self.store.value_type_arr.value_at_or_default(idx))
            .context(error::UnrecognizedAttributeValueTypeSnafu)
    }

    fn otel_value(&self, idx: usize, value_type: AttributeValueType) -> Option<Value> {
        // Note: we do not expect any match arm to return None,
        // and we use _or_default() or equivalent.  If any of
        // these evaluate to None, somehow create a warning, as it
        // indicates corrupted data?

        match value_type {
            AttributeValueType::Str => self
                .store
                .value_str_arr
                .value_at(idx)
                .map(Value::StringValue),
            AttributeValueType::Int => self.store.value_int_arr.value_at(idx).map(Value::IntValue),
            AttributeValueType::Double => self
                .store
                .value_double_arr
                .value_at(idx)
                .map(Value::DoubleValue),
            AttributeValueType::Bool => self
                .store
                .value_bool_arr
                .value_at(idx)
                .map(Value::BoolValue),
            AttributeValueType::Bytes => self
                .store
                .value_bytes_arr
                .value_at(idx)
                .map(Value::BytesValue),
            AttributeValueType::Map | AttributeValueType::Slice => self
                .store
                .value_ser_arr
                .value_at(idx)
                .map(|ref bytes| cbor::decode_pcommon_val(&bytes).ok())
                .flatten()
                .flatten(),

            AttributeValueType::Empty => None,
        }
    }
}

impl<'a, T> Iterator for AttributeIterator<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
{
    type Item = KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_idx >= self.end_idx {
            return None;
        }

        let key = self.store.keys.value_at_or_default(self.curr_idx);
        let value_type = self.value_type(self.curr_idx).unwrap();
        let value = self.otel_value(self.curr_idx, value_type);
        self.curr_idx += 1;

        Some(KeyValue {
            key,
            value: Some(AnyValue { value: value }),
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::{
        array::{Int64Array, RecordBatch, StringArray, UInt8Array, UInt16Array},
        datatypes::{DataType, Field, Schema},
    };

    use crate::schema::consts;

    use super::{AttributeStoreV2, AttributeValueType};

    #[test]
    fn test_attribute_store_v2() {
        let test_data = vec![
            (1, "attr1", Some("hello"), None),
            (1, "attr1", Some("hello2"), None),
            (1, "attr2", Some("hello"), None),
            (1, "attr2", Some("hello3"), None),
            (1, "attr3", None, Some(1)),
            (1, "attr4", None, Some(2)),
            (2, "attr1", Some("hello"), None),
            (2, "attr2", Some("hello2"), None),
            (4, "attr1", Some("hello"), None),
            (4, "attr2", Some("hello2"), None),
            (4, "attr3", None, Some(3)),
        ];

        let parent_id = UInt16Array::from_iter_values(test_data.iter().map(|a| a.0));
        let key_arr = StringArray::from_iter_values(test_data.iter().map(|a| a.1));
        let string_arr = StringArray::from_iter(test_data.iter().map(|a| a.2));
        let int_arr = Int64Array::from_iter(test_data.iter().map(|a| a.3));

        let type_arr = UInt8Array::from_iter_values(test_data.iter().map(|a| if a.2.is_some() {
            AttributeValueType::Str
            } else {
                AttributeValueType::Int
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
                Arc::new(parent_id),
                Arc::new(type_arr),
                Arc::new(key_arr),
                Arc::new(string_arr),
                Arc::new(int_arr),
            ],
        )
        .unwrap();

        let mut attr_store = AttributeStoreV2::<u16>::try_from(&record_batch).unwrap();

        let parent_1_attrs = attr_store
            .attribute_by_delta_id(1)
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(6, parent_1_attrs.len());

        // delta encoded parents -- 1 + 1 = 2
        let parent_2_attrs = attr_store
            .attribute_by_delta_id(1)
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(2, parent_2_attrs.len());

        let parent_3_attrs = attr_store.attribute_by_delta_id(1);
        assert!(parent_3_attrs.is_none());

        let parent_4_attrs = attr_store
            .attribute_by_delta_id(1)
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(3, parent_4_attrs.len());
    }
}
