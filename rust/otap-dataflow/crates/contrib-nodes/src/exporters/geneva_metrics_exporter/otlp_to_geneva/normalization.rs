// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Coalesces split metric data before using the shared borrowed OTLP views.

use std::borrow::Cow;

use otel_arrow_dfe_pdata::proto::consts::field_num::metrics::{
    EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY, HISTOGRAM_AGGREGATION_TEMPORALITY,
    METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE, METRIC_HISTOGRAM, METRIC_SUM, METRIC_SUMMARY,
    METRICS_DATA_RESOURCE_METRICS, RESOURCE_METRICS_SCOPE_METRICS, SCOPE_METRICS_METRICS,
    SUM_AGGREGATION_TEMPORALITY, SUM_IS_MONOTONIC,
};
use otel_arrow_dfe_pdata::proto::consts::wire_types;
use otel_arrow_dfe_pdata::views::otlp::bytes::decode::{
    read_fixed32, read_fixed64, read_len_delim, read_varint,
};

use super::{MappingError, MappingMemory};

const CONTAINERS: [u64; 3] = [
    METRICS_DATA_RESOURCE_METRICS,
    RESOURCE_METRICS_SCOPE_METRICS,
    SCOPE_METRICS_METRICS,
];

pub(super) fn normalize<'a>(
    bytes: &'a [u8],
    memory: &MappingMemory,
) -> Result<Cow<'a, [u8]>, MappingError> {
    if !needs_normalization(bytes, 0) {
        return Ok(Cow::Borrowed(bytes));
    }

    // Removing replaced data, duplicate headers, and superseded scalars cannot
    // enlarge the input. Charge the sole output buffer before allocating it.
    memory.charge_items::<u8>(bytes.len())?;
    let mut output = Vec::with_capacity(bytes.len());
    rewrite(bytes, 0, &mut output);
    debug_assert!(output.len() <= bytes.len());
    Ok(Cow::Owned(output))
}

#[derive(Clone, Copy)]
struct Field<'a> {
    number: u64,
    wire_type: u64,
    encoded: &'a [u8],
    value: &'a [u8],
    tag_len: usize,
}

impl Field<'_> {
    fn is_data(&self) -> bool {
        self.wire_type == wire_types::LEN
            && matches!(
                self.number,
                METRIC_GAUGE
                    | METRIC_SUM
                    | METRIC_HISTOGRAM
                    | METRIC_EXPONENTIAL_HISTOGRAM
                    | METRIC_SUMMARY
            )
    }
}

struct Fields<'a> {
    remaining: &'a [u8],
}

impl<'a> Fields<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    // Leave malformed tails intact for the shared view's best-effort handling.
    fn next(&mut self) -> Option<Field<'a>> {
        let bytes = self.remaining;
        let (tag, tag_len) = read_varint(bytes, 0)?;
        let wire_type = tag & 7;
        let (value, end) = match wire_type {
            wire_types::LEN => read_len_delim(bytes, tag_len)?,
            wire_types::FIXED64 => read_fixed64(bytes, tag_len)?,
            wire_types::FIXED32 => read_fixed32(bytes, tag_len)?,
            wire_types::VARINT => {
                let (_, end) = read_varint(bytes, tag_len)?;
                (&bytes[tag_len..end], end)
            }
            _ => return None,
        };
        self.remaining = &bytes[end..];
        Some(Field {
            number: tag >> 3,
            wire_type,
            encoded: &bytes[..end],
            value,
            tag_len,
        })
    }
}

fn needs_normalization(bytes: &[u8], depth: usize) -> bool {
    let mut fields = Fields::new(bytes);
    let mut has_data = false;
    while let Some(field) = fields.next() {
        if depth == CONTAINERS.len() {
            if field.is_data() {
                if has_data {
                    return true;
                }
                has_data = true;
            }
        } else if field.number == CONTAINERS[depth]
            && field.wire_type == wire_types::LEN
            && needs_normalization(field.value, depth + 1)
        {
            return true;
        }
    }
    false
}

fn rewrite(bytes: &[u8], depth: usize, output: &mut Vec<u8>) {
    if depth == CONTAINERS.len() {
        rewrite_metric(bytes, output);
        return;
    }

    let mut fields = Fields::new(bytes);
    while let Some(field) = fields.next() {
        if field.number == CONTAINERS[depth]
            && field.wire_type == wire_types::LEN
            && needs_normalization(field.value, depth + 1)
        {
            let length_start = output.len() + field.tag_len;
            let header_len = field.encoded.len() - field.value.len();
            output.extend_from_slice(&field.encoded[..header_len]);
            let body_start = output.len();
            rewrite(field.value, depth + 1, output);
            let (length, width) = varint((output.len() - body_start) as u64);
            output[length_start..length_start + width].copy_from_slice(&length[..width]);
            let removed = body_start - length_start - width;
            if removed != 0 {
                output.copy_within(body_start.., length_start + width);
                output.truncate(output.len() - removed);
            }
        } else {
            output.extend_from_slice(field.encoded);
        }
    }
    output.extend_from_slice(fields.remaining);
}

fn rewrite_metric(bytes: &[u8], output: &mut Vec<u8>) {
    let mut fields = Fields::new(bytes);
    let mut selected: Option<(u64, &[u8])> = None;
    loop {
        let remaining = fields.remaining;
        let Some(field) = fields.next() else {
            break;
        };
        if field.is_data() {
            if selected.is_none_or(|(number, _)| number != field.number) {
                selected = Some((field.number, remaining));
            }
        } else {
            output.extend_from_slice(field.encoded);
        }
    }

    if let Some((number, fragments)) = selected {
        let temporality = match number {
            METRIC_SUM => Some(SUM_AGGREGATION_TEMPORALITY),
            METRIC_HISTOGRAM => Some(HISTOGRAM_AGGREGATION_TEMPORALITY),
            METRIC_EXPONENTIAL_HISTOGRAM => Some(EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY),
            _ => None,
        };
        let monotonic = (number == METRIC_SUM).then_some(SUM_IS_MONOTONIC);
        let mut scalars = [None, None];
        let mut body_len = 0;
        visit_data(fragments, number, |field| {
            let mut values = Fields::new(field.value);
            while let Some(value) = values.next() {
                if let Some(index) = scalar_index(value, [temporality, monotonic]) {
                    scalars[index] = Some(value.encoded);
                } else {
                    body_len += value.encoded.len();
                }
            }
            body_len += values.remaining.len();
        });
        body_len += scalars
            .iter()
            .flatten()
            .map(|bytes| bytes.len())
            .sum::<usize>();

        append_varint(number << 3 | wire_types::LEN, output);
        append_varint(body_len as u64, output);
        visit_data(fragments, number, |field| {
            let mut values = Fields::new(field.value);
            while let Some(value) = values.next() {
                if scalar_index(value, [temporality, monotonic]).is_none() {
                    output.extend_from_slice(value.encoded);
                }
            }
            output.extend_from_slice(values.remaining);
        });
        for scalar in scalars.into_iter().flatten() {
            output.extend_from_slice(scalar);
        }
    }
    output.extend_from_slice(fields.remaining);
}

fn visit_data<'a>(bytes: &'a [u8], number: u64, mut visit: impl FnMut(Field<'a>)) {
    let mut fields = Fields::new(bytes);
    while let Some(field) = fields.next() {
        if field.number == number && field.wire_type == wire_types::LEN {
            visit(field);
        }
    }
}

fn scalar_index(field: Field<'_>, numbers: [Option<u64>; 2]) -> Option<usize> {
    if field.wire_type != wire_types::VARINT {
        return None;
    }
    numbers
        .iter()
        .position(|number| *number == Some(field.number))
}

fn varint(mut value: u64) -> ([u8; 10], usize) {
    let mut bytes = [0; 10];
    let mut width = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        bytes[width] = byte | if value == 0 { 0 } else { 0x80 };
        width += 1;
        if value == 0 {
            return (bytes, width);
        }
    }
}

fn append_varint(value: u64, output: &mut Vec<u8>) {
    let (bytes, width) = varint(value);
    output.extend_from_slice(&bytes[..width]);
}

#[cfg(test)]
mod tests {
    use super::super::{map_metrics, map_otlp_bytes};
    use super::*;
    use crate::exporters::geneva_metrics_exporter::publication_preparation::test_support::{
        fragmented_request, gauge_point, mapping_config, otlp_pdata, single_point_request,
        wrap_metric,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, MetricsData, NumberDataPoint, Sum, Summary, SummaryDataPoint, metric::Data,
    };
    use prost::Message;

    const RECEIVE_TIME: u64 = 1_700_000_000_000_000_000;

    fn assert_mapping_matches_protobuf(bytes: &[u8]) {
        let decoded = MetricsData::decode(bytes).expect("fixture should be valid protobuf");
        let config = mapping_config();
        let expected =
            map_metrics(&decoded, &config, RECEIVE_TIME).expect("canonical input should map");
        let actual = map_otlp_bytes(bytes, &config, RECEIVE_TIME).expect("split input should map");
        assert_eq!(actual, expected);
    }

    /// Scenario: A canonical OTLP request needs no metric data coalescing.
    /// Guarantees: Normalization borrows the original bytes without consuming any allocation budget.
    #[test]
    fn borrows_ordinary_input_without_charging_memory() {
        let data = otlp_pdata(single_point_request());
        let payload = data.payload_ref();
        let encoded = payload.encoded_view().expect("fixture should be encoded");
        let normalized = normalize(encoded.bytes(), &MappingMemory::new(0))
            .expect("borrowing should not need allocation budget");
        assert!(matches!(normalized, Cow::Borrowed(_)));
        assert_eq!(normalized.as_ptr(), encoded.bytes().as_ptr());
    }

    /// Scenario: Every metric kind contains two data messages with distinct point timestamps and scalar updates.
    /// Guarantees: Geneva output and rejected-point counts match canonical protobuf decoding without runtime object decoding.
    #[test]
    fn merges_all_metric_kinds_like_protobuf() {
        let fragments = |offset: u64| {
            let time = RECEIVE_TIME + offset * 1_000_000_000;
            let point = NumberDataPoint {
                time_unix_nano: time,
                ..gauge_point(Vec::new(), 0)
            };
            [
                Data::Gauge(Gauge {
                    data_points: vec![point.clone()],
                }),
                Data::Sum(Sum {
                    data_points: vec![point],
                    aggregation_temporality: offset as i32,
                    is_monotonic: true,
                }),
                Data::Histogram(Histogram {
                    data_points: vec![HistogramDataPoint {
                        time_unix_nano: time,
                        count: 1,
                        bucket_counts: vec![1],
                        ..Default::default()
                    }],
                    aggregation_temporality: offset as i32,
                }),
                Data::ExponentialHistogram(ExponentialHistogram {
                    data_points: vec![ExponentialHistogramDataPoint {
                        time_unix_nano: time,
                        count: 1,
                        zero_count: 1,
                        ..Default::default()
                    }],
                    aggregation_temporality: offset as i32,
                }),
                Data::Summary(Summary {
                    data_points: vec![SummaryDataPoint {
                        time_unix_nano: time,
                        count: 1,
                        sum: 1.0,
                        ..Default::default()
                    }],
                }),
            ]
        };
        for (first, second) in fragments(1).into_iter().zip(fragments(2)) {
            assert_mapping_matches_protobuf(&fragmented_request([first, second]));
        }
    }

    /// Scenario: A metric changes data kinds and later returns to a previous kind, including an empty replacement.
    /// Guarantees: Discarded kinds cannot leak points or scalar state into the final Geneva result.
    #[test]
    fn replaces_oneof_data_without_resurrecting_old_state() {
        let gauge = Data::Gauge(Gauge {
            data_points: vec![gauge_point(Vec::new(), 0)],
        });
        let sum = Data::Sum(Sum {
            data_points: vec![gauge_point(Vec::new(), 0)],
            aggregation_temporality: 2,
            is_monotonic: true,
        });
        for fragments in [
            vec![gauge.clone(), sum.clone(), gauge.clone(), gauge.clone()],
            vec![sum, gauge.clone(), Data::Sum(Sum::default())],
            vec![gauge, Data::Summary(Summary::default())],
        ] {
            assert_mapping_matches_protobuf(&fragmented_request(fragments));
        }
    }

    /// Scenario: Later sum messages omit scalar fields or explicitly reset temporality and monotonicity to zero.
    /// Guarantees: Absent scalars retain earlier values, while explicit defaults override them before Geneva mapping.
    #[test]
    fn preserves_scalar_presence_and_explicit_defaults() {
        let mut metric = Metric {
            name: "temperature".into(),
            data: Some(Data::Sum(Sum {
                data_points: vec![gauge_point(Vec::new(), 0)],
                aggregation_temporality: 2,
                is_monotonic: true,
            })),
            ..Default::default()
        }
        .encode_to_vec();
        Metric {
            data: Some(Data::Sum(Sum::default())),
            ..Default::default()
        }
        .encode(&mut metric)
        .expect("empty sum should encode");
        assert_mapping_matches_protobuf(&wrap_metric(metric.clone()));
        // Metric.sum with explicitly encoded zero temporality and false monotonicity.
        metric.extend_from_slice(&[0x3a, 0x04, 0x10, 0x00, 0x18, 0x00]);
        assert_mapping_matches_protobuf(&wrap_metric(metric));
    }

    /// Scenario: Many empty gauge fragments separate two points and make all three enclosing lengths shrink below 128 bytes.
    /// Guarantees: Rewritten length prefixes remain valid and neither point is lost at an integer-width boundary.
    #[test]
    fn rewrites_lengths_across_encoding_width_boundaries() {
        let gauge = Data::Gauge(Gauge {
            data_points: vec![gauge_point(Vec::new(), 0)],
        });
        let fragments = std::iter::once(gauge.clone())
            .chain(std::iter::repeat_n(Data::Gauge(Gauge::default()), 128))
            .chain(std::iter::once(gauge));
        assert_mapping_matches_protobuf(&fragmented_request(fragments));
    }

    /// Scenario: Coalescing requires a buffer that exceeds the budget, or fits but leaves too little budget for mapping.
    /// Guarantees: Normalization and point mapping consume one shared limit rather than each obtaining a fresh allowance.
    #[test]
    fn shares_normalization_and_mapping_budget() {
        let mut metric = Metric {
            name: "temperature".into(),
            description: "x".repeat(4096),
            data: Some(Data::Gauge(Gauge {
                data_points: vec![gauge_point(Vec::new(), 0)],
            })),
            ..Default::default()
        }
        .encode_to_vec();
        Metric {
            data: Some(Data::Gauge(Gauge {
                data_points: vec![gauge_point(Vec::new(), 0)],
            })),
            ..Default::default()
        }
        .encode(&mut metric)
        .expect("second gauge should encode");
        let bytes = wrap_metric(metric);
        let charge = bytes.len() * 4;
        assert!(matches!(
            normalize(&bytes, &MappingMemory::new(charge - 1)),
            Err(MappingError::MemoryLimit { .. })
        ));
        let mut config = mapping_config();
        config.max_mapping_bytes = charge + 1;
        let decoded = MetricsData::decode(bytes.as_slice()).expect("fixture should decode");
        assert!(map_metrics(&decoded, &config, RECEIVE_TIME).is_ok());
        assert!(matches!(
            map_otlp_bytes(&bytes, &config, RECEIVE_TIME),
            Err(MappingError::MemoryLimit { .. })
        ));
    }
}
