// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

use crate::fake_signal_receiver::config::MetricType;
use crate::fake_signal_receiver::fake_data::*;
use crate::proto::opentelemetry::{
    common::v1::InstrumentationScope,
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
        metric::Data, number_data_point::Value,
    },
    trace::v1::{
        ResourceSpans, ScopeSpans, Span,
        span::{Event, Link},
    },
};

use crate::proto::opentelemetry::collector::{
    logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
    trace::v1::ExportTraceServiceRequest,
};

/// Genererates ExportMetricsServiceRequest based on the provided count for resource, scope, metric, datapoint count
#[must_use]
pub fn fake_otlp_metrics(
    resource_metrics_count: usize,
    scope_metrics_count: usize,
    metric_count: usize,
    datapoint_count: usize,
    datapoint_type: MetricType,
    attribute_count: usize,
) -> ExportMetricsServiceRequest {
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
            scope_metrics.push(ScopeMetrics {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(attribute_count),
                    dropped_attributes_count: 0,
                }),
                metrics: metrics.clone(),
            });
        }

        resource_metrics.push(ResourceMetrics {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: None,
            scope_metrics: scope_metrics.clone(),
        });
    }
    ExportMetricsServiceRequest {
        resource_metrics: resource_metrics.clone(),
    }
}

/// Genererates ExportTraceServiceRequest based on the provided count for resource, scope, span, event, link count
#[must_use]
pub fn fake_otlp_traces(
    resource_spans_count: usize,
    scope_spans_count: usize,
    span_count: usize,
    event_count: usize,
    link_count: usize,
    attribute_count: usize,
) -> ExportTraceServiceRequest {
    let mut resource_spans: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_spans_count {
        let mut scope_spans: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_spans_count {
            let mut spans: Vec<Span> = vec![];
            for _ in 0..span_count {
                spans.push(fake_span(event_count, link_count, attribute_count));
            }
            scope_spans.push(ScopeSpans {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(attribute_count),
                    dropped_attributes_count: 0,
                }),
                spans: spans.clone(),
            });
        }

        resource_spans.push(ResourceSpans {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: None,
            scope_spans: scope_spans.clone(),
        });
    }

    ExportTraceServiceRequest {
        resource_spans: resource_spans.clone(),
    }
}

/// Genererates ExportLogsServiceRequest based on the provided count for resource logs, scope logs and log records
#[must_use]
pub fn fake_otlp_logs(
    resource_logs_count: usize,
    scope_logs_count: usize,
    log_records_count: usize,
    attribute_count: usize,
) -> ExportLogsServiceRequest {
    let mut resource_logs: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_logs_count {
        let mut scope_logs: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_logs_count {
            let mut log_records: Vec<LogRecord> = vec![];
            for _ in 0..log_records_count {
                log_records.push(fake_log_records(attribute_count));
            }
            scope_logs.push(ScopeLogs {
                schema_url: "http://schema.opentelemetry.io".to_string(),
                scope: Some(InstrumentationScope {
                    name: get_scope_name(),
                    version: get_scope_version(),
                    attributes: get_attributes(attribute_count),
                    dropped_attributes_count: 0,
                }),
                log_records: log_records.clone(),
            });
        }

        resource_logs.push(ResourceLogs {
            schema_url: "http://schema.opentelemetry.io".to_string(),
            resource: None,
            scope_logs: scope_logs.clone(),
        });
    }

    ExportLogsServiceRequest {
        resource_logs: resource_logs.clone(),
    }
}

#[must_use]
fn fake_span(event_count: usize, link_count: usize, attribute_count: usize) -> Span {
    let mut links: Vec<Link> = vec![];
    for _ in 0..link_count {
        links.push(Link {
            trace_id: get_trace_id(),
            span_id: get_span_id(),
            attributes: vec![],
            trace_state: get_trace_state(),
            dropped_attributes_count: 0,
            flags: 4,
        });
    }
    let mut events: Vec<Event> = vec![];
    for _ in 0..event_count {
        events.push(Event {
            time_unix_nano: get_time_unix_nano(),
            name: get_event_name(),
            attributes: vec![],
            dropped_attributes_count: 0,
        })
    }

    Span {
        end_time_unix_nano: get_end_time_unix_nano(),
        start_time_unix_nano: get_start_time_unix_nano(),
        name: get_span_name(),
        kind: get_span_kind() as i32,
        trace_state: get_trace_state(),
        status: Some(get_status()),
        links: links.clone(),
        events: events.clone(),
        attributes: get_attributes(attribute_count),
        trace_id: get_trace_id(),
        span_id: get_span_id(),
        parent_span_id: get_span_id(),
        dropped_attributes_count: 0,
        flags: get_span_flag() as u32,
        dropped_events_count: 0,
        dropped_links_count: 0,
    }
}
#[must_use]
fn fake_metric(
    datapoint_type: MetricType,
    datapoint_count: usize,
    attribute_count: usize,
) -> Metric {
    let metric_data = match datapoint_type {
        MetricType::Gauge => {
            let datapoints = fake_number_datapoints(datapoint_count, attribute_count);
            Data::Gauge(Gauge {
                data_points: datapoints.clone(),
            })
        }
        MetricType::Sum => {
            let datapoints = fake_number_datapoints(datapoint_count, attribute_count);
            Data::Sum(Sum {
                data_points: datapoints.clone(),
                aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
                is_monotonic: true,
            })
        }
        MetricType::Histogram => {
            let datapoints = fake_histogram_datapoints(datapoint_count, attribute_count);
            Data::Histogram(Histogram {
                data_points: datapoints.clone(),
                aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
            })
        }
        MetricType::ExponentialHistogram => {
            let datapoints = fake_exp_histogram_datapoints(datapoint_count, attribute_count);
            Data::ExponentialHistogram(ExponentialHistogram {
                data_points: datapoints.clone(),
                aggregation_temporality: 4, // AGGREGATION_TEMPORALITY_DELTA
            })
        }
        MetricType::Summary => {
            let datapoints = fake_summary_datapoints(datapoint_count, attribute_count);
            Data::Summary(Summary {
                data_points: datapoints.clone(),
            })
        }
    };

    Metric {
        name: "metric_name".to_string(),
        description: "metric_description".to_string(),
        unit: "s".to_string(),
        metadata: vec![],
        data: Some(metric_data),
    }
}

#[must_use]
fn fake_log_records(attribute_count: usize) -> LogRecord {
    let severity_number = get_severity_number();
    let severity_text = get_severity_text(severity_number);

    LogRecord {
        time_unix_nano: get_time_unix_nano(),
        observed_time_unix_nano: get_time_unix_nano(),
        severity_text: severity_text,
        severity_number: severity_number,
        event_name: get_event_name(),
        attributes: get_attributes(attribute_count),
        trace_id: get_trace_id(),
        span_id: get_span_id(),
        body: Some(get_body_text()),
        flags: get_log_record_flag() as u32,
        dropped_attributes_count: 0,
    }
}

/// generate gauge datapoints
#[must_use]
fn fake_number_datapoints(datapoint_count: usize, attribute_count: usize) -> Vec<NumberDataPoint> {
    let mut datapoints = vec![];
    for _ in 0..datapoint_count {
        datapoints.push(NumberDataPoint {
            start_time_unix_nano: get_start_time_unix_nano(),
            time_unix_nano: get_time_unix_nano(),
            attributes: get_attributes(attribute_count),
            value: Some(Value::AsDouble(get_double_value())),
            flags: 0,
            exemplars: vec![],
        });
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
        datapoints.push(HistogramDataPoint {
            attributes: get_attributes(attribute_count),
            start_time_unix_nano: get_start_time_unix_nano(),
            time_unix_nano: get_time_unix_nano(),
            explicit_bounds: get_explicit_bounds(),
            bucket_counts: vec![],
            sum: Some(get_double_value()),
            count: 0,
            flags: 0,
            min: Some(12.0),
            max: Some(100.1),
            exemplars: vec![],
        })
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
        datapoints.push(ExponentialHistogramDataPoint {
            attributes: get_attributes(attribute_count),
            start_time_unix_nano: get_start_time_unix_nano(),
            time_unix_nano: get_time_unix_nano(),
            sum: Some(get_double_value()),
            count: 0,
            flags: 0,
            min: Some(12.0),
            max: Some(100.1),
            exemplars: vec![],
            scale: 1,
            positive: None,
            negative: None,
            zero_threshold: 0.0,
            zero_count: 0,
        });
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
        datapoints.push(SummaryDataPoint {
            start_time_unix_nano: get_start_time_unix_nano(),
            time_unix_nano: get_time_unix_nano(),
            attributes: get_attributes(attribute_count),
            sum: get_double_value(),
            count: 0,
            flags: 0,
            quantile_values: vec![],
        });
    }
    datapoints
}
