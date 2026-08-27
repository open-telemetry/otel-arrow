// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exemplar encoding for the Geneva Metrics ingestion protocol.

use super::model::*;
use super::numeric::{serializable_as_i64, write_double_or_long};
use super::writer::Writer;

const MAX_EXEMPLAR_PAYLOAD_SIZE: usize = 512;
const MAX_SINGLE_EXEMPLAR_SIZE: usize = 200;
const MIN_SINGLE_EXEMPLAR_SIZE: usize = 5;

pub(super) fn write_exemplars(
    writer: &mut Writer,
    exemplars: &[MetricExemplar],
) -> Result<(), EncodeError> {
    let length = encoded_exemplar_list_size(exemplars)?;
    let list_length_position = writer.reserve(size_of::<u16>());
    let list_body_position = writer.len();
    writer.write_u8(0);
    writer.write_unsigned_base128(exemplars.len() as u64);
    for exemplar in exemplars {
        write_exemplar(writer, exemplar)?;
    }
    debug_assert_eq!(writer.len() - list_body_position, length);
    writer.write_u16_at(list_length_position, length as u16);
    Ok(())
}

fn write_exemplar(writer: &mut Writer, exemplar: &MetricExemplar) -> Result<(), EncodeError> {
    let length = encoded_exemplar_size(exemplar)?;
    let start_position = writer.len();
    writer.write_u8(0);
    let length_position = writer.reserve(size_of::<u8>());

    let stored_as_long = serializable_as_i64(exemplar.value);
    let time_unix_nano = exemplar.time_unix_nano.filter(|timestamp| *timestamp != 0);
    let span_id = exemplar
        .span_id
        .filter(|identifier| identifier.iter().any(|byte| *byte != 0));
    let trace_id = exemplar
        .trace_id
        .filter(|identifier| identifier.iter().any(|byte| *byte != 0));
    let mut flags = if stored_as_long {
        EXEMPLAR_VALUE_STORED_AS_LONG
    } else {
        0
    };
    if time_unix_nano.is_some() {
        flags |= EXEMPLAR_TIMESTAMP_AVAILABLE;
    }
    if span_id.is_some() {
        flags |= EXEMPLAR_SPAN_ID_EXISTS;
    }
    if trace_id.is_some() {
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

    if let Some(time_unix_nano) = time_unix_nano {
        writer.write_u64(time_unix_nano);
    }
    if let Some(trace_id) = trace_id {
        writer.write_bytes(&trace_id);
    }
    if let Some(span_id) = span_id {
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

    debug_assert_eq!(writer.len() - start_position, length);
    writer.write_u8_at(length_position, length as u8);
    Ok(())
}

fn encoded_exemplar_list_size(exemplars: &[MetricExemplar]) -> Result<usize, EncodeError> {
    let exemplar_count =
        u64::try_from(exemplars.len()).map_err(|_| EncodeError::LengthCalculationOverflow {
            field: "exemplar list",
        })?;
    let header_size = checked_size_add(
        "exemplar list",
        size_of::<u8>(),
        unsigned_base128_size(exemplar_count),
    )?;
    let minimum_payload_size = exemplars
        .len()
        .checked_mul(MIN_SINGLE_EXEMPLAR_SIZE)
        .ok_or(EncodeError::LengthCalculationOverflow {
            field: "exemplar list",
        })?;
    if minimum_payload_size > MAX_EXEMPLAR_PAYLOAD_SIZE {
        return Err(EncodeError::LengthOverflow {
            field: "exemplar list",
            length: minimum_payload_size,
            maximum: MAX_EXEMPLAR_PAYLOAD_SIZE,
        });
    }

    let mut payload_size = 0;
    for exemplar in exemplars {
        payload_size = checked_size_add(
            "exemplar list",
            payload_size,
            encoded_exemplar_size(exemplar)?,
        )?;
        if payload_size > MAX_EXEMPLAR_PAYLOAD_SIZE {
            return Err(EncodeError::LengthOverflow {
                field: "exemplar list",
                length: payload_size,
                maximum: MAX_EXEMPLAR_PAYLOAD_SIZE,
            });
        }
    }
    checked_size_add("exemplar list", header_size, payload_size)
}

fn encoded_exemplar_size(exemplar: &MetricExemplar) -> Result<usize, EncodeError> {
    let label_count = exemplar.filtered_attributes.len();
    if label_count > u8::MAX as usize {
        return Err(EncodeError::LengthOverflow {
            field: "exemplar label count",
            length: label_count,
            maximum: u8::MAX as usize,
        });
    }

    let stored_as_long = serializable_as_i64(exemplar.value);
    let value_size = if stored_as_long {
        signed_base128_size(exemplar.value as i64)
    } else {
        size_of::<f64>()
    };
    let mut length = checked_size_add("single exemplar", 3, value_size)?;
    length = checked_size_add("single exemplar", length, size_of::<u8>())?;
    if exemplar
        .time_unix_nano
        .is_some_and(|timestamp| timestamp != 0)
    {
        length = checked_size_add("single exemplar", length, size_of::<u64>())?;
    }
    if exemplar
        .trace_id
        .is_some_and(|identifier| identifier.iter().any(|byte| *byte != 0))
    {
        length = checked_size_add("single exemplar", length, 16)?;
    }
    if exemplar
        .span_id
        .is_some_and(|identifier| identifier.iter().any(|byte| *byte != 0))
    {
        length = checked_size_add("single exemplar", length, 8)?;
    }
    if exemplar.sample_count.is_some_and(|count| count != 0.0) {
        length = checked_size_add("single exemplar", length, size_of::<f64>())?;
    }
    for (name, value) in &exemplar.filtered_attributes {
        length = checked_size_add(
            "single exemplar",
            length,
            unsigned_base128_size(name.len() as u64),
        )?;
        length = checked_size_add("single exemplar", length, name.len())?;
        length = checked_size_add(
            "single exemplar",
            length,
            unsigned_base128_size(value.len() as u64),
        )?;
        length = checked_size_add("single exemplar", length, value.len())?;
    }
    if length > MAX_SINGLE_EXEMPLAR_SIZE {
        return Err(EncodeError::LengthOverflow {
            field: "single exemplar",
            length,
            maximum: MAX_SINGLE_EXEMPLAR_SIZE,
        });
    }
    Ok(length)
}

fn checked_size_add(field: &'static str, left: usize, right: usize) -> Result<usize, EncodeError> {
    left.checked_add(right)
        .ok_or(EncodeError::LengthCalculationOverflow { field })
}

fn unsigned_base128_size(mut value: u64) -> usize {
    let mut size = 1;
    while value >= 0x80 {
        value >>= 7;
        size += 1;
    }
    size
}

fn signed_base128_size(value: i64) -> usize {
    let mut remaining = value.unsigned_abs() >> 6;
    let mut size = 1;
    while remaining != 0 {
        remaining >>= 7;
        size += 1;
    }
    size
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::super::writer::Writer;
    use super::super::*;
    use super::{
        MAX_EXEMPLAR_PAYLOAD_SIZE, MAX_SINGLE_EXEMPLAR_SIZE, write_exemplar, write_exemplars,
    };

    const DEFAULT_UNIX_SECONDS: u64 = 1_388_577_600;

    fn exemplar_with_attribute_lengths(name_length: usize, value_length: usize) -> MetricExemplar {
        MetricExemplar {
            value: 1.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: vec![("n".repeat(name_length), "v".repeat(value_length))],
        }
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

    /// Scenario: An exemplar supplies zero timestamp, trace ID, and span ID values.
    /// Guarantees: Zero-valued optional fields encode identically to absent fields, matching ME presence semantics.
    #[test]
    fn omits_zero_valued_exemplar_fields() {
        let exemplar = MetricExemplar {
            value: 1.0,
            time_unix_nano: Some(0),
            trace_id: Some([0; 16]),
            span_id: Some([0; 8]),
            sample_count: None,
            filtered_attributes: Vec::new(),
        };
        let absent = MetricExemplar {
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            ..exemplar.clone()
        };
        let mut writer = Writer::default();
        let mut expected = Writer::default();

        write_exemplar(&mut writer, &exemplar).expect("zero-valued fields should encode");
        write_exemplar(&mut expected, &absent).expect("absent fields should encode");

        assert_eq!(writer.finish(), expected.finish());
    }

    /// Scenario: An exemplar contains more filtered attributes than its one-byte count can represent.
    /// Guarantees: Encoding reports the exact label-count limit instead of truncating attributes.
    #[test]
    fn rejects_exemplar_label_count_overflow() {
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT | EXEMPLAR);
        metric.exemplars.push(MetricExemplar {
            value: 1.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: (0..=u8::MAX)
                .map(|index| (format!("key-{index}"), "value".to_string()))
                .collect(),
        });

        assert_eq!(
            encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![metric],
            }),
            Err(EncodeError::LengthOverflow {
                field: "exemplar label count",
                length: usize::from(u8::MAX) + 1,
                maximum: usize::from(u8::MAX),
            })
        );
    }

    /// Scenario: An exemplar's serialized body is exactly the Geneva backend's 200-byte limit.
    /// Guarantees: Encoding accepts the largest backend-supported individual exemplar.
    #[test]
    fn accepts_single_exemplar_at_backend_limit() {
        let mut writer = Writer::default();

        write_exemplar(&mut writer, &exemplar_with_attribute_lengths(192, 0))
            .expect("200-byte exemplar should encode");

        assert_eq!(writer.finish().len(), MAX_SINGLE_EXEMPLAR_SIZE);
    }

    /// Scenario: A standalone or listed exemplar exceeds the Geneva backend limit by one byte.
    /// Guarantees: Both paths reject it before writing an item or outer list prefix.
    #[test]
    fn rejects_single_exemplar_over_backend_limit() {
        let exemplar = exemplar_with_attribute_lengths(193, 0);
        let mut writer = Writer::default();

        assert_eq!(
            write_exemplar(&mut writer, &exemplar),
            Err(EncodeError::LengthOverflow {
                field: "single exemplar",
                length: MAX_SINGLE_EXEMPLAR_SIZE + 1,
                maximum: MAX_SINGLE_EXEMPLAR_SIZE,
            })
        );
        assert!(writer.bytes().is_empty());

        let mut list_writer = Writer::default();
        assert_eq!(
            write_exemplars(&mut list_writer, std::slice::from_ref(&exemplar)),
            Err(EncodeError::LengthOverflow {
                field: "single exemplar",
                length: MAX_SINGLE_EXEMPLAR_SIZE + 1,
                maximum: MAX_SINGLE_EXEMPLAR_SIZE,
            })
        );
        assert!(list_writer.bytes().is_empty());
    }

    /// Scenario: Serialized exemplars total exactly 512 bytes before list framing is added.
    /// Guarantees: Encoding accepts ME's largest payload even though version and count make the wire body larger.
    #[test]
    fn accepts_exemplar_list_at_backend_limit() {
        let exemplar = MetricExemplar {
            value: 0.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: Vec::new(),
        };
        let mut exemplars = vec![exemplar; 100];
        exemplars.push(exemplar_with_attribute_lengths(4, 1));
        let mut writer = Writer::default();

        write_exemplars(&mut writer, &exemplars).expect("512-byte exemplar payload should encode");

        assert_eq!(
            writer.finish().len(),
            size_of::<u16>() + size_of::<u8>() + size_of::<u8>() + MAX_EXEMPLAR_PAYLOAD_SIZE
        );
    }

    /// Scenario: Serialized exemplars total 513 bytes before list framing is added.
    /// Guarantees: Encoding rejects the payload before writing the otherwise representable wire body.
    #[test]
    fn rejects_exemplar_list_over_backend_limit() {
        let exemplar = MetricExemplar {
            value: 0.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: Vec::new(),
        };
        let mut exemplars = vec![exemplar; 100];
        exemplars.push(exemplar_with_attribute_lengths(5, 1));
        let mut writer = Writer::default();

        assert_eq!(
            write_exemplars(&mut writer, &exemplars),
            Err(EncodeError::LengthOverflow {
                field: "exemplar list",
                length: MAX_EXEMPLAR_PAYLOAD_SIZE + 1,
                maximum: MAX_EXEMPLAR_PAYLOAD_SIZE,
            })
        );
        assert!(writer.bytes().is_empty());
    }

    /// Scenario: Exemplar list counts cross the signed base-128 first-byte boundary.
    /// Guarantees: Counts 63, 64, and 101 use the unsigned encoding consumed by the Geneva reader.
    #[test]
    fn encodes_exemplar_count_as_unsigned_base128() {
        let exemplar = MetricExemplar {
            value: 0.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: Vec::new(),
        };

        for count in [63, 64, 101] {
            let mut writer = Writer::default();
            write_exemplars(&mut writer, &vec![exemplar.clone(); count])
                .expect("exemplar list should encode");
            let bytes = writer.finish();

            assert_eq!(read_unsigned_base128(&bytes[3..]), count as u64);
        }
    }
}
