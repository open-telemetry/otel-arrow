use crate::{execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct StringValueData {
    value: Box<str>,
}

impl StringValueData {
    pub fn new(value: &str) -> StringValueData {
        Self {
            value: value.into(),
        }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut str {
        &mut self.value
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(self.value.as_bytes());
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::StringValue(other_string_value) = other {
            return self.value.eq(&other_string_value.value);
        }

        let mut result = false;

        other.as_string_value(|other_value: Option<&str>| {
            if other_value.is_none() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of string equality expression could not be convered into a string", other)));
            }
            else {
                let self_value: &str = &self.value;
                result = self_value.eq(other_value.unwrap())
            }
        });

        result
    }
}
