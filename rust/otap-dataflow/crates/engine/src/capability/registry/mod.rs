// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability registry and resolution.
//!
//! [`CapabilityRegistry`] collects type-erased capability trait objects from
//! extensions during the build phase. [`Capabilities`] is produced per-node
//! by [`resolve_bindings`] and provides the consumer API
//! (`require_local`, `require_shared`, `optional_local`, `optional_shared`).
//!
//! TODO(extension-system): the pieces in this module are scaffolding —
//! exercised only by the unit tests below. The engine build phase will
//! populate the registry from extension factories, invoke
//! [`resolve_bindings`] per node, and use the [`ConsumedTracker`]'s
//! `unconsumed_local` / `unconsumed_shared` methods to drop unused
//! extension variants.
//!
//! # Module layout
//!
//! | Submodule        | Contents                                                                                                         |
//! |------------------|------------------------------------------------------------------------------------------------------------------|
//! | [`factory`]      | [`SharedCapabilityFactory`], [`LocalCapabilityFactory`] traits and the ClonePerConsumer / FreshPerConsumer impls |
//! | [`entry`]        | [`LocalCapabilityEntry`], [`SharedCapabilityEntry`], plus per-node resolved-entry structs                        |
//! | [`storage`]      | [`CapabilityRegistry`] — build-phase storage of registrations                                                    |
//! | [`capabilities`] | [`Capabilities`] — per-node consumer API                                                                         |
//! | [`tracker`]      | [`ConsumedTracker`] — records which variants were consumed                                                       |
//! | [`resolve`]      | [`resolve_bindings`] — the per-node resolution pass                                                              |

mod capabilities;
mod entry;
mod factory;
mod resolve;
mod storage;
mod tracker;

#[cfg(test)]
mod tests;

/// Error type alias for capability operations.
pub type Error = crate::error::Error;

// ── Public re-exports ────────────────────────────────────────────────────────

pub use capabilities::Capabilities;
pub use entry::{LocalCapabilityEntry, SharedCapabilityEntry};
pub use factory::{
    ClonePerConsumerLocalFactory, ClonePerConsumerSharedFactory, FreshPerConsumerLocalFactory,
    FreshPerConsumerSharedFactory, LocalCapabilityFactory, SharedCapabilityFactory,
};
pub use storage::CapabilityRegistry;

// ── Crate-internal re-exports ────────────────────────────────────────────────

pub(crate) use entry::{ResolvedLocalEntry, ResolvedSharedEntry};
pub(crate) use factory::downcast_produced;
// TODO(extension-system): wired by engine build phase in a follow-up PR;
// until then only the test module imports it via `use super::*;`.
#[allow(unused_imports)]
pub(crate) use resolve::resolve_bindings;
pub(crate) use tracker::ConsumedTracker;
