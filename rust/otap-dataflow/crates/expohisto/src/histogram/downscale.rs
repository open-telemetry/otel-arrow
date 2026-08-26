// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Downscale operations.

use super::HistogramNN;
use super::swar::{narrow, widen};
use super::width::Width;

/// Outcome of the in-word widening phase.
struct Widening {
    /// Lane width after widening.
    width: Width,
    /// Widening steps applied; each one consumes a scale step.
    steps: u32,
    /// OR-fold of every active lane at `width`; its highest set bit
    /// matches the largest lane's, so `Width::from_max_value` yields
    /// the narrowest width the data fits in.
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

    /// Physical data index for `widx` under the mapping that holds
    /// once `shift_indices(steps)` has run.
    #[inline]
    fn shifted_data_idx(&self, widx: i32, steps: u32) -> usize {
        (widx - (self.word_base >> steps)).rem_euclid(N as i32) as usize
    }

    /// Packs one aligned group of `1 << narrow_steps` input words
    /// into a single output word.  Reads use the current physical
    /// mapping; words outside the active window read as zero.
    fn repack_group(&self, ostart: i32, narrow_steps: u32, cur: Width, output: Width) -> u64 {
        let group = 1i32 << narrow_steps;
        let chunk_bits = 64u32 >> narrow_steps;

        let mut acc = 0u64;
        for r in 0..group {
            let widx = ostart + r;
            let value = if widx >= self.word_start && widx <= self.word_end {
                self.data[self.data_idx(widx)]
            } else {
                0
            };
            acc |= narrow(cur, output, value) << (r as u32 * chunk_bits);
        }
        acc
    }

    /// OR-fold of every active lane at `width`, without modifying data.
    fn or_fold_words(&self, width: Width) -> u64 {
        (self.word_start..=self.word_end)
            .map(|widx| width.or_fold_lanes(self.data[self.data_idx(widx)]))
            .fold(0u64, |acc, lanes| acc | lanes)
    }

    /// Sums each aligned run of `1 << steps` words visited by `run`
    /// into one word, writing each group's sum under the mapping
    /// shifted by `steps`, and returns the OR-fold of those sums.
    ///
    /// `run` must visit consecutive word indices -- ascending or
    /// descending -- so that a group's words arrive together.
    fn fold_run(&mut self, run: impl Iterator<Item = i32>, steps: u32) -> u64 {
        let mut acc = 0u64;
        let mut out: Option<i32> = None;
        let mut sum = 0u64;

        for widx in run {
            let o = widx >> steps;
            if out != Some(o) {
                if let Some(prev) = out {
                    acc |= sum;
                    let di = self.shifted_data_idx(prev, steps);
                    self.data[di] = sum;
                }
                out = Some(o);
                sum = 0;
            }
            sum += self.data[self.data_idx(widx)];
        }

        if let Some(prev) = out {
            acc |= sum;
            let di = self.shifted_data_idx(prev, steps);
            self.data[di] = sum;
        }
        acc
    }

    /// Sums each aligned run of `1 << steps` words into one word, in
    /// place, reindexes the window, and returns the OR-fold of the
    /// words produced.
    ///
    /// Only meaningful at `Width::U64`, where a lane is a whole word
    /// and the fold extends the width ladder past the word ceiling to
    /// a logical lane of `64 << steps` bits.
    ///
    /// Like the repack in Phase 3, it walks outward from `word_base`
    /// in two passes so no output clobbers an unread input: an output
    /// lands at or before the first input of its own group, measured
    /// as distance from `word_base`, and that distance only shrinks.
    fn fold_groups_in_place(&mut self, steps: u32) -> u64 {
        debug_assert!(steps > 0);

        let old_start = self.word_start;
        let old_end = self.word_end;
        // The group holding `word_base` may reach below it; its
        // output belongs to the forward pass.
        let fwd_start = old_start.max((self.word_base >> steps) << steps);

        let mut acc = self.fold_run(fwd_start..=old_end, steps);
        if fwd_start > old_start {
            acc |= self.fold_run((old_start..fwd_start).rev(), steps);
        }

        self.shift_indices(steps);
        acc
    }

    /// Narrowest width the output may take: what the widest lane
    /// needs, floored at the input width, since downscaling only adds
    /// counts together.
    fn required_width(lanes_or: u64, input_width: Width) -> Width {
        Width::from_max_value(lanes_or).max(input_width)
    }

    /// Widens lanes in place until `range_steps` levels sit above the
    /// required width, or until widening is exhausted at `Width::U64`.
    ///
    /// Each step sums adjacent lanes, so it costs one scale step; the
    /// levels known to be needed up front are applied in a single
    /// pass. Reaching U64 is always affordable, since each step also
    /// halves the bucket count, carrying range coverage down to
    /// `MIN_SCALE`, where the range is two buckets held by `N >= 2`
    /// U64 words.
    ///
    /// The loop terminates: a step raises the width one level while
    /// at most doubling the largest lane, so the gap to the required
    /// width never shrinks and U64 is the worst case.
    fn widen_to_fit(&mut self, range_steps: u32) -> Widening {
        let start = self.current.width;
        let mut steps = range_steps.min(start.to_u64_widen_steps());
        let mut width = start.wider_by(steps).expect("capped at U64");
        let mut lanes_or = if steps == 0 {
            self.or_fold_words(width)
        } else {
            self.widen_words(start, width)
        };

        loop {
            // The levels between the current and the required width
            // are the range the narrow step will win back.
            let required = Self::required_width(lanes_or, start);
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

    /// Continues the width ladder past the U64 ceiling by summing
    /// whole words: after `cross_steps` folds a logical lane spans
    /// `64 << cross_steps` bits -- U128, U256, and so on. Folds until
    /// `range_steps` levels of that extended ladder sit above the
    /// required width. Returns the folds applied and the OR-fold of
    /// the lanes they produced.
    ///
    /// Each round jumps by the whole deficit rather than creeping,
    /// and the required width only grows, so the deficit shrinks
    /// every round; each fold also divides the active span by `2^s`,
    /// so the phase costs little more than its first round. The total
    /// stays within `range_steps` because the required width never
    /// exceeds U64.
    ///
    /// `lanes_or` enters as the zero-step fold of the U64 lanes, so
    /// the caller must have widened the lanes to `Width::U64`.
    fn fold_to_fit(&mut self, range_steps: u32, input_width: Width, lanes_or: u64) -> (u32, u64) {
        let mut cross_steps = 0u32;
        let mut lanes_or = lanes_or;

        loop {
            let effective = Width::U64 as i32 + cross_steps as i32;
            let required = Self::required_width(lanes_or, input_width) as i32;
            // Phase 1's rule on the extended ladder.
            let deficit = range_steps as i32 - (effective - required);
            if deficit <= 0 {
                return (cross_steps, lanes_or);
            }
            debug_assert!(cross_steps + deficit as u32 <= range_steps);
            lanes_or = self.fold_groups_in_place(deficit as u32);
            cross_steps += deficit as u32;
        }
    }

    /// Relaxes the range the histogram covers by a factor of
    /// `2^range_steps`.
    ///
    /// Range is won by narrowing: each level a lane gives back
    /// doubles the buckets a word holds while the scale stays put.
    /// Widening is only the means -- it halves the buckets and drops
    /// the scale by one, leaving the range untouched -- so a caller
    /// that wants wider counters rather than more range wants
    /// [`HistogramNN::widen_to`] instead.
    ///
    /// The output is never narrower than the input, since summing
    /// buckets only grows counts.
    ///
    /// Returns the scale steps applied, which exceeds `range_steps`
    /// whenever the sums force the output wider than the input.
    pub(super) fn do_downscale(&mut self, range_steps: u32) -> u32 {
        debug_assert!(range_steps != 0);
        debug_assert!(!self.buckets_empty());

        let input_width = self.current.width;
        let entry_start = self.word_start;
        let entry_end = self.word_end;

        // Slack is the range relaxation the histogram can still
        // absorb:
        //
        //   slack = scale - width.min_scale()
        //         = (scale - MIN_SCALE) - (U64 - width)
        //
        // Range coverage is exactly `slack >= 0`, and this call
        // spends `range_steps` of it: widening is slack-neutral (one
        // scale step down, one width level up), while each fold and
        // each narrowing step costs one, and those come to
        // `range_steps`. Callers stay within it: an out-of-range
        // index still lies inside the value range, which covers
        // `2^(slack + 1)` times the words the window holds, and
        // `HighLow::change_steps` halves only until the span fits.
        let slack = self.current.scale.scale() - input_width.min_scale();
        debug_assert!(
            range_steps as i32 <= slack,
            "range relaxation of {range_steps} requested, only {slack} available at {input_width:?} and scale {}",
            self.current.scale.scale(),
        );

        // Phase 1: widen lanes in place until they have room to be
        // narrowed back by `range_steps`.
        let Widening {
            width: cur,
            steps: widen_steps,
            lanes_or,
        } = self.widen_to_fit(range_steps);

        // Phase 2: past the U64 ceiling a lane cannot grow, so the
        // ladder continues by summing whole words; `lanes_or` follows
        // the lanes up that extended ladder.
        let (cross_steps, lanes_or) = if cur == Width::U64 {
            self.fold_to_fit(range_steps, input_width, lanes_or)
        } else {
            (0, lanes_or)
        };

        // Phase 3: narrow to the width the data still needs;
        // everything above `required` is headroom the sums did not
        // use. No cap is needed in either direction. Not too far:
        // `required` is floored at the input width. Not too little:
        // Phases 1 and 2 stop as soon as `range_steps` levels sit
        // above `required` and cannot overshoot, since each jumps by
        // exactly the deficit it owes and `required` only grows. So
        // the headroom here is the range still owed.
        let required = Self::required_width(lanes_or, input_width);
        let narrow_steps = cur.subtract(required) as u32;
        debug_assert_eq!(
            narrow_steps,
            range_steps - cross_steps,
            "narrowing to {required:?} from {cur:?} does not settle the {range_steps} steps of \
             range owed, {cross_steps} of which were folded",
        );

        // Reindexing follows `word = bucket >> (U64 - width)`.
        // Widening halves the bucket index and lowers the shift by
        // one, so the window does not move -- Phase 1 never
        // reindexes. Folding runs at U64, where a word is a bucket,
        // and moves it by `cross_steps` (done inside
        // `fold_groups_in_place`). Narrowing raises the shift and
        // moves it by `narrow_steps`, below. The repack walks outward
        // from `word_base` in two passes so no output clobbers an
        // unread input.
        if narrow_steps > 0 {
            let group = 1i32 << narrow_steps;
            let stride = group as usize;
            let word_end = self.word_end;

            // Group starts: the group holding `word_base`, then
            // outward in both directions.
            let fwd_start = (self.word_base >> narrow_steps) << narrow_steps;
            let rev_start = fwd_start - group;
            let rev_end = (self.word_start >> narrow_steps) << narrow_steps;

            let forward = (fwd_start..=word_end).step_by(stride);
            let reverse = (rev_end..=rev_start).rev().step_by(stride);

            for ostart in forward.chain(reverse) {
                let acc = self.repack_group(ostart, narrow_steps, cur, required);
                let di = self.shifted_data_idx(ostart >> narrow_steps, narrow_steps);
                self.data[di] = acc;
            }

            self.shift_indices(narrow_steps);
        }

        self.current.width = required;

        // The window shrinks by exactly the factor asked for, which
        // is what makes the out-of-range index fit.
        debug_assert_eq!(
            (self.word_start, self.word_end),
            (entry_start >> range_steps, entry_end >> range_steps),
            "window [{entry_start}, {entry_end}] did not shrink by the {range_steps} steps asked \
             for ({cross_steps} folded, {narrow_steps} narrowed)",
        );
        self.debug_assert_range_coverage();
        widen_steps + cross_steps
    }
}

#[cfg(test)]
mod tests {
    use super::super::HistogramNN;
    use crate::histogram::width::Width;
    use crate::mapping::{MIN_SCALE, Scale, ScaleError, min_scale_for, table_scale};
    use std::num::NonZeroU64;

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
            let incr = NonZeroU64::new(1 + (r % incr_max)).unwrap();
            if hist.record_incr(value, incr).is_err() {
                break;
            }
            expect_count += incr.get();
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

    /// Scenario: 63 words of saturated `U8` counters are downscaled by
    /// 8 steps, so Phase 1 exhausts widening at U64 and Phase 2 folds
    /// far enough that the sums outgrow the width its first round
    /// assumed.
    /// Guarantees: Phase 2 re-evaluates and spends a second round
    /// rather than under-counting the ladder -- the fold consumes
    /// 7 steps, not 6 -- and every count survives the fold and the
    /// narrowing that follows.
    #[test]
    fn folding_takes_a_second_round_when_the_sums_outgrow_the_width() {
        let mut hist: HistogramNN<64> = HistogramNN::new()
            .with_min_width(Width::U8)
            .expect("width fits the default scale")
            .with_max_scale(table_scale())
            .expect("scale covers this many buckets");

        // Every lane at its maximum: 63 words x 8 lanes x 255.
        const WORDS: i32 = 63;
        hist.word_base = 0;
        hist.word_start = 0;
        hist.word_end = WORDS - 1;
        for w in 0..WORDS {
            hist.data[hist.data_idx(w)] = u64::MAX;
        }
        let total = WORDS as u64 * 8 * 255;

        // Widening U8 -> U64 costs 3, leaving lanes at 2040 (U16), so
        // the first fold round asks for 8 - (U64 - U16) = 6 steps.
        // Summing 64 words then reaches 128520, past U16, and the
        // second round buys the one step that shortfall costs.
        let steps = hist.do_downscale(8);
        assert_eq!(steps, 3 + 7);
        assert_eq!(hist.current.width, Width::U32);

        let view = hist.view();
        let sum: u64 = view.positive().iter().sum();
        assert_eq!(sum, total);
    }

    /// Scenario: observations are recorded until the histogram has
    /// widened, folded across words, and narrowed, then every bucket
    /// is compared against a model that maps each observation
    /// directly at the final scale.
    /// Guarantees: a downscale moves each count into exactly the
    /// bucket its value maps to at the new scale -- counts are never
    /// lost, duplicated, or shifted by a word, which the count-only
    /// checks in `exercise` cannot see.
    fn check_buckets<const N: usize>(
        width: Width,
        scale: i32,
        spread: f64,
        incr_max: u64,
        seed: u64,
    ) {
        const OBS: usize = 500;
        const MODEL: usize = 4096;

        let mut hist: HistogramNN<N> = HistogramNN::new()
            .with_min_width(width)
            .expect("width fits the default scale")
            .with_max_scale(scale)
            .expect("scale covers this many buckets");
        let mut rng = Rng(seed | 1);
        let mut obs = [(0.0f64, 0u64); OBS];
        let mut nobs = 0usize;

        for _ in 0..OBS {
            let r = rng.next();
            let e = ((r >> 3) % 2000) as f64 / 2000.0;
            let value = (e * spread).exp2();
            let incr = NonZeroU64::new(1 + (r % incr_max)).unwrap();
            if hist.record_incr(value, incr).is_err() {
                break;
            }
            obs[nobs] = (value, incr.get());
            nobs += 1;
        }
        assert!(nobs > 0);

        let final_scale = Scale::new(hist.current.scale.scale()).expect("scale is representable");
        let view = hist.view();
        let buckets = view.positive();
        let offset = buckets.offset();
        let len = buckets.len() as usize;
        assert!(len <= MODEL);

        let mut model = [0u64; MODEL];
        for &(value, incr) in &obs[..nobs] {
            let rel = final_scale.map_to_index(value) - offset;
            assert!(
                rel >= 0 && (rel as usize) < len,
                "value {value} maps to slot {} outside [{offset}, {})",
                rel + offset,
                offset + len as i32,
            );
            model[rel as usize] += incr;
        }

        for (slot, got) in buckets.iter().enumerate() {
            assert_eq!(
                got,
                model[slot],
                "slot {} (index {}) holds {got}, model says {}",
                slot,
                offset + slot as i32,
                model[slot],
            );
        }
    }

    /// Scenario: the same recording pattern as the coverage test is
    /// replayed at every width, several scales and pool sizes, and
    /// each resulting histogram is checked bucket by bucket.
    /// Guarantees: widening, cross-word folding, and narrowing
    /// together preserve the mapping from value to bucket, so a
    /// downscaled histogram is the histogram that would have been
    /// built at the final scale.
    #[test]
    fn downscale_places_every_count_in_the_bucket_its_value_maps_to() {
        for width in ALL {
            for delta in [0, 1, 3] {
                for (spread, incr_max) in [(3.0, 1u64), (60.0, 7), (60.0, 1 << 30), (60.0, 1 << 50)]
                {
                    let at = |floor: i32| (floor + delta).min(table_scale());
                    check_buckets::<2>(
                        width,
                        at(HistogramNN::<2>::min_scale(width)),
                        spread,
                        incr_max,
                        12345,
                    );
                    check_buckets::<8>(
                        width,
                        at(HistogramNN::<8>::min_scale(width)),
                        spread,
                        incr_max,
                        777,
                    );
                    check_buckets::<64>(
                        width,
                        at(HistogramNN::<64>::min_scale(width)),
                        spread,
                        incr_max,
                        99,
                    );
                }
            }
        }
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
                for (spread, incr_max) in
                    [(3.0, 1u64), (60.0, 7), (60.0, 1 << 30), (900.0, 1 << 50)]
                {
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

    /// Scenario: the word-count floor `HistogramNN::MIN_WORDS` is
    /// derived by inverting `min_scale_for` rather than written down.
    /// Guarantees: the floor is exactly the buckets the value range
    /// spans at `MIN_SCALE`, so a histogram at the floor can still be
    /// downscaled to `MIN_SCALE` at `Width::U64` (one bucket per
    /// word), while one word short of it could not.
    #[test]
    fn word_count_floor_matches_the_range_at_min_scale() {
        const FLOOR: usize = HistogramNN::<2>::MIN_WORDS;

        assert_eq!(FLOOR, 2);
        assert_eq!(min_scale_for(FLOOR as u64), MIN_SCALE);
        assert!(min_scale_for(FLOOR as u64 + 1) > MIN_SCALE);
        assert_eq!(HistogramNN::<2>::min_scale(Width::U64), MIN_SCALE);
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
