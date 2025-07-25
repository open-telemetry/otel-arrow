use std::cell::RefCell;

use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct DoubleValueData {
    value: f64,
    string_value: RefCell<Option<String>>,
}

impl DoubleValueData {
    pub fn new(value: f64) -> DoubleValueData {
        Self {
            value,
            string_value: RefCell::new(None),
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(&self.value.to_le_bytes());
    }

    pub(crate) fn compare(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<i32, Error> {
        if let AnyValue::DoubleValue(other_double_value) = other {
            return Ok(DoubleValueData::compare_values(
                self.value,
                other_double_value.value,
            ));
        } else if let AnyValue::LongValue(other_long_value) = other {
            return Ok(DoubleValueData::compare_values(
                self.value,
                other_long_value.get_value() as f64,
            ));
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = other_string_value.get_value().parse::<f64>();
            if let Err(e) = result {
                return Err(Error::ExpressionError(
                    expression_id,
                    Error::DoubleParseError(e).into(),
                ));
            }

            return Ok(DoubleValueData::compare_values(self.value, result.unwrap()));
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{other:?}' provided as right side of double compare expression could not be convered into a double")));

        Err(Error::new_expression_not_supported(
            expression_id,
            "AnyValue type on right side of compare expression is not supported",
        ))
    }

    pub(crate) fn compare_values(left: f64, right: f64) -> i32 {
        if left == right {
            0
        } else if left < right {
            return -1;
        } else {
            return 1;
        }
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::DoubleValue(other_double_value) = other {
            return self.value == other_double_value.value;
        } else if let AnyValue::LongValue(other_long_value) = other {
            return self.value == other_long_value.get_value() as f64;
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = other_string_value.get_value().parse::<f64>();
            if result.is_err() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of double equality expression could not be convered into a double: {}", other, result.unwrap_err())));

                return false;
            }

            return self.value == result.unwrap();
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{other:?}' provided as right side of double equality expression could not be convered into a double")));

        false
    }

    pub(crate) fn to_string<F>(&self, action: F)
    where
        F: FnOnce(&str),
    {
        let mut string_value = self.string_value.borrow_mut();

        if string_value.is_none() {
            *string_value = Some(self.value.to_string());
        }

        action(string_value.as_ref().unwrap())
    }
}
