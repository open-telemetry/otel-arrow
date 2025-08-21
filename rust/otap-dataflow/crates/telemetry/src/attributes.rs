// SPDX-License-Identifier: Apache-2.0

//! Interface defining a collection of attributes (pairs of key -> value) associated with a
//! [`metrics::MetricSet`].

use crate::descriptor::{AttributesDescriptor, AttributeValueType};

/// Trait implemented by structs representing a set of attributes.
pub trait AttributeSetHandler {
    /// Returns the static descriptor describing this attribute set.
    fn descriptor(&self) -> &'static AttributesDescriptor;

    /// Returns an iterator over key-value pairs of all attributes in this set.
    fn iter_attributes<'a>(&'a self) -> Box<dyn Iterator<Item = (&'static str, AttributeValue)> + 'a>;
}

/// Represents a single attribute value that can be of different types.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// String attribute value
    String(String),
    /// Signed integer attribute value
    Int(i64),
    /// Unsigned integer attribute value
    UInt(u64),
    /// Double-precision floating-point attribute value
    Double(f64),
    /// Boolean attribute value
    Boolean(bool),
}

impl AttributeValue {
    /// Returns the value type of this attribute value.
    pub fn value_type(&self) -> AttributeValueType {
        match self {
            AttributeValue::String(_) => AttributeValueType::String,
            AttributeValue::Int(_) => AttributeValueType::Int,
            // Semantic Convention: UInt is treated as Int
            AttributeValue::UInt(_) => AttributeValueType::Int, 
            AttributeValue::Double(_) => AttributeValueType::Double,
            AttributeValue::Boolean(_) => AttributeValueType::Boolean,
        }
    }

    /// Converts the attribute value to a string representation for serialization.
    pub fn to_string_value(&self) -> String {
        match self {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Int(i) => i.to_string(),
            AttributeValue::UInt(u) => u.to_string(),
            AttributeValue::Double(f) => f.to_string(),
            AttributeValue::Boolean(b) => b.to_string(),
        }
    }
}