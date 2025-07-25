// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

use crate::fake_signal_receiver::config::{AttributeValue, EventConfig, LinkConfig, DatapointType};
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
use std::collections::HashMap;

/// Genererates MetricsData based on the provided count for resource, scope, metric, datapoint count
#[must_use]
pub fn fake_otlp_metrics(
    resource_metrics_count: usize,
    scope_metrics_count: usize,
    metric_count: usize,
    datapoint_count: usize,
    datapoint_type: DatapointType,
    attributes: &HashMap<String, Vec<AttributeValue>>,
) -> MetricsData {
    let mut resource_metrics: Vec<ResourceMetrics> = vec![];

    for _ in 0..resource_metrics_count {
        let mut scope_metrics: Vec<ScopeMetrics> = vec![];
        for _ in 0..scope_metrics_count {
            // create some fake metrics based on the metric_count value provided
            let mut metrics: Vec<Metric> = vec![];
            for _ in 0..metric_count {
                metrics.push(fake_metric(datapoint_type, datapoint_count, attributes));
            }
            scope_metrics.push(
                ScopeMetrics::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
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
    attributes: &HashMap<String, Vec<AttributeValue>>,
    span_names: &Vec<String>,
    events: &EventConfig,
    links: &LinkConfig,
) -> TracesData {
    let mut resource_spans: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_spans_count {
        let mut scope_spans: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_spans_count {
            let mut spans: Vec<Span> = vec![];
            for _ in 0..span_count {
                spans.push(fake_span(events, links, attributes, span_names));
            }
            scope_spans.push(
                ScopeSpans::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
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
    attributes: &HashMap<String, Vec<AttributeValue>>,
    event_names: &Vec<String>,
) -> LogsData {
    let mut resource_logs: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_logs_count {
        let mut scope_logs: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_logs_count {
            let mut log_records: Vec<LogRecord> = vec![];
            for _ in 0..log_records_count {
                log_records.push(fake_log_records(attributes, event_names));
            }
            scope_logs.push(
                ScopeLogs::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
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
fn fake_span(
    event_config: &EventConfig,
    link_config: &LinkConfig,
    attributes: &HashMap<String, Vec<AttributeValue>>,
    span_names: &Vec<String>,
) -> Span {
    let mut links: Vec<Link> = vec![];
    let link_attributes = link_config.get_attributes();
    let link_trace_states = link_config.get_trace_states();
    let link_count = link_config.get_count();
    for _ in 0..link_count {
        links.push(
            Link::build(get_trace_id(), get_span_id())
                .attributes(get_attributes(link_attributes))
                .trace_state(get_random_string_from_vec(link_trace_states))
                .finish(),
        );
    }
    let mut events: Vec<Event> = vec![];
    let event_attributes = event_config.get_attributes();
    let event_names = event_config.get_event_names();
    let event_count = event_config.get_count();
    for _ in 0..event_count {
        events.push(
            Event::build(
                get_random_string_from_vec(event_names),
                get_time_unix_nano(),
            )
            .attributes(get_attributes(event_attributes))
            .finish(),
        );
    }

    Span::build(
        get_trace_id(),
        get_span_id(),
        get_random_string_from_vec(span_names),
        get_start_time_unix_nano(),
    )
    .attributes(get_attributes(attributes))
    .flags(get_span_flag())
    .kind(get_span_kind())
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
    datapoint_type: DatapointType,
    datapoint_count: usize,
    attributes: &HashMap<String, Vec<AttributeValue>>,
) -> Metric {
    match datapoint_type {
        DatapointType::Gauge => {
            let datapoints = fake_number_datapoints(datapoint_count, attributes);
            Metric::new_gauge("metric_gauge", Gauge::new(datapoints))
        }
        DatapointType::Sum => {
            let datapoints = fake_number_datapoints(datapoint_count, attributes);
            Metric::new_sum("metric_sum", Sum::new(get_aggregation(), true, datapoints))
        }
        DatapointType::Histogram => {
            let datapoints = fake_histogram_datapoints(datapoint_count, attributes);
            Metric::new_histogram(
                "metric_histogram",
                Histogram::new(get_aggregation(), datapoints),
            )
        }
        DatapointType::ExponentialHistogram => {
            let datapoints = fake_exp_histogram_datapoints(datapoint_count, attributes);
            Metric::new_exponential_histogram(
                "metric_exponential_histogram",
                ExponentialHistogram::new(get_aggregation(), datapoints),
            )
        }
        DatapointType::Summary => {
            let datapoints = fake_summary_datapoints(datapoint_count, attributes);
            Metric::new_summary("metric_summary", Summary::new(datapoints))
        }
    }
}

#[must_use]
fn fake_log_records(
    attributes: &HashMap<String, Vec<AttributeValue>>,
    event_names: &Vec<String>,
) -> LogRecord {
    let severity_number = get_severity_number();
    let severity_text = get_severity_text(severity_number);
    LogRecord::build(
        get_time_unix_nano(),
        severity_number,
        get_random_string_from_vec(event_names),
    )
    .observed_time_unix_nano(get_time_unix_nano())
    .trace_id(get_trace_id())
    .span_id(get_span_id())
    .severity_text(severity_text)
    .attributes(get_attributes(attributes))
    .dropped_attributes_count(0u32)
    .flags(get_log_record_flag())
    .body(get_body_text())
    .finish()
}

/// generate gauge datapoints
#[must_use]
fn fake_number_datapoints(
    datapoint_count: usize,
    attributes: &HashMap<String, Vec<AttributeValue>>,
) -> Vec<NumberDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            NumberDataPoint::build_double(get_time_unix_nano(), get_double_value())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attributes))
                .finish(),
        );
    }

    datapoints
}

/// generate histogram datapoints
#[must_use]
fn fake_histogram_datapoints(
    datapoint_count: usize,
    attributes: &HashMap<String, Vec<AttributeValue>>,
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
            .attributes(get_attributes(attributes))
            .finish(),
        )
    }

    datapoints
}

/// generate exponential histogram datapoints
#[must_use]
fn fake_exp_histogram_datapoints(
    datapoint_count: usize,
    attributes: &HashMap<String, Vec<AttributeValue>>,
) -> Vec<ExponentialHistogramDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            ExponentialHistogramDataPoint::build(get_time_unix_nano(), 1, get_buckets())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attributes))
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
    attributes: &HashMap<String, Vec<AttributeValue>>,
) -> Vec<SummaryDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(
            SummaryDataPoint::build(get_time_unix_nano(), get_quantiles())
                .start_time_unix_nano(get_start_time_unix_nano())
                .attributes(get_attributes(attributes))
                .count(get_int_value())
                .sum(get_double_value())
                .finish(),
        );
    }
    datapoints
}

#[cfg(test)]
mod tests {
    use crate::fake_signal_receiver::fake_signal::*;

    #[test]
    fn test_fake_metric() {}

    #[test]
    fn test_fake_trace() {}

    #[test]
    fn test_fake_log() {}
}
