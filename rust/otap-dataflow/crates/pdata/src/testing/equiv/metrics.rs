// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric equivalence checking.

use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;

/// Assert that two metric requests are semantically equivalent.
///
/// TODO: Implement data point flattening similar to logs
pub fn assert_metrics_equivalent(
    _expected: &ExportMetricsServiceRequest,
    _actual: &ExportMetricsServiceRequest,
) {
    unimplemented!("Metric equivalence checking not yet implemented")
}
