// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exponential histogram scale mapping functions.
//!
//! This module provides the `Scale` struct which converts f64 values to
//! bucket indices. For scale <= 0, it uses direct exponent mapping. For
//! scale > 0, it uses the compile-time generated lookup table algorithm.
//! Scales above the compiled table scale are rejected by [`Scale::new`].

use core::fmt;

use crate::float64::{EXPONENT_BIAS, MAX_NORMAL_EXPONENT, MIN_NORMAL_EXPONENT, SIGNIFICAND_WIDTH};
use crate::lookup::BOUNDARIES;

/// Minimum scale for the exponent mapping.
/// At scale -10, values in (0, 1] map to bucket -1 and values in (1, MAX) map to bucket 0.
pub const MIN_SCALE: i32 = -10;

/// Error types for mapping operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScaleError {
    /// The bucket index corresponds to a subnormal value.
    Underflow,
    /// The bucket index corresponds to +Inf.
    Overflow,
    /// Invalid scale parameter.
    InvalidScale,
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Underflow => f.write_str("bucket index corresponds to a subnormal value"),
            Self::Overflow => f.write_str("bucket index corresponds to +Inf"),
            Self::InvalidScale => f.write_str("invalid scale parameter"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ScaleError {}

/// Returns the maximum scale supported by the mapping.
///
/// This equals the compiled lookup table scale (set by the `scale-N`
/// feature). Exponent mapping (scale <= 0) is always available.
#[inline]
#[must_use]
pub const fn table_scale() -> i32 {
    crate::lookup::TABLE_SCALE
}

/// Converts values to bucket indices at a given scale.
///
/// Scale is constrained to `MIN_SCALE..=table_scale()` and stored as `i8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Scale(i8);

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Scale {
    /// Creates a new mapping for the given scale.
    ///
    /// Returns `ScaleError::InvalidScale` if scale is outside
    /// [`MIN_SCALE`]..=[`table_scale()`].
    pub fn new(scale: i32) -> Result<Self, ScaleError> {
        if !(MIN_SCALE..=table_scale()).contains(&scale) {
            return Err(ScaleError::InvalidScale);
        }

        Ok(Self(scale as i8))
    }

    /// Returns the current scale as `i32`.
    #[inline]
    #[must_use]
    pub const fn scale(&self) -> i32 {
        self.0 as i32
    }

    /// Maps a f64 value to a bucket index. Ignores sign.
    #[inline]
    #[must_use]
    pub fn map_decomposed(&self, significand: u64, base2_exp: i32) -> i32 {
        let scale = self.scale();

        if scale <= 0 {
            crate::exponent::map_decomposed(significand, base2_exp, scale)
        } else {
            crate::lookup::map_decomposed(significand, base2_exp, scale)
        }
    }

    /// Maps a f64 value to a bucket index. Ignores sign.
    ///
    /// Handles subnormals by clamping to `MIN_VALUE`.
    #[must_use]
    pub fn map_to_index(&self, mut value: f64) -> i32 {
        debug_assert!(!value.is_infinite());
        debug_assert!(!value.is_nan());
        if value < crate::float64::MIN_VALUE {
            // Subnormal case is ordinarily handled in histogram/mod.rs
            value = crate::float64::MIN_VALUE;
        }
        self.map_decomposed(
            crate::float64::get_significand(value),
            crate::float64::get_unbiased_exponent(value),
        )
    }

    /// Constructs 2^k as an f64 from bits.
    #[inline]
    const fn pow2(k: i32, s: u64) -> Result<f64, ScaleError> {
        if k < MIN_NORMAL_EXPONENT {
            Err(ScaleError::Underflow)
        } else if k > MAX_NORMAL_EXPONENT {
            Err(ScaleError::Overflow)
        } else {
            let biased = (k + EXPONENT_BIAS) as u64;
            Ok(f64::from_bits(biased << SIGNIFICAND_WIDTH | s))
        }
    }

    /// Constructs a mask for index position between [0, 2^S)
    const fn mask(&self) -> usize {
        let scale = self.0 as usize;
        (1 << scale) - 1
    }

    /// Returns the lower boundary of a bucket at the given index.
    #[inline]
    pub fn lower_boundary(&self, index: i32) -> Result<f64, ScaleError> {
        let scale = self.0 as i32;
        if scale <= 0 {
            let shift = -scale;
            return Self::pow2(index << shift, 0);
        }

        let offset = index >> scale;
        let subidx = index as usize & self.mask();
        if subidx == 0 {
            Self::pow2(offset, 0)
        } else {
            Self::pow2(offset, BOUNDARIES[(subidx << (table_scale() - scale)) + 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smallest f64 strictly greater than a finite positive `x`.
    fn next_up(x: f64) -> f64 {
        f64::from_bits(x.to_bits() + 1)
    }

    /// Largest f64 strictly less than a finite positive `x`.
    fn next_down(x: f64) -> f64 {
        f64::from_bits(x.to_bits() - 1)
    }

    /// Scenario: `lower_boundary` is evaluated at every supported scale over a
    /// wide index range, and the representable values straddling each boundary
    /// are fed back through `map_to_index`.
    /// Guarantees: each boundary is the shared edge between adjacent buckets --
    /// the value just below it maps to `index - 1` and the value just above it
    /// maps to `index` -- proving the table-built boundary agrees with the
    /// mapping to within one ULP.
    #[test]
    fn lower_boundary_brackets_mapping() {
        for scale in MIN_SCALE..=table_scale() {
            let s = Scale::new(scale).unwrap();
            for index in -1500..=1500 {
                let b = match s.lower_boundary(index) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                assert!(b.is_finite() && b > 0.0, "scale={scale} index={index}");

                // Just below the boundary belongs to the bucket below. Skip
                // when the predecessor would be subnormal (clamped by mapping).
                if b > crate::float64::MIN_VALUE {
                    assert_eq!(
                        s.map_to_index(next_down(b)),
                        index - 1,
                        "below maps low: scale={scale} index={index} b={b:e}"
                    );
                }

                // Just above the boundary belongs to the bucket itself.
                let up = next_up(b);
                if up.is_finite() {
                    assert_eq!(
                        s.map_to_index(up),
                        index,
                        "above maps in: scale={scale} index={index} up={up:e}"
                    );
                }
            }
        }
    }

    /// Scenario: `lower_boundary` results are compared with the analytic value
    /// 2^(index / 2^scale) computed independently through `exp2`.
    /// Guarantees: the table-driven result agrees with the mathematical
    /// definition to a tight relative tolerance, ruling out table-indexing or
    /// offset-decomposition mistakes (an off-by-one bucket differs by ~0.27%).
    #[test]
    fn lower_boundary_matches_analytic_value() {
        for scale in MIN_SCALE..=table_scale() {
            let s = Scale::new(scale).unwrap();
            let denom = (2.0_f64).powi(scale);
            for index in -1500..=1500 {
                let b = match s.lower_boundary(index) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let reference = (index as f64 / denom).exp2();
                if !reference.is_finite() || reference <= 0.0 {
                    continue;
                }
                let rel = (b - reference).abs() / reference;
                assert!(
                    rel < 1e-9,
                    "scale={scale} index={index} got={b:e} want={reference:e} rel={rel:e}"
                );
            }
        }
    }

    /// Scenario: boundaries are inspected at exact octave positions and one
    /// octave apart for every non-negative scale.
    /// Guarantees: indices that are integer multiples of 2^scale yield exact
    /// powers of two, and advancing the index by 2^scale doubles the boundary
    /// exactly, confirming clean octave alignment with no rounding drift.
    #[test]
    fn lower_boundary_octaves_are_exact_powers_of_two() {
        for scale in 0..=table_scale() {
            let s = Scale::new(scale).unwrap();
            let step = 1_i32 << scale;
            for k in -60..=60 {
                let index = k * step;
                let b = s.lower_boundary(index).unwrap();
                assert_eq!(b, (2.0_f64).powi(k), "power: scale={scale} k={k}");

                if let Ok(next_octave) = s.lower_boundary(index + step) {
                    assert_eq!(next_octave, 2.0 * b, "double: scale={scale} k={k}");
                }
            }
        }
    }

    /// Scenario: consecutive boundaries are read across several octaves at the
    /// finest table scale.
    /// Guarantees: bucket edges are strictly increasing, a prerequisite for
    /// correct upper-inclusive bucketing and monotone quantile interpolation.
    #[test]
    fn lower_boundary_is_strictly_increasing() {
        let s = Scale::new(table_scale()).unwrap();
        let mut prev = s.lower_boundary(-1024).unwrap();
        for index in -1023..=1024 {
            let b = s.lower_boundary(index).unwrap();
            assert!(b > prev, "monotonic: index={index} b={b:e} prev={prev:e}");
            prev = b;
        }
    }

    /// Scenario: `lower_boundary` is pushed past the representable f64 range at
    /// the finest table scale.
    /// Guarantees: indices whose octave exceeds the maximum normal exponent
    /// report `Overflow`, and indices below the minimum normal exponent report
    /// `Underflow`, so quantile callers can rely on these errors.
    #[test]
    fn lower_boundary_reports_overflow_and_underflow() {
        let s = Scale::new(table_scale()).unwrap();
        let scale = table_scale();
        // Octave just past MAX_NORMAL_EXPONENT overflows.
        let overflow_index = (MAX_NORMAL_EXPONENT + 1) << scale;
        assert_eq!(s.lower_boundary(overflow_index), Err(ScaleError::Overflow));
        // Octave just below MIN_NORMAL_EXPONENT underflows.
        let underflow_index = (MIN_NORMAL_EXPONENT - 1) << scale;
        assert_eq!(
            s.lower_boundary(underflow_index),
            Err(ScaleError::Underflow)
        );
    }

    /// Scenario: at scale 1 (base = sqrt(2)) a spread of values across the
    /// octave [1, 2] is mapped to bucket indices, and the lower boundaries of
    /// buckets -1..=2 are read back.
    /// Guarantees: mapping obeys the OpenTelemetry upper-inclusive rule --
    /// values sitting exactly on a lower boundary (the exact powers 1.0 =
    /// base^0 and 2.0 = base^2) fall in the bucket below, yielding the
    /// sequence -1, 0, 0, 1, 1, 1 -- and the boundaries are the sqrt(2)-spaced
    /// powers of the base: sqrt(2)/2, 1, sqrt(2), 2.
    #[test]
    fn scale_one_maps_values_and_boundaries() {
        let s = Scale::new(1).unwrap();

        // Buckets span (base^i, base^(i+1)] with base = sqrt(2). A value equal
        // to a lower boundary (1.0 and 2.0) belongs to the bucket below.
        for (value, want) in [
            (1.0_f64, -1),
            (1.1, 0),
            (1.4, 0),
            (1.5, 1),
            (1.9, 1),
            (2.0, 1),
        ] {
            assert_eq!(s.map_to_index(value), want, "map_to_index({value})");
        }

        // Only a value strictly above the 2.0 boundary reaches bucket 2.
        assert_eq!(s.map_to_index(next_up(2.0)), 2);

        // Lower boundaries are 2^(index/2): sqrt(2)/2, 1, sqrt(2), 2.
        let sqrt2 = core::f64::consts::SQRT_2;
        assert!((s.lower_boundary(-1).unwrap() - sqrt2 / 2.0).abs() < 1e-15);
        assert_eq!(s.lower_boundary(0).unwrap(), 1.0);
        assert!((s.lower_boundary(1).unwrap() - sqrt2).abs() < 1e-15);
        assert_eq!(s.lower_boundary(2).unwrap(), 2.0);
    }
}
