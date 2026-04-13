//! ClickHouse schema initialization and view wiring for the OTAP-based ClickHouse exporter.
//!
//! This module is responsible for translating the exporter [`Config`] into concrete ClickHouse
//! DDL (CREATE TABLE / CREATE VIEW) for the OTAP (Arrow) payload types that are stored in
//! ClickHouse. It supports two broad storage layouts for attributes:
//!
//! - **Inline attributes**: attributes are stored directly on the signal tables (logs/spans) using
//!   either JSON or a `Map(String, String)` representation.
//! - **Lookup / normalized attributes**: attributes are stored in dedicated attribute tables and
//!   the signal tables store only IDs. In this case the module can create:
//!   - **Physical “raw” tables** (e.g. `{name}_raw`) that receive inserts, and
//!   - **Logical views** (named after the configured table) that join signal rows to the relevant
//!     attribute tables to present a query-friendly schema.
//!
//! Key responsibilities:
//!
//! - Build ClickHouse `CREATE TABLE` statements via [`CHTableBuilder`], including engine settings,
//!   columns, ORDER BY / PARTITION BY / PRIMARY KEY, and TTL.
//! - Merge per-table overrides with global defaults (`finalize_table_config`) to determine the
//!   effective engine/TTL/schema-creation behavior.
//! - Decide, based on attribute storage/representation, whether a payload writes to a logical table
//!   name or to a physical “raw” table (`is_inline`, `is_dst_table_raw`, `get_physical_table_name`).
//! - Produce runtime lookup maps used by the ingestion pipeline:
//!   - which payloads require an ID field (`build_payloads_with_id_map`), and
//!   - which destination table each payload should be inserted into
//!     (`build_payload_destination_table_map`).
//! - Create auxiliary ClickHouse **views** when needed:
//!   - Attribute views for `OtapArray` attribute tables that aggregate key/value rows into a map
//!     (`init_attribute_views` / `build_attribute_view`).
//!   - Signal views (logs/traces) that LEFT JOIN resource/scope/signal attribute tables when any
//!     attributes are normalized (`init_signal_views` / `build_signal_view_sql`).
//! - Create base tables and attribute tables according to storage/representation rules
//!   (`init_table`, `build_table_sql`, `build_attribute_table`, `push_inline_or_lookup`).
//!
//! The module also defines [`TableKind`] and [`TableEntry`] plus iterators over configured tables
//! (via [`TablesConfig::iter_tables`] and [`MetricsTableConfig::iter`]) to drive initialization.
use std::collections::HashMap;

use clickhouse_arrow::{ArrowClient, Qid};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts::PARENT_ID;

use crate::clickhouse_exporter::config::{
    AttributeConfig, AttributeRepresentation, AttributeStorage, Config, DefaultTableConfig,
    MetricsTableConfig, TableConfig, TableEngine, TablesConfig,
};
use crate::clickhouse_exporter::error::ClickhouseExporterError;
use crate::clickhouse_exporter::schema;
use crate::clickhouse_exporter::{TABLE_BACKED_ARROW_PAYLOAD_TYPES, consts as ch_consts};

const DEFAULT_PHYSICAL_TABLE_SUFFIX: &str = "raw";

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
    pub ttl: Option<String>,
    pub timestamp_field: String,
}
impl CHTableBuilder {
    pub fn build(self) -> Result<String, ClickhouseExporterError> {
        let mut columns_sql = Vec::new();

        // Columns
        for column in self.columns {
            columns_sql.push(column.render());
        }

        // ORDER BY
        let order_by_sql = if self.order_by.is_empty() {
            "ORDER BY tuple()".into()
        } else {
            format!("ORDER BY ({})", self.order_by.join(", "))
        };

        // PARTITION BY
        let parition_by_sql = if self.partition_by.is_empty() {
            "PARTITION BY tuple()".into()
        } else {
            format!("PARTITION BY ({})", self.partition_by.join(", "))
        };

        // PRIMARY KEY
        let primary_key_sql = if self.primary_key.is_empty() {
            "PRIMARY KEY tuple()".into()
        } else {
            format!("PRIMARY KEY ({})", self.primary_key.join(", "))
        };

        // TTL
        let ttl_sql = self
            .ttl
            .as_ref()
            .map(|v| format!("TTL {} + INTERVAL {}", self.timestamp_field, v))
            .unwrap_or_default();

        let engine_param_sql = self.engine_params.unwrap_or_default();

        // Final SQL
        Ok(format!(
            "CREATE TABLE IF NOT EXISTS {}.{} (\n    {}\n) ENGINE = {}{}\n{}\n{}\n{}\n{};",
            self.database,
            self.table,
            columns_sql.join(",\n    "),
            self.engine,
            engine_param_sql,
            parition_by_sql,
            primary_key_sql,
            order_by_sql,
            ttl_sql,
        ))
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
    /// Optional physical table name. If None and attributes are lookup, defaults to "{name}_normalized"
    pub physical_name: Option<String>,
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
        physical_name: t.physical_name.clone().or({
            // fallback: if attributes are lookup/deduped,
            // caller can insert name-based default here if desired
            None
        }),
        ttl: t.ttl.clone(),
        engine: t.engine.clone().unwrap_or(d.engine.clone()),
        create_schema: t.create_schema.unwrap_or(d.create_schema),
    }
}

/// Get the runtime attribute configuration for a given table type.
fn get_attribute_table_config(kind: &TableKind, config: &Config) -> Option<AttributeConfig> {
    match kind {
        TableKind::LogAttributes => Some(config.attributes.log.clone()),
        TableKind::MetricAttributes => Some(config.attributes.metric.clone()),
        TableKind::ResourceAttributes => Some(config.attributes.resource.clone()),
        TableKind::ScopeAttributes => Some(config.attributes.scope.clone()),
        TableKind::TraceAttributes => Some(config.attributes.trace.clone()),
        _ => None,
    }
}

/// Determine if the given config includes any json columns.
pub fn has_json_column(config: &Config) -> bool {
    config.attributes.log.representation == AttributeRepresentation::Json
        || config.attributes.scope.representation == AttributeRepresentation::Json
        || config.attributes.resource.representation == AttributeRepresentation::Json
        || config.attributes.metric.representation == AttributeRepresentation::Json
        || config.attributes.trace.representation == AttributeRepresentation::Json
}

/// Determine if the given table kind includes 1 or more lookup tables based on configs.
fn has_lookup_table(config: &Config, kind: &TableKind) -> bool {
    let common_resource_lookup = config.attributes.resource.storage != AttributeStorage::Inline
        || config.attributes.scope.storage != AttributeStorage::Inline;
    match kind {
        TableKind::Gauge | TableKind::Histogram | TableKind::Sum => {
            common_resource_lookup || config.attributes.metric.storage != AttributeStorage::Inline
        }
        TableKind::Logs => {
            common_resource_lookup || config.attributes.log.storage != AttributeStorage::Inline
        }
        TableKind::Traces => {
            common_resource_lookup || config.attributes.trace.storage != AttributeStorage::Inline
        }
        // Otherwise it's an attribute table
        _ => false,
    }
}

/// Get the name of the "raw" table for tables with attributes that aren't inlined
fn get_physical_table_name(tc: &FinalTableConfig) -> String {
    match &tc.physical_name {
        Some(pn) => pn.to_string(),
        None => format!("{}_{}", tc.name, DEFAULT_PHYSICAL_TABLE_SUFFIX),
    }
}

fn table_section(config: &Config, pt: ArrowPayloadType) -> &TableConfig {
    match pt {
        ArrowPayloadType::Spans => &config.tables.traces,
        ArrowPayloadType::SpanAttrs => &config.tables.trace_attributes,
        ArrowPayloadType::Logs => &config.tables.logs,
        ArrowPayloadType::LogAttrs => &config.tables.log_attributes,
        ArrowPayloadType::ResourceAttrs => &config.tables.resource_attributes,
        ArrowPayloadType::ScopeAttrs => &config.tables.scope_attributes,
        _ => unreachable!("unsupported ArrowPayloadType"),
    }
}

fn is_inline(config: &Config, pt: ArrowPayloadType) -> bool {
    match pt {
        ArrowPayloadType::Logs => !has_lookup_table(config, &TableKind::Logs),
        ArrowPayloadType::LogAttrs => config.attributes.log.storage == AttributeStorage::Inline,
        ArrowPayloadType::Spans => !has_lookup_table(config, &TableKind::Traces),
        ArrowPayloadType::SpanAttrs => config.attributes.trace.storage == AttributeStorage::Inline,
        ArrowPayloadType::ResourceAttrs => {
            config.attributes.resource.storage == AttributeStorage::Inline
        }
        ArrowPayloadType::ScopeAttrs => config.attributes.scope.storage == AttributeStorage::Inline,
        _ => false,
    }
}

fn is_dst_table_raw(config: &Config, pt: ArrowPayloadType) -> bool {
    match pt {
        // If a signal table has all inline attributes, dst is not raw
        ArrowPayloadType::Logs => has_lookup_table(config, &TableKind::Logs),
        // If an attribute table is not on otap_array
        ArrowPayloadType::LogAttrs => {
            config.attributes.log.representation == AttributeRepresentation::OtapArray
        }
        ArrowPayloadType::Spans => has_lookup_table(config, &TableKind::Traces),
        ArrowPayloadType::SpanAttrs => {
            config.attributes.trace.representation == AttributeRepresentation::OtapArray
        }
        ArrowPayloadType::ResourceAttrs => {
            config.attributes.resource.representation == AttributeRepresentation::OtapArray
        }
        ArrowPayloadType::ScopeAttrs => {
            config.attributes.scope.representation == AttributeRepresentation::OtapArray
        }
        _ => false,
    }
}

/// Create a map of payload types that will include an ID field of some sort.
pub(crate) fn build_payloads_with_id_map(config: &Config) -> HashMap<ArrowPayloadType, bool> {
    let mut payload_with_id_map = HashMap::new();
    for pt in TABLE_BACKED_ARROW_PAYLOAD_TYPES {
        // Check if the attributes are inline or if all of the signal's attribute types are inline.
        if !is_inline(config, *pt) {
            let _ = payload_with_id_map.insert(*pt, true);
        }
    }
    payload_with_id_map
}

/// Create a map of payload types to the name of the table where the data will be stored.
pub(crate) fn build_payload_destination_table_map(
    config: &Config,
) -> HashMap<ArrowPayloadType, String> {
    let mut dst_tables = HashMap::new();
    for pt in TABLE_BACKED_ARROW_PAYLOAD_TYPES {
        let fc = finalize_table_config(table_section(config, *pt), &config.table_defaults);

        let value = if is_dst_table_raw(config, *pt) {
            get_physical_table_name(&fc)
        } else {
            fc.name.clone()
        };

        let _ = dst_tables.insert(*pt, value);
    }
    dst_tables
}

/// Initialize views for attributes represented as otap_arrays
pub(crate) async fn init_attribute_views(
    client: &ArrowClient,
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<(), ClickhouseExporterError> {
    let view_sql = build_attribute_view(entry, config)?;
    if let Some(sql) = view_sql {
        client.execute(sql, Some(Qid::new())).await.map_err(|e| {
            ClickhouseExporterError::TableCreationError {
                error: format!("{e}"),
            }
        })?;
    }
    Ok(())
}

fn build_attribute_view(
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<Option<String>, ClickhouseExporterError> {
    let final_config = finalize_table_config(entry.config, &config.table_defaults);

    // Check if we need to create a view
    if !entry.kind.is_attr_kind() || // if it's not an attribute kind.
        !final_config.create_schema || // If we're not creating schemas
        get_attribute_table_config(&entry.kind, config) // If it's not an otap array we don't need a view.
            .is_none_or(|ac| ac.representation != AttributeRepresentation::OtapArray)
    {
        return Ok(None);
    }

    let physical_name = get_physical_table_name(&final_config);
    Ok(Some(format!(
        "
CREATE VIEW IF NOT EXISTS {db}.{view} AS
SELECT
    part_id,
    parent_id,
    mapFromArrays(
        groupArray(key),
        groupArray(
            CASE type
                WHEN 0 THEN ''                                -- Empty
                WHEN 1 THEN COALESCE(str, '')                 -- Str
                WHEN 2 THEN COALESCE(toString(int), '')       -- Int
                WHEN 3 THEN COALESCE(toString(double), '')    -- Double
                WHEN 4 THEN COALESCE(toString(bool), '')      -- Bool
                WHEN 5 THEN COALESCE(ser, '')                 -- Map - serialized JSON
                WHEN 6 THEN COALESCE(ser, '')                 -- Slice - serialized JSON/array
                WHEN 7 THEN COALESCE(base64Encode(bytes), '') -- Bytes - safe encoding
                ELSE ''                                       -- Empty or unknown - empty string
            END
        )
    ) AS attributes
FROM {db}.{physical}
GROUP BY part_id, parent_id;",
        db = config.database.clone(),
        view = final_config.name,
        physical = physical_name,
    )))
}

/// Initialize views for the main signal tables if attributes aren't all inlined.
pub(crate) async fn init_signal_views(
    client: &ArrowClient,
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<(), ClickhouseExporterError> {
    let view_sql = build_signal_view_sql(entry, config)?;
    if let Some(sql) = view_sql {
        client.execute(sql, Some(Qid::new())).await.map_err(|e| {
            ClickhouseExporterError::TableCreationError {
                error: format!("{e}"),
            }
        })?;
    }
    Ok(())
}

fn build_signal_view_sql(
    entry: &TableEntry<'_>,
    config: &Config,
) -> Result<Option<String>, ClickhouseExporterError> {
    let final_config = finalize_table_config(entry.config, &config.table_defaults);

    // Check if we need to create a view
    if !entry.kind.is_signal_kind() || // if it's not an signal kind.
        !final_config.create_schema || // If we're not creating schemas
        !has_lookup_table(config, &entry.kind)
    // If all of the attributes are 'inline'
    {
        return Ok(None);
    }

    // Build the final clickstack / otel clickhouse exporter views based on the table type
    let mut select_fields = Vec::new();
    let mut joins = Vec::new();
    let from = format!(
        "FROM {}.{} AS signal",
        config.database.clone(),
        get_physical_table_name(&final_config)
    );
    match entry.kind {
        // The final log view contains the same fields as this (except TraceFlags which isn't in otap):
        // https://clickhouse.com/docs/use-cases/observability/clickstack/ingesting-data/schemas#logs
        TableKind::Logs => {
            select_fields.extend(vec![
                "signal.Timestamp as Timestamp",
                "signal.TimestampTime",
                "signal.ResourceSchemaUrl",
            ]);
            match config.attributes.resource.storage {
                AttributeStorage::Inline => select_fields.push("signal.ResourceAttributes"),
                _ => {
                    // We create the log view from the attribute table name (which is either a physical table or view)
                    let resource_attr_table = &config.tables.resource_attributes.name;
                    select_fields.push("ra.attributes as ResourceAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS ra
                            ON signal.part_id     = ra.part_id
                            AND signal.resource_id = ra.parent_id
                        ",
                        config.database.clone(),
                        resource_attr_table
                    ));
                }
            }
            select_fields.extend(vec![
                "signal.ScopeSchemaUrl",
                "signal.ScopeName",
                "signal.ScopeVersion",
            ]);
            match config.attributes.scope.storage {
                AttributeStorage::Inline => select_fields.push("signal.ScopeAttributes"),
                _ => {
                    // We create the log view from the attribute table name (which is either a physical table or view)
                    let scope_attr_table = &config.tables.scope_attributes.name;
                    select_fields.push("sa.attributes as ScopeAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS sa
                            ON signal.part_id     = sa.part_id
                            AND signal.scope_id = sa.parent_id
                        ",
                        config.database.clone(),
                        scope_attr_table
                    ));
                }
            }
            match config.attributes.log.storage {
                AttributeStorage::Inline => select_fields.push("signal.LogAttributes"),
                _ => {
                    // We create the log view from the attribute table name (which is either a physical table or view)
                    let log_attr_table = &config.tables.log_attributes.name;
                    select_fields.push("la.attributes as LogAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS la
                            ON signal.part_id     = la.part_id
                            AND signal.id = la.parent_id
                        ",
                        config.database.clone(),
                        log_attr_table
                    ));
                }
            }
            select_fields.extend(vec![
                "signal.TraceId",
                "signal.SpanId",
                "signal.SeverityText",
                "signal.SeverityNumber",
                "signal.ServiceName",
                "signal.Body",
                "signal.EventName",
            ]);
        }
        // The final trace view contains the same fields as this:
        // https://clickhouse.com/docs/use-cases/observability/clickstack/ingesting-data/schemas#traces
        TableKind::Traces => {
            select_fields.extend(vec![
                "signal.Timestamp as Timestamp",
                "signal.TraceId",
                "signal.SpanId",
                "signal.ParentSpanId",
                "signal.TraceState",
                "signal.SpanName",
                "signal.SpanKind",
                "signal.ServiceName",
            ]);
            match config.attributes.resource.storage {
                AttributeStorage::Inline => select_fields.push("signal.ResourceAttributes"),
                _ => {
                    // We create the trace view from the attribute table name (which is either a physical table or view)
                    let resource_attr_table = &config.tables.resource_attributes.name;
                    select_fields.push("ra.attributes as ResourceAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS ra
                            ON signal.part_id     = ra.part_id
                            AND signal.resource_id = ra.parent_id
                        ",
                        config.database.clone(),
                        resource_attr_table
                    ));
                }
            }
            select_fields.extend(vec!["signal.ScopeName", "signal.ScopeVersion"]);
            match config.attributes.scope.storage {
                AttributeStorage::Inline => select_fields.push("signal.ScopeAttributes"),
                _ => {
                    // We create the log view from the attribute table name (which is either a physical table or view)
                    let scope_attr_table = &config.tables.scope_attributes.name;
                    select_fields.push("sa.attributes as ScopeAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS sa
                            ON signal.part_id     = sa.part_id
                            AND signal.scope_id = sa.parent_id
                        ",
                        config.database.clone(),
                        scope_attr_table
                    ));
                }
            }
            match config.attributes.trace.storage {
                AttributeStorage::Inline => select_fields.push("signal.SpanAttributes"),
                _ => {
                    // We create the log view from the attribute table name (which is either a physical table or view)
                    let trace_attr_table = &config.tables.trace_attributes.name;
                    select_fields.push("ta.attributes as SpanAttributes");
                    joins.push(format!(
                        "
                        LEFT JOIN {}.{} AS ta
                            ON signal.part_id     = ta.part_id
                            AND signal.id = ta.parent_id
                        ",
                        config.database.clone(),
                        trace_attr_table
                    ));
                }
            }
            select_fields.extend(vec![
                "signal.Duration",
                "signal.StatusCode",
                "signal.StatusMessage",
                "signal.`Events.Timestamp`",
                "signal.`Events.Name`",
                "signal.`Events.Attributes`",
                "signal.`Links.TraceId`",
                "signal.`Links.SpanId`",
                "signal.`Links.TraceState`",
                "signal.`Links.Attributes`",
            ]);
        }
        // TODO: [support_new_signal] add support here
        _ => return Ok(None),
    }
    // Build the final sql statement
    Ok(Some(
        format!(
            r#"
            CREATE VIEW IF NOT EXISTS {}.{} AS
            SELECT
                {}
            {}
            {}
        "#,
            config.database.clone(),
            final_config.name,
            select_fields.join(",\n    "),
            from,
            joins.join("\n")
        )
        .replace("            ", ""),
    ))
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
    };
    let maybe_sql = match entry.kind {
        TableKind::Logs => {
            // If we're not inlining 1 or more of the attribute types, we need to insert the global IDs.
            if has_lookup_table(config, &TableKind::Logs) {
                builder
                    .columns
                    .extend_from_slice(schema::SIGNAL_GLOBAL_ID_COLUMNS);
                // If we have a lookup attribute table, the main signal tables are views.
                builder.table = match final_config.physical_name {
                    Some(pn) => pn,
                    None => format!("{}_{}", final_config.name, DEFAULT_PHYSICAL_TABLE_SUFFIX),
                }
            }
            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_RESOURCE_ATTR_COLUMN,
                Some(schema::LOOKUP_RESOURCE_ATTR_COLUMN),
                &config.attributes.resource.storage,
                &config.attributes.resource.representation,
            );
            builder
                .columns
                .push(schema::RESOURCE_SCHEMA_URL_COLUMN.clone());
            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_SCOPE_ATTR_COLUMN,
                Some(schema::LOOKUP_SCOPE_ATTR_COLUMN),
                &config.attributes.scope.storage,
                &config.attributes.scope.representation,
            );
            builder.columns.extend_from_slice(&[
                schema::SCOPE_SCHEMA_URL_COLUMN.clone(),
                schema::SCOPE_NAME_COLUMN.clone(),
                schema::SCOPE_VERSION_COLUMN.clone(),
            ]);
            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_LOG_ATTR_COLUMN,
                // The lookup column steps for the main 'signal attributes' happen
                // at the top where we determine if any lookups exist.
                None,
                &config.attributes.log.storage,
                &config.attributes.log.representation,
            );
            builder.columns.extend_from_slice(&[
                schema::TIMESTAMP_COLUMN.clone(),
                schema::TIMESTAMP_TIME_COLUMN.clone(),
                schema::TRACE_ID_COLUMN.clone(),
                schema::SPAN_ID_COLUMN.clone(),
                schema::SEVERITY_TEXT_COLUMN.clone(),
                schema::SEVERITY_NUMBER_COLUMN.clone(),
                schema::SERVICE_NAME_COLUMN.clone(),
                schema::BODY_COLUMN.clone(),
                schema::EVENT_NAME_COLUMN.clone(),
            ]);

            builder.order_by.extend(vec![
                ch_consts::CH_SERVICE_NAME.into(),
                ch_consts::CH_TIMESTAMP_TIME.into(),
                ch_consts::CH_TIMESTAMP.into(),
            ]);
            builder.partition_by.push(logs_partition_expr());
            builder.primary_key.extend(vec![
                ch_consts::CH_SERVICE_NAME.into(),
                ch_consts::CH_TIMESTAMP_TIME.into(),
            ]);
            builder.clone().validate()?;
            Some(builder.build()?)
        }
        TableKind::Traces => {
            // If we're not inlining 1 or more of the attribute types, we need to insert the global IDs.
            if has_lookup_table(config, &TableKind::Traces) {
                builder
                    .columns
                    .extend_from_slice(schema::SIGNAL_GLOBAL_ID_COLUMNS);
                // If we have a lookup attribute table, the main signal tables are views.
                builder.table = match final_config.physical_name {
                    Some(pn) => pn,
                    None => format!("{}_{}", final_config.name, DEFAULT_PHYSICAL_TABLE_SUFFIX),
                }
            }
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

            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_RESOURCE_ATTR_COLUMN,
                Some(schema::LOOKUP_RESOURCE_ATTR_COLUMN),
                &config.attributes.resource.storage,
                &config.attributes.resource.representation,
            );
            builder.columns.extend_from_slice(&[
                schema::SCOPE_NAME_COLUMN.clone(),
                schema::SCOPE_VERSION_COLUMN.clone(),
            ]);
            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_SCOPE_ATTR_COLUMN,
                Some(schema::LOOKUP_SCOPE_ATTR_COLUMN),
                &config.attributes.scope.storage,
                &config.attributes.scope.representation,
            );
            push_inline_or_lookup(
                &mut builder.columns,
                schema::INLINE_SPAN_ATTR_COLUMN,
                // The lookup column steps for the main 'signal attributes' happen
                // at the top where we determine if any lookups exist.
                None,
                &config.attributes.trace.storage,
                &config.attributes.trace.representation,
            );
            builder.columns.extend_from_slice(&[
                schema::DURATION_COLUMN.clone(),
                schema::STATUS_CODE_COLUMN.clone(),
                schema::STATUS_MESSAGE_COLUMN.clone(),
                schema::EVENTS_TIMESTAMP_COLUMN.clone(),
                schema::EVENTS_NAME_COLUMN.clone(),
                schema::EVENTS_ATTRIBUTES_COLUMN.clone(),
                schema::LINKS_TRACE_ID_COLUMN.clone(),
                schema::LINKS_SPAN_ID_COLUMN.clone(),
                schema::LINKS_TRACE_STATE_COLUMN.clone(),
                schema::LINKS_ATTRIBUTES_COLUMN.clone(),
            ]);

            builder.order_by.extend(vec![
                ch_consts::CH_SERVICE_NAME.into(),
                ch_consts::CH_SPAN_NAME.into(),
                format!("toDateTime({})", ch_consts::CH_TIMESTAMP),
            ]);
            builder
                .partition_by
                .push(format!("toDate({})", ch_consts::CH_TIMESTAMP));
            builder.clone().validate()?;
            Some(builder.build()?)
        }
        TableKind::ResourceAttributes => build_attribute_table(
            builder,
            &config.attributes.resource.storage,
            &config.attributes.resource.representation,
            &final_config,
        )?,
        TableKind::ScopeAttributes => build_attribute_table(
            builder,
            &config.attributes.scope.storage,
            &config.attributes.scope.representation,
            &final_config,
        )?,
        TableKind::LogAttributes => build_attribute_table(
            builder,
            &config.attributes.log.storage,
            &config.attributes.log.representation,
            &final_config,
        )?,
        TableKind::TraceAttributes => build_attribute_table(
            builder,
            &config.attributes.trace.storage,
            &config.attributes.trace.representation,
            &final_config,
        )?,
        // TODO: [support_new_signal] implement these
        _ => Some("SELECT 1;".into()), // TableKind::Traces => initialize_traces(config)?,
                                       // TableKind::Gauge => init_metric("gauge", config)?,
                                       // TableKind::Sum => init_metric("sum", config)?,
                                       // TableKind::Histogram => init_histogram(config)?,
                                       // TableKind::MetricAttributes   => init_attr("metric", config)?,
                                       // TableKind::TraceAttributes    => init_attr("trace", config)?,
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

fn build_attribute_table(
    mut builder: CHTableBuilder,
    storage: &AttributeStorage,
    representation: &AttributeRepresentation,
    final_config: &FinalTableConfig,
) -> Result<Option<String>, ClickhouseExporterError> {
    if storage == &AttributeStorage::Inline {
        return Ok(None);
    }

    builder.timestamp_field = ch_consts::INSERT_TIME.into();

    if representation == &AttributeRepresentation::OtapArray {
        builder.table = get_physical_table_name(final_config);
    }

    match representation {
        AttributeRepresentation::Json => {
            builder
                .columns
                .extend_from_slice(schema::COMMON_FLATTENED_SCHEMA_COLUMNS);
            let mut col = schema::FLATTENED_ATTRIBUTE_COLUMN.clone();
            col.ch_type = schema::ColumnType::Json;
            builder.columns.push(col);
        }
        AttributeRepresentation::StringMap => {
            builder
                .columns
                .extend_from_slice(schema::COMMON_FLATTENED_SCHEMA_COLUMNS);
            let mut col = schema::FLATTENED_ATTRIBUTE_COLUMN.clone();
            col.ch_type = schema::ColumnType::MapLCStringString;
            builder.columns.push(col);
        }
        AttributeRepresentation::OtapArray => {
            builder
                .columns
                .extend_from_slice(schema::ARROW_ATTRIBUTE_SCHEMA_COLUMNS);
        }
    }

    builder.order_by.extend([
        ch_consts::PART_ID.into(),
        PARENT_ID.into(),
        ch_consts::INSERT_TIME.into(),
    ]);

    builder
        .partition_by
        .push(format!("toDate({})", ch_consts::INSERT_TIME));

    builder
        .primary_key
        .extend([ch_consts::PART_ID.into(), PARENT_ID.into()]);

    builder.clone().validate()?;
    Ok(Some(builder.build()?))
}

fn push_inline_or_lookup(
    columns: &mut Vec<schema::Column>,
    inline_col: &schema::Column,
    lookup_col: Option<&schema::Column>,
    storage: &AttributeStorage,
    representation: &AttributeRepresentation,
) {
    match (storage, representation) {
        (AttributeStorage::Inline, AttributeRepresentation::Json) => {
            let mut c = inline_col.clone();
            c.ch_type = schema::ColumnType::Json;
            columns.push(c);
        }
        (AttributeStorage::Inline, AttributeRepresentation::StringMap) => {
            let mut c = inline_col.clone();
            c.ch_type = schema::ColumnType::MapLCStringString;
            columns.push(c);
        }
        _ => {
            if let Some(lc) = lookup_col {
                columns.push(lc.clone())
            }
        }
    }
}

fn logs_partition_expr() -> String {
    format!("toDateTime(toUnixTimestamp({}))", ch_consts::CH_TIMESTAMP)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Logs,
    Traces,
    Gauge,
    Sum,
    Histogram,
    ResourceAttributes,
    ScopeAttributes,
    LogAttributes,
    MetricAttributes,
    TraceAttributes,
}

#[derive(Debug)]
pub struct TableEntry<'a> {
    pub kind: TableKind,
    pub config: &'a TableConfig,
}

impl TableKind {
    pub fn is_attr_kind(&self) -> bool {
        matches!(
            self,
            Self::ResourceAttributes
                | Self::ScopeAttributes
                | Self::LogAttributes
                | Self::MetricAttributes
                | Self::TraceAttributes
        )
    }
    pub fn is_signal_kind(&self) -> bool {
        matches!(
            self,
            Self::Logs | Self::Traces | Self::Gauge | Self::Sum | Self::Histogram
        )
    }
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            TableKind::ResourceAttributes => "resource_attributes",
            TableKind::ScopeAttributes => "scope_attributes",
            TableKind::LogAttributes => "log_attributes",
            TableKind::TraceAttributes => "trace_attributes",
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

        // optional attributes
        let attrs = [
            (TableKind::ResourceAttributes, &self.resource_attributes),
            (TableKind::ScopeAttributes, &self.scope_attributes),
            (TableKind::LogAttributes, &self.log_attributes),
            (TableKind::MetricAttributes, &self.metric_attributes),
            (TableKind::TraceAttributes, &self.trace_attributes),
        ];

        // nested metric tables (MetricsTableConfig exposes its own iterator)
        let metrics = self.metrics.iter();

        base.into_iter()
            .map(|(kind, config)| TableEntry { kind, config })
            .chain(attrs.map(|(kind, config)| TableEntry { kind, config }))
            .chain(metrics)
    }
}

#[cfg(test)]
mod tests {
    use crate::clickhouse_exporter::config::ConfigPatch;

    use super::*;

    /// Assert that all expected fragments appear in the SQL string.
    fn assert_sql_contains_all(sql: &str, fragments: &[&str]) {
        for fragment in fragments {
            assert!(
                sql.contains(fragment),
                "SQL did not contain expected fragment:\n\
                --- fragment ---\n{}\n\
                --- sql ---\n{}",
                fragment,
                sql
            );
        }
    }
    /// Assert that a fragment does *not* appear in the SQL string.
    fn assert_sql_not_contains(sql: &str, fragment: &str) {
        assert!(
            !sql.contains(fragment),
            "SQL unexpectedly contained fragment:\n\
            --- fragment ---\n{}\n\
            --- sql ---\n{}",
            fragment,
            sql
        );
    }
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

    fn build_attributes(
        key: &str,
        representation: &str,
        storage: Option<&str>,
    ) -> serde_json::Value {
        let mut attr = serde_json::json!({
            "representation": representation
        });

        if let Some(storage) = storage {
            attr["storage"] = serde_json::json!(storage);
        }

        serde_json::json!({
            key: attr
        })
    }

    fn get_table_config(kind: TableKind, config: &Config) -> &TableConfig {
        match kind {
            TableKind::ResourceAttributes => &config.tables.resource_attributes,
            TableKind::ScopeAttributes => &config.tables.scope_attributes,
            TableKind::LogAttributes => &config.tables.log_attributes,
            TableKind::MetricAttributes => &config.tables.metric_attributes,
            TableKind::TraceAttributes => &config.tables.trace_attributes,
            TableKind::Logs => &config.tables.logs,
            TableKind::Traces => &config.tables.traces,
            TableKind::Gauge => &config.tables.metrics.gauge,
            TableKind::Sum => &config.tables.metrics.sum,
            TableKind::Histogram => &config.tables.metrics.histogram,
        }
    }

    #[test]
    fn resource_attributes_inline_storage_is_skipped() {
        // 1. Build config from JSON
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "foo",
            "password": "bar",
            "attributes": {
                "resource": {
                    "representation": "json",
                    "storage": "inline"
                }
            }
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config: Config = Config::from_patch(patch);

        // 2. Create a TableEntry
        let entry = TableEntry {
            kind: TableKind::ResourceAttributes,
            config: &config.tables.resource_attributes,
        };

        // 3. Generate SQL
        let sql = build_table_sql(&entry, &config).unwrap();

        // 4. Assert table creation is skipped
        assert!(
            sql.is_none(),
            "resource attribute table should not be created when storage is inline"
        );
    }

    #[test]
    fn resource_attributes_lookup_json_generates_flattened_schema() {
        // 1. Build config
        let json = serde_json::json!({
            "endpoint": "http://localhost:8123",
            "database": "otap",
            "username": "foo",
            "password": "bar",
            "attributes": {
                "resource": {
                    "representation": "json",
                    "storage": "lookup"
                }
            }
        });

        let patch: ConfigPatch = serde_json::from_value(json).unwrap();
        let config: Config = Config::from_patch(patch);

        // 2. Table entry
        let entry = TableEntry {
            kind: TableKind::ResourceAttributes,
            config: &config.tables.resource_attributes,
        };

        // 3. Generate SQL
        let sql = build_table_sql(&entry, &config)
            .expect("SQL generation failed")
            .expect("resource attributes table should be created");

        // 4. Assert core structure
        assert_sql_contains_all(
            &sql,
            &[
                // Table creation
                "CREATE TABLE",
                "resource_attributes",
                // Flattened schema
                "part_id",
                "parent_id",
                // JSON attribute column
                "attributes` JSON",
                // Timestamp
                "insert_time",
                // Ordering / keys
                "ORDER BY (part_id, parent_id, insert_time)",
                "PRIMARY KEY (part_id, parent_id)",
                // Partitioning
                "PARTITION BY (toDate(insert_time))",
            ],
        );

        // 5. Sanity check: not otap array
        assert_sql_not_contains(&sql, "key");
    }

    #[test]
    fn snapshot_all_attribute_table_combinations() {
        let cases = [
            ("resource", TableKind::ResourceAttributes),
            ("scope", TableKind::ScopeAttributes),
            ("log", TableKind::LogAttributes),
            ("trace", TableKind::TraceAttributes),
        ];

        let variants = [
            ("lookup_json", "json", Some("lookup")),
            ("lookup_otap", "otap_array", Some("lookup")),
            ("inline", "json", Some("inline")),
        ];

        for (attr_name, table_kind) in cases {
            for (variant_name, representation, storage) in variants {
                let attributes = build_attributes(attr_name, representation, storage);
                let config = base_config(attributes);

                let entry = TableEntry {
                    kind: table_kind,
                    config: get_table_config(table_kind, &config),
                };

                let maybe_sql = build_table_sql(&entry, &config).expect("sql generation failed");

                if storage == Some("inline") {
                    assert!(
                        maybe_sql.is_none(),
                        "{attr_name} {variant_name} should not create a table"
                    );
                    continue;
                }
                let sql = maybe_sql.unwrap();

                insta::with_settings!({
                    prepend_module_to_snapshot => false,
                    snapshot_path => "table_snapshots"
                }, {
                    insta::assert_snapshot!(format!(
                        "{}_{}",
                        table_kind.as_str(),
                        variant_name
                    ),
                    sql);
                });

                let view_sql =
                    build_attribute_view(&entry, &config).expect("sql generation failed");

                if let Some(view_sql) = view_sql {
                    insta::with_settings!({
                        prepend_module_to_snapshot => false,
                        snapshot_path => "table_snapshots"
                    }, {
                        insta::assert_snapshot!(format!(
                            "{}_otap_view",
                            table_kind.as_str(),
                        ),
                        view_sql);
                    });
                }
            }
        }
    }

    #[test]
    fn snapshot_signal_tables() {
        let cases = [
            (
                "all_lookup",
                serde_json::json!({
                    "resource": { "representation": "json", "storage": "lookup" },
                    "scope":    { "representation": "json", "storage": "lookup" },
                    "log":      { "representation": "json", "storage": "lookup" },
                    "trace":      { "representation": "json", "storage": "lookup" }
                }),
            ),
            (
                "all_inline",
                serde_json::json!({
                    "resource": { "representation": "json", "storage": "inline" },
                    "scope":    { "representation": "json", "storage": "inline" },
                    "log":      { "representation": "json", "storage": "inline" },
                    "trace":      { "representation": "json", "storage": "inline" },
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

                let view_sql =
                    build_signal_view_sql(&entry, &config).expect("sql generation failed");

                if let Some(view_sql) = view_sql {
                    insta::with_settings!({
                        prepend_module_to_snapshot => false,
                        snapshot_path => "table_snapshots"
                    }, {
                        insta::assert_snapshot!(format!(
                            "{}_table_view_{}",
                            signal_type.as_str(),
                            name,
                        ),
                        view_sql);
                    });
                }
            }
        }
    }
}
