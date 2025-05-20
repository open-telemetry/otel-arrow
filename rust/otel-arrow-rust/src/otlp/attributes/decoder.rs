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

// https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/common/arrow/attributes.go#L40

use std::ops::AddAssign;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BooleanArray, DictionaryArray, PrimitiveArray, RecordBatch,
};
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};
use snafu::{OptionExt, ResultExt};

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor, get_u8_array};
use crate::error::{self, Result};
use crate::otlp::attributes::parent_id::ParentId;
use crate::otlp::attributes::store::AttributeValueType;
use crate::proto::opentelemetry::common::v1::any_value;
use crate::schema::consts;

pub type Attrs16ParentIdDecoder = AttrsParentIdDecoder<u16>;
pub type Attrs32ParentIdDecoder = AttrsParentIdDecoder<u32>;

// AttrsParentIdDecoder implements parent_id decoding for attribute
// sets.  The parent_id in this case is the entity which refers to the
// set of attributes (e.g., a resource, a scope, a metric) contained
// in a RecordBatch.
//
// Phase 1 note: there were several experimental encoding schemes
// tested.  Two schemes named "ParentIdDeltaEncoding",
// "ParentIdNoEncoding" have been removed.
pub struct AttrsParentIdDecoder<T> {
    prev_parent_id: T,
    prev_key: Option<String>,
    prev_value: Option<any_value::Value>,
}

impl<T> Default for AttrsParentIdDecoder<T>
where
    T: ParentId,
{
    fn default() -> Self {
        Self {
            prev_parent_id: T::default(),
            prev_key: None,
            prev_value: None,
        }
    }
}

impl<T> AttrsParentIdDecoder<T>
where
    T: ParentId,
{
    pub fn decode(&mut self, delta_or_parent_id: T, key: &str, value: &any_value::Value) -> T {
        if self.prev_key.as_deref() == Some(key) && self.prev_value.as_ref() == Some(value) {
            let parent_id = self.prev_parent_id.add(delta_or_parent_id);
            self.prev_parent_id = parent_id;
            parent_id
        } else {
            self.prev_key = Some(key.to_string());
            self.prev_value = Some(value.clone());
            self.prev_parent_id = delta_or_parent_id;
            delta_or_parent_id
        }
    }
}

/// Decodes the parent IDs from their transport optimized encoding to the actual ID values.
///
/// In the transport optimized encoding, the record batch is sorted by value Type, then
/// attribute key, then attribute value. If subsequent rows have the same key & value, the
/// parent IDs are delta encoded.
///
/// There are additional exceptions that end the sequence of delta encoding:
/// - null values (we don't consider null values equal, even if current row & previous
///   row value are both null)
/// - value types Empty, Slice & Map are never considered equal
///
/// For example:
///
/// | attr key |  attr val  | parent_id |
/// |----------|------------| --------- |
/// |   "a1"   |  str("a")  |  0        | <-- parent id = 0
/// |   "a1"   |  str("a")  |  0        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 0 = 0
/// |   "a1"   |  str("a")  |  1        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 1 = 1
/// |   "a1"   |  str("a")  |  2        | <-- key & val == previous row -> delta encoded -> parent id = prev id + 2 = 3
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
pub fn materialize_parent_id<T>(record_batch: &RecordBatch) -> Result<RecordBatch>
where
    T: ParentId,
    <T as ParentId>::ArrayType: ArrowPrimitiveType,
    <<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native: AddAssign,
{
    // if the batch is empty, just skip all this logic and return a batch
    if record_batch.num_rows() == 0 {
        return Ok(record_batch.clone());
    }

    let keys_arr =
        record_batch
            .column_by_name(consts::ATTRIBUTE_KEY)
            .context(error::ColumnNotFoundSnafu {
                name: consts::ATTRIBUTE_KEY,
            })?;

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

    let parent_id_arr =
        MaybeDictArrayAccessor::<PrimitiveArray<<T as ParentId>::ArrayType>>::try_new_for_column(
            record_batch,
            consts::PARENT_ID,
        )?;

    let mut materialized_parent_ids =
        PrimitiveArray::<T::ArrayType>::builder(record_batch.num_rows());

    // below we're iterating through the record batch and each time we find a contiguous range
    // where all the types & attribute keys are the same, we use the "eq" compute kernel to
    // compare all the values. Then we use the resulting next-element equality array for the
    // values to determine if there is delta encoding

    // start of the contiguous range:
    let mut curr_range_start = 0;
    // next-element equality array for attribute keys for current type:
    let mut key_eq_next: Option<BooleanArray> = None;

    for idx in 0..record_batch.num_rows() {
        // check if we've found the end of a range of where all the type & attribute are the same
        let found_range_end = if idx == types_eq_next.len() {
            true // end of list
        } else if !types_eq_next.value(idx) {
            // there's a new type -- generate the next element equality array for keys of this type
            let key_range = keys_arr.slice(curr_range_start, idx + 1 - curr_range_start);
            key_eq_next = Some(create_next_element_equality_array(&key_range)?);
            true
        } else if let Some(key_eq_next) = key_eq_next.as_ref() {
            // the range of contiguous keys for this type will end when we find a new key
            !key_eq_next.value(idx - curr_range_start)
        } else {
            false
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

    // safety: RecordBatch::try_new will only return error if our schema doesn't
    // match the passed arrays, or if the arrays are different lengths. Both of
    // these criteria should be met at this point
    Ok(RecordBatch::try_new(schema, columns)
        .expect("should be able to create record batch with parent_id replaced"))
}

// Creates a boolean array where an element having value true means that the
// element at index i of the passed array equals the element at index i + 1.
// Nulls are always treated as not equal
//
// For example,
//    arr = [1, 1, 1, 2, 2,    null, null, 1, 1]
// result = [T, T, F, T, null, null, null, T]
//
fn create_next_element_equality_array(arr: &ArrayRef) -> Result<BooleanArray> {
    // if the array is a dictionary, we compare the dicitonary keys
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
    use crate::arrays::get_u16_array;

    use super::*;
    use arrow::array::{
        BinaryArray, Float64Array, Int64Array, StringArray, UInt8Array, UInt16Array,
    };
    use arrow::datatypes::{ArrowDictionaryKeyType, DataType, Field, Schema, UInt16Type};
    use std::sync::Arc;

    #[test]
    fn test_materialize_parent_id_val_change() {
        let test_data = [
            // (key, str_val, int_val, parent_id, expected)
            ("attr1", Some("a"), None, 1, 1), //
            ("attr1", Some("a"), None, 0, 1), // delta = 0
            ("attr1", Some("a"), None, 0, 1), // delta = 0
            ("attr1", Some("a"), None, 1, 2), // delta = 1
            ("attr1", Some("a"), None, 1, 3), // delta = 1
            ("attr1", Some("b"), None, 1, 1), // not delta (val changed)
            ("attr1", Some("b"), None, 1, 2), // delta = 1
            ("attr2", Some("a"), None, 1, 1), // not delta (key changed)
            ("attr2", Some("a"), None, 1, 2), // delta = 1
            ("attr2", None, Some(1), 1, 1),   // not delta (type changed)
            ("attr2", None, Some(1), 0, 1),   // delta = 0
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

        let result_batch = materialize_parent_id::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.4));
        assert_eq!(parent_ids, &expected)
    }

    #[test]
    fn test_materialize_parent_id_with_nulls() {
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

        let result_batch = materialize_parent_id::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
        assert_eq!(parent_ids, &expected)
    }

    #[test]
    fn test_materialize_parent_id_empty() {
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

        let result_batch = materialize_parent_id::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(vec![]);
        assert_eq!(parent_ids, &expected)
    }

    // test that the materialize_parent_id
    #[test]
    fn test_materialize_parent_id_dicts_values() {
        fn run_test_with_dict_key_type<K>()
        where
            K: ArrowDictionaryKeyType,
            K::Native: TryFrom<u8>,
            <<K as ArrowPrimitiveType>::Native as TryFrom<u8>>::Error: std::fmt::Debug,
        {
            let test_data = [
                // (key, dict_key, parent_id, expected)
                ("attr1", 0, 1, 1),
                ("attr1", 0, 0, 1), // delta = 0
                ("attr1", 0, 0, 1), // delta = 0
                ("attr1", 0, 1, 2), // delta = 1
                ("attr1", 0, 1, 3), // delta = 1
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

            let result_batch = materialize_parent_id::<u16>(&record_batch).unwrap();
            let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
            let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
            assert_eq!(parent_ids, &expected)
        }

        run_test_with_dict_key_type::<UInt8Type>();
        run_test_with_dict_key_type::<UInt16Type>();
    }

    #[test]
    fn test_materialize_parent_ids_other_values_types() {
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

        let result_batch = materialize_parent_id::<u16>(&record_batch).unwrap();
        let parent_ids = get_u16_array(&result_batch, consts::PARENT_ID).unwrap();
        let expected = UInt16Array::from_iter_values(test_data.iter().map(|a| a.3));
        assert_eq!(parent_ids, &expected)
    }
}
