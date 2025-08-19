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

impl OwnedValue {
    pub fn from_json(
        query_location: &QueryLocation,
        input: &str,
    ) -> Result<OwnedValue, ExpressionError> {
        return match serde_json::from_str::<serde_json::Value>(input) {
            Ok(v) => from_value(query_location, v),
            Err(e) => Err(ExpressionError::ParseError(
                query_location.clone(),
                format!("Input could not be parsed as JSON: {e}"),
            )),
        };

        fn from_value(
            query_location: &QueryLocation,
            value: serde_json::Value,
        ) -> Result<OwnedValue, ExpressionError> {
            match value {
                serde_json::Value::Null => Ok(OwnedValue::Null),
                serde_json::Value::Bool(b) => Ok(OwnedValue::Boolean(BooleanValueStorage::new(b))),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(OwnedValue::Integer(IntegerValueStorage::new(i)))
                    } else {
                        match n
                            .as_f64()
                            .map(|f| OwnedValue::Double(DoubleValueStorage::new(f)))
                        {
                            Some(v) => Ok(v),
                            None => Err(ExpressionError::ParseError(
                                query_location.clone(),
                                format!("Input '{n}' could not be parsed as a number"),
                            )),
                        }
                    }
                }
                serde_json::Value::String(s) => Ok(OwnedValue::String(StringValueStorage::new(s))),
                serde_json::Value::Array(v) => {
                    let mut values = Vec::new();
                    for value in v {
                        values.push(from_value(query_location, value)?);
                    }
                    Ok(OwnedValue::Array(ArrayValueStorage::new(values)))
                }
                serde_json::Value::Object(m) => {
                    let mut values = HashMap::new();
                    for (key, value) in m {
                        values.insert(key.into(), from_value(query_location, value)?);
                    }
                    Ok(OwnedValue::Map(MapValueStorage::new(values)))
                }
            }
        }
    }
}

impl AsStaticValue for OwnedValue {
    fn to_static_value(&self) -> StaticValue<'_> {
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
    fn to_static_value_mut(&mut self) -> Option<StaticValueMut<'_>> {
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
            Value::Array(a) => OwnedValue::Array(a.into()),
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

impl From<&dyn ArrayValue> for ArrayValueStorage<OwnedValue> {
    fn from(value: &dyn ArrayValue) -> Self {
        let mut values = Vec::new();

        value.get_items(&mut IndexValueClosureCallback::new(|_, v| {
            values.push(v.into());
            true
        }));

        ArrayValueStorage::new(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_from_json() {
        let run_test = |input: &str| {
            let value = OwnedValue::from_json(&QueryLocation::new_fake(), input).unwrap();

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
