// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Metrics ingestion protocol version 6 packet encoding.

use std::collections::HashMap;

use super::exemplar::write_exemplars;
use super::histogram::write_histogram;
use super::model::*;
use super::numeric::{can_store_double_values_as_long, required, write_double_or_long};
use super::writer::Writer;

const MAX_DIMENSIONS: usize = 74;
const MAX_METRIC_NAME_LENGTH: usize = 512;
const MAX_DIMENSION_NAME_LENGTH: usize = 512;
const MAX_DIMENSION_VALUE_LENGTH: usize = 1_024;
const STALE_NAN_BITS: u64 = 0x7ff0_0000_0000_0002;
const TICKS_PER_MILLISECOND: u64 = 10_000;
const TICKS_PER_SECOND: u64 = 1_000 * TICKS_PER_MILLISECOND;
const MAX_TIME_BUCKET: u64 = u64::MAX / TICKS_PER_SECOND;

/// Encodes a packet using Geneva Metrics ingestion protocol version 6.
pub fn encode(packet: &Packet) -> Result<Vec<u8>, EncodeError> {
    validate_timestamp("packet", packet.current_time_bucket, 0)?;
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

    for (metric_index, metric) in packet.metrics.iter().enumerate() {
        if metric.time_bucket < 0 {
            return Err(EncodeError::NegativeTimeBucket(metric.time_bucket));
        }
        if metric.dimensions.len() > MAX_DIMENSIONS {
            return Err(EncodeError::DimensionCountOverflow {
                metric_index,
                count: metric.dimensions.len(),
                maximum: MAX_DIMENSIONS,
            });
        }
        validate_string_length("metric name", &metric.name, MAX_METRIC_NAME_LENGTH)?;
        if let Some((dimension_index, _)) = metric
            .dimensions
            .iter()
            .enumerate()
            .find(|(_, dimension)| dimension.value.contains('\0'))
        {
            return Err(EncodeError::InvalidDimensionValue {
                metric_index,
                dimension_index,
            });
        }
        for dimension in &metric.dimensions {
            validate_string_length("dimension name", &dimension.name, MAX_DIMENSION_NAME_LENGTH)?;
            validate_string_length(
                "dimension value",
                &dimension.value,
                MAX_DIMENSION_VALUE_LENGTH,
            )?;
        }
        let metric_time_bucket = metric.time_bucket as u64;
        let milliseconds = if metric.sampling_type & HIGH_RESOLUTION_TIMESTAMP != 0 {
            match &metric.values {
                MetricValues::Unsigned(values) => values.milliseconds,
                MetricValues::Double(values) => values.milliseconds,
            }
            .filter(|milliseconds| *milliseconds <= 999)
            .unwrap_or(0)
        } else {
            0
        };
        validate_timestamp("metric", metric_time_bucket, milliseconds)?;
        let metadata = MetadataKey {
            namespace: metric.namespace.clone(),
            name: metric.name.clone(),
            dimension_names: metric
                .dimensions
                .iter()
                .map(|dimension| dimension.name.clone())
                .collect(),
        };
        let (metadata_index, metadata_is_new) = metadata_table.intern(metadata)?;
        if metadata_is_new {
            let metadata = &metadata_table.values()[metadata_index as usize];
            let _ = string_table.intern(metadata.namespace.clone())?;
            let _ = string_table.intern(metadata.name.clone())?;
            for dimension_name in metadata
                .dimension_names
                .iter()
                .filter(|name| !name.is_empty())
            {
                let _ = string_table.intern(dimension_name.clone())?;
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
            let (string_index, _) = string_table.intern(dimension.value.clone())?;
            writer.write_unsigned_base128(string_index as u64);
        }

        write_metric(&mut writer, metric)?;
    }

    let metadata_offset =
        u64::try_from(writer.len()).map_err(|_| EncodeError::OffsetOverflow(writer.len()))?;
    writer.write_u64_at(metadata_offset_position, metadata_offset);
    writer.write_unsigned_base128(metadata_table.len() as u64);

    for metadata in metadata_table.values() {
        let (namespace_index, _) = string_table.intern(metadata.namespace.clone())?;
        let (name_index, _) = string_table.intern(metadata.name.clone())?;
        writer.write_unsigned_base128(namespace_index as u64);
        writer.write_unsigned_base128(name_index as u64);
        let dimension_count = metadata
            .dimension_names
            .iter()
            .filter(|name| !name.is_empty())
            .count();
        writer.write_unsigned_base128(dimension_count as u64);
        for dimension_name in metadata
            .dimension_names
            .iter()
            .filter(|name| !name.is_empty())
        {
            let (dimension_index, _) = string_table.intern(dimension_name.clone())?;
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

    let crc = crc32fast::hash(&writer.bytes()[CRC_INPUT_OFFSET..]);
    writer.write_u32_at(crc_position, crc);
    Ok(writer.finish())
}

fn validate_timestamp(
    field: &'static str,
    time_bucket: u64,
    milliseconds: u32,
) -> Result<(), EncodeError> {
    let ticks_overflow = time_bucket > MAX_TIME_BUCKET
        || time_bucket
            .checked_mul(TICKS_PER_SECOND)
            .and_then(|ticks| {
                u64::from(milliseconds)
                    .checked_mul(TICKS_PER_MILLISECOND)
                    .and_then(|millisecond_ticks| ticks.checked_add(millisecond_ticks))
            })
            .is_none();
    if ticks_overflow {
        return Err(EncodeError::TimestampOutOfRange {
            field,
            time_bucket,
            milliseconds,
        });
    }
    Ok(())
}

fn validate_string_length(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), EncodeError> {
    let length = value.encode_utf16().count();
    if length > maximum {
        return Err(EncodeError::StringLengthOverflow {
            field,
            length,
            maximum,
        });
    }
    Ok(())
}

pub(super) fn write_metric(writer: &mut Writer, metric: &Metric) -> Result<(), EncodeError> {
    let mut sampling_type = metric.sampling_type;
    let histogram = metric.values.histogram();
    if histogram.is_none()
        || matches!(histogram, Some(MetricHistogram::Raw(buckets)) if buckets.is_empty())
    {
        sampling_type &= !HISTOGRAM;
    }
    if metric.exemplars.is_empty() {
        sampling_type &= !EXEMPLAR;
    }
    if sampling_type & HYPER_LOG_LOG_SKETCH != 0 {
        return Err(EncodeError::UnsupportedSamplingFlag {
            flag: HYPER_LOG_LOG_SKETCH,
        });
    }
    if sampling_type & HIGH_RESOLUTION_TIMESTAMP != 0 && sampling_type & COUNT == 0 {
        return Err(EncodeError::MissingRequiredSamplingFlag {
            flag: HIGH_RESOLUTION_TIMESTAMP,
            required_flag: COUNT,
        });
    }
    let required_flags = SUM | COUNT;
    if sampling_type & required_flags != required_flags {
        return Err(EncodeError::MissingRequiredSamplingFlags {
            sampling_type,
            required_flags,
        });
    }

    match &metric.values {
        MetricValues::Unsigned(_) => {
            sampling_type &= !(DOUBLE_VALUE_TYPE | DOUBLE_VALUE_STORED_AS_LONG_TYPE);
        }
        MetricValues::Double(values) => {
            validate_double_values(sampling_type, values)?;
            sampling_type |= DOUBLE_VALUE_TYPE;
            sampling_type &= !DOUBLE_VALUE_STORED_AS_LONG_TYPE;
            if can_store_double_values_as_long(sampling_type, values) {
                sampling_type |= DOUBLE_VALUE_STORED_AS_LONG_TYPE;
            }
        }
    }
    validate_histogram(sampling_type, histogram)?;
    validate_exponential_histogram_count(sampling_type, &metric.values, histogram)?;

    writer.write_unsigned_base128(sampling_type as u64);
    match &metric.values {
        MetricValues::Unsigned(values) => write_unsigned_values(writer, sampling_type, values)?,
        MetricValues::Double(values) => write_double_values(writer, sampling_type, values)?,
    }

    if sampling_type & EXEMPLAR != 0 {
        write_exemplars(writer, &metric.exemplars)?;
    }
    Ok(())
}

fn validate_double_values(
    sampling_type: u32,
    values: &NumericValues<f64>,
) -> Result<(), EncodeError> {
    for (flag, field, value) in [
        (MIN, "min", values.min),
        (MAX, "max", values.max),
        (SUM, "sum", values.sum),
    ] {
        if sampling_type & flag == 0 {
            continue;
        }
        let value = required(value, flag, field)?;
        if !value.is_finite() && value.to_bits() != STALE_NAN_BITS {
            return Err(EncodeError::InvalidDoubleValue {
                field,
                bits: value.to_bits(),
            });
        }
    }
    Ok(())
}

fn validate_histogram(
    sampling_type: u32,
    histogram: Option<&MetricHistogram>,
) -> Result<(), EncodeError> {
    if sampling_type & HISTOGRAM == 0 {
        return Ok(());
    }

    let metric_type = sampling_type & METRIC_TYPE_MASK;
    let incompatible = match histogram {
        Some(MetricHistogram::Raw(_)) => None,
        Some(MetricHistogram::Explicit(_))
            if sampling_type & DOUBLE_VALUE_TYPE != 0
                && matches!(
                    metric_type,
                    METRIC_TYPE_CUMULATIVE_HISTOGRAM | METRIC_TYPE_DELTA_HISTOGRAM
                ) =>
        {
            None
        }
        Some(MetricHistogram::Explicit(_)) => Some("explicit"),
        Some(MetricHistogram::Exponential(_))
            if matches!(
                metric_type,
                METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM
                    | METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM
            ) =>
        {
            None
        }
        Some(MetricHistogram::Exponential(_)) => Some("exponential"),
        None => None,
    };

    if let Some(histogram) = incompatible {
        return Err(EncodeError::IncompatibleHistogram {
            histogram,
            sampling_type,
        });
    }
    Ok(())
}

fn validate_exponential_histogram_count(
    sampling_type: u32,
    values: &MetricValues,
    histogram: Option<&MetricHistogram>,
) -> Result<(), EncodeError> {
    let Some(MetricHistogram::Exponential(histogram)) = histogram else {
        return Ok(());
    };
    if sampling_type & HISTOGRAM == 0 {
        return Ok(());
    }

    let count = if sampling_type & HIGH_RESOLUTION_TIMESTAMP != 0 {
        1
    } else {
        match values {
            MetricValues::Unsigned(values) => required(values.count, COUNT, "count")?,
            MetricValues::Double(values) => required(values.count, COUNT, "count")?,
        }
    };
    let bucket_count = histogram
        .negative
        .iter()
        .chain(&histogram.positive)
        .try_fold(u128::from(histogram.zero_count), |total, (_, count)| {
            total.checked_add(u128::from(*count))
        })
        .ok_or(EncodeError::ExponentialHistogramBucketTotalOverflow)?;
    if bucket_count != u128::from(count) {
        return Err(EncodeError::ExponentialHistogramCountMismatch {
            count,
            bucket_count,
        });
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
        let milliseconds = required(
            values.milliseconds,
            HIGH_RESOLUTION_TIMESTAMP,
            "milliseconds",
        )?;
        if milliseconds > 999 {
            return Err(EncodeError::ValueOverflow {
                field: "milliseconds",
                value: u64::from(milliseconds),
                maximum: 999,
            });
        }
        writer.write_unsigned_base128(u64::from(milliseconds));
    } else if sampling_type & COUNT != 0 {
        let count = required(values.count, COUNT, "count")?;
        let count = u32::try_from(count).map_err(|_| EncodeError::ValueOverflow {
            field: "count",
            value: count,
            maximum: u64::from(u32::MAX),
        })?;
        writer.write_unsigned_base128(u64::from(count));
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
    fn intern(&mut self, value: T) -> Result<(u32, bool), EncodeError> {
        if let Some(index) = self.indexes.get(&value) {
            return Ok((*index, false));
        }
        let index = u32::try_from(self.values.len())
            .map_err(|_| EncodeError::DictionaryCountOverflow(self.values.len()))?;
        self.values.push(value.clone());
        let _ = self.indexes.insert(value, index);
        Ok((index, true))
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

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;

    fn read_unsigned_base128_at(bytes: &[u8], position: &mut usize) -> u64 {
        let mut value = 0;
        let mut shift = 0;
        loop {
            let byte = bytes[*position];
            *position += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
            shift += 7;
        }
    }

    fn read_u64(bytes: &[u8], position: usize) -> u64 {
        u64::from_le_bytes(
            bytes[position..position + size_of::<u64>()]
                .try_into()
                .expect("u64 field"),
        )
    }

    fn read_string_table(bytes: &[u8]) -> Vec<(usize, String)> {
        let mut position = read_u64(bytes, 18) as usize;
        let count = read_unsigned_base128_at(bytes, &mut position);
        (0..count)
            .map(|_| {
                let length = read_unsigned_base128_at(bytes, &mut position) as usize;
                let value = String::from_utf8(bytes[position..position + length].to_vec())
                    .expect("string table value");
                position += length;
                (length, value)
            })
            .collect()
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

    /// Scenario: An integral double metric supplies an empty legacy raw histogram.
    /// Guarantees: The empty histogram flag is removed before compact double encoding is selected.
    #[test]
    fn removes_empty_raw_histogram_before_numeric_encoding() {
        let metric = standard_metric(
            MetricValues::Double(NumericValues {
                min: None,
                max: None,
                sum: Some(1.0),
                count: Some(1),
                milliseconds: None,
                histogram: Some(MetricHistogram::Raw(Vec::new())),
            }),
            SUM | COUNT | HISTOGRAM,
        );
        let mut writer = Writer::default();

        write_metric(&mut writer, &metric).expect("metric should encode");

        assert_eq!(
            read_unsigned_base128(writer.bytes()),
            u64::from(SUM | COUNT | DOUBLE_VALUE_TYPE | DOUBLE_VALUE_STORED_AS_LONG_TYPE)
        );
    }

    /// Scenario: Histogram bodies contradict their normalized numeric or histogram metric type.
    /// Guarantees: Explicit and exponential bodies are rejected before emitting misleading sampling flags.
    #[test]
    fn rejects_incompatible_histogram_bodies() {
        let cases = [
            (
                standard_metric(
                    MetricValues::Unsigned(NumericValues {
                        min: None,
                        max: None,
                        sum: Some(1),
                        count: Some(1),
                        milliseconds: None,
                        histogram: Some(MetricHistogram::Explicit(vec![(1.0, 1)])),
                    }),
                    SUM | COUNT | HISTOGRAM | METRIC_TYPE_DELTA_HISTOGRAM,
                ),
                "explicit",
                SUM | COUNT | HISTOGRAM | METRIC_TYPE_DELTA_HISTOGRAM,
            ),
            (
                standard_metric(
                    MetricValues::Double(NumericValues {
                        min: None,
                        max: None,
                        sum: Some(1.0),
                        count: Some(1),
                        milliseconds: None,
                        histogram: Some(MetricHistogram::Exponential(ExponentialHistogram {
                            scale: 0,
                            zero_count: 0,
                            negative: Vec::new(),
                            positive: vec![(0, 1)],
                        })),
                    }),
                    SUM | COUNT | HISTOGRAM | METRIC_TYPE_DELTA_HISTOGRAM,
                ),
                "exponential",
                SUM | COUNT | HISTOGRAM | DOUBLE_VALUE_TYPE | METRIC_TYPE_DELTA_HISTOGRAM,
            ),
        ];

        for (metric, histogram, sampling_type) in cases {
            let mut writer = Writer::default();
            assert_eq!(
                write_metric(&mut writer, &metric),
                Err(EncodeError::IncompatibleHistogram {
                    histogram,
                    sampling_type,
                })
            );
            assert!(writer.bytes().is_empty());
        }
    }

    /// Scenario: An exponential histogram scalar count differs from the sum of its buckets.
    /// Guarantees: Encoding rejects the inconsistent histogram before emitting metric bytes.
    #[test]
    fn rejects_exponential_histogram_count_mismatch() {
        let metric = standard_metric(
            MetricValues::Double(NumericValues {
                min: None,
                max: None,
                sum: Some(1.0),
                count: Some(1),
                milliseconds: None,
                histogram: Some(MetricHistogram::Exponential(ExponentialHistogram {
                    scale: 0,
                    zero_count: 2,
                    negative: Vec::new(),
                    positive: Vec::new(),
                })),
            }),
            SUM | COUNT | HISTOGRAM | METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM,
        );
        let mut writer = Writer::default();

        assert_eq!(
            write_metric(&mut writer, &metric),
            Err(EncodeError::ExponentialHistogramCountMismatch {
                count: 1,
                bucket_count: 2,
            })
        );
        assert!(writer.bytes().is_empty());
    }

    /// Scenario: An exponential histogram bucket total exceeds the scalar count's unsigned 64-bit range.
    /// Guarantees: Widened summation reports the exact total instead of wrapping before comparison.
    #[test]
    fn rejects_exponential_histogram_bucket_total_above_u64_range() {
        let metric = standard_metric(
            MetricValues::Double(NumericValues {
                min: None,
                max: None,
                sum: Some(1.0),
                count: Some(u64::MAX),
                milliseconds: None,
                histogram: Some(MetricHistogram::Exponential(ExponentialHistogram {
                    scale: 0,
                    zero_count: u64::MAX,
                    negative: Vec::new(),
                    positive: vec![(0, 1)],
                })),
            }),
            SUM | COUNT | HISTOGRAM | METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM,
        );
        let mut writer = Writer::default();

        assert_eq!(
            write_metric(&mut writer, &metric),
            Err(EncodeError::ExponentialHistogramCountMismatch {
                count: u64::MAX,
                bucket_count: u128::from(u64::MAX) + 1,
            })
        );
        assert!(writer.bytes().is_empty());
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

    /// Scenario: Selected metric doubles contain generic NaN or positive/negative infinity.
    /// Guarantees: Encoding rejects every non-finite value that is not ME's stale-NaN sentinel.
    #[test]
    fn rejects_unsupported_non_finite_metric_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let metric = standard_metric(double_values(value, 1), SUM | COUNT);
            let mut writer = Writer::default();

            assert_eq!(
                write_metric(&mut writer, &metric),
                Err(EncodeError::InvalidDoubleValue {
                    field: "sum",
                    bits: value.to_bits(),
                })
            );
            assert!(writer.bytes().is_empty());
        }
    }

    /// Scenario: A selected metric sum uses ME's exact Prometheus stale-NaN representation.
    /// Guarantees: The sentinel remains accepted and is serialized with its original IEEE 754 bits.
    #[test]
    fn preserves_me_stale_nan_metric_value() {
        let value = f64::from_bits(STALE_NAN_BITS);
        let metric = standard_metric(double_values(value, 1), SUM | COUNT);
        let mut writer = Writer::default();

        write_metric(&mut writer, &metric).expect("stale NaN should encode");

        let mut position = 0;
        let sampling_type = read_unsigned_base128_at(writer.bytes(), &mut position) as u32;
        assert_eq!(
            sampling_type,
            SUM | COUNT | DOUBLE_VALUE_TYPE,
            "stale NaN must not use compact integer storage"
        );
        assert_eq!(
            u64::from_le_bytes(
                writer.bytes()[position..position + size_of::<u64>()]
                    .try_into()
                    .expect("double sum"),
            ),
            STALE_NAN_BITS
        );
    }

    /// Scenario: A delta non-monotonic sum uses Geneva's delta up-down-counter metric type.
    /// Guarantees: The encoder preserves the ME metric-type bits instead of dropping or rewriting them.
    #[test]
    fn preserves_delta_up_down_counter_type() {
        let metric = standard_metric(
            unsigned_values(1, 1),
            SUM | COUNT | METRIC_TYPE_DELTA_UP_DOWN_COUNTER,
        );
        let mut writer = Writer::default();

        write_metric(&mut writer, &metric).expect("metric should encode");

        assert_eq!(
            read_unsigned_base128(writer.bytes()),
            u64::from(SUM | COUNT | METRIC_TYPE_DELTA_UP_DOWN_COUNTER)
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
                sampling_type: SUM | COUNT,
                values: MetricValues::Unsigned(NumericValues {
                    min: None,
                    max: None,
                    sum: None,
                    count: Some(1),
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

    /// Scenario: A metric sampling type contains only one of the mandatory sum and count flags.
    /// Guarantees: Encoding rejects both incomplete combinations before writing metric bytes.
    #[test]
    fn rejects_sampling_type_missing_sum_or_count() {
        for sampling_type in [SUM, COUNT] {
            let metric = standard_metric(unsigned_values(1, 1), sampling_type);
            let mut writer = Writer::default();

            assert_eq!(
                write_metric(&mut writer, &metric),
                Err(EncodeError::MissingRequiredSamplingFlags {
                    sampling_type,
                    required_flags: SUM | COUNT,
                })
            );
            assert!(writer.bytes().is_empty());
        }
    }

    /// Scenario: A metric sets the HyperLogLog sketch flag without a supported sketch body model.
    /// Guarantees: Encoding rejects the unsupported flag instead of emitting a truncated metric payload.
    #[test]
    fn rejects_unsupported_hyper_log_log_sketch() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(
                unsigned_values(1, 1),
                SUM | COUNT | HYPER_LOG_LOG_SKETCH,
            )],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::UnsupportedSamplingFlag {
                flag: HYPER_LOG_LOG_SKETCH,
            })
        );
    }

    /// Scenario: A high-resolution timestamp metric omits the count sampling flag required by ME.
    /// Guarantees: Encoding rejects the invalid flag combination before count and milliseconds lose alignment.
    #[test]
    fn rejects_high_resolution_timestamp_without_count() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(
                MetricValues::Unsigned(NumericValues {
                    min: None,
                    max: None,
                    sum: Some(1),
                    count: None,
                    milliseconds: Some(500),
                    histogram: None,
                }),
                SUM | HIGH_RESOLUTION_TIMESTAMP,
            )],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::MissingRequiredSamplingFlag {
                flag: HIGH_RESOLUTION_TIMESTAMP,
                required_flag: COUNT,
            })
        );
    }

    /// Scenario: An OTLP-sized sample count exceeds Geneva's unsigned 32-bit count field.
    /// Guarantees: Encoding reports the exact protocol limit instead of truncating the count.
    #[test]
    fn rejects_unrepresentable_count() {
        let count = u64::from(u32::MAX) + 1;
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(unsigned_values(1, count), SUM | COUNT)],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::ValueOverflow {
                field: "count",
                value: count,
                maximum: u64::from(u32::MAX),
            })
        );
    }

    /// Scenario: A high-resolution metric supplies a millisecond component outside one second.
    /// Guarantees: Encoding rejects the invalid component rather than publishing an ambiguous timestamp.
    #[test]
    fn rejects_invalid_millisecond_component() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(
                MetricValues::Unsigned(NumericValues {
                    min: None,
                    max: None,
                    sum: Some(1),
                    count: Some(1),
                    milliseconds: Some(1_000),
                    histogram: None,
                }),
                SUM | COUNT | HIGH_RESOLUTION_TIMESTAMP,
            )],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::ValueOverflow {
                field: "milliseconds",
                value: 1_000,
                maximum: 999,
            })
        );
    }

    /// Scenario: Packet and metric whole-second buckets reach and exceed ME's maximum u64 tick value.
    /// Guarantees: The largest reconstructable bucket is accepted and either field rejects one additional second.
    #[test]
    fn validates_whole_second_tick_boundary() {
        let _ = encode(&Packet {
            current_time_bucket: MAX_TIME_BUCKET,
            metrics: Vec::new(),
        })
        .expect("maximum packet time bucket should encode");

        assert_eq!(
            encode(&Packet {
                current_time_bucket: MAX_TIME_BUCKET + 1,
                metrics: Vec::new(),
            }),
            Err(EncodeError::TimestampOutOfRange {
                field: "packet",
                time_bucket: MAX_TIME_BUCKET + 1,
                milliseconds: 0,
            })
        );

        let packet = Packet {
            current_time_bucket: MAX_TIME_BUCKET,
            metrics: vec![Metric {
                time_bucket: (MAX_TIME_BUCKET + 1) as i64,
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
            Err(EncodeError::TimestampOutOfRange {
                field: "metric",
                time_bucket: MAX_TIME_BUCKET + 1,
                milliseconds: 0,
            })
        );
    }

    /// Scenario: A metric at ME's maximum whole-second bucket uses the last valid and first invalid millisecond.
    /// Guarantees: Tick reconstruction accepts 955 milliseconds and rejects 956 before u64 overflow.
    #[test]
    fn validates_high_resolution_tick_boundary() {
        let maximum_milliseconds = ((u64::MAX % TICKS_PER_SECOND) / TICKS_PER_MILLISECOND) as u32;
        assert_eq!(maximum_milliseconds, 955);

        let metric = |milliseconds| Metric {
            time_bucket: MAX_TIME_BUCKET as i64,
            namespace: "namespace".to_string(),
            name: "metric".to_string(),
            dimensions: Vec::new(),
            sampling_type: SUM | COUNT | HIGH_RESOLUTION_TIMESTAMP,
            values: MetricValues::Unsigned(NumericValues {
                min: None,
                max: None,
                sum: Some(1),
                count: Some(1),
                milliseconds: Some(milliseconds),
                histogram: None,
            }),
            exemplars: Vec::new(),
        };

        let _ = encode(&Packet {
            current_time_bucket: MAX_TIME_BUCKET,
            metrics: vec![metric(maximum_milliseconds)],
        })
        .expect("last reconstructable millisecond should encode");

        assert_eq!(
            encode(&Packet {
                current_time_bucket: MAX_TIME_BUCKET,
                metrics: vec![metric(maximum_milliseconds + 1)],
            }),
            Err(EncodeError::TimestampOutOfRange {
                field: "metric",
                time_bucket: MAX_TIME_BUCKET,
                milliseconds: maximum_milliseconds + 1,
            })
        );
    }

    /// Scenario: A metric time bucket predates the .NET epoch used by Geneva timestamps.
    /// Guarantees: Encoding rejects the negative bucket instead of producing a timestamp that wraps on decode.
    #[test]
    fn rejects_negative_metric_time_bucket() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
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

        assert_eq!(encode(&packet), Err(EncodeError::NegativeTimeBucket(-1)));
    }

    /// Scenario: A metric dimension value contains an embedded NUL character.
    /// Guarantees: Encoding rejects the invalid aggregation key instead of publishing a value ME would skip.
    #[test]
    fn rejects_nul_in_dimension_value() {
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        metric.dimensions[1].value = "invalid\0value".to_string();
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![metric],
        };

        assert_eq!(
            encode(&packet),
            Err(EncodeError::InvalidDimensionValue {
                metric_index: 0,
                dimension_index: 1,
            })
        );
    }

    /// Scenario: Metrics contain 74 and 75 original dimensions that would all be trimmed from the wire.
    /// Guarantees: The Geneva maximum is accepted, while one extra original dimension is rejected before trimming.
    #[test]
    fn validates_original_dimension_count_limit() {
        let dimensions = |count| {
            (0..count)
                .map(|index| dimension("", &format!("ignored-{index}")))
                .collect::<Vec<_>>()
        };
        let mut accepted = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        accepted.dimensions = dimensions(MAX_DIMENSIONS);
        let _ = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![accepted],
        })
        .expect("74 dimensions should encode");

        let mut rejected = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        rejected.dimensions = dimensions(MAX_DIMENSIONS + 1);
        assert_eq!(
            encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![rejected],
            }),
            Err(EncodeError::DimensionCountOverflow {
                metric_index: 0,
                count: MAX_DIMENSIONS + 1,
                maximum: MAX_DIMENSIONS,
            })
        );
    }

    /// Scenario: Metric and dimension strings are exactly at and one character beyond ME's limits.
    /// Guarantees: Every maximum is accepted and oversized strings are rejected before interning.
    #[test]
    fn validates_me_string_length_limits() {
        fn assert_limit(
            field: &'static str,
            maximum: usize,
            set_value: impl Fn(&mut Metric, String),
        ) {
            let mut accepted = standard_metric(unsigned_values(1, 1), SUM | COUNT);
            set_value(&mut accepted, "a".repeat(maximum));
            let _ = encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![accepted],
            })
            .expect("maximum string length should encode");

            let mut rejected = standard_metric(unsigned_values(1, 1), SUM | COUNT);
            set_value(&mut rejected, "a".repeat(maximum + 1));
            assert_eq!(
                encode(&Packet {
                    current_time_bucket: DEFAULT_TIME_BUCKET,
                    metrics: vec![rejected],
                }),
                Err(EncodeError::StringLengthOverflow {
                    field,
                    length: maximum + 1,
                    maximum,
                })
            );
        }

        assert_limit("metric name", MAX_METRIC_NAME_LENGTH, |metric, value| {
            metric.name = value;
        });
        assert_limit(
            "dimension name",
            MAX_DIMENSION_NAME_LENGTH,
            |metric, value| {
                metric.dimensions[0].name = value;
            },
        );
        assert_limit(
            "dimension value",
            MAX_DIMENSION_VALUE_LENGTH,
            |metric, value| {
                metric.dimensions[0].value = value;
            },
        );
    }

    /// Scenario: A metric name contains non-BMP characters that occupy two UTF-16 code units each.
    /// Guarantees: String limits match Windows ME's wide-string length rather than Unicode scalar count.
    #[test]
    fn counts_string_limits_in_utf16_code_units() {
        let supplementary_character = "\u{1f600}";
        let mut accepted = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        accepted.name = supplementary_character.repeat(MAX_METRIC_NAME_LENGTH / 2);
        let _ = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![accepted],
        })
        .expect("512 UTF-16 code units should encode");

        let mut rejected = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        rejected.name = supplementary_character.repeat(MAX_METRIC_NAME_LENGTH / 2 + 1);
        assert_eq!(
            encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![rejected],
            }),
            Err(EncodeError::StringLengthOverflow {
                field: "metric name",
                length: MAX_METRIC_NAME_LENGTH + 2,
                maximum: MAX_METRIC_NAME_LENGTH,
            })
        );
    }

    /// Scenario: A packet is encoded with a protocol header and checksum.
    /// Guarantees: Version, serializer flags, and CRC32 match the documented Geneva packet layout.
    #[test]
    fn writes_version_flags_and_crc() {
        let packet = Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![standard_metric(unsigned_values(1, 1), SUM | COUNT)],
        };
        let bytes = encode(&packet).expect("packet should encode");

        assert_eq!(
            u16::from_le_bytes(bytes[0..2].try_into().expect("version field")),
            PROTOCOL_VERSION
        );
        assert_eq!(
            u32::from_le_bytes(bytes[6..10].try_into().expect("serializer flags")),
            TYPE_SERIALIZER_FLAGS
        );
        assert_eq!(
            u32::from_le_bytes(bytes[2..6].try_into().expect("CRC field")),
            crc32fast::hash(&bytes[CRC_INPUT_OFFSET..])
        );
    }

    /// Scenario: Two metrics have identical namespace, name, and dimension names but different values.
    /// Guarantees: Metadata is interned once while each distinct dimension value remains in the string table.
    #[test]
    fn interns_shared_metadata_and_distinct_values() {
        let metric = |value: &str| Metric {
            time_bucket: DEFAULT_TIME_BUCKET as i64,
            namespace: "namespace".to_string(),
            name: "metric".to_string(),
            dimensions: vec![dimension("region", value)],
            sampling_type: SUM | COUNT,
            values: unsigned_values(1, 1),
            exemplars: Vec::new(),
        };
        let bytes = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![metric("east"), metric("west")],
        })
        .expect("packet should encode");

        let mut metadata_position = read_u64(&bytes, 10) as usize;
        assert_eq!(read_unsigned_base128_at(&bytes, &mut metadata_position), 1);
        assert_eq!(
            read_string_table(&bytes)
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            ["namespace", "metric", "region", "east", "west"]
        );
    }

    /// Scenario: Two metrics emit the same non-empty dimension names but have different original name lists.
    /// Guarantees: Empty-name trimming does not collapse distinct ME metadata identities into one index.
    #[test]
    fn preserves_original_dimension_names_for_metadata_identity() {
        let mut with_empty_name = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        with_empty_name.dimensions = vec![dimension("", "ignored"), dimension("region", "east")];
        let mut without_empty_name = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        without_empty_name.dimensions = vec![dimension("region", "west")];

        let bytes = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![with_empty_name, without_empty_name],
        })
        .expect("packet should encode");

        let mut metadata_position = read_u64(&bytes, 10) as usize;
        assert_eq!(read_unsigned_base128_at(&bytes, &mut metadata_position), 2);
        assert!(
            !read_string_table(&bytes)
                .into_iter()
                .any(|(_, value)| value.is_empty() || value == "ignored")
        );
    }

    /// Scenario: A metric contains a dimension with an empty name.
    /// Guarantees: The invalid dimension name and its value are omitted from metadata and string tables.
    #[test]
    fn omits_empty_dimension_names_and_values() {
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        metric.dimensions = vec![dimension("", "ignored")];
        let bytes = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![metric],
        })
        .expect("packet should encode");

        let mut metadata_position = read_u64(&bytes, 10) as usize;
        assert_eq!(read_unsigned_base128_at(&bytes, &mut metadata_position), 1);
        let _namespace_index = read_unsigned_base128_at(&bytes, &mut metadata_position);
        let _name_index = read_unsigned_base128_at(&bytes, &mut metadata_position);
        assert_eq!(read_unsigned_base128_at(&bytes, &mut metadata_position), 0);
        assert_eq!(
            read_string_table(&bytes)
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            ["MetricNamespace", "MetricName"]
        );
    }

    /// Scenario: A metric name contains a non-ASCII character represented by an ASCII Rust escape.
    /// Guarantees: String-table lengths count UTF-8 bytes rather than Unicode scalar values.
    #[test]
    fn writes_utf8_string_lengths_in_bytes() {
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT);
        metric.name = "m\u{00e9}tric".to_string();
        let bytes = encode(&Packet {
            current_time_bucket: DEFAULT_TIME_BUCKET,
            metrics: vec![metric],
        })
        .expect("packet should encode");

        let (_, name) = &read_string_table(&bytes)[1];
        assert_eq!(name, "m\u{00e9}tric");
        assert_eq!(name.len(), 7);
        assert_eq!(name.chars().count(), 6);
    }
}
