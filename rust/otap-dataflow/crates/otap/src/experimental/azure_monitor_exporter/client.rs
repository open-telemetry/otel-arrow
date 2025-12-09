// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::TokenCredential;
use azure_core::time::OffsetDateTime;

use bytes::Bytes;

use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderValue},
};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

use crate::experimental::azure_monitor_exporter::auth::Auth;
use crate::experimental::azure_monitor_exporter::config::Config;

/// HTTP client for Azure Log Analytics Data Collection Rule (DCR) endpoint.
///
/// Handles authentication, compression, and HTTP communication with the
/// Azure Monitor Logs Ingestion API.
#[derive(Clone)]
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
        self.clients
            .pop()
            .expect("client pool is empty")
    }

    #[inline(always)]
    pub fn release(&mut self, client: LogsIngestionClient) {
        self.clients.push(client);
    }
}

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
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_nodelay(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.api.dcr_endpoint, config.api.dcr, config.api.stream_name
        );

        let auth =
            Auth::new(&config.auth)
                .map_err(|e| format!("Failed to create auth handler: {e}"))?;

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
        let valid_seconds = (token.expires_on - OffsetDateTime::now_utc())
            .whole_seconds();

        self.token_valid_until = Instant::now() + Duration::from_secs(valid_seconds.max(0) as u64);
        self.token_refresh_after = self.token_valid_until - Duration::from_secs(300);

        println!("[AzureMonitorExporter] Acquired new token, valid for {} seconds, valid until {:?}", valid_seconds, self.token_valid_until);

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
    /// Export compressed data to Log Analytics ingestion API.
    ///
    /// # Arguments
    /// * `body` - The data to send (must be serializable to JSON)
    ///
    /// # Returns
    /// * `Ok(())` - If the request was successful
    /// * `Err(String)` - Error message if the request failed
    pub async fn export(&mut self, body: Bytes) -> Result<(), String> {
        let start = Instant::now();

        // Send compressed body - avoid cloning headers by setting them individually
        let response = self
            .http_client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .header(AUTHORIZATION, &self.auth_header)
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;

        let duration = start.elapsed();
        println!("[AzureMonitorExporter] Sent batch in {:?}", duration);

        // Fast path for success
        if response.status().is_success() {
            return Ok(());
        }

        // Slow path: handle errors
        let status = response.status();
        let error = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => {
                // Invalidate token and force refresh on next call
                self.token_valid_until = Instant::now();
                self.auth.invalidate_token().await;
                self.ensure_valid_token().await?;

                Err(format!("Authentication failed: {error}"))
            }
            403 => Err(format!("Authorization failed: {error}")),
            413 => Err("Payload too large - reduce batch size".to_string()),
            429 => Err(format!("Rate limited: {error}")),
            _ => Err(format!("Request failed ({status}): {error}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::{ApiConfig, AuthConfig, AuthMethod};
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

        // This should fail with 401, but invalidate the token
        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Authentication failed"));

        // Token should have been fetched once initially
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Next call should fetch token again because it was invalidated
        mock_server.reset().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let result = client.export(Bytes::from(vec![1, 2, 3])).await;
        assert!(result.is_ok());
        assert_eq!(*call_count.lock().unwrap(), 2); // Token fetched again
    }

    #[tokio::test]
    async fn test_export_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429))
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
        assert!(result.unwrap_err().contains("Rate limited"));
    }
}
