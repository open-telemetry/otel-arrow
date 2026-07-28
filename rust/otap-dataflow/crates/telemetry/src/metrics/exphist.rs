// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoding primitives that project exponential-histogram aggregations into
//! OTLP `ExponentialHistogramDataPoint`s.
//!
//! These primitives are the foundation for encoding ITS histogram instruments
//! consistently: both the pre-aggregated min/max/sum/count summary (the
//! "basic" level) and full [`otap_df_expohisto::HistogramNN`] aggregations (the
//! "normal" and "detailed" levels) are projected onto the same OTLP
//! exponential-histogram point type.
//!
//! All points are emitted with delta temporality by the caller; these
//! functions only build the per-point payload.

use crate::instrument::{Distribution, Mmsc};
use otap_df_expohisto::{HistogramView, MIN_SCALE, Scale, table_scale};
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

/// Chooses the bucket that encloses the observed range `[lo, hi]`, at the
/// finest scale where both endpoints land in the same bucket.
///
/// The basic tier records no bucket structure, so any projection onto buckets
/// is a summary rather than a reconstruction. Searching downward from the
/// finest scale keeps that summary as tight as the data allows: a population
/// whose values all sit within one bucket width is described exactly, while a
/// wide population widens the bucket instead of inventing a shape.
///
/// `MIN_SCALE` splits the f64 range into just two buckets -- `(0, 1]` and
/// `(1, MAX)` -- so a range straddling 1.0 has no single enclosing bucket. In
/// that case the search bottoms out and the bucket holding `hi` is returned.
/// The point's exact `min` still reports the true lower end.
fn enclosing_bucket(lo: f64, hi: f64) -> (Scale, i32) {
    let mut scale = Scale::new(table_scale()).expect("table scale is a valid scale");
    loop {
        let hi_index = scale.map_to_index(hi);
        if scale.map_to_index(lo) == hi_index {
            return (scale, hi_index);
        }
        let next = scale.scale() - 1;
        if next < MIN_SCALE {
            return (scale, hi_index);
        }
        scale = Scale::new(next).expect("scale above MIN_SCALE is valid");
    }
}

/// Projects a pre-aggregated min/max/sum/count summary onto an OTLP
/// `ExponentialHistogramDataPoint`.
///
/// This is the "basic" tier: the instrument keeps no buckets, only exact
/// `count`, `min`, `max`, and `sum`. OTLP nonetheless requires `count` to equal
/// `zero_count` plus the sum of all bucket counts, so the observations cannot
/// simply be omitted. They are placed in the single bucket returned by
/// [`enclosing_bucket`], which spans the observed range at the finest scale
/// that still contains it.
///
/// The scalar fields therefore stay exact and the point stays well formed; the
/// bucket conveys only the range, which is all this tier knows.
///
/// `zero_count` is always 0. This tier does not track exact zeros: a zero is an
/// ordinary observation that lowers `min`, and is summarized into the enclosing
/// bucket with everything else. Because a bucket cannot have a lower boundary
/// of zero, a population containing zeros is enclosed from the smallest normal
/// f64 upward; the exact `min` field still reports the true lower end.
///
/// `sum` is populated only for non-negative populations, for which the OTLP sum
/// is well defined. `Mmsc::record` debug-asserts non-negative observations, so
/// a negative `min` only arises in a release build with a misbehaving call
/// site.
pub(crate) fn mmsc_exponential_histogram_data_point(
    mmsc: &Mmsc,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    let mut builder = ExponentialHistogramDataPoint::build()
        .attributes(attributes.to_vec())
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(mmsc.count)
        .zero_count(0u64);
    if mmsc.count > 0 {
        builder = builder.min(mmsc.min).max(mmsc.max);
        if mmsc.min >= 0.0 {
            builder = builder.sum(mmsc.sum);
        }
        let lo = if mmsc.min > 0.0 {
            mmsc.min
        } else {
            f64::MIN_POSITIVE
        };
        let (scale, index) = enclosing_bucket(lo, mmsc.max.max(lo));
        builder = builder
            .scale(scale.scale())
            .positive(Buckets::new(index, vec![mmsc.count]));
    }
    builder.finish()
}

/// Projects a [`Distribution`] onto an OTLP `ExponentialHistogramDataPoint`,
/// dispatching each tier onto the matching primitive.
pub(crate) fn distribution_exponential_histogram_data_point(
    distribution: &Distribution,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
    attributes: &[KeyValue],
) -> ExponentialHistogramDataPoint {
    match distribution {
        Distribution::Basic(mmsc) => mmsc_exponential_histogram_data_point(
            mmsc,
            start_time_unix_nano,
            time_unix_nano,
            attributes,
        ),
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

    /// Scenario: A pre-aggregated min/max/sum/count summary that also observed
    /// an exact zero is projected onto the basic form.
    /// Guarantees: The point preserves count, min, max, and sum exactly, and
    /// satisfies OTLP's requirement that count equal zero_count plus the sum of
    /// all bucket counts, so an Mmsc encodes as a well formed exponential
    /// histogram.
    #[test]
    fn mmsc_projects_to_a_range_bucket() {
        let mut mmsc = Mmsc::default();
        for v in [0.0_f64, 2.0, 9.0, 9.0] {
            mmsc.record(v);
        }

        let point = mmsc_exponential_histogram_data_point(&mmsc, 5, 7, &[]);

        assert_eq!(point.count, 4);
        assert_eq!(point.min, Some(0.0));
        assert_eq!(point.max, Some(9.0));
        assert_eq!(point.sum, Some(20.0));
        assert!(point.negative.is_none());

        let positive = point.positive.expect("observations are bucketed");
        let bucketed: u64 = positive.bucket_counts.iter().sum();
        assert_eq!(point.zero_count + bucketed, point.count);
    }

    /// Scenario: An Mmsc population observes exact zeros alongside positive
    /// values and is projected onto the basic form.
    /// Guarantees: The basic tier reports zero_count as 0 and folds the zeros
    /// into the enclosing bucket rather than tracking them separately, so every
    /// observation is accounted for by the bucket range alone.
    #[test]
    fn mmsc_folds_zeros_into_the_range_bucket() {
        let mut mmsc = Mmsc::default();
        mmsc.record(0.0);
        mmsc.record(0.0);
        mmsc.record(4.0);

        let point = mmsc_exponential_histogram_data_point(&mmsc, 0, 0, &[]);

        assert_eq!(point.count, 3);
        assert_eq!(point.zero_count, 0);
        assert_eq!(point.min, Some(0.0));
        let positive = point.positive.expect("observations are bucketed");
        assert_eq!(positive.bucket_counts, vec![3]);
    }

    /// Scenario: An Mmsc population whose values all fall within a narrow range
    /// is projected onto the basic form.
    /// Guarantees: The enclosing bucket is chosen at a scale fine enough that
    /// both min and max land inside it, so the summary bucket actually spans
    /// the observed range rather than defaulting to the widest scale.
    #[test]
    fn mmsc_range_bucket_encloses_min_and_max() {
        let mut mmsc = Mmsc::default();
        for v in [8.0_f64, 8.1, 8.2] {
            mmsc.record(v);
        }

        let point = mmsc_exponential_histogram_data_point(&mmsc, 0, 0, &[]);

        let positive = point.positive.expect("observations are bucketed");
        assert_eq!(positive.bucket_counts, vec![3]);
        let scale = Scale::new(point.scale).expect("emitted scale is valid");
        assert_eq!(scale.map_to_index(8.0), positive.offset);
        assert_eq!(scale.map_to_index(8.2), positive.offset);
        assert_eq!(point.zero_count, 0);
    }

    /// Scenario: An empty Mmsc, with no observations at all, is projected onto
    /// the basic form.
    /// Guarantees: No bucket range and no scalar fields are emitted, so an
    /// untouched instrument does not fabricate a bucket around its meaningless
    /// zero-valued min and max.
    #[test]
    fn mmsc_empty_population_emits_no_buckets() {
        let mmsc = Mmsc::default();

        let point = mmsc_exponential_histogram_data_point(&mmsc, 0, 0, &[]);

        assert_eq!(point.count, 0);
        assert_eq!(point.zero_count, 0);
        assert!(point.positive.is_none());
        assert!(point.negative.is_none());
        assert!(point.sum.is_none());
    }

    /// Scenario: An Mmsc population consisting only of exact zeros is projected
    /// onto the basic form.
    /// Guarantees: A bucket carrying every observation is still emitted, so the
    /// OTLP count invariant holds even though no bucket can have a lower
    /// boundary of zero; min and max continue to report the true values.
    #[test]
    fn mmsc_all_zero_population_still_satisfies_the_count_invariant() {
        let mut mmsc = Mmsc::default();
        mmsc.record(0.0);
        mmsc.record(0.0);

        let point = mmsc_exponential_histogram_data_point(&mmsc, 0, 0, &[]);

        assert_eq!(point.count, 2);
        assert_eq!(point.zero_count, 0);
        assert_eq!(point.min, Some(0.0));
        assert_eq!(point.max, Some(0.0));
        let positive = point.positive.expect("observations are bucketed");
        let bucketed: u64 = positive.bucket_counts.iter().sum();
        assert_eq!(point.zero_count + bucketed, point.count);
    }

    /// Scenario: An Mmsc population spans a range too wide for any single
    /// bucket, straddling 1.0 even at the coarsest scale.
    /// Guarantees: The projection still bottoms out at MIN_SCALE and emits one
    /// bucket carrying every observation, so the OTLP count invariant holds
    /// even when the range cannot be enclosed.
    #[test]
    fn mmsc_range_wider_than_any_bucket_falls_back_to_min_scale() {
        let mut mmsc = Mmsc::default();
        mmsc.record(1e-300);
        mmsc.record(1e300);

        let point = mmsc_exponential_histogram_data_point(&mmsc, 0, 0, &[]);

        assert_eq!(point.scale, MIN_SCALE);
        let positive = point.positive.expect("observations are bucketed");
        assert_eq!(positive.bucket_counts, vec![2]);
        assert_eq!(point.zero_count + 2, point.count);
    }
}
