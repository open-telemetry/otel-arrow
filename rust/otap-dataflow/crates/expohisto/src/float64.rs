// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! IEEE 754 double-precision floating-point constants and utilities.

/// Size of an IEEE 754 double-precision floating-point significand.
pub(crate) const SIGNIFICAND_WIDTH: u32 = 52;

/// Size of an IEEE 754 double-precision floating-point exponent.
pub(crate) const EXPONENT_WIDTH: u32 = 11;

/// Mask for the significand of an IEEE 754 double-precision value: 0xFFFFFFFFFFFFF.
pub(crate) const SIGNIFICAND_MASK: u64 = (1 << SIGNIFICAND_WIDTH) - 1;

/// Exponent bias for IEEE 754 double-precision: 1023.
pub(crate) const EXPONENT_BIAS: i32 = f64::MAX_EXP - 1;

/// Exponent value for IEEE 754 NaN and Inf values: 2047.
pub(crate) const NAN_INF_BIASED: u32 = 2 * f64::MAX_EXP as u32 - 1;

/// Mask for the exponent bits: 0x7FF0000000000000.
pub(crate) const EXPONENT_MASK: u64 = ((1u64 << EXPONENT_WIDTH) - 1) << SIGNIFICAND_WIDTH;

/// Minimum exponent of a normalized floating point: -1022.
pub(crate) const MIN_NORMAL_EXPONENT: i32 = -EXPONENT_BIAS + 1;

/// Maximum exponent of a normalized floating point: 1023.
pub(crate) const MAX_NORMAL_EXPONENT: i32 = EXPONENT_BIAS;

/// Smallest normal f64 value: 2^-1022 (same as `f64::MIN_POSITIVE`).
pub(crate) const MIN_VALUE: f64 = f64::MIN_POSITIVE;

/// Extracts the unbiased base-2 exponent from an f64.
#[inline]
pub(crate) const fn get_unbiased_exponent(value: f64) -> i32 {
    unbias_exponent(get_biased_exponent(value))
}

/// Removes the bias from the f64 exponent value.
#[inline]
pub(crate) const fn unbias_exponent(biased: u32) -> i32 {
    biased as i32 - EXPONENT_BIAS
}

/// Extracts the biased base-2 exponent from an f64. Ignores sign bit.
/// Return value 0 indicates +/-0 or subnormal. Return value 2047 indicates
/// Inf or NaN.
#[inline]
pub(crate) const fn get_biased_exponent(value: f64) -> u32 {
    ((value.to_bits() & EXPONENT_MASK) >> SIGNIFICAND_WIDTH) as u32
}

/// Returns the 52-bit significand as an unsigned value.
#[inline]
pub(crate) const fn get_significand(value: f64) -> u64 {
    value.to_bits() & SIGNIFICAND_MASK
}
