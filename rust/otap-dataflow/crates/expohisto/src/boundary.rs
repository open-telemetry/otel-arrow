// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bucket boundary computation.
//!
//! Converts bucket indices back to f64 boundaries.  For scale ≤ 0 the
//! boundaries are exact powers of two.  For positive scales, they are
//! computed via `exp(index * ln(2) / 2^scale)` using a precomputed
//! inverse-factor table covering scales 1..=20.
//!
//! This entire module is gated behind `#[cfg(feature = "boundary")]`.

use crate::float64::{MAX_NORMAL_EXPONENT, MIN_NORMAL_EXPONENT, MIN_VALUE, pow2};
use crate::mapping::{Scale, ScaleError};

include!("inverse_factors.rs");

/// Maximum valid bucket index for `lower_boundary` at a non-positive scale.
#[inline]
const fn max_normal_index_exp(scale: i32) -> i32 {
    let shift = (-scale) as u32;
    MAX_NORMAL_EXPONENT >> shift
}

/// Minimum valid bucket index for `lower_boundary` at a non-positive scale.
#[inline]
const fn min_normal_lower_boundary_index(scale: i32) -> i32 {
    let shift = (-scale) as u32;
    MIN_NORMAL_EXPONENT >> shift
}

/// Returns the lower boundary of a bucket at non-positive scale.
fn lower_boundary_exponent(index: i32, scale: i32) -> Result<f64, ScaleError> {
    debug_assert!(scale <= 0);
    let shift = (-scale) as u32;

    if index < min_normal_lower_boundary_index(scale) {
        return Err(ScaleError::Underflow);
    }
    if index > max_normal_index_exp(scale) {
        return Err(ScaleError::Overflow);
    }

    Ok(pow2(index << shift))
}

/// Minimum valid bucket index for `lower_boundary` at a positive scale.
#[inline]
const fn min_normal_index_log(scale: i32) -> i32 {
    MIN_NORMAL_EXPONENT << scale
}

/// Maximum valid bucket index for `lower_boundary` at a positive scale.
#[inline]
const fn max_normal_index_log(scale: i32) -> i32 {
    ((MAX_NORMAL_EXPONENT + 1) << scale) - 1
}

/// Returns the lower boundary of a bucket at positive scale.
fn lower_boundary_logarithm(index: i32, scale: i32) -> Result<f64, ScaleError> {
    debug_assert!((1..=INVERSE_FACTOR.len()).contains(&(scale as usize)));
    let inv = INVERSE_FACTOR[scale as usize - 1];
    let max_idx = max_normal_index_log(scale);
    let min_idx = min_normal_index_log(scale);

    if index >= max_idx {
        if index == max_idx {
            return Ok(2.0 * crate::float64::exp((index - (1 << scale)) as f64 * inv));
        }
        return Err(ScaleError::Overflow);
    }

    if index <= min_idx {
        if index == min_idx {
            return Ok(MIN_VALUE);
        } else if index == min_idx - 1 {
            return Ok(crate::float64::exp((index + (1 << scale)) as f64 * inv) / 2.0);
        }
        return Err(ScaleError::Underflow);
    }

    Ok(crate::float64::exp(index as f64 * inv))
}

impl Scale {
    /// Returns the lower boundary of a bucket at the given index.
    ///
    /// For scale <= 0 this is an exact power of two.  For positive
    /// scales, uses `exp()` with a precomputed inverse factor.
    #[inline]
    pub fn lower_boundary(&self, index: i32) -> Result<f64, ScaleError> {
        if self.scale() <= 0 {
            lower_boundary_exponent(index, self.scale())
        } else {
            lower_boundary_logarithm(index, self.scale())
        }
    }
}
