// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the condense attributes processor.

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry_macros::metric_set;

/// Metrics automatically recorded by the condense attributes processor.
#[metric_set(name = "condense_attributes.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct CondenseAttributesMetrics {
    /// Wall-clock duration of the condense processing compute, in nanoseconds.
    #[metric(name = "process.duration", unit = "ns")]
    pub process_duration: Mmsc,
}
