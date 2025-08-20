// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Debug, Display, Write};

use chrono::{DateTime, FixedOffset, SecondsFormat, TimeDelta, TimeZone, Utc};
use regex::{Regex, RegexBuilder};
use serde_json::json;

use crate::{
    ArrayValue, ExpressionError, IndexValueClosureCallback, KeyValueClosureCallback, MapValue,
    QueryLocation, ValueType, array_value, date_utils, map_value,
};

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Array(&'a dyn ArrayValue),
    Boolean(&'a dyn BooleanValue),
    DateTime(&'a dyn DateTimeValue),
    Double(&'a dyn DoubleValue),
    Integer(&'a dyn IntegerValue),
    Map(&'a dyn MapValue),
    Null,
    Regex(&'a dyn RegexValue),
    String(&'a dyn StringValue),
    TimeSpan(&'a dyn TimeSpanValue),
}

impl Value<'_> {
    pub fn get_value_type(&self) -> ValueType {
        match self {
            Value::Array(_) => ValueType::Array,
            Value::Boolean(_) => ValueType::Boolean,
            Value::DateTime(_) => ValueType::DateTime,
            Value::Double(_) => ValueType::Double,
            Value::Integer(_) => ValueType::Integer,
            Value::Map(_) => ValueType::Map,
            Value::Null => ValueType::Null,
            Value::Regex(_) => ValueType::Regex,
            Value::String(_) => ValueType::String,
            Value::TimeSpan(_) => ValueType::TimeSpan,
        }
    }

    pub fn convert_to_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(b.get_value()),
            Value::Integer(i) => Some(i.get_value() != 0),
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt().map(|v| v != 0),
            Value::Double(d) => Some(d.get_value() != 0.0),
            Value::String(s) => {
                let v = s.get_value();
                if caseless::default_caseless_match_str(v, "true") {
                    Some(true)
                } else if caseless::default_caseless_match_str(v, "false") {
                    Some(false)
                } else {
                    None
                }
            }
            Value::TimeSpan(ts) => ts.get_value().num_nanoseconds().map(|v| v != 0),
            _ => None,
        }
    }

    pub fn convert_to_integer(&self) -> Option<i64> {
        match self {
            Value::Boolean(b) => Some(if b.get_value() { 1 } else { 0 }),
            Value::Integer(i) => Some(i.get_value()),
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt(),
            Value::Double(d) => Some(d.get_value() as i64),
            Value::String(s) => s.get_value().parse::<i64>().ok(),
            Value::TimeSpan(ts) => ts.get_value().num_nanoseconds(),
            _ => None,
        }
    }

    pub fn convert_to_datetime(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            Value::DateTime(d) => Some(d.get_value()),
            _ => {
                if let Some(i) = self.convert_to_integer() {
                    Some(Utc.timestamp_nanos(i).into())
                } else {
                    let mut result = None;
                    self.convert_to_string(&mut |v| {
                        result = Some(date_utils::parse_date_time(v));
                    });

                    match result {
                        Some(v) => v.ok(),
                        None => panic!(
                            "Encountered a Value which does not correctly implement convert_to_string"
                        ),
                    }
                }
            }
        }
    }

    pub fn convert_to_double(&self) -> Option<f64> {
        match self {
            Value::Boolean(b) => Some(if b.get_value() { 1.0 } else { 0.0 }),
            Value::Integer(i) => Some(i.get_value() as f64),
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt().map(|v| v as f64),
            Value::Double(d) => Some(d.get_value()),
            Value::String(s) => s.get_value().parse::<f64>().ok(),
            Value::TimeSpan(ts) => ts.get_value().num_nanoseconds().map(|v| v as f64),
            _ => None,
        }
    }

    pub fn convert_to_string<F>(&self, action: &mut F)
    where
        F: FnMut(&str),
    {
        match self {
            Value::Array(a) => a.to_string(action),
            Value::Boolean(b) => b.to_string(action),
            Value::DateTime(d) => d.to_string(action),
            Value::Double(d) => d.to_string(action),
            Value::Integer(i) => i.to_string(action),
            Value::Map(m) => m.to_string(action),
            Value::Null => (action)("null"),
            Value::Regex(r) => r.to_string(action),
            Value::String(s) => (action)(s.get_value()),
            Value::TimeSpan(t) => t.to_string(action),
        }
    }

    pub fn convert_to_timespan(&self) -> Option<TimeDelta> {
        match self {
            Value::TimeSpan(t) => Some(t.get_value()),
            _ => {
                if let Some(i) = self.convert_to_integer() {
                    Some(TimeDelta::nanoseconds(i))
                } else {
                    let mut result = None;
                    self.convert_to_string(&mut |v| {
                        result = Some(date_utils::parse_timespan(v));
                    });

                    match result {
                        Some(v) => v.ok(),
                        None => panic!(
                            "Encountered a Value which does not correctly implement convert_to_string"
                        ),
                    }
                }
            }
        }
    }

    pub(crate) fn to_json_value(&self) -> serde_json::Value {
        match self {
            Value::Array(a) => {
                let mut values = Vec::new();

                a.get_items(&mut IndexValueClosureCallback::new(|_, value| {
                    values.push(value.to_json_value());
                    true
                }));

                serde_json::Value::Array(values)
            }
            Value::Boolean(b) => json!(b.get_value()),
            Value::DateTime(_) => json!(self.to_string()),
            Value::Double(d) => json!(d.get_value()),
            Value::Integer(o) => json!(o.get_value()),
            Value::Map(m) => {
                let mut values = serde_json::Map::new();

                m.get_items(&mut KeyValueClosureCallback::new(|key, value| {
                    values.insert(key.into(), value.to_json_value());
                    true
                }));

                serde_json::Value::Object(values)
            }
            Value::Null => json!(null),
            Value::Regex(_) => json!(self.to_string()),
            Value::String(s) => json!(s.get_value()),
            Value::TimeSpan(_) => json!(self.to_string()),
        }
    }

    pub fn are_values_equal(
        query_location: &QueryLocation,
        left: &Value,
        right: &Value,
        case_insensitive: bool,
    ) -> Result<bool, ExpressionError> {
        let is_left_null = left.get_value_type() == ValueType::Null;
        let is_right_null = right.get_value_type() == ValueType::Null;

        if is_left_null || is_right_null {
            return Ok(is_left_null == is_right_null);
        }

        match left {
            Value::Array(left_array) => {
                if let Value::Array(right_array) = right {
                    array_value::equal_to(
                        query_location,
                        *left_array,
                        *right_array,
                        case_insensitive,
                    )
                } else if let Value::String(right_string) = right {
                    Ok(Self::are_string_values_equal(
                        right_string.get_value(),
                        left,
                        case_insensitive,
                    ))
                } else {
                    Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on right side of equality operation could not be converted to an array",
                            right.get_value_type(),
                        ),
                    ))
                }
            }
            Value::Boolean(b) => match right.convert_to_bool() {
                Some(o) => Ok(b.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "Value of '{:?}' type on right side of equality operation could not be converted to bool",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::DateTime(d) => match right.convert_to_datetime() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "Value of '{:?}' type on right side of equality operation could not be converted to DateTime",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::Double(d) => match right.convert_to_double() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "Value of '{:?}' type on right side of equality operation could not be converted to double",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::Integer(i) => match right.convert_to_integer() {
                Some(o) => Ok(i.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "Value of '{:?}' type on right side of equality operation could not be converted to int",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::Map(left_map) => {
                if let Value::Map(right_map) = right {
                    map_value::equal_to(query_location, *left_map, *right_map, case_insensitive)
                } else if let Value::String(right_string) = right {
                    Ok(Self::are_string_values_equal(
                        right_string.get_value(),
                        left,
                        case_insensitive,
                    ))
                } else {
                    Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on right side of equality operation could not be converted to a map",
                            right.get_value_type(),
                        ),
                    ))
                }
            }
            Value::Null => panic!("Null equality should be handled before match"),
            Value::Regex(r) => Ok(Self::are_string_values_equal(
                r.get_value().as_str(),
                right,
                case_insensitive,
            )),
            Value::String(s) => Ok(Self::are_string_values_equal(
                s.get_value(),
                right,
                case_insensitive,
            )),
            Value::TimeSpan(t) => match right.convert_to_timespan() {
                Some(o) => Ok(t.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "Value of '{:?}' type on right side of equality operation could not be converted to TimeSpan",
                        right.get_value_type(),
                    ),
                )),
            },
        }
    }

    /// Compare two values and return:
    ///
    /// * -1 if left is less than right
    /// * 0 if left is equal to right
    /// * 1 if left is greater than right
    pub fn compare_values(
        query_location: &QueryLocation,
        left: &Value,
        right: &Value,
    ) -> Result<i64, ExpressionError> {
        let left_type = left.get_value_type();
        let right_type = right.get_value_type();

        if left_type == ValueType::DateTime || right_type == ValueType::DateTime {
            let v_left = match left.convert_to_datetime() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on left side of comparison operation could not be converted to DateTime",
                            left.get_value_type(),
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_datetime() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on right side of comparison operation could not be converted to DateTime",
                            right.get_value_type(),
                        ),
                    ));
                }
            };

            Ok(compare_ordered_values(v_left, v_right))
        } else if left_type == ValueType::Double || right_type == ValueType::Double {
            let v_left = match left.convert_to_double() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on left side of comparison operation could not be converted to double",
                            left.get_value_type(),
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_double() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on right side of comparison operation could not be converted to double",
                            right.get_value_type(),
                        ),
                    ));
                }
            };

            Ok(compare_double_values(v_left, v_right))
        } else {
            let v_left = match left.convert_to_integer() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on left side of comparison operation could not be converted to int",
                            left.get_value_type(),
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_integer() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        query_location.clone(),
                        format!(
                            "Value of '{:?}' type on right side of comparison operation could not be converted to int",
                            right.get_value_type(),
                        ),
                    ));
                }
            };

            Ok(compare_ordered_values(v_left, v_right))
        }
    }

    pub fn contains(
        query_location: &QueryLocation,
        haystack: &Value,
        needle: &Value,
        case_insensitive: bool,
    ) -> Result<bool, ExpressionError> {
        match haystack {
            Value::Array(array) => {
                let mut found = false;
                array.get_items(&mut IndexValueClosureCallback::new(|_, item_value| {
                    match Self::are_values_equal(
                        query_location,
                        &item_value,
                        needle,
                        case_insensitive,
                    ) {
                        Ok(is_equal) => {
                            if is_equal {
                                found = true;
                                false // Stop iteration
                            } else {
                                true // Continue iteration
                            }
                        }
                        Err(_) => true, // Continue iteration on error
                    }
                }));
                Ok(found)
            }
            Value::String(string_val) => {
                let haystack_str = string_val.get_value();

                let mut result = None;
                needle.convert_to_string(&mut |s| {
                    let contains_result = if case_insensitive {
                        let folded_haystack = caseless::default_case_fold_str(haystack_str);
                        let folded_needle = caseless::default_case_fold_str(s);

                        folded_haystack.contains(&folded_needle)
                    } else {
                        haystack_str.contains(s)
                    };
                    result = Some(contains_result)
                });

                if let Some(r) = result {
                    Ok(r)
                } else {
                    panic!(
                        "Encountered a Value type which does not correctly implement convert_to_string"
                    )
                }
            }
            _ => Err(ExpressionError::TypeMismatch(
                query_location.clone(),
                format!(
                    "Haystack value of '{:?}' type is not supported for contains operation. Only Array and String values are supported.",
                    haystack.get_value_type(),
                ),
            )),
        }
    }

    pub fn replace_matches(
        haystack: &Value,
        needle: &Value,
        replacement: &Value,
        case_insensitive: bool,
    ) -> Option<String> {
        match (haystack, needle, replacement) {
            // String needle - simple text replacement (with case sensitivity support)
            (
                Value::String(haystack_str),
                Value::String(needle_str),
                Value::String(replacement_str),
            ) => {
                let haystack_val = haystack_str.get_value();
                let needle_val = needle_str.get_value();
                let replacement_val = replacement_str.get_value();

                if case_insensitive {
                    // Use caseless crate for case-insensitive replacement
                    let mut result = String::new();
                    let mut remaining = haystack_val;

                    loop {
                        // Find the first occurrence of needle in remaining text (case-insensitive)
                        let mut match_start = None;
                        let mut match_end = None;

                        // Search for needle at each position in remaining
                        for i in 0..=remaining.len() {
                            if i + needle_val.len() <= remaining.len() {
                                let candidate = &remaining[i..i + needle_val.len()];
                                if caseless::default_caseless_match_str(candidate, needle_val) {
                                    match_start = Some(i);
                                    match_end = Some(i + needle_val.len());
                                    break;
                                }
                            }
                        }

                        if let (Some(start), Some(end)) = (match_start, match_end) {
                            result.push_str(&remaining[..start]);
                            result.push_str(replacement_val);
                            remaining = &remaining[end..];
                        } else {
                            result.push_str(remaining);
                            break;
                        }
                    }

                    Some(result)
                } else {
                    Some(haystack_val.replace(needle_val, replacement_val))
                }
            }
            // Regex needle - regex replacement with capture group support
            (
                Value::String(haystack_str),
                Value::Regex(needle_regex),
                Value::String(replacement_str),
            ) => {
                let regex = needle_regex.get_value();
                let result =
                    regex.replace_all(haystack_str.get_value(), replacement_str.get_value());
                Some(result.to_string())
            }
            _ => None,
        }
    }

    pub fn add(left: &Value, right: &Value) -> Option<NumericValue> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Some(NumericValue::Integer(l.get_value() + r.get_value()))
            }
            (Value::Double(l), Value::Double(r)) => {
                Some(NumericValue::Double(l.get_value() + r.get_value()))
            }
            (Value::DateTime(l), r) => Some(NumericValue::DateTime(
                l.get_value() + r.convert_to_timespan()?,
            )),
            (Value::TimeSpan(l), r) => Some(NumericValue::TimeSpan(
                l.get_value() + r.convert_to_timespan()?,
            )),
            _ => {
                if Self::values_may_be_double(left, right) {
                    let left_double = left.convert_to_double()?;
                    let right_double = right.convert_to_double()?;
                    Some(NumericValue::Double(left_double + right_double))
                } else {
                    let left_integer = left.convert_to_integer()?;
                    let right_integer = right.convert_to_integer()?;
                    Some(NumericValue::Integer(left_integer + right_integer))
                }
            }
        }
    }

    pub fn subtract(left: &Value, right: &Value) -> Option<NumericValue> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Some(NumericValue::Integer(l.get_value() - r.get_value()))
            }
            (Value::Double(l), Value::Double(r)) => {
                Some(NumericValue::Double(l.get_value() - r.get_value()))
            }
            (Value::DateTime(l), r) => Some(NumericValue::DateTime(
                l.get_value() - r.convert_to_timespan()?,
            )),
            (Value::TimeSpan(l), r) => Some(NumericValue::TimeSpan(
                l.get_value() - r.convert_to_timespan()?,
            )),
            _ => {
                if Self::values_may_be_double(left, right) {
                    let left_double = left.convert_to_double()?;
                    let right_double = right.convert_to_double()?;
                    Some(NumericValue::Double(left_double - right_double))
                } else {
                    let left_integer = left.convert_to_integer()?;
                    let right_integer = right.convert_to_integer()?;
                    Some(NumericValue::Integer(left_integer - right_integer))
                }
            }
        }
    }

    pub fn multiply(left: &Value, right: &Value) -> Option<NumericValue> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Some(NumericValue::Integer(l.get_value() * r.get_value()))
            }
            (Value::Double(l), Value::Double(r)) => {
                Some(NumericValue::Double(l.get_value() * r.get_value()))
            }
            _ => {
                if Self::values_may_be_double(left, right) {
                    let left_double = left.convert_to_double()?;
                    let right_double = right.convert_to_double()?;
                    Some(NumericValue::Double(left_double * right_double))
                } else {
                    let left_integer = left.convert_to_integer()?;
                    let right_integer = right.convert_to_integer()?;
                    Some(NumericValue::Integer(left_integer * right_integer))
                }
            }
        }
    }

    pub fn divide(left: &Value, right: &Value) -> Option<NumericValue> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Some(NumericValue::Integer(l.get_value() / r.get_value()))
            }
            (Value::Double(l), Value::Double(r)) => {
                Some(NumericValue::Double(l.get_value() / r.get_value()))
            }
            _ => {
                if Self::values_may_be_double(left, right) {
                    let left_double = left.convert_to_double()?;
                    let right_double = right.convert_to_double()?;
                    Some(NumericValue::Double(left_double / right_double))
                } else {
                    let left_integer = left.convert_to_integer()?;
                    let right_integer = right.convert_to_integer()?;
                    Some(NumericValue::Integer(left_integer / right_integer))
                }
            }
        }
    }

    pub fn modulus(left: &Value, right: &Value) -> Option<NumericValue> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Some(NumericValue::Integer(l.get_value() % r.get_value()))
            }
            (Value::Double(l), Value::Double(r)) => {
                Some(NumericValue::Double(l.get_value() % r.get_value()))
            }
            _ => {
                if Self::values_may_be_double(left, right) {
                    let left_double = left.convert_to_double()?;
                    let right_double = right.convert_to_double()?;
                    Some(NumericValue::Double(left_double % right_double))
                } else {
                    let left_integer = left.convert_to_integer()?;
                    let right_integer = right.convert_to_integer()?;
                    Some(NumericValue::Integer(left_integer % right_integer))
                }
            }
        }
    }

    pub fn bin(value: &Value, bin_size: &Value) -> Option<NumericValue> {
        match (value, bin_size) {
            (Value::Integer(value), Value::Integer(bin_size)) => {
                let bin_size = bin_size.get_value();
                Some(NumericValue::Integer(
                    (value.get_value() / bin_size) * bin_size,
                ))
            }
            (Value::Double(value), Value::Double(bin_size)) => {
                let bin_size = bin_size.get_value();
                Some(NumericValue::Double(
                    (value.get_value() / bin_size).floor() * bin_size,
                ))
            }
            (Value::DateTime(value), right) => {
                if let Some(interval) = right.convert_to_timespan() {
                    let dt = value.get_value();

                    let timestamp_ms = dt.timestamp_millis();

                    let interval_ms = interval.num_milliseconds();

                    let binned_timestamp_ms = (timestamp_ms / interval_ms) * interval_ms;

                    Some(NumericValue::DateTime(
                        DateTime::from_timestamp_millis(binned_timestamp_ms)
                            .expect("Timestamp conversion should not fail")
                            .with_timezone(dt.offset()),
                    ))
                } else {
                    None
                }
            }
            (Value::TimeSpan(value), right) => {
                if let Some(interval) = right.convert_to_timespan() {
                    let t = value.get_value();

                    let timestamp_ms = t.num_milliseconds();

                    let interval_ms = interval.num_milliseconds();

                    let binned_timestamp_ms = (timestamp_ms / interval_ms) * interval_ms;

                    Some(NumericValue::TimeSpan(TimeDelta::milliseconds(
                        binned_timestamp_ms,
                    )))
                } else {
                    None
                }
            }
            _ => {
                if Self::values_may_be_double(value, bin_size) {
                    let value_double = value.convert_to_double()?;
                    let bin_size_double = bin_size.convert_to_double()?;
                    Some(NumericValue::Double(
                        (value_double / bin_size_double).floor() * bin_size_double,
                    ))
                } else {
                    let value_integer = value.convert_to_integer()?;
                    let bin_size_integer = bin_size.convert_to_integer()?;
                    Some(NumericValue::Integer(
                        (value_integer / bin_size_integer) * bin_size_integer,
                    ))
                }
            }
        }
    }

    pub fn ceiling(value: &Value) -> Option<i64> {
        if let Value::Double(d) = value {
            Some(d.get_value().ceil() as i64)
        } else if let Value::String(s) = value
            && let Some(d) = s.get_value().parse::<f64>().ok()
        {
            Some(d.ceil() as i64)
        } else {
            value.convert_to_integer()
        }
    }

    pub fn floor(value: &Value) -> Option<i64> {
        if let Value::Double(d) = value {
            Some(d.get_value().floor() as i64)
        } else if let Value::String(s) = value
            && let Some(d) = s.get_value().parse::<f64>().ok()
        {
            Some(d.floor() as i64)
        } else {
            value.convert_to_integer()
        }
    }

    pub fn parse_regex(
        query_location: &QueryLocation,
        pattern: &Value,
        options: Option<&Value>,
    ) -> Result<Regex, ExpressionError> {
        if let Value::String(s) = pattern {
            match options {
                None => Regex::new(s.get_value()).map_err(|e| {
                    ExpressionError::ParseError(
                        query_location.clone(),
                        format!("Failed to parse Regex from pattern: {e}"),
                    )
                }),
                Some(Value::String(options)) => {
                    let options = options.get_value();

                    let mut builder = RegexBuilder::new(s.get_value());

                    if options.contains('i') {
                        builder.case_insensitive(true);
                    }
                    if options.contains('m') {
                        builder.multi_line(true);
                    }
                    if options.contains('s') {
                        builder.dot_matches_new_line(true);
                    }

                    builder.build().map_err(|e| {
                        ExpressionError::ParseError(
                            query_location.clone(),
                            format!("Failed to parse Regex from pattern: {e}"),
                        )
                    })
                }
                _ => Err(ExpressionError::ParseError(
                    query_location.clone(),
                    format!(
                        "Input of '{:?}' type could not be pased as Regex options",
                        options.unwrap().get_value_type()
                    ),
                )),
            }
        } else {
            Err(ExpressionError::ParseError(
                query_location.clone(),
                format!(
                    "Input of '{:?}' type could not be pased as a Regex",
                    pattern.get_value_type()
                ),
            ))
        }
    }

    pub fn negate(value: &Value) -> Option<NumericValue> {
        match value {
            Value::Integer(i) => Some(NumericValue::Integer(-i.get_value())),
            Value::Double(d) => Some(NumericValue::Double(-d.get_value())),
            Value::TimeSpan(t) => Some(NumericValue::TimeSpan(-t.get_value())),
            _ => {
                if let Value::String(l) = value
                    && l.get_value().contains(['.', 'e'])
                {
                    value.convert_to_double().map(|v| NumericValue::Double(-v))
                } else {
                    value
                        .convert_to_integer()
                        .map(|v| NumericValue::Integer(-v))
                }
            }
        }
    }

    fn are_string_values_equal(left: &str, right: &Value, case_insensitive: bool) -> bool {
        let mut r = None;

        right.convert_to_string(&mut |o| {
            if case_insensitive {
                r = Some(caseless::default_caseless_match_str(left, o))
            } else {
                r = Some(left == o)
            }
        });

        r.expect("Encountered a type which does not correctly implement convert_to_string")
    }

    fn values_may_be_double(left: &Value, right: &Value) -> bool {
        if left.get_value_type() == ValueType::Double || right.get_value_type() == ValueType::Double
        {
            return true;
        }

        let left_may_by_double = if let Value::String(l) = left
            && l.get_value().contains(['.', 'e'])
        {
            true
        } else {
            false
        };

        let right_may_by_double = if let Value::String(r) = right
            && r.get_value().contains(['.', 'e'])
        {
            true
        } else {
            false
        };

        left_may_by_double || right_may_by_double
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = None;
        self.convert_to_string(&mut |v| {
            result = Some(f.write_str(v));
        });
        result.expect("Encountered a type which does not correctly implement convert_to_string")
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        Self::are_values_equal(&QueryLocation::new_fake(), self, other, false).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    Integer(i64),
    DateTime(DateTime<FixedOffset>),
    Double(f64),
    TimeSpan(TimeDelta),
}

pub trait AsValue: Debug {
    fn get_value_type(&self) -> ValueType;

    fn to_value(&self) -> Value<'_>;
}

#[derive(Debug, Clone)]
pub enum StaticValue<'a> {
    Array(&'a (dyn ArrayValue + 'static)),
    Boolean(&'a (dyn BooleanValue + 'static)),
    DateTime(&'a (dyn DateTimeValue + 'static)),
    Double(&'a (dyn DoubleValue + 'static)),
    Integer(&'a (dyn IntegerValue + 'static)),
    Map(&'a (dyn MapValue + 'static)),
    Null,
    Regex(&'a (dyn RegexValue + 'static)),
    String(&'a (dyn StringValue + 'static)),
    TimeSpan(&'a (dyn TimeSpanValue + 'static)),
}

pub trait AsStaticValue: AsValue {
    fn to_static_value(&self) -> StaticValue<'_>;
}

impl<T: AsStaticValue> AsValue for T {
    fn get_value_type(&self) -> ValueType {
        match self.to_static_value() {
            StaticValue::Array(_) => ValueType::Array,
            StaticValue::Boolean(_) => ValueType::Boolean,
            StaticValue::DateTime(_) => ValueType::DateTime,
            StaticValue::Double(_) => ValueType::Double,
            StaticValue::Integer(_) => ValueType::Integer,
            StaticValue::Map(_) => ValueType::Map,
            StaticValue::Null => ValueType::Null,
            StaticValue::Regex(_) => ValueType::Regex,
            StaticValue::String(_) => ValueType::String,
            StaticValue::TimeSpan(_) => ValueType::TimeSpan,
        }
    }

    fn to_value(&self) -> Value<'_> {
        match self.to_static_value() {
            StaticValue::Array(a) => Value::Array(a),
            StaticValue::Boolean(b) => Value::Boolean(b),
            StaticValue::DateTime(d) => Value::DateTime(d),
            StaticValue::Double(d) => Value::Double(d),
            StaticValue::Integer(i) => Value::Integer(i),
            StaticValue::Map(m) => Value::Map(m),
            StaticValue::Null => Value::Null,
            StaticValue::Regex(r) => Value::Regex(r),
            StaticValue::String(s) => Value::String(s),
            StaticValue::TimeSpan(t) => Value::TimeSpan(t),
        }
    }
}

pub trait BooleanValue: Debug {
    fn get_value(&self) -> bool;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(if self.get_value() { "true" } else { "false" })
    }
}

pub trait IntegerValue: Debug {
    fn get_value(&self) -> i64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait DateTimeValue: Debug {
    fn get_value(&self) -> DateTime<FixedOffset>;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(
            &self
                .get_value()
                .to_rfc3339_opts(SecondsFormat::AutoSi, true),
        )
    }
}

pub trait DoubleValue: Debug {
    fn get_value(&self) -> f64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait RegexValue: Debug {
    fn get_value(&self) -> &Regex;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(self.get_value().as_str())
    }
}

pub trait StringValue: Debug {
    fn get_value(&self) -> &str;
}

pub trait TimeSpanValue: Debug {
    fn get_value(&self) -> TimeDelta;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        let mut v = String::new();

        let raw_nano_seconds = self.get_value().num_nanoseconds().unwrap_or(0);

        let mut nano_seconds = if raw_nano_seconds < 0 {
            v.push('-');
            raw_nano_seconds.unsigned_abs()
        } else {
            raw_nano_seconds as u64
        };

        let days = nano_seconds / (86_400 * 1_000_000_000);
        if days > 0 {
            write!(&mut v, "{}.", days).ok();
            nano_seconds -= days * (86_400 * 1_000_000_000);
        }

        let hours = nano_seconds / (3_600 * 1_000_000_000);
        write!(&mut v, "{:02}:", hours).ok();
        if hours > 0 {
            nano_seconds -= hours * (3_600 * 1_000_000_000);
        }

        let minutes = nano_seconds / (60 * 1_000_000_000);
        write!(&mut v, "{:02}:", minutes).ok();
        if minutes > 0 {
            nano_seconds -= minutes * (60 * 1_000_000_000);
        }

        let seconds = nano_seconds / 1_000_000_000;
        write!(&mut v, "{:02}", seconds).ok();
        if seconds > 0 {
            nano_seconds -= seconds * 1_000_000_000;
        }

        let remaining_ticks = nano_seconds / 100;
        if remaining_ticks > 0 {
            let (trailing_zeros, mut significant) = count_trailing_zeros(remaining_ticks);

            let mut buffer = [0x00u8; 7];

            let mut pos = 6i32 - trailing_zeros as i32;
            while pos >= 0 {
                let temp = 48 + significant;
                significant /= 10;
                buffer[pos as usize] = (temp - (significant * 10)) as u8;
                pos -= 1;
            }

            v.push('.');
            v.push_str(std::str::from_utf8(&buffer[..(7 - trailing_zeros)]).unwrap());
        }

        (action)(v.as_str());

        fn count_trailing_zeros(value: u64) -> (usize, u64) {
            let mut count = 0;
            let mut v = value;

            while v > 0 && v % 10 == 0 {
                count += 1;
                v /= 10;
            }

            (count, v)
        }
    }
}

fn compare_ordered_values<T: Ord>(left: T, right: T) -> i64 {
    match left.cmp(&right) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn compare_double_values(left: f64, right: f64) -> i64 {
    if left == right {
        0
    } else if left < right {
        -1
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{TimeZone, Utc};

    use crate::*;

    use super::*;

    #[test]
    pub fn test_convert_to_bool() {
        let run_test_success = |value: Value, expected: Option<bool>| {
            let actual = Value::convert_to_bool(&value);

            assert_eq!(expected, actual)
        };

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Some(true),
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                false,
            )),
            Some(false),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Some(true),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                100,
            )),
            Some(true),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            Some(false),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Some(true),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                100.0,
            )),
            Some(true),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 0.0)),
            Some(false),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "true",
            )),
            Some(true),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "TRUE",
            )),
            Some(true),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "false",
            )),
            Some(false),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "FALSE",
            )),
            Some(false),
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(0).into(),
            )),
            Some(false),
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(1).into(),
            )),
            Some(true),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            None,
        );
    }

    #[test]
    pub fn test_convert_to_integer() {
        let run_test_success = |value: Value, expected: Option<i64>| {
            let actual = Value::convert_to_integer(&value);

            assert_eq!(expected, actual)
        };

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Some(1),
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                false,
            )),
            Some(0),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Some(1),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(1),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Some(1),
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(1).into(),
            )),
            Some(1),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            None,
        );
    }

    #[test]
    pub fn test_convert_to_datetime() {
        let run_test_success = |value: Value, expected: Option<DateTime<FixedOffset>>| {
            let actual = Value::convert_to_datetime(&value);

            assert_eq!(expected, actual)
        };

        let dt: DateTime<FixedOffset> = Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into();
        let unix_plus_one = Utc.timestamp_nanos(1).into();

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                dt,
            )),
            Some(dt),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "6/29/2025",
            )),
            Some(dt),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Some(unix_plus_one),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Some(unix_plus_one),
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Some(unix_plus_one),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world 8/25/2025",
            )),
            None,
        );
    }

    #[test]
    pub fn test_convert_to_double() {
        let run_test_success = |value: Value, expected: Option<f64>| {
            let actual = Value::convert_to_double(&value);

            assert_eq!(expected, actual)
        };

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Some(1.0),
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                false,
            )),
            Some(0.0),
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Some(1.0),
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(1.18),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.18",
            )),
            Some(1.18),
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(1).into(),
            )),
            Some(1.0),
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            None,
        );
    }

    #[test]
    pub fn test_convert_to_string() {
        let run_test_success =
            |value: Value, expected: &str| assert_eq!(expected, value.to_string());

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            "true",
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            "1",
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            "1.18",
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            "hello world",
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            "2025-06-29T00:00:00Z",
        );

        run_test_success(
            Value::Regex(&RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(".*").unwrap(),
            )),
            ".*",
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
            )),
            "{}",
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )]),
            )),
            "{\"key1\":1}",
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                Vec::new(),
            )),
            "[]",
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )],
            )),
            "[1]",
        );
    }

    #[test]
    pub fn test_are_values_equal() {
        let run_test_success =
            |left: Value, right: Value, case_insensitive: bool, expected: bool| {
                let actual = Value::are_values_equal(
                    &QueryLocation::new_fake(),
                    &left,
                    &right,
                    case_insensitive,
                )
                .unwrap();

                assert_eq!(expected, actual)
            };

        let run_test_failure =
            |left: Value, right: Value, case_insensitive: bool, expected: &str| {
                let actual = Value::are_values_equal(
                    &QueryLocation::new_fake(),
                    &left,
                    &right,
                    case_insensitive,
                )
                .unwrap_err();

                if let ExpressionError::TypeMismatch(_, msg) = actual {
                    assert_eq!(expected, msg);
                } else {
                    panic!("Expected NotSupported")
                }
            };

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            false,
            true,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "HELLO WORLD",
            )),
            false,
            false,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "HELLO WORLD",
            )),
            true,
            true,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1234",
            )),
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                1234,
            )),
            false,
            true,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1234",
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1235.0,
            )),
            false,
            false,
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "TRUE",
            )),
            false,
            true,
        );

        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            false,
            false,
        );

        run_test_failure(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            false,
            "Value of 'String' type on right side of equality operation could not be converted to bool",
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            false,
            true,
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 0.0)),
            false,
            false,
        );

        run_test_failure(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            false,
            "Value of 'String' type on right side of equality operation could not be converted to int",
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            false,
            true,
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            false,
            false,
        );

        run_test_failure(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            false,
            "Value of 'String' type on right side of equality operation could not be converted to double",
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            false,
            true,
        );

        run_test_failure(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            false,
            "Value of 'String' type on right side of equality operation could not be converted to DateTime",
        );

        run_test_success(
            Value::Regex(&RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(".*").unwrap(),
            )),
            Value::Regex(&RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(".*").unwrap(),
            )),
            false,
            true,
        );

        run_test_success(
            Value::Regex(&RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(".*").unwrap(),
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                ".*",
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            )),
            false,
            false,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )],
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    )),
                ],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Double(DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.0,
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "2",
                    )),
                ],
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    )),
                ],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                ],
            )),
            false,
            false,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "HELLO",
                ))],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello",
                ))],
            )),
            false,
            false,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "HELLO",
                ))],
            )),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello",
                ))],
            )),
            true,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello",
                ))],
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "[\"hello\"]",
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello",
                ))],
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "[\"HELLO\"]",
            )),
            true,
            true,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
            )),
            false,
            true,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
            )),
            false,
            false,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "hello",
                        )),
                    ),
                    (
                        "key2".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "world",
                        )),
                    ),
                ]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key2".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "world",
                        )),
                    ),
                    (
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "hello",
                        )),
                    ),
                ]),
            )),
            false,
            true,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "hello",
                        )),
                    ),
                    (
                        "key2".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "world",
                        )),
                    ),
                ]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key_other".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "other",
                        )),
                    ),
                    (
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "hello",
                        )),
                    ),
                ]),
            )),
            false,
            false,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "1",
                        )),
                    ),
                    (
                        "key2".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "2",
                        )),
                    ),
                ]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "key2".into(),
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            2,
                        )),
                    ),
                    (
                        "key1".into(),
                        StaticScalarExpression::Double(DoubleScalarExpression::new(
                            QueryLocation::new_fake(),
                            1.0,
                        )),
                    ),
                ]),
            )),
            false,
            true,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "HELLO",
                    )),
                )]),
            )),
            false,
            false,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "HELLO",
                    )),
                )]),
            )),
            true,
            true,
        );

        // Note: In this test the two maps are NOT equal because keys are
        // currently never handled as case-insensitive.
        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "KEY1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "HELLO",
                    )),
                )]),
            )),
            true,
            false,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "{\"key1\":\"hello\"}",
            )),
            false,
            true,
        );

        run_test_success(
            Value::Map(&MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                )]),
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "{\"KEY1\":\"HELLO\"}",
            )),
            true,
            true,
        );

        run_test_success(Value::Null, Value::Null, false, true);

        run_test_success(
            Value::Null,
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            false,
            false,
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::Null,
            false,
            false,
        );
    }

    #[test]
    pub fn test_compare_values() {
        let run_test_success = |left: Value, right: Value, expected: i64| {
            let actual = Value::compare_values(&QueryLocation::new_fake(), &left, &right).unwrap();

            assert_eq!(expected, actual)
        };

        let run_test_failure = |left: Value, right: Value, expected: &str| {
            let actual =
                Value::compare_values(&QueryLocation::new_fake(), &left, &right).unwrap_err();

            if let ExpressionError::TypeMismatch(_, msg) = actual {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected NotSupported")
            }
        };

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            -1,
        );

        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            -1,
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            0,
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 0)),
            1,
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            0,
        );

        run_test_success(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(1).into(),
            )),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            0,
        );

        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.timestamp_nanos(1).into(),
            )),
            0,
        );

        run_test_failure(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            "Value of 'String' type on left side of comparison operation could not be converted to int",
        );

        run_test_failure(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            "Value of 'String' type on right side of comparison operation could not be converted to int",
        );

        run_test_failure(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            "Value of 'String' type on right side of comparison operation could not be converted to double",
        );

        run_test_failure(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            "Value of 'String' type on left side of comparison operation could not be converted to double",
        );
    }

    #[test]
    pub fn test_contains() {
        let run_test_success =
            |haystack: Value, needle: Value, case_insensitive: bool, expected: bool| {
                let actual = Value::contains(
                    &QueryLocation::new_fake(),
                    &haystack,
                    &needle,
                    case_insensitive,
                )
                .unwrap();
                assert_eq!(expected, actual);
            };

        let run_test_failure =
            |haystack: Value, needle: Value, case_insensitive: bool, expected: &str| {
                let actual = Value::contains(
                    &QueryLocation::new_fake(),
                    &haystack,
                    &needle,
                    case_insensitive,
                )
                .unwrap_err();

                #[allow(irrefutable_let_patterns)]
                if let ExpressionError::TypeMismatch(_, msg) = actual {
                    assert_eq!(expected, msg);
                } else {
                    panic!("Expected TypeMismatch")
                }
            };

        // Test Array contains
        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                ],
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            false,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                ],
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "HELLO",
            )),
            true,
            true,
        );

        run_test_success(
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                ],
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "foo",
            )),
            false,
            false,
        );

        // Test String contains
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "world",
            )),
            false,
            true,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "WORLD",
            )),
            true,
            true,
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "foo",
            )),
            false,
            false,
        );

        // Test error cases
        run_test_failure(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "world",
            )),
            false,
            "Haystack value of 'Integer' type is not supported for contains operation. Only Array and String values are supported.",
        );
    }

    #[test]
    pub fn test_add() {
        let run_test_success = |left: Value, right: Value, expected: Option<NumericValue>| {
            let actual = Value::add(&left, &right);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.01,
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(NumericValue::Double(2.19)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
            Some(NumericValue::Integer(60)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.01",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.18",
            )),
            Some(NumericValue::Double(2.19)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1e10",
            )),
            Some(NumericValue::Double(10000000001.0)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            Some(NumericValue::Integer(19)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    pub fn test_subtract() {
        let run_test_success = |left: Value, right: Value, expected: Option<NumericValue>| {
            let actual = Value::subtract(&left, &right);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.01,
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(NumericValue::Double(-0.16999999999999993)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
            Some(NumericValue::Integer(24)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.01",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.18",
            )),
            Some(NumericValue::Double(-0.16999999999999993)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1e10",
            )),
            Some(NumericValue::Double(-9999999999.0)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            Some(NumericValue::Integer(-17)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    pub fn test_multiply() {
        let run_test_success = |left: Value, right: Value, expected: Option<NumericValue>| {
            let actual = Value::multiply(&left, &right);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.01,
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(NumericValue::Double(1.1918)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
            Some(NumericValue::Integer(756)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.01",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.18",
            )),
            Some(NumericValue::Double(1.1918)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1e10",
            )),
            Some(NumericValue::Double(1.0e10)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            Some(NumericValue::Integer(18)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    pub fn test_divide() {
        let run_test_success = |left: Value, right: Value, expected: Option<NumericValue>| {
            let actual = Value::divide(&left, &right);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.01,
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                1.18,
            )),
            Some(NumericValue::Double(0.8559322033898306)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
            Some(NumericValue::Integer(2)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.01",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.18",
            )),
            Some(NumericValue::Double(0.8559322033898306)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1e10",
            )),
            Some(NumericValue::Double(1e-10)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            Some(NumericValue::Integer(0)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    pub fn test_modulus() {
        let run_test_success = |left: Value, right: Value, expected: Option<NumericValue>| {
            let actual = Value::modulus(&left, &right);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                10.18,
            )),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 3.0)),
            Some(NumericValue::Double(1.1799999999999997)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 10)),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 3)),
            Some(NumericValue::Integer(1)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "10.18",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "3.0",
            )),
            Some(NumericValue::Double(1.1799999999999997)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1e10",
            )),
            Some(NumericValue::Double(1.0)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "10",
            )),
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "3")),
            Some(NumericValue::Integer(1)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    pub fn test_ceiling() {
        let run_test_success = |value: Value, expected: Option<i64>| {
            let actual = Value::ceiling(&value);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1)),
            Some(2),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                -1.9,
            )),
            Some(-1),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 0.0)),
            Some(0),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0)),
            Some(2),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Some(42),
        );
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                -42,
            )),
            Some(-42),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.1",
            )),
            Some(2),
        );
        // String values that cannot be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, None);
    }

    #[test]
    pub fn test_floor() {
        let run_test_success = |value: Value, expected: Option<i64>| {
            let actual = Value::floor(&value);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1)),
            Some(1),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                -1.9,
            )),
            Some(-2),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 0.0)),
            Some(0),
        );
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0)),
            Some(2),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Some(42),
        );
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                -42,
            )),
            Some(-42),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.1",
            )),
            Some(1),
        );
        // String values that cannot be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, None);
    }

    #[test]
    pub fn test_bin() {
        let run_test_success = |value: Value, bin_size: Value, expected: Option<NumericValue>| {
            let actual = Value::bin(&value, &bin_size);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                10018.18,
            )),
            Value::Double(&DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                100.0,
            )),
            Some(NumericValue::Double(10000.0)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                10018,
            )),
            Value::Integer(&IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                100,
            )),
            Some(NumericValue::Integer(10000)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "10018.18",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "100.0",
            )),
            Some(NumericValue::Double(10000.0)),
        );
        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "10018",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "100.0",
            )),
            Some(NumericValue::Double(10000.0)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "10018",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "100",
            )),
            Some(NumericValue::Integer(10000)),
        );
        // String values that cannot be parsed as numeric
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, Value::Null, None);
    }

    #[test]
    fn test_parse_regex() {
        let run_test_success = |pattern: Value, options: Option<Value>, test: &str| {
            let is_match =
                Value::parse_regex(&QueryLocation::new_fake(), &pattern, options.as_ref())
                    .unwrap()
                    .is_match(test);

            assert!(is_match);
        };

        let run_test_failure = |pattern: Value, options: Option<Value>| {
            Value::parse_regex(&QueryLocation::new_fake(), &pattern, options.as_ref()).unwrap_err();
        };

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "[^a]ello world",
            )),
            None,
            "hello world",
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "i",
            ))),
            "HELLO WORLD",
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^\\w*.\\w*$",
            )),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "s",
            ))),
            "hello\nworld",
        );

        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "^\\w*$",
            )),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "m",
            ))),
            "hello\nworld",
        );

        run_test_failure(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "(")),
            None,
        );

        run_test_failure(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "(")),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "m",
            ))),
        );

        run_test_failure(Value::Null, None);

        run_test_failure(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "(")),
            Some(Value::Null),
        );
    }

    #[test]
    pub fn test_negate() {
        let run_test_success = |value: Value, expected: Option<NumericValue>| {
            let actual = Value::negate(&value);
            assert_eq!(expected, actual)
        };

        // Double values
        run_test_success(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1)),
            Some(NumericValue::Double(-1.1)),
        );

        // Integer values
        run_test_success(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 42)),
            Some(NumericValue::Integer(-42)),
        );

        // String values that can be parsed as double
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "1.1",
            )),
            Some(NumericValue::Double(-1.1)),
        );
        // String values that can be parsed as integer
        run_test_success(
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "1")),
            Some(NumericValue::Integer(-1)),
        );
        // String values that cannot be parsed
        run_test_success(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            )),
            None,
        );

        // Null value
        run_test_success(Value::Null, None);

        // Value convertable to int
        run_test_success(
            Value::Boolean(&BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
            Some(NumericValue::Integer(-1)),
        );
    }

    #[test]
    pub fn test_timespan_to_string() {
        let run_test = |input: TimeDelta, expected: &str| {
            let timespan = StaticScalarExpression::TimeSpan(TimeSpanScalarExpression::new(
                QueryLocation::new_fake(),
                input,
            ));
            assert_eq!(expected, timespan.to_value().to_string());
        };

        run_test(TimeDelta::days(1), "1.00:00:00");
        run_test(TimeDelta::hours(1), "01:00:00");
        run_test(TimeDelta::minutes(1), "00:01:00");
        run_test(TimeDelta::seconds(1), "00:00:01");
        run_test(TimeDelta::milliseconds(1), "00:00:00.001");
        run_test(TimeDelta::milliseconds(900), "00:00:00.9");
        run_test(TimeDelta::milliseconds(999), "00:00:00.999");
        run_test(TimeDelta::microseconds(1), "00:00:00.000001");
        run_test(TimeDelta::nanoseconds(100), "00:00:00.0000001");
        run_test(TimeDelta::nanoseconds(1), "00:00:00");

        run_test(
            TimeDelta::milliseconds(1) + TimeDelta::microseconds(1) + TimeDelta::nanoseconds(100),
            "00:00:00.0010011",
        );
        run_test(
            TimeDelta::days(-1)
                - TimeDelta::hours(23)
                - TimeDelta::minutes(59)
                - TimeDelta::seconds(59),
            "-1.23:59:59",
        );
    }
}
