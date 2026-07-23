// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for `#[metric_set]` fields that declare an
//! exponential-histogram distribution tier by field type.

#![allow(missing_docs)]

use otap_df_telemetry::descriptor::{Instrument, MetricValueType, Temporality};
use otap_df_telemetry::instrument::{DistributionValue, HistogramNormal};
use otap_df_telemetry::metrics::{MetricSetHandler, MetricValue};
use otap_df_telemetry_macros::metric_set;

#[metric_set(name = "test.distribution_set")]
#[derive(Debug, Default, Clone)]
struct LatencyMetrics {
    /// Request latency recorded as an exponential-histogram distribution.
    #[metric(unit = "ms")]
    request_latency: HistogramNormal,
}

// Scenario: A `#[metric_set]` struct declares a `HistogramNormal` field, records
// observations, and is snapshotted, cleared, and checked for flush readiness.
// Guarantees: The macro maps the field to an `ExponentialHistogram` delta F64
// instrument, `snapshot_values` yields a live `Distribution`, `needs_flush`
// tracks whether observations exist, and `clear_values` resets the tier.
#[test]
fn histogram_normal_field_wires_through_metric_set_macro() {
    let mut metrics = LatencyMetrics::default();

    let field = &metrics.descriptor().metrics[0];
    assert_eq!(field.name, "request.latency");
    assert_eq!(field.instrument, Instrument::ExponentialHistogram);
    assert_eq!(field.temporality, Some(Temporality::Delta));
    assert_eq!(field.value_type, MetricValueType::F64);

    assert!(
        !metrics.needs_flush(),
        "empty distribution should not flush"
    );

    metrics.request_latency.record(1.0);
    metrics.request_latency.record(4.0);
    assert!(metrics.needs_flush(), "recorded distribution should flush");

    let values = metrics.snapshot_values();
    let [MetricValue::Distribution(value)] = values.as_slice() else {
        panic!("expected a single distribution value")
    };
    let DistributionValue::Live(distribution) = value.as_ref() else {
        panic!("snapshot should carry a live distribution")
    };
    let (count, sum, min, max) = distribution.summary();
    assert_eq!(count, 2);
    assert_eq!(sum, 5.0);
    assert_eq!(min, 1.0);
    assert_eq!(max, 4.0);

    metrics.clear_values();
    assert!(
        !metrics.needs_flush(),
        "cleared distribution should not flush"
    );
}
