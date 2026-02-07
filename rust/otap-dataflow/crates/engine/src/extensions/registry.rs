// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension registry for storing and retrieving extension trait implementations by name.
//!
//! This registry uses a caster-based approach where extensions store a single boxed
//! instance and cast functions for each trait they implement. This avoids Arc/Rc
//! and the associated Send+Sync requirements on trait objects.
//!
//! # Example
//!
//! ```ignore
//! // An extension registers its capabilities using the macro:
//! let instance = AzureIdentityAuthExtension::new(...);
//! let casters = extension_traits!(AzureIdentityAuthExtension => BearerTokenProvider);
//!
//! // Pass to ExtensionWrapper which builds the registry entry:
//! ExtensionWrapper::local(instance, casters, node_id, config, ...);
//!
//! // A consumer retrieves a capability by trait (returns a reference):
//! let token_provider: &dyn BearerTokenProvider = registry
//!     .get_trait::<dyn BearerTokenProvider>("azure_auth")?;
//! ```

// Allow unsafe code in this module for fat pointer transmutation.
// The safety invariants are documented and upheld by the implementation.
#![allow(unsafe_code)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// A cast function: downcasts `&dyn Any` → `&ConcreteType` → `&dyn Trait`,
/// then returns the fat pointer as `[usize; 2]` for type-erased storage.
/// Returns None if the downcast fails.
pub type CastFn = fn(&dyn Any) -> Option<[usize; 2]>;

/// Reconstruct a `&dyn Trait` from a `[usize; 2]` fat pointer.
///
/// # Safety
/// The caller must ensure `fat` was produced by `trait_ref_to_raw` with the
/// same `Trait` type, and that the underlying data is still alive.
#[inline]
pub unsafe fn raw_to_trait_ref<'a, T: ?Sized + 'a>(fat: [usize; 2]) -> &'a T {
    // SAFETY: The caller guarantees fat was produced by trait_ref_to_raw with the same T.
    unsafe { std::mem::transmute_copy(&fat) }
}

/// Convert a `&dyn Trait` fat pointer into `[usize; 2]` for storage.
///
/// # Safety
/// Relies on the standard Rust fat-pointer layout: `[data_ptr, vtable_ptr]`.
#[inline]
pub unsafe fn trait_ref_to_raw<T: ?Sized>(r: &T) -> [usize; 2] {
    // SAFETY: Fat pointer layout is stable for trait objects.
    unsafe { std::mem::transmute_copy(&r) }
}

/// Marker trait for TypeId lookup of trait types.
/// Used to get a stable TypeId for `dyn Trait` types.
pub trait TraitId<T: ?Sized> {}

/// Error when retrieving an extension trait.
#[derive(Debug)]
pub enum ExtensionError {
    /// Extension not found by name.
    NotFound {
        /// The name of the extension that was not found.
        name: String,
    },
    /// Extension found but doesn't implement the requested trait.
    TraitNotImplemented {
        /// The name of the extension.
        name: String,
        /// The expected trait name.
        expected: &'static str,
    },
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::NotFound { name } => {
                write!(f, "extension '{}' not found", name)
            }
            ExtensionError::TraitNotImplemented { name, expected } => {
                write!(
                    f,
                    "extension '{}' does not implement trait {}",
                    name, expected
                )
            }
        }
    }
}

impl std::error::Error for ExtensionError {}

/// Cast functions for an extension's trait implementations.
///
/// This is the return type of the [`extension_traits!`] macro. It contains
/// the mapping from trait TypeIds to cast functions that can convert
/// `&dyn Any` to `&dyn Trait`.
///
/// # Example
///
/// ```ignore
/// use otap_df_engine::extension_traits;
/// use otap_df_engine::extensions::BearerTokenProvider;
///
/// struct MyAuthExtension { /* ... */ }
/// impl BearerTokenProvider for MyAuthExtension { /* ... */ }
///
/// let casters = extension_traits!(MyAuthExtension => BearerTokenProvider);
/// ```
#[derive(Default)]
pub struct ExtensionTraits {
    casters: HashMap<TypeId, CastFn>,
}

impl ExtensionTraits {
    /// Create a new empty casters collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            casters: HashMap::new(),
        }
    }

    /// Create from a raw HashMap (used by the macro).
    #[must_use]
    pub fn from_map(casters: HashMap<TypeId, CastFn>) -> Self {
        Self { casters }
    }

    /// Returns the inner HashMap.
    #[must_use]
    pub fn into_inner(self) -> HashMap<TypeId, CastFn> {
        self.casters
    }

    /// Check if a trait is registered.
    #[must_use]
    pub fn contains<T: ?Sized + 'static>(&self) -> bool {
        self.casters.contains_key(&TypeId::of::<dyn TraitId<T>>())
    }

    /// Returns true if no traits are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.casters.is_empty()
    }

    /// Returns the number of registered traits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.casters.len()
    }
}

impl std::fmt::Debug for ExtensionTraits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionTraits")
            .field("trait_count", &self.casters.len())
            .finish()
    }
}

/// Macro to generate cast functions for an extension's trait implementations.
///
/// This macro generates a mapping from trait TypeIds to cast functions that
/// can convert `&dyn Any` to `&dyn Trait`. No Arc/Rc cloning is involved.
///
/// # Arguments
///
/// * First: The concrete type name (needed for downcast)
/// * After `=>`: Comma-separated list of trait names this type implements
///
/// Returns an [`ExtensionTraits`] that can be passed to `ExtensionWrapper::local()`.
///
/// # Type Safety
///
/// Only traits that implement [`crate::extensions::ExtensionTrait`] can be used
/// with this macro. This is enforced at compile time - attempting to use an
/// arbitrary trait will result in a compilation error. The macro also verifies
/// that the concrete type implements each specified trait.
///
/// # Example
///
/// ```ignore
/// use otap_df_engine::extension_traits;
/// use otap_df_engine::extensions::BearerTokenProvider;
///
/// struct MyAuthExtension { /* ... */ }
/// impl BearerTokenProvider for MyAuthExtension { /* ... */ }
///
/// let instance = MyAuthExtension { /* ... */ };
/// let traits = extension_traits!(MyAuthExtension => BearerTokenProvider);
///
/// ExtensionWrapper::local(instance, traits, node_id, user_config, config);
/// ```
#[macro_export]
macro_rules! extension_traits {
    ($concrete_ty:ty => $($trait:ident),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut casters: std::collections::HashMap<
            std::any::TypeId,
            $crate::extensions::registry::CastFn
        > = std::collections::HashMap::new();
        $(
            {
                // Compile-time check: ensure the trait is a valid ExtensionTrait.
                // This prevents using arbitrary traits with this macro.
                const _: fn() = || {
                    fn assert_extension_trait<T: ?Sized + $crate::extensions::ExtensionTrait>() {}
                    assert_extension_trait::<dyn $trait>();
                };

                // Inner fn is monomorphic — $concrete_ty is substituted by the macro,
                // so there are no captures and this coerces to a fn pointer.
                fn __cast(any: &dyn std::any::Any) -> Option<[usize; 2]> {
                    let concrete = any.downcast_ref::<$concrete_ty>()?;
                    let trait_ref: &dyn $trait = concrete;
                    // SAFETY: We're converting a valid trait reference to its raw representation
                    Some(unsafe { $crate::extensions::registry::trait_ref_to_raw(trait_ref) })
                }
                let _ = casters.insert(
                    std::any::TypeId::of::<dyn $crate::extensions::registry::TraitId<dyn $trait>>(),
                    __cast as $crate::extensions::registry::CastFn,
                );
            }
        )*
        $crate::extensions::registry::ExtensionTraits::from_map(casters)
    }};
}

/// Internal storage for an extension instance and its casters.
///
/// This is used internally by the registry to store extensions.
/// Users should not create this directly - use [`extension_traits!`] macro
/// with `ExtensionWrapper::local()` or `::shared()`.
pub struct ExtensionEntry {
    /// The single concrete instance, type-erased.
    instance: Box<dyn Any + Send>,
    /// One cast function per registered trait.
    casters: HashMap<TypeId, CastFn>,
}

impl ExtensionEntry {
    /// Create a new entry from an instance and casters.
    pub fn new<T: Send + 'static>(instance: T, casters: ExtensionTraits) -> Self {
        Self {
            instance: Box::new(instance),
            casters: casters.into_inner(),
        }
    }

    /// Get a trait reference from the entry.
    #[must_use]
    pub fn get<T: ?Sized + 'static>(&self) -> Option<&T> {
        let cast = self.casters.get(&TypeId::of::<dyn TraitId<T>>())?;
        let fat = cast(self.instance.as_ref())?;
        // SAFETY: `fat` was produced by `trait_ref_to_raw::<T>` from a valid
        // `&T` derived from the boxed instance. The box is alive for `&self`.
        Some(unsafe { raw_to_trait_ref(fat) })
    }

    /// Check if the entry contains a trait implementation.
    #[must_use]
    pub fn contains<T: ?Sized + 'static>(&self) -> bool {
        self.casters.contains_key(&TypeId::of::<dyn TraitId<T>>())
    }

    /// Returns the number of trait implementations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.casters.len()
    }

    /// Returns true if no traits are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.casters.is_empty()
    }
}

// ExtensionEntry is Send because:
// - instance: Box<dyn Any + Send> is Send
// - casters: HashMap<TypeId, CastFn> - TypeId is Send+Sync, fn pointers are Send+Sync
//
// ExtensionEntry is Sync because:
// - The entry is immutable after construction
// - get() only returns shared references
unsafe impl Sync for ExtensionEntry {}

impl std::fmt::Debug for ExtensionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionEntry")
            .field("trait_count", &self.casters.len())
            .finish()
    }
}

/// Registry for extension trait implementations.
///
/// Extensions register themselves here during creation so other components
/// can look them up by name and retrieve trait references.
///
/// The registry wraps entries in an `Arc` so it can be cheaply cloned
/// (e.g., when cloning effect handlers). Callers receive borrowed
/// `&dyn Trait` references tied to the registry's lifetime.
#[derive(Default, Clone)]
pub struct ExtensionRegistry {
    extensions: Arc<HashMap<String, ExtensionEntry>>,
}

impl ExtensionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(HashMap::new()),
        }
    }

    /// Create a registry from a map of extension entries.
    #[must_use]
    pub fn from_map(extensions: HashMap<String, ExtensionEntry>) -> Self {
        Self {
            extensions: Arc::new(extensions),
        }
    }

    /// Get a trait reference by extension name.
    ///
    /// Returns a borrowed `&dyn Trait` tied to the registry's lifetime.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn BearerTokenProvider`).
    ///
    /// # Errors
    ///
    /// Returns `ExtensionError::NotFound` if no extension with that name exists.
    /// Returns `ExtensionError::TraitNotImplemented` if the extension doesn't implement the trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let token_provider: &dyn BearerTokenProvider = registry
    ///     .get_trait::<dyn BearerTokenProvider>("azure_auth")?;
    /// ```
    pub fn get_trait<T: ?Sized + 'static>(&self, name: &str) -> Result<&T, ExtensionError> {
        let entry = self
            .extensions
            .get(name)
            .ok_or_else(|| ExtensionError::NotFound {
                name: name.to_string(),
            })?;

        entry.get::<T>().ok_or_else(|| ExtensionError::TraitNotImplemented {
            name: name.to_string(),
            expected: std::any::type_name::<T>(),
        })
    }

    /// Check if an extension exists by name.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.extensions.contains_key(name)
    }

    /// Returns the number of registered extensions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    /// Returns true if no extensions are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    /// Returns an iterator over extension names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.extensions.keys()
    }
}

impl std::fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistry")
            .field("extensions", &self.extensions.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Builder for constructing an [`ExtensionRegistry`].
///
/// Use this to register extension entries before creating the immutable registry.
///
/// # Example
///
/// ```ignore
/// let mut builder = ExtensionRegistryBuilder::new();
///
/// let auth = AzureIdentityAuthExtension::new(...);
/// let casters = extension_traits!(AzureIdentityAuthExtension => BearerTokenProvider);
/// builder.register("azure_auth", auth, casters);
///
/// let registry = builder.build();
/// ```
#[derive(Default)]
pub struct ExtensionRegistryBuilder {
    /// The map of extension names to entries being built.
    pub extensions: HashMap<String, ExtensionEntry>,
}

impl ExtensionRegistryBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    /// Register an extension with a name, instance, and casters.
    pub fn register<T: Send + 'static>(
        &mut self,
        name: String,
        instance: T,
        casters: ExtensionTraits,
    ) {
        let _ = self.extensions.insert(name, ExtensionEntry::new(instance, casters));
    }

    /// Build the immutable registry.
    #[must_use]
    pub fn build(self) -> ExtensionRegistry {
        ExtensionRegistry::from_map(self.extensions)
    }
}

impl std::fmt::Debug for ExtensionRegistryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistryBuilder")
            .field("extensions", &self.extensions.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::BearerToken;
    use crate::extensions::BearerTokenProvider;
    use tokio::sync::watch;

    struct TestTokenProvider {
        token: String,
    }

    #[async_trait::async_trait]
    impl BearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, crate::extensions::Error> {
            Ok(BearerToken::new(self.token.clone(), 0))
        }

        fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>> {
            let (tx, rx) = watch::channel(None);
            drop(tx);
            rx
        }
    }

    #[test]
    fn test_extension_casters() {
        let casters = crate::extension_traits!(TestTokenProvider => BearerTokenProvider);
        assert_eq!(casters.len(), 1);
        assert!(casters.contains::<dyn BearerTokenProvider>());
    }

    #[test]
    fn test_extension_entry() {
        let instance = TestTokenProvider {
            token: "test_token".to_string(),
        };
        let casters = crate::extension_traits!(TestTokenProvider => BearerTokenProvider);
        let entry = ExtensionEntry::new(instance, casters);

        assert_eq!(entry.len(), 1);
        assert!(entry.contains::<dyn BearerTokenProvider>());

        let token_provider: &dyn BearerTokenProvider = entry.get().unwrap();
        drop(token_provider);
    }

    #[test]
    fn test_registry_get_trait() {
        let instance = TestTokenProvider {
            token: "test_token".to_string(),
        };
        let casters = crate::extension_traits!(TestTokenProvider => BearerTokenProvider);
        let entry = ExtensionEntry::new(instance, casters);

        let mut map = HashMap::new();
        let _ = map.insert("test_ext".to_string(), entry);

        let registry = ExtensionRegistry::from_map(map);

        let result: Result<&dyn BearerTokenProvider, _> = registry.get_trait("test_ext");
        assert!(result.is_ok());

        let not_found: Result<&dyn BearerTokenProvider, _> = registry.get_trait("missing");
        assert!(matches!(not_found, Err(ExtensionError::NotFound { .. })));
    }

    #[test]
    fn test_registry_builder() {
        let mut builder = ExtensionRegistryBuilder::new();
        assert!(builder.extensions.is_empty());

        let instance = TestTokenProvider {
            token: "builder_test".to_string(),
        };
        let casters = crate::extension_traits!(TestTokenProvider => BearerTokenProvider);

        builder.register("my_extension".to_string(), instance, casters);

        let registry = builder.build();
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("my_extension"));
        let _: &dyn BearerTokenProvider = registry.get_trait("my_extension").unwrap();
    }

    #[test]
    fn test_extension_error_display() {
        let not_found = ExtensionError::NotFound {
            name: "missing_ext".to_string(),
        };
        let display = format!("{}", not_found);
        assert!(display.contains("missing_ext"));
        assert!(display.contains("not found"));

        let not_impl = ExtensionError::TraitNotImplemented {
            name: "my_ext".to_string(),
            expected: "BearerTokenProvider",
        };
        let display = format!("{}", not_impl);
        assert!(display.contains("my_ext"));
        assert!(display.contains("BearerTokenProvider"));
    }

    #[test]
    fn test_registry_debug() {
        let instance = TestTokenProvider {
            token: "test".to_string(),
        };
        let casters = crate::extension_traits!(TestTokenProvider => BearerTokenProvider);
        let entry = ExtensionEntry::new(instance, casters);

        let registry =
            ExtensionRegistry::from_map(HashMap::from([("test_ext".to_string(), entry)]));
        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ExtensionRegistry"));
        assert!(debug_str.contains("test_ext"));
    }
}
