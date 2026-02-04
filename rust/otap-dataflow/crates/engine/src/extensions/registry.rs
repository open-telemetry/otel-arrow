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
//! bundle.insert::<dyn BearerTokenProvider>(Arc::new(my_token_provider));
//! bundle.insert::<dyn Logger>(Arc::new(my_logger));
//! registry.register("azure_auth", bundle);
//!
//! // A consumer retrieves a capability by trait:
//! let token_provider: Arc<dyn BearerTokenProvider> = registry
//!     .get_trait::<dyn BearerTokenProvider>("azure_auth")?;
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
/// Only extension traits defined in [`crate::extensions`] can be used with this macro.
/// External crates can implement these traits on their types, but cannot create new
/// extension trait types.
///
/// # Example
///
/// ```ignore
/// use otap_df_engine::extension_bundle;
/// use otap_df_engine::extensions::BearerTokenProvider;
/// use std::sync::Arc;
///
/// // Your extension type implements BearerTokenProvider
/// struct MyAuthExtension { /* ... */ }
/// impl BearerTokenProvider for MyAuthExtension { /* ... */ }
///
/// let instance = Arc::new(MyAuthExtension { /* ... */ });
/// let bundle = extension_bundle!(instance => BearerTokenProvider);
///
/// // This would NOT compile (Debug is not an extension trait):
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
/// let bundle = extension_bundle!(instance => BearerTokenProvider, Logger);
/// ```
///
/// Or manually:
/// ```ignore
/// let mut bundle = ExtensionBundle::new();
/// bundle.insert::<dyn BearerTokenProvider>(instance.clone() as Arc<dyn BearerTokenProvider>);
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
    /// * `T` - The trait type (e.g., `dyn BearerTokenProvider`). Must implement `ExtensionTrait`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// bundle.insert::<dyn BearerTokenProvider>(Arc::new(my_impl) as Arc<dyn BearerTokenProvider>);
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
    /// * `T` - The trait type (e.g., `dyn BearerTokenProvider`). Must implement `ExtensionTrait`.
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
    /// * `T` - The trait type (e.g., `dyn BearerTokenProvider`). Must implement `ExtensionTrait`.
    ///
    /// # Errors
    ///
    /// Returns `ExtensionError::NotFound` if no extension with that name exists.
    /// Returns `ExtensionError::TraitNotImplemented` if the extension doesn't implement the trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let token_provider: Arc<dyn BearerTokenProvider> = registry
    ///     .get_trait::<dyn BearerTokenProvider>("azure_auth")?;
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
    use crate::extensions::bearer_token_provider::{BearerToken, BearerTokenProvider};
    use async_trait::async_trait;

    // Test implementation of BearerTokenProvider
    struct TestTokenProvider {
        token: String,
    }

    #[async_trait]
    impl BearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, crate::extensions::Error> {
            Ok(BearerToken::new(self.token.clone(), 0))
        }

        fn subscribe_token_refresh(&self) -> tokio::sync::watch::Receiver<Option<BearerToken>> {
            let (tx, rx) = tokio::sync::watch::channel(None);
            drop(tx);
            rx
        }
    }

    #[test]
    fn test_bundle_insert_and_get() {
        let instance = Arc::new(TestTokenProvider {
            token: "test_token".to_string(),
        });

        let mut bundle = ExtensionBundle::new();
        bundle.insert::<dyn BearerTokenProvider>(instance.clone() as Arc<dyn BearerTokenProvider>);

        // Get by trait
        let token_provider: Arc<dyn BearerTokenProvider> = bundle.get::<dyn BearerTokenProvider>().unwrap();
        // We can't easily test async here, but we can verify it's retrieved
        assert_eq!(bundle.len(), 1);
        assert!(bundle.contains::<dyn BearerTokenProvider>());
        drop(token_provider);
    }

    #[test]
    fn test_registry_get_trait() {
        let instance = Arc::new(TestTokenProvider {
            token: "test_token".to_string(),
        });

        let mut bundle = ExtensionBundle::new();
        bundle.insert::<dyn BearerTokenProvider>(instance as Arc<dyn BearerTokenProvider>);

        let mut map = HashMap::new();
        let _ = map.insert("test_ext".to_string(), bundle);

        let registry = ExtensionRegistry::from_map(map);

        // Get trait by name
        let _token_provider: Arc<dyn BearerTokenProvider> = registry.get_trait("test_ext").unwrap();
    }

    #[test]
    fn test_not_found() {
        let registry = ExtensionRegistry::new();
        let result: Result<Arc<dyn BearerTokenProvider>, _> = registry.get_trait("missing");
        assert!(matches!(result, Err(ExtensionError::NotFound { .. })));
    }

    #[test]
    fn test_trait_not_implemented() {
        // Empty bundle - no traits registered
        let bundle = ExtensionBundle::new();

        let mut map = HashMap::new();
        let _ = map.insert("test_ext".to_string(), bundle);

        let registry = ExtensionRegistry::from_map(map);

        // BearerTokenProvider fails because it wasn't registered
        let result: Result<Arc<dyn BearerTokenProvider>, _> = registry.get_trait("test_ext");
        assert!(matches!(
            result,
            Err(ExtensionError::TraitNotImplemented { .. })
        ));
    }

    #[test]
    fn test_extension_bundle_macro() {
        let instance = Arc::new(TestTokenProvider {
            token: "macro_test".to_string(),
        });

        // Use the macro to create a bundle
        let bundle = crate::extension_bundle!(instance => BearerTokenProvider);

        // Verify trait is accessible
        let _token_provider: Arc<dyn BearerTokenProvider> = bundle.get::<dyn BearerTokenProvider>().unwrap();

        assert_eq!(bundle.len(), 1);
    }
}
