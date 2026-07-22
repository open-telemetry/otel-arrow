// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding primitives that project exponential-histogram aggregations into
//! OTLP `ExponentialHistogramDataPoint`s.
//!
//! These primitives are the foundation for encoding ITS histogram instruments
//! consistently: both the pre-aggregated min/max/sum/count summary (the
//! "basic" tier) and full [`otap_df_expohisto::Histogram`] aggregations (the
//! "normal" and "detailed" tiers) are projected onto the same OTLP
//! exponential-histogram point type.
//!
//! All points are emitted with delta temporality by the caller; these
//! functions only build the per-point payload.

use crate::instrument::MmscSnapshot;
use otap_df_expohisto::HistogramView;
use otap_df_pdata::proto::opentelemetry::common::v1::KeyValue;
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    ExponentialHistogramDataPoint, exponential_histogram_data_point::Buckets,
};

/// Projects an exponential-histogram view onto an OTLP
/// `ExponentialHistogramDataPoint`.
///
/// The positive-only [`HistogramView`] maps directly onto OTLP's positive
/// bucket range at the view's current `scale`. Because the histogram counts
/// exact zeros in its total but never places them in a bucket, the OTLP
/// `zero_count` is recovered as `count - sum(positive bucket counts)`.
///
/// `sum`, `min`, and `max` are populated only when at least one observation
/// has been recorded. The sum is always OTLP-valid here because the source
/// histogram rejects negative values.
// Wired into `encode_metric` once the histogram-carrying `MetricValue`
// variant lands; kept standalone and tested in this staged step.
#[allow(dead_code)]
pub(crate) fn exponential_histogram_data_point<const N: usize>(
    view: &HistogramView<'_, N>,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    let stats = view.stats();
    let positive = view.positive();

    let bucket_counts: Vec<u64> = positive.iter().collect();
    let positive_total: u64 = bucket_counts.iter().sum();
    // Zeros contribute to `count` but never to a bucket (see `record_incr`).
    let zero_count = stats.count.saturating_sub(positive_total);

    let mut builder = ExponentialHistogramDataPoint::build()
        .attributes(attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(stats.count)
        .scale(view.scale())
        .zero_count(zero_count);

    if !bucket_counts.is_empty() {
        builder = builder.positive(Buckets::new(positive.offset(), bucket_counts));
    }
    if stats.count > 0 {
        builder = builder.sum(stats.sum).min(stats.min).max(stats.max);
    }

    builder.finish()
}

/// Projects a pre-aggregated min/max/sum/count summary onto a bucketless OTLP
/// `ExponentialHistogramDataPoint`.
///
/// This is the "basic" tier: it carries no buckets (`scale` 0, empty positive
/// and negative ranges, zero `zero_count`) and preserves the exact `count`,
/// `min`, and `max`. `sum` is populated only for non-negative populations, for
/// which the OTLP sum is well defined.
pub(crate) fn mmsc_exponential_histogram_data_point(
    snapshot: &MmscSnapshot,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    let mut builder = ExponentialHistogramDataPoint::build()
        .attributes(attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(snapshot.count)
        .scale(0)
        .zero_count(0u64)
        .min(snapshot.min)
        .max(snapshot.max);
    if snapshot.min >= 0.0 {
        builder = builder.sum(snapshot.sum);
    }
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_expohisto::Histogram;

    /// Scenario: A positive-only histogram records several positive values and
    /// is projected onto an OTLP exponential-histogram point.
    /// Guarantees: The point preserves the exact count, sum, min, and max, uses
    /// the view's scale, carries a populated positive bucket range whose counts
    /// sum to the number of bucketed observations, and reports no zeros.
    #[test]
    fn projects_positive_observations_into_buckets() {
        let mut hist: Histogram<16> = Histogram::new();
        for v in [1.5_f64, 2.7, 4.0, 100.0] {
            hist.update(v).expect("positive value is recordable");
        }
        let view = hist.view();

        let point = exponential_histogram_data_point(&view, 10, 20, &[]);

        assert_eq!(point.count, 4);
        assert_eq!(point.scale, view.scale());
        assert_eq!(point.start_time_unix_nano, 10);
        assert_eq!(point.time_unix_nano, 20);
        assert_eq!(point.zero_count, 0);
        assert_eq!(point.min, Some(1.5));
        assert_eq!(point.max, Some(100.0));
        let sum = point.sum.expect("non-negative population has a sum");
        assert!((sum - 108.2).abs() < 1e-9);

        let positive = point.positive.expect("bucketed observations present");
        assert_eq!(positive.offset, view.positive().offset());
        let bucketed: u64 = positive.bucket_counts.iter().sum();
        assert_eq!(bucketed, 4);
    }

    /// Scenario: A histogram records both exact zeros and positive values.
    /// Guarantees: Zeros are reflected in the total count and recovered as the
    /// OTLP `zero_count`, while positive observations remain in the bucket
    /// range, so `zero_count + sum(bucket_counts) == count`.
    #[test]
    fn recovers_zero_count_from_total() {
        let mut hist: Histogram<16> = Histogram::new();
        hist.update(0.0).expect("zero is recordable");
        hist.update(0.0).expect("zero is recordable");
        hist.update(3.0).expect("positive value is recordable");
        let view = hist.view();

        let point = exponential_histogram_data_point(&view, 0, 0, &[]);

        assert_eq!(point.count, 3);
        assert_eq!(point.zero_count, 2);
        let bucketed: u64 = point
            .positive
            .as_ref()
            .map(|b| b.bucket_counts.iter().sum())
            .unwrap_or(0);
        assert_eq!(point.zero_count + bucketed, point.count);
    }

    /// Scenario: An untouched histogram is projected onto an OTLP point.
    /// Guarantees: The empty histogram yields a zero-count point with no
    /// positive buckets and no sum, so downstream consumers can drop it.
    #[test]
    fn empty_histogram_yields_empty_point() {
        let hist: Histogram<16> = Histogram::new();
        let view = hist.view();

        let point = exponential_histogram_data_point(&view, 0, 0, &[]);

        assert_eq!(point.count, 0);
        assert_eq!(point.zero_count, 0);
        assert!(point.positive.is_none());
        assert!(point.sum.is_none());
    }

    /// Scenario: A pre-aggregated min/max/sum/count summary is projected onto
    /// the bucketless "basic" exponential-histogram form.
    /// Guarantees: The point preserves count, min, max, and sum, uses scale 0,
    /// and carries neither positive nor negative buckets.
    #[test]
    fn mmsc_projects_to_bucketless_point() {
        let snapshot = MmscSnapshot {
            min: 2.0,
            max: 9.0,
            sum: 20.0,
            count: 4,
        };

        let point = mmsc_exponential_histogram_data_point(&snapshot, 5, 7, &[]);

        assert_eq!(point.count, 4);
        assert_eq!(point.scale, 0);
        assert_eq!(point.zero_count, 0);
        assert_eq!(point.min, Some(2.0));
        assert_eq!(point.max, Some(9.0));
        assert_eq!(point.sum, Some(20.0));
        assert!(point.positive.is_none());
        assert!(point.negative.is_none());
    }
}
