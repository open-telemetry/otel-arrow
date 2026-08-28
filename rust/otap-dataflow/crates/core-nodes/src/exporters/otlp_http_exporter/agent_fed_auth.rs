// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer-header adapter for agent-fed credentials.

use std::sync::Arc;
use std::time::{Duration, Instant};

use http::HeaderValue;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::agent_fed_credential_provider::AgentFedCredentialSnapshot;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;
use otel_arrow_dfe_engine::local::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider;
use otel_arrow_dfe_telemetry_macros::AttributeEnum;

pub(crate) const CREDENTIAL_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const CREDENTIAL_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Resolves, validates, and reuses agent-fed credentials for outbound requests.
pub(crate) struct AgentFedAuth {
    provider: Box<dyn AgentFedCredentialProvider>,
    cached_credential: Option<CachedCredential>,
    generation: u64,
    rejected_generation: Option<u64>,
    lookup_required: bool,
    retry_at: Instant,
    failure_limiter: FailureLogLimiter,
}

#[derive(Debug)]
struct CachedCredential {
    snapshot: Arc<AgentFedCredentialSnapshot>,
    header: HeaderValue,
    expires_on: Option<Instant>,
    generation: u64,
}

impl AgentFedAuth {
    /// Creates an adapter over a resolved local capability.
    pub(crate) fn new(provider: Box<dyn AgentFedCredentialProvider>) -> Self {
        Self {
            provider,
            cached_credential: None,
            generation: 0,
            rejected_generation: None,
            lookup_required: true,
            retry_at: Instant::now(),
            failure_limiter: FailureLogLimiter::default(),
        }
    }

    /// Whether a validated, non-rejected header is ready.
    pub(crate) fn is_ready(&self) -> bool {
        !self.lookup_required
            && self.cached_credential.as_ref().is_some_and(|credential| {
                self.rejected_generation != Some(credential.generation)
                    && credential
                        .expires_on
                        .is_none_or(|expires_on| expires_on > Instant::now() + TOKEN_USABLE_MARGIN)
            })
    }

    /// Whether the provider must be checked before another request is admitted.
    pub(crate) fn should_poll(&self) -> bool {
        self.lookup_required || !self.is_ready()
    }

    /// Requires a fresh provider check before the pending batch can be sent.
    pub(crate) fn require_request_check(&mut self) {
        self.lookup_required = true;
    }

    /// When the cached credential crosses the shared usability margin.
    pub(crate) fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_credential
            .as_ref()
            .and_then(|credential| credential.expires_on)
            .and_then(|expires_on| expires_on.checked_sub(TOKEN_USABLE_MARGIN))
    }

    /// Waits until the next allowed lookup and observes the current snapshot.
    ///
    /// The caller drives this future from its main `select!`, so control traffic
    /// can cancel a slow capability lookup. Unchanged snapshots reuse their
    /// validated header; failures are delayed to avoid a busy loop.
    pub(crate) async fn poll_credential(&mut self) -> Result<(), AgentFedAuthFailure> {
        tokio::time::sleep_until(tokio::time::Instant::from_std(self.retry_at)).await;
        match self.lookup_with_timeout(CREDENTIAL_LOOKUP_TIMEOUT).await {
            Ok(snapshot) => self.accept_snapshot(snapshot),
            Err(error) => Err(self.record_failure(error)),
        }
    }

    /// Clones the cached header and identifies the snapshot generation used.
    pub(crate) fn header(&mut self) -> Option<(HeaderValue, u64)> {
        if !self.is_ready() {
            return None;
        }
        self.cached_credential
            .as_ref()
            .map(|credential| (credential.header.clone(), credential.generation))
    }

    /// Rejects the current snapshot generation after an HTTP 401.
    pub(crate) fn invalidate(&mut self, generation: u64) {
        if self
            .cached_credential
            .as_ref()
            .is_some_and(|credential| credential.generation == generation)
        {
            self.rejected_generation = Some(generation);
            self.lookup_required = true;
        }
    }

    /// Reason input cannot currently be authenticated.
    pub(crate) fn not_ready_reason(&self) -> &'static str {
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
            Some(_) => "awaiting agent-fed credential snapshot check",
            None => "agent-fed bearer token unavailable",
        }
    }

    fn accept_snapshot(
        &mut self,
        snapshot: Arc<AgentFedCredentialSnapshot>,
    ) -> Result<(), AgentFedAuthFailure> {
        if let Some(cached) = &self.cached_credential {
            if Arc::ptr_eq(&cached.snapshot, &snapshot) {
                if self.rejected_generation == Some(cached.generation) {
                    return Err(self.record_failure(AgentFedAuthError::RejectedCredentialUnchanged));
                }
                if cached
                    .expires_on
                    .is_some_and(|expires_on| expires_on <= Instant::now() + TOKEN_USABLE_MARGIN)
                {
                    return Err(self.record_failure(AgentFedAuthError::TokenNearExpiry));
                }
                self.lookup_required = false;
                self.failure_limiter.record_success();
                return Ok(());
            }
        }

        let credential = self
            .validate_snapshot(snapshot)
            .map_err(|error| self.record_failure(error))?;
        self.cached_credential = Some(credential);
        self.lookup_required = false;
        self.failure_limiter.record_success();
        Ok(())
    }

    fn validate_snapshot(
        &mut self,
        snapshot: Arc<AgentFedCredentialSnapshot>,
    ) -> Result<CachedCredential, AgentFedAuthError> {
        let token = snapshot.token();
        if token.expose_token().trim().is_empty() {
            return Err(AgentFedAuthError::EmptyToken);
        }
        if token
            .expires_on()
            .is_some_and(|expires_on| expires_on <= Instant::now() + TOKEN_USABLE_MARGIN)
        {
            return Err(AgentFedAuthError::TokenNearExpiry);
        }

        let mut header = HeaderValue::from_str(&format!("Bearer {}", token.expose_token()))
            .map_err(AgentFedAuthError::InvalidToken)?;
        header.set_sensitive(true);
        let expires_on = token.expires_on();
        self.generation = self.generation.wrapping_add(1);
        Ok(CachedCredential {
            snapshot,
            header,
            expires_on,
            generation: self.generation,
        })
    }

    async fn lookup_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Arc<AgentFedCredentialSnapshot>, AgentFedAuthError> {
        tokio::time::timeout(timeout, self.provider.get_credential())
            .await
            .map_err(|_| AgentFedAuthError::LookupTimeout)?
            .map_err(AgentFedAuthError::CredentialUnavailable)
    }

    fn record_failure(&mut self, error: AgentFedAuthError) -> AgentFedAuthFailure {
        self.lookup_required = true;
        self.retry_at = Instant::now() + CREDENTIAL_RETRY_DELAY;
        AgentFedAuthFailure {
            error,
            log_count: self.failure_limiter.record_failure(),
        }
    }
}

#[derive(Debug, Default)]
struct FailureLogLimiter {
    consecutive_failures: u64,
}

impl FailureLogLimiter {
    fn record_success(&mut self) {
        self.consecutive_failures = 0;
    }

    fn record_failure(&mut self) -> Option<u64> {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.consecutive_failures
            .is_power_of_two()
            .then_some(self.consecutive_failures)
    }
}

/// One failed agent-fed lookup and its warning-sampling decision.
#[derive(Debug)]
pub(crate) struct AgentFedAuthFailure {
    error: AgentFedAuthError,
    log_count: Option<u64>,
}

impl AgentFedAuthFailure {
    pub(crate) fn error_type(&self) -> AgentFedAuthErrorType {
        self.error.error_type()
    }

    pub(crate) fn log_count(&self) -> Option<u64> {
        self.log_count
    }
}

impl std::fmt::Display for AgentFedAuthFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

/// Bounded reason for an agent-fed credential lookup failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(crate) enum AgentFedAuthErrorType {
    CredentialUnavailable,
    LookupTimeout,
    EmptyToken,
    TokenNearExpiry,
    InvalidToken,
    RejectedCredentialUnchanged,
}

/// Failure to obtain a safe bearer header from an agent-fed snapshot.
#[derive(Debug, thiserror::Error)]
enum AgentFedAuthError {
    #[error("agent-fed credential is unavailable: {0}")]
    CredentialUnavailable(CapabilityError),
    #[error("agent-fed credential lookup timed out")]
    LookupTimeout,
    #[error("agent-fed bearer token is empty or whitespace")]
    EmptyToken,
    #[error("agent-fed bearer token is expired or near expiry")]
    TokenNearExpiry,
    #[error("agent-fed bearer token is not a valid HTTP header value: {0}")]
    InvalidToken(http::header::InvalidHeaderValue),
    #[error("agent-fed bearer token was rejected and the provider snapshot is unchanged")]
    RejectedCredentialUnchanged,
}

impl AgentFedAuthError {
    fn error_type(&self) -> AgentFedAuthErrorType {
        match self {
            Self::CredentialUnavailable(_) => AgentFedAuthErrorType::CredentialUnavailable,
            Self::LookupTimeout => AgentFedAuthErrorType::LookupTimeout,
            Self::EmptyToken => AgentFedAuthErrorType::EmptyToken,
            Self::TokenNearExpiry => AgentFedAuthErrorType::TokenNearExpiry,
            Self::InvalidToken(_) => AgentFedAuthErrorType::InvalidToken,
            Self::RejectedCredentialUnchanged => AgentFedAuthErrorType::RejectedCredentialUnchanged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use otel_arrow_dfe_engine::capability::CapabilityErrorSource;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::agent_fed_credential_provider::{
        AgentFedCredentialProvider as AgentFedCredentialProviderCap, AgentFedCredentialSnapshot,
    };
    use serde_json::Map;
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

    /// Scenario: The provider returns the same published snapshot for consecutive requests.
    /// Guarantees: The validated header and local request generation are reused.
    #[tokio::test]
    async fn reuses_unchanged_snapshot() {
        let snapshot = snapshot("same");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            Arc::clone(&snapshot),
            snapshot,
        ])));

        auth.poll_credential().await.unwrap();
        let first = auth.header().unwrap();
        auth.poll_credential().await.unwrap();
        let second = auth.header().unwrap();

        assert_eq!(first, second);
        assert_eq!(first.1, 1);
    }

    /// Scenario: HTTP 401 rejects the current snapshot and the provider has not rotated it.
    /// Guarantees: The rejected header is not returned again and lookup remains backpressured.
    #[tokio::test]
    async fn does_not_reuse_rejected_unchanged_snapshot() {
        let snapshot = snapshot("rejected");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([
            Arc::clone(&snapshot),
            snapshot,
        ])));
        auth.poll_credential().await.unwrap();
        let (_, generation) = auth.header().unwrap();

        auth.invalidate(generation);
        assert!(auth.header().is_none());
        let failure = auth.poll_credential().await.unwrap_err();

        assert_eq!(
            failure.error_type(),
            AgentFedAuthErrorType::RejectedCredentialUnchanged
        );
        assert!(!auth.is_ready());
    }

    /// Scenario: The provider publishes a different snapshot after the current one is rejected.
    /// Guarantees: The new header receives a new generation and resumes authentication.
    #[tokio::test]
    async fn accepts_rotated_snapshot_after_rejection() {
        let first = snapshot("first");
        let second = snapshot("second");
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_snapshots([first, second])));
        auth.poll_credential().await.unwrap();
        let (_, generation) = auth.header().unwrap();
        auth.invalidate(generation);

        auth.poll_credential().await.unwrap();
        let (header, next_generation) = auth.header().unwrap();

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
        auth.poll_credential().await.unwrap();
        let (_, old_generation) = auth.header().unwrap();
        auth.poll_credential().await.unwrap();
        let new_generation = auth
            .cached_credential
            .as_ref()
            .expect("new credential")
            .generation;

        auth.invalidate(old_generation);

        assert!(auth.is_ready());
        assert_eq!(auth.header().unwrap().1, new_generation);
    }

    /// Scenario: A cached credential crosses its usability margin while no export is attempted.
    /// Guarantees: The stale header is no longer ready and cannot be used for a request.
    #[tokio::test]
    async fn expires_cached_credential_while_idle() {
        let expires_on = Instant::now() + TOKEN_USABLE_MARGIN + Duration::from_secs(1);
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([
            BearerToken::with_expiry("expiring".to_owned(), Some(expires_on)),
        ])));

        auth.poll_credential().await.unwrap();
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
        assert!(auth.header().is_none());
        assert_eq!(
            auth.poll_credential().await.unwrap_err().error_type(),
            AgentFedAuthErrorType::TokenNearExpiry
        );
    }

    /// Scenario: The host supplies an empty, near-expiry, or malformed token.
    /// Guarantees: Each unsafe token maps to a bounded failure reason and no header is produced.
    #[tokio::test]
    async fn rejects_invalid_tokens() {
        let cases = [
            (
                BearerToken::without_expiry("  ".to_owned()),
                AgentFedAuthErrorType::EmptyToken,
            ),
            (
                BearerToken::with_expiry(
                    "near-expiry".to_owned(),
                    Some(Instant::now() + TOKEN_USABLE_MARGIN),
                ),
                AgentFedAuthErrorType::TokenNearExpiry,
            ),
            (
                BearerToken::without_expiry("bad\r\ntoken".to_owned()),
                AgentFedAuthErrorType::InvalidToken,
            ),
        ];

        for (token, expected) in cases {
            let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([token])));
            assert_eq!(
                auth.poll_credential().await.unwrap_err().error_type(),
                expected
            );
        }
    }

    /// Scenario: The agent-fed provider does not complete within the lookup deadline.
    /// Guarantees: Credential resolution fails with a bounded timeout reason.
    #[tokio::test]
    async fn bounds_credential_lookup() {
        let auth = AgentFedAuth::new(Box::new(MockProvider {
            snapshots: Mutex::new(VecDeque::new()),
            delay: Duration::from_millis(50),
        }));

        let error = auth
            .lookup_with_timeout(Duration::from_millis(1))
            .await
            .unwrap_err();
        assert!(matches!(error, AgentFedAuthError::LookupTimeout));
        assert_eq!(error.error_type(), AgentFedAuthErrorType::LookupTimeout);
    }

    /// Scenario: The agent-fed provider reports that no credential is available.
    /// Guarantees: The capability failure maps to the bounded unavailable reason.
    #[tokio::test]
    async fn surfaces_unavailable_credentials() {
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([])));

        let failure = auth.poll_credential().await.unwrap_err();

        assert_eq!(
            failure.error_type(),
            AgentFedAuthErrorType::CredentialUnavailable
        );
        assert_eq!(failure.log_count(), Some(1));
        assert!(failure.to_string().contains("credential is unavailable"));
    }

    /// Scenario: Agent-fed readiness changes from unavailable to checking to rejected.
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

        auth.poll_credential().await.unwrap();
        let (_, generation) = auth.header().unwrap();
        auth.require_request_check();
        assert_eq!(
            auth.not_ready_reason(),
            "awaiting agent-fed credential snapshot check"
        );

        auth.invalidate(generation);
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token was rejected; awaiting a different snapshot"
        );
    }

    /// Scenario: Credential failures continue, recover, and fail again.
    /// Guarantees: Warning sampling follows powers of two and resets after success.
    #[test]
    fn failure_log_limiter_uses_exponential_sampling_and_resets() {
        let mut limiter = FailureLogLimiter::default();
        assert_eq!(limiter.record_failure(), Some(1));
        assert_eq!(limiter.record_failure(), Some(2));
        assert_eq!(limiter.record_failure(), None);
        assert_eq!(limiter.record_failure(), Some(4));
        limiter.record_success();
        assert_eq!(limiter.record_failure(), Some(1));
    }
}
