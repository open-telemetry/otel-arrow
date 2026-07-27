// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Downscale operations.

use super::HistogramNN;
use super::swar::{narrow, widen};
use super::width::{ALL_WIDTHS, Width};

/// Outcome of the in-word widening phase.
struct Widening {
    /// Lane width after widening.
    width: Width,
    /// Widening steps applied; each one consumes a scale step.
    steps: u32,
    /// OR-fold of every active lane at `width`. Its highest set bit
    /// matches that of the largest lane, so `Width::from_max_value`
    /// yields the narrowest width the data still fits in.
    lanes_or: u64,
}

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

    /// OR-fold of every active lane at `width`, without modifying data.
    fn or_fold_words(&self, width: Width) -> u64 {
        (self.word_start..=self.word_end)
            .map(|widx| width.or_fold_lanes(self.data[self.data_idx(widx)]))
            .fold(0u64, |acc, lanes| acc | lanes)
    }

    /// OR-fold of the words that grouping by `steps` would produce,
    /// without modifying data: each aligned run of `1 << steps`
    /// consecutive words summed into one.
    ///
    /// Only meaningful at `Width::U64`, where a lane is a whole word;
    /// `steps == 0` is then just [`Self::or_fold_words`].
    fn or_fold_groups(&self, steps: u32) -> u64 {
        let group = 1i32 << steps;
        let mut acc = 0u64;
        let mut gstart = self.word_start & !(group - 1);
        while gstart <= self.word_end {
            let mut sum = 0u64;
            for g in 0..group {
                let widx = gstart + g;
                if widx >= self.word_start && widx <= self.word_end {
                    sum += self.data[self.data_idx(widx)];
                }
            }
            acc |= sum;
            gstart += group;
        }
        acc
    }

    /// Narrowest width the output may take: what the widest lane needs
    /// to survive, floored by the caller's `min_output_width`.
    fn required_width(lanes_or: u64, min_output_width: Width) -> Width {
        Width::from_max_value(lanes_or).max(min_output_width)
    }

    /// Widens lanes in place until `range_steps` levels sit above the
    /// required width, or until widening is exhausted at `Width::U64`.
    ///
    /// Each widening step sums adjacent lanes, so it consumes one
    /// scale step; the levels the caller is known to need up front are
    /// applied as a single pass. Reaching U64 is always
    /// affordable: each step also halves the bucket count, so the
    /// range-coverage invariant carries all the way down to
    /// `MIN_SCALE`, where the range is two buckets held by `N >= 2`
    /// U64 words.
    ///
    /// The loop terminates: one step raises the lane width by one
    /// level while at most doubling the largest lane, so the gap
    /// between the current and the required width never shrinks and
    /// U64 is the worst case.
    fn widen_to_fit(&mut self, range_steps: u32, min_output_width: Width) -> Widening {
        let start = self.current.width;
        // The caller's floor plus the requested range is the least
        // widening that can possibly satisfy the loop below.
        let wanted = min_output_width.subtract(start) as u32 + range_steps;
        let mut steps = wanted.min(start.to_u64_widen_steps());
        let mut width = start.wider_by(steps).expect("capped at U64");
        let mut lanes_or = if steps == 0 {
            self.or_fold_words(width)
        } else {
            self.widen_words(start, width)
        };

        loop {
            // The levels between the current and the required width
            // are the range the narrow step will win back.
            let required = Self::required_width(lanes_or, min_output_width);
            if width.subtract(required) >= range_steps as i32 || width == Width::U64 {
                return Widening {
                    width,
                    steps,
                    lanes_or,
                };
            }
            let prev = width;
            width = prev.wider_by(1).expect("not yet U64");
            lanes_or = self.widen_words(prev, width);
            steps += 1;
        }
    }

    /// Relaxes the range the histogram covers by a factor of
    /// `2^range_steps`, leaving the output width at least
    /// `min_output_width`.
    ///
    /// The two requests are independent. Range is won by narrowing:
    /// each level a lane gives back doubles the buckets a word holds
    /// while the scale stays put. Width is won by widening, which
    /// halves the buckets and drops the scale by one, leaving the range
    /// untouched. A caller that only needs wider counters (a counter
    /// overflow) therefore passes `range_steps == 0`.
    ///
    /// Returns the scale steps applied: the widening and grouping the
    /// two requests together demanded, which exceeds `range_steps`
    /// whenever the output ends up wider than the input.
    pub(super) fn do_downscale(&mut self, range_steps: u32, min_output_width: Width) -> u32 {
        debug_assert!(!self.buckets_empty());

        let input_width = self.current.width;
        debug_assert!(
            min_output_width >= input_width,
            "callers only ever widen: {min_output_width:?} is narrower than {input_width:?}",
        );

        // We require N>=2 b/c MIN_SCALE results in 2 buckets at u64,
        // so as a precondition we must be able to change scale at
        // least as much as requested.
        let budget = self.current.scale.scale() - crate::mapping::MIN_SCALE;
        debug_assert!(
            input_width.to_u64_widen_steps() as i32 <= budget,
            "width {input_width:?} needs {} steps to reach U64, only {budget} left above MIN_SCALE",
            input_width.to_u64_widen_steps(),
        );
        debug_assert!(
            range_steps as i32 <= budget,
            "range relaxation of {range_steps} requested, only {budget} steps left above MIN_SCALE",
        );

        // Phase 1: Widen lanes in place, consuming one scale step per
        // step, until they have room to be narrowed back by
        // `range_steps`.
        let Widening {
            width: cur,
            steps: total_widen,
            lanes_or,
        } = self.widen_to_fit(range_steps, min_output_width);

        // Phase 2: Group consecutive words, standing in for the
        // widening that the U64 ceiling denied.
        //
        // A lane cannot exceed a word, so once Phase 1 hits U64 the
        // ladder continues by summing whole words: after `cross_steps`
        // groupings a logical lane spans `64 << cross_steps` bits --
        // U128, U256, and so on. Grouping is therefore considered only
        // at U64.
        //
        // The loop stops by `cross_steps == range_steps` at the latest:
        // `required` never exceeds U64, so the rule holds there.
        let mut cross_steps = 0u32;

        if cur == Width::U64 {
            // A lane is a whole word here, so `lanes_or` is already
            // the zero-step grouping.
            let mut grouped_or = lanes_or;
            loop {
                let effective = cur as i32 + cross_steps as i32;
                let required = Self::required_width(grouped_or, min_output_width);
                // Phase 1's rule on the extended ladder.
                if effective - required as i32 >= range_steps as i32 {
                    break;
                }
                cross_steps += 1;
                grouped_or = self.or_fold_groups(cross_steps);
            }
        }

        debug_assert!(
            (total_widen + cross_steps) as i32 <= budget,
            "consumed {} scale steps to relax the range by {range_steps}, only {budget} available",
            total_widen + cross_steps,
        );

        // Phase 3: Narrow and repack with two-pass clobber prevention.
        //
        // Narrowing is what wins the range back, `range_steps` levels
        // of it, less the groupings that already stood in for some.
        // No floor cap is needed: Phases 1 and 2 stop only once that
        // many levels sit above `required`, which already includes
        // min_output_width.
        //
        // Narrowing is also capped by range coverage: each step
        // doubles the buckets a word holds, and two words must still
        // fit within the range at the new scale. Widening and
        // cross-word grouping cannot violate that bound, so only this
        // step needs the cap:
        //
        //   new_scale >= output_width.min_scale()
        //             =  MIN_SCALE + (U64 - output_width)
        //
        // with output_width = cur - narrow_steps, which rearranges to
        //
        //   narrow_steps <= (new_scale - MIN_SCALE) - (U64 - cur).
        //
        // The last term matters when widening stopped short of U64.
        let new_scale = self.current.scale.scale() - (total_widen + cross_steps) as i32;
        let coverage_cap =
            (new_scale - crate::mapping::MIN_SCALE - cur.to_u64_widen_steps() as i32).max(0) as u32;
        let narrow_steps = (range_steps - cross_steps).min(coverage_cap);
        let word_shift = cross_steps + narrow_steps;
        let output_width = ALL_WIDTHS[cur as usize - narrow_steps as usize];
        debug_assert!(output_width >= min_output_width);

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
        self.debug_assert_range_coverage();
        total_widen + cross_steps
    }
}

#[cfg(test)]
mod tests {
    use super::super::HistogramNN;
    use crate::histogram::width::Width;
    use crate::mapping::{MIN_SCALE, Scale, ScaleError, min_scale_for, table_scale};

    /// Deterministic xorshift, so failures reproduce exactly.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    const ALL: [Width; 7] = [
        Width::B1,
        Width::B2,
        Width::B4,
        Width::U8,
        Width::U16,
        Width::U32,
        Width::U64,
    ];

    /// Scenario: a histogram is driven with many observations spread
    /// over a wide value range with large increments, forcing repeated
    /// widening, cross-word grouping, and narrowing.
    /// Guarantees: every downscale leaves the histogram covering no
    /// more buckets than the value range spans at its scale, so a
    /// later counter overflow can always be paid for; total count and
    /// word bounds stay consistent.
    fn exercise<const N: usize>(width: Width, scale: i32, spread: f64, incr_max: u64, seed: u64) {
        let mut hist: HistogramNN<N> = HistogramNN::new()
            .with_min_width(width)
            .expect("width fits the default scale")
            .with_max_scale(scale)
            .expect("scale covers this many buckets");
        let mut rng = Rng(seed | 1);
        let mut expect_count = 0u64;

        for _ in 0..500 {
            let r = rng.next();
            let e = ((r >> 3) % 2000) as f64 / 2000.0;
            let value = (e * spread).exp2();
            let incr = 1 + (r % incr_max);
            if hist.record_incr(value, incr).is_err() {
                break;
            }
            expect_count += incr;
            // The invariant every downscale must preserve.
            assert!(
                hist.current.scale.scale() >= hist.current.width.min_scale(),
                "scale {} below floor {} for width {:?}",
                hist.current.scale.scale(),
                hist.current.width.min_scale(),
                hist.current.width,
            );
            assert!(hist.word_end - hist.word_start < N as i32);
        }

        assert_eq!(hist.stats().count, expect_count);
    }

    /// Scenario: the number of buckets spanning the normal f64 range
    /// is computed at every representable scale and compared against
    /// the closed form `min_scale_for` uses.
    /// Guarantees: the count halves with each scale step down and
    /// bottoms out at exactly 2 buckets at `MIN_SCALE`, which is what
    /// lets an `N >= 2` histogram always downscale to the floor; and
    /// `min_scale_for` never understates the required scale by more
    /// than the one step its rounding permits.
    #[test]
    fn range_bucket_count_halves_down_to_two() {
        let count = |scale: i32| Scale::new(scale).expect("in range").range_bucket_count();
        assert_eq!(count(MIN_SCALE), 2);
        assert_eq!(count(-4), 128);
        for scale in MIN_SCALE..table_scale() {
            let (lo, hi) = (count(scale), count(scale + 1));
            assert!(
                hi == 2 * lo || hi == 2 * lo - 1 || hi == 2 * lo + 1,
                "range count {lo} at scale {scale} does not roughly double to {hi}",
            );
            // The closed form agrees with the measured range, to
            // within the documented one-step slack.
            let exact = (MIN_SCALE..=table_scale())
                .find(|&s| count(s) >= lo)
                .expect("scale covers its own range");
            let closed = min_scale_for(lo);
            assert!(
                closed == exact || closed == exact - 1,
                "min_scale_for({lo}) = {closed}, measured floor is {exact}",
            );
        }

        // Exact where it is load bearing: the floor for the two words
        // every histogram has.
        for width in ALL {
            assert_eq!(
                width.min_scale(),
                min_scale_for(2 * width.slots_per_u64() as u64),
            );
        }
    }

    /// Scenario: every counter width is started at its lowest legal
    /// scale and at higher scales, with pool sizes from 2 to 64 and
    /// increments from 1 to near-u64::MAX.
    /// Guarantees: downscaling terminates and preserves range
    /// coverage everywhere, including at the lowest legal scale where
    /// a misjudged scale budget previously caused an infinite retry
    /// loop in `record_incr`.
    #[test]
    fn downscale_preserves_range_coverage_at_every_width_and_scale() {
        for width in ALL {
            for delta in [0, 1, 3] {
                for (spread, incr_max) in [(3.0, 1u64), (60.0, 7), (900.0, 1 << 50)] {
                    let at = |floor: i32| (floor + delta).min(table_scale());
                    exercise::<2>(
                        width,
                        at(HistogramNN::<2>::min_scale(width)),
                        spread,
                        incr_max,
                        12345,
                    );
                    exercise::<8>(
                        width,
                        at(HistogramNN::<8>::min_scale(width)),
                        spread,
                        incr_max,
                        777,
                    );
                    exercise::<64>(
                        width,
                        at(HistogramNN::<64>::min_scale(width)),
                        spread,
                        incr_max,
                        99,
                    );
                }
            }
            for scale in [0, table_scale()] {
                exercise::<2>(width, scale, 900.0, 1 << 50, 5);
                exercise::<64>(
                    width,
                    scale.max(HistogramNN::<64>::min_scale(width)),
                    900.0,
                    1 << 50,
                    6,
                );
            }
        }
    }

    /// Scenario: a scale is requested at which the histogram would
    /// define more buckets than the value range covers, in both
    /// builder orders, plus the boundary cases that are legal.
    /// Guarantees: infeasible (scale, width) pairs are rejected at
    /// construction rather than deadlocking later, and the exact floor
    /// -- the scale whose range is at least `N * slots_per_u64` wide
    /// -- is accepted.
    #[test]
    fn builder_rejects_scale_that_does_not_cover_the_buckets() {
        // Two words of B1 counters are 128 buckets, exactly the range
        // covered at scale -4; at U64 they are 2 buckets, the range at
        // MIN_SCALE.
        assert_eq!(HistogramNN::<2>::min_scale(Width::B1), -4);
        assert_eq!(HistogramNN::<2>::min_scale(Width::U64), MIN_SCALE);
        // More words need a finer scale to stay inside the range.
        assert_eq!(HistogramNN::<64>::bucket_capacity(Width::B1), 4096);
        assert_eq!(HistogramNN::<64>::min_scale(Width::B1), 1);

        for width in ALL {
            let floor = HistogramNN::<2>::min_scale(width);
            let hist: HistogramNN<2> = HistogramNN::new()
                .with_min_width(width)
                .expect("default scale covers any width");
            assert!(hist.with_max_scale(floor).is_ok());

            let hist: HistogramNN<2> = HistogramNN::new()
                .with_min_width(width)
                .expect("default scale covers any width");
            if floor > MIN_SCALE {
                assert_eq!(
                    hist.with_max_scale(floor - 1).err(),
                    Some(ScaleError::RangeCoverage)
                );
            }
        }

        // The other builder order is rejected symmetrically.
        let hist: HistogramNN<2> = HistogramNN::new()
            .with_min_width(Width::U64)
            .expect("default scale covers any width")
            .with_max_scale(MIN_SCALE)
            .expect("two U64 counters cover the range at MIN_SCALE");
        assert_eq!(
            hist.with_min_width(Width::U32).err(),
            Some(ScaleError::RangeCoverage)
        );

        // Out-of-range scales still report InvalidScale.
        let hist: HistogramNN<2> = HistogramNN::new();
        assert_eq!(
            hist.with_max_scale(MIN_SCALE - 1).err(),
            Some(ScaleError::InvalidScale)
        );
    }
}
