// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//!

use otap_df_pdata::otap::filter::{logs::LogFilter, traces::TraceFilter};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ToDo: add metrics
    #[serde(default = "default_log_filter")]
    logs: LogFilter,
    #[serde(default = "default_trace_filter")]
    traces: TraceFilter,
}

/// create empty log filter as default value
fn default_log_filter() -> LogFilter {
    LogFilter::new(None, None, vec![])
}

/// create empty trace filter as default value
fn default_trace_filter() -> TraceFilter {
    TraceFilter::new(None, None)
}

impl Config {
    pub fn new(logs: LogFilter, traces: TraceFilter) -> Self {
        Self { logs, traces }
    }

    #[must_use]
    pub fn log_filters(&self) -> &LogFilter {
        &self.logs
    }

    #[must_use]
    pub fn trace_filters(&self) -> &TraceFilter {
        &self.traces
    }
}
