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
//! - `pub mod local::BearerTokenProvider` (`!Send` trait variant)
//! - `pub mod shared::BearerTokenProvider` (`Send` trait variant)
//! - A `SharedAsLocalBearerTokenProvider` adapter
//! - A zero-sized `pub struct BearerTokenProvider` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use super::error::CapabilityError;
use futures::Stream;
use otap_df_engine_macros::capability;
use std::fmt;
use std::pin::Pin;
use std::time::Instant;

/// An OAuth bearer token plus its (optional) expiry.
///
/// The secret is held in memory only and is never emitted by [`Debug`]
/// (see the manual impl below) so it cannot leak into logs or telemetry.
/// `expires_on` is a monotonic [`Instant`] — providers convert the
/// credential's absolute wall-clock expiry to an `Instant` once, so the
/// value is immune to wall-clock jumps thereafter. `None` means the token
/// does not expire (or no expiry was reported).
#[derive(Clone)]
pub struct BearerToken {
    secret: String,
    expires_on: Option<Instant>,
}

impl BearerToken {
    /// Creates a token from its secret and optional monotonic expiry.
    #[must_use]
    pub fn new(secret: String, expires_on: Option<Instant>) -> Self {
        Self { secret, expires_on }
    }

    /// The bearer token secret, for injection into an `Authorization`
    /// header or an `object_store` credential.
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// The monotonic instant at which this token expires, if known.
    #[must_use]
    pub fn expires_on(&self) -> Option<Instant> {
        self.expires_on
    }
}

// Manual `Debug` that redacts the secret. Never print `self.secret`.
impl fmt::Debug for BearerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BearerToken")
            .field("secret", &"<redacted>")
            .field("expires_on", &self.expires_on)
            .finish()
    }
}

/// A per-consumer subscription to token refreshes.
///
/// Boxed to hide the concrete stream type so providers can back it
/// differently (e.g. a `watch` channel or an `unfold`) without changing
/// the signature. The `Send` bound is intentionally omitted: the
/// subscription is always consumed on the core that created it
/// (thread-per-core), so it need not be `Send`. The `#[capability]`
/// macro emits this signature into both the `local` (`?Send`) and
/// `shared` (`Send`) trait variants unchanged.
pub type TokenStream = Pin<Box<dyn Stream<Item = Result<BearerToken, CapabilityError>> + 'static>>;

/// Hands out OAuth bearer tokens to data-path nodes.
#[capability(
    name = "bearer_token_provider",
    description = "Provides OAuth bearer tokens, refreshed in the background"
)]
pub trait BearerTokenProvider {
    /// Returns the current valid token.
    ///
    /// The fast path reads a cached token; the slow path performs a
    /// single (coalesced) credential call on a cache miss. Returns a
    /// [`CapabilityError`] if no valid token can be produced.
    async fn get_token(&self) -> Result<BearerToken, CapabilityError>;

    /// Subscribes to the stream of token refreshes.
    ///
    /// Yields each newly published token for the lifetime of the
    /// extension. Each call returns an independent subscription.
    fn token_stream(&self) -> TokenStream;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_round_trip() {
        let now = Instant::now();
        let token = BearerToken::new("super-secret".to_owned(), Some(now));
        assert_eq!(token.secret(), "super-secret");
        assert_eq!(token.expires_on(), Some(now));

        let non_expiring = BearerToken::new("s".to_owned(), None);
        assert_eq!(non_expiring.expires_on(), None);
    }

    #[test]
    fn debug_redacts_secret() {
        let token = BearerToken::new("super-secret".to_owned(), None);
        let rendered = format!("{token:?}");
        assert!(
            !rendered.contains("super-secret"),
            "secret leaked: {rendered}"
        );
        assert!(rendered.contains("<redacted>"));
    }
}
