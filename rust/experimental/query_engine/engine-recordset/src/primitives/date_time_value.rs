use chrono::{DateTime, FixedOffset};

use crate::{error::Error, execution_context::ExecutionContext, expression::*};

use super::any_value::AnyValue;

#[derive(Debug, Clone)]
pub struct DateTimeValueData {
    raw_value: Box<str>,
    value: DateTime<FixedOffset>,
}

impl DateTimeValueData {
    pub fn new_from_string(value: &str) -> Result<DateTimeValueData, Error> {
        let result = value.parse::<DateTime<FixedOffset>>();
        if let Err(error) = result {
            return Err(Error::DateTimeParseError(error));
        }

        Ok(Self {
            raw_value: value.into(),
            value: result.unwrap(),
        })
    }

    pub fn new(raw_value: &str, value: DateTime<FixedOffset>) -> DateTimeValueData {
        Self {
            raw_value: raw_value.into(),
            value,
        }
    }

    pub fn get_value(&self) -> DateTime<FixedOffset> {
        self.value
    }

    pub fn get_raw_value(&self) -> &str {
        &self.raw_value
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(self.get_raw_value().as_bytes());
    }

    pub(crate) fn compare(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<i32, Error> {
        if let AnyValue::DateTimeValue(other_date_time_value) = other {
            return Ok(compare_values(
                self.value,
                other_date_time_value.get_value(),
            ));
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = other_string_value
                .get_value()
                .parse::<DateTime<FixedOffset>>();
            if let Err(e) = result {
                return Err(Error::ExpressionError(
                    expression_id,
                    Error::DateTimeParseError(e).into(),
                ));
            }
            return Ok(compare_values(self.value, result.unwrap()));
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{:?}' provided as right side of date time compare expression could not be convered into a date time", other)));

        return Err(Error::new_expression_not_supported(
            expression_id,
            "AnyValue type on right side of compare expression is not supported",
        ));

        fn compare_values(left: DateTime<FixedOffset>, right: DateTime<FixedOffset>) -> i32 {
            match left.cmp(&right) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> bool {
        if let AnyValue::DateTimeValue(other_date_time_value) = other {
            return self.value == other_date_time_value.get_value();
        } else if let AnyValue::StringValue(other_string_value) = other {
            let result = other_string_value
                .get_value()
                .parse::<DateTime<FixedOffset>>();
            if result.is_err() {
                execution_context.add_message_for_expression_id(
                    expression_id,
                    ExpressionMessage::warn(
                        format!("AnyValue '{:?}' provided as right side of date time equality expression could not be convered into a date time: {}", other, result.unwrap_err())));

                return false;
            }

            return self.value == result.unwrap();
        }

        execution_context.add_message_for_expression_id(
            expression_id,
            ExpressionMessage::warn(
                format!("AnyValue '{:?}' provided as right side of date time equality expression could not be convered into a date time", other)));

        false
    }
}
