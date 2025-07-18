// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!

use crate::fake_signal_receiver::config::MetricType;
use crate::fake_signal_receiver::fake_data::*;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::InstrumentationScope,
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
    metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
        SummaryDataPoint,
    },
    resource::v1::Resource,
    trace::v1::{
        ResourceSpans, ScopeSpans, Span, TracesData,
        span::{Event, Link},
    },
};

/// Genererates MetricsData based on the provided count for resource, scope, metric, datapoint count
#[must_use]
pub fn fake_otlp_metrics(
    resource_metrics_count: usize,
    scope_metrics_count: usize,
    metric_count: usize,
    datapoint_count: usize,
    datapoint_type: MetricType,
    attribute_count: usize,
) -> MetricsData {
    let mut resource_metrics: Vec<ResourceMetrics> = vec![];

    for _ in 0..resource_metrics_count {
        let mut scope_metrics: Vec<ScopeMetrics> = vec![];
        for _ in 0..scope_metrics_count {
            // create some fake metrics based on the metric_count value provided
            let mut metrics: Vec<Metric> = vec![];
            for _ in 0..metric_count {
                metrics.push(fake_metric(
                    datapoint_type,
                    datapoint_count,
                    attribute_count,
                ));
            }
            scope_metrics.push(
                ScopeMetrics::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .attributes(get_attributes(attribute_count))
                        .finish(),
                )
                .metrics(metrics)
                .finish(),
            );
        }

        resource_metrics.push(
            ResourceMetrics::build(Resource::default())
                .scope_metrics(scope_metrics)
                .finish(),
        );
    }

    MetricsData::new(resource_metrics)
}

/// Genererates TracesData based on the provided count for resource, scope, span, event, link count
#[must_use]
pub fn fake_otlp_traces(
    resource_spans_count: usize,
    scope_spans_count: usize,
    span_count: usize,
    event_count: usize,
    link_count: usize,
    attribute_count: usize,
) -> TracesData {
    let mut resource_spans: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_spans_count {
        let mut scope_spans: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_spans_count {
            let mut spans: Vec<Span> = vec![];
            for _ in 0..span_count {
                spans.push(fake_span(event_count, link_count, attribute_count));
            }
            scope_spans.push(
                ScopeSpans::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .attributes(get_attributes(attribute_count))
                        .finish(),
                )
                .spans(spans)
                .finish(),
            );
        }

        resource_spans.push(
            ResourceSpans::build(Resource::default())
                .scope_spans(scope_spans)
                .finish(),
        );
    }

    TracesData::new(resource_spans)
}

/// Genererates LogsData based on the provided count for resource logs, scope logs and log records
#[must_use]
pub fn fake_otlp_logs(
    resource_logs_count: usize,
    scope_logs_count: usize,
    log_records_count: usize,
    attribute_count: usize,
) -> LogsData {
    let mut resource_logs: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_logs_count {
        let mut scope_logs: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_logs_count {
            let mut log_records: Vec<LogRecord> = vec![];
            for _ in 0..log_records_count {
                log_records.push(fake_log_records(attribute_count));
            }
            scope_logs.push(
                ScopeLogs::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .attributes(get_attributes(attribute_count))
                        .finish(),
                )
                .log_records(log_records)
                .finish(),
            );
        }

        resource_logs.push(
            ResourceLogs::build(Resource::default())
                .scope_logs(scope_logs)
                .finish(),
        );
    }

    LogsData::new(resource_logs)
}

#[must_use]
fn fake_span(event_count: usize, link_count: usize, attribute_count: usize) -> Span {
    let mut links: Vec<Link> = vec![];
    for _ in 0..link_count {
        links.push(Link::new(get_trace_id(), get_span_id()));
    }
    let mut events: Vec<Event> = vec![];
    for _ in 0..event_count {
        events.push(Event::new(get_event_name(), get_time_unix_nano()));
    }

    Span::build(
        get_trace_id(),
        get_span_id(),
        get_span_name(),
        get_start_time_unix_nano(),
    )
    .attributes(get_attributes(attribute_count))
    .flags(get_span_flag())
    .kind(get_span_kind())
    .trace_state(get_trace_state())
    .links(links)
    .events(events)
    .end_time_unix_nano(get_end_time_unix_nano())
    .status(get_status())
    .dropped_attributes_count(0u32)
    .dropped_events_count(0u32)
    .dropped_links_count(0u32)
    .finish()
}
#[must_use]
fn fake_metric(
    datapoint_type: MetricType,
    datapoint_count: usize,
    attribute_count: usize,
) -> Metric {
    match datapoint_type {
        MetricType::Gauge => {
            let datapoints = fake_number_datapoints(datapoint_count, attribute_count);
            Metric::new_gauge("metric_gauge", Gauge::new(datapoints))
        }
        MetricType::Sum => {
            let datapoints = fake_number_datapoints(datapoint_count, attribute_count);
            Metric::new_sum("metric_sum", Sum::new(get_aggregation(), true, datapoints))
        }
        MetricType::Histogram => {
            let datapoints = fake_histogram_datapoints(datapoint_count, attribute_count);
            Metric::new_histogram(
                "metric_histogram",
                Histogram::new(get_aggregation(), datapoints),
            )
        }
        MetricType::ExponentialHistogram => {
            let datapoints = fake_exp_histogram_datapoints(datapoint_count, attribute_count);
            Metric::new_exponential_histogram(
                "metric_exponential_histogram",
                ExponentialHistogram::new(get_aggregation(), datapoints),
            )
        }
        MetricType::Summary => {
            let datapoints = fake_summary_datapoints(datapoint_count, attribute_count);
            Metric::new_summary("metric_summary", Summary::new(datapoints))
        }
    }
}

#[must_use]
fn fake_log_records(attribute_count: usize) -> LogRecord {
    let severity_number = get_severity_number();
    let severity_text = get_severity_text(severity_number);
    LogRecord::build(get_time_unix_nano(), severity_number, get_event_name())
        .observed_time_unix_nano(get_time_unix_nano())
        .trace_id(get_trace_id())
        .span_id(get_span_id())
        .severity_text(severity_text)
        .attributes(get_attributes(attribute_count))
        .dropped_attributes_count(0u32)
        .flags(get_log_record_flag())
        .body(get_body_text())
        .finish()
}

/// generate gauge datapoints
#[must_use]
fn fake_number_datapoints(datapoint_count: usize, attribute_count: usize) -> Vec<NumberDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            NumberDataPoint::build_double(get_time_unix_nano(), get_double_value())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attribute_count))
                .finish(),
        );
    }

    datapoints
}

/// generate histogram datapoints
#[must_use]
fn fake_histogram_datapoints(
    datapoint_count: usize,
    attribute_count: usize,
) -> Vec<HistogramDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            HistogramDataPoint::build(
                get_time_unix_nano(),
                get_buckets_count(),
                get_explicit_bounds(),
            )
            .start_time_unix_nano(get_start_time_unix_nano())
            .attributes(get_attributes(attribute_count))
            .finish(),
        )
    }

    datapoints
}

/// generate exponential histogram datapoints
#[must_use]
fn fake_exp_histogram_datapoints(
    datapoint_count: usize,
    attribute_count: usize,
) -> Vec<ExponentialHistogramDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            ExponentialHistogramDataPoint::build(get_time_unix_nano(), 1, get_buckets())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attribute_count))
                .count(get_int_value())
                .zero_count(get_int_value())
                .negative(get_buckets())
                .finish(),
        );
    }

    datapoints
}

/// generate summary datapoints
#[must_use]
fn fake_summary_datapoints(
    datapoint_count: usize,
    attribute_count: usize,
) -> Vec<SummaryDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            SummaryDataPoint::build(get_time_unix_nano(), get_quantiles())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attribute_count))
                .count(get_int_value())
                .sum(get_double_value())
                .finish(),
        );
    }
    datapoints
}
