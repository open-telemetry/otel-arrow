// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ME-to-FE protocol version 6 packet encoder.

use std::collections::HashMap;

use thiserror::Error;

const PROTOCOL_VERSION: u16 = 6;
const TYPE_SERIALIZER_FLAGS: u32 = 0x1202_0000;
const CRC_OFFSET: usize = size_of::<u16>() + size_of::<u32>();

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

const HISTOGRAM_FORMAT_DOUBLE: u32 = 0x2000_0000;
const HISTOGRAM_FORMAT_EXPONENTIAL: u32 = 0x4000_0000;
const HISTOGRAM_FORMAT_CUMULATIVE: u32 = 0x8000_0000;
const HISTOGRAM_SIZE_MASK: u32 = 0x0fff_ffff;

const EXPONENTIAL_POSITIVE_RANGE: u8 = 1 << 0;
const EXPONENTIAL_ZERO_RANGE: u8 = 1 << 3;
const EXPONENTIAL_NEGATIVE_RANGE: u8 = 1 << 4;

const EXEMPLAR_VALUE_STORED_AS_LONG: u8 = 1 << 0;
const EXEMPLAR_TIMESTAMP_AVAILABLE: u8 = 1 << 1;
const EXEMPLAR_SPAN_ID_EXISTS: u8 = 1 << 2;
const EXEMPLAR_TRACE_ID_EXISTS: u8 = 1 << 3;
const EXEMPLAR_SAMPLE_COUNT_EXISTS: u8 = 1 << 4;

/// A complete ME-to-FE protocol version 6 packet.
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
    /// ME sampling and metric type flags.
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
    /// Sample count.
    pub count: Option<u32>,
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

/// ME-to-FE packet encoding failure.
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
}

/// Encodes a packet using ME-to-FE protocol version 6.
pub fn encode(packet: &Packet) -> Result<Vec<u8>, EncodeError> {
    let metric_count = u32::try_from(packet.metrics.len())
        .map_err(|_| EncodeError::MetricCountOverflow(packet.metrics.len()))?;
    let mut writer = Writer::default();
    writer.write_u16(PROTOCOL_VERSION);
    let crc_position = writer.reserve(size_of::<u32>());
    writer.write_u32(TYPE_SERIALIZER_FLAGS);
    let metadata_offset_position = writer.reserve(size_of::<u64>());
    let string_offset_position = writer.reserve(size_of::<u64>());

    let mut metadata_table = OrderedInterner::<MetadataKey>::default();
    let mut string_table = OrderedInterner::<String>::default();

    writer.write_unsigned_base128(packet.current_time_bucket);
    writer.write_u32(metric_count);

    for metric in &packet.metrics {
        let metadata = MetadataKey {
            namespace: metric.namespace.clone(),
            name: metric.name.clone(),
            dimension_names: metric
                .dimensions
                .iter()
                .filter(|dimension| !dimension.name.is_empty())
                .map(|dimension| dimension.name.clone())
                .collect(),
        };
        let metadata_is_new = !metadata_table.contains(&metadata);
        let metadata_index = metadata_table.intern(metadata);
        if metadata_is_new {
            let metadata = &metadata_table.values()[metadata_index as usize];
            let _ = string_table.intern(metadata.namespace.clone());
            let _ = string_table.intern(metadata.name.clone());
            for dimension_name in &metadata.dimension_names {
                let _ = string_table.intern(dimension_name.clone());
            }
        }
        writer.write_unsigned_base128(metadata_index as u64);
        let time_difference =
            i128::from(packet.current_time_bucket) - i128::from(metric.time_bucket);
        writer.write_signed_base128(i64::try_from(time_difference).map_err(|_| {
            EncodeError::TimeDifferenceOverflow {
                current_time_bucket: packet.current_time_bucket,
                metric_time_bucket: metric.time_bucket,
            }
        })?);

        for dimension in metric
            .dimensions
            .iter()
            .filter(|dimension| !dimension.name.is_empty())
        {
            let string_index = string_table.intern(dimension.value.clone());
            writer.write_unsigned_base128(string_index as u64);
        }

        write_metric(&mut writer, metric)?;
    }

    let metadata_offset =
        u64::try_from(writer.len()).map_err(|_| EncodeError::OffsetOverflow(writer.len()))?;
    writer.write_u64_at(metadata_offset_position, metadata_offset);
    writer.write_unsigned_base128(metadata_table.len() as u64);

    for metadata in metadata_table.values() {
        let namespace_index = string_table.intern(metadata.namespace.clone());
        let name_index = string_table.intern(metadata.name.clone());
        writer.write_unsigned_base128(namespace_index as u64);
        writer.write_unsigned_base128(name_index as u64);
        writer.write_unsigned_base128(metadata.dimension_names.len() as u64);
        for dimension_name in &metadata.dimension_names {
            let dimension_index = string_table.intern(dimension_name.clone());
            writer.write_unsigned_base128(dimension_index as u64);
        }
    }

    let string_offset =
        u64::try_from(writer.len()).map_err(|_| EncodeError::OffsetOverflow(writer.len()))?;
    writer.write_u64_at(string_offset_position, string_offset);
    writer.write_unsigned_base128(string_table.len() as u64);
    for value in string_table.values() {
        writer.write_unsigned_base128(value.len() as u64);
        writer.write_bytes(value.as_bytes());
    }

    let crc = crc32(&writer.bytes()[CRC_OFFSET..]);
    writer.write_u32_at(crc_position, crc);
    Ok(writer.finish())
}

fn write_metric(writer: &mut Writer, metric: &Metric) -> Result<(), EncodeError> {
    let mut sampling_type = metric.sampling_type;
    let histogram = metric.values.histogram();
    if histogram.is_none() {
        sampling_type &= !HISTOGRAM;
    }
    if metric.exemplars.is_empty() {
        sampling_type &= !EXEMPLAR;
    }

    match &metric.values {
        MetricValues::Unsigned(values) => {
            sampling_type &= !(DOUBLE_VALUE_TYPE | DOUBLE_VALUE_STORED_AS_LONG_TYPE);
            writer.write_unsigned_base128(sampling_type as u64);
            write_unsigned_values(writer, sampling_type, values)?;
        }
        MetricValues::Double(values) => {
            sampling_type |= DOUBLE_VALUE_TYPE;
            sampling_type &= !DOUBLE_VALUE_STORED_AS_LONG_TYPE;
            if can_store_double_values_as_long(sampling_type, values) {
                sampling_type |= DOUBLE_VALUE_STORED_AS_LONG_TYPE;
            }
            writer.write_unsigned_base128(sampling_type as u64);
            write_double_values(writer, sampling_type, values)?;
        }
    }

    if sampling_type & EXEMPLAR != 0 {
        write_exemplars(writer, &metric.exemplars)?;
    }
    Ok(())
}

fn write_unsigned_values(
    writer: &mut Writer,
    sampling_type: u32,
    values: &NumericValues<u64>,
) -> Result<(), EncodeError> {
    if sampling_type & MIN != 0 {
        writer.write_unsigned_base128(required(values.min, MIN, "min")?);
    }
    if sampling_type & MAX != 0 {
        writer.write_unsigned_base128(required(values.max, MAX, "max")?);
    }
    if sampling_type & SUM != 0 {
        writer.write_unsigned_base128(required(values.sum, SUM, "sum")?);
    }
    write_count_and_histogram(writer, sampling_type, values)
}

fn write_double_values(
    writer: &mut Writer,
    sampling_type: u32,
    values: &NumericValues<f64>,
) -> Result<(), EncodeError> {
    let stored_as_long = sampling_type & DOUBLE_VALUE_STORED_AS_LONG_TYPE != 0;
    if sampling_type & MIN != 0 {
        write_double_or_long(writer, required(values.min, MIN, "min")?, stored_as_long);
    }
    if sampling_type & MAX != 0 {
        write_double_or_long(writer, required(values.max, MAX, "max")?, stored_as_long);
    }
    if sampling_type & SUM != 0 {
        write_double_or_long(writer, required(values.sum, SUM, "sum")?, stored_as_long);
    }
    write_count_and_histogram(writer, sampling_type, values)
}

fn write_count_and_histogram<T>(
    writer: &mut Writer,
    sampling_type: u32,
    values: &NumericValues<T>,
) -> Result<(), EncodeError> {
    if sampling_type & HIGH_RESOLUTION_TIMESTAMP != 0 {
        writer.write_unsigned_base128(1);
        writer.write_unsigned_base128(required(
            values.milliseconds,
            HIGH_RESOLUTION_TIMESTAMP,
            "milliseconds",
        )? as u64);
    } else if sampling_type & COUNT != 0 {
        writer.write_unsigned_base128(required(values.count, COUNT, "count")? as u64);
    }

    if sampling_type & HISTOGRAM != 0 {
        write_histogram(
            writer,
            required(values.histogram.as_ref(), HISTOGRAM, "histogram")?,
            sampling_type,
        )?;
    }
    Ok(())
}

fn write_histogram(
    writer: &mut Writer,
    histogram: &MetricHistogram,
    sampling_type: u32,
) -> Result<(), EncodeError> {
    let cumulative = matches!(
        sampling_type & METRIC_TYPE_MASK,
        METRIC_TYPE_CUMULATIVE_HISTOGRAM | METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM
    );
    match histogram {
        MetricHistogram::Raw(buckets) => write_raw_histogram(writer, buckets),
        MetricHistogram::Explicit(buckets) => write_explicit_histogram(writer, buckets, cumulative),
        MetricHistogram::Exponential(histogram) => {
            write_exponential_histogram(writer, histogram, cumulative)
        }
    }
}

fn write_raw_histogram(writer: &mut Writer, buckets: &[(u64, u32)]) -> Result<(), EncodeError> {
    let prefix_position = writer.reserve(size_of::<u32>());
    writer.write_unsigned_base128(buckets.len() as u64);
    let mut previous = None;
    for &(key, count) in buckets {
        if let Some((previous_key, previous_count)) = previous {
            writer.write_unsigned_base128(key.wrapping_sub(previous_key));
            writer.write_signed_base128(i64::from(count) - i64::from(previous_count));
        } else {
            writer.write_unsigned_base128(key);
            writer.write_unsigned_base128(count as u64);
        }
        previous = Some((key, count));
    }
    finish_histogram_prefix(writer, prefix_position, 0)
}

fn write_explicit_histogram(
    writer: &mut Writer,
    buckets: &[(f64, u32)],
    cumulative: bool,
) -> Result<(), EncodeError> {
    let prefix_position = writer.reserve(size_of::<u32>());
    writer.write_unsigned_base128(buckets.len() as u64);
    let mut previous_count = None;
    for &(boundary, count) in buckets {
        writer.write_f64(boundary);
        if let Some(previous_count) = previous_count {
            writer.write_signed_base128(i64::from(count) - i64::from(previous_count));
        } else {
            writer.write_unsigned_base128(count as u64);
        }
        previous_count = Some(count);
    }
    let format = HISTOGRAM_FORMAT_DOUBLE
        | if cumulative {
            HISTOGRAM_FORMAT_CUMULATIVE
        } else {
            0
        };
    finish_histogram_prefix(writer, prefix_position, format)
}

fn write_exponential_histogram(
    writer: &mut Writer,
    histogram: &ExponentialHistogram,
    cumulative: bool,
) -> Result<(), EncodeError> {
    let prefix_position = writer.reserve(size_of::<u32>());
    writer.write_u8(histogram.scale as u8);
    let mut distribution = 0;
    if histogram.zero_count > 0 {
        distribution |= EXPONENTIAL_ZERO_RANGE;
    }
    let negative_count = histogram
        .negative
        .iter()
        .filter(|(_, count)| *count != 0)
        .count();
    let positive_count = histogram
        .positive
        .iter()
        .filter(|(_, count)| *count != 0)
        .count();
    if negative_count != 0 {
        distribution |= EXPONENTIAL_NEGATIVE_RANGE;
    }
    if positive_count != 0 {
        distribution |= EXPONENTIAL_POSITIVE_RANGE;
    }
    writer.write_u8(distribution);

    if histogram.zero_count > 0 {
        writer.write_unsigned_base128(histogram.zero_count);
    }
    if negative_count != 0 {
        writer.write_unsigned_base128(negative_count as u64);
    }
    if positive_count != 0 {
        writer.write_unsigned_base128(positive_count as u64);
    }

    write_exponential_buckets(writer, histogram.negative.iter().rev().copied());
    write_exponential_buckets(writer, histogram.positive.iter().copied());

    let format = HISTOGRAM_FORMAT_EXPONENTIAL
        | if cumulative {
            HISTOGRAM_FORMAT_CUMULATIVE
        } else {
            0
        };
    finish_histogram_prefix(writer, prefix_position, format)
}

fn write_exponential_buckets(writer: &mut Writer, buckets: impl Iterator<Item = (i32, u64)>) {
    let mut previous = None;
    for (exponent, count) in buckets.filter(|(_, count)| *count != 0) {
        if let Some((previous_exponent, previous_count)) = previous {
            writer.write_signed_base128(i64::from(exponent - previous_exponent));
            writer.write_signed_base128(count.wrapping_sub(previous_count) as i64);
        } else {
            writer.write_signed_base128(i64::from(exponent));
            writer.write_unsigned_base128(count);
        }
        previous = Some((exponent, count));
    }
}

fn finish_histogram_prefix(
    writer: &mut Writer,
    prefix_position: usize,
    format: u32,
) -> Result<(), EncodeError> {
    let length = writer.len() - prefix_position - size_of::<u32>();
    if length > HISTOGRAM_SIZE_MASK as usize {
        return Err(EncodeError::LengthOverflow {
            field: "histogram",
            length,
            maximum: HISTOGRAM_SIZE_MASK as usize,
        });
    }
    writer.write_u32_at(prefix_position, format | length as u32);
    Ok(())
}

fn write_exemplars(writer: &mut Writer, exemplars: &[MetricExemplar]) -> Result<(), EncodeError> {
    let list_length_position = writer.reserve(size_of::<u16>());
    let list_body_position = writer.len();
    writer.write_u8(0);
    writer.write_signed_base128(exemplars.len() as i64);
    for exemplar in exemplars {
        write_exemplar(writer, exemplar)?;
    }
    let length = writer.len() - list_body_position;
    let length = u16::try_from(length).map_err(|_| EncodeError::LengthOverflow {
        field: "exemplar list",
        length,
        maximum: u16::MAX as usize,
    })?;
    writer.write_u16_at(list_length_position, length);
    Ok(())
}

fn write_exemplar(writer: &mut Writer, exemplar: &MetricExemplar) -> Result<(), EncodeError> {
    let start_position = writer.len();
    writer.write_u8(0);
    let length_position = writer.reserve(size_of::<u8>());

    let stored_as_long = serializable_as_i64(exemplar.value);
    let mut flags = if stored_as_long {
        EXEMPLAR_VALUE_STORED_AS_LONG
    } else {
        0
    };
    if exemplar.time_unix_nano.is_some() {
        flags |= EXEMPLAR_TIMESTAMP_AVAILABLE;
    }
    if exemplar.span_id.is_some() {
        flags |= EXEMPLAR_SPAN_ID_EXISTS;
    }
    if exemplar.trace_id.is_some() {
        flags |= EXEMPLAR_TRACE_ID_EXISTS;
    }
    if exemplar.sample_count.is_some_and(|count| count != 0.0) {
        flags |= EXEMPLAR_SAMPLE_COUNT_EXISTS;
    }
    writer.write_u8(flags);
    write_double_or_long(writer, exemplar.value, stored_as_long);

    let label_count = u8::try_from(exemplar.filtered_attributes.len()).map_err(|_| {
        EncodeError::LengthOverflow {
            field: "exemplar label count",
            length: exemplar.filtered_attributes.len(),
            maximum: u8::MAX as usize,
        }
    })?;
    writer.write_u8(label_count);

    if let Some(time_unix_nano) = exemplar.time_unix_nano {
        writer.write_u64(time_unix_nano);
    }
    if let Some(trace_id) = exemplar.trace_id {
        writer.write_bytes(&trace_id);
    }
    if let Some(span_id) = exemplar.span_id {
        writer.write_bytes(&span_id);
    }
    if let Some(sample_count) = exemplar.sample_count.filter(|count| *count != 0.0) {
        writer.write_f64(sample_count);
    }
    for (name, value) in &exemplar.filtered_attributes {
        writer.write_unsigned_base128(name.len() as u64);
        writer.write_bytes(name.as_bytes());
        writer.write_unsigned_base128(value.len() as u64);
        writer.write_bytes(value.as_bytes());
    }

    let length = writer.len() - start_position;
    let length = u8::try_from(length).map_err(|_| EncodeError::LengthOverflow {
        field: "single exemplar",
        length,
        maximum: u8::MAX as usize,
    })?;
    writer.write_u8_at(length_position, length);
    Ok(())
}

fn required<T: Copy>(value: Option<T>, flag: u32, field: &'static str) -> Result<T, EncodeError> {
    value.ok_or(EncodeError::MissingValue { flag, field })
}

fn can_store_double_values_as_long(sampling_type: u32, values: &NumericValues<f64>) -> bool {
    sampling_type & HISTOGRAM == 0
        && (sampling_type & SUM == 0 || values.sum.is_some_and(serializable_as_i64))
        && (sampling_type & MIN == 0 || values.min.is_some_and(serializable_as_i64))
        && (sampling_type & MAX == 0 || values.max.is_some_and(serializable_as_i64))
}

fn serializable_as_i64(value: f64) -> bool {
    value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value < -(i64::MIN as f64)
}

fn write_double_or_long(writer: &mut Writer, value: f64, stored_as_long: bool) {
    if stored_as_long {
        writer.write_signed_base128(value as i64);
    } else {
        writer.write_f64(value);
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct MetadataKey {
    namespace: String,
    name: String,
    dimension_names: Vec<String>,
}

#[derive(Clone, Debug)]
struct OrderedInterner<T> {
    indexes: HashMap<T, u32>,
    values: Vec<T>,
}

impl<T> Default for OrderedInterner<T> {
    fn default() -> Self {
        Self {
            indexes: HashMap::new(),
            values: Vec::new(),
        }
    }
}

impl<T> OrderedInterner<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    fn intern(&mut self, value: T) -> u32 {
        if let Some(index) = self.indexes.get(&value) {
            return *index;
        }
        let index = self.values.len() as u32;
        self.values.push(value.clone());
        let _ = self.indexes.insert(value, index);
        index
    }

    fn contains(&self, value: &T) -> bool {
        self.indexes.contains_key(value)
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn values(&self) -> &[T] {
        &self.values
    }
}

trait HistogramValues {
    fn histogram(&self) -> Option<&MetricHistogram>;
}

impl HistogramValues for MetricValues {
    fn histogram(&self) -> Option<&MetricHistogram> {
        match self {
            Self::Unsigned(values) => values.histogram.as_ref(),
            Self::Double(values) => values.histogram.as_ref(),
        }
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn reserve(&mut self, count: usize) -> usize {
        let position = self.len();
        self.bytes.resize(position + count, 0);
        position
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_f64(&mut self, value: f64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u8_at(&mut self, position: usize, value: u8) {
        self.bytes[position] = value;
    }

    fn write_u16_at(&mut self, position: usize, value: u16) {
        self.bytes[position..position + size_of::<u16>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32_at(&mut self, position: usize, value: u32) {
        self.bytes[position..position + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u64_at(&mut self, position: usize, value: u64) {
        self.bytes[position..position + size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_unsigned_base128(&mut self, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn write_signed_base128(&mut self, value: i64) {
        let negative = value < 0;
        let mut remaining = value.unsigned_abs();
        let mut first = true;
        loop {
            let mut byte = if first {
                let mut byte = (remaining & 0x3f) as u8;
                remaining >>= 6;
                if negative {
                    byte |= 0x40;
                }
                first = false;
                byte
            } else {
                let byte = (remaining & 0x7f) as u8;
                remaining >>= 7;
                byte
            };
            if remaining != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte);
            if remaining == 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_TIME_BUCKET: u64 = 63_524_174_400;
    const DEFAULT_UNIX_SECONDS: u64 = 1_388_577_600;

    fn dimension(name: &str, value: &str) -> Dimension {
        Dimension {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn unsigned_values(sum: u64, count: u32) -> MetricValues {
        MetricValues::Unsigned(NumericValues {
            min: None,
            max: None,
            sum: Some(sum),
            count: Some(count),
            milliseconds: None,
            histogram: None,
        })
    }

    fn double_values(sum: f64, count: u32) -> MetricValues {
        MetricValues::Double(NumericValues {
            min: None,
            max: None,
            sum: Some(sum),
            count: Some(count),
            milliseconds: None,
            histogram: None,
        })
    }

    fn standard_dimensions() -> Vec<Dimension> {
        vec![
            dimension("Dim1Name", "Dim1Value"),
            dimension("Dim2Name", "Dim2Value"),
        ]
    }

    fn standard_metric(values: MetricValues, sampling_type: u32) -> Metric {
        Metric {
            time_bucket: DEFAULT_TIME_BUCKET as i64,
            namespace: "MetricNamespace".to_string(),
            name: "MetricName".to_string(),
            dimensions: standard_dimensions(),
            sampling_type,
            values,
            exemplars: Vec::new(),
        }
    }

    fn assert_fixture(packet: Packet, expected: &[u8]) {
        assert_eq!(encode(&packet).expect("packet should encode"), expected);
    }

    fn read_unsigned_base128(bytes: &[u8]) -> u64 {
        let mut value = 0;
        for (index, byte) in bytes.iter().enumerate() {
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                return value;
            }
        }
        panic!("unterminated base-128 value");
    }

    /// Scenario: Core OTLP gauge and sum metric types are encoded as raw protocol v6 entries.
    /// Guarantees: Sampling flags, value encodings, metadata interning, string interning, offsets, and CRC match the C++ packet.
    #[test]
    fn matches_core_metric_types_fixture() {
        let dimensions = vec![dimension("region", "eastus")];
        let metric = |name: &str, sampling_type: u32, values: MetricValues| Metric {
            time_bucket: DEFAULT_TIME_BUCKET as i64,
            namespace: "Compatibility".to_string(),
            name: name.to_string(),
            dimensions: dimensions.clone(),
            sampling_type,
            values,
            exemplars: Vec::new(),
        };
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![
                metric(
                    "gauge",
                    SUM | COUNT
                        | IS_RAW_DATA
                        | DOUBLE_VALUE_TYPE
                        | METRIC_TYPE_GAUGE
                        | METRIC_ORIGIN_OPEN_TELEMETRY,
                    double_values(12.5, 1),
                ),
                metric(
                    "delta_counter",
                    SUM | COUNT
                        | IS_RAW_DATA
                        | DOUBLE_VALUE_TYPE
                        | METRIC_TYPE_DELTA_COUNTER
                        | METRIC_ORIGIN_OPEN_TELEMETRY,
                    double_values(22.5, 1),
                ),
                metric(
                    "cumulative_counter",
                    SUM | COUNT
                        | IS_RAW_DATA
                        | METRIC_TYPE_CUMULATIVE_COUNTER
                        | METRIC_ORIGIN_OPEN_TELEMETRY,
                    unsigned_values(32, 1),
                ),
                metric(
                    "cumulative_up_down_counter",
                    SUM | COUNT
                        | IS_RAW_DATA
                        | DOUBLE_VALUE_TYPE
                        | METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER
                        | METRIC_ORIGIN_OPEN_TELEMETRY,
                    double_values(-42.5, 1),
                ),
            ],
        };

        assert_fixture(
            packet,
            include_bytes!("fixtures/CompatibilityFixtureCoreMetricTypes.bin"),
        );
    }

    /// Scenario: An unsigned metric contains two dimensions.
    /// Guarantees: Dimension values precede metadata strings in the intern table and match the C++ packet byte-for-byte.
    #[test]
    fn matches_dimension_fixture() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(unsigned_values(100, 1), SUM | COUNT)],
        };

        assert_fixture(packet, include_bytes!("fixtures/MetricWithDimsULong.bin"));
    }

    /// Scenario: A metric timestamp is twelve seconds after the packet serialization timestamp.
    /// Guarantees: The negative signed base-128 time difference and complete packet match the C++ protocol v6 fixture.
    #[test]
    fn matches_timestamp_fixture() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![Metric {
                time_bucket: DEFAULT_TIME_BUCKET as i64 + 12,
                namespace: "MetricNamespace".to_string(),
                name: "MetricName".to_string(),
                dimensions: Vec::new(),
                sampling_type: SUM | COUNT,
                values: unsigned_values(100, 1),
                exemplars: Vec::new(),
            }],
        };

        assert_fixture(
            packet,
            include_bytes!("fixtures/MetricTimeStampProtocol6.bin"),
        );
    }

    /// Scenario: A delta explicit histogram contains one double boundary and one count.
    /// Guarantees: Scalar values, histogram prefix, boundary encoding, dictionaries, and CRC match the C++ packet.
    #[test]
    fn matches_explicit_histogram_fixture() {
        let values = MetricValues::Double(NumericValues {
            min: Some(12.34),
            max: Some(12.34),
            sum: Some(12.34),
            count: Some(1),
            milliseconds: None,
            histogram: Some(MetricHistogram::Explicit(vec![(12.34, 1)])),
        });
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(
                values,
                MIN | MAX
                    | SUM
                    | COUNT
                    | HISTOGRAM
                    | DOUBLE_VALUE_TYPE
                    | METRIC_TYPE_DELTA_HISTOGRAM,
            )],
        };

        assert_fixture(
            packet,
            include_bytes!("fixtures/MinMaxHistoExplicitMetricDouble.bin"),
        );
    }

    /// Scenario: A delta exponential histogram contains one positive bucket.
    /// Guarantees: Scale, distribution flags, sparse bucket encoding, scalar values, and packet framing match C++.
    #[test]
    fn matches_exponential_histogram_fixture() {
        let values = MetricValues::Double(NumericValues {
            min: Some(12.34),
            max: Some(12.34),
            sum: Some(12.34),
            count: Some(1),
            milliseconds: None,
            histogram: Some(MetricHistogram::Exponential(ExponentialHistogram {
                scale: 1,
                zero_count: 0,
                negative: Vec::new(),
                positive: vec![(7, 1)],
            })),
        });
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(
                values,
                MIN | MAX
                    | SUM
                    | COUNT
                    | HISTOGRAM
                    | DOUBLE_VALUE_TYPE
                    | METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM,
            )],
        };

        assert_fixture(
            packet,
            include_bytes!("fixtures/MinMaxHistoExponentialMetricDouble.bin"),
        );
    }

    /// Scenario: An aggregated double metric carries two trace-correlated exemplars.
    /// Guarantees: Exemplar lengths, flags, values, timestamps, identifiers, labels, and surrounding metric packet match C++.
    #[test]
    fn matches_exemplar_fixture() {
        let trace_id = [
            0x5b, 0x8a, 0xa5, 0xa2, 0xd2, 0xc8, 0x72, 0xe8, 0x32, 0x1c, 0xf3, 0x73, 0x08, 0xd6,
            0x9d, 0xf2,
        ];
        let span_id = [0x05, 0x15, 0x81, 0xbf, 0x3c, 0xb5, 0x5c, 0x13];
        let exemplar = |value, seconds, labels: &[(&str, &str)]| MetricExemplar {
            value,
            time_unix_nano: Some((DEFAULT_UNIX_SECONDS + seconds) * 1_000_000_000),
            trace_id: Some(trace_id),
            span_id: Some(span_id),
            sample_count: None,
            filtered_attributes: labels
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        };
        let values = MetricValues::Double(NumericValues {
            min: Some(-1.0),
            max: Some(5.6),
            sum: Some(4.6),
            count: Some(2),
            milliseconds: None,
            histogram: None,
        });
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![Metric {
                time_bucket: DEFAULT_TIME_BUCKET as i64 + 60,
                namespace: "MetricNamespace".to_string(),
                name: "MetricName".to_string(),
                dimensions: vec![
                    dimension("DimensionName1", "DimensionValue11"),
                    dimension("DimensionName2", "DimensionValue21"),
                ],
                sampling_type: MIN | MAX | SUM | COUNT | DOUBLE_VALUE_TYPE | EXEMPLAR,
                values,
                exemplars: vec![
                    exemplar(
                        4.5,
                        5,
                        &[("filtered_attrb1", "val1"), ("filtered_attrb2", "val2")],
                    ),
                    exemplar(
                        -1.4,
                        25,
                        &[("filtered_attrb3", "val3"), ("filtered_attrb4", "val4")],
                    ),
                ],
            }],
        };

        assert_fixture(
            packet,
            include_bytes!("fixtures/DeserializeWithExemplar.bin"),
        );
    }

    /// Scenario: A histogram sampling flag is present without histogram data.
    /// Guarantees: The encoder removes the empty histogram flag like the C++ serializer.
    #[test]
    fn removes_empty_histogram_flag() {
        let metric = Metric {
            time_bucket: DEFAULT_TIME_BUCKET as i64,
            namespace: "namespace".to_string(),
            name: "metric".to_string(),
            dimensions: Vec::new(),
            sampling_type: SUM | COUNT | HISTOGRAM,
            values: unsigned_values(1, 1),
            exemplars: Vec::new(),
        };
        let mut writer = Writer::default();

        write_metric(&mut writer, &metric).expect("metric should encode");
        assert_eq!(
            read_unsigned_base128(writer.bytes()),
            u64::from(SUM | COUNT)
        );
    }

    /// Scenario: Numeric type flags conflict with the typed metric value representation.
    /// Guarantees: Unsigned values clear double flags, while doubles set their type and recalculate integer storage.
    #[test]
    fn normalizes_numeric_type_flags() {
        let unsigned_metric = standard_metric(
            unsigned_values(1, 1),
            SUM | COUNT | DOUBLE_VALUE_TYPE | DOUBLE_VALUE_STORED_AS_LONG_TYPE,
        );
        let double_metric = standard_metric(
            double_values(1.5, 1),
            SUM | COUNT | DOUBLE_VALUE_STORED_AS_LONG_TYPE,
        );
        let mut unsigned_writer = Writer::default();
        let mut double_writer = Writer::default();

        write_metric(&mut unsigned_writer, &unsigned_metric).expect("metric should encode");
        write_metric(&mut double_writer, &double_metric).expect("metric should encode");

        assert_eq!(
            read_unsigned_base128(unsigned_writer.bytes()),
            u64::from(SUM | COUNT)
        );
        assert_eq!(
            read_unsigned_base128(double_writer.bytes()),
            u64::from(SUM | COUNT | DOUBLE_VALUE_TYPE)
        );
    }

    /// Scenario: A sampling flag selects a scalar value that is absent.
    /// Guarantees: Encoding fails explicitly instead of producing a malformed packet.
    #[test]
    fn rejects_missing_required_value() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![Metric {
                time_bucket: DEFAULT_TIME_BUCKET as i64,
                namespace: "namespace".to_string(),
                name: "metric".to_string(),
                dimensions: Vec::new(),
                sampling_type: SUM,
                values: MetricValues::Unsigned(NumericValues {
                    min: None,
                    max: None,
                    sum: None,
                    count: None,
                    milliseconds: None,
                    histogram: None,
                }),
                exemplars: Vec::new(),
            }],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::MissingValue {
                flag: SUM,
                field: "sum"
            })
        );
    }

    /// Scenario: Packet and metric timestamps produce a difference outside the protocol's signed range.
    /// Guarantees: Encoding returns a typed error instead of wrapping or panicking.
    #[test]
    fn rejects_unrepresentable_time_difference() {
        let packet = Packet {
            current_time_bucket: u64::MAX,
            metrics: vec![Metric {
                time_bucket: -1,
                namespace: "namespace".to_string(),
                name: "metric".to_string(),
                dimensions: Vec::new(),
                sampling_type: SUM | COUNT,
                values: unsigned_values(1, 1),
                exemplars: Vec::new(),
            }],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::TimeDifferenceOverflow {
                current_time_bucket: u64::MAX,
                metric_time_bucket: -1,
            })
        );
    }
}
