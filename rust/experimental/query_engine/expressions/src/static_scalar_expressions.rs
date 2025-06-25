use chrono::{DateTime, FixedOffset};

use crate::{Expression, QueryLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum StaticScalarExpression {
    /// Resolve a static bool value provided directly in a query.
    Boolean(BooleanScalarExpression),

    /// Resolve a static DateTime value provided directly in a query.
    DateTime(DateTimeScalarExpression),

    /// Resolve a static double value provided directly in a query.
    Double(DoubleScalarExpression),

    /// Resolve a static integer value provided directly in a query.
    Integer(IntegerScalarExpression),

    /// Resolve a static string value provided directly in a query.
    String(StringScalarExpression),
}

impl StaticScalarExpression {
    pub fn to_value(&self) -> Value {
        match self {
            StaticScalarExpression::Boolean(b) => b.to_value(),
            StaticScalarExpression::DateTime(d) => d.to_value(),
            StaticScalarExpression::Double(d) => d.to_value(),
            StaticScalarExpression::Integer(i) => i.to_value(),
            StaticScalarExpression::String(s) => s.to_value(),
        }
    }
}

impl Expression for StaticScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            StaticScalarExpression::Boolean(b) => b.get_query_location(),
            StaticScalarExpression::DateTime(d) => d.get_query_location(),
            StaticScalarExpression::Double(d) => d.get_query_location(),
            StaticScalarExpression::Integer(i) => i.get_query_location(),
            StaticScalarExpression::String(s) => s.get_query_location(),
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

    pub fn get_value(&self) -> bool {
        self.value
    }

    pub fn to_value(&self) -> Value {
        Value::Boolean(self.get_value())
    }
}

impl Expression for BooleanScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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

    pub fn get_value(&self) -> DateTime<FixedOffset> {
        self.value
    }

    pub fn to_value(&self) -> Value {
        Value::DateTime(self.get_value())
    }
}

impl Expression for DateTimeScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn to_value(&self) -> Value {
        Value::Double(self.get_value())
    }
}

impl Expression for DoubleScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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

    pub fn get_value(&self) -> i64 {
        self.value
    }

    pub fn to_value(&self) -> Value {
        Value::Integer(self.get_value())
    }
}

impl Expression for IntegerScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn to_value(&self) -> Value {
        Value::String(self.get_value())
    }
}

impl Expression for StringScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

pub enum Value<'a> {
    Boolean(bool),
    Integer(i64),
    DateTime(DateTime<FixedOffset>),
    Double(f64),
    String(&'a str),
}
