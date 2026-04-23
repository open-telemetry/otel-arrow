// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Capabilities`] — the per-node consumer API for resolved capability
//! bindings, with `require_*` and `optional_*` accessors.

use super::{Error, ResolvedLocalEntry, ResolvedSharedEntry};
use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;

/// Per-node capability bindings resolved from the
/// [`CapabilityRegistry`](super::CapabilityRegistry).
///
/// Passed to node factories as `&Capabilities`. Provides type-safe
/// access to extension capabilities via the [`ExtensionCapability`]
/// sealed trait.
///
/// [`ExtensionCapability`]: crate::capability::ExtensionCapability
pub struct Capabilities {
    local: HashMap<TypeId, ResolvedLocalEntry>,
    shared: HashMap<TypeId, ResolvedSharedEntry>,
}

impl Capabilities {
    /// Creates a new `Capabilities` from resolved entries.
    pub(crate) fn new(
        local: HashMap<TypeId, ResolvedLocalEntry>,
        shared: HashMap<TypeId, ResolvedSharedEntry>,
    ) -> Self {
        Capabilities { local, shared }
    }

    /// Creates an empty `Capabilities` (no bindings).
    #[must_use]
    pub fn empty() -> Self {
        Capabilities {
            local: HashMap::new(),
            shared: HashMap::new(),
        }
    }

    /// Resolve a **required** local capability.
    ///
    /// Returns `Rc<dyn C::Local>`. If the capability was registered by a
    /// shared-only extension, the `SharedAsLocal` adapter is returned
    /// transparently — the caller always gets a local trait object.
    ///
    /// # One-shot contract
    ///
    /// Each capability binding may be claimed **at most once per
    /// node** across all four accessors (`require_local`,
    /// `require_shared`, `optional_local`, `optional_shared`). Node
    /// factories are expected to call this once at construction,
    /// store the returned handle, and clone/share it within the node
    /// as needed. A second call for the same capability — including
    /// the implicit shared claim made by the `SharedAsLocal` fallback
    /// — returns [`Error::CapabilityAlreadyConsumed`].
    ///
    /// # Errors
    ///
    /// - [`Error::CapabilityNotBound`] if no extension is bound to this
    ///   capability for this node. Either add the binding to the
    ///   node's capability declaration or switch to
    ///   [`Self::optional_local`].
    /// - [`Error::CapabilityAlreadyConsumed`] if the capability was
    ///   already claimed on this node.
    ///
    /// # Panics
    ///
    /// Panics on a type-erasure downcast mismatch — this indicates a
    /// registry bug (the `#[capability]` proc macro guarantees the
    /// stored entry's concrete type matches `C::Local`).
    pub fn require_local<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Rc<C::Local>, Error> {
        let id = TypeId::of::<C>();
        let entry = self
            .local
            .get(&id)
            .ok_or_else(|| Error::CapabilityNotBound {
                capability: C::name().to_owned(),
                execution_model: "local",
            })?;
        if entry.claimed.replace(true) {
            return Err(Error::CapabilityAlreadyConsumed {
                capability: C::name().to_owned(),
            });
        }
        let rc_any = (entry.produce)();
        let trait_object = rc_any
            .downcast_ref::<Rc<C::Local>>()
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "BUG: capability '{}': local entry type mismatch in registry",
                    C::name(),
                )
            });
        entry.tracker_consumed.set(true);
        Ok(trait_object)
    }

    /// Resolve a **required** shared capability.
    ///
    /// Returns `Box<dyn C::Shared>` — the extension's shared
    /// implementation produced for this node.
    ///
    /// # One-shot contract
    ///
    /// See [`Self::require_local`].
    ///
    /// # Errors
    ///
    /// - [`Error::CapabilityNotBound`] if no extension is bound to this
    ///   capability for this node. Either add the binding to the
    ///   node's capability declaration or switch to
    ///   [`Self::optional_shared`].
    /// - [`Error::CapabilityAlreadyConsumed`] if the capability was
    ///   already claimed on this node.
    ///
    /// # Panics
    ///
    /// Panics on a type-erasure downcast mismatch — this indicates a
    /// registry bug (the `#[capability]` proc macro guarantees the
    /// stored entry's concrete type matches `C::Shared`).
    pub fn require_shared<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Box<C::Shared>, Error> {
        let id = TypeId::of::<C>();
        let entry = self
            .shared
            .get(&id)
            .ok_or_else(|| Error::CapabilityNotBound {
                capability: C::name().to_owned(),
                execution_model: "shared",
            })?;
        if entry.claimed.replace(true) {
            return Err(Error::CapabilityAlreadyConsumed {
                capability: C::name().to_owned(),
            });
        }
        let trait_object = (entry.produce)()
            .downcast::<Box<C::Shared>>()
            .map(|b| *b)
            .unwrap_or_else(|_| {
                panic!(
                    "BUG: capability '{}': shared entry type mismatch in registry",
                    C::name(),
                )
            });
        entry.tracker_consumed.set(true);
        Ok(trait_object)
    }

    /// Resolve an **optional** local capability.
    ///
    /// Returns `Ok(Some(_))` if bound, `Ok(None)` if the capability
    /// was not configured for this node.
    ///
    /// # One-shot contract
    ///
    /// See [`Self::require_local`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::CapabilityAlreadyConsumed`] if the capability
    /// was already claimed on this node.
    ///
    /// # Panics
    ///
    /// Panics on a type-erasure downcast mismatch (registry bug).
    pub fn optional_local<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Option<Rc<C::Local>>, Error> {
        let id = TypeId::of::<C>();
        if !self.local.contains_key(&id) {
            return Ok(None);
        }
        self.require_local::<C>().map(Some)
    }

    /// Resolve an **optional** shared capability.
    ///
    /// Returns `Ok(Some(_))` if bound, `Ok(None)` if the capability
    /// was not configured for this node.
    ///
    /// # One-shot contract
    ///
    /// See [`Self::require_local`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::CapabilityAlreadyConsumed`] if the capability
    /// was already claimed on this node.
    ///
    /// # Panics
    ///
    /// Panics on a type-erasure downcast mismatch (registry bug).
    pub fn optional_shared<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Option<Box<C::Shared>>, Error> {
        let id = TypeId::of::<C>();
        if !self.shared.contains_key(&id) {
            return Ok(None);
        }
        self.require_shared::<C>().map(Some)
    }
}

impl std::fmt::Debug for Capabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Entry values are type-erased; only the shape is printable.
        f.debug_struct("Capabilities")
            .field("local_bindings", &self.local.len())
            .field("shared_bindings", &self.shared.len())
            .finish()
    }
}
