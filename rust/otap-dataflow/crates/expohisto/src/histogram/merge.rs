// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Merge logic for combining histograms.
//!
//! Both sides store their counters packed several-to-a-`u64` (see
//! [`swar`](super::swar)), so a merge is not a bucket-by-bucket loop: it
//! rewrites whole words at a time. Two things can differ between the sides,
//! and both are resolved before any counter is added:
//!
//! 1. **Prepare** -- Set the width to `max(W_self, W_src)` and downscale self
//!    until the combined slot range fits in `N` words. After this the
//!    destination's geometry is fixed, and the source's scale is at least as
//!    fine as the destination's.
//! 2. **Merge** -- Walk the destination word by word. For each one, gather
//!    the source counts that land in it and repack them into the
//!    destination's lane layout, then add the packed word with
//!    [`swar_add_checked`].
//!
//! Repacking goes one of two ways, depending on whether the destination's
//! lanes are the wider or the narrower ones, and each direction is a mirror
//! of the other:
//!
//! - **many source words into one** -- widen lanes, sum across words, then
//!   [`narrow`] and pack several sub-groups into the destination word;
//! - **one source word across many** -- widen lanes, then [`spread`] a
//!   contiguous slice of them into the destination's wider lanes.
//!
//! # Overflow during the merge
//!
//! Either step of the add can find a counter too large for the destination's
//! lanes: the packed source counts alone may not fit, or they may fit but
//! overflow when added to what is already there. Both are handled the same
//! way -- widen the destination and repack from the source -- because
//! widening re-lays out every lane and so invalidates an already-packed word.
//!
//! Packing itself can only overflow in the many-into-one direction, where
//! several source counts are summed and then narrowed. In the other direction
//! the destination's lanes are strictly wider than the source's, so every
//! count fits by construction.
//!
//! That retry terminates, and does not disturb which source words feed which
//! destination word, because the grouping factor
//!
//! ```text
//! word_ratio_log2 = (src_scale - dest_scale) + src_width - dest_width
//! ```
//!
//! is invariant under widening the destination. Widening by `k` levels raises
//! `dest_width` by `k`, and [`widen_to`](HistogramNN::widen_to) lowers
//! `dest_scale` by the same `k` to preserve range coverage, so the two changes
//! cancel. Each retry therefore repacks exactly the same source words into
//! exactly the same destination word, one width wider, and the width ladder
//! is finite.

use super::swar::{narrow, spread, swar_add_checked, widen};
use super::width::Width;
use super::{Error, HighLow, HistogramNN};

/// Result of packing source counts for one destination word, in the
/// many-source-words-into-one direction.
///
/// This is deliberately not a `Result`: neither case is an error, and the
/// caller acts on both. `TooWide` is a request to grow the destination and
/// ask again.
enum Packed {
    /// Counts laid out in the destination's current lane width, ready to be
    /// added to a destination word.
    Word(u64),
    /// A source count did not fit a destination lane. Carries an OR-fold of
    /// the offending lanes, whose highest set bit gives the width the
    /// destination must reach before the pack can be retried.
    TooWide(u64),
}

/// How the source's lanes line up with the destination once the difference in
/// scale is accounted for.
///
/// Merging into a coarser destination means summing groups of `2^shift`
/// source buckets. That sum is split into two stages, because a lane can only
/// absorb so much before it runs out of bits:
///
/// - `in_word` steps are done inside each source word, by widening its lanes
///   and letting the SWAR pair-sums fold neighbours together;
/// - `cross` steps are what remains once the lanes have hit the `u64`
///   ceiling, and are done by summing whole source words.
#[derive(Debug, Clone, Copy)]
struct SourceLanes {
    /// Lane width the source is widened to before any cross-word summing.
    cur: Width,
    /// Widening steps applied within each source word (`src_width -> cur`).
    in_word: u32,
    /// Scale steps left over for cross-word summing.
    cross: u32,
}

impl SourceLanes {
    /// Plans the two-stage fold for a source at `src_scale` feeding a
    /// destination at `dest_scale`.
    fn plan(src_width: Width, src_scale: i32, dest_scale: i32) -> Self {
        // Only a finer source needs folding. The preparation step guarantees
        // the source is never coarser, but the clamp keeps this total.
        let shift = (src_scale - dest_scale).max(0) as u32;
        let in_word = shift.min(src_width.to_u64_widen_steps());
        Self {
            cur: src_width.wider_by(in_word).expect("capped at U64"),
            in_word,
            cross: shift - in_word,
        }
    }

    /// Widens one raw source word to [`cur`](Self::cur), folding neighbouring
    /// buckets together as it goes.
    fn widen_word(&self, src_width: Width, raw: u64) -> u64 {
        if self.in_word > 0 {
            widen(src_width, self.cur, raw)
        } else {
            raw
        }
    }
}

impl<const N: usize> HistogramNN<N> {
    /// Merges another histogram into this one.
    ///
    /// The source histogram may have a different pool size (`M`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] if the combined total count would
    /// exceed `u64::MAX`.
    pub fn merge_from<const M: usize>(&mut self, other: &HistogramNN<M>) -> Result<(), Error> {
        if other.stats.count == 0 {
            return Ok(());
        }

        let new_count = self
            .checked_add_count(other.stats.count)
            .ok_or(Error::Overflow)?;

        self.merge_buckets(other);

        self.commit_merge(&other.stats, new_count);
        Ok(())
    }

    /// Core merge: prepare self (width + scale), then word-by-word
    /// merge with on-the-fly repacking.
    ///
    /// Infallible: count overflow is checked by the caller, and all
    /// internal operations (downscale, widen) always succeed.
    fn merge_buckets<const M: usize>(&mut self, other: &HistogramNN<M>) {
        // A source that recorded nothing but exact zeros has a non-zero count
        // and no buckets. There is nothing to merge here; its count is folded
        // in by the caller, where it becomes part of the merged zero count.
        if other.buckets_empty() {
            return;
        }

        let src_scale = other.current.scale.scale();
        let src_width = other.current.width;
        let merge_width = self.current.width.max(src_width);
        let min_scale = self.current.scale.scale().min(src_scale);

        // Combined slot range at min_scale, using merge_width for
        // word capacity.
        let self_hl = self.slot_range_at_scale(min_scale);
        let other_hl = other.slot_range_at_scale(min_scale);
        let combined = self_hl.merge(other_hl);

        let word_hl = HighLow {
            low: merge_width.slot_to_word_index(combined.low),
            high: merge_width.slot_to_word_index(combined.high),
        };
        let extra = word_hl.change_steps(N);
        let target_scale = min_scale - extra as i32;

        // Two independent requests: relax the range down to
        // `target_scale`, and hold counters at least as wide as the
        // source needs.
        let range_change = (self.current.scale.scale() - target_scale).max(0) as u32;

        // The range fix is affordable under the range-coverage
        // invariant: the range shrinks to two buckets at MIN_SCALE, so
        // no range fix can demand more steps than remain.
        debug_assert!(
            range_change as i32 <= self.current.scale.scale() - crate::mapping::MIN_SCALE,
            "merge needs {range_change} scale steps, only {} available",
            self.current.scale.scale() - crate::mapping::MIN_SCALE,
        );

        // Width first: the merge repacks source lanes into ours, so
        // ours must be wide enough to hold them. Then the range fix,
        // which narrows back no further than the width just set.
        self.widen_to(merge_width);
        if self.buckets_empty() {
            // No data to transform -- the scale moves on its own.
            self.change_scale(range_change);
        } else {
            self.downscale_by(range_change);
        }

        // Word-by-word merge with on-the-fly repacking.
        self.merge_words(other, src_width, src_scale);
    }

    /// Pre-extend the word range to cover `[lo_widx, hi_widx]` and
    /// zero-fill any newly exposed words.
    fn extend_word_range(&mut self, lo_widx: i32, hi_widx: i32) {
        if self.buckets_empty() {
            self.word_start = lo_widx;
            self.word_end = hi_widx;
            self.word_base = lo_widx;
            return;
        }
        if lo_widx < self.word_start {
            for w in lo_widx..self.word_start {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_start = lo_widx;
        }
        if hi_widx > self.word_end {
            for w in (self.word_end + 1)..=hi_widx {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_end = hi_widx;
        }
    }

    /// Word-by-word merge with on-the-fly repacking.
    ///
    /// Dispatches on the grouping factor
    ///
    /// ```text
    /// word_ratio_log2 = (src_scale - dest_scale) + src_width - dest_width
    /// ```
    ///
    /// the log2 of how many source words feed one destination word. It is
    /// invariant under widening the destination, so the two directions below
    /// never trade places partway through a retry (see the module docs).
    ///
    /// A non-negative ratio is the usual case: the destination is coarser
    /// and/or narrower, so several source words collapse into one. A negative
    /// ratio means the reverse -- one source word spreads across
    /// `2^-word_ratio_log2` destination words -- which arises when the
    /// destination's lanes are much wider than the source's.
    fn merge_words<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
    ) {
        let shift = src_scale - self.current.scale.scale();
        let word_ratio_log2 = shift + src_width as i32 - self.current.width as i32;

        if word_ratio_log2 >= 0 {
            self.merge_src_words_into_one(other, src_width, src_scale, word_ratio_log2 as u32);
        } else {
            self.spread_src_word_across_dests(other, src_width, src_scale, -word_ratio_log2 as u32);
        }
    }

    /// Adds an already-packed word into destination word `dest_widx`.
    ///
    /// Returns `false` when the add would overflow a lane. In that case this
    /// histogram has been widened to fit the sum and the caller must repack
    /// from the source before retrying: `packed` was laid out for the old,
    /// narrower lanes and is stale.
    fn add_into_word(&mut self, dest_widx: i32, packed: u64) -> bool {
        let width = self.current.width;
        let didx = self.data_idx(dest_widx);
        if let Some(result) = swar_add_checked(self.data[didx], packed, width) {
            self.data[didx] = result;
            return true;
        }

        // Size the new width from the largest sum any lane could produce: the
        // widest lane on each side, added together.
        let max_dest = width.or_fold_lanes(self.data[didx]);
        let max_packed = width.or_fold_lanes(packed);
        self.widen_to(Width::from_max_value(max_dest + max_packed));
        false
    }

    /// Merges when several source words feed one destination word.
    ///
    /// Each destination word `d` draws from source words
    /// `[d << ratio_log2, (d + 1) << ratio_log2)`.
    fn merge_src_words_into_one<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        ratio_log2: u32,
    ) {
        debug_assert!(
            ratio_log2 < 31,
            "ratio_log2={ratio_log2}: scale bounds violated",
        );

        // An arithmetic shift right floors toward negative infinity, so this
        // already names the group containing the source's first word; no
        // separate rounding to a group boundary is needed.
        let dest_lo = other.word_start >> ratio_log2;
        let dest_hi = other.word_end >> ratio_log2;

        self.extend_word_range(dest_lo, dest_hi);

        for dest_widx in dest_lo..=dest_hi {
            let src_start = dest_widx << ratio_log2;

            loop {
                let packed = match Self::repack_source(
                    other,
                    src_width,
                    src_scale,
                    self.current.scale.scale(),
                    self.current.width,
                    src_start,
                ) {
                    Packed::TooWide(lanes) => {
                        self.widen_to(Width::from_max_value(lanes));
                        continue;
                    }
                    // Nothing to add: this group of source words is empty.
                    Packed::Word(0) => break,
                    Packed::Word(packed) => packed,
                };

                if self.add_into_word(dest_widx, packed) {
                    break;
                }
            }
        }
    }

    /// Merges when one source word spreads across several destination words.
    ///
    /// Widening the source by [`SourceLanes::in_word`] steps makes each of
    /// its lanes equal exactly one destination slot. Those lanes are narrower
    /// than the destination's, so one source word's worth of them fills
    /// `2^ratio_log2` destination words; this walks the source and hands each
    /// group of lanes to the destination word that owns it.
    fn spread_src_word_across_dests<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        ratio_log2: u32,
    ) {
        let dests_per_src = 1i32 << ratio_log2;

        for src_widx in other.word_start..=other.word_end {
            let raw = other.data[other.data_idx(src_widx)];
            if raw == 0 {
                continue;
            }

            for chunk in 0..dests_per_src {
                let dest_widx = src_widx * dests_per_src + chunk;

                // Only reason to repeat: adding overflowed a destination lane
                // and `add_into_word` widened the destination, which changes
                // the layout the chunk has to be packed into.
                loop {
                    let packed = Self::extract_source_chunk(
                        src_width,
                        src_scale,
                        self.current.scale.scale(),
                        self.current.width,
                        raw,
                        chunk,
                    );
                    if packed == 0 {
                        break;
                    }

                    // Extend only once a chunk is known to carry counts.
                    // Claiming a word for an empty chunk would advance the
                    // circular buffer's window and evict live data.
                    self.extend_word_range(dest_widx, dest_widx);

                    if self.add_into_word(dest_widx, packed) {
                        break;
                    }
                }
            }
        }
    }

    /// Extracts the lanes of source word `raw` that belong to destination word
    /// `chunk_index`, packed into the destination's lane layout.
    ///
    /// Used only when one source word spreads across several destination
    /// words. Each widened source lane already equals one destination slot,
    /// so this selects the right run of lanes and spreads them into the
    /// destination's wider ones -- the mirror image of the narrow-and-pack
    /// step in [`repack_source`](Self::repack_source).
    ///
    /// The destination's lanes are strictly the wider ones here, so every
    /// source count is guaranteed to fit and there is no `TooWide` case.
    fn extract_source_chunk(
        src_width: Width,
        src_scale: i32,
        dest_scale: i32,
        dest_width: Width,
        raw: u64,
        chunk_index: i32,
    ) -> u64 {
        let lanes = SourceLanes::plan(src_width, src_scale, dest_scale);
        let widened = lanes.widen_word(src_width, raw);

        let spread_steps = dest_width as u32 - lanes.cur as u32;
        if spread_steps == 0 {
            return widened;
        }

        // Each destination word is fed by an equal, contiguous slice of the
        // source word's bits; the slices tile it exactly.
        let chunk_bits = 64u32 >> spread_steps;
        spread(
            lanes.cur,
            dest_width,
            widened >> (chunk_index as u32 * chunk_bits),
        )
    }

    /// Repacks the source words feeding one destination word into a single
    /// word in the destination's lane layout.
    ///
    /// The group starts at `src_start` and runs for as many words as the
    /// destination's geometry demands. Its counts are summed at the
    /// [`SourceLanes::cur`] width and then narrowed to the destination's,
    /// several sub-groups to a word when the destination's lanes are the
    /// narrower ones -- the mirror image of the spread step in
    /// [`extract_source_chunk`](Self::extract_source_chunk).
    fn repack_source<const M: usize>(
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        dest_scale: i32,
        dest_width: Width,
        src_start: i32,
    ) -> Packed {
        let lanes = SourceLanes::plan(src_width, src_scale, dest_scale);
        let narrow_steps = lanes.cur as u32 - dest_width as u32;
        // Source words summed per sub-group, and sub-groups per dest word.
        let group = 1i32 << lanes.cross;
        let repack_count = 1i32 << narrow_steps;

        // At most one sub-group per destination lane, and a u64 holds at most
        // 64 lanes (at width B1), so 64 entries always suffice.
        let mut sums = [0u64; 64];
        let mut or_sums = 0u64;
        for r in 0..repack_count {
            let gstart = src_start + r * group;
            let mut value = 0u64;
            // Plain `+=` rather than `swar_add_checked`: `cross > 0` only once
            // the lanes have reached U64, where a word is a single lane, so
            // this cannot carry between lanes. The u64 itself cannot overflow
            // because `merge_from` has already bounded the combined count.
            for g in 0..group {
                let widx = gstart + g;
                if widx >= other.word_start && widx <= other.word_end {
                    let word = other.data[other.data_idx(widx)];
                    value += lanes.widen_word(src_width, word);
                }
            }
            sums[r as usize] = value;
            or_sums |= lanes.cur.or_fold_lanes(value);
        }

        // Source sums exceed dest counter capacity.
        if or_sums > dest_width.counter_max() {
            return Packed::TooWide(or_sums);
        }

        // Narrow and pack into one dest-width SWAR word.
        Packed::Word(if narrow_steps > 0 {
            let chunk_bits = 64u32 >> narrow_steps;
            let mut acc = 0u64;
            for r in 0..repack_count {
                acc |= narrow(lanes.cur, dest_width, sums[r as usize]) << (r as u32 * chunk_bits);
            }
            acc
        } else {
            sums[0]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::HistogramNN;

    /// Scenario: A histogram carrying observations is merged into a freshly
    /// created, empty histogram.
    /// Guarantees: the destination adopts the source's exact minimum instead
    /// of folding it against the empty sentinel 0.0, so the merged result does
    /// not report a zero observation that was never recorded.
    #[test]
    fn merging_into_an_empty_histogram_adopts_the_source_min() {
        let mut src: HistogramNN<10> = HistogramNN::new();
        for v in [5.0, 100.0, 3.0] {
            src.update(v).unwrap();
        }

        let mut dst: HistogramNN<10> = HistogramNN::new();
        dst.merge_from(&src).unwrap();

        let view = dst.view();
        assert_eq!(view.stats().min, 3.0);
        assert_eq!(view.stats().max, 100.0);
        assert_eq!(view.stats().count, 3);
        assert_eq!(view.scan_buckets(|_| {}).zero_count, 0);
    }

    /// Scenario: A histogram whose population includes an exact zero is merged
    /// into an empty histogram.
    /// Guarantees: the merged minimum is 0.0 and the zero count is preserved,
    /// distinguishing a genuine zero observation from the empty sentinel.
    #[test]
    fn merging_into_an_empty_histogram_preserves_a_real_zero() {
        let mut src: HistogramNN<10> = HistogramNN::new();
        for v in [0.0, 5.0, 100.0] {
            src.update(v).unwrap();
        }

        let mut dst: HistogramNN<10> = HistogramNN::new();
        dst.merge_from(&src).unwrap();

        let view = dst.view();
        assert_eq!(view.stats().min, 0.0);
        assert_eq!(view.stats().count, 3);
        assert_eq!(view.scan_buckets(|_| {}).zero_count, 1);
    }

    /// Scenario: An aggregator that recorded only exact zeros -- a non-zero
    /// count with empty buckets -- is distinguished from a never-touched one,
    /// and is merged into an empty destination.
    /// Guarantees: the all-zero source is not mistaken for empty and skipped:
    /// its count survives the merge as the destination's zero count, keeping
    /// `zero_count == count` for a population that occupies no bucket.
    #[test]
    fn merging_an_all_zero_source_preserves_its_zero_observations() {
        let mut src: HistogramNN<10> = HistogramNN::new();
        for _ in 0..4 {
            src.update(0.0).unwrap();
        }
        assert_eq!(src.view().stats().count, 4, "zeros are counted");
        assert_eq!(
            src.view().scan_buckets(|_| {}).zero_count,
            4,
            "zeros occupy no bucket"
        );

        let mut dst: HistogramNN<10> = HistogramNN::new();
        dst.merge_from(&src).unwrap();

        let view = dst.view();
        assert_eq!(view.stats().count, 4);
        assert_eq!(view.scan_buckets(|_| {}).zero_count, 4);
        assert_eq!(view.stats().min, 0.0);
        assert_eq!(view.stats().max, 0.0);
        assert_eq!(view.stats().sum, 0.0);
    }

    /// Scenario: A positive population is merged into a destination that has
    /// recorded only exact zeros, and the mirror-image merge is performed.
    /// Guarantees: the all-zero destination is treated as populated rather
    /// than empty, so its zeros hold the merged minimum at 0.0; both merge
    /// orders agree on count, zero count, min, and max.
    #[test]
    fn merging_with_an_all_zero_side_is_order_independent() {
        let mut zeros: HistogramNN<10> = HistogramNN::new();
        for _ in 0..2 {
            zeros.update(0.0).unwrap();
        }
        let mut positives: HistogramNN<10> = HistogramNN::new();
        for v in [5.0, 100.0] {
            positives.update(v).unwrap();
        }

        let mut zeros_first = zeros.clone();
        zeros_first.merge_from(&positives).unwrap();
        let mut positives_first = positives.clone();
        positives_first.merge_from(&zeros).unwrap();

        for merged in [&zeros_first, &positives_first] {
            let view = merged.view();
            assert_eq!(view.stats().count, 4);
            assert_eq!(view.scan_buckets(|_| {}).zero_count, 2);
            assert_eq!(view.stats().min, 0.0, "recorded zeros hold the minimum");
            assert_eq!(view.stats().max, 100.0);
            assert_eq!(view.stats().sum, 105.0);
        }
    }

    /// Scenario: A never-touched histogram is merged into a populated one.
    /// Guarantees: the empty source contributes nothing -- count, sum, min,
    /// and max are unchanged -- so an idle aggregator cannot drag the merged
    /// minimum down to its 0.0 sentinel.
    #[test]
    fn merging_an_empty_source_leaves_the_destination_untouched() {
        let mut dst: HistogramNN<10> = HistogramNN::new();
        for v in [5.0, 100.0] {
            dst.update(v).unwrap();
        }

        let empty: HistogramNN<10> = HistogramNN::new();
        dst.merge_from(&empty).unwrap();

        let view = dst.view();
        assert_eq!(view.stats().count, 2);
        assert_eq!(view.scan_buckets(|_| {}).zero_count, 0);
        assert_eq!(view.stats().min, 5.0, "empty source is not a zero sample");
        assert_eq!(view.stats().max, 100.0);
    }

    /// Builds a histogram over `values`.
    fn build<const N: usize>(values: &[f64]) -> HistogramNN<N> {
        let mut h: HistogramNN<N> = HistogramNN::new();
        for &v in values {
            h.update(v).unwrap();
        }
        h
    }

    /// Scenario: Populations that force the destination to downscale, to
    /// widen its counters, or both are merged in from a source with a
    /// different pool size, scale, and counter width.
    /// Guarantees: every positive observation from both sides is still
    /// counted in exactly one bucket afterwards, so the SWAR repacking
    /// neither drops nor duplicates counts while regrouping source words into
    /// the destination's layout.
    #[test]
    fn merging_preserves_every_bucketed_observation() {
        let cases: [(Vec<f64>, Vec<f64>); 5] = [
            // Same scale and width on both sides.
            (
                (1..=50).map(f64::from).collect(),
                (1..=50).map(f64::from).collect(),
            ),
            // Disjoint magnitudes, forcing the destination to downscale.
            (
                (1..=200).map(|i| f64::from(i) * 0.01).collect(),
                (1..=200).map(f64::from).collect(),
            ),
            // Extreme separation: the merged range spans the exponent range.
            (vec![1e-8; 30], vec![1e8; 30]),
            // A heavily populated destination merging a tiny source.
            ((1..=1000).map(f64::from).collect(), vec![1.0, 2.0]),
            // One deep bucket, forcing the destination to widen its counters.
            (
                vec![1.0; 300],
                (1..=17).map(|i| f64::from(i) * 1e3).collect(),
            ),
        ];

        for (i, (into, from)) in cases.iter().enumerate() {
            let mut dst: HistogramNN<10> = build(into);
            let src: HistogramNN<26> = build(from);
            dst.merge_from(&src).unwrap();

            let view = dst.view();
            let totals = view.scan_buckets(|_| {});
            let expected = into.iter().chain(from).filter(|v| **v > 0.0).count() as u64;
            assert_eq!(
                totals.positive_total, expected,
                "case {i}: bucket mass changed"
            );
            assert_eq!(
                view.stats().count,
                (into.len() + from.len()) as u64,
                "case {i}"
            );
            assert_eq!(
                totals.zero_count + totals.positive_total,
                view.stats().count,
                "case {i}: counts no longer partition the population"
            );
        }
    }

    /// Scenario: Randomized populations spanning 60 binary orders of
    /// magnitude are merged between histograms of different pool sizes,
    /// exercising both merge directions -- several source words folded into
    /// one destination word, and one source word spread across several.
    /// Guarantees: the merged histogram accounts for every observation and
    /// reports the true maximum, across the scale and width combinations the
    /// two directions and their overflow retries are reachable from.
    #[test]
    fn merging_preserves_observations_across_random_scales_and_widths() {
        let mut state = 0x243F_6A88_85A3_08D3_u64;
        let mut rand = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let sample = |n: usize, rand: &mut dyn FnMut() -> u64| -> Vec<f64> {
            (0..n)
                .map(|_| {
                    let r = rand();
                    let exponent = ((r >> 8) % 60) as i32 - 30;
                    let mantissa = 1.0 + (r % 1000) as f64 / 1000.0;
                    mantissa * 2f64.powi(exponent)
                })
                .collect()
        };

        for iteration in 0..2000 {
            let into = sample((rand() % 60) as usize, &mut rand);
            let from = sample((rand() % 60) as usize + 1, &mut rand);

            let mut dst: HistogramNN<10> = build(&into);
            let src: HistogramNN<4> = build(&from);
            dst.merge_from(&src).unwrap();

            let view = dst.view();
            let totals = view.scan_buckets(|_| {});
            assert_eq!(
                totals.positive_total,
                (into.len() + from.len()) as u64,
                "iteration {iteration}: bucket mass changed"
            );
            let expected_max = into.iter().chain(&from).copied().fold(f64::MIN, f64::max);
            assert_eq!(view.stats().max, expected_max, "iteration {iteration}");
        }
    }
}
