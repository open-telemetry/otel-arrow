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
    entities: SlotMap<EntityKey, EntityEntry>,
    entities_by_signature: HashMap<EntityAttributeSet, EntityKey>,
}

#[derive(Clone)]
struct EntityEntry {
    /// Shared attribute set for this entity.
    attrs: Arc<EntityAttributeSet>,
    /// Reference count for deduplicated entities so we can keep a stable key while
    /// multiple callers register the same attribute set. This is bookkeeping to
    /// drop entities once no metric set or event references them, which is needed
    /// for live reconfiguration without registry leaks.
    refs: usize,
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

/// Register returns an entity key and information about whether the key was
/// created or already existing.
pub enum RegisterOutcome {
    /// The entity was first registered.
    Created(EntityKey),
    /// The entity was previously registered.
    Existing(EntityKey),
}

impl RegisterOutcome {
    /// Returns the entity key, consumes the outcome.
    #[must_use]
    pub const fn key(self) -> EntityKey {
        match self {
            Self::Created(k) => k,
            Self::Existing(k) => k,
        }
    }
}

impl EntityRegistry {
    /// Registers (or reuses) an entity for the provided attribute set and returns its key.
    /// The boolean indicates whether a new entry was created.
    #[must_use]
    pub(crate) fn register(&mut self, attrs: impl AttributeSetHandler) -> RegisterOutcome {
        let entity = EntityAttributeSet::new(attrs);
        if let Some(existing) = self.entities_by_signature.get(&entity) {
            if let Some(entry) = self.entities.get_mut(*existing) {
                entry.refs = entry.refs.saturating_add(1);
            }
            return RegisterOutcome::Existing(*existing);
        }

        let attrs = Arc::new(entity.clone());

        let entity_key = self.entities.insert(EntityEntry { attrs, refs: 1 });
        let _ = self.entities_by_signature.insert(entity, entity_key);
        RegisterOutcome::Created(entity_key)
    }

    /// Increments the reference count for an existing entity key.
    #[must_use]
    pub fn retain(&mut self, entity_key: EntityKey) -> bool {
        if let Some(entry) = self.entities.get_mut(entity_key) {
            entry.refs = entry.refs.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Unregisters an entity by key. Returns true if the key was found.
    #[must_use]
    pub fn unregister(&mut self, entity_key: EntityKey) -> bool {
        let Some(entry) = self.entities.get_mut(entity_key) else {
            return false;
        };

        if entry.refs > 1 {
            entry.refs -= 1;
            return true;
        }

        let entry = self.entities.remove(entity_key).expect("entry exists");
        let _ = self.entities_by_signature.remove(entry.attrs.as_ref());
        true
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
        self.entities.get(key).map(|entry| entry.attrs.as_ref())
    }

    /// Returns a shared attribute set handle for the given key, if it exists.
    #[must_use]
    pub fn get_shared(&self, key: EntityKey) -> Option<Arc<EntityAttributeSet>> {
        self.entities.get(key).map(|entry| entry.attrs.clone())
    }

    /// Visits all registered entities.
    pub fn visit_entities<F>(&self, mut f: F)
    where
        F: FnMut(EntityKey, &dyn AttributeSetHandler),
    {
        for (key, attrs) in self.entities.iter() {
            f(key, attrs.attrs.as_ref());
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
        AttributeValue::Map(m) => {
            5u8.hash(state);
            m.len().hash(state);
            // BTreeMap iterates in sorted key order, so hashing is deterministic.
            for (k, v) in m {
                k.hash(state);
                hash_attribute_value(v, state);
            }
        }
    }
}

const fn attribute_value_type_rank(value_type: AttributeValueType) -> u8 {
    match value_type {
        AttributeValueType::String => 0,
        AttributeValueType::Int => 1,
        AttributeValueType::Double => 2,
        AttributeValueType::Boolean => 3,
        AttributeValueType::Map => 4,
    }
}

fn attribute_value_equal(left: &AttributeValue, right: &AttributeValue) -> bool {
    match (left, right) {
        (AttributeValue::String(a), AttributeValue::String(b)) => a == b,
        (AttributeValue::Int(a), AttributeValue::Int(b)) => a == b,
        (AttributeValue::UInt(a), AttributeValue::UInt(b)) => a == b,
        (AttributeValue::Double(a), AttributeValue::Double(b)) => a.to_bits() == b.to_bits(),
        (AttributeValue::Boolean(a), AttributeValue::Boolean(b)) => a == b,
        (AttributeValue::Map(a), AttributeValue::Map(b)) => a == b,
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

        let outcome1 = registry.register(MockAttributeSet::new("value".to_string()));
        let outcome2 = registry.register(MockAttributeSet::new("value".to_string()));
        assert!(matches!(outcome1, RegisterOutcome::Created(_)));
        assert!(matches!(outcome2, RegisterOutcome::Existing(_)));

        assert_eq!(outcome1.key(), outcome2.key());
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

        let outcome1 = registry.register(AttributeSetA {
            values: vec![
                AttributeValue::String("value".to_string()),
                AttributeValue::Int(7),
            ],
        });
        assert!(matches!(outcome1, RegisterOutcome::Created(_)));
        let outcome2 = registry.register(AttributeSetB {
            values: vec![
                AttributeValue::Int(7),
                AttributeValue::String("value".to_string()),
            ],
        });
        assert!(matches!(outcome2, RegisterOutcome::Existing(_)));

        assert_eq!(outcome1.key(), outcome2.key());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_attributes() {
        let mut registry = EntityRegistry::default();

        let key = registry
            .register(MockAttributeSet::new("value".to_string()))
            .key();
        let attrs = registry.get_shared(key).expect("missing attributes");

        let collected: Vec<_> = attrs.iter_attributes().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, "test_key");
        assert_eq!(*collected[0].1, AttributeValue::String("value".to_string()));
    }

    #[test]
    fn test_unregister() {
        let mut registry = EntityRegistry::default();

        let key = registry
            .register(MockAttributeSet::new("value".to_string()))
            .key();
        let _dup = registry
            .register(MockAttributeSet::new("value".to_string()))
            .key();

        assert!(registry.unregister(key));
        assert_eq!(registry.len(), 1);
        assert!(registry.get_shared(key).is_some());
        assert!(registry.unregister(key));
        assert_eq!(registry.len(), 0);
        assert!(registry.get_shared(key).is_none());
        assert!(!registry.unregister(key));
    }
}
