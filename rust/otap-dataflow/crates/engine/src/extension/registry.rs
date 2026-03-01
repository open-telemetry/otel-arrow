// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension registry for storing and retrieving extension trait implementations by name.
//!
//! The registry stores `Box<dyn Any + Send>` for type-erased storage and produces
//! `Box<dyn Trait>` for trait-based lookups. It is `Clone` and `Send` — cloning
//! deep-copies each stored extension (which is cheap when the extension itself
//! wraps shared state in `Arc`).
//!
//! Extensions that publish traits override
//! [`Extension::extension_traits`](crate::local::extension::Extension::extension_traits),
//! using the [`extension_traits!`] macro to declare their trait implementations.
//! The engine calls `extension_traits()` during pipeline build and inserts the
//! results into the registry.
//!
//! # Extension writer contract
//!
//! Extension structs that publish traits must be `Clone + Send + 'static`.
//! Shared mutable state (e.g. credentials, token senders) should be held behind
//! `Arc` so that independent clones still observe the same state.
//!
//! Extensions that don't publish any traits (pure background tasks) have no
//! `Clone` requirement.
//!
//! # Example
//!
//! ```ignore
//! // In the Extension impl:
//! fn extension_traits(&self) -> Vec<TraitRegistration> {
//!     extension_traits!(self => BearerTokenProvider)
//! }
//!
//! // A consumer retrieves an owned trait object:
//! let provider: Box<dyn BearerTokenProvider> = registry
//!     .get::<dyn BearerTokenProvider>("azure_auth")?;
//! provider.get_token().await?;
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ── Sealed trait infrastructure ─────────────────────────────────────────────

// Sealed module — `pub(crate)` so extension trait files in `extension/` can
// add `impl Sealed` for their own `dyn Trait` types, while external crates
// cannot.
pub(crate) mod private {
    /// Sealing trait — prevents external crates from implementing
    /// [`ExtensionTrait`](super::ExtensionTrait).
    pub trait Sealed {}
}

/// Marker trait for extension trait types that can be stored in the
/// [`ExtensionRegistry`].
///
/// This trait is **sealed** — it can only be implemented inside this crate.
/// Each extension trait file in `extension/` adds its own `impl Sealed` +
/// `impl ExtensionTrait` pair (see
/// [`bearer_token_provider`](super::bearer_token_provider) for the pattern).
pub trait ExtensionTrait: private::Sealed {}

/// Error type for extension trait operations.
///
/// Thread-safe error type compatible with any `thiserror`-derived error.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

// ── CloneAnySend helper trait ────────────────────────────────────────────────

/// Internal trait for type-erased, cloneable, `Send` storage.
///
/// Each concrete `T: Clone + Send + 'static` gets a blanket implementation.
/// `Box<dyn CloneAnySend>` implements `Clone` via `clone_box()`.
pub(crate) trait CloneAnySend: Send {
    /// Deep-clone into a new boxed trait object.
    fn clone_box(&self) -> Box<dyn CloneAnySend>;
    /// Access the concrete value as `&dyn Any` for downcasting.
    fn as_any_ref(&self) -> &dyn Any;
}

impl<T: Clone + Send + 'static> CloneAnySend for T {
    fn clone_box(&self) -> Box<dyn CloneAnySend> {
        Box::new(self.clone())
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

impl Clone for Box<dyn CloneAnySend> {
    fn clone(&self) -> Self {
        // Explicit double-deref so method resolution dispatches through the
        // vtable of `dyn CloneAnySend` (→ concrete type), NOT through the
        // blanket `CloneAnySend for Box<dyn CloneAnySend>` which would recurse.
        (**self).clone_box()
    }
}

// ── RegistryEntry ────────────────────────────────────────────────────────────

/// A single entry in the registry: a cloneable concrete value plus a coerce
/// function that knows how to produce `Box<dyn Any + Send>` (containing a
/// `Box<dyn Trait>`) from a `&dyn Any` reference pointing at the concrete type.
///
/// The `coerce` function pointer is monomorphised at registration time (inside
/// the [`extension_traits!`] macro) and is `Copy`, so the entry is
/// cheaply cloneable.
struct RegistryEntry {
    /// The concrete extension value, type-erased but cloneable.
    value: Box<dyn CloneAnySend>,
    /// Clones the concrete value out of `&dyn Any` and wraps it as
    /// `Box<Box<dyn Trait>>` erased to `Box<dyn Any + Send>`.
    coerce: fn(&dyn Any) -> Box<dyn Any + Send>,
}

impl Clone for RegistryEntry {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            coerce: self.coerce,
        }
    }
}

// ── TraitRegistration ────────────────────────────────────────────────────────

/// A self-contained registration for one trait that an extension implements.
///
/// Produced by the [`extension_traits!`] macro. Each registration carries:
/// - A cloned copy of the concrete extension value (type-erased)
/// - A monomorphised `coerce` function pointer for producing `Box<dyn Trait>`
/// - The `TypeId` of `Box<dyn Trait>` for registry lookup
///
/// The extension writer just returns `Vec<TraitRegistration>` from
/// [`Extension::extension_traits`](crate::local::extension::Extension::extension_traits);
/// the engine inserts them into the [`ExtensionRegistry`] by name.
pub struct TraitRegistration {
    /// `TypeId` of `Box<dyn Trait>` — used as registry lookup key.
    trait_id: TypeId,
    /// The concrete extension value, type-erased but cloneable.
    value: Box<dyn CloneAnySend>,
    /// Monomorphised fn: given `&dyn Any` pointing at the concrete extension
    /// type, clone it, wrap in `Box<dyn Trait>`, and return as
    /// `Box<dyn Any + Send>`.
    coerce: fn(&dyn Any) -> Box<dyn Any + Send>,
}

impl TraitRegistration {
    /// Creates a new trait registration.
    ///
    /// This is intended for use by the [`extension_traits!`] macro — not for
    /// direct use by extension writers.
    #[doc(hidden)]
    pub fn new(
        trait_id: TypeId,
        value: impl Clone + Send + 'static,
        coerce: fn(&dyn Any) -> Box<dyn Any + Send>,
    ) -> Self {
        Self {
            trait_id,
            value: Box::new(value),
            coerce,
        }
    }
}

// ── Public types ─────────────────────────────────────────────────────────────

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

// ── ExtensionRegistry ────────────────────────────────────────────────────────

/// Registry for extension trait implementations.
///
/// Extensions register themselves here during pipeline build so other components
/// can look them up by name and retrieve `Box<dyn Trait>` references.
///
/// The registry is `Clone` and `Send`. Cloning deep-copies each stored
/// extension value (cheap when the extension wraps shared state in `Arc`).
/// Each `get` call returns a freshly-cloned `Box<dyn Trait>`.
#[derive(Default, Clone)]
pub struct ExtensionRegistry {
    /// `(extension_name, TypeId::of::<Box<dyn Trait>>())` → `RegistryEntry`
    handles: HashMap<(String, TypeId), RegistryEntry>,
}

impl ExtensionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    /// Insert pre-built trait registrations for an extension.
    ///
    /// Each [`TraitRegistration`] carries a cloned value and coerce function.
    /// This method inserts them into the registry keyed by `(name, trait_id)`.
    ///
    /// Called by the engine during pipeline build — not intended for direct use
    /// by extension writers.
    pub(crate) fn register_all(&mut self, name: &str, registrations: Vec<TraitRegistration>) {
        for reg in registrations {
            let entry = RegistryEntry {
                value: reg.value,
                coerce: reg.coerce,
            };
            let _ = self.handles.insert((name.to_string(), reg.trait_id), entry);
        }
    }

    /// Get an owned clone of a trait implementation by extension name.
    ///
    /// Returns `Box<dyn Trait>` — a fresh clone produced from the stored
    /// extension value. The clone shares any `Arc`-wrapped state with the
    /// original and with other clones.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn BearerTokenProvider`).
    ///
    /// # Errors
    ///
    /// Returns `ExtensionError::NotFound` if no extension with that name exists.
    /// Returns `ExtensionError::TraitNotImplemented` if the extension doesn't expose that trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let provider: Box<dyn BearerTokenProvider> = registry
    ///     .get::<dyn BearerTokenProvider>("azure_auth")?;
    /// provider.get_token().await?;
    /// ```
    pub fn get<T: ?Sized + 'static>(&self, name: &str) -> Result<Box<T>, ExtensionError> {
        let key = (name.to_string(), TypeId::of::<Box<T>>());
        let entry = self.handles.get(&key).ok_or_else(|| {
            // Distinguish "extension not found" from "trait not implemented"
            let has_any = self.handles.keys().any(|(n, _)| n == name);
            if has_any {
                ExtensionError::TraitNotImplemented {
                    name: name.to_string(),
                    expected: std::any::type_name::<T>(),
                }
            } else {
                ExtensionError::NotFound {
                    name: name.to_string(),
                }
            }
        })?;

        // Coerce produces Box<dyn Any + Send> that is actually Box<Box<dyn Trait>>.
        // Explicit deref (*entry.value) ensures we dispatch through the vtable
        // of `dyn CloneAnySend` to reach the concrete type, not the blanket
        // impl on `Box<dyn CloneAnySend>` itself.
        let erased = (entry.coerce)((*entry.value).as_any_ref());
        let double_boxed = erased
            .downcast::<Box<T>>()
            .expect("TypeId matched but downcast failed — this is a bug");

        Ok(*double_boxed)
    }

    /// Check if an extension exists by name.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.handles.keys().any(|(n, _)| n == name)
    }

    /// Returns the number of registered extensions (unique names).
    #[must_use]
    pub fn len(&self) -> usize {
        self.handles
            .keys()
            .map(|(n, _)| n)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Returns true if no extensions are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Returns an iterator over unique extension names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.handles
            .keys()
            .map(|(n, _)| n)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
    }
}

impl std::fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&String> = self.names().collect();
        f.debug_struct("ExtensionRegistry")
            .field("extensions", &names)
            .finish()
    }
}

/// Macro to declare which extension traits a concrete type implements.
///
/// Has two forms:
///
/// ## Convenience form (inside `impl Extension<PData>` block)
///
/// Expands to a complete
/// [`Extension::extension_traits`](crate::local::extension::Extension::extension_traits)
/// method definition. Place it directly inside an `impl Extension<PData>` block:
///
/// ```ignore
/// #[async_trait(?Send)]
/// impl Extension<OtapPdata> for MyExtension {
///     otap_df_engine::extension_traits!(BearerTokenProvider, SomeOtherTrait);
///
///     async fn start(...) { ... }
/// }
/// ```
///
/// ## Explicit form (returns `Vec<TraitRegistration>`)
///
/// Returns `Vec<TraitRegistration>` — self-contained registrations each carrying
/// a cloned copy of `self` and a monomorphised coerce function pointer.  The
/// extension writer returns this from
/// [`Extension::extension_traits`](crate::local::extension::Extension::extension_traits);
/// the engine inserts the registrations into the [`ExtensionRegistry`] by name.
///
/// ```ignore
/// fn extension_traits(&self) -> Vec<TraitRegistration> {
///     extension_traits!(self => BearerTokenProvider)
/// }
/// ```
///
/// # Type Safety
///
/// The macro verifies at compile time that:
/// - Each listed trait implements [`ExtensionTrait`] (sealed)
/// - The concrete type implements each listed trait plus `Clone + Send + 'static`
#[macro_export]
macro_rules! extension_traits {
    // Explicit form: `extension_traits!(self => Trait1, Trait2)`
    ($self:expr => $($trait:ident),* $(,)?) => {{
        let mut __regs: Vec<$crate::extension::registry::TraitRegistration> = Vec::new();
        $(
            {
                // Compile-time: ensure the trait is a sealed ExtensionTrait.
                const _: fn() = || {
                    fn assert_extension_trait<T: ?Sized + $crate::extension::registry::ExtensionTrait>() {}
                    assert_extension_trait::<dyn $trait>();
                };

                // Generic coerce fn — monomorphised for concrete T by the call
                // to `__make_reg` below.
                fn __coerce<T: Clone + Send + 'static + $trait>(
                    any: &dyn std::any::Any,
                ) -> Box<dyn std::any::Any + Send> {
                    let concrete = any
                        .downcast_ref::<T>()
                        .expect("registry entry type mismatch — this is a bug");
                    let cloned = concrete.clone();
                    let trait_obj: Box<dyn $trait> = Box::new(cloned);
                    Box::new(trait_obj) as Box<dyn std::any::Any + Send>
                }

                // Generic helper whose T is inferred from $self.
                fn __make_reg<T: Clone + Send + 'static + $trait>(
                    instance: &T,
                ) -> $crate::extension::registry::TraitRegistration {
                    $crate::extension::registry::TraitRegistration::new(
                        std::any::TypeId::of::<Box<dyn $trait>>(),
                        instance.clone(),
                        __coerce::<T>,
                    )
                }

                __regs.push(__make_reg($self));
            }
        )*
        __regs
    }};
    // Convenience form: `extension_traits!(Trait1, Trait2)`
    // Expands to a full method definition inside an `impl Extension` block.
    ($($trait:ident),* $(,)?) => {
        fn extension_traits(&self) -> Vec<$crate::extension::registry::TraitRegistration> {
            $crate::extension_traits!(self => $($trait),*)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::bearer_token_provider::BearerToken;
    use crate::extension::bearer_token_provider::BearerTokenProvider;
    use tokio::sync::watch;

    #[derive(Clone)]
    struct TestTokenProvider {
        token: String,
    }

    #[async_trait::async_trait]
    impl BearerTokenProvider for TestTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, Error> {
            Ok(BearerToken::new(self.token.clone(), 0))
        }

        fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>> {
            let (tx, rx) = watch::channel(None);
            drop(tx);
            rx
        }
    }

    /// Helper: register a TestTokenProvider with the given name.
    fn register_provider(registry: &mut ExtensionRegistry, name: &str, token: &str) {
        let instance = TestTokenProvider {
            token: token.to_string(),
        };
        let regs = crate::extension_traits!(&instance => BearerTokenProvider);
        registry.register_all(name, regs);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "test_ext", "test_token");

        let result: Result<Box<dyn BearerTokenProvider>, _> =
            registry.get::<dyn BearerTokenProvider>("test_ext");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_returns_independent_clones() {
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "ext", "shared_test");

        let a: Box<dyn BearerTokenProvider> =
            registry.get::<dyn BearerTokenProvider>("ext").unwrap();
        let b: Box<dyn BearerTokenProvider> =
            registry.get::<dyn BearerTokenProvider>("ext").unwrap();

        // Both are independent clones (different pointers)
        assert!(!std::ptr::eq(
            &*a as *const dyn BearerTokenProvider,
            &*b as *const dyn BearerTokenProvider,
        ));
    }

    #[test]
    fn test_registry_clone_produces_deep_copy() {
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "ext", "clone_test");

        let cloned = registry.clone();

        let from_original: Box<dyn BearerTokenProvider> =
            registry.get::<dyn BearerTokenProvider>("ext").unwrap();
        let from_clone: Box<dyn BearerTokenProvider> =
            cloned.get::<dyn BearerTokenProvider>("ext").unwrap();

        // Deep copy — different pointers
        assert!(!std::ptr::eq(
            &*from_original as *const dyn BearerTokenProvider,
            &*from_clone as *const dyn BearerTokenProvider,
        ));
    }

    #[test]
    fn test_not_found() {
        let registry = ExtensionRegistry::new();
        let result = registry.get::<dyn BearerTokenProvider>("missing");
        assert!(matches!(result, Err(ExtensionError::NotFound { .. })));
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
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "test_ext", "test");

        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ExtensionRegistry"));
        assert!(debug_str.contains("test_ext"));
    }

    #[test]
    fn test_contains_and_len() {
        let mut registry = ExtensionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        register_provider(&mut registry, "ext", "test");
        assert!(registry.contains("ext"));
        assert!(!registry.contains("missing"));
        assert_eq!(registry.len(), 1);
    }

    #[tokio::test]
    async fn test_get_extension_actually_works() {
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "auth", "real_token");

        let provider: Box<dyn BearerTokenProvider> =
            registry.get::<dyn BearerTokenProvider>("auth").unwrap();
        let token = provider.get_token().await.unwrap();
        assert_eq!(token.token.secret(), "real_token");
    }

    #[test]
    fn test_multiple_extensions_same_trait() {
        let mut registry = ExtensionRegistry::new();
        register_provider(&mut registry, "azure_prod", "prod_token");
        register_provider(&mut registry, "azure_staging", "staging_token");

        assert_eq!(registry.len(), 2);

        let _p1 = registry
            .get::<dyn BearerTokenProvider>("azure_prod")
            .unwrap();
        let _p2 = registry
            .get::<dyn BearerTokenProvider>("azure_staging")
            .unwrap();
    }

    #[test]
    fn test_registry_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ExtensionRegistry>();
    }
}
