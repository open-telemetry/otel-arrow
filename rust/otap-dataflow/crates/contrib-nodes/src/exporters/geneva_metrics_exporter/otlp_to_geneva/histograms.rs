// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP histogram conversion helpers.

use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    HistogramDataPoint, exponential_histogram_data_point,
};

use super::super::encoder::MetricHistogram;

pub(super) const MIN_EXPONENTIAL_SCALE: i32 = -11;
pub(super) const MAX_EXPONENTIAL_SCALE: i32 = 20;
const MAX_EXPONENTIAL_BUCKETS: usize = 502;

pub(super) fn valid_explicit_histogram(point: &HistogramDataPoint) -> bool {
    point.explicit_bounds.is_empty() || point.bucket_counts.len() > point.explicit_bounds.len()
}

pub(super) fn explicit_histogram(point: &HistogramDataPoint) -> Option<MetricHistogram> {
    if point.bucket_counts.is_empty() {
        return None;
    }
    let mut buckets = point
        .explicit_bounds
        .iter()
        .zip(&point.bucket_counts)
        .map(|(bound, count)| (*bound, (*count).min(u64::from(u32::MAX)) as u32))
        .collect::<Vec<_>>();
    let overflow_count = point.bucket_counts[point.explicit_bounds.len()];
    let overflow_bound = point.explicit_bounds.last().copied().unwrap_or(0.0) + 1.0;
    buckets.push((
        overflow_bound,
        overflow_count.min(u64::from(u32::MAX)) as u32,
    ));
    Some(MetricHistogram::Explicit(buckets))
}

pub(super) fn sparse_buckets(
    buckets: Option<&exponential_histogram_data_point::Buckets>,
) -> Option<Vec<(i32, u64)>> {
    let Some(buckets) = buckets else {
        return Some(Vec::new());
    };
    buckets
        .bucket_counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count != 0)
        .map(|(index, count)| {
            let index = i32::try_from(index).ok()?;
            Some((buckets.offset.checked_add(index)?, *count))
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
    use super::*;

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

    /// Scenario: Explicit histogram bucket counts omit or include the required overflow bucket.
    /// Guarantees: Only shapes with one more count than bound are accepted.
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
    /// Guarantees: No histogram body is generated for an empty distribution.
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
