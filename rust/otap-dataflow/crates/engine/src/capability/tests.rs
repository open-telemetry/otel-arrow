// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests for the [`extension_capabilities!`] macro.
//!
//! Each test defines a hand-rolled test capability (the `#[capability]`
//! proc macro cannot expand inside the engine crate itself — proc
//! macros can't target paths in their host crate), invokes one arm of
//! the macro, exercises the resulting `register_*` fn pointers against
//! a real `CapabilityRegistry`, then resolves bindings and consumes the
//! capability.

use super::registry::{
    Capabilities, CapabilityRegistry, ConsumedTracker, LocalCapabilityEntry, SharedCapabilityEntry,
    resolve_bindings,
};
use super::{ExtensionCapability, KNOWN_CAPABILITIES, KnownCapability};
use crate::capability::factory::{LocalInstanceFactory, SharedInstanceFactory};
use otap_df_config::{CapabilityId, ExtensionId};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ── Test capability (mirrors what `#[capability]` generates) ─────────

trait MacroTestCapLocal {
    fn value(&self) -> &str;
}
trait MacroTestCapShared: Send {
    fn value(&self) -> &str;
}

struct MacroTestCap;
impl super::private::Sealed for MacroTestCap {}
impl ExtensionCapability for MacroTestCap {
    const NAME: &'static str = "macro_test_cap";
    type Local = dyn MacroTestCapLocal;
    type Shared = dyn MacroTestCapShared;
    fn wrap_shared_as_local(shared: Box<Self::Shared>) -> Rc<Self::Local> {
        struct Adapter(Box<dyn MacroTestCapShared>);
        impl MacroTestCapLocal for Adapter {
            fn value(&self) -> &str {
                self.0.value()
            }
        }
        Rc::new(Adapter(shared))
    }
}

// Register the test capability so resolve_bindings sees it as known.
#[allow(unsafe_code)]
#[linkme::distributed_slice(KNOWN_CAPABILITIES)]
#[linkme(crate = linkme)]
static _MACRO_TEST_CAP: KnownCapability = KnownCapability {
    name: "macro_test_cap",
    description: "Test capability for extension_capabilities! macro tests",
    type_id: || TypeId::of::<MacroTestCap>(),
};

// Casters mirror what `#[capability]` emits. The macro calls
// `<MacroTestCap>::shared_entry::<E>` and `local_entry::<E>`.
impl MacroTestCap {
    fn shared_entry<E>(ext_id: ExtensionId, factory: SharedInstanceFactory) -> SharedCapabilityEntry
    where
        E: MacroTestCapShared + 'static,
    {
        let produce = move || -> Box<dyn Any + Send> {
            let erased = factory.produce();
            let concrete: Box<E> = erased.downcast().expect("instance factory");
            let shared: Box<dyn MacroTestCapShared> = concrete;
            Box::new(shared) as Box<dyn Any + Send>
        };
        let adapt_as_local: fn(Box<dyn Any + Send>) -> Rc<dyn Any> = |erased| {
            let shared: Box<Box<dyn MacroTestCapShared>> = erased.downcast().expect("envelope");
            let rc_local = <MacroTestCap as ExtensionCapability>::wrap_shared_as_local(*shared);
            Rc::new(rc_local) as Rc<dyn Any>
        };
        SharedCapabilityEntry::new(ext_id, produce, adapt_as_local)
    }

    fn local_entry<E>(ext_id: ExtensionId, factory: LocalInstanceFactory) -> LocalCapabilityEntry
    where
        E: MacroTestCapLocal + 'static,
    {
        let produce = move || -> Rc<dyn Any> {
            let erased = factory.produce();
            let concrete: Rc<E> = erased.downcast().expect("instance factory");
            let local: Rc<dyn MacroTestCapLocal> = concrete;
            Rc::new(local) as Rc<dyn Any>
        };
        LocalCapabilityEntry::new(ext_id, produce)
    }
}

// ── Test extension impls ─────────────────────────────────────────────

#[derive(Clone)]
struct Shared(&'static str);
impl MacroTestCapShared for Shared {
    fn value(&self) -> &str {
        self.0
    }
}

struct Local(&'static str);
impl MacroTestCapLocal for Local {
    fn value(&self) -> &str {
        self.0
    }
}

fn shared_factory(val: &'static str) -> SharedInstanceFactory {
    SharedInstanceFactory::new(move || Box::new(Shared(val)) as Box<dyn Any + Send>)
}
fn local_factory(val: &'static str) -> LocalInstanceFactory {
    let shared: Rc<Local> = Rc::new(Local(val));
    LocalInstanceFactory::new(move || Rc::clone(&shared) as Rc<dyn Any>)
}

fn bindings() -> HashMap<CapabilityId, ExtensionId> {
    let mut m = HashMap::new();
    let _ = m.insert("macro_test_cap".into(), "ext".into());
    m
}
fn known_exts() -> HashSet<ExtensionId> {
    let mut s = HashSet::new();
    let _ = s.insert("ext".into());
    s
}

fn resolve(registry: &CapabilityRegistry) -> Capabilities {
    let mut tracker = ConsumedTracker::new();
    resolve_bindings(&bindings(), registry, &known_exts(), &mut tracker).expect("resolve_bindings")
}

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn macro_shared_only_form() {
    let ec = extension_capabilities!(shared: Shared => [MacroTestCap]);
    assert_eq!(ec.shared, &["macro_test_cap"]);
    assert!(ec.local.is_empty());

    let mut registry = CapabilityRegistry::new();
    (ec.register_shared)("ext".into(), shared_factory("s-only"), &mut registry)
        .expect("register_shared");
    // register_local must be a no-op even when called with a local factory.
    (ec.register_local)("ext".into(), local_factory("unused"), &mut registry)
        .expect("register_local no-op");

    // Native shared claim works.
    let caps = resolve(&registry);
    assert_eq!(
        caps.require_shared::<MacroTestCap>().unwrap().value(),
        "s-only"
    );

    // Fresh resolve: local consumers are served by the
    // SharedAsLocal fallback, returning the same value.
    let caps = resolve(&registry);
    assert_eq!(
        caps.require_local::<MacroTestCap>().unwrap().value(),
        "s-only"
    );

    // Per-binding one-shot: claiming the local execution model via
    // the SharedAsLocal fallback consumes the binding for the
    // native shared execution model too.
    let caps = resolve(&registry);
    let _ = caps.require_local::<MacroTestCap>().unwrap();
    assert!(matches!(
        caps.require_shared::<MacroTestCap>(),
        Err(crate::capability::registry::Error::CapabilityAlreadyConsumed { .. })
    ));
}

#[test]
fn macro_local_only_form() {
    let ec = extension_capabilities!(local: Local => [MacroTestCap]);
    assert!(ec.shared.is_empty());
    assert_eq!(ec.local, &["macro_test_cap"]);

    let mut registry = CapabilityRegistry::new();
    (ec.register_local)("ext".into(), local_factory("l-only"), &mut registry)
        .expect("register_local");

    let caps = resolve(&registry);
    assert_eq!(
        caps.require_local::<MacroTestCap>().unwrap().value(),
        "l-only"
    );
    assert!(caps.require_shared::<MacroTestCap>().is_err());
}

#[test]
fn macro_dual_form() {
    let ec = extension_capabilities!(
        (shared: Shared, local: Local) => [MacroTestCap]
    );
    assert_eq!(ec.shared, &["macro_test_cap"]);
    assert_eq!(ec.local, &["macro_test_cap"]);

    let mut registry = CapabilityRegistry::new();
    (ec.register_shared)("ext".into(), shared_factory("s-dual"), &mut registry)
        .expect("register_shared");
    (ec.register_local)("ext".into(), local_factory("l-dual"), &mut registry)
        .expect("register_local");

    // Native dual: local and shared are *distinct* registrations
    // (different concrete types) and therefore have independent
    // one-shot guards. A single node may claim both — each yields
    // the value of its own concrete type. The per-binding one-shot
    // contract only collapses local and shared for the SharedAsLocal
    // fallback path (where they back the same underlying object).
    let caps = resolve(&registry);
    assert_eq!(
        caps.require_shared::<MacroTestCap>().unwrap().value(),
        "s-dual"
    );
    assert_eq!(
        caps.require_local::<MacroTestCap>().unwrap().value(),
        "l-dual"
    );

    // Each execution model is still one-shot in isolation.
    let caps = resolve(&registry);
    let _ = caps.require_shared::<MacroTestCap>().unwrap();
    assert!(matches!(
        caps.require_shared::<MacroTestCap>(),
        Err(crate::capability::registry::Error::CapabilityAlreadyConsumed { .. })
    ));
}
