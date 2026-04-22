// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`Capabilities`] — the per-node consumer API for resolved capability
//! bindings, with `require_*` and `optional_*` accessors.

use super::{Error, ResolvedLocalEntry, ResolvedSharedEntry, downcast_produced};
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
    /// # Errors
    ///
    /// - [`Error::ConfigError`] if no extension is bound to this
    ///   capability for this node. The user must add a binding to the
    ///   node's config.
    /// - [`Error::InternalError`] on a type-erasure downcast mismatch
    ///   (indicates a registry bug).
    pub fn require_local<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Rc<C::Local>, Error> {
        let id = TypeId::of::<C>();
        let entry = self.local.get(&id).ok_or_else(|| {
            Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "required local capability '{}' not bound for this node",
                        C::name(),
                    ),
                },
            ))
        })?;
        let rc_any = entry.factory.produce_any();
        let trait_object = rc_any
            .downcast_ref::<Rc<C::Local>>()
            .cloned()
            .ok_or_else(|| Error::InternalError {
                message: format!(
                    "capability '{}': local entry type mismatch (internal error)",
                    C::name(),
                ),
            })?;
        entry.consumed.set(true);
        Ok(trait_object)
    }

    /// Resolve a **required** shared capability.
    ///
    /// Returns `Box<dyn C::Shared>` — a fresh clone of the extension's
    /// shared implementation.
    ///
    /// # Errors
    ///
    /// - [`Error::ConfigError`] if no extension is bound to this
    ///   capability for this node. The user must add a binding to the
    ///   node's config.
    /// - [`Error::InternalError`] on a type-erasure downcast mismatch
    ///   (indicates a registry bug).
    pub fn require_shared<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Result<Box<C::Shared>, Error> {
        let id = TypeId::of::<C>();
        let entry = self.shared.get(&id).ok_or_else(|| {
            Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "required shared capability '{}' not bound for this node",
                        C::name(),
                    ),
                },
            ))
        })?;
        let trait_object = downcast_produced::<C>(&*entry.factory)?;
        entry.consumed.set(true);
        Ok(trait_object)
    }

    /// Resolve an **optional** local capability.
    ///
    /// Returns `Some(Rc<dyn C::Local>)` if bound, `None` if the capability
    /// was not configured for this node.
    ///
    /// # Panics
    ///
    /// Panics on type mismatch (internal bug — the stored entry doesn't
    /// match the expected capability type).
    #[must_use]
    pub fn optional_local<C: crate::capability::ExtensionCapability>(&self) -> Option<Rc<C::Local>> {
        let id = TypeId::of::<C>();
        if !self.local.contains_key(&id) {
            return None;
        }
        Some(
            self.require_local::<C>().expect(
                "BUG: capability entry exists but downcast failed — type mismatch in registry",
            ),
        )
    }

    /// Resolve an **optional** shared capability.
    ///
    /// Returns `Some(Box<dyn C::Shared>)` if bound, `None` if the capability
    /// was not configured for this node.
    ///
    /// # Panics
    ///
    /// Panics on type mismatch (internal bug — the stored entry doesn't
    /// match the expected capability type).
    #[must_use]
    pub fn optional_shared<C: crate::capability::ExtensionCapability>(
        &self,
    ) -> Option<Box<C::Shared>> {
        let id = TypeId::of::<C>();
        if !self.shared.contains_key(&id) {
            return None;
        }
        Some(
            self.require_shared::<C>().expect(
                "BUG: capability entry exists but downcast failed — type mismatch in registry",
            ),
        )
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
