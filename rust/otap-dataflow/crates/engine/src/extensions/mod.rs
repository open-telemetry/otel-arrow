//! Extension traits and registry for capability-based lookups.
//!
//! This module provides:
//! - [`ExtensionBundle`](registry::ExtensionBundle) - A collection of trait implementations for a single extension
//! - [`ExtensionRegistry`](registry::ExtensionRegistry) - A registry to look up extension traits by name
//! - Common extension traits like [`TokenProvider`](token_provider::TokenProvider)
//!
//! # Adding New Extension Traits
//!
//! All extension traits must implement the [`ExtensionTrait`] marker trait to be usable
//! with [`ExtensionBundle`](registry::ExtensionBundle). This ensures type safety and
//! restricts the bundle to only contain recognized extension capabilities.
//!
//! ```ignore
//! pub trait MyNewTrait: ExtensionTrait {
//!     fn my_method(&self);
//! }
//! ```

pub mod registry;

// Re-export commonly used types
pub use registry::{ExtensionBundle, ExtensionError, ExtensionRegistry, ExtensionRegistryBuilder};

/// Extension traits that components can implement to expose capabilities.
pub mod token_provider;

/// Marker trait for all extension traits.
///
/// This trait must be implemented by any trait that can be stored in an
/// [`ExtensionBundle`](registry::ExtensionBundle). It ensures that only
/// recognized extension capabilities can be registered.
///
/// # Defining a New Extension Trait
///
/// ```ignore
/// use otap_df_engine::extensions::ExtensionTrait;
///
/// // The extension trait must have ExtensionTrait as a supertrait
/// pub trait MyCapability: ExtensionTrait {
///     fn do_something(&self);
/// }
///
/// // Concrete implementations must implement ExtensionTrait
/// struct MyImpl;
/// impl ExtensionTrait for MyImpl {}
/// impl MyCapability for MyImpl {
///     fn do_something(&self) { /* ... */ }
/// }
/// ```
pub trait ExtensionTrait: Send + Sync {}

pub use token_provider::TokenProvider;