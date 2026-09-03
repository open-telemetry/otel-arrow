// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `BasicAuthProvider` capability.
//!
//! A small, purpose-built capability that hands out credentials to data-path
//! nodes. It is intentionally provider- and execution-model agnostic: the same
//! trait serves active and passive providers across both the shared and local
//! execution models. Consumers depend only on the two methods below, never on
//! how credentials are produced or refreshed.
//!
//! The `#[capability]` proc macro expands the trait into:
//!
//! - A `pub(crate) mod local` containing the `!Send` `BasicAuthProvider` trait variant
//! - A `pub(crate) mod shared` containing the `Send + Sync` `BasicAuthProvider` trait variant
//! - A `SharedAsLocalBasicAuthProvider` adapter
//! - A zero-sized `pub struct BasicAuthProvider` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use crate::capability::auth::BasicAuthCredential;
use crate::capability::error::CapabilityError;
use futures::Stream;
use otel_arrow_dfe_engine_macros::capability;
use std::pin::Pin;
use std::time::Duration;

/// How close to [`BasicAuthCredential::expires_on`] a credential stops being usable.
///
/// Part of the capability contract rather than either side's private tuning,
/// because both sides have to agree on it:
///
/// - A **provider** must not serve a credential inside this window, and must
///   schedule its refresh far enough ahead of expiry to publish a replacement
///   before the current one enters it. A provider whose refresh lead time is
///   smaller than this margin strands its consumers: the credential it is still
///   serving has already stopped being usable.
/// - A **consumer** must stop sending requests once its cached credential is
///   inside this window, so a request cannot outlive the credential it carries
///   while in flight, in the presence of clock skew between the consumer, the
///   Acredential issuer and the service.
///
/// Fixed rather than configurable so a provider can validate its own refresh
/// settings against the same value every consumer enforces. It has to cover a
/// request's own duration plus that clock skew; 30s matches the default credential
/// endpoint timeout.
pub const BASIC_AUTH_CREDENTIAL_USABLE_MARGIN: Duration = Duration::from_secs(30);

/// A per-consumer subscription to Basic Auth credential refreshes.
///
/// The item is a plain [`BasicAuthCredential`], not a `Result`: a refresh failure does not
/// terminate the subscription. The stream simply does not emit until the next
/// successful refresh, and failures surface via [`BasicAuthProvider::get_credential`]
/// and telemetry instead. Because the item is [`Clone`], a provider can fan one
/// refreshed credential out to all subscribers via a `watch`/`broadcast` channel.
///
/// Boxed to hide the concrete stream type so providers can back it differently
/// (e.g. a `watch` channel or an `unfold`) without changing the signature. The
/// `Send` bound is intentionally omitted: the subscription is always consumed
/// on the core that created it (thread-per-core), so it need not be `Send`. The
/// `#[capability]` macro emits this signature into both the `local` (`?Send`)
/// and `shared` (`Send + Sync`) trait variants unchanged.
pub type BasicAuthCredentialStream = Pin<Box<dyn Stream<Item = BasicAuthCredential> + 'static>>;

/// Hands out BasicAuthCredential to data-path nodes.
#[capability(
    name = "basic_auth_provider",
    description = "Provides Basic Auth credentials, refreshed in the background"
)]
pub trait BasicAuthProvider {
    /// Returns the current valid credential for the provider's configured
    /// scope(s).
    ///
    /// The fast path reads a cached credential; on a cache miss the provider
    /// performs a lookup operation. A provider that shares its cache and
    /// refresh state across cloned instances can coalesce concurrent misses
    /// into a single call -- but that is a provider implementation detail, not
    /// a guarantee of this trait. Returns a [`CapabilityError`] if no valid
    /// credential can be produced.
    ///
    /// The credential is scoped to the resource(s) the provider was configured
    /// for. There is no wiring-time check that a consumer's target resource
    /// matches the provider's scope, so a mismatch surfaces at the service as
    /// an auth failure (e.g. HTTP 401) rather than at startup. Consumers must
    /// bind to a provider configured for their resource.
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError>;

    /// Subscribes to the stream of credential refreshes.
    ///
    /// Yields each newly published credential for the lifetime of the
    /// extension; each call returns an independent subscription. The stream
    /// does not carry errors: a failed refresh does not end the subscription,
    /// and the next successful refresh still yields a credential (see
    /// [`BasicAuthCredentialStream`]).
    ///
    /// # Contract
    ///
    /// A subscription created *after* a credential has already been published
    /// MUST immediately yield the current credential rather than block until
    /// the next refresh. This lets a consumer subscribe at any point (for
    /// example after the provider's readiness gate has fired) and obtain a
    /// usable credential without a separate
    /// [`get_credential`](Self::get_credential) call, avoiding a race between
    /// reading the current token and subscribing to updates. A
    /// `tokio::sync::watch`-backed implementation satisfies this naturally,
    /// since a fresh receiver observes the channel's current value on its first
    /// poll.
    fn credential_stream(&self) -> BasicAuthCredentialStream;
}
