// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding primitives that project exponential-histogram aggregations into
//! OTLP `ExponentialHistogramDataPoint`s.
//!
//! These primitives encode the normal and detailed histogram tiers, which
//! retain real exponential bucket counts. The basic MMSC tier is encoded as an
//! explicit-boundary `HistogramDataPoint` instead, because it retains only
//! min, max, sum, and count.
//!
//! All points are emitted with delta temporality by the caller; these
//! functions only build the per-point payload.

use crate::instrument::Distribution;
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
/// `zero_count` is recovered as `count - sum(positive bucket counts)`. That
/// subtraction is a byproduct of the bucket scan this encoder performs
/// anyway, which is why [`HistogramView::scan_buckets`] hands back both the
/// bucket counts and the totals in one pass.
///
/// `sum`, `min`, and `max` are populated only when at least one observation
/// has been recorded. The sum is always OTLP-valid here because the source
/// histogram rejects negative values.
pub(crate) fn exponential_histogram_data_point<const N: usize>(
    view: &HistogramView<'_, N>,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    let stats = view.stats();
    let positive = view.positive();

    let mut bucket_counts: Vec<u64> = Vec::with_capacity(positive.len() as usize);
    // Zeros contribute to `count` but never to a bucket (see `record_incr`).
    let totals = view.scan_buckets(|count| bucket_counts.push(count));

    let mut builder = ExponentialHistogramDataPoint::build()
        .attributes(attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(stats.count)
        .scale(view.scale())
        .zero_count(totals.zero_count);

    if !bucket_counts.is_empty() {
        builder = builder.positive(Buckets::new(positive.offset(), bucket_counts));
    }
    if stats.count > 0 {
        builder = builder.sum(stats.sum).min(stats.min).max(stats.max);
    }

    builder.finish()
}

/// Projects a [`Distribution`] onto an OTLP `ExponentialHistogramDataPoint`,
/// dispatching each bucketed tier onto the matching primitive.
///
/// The caller validates that this function is used only for a true
/// exponential-histogram instrument.
pub(crate) fn distribution_exponential_histogram_data_point(
    distribution: &Distribution,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    match distribution {
        Distribution::Basic(_) => {
            unreachable!("basic MMSC distributions use explicit-boundary histograms")
        }
        Distribution::Normal(hist) => exponential_histogram_data_point(
            &hist.view(),
            start_time_unix_nano,
            time_unix_nano,
            attributes,
        ),
        Distribution::Detailed(hist) => exponential_histogram_data_point(
            &hist.view(),
            start_time_unix_nano,
            time_unix_nano,
            attributes,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_expohisto::HistogramNN;

    /// Scenario: A positive-only histogram records several positive values and
    /// is projected onto an OTLP exponential-histogram point.
    /// Guarantees: The point preserves the exact count, sum, min, and max, uses
    /// the view's scale, carries a populated positive bucket range whose counts
    /// sum to the number of bucketed observations, and reports no zeros.
    #[test]
    fn projects_positive_observations_into_buckets() {
        let mut hist: HistogramNN<16> = HistogramNN::new();
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
        let mut hist: HistogramNN<16> = HistogramNN::new();
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
        let hist: HistogramNN<16> = HistogramNN::new();
        let view = hist.view();

        let point = exponential_histogram_data_point(&view, 0, 0, &[]);

        assert_eq!(point.count, 0);
        assert_eq!(point.zero_count, 0);
        assert!(point.positive.is_none());
        assert!(point.sum.is_none());
    }
}
