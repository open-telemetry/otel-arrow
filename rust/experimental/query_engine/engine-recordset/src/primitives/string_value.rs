use std::str::FromStr;

use regex::Regex;

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

    pub(crate) fn truncate_value(&mut self, max_chars: usize) {
        match self.value.char_indices().nth(max_chars) {
            None => {}
            Some((idx, _)) => self.value = (self.value[..idx]).into(),
        }
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(&self.value.as_bytes());
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

        return result;
    }

    pub(crate) fn contains(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::StringValue(other_string_value) = other {
            return (&self.value as &str).contains(&other_string_value.value as &str);
        } else if let AnyValue::ArrayValue(other_array_value) = other {
            for other_value in other_array_value.get_values().iter() {
                if !self.contains(execution_context, expression_id, other_value) {
                    return false;
                }
            }
            return true;
        }

        let mut result = false;

        other.as_string_value(|v: Option<&str>| {
            if v.is_none() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of string contains expression could not be convered into a string", other)));
            }
            else {
                result = self.value.contains(v.clone().unwrap())
            }
        });

        return result;
    }

    pub(crate) fn matches(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::RegexValue(other_regex_value) = other {
            return other_regex_value.get_regex().is_match(&self.value);
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = Regex::from_str(&other_string_value.value);
            if result.is_err() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of match expression could not be convered into a regex: {}", other, result.unwrap_err())));

                return false;
            }
            return result.unwrap().is_match(&self.value);
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{:?}' provided as right side of match expression could not be convered into a regex", other)));

        return false;
    }
}
