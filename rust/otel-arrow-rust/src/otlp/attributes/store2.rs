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
    ByteArrayAccessor, Int64ArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    get_bool_array_opt, get_f64_array_opt, get_u8_array,
};

use crate::error;
use crate::otlp::attributes::parent_id::{ParentId, TryNew};
use crate::proto::opentelemetry::common::v1::any_value;
use crate::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use crate::schema::consts;
use arrow::array::{ArrowPrimitiveType, BooleanArray, Float64Array, RecordBatch, UInt8Array};
use snafu::{OptionExt, ResultExt};

use super::cbor;
use super::store::AttributeValueType;

/// A reference-based attribute store that directly accesses the RecordBatch.
/// This avoids copying data into a HashMap and creating new KeyValue objects.
#[derive(Default)]
pub struct AttributeStore2<'a, T: ParentId<'a>> {
    /// The RecordBatch containing attribute data
    pub record_batch: Option<&'a RecordBatch>,
    /// The last ID used in delta ID lookups
    pub last_id: T,
    /// Maps parent_id -> index of first attribute with that parent_id.
    /// 0 implies None. Capacity should be cardinality of parent_id
    pub first_index_by_id: Vec<usize>,
    /// For each row, contains index of the next row with the same parent_id, or None
    pub next_indices: Vec<usize>,
    /// Precomputed column arrays.
    pub columns: Option<AttributeColumns<'a, T>>,
}

pub type Attribute32Store2<'a> = AttributeStore2<'a, u32>;
pub type Attribute16Store2<'a> = AttributeStore2<'a, u16>;

pub struct AttributeColumns<'a, T: ParentId<'a>> {
    parent_id: T::ArrayType,
    key: StringArrayAccessor<'a>,
    value_type: UInt8Array,
    value_str: StringArrayAccessor<'a>,
    value_int: Int64ArrayAccessor<'a>,
    value_bool: BooleanArray,
    value_double: Float64Array,
    value_bytes: ByteArrayAccessor<'a>,
    value_ser: ByteArrayAccessor<'a>,
}

impl<'a, T> AttributeStore2<'a, T>
where
    T: ParentId<'a> + Clone,
{
    /// Retrieve attributes by ID with delta encoding (increments last_id by delta).
    /// Returns an iterator over KeyValue pairs with the given parent ID.
    pub fn attributes_by_delta_id(&mut self, delta: T) -> AttributeIterator<'a, T> {
        self.last_id += delta;
        self.attributes_by_id(self.last_id)
    }

    /// Retrieve attributes by ID without delta encoding.
    /// Returns an iterator over KeyValue pairs with the given parent ID.
    pub fn attributes_by_id(&self, id: T) -> AttributeIterator<'a, T> {
        // TODO: having trouble here.
        let idx: usize = 0; // @@@<T as ParentId>::as_usize(id);
        AttributeIterator {
            store: self,
            current_idx: self.first_index_by_id.get(idx).cloned(),
            columns: self.columns.expect("is Some"),
        }
    }
}

/// Iterator that yields KeyValue pairs from an AttributeStore2.
pub struct AttributeIterator<'a, T: ParentId<'a>> {
    store: &'a AttributeStore2<'a, T>,
    current_idx: Option<usize>,

    /// Precomputed column arrays.
    pub columns: AttributeColumns<'a, T>,
}

impl<'a, T: ParentId<'a>> AttributeColumns<'a, T> {
    fn value_type(&self, idx: usize) -> error::Result<AttributeValueType> {
        AttributeValueType::try_from(self.value_type.value_at_or_default(idx))
            .context(error::UnrecognizedAttributeValueTypeSnafu)
    }

    fn otel_value(&self, idx: usize, value_type: AttributeValueType) -> Option<any_value::Value> {
        // Note: we do not expect any match arm to return None,
        // and we use _or_default() or equivalent.  If any of
        // these evaluate to None, somehow create a warning, as it
        // indicates corrupted data?

        use any_value::Value;
        match value_type {
            AttributeValueType::Str => self.value_str.value_at(idx).map(Value::StringValue),
            AttributeValueType::Int => self.value_int.value_at(idx).map(Value::IntValue),
            AttributeValueType::Double => self.value_double.value_at(idx).map(Value::DoubleValue),
            AttributeValueType::Bool => self.value_bool.value_at(idx).map(Value::BoolValue),
            AttributeValueType::Bytes => self.value_bytes.value_at(idx).map(Value::BytesValue),
            AttributeValueType::Slice => self
                .value_ser
                .value_at(idx)
                .map(|ref bytes| cbor::decode_pcommon_val(&bytes).ok())
                .flatten()
                .flatten(),

            AttributeValueType::Map => self
                .value_ser
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
    T: ParentId<'a> + Clone,
{
    type Item = KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.current_idx?;

        // Update for the next iteration
        self.current_idx = self.store.next_indices.get(idx).cloned();

        // Note! Repeat calculation of key and value; ok()? won't fail.
        let key = self.columns.key.value_at_or_default(idx);
        let value = self
            .columns
            .otel_value(idx, self.columns.value_type(idx).ok()?);

        Some(KeyValue {
            key,
            value: Some(AnyValue { value: value }),
        })
    }
}

impl<'a, T> TryFrom<&RecordBatch> for AttributeStore2<'a, T>
where
    T: ParentId<'a>,
    <T as ParentId<'a>>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId<'a>>::ArrayType as ArrowPrimitiveType>::Native: Into<T>,
{
    type Error = error::Error;

    fn try_from(rb: &RecordBatch) -> Result<Self, Self::Error> {
        let num_rows = rb.num_rows();
        if num_rows == 0 {
            return Ok(Self::default());
        }

        // Mandatory fields
        let cols = AttributeColumns {
            key: rb
                .column_by_name(consts::ATTRIBUTE_KEY)
                .map(StringArrayAccessor::try_new)
                .transpose()?,
            value_type: get_u8_array(rb, consts::ATTRIBUTE_TYPE)?,
            parent_id: rb
                .column_by_name(consts::PARENT_ID)
                .map(T::AccessorType::try_new)
                .transpose()?,

            // Value columns are all optional
            value_str: rb
                .column_by_name(consts::ATTRIBUTE_STR)
                .and_then(|arr| StringArrayAccessor::try_new(arr).ok()),
            value_int: rb
                .column_by_name(consts::ATTRIBUTE_INT)
                .and_then(|arr| Int64ArrayAccessor::try_new(arr).ok()),
            value_double: get_f64_array_opt(rb, consts::ATTRIBUTE_DOUBLE)?,
            value_bool: get_bool_array_opt(rb, consts::ATTRIBUTE_BOOL)?,
            value_bytes: rb
                .column_by_name(consts::ATTRIBUTE_BYTES)
                .and_then(|arr| ByteArrayAccessor::try_new(arr).ok()),
            value_ser: rb
                .column_by_name(consts::ATTRIBUTE_SER)
                .and_then(|arr| ByteArrayAccessor::try_new(arr).ok()),
        };

        let mut parent_id_decoder = T::new_decoder();
        let mut next_indices = Vec::with_capacity(num_rows, 0);

        // Capacity should be cardinality of parent_id, which is
        // unknown in this location. Note: is there a way to pass in
        // the cardinality?
        let mut first_index_by_id = Vec::new();
        let mut last_index_by_id = Vec::new();

        for idx in 0..num_rows {
            let key = cols.key.value_at_or_default(idx);
            let value = cols.otel_value(idx, cols.value_type(idx)?);

            // Note! Can we compare the encoded form of key/value
            // instead of calling otel_value() twice?
            let parent_id = parent_id_decoder.decode(
                cols.parent_id.value_at_or_default(idx).into(),
                &key,
                &value,
            );

            // Update first_index_by_id if this is the first time we see this parent_id
            if !first_index_by_id.get(parent_id) {
                first_index_by_id.insert(parent_id.clone(), idx);
            }

            // Update the next_indices for the previous entry with the same parent_id
            if let Some(&prev_idx) = last_index_by_id.get(parent_id) {
                next_indices[prev_idx] = Some(idx);
            }

            // Remember this as the last seen index for this parent_id
            *last_index_by_id.get_mut(parent_id) = idx;
        }

        Ok(Self {
            record_batch: Some(rb),
        })
    }
}
