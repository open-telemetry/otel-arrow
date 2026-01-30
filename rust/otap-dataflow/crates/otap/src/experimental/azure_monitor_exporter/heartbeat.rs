// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

use super::AZURE_MONITOR_EXPORTER_URN;
use super::config::ApiConfig;
use super::error::Error;
use chrono::Utc;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue},
};
use std::time::Duration;
use sysinfo::System;

const HEARTBEAT_STREAM_NAME: &str = "HEALTH_ASSESSMENT_BLOB";
const MAX_IDLE_CONNECTIONS_PER_HOST: usize = 2;

/// Heartbeat client for Azure Monitor Exporter.
pub struct Heartbeat {
    client: Client,
    endpoint: String,
    heartbeat_row: HeartbeatRow,

    /// Pre-formatted authorization header for zero-allocation reuse
    pub auth_header: HeaderValue,
}

#[derive(Serialize)]
struct HeartbeatRow {
    #[serde(rename = "Time")]
    time: String,

    #[serde(rename = "Version")]
    version: String,

    #[serde(rename = "OSName")]
    os_name: String,

    #[serde(rename = "Computer")]
    computer: String,

    #[serde(rename = "OSMajorVersion")]
    os_major_version: String,

    #[serde(rename = "OSMinorVersion")]
    os_minor_version: String,
}

#[inline]
fn default_heartbeat_version() -> String {
    std::env::var("IMAGE").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())
}

#[inline]
fn default_heartbeat_os_name() -> String {
    System::name().unwrap_or_else(|| std::env::consts::OS.to_string())
}

#[inline]
fn default_heartbeat_computer() -> String {
    std::env::var("ARM_RESOURCE_ID")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| System::host_name().unwrap_or_else(|| "UnknownComputer".to_string()))
}

/// Parse OS version into (major, minor) components
/// e.g., "22.04.3" -> ("22", "04")
/// e.g., "10 (22H2)" -> ("10", "22H2")
/// e.g., "5.15.0-generic" -> ("5", "15")
fn parse_os_version() -> (String, String) {
    let version = System::os_version().unwrap_or_default();

    if version.is_empty() {
        return ("Unknown".to_string(), "Unknown".to_string());
    }

    // Handle Windows format: "10 (22H2)"
    if let Some(paren_start) = version.find('(') {
        let major = version[..paren_start].trim().to_string();
        let minor = version[paren_start..]
            .trim_matches(|c| c == '(' || c == ')')
            .to_string();
        return (major, minor);
    }

    // Handle semver format: "22.04.3" or "5.15.0-generic"
    let parts: Vec<&str> = version.split('.').collect();
    match parts.as_slice() {
        [major] => (major.to_string(), "0".to_string()),
        [major, minor, ..] => (major.to_string(), minor.to_string()),
        [] => ("Unknown".to_string(), "Unknown".to_string()),
    }
}

#[inline]
fn default_heartbeat_os_major_version() -> String {
    std::env::var("POD_NAME").unwrap_or_else(|_| {
        let (major, _) = parse_os_version();
        major
    })
}

#[inline]
fn default_heartbeat_os_minor_version() -> String {
    AZURE_MONITOR_EXPORTER_URN.to_string()
}

impl Heartbeat {
    /// Create a new Heartbeat instance.
    pub fn new(config: &ApiConfig) -> Result<Self, Error> {
        let http_client = Client::builder()
            .http1_only()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(MAX_IDLE_CONNECTIONS_PER_HOST)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_nodelay(true)
            .build()
            .map_err(Error::CreateClient)?;

        Ok(Self {
            client: http_client,
            endpoint: format!(
                "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
                config.dcr_endpoint, config.dcr, HEARTBEAT_STREAM_NAME
            ),
            heartbeat_row: HeartbeatRow {
                time: Utc::now().to_rfc3339(),
                version: default_heartbeat_version(),
                os_name: default_heartbeat_os_name(),
                computer: default_heartbeat_computer(),
                os_major_version: default_heartbeat_os_major_version(),
                os_minor_version: default_heartbeat_os_minor_version(),
            },
            auth_header: HeaderValue::from_static("Bearer "),
        })
    }

    /// Create a Heartbeat from individual components (for testing).
    #[cfg(test)]
    #[must_use]
    pub fn from_parts(client: Client, endpoint: String) -> Self {
        Self {
            client,
            endpoint,
            heartbeat_row: HeartbeatRow {
                time: Utc::now().to_rfc3339(),
                version: "test-version".to_string(),
                os_name: "test-os".to_string(),
                computer: "test-computer".to_string(),
                os_major_version: "1".to_string(),
                os_minor_version: "0".to_string(),
            },
            auth_header: HeaderValue::from_static("Bearer "),
        }
    }

    /// Update the authorization header with a new access token.
    pub fn update_auth(&mut self, header: HeaderValue) {
        self.auth_header = header;
    }

    /// Send a heartbeat to the Azure Monitor Logs Ingestion endpoint.
    pub async fn send(&mut self) -> Result<(), Error> {
        self.heartbeat_row.time = Utc::now().to_rfc3339();
        let payload = serde_json::json!([self.heartbeat_row]);
        let response = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, &self.auth_header)
            .body(payload.to_string())
            .send()
            .await
            .map_err(Error::network)?;

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
    use std::collections::HashMap;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // ==================== Test Helpers ====================

    fn create_test_client() -> Client {
        Client::builder().build().unwrap()
    }

    // ==================== parse_os_version Tests ====================

    #[test]
    fn test_parse_os_version_semver() {
        // Can't mock System::os_version, but we can test the parsing logic
        // by checking the function returns reasonable values
        let (major, minor) = parse_os_version();
        // Should return some string (either parsed or "Unknown")
        assert!(!major.is_empty());
        assert!(!minor.is_empty());
    }

    // ==================== HeartbeatRow Serialization Tests ====================

    #[test]
    fn test_heartbeat_row_serialization() {
        let row = HeartbeatRow {
            time: "2026-01-22T10:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            os_name: "Linux".to_string(),
            computer: "test-computer".to_string(),
            os_major_version: "22".to_string(),
            os_minor_version: "04".to_string(),
        };

        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["Time"], "2026-01-22T10:00:00Z");
        assert_eq!(json["Version"], "1.0.0");
        assert_eq!(json["OSName"], "Linux");
        assert_eq!(json["Computer"], "test-computer");
        assert_eq!(json["OSMajorVersion"], "22");
        assert_eq!(json["OSMinorVersion"], "04");
    }

    #[test]
    fn test_heartbeat_row_serialization_field_names() {
        let row = HeartbeatRow {
            time: "".to_string(),
            version: "".to_string(),
            os_name: "".to_string(),
            computer: "".to_string(),
            os_major_version: "".to_string(),
            os_minor_version: "".to_string(),
        };

        let json = serde_json::to_string(&row).unwrap();

        // Verify PascalCase field names
        assert!(json.contains("\"Time\""));
        assert!(json.contains("\"Version\""));
        assert!(json.contains("\"OSName\""));
        assert!(json.contains("\"Computer\""));
        assert!(json.contains("\"OSMajorVersion\""));
        assert!(json.contains("\"OSMinorVersion\""));

        // Verify no snake_case field names
        assert!(!json.contains("\"time\""));
        assert!(!json.contains("\"version\""));
        assert!(!json.contains("\"os_name\""));
    }

    #[test]
    fn test_heartbeat_payload_is_array() {
        let row = HeartbeatRow {
            time: "2026-01-22T10:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            os_name: "Linux".to_string(),
            computer: "test".to_string(),
            os_major_version: "22".to_string(),
            os_minor_version: "04".to_string(),
        };

        let payload = serde_json::json!([row]);
        assert!(payload.is_array());
        assert_eq!(payload.as_array().unwrap().len(), 1);
    }

    // ==================== Endpoint Construction Tests ====================

    #[test]
    fn test_endpoint_format() {
        let config = ApiConfig {
            dcr_endpoint: "https://test.ingest.monitor.azure.com".to_string(),
            dcr: "dcr-abc123".to_string(),
            stream_name: "Custom-Logs".to_string(),
            schema: super::super::config::SchemaConfig {
                resource_mapping: HashMap::new(),
                scope_mapping: HashMap::new(),
                log_record_mapping: HashMap::new(),
            },
        };

        let expected = format!(
            "{}/dataCollectionRules/{}/streams/{}?api-version=2021-11-01-preview",
            config.dcr_endpoint, config.dcr, HEARTBEAT_STREAM_NAME
        );

        assert_eq!(
            expected,
            "https://test.ingest.monitor.azure.com/dataCollectionRules/dcr-abc123/streams/HEALTH_ASSESSMENT_BLOB?api-version=2021-11-01-preview"
        );
    }

    // ==================== Default Value Tests ====================

    #[test]
    fn test_default_heartbeat_version_fallback() {
        // When IMAGE env var is not set, should use CARGO_PKG_VERSION
        let version = default_heartbeat_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_default_heartbeat_os_name_returns_value() {
        let os_name = default_heartbeat_os_name();
        assert!(!os_name.is_empty());
    }

    #[test]
    fn test_default_heartbeat_computer_fallback() {
        let computer = default_heartbeat_computer();
        assert!(!computer.is_empty());
    }

    #[test]
    fn test_default_heartbeat_os_major_version_returns_value() {
        let major = default_heartbeat_os_major_version();
        assert!(!major.is_empty());
    }

    #[test]
    fn test_default_heartbeat_os_minor_version_returns_value() {
        let minor = default_heartbeat_os_minor_version();
        assert!(!minor.is_empty());
    }

    // ==================== Constants Tests ====================

    #[test]
    fn test_heartbeat_stream_name() {
        assert_eq!(HEARTBEAT_STREAM_NAME, "HEALTH_ASSESSMENT_BLOB");
    }

    #[test]
    fn test_max_idle_connections() {
        assert_eq!(MAX_IDLE_CONNECTIONS_PER_HOST, 2);
    }

    // ==================== Send Method Tests ====================

    #[tokio::test]
    async fn test_send_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        // Set up auth header for the test
        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth { kind, .. } => {
                assert!(matches!(
                    kind,
                    super::super::error::AuthErrorKind::Unauthorized
                ));
            }
            e => panic!("Expected Auth/Unauthorized error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_forbidden() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Auth { kind, .. } => {
                assert!(matches!(
                    kind,
                    super::super::error::AuthErrorKind::Forbidden
                ));
            }
            e => panic!("Expected Auth/Forbidden error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_string("Too Many Requests")
                    .insert_header("Retry-After", "60"),
            )
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RateLimited { retry_after, .. } => {
                assert_eq!(retry_after, Some(Duration::from_secs(60)));
            }
            e => panic!("Expected RateLimited error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ServerError { status, .. } => {
                assert_eq!(status.as_u16(), 500);
            }
            e => panic!("Expected ServerError, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_payload_too_large() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(413))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::PayloadTooLarge => {}
            e => panic!("Expected PayloadTooLarge error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_unexpected_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(418).set_body_string("I'm a teapot"))
            .mount(&mock_server)
            .await;

        let mut heartbeat = Heartbeat::from_parts(create_test_client(), mock_server.uri());

        heartbeat.update_auth(HeaderValue::from_static("Bearer test_token"));
        let result = heartbeat.send().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedStatus { status, .. } => {
                assert_eq!(status.as_u16(), 418);
            }
            e => panic!("Expected UnexpectedStatus error, got: {:?}", e),
        }
    }

    // ==================== from_parts Tests ====================

    #[test]
    fn test_from_parts_creates_heartbeat() {
        let heartbeat =
            Heartbeat::from_parts(create_test_client(), "https://example.com".to_string());

        assert_eq!(heartbeat.endpoint, "https://example.com");
        // Verify heartbeat row has default values
        assert!(!heartbeat.heartbeat_row.version.is_empty());
        assert!(!heartbeat.heartbeat_row.os_name.is_empty());
        assert!(!heartbeat.heartbeat_row.computer.is_empty());
    }

    // ==================== update_auth Tests ====================

    #[test]
    fn test_update_auth_changes_header() {
        let mut heartbeat =
            Heartbeat::from_parts(create_test_client(), "https://example.com".to_string());

        assert_eq!(heartbeat.auth_header, HeaderValue::from_static("Bearer "));

        heartbeat.update_auth(HeaderValue::from_static("Bearer new_token"));

        assert_eq!(
            heartbeat.auth_header,
            HeaderValue::from_static("Bearer new_token")
        );
    }
}
