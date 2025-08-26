// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the ViewMarshaler for converting OTLP views to structured string reports.

use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::LogsData,
    metrics::v1::MetricsData,
    trace::v1::TracesData,
};

/// Trait that provides methods to take OTLP views and extract information from them and generate a report
pub trait ViewMarshaler {
    /// extract data from logs and generate string report
    fn marshal_logs(&self, logs: LogsData) -> String;
    /// extract data from metricss and generate string report
    fn marshal_metrics(&self, metrics: MetricsData) -> String;
    /// extract data from traces and generate string report
    fn marshal_traces(&self, traces: TracesData) -> String;
}
