// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! An entity registry recording the attribute sets associated with entities.

use crate::attributes::{AttributeSetHandler, AttributeValue};
use crate::descriptor::{AttributeValueType, AttributesDescriptor};
use crate::registry::EntityKey;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// An attribute set associated with an entity.
#[derive(Debug, Clone)]
pub struct EntityAttributeSet {
    descriptor: &'static AttributesDescriptor,
    values: Arc<[AttributeValue]>,
    // Precomputed sorted indices of attributes by key for efficient equality and hashing.
    sorted_indices: Arc<[usize]>,
}

/// A registry that maintains de-duplicated attribute sets for entities.
#[derive(Default)]
pub struct EntityRegistry {
    entities: SlotMap<EntityKey, Arc<EntityAttributeSet>>,
    entities_by_signature: HashMap<EntityAttributeSet, EntityKey>,
}

impl AttributeSetHandler for EntityAttributeSet {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        self.descriptor
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &self.values
    }
}

impl EntityAttributeSet {
    fn new(attrs: impl AttributeSetHandler) -> Self {
        let descriptor = attrs.descriptor();
        let values: Arc<[AttributeValue]> = attrs.attribute_values().to_vec().into();
        debug_assert_eq!(
            descriptor.fields.len(),
            values.len(),
            "descriptor.fields and attribute values length must match"
        );

        let mut indices: Vec<usize> = (0..descriptor.fields.len()).collect();
        indices.sort_by(|&left, &right| {
            descriptor.fields[left]
                .key
                .cmp(descriptor.fields[right].key)
        });

        Self {
            descriptor,
            values,
            sorted_indices: indices.into(),
        }
    }
}

impl PartialEq for EntityAttributeSet {
    fn eq(&self, other: &Self) -> bool {
        // Defensive: in release builds, return false on length mismatch to avoid OOB access.
        if self.descriptor.fields.len() != self.values.len()
            || other.descriptor.fields.len() != other.values.len()
        {
            return false;
        }

        if self.sorted_indices.len() != other.sorted_indices.len() {
            return false;
        }

        self.sorted_indices
            .iter()
            .zip(other.sorted_indices.iter())
            .all(|(lhs_idx, rhs_idx)| {
                let lhs_field = &self.descriptor.fields[*lhs_idx];
                let rhs_field = &other.descriptor.fields[*rhs_idx];
                if lhs_field.key != rhs_field.key || lhs_field.r#type != rhs_field.r#type {
                    return false;
                }
                let lhs_value = &self.values[*lhs_idx];
                let rhs_value = &other.values[*rhs_idx];
                attribute_value_equal(lhs_value, rhs_value)
            })
    }
}

impl Eq for EntityAttributeSet {}

impl Hash for EntityAttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let descriptor_len = self.descriptor.fields.len();
        let values_len = self.values.len();
        descriptor_len.hash(state);
        values_len.hash(state);
        if descriptor_len != values_len {
            return;
        }

        self.sorted_indices.len().hash(state);
        for idx in self.sorted_indices.iter() {
            let field = &self.descriptor.fields[*idx];
            field.key.hash(state);
            attribute_value_type_rank(field.r#type).hash(state);
            let value = &self.values[*idx];
            hash_attribute_value(value, state);
        }
    }
}

impl Debug for EntityRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityRegistry")
            .field("entities_len", &self.entities.len())
            .finish()
    }
}

impl EntityRegistry {
    /// Registers (or reuses) an entity for the provided attribute set and returns its key.
    #[must_use]
    pub fn register(&mut self, attrs: impl AttributeSetHandler) -> EntityKey {
        let entity = EntityAttributeSet::new(attrs);
        if let Some(existing) = self.entities_by_signature.get(&entity) {
            return *existing;
        }

        let attrs = Arc::new(entity.clone());

        let entity_key = self.entities.insert(attrs);
        let _ = self.entities_by_signature.insert(entity, entity_key);
        entity_key
    }

    /// Unregisters an entity by key. Returns true if the entity was found and removed.
    #[must_use]
    pub fn unregister(&mut self, entity_key: EntityKey) -> bool {
        if let Some(attrs) = self.entities.remove(entity_key) {
            let _ = self.entities_by_signature.remove(attrs.as_ref());
            true
        } else {
            false
        }
    }

    /// Returns the total number of registered entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns true if there are no registered entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the attribute set for the given key, if it exists.
    #[must_use]
    pub fn get(&self, key: EntityKey) -> Option<&EntityAttributeSet> {
        self.entities.get(key).map(|attrs| attrs.as_ref())
    }

    /// Returns a shared attribute set handle for the given key, if it exists.
    #[must_use]
    pub fn get_shared(&self, key: EntityKey) -> Option<Arc<EntityAttributeSet>> {
        self.entities.get(key).cloned()
    }

    /// Visits all registered entities.
    pub fn visit_entities<F>(&self, mut f: F)
    where
        F: FnMut(EntityKey, &dyn AttributeSetHandler),
    {
        for (key, attrs) in self.entities.iter() {
            f(key, attrs.as_ref());
        }
    }
}

fn hash_attribute_value<H: Hasher>(value: &AttributeValue, state: &mut H) {
    match value {
        AttributeValue::String(v) => {
            0u8.hash(state);
            v.hash(state);
        }
        AttributeValue::Int(v) => {
            1u8.hash(state);
            v.hash(state);
        }
        AttributeValue::UInt(v) => {
            2u8.hash(state);
            v.hash(state);
        }
        AttributeValue::Double(v) => {
            3u8.hash(state);
            v.to_bits().hash(state);
        }
        AttributeValue::Boolean(v) => {
            4u8.hash(state);
            v.hash(state);
        }
    }
}

fn attribute_value_type_rank(value_type: AttributeValueType) -> u8 {
    match value_type {
        AttributeValueType::String => 0,
        AttributeValueType::Int => 1,
        AttributeValueType::Double => 2,
        AttributeValueType::Boolean => 3,
    }
}

fn attribute_value_equal(left: &AttributeValue, right: &AttributeValue) -> bool {
    match (left, right) {
        (AttributeValue::String(a), AttributeValue::String(b)) => a == b,
        (AttributeValue::Int(a), AttributeValue::Int(b)) => a == b,
        (AttributeValue::UInt(a), AttributeValue::UInt(b)) => a == b,
        (AttributeValue::Double(a), AttributeValue::Double(b)) => a.to_bits() == b.to_bits(),
        (AttributeValue::Boolean(a), AttributeValue::Boolean(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField {
            key: "test_key",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }],
    };

    #[derive(Debug)]
    struct MockAttributeSet {
        values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: String) -> Self {
            Self {
                values: vec![AttributeValue::String(value)],
            }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    #[test]
    fn test_register_dedupes() {
        let mut registry = EntityRegistry::default();

        let key1 = registry.register(MockAttributeSet::new("value".to_string()));
        let key2 = registry.register(MockAttributeSet::new("value".to_string()));

        assert_eq!(key1, key2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_dedupes_sorted_attributes() {
        static DESCRIPTOR_A: AttributesDescriptor = AttributesDescriptor {
            name: "attrs_a",
            fields: &[
                AttributeField {
                    key: "alpha",
                    r#type: AttributeValueType::String,
                    brief: "alpha",
                },
                AttributeField {
                    key: "beta",
                    r#type: AttributeValueType::Int,
                    brief: "beta",
                },
            ],
        };

        static DESCRIPTOR_B: AttributesDescriptor = AttributesDescriptor {
            name: "attrs_b",
            fields: &[
                AttributeField {
                    key: "beta",
                    r#type: AttributeValueType::Int,
                    brief: "beta",
                },
                AttributeField {
                    key: "alpha",
                    r#type: AttributeValueType::String,
                    brief: "alpha",
                },
            ],
        };

        #[derive(Debug)]
        struct AttributeSetA {
            values: Vec<AttributeValue>,
        }

        #[derive(Debug)]
        struct AttributeSetB {
            values: Vec<AttributeValue>,
        }

        impl AttributeSetHandler for AttributeSetA {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &DESCRIPTOR_A
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        impl AttributeSetHandler for AttributeSetB {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &DESCRIPTOR_B
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        let mut registry = EntityRegistry::default();

        let key1 = registry.register(AttributeSetA {
            values: vec![
                AttributeValue::String("value".to_string()),
                AttributeValue::Int(7),
            ],
        });
        let key2 = registry.register(AttributeSetB {
            values: vec![
                AttributeValue::Int(7),
                AttributeValue::String("value".to_string()),
            ],
        });

        assert_eq!(key1, key2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_attributes() {
        let mut registry = EntityRegistry::default();

        let key = registry.register(MockAttributeSet::new("value".to_string()));
        let attrs = registry.get_shared(key).expect("missing attributes");

        let collected: Vec<_> = attrs.iter_attributes().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, "test_key");
        assert_eq!(*collected[0].1, AttributeValue::String("value".to_string()));
    }

    #[test]
    fn test_unregister() {
        let mut registry = EntityRegistry::default();

        let key = registry.register(MockAttributeSet::new("value".to_string()));

        assert!(registry.unregister(key));
        assert_eq!(registry.len(), 0);
        assert!(registry.get_shared(key).is_none());
        assert!(!registry.unregister(key));
    }
}
