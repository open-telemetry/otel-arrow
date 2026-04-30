// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the capability registry.
//!
//! These exercise the registry infrastructure — register, resolve,
//! consume, SharedAsLocal adapter — via a hand-written `TestCap`
//! capability. The `#[capability]` proc macro cannot be invoked from
//! within the engine crate because proc macros cannot target paths in
//! their own host crate (`crate::capability::*`); the hand-written
//! impl below mirrors what the macro emits.

use super::*;
use otap_df_config::ExtensionId;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ── Hand-written test capability ─────────────────────────────────────

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

    fn wrap_shared_as_local(shared: Box<Self::Shared>) -> Rc<Self::Local> {
        struct Adapter(Box<dyn TestCapShared>);
        impl TestCapLocal for Adapter {
            fn value(&self) -> &str {
                self.0.value()
            }
        }
        Rc::new(Adapter(shared))
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
};

// ── `#[capability]`-style helpers on TestCap ────────────────────────────────
//
// The `#[capability]` proc macro emits these on real capabilities. The
// extension-side `extension_capabilities!(shared: E => [TestCap])` macro
// expansion calls them to bridge an `ExtensionId` + instance factory
// into a registry entry. Hand-rolled here so the `TestCap` in this
// test module can stand in for a macro-generated capability.
impl TestCap {
    fn shared_entry<E>(
        extension_id: ExtensionId,
        factory: crate::capability::SharedInstanceFactory,
    ) -> SharedCapabilityEntry
    where
        E: TestCapShared + 'static,
    {
        let produce = move || -> Box<dyn Any + Send> {
            let erased = factory.produce();
            let concrete: Box<E> = erased
                .downcast()
                .expect("instance_factory produced wrong type");
            let shared: Box<dyn TestCapShared> = concrete;
            Box::new(shared) as Box<dyn Any + Send>
        };
        let adapt_as_local: fn(Box<dyn Any + Send>) -> Rc<dyn Any> = |erased| {
            let shared: Box<Box<dyn TestCapShared>> = erased.downcast().expect("envelope");
            let rc_local =
                <TestCap as super::super::ExtensionCapability>::wrap_shared_as_local(*shared);
            Rc::new(rc_local) as Rc<dyn Any>
        };
        SharedCapabilityEntry::new(extension_id, produce, adapt_as_local)
    }

    fn local_entry<E>(
        extension_id: ExtensionId,
        factory: crate::capability::LocalInstanceFactory,
    ) -> LocalCapabilityEntry
    where
        E: TestCapLocal + 'static,
    {
        let produce = move || -> Rc<dyn Any> {
            let erased = factory.produce();
            let concrete: Rc<E> = erased
                .downcast()
                .expect("instance_factory produced wrong type");
            let local: Rc<dyn TestCapLocal> = concrete;
            Rc::new(local) as Rc<dyn Any>
        };
        LocalCapabilityEntry::new(extension_id, produce)
    }
}

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

// Build a SharedInstanceFactory that always produces a new
// `Box<SharedImpl>` with the given value. Mimics the builder's
// clone-per-consumer output.
fn shared_instance_factory(val: &'static str) -> crate::capability::SharedInstanceFactory {
    crate::capability::SharedInstanceFactory::new(move || {
        Box::new(SharedImpl(val)) as Box<dyn Any + Send>
    })
}

// Build a LocalInstanceFactory producing a shared `Rc<LocalImpl>`.
fn local_instance_factory(val: &'static str) -> crate::capability::LocalInstanceFactory {
    let shared: Rc<LocalImpl> = Rc::new(LocalImpl(val));
    crate::capability::LocalInstanceFactory::new(move || Rc::clone(&shared) as Rc<dyn Any>)
}

fn register_shared(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
    let ext_id: ExtensionId = ext_id.into();
    registry
        .register_shared(
            TypeId::of::<TestCap>(),
            TestCap::shared_entry::<SharedImpl>(ext_id, shared_instance_factory(val)),
        )
        .unwrap();
}

fn register_local(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
    let ext_id: ExtensionId = ext_id.into();
    registry
        .register_local(
            TypeId::of::<TestCap>(),
            TestCap::local_entry::<LocalImpl>(ext_id, local_instance_factory(val)),
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

    // Use two independent resolutions because `require_shared` and
    // the `SharedAsLocal`-backed `require_local` share the same
    // consumed cell, and the one-shot contract only allows one claim
    // per (cap, ext) pair per node.
    let mut tracker = ConsumedTracker::new();
    let caps_shared = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let shared = caps_shared.require_shared::<TestCap>().unwrap();
    assert_eq!(shared.value(), "shared-val");

    let caps_local = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let local = caps_local.require_local::<TestCap>().unwrap();
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
    assert!(caps.optional_local::<TestCap>().unwrap().is_none());
    assert!(caps.optional_shared::<TestCap>().unwrap().is_none());
}

#[test]
fn test_extension_capabilities_shared_only() {
    let ec = super::super::ExtensionCapabilities {
        shared: &["bearer_token_provider"],
        local: &[],
        register_shared: |_, _, _| Ok(()),
        register_local: |_, _, _| Ok(()),
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
/// This preserves the per-node instance semantics that a shared impl
/// may rely on (e.g. per-caller mutable state) — the adapter is not
/// pre-built once and shared across nodes.
#[test]
fn test_shared_as_local_builds_new_adapter_per_node() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // The shared instance factory bumps a counter every time it
    // mints a new `SharedImpl`.
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_for_closure = Arc::clone(&counter);
    let factory = crate::capability::SharedInstanceFactory::new(move || {
        let _ = counter_for_closure.fetch_add(1, Ordering::SeqCst);
        Box::new(SharedImpl("val")) as Box<dyn Any + Send>
    });

    let mut reg = CapabilityRegistry::new();
    reg.register_shared(
        TypeId::of::<TestCap>(),
        TestCap::shared_entry::<SharedImpl>("ext-a".into(), factory),
    )
    .unwrap();

    // The adapter must not run at registration time — work is deferred
    // to the consumer's `require_local` call.
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    // Two nodes bind the capability; resolve alone does not mint
    // (factory invocation is deferred to the one-shot `require_local`
    // call on each node's `Capabilities`).
    let mut tracker = ConsumedTracker::new();
    let caps_a = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let caps_b = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "fallback must not mint at resolve time",
    );

    // Each node's `require_local` mints its own clone via the shared
    // factory.
    let _ = caps_a.require_local::<TestCap>().unwrap();
    let _ = caps_b.require_local::<TestCap>().unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

/// Duplicate registrations indicate a programmer bug — the registry
/// must reject them loudly rather than silently overwriting.
#[test]
fn test_register_local_rejects_duplicate() {
    let mut reg = CapabilityRegistry::new();
    register_local(&mut reg, "ext-a", "v1");

    let err = reg
        .register_local(
            TypeId::of::<TestCap>(),
            TestCap::local_entry::<LocalImpl>("ext-a".into(), local_instance_factory("v2")),
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
            TestCap::shared_entry::<SharedImpl>("ext-a".into(), shared_instance_factory("v2")),
        )
        .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("duplicate"), "error: {msg}");
    assert!(msg.contains("ext-a"), "error: {msg}");
}

// ── One-shot consumption contract ───────────────────────────────────────────

#[test]
fn test_require_local_second_call_returns_already_consumed() {
    let mut reg = CapabilityRegistry::new();
    register_local(&mut reg, "ext-a", "val");

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();

    let _ = caps.require_local::<TestCap>().unwrap();
    let err = match caps.require_local::<TestCap>() {
        Err(e) => e,
        Ok(_) => panic!("expected CapabilityAlreadyConsumed"),
    };
    assert!(
        matches!(err, crate::error::Error::CapabilityAlreadyConsumed { ref capability } if capability == "test_cap"),
        "expected CapabilityAlreadyConsumed, got {err:?}"
    );
}

#[test]
fn test_require_shared_second_call_returns_already_consumed() {
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

    let _ = caps.require_shared::<TestCap>().unwrap();
    let err = match caps.require_shared::<TestCap>() {
        Err(e) => e,
        Ok(_) => panic!("expected CapabilityAlreadyConsumed"),
    };
    assert!(
        matches!(err, crate::error::Error::CapabilityAlreadyConsumed { ref capability } if capability == "test_cap"),
        "expected CapabilityAlreadyConsumed, got {err:?}"
    );
}

#[test]
fn test_optional_local_second_call_returns_already_consumed() {
    let mut reg = CapabilityRegistry::new();
    register_local(&mut reg, "ext-a", "val");

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();

    let first = caps.optional_local::<TestCap>().unwrap();
    assert!(first.is_some());
    let err = match caps.optional_local::<TestCap>() {
        Err(e) => e,
        Ok(_) => panic!("expected CapabilityAlreadyConsumed"),
    };
    assert!(matches!(
        err,
        crate::error::Error::CapabilityAlreadyConsumed { .. }
    ));
}

#[test]
fn test_optional_shared_second_call_returns_already_consumed() {
    // Symmetric to `test_optional_local_second_call_returns_already_consumed`:
    // even when the binding exists, `optional_shared` is governed by the
    // same per-node one-shot guard as `require_shared`, so a second call
    // on the same node returns `CapabilityAlreadyConsumed` rather than
    // silently re-minting an instance.
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

    let first = caps.optional_shared::<TestCap>().unwrap();
    assert!(first.is_some());
    let err = match caps.optional_shared::<TestCap>() {
        Err(e) => e,
        Ok(_) => panic!("expected CapabilityAlreadyConsumed"),
    };
    assert!(matches!(
        err,
        crate::error::Error::CapabilityAlreadyConsumed { .. }
    ));
}

#[test]
fn test_fallback_local_and_shared_share_one_shot_guard() {
    // SharedAsLocal fallback creates two per-node resolved entries
    // (one in `local_entries`, one in `shared_entries`) backed by the
    // same shared registration. They share a single per-binding
    // `claimed` flag so the binding may be claimed at most once per
    // node across all four accessors — claiming the local
    // execution model (via the adapter) consumes the binding for
    // the native shared execution model too, and vice versa.
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

    let local = caps.require_local::<TestCap>().unwrap();
    assert_eq!(local.value(), "val");
    match caps.require_shared::<TestCap>() {
        Err(crate::error::Error::CapabilityAlreadyConsumed { capability }) => {
            assert_eq!(capability, "test_cap");
        }
        Err(other) => panic!("expected CapabilityAlreadyConsumed, got {other:?}"),
        Ok(_) => panic!("expected CapabilityAlreadyConsumed after fallback-local claim, got Ok"),
    }

    // Re-resolving on the same registry yields a fresh per-binding
    // claim cell, and now the shared execution model can be claimed
    // first while the fallback-local execution model is rejected.
    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let shared = caps.require_shared::<TestCap>().unwrap();
    assert_eq!(shared.value(), "val");
    match caps.require_local::<TestCap>() {
        Err(crate::error::Error::CapabilityAlreadyConsumed { capability }) => {
            assert_eq!(capability, "test_cap");
        }
        Err(other) => panic!("expected CapabilityAlreadyConsumed, got {other:?}"),
        Ok(_) => panic!("expected CapabilityAlreadyConsumed after native-shared claim, got Ok"),
    }
}

// ── End-to-end: builder → bundle → register_into → resolve → require ────────
//
// Proves the full wiring from the typestate builder (which owns the
// `SharedInstanceFactory`), through `ExtensionBundle::register_into`
// (which calls the `ExtensionCapabilities` fn pointers), through the
// `#[capability]`-style `shared_entry::<E>` helper (which wraps
// `factory.produce()` in a downcast + coercion closure), through
// `CapabilityRegistry` + `resolve_bindings`, out to `require_shared`.

#[test]
fn test_end_to_end_shared_only_via_bundle() {
    use crate::capability::ExtensionCapabilities;
    use crate::config::ExtensionConfig;
    use crate::extension::ExtensionWrapper;
    use otap_df_config::extension::ExtensionUserConfig;
    use std::sync::Arc;

    // 1. Build a passive-cloned shared bundle around SharedImpl.
    let name: ExtensionId = "azure-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:azure".into(),
        serde_json::Value::Null,
    ));
    let runtime_config = ExtensionConfig::new("azure-auth");
    let bundle = ExtensionWrapper::builder(name.clone(), user_config, &runtime_config)
        .passive()
        .cloned()
        .shared(SharedImpl("token-123"))
        .build()
        .expect("bundle builds");

    // 2. Build the ExtensionCapabilities descriptor the way the
    //    extension_capabilities! macro would.
    let caps = ExtensionCapabilities {
        shared: &["test_cap"],
        local: &[],
        register_shared: |ext_id, factory, registry| {
            registry.register_shared(
                TypeId::of::<TestCap>(),
                TestCap::shared_entry::<SharedImpl>(ext_id, factory),
            )
        },
        register_local: |_, _, _| Ok(()),
    };

    // 3. Drain bundle → registry via register_into.
    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(Some(&caps), &mut registry)
        .expect("register_into");

    // 4. Resolve a fake node's bindings and consume the shared cap.
    let mut tracker = ConsumedTracker::new();
    let resolved = resolve_bindings(
        &bindings("test_cap", "azure-auth"),
        &registry,
        &known_exts(&["azure-auth"]),
        &mut tracker,
    )
    .expect("resolve");

    let shared = resolved
        .require_shared::<TestCap>()
        .expect("require_shared");
    assert_eq!(shared.value(), "token-123");

    // Fallback: SharedAsLocal adapter flows through the same bundle.
    // The one-shot contract on `Capabilities` means the `require_shared`
    // above already consumed the (cap, ext) slot, so we resolve a
    // separate node to exercise the local-via-shared path.
    let resolved2 = resolve_bindings(
        &bindings("test_cap", "azure-auth"),
        &registry,
        &known_exts(&["azure-auth"]),
        &mut tracker,
    )
    .expect("resolve");
    let local = resolved2.require_local::<TestCap>().expect("require_local");
    assert_eq!(local.value(), "token-123");
}

#[test]
fn test_end_to_end_local_only_via_bundle() {
    use crate::capability::ExtensionCapabilities;
    use crate::config::ExtensionConfig;
    use crate::extension::ExtensionWrapper;
    use otap_df_config::extension::ExtensionUserConfig;
    use std::sync::Arc;

    let name: ExtensionId = "kv".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:kv".into(),
        serde_json::Value::Null,
    ));
    let runtime_config = ExtensionConfig::new("kv");
    let bundle = ExtensionWrapper::builder(name.clone(), user_config, &runtime_config)
        .passive()
        .cloned()
        .local(Rc::new(LocalImpl("kv-value")))
        .build()
        .expect("bundle builds");

    let caps = ExtensionCapabilities {
        shared: &[],
        local: &["test_cap"],
        register_shared: |_, _, _| Ok(()),
        register_local: |ext_id, factory, registry| {
            registry.register_local(
                TypeId::of::<TestCap>(),
                TestCap::local_entry::<LocalImpl>(ext_id, factory),
            )
        },
    };

    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(Some(&caps), &mut registry)
        .expect("register_into");

    let mut tracker = ConsumedTracker::new();
    let resolved = resolve_bindings(
        &bindings("test_cap", "kv"),
        &registry,
        &known_exts(&["kv"]),
        &mut tracker,
    )
    .expect("resolve");

    let local = resolved.require_local::<TestCap>().expect("require_local");
    assert_eq!(local.value(), "kv-value");
}

#[test]
fn test_end_to_end_shared_constructed_policy_mints_independent_instances() {
    // With the `.constructed()` (factory) policy, the instance factory
    // invokes the user's closure on each produce(); each consumer
    // should observe its own instance. Shared+Clone consumers observe
    // clone-of-prototype semantics — also independent here since
    // SharedImpl has no interior mutability.
    use crate::capability::ExtensionCapabilities;
    use crate::config::ExtensionConfig;
    use crate::extension::ExtensionWrapper;
    use otap_df_config::extension::ExtensionUserConfig;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let name: ExtensionId = "counter".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:counter".into(),
        serde_json::Value::Null,
    ));
    let runtime_config = ExtensionConfig::new("counter");

    // ConstructedImpl: each `start()` increments a shared counter so
    // we can confirm the factory ran per-consumer.
    #[derive(Clone)]
    struct ConstructedImpl(Arc<AtomicUsize>, &'static str);
    impl TestCapShared for ConstructedImpl {
        fn value(&self) -> &str {
            self.1
        }
    }

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_for_closure = Arc::clone(&counter);

    let bundle = ExtensionWrapper::builder(name.clone(), user_config, &runtime_config)
        .passive()
        .constructed()
        .shared::<ConstructedImpl, _>(move || {
            let _ = counter_for_closure.fetch_add(1, Ordering::SeqCst);
            ConstructedImpl(Arc::clone(&counter_for_closure), "constructed")
        })
        .build()
        .expect("bundle builds");

    let caps = ExtensionCapabilities {
        shared: &["test_cap"],
        local: &[],
        register_shared: |ext_id, factory, registry| {
            registry.register_shared(
                TypeId::of::<TestCap>(),
                TestCap::shared_entry::<ConstructedImpl>(ext_id, factory),
            )
        },
        register_local: |_, _, _| Ok(()),
    };

    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(Some(&caps), &mut registry)
        .expect("register_into");

    let mut tracker = ConsumedTracker::new();
    let resolved_a = resolve_bindings(
        &bindings("test_cap", "counter"),
        &registry,
        &known_exts(&["counter"]),
        &mut tracker,
    )
    .expect("resolve");
    let resolved_b = resolve_bindings(
        &bindings("test_cap", "counter"),
        &registry,
        &known_exts(&["counter"]),
        &mut tracker,
    )
    .expect("resolve");

    // One claim per resolved Capabilities (one-shot contract). The
    // `.constructed()` policy invokes the user's factory on each
    // `produce()`, so two consumers should observe two factory
    // invocations and no resolve-time mint (the `SharedAsLocal`
    // fallback is deferred to the `require_local` call, which is never
    // made here).
    let s1 = resolved_a.require_shared::<TestCap>().unwrap();
    let s2 = resolved_b.require_shared::<TestCap>().unwrap();
    assert_eq!(s1.value(), "constructed");
    assert_eq!(s2.value(), "constructed");
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "constructed factory should have been invoked exactly 2x (one per require_shared consumer)"
    );
}

// ── Envelope / error-shape regression tests ─────────────────────────────────

/// `register_into` must reject a metadata-vs-bundle mismatch: when the
/// `extension_capabilities!` macro advertises an execution model that the runtime
/// `ExtensionBundle` does not contain, the call fails fast with
/// `Error::InternalError` instead of silently skipping registration
/// and letting the drift surface as a confusing
/// "extension does not provide capability" `ConfigError` from
/// `resolve_bindings`. The macro only checks the type-level trait
/// bound; this guard catches the runtime bundle shape.
#[test]
fn test_register_into_rejects_metadata_vs_bundle_mismatch() {
    use crate::capability::ExtensionCapabilities;
    use crate::config::ExtensionConfig;
    use crate::extension::ExtensionWrapper;
    use otap_df_config::extension::ExtensionUserConfig;
    use std::sync::Arc;

    let name: ExtensionId = "drifty".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:drifty".into(),
        serde_json::Value::Null,
    ));
    let runtime_config = ExtensionConfig::new("drifty");

    // Build a shared-only bundle (no local variant).
    let bundle = ExtensionWrapper::builder(name.clone(), user_config, &runtime_config)
        .passive()
        .cloned()
        .shared(SharedImpl("v"))
        .build()
        .expect("bundle builds");

    // ExtensionCapabilities advertises BOTH sides as if the macro had
    // been written with the dual form, even though the bundle only
    // provides the shared variant. This is the drift the guard must
    // catch.
    let caps = ExtensionCapabilities {
        shared: &["test_cap"],
        local: &["test_cap"],
        register_shared: |ext_id, factory, registry| {
            registry.register_shared(
                TypeId::of::<TestCap>(),
                TestCap::shared_entry::<SharedImpl>(ext_id, factory),
            )
        },
        register_local: |_ext_id, _factory, _registry| Ok(()),
    };

    let mut registry = CapabilityRegistry::new();
    let err = bundle
        .register_into(Some(&caps), &mut registry)
        .expect_err("register_into must reject metadata-vs-bundle drift");
    match err {
        Error::InternalError { message } => {
            assert!(
                message.contains("local") && message.contains("drifty"),
                "InternalError message should name the extension and the missing execution model; got: {message}",
            );
        }
        other => panic!("expected InternalError, got {other:?}"),
    }

    // Symmetric direction: shared advertised, bundle is local-only.
    let local_only_name: ExtensionId = "drifty-local".into();
    let local_only_user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:drifty-local".into(),
        serde_json::Value::Null,
    ));
    let local_only_runtime_config = ExtensionConfig::new("drifty-local");
    let bundle = ExtensionWrapper::builder(
        local_only_name,
        local_only_user_config,
        &local_only_runtime_config,
    )
    .passive()
    .cloned()
    .local(Rc::new(LocalImpl("v")))
    .build()
    .expect("bundle builds");

    let caps = ExtensionCapabilities {
        shared: &["test_cap"],
        local: &["test_cap"],
        register_shared: |_ext_id, _factory, _registry| Ok(()),
        register_local: |ext_id, factory, registry| {
            registry.register_local(
                TypeId::of::<TestCap>(),
                TestCap::local_entry::<LocalImpl>(ext_id, factory),
            )
        },
    };
    let err = bundle
        .register_into(Some(&caps), &mut registry)
        .expect_err("register_into must reject metadata-vs-bundle drift");
    match err {
        Error::InternalError { message } => {
            assert!(
                message.contains("shared") && message.contains("drifty-local"),
                "InternalError message should name the extension and the missing execution model; got: {message}",
            );
        }
        other => panic!("expected InternalError, got {other:?}"),
    }
}

/// The shared-side registry envelope must be `Box<Box<dyn C::Shared>>`
/// erased as `Box<dyn Any + Send>`. `require_shared` downcasts the
/// outer `Any` to `Box<dyn C::Shared>` then dereferences. This test
/// pins the shape so anyone refactoring the macro cannot collapse the
/// double-box without the consumer path failing loudly.
#[test]
fn test_shared_entry_produce_uses_double_box_envelope() {
    let factory = shared_instance_factory("envelope-val");
    let entry = TestCap::shared_entry::<SharedImpl>("ext-env".into(), factory);

    // Directly invoke the stored produce closure and downcast using
    // the exact shape the consumer (`require_shared`) relies on.
    let erased: Box<dyn Any + Send> = (entry.produce)();
    let boxed_trait_object: Box<Box<dyn TestCapShared>> = erased
        .downcast::<Box<dyn TestCapShared>>()
        .expect("shared_entry must emit Box<Box<dyn C::Shared>> erased as Box<dyn Any + Send>");
    assert_eq!((*boxed_trait_object).value(), "envelope-val");
}

/// `require_local` on an unbound capability must return the dedicated
/// `CapabilityNotBound` variant (not a generic `ConfigError`) so the
/// diagnostic points at "node-code/declaration mismatch", not
/// "user YAML problem".
#[test]
fn test_require_local_unbound_returns_capability_not_bound() {
    let reg = CapabilityRegistry::new();
    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(&HashMap::new(), &reg, &known_exts(&[]), &mut tracker).unwrap();

    match caps.require_local::<TestCap>() {
        Err(Error::CapabilityNotBound {
            capability,
            execution_model,
        }) => {
            assert_eq!(capability, "test_cap");
            assert_eq!(execution_model, "local");
        }
        Err(other) => panic!("expected CapabilityNotBound, got {other:?}"),
        Ok(_) => panic!("expected CapabilityNotBound, got Ok"),
    }
}

#[test]
fn test_require_shared_unbound_returns_capability_not_bound() {
    let reg = CapabilityRegistry::new();
    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(&HashMap::new(), &reg, &known_exts(&[]), &mut tracker).unwrap();

    match caps.require_shared::<TestCap>() {
        Err(Error::CapabilityNotBound {
            capability,
            execution_model,
        }) => {
            assert_eq!(capability, "test_cap");
            assert_eq!(execution_model, "shared");
        }
        Err(other) => panic!("expected CapabilityNotBound, got {other:?}"),
        Ok(_) => panic!("expected CapabilityNotBound, got Ok"),
    }
}

/// When a node binds a shared-only extension through its *local*-facing
/// capability (the `SharedAsLocal` fallback), consumption of the
/// fallback adapter must flip the shared bucket of `ConsumedTracker`
/// — not a phantom local bucket. Otherwise `unconsumed_shared` would
/// claim the shared variant is unused and the engine would drop it
/// while the adapter still depended on it.
#[test]
fn test_fallback_local_consumption_flips_shared_bucket() {
    let mut reg = CapabilityRegistry::new();
    register_shared(&mut reg, "ext-only-shared", "val");

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-only-shared"),
        &reg,
        &known_exts(&["ext-only-shared"]),
        &mut tracker,
    )
    .unwrap();

    // Before claim: both buckets show the shared extension unconsumed.
    assert!(
        tracker
            .unconsumed_shared()
            .iter()
            .any(|(ext, _)| ext.as_ref() == "ext-only-shared"),
        "shared bucket should list ext-only-shared before the fallback claim",
    );
    assert!(
        tracker.unconsumed_local().is_empty(),
        "no native local registration was made, so the local bucket must be empty",
    );

    // Fallback claim (local-facing, but backed by the shared factory).
    let _ = caps.require_local::<TestCap>().unwrap();

    // After claim: shared bucket flips (the shared *variant* is what
    // got used).
    assert!(
        tracker.unconsumed_shared().is_empty(),
        "fallback consumption must flip the shared bucket",
    );
}

#[test]
fn test_native_dual_local_claim_invalidates_shared_on_same_node() {
    // Native dual: extension `ext-a` registered both a native local and
    // a native shared implementation. The per-binding one-shot contract
    // applies uniformly: claiming `require_local` on a node consumes the
    // binding for that node, so a subsequent `require_shared` on the same
    // node returns `CapabilityAlreadyConsumed` even though the shared
    // entry is a distinct object. This is enforced by `require_local`
    // taking (and dropping unrun) the shared entry's `produce` cell on
    // success — no auxiliary flag needed.
    //
    // Counterpart: `test_native_dual_shared_claim_invalidates_local_on_same_node`
    // verifies the symmetric direction.
    let mut reg = CapabilityRegistry::new();
    register_local(&mut reg, "ext-a", "local-val");
    register_shared(&mut reg, "ext-a", "shared-val");

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();

    // First: native-local claim succeeds and yields the native local
    // implementation (not the shared-as-local adapter).
    let local = caps.require_local::<TestCap>().unwrap();
    assert_eq!(local.value(), "local-val");

    // Per-binding one-shot: shared side is now invalidated on this node.
    match caps.require_shared::<TestCap>() {
        Err(crate::error::Error::CapabilityAlreadyConsumed { capability }) => {
            assert_eq!(capability, "test_cap");
        }
        Err(other) => panic!("expected CapabilityAlreadyConsumed, got {other:?}"),
        Ok(_) => panic!("expected CapabilityAlreadyConsumed after native-local claim, got Ok"),
    }
    // Same for the second call on the originally-claimed side.
    assert!(matches!(
        caps.require_local::<TestCap>(),
        Err(crate::error::Error::CapabilityAlreadyConsumed { .. })
    ));

    // Cross-node tracker: only the local bucket flipped — the shared
    // variant was *not* consumed by any node, just invalidated locally.
    // The engine should still call `drop_shared()` on `ext-a` if no
    // other node claims it.
    assert!(
        tracker.unconsumed_local().is_empty(),
        "native local claim must flip the local bucket",
    );
    assert_eq!(
        tracker.unconsumed_shared().len(),
        1,
        "native local claim must NOT flip the shared bucket — invalidating \
         the shared alternative is not the same as consuming it",
    );
}

#[test]
fn test_native_dual_shared_claim_invalidates_local_on_same_node() {
    // Symmetric counterpart to
    // `test_native_dual_local_claim_invalidates_shared_on_same_node`:
    // claiming `require_shared` first invalidates the native-local entry
    // on the same node, so a subsequent `require_local` returns
    // `CapabilityAlreadyConsumed`.
    let mut reg = CapabilityRegistry::new();
    register_local(&mut reg, "ext-a", "local-val");
    register_shared(&mut reg, "ext-a", "shared-val");

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();

    let shared = caps.require_shared::<TestCap>().unwrap();
    assert_eq!(shared.value(), "shared-val");

    match caps.require_local::<TestCap>() {
        Err(crate::error::Error::CapabilityAlreadyConsumed { capability }) => {
            assert_eq!(capability, "test_cap");
        }
        Err(other) => panic!("expected CapabilityAlreadyConsumed, got {other:?}"),
        Ok(_) => panic!("expected CapabilityAlreadyConsumed after native-shared claim, got Ok"),
    }
    assert!(matches!(
        caps.require_shared::<TestCap>(),
        Err(crate::error::Error::CapabilityAlreadyConsumed { .. })
    ));

    // Cross-node tracker: only the shared bucket flipped.
    assert!(
        tracker.unconsumed_shared().is_empty(),
        "native shared claim must flip the shared bucket",
    );
    assert_eq!(
        tracker.unconsumed_local().len(),
        1,
        "native shared claim must NOT flip the local bucket — invalidating \
         the local alternative is not the same as consuming it",
    );
}

// ── Background extension tests ──────────────────────────────────────────────

/// Background extensions pass `None` to `register_into` and contribute zero
/// entries to the registry. `ConsumedTracker` is untouched because no
/// resolution can target a Background extension.
#[test]
fn test_register_into_background_no_op() {
    use crate::config::ExtensionConfig;
    use crate::control::ExtensionControlMsg;
    use crate::error::Error as EngineError;
    use crate::extension::ExtensionWrapper;
    use crate::shared::extension as shared_ext;
    use crate::terminal_state::TerminalState;
    use async_trait::async_trait;
    use otap_df_config::extension::ExtensionUserConfig;
    use std::sync::Arc;

    // A minimal shared bg-task type. The body of `start()` is irrelevant
    // for this test — we only build the bundle and call register_into.
    #[derive(Clone)]
    struct BgTask;

    #[async_trait]
    impl shared_ext::Extension for BgTask {
        async fn start(
            self: Box<Self>,
            mut ctrl: shared_ext::ControlChannel,
            _eh: crate::extension::EffectHandler,
        ) -> Result<TerminalState, EngineError> {
            loop {
                if let ExtensionControlMsg::Shutdown { .. } = ctrl.recv().await? {
                    break;
                }
            }
            Ok(TerminalState::default())
        }
    }

    let name: ExtensionId = "bg".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        "urn:test:bg".into(),
        serde_json::Value::Null,
    ));
    let runtime_config = ExtensionConfig::new("bg");

    let bundle = ExtensionWrapper::builder(name.clone(), user_config, &runtime_config)
        .background()
        .shared(BgTask)
        .build()
        .expect("background bundle builds");

    // Background factories use `capabilities: None`, which `register_into`
    // treats as a no-op. The registry stays empty.
    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(None, &mut registry)
        .expect("background register_into is a no-op");

    assert!(
        registry
            .get_shared(&TypeId::of::<TestCap>(), "bg")
            .is_none()
    );
    assert!(registry.get_local(&TypeId::of::<TestCap>(), "bg").is_none());

    let tracker = ConsumedTracker::new();
    assert!(tracker.unconsumed_shared().is_empty());
    assert!(tracker.unconsumed_local().is_empty());
}

/// A node config that tries to bind a capability to a Background extension
/// should surface the existing Step-4 error: the Background extension is
/// known (it is loaded), but it does not provide the requested capability.
/// This is the same behavior as binding to any active/passive extension
/// that simply doesn't list the capability — Background is not a special
/// case at the resolution layer.
#[test]
fn test_resolve_bindings_background_not_a_provider() {
    // Registry contains `provider-ext` providing `test_cap`. The node
    // config tries to bind `test_cap` to `bg-ext` — a known-loaded
    // Background extension that registered nothing.
    let mut reg = CapabilityRegistry::new();
    register_shared(&mut reg, "provider-ext", "v");

    let mut tracker = ConsumedTracker::new();
    let result = resolve_bindings(
        &bindings("test_cap", "bg-ext"),
        &reg,
        &known_exts(&["bg-ext", "provider-ext"]),
        &mut tracker,
    );

    let err = result.expect_err("must reject binding to Background extension");
    let msg = format!("{err}");
    assert!(
        msg.contains("does not provide"),
        "expected Step-4 'does not provide' error; got: {msg}",
    );
}
