// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric equivalence checking.

use crate::proto::opentelemetry::metrics::v1::MetricsData;

/// Assert that two metric requests are semantically equivalent.
///
/// TODO: Implement data point flattening similar to logs
pub fn assert_metrics_equivalent(_expected: &[MetricsData], _actual: &[MetricsData]) {
    unimplemented!("Metric equivalence checking not yet implemented")
}
