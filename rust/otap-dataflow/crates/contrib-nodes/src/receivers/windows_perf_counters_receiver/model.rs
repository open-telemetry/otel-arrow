// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Numeric performance-counter values, exact-counter samples, and decimal scaling.

use super::config::{MAX_SCALE_POWER10, MIN_SCALE_POWER10};

/// A numeric performance-counter value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Number {
    /// An exact signed integer.
    Integer(i64),
    /// A finite calculated or scaled value, including subnormals and signed zero.
    Double(f64),
}

/// One configured counter's observation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum SampleValue {
    /// A scaled value ready to emit.
    Value(Number),
    /// No value to emit, such as during warm-up or an idle calculation.
    NoObservation,
}

/// One configured counter's result in a sample.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SamplePoint {
    /// Index into the configured counters.
    pub(super) counter_index: usize,
    /// Ready value or expected omission.
    pub(super) value: SampleValue,
}

/// A sample from one collection.
#[derive(Debug, Clone)]
pub(super) struct Sample {
    /// Start of the cumulative sequence, used only by emitted Sum points.
    /// Collection owns sequence continuity and clock rollback handling.
    pub(super) start_time_unix_nano: i64,
    /// Positive observation time in nanoseconds since the Unix epoch.
    pub(super) timestamp_unix_nano: i64,
    /// At most one point per configured counter.
    pub(super) points: Vec<SamplePoint>,
}

/// A bounded reason why a numeric value cannot be scaled.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(super) enum ScaleError {
    /// The input or rounded result is NaN or infinite.
    #[error("scaling input or result is not finite")]
    NonFinite,
    /// The decimal exponent is outside the supported range.
    #[error(
        "scale_power10 must be between {} and {}",
        MIN_SCALE_POWER10,
        MAX_SCALE_POWER10
    )]
    InvalidScale,
}

/// Scales an integer by a power of ten.
/// Zero scale preserves the integer.
/// Nonzero scales return a double; magnitudes above 2^53 may round during conversion.
/// Powers outside the configuration's supported range return an error.
pub(super) fn scale_integer(value: i64, power10: i32) -> Result<Number, ScaleError> {
    if power10 == 0 {
        return Ok(Number::Integer(value));
    }
    scale_double(value as f64, power10)
}

/// Scales a calculated value by a power of ten without clamping.
/// Uses normal f64 rounding and accepts subnormals and signed zero.
/// Non-finite inputs or results and unsupported powers return an error.
pub(super) fn scale_double(value: f64, power10: i32) -> Result<Number, ScaleError> {
    if !value.is_finite() {
        return Err(ScaleError::NonFinite);
    }
    // The supported powers of ten are exact in f64, so this cast does not round.
    let factor = scale_factor(power10)? as f64;
    let scaled = if power10 < 0 {
        value / factor
    } else {
        value * factor
    };
    if !scaled.is_finite() {
        return Err(ScaleError::NonFinite);
    }
    Ok(Number::Double(scaled))
}

/// Returns 10 raised to the absolute value of a supported power.
fn scale_factor(power10: i32) -> Result<i64, ScaleError> {
    if !(MIN_SCALE_POWER10..=MAX_SCALE_POWER10).contains(&power10) {
        return Err(ScaleError::InvalidScale);
    }
    // Keep the overflow check in case the supported range is widened.
    10_i64
        .checked_pow(power10.unsigned_abs())
        .ok_or(ScaleError::InvalidScale)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Integer values use zero, positive, negative and boundary decimal scales.
    /// Guarantees: Zero scale preserves signed integers; every nonzero scale returns a double.
    #[test]
    fn scales_direct_values_by_decimal_power() {
        assert_eq!(
            scale_integer(i64::MAX, 0).unwrap(),
            Number::Integer(i64::MAX)
        );
        assert_eq!(
            scale_integer(i64::MIN, 0).unwrap(),
            Number::Integer(i64::MIN)
        );
        assert_eq!(scale_integer(42, 3).unwrap(), Number::Double(42_000.0));
        assert_eq!(scale_integer(-42, 3).unwrap(), Number::Double(-42_000.0));
        assert_eq!(scale_integer(9, 18).unwrap(), Number::Double(9e18));
        assert_eq!(scale_integer(-9, 18).unwrap(), Number::Double(-9e18));
        assert_eq!(scale_integer(0, 18).unwrap(), Number::Double(0.0));
        assert_eq!(scale_integer(42, -1).unwrap(), Number::Double(4.2));
        assert_eq!(scale_integer(-42, -1).unwrap(), Number::Double(-4.2));
        assert_eq!(scale_integer(3, -1).unwrap(), Number::Double(0.3));
        assert_eq!(scale_integer(7, -1).unwrap(), Number::Double(0.7));
        assert_eq!(
            scale_integer(86_580_518_913, -6).unwrap(),
            Number::Double(86_580.518_913)
        );
        assert_eq!(
            scale_integer(1_000_000_000_000_000_000, -18).unwrap(),
            Number::Double(1.0)
        );
        assert_eq!(scale_integer(0, -18).unwrap(), Number::Double(0.0));
    }

    /// Scenario: Integer extrema are scaled at both supported exponent limits.
    /// Guarantees: Large scaled values remain finite doubles rather than overflowing or being omitted.
    #[test]
    fn scales_integer_extremes_without_overflow() {
        assert_eq!(scale_integer(10, 18).unwrap(), Number::Double(1e19));
        assert_eq!(
            scale_integer(i64::MAX, 18).unwrap(),
            Number::Double(9.223_372_036_854_776e36)
        );
        assert_eq!(
            scale_integer(i64::MIN, 18).unwrap(),
            Number::Double(-9.223_372_036_854_776e36)
        );
        assert_eq!(
            scale_integer(i64::MAX, -18).unwrap(),
            Number::Double(9.223_372_036_854_776)
        );
        assert_eq!(
            scale_integer(i64::MIN, -18).unwrap(),
            Number::Double(-9.223_372_036_854_776)
        );
    }

    /// Scenario: Calculated values include percentages above 100, signed fractions and small magnitudes.
    /// Guarantees: Scaling preserves these values without clamping, integer coercion or discarding subnormals.
    #[test]
    fn scales_calculated_values_without_clamping() {
        assert_eq!(scale_double(250.0, 0).unwrap(), Number::Double(250.0));
        assert_eq!(scale_double(250.0, -2).unwrap(), Number::Double(2.5));
        assert_eq!(scale_double(-12.5, -1).unwrap(), Number::Double(-1.25));
        assert_eq!(scale_double(1.25, 1).unwrap(), Number::Double(12.5));
        assert_eq!(scale_double(1.0, 18).unwrap(), Number::Double(1e18));
        assert_eq!(scale_double(1.0, -18).unwrap(), Number::Double(1e-18));
        assert_eq!(scale_double(0.0, -18).unwrap(), Number::Double(0.0));
        assert_eq!(scale_double(1e-300, -18).unwrap(), Number::Double(1e-318));
        assert_eq!(scale_double(-1e-300, -18).unwrap(), Number::Double(-1e-318));
    }

    /// Scenario: Small calculated values round to zero, or a signed zero is scaled in either direction.
    /// Guarantees: Zero results retain the input's sign.
    #[test]
    fn preserves_signed_zero_when_scaling_underflows() {
        for value in [f64::MIN_POSITIVE, -f64::MIN_POSITIVE] {
            let Number::Double(scaled) = scale_double(value, -18).unwrap() else {
                panic!("calculated values must remain doubles");
            };
            assert_eq!(scaled, 0.0);
            assert_eq!(scaled.is_sign_negative(), value.is_sign_negative());
        }
        for value in [0.0_f64, -0.0_f64] {
            for power10 in [-18, 0, 18] {
                let Number::Double(scaled) = scale_double(value, power10).unwrap() else {
                    panic!("calculated values must remain doubles");
                };
                assert_eq!(scaled, 0.0);
                assert_eq!(scaled.is_sign_negative(), value.is_sign_negative());
            }
        }
    }

    /// Scenario: A calculated value is non-finite or scaling overflows the finite f64 range.
    /// Guarantees: Invalid observations return a bounded error rather than clamping or emitting infinities.
    #[test]
    fn rejects_non_finite_double_scaling() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for power10 in [-18, 0, 18] {
                assert_eq!(scale_double(value, power10), Err(ScaleError::NonFinite));
            }
        }
        assert_eq!(scale_double(f64::MAX, 18), Err(ScaleError::NonFinite));
        assert_eq!(scale_double(-f64::MAX, 18), Err(ScaleError::NonFinite));
    }

    /// Scenario: A scaling helper receives an exponent outside the configuration's supported range.
    /// Guarantees: Invalid powers return explicit errors without overflow, panic or non-finite output.
    #[test]
    fn rejects_unsupported_scale_powers() {
        for power10 in [i32::MIN, -19, 19, i32::MAX] {
            assert_eq!(
                scale_integer(i64::MAX, power10),
                Err(ScaleError::InvalidScale)
            );
            assert_eq!(scale_double(1.0, power10), Err(ScaleError::InvalidScale));
        }
    }
}
