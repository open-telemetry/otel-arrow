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

// Bridge fn: turns an owned `SharedImpl` into a `Box<dyn TestCapShared>`.
// This is the per-`(cap, ext)` coercion that macro-generated
// registration code will emit for real capabilities; tests use it
// directly to construct `ClonePerConsumerSharedFactory<TestCap, SharedImpl>`.
fn shared_impl_as_box(s: SharedImpl) -> Box<dyn TestCapShared> {
    Box::new(s)
}

fn test_factory(val: &'static str) -> ClonePerConsumerSharedFactory<TestCap, SharedImpl> {
    ClonePerConsumerSharedFactory::<TestCap, SharedImpl>::new(SharedImpl(val), shared_impl_as_box)
}

fn register_shared(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
    let ext_id: ExtensionId = ext_id.into();
    registry
        .register_shared(
            TypeId::of::<TestCap>(),
            SharedCapabilityEntry {
                extension_id: ext_id,
                factory: Box::new(test_factory(val)),
            },
        )
        .unwrap();
}

fn register_local(registry: &mut CapabilityRegistry, ext_id: &'static str, val: &'static str) {
    let ext_id: ExtensionId = ext_id.into();
    let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl(val));
    let instance: Rc<dyn Any> = Rc::new(rc_local);
    registry
        .register_local(
            TypeId::of::<TestCap>(),
            LocalCapabilityEntry {
                extension_id: ext_id,
                factory: Box::new(ClonePerConsumerLocalFactory::new(instance)),
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

    #[derive(Clone)]
    struct CountingFactory;
    impl SharedCapabilityFactory for CountingFactory {
        fn clone_box(&self) -> Box<dyn SharedCapabilityFactory> {
            Box::new(self.clone())
        }
        fn produce_any(&self) -> Box<dyn Any + Send> {
            let _ = CLONE_COUNT.fetch_add(1, Ordering::SeqCst);
            let shared: Box<dyn TestCapShared> = Box::new(SharedImpl("val"));
            Box::new(shared)
        }
        fn adapt_as_local_any(&self) -> Rc<dyn Any> {
            let _ = CLONE_COUNT.fetch_add(1, Ordering::SeqCst);
            let shared: Box<dyn TestCapShared> = Box::new(SharedImpl("val"));
            let rc_local =
                <TestCap as super::super::ExtensionCapability>::wrap_shared_as_local(shared);
            Rc::new(rc_local)
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
    let instance: Rc<dyn Any> = Rc::new(rc_local);
    let err = reg
        .register_local(
            TypeId::of::<TestCap>(),
            LocalCapabilityEntry {
                extension_id: "ext-a".into(),
                factory: Box::new(ClonePerConsumerLocalFactory::new(instance)),
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
                factory: Box::new(test_factory("v2")),
            },
        )
        .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("duplicate"), "error: {msg}");
    assert!(msg.contains("ext-a"), "error: {msg}");
}

// ── FreshPerConsumer factory tests ───────────────────────────────────

/// Each call to `produce_any` on a fresh-factory mints a new
/// instance: the counter captured by the closure increments,
/// proving independent construction.
#[test]
fn test_fresh_shared_factory_produces_independent_instances() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_c = Arc::clone(&counter);
    let fac = FreshPerConsumerSharedFactory::<TestCap, _>::new(move || {
        let n = counter_c.fetch_add(1, Ordering::SeqCst);
        // Leak a &'static str via Box is overkill; just use a fixed value —
        // the assertion is on call count, not per-call payload distinctness.
        let _ = n;
        Box::new(SharedImpl("fresh")) as Box<dyn TestCapShared>
    });

    let _ = fac.produce_any();
    let _ = fac.produce_any();
    let _ = fac.produce_any();
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

/// `clone_box` must duplicate the factory so per-node resolved
/// entries can each mint their own instances.
#[test]
fn test_fresh_shared_factory_clone_box_preserves_closure() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_c = Arc::clone(&counter);
    let fac = FreshPerConsumerSharedFactory::<TestCap, _>::new(move || {
        let _ = counter_c.fetch_add(1, Ordering::SeqCst);
        Box::new(SharedImpl("fresh")) as Box<dyn TestCapShared>
    });

    let cloned = fac.clone_box();
    let _ = fac.produce_any();
    let _ = cloned.produce_any();
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

/// `adapt_as_local_any` for a fresh shared factory must invoke the
/// closure and route through `wrap_shared_as_local`, yielding a
/// downcastable `Rc<dyn TestCapLocal>`.
#[test]
fn test_fresh_shared_factory_adapt_as_local() {
    let fac = FreshPerConsumerSharedFactory::<TestCap, _>::new(|| {
        Box::new(SharedImpl("fresh-via-local")) as Box<dyn TestCapShared>
    });
    let any_rc = fac.adapt_as_local_any();
    let rc_local = any_rc.downcast::<Rc<dyn TestCapLocal>>().unwrap();
    assert_eq!(rc_local.value(), "fresh-via-local");
}

/// Each call to `produce_any` on a fresh local factory yields a
/// *different* underlying `Rc<dyn TestCapLocal>` — verified by
/// comparing raw pointers of the downcast trait objects.
#[test]
fn test_fresh_local_factory_produces_independent_instances() {
    let fac = FreshPerConsumerLocalFactory::new(|| {
        let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl("fresh"));
        Rc::new(rc_local) as Rc<dyn Any>
    });

    let a = fac.produce_any();
    let b = fac.produce_any();
    let a_inner = a.downcast::<Rc<dyn TestCapLocal>>().unwrap();
    let b_inner = b.downcast::<Rc<dyn TestCapLocal>>().unwrap();
    assert!(!Rc::ptr_eq(&a_inner, &b_inner));
    assert_eq!(a_inner.value(), "fresh");
    assert_eq!(b_inner.value(), "fresh");
}

/// `clone_box` on a fresh local factory duplicates the closure;
/// both the original and the clone still produce independent
/// instances on each call.
#[test]
fn test_fresh_local_factory_clone_box() {
    let fac = FreshPerConsumerLocalFactory::new(|| {
        let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl("fresh-clone"));
        Rc::new(rc_local) as Rc<dyn Any>
    });

    let cloned = fac.clone_box();
    let a = fac.produce_any();
    let b = cloned.produce_any();
    let a_inner = a.downcast::<Rc<dyn TestCapLocal>>().unwrap();
    let b_inner = b.downcast::<Rc<dyn TestCapLocal>>().unwrap();
    assert!(!Rc::ptr_eq(&a_inner, &b_inner));
}

/// A fresh shared factory plugs into `SharedCapabilityEntry` and
/// flows through `resolve_bindings` / `require_shared` the same
/// way a clone factory does.
#[test]
fn test_fresh_shared_factory_via_registry() {
    let mut reg = CapabilityRegistry::new();
    reg.register_shared(
        TypeId::of::<TestCap>(),
        SharedCapabilityEntry {
            extension_id: "ext-a".into(),
            factory: Box::new(FreshPerConsumerSharedFactory::<TestCap, _>::new(|| {
                Box::new(SharedImpl("fresh-reg")) as Box<dyn TestCapShared>
            })),
        },
    )
    .unwrap();

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let s = caps.require_shared::<TestCap>().unwrap();
    assert_eq!(s.value(), "fresh-reg");
}

/// A fresh local factory plugs into `LocalCapabilityEntry` and
/// flows through `resolve_bindings` / `require_local` the same
/// way a clone factory does.
#[test]
fn test_fresh_local_factory_via_registry() {
    let mut reg = CapabilityRegistry::new();
    reg.register_local(
        TypeId::of::<TestCap>(),
        LocalCapabilityEntry {
            extension_id: "ext-a".into(),
            factory: Box::new(FreshPerConsumerLocalFactory::new(|| {
                let rc_local: Rc<dyn TestCapLocal> = Rc::new(LocalImpl("fresh-local-reg"));
                Rc::new(rc_local) as Rc<dyn Any>
            })),
        },
    )
    .unwrap();

    let mut tracker = ConsumedTracker::new();
    let caps = resolve_bindings(
        &bindings("test_cap", "ext-a"),
        &reg,
        &known_exts(&["ext-a"]),
        &mut tracker,
    )
    .unwrap();
    let l = caps.require_local::<TestCap>().unwrap();
    assert_eq!(l.value(), "fresh-local-reg");
}
