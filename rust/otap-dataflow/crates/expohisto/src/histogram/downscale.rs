// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Downscale operations.

use super::HistogramNN;
use super::swar::{narrow, widen};
use super::width::{ALL_WIDTHS, Width};

impl<const N: usize> HistogramNN<N> {
    /// Widen every active word from `before` to `after` and return
    /// the OR-fold of all lanes at the new width.
    pub(crate) fn widen_words(&mut self, before: Width, after: Width) -> u64 {
        let mut total_or = 0u64;
        for widx in self.word_start..=self.word_end {
            let di = self.data_idx(widx);
            self.data[di] = widen(before, after, self.data[di]);
            total_or |= after.or_fold_lanes(self.data[di]);
        }
        total_or
    }

    /// Process one aligned group of `total_merge` input words into a
    /// single packed output word.  Reads use the current physical mapping.
    fn repack_group(
        &self,
        ostart: i32,
        change: u32,
        cross_steps: u32,
        cur: Width,
        output_width: Width,
    ) -> (i32, u64) {
        let narrow_steps = change - cross_steps;
        let group = 1i32 << cross_steps;
        let repack = 1i32 << narrow_steps;
        let chunk_bits = 64u32 >> narrow_steps;

        let out_widx = ostart >> change;
        let mut acc = 0u64;
        for r in 0..repack {
            let gstart = ostart + r * group;
            let mut value = 0u64;
            for g in 0..group {
                let widx = gstart + g;
                if widx >= self.word_start && widx <= self.word_end {
                    value += self.data[self.data_idx(widx)];
                }
            }
            let narrowed = narrow(cur, output_width, value);
            acc |= narrowed << (r as u32 * chunk_bits);
        }
        (out_widx, acc)
    }

    /// Process one aligned group for pure cross-word grouping (no narrowing).
    fn repack_group_cross(&self, ostart: i32, change: u32) -> (i32, u64) {
        let total_merge = 1i32 << change;

        let out_widx = ostart >> change;
        let mut sum = 0u64;
        for g in 0..total_merge {
            let widx = ostart + g;
            if widx >= self.word_start && widx <= self.word_end {
                sum += self.data[self.data_idx(widx)];
            }
        }
        (out_widx, sum)
    }

    /// Downscales the histogram by at least `change` scale steps.
    ///
    /// The output width will be at least `min_output_width`. This
    /// prevents the narrow step from undoing widening that the caller
    /// needs (e.g., the merge path needs counters wide enough for the
    /// source data).
    ///
    /// Returns the actual number of scale steps applied, which may
    /// exceed `change` when bucket sums require a wider output width.
    pub(super) fn do_downscale(&mut self, change: u32, min_output_width: Width) -> u32 {
        debug_assert!(change != 0);
        debug_assert!(!self.buckets_empty());

        let input_width = self.current.width;
        let to_u64 = input_width.to_u64_widen_steps();

        // Absolute budget: total scale steps must not push below
        // MIN_SCALE.
        let abs_budget = (self.current.scale.scale() - crate::mapping::MIN_SCALE) as u32;

        // Phase 1: Widen by up to `change` steps (capped at U64 and budget).
        let first_widen = change.min(to_u64).min(abs_budget);
        let mut cur = input_width;
        let mut total_widen = first_widen;
        let mut total_or = 0u64;

        if first_widen > 0 {
            cur = input_width.wider_by(first_widen).expect("capped at U64");
            total_or = self.widen_words(input_width, cur);
        }

        // Phase 2: Widen one step at a time until the gap between
        // current width and required width reaches `change` AND
        // we've reached at least min_output_width, or we exhaust
        // in-word widening at U64.
        loop {
            if cur == Width::U64 {
                break;
            }
            if total_widen >= abs_budget {
                break;
            }
            let required = Width::from_max_value(total_or);
            if cur.subtract(required) >= change as i32 && cur >= min_output_width {
                break;
            }

            let prev = cur;
            cur = cur.wider_by(1).expect("not yet U64");
            total_or = self.widen_words(prev, cur);
            total_widen += 1;
        }

        // Phase 3: Determine cross-word grouping steps.
        //
        // If the widen loop achieved gap >= change, no cross-word
        // grouping is needed (cross_steps = 0). Otherwise we are
        // at U64 and must sum consecutive words to make up the
        // difference. Each doubling adds at most 1 bit to the max,
        // so gap decreases by at most 1 per step while cross_steps
        // increases by 1 — the sum is non-decreasing and the loop
        // always terminates.
        //
        // The narrow cap from min_output_width relaxes the fit condition
        // (gap >= capped narrow_steps), but we also need enough total
        // scale steps (total_widen + cross_steps >= change).
        let max_narrow = (cur as u32).saturating_sub(min_output_width.max(input_width) as u32);
        let mut cross_steps = 0u32;

        if cur == Width::U64 {
            if total_or == 0 {
                // When width started at U64, phase 1 and 2 were skipped.
                for widx in self.word_start..=self.word_end {
                    total_or |= self.data[self.data_idx(widx)];
                }
            }
            let required = Width::from_max_value(total_or);
            let gap = cur.subtract(required) as u32;

            let narrow_needed = change.min(max_narrow);
            let scale_ok = total_widen >= change;
            if !scale_ok || gap < narrow_needed {
                loop {
                    if total_widen + cross_steps >= abs_budget {
                        // Hard floor: cannot consume more scale.
                        break;
                    }
                    cross_steps += 1;
                    let group_size = 1i32 << cross_steps;
                    let aligned = self.word_start & !(group_size - 1);
                    let mut or_sums = 0u64;
                    let mut gstart = aligned;
                    while gstart <= self.word_end {
                        let mut sum = 0u64;
                        for g in 0..group_size {
                            let widx = gstart + g;
                            if widx >= self.word_start && widx <= self.word_end {
                                sum += self.data[self.data_idx(widx)];
                            }
                        }
                        or_sums |= sum;
                        gstart += group_size;
                    }
                    let required = Width::from_max_value(or_sums);
                    let gap = Width::U64.subtract(required) as u32;
                    let narrow_needed = (change - cross_steps).min(max_narrow);
                    let scale_ok = total_widen + cross_steps >= change;
                    if scale_ok && gap >= narrow_needed {
                        break;
                    }
                }
            }
        }

        // Phase 4: Narrow and repack with two-pass clobber prevention.
        //
        // narrow_steps is capped by max_narrow so that the output width
        // never drops below min_output_width. word_shift is the actual
        // word-level compression (may be < change when capped).
        //
        // Also cap so the resulting scale can still afford widening to
        // U64: new_scale >= MIN_SCALE + (U64 - output_width), which
        // gives narrow_steps <= old_scale - total_widen - cross_steps
        //                       - MIN_SCALE.
        let headroom_cap = (self.current.scale.scale()
            - (total_widen + cross_steps) as i32
            - crate::mapping::MIN_SCALE) as u32;
        let narrow_steps = (change - cross_steps).min(max_narrow).min(headroom_cap);
        let word_shift = cross_steps + narrow_steps;
        let output_width = ALL_WIDTHS[cur as usize - narrow_steps as usize];

        let total_merge = 1i32 << word_shift;
        let new_word_base = self.word_base >> word_shift;

        // Write physical index under the shifted mapping.
        let write_idx =
            |out_widx: i32| -> usize { (out_widx - new_word_base).rem_euclid(N as i32) as usize };

        // The aligned group that contains word_base.
        let fwd_start = self.word_base & !(total_merge - 1);
        let rev_start = fwd_start - total_merge;
        let aligned_ws = self.word_start & !(total_merge - 1);

        if narrow_steps > 0 {
            // Forward pass: from the group containing word_base toward word_end.
            let mut ostart = fwd_start;
            while ostart <= self.word_end {
                let (out_widx, acc) =
                    self.repack_group(ostart, word_shift, cross_steps, cur, output_width);
                self.data[write_idx(out_widx)] = acc;
                ostart += total_merge;
            }

            // Reverse pass: from the group below word_base toward word_start.
            let mut ostart = rev_start;
            while ostart >= aligned_ws {
                let (out_widx, acc) =
                    self.repack_group(ostart, word_shift, cross_steps, cur, output_width);
                self.data[write_idx(out_widx)] = acc;
                ostart -= total_merge;
            }
        } else {
            // Pure cross-word grouping, output stays at U64.
            let mut ostart = fwd_start;
            while ostart <= self.word_end {
                let (out_widx, sum) = self.repack_group_cross(ostart, word_shift);
                self.data[write_idx(out_widx)] = sum;
                ostart += total_merge;
            }

            let mut ostart = rev_start;
            while ostart >= aligned_ws {
                let (out_widx, sum) = self.repack_group_cross(ostart, word_shift);
                self.data[write_idx(out_widx)] = sum;
                ostart -= total_merge;
            }
        }

        self.shift_indices(word_shift);
        self.current.width = output_width;
        total_widen + cross_steps
    }
}
