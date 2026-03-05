// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the KQL recordset processor.

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry_macros::metric_set;

/// Metrics automatically recorded by the KQL recordset processor.
#[metric_set(name = "recordset_kql.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct RecordsetKqlMetrics {
    /// Wall-clock duration of the KQL processing compute, in nanoseconds.
    #[metric(name = "process.duration", unit = "ns")]
    pub process_duration: Mmsc,
}
