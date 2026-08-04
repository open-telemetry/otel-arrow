// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `BearerTokenProvider` capability.
//!
//! A small, purpose-built capability that hands out OAuth bearer tokens
//! to data-path nodes. It is intentionally provider- and execution-model
//! agnostic: the same trait serves active and passive providers across
//! both the shared and local execution models. Consumers depend only on
//! the two methods below, never on how a token is produced or refreshed.
//!
//! The `#[capability]` proc macro expands the trait into:
//!
//! - A `pub(crate) mod local` containing the `!Send` `BearerTokenProvider` trait variant
//! - A `pub(crate) mod shared` containing the `Send` `BearerTokenProvider` trait variant
//! - A `SharedAsLocalBearerTokenProvider` adapter
//! - A zero-sized `pub struct BearerTokenProvider` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use super::BearerToken;
use crate::capability::error::CapabilityError;
use futures::Stream;
use otap_df_engine_macros::capability;
use std::pin::Pin;

/// A per-consumer subscription to token refreshes.
///
/// The item is a plain [`BearerToken`], not a `Result`: a refresh
/// failure does not terminate the subscription. The stream simply does
/// not emit until the next successful refresh, and failures surface via
/// [`BearerTokenProvider::get_token`] and telemetry instead. Because the
/// item is [`Clone`], a provider can fan one refreshed token out to all
/// subscribers via a `watch`/`broadcast` channel.
///
/// Boxed to hide the concrete stream type so providers can back it
/// differently (e.g. a `watch` channel or an `unfold`) without changing
/// the signature. The `Send` bound is intentionally omitted: the
/// subscription is always consumed on the core that created it
/// (thread-per-core), so it need not be `Send`. The `#[capability]`
/// macro emits this signature into both the `local` (`?Send`) and
/// `shared` (`Send`) trait variants unchanged.
pub type TokenStream = Pin<Box<dyn Stream<Item = BearerToken> + 'static>>;

/// Hands out OAuth bearer tokens to data-path nodes.
#[capability(
    name = "bearer_token_provider",
    description = "Provides OAuth bearer tokens, refreshed in the background"
)]
pub trait BearerTokenProvider {
    /// Returns the current valid token for the provider's configured
    /// scope(s).
    ///
    /// The fast path reads a cached token; on a cache miss the provider
    /// performs a credential call. A provider that shares its cache and
    /// refresh state across cloned instances can coalesce concurrent
    /// misses into a single call -- but that is a provider implementation
    /// detail, not a guarantee of this trait. Returns a
    /// [`CapabilityError`] if no valid token can be produced.
    ///
    /// The token is scoped to the resource(s) the provider was configured
    /// for. There is no wiring-time check that a consumer's target
    /// resource matches the provider's scope, so a mismatch surfaces at
    /// the service as an auth failure (e.g. HTTP 401) rather than at
    /// startup. Consumers must bind to a provider configured for their
    /// resource.
    async fn get_token(&self) -> Result<BearerToken, CapabilityError>;

    /// Subscribes to the stream of token refreshes.
    ///
    /// Yields each newly published token for the lifetime of the
    /// extension; each call returns an independent subscription. The
    /// stream does not carry errors: a failed refresh does not end the
    /// subscription, and the next successful refresh still yields a token
    /// (see [`TokenStream`]).
    ///
    /// # Contract
    ///
    /// A subscription created *after* a token has already been published
    /// MUST immediately yield the current token rather than block until the
    /// next refresh. This lets a consumer subscribe at any point (for
    /// example after the provider's readiness gate has fired) and obtain a
    /// usable token without a separate [`get_token`](Self::get_token) call,
    /// avoiding a race between reading the current token and subscribing to
    /// updates. A `tokio::sync::watch`-backed implementation satisfies this
    /// naturally, since a fresh receiver observes the channel's current
    /// value on its first poll.
    fn token_stream(&self) -> TokenStream;
}
