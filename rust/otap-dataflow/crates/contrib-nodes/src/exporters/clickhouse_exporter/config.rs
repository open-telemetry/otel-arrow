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
//! - Attribute configuration primitives:
//!   - [`AttributeRepresentation`] selects how attribute values are encoded in ClickHouse
//!     (string map, JSON, or OTAP array form).
//!   - [`AttributeStorage`] selects whether attribute sets are stored inline on signal tables or in
//!     separate lookup tables (optionally deduped).
//!   - [`AttributeConfig`] combines representation + storage and enforces validation during
//!     deserialization (notably: `OtapArray` cannot be stored inline).
//!   - [`AttributesConfig`] groups attribute configs for resource, scope, log, metric, and trace
//!     attributes.
//!
//! - Table configuration primitives:
//!   - [`TableEngine`] describes the ClickHouse engine name and optional parameters.
//!   - [`DefaultTableConfig`] provides global defaults (TTL interval, engine, schema creation).
//!   - [`TableConfigPatch`] and [`TableConfig`] model per-table overrides (names, physical table
//!     names, TTL, engine overrides, and schema creation behavior).
//!   - [`MetricsTableConfigPatch`] / [`MetricsTableConfig`] group metric tables by type.
//!   - [`TablesConfigPatch`] / [`TablesConfig`] contain the full set of signal and attribute table
//!     configurations, with sensible defaults for common table names.
//!
//! The merge helpers (`from_patch` and `merge_table`) apply patch overrides on top of defaults so
//! downstream code can generate schemas and write data without needing to reason about missing
//! configuration fields.
use serde::Deserialize;
use serde::de::{self, Deserializer};

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigPatch {
    pub endpoint: String,
    pub database: String,
    pub username: String,
    pub password: String,

    pub async_insert: Option<bool>,

    #[serde(default)]
    pub attributes: AttributesConfig,

    #[serde(default)]
    pub table_defaults: DefaultTableConfig,

    #[serde(default)]
    pub tables: TablesConfigPatch,
}

/// Configuration for the Clickhouse Exporter
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Clickhouse endpoint URL
    pub endpoint: String,
    /// Database to write to (e.g., "otap")
    pub database: String,
    /// Clickhouse user name
    pub username: String,
    /// Clickhouse password
    pub password: String,
    /// Use anync insert
    pub async_insert: bool,
    /// Attribute configurations
    pub attributes: AttributesConfig,
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
            attributes: p.attributes,
            table_defaults: p.table_defaults,
            tables,
        }
    }
}

/// How attribute values are represented in ClickHouse
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AttributeRepresentation {
    #[default]
    StringMap,
    Json,
    OtapArray,
}

/// How attribute sets are stored
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AttributeStorage {
    Inline, // Denormalized, values stored directly in main table
    #[default]
    Lookup, // Stored in separate table, referenced by foreign key
    LookupDeduped, // Stored in separate table with deduplication
}

/// All attribute groups
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AttributesConfig {
    pub resource: AttributeConfig,
    pub scope: AttributeConfig,
    pub log: AttributeConfig,
    pub metric: AttributeConfig,
    pub trace: AttributeConfig,
}

/// AttributeConfig with validation during deserialization
#[derive(Debug, Clone, Default)]
pub struct AttributeConfig {
    pub representation: AttributeRepresentation,
    pub storage: AttributeStorage,
}

impl<'de> Deserialize<'de> for AttributeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            representation: AttributeRepresentation,
            #[serde(default)]
            storage: AttributeStorage,
        }

        let helper = Helper::deserialize(deserializer)?;

        // Validation rules
        if let (AttributeRepresentation::OtapArray, AttributeStorage::Inline) =
            (&helper.representation, &helper.storage)
        {
            return Err(de::Error::custom(
                "OtapArray representation cannot be stored inline",
            ));
        }

        Ok(AttributeConfig {
            representation: helper.representation,
            storage: helper.storage,
        })
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
    pub physical_name: Option<String>,
    pub ttl: Option<String>,
    pub engine: Option<TableEngine>,
    pub create_schema: Option<bool>,
}

/// Configuration for a single table, Option values are overriden by global defaults if None.
#[derive(Debug, Clone, Default)]
pub struct TableConfig {
    /// Logical name (users query this)
    pub name: String,

    /// Optional physical table name. If None and attributes are lookup, defaults to "{name}_normalized"
    pub physical_name: Option<String>,

    /// TTL, e.g., "72h"
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

    pub resource_attributes: Option<TableConfigPatch>,
    pub scope_attributes: Option<TableConfigPatch>,
    pub log_attributes: Option<TableConfigPatch>,
    pub metric_attributes: Option<TableConfigPatch>,
    pub trace_attributes: Option<TableConfigPatch>,
}

/// Full tables configuration
#[derive(Debug, Clone)]
pub struct TablesConfig {
    /// Main logs and traces tables
    pub logs: TableConfig,
    pub traces: TableConfig,

    /// Metrics tables grouped by type
    pub metrics: MetricsTableConfig,

    /// Attribute lookup tables (only created if attributes are lookup/deduped)
    pub resource_attributes: TableConfig,
    pub scope_attributes: TableConfig,
    pub log_attributes: TableConfig,
    pub metric_attributes: TableConfig,
    pub trace_attributes: TableConfig,
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
            resource_attributes: TableConfig {
                name: "otel_resource_attributes".to_string(),
                ..Default::default()
            },
            scope_attributes: TableConfig {
                name: "otel_scope_attributes".to_string(),
                ..Default::default()
            },
            log_attributes: TableConfig {
                name: "otel_log_attributes".to_string(),
                ..Default::default()
            },
            metric_attributes: TableConfig {
                name: "otel_metric_attributes".to_string(),
                ..Default::default()
            },
            trace_attributes: TableConfig {
                name: "otel_trace_attributes".to_string(),
                ..Default::default()
            },
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

            resource_attributes: merge_table(defaults.resource_attributes, p.resource_attributes),
            scope_attributes: merge_table(defaults.scope_attributes, p.scope_attributes),
            log_attributes: merge_table(defaults.log_attributes, p.log_attributes),
            metric_attributes: merge_table(defaults.metric_attributes, p.metric_attributes),
            trace_attributes: merge_table(defaults.trace_attributes, p.trace_attributes),
        }
    }
}

fn merge_table(default: TableConfig, patch: Option<TableConfigPatch>) -> TableConfig {
    match patch {
        None => default,
        Some(p) => TableConfig {
            name: p.name.unwrap_or(default.name),
            physical_name: p.physical_name.or(default.physical_name),
            ttl: p.ttl.or(default.ttl),
            engine: p.engine.or(default.engine),
            create_schema: p.create_schema.or(default.create_schema),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "async_insert": false,
            "attributes": {
                "resource": {
                    "representation": "json",
                    "storage": "lookup"
                },
                "scope": {
                    "representation": "json",
                },
                "log": {
                    "representation": "string_map",
                    "storage": "inline"
                }
            },
            "table_defaults": {
                "ttl_interval": "48 HOUR",
                "engine": {
                    "name": "ReplacingMergeTree"
                }
            },
            "tables": {
                "logs": {
                    "ttl": "12h"
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
        assert_eq!(config.password, "secret");
        assert!(!config.async_insert);

        // --- Attributes defaults & overrides ---
        assert_eq!(
            config.attributes.resource.representation,
            AttributeRepresentation::Json
        );
        assert_eq!(config.attributes.resource.storage, AttributeStorage::Lookup);

        // Overridden log attributes
        assert_eq!(
            config.attributes.log.representation,
            AttributeRepresentation::StringMap
        );
        assert_eq!(config.attributes.log.storage, AttributeStorage::Inline);

        // Defaulted attribute groups
        assert_eq!(
            config.attributes.scope.representation,
            AttributeRepresentation::Json
        );
        assert_eq!(config.attributes.scope.storage, AttributeStorage::Lookup);

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
        assert_eq!(config.tables.logs.ttl.as_deref(), Some("12h"));

        // Defaulted tables still present
        assert_eq!(config.tables.traces.name, "otel_traces");

        // Metrics table override
        assert_eq!(config.tables.metrics.gauge.name, "custom_metrics_gauge");

        // Default metrics tables
        assert_eq!(config.tables.metrics.sum.name, "otel_metrics_sum");
    }

    #[test]
    fn test_config_error_invalid_attribute_params() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "clickhouse",
            "password": "secret",
            "async_insert": false,
            "attributes": {
                "resource": {
                    // This is not a valid combination
                    "representation": "otap_array",
                    "storage": "inline"
                },
            }
        });
        let err = serde_json::from_value::<ConfigPatch>(json)
            .expect_err("config should fail to deserialize");

        let msg = err.to_string();
        assert!(
            msg.contains("OtapArray representation cannot be stored inline"),
            "unexpected error message: {msg}"
        );
    }
}
