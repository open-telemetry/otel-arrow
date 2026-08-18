// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared constructors and packet readers for encoder unit tests.

use super::*;

pub(super) const DEFAULT_TIME_BUCKET: u64 = 63_524_174_400;

pub(super) fn dimension(name: &str, value: &str) -> Dimension {
    Dimension {
        name: name.to_string(),
        value: value.to_string(),
    }
}

pub(super) fn unsigned_values(sum: u64, count: u32) -> MetricValues {
    MetricValues::Unsigned(NumericValues {
        min: None,
        max: None,
        sum: Some(sum),
        count: Some(count),
        milliseconds: None,
        histogram: None,
    })
}

pub(super) fn double_values(sum: f64, count: u32) -> MetricValues {
    MetricValues::Double(NumericValues {
        min: None,
        max: None,
        sum: Some(sum),
        count: Some(count),
        milliseconds: None,
        histogram: None,
    })
}

pub(super) fn standard_metric(values: MetricValues, sampling_type: u32) -> Metric {
    Metric {
        time_bucket: DEFAULT_TIME_BUCKET as i64,
        namespace: "MetricNamespace".to_string(),
        name: "MetricName".to_string(),
        dimensions: vec![
            dimension("Dim1Name", "Dim1Value"),
            dimension("Dim2Name", "Dim2Value"),
        ],
        sampling_type,
        values,
        exemplars: Vec::new(),
    }
}

pub(super) fn assert_fixture(packet: Packet, expected: &[u8]) {
    assert_eq!(encode(&packet).expect("packet should encode"), expected);
}

pub(super) fn read_unsigned_base128(bytes: &[u8]) -> u64 {
    let mut value = 0;
    let mut shift = 0;
    for byte in bytes {
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
        shift += 7;
    }
    panic!("unterminated base-128 value");
}
