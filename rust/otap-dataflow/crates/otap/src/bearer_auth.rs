// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `bearer_token_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a bearer token, so the exporter itself stays auth-agnostic: it drives
//! [`BearerAuth::poll_refresh`] in its `select!` loop, asks
//! [`BearerAuth::is_ready`] before admitting data, and stamps
//! [`BearerAuth::header`] onto each request. The cached credential is an
//! `http::HeaderValue`, which both transports accept (tonic's `MetadataMap` is
//! backed by an `http::HeaderMap`), so core and contrib nodes on either
//! protocol can share this adapter.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; this **adapter** only subscribes to the provider's token
//! stream, caches the built `Authorization` header, and tracks whether that
//! cached token is still usable. The exporter is the "dumb caller".

use std::time::Instant;

use futures::StreamExt;
use http::HeaderValue;
use http::header::InvalidHeaderValue;
use otap_df_engine::capability::auth::bearer_token_provider::{TOKEN_USABLE_MARGIN, TokenStream};
use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;

/// The warnings this adapter can raise, supplied by the owning exporter so each
/// event name is namespaced to that exporter (e.g. `otlp.exporter.grpc.*`)
/// rather than to the shared adapter. `otel_warn!` const-validates its event
/// name, so the name has to be a literal at the emitting call site; passing the
/// emitters as function pointers satisfies that without making the adapter
/// generic over an exporter marker type.
#[derive(Clone, Copy)]
pub struct BearerAuthEvents {
    /// A published token could not be turned into an `Authorization` header.
    pub invalid_token: fn(&InvalidHeaderValue),

    /// The provider closed its token stream; no further refreshes will arrive.
    pub token_stream_closed: fn(),
}

/// Consumer-side bearer-token authenticator: subscribes to a provider's token
/// stream, caches the built `Authorization` header, and reports usability.
///
/// All token/expiry/stream state lives here, so an exporter holds one of these
/// and never touches a token directly.
pub struct BearerAuth {
    /// Subscription to the provider's token refreshes.
    stream: TokenStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The `Authorization: Bearer <token>` header built from the latest token.
    cached_header: Option<HeaderValue>,
    /// Expiry of the token behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached token, bumped on each
    /// successful refresh (starts at 0, meaning "no token yet"). Stamped onto
    /// each request so a later 401 can be matched to the exact token generation
    /// it used, letting a rejection for an already-replaced token be ignored.
    generation: u64,
    /// The owning exporter's namespaced warning emitters.
    events: BearerAuthEvents,
}

impl BearerAuth {
    /// Subscribes to `provider`'s token stream, raising warnings through
    /// `events`. Per the `BearerTokenProvider::token_stream` contract, a
    /// subscription created after a token has been published immediately yields
    /// that current token, so the exporter needs no separate `get_token()`
    /// seeding step.
    #[must_use]
    pub fn new(provider: Box<dyn BearerTokenProvider>, events: BearerAuthEvents) -> Self {
        Self {
            stream: provider.token_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
            events,
        }
    }

    /// Whether the token stream is still live and worth polling. Once the
    /// provider closes it, this returns `false` and the last cached token (if
    /// any) keeps being used.
    pub fn is_active(&self) -> bool {
        self.stream_active
    }

    /// Whether a usable token is cached: present and, if it expires, comfortably
    /// before expiry. The exporter admits data only when this is `true`.
    pub fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring token
            (true, Some(expires_on)) => expires_on > Instant::now() + TOKEN_USABLE_MARGIN,
        }
    }

    /// A human-readable reason [`is_ready`](Self::is_ready) is false, for NACK
    /// messages.
    pub fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "bearer token at/near expiry; awaiting refresh"
        } else {
            "bearer token unavailable"
        }
    }

    /// The cached `Authorization` header to stamp on a request, together with the
    /// generation of the token it was built from, cloned for the per-request send
    /// (a cheap refcount bump). `None` when no token is cached; callers
    /// should gate on [`is_ready`](Self::is_ready) first.
    pub fn header(&self) -> Option<(HeaderValue, u64)> {
        self.cached_header
            .clone()
            .map(|header| (header, self.generation))
    }

    /// The instant at which a currently-usable, expiring token crosses the
    /// usability margin (when [`is_ready`](Self::is_ready) flips to false).
    /// `None` when no usable token is cached or the token never expires, so the
    /// caller arms no timer in those cases. When `Some`, it is always in the
    /// future: a usable token is by definition still beyond the margin.
    pub fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(TOKEN_USABLE_MARGIN))
    }

    /// Drops the cached token *if* `generation` is still the one currently cached,
    /// so [`is_ready`](Self::is_ready) returns false until the provider publishes a
    /// replacement. Called when the server rejects a token (HTTP 401, or gRPC
    /// `UNAUTHENTICATED`) so the rejected credential is not sent again.
    ///
    /// The generation guard makes a stale 401 harmless: if a newer token was
    /// cached (or the rejected token already cleared) after the failing request
    /// was sent, `generation` no longer matches the current one and the
    /// still-valid token is kept, avoiding a needless back-pressure stall.
    pub fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    /// Awaits the next published token and refreshes the cache. Only meaningful
    /// while [`is_active`](Self::is_active); on stream close it flips inactive
    /// and keeps the last cached token. Malformed tokens and stream closure are
    /// logged internally.
    pub async fn poll_refresh(&mut self) {
        match self.stream.next().await {
            Some(token) => {
                match HeaderValue::from_str(&format!("Bearer {}", token.expose_token())) {
                    Ok(mut value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        value.set_sensitive(true);
                        self.cached_header = Some(value);
                        self.cached_expiry = token.expires_on();
                        // A new cached token starts a new generation, so a 401 for
                        // an earlier token no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                    }
                    Err(e) => {
                        // Malformed token: keep the previous cached token (if any).
                        (self.events.invalid_token)(&e);
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached token. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                (self.events.token_stream_closed)();
            }
        }
    }
}

/// Applies a token rejection reported by a completed export to the bearer
/// adapter: drops the rejected token generation so it is not sent again, leaving
/// the consumer back-pressured until the provider's next publication.
///
/// Takes the exporter's `Option<BearerAuth>` directly so the common
/// "rejection reported, provider may or may not be bound" shape is expressed
/// once. A no-op when no provider is bound (`rejected_generation` is `None`) or
/// the rejection is stale (a newer token was already cached), per
/// [`BearerAuth::invalidate`]'s generation guard.
pub fn apply_auth_rejection(auth: &mut Option<BearerAuth>, rejected_generation: Option<u64>) {
    if let (Some(generation), Some(adapter)) = (rejected_generation, auth.as_mut()) {
        adapter.invalidate(generation);
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support {
    //! Test doubles shared by the nodes that consume this adapter, so every
    //! consumer's test suite drives the same provider behavior instead of each
    //! maintaining its own copy.

    use async_trait::async_trait;
    use futures::StreamExt;
    use otap_df_engine::capability::CapabilityError;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::bearer_token_provider::TokenStream;
    use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
    use std::time::Instant;

    /// Test double for the `BearerTokenProvider` capability with configurable
    /// stream behavior.
    pub struct MockTokenProvider {
        /// Tokens published on the stream, in order.
        pub tokens: Vec<String>,
        /// Whether the stream stays pending after the tokens are drained
        /// (never ends) rather than closing, which would simulate a provider
        /// that stops refreshing.
        pub keep_open: bool,
        /// Expiry applied to every published token (`None` = non-expiring).
        pub expires_on: Option<Instant>,
    }

    impl MockTokenProvider {
        /// A provider that publishes a single non-expiring token and keeps its
        /// stream open.
        #[must_use]
        pub fn new(token: &str) -> Self {
            Self {
                tokens: vec![token.to_string()],
                keep_open: true,
                expires_on: None,
            }
        }

        /// A provider that is bound but never publishes a token, with its stream
        /// held open so the consumer keeps waiting rather than treating the
        /// silence as a closed stream.
        #[must_use]
        pub fn never_publishes() -> Self {
            Self {
                tokens: vec![],
                keep_open: true,
                expires_on: None,
            }
        }
    }

    #[async_trait(?Send)]
    impl BearerTokenProvider for MockTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            // Not exercised by the exporters (they consume `token_stream`);
            // return the first configured token for completeness.
            Ok(BearerToken::with_expiry(
                self.tokens.first().cloned().unwrap_or_default(),
                None,
            ))
        }

        fn token_stream(&self) -> TokenStream {
            let expires_on = self.expires_on;
            let published = futures::stream::iter(
                self.tokens
                    .clone()
                    .into_iter()
                    .map(move |t| BearerToken::with_expiry(t, expires_on)),
            );
            if self.keep_open {
                published.chain(futures::stream::pending()).boxed()
            } else {
                published.boxed()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use otap_df_engine::capability::auth::BearerToken;
    use std::cell::Cell;

    thread_local! {
        /// Number of `invalid_token` notifications raised on this test thread.
        static INVALID_TOKENS: Cell<usize> = const { Cell::new(0) };
        /// Number of `token_stream_closed` notifications raised on this test thread.
        static STREAM_CLOSURES: Cell<usize> = const { Cell::new(0) };
    }

    /// Recording event hooks. The hooks take no receiver, so the counters are
    /// thread-local; the test harness gives each test its own thread, and every
    /// test resets them before use.
    const TEST_EVENTS: BearerAuthEvents = BearerAuthEvents {
        invalid_token: |_error| INVALID_TOKENS.set(INVALID_TOKENS.get() + 1),
        token_stream_closed: || STREAM_CLOSURES.set(STREAM_CLOSURES.get() + 1),
    };

    fn reset_events() {
        INVALID_TOKENS.set(0);
        STREAM_CLOSURES.set(0);
    }

    /// Builds an adapter holding a usable, non-expiring token at `generation`,
    /// with an inert (empty) stream so only `invalidate` behavior is exercised.
    fn auth_with_cached_token(generation: u64) -> BearerAuth {
        BearerAuth {
            stream: stream::empty().boxed_local(),
            stream_active: false,
            cached_header: Some(HeaderValue::from_static("Bearer test-token")),
            cached_expiry: None,
            generation,
            events: TEST_EVENTS,
        }
    }

    /// Builds a token-less adapter subscribed to a finite stream that publishes
    /// `tokens` in order and then ends, so a test can drive `poll_refresh` one
    /// publication at a time and also reach the stream-closed branch.
    fn auth_over(tokens: Vec<BearerToken>) -> BearerAuth {
        reset_events();
        BearerAuth {
            stream: stream::iter(tokens).boxed_local(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
            events: TEST_EVENTS,
        }
    }

    // Scenario: a 401 names the token generation currently cached.
    // Guarantees: the rejected token is dropped so intake back-pressures until the
    // provider's next publication, instead of the rejected token being sent again.
    #[test]
    fn invalidate_drops_the_matching_generation() {
        let mut auth = auth_with_cached_token(7);
        assert!(auth.is_ready());

        auth.invalidate(7);

        assert!(
            !auth.is_ready(),
            "a 401 for the cached generation must clear the token"
        );
    }

    // Scenario: a 401 names an older generation than the one now cached, i.e. a
    // newer token was published after the failing request was sent.
    // Guarantees: the still-valid current token is kept, so a stale rejection
    // does not stall exports until an unnecessary extra refresh.
    #[test]
    fn invalidate_ignores_a_stale_generation() {
        let mut auth = auth_with_cached_token(7);

        auth.invalidate(6);

        assert!(
            auth.is_ready(),
            "a 401 for a superseded generation must not clear the newer token"
        );
    }

    // Scenario: the provider publishes its first token on the subscription.
    // Guarantees: the adapter caches an `Authorization: Bearer <token>` header,
    // marks it sensitive so it is redacted in `Debug` and excluded from the
    // HPACK dynamic table, reports readiness, and stamps a non-zero generation
    // so a later rejection can name exactly this token.
    #[tokio::test]
    async fn poll_refresh_caches_the_published_token_as_a_sensitive_header() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("first")]);

        auth.poll_refresh().await;

        assert!(
            auth.is_ready(),
            "a published token must make the adapter ready"
        );
        let (header, generation) = auth.header().expect("a cached token must yield a header");
        assert_eq!(header.to_str().unwrap(), "Bearer first");
        assert!(
            header.is_sensitive(),
            "the credential must be marked sensitive so it is never HPACK-indexed"
        );
        assert_eq!(
            generation, 1,
            "the first cached token must not reuse the \
            'no token yet' generation, so a rejection can be attributed"
        );
    }

    // Scenario: a refresh publishes a token whose bytes cannot form a header
    // value, while a usable token is already cached.
    // Guarantees: the malformed publication is reported and dropped, and the
    // previously cached token keeps being used at its own generation, so a
    // single bad refresh cannot stall exports.
    #[tokio::test]
    async fn a_malformed_refresh_is_reported_and_leaves_the_cached_token_intact() {
        let mut auth = auth_over(vec![
            BearerToken::without_expiry("good"),
            BearerToken::without_expiry("bad\nvalue"),
        ]);

        auth.poll_refresh().await;
        auth.poll_refresh().await;

        assert_eq!(
            INVALID_TOKENS.get(),
            1,
            "a token that cannot become a header value must be reported"
        );
        let (header, generation) = auth.header().expect("the earlier token must be kept");
        assert_eq!(header.to_str().unwrap(), "Bearer good");
        assert_eq!(
            generation, 1,
            "a rejected publication must not advance the generation"
        );
    }

    // Scenario: the provider closes its token stream after publishing a token.
    // Guarantees: the closure is reported, the adapter stops advertising itself
    // as pollable so the exporter's `select!` arm goes quiet instead of
    // busy-looping on a dead stream, and the last token stays usable.
    #[tokio::test]
    async fn a_closed_stream_is_reported_and_the_last_token_stays_usable() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("last")]);

        auth.poll_refresh().await;
        auth.poll_refresh().await;

        assert_eq!(
            STREAM_CLOSURES.get(),
            1,
            "the provider closing its stream must be reported"
        );
        assert!(
            !auth.is_active(),
            "a closed stream must not be polled again"
        );
        assert!(
            auth.is_ready(),
            "closing the stream must not discard the last usable token"
        );
    }

    // Scenario: no token has been published yet.
    // Guarantees: the adapter is not ready, hands back no header to stamp, arms
    // no refresh timer, and reports the reason that distinguishes "never
    // arrived" from "expiring", so the NACK text tells an operator which it is.
    #[test]
    fn an_adapter_without_a_token_is_unusable_and_says_why() {
        let auth = auth_over(vec![]);

        assert!(!auth.is_ready());
        assert!(auth.header().is_none());
        assert!(auth.refresh_deadline().is_none());
        assert_eq!(auth.not_ready_reason(), "bearer token unavailable");
    }

    // Scenario: the cached token is still valid but expires inside the
    // usability margin.
    // Guarantees: it is treated as unusable so the exporter back-pressures
    // rather than sending a request that could outlive its token, no refresh
    // timer is armed for an already-lapsed margin, and the reason names expiry.
    #[tokio::test]
    async fn a_token_inside_the_usability_margin_is_not_usable() {
        let mut auth = auth_over(vec![BearerToken::with_expiry(
            "expiring",
            Some(Instant::now() + TOKEN_USABLE_MARGIN / 2),
        )]);

        auth.poll_refresh().await;

        assert!(
            !auth.is_ready(),
            "a token inside the usability margin must gate intake"
        );
        assert!(
            auth.refresh_deadline().is_none(),
            "an already-lapsed margin must arm no timer"
        );
        assert_eq!(
            auth.not_ready_reason(),
            "bearer token at/near expiry; awaiting refresh"
        );
    }

    // Scenario: the cached token expires comfortably beyond the usability
    // margin.
    // Guarantees: it is usable now, and the reported deadline is exactly the
    // instant readiness flips, so the exporter wakes to gate intake before a
    // near-expiry batch is admitted rather than after.
    #[tokio::test]
    async fn refresh_deadline_is_the_instant_readiness_lapses() {
        let expires_on = Instant::now() + TOKEN_USABLE_MARGIN * 10;
        let mut auth = auth_over(vec![BearerToken::with_expiry(
            "long-lived",
            Some(expires_on),
        )]);

        auth.poll_refresh().await;

        assert!(auth.is_ready());
        assert_eq!(
            auth.refresh_deadline(),
            Some(expires_on - TOKEN_USABLE_MARGIN),
            "the timer must fire when the token enters the usability margin"
        );
    }

    // Scenario: the provider publishes a token with no known expiry.
    // Guarantees: it is usable and arms no refresh timer, so the exporter does
    // not register a timer that can never be justified by an expiry.
    #[tokio::test]
    async fn a_non_expiring_token_arms_no_refresh_deadline() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("forever")]);

        auth.poll_refresh().await;

        assert!(auth.is_ready());
        assert!(auth.refresh_deadline().is_none());
    }

    // Scenario: a completed export reports the generation the server rejected.
    // Guarantees: the exporter's rejection hand-off drops exactly that token, so
    // the retry waits for the provider's next publication instead of replaying
    // the rejected credential.
    #[test]
    fn apply_auth_rejection_drops_the_reported_generation() {
        let mut auth = Some(auth_with_cached_token(3));

        apply_auth_rejection(&mut auth, Some(3));

        assert!(!auth.expect("the adapter is retained").is_ready());
    }

    // Scenario: an export completes without naming a rejected generation (it
    // succeeded, or failed for a non-auth reason).
    // Guarantees: the cached token survives, so ordinary transport failures do
    // not stall intake behind an unnecessary refresh.
    #[test]
    fn apply_auth_rejection_keeps_the_token_when_nothing_was_rejected() {
        let mut auth = Some(auth_with_cached_token(3));

        apply_auth_rejection(&mut auth, None);

        assert!(auth.expect("the adapter is retained").is_ready());
    }

    // Scenario: no provider is bound, so the exporter holds no adapter.
    // Guarantees: the shared rejection hand-off is a no-op rather than a panic,
    // which is what lets the exporter call it unconditionally on every
    // completion.
    #[test]
    fn apply_auth_rejection_without_a_bound_provider_is_a_no_op() {
        let mut auth: Option<BearerAuth> = None;

        apply_auth_rejection(&mut auth, Some(1));

        assert!(auth.is_none());
    }
}
