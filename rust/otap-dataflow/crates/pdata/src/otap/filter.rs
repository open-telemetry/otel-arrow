// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, DictionaryArray, Float64Array, Int64Array,
    RecordBatch, StringArray, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};

use crate::arrays::get_required_array;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use arrow::buffer::BooleanBuffer;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
pub mod logs;
pub mod traces;
// threshold numbers to determine which method to use for building id filter
// ToDo: determine optimimal numbers
const ID_COLUMN_LENGTH_MIN_THRESHOLD: usize = 2000;
const IDS_PERCENTAGE_MAX_THRESHOLD: f64 = 0.1;

// default boolean array length to use for filter if there is no record batch set
// when attempting to build a filter for a optional record batch
const NO_RECORD_BATCH_FILTER_SIZE: usize = 1;

/// MatchType describes how we should match the String values provided
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// match on the string values exactly how they are defined
    Strict,
    /// apply string values as a regexp
    Regexp,
}

const fn default_match_type() -> MatchType {
    MatchType::Strict
}

/// enum that allows a field to have any type
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AnyValue {
    /// string type
    String(String),
    /// int type
    Int(i64),
    /// double type
    Double(f64),
    /// boolean type
    Boolean(bool),
    /// array of any type
    Array(Vec<AnyValue>),
    /// keyvalue type
    KeyValue(Vec<KeyValue>),
}

/// struct that represents attributes and other key value pairs
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KeyValue {
    key: String,
    value: AnyValue,
}

impl KeyValue {
    /// create a new key value pair
    #[must_use]
    pub fn new(key: String, value: AnyValue) -> Self {
        Self { key, value }
    }
}

/// Finds all nulls and converts them to false values
/// null values affect the filter computation as when
/// we perform the not operation nothing happens to
/// the null values
#[must_use]
fn nulls_to_false(a: &BooleanArray) -> BooleanArray {
    // is_not_null doesn't error see https://docs.rs/arrow/latest/arrow/compute/fn.is_not_null.html
    let valid = arrow::compute::is_not_null(a).expect("is_not_null doesn't error"); // BooleanArray with no nulls
    // the result of boolean array will be a boolean array of equal length so we can guarantee that these two columns have the same length
    arrow::compute::and_kleene(a, &valid).expect("can combine two columns with equal length") // nulls become false; trues stay true
}

enum IdSet {
    U16(HashSet<u16>),
    U32(HashSet<u32>),
}

/// regex_match_column() takes a string column and a regex expression. The function
/// determines what type the string column is either string array or a dictionary
/// array and then applys a regex expression onto it, returns the corresponding boolean
/// array.
/// Returns an error if string column is not a utf8, dictionary(uint8, utf8), or dictionary(uint16, utf8)
fn regex_match_column(src: &ArrayRef, regex: &str) -> Result<BooleanArray> {
    match src.data_type() {
        DataType::Utf8 => {
            let string_array = src
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("array can be downcast to StringArray");

            Ok(
                arrow::compute::regexp_is_match_scalar(string_array, regex, None)
                    .expect("can apply match string column with regexp scalar"),
            )
        }

        DataType::Dictionary(key, val) => {
            match (key.as_ref(), val.as_ref()) {
                (&DataType::UInt8, &DataType::Utf8) => {
                    let dict_arr = src
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("can cast to dictionary array uint8type");

                    // get string from values
                    // safety: we've checked the type
                    let string_values = dict_arr
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect("can cast to string type");
                    // regex check against the values
                    let val_filt =
                        arrow::compute::regexp_is_match_scalar(string_values, regex, None)
                            .expect("can compare string value column to string regex scalar");
                    // now we need to map to the keys
                    let mut key_filt = BooleanBuilder::with_capacity(dict_arr.len());
                    for key in dict_arr.keys() {
                        if let Some(k) = key {
                            key_filt.append_value(val_filt.value(k as usize));
                        } else {
                            key_filt.append_value(false);
                        }
                    }
                    Ok(key_filt.finish())
                }
                (&DataType::UInt16, &DataType::Utf8) => {
                    let dict_arr = src
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can cast to dictionary array uint16type");

                    // get string from values
                    // safety: we've checked the type
                    let string_values = dict_arr
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .expect("can cast to string type");
                    // since we use a scalar here we don't have to worry a column length mismatch when we compare string values to regexp
                    let val_filt =
                        arrow::compute::regexp_is_match_scalar(string_values, regex, None)
                            .expect("can compare string value column to string regex scalar");
                    // now we need to map to the keys
                    let mut key_filt = BooleanBuilder::with_capacity(dict_arr.len());
                    for key in dict_arr.keys() {
                        if let Some(k) = key {
                            key_filt.append_value(val_filt.value(k as usize));
                        } else {
                            key_filt.append_value(false);
                        }
                    }
                    Ok(key_filt.finish())
                }
                _ => {
                    // return error not correct column type
                    Err(Error::UnsupportedStringDictKeyType {
                        data_type: *key.clone(),
                    })
                }
            }
        }
        _ => {
            // return error not correct column type
            Err(Error::UnsupportedStringColumnType {
                data_type: src.data_type().clone(),
            })
        }
    }
}

/// build_uint16_id_filter() takes a id_set which contains ids of u16 we want to remove and the id_column that
/// the set of id's should map to. The function then iterates through the ids and builds a filter
/// that matches those ids and inverts it so the returned BooleanArray when applied will remove rows
/// that contain those ids
/// This will return an error if the column is not DataType::UInt16
fn build_uint16_id_filter(
    id_column: &Arc<dyn Array>,
    id_set: HashSet<u16>,
) -> Result<BooleanArray> {
    if (id_column.len() >= ID_COLUMN_LENGTH_MIN_THRESHOLD)
        && ((id_set.len() as f64 / id_column.len() as f64) <= IDS_PERCENTAGE_MAX_THRESHOLD)
    {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        // build id filter using the id hashset
        for id in id_set {
            let id_scalar = UInt16Array::new_scalar(id);
            // since we use a scalar here we don't have to worry a column length mismatch when we compare
            let id_filter =
                arrow::compute::kernels::cmp::eq(id_column, &id_scalar).map_err(|_| {
                    Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt16,
                    }
                })?;
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        Ok(combined_id_filter)
    } else {
        // convert id to something we can iterate through
        // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
        let uint16_id_array = id_column
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: consts::ID.into(),
                actual: id_column.data_type().clone(),
                expect: DataType::UInt16,
            })?;

        let mut id_filter = BooleanBuilder::new();
        for uint16_id in uint16_id_array {
            match uint16_id {
                Some(uint16) => {
                    id_filter.append_value(id_set.contains(&uint16));
                }
                None => {
                    id_filter.append_value(false);
                }
            }
        }

        Ok(id_filter.finish())
    }
}

/// build_uint32_id_filter() takes a id_set of u32 which contains ids we want to remove and the id_column that
/// the set of id's should map to. The function then iterates through the ids and builds a filter
/// that matches those ids and inverts it so the returned BooleanArray when applied will remove rows
/// that contain those ids
/// This will return an error if the column is not DataType::UInt32, DataType::Dictionary(UInt8, UInt32)
/// or DataType::Dictionary(UInt16, UInt32)
fn build_uint32_id_filter(
    id_column: &Arc<dyn Array>,
    id_set: HashSet<u32>,
) -> Result<BooleanArray> {
    if (id_column.len() >= ID_COLUMN_LENGTH_MIN_THRESHOLD)
        && ((id_set.len() as f64 / id_column.len() as f64) <= IDS_PERCENTAGE_MAX_THRESHOLD)
    {
        let mut combined_id_filter = BooleanArray::new_null(id_column.len());
        // build id filter using the id hashset
        for id in id_set {
            let id_scalar = UInt32Array::new_scalar(id);
            // since we use a scalar here we don't have to worry a column length mismatch when we compare
            let id_filter =
                arrow::compute::kernels::cmp::eq(id_column, &id_scalar).map_err(|_| {
                    Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt32,
                    }
                })?;
            combined_id_filter = arrow::compute::or_kleene(&combined_id_filter, &id_filter)
                .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        }

        Ok(combined_id_filter)
    } else {
        // convert id to something we can iterate through
        // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
        match id_column.data_type() {
            DataType::UInt32 => {
                // convert id to something we can iterate through
                // iterate through and check if id is in the id_set if so then we append true to boolean builder if not then false
                let uint32_id_array = id_column
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: consts::ID.into(),
                        actual: id_column.data_type().clone(),
                        expect: DataType::UInt32,
                    })?;

                let mut id_filter = BooleanBuilder::with_capacity(uint32_id_array.len());
                for uint32_id in uint32_id_array {
                    match uint32_id {
                        Some(uint32) => {
                            id_filter.append_value(id_set.contains(&uint32));
                        }
                        None => {
                            id_filter.append_value(false);
                        }
                    }
                }

                Ok(id_filter.finish())
            }
            DataType::Dictionary(key, val) => match (key.as_ref(), val.as_ref()) {
                (&DataType::UInt8, &DataType::UInt32) => {
                    let uint32_id_dict_array = id_column
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: id_column.data_type().clone(),
                            expect: DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::UInt32),
                            ),
                        })?;

                    let uint32_id_array = uint32_id_dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: uint32_id_dict_array.data_type().clone(),
                            expect: DataType::UInt32,
                        })?;

                    let mut id_filter = BooleanBuilder::with_capacity(uint32_id_dict_array.len());
                    for key in uint32_id_dict_array.keys() {
                        if let Some(k) = key {
                            id_filter
                                .append_value(id_set.contains(&uint32_id_array.value(k as usize)));
                        } else {
                            id_filter.append_value(false);
                        }
                    }

                    Ok(id_filter.finish())
                }
                (&DataType::UInt16, &DataType::UInt32) => {
                    let uint32_id_dict_array = id_column
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: id_column.data_type().clone(),
                            expect: DataType::Dictionary(
                                Box::new(DataType::UInt16),
                                Box::new(DataType::UInt32),
                            ),
                        })?;

                    let uint32_id_array = uint32_id_dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .ok_or_else(|| Error::ColumnDataTypeMismatch {
                            name: consts::ID.into(),
                            actual: uint32_id_dict_array.data_type().clone(),
                            expect: DataType::UInt32,
                        })?;

                    let mut id_filter = BooleanBuilder::with_capacity(uint32_id_dict_array.len());
                    for key in uint32_id_dict_array.keys() {
                        if let Some(k) = key {
                            id_filter
                                .append_value(id_set.contains(&uint32_id_array.value(k as usize)));
                        } else {
                            id_filter.append_value(false);
                        }
                    }

                    Ok(id_filter.finish())
                }
                _ => Err(Error::InvalidListArray {
                    expect_oneof: vec![
                        DataType::UInt32,
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                        DataType::Dictionary(
                            Box::new(DataType::UInt16),
                            Box::new(DataType::UInt32),
                        ),
                    ],
                    actual: (id_column.data_type().clone()),
                }),
            },
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::UInt32,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                ],
                actual: (id_column.data_type().clone()),
            }),
        }
    }
}

/// build_id_filter() takes a set of ids either u16 or u32 and maps it it's corresponding method
/// and then returns a boolean array that masks a column based on the set of ids provided
fn build_id_filter(id_column: &Arc<dyn Array>, id_set: IdSet) -> Result<BooleanArray> {
    match id_set {
        IdSet::U16(u16_ids) => build_uint16_id_filter(id_column, u16_ids),
        IdSet::U32(u32_ids) => build_uint32_id_filter(id_column, u32_ids),
    }
}

/// get_ids() takes the id_column from a record batch and the corresponding filter
/// and applies it to extract all ids that match and then returns the set of ids.
/// This will return an error if the column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32), or DataType::Dictionary(UInt16, UInt32)
fn get_ids(id_column: &Arc<dyn Array>, filter: &BooleanArray) -> Result<IdSet> {
    // get ids being removed
    // error out herre
    let filtered_ids = arrow::compute::filter(id_column, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

    match filtered_ids.data_type() {
        DataType::UInt16 => {
            let filtered_ids = filtered_ids
                .as_any()
                .downcast_ref::<UInt16Array>()
                .expect("Data type is uint16 so we can safely downcast");
            Ok(IdSet::U16(filtered_ids.iter().flatten().collect()))
        }
        DataType::UInt32 => {
            let filtered_ids = filtered_ids
                .as_any()
                .downcast_ref::<UInt32Array>()
                .expect("Data type is uint32 so we can safely downcast");
            Ok(IdSet::U32(filtered_ids.iter().flatten().collect()))
        }
        DataType::Dictionary(key, val) => match (key.as_ref(), val.as_ref()) {
            (&DataType::UInt8, &DataType::UInt32) => {
                let filtered_ids_dictionary = filtered_ids
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect(
                        "Data type is dictionary with key type uint8, so we can safely downcast",
                    );

                let filtered_ids_value = filtered_ids_dictionary
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .expect("Data type of values is Uint32, so we can safely downcast");

                Ok(IdSet::U32(
                    filtered_ids_dictionary
                        .keys()
                        .into_iter()
                        .flatten()
                        .map(|k| filtered_ids_value.value(k as usize))
                        .collect(),
                ))
            }
            (&DataType::UInt16, &DataType::UInt32) => {
                let filtered_ids_dictionary = filtered_ids
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect(
                        "Data type is dictionary with key type uint8, so we can safely downcast",
                    );

                let filtered_ids_value = filtered_ids_dictionary
                    .values()
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .expect("Data type of values is Uint32, so we can safely downcast");

                Ok(IdSet::U32(
                    filtered_ids_dictionary
                        .keys()
                        .into_iter()
                        .flatten()
                        .map(|k| filtered_ids_value.value(k as usize))
                        .collect(),
                ))
            }
            _ => Err(Error::InvalidListArray {
                expect_oneof: vec![
                    DataType::UInt16,
                    DataType::UInt32,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
                ],
                actual: (filtered_ids.data_type().clone()),
            }),
        },
        _ => Err(Error::InvalidListArray {
            expect_oneof: vec![
                DataType::UInt16,
                DataType::UInt32,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
            ],
            actual: (filtered_ids.data_type().clone()),
        }),
    }
}

/// apply_filter() takes a payload, payload_type, and filter and uses the payload type
/// to extract the record_batch then applys the filter and updates the payload with the
/// new record batch.
/// This function will error out if the record batch doesn't exist or the filter column length
/// doesn't match the record batch column length
fn apply_filter(
    payload: &mut OtapArrowRecords,
    payload_type: ArrowPayloadType,
    filter: &BooleanArray,
) -> Result<(u64, u64)> {
    let record_batch = payload
        .get(payload_type)
        .ok_or_else(|| Error::RecordBatchNotFound { payload_type })?;
    let num_rows_before = record_batch.num_rows() as u64;
    let filtered_record_batch = arrow::compute::filter_record_batch(record_batch, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    let num_rows_removed = num_rows_before - (filtered_record_batch.num_rows() as u64);
    payload.set(payload_type, filtered_record_batch);
    Ok((num_rows_before, num_rows_removed))
}

/// update_child_record_batch_filter() takes an child record batch, with it's respective filter
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to update
/// the optional record batch
/// This function will return an error if the id column is not DataType::UInt16 or the filter
/// column length doesn't match the record batch column length
fn update_child_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_ids(id_column, parent_filter)?;
    let parent_id_filter = build_id_filter(parent_id_column, ids_filtered)?;

    arrow::compute::and_kleene(child_filter, &parent_id_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })
}

/// update_parent_record_batch_filter() takes an child record batch, with it's respective filter
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the optional record batch and uses these ids to update
/// the parent record batch
/// This function will return an error if the id column is not DataType::UInt16 or the filter
/// column length doesn't match the record batch column length
fn update_parent_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    // starting with the resource_attr
    // -> get ids of filtered attributes
    // -> map ids to resource_ids in span
    // -> create filter to require these resource_ids
    // -> update span filter
    let parent_ids_column = get_required_array(record_batch, consts::PARENT_ID)?;

    // let ids_filter = match parent_ids_column.data_type() {
    //     DataType::UInt16 => {
    let parent_ids_filtered = get_ids(parent_ids_column, child_filter)?;

    // create filter to remove these ids from span
    let ids_filter = build_id_filter(id_column, parent_ids_filtered)?;

    arrow::compute::and_kleene(parent_filter, &ids_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })
}

/// new_child_record_batch_filter() takes an child record batch,
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to
/// create a filter for the provided child record batch
/// This function will return an error if the id column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32) or the filter column length doesn't match the record batch column length
fn new_child_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    parent_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_ids(id_column, parent_filter)?;
    build_id_filter(parent_id_column, ids_filtered)
}

/// new_parent_record_batch_filter() takes an child record batch,
/// id column from the parent record batch, and the parent record batch filter.
/// This function extracts the masked id from the parent record batch and uses these ids to
/// create a filter for the provided child record batch
/// This function will return an error if the id column is not DataType::UInt16, DataType::UInt32,
/// DataType::Dictionary(UInt8, UInt32) or the filter column length doesn't match the record batch column length
fn new_parent_record_batch_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    child_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_ids_column = get_required_array(record_batch, consts::PARENT_ID)?;

    let parent_ids_filtered = get_ids(parent_ids_column, child_filter)?;

    // create filter to remove these ids from span
    build_id_filter(id_column, parent_ids_filtered)
}

/// get_resource_attr_filter() takes a payload a vec of keyvalue pairs defining attributes to filter on, and the match type
/// and creates a booleanarray that will filter a resource_attribute record batch based on the
/// defined resource attributes we want to match
fn get_resource_attr_filter(
    payload: &OtapArrowRecords,
    attributes: &Vec<KeyValue>,
    match_type: &MatchType,
) -> Result<BooleanArray> {
    // get resource_attrs record batch
    let resource_attrs = match payload.get(ArrowPayloadType::ResourceAttrs) {
        Some(record_batch) => {
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    record_batch.num_rows(),
                )));
            }
            record_batch
        }
        None => {
            // if there is no record batch then
            // if we didn't plan to match any resource attributes -> allow all values through
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            } else {
                // if we did match on resource attributes then there are no attributes to match
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            }
        }
    };

    let num_rows = resource_attrs.num_rows();
    let mut attributes_filter = BooleanArray::new_null(num_rows);
    let key_column = get_required_array(resource_attrs, consts::ATTRIBUTE_KEY)?;

    // generate the filter for this record_batch
    for attribute in attributes {
        // match on key
        let key_scalar = StringArray::new_scalar(attribute.key.clone());
        // since we use a scalar here we don't have to worry a column length mismatch when we compare
        let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
            .expect("can compare string key column to string scalar");
        // and match on value
        let value_filter = match &attribute.value {
            AnyValue::String(value) => {
                // get string column
                let string_column = get_required_array(resource_attrs, consts::ATTRIBUTE_STR)?;
                match match_type {
                    MatchType::Regexp => regex_match_column(string_column, value)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                }
            }
            AnyValue::Int(value) => {
                let int_column = resource_attrs.column_by_name(consts::ATTRIBUTE_INT);

                // check if column exists if not then there is no resource that has this attribute so we can return a all false boolean array
                match int_column {
                    Some(column) => {
                        let value_scalar = Int64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare i64 value column to i64 scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            AnyValue::Double(value) => {
                let double_column = resource_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                match double_column {
                    Some(column) => {
                        let value_scalar = Float64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare f64 value column to f64 scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            AnyValue::Boolean(value) => {
                let bool_column = resource_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                match bool_column {
                    Some(column) => {
                        let value_scalar = BooleanArray::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare

                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare bool value column to bool scalar")
                    }
                    None => {
                        return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
                    }
                }
            }
            _ => {
                // ToDo add keyvalue, array, and bytes
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(num_rows)));
            }
        };
        // build filter that checks for both matching key and value filter
        let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        // combine with overrall filter
        attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    }

    // using the attribute filter we need to get the ids of the rows that match and use that to build our final filter
    // this is to make sure we don't drop attributes that belong to a resource that matched the resource_attributes that
    // were defined

    // we get the id column and apply filter to get the ids we should keep
    let parent_id_column = get_required_array(resource_attrs, consts::PARENT_ID)?;
    // the ids should show up self.resource_attr.len() times otherwise they don't have all the required attributes
    let ids = arrow::compute::filter(&parent_id_column, &attributes_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    // extract correct ids
    let ids = ids
        .as_any()
        .downcast_ref::<UInt16Array>()
        .expect("array can be downcast to UInt16Array");
    // remove null values
    let mut ids_counted: HashMap<u16, usize> = HashMap::with_capacity(ids.len());
    // since we require that all the resource attributes match we use the count of the ids extracted to determine a full match
    // a full match should meant that the amount of times a id appears is equal the number of resource attributes we want to match on
    for id in ids.iter().flatten() {
        *ids_counted.entry(id).or_default() += 1;
    }

    let required_ids_count = attributes.len();
    // filter out ids that don't fully match
    ids_counted.retain(|_key, value| *value >= required_ids_count);

    // return filter built with the ids
    build_id_filter(
        parent_id_column,
        IdSet::U16(ids_counted.into_keys().collect()),
    )
}

/// get_attr_filter() takes a payload a vec of keyvalue pairs defining attributes to filter on, the match type
/// and the specific attribute payload type to filter on and creates a booleanarray that will filter for that
/// record batch based on the provided attributes
fn get_attr_filter(
    payload: &OtapArrowRecords,
    attributes: &Vec<KeyValue>,
    match_type: &MatchType,
    attribute_payload_type: ArrowPayloadType,
) -> Result<BooleanArray> {
    // get record batch containing attributes
    let log_attrs = match payload.get(attribute_payload_type) {
        Some(record_batch) => {
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    record_batch.num_rows(),
                )));
            }
            record_batch
        }
        None => {
            // if there is no record batch then
            // if we didn't plan to match any record attributes -> allow all values through
            if attributes.is_empty() {
                return Ok(BooleanArray::from(BooleanBuffer::new_set(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            } else {
                // if we did match on record attributes then there are no attributes to match
                return Ok(BooleanArray::from(BooleanBuffer::new_unset(
                    NO_RECORD_BATCH_FILTER_SIZE,
                )));
            }
        }
    };

    let num_rows = log_attrs.num_rows();
    // if there is nothing to filter we return all true
    let mut attributes_filter = BooleanArray::from(BooleanBuffer::new_unset(num_rows));

    let key_column = get_required_array(log_attrs, consts::ATTRIBUTE_KEY)?;

    // generate the filter for this record_batch
    for attribute in attributes {
        // match on key
        let key_scalar = StringArray::new_scalar(attribute.key.clone());
        // since we use a scalar here we don't have to worry a column length mismatch when we compare

        let key_filter = arrow::compute::kernels::cmp::eq(&key_column, &key_scalar)
            .expect("can compare string key column to string scalar");
        // match on value
        let value_filter = match &attribute.value {
            AnyValue::String(value) => {
                // get string column
                let string_column = get_required_array(log_attrs, consts::ATTRIBUTE_STR)?;

                match match_type {
                    MatchType::Regexp => regex_match_column(string_column, value)?,
                    MatchType::Strict => {
                        let value_scalar = StringArray::new_scalar(value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&string_column, &value_scalar)
                            .expect("can compare string value column to string scalar")
                    }
                }
            }
            AnyValue::Int(value) => {
                let int_column = log_attrs.column_by_name(consts::ATTRIBUTE_INT);
                match int_column {
                    Some(column) => {
                        let value_scalar = Int64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare i64 value column to i64 scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            AnyValue::Double(value) => {
                let double_column = log_attrs.column_by_name(consts::ATTRIBUTE_DOUBLE);
                match double_column {
                    Some(column) => {
                        let value_scalar = Float64Array::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare f64 value column to f64 scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            AnyValue::Boolean(value) => {
                let bool_column = log_attrs.column_by_name(consts::ATTRIBUTE_BOOL);
                match bool_column {
                    Some(column) => {
                        let value_scalar = BooleanArray::new_scalar(*value);
                        // since we use a scalar here we don't have to worry a column length mismatch when we compare
                        arrow::compute::kernels::cmp::eq(&column, &value_scalar)
                            .expect("can compare bool value column to bool scalar")
                    }
                    None => {
                        continue;
                    }
                }
            }
            _ => {
                // ToDo add keyvalue, array, and bytes
                continue;
            }
        };
        // build filter that checks for both matching key and value filter
        let attribute_filter = arrow::compute::and_kleene(&key_filter, &value_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
        // combine with rest of filters
        attributes_filter = arrow::compute::or_kleene(&attributes_filter, &attribute_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    }

    // now we get ids of filtered attributes to make sure we don't drop any attributes that belong to the log record
    let parent_id_column = get_required_array(log_attrs, consts::PARENT_ID)?;

    let ids = get_ids(parent_id_column, &attributes_filter)?;
    // build filter around the ids and return the filter
    build_id_filter(parent_id_column, ids)
}
