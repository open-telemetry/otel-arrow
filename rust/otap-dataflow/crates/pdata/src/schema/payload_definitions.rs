// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload type definitions from the OTAP spec.
//!
//! Each payload type has a definition that describes the columns it contains,
//! their native Arrow types, and dictionary encoding constraints.
//!
//! See docs/otap-spec.md sections 5.1-5.4 for the full specification.

use super::consts::*;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Native Arrow types from the OTAP spec.
///
/// This is a const-constructible alternative to `arrow::datatypes::DataType`
/// which uses Arc/Box for complex types like Struct and List.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeType {
    Boolean,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float64,
    Utf8,
    Binary,
    FixedSizeBinary(i32),
    /// Timestamp(Nanosecond, None)
    TimestampNs,
    /// Duration(Nanosecond)
    DurationNs,
    /// Struct (sub-fields defined via dotted paths in the same definition)
    Struct,
    /// List(Struct { ... })
    ListStruct,
    /// List(UInt64)
    ListUInt64,
    /// List(Float64)
    ListFloat64,
}

/// Minimum dictionary key size constraint from the OTAP spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinDictKeySize {
    U8,
    U16,
}

/// Column definition from the OTAP spec.
#[derive(Debug, Clone, Copy)]
pub struct ColumnDef {
    /// The native (non-dictionary) Arrow type for this column.
    pub native_type: NativeType,
    /// Minimum dictionary key size if dictionary encoding is allowed.
    /// None if the column does not support dictionary encoding.
    pub min_dict_key_size: Option<MinDictKeySize>,
}

/// Schema definition for a payload type.
///
/// Columns are looked up via match statements for O(1) dispatch on string
/// names. Top-level columns use `get()`, struct sub-fields use
/// `get_nested(parent, child)`.
pub struct PayloadDefinition {
    pub(crate) get_fn: fn(&str) -> Option<&'static ColumnDef>,
    pub(crate) get_nested_fn: fn(&str, &str) -> Option<&'static ColumnDef>,
}

impl PayloadDefinition {
    /// An empty definition with no columns. When used with `select_dictionary_type`,
    /// all dictionary columns will be converted to their native type since no column
    /// will be found in the definition.
    pub const EMPTY: PayloadDefinition = PayloadDefinition {
        get_fn: |_| None,
        get_nested_fn: |_, _| None,
    };

    /// Look up a column by name. Returns `None` if the column is not present
    /// in the definition.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&'static ColumnDef> {
        (self.get_fn)(name)
    }

    /// Look up a nested column without allocating. E.g., `get_nested("body",
    /// "int")` looks up the `int` sub-field of the `body` struct.
    #[must_use]
    pub fn get_nested(&self, parent: &str, child: &str) -> Option<&'static ColumnDef> {
        (self.get_nested_fn)(parent, child)
    }
}

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/// Get the payload definition for a given payload type.
#[must_use]
pub fn get_definition(typ: ArrowPayloadType) -> &'static PayloadDefinition {
    match typ {
        ArrowPayloadType::Unknown => &EMPTY_DEFINITION,

        // Logs
        ArrowPayloadType::Logs => &LOGS_DEFINITION,
        ArrowPayloadType::LogAttrs => &U16_ATTRS_DEFINITION,

        // Traces
        ArrowPayloadType::Spans => &SPANS_DEFINITION,
        ArrowPayloadType::SpanAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::SpanEvents => &SPAN_EVENTS_DEFINITION,
        ArrowPayloadType::SpanLinks => &SPAN_LINKS_DEFINITION,
        ArrowPayloadType::SpanEventAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::SpanLinkAttrs => &U32_ATTRS_DEFINITION,

        // Metrics
        ArrowPayloadType::UnivariateMetrics => &UNIVARIATE_METRICS_DEFINITION,
        ArrowPayloadType::MultivariateMetrics => &EMPTY_DEFINITION,
        ArrowPayloadType::NumberDataPoints => &NUMBER_DATA_POINTS_DEFINITION,
        ArrowPayloadType::SummaryDataPoints => &SUMMARY_DATA_POINTS_DEFINITION,
        ArrowPayloadType::HistogramDataPoints => &HISTOGRAM_DATA_POINTS_DEFINITION,
        ArrowPayloadType::ExpHistogramDataPoints => &EXP_HISTOGRAM_DATA_POINTS_DEFINITION,
        ArrowPayloadType::NumberDpExemplars => &EXEMPLARS_DEFINITION,
        ArrowPayloadType::HistogramDpExemplars => &EXEMPLARS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplars => &EXEMPLARS_DEFINITION,

        // Metric attributes
        ArrowPayloadType::MetricAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::NumberDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::SummaryDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::HistogramDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::NumberDpExemplarAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::HistogramDpExemplarAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs => &U32_ATTRS_DEFINITION,

        // Common
        ArrowPayloadType::ResourceAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::ScopeAttrs => &U16_ATTRS_DEFINITION,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const fn native(native_type: NativeType) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: None,
    }
}

const fn dict(native_type: NativeType, min: MinDictKeySize) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: Some(min),
    }
}

const U8: MinDictKeySize = MinDictKeySize::U8;
const U16: MinDictKeySize = MinDictKeySize::U16;

// ---------------------------------------------------------------------------
// Empty definition
// ---------------------------------------------------------------------------

static EMPTY_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: |_| None,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// U16 Attributes (RESOURCE_ATTRS, SCOPE_ATTRS, LOG_ATTRS, METRIC_ATTRS,
//                 SPAN_ATTRS)
// Spec: 5.4.2
// ---------------------------------------------------------------------------

fn u16_attrs_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static KEY: ColumnDef = dict(NativeType::Utf8, U8);
    static TYPE: ColumnDef = native(NativeType::UInt8);
    static STR: ColumnDef = dict(NativeType::Utf8, U16);
    static INT: ColumnDef = dict(NativeType::Int64, U16);
    static DOUBLE: ColumnDef = native(NativeType::Float64);
    static BOOL: ColumnDef = native(NativeType::Boolean);
    static BYTES: ColumnDef = dict(NativeType::Binary, U16);
    static SER: ColumnDef = dict(NativeType::Binary, U16);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        ATTRIBUTE_KEY => Some(&KEY),
        ATTRIBUTE_TYPE => Some(&TYPE),
        // Optional
        ATTRIBUTE_STR => Some(&STR),
        ATTRIBUTE_INT => Some(&INT),
        ATTRIBUTE_DOUBLE => Some(&DOUBLE),
        ATTRIBUTE_BOOL => Some(&BOOL),
        ATTRIBUTE_BYTES => Some(&BYTES),
        ATTRIBUTE_SER => Some(&SER),
        _ => None,
    }
}

static U16_ATTRS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: u16_attrs_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// U32 Attributes (SPAN_EVENT_ATTRS, SPAN_LINK_ATTRS, all DP attrs,
//                 all exemplar attrs)
// Spec: 5.4.1
// ---------------------------------------------------------------------------

fn u32_attrs_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = dict(NativeType::UInt32, U8);
    static KEY: ColumnDef = dict(NativeType::Utf8, U8);
    static TYPE: ColumnDef = native(NativeType::UInt8);
    static STR: ColumnDef = dict(NativeType::Utf8, U16);
    static INT: ColumnDef = dict(NativeType::Int64, U16);
    static DOUBLE: ColumnDef = native(NativeType::Float64);
    static BOOL: ColumnDef = native(NativeType::Boolean);
    static BYTES: ColumnDef = dict(NativeType::Binary, U16);
    static SER: ColumnDef = dict(NativeType::Binary, U16);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        ATTRIBUTE_KEY => Some(&KEY),
        ATTRIBUTE_TYPE => Some(&TYPE),
        // Optional
        ATTRIBUTE_STR => Some(&STR),
        ATTRIBUTE_INT => Some(&INT),
        ATTRIBUTE_DOUBLE => Some(&DOUBLE),
        ATTRIBUTE_BOOL => Some(&BOOL),
        ATTRIBUTE_BYTES => Some(&BYTES),
        ATTRIBUTE_SER => Some(&SER),
        _ => None,
    }
}

static U32_ATTRS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: u32_attrs_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// LOGS (ROOT) - Spec: 5.1.1
// ---------------------------------------------------------------------------

fn logs_get(name: &str) -> Option<&'static ColumnDef> {
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static OBSERVED: ColumnDef = native(NativeType::TimestampNs);
    static BODY_COL: ColumnDef = native(NativeType::Struct);
    static LOG_ID: ColumnDef = native(NativeType::UInt16);
    static SEV_NUM: ColumnDef = dict(NativeType::Int32, U8);
    static SEV_TEXT: ColumnDef = dict(NativeType::Utf8, U8);
    static DROP_ATTR: ColumnDef = native(NativeType::UInt32);
    static EVNAME: ColumnDef = dict(NativeType::Utf8, U8);
    static LOG_FLAGS: ColumnDef = native(NativeType::UInt32);
    static TID: ColumnDef = dict(NativeType::FixedSizeBinary(16), U8);
    static SID: ColumnDef = dict(NativeType::FixedSizeBinary(8), U8);
    static SURL: ColumnDef = dict(NativeType::Utf8, U8);
    static RES: ColumnDef = native(NativeType::Struct);
    static SC: ColumnDef = native(NativeType::Struct);
    match name {
        // Required
        TIME_UNIX_NANO => Some(&TIME),
        OBSERVED_TIME_UNIX_NANO => Some(&OBSERVED),
        BODY => Some(&BODY_COL),
        // Optional
        ID => Some(&LOG_ID),
        SEVERITY_NUMBER => Some(&SEV_NUM),
        SEVERITY_TEXT => Some(&SEV_TEXT),
        DROPPED_ATTRIBUTES_COUNT => Some(&DROP_ATTR),
        EVENT_NAME => Some(&EVNAME),
        FLAGS => Some(&LOG_FLAGS),
        TRACE_ID => Some(&TID),
        SPAN_ID => Some(&SID),
        SCHEMA_URL => Some(&SURL),
        RESOURCE => Some(&RES),
        SCOPE => Some(&SC),
        _ => None,
    }
}

fn logs_get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
    static BODY_TYPE: ColumnDef = native(NativeType::UInt8);
    static BODY_STR: ColumnDef = dict(NativeType::Utf8, U16);
    static BODY_INT: ColumnDef = dict(NativeType::Int64, U16);
    static BODY_DOUBLE: ColumnDef = native(NativeType::Float64);
    static BODY_BOOL: ColumnDef = native(NativeType::Boolean);
    static BODY_BYTES: ColumnDef = dict(NativeType::Binary, U16);
    static BODY_SER: ColumnDef = dict(NativeType::Binary, U16);
    static RES_ID: ColumnDef = native(NativeType::UInt16);
    static RES_DROP: ColumnDef = native(NativeType::UInt32);
    static RES_SURL: ColumnDef = dict(NativeType::Utf8, U8);
    static SC_ID: ColumnDef = native(NativeType::UInt16);
    static SC_DROP: ColumnDef = native(NativeType::UInt32);
    static SC_NAME: ColumnDef = dict(NativeType::Utf8, U8);
    static SC_VER: ColumnDef = dict(NativeType::Utf8, U8);
    match (parent, child) {
        // Required (body.type, body.str are required sub-fields of body)
        (BODY, ATTRIBUTE_TYPE) => Some(&BODY_TYPE),
        (BODY, ATTRIBUTE_STR) => Some(&BODY_STR),
        // Optional body sub-fields
        (BODY, ATTRIBUTE_INT) => Some(&BODY_INT),
        (BODY, ATTRIBUTE_DOUBLE) => Some(&BODY_DOUBLE),
        (BODY, ATTRIBUTE_BOOL) => Some(&BODY_BOOL),
        (BODY, ATTRIBUTE_BYTES) => Some(&BODY_BYTES),
        (BODY, ATTRIBUTE_SER) => Some(&BODY_SER),
        // Resource sub-fields
        (RESOURCE, ID) => Some(&RES_ID),
        (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RES_DROP),
        (RESOURCE, SCHEMA_URL) => Some(&RES_SURL),
        // Scope sub-fields
        (SCOPE, ID) => Some(&SC_ID),
        (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SC_DROP),
        (SCOPE, NAME) => Some(&SC_NAME),
        (SCOPE, VERSION) => Some(&SC_VER),
        _ => None,
    }
}

static LOGS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: logs_get,
    get_nested_fn: logs_get_nested,
};

// ---------------------------------------------------------------------------
// SPANS (ROOT) - Spec: 5.2.1
// ---------------------------------------------------------------------------

fn spans_get(name: &str) -> Option<&'static ColumnDef> {
    static START: ColumnDef = native(NativeType::TimestampNs);
    static DUR: ColumnDef = native(NativeType::DurationNs);
    static TID: ColumnDef = native(NativeType::FixedSizeBinary(16));
    static SID: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static N: ColumnDef = native(NativeType::Utf8);
    static SPAN_ID_COL: ColumnDef = native(NativeType::UInt16);
    static K: ColumnDef = native(NativeType::Int32);
    static PSID: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static DROP_ATTR: ColumnDef = native(NativeType::UInt32);
    static DROP_EVT: ColumnDef = native(NativeType::UInt32);
    static DROP_LNK: ColumnDef = native(NativeType::UInt32);
    static SURL: ColumnDef = native(NativeType::Utf8);
    static TS: ColumnDef = native(NativeType::Utf8);
    static RES: ColumnDef = native(NativeType::Struct);
    static SC: ColumnDef = native(NativeType::Struct);
    static STAT: ColumnDef = native(NativeType::Struct);
    match name {
        // Required
        START_TIME_UNIX_NANO => Some(&START),
        DURATION_TIME_UNIX_NANO => Some(&DUR),
        TRACE_ID => Some(&TID),
        SPAN_ID => Some(&SID),
        NAME => Some(&N),
        // Optional
        ID => Some(&SPAN_ID_COL),
        KIND => Some(&K),
        PARENT_SPAN_ID => Some(&PSID),
        DROPPED_ATTRIBUTES_COUNT => Some(&DROP_ATTR),
        DROPPED_EVENTS_COUNT => Some(&DROP_EVT),
        DROPPED_LINKS_COUNT => Some(&DROP_LNK),
        SCHEMA_URL => Some(&SURL),
        TRACE_STATE => Some(&TS),
        RESOURCE => Some(&RES),
        SCOPE => Some(&SC),
        STATUS => Some(&STAT),
        _ => None,
    }
}

fn spans_get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
    static RES_ID: ColumnDef = native(NativeType::UInt16);
    static RES_DROP: ColumnDef = native(NativeType::UInt32);
    static RES_SURL: ColumnDef = dict(NativeType::Utf8, U8);
    static SC_ID: ColumnDef = native(NativeType::UInt16);
    static SC_DROP: ColumnDef = native(NativeType::UInt32);
    static SC_NAME: ColumnDef = native(NativeType::Utf8);
    static SC_VER: ColumnDef = native(NativeType::Utf8);
    static STAT_CODE: ColumnDef = dict(NativeType::Int32, U8);
    static STAT_MSG: ColumnDef = dict(NativeType::Utf8, U8);
    match (parent, child) {
        // Resource sub-fields
        (RESOURCE, ID) => Some(&RES_ID),
        (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RES_DROP),
        (RESOURCE, SCHEMA_URL) => Some(&RES_SURL),
        // Scope sub-fields
        (SCOPE, ID) => Some(&SC_ID),
        (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SC_DROP),
        (SCOPE, NAME) => Some(&SC_NAME),
        (SCOPE, VERSION) => Some(&SC_VER),
        // Status sub-fields
        (STATUS, STATUS_CODE) => Some(&STAT_CODE),
        (STATUS, STATUS_MESSAGE) => Some(&STAT_MSG),
        _ => None,
    }
}

static SPANS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: spans_get,
    get_nested_fn: spans_get_nested,
};

// ---------------------------------------------------------------------------
// SPAN_EVENTS - Spec: 5.2.2
// ---------------------------------------------------------------------------

fn span_events_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static N: ColumnDef = native(NativeType::Utf8);
    static EVT_ID: ColumnDef = native(NativeType::UInt32);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static DROP_ATTR: ColumnDef = native(NativeType::UInt32);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        NAME => Some(&N),
        // Optional
        ID => Some(&EVT_ID),
        TIME_UNIX_NANO => Some(&TIME),
        DROPPED_ATTRIBUTES_COUNT => Some(&DROP_ATTR),
        _ => None,
    }
}

static SPAN_EVENTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: span_events_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// SPAN_LINKS - Spec: 5.2.3
// ---------------------------------------------------------------------------

fn span_links_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static LNK_ID: ColumnDef = native(NativeType::UInt32);
    static SID: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static TID: ColumnDef = native(NativeType::FixedSizeBinary(16));
    static TS: ColumnDef = native(NativeType::Utf8);
    static DROP_ATTR: ColumnDef = native(NativeType::UInt32);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        // Optional
        ID => Some(&LNK_ID),
        SPAN_ID => Some(&SID),
        TRACE_ID => Some(&TID),
        TRACE_STATE => Some(&TS),
        DROPPED_ATTRIBUTES_COUNT => Some(&DROP_ATTR),
        _ => None,
    }
}

static SPAN_LINKS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: span_links_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// UNIVARIATE_METRICS (ROOT) - Spec: 5.3.1
// ---------------------------------------------------------------------------

fn univariate_metrics_get(name: &str) -> Option<&'static ColumnDef> {
    static MET_ID: ColumnDef = native(NativeType::UInt16);
    static MT: ColumnDef = native(NativeType::UInt8);
    static N: ColumnDef = native(NativeType::Utf8);
    static AGG: ColumnDef = native(NativeType::Int32);
    static DESC: ColumnDef = native(NativeType::Utf8);
    static MONO: ColumnDef = native(NativeType::Boolean);
    static U: ColumnDef = native(NativeType::Utf8);
    static SURL: ColumnDef = native(NativeType::Utf8);
    static RES: ColumnDef = native(NativeType::Struct);
    static SC: ColumnDef = native(NativeType::Struct);
    match name {
        // Required
        ID => Some(&MET_ID),
        METRIC_TYPE => Some(&MT),
        NAME => Some(&N),
        // Optional
        AGGREGATION_TEMPORALITY => Some(&AGG),
        DESCRIPTION => Some(&DESC),
        IS_MONOTONIC => Some(&MONO),
        UNIT => Some(&U),
        SCHEMA_URL => Some(&SURL),
        RESOURCE => Some(&RES),
        SCOPE => Some(&SC),
        _ => None,
    }
}

fn univariate_metrics_get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
    static RES_ID: ColumnDef = native(NativeType::UInt16);
    static RES_DROP: ColumnDef = native(NativeType::UInt32);
    static RES_SURL: ColumnDef = native(NativeType::Utf8);
    static SC_ID: ColumnDef = native(NativeType::UInt16);
    static SC_DROP: ColumnDef = native(NativeType::UInt32);
    static SC_NAME: ColumnDef = native(NativeType::Utf8);
    static SC_VER: ColumnDef = native(NativeType::Utf8);
    match (parent, child) {
        // Resource sub-fields
        (RESOURCE, ID) => Some(&RES_ID),
        (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RES_DROP),
        (RESOURCE, SCHEMA_URL) => Some(&RES_SURL),
        // Scope sub-fields
        (SCOPE, ID) => Some(&SC_ID),
        (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SC_DROP),
        (SCOPE, NAME) => Some(&SC_NAME),
        (SCOPE, VERSION) => Some(&SC_VER),
        _ => None,
    }
}

static UNIVARIATE_METRICS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: univariate_metrics_get,
    get_nested_fn: univariate_metrics_get_nested,
};

// ---------------------------------------------------------------------------
// NUMBER_DATA_POINTS - Spec: 5.3.2
// ---------------------------------------------------------------------------

fn number_data_points_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static START: ColumnDef = native(NativeType::TimestampNs);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static IV: ColumnDef = native(NativeType::Int64);
    static DV: ColumnDef = native(NativeType::Float64);
    static DP_ID: ColumnDef = native(NativeType::UInt32);
    static DP_FLAGS: ColumnDef = native(NativeType::UInt32);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        START_TIME_UNIX_NANO => Some(&START),
        TIME_UNIX_NANO => Some(&TIME),
        INT_VALUE => Some(&IV),
        DOUBLE_VALUE => Some(&DV),
        // Optional
        ID => Some(&DP_ID),
        FLAGS => Some(&DP_FLAGS),
        _ => None,
    }
}

static NUMBER_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: number_data_points_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// SUMMARY_DATA_POINTS - Spec: 5.3.3
// ---------------------------------------------------------------------------

fn summary_data_points_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static DP_ID: ColumnDef = native(NativeType::UInt32);
    static CNT: ColumnDef = native(NativeType::UInt64);
    static S: ColumnDef = native(NativeType::Float64);
    static START: ColumnDef = native(NativeType::TimestampNs);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static DP_FLAGS: ColumnDef = native(NativeType::UInt32);
    static QUANT: ColumnDef = native(NativeType::ListStruct);
    static VAL: ColumnDef = native(NativeType::ListFloat64);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        // Optional
        ID => Some(&DP_ID),
        SUMMARY_COUNT => Some(&CNT),
        SUMMARY_SUM => Some(&S),
        START_TIME_UNIX_NANO => Some(&START),
        TIME_UNIX_NANO => Some(&TIME),
        FLAGS => Some(&DP_FLAGS),
        SUMMARY_QUANTILE_VALUES => Some(&QUANT),
        METRIC_VALUE => Some(&VAL),
        _ => None,
    }
}

fn summary_data_points_get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
    static QQ: ColumnDef = native(NativeType::Float64);
    static QV: ColumnDef = native(NativeType::Float64);
    match (parent, child) {
        (SUMMARY_QUANTILE, SUMMARY_QUANTILE) => Some(&QQ),
        (SUMMARY_QUANTILE, SUMMARY_VALUE) => Some(&QV),
        _ => None,
    }
}

static SUMMARY_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: summary_data_points_get,
    get_nested_fn: summary_data_points_get_nested,
};

// ---------------------------------------------------------------------------
// HISTOGRAM_DATA_POINTS - Spec: 5.3.4
// ---------------------------------------------------------------------------

fn histogram_data_points_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static DP_ID: ColumnDef = native(NativeType::UInt32);
    static CNT: ColumnDef = native(NativeType::UInt64);
    static S: ColumnDef = native(NativeType::Float64);
    static MN: ColumnDef = native(NativeType::Float64);
    static MX: ColumnDef = native(NativeType::Float64);
    static BC: ColumnDef = native(NativeType::ListUInt64);
    static EB: ColumnDef = native(NativeType::ListFloat64);
    static START: ColumnDef = native(NativeType::TimestampNs);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static DP_FLAGS: ColumnDef = native(NativeType::UInt32);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        // Optional
        ID => Some(&DP_ID),
        HISTOGRAM_COUNT => Some(&CNT),
        HISTOGRAM_SUM => Some(&S),
        HISTOGRAM_MIN => Some(&MN),
        HISTOGRAM_MAX => Some(&MX),
        HISTOGRAM_BUCKET_COUNTS => Some(&BC),
        HISTOGRAM_EXPLICIT_BOUNDS => Some(&EB),
        START_TIME_UNIX_NANO => Some(&START),
        TIME_UNIX_NANO => Some(&TIME),
        FLAGS => Some(&DP_FLAGS),
        _ => None,
    }
}

static HISTOGRAM_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: histogram_data_points_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// EXP_HISTOGRAM_DATA_POINTS - Spec: 5.3.5
// ---------------------------------------------------------------------------

fn exp_histogram_data_points_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = native(NativeType::UInt16);
    static DP_ID: ColumnDef = native(NativeType::UInt32);
    static CNT: ColumnDef = native(NativeType::UInt64);
    static S: ColumnDef = native(NativeType::Float64);
    static MN: ColumnDef = native(NativeType::Float64);
    static MX: ColumnDef = native(NativeType::Float64);
    static SC: ColumnDef = native(NativeType::Int32);
    static ZC: ColumnDef = native(NativeType::UInt64);
    static POS: ColumnDef = native(NativeType::Struct);
    static NEG: ColumnDef = native(NativeType::Struct);
    static START: ColumnDef = native(NativeType::TimestampNs);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static DP_FLAGS: ColumnDef = native(NativeType::UInt32);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        // Optional
        ID => Some(&DP_ID),
        HISTOGRAM_COUNT => Some(&CNT),
        HISTOGRAM_SUM => Some(&S),
        HISTOGRAM_MIN => Some(&MN),
        HISTOGRAM_MAX => Some(&MX),
        EXP_HISTOGRAM_SCALE => Some(&SC),
        EXP_HISTOGRAM_ZERO_COUNT => Some(&ZC),
        EXP_HISTOGRAM_POSITIVE => Some(&POS),
        EXP_HISTOGRAM_NEGATIVE => Some(&NEG),
        START_TIME_UNIX_NANO => Some(&START),
        TIME_UNIX_NANO => Some(&TIME),
        FLAGS => Some(&DP_FLAGS),
        _ => None,
    }
}

fn exp_histogram_data_points_get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
    static BC: ColumnDef = native(NativeType::ListUInt64);
    static OFF: ColumnDef = native(NativeType::Int32);
    match (parent, child) {
        (EXP_HISTOGRAM_POSITIVE, EXP_HISTOGRAM_BUCKET_COUNTS) => Some(&BC),
        (EXP_HISTOGRAM_POSITIVE, EXP_HISTOGRAM_OFFSET) => Some(&OFF),
        (EXP_HISTOGRAM_NEGATIVE, EXP_HISTOGRAM_BUCKET_COUNTS) => Some(&BC),
        (EXP_HISTOGRAM_NEGATIVE, EXP_HISTOGRAM_OFFSET) => Some(&OFF),
        _ => None,
    }
}

static EXP_HISTOGRAM_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: exp_histogram_data_points_get,
    get_nested_fn: exp_histogram_data_points_get_nested,
};

// ---------------------------------------------------------------------------
// EXEMPLARS (NUMBER_DP_EXEMPLARS, HISTOGRAM_DP_EXEMPLARS,
//            EXP_HISTOGRAM_DP_EXEMPLARS) - Spec: 5.3.6
// ---------------------------------------------------------------------------

fn exemplars_get(name: &str) -> Option<&'static ColumnDef> {
    static PID: ColumnDef = dict(NativeType::UInt32, U8);
    static EX_ID: ColumnDef = native(NativeType::UInt32);
    static DV: ColumnDef = native(NativeType::Float64);
    static IV: ColumnDef = dict(NativeType::Int64, U8);
    static SID: ColumnDef = dict(NativeType::FixedSizeBinary(8), U8);
    static TIME: ColumnDef = native(NativeType::TimestampNs);
    static TID: ColumnDef = dict(NativeType::FixedSizeBinary(16), U8);
    match name {
        // Required
        PARENT_ID => Some(&PID),
        // Optional
        ID => Some(&EX_ID),
        DOUBLE_VALUE => Some(&DV),
        INT_VALUE => Some(&IV),
        SPAN_ID => Some(&SID),
        TIME_UNIX_NANO => Some(&TIME),
        TRACE_ID => Some(&TID),
        _ => None,
    }
}

static EXEMPLARS_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: exemplars_get,
    get_nested_fn: |_, _| None,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_top_level_column() {
        let def = get_definition(ArrowPayloadType::LogAttrs);

        let str_col = def.get("str").unwrap();
        assert_eq!(str_col.native_type, NativeType::Utf8);
        assert_eq!(str_col.min_dict_key_size, Some(MinDictKeySize::U16));

        let key_col = def.get("key").unwrap();
        assert_eq!(key_col.native_type, NativeType::Utf8);
        assert_eq!(key_col.min_dict_key_size, Some(MinDictKeySize::U8));

        let type_col = def.get("type").unwrap();
        assert_eq!(type_col.native_type, NativeType::UInt8);
        assert_eq!(type_col.min_dict_key_size, None);

        assert!(def.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_nested_column() {
        let def = get_definition(ArrowPayloadType::Logs);

        let body_str = def.get_nested("body", "str").unwrap();
        assert_eq!(body_str.native_type, NativeType::Utf8);
        assert_eq!(body_str.min_dict_key_size, Some(MinDictKeySize::U16));

        let body_int = def.get_nested("body", "int").unwrap();
        assert_eq!(body_int.native_type, NativeType::Int64);
        assert_eq!(body_int.min_dict_key_size, Some(MinDictKeySize::U16));

        let resource_schema = def.get_nested("resource", "schema_url").unwrap();
        assert_eq!(resource_schema.native_type, NativeType::Utf8);
        assert_eq!(resource_schema.min_dict_key_size, Some(MinDictKeySize::U8));

        assert!(def.get_nested("body", "nonexistent").is_none());
        assert!(def.get_nested("nonexistent", "str").is_none());
    }

    #[test]
    fn test_u16_attrs_value_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::ResourceAttrs);
        for col_name in &["str", "int", "bytes", "ser"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_name,
            );
        }
    }

    #[test]
    fn test_u32_attrs_value_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::SpanEventAttrs);
        for col_name in &["str", "int", "bytes", "ser"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_name,
            );
        }
    }

    #[test]
    fn test_logs_body_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::Logs);
        for col_path in &["body.str", "body.int", "body.bytes", "body.ser"] {
            let parts: Vec<&str> = col_path.split('.').collect();
            let col = def
                .get_nested(parts[0], parts[1])
                .unwrap_or_else(|| panic!("missing column: {}", col_path));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_path,
            );
        }
    }

    #[test]
    fn test_exemplars_dict_columns() {
        let def = get_definition(ArrowPayloadType::NumberDpExemplars);
        for col_name in &["parent_id", "int_value", "span_id", "trace_id"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U8),
                "exemplar column {} should allow Dict(u8)",
                col_name,
            );
        }
    }

    #[test]
    fn test_empty_definition() {
        let def = get_definition(ArrowPayloadType::Unknown);
        assert!(def.get("anything").is_none());
        assert!(def.get_nested("any", "thing").is_none());
    }

    #[test]
    fn test_all_payload_types_return_definition() {
        let all_types = [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ];

        for typ in all_types {
            // Should not panic
            let _ = get_definition(typ);
        }
    }

    #[test]
    fn test_unknown_columns_return_none() {
        let all_types = [
            ArrowPayloadType::Logs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::SpanEventAttrs,
        ];

        for typ in all_types {
            let def = get_definition(typ);
            assert!(
                def.get("nonexistent_column_xyz").is_none(),
                "{:?} should return None for unknown column",
                typ,
            );
            assert!(
                def.get_nested("nonexistent", "column").is_none(),
                "{:?} should return None for unknown nested column",
                typ,
            );
        }
    }

    #[test]
    fn test_spans_nested_columns() {
        let def = get_definition(ArrowPayloadType::Spans);

        let status_code = def.get_nested("status", "code").unwrap();
        assert_eq!(status_code.native_type, NativeType::Int32);
        assert_eq!(status_code.min_dict_key_size, Some(MinDictKeySize::U8));

        let status_msg = def.get_nested("status", "status_message").unwrap();
        assert_eq!(status_msg.native_type, NativeType::Utf8);
        assert_eq!(status_msg.min_dict_key_size, Some(MinDictKeySize::U8));

        let scope_name = def.get_nested("scope", "name").unwrap();
        assert_eq!(scope_name.native_type, NativeType::Utf8);
        assert_eq!(scope_name.min_dict_key_size, None);
    }

    #[test]
    fn test_exp_histogram_nested_columns() {
        let def = get_definition(ArrowPayloadType::ExpHistogramDataPoints);

        // Top-level structs
        let pos = def.get("positive").unwrap();
        assert_eq!(pos.native_type, NativeType::Struct);
        let neg = def.get("negative").unwrap();
        assert_eq!(neg.native_type, NativeType::Struct);

        // Nested sub-fields
        let pbc = def.get_nested("positive", "bucket_counts").unwrap();
        assert_eq!(pbc.native_type, NativeType::ListUInt64);
        let po = def.get_nested("positive", "offset").unwrap();
        assert_eq!(po.native_type, NativeType::Int32);
        let nbc = def.get_nested("negative", "bucket_counts").unwrap();
        assert_eq!(nbc.native_type, NativeType::ListUInt64);
        let no = def.get_nested("negative", "offset").unwrap();
        assert_eq!(no.native_type, NativeType::Int32);

        assert!(def.get_nested("positive", "nonexistent").is_none());
    }

    #[test]
    fn test_summary_nested_columns() {
        let def = get_definition(ArrowPayloadType::SummaryDataPoints);

        let qq = def.get_nested("quantile", "quantile").unwrap();
        assert_eq!(qq.native_type, NativeType::Float64);

        let qv = def.get_nested("quantile", "value").unwrap();
        assert_eq!(qv.native_type, NativeType::Float64);
    }
}
