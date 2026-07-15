// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `NoOpStateful` test capability.
//!
//! A reference capability that carries per-instance mutable state,
//! intended for end-to-end tests that need to verify the extension
//! wiring respects `&mut self` correctly (both natively and through
//! the `SharedAsLocal` adapter), and that per-consumer instance
//! isolation works as advertised.
//!
//! Together with [`NoOpStateless`](super::no_op_stateless::NoOpStateless)
//! these two capabilities cover the matrix of receiver shapes
//! (`&self` / `&mut self`), method modes (sync / async), and argument
//! / return shapes that the `#[capability]` proc macro must support.
//!
//! The `#[capability]` proc macro expands the trait below into:
//!
//! - `pub(crate) mod local::NoOpStateful` (`!Send` trait variant)
//! - `pub(crate) mod shared::NoOpStateful` (`Send` trait variant)
//! - A `SharedAsLocalNoOpStateful` adapter (auto-delegates `&mut self`)
//! - A zero-sized `pub struct NoOpStateful` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use otap_df_engine_macros::capability;

/// No-op test capability with per-instance mutable state.
///
/// Reference implementations track simple counters / last-recorded
/// values so tests can assert that mutations on one consumer's
/// instance do not leak into another's (per-consumer `Box`-clone
/// ownership semantics).
#[capability(
    name = "no_op_stateful",
    description = "No-op test capability with per-instance mutable state"
)]
pub trait NoOpStateful {
    /// Number of times [`increment`](Self::increment) has been called
    /// on this instance since construction or the last
    /// [`reset`](Self::reset).
    ///
    /// Exercises a `&self` read against state that was last updated
    /// by a `&mut self` method.
    fn count(&self) -> u64;

    /// Increments the internal counter by one and returns the new
    /// value.
    ///
    /// Exercises the sync `&mut self` codegen path on both the local
    /// and shared trait variants, and exercises `SharedAsLocal`
    /// `&mut self` delegation through the adapter's
    /// `Box<dyn shared::NoOpStateful>` field.
    fn increment(&mut self) -> u64;

    /// Clears all mutable state to its initial value (counter back
    /// to zero, last-recorded back to `None`).
    ///
    /// Exercises a sync `&mut self` method with no return value.
    fn reset(&mut self);

    /// Records a value asynchronously and returns the previous
    /// `count()` plus the recorded value (a synthetic acknowledgment
    /// chosen to make the call observable in tests).
    ///
    /// Exercises the async `&mut self` codegen path with an owned
    /// argument and an owned return.
    async fn record(&mut self, value: u64) -> u64;

    /// Returns the value of the most recent [`record`](Self::record)
    /// call on this instance, or `None` if `record` has never been
    /// called (or the instance has been reset since).
    ///
    /// Exercises a `&self` read returning an owned `Option<_>`.
    fn last_recorded(&self) -> Option<u64>;
}

// The `#[capability]` macro emits the `local`/`shared` trait modules as
// `pub(crate)`, so downstream test crates reach the variants through these
// public aliases (a single surface) rather than the private submodule paths.

/// The `!Send` (local) variant of the [`NoOpStateful`] capability trait.
pub use local::NoOpStateful as LocalNoOpStateful;
/// The `Send` (shared) variant of the [`NoOpStateful`] capability trait.
pub use shared::NoOpStateful as SharedNoOpStateful;
