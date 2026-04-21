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
//!   “ZSTD + Array” variants) used across the schema constants.
//! - Constants describing the column sets for:
//!   - Signal tables (logs/spans), including common IDs, timestamps, resource/scope fields, and
//!     trace/log-specific fields.
//!   - Inline vs lookup attribute handling, via “inline attribute” placeholder columns (whose type
//!     is overridden at build time based on configured representation) and “lookup ID” columns.
//!   - Attribute tables in two shapes:
//!     - OTAP array representation (`ARROW_ATTRIBUTE_SCHEMA_COLUMNS`) storing per-key typed values,
//!     - Flattened representations (`COMMON_FLATTENED_SCHEMA_COLUMNS` + `FLATTENED_ATTRIBUTE_COLUMN`)
//!       for JSON or `Map(LowCardinality(String), String)` storage.
//!
//! These definitions are consumed by the table/view initialization logic to ensure consistent,
//! ClickStack-compatible schemas and compression settings across all created tables.
use otap_df_pdata::schema::consts;

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
    FixedString(u32),
    UInt32,
    UInt16,
    UInt8,
    UInt64,
    Int64,
    Float64,
    Boolean,
    Json,
    MapLCStringString,
    LowCardinalityString,
    String,
    DateTime,
    DateTime64(u32),
    Dynamic, // placeholder for override
}
impl ColumnType {
    pub fn render(&self) -> String {
        match self {
            ColumnType::FixedString(n) => format!("FixedString({})", n),
            ColumnType::UInt64 => "UInt64".into(),
            ColumnType::UInt32 => "UInt32".into(),
            ColumnType::UInt16 => "UInt16".into(),
            ColumnType::UInt8 => "UInt8".into(),
            ColumnType::Json => "JSON".into(),
            ColumnType::Int64 => "Int64".into(),
            ColumnType::Float64 => "Float64".into(),
            ColumnType::Boolean => "UInt8".into(),
            ColumnType::MapLCStringString => "Map(LowCardinality(String), String)".into(),
            ColumnType::LowCardinalityString => "LowCardinality(String)".into(),
            ColumnType::String => "String".into(),
            ColumnType::DateTime => "DateTime".into(),
            ColumnType::DateTime64(n) => format!("DateTime64({})", n),
            ColumnType::Dynamic => "<dynamic>".into(),
        }
    }
}

pub const SIGNAL_GLOBAL_ID_COLUMNS: &[Column] = &[
    Column::new(
        ch_consts::PART_ID,
        ColumnType::FixedString(32),
        ColumnOpts::ZSTD,
    ),
    Column::new(consts::ID, ColumnType::UInt16, ColumnOpts::ZSTD),
];

pub const INLINE_RESOURCE_ATTR_COLUMN: &Column = &Column::new(
    ch_consts::CH_RESOURCE_ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);

pub const LOOKUP_RESOURCE_ATTR_COLUMN: &Column =
    &Column::new(ch_consts::RESOURCE_ID, ColumnType::UInt32, ColumnOpts::ZSTD);

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

pub const LOOKUP_SCOPE_ATTR_COLUMN: &Column =
    &Column::new(ch_consts::SCOPE_ID, ColumnType::UInt32, ColumnOpts::ZSTD);

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

pub const TIMESTAMP_TIME_COLUMN: &Column = &Column {
    // Timestamp DateTime64(9) CODEC(Delta(8), ZSTD(1))
    name: ch_consts::CH_TIMESTAMP_TIME,
    ch_type: ColumnType::DateTime,
    nullable: false,
    array: false,
    default: Some("toDateTime(Timestamp)"),
    codec: None,
};

pub const TRACE_ID_COLUMN: &Column = &Column::new(
    // TraceId String CODEC(ZSTD(1))
    ch_consts::CH_TRACE_ID,
    ColumnType::FixedString(16),
    ColumnOpts::ZSTD,
);

pub const SPAN_ID_COLUMN: &Column = &Column::new(
    // SpanId String CODEC(ZSTD(1))
    ch_consts::CH_SPAN_ID,
    ColumnType::FixedString(8),
    ColumnOpts::ZSTD,
);

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

// TODO: [support_new_signal] add support here

pub const ARROW_ATTRIBUTE_SCHEMA_COLUMNS: &[Column] = &[
    Column {
        name: ch_consts::PART_ID,
        ch_type: ColumnType::FixedString(32),
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::PARENT_ID,
        ch_type: ColumnType::UInt32,
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: ch_consts::INSERT_TIME,
        ch_type: ColumnType::DateTime,
        nullable: false,
        array: false,
        default: Some("now()"),
        codec: None,
    },
    Column {
        name: consts::ATTRIBUTE_KEY,
        ch_type: ColumnType::String,
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::ATTRIBUTE_TYPE,
        ch_type: ColumnType::UInt8,
        nullable: false,
        array: false,
        default: None,
        codec: None,
    },
    Column {
        name: consts::ATTRIBUTE_STR,
        ch_type: ColumnType::String,
        nullable: true,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::ATTRIBUTE_INT,
        ch_type: ColumnType::Int64,
        nullable: true,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::ATTRIBUTE_DOUBLE,
        ch_type: ColumnType::Float64,
        nullable: true,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::ATTRIBUTE_BOOL,
        ch_type: ColumnType::Boolean,
        nullable: true,
        array: false,
        default: None,
        codec: None,
    },
    Column {
        name: consts::ATTRIBUTE_BYTES,
        ch_type: ColumnType::String,
        nullable: true,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::ATTRIBUTE_SER,
        ch_type: ColumnType::String,
        nullable: true,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
];

pub const COMMON_FLATTENED_SCHEMA_COLUMNS: &[Column] = &[
    Column {
        name: ch_consts::PART_ID,
        ch_type: ColumnType::FixedString(32),
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: consts::PARENT_ID,
        ch_type: ColumnType::UInt32,
        nullable: false,
        array: false,
        default: None,
        codec: Some("ZSTD(1)"),
    },
    Column {
        name: ch_consts::INSERT_TIME,
        ch_type: ColumnType::DateTime,
        nullable: false,
        array: false,
        default: Some("now()"),
        codec: None,
    },
];

pub const FLATTENED_ATTRIBUTE_COLUMN: &Column = &Column::new(
    consts::ATTRIBUTES,
    ColumnType::Dynamic, // placeholder
    ColumnOpts::ZSTD,
);
