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
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigPatch {
    pub endpoint: String,
    pub database: String,
    pub username: String,
    pub password: SecretString,

    pub async_insert: Option<bool>,

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
    /// Clickhouse password
    pub password: SecretString,
    /// Use async insert
    pub async_insert: bool,
    pub table_defaults: DefaultTableConfig,
    pub tables: TablesConfig,
}

impl Config {
    pub fn from_patch(p: ConfigPatch) -> Self {
        let async_insert = p.async_insert.unwrap_or(true);

        let tables = TablesConfig::from_patch(p.tables);

        Self {
            endpoint: p.endpoint,
            database: p.database,
            username: p.username,
            password: p.password,
            async_insert,
            table_defaults: p.table_defaults,
            tables,
        }
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
        assert_eq!(config.password.expose_secret(), "secret");
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
}
