// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the log sampling processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the log sampling processor.
#[metric_set(name = "log_sampling.processor.pdata")]
#[derive(Debug, Default, Clone)]
pub struct LogSamplingMetrics {
    /// Total log records received by the processor.
    #[metric(unit = "{log}")]
    pub log_signals_consumed: Counter<u64>,

    /// Log records dropped by sampling.
    #[metric(unit = "{log}")]
    pub log_signals_dropped: Counter<u64>,

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

    #[test]
    fn test_metrics_default() {
        let m = LogSamplingMetrics::default();
        assert_eq!(m.log_signals_consumed.get(), 0);
        assert_eq!(m.log_signals_dropped.get(), 0);
        assert_eq!(m.filtering_errors.get(), 0);
    }

    #[test]
    fn test_metrics_add() {
        let mut m = LogSamplingMetrics::default();
        m.log_signals_consumed.add(100);
        m.log_signals_dropped.add(90);
        m.filtering_errors.inc();

        assert_eq!(m.log_signals_consumed.get(), 100);
        assert_eq!(m.log_signals_dropped.get(), 90);
        assert_eq!(m.filtering_errors.get(), 1);
    }
}
