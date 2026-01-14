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
// Note: this is similar to datafusion's `ExecutionState`, which it also uses for extensions, but
// without the need for `Send + Sync` bounds, as those are not required in this context due to these
// pipeline stages executing in a single threaded runtime. This also means that pipeline
// stages can get mutable references to extensions if needed.
//
// In the future, this may be expanded to include other execution-related state like metrics
//
// Also note that when the pipeline is executed without an ExecutionState being provided, a default
// one will be created. This means that anything added to this in the future should be inexpensive
// to initialize in the default case.
#[derive(Default)]
pub struct ExecutionState {
    extensions: Option<ExtensionMap>,
}

impl ExecutionState {
    /// Create new execution options.
    pub fn new() -> Self {
        Self { extensions: None }
    }

    /// Get extension of type T, if it exists.
    pub fn get_extension<T: 'static>(&self) -> Option<&T> {
        self.extensions.as_ref().and_then(|map| {
            map.get(&TypeId::of::<T>())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        })
    }

    /// Get mutable extension of type T, if it exists.
    pub fn get_extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extensions.as_mut().and_then(|map| {
            map.get_mut(&TypeId::of::<T>())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        })
    }

    /// Set extension of type T.
    pub fn set_extension<T: 'static>(&mut self, value: T) {
        let map = self
            .extensions
            .get_or_insert_with(|| ExtensionMap::default());
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
