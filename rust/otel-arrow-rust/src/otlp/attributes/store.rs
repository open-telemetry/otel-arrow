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
    get_binary_array_opt, get_bool_array_opt, get_f64_array_opt, get_i64_array_opt, get_u8_array,
    NullableArrayAccessor, StringArrayAccessor,
};
use crate::error;
use crate::otlp::attributes::parent_id::ParentId;
use crate::schema::consts;
use arrow::array::{Array, RecordBatch};
use arrow::datatypes::Schema;
use num_enum::TryFromPrimitive;
use opentelemetry_proto::tonic::common::v1::any_value::Value;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;

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
    <T as ParentId>::Array: Array,
{
    type Error = error::Error;

    fn try_from(rb: &RecordBatch) -> Result<Self, Self::Error> {
        let mut store = Self::default();

        let key_arr = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .map(StringArrayAccessor::new)
            .transpose()?;
        let value_type_arr = get_u8_array(rb, consts::ATTRIBUTE_TYPE)?;

        let value_str_arr = StringArrayAccessor::new(
            rb.column_by_name(consts::ATTRIBUTE_STR)
                .context(error::ColumnNotFoundSnafu {
                    name: consts::ATTRIBUTE_STR,
                })?,
        )?;

        let value_int_arr = get_i64_array_opt(rb, consts::ATTRIBUTE_INT)?;
        let value_double_arr = get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?;
        let value_bool_arr = get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?;
        let value_bytes_arr = get_binary_array_opt(rb, consts::ATTRIBUTE_BYTES)?;

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
                AttributeValueType::Slice => {
                    // todo: support deserialize [any_value::Value::ArrayValue]
                    return error::UnsupportedAttributeValueSnafu { type_name: "slice" }.fail();
                }
                AttributeValueType::Map => {
                    // todo: support deserialize [any_value::Value::KvlistValue]
                    return error::UnsupportedAttributeValueSnafu { type_name: "map" }.fail();
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
            let parent_id_arr = parent_id_arr.as_any().downcast_ref::<T::Array>().context(
                error::ColumnDataTypeMismatchSnafu {
                    name: consts::PARENT_ID,
                    expect: T::arrow_data_type(),
                    actual: parent_id_arr.data_type().clone(),
                },
            )?;
            // Curious, but looks like this is not used anywhere in otel-arrow
            // See https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/common/otlp/attributes.go#L134
            let _delta_encoded = is_delta_encoded(rb.schema_ref());
            let mut parent_id_decoder = T::new_decoder();

            let parent_id =
                parent_id_decoder.decode(parent_id_arr.value_at_or_default(idx), &key, &value);
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
        &mut self.last_mut().unwrap().value
    }
}

/// Key form
const DELTA_ENCODING_KEY: &str = "encoding";
const DELTA_ENCODING_VALUE: &str = "delta";

/// Checks if parent id field is delta encoded from the metadata of schema.
fn is_delta_encoded(schema: &Schema) -> bool {
    schema
        .metadata
        .get(DELTA_ENCODING_KEY)
        .map(|v| v == DELTA_ENCODING_VALUE)
        .unwrap_or(false)
}
