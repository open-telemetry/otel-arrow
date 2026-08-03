// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test-only verifier for a histogram's resulting geometry.
//!
//! This does not recompute the expected result and compare. It checks the
//! properties any correct result must have, given the values that produced it:
//!
//! 1. Exact: the buckets reproduce the population at the reported scale.
//! 2. Representable: the occupied range fits `N` words at the reported width.
//! 3. Tight: neither the scale nor the counter width could be better, given
//!    the finest scale that was still reachable.
//!
//! Tightness is what distinguishes this from an equality check against a
//! from-scratch optimum. A histogram reaches its geometry by a path -- values
//! arrive in some order, merges combine two states -- and a check that only
//! confirms the counts would accept a result that gave up resolution it never
//! had to. Here the width must be the narrowest that holds the largest bucket,
//! and the next finer scale must genuinely fail to fit.
//!
//! The path does bound what is reachable, which is why the caller states the
//! ceiling: merging cannot undo coarsening a source already did.
//!
//! Counter widths and word spans are computed from a local table rather than
//! from [`Width`], so a mistake in that arithmetic cannot cancel itself out.

use super::{HistogramNN, Width};
use crate::mapping::Scale;
use std::collections::BTreeMap;

const WIDTHS: [Width; 7] = [
    Width::B1,
    Width::B2,
    Width::B4,
    Width::U8,
    Width::U16,
    Width::U32,
    Width::U64,
];

/// Counter bits at each width, in the order of [`WIDTHS`].
const WIDTH_BITS: [u32; 7] = [1, 2, 4, 8, 16, 32, 64];

/// A population's bucket counts at one scale.
struct Population {
    counts: BTreeMap<i32, u64>,
    zero_count: u64,
}

impl Population {
    /// Buckets `values` at `scale`, keeping exact zeros aside.
    fn new(values: &[f64], scale: i32) -> Self {
        let mapping = Scale::new(scale).expect("scale is in range");
        let mut counts = BTreeMap::new();
        let mut zero_count = 0;
        for &value in values {
            assert!(value >= 0.0, "verifier expects non-negative values");
            if value == 0.0 {
                zero_count += 1;
            } else {
                *counts.entry(mapping.map_to_index(value)).or_default() += 1;
            }
        }
        Self { counts, zero_count }
    }

    fn max_count(&self) -> u64 {
        self.counts.values().copied().max().unwrap_or(0)
    }

    /// First and last occupied bucket, or `None` if only zeros were recorded.
    fn bounds(&self) -> Option<(i32, i32)> {
        let first = *self.counts.keys().next()?;
        let last = *self.counts.keys().next_back().expect("first implies last");
        Some((first, last))
    }
}

/// Position of `width` in the width ladder.
fn width_index(width: Width) -> usize {
    WIDTHS
        .iter()
        .position(|&candidate| candidate == width)
        .expect("every width is in the ladder")
}

/// Largest count `width` can hold.
fn width_capacity(width: Width) -> u64 {
    let bits = WIDTH_BITS[width_index(width)];
    if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

/// Narrowest width that can hold `count`.
fn width_holding(count: u64) -> Width {
    WIDTHS
        .into_iter()
        .find(|&width| count <= width_capacity(width))
        .expect("U64 holds every count")
}

/// Words spanned by buckets `first..=last` at `width`.
fn word_span(first: i32, last: i32, width: Width) -> i64 {
    let shift = 6 - WIDTH_BITS[width_index(width)].trailing_zeros();
    i64::from(last >> shift) - i64::from(first >> shift) + 1
}

/// Narrowest width `N` words may use at `scale`.
///
/// Narrow counters define more buckets, and a histogram may never define more
/// buckets than the value range spans, so a low scale rules them out.
fn width_floor<const N: usize>(scale: i32) -> Width {
    WIDTHS
        .into_iter()
        .find(|&width| scale >= HistogramNN::<N>::min_scale(width))
        .expect("U64 counters are legal at every scale")
}

/// The only width a correct result may report for this population.
///
/// Whichever of the three demands is largest wins: the largest bucket must
/// fit, the configured minimum width stands, and the scale imposes a floor.
fn required_width<const N: usize>(population: &Population, scale: i32, minimum: Width) -> Width {
    width_holding(population.max_count())
        .max(minimum)
        .max(width_floor::<N>(scale))
}

/// Whether this population is representable at `scale` in `N` words.
fn fits<const N: usize>(population: &Population, scale: i32, minimum: Width) -> bool {
    let Some((first, last)) = population.bounds() else {
        return true;
    };
    let width = required_width::<N>(population, scale, minimum);
    word_span(first, last, width) <= N as i64
}

/// Compares a sum against a tolerance.
///
/// Merging adds two partial sums, and floating point addition is not
/// associative, so the last bits legitimately depend on the order in which the
/// observations were combined.
fn assert_sum_close(actual: f64, expected: f64) {
    let tolerance = 1e-9 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "sum {actual} is not within {tolerance} of {expected}"
    );
}

/// Asserts the histogram reproduces `values` exactly at the best geometry
/// available to it.
///
/// `values` must be every observation the histogram received, in any order.
///
/// `scale_ceiling` is the finest scale this result could have reached. For a
/// histogram that recorded its own values that is its configured maximum. For
/// a merge it is the coarser of the two inputs: a source that has already
/// given up resolution cannot hand back what its buckets no longer
/// distinguish, however much room the destination has.
pub(crate) fn assert_exact_and_tight<const N: usize>(
    histogram: &HistogramNN<N>,
    values: &[f64],
    scale_ceiling: i32,
) {
    let minimum_width = histogram.initial.width;
    let maximum_scale = scale_ceiling.min(histogram.initial.scale.scale());
    let view = histogram.view();
    let scale = view.scale();
    let buckets = view.positive();

    let stats = view.stats();
    assert_eq!(stats.count, values.len() as u64, "observation count");
    assert_sum_close(stats.sum, values.iter().sum::<f64>());
    assert_eq!(
        stats.min,
        values.iter().copied().fold(f64::INFINITY, f64::min),
        "minimum"
    );
    assert_eq!(
        stats.max,
        values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        "maximum"
    );

    let population = Population::new(values, scale);
    assert_eq!(
        view.scan_buckets(|_| {}).zero_count,
        population.zero_count,
        "exact zeros are counted outside the buckets"
    );

    let Some((first, last)) = population.bounds() else {
        assert_eq!(
            buckets.len(),
            0,
            "an all-zero population occupies no bucket"
        );
        return;
    };

    // Exact: every value lands in the bucket its scale maps it to.
    assert_eq!(
        buckets.offset(),
        first,
        "first occupied bucket at scale {scale}"
    );
    assert_eq!(
        buckets.len(),
        (last - first + 1) as u32,
        "occupied bucket span at scale {scale}"
    );
    for (index, count) in (first..=last).zip(buckets.iter()) {
        assert_eq!(
            count,
            population.counts.get(&index).copied().unwrap_or(0),
            "bucket {index} at scale {scale}"
        );
    }

    // Representable: the occupied range fits, and the counters hold the counts.
    let width = buckets.width();
    assert!(
        word_span(first, last, width) <= N as i64,
        "buckets {first}..={last} at {width:?} do not fit {N} words"
    );
    assert!(
        scale >= HistogramNN::<N>::min_scale(width),
        "scale {scale} is below the floor for {width:?}"
    );

    // Tight in width: anything narrower would drop a count, and anything
    // wider wastes buckets that could have paid for a finer scale.
    assert_eq!(
        width,
        required_width::<N>(&population, scale, minimum_width),
        "largest bucket is {}, so {width:?} is not the width to use at scale {scale}",
        population.max_count()
    );

    // Tight in scale: the next finer scale has to be out of reach.
    if scale < maximum_scale {
        let finer = Population::new(values, scale + 1);
        let (finer_first, finer_last) = finer.bounds().expect("the population has buckets");
        let finer_width = required_width::<N>(&finer, scale + 1, minimum_width);
        assert!(
            !fits::<N>(&finer, scale + 1, minimum_width),
            "scale {} fits too: {finer_width:?} counters hold its largest bucket ({}) and its \
             buckets {finer_first}..={finer_last} span {} of {N} words, so scale {scale} with \
             {width:?} gave up resolution",
            scale + 1,
            finer.max_count(),
            word_span(finer_first, finer_last, finer_width),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::assert_exact_and_tight;
    use crate::histogram::{HistogramNN, Width};
    use crate::mapping::table_scale;

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

        /// A positive value whose exponent spans `octaves` around 1.0.
        fn value(&mut self, octaves: i32) -> f64 {
            let exponent = (self.next() % (2 * octaves as u64 + 1)) as i32 - octaves;
            let fraction = 1.0 + (self.next() % 1024) as f64 / 1024.0;
            fraction * 2f64.powi(exponent)
        }
    }

    fn build<const N: usize>(values: &[f64]) -> HistogramNN<N> {
        let mut histogram = HistogramNN::new();
        for &value in values {
            histogram.update(value).expect("values are recordable");
        }
        histogram
    }

    /// Scenario: populations of varying width and span -- repeated values that
    /// drive counters up the width ladder, and scattered values that drive the
    /// occupied range past the pool -- are recorded into pools of several sizes.
    /// Guarantees: recording reproduces every observation exactly and settles
    /// on the narrowest counter width and the finest scale that fit, so a
    /// counter overflow never costs resolution the pool could have afforded.
    #[test]
    fn recording_keeps_the_finest_geometry_that_fits() {
        let mut rng = Rng(0x243F_6A88_85A3_08D3);
        for octaves in [1, 3, 10, 40] {
            for repeats in [1, 5, 300] {
                let mut values = Vec::new();
                for _ in 0..48 {
                    let value = rng.value(octaves);
                    for _ in 0..repeats {
                        values.push(value);
                    }
                }
                values.push(0.0);

                assert_exact_and_tight(&build::<2>(&values), &values, table_scale());
                assert_exact_and_tight(&build::<8>(&values), &values, table_scale());
                assert_exact_and_tight(&build::<64>(&values), &values, table_scale());
            }
        }
    }

    /// Scenario: a histogram configured with a raised minimum counter width
    /// records a population that never needs counters that wide.
    /// Guarantees: the configured floor is honoured rather than narrowed away,
    /// and the scale still reflects the buckets those wider counters leave.
    #[test]
    fn a_configured_minimum_width_is_honoured() {
        let values = vec![1.0, 2.0, 4.0, 8.0];
        let mut histogram: HistogramNN<8> = HistogramNN::new()
            .with_min_width(Width::U16)
            .expect("the default scale covers U16 counters");
        for &value in &values {
            histogram.update(value).expect("values are recordable");
        }

        assert_eq!(histogram.view().positive().width(), Width::U16);
        assert_exact_and_tight(&histogram, &values, table_scale());
    }

    /// Scenario: two independently recorded populations, differing in scale,
    /// counter width, and pool size, are merged.
    /// Guarantees: the merged histogram accounts for both populations exactly
    /// and is as fine as the coarser input allows, so combining costs no
    /// resolution beyond what the inputs had already given up.
    #[test]
    fn merging_keeps_the_finest_geometry_that_fits() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for left_octaves in [1, 6, 30] {
            for right_octaves in [1, 6, 30] {
                for repeats in [1, 40] {
                    let mut left = Vec::new();
                    let mut right = Vec::new();
                    for _ in 0..24 {
                        let value = rng.value(left_octaves);
                        for _ in 0..repeats {
                            left.push(value);
                        }
                        let value = rng.value(right_octaves);
                        for _ in 0..repeats {
                            right.push(value);
                        }
                    }

                    let mut merged = build::<16>(&left);
                    let source = build::<4>(&right);
                    let ceiling = merged.view().scale().min(source.view().scale());
                    merged
                        .merge_from(&source)
                        .expect("counts stay well inside u64");

                    let mut combined = left.clone();
                    combined.extend_from_slice(&right);
                    assert_exact_and_tight(&merged, &combined, ceiling);
                }
            }
        }
    }

    /// Scenario: several populations are merged one after another into a
    /// destination that has already been merged into.
    /// Guarantees: repeated merging stays exact and stays as fine as its
    /// inputs allow, so resolution does not erode as partial aggregates
    /// accumulate.
    #[test]
    fn repeated_merging_does_not_erode_geometry() {
        let mut rng = Rng(0xB5AD_4ECE_DA1C_E2A9);
        let mut combined = Vec::new();
        let mut destination: HistogramNN<32> = HistogramNN::new();
        let mut ceiling = table_scale();

        for round in 0..8 {
            let mut values = Vec::new();
            for _ in 0..16 {
                let value = rng.value(2 + round);
                for _ in 0..(1 << round) {
                    values.push(value);
                }
            }

            let source = build::<8>(&values);
            ceiling = ceiling.min(source.view().scale());
            destination
                .merge_from(&source)
                .expect("counts stay well inside u64");
            combined.extend_from_slice(&values);
            assert_exact_and_tight(&destination, &combined, ceiling);
        }
    }
}
