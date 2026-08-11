// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry metrics for the log sampling processor.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::{attribute_set, metric_set};

use otap_df_config::SignalType;

#[attribute_set(item, registration)]
#[derive(Debug, Clone, Copy)]
pub struct LogSamplingRegistrationAttributes {
    pub signal: SignalType,
}

/// Metrics for the log sampling processor.
#[metric_set(
    name = "processor.log_sampling",
    registration_attributes = LogSamplingRegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct LogSamplingMetrics {
    /// Log records dropped by sampling.
    #[metric(name = "dropped.items", unit = "{item}")]
    pub dropped_items: Counter<u64>,

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
        assert_eq!(m.dropped_items.get(), 0);
        assert_eq!(m.filtering_errors.get(), 0);
        assert_eq!(m.filter_buffer_reclamation_failures.get(), 0);
    }

    /// Scenario: Measurements are recorded via increment or addition.
    /// Guarantees: The underlying counters reflect the accumulated totals.
    #[test]
    fn test_metrics_add() {
        let mut m = LogSamplingMetrics::default();
        m.dropped_items.add(100);
        m.filtering_errors.inc();
        m.filter_buffer_reclamation_failures.inc();

        assert_eq!(m.dropped_items.get(), 100);
        assert_eq!(m.filtering_errors.get(), 1);
        assert_eq!(m.filter_buffer_reclamation_failures.get(), 1);
    }
}
