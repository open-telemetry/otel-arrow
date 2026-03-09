// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub const ID: &str = "id";
pub const PARENT_ID: &str = "parent_id";

pub const METRIC_TYPE: &str = "metric_type";

pub const RESOURCE_METRICS: &str = "resource_metrics";
pub const TIME_UNIX_NANO: &str = "time_unix_nano";
pub const START_TIME_UNIX_NANO: &str = "start_time_unix_nano";
pub const DURATION_TIME_UNIX_NANO: &str = "duration_time_unix_nano";
pub const OBSERVED_TIME_UNIX_NANO: &str = "observed_time_unix_nano";
pub const SEVERITY_NUMBER: &str = "severity_number";
pub const SEVERITY_TEXT: &str = "severity_text";
pub const DROPPED_ATTRIBUTES_COUNT: &str = "dropped_attributes_count";
pub const DROPPED_EVENTS_COUNT: &str = "dropped_events_count";
pub const DROPPED_LINKS_COUNT: &str = "dropped_links_count";
pub const EVENT_NAME: &str = "event_name";
pub const FLAGS: &str = "flags";
pub const TRACE_ID: &str = "trace_id";
pub const TRACE_STATE: &str = "trace_state";
pub const SPAN_ID: &str = "span_id";
pub const PARENT_SPAN_ID: &str = "parent_span_id";
pub const ATTRIBUTES: &str = "attributes";
pub const RESOURCE: &str = "resource";
pub const SCOPE_METRICS: &str = "scope_metrics";
pub const SCOPE: &str = "scope";
pub const NAME: &str = "name";
pub const KIND: &str = "kind";
pub const VERSION: &str = "version";
pub const BODY: &str = "body";
pub const STATUS: &str = "status";
pub const DESCRIPTION: &str = "description";
pub const UNIT: &str = "unit";
pub const DATA: &str = "data";
pub const STATUS_MESSAGE: &str = "status_message";
pub const STATUS_CODE: &str = "code";
pub const SUMMARY_COUNT: &str = "count";
pub const SUMMARY_SUM: &str = "sum";
pub const SUMMARY_QUANTILE_VALUES: &str = "quantile";
pub const SUMMARY_QUANTILE: &str = "quantile";
pub const SUMMARY_VALUE: &str = "value";
pub const METRIC_VALUE: &str = "value";
pub const INT_VALUE: &str = "int_value";
pub const DOUBLE_VALUE: &str = "double_value";
pub const HISTOGRAM_COUNT: &str = "count";
pub const HISTOGRAM_SUM: &str = "sum";
pub const HISTOGRAM_MIN: &str = "min";
pub const HISTOGRAM_MAX: &str = "max";
pub const HISTOGRAM_BUCKET_COUNTS: &str = "bucket_counts";
pub const HISTOGRAM_EXPLICIT_BOUNDS: &str = "explicit_bounds";
pub const EXP_HISTOGRAM_SCALE: &str = "scale";
pub const EXP_HISTOGRAM_ZERO_COUNT: &str = "zero_count";
pub const EXP_HISTOGRAM_ZERO_THRESHOLD: &str = "zero_threshold";
pub const EXP_HISTOGRAM_POSITIVE: &str = "positive";
pub const EXP_HISTOGRAM_NEGATIVE: &str = "negative";
pub const EXP_HISTOGRAM_OFFSET: &str = "offset";
pub const EXP_HISTOGRAM_BUCKET_COUNTS: &str = "bucket_counts";
pub const SCHEMA_URL: &str = "schema_url";
pub const I64_METRIC_VALUE: &str = "i64";
pub const F64_METRIC_VALUE: &str = "f64";
pub const EXEMPLARS: &str = "exemplars";
pub const IS_MONOTONIC: &str = "is_monotonic";
pub const AGGREGATION_TEMPORALITY: &str = "aggregation_temporality";

pub const ATTRIBUTE_KEY: &str = "key";
pub const ATTRIBUTE_TYPE: &str = "type";
pub const ATTRIBUTE_STR: &str = "str";
pub const ATTRIBUTE_INT: &str = "int";
pub const ATTRIBUTE_DOUBLE: &str = "double";
pub const ATTRIBUTE_BOOL: &str = "bool";
pub const ATTRIBUTE_BYTES: &str = "bytes";
pub const ATTRIBUTE_SER: &str = "ser";

// body sub-fields
pub const BODY_TYPE: &str = "body.type";
pub const BODY_STR: &str = "body.str";
pub const BODY_INT: &str = "body.int";
pub const BODY_DOUBLE: &str = "body.double";
pub const BODY_BOOL: &str = "body.bool";
pub const BODY_BYTES: &str = "body.bytes";
pub const BODY_SER: &str = "body.ser";

// resource sub-fields
pub const RESOURCE_ID: &str = "resource.id";
pub const RESOURCE_DROPPED_ATTRIBUTES_COUNT: &str = "resource.dropped_attributes_count";
pub const RESOURCE_SCHEMA_URL: &str = "resource.schema_url";

// scope sub-fields
pub const SCOPE_ID: &str = "scope.id";
pub const SCOPE_DROPPED_ATTRIBUTES_COUNT: &str = "scope.dropped_attributes_count";
pub const SCOPE_NAME: &str = "scope.name";
pub const SCOPE_VERSION: &str = "scope.version";

// status sub-fields
pub const STATUS_DOT_CODE: &str = "status.code";
pub const STATUS_DOT_STATUS_MESSAGE: &str = "status.status_message";

// quantile sub-fields
pub const QUANTILE_QUANTILE: &str = "quantile.quantile";
pub const QUANTILE_VALUE: &str = "quantile.value";

// exp histogram positive/negative sub-fields
pub const POSITIVE_BUCKET_COUNTS: &str = "positive.bucket_counts";
pub const POSITIVE_OFFSET: &str = "positive.offset";
pub const NEGATIVE_BUCKET_COUNTS: &str = "negative.bucket_counts";
pub const NEGATIVE_OFFSET: &str = "negative.offset";

pub mod metadata {
    /// schema metadata for which columns the record batch is sorted by
    pub const SORT_COLUMNS: &str = "sort_columns";

    /// field metadata key for the encoding of some column
    pub const COLUMN_ENCODING: &str = "encoding";

    pub mod encodings {
        /// delta encoding
        pub const DELTA: &str = "delta";

        /// plain encoding - e.g. the values in the array are not encoded
        pub const PLAIN: &str = "plain";

        /// quasi-delta encoding - in this encoding scheme subsequent runs of matching columns
        /// will have the parent_id field delta encoded.
        pub const QUASI_DELTA: &str = "quasidelta";
    }
}
