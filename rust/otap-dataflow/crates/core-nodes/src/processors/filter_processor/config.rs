// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//!

use otap_df_pdata::otap::filter::{logs::LogFilter, metrics::MetricFilter, traces::TraceFilter};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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
