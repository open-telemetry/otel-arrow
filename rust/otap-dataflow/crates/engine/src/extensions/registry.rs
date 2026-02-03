// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension registry for storing and retrieving extension trait implementations by name.
//!
//! This registry allows extensions to expose multiple trait implementations (capabilities)
//! that other components can look up by name and trait type.
//!
//! # Example
//!
//! ```ignore
//! // An extension registers its capabilities:
//! let mut bundle = ExtensionBundle::new();
//! bundle.insert::<dyn TokenProvider>(Arc::new(my_token_provider));
//! bundle.insert::<dyn Logger>(Arc::new(my_logger));
//! registry.register("azure_auth", bundle);
//!
//! // A consumer retrieves a capability by trait:
//! let token_provider: Arc<dyn TokenProvider> = registry
//!     .get_trait::<dyn TokenProvider>("azure_auth")?;
//! ```

use crate::extensions::ExtensionTrait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

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

/// Macro to build an [`ExtensionBundle`] from an `Arc` instance.
///
/// This macro reduces boilerplate when an extension implements multiple traits.
/// The same `Arc<ConcreteType>` is stored multiple times, each cast to a different trait.
///
/// # Type Safety
///
/// Only traits that have [`ExtensionTrait`](crate::extensions::ExtensionTrait) as a supertrait
/// can be used with this macro. Attempting to use a trait that doesn't implement `ExtensionTrait`
/// will result in a compile-time error.
///
/// # Example
///
/// ```ignore
/// use otap_df_engine::extension_bundle;
/// use otap_df_engine::extensions::ExtensionTrait;
///
/// // Define traits with ExtensionTrait as supertrait
/// trait TokenProvider: ExtensionTrait { fn get_token(&self) -> String; }
/// trait Logger: ExtensionTrait { fn log(&self, msg: &str); }
///
/// struct MyExtension { /* ... */ }
/// impl ExtensionTrait for MyExtension {}
/// impl TokenProvider for MyExtension { /* ... */ }
/// impl Logger for MyExtension { /* ... */ }
///
/// let instance = Arc::new(MyExtension { /* ... */ });
/// let bundle = extension_bundle!(instance => TokenProvider, Logger);
///
/// // This would NOT compile (Debug doesn't implement ExtensionTrait):
/// // let bundle = extension_bundle!(instance => Debug);  // ERROR!
/// ```
#[macro_export]
macro_rules! extension_bundle {
    ($instance:expr => $($trait:ident),* $(,)?) => {{
        let mut bundle = $crate::extensions::registry::ExtensionBundle::new();
        $(
            // This call enforces T: ExtensionTrait at compile time
            bundle.insert::<dyn $trait>($instance.clone() as std::sync::Arc<dyn $trait>);
        )*
        bundle
    }};
}

/// A bundle of trait implementations for a single extension.
///
/// Each extension can implement multiple traits (capabilities). This bundle stores
/// `Arc<dyn Trait>` objects keyed by their trait's `TypeId`.
///
/// The same underlying `Arc<ConcreteType>` can be cast to multiple trait objects
/// and stored multiple times, once per trait.
///
/// # Example
///
/// Using the [`extension_bundle!`] macro:
/// ```ignore
/// let instance = Arc::new(MyExtension { /* ... */ });
/// let bundle = extension_bundle!(instance => TokenProvider, Logger);
/// ```
///
/// Or manually:
/// ```ignore
/// let mut bundle = ExtensionBundle::new();
/// bundle.insert::<dyn TokenProvider>(instance.clone() as Arc<dyn TokenProvider>);
/// bundle.insert::<dyn Logger>(instance.clone() as Arc<dyn Logger>);
/// ```
#[derive(Default)]
pub struct ExtensionBundle {
    traits: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ExtensionBundle {
    /// Create a new empty bundle.
    #[must_use]
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
        }
    }

    /// Insert a trait implementation into the bundle.
    ///
    /// The trait must implement [`ExtensionTrait`] to ensure only recognized
    /// extension capabilities can be stored.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn TokenProvider`). Must implement `ExtensionTrait`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// bundle.insert::<dyn TokenProvider>(Arc::new(my_impl) as Arc<dyn TokenProvider>);
    /// ```
    pub fn insert<T: ExtensionTrait + ?Sized + 'static>(&mut self, value: Arc<T>)
    where
        Arc<T>: Send + Sync,
    {
        let _ = self.traits.insert(TypeId::of::<Arc<T>>(), Box::new(value));
    }

    /// Get a trait implementation from the bundle.
    ///
    /// The trait must implement [`ExtensionTrait`].
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn TokenProvider`). Must implement `ExtensionTrait`.
    #[must_use]
    pub fn get<T: ExtensionTrait + ?Sized + 'static>(&self) -> Option<Arc<T>>
    where
        Arc<T>: Send + Sync + Clone,
    {
        self.traits
            .get(&TypeId::of::<Arc<T>>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
    }

    /// Check if the bundle contains a trait implementation.
    ///
    /// The trait must implement [`ExtensionTrait`].
    #[must_use]
    pub fn contains<T: ExtensionTrait + ?Sized + 'static>(&self) -> bool {
        self.traits.contains_key(&TypeId::of::<Arc<T>>())
    }

    /// Returns true if the bundle has no trait implementations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }

    /// Returns the number of trait implementations in the bundle.
    #[must_use]
    pub fn len(&self) -> usize {
        self.traits.len()
    }
}

impl std::fmt::Debug for ExtensionBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionBundle")
            .field("trait_count", &self.traits.len())
            .finish()
    }
}

/// Registry for extension trait implementations.
///
/// Extensions register themselves here during creation so other components
/// can look them up by name and retrieve specific trait implementations.
///
/// Each extension has a name (String) and an [`ExtensionBundle`] containing
/// its trait implementations.
#[derive(Clone, Default)]
pub struct ExtensionRegistry {
    extensions: Arc<HashMap<String, ExtensionBundle>>,
}

impl ExtensionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(HashMap::new()),
        }
    }

    /// Create a registry from a map of extension bundles.
    #[must_use]
    pub fn from_map(extensions: HashMap<String, ExtensionBundle>) -> Self {
        Self {
            extensions: Arc::new(extensions),
        }
    }

    /// Get a trait implementation by extension name.
    ///
    /// The trait must implement [`ExtensionTrait`].
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn TokenProvider`). Must implement `ExtensionTrait`.
    ///
    /// # Errors
    ///
    /// Returns `ExtensionError::NotFound` if no extension with that name exists.
    /// Returns `ExtensionError::TraitNotImplemented` if the extension doesn't implement the trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let token_provider: Arc<dyn TokenProvider> = registry
    ///     .get_trait::<dyn TokenProvider>("azure_auth")?;
    /// ```
    pub fn get_trait<T: ExtensionTrait + ?Sized + 'static>(
        &self,
        name: &str,
    ) -> Result<Arc<T>, ExtensionError>
    where
        Arc<T>: Send + Sync + Clone,
    {
        let bundle = self
            .extensions
            .get(name)
            .ok_or_else(|| ExtensionError::NotFound {
                name: name.to_string(),
            })?;

        bundle
            .get::<T>()
            .ok_or_else(|| ExtensionError::TraitNotImplemented {
                name: name.to_string(),
                expected: std::any::type_name::<T>(),
            })
    }

    /// Get an extension bundle by name.
    #[must_use]
    pub fn get_bundle(&self, name: &str) -> Option<&ExtensionBundle> {
        self.extensions.get(name)
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
/// Use this to register extension bundles before creating the immutable registry.
///
/// # Example
///
/// ```ignore
/// let mut builder = ExtensionRegistryBuilder::new();
/// builder.register("my_extension".to_string(), bundle);
/// let registry = builder.build();
/// ```
#[derive(Default)]
pub struct ExtensionRegistryBuilder {
    extensions: HashMap<String, ExtensionBundle>,
}

impl ExtensionRegistryBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    /// Register an extension bundle with a name.
    pub fn register(&mut self, name: String, bundle: ExtensionBundle) {
        let _ = self.extensions.insert(name, bundle);
    }

    /// Build the immutable registry.
    #[must_use]
    pub fn build(self) -> ExtensionRegistry {
        ExtensionRegistry::from_map(self.extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::ExtensionTrait;

    // Test traits must implement ExtensionTrait
    trait TestTrait: ExtensionTrait {
        fn value(&self) -> i32;
    }

    trait AnotherTrait: ExtensionTrait {
        fn name(&self) -> &str;
    }

    struct TestImpl {
        val: i32,
        name: String,
    }

    // TestImpl must implement ExtensionTrait to be cast to trait objects
    impl ExtensionTrait for TestImpl {}

    impl TestTrait for TestImpl {
        fn value(&self) -> i32 {
            self.val
        }
    }

    impl AnotherTrait for TestImpl {
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_bundle_insert_and_get() {
        let instance = Arc::new(TestImpl {
            val: 42,
            name: "test".to_string(),
        });

        let mut bundle = ExtensionBundle::new();
        bundle.insert::<dyn TestTrait>(instance.clone() as Arc<dyn TestTrait>);
        bundle.insert::<dyn AnotherTrait>(instance.clone() as Arc<dyn AnotherTrait>);

        // Get by trait
        let test_trait: Arc<dyn TestTrait> = bundle.get::<dyn TestTrait>().unwrap();
        assert_eq!(test_trait.value(), 42);

        let another_trait: Arc<dyn AnotherTrait> = bundle.get::<dyn AnotherTrait>().unwrap();
        assert_eq!(another_trait.name(), "test");

        assert_eq!(bundle.len(), 2);
    }

    #[test]
    fn test_registry_get_trait() {
        let instance = Arc::new(TestImpl {
            val: 42,
            name: "test".to_string(),
        });

        let mut bundle = ExtensionBundle::new();
        bundle.insert::<dyn TestTrait>(instance.clone() as Arc<dyn TestTrait>);
        bundle.insert::<dyn AnotherTrait>(instance.clone() as Arc<dyn AnotherTrait>);

        let mut map = HashMap::new();
        let _ = map.insert("test_ext".to_string(), bundle);

        let registry = ExtensionRegistry::from_map(map);

        // Get trait by name
        let test_trait: Arc<dyn TestTrait> = registry.get_trait("test_ext").unwrap();
        assert_eq!(test_trait.value(), 42);

        let another_trait: Arc<dyn AnotherTrait> = registry.get_trait("test_ext").unwrap();
        assert_eq!(another_trait.name(), "test");
    }

    #[test]
    fn test_not_found() {
        let registry = ExtensionRegistry::new();
        let result: Result<Arc<dyn TestTrait>, _> = registry.get_trait("missing");
        assert!(matches!(result, Err(ExtensionError::NotFound { .. })));
    }

    #[test]
    fn test_trait_not_implemented() {
        let instance = Arc::new(TestImpl {
            val: 42,
            name: "test".to_string(),
        });

        let mut bundle = ExtensionBundle::new();
        // Only register TestTrait, not AnotherTrait
        bundle.insert::<dyn TestTrait>(instance as Arc<dyn TestTrait>);

        let mut map = HashMap::new();
        let _ = map.insert("test_ext".to_string(), bundle);

        let registry = ExtensionRegistry::from_map(map);

        // TestTrait works
        let test_trait: Arc<dyn TestTrait> = registry.get_trait("test_ext").unwrap();
        assert_eq!(test_trait.value(), 42);

        // AnotherTrait fails
        let result: Result<Arc<dyn AnotherTrait>, _> = registry.get_trait("test_ext");
        assert!(matches!(
            result,
            Err(ExtensionError::TraitNotImplemented { .. })
        ));
    }

    #[test]
    fn test_extension_bundle_macro() {
        let instance = Arc::new(TestImpl {
            val: 99,
            name: "macro_test".to_string(),
        });

        // Use the macro to create a bundle
        let bundle = crate::extension_bundle!(instance => TestTrait, AnotherTrait);

        // Verify both traits are accessible
        let test_trait: Arc<dyn TestTrait> = bundle.get::<dyn TestTrait>().unwrap();
        assert_eq!(test_trait.value(), 99);

        let another_trait: Arc<dyn AnotherTrait> = bundle.get::<dyn AnotherTrait>().unwrap();
        assert_eq!(another_trait.name(), "macro_test");

        assert_eq!(bundle.len(), 2);
    }
}
