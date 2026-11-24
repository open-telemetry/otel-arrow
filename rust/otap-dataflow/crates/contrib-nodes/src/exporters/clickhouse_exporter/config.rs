// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the OTAP ClickHouse exporter.
//!
//! This module defines the user-facing configuration model (deserialized via `serde`) and the
//! internal, fully-resolved configuration used by the exporter at runtime.
//!
//! It provides:
//!
//! - A patch-based configuration flow:
//!   - [`ConfigPatch`] is the deserializable representation (typically from YAML/JSON), using
//!     optional fields and defaults.
//!   - [`Config`] is the normalized runtime configuration produced by [`Config::from_patch`],
//!     including defaulting (e.g. `async_insert`) and table config expansion.
//!
//! - Table configuration primitives:
//!   - [`TableEngine`] describes the ClickHouse engine name and optional parameters.
//!   - [`DefaultTableConfig`] provides global defaults (TTL interval, engine, schema creation).
//!   - [`TableConfigPatch`] and [`TableConfig`] model per-table overrides (names, TTL, engine
//!     overrides, and schema creation behavior).
//!   - [`MetricsTableConfigPatch`] / [`MetricsTableConfig`] group metric tables by type.
//!   - [`TablesConfigPatch`] / [`TablesConfig`] contain the full set of signal table
//!     configurations, with sensible defaults for common table names.
//!
//! The merge helpers (`from_patch` and `merge_table`) apply patch overrides on top of defaults so
//! downstream code can generate schemas and write data without needing to reason about missing
//! configuration fields.
use otel_arrow_dfe_config::tls::TlsClientConfig;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::num::NonZeroUsize;

const DEFAULT_MAX_IN_FLIGHT: NonZeroUsize =
    NonZeroUsize::new(10).expect("default max_in_flight must be non-zero");

/// How the exporter authenticates to ClickHouse over HTTP(S).
#[derive(Debug, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClickhouseAuthMode {
    /// Authenticate with `username` and `password`
    /// (`X-ClickHouse-User` / `X-ClickHouse-Key` headers).
    #[default]
    Password,
    /// Authenticate as `username` using the TLS client certificate presented
    /// during the handshake. ClickHouse matches the certificate's CN/SAN
    /// against the user's `ssl_certificates` definition. Requires an
    /// `https://` endpoint and a client certificate and key in `tls`.
    SslCertificate,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigPatch {
    pub endpoint: String,
    pub database: String,
    pub username: String,
    /// Required when `auth` is `password`; must be absent or empty when `auth`
    /// is `ssl_certificate`.
    #[serde(default)]
    pub password: Option<SecretString>,

    /// Authentication method used to log in to ClickHouse.
    #[serde(default)]
    pub auth: ClickhouseAuthMode,

    pub async_insert: Option<bool>,

    /// Maximum number of ClickHouse insert requests allowed to run concurrently.
    pub max_in_flight: Option<NonZeroUsize>,

    /// Client-side TLS/mTLS configuration for the ClickHouse HTTP connection.
    #[serde(default)]
    pub tls: Option<TlsClientConfig>,

    #[serde(default)]
    pub table_defaults: DefaultTableConfig,

    #[serde(default)]
    pub tables: TablesConfigPatch,
}

/// Configuration for the Clickhouse Exporter
#[derive(Debug, Clone)]
pub struct Config {
    /// ClickHouse HTTP(S) endpoint URL (e.g. "http://localhost:8123"). TCP is not supported for now.
    pub endpoint: String,
    /// Database to write to (e.g., "otap")
    pub database: String,
    /// Clickhouse user name
    pub username: String,
    /// Clickhouse password (only used when `auth` is `password`)
    pub password: Option<SecretString>,
    /// Authentication method used to log in to ClickHouse
    pub auth: ClickhouseAuthMode,
    /// Use async insert
    pub async_insert: bool,
    /// Maximum number of ClickHouse insert requests allowed to run concurrently.
    pub max_in_flight: NonZeroUsize,
    /// Client-side TLS/mTLS configuration for the ClickHouse HTTP connection.
    pub tls: Option<TlsClientConfig>,
    pub table_defaults: DefaultTableConfig,
    pub tables: TablesConfig,
}

impl Config {
    pub fn from_patch(p: ConfigPatch) -> Self {
        let async_insert = p.async_insert.unwrap_or(true);
        let max_in_flight = p.max_in_flight.unwrap_or(DEFAULT_MAX_IN_FLIGHT);

        let tables = TablesConfig::from_patch(p.tables);

        Self {
            endpoint: p.endpoint,
            database: p.database,
            username: p.username,
            password: p.password,
            auth: p.auth,
            async_insert,
            max_in_flight,
            tls: p.tls,
            table_defaults: p.table_defaults,
            tables,
        }
    }

    /// Validates cross-field constraints that serde cannot express.
    ///
    /// Returns a human-readable error describing the first violated constraint.
    pub fn validate(&self) -> Result<(), String> {
        match self.auth {
            ClickhouseAuthMode::Password => {
                if self.password.is_none() {
                    return Err("password is required when auth is 'password'; set \
                         password (it may be empty) or use auth: ssl_certificate"
                        .to_string());
                }
            }
            ClickhouseAuthMode::SslCertificate => {
                // ClickHouse falls back to the default session user when
                // X-ClickHouse-User is empty, which would silently log in as a
                // different user than intended.
                if self.username.is_empty() {
                    return Err("username must not be empty when auth is \
                         'ssl_certificate'; set it to the ClickHouse user the \
                         client certificate identifies"
                        .to_string());
                }
                // ClickHouse rejects certificate login combined with a
                // password ("not allowed to use SSL certificate authentication
                // and authentication via password simultaneously").
                if self
                    .password
                    .as_ref()
                    .is_some_and(|p| !p.expose_secret().is_empty())
                {
                    return Err("password must not be set when auth is \
                         'ssl_certificate'; ClickHouse rejects certificate \
                         login combined with a password"
                        .to_string());
                }
                if !self.endpoint.starts_with("https://") {
                    return Err("auth 'ssl_certificate' requires an https:// \
                         endpoint"
                        .to_string());
                }
                let has_identity = self.tls.as_ref().is_some_and(|tls| {
                    let tls = &tls.config;
                    (tls.cert_file.is_some() || tls.cert_pem.is_some())
                        && (tls.key_file.is_some() || tls.key_pem.is_some())
                });
                if !has_identity {
                    return Err("auth 'ssl_certificate' requires a client \
                         certificate and key in the tls block (cert_file or \
                         cert_pem, and key_file or key_pem)"
                        .to_string());
                }
            }
        }
        Ok(())
    }
}

/// Configuration for a ClickHouse table engine
#[derive(Debug, Deserialize, Clone)]
pub struct TableEngine {
    pub name: String,
    #[serde(default)]
    pub params: Option<String>, // e.g., "('/clickhouse/tables/{shard}/{db}/{table}', '{replica}')"
}
impl Default for TableEngine {
    fn default() -> Self {
        Self {
            name: "MergeTree".into(),
            params: None, //Some("('/clickhouse/tables/{uuid}/{shard}', '{replica}')".into()),
        }
    }
}

/// Configuration for a single table, Option values are overriden by global defaults if None.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TableConfigPatch {
    pub name: Option<String>,
    pub ttl: Option<String>,
    pub engine: Option<TableEngine>,
    pub create_schema: Option<bool>,
}

/// Configuration for a single table, Option values are overriden by global defaults if None.
#[derive(Debug, Clone, Default)]
pub struct TableConfig {
    /// Logical name (users query this)
    pub name: String,

    /// TTL, e.g., "72 HOUR"
    pub ttl: Option<String>,

    /// Optional table engine override; falls back to top-level default
    pub engine: Option<TableEngine>,

    /// Whether to create the schema automatically or not, use global value if None.
    pub create_schema: Option<bool>,
}

/// Default Table configuration settings
#[derive(Debug, Deserialize, Clone)]
pub struct DefaultTableConfig {
    /// TTL INTERVAL, e.g., "72 HOUR"
    #[serde(default)]
    pub ttl_interval: Option<String>,

    /// Optional table engine override
    #[serde(default)]
    pub engine: TableEngine,

    /// Whether to create the schema automatically or not
    #[serde(default = "default_true")]
    pub create_schema: bool,
}
impl Default for DefaultTableConfig {
    fn default() -> Self {
        Self {
            ttl_interval: None,
            engine: TableEngine::default(),
            create_schema: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Configuration for metrics tables by type
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct MetricsTableConfigPatch {
    pub gauge: Option<TableConfigPatch>,
    pub sum: Option<TableConfigPatch>,
    pub summary: Option<TableConfigPatch>,
    pub histogram: Option<TableConfigPatch>,
    pub exponential_histogram: Option<TableConfigPatch>,
}

/// Configuration for metrics tables by type
#[derive(Debug, Clone)]
pub struct MetricsTableConfig {
    pub gauge: TableConfig,
    pub sum: TableConfig,
    pub summary: TableConfig,
    pub histogram: TableConfig,
    pub exponential_histogram: TableConfig,
}
impl Default for MetricsTableConfig {
    fn default() -> Self {
        Self {
            gauge: TableConfig {
                name: "otel_metrics_gauge".to_string(),
                ..Default::default()
            },
            sum: TableConfig {
                name: "otel_metrics_sum".to_string(),
                ..Default::default()
            },
            summary: TableConfig {
                name: "otel_metrics_summary".to_string(),
                ..Default::default()
            },
            histogram: TableConfig {
                name: "otel_metrics_histogram".to_string(),
                ..Default::default()
            },
            exponential_histogram: TableConfig {
                name: "otel_metrics_exp_histogram".to_string(),
                ..Default::default()
            },
        }
    }
}

impl MetricsTableConfig {
    pub fn from_patch(p: MetricsTableConfigPatch) -> Self {
        let defaults = MetricsTableConfig::default();

        Self {
            gauge: merge_table(defaults.gauge, p.gauge),
            sum: merge_table(defaults.sum, p.sum),
            summary: merge_table(defaults.summary, p.summary),

            histogram: merge_table(defaults.histogram, p.histogram),
            exponential_histogram: merge_table(
                defaults.exponential_histogram,
                p.exponential_histogram,
            ),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TablesConfigPatch {
    pub logs: Option<TableConfigPatch>,
    pub traces: Option<TableConfigPatch>,
    pub metrics: Option<MetricsTableConfigPatch>,
}

/// Full tables configuration
#[derive(Debug, Clone)]
pub struct TablesConfig {
    /// Main logs and traces tables
    pub logs: TableConfig,
    pub traces: TableConfig,

    /// Metrics tables grouped by type
    pub metrics: MetricsTableConfig,
}

impl Default for TablesConfig {
    fn default() -> Self {
        Self {
            logs: TableConfig {
                name: "otel_logs".to_string(),
                ..Default::default()
            },
            traces: TableConfig {
                name: "otel_traces".to_string(),
                ..Default::default()
            },
            metrics: MetricsTableConfig::default(),
        }
    }
}

impl TablesConfig {
    pub fn from_patch(p: TablesConfigPatch) -> Self {
        let defaults = TablesConfig::default();

        Self {
            logs: merge_table(defaults.logs, p.logs),
            traces: merge_table(defaults.traces, p.traces),
            metrics: p
                .metrics
                .map(MetricsTableConfig::from_patch)
                .unwrap_or(defaults.metrics),
        }
    }
}

fn merge_table(default: TableConfig, patch: Option<TableConfigPatch>) -> TableConfig {
    match patch {
        None => default,
        Some(p) => TableConfig {
            name: p.name.unwrap_or(default.name),
            ttl: p.ttl.or(default.ttl),
            engine: p.engine.or(default.engine),
            create_schema: p.create_schema.or(default.create_schema),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    /// Scenario: a ClickHouse exporter config omits the concurrency setting.
    /// Guarantees: the runtime configuration permits ten concurrent inserts by default.
    #[test]
    fn max_in_flight_defaults_to_ten() {
        let patch: ConfigPatch = serde_json::from_value(serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret"
        }))
        .unwrap();

        assert_eq!(Config::from_patch(patch).max_in_flight.get(), 10);
    }

    /// Scenario: a ClickHouse exporter config requests OTC-equivalent concurrency.
    /// Guarantees: the configured maximum is preserved in the runtime configuration.
    #[test]
    fn max_in_flight_accepts_configured_value() {
        let patch: ConfigPatch = serde_json::from_value(serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "max_in_flight": 10
        }))
        .unwrap();

        assert_eq!(Config::from_patch(patch).max_in_flight.get(), 10);
    }

    /// Scenario: a ClickHouse exporter config sets the concurrency limit to zero.
    /// Guarantees: invalid zero-capacity configurations are rejected during deserialization.
    #[test]
    fn max_in_flight_rejects_zero() {
        let error = serde_json::from_value::<ConfigPatch>(serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "max_in_flight": 0
        }))
        .unwrap_err();

        assert!(error.to_string().contains("nonzero usize"));
    }

    /// Scenario: all supported top-level and nested exporter fields are configured.
    /// Guarantees: explicit values are retained and unspecified table values use defaults.
    #[test]
    fn test_config_deserialization() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "async_insert": false,
            "table_defaults": {
                "ttl_interval": "48 HOUR",
                "engine": {
                    "name": "ReplacingMergeTree"
                }
            },
            "tables": {
                "logs": {
                    "ttl": "12 HOUR"
                },
                "metrics": {
                    "gauge": {
                        "name": "custom_metrics_gauge"
                    }
                }
            }
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config: Config = Config::from_patch(patch);

        // --- Top-level fields ---
        assert_eq!(config.endpoint, "http://localhost:8123");
        assert_eq!(config.database, "otap");
        assert_eq!(config.username, "clickhouse");
        assert_eq!(
            config.password.as_ref().map(|p| p.expose_secret()),
            Some("secret")
        );
        assert_eq!(config.auth, ClickhouseAuthMode::Password);
        assert!(!config.async_insert);

        // --- Table defaults ---
        assert_eq!(
            config.table_defaults.ttl_interval,
            Some("48 HOUR".to_string())
        );
        assert!(config.table_defaults.create_schema);
        assert_eq!(config.table_defaults.engine.name, "ReplacingMergeTree");
        assert!(config.table_defaults.engine.params.is_none());

        // --- Tables ---
        assert_eq!(config.tables.logs.name, "otel_logs");
        assert_eq!(config.tables.logs.ttl.as_deref(), Some("12 HOUR"));

        // Defaulted tables still present
        assert_eq!(config.tables.traces.name, "otel_traces");

        // Metrics table override
        assert_eq!(config.tables.metrics.gauge.name, "custom_metrics_gauge");

        // Default metrics tables
        assert_eq!(config.tables.metrics.sum.name, "otel_metrics_sum");
    }

    /// Scenario: a ClickHouse exporter config contains an unsupported top-level field.
    /// Guarantees: deserialization rejects the typo instead of silently ignoring it.
    #[test]
    fn test_config_patch_rejects_unknown_fields() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "unknown_field": true
        });

        let err = serde_json::from_value::<ConfigPatch>(json).unwrap_err();
        assert!(err.to_string().contains("unknown field `unknown_field`"));
    }

    /// Scenario: a ClickHouse exporter config includes a TLS block with mTLS fields.
    /// Guarantees: the TLS configuration is deserialized and propagated to the runtime config.
    #[test]
    fn tls_config_deserializes_mtls_fields() {
        let json = serde_json::json!({
            "endpoint": "https://clickhouse.example.com:8443",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "tls": {
                "ca_file": "/certs/ca.crt",
                "cert_file": "/certs/client.crt",
                "key_file": "/certs/client.key"
            }
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config = Config::from_patch(patch);

        let tls = config.tls.expect("tls config must be present");
        assert_eq!(
            tls.ca_file.as_ref().unwrap().to_str().unwrap(),
            "/certs/ca.crt"
        );
        assert_eq!(
            tls.config.cert_file.as_ref().unwrap().to_str().unwrap(),
            "/certs/client.crt"
        );
        assert_eq!(
            tls.config.key_file.as_ref().unwrap().to_str().unwrap(),
            "/certs/client.key"
        );
    }

    /// Scenario: a ClickHouse exporter config omits the TLS block.
    /// Guarantees: the runtime config has no TLS configuration.
    #[test]
    fn tls_config_absent_when_not_specified() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret"
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config = Config::from_patch(patch);
        assert!(config.tls.is_none());
    }

    /// Scenario: only the required connection fields are configured.
    /// Guarantees: runtime table names and table settings match the documented defaults.
    #[test]
    fn test_config_defaults_match_spec() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret"
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config = Config::from_patch(patch);

        assert!(config.async_insert);
        assert!(config.table_defaults.create_schema);
        assert_eq!(config.table_defaults.engine.name, "MergeTree");
    }

    /// Parse a JSON config and run cross-field validation, mirroring what the
    /// exporter factory does at startup.
    fn parse_and_validate(json: serde_json::Value) -> Result<Config, String> {
        let patch: ConfigPatch = serde_json::from_value(json).map_err(|e| e.to_string())?;
        let config = Config::from_patch(patch);
        config.validate()?;
        Ok(config)
    }

    /// Build a certificate-auth config JSON with an mTLS client identity; the
    /// caller may then tweak individual fields.
    fn ssl_cert_json() -> serde_json::Value {
        serde_json::json!({
            "endpoint": "https://clickhouse:8443",
            "database": "otel",
            "username": "telemetry_ingest",
            "auth": "ssl_certificate",
            "tls": {
                "ca_file": "/certs/ca.crt",
                "cert_file": "/certs/tls.crt",
                "key_file": "/certs/tls.key"
            }
        })
    }

    /// Scenario: a config omits `auth`.
    /// Guarantees: the exporter defaults to password authentication so that
    /// existing configurations keep their current login behavior.
    #[test]
    fn auth_defaults_to_password() {
        let config = parse_and_validate(serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": ""
        }))
        .unwrap();
        assert_eq!(config.auth, ClickhouseAuthMode::Password);
    }

    /// Scenario: password auth (the default) is used but `password` is omitted.
    /// Guarantees: validation rejects the config, so a forgotten password is
    /// not silently treated as an empty one.
    #[test]
    fn password_auth_requires_password() {
        let err = parse_and_validate(serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse"
        }))
        .unwrap_err();
        assert!(err.contains("password is required"), "{err}");
    }

    /// Scenario: `auth: ssl_certificate` with an mTLS client identity and no
    /// `password` field.
    /// Guarantees: certificate-only configs are accepted without a password
    /// and the auth mode is carried into the runtime config.
    #[test]
    fn ssl_certificate_auth_accepts_missing_password() {
        let config = parse_and_validate(ssl_cert_json()).unwrap();
        assert_eq!(config.auth, ClickhouseAuthMode::SslCertificate);
        assert!(config.password.is_none());
    }

    /// Scenario: `auth: ssl_certificate` with an explicitly empty password and
    /// an inline (PEM) client identity.
    /// Guarantees: an empty password is accepted (ClickHouse permits an empty
    /// key alongside certificate login) and inline PEM material counts as a
    /// client identity.
    #[test]
    fn ssl_certificate_auth_accepts_empty_password_and_inline_pem() {
        let mut json = ssl_cert_json();
        json["password"] = serde_json::json!("");
        json["tls"] = serde_json::json!({
            "cert_pem": "-----BEGIN CERTIFICATE-----",
            "key_pem": "-----BEGIN PRIVATE KEY-----"
        });
        assert!(parse_and_validate(json).is_ok());
    }

    /// Scenario: `auth: ssl_certificate` combined with a non-empty password.
    /// Guarantees: validation rejects the config because ClickHouse refuses
    /// certificate login mixed with password authentication.
    #[test]
    fn ssl_certificate_auth_rejects_non_empty_password() {
        let mut json = ssl_cert_json();
        json["password"] = serde_json::json!("secret");
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("password must not be set"), "{err}");
    }

    /// Scenario: `auth: ssl_certificate` without any tls block.
    /// Guarantees: validation rejects the config because no client certificate
    /// would be presented for ClickHouse to authenticate.
    #[test]
    fn ssl_certificate_auth_rejects_missing_tls_block() {
        let mut json = ssl_cert_json();
        let _ = json.as_object_mut().unwrap().remove("tls");
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("client certificate and key"), "{err}");
    }

    /// Scenario: `auth: ssl_certificate` with a client certificate but no key.
    /// Guarantees: validation rejects an incomplete client identity.
    #[test]
    fn ssl_certificate_auth_rejects_cert_without_key() {
        let mut json = ssl_cert_json();
        json["tls"] = serde_json::json!({
            "ca_file": "/certs/ca.crt",
            "cert_file": "/certs/tls.crt"
        });
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("client certificate and key"), "{err}");
    }

    /// Scenario: `auth: ssl_certificate` with an http:// endpoint.
    /// Guarantees: validation rejects the config because a client certificate
    /// can only be presented over TLS.
    #[test]
    fn ssl_certificate_auth_rejects_http_endpoint() {
        let mut json = ssl_cert_json();
        json["endpoint"] = serde_json::json!("http://clickhouse:8123");
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("https://"), "{err}");
    }

    /// Scenario: `auth: ssl_certificate` with an empty username.
    /// Guarantees: validation rejects the config instead of letting ClickHouse
    /// fall back to its default session user.
    #[test]
    fn ssl_certificate_auth_rejects_empty_username() {
        let mut json = ssl_cert_json();
        json["username"] = serde_json::json!("");
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("username must not be empty"), "{err}");
    }

    /// Scenario: `auth` is set to an unsupported value.
    /// Guarantees: deserialization rejects the value instead of silently
    /// falling back to password authentication.
    #[test]
    fn auth_rejects_unknown_mode() {
        let mut json = ssl_cert_json();
        json["auth"] = serde_json::json!("kerberos");
        let err = parse_and_validate(json).unwrap_err();
        assert!(err.contains("unknown variant `kerberos`"), "{err}");
    }
}
