use chrono::{DateTime, FixedOffset, SecondsFormat};

use crate::{Expression, ExpressionError, QueryLocation, ValueType};

pub enum Value<'a> {
    Boolean(&'a dyn BooleanValue),
    Integer(&'a dyn IntegerValue),
    DateTime(&'a dyn DateTimeValue),
    Double(&'a dyn DoubleValue),
    String(&'a dyn StringValue),
}

impl Expression for Value<'_> {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            Value::Boolean(b) => b.get_query_location(),
            Value::Integer(i) => i.get_query_location(),
            Value::DateTime(d) => d.get_query_location(),
            Value::Double(d) => d.get_query_location(),
            Value::String(s) => s.get_query_location(),
        }
    }
}

impl Value<'_> {
    pub fn get_value_type(&self) -> ValueType {
        match self {
            Value::Boolean(_) => ValueType::Boolean,
            Value::Integer(_) => ValueType::Integer,
            Value::DateTime(_) => ValueType::DateTime,
            Value::Double(_) => ValueType::Double,
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
            Value::Boolean(b) => b.to_string(action),
            Value::Integer(i) => i.to_string(action),
            Value::DateTime(d) => d.to_string(action),
            Value::Double(d) => d.to_string(action),
            Value::String(s) => (action)(s.get_value()),
        }
    }

    pub fn are_values_equal(
        left: &Value,
        right: &Value,
        case_insensitive: bool,
    ) -> Result<bool, ExpressionError> {
        match left {
            Value::Boolean(b) => match right.convert_to_bool() {
                Some(o) => Ok(b.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    right.get_query_location().clone(),
                    format!(
                        "{:?} value '{}' on right side of equality operation could not be converted to bool",
                        right.get_value_type(),
                        right.to_string()
                    ),
                )),
            },
            Value::Integer(i) => match right.convert_to_integer() {
                Some(o) => Ok(i.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    right.get_query_location().clone(),
                    format!(
                        "{:?} value '{}' on right side of equality operation could not be converted to int",
                        right.get_value_type(),
                        right.to_string()
                    ),
                )),
            },
            Value::DateTime(d) => match right.convert_to_datetime() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    right.get_query_location().clone(),
                    format!(
                        "{:?} value '{}' on right side of equality operation could not be converted to DateTime",
                        right.get_value_type(),
                        right.to_string()
                    ),
                )),
            },
            Value::Double(d) => match right.convert_to_double() {
                Some(o) => Ok(d.get_value() == o),
                None => Err(ExpressionError::TypeMismatch(
                    right.get_query_location().clone(),
                    format!(
                        "{:?} value '{}' on right side of equality operation could not be converted to double",
                        right.get_value_type(),
                        right.to_string()
                    ),
                )),
            },
            Value::String(s) => {
                let mut r = None;

                right.convert_to_string(&mut |o| {
                    if case_insensitive {
                        r = Some(caseless::default_caseless_match_str(s.get_value(), o))
                    } else {
                        r = Some(s.get_value() == o)
                    }
                });

                Ok(r.expect(
                    "Encountered a type which does not correctly implement convert_to_string",
                ))
            }
        }
    }

    pub fn compare_values(left: &Value, right: &Value) -> Result<i64, ExpressionError> {
        let left_type = left.get_value_type();
        let right_type = right.get_value_type();

        if left_type == ValueType::DateTime || right_type == ValueType::DateTime {
            let v_left = match left.convert_to_datetime() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on left side of comparison operation could not be converted to DateTime",
                            left.get_value_type(),
                            left.to_string()
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_datetime() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on right side of comparison operation could not be converted to DateTime",
                            right.get_value_type(),
                            right.to_string()
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
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on left side of comparison operation could not be converted to double",
                            left.get_value_type(),
                            left.to_string()
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_double() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on right side of comparison operation could not be converted to double",
                            right.get_value_type(),
                            right.to_string()
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
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on left side of comparison operation could not be converted to int",
                            left.get_value_type(),
                            left.to_string()
                        ),
                    ));
                }
            };

            let v_right = match right.convert_to_integer() {
                Some(i) => i,
                None => {
                    return Err(ExpressionError::TypeMismatch(
                        right.get_query_location().clone(),
                        format!(
                            "{:?} value '{}' on right side of comparison operation could not be converted to int",
                            right.get_value_type(),
                            right.to_string()
                        ),
                    ));
                }
            };

            Ok(compare_ordered_values(v_left, v_right))
        }
    }

    /// Note: Only call this for tests and errors as it will copy the string to
    /// the heap. Call convert_to_string instead to operate on the &str behind
    /// the value.
    fn to_string(&self) -> Box<str> {
        let mut value: Option<Box<str>> = None;
        self.convert_to_string(&mut |s: &str| value = Some(s.into()));
        value.expect("msg")
    }
}

pub trait BooleanValue: Expression {
    fn get_value(&self) -> bool;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(if self.get_value() { "true" } else { "false" })
    }
}

pub trait IntegerValue: Expression {
    fn get_value(&self) -> i64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait DateTimeValue: Expression {
    fn get_value(&self) -> DateTime<FixedOffset>;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(
            &self
                .get_value()
                .to_rfc3339_opts(SecondsFormat::AutoSi, true),
        )
    }
}

pub trait DoubleValue: Expression {
    fn get_value(&self) -> f64;

    fn to_string(&self, action: &mut dyn FnMut(&str)) {
        (action)(&self.get_value().to_string())
    }
}

pub trait StringValue: Expression {
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
        let run_test_success = |value: Value, expected: &str| {
            let actual = Value::to_string(&value);

            assert_eq!(expected, actual.as_ref())
        };

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
    }

    #[test]
    pub fn test_are_values_equal() {
        let run_test_success =
            |left: Value, right: Value, case_insensitive: bool, expected: bool| {
                let actual = Value::are_values_equal(&left, &right, case_insensitive).unwrap();

                assert_eq!(expected, actual)
            };

        let run_test_failure =
            |left: Value, right: Value, case_insensitive: bool, expected: &str| {
                let actual = Value::are_values_equal(&left, &right, case_insensitive).unwrap_err();

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
    }

    #[test]
    pub fn test_compare_values() {
        let run_test_success = |left: Value, right: Value, expected: i64| {
            let actual = Value::compare_values(&left, &right).unwrap();

            assert_eq!(expected, actual)
        };

        let run_test_failure = |left: Value, right: Value, expected: &str| {
            let actual = Value::compare_values(&left, &right).unwrap_err();

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
