use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct ArrayValueData {
    values: Vec<AnyValue>,
}

impl ArrayValueData {
    pub fn new(values: Vec<AnyValue>) -> ArrayValueData {
        Self { values }
    }

    pub fn get_values(&self) -> &Vec<AnyValue> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut Vec<AnyValue> {
        &mut self.values
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(&(self.values.len() as i32).to_le_bytes());
        for value in &self.values {
            value.add_hash_bytes(hasher);
        }
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<bool, Error> {
        if let AnyValue::ArrayValue(array_value) = other {
            let len = self.values.len();

            if len != array_value.values.len() {
                return Ok(false);
            }

            for i in 0..len {
                let left = &self.values[i];
                let right = &array_value.values[i];

                if !left.equals(execution_context, expression_id, right)? {
                    return Ok(false);
                }
            }

            return Ok(true);
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{:?}' provided as right side of array equality comparison was not an array", other)));

        return Ok(false);
    }

    pub(crate) fn contains(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<bool, Error> {
        if let AnyValue::ArrayValue(array_value) = other {
            for other_value in array_value.values.iter() {
                if !self.contains(execution_context, expression_id, other_value)? {
                    return Ok(false);
                }
            }

            return Ok(true);
        }

        for value in self.values.iter() {
            if value.equals(execution_context, expression_id, other)? {
                return Ok(true);
            }
        }

        return Ok(false);
    }
}
