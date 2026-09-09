// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `basic_auth_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a credential, so the exporter itself stays auth-agnostic: it drives
//! [`BearerAuth::poll_refresh`] in its `select!` loop, asks
//! [`BearerAuth::is_ready`] before admitting data, and stamps
//! [`BearerAuth::header`] onto each request. The cached credential is an
//! `http::HeaderValue`, which both transports accept (tonic's `MetadataMap` is
//! backed by an `http::HeaderMap`), so core and contrib nodes on either
//! protocol can share this adapter.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; this **adapter** only subscribes to the provider's credential
//! stream, caches the built `Authorization` header, and tracks whether that
//! cached credential is still usable. The exporter is the "dumb caller".

use std::time::Instant;

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use futures::StreamExt;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::{
    BASIC_AUTH_CREDENTIAL_USABLE_MARGIN, BasicAuthCredentialStream,
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;

use crate::http_client_auth_provider::*;

/// Consumer-side basic-auth authenticator: subscribes to a provider's credential
/// stream, caches the built `Authorization` header, and reports usability.
///
/// All credential/expiry/stream state lives here, so an exporter holds one of these
/// and never touches a credential directly.
pub struct BasicAuth {
    /// Subscription to the provider's credential refreshes.
    stream: BasicAuthCredentialStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The `Authorization: Basic <base64(username:password)>` header built from the latest credential.
    cached_header: Option<HeaderValue>,
    /// Expiry of the credential behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached credential, bumped on each
    /// successful refresh (starts at 0, meaning "no credential yet"). Stamped onto
    /// each request so a later 401 can be matched to the exact credential generation
    /// it used, letting a rejection for an already-replaced credential be ignored.
    generation: u64,
}

impl BasicAuth {
    /// Subscribes to `provider`'s credential stream, raising warnings through
    /// `events`. Per the `BasicAuthProvider::credential_stream` contract, a
    /// subscription created after a credential has been published immediately yields
    /// that current credential, so the exporter needs no separate `get_credential()`
    /// seeding step.
    #[must_use]
    pub fn new(provider: Box<dyn BasicAuthProvider>) -> Self {
        Self {
            stream: provider.credential_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for BasicAuth {
    fn is_active(&self) -> bool {
        self.stream_active
    }

    fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring credential
            (true, Some(expires_on)) => {
                expires_on > Instant::now() + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN
            }
        }
    }

    fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "credential at/near expiry; awaiting refresh"
        } else {
            "credential unavailable"
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_header
            .clone()
            .map(|header| (http::header::AUTHORIZATION, header, self.generation))
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(BASIC_AUTH_CREDENTIAL_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) {
        match self.stream.next().await {
            Some(credential) => {
                let credentials = format!(
                    "{}:{}",
                    credential.expose_username(),
                    credential.expose_password()
                );
                let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

                match HeaderValue::from_str(&format!("Basic {encoded_credentials}")) {
                    Ok(mut value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        value.set_sensitive(true);
                        self.cached_header = Some(value);
                        self.cached_expiry = credential.expires_on();
                        // A new cached credential starts a new generation, so a 401 for
                        // an earlier credential no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                    }
                    Err(_) => {
                        // Malformed credential: keep the previous cached credential (if any).
                        (events.invalid)("Malformed credential");
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached credential. Not expected with a
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
    use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
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

    /// Builds an adapter holding a usable, non-expiring credential at `generation`,
    /// with an inert (empty) stream so only `invalidate` behavior is exercised.
    fn auth_with_cached_credential(generation: u64) -> BasicAuth {
        BasicAuth {
            stream: stream::empty().boxed_local(),
            stream_active: false,
            cached_header: Some(HeaderValue::from_static("test-credential")),
            cached_expiry: None,
            generation,
        }
    }

    /// Builds an adapter subscribed to a finite stream that publishes
    /// `credentials` in order and then ends, so a test can drive `poll_refresh` one
    /// publication at a time and also reach the stream-closed branch.
    fn auth_over(credentials: Vec<BasicAuthCredential>) -> BasicAuth {
        reset_events();
        BasicAuth {
            stream: stream::iter(credentials).boxed_local(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }

    // Scenario: a 401 names the credential generation currently cached.
    // Guarantees: the rejected credential is dropped so intake back-pressures until the
    // provider's next publication, instead of the rejected credential being sent again.
    #[test]
    fn invalidate_drops_the_matching_generation() {
        let mut auth = auth_with_cached_credential(7);
        assert!(auth.is_ready());

        auth.invalidate(7);

        assert!(
            !auth.is_ready(),
            "a 401 for the cached generation must clear the credential"
        );
    }

    // Scenario: a 401 names an older generation than the one now cached, i.e. a
    // newer credential was published after the failing request was sent.
    // Guarantees: the still-valid current credential is kept, so a stale rejection
    // does not stall exports until an unnecessary extra refresh.
    #[test]
    fn invalidate_ignores_a_stale_generation() {
        let mut auth = auth_with_cached_credential(7);

        auth.invalidate(6);

        assert!(
            auth.is_ready(),
            "a 401 for a superseded generation must not clear the newer credential"
        );
    }

    // Scenario: the provider publishes its first credential on the subscription.
    // Guarantees: the adapter caches an `<name>: <value>` header,
    // marks it sensitive so it is redacted in `Debug` and excluded from the
    // HPACK dynamic table, reports readiness, and stamps a non-zero generation
    // so a later rejection can name exactly this credential.
    #[tokio::test]
    async fn poll_refresh_caches_the_published_credential_as_a_sensitive_header() {
        let mut auth = auth_over(vec![
            BasicAuthCredential::new("user", "pass").expect("valid credential"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(
            auth.is_ready(),
            "a published credential must make the adapter ready"
        );
        let (name, value, generation) = auth
            .header()
            .expect("a cached credential must yield a header");
        assert_eq!(name.as_str(), "authorization");
        assert_eq!(value.to_str().unwrap(), "Basic dXNlcjpwYXNz");
        assert!(
            value.is_sensitive(),
            "the credential must be marked sensitive so it is never HPACK-indexed"
        );
        assert_eq!(
            generation, 1,
            "the first cached credential must not reuse the \
            'no credential yet' generation, so a rejection can be attributed"
        );
    }

    // Scenario: the provider closes its credential stream after publishing a credential.
    // Guarantees: the closure is reported, the adapter stops advertising itself
    // as pollable so the exporter's `select!` arm goes quiet instead of
    // busy-looping on a dead stream, and the last credential stays usable.
    #[tokio::test]
    async fn closed_stream_is_reported_and_the_last_credential_stays_usable() {
        let mut auth = auth_over(vec![
            BasicAuthCredential::new("user", "pass").expect("valid credential"),
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
            "closing the stream must not discard the last usable credential"
        );
    }

    // Scenario: no credential has been published yet.
    // Guarantees: the adapter is not ready, hands back no header to stamp, arms
    // no refresh timer, and reports the reason that distinguishes "never
    // arrived" from "expiring", so the NACK text tells an operator which it is.
    #[test]
    fn adapter_without_a_credential_is_unusable_and_says_why() {
        let auth = auth_over(vec![]);

        assert!(!auth.is_ready());
        assert!(auth.header().is_none());
        assert!(auth.refresh_deadline().is_none());
        assert_eq!(auth.not_ready_reason(), "credential unavailable");
    }

    // Scenario: the cached credential is still valid but expires inside the
    // usability margin.
    // Guarantees: it is treated as unusable so the exporter back-pressures
    // rather than sending a request that could outlive its credential, no refresh
    // timer is armed for an already-lapsed margin, and the reason names expiry.
    #[tokio::test]
    async fn api_key_inside_the_usability_margin_is_not_usable() {
        let mut auth = auth_over(vec![
            BasicAuthCredential::new("user", "pass")
                .expect("valid credential")
                .with_expiry(Instant::now() + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN / 2),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(
            !auth.is_ready(),
            "a credential inside the usability margin must gate intake"
        );
        assert!(
            auth.refresh_deadline().is_none(),
            "an already-lapsed margin must arm no timer"
        );
        assert_eq!(
            auth.not_ready_reason(),
            "credential at/near expiry; awaiting refresh"
        );
    }

    // Scenario: the cached credential expires comfortably beyond the usability
    // margin.
    // Guarantees: it is usable now, and the reported deadline is exactly the
    // instant readiness flips, so the exporter wakes to gate intake before a
    // near-expiry batch is admitted rather than after.
    #[tokio::test]
    async fn refresh_deadline_is_the_instant_readiness_lapses() {
        let expires_on = Instant::now() + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN * 10;
        let mut auth = auth_over(vec![
            BasicAuthCredential::new("user", "pass")
                .expect("valid credential")
                .with_expiry(expires_on),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(auth.is_ready());
        assert_eq!(
            auth.refresh_deadline(),
            Some(expires_on - BASIC_AUTH_CREDENTIAL_USABLE_MARGIN),
            "the timer must fire when the credential enters the usability margin"
        );
    }

    // Scenario: the provider publishes a credential with no known expiry.
    // Guarantees: it is usable and arms no refresh timer, so the exporter does
    // not register a timer that can never be justified by an expiry.
    #[tokio::test]
    async fn non_expiring_credential_arms_no_refresh_deadline() {
        let mut auth = auth_over(vec![
            BasicAuthCredential::new("user", "pass").expect("valid credential"),
        ]);

        auth.poll_refresh(&TEST_EVENTS).await;

        assert!(auth.is_ready());
        assert!(auth.refresh_deadline().is_none());
    }

    // Scenario: a completed export reports the generation the server rejected.
    // Guarantees: the exporter's rejection hand-off drops exactly that credential, so
    // the retry waits for the provider's next publication instead of replaying
    // the rejected credential.
    #[test]
    fn apply_auth_rejection_drops_the_reported_generation() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_credential(3)));

        apply_auth_rejection(&mut auth, Some(3));

        assert!(!auth.expect("the adapter is retained").is_ready());
    }

    // Scenario: an export completes without naming a rejected generation (it
    // succeeded, or failed for a non-auth reason).
    // Guarantees: the cached credential survives, so ordinary transport failures do
    // not stall intake behind an unnecessary refresh.
    #[test]
    fn apply_auth_rejection_keeps_the_credential_when_nothing_was_rejected() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_credential(3)));

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
