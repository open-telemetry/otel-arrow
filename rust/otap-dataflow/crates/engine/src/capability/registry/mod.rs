// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability registry and resolution.
//!
//! [`CapabilityRegistry`] collects type-erased capability entries from
//! extensions during the build phase. [`Capabilities`] is produced
//! per-node by [`resolve_bindings`] and provides the consumer API
//! (`require_local`, `require_shared`, `optional_local`,
//! `optional_shared`).
//!
//! # Module layout
//!
//! | Submodule        | Contents                                                                      |
//! |------------------|-------------------------------------------------------------------------------|
//! | [`entry`]        | [`LocalCapabilityEntry`], [`SharedCapabilityEntry`], plus resolved-entry types |
//! | [`storage`]      | [`CapabilityRegistry`] — build-phase storage of registrations                  |
//! | [`capabilities`] | [`Capabilities`] — per-node consumer API                                       |
//! | [`tracker`]      | [`ConsumedTracker`] — records which variants were consumed                     |
//! | [`resolve`]      | [`resolve_bindings`] — the per-node resolution pass                            |

mod capabilities;
mod entry;
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
pub use storage::CapabilityRegistry;

// ── Crate-internal re-exports ────────────────────────────────────────────────

pub(crate) use entry::{ResolvedLocalEntry, ResolvedSharedEntry};
// TODO(extension-system): wired by engine build phase in a follow-up PR;
// until then only the test module imports it via `use super::*;`.
#[allow(unused_imports)]
pub(crate) use resolve::resolve_bindings;
pub(crate) use tracker::ConsumedTracker;
