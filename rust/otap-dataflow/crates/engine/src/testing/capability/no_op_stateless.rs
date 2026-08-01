// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `NoOpStateless` test capability.
//!
//! A reference capability with no internal state, intended for end-to-end
//! tests of the extension/capability wiring (registration, resolution,
//! `SharedAsLocal` fallback, builder APIs). Methods cover the relevant
//! codegen permutations: `&self` x {sync, async} x {with-args, no-args} x
//! {borrowed return, owned return}.
//!
//! The `#[capability]` proc macro expands the trait below into:
//!
//! - A `pub(crate) mod local` containing the `!Send` `NoOpStateless` trait variant
//! - A `pub(crate) mod shared` containing the `Send` `NoOpStateless` trait variant
//! - A `SharedAsLocalNoOpStateless` adapter
//! - A zero-sized `pub struct NoOpStateless` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use otap_df_engine_macros::capability;

/// No-op test capability with no internal state.
///
/// All methods are pure: they never mutate `self`, and reference
/// implementations can be plain value types with no interior
/// mutability. Used by tests to exercise the registration and
/// resolution paths without entangling them with state semantics.
#[capability(
    name = "no_op_stateless",
    description = "No-op test capability with no internal state"
)]
pub trait NoOpStateless {
    /// Returns a stable identifier for this capability instance.
    ///
    /// Borrows from `self`, exercising the lifetime-elision codegen
    /// path on both the local and shared trait variants.
    fn name(&self) -> &str;

    /// Echoes a primitive value synchronously.
    ///
    /// Exercises the sync-method path with a `Copy` argument and a
    /// `Copy` return.
    fn echo(&self, value: u64) -> u64;

    /// Returns a fixed token asynchronously.
    ///
    /// Exercises the async-method path with no arguments. The
    /// `#[capability]` macro wires `#[async_trait]` automatically
    /// for both the local (`?Send`) and shared trait variants.
    async fn ping(&self) -> u64;

    /// Echoes an owned value asynchronously.
    ///
    /// Exercises the async-method path with an owned (non-`Copy`)
    /// argument and an owned return.
    async fn echo_async(&self, value: String) -> String;
}

// The `#[capability]` macro emits the `local`/`shared` trait modules as
// `pub(crate)`, so downstream test crates reach the variants through these
// public aliases (a single surface) rather than the private submodule paths.

/// The `!Send` (local) variant of the [`NoOpStateless`] capability trait.
pub use local::NoOpStateless as LocalNoOpStateless;
/// The `Send` (shared) variant of the [`NoOpStateless`] capability trait.
pub use shared::NoOpStateless as SharedNoOpStateless;
