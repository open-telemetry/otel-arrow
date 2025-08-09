use std::fmt::{Debug, Display};

use chrono::{DateTime, FixedOffset, SecondsFormat, TimeZone, Utc};
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
        }
    }

    pub fn convert_to_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(b.get_value()),
            Value::Integer(i) => Some(i.get_value() != 0),
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
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt().map(|v| v != 0),
            _ => None,
        }
    }

    pub fn convert_to_integer(&self) -> Option<i64> {
        match self {
            Value::Boolean(b) => Some(if b.get_value() { 1 } else { 0 }),
            Value::Integer(i) => Some(i.get_value()),
            Value::Double(d) => Some(d.get_value() as i64),
            Value::String(s) => s.get_value().parse::<i64>().ok(),
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt(),
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
            Value::Double(d) => Some(d.get_value()),
            Value::String(s) => s.get_value().parse::<f64>().ok(),
            Value::DateTime(d) => d.get_value().timestamp_nanos_opt().map(|v| v as f64),
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
}
