// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `agent_fed_auth_provider` capability.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::{
    capability::auth::{
        agent_fed_credential_provider::AgentFedCredentialSnapshot,
        bearer_token_provider::TOKEN_USABLE_MARGIN,
    },
    local::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider,
};
use rand::RngExt;

use crate::http_client_auth_provider::*;

/// Base reschedule delay after a failed acquisition. Consecutive failures grow
/// this exponentially (with jitter) up to `MAX_TOKEN_REFRESH_RETRY_SECS`.
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;
/// Upper bound on the retry backoff after repeated failures.
const MAX_TOKEN_REFRESH_RETRY_SECS: u64 = 300;

const NAME: &str = "AgentFedAuth";

/// Consumer-side bearer-token authenticator: subscribes to a provider's agent
/// fed credentials caches the built `Authorization` header, and reports
/// usability.
///
/// All credential/expiry state lives here, so an exporter holds one of these
/// and never touches a credential directly.
pub struct AgentFedAuth {
    provider: Box<dyn AgentFedCredentialProvider>,
    cached_credential: Option<CachedCredential>,
    generation: u64,
    rejected_generation: Option<u64>,
}

#[derive(Debug)]
struct CachedCredential {
    snapshot: Arc<AgentFedCredentialSnapshot>,
    header: HeaderValue,
    expires_on: Option<Instant>,
    generation: u64,
}

impl AgentFedAuth {
    #[must_use]
    pub fn new(provider: Box<dyn AgentFedCredentialProvider>) -> Self {
        Self {
            provider,
            cached_credential: None,
            generation: 0,
            rejected_generation: None,
        }
    }

    fn try_accept_snapshot(
        &mut self,
        snapshot: Arc<AgentFedCredentialSnapshot>,
        events: &HttpClientAuthProviderEvents,
    ) -> Result<bool, ()> {
        if let Some(cached) = &self.cached_credential
            && Arc::ptr_eq(&cached.snapshot, &snapshot)
        {
            if self.rejected_generation == Some(cached.generation) {
                // Credentials returned were already rejected. Return Err to signal caller to retry.
                return Err(());
            }
            // Continue using the cached credential
            return Ok(false);
        }

        let token = snapshot.token();
        if token.expose_token().trim().is_empty() {
            events.emit_invalid(self, "Malformed token: Empty");
            // Keep using the previously cached token (if any)
            return Ok(false);
        }

        match HeaderValue::from_str(&format!("Bearer {}", token.expose_token())) {
            Ok(mut header) => {
                header.set_sensitive(true);
                let expires_on = token.expires_on();
                self.generation = self.generation.wrapping_add(1);
                self.cached_credential = Some(CachedCredential {
                    snapshot,
                    header,
                    expires_on,
                    generation: self.generation,
                });
                Ok(true)
            }
            Err(e) => {
                // Keep using the previously cached token (if any)
                events.emit_invalid(self, &format!("Malformed token: {e}"));
                Ok(false)
            }
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for AgentFedAuth {
    fn name(&self) -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn is_active(&self) -> bool {
        true
    }

    fn is_ready(&self) -> bool {
        self.cached_credential.as_ref().is_some_and(|credential| {
            self.rejected_generation != Some(credential.generation)
                && credential
                    .expires_on
                    .is_none_or(|expires_on| expires_on > Instant::now() + TOKEN_USABLE_MARGIN)
        })
    }

    fn not_ready_reason(&self) -> &'static str {
        match self.cached_credential.as_ref() {
            Some(credential) if self.rejected_generation == Some(credential.generation) => {
                "agent-fed bearer token was rejected; awaiting a different snapshot"
            }
            Some(credential)
                if credential.expires_on.is_some_and(|expires_on| {
                    expires_on <= Instant::now() + TOKEN_USABLE_MARGIN
                }) =>
            {
                "agent-fed bearer token at/near expiry; awaiting refresh"
            }
            Some(_) => "agent-fed credential refresh pending",
            None => "agent-fed bearer token unavailable",
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_credential.as_ref().map(|credential| {
            (
                http::header::AUTHORIZATION,
                credential.header.clone(),
                credential.generation,
            )
        })
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_credential
            .as_ref()
            .and_then(|credential| credential.expires_on)
            .and_then(|expires_on| expires_on.checked_sub(TOKEN_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if self
            .cached_credential
            .as_ref()
            .is_some_and(|credential| credential.generation == generation)
        {
            self.rejected_generation = Some(generation);
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) -> bool {
        let mut consecutive_failures = 0;
        loop {
            match self.provider.get_credential().await {
                Ok(credential) => {
                    match self.try_accept_snapshot(credential, events) {
                        Err(_) => {
                            // Previously rejected credential encountered. Need to
                            // wait for a new credential to arrive.

                            events.emit_retry(
                                self,
                                "A previously rejected credential was retrieved; operation will be retried",
                                consecutive_failures
                            );

                            let backoff =
                                jittered_backoff(retry_backoff_secs(consecutive_failures));

                            tokio::time::sleep(backoff).await;

                            consecutive_failures += 1;

                            continue;
                        }
                        Ok(r) => {
                            return r;
                        }
                    }
                }
                Err(e) => {
                    // Retrieval error: Keep using the last cached token (if any).
                    events.emit_error(self, &format!("Error retrieving token: {e}"));
                    return false;
                }
            }
        }
    }
}

/// Base (un-jittered) backoff before retrying after a failed acquisition.
///
/// Grows exponentially with the number of consecutive prior failures, from
/// `TOKEN_REFRESH_RETRY_SECS` up to `MAX_TOKEN_REFRESH_RETRY_SECS`, so a
/// sustained token-endpoint outage settles into infrequent retries instead of a
/// tight loop.
fn retry_backoff_secs(consecutive_failures: u32) -> u64 {
    // Cap the shift so `1 << shift` cannot overflow; the value is clamped to the
    // max below long before the shift approaches that bound.
    let shift = consecutive_failures.min(16);
    TOKEN_REFRESH_RETRY_SECS
        .saturating_mul(1u64 << shift)
        .min(MAX_TOKEN_REFRESH_RETRY_SECS)
}

/// Applies "equal jitter" to a backoff: half the delay is a fixed floor and the
/// other half is randomized, yielding a delay in `[base/2, base]`. This keeps
/// per-core extensions from retrying in lockstep during an outage.
fn jittered_backoff(base_secs: u64) -> Duration {
    let half = base_secs / 2;
    let jitter = if half == 0 {
        0
    } else {
        rand::rng().random_range(0..=half)
    };
    Duration::from_secs(half + jitter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::agent_fed_credential_provider::{
        AgentFedCredentialProvider as AgentFedCredentialProviderCap, AgentFedCredentialSnapshot,
    };
    use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
    use serde_json::Map;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct MockProvider {
        snapshots: Mutex<VecDeque<Arc<AgentFedCredentialSnapshot>>>,
        delay: Duration,
    }

    impl MockProvider {
        fn with_snapshots(
            snapshots: impl IntoIterator<Item = Arc<AgentFedCredentialSnapshot>>,
        ) -> Self {
            Self {
                snapshots: Mutex::new(snapshots.into_iter().collect()),
                delay: Duration::ZERO,
            }
        }

        fn with_tokens(tokens: impl IntoIterator<Item = BearerToken>) -> Self {
            Self::with_snapshots(tokens.into_iter().map(|token| {
                Arc::new(AgentFedCredentialSnapshot::new(token, Arc::new(Map::new())))
            }))
        }
    }

    #[async_trait(?Send)]
    impl AgentFedCredentialProvider for MockProvider {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
            let mut snapshots = self.snapshots.lock().expect("mock snapshots");
            if snapshots.len() > 1 {
                Ok(snapshots.pop_front().expect("snapshot"))
            } else {
                snapshots.front().cloned().map_or_else(
                    || {
                        Err(CapabilityErrorSource::<AgentFedCredentialProviderCap>::new(
                            "mock-agent".into(),
                        )
                        .error("no snapshot"))
                    },
                    Ok,
                )
            }
        }
    }

    fn snapshot(token: &str) -> Arc<AgentFedCredentialSnapshot> {
        Arc::new(AgentFedCredentialSnapshot::new(
            BearerToken::without_expiry(token.to_owned()),
            Arc::new(Map::new()),
        ))
    }

    thread_local! {
        /// Number of `invalid` notifications raised on this test thread.
        static INVALID: Cell<usize> = const { Cell::new(0) };
        /// Number of `error` notifications raised on this test thread.
        static ERROR: Cell<usize> = const { Cell::new(0) };
    }

    const TEST_EVENTS: HttpClientAuthProviderEvents = HttpClientAuthProviderEvents {
        invalid: |_, _| INVALID.set(INVALID.get() + 1),
        error: |_, _| ERROR.set(ERROR.get() + 1),
        retry: |_, _, _| {},
        stream_closed: |_| {},
    };

    fn reset_events() {
        INVALID.set(0);
        ERROR.set(0);
    }

    /// Scenario: The provider returns the same published snapshot for consecutive requests.
    /// Guarantees: The validated header and local request generation are reused.
    #[tokio::test]
    async fn reuses_unchanged_snapshot() {
        let snapshot = snapshot("same");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            Arc::clone(&snapshot),
            snapshot,
        ])));

        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let first = auth.header().unwrap();
        assert!(!auth.poll_refresh(&TEST_EVENTS).await);
        let second = auth.header().unwrap();

        assert_eq!(first, second);
        assert_eq!(first.2, 1);
    }

    /// Scenario: HTTP 401 rejects the current snapshot and the provider has not rotated it.
    /// Guarantees: The rejected header is not returned again and lookup remains backpressured.
    #[tokio::test]
    async fn does_not_reuse_rejected_unchanged_snapshot() {
        let snapshot = snapshot("rejected");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            Arc::clone(&snapshot),
            Arc::clone(&snapshot),
        ])));
        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let (_, _, generation) = auth.header().unwrap();

        auth.invalidate(generation);
        assert!(auth.header().is_some());
        assert!(!auth.is_ready());

        assert!(auth.try_accept_snapshot(snapshot, &TEST_EVENTS).is_err());
    }

    /// Scenario: The provider publishes a different snapshot after the current one is rejected.
    /// Guarantees: The new header receives a new generation and resumes authentication.
    #[tokio::test]
    async fn accepts_rotated_snapshot_after_rejection() {
        let first = snapshot("first");
        let second = snapshot("second");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([first, second])));
        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let (_, _, generation) = auth.header().unwrap();
        auth.invalidate(generation);

        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let (_, header, next_generation) = auth.header().unwrap();

        assert_eq!(header, "Bearer second");
        assert_eq!(next_generation, generation + 1);
    }

    /// Scenario: An older request returns HTTP 401 after a replacement snapshot is cached.
    /// Guarantees: Rejecting the stale generation leaves the newer credential ready.
    #[tokio::test]
    async fn ignores_rejection_for_superseded_snapshot() {
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            snapshot("first"),
            snapshot("second"),
        ])));
        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let (_, _, old_generation) = auth.header().unwrap();
        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let new_generation = auth
            .cached_credential
            .as_ref()
            .expect("new credential")
            .generation;

        auth.invalidate(old_generation);

        assert!(auth.is_ready());
        assert_eq!(auth.header().unwrap().2, new_generation);
    }

    /// Scenario: A cached credential crosses its usability margin while no export is attempted.
    /// Guarantees: The stale header is no longer ready and cannot be used for a request.
    #[tokio::test]
    async fn expires_cached_credential_while_idle() {
        let expires_on = Instant::now() + TOKEN_USABLE_MARGIN + Duration::from_secs(1);
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([
            BearerToken::with_expiry("expiring".to_owned(), Some(expires_on)),
        ])));

        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        assert!(auth.is_ready());
        assert_eq!(
            auth.refresh_deadline(),
            expires_on.checked_sub(TOKEN_USABLE_MARGIN)
        );

        tokio::time::sleep(Duration::from_millis(1100)).await;

        assert!(!auth.is_ready());
        assert!(auth.refresh_deadline().is_none());
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token at/near expiry; awaiting refresh"
        );
        assert!(auth.header().is_some());
    }

    /// Scenario: The host supplies an empty, near-expiry, or malformed token.
    /// Guarantees: Each unsafe token maps to a bounded failure reason and no header is produced.
    #[tokio::test]
    async fn rejects_invalid_tokens() {
        let cases = [
            (BearerToken::without_expiry("  ".to_owned()), false),
            (
                BearerToken::with_expiry(
                    "near-expiry".to_owned(),
                    Some(Instant::now() + TOKEN_USABLE_MARGIN),
                ),
                true,
            ),
            (
                BearerToken::without_expiry("bad\r\ntoken".to_owned()),
                false,
            ),
        ];

        for (token, expected) in cases {
            let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([token])));
            assert_eq!(auth.poll_refresh(&TEST_EVENTS).await, expected);
        }
    }

    /// Scenario: The agent-fed provider reports that no credential is available.
    /// Guarantees: The capability failure maps to the bounded unavailable reason.
    #[tokio::test]
    async fn surfaces_unavailable_credentials() {
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([])));

        reset_events();

        assert!(!auth.poll_refresh(&TEST_EVENTS).await);

        assert_eq!(ERROR.get(), 1);
    }

    /// Scenario: Agent-fed readiness changes from unavailable to rejected.
    /// Guarantees: Each operator-facing not-ready reason identifies the current state.
    #[tokio::test]
    async fn reports_each_not_ready_reason() {
        let snapshot = snapshot("token");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            Arc::clone(&snapshot),
            snapshot,
        ])));
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token unavailable"
        );

        assert!(auth.poll_refresh(&TEST_EVENTS).await);
        let (_, _, generation) = auth.header().unwrap();
        auth.invalidate(generation);
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token was rejected; awaiting a different snapshot"
        );
    }
}
