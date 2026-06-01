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

        // `scope_keys` is part of the semantic identity of an attribute set:
        // two sets with identical fields/values but different scope designations
        // must not be de-duplicated, otherwise a scope attribute could be emitted
        // as a data-point attribute (or vice versa) after type erasure. Compare
        // order-independently since the field order is not significant here.
        if !scope_keys_equal(self.descriptor.scope_keys, other.descriptor.scope_keys) {
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

        // Mirror the `PartialEq` treatment of `scope_keys` so the scope/data-point
        // designation survives registry de-duplication. Hash in sorted order to
        // stay consistent with the order-independent equality comparison.
        let mut scope_keys: Vec<&'static str> = self.descriptor.scope_keys.to_vec();
        scope_keys.sort_unstable();
        scope_keys.len().hash(state);
        for key in scope_keys {
            key.hash(state);
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

/// Compares two scope-key slices for set equality, ignoring ordering.
fn scope_keys_equal(left: &[&'static str], right: &[&'static str]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut left_sorted: Vec<&'static str> = left.to_vec();
    let mut right_sorted: Vec<&'static str> = right.to_vec();
    left_sorted.sort_unstable();
    right_sorted.sort_unstable();
    left_sorted == right_sorted
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
        scope_keys: &[],
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
            scope_keys: &[],
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
            scope_keys: &[],
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
    fn test_scope_keys_prevent_dedup() {
        // Two descriptors with identical fields and values, differing only in
        // which key is designated as a scope-level attribute. They must NOT be
        // de-duplicated, otherwise a scope attribute could be emitted as a
        // data-point attribute (or vice versa) after type erasure.
        static DESCRIPTOR_UNSCOPED: AttributesDescriptor = AttributesDescriptor {
            name: "unscoped",
            fields: &[AttributeField {
                key: "alpha",
                r#type: AttributeValueType::String,
                brief: "alpha",
            }],
            scope_keys: &[],
        };

        static DESCRIPTOR_SCOPED: AttributesDescriptor = AttributesDescriptor {
            name: "scoped",
            fields: &[AttributeField {
                key: "alpha",
                r#type: AttributeValueType::String,
                brief: "alpha",
            }],
            scope_keys: &["alpha"],
        };

        #[derive(Debug)]
        struct Unscoped {
            values: Vec<AttributeValue>,
        }

        #[derive(Debug)]
        struct Scoped {
            values: Vec<AttributeValue>,
        }

        impl AttributeSetHandler for Unscoped {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &DESCRIPTOR_UNSCOPED
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        impl AttributeSetHandler for Scoped {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &DESCRIPTOR_SCOPED
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        let mut registry = EntityRegistry::default();

        let outcome1 = registry.register(Unscoped {
            values: vec![AttributeValue::String("value".to_string())],
        });
        assert!(matches!(outcome1, RegisterOutcome::Created(_)));

        let outcome2 = registry.register(Scoped {
            values: vec![AttributeValue::String("value".to_string())],
        });
        assert!(matches!(outcome2, RegisterOutcome::Created(_)));

        let key1 = outcome1.key();
        let key2 = outcome2.key();
        assert_ne!(key1, key2);
        assert_eq!(registry.len(), 2);

        // Registering the same scoped set again should reuse the scoped entry.
        let outcome3 = registry.register(Scoped {
            values: vec![AttributeValue::String("value".to_string())],
        });
        assert!(matches!(outcome3, RegisterOutcome::Existing(_)));
        assert_eq!(key2, outcome3.key());
        assert_eq!(registry.len(), 2);
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

    // Regression: scope-attribute designation must survive type erasure through
    // `EntityAttributeSet`. The registry stores attribute sets as
    // `EntityAttributeSet` (descriptor + values), discarding the original
    // concrete type. Because `is_scope_attribute` is backed by the static
    // descriptor's `scope_keys`, the type-erased entity must still report which
    // attributes belong on the instrumentation scope.
    #[test]
    fn test_scope_attribute_survives_type_erasure() {
        static SCOPED_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
            name: "scoped_attrs",
            fields: &[
                AttributeField {
                    key: "data.point.key",
                    r#type: AttributeValueType::String,
                    brief: "a data-point attribute",
                },
                AttributeField {
                    key: "scope.key",
                    r#type: AttributeValueType::String,
                    brief: "a scope-level attribute",
                },
            ],
            scope_keys: &["scope.key"],
        };

        #[derive(Debug)]
        struct ScopedAttributeSet {
            values: Vec<AttributeValue>,
        }

        impl AttributeSetHandler for ScopedAttributeSet {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &SCOPED_DESCRIPTOR
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        let mut registry = EntityRegistry::default();
        let key = registry
            .register(ScopedAttributeSet {
                values: vec![
                    AttributeValue::String("dp".to_string()),
                    AttributeValue::String("sc".to_string()),
                ],
            })
            .key();

        // `get` returns the type-erased `EntityAttributeSet`.
        let erased = registry.get(key).expect("missing attributes");
        assert!(
            erased.is_scope_attribute("scope.key"),
            "scope attribute should survive type erasure"
        );
        assert!(
            !erased.is_scope_attribute("data.point.key"),
            "data-point attribute must not be reported as scope-level"
        );
        assert!(!erased.is_scope_attribute("unknown.key"));
    }
}
