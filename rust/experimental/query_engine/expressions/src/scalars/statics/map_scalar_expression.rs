use std::collections::HashMap;

use crate::{
    AsValue, Expression, KeyValueCallback, MapValue, QueryLocation, StaticScalarExpression, Value,
    ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MapScalarExpression {
    query_location: QueryLocation,
    value: HashMap<Box<str>, StaticScalarExpression>,
}

impl MapScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value: HashMap<Box<str>, StaticScalarExpression>,
    ) -> MapScalarExpression {
        Self {
            query_location,
            value,
        }
    }
}

impl Expression for MapScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MapScalarExpression"
    }
}

impl AsValue for MapScalarExpression {
    fn get_value_type(&self) -> ValueType {
        ValueType::Map
    }

    fn to_value(&self) -> Value {
        Value::Map(self)
    }
}

impl MapValue for MapScalarExpression {
    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn len(&self) -> usize {
        self.value.len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.value.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&(dyn AsValue + 'static)> {
        self.value.get(key).map(|v| v as &dyn AsValue)
    }

    fn get_items(&self, item_callback: &mut dyn KeyValueCallback) -> bool {
        for (key, value) in &self.value {
            if !item_callback.next(key, value.to_value()) {
                return false;
            }
        }

        true
    }
}
