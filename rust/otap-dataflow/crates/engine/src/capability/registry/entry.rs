// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-erased capability entries.
//!
//! Two families:
//!
//! - **Registry entries** ([`LocalCapabilityEntry`], [`SharedCapabilityEntry`]):
//!   what [`CapabilityRegistry`](super::CapabilityRegistry) stores per
//!   `(capability, extension)` registration. Each owns a produce
//!   closure minted by the `#[capability]`-generated caster
//!   (`shared_entry::<E>` / `local_entry::<E>`) which internally calls
//!   the extension's instance factory, downcasts to the concrete
//!   extension type, and coerces to the capability's trait object.
//! - **Resolved entries** ([`ResolvedLocalEntry`], [`ResolvedSharedEntry`]):
//!   what [`resolve_bindings`](super::resolve_bindings) produces per
//!   node, each holding a cloned produce closure and a shared
//!   consumption cell.

use otap_df_config::ExtensionId;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

// ── Cloneable produce closures ──────────────────────────────────────────────
//
// `Box<dyn Fn>` is not `Clone`, so we thread cloning through an
// object-safe `clone_box` method. One extension may provide multiple
// capabilities; each capability needs its own produce closure, and
// `resolve_bindings` hands per-node entries their own copy.

/// Object-safe `Fn + Clone` producing an erased shared trait object.
///
/// The stored closure returns `Box<Box<dyn C::Shared>>` erased as
/// `Box<dyn Any + Send>` — the double-box envelope the registry uses.
#[doc(hidden)]
pub trait SharedProduce: Fn() -> Box<dyn Any + Send> + Send {
    fn clone_box(&self) -> Box<dyn SharedProduce>;
}

impl<F> SharedProduce for F
where
    F: Fn() -> Box<dyn Any + Send> + Send + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn SharedProduce> {
        Box::new(self.clone())
    }
}

/// Object-safe `Fn + Clone` producing an erased local trait object.
///
/// The stored closure returns `Rc<Rc<dyn C::Local>>` erased as
/// `Rc<dyn Any>` — the double-`Rc` envelope the registry uses.
#[doc(hidden)]
pub trait LocalProduce: Fn() -> Rc<dyn Any> {
    fn clone_box(&self) -> Box<dyn LocalProduce>;
}

impl<F> LocalProduce for F
where
    F: Fn() -> Rc<dyn Any> + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn LocalProduce> {
        Box::new(self.clone())
    }
}

// ── Registry entries ────────────────────────────────────────────────────────

/// A type-erased local (!Send) capability entry.
///
/// The `produce` closure — built by the `#[capability]`-generated
/// `local_entry::<E>` caster — mints a fresh `Rc<dyn C::Local>` by
/// calling the extension's `LocalInstanceFactory`, downcasting to `E`,
/// and coercing to the trait object.
#[doc(hidden)]
pub struct LocalCapabilityEntry {
    /// The extension that provided this capability.
    pub(crate) extension_id: ExtensionId,
    /// Produce a fresh local trait object erased as `Rc<dyn Any>`.
    pub(crate) produce: Box<dyn LocalProduce>,
}

impl LocalCapabilityEntry {
    /// Construct a new local entry from a produce closure.
    #[must_use]
    pub fn new<F>(extension_id: ExtensionId, produce: F) -> Self
    where
        F: Fn() -> Rc<dyn Any> + Clone + 'static,
    {
        Self {
            extension_id,
            produce: Box::new(produce),
        }
    }
}

/// A type-erased shared (Send) capability entry.
///
/// Two closures/fn pointers, both minted by the
/// `#[capability]`-generated `shared_entry::<E>` caster:
///
/// - `produce` — builds `Box<Box<dyn C::Shared>>` erased as
///   `Box<dyn Any + Send>`. Called per consumer.
/// - `adapt_as_local` — takes that same erased double box and returns
///   `Rc<Rc<dyn C::Local>>` erased as `Rc<dyn Any>`. Used by the
///   `SharedAsLocal` fallback path in `resolve_bindings`.
#[doc(hidden)]
pub struct SharedCapabilityEntry {
    /// The extension that provided this capability.
    pub(crate) extension_id: ExtensionId,
    /// Produce a fresh shared trait object erased as
    /// `Box<dyn Any + Send>`.
    pub(crate) produce: Box<dyn SharedProduce>,
    /// Turn a produced shared trait object (erased) into a local trait
    /// object (erased). Only called during resolve for the
    /// `SharedAsLocal` fallback.
    pub(crate) adapt_as_local: fn(Box<dyn Any + Send>) -> Rc<dyn Any>,
}

impl SharedCapabilityEntry {
    /// Construct a new shared entry from a produce closure and an
    /// adapt-as-local fn pointer.
    #[must_use]
    pub fn new<F>(
        extension_id: ExtensionId,
        produce: F,
        adapt_as_local: fn(Box<dyn Any + Send>) -> Rc<dyn Any>,
    ) -> Self
    where
        F: Fn() -> Box<dyn Any + Send> + Send + Clone + 'static,
    {
        Self {
            extension_id,
            produce: Box::new(produce),
            adapt_as_local,
        }
    }
}

// ── Resolved entries (per-node) ─────────────────────────────────────────────

/// A resolved local capability entry for a specific node.
pub(crate) struct ResolvedLocalEntry {
    /// Per-node produce closure, cloned from the registry entry.
    pub(crate) produce: Box<dyn LocalProduce>,
    /// Consumption flag for the underlying extension variant.
    ///
    /// - For native local bindings: cell lives under the tracker's
    ///   *local* bucket for the `(capability, extension)` pair.
    /// - For `SharedAsLocal` bindings: cell lives under the tracker's
    ///   *shared* bucket — consuming the adapter is considered a use
    ///   of the shared variant.
    pub(crate) consumed: Rc<Cell<bool>>,
}

/// A resolved shared capability entry for a specific node.
pub(crate) struct ResolvedSharedEntry {
    /// Per-node produce closure, cloned from the registry entry.
    pub(crate) produce: Box<dyn SharedProduce>,
    /// Consumption flag, shared with the `ConsumedTracker`.
    pub(crate) consumed: Rc<Cell<bool>>,
}
