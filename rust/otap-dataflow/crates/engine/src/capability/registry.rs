// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability registry and resolution.
//!
//! [`CapabilityRegistry`] collects type-erased capability trait objects from
//! extensions during the build phase. [`Capabilities`] is produced per-node
//! by `resolve_bindings()` and provides the consumer API
//! (`require_local`, `require_shared`, `optional_local`, `optional_shared`).
//!
//! TODO(extension-system): the pieces in this module are scaffolding —
//! exercised only by the unit tests below. The engine build phase will
//! populate the registry from extension factories, invoke
//! `resolve_bindings()` per node, and use the `ConsumedTracker`'s
//! `unconsumed_local` / `unconsumed_shared` methods to drop unused
//! extension variants.

use otap_df_config::ExtensionId;
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Error type alias for capability operations.
pub type Error = crate::error::Error;

// ── Shared-capability factory trait ──────────────────────────────────────────

/// Object-safe factory for the shared variant of a capability.
///
/// One concrete impl exists per `(capability, extension)` pair. The impl
/// holds the extension instance by value (typically a `Clone + Send +
/// 'static` type with `Arc`-wrapped shared state — see the
/// [extension system architecture doc][arch]) and:
///
/// - [`clone_box`](Self::clone_box) duplicates the factory itself so
///   each resolved per-node entry owns an independent `Box`. This is
///   the `Box + Clone` idiom: `dyn Trait` is not `Clone`, so we thread
///   cloning through an object-safe method.
/// - [`produce_any`](Self::produce_any) mints a `Box<dyn shared::Trait>`
///   for a consumer and type-erases it as `Box<dyn Any + Send>` for
///   downcast at the [`Capabilities`] boundary.
///
/// "Factory" here follows the DI-container convention: a thing that
/// hands you an instance on demand. The lifetime policy is an
/// implementation detail of [`produce_any`] — today every capability
/// clones a prototype (GoF Prototype pattern), tomorrow a capability
/// could construct a fresh instance instead.
///
/// The trait is `Send`-only (not `Send + Sync`): registries are never
/// shared across threads — they are cloned/owned per consumer — and
/// `Box<T>: Send` does not require `T: Sync`. This matches the extension
/// design doc's contract that shared extensions are `Clone + Send` with
/// `Arc`-wrapped state.
///
/// [arch]: ../../../docs/extension-system-architecture.md
#[doc(hidden)]
pub trait SharedCapabilityFactory: Send {
    /// Duplicate this factory. Each resolved per-node entry owns its
    /// own `Box<dyn SharedCapabilityFactory>` produced via this method.
    fn clone_box(&self) -> Box<dyn SharedCapabilityFactory>;
    /// Produce a fresh shared trait object for a consumer. The returned
    /// `Box<dyn Any + Send>` contains `Box<dyn shared::Trait>` — the
    /// `require_shared` code downcasts to recover the concrete trait.
    fn produce_any(&self) -> Box<dyn Any + Send>;
}

// ── Type-erased entries ──────────────────────────────────────────────────────

/// A type-erased local (!Send) capability entry.
///
/// Wraps a natively-registered `Rc<dyn local::Trait>`. All nodes bound to
/// the same `(capability, extension)` pair share this instance via
/// `Rc`-cloning. Consumers downcast via [`Capabilities::require_local`].
///
/// Extensions that only register a shared variant are handled directly
/// by [`resolve_bindings`] via the `SharedAsLocal` adapter — there is no
/// separate “fallback” local entry stored in the registry.
#[doc(hidden)]
pub struct LocalCapabilityEntry {
    /// The extension that provided this capability.
    pub(crate) extension_id: ExtensionId,
    /// Type-erased `Rc<dyn local::Trait>`.
    /// Stored as `Rc<dyn Any>` so the entry can be shared between the
    /// registry and per-node resolved entries. The inner value is an
    /// `Rc<dyn local::Trait>` — consumers downcast via
    /// `downcast_ref::<Rc<dyn Trait>>().clone()`.
    pub(crate) trait_object: Rc<dyn Any>,
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

/// Type-erased adapter function that creates a local entry from a shared
/// capability factory. Registered per-capability by the `#[capability]` proc macro
/// via `KNOWN_CAPABILITIES`.
///
/// The function receives the shared `factory` and returns an `Rc<dyn Any>`
/// containing `Rc<dyn local::Trait>` (via the generated `SharedAsLocal` adapter).
/// Returns `None` if the capability doesn't support shared→local fallback.
#[doc(hidden)]
pub type SharedAsLocalAdaptFn = fn(&dyn SharedCapabilityFactory) -> Option<Rc<dyn Any>>;

// ── CapabilityRegistry ───────────────────────────────────────────────────────

/// Collects type-erased capability registrations from extensions during
/// the pipeline build phase.
///
/// Keyed by `TypeId` of the zero-sized `ExtensionCapability` registration
/// struct (e.g., `TypeId::of::<BearerTokenProvider>()`).
///
/// After all extensions have registered, call `resolve_bindings()` to
/// produce per-node [`Capabilities`] structs.
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
    /// for this capability. Does not count shared entries that could be
    /// adapted via `SharedAsLocal`; a local binding is reachable when
    /// either `has_local` is true, or `has_shared` is true and the
    /// capability's [`ExtensionCapability::adapt_shared_to_local`] returns
    /// `Some` for the extension's factory.
    ///
    /// [`ExtensionCapability::adapt_shared_to_local`]: super::ExtensionCapability::adapt_shared_to_local
    #[must_use]
    pub fn has_local(&self, capability_id: &TypeId) -> bool {
        self.local
            .get(capability_id)
            .is_some_and(|m| !m.is_empty())
    }

    /// Returns `true` if any extension provides a shared entry for this capability.
    #[must_use]
    pub fn has_shared(&self, capability_id: &TypeId) -> bool {
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

// ── Resolved entries (per-node) ──────────────────────────────────────────────

/// A resolved local capability entry for a specific node.
pub(crate) struct ResolvedLocalEntry {
    /// Type-erased `Rc<dyn local::Trait>` — either an `Rc`-clone of the
    /// registry's native entry or a freshly-built `SharedAsLocal` adapter
    /// wrapping this node's own clone of the shared extension instance.
    pub(crate) trait_object: Rc<dyn Any>,
    /// Consumption flag for the underlying extension variant.
    ///
    /// - For native local bindings: the cell is registered in
    ///   [`ConsumedTracker::local`] for the `(capability, extension)` pair.
    /// - For `SharedAsLocal` bindings: the cell is registered in
    ///   [`ConsumedTracker::shared`] for the `(capability, extension)`
    ///   pair — consuming the adapter is considered a use of the shared
    ///   variant, because that's the only variant the extension actually
    ///   provided. No phantom local entry is recorded.
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

// ── Capabilities (consumer API) ──────────────────────────────────────────────

/// Per-node capability bindings resolved from the [`CapabilityRegistry`].
///
/// Passed to node factories as `&Capabilities`. Provides type-safe
/// access to extension capabilities via the [`ExtensionCapability`]
/// sealed trait.
///
/// [`ExtensionCapability`]: super::ExtensionCapability
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
    /// Returns an error if no extension provides this capability in
    /// local (or shared-with-fallback) mode.
    pub fn require_local<C: super::ExtensionCapability>(&self) -> Result<Rc<C::Local>, Error> {
        let id = TypeId::of::<C>();
        let entry = self.local.get(&id).ok_or_else(|| Error::InternalError {
            message: format!(
                "required local capability '{}' not bound for this node",
                C::name(),
            ),
        })?;
        let trait_object = entry
            .trait_object
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
    /// Returns an error if no extension provides this capability in
    /// shared mode.
    pub fn require_shared<C: super::ExtensionCapability>(&self) -> Result<Box<C::Shared>, Error> {
        let id = TypeId::of::<C>();
        let entry = self.shared.get(&id).ok_or_else(|| Error::InternalError {
            message: format!(
                "required shared capability '{}' not bound for this node",
                C::name(),
            ),
        })?;
        let boxed_any = entry.factory.produce_any();
        let trait_object = *boxed_any
            .downcast::<Box<C::Shared>>()
            .map_err(|_| Error::InternalError {
                message: format!(
                    "capability '{}': shared entry type mismatch (internal error)",
                    C::name(),
                ),
            })?;
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
    pub fn optional_local<C: super::ExtensionCapability>(&self) -> Option<Rc<C::Local>> {
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
    pub fn optional_shared<C: super::ExtensionCapability>(&self) -> Option<Box<C::Shared>> {
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

// ── ConsumedTracker ──────────────────────────────────────────────────────────

/// Tracks which capability variants were consumed by node factories.
///
/// Created alongside per-node [`Capabilities`] during `resolve_bindings()`.
/// After all node factories have run, the engine inspects this tracker to
/// determine which extension variants are unused and can be dropped.
///
/// Keyed by `(capability TypeId, extension ID)` so multiple providers of
/// the same capability are tracked independently. The `Rc<Cell<bool>>`
/// for a given key is shared across all nodes that bind to that
/// provider — once any of them consumes the capability the cell is set.
pub(crate) struct ConsumedTracker {
    local: HashMap<(TypeId, ExtensionId), ConsumedEntry>,
    shared: HashMap<(TypeId, ExtensionId), ConsumedEntry>,
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
    /// `(capability, extension)` pair, creating a fresh entry if none
    /// exists yet. Any returned clone shared with a resolved entry will
    /// flip the same cell when consumed.
    pub(crate) fn track_local(
        &mut self,
        capability_id: TypeId,
        name: &'static str,
        extension_id: ExtensionId,
        consumed: Rc<Cell<bool>>,
    ) -> Rc<Cell<bool>> {
        let entry = self
            .local
            .entry((capability_id, extension_id.clone()))
            .or_insert_with(|| ConsumedEntry {
                name,
                extension_id,
                consumed,
            });
        Rc::clone(&entry.consumed)
    }

    /// Returns the `Rc<Cell<bool>>` tracking shared consumption for this
    /// `(capability, extension)` pair, creating a fresh entry if none
    /// exists yet.
    pub(crate) fn track_shared(
        &mut self,
        capability_id: TypeId,
        name: &'static str,
        extension_id: ExtensionId,
        consumed: Rc<Cell<bool>>,
    ) -> Rc<Cell<bool>> {
        let entry = self
            .shared
            .entry((capability_id, extension_id.clone()))
            .or_insert_with(|| ConsumedEntry {
                name,
                extension_id,
                consumed,
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

// ── resolve_bindings ─────────────────────────────────────────────────────────

/// Resolves a node's capability bindings against the registry.
///
/// For each `(capability_name, extension_name)` in the node's config:
///
/// 1. **Extension exists** — `extension_name` is in `known_extensions`.
/// 2. **Known capability** — `capability_name` is in `KNOWN_CAPABILITIES`.
/// 3. **Capability provided** — The registry has an entry for this capability.
/// 4. **Specific extension provides it** — The registry entry was registered
///    by `extension_name`.
///
/// On success, returns a [`Capabilities`] for the node and updates the
/// `tracker` with consumption flags.
///
/// # Errors
///
/// Returns an error on the first validation failure with a descriptive message.
pub(crate) fn resolve_bindings(
    bindings: &HashMap<otap_df_config::CapabilityId, ExtensionId>,
    registry: &CapabilityRegistry,
    known_extensions: &HashSet<ExtensionId>,
    tracker: &mut ConsumedTracker,
) -> Result<Capabilities, Error> {
    // Build lookup from capability name → KnownCapability
    let known_caps: HashMap<&str, &super::KnownCapability> = super::KNOWN_CAPABILITIES
        .iter()
        .map(|kc| (kc.name, kc))
        .collect();

    let mut local_entries: HashMap<TypeId, ResolvedLocalEntry> = HashMap::new();
    let mut shared_entries: HashMap<TypeId, ResolvedSharedEntry> = HashMap::new();

    for (cap_name, ext_name) in bindings {
        let cap_name_str: &str = cap_name.as_ref();
        let ext_name_str: &str = ext_name.as_ref();

        // Step 1: Extension exists
        if !known_extensions.contains(ext_name_str) {
            return Err(Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "capability binding '{cap_name_str}': no extension named '{ext_name_str}' exists",
                    ),
                },
            )));
        }

        // Step 2: Known capability type
        let known_cap = known_caps.get(cap_name_str).ok_or_else(|| {
            let known_names: Vec<&str> = known_caps.keys().copied().collect();
            Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "unknown capability '{cap_name_str}'. Known capabilities: {known_names:?}",
                    ),
                },
            ))
        })?;

        let cap_type_id = (known_cap.type_id)();

        // Step 3: At least one extension must provide this capability
        let has_local = registry.has_local(&cap_type_id);
        let has_shared = registry.has_shared(&cap_type_id);
        if !has_local && !has_shared {
            return Err(Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "capability '{cap_name_str}': no loaded extension provides it",
                    ),
                },
            )));
        }

        // Step 4: The specific named extension must provide it.
        let local_entry = registry.get_local(&cap_type_id, ext_name_str);
        let shared_entry = registry.get_shared(&cap_type_id, ext_name_str);

        if local_entry.is_none() && shared_entry.is_none() {
            return Err(Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!(
                        "capability '{cap_name_str}': extension '{ext_name_str}' does not provide it",
                    ),
                },
            )));
        }

        // Resolve local entry: prefer a native local registration; else,
        // if the extension registered a shared entry, invoke the
        // capability's `SharedAsLocal` adapter to build a fresh local
        // wrapper around this node's own clone of the shared instance.
        let native_local = local_entry;

        if let Some(local_entry) = native_local {
            let consumed = tracker.track_local(
                cap_type_id,
                known_cap.name,
                local_entry.extension_id.clone(),
                Rc::new(Cell::new(false)),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    trait_object: Rc::clone(&local_entry.trait_object),
                    consumed,
                },
            );
        } else if let Some(shared_entry) = shared_entry {
            // SharedAsLocal fallback: invoke the capability's adapter with
            // this node's own clone of the shared instance. Each node gets
            // a fresh adapter. The adapter fn is a required method on
            // `ExtensionCapability`, so it's always present; it may
            // legitimately return `None` to signal that this capability
            // does not support shared→local adaptation.
            let adapt_fn = known_cap.adapt_shared_to_local;
            let trait_object = adapt_fn(&*shared_entry.factory).ok_or_else(|| {
                Error::InternalError {
                    message: format!(
                        "capability '{cap_name_str}': SharedAsLocal adapter for extension '{ext_name_str}' returned None",
                    ),
                }
            })?;

            // The extension only provided a shared variant — track the
            // adapter consumer under the shared bucket. The follow-up
            // `// Resolve shared entry` block below also calls
            // `track_shared` for this `(cap, ext)` pair; `track_shared`
            // is get-or-insert, so both users share the same
            // `Rc<Cell<bool>>` and either one flipping marks the shared
            // variant consumed. No phantom entry is created in
            // `tracker.local`.
            let consumed = tracker.track_shared(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
                Rc::new(Cell::new(false)),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    trait_object,
                    consumed,
                },
            );
        }

        // Resolve shared entry
        if let Some(shared_entry) = shared_entry {
            let consumed = tracker.track_shared(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
                Rc::new(Cell::new(false)),
            );

            let _ = shared_entries.insert(
                cap_type_id,
                ResolvedSharedEntry {
                    factory: shared_entry.factory.clone_box(),
                    consumed,
                },
            );
        }
    }

    Ok(Capabilities::new(local_entries, shared_entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    // ── Hand-written test capability ─────────────────────────────────────
    //
    // This manually implements `ExtensionCapability` and registers a
    // `KnownCapability` entry — the same interfaces that the `#[capability]`
    // proc macro targets. We can't use the macro here because proc macros
    // cannot be invoked from within the crate that hosts their generated
    // target paths (`crate::capability::*`).
    //
    // These tests verify the **registry infrastructure** (register, resolve,
    // consume, SharedAsLocal adapter). Update the hand-written impls below if `ExtensionCapability`,
    // `KnownCapability`, or the registry API changes.

    /// A minimal test capability trait (local version).
    trait TestCapLocal {
        fn value(&self) -> &str;
    }

    /// A minimal test capability trait (shared version).
    trait TestCapShared: Send {
        fn value(&self) -> &str;
    }

    /// Zero-sized registration struct for `TestCap`.
    struct TestCap;

    impl super::super::private::Sealed for TestCap {}

    impl super::super::ExtensionCapability for TestCap {
        const NAME: &'static str = "test_cap";
        type Local = dyn TestCapLocal;
        type Shared = dyn TestCapShared;

        fn adapt_shared_to_local(
            factory: &dyn SharedCapabilityFactory,
        ) -> Option<Rc<dyn Any>> {
            let boxed_any = factory.produce_any();
            let boxed_shared: Box<dyn TestCapShared> = *boxed_any
                .downcast::<Box<dyn TestCapShared>>()
                .expect("BUG: factory for test_cap must produce Box<Box<dyn TestCapShared>>");
            struct Adapter(Box<dyn TestCapShared>);
            impl TestCapLocal for Adapter {
                fn value(&self) -> &str {
                    self.0.value()
                }
            }
            let rc_local: Rc<dyn TestCapLocal> = Rc::new(Adapter(boxed_shared));
            Some(Rc::new(rc_local))
        }
    }

    /// Register TestCap in KNOWN_CAPABILITIES for resolve_bindings tests.
    #[allow(unsafe_code)]
    #[linkme::distributed_slice(super::super::KNOWN_CAPABILITIES)]
    #[linkme(crate = linkme)]
    static _TEST_CAP: super::super::KnownCapability = super::super::KnownCapability {
        name: "test_cap",
        description: "Test capability for unit tests",
        type_id: || TypeId::of::<TestCap>(),
        adapt_shared_to_local:
            <TestCap as super::super::ExtensionCapability>::adapt_shared_to_local,
    };

    // ── Test implementations ─────────────────────────────────────────────

    #[derive(Clone)]
    struct SharedImpl(&'static str);
    impl TestCapShared for SharedImpl {
        fn value(&self) -> &str {
            self.0
        }
    }

    struct LocalImpl(&'static str);
    impl TestCapLocal for LocalImpl {
        fn value(&self) -> &str {
            self.0
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    // Reusable test factory: per-`(cap, ext)` producer that clones a
    // `&'static str` value and produces fresh `Box<dyn TestCapShared>`
    // instances. Mirrors how real extension-registration code will shape
    // factories (concrete Clone + Send type carrying the extension state).
    struct TestFactory {
        val: &'static str,
    }
    impl SharedCapabilityFactory for TestFactory {
        fn clone_box(&self) -> Box<dyn SharedCapabilityFactory> {
            Box::new(TestFactory { val: self.val })
        }
        fn produce_any(&self) -> Box<dyn Any + Send> {
            let b: Box<dyn TestCapShared> = Box::new(SharedImpl(self.val));
            Box::new(b) as Box<dyn Any + Send>
        }
    }

    fn register_shared(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
        let ext_id: ExtensionId = ext_id.into();
        registry
            .register_shared(
                TypeId::of::<TestCap>(),
                SharedCapabilityEntry {
                    extension_id: ext_id,
                    factory: Box::new(TestFactory { val }),
                },
            )
            .unwrap();
    }

    fn register_local(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
        let ext_id: ExtensionId = ext_id.into();
        let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl(val));
        registry
            .register_local(
                TypeId::of::<TestCap>(),
                LocalCapabilityEntry {
                    extension_id: ext_id,
                    trait_object: Rc::new(rc_local),
                },
            )
            .unwrap();
    }

    fn bindings(
        cap: &'static str,
        ext: &'static str,
    ) -> HashMap<otap_df_config::CapabilityId, ExtensionId> {
        let mut m = HashMap::new();
        let _ = m.insert(cap.into(), ext.into());
        m
    }

    fn known_exts(names: &[&'static str]) -> HashSet<ExtensionId> {
        names.iter().map(|n| (*n).into()).collect()
    }

    // ── Tests ────────────────────────────────────────────────────────────

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "hello");
        assert!(reg.get_shared(&TypeId::of::<TestCap>(), "ext-a").is_some());
        assert!(reg.get_shared(&TypeId::of::<TestCap>(), "ext-b").is_none());
        assert!(reg.get_local(&TypeId::of::<TestCap>(), "ext-a").is_none());
    }

    #[test]
    fn test_resolve_bindings_shared_only() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "shared-val");

        let mut tracker = ConsumedTracker::new();
        let caps = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        // require_shared works
        let shared = caps.require_shared::<TestCap>().unwrap();
        assert_eq!(shared.value(), "shared-val");

        // require_local works via SharedAsLocal adapter
        let local = caps.require_local::<TestCap>().unwrap();
        assert_eq!(local.value(), "shared-val");
    }

    #[test]
    fn test_resolve_bindings_local_only() {
        let mut reg = CapabilityRegistry::new();
        register_local(&mut reg, "ext-a", "local-val");

        let mut tracker = ConsumedTracker::new();
        let caps = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        let local = caps.require_local::<TestCap>().unwrap();
        assert_eq!(local.value(), "local-val");

        // shared is not available
        assert!(caps.require_shared::<TestCap>().is_err());
    }

    #[test]
    fn test_resolve_bindings_step1_unknown_extension() {
        let reg = CapabilityRegistry::new();
        let mut tracker = ConsumedTracker::new();
        let result = resolve_bindings(
            &bindings("test_cap", "nonexistent"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        );
        assert!(result.is_err());
        let msg = format!("{}", result.err().unwrap());
        assert!(msg.contains("nonexistent"), "error: {msg}");
    }

    #[test]
    fn test_resolve_bindings_step2_unknown_capability() {
        let reg = CapabilityRegistry::new();
        let mut tracker = ConsumedTracker::new();
        let result = resolve_bindings(
            &bindings("totally_unknown_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        );
        assert!(result.is_err());
        let msg = format!("{}", result.err().unwrap());
        assert!(msg.contains("unknown capability"), "error: {msg}");
    }

    #[test]
    fn test_resolve_bindings_step3_not_provided() {
        // Extension exists, capability is known, but no extension provides it.
        let reg = CapabilityRegistry::new();
        let mut tracker = ConsumedTracker::new();
        let result = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        );
        assert!(result.is_err());
        let msg = format!("{}", result.err().unwrap());
        assert!(msg.contains("no loaded extension provides"), "error: {msg}");
    }

    #[test]
    fn test_resolve_bindings_step4_wrong_extension() {
        // ext-b provides test_cap, but binding says ext-a.
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-b", "val");

        let mut tracker = ConsumedTracker::new();
        let result = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a", "ext-b"]),
            &mut tracker,
        );
        assert!(result.is_err());
        let msg = format!("{}", result.err().unwrap());
        assert!(msg.contains("does not provide"), "error: {msg}");
    }

    #[test]
    fn test_consumed_tracking_shared() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "val");

        let mut tracker = ConsumedTracker::new();
        let caps = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        // Not consumed yet
        assert_eq!(tracker.unconsumed_shared().len(), 1);

        // Consume
        let _ = caps.require_shared::<TestCap>().unwrap();
        assert!(tracker.unconsumed_shared().is_empty());
    }

    #[test]
    fn test_consumed_tracking_local_marks_shared_via_adapter() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "val");

        let mut tracker = ConsumedTracker::new();
        let caps = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        // Consume via local adapter
        let _ = caps.require_local::<TestCap>().unwrap();

        // The extension only registered a shared variant, so there is no
        // native local tracker entry for this (cap, ext) pair — the
        // SharedAsLocal adapter's consumption counts as shared use.
        assert!(tracker.unconsumed_local().is_empty());
        assert!(
            tracker.unconsumed_shared().is_empty(),
            "consuming SharedAsLocal adapter must mark shared variant consumed",
        );
    }

    #[test]
    fn test_unconsumed_tracking() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "val");

        let mut tracker = ConsumedTracker::new();
        let _caps = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        // Not consumed — should appear in unconsumed
        let unconsumed = tracker.unconsumed_shared();
        assert_eq!(unconsumed.len(), 1);
        assert_eq!(unconsumed[0].0.as_ref(), "ext-a");
        assert_eq!(unconsumed[0].1, "test_cap");
    }

    #[test]
    fn test_optional_returns_none_when_not_bound() {
        let caps = Capabilities::empty();
        assert!(caps.optional_local::<TestCap>().is_none());
        assert!(caps.optional_shared::<TestCap>().is_none());
    }

    #[test]
    fn test_extension_capabilities_none() {
        let ec = super::super::ExtensionCapabilities::none();
        assert!(ec.shared.is_empty());
        assert!(ec.local.is_empty());
    }

    #[test]
    fn test_extension_capabilities_shared_only() {
        let ec = super::super::ExtensionCapabilities {
            shared: &["bearer_token_provider"],
            local: &[],
        };
        assert_eq!(ec.shared, &["bearer_token_provider"]);
        assert!(ec.local.is_empty());
    }

    #[test]
    fn test_known_capabilities_contains_test_cap() {
        let found = super::super::KNOWN_CAPABILITIES
            .iter()
            .any(|kc| kc.name == "test_cap");
        assert!(found, "test_cap should be in KNOWN_CAPABILITIES");
    }

    #[test]
    fn test_multiple_providers_same_capability() {
        // Two extensions both provide test_cap with different values.
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "value-a");
        register_shared(&mut reg, "ext-b", "value-b");

        // Both are accessible by (cap, ext) key.
        assert!(reg.get_shared(&TypeId::of::<TestCap>(), "ext-a").is_some());
        assert!(reg.get_shared(&TypeId::of::<TestCap>(), "ext-b").is_some());

        // Node bound to ext-a gets ext-a's value.
        let mut tracker = ConsumedTracker::new();
        let caps_a = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a", "ext-b"]),
            &mut tracker,
        )
        .unwrap();
        let shared_a = caps_a.require_shared::<TestCap>().unwrap();
        assert_eq!(shared_a.value(), "value-a");

        // Node bound to ext-b gets ext-b's value.
        let mut tracker = ConsumedTracker::new();
        let caps_b = resolve_bindings(
            &bindings("test_cap", "ext-b"),
            &reg,
            &known_exts(&["ext-a", "ext-b"]),
            &mut tracker,
        )
        .unwrap();
        let shared_b = caps_b.require_shared::<TestCap>().unwrap();
        assert_eq!(shared_b.value(), "value-b");
    }

    /// Regression: the `ConsumedTracker` must not lose track of a node's
    /// consumption when a second node resolves the same capability.
    ///
    /// Scenario: two nodes bind `test_cap` to `ext-a` (local). Node A
    /// consumes it, node B does not. The tracker should still report the
    /// capability as consumed (because at least one node consumed it) —
    /// otherwise the engine will drop an extension variant that's actually
    /// in use.
    #[test]
    fn test_consumed_tracking_persists_across_nodes_local() {
        let mut reg = CapabilityRegistry::new();
        register_local(&mut reg, "ext-a", "val");

        let mut tracker = ConsumedTracker::new();

        // Node A resolves + consumes (local).
        let caps_a = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();
        let _ = caps_a.require_local::<TestCap>().unwrap();

        // Node B resolves but does NOT consume.
        let _caps_b = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        // At least one node consumed it — should not appear in unconsumed.
        assert!(
            tracker.unconsumed_local().is_empty(),
            "unconsumed_local should be empty but got {:?}",
            tracker.unconsumed_local()
        );
    }

    /// Same regression for shared variant (already works via get-or-insert,
    /// but worth locking in behavior).
    #[test]
    fn test_consumed_tracking_persists_across_nodes_shared() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "val");

        let mut tracker = ConsumedTracker::new();

        let caps_a = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();
        let _ = caps_a.require_shared::<TestCap>().unwrap();

        let _caps_b = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        assert!(tracker.unconsumed_shared().is_empty());
    }

    /// Regression for Option C: each node resolving a `SharedAsLocal`
    /// binding receives its own clone of the shared extension instance.
    /// This preserves per-caller-fresh semantics that a shared impl may
    /// rely on (e.g. per-caller mutable state) — the adapter is not
    /// pre-built once and shared across nodes.
    #[test]
    fn test_shared_as_local_builds_fresh_adapter_per_node() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CLONE_COUNT: AtomicUsize = AtomicUsize::new(0);

        // Register a shared impl whose factory bumps a counter every
        // time the extension is cloned.
        CLONE_COUNT.store(0, Ordering::SeqCst);
        let mut reg = CapabilityRegistry::new();
        let ext_id: ExtensionId = "ext-a".into();

        struct CountingFactory;
        impl SharedCapabilityFactory for CountingFactory {
            fn clone_box(&self) -> Box<dyn SharedCapabilityFactory> {
                Box::new(CountingFactory)
            }
            fn produce_any(&self) -> Box<dyn Any + Send> {
                let _ = CLONE_COUNT.fetch_add(1, Ordering::SeqCst);
                let b: Box<dyn TestCapShared> = Box::new(SharedImpl("val"));
                Box::new(b) as Box<dyn Any + Send>
            }
        }

        reg.register_shared(
            TypeId::of::<TestCap>(),
            SharedCapabilityEntry {
                extension_id: ext_id,
                factory: Box::new(CountingFactory),
            },
        )
        .unwrap();

        // The adapter must not run at registration time — work is deferred
        // to resolve_bindings so each node gets a fresh shared clone.
        assert_eq!(CLONE_COUNT.load(Ordering::SeqCst), 0);

        // Two nodes bind the capability; each should get its own clone.
        let mut tracker = ConsumedTracker::new();
        let _caps_a = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();
        let _caps_b = resolve_bindings(
            &bindings("test_cap", "ext-a"),
            &reg,
            &known_exts(&["ext-a"]),
            &mut tracker,
        )
        .unwrap();

        assert_eq!(CLONE_COUNT.load(Ordering::SeqCst), 2);
    }

    /// Duplicate registrations indicate a programmer bug — the registry
    /// must reject them loudly rather than silently overwriting.
    #[test]
    fn test_register_local_rejects_duplicate() {
        let mut reg = CapabilityRegistry::new();
        register_local(&mut reg, "ext-a", "v1");

        let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl("v2"));
        let err = reg
            .register_local(
                TypeId::of::<TestCap>(),
                LocalCapabilityEntry {
                    extension_id: "ext-a".into(),
                    trait_object: Rc::new(rc_local),
                },
            )
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("duplicate"), "error: {msg}");
        assert!(msg.contains("ext-a"), "error: {msg}");
    }

    #[test]
    fn test_register_shared_rejects_duplicate() {
        let mut reg = CapabilityRegistry::new();
        register_shared(&mut reg, "ext-a", "v1");

        let err = reg
            .register_shared(
                TypeId::of::<TestCap>(),
                SharedCapabilityEntry {
                    extension_id: "ext-a".into(),
                    factory: Box::new(TestFactory { val: "v2" }),
                },
            )
            .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("duplicate"), "error: {msg}");
        assert!(msg.contains("ext-a"), "error: {msg}");
    }
}
