use std::collections::HashMap;

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

impl From<Value<'_>> for OwnedValue {
    fn from(value: Value<'_>) -> Self {
        match value {
            Value::Array(a) => {
                let mut values = Vec::new();

                a.get_items(&mut IndexValueClosureCallback::new(|_, v| {
                    values.push(v.into());
                    true
                }));

                OwnedValue::Array(ArrayValueStorage::new(values))
            }
            Value::Boolean(b) => OwnedValue::Boolean(BooleanValueStorage::new(b.get_value())),
            Value::DateTime(d) => OwnedValue::DateTime(DateTimeValueStorage::new(d.get_value())),
            Value::Double(d) => OwnedValue::Double(DoubleValueStorage::new(d.get_value())),
            Value::Integer(i) => OwnedValue::Integer(IntegerValueStorage::new(i.get_value())),
            Value::Map(m) => {
                let mut values: HashMap<Box<str>, OwnedValue> = HashMap::new();

                m.get_items(&mut KeyValueClosureCallback::new(|k, v| {
                    values.insert(k.into(), v.into());
                    true
                }));

                OwnedValue::Map(MapValueStorage::new(values))
            }
            Value::Null => OwnedValue::Null,
            Value::Regex(r) => OwnedValue::Regex(RegexValueStorage::new(r.get_value().clone())),
            Value::String(s) => OwnedValue::String(StringValueStorage::new(s.get_value().into())),
        }
    }
}
