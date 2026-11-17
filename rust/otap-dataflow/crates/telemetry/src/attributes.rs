// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Interface defining a collection of attributes (pairs of key -> value) associated with a
//! [`crate::metrics::MetricSet`].

use crate::descriptor::{
    AttributeField, AttributeValueType, AttributesDescriptor, MeasurementAttributeDescriptor,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Specialized iterator over attribute key-value pairs with performance optimizations.
/// Stored values are borrowed without allocation; generated snapshots are consumed by value.
pub struct AttributeIterator<'a> {
    fields: std::slice::Iter<'static, AttributeField>,
    values: AttributeValuesIterator<'a>,
}

enum AttributeValuesIterator<'a> {
    Borrowed(std::slice::Iter<'a, AttributeValue>),
    Owned(std::vec::IntoIter<AttributeValue>),
}

impl<'a> AttributeIterator<'a> {
    /// Creates a new attribute iterator.
    ///
    /// # Panics
    /// Panics if `fields.len() != values.len()`, including in release builds.
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) fn new(fields: &'static [AttributeField], values: &'a [AttributeValue]) -> Self {
        Self::from_values(fields, Cow::Borrowed(values))
    }

    fn from_values(fields: &'static [AttributeField], values: Cow<'a, [AttributeValue]>) -> Self {
        assert_eq!(
            fields.len(),
            values.len(),
            "descriptor.fields and attribute values length must match"
        );
        Self {
            fields: fields.iter(),
            values: match values {
                Cow::Borrowed(values) => AttributeValuesIterator::Borrowed(values.iter()),
                Cow::Owned(values) => AttributeValuesIterator::Owned(values.into_iter()),
            },
        }
    }
}

impl<'a> Iterator for AttributeIterator<'a> {
    type Item = (&'static str, Cow<'a, AttributeValue>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let field = self.fields.next()?;
        let value = match &mut self.values {
            AttributeValuesIterator::Borrowed(values) => values.next().map(Cow::Borrowed),
            AttributeValuesIterator::Owned(values) => values.next().map(Cow::Owned),
        };

        value.map(|value| (field.key, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.fields.size_hint()
    }
}

impl<'a> ExactSizeIterator for AttributeIterator<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.fields.len()
    }
}

// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
// Similar to NonZeroMetrics iterator, this provides optimization benefits.
impl<'a> core::iter::FusedIterator for AttributeIterator<'a> {}

/// Trait implemented by structs representing a set of attributes.
pub trait AttributeSetHandler {
    /// Returns the static descriptor describing this attribute set.
    fn descriptor(&self) -> &'static AttributesDescriptor;

    /// Returns attribute values in descriptor order.
    ///
    /// Stored values may be borrowed from `self`. Derived implementations return an
    /// owned snapshot, so subsequent calls cannot invalidate retained values.
    fn attribute_values(&self) -> Cow<'_, [AttributeValue]>;

    /// Returns an iterator over key-value pairs of all attributes in this set.
    /// Borrowed sets require no allocation or cloning. Owned snapshots are moved
    /// into the iterator, and yielded values remain valid after it is dropped.
    fn iter_attributes<'a>(&'a self) -> AttributeIterator<'a> {
        AttributeIterator::from_values(self.descriptor().fields, self.attribute_values())
    }

    /// Returns the schema name for this attribute set (e.g., "pipeline.attrs").
    fn schema_name(&self) -> &'static str {
        self.descriptor().name
    }

    /// Format the attribute set as a string.
    ///
    /// TODO(#1907) this is an inefficient method impacting
    /// console_async logging performance currently. It's OK for this to be
    /// inefficient except it's called inside the logging path where we should not!
    fn attributes_to_string(&self) -> String {
        self.iter_attributes()
            .map(|(k, v)| format!(" {}={}", k, v.to_string_value()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Compile-time key schema for an [`AttributeSetHandler`].
///
/// Generated implementations preserve composed attribute sets so metric-set
/// macros can validate item attribute keys without a runtime registration
/// path.
pub trait AttributeSetKeySchema {
    /// Keys declared directly by this attribute set or by its composed sets.
    const KEY_SCHEMA: &'static [AttributeKeySchema];
}

/// One node in an [`AttributeSetKeySchema`].
pub enum AttributeKeySchema {
    /// A key declared directly on an attribute set.
    Key(&'static str),
    /// The key schema of a composed attribute set.
    Composed(&'static [AttributeKeySchema]),
}

/// Checks for an overlapping registration and measurement attribute key.
///
/// This is public only because generated `#[metric_set]` code evaluates it in a
/// compile-time assertion.
#[doc(hidden)]
#[must_use]
pub const fn has_item_attribute_key_collision(
    registration_schema: &[AttributeKeySchema],
    measurement: &[MeasurementAttributeDescriptor],
) -> bool {
    let mut index = 0;
    while index < registration_schema.len() {
        match registration_schema[index] {
            AttributeKeySchema::Key(registration_key) => {
                let mut measurement_index = 0;
                while measurement_index < measurement.len() {
                    if str_eq(registration_key, measurement[measurement_index].key) {
                        return true;
                    }
                    measurement_index += 1;
                }
            }
            AttributeKeySchema::Composed(schema) => {
                if has_item_attribute_key_collision(schema, measurement) {
                    return true;
                }
            }
        }
        index += 1;
    }
    false
}

const fn str_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// A closed-set (`enum`) attribute value whose variants render to stable strings.
///
/// Implemented via `#[derive(AttributeEnum)]`. Because the value space is closed,
/// the total number of variants ([`AttributeEnum::CARDINALITY`]) and each
/// variant's ordinal ([`AttributeEnum::variant_index`]) are known at compile
/// time, enabling dense mixed-radix bucketing of per-item attributes and a
/// compile-time cardinality bound.
pub trait AttributeEnum: Copy {
    /// Number of variants in the enum.
    const CARDINALITY: usize;
    /// Ordered string forms of every variant (declaration order).
    const VARIANTS: &'static [&'static str];
    /// Returns the ordinal (declaration order) of this value.
    fn variant_index(self) -> usize;
    /// Returns the string form of this value.
    fn as_str(self) -> &'static str {
        Self::VARIANTS[self.variant_index()]
    }
}

/// A set of [`AttributeEnum`]s whose values vary per recorded measurement.
///
/// The set maps to a dense mixed-radix bucket space: the first declared field is
/// the low-order digit. [`CARDINALITY`](Self::CARDINALITY) is the product of the
/// fields' variant counts and equals the number of buckets a measurement metric set
/// allocates.
pub trait MeasurementAttributeSet: Copy {
    /// Total number of attribute combinations (product of field variant counts).
    const CARDINALITY: usize;
    /// Per-field descriptors (key + ordered variant strings), declaration order.
    const DESCRIPTORS: &'static [MeasurementAttributeDescriptor];
    /// Computes the dense bucket index for this attribute combination.
    ///
    /// The first declared field is the low-order digit:
    /// `index = sum(field_i.variant_index() * radix_0 * .. * radix_{i-1})`.
    fn bucket_index(&self) -> usize;
}

/// Represents a single attribute value that can be of different types.
/// TODO(4453) Consolidate with otel_arrow_dfe_config::pipeline::AttributeValue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Map of key:value
    Map(BTreeMap<String, AttributeValue>),
}

impl AttributeValue {
    /// Returns the value type of this attribute value.
    #[must_use]
    pub const fn value_type(&self) -> AttributeValueType {
        match self {
            AttributeValue::String(_) => AttributeValueType::String,
            AttributeValue::Int(_) => AttributeValueType::Int,
            // Semantic Convention: UInt is treated as Int
            AttributeValue::UInt(_) => AttributeValueType::Int,
            AttributeValue::Double(_) => AttributeValueType::Double,
            AttributeValue::Boolean(_) => AttributeValueType::Boolean,
            AttributeValue::Map(_) => AttributeValueType::Map,
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
            AttributeValue::Map(m) => {
                let entries: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_string_value()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }
}

#[cfg(test)]
#[forbid(unsafe_code, static_mut_refs)]
mod tests {
    use super::*;

    #[derive(otel_arrow_dfe_telemetry_macros::AttributeSetHandler)]
    #[attributes(name = "snapshot")]
    struct SnapshotAttributes {
        label: String,
        count: u64,
    }

    #[derive(otel_arrow_dfe_telemetry_macros::AttributeSetHandler)]
    #[attributes(name = "composed.snapshot")]
    struct ComposedSnapshotAttributes {
        active: bool,
        #[compose]
        first: SnapshotAttributes,
        #[compose]
        second: SnapshotAttributes,
    }

    #[derive(otel_arrow_dfe_telemetry_macros::AttributeSetHandler)]
    #[attributes(name = "nested.snapshot")]
    struct NestedSnapshotAttributes {
        #[compose]
        inner: ComposedSnapshotAttributes,
    }

    struct StoredAttributes(Vec<AttributeValue>);

    impl AttributeSetHandler for StoredAttributes {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &AttributesDescriptor {
                name: "stored",
                fields: &[AttributeField {
                    key: "label",
                    r#type: AttributeValueType::String,
                    brief: "Stored label",
                }],
            }
        }

        fn attribute_values(&self) -> Cow<'_, [AttributeValue]> {
            Cow::Borrowed(&self.0)
        }
    }

    impl AttributeSetKeySchema for StoredAttributes {
        const KEY_SCHEMA: &'static [AttributeKeySchema] = &[AttributeKeySchema::Key("label")];
    }

    /// Scenario: two instances of one derived type produce values while earlier slices are live.
    /// Guarantees: repeated calls do not overwrite or invalidate a prior snapshot or its strings.
    #[test]
    fn derived_attribute_snapshots_are_independent() {
        let first = SnapshotAttributes {
            label: "first".into(),
            count: 1,
        };
        let second = SnapshotAttributes {
            label: "second with a longer string".into(),
            count: 2,
        };
        let first_values = first.attribute_values();
        let retained = first_values.as_ref();
        let second_values = second.attribute_values();
        let repeated = first.attribute_values();
        assert_eq!(
            retained,
            &[
                AttributeValue::String("first".into()),
                AttributeValue::UInt(1)
            ]
        );
        assert_eq!(repeated, first_values);
        assert_eq!(
            second_values.as_ref(),
            &[
                AttributeValue::String("second with a longer string".into()),
                AttributeValue::UInt(2),
            ]
        );
    }

    /// Scenario: derived iterators are interleaved and an emitted value outlives its iterator.
    /// Guarantees: retained items and pending items keep their original values across other calls.
    #[test]
    fn derived_attribute_iterators_are_independent() {
        let first = SnapshotAttributes {
            label: "first".into(),
            count: 1,
        };
        let second = SnapshotAttributes {
            label: "second".into(),
            count: 2,
        };
        let mut first_iter = first.iter_attributes();
        let retained = first_iter.next().unwrap();
        let mut second_iter = second.iter_attributes();
        assert_eq!(first_iter.len(), 1);
        assert_eq!(
            first_iter.next().unwrap().1.as_ref(),
            &AttributeValue::UInt(1)
        );
        assert!(first_iter.next().is_none());
        drop(first_iter);
        assert_eq!(retained.0, "label");
        assert_eq!(retained.1.as_ref(), &AttributeValue::String("first".into()));
        assert_eq!(
            second_iter.next().unwrap().1.as_ref(),
            &AttributeValue::String("second".into())
        );
        assert_eq!(second_iter.size_hint(), (1, Some(1)));
        assert_eq!(
            second_iter.next().unwrap().1.as_ref(),
            &AttributeValue::UInt(2)
        );
        assert_eq!(second_iter.len(), 0);
        assert!(second_iter.next().is_none());
        assert!(second_iter.next().is_none());
    }

    /// Scenario: nested composition contains two fields of the same derived type.
    /// Guarantees: local-first order and both sibling values survive repeated composed calls.
    #[test]
    fn composed_attribute_snapshots_are_independent() {
        let make = |count| NestedSnapshotAttributes {
            inner: ComposedSnapshotAttributes {
                active: true,
                first: SnapshotAttributes {
                    label: "left".into(),
                    count,
                },
                second: SnapshotAttributes {
                    label: "right".into(),
                    count: count + 1,
                },
            },
        };
        let first = make(10);
        let second = make(20);
        assert!(std::ptr::eq(first.descriptor(), second.descriptor()));
        assert_eq!(
            first
                .descriptor()
                .fields
                .iter()
                .map(|field| field.key)
                .collect::<Vec<_>>(),
            ["active", "label", "count", "label", "count"]
        );
        let snapshot = first.attribute_values();
        let retained = snapshot.as_ref();
        let mut iter = first.iter_attributes();
        let first_item = iter.next().unwrap();
        let other_snapshot = second.attribute_values();
        let other_items: Vec<_> = second.iter_attributes().collect();
        let expected = [
            AttributeValue::Boolean(true),
            AttributeValue::String("left".into()),
            AttributeValue::UInt(10),
            AttributeValue::String("right".into()),
            AttributeValue::UInt(11),
        ];
        assert_eq!(retained, &expected);
        assert_eq!(first_item.1.as_ref(), &expected[0]);
        let remaining: Vec<_> = iter.map(|(key, value)| (key, value.into_owned())).collect();
        assert_eq!(
            remaining,
            vec![
                ("label", expected[1].clone()),
                ("count", expected[2].clone()),
                ("label", expected[3].clone()),
                ("count", expected[4].clone()),
            ]
        );
        assert_eq!(other_snapshot[2], AttributeValue::UInt(20));
        assert_eq!(other_items[4].1.as_ref(), &AttributeValue::UInt(21));
    }

    /// Scenario: composition combines a borrowed child and an owned child after local fields.
    /// Guarantees: snapshots retain ordered values independently of subsequent child mutations.
    #[test]
    fn composed_attribute_snapshots_mix_borrowed_and_owned_children() {
        #[derive(otel_arrow_dfe_telemetry_macros::AttributeSetHandler)]
        #[attributes(name = "mixed.snapshot")]
        struct MixedAttributes {
            active: bool,
            #[compose]
            stored: StoredAttributes,
            #[compose]
            generated: SnapshotAttributes,
        }

        let mut attrs = MixedAttributes {
            active: true,
            stored: StoredAttributes(vec![AttributeValue::String("stored".into())]),
            generated: SnapshotAttributes {
                label: "generated".into(),
                count: 7,
            },
        };
        let snapshot = attrs.attribute_values().into_owned();
        let items: Vec<_> = attrs
            .iter_attributes()
            .map(|(key, value)| (key, value.into_owned()))
            .collect();
        attrs.stored.0[0] = AttributeValue::String("changed stored".into());
        attrs.generated.label = "changed generated".into();
        let updated = attrs.attribute_values();
        let expected = vec![
            AttributeValue::Boolean(true),
            AttributeValue::String("stored".into()),
            AttributeValue::String("generated".into()),
            AttributeValue::UInt(7),
        ];
        assert_eq!(snapshot, expected);
        assert_eq!(
            items,
            ["active", "label", "label", "count"]
                .into_iter()
                .zip(expected)
                .collect::<Vec<_>>()
        );
        assert_eq!(updated[1], AttributeValue::String("changed stored".into()));
        assert_eq!(
            updated[2],
            AttributeValue::String("changed generated".into())
        );
    }

    /// Scenario: a stored attribute set exposes a string through the trait-object iterator.
    /// Guarantees: iteration borrows existing storage without cloning or allocating a snapshot.
    #[test]
    fn stored_attribute_iteration_borrows_values() {
        let attrs = StoredAttributes(vec![AttributeValue::String("stored".into())]);
        let handler: &dyn AttributeSetHandler = &attrs;
        let (_, value) = handler.iter_attributes().next().unwrap();
        assert!(matches!(value, Cow::Borrowed(_)));
        assert!(std::ptr::eq(value.as_ref(), &attrs.0[0]));
    }

    /// Scenario: safe iterator construction receives more values than descriptor fields.
    /// Guarantees: mismatched input panics before unchecked indexing, even in release builds.
    #[test]
    #[should_panic(expected = "descriptor.fields and attribute values length must match")]
    fn attribute_iterator_rejects_extra_values() {
        let _ = AttributeIterator::new(&[], &[AttributeValue::UInt(1)]).next();
    }

    /// Scenario: safe iterator construction receives fewer values than descriptor fields.
    /// Guarantees: the length invariant is enforced in both mismatch directions.
    #[test]
    #[should_panic(expected = "descriptor.fields and attribute values length must match")]
    fn attribute_iterator_rejects_missing_values() {
        let _ = AttributeIterator::new(
            &[AttributeField {
                key: "missing",
                r#type: AttributeValueType::Int,
                brief: "Missing value",
            }],
            &[],
        );
    }

    /// Scenario: fixed and per-record attribute schemas share or differ in keys.
    /// Guarantees: only overlapping keys are identified as collisions.
    #[test]
    fn item_attribute_key_collision_detection() {
        const STATIC: &[AttributeKeySchema] = &[
            AttributeKeySchema::Key("signal"),
            AttributeKeySchema::Composed(&[AttributeKeySchema::Key("component")]),
        ];
        const MEASUREMENT_WITH_COLLISION: &[MeasurementAttributeDescriptor] =
            &[MeasurementAttributeDescriptor {
                key: "component",
                variants: &["receiver"],
            }];
        const DISJOINT_MEASUREMENT: &[MeasurementAttributeDescriptor] =
            &[MeasurementAttributeDescriptor {
                key: "outcome",
                variants: &["success", "error"],
            }];

        assert!(has_item_attribute_key_collision(
            STATIC,
            MEASUREMENT_WITH_COLLISION
        ));
        assert!(!has_item_attribute_key_collision(
            STATIC,
            DISJOINT_MEASUREMENT
        ));
    }

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

    /// Scenario: attribute iteration runs with unchecked-index enabled elsewhere in the crate.
    /// Guarantees: the same borrowed values and exhaustion behavior are preserved.
    #[cfg(feature = "unchecked-index")]
    #[test]
    fn test_attribute_iterator_with_unchecked_index_feature() {
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

            fn attribute_values(&self) -> Cow<'_, [AttributeValue]> {
                Cow::Borrowed(&self.values)
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
