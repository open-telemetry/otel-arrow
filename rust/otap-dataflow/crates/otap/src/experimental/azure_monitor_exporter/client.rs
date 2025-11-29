// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::TokenCredential;
use azure_core::time::OffsetDateTime;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    // Use Instant for faster comparisons (monotonic clock)
    token_valid_until: Instant,
    // Pre-built static headers that never change
    static_headers: HeaderMap,
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
        // Pre-build static headers
        let mut static_headers = HeaderMap::with_capacity(2);
        _ = static_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        _ = static_headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));

        Self {
            http_client,
            endpoint,
            auth: Auth::from_credential(credential, scope),
            auth_header: HeaderValue::from_static("Bearer "),
            token_valid_until: Instant::now(),
            static_headers,
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
            .pool_max_idle_per_host(10) // Reuse connections
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        // Build the endpoint URL from config components
        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.api.dcr_endpoint, config.api.dcr, config.api.stream_name
        );

        // Create auth handler
        let auth =
            Auth::new(&config.auth).map_err(|e| format!("Failed to create auth handler: {e}"))?;

        // Pre-build static headers
        let mut static_headers = HeaderMap::with_capacity(2);
        _ = static_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        _ = static_headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));

        Ok(Self {
            http_client,
            endpoint,
            auth,
            auth_header: HeaderValue::from_static("Bearer "),
            token_valid_until: Instant::now(),
            static_headers,
        })
    }

    /// Refresh the token if needed and update the pre-formatted header
    #[inline]
    async fn ensure_valid_token(&mut self) -> Result<(), String> {
        let now = Instant::now();

        // Fast path: token is still valid
        if now < self.token_valid_until {
            return Ok(());
        }

        // Slow path: need to refresh token
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
            .whole_seconds()
            .saturating_sub(300); // 5 minutes = 300 seconds

        self.token_valid_until = now + Duration::from_secs(valid_seconds.max(0) as u64);

        Ok(())
    }

    // TODO: Remove print_stdout after logging is set up
    #[allow(clippy::print_stdout)]
    /// Send compressed data to Log Analytics ingestion API.
    ///
    /// # Arguments
    /// * `body` - The data to send (must be serializable to JSON)
    ///
    /// # Returns
    /// * `Ok(())` - If the request was successful
    /// * `Err(String)` - Error message if the request failed
    pub async fn send(&mut self, body: Vec<u8>) -> Result<(), String> {
        // Ensure we have a valid token (fast path when cached)
        self.ensure_valid_token().await?;

        // Clone static headers and add the auth header
        let mut headers = self.static_headers.clone();
        _ = headers.insert(AUTHORIZATION, self.auth_header.clone());

        let start = Instant::now();

        // Send compressed body
        let response = self
            .http_client
            .post(&self.endpoint)
            .headers(headers)
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
                self.auth.invalidate_token();
                Err(format!("Authentication failed: {error}"))
            }
            403 => Err(format!("Authorization failed: {error}")),
            413 => Err("Payload too large - reduce batch size".to_string()),
            429 => Err(format!("Rate limited: {error}")),
            _ => Err(format!("Request failed ({status}): {error}")),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
//     use time::{Duration, OffsetDateTime};
//     use wiremock::{
//         Mock, MockServer, ResponseTemplate,
//         matchers::{header, method},
//     };

//     #[derive(Debug)]
//     struct FakeCredential;

//     #[async_trait::async_trait]
//     impl TokenCredential for FakeCredential {
//         async fn get_token(
//             &self,
//             _scopes: &[&str],
//             _options: Option<TokenRequestOptions<'_>>,
//         ) -> azure_core::Result<AccessToken> {
//             Ok(AccessToken::new(
//                 "fake-token",
//                 OffsetDateTime::now_utc() + Duration::hours(1),
//             ))
//         }
//     }

//     #[tokio::test]
//     async fn test_send_success() {
//         let server = MockServer::start().await;

//         Mock::given(method("POST"))
//             .and(header("content-encoding", "gzip"))
//             .and(header("authorization", "Bearer fake-token"))
//             .respond_with(ResponseTemplate::new(200))
//             .mount(&server)
//             .await;

//         let client = LogsIngestionClient::from_parts(
//             Client::new(),
//             server.uri(),
//             Arc::new(FakeCredential),
//             "https://monitor.azure.com/.default".into(),
//         );

//         let body = serde_json::json!({"test": "data"});
//         let result = client.send(body).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_send_auth_failure() {
//         let server = MockServer::start().await;

//         Mock::given(method("POST"))
//             .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
//             .mount(&server)
//             .await;

//         let client = LogsIngestionClient::from_parts(
//             Client::new(),
//             server.uri(),
//             Arc::new(FakeCredential),
//             "https://monitor.azure.com/.default".into(),
//         );

//         let result = client.send(serde_json::json!({"test": "data"})).await;
//         assert!(result.is_err());
//         assert!(result.unwrap_err().contains("Authentication failed"));
//     }

//     #[test]
//     fn test_gzip_compress() {
//         let client = LogsIngestionClient::from_parts(
//             Client::new(),
//             String::new(),
//             Arc::new(FakeCredential),
//             String::new(),
//         );

//         let data = b"test data to compress";
//         let compressed = client.gzip_compress(data).unwrap();

//         // Verify it's actually compressed (should be smaller for repetitive data)
//         assert!(!compressed.is_empty());

//         // Verify gzip magic bytes
//         assert_eq!(compressed[0], 0x1f);
//         assert_eq!(compressed[1], 0x8b);
//     }

//     #[tokio::test]
//     async fn test_error_responses() {
//         use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

//         let cases = vec![
//             (403, "Authorization failed"),
//             (413, "Payload too large"),
//             (429, "Rate limited"),
//             (500, "Request failed"),
//         ];

//         for (status, expected_msg) in cases {
//             let server = MockServer::start().await;

//             Mock::given(method("POST"))
//                 .respond_with(ResponseTemplate::new(status))
//                 .mount(&server)
//                 .await;

//             let client = LogsIngestionClient::from_parts(
//                 Client::new(),
//                 server.uri(),
//                 Arc::new(FakeCredential),
//                 "scope".into(),
//             );

//             let result = client.send(serde_json::json!({"test": "data"})).await;
//             assert!(result.is_err());
//             assert!(result.unwrap_err().contains(expected_msg));
//         }
//     }

//     #[tokio::test]
//     async fn send_happy_path_compresses_and_posts() {
//         use wiremock::{
//             Mock, MockServer, ResponseTemplate,
//             matchers::{header, method, path, query_param},
//         };
//         let server = MockServer::start().await;

//         Mock::given(method("POST"))
//             .and(header("content-encoding", "gzip"))
//             .and(path("/dataCollectionRules/dcr/streams/stream"))
//             .and(query_param("api-version", "2021-11-01-preview"))
//             .respond_with(ResponseTemplate::new(200))
//             .mount(&server)
//             .await;

//         let client = Client::new();
//         let logs_client = LogsIngestionClient::from_parts(
//             client,
//             format!(
//                 "{}/dataCollectionRules/dcr/streams/stream?api-version=2021-11-01-preview",
//                 server.uri()
//             ),
//             Arc::new(FakeCredential),
//             "https://monitor.azure.com/.default".to_string(),
//         );

//         logs_client
//             .send(vec![serde_json::json!({"foo": "bar"})])
//             .await
//             .unwrap();
//     }

//     #[test]
//     fn test_create_credential_managed_identity_system_assigned() {
//         use crate::experimental::azure_monitor_exporter::config::AuthConfig;

//         let auth_config = AuthConfig {
//             method: AuthMethod::ManagedIdentity,
//             client_id: None,
//             scope: "https://monitor.azure.com/.default".to_string(),
//         };

//         let result = LogsIngestionClient::create_credential(&auth_config);
//         assert!(
//             result.is_ok(),
//             "Should successfully create system-assigned managed identity credential"
//         );
//     }

//     #[test]
//     fn test_create_credential_managed_identity_user_assigned() {
//         use crate::experimental::azure_monitor_exporter::config::AuthConfig;

//         let auth_config = AuthConfig {
//             method: AuthMethod::ManagedIdentity,
//             client_id: Some("test-client-id-12345".to_string()),
//             scope: "https://monitor.azure.com/.default".to_string(),
//         };

//         let result = LogsIngestionClient::create_credential(&auth_config);
//         assert!(
//             result.is_ok(),
//             "Should successfully create user-assigned managed identity credential"
//         );
//     }

//     #[test]
//     fn test_create_credential_development() {
//         use crate::experimental::azure_monitor_exporter::config::AuthConfig;

//         let auth_config = AuthConfig {
//             method: AuthMethod::Development,
//             client_id: None,
//             scope: "https://monitor.azure.com/.default".to_string(),
//         };

//         let result = LogsIngestionClient::create_credential(&auth_config);
//         assert!(
//             result.is_ok(),
//             "Should successfully create developer tools credential"
//         );
//     }
// }
