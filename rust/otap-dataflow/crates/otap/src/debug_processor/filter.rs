// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the filter definition for the debug processor

use super::predicate::Predicate;
use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData,
};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterMode {
    Include,
    Exclude,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FilterRules {
    predicate: Predicate,
    mode: FilterMode,
}

impl FilterRules {
    #[must_use]
    pub fn new(predicate: Predicate, mode: FilterMode) -> Self {
        Self { predicate, mode }
    }
    pub fn filter_logs(&self, logs_data: &mut LogsData) {
        // call predicate function to check for match against data
        for resource_logs in &mut logs_data.resource_logs {
            for scope_logs in &mut resource_logs.scope_logs {
                match self.mode {
                    FilterMode::Include => scope_logs
                        .log_records
                        .retain(|signal| self.predicate.match_log(signal)),

                    FilterMode::Exclude => scope_logs
                        .log_records
                        .retain(|signal| !self.predicate.match_log(signal)),
                }
            }
        }
    }

    pub fn filter_metrics(&self, metrics_data: &mut MetricsData) {
        for resource_metrics in &mut metrics_data.resource_metrics {
            for scope_metrics in &mut resource_metrics.scope_metrics {
                match self.mode {
                    FilterMode::Include => scope_metrics
                        .metrics
                        .retain(|signal| self.predicate.match_metric(signal)),

                    FilterMode::Exclude => scope_metrics
                        .metrics
                        .retain(|signal| !self.predicate.match_metric(signal)),
                }
            }
        }
    }

    pub fn filter_traces(&self, traces_data: &mut TracesData) {
        for resource_spans in &mut traces_data.resource_spans {
            for scope_spans in &mut resource_spans.scope_spans {
                match self.mode {
                    FilterMode::Include => scope_spans
                        .spans
                        .retain(|signal| self.predicate.match_span(signal)),

                    FilterMode::Exclude => scope_spans
                        .spans
                        .retain(|signal| !self.predicate.match_span(signal)),
                }
            }
        }
    }
}
