// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, TimeDelta};
use regex::Regex;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StaticScalarExpression {
    /// Resolve a static array value provided directly in a query.
    Array(ArrayScalarExpression),

    /// Resolve a static bool value provided directly in a query.
    Boolean(BooleanScalarExpression),

    /// Resolve a static DateTime value provided directly in a query.
    DateTime(DateTimeScalarExpression),

    /// Resolve a static double value provided directly in a query.
    Double(DoubleScalarExpression),

    /// Resolve a static integer value provided directly in a query.
    Integer(IntegerScalarExpression),

    /// Resolve a static map value provided directly in a query.
    Map(MapScalarExpression),

    /// Resolve a static null value provided directly in a query.
    Null(NullScalarExpression),

    /// Resolve a static regex value provided directly in a query.
    Regex(RegexScalarExpression),

    /// Resolve a static string value provided directly in a query.
    String(StringScalarExpression),

    /// Resolve a static TimeSpan value provided directly in a query.
    TimeSpan(TimeSpanScalarExpression),
}

impl StaticScalarExpression {
    pub(crate) fn try_fold(&self) -> Option<StaticScalarExpression> {
        // Note: The goal here is to diferentiate which statics can be
        // folded/copied in the expression tree and which ones should always be
        // referenced.
        match self {
            StaticScalarExpression::Array(_) => None,
            StaticScalarExpression::Boolean(b) => Some(StaticScalarExpression::Boolean(b.clone())),
            StaticScalarExpression::DateTime(d) => {
                Some(StaticScalarExpression::DateTime(d.clone()))
            }
            StaticScalarExpression::Double(d) => Some(StaticScalarExpression::Double(d.clone())),
            StaticScalarExpression::Integer(i) => Some(StaticScalarExpression::Integer(i.clone())),
            StaticScalarExpression::Map(_) => None,
            StaticScalarExpression::Null(n) => Some(StaticScalarExpression::Null(n.clone())),
            StaticScalarExpression::Regex(_) => None,
            StaticScalarExpression::String(s) => {
                let value = &s.value;
                if value.len() <= 32 {
                    Some(StaticScalarExpression::String(s.clone()))
                } else {
                    None
                }
            }
            StaticScalarExpression::TimeSpan(t) => {
                Some(StaticScalarExpression::TimeSpan(t.clone()))
            }
        }
    }

    pub fn from_json(
        query_location: QueryLocation,
        input: &str,
    ) -> Result<StaticScalarExpression, ExpressionError> {
        return match serde_json::from_str::<serde_json::Value>(input) {
            Ok(v) => Ok(from_value(&query_location, v)?),
            Err(e) => Err(ExpressionError::ParseError(
                query_location,
                format!("Input could not be parsed as JSON: {e}"),
            )),
        };

        fn from_value(
            query_location: &QueryLocation,
            value: serde_json::Value,
        ) -> Result<StaticScalarExpression, ExpressionError> {
            match value {
                serde_json::Value::Null => Ok(StaticScalarExpression::Null(
                    NullScalarExpression::new(query_location.clone()),
                )),
                serde_json::Value::Bool(b) => Ok(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(query_location.clone(), b),
                )),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(query_location.clone(), i),
                        ))
                    } else {
                        match n.as_f64().map(|f| {
                            StaticScalarExpression::Double(DoubleScalarExpression::new(
                                query_location.clone(),
                                f,
                            ))
                        }) {
                            Some(s) => Ok(s),
                            None => Err(ExpressionError::ParseError(
                                query_location.clone(),
                                format!("Input '{n}' could not be parsed as a number"),
                            )),
                        }
                    }
                }
                serde_json::Value::String(s) => Ok(StaticScalarExpression::String(
                    StringScalarExpression::new(query_location.clone(), &s),
                )),
                serde_json::Value::Array(v) => {
                    let mut values = Vec::new();
                    for value in v {
                        values.push(from_value(query_location, value)?);
                    }
                    Ok(StaticScalarExpression::Array(ArrayScalarExpression::new(
                        query_location.clone(),
                        values,
                    )))
                }
                serde_json::Value::Object(m) => {
                    let mut values = HashMap::new();
                    for (key, value) in m {
                        values.insert(key.into(), from_value(query_location, value)?);
                    }
                    Ok(StaticScalarExpression::Map(MapScalarExpression::new(
                        query_location.clone(),
                        values,
                    )))
                }
            }
        }
    }
}

impl Expression for StaticScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            StaticScalarExpression::Array(a) => a.get_query_location(),
            StaticScalarExpression::Boolean(b) => b.get_query_location(),
            StaticScalarExpression::DateTime(d) => d.get_query_location(),
            StaticScalarExpression::Double(d) => d.get_query_location(),
            StaticScalarExpression::Integer(i) => i.get_query_location(),
            StaticScalarExpression::Map(m) => m.get_query_location(),
            StaticScalarExpression::Null(n) => n.get_query_location(),
            StaticScalarExpression::Regex(r) => r.get_query_location(),
            StaticScalarExpression::String(s) => s.get_query_location(),
            StaticScalarExpression::TimeSpan(t) => t.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            StaticScalarExpression::Array(_) => "StaticScalar(Array)",
            StaticScalarExpression::Boolean(_) => "StaticScalar(Boolean)",
            StaticScalarExpression::DateTime(_) => "StaticScalar(DateTime)",
            StaticScalarExpression::Double(_) => "StaticScalar(Double)",
            StaticScalarExpression::Integer(_) => "StaticScalar(Integer)",
            StaticScalarExpression::Map(_) => "StaticScalar(Map)",
            StaticScalarExpression::Null(_) => "StaticScalar(Null)",
            StaticScalarExpression::String(_) => "StaticScalar(String)",
            StaticScalarExpression::Regex(_) => "StaticScalar(Regex)",
            StaticScalarExpression::TimeSpan(_) => "StaticScalar(TimeSpan)",
        }
    }
}

impl AsStaticValue for StaticScalarExpression {
    fn to_static_value(&self) -> StaticValue<'_> {
        match self {
            StaticScalarExpression::Array(a) => StaticValue::Array(a),
            StaticScalarExpression::Boolean(b) => StaticValue::Boolean(b),
            StaticScalarExpression::DateTime(d) => StaticValue::DateTime(d),
            StaticScalarExpression::Double(d) => StaticValue::Double(d),
            StaticScalarExpression::Integer(i) => StaticValue::Integer(i),
            StaticScalarExpression::Map(m) => StaticValue::Map(m),
            StaticScalarExpression::Null(_) => StaticValue::Null,
            StaticScalarExpression::Regex(r) => StaticValue::Regex(r),
            StaticScalarExpression::String(s) => StaticValue::String(s),
            StaticScalarExpression::TimeSpan(t) => StaticValue::TimeSpan(t),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanScalarExpression {
    query_location: QueryLocation,
    value: bool,
}

impl BooleanScalarExpression {
    pub fn new(query_location: QueryLocation, value: bool) -> BooleanScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for BooleanScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "BooleanScalarExpression"
    }
}

impl BooleanValue for BooleanScalarExpression {
    fn get_value(&self) -> bool {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTimeScalarExpression {
    query_location: QueryLocation,
    value: DateTime<FixedOffset>,
}

impl DateTimeScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value: DateTime<FixedOffset>,
    ) -> DateTimeScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for DateTimeScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "DateTimeScalarExpression"
    }
}

impl DateTimeValue for DateTimeScalarExpression {
    fn get_value(&self) -> DateTime<FixedOffset> {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleScalarExpression {
    query_location: QueryLocation,
    value: f64,
}

impl DoubleScalarExpression {
    pub fn new(query_location: QueryLocation, value: f64) -> DoubleScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for DoubleScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "DoubleScalarExpression"
    }
}

impl DoubleValue for DoubleScalarExpression {
    fn get_value(&self) -> f64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerScalarExpression {
    query_location: QueryLocation,
    value: i64,
}

impl IntegerScalarExpression {
    pub fn new(query_location: QueryLocation, value: i64) -> IntegerScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for IntegerScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "IntegerScalarExpression"
    }
}

impl IntegerValue for IntegerScalarExpression {
    fn get_value(&self) -> i64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullScalarExpression {
    query_location: QueryLocation,
}

impl NullScalarExpression {
    pub fn new(query_location: QueryLocation) -> NullScalarExpression {
        Self { query_location }
    }
}

impl Expression for NullScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "NullScalarExpression"
    }
}

#[derive(Debug, Clone)]
pub struct RegexScalarExpression {
    query_location: QueryLocation,
    value: Regex,
}

impl RegexScalarExpression {
    pub fn new(query_location: QueryLocation, value: Regex) -> RegexScalarExpression {
        Self {
            query_location,
            value,
        }
    }

    pub fn get_value(&self) -> &Regex {
        &self.value
    }
}

impl Expression for RegexScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "RegexScalarExpression"
    }
}

impl RegexValue for RegexScalarExpression {
    fn get_value(&self) -> &Regex {
        &self.value
    }
}

impl PartialEq for RegexScalarExpression {
    fn eq(&self, other: &Self) -> bool {
        self.query_location == other.query_location && self.value.as_str() == other.value.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringScalarExpression {
    query_location: QueryLocation,
    value: Box<str>,
}

impl StringScalarExpression {
    pub fn new(query_location: QueryLocation, value: &str) -> StringScalarExpression {
        Self {
            query_location,
            value: value.into(),
        }
    }
}

impl Expression for StringScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "StringScalarExpression"
    }
}

impl StringValue for StringScalarExpression {
    fn get_value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSpanScalarExpression {
    query_location: QueryLocation,
    value: TimeDelta,
}

impl TimeSpanScalarExpression {
    pub fn new(query_location: QueryLocation, value: TimeDelta) -> TimeSpanScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for TimeSpanScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "TimeSpanScalarExpression"
    }
}

impl TimeSpanValue for TimeSpanScalarExpression {
    fn get_value(&self) -> TimeDelta {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_from_json() {
        let run_test = |input: &str| {
            let value =
                StaticScalarExpression::from_json(QueryLocation::new_fake(), input).unwrap();

            assert_eq!(input, value.to_value().to_string());
        };

        run_test("true");
        run_test("false");
        run_test("18");
        run_test("18.18");
        run_test("null");
        run_test("[]");
        run_test("[1,\"two\",null]");
        run_test("{}");
        run_test("{\"key1\":1,\"key2\":\"two\",\"key3\":null}");
    }
}
