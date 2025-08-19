// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: Apache-2.0

//! Debug Counter which enables the debug exporter to keep track of stats to report on

use std::fmt::Write;

/// Struct that has a counter for various data we want to track
#[derive(Default, Clone, Debug)]
pub struct DebugCounter {
    // counter to count the signals we receive between timerticks
    metric_signal_count: u64,
    profile_signal_count: u64,
    span_signal_count: u64,
    log_signal_count: u64,

    // counters to keep total count
    // counter to count the signals we receive
    total_metric_signal_count: u64,
    total_profile_signal_count: u64,
    total_span_signal_count: u64,
    total_log_signal_count: u64,

    // counter to count the resources of each signal
    total_resource_metric_count: u64,
    total_resource_span_count: u64,
    total_resource_profile_count: u64,
    total_resource_log_count: u64,

    // counter to count additional data we receive from the signals
    total_metric_count: u64,     // count the metrics
    total_log_count: u64,        // count the log records
    total_span_count: u64,       // count the spans
    total_sample_count: u64,     // count the samples we receive in profile signal
    total_span_event_count: u64, // count the number of span events
    total_span_link_count: u64,  // count the number of span links
    total_log_event_count: u64,  // count the number of log records with event name
    total_data_point_count: u64, // count the datapoints we receive in metric signal
}

impl DebugCounter {
    /// reset the signal counts after timertick
    pub fn reset_signal_count(&mut self) {
        self.metric_signal_count = 0;
        self.profile_signal_count = 0;
        self.span_signal_count = 0;
        self.log_signal_count = 0;
    }
    /// increment counter for metric signal
    pub fn increment_metric_signal_count(&mut self) {
        self.total_metric_signal_count += 1;
        self.metric_signal_count += 1;
    }
    /// increment counter for profile signal
    pub fn increment_profile_signal_count(&mut self) {
        self.total_profile_signal_count += 1;
        self.profile_signal_count += 1;
    }
    /// increment counter for span signal
    pub fn increment_span_signal_count(&mut self) {
        self.total_span_signal_count += 1;
        self.span_signal_count += 1;
    }
    /// increment counter for log signal
    pub fn increment_log_signal_count(&mut self) {
        self.total_log_signal_count += 1;
        self.log_signal_count += 1;
    }

    /// update the counters for metric data
    pub fn update_metric_data(&mut self, resource_metrics: u64, metrics: u64, data_points: u64) {
        self.total_resource_metric_count += resource_metrics;
        self.total_metric_count += metrics;
        self.total_data_point_count += data_points;
    }
    /// update the counters for span data
    pub fn update_span_data(&mut self, resource_spans: u64, spans: u64, events: u64, links: u64) {
        self.total_resource_span_count += resource_spans;
        self.total_span_count += spans;
        self.total_span_event_count += events;
        self.total_span_link_count += links;
    }
    /// update the counters for log date
    pub fn update_log_data(&mut self, resource_logs: u64, log_records: u64, events: u64) {
        self.total_resource_log_count += resource_logs;
        self.total_log_count += log_records;
        self.total_log_event_count += events;
    }
    /// update the counters for profile data
    pub fn update_profile_data(&mut self, resource_profiles: u64, samples: u64) {
        self.total_resource_profile_count += resource_profiles;
        self.total_sample_count += samples;
    }
    /// Generate report of total data received, output stats
    #[must_use]
    pub fn debug_report(&self) -> String {
        let mut report = String::new();
        _ = writeln!(&mut report, "Debug Exporter Summary:");

        _ = writeln!(
            &mut report,
            "OTLP Metric objects received: {metric_signal}",
            metric_signal = self.total_metric_signal_count
        );
        _ = writeln!(
            &mut report,
            "Received {resource_metric} Resource Metrics",
            resource_metric = self.total_resource_metric_count
        );
        _ = writeln!(
            &mut report,
            "Received {metric} metrics",
            metric = self.total_metric_count
        );
        _ = writeln!(
            &mut report,
            "Received {datapoint} datapoints",
            datapoint = self.total_data_point_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Trace objects received: {span_signal}",
            span_signal = self.total_span_signal_count
        );
        _ = writeln!(
            &mut report,
            "Received {resource_spans} Resource Spans",
            resource_spans = self.total_resource_span_count
        );
        _ = writeln!(
            &mut report,
            "Received {spans} spans",
            spans = self.total_span_count
        );
        _ = writeln!(
            &mut report,
            "Received {events} events",
            events = self.total_span_event_count
        );
        _ = writeln!(
            &mut report,
            "Received {links} links",
            links = self.total_span_link_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Log objects received: {log_signal}",
            log_signal = self.total_log_signal_count
        );
        _ = writeln!(
            &mut report,
            "Received {resource_log} Resource logs",
            resource_log = self.total_resource_log_count
        );
        _ = writeln!(
            &mut report,
            "Received {log_record} log records",
            log_record = self.total_log_count
        );
        _ = writeln!(
            &mut report,
            "Received {log_event} log events",
            log_event = self.total_log_event_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Profile objects received: {profile_signal}",
            profile_signal = self.total_profile_signal_count
        );
        _ = writeln!(
            &mut report,
            "Received {resource_profile} Resource profiles",
            resource_profile = self.total_resource_profile_count
        );
        _ = writeln!(
            &mut report,
            "Received {samples} samples",
            samples = self.total_sample_count
        );

        report
    }
    /// Generate report of objects received between timer ticks
    #[must_use]
    pub fn signals_count_report(&self) -> String {
        let mut report = String::new();
        _ = writeln!(
            &mut report,
            "OTLP Metric objects received: {metric_count}",
            metric_count = self.metric_signal_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Trace objects received: {span_count}",
            span_count = self.span_signal_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Profile objects received: {profile_count}",
            profile_count = self.profile_signal_count
        );
        _ = writeln!(
            &mut report,
            "OTLP Log objects received: {log_count}",
            log_count = self.log_signal_count
        );

        report
    }
}
