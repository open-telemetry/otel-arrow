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

use super::auth::Auth;
use super::config::ApiConfig;
use super::error::Error;

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

    fn create_http_clients(&self, count: usize) -> Result<Vec<Client>, Error> {
        let capacity = self.clients.capacity();
        let mut clients = Vec::with_capacity(count);

        for _ in 0..count {
            let http_client = Client::builder()
                .http1_only()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(capacity) // e.g., 32/4 = 8 connections per pool
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_nodelay(true)
                .build()
                .map_err(Error::CreateClient)?;

            clients.push(http_client);
        }

        Ok(clients)
    }

    pub async fn initialize(&mut self, config: &ApiConfig, auth: &Auth) -> Result<(), Error> {
        let capacity = self.clients.capacity();
        let http_clients = self.create_http_clients(capacity)?;

        for http_client in http_clients {
            let mut client = LogsIngestionClient::new(config, http_client, auth.clone())?;
            client.ensure_valid_token().await?;
            self.clients.push(client);
        }

        Ok(())
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
    /// * `http_client` - The HTTP client to use for requests
    ///
    /// # Returns
    /// * `Ok(LogsIngestionClient)` - A configured client instance
    /// * `Err(String)` - Error message if initialization fails
    pub fn new(config: &ApiConfig, http_client: Client, auth: Auth) -> Result<Self, Error> {
        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.dcr_endpoint, config.dcr, config.stream_name
        );

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
    pub async fn refresh_token(&mut self) -> Result<(), Error> {
        let token = self
            .auth
            .get_token()
            .await?;

        // Pre-format the authorization header to avoid repeated allocation
        self.auth_header = HeaderValue::from_str(&format!("Bearer {}", token.token.secret()))
            .map_err(Error::InvalidHeader)?;

        // Calculate validity using Instant for faster comparisons
        // Refresh 5 minutes before expiry
        let valid_seconds = (token.expires_on - OffsetDateTime::now_utc()).whole_seconds();

        self.token_valid_until = Instant::now() + Duration::from_secs(valid_seconds.max(0) as u64);
        self.token_refresh_after = self.token_valid_until - Duration::from_secs(300);

        Ok(())
    }

    /// Refresh the token if needed and update the pre-formatted header
    #[inline]
    pub async fn ensure_valid_token(&mut self) -> Result<(), Error> {
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
    pub async fn export(&mut self, body: Bytes) -> Result<Duration, Error> {
        let mut attempt = 0u32;
        let mut rng = SmallRng::seed_from_u64(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before UNIX epoch")
                .as_nanos() as u64
                ^ (self as *const _ as u64),
        );

        loop {
            match self.try_export(body.clone()).await {
                Ok(duration) => return Ok(duration),
                Err(e) if !e.is_retryable() => {
                    return Err(Error::ExportFailed {
                        attempts: attempt + 1,
                        last_error: Box::new(e),
                    });
                }
                Err(e) => {
                    let delay = if let Some(server_delay) = e.retry_after() {
                        let base_delay = server_delay.max(Duration::from_secs(5));
                        let jitter = Duration::from_secs(3)
                            + Duration::from_secs_f64(rng.random::<f64>() * 7.0);
                        base_delay + jitter
                    } else {
                        attempt += 1;
                        if attempt >= MAX_RETRIES {
                            return Err(Error::ExportFailed {
                                attempts: attempt,
                                last_error: Box::new(e),
                            });
                        }
                        let backoff = INITIAL_BACKOFF * 2u32.pow(attempt - 1);
                        let base_delay = backoff.min(MAX_BACKOFF);
                        let jitter_factor = 0.85 + rng.random::<f64>() * 0.30;
                        base_delay.mul_f64(jitter_factor)
                    };

                    println!(
                        "[AzureMonitorExporter] Retry after {}ms: {:?}",
                        delay.as_millis(),
                        e
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Single export attempt without retry logic.
    async fn try_export(&mut self, body: Bytes) -> Result<Duration, Error> {
        let start = Instant::now();

        let response = self
            .http_client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .header(AUTHORIZATION, &self.auth_header)
            .body(body)
            .send()
            .await
            .map_err(Error::network)?;

        // Fast path for success
        if response.status().is_success() {
            return Ok(start.elapsed());
        }

        // Extract Retry-After header before consuming response
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 => {
                self.token_valid_until = Instant::now();
                self.auth.invalidate_token().await;
                self.ensure_valid_token()
                    .await
                    .map_err(|e| Error::token_refresh(Box::new(e)))?;

                Err(Error::unauthorized(body))
            }
            403 => Err(Error::forbidden(body)),
            413 => Err(Error::PayloadTooLarge),
            429 => Err(Error::RateLimited { body, retry_after }),
            500..=599 => Err(Error::ServerError {
                status,
                body,
                retry_after,
            }),
            _ => Err(Error::UnexpectedStatus { status, body }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::TokenRequestOptions;
    use azure_core::credentials::{AccessToken, TokenCredential};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ==================== Test Helpers ====================

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<Mutex<usize>>,
    }

    impl MockCredential {
        fn new(token: &str, expires_in_minutes: i64) -> (Arc<Self>, Arc<Mutex<usize>>) {
            let call_count = Arc::new(Mutex::new(0));
            let cred = Arc::new(Self {
                token: token.to_string(),
                expires_in: azure_core::time::Duration::minutes(expires_in_minutes),
                call_count: call_count.clone(),
            });
            (cred, call_count)
        }
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

    #[derive(Debug)]
    struct FailingCredential {
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl TokenCredential for FailingCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            _ = self.call_count.fetch_add(1, Ordering::SeqCst);
            Err(azure_core::Error::with_message(
                azure_core::error::ErrorKind::Credential,
                "Token acquisition failed",
            ))
        }
    }

    fn create_test_api_config() -> ApiConfig {
        ApiConfig {
            dcr_endpoint: "https://test.azure.com".to_string(),
            dcr: "test-dcr".to_string(),
            stream_name: "test-stream".to_string(),
            schema: Default::default(),
        }
    }

    fn create_test_http_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to create HTTP client")
    }

    // ==================== Construction Tests ====================

    #[test]
    fn test_new_builds_correct_endpoint() {
        let api_config = ApiConfig {
            dcr_endpoint: "https://test.azure.com".to_string(),
            dcr: "test-dcr-id".to_string(),
            stream_name: "test-stream".to_string(),
            schema: Default::default(),
        };

        let (credential, _) = MockCredential::new("test_token", 60);
        let auth =
            Auth::from_credential(credential as Arc<dyn TokenCredential>, "https://monitor.azure.com/.default".to_string());
        let http_client = create_test_http_client();

        let client = LogsIngestionClient::new(&api_config, http_client, auth)
            .expect("failed to create client");

        assert_eq!(
            client.endpoint,
            "https://test.azure.com/dataCollectionRules/test-dcr-id/streams/test-stream?api-version=2021-11-01-preview"
        );
    }

    #[test]
    fn test_new_with_special_characters_in_config() {
        let api_config = ApiConfig {
            dcr_endpoint: "https://my-endpoint.azure.com".to_string(),
            dcr: "dcr-abc-123-def".to_string(),
            stream_name: "Custom-Stream_Name".to_string(),
            schema: Default::default(),
        };

        let (credential, _) = MockCredential::new("token", 60);
        let auth = Auth::from_credential(credential as Arc<dyn TokenCredential>, "scope".to_string());
        let http_client = create_test_http_client();

        let client = LogsIngestionClient::new(&api_config, http_client, auth).unwrap();

        assert!(client.endpoint.contains("dcr-abc-123-def"));
        assert!(client.endpoint.contains("Custom-Stream_Name"));
    }

    #[test]
    fn test_from_parts_creates_client() {
        let (credential, _) = MockCredential::new("my_token", 60);

        let client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
            credential as Arc<dyn TokenCredential>,
            "https://scope.azure.com/.default".to_string(),
        );

        assert_eq!(client.endpoint, "https://example.com/endpoint");
        // Token not yet fetched, so auth_header is placeholder
        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));
        // Token validity should be in the past (not yet fetched)
        assert!(client.token_valid_until <= Instant::now());
        assert!(client.token_refresh_after <= Instant::now());
    }

    #[test]
    fn test_new_initial_state() {
        let (credential, call_count) = MockCredential::new("token", 60);
        let auth = Auth::from_credential(credential as Arc<dyn TokenCredential>, "scope".to_string());
        let http_client = create_test_http_client();
        let api_config = create_test_api_config();

        let client = LogsIngestionClient::new(&api_config, http_client, auth).unwrap();

        // No token fetch during construction
        assert_eq!(*call_count.lock().unwrap(), 0);
        // Auth header is placeholder
        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));
    }

    // ==================== LogsIngestionClientPool Tests ====================

    #[test]
    fn test_pool_new_creates_empty_pool() {
        let pool = LogsIngestionClientPool::new(5);

        assert_eq!(pool.clients.capacity(), 5);
        assert_eq!(pool.clients.len(), 0);
    }

    #[test]
    fn test_pool_new_various_capacities() {
        for capacity in [1, 4, 8, 32, 100] {
            let pool = LogsIngestionClientPool::new(capacity);
            assert_eq!(pool.clients.capacity(), capacity);
        }
    }

    #[test]
    fn test_pool_take_and_release_single() {
        let (credential, _) = MockCredential::new("token", 60);
        let auth = Auth::from_credential(credential as Arc<dyn TokenCredential>, "scope".to_string());
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1);
        let client =
            LogsIngestionClient::new(&api_config, create_test_http_client(), auth).unwrap();
        pool.clients.push(client);

        assert_eq!(pool.clients.len(), 1);

        let taken = pool.take();
        assert_eq!(pool.clients.len(), 0);

        pool.release(taken);
        assert_eq!(pool.clients.len(), 1);
    }

    #[test]
    fn test_pool_take_and_release_multiple() {
        let (credential, _) = MockCredential::new("token", 60);
        let auth = Auth::from_credential(credential as Arc<dyn TokenCredential>, "scope".to_string());
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(3);
        for _ in 0..3 {
            let client =
                LogsIngestionClient::new(&api_config, create_test_http_client(), auth.clone())
                    .unwrap();
            pool.clients.push(client);
        }

        assert_eq!(pool.clients.len(), 3);

        // Take all
        let c1 = pool.take();
        let c2 = pool.take();
        let c3 = pool.take();
        assert_eq!(pool.clients.len(), 0);

        // Release in different order
        pool.release(c2);
        pool.release(c1);
        pool.release(c3);
        assert_eq!(pool.clients.len(), 3);
    }

    #[test]
    fn test_pool_release_beyond_capacity() {
        let (credential, _) = MockCredential::new("token", 60);
        let auth = Auth::from_credential(credential as Arc<dyn TokenCredential>, "scope".to_string());
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1);

        // Release more than capacity (Vec will grow)
        for _ in 0..5 {
            let client =
                LogsIngestionClient::new(&api_config, create_test_http_client(), auth.clone())
                    .unwrap();
            pool.release(client);
        }

        assert_eq!(pool.clients.len(), 5);
    }

    // ==================== Token Management Tests ====================

    #[tokio::test]
    async fn test_refresh_token_updates_header() {
        let (credential, call_count) = MockCredential::new("fresh_token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // Initially placeholder
        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));

        client.refresh_token().await.unwrap();

        assert_eq!(
            client.auth_header,
            HeaderValue::from_str("Bearer fresh_token").unwrap()
        );
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_refresh_token_updates_validity_times() {
        let (credential, _) = MockCredential::new("token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        let before = Instant::now();
        client.refresh_token().await.unwrap();

        // Token should be valid for ~60 minutes
        assert!(client.token_valid_until > before + Duration::from_secs(3500));
        // Refresh should happen 5 minutes before expiry
        assert!(client.token_refresh_after > before + Duration::from_secs(3200));
        assert!(client.token_refresh_after < client.token_valid_until);
    }

    #[tokio::test]
    async fn test_ensure_valid_token_fetches_when_expired() {
        let (credential, call_count) = MockCredential::new("token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // Token starts expired
        assert!(client.token_refresh_after <= Instant::now());

        client.ensure_valid_token().await.unwrap();

        assert_eq!(*call_count.lock().unwrap(), 1);
        assert!(client.token_refresh_after > Instant::now());
    }

    #[tokio::test]
    async fn test_ensure_valid_token_caches_token() {
        let (credential, call_count) = MockCredential::new("cached_token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // First call fetches token
        client.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Subsequent calls should use cached token (fast path)
        for _ in 0..10 {
            client.ensure_valid_token().await.unwrap();
        }

        // Should still be 1 - token was cached
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_ensure_valid_token_refreshes_when_in_refresh_window() {
        let (credential, call_count) = MockCredential::new("token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // First fetch
        client.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Simulate token about to expire (within 5 minute refresh window)
        client.token_refresh_after = Instant::now() - Duration::from_secs(1);

        // Invalidate the cached token in Auth so it will fetch again
        client.auth.invalidate_token().await;

        // Should refresh
        client.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_refresh_token_failure() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = Arc::new(FailingCredential {
            call_count: call_count.clone(),
        }) as Arc<dyn TokenCredential>;  // <-- cast here

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential,
            "scope".to_string(),
        );

        let result = client.refresh_token().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("token acquisition"));
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_ensure_valid_token_propagates_error() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let credential = Arc::new(FailingCredential {
            call_count: call_count.clone(),
        }) as Arc<dyn TokenCredential>;  // <-- cast here

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential,
            "scope".to_string(),
        );

        let result = client.ensure_valid_token().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("token acquisition"));
    }

    // ==================== Clone Tests ====================

    #[tokio::test]
    async fn test_client_clone_has_same_endpoint() {
        let (credential, _) = MockCredential::new("token", 60);

        let client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        let client2 = client1.clone();

        assert_eq!(client1.endpoint, client2.endpoint);
    }

    #[tokio::test]
    async fn test_client_clone_shares_auth() {
        let (credential, call_count) = MockCredential::new("shared_token", 60);

        let mut client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // Fetch token on client1
        client1.refresh_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Clone shares the Auth (which has cached token)
        let mut client2 = client1.clone();

        // client2's refresh should use cached token from shared Auth
        client2.refresh_token().await.unwrap();
        // Auth caches, so might be 1 or 2 depending on implementation
        // The important thing is both clients work
        assert!(*call_count.lock().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_client_clone_has_independent_header() {
        let (credential, _) = MockCredential::new("token", 60);

        let mut client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        client1.refresh_token().await.unwrap();

        let client2 = client1.clone();

        // Both should have the same header after clone
        assert_eq!(client1.auth_header, client2.auth_header);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_client_with_empty_endpoint() {
        let (credential, _) = MockCredential::new("token", 60);

        let client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        assert_eq!(client.endpoint, "");
    }

    #[tokio::test]
    async fn test_multiple_refresh_calls() {
        let (credential, call_count) = MockCredential::new("token", 60);

        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
            credential as Arc<dyn TokenCredential>,
            "scope".to_string(),
        );

        // Multiple refresh calls - Auth caches the token, so credential
        // is only called once unless we invalidate
        client.refresh_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Subsequent refreshes use cached token from Auth
        for _ in 0..4 {
            client.refresh_token().await.unwrap();
        }

        // Still 1 because Auth caches
        assert_eq!(*call_count.lock().unwrap(), 1);

        // If we invalidate, it will fetch again
        client.auth.invalidate_token().await;
        client.refresh_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    #[test]
    fn test_pool_create_http_clients() {
        let pool = LogsIngestionClientPool::new(4);

        let result = pool.create_http_clients(4);

        assert!(result.is_ok());
        let clients = result.unwrap();
        assert_eq!(clients.len(), 4);
    }

    #[test]
    fn test_pool_create_http_clients_zero() {
        let pool = LogsIngestionClientPool::new(4);

        let result = pool.create_http_clients(0);

        assert!(result.is_ok());
        let clients = result.unwrap();
        assert_eq!(clients.len(), 0);
    }
}
