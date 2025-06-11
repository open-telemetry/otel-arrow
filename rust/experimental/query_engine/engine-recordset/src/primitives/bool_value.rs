use crate::{execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct BoolValueData {
    value: bool,
}

impl BoolValueData {
    pub fn new(value: bool) -> BoolValueData {
        Self { value }
    }

    pub fn get_value(&self) -> bool {
        self.value
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(either!(self.value => &[1]; &[0]));
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::BoolValue(other_bool_value) = other {
            return self.value == other_bool_value.value;
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = other_string_value.get_value().parse::<bool>();
            if result.is_err() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of bool equality expression could not be convered into a bool: {}", other, result.unwrap_err())));

                return false;
            }

            return self.value == result.unwrap();
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{:?}' provided as right side of bool equality comparison could not be convered into a bool", other)));

        return false;
    }

    pub(crate) fn to_string(&self) -> &str {
        return either!(self.value => "true"; "false");
    }
}
