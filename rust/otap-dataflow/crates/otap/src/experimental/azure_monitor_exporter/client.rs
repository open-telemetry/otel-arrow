// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;

use rand::{Rng, SeedableRng, rngs::SmallRng};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderValue},
};
use tokio::time::{Duration, Instant};

use super::config::ApiConfig;
use super::error::Error;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: Duration = Duration::from_secs(3);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const MAX_IDLE_CONNECTIONS_PER_HOST: usize = 2;

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

    // Pre-formatted authorization header provider
    auth_header: HeaderValue,
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
        let mut clients = Vec::with_capacity(count);

        for _ in 0..count {
            let http_client = Client::builder()
                .http1_only()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(MAX_IDLE_CONNECTIONS_PER_HOST)
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_nodelay(true)
                .build()
                .map_err(Error::CreateClient)?;

            clients.push(http_client);
        }

        Ok(clients)
    }

    pub async fn initialize(&mut self, config: &ApiConfig) -> Result<(), Error> {
        let http_clients = self.create_http_clients(self.clients.capacity())?;

        for http_client in http_clients {
            let client = LogsIngestionClient::new(config, http_client)?;
            self.clients.push(client);
        }

        Ok(())
    }

    pub fn update_auth(&mut self, header: HeaderValue) {
        for client in &mut self.clients {
            client.update_auth(header.clone());
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
    /// Creates a new Azure Monitor logs ingestion client instance from provided components, mainly for testing purposes.
    ///
    /// # Arguments
    /// * `http_client` - The HTTP client to use for requests
    /// * `endpoint` - The full endpoint URL for the Azure Monitor ingestion API
    /// * `credential` - The token credential for authentication
    /// * `scope` - The OAuth scope for token acquisition
    ///
    /// # Returns
    /// * `LogsIngestionClient` - A configured client instance
    #[must_use]
    pub fn from_parts(http_client: Client, endpoint: String) -> Self {
        Self {
            http_client,
            endpoint,
            auth_header: HeaderValue::from_static("Bearer "), // placeholder, will be updated on first use
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
    pub fn new(config: &ApiConfig, http_client: Client) -> Result<Self, Error> {
        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.dcr_endpoint, config.dcr, config.stream_name
        );

        Ok(Self {
            http_client,
            endpoint,
            auth_header: HeaderValue::from_static("Bearer "), // placeholder, will be updated on first use
        })
    }

    /// Update the authorization header with a new access token.
    pub fn update_auth(&mut self, header: HeaderValue) {
        self.auth_header = header;
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
            401 => Err(Error::unauthorized(body)),
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
    use reqwest::header::HeaderValue;

    // ==================== Test Helpers ====================

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

        let http_client = create_test_http_client();

        let client =
            LogsIngestionClient::new(&api_config, http_client).expect("failed to create client");

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

        let http_client = create_test_http_client();

        let client = LogsIngestionClient::new(&api_config, http_client).unwrap();

        assert!(client.endpoint.contains("dcr-abc-123-def"));
        assert!(client.endpoint.contains("Custom-Stream_Name"));
    }

    #[test]
    fn test_from_parts_creates_client() {
        let client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
        );

        assert_eq!(client.endpoint, "https://example.com/endpoint");
        // auth_header is placeholder
        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));
    }

    #[test]
    fn test_new_initial_state() {
        let http_client = create_test_http_client();
        let api_config = create_test_api_config();

        let client = LogsIngestionClient::new(&api_config, http_client).unwrap();

        // Auth header is placeholder
        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));
    }

    // ==================== Auth Header Update Tests ====================

    #[test]
    fn test_update_auth_changes_header() {
        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
        );

        assert_eq!(client.auth_header, HeaderValue::from_static("Bearer "));

        client.update_auth(HeaderValue::from_static("Bearer new_token"));

        assert_eq!(
            client.auth_header,
            HeaderValue::from_static("Bearer new_token")
        );
    }

    #[test]
    fn test_update_auth_multiple_times() {
        let mut client = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
        );

        client.update_auth(HeaderValue::from_static("Bearer token1"));
        assert_eq!(
            client.auth_header,
            HeaderValue::from_static("Bearer token1")
        );

        client.update_auth(HeaderValue::from_static("Bearer token2"));
        assert_eq!(
            client.auth_header,
            HeaderValue::from_static("Bearer token2")
        );

        client.update_auth(HeaderValue::from_static("Bearer token3"));
        assert_eq!(
            client.auth_header,
            HeaderValue::from_static("Bearer token3")
        );
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
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1);
        let client = LogsIngestionClient::new(&api_config, create_test_http_client()).unwrap();
        pool.clients.push(client);

        assert_eq!(pool.clients.len(), 1);

        let taken = pool.take();
        assert_eq!(pool.clients.len(), 0);

        pool.release(taken);
        assert_eq!(pool.clients.len(), 1);
    }

    #[test]
    fn test_pool_take_and_release_multiple() {
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(3);
        for _ in 0..3 {
            let client = LogsIngestionClient::new(&api_config, create_test_http_client()).unwrap();
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
        let api_config = create_test_api_config();

        let mut pool = LogsIngestionClientPool::new(1);

        // Release more than capacity (Vec will grow)
        for _ in 0..5 {
            let client = LogsIngestionClient::new(&api_config, create_test_http_client()).unwrap();
            pool.release(client);
        }

        assert_eq!(pool.clients.len(), 5);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_client_clone_has_same_endpoint() {
        let client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com/endpoint".to_string(),
        );

        let client2 = client1.clone();

        assert_eq!(client1.endpoint, client2.endpoint);
    }

    #[test]
    fn test_client_clone_has_same_auth_header() {
        let mut client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
        );

        client1.update_auth(HeaderValue::from_static("Bearer test_token"));
        let client2 = client1.clone();

        assert_eq!(client1.auth_header, client2.auth_header);
    }

    #[test]
    fn test_client_clone_has_independent_header() {
        let mut client1 = LogsIngestionClient::from_parts(
            create_test_http_client(),
            "https://example.com".to_string(),
        );

        client1.update_auth(HeaderValue::from_static("Bearer token1"));
        let mut client2 = client1.clone();

        // Modify client2's header
        client2.update_auth(HeaderValue::from_static("Bearer token2"));

        // client1's header should be unchanged
        assert_eq!(
            client1.auth_header,
            HeaderValue::from_static("Bearer token1")
        );
        assert_eq!(
            client2.auth_header,
            HeaderValue::from_static("Bearer token2")
        );
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_client_with_empty_endpoint() {
        let client = LogsIngestionClient::from_parts(create_test_http_client(), "".to_string());

        assert_eq!(client.endpoint, "");
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
