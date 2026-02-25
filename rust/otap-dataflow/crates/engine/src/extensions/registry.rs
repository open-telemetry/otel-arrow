// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-safe extension registry.
//!
//! The registry stores extension service handles as type-erased, cloneable values.
//! Handles are looked up by `(extension_name, TypeId)` and returned via
//! `downcast_ref::<T>()` followed by `.clone()`, so every consumer gets its own
//! independent copy.
//!
//! `ExtensionRegistry` is `Clone + Send` — each pipeline component receives
//! its own clone at startup, consistent with the thread-per-core, shared-nothing
//! architecture. No `Arc` is needed.
//!
//! Handle types must be `Clone + Send + 'static`. The registry intentionally
//! does **not** require `Sync` — it is a startup-time resource consumed once
//! at the top of each component's `start()` method. Handles that enter
//! tonic/gRPC services (which require `Sync`) are naturally `Sync` because
//! they wrap `Arc<dyn Trait>`, but that is a property of the handle, not a
//! registry requirement.

use crate::error::Error;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A type-erased, cloneable handle entry.
///
/// Stores the handle as `Box<dyn Any + Send>` alongside a function pointer
/// that knows how to clone it back into another `Box<dyn Any + Send>`.
pub(crate) struct ErasedHandle {
    value: Box<dyn Any + Send>,
    clone_fn: fn(&dyn Any) -> Box<dyn Any + Send>,
}

impl ErasedHandle {
    /// Creates a new erased handle, capturing the concrete clone function.
    fn new<T: Clone + Send + 'static>(handle: T) -> Self {
        Self {
            value: Box::new(handle),
            clone_fn: |any| {
                let val = any
                    .downcast_ref::<T>()
                    .expect("TypeId mismatch in ErasedHandle clone — this is a bug");
                Box::new(val.clone())
            },
        }
    }
}

impl Clone for ErasedHandle {
    fn clone(&self) -> Self {
        Self {
            value: (self.clone_fn)(&*self.value),
            clone_fn: self.clone_fn,
        }
    }
}

/// A single extension's set of typed service handles.
///
/// Created by the extension factory and later merged into the
/// [`ExtensionRegistryBuilder`].
pub struct ExtensionHandles {
    /// (TypeId → handle) for this extension.
    handles: Vec<(TypeId, ErasedHandle)>,
}

impl ExtensionHandles {
    /// Creates a new, empty set of handles.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Registers a typed handle.
    ///
    /// The same concrete type should only be registered once per extension.
    /// Registering the same type twice will result in a duplicate entry; the
    /// builder will keep the last one.
    pub fn register<T: Clone + Send + 'static>(&mut self, handle: T) {
        self.handles
            .push((TypeId::of::<T>(), ErasedHandle::new(handle)));
    }

    /// Consumes self and returns the inner handle list.
    #[must_use]
    pub(crate) fn into_inner(self) -> Vec<(TypeId, ErasedHandle)> {
        self.handles
    }
}

impl Default for ExtensionHandles {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`ExtensionRegistry`].
///
/// Collects handles from all extensions during pipeline build, then freezes
/// into an immutable registry that is cloned to each component.
pub struct ExtensionRegistryBuilder {
    /// (extension_name, TypeId) → handle
    entries: HashMap<(String, TypeId), ErasedHandle>,
}

impl ExtensionRegistryBuilder {
    /// Creates a new, empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Merges all handles produced by one extension into the builder.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ExtensionHandleAlreadyRegistered`] if a `(name, TypeId)` pair
    /// is already present.
    pub fn merge(&mut self, extension_name: &str, handles: ExtensionHandles) -> Result<(), Error> {
        for (type_id, handle) in handles.into_inner() {
            let key = (extension_name.to_owned(), type_id);
            if self.entries.contains_key(&key) {
                return Err(Error::ExtensionHandleAlreadyRegistered {
                    extension: extension_name.to_owned(),
                    type_name: format!("{type_id:?}"),
                });
            }
            let _ = self.entries.insert(key, handle);
        }
        Ok(())
    }

    /// Freezes the builder into an immutable [`ExtensionRegistry`].
    #[must_use]
    pub fn build(self) -> ExtensionRegistry {
        ExtensionRegistry {
            entries: self.entries,
        }
    }
}

impl Default for ExtensionRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// An immutable, cloneable registry of extension service handles.
///
/// Created once during pipeline build, then cloned to each component's
/// `start()` call. Handles are retrieved by extension name + concrete type.
///
/// `Clone + Send` — no `Arc` needed. Each component owns its own copy,
/// consistent with the shared-nothing, thread-per-core model. `Sync` is
/// intentionally not required — the registry is consumed at startup before
/// any handles enter tonic services.
#[derive(Clone)]
pub struct ExtensionRegistry {
    /// (extension_name, TypeId) → type-erased handle.
    entries: HashMap<(String, TypeId), ErasedHandle>,
}

impl ExtensionRegistry {
    /// Creates an empty registry (useful for tests or pipelines with no extensions).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Looks up a handle by extension name and concrete type, returning a clone.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ExtensionHandleNotFound`] when no handle of the requested
    /// type is registered for the given extension name.
    pub fn get<T: Clone + Send + 'static>(&self, extension_name: &str) -> Result<T, Error> {
        let key = (extension_name.to_owned(), TypeId::of::<T>());
        let entry = self
            .entries
            .get(&key)
            .ok_or_else(|| Error::ExtensionHandleNotFound {
                extension: extension_name.to_owned(),
                type_name: std::any::type_name::<T>().to_owned(),
            })?;
        // Safety: we stored a `T` under `TypeId::of::<T>()`, so downcast always succeeds.
        let handle = entry
            .value
            .downcast_ref::<T>()
            .expect("TypeId mismatch in extension registry — this is a bug");
        Ok(handle.clone())
    }

    /// Returns `true` if the registry contains no handles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of registered handles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl std::fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistry")
            .field("num_handles", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestHandle {
        value: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct AnotherHandle {
        count: u64,
    }

    #[test]
    fn register_and_retrieve_handle() {
        let mut handles = ExtensionHandles::new();
        handles.register(TestHandle {
            value: "hello".into(),
        });

        let mut builder = ExtensionRegistryBuilder::new();
        builder.merge("my_ext", handles).unwrap();
        let registry = builder.build();

        let h: TestHandle = registry.get("my_ext").unwrap();
        assert_eq!(h.value, "hello");
    }

    #[test]
    fn retrieve_wrong_type_fails() {
        let mut handles = ExtensionHandles::new();
        handles.register(TestHandle {
            value: "hello".into(),
        });

        let mut builder = ExtensionRegistryBuilder::new();
        builder.merge("my_ext", handles).unwrap();
        let registry = builder.build();

        let result = registry.get::<AnotherHandle>("my_ext");
        assert!(result.is_err());
    }

    #[test]
    fn retrieve_wrong_name_fails() {
        let mut handles = ExtensionHandles::new();
        handles.register(TestHandle {
            value: "hello".into(),
        });

        let mut builder = ExtensionRegistryBuilder::new();
        builder.merge("my_ext", handles).unwrap();
        let registry = builder.build();

        let result = registry.get::<TestHandle>("other_ext");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_types_from_same_extension() {
        let mut handles = ExtensionHandles::new();
        handles.register(TestHandle {
            value: "token".into(),
        });
        handles.register(AnotherHandle { count: 42 });

        let mut builder = ExtensionRegistryBuilder::new();
        builder.merge("auth", handles).unwrap();
        let registry = builder.build();

        assert_eq!(
            registry.get::<TestHandle>("auth").unwrap(),
            TestHandle {
                value: "token".into()
            }
        );
        assert_eq!(
            registry.get::<AnotherHandle>("auth").unwrap(),
            AnotherHandle { count: 42 }
        );
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut handles_a = ExtensionHandles::new();
        handles_a.register(TestHandle {
            value: "first".into(),
        });
        let mut handles_b = ExtensionHandles::new();
        handles_b.register(TestHandle {
            value: "second".into(),
        });

        let mut builder = ExtensionRegistryBuilder::new();
        builder.merge("ext", handles_a).unwrap();
        let err = builder.merge("ext", handles_b);
        assert!(err.is_err());
    }

    #[test]
    fn empty_registry() {
        let registry = ExtensionRegistry::empty();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
