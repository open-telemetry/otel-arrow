// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability factory traits and built-in impls.
//!
//! Each `(capability, extension)` registration owns one factory — a
//! small object-safe value that encapsulates the **instance policy**
//! (how consumers receive instances of the capability) and can
//! duplicate itself per resolved node.
//!
//! Two sides:
//!
//! - [`SharedCapabilityFactory`] — `Send`, produces `Box<dyn shared::Trait>`.
//! - [`LocalCapabilityFactory`]  — `!Send`, produces `Rc<dyn local::Trait>`.
//!
//! Two built-in policies per side:
//!
//! - `ClonePerConsumer*Factory` — clones a stored prototype per consumer.
//! - `FreshPerConsumer*Factory` — invokes a stored closure per consumer.

use super::Error;
use std::any::Any;
use std::marker::PhantomData;
use std::rc::Rc;

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
///   downcast at the [`Capabilities`](super::Capabilities) boundary.
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
/// [arch]: ../../../../docs/extension-system-architecture.md
#[doc(hidden)]
pub trait SharedCapabilityFactory: Send {
    /// Duplicate this factory. Each resolved per-node entry owns its
    /// own `Box<dyn SharedCapabilityFactory>` produced via this method.
    fn clone_box(&self) -> Box<dyn SharedCapabilityFactory>;

    /// Produce a fresh shared trait object for a consumer. The returned
    /// `Box<dyn Any + Send>` contains `Box<dyn shared::Trait>` — the
    /// `require_shared` code downcasts to recover the concrete trait.
    ///
    /// # Why two `Box`es?
    ///
    /// The registry stores factories for many different capabilities in
    /// one `HashMap`, so the factory trait must be object-safe and its
    /// produce method must return a single uniform type — hence
    /// [`Box<dyn Any + Send>`]. But `Any` requires its contents to be
    /// `Sized`, and `dyn shared::Trait` is `!Sized`. Wrapping the trait
    /// object in an inner `Box<dyn shared::Trait>` makes it `Sized` (a
    /// fat pointer) so it can ride inside the `Any` envelope. The
    /// outer `Box` is the uniform storage; the inner `Box` owns the
    /// heap-allocated trait object.
    fn produce_any(&self) -> Box<dyn Any + Send>;

    /// Produce a fresh shared instance and wrap it as a local trait
    /// object, returning `Rc<Rc<dyn local::Trait>>` type-erased to
    /// `Rc<dyn Any>`. Used by [`resolve_bindings`](super::resolve_bindings)
    /// to populate the per-node local slot when an extension provides
    /// only the shared variant (`SharedAsLocal` fallback).
    ///
    /// This method exists on the factory — not on
    /// [`ExtensionCapability`] — because the factory knows both the
    /// capability type and the concrete `Shared` type at
    /// monomorphization time. That lets the impl call
    /// [`ExtensionCapability::wrap_shared_as_local`] directly, with no
    /// runtime downcast and therefore no panic path.
    ///
    /// # Implementation contract
    ///
    /// Implementors must call `C::wrap_shared_as_local(produced)` and
    /// wrap the result in an outer `Rc` so the value can be stored as
    /// `Rc<dyn Any>`. The `#[capability]` proc macro will emit the
    /// correct shape; today the only hand-written impls live in engine
    /// crate tests.
    ///
    /// [`ExtensionCapability`]: crate::capability::ExtensionCapability
    /// [`ExtensionCapability::wrap_shared_as_local`]: crate::capability::ExtensionCapability::wrap_shared_as_local
    fn adapt_as_local_any(&self) -> Rc<dyn Any>;
}

/// Downcast a factory's [`produce_any`] output to `Box<C::Shared>`.
///
/// Hides the double-box erasure convention
/// (`Box<Box<dyn Shared>>` stored as `Box<dyn Any + Send>`) that
/// [`SharedCapabilityFactory::produce_any`] uses.
///
/// # Errors
///
/// Returns [`Error::InternalError`] if the factory produced a
/// `Box<dyn Any + Send>` whose inner type is not `Box<C::Shared>`. This
/// indicates a bug in a hand-rolled [`SharedCapabilityFactory`] impl
/// that used the wrong inner box type. Macro-generated impls uphold
/// the convention by construction.
///
/// [`produce_any`]: SharedCapabilityFactory::produce_any
pub(crate) fn downcast_produced<C: crate::capability::ExtensionCapability>(
    factory: &dyn SharedCapabilityFactory,
) -> Result<Box<C::Shared>, Error> {
    factory
        .produce_any()
        .downcast::<Box<C::Shared>>()
        .map(|b| *b)
        .map_err(|_| Error::InternalError {
            message: format!(
                "capability '{}': shared entry type mismatch (internal error)",
                C::name(),
            ),
        })
}

/// A [`SharedCapabilityFactory`] implementing the **ClonePerConsumer**
/// instance policy: a prototype extension value `E` is stored at
/// registration time, and every consumer receives a fresh
/// `Box<dyn C::Shared>` containing an independent clone of `E`.
///
/// Consumers do **not** share state — each holds its own copy.
/// (If a capability needs shared mutable state across consumers, the
/// `E` should internally hold an `Arc`/`Mutex`; `Clone` on `E` then
/// clones the handle, not the state.)
///
/// Symmetric in name with [`ClonePerConsumerLocalFactory`]: the two
/// encode the same registration-time policy (one prototype is stored;
/// each consumer gets a clone of it) on the two ownership shapes
/// (`Rc` local, `Box` shared).
///
/// The `to_shared` fn pointer is the per-`(C, E)` bridge that coerces
/// an owned `E` into a `Box<dyn C::Shared>`; typically
/// `|e| Box::new(e)` when `E: C::Shared`. It captures the coercion at
/// construction so `produce_any` needs no runtime downcast.
#[doc(hidden)]
pub struct ClonePerConsumerSharedFactory<
    C: crate::capability::ExtensionCapability,
    E: Clone + Send + 'static,
> {
    extension: E,
    to_shared: fn(E) -> Box<C::Shared>,
}

impl<C: crate::capability::ExtensionCapability, E: Clone + Send + 'static>
    ClonePerConsumerSharedFactory<C, E>
{
    /// Construct a ClonePerConsumer shared factory. `to_shared` is the
    /// per-`(C, E)` bridge that boxes an owned `E` as `Box<dyn C::Shared>`;
    /// typically `|e| Box::new(e)` when `E: C::Shared`.
    #[must_use]
    pub fn new(extension: E, to_shared: fn(E) -> Box<C::Shared>) -> Self {
        Self {
            extension,
            to_shared,
        }
    }
}

impl<C: crate::capability::ExtensionCapability, E: Clone + Send + 'static> SharedCapabilityFactory
    for ClonePerConsumerSharedFactory<C, E>
{
    fn clone_box(&self) -> Box<dyn SharedCapabilityFactory> {
        Box::new(ClonePerConsumerSharedFactory::<C, E> {
            extension: self.extension.clone(),
            to_shared: self.to_shared,
        })
    }
    fn produce_any(&self) -> Box<dyn Any + Send> {
        // Double-box convention: the inner `Box<C::Shared>` is a fat
        // pointer (Sized) and can ride inside `Box<dyn Any + Send>`.
        let shared: Box<C::Shared> = (self.to_shared)(self.extension.clone());
        Box::new(shared)
    }
    fn adapt_as_local_any(&self) -> Rc<dyn Any> {
        let shared: Box<C::Shared> = (self.to_shared)(self.extension.clone());
        let rc_local = C::wrap_shared_as_local(shared);
        Rc::new(rc_local)
    }
}

/// A [`SharedCapabilityFactory`] implementing the **FreshPerConsumer**
/// instance policy: a closure `F` is stored at registration time, and
/// every consumer receives a freshly-constructed `Box<dyn C::Shared>`
/// by invoking the closure.
///
/// Counterpart to [`ClonePerConsumerSharedFactory`]: where the Clone
/// variant clones a prototype per consumer, this variant constructs a
/// new instance per consumer. Useful when the extension has no
/// meaningful `Clone` (or cloning would be semantically wrong — e.g.,
/// buffers, file handles) but can be built on demand from captured
/// configuration.
///
/// `F: Clone` is required so [`clone_box`](Self::clone_box) can hand
/// per-node factories their own copy of the closure. Typical closures
/// capture `Clone` configuration (`Arc<Config>`, `String`, etc.) and
/// are naturally `Clone`.
#[doc(hidden)]
pub struct FreshPerConsumerSharedFactory<C: crate::capability::ExtensionCapability, F>
where
    F: Fn() -> Box<C::Shared> + Send + Clone + 'static,
{
    produce: F,
    _phantom: PhantomData<fn() -> C>,
}

impl<C: crate::capability::ExtensionCapability, F> FreshPerConsumerSharedFactory<C, F>
where
    F: Fn() -> Box<C::Shared> + Send + Clone + 'static,
{
    /// Construct a FreshPerConsumer shared factory from a closure that
    /// produces a fresh `Box<dyn C::Shared>` on each call.
    #[must_use]
    pub fn new(produce: F) -> Self {
        Self {
            produce,
            _phantom: PhantomData,
        }
    }
}

impl<C: crate::capability::ExtensionCapability, F> SharedCapabilityFactory
    for FreshPerConsumerSharedFactory<C, F>
where
    F: Fn() -> Box<C::Shared> + Send + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn SharedCapabilityFactory> {
        Box::new(FreshPerConsumerSharedFactory::<C, F> {
            produce: self.produce.clone(),
            _phantom: PhantomData,
        })
    }
    fn produce_any(&self) -> Box<dyn Any + Send> {
        let shared: Box<C::Shared> = (self.produce)();
        Box::new(shared)
    }
    fn adapt_as_local_any(&self) -> Rc<dyn Any> {
        let shared: Box<C::Shared> = (self.produce)();
        let rc_local = C::wrap_shared_as_local(shared);
        Rc::new(rc_local)
    }
}

// ── Local-capability factory trait ───────────────────────────────────────────

/// Object-safe factory for the local variant of a capability.
///
/// The local-side mirror of [`SharedCapabilityFactory`]. One concrete
/// impl exists per `(capability, extension)` registration. The impl
/// encodes the **instance policy** — how the registry supplies
/// instances when consumers call
/// [`Capabilities::require_local`](super::Capabilities::require_local):
///
/// - [`ClonePerConsumerLocalFactory`] — all consumers receive an
///   `Rc::clone` of a single cached instance (**ClonePerConsumer**
///   policy; "clone" here is `Rc::clone`, so consumers observe a
///   shared underlying object, mirroring the shared-side policy of
///   handing out a clone per consumer).
/// - [`FreshPerConsumerLocalFactory`] — each consumer receives a
///   freshly-constructed instance via a stored closure
///   (**FreshPerConsumer** policy). Consumers observe independent
///   underlying trait objects; interior mutability is **not** shared.
///
/// The trait is object-safe: entries store `Box<dyn
/// LocalCapabilityFactory>` so multiple policies can coexist in one
/// registry. Not `Send`: local factories hold `Rc`s and must never
/// cross thread boundaries.
#[doc(hidden)]
pub trait LocalCapabilityFactory {
    /// Duplicate this factory. Each resolved per-node entry owns its
    /// own `Box<dyn LocalCapabilityFactory>` produced via this method.
    /// Policies that cache an underlying `Rc` share it across clones
    /// so every per-node clone points at the same instance.
    fn clone_box(&self) -> Box<dyn LocalCapabilityFactory>;

    /// Produce an `Rc<dyn local::Trait>` for a consumer, type-erased
    /// as `Rc<dyn Any>`.
    /// [`Capabilities::require_local`](super::Capabilities::require_local)
    /// downcasts to `Rc<C::Local>` to recover the concrete trait.
    /// Whether this returns a cached clone or a freshly-built instance
    /// is a policy-level choice (see the trait docs).
    fn produce_any(&self) -> Rc<dyn Any>;
}

/// A [`LocalCapabilityFactory`] implementing the **ClonePerConsumer**
/// instance policy: every consumer receives an `Rc::clone` of a single
/// stored `Rc<dyn local::Trait>`.
///
/// Note on semantics: because cloning here is `Rc::clone` (not a deep
/// data clone), consumers share the **same** underlying trait object —
/// any interior mutability (`RefCell`, `Cell`, …) is visible across
/// consumers. This matches the behavior of the pre-factory local
/// registration and is symmetrical in *name* with the shared-side
/// ClonePerConsumer policy, which hands out independently-cloned
/// `Box<dyn shared::Trait>` instances. If fully-independent local
/// instances are needed, use [`FreshPerConsumerLocalFactory`] instead.
///
/// The `instance` field stores an `Rc<dyn Any>` whose inner type is
/// `Rc<dyn local::Trait>`. The double-`Rc` is the local-side analogue
/// of [`SharedCapabilityFactory`]'s double-`Box` convention: the outer
/// `Rc` is the type-erased container; the inner `Rc<dyn Trait>` is
/// what `require_local` downcasts to and clones out.
#[doc(hidden)]
pub struct ClonePerConsumerLocalFactory {
    instance: Rc<dyn Any>,
}

impl ClonePerConsumerLocalFactory {
    /// Wrap a type-erased `Rc<dyn Any>` (whose inner value must be
    /// `Rc<dyn local::Trait>`) as a ClonePerConsumer local factory.
    #[must_use]
    pub fn new(instance: Rc<dyn Any>) -> Self {
        Self { instance }
    }
}

impl LocalCapabilityFactory for ClonePerConsumerLocalFactory {
    fn clone_box(&self) -> Box<dyn LocalCapabilityFactory> {
        Box::new(ClonePerConsumerLocalFactory {
            instance: Rc::clone(&self.instance),
        })
    }
    fn produce_any(&self) -> Rc<dyn Any> {
        Rc::clone(&self.instance)
    }
}

/// A [`LocalCapabilityFactory`] implementing the **FreshPerConsumer**
/// instance policy: a closure `F` is stored at registration time, and
/// every consumer receives a freshly-constructed `Rc<dyn Any>` (whose
/// inner type must be `Rc<dyn local::Trait>`) by invoking the closure.
///
/// Counterpart to [`ClonePerConsumerLocalFactory`]: where the Clone
/// variant shares one cached `Rc` across all consumers, this variant
/// constructs a new instance per consumer. Consumers observe
/// **independent** underlying trait objects — interior mutability is
/// not shared.
///
/// `F: Clone` is required so [`clone_box`](Self::clone_box) can hand
/// per-node factories their own copy of the closure.
///
/// The closure is responsible for producing the double-`Rc` shape
/// `Rc<dyn Any>` containing `Rc<dyn local::Trait>` — the same
/// convention as the stored form of
/// [`ClonePerConsumerLocalFactory::instance`]. The extension-builder
/// layer handles this wrapping when it constructs the factory.
#[doc(hidden)]
pub struct FreshPerConsumerLocalFactory<F>
where
    F: Fn() -> Rc<dyn Any> + Clone + 'static,
{
    produce: F,
}

impl<F> FreshPerConsumerLocalFactory<F>
where
    F: Fn() -> Rc<dyn Any> + Clone + 'static,
{
    /// Construct a FreshPerConsumer local factory from a closure that
    /// produces `Rc<dyn Any>` wrapping `Rc<dyn local::Trait>` on each
    /// call.
    #[must_use]
    pub fn new(produce: F) -> Self {
        Self { produce }
    }
}

impl<F> LocalCapabilityFactory for FreshPerConsumerLocalFactory<F>
where
    F: Fn() -> Rc<dyn Any> + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn LocalCapabilityFactory> {
        Box::new(FreshPerConsumerLocalFactory {
            produce: self.produce.clone(),
        })
    }
    fn produce_any(&self) -> Rc<dyn Any> {
        (self.produce)()
    }
}
