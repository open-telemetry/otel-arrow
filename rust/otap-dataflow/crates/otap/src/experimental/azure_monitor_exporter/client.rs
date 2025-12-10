// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::TokenCredential;
use azure_core::time::OffsetDateTime;

use bytes::Bytes;

use rand::{Rng, SeedableRng, rngs::SmallRng};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderValue},
};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

use crate::experimental::azure_monitor_exporter::auth::Auth;
use crate::experimental::azure_monitor_exporter::config::Config;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: Duration = Duration::from_secs(3);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// HTTP client for Azure Log Analytics Data Collection Rule (DCR) endpoint.
///
/// Handles authentication, compression, and HTTP communication with the
/// Azure Monitor Logs Ingestion API.
#[derive(Clone)]
// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
pub struct LogsIngestionClient {
    http_client: Client,
    endpoint: String,
    auth: Auth,

    // Pre-formatted authorization header for zero-allocation reuse
    auth_header: HeaderValue,

    /// Token expiry time using monotonic clock for faster comparisons
    pub token_valid_until: Instant,

    token_refresh_after: Instant,
}

pub struct LogsIngestionClientPool {
    clients: Vec<LogsIngestionClient>,
}

impl LogsIngestionClientPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            clients: Vec::with_capacity(capacity),
        }
    }

    pub fn initialize(&mut self, client: &LogsIngestionClient) {
        let capacity = self.clients.capacity();
        for _ in 0..capacity {
            self.clients.push(client.clone());
        }
    }

    #[inline(always)]
    pub fn take(&mut self) -> LogsIngestionClient {
        self.clients.pop().expect("client pool is empty")
    }

    #[inline(always)]
    pub fn release(&mut self, client: LogsIngestionClient) {
        self.clients.push(client);
    }
}

// TODO: Remove print_stdout after logging is set up
#[allow(clippy::print_stdout)]
impl LogsIngestionClient {
    /// Creates a new Azure Monitor logs ingestion client instance from provided components.
    ///
    /// # Arguments
    /// * `http_client` - The HTTP client to use for requests
    /// * `endpoint` - The full endpoint URL for the Azure Monitor ingestion API
    /// * `credential` - The token credential for authentication
    /// * `scope` - The OAuth scope for token acquisition
    ///
    /// # Returns
    /// * `LogsIngestionClient` - A configured client instance
    pub fn from_parts(
        http_client: Client,
        endpoint: String,
        credential: Arc<dyn TokenCredential>,
        scope: String,
    ) -> Self {
        Self {
            http_client,
            endpoint,
            auth: Auth::from_credential(credential, scope),
            auth_header: HeaderValue::from_static("Bearer "),
            token_valid_until: Instant::now(),
            token_refresh_after: Instant::now(),
        }
    }

    /// Creates a new Azure Monitor logs ingestion client instance from the configuration.
    ///
    /// # Arguments
    /// * `config` - The Azure Monitor Exporter configuration
    ///
    /// # Returns
    /// * `Ok(LogsIngestionClient)` - A configured client instance
    /// * `Err(String)` - Error message if initialization fails
    pub fn new(config: &Config) -> Result<Self, String> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .http2_prior_knowledge()  // Use HTTP/2 directly (faster, no upgrade negotiation)
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_nodelay(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.api.dcr_endpoint, config.api.dcr, config.api.stream_name
        );

        let auth =
            Auth::new(&config.auth).map_err(|e| format!("Failed to create auth handler: {e}"))?;

        Ok(Self {
            http_client,
            endpoint,
            auth,
            auth_header: HeaderValue::from_static("Bearer "),
            token_valid_until: Instant::now(),
            token_refresh_after: Instant::now(),
        })
    }

    /// Refresh the token and update the pre-formatted header
    pub async fn refresh_token(&mut self) -> Result<(), String> {
        let token = self
            .auth
            .get_token()
            .await
            .map_err(|e| format!("Failed to acquire token: {e}"))?;

        // Pre-format the authorization header to avoid repeated allocation
        self.auth_header = HeaderValue::from_str(&format!("Bearer {}", token.token.secret()))
            .map_err(|_| "Invalid token format".to_string())?;

        // Calculate validity using Instant for faster comparisons
        // Refresh 5 minutes before expiry
        let valid_seconds = (token.expires_on - OffsetDateTime::now_utc()).whole_seconds();

        self.token_valid_until = Instant::now() + Duration::from_secs(valid_seconds.max(0) as u64);
        self.token_refresh_after = self.token_valid_until - Duration::from_secs(300);

        println!(
            "[AzureMonitorExporter] Acquired new token, valid for {} seconds, valid until {:?}",
            valid_seconds, self.token_valid_until
        );

        Ok(())
    }

    /// Refresh the token if needed and update the pre-formatted header
    #[inline]
    pub async fn ensure_valid_token(&mut self) -> Result<(), String> {
        let now = Instant::now();

        // Fast path: token is still valid
        if now < self.token_refresh_after {
            return Ok(());
        }

        self.refresh_token().await?;

        Ok(())
    }

    // TODO: Remove print_stdout after logging is set up
    #[allow(clippy::print_stdout)]
    /// Export compressed data to Log Analytics ingestion API with automatic retry.
    ///
    /// Retries on:
    /// - Network errors
    /// - 401 (after token refresh)
    /// - 429 (rate limiting) - uses Retry-After header if present
    /// - 5xx (server errors)
    ///
    /// # Arguments
    /// * `body` - The gzip-compressed JSON data to send
    ///
    /// # Returns
    /// * `Ok(Duration)` - Total time spent (including retries) if successful
    /// * `Err(String)` - Error message if all retries exhausted or non-retryable error
    pub async fn export(&mut self, body: Bytes) -> Result<Duration, String> {
        let start = Instant::now();
        let mut attempt = 0u32;
        let mut rng = SmallRng::seed_from_u64(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
            ^ (self as *const _ as u64));  // Mix in object address

        loop {
            match self.try_export(body.clone()).await {
                Ok(()) => return Ok(start.elapsed()),
                Err(ExportAttemptError::Terminal(e)) => return Err(e),
                Err(ExportAttemptError::Retryable { message, retry_after }) => {
                    let delay = if let Some(server_delay) = retry_after {
                        // Server specified delay - add 3-6 seconds jitter on top, minimum 5 seconds base
                        let base_delay = server_delay.max(Duration::from_secs(5));
                        let jitter = Duration::from_secs(3) + Duration::from_secs_f64(rng.random::<f64>() * 7.0);
                        base_delay + jitter
                    } else {
                        // No server hint - use exponential backoff with max retries
                        attempt += 1;
                        if attempt >= MAX_RETRIES {
                            return Err(format!(
                                "Export failed after {attempt} attempts: {message}"
                            ));
                        }
                        let backoff = INITIAL_BACKOFF * 2u32.pow(attempt - 1);
                        let base_delay = backoff.min(MAX_BACKOFF);
                        // Add ±15% jitter (0.85 to 1.15)
                        let jitter_factor = 0.85 + rng.random::<f64>() * 0.30;
                        base_delay.mul_f64(jitter_factor)
                    };

                    println!(
                        "[AzureMonitorExporter] Retry after {}ms: {message}",
                        delay.as_secs_f64()
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Single export attempt without retry logic.
    async fn try_export(&mut self, body: Bytes) -> Result<(), ExportAttemptError> {
        let response = self
            .http_client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .header(AUTHORIZATION, &self.auth_header)
            .body(body)
            .send()
            .await
            .map_err(|e| {
                let detail = if e.is_timeout() {
                    "timeout"
                } else if e.is_connect() {
                    "connect"
                } else if e.is_request() {
                    "request"
                } else if e.is_body() {
                    "body"
                } else {
                    "unknown"
                };
                
                // Walk the error chain for full context
                let mut sources = Vec::new();
                let mut current: &dyn std::error::Error = &e;
                while let Some(source) = current.source() {
                    sources.push(format!("{}", source));
                    current = source;
                }
                
                let source_chain = if sources.is_empty() {
                    String::new()
                } else {
                    format!(" [causes: {}]", sources.join(" -> "))
                };
                
                ExportAttemptError::Retryable {
                    message: format!("Network error ({}): {}{}", detail, e, source_chain),
                    retry_after: None,
                }
            })?;

        // Fast path for success
        if response.status().is_success() {
            return Ok(());
        }

        // Extract Retry-After header before consuming response
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        let status = response.status();
        let error = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => {
                // Invalidate token and refresh for next attempt
                self.token_valid_until = Instant::now();
                self.auth.invalidate_token().await;
                self.ensure_valid_token()
                    .await
                    .map_err(|e| ExportAttemptError::Terminal(e))?;

                Err(ExportAttemptError::Retryable {
                    message: format!("Authentication failed: {error}"),
                    retry_after: None,
                })
            }
            403 => Err(ExportAttemptError::Terminal(format!(
                "Authorization failed: {error}"
            ))),
            413 => Err(ExportAttemptError::Terminal(
                "Payload too large - reduce batch size".to_string(),
            )),
            429 => Err(ExportAttemptError::Retryable {
                message: format!("Rate limited: {error}"),
                retry_after,
            }),
            500..=599 => Err(ExportAttemptError::Retryable {
                message: format!("Server error ({status}): {error}"),
                retry_after,
            }),
            _ => Err(ExportAttemptError::Terminal(format!(
                "Request failed ({status}): {error}"
            ))),
        }
    }
}

/// Internal error type for single export attempt
enum ExportAttemptError {
    /// Retryable error with optional server-specified delay
    Retryable {
        message: String,
        retry_after: Option<Duration>,
    },
    /// Non-retryable error
    Terminal(String),
}

#[cfg(test)]
mod tests {
    use super::super::config::{ApiConfig, AuthConfig, AuthMethod};
    use super::*;
    use azure_core::Bytes;
    use azure_core::credentials::TokenRequestOptions;
    use azure_core::credentials::{AccessToken, TokenCredential};
    use std::sync::Mutex;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<Mutex<usize>>,
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc() + self.expires_in,
            })
        }
    }

    #[test]
    fn test_new_builds_correct_endpoint() {
        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.azure.com".to_string(),
                dcr: "test-dcr-id".to_string(),
                stream_name: "test-stream".to_string(),
                schema: Default::default(),
            },
            auth: AuthConfig {
                method: AuthMethod::ManagedIdentity,
                client_id: Some("test-client-id".to_string()),
                scope: "https://monitor.azure.com/.default".to_string(),
            },
        };

        // We can at least test that new() doesn't panic and builds the correct endpoint
        match LogsIngestionClient::new(&config) {
            Ok(client) => {
                assert_eq!(
                    client.endpoint,
                    "https://test.azure.com/dataCollectionRules/test-dcr-id/streams/test-stream?api-version=2021-11-01-preview"
                );
                // REMOVED: assert_eq!(client.headers.len(), 2);
                // We no longer have a headers field - check auth_header instead
                assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));
            }
            Err(e) => {
                // This is acceptable if running in an environment without proper Azure setup
                assert!(e.contains("Failed to create auth handler"));
            }
        }
    }

    #[tokio::test]
    async fn test_export_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/dataCollectionRules/dcr1/streams/stream1"))
            .and(header("Content-Type", "application/json"))
            .and(header("Content-Encoding", "gzip"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut client = LogsIngestionClient::from_parts(
            Client::new(),
            format!(
                "{}/dataCollectionRules/dcr1/streams/stream1?api-version=2021-11-01-preview",
                mock_server.uri()
            ),
            credential,
            "scope".to_string(),
        );

        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_ok());
        assert_eq!(*call_count.lock().unwrap(), 1); // Token fetched once
    }

    #[tokio::test]
    async fn test_export_auth_failure_refreshes_token() {
        let mock_server = MockServer::start().await;

        // First request fails with 401
        Mock::given(method("POST"))
            .and(path("/dataCollectionRules/dcr1/streams/stream1"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&mock_server)
            .await;

        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut client = LogsIngestionClient::from_parts(
            Client::new(),
            format!(
                "{}/dataCollectionRules/dcr1/streams/stream1?api-version=2021-11-01-preview",
                mock_server.uri()
            ),
            credential,
            "scope".to_string(),
        );

        // Reset and add success response for retry
        mock_server.reset().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        // Should succeed after retry with refreshed token
        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_ok());
        assert_eq!(*call_count.lock().unwrap(), 2); // Token fetched twice (initial + refresh)
    }

    #[tokio::test]
    async fn test_export_rate_limited_with_retry_after() {
        let mock_server = MockServer::start().await;

        // First request returns 429 with Retry-After, second succeeds
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut client = LogsIngestionClient::from_parts(
            Client::new(),
            format!(
                "{}/dataCollectionRules/dcr1/streams/stream1?api-version=2021-11-01-preview",
                mock_server.uri()
            ),
            credential,
            "scope".to_string(),
        );

        let start = Instant::now();
        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should have waited at least 1 second for retry
        assert!(elapsed >= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_export_terminal_error_no_retry() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(403))
            .expect(1) // Should only be called once - no retry
            .mount(&mock_server)
            .await;

        let call_count = Arc::new(Mutex::new(0));
        let credential = Arc::new(MockCredential {
            token: "test_token".to_string(),
            expires_in: azure_core::time::Duration::minutes(60),
            call_count: call_count.clone(),
        });

        let mut client = LogsIngestionClient::from_parts(
            Client::new(),
            format!(
                "{}/dataCollectionRules/dcr1/streams/stream1?api-version=2021-11-01-preview",
                mock_server.uri()
            ),
            credential,
            "scope".to_string(),
        );

        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Authorization failed"));
    }
}
