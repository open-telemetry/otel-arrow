// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Histogram encoding for the Geneva Metrics ingestion protocol.

use super::model::*;
use super::writer::Writer;

pub(super) fn write_histogram(
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

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::super::writer::Writer;
    use super::*;

    /// Scenario: A legacy raw histogram contains multiple ordered buckets with increasing and decreasing counts.
    /// Guarantees: The first bucket is absolute and subsequent keys and counts use the Geneva delta representation.
    #[test]
    fn encodes_raw_histogram_bucket_deltas() {
        let mut writer = Writer::default();

        write_histogram(
            &mut writer,
            &MetricHistogram::Raw(vec![(10, 5), (15, 2), (142, 130)]),
            0,
        )
        .expect("raw histogram should encode");

        assert_eq!(
            writer.finish(),
            vec![
                0x08, 0x00, 0x00, 0x00, // Format and body length.
                0x03, // Bucket count.
                0x0a, 0x05, // First key and count.
                0x05, 0x43, // Key delta 5 and count delta -3.
                0x7f, 0x80, 0x02, // Key delta 127 and count delta 128.
            ]
        );
    }

    /// Scenario: A cumulative explicit histogram contains multiple double boundaries and a decreasing count.
    /// Guarantees: The prefix carries double and cumulative flags while later counts use signed deltas.
    #[test]
    fn encodes_cumulative_explicit_histogram() {
        let mut writer = Writer::default();

        write_histogram(
            &mut writer,
            &MetricHistogram::Explicit(vec![(1.5, 10), (2.5, 7)]),
            METRIC_TYPE_CUMULATIVE_HISTOGRAM,
        )
        .expect("explicit histogram should encode");

        let mut expected = vec![
            0x13, 0x00, 0x00, 0xa0, // Double, cumulative, and body length.
            0x02, // Bucket count.
        ];
        expected.extend_from_slice(&1.5_f64.to_le_bytes());
        expected.push(0x0a);
        expected.extend_from_slice(&2.5_f64.to_le_bytes());
        expected.push(0x43); // Count delta -3.
        assert_eq!(writer.finish(), expected);
    }

    /// Scenario: A cumulative exponential histogram has zero, negative, positive, and empty sparse buckets.
    /// Guarantees: Distribution flags and counts exclude empty buckets, negative exponents reverse, and both ranges delta encode.
    #[test]
    fn encodes_all_exponential_histogram_ranges() {
        let mut writer = Writer::default();

        write_histogram(
            &mut writer,
            &MetricHistogram::Exponential(ExponentialHistogram {
                scale: -2,
                zero_count: 4,
                negative: vec![(-5, 2), (-4, 0), (-2, 5)],
                positive: vec![(1, 3), (2, 0), (4, 8)],
            }),
            METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM,
        )
        .expect("exponential histogram should encode");

        assert_eq!(
            writer.finish(),
            vec![
                0x0d, 0x00, 0x00, 0xc0, // Exponential, cumulative, and body length.
                0xfe, // Scale -2.
                0x19, // Positive, zero, and negative ranges.
                0x04, // Zero count.
                0x02, 0x02, // Non-zero negative and positive bucket counts.
                0x42, 0x05, // First negative exponent -2 and count 5.
                0x43, 0x43, // Negative exponent and count deltas -3.
                0x01, 0x03, // First positive exponent 1 and count 3.
                0x03, 0x05, // Positive exponent delta 3 and count delta 5.
            ]
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
}
