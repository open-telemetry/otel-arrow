// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use azure_core::credentials::TokenCredential;
use azure_identity::{
    DeveloperToolsCredential, DeveloperToolsCredentialOptions, ManagedIdentityCredential,
    ManagedIdentityCredentialOptions, UserAssignedId,
};
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use crate::experimental::azure_monitor_exporter::config::{AuthMethod, Config};

/// HTTP client for Azure Log Analytics Data Collection Rule (DCR) endpoint.
///
/// Handles authentication, compression, and HTTP communication with the
/// Azure Monitor Logs Ingestion API.
pub struct LogsIngestionClient {
    http_client: Client,
    endpoint: String,
    credential: Arc<dyn TokenCredential>,
    scope: String,
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
            credential,
            scope,
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
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        // Build the endpoint URL from config components
        let endpoint = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.api.dcr_endpoint, config.api.dcr, config.api.stream_name
        );

        // Create credential based on auth method in config
        let credential = Self::create_credential(&config.auth)?;

        Ok(Self {
            http_client,
            endpoint,
            credential,
            scope: config.auth.scope.clone(),
        })
    }

    // TODO: Remove print_stdout after logging is set up
    #[allow(clippy::print_stdout)]
    /// Creates the appropriate credential based on the authentication method in config.
    ///
    /// # Arguments
    /// * `config` - The authentication configuration
    ///
    /// # Returns
    /// * `Ok(Arc<dyn TokenCredential>)` - The configured credential
    /// * `Err(String)` - Error message if credential creation fails
    fn create_credential(
        auth_config: &crate::experimental::azure_monitor_exporter::config::AuthConfig,
    ) -> Result<Arc<dyn TokenCredential>, String> {
        match auth_config.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &auth_config.client_id {
                    // User-assigned managed identity
                    println!("Using user-assigned managed identity with client_id: {client_id}");
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                } else {
                    // System-assigned managed identity
                    println!("Using system-assigned managed identity");
                    // user_assigned_id remains None for system-assigned
                }

                let credential = ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| format!("Failed to create managed identity credential: {e}"))?;

                Ok(credential as Arc<dyn TokenCredential>)
            }
            AuthMethod::Development => {
                println!("Using developer tools credential (Azure CLI / Azure Developer CLI)");
                // DeveloperToolsCredential tries Azure CLI and Azure Developer CLI
                let credential =
                    DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                        .map_err(|e| {
                            format!(
                                "Failed to create developer tools credential: {e}. \
                            Ensure Azure CLI or Azure Developer CLI is installed and logged in"
                            )
                        })?;

                Ok(credential as Arc<dyn TokenCredential>)
            }
        }
    }

    /// Compress data using gzip
    fn gzip_compress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|e| format!("Failed to write to gzip encoder: {e}"))?;
        encoder
            .finish()
            .map_err(|e| format!("Failed to finish gzip compression: {e}"))
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
    pub async fn send(&self, body: impl Serialize) -> Result<(), String> {
        // Use scope from config instead of hardcoded value
        let token_response = self
            .credential
            .get_token(
                &[&self.scope],
                Some(azure_core::credentials::TokenRequestOptions::default()),
            )
            .await
            .map_err(|e| format!("Failed to get token: {e}"))?;

        let token = token_response.token.secret();

        // Serialize to JSON
        let json_bytes =
            serde_json::to_vec(&body).map_err(|e| format!("Failed to serialize to JSON: {e}"))?;

        // Compress the JSON
        let compressed_body = self.gzip_compress(&json_bytes)?;

        // Build headers
        let mut headers = HeaderMap::new();
        let _ = headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let _ = headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        let _ = headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| "Invalid token format".to_string())?,
        );

        // TODO: Log as debug after logging is set up (here for debugging for now)
        let compression_ratio = json_bytes.len() as f64 / compressed_body.len() as f64;
        println!(
            "Compressed {} bytes to {} bytes (ratio: {:.2}x)",
            json_bytes.len(),
            compressed_body.len(),
            compression_ratio
        );

        // Send compressed body
        let response = self
            .http_client
            .post(&self.endpoint)
            .headers(headers)
            .body(compressed_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await.unwrap_or_default();

            match status.as_u16() {
                401 => return Err(format!("Authentication failed: {error}")),
                403 => return Err(format!("Authorization failed: {error}")),
                413 => return Err("Payload too large - reduce batch size".to_string()),
                429 => return Err(format!("Rate limited: {error}")),
                _ => return Err(format!("Request failed ({status}): {error}")),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
    use time::{Duration, OffsetDateTime};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method},
    };

    #[derive(Debug)]
    struct FakeCredential;

    #[async_trait::async_trait]
    impl TokenCredential for FakeCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            Ok(AccessToken::new(
                "fake-token",
                OffsetDateTime::now_utc() + Duration::hours(1),
            ))
        }
    }

    #[tokio::test]
    async fn test_send_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("content-encoding", "gzip"))
            .and(header("authorization", "Bearer fake-token"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = LogsIngestionClient::from_parts(
            Client::new(),
            server.uri(),
            Arc::new(FakeCredential),
            "https://monitor.azure.com/.default".into(),
        );

        let body = serde_json::json!({"test": "data"});
        let result = client.send(body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_auth_failure() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let client = LogsIngestionClient::from_parts(
            Client::new(),
            server.uri(),
            Arc::new(FakeCredential),
            "https://monitor.azure.com/.default".into(),
        );

        let result = client.send(serde_json::json!({"test": "data"})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Authentication failed"));
    }

    #[test]
    fn test_gzip_compress() {
        let client = LogsIngestionClient::from_parts(
            Client::new(),
            String::new(),
            Arc::new(FakeCredential),
            String::new(),
        );

        let data = b"test data to compress";
        let compressed = client.gzip_compress(data).unwrap();

        // Verify it's actually compressed (should be smaller for repetitive data)
        assert!(!compressed.is_empty());

        // Verify gzip magic bytes
        assert_eq!(compressed[0], 0x1f);
        assert_eq!(compressed[1], 0x8b);
    }

    #[tokio::test]
    async fn test_error_responses() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

        let cases = vec![
            (403, "Authorization failed"),
            (413, "Payload too large"),
            (429, "Rate limited"),
            (500, "Request failed"),
        ];

        for (status, expected_msg) in cases {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;

            let client = LogsIngestionClient::from_parts(
                Client::new(),
                server.uri(),
                Arc::new(FakeCredential),
                "scope".into(),
            );

            let result = client.send(serde_json::json!({"test": "data"})).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().contains(expected_msg));
        }
    }

    #[tokio::test]
    async fn send_happy_path_compresses_and_posts() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{header, method, path, query_param},
        };
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("content-encoding", "gzip"))
            .and(path("/dataCollectionRules/dcr/streams/stream"))
            .and(query_param("api-version", "2021-11-01-preview"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = Client::new();
        let logs_client = LogsIngestionClient::from_parts(
            client,
            format!(
                "{}/dataCollectionRules/dcr/streams/stream?api-version=2021-11-01-preview",
                server.uri()
            ),
            Arc::new(FakeCredential),
            "https://monitor.azure.com/.default".to_string(),
        );

        logs_client
            .send(vec![serde_json::json!({"foo": "bar"})])
            .await
            .unwrap();
    }

    #[test]
    fn test_create_credential_managed_identity_system_assigned() {
        use crate::experimental::azure_monitor_exporter::config::AuthConfig;

        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: None,
            scope: "https://monitor.azure.com/.default".to_string(),
        };

        let result = LogsIngestionClient::create_credential(&auth_config);
        assert!(
            result.is_ok(),
            "Should successfully create system-assigned managed identity credential"
        );
    }

    #[test]
    fn test_create_credential_managed_identity_user_assigned() {
        use crate::experimental::azure_monitor_exporter::config::AuthConfig;

        let auth_config = AuthConfig {
            method: AuthMethod::ManagedIdentity,
            client_id: Some("test-client-id-12345".to_string()),
            scope: "https://monitor.azure.com/.default".to_string(),
        };

        let result = LogsIngestionClient::create_credential(&auth_config);
        assert!(
            result.is_ok(),
            "Should successfully create user-assigned managed identity credential"
        );
    }

    #[test]
    fn test_create_credential_development() {
        use crate::experimental::azure_monitor_exporter::config::AuthConfig;

        let auth_config = AuthConfig {
            method: AuthMethod::Development,
            client_id: None,
            scope: "https://monitor.azure.com/.default".to_string(),
        };

        let result = LogsIngestionClient::create_credential(&auth_config);
        assert!(
            result.is_ok(),
            "Should successfully create developer tools credential"
        );
    }
}
