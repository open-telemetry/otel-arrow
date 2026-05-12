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
///
/// Inserted only for **native** local registrations. The
/// SharedAsLocal fallback path does *not* produce a separate local
/// entry — instead, [`Capabilities::require_local`] falls through to
/// the [`ResolvedSharedEntry`] when no native local entry exists,
/// taking the shared `produce` cell and routing its output through
/// the entry's `adapt_as_local` fn pointer. This collapses the
/// fallback's two former entries into one, so the per-binding
/// one-shot guard is encoded directly in the shared entry's
/// `Cell::take()` — no separate claim flag needed.
pub(crate) struct ResolvedLocalEntry {
    /// Per-node produce closure, cloned from the registry entry.
    ///
    /// Wrapped in `Cell<Option<_>>` so the consumer-side one-shot
    /// guard is encoded in the type: `Some` = unclaimed, `None` =
    /// already claimed. Consumption is a single `Cell::take()` that
    /// atomically moves the closure out and marks the binding as
    /// claimed; a second `take()` returns `None` and yields
    /// [`Error::CapabilityAlreadyConsumed`](super::Error::CapabilityAlreadyConsumed).
    pub(crate) produce: Cell<Option<Box<dyn LocalProduce>>>,
    /// Cross-node consumption flag for the local variant, shared
    /// with the [`ConsumedTracker`].
    pub(crate) tracker_consumed: Rc<Cell<bool>>,
}

/// A resolved shared capability entry for a specific node.
///
/// Serves two roles. As a native shared entry, [`Capabilities::require_shared`]
/// takes its `produce` cell to mint a `Box<dyn C::Shared>`. As the
/// SharedAsLocal fallback for a binding with no native local entry,
/// [`Capabilities::require_local`] takes the same cell and routes the
/// output through `adapt_as_local` to mint an `Rc<dyn C::Local>`.
/// Either path consumes the single underlying [`Cell::take()`], so the
/// per-binding one-shot contract is enforced naturally without an
/// auxiliary claim flag.
pub(crate) struct ResolvedSharedEntry {
    /// Per-node produce closure, cloned from the registry entry.
    ///
    /// Wrapped in `Cell<Option<_>>` for the same reason as
    /// [`ResolvedLocalEntry::produce`]: `Cell::take()` doubles as
    /// the one-shot consumer-side guard, and is shared between the
    /// native shared and SharedAsLocal-fallback paths.
    pub(crate) produce: Cell<Option<Box<dyn SharedProduce>>>,
    /// Cross-node consumption flag, shared with the
    /// [`ConsumedTracker`].
    pub(crate) tracker_consumed: Rc<Cell<bool>>,
    /// Adapter fn pointer used by the SharedAsLocal fallback path
    /// in [`Capabilities::require_local`]. Takes a produced shared
    /// trait object (erased as `Box<dyn Any + Send>`) and returns a
    /// local trait object (erased as `Rc<dyn Any>` wrapping
    /// `Rc<dyn C::Local>`). Originally minted by the
    /// `#[capability]`-generated `shared_entry::<E>` caster.
    pub(crate) adapt_as_local: fn(Box<dyn Any + Send>) -> Rc<dyn Any>,
}
