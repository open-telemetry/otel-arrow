// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Microsoft Common Schema processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Internal telemetry for the Microsoft Common Schema processor.
#[metric_set(name = "microsoft.common_schema.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub(super) struct MicrosoftCommonSchemaProcessorMetrics {
    /// Number of log records inspected by the processor.
    #[metric(unit = "{item}")]
    pub records_seen: Counter<u64>,
    /// Number of log records promoted as Microsoft Common Schema logs.
    #[metric(unit = "{item}")]
    pub records_promoted: Counter<u64>,
    /// Number of log records inspected but not promoted as Microsoft Common Schema.
    #[metric(unit = "{item}")]
    pub records_skipped_not_common_schema: Counter<u64>,
    /// Number of log batches that had at least one promoted record.
    #[metric(unit = "{batch}")]
    pub batches_promoted: Counter<u64>,
    /// Number of Arrow log batches forwarded without OTLP conversion because no
    /// `__csver__` attribute was present.
    #[metric(unit = "{batch}")]
    pub arrow_batches_skipped_no_csver: Counter<u64>,
}
