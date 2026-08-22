// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bearer-header adapter for agent-fed credentials.

use std::time::{Duration, Instant};

use http::HeaderValue;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;
use otel_arrow_dfe_engine::local::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider;

const CREDENTIAL_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);
const CREDENTIAL_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Resolves and validates agent-fed credentials for an outbound request.
pub(crate) struct AgentFedAuth {
    provider: Box<dyn AgentFedCredentialProvider>,
    cached_credential: Option<CachedCredential>,
    retry_at: Instant,
}

#[derive(Debug)]
struct CachedCredential {
    header: HeaderValue,
    expires_on: Option<Instant>,
}

impl AgentFedAuth {
    /// Creates an adapter over a resolved local capability.
    pub(crate) fn new(provider: Box<dyn AgentFedCredentialProvider>) -> Self {
        Self {
            provider,
            cached_credential: None,
            retry_at: Instant::now(),
        }
    }

    /// Whether a validated header is ready for one export attempt.
    pub(crate) fn is_ready(&self) -> bool {
        self.cached_credential.as_ref().is_some_and(|credential| {
            credential
                .expires_on
                .is_none_or(|expires_on| expires_on > Instant::now() + TOKEN_USABLE_MARGIN)
        })
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

    /// Waits until the next allowed lookup, then caches one current credential.
    ///
    /// The caller drives this future from its main `select!`, so control traffic
    /// can cancel a slow capability lookup. A failure is delayed before retrying
    /// to avoid a busy loop against an unavailable host provider.
    pub(crate) async fn poll_credential(&mut self) -> Result<(), AgentFedAuthError> {
        tokio::time::sleep_until(tokio::time::Instant::from_std(self.retry_at)).await;
        match self.header_with_timeout(CREDENTIAL_LOOKUP_TIMEOUT).await {
            Ok(credential) => {
                self.cached_credential = Some(credential);
                Ok(())
            }
            Err(error) => {
                self.retry_at = Instant::now() + CREDENTIAL_RETRY_DELAY;
                Err(error)
            }
        }
    }

    /// Takes the credential reserved for the next export attempt.
    pub(crate) fn take_header(&mut self) -> Option<HeaderValue> {
        if !self.is_ready() {
            self.cached_credential = None;
            return None;
        }
        self.cached_credential
            .take()
            .map(|credential| credential.header)
    }

    async fn header_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<CachedCredential, AgentFedAuthError> {
        let snapshot = tokio::time::timeout(timeout, self.provider.get_credential())
            .await
            .map_err(|_| AgentFedAuthError::LookupTimeout)?
            .map_err(AgentFedAuthError::CredentialUnavailable)?;
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
        Ok(CachedCredential {
            header,
            expires_on: token.expires_on(),
        })
    }
}

/// Failure to obtain a safe bearer header from an agent-fed snapshot.
#[derive(Debug, thiserror::Error)]
pub(crate) enum AgentFedAuthError {
    /// The capability could not provide a snapshot.
    #[error("agent-fed credential is unavailable: {0}")]
    CredentialUnavailable(CapabilityError),
    /// The bounded capability lookup did not complete.
    #[error("agent-fed credential lookup timed out")]
    LookupTimeout,
    /// The host supplied no usable secret.
    #[error("agent-fed bearer token is empty or whitespace")]
    EmptyToken,
    /// The token cannot safely cover a request.
    #[error("agent-fed bearer token is expired or near expiry")]
    TokenNearExpiry,
    /// The token cannot be represented as an HTTP header.
    #[error("agent-fed bearer token is not a valid HTTP header value: {0}")]
    InvalidToken(http::header::InvalidHeaderValue),
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
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    struct MockProvider {
        snapshots: Mutex<VecDeque<Result<Arc<AgentFedCredentialSnapshot>, CapabilityError>>>,
        delay: Duration,
    }

    impl MockProvider {
        fn with_tokens(tokens: impl IntoIterator<Item = BearerToken>) -> Self {
            Self {
                snapshots: Mutex::new(
                    tokens
                        .into_iter()
                        .map(|token| {
                            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                                token,
                                Arc::new(Map::new()),
                            )))
                        })
                        .collect(),
                ),
                delay: Duration::ZERO,
            }
        }
    }

    #[async_trait(?Send)]
    impl AgentFedCredentialProvider for MockProvider {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
            self.snapshots
                .lock()
                .expect("mock snapshots")
                .pop_front()
                .unwrap_or_else(|| {
                    Err(CapabilityErrorSource::<AgentFedCredentialProviderCap>::new(
                        "mock-agent".into(),
                    )
                    .error("no snapshot"))
                })
        }
    }

    /// Scenario: The host rotates its agent-fed token between export attempts.
    /// Guarantees: Each header uses the token from the latest snapshot lookup.
    #[tokio::test]
    async fn reads_each_token_rotation() {
        let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([
            BearerToken::without_expiry("first".to_owned()),
            BearerToken::without_expiry("second".to_owned()),
        ])));

        auth.poll_credential().await.unwrap();
        assert_eq!(auth.take_header().unwrap(), "Bearer first");
        auth.poll_credential().await.unwrap();
        assert_eq!(auth.take_header().unwrap(), "Bearer second");
    }

    /// Scenario: A cached credential crosses its usability margin while no export is attempted.
    /// Guarantees: The stale header is no longer ready and cannot be taken for a request.
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
        assert!(auth.take_header().is_none());
    }

    /// Scenario: The host supplies an empty, near-expiry, or malformed token.
    /// Guarantees: No unsafe Authorization header is produced.
    #[tokio::test]
    async fn rejects_invalid_tokens() {
        let cases = [
            BearerToken::without_expiry("  ".to_owned()),
            BearerToken::with_expiry(
                "near-expiry".to_owned(),
                Some(Instant::now() + TOKEN_USABLE_MARGIN),
            ),
            BearerToken::without_expiry("bad\r\ntoken".to_owned()),
        ];

        for token in cases {
            let mut auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([token])));
            assert!(auth.poll_credential().await.is_err());
        }
    }

    /// Scenario: An agent-fed provider does not complete within the lookup deadline.
    /// Guarantees: Credential resolution fails with a bounded timeout.
    #[tokio::test]
    async fn bounds_credential_lookup() {
        let auth = AgentFedAuth::new(Box::new(MockProvider {
            snapshots: Mutex::new(VecDeque::new()),
            delay: Duration::from_millis(50),
        }));

        let error = auth
            .header_with_timeout(Duration::from_millis(1))
            .await
            .unwrap_err();
        assert!(matches!(error, AgentFedAuthError::LookupTimeout));
    }

    /// Scenario: The agent-fed provider reports that no credential is available.
    /// Guarantees: The capability error is surfaced without producing a header.
    #[tokio::test]
    async fn surfaces_unavailable_credentials() {
        let auth = AgentFedAuth::new(Box::new(MockProvider::with_tokens([])));

        let error = auth
            .header_with_timeout(CREDENTIAL_LOOKUP_TIMEOUT)
            .await
            .unwrap_err();
        assert!(matches!(error, AgentFedAuthError::CredentialUnavailable(_)));
    }
}
