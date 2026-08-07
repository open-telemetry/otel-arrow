// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//!

use otap_df_pdata::otap::filter::{logs::LogFilter, metrics::MetricFilter, traces::TraceFilter};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_metric_filter")]
    metrics: MetricFilter,
    #[serde(default = "default_log_filter")]
    logs: LogFilter,
    #[serde(default = "default_trace_filter")]
    traces: TraceFilter,
}

/// create empty log filter as default value
const fn default_log_filter() -> LogFilter {
    LogFilter::new(None, None, vec![])
}

/// create empty metric filter as default value
const fn default_metric_filter() -> MetricFilter {
    MetricFilter::new(None, None)
}

/// create empty trace filter as default value
const fn default_trace_filter() -> TraceFilter {
    TraceFilter::new(None, None)
}

impl Config {
    pub const fn new(logs: LogFilter, traces: TraceFilter) -> Self {
        Self {
            metrics: MetricFilter::new(None, None),
            logs,
            traces,
        }
    }

    pub const fn new_with_metrics(
        metrics: MetricFilter,
        logs: LogFilter,
        traces: TraceFilter,
    ) -> Self {
        Self {
            metrics,
            logs,
            traces,
        }
    }

    #[must_use]
    pub const fn metric_filters(&self) -> &MetricFilter {
        &self.metrics
    }

    #[must_use]
    pub const fn log_filters(&self) -> &LogFilter {
        &self.logs
    }

    #[must_use]
    pub const fn trace_filters(&self) -> &TraceFilter {
        &self.traces
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use serde_json::json;

    /// Scenario: filter processor configuration contains an unknown top-level field.
    /// Guarantees: typed startup validation rejects the field and identifies it by name.
    #[test]
    fn rejects_unknown_top_level_field() {
        let error =
            otap_df_config::validation::validate_typed_config::<Config>(&json!({"__bogus__": 1}))
                .expect_err("unknown filter processor fields must be rejected");

        assert!(
            error.to_string().contains("unknown field `__bogus__`"),
            "unexpected validation error: {error}"
        );
    }
}
