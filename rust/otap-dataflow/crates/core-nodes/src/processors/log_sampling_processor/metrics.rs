// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the log sampling processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Action taken by the log sampling processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum SamplingAction {
    /// Log records received by the processor.
    Consumed,
    /// Log records dropped by sampling.
    Dropped,
    /// An error occurred during processing.
    Error,
}

/// Attributes for log sampling metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SamplingAttributes {
    /// Action taken on the logs.
    pub action: SamplingAction,
}

/// Metrics for the log sampling processor.
#[metric_set(
    name = "processor.log_sampling.pdata",
    measurement_attributes = SamplingAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct LogSamplingMetrics {
    /// Log record counts, partitioned by `SamplingAction` (Consumed=received, Dropped=dropped, Error=error).
    #[metric(unit = "{log}")]
    pub log_signals: Counter<u64>,

    /// Errors encountered while filtering OTAP batches.
    #[metric(unit = "{error}")]
    pub filtering_errors: Counter<u64>,

    /// How many times we fail to reclaim the underlying filter
    /// buffer.
    #[metric(unit = "{error}")]
    pub filter_buffer_reclamation_failures: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A newly constructed LogSamplingMetrics has not recorded any measurements.
    /// Guarantees: All counters start at 0.
    #[test]
    fn test_metrics_default() {
        let m = LogSamplingMetrics::default();
        assert_eq!(m.log_signals.get(), 0);
        assert_eq!(m.filtering_errors.get(), 0);
        assert_eq!(m.filter_buffer_reclamation_failures.get(), 0);
    }

    /// Scenario: Measurements are recorded via increment or addition.
    /// Guarantees: The underlying counters reflect the accumulated totals.
    #[test]
    fn test_metrics_add() {
        let mut m = LogSamplingMetrics::default();
        m.log_signals.add(100);
        m.filtering_errors.inc();
        m.filter_buffer_reclamation_failures.inc();

        assert_eq!(m.log_signals.get(), 100);
        assert_eq!(m.filtering_errors.get(), 1);
        assert_eq!(m.filter_buffer_reclamation_failures.get(), 1);
    }
}
