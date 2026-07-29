// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Merge logic for combining histograms.
//!
//! 1. **Prepare** -- Set width to `max(W_self, W_src)` and downscale
//!    self so the combined slot range fits in N words.
//! 2. **Merge** -- For each dest word, repack its contributing source
//!    words (widen, cross-word sum, narrow, pack) then
//!    `swar_add_checked`. Overflow widens self and retries;
//!    `tm_log` is invariant under widening so group boundaries are
//!    stable.

use super::swar::{narrow, swar_add_checked, widen};
use super::width::Width;
use super::{Error, HighLow, HistogramNN};

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
    /// `tm_log = shift + src_w - dest_w` is the log2 of source words
    /// per dest word.  It is invariant under self-widening, so group
    /// boundaries are stable across overflow retries.
    ///
    /// When `tm_log >= 0`, each dest word maps to `2^tm_log` source
    /// words (the normal case).  When `tm_log < 0`, each source word
    /// spans `2^(-tm_log)` dest words -- this happens when the dest is
    /// much wider than the source after downscaling.
    fn merge_words<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
    ) {
        let shift = src_scale - self.current.scale.scale();
        let tm_log = shift + src_width as i32 - self.current.width as i32;

        if tm_log >= 0 {
            self.merge_words_positive(other, src_width, src_scale, tm_log as u32);
        } else {
            self.merge_words_negative(other, src_width, src_scale, (-tm_log) as u32);
        }
    }

    /// Merge when `tm_log >= 0`: each dest word <- `2^tm_log` source words.
    fn merge_words_positive<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        tm_log: u32,
    ) {
        debug_assert!(tm_log < 31, "tm_log={tm_log}: scale bounds violated",);

        let total_merge = 1i32 << tm_log;
        let aligned_start = other.word_start & !(total_merge - 1);
        let dest_lo = aligned_start >> tm_log;
        let dest_hi = other.word_end >> tm_log;

        self.extend_word_range(dest_lo, dest_hi);

        for dest_widx in dest_lo..=dest_hi {
            let src_start = dest_widx << tm_log;

            loop {
                let acc = match Self::repack_source(
                    other,
                    src_width,
                    src_scale,
                    self.current.scale.scale(),
                    self.current.width,
                    src_start,
                ) {
                    Err(or_sums) => {
                        self.widen_to(Width::from_max_value(or_sums));
                        continue;
                    }
                    Ok(0) => break,
                    Ok(acc) => acc,
                };

                let dest_width = self.current.width;
                let didx = self.data_idx(dest_widx);
                if let Some(result) = swar_add_checked(self.data[didx], acc, dest_width) {
                    self.data[didx] = result;
                    break;
                }

                // Overflow: widen self and retry.
                let max_a = dest_width.or_fold_lanes(self.data[didx]);
                let max_b = dest_width.or_fold_lanes(acc);
                self.widen_to(Width::from_max_value(max_a + max_b));
            }
        }
    }

    /// Merge when `tm_log < 0`: each source word -> `2^neg_tm` dest words.
    ///
    /// After widening the source by `in_word` steps to `cur`, each
    /// cur-lane equals one dest slot.  Since `cur < dest_width`, each
    /// source word's cur-lanes are split across multiple dest words.
    /// We iterate over source words and distribute lane groups to
    /// their respective dest words.
    fn merge_words_negative<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        neg_tm: u32,
    ) {
        let dests_per_src = 1i32 << neg_tm;

        for src_widx in other.word_start..=other.word_end {
            let raw = other.data[other.data_idx(src_widx)];
            if raw == 0 {
                continue;
            }

            for k in 0..dests_per_src {
                let dest_widx = src_widx * dests_per_src + k;

                loop {
                    let acc = match Self::extract_source_chunk(
                        other,
                        src_width,
                        src_scale,
                        self.current.scale.scale(),
                        self.current.width,
                        src_widx,
                        k,
                    ) {
                        Err(or_val) => {
                            self.widen_to(Width::from_max_value(or_val));
                            continue;
                        }
                        Ok(0) => break,
                        Ok(acc) => acc,
                    };

                    // Extend word range only for non-zero contributions
                    // to avoid clobbering data in the circular buffer.
                    self.extend_word_range(dest_widx, dest_widx);

                    let dest_width = self.current.width;
                    let didx = self.data_idx(dest_widx);
                    if let Some(result) = swar_add_checked(self.data[didx], acc, dest_width) {
                        self.data[didx] = result;
                        break;
                    }

                    let max_a = dest_width.or_fold_lanes(self.data[didx]);
                    let max_b = dest_width.or_fold_lanes(acc);
                    self.widen_to(Width::from_max_value(max_a + max_b));
                }
            }
        }
    }

    /// Extract one dest word's worth of data from a single source word
    /// for the `tm_log < 0` case.
    ///
    /// Widens the source word by `in_word` steps to `cur`, then
    /// extracts the cur-lanes belonging to dest word `chunk_index`
    /// and packs them into a dest-width SWAR word.
    ///
    /// Returns `Err(max_lane)` if any lane value exceeds
    /// `dest_width.counter_max()`.
    fn extract_source_chunk<const M: usize>(
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        dest_scale: i32,
        dest_width: Width,
        src_widx: i32,
        chunk_index: i32,
    ) -> Result<u64, u64> {
        let shift = (src_scale - dest_scale).max(0) as u32;
        let in_word = shift.min(src_width.to_u64_widen_steps());

        let cur = if in_word > 0 {
            src_width.wider_by(in_word).expect("capped at U64")
        } else {
            src_width
        };

        // Read and widen source word.
        let raw = if src_widx >= other.word_start && src_widx <= other.word_end {
            other.data[other.data_idx(src_widx)]
        } else {
            return Ok(0);
        };
        let widened = if in_word > 0 {
            widen(src_width, cur, raw)
        } else {
            raw
        };

        // Lane geometry at cur width.
        let cur_lane_bits = 1u32 << (cur as u32);
        let cur_lane_mask = if cur_lane_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << cur_lane_bits) - 1
        };
        let lanes_per_cur = 1u32 << (6 - cur as u32);

        // Lane geometry at dest width.
        let dest_lane_bits = 1u32 << (dest_width as u32);
        let lanes_per_dest = 1u32 << (6 - dest_width as u32);

        // Extract the lanes belonging to this chunk and pack into
        // a dest-width SWAR word.  Each cur-lane value equals one
        // dest slot count (the in-word widening handled the scale
        // shift), so we just zero-extend into the wider dest lanes.
        let start_lane = chunk_index as u32 * lanes_per_dest;
        let mut packed = 0u64;
        let mut or_val = 0u64;

        for j in 0..lanes_per_dest {
            let lane_idx = start_lane + j;
            if lane_idx >= lanes_per_cur {
                break;
            }
            let val = (widened >> (lane_idx * cur_lane_bits)) & cur_lane_mask;
            or_val |= val;
            packed |= val << (j * dest_lane_bits);
        }

        if or_val > dest_width.counter_max() {
            return Err(or_val);
        }

        Ok(packed)
    }

    /// Repack source words `[src_start .. src_start + total_merge)`
    /// into a single dest-width SWAR word.
    ///
    /// Decomposes the scale shift into in-word widening (src lanes
    /// toward U64) and cross-word grouping (sum adjacent words), then
    /// narrows the result to dest_width.
    ///
    /// Returns `Err(or_sums)` if the source sums overflow dest_width
    /// lanes (caller must widen self and retry).
    fn repack_source<const M: usize>(
        other: &HistogramNN<M>,
        src_width: Width,
        src_scale: i32,
        dest_scale: i32,
        dest_width: Width,
        src_start: i32,
    ) -> Result<u64, u64> {
        let shift = (src_scale - dest_scale).max(0) as u32;
        let in_word = shift.min(src_width.to_u64_widen_steps());
        let cross = shift - in_word;
        let cur = if in_word > 0 {
            src_width.wider_by(in_word).expect("capped at U64")
        } else {
            src_width
        };
        let narrow_steps = cur as u32 - dest_width as u32;
        let group = 1i32 << cross;
        let repack_count = 1i32 << narrow_steps;

        // Gather sub-group sums at `cur` width.
        let mut sums = [0u64; 64];
        let mut or_sums = 0u64;
        for r in 0..repack_count {
            let gstart = src_start + r * group;
            let mut value = 0u64;
            for g in 0..group {
                let widx = gstart + g;
                if widx >= other.word_start && widx <= other.word_end {
                    let word = other.data[other.data_idx(widx)];
                    value += if in_word > 0 {
                        widen(src_width, cur, word)
                    } else {
                        word
                    };
                }
            }
            sums[r as usize] = value;
            or_sums |= cur.or_fold_lanes(value);
        }

        // Source sums exceed dest counter capacity.
        if or_sums > dest_width.counter_max() {
            return Err(or_sums);
        }

        // Narrow and pack into one dest-width SWAR word.
        Ok(if narrow_steps > 0 {
            let chunk_bits = 64u32 >> narrow_steps;
            let mut acc = 0u64;
            for r in 0..repack_count {
                acc |= narrow(cur, dest_width, sums[r as usize]) << (r as u32 * chunk_bits);
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
}
