// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Execution state for columnar query engine.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// Additional state that may be carried along during the execution of a pipeline.
///
/// This can be used to store arbitrary extensions that may be needed by custom pipeline stages.
/// Extensions are stored in a type-map and pipeline stages can retrieve them by a known type.
///
/// This is similar to datafusion's `ExecutionState`, which it also uses for extensions, but
/// without the need for `Send + Sync` bounds, as those are not required in this context due to these
/// pipeline stages executing in a single threaded runtime. This also means that pipeline
// stages can get mutable references to extensions if needed.
//
// In the future, this may be expanded to include other execution-related state like metrics,or
// other state needed for stateful stream processing.
//
// When the pipeline is executed without an ExecutionState being provided, a default one will be
// created. This means that anything added to this in the should be inexpensive to initialize in
// the default case.
#[derive(Default)]
pub struct ExecutionState {
    extensions: Option<ExtensionMap>,
}

impl ExecutionState {
    /// Create new execution options.
    #[must_use]
    pub fn new() -> Self {
        Self { extensions: None }
    }

    /// Get extension of type T, if it exists.
    #[must_use]
    pub fn get_extension<T: 'static>(&self) -> Option<&T> {
        self.extensions.as_ref().and_then(|map| {
            map.get(&TypeId::of::<T>())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        })
    }

    /// Get mutable extension of type T, if it exists.
    #[must_use]
    pub fn get_extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extensions.as_mut().and_then(|map| {
            map.get_mut(&TypeId::of::<T>())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        })
    }

    /// Set extension of type T.
    pub fn set_extension<T: 'static>(&mut self, value: T) {
        let map = self.extensions.get_or_insert_with(ExtensionMap::default);
        _ = map.insert(TypeId::of::<T>(), Box::new(value));
    }
}

/// Map that holds opaque objects indexed by their type.
///
// Note: this is similar to datafusion's `AnyMap`, which it also uses for extensions, but
// without the `Send + Sync` bounds, as those are not required in this context due to these
// pipeline stages executing in a single threaded runtime. This also means that pipeline
// stages can get mutable references to extensions if needed.
type ExtensionMap = HashMap<TypeId, Box<dyn Any + 'static>, BuildHasherDefault<IdHasher>>;

/// Hasher for [`ExtensionMap`].
///
// This is the same as the one used by datafusion's `AnyMap`.
//
// With [`TypeId`]s as keys, there's no need to hash them. They are already hashes themselves,
// coming from the compiler. The [`IdHasher`] just holds the [`u64`] of the [`TypeId`], and then
// returns it, instead of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestExtension {
        value: i32,
    }

    #[derive(Debug, PartialEq)]
    struct AnotherExtension {
        name: String,
    }

    #[test]
    fn test_set_and_get_extension() {
        let mut state = ExecutionState::new();

        // Initially, extension should not exist
        assert!(state.get_extension::<TestExtension>().is_none());

        // Set an extension
        state.set_extension(TestExtension { value: 42 });

        // Now we should be able to get it
        let ext = state.get_extension::<TestExtension>();
        assert!(ext.is_some());
        assert_eq!(ext.unwrap().value, 42);
    }

    #[test]
    fn test_get_extension_mut() {
        let mut state = ExecutionState::new();

        // Set an extension
        state.set_extension(TestExtension { value: 10 });

        // Get mutable reference and modify it
        {
            let ext = state.get_extension_mut::<TestExtension>();
            assert!(ext.is_some());
            ext.unwrap().value = 20;
        }

        // Verify the modification
        let ext = state.get_extension::<TestExtension>();
        assert_eq!(ext.unwrap().value, 20);
    }

    #[test]
    fn test_multiple_extensions() {
        let mut state = ExecutionState::new();

        // Set multiple different extensions
        state.set_extension(TestExtension { value: 100 });
        state.set_extension(AnotherExtension {
            name: "test".to_string(),
        });

        // Both should be retrievable independently
        let test_ext = state.get_extension::<TestExtension>();
        assert!(test_ext.is_some());
        assert_eq!(test_ext.unwrap().value, 100);

        let another_ext = state.get_extension::<AnotherExtension>();
        assert!(another_ext.is_some());
        assert_eq!(another_ext.unwrap().name, "test");
    }

    #[test]
    fn test_overwrite_extension() {
        let mut state = ExecutionState::new();

        // Set an extension
        state.set_extension(TestExtension { value: 1 });

        // Overwrite it
        state.set_extension(TestExtension { value: 2 });

        // Should have the new value
        let ext = state.get_extension::<TestExtension>();
        assert_eq!(ext.unwrap().value, 2);
    }
}
