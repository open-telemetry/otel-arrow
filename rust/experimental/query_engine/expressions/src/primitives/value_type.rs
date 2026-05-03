// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// representations for stringified value types:
const VALUE_TYPE_AS_STR_ARRAY: &str = "Array";
const VALUE_TYPE_AS_STR_BOOLEAN: &str = "Boolean";
const VALUE_TYPE_AS_STR_DATETIME: &str = "DateTime";
const VALUE_TYPE_AS_STR_DOUBLE: &str = "Double";
const VALUE_TYPE_AS_STR_INTEGER: &str = "Integer";
const VALUE_TYPE_AS_STR_MAP: &str = "Map";
const VALUE_TYPE_AS_STR_NULL: &str = "Null";
const VALUE_TYPE_AS_STR_REGEX: &str = "Regex";
const VALUE_TYPE_AS_STR_STRING: &str = "String";
const VALUE_TYPE_AS_STR_TIMESPAN: &str = "TimeSpan";

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Array = 0,
    Boolean = 1,
    DateTime = 2,
    Double = 3,
    Integer = 4,
    Map = 5,
    Null = 6,
    Regex = 7,
    String = 8,
    TimeSpan = 9,
}

impl ValueType {
    pub fn get_value_types() -> impl Iterator<Item = ValueType> {
        // Note: Order here must match the enum definition
        static VARIANTS: [ValueType; 10] = [
            ValueType::Array,
            ValueType::Boolean,
            ValueType::DateTime,
            ValueType::Double,
            ValueType::Integer,
            ValueType::Map,
            ValueType::Null,
            ValueType::Regex,
            ValueType::String,
            ValueType::TimeSpan,
        ];
        VARIANTS.iter().cloned()
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = self.clone().into();
        write!(f, "{s}")
    }
}

impl ValueType {
    /// Try to parse the [`ValueType`] from a string. Returns None if the string does not represent
    /// a known variant of this enum.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            VALUE_TYPE_AS_STR_ARRAY => ValueType::Array,
            VALUE_TYPE_AS_STR_BOOLEAN => ValueType::Boolean,
            VALUE_TYPE_AS_STR_DATETIME => ValueType::DateTime,
            VALUE_TYPE_AS_STR_DOUBLE => ValueType::Double,
            VALUE_TYPE_AS_STR_INTEGER => ValueType::Integer,
            VALUE_TYPE_AS_STR_MAP => ValueType::Map,
            VALUE_TYPE_AS_STR_NULL => ValueType::Null,
            VALUE_TYPE_AS_STR_REGEX => ValueType::Regex,
            VALUE_TYPE_AS_STR_STRING => ValueType::String,
            VALUE_TYPE_AS_STR_TIMESPAN => ValueType::TimeSpan,
            _ => return None,
        })
    }
}

impl From<ValueType> for &str {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Array => VALUE_TYPE_AS_STR_ARRAY,
            ValueType::Boolean => VALUE_TYPE_AS_STR_BOOLEAN,
            ValueType::DateTime => VALUE_TYPE_AS_STR_DATETIME,
            ValueType::Double => VALUE_TYPE_AS_STR_DOUBLE,
            ValueType::Integer => VALUE_TYPE_AS_STR_INTEGER,
            ValueType::Map => VALUE_TYPE_AS_STR_MAP,
            ValueType::Null => VALUE_TYPE_AS_STR_NULL,
            ValueType::Regex => VALUE_TYPE_AS_STR_REGEX,
            ValueType::String => VALUE_TYPE_AS_STR_STRING,
            ValueType::TimeSpan => VALUE_TYPE_AS_STR_TIMESPAN,
        }
    }
}
