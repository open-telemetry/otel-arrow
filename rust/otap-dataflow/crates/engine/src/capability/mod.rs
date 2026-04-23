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

pub mod factory;
pub mod registry;

pub use factory::{LocalInstanceFactory, SharedInstanceFactory};

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
    /// Called from each capability entry's
    /// `adapt_as_local` fn pointer when a shared-only extension is
    /// consumed via `require_local` / the `SharedAsLocal` fallback
    /// path in `resolve_bindings`.
    ///
    /// # Per-node freshness
    ///
    /// Invoked **once per node** that binds the capability via the
    /// fallback path, with a fresh `Box<Self::Shared>` minted by the
    /// factory for that node. This matches the per-caller-fresh
    /// semantics of [`Capabilities::require_shared`] — every binding
    /// gets its own instance.
    ///
    /// If your shared impl relies on state that must be reset **per call**
    /// rather than per node, declare an explicit `local:` variant via
    /// the `extension_capabilities!` macro instead of relying on this
    /// fallback.
    ///
    /// Authors normally don't implement this by hand; the macro handles
    /// it. If you are hand-rolling an `ExtensionCapability` impl (only
    /// needed inside the engine crate for testing), return
    /// `Rc::new(YourAdapter(shared))`.
    ///
    /// [`Capabilities::require_shared`]: registry::Capabilities::require_shared
    fn wrap_shared_as_local(shared: Box<Self::Shared>) -> std::rc::Rc<Self::Local>;
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
/// config validation time to map string names to `TypeId`s.
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
}

/// Link-time registry of all capabilities defined in the binary.
///
/// Populated by `#[capability]` proc macro entries. Used by
/// `resolve_bindings()` to validate capability names and retrieve
/// `TypeId`s.
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
///
/// The `register_shared` / `register_local` fn pointers are the bridge
/// between the extension's type-erased instance factories and the
/// capability registry. The engine invokes them at bundle-registration
/// time, passing the extension's `ExtensionId` plus a clone of the
/// appropriate `*InstanceFactory`. The fn pointer internally builds one
/// [`SharedCapabilityEntry`](registry::SharedCapabilityEntry) per
/// listed capability and inserts it into the registry.
#[derive(Clone)]
pub struct ExtensionCapabilities {
    /// Capability names provided by the **shared** variant.
    pub shared: &'static [&'static str],
    /// Capability names provided by the **local** variant.
    pub local: &'static [&'static str],
    /// Register all shared-variant capabilities into the registry.
    /// No-op when `shared` is empty.
    pub register_shared: fn(
        ext_id: otap_df_config::ExtensionId,
        factory: SharedInstanceFactory,
        registry: &mut registry::CapabilityRegistry,
    ) -> Result<(), registry::Error>,
    /// Register all local-variant capabilities into the registry.
    /// No-op when `local` is empty.
    pub register_local: fn(
        ext_id: otap_df_config::ExtensionId,
        factory: LocalInstanceFactory,
        registry: &mut registry::CapabilityRegistry,
    ) -> Result<(), registry::Error>,
}

impl ExtensionCapabilities {
    /// No-op `register_shared` fn pointer. Used by the
    /// `extension_capabilities!` macro arms that don't provide a shared
    /// variant, and by [`ExtensionCapabilities::none`].
    #[doc(hidden)]
    pub const NOOP_REGISTER_SHARED: fn(
        otap_df_config::ExtensionId,
        SharedInstanceFactory,
        &mut registry::CapabilityRegistry,
    ) -> Result<(), registry::Error> = |_, _, _| Ok(());

    /// No-op `register_local` fn pointer. Counterpart of
    /// [`NOOP_REGISTER_SHARED`](Self::NOOP_REGISTER_SHARED).
    #[doc(hidden)]
    pub const NOOP_REGISTER_LOCAL: fn(
        otap_df_config::ExtensionId,
        LocalInstanceFactory,
        &mut registry::CapabilityRegistry,
    ) -> Result<(), registry::Error> = |_, _, _| Ok(());

    /// No capabilities — used by extensions that only have a lifecycle
    /// (active) but don't expose any capabilities to nodes.
    #[must_use]
    pub const fn none() -> Self {
        ExtensionCapabilities {
            shared: &[],
            local: &[],
            register_shared: Self::NOOP_REGISTER_SHARED,
            register_local: Self::NOOP_REGISTER_LOCAL,
        }
    }
}

/// Declares which capabilities an extension provides.
///
/// The left side names the extension type(s); the right side is a single
/// capability list that applies to both sides (no per-side divergence).
/// Three forms:
///
/// ```rust,ignore
/// // Shared-only (local consumers served via SharedAsLocal fallback).
/// extension_capabilities!(shared: MyExt => [BearerTokenProvider, KeyValueStore]);
///
/// // Local-only.
/// extension_capabilities!(local: MyLocalExt => [KeyValueStore]);
///
/// // Dual-type — distinct shared/local types, same capability list.
/// extension_capabilities!(
///     (shared: MySharedKv, local: MyLocalKv) => [KeyValueStore]
/// );
/// ```
///
/// Each capability `$cap` in the list must have a `#[capability]`-generated
/// `shared_entry::<E>` and/or `local_entry::<E>` associated fn. The macro
/// invokes them per listed capability, passing a clone of the extension's
/// instance factory, and inserts the result into the registry.
///
/// In the dual form, `S` must implement `shared::$cap` and `L` must
/// implement `local::$cap` for every capability in the list — mismatches
/// surface as standard trait-bound errors at the macro call site.
#[macro_export]
macro_rules! extension_capabilities {
    // Shared-only extension (automatic local fallback via SharedAsLocal).
    (shared: $ext:ty => [$($cap:ty),+ $(,)?]) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
            local: &[],
            register_shared: |ext_id, factory, registry| {
                $(
                    registry.register_shared(
                        ::std::any::TypeId::of::<$cap>(),
                        <$cap>::shared_entry::<$ext>(ext_id.clone(), factory.clone()),
                    )?;
                )+
                Ok(())
            },
            register_local: $crate::capability::ExtensionCapabilities::NOOP_REGISTER_LOCAL,
        }
    };
    // Local-only extension.
    (local: $ext:ty => [$($cap:ty),+ $(,)?]) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[],
            local: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
            register_shared: $crate::capability::ExtensionCapabilities::NOOP_REGISTER_SHARED,
            register_local: |ext_id, factory, registry| {
                $(
                    registry.register_local(
                        ::std::any::TypeId::of::<$cap>(),
                        <$cap>::local_entry::<$ext>(ext_id.clone(), factory.clone()),
                    )?;
                )+
                Ok(())
            },
        }
    };
    // Dual-type extension — distinct shared/local types, same capability list.
    ((shared: $sext:ty, local: $lext:ty) => [$($cap:ty),+ $(,)?]) => {
        $crate::capability::ExtensionCapabilities {
            shared: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
            local: &[$(<$cap as $crate::capability::ExtensionCapability>::NAME),+],
            register_shared: |ext_id, factory, registry| {
                $(
                    registry.register_shared(
                        ::std::any::TypeId::of::<$cap>(),
                        <$cap>::shared_entry::<$sext>(ext_id.clone(), factory.clone()),
                    )?;
                )+
                Ok(())
            },
            register_local: |ext_id, factory, registry| {
                $(
                    registry.register_local(
                        ::std::any::TypeId::of::<$cap>(),
                        <$cap>::local_entry::<$lext>(ext_id.clone(), factory.clone()),
                    )?;
                )+
                Ok(())
            },
        }
    };
}

#[cfg(test)]
mod tests;
