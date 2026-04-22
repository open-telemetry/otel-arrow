// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! [`resolve_bindings`] — the per-node resolution pass that validates
//! a node's capability bindings against the registry and produces a
//! [`Capabilities`] for consumption.

use super::{
    Capabilities, CapabilityRegistry, ClonePerConsumerLocalFactory, ConsumedTracker, Error,
    LocalCapabilityFactory, ResolvedLocalEntry, ResolvedSharedEntry,
};
use otap_df_config::ExtensionId;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

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
/// `bindings` is iterated in unspecified order (it's a `HashMap`), so
/// when multiple bindings are invalid the specific one reported is not
/// stable across runs.
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
        let has_native_local = registry.has_native_local(&cap_type_id);
        let has_shared = registry.has_shared(&cap_type_id);
        if !has_native_local && !has_shared {
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
            let consumed = tracker.ensure_local_consumer_slot(
                cap_type_id,
                known_cap.name,
                local_entry.extension_id.clone(),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    factory: local_entry.factory.clone_box(),
                    consumed,
                },
            );
        } else if let Some(shared_entry) = shared_entry {
            // SharedAsLocal fallback: invoke the shared factory's
            // `adapt_as_local_any` to mint a fresh local trait object,
            // then wrap that `Rc<dyn Any>` in a
            // `ClonePerConsumerLocalFactory`
            // so the fallback flows through the same
            // `LocalCapabilityFactory` path as a native local
            // registration. Each node gets a fresh adapter
            // (shared-across-components within the node once cached).
            // Shared registrations are always adaptable to local — the
            // shared factory's impl is the single source of truth and
            // contains no downcast or panic path.
            let rc_any = shared_entry.factory.adapt_as_local_any();
            let local_factory: Box<dyn LocalCapabilityFactory> =
                Box::new(ClonePerConsumerLocalFactory::new(rc_any));

            // The extension only provided a shared variant — route the
            // adapter consumer's cell under the shared bucket. The
            // follow-up `// Resolve shared entry` block below also calls
            // `ensure_shared_consumer_slot` for this `(cap, ext)` pair;
            // that method is get-or-insert, so both users share the same
            // `Rc<Cell<bool>>` and either one flipping marks the shared
            // variant consumed. No phantom entry is created in
            // `tracker.local`.
            let consumed = tracker.ensure_shared_consumer_slot(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
            );

            let _ = local_entries.insert(
                cap_type_id,
                ResolvedLocalEntry {
                    factory: local_factory,
                    consumed,
                },
            );
        }

        // Resolve shared entry. If the SharedAsLocal fallback above
        // already called `ensure_shared_consumer_slot` for this
        // `(cap, ext)` pair, this call is a no-op lookup that returns
        // the same cell — get-or-insert idempotency keeps the two
        // users pointing at one slot.
        if let Some(shared_entry) = shared_entry {
            let consumed = tracker.ensure_shared_consumer_slot(
                cap_type_id,
                known_cap.name,
                shared_entry.extension_id.clone(),
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
