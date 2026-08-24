// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lookup table-based mapping for exponential histograms.
//!
//! This module provides compile-time generated boundary and index tables
//! at the highest compiled-in scale. The algorithm uses 2N linear buckets
//! per scale with one boundary correction.

include!("lookup_tables.rs");

/// Maps a positive f64 value to a bucket index using a compiled lookup table.
///
/// Always computes at `TABLE_SCALE` using the full boundary and index
/// tables, then shifts down to the requested scale.
#[inline]
pub(crate) fn map_decomposed(significand: u64, base2_exp: i32, scale: i32) -> i32 {
    debug_assert!(scale > 0);
    debug_assert!(scale <= TABLE_SCALE);
    debug_assert_eq!(51, TABLE_SCALE as u32 + INDEX_SHIFT);

    let linear_idx = (significand >> INDEX_SHIFT) as usize;
    let approx = INDEX_TABLE[linear_idx] as usize;

    let mut bucket = approx as i32;
    // A sentinel case at 0 implements upper-inclusivity.
    if significand >= BOUNDARIES[approx + 1] {
        bucket += 1;
    }

    let fine = (base2_exp << TABLE_SCALE) + bucket - 1;
    fine >> (TABLE_SCALE - scale)
}

/// Returns the compiled-in scale of the lookup table. The exact lookup
/// table has 2^table_scale() entries.
#[inline]
#[must_use]
pub const fn table_scale() -> i32 {
    TABLE_SCALE
}
