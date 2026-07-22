// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! SWAR: SIMD within a register

use super::width::Width;

// ── Single-step SWAR primitives ──────────────────────────────────────

/// B1 → B2: sum pairs of 1-bit lanes.
#[inline(always)]
fn step_b1_b2(w: u64) -> u64 {
    (w & 0x5555_5555_5555_5555) + ((w >> 1) & 0x5555_5555_5555_5555)
}

/// B2 → B4: sum pairs of 2-bit lanes.
#[inline(always)]
fn step_b2_b4(w: u64) -> u64 {
    (w & 0x3333_3333_3333_3333) + ((w >> 2) & 0x3333_3333_3333_3333)
}

/// B4 → U8: sum pairs of 4-bit lanes.
#[inline(always)]
fn step_b4_u8(w: u64) -> u64 {
    (w & 0x0F0F_0F0F_0F0F_0F0F) + ((w >> 4) & 0x0F0F_0F0F_0F0F_0F0F)
}

/// U8 → U16: sum pairs of 8-bit lanes.
#[inline(always)]
fn step_u8_u16(w: u64) -> u64 {
    (w & 0x00FF_00FF_00FF_00FF) + ((w >> 8) & 0x00FF_00FF_00FF_00FF)
}

/// U16 → U32: sum pairs of 16-bit lanes.
#[inline(always)]
fn step_u16_u32(w: u64) -> u64 {
    (w & 0x0000_FFFF_0000_FFFF) + ((w >> 16) & 0x0000_FFFF_0000_FFFF)
}

/// U32 → U64: sum pair of 32-bit lanes.
#[inline(always)]
fn step_u32_u64(w: u64) -> u64 {
    (w & 0x0000_0000_FFFF_FFFF) + (w >> 32)
}

// ── Public API ───────────────────────────────────────────────────────

/// Widen a single u64 word in place from `before` lane width to
/// `after` lane width by chaining SWAR pair-sum steps.
/// Uses count_ones() shortcuts for B1→U32 and B1→U64.
#[inline]
pub(crate) fn widen(before: Width, after: Width, w: u64) -> u64 {
    use Width::*;
    debug_assert!(before < after);
    match (before, after) {
        // B1 → *
        (B1, B2) => step_b1_b2(w),
        (B1, B4) => step_b2_b4(step_b1_b2(w)),
        (B1, U8) => step_b4_u8(step_b2_b4(step_b1_b2(w))),
        (B1, U16) => step_u8_u16(step_b4_u8(step_b2_b4(step_b1_b2(w)))),
        (B1, U32) => {
            let lo = (w as u32).count_ones() as u64;
            let hi = ((w >> 32) as u32).count_ones() as u64;
            lo | (hi << 32)
        }
        (B1, U64) => w.count_ones() as u64,

        // B2 → *
        (B2, B4) => step_b2_b4(w),
        (B2, U8) => step_b4_u8(step_b2_b4(w)),
        (B2, U16) => step_u8_u16(step_b4_u8(step_b2_b4(w))),
        (B2, U32) => step_u16_u32(step_u8_u16(step_b4_u8(step_b2_b4(w)))),
        (B2, U64) => step_u32_u64(step_u16_u32(step_u8_u16(step_b4_u8(step_b2_b4(w))))),

        // B4 → *
        (B4, U8) => step_b4_u8(w),
        (B4, U16) => step_u8_u16(step_b4_u8(w)),
        (B4, U32) => step_u16_u32(step_u8_u16(step_b4_u8(w))),
        (B4, U64) => step_u32_u64(step_u16_u32(step_u8_u16(step_b4_u8(w)))),

        // U8 → *
        (U8, U16) => step_u8_u16(w),
        (U8, U32) => step_u16_u32(step_u8_u16(w)),
        (U8, U64) => step_u32_u64(step_u16_u32(step_u8_u16(w))),

        // U16 → *
        (U16, U32) => step_u16_u32(w),
        (U16, U64) => step_u32_u64(step_u16_u32(w)),

        // U32 → U64
        (U32, U64) => step_u32_u64(w),

        _ => unreachable!(),
    }
}

// ── Single-step SWAR narrow primitives ───────────────────────────────
//
// Each function masks a u64 word to keep only the target-width low bits
// in each source-width lane, then compacts the values by removing the
// interleaved gaps.  The result occupies the low 32 bits of the u64.

/// B2 → B1: keep low 1 bit of each 2-bit lane, compact 32 values.
#[inline(always)]
fn nstep_b2_b1(w: u64) -> u64 {
    let w = w & 0x5555_5555_5555_5555;
    let w = (w | (w >> 1)) & 0x3333_3333_3333_3333;
    let w = (w | (w >> 2)) & 0x0F0F_0F0F_0F0F_0F0F;
    let w = (w | (w >> 4)) & 0x00FF_00FF_00FF_00FF;
    let w = (w | (w >> 8)) & 0x0000_FFFF_0000_FFFF;
    (w | (w >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// B4 → B2: keep low 2 bits of each 4-bit lane, compact 16 values.
#[inline(always)]
fn nstep_b4_b2(w: u64) -> u64 {
    let w = w & 0x3333_3333_3333_3333;
    let w = (w | (w >> 2)) & 0x0F0F_0F0F_0F0F_0F0F;
    let w = (w | (w >> 4)) & 0x00FF_00FF_00FF_00FF;
    let w = (w | (w >> 8)) & 0x0000_FFFF_0000_FFFF;
    (w | (w >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// U8 → B4: keep low 4 bits of each 8-bit lane, compact 8 values.
#[inline(always)]
fn nstep_u8_b4(w: u64) -> u64 {
    let w = w & 0x0F0F_0F0F_0F0F_0F0F;
    let w = (w | (w >> 4)) & 0x00FF_00FF_00FF_00FF;
    let w = (w | (w >> 8)) & 0x0000_FFFF_0000_FFFF;
    (w | (w >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// U16 → U8: keep low 8 bits of each 16-bit lane, compact 4 values.
#[inline(always)]
fn nstep_u16_u8(w: u64) -> u64 {
    let w = w & 0x00FF_00FF_00FF_00FF;
    let w = (w | (w >> 8)) & 0x0000_FFFF_0000_FFFF;
    (w | (w >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// U32 → U16: keep low 16 bits of each 32-bit lane, compact 2 values.
#[inline(always)]
fn nstep_u32_u16(w: u64) -> u64 {
    let w = w & 0x0000_FFFF_0000_FFFF;
    (w | (w >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// U64 → U32: keep low 32 bits.
#[inline(always)]
fn nstep_u64_u32(w: u64) -> u64 {
    w & 0x0000_0000_FFFF_FFFF
}

/// Narrow a single u64 word in place from `before` lane width to
/// `after` lane width by chaining SWAR mask-and-compact steps.
///
/// Each source-width lane is truncated to `after` bits and the values
/// are packed contiguously starting from bit 0.  The result occupies
/// the low `64 × after_bits / before_bits` bits of the returned u64.
#[inline]
pub(crate) fn narrow(before: Width, after: Width, w: u64) -> u64 {
    use Width::*;
    debug_assert!(before > after);
    match (before, after) {
        // B2 → *
        (B2, B1) => nstep_b2_b1(w),

        // B4 → *
        (B4, B2) => nstep_b4_b2(w),
        (B4, B1) => nstep_b2_b1(nstep_b4_b2(w)),

        // U8 → *
        (U8, B4) => nstep_u8_b4(w),
        (U8, B2) => nstep_b4_b2(nstep_u8_b4(w)),
        (U8, B1) => nstep_b2_b1(nstep_b4_b2(nstep_u8_b4(w))),

        // U16 → *
        (U16, U8) => nstep_u16_u8(w),
        (U16, B4) => nstep_u8_b4(nstep_u16_u8(w)),
        (U16, B2) => nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(w))),
        (U16, B1) => nstep_b2_b1(nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(w)))),

        // U32 → *
        (U32, U16) => nstep_u32_u16(w),
        (U32, U8) => nstep_u16_u8(nstep_u32_u16(w)),
        (U32, B4) => nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(w))),
        (U32, B2) => nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(w)))),
        (U32, B1) => nstep_b2_b1(nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(w))))),

        // U64 → *
        (U64, U32) => nstep_u64_u32(w),
        (U64, U16) => nstep_u32_u16(nstep_u64_u32(w)),
        (U64, U8) => nstep_u16_u8(nstep_u32_u16(nstep_u64_u32(w))),
        (U64, B4) => nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(nstep_u64_u32(w)))),
        (U64, B2) => nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(nstep_u64_u32(w))))),
        (U64, B1) => nstep_b2_b1(nstep_b4_b2(nstep_u8_b4(nstep_u16_u8(nstep_u32_u16(
            nstep_u64_u32(w),
        ))))),

        _ => unreachable!(),
    }
}

/// SWAR addition with per-lane overflow detection.
///
/// Returns `Some(a + b)` if no lane overflows, or `None` if any lane
/// would exceed its maximum value.  Neither input is modified on failure.
///
/// Algorithm: zero the MSB of each lane, add the lower bits (which
/// cannot carry across lanes), then detect overflow via the majority
/// function of (a_msb, b_msb, carry_from_below).
#[inline]
pub(crate) fn swar_add_checked(a: u64, b: u64, width: Width) -> Option<u64> {
    if width == Width::U64 {
        return a.checked_add(b);
    }
    let msb = width.msb_mask();
    let a_lo = a & !msb;
    let b_lo = b & !msb;
    let sum_lo = a_lo + b_lo; // no inter-lane carry (MSB was zeroed)
    let carry = sum_lo & msb;
    let a_hi = a & msb;
    let b_hi = b & msb;
    // Overflow in a lane iff at least 2 of {a_msb, b_msb, carry} are set.
    let overflow = (a_hi & b_hi) | (a_hi & carry) | (b_hi & carry);
    if overflow != 0 {
        return None;
    }
    Some(sum_lo ^ a_hi ^ b_hi)
}

impl Width {
    /// Mask with the MSB of each SWAR lane set.
    #[inline]
    #[must_use]
    pub(crate) const fn msb_mask(self) -> u64 {
        use Width::{B1, B2, B4, U8, U16, U32, U64};
        match self {
            B1 => 0xFFFF_FFFF_FFFF_FFFF,
            B2 => 0xAAAA_AAAA_AAAA_AAAA,
            B4 => 0x8888_8888_8888_8888,
            U8 => 0x8080_8080_8080_8080,
            U16 => 0x8000_8000_8000_8000,
            U32 => 0x8000_0000_8000_0000,
            U64 => 0x8000_0000_0000_0000,
        }
    }

    /// OR-fold all SWAR lanes within a word into a single representative
    /// value.  The result has the same highest-set-bit as the true
    /// maximum lane, so `Width::from_max_value(or_fold_lanes(w, word))`
    /// gives the exact minimum width needed to hold any lane.
    #[inline]
    pub(crate) fn or_fold_lanes(self, w: u64) -> u64 {
        use Width::{B1, B2, B4, U8, U16, U32, U64};
        match self {
            B1 => (w != 0) as u64,
            B2 => {
                let w = w | (w >> 2);
                let w = w | (w >> 4);
                let w = w | (w >> 8);
                let w = w | (w >> 16);
                (w | (w >> 32)) & 0x3
            }
            B4 => {
                let w = w | (w >> 4);
                let w = w | (w >> 8);
                let w = w | (w >> 16);
                (w | (w >> 32)) & 0xF
            }
            U8 => {
                let w = w | (w >> 8);
                let w = w | (w >> 16);
                (w | (w >> 32)) & 0xFF
            }
            U16 => {
                let w = w | (w >> 16);
                (w | (w >> 32)) & 0xFFFF
            }
            U32 => (w | (w >> 32)) & 0xFFFF_FFFF,
            U64 => w,
        }
    }
}
