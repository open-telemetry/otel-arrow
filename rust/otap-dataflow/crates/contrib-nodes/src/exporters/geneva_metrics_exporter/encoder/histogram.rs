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
    use super::super::*;

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
