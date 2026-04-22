// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`CapabilityRegistry`] — the build-phase store of capability
//! registrations, keyed by `(capability TypeId, extension ID)` and
//! split into local (!Send) and shared (Send) buckets.

use super::{Error, LocalCapabilityEntry, SharedCapabilityEntry};
use otap_df_config::ExtensionId;
use std::any::TypeId;
use std::collections::HashMap;

/// Collects type-erased capability registrations from extensions during
/// the pipeline build phase.
///
/// Keyed by `TypeId` of the zero-sized `ExtensionCapability` registration
/// struct (e.g., `TypeId::of::<BearerTokenProvider>()`).
///
/// After all extensions have registered, call
/// [`resolve_bindings`](super::resolve_bindings) to produce per-node
/// [`Capabilities`](super::Capabilities) structs.
pub struct CapabilityRegistry {
    /// Local entries keyed by (capability TypeId, extension name).
    local: HashMap<TypeId, HashMap<ExtensionId, LocalCapabilityEntry>>,
    /// Shared entries keyed by (capability TypeId, extension name).
    shared: HashMap<TypeId, HashMap<ExtensionId, SharedCapabilityEntry>>,
}

impl CapabilityRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        CapabilityRegistry {
            local: HashMap::new(),
            shared: HashMap::new(),
        }
    }

    /// Register a local capability entry for the given capability and extension.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InternalError`] if an entry already exists for
    /// the same `(capability_id, entry.extension_id)` pair. Each
    /// extension must register a given capability at most once; a
    /// duplicate indicates an engine-side or extension-author bug.
    pub fn register_local(
        &mut self,
        capability_id: TypeId,
        entry: LocalCapabilityEntry,
    ) -> Result<(), Error> {
        let ext_id = entry.extension_id.clone();
        let slot = self.local.entry(capability_id).or_default();
        if slot.contains_key(&ext_id) {
            return Err(Error::InternalError {
                message: format!(
                    "duplicate local capability registration: extension '{ext_id}' already registered capability TypeId {capability_id:?}",
                ),
            });
        }
        let _ = slot.insert(ext_id, entry);
        Ok(())
    }

    /// Register a shared capability entry for the given capability and extension.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InternalError`] if an entry already exists for
    /// the same `(capability_id, entry.extension_id)` pair. Each
    /// extension must register a given capability at most once; a
    /// duplicate indicates an engine-side or extension-author bug.
    pub fn register_shared(
        &mut self,
        capability_id: TypeId,
        entry: SharedCapabilityEntry,
    ) -> Result<(), Error> {
        let ext_id = entry.extension_id.clone();
        let slot = self.shared.entry(capability_id).or_default();
        if slot.contains_key(&ext_id) {
            return Err(Error::InternalError {
                message: format!(
                    "duplicate shared capability registration: extension '{ext_id}' already registered capability TypeId {capability_id:?}",
                ),
            });
        }
        let _ = slot.insert(ext_id, entry);
        Ok(())
    }

    /// Look up a local capability entry by capability type and extension name.
    #[must_use]
    pub fn get_local(
        &self,
        capability_id: &TypeId,
        extension_id: &str,
    ) -> Option<&LocalCapabilityEntry> {
        self.local
            .get(capability_id)
            .and_then(|m| m.get(extension_id))
    }

    /// Look up a shared capability entry by capability type and extension name.
    #[must_use]
    pub fn get_shared(
        &self,
        capability_id: &TypeId,
        extension_id: &str,
    ) -> Option<&SharedCapabilityEntry> {
        self.shared
            .get(capability_id)
            .and_then(|m| m.get(extension_id))
    }

    /// Returns `true` if any extension provides a **native** local entry
    /// for this capability. A local binding is also reachable whenever
    /// [`has_shared`](Self::has_shared) is true — shared registrations
    /// are always adaptable via `SharedAsLocal`. For the composite
    /// "can a node bind this as local?" predicate, use
    /// `has_native_local(id) || has_shared(id)`.
    #[must_use]
    pub(crate) fn has_native_local(&self, capability_id: &TypeId) -> bool {
        self.local
            .get(capability_id)
            .is_some_and(|m| !m.is_empty())
    }

    /// Returns `true` if any extension provides a shared entry for this capability.
    #[must_use]
    pub(crate) fn has_shared(&self, capability_id: &TypeId) -> bool {
        self.shared
            .get(capability_id)
            .is_some_and(|m| !m.is_empty())
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CapabilityRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The entry values hold type-erased produce closures
        // (`dyn SharedProduce` / `dyn LocalProduce`) and can't be
        // printed. Summarize the shape instead.
        f.debug_struct("CapabilityRegistry")
            .field("local_capabilities", &self.local.len())
            .field("shared_capabilities", &self.shared.len())
            .finish()
    }
}
