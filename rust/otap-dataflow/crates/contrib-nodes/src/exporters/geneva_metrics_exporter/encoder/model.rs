// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Metrics ingestion protocol version 6 data model and flags.

use thiserror::Error;

pub(super) const PROTOCOL_VERSION: u16 = 6;
pub(super) const TYPE_SERIALIZER_FLAGS: u32 = 0x1202_0000;
// The checksum covers the serializer flags onward, excluding the version and CRC fields.
pub(super) const CRC_INPUT_OFFSET: usize = size_of::<u16>() + size_of::<u32>();

/// Sampling type flag for a minimum value.
pub const MIN: u32 = 1 << 0;
/// Sampling type flag for a maximum value.
pub const MAX: u32 = 1 << 1;
/// Sampling type flag for a sum value.
pub const SUM: u32 = 1 << 2;
/// Sampling type flag for a count value.
pub const COUNT: u32 = 1 << 4;
/// Sampling type flag for histogram data.
pub const HISTOGRAM: u32 = 1 << 5;
/// Sampling type flag for HyperLogLog sketch data.
pub const HYPER_LOG_LOG_SKETCH: u32 = 1 << 6;
/// Sampling type flag for raw data.
pub const IS_RAW_DATA: u32 = 1 << 15;
/// Sampling type flag for exemplar data.
pub const EXEMPLAR: u32 = 1 << 19;
/// Sampling type flag for millisecond timestamp precision.
pub const HIGH_RESOLUTION_TIMESTAMP: u32 = 1 << 20;
/// Sampling type flag for double values.
pub const DOUBLE_VALUE_TYPE: u32 = 1 << 9;
/// Sampling type flag for doubles encoded as signed base-128 integers.
pub const DOUBLE_VALUE_STORED_AS_LONG_TYPE: u32 = 1 << 10;
/// Sampling type mask.
pub const METRIC_TYPE_MASK: u32 = 0x07c0_0000;
/// Gauge metric type.
pub const METRIC_TYPE_GAUGE: u32 = 0x0040_0000;
/// Cumulative up-down counter metric type.
pub const METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER: u32 = 0x0240_0000;
/// Delta up-down counter metric type.
pub const METRIC_TYPE_DELTA_UP_DOWN_COUNTER: u32 = 0x0340_0000;
/// Cumulative counter metric type.
pub const METRIC_TYPE_CUMULATIVE_COUNTER: u32 = 0x00c0_0000;
/// Delta counter metric type.
pub const METRIC_TYPE_DELTA_COUNTER: u32 = 0x01c0_0000;
/// Cumulative explicit histogram metric type.
pub const METRIC_TYPE_CUMULATIVE_HISTOGRAM: u32 = 0x0080_0000;
/// Delta explicit histogram metric type.
pub const METRIC_TYPE_DELTA_HISTOGRAM: u32 = 0x0180_0000;
/// Cumulative exponential histogram metric type.
pub const METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM: u32 = 0x0280_0000;
/// Delta exponential histogram metric type.
pub const METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM: u32 = 0x0380_0000;
/// OpenTelemetry metric origin.
pub const METRIC_ORIGIN_OPEN_TELEMETRY: u32 = 0x1000_0000;

pub(super) const HISTOGRAM_FORMAT_DOUBLE: u32 = 0x2000_0000;
pub(super) const HISTOGRAM_FORMAT_EXPONENTIAL: u32 = 0x4000_0000;
pub(super) const HISTOGRAM_FORMAT_CUMULATIVE: u32 = 0x8000_0000;
pub(super) const HISTOGRAM_SIZE_MASK: u32 = 0x0fff_ffff;

pub(super) const EXPONENTIAL_POSITIVE_RANGE: u8 = 1 << 0;
pub(super) const EXPONENTIAL_ZERO_RANGE: u8 = 1 << 3;
pub(super) const EXPONENTIAL_NEGATIVE_RANGE: u8 = 1 << 4;

pub(super) const EXEMPLAR_VALUE_STORED_AS_LONG: u8 = 1 << 0;
pub(super) const EXEMPLAR_TIMESTAMP_AVAILABLE: u8 = 1 << 1;
pub(super) const EXEMPLAR_SPAN_ID_EXISTS: u8 = 1 << 2;
pub(super) const EXEMPLAR_TRACE_ID_EXISTS: u8 = 1 << 3;
pub(super) const EXEMPLAR_SAMPLE_COUNT_EXISTS: u8 = 1 << 4;

/// A complete Geneva Metrics ingestion protocol version 6 packet.
#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    /// Serialization time as whole seconds since the .NET epoch.
    pub current_time_bucket: u64,
    /// Metric entries in packet order.
    pub metrics: Vec<Metric>,
}

/// A metric and its associated metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    /// Metric timestamp as whole seconds since the .NET epoch.
    pub time_bucket: i64,
    /// Metric namespace.
    pub namespace: String,
    /// Metric name.
    pub name: String,
    /// Ordered dimension name/value pairs.
    pub dimensions: Vec<Dimension>,
    /// Geneva Metrics sampling and metric type flags.
    pub sampling_type: u32,
    /// Metric values selected by `sampling_type`.
    pub values: MetricValues,
    /// Exemplars appended after the metric values.
    pub exemplars: Vec<MetricExemplar>,
}

/// A metric dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimension {
    /// Dimension name.
    pub name: String,
    /// Dimension value.
    pub value: String,
}

/// Numeric data carried by a metric.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricValues {
    /// Unsigned integer metric values.
    Unsigned(NumericValues<u64>),
    /// Double metric values.
    Double(NumericValues<f64>),
}

/// Values selected by the metric sampling flags.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericValues<T> {
    /// Minimum value.
    pub min: Option<T>,
    /// Maximum value.
    pub max: Option<T>,
    /// Sum value.
    pub sum: Option<T>,
    /// Sample count, checked against the Geneva Metrics `u32` wire limit during encoding.
    pub count: Option<u64>,
    /// Millisecond component used with high-resolution timestamps.
    pub milliseconds: Option<u32>,
    /// Optional histogram body.
    pub histogram: Option<MetricHistogram>,
}

/// Histogram body associated with a metric.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricHistogram {
    /// Legacy unsigned histogram.
    Raw(Vec<(u64, u32)>),
    /// Explicit histogram with double boundaries.
    Explicit(Vec<(f64, u32)>),
    /// Exponential histogram.
    Exponential(ExponentialHistogram),
}

/// Sparse exponential histogram data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExponentialHistogram {
    /// Exponential histogram scale.
    pub scale: i8,
    /// Number of zero values.
    pub zero_count: u64,
    /// Negative buckets in ascending exponent order.
    pub negative: Vec<(i32, u64)>,
    /// Positive buckets in ascending exponent order.
    pub positive: Vec<(i32, u64)>,
}

/// Exemplar associated with a metric.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricExemplar {
    /// Exemplar numeric value.
    pub value: f64,
    /// Optional Unix timestamp in nanoseconds.
    pub time_unix_nano: Option<u64>,
    /// Optional trace identifier.
    pub trace_id: Option<[u8; 16]>,
    /// Optional span identifier.
    pub span_id: Option<[u8; 8]>,
    /// Optional representative sample count.
    pub sample_count: Option<f64>,
    /// Filtered attributes in input order.
    pub filtered_attributes: Vec<(String, String)>,
}

/// Geneva Metrics ingestion packet encoding failure.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EncodeError {
    /// A sampling flag requires a value that was not supplied.
    #[error("sampling flag {flag:#x} requires {field}")]
    MissingValue {
        /// Sampling flag selecting the field.
        flag: u32,
        /// Required field name.
        field: &'static str,
    },
    /// A sampling flag is not supported by this encoder.
    #[error("sampling flag {flag:#x} is not supported")]
    UnsupportedSamplingFlag {
        /// Unsupported sampling flag.
        flag: u32,
    },
    /// A sampling flag requires another sampling flag.
    #[error("sampling flag {flag:#x} requires sampling flag {required_flag:#x}")]
    MissingRequiredSamplingFlag {
        /// Sampling flag with the requirement.
        flag: u32,
        /// Required companion sampling flag.
        required_flag: u32,
    },
    /// A sampling type omits flags required for every metric.
    #[error("sampling type {sampling_type:#x} must include flags {required_flags:#x}")]
    MissingRequiredSamplingFlags {
        /// Supplied sampling type.
        sampling_type: u32,
        /// Required sampling flags.
        required_flags: u32,
    },
    /// A selected double value is not supported by ME.
    #[error("{field} double value with bits {bits:#018x} is not supported")]
    InvalidDoubleValue {
        /// Metric value field name.
        field: &'static str,
        /// IEEE 754 bit representation.
        bits: u64,
    },
    /// A histogram body is incompatible with the metric sampling type.
    #[error("{histogram} histogram is incompatible with sampling type {sampling_type:#x}")]
    IncompatibleHistogram {
        /// Histogram body type.
        histogram: &'static str,
        /// Normalized sampling type.
        sampling_type: u32,
    },
    /// A fixed-width protocol length cannot represent the encoded body.
    #[error("{field} length {length} exceeds protocol maximum {maximum}")]
    LengthOverflow {
        /// Length field name.
        field: &'static str,
        /// Actual encoded length.
        length: usize,
        /// Maximum representable length.
        maximum: usize,
    },
    /// Packet metadata count exceeds the fixed-width field.
    #[error("metric count {0} exceeds protocol maximum")]
    MetricCountOverflow(usize),
    /// Packet offsets cannot be represented by the protocol.
    #[error("packet offset {0} exceeds protocol maximum")]
    OffsetOverflow(usize),
    /// A dictionary index cannot be represented by the protocol.
    #[error("dictionary entry count {0} exceeds protocol maximum")]
    DictionaryCountOverflow(usize),
    /// A numeric value cannot be represented by its protocol field.
    #[error("{field} value {value} exceeds protocol maximum {maximum}")]
    ValueOverflow {
        /// Protocol field name.
        field: &'static str,
        /// Supplied value.
        value: u64,
        /// Maximum representable value.
        maximum: u64,
    },
    /// The packet-to-metric timestamp difference cannot be encoded.
    #[error(
        "time difference between packet bucket {current_time_bucket} and metric bucket {metric_time_bucket} exceeds signed 64-bit range"
    )]
    TimeDifferenceOverflow {
        /// Packet serialization time bucket.
        current_time_bucket: u64,
        /// Metric time bucket.
        metric_time_bucket: i64,
    },
    /// A metric timestamp predates the .NET epoch used by the protocol.
    #[error("metric time bucket {0} predates the .NET epoch")]
    NegativeTimeBucket(i64),
    /// A dimension value contains a NUL character rejected by ME.
    #[error("metric {metric_index} dimension {dimension_index} value contains NUL")]
    InvalidDimensionValue {
        /// Metric index within the packet.
        metric_index: usize,
        /// Dimension index within the metric.
        dimension_index: usize,
    },
    /// A metric exceeds the Geneva dimension limit.
    #[error("metric {metric_index} contains {count} dimensions, exceeding maximum {maximum}")]
    DimensionCountOverflow {
        /// Metric index within the packet.
        metric_index: usize,
        /// Supplied dimension count.
        count: usize,
        /// Maximum supported dimension count.
        maximum: usize,
    },
    /// A packet or metric timestamp cannot be reconstructed as ME ticks.
    #[error(
        "{field} time bucket {time_bucket} with {milliseconds} milliseconds exceeds ME tick range"
    )]
    TimestampOutOfRange {
        /// Timestamp field name.
        field: &'static str,
        /// Whole-second bucket since the .NET epoch.
        time_bucket: u64,
        /// Additional millisecond component.
        milliseconds: u32,
    },
    /// A string exceeds an ME ingestion limit.
    #[error("{field} length {length} exceeds ME maximum {maximum}")]
    StringLengthOverflow {
        /// String field name.
        field: &'static str,
        /// Supplied UTF-16 code-unit count.
        length: usize,
        /// Maximum supported UTF-16 code-unit count.
        maximum: usize,
    },
    /// Encoded length arithmetic overflowed.
    #[error("{field} encoded length exceeds platform range")]
    LengthCalculationOverflow {
        /// Encoded field name.
        field: &'static str,
    },
    /// An exponential histogram scale is outside ME's supported range.
    #[error("exponential histogram scale {scale} is outside supported range {minimum}..={maximum}")]
    ExponentialHistogramScaleOutOfRange {
        /// Supplied histogram scale.
        scale: i8,
        /// Minimum supported scale.
        minimum: i8,
        /// Maximum supported scale.
        maximum: i8,
    },
    /// An exponential histogram range exceeds the Geneva backend bucket limit.
    #[error(
        "exponential histogram {range} range contains {count} buckets, exceeding backend maximum {maximum}"
    )]
    ExponentialHistogramBucketCountOverflow {
        /// Histogram range name.
        range: &'static str,
        /// Number of serialized non-zero buckets.
        count: usize,
        /// Maximum supported buckets per range.
        maximum: usize,
    },
    /// An exponential histogram bucket total overflowed widened arithmetic.
    #[error("exponential histogram bucket total exceeds unsigned 128-bit range")]
    ExponentialHistogramBucketTotalOverflow,
    /// An exponential histogram scalar count differs from its bucket total.
    #[error("exponential histogram count {count} does not match bucket total {bucket_count}")]
    ExponentialHistogramCountMismatch {
        /// Metric scalar count.
        count: u64,
        /// Sum of zero, positive, and negative bucket counts.
        bucket_count: u128,
    },
    /// The difference between exponential histogram bucket counts is not ME-compatible.
    #[error(
        "difference between exponential histogram bucket counts {previous_count} and {count} cannot be encoded as an ME-compatible signed 64-bit value"
    )]
    ExponentialHistogramCountDeltaOverflow {
        /// Previous non-zero bucket count.
        previous_count: u64,
        /// Current non-zero bucket count.
        count: u64,
    },
    /// A histogram delta cannot be represented by its signed 32-bit protocol field.
    #[error("{field} delta {delta} exceeds signed 32-bit range")]
    HistogramDeltaOverflow {
        /// Histogram field name.
        field: &'static str,
        /// Calculated delta.
        delta: i64,
    },
}
