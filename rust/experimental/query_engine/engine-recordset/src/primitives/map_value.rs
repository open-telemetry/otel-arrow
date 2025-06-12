use std::collections::HashMap;

use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::*;

#[derive(Debug, Clone)]
pub struct MapValueData {
    values: HashMap<String, AnyValue>,
}

impl MapValueData {
    pub fn new(values: HashMap<String, AnyValue>) -> MapValueData {
        Self { values }
    }

    pub fn get_values(&self) -> &HashMap<String, AnyValue> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut HashMap<String, AnyValue> {
        &mut self.values
    }

    pub fn get(&self, key: &str) -> Option<&AnyValue> {
        self.values.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(any_value) => any_value.get_string_value(),
            None => None,
        }
    }

    pub fn get_long(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(any_value) => any_value.get_long_value(),
            None => None,
        }
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(any_value) => any_value.get_double_value(),
            None => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(any_value) => any_value.get_bool_value(),
            None => None,
        }
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<AnyValue>> {
        match self.get(key) {
            Some(any_value) => any_value.get_array_value(),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut AnyValue> {
        self.values.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<AnyValue> {
        self.values.remove(key)
    }

    pub fn insert(&mut self, key: &str, value: AnyValue) -> Option<AnyValue> {
        self.values.insert(key.to_string(), value)
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(&(self.values.len() as i32).to_le_bytes());
        for value in &self.values {
            hasher.add_bytes(value.0.as_bytes());
            value.1.add_hash_bytes(hasher);
        }
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<bool, Error> {
        if let AnyValue::MapValue(map_value) = other {
            let len = self.values.len();

            if len != map_value.values.len() {
                return Ok(false);
            }

            for kvp in self.values.iter() {
                let value = map_value.values.get(kvp.0);
                if value.is_none() {
                    return Ok(false);
                }

                let any_value = value.unwrap();

                if !kvp.1.equals(execution_context, expression_id, any_value)? {
                    return Ok(false);
                }
            }

            return Ok(true);
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(format!(
                "AnyValue '{:?}' provided as right side of map equality comparison was not a map",
                other
            )),
        );

        Ok(false)
    }
}
