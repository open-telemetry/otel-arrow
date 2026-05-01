// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`ConsumedTracker`] — records which capability variants were consumed
//! by node factories so the engine can drop unused extension *variants*
//! after the build phase.
//!
//! The tracker observes capability **consumption**, not extension
//! lifetime. It exists solely to answer: "for each capability variant an
//! extension exposed, did any node bind to it?" If the answer is no, the
//! engine drops that variant (`drop_local` / `drop_shared`). The
//! tracker has no opinion on whether the extension itself keeps running
//! — an extension's `start()` event loop is wholly independent of
//! tracker state.
//!
//! Background extensions (the lifecycle that registers an engine-driven
//! event loop and exposes **zero** capabilities) are intentionally
//! absent from this structure: they have nothing for any node to
//! consume, so there are no `(TypeId, ExtensionId)` keys to track and
//! the engine never calls `drop_local` / `drop_shared` on them.

use otap_df_config::ExtensionId;
use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

/// Tracks which capability variants were consumed by node factories.
///
/// Created alongside per-node [`Capabilities`](super::Capabilities)
/// during [`resolve_bindings`](super::resolve_bindings). After all node
/// factories have run, the engine inspects this tracker to determine
/// which extension *variants* are unused and can be dropped.
///
/// Keyed by `(capability TypeId, extension ID)` so multiple providers of
/// the same capability are tracked independently. An extension that
/// exposes N capabilities therefore appears under N distinct keys in
/// each of [`unconsumed_local`](Self::unconsumed_local) /
/// [`unconsumed_shared`](Self::unconsumed_shared) — once per
/// `(TypeId, ExtensionId)` pair — so the same `ExtensionId` shows up
/// multiple times when iterating, once per capability it provides.
/// The `Rc<Cell<bool>>` for a given key is shared across all nodes that
/// bind to that provider — once any of them consumes the capability the
/// cell is set.
pub(crate) struct ConsumedTracker {
    local: HashMap<(TypeId, ExtensionId), ConsumedEntry>,
    shared: HashMap<(TypeId, ExtensionId), ConsumedEntry>,
}

impl std::fmt::Debug for ConsumedTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `TypeId` is `Debug` but its output is opaque (`TypeId { t: ... }`);
        // summarize counts instead. For detail, see `unconsumed_local` /
        // `unconsumed_shared`, which return human-readable names.
        f.debug_struct("ConsumedTracker")
            .field("local_slots", &self.local.len())
            .field("shared_slots", &self.shared.len())
            .finish()
    }
}

/// A single consumption tracking entry.
pub(crate) struct ConsumedEntry {
    /// Human-readable capability name (for warnings).
    pub(crate) name: &'static str,
    /// The extension that provides this capability.
    pub(crate) extension_id: ExtensionId,
    /// Shared flag — set to `true` when any node consumes this variant.
    pub(crate) consumed: Rc<Cell<bool>>,
}

impl ConsumedTracker {
    /// Creates an empty tracker.
    pub(crate) fn new() -> Self {
        ConsumedTracker {
            local: HashMap::new(),
            shared: HashMap::new(),
        }
    }

    /// Returns the `Rc<Cell<bool>>` tracking local consumption for this
    /// `(capability, extension)` pair, creating a fresh cell (initialized
    /// to `false`) if none exists yet. This **registers** the consumer
    /// slot; it does **not** mark the capability as consumed. The cell
    /// is flipped to `true` later, only when a consumer calls
    /// [`Capabilities::require_local`](super::Capabilities::require_local)
    /// / [`Capabilities::optional_local`](super::Capabilities::optional_local).
    pub(crate) fn ensure_local_consumer_slot(
        &mut self,
        capability_id: TypeId,
        name: &'static str,
        extension_id: ExtensionId,
    ) -> Rc<Cell<bool>> {
        let entry = self
            .local
            .entry((capability_id, extension_id.clone()))
            .or_insert_with(|| ConsumedEntry {
                name,
                extension_id,
                consumed: Rc::new(Cell::new(false)),
            });
        Rc::clone(&entry.consumed)
    }

    /// Returns the `Rc<Cell<bool>>` tracking shared consumption for this
    /// `(capability, extension)` pair, creating a fresh cell (initialized
    /// to `false`) if none exists yet. This **registers** the consumer
    /// slot; it does **not** mark the capability as consumed. The cell
    /// is flipped to `true` later, only when a consumer calls
    /// [`Capabilities::require_shared`](super::Capabilities::require_shared)
    /// / [`Capabilities::optional_shared`](super::Capabilities::optional_shared)
    /// (or [`Capabilities::require_local`](super::Capabilities::require_local)
    /// / [`Capabilities::optional_local`](super::Capabilities::optional_local)
    /// for an extension that only provides a shared variant).
    pub(crate) fn ensure_shared_consumer_slot(
        &mut self,
        capability_id: TypeId,
        name: &'static str,
        extension_id: ExtensionId,
    ) -> Rc<Cell<bool>> {
        let entry = self
            .shared
            .entry((capability_id, extension_id.clone()))
            .or_insert_with(|| ConsumedEntry {
                name,
                extension_id,
                consumed: Rc::new(Cell::new(false)),
            });
        Rc::clone(&entry.consumed)
    }

    /// Returns extension IDs whose local variant was never consumed.
    /// Used by the engine to call `drop_local()` on those extensions.
    pub(crate) fn unconsumed_local(&self) -> Vec<(ExtensionId, &'static str)> {
        self.local
            .values()
            .filter(|e| !e.consumed.get())
            .map(|e| (e.extension_id.clone(), e.name))
            .collect()
    }

    /// Returns extension IDs whose shared variant was never consumed.
    /// Used by the engine to call `drop_shared()` on those extensions.
    pub(crate) fn unconsumed_shared(&self) -> Vec<(ExtensionId, &'static str)> {
        self.shared
            .values()
            .filter(|e| !e.consumed.get())
            .map(|e| (e.extension_id.clone(), e.name))
            .collect()
    }
}
