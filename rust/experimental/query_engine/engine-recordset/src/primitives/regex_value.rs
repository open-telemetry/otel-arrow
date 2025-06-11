use regex::Regex;

use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct RegexValueData {
    regex: Regex,
}

impl RegexValueData {
    pub fn new_from_string(value: &str) -> Result<RegexValueData, Error> {
        let result = Regex::new(&value);
        if let Err(error) = result {
            return Err(Error::RegexParseError(error));
        }

        return Ok(Self {
            regex: result.unwrap(),
        });
    }

    pub fn new_from_regex(regex: Regex) -> RegexValueData {
        Self { regex }
    }

    pub fn get_regex(&self) -> &regex::Regex {
        &self.regex
    }

    pub fn get_pattern(&self) -> &str {
        self.regex.as_str()
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(&self.get_pattern().as_bytes());
    }

    pub(crate) fn matches(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::StringValue(other_string_value) = other {
            return self.regex.is_match(&other_string_value.get_value());
        }

        let mut result = false;

        other.as_string_value(|v| {
            match v {
                Some(s) => {
                    result = self.regex.is_match(s);
                },
                None => {
                    execution_context.add_message_for_expression_id(
                        expression_id,
                        ExpressionMessage::warn(
                            format!("AnyValue '{:?}' provided as right side of match comparison is not supported", other)));
                },
            }
        });

        return result;
    }
}
