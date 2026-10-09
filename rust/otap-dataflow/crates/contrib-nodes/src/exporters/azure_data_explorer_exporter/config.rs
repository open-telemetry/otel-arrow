// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::net::IpAddr;
use std::time::Duration;

use super::error::Error;

/// Conservative safety limit below ADX's 1 MiB dynamic-value limit.
pub(crate) const MAX_SAFE_ROW_BYTES: usize = 900 * 1024;
/// ADX streaming ingestion limit for one uncompressed request.
pub(crate) const MAX_STREAMING_REQUEST_BYTES: usize = 4 * 1024 * 1024;
/// Local work limit for rows expanded from one source message or request.
pub(crate) const MAX_STREAMING_REQUEST_ROWS: usize = 100_000;
/// Defensive upper bound for concurrent requests per exporter instance.
const MAX_IN_FLIGHT_REQUESTS: usize = 1_024;

/// Configuration for the Azure Data Explorer Exporter.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Kusto cluster URI for streaming ingestion.
    ///
    /// Use the **Cluster URI** (e.g. `https://mycluster.westus2.kusto.windows.net`),
    /// **not** the Data Ingestion URI (which has an `ingest-` prefix).
    /// The streaming ingestion REST API is served by the cluster endpoint.
    pub cluster_uri: String,

    /// Target database name.
    #[serde(default = "default_db_name")]
    pub db_name: String,

    /// Signal-specific ADX table and transformation settings.
    #[serde(default)]
    pub tables: TablesConfig,

    /// Deadline for one outbound ADX operation, including response handling,
    /// protocol retries, and retry backoff.
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,

    /// Gzip compression level (0-9, default 6).
    #[serde(default = "default_gzip_compression_level")]
    pub gzip_compression_level: u32,

    /// Maximum serialized JSON row size, including its JSON Lines separator.
    #[serde(default = "default_max_row_bytes")]
    pub max_row_bytes: usize,

    /// Maximum number of concurrent HTTP requests in flight.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,

    /// Maximum retries after the initial attempt for transient HTTP errors
    /// (429, 5xx) and network failures. Uses exponential backoff with jitter
    /// between retries (initial 3 s, capped at 30 s).
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Controls optional coalescing of inbound messages into fewer ADX requests.
    #[serde(default)]
    pub network_requests: NetworkRequestsConfig,

    /// Log the complete failed JSON batch at debug level for troubleshooting.
    /// Not recommended in production because payloads may contain sensitive
    /// telemetry.
    #[serde(default)]
    pub log_failed_payload: bool,

    /// Log ADX error response bodies at debug level for troubleshooting.
    /// Not recommended in production because responses may contain sensitive
    /// data.
    #[serde(default)]
    pub log_response_body: bool,
}

impl Config {
    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Error> {
        let cluster_uri = reqwest::Url::parse(&self.cluster_uri)
            .map_err(|error| Error::Config(format!("cluster_uri must be a valid URL: {error}")))?;
        let Some(host) = cluster_uri.host_str() else {
            return Err(Error::Config("cluster_uri must include a host".to_string()));
        };
        let is_loopback = host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<IpAddr>()
                .is_ok_and(|address| address.is_loopback());
        if cluster_uri.scheme() != "https" && !(cluster_uri.scheme() == "http" && is_loopback) {
            return Err(Error::Config(
                "cluster_uri must use https unless the host is loopback".to_string(),
            ));
        }
        if !cluster_uri.username().is_empty() || cluster_uri.password().is_some() {
            return Err(Error::Config(
                "cluster_uri must not include user information".to_string(),
            ));
        }
        if cluster_uri.query().is_some() || cluster_uri.fragment().is_some() {
            return Err(Error::Config(
                "cluster_uri must not include a query or fragment".to_string(),
            ));
        }
        if cluster_uri.path() != "/" && !cluster_uri.path().is_empty() {
            return Err(Error::Config(
                "cluster_uri must not include a path".to_string(),
            ));
        }
        if self.db_name.is_empty() {
            return Err(Error::Config("db_name must not be empty".to_string()));
        }
        if self.tables.logs.name.is_empty()
            || self.tables.metrics.name.is_empty()
            || self.tables.traces.name.is_empty()
        {
            return Err(Error::Config("table names must not be empty".to_string()));
        }
        if self.timeout.is_zero() {
            return Err(Error::Config(
                "timeout must be greater than zero".to_string(),
            ));
        }
        if self.max_in_flight == 0 || self.max_in_flight > MAX_IN_FLIGHT_REQUESTS {
            return Err(Error::Config(format!(
                "max_in_flight must be between 1 and {MAX_IN_FLIGHT_REQUESTS}"
            )));
        }
        if self.network_requests.coalesce && self.network_requests.max_pending_messages == 0 {
            return Err(Error::Config(
                "network_requests.max_pending_messages must be greater than zero".to_string(),
            ));
        }
        if self.gzip_compression_level > 9 {
            return Err(Error::Config(format!(
                "gzip_compression_level must be 0-9, got {}",
                self.gzip_compression_level
            )));
        }
        if self.max_row_bytes == 0 || self.max_row_bytes > MAX_SAFE_ROW_BYTES {
            return Err(Error::Config(format!(
                "max_row_bytes must be between 1 and {MAX_SAFE_ROW_BYTES}, got {}",
                self.max_row_bytes
            )));
        }
        if self.network_requests.max_rows == 0
            || self.network_requests.max_rows > MAX_STREAMING_REQUEST_ROWS
        {
            return Err(Error::Config(format!(
                "network_requests.max_rows must be between 1 and {MAX_STREAMING_REQUEST_ROWS}"
            )));
        }
        if self.network_requests.max_bytes == 0
            || self.network_requests.max_bytes > MAX_STREAMING_REQUEST_BYTES
        {
            return Err(Error::Config(format!(
                "network_requests.max_bytes must be between 1 and {MAX_STREAMING_REQUEST_BYTES}"
            )));
        }
        // Catch the common mistake of using the Data Ingestion URI
        // (ingest-*.kusto.windows.net) instead of the Cluster URI.
        if host.starts_with("ingest-") || host.starts_with("ingest.") {
            return Err(Error::Config(format!(
                "cluster_uri appears to be a Data Ingestion URI ('{}'). \
                 Use the Cluster URI instead (remove the 'ingest-' prefix). \
                 The streaming ingestion API is served by the cluster endpoint, \
                 not the data management service.",
                self.cluster_uri
            )));
        }
        Ok(())
    }
}

/// Signal-specific ADX table settings.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TablesConfig {
    /// Log table and transformation settings.
    pub logs: LogsTableConfig,

    /// Metrics table settings.
    pub metrics: MetricsTableConfig,

    /// Traces table settings.
    pub traces: TracesTableConfig,
}

/// ADX log table and transformation settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct LogsTableConfig {
    /// Target table name.
    #[serde(default = "default_logs_table")]
    pub name: String,

    /// Optional ADX JSON ingestion mapping name.
    pub mapping: Option<String>,

    /// Encode log bodies as strings for compatibility with legacy ADX tables.
    pub legacy_body_string: bool,

    /// Include the OTLP log record event name as the top-level `EventName` property.
    pub export_event_name: bool,

    /// Add `event.name` to `LogsAttributes` when the log record has an event
    /// name and the merged attributes do not already contain that key.
    pub add_event_name_to_attributes: bool,
}

impl Default for LogsTableConfig {
    fn default() -> Self {
        Self {
            name: default_logs_table(),
            mapping: None,
            legacy_body_string: false,
            export_event_name: true,
            add_event_name_to_attributes: true,
        }
    }
}

/// ADX metrics table settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsTableConfig {
    /// Target table name.
    #[serde(default = "default_metrics_table")]
    pub name: String,

    /// Optional ADX JSON ingestion mapping name.
    pub mapping: Option<String>,
}

impl Default for MetricsTableConfig {
    fn default() -> Self {
        Self {
            name: default_metrics_table(),
            mapping: None,
        }
    }
}

/// ADX traces table settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct TracesTableConfig {
    /// Target table name.
    #[serde(default = "default_traces_table")]
    pub name: String,

    /// Optional ADX JSON ingestion mapping name.
    pub mapping: Option<String>,
}

impl Default for TracesTableConfig {
    fn default() -> Self {
        Self {
            name: default_traces_table(),
            mapping: None,
        }
    }
}

/// Optional request coalescing configuration.
///
/// Coalescing retains source acknowledgements until ADX accepts the combined
/// request. `max_pending_messages` bounds those retained source messages per
/// exporter instance.
#[derive(Debug, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkRequestsConfig {
    /// Combine multiple inbound PData messages into fewer outbound requests.
    pub coalesce: bool,

    /// Maximum source PData messages retained across accumulators, retries, and
    /// in-flight coalesced requests.
    pub max_pending_messages: usize,

    /// Maximum number of JSON rows in one coalesced request.
    pub max_rows: usize,

    /// Maximum uncompressed bytes in one coalesced request.
    pub max_bytes: usize,

    /// Maximum time to hold a partial coalesced request when `coalesce` is
    /// enabled. Source messages remain unacknowledged until the request is
    /// flushed and ADX responds. Ignored when coalescing is disabled.
    #[serde(with = "humantime_serde")]
    pub flush_interval: Duration,
}

impl Default for NetworkRequestsConfig {
    fn default() -> Self {
        Self {
            coalesce: false,
            max_pending_messages: 16,
            max_rows: 1_000,
            max_bytes: 4 * 1024 * 1024,
            flush_interval: Duration::from_secs(1),
        }
    }
}

fn default_db_name() -> String {
    "oteldb".to_string()
}

fn default_logs_table() -> String {
    "OTELLogs".to_string()
}

fn default_metrics_table() -> String {
    "OTELMetrics".to_string()
}

fn default_traces_table() -> String {
    "OTELTraces".to_string()
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_gzip_compression_level() -> u32 {
    6
}

fn default_max_row_bytes() -> usize {
    MAX_SAFE_ROW_BYTES
}

fn default_max_in_flight() -> usize {
    16
}

fn default_max_retries() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validation_message(config: &Config) -> String {
        let Error::Config(message) = config
            .validate()
            .expect_err("configuration must be invalid")
        else {
            panic!("expected configuration error");
        };
        message
    }

    /// Scenario: only the required ADX cluster URI is configured.
    /// Guarantees: all documented defaults deserialize and pass validation.
    #[test]
    fn test_defaults() {
        let config: Config =
            serde_json::from_str(r#"{"cluster_uri": "https://mycluster.kusto.windows.net"}"#)
                .expect("deserialize");
        assert_eq!(config.db_name, "oteldb");
        assert_eq!(config.tables.logs.name, "OTELLogs");
        assert_eq!(config.tables.metrics.name, "OTELMetrics");
        assert_eq!(config.tables.traces.name, "OTELTraces");
        assert!(config.tables.logs.mapping.is_none());
        assert!(config.tables.metrics.mapping.is_none());
        assert!(config.tables.traces.mapping.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_row_bytes, MAX_SAFE_ROW_BYTES);
        assert!(!config.tables.logs.legacy_body_string);
        assert!(config.tables.logs.export_event_name);
        assert!(config.tables.logs.add_event_name_to_attributes);
        assert!(!config.log_failed_payload);
        assert!(!config.log_response_body);
        assert!(config.validate().is_ok());
    }

    /// Scenario: ADX response-body logging is explicitly enabled for troubleshooting.
    /// Guarantees: the opt-in setting deserializes independently of failed payload logging.
    #[test]
    fn response_body_logging_can_be_enabled() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "log_response_body": true
            }"#,
        )
        .expect("deserialize");
        assert!(config.log_response_body);
        assert!(!config.log_failed_payload);
    }

    /// Scenario: both log event-name output options are explicitly disabled.
    /// Guarantees: configuration preserves each independently selectable behavior.
    #[test]
    fn event_name_options_can_be_disabled() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "tables": {
                    "logs": {
                        "export_event_name": false,
                        "add_event_name_to_attributes": false
                    }
                }
            }"#,
        )
        .expect("deserialize");

        assert!(!config.tables.logs.export_event_name);
        assert!(!config.tables.logs.add_event_name_to_attributes);
    }

    /// Scenario: an environment-substituted ADX mapping name is empty.
    /// Guarantees: an empty optional mapping remains valid so the request can omit it.
    #[test]
    fn empty_mapping_is_allowed() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "tables": {
                    "logs": { "mapping": "" }
                }
            }"#,
        )
        .expect("deserialize");

        assert!(config.validate().is_ok());
    }

    /// Scenario: an ADX operation timeout is explicitly configured.
    /// Guarantees: the duration is parsed and retained for outbound request deadlines.
    #[test]
    fn custom_operation_timeout_is_deserialized() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "timeout": "2m"
            }"#,
        )
        .expect("deserialize");

        assert_eq!(config.timeout, Duration::from_secs(120));
    }

    /// Scenario: the ADX operation timeout is configured as zero.
    /// Guarantees: validation rejects a deadline that would make every export fail immediately.
    #[test]
    fn zero_operation_timeout_is_rejected() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "timeout": "0s"
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "timeout must be greater than zero"
        );
    }

    /// Scenario: the configured serialized-row limit is zero or exceeds the ADX safety ceiling.
    /// Guarantees: validation rejects limits that disable protection or exceed the supported bound.
    #[test]
    fn invalid_max_row_bytes_is_rejected() {
        for max_row_bytes in [0, MAX_SAFE_ROW_BYTES + 1] {
            let config: Config = serde_json::from_value(serde_json::json!({
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "max_row_bytes": max_row_bytes
            }))
            .expect("deserialize");

            assert!(config.validate().is_err());
        }
    }

    /// Scenario: an operator configures a stricter positive serialized-row limit.
    /// Guarantees: validation accepts the operator-selected bound.
    #[test]
    fn stricter_max_row_bytes_is_allowed() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "max_row_bytes": 1024
            }"#,
        )
        .expect("deserialize");

        assert!(config.validate().is_ok());
        assert_eq!(config.max_row_bytes, 1024);
    }

    /// Scenario: legacy exporter-local authentication fields are configured.
    /// Guarantees: configuration fails so credentials can only come from the bound capability.
    #[test]
    fn exporter_local_auth_is_rejected() {
        let error = serde_json::from_str::<Config>(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "auth": { "method": "managed_identity" }
            }"#,
        )
        .expect_err("legacy auth must be rejected");

        assert!(error.to_string().contains("unknown field `auth`"));
    }

    /// Scenario: table names, mappings, and log controls are configured by signal.
    /// Guarantees: every nested table setting deserializes into its corresponding field.
    #[test]
    fn signal_table_settings_are_deserialized() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "tables": {
                    "logs": {
                        "name": "Logs",
                        "mapping": "LogsMapping",
                        "legacy_body_string": true,
                        "export_event_name": false,
                        "add_event_name_to_attributes": false
                    },
                    "metrics": {
                        "name": "Metrics",
                        "mapping": "MetricsMapping"
                    },
                    "traces": {
                        "name": "Traces",
                        "mapping": "TracesMapping"
                    }
                }
            }"#,
        )
        .expect("deserialize");

        assert_eq!(config.tables.logs.name, "Logs");
        assert_eq!(config.tables.logs.mapping.as_deref(), Some("LogsMapping"));
        assert!(config.tables.logs.legacy_body_string);
        assert!(!config.tables.logs.export_event_name);
        assert!(!config.tables.logs.add_event_name_to_attributes);
        assert_eq!(config.tables.metrics.name, "Metrics");
        assert_eq!(
            config.tables.metrics.mapping.as_deref(),
            Some("MetricsMapping")
        );
        assert_eq!(config.tables.traces.name, "Traces");
        assert_eq!(
            config.tables.traces.mapping.as_deref(),
            Some("TracesMapping")
        );
    }

    /// Scenario: an ADX exporter configuration omits the required cluster URI value.
    /// Guarantees: validation rejects the exporter before runtime initialization.
    #[test]
    fn test_validate_missing_cluster() {
        let config: Config = serde_json::from_str(r#"{"cluster_uri": ""}"#).expect("deserialize");
        assert!(config.validate().is_err());
    }

    /// Scenario: network request settings are omitted.
    /// Guarantees: request coalescing is opt-in and retains conservative bounded defaults.
    #[test]
    fn test_batch_defaults() {
        let config: Config =
            serde_json::from_str(r#"{"cluster_uri": "https://mycluster.kusto.windows.net"}"#)
                .expect("deserialize");
        assert!(!config.network_requests.coalesce);
        assert_eq!(config.network_requests.max_pending_messages, 16);
        assert_eq!(config.network_requests.max_rows, 1_000);
        assert_eq!(config.network_requests.max_bytes, 4 * 1024 * 1024);
        assert_eq!(
            config.network_requests.flush_interval,
            Duration::from_secs(1)
        );
    }

    /// Scenario: request coalescing and custom bounds are configured.
    /// Guarantees: the opt-in and every grouped request bound deserialize without conversion errors.
    #[test]
    fn test_batch_custom() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "network_requests": {
                    "coalesce": true,
                    "max_pending_messages": 8,
                    "max_rows": 5000,
                    "max_bytes": 2097152,
                    "flush_interval": "1s"
                }
            }"#,
        )
        .expect("deserialize");
        assert!(config.network_requests.coalesce);
        assert_eq!(config.network_requests.max_pending_messages, 8);
        assert_eq!(config.network_requests.max_rows, 5000);
        assert_eq!(config.network_requests.max_bytes, 2097152);
        assert_eq!(
            config.network_requests.flush_interval,
            Duration::from_secs(1)
        );
    }

    /// Scenario: the retained-message limit is configured as zero.
    /// Guarantees: validation rejects an unbounded or deadlocked coalescing gate.
    #[test]
    fn zero_max_pending_messages_is_rejected() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "network_requests": {
                    "coalesce": true,
                    "max_pending_messages": 0
                }
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "network_requests.max_pending_messages must be greater than zero"
        );
    }

    /// Scenario: request coalescing is disabled and its pending-message limit is zero.
    /// Guarantees: an inactive coalescing limit does not prevent exporter startup.
    #[test]
    fn zero_max_pending_messages_is_allowed_when_coalescing_is_disabled() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "network_requests": {
                    "coalesce": false,
                    "max_pending_messages": 0
                }
            }"#,
        )
        .expect("deserialize");

        assert!(config.validate().is_ok());
    }

    /// Scenario: gzip compression is configured above flate2's supported maximum.
    /// Guarantees: validation rejects the invalid compression level before startup.
    #[test]
    fn test_validate_gzip_level_too_high() {
        let config: Config = serde_json::from_str(
            r#"{"cluster_uri": "https://mycluster.kusto.windows.net", "gzip_compression_level": 10}"#,
        )
        .expect("deserialize");
        assert!(config.validate().is_err());
    }

    /// Scenario: gzip compression is configured at the supported maximum.
    /// Guarantees: validation accepts level nine.
    #[test]
    fn test_validate_gzip_level_max_ok() {
        let config: Config = serde_json::from_str(
            r#"{"cluster_uri": "https://mycluster.kusto.windows.net", "gzip_compression_level": 9}"#,
        )
        .expect("deserialize");
        assert!(config.validate().is_ok());
    }

    /// Scenario: a configuration still contains the removed ingestion-type selector.
    /// Guarantees: deserialization rejects the obsolete field with an actionable unknown-field error.
    #[test]
    fn removed_ingestion_type_is_rejected() {
        let error = serde_json::from_str::<Config>(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "ingestion_type": "managed"
            }"#,
        )
        .expect_err("removed ingestion type must be rejected");

        assert!(error.to_string().contains("unknown field `ingestion_type`"));
    }

    /// Scenario: the configured in-flight request limit is zero.
    /// Guarantees: validation rejects an unusable queue bound before the exporter can poll an empty queue.
    #[test]
    fn zero_max_in_flight_is_rejected() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://mycluster.kusto.windows.net",
                "max_in_flight": 0
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "max_in_flight must be between 1 and 1024"
        );
    }

    /// Scenario: a cluster URI uses cleartext HTTP.
    /// Guarantees: validation prevents bearer tokens from being sent without TLS.
    #[test]
    fn http_cluster_uri_is_rejected() {
        let config: Config =
            serde_json::from_str(r#"{"cluster_uri": "http://example.kusto.windows.net"}"#)
                .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "cluster_uri must use https unless the host is loopback"
        );
    }

    /// Scenario: a local ADX-compatible development endpoint uses HTTP.
    /// Guarantees: loopback testing remains possible without allowing remote cleartext tokens.
    #[test]
    fn loopback_http_cluster_uri_is_allowed() {
        for cluster_uri in ["http://localhost:8080", "http://127.0.0.1:8080"] {
            let config: Config = serde_json::from_value(serde_json::json!({
                "cluster_uri": cluster_uri
            }))
            .expect("deserialize");

            assert!(config.validate().is_ok(), "{cluster_uri}");
        }
    }

    /// Scenario: a cluster URI includes an endpoint path.
    /// Guarantees: validation accepts only a cluster origin used to construct ADX paths.
    #[test]
    fn cluster_uri_with_path_is_rejected() {
        let config: Config =
            serde_json::from_str(r#"{"cluster_uri": "https://example.kusto.windows.net/custom"}"#)
                .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "cluster_uri must not include a path"
        );
    }

    /// Scenario: the request byte limit exceeds ADX's streaming ingestion limit.
    /// Guarantees: validation caps every uncompressed request at 4 MiB.
    #[test]
    fn request_byte_limit_above_service_limit_is_rejected() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://example.kusto.windows.net",
                "network_requests": {"max_bytes": 4194305}
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "network_requests.max_bytes must be between 1 and 4194304"
        );
    }

    /// Scenario: the transformed-row limit is disabled with zero.
    /// Guarantees: every source message has a finite transformation work budget.
    #[test]
    fn zero_request_row_limit_is_rejected() {
        let config: Config = serde_json::from_str(
            r#"{
                "cluster_uri": "https://example.kusto.windows.net",
                "network_requests": {"max_rows": 0}
            }"#,
        )
        .expect("deserialize");

        assert_eq!(
            validation_message(&config),
            "network_requests.max_rows must be between 1 and 100000"
        );
    }
}
