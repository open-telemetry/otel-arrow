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
    RecordBatch, UInt8Array,
};
use num_enum::TryFromPrimitive;
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::hash::Hash;

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
    // <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: Into<T>,
{
    last_id: T,

    // TODO, not sure if this is the right datastructure here,
    // and if it is, we probably want to fix the qualifications
    first_parent_id_offsets:
        HashMap<<<T as ParentId>::ArrayType as arrow::array::ArrowPrimitiveType>::Native, usize>,

    // TODO fix type names and put in a better order
    parent_ids: MaybeDictArrayAccessor<'a, PrimitiveArray<T::ArrayType>>,
    keys: StringArrayAccessor<'a>,
    value_type_arr: &'a UInt8Array,
    value_str_arr: Option<StringArrayAccessor<'a>>,
    value_int_arr: Option<Int64ArrayAccessor<'a>>,
    value_double_arr: Option<&'a Float64Array>,
    value_bool_arr: Option<&'a BooleanArray>,
    // TODO other types
}

#[allow(unused_qualifications)]
impl<'a, T> AttributeStoreV2<'a, T>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: std::cmp::Eq + Hash + From<T>,
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

        let parent_id_arr =
            rb.column_by_name(consts::PARENT_ID)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::PARENT_ID,
                })?;
        let parent_id_arr =
            MaybeDictArrayAccessor::<PrimitiveArray<<T as ParentId>::ArrayType>>::try_new(
                parent_id_arr,
            )?;

        // TODO this probabaly isn't how we wanna iterate this
        let arr = match parent_id_arr {
            MaybeDictArrayAccessor::Native(arr) => arr,
            // MaybeDictArrayAccessor::Dictionary16(arr) => arr,
            _ => {
                todo!()
            }
        };
        let mut p_id_offsets = HashMap::new();
        for (i, val) in arr.iter().enumerate() {
            let val = val.unwrap();
            if !p_id_offsets.contains_key(&val) {
                _ = p_id_offsets.insert(val, i);
            }
        }

        Ok(Self {
            last_id: T::default(),
            keys: key_arr,
            value_type_arr,
            value_str_arr,
            value_int_arr,
            value_double_arr,
            value_bool_arr,
            parent_ids: parent_id_arr,
            first_parent_id_offsets: p_id_offsets,
        })
    }

    pub fn attribute_by_delta_id(&mut self, delta: T) -> Option<Vec<KeyValue>> {
        self.last_id += delta;
        // self.attribute_by_ids
        //     .get(&self.last_id)
        //     .map(|r| r.as_slice())
        self.attribute_by_id(self.last_id)
    }

    pub fn attribute_by_id(&self, id: T) -> Option<Vec<KeyValue>> {
        let idx = self.first_parent_id_offsets.get(&id.into());
        if idx.is_none() {
            return None;
        }

        let mut idx = *idx.unwrap();

        let parent_id = self.parent_ids.value_at(idx);
        let mut result = vec![];
        loop {
            if self.parent_ids.value_at(idx) != parent_id {
                break;
            }

            let value_type =
                AttributeValueType::try_from(self.value_type_arr.value_at_or_default(idx))
                    .context(error::UnrecognizedAttributeValueTypeSnafu)
                    .unwrap();

            let value = match value_type {
                AttributeValueType::Str => {
                    Value::StringValue(self.value_str_arr.value_at(idx).unwrap_or_default())
                }
                AttributeValueType::Int => {
                    Value::IntValue(self.value_int_arr.value_at_or_default(idx))
                }
                AttributeValueType::Double => {
                    Value::DoubleValue(self.value_double_arr.value_at_or_default(idx))
                }
                AttributeValueType::Bool => {
                    Value::BoolValue(self.value_bool_arr.value_at_or_default(idx))
                }
                _ => {
                    todo!()
                } // AttributeValueType::Bytes => {
                  //     Value::BytesValue(value_bytes_arr.value_at_or_default(idx))
                  // }
                  // AttributeValueType::Slice | AttributeValueType::Map => {
                  //     let bytes = value_ser_arr.value_at(idx);
                  //     if bytes.is_none() {
                  //         continue;
                  //     }

                  //     let decoded_result = cbor::decode_pcommon_val(&bytes.expect("expected Some"))?;
                  //     match decoded_result {
                  //         Some(value) => value,
                  //         None => continue,
                  //     }
                  // }
                  // AttributeValueType::Empty => {
                  //     // should warn here.
                  //     continue;
                  // }
            };
            result.push(KeyValue {
                key: self.keys.value_at(idx).unwrap(),
                value: Some(AnyValue { value: Some(value) }),
            })
        }

        Some(result)
    }
}
