// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free merge logic for combining histograms.
//!
//! Geometry selection is a read-only operation over stable snapshots. Each
//! candidate scale and width is tested by normalizing fixed-size clones, then
//! combining packed words into a fixed output buffer. A lane that does not fit
//! widens the candidate and the pass repeats; otherwise the buffered words are
//! installed in one commit, so the destination is never left partly merged.
//!
//! A source holding a single bucket skips that machinery and goes through the
//! ordinary recording path, which reaches the same geometry with less setup.

use super::swar::{spread, swar_add_checked};
use super::width::Width;
use super::{Error, HighLow, HistogramNN, Settings};
use crate::mapping::{MIN_SCALE, Scale};

#[derive(Clone, Copy)]
struct Geometry {
    scale: i32,
    width: Width,
}

fn projected_slot_range<const N: usize>(
    histogram: &HistogramNN<N>,
    target_scale: i32,
) -> Option<HighLow> {
    if histogram.buckets_empty() {
        return None;
    }
    let shift = histogram.current.scale.scale() - target_scale;
    debug_assert!(shift >= 0);
    Some(HighLow {
        low: histogram.first_slot() >> shift,
        high: histogram.last_slot() >> shift,
    })
}

fn merge_ranges(left: Option<HighLow>, right: Option<HighLow>) -> Option<HighLow> {
    match (left, right) {
        (Some(left), Some(right)) => Some(HighLow {
            low: left.low.min(right.low),
            high: left.high.max(right.high),
        }),
        (Some(range), None) | (None, Some(range)) => Some(range),
        (None, None) => None,
    }
}

fn normalize_clone<const N: usize>(
    histogram: &HistogramNN<N>,
    geometry: Geometry,
) -> HistogramNN<N> {
    if histogram.buckets_empty() {
        let mut normalized = histogram.clone();
        normalized.current = Settings::new(
            Scale::new(geometry.scale).expect("candidate scale is valid"),
            geometry.width,
        );
        normalized.word_base = 0;
        normalized.word_start = 0;
        normalized.word_end = 0;
        normalized.data[0] = 0;
        return normalized;
    }

    let mut normalized = histogram.clone();
    let mut scale_decrease = normalized.current.scale.scale() - geometry.scale;
    let width_increase = geometry.width.subtract(normalized.current.width) as u32;
    let aligned_steps = width_increase.min(scale_decrease as u32);
    if aligned_steps > 0 {
        let before = normalized.current.width;
        let after = before
            .wider_by(aligned_steps)
            .expect("candidate width bounds alignment");
        let _ = normalized.widen_words(before, after);
        normalized.current.width = after;
        normalized.change_scale(aligned_steps);
        scale_decrease -= aligned_steps as i32;
    }

    // The width floor above gives do_downscale enough scale slack even when a
    // smaller source pool cannot hold the final destination width itself.
    if scale_decrease > 0 {
        normalized.downscale_by(scale_decrease as u32);
    }
    normalized
}

#[inline]
fn raw_word<const N: usize>(histogram: &HistogramNN<N>, word_index: i32) -> u64 {
    if histogram.buckets_empty()
        || word_index < histogram.word_start
        || word_index > histogram.word_end
    {
        0
    } else {
        histogram.data[histogram.data_idx(word_index)]
    }
}

fn packed_word_at_width<const N: usize>(
    histogram: &HistogramNN<N>,
    target_width: Width,
    word_index: i32,
) -> u64 {
    let source_width = histogram.current.width;
    if source_width == target_width {
        return raw_word(histogram, word_index);
    }

    debug_assert!(source_width < target_width);
    let steps = target_width.subtract(source_width) as u32;
    let words_per_source = 1_i32 << steps;
    let packed_bits = 64_u32 >> steps;
    let source_word = word_index >> steps;
    let chunk = word_index.rem_euclid(words_per_source) as u32;
    let packed = raw_word(histogram, source_word) >> (chunk * packed_bits);
    spread(source_width, target_width, packed)
}

fn required_add_width(left: u64, right: u64, width: Width) -> Width {
    let lane_bits = width.bits_per_slot();
    let lane_max = width.counter_max();
    let mut maximum = 0u64;
    let mut shift = 0;
    while shift < 64 {
        let left_lane = (left >> shift) & lane_max;
        let right_lane = (right >> shift) & lane_max;
        maximum = maximum.max(
            left_lane
                .checked_add(right_lane)
                .expect("bucket sum is bounded by aggregate count"),
        );
        shift += lane_bits;
    }
    Width::from_max_value(maximum)
}

fn combine_words<const N: usize, const M: usize>(
    left: &HistogramNN<N>,
    right: &HistogramNN<M>,
    width: Width,
    words: HighLow,
    output: &mut [u64; N],
) -> Width {
    let mut required = width;
    for word_index in words.low..=words.high {
        let left = packed_word_at_width(left, width, word_index);
        let right = packed_word_at_width(right, width, word_index);
        let output_index = (word_index - words.low) as usize;
        match swar_add_checked(left, right, width) {
            Some(combined) => output[output_index] = combined,
            None => {
                required = required.max(required_add_width(left, right, width));
            }
        }
    }
    required
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

    /// Selects stable packed geometry, preflights it, then commits it.
    fn merge_buckets<const M: usize>(&mut self, other: &HistogramNN<M>) {
        if other.buckets_empty() {
            return;
        }
        // One bucket cannot amortize geometry selection, and the recording
        // path already lands on the same scale and width.
        if other.trimmed_slot_count() == 1 {
            self.merge_buckets_sequential(other)
                .expect("aggregate count precheck bounds every bucket");
            return;
        }

        let mut geometry = Geometry {
            scale: self.current.scale.scale().min(other.current.scale.scale()),
            width: self
                .current
                .width
                .max(self.initial.width)
                .max(other.current.width),
        };
        let mut output = [0_u64; N];

        loop {
            while geometry.scale < geometry.width.min_scale() {
                geometry.width = geometry
                    .width
                    .wider_by(1)
                    .expect("U64 counters cover MIN_SCALE");
            }

            let slots = merge_ranges(
                projected_slot_range(self, geometry.scale),
                projected_slot_range(other, geometry.scale),
            )
            .expect("source has buckets");
            let words = HighLow {
                low: geometry.width.slot_to_word_index(slots.low),
                high: geometry.width.slot_to_word_index(slots.high),
            };
            let scale_decrease = words.change_steps(N);
            if scale_decrease > 0 {
                geometry.scale -= scale_decrease as i32;
                debug_assert!(geometry.scale >= MIN_SCALE);
                continue;
            }

            if self.current.scale.scale() == geometry.scale
                && other.current.scale.scale() == geometry.scale
            {
                let required_width = combine_words(self, other, geometry.width, words, &mut output);
                if required_width > geometry.width {
                    geometry.width = required_width;
                    continue;
                }
                self.commit_packed_words(geometry, words, &output);
                return;
            }

            let destination = normalize_clone(self, geometry);
            let source = normalize_clone(other, geometry);
            let normalized_width = destination.current.width.max(source.current.width);
            if normalized_width > geometry.width {
                geometry.width = normalized_width;
                continue;
            }

            let normalized_scale = destination
                .current
                .scale
                .scale()
                .min(source.current.scale.scale());
            if normalized_scale < geometry.scale {
                geometry.scale = normalized_scale;
                continue;
            }
            debug_assert_eq!(destination.current.scale.scale(), geometry.scale);
            debug_assert_eq!(source.current.scale.scale(), geometry.scale);

            let required_width =
                combine_words(&destination, &source, geometry.width, words, &mut output);
            if required_width > geometry.width {
                geometry.width = required_width;
                continue;
            }

            self.commit_packed_words(geometry, words, &output);
            return;
        }
    }

    /// Installs words combined under the selected geometry.
    fn commit_packed_words(&mut self, geometry: Geometry, words: HighLow, output: &[u64; N]) {
        debug_assert!((words.high - words.low) < N as i32);

        self.current = Settings::new(
            Scale::new(geometry.scale).expect("candidate scale is valid"),
            geometry.width,
        );
        self.word_base = words.low;
        self.word_start = words.low;
        self.word_end = words.high;

        for word_index in words.low..=words.high {
            let output_index = (word_index - words.low) as usize;
            self.data[output_index] = output[output_index];
        }
        self.debug_assert_range_coverage();
    }

    /// Sequential merge retained as a benchmark and test reference.
    #[cfg(any(test, feature = "bench"))]
    fn merge_from_sequential_impl<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
    ) -> Result<(), Error> {
        if other.stats.count == 0 {
            return Ok(());
        }

        let new_count = self
            .checked_add_count(other.stats.count)
            .ok_or(Error::Overflow)?;
        self.merge_buckets_sequential(other)?;
        self.commit_merge(&other.stats, new_count);
        Ok(())
    }

    /// Exposes the sequential reference implementation to Criterion benches.
    #[cfg(feature = "bench")]
    #[doc(hidden)]
    pub fn merge_from_sequential_reference<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
    ) -> Result<(), Error> {
        self.merge_from_sequential_impl(other)
    }

    /// Inserts source buckets through the ordinary recording retry path.
    fn merge_buckets_sequential<const M: usize>(
        &mut self,
        other: &HistogramNN<M>,
    ) -> Result<(), Error> {
        if other.buckets_empty() {
            return Ok(());
        }

        let common_scale = self.current.scale.scale().min(other.current.scale.scale());
        let destination_decrease = self.current.scale.scale() - common_scale;
        if destination_decrease > 0 {
            if self.buckets_empty() {
                self.change_scale(destination_decrease as u32);
            } else {
                self.downscale_by(destination_decrease as u32);
            }
        }

        let mut source = other.clone();
        let source_decrease = source.current.scale.scale() - common_scale;
        if source_decrease > 0 {
            source.downscale_by(source_decrease as u32);
        }

        for source_slot in source.first_slot()..=source.last_slot() {
            let count = source.bucket_get(&source.current.width.slot_addr(source_slot));
            if count == 0 {
                continue;
            }
            self.retry_increment(count, |destination| {
                source_slot >> (common_scale - destination.current.scale.scale())
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::HistogramNN;
    use crate::histogram::Width;
    use crate::mapping::{MIN_SCALE, Scale, table_scale};
    use std::collections::BTreeMap;
    use std::num::NonZeroU64;

    #[derive(Debug)]
    struct Oracle {
        scale: i32,
        width: Width,
        counts: BTreeMap<i32, u64>,
    }

    fn oracle_width(max_count: u64) -> Width {
        match max_count {
            0..=1 => Width::B1,
            2..=3 => Width::B2,
            4..=15 => Width::B4,
            16..=255 => Width::U8,
            256..=65_535 => Width::U16,
            65_536..=4_294_967_295 => Width::U32,
            _ => Width::U64,
        }
    }

    fn oracle_min_scale(width: Width) -> i32 {
        MIN_SCALE + (Width::U64 as i32 - width as i32)
    }

    fn geometry_oracle<const N: usize>(values: &[f64]) -> Oracle {
        for scale in (MIN_SCALE..=table_scale()).rev() {
            let mapping = Scale::new(scale).unwrap();
            let mut counts = BTreeMap::<i32, u64>::new();
            for &value in values {
                if value == 0.0 {
                    continue;
                }
                *counts.entry(mapping.map_to_index(value)).or_default() += 1;
            }

            let max_count = counts.values().copied().max().unwrap_or(0);
            let mut width = oracle_width(max_count);
            while scale < oracle_min_scale(width) {
                width = match width {
                    Width::B1 => Width::B2,
                    Width::B2 => Width::B4,
                    Width::B4 => Width::U8,
                    Width::U8 => Width::U16,
                    Width::U16 => Width::U32,
                    Width::U32 | Width::U64 => Width::U64,
                };
            }

            let first = *counts.keys().next().expect("oracle needs positive values");
            let last = *counts.keys().next_back().unwrap();
            let word_shift = Width::U64 as u32 - width as u32;
            let first_word = first >> word_shift;
            let last_word = last >> word_shift;
            if i64::from(last_word) - i64::from(first_word) < N as i64 {
                return Oracle {
                    scale,
                    width,
                    counts,
                };
            }
        }
        unreachable!("MIN_SCALE with U64 counters fits all values");
    }

    fn assert_matches_oracle<const N: usize>(
        histogram: &HistogramNN<N>,
        values: &[f64],
        oracle: &Oracle,
    ) {
        let view = histogram.view();
        assert_eq!(view.scale(), oracle.scale);
        assert_eq!(view.positive().width(), oracle.width);

        let first = *oracle.counts.keys().next().unwrap();
        let last = *oracle.counts.keys().next_back().unwrap();
        assert_eq!(view.positive().offset(), first);
        assert_eq!(view.positive().len(), (last - first + 1) as u32);
        for (slot, actual) in (first..=last).zip(view.positive().iter()) {
            assert_eq!(
                actual,
                oracle.counts.get(&slot).copied().unwrap_or(0),
                "bucket {slot}"
            );
        }

        let stats = view.stats();
        assert_eq!(stats.count, values.len() as u64);
        assert_eq!(stats.sum, values.iter().sum::<f64>());
        assert_eq!(
            stats.min,
            values.iter().copied().fold(f64::INFINITY, f64::min)
        );
        assert_eq!(
            stats.max,
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        );
        let totals = view.scan_buckets(|_| {});
        assert_eq!(
            totals.positive_total,
            values.iter().filter(|&&value| value != 0.0).count() as u64
        );
        assert_eq!(
            totals.zero_count,
            values.iter().filter(|&&value| value == 0.0).count() as u64
        );
    }

    fn assert_histograms_equivalent<const N: usize>(
        optimized: &HistogramNN<N>,
        sequential: &HistogramNN<N>,
    ) {
        assert_eq!(optimized.current_settings(), sequential.current_settings());

        let optimized = optimized.view();
        let sequential = sequential.view();
        let optimized_stats = optimized.stats();
        let sequential_stats = sequential.stats();
        assert_eq!(optimized_stats.count, sequential_stats.count);
        assert_eq!(optimized_stats.sum, sequential_stats.sum);
        assert_eq!(optimized_stats.min, sequential_stats.min);
        assert_eq!(optimized_stats.max, sequential_stats.max);

        let optimized = optimized.positive();
        let sequential = sequential.positive();
        assert_eq!(optimized.width(), sequential.width());
        assert_eq!(optimized.offset(), sequential.offset());
        assert_eq!(optimized.len(), sequential.len());
        assert_eq!(
            optimized.iter().collect::<Vec<_>>(),
            sequential.iter().collect::<Vec<_>>()
        );
    }

    /// Scenario: one scale-8 bucket receives 100 identical observations and
    /// its packed counter must grow from B1 through U8.
    /// Guarantees: unused backing words pay for wider counters, preserving
    /// scale 8 and the exact count instead of coarsening to scale 5.
    #[test]
    fn hot_single_bucket_widening_preserves_scale() {
        let values = vec![1.0; 100];
        let oracle = geometry_oracle::<64>(&values);
        assert_eq!((oracle.scale, oracle.width), (8, Width::U8));

        let histogram = build::<64>(&values);
        assert_matches_oracle(&histogram, &values, &oracle);
    }

    /// Scenario: a bucket below 1.0 has a negative scale-8 index and receives
    /// enough observations to widen its counter to U8.
    /// Guarantees: circular addressing preserves the negative bucket index,
    /// scale, count, and statistics while widening.
    #[test]
    fn negative_bucket_widening_preserves_scale_and_index() {
        let values = vec![0.5; 100];
        let oracle = geometry_oracle::<64>(&values);
        assert_eq!((oracle.scale, oracle.width), (8, Width::U8));
        assert!(*oracle.counts.keys().next().unwrap() < 0);

        let histogram = build::<64>(&values);
        assert_matches_oracle(&histogram, &values, &oracle);
    }

    /// Scenario: a hot bucket and a distant sparse bucket cannot fit at scale
    /// 8 once the hot counter requires U8 storage.
    /// Guarantees: production chooses exactly the oracle's highest feasible
    /// lower scale and preserves all bucket mass and statistics.
    #[test]
    fn wide_sparse_range_downscales_only_as_far_as_required() {
        let mut values = vec![1.0; 100];
        values.push(2f64.powi(20));
        let oracle = geometry_oracle::<8>(&values);
        assert!(oracle.scale < table_scale());

        let histogram = build::<8>(&values);
        assert_matches_oracle(&histogram, &values, &oracle);
    }

    /// Scenario: two histograms have overlapping hot scale-8 buckets whose
    /// individual B4 counts combine into a U8 count during merge.
    /// Guarantees: merge selects the combined oracle geometry before writing,
    /// preserving scale 8, exact bucket mass, zeros, and aggregate statistics.
    #[test]
    fn merging_overlapping_hot_buckets_uses_combined_geometry() {
        let mut left_values = vec![0.5; 10];
        left_values.push(0.0);
        let mut right_values = vec![0.5; 10];
        right_values.push(0.0);

        let mut merged_values = left_values.clone();
        merged_values.extend_from_slice(&right_values);
        let oracle = geometry_oracle::<64>(&merged_values);
        assert_eq!((oracle.scale, oracle.width), (8, Width::U8));

        let original = build::<64>(&left_values);
        let mut destination = original.clone();
        let source = build::<4>(&right_values);
        destination.merge_from(&source).unwrap();
        assert_matches_oracle(&destination, &merged_values, &oracle);

        let mut sequential = original;
        sequential.merge_from_sequential_impl(&source).unwrap();
        assert_histograms_equivalent(&destination, &sequential);
    }

    /// Scenario: individually fine-scale histograms occupy a hot low bucket
    /// and a distant high bucket whose combined U8 geometry needs downscaling.
    /// Guarantees: merge selects the oracle's minimal scale reduction from
    /// stable sources and preserves every count and statistic.
    #[test]
    fn merging_wide_range_matches_oracle_geometry() {
        let left_values = vec![1.0; 100];
        let right_values = vec![2f64.powi(20)];
        let mut merged_values = left_values.clone();
        merged_values.extend_from_slice(&right_values);
        let oracle = geometry_oracle::<8>(&merged_values);
        assert!(oracle.scale < table_scale());

        let mut destination = build::<8>(&left_values);
        let source = build::<4>(&right_values);
        destination.merge_from(&source).unwrap();
        assert_matches_oracle(&destination, &merged_values, &oracle);
    }

    /// Scenario: merging one more observation into a histogram whose total
    /// count is already u64::MAX would overflow the aggregate count.
    /// Guarantees: merge reports Overflow before rebuilding, leaving bucket
    /// geometry, bucket mass, and statistics unchanged.
    #[test]
    fn merge_count_overflow_leaves_destination_unchanged() {
        let mut destination: HistogramNN<64> = HistogramNN::new();
        destination
            .record_incr(1.0, NonZeroU64::new(u64::MAX).unwrap())
            .unwrap();
        let before_settings = destination.current_settings();
        let before_stats = destination.view().stats();
        let before_buckets = destination.view().positive().iter().collect::<Vec<_>>();

        let source = build::<4>(&[1.0]);
        assert_eq!(
            destination.merge_from(&source),
            Err(super::super::Error::Overflow)
        );
        assert_eq!(destination.current_settings(), before_settings);
        let after_stats = destination.view().stats();
        assert_eq!(after_stats.count, before_stats.count);
        assert_eq!(after_stats.sum, before_stats.sum);
        assert_eq!(after_stats.min, before_stats.min);
        assert_eq!(after_stats.max, before_stats.max);
        assert_eq!(
            destination.view().positive().iter().collect::<Vec<_>>(),
            before_buckets
        );
    }

    /// Scenario: two histograms each hold two buckets carrying a quarter of
    /// u64::MAX, so the packed path must add counters at the widest rung.
    /// Guarantees: the combined counters reach U64 and hold the exact sums,
    /// so a merge at the top of the width ladder neither wraps nor saturates.
    #[test]
    fn merging_counts_that_fill_u64_counters_stays_exact() {
        let quarter = u64::MAX / 4;
        let increment = NonZeroU64::new(quarter).expect("a quarter of u64::MAX is non-zero");
        let mut destination: HistogramNN<8> = HistogramNN::new();
        let mut source: HistogramNN<8> = HistogramNN::new();
        for value in [1.0, 4.0] {
            destination
                .record_incr(value, increment)
                .expect("the first quarter fits");
            source
                .record_incr(value, increment)
                .expect("the first quarter fits");
        }

        destination
            .merge_from(&source)
            .expect("four quarters fit in u64");

        let view = destination.view();
        assert_eq!(view.stats().count, 4 * quarter);
        assert_eq!(view.positive().width(), Width::U64);
        let occupied: Vec<u64> = view.positive().iter().filter(|&count| count != 0).collect();
        assert_eq!(occupied, vec![2 * quarter, 2 * quarter]);
    }

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
    /// scales, and counter widths.
    /// Guarantees: the merged histogram accounts for every observation and
    /// reports the true maximum across all reachable source geometries.
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
