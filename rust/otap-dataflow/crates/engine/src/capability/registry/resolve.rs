// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`resolve_bindings`] — the per-node resolution pass that validates
//! a node's capability bindings against the registry and produces a
//! [`Capabilities`] for consumption.

use super::{
    Capabilities, CapabilityRegistry, ConsumedTracker, Error, LocalProduce, ResolvedLocalEntry,
    ResolvedSharedEntry,
};
use otap_df_config::ExtensionId;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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
/// Returns an error on a validation failure with a descriptive message.
/// Bindings are iterated in sorted order by capability name, so when
/// multiple bindings are invalid the specific one reported is stable
/// across runs.
pub(crate) fn resolve_bindings(
    bindings: &HashMap<otap_df_config::CapabilityId, ExtensionId>,
    registry: &CapabilityRegistry,
    known_extensions: &HashSet<ExtensionId>,
    tracker: &mut ConsumedTracker,
) -> Result<Capabilities, Error> {
    // Build lookup from capability name → KnownCapability
    let known_caps: HashMap<&str, &crate::capability::KnownCapability> =
        crate::capability::KNOWN_CAPABILITIES
            .iter()
            .map(|kc| (kc.name, kc))
            .collect();

    let mut local_entries: HashMap<TypeId, ResolvedLocalEntry> = HashMap::new();
    let mut shared_entries: HashMap<TypeId, ResolvedSharedEntry> = HashMap::new();

    // Iterate in sorted order by capability name for deterministic
    // error messages across runs.
    let mut sorted_bindings: Vec<(&otap_df_config::CapabilityId, &ExtensionId)> =
        bindings.iter().collect();
    sorted_bindings.sort_unstable_by(|(a, _), (b, _)| a.as_ref().cmp(b.as_ref()));

    for (cap_name, ext_name) in sorted_bindings {
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
            let mut known_names: Vec<&str> = known_caps.keys().copied().collect();
            known_names.sort_unstable();
            Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "unknown capability '{cap_name_str}'. Known capabilities: {known_names:?}",
                ),
            }))
        })?;

        let cap_type_id = (known_cap.type_id)();

        // Step 3: At least one extension must provide this capability
        let has_native_local = registry.has_native_local(&cap_type_id);
        let has_shared = registry.has_shared(&cap_type_id);
        if !has_native_local && !has_shared {
            return Err(Error::ConfigError(Box::new(
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!("capability '{cap_name_str}': no loaded extension provides it",),
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
            let tracker_consumed = tracker.ensure_local_consumer_slot(
                cap_type_id,
                known_cap.name,
                local_entry.extension_id.clone(),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    produce: local_entry.produce.clone_box(),
                    claimed: std::cell::Cell::new(false),
                    tracker_consumed,
                },
            );
        } else if let Some(shared_entry) = shared_entry {
            // SharedAsLocal fallback: defer the shared mint to the
            // (single, one-shot) `require_local` call. The closure
            // captures a refcounted handle to the shared produce
            // closure plus the capability's `adapt_as_local` fn
            // pointer, then on invocation produces a fresh shared
            // instance and routes it through the adapter to yield a
            // type-erased `Rc<dyn Any>` wrapping the local trait
            // object.
            let adapt = shared_entry.adapt_as_local;
            let produce_shared: Rc<Box<dyn super::entry::SharedProduce>> =
                Rc::new(shared_entry.produce.clone_box());
            let local_produce: Box<dyn LocalProduce> = Box::new(move || adapt((*produce_shared)()));

            // The extension only provided a shared variant — record
            // the adapter consumer's cross-node consumption against
            // the shared bucket. The `// Resolve shared entry` block
            // below also calls `ensure_shared_consumer_slot` for
            // this `(cap, ext)` pair; that method is get-or-insert,
            // so both users share the same `Rc<Cell<bool>>` for
            // tracker accounting. Per-node one-shot enforcement is
            // tracked separately in each entry's `claimed` cell, so
            // a node *can* claim both the local-via-shared and
            // native-shared sides of a fallback binding.
            let tracker_consumed = tracker.ensure_shared_consumer_slot(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    produce: local_produce,
                    claimed: std::cell::Cell::new(false),
                    tracker_consumed,
                },
            );
        }

        // Resolve shared entry. If the SharedAsLocal fallback above
        // already called `ensure_shared_consumer_slot` for this
        // `(cap, ext)` pair, this call is a no-op lookup that returns
        // the same cell — get-or-insert idempotency keeps the two
        // users pointing at one slot.
        if let Some(shared_entry) = shared_entry {
            let tracker_consumed = tracker.ensure_shared_consumer_slot(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
            );

            let _ = shared_entries.insert(
                cap_type_id,
                ResolvedSharedEntry {
                    produce: shared_entry.produce.clone_box(),
                    claimed: std::cell::Cell::new(false),
                    tracker_consumed,
                },
            );
        }
    }

    Ok(Capabilities::new(local_entries, shared_entries))
}
