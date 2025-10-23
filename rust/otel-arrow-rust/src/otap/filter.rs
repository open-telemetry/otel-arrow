// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{ArrayRef, BooleanArray, BooleanBuilder, DictionaryArray, StringArray};
use arrow::datatypes::{ArrowDictionaryKeyType, DataType, UInt8Type, UInt16Type};

use crate::otap::error::{self, Result};
use serde::Deserialize;
pub mod logs;

/// MatchType describes how we should match the String values provided
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// match on the string values exactly how they are defined
    Strict,
    /// apply string values as a regexp
    Regexp,
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
pub fn nulls_to_false(a: &BooleanArray) -> BooleanArray {
    // is_not_null doesn't error see https://docs.rs/arrow/latest/arrow/compute/fn.is_not_null.html
    let valid = arrow::compute::is_not_null(a).expect("is_not_null doesn't error"); // BooleanArray with no nulls
    // the result of boolean array will be a boolean array of equal length so we can guarantee that these two columns have the same length
    arrow::compute::and_kleene(a, &valid).expect("can combine two columns with equal length") // nulls become false; trues stay true
}

///
pub fn regex_match_column(src: &ArrayRef, regex: &str) -> Result<BooleanArray> {
    match src.data_type() {
        DataType::Utf8 => {
            let string_array = src
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("array can be downcast to StringArray");

            return Ok(
                arrow::compute::regexp_is_match_scalar(string_array, regex, None)
                    .expect("can apply match string column with regexp scalar"),
            );
        }

        DataType::Dictionary(key, val) => {
            match (key.as_ref(), val.as_ref()) {
                (&DataType::UInt8, &DataType::Utf8) => {
                    regex_is_match_for_dict::<UInt8Type>(src, regex)
                }
                (&DataType::UInt16, &DataType::Utf8) => {
                    regex_is_match_for_dict::<UInt16Type>(src, regex)
                }
                _ => {
                    // return error not correct column type
                    panic!("imps");
                }
            }
        }
        _ => {
            // return error not correct column type
            panic!("impossible");
        }
    }
}

// todo return errors instead of expect here
fn regex_is_match_for_dict<K: ArrowDictionaryKeyType>(
    src: &ArrayRef,
    regex: &str,
) -> Result<BooleanArray> {
    let dict_arr = src
        .as_any()
        .downcast_ref::<DictionaryArray<K>>()
        .expect("can cast to type");

    // get string from values
    // safety: we've checked the type
    let string_values = dict_arr
        .values()
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("can cast to type");
    // regex check against the values
    let val_filt = arrow::compute::regexp_is_match_scalar(string_values, regex, None).expect("");
    Ok(val_filt)
    // let key_filt = BooleanBuilder::with_capacity(dict_arr.len());
    // for key in dict_arr.keys() {
    //     if let Some(k) = key {
    //         key_filter.append_value(val_filt.value(k as usize));
    //     }
    // }
    // Ok(key_filt.finish())
}
