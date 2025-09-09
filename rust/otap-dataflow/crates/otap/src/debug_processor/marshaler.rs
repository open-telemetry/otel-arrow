// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the ViewMarshaler for converting OTLP views to structured string reports.

use otel_arrow_rust::proto::opentelemetry::{
    logs::v1::{LogsData, LogRecord}, metrics::v1::{MetricsData, Metric}, trace::v1::{TracesData, Span}
};

/// Trait that provides methods to take OTLP views and extract information from them and generate a report
pub trait ViewMarshaler {
    /// extract data from logs batch and generate string report
    fn marshal_logs(&self, logs: LogsData) -> String;
    /// extract data from metrics batch and generate string report
    fn marshal_metrics(&self, metrics: MetricsData) -> String;
    /// extract data from traces batch and generate string report
    fn marshal_traces(&self, traces: TracesData) -> String;
    /// extract data from log signal and generate string report
    fn marshal_logs_signal(&self, log_record: LogsRecord) -> String;
    /// extract data from metric signal and generate string report
    fn marshal_metrics_signal(&self, metric: Metric) -> String;
    /// extract data from span signal and generate string report
    fn marshal_span_signal(&self, span: Span) -> String;
}
