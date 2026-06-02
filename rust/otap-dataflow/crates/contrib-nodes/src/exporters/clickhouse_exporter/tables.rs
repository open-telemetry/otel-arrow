//! ClickHouse schema initialization for the OTAP-based ClickHouse exporter.
//!
//! This module is responsible for translating the exporter [`Config`] into concrete ClickHouse
//! DDL (CREATE TABLE) for the OTAP (Arrow) payload types that are stored in ClickHouse.
//! Attributes are always stored inline on the signal tables (logs/spans) using either JSON or
//! a `Map(String, String)` representation.
//!
//! Key responsibilities:
//!
//! - Build ClickHouse `CREATE TABLE` statements via [`CHTableBuilder`], including engine settings,
//!   columns, ORDER BY / PARTITION BY / PRIMARY KEY, and TTL.
//! - Merge per-table overrides with global defaults (`finalize_table_config`) to determine the
//!   effective engine/TTL/schema-creation behavior.
//! - Produce a runtime lookup map of which destination table each signal payload should be inserted
//!   into (`build_payload_destination_table_map`).
//! - Create base tables according to representation rules (`init_table`, `build_table_sql`).
//!
//! The module also defines [`TableKind`] and [`TableEntry`] plus iterators over configured tables
//! (via [`TablesConfig::iter_tables`] and [`MetricsTableConfig::iter`]) to drive initialization.
use std::collections::HashMap;

use clickhouse_arrow::{ArrowClient, Qid};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::clickhouse_exporter::config::{
    AttributeRepresentation, Config, DefaultTableConfig, MetricsTableConfig, TableConfig,
    TableEngine, TablesConfig,
};
use crate::clickhouse_exporter::consts as ch_consts;
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::schema;

#[derive(Debug, Default, Clone)]
pub struct CHTableBuilder {
    pub database: String,
    pub table: String,
    pub engine: String,
    pub engine_params: Option<String>,
    pub order_by: Vec<String>,
    pub partition_by: Vec<String>,
    pub primary_key: Vec<String>,
    pub columns: Vec<schema::Column>,
    pub indexes: Vec<schema::Index>,
    pub settings: Vec<(String, String)>,
    pub ttl: Option<String>,
    pub timestamp_field: String,
}
impl CHTableBuilder {
    pub fn build(self) -> Result<String, ClickhouseExporterError> {
        let mut body_items = Vec::new();

        // Columns
        for column in &self.columns {
            body_items.push(column.render());
        }

        // Indexes (render after columns within the parenthesized list)
        for index in &self.indexes {
            body_items.push(index.render());
        }

        // ORDER BY
        let order_by_sql = if self.order_by.is_empty() {
            "ORDER BY tuple()".to_string()
        } else {
            format!("ORDER BY ({})", self.order_by.join(", "))
        };

        // PARTITION BY
        let partition_by_sql = if self.partition_by.is_empty() {
            "PARTITION BY tuple()".to_string()
        } else {
            format!("PARTITION BY ({})", self.partition_by.join(", "))
        };

        // PRIMARY KEY -- omit when empty (let ClickHouse default to ORDER BY)
        let primary_key_sql = if self.primary_key.is_empty() {
            None
        } else {
            Some(format!("PRIMARY KEY ({})", self.primary_key.join(", ")))
        };

        // SETTINGS
        let settings_sql = if self.settings.is_empty() {
            None
        } else {
            let pairs: Vec<String> = self
                .settings
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            Some(format!("SETTINGS {}", pairs.join(", ")))
        };

        // TTL
        let ttl_sql = self
            .ttl
            .as_ref()
            .map(|v| format!("TTL {} + INTERVAL {}", self.timestamp_field, v));

        let engine_param_sql = self.engine_params.unwrap_or_default();

        // Build final SQL with only non-empty clauses
        let mut clauses = Vec::new();
        clauses.push(format!(
            "CREATE TABLE IF NOT EXISTS {}.{} (\n    {}\n) ENGINE = {}{}",
            self.database,
            self.table,
            body_items.join(",\n    "),
            self.engine,
            engine_param_sql,
        ));
        clauses.push(partition_by_sql);
        if let Some(pk) = primary_key_sql {
            clauses.push(pk);
        }
        clauses.push(order_by_sql);
        if let Some(settings) = settings_sql {
            clauses.push(settings);
        }
        if let Some(ttl) = ttl_sql {
            clauses.push(ttl);
        }

        Ok(format!("{}\n;", clauses.join("\n")))
    }

    /// Validate the user-supplied inputs to the table construction for basic SQL validity/safety
    pub fn validate(self) -> Result<(), ClickhouseExporterError> {
        validate_identifier(&self.database, "database")?;
        validate_identifier(&self.table, "table")?;
        validate_identifier(&self.engine, "engine")?;
        if self.engine_params.is_some() {
            validate_identifier(&self.engine_params.unwrap_or_default(), "engine_params")?;
        }
        if self.ttl.is_some() {
            validate_identifier(&self.ttl.unwrap_or_default(), "ttl")?;
        }
        Ok(())
    }
}

/// Validate user supplied identifiers for basic sql validity.
pub fn validate_identifier(name: &str, section: &str) -> Result<(), ClickhouseExporterError> {
    if name.is_empty() {
        return Err(ClickhouseExporterError::TableCreationError {
            error: format!("Identifier cannot be empty: {}.", section),
        });
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ClickhouseExporterError::TableCreationError {
            error: format!("Invalid identifier: {}", name),
        });
    }
    Ok(())
}

struct FinalTableConfig {
    /// Logical name (users query this)
    pub name: String,
    /// TTL, e.g., "72h"
    pub ttl: Option<String>,
    /// Optional table engine override
    pub engine: TableEngine,
    /// Whether to create the schema automatically or not
    pub create_schema: bool,
}
/// Merge the default table configs with any more specific overrides to produce a final config.
fn finalize_table_config(t: &TableConfig, d: &DefaultTableConfig) -> FinalTableConfig {
    FinalTableConfig {
        name: t.name.clone(),
        ttl: t.ttl.clone(),
        engine: t.engine.clone().unwrap_or(d.engine.clone()),
        create_schema: t.create_schema.unwrap_or(d.create_schema),
    }
}

/// Create a map of signal payload types to the name of the table where the data will be stored.
/// Only signal payloads (Logs, Spans) get entries. Attribute payloads are consumed during
/// transformation and don't need table mappings.
pub(crate) fn build_payload_destination_table_map(
    config: &Config,
) -> HashMap<ArrowPayloadType, String> {
    let mut dst_tables = HashMap::new();
    let _ = dst_tables.insert(ArrowPayloadType::Logs, config.tables.logs.name.clone());
    let _ = dst_tables.insert(ArrowPayloadType::Spans, config.tables.traces.name.clone());
    dst_tables
}

/// Clone an inline attribute column definition, overriding its ColumnType based on the
/// configured attribute representation.
fn attribute_column(base_col: &schema::Column, repr: &AttributeRepresentation) -> schema::Column {
    let mut col = base_col.clone();
    col.ch_type = match repr {
        AttributeRepresentation::Json => schema::ColumnType::Json,
        AttributeRepresentation::StringMap => schema::ColumnType::MapLCStringString,
    };
    col
}

fn build_table_sql(
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<Option<String>, ClickhouseExporterError> {
    let final_config = finalize_table_config(entry.config, &config.table_defaults);
    // Check if we need to create the schema or return if not.
    if !final_config.create_schema {
        return Ok(None);
    }

    let mut builder = CHTableBuilder {
        database: config.database.clone(),
        table: final_config.name.clone(),
        engine: final_config.engine.name.clone(),
        engine_params: final_config.engine.params.clone(),
        ttl: final_config.ttl.clone(),
        timestamp_field: ch_consts::CH_TIMESTAMP.into(),
        order_by: vec![],
        partition_by: vec![],
        primary_key: vec![],
        columns: vec![],
        indexes: vec![],
        settings: vec![],
    };
    let maybe_sql = match entry.kind {
        TableKind::Logs => {
            // Column order matches Go OTel Collector ClickHouse exporter logs schema
            builder.columns.extend_from_slice(&[
                schema::TIMESTAMP_COLUMN.clone(),
                schema::TRACE_ID_COLUMN.clone(),
                schema::SPAN_ID_COLUMN.clone(),
                schema::TRACE_FLAGS_COLUMN.clone(),
                schema::SEVERITY_TEXT_COLUMN.clone(),
                schema::SEVERITY_NUMBER_COLUMN.clone(),
                schema::SERVICE_NAME_COLUMN.clone(),
                schema::BODY_COLUMN.clone(),
                schema::RESOURCE_SCHEMA_URL_COLUMN.clone(),
            ]);
            builder.columns.push(attribute_column(
                schema::INLINE_RESOURCE_ATTR_COLUMN,
                &config.attributes.resource.representation,
            ));
            builder
                .columns
                .push(schema::SCOPE_SCHEMA_URL_COLUMN.clone());
            builder.columns.extend_from_slice(&[
                schema::SCOPE_NAME_COLUMN.clone(),
                schema::SCOPE_VERSION_COLUMN.clone(),
            ]);
            builder.columns.push(attribute_column(
                schema::INLINE_SCOPE_ATTR_COLUMN,
                &config.attributes.scope.representation,
            ));
            builder.columns.push(attribute_column(
                schema::INLINE_LOG_ATTR_COLUMN,
                &config.attributes.log.representation,
            ));
            builder.columns.push(schema::EVENT_NAME_COLUMN.clone());

            // JSON mode: add companion *Keys columns after their attribute columns
            if config.attributes.resource.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::RESOURCE_ATTRIBUTES_KEYS_COLUMN.clone());
            }
            if config.attributes.scope.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::SCOPE_ATTRIBUTES_KEYS_COLUMN.clone());
            }
            if config.attributes.log.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::LOG_ATTRIBUTES_KEYS_COLUMN.clone());
            }

            // Indexes
            builder.indexes.push(schema::IDX_TRACE_ID);
            match config.attributes.resource.representation {
                AttributeRepresentation::StringMap => {
                    builder.indexes.push(schema::IDX_RES_ATTR_KEY);
                    builder.indexes.push(schema::IDX_RES_ATTR_VALUE);
                }
                AttributeRepresentation::Json => {
                    builder.indexes.push(schema::IDX_RES_ATTR_KEYS);
                }
            }
            match config.attributes.scope.representation {
                AttributeRepresentation::StringMap => {
                    builder.indexes.push(schema::IDX_SCOPE_ATTR_KEY);
                    builder.indexes.push(schema::IDX_SCOPE_ATTR_VALUE);
                }
                AttributeRepresentation::Json => {
                    builder.indexes.push(schema::IDX_SCOPE_ATTR_KEYS);
                }
            }
            match config.attributes.log.representation {
                AttributeRepresentation::StringMap => {
                    builder.indexes.push(schema::IDX_LOG_ATTR_KEY);
                    builder.indexes.push(schema::IDX_LOG_ATTR_VALUE);
                }
                AttributeRepresentation::Json => {
                    builder.indexes.push(schema::IDX_LOG_ATTR_KEYS);
                }
            }
            builder.indexes.push(schema::IDX_LOWER_BODY);

            // Table properties matching Go exporter
            builder.order_by.extend(vec![
                format!("toStartOfFiveMinutes({})", ch_consts::CH_TIMESTAMP),
                ch_consts::CH_SERVICE_NAME.into(),
                ch_consts::CH_TIMESTAMP.into(),
            ]);
            builder
                .partition_by
                .push(format!("toDate({})", ch_consts::CH_TIMESTAMP));
            // primary_key left empty -> omitted, defaults to ORDER BY
            builder.settings.extend(default_table_settings());

            builder.clone().validate()?;
            Some(builder.build()?)
        }
        TableKind::Traces => {
            // Column order matches Go OTel Collector ClickHouse exporter traces schema
            builder.columns.extend_from_slice(&[
                schema::TIMESTAMP_COLUMN.clone(),
                schema::TRACE_ID_COLUMN.clone(),
                schema::SPAN_ID_COLUMN.clone(),
                schema::PARENT_SPAN_ID_COLUMN.clone(),
                schema::TRACE_STATE_COLUMN.clone(),
                schema::SPAN_NAME_COLUMN.clone(),
                schema::SPAN_KIND_COLUMN.clone(),
                schema::SERVICE_NAME_COLUMN.clone(),
            ]);
            builder.columns.push(attribute_column(
                schema::INLINE_RESOURCE_ATTR_COLUMN,
                &config.attributes.resource.representation,
            ));
            builder.columns.extend_from_slice(&[
                schema::SCOPE_NAME_COLUMN.clone(),
                schema::SCOPE_VERSION_COLUMN.clone(),
            ]);
            // No ScopeAttributes for traces (matches Go exporter)
            builder.columns.push(attribute_column(
                schema::INLINE_SPAN_ATTR_COLUMN,
                &config.attributes.trace.representation,
            ));

            // JSON mode: add companion *Keys columns
            if config.attributes.resource.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::RESOURCE_ATTRIBUTES_KEYS_COLUMN.clone());
            }
            if config.attributes.trace.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::SPAN_ATTRIBUTES_KEYS_COLUMN.clone());
            }

            builder.columns.extend_from_slice(&[
                schema::DURATION_COLUMN.clone(),
                schema::STATUS_CODE_COLUMN.clone(),
                schema::STATUS_MESSAGE_COLUMN.clone(),
            ]);

            // Events Nested columns
            builder
                .columns
                .push(schema::EVENTS_TIMESTAMP_COLUMN.clone());
            builder.columns.push(schema::EVENTS_NAME_COLUMN.clone());
            if config.attributes.trace.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::EVENTS_ATTRIBUTES_JSON_COLUMN.clone());
            } else {
                builder
                    .columns
                    .push(schema::EVENTS_ATTRIBUTES_COLUMN.clone());
            }

            // Links Nested columns
            builder.columns.push(schema::LINKS_TRACE_ID_COLUMN.clone());
            builder.columns.push(schema::LINKS_SPAN_ID_COLUMN.clone());
            builder
                .columns
                .push(schema::LINKS_TRACE_STATE_COLUMN.clone());
            if config.attributes.trace.representation == AttributeRepresentation::Json {
                builder
                    .columns
                    .push(schema::LINKS_ATTRIBUTES_JSON_COLUMN.clone());
            } else {
                builder
                    .columns
                    .push(schema::LINKS_ATTRIBUTES_COLUMN.clone());
            }

            // Indexes
            builder.indexes.push(schema::IDX_TRACE_ID);
            match config.attributes.resource.representation {
                AttributeRepresentation::StringMap => {
                    builder.indexes.push(schema::IDX_RES_ATTR_KEY);
                    builder.indexes.push(schema::IDX_RES_ATTR_VALUE);
                }
                AttributeRepresentation::Json => {
                    builder.indexes.push(schema::IDX_RES_ATTR_KEYS);
                }
            }
            match config.attributes.trace.representation {
                AttributeRepresentation::StringMap => {
                    builder.indexes.push(schema::IDX_SPAN_ATTR_KEY);
                    builder.indexes.push(schema::IDX_SPAN_ATTR_VALUE);
                }
                AttributeRepresentation::Json => {
                    builder.indexes.push(schema::IDX_SPAN_ATTR_KEYS);
                }
            }
            builder.indexes.push(schema::IDX_DURATION);

            // Table properties matching Go exporter
            builder.order_by.extend(vec![
                ch_consts::CH_SERVICE_NAME.into(),
                ch_consts::CH_SPAN_NAME.into(),
                format!("toDateTime({})", ch_consts::CH_TIMESTAMP),
            ]);
            builder
                .partition_by
                .push(format!("toDate({})", ch_consts::CH_TIMESTAMP));
            // primary_key left empty -> omitted, defaults to ORDER BY
            builder.settings.extend(default_table_settings());

            builder.clone().validate()?;
            Some(builder.build()?)
        }
        // TODO: [support_new_signal] implement these
        _ => Some("SELECT 1;".into()),
    };
    Ok(maybe_sql)
}

pub async fn init_table(
    client: &ArrowClient,
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<(), ClickhouseExporterError> {
    let Some(sql) = build_table_sql(entry, config)? else {
        return Ok(());
    };

    client.execute(sql, Some(Qid::new())).await.map_err(|e| {
        ClickhouseExporterError::TableCreationError {
            error: format!("{e}"),
        }
    })?;
    Ok(())
}

/// Default SETTINGS matching the Go OTel Collector ClickHouse exporter.
fn default_table_settings() -> Vec<(String, String)> {
    vec![
        ("index_granularity".into(), "8192".into()),
        ("ttl_only_drop_parts".into(), "1".into()),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Logs,
    Traces,
    Gauge,
    Sum,
    Histogram,
}

#[derive(Debug)]
pub struct TableEntry<'a> {
    pub kind: TableKind,
    pub config: &'a TableConfig,
}

impl TableKind {
    pub fn is_signal_kind(&self) -> bool {
        matches!(
            self,
            Self::Logs | Self::Traces | Self::Gauge | Self::Sum | Self::Histogram
        )
    }
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            TableKind::Traces => "trace",
            TableKind::Logs => "log",
            _ => "unknown",
        }
    }
}

impl MetricsTableConfig {
    /// Helper function for iterrating the metric table configs.
    pub fn iter(&self) -> impl Iterator<Item = TableEntry<'_>> {
        [
            (TableKind::Gauge, &self.gauge),
            (TableKind::Sum, &self.sum),
            (TableKind::Histogram, &self.histogram),
        ]
        .into_iter()
        .map(|(kind, config)| TableEntry { kind, config })
    }
}

impl TablesConfig {
    /// Helper function for iterrating the table configs.
    pub fn iter_tables(&self) -> impl Iterator<Item = TableEntry<'_>> {
        // base tables
        let base = [
            (TableKind::Logs, &self.logs),
            (TableKind::Traces, &self.traces),
        ];

        // nested metric tables (MetricsTableConfig exposes its own iterator)
        let metrics = self.metrics.iter();

        base.into_iter()
            .map(|(kind, config)| TableEntry { kind, config })
            .chain(metrics)
    }
}

#[cfg(test)]
mod tests {
    use crate::clickhouse_exporter::config::ConfigPatch;

    use super::*;

    fn base_config(attributes: serde_json::Value) -> Config {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "foo",
            "password": "bar",
            "attributes": attributes,
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        Config::from_patch(patch)
    }

    fn get_table_config(kind: TableKind, config: &Config) -> &TableConfig {
        match kind {
            TableKind::Logs => &config.tables.logs,
            TableKind::Traces => &config.tables.traces,
            TableKind::Gauge => &config.tables.metrics.gauge,
            TableKind::Sum => &config.tables.metrics.sum,
            TableKind::Histogram => &config.tables.metrics.histogram,
        }
    }

    #[test]
    fn snapshot_signal_tables() {
        let cases = [
            (
                "map_attrs",
                serde_json::json!({
                    "resource": { "representation": "string_map" },
                    "scope":    { "representation": "string_map" },
                    "log":      { "representation": "string_map" },
                    "trace":    { "representation": "string_map" },
                }),
            ),
            (
                "json_attrs",
                serde_json::json!({
                    "resource": { "representation": "json" },
                    "scope":    { "representation": "json" },
                    "log":      { "representation": "json" },
                    "trace":    { "representation": "json" },
                }),
            ),
        ];

        let signal_types = [TableKind::Logs, TableKind::Traces];

        for signal_type in signal_types {
            for (name, attributes) in cases.clone() {
                let config = base_config(attributes);

                let entry = TableEntry {
                    kind: signal_type,
                    config: get_table_config(signal_type, &config),
                };

                let sql = build_table_sql(&entry, &config)
                    .expect("sql generation failed")
                    .expect("signal table should always be created");

                insta::with_settings!({
                    prepend_module_to_snapshot => false,
                    snapshot_path => "table_snapshots"
                }, {
                    insta::assert_snapshot!(format!(
                        "{}_table_{}",
                        signal_type.as_str(),
                        name,
                    ),
                    sql);
                });
            }
        }
    }

    // --- Task 2.6: CHTableBuilder unit tests ---

    #[test]
    fn test_builder_with_indexes_and_settings() {
        let builder = CHTableBuilder {
            database: "test_db".into(),
            table: "test_table".into(),
            engine: "MergeTree".into(),
            engine_params: None,
            order_by: vec!["col1".into()],
            partition_by: vec!["toDate(col1)".into()],
            primary_key: vec![],
            columns: vec![schema::Column::new(
                "col1",
                schema::ColumnType::String,
                schema::ColumnOpts::ZSTD,
            )],
            indexes: vec![schema::Index::new(
                "idx_test",
                "col1",
                "bloom_filter(0.01)",
                1,
            )],
            settings: vec![
                ("index_granularity".into(), "8192".into()),
                ("ttl_only_drop_parts".into(), "1".into()),
            ],
            ttl: None,
            timestamp_field: "col1".into(),
        };
        let sql = builder.build().unwrap();
        assert!(sql.contains("INDEX idx_test col1 TYPE bloom_filter(0.01) GRANULARITY 1"));
        assert!(sql.contains("SETTINGS index_granularity=8192, ttl_only_drop_parts=1"));
        assert!(!sql.contains("PRIMARY KEY"));
    }

    #[test]
    fn test_builder_empty_order_by_uses_tuple() {
        let builder = CHTableBuilder {
            database: "db".into(),
            table: "tbl".into(),
            engine: "MergeTree".into(),
            order_by: vec![],
            ..Default::default()
        };
        let sql = builder.build().unwrap();
        assert!(sql.contains("ORDER BY tuple()"));
    }

    #[test]
    fn test_builder_empty_primary_key_omits_clause() {
        let builder = CHTableBuilder {
            database: "db".into(),
            table: "tbl".into(),
            engine: "MergeTree".into(),
            primary_key: vec![],
            ..Default::default()
        };
        let sql = builder.build().unwrap();
        assert!(
            !sql.contains("PRIMARY KEY"),
            "Empty primary_key should not produce a PRIMARY KEY clause"
        );
    }

    #[test]
    fn test_builder_with_primary_key() {
        let builder = CHTableBuilder {
            database: "db".into(),
            table: "tbl".into(),
            engine: "MergeTree".into(),
            primary_key: vec!["a".into(), "b".into()],
            ..Default::default()
        };
        let sql = builder.build().unwrap();
        assert!(sql.contains("PRIMARY KEY (a, b)"));
    }

    #[test]
    fn test_builder_with_ttl() {
        let builder = CHTableBuilder {
            database: "db".into(),
            table: "tbl".into(),
            engine: "MergeTree".into(),
            ttl: Some("72 HOUR".into()),
            timestamp_field: "Timestamp".into(),
            ..Default::default()
        };
        let sql = builder.build().unwrap();
        assert!(sql.contains("TTL Timestamp + INTERVAL 72 HOUR"));
    }

    #[test]
    fn test_builder_indexes_render_after_columns() {
        let builder = CHTableBuilder {
            database: "db".into(),
            table: "tbl".into(),
            engine: "MergeTree".into(),
            columns: vec![schema::Column::new(
                "col1",
                schema::ColumnType::String,
                schema::ColumnOpts::ZSTD,
            )],
            indexes: vec![schema::Index::new("idx_c1", "col1", "minmax", 1)],
            ..Default::default()
        };
        let sql = builder.build().unwrap();
        let col_pos = sql.find("`col1`").unwrap();
        let idx_pos = sql.find("INDEX idx_c1").unwrap();
        assert!(idx_pos > col_pos, "Index should appear after column in DDL");
    }

    // --- Payload destination table map tests ---

    #[test]
    fn test_payload_destination_table_map_default() {
        let config = base_config(serde_json::json!({}));
        let map = build_payload_destination_table_map(&config);
        assert_eq!(map.len(), 2);
        assert_eq!(map[&ArrowPayloadType::Logs], "otel_logs");
        assert_eq!(map[&ArrowPayloadType::Spans], "otel_traces");
    }

    // --- Metric stubs ---

    #[test]
    fn test_gauge_table_produces_stub() {
        let config = base_config(serde_json::json!({}));
        let entry = TableEntry {
            kind: TableKind::Gauge,
            config: get_table_config(TableKind::Gauge, &config),
        };
        let sql = build_table_sql(&entry, &config).unwrap().unwrap();
        assert_eq!(sql, "SELECT 1;");
    }

    // --- Schema creation disabled ---

    #[test]
    fn test_disabled_schema_creation_returns_none() {
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "foo",
            "password": "bar",
            "table_defaults": { "create_schema": false },
        });
        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config = Config::from_patch(patch);
        let entry = TableEntry {
            kind: TableKind::Logs,
            config: &config.tables.logs,
        };
        let result = build_table_sql(&entry, &config).unwrap();
        assert!(result.is_none());
    }

    // --- Validation tests ---

    #[test]
    fn test_validate_valid_identifiers() {
        let builder = CHTableBuilder {
            database: "otap".into(),
            table: "otel_logs".into(),
            engine: "MergeTree".into(),
            ..Default::default()
        };
        assert!(builder.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_database_fails() {
        let builder = CHTableBuilder {
            database: "".into(),
            table: "otel_logs".into(),
            engine: "MergeTree".into(),
            ..Default::default()
        };
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_validate_special_chars_fail() {
        let builder = CHTableBuilder {
            database: "otap".into(),
            table: "otel-logs".into(),
            engine: "MergeTree".into(),
            ..Default::default()
        };
        assert!(builder.validate().is_err());
    }
}
