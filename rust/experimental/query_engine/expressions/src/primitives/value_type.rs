// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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

impl From<ValueType> for &str {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Array => "Array",
            ValueType::Boolean => "Boolean",
            ValueType::DateTime => "DateTime",
            ValueType::Double => "Double",
            ValueType::Integer => "Integer",
            ValueType::Map => "Map",
            ValueType::Null => "Null",
            ValueType::Regex => "Regex",
            ValueType::String => "String",
            ValueType::TimeSpan => "TimeSpan",
        }
    }
}
