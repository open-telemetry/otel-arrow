// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Constants related to OTLP proto messages such as field numbers and other proto constants like
//! wire types

/// Protobuf wire types
pub mod wire_types {
    /// Varint (int32, int64, uint32, uint64, sint32, sint64, bool)
    pub const VARINT: u64 = 0;
    /// 64-bit (fixed64, sfixed64, double)
    pub const FIXED64: u64 = 1;
    /// Length-delimited (string, bytes, embedded messages)
    pub const LEN: u64 = 2;
    /// 32-bit (fixed32, sfixed32, float)
    pub const FIXED32: u64 = 5;
}

/// field number for OTLP protobuf messages
pub mod field_num {
    #[allow(missing_docs)]
    pub mod common {
        pub const KEY_VALUE_KEY: u64 = 1;
        pub const KEY_VALUE_VALUE: u64 = 2;

        pub const ANY_VALUE_STRING_VALUE: u64 = 1;
        pub const ANY_VALUE_BOOL_VALUE: u64 = 2;
        pub const ANY_VALUE_INT_VALUE: u64 = 3;
        pub const ANY_VALUE_DOUBLE_VALUE: u64 = 4;
        pub const ANY_VALUE_ARRAY_VALUE: u64 = 5;
        pub const ANY_VALUE_KVLIST_VALUE: u64 = 6;
        pub const ANY_VALUE_BYTES_VALUE: u64 = 7;

        pub const KEY_VALUE_LIST_VALUES: u64 = 1;
        pub const ARRAY_VALUE_VALUES: u64 = 1;

        pub const INSTRUMENTATION_SCOPE_NAME: u64 = 1;
        pub const INSTRUMENTATION_SCOPE_VERSION: u64 = 2;
        pub const INSTRUMENTATION_SCOPE_ATTRIBUTES: u64 = 3;
        pub const INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT: u64 = 4;
    }

    #[allow(missing_docs)]
    pub mod logs {
        pub const LOGS_DATA_RESOURCE: u64 = 1;

        pub const RESOURCE_LOGS_RESOURCE: u64 = 1;
        pub const RESOURCE_LOGS_SCOPE_LOGS: u64 = 2;
        pub const RESOURCE_LOGS_SCHEMA_URL: u64 = 3;

        pub const SCOPE_LOG_SCOPE: u64 = 1;
        pub const SCOPE_LOGS_LOG_RECORDS: u64 = 2;
        pub const SCOPE_LOGS_SCHEMA_URL: u64 = 3;

        pub const LOG_RECORD_TIME_UNIX_NANO: u64 = 1;
        pub const LOG_RECORD_OBSERVED_TIME_UNIX_NANO: u64 = 11;
        pub const LOG_RECORD_SEVERITY_NUMBER: u64 = 2;
        pub const LOG_RECORD_SEVERITY_TEXT: u64 = 3;
        pub const LOG_RECORD_BODY: u64 = 5;
        pub const LOG_RECORD_ATTRIBUTES: u64 = 6;
        pub const LOG_RECORD_DROPPED_ATTRIBUTES_COUNT: u64 = 7;
        pub const LOG_RECORD_FLAGS: u64 = 8;
        pub const LOG_RECORD_TRACE_ID: u64 = 9;
        pub const LOG_RECORD_SPAN_ID: u64 = 10;
        pub const LOG_RECORD_EVENT_NAME: u64 = 12;
    }

    #[allow(missing_docs)]
    pub mod metrics {
        pub const METRICS_DATA_RESOURCE_METRICS: u64 = 1;

        pub const RESOURCE_METRICS_RESOURCE: u64 = 1;
        pub const RESOURCE_METRICS_SCOPE_METRICS: u64 = 2;
        pub const RESOURCE_METRICS_SCHEMA_URL: u64 = 3;

        pub const SCOPE_METRICS_SCOPE: u64 = 1;
        pub const SCOPE_METRICS_METRICS: u64 = 2;
        pub const SCOPE_METRICS_SCHEMA_URL: u64 = 3;

        pub const METRIC_NAME: u64 = 1;
        pub const METRIC_DESCRIPTION: u64 = 2;
        pub const METRIC_UNIT: u64 = 3;
        pub const METRIC_GAUGE: u64 = 5;
        pub const METRIC_SUM: u64 = 7;
        pub const METRIC_HISTOGRAM: u64 = 9;
        pub const METRIC_EXPONENTIAL_HISTOGRAM: u64 = 10;
        pub const METRIC_SUMMARY: u64 = 11;

        pub const GAUGE_DATA_POINTS: u64 = 1;

        pub const SUM_DATA_POINTS: u64 = 1;
        pub const SUM_AGGREGATION_TEMPORALITY: u64 = 2;
        pub const SUM_IS_MONOTONIC: u64 = 3;

        pub const HISTOGRAM_DATA_POINTS: u64 = 1;
        pub const HISTOGRAM_AGGREGATION_TEMPORALITY: u64 = 2;

        pub const EXPONENTIAL_HISTOGRAM_DATA_POINTS: u64 = 1;
        pub const EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY: u64 = 2;

        pub const SUMMARY_DATA_POINTS: u64 = 1;

        pub const NUMBER_DP_ATTRIBUTES: u64 = 7;
        pub const NUMBER_DP_START_TIME_UNIX_NANO: u64 = 2;
        pub const NUMBER_DP_TIME_UNIX_NANO: u64 = 3;
        pub const NUMBER_DP_AS_DOUBLE: u64 = 4;
        pub const NUMBER_DP_AS_INT: u64 = 6;
        pub const NUMBER_DP_EXEMPLARS: u64 = 5;
        pub const NUMBER_DP_FLAGS: u64 = 8;

        pub const HISTOGRAM_DP_ATTRIBUTES: u64 = 9;
        pub const HISTOGRAM_DP_START_TIME_UNIX_NANO: u64 = 2;
        pub const HISTOGRAM_DP_TIME_UNIX_NANO: u64 = 3;
        pub const HISTOGRAM_DP_COUNT: u64 = 4;
        pub const HISTOGRAM_DP_SUM: u64 = 5;
        pub const HISTOGRAM_DP_BUCKET_COUNTS: u64 = 6;
        pub const HISTOGRAM_DP_EXPLICIT_BOUNDS: u64 = 7;
        pub const HISTOGRAM_DP_EXEMPLARS: u64 = 8;
        pub const HISTOGRAM_DP_FLAGS: u64 = 10;
        pub const HISTOGRAM_DP_MIN: u64 = 11;
        pub const HISTOGRAM_DP_MAX: u64 = 12;

        pub const EXP_HISTOGRAM_DP_ATTRIBUTES: u64 = 1;
        pub const EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO: u64 = 2;
        pub const EXP_HISTOGRAM_DP_TIME_UNIX_NANO: u64 = 3;
        pub const EXP_HISTOGRAM_DP_COUNT: u64 = 4;
        pub const EXP_HISTOGRAM_DP_SUM: u64 = 5;
        pub const EXP_HISTOGRAM_DP_SCALE: u64 = 6;
        pub const EXP_HISTOGRAM_DP_ZERO_COUNT: u64 = 7;
        pub const EXP_HISTOGRAM_DP_POSITIVE: u64 = 8;
        pub const EXP_HISTOGRAM_DP_NEGATIVE: u64 = 9;
        pub const EXP_HISTOGRAM_DP_FLAGS: u64 = 10;
        pub const EXP_HISTOGRAM_DP_EXEMPLARS: u64 = 11;
        pub const EXP_HISTOGRAM_DP_MIN: u64 = 12;
        pub const EXP_HISTOGRAM_DP_MAX: u64 = 13;
        pub const EXP_HISTOGRAM_DP_ZERO_THRESHOLD: u64 = 14;

        pub const EXP_HISTOGRAM_BUCKET_OFFSET: u64 = 1;
        pub const EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS: u64 = 2;

        pub const SUMMARY_DP_ATTRIBUTES: u64 = 7;
        pub const SUMMARY_DP_START_TIME_UNIX_NANO: u64 = 2;
        pub const SUMMARY_DP_TIME_UNIX_NANO: u64 = 3;
        pub const SUMMARY_DP_COUNT: u64 = 4;
        pub const SUMMARY_DP_SUM: u64 = 5;
        pub const SUMMARY_DP_QUANTILE_VALUES: u64 = 6;
        pub const SUMMARY_DP_FLAGS: u64 = 6;

        pub const VALUE_AT_QUANTILE_QUANTILE: u64 = 1;
        pub const VALUE_AT_QUANTILE_VALUE: u64 = 2;

        pub const EXEMPLAR_FILTERED_ATTRIBUTES: u64 = 7;
        pub const EXEMPLAR_TIME_UNIX_NANO: u64 = 2;
        pub const EXEMPLAR_AS_DOUBLE: u64 = 3;
        pub const EXEMPLAR_AS_INT: u64 = 6;
        pub const EXEMPLAR_SPAN_ID: u64 = 4;
        pub const EXEMPLAR_TRACE_ID: u64 = 5;
    }

    #[allow(missing_docs)]
    pub mod resource {
        pub const RESOURCE_ATTRIBUTES: u64 = 1;
        pub const RESOURCE_DROPPED_ATTRIBUTES_COUNT: u64 = 2;
    }

    #[allow(missing_docs)]
    pub mod traces {
        pub const TRACES_DATA_RESOURCE_SPANS: u64 = 1;

        pub const RESOURCE_SPANS_RESOURCE: u64 = 1;
        pub const RESOURCE_SPANS_SCOPE_SPANS: u64 = 2;
        pub const RESOURCE_SPANS_SCHEMA_URL: u64 = 3;

        pub const SCOPE_SPANS_SCOPE: u64 = 1;
        pub const SCOPE_SPANS_SPANS: u64 = 2;
        pub const SCOPE_SPANS_SCHEMA_URL: u64 = 3;

        pub const SPAN_TRACE_ID: u64 = 1;
        pub const SPAN_SPAN_ID: u64 = 2;
        pub const SPAN_TRACE_STATE: u64 = 3;
        pub const SPAN_PARENT_SPAN_ID: u64 = 4;
        pub const SPAN_NAME: u64 = 5;
        pub const SPAN_KIND: u64 = 6;
        pub const SPAN_START_TIME_UNIX_NANO: u64 = 7;
        pub const SPAN_END_TIME_UNIX_NANO: u64 = 8;
        pub const SPAN_ATTRIBUTES: u64 = 9;
        pub const SPAN_DROPPED_ATTRIBUTES_COUNT: u64 = 10;
        pub const SPAN_EVENTS: u64 = 11;
        pub const SPAN_DROPPED_EVENTS_COUNT: u64 = 12;
        pub const SPAN_LINKS: u64 = 13;
        pub const SPAN_DROPPED_LINKS_COUNT: u64 = 14;
        pub const SPAN_STATUS: u64 = 15;
        pub const SPAN_FLAGS: u64 = 16;

        pub const SPAN_EVENT_TIME_UNIX_NANO: u64 = 1;
        pub const SPAN_EVENT_NAME: u64 = 2;
        pub const SPAN_EVENT_ATTRIBUTES: u64 = 3;
        pub const SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS: u64 = 4;

        pub const SPAN_LINK_TRACE_ID: u64 = 1;
        pub const SPAN_LINK_SPAN_ID: u64 = 2;
        pub const SPAN_LINK_TRACE_STATE: u64 = 3;
        pub const SPAN_LINK_ATTRIBUTES: u64 = 4;
        pub const SPAN_LINK_DROPPED_ATTRIBUTES_COUNT: u64 = 5;
        pub const SPAN_LINK_FLAGS: u64 = 6;

        pub const SPAN_STATUS_MESSAGE: u64 = 2;
        pub const SPAN_STATUS_CODE: u64 = 3;
    }
}
