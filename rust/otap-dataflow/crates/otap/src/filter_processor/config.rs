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
    #[serde(default = "default_span_filter")]
    spans: TraceFilter, 
}

/// create empty log filter as default value
fn default_log_filter() -> LogFilter {
    LogFilter::new(None, None, vec![])
}

/// create empty span filter as default value
fn default_span_filter() -> TraceFilter {
    TraceFilter::new(None, None)
}

impl Config {
    pub fn new(logs: LogFilter, spans: TraceFilter) -> Self {
        Self { logs, spans }
    }

    #[must_use]
    pub fn log_filters(&self) -> &LogFilter {
        &self.logs
    }

    #[must_use]
    pub fn span_filters(&self) -> &TraceFilter {
        &self.spans
    }
}
