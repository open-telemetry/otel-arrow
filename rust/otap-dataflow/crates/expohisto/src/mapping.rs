// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exponential histogram scale mapping functions.
//!
//! This module provides the `Scale` struct which converts f64 values to
//! bucket indices. For scale <= 0, it uses direct exponent mapping. For
//! scale > 0, it uses the compile-time generated lookup table algorithm.
//! Scales above the compiled table scale are rejected by [`Scale::new`].

use core::fmt;

/// Minimum scale for the exponent mapping.
/// At scale -10, values in (0, 1] map to bucket -1 and values in (1, MAX) map to bucket 0.
pub const MIN_SCALE: i32 = -10;

/// Maximum scale supported is the finest resolution.
/// At scale 16, the index table requires 16-bit entries. Scales 17–20
/// would require wider table entries and are not currently supported.
pub const MAX_SCALE: i32 = 16;

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
/// feature). Exponent mapping (scale ≤ 0) is always available.
#[inline]
#[must_use]
pub const fn table_scale() -> i32 {
    crate::lookup::TABLE_SCALE
}

/// Converts values to bucket indices at a given scale.
///
/// Scale fits in −10..=16 and is stored as `i8` for compactness.
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
}
