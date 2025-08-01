use chrono::{DateTime, FixedOffset};
use data_engine_expressions::*;
use regex::Regex;

use crate::*;

#[derive(Debug, Clone)]
pub enum OwnedValue {
    Array(ArrayValueStorage<OwnedValue>),
    Boolean(ValueStorage<bool>),
    DateTime(ValueStorage<DateTime<FixedOffset>>),
    Double(ValueStorage<f64>),
    Integer(ValueStorage<i64>),
    Map(MapValueStorage<OwnedValue>),
    Null,
    Regex(ValueStorage<Regex>),
    String(ValueStorage<String>),
}

impl AsValue for OwnedValue {
    fn get_value_type(&self) -> ValueType {
        match self {
            OwnedValue::Array(_) => ValueType::Array,
            OwnedValue::Boolean(_) => ValueType::Boolean,
            OwnedValue::DateTime(_) => ValueType::DateTime,
            OwnedValue::Double(_) => ValueType::Double,
            OwnedValue::Integer(_) => ValueType::Integer,
            OwnedValue::Map(_) => ValueType::Map,
            OwnedValue::Null => ValueType::Null,
            OwnedValue::Regex(_) => ValueType::Regex,
            OwnedValue::String(_) => ValueType::String,
        }
    }

    fn to_value(&self) -> Value {
        match self {
            OwnedValue::Array(a) => Value::Array(a),
            OwnedValue::Boolean(b) => Value::Boolean(b),
            OwnedValue::DateTime(d) => Value::DateTime(d),
            OwnedValue::Double(v) => Value::Double(v),
            OwnedValue::Integer(v) => Value::Integer(v),
            OwnedValue::Map(m) => Value::Map(m),
            OwnedValue::Null => Value::Null,
            OwnedValue::Regex(r) => Value::Regex(r),
            OwnedValue::String(s) => Value::String(s),
        }
    }
}

impl AsValueMut for OwnedValue {
    fn to_value_mut(&mut self) -> Option<ValueMut> {
        match self {
            OwnedValue::Array(a) => Some(ValueMut::Array(a)),
            OwnedValue::Map(m) => Some(ValueMut::Map(m)),
            OwnedValue::String(s) => Some(ValueMut::String(s)),
            _ => None,
        }
    }
}

impl ValueSource<OwnedValue> for OwnedValue {
    fn from_owned(value: OwnedValue) -> OwnedValue {
        value
    }

    fn to_owned(self) -> OwnedValue {
        self
    }
}
