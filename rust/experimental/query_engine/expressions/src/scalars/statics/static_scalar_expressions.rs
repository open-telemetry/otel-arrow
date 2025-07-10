use chrono::{DateTime, FixedOffset};
use regex::Regex;

use crate::{ArrayScalarExpression, Expression, MapScalarExpression, QueryLocation, primitives::*};

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
}

impl StaticScalarExpression {
    pub fn get_value_type(&self) -> ValueType {
        match self {
            StaticScalarExpression::Array(_) => ValueType::Array,
            StaticScalarExpression::Boolean(_) => ValueType::Boolean,
            StaticScalarExpression::DateTime(_) => ValueType::DateTime,
            StaticScalarExpression::Double(_) => ValueType::Double,
            StaticScalarExpression::Integer(_) => ValueType::Integer,
            StaticScalarExpression::Map(_) => ValueType::Map,
            StaticScalarExpression::Null(_) => ValueType::Null,
            StaticScalarExpression::Regex(_) => ValueType::Regex,
            StaticScalarExpression::String(_) => ValueType::String,
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            StaticScalarExpression::Array(a) => Value::Array(a),
            StaticScalarExpression::Boolean(b) => Value::Boolean(b),
            StaticScalarExpression::DateTime(d) => Value::DateTime(d),
            StaticScalarExpression::Double(d) => Value::Double(d),
            StaticScalarExpression::Integer(i) => Value::Integer(i),
            StaticScalarExpression::Map(m) => Value::Map(m),
            StaticScalarExpression::Null(_) => Value::Null,
            StaticScalarExpression::Regex(r) => Value::Regex(r),
            StaticScalarExpression::String(s) => Value::String(s),
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
