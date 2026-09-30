// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP histogram conversion helpers.

use otel_arrow_dfe_pdata_views::views::metrics::{BucketsView, HistogramDataPointView};

use super::super::encoder::MetricHistogram;

pub(super) const MIN_EXPONENTIAL_SCALE: i32 = -11;
pub(super) const MAX_EXPONENTIAL_SCALE: i32 = 20;
const MAX_EXPONENTIAL_BUCKETS: usize = 502;

pub(super) fn valid_explicit_histogram<P>(point: &P) -> bool
where
    P: HistogramDataPointView,
{
    let mut bound_count = 0;
    for bound in point.explicit_bounds() {
        if bound.is_nan() {
            return false;
        }
        bound_count += 1;
    }
    bound_count == 0 || point.bucket_counts().count() > bound_count
}

pub(super) fn explicit_histogram<P>(point: &P) -> Option<MetricHistogram>
where
    P: HistogramDataPointView,
{
    let mut counts = point.bucket_counts();
    let first_count = counts.next()?;
    let bounds = point.explicit_bounds();
    let mut buckets = Vec::with_capacity(bounds.size_hint().0 + 1);
    let mut current_count = Some(first_count);
    let mut overflow_bound = 1.0;
    for bound in bounds {
        if bound.is_nan() {
            return None;
        }
        let count = current_count.take()?;
        buckets.push((bound, clamp_bucket_count(count)));
        overflow_bound = bound + 1.0;
        current_count = counts.next();
    }
    let overflow_count = current_count?;
    buckets.push((overflow_bound, clamp_bucket_count(overflow_count)));
    normalize_explicit_buckets(&mut buckets);
    Some(MetricHistogram::Explicit(buckets))
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

    fn valid_explicit_histogram(point: &HistogramDataPoint) -> bool {
        super::valid_explicit_histogram(&ObjHistogramDataPoint::new(point))
    }

    fn explicit_histogram(point: &HistogramDataPoint) -> Option<MetricHistogram> {
        super::explicit_histogram(&ObjHistogramDataPoint::new(point))
    }

    fn sparse_buckets(
        buckets: Option<&exponential_histogram_data_point::Buckets>,
    ) -> Option<Vec<(i32, u64)>> {
        super::sparse_buckets(buckets.map(ObjBuckets::new))
    }

    fn explicit_point(bounds: Vec<f64>, counts: Vec<u64>) -> HistogramDataPoint {
        HistogramDataPoint {
            explicit_bounds: bounds,
            bucket_counts: counts,
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

    /// Scenario: Explicit histogram bucket counts omit, include, or exceed the required overflow bucket.
    /// Guarantees: Missing overflow counts are rejected, and counts beyond the first overflow bucket are ignored.
    #[test]
    fn validates_explicit_histogram_bucket_shape() {
        assert!(!valid_explicit_histogram(&explicit_point(
            vec![1.0, 2.0],
            vec![3, 4],
        )));
        assert!(valid_explicit_histogram(&explicit_point(
            vec![1.0, 2.0],
            vec![3, 4, 5],
        )));
        let extra_counts = explicit_point(vec![1.0, 2.0], vec![3, 4, 5, 999]);
        assert!(valid_explicit_histogram(&extra_counts));
        assert_eq!(
            explicit_histogram(&extra_counts),
            Some(MetricHistogram::Explicit(vec![
                (1.0, 3),
                (2.0, 4),
                (3.0, 5),
            ]))
        );
    }

    /// Scenario: An explicit histogram contains finite bounds and an overflow bucket.
    /// Guarantees: Bounds and counts map in order and the overflow bucket receives a synthetic final bound.
    #[test]
    fn maps_explicit_histogram_with_overflow_bucket() {
        let histogram = explicit_histogram(&explicit_point(vec![1.0, 2.0], vec![3, 4, 5]));

        assert_eq!(
            histogram,
            Some(MetricHistogram::Explicit(vec![
                (1.0, 3),
                (2.0, 4),
                (3.0, 5),
            ]))
        );
    }

    /// Scenario: Explicit histogram bounds are unordered, duplicated, and share a bound with the synthetic overflow bucket.
    /// Guarantees: Buckets are sorted and equal boundaries are coalesced using protocol-compatible count addition.
    #[test]
    fn sorts_and_coalesces_explicit_histogram_buckets() {
        let histogram = explicit_histogram(&explicit_point(
            vec![3.0, 1.0, 1.0, f64::MAX],
            vec![1, 2, 3, 4, 5],
        ));

        assert_eq!(
            histogram,
            Some(MetricHistogram::Explicit(vec![
                (1.0, 5),
                (3.0, 1),
                (f64::MAX, 9),
            ]))
        );
    }

    /// Scenario: An explicit histogram includes a NaN bound among otherwise valid bounds.
    /// Guarantees: Malformed input is rejected before the quadratic insertion fallback can run.
    #[test]
    fn rejects_nan_explicit_histogram_bounds() {
        let point = explicit_point(vec![1.0, f64::NAN, 2.0], vec![1, 2, 3, 4]);

        assert!(!valid_explicit_histogram(&point));
        assert_eq!(explicit_histogram(&point), None);
    }

    /// Scenario: Explicit histogram bucket counts exceed the Geneva u32 representation.
    /// Guarantees: Every oversized bucket count is clamped to u32::MAX.
    #[test]
    fn clamps_explicit_histogram_bucket_counts() {
        let oversized = u64::from(u32::MAX) + 1;
        let histogram = explicit_histogram(&explicit_point(vec![1.0], vec![oversized, u64::MAX]));

        assert_eq!(
            histogram,
            Some(MetricHistogram::Explicit(vec![
                (1.0, u32::MAX),
                (2.0, u32::MAX),
            ]))
        );
    }

    /// Scenario: An explicit histogram data point contains no bucket counts.
    /// Guarantees: No histogram body is generated when OTLP supplies only scalar count and sum.
    #[test]
    fn omits_empty_explicit_histogram() {
        assert_eq!(
            explicit_histogram(&explicit_point(Vec::new(), Vec::new())),
            None
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
