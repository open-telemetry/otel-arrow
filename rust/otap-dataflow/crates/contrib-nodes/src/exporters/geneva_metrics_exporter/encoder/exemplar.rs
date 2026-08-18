// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exemplar encoding for the Geneva Metrics ingestion protocol.

use super::model::*;
use super::numeric::{serializable_as_i64, write_double_or_long};
use super::writer::Writer;

pub(super) fn write_exemplars(
    writer: &mut Writer,
    exemplars: &[MetricExemplar],
) -> Result<(), EncodeError> {
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

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::super::*;

    const DEFAULT_UNIX_SECONDS: u64 = 1_388_577_600;

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

    /// Scenario: One exemplar's attributes exceed its one-byte body length.
    /// Guarantees: Encoding rejects the oversized exemplar rather than wrapping its length.
    #[test]
    fn rejects_single_exemplar_length_overflow() {
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT | EXEMPLAR);
        metric.exemplars.push(MetricExemplar {
            value: 1.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: vec![("k".repeat(200), "v".repeat(100))],
        });

        assert!(matches!(
            encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![metric],
            }),
            Err(EncodeError::LengthOverflow {
                field: "single exemplar",
                maximum,
                ..
            }) if maximum == usize::from(u8::MAX)
        ));
    }

    /// Scenario: A metric contains enough valid exemplars to exceed the two-byte list length.
    /// Guarantees: Encoding reports exemplar-list overflow after validating each individual exemplar.
    #[test]
    fn rejects_exemplar_list_length_overflow() {
        let exemplar = MetricExemplar {
            value: 0.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: Vec::new(),
        };
        let mut metric = standard_metric(unsigned_values(1, 1), SUM | COUNT | EXEMPLAR);
        metric.exemplars = vec![exemplar; 14_000];

        assert!(matches!(
            encode(&Packet {
                current_time_bucket: DEFAULT_TIME_BUCKET,
                metrics: vec![metric],
            }),
            Err(EncodeError::LengthOverflow {
                field: "exemplar list",
                maximum,
                ..
            }) if maximum == usize::from(u16::MAX)
        ));
    }
}
