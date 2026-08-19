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
        && value > i64::MIN as f64
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
        for (value, stored_as_long) in [
            (-42.0, true),
            (42.0, true),
            (42.5, false),
            (f64::NAN, false),
            (f64::INFINITY, false),
            (f64::NEG_INFINITY, false),
        ] {
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

    /// Scenario: Integral doubles sit at and immediately inside the signed 64-bit conversion boundaries.
    /// Guarantees: The compact range matches ME by excluding both 2^63 endpoints while accepting adjacent representable values.
    #[test]
    fn matches_me_compact_integer_boundaries() {
        const I64_MIN_AS_F64: f64 = -9_223_372_036_854_775_808.0;
        const NEXT_AFTER_I64_MIN: f64 = -9_223_372_036_854_774_784.0;
        const PREVIOUS_BEFORE_I64_MAX: f64 = 9_223_372_036_854_774_784.0;
        const TWO_TO_THE_63: f64 = 9_223_372_036_854_775_808.0;

        assert!(!serializable_as_i64(I64_MIN_AS_F64));
        assert!(serializable_as_i64(NEXT_AFTER_I64_MIN));
        assert!(serializable_as_i64(PREVIOUS_BEFORE_I64_MAX));
        assert!(!serializable_as_i64(TWO_TO_THE_63));
    }

    /// Scenario: A double metric selects min, max, and sum values and may also carry a histogram.
    /// Guarantees: Compact storage requires every selected scalar to be integral and is disabled for histogram metrics.
    #[test]
    fn requires_integral_selected_scalars_without_histogram() {
        let mut values = NumericValues {
            min: Some(1.0),
            max: Some(2.0),
            sum: Some(3.0),
            count: Some(3),
            milliseconds: None,
            histogram: None,
        };
        let sampling_type = MIN | MAX | SUM | COUNT;

        assert!(can_store_double_values_as_long(sampling_type, &values));

        values.max = Some(2.5);
        assert!(!can_store_double_values_as_long(sampling_type, &values));

        values.max = Some(2.0);
        values.histogram = Some(MetricHistogram::Explicit(vec![(2.0, 3)]));
        assert!(!can_store_double_values_as_long(
            sampling_type | HISTOGRAM,
            &values
        ));
    }
}
