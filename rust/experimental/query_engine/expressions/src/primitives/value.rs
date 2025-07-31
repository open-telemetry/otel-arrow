use std::fmt::{Debug, Display};

use chrono::{DateTime, FixedOffset, SecondsFormat};
use regex::Regex;
use serde_json::json;

use crate::{
    ArrayValue, ExpressionError, IndexValueClosureCallback, KeyValueClosureCallback, MapValue,
    QueryLocation, ValueType, array_value, map_value,
};

#[derive(Debug, Clone)]
pub enum Value<'a> {
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
            _ => None,
        }
    }

    pub fn convert_to_integer(&self) -> Option<i64> {
        match self {
            Value::Boolean(b) => Some(if b.get_value() { 1 } else { 0 }),
            Value::Integer(i) => Some(i.get_value()),
            Value::Double(d) => Some(d.get_value() as i64),
            Value::String(s) => s.get_value().parse::<i64>().ok(),
            _ => None,
        }
    }

    pub fn convert_to_datetime(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            Value::DateTime(d) => Some(d.get_value()),
            _ => None,
        }
    }

    pub fn convert_to_double(&self) -> Option<f64> {
        match self {
            Value::Boolean(b) => Some(if b.get_value() { 1.0 } else { 0.0 }),
            Value::Integer(i) => Some(i.get_value() as f64),
            Value::Double(d) => Some(d.get_value()),
            Value::String(s) => s.get_value().parse::<f64>().ok(),
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
            Value::Null => (action)(""),
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
                            "{:?} value '{right}' on right side of equality operation could not be converted to an array",
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
                        "{:?} value '{right}' on right side of equality operation could not be converted to bool",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::DateTime(d) => match right.convert_to_datetime() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "{:?} value '{right}' on right side of equality operation could not be converted to DateTime",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::Double(d) => match right.convert_to_double() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "{:?} value '{right}' on right side of equality operation could not be converted to double",
                        right.get_value_type(),
                    ),
                )),
            },
            Value::Integer(i) => match right.convert_to_integer() {
                Some(o) => Ok(i.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    query_location.clone(),
                    format!(
                        "{:?} value '{right}' on right side of equality operation could not be converted to int",
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
                            "{:?} value '{right}' on right side of equality operation could not be converted to a map",
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
                            "{:?} value '{left}' on left side of comparison operation could not be converted to DateTime",
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
                            "{:?} value '{right}' on right side of comparison operation could not be converted to DateTime",
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
                            "{:?} value '{left}' on left side of comparison operation could not be converted to double",
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
                            "{:?} value '{right}' on right side of comparison operation could not be converted to double",
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
                            "{:?} value '{left}' on left side of comparison operation could not be converted to int",
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
                            "{:?} value '{right}' on right side of comparison operation could not be converted to int",
                            right.get_value_type(),
                        ),
                    ));
                }
            };

            Ok(compare_ordered_values(v_left, v_right))
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

    fn to_value(&self) -> Value;
}

pub trait BooleanValue: AsValue {
    fn get_value(&self) -> bool;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(if self.get_value() { "true" } else { "false" })
    }
}

pub trait IntegerValue: AsValue {
    fn get_value(&self) -> i64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait DateTimeValue: AsValue {
    fn get_value(&self) -> DateTime<FixedOffset>;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(
            &self
                .get_value()
                .to_rfc3339_opts(SecondsFormat::AutoSi, true),
        )
    }
}

pub trait DoubleValue: AsValue {
    fn get_value(&self) -> f64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait RegexValue: AsValue {
    fn get_value(&self) -> &Regex;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(self.get_value().as_str())
    }
}

pub trait StringValue: AsValue {
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
                "hello world",
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

                // todo: Remove this when another ExpressionError is defined
                #[allow(irrefutable_let_patterns)]
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
            "String value 'hello world' on right side of equality operation could not be converted to bool",
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
            "String value 'hello world' on right side of equality operation could not be converted to int",
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
            "String value 'hello world' on right side of equality operation could not be converted to double",
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
            "String value 'hello world' on right side of equality operation could not be converted to DateTime",
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

            // todo: Remove this when another ExpressionError is defined
            #[allow(irrefutable_let_patterns)]
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

        run_test_failure(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            "String value 'hello world' on left side of comparison operation could not be converted to int",
        );

        run_test_failure(
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            "String value 'hello world' on right side of comparison operation could not be converted to int",
        );

        run_test_failure(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            "String value 'hello world' on right side of comparison operation could not be converted to double",
        );

        run_test_failure(
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
            "String value 'hello world' on left side of comparison operation could not be converted to double",
        );

        run_test_failure(
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
            "Integer value '1' on right side of comparison operation could not be converted to DateTime",
        );

        run_test_failure(
            Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1)),
            Value::DateTime(&DateTimeScalarExpression::new(
                QueryLocation::new_fake(),
                Utc.with_ymd_and_hms(2025, 6, 29, 0, 0, 0).unwrap().into(),
            )),
            "Double value '1.1' on left side of comparison operation could not be converted to DateTime",
        );
    }
}
