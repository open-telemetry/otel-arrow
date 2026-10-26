// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Adapter from the engine bearer-token capability to Geneva GCS authentication.

use geneva_uploader::{GcsTokenFuture, GcsTokenProvider, GcsTokenProviderError};
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider;
use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};

const TOKEN_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Supplies the uploader from one shared bearer-token capability binding.
pub(crate) struct GenevaGcsTokenProvider {
    provider: Box<dyn BearerTokenProvider>,
    expected_resource: String,
}

impl GenevaGcsTokenProvider {
    pub(crate) fn new(provider: Box<dyn BearerTokenProvider>, expected_resource: String) -> Self {
        Self {
            provider,
            expected_resource,
        }
    }

    fn error(kind: ErrorKind, message: impl Into<String>) -> GcsTokenProviderError {
        Box::new(Error::new(kind, message.into()))
    }
}

impl std::fmt::Debug for GenevaGcsTokenProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenevaGcsTokenProvider")
            .field("expected_resource", &self.expected_resource)
            .finish_non_exhaustive()
    }
}

impl GcsTokenProvider for GenevaGcsTokenProvider {
    fn get_token<'a>(&'a self, resource: &'a str) -> GcsTokenFuture<'a> {
        Box::pin(async move {
            // This protects the adapter contract if the uploader requests a
            // different resource later. The capability is scope-preconfigured
            // and does not expose that scope, so GCS remains authoritative for
            // validating the token's actual audience.
            if resource != self.expected_resource {
                return Err(Self::error(
                    ErrorKind::InvalidInput,
                    format!(
                        "Geneva requested Entra resource '{resource}', but the provider is configured for '{}'",
                        self.expected_resource
                    ),
                ));
            }

            let token = tokio::time::timeout(TOKEN_LOOKUP_TIMEOUT, self.provider.get_token())
                .await
                .map_err(|_| Self::error(ErrorKind::TimedOut, "bearer token lookup timed out"))?
                .map_err(|error| -> GcsTokenProviderError { Box::new(error) })?;
            if token.expose_token().trim().is_empty() {
                return Err(Self::error(
                    ErrorKind::InvalidData,
                    "bearer token provider returned an empty token",
                ));
            }
            if token
                .expires_on()
                .is_some_and(|expiry| expiry <= Instant::now() + TOKEN_USABLE_MARGIN)
            {
                return Err(Self::error(
                    ErrorKind::InvalidData,
                    "bearer token provider returned an expired or near-expiry token",
                ));
            }

            Ok(token.expose_token().to_owned().into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::stream;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider as BearerTokenProviderCap;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TokenStream;
    use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestProvider {
        token: BearerToken,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl BearerTokenProvider for TestProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            let _ = self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.token.clone())
        }

        fn token_stream(&self) -> TokenStream {
            Box::pin(stream::empty())
        }
    }

    fn provider(token: BearerToken) -> (GenevaGcsTokenProvider, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            GenevaGcsTokenProvider::new(
                Box::new(TestProvider {
                    token,
                    calls: Arc::clone(&calls),
                }),
                "https://monitor.azure.com".to_string(),
            ),
            calls,
        )
    }

    struct FailingProvider;

    #[async_trait]
    impl BearerTokenProvider for FailingProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            Err(
                CapabilityErrorSource::<BearerTokenProviderCap>::new("azure_identity".into())
                    .error(Error::new(
                        ErrorKind::PermissionDenied,
                        "Arc challenge file could not be read",
                    )),
            )
        }

        fn token_stream(&self) -> TokenStream {
            Box::pin(stream::empty())
        }
    }

    /// Scenario: Token acquisition fails with a capability error wrapping an I/O failure.
    /// Guarantees: The returned error preserves provider identity and the original source error.
    #[tokio::test]
    async fn preserves_capability_error_and_source() {
        let provider = GenevaGcsTokenProvider::new(
            Box::new(FailingProvider),
            "https://monitor.azure.com".to_string(),
        );

        let error = provider
            .get_token("https://monitor.azure.com")
            .await
            .expect_err("provider failure must propagate");

        let capability_error = error
            .downcast_ref::<CapabilityError>()
            .expect("original capability error");
        assert_eq!(capability_error.extension, "azure_identity");
        assert_eq!(capability_error.capability, "bearer_token_provider");
        let source = error.source().expect("acquisition failure");
        let io_error = source.downcast_ref::<Error>().expect("original I/O error");
        assert_eq!(io_error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(io_error.to_string(), "Arc challenge file could not be read");
    }

    struct PendingThenReadyProvider(AtomicUsize);

    #[async_trait]
    impl BearerTokenProvider for PendingThenReadyProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
                return std::future::pending().await;
            }
            Ok(BearerToken::without_expiry("recovered-token".to_string()))
        }

        fn token_stream(&self) -> TokenStream {
            Box::pin(stream::empty())
        }
    }

    /// Scenario: The bound provider never completes its first token lookup.
    /// Guarantees: Lookup fails after five seconds and a later lookup can recover.
    #[tokio::test(start_paused = true)]
    async fn times_out_stuck_provider_and_recovers() {
        let provider = GenevaGcsTokenProvider::new(
            Box::new(PendingThenReadyProvider(AtomicUsize::new(0))),
            "https://monitor.azure.com".to_string(),
        );
        let started = tokio::time::Instant::now();

        let error = provider
            .get_token("https://monitor.azure.com")
            .await
            .expect_err("stuck provider must time out");

        assert_eq!(started.elapsed(), Duration::from_secs(5));
        assert_eq!(
            error.downcast_ref::<Error>().expect("I/O error").kind(),
            ErrorKind::TimedOut
        );
        assert!(error.to_string().contains("bearer token lookup timed out"));
        assert!(
            provider
                .get_token("https://monitor.azure.com")
                .await
                .is_ok()
        );
    }

    /// Scenario: One token lookup remains pending while a second lookup can return a token.
    /// Guarantees: The second lookup succeeds without waiting for the first lookup to time out.
    #[tokio::test(start_paused = true)]
    async fn pending_lookup_does_not_block_concurrent_lookup() {
        let provider = GenevaGcsTokenProvider::new(
            Box::new(PendingThenReadyProvider(AtomicUsize::new(0))),
            "https://monitor.azure.com".to_string(),
        );
        let mut pending_lookup = provider.get_token("https://monitor.azure.com");
        assert!(futures::poll!(&mut pending_lookup).is_pending());
        let started = tokio::time::Instant::now();

        let _token = tokio::time::timeout(
            Duration::from_secs(1),
            provider.get_token("https://monitor.azure.com"),
        )
        .await
        .expect("second lookup must not wait for the stalled lookup")
        .expect("second lookup should return a usable token");

        assert_eq!(started.elapsed(), Duration::ZERO);
        assert!(futures::poll!(&mut pending_lookup).is_pending());
    }

    /// Scenario: The uploader requests the resource configured on the adapter.
    /// Guarantees: A usable capability token is returned without a second cache or refresh layer.
    #[tokio::test]
    async fn returns_usable_token_for_configured_resource() {
        let (provider, calls) = provider(BearerToken::without_expiry("token".to_string()));

        let token = provider
            .get_token("https://monitor.azure.com")
            .await
            .expect("configured resource should resolve");

        assert!(!format!("{token:?}").contains("token"));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    /// Scenario: The uploader requests a resource other than the adapter's configured resource.
    /// Guarantees: The mismatch fails before consulting the bound capability.
    #[tokio::test]
    async fn rejects_resource_mismatch_before_provider_call() {
        let (provider, calls) = provider(BearerToken::without_expiry("token".to_string()));

        let error = provider
            .get_token("https://storage.azure.com")
            .await
            .expect_err("resource mismatch must fail");

        assert!(error.to_string().contains("configured for"));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    /// Scenario: The bound capability returns an empty or near-expiry token.
    /// Guarantees: Credentials that are not safe for a GCS request are rejected.
    #[tokio::test]
    async fn rejects_empty_and_near_expiry_tokens() {
        let (empty, _) = provider(BearerToken::without_expiry(" \t".to_string()));
        assert!(
            empty
                .get_token("https://monitor.azure.com")
                .await
                .expect_err("empty token must fail")
                .to_string()
                .contains("empty")
        );

        let (expiring, _) = provider(BearerToken::with_expiry(
            "token".to_string(),
            Some(Instant::now() + TOKEN_USABLE_MARGIN - Duration::from_secs(1)),
        ));
        assert!(
            expiring
                .get_token("https://monitor.azure.com")
                .await
                .expect_err("near-expiry token must fail")
                .to_string()
                .contains("near-expiry")
        );
    }
}
