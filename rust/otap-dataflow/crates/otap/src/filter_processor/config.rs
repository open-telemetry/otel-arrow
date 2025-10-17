// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the configuration of the filter processor
//!

use otel_arrow_rust::otap::filter::LogFilter;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ToDo: add metrics and spans
    logs: LogFilter,
}

impl Config {
    pub fn new(logs: LogFilter) -> Self {
        Self { logs }
    }

    #[must_use]
    pub fn log_filters(&self) -> &LogFilter {
        &self.logs
    }
}
