// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! IEEE 754 double-precision floating-point constants and utilities.
//!
//! An optional math layer (`exp`, `ln`, `floor`) is compiled when the
//! `boundary` feature is active, using `std` f64 methods.

/// Size of an IEEE 754 double-precision floating-point significand.
pub const SIGNIFICAND_WIDTH: u32 = 52;

/// Size of an IEEE 754 double-precision floating-point exponent.
pub const EXPONENT_WIDTH: u32 = 11;

/// Mask for the significand of an IEEE 754 double-precision value: 0xFFFFFFFFFFFFF.
pub const SIGNIFICAND_MASK: u64 = (1 << SIGNIFICAND_WIDTH) - 1;

/// Exponent bias for IEEE 754 double-precision: 1023.
pub const EXPONENT_BIAS: i32 = f64::MAX_EXP - 1;

/// Exponent value for IEEE 754 NaN and Inf values: 2047.
pub const NAN_INF_BIASED: u32 = 2 * f64::MAX_EXP as u32 - 1;

/// Mask for the exponent bits: 0x7FF0000000000000.
pub const EXPONENT_MASK: u64 = ((1u64 << EXPONENT_WIDTH) - 1) << SIGNIFICAND_WIDTH;

/// Minimum exponent of a normalized floating point: -1022.
pub const MIN_NORMAL_EXPONENT: i32 = -EXPONENT_BIAS + 1;

/// Maximum exponent of a normalized floating point: 1023.
#[cfg_attr(not(feature = "boundary"), allow(dead_code))]
pub const MAX_NORMAL_EXPONENT: i32 = EXPONENT_BIAS;

/// Smallest normal f64 value: 2^-1022 (same as `f64::MIN_POSITIVE`).
pub const MIN_VALUE: f64 = f64::MIN_POSITIVE;

/// Const `f64::to_bits()` (const-stable since Rust 1.83).
#[inline]
pub const fn to_bits(v: f64) -> u64 {
    v.to_bits()
}

/// Const `f64::from_bits()` (const-stable since Rust 1.83).
#[inline]
pub const fn from_bits(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// Extracts the unbiased base-2 exponent from an f64.
#[inline]
pub const fn get_unbiased_exponent(value: f64) -> i32 {
    unbias_exponent(get_biased_exponent(value))
}

/// Removes the bias from the f64 exponent value.
#[inline]
pub const fn unbias_exponent(biased: u32) -> i32 {
    biased as i32 - EXPONENT_BIAS
}

/// Extracts the biased base-2 exponent from an f64. Ignores sign bit.
/// Return value 0 indicates +/-0 or subnormal. Return value 2047 indicates
/// Inf or NaN.
#[inline]
pub const fn get_biased_exponent(value: f64) -> u32 {
    ((to_bits(value) & EXPONENT_MASK) >> SIGNIFICAND_WIDTH) as u32
}

/// Returns the 52-bit significand as an unsigned value.
#[inline]
pub const fn get_significand(value: f64) -> u64 {
    to_bits(value) & SIGNIFICAND_MASK
}

/// Constructs 2^k as an f64 using direct IEEE 754 bit manipulation.
///
/// Valid for k in \[`MIN_NORMAL_EXPONENT`, `MAX_NORMAL_EXPONENT`\] (i.e. −1022..=1023).
/// Panics in debug mode if k is out of range.
#[cfg_attr(not(feature = "boundary"), allow(dead_code))]
#[inline]
pub const fn pow2(k: i32) -> f64 {
    debug_assert!(
        k >= MIN_NORMAL_EXPONENT && k <= MAX_NORMAL_EXPONENT,
        "pow2 out of range"
    );
    let biased = (k + EXPONENT_BIAS) as u64;
    from_bits(biased << SIGNIFICAND_WIDTH)
}

// ── Math helpers (std only) ──────────────────────────────────────────
// Only compiled when the `boundary` feature is active, which implies `std`.

#[cfg(feature = "boundary")]
mod math_imp {
    #[inline]
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }
}

#[cfg(feature = "boundary")]
pub use math_imp::*;
