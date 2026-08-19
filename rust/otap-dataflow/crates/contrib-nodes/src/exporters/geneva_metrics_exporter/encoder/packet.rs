// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Metrics ingestion protocol version 6 packet encoding.

use std::collections::HashMap;

use super::exemplar::write_exemplars;
use super::histogram::write_histogram;
use super::model::*;
use super::numeric::{can_store_double_values_as_long, required, write_double_or_long};
use super::writer::Writer;

/// Encodes a packet using Geneva Metrics ingestion protocol version 6.
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
        let (metadata_index, metadata_is_new) = metadata_table.intern(metadata)?;
        if metadata_is_new {
            let metadata = &metadata_table.values()[metadata_index as usize];
            let _ = string_table.intern(metadata.namespace.clone())?;
            let _ = string_table.intern(metadata.name.clone())?;
            for dimension_name in &metadata.dimension_names {
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
        writer.write_unsigned_base128(metadata.dimension_names.len() as u64);
        for dimension_name in &metadata.dimension_names {
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

pub(super) fn write_metric(writer: &mut Writer, metric: &Metric) -> Result<(), EncodeError> {
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
