use data_engine_expressions::*;

use crate::*;

#[derive(Debug, Clone)]
pub enum OwnedValue {
    Array(ArrayValueStorage<OwnedValue>),
    Boolean(BooleanValueStorage),
    DateTime(DateTimeValueStorage),
    Double(DoubleValueStorage<f64>),
    Integer(IntegerValueStorage<i64>),
    Map(MapValueStorage<OwnedValue>),
    Null,
    Regex(RegexValueStorage),
    String(StringValueStorage),
}

impl AsStaticValue for OwnedValue {
    fn to_static_value(&self) -> StaticValue {
        match self {
            OwnedValue::Array(a) => StaticValue::Array(a),
            OwnedValue::Boolean(b) => StaticValue::Boolean(b),
            OwnedValue::DateTime(d) => StaticValue::DateTime(d),
            OwnedValue::Double(d) => StaticValue::Double(d),
            OwnedValue::Integer(i) => StaticValue::Integer(i),
            OwnedValue::Map(m) => StaticValue::Map(m),
            OwnedValue::Null => StaticValue::Null,
            OwnedValue::Regex(r) => StaticValue::Regex(r),
            OwnedValue::String(s) => StaticValue::String(s),
        }
    }
}

impl AsStaticValueMut for OwnedValue {
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut> {
        match self {
            OwnedValue::Array(a) => Some(StaticValueMut::Array(a)),
            OwnedValue::Map(m) => Some(StaticValueMut::Map(m)),
            OwnedValue::String(s) => Some(StaticValueMut::String(s)),
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
