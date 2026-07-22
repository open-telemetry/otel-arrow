// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exponent-based mapping for exponential histograms (scale <= 0).
//!
//! At these scales, bucket boundaries are exact powers of two and the
//! mapping reduces to extracting the IEEE 754 exponent with a right-shift.
//! This is the simplest and fastest algorithm, always used for non-positive scales.

/// Maps a positive f64 value to a bucket index at a non-positive scale.
#[inline]
pub(crate) fn map_decomposed(significand: u64, base2_exp: i32, scale: i32) -> i32 {
    debug_assert!(scale <= 0);

    // Upper-inclusive correction: exact powers of two (significand == 0)
    // must map one bucket lower.
    let correction = if significand == 0 { -1 } else { 0 };

    // Arithmetic right shift handles negative exponents correctly
    (base2_exp + correction) >> -scale
}
