// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability system for extensions.
//!
//! This module defines the type-safe capability resolution infrastructure.
//! Extensions register capabilities via [`ExtensionCapabilities`], and
//! node factories consume them via [`registry::Capabilities`].
//!
//! Capability traits are defined per-capability in submodules (e.g.,
//! `bearer_token_provider`), with local (!Send) and shared (Send) variants
//! re-exported from [`local::capability`](crate::local::capability) and
//! [`shared::capability`](crate::shared::capability).

pub mod registry;

// ── Sealed ExtensionCapability trait ─────────────────────────────────────────

/// Sealing module — prevents external crates from implementing
/// [`ExtensionCapability`].
mod private {
    /// Sealed marker trait. Only the `#[capability]` proc macro (or
    /// hand-written impls inside the engine crate) can implement this.
    pub trait Sealed {}
}

/// Compile-time–sealed trait binding a capability registration struct
/// to its local and shared trait object types.
///
/// Each capability (e.g., `BearerTokenProvider`) has a zero-sized
/// registration struct that implements this trait. The associated types
/// tell [`Capabilities::require_local`](registry::Capabilities::require_local)
/// and [`Capabilities::require_shared`](registry::Capabilities::require_shared)
/// which concrete `dyn Trait` to downcast to.
///
/// This trait is sealed — only the engine crate (via the
/// `#[capability]` proc macro or manual impls) can add new
/// capabilities.
pub trait ExtensionCapability: private::Sealed + 'static {
    /// Capability name as a const, usable in static contexts.
    /// Must match the `name` argument in the `#[capability]` attribute.
    const NAME: &'static str;

    /// The local (!Send) trait object type for this capability
    /// (e.g., `dyn local::capability::BearerTokenProvider`).
    type Local: ?Sized + 'static;

    /// The shared (Send) trait object type for this capability
    /// (e.g., `dyn shared::capability::BearerTokenProvider`).
    type Shared: ?Sized + Send + 'static;

    /// Human-readable name used in error messages and config validation.
    #[must_use]
    fn name() -> &'static str {
        Self::NAME
    }

    /// Wraps a freshly-produced shared trait object as a local trait object.
    ///
    /// The `#[capability]` proc macro generates an impl that constructs
    /// the capability's `SharedAsLocal` adapter. Because `local::Trait`
    /// and `shared::Trait` are generated from the same source trait,
    /// the adapter is always constructible — there is no opt-out.
    /// A capability whose local and shared semantics diverge should be
    /// split into two distinct capabilities rather than expressed as an
    /// adapter refusal.
    ///
    /// Authors normally don't implement this by hand; the macro handles
    /// it. If you are hand-rolling an `ExtensionCapability` impl (only
    /// needed inside the engine crate for testing), return
    /// `Rc::new(YourAdapter(shared))`.
    fn wrap_shared_as_local(shared: Box<Self::Shared>) -> std::rc::Rc<Self::Local>;

    /// Creates a local adapter entry from a shared capability factory.
    ///
    /// When a shared-only extension provides this capability, the engine
    /// calls this method to populate the local registry slot via a
    /// `SharedAsLocal` adapter. The returned `Rc<dyn Any>` contains an
    /// `Rc<dyn local::Trait>` backed by the adapter. Every shared
    /// registration is reachable as a local binding — there is no
    /// opt-out.
    ///
    /// The default impl funnels through the internal
    /// `registry::downcast_produced` helper so the
    /// `Box<Box<dyn Shared>> as Box<dyn Any>` erasure convention lives
    /// in one place. Override only for niche cases where you need to
    /// sidestep that helper.
    ///
    /// # Per-node freshness
    ///
    /// The engine invokes this method **once per node** that binds the
    /// capability, calling `factory.produce_any()` inside to obtain a fresh
    /// shared instance for that node. This matches the per-caller-fresh
    /// semantics of [`Capabilities::require_shared`] — an extension
    /// author can rely on each binding receiving its own instance.
    ///
    /// If your shared impl relies on state that must be reset **per call**
    /// rather than per node, declare an explicit `local:` variant via
    /// the `extension_capabilities!` macro instead of relying on this
    /// fallback.
    ///
    /// # Invariants
    ///
    /// The `factory` argument must have been constructed via the
    /// [`TypedSharedFactory`] blanket impl so its `produce_any` output
    /// is `Box<Box<C::Shared>>`. Violating this is a bug in the
    /// registration code and causes a panic (see the `BUG:` expect
    /// message below). Call sites in this crate uphold the invariant
    /// by construction.
    ///
    /// [`Capabilities::require_shared`]: registry::Capabilities::require_shared
    /// [`SharedCapabilityFactory`]: registry::SharedCapabilityFactory
    /// [`TypedSharedFactory`]: registry::TypedSharedFactory
    fn adapt_shared_to_local(
        factory: &dyn registry::SharedCapabilityFactory,
    ) -> std::rc::Rc<dyn std::any::Any>
    where
        Self: Sized,
    {
        let shared = registry::downcast_produced::<Self>(factory)
            .expect("BUG: SharedCapabilityFactory produced wrong inner type; see downcast_produced docs");
        let rc_local: std::rc::Rc<Self::Local> = Self::wrap_shared_as_local(shared);
        std::rc::Rc::new(rc_local)
    }
}

/// Re-export for use by the `#[capability]` proc macro's generated code.
/// `pub(crate)` (not `pub`) preserves the seal: the macro only expands
/// inside this crate, so external crates still can't reach `Sealed` to
/// forge an `ExtensionCapability` impl.
#[doc(hidden)]
#[allow(unused_imports)] // used by future `#[capability]` invocations
pub(crate) use private::Sealed as CapabilitySealed;

// ── KNOWN_CAPABILITIES (link-time registration) ──────────────────────────────

/// A link-time–registered capability descriptor.
///
/// Each `#[capability]` invocation produces a static entry in the
/// [`KNOWN_CAPABILITIES`] distributed slice. The engine uses this at
/// config validation time to map string names to `TypeId`s and adapter
/// functions.
#[doc(hidden)]
pub struct KnownCapability {
    /// Human-readable name (e.g., `"bearer_token_provider"`).
    pub name: &'static str,
    /// Short description of what the capability does. Authored at the
    /// `#[capability(description = "...")]` site.
    ///
    /// TODO(extension-system): not yet read by any consumer.
    pub description: &'static str,
    /// `TypeId` of the zero-sized registration struct.
    pub type_id: fn() -> std::any::TypeId,
    /// Adapter function for shared→local fallback.
    pub adapt_shared_to_local: registry::SharedAsLocalAdaptFn,
}

/// Link-time registry of all capabilities defined in the binary.
///
/// Populated by `#[capability]` proc macro entries. Used by
/// `resolve_bindings()` to validate capability names and retrieve
/// `TypeId`s and adapter functions.
//
// `linkme::distributed_slice` requires a `pub static`; `#[doc(hidden)]`
// excludes it from generated rustdoc so external crates don't see it in
// the public API surface.
#[doc(hidden)]
#[allow(unsafe_code)]
#[linkme::distributed_slice]
pub static KNOWN_CAPABILITIES: [KnownCapability] = [..];

// ── ExtensionCapabilities (factory metadata) ─────────────────────────────────

/// Static metadata describing which capabilities an extension factory provides.
///
/// Carried on [`ExtensionFactory`](crate::ExtensionFactory) and used by:
/// - Config validation: checking that capability bindings reference
///   capabilities the extension actually provides.
/// - `resolve_bindings()`: knowing which registry slots to populate.
///
/// Constructed via the `extension_capabilities!` macro.
pub struct ExtensionCapabilities {
    /// Capability names provided by the **shared** variant.
    pub shared: &'static [&'static str],
    /// Capability names provided by the **local** variant.
    pub local: &'static [&'static str],
}

impl ExtensionCapabilities {
    /// No capabilities — used by extensions that only have a lifecycle
    /// (active) but don't expose any capabilities to nodes.
    #[must_use]
    pub const fn none() -> Self {
        ExtensionCapabilities {
            shared: &[],
            local: &[],
        }
    }
}

/// Declares which capabilities an extension provides.
///
/// Three forms:
///
/// ```rust,ignore
/// // Shared-only (with automatic local fallback via SharedAsLocal)
/// extension_capabilities!(shared: MyExt => [BearerTokenProvider, KeyValueStore])
///
/// // Local-only
/// extension_capabilities!(local: MyLocalExt => [KeyValueStore])
///
/// // Dual — different types for local and shared, same capability
/// extension_capabilities!(
///     shared: MySharedKvStore => [KeyValueStore],
///     local: MyLocalKvStore => [KeyValueStore],
/// )
/// ```
///
/// The extension type (`$ext`) is matched by the macro but currently discarded.
//
// TODO(extension-system): `$ext` (and `$sext` / `$lext`) will carry the
// extension instance type used by generated registration closures on
// `ExtensionCapabilities` — a compile-time `assert_*_impl::<$ext>()` check
// that every listed capability is actually implemented, and a runtime
// downcast target inside `register_shared` / `register_local` fn pointers
// that the engine invokes during the build phase with the extension
// erased as `&dyn Any` / `Rc<dyn Any>`.
#[macro_export]
macro_rules! extension_capabilities {
    // Shared-only extension (automatic local fallback via SharedAsLocal).
    (shared: $ext:ty => [$($cap:ty),+ $(,)?]) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
            local: &[],
        }
    };
    // Local-only extension.
    (local: $ext:ty => [$($cap:ty),+ $(,)?]) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[],
            local: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
        }
    };
    // Dual extension — different types for shared and local.
    (shared: $sext:ty => [$($scap:ty),+ $(,)?], local: $lext:ty => [$($lcap:ty),+ $(,)?] $(,)?) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[$(<$scap as $crate::capability::ExtensionCapability>::NAME),+],
            local: &[$(<$lcap as $crate::capability::ExtensionCapability>::NAME),+],
        }
    };
}
