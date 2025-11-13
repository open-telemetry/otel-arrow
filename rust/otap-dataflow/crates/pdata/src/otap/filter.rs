// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBuilder, DictionaryArray, RecordBatch, StringArray,
    UInt16Array,
};
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};

use crate::arrays::get_required_array;
use crate::otap::OtapArrowRecords;
use crate::otap::error::{Error, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use serde::Deserialize;
use std::collections::HashSet;
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

/// build_uint16_id_filter() takes a id_set which contains ids we want to remove and the id_column that
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
            let id_filter = arrow::compute::kernels::cmp::eq(id_column, &id_scalar)
                .expect("can compare uint16 id column with uint16 scalar");
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

/// get_uint16_ids() takes the id_column from a record batch and the corresponding filter
/// and applies it to extract all ids that match and then returns the set of ids.
/// This will return an error if the column is not DataType::UInt16
fn get_uint16_ids(
    id_column: &Arc<dyn Array>,
    filter: &BooleanArray,
    column_type: &str,
) -> Result<HashSet<u16>> {
    // get ids being removed
    // error out herre
    let filtered_ids = arrow::compute::filter(id_column, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;

    // downcast id and get unique values
    let filtered_ids = filtered_ids
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| Error::ColumnDataTypeMismatch {
            name: column_type.into(),
            actual: filtered_ids.data_type().clone(),
            expect: DataType::UInt16,
        })?;
    Ok(filtered_ids.iter().flatten().collect())
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
) -> Result<()> {
    let record_batch = payload
        .get(payload_type)
        .ok_or_else(|| Error::RecordBatchNotFound { payload_type })?;
    let filtered_record_batch = arrow::compute::filter_record_batch(record_batch, filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?;
    payload.set(payload_type, filtered_record_batch);
    Ok(())
}

/// update_filter() takes the record batch, id column, filter to update and the primary filter
/// that should be applied to the record batch to determine the ids that should be removed
/// This function will return an error if the id column is not DataType::UInt16 or the filter
/// column length doesn't match the record batch column length
fn update_optional_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    filter_to_update: &BooleanArray,
    primary_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_uint16_ids(id_column, primary_filter, consts::ID)?;
    let parent_id_filter = build_uint16_id_filter(parent_id_column, ids_filtered)?;
    Ok(
        arrow::compute::and_kleene(filter_to_update, &parent_id_filter)
            .map_err(|e| Error::ColumnLengthMismatch { source: e })?,
    )
}

fn update_primary_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    filter_to_update: &BooleanArray,
    primary_filter: &BooleanArray,
) -> Result<BooleanArray> {
    // starting with the resource_attr
    // -> get ids of filtered attributes
    // -> map ids to resource_ids in span
    // -> create filter to require these resource_ids
    // -> update span filter
    let parent_ids_column = get_required_array(record_batch, consts::PARENT_ID)?;

    let parent_ids_filtered =
        get_uint16_ids(parent_ids_column, &primary_filter, consts::PARENT_ID)?;

    // create filter to remove these ids from span
    let ids_filter = build_uint16_id_filter(ids_column, parent_ids_filtered)?;

    Ok(arrow::compute::and_kleene(primary_update, &ids_filter)
        .map_err(|e| Error::ColumnLengthMismatch { source: e })?)
}

fn new_filter(
    record_batch: &RecordBatch,
    id_column: &Arc<dyn Array>,
    primary_filter: &BooleanArray,
) -> Result<BooleanArray> {
    let parent_id_column = get_required_array(record_batch, consts::PARENT_ID)?;
    let ids_filtered = get_uint16_ids(id_column, primary_filter, consts::ID)?;
    Ok(build_uint16_id_filter(parent_id_column, ids_filtered)?)
}
