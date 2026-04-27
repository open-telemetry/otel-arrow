//! ClickHouse column definitions for the OTAP ClickHouse exporter.
//!
//! This module provides the ClickHouse-side schema building blocks used by the exporter when
//! generating `CREATE TABLE` DDL.
//!
//! It contains:
//!
//! - A lightweight [`Column`] model with rendering logic (`render`) that produces ClickHouse column
//!   declarations, including optional `Array(...)` wrapping, `NULL`, `DEFAULT`, and `CODEC(...)`.
//! - [`ColumnType`], an enum of ClickHouse types used by the exporter (including JSON, low-cardinality
//!   strings, map types, and high-precision timestamps), with a `render` method to convert to SQL.
//! - [`ColumnOpts`], a small set of reusable column option presets (notably ZSTD compression and
//!   "ZSTD + Array" variants) used across the schema constants.
//! - Constants describing the column sets for signal tables (logs/spans), including common IDs,
//!   timestamps, resource/scope fields, and trace/log-specific fields.
//!
//! These definitions are consumed by the table/view initialization logic to ensure consistent,
//! ClickStack-compatible schemas and compression settings across all created tables.
use crate::clickhouse_exporter::consts as ch_consts;

#[derive(Debug, Clone)]
pub struct Column {
    pub name: &'static str,
    pub ch_type: ColumnType,
    pub nullable: bool,
    pub array: bool,
    pub default: Option<&'static str>,
    pub codec: Option<&'static str>,
}
impl Column {
    pub const fn new(name: &'static str, ch_type: ColumnType, opts: ColumnOpts) -> Self {
        Self {
            name,
            ch_type,
            array: opts.array,
            nullable: opts.nullable,
            default: opts.default,
            codec: opts.codec,
        }
    }

    pub fn render(&self) -> String {
        let mut base = format!("`{}` ", self.name);

        match self.array {
            true => {
                base.push_str(&format!("Array({})", self.ch_type.render()));
            }
            false => {
                base.push_str(self.ch_type.render().as_str());
            }
        }

        if self.nullable {
            base.push_str(" NULL");
        }

        if let Some(default) = self.default {
            base.push_str(&format!(" DEFAULT {}", default));
        }

        if let Some(codec) = self.codec {
            base.push_str(&format!(" CODEC({})", codec));
        }

        base
    }
}

#[derive(Copy, Clone)]
pub struct ColumnOpts {
    pub nullable: bool,
    pub array: bool,
    pub default: Option<&'static str>,
    pub codec: Option<&'static str>,
}

impl ColumnOpts {
    pub const ZSTD: Self = Self {
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    };
    pub const ZSTD_ARRAY: Self = Self {
        nullable: false,
        array: true,
        default: None,
        codec: Some("ZSTD(1)"),
    };
}

#[derive(Debug, Clone)]
pub enum ColumnType {
    UInt8,
    UInt64,
    Json,
    MapLCStringString,
    LowCardinalityString,
    String,
    DateTime64(u32),
    Dynamic, // placeholder for override
}
impl ColumnType {
    pub fn render(&self) -> String {
        match self {
            ColumnType::UInt64 => "UInt64".into(),
            ColumnType::UInt8 => "UInt8".into(),
            ColumnType::Json => "JSON".into(),
            ColumnType::MapLCStringString => "Map(LowCardinality(String), String)".into(),
            ColumnType::LowCardinalityString => "LowCardinality(String)".into(),
            ColumnType::String => "String".into(),
            ColumnType::DateTime64(n) => format!("DateTime64({})", n),
            ColumnType::Dynamic => "<dynamic>".into(),
        }
    }
}

pub const INLINE_RESOURCE_ATTR_COLUMN: &Column = &Column::new(
    ch_consts::CH_RESOURCE_ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);

pub const RESOURCE_SCHEMA_URL_COLUMN: &Column = &Column::new(
    // ResourceSchemaUrl LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_RESOURCE_SCHEMA_URL,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const INLINE_SCOPE_ATTR_COLUMN: &Column = &Column::new(
    ch_consts::CH_SCOPE_ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);

pub const SCOPE_SCHEMA_URL_COLUMN: &Column = &Column::new(
    ch_consts::CH_SCOPE_SCHEMA_URL,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);
pub const SCOPE_NAME_COLUMN: &Column = &Column::new(
    ch_consts::CH_SCOPE_NAME,
    ColumnType::String,
    ColumnOpts::ZSTD,
);
pub const SCOPE_VERSION_COLUMN: &Column = &Column::new(
    ch_consts::CH_SCOPE_VERSION,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const INLINE_LOG_ATTR_COLUMN: &Column = &Column::new(
    ch_consts::CH_LOG_ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);

pub const INLINE_SPAN_ATTR_COLUMN: &Column = &Column::new(
    ch_consts::CH_SPAN_ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);

pub const TIMESTAMP_COLUMN: &Column = &Column {
    // Timestamp DateTime64(9) CODEC(Delta(8), ZSTD(1))
    name: ch_consts::CH_TIMESTAMP,
    ch_type: ColumnType::DateTime64(9),
    nullable: false,
    array: false,
    default: None,
    codec: Some("Delta(8), ZSTD(1)"),
};

pub const TRACE_ID_COLUMN: &Column = &Column::new(
    // TraceId String CODEC(ZSTD(1))
    ch_consts::CH_TRACE_ID,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const SPAN_ID_COLUMN: &Column = &Column::new(
    // SpanId String CODEC(ZSTD(1))
    ch_consts::CH_SPAN_ID,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const TRACE_FLAGS_COLUMN: &Column = &Column {
    // TraceFlags UInt8
    name: ch_consts::CH_TRACE_FLAGS,
    ch_type: ColumnType::UInt8,
    nullable: false,
    array: false,
    default: None,
    codec: None,
};

pub const SEVERITY_TEXT_COLUMN: &Column = &Column::new(
    // SeverityText LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_SEVERITY_TEXT,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const SEVERITY_NUMBER_COLUMN: &Column = &Column {
    // SeverityNumber UInt8
    name: ch_consts::CH_SEVERITY_NUMBER,
    ch_type: ColumnType::UInt8,
    nullable: false,
    array: false,
    default: None,
    codec: None,
};

pub const SERVICE_NAME_COLUMN: &Column = &Column::new(
    // ServiceName LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_SERVICE_NAME,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const BODY_COLUMN: &Column = &Column::new(
    // Body String CODEC(ZSTD(1))
    ch_consts::CH_BODY,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const EVENT_NAME_COLUMN: &Column = &Column::new(
    // EventName String CODEC(ZSTD(1))
    ch_consts::CH_EVENT_NAME,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const PARENT_SPAN_ID_COLUMN: &Column = &Column::new(
    // ParentSpanId String CODEC(ZSTD(1))
    ch_consts::CH_PARENT_SPAN_ID,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const TRACE_STATE_COLUMN: &Column = &Column::new(
    // TraceState String CODEC(ZSTD(1))
    ch_consts::CH_TRACE_STATE,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const SPAN_NAME_COLUMN: &Column = &Column::new(
    // SpanName LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_SPAN_NAME,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const SPAN_KIND_COLUMN: &Column = &Column::new(
    // SpanKind LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_SPAN_KIND,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const DURATION_COLUMN: &Column = &Column::new(
    // Duration UInt64 CODEC(ZSTD(1))
    ch_consts::CH_DURATION,
    ColumnType::UInt64,
    ColumnOpts::ZSTD,
);

pub const STATUS_CODE_COLUMN: &Column = &Column::new(
    // StatusCode LowCardinality(String) CODEC(ZSTD(1))
    ch_consts::CH_STATUS_CODE,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD,
);

pub const STATUS_MESSAGE_COLUMN: &Column = &Column::new(
    // StatusMessage String CODEC(ZSTD(1))
    ch_consts::CH_STATUS_MESSAGE,
    ColumnType::String,
    ColumnOpts::ZSTD,
);

pub const EVENTS_TIMESTAMP_COLUMN: &Column = &Column::new(
    // Events.Timestamp Array(DateTime64(9)) CODEC(ZSTD(1))
    ch_consts::CH_EVENTS_TIMESTAMP,
    ColumnType::DateTime64(9),
    ColumnOpts::ZSTD_ARRAY,
);

pub const EVENTS_NAME_COLUMN: &Column = &Column::new(
    // Events.Name Array(LowCardinality(String)) CODEC(ZSTD(1))
    ch_consts::CH_EVENTS_NAME,
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD_ARRAY,
);

pub const EVENTS_ATTRIBUTES_COLUMN: &Column = &Column::new(
    // Events.Attributes Array(Map(LowCardinality(String), String)) CODEC(ZSTD(1))
    ch_consts::CH_EVENTS_ATTRIBUTES,
    ColumnType::MapLCStringString,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LINKS_TRACE_ID_COLUMN: &Column = &Column::new(
    // Links.TraceId Array(String) CODEC(ZSTD(1))
    ch_consts::CH_LINKS_TRACE_ID,
    ColumnType::String,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LINKS_SPAN_ID_COLUMN: &Column = &Column::new(
    // Links.SpanId Array(String) CODEC(ZSTD(1))
    ch_consts::CH_LINKS_SPAN_ID,
    ColumnType::String,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LINKS_TRACE_STATE_COLUMN: &Column = &Column::new(
    // Links.TraceState Array(String) CODEC(ZSTD(1))
    ch_consts::CH_LINKS_TRACE_STATE,
    ColumnType::String,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LINKS_ATTRIBUTES_COLUMN: &Column = &Column::new(
    // Links.Attributes Array(Map(LowCardinality(String), String)) CODEC(ZSTD(1))
    ch_consts::CH_LINKS_ATTRIBUTES,
    ColumnType::MapLCStringString,
    ColumnOpts::ZSTD_ARRAY,
);

/// A ClickHouse secondary index declaration.
///
/// Rendered as: `INDEX <name> <expression> TYPE <index_type> GRANULARITY <granularity>`
#[derive(Debug, Clone)]
pub struct Index {
    pub name: &'static str,
    pub expression: &'static str,
    pub index_type: &'static str,
    pub granularity: u32,
}

impl Index {
    pub const fn new(
        name: &'static str,
        expression: &'static str,
        index_type: &'static str,
        granularity: u32,
    ) -> Self {
        Self {
            name,
            expression,
            index_type,
            granularity,
        }
    }

    pub fn render(&self) -> String {
        format!(
            "INDEX {} {} TYPE {} GRANULARITY {}",
            self.name, self.expression, self.index_type, self.granularity
        )
    }
}

// --- Companion *AttributesKeys columns for JSON attribute mode ---

pub const RESOURCE_ATTRIBUTES_KEYS_COLUMN: &Column = &Column::new(
    "ResourceAttributesKeys",
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD_ARRAY,
);

pub const SCOPE_ATTRIBUTES_KEYS_COLUMN: &Column = &Column::new(
    "ScopeAttributesKeys",
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LOG_ATTRIBUTES_KEYS_COLUMN: &Column = &Column::new(
    "LogAttributesKeys",
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD_ARRAY,
);

pub const SPAN_ATTRIBUTES_KEYS_COLUMN: &Column = &Column::new(
    "SpanAttributesKeys",
    ColumnType::LowCardinalityString,
    ColumnOpts::ZSTD_ARRAY,
);

// --- JSON variants of Events/Links Attributes ---

pub const EVENTS_ATTRIBUTES_JSON_COLUMN: &Column = &Column::new(
    ch_consts::CH_EVENTS_ATTRIBUTES,
    ColumnType::Json,
    ColumnOpts::ZSTD_ARRAY,
);

pub const LINKS_ATTRIBUTES_JSON_COLUMN: &Column = &Column::new(
    ch_consts::CH_LINKS_ATTRIBUTES,
    ColumnType::Json,
    ColumnOpts::ZSTD_ARRAY,
);

// --- Index constants matching Go OTel Collector ClickHouse exporter ---

pub const IDX_TRACE_ID: Index = Index::new("idx_trace_id", "TraceId", "bloom_filter(0.001)", 1);

pub const IDX_RES_ATTR_KEY: Index = Index::new(
    "idx_res_attr_key",
    "mapKeys(ResourceAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_RES_ATTR_VALUE: Index = Index::new(
    "idx_res_attr_value",
    "mapValues(ResourceAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_SCOPE_ATTR_KEY: Index = Index::new(
    "idx_scope_attr_key",
    "mapKeys(ScopeAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_SCOPE_ATTR_VALUE: Index = Index::new(
    "idx_scope_attr_value",
    "mapValues(ScopeAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_LOG_ATTR_KEY: Index = Index::new(
    "idx_log_attr_key",
    "mapKeys(LogAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_LOG_ATTR_VALUE: Index = Index::new(
    "idx_log_attr_value",
    "mapValues(LogAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_LOWER_BODY: Index = Index::new(
    "idx_lower_body",
    "lower(Body)",
    "tokenbf_v1(32768, 3, 0)",
    8,
);

pub const IDX_SPAN_ATTR_KEY: Index = Index::new(
    "idx_span_attr_key",
    "mapKeys(SpanAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_SPAN_ATTR_VALUE: Index = Index::new(
    "idx_span_attr_value",
    "mapValues(SpanAttributes)",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_DURATION: Index = Index::new("idx_duration", "Duration", "minmax", 1);

// --- JSON mode index constants (on companion Keys columns) ---

pub const IDX_RES_ATTR_KEYS: Index = Index::new(
    "idx_res_attr_keys",
    "ResourceAttributesKeys",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_SCOPE_ATTR_KEYS: Index = Index::new(
    "idx_scope_attr_keys",
    "ScopeAttributesKeys",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_LOG_ATTR_KEYS: Index = Index::new(
    "idx_log_attr_keys",
    "LogAttributesKeys",
    "bloom_filter(0.01)",
    1,
);

pub const IDX_SPAN_ATTR_KEYS: Index = Index::new(
    "idx_span_attr_keys",
    "SpanAttributesKeys",
    "bloom_filter(0.01)",
    1,
);

// TODO: [support_new_signal] add support here

#[cfg(test)]
mod tests {
    use super::*;

    // --- Task 1.4: Index::render() tests ---

    #[test]
    fn test_index_render_bloom_filter() {
        let idx = Index::new("idx_trace_id", "TraceId", "bloom_filter(0.001)", 1);
        assert_eq!(
            idx.render(),
            "INDEX idx_trace_id TraceId TYPE bloom_filter(0.001) GRANULARITY 1"
        );
    }

    #[test]
    fn test_index_render_bloom_filter_on_map_expression() {
        let idx = Index::new(
            "idx_res_attr_key",
            "mapKeys(ResourceAttributes)",
            "bloom_filter(0.01)",
            1,
        );
        assert_eq!(
            idx.render(),
            "INDEX idx_res_attr_key mapKeys(ResourceAttributes) TYPE bloom_filter(0.01) GRANULARITY 1"
        );
    }

    #[test]
    fn test_index_render_tokenbf() {
        let idx = Index::new(
            "idx_lower_body",
            "lower(Body)",
            "tokenbf_v1(32768, 3, 0)",
            8,
        );
        assert_eq!(
            idx.render(),
            "INDEX idx_lower_body lower(Body) TYPE tokenbf_v1(32768, 3, 0) GRANULARITY 8"
        );
    }

    #[test]
    fn test_index_render_minmax() {
        let idx = Index::new("idx_duration", "Duration", "minmax", 1);
        assert_eq!(
            idx.render(),
            "INDEX idx_duration Duration TYPE minmax GRANULARITY 1"
        );
    }

    // --- Task 1.5: Column::render() tests ---

    #[test]
    fn test_column_render_with_zstd_codec() {
        let col = Column::new(
            "Timestamp",
            ColumnType::DateTime64(9),
            ColumnOpts {
                nullable: false,
                array: false,
                default: None,
                codec: Some("Delta(8), ZSTD(1)"),
            },
        );
        assert_eq!(
            col.render(),
            "`Timestamp` DateTime64(9) CODEC(Delta(8), ZSTD(1))"
        );
    }

    #[test]
    fn test_column_render_array_with_codec() {
        let col = Column::new(
            "Events.Timestamp",
            ColumnType::DateTime64(9),
            ColumnOpts::ZSTD_ARRAY,
        );
        assert_eq!(
            col.render(),
            "`Events.Timestamp` Array(DateTime64(9)) CODEC(ZSTD(1))"
        );
    }

    #[test]
    fn test_column_render_no_codec() {
        let col = Column {
            name: "SeverityNumber",
            ch_type: ColumnType::UInt8,
            nullable: false,
            array: false,
            default: None,
            codec: None,
        };
        assert_eq!(col.render(), "`SeverityNumber` UInt8");
    }

    #[test]
    fn test_column_render_with_default() {
        let col = Column {
            name: "TimestampTime",
            ch_type: ColumnType::DateTime64(9),
            nullable: false,
            array: false,
            default: Some("toDateTime(Timestamp)"),
            codec: None,
        };
        assert_eq!(
            col.render(),
            "`TimestampTime` DateTime64(9) DEFAULT toDateTime(Timestamp)"
        );
    }

    #[test]
    fn test_column_render_nullable() {
        let col = Column {
            name: "NullableCol",
            ch_type: ColumnType::String,
            nullable: true,
            array: false,
            default: None,
            codec: Some("ZSTD(1)"),
        };
        assert_eq!(col.render(), "`NullableCol` String NULL CODEC(ZSTD(1))");
    }

    // --- ColumnType::render() tests ---

    #[test]
    fn test_column_type_render_all_variants() {
        assert_eq!(ColumnType::UInt8.render(), "UInt8");
        assert_eq!(ColumnType::UInt64.render(), "UInt64");
        assert_eq!(ColumnType::Json.render(), "JSON");
        assert_eq!(
            ColumnType::MapLCStringString.render(),
            "Map(LowCardinality(String), String)"
        );
        assert_eq!(
            ColumnType::LowCardinalityString.render(),
            "LowCardinality(String)"
        );
        assert_eq!(ColumnType::String.render(), "String");
        assert_eq!(ColumnType::DateTime64(9).render(), "DateTime64(9)");
        assert_eq!(ColumnType::Dynamic.render(), "<dynamic>");
    }

    // --- Task 1.6: Column constants render correct DDL ---

    #[test]
    fn test_trace_flags_column_constant() {
        assert_eq!(TRACE_FLAGS_COLUMN.name, "TraceFlags");
        assert!(matches!(TRACE_FLAGS_COLUMN.ch_type, ColumnType::UInt8));
        assert_eq!(TRACE_FLAGS_COLUMN.codec, None);
        assert_eq!(TRACE_FLAGS_COLUMN.render(), "`TraceFlags` UInt8");
    }

    #[test]
    fn test_timestamp_column_constant() {
        assert_eq!(
            TIMESTAMP_COLUMN.render(),
            "`Timestamp` DateTime64(9) CODEC(Delta(8), ZSTD(1))"
        );
    }

    #[test]
    fn test_trace_id_column_constant() {
        assert_eq!(TRACE_ID_COLUMN.render(), "`TraceId` String CODEC(ZSTD(1))");
    }

    #[test]
    fn test_span_id_column_constant() {
        assert_eq!(SPAN_ID_COLUMN.render(), "`SpanId` String CODEC(ZSTD(1))");
    }

    #[test]
    fn test_service_name_column_constant() {
        assert_eq!(
            SERVICE_NAME_COLUMN.render(),
            "`ServiceName` LowCardinality(String) CODEC(ZSTD(1))"
        );
    }

    #[test]
    fn test_duration_column_constant() {
        assert_eq!(DURATION_COLUMN.render(), "`Duration` UInt64 CODEC(ZSTD(1))");
    }

    #[test]
    fn test_events_attributes_column_constant() {
        assert_eq!(
            EVENTS_ATTRIBUTES_COLUMN.render(),
            "`Events.Attributes` Array(Map(LowCardinality(String), String)) CODEC(ZSTD(1))"
        );
    }

    #[test]
    fn test_links_attributes_column_constant() {
        assert_eq!(
            LINKS_ATTRIBUTES_COLUMN.render(),
            "`Links.Attributes` Array(Map(LowCardinality(String), String)) CODEC(ZSTD(1))"
        );
    }

    #[test]
    fn test_all_logs_columns_render_valid_ddl() {
        // All column constants that appear in the logs DDL should render non-empty strings
        let columns: Vec<&Column> = vec![
            TIMESTAMP_COLUMN,
            TRACE_ID_COLUMN,
            SPAN_ID_COLUMN,
            TRACE_FLAGS_COLUMN,
            SEVERITY_TEXT_COLUMN,
            SEVERITY_NUMBER_COLUMN,
            SERVICE_NAME_COLUMN,
            BODY_COLUMN,
            RESOURCE_SCHEMA_URL_COLUMN,
            SCOPE_SCHEMA_URL_COLUMN,
            SCOPE_NAME_COLUMN,
            SCOPE_VERSION_COLUMN,
            EVENT_NAME_COLUMN,
        ];
        for col in columns {
            let rendered = col.render();
            assert!(!rendered.is_empty(), "Column {} rendered empty", col.name);
            assert!(
                rendered.starts_with('`'),
                "Column {} should start with backtick: {}",
                col.name,
                rendered
            );
        }
    }

    #[test]
    fn test_all_traces_columns_render_valid_ddl() {
        let columns: Vec<&Column> = vec![
            TIMESTAMP_COLUMN,
            TRACE_ID_COLUMN,
            SPAN_ID_COLUMN,
            PARENT_SPAN_ID_COLUMN,
            TRACE_STATE_COLUMN,
            SPAN_NAME_COLUMN,
            SPAN_KIND_COLUMN,
            SERVICE_NAME_COLUMN,
            SCOPE_NAME_COLUMN,
            SCOPE_VERSION_COLUMN,
            DURATION_COLUMN,
            STATUS_CODE_COLUMN,
            STATUS_MESSAGE_COLUMN,
            EVENTS_TIMESTAMP_COLUMN,
            EVENTS_NAME_COLUMN,
            EVENTS_ATTRIBUTES_COLUMN,
            LINKS_TRACE_ID_COLUMN,
            LINKS_SPAN_ID_COLUMN,
            LINKS_TRACE_STATE_COLUMN,
            LINKS_ATTRIBUTES_COLUMN,
        ];
        for col in columns {
            let rendered = col.render();
            assert!(!rendered.is_empty(), "Column {} rendered empty", col.name);
            assert!(
                rendered.starts_with('`'),
                "Column {} should start with backtick: {}",
                col.name,
                rendered
            );
        }
    }

    // --- Index constants render correct DDL ---

    #[test]
    fn test_all_index_constants_render() {
        let indexes = vec![
            &IDX_TRACE_ID,
            &IDX_RES_ATTR_KEY,
            &IDX_RES_ATTR_VALUE,
            &IDX_SCOPE_ATTR_KEY,
            &IDX_SCOPE_ATTR_VALUE,
            &IDX_LOG_ATTR_KEY,
            &IDX_LOG_ATTR_VALUE,
            &IDX_LOWER_BODY,
            &IDX_SPAN_ATTR_KEY,
            &IDX_SPAN_ATTR_VALUE,
            &IDX_DURATION,
            &IDX_RES_ATTR_KEYS,
            &IDX_SCOPE_ATTR_KEYS,
            &IDX_LOG_ATTR_KEYS,
            &IDX_SPAN_ATTR_KEYS,
        ];
        for idx in indexes {
            let rendered = idx.render();
            assert!(
                rendered.starts_with("INDEX "),
                "Index {} should start with INDEX: {}",
                idx.name,
                rendered
            );
            assert!(
                rendered.contains("TYPE "),
                "Index {} should contain TYPE: {}",
                idx.name,
                rendered
            );
            assert!(
                rendered.contains("GRANULARITY "),
                "Index {} should contain GRANULARITY: {}",
                idx.name,
                rendered
            );
        }
    }
}
