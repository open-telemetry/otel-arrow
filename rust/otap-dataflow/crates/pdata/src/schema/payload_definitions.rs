// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload type definitions from the OTAP spec.
//!
//! Each payload type has a definition that describes the columns it contains,
//! their native Arrow types, and dictionary encoding constraints.
//!
//! In the future we can improve definitions to:
//!
//! - Include which columns are optional or not
//! - Add iterators over all the columns and their definitions per payload
//! - Be automatically generated from some common spec definition

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

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
pub enum DictKeySize {
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
    pub min_dict_key_size: Option<DictKeySize>,
}

/// Schema definition for a payload type.
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

/// Get the payload definition for a given payload type.
#[must_use]
pub fn get(typ: ArrowPayloadType) -> &'static PayloadDefinition {
    match typ {
        ArrowPayloadType::Unknown => &EMPTY_DEFINITION,

        // Logs
        ArrowPayloadType::Logs => &logs::DEFINITION,
        ArrowPayloadType::LogAttrs => &attributes_16::DEFINITION,

        // Traces
        ArrowPayloadType::Spans => &spans::DEFINITION,
        ArrowPayloadType::SpanAttrs => &attributes_16::DEFINITION,
        ArrowPayloadType::SpanEvents => &span_events::DEFINITION,
        ArrowPayloadType::SpanLinks => &span_links::DEFINITION,
        ArrowPayloadType::SpanEventAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::SpanLinkAttrs => &attributes_32::DEFINITION,

        // Metrics
        ArrowPayloadType::UnivariateMetrics => &univariate_metrics::DEFINITION,
        ArrowPayloadType::MultivariateMetrics => &EMPTY_DEFINITION,
        ArrowPayloadType::NumberDataPoints => &number_data_points::DEFINITION,
        ArrowPayloadType::SummaryDataPoints => &summary_data_points::DEFINITION,
        ArrowPayloadType::HistogramDataPoints => &histogram_data_points::DEFINITION,
        ArrowPayloadType::ExpHistogramDataPoints => &exp_histogram_data_points::DEFINITION,
        ArrowPayloadType::NumberDpExemplars => &exemplars::DEFINITION,
        ArrowPayloadType::HistogramDpExemplars => &exemplars::DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplars => &exemplars::DEFINITION,

        // Metric attributes
        ArrowPayloadType::MetricAttrs => &attributes_16::DEFINITION,
        ArrowPayloadType::NumberDpAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::SummaryDpAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::HistogramDpAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::ExpHistogramDpAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::NumberDpExemplarAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::HistogramDpExemplarAttrs => &attributes_32::DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs => &attributes_32::DEFINITION,

        // Common
        ArrowPayloadType::ResourceAttrs => &attributes_16::DEFINITION,
        ArrowPayloadType::ScopeAttrs => &attributes_16::DEFINITION,
    }
}

const fn native(native_type: NativeType) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: None,
    }
}

const fn dict(native_type: NativeType, min: DictKeySize) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: Some(min),
    }
}

static EMPTY_DEFINITION: PayloadDefinition = PayloadDefinition {
    get_fn: |_| None,
    get_nested_fn: |_, _| None,
};

mod attributes_16 {
    use super::{ColumnDef, DictKeySize, NativeType, PayloadDefinition, dict, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static KEY_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static TYPE_DEF: ColumnDef = native(NativeType::UInt8);
    static STR_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U16);
    static INT_DEF: ColumnDef = dict(NativeType::Int64, DictKeySize::U16);
    static DOUBLE_DEF: ColumnDef = native(NativeType::Float64);
    static BOOL_DEF: ColumnDef = native(NativeType::Boolean);
    static BYTES_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);
    static SER_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            ATTRIBUTE_KEY => Some(&KEY_DEF),
            ATTRIBUTE_TYPE => Some(&TYPE_DEF),
            // Optional
            ATTRIBUTE_STR => Some(&STR_DEF),
            ATTRIBUTE_INT => Some(&INT_DEF),
            ATTRIBUTE_DOUBLE => Some(&DOUBLE_DEF),
            ATTRIBUTE_BOOL => Some(&BOOL_DEF),
            ATTRIBUTE_BYTES => Some(&BYTES_DEF),
            ATTRIBUTE_SER => Some(&SER_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod attributes_32 {
    use super::{ColumnDef, DictKeySize, NativeType, PayloadDefinition, dict, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = dict(NativeType::UInt32, DictKeySize::U8);
    static KEY_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static TYPE_DEF: ColumnDef = native(NativeType::UInt8);
    static STR_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U16);
    static INT_DEF: ColumnDef = dict(NativeType::Int64, DictKeySize::U16);
    static DOUBLE_DEF: ColumnDef = native(NativeType::Float64);
    static BOOL_DEF: ColumnDef = native(NativeType::Boolean);
    static BYTES_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);
    static SER_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            ATTRIBUTE_KEY => Some(&KEY_DEF),
            ATTRIBUTE_TYPE => Some(&TYPE_DEF),
            // Optional
            ATTRIBUTE_STR => Some(&STR_DEF),
            ATTRIBUTE_INT => Some(&INT_DEF),
            ATTRIBUTE_DOUBLE => Some(&DOUBLE_DEF),
            ATTRIBUTE_BOOL => Some(&BOOL_DEF),
            ATTRIBUTE_BYTES => Some(&BYTES_DEF),
            ATTRIBUTE_SER => Some(&SER_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod logs {
    use super::{ColumnDef, DictKeySize, NativeType, PayloadDefinition, dict, native};
    use crate::schema::consts::*;

    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static OBSERVED_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static BODY_DEF: ColumnDef = native(NativeType::Struct);
    static ID_DEF: ColumnDef = native(NativeType::UInt16);
    static SEVERITY_NUMBER_DEF: ColumnDef = dict(NativeType::Int32, DictKeySize::U8);
    static SEVERITY_TEXT_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static EVENT_NAME_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static FLAGS_DEF: ColumnDef = native(NativeType::UInt32);
    static TRACE_ID_DEF: ColumnDef = dict(NativeType::FixedSizeBinary(16), DictKeySize::U8);
    static SPAN_ID_DEF: ColumnDef = dict(NativeType::FixedSizeBinary(8), DictKeySize::U8);
    static SCHEMA_URL_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static RESOURCE_DEF: ColumnDef = native(NativeType::Struct);
    static SCOPE_DEF: ColumnDef = native(NativeType::Struct);

    // Nested: body sub-fields
    static BODY_TYPE_DEF: ColumnDef = native(NativeType::UInt8);
    static BODY_STR_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U16);
    static BODY_INT_DEF: ColumnDef = dict(NativeType::Int64, DictKeySize::U16);
    static BODY_DOUBLE_DEF: ColumnDef = native(NativeType::Float64);
    static BODY_BOOL_DEF: ColumnDef = native(NativeType::Boolean);
    static BODY_BYTES_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);
    static BODY_SER_DEF: ColumnDef = dict(NativeType::Binary, DictKeySize::U16);

    // Nested: resource sub-fields
    static RESOURCE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static RESOURCE_SCHEMA_URL_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);

    // Nested: scope sub-fields
    static SCOPE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static SCOPE_NAME_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);
    static SCOPE_VERSION_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            OBSERVED_TIME_UNIX_NANO => Some(&OBSERVED_TIME_UNIX_NANO_DEF),
            BODY => Some(&BODY_DEF),
            // Optional
            ID => Some(&ID_DEF),
            SEVERITY_NUMBER => Some(&SEVERITY_NUMBER_DEF),
            SEVERITY_TEXT => Some(&SEVERITY_TEXT_DEF),
            DROPPED_ATTRIBUTES_COUNT => Some(&DROPPED_ATTRIBUTES_COUNT_DEF),
            EVENT_NAME => Some(&EVENT_NAME_DEF),
            FLAGS => Some(&FLAGS_DEF),
            TRACE_ID => Some(&TRACE_ID_DEF),
            SPAN_ID => Some(&SPAN_ID_DEF),
            SCHEMA_URL => Some(&SCHEMA_URL_DEF),
            RESOURCE => Some(&RESOURCE_DEF),
            SCOPE => Some(&SCOPE_DEF),
            // Dotted paths for nested columns
            BODY_TYPE => Some(&BODY_TYPE_DEF),
            BODY_STR => Some(&BODY_STR_DEF),
            BODY_INT => Some(&BODY_INT_DEF),
            BODY_DOUBLE => Some(&BODY_DOUBLE_DEF),
            BODY_BOOL => Some(&BODY_BOOL_DEF),
            BODY_BYTES => Some(&BODY_BYTES_DEF),
            BODY_SER => Some(&BODY_SER_DEF),
            RESOURCE_ID => Some(&RESOURCE_ID_DEF),
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            RESOURCE_SCHEMA_URL => Some(&RESOURCE_SCHEMA_URL_DEF),
            SCOPE_ID => Some(&SCOPE_ID_DEF),
            SCOPE_DROPPED_ATTRIBUTES_COUNT => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            SCOPE_NAME => Some(&SCOPE_NAME_DEF),
            SCOPE_VERSION => Some(&SCOPE_VERSION_DEF),
            _ => None,
        }
    }

    fn get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
        match (parent, child) {
            // Required (body.type, body.str are required sub-fields of body)
            (BODY, ATTRIBUTE_TYPE) => Some(&BODY_TYPE_DEF),
            (BODY, ATTRIBUTE_STR) => Some(&BODY_STR_DEF),
            // Optional body sub-fields
            (BODY, ATTRIBUTE_INT) => Some(&BODY_INT_DEF),
            (BODY, ATTRIBUTE_DOUBLE) => Some(&BODY_DOUBLE_DEF),
            (BODY, ATTRIBUTE_BOOL) => Some(&BODY_BOOL_DEF),
            (BODY, ATTRIBUTE_BYTES) => Some(&BODY_BYTES_DEF),
            (BODY, ATTRIBUTE_SER) => Some(&BODY_SER_DEF),
            // Resource sub-fields
            (RESOURCE, ID) => Some(&RESOURCE_ID_DEF),
            (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (RESOURCE, SCHEMA_URL) => Some(&RESOURCE_SCHEMA_URL_DEF),
            // Scope sub-fields
            (SCOPE, ID) => Some(&SCOPE_ID_DEF),
            (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (SCOPE, NAME) => Some(&SCOPE_NAME_DEF),
            (SCOPE, VERSION) => Some(&SCOPE_VERSION_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: get_nested,
    };
}

mod spans {
    use super::{ColumnDef, DictKeySize, NativeType, PayloadDefinition, dict, native};
    use crate::schema::consts::*;

    static START_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static DURATION_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::DurationNs);
    static TRACE_ID_DEF: ColumnDef = native(NativeType::FixedSizeBinary(16));
    static SPAN_ID_DEF: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static NAME_DEF: ColumnDef = native(NativeType::Utf8);
    static ID_DEF: ColumnDef = native(NativeType::UInt16);
    static KIND_DEF: ColumnDef = native(NativeType::Int32);
    static PARENT_SPAN_ID_DEF: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static DROPPED_EVENTS_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static DROPPED_LINKS_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static SCHEMA_URL_DEF: ColumnDef = native(NativeType::Utf8);
    static TRACE_STATE_DEF: ColumnDef = native(NativeType::Utf8);
    static RESOURCE_DEF: ColumnDef = native(NativeType::Struct);
    static SCOPE_DEF: ColumnDef = native(NativeType::Struct);
    static STATUS_DEF: ColumnDef = native(NativeType::Struct);

    // Nested: resource sub-fields
    static RESOURCE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static RESOURCE_SCHEMA_URL_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);

    // Nested: scope sub-fields
    static SCOPE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static SCOPE_NAME_DEF: ColumnDef = native(NativeType::Utf8);
    static SCOPE_VERSION_DEF: ColumnDef = native(NativeType::Utf8);

    // Nested: status sub-fields
    static STATUS_CODE_DEF: ColumnDef = dict(NativeType::Int32, DictKeySize::U8);
    static STATUS_MESSAGE_DEF: ColumnDef = dict(NativeType::Utf8, DictKeySize::U8);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            START_TIME_UNIX_NANO => Some(&START_TIME_UNIX_NANO_DEF),
            DURATION_TIME_UNIX_NANO => Some(&DURATION_TIME_UNIX_NANO_DEF),
            TRACE_ID => Some(&TRACE_ID_DEF),
            SPAN_ID => Some(&SPAN_ID_DEF),
            NAME => Some(&NAME_DEF),
            // Optional
            ID => Some(&ID_DEF),
            KIND => Some(&KIND_DEF),
            PARENT_SPAN_ID => Some(&PARENT_SPAN_ID_DEF),
            DROPPED_ATTRIBUTES_COUNT => Some(&DROPPED_ATTRIBUTES_COUNT_DEF),
            DROPPED_EVENTS_COUNT => Some(&DROPPED_EVENTS_COUNT_DEF),
            DROPPED_LINKS_COUNT => Some(&DROPPED_LINKS_COUNT_DEF),
            SCHEMA_URL => Some(&SCHEMA_URL_DEF),
            TRACE_STATE => Some(&TRACE_STATE_DEF),
            RESOURCE => Some(&RESOURCE_DEF),
            SCOPE => Some(&SCOPE_DEF),
            STATUS => Some(&STATUS_DEF),
            // Dotted paths for nested columns
            RESOURCE_ID => Some(&RESOURCE_ID_DEF),
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            RESOURCE_SCHEMA_URL => Some(&RESOURCE_SCHEMA_URL_DEF),
            SCOPE_ID => Some(&SCOPE_ID_DEF),
            SCOPE_DROPPED_ATTRIBUTES_COUNT => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            SCOPE_NAME => Some(&SCOPE_NAME_DEF),
            SCOPE_VERSION => Some(&SCOPE_VERSION_DEF),
            STATUS_DOT_CODE => Some(&STATUS_CODE_DEF),
            STATUS_DOT_STATUS_MESSAGE => Some(&STATUS_MESSAGE_DEF),
            _ => None,
        }
    }

    fn get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
        match (parent, child) {
            // Resource sub-fields
            (RESOURCE, ID) => Some(&RESOURCE_ID_DEF),
            (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (RESOURCE, SCHEMA_URL) => Some(&RESOURCE_SCHEMA_URL_DEF),
            // Scope sub-fields
            (SCOPE, ID) => Some(&SCOPE_ID_DEF),
            (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (SCOPE, NAME) => Some(&SCOPE_NAME_DEF),
            (SCOPE, VERSION) => Some(&SCOPE_VERSION_DEF),
            // Status sub-fields
            (STATUS, STATUS_CODE) => Some(&STATUS_CODE_DEF),
            (STATUS, STATUS_MESSAGE) => Some(&STATUS_MESSAGE_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: get_nested,
    };
}

mod span_events {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static NAME_DEF: ColumnDef = native(NativeType::Utf8);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            NAME => Some(&NAME_DEF),
            // Optional
            ID => Some(&ID_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            DROPPED_ATTRIBUTES_COUNT => Some(&DROPPED_ATTRIBUTES_COUNT_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod span_links {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static SPAN_ID_DEF: ColumnDef = native(NativeType::FixedSizeBinary(8));
    static TRACE_ID_DEF: ColumnDef = native(NativeType::FixedSizeBinary(16));
    static TRACE_STATE_DEF: ColumnDef = native(NativeType::Utf8);
    static DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            // Optional
            ID => Some(&ID_DEF),
            SPAN_ID => Some(&SPAN_ID_DEF),
            TRACE_ID => Some(&TRACE_ID_DEF),
            TRACE_STATE => Some(&TRACE_STATE_DEF),
            DROPPED_ATTRIBUTES_COUNT => Some(&DROPPED_ATTRIBUTES_COUNT_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod univariate_metrics {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static ID_DEF: ColumnDef = native(NativeType::UInt16);
    static METRIC_TYPE_DEF: ColumnDef = native(NativeType::UInt8);
    static NAME_DEF: ColumnDef = native(NativeType::Utf8);
    static AGGREGATION_TEMPORALITY_DEF: ColumnDef = native(NativeType::Int32);
    static DESCRIPTION_DEF: ColumnDef = native(NativeType::Utf8);
    static IS_MONOTONIC_DEF: ColumnDef = native(NativeType::Boolean);
    static UNIT_DEF: ColumnDef = native(NativeType::Utf8);
    static SCHEMA_URL_DEF: ColumnDef = native(NativeType::Utf8);
    static RESOURCE_DEF: ColumnDef = native(NativeType::Struct);
    static SCOPE_DEF: ColumnDef = native(NativeType::Struct);

    // Nested: resource sub-fields
    static RESOURCE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static RESOURCE_SCHEMA_URL_DEF: ColumnDef = native(NativeType::Utf8);

    // Nested: scope sub-fields
    static SCOPE_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF: ColumnDef = native(NativeType::UInt32);
    static SCOPE_NAME_DEF: ColumnDef = native(NativeType::Utf8);
    static SCOPE_VERSION_DEF: ColumnDef = native(NativeType::Utf8);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            ID => Some(&ID_DEF),
            METRIC_TYPE => Some(&METRIC_TYPE_DEF),
            NAME => Some(&NAME_DEF),
            // Optional
            AGGREGATION_TEMPORALITY => Some(&AGGREGATION_TEMPORALITY_DEF),
            DESCRIPTION => Some(&DESCRIPTION_DEF),
            IS_MONOTONIC => Some(&IS_MONOTONIC_DEF),
            UNIT => Some(&UNIT_DEF),
            SCHEMA_URL => Some(&SCHEMA_URL_DEF),
            RESOURCE => Some(&RESOURCE_DEF),
            SCOPE => Some(&SCOPE_DEF),
            // Dotted paths for nested columns
            RESOURCE_ID => Some(&RESOURCE_ID_DEF),
            RESOURCE_DROPPED_ATTRIBUTES_COUNT => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            RESOURCE_SCHEMA_URL => Some(&RESOURCE_SCHEMA_URL_DEF),
            SCOPE_ID => Some(&SCOPE_ID_DEF),
            SCOPE_DROPPED_ATTRIBUTES_COUNT => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            SCOPE_NAME => Some(&SCOPE_NAME_DEF),
            SCOPE_VERSION => Some(&SCOPE_VERSION_DEF),
            _ => None,
        }
    }

    fn get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
        match (parent, child) {
            // Resource sub-fields
            (RESOURCE, ID) => Some(&RESOURCE_ID_DEF),
            (RESOURCE, DROPPED_ATTRIBUTES_COUNT) => Some(&RESOURCE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (RESOURCE, SCHEMA_URL) => Some(&RESOURCE_SCHEMA_URL_DEF),
            // Scope sub-fields
            (SCOPE, ID) => Some(&SCOPE_ID_DEF),
            (SCOPE, DROPPED_ATTRIBUTES_COUNT) => Some(&SCOPE_DROPPED_ATTRIBUTES_COUNT_DEF),
            (SCOPE, NAME) => Some(&SCOPE_NAME_DEF),
            (SCOPE, VERSION) => Some(&SCOPE_VERSION_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: get_nested,
    };
}

mod number_data_points {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static START_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static INT_VALUE_DEF: ColumnDef = native(NativeType::Int64);
    static DOUBLE_VALUE_DEF: ColumnDef = native(NativeType::Float64);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static FLAGS_DEF: ColumnDef = native(NativeType::UInt32);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            START_TIME_UNIX_NANO => Some(&START_TIME_UNIX_NANO_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            INT_VALUE => Some(&INT_VALUE_DEF),
            DOUBLE_VALUE => Some(&DOUBLE_VALUE_DEF),
            // Optional
            ID => Some(&ID_DEF),
            FLAGS => Some(&FLAGS_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod summary_data_points {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static COUNT_DEF: ColumnDef = native(NativeType::UInt64);
    static SUM_DEF: ColumnDef = native(NativeType::Float64);
    static START_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static FLAGS_DEF: ColumnDef = native(NativeType::UInt32);
    static QUANTILE_DEF: ColumnDef = native(NativeType::ListStruct);
    static VALUE_DEF: ColumnDef = native(NativeType::ListFloat64);

    // Nested: quantile sub-fields
    static QUANTILE_QUANTILE_DEF: ColumnDef = native(NativeType::Float64);
    static QUANTILE_VALUE_DEF: ColumnDef = native(NativeType::Float64);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            // Optional
            ID => Some(&ID_DEF),
            SUMMARY_COUNT => Some(&COUNT_DEF),
            SUMMARY_SUM => Some(&SUM_DEF),
            START_TIME_UNIX_NANO => Some(&START_TIME_UNIX_NANO_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            FLAGS => Some(&FLAGS_DEF),
            SUMMARY_QUANTILE_VALUES => Some(&QUANTILE_DEF),
            METRIC_VALUE => Some(&VALUE_DEF),
            // Dotted paths for nested columns
            QUANTILE_QUANTILE => Some(&QUANTILE_QUANTILE_DEF),
            QUANTILE_VALUE => Some(&QUANTILE_VALUE_DEF),
            _ => None,
        }
    }

    fn get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
        match (parent, child) {
            (SUMMARY_QUANTILE, SUMMARY_QUANTILE) => Some(&QUANTILE_QUANTILE_DEF),
            (SUMMARY_QUANTILE, SUMMARY_VALUE) => Some(&QUANTILE_VALUE_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: get_nested,
    };
}

mod histogram_data_points {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static COUNT_DEF: ColumnDef = native(NativeType::UInt64);
    static SUM_DEF: ColumnDef = native(NativeType::Float64);
    static MIN_DEF: ColumnDef = native(NativeType::Float64);
    static MAX_DEF: ColumnDef = native(NativeType::Float64);
    static BUCKET_COUNTS_DEF: ColumnDef = native(NativeType::ListUInt64);
    static EXPLICIT_BOUNDS_DEF: ColumnDef = native(NativeType::ListFloat64);
    static START_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static FLAGS_DEF: ColumnDef = native(NativeType::UInt32);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            // Optional
            ID => Some(&ID_DEF),
            HISTOGRAM_COUNT => Some(&COUNT_DEF),
            HISTOGRAM_SUM => Some(&SUM_DEF),
            HISTOGRAM_MIN => Some(&MIN_DEF),
            HISTOGRAM_MAX => Some(&MAX_DEF),
            HISTOGRAM_BUCKET_COUNTS => Some(&BUCKET_COUNTS_DEF),
            HISTOGRAM_EXPLICIT_BOUNDS => Some(&EXPLICIT_BOUNDS_DEF),
            START_TIME_UNIX_NANO => Some(&START_TIME_UNIX_NANO_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            FLAGS => Some(&FLAGS_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

mod exp_histogram_data_points {
    use super::{ColumnDef, NativeType, PayloadDefinition, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = native(NativeType::UInt16);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static COUNT_DEF: ColumnDef = native(NativeType::UInt64);
    static SUM_DEF: ColumnDef = native(NativeType::Float64);
    static MIN_DEF: ColumnDef = native(NativeType::Float64);
    static MAX_DEF: ColumnDef = native(NativeType::Float64);
    static SCALE_DEF: ColumnDef = native(NativeType::Int32);
    static ZERO_COUNT_DEF: ColumnDef = native(NativeType::UInt64);
    static POSITIVE_DEF: ColumnDef = native(NativeType::Struct);
    static NEGATIVE_DEF: ColumnDef = native(NativeType::Struct);
    static START_TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static FLAGS_DEF: ColumnDef = native(NativeType::UInt32);

    // Nested: positive/negative sub-fields
    static BUCKET_COUNTS_DEF: ColumnDef = native(NativeType::ListUInt64);
    static OFFSET_DEF: ColumnDef = native(NativeType::Int32);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            // Optional
            ID => Some(&ID_DEF),
            HISTOGRAM_COUNT => Some(&COUNT_DEF),
            HISTOGRAM_SUM => Some(&SUM_DEF),
            HISTOGRAM_MIN => Some(&MIN_DEF),
            HISTOGRAM_MAX => Some(&MAX_DEF),
            EXP_HISTOGRAM_SCALE => Some(&SCALE_DEF),
            EXP_HISTOGRAM_ZERO_COUNT => Some(&ZERO_COUNT_DEF),
            EXP_HISTOGRAM_POSITIVE => Some(&POSITIVE_DEF),
            EXP_HISTOGRAM_NEGATIVE => Some(&NEGATIVE_DEF),
            START_TIME_UNIX_NANO => Some(&START_TIME_UNIX_NANO_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            FLAGS => Some(&FLAGS_DEF),
            // Dotted paths for nested columns
            POSITIVE_BUCKET_COUNTS => Some(&BUCKET_COUNTS_DEF),
            POSITIVE_OFFSET => Some(&OFFSET_DEF),
            NEGATIVE_BUCKET_COUNTS => Some(&BUCKET_COUNTS_DEF),
            NEGATIVE_OFFSET => Some(&OFFSET_DEF),
            _ => None,
        }
    }

    fn get_nested(parent: &str, child: &str) -> Option<&'static ColumnDef> {
        match (parent, child) {
            (EXP_HISTOGRAM_POSITIVE, EXP_HISTOGRAM_BUCKET_COUNTS) => Some(&BUCKET_COUNTS_DEF),
            (EXP_HISTOGRAM_POSITIVE, EXP_HISTOGRAM_OFFSET) => Some(&OFFSET_DEF),
            (EXP_HISTOGRAM_NEGATIVE, EXP_HISTOGRAM_BUCKET_COUNTS) => Some(&BUCKET_COUNTS_DEF),
            (EXP_HISTOGRAM_NEGATIVE, EXP_HISTOGRAM_OFFSET) => Some(&OFFSET_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: get_nested,
    };
}

mod exemplars {
    use super::{ColumnDef, DictKeySize, NativeType, PayloadDefinition, dict, native};
    use crate::schema::consts::*;

    static PARENT_ID_DEF: ColumnDef = dict(NativeType::UInt32, DictKeySize::U8);
    static ID_DEF: ColumnDef = native(NativeType::UInt32);
    static DOUBLE_VALUE_DEF: ColumnDef = native(NativeType::Float64);
    static INT_VALUE_DEF: ColumnDef = dict(NativeType::Int64, DictKeySize::U8);
    static SPAN_ID_DEF: ColumnDef = dict(NativeType::FixedSizeBinary(8), DictKeySize::U8);
    static TIME_UNIX_NANO_DEF: ColumnDef = native(NativeType::TimestampNs);
    static TRACE_ID_DEF: ColumnDef = dict(NativeType::FixedSizeBinary(16), DictKeySize::U8);

    fn get(name: &str) -> Option<&'static ColumnDef> {
        match name {
            // Required
            PARENT_ID => Some(&PARENT_ID_DEF),
            // Optional
            ID => Some(&ID_DEF),
            DOUBLE_VALUE => Some(&DOUBLE_VALUE_DEF),
            INT_VALUE => Some(&INT_VALUE_DEF),
            SPAN_ID => Some(&SPAN_ID_DEF),
            TIME_UNIX_NANO => Some(&TIME_UNIX_NANO_DEF),
            TRACE_ID => Some(&TRACE_ID_DEF),
            _ => None,
        }
    }

    pub(super) static DEFINITION: PayloadDefinition = PayloadDefinition {
        get_fn: get,
        get_nested_fn: |_, _| None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::{Logs, Metrics, OtapBatchStore, Traces};
    use crate::schema::consts::*;

    #[test]
    fn test_get_top_level_column() {
        let def = get(ArrowPayloadType::LogAttrs);

        let str_col = def.get("str").unwrap();
        assert_eq!(str_col.native_type, NativeType::Utf8);
        assert_eq!(str_col.min_dict_key_size, Some(DictKeySize::U16));

        let key_col = def.get("key").unwrap();
        assert_eq!(key_col.native_type, NativeType::Utf8);
        assert_eq!(key_col.min_dict_key_size, Some(DictKeySize::U8));

        let type_col = def.get("type").unwrap();
        assert_eq!(type_col.native_type, NativeType::UInt8);
        assert_eq!(type_col.min_dict_key_size, None);

        assert!(def.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_nested_column() {
        let def = get(ArrowPayloadType::Logs);

        let body_str = def.get_nested("body", "str").unwrap();
        assert_eq!(body_str.native_type, NativeType::Utf8);
        assert_eq!(body_str.min_dict_key_size, Some(DictKeySize::U16));

        let body_int = def.get_nested("body", "int").unwrap();
        assert_eq!(body_int.native_type, NativeType::Int64);
        assert_eq!(body_int.min_dict_key_size, Some(DictKeySize::U16));

        let resource_schema = def.get_nested("resource", "schema_url").unwrap();
        assert_eq!(resource_schema.native_type, NativeType::Utf8);
        assert_eq!(resource_schema.min_dict_key_size, Some(DictKeySize::U8));

        assert!(def.get_nested("body", "nonexistent").is_none());
        assert!(def.get_nested("nonexistent", "str").is_none());
    }

    #[test]
    fn test_empty_definition() {
        let def = get(ArrowPayloadType::Unknown);
        assert!(def.get("anything").is_none());
        assert!(def.get_nested("any", "thing").is_none());
    }

    #[test]
    fn test_all_payload_types_return_definition() {
        let mut all_types = Vec::new();
        all_types.extend(Logs::allowed_payload_types());
        all_types.extend(Metrics::allowed_payload_types());
        all_types.extend(Traces::allowed_payload_types());

        for typ in all_types {
            let _ = get(typ);
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
            let def = get(typ);
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
        let def = get(ArrowPayloadType::Spans);

        let status_code = def.get_nested("status", "code").unwrap();
        assert_eq!(status_code.native_type, NativeType::Int32);
        assert_eq!(status_code.min_dict_key_size, Some(DictKeySize::U8));

        let status_msg = def.get_nested("status", "status_message").unwrap();
        assert_eq!(status_msg.native_type, NativeType::Utf8);
        assert_eq!(status_msg.min_dict_key_size, Some(DictKeySize::U8));

        let scope_name = def.get_nested("scope", "name").unwrap();
        assert_eq!(scope_name.native_type, NativeType::Utf8);
        assert_eq!(scope_name.min_dict_key_size, None);
    }

    #[test]
    fn test_exp_histogram_nested_columns() {
        let def = get(ArrowPayloadType::ExpHistogramDataPoints);

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
        let def = get(ArrowPayloadType::SummaryDataPoints);

        let qq = def.get_nested("quantile", "quantile").unwrap();
        assert_eq!(qq.native_type, NativeType::Float64);

        let qv = def.get_nested("quantile", "value").unwrap();
        assert_eq!(qv.native_type, NativeType::Float64);
    }

    #[test]
    fn test_dotted_path_logs() {
        let def = get(ArrowPayloadType::Logs);

        let cases = [
            (BODY_TYPE, NativeType::UInt8, None),
            (BODY_STR, NativeType::Utf8, Some(DictKeySize::U16)),
            (BODY_INT, NativeType::Int64, Some(DictKeySize::U16)),
            (BODY_DOUBLE, NativeType::Float64, None),
            (BODY_BOOL, NativeType::Boolean, None),
            (BODY_BYTES, NativeType::Binary, Some(DictKeySize::U16)),
            (BODY_SER, NativeType::Binary, Some(DictKeySize::U16)),
            (RESOURCE_ID, NativeType::UInt16, None),
            (RESOURCE_DROPPED_ATTRIBUTES_COUNT, NativeType::UInt32, None),
            (RESOURCE_SCHEMA_URL, NativeType::Utf8, Some(DictKeySize::U8)),
            (SCOPE_ID, NativeType::UInt16, None),
            (SCOPE_DROPPED_ATTRIBUTES_COUNT, NativeType::UInt32, None),
            (SCOPE_NAME, NativeType::Utf8, Some(DictKeySize::U8)),
            (SCOPE_VERSION, NativeType::Utf8, Some(DictKeySize::U8)),
        ];
        for (path, expected_type, expected_dict) in cases {
            let col = def
                .get(path)
                .unwrap_or_else(|| panic!("missing dotted path: {}", path));
            assert_eq!(col.native_type, expected_type, "type mismatch for {}", path);
            assert_eq!(
                col.min_dict_key_size, expected_dict,
                "dict mismatch for {}",
                path
            );
        }
    }

    #[test]
    fn test_dotted_path_spans() {
        let def = get(ArrowPayloadType::Spans);

        let cases = [
            (RESOURCE_ID, NativeType::UInt16, None),
            (RESOURCE_SCHEMA_URL, NativeType::Utf8, Some(DictKeySize::U8)),
            (SCOPE_NAME, NativeType::Utf8, None),
            (SCOPE_VERSION, NativeType::Utf8, None),
            (STATUS_DOT_CODE, NativeType::Int32, Some(DictKeySize::U8)),
            (
                STATUS_DOT_STATUS_MESSAGE,
                NativeType::Utf8,
                Some(DictKeySize::U8),
            ),
        ];
        for (path, expected_type, expected_dict) in cases {
            let col = def
                .get(path)
                .unwrap_or_else(|| panic!("missing dotted path: {}", path));
            assert_eq!(col.native_type, expected_type, "type mismatch for {}", path);
            assert_eq!(
                col.min_dict_key_size, expected_dict,
                "dict mismatch for {}",
                path
            );
        }
    }

    #[test]
    fn test_dotted_path_exp_histogram() {
        let def = get(ArrowPayloadType::ExpHistogramDataPoints);

        let cases = [
            (POSITIVE_BUCKET_COUNTS, NativeType::ListUInt64),
            (POSITIVE_OFFSET, NativeType::Int32),
            (NEGATIVE_BUCKET_COUNTS, NativeType::ListUInt64),
            (NEGATIVE_OFFSET, NativeType::Int32),
        ];
        for (path, expected_type) in cases {
            let col = def
                .get(path)
                .unwrap_or_else(|| panic!("missing dotted path: {}", path));
            assert_eq!(col.native_type, expected_type, "type mismatch for {}", path);
        }
    }

    #[test]
    fn test_dotted_path_matches_get_nested() {
        let logs = get(ArrowPayloadType::Logs);
        assert!(std::ptr::eq(
            logs.get(BODY_STR).unwrap(),
            logs.get_nested("body", "str").unwrap(),
        ));
        assert!(std::ptr::eq(
            logs.get(RESOURCE_SCHEMA_URL).unwrap(),
            logs.get_nested("resource", "schema_url").unwrap(),
        ));
        assert!(std::ptr::eq(
            logs.get(SCOPE_NAME).unwrap(),
            logs.get_nested("scope", "name").unwrap(),
        ));

        let spans = get(ArrowPayloadType::Spans);
        assert!(std::ptr::eq(
            spans.get(STATUS_DOT_CODE).unwrap(),
            spans.get_nested("status", "code").unwrap(),
        ));

        let exp = get(ArrowPayloadType::ExpHistogramDataPoints);
        assert!(std::ptr::eq(
            exp.get(POSITIVE_BUCKET_COUNTS).unwrap(),
            exp.get_nested("positive", "bucket_counts").unwrap(),
        ));

        let summary = get(ArrowPayloadType::SummaryDataPoints);
        assert!(std::ptr::eq(
            summary.get(QUANTILE_QUANTILE).unwrap(),
            summary.get_nested("quantile", "quantile").unwrap(),
        ));
    }
}
