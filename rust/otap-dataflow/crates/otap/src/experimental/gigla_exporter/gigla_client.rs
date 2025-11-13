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

use crate::experimental::gigla_exporter::config::{AuthMethod, Config};

/// HTTP client for Azure Log Analytics Data Collection Rule (DCR) endpoint.
///
/// Handles authentication, compression, and HTTP communication with the Azure
/// Monitor ingestion API.
pub struct GigLaClient {
    http_client: Client,
    endpoint: String,
    credential: Arc<dyn TokenCredential>,
    scope: String,
}

impl GigLaClient {
    /// Creates a new GigLA client instance from the configuration.
    ///
    /// # Arguments
    /// * `config` - The GigLA exporter configuration
    ///
    /// # Returns
    /// * `Ok(GigLaClient)` - A configured client instance
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
        let credential: Arc<dyn TokenCredential> = match config.auth.method {
            AuthMethod::ManagedIdentity => {
                let mut options = ManagedIdentityCredentialOptions::default();

                if let Some(client_id) = &config.auth.client_id {
                    // User-assigned managed identity
                    log::info!("Using user-assigned managed identity with client_id: {client_id}");
                    options.user_assigned_id = Some(UserAssignedId::ClientId(client_id.clone()));
                } else {
                    // System-assigned managed identity
                    log::info!("Using system-assigned managed identity");
                    // user_assigned_id remains None for system-assigned
                }

                ManagedIdentityCredential::new(Some(options))
                    .map_err(|e| format!("Failed to create managed identity credential: {e}"))?
            }
            AuthMethod::Development => {
                log::info!("Using developer tools credential (Azure CLI / Azure Developer CLI)");
                // DeveloperToolsCredential tries Azure CLI and Azure Developer CLI
                DeveloperToolsCredential::new(Some(DeveloperToolsCredentialOptions::default()))
                    .map_err(|e| {
                        format!(
                            "Failed to create developer tools credential: {e}. \
                        Ensure Azure CLI or Azure Developer CLI is installed and logged in"
                        )
                    })?
            }
        };

        Ok(Self {
            http_client,
            endpoint,
            credential,
            scope: config.auth.scope.clone(),
        })
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
            .get_token(&[&self.scope], None)
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

        // Log compression ratio for debugging
        let compression_ratio = json_bytes.len() as f64 / compressed_body.len() as f64;
        log::debug!(
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
