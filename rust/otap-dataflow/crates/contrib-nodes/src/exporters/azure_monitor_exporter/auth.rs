// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::{AccessToken, TokenCredential};
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use otap_df_telemetry::{otel_debug, otel_warn};
use std::sync::Arc;

use super::Error;
use super::config::{AuthConfig, AuthMethod};
use super::metrics::AzureMonitorExporterMetricsRc;

/// Brief pause between outer retry loops.
/// The Azure SDK already performs exponential backoff internally
/// (e.g., 6 retries over ~72s for IMDS), so this is just a short
/// breather before the next SDK retry cycle.
const RETRY_PAUSE: tokio::time::Duration = tokio::time::Duration::from_secs(1);

#[derive(Clone, Debug)]
// TODO - Consolidate with crates/otap/src/{cloud_auth,object_store)/azure.rs
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
    metrics: AzureMonitorExporterMetricsRc,
}

impl Auth {
    pub fn new(
        auth_config: &AuthConfig,
        metrics: AzureMonitorExporterMetricsRc,
    ) -> Result<Self, Error> {
        let credential = Self::create_credential(auth_config)?;

        Ok(Self {
            credential,
            scope: auth_config.scope.clone(),
            metrics,
        })
    }

    #[cfg(test)]
    pub fn from_credential(
        credential: Arc<dyn TokenCredential>,
        scope: String,
        metrics: AzureMonitorExporterMetricsRc,
    ) -> Self {
        Self {
            credential,
            scope,
            metrics,
        }
    }

    async fn get_token_internal(&self) -> Result<AccessToken, Error> {
        let token_response = self
            .credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(Error::token_acquisition)?;

        Ok(token_response)
    }

    /// Attempt token acquisition with bounded retries and an overall timeout.
    /// Used at startup to surface auth misconfigurations quickly.
    /// Retries up to `max_attempts` times within the `timeout` duration.
    /// Returns Ok(token) on first success, or the last error if all attempts
    /// fail or the overall timeout is reached.
    pub async fn try_get_token(
        &self,
        timeout: tokio::time::Duration,
        max_attempts: u32,
    ) -> Result<AccessToken, Error> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            match tokio::time::timeout(remaining, self.get_token_internal()).await {
                Ok(Ok(token)) => return Ok(token),
                Ok(Err(e)) => {
                    otel_debug!(
                        "azure_monitor_exporter.auth.startup_attempt_failed",
                        attempt = attempt,
                        max_attempts = max_attempts,
                        error = %e
                    );
                    last_error = Some(e);
                }
                Err(_elapsed) => {
                    return Err(Error::token_acquisition_timeout(timeout));
                }
            }

            // Brief pause between retries (not after the last attempt)
            if attempt < max_attempts {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if !remaining.is_zero() {
                    tokio::time::sleep(remaining.min(tokio::time::Duration::from_millis(500)))
                        .await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::token_acquisition_timeout(timeout)))
    }

    pub async fn get_token(&mut self) -> Result<AccessToken, Error> {
        let mut attempt = 0_i32;
        let start = tokio::time::Instant::now();
        loop {
            attempt += 1;

            match self.get_token_internal().await {
                Ok(token) => {
                    otel_debug!("azure_monitor_exporter.auth.get_token_succeeded", expires_on = %token.expires_on);
                    let mut m = self.metrics.borrow_mut();
                    m.add_auth_success_latency(start.elapsed().as_millis() as f64);
                    return Ok(token);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let first_line = error_msg.lines().next().unwrap_or(&error_msg);

                    otel_warn!(
                        "azure_monitor_exporter.auth.get_token_failed",
                        message = "Token acquisition failed. Will keep retrying. The error may mention retries being exhausted; that refers to an internal retry layer, not this outer loop.",
                        attempt = attempt,
                        error = %first_line
                    );
                    otel_debug!("azure_monitor_exporter.auth.get_token_failed.details", attempt = attempt, error = %e);
                    self.metrics.borrow_mut().add_auth_failure();

                    // The Azure SDK already retries internally with exponential
                    // backoff (e.g., 6 retries over ~72s for IMDS). This short
                    // pause just prevents a tight spin before the next SDK cycle.
                    tokio::time::sleep(RETRY_PAUSE).await;
                }
            }
        }
    }

    fn create_credential(auth_config: &AuthConfig) -> Result<Arc<dyn TokenCredential>, Error> {
        match auth_config.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &auth_config.client_id {
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                }

                Ok(ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| Error::create_credential(AuthMethod::ManagedIdentity, e))?)
            }
            AuthMethod::Development => Ok(DeveloperToolsCredential::new(Some(
                DeveloperToolsCredentialOptions::default(),
            ))
            .map_err(|e| Error::create_credential(AuthMethod::Development, e))?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::metrics::{AzureMonitorExporterMetrics, AzureMonitorExporterMetricsTracker};
    use super::*;
    use azure_core::credentials::TokenRequestOptions;
    use azure_core::time::OffsetDateTime;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::testing::EmptyAttributes;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn create_test_metrics() -> AzureMonitorExporterMetricsRc {
        let registry = TelemetryRegistryHandle::new();
        let metric_set =
            registry.register_metric_set::<AzureMonitorExporterMetrics>(EmptyAttributes());
        Rc::new(RefCell::new(AzureMonitorExporterMetricsTracker::new(
            metric_set,
        )))
    }

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<AtomicUsize>,
    }

    fn make_mock_credential(
        token: &str,
        expires_in: azure_core::time::Duration,
        call_count: Arc<AtomicUsize>,
    ) -> Arc<dyn TokenCredential> {
        let cred: Arc<dyn TokenCredential> = Arc::new(MockCredential {
            token: token.to_string(),
            expires_in,
            call_count,
        });
        cred
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let _ = self.call_count.fetch_add(1, Ordering::SeqCst);

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc() + self.expires_in,
            })
        }
    }

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_from_credential_creates_auth() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let auth =
            Auth::from_credential(credential, "test_scope".to_string(), create_test_metrics());
        assert_eq!(auth.scope, "test_scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_user_assigned() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: Some("test-client-id".to_string()),
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config, create_test_metrics());
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.scope, "https://test.scope");
    }

    #[tokio::test]
    async fn test_new_with_managed_identity_system_assigned() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        let auth = Auth::new(&auth_config, create_test_metrics());
        assert!(auth.is_ok());
    }

    #[tokio::test]
    async fn test_new_with_development_auth() {
        let auth_config = AuthConfig {
            method: AuthMethod::Development,
            client_id: None,
            scope: "https://test.scope".to_string(),
        };

        // May fail if Azure CLI not installed - both outcomes are valid
        let result = Auth::new(&auth_config, create_test_metrics());
        match result {
            Ok(auth) => assert_eq!(auth.scope, "https://test.scope"),
            Err(Error::Auth {
                kind: super::super::error::AuthErrorKind::CreateCredential { method },
                ..
            }) => {
                assert_eq!(method, AuthMethod::Development);
            }
            Err(err) => panic!("Unexpected error type: {:?}", err),
        }
    }

    // ==================== Token Fetching Tests ====================

    #[tokio::test]
    async fn test_get_token_internal_returns_valid_token() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let token = auth.get_token_internal().await.unwrap();
        assert_eq!(token.token.secret(), "test_token");
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_get_token_internal_calls_credential_each_time() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        // Each call to get_token_internal should call the credential
        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);

        let _ = auth.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_get_token_internal_returns_cloned_tokens() {
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            Arc::new(AtomicUsize::new(0)),
        );

        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let token1 = auth.get_token_internal().await.unwrap();
        let token2 = auth.get_token_internal().await.unwrap();

        // Same value from both calls
        assert_eq!(token1.token.secret(), token2.token.secret());
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_get_token_internal_propagates_credential_error() {
        #[derive(Debug)]
        struct FailingCredential;

        #[async_trait::async_trait]
        impl TokenCredential for FailingCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                Err(azure_core::error::Error::new(
                    azure_core::error::ErrorKind::Credential,
                    "Mock credential failure",
                ))
            }
        }

        let cred = FailingCredential;
        let credential: Arc<dyn TokenCredential> = Arc::new(cred);
        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let result = auth.get_token_internal().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth {
                kind: super::super::error::AuthErrorKind::TokenAcquisition,
                ..
            } => {}
            err => panic!("Expected Auth token acquisition error, got: {:?}", err),
        }
    }

    // ==================== try_get_token Tests ====================

    #[tokio::test]
    async fn test_try_get_token_returns_token_on_first_success() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "startup_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let result = auth
            .try_get_token(tokio::time::Duration::from_secs(5), 3)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().token.secret(), "startup_token");
        // Should succeed on the first attempt — no retries needed
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_try_get_token_retries_then_succeeds() {
        use std::sync::atomic::AtomicU32;

        /// Credential that fails the first N attempts then succeeds.
        #[derive(Debug)]
        struct FailThenSucceedCredential {
            fail_count: AtomicU32,
            failures_remaining: AtomicU32,
        }

        #[async_trait::async_trait]
        impl TokenCredential for FailThenSucceedCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                let remaining = self.failures_remaining.load(Ordering::SeqCst);
                if remaining > 0 {
                    let _ = self.failures_remaining.fetch_sub(1, Ordering::SeqCst);
                    let _ = self.fail_count.fetch_add(1, Ordering::SeqCst);
                    return Err(azure_core::error::Error::new(
                        azure_core::error::ErrorKind::Credential,
                        "transient failure",
                    ));
                }
                Ok(AccessToken {
                    token: "recovered_token".to_string().into(),
                    expires_on: OffsetDateTime::now_utc() + azure_core::time::Duration::minutes(60),
                })
            }
        }

        let credential: Arc<dyn TokenCredential> = Arc::new(FailThenSucceedCredential {
            fail_count: AtomicU32::new(0),
            failures_remaining: AtomicU32::new(2), // fail twice, succeed on 3rd
        });
        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let result = auth
            .try_get_token(tokio::time::Duration::from_secs(5), 3)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().token.secret(), "recovered_token");
    }

    #[tokio::test]
    async fn test_try_get_token_returns_error_after_max_attempts() {
        #[derive(Debug)]
        struct FailingCredential;

        #[async_trait::async_trait]
        impl TokenCredential for FailingCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                Err(azure_core::error::Error::new(
                    azure_core::error::ErrorKind::Credential,
                    "Mock credential failure",
                ))
            }
        }

        let credential: Arc<dyn TokenCredential> = Arc::new(FailingCredential);
        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let result = auth
            .try_get_token(tokio::time::Duration::from_secs(5), 3)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_get_token_times_out_on_slow_credential() {
        #[derive(Debug)]
        struct SlowCredential;

        #[async_trait::async_trait]
        impl TokenCredential for SlowCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                // Simulate a slow IMDS endpoint
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                Ok(AccessToken {
                    token: "slow_token".to_string().into(),
                    expires_on: OffsetDateTime::now_utc() + azure_core::time::Duration::minutes(60),
                })
            }
        }

        let credential: Arc<dyn TokenCredential> = Arc::new(SlowCredential);
        let auth = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());

        let start = tokio::time::Instant::now();
        let result = auth
            .try_get_token(tokio::time::Duration::from_millis(100), 3)
            .await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        // Should have timed out quickly, not waited 60 seconds
        assert!(elapsed < tokio::time::Duration::from_secs(1));
    }

    // ==================== Clone Behavior Tests ====================

    #[tokio::test]
    async fn test_cloned_auth_shares_credential() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = make_mock_credential(
            "test_token",
            azure_core::time::Duration::minutes(60),
            call_count.clone(),
        );

        let auth1 = Auth::from_credential(credential, "scope".to_string(), create_test_metrics());
        let auth2 = auth1.clone();

        // Both auth instances share the same credential
        let _ = auth1.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        let _ = auth2.get_token_internal().await.unwrap();
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
