// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Interface defining a collection of attributes (pairs of key -> value) associated with a
//! [`metrics::MetricSet`].

use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
use serde::Serialize;

/// Specialized iterator over attribute key-value pairs with performance optimizations.
/// This iterator avoids heap allocations and can leverage unsafe optimizations when enabled.
pub struct AttributeIterator<'a> {
    fields: &'static [AttributeField],
    values: &'a [AttributeValue],
    idx: usize,
    len: usize,
}

impl<'a> AttributeIterator<'a> {
    /// Creates a new attribute iterator.
    ///
    /// # Safety
    /// The caller must ensure that `fields.len() == values.len()`.
    #[inline]
    #[must_use]
    pub fn new(fields: &'static [AttributeField], values: &'a [AttributeValue]) -> Self {
        let len = values.len();
        debug_assert_eq!(
            fields.len(),
            len,
            "descriptor.fields and attribute values length must match"
        );
        Self {
            fields,
            values,
            idx: 0,
            len,
        }
    }
}

impl<'a> Iterator for AttributeIterator<'a> {
    type Item = (&'static str, &'a AttributeValue);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let i = self.idx;
            self.idx += 1;

            // Use unchecked indexing when the feature is enabled, otherwise use safe indexing
            let field = {
                #[cfg(feature = "unchecked-index")]
                {
                    // SAFETY: We know `i` is valid because:
                    // 1. `i` was captured from `self.idx` before incrementing
                    // 2. Loop condition ensures `self.idx < self.len` when we enter
                    // 3. fields.len() == values.len() is asserted in new()
                    #[allow(unsafe_code)]
                    unsafe {
                        self.fields.get_unchecked(i)
                    }
                }
                #[cfg(not(feature = "unchecked-index"))]
                {
                    &self.fields[i]
                }
            };

            let value = {
                #[cfg(feature = "unchecked-index")]
                {
                    // SAFETY: Same invariants as above apply to values array
                    #[allow(unsafe_code)]
                    unsafe {
                        self.values.get_unchecked(i)
                    }
                }
                #[cfg(not(feature = "unchecked-index"))]
                {
                    &self.values[i]
                }
            };

            Some((field.key, value))
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.idx);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for AttributeIterator<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.len.saturating_sub(self.idx)
    }
}

// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
// Similar to NonZeroMetrics iterator, this provides optimization benefits.
impl<'a> core::iter::FusedIterator for AttributeIterator<'a> {}

/// Trait implemented by structs representing a set of attributes.
pub trait AttributeSetHandler {
    /// Returns the static descriptor describing this attribute set.
    fn descriptor(&self) -> &'static AttributesDescriptor;

    /// Returns a reference to attribute value slice for this set.
    fn attribute_values(&self) -> &[AttributeValue];

    /// Returns an iterator over key-value pairs of all attributes in this set.
    /// This avoids heap allocations and leverages unsafe optimizations when enabled.
    fn iter_attributes<'a>(&'a self) -> AttributeIterator<'a> {
        AttributeIterator::new(self.descriptor().fields, self.attribute_values())
    }
}

/// Represents a single attribute value that can be of different types.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    #[must_use]
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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_iterator_basic() {
        let fields = &[
            AttributeField {
                key: "attr1",
                r#type: AttributeValueType::String,
                brief: "Test attribute 1",
            },
            AttributeField {
                key: "attr2",
                r#type: AttributeValueType::Int,
                brief: "Test attribute 2",
            },
        ];

        let values = [
            AttributeValue::String("test_value".to_string()),
            AttributeValue::Int(42),
        ];

        let mut iter = AttributeIterator::new(fields, &values);

        let (key1, value1) = iter.next().unwrap();
        assert_eq!(key1, "attr1");
        assert_eq!(*value1, AttributeValue::String("test_value".to_string()));

        let (key2, value2) = iter.next().unwrap();
        assert_eq!(key2, "attr2");
        assert_eq!(*value2, AttributeValue::Int(42));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_attribute_iterator_empty() {
        let fields: &[AttributeField] = &[];
        let values: &[AttributeValue] = &[];

        let mut iter = AttributeIterator::new(fields, values);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_attribute_iterator_size_hint() {
        let fields = &[
            AttributeField {
                key: "attr1",
                r#type: AttributeValueType::String,
                brief: "Test attribute 1",
            },
            AttributeField {
                key: "attr2",
                r#type: AttributeValueType::Boolean,
                brief: "Test attribute 2",
            },
        ];

        let values = [
            AttributeValue::String("test".to_string()),
            AttributeValue::Boolean(true),
        ];

        let iter = AttributeIterator::new(fields, &values);
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 2);
        assert_eq!(upper, Some(2));

        // After consuming one element
        let mut iter = AttributeIterator::new(fields, &values);
        let _ = iter.next();
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 1);
        assert_eq!(upper, Some(1));
    }

    #[test]
    fn test_attribute_iterator_exact_size() {
        let fields = &[AttributeField {
            key: "attr1",
            r#type: AttributeValueType::Double,
            brief: "Test attribute 1",
        }];

        let values = [AttributeValue::Double(std::f64::consts::PI)];

        let mut iter = AttributeIterator::new(fields, &values);
        assert_eq!(iter.len(), 1);

        let _ = iter.next();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn test_attribute_iterator_fused() {
        let fields = &[AttributeField {
            key: "attr1",
            r#type: AttributeValueType::Int,
            brief: "Test attribute 1",
        }];

        let values = [AttributeValue::UInt(100)];

        let mut iter = AttributeIterator::new(fields, &values);

        // Consume the iterator
        let _first = iter.next();
        assert!(iter.next().is_none());

        // Should consistently return None after exhaustion
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_attribute_iterator_different_types() {
        let fields = &[
            AttributeField {
                key: "string_attr",
                r#type: AttributeValueType::String,
                brief: "String attribute",
            },
            AttributeField {
                key: "int_attr",
                r#type: AttributeValueType::Int,
                brief: "Integer attribute",
            },
            AttributeField {
                key: "double_attr",
                r#type: AttributeValueType::Double,
                brief: "Double attribute",
            },
            AttributeField {
                key: "bool_attr",
                r#type: AttributeValueType::Boolean,
                brief: "Boolean attribute",
            },
        ];

        let values = [
            AttributeValue::String("hello".to_string()),
            AttributeValue::Int(-42),
            AttributeValue::Double(std::f64::consts::E),
            AttributeValue::Boolean(false),
        ];

        let iter = AttributeIterator::new(fields, &values);
        let collected: Vec<_> = iter.collect();

        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0].0, "string_attr");
        assert_eq!(*collected[0].1, AttributeValue::String("hello".to_string()));
        assert_eq!(collected[1].0, "int_attr");
        assert_eq!(*collected[1].1, AttributeValue::Int(-42));
        assert_eq!(collected[2].0, "double_attr");
        assert_eq!(*collected[2].1, AttributeValue::Double(std::f64::consts::E));
        assert_eq!(collected[3].0, "bool_attr");
        assert_eq!(*collected[3].1, AttributeValue::Boolean(false));
    }

    #[cfg(feature = "unchecked-index")]
    #[test]
    fn test_attribute_iterator_unchecked_optimization() {
        // This test ensures that the unchecked indexing path is exercised
        // when the feature is enabled. The behavior should be identical to safe indexing.
        let fields = &[AttributeField {
            key: "test_attr",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }];

        let values = [AttributeValue::String("unchecked_test".to_string())];

        let mut iter = AttributeIterator::new(fields, &values);
        let (key, value) = iter.next().unwrap();

        assert_eq!(key, "test_attr");
        assert_eq!(*value, AttributeValue::String("unchecked_test".to_string()));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_attribute_value_type() {
        assert_eq!(
            AttributeValue::String("test".to_string()).value_type(),
            AttributeValueType::String
        );
        assert_eq!(
            AttributeValue::Int(42).value_type(),
            AttributeValueType::Int
        );
        assert_eq!(
            AttributeValue::UInt(42).value_type(),
            AttributeValueType::Int
        ); // UInt treated as Int
        assert_eq!(
            AttributeValue::Double(std::f64::consts::PI).value_type(),
            AttributeValueType::Double
        );
        assert_eq!(
            AttributeValue::Boolean(true).value_type(),
            AttributeValueType::Boolean
        );
    }

    #[test]
    fn test_attribute_value_to_string() {
        assert_eq!(
            AttributeValue::String("hello".to_string()).to_string_value(),
            "hello"
        );
        assert_eq!(AttributeValue::Int(-42).to_string_value(), "-42");
        assert_eq!(AttributeValue::UInt(42).to_string_value(), "42");
        assert_eq!(
            AttributeValue::Double(std::f64::consts::PI).to_string_value(),
            std::f64::consts::PI.to_string()
        );
        assert_eq!(AttributeValue::Boolean(true).to_string_value(), "true");
        assert_eq!(AttributeValue::Boolean(false).to_string_value(), "false");
    }

    // Test for backward compatibility
    #[test]
    fn test_renamed_method() {
        struct TestAttributeSet {
            values: Vec<AttributeValue>,
        }

        static TEST_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
            name: "test_attrs",
            fields: &[AttributeField {
                key: "test_key",
                r#type: AttributeValueType::String,
                brief: "Test attribute",
            }],
        };

        impl AttributeSetHandler for TestAttributeSet {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &TEST_DESCRIPTOR
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        let attr_set = TestAttributeSet {
            values: vec![AttributeValue::String("test_value".to_string())],
        };

        // Test the renamed method works
        let iter = attr_set.iter_attributes();
        let result: Vec<_> = iter.collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "test_key");
        assert_eq!(
            *result[0].1,
            AttributeValue::String("test_value".to_string())
        );
    }
}
