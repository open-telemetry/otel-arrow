use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, FixedOffset};

use crate::{error::Error, execution_context::ExecutionContext, expression::Hasher};

use super::{
    array_value::ArrayValueData, bool_value::BoolValueData, date_time_value::DateTimeValueData,
    double_value::DoubleValueData, json_value::JsonValueData, long_value::LongValueData,
    map_value::MapValueData, regex_value::RegexValueData, string_value::StringValueData,
    xml_value::XmlValueData,
};

#[derive(Debug, Clone, Default)]
pub enum AnyValue {
    ArrayValue(ArrayValueData),
    BoolValue(BoolValueData),
    DateTimeValue(DateTimeValueData),
    DoubleValue(DoubleValueData),
    JsonValue(JsonValueData),
    LongValue(LongValueData),
    MapValue(MapValueData),
    #[default]
    NullValue,
    RegexValue(RegexValueData),
    StringValue(StringValueData),
    XmlValue(XmlValueData),
}

impl AnyValue {
    pub(crate) fn compare(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<i32, Error> {
        match self {
            AnyValue::ArrayValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "ArrayValue type on left side of compare expression is not supported",
            )),
            AnyValue::BoolValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "BoolValue type on left side of compare expression is not supported",
            )),
            AnyValue::DateTimeValue(date_time_value) => {
                date_time_value.compare(execution_context, expression_id, other)
            }
            AnyValue::DoubleValue(double_value) => {
                double_value.compare(execution_context, expression_id, other)
            }
            AnyValue::JsonValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "JsonValue type on left side of compare expression is not supported",
            )),
            AnyValue::LongValue(long_value) => {
                long_value.compare(execution_context, expression_id, other)
            }
            AnyValue::MapValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "MapValue type on left side of compare expression is not supported",
            )),
            AnyValue::NullValue => Err(Error::new_expression_not_supported(
                expression_id,
                "NullValue type on left side of compare expression is not supported",
            )),
            AnyValue::RegexValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "RegexValue type on left side of compare expression is not supported",
            )),
            AnyValue::StringValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "StringValue type on left side of compare expression is not supported",
            )),
            AnyValue::XmlValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "XmlValue type on left side of compare expression is not supported",
            )),
        }
    }

    pub(crate) fn equals(
        &self,
        execution_context: &dyn ExecutionContext,
        expression_id: usize,
        other: &AnyValue,
    ) -> Result<bool, Error> {
        match self {
            AnyValue::NullValue => Ok(other.is_null()),
            AnyValue::ArrayValue(array_value) => {
                array_value.equals(execution_context, expression_id, other)
            }
            AnyValue::BoolValue(bool_value) => {
                Ok(bool_value.equals(execution_context, expression_id, other))
            }
            AnyValue::DateTimeValue(date_time_value) => {
                Ok(date_time_value.equals(execution_context, expression_id, other))
            }
            AnyValue::DoubleValue(double_value) => {
                Ok(double_value.equals(execution_context, expression_id, other))
            }
            AnyValue::JsonValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "JsonValue type on left side of equality expression is not supported",
            )),
            AnyValue::LongValue(long_value) => {
                Ok(long_value.equals(execution_context, expression_id, other))
            }
            AnyValue::MapValue(map_value) => {
                map_value.equals(execution_context, expression_id, other)
            }
            AnyValue::RegexValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "RegexValue type on left side of equality expression is not supported",
            )),
            AnyValue::StringValue(string_value) => {
                Ok(string_value.equals(execution_context, expression_id, other))
            }
            AnyValue::XmlValue(_) => Err(Error::new_expression_not_supported(
                expression_id,
                "XmlValue type on left side of equality expression is not supported",
            )),
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        if let AnyValue::NullValue = self {
            return true;
        }

        false
    }

    pub(crate) fn as_string_value<F>(&self, action: F)
    where
        F: FnOnce(Option<&str>),
    {
        match self {
            AnyValue::ArrayValue(_) => action(None),
            AnyValue::BoolValue(v) => action(Some(v.to_string())),
            AnyValue::DateTimeValue(v) => action(Some(v.get_raw_value())),
            AnyValue::DoubleValue(v) => v.to_string(|v| action(Some(v))),
            AnyValue::JsonValue(v) => action(Some(v.get_raw_value())),
            AnyValue::LongValue(v) => v.to_string(|v| action(Some(v))),
            AnyValue::MapValue(_) => action(None),
            AnyValue::NullValue => action(None),
            AnyValue::RegexValue(v) => action(Some(v.get_pattern())),
            AnyValue::StringValue(v) => action(Some(v.get_value())),
            AnyValue::XmlValue(v) => action(Some(v.get_raw_value())),
        };
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        match self {
            AnyValue::ArrayValue(array_value) => array_value.add_hash_bytes(hasher),
            AnyValue::BoolValue(bool_value) => bool_value.add_hash_bytes(hasher),
            AnyValue::DateTimeValue(date_time_value) => date_time_value.add_hash_bytes(hasher),
            AnyValue::DoubleValue(double_value) => double_value.add_hash_bytes(hasher),
            AnyValue::JsonValue(json_value) => json_value.add_hash_bytes(hasher),
            AnyValue::LongValue(long_value) => long_value.add_hash_bytes(hasher),
            AnyValue::MapValue(map_value) => map_value.add_hash_bytes(hasher),
            AnyValue::NullValue => hasher.add_bytes(&[0xFF]),
            AnyValue::RegexValue(regex_value) => regex_value.add_hash_bytes(hasher),
            AnyValue::StringValue(string_value) => string_value.add_hash_bytes(hasher),
            AnyValue::XmlValue(xml_value) => xml_value.add_hash_bytes(hasher),
        }
    }

    pub(crate) fn write_debug<T: Debug>(
        value: &T,
        heading: &'static str,
        level: i32,
        output: &mut String,
    ) {
        let padding = "\t".repeat(level as usize);

        output.push_str(&padding);
        output.push_str(heading);
        output.push_str(format!("{value:?}").as_str());
        output.push('\n');
    }

    pub fn new_string_value(value: &str) -> AnyValue {
        AnyValue::StringValue(StringValueData::new(value))
    }

    pub fn new_array_value(values: Vec<AnyValue>) -> AnyValue {
        AnyValue::ArrayValue(ArrayValueData::new(values))
    }

    pub fn new_map_value(values: HashMap<String, AnyValue>) -> AnyValue {
        AnyValue::MapValue(MapValueData::new(values))
    }

    pub fn new_date_time_value(raw_value: &str, value: DateTime<FixedOffset>) -> AnyValue {
        AnyValue::DateTimeValue(DateTimeValueData::new(raw_value, value))
    }

    pub fn new_double_value(value: f64) -> AnyValue {
        AnyValue::DoubleValue(DoubleValueData::new(value))
    }

    pub fn new_long_value(value: i64) -> AnyValue {
        AnyValue::LongValue(LongValueData::new(value))
    }

    pub fn new_bool_value(value: bool) -> AnyValue {
        AnyValue::BoolValue(BoolValueData::new(value))
    }

    pub fn get_string_value(&self) -> Option<&str> {
        if let AnyValue::StringValue(string_value) = self {
            return Some(string_value.get_value());
        }

        None
    }

    pub fn get_long_value(&self) -> Option<i64> {
        if let AnyValue::LongValue(long_value) = self {
            return Some(long_value.get_value());
        }

        None
    }

    pub fn get_double_value(&self) -> Option<f64> {
        if let AnyValue::DoubleValue(double_value) = self {
            return Some(double_value.get_value());
        }

        None
    }

    pub fn get_bool_value(&self) -> Option<bool> {
        if let AnyValue::BoolValue(bool_value) = self {
            return Some(bool_value.get_value());
        }

        None
    }

    pub fn get_array_value(&self) -> Option<&Vec<AnyValue>> {
        if let AnyValue::ArrayValue(array_value) = self {
            return Some(array_value.get_values());
        }

        None
    }

    pub fn get_map_value(&self) -> Option<&HashMap<String, AnyValue>> {
        if let AnyValue::MapValue(map_value) = self {
            return Some(map_value.get_values());
        }

        None
    }
}
