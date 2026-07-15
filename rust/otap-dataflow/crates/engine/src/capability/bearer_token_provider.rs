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
//! - `pub(crate) mod local::BearerTokenProvider` (`!Send` trait variant)
//! - `pub(crate) mod shared::BearerTokenProvider` (`Send` trait variant)
//! - A `SharedAsLocalBearerTokenProvider` adapter
//! - A zero-sized `pub struct BearerTokenProvider` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use super::error::CapabilityError;
use futures::Stream;
use otap_df_engine_macros::capability;
use secrecy::{ExposeSecret, SecretString};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// An OAuth bearer token plus its (optional) expiry.
///
/// The secret is wrapped in [`SecretString`], which zeroizes on drop and
/// masks itself in [`Debug`] output, so it cannot leak into logs or
/// telemetry. The `SecretString` sits behind an [`Arc`] so cloning a
/// token (handing it to multiple `token_stream` subscribers, or returning
/// it from `get_token` on the hot path) is a cheap refcount bump that
/// shares one plaintext allocation rather than copying the secret bytes.
///
/// `expires_on` is a monotonic [`Instant`] — providers convert the
/// credential's absolute wall-clock expiry to an `Instant` once, so the
/// value is immune to wall-clock jumps thereafter. `None` means the token
/// does not expire (or no expiry was reported).
#[derive(Clone, Debug)]
pub struct BearerToken {
    secret: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl BearerToken {
    /// Creates a token from its secret and optional monotonic expiry.
    ///
    /// Accepts anything convertible into [`SecretString`] (e.g. a
    /// `String`), which is then shared behind an [`Arc`].
    #[must_use]
    pub fn new(secret: impl Into<SecretString>, expires_on: Option<Instant>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on,
        }
    }

    /// Creates a token from its secret and an **absolute** wall-clock
    /// expiry.
    ///
    /// Credential services that report an absolute expiry often give it as
    /// calendar time ([`SystemTime`]), but [`BearerToken`] stores a
    /// monotonic [`Instant`] (see the field docs for why). Every such
    /// provider has to perform the same wall-clock-to-monotonic
    /// conversion; this constructor centralizes it so no provider gets it
    /// subtly wrong.
    ///
    /// It measures how far `expires_on` is from *now* and offsets the
    /// current `Instant` by that duration. An `expires_on` already in the
    /// past (or a backwards clock) clamps to "expires immediately"
    /// (`Instant::now()`) rather than producing a time before now.
    #[must_use]
    pub fn from_absolute_expiry(secret: impl Into<SecretString>, expires_on: SystemTime) -> Self {
        let remaining = expires_on
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO);
        Self::new(secret, Some(Instant::now() + remaining))
    }

    /// Exposes the bearer token secret, for injection into an
    /// `Authorization` header or an `object_store` credential.
    ///
    /// Named `expose_token` (rather than a plain getter) so every
    /// plaintext access is explicit and greppable.
    #[must_use]
    pub fn expose_token(&self) -> &str {
        self.secret.expose_secret()
    }

    /// The monotonic instant at which this token expires, if known.
    #[must_use]
    pub fn expires_on(&self) -> Option<Instant> {
        self.expires_on
    }
}

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
    /// misses into a single call — but that is a provider implementation
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
    fn token_stream(&self) -> TokenStream;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_round_trip() {
        let now = Instant::now();
        let token = BearerToken::new("super-secret".to_owned(), Some(now));
        assert_eq!(token.expose_token(), "super-secret");
        assert_eq!(token.expires_on(), Some(now));

        let non_expiring = BearerToken::new("s".to_owned(), None);
        assert_eq!(non_expiring.expires_on(), None);
    }

    #[test]
    fn from_absolute_expiry_converts_future_wall_clock_to_instant() {
        let before = Instant::now();
        let token = BearerToken::from_absolute_expiry(
            "s".to_owned(),
            SystemTime::now() + Duration::from_secs(60),
        );
        let after = Instant::now();
        let expiry = token.expires_on().expect("future expiry is set");
        // The converted instant lands ~60s ahead of when we called it.
        assert!(expiry >= before + Duration::from_secs(59));
        assert!(expiry <= after + Duration::from_secs(61));
    }

    #[test]
    fn from_absolute_expiry_clamps_past_wall_clock_to_now() {
        let before = Instant::now();
        let token = BearerToken::from_absolute_expiry(
            "s".to_owned(),
            SystemTime::now() - Duration::from_secs(60),
        );
        let after = Instant::now();
        let expiry = token.expires_on().expect("expiry is set");
        // A past expiry clamps to "now", never a time before now.
        assert!(expiry >= before);
        assert!(expiry <= after);
    }

    #[test]
    fn clone_shares_the_same_secret_allocation() {
        let token = BearerToken::new("super-secret".to_owned(), None);
        let cloned = token.clone();
        // Both handles observe the same plaintext...
        assert_eq!(token.expose_token(), cloned.expose_token());
        // ...backed by one shared allocation (a clone is a refcount bump,
        // not a fresh copy of the secret bytes).
        assert!(std::ptr::eq(token.expose_token(), cloned.expose_token()));
    }

    #[test]
    fn debug_never_leaks_the_secret() {
        let token = BearerToken::new("super-secret".to_owned(), None);
        let rendered = format!("{token:?}");
        assert!(
            !rendered.contains("super-secret"),
            "secret leaked: {rendered}"
        );
    }
}
