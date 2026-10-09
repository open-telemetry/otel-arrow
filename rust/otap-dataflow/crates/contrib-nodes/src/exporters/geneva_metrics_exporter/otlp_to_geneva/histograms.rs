// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP histogram conversion helpers.

use otel_arrow_dfe_pdata_views::views::metrics::{BucketsView, HistogramDataPointView};

use super::super::encoder::MetricHistogram;

pub(super) const MIN_EXPONENTIAL_SCALE: i32 = -11;
pub(super) const MAX_EXPONENTIAL_SCALE: i32 = 20;
const MAX_EXPONENTIAL_BUCKETS: usize = 502;

/// Outcome of parsing an OTLP explicit histogram's bucket distribution.
#[derive(Debug, PartialEq)]
pub(super) enum ExplicitHistogram {
    /// Bucket counts did not match the number of bounds; the data point must be rejected.
    Invalid,
    /// No bucket distribution was present (scalar count/sum only).
    Empty,
    /// A valid bucket distribution.
    Buckets(MetricHistogram),
}

/// Validates and builds an explicit histogram's bucket distribution
pub(super) fn explicit_histogram<P>(point: &P) -> ExplicitHistogram
where
    P: HistogramDataPointView,
{
    let mut counts = point.bucket_counts();
    let mut current_count = counts.next();
    let bounds = point.explicit_bounds();
    let mut buckets = Vec::with_capacity(bounds.size_hint().0 + 1);
    let mut bound_count = 0_usize;
    // Geneva's percentile lookup returns a bucket's own bound as the answer, so it must stay
    // finite; matches the Metrics Extension behavior of treating an absent last bound as 0.0,
    // giving bound+1 = 1.0.
    let mut overflow_bound = 1.0;
    let mut total = 0_u64;
    for bound in bounds {
        if bound.is_nan() {
            return ExplicitHistogram::Invalid;
        }
        bound_count += 1;
        let Some(count) = current_count else {
            return ExplicitHistogram::Invalid;
        };
        let Some(running_total) = total.checked_add(count) else {
            return ExplicitHistogram::Invalid;
        };
        total = running_total;
        buckets.push((bound, clamp_bucket_count(count)));
        // `next_up` avoids `bound + 1.0` rounding back to `bound` for large bounds.
        let next_bound = bound.next_up();
        if next_bound.is_infinite() {
            return ExplicitHistogram::Invalid;
        }
        overflow_bound = next_bound;
        current_count = counts.next();
    }
    if bound_count == 0 {
        return match current_count {
            Some(count) if count == point.count() && counts.next().is_none() => {
                ExplicitHistogram::Buckets(MetricHistogram::Explicit(vec![(
                    overflow_bound,
                    clamp_bucket_count(count),
                )]))
            }
            Some(_) => ExplicitHistogram::Invalid,
            None => ExplicitHistogram::Empty,
        };
    }
    let Some(overflow_count) = current_count else {
        return ExplicitHistogram::Invalid;
    };
    let Some(total) = total.checked_add(overflow_count) else {
        return ExplicitHistogram::Invalid;
    };
    // The bucket-count array must be exactly one longer than the bounds array; any counts
    // remaining after the overflow bucket mean the point's shape is malformed.
    if total != point.count() || counts.next().is_some() {
        return ExplicitHistogram::Invalid;
    }
    buckets.push((overflow_bound, clamp_bucket_count(overflow_count)));
    normalize_explicit_buckets(&mut buckets);
    ExplicitHistogram::Buckets(MetricHistogram::Explicit(buckets))
}

fn clamp_bucket_count(count: u64) -> u32 {
    count.min(u64::from(u32::MAX)) as u32
}

fn normalize_explicit_buckets(buckets: &mut Vec<(f64, u32)>) {
    buckets.sort_unstable_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .expect("NaN bounds are rejected before normalization")
    });
    let mut write_index = 0;
    for read_index in 0..buckets.len() {
        let (bound, count) = buckets[read_index];
        if write_index > 0 && buckets[write_index - 1].0 == bound {
            // Matches ME's unguarded `uint32_t +=` merge (wraps on overflow).
            buckets[write_index - 1].1 = buckets[write_index - 1].1.wrapping_add(count);
        } else {
            buckets[write_index] = (bound, count);
            write_index += 1;
        }
    }
    buckets.truncate(write_index);
}

pub(super) fn sparse_buckets<B>(buckets: Option<B>) -> Option<Vec<(i32, u64)>>
where
    B: BucketsView,
{
    let Some(buckets) = buckets else {
        return Some(Vec::new());
    };
    buckets
        .bucket_counts()
        .enumerate()
        .filter(|(_, count)| *count != 0)
        .map(|(index, count)| {
            let index = i32::try_from(index).ok()?;
            Some((buckets.offset().checked_add(index)?, count))
        })
        .collect()
}

pub(super) fn bucket_sum(buckets: &[(i32, u64)]) -> Option<u64> {
    buckets
        .iter()
        .try_fold(0_u64, |total, (_, count)| total.checked_add(*count))
}

pub(super) fn downscale_if_required(
    scale: &mut i32,
    positive: &mut Vec<(i32, u64)>,
    negative: &mut Vec<(i32, u64)>,
) {
    while positive.len().max(negative.len()) > MAX_EXPONENTIAL_BUCKETS {
        let largest = positive.len().max(negative.len());
        let ratio = largest.div_ceil(MAX_EXPONENTIAL_BUCKETS);
        let factor = usize::BITS - (ratio - 1).leading_zeros();
        downscale_buckets(positive, factor);
        downscale_buckets(negative, factor);
        *scale -= i32::try_from(factor).unwrap_or(i32::MAX);
    }
}

fn downscale_buckets(buckets: &mut Vec<(i32, u64)>, factor: u32) {
    let divisor = 1_i64 << factor.min(31);
    let mut downscaled: Vec<(i32, u64)> = Vec::with_capacity(buckets.len());
    for &(index, count) in buckets.iter() {
        let new_index = i64::from(index).div_euclid(divisor) as i32;
        if let Some((last_index, last_count)) = downscaled.last_mut()
            && *last_index == new_index
        {
            *last_count = last_count.saturating_add(count);
        } else {
            downscaled.push((new_index, count));
        }
    }
    *buckets = downscaled;
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        HistogramDataPoint, exponential_histogram_data_point,
    };
    use otel_arrow_dfe_pdata::views::otlp::proto::metrics::{ObjBuckets, ObjHistogramDataPoint};
    use otel_arrow_dfe_pdata::views::otlp::proto::wrappers::Wraps;

    use super::*;

    fn explicit_histogram(point: &HistogramDataPoint) -> ExplicitHistogram {
        super::explicit_histogram(&ObjHistogramDataPoint::new(point))
    }

    fn sparse_buckets(
        buckets: Option<&exponential_histogram_data_point::Buckets>,
    ) -> Option<Vec<(i32, u64)>> {
        super::sparse_buckets(buckets.map(ObjBuckets::new))
    }

    fn explicit_point(bounds: Vec<f64>, counts: Vec<u64>, count: u64) -> HistogramDataPoint {
        HistogramDataPoint {
            explicit_bounds: bounds,
            bucket_counts: counts,
            count,
            ..Default::default()
        }
    }

    fn exponential_buckets(
        offset: i32,
        bucket_counts: Vec<u64>,
    ) -> exponential_histogram_data_point::Buckets {
        exponential_histogram_data_point::Buckets {
            offset,
            bucket_counts,
        }
    }

    /// Scenario: Explicit histogram bucket counts omit, exactly match, or exceed the required overflow bucket.
    /// Guarantees: Missing overflow counts are rejected, a correctly sized count array is mapped, and counts
    /// beyond the required overflow bucket are rejected rather than silently ignored.
    #[test]
    fn validates_explicit_histogram_bucket_shape() {
        assert_eq!(
            explicit_histogram(&explicit_point(vec![1.0, 2.0], vec![3, 4], 7)),
            ExplicitHistogram::Invalid
        );
        assert_eq!(
            explicit_histogram(&explicit_point(vec![1.0, 2.0], vec![3, 4, 5], 12)),
            ExplicitHistogram::Buckets(MetricHistogram::Explicit(vec![
                (1.0, 3),
                (2.0, 4),
                (2.0_f64.next_up(), 5),
            ]))
        );
        let extra_counts = explicit_point(vec![1.0, 2.0], vec![3, 4, 5, 999], 12);
        assert_eq!(
            explicit_histogram(&extra_counts),
            ExplicitHistogram::Invalid
        );
    }

    /// Scenario: Explicit histogram bounds are unordered, duplicated, and share a bound with the synthetic overflow bucket.
    /// Guarantees: Buckets are sorted and equal boundaries are coalesced using protocol-compatible count addition.
    #[test]
    fn sorts_and_coalesces_explicit_histogram_buckets() {
        let huge_bound = 1.0e300;
        let histogram = explicit_histogram(&explicit_point(
            vec![3.0, 1.0, 1.0, huge_bound],
            vec![1, 2, 3, 4, 5],
            15,
        ));

        assert_eq!(
            histogram,
            ExplicitHistogram::Buckets(MetricHistogram::Explicit(vec![
                (1.0, 5),
                (3.0, 1),
                (huge_bound, 4),
                (huge_bound.next_up(), 5),
            ]))
        );
    }

    /// Scenario: An explicit histogram's last bound is f64::MAX, leaving no finite value for the synthetic overflow bucket.
    /// Guarantees: The data point is rejected rather than encoding a non-finite bucket bound.
    #[test]
    fn rejects_explicit_histogram_with_max_bound() {
        let histogram = explicit_histogram(&explicit_point(vec![f64::MAX], vec![1, 2], 3));

        assert_eq!(histogram, ExplicitHistogram::Invalid);
    }

    /// Scenario: An explicit histogram includes a NaN bound among otherwise valid bounds.
    /// Guarantees: Malformed input is rejected before the quadratic insertion fallback can run.
    #[test]
    fn rejects_nan_explicit_histogram_bounds() {
        let point = explicit_point(vec![1.0, f64::NAN, 2.0], vec![1, 2, 3, 4], 10);

        assert_eq!(explicit_histogram(&point), ExplicitHistogram::Invalid);
    }

    /// Scenario: Explicit histogram bucket counts exceed the Geneva u32 representation.
    /// Guarantees: Every oversized bucket count is clamped to u32::MAX.
    #[test]
    fn clamps_explicit_histogram_bucket_counts() {
        let oversized = u64::from(u32::MAX) + 1;
        let histogram = explicit_histogram(&explicit_point(
            vec![1.0],
            vec![oversized, oversized],
            oversized * 2,
        ));

        assert_eq!(
            histogram,
            ExplicitHistogram::Buckets(MetricHistogram::Explicit(vec![
                (1.0, u32::MAX),
                (1.0_f64.next_up(), u32::MAX),
            ]))
        );
    }

    /// Scenario: Explicit histogram bucket counts sum to more than a u64 can represent.
    /// Guarantees: The data point is rejected rather than silently wrapping the running total.
    #[test]
    fn rejects_explicit_bucket_total_overflow() {
        let histogram = explicit_histogram(&explicit_point(vec![1.0], vec![u64::MAX, 1], u64::MAX));

        assert_eq!(histogram, ExplicitHistogram::Invalid);
    }

    /// Scenario: An explicit histogram data point contains no bucket counts.
    /// Guarantees: No histogram body is generated when OTLP supplies only scalar count and sum.
    #[test]
    fn omits_empty_explicit_histogram() {
        assert_eq!(
            explicit_histogram(&explicit_point(Vec::new(), Vec::new(), 0)),
            ExplicitHistogram::Empty
        );
    }

    /// Scenario: An explicit histogram data point has no configured bounds but reports a single bucket count.
    /// Guarantees: The lone count is treated as a synthetic single-bucket distribution rather than discarded.
    #[test]
    fn maps_unbounded_explicit_histogram_single_bucket() {
        assert_eq!(
            explicit_histogram(&explicit_point(Vec::new(), vec![7], 7)),
            ExplicitHistogram::Buckets(MetricHistogram::Explicit(vec![(1.0, 7)]))
        );
    }

    /// Scenario: An explicit histogram data point has no configured bounds but reports more than one bucket count.
    /// Guarantees: The trailing count is rejected rather than silently ignored.
    #[test]
    fn rejects_unbounded_explicit_histogram_trailing_counts() {
        assert_eq!(
            explicit_histogram(&explicit_point(Vec::new(), vec![7, 1], 8)),
            ExplicitHistogram::Invalid
        );
    }

    /// Scenario: Exponential histogram buckets contain zeros around populated buckets and a negative offset.
    /// Guarantees: Zero buckets are removed and populated bucket indexes include the OTLP offset.
    #[test]
    fn converts_sparse_exponential_buckets() {
        let buckets = exponential_buckets(-2, vec![0, 4, 0, 5]);

        assert_eq!(sparse_buckets(Some(&buckets)), Some(vec![(-1, 4), (1, 5)]));
        assert_eq!(sparse_buckets(None), Some(Vec::new()));
    }

    /// Scenario: Applying an OTLP bucket offset would exceed the i32 index range.
    /// Guarantees: The malformed exponential histogram range is rejected.
    #[test]
    fn rejects_exponential_bucket_index_overflow() {
        let buckets = exponential_buckets(i32::MAX, vec![0, 1]);

        assert_eq!(sparse_buckets(Some(&buckets)), None);
    }

    /// Scenario: Summing exponential histogram buckets exceeds u64.
    /// Guarantees: Bucket total overflow is reported instead of wrapping.
    #[test]
    fn rejects_exponential_bucket_total_overflow() {
        assert_eq!(bucket_sum(&[(0, u64::MAX), (1, 1)]), None);
    }

    /// Scenario: An exponential histogram is already within the Geneva bucket-count limit.
    /// Guarantees: Downscaling leaves its scale, indexes, and counts unchanged.
    #[test]
    fn leaves_bounded_exponential_histogram_unchanged() {
        let original = (0_i32..MAX_EXPONENTIAL_BUCKETS as i32)
            .map(|index| (index, 1))
            .collect::<Vec<_>>();
        let mut positive = original.clone();
        let mut negative = Vec::new();
        let mut scale = 10;

        downscale_if_required(&mut scale, &mut positive, &mut negative);

        assert_eq!(scale, 10);
        assert_eq!(positive, original);
        assert!(negative.is_empty());
    }

    /// Scenario: An exponential histogram exceeds the Geneva bucket-count limit by one bucket.
    /// Guarantees: Downscaling reduces the range, decrements scale, and preserves the total count.
    #[test]
    fn downscales_oversized_exponential_histogram() {
        let mut positive = (0_i32..=MAX_EXPONENTIAL_BUCKETS as i32)
            .map(|index| (index, 1))
            .collect::<Vec<_>>();
        let mut negative = Vec::new();
        let mut scale = 10;

        downscale_if_required(&mut scale, &mut positive, &mut negative);

        assert_eq!(scale, 9);
        assert!(positive.len() <= MAX_EXPONENTIAL_BUCKETS);
        assert_eq!(bucket_sum(&positive), Some(503));
        assert!(negative.is_empty());
    }

    /// Scenario: Adjacent negative and positive exponential bucket indexes are downscaled by two.
    /// Guarantees: Euclidean division combines each signed index pair into the correct destination bucket.
    #[test]
    fn downscales_negative_bucket_indexes_with_euclidean_division() {
        let mut buckets = vec![(-2, 1), (-1, 2), (0, 3), (1, 4)];

        downscale_buckets(&mut buckets, 1);

        assert_eq!(buckets, vec![(-1, 3), (0, 7)]);
    }

    /// Scenario: Downscaling combines bucket counts whose mathematical sum exceeds u64.
    /// Guarantees: The merged bucket count saturates instead of wrapping to a smaller value.
    #[test]
    fn saturates_combined_bucket_counts() {
        let mut buckets = vec![(0, u64::MAX), (1, 1)];

        downscale_buckets(&mut buckets, 1);

        assert_eq!(buckets, vec![(0, u64::MAX)]);
    }
}
