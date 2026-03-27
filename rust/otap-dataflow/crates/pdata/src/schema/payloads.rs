// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload type definitions from the OTAP spec.
//!
//! Each payload type has a [`Schema`] that describes its columns, their OTAP
//! data types, dictionary encoding constraints, and required/optional status.

use super::schema::{DataType, DictKeySize, Field, Schema, SimpleType};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::*;

/// Get the schema definition for a given payload type.
#[must_use]
pub fn get(typ: ArrowPayloadType) -> &'static Schema {
    match typ {
        ArrowPayloadType::Unknown => &Schema::EMPTY,

        // Logs
        ArrowPayloadType::Logs => &logs::SCHEMA,
        ArrowPayloadType::LogAttrs => &attributes_16::SCHEMA,

        // Traces
        ArrowPayloadType::Spans => &spans::SCHEMA,
        ArrowPayloadType::SpanAttrs => &attributes_16::SCHEMA,
        ArrowPayloadType::SpanEvents => &span_events::SCHEMA,
        ArrowPayloadType::SpanLinks => &span_links::SCHEMA,
        ArrowPayloadType::SpanEventAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::SpanLinkAttrs => &attributes_32::SCHEMA,

        // Metrics
        ArrowPayloadType::UnivariateMetrics => &univariate_metrics::SCHEMA,
        ArrowPayloadType::MultivariateMetrics => &Schema::EMPTY,
        ArrowPayloadType::NumberDataPoints => &number_data_points::SCHEMA,
        ArrowPayloadType::SummaryDataPoints => &summary_data_points::SCHEMA,
        ArrowPayloadType::HistogramDataPoints => &histogram_data_points::SCHEMA,
        ArrowPayloadType::ExpHistogramDataPoints => &exp_histogram_data_points::SCHEMA,
        ArrowPayloadType::NumberDpExemplars => &exemplars::SCHEMA,
        ArrowPayloadType::HistogramDpExemplars => &exemplars::SCHEMA,
        ArrowPayloadType::ExpHistogramDpExemplars => &exemplars::SCHEMA,

        // Metric attributes
        ArrowPayloadType::MetricAttrs => &attributes_16::SCHEMA,
        ArrowPayloadType::NumberDpAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::SummaryDpAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::HistogramDpAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::ExpHistogramDpAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::NumberDpExemplarAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::HistogramDpExemplarAttrs => &attributes_32::SCHEMA,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs => &attributes_32::SCHEMA,

        // Common
        ArrowPayloadType::ResourceAttrs => &attributes_16::SCHEMA,
        ArrowPayloadType::ScopeAttrs => &attributes_16::SCHEMA,
    }
}

use SimpleType::*;

static RESOURCE_SCHEMA: Schema = Schema {
    fields: &[
        Field {
            name: ID,
            data_type: DataType::Simple(UInt16),
            required: false,
        },
        Field {
            name: DROPPED_ATTRIBUTES_COUNT,
            data_type: DataType::Simple(UInt32),
            required: false,
        },
        Field {
            name: SCHEMA_URL,
            data_type: DataType::Dictionary {
                min_key_size: DictKeySize::U8,
                value_type: Utf8,
            },
            required: false,
        },
    ],
    idx: resource_idx,
};

fn resource_idx(name: &str) -> Option<usize> {
    match name {
        ID => Some(0),
        DROPPED_ATTRIBUTES_COUNT => Some(1),
        SCHEMA_URL => Some(2),
        _ => None,
    }
}

static SCOPE_SCHEMA: Schema = Schema {
    fields: &[
        Field {
            name: ID,
            data_type: DataType::Simple(UInt16),
            required: false,
        },
        Field {
            name: DROPPED_ATTRIBUTES_COUNT,
            data_type: DataType::Simple(UInt32),
            required: false,
        },
        Field {
            name: NAME,
            data_type: DataType::Dictionary {
                min_key_size: DictKeySize::U8,
                value_type: Utf8,
            },
            required: false,
        },
        Field {
            name: VERSION,
            data_type: DataType::Dictionary {
                min_key_size: DictKeySize::U8,
                value_type: Utf8,
            },
            required: false,
        },
    ],
    idx: scope_idx,
};

fn scope_idx(name: &str) -> Option<usize> {
    match name {
        ID => Some(0),
        DROPPED_ATTRIBUTES_COUNT => Some(1),
        NAME => Some(2),
        VERSION => Some(3),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Payload definitions
// ---------------------------------------------------------------------------

mod attributes_16 {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: ATTRIBUTE_KEY,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_TYPE,
                data_type: DataType::Simple(UInt8),
                required: false,
            },
            Field {
                name: ATTRIBUTE_STR,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_INT,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Int64,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_DOUBLE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BOOL,
                data_type: DataType::Simple(Boolean),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BYTES,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_SER,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ATTRIBUTE_KEY => Some(1),
            ATTRIBUTE_TYPE => Some(2),
            ATTRIBUTE_STR => Some(3),
            ATTRIBUTE_INT => Some(4),
            ATTRIBUTE_DOUBLE => Some(5),
            ATTRIBUTE_BOOL => Some(6),
            ATTRIBUTE_BYTES => Some(7),
            ATTRIBUTE_SER => Some(8),
            _ => None,
        }
    }
}

mod attributes_32 {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: UInt32,
                },
                required: true,
            },
            Field {
                name: ATTRIBUTE_KEY,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_TYPE,
                data_type: DataType::Simple(UInt8),
                required: false,
            },
            Field {
                name: ATTRIBUTE_STR,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_INT,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Int64,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_DOUBLE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BOOL,
                data_type: DataType::Simple(Boolean),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BYTES,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_SER,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ATTRIBUTE_KEY => Some(1),
            ATTRIBUTE_TYPE => Some(2),
            ATTRIBUTE_STR => Some(3),
            ATTRIBUTE_INT => Some(4),
            ATTRIBUTE_DOUBLE => Some(5),
            ATTRIBUTE_BOOL => Some(6),
            ATTRIBUTE_BYTES => Some(7),
            ATTRIBUTE_SER => Some(8),
            _ => None,
        }
    }
}

mod logs {
    use super::*;

    static BODY_SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: ATTRIBUTE_TYPE,
                data_type: DataType::Simple(UInt8),
                required: false,
            },
            Field {
                name: ATTRIBUTE_STR,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_INT,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Int64,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_DOUBLE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BOOL,
                data_type: DataType::Simple(Boolean),
                required: false,
            },
            Field {
                name: ATTRIBUTE_BYTES,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
            Field {
                name: ATTRIBUTE_SER,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U16,
                    value_type: Binary,
                },
                required: false,
            },
        ],
        idx: body_idx,
    };

    fn body_idx(name: &str) -> Option<usize> {
        match name {
            ATTRIBUTE_TYPE => Some(0),
            ATTRIBUTE_STR => Some(1),
            ATTRIBUTE_INT => Some(2),
            ATTRIBUTE_DOUBLE => Some(3),
            ATTRIBUTE_BOOL => Some(4),
            ATTRIBUTE_BYTES => Some(5),
            ATTRIBUTE_SER => Some(6),
            _ => None,
        }
    }

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: OBSERVED_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: BODY,
                data_type: DataType::Struct(&BODY_SCHEMA),
                required: false,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt16),
                required: false,
            },
            Field {
                name: SEVERITY_NUMBER,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Int32,
                },
                required: false,
            },
            Field {
                name: SEVERITY_TEXT,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: DROPPED_ATTRIBUTES_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: EVENT_NAME,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: TRACE_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(16),
                },
                required: false,
            },
            Field {
                name: SPAN_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(8),
                },
                required: false,
            },
            Field {
                name: SCHEMA_URL,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: RESOURCE,
                data_type: DataType::Struct(&RESOURCE_SCHEMA),
                required: false,
            },
            Field {
                name: SCOPE,
                data_type: DataType::Struct(&SCOPE_SCHEMA),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            TIME_UNIX_NANO => Some(0),
            OBSERVED_TIME_UNIX_NANO => Some(1),
            BODY => Some(2),
            ID => Some(3),
            SEVERITY_NUMBER => Some(4),
            SEVERITY_TEXT => Some(5),
            DROPPED_ATTRIBUTES_COUNT => Some(6),
            EVENT_NAME => Some(7),
            FLAGS => Some(8),
            TRACE_ID => Some(9),
            SPAN_ID => Some(10),
            SCHEMA_URL => Some(11),
            RESOURCE => Some(12),
            SCOPE => Some(13),
            _ => None,
        }
    }
}

mod spans {
    use super::*;

    static STATUS_SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: STATUS_CODE,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Int32,
                },
                required: false,
            },
            Field {
                name: STATUS_MESSAGE,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
        ],
        idx: status_idx,
    };

    fn status_idx(name: &str) -> Option<usize> {
        match name {
            STATUS_CODE => Some(0),
            STATUS_MESSAGE => Some(1),
            _ => None,
        }
    }

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: START_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: DURATION_TIME_UNIX_NANO,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: DurationNanosecond,
                },
                required: false,
            },
            Field {
                name: TRACE_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(16),
                },
                required: false,
            },
            Field {
                name: SPAN_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(8),
                },
                required: false,
            },
            Field {
                name: NAME,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt16),
                required: false,
            },
            Field {
                name: KIND,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Int32,
                },
                required: false,
            },
            Field {
                name: PARENT_SPAN_ID,
                data_type: DataType::Simple(FixedSizeBinary(8)),
                required: false,
            },
            Field {
                name: DROPPED_ATTRIBUTES_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: DROPPED_EVENTS_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: DROPPED_LINKS_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: SCHEMA_URL,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: TRACE_STATE,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: RESOURCE,
                data_type: DataType::Struct(&RESOURCE_SCHEMA),
                required: false,
            },
            Field {
                name: SCOPE,
                data_type: DataType::Struct(&SCOPE_SCHEMA),
                required: false,
            },
            Field {
                name: STATUS,
                data_type: DataType::Struct(&STATUS_SCHEMA),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            START_TIME_UNIX_NANO => Some(0),
            DURATION_TIME_UNIX_NANO => Some(1),
            TRACE_ID => Some(2),
            SPAN_ID => Some(3),
            NAME => Some(4),
            ID => Some(5),
            KIND => Some(6),
            PARENT_SPAN_ID => Some(7),
            DROPPED_ATTRIBUTES_COUNT => Some(8),
            DROPPED_EVENTS_COUNT => Some(9),
            DROPPED_LINKS_COUNT => Some(10),
            SCHEMA_URL => Some(11),
            TRACE_STATE => Some(12),
            FLAGS => Some(13),
            RESOURCE => Some(14),
            SCOPE => Some(15),
            STATUS => Some(16),
            _ => None,
        }
    }
}

mod span_events {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: NAME,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: DROPPED_ATTRIBUTES_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            NAME => Some(1),
            ID => Some(2),
            TIME_UNIX_NANO => Some(3),
            DROPPED_ATTRIBUTES_COUNT => Some(4),
            _ => None,
        }
    }
}

mod span_links {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: SPAN_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(8),
                },
                required: false,
            },
            Field {
                name: TRACE_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(16),
                },
                required: false,
            },
            Field {
                name: TRACE_STATE,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: DROPPED_ATTRIBUTES_COUNT,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ID => Some(1),
            SPAN_ID => Some(2),
            TRACE_ID => Some(3),
            TRACE_STATE => Some(4),
            FLAGS => Some(5),
            DROPPED_ATTRIBUTES_COUNT => Some(6),
            _ => None,
        }
    }
}

mod univariate_metrics {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: ID,
                data_type: DataType::Simple(UInt16),
                required: false,
            },
            Field {
                name: METRIC_TYPE,
                data_type: DataType::Simple(UInt8),
                required: false,
            },
            Field {
                name: NAME,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: AGGREGATION_TEMPORALITY,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Int32,
                },
                required: false,
            },
            Field {
                name: DESCRIPTION,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: IS_MONOTONIC,
                data_type: DataType::Simple(Boolean),
                required: false,
            },
            Field {
                name: UNIT,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: SCHEMA_URL,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Utf8,
                },
                required: false,
            },
            Field {
                name: RESOURCE,
                data_type: DataType::Struct(&RESOURCE_SCHEMA),
                required: false,
            },
            Field {
                name: SCOPE,
                data_type: DataType::Struct(&SCOPE_SCHEMA),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            ID => Some(0),
            METRIC_TYPE => Some(1),
            NAME => Some(2),
            AGGREGATION_TEMPORALITY => Some(3),
            DESCRIPTION => Some(4),
            IS_MONOTONIC => Some(5),
            UNIT => Some(6),
            SCHEMA_URL => Some(7),
            RESOURCE => Some(8),
            SCOPE => Some(9),
            _ => None,
        }
    }
}

mod number_data_points {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: START_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: INT_VALUE,
                data_type: DataType::Simple(Int64),
                required: false,
            },
            Field {
                name: DOUBLE_VALUE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            START_TIME_UNIX_NANO => Some(1),
            TIME_UNIX_NANO => Some(2),
            INT_VALUE => Some(3),
            DOUBLE_VALUE => Some(4),
            ID => Some(5),
            FLAGS => Some(6),
            _ => None,
        }
    }
}

mod summary_data_points {
    use super::*;

    static QUANTILE_ITEM_SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: SUMMARY_QUANTILE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: SUMMARY_VALUE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
        ],
        idx: quantile_idx,
    };

    fn quantile_idx(name: &str) -> Option<usize> {
        match name {
            SUMMARY_QUANTILE => Some(0),
            SUMMARY_VALUE => Some(1),
            _ => None,
        }
    }

    static QUANTILE_ITEM_DT: DataType = DataType::Struct(&QUANTILE_ITEM_SCHEMA);
    static VALUE_ITEM_DT: DataType = DataType::Simple(Float64);

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: SUMMARY_COUNT,
                data_type: DataType::Simple(UInt64),
                required: false,
            },
            Field {
                name: SUMMARY_SUM,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: START_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: SUMMARY_QUANTILE_VALUES,
                data_type: DataType::List(&QUANTILE_ITEM_DT),
                required: false,
            },
            Field {
                name: METRIC_VALUE,
                data_type: DataType::List(&VALUE_ITEM_DT),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ID => Some(1),
            SUMMARY_COUNT => Some(2),
            SUMMARY_SUM => Some(3),
            START_TIME_UNIX_NANO => Some(4),
            TIME_UNIX_NANO => Some(5),
            FLAGS => Some(6),
            SUMMARY_QUANTILE_VALUES => Some(7),
            METRIC_VALUE => Some(8),
            _ => None,
        }
    }
}

mod histogram_data_points {
    use super::*;

    static BUCKET_COUNTS_ITEM_DT: DataType = DataType::Simple(UInt64);
    static EXPLICIT_BOUNDS_ITEM_DT: DataType = DataType::Simple(Float64);

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: HISTOGRAM_COUNT,
                data_type: DataType::Simple(UInt64),
                required: false,
            },
            Field {
                name: HISTOGRAM_SUM,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: HISTOGRAM_MIN,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: HISTOGRAM_MAX,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: HISTOGRAM_BUCKET_COUNTS,
                data_type: DataType::List(&BUCKET_COUNTS_ITEM_DT),
                required: false,
            },
            Field {
                name: HISTOGRAM_EXPLICIT_BOUNDS,
                data_type: DataType::List(&EXPLICIT_BOUNDS_ITEM_DT),
                required: false,
            },
            Field {
                name: START_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ID => Some(1),
            HISTOGRAM_COUNT => Some(2),
            HISTOGRAM_SUM => Some(3),
            HISTOGRAM_MIN => Some(4),
            HISTOGRAM_MAX => Some(5),
            HISTOGRAM_BUCKET_COUNTS => Some(6),
            HISTOGRAM_EXPLICIT_BOUNDS => Some(7),
            START_TIME_UNIX_NANO => Some(8),
            TIME_UNIX_NANO => Some(9),
            FLAGS => Some(10),
            _ => None,
        }
    }
}

mod exp_histogram_data_points {
    use super::*;

    static BUCKET_COUNTS_ITEM_DT: DataType = DataType::Simple(UInt64);

    static BUCKETS_SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: EXP_HISTOGRAM_BUCKET_COUNTS,
                data_type: DataType::List(&BUCKET_COUNTS_ITEM_DT),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_OFFSET,
                data_type: DataType::Simple(Int32),
                required: false,
            },
        ],
        idx: buckets_idx,
    };

    fn buckets_idx(name: &str) -> Option<usize> {
        match name {
            EXP_HISTOGRAM_BUCKET_COUNTS => Some(0),
            EXP_HISTOGRAM_OFFSET => Some(1),
            _ => None,
        }
    }

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Simple(UInt16),
                required: true,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: HISTOGRAM_COUNT,
                data_type: DataType::Simple(UInt64),
                required: false,
            },
            Field {
                name: HISTOGRAM_SUM,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: HISTOGRAM_MIN,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: HISTOGRAM_MAX,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_SCALE,
                data_type: DataType::Simple(Int32),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_ZERO_COUNT,
                data_type: DataType::Simple(UInt64),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_ZERO_THRESHOLD,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_POSITIVE,
                data_type: DataType::Struct(&BUCKETS_SCHEMA),
                required: false,
            },
            Field {
                name: EXP_HISTOGRAM_NEGATIVE,
                data_type: DataType::Struct(&BUCKETS_SCHEMA),
                required: false,
            },
            Field {
                name: START_TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: FLAGS,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ID => Some(1),
            HISTOGRAM_COUNT => Some(2),
            HISTOGRAM_SUM => Some(3),
            HISTOGRAM_MIN => Some(4),
            HISTOGRAM_MAX => Some(5),
            EXP_HISTOGRAM_SCALE => Some(6),
            EXP_HISTOGRAM_ZERO_COUNT => Some(7),
            EXP_HISTOGRAM_ZERO_THRESHOLD => Some(8),
            EXP_HISTOGRAM_POSITIVE => Some(9),
            EXP_HISTOGRAM_NEGATIVE => Some(10),
            START_TIME_UNIX_NANO => Some(11),
            TIME_UNIX_NANO => Some(12),
            FLAGS => Some(13),
            _ => None,
        }
    }
}

mod exemplars {
    use super::*;

    pub(super) static SCHEMA: Schema = Schema {
        fields: &[
            Field {
                name: PARENT_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: UInt32,
                },
                required: true,
            },
            Field {
                name: ID,
                data_type: DataType::Simple(UInt32),
                required: false,
            },
            Field {
                name: DOUBLE_VALUE,
                data_type: DataType::Simple(Float64),
                required: false,
            },
            Field {
                name: INT_VALUE,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: Int64,
                },
                required: false,
            },
            Field {
                name: SPAN_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(8),
                },
                required: false,
            },
            Field {
                name: TIME_UNIX_NANO,
                data_type: DataType::Simple(TimestampNanosecond),
                required: false,
            },
            Field {
                name: TRACE_ID,
                data_type: DataType::Dictionary {
                    min_key_size: DictKeySize::U8,
                    value_type: FixedSizeBinary(16),
                },
                required: false,
            },
        ],
        idx,
    };

    fn idx(name: &str) -> Option<usize> {
        match name {
            PARENT_ID => Some(0),
            ID => Some(1),
            DOUBLE_VALUE => Some(2),
            INT_VALUE => Some(3),
            SPAN_ID => Some(4),
            TIME_UNIX_NANO => Some(5),
            TRACE_ID => Some(6),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::{Logs, Metrics, OtapBatchStore, Traces};
    use arrow::array::*;
    use arrow::datatypes::{
        DataType as ArrowDT, Field as ArrowField, Fields, Schema as ArrowSchema, TimeUnit,
    };
    use std::sync::Arc;

    /// Return the union of all payload types across Logs, Traces, and Metrics.
    fn all_payload_types() -> Vec<ArrowPayloadType> {
        Logs::allowed_payload_types()
            .iter()
            .chain(Traces::allowed_payload_types())
            .chain(Metrics::allowed_payload_types())
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Build a 1-element Arrow array for the given OTAP DataType.
    fn build_array(dt: &DataType) -> (ArrowDT, Arc<dyn Array>) {
        use arrow::array::*;
        use arrow::buffer::OffsetBuffer;

        match dt {
            DataType::Simple(s) | DataType::Dictionary { value_type: s, .. } => match s {
                Boolean => (ArrowDT::Boolean, Arc::new(BooleanArray::from(vec![false]))),
                UInt8 => (ArrowDT::UInt8, Arc::new(UInt8Array::from(vec![0u8]))),
                UInt16 => (ArrowDT::UInt16, Arc::new(UInt16Array::from(vec![0u16]))),
                UInt32 => (ArrowDT::UInt32, Arc::new(UInt32Array::from(vec![0u32]))),
                UInt64 => (ArrowDT::UInt64, Arc::new(UInt64Array::from(vec![0u64]))),
                Int32 => (ArrowDT::Int32, Arc::new(Int32Array::from(vec![0i32]))),
                Int64 => (ArrowDT::Int64, Arc::new(Int64Array::from(vec![0i64]))),
                Float64 => (ArrowDT::Float64, Arc::new(Float64Array::from(vec![0.0]))),
                Utf8 => (ArrowDT::Utf8, Arc::new(StringArray::from(vec![""]))),
                Binary => (
                    ArrowDT::Binary,
                    Arc::new(BinaryArray::from(vec![b"" as &[u8]])),
                ),
                FixedSizeBinary(n) => {
                    let val = vec![0u8; *n as usize];
                    (
                        ArrowDT::FixedSizeBinary(*n),
                        Arc::new(
                            FixedSizeBinaryArray::try_from_iter(std::iter::once(val.as_slice()))
                                .unwrap(),
                        ),
                    )
                }
                TimestampNanosecond => (
                    ArrowDT::Timestamp(TimeUnit::Nanosecond, None),
                    Arc::new(TimestampNanosecondArray::from(vec![0i64])),
                ),
                DurationNanosecond => (
                    ArrowDT::Duration(TimeUnit::Nanosecond),
                    Arc::new(DurationNanosecondArray::from(vec![0i64])),
                ),
            },
            DataType::Struct(_) => (
                ArrowDT::Struct(Fields::empty()),
                Arc::new(StructArray::new_empty_fields(1, None)),
            ),
            DataType::List(inner) => match inner {
                DataType::Struct(_) => {
                    let item_field = Arc::new(ArrowField::new(
                        "item",
                        ArrowDT::Struct(Fields::empty()),
                        true,
                    ));
                    let items = StructArray::new_empty_fields(0, None);
                    let offsets = OffsetBuffer::from_lengths([0usize]);
                    let list = ListArray::new(item_field.clone(), offsets, Arc::new(items), None);
                    (ArrowDT::List(item_field), Arc::new(list))
                }
                DataType::Simple(s) => {
                    let arrow_dt = s.to_arrow();
                    let item_field = Arc::new(ArrowField::new("item", arrow_dt.clone(), true));
                    let values = new_null_array(&arrow_dt, 0);
                    let offsets = OffsetBuffer::from_lengths([0usize]);
                    let list = ListArray::new(item_field.clone(), offsets, values, None);
                    (ArrowDT::List(item_field), Arc::new(list))
                }
                _ => panic!("unsupported List inner type: {inner:?}"),
            },
        }
    }

    /// Build a valid 1-row RecordBatch with ALL columns from the definition.
    fn build_valid_batch(payload_type: ArrowPayloadType) -> Option<RecordBatch> {
        let schema = get(payload_type);
        if schema.fields().is_empty() {
            return None;
        }

        let mut fields = Vec::with_capacity(schema.fields().len());
        let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(schema.fields().len());

        for field_def in schema.fields() {
            let (dt, array) = build_array(&field_def.data_type);
            fields.push(ArrowField::new(field_def.name, dt, !field_def.required));
            arrays.push(array);
        }

        let arrow_schema = Arc::new(ArrowSchema::new(fields));
        Some(RecordBatch::try_new(arrow_schema, arrays).unwrap())
    }

    /// Build a valid 1-row RecordBatch but omit the named column.
    fn build_batch_omitting(payload_type: ArrowPayloadType, omit_col: &str) -> RecordBatch {
        let schema = get(payload_type);
        let capacity = schema.fields().len() - 1;
        let mut fields = Vec::with_capacity(capacity);
        let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(capacity);

        for field_def in schema.fields() {
            if field_def.name == omit_col {
                continue;
            }
            let (dt, arr) = build_array(&field_def.data_type);
            fields.push(ArrowField::new(field_def.name, dt, !field_def.required));
            arrays.push(arr);
        }

        let arrow_schema = Arc::new(ArrowSchema::new(fields));
        RecordBatch::try_new(arrow_schema, arrays).unwrap()
    }

    /// Build a 1-element dictionary-encoded array.
    fn build_dict_array(key_dt: ArrowDT, value_dt: &DataType) -> (ArrowDT, Arc<dyn Array>) {
        use arrow::compute::kernels::cast::cast;

        let (val_arrow_dt, native_arr) = build_array(value_dt);
        let dict_dt = ArrowDT::Dictionary(Box::new(key_dt), Box::new(val_arrow_dt));
        let dict_arr = cast(&*native_arr, &dict_dt)
            .unwrap_or_else(|e| panic!("cast to {dict_dt:?} failed: {e}"));
        (dict_dt, dict_arr)
    }

    /// Build a 1-row RecordBatch where one column has a null value.
    fn build_batch_with_null_column(payload_type: ArrowPayloadType, null_col: &str) -> RecordBatch {
        use arrow::array::*;

        let schema = get(payload_type);
        let mut fields = Vec::with_capacity(schema.fields().len());
        let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(schema.fields().len());

        for field_def in schema.fields() {
            if field_def.name == null_col {
                let (dt, array): (ArrowDT, Arc<dyn Array>) = match &field_def.data_type {
                    DataType::Simple(s) | DataType::Dictionary { value_type: s, .. } => match s {
                        Boolean => (
                            ArrowDT::Boolean,
                            Arc::new(BooleanArray::from(vec![None as Option<bool>])),
                        ),
                        UInt8 => (
                            ArrowDT::UInt8,
                            Arc::new(UInt8Array::from(vec![None as Option<u8>])),
                        ),
                        UInt16 => (
                            ArrowDT::UInt16,
                            Arc::new(UInt16Array::from(vec![None as Option<u16>])),
                        ),
                        UInt32 => (
                            ArrowDT::UInt32,
                            Arc::new(UInt32Array::from(vec![None as Option<u32>])),
                        ),
                        UInt64 => (
                            ArrowDT::UInt64,
                            Arc::new(UInt64Array::from(vec![None as Option<u64>])),
                        ),
                        Int32 => (
                            ArrowDT::Int32,
                            Arc::new(Int32Array::from(vec![None as Option<i32>])),
                        ),
                        Int64 => (
                            ArrowDT::Int64,
                            Arc::new(Int64Array::from(vec![None as Option<i64>])),
                        ),
                        Float64 => (
                            ArrowDT::Float64,
                            Arc::new(Float64Array::from(vec![None as Option<f64>])),
                        ),
                        Utf8 => (
                            ArrowDT::Utf8,
                            Arc::new(StringArray::from(vec![None as Option<&str>])),
                        ),
                        Binary => (
                            ArrowDT::Binary,
                            Arc::new(BinaryArray::from(vec![None as Option<&[u8]>])),
                        ),
                        FixedSizeBinary(n) => {
                            let mut builder = FixedSizeBinaryBuilder::with_capacity(1, *n);
                            builder.append_null();
                            (
                                ArrowDT::FixedSizeBinary(*n),
                                Arc::new(builder.finish()) as Arc<dyn Array>,
                            )
                        }
                        TimestampNanosecond => (
                            ArrowDT::Timestamp(TimeUnit::Nanosecond, None),
                            Arc::new(TimestampNanosecondArray::from(vec![None as Option<i64>])),
                        ),
                        DurationNanosecond => (
                            ArrowDT::Duration(TimeUnit::Nanosecond),
                            Arc::new(DurationNanosecondArray::from(vec![None as Option<i64>])),
                        ),
                    },
                    // Complex types should never be required.
                    _ => build_array(&field_def.data_type),
                };
                fields.push(ArrowField::new(field_def.name, dt, true));
                arrays.push(array);
            } else {
                let (dt, arr) = build_array(&field_def.data_type);
                fields.push(ArrowField::new(field_def.name, dt, !field_def.required));
                arrays.push(arr);
            }
        }

        let arrow_schema = Arc::new(ArrowSchema::new(fields));
        RecordBatch::try_new(arrow_schema, arrays).unwrap()
    }

    // -- Targeted sub-schema idx tests -----------------------------------------

    /// Verify every field in the resource sub-schema (used by all root signal
    /// tables) is reachable via its `idx` function, and unknown names return
    /// `None`.  This exercises `resource_idx` directly.
    #[test]
    fn test_resource_idx() {
        // Reach the resource sub-schema through the Logs payload.
        let logs_schema = get(ArrowPayloadType::Logs);
        let res_field = logs_schema.get(RESOURCE).expect("resource field in logs");
        let res_sub = res_field
            .data_type
            .as_struct_schema()
            .expect("resource is a struct");

        assert!(res_sub.get(ID).is_some(), "resource sub-schema has 'id'");
        assert!(
            res_sub.get(DROPPED_ATTRIBUTES_COUNT).is_some(),
            "resource sub-schema has 'dropped_attributes_count'"
        );
        assert!(
            res_sub.get(SCHEMA_URL).is_some(),
            "resource sub-schema has 'schema_url'"
        );
        assert!(
            res_sub.get("__unknown__").is_none(),
            "resource sub-schema rejects unknown field"
        );
    }

    /// Verify every field in the quantile-item sub-schema (nested inside the
    /// `quantile` List column of SummaryDataPoints) is reachable via its `idx`
    /// function.  This exercises `quantile_idx` directly.
    #[test]
    fn test_quantile_idx() {
        let schema = get(ArrowPayloadType::SummaryDataPoints);
        let qv_field = schema
            .get(SUMMARY_QUANTILE_VALUES)
            .expect("quantile_values field in SummaryDataPoints");
        let inner_dt = match &qv_field.data_type {
            DataType::List(inner) => inner,
            other => panic!("expected List, got {other:?}"),
        };
        let item_sub = inner_dt
            .as_struct_schema()
            .expect("quantile list item is a struct");

        assert!(
            item_sub.get(SUMMARY_QUANTILE).is_some(),
            "quantile item schema has 'quantile'"
        );
        assert!(
            item_sub.get(SUMMARY_VALUE).is_some(),
            "quantile item schema has 'value'"
        );
        assert!(
            item_sub.get("__unknown__").is_none(),
            "quantile item schema rejects unknown field"
        );
    }

    /// Verify every field in the buckets sub-schema (used by the `positive` and
    /// `negative` struct columns of ExpHistogramDataPoints) is reachable via its
    /// `idx` function.  This exercises `buckets_idx` directly.
    #[test]
    fn test_buckets_idx() {
        let schema = get(ArrowPayloadType::ExpHistogramDataPoints);

        for col_name in [EXP_HISTOGRAM_POSITIVE, EXP_HISTOGRAM_NEGATIVE] {
            let field = schema
                .get(col_name)
                .unwrap_or_else(|| panic!("field '{col_name}' in ExpHistogramDataPoints"));
            let buckets_sub = field
                .data_type
                .as_struct_schema()
                .unwrap_or_else(|| panic!("'{col_name}' is a struct"));

            assert!(
                buckets_sub.get(EXP_HISTOGRAM_BUCKET_COUNTS).is_some(),
                "'{col_name}' sub-schema has 'bucket_counts'"
            );
            assert!(
                buckets_sub.get(EXP_HISTOGRAM_OFFSET).is_some(),
                "'{col_name}' sub-schema has 'offset'"
            );
            assert!(
                buckets_sub.get("__unknown__").is_none(),
                "'{col_name}' sub-schema rejects unknown field"
            );
        }
    }

    // -- Structural tests ------------------------------------------------------

    #[test]
    fn test_allowed_payload_types_covered() {
        for pt in Logs::allowed_payload_types() {
            let _ = get(*pt);
        }
        for pt in Traces::allowed_payload_types() {
            let _ = get(*pt);
        }
        for pt in Metrics::allowed_payload_types() {
            let _ = get(*pt);
        }
    }

    #[test]
    fn test_idx_functions_consistent_with_fields() {
        for pt in all_payload_types() {
            let schema = get(pt);
            for (i, field) in schema.fields().iter().enumerate() {
                let got = schema.get(field.name);
                assert!(
                    got.is_some(),
                    "{pt:?}: field {} at index {i} not found via get()",
                    field.name,
                );
                assert_eq!(
                    got.unwrap().name,
                    field.name,
                    "{pt:?}: field at index {i} mismatch"
                );
            }
        }
    }

    #[test]
    fn test_required_fields_are_in_schema() {
        for pt in all_payload_types() {
            let schema = get(pt);
            for req_name in schema.required_field_names() {
                let field = schema.get(req_name);
                assert!(
                    field.is_some(),
                    "{pt:?}: required field {req_name} not found"
                );
                assert!(
                    field.unwrap().required,
                    "{pt:?}: field {req_name} is in required_field_names but required=false"
                );
            }
        }
    }

    #[test]
    fn test_required_count_le_total_count() {
        for pt in all_payload_types() {
            let schema = get(pt);
            let req_count = schema.required_field_names().count();
            assert!(
                req_count <= schema.fields().len(),
                "{pt:?}: required ({req_count}) > total ({})",
                schema.fields().len(),
            );
        }
    }

    #[test]
    fn test_shared_resource_scope_schemas() {
        let logs_schema = get(ArrowPayloadType::Logs);
        let spans_schema = get(ArrowPayloadType::Spans);
        let metrics_schema = get(ArrowPayloadType::UnivariateMetrics);

        for (label, schema) in [
            ("logs", logs_schema),
            ("spans", spans_schema),
            ("metrics", metrics_schema),
        ] {
            let res = schema
                .get(RESOURCE)
                .unwrap_or_else(|| panic!("{label}: no resource"));
            let res_sub = res
                .data_type
                .as_struct_schema()
                .unwrap_or_else(|| panic!("{label}: resource not struct"));
            assert_eq!(res_sub.fields().len(), 3, "{label}: resource sub-fields");

            let scope = schema
                .get(SCOPE)
                .unwrap_or_else(|| panic!("{label}: no scope"));
            let scope_sub = scope
                .data_type
                .as_struct_schema()
                .unwrap_or_else(|| panic!("{label}: scope not struct"));
            assert_eq!(scope_sub.fields().len(), 4, "{label}: scope sub-fields");
        }
    }

    // -- Validation tests ------------------------------------------------------

    #[test]
    fn test_validate_accepts_valid_batch_all_payload_types() {
        for pt in all_payload_types() {
            let batch = match build_valid_batch(pt) {
                Some(b) => b,
                None => continue,
            };
            get(pt)
                .check_match(&batch)
                .unwrap_or_else(|e| panic!("check_match failed for {pt:?}: {e:?}"));
        }
    }

    #[test]
    fn test_validate_rejects_extraneous_column_all_payload_types() {
        use arrow::array::Int32Array;

        for pt in all_payload_types() {
            let schema_def = get(pt);
            if schema_def.fields().is_empty() {
                continue;
            }

            let field = ArrowField::new("__bogus_column__", ArrowDT::Int32, true);
            let schema = Arc::new(ArrowSchema::new(vec![field]));
            let array = Arc::new(Int32Array::from(vec![0]));
            let batch = RecordBatch::try_new(schema, vec![array]).unwrap();

            match get(pt).check_match(&batch) {
                Err(crate::schema::error::Error::ExtraneousField { .. }) => {}
                other => panic!("expected ExtraneousField for {pt:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_validate_rejects_missing_required_all_payload_types() {
        for pt in all_payload_types() {
            let schema_def = get(pt);
            if schema_def.fields().is_empty() {
                continue;
            }
            let required: Vec<&str> = schema_def.required_field_names().collect();
            if required.is_empty() {
                continue;
            }
            for &req_col in &required {
                let batch = build_batch_omitting(pt, req_col);
                match get(pt).check_match(&batch) {
                    Err(crate::schema::error::Error::MissingRequiredFields { names, .. }) => {
                        assert!(
                            names.contains(&req_col.to_string()),
                            "{pt:?}: missing column {req_col} not in names {names:?}",
                        );
                    }
                    other => panic!(
                        "{pt:?}: expected MissingRequiredFields when omitting {req_col}, got {other:?}",
                    ),
                }
            }
        }
    }

    #[test]
    fn test_validate_rejects_wrong_type_all_payload_types() {
        use arrow::array::{BooleanArray, Int64Array};

        for pt in all_payload_types() {
            let schema_def = get(pt);
            if schema_def.fields().is_empty() {
                continue;
            }
            for field_def in schema_def.fields() {
                if field_def.data_type.is_complex() {
                    continue;
                }

                let (wrong_dt, wrong_array): (ArrowDT, Arc<dyn Array>) = match &field_def.data_type
                {
                    DataType::Simple(Int64 | UInt64)
                    | DataType::Dictionary {
                        value_type: Int64 | UInt64,
                        ..
                    } => (ArrowDT::Boolean, Arc::new(BooleanArray::from(vec![false]))),
                    _ => (ArrowDT::Int64, Arc::new(Int64Array::from(vec![0i64]))),
                };

                let mut fields = Vec::with_capacity(schema_def.fields().len());
                let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(schema_def.fields().len());

                for fd in schema_def.fields() {
                    if fd.name == field_def.name {
                        fields.push(ArrowField::new(fd.name, wrong_dt.clone(), true));
                        arrays.push(wrong_array.clone());
                    } else {
                        let (dt, arr) = build_array(&fd.data_type);
                        fields.push(ArrowField::new(fd.name, dt, !fd.required));
                        arrays.push(arr);
                    }
                }

                let schema = Arc::new(ArrowSchema::new(fields));
                let batch = RecordBatch::try_new(schema, arrays).unwrap();
                match get(pt).check_match(&batch) {
                    Err(crate::schema::error::Error::FieldTypeMismatch { .. }) => {}
                    other => panic!(
                        "{pt:?}: expected FieldTypeMismatch for column {}, got {other:?}",
                        field_def.name,
                    ),
                }
            }
        }
    }

    #[test]
    fn test_validate_dict_key_size_all_payload_types() {
        for pt in all_payload_types() {
            let schema_def = get(pt);
            if schema_def.fields().is_empty() {
                continue;
            }

            for field_def in schema_def.fields() {
                // Skip types that can't be dictionary-encoded via cast.
                // Skip types that Arrow cannot dictionary-encode via cast.
                let skip = matches!(
                    &field_def.data_type,
                    DataType::Simple(Boolean | TimestampNanosecond | DurationNanosecond)
                        | DataType::Dictionary {
                            value_type: TimestampNanosecond | DurationNanosecond,
                            ..
                        }
                ) || field_def.data_type.is_complex();
                if skip {
                    continue;
                }

                // Helper: build a batch where this column uses dict encoding with
                // the given key DataType, and all other columns are native.
                let build_batch_with_dict_col = |key_dt: ArrowDT| -> RecordBatch {
                    let mut fields = Vec::with_capacity(schema_def.fields().len());
                    let mut arrays: Vec<Arc<dyn Array>> =
                        Vec::with_capacity(schema_def.fields().len());

                    for fd in schema_def.fields() {
                        if fd.name == field_def.name {
                            let (dt, arr) = build_dict_array(key_dt.clone(), &fd.data_type);
                            fields.push(ArrowField::new(fd.name, dt, !fd.required));
                            arrays.push(arr);
                        } else {
                            let (dt, arr) = build_array(&fd.data_type);
                            fields.push(ArrowField::new(fd.name, dt, !fd.required));
                            arrays.push(arr);
                        }
                    }
                    let schema = Arc::new(ArrowSchema::new(fields));
                    RecordBatch::try_new(schema, arrays).unwrap()
                };

                let schema_def_for_pt = get(pt);
                match field_def.data_type.min_dict_key_size() {
                    Some(DictKeySize::U8) => {
                        // U8 key at min -> Ok
                        let batch = build_batch_with_dict_col(ArrowDT::UInt8);
                        schema_def_for_pt.check_match(&batch).unwrap_or_else(|e| {
                            panic!(
                                "{pt:?}.{}: dict U8 key should be accepted (min=U8): {e:?}",
                                field_def.name,
                            )
                        });
                        // U16 key above min -> Ok
                        let batch = build_batch_with_dict_col(ArrowDT::UInt16);
                        schema_def_for_pt.check_match(&batch).unwrap_or_else(|e| {
                            panic!(
                                "{pt:?}.{}: dict U16 key should be accepted (min=U8): {e:?}",
                                field_def.name,
                            )
                        });
                    }
                    Some(DictKeySize::U16) => {
                        // U16 key at min -> Ok
                        let batch = build_batch_with_dict_col(ArrowDT::UInt16);
                        schema_def_for_pt.check_match(&batch).unwrap_or_else(|e| {
                            panic!(
                                "{pt:?}.{}: dict U16 key should be accepted (min=U16): {e:?}",
                                field_def.name,
                            )
                        });
                        // U8 key below min -> TypeMismatch
                        let batch = build_batch_with_dict_col(ArrowDT::UInt8);
                        match schema_def_for_pt.check_match(&batch) {
                            Err(crate::schema::error::Error::FieldTypeMismatch { .. }) => {}
                            other => panic!(
                                "{pt:?}.{}: dict U8 key should be rejected (min=U16), got {other:?}",
                                field_def.name,
                            ),
                        }
                    }
                    None => {
                        // Dict encoding not allowed -> TypeMismatch
                        let batch = build_batch_with_dict_col(ArrowDT::UInt8);
                        match schema_def_for_pt.check_match(&batch) {
                            Err(crate::schema::error::Error::FieldTypeMismatch { .. }) => {}
                            other => panic!(
                                "{pt:?}.{}: dict should be rejected (no dict allowed), got {other:?}",
                                field_def.name,
                            ),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_validate_rejects_nulls_in_required_all_payload_types() {
        for pt in all_payload_types() {
            let schema_def = get(pt);
            if schema_def.fields().is_empty() {
                continue;
            }
            let required: Vec<&str> = schema_def.required_field_names().collect();
            if required.is_empty() {
                continue;
            }
            for &req_col in &required {
                let field_def = schema_def.get(req_col).unwrap();
                if field_def.data_type.is_complex() {
                    continue;
                }

                let batch = build_batch_with_null_column(pt, req_col);
                match get(pt).check_match(&batch) {
                    Err(crate::schema::error::Error::MissingRequiredFields { names, .. }) => {
                        assert!(
                            names.contains(&req_col.to_string()),
                            "{pt:?}: expected null error for {req_col}, got names {names:?}",
                        );
                    }
                    other => {
                        panic!(
                            "{pt:?}: expected MissingRequiredFields for {req_col}, got {other:?}",
                        )
                    }
                }
            }
        }
    }
}
