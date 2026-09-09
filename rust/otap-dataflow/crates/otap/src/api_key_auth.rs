// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `api_key_provider` capability.

use std::str::FromStr;
use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::capability::auth::api_key_provider::{
    API_KEY_USABLE_MARGIN, ApiKeyStream,
};
use otel_arrow_dfe_engine::local::capability::auth::api_key_provider::ApiKeyProvider;

use crate::http_client_auth_provider::*;

/// Consumer-side API Key authenticator: subscribes to a provider's API Key
/// stream, caches the built header, and reports usability.
///
/// All API Key expiry/stream state lives here, so an exporter holds one of these
/// and never touches an API Key directly.
pub struct ApiKeyAuth {
    /// Subscription to the provider's API Key refreshes.
    stream: ApiKeyStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The header built from the latest API Key.
    cached_header: Option<(HeaderName, HeaderValue)>,
    /// Expiry of the API Key behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached API Key, bumped on
    /// each successful refresh (starts at 0, meaning "no API Key yet"). Stamped
    /// onto each request so a later 401 can be matched to the exact API Key
    /// generation it used, letting a rejection for an already-replaced API Key
    /// be ignored.
    generation: u64,
}

impl ApiKeyAuth {
    /// Subscribes to `provider`'s API Key stream, raising warnings through
    /// `events`. Per the `ApiKeyProvider::api_key_stream` contract, a
    /// subscription created after an API Key has been published immediately yields
    /// that current API Key, so the exporter needs no separate `get_api_key()`
    /// seeding step.
    #[must_use]
    pub fn new(provider: Box<dyn ApiKeyProvider>) -> Self {
        Self {
            stream: provider.api_key_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for ApiKeyAuth {
    fn is_active(&self) -> bool {
        self.stream_active
    }

    fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring API Key
            (true, Some(expires_on)) => expires_on > Instant::now() + API_KEY_USABLE_MARGIN,
        }
    }

    fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "api key at/near expiry; awaiting refresh"
        } else {
            "api key unavailable"
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_header
            .clone()
            .map(|(name, value)| (name, value, self.generation))
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(API_KEY_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) {
        match self.stream.next().await {
            Some(api_key) => {
                let header_name = match api_key.get_http_header_name_attribute() {
                    Some(header) => match HeaderName::from_str(header) {
                        Ok(header) => header,
                        Err(_) => {
                            (events.invalid)(
                                "API Key configured HTTP header attribute is malformed",
                            );
                            return;
                        }
                    },
                    None => {
                        (events.invalid)("API Key HTTP header attribute not configured");
                        return;
                    }
                };

                let header_value = if let Some(scheme) = api_key.get_http_header_scheme_attribute()
                {
                    HeaderValue::from_str(&format!("{scheme} {}", api_key.expose_value()))
                } else {
                    HeaderValue::from_str(api_key.expose_value())
                };

                match header_value {
                    Ok(mut header_value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        header_value.set_sensitive(true);
                        self.cached_header = Some((header_name, header_value));
                        self.cached_expiry = api_key.get_expires_on();
                        // A new cached API Key starts a new generation, so a 401 for
                        // an earlier API Key no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                    }
                    Err(_) => {
                        // Malformed API Key: keep the previous cached API Key (if any).
                        (events.invalid)("Malformed API Key");
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached API Key. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                (events.stream_closed)();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use otel_arrow_dfe_engine::capability::auth::ApiKey;
    use std::cell::Cell;

    thread_local! {
        /// Number of `invalid` notifications raised on this test thread.
        static INVALID: Cell<usize> = const { Cell::new(0) };
        /// Number of `stream_closed` notifications raised on this test thread.
        static STREAM_CLOSURES: Cell<usize> = const { Cell::new(0) };
    }

    /// Recording event hooks. The hooks take no receiver, so the counters are
    /// thread-local; the test harness gives each test its own thread, and every
    /// test resets them before use.
    const TEST_EVENTS: HttpClientAuthProviderEvents = HttpClientAuthProviderEvents {
        invalid: |_| INVALID.set(INVALID.get() + 1),
        stream_closed: || STREAM_CLOSURES.set(STREAM_CLOSURES.get() + 1),
    };

    fn reset_events() {
        INVALID.set(0);
        STREAM_CLOSURES.set(0);
    }

    /// Builds an adapter holding a usable, non-expiring api key at `generation`,
    /// with an inert (empty) stream so only `invalidate` behavior is exercised.
    fn auth_with_cached_api_key(generation: u64) -> ApiKeyAuth {
        ApiKeyAuth {
            stream: stream::empty().boxed_local(),
            stream_active: false,
            cached_header: Some((
                HeaderName::from_str("X-API-KEY").unwrap(),
                HeaderValue::from_static("test-api-key"),
            )),
            cached_expiry: None,
            generation,
        }
    }

    /// Builds an adapter subscribed to a finite stream that publishes
    /// `api_keys` in order and then ends, so a test can drive `poll_refresh` one
    /// publication at a time and also reach the stream-closed branch.
    fn auth_over(api_keys: Vec<ApiKey>) -> ApiKeyAuth {
        reset_events();
        ApiKeyAuth {
            stream: stream::iter(api_keys).boxed_local(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }

    // Scenario: a 401 names the api key generation currently cached.
    // Guarantees: the rejected api key is dropped so intake back-pressures until the
    // provider's next publication, instead of the rejected api key being sent again.
    #[test]
    fn invalidate_drops_the_matching_generation() {
        let mut auth = auth_with_cached_api_key(7);
        assert!(auth.is_ready());

        auth.invalidate(7);

        assert!(
            !auth.is_ready(),
            "a 401 for the cached generation must clear the api key"
        );
    }

    // Scenario: a 401 names an older generation than the one now cached, i.e. a
    // newer api key was published after the failing request was sent.
    // Guarantees: the still-valid current api key is kept, so a stale rejection
    // does not stall exports until an unnecessary extra refresh.
    #[test]
    fn invalidate_ignores_a_stale_generation() {
        let mut auth = auth_with_cached_api_key(7);

        auth.invalidate(6);

        assert!(
            auth.is_ready(),
            "a 401 for a superseded generation must not clear the newer api key"
        );
    }

    // Scenario: the provider publishes its first api key on the subscription.
    // Guarantees: the adapter caches an `<name>: <value>` header,
    // marks it sensitive so it is redacted in `Debug` and excluded from the
    // HPACK dynamic table, reports readiness, and stamps a non-zero generation
    // so a later rejection can name exactly this api key.
    #[tokio::test]
    async fn poll_refresh_caches_the_published_api_key_as_a_sensitive_header() {
        let mut auth = auth_over(vec![
            ApiKey::new("first").with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(
            auth.is_ready(),
            "a published api key must make the adapter ready"
        );
        let (name, value, generation) =
            auth.header().expect("a cached api key must yield a header");
        assert_eq!(name.as_str(), "x-api-key");
        assert_eq!(value.to_str().unwrap(), "first");
        assert!(
            value.is_sensitive(),
            "the credential must be marked sensitive so it is never HPACK-indexed"
        );
        assert_eq!(
            generation, 1,
            "the first cached api key must not reuse the \
            'no api key yet' generation, so a rejection can be attributed"
        );
    }

    // Scenario: the provider publishes its first api key on the subscription.
    // Guarantees: the adapter caches an `<header>: <scheme> <value>` header
    // and stamps a non-zero generation
    // so a later rejection can name exactly this api key.
    #[tokio::test]
    async fn poll_refresh_caches_the_published_api_key_as_a_header_with_scheme() {
        let mut auth = auth_over(vec![
            ApiKey::new("first")
                .with_http_header_name_attribute("x-api-key")
                .with_http_header_scheme_attribute("MY_SCHEME"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(
            auth.is_ready(),
            "a published api key must make the adapter ready"
        );
        let (name, value, generation) =
            auth.header().expect("a cached api key must yield a header");
        assert_eq!(name.as_str(), "x-api-key");
        assert_eq!(value.to_str().unwrap(), "MY_SCHEME first");
        assert_eq!(
            generation, 1,
            "the first cached api key must not reuse the \
            'no api key yet' generation, so a rejection can be attributed"
        );
    }

    #[tokio::test]
    async fn malformed_refresh_is_reported_when_http_header_name_is_missing() {
        let mut auth = auth_over(vec![ApiKey::new("good")]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert_eq!(
            INVALID.get(),
            1,
            "an api key that cannot become a header value must be reported"
        );
    }

    // Scenario: a refresh publishes an api key whose bytes cannot form a header
    // value, while a usable api key is already cached.
    // Guarantees: the malformed publication is reported and dropped, and the
    // previously cached api key keeps being used at its own generation, so a
    // single bad refresh cannot stall exports.
    #[tokio::test]
    async fn malformed_refresh_is_reported_and_leaves_the_cached_api_key_intact() {
        let mut auth = auth_over(vec![
            ApiKey::new("good").with_http_header_name_attribute("X-API-KEY"),
            ApiKey::new("bad\nvalue").with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;
        auth.poll_refresh(&TEST_EVENTS).await;

        assert_eq!(
            INVALID.get(),
            1,
            "an api key that cannot become a header value must be reported"
        );
        let (name, value, generation) = auth.header().expect("the earlier api key must be kept");
        assert_eq!(name.as_str(), "x-api-key");
        assert_eq!(value.to_str().unwrap(), "good");
        assert_eq!(
            generation, 1,
            "a rejected publication must not advance the generation"
        );
    }

    // Scenario: the provider closes its api key stream after publishing an api key.
    // Guarantees: the closure is reported, the adapter stops advertising itself
    // as pollable so the exporter's `select!` arm goes quiet instead of
    // busy-looping on a dead stream, and the last api key stays usable.
    #[tokio::test]
    async fn closed_stream_is_reported_and_the_last_api_key_stays_usable() {
        let mut auth = auth_over(vec![
            ApiKey::new("last").with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;
        auth.poll_refresh(&TEST_EVENTS).await;

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
            "closing the stream must not discard the last usable api key"
        );
    }

    // Scenario: no api key has been published yet.
    // Guarantees: the adapter is not ready, hands back no header to stamp, arms
    // no refresh timer, and reports the reason that distinguishes "never
    // arrived" from "expiring", so the NACK text tells an operator which it is.
    #[test]
    fn adapter_without_a_api_key_is_unusable_and_says_why() {
        let auth = auth_over(vec![]);

        assert!(!auth.is_ready());
        assert!(auth.header().is_none());
        assert!(auth.refresh_deadline().is_none());
        assert_eq!(auth.not_ready_reason(), "api key unavailable");
    }

    // Scenario: the cached api key is still valid but expires inside the
    // usability margin.
    // Guarantees: it is treated as unusable so the exporter back-pressures
    // rather than sending a request that could outlive its api key, no refresh
    // timer is armed for an already-lapsed margin, and the reason names expiry.
    #[tokio::test]
    async fn api_key_inside_the_usability_margin_is_not_usable() {
        let mut auth = auth_over(vec![
            ApiKey::new("expiring")
                .with_expiry(Instant::now() + API_KEY_USABLE_MARGIN / 2)
                .with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(
            !auth.is_ready(),
            "an api key inside the usability margin must gate intake"
        );
        assert!(
            auth.refresh_deadline().is_none(),
            "an already-lapsed margin must arm no timer"
        );
        assert_eq!(
            auth.not_ready_reason(),
            "api key at/near expiry; awaiting refresh"
        );
    }

    // Scenario: the cached api key expires comfortably beyond the usability
    // margin.
    // Guarantees: it is usable now, and the reported deadline is exactly the
    // instant readiness flips, so the exporter wakes to gate intake before a
    // near-expiry batch is admitted rather than after.
    #[tokio::test]
    async fn refresh_deadline_is_the_instant_readiness_lapses() {
        let expires_on = Instant::now() + API_KEY_USABLE_MARGIN * 10;
        let mut auth = auth_over(vec![
            ApiKey::new("long-lived")
                .with_expiry(expires_on)
                .with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(auth.is_ready());
        assert_eq!(
            auth.refresh_deadline(),
            Some(expires_on - API_KEY_USABLE_MARGIN),
            "the timer must fire when the api key enters the usability margin"
        );
    }

    // Scenario: the provider publishes an api key with no known expiry.
    // Guarantees: it is usable and arms no refresh timer, so the exporter does
    // not register a timer that can never be justified by an expiry.
    #[tokio::test]
    async fn non_expiring_api_key_arms_no_refresh_deadline() {
        let mut auth = auth_over(vec![
            ApiKey::new("forever").with_http_header_name_attribute("x-api-key"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(auth.is_ready());
        assert!(auth.refresh_deadline().is_none());
    }

    // Scenario: a completed export reports the generation the server rejected.
    // Guarantees: the exporter's rejection hand-off drops exactly that api key, so
    // the retry waits for the provider's next publication instead of replaying
    // the rejected credential.
    #[test]
    fn apply_auth_rejection_drops_the_reported_generation() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_api_key(3)));

        apply_auth_rejection(&mut auth, Some(3));

        assert!(!auth.expect("the adapter is retained").is_ready());
    }

    // Scenario: an export completes without naming a rejected generation (it
    // succeeded, or failed for a non-auth reason).
    // Guarantees: the cached api key survives, so ordinary transport failures do
    // not stall intake behind an unnecessary refresh.
    #[test]
    fn apply_auth_rejection_keeps_the_api_key_when_nothing_was_rejected() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_api_key(3)));

        apply_auth_rejection(&mut auth, None);

        assert!(auth.expect("the adapter is retained").is_ready());
    }

    // Scenario: no provider is bound, so the exporter holds no adapter.
    // Guarantees: the shared rejection hand-off is a no-op rather than a panic,
    // which is what lets the exporter call it unconditionally on every
    // completion.
    #[test]
    fn apply_auth_rejection_without_a_bound_provider_is_a_no_op() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> = None;

        apply_auth_rejection(&mut auth, Some(1));

        assert!(auth.is_none());
    }
}
