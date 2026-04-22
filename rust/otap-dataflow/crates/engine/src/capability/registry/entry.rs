// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-erased capability entries.
//!
//! Two families live here:
//!
//! - **Registry entries** ([`LocalCapabilityEntry`], [`SharedCapabilityEntry`]):
//!   what [`CapabilityRegistry`](super::CapabilityRegistry) stores per
//!   `(capability, extension)` registration.
//! - **Resolved entries** ([`ResolvedLocalEntry`], [`ResolvedSharedEntry`]):
//!   what [`resolve_bindings`](super::resolve_bindings) produces per node,
//!   each holding a cloned factory and a shared consumption cell.

use super::{LocalCapabilityFactory, SharedCapabilityFactory};
use otap_df_config::ExtensionId;
use std::cell::Cell;
use std::rc::Rc;

/// A type-erased local (!Send) capability entry.
///
/// Holds a [`LocalCapabilityFactory`] whose concrete type encodes the
/// instance policy for this `(capability, extension)` registration.
/// Today the only impl is
/// [`ClonePerConsumerLocalFactory`](super::ClonePerConsumerLocalFactory);
/// future policies will plug in as additional impls without changing
/// this struct.
///
/// Extensions that only register a shared variant are handled by
/// [`resolve_bindings`](super::resolve_bindings): it invokes the shared
/// factory's
/// [`adapt_as_local_any`](SharedCapabilityFactory::adapt_as_local_any)
/// and wraps the result in a `ClonePerConsumerLocalFactory` so the
/// `SharedAsLocal` path flows through the same `LocalCapabilityFactory`
/// pipeline as a native local registration.
#[doc(hidden)]
pub struct LocalCapabilityEntry {
    /// The extension that provided this capability.
    pub(crate) extension_id: ExtensionId,
    /// Factory supplying local trait objects for this `(cap, ext)` pair.
    pub(crate) factory: Box<dyn LocalCapabilityFactory>,
}

/// A type-erased shared (Send) capability entry.
///
/// [`factory`](Self::factory) is a per-`(capability, extension)` producer
/// that can both clone itself and mint `Box<dyn shared::Trait>`
/// instances. See [`SharedCapabilityFactory`].
#[doc(hidden)]
pub struct SharedCapabilityEntry {
    /// The extension that provided this capability.
    pub(crate) extension_id: ExtensionId,
    /// Cloneable factory for shared capability trait objects.
    pub(crate) factory: Box<dyn SharedCapabilityFactory>,
}

// ── Resolved entries (per-node) ──────────────────────────────────────────────

/// A resolved local capability entry for a specific node.
pub(crate) struct ResolvedLocalEntry {
    /// Per-node local factory, produced by cloning the registry
    /// entry's factory. The concrete impl encodes the instance policy
    /// (today always
    /// [`ClonePerConsumerLocalFactory`](super::ClonePerConsumerLocalFactory)).
    pub(crate) factory: Box<dyn LocalCapabilityFactory>,
    /// Consumption flag for the underlying extension variant.
    ///
    /// - For native local bindings: the cell lives under the tracker's
    ///   *local* bucket for the `(capability, extension)` pair.
    /// - For `SharedAsLocal` bindings: the cell lives under the tracker's
    ///   *shared* bucket — consuming the adapter is considered a use of
    ///   the shared variant, because that's the only variant the
    ///   extension actually provided. No phantom local entry is recorded.
    pub(crate) consumed: Rc<Cell<bool>>,
}

/// A resolved shared capability entry for a specific node.
pub(crate) struct ResolvedSharedEntry {
    /// Per-node factory, produced by cloning the registry entry's factory.
    /// Each node owns an independent `Box` (no cross-thread aliasing).
    pub(crate) factory: Box<dyn SharedCapabilityFactory>,
    /// Consumption flag, shared with the `ConsumedTracker`.
    pub(crate) consumed: Rc<Cell<bool>>,
}
