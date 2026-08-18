// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Numeric encoding helpers for the Geneva Metrics ingestion protocol.

use super::model::*;
use super::writer::Writer;

pub(super) fn required<T: Copy>(
    value: Option<T>,
    flag: u32,
    field: &'static str,
) -> Result<T, EncodeError> {
    value.ok_or(EncodeError::MissingValue { flag, field })
}

pub(super) fn can_store_double_values_as_long(
    sampling_type: u32,
    values: &NumericValues<f64>,
) -> bool {
    sampling_type & HISTOGRAM == 0
        && (sampling_type & SUM == 0 || values.sum.is_some_and(serializable_as_i64))
        && (sampling_type & MIN == 0 || values.min.is_some_and(serializable_as_i64))
        && (sampling_type & MAX == 0 || values.max.is_some_and(serializable_as_i64))
}

pub(super) fn serializable_as_i64(value: f64) -> bool {
    value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value < -(i64::MIN as f64)
}

pub(super) fn write_double_or_long(writer: &mut Writer, value: f64, stored_as_long: bool) {
    if stored_as_long {
        writer.write_signed_base128(value as i64);
    } else {
        writer.write_f64(value);
    }
}

#[cfg(test)]
mod tests {
    use super::super::packet::write_metric;
    use super::super::test_support::*;
    use super::super::writer::Writer;
    use super::*;

    /// Scenario: Double metrics contain integral, fractional, and non-finite values.
    /// Guarantees: Only finite integral doubles use the compact signed base-128 representation.
    #[test]
    fn selects_compact_storage_only_for_integral_doubles() {
        for (value, stored_as_long) in [(42.0, true), (42.5, false), (f64::INFINITY, false)] {
            let metric = standard_metric(double_values(value, 1), SUM | COUNT);
            let mut writer = Writer::default();
            write_metric(&mut writer, &metric).expect("metric should encode");
            let sampling_type = read_unsigned_base128(writer.bytes()) as u32;

            assert_ne!(sampling_type & DOUBLE_VALUE_TYPE, 0);
            assert_eq!(
                sampling_type & DOUBLE_VALUE_STORED_AS_LONG_TYPE != 0,
                stored_as_long,
                "value {value}"
            );
        }
    }
}
