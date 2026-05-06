// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-erased instance factories for capability registration.
//!
//! These closures produce fresh instances of an extension's concrete type
//! on demand. They are the bridge between the extension's instance policy
//! (`Cloned` vs `Fresh`, chosen at the builder) and the capability
//! registry, which consumes them to mint per-node capability entries.
//!
//! Living in [`crate::capability`] rather than [`crate::extension`]
//! breaks the module-level cycle between the two folders: the capability
//! registry (and [`ExtensionCapabilities`](super::ExtensionCapabilities))
//! needs these types to describe registration fn pointers, while the
//! extension builder needs them to encode instance policy. Placing them
//! with the registry side makes the dependency one-way:
//! `extension → capability`.

use std::any::Any;

// ── Shared ───────────────────────────────────────────────────────────────────

/// Produces instances of a shared extension's concrete type for capability
/// consumers.
///
/// The wrapper is type-erased over the extension's concrete type `E`. When
/// the engine wires a consumer's capability binding, the generated
/// registration glue (see `extension_capabilities!`) downcasts the returned
/// `Box<dyn Any + Send>` back to `E` and wraps it in the requested
/// capability trait object.
///
/// The instance policy chosen at the builder (`.cloned(...)` vs
/// `.fresh(...)`) is baked into the stored closure:
/// - **Cloned** — the closure captures a prototype `E: Clone`
///   and returns `e.clone()` on each call.
/// - **Fresh** — the closure captures the user-supplied
///   `Fn() -> E` and invokes it on each call.
///
/// `SharedInstanceFactory` is [`Clone`]: one extension may provide
/// multiple capabilities, and each capability registration needs its
/// own copy of the factory for its produce closure.
pub struct SharedInstanceFactory {
    produce: Box<dyn SharedFnClone>,
}

/// Object-safe `Fn + Clone` helper for [`SharedInstanceFactory`].
///
/// `Box<dyn Fn>` is not `Clone`, so we thread cloning through an
/// object-safe `clone_box` method. The blanket impl below covers any
/// closure that is `Fn + Send + Clone + 'static`.
#[doc(hidden)]
pub trait SharedFnClone: Fn() -> Box<dyn Any + Send> + Send {
    fn clone_box(&self) -> Box<dyn SharedFnClone>;
}

impl<F> SharedFnClone for F
where
    F: Fn() -> Box<dyn Any + Send> + Send + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn SharedFnClone> {
        Box::new(self.clone())
    }
}

impl SharedInstanceFactory {
    /// Construct a factory from a closure producing type-erased instances.
    #[must_use]
    pub fn new<F>(produce: F) -> Self
    where
        F: Fn() -> Box<dyn Any + Send> + Send + Clone + 'static,
    {
        SharedInstanceFactory {
            produce: Box::new(produce),
        }
    }

    /// Produce a fresh instance as `Box<dyn Any + Send>`.
    #[must_use]
    pub fn produce(&self) -> Box<dyn Any + Send> {
        (self.produce)()
    }
}

impl Clone for SharedInstanceFactory {
    fn clone(&self) -> Self {
        SharedInstanceFactory {
            produce: self.produce.clone_box(),
        }
    }
}

// ── Local ────────────────────────────────────────────────────────────────────

/// Produces instances of a local (!Send) extension's concrete type for
/// capability consumers. See [`SharedInstanceFactory`] for background.
///
/// `LocalInstanceFactory` is [`Clone`] for the same reason as its
/// shared counterpart.
pub struct LocalInstanceFactory {
    produce: Box<dyn LocalFnClone>,
}

/// Object-safe `Fn + Clone` helper for [`LocalInstanceFactory`].
#[doc(hidden)]
pub trait LocalFnClone: Fn() -> std::rc::Rc<dyn Any> {
    fn clone_box(&self) -> Box<dyn LocalFnClone>;
}

impl<F> LocalFnClone for F
where
    F: Fn() -> std::rc::Rc<dyn Any> + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn LocalFnClone> {
        Box::new(self.clone())
    }
}

impl LocalInstanceFactory {
    /// Construct a factory from a closure producing type-erased instances.
    #[must_use]
    pub fn new<F>(produce: F) -> Self
    where
        F: Fn() -> std::rc::Rc<dyn Any> + Clone + 'static,
    {
        LocalInstanceFactory {
            produce: Box::new(produce),
        }
    }

    /// Produce an instance as `Rc<dyn Any>`.
    #[must_use]
    pub fn produce(&self) -> std::rc::Rc<dyn Any> {
        (self.produce)()
    }
}

impl Clone for LocalInstanceFactory {
    fn clone(&self) -> Self {
        LocalInstanceFactory {
            produce: self.produce.clone_box(),
        }
    }
}
