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
    /// Each resolved entry is one-shot per node: a `require_local`
    /// claim consumes the local entry, and a `require_shared` claim
    /// consumes the shared entry. Node factories are expected to
    /// call each accessor at most once at construction, store the
    /// returned handle, and clone/share it within the node as needed.
    ///
    /// The contract is **per-binding**, not per-execution-model: a
    /// node claims a binding at most once, regardless of which
    /// accessor (`require_local`, `require_shared`, `optional_local`,
    /// `optional_shared`) is used. Claiming one execution model
    /// invalidates the other on the same node, so a subsequent call
    /// to the alternative accessor returns
    /// [`Error::CapabilityAlreadyConsumed`].
    ///
    /// This holds uniformly across both binding shapes:
    /// - **SharedAsLocal fallback** (extension registered only a
    ///   shared variant): there is one underlying produce closure,
    ///   so the shared entry's `Cell::take()` is the single guard.
    /// - **Native dual** (extension registered both native local and
    ///   native shared variants): the two resolved entries are
    ///   distinct, but a successful claim on either side also takes
    ///   (and drops, unrun) the alternative entry's produce closure
    ///   on this node, so the per-binding contract holds without an
    ///   auxiliary flag.
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

        // Native local path. `Cell::take()` is the one-shot guard.
        if let Some(entry) = self.local.get(&id) {
            let produce = entry
                .produce
                .take()
                .ok_or_else(|| Error::CapabilityAlreadyConsumed {
                    capability: C::name().to_owned(),
                })?;
            let rc_any = produce();
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
            // Per-binding one-shot: invalidate the native-shared
            // alternative on this node so a subsequent
            // `require_shared`/`optional_shared` returns
            // `CapabilityAlreadyConsumed`. In the SharedAsLocal
            // fallback there is no separate native local entry, so
            // this branch does not run for that path — the shared
            // entry's `Cell::take()` below is the single guard.
            if let Some(shared_entry) = self.shared.get(&id) {
                let _ = shared_entry.produce.take();
            }
            return Ok(trait_object);
        }

        // SharedAsLocal fallback. The same `Cell::take()` on the
        // shared entry is the binding's one-shot guard, so claiming
        // the local-via-fallback accessor here naturally consumes
        // the native shared accessor too — a subsequent
        // `require_shared` returns [`Error::CapabilityAlreadyConsumed`].
        let entry = self
            .shared
            .get(&id)
            .ok_or_else(|| Error::CapabilityNotBound {
                capability: C::name().to_owned(),
                execution_model: "local",
            })?;
        let produce = entry
            .produce
            .take()
            .ok_or_else(|| Error::CapabilityAlreadyConsumed {
                capability: C::name().to_owned(),
            })?;
        let rc_any = (entry.adapt_as_local)(produce());
        let trait_object = rc_any
            .downcast_ref::<Rc<C::Local>>()
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "BUG: capability '{}': SharedAsLocal adapter type mismatch in registry",
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
        let produce = entry
            .produce
            .take()
            .ok_or_else(|| Error::CapabilityAlreadyConsumed {
                capability: C::name().to_owned(),
            })?;
        let trait_object = produce()
            .downcast::<Box<C::Shared>>()
            .map(|b| *b)
            .unwrap_or_else(|_| {
                panic!(
                    "BUG: capability '{}': shared entry type mismatch in registry",
                    C::name(),
                )
            });
        entry.tracker_consumed.set(true);
        // Per-binding one-shot: invalidate the native-local
        // alternative on this node so a subsequent
        // `require_local`/`optional_local` returns
        // `CapabilityAlreadyConsumed`. In the SharedAsLocal fallback
        // (no native local entry) this is a no-op — the shared
        // entry's `Cell::take()` above already serves both sides.
        if let Some(local_entry) = self.local.get(&id) {
            let _ = local_entry.produce.take();
        }
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
        // Available either as a native local entry or as a
        // SharedAsLocal fallback through the shared entry.
        if !self.local.contains_key(&id) && !self.shared.contains_key(&id) {
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
