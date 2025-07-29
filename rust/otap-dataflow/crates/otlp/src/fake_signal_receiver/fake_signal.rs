// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{InstrumentationScope, AnyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber, LogsData},
    metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
        metric::Data, number_data_point::Value, MetricsData
    },
    trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData, span::SpanKind, span::Event},
    resource::v1::Resource
};
use crate::fake_signal_receiver::fake_data::{current_time, gen_span_id, gen_trace_id, get_scope_name, get_scope_version};
use crate::fake_signal_receiver::attributes::get_attribute_name_value;
use weaver_forge::registry::ResolvedRegistry;
use weaver_semconv::group::{GroupType, InstrumentSpec, SpanKindSpec};

/// Generates TracesData with the specified resource/scope count and defined spans in the registry
#[must_use]
pub fn fake_otlp_traces(
    resource_count: usize,
    scope_count: usize,
    registry: &ResolvedRegistry,
) -> TracesData {
    let mut resources: Vec<ResourceSpans> = vec![];

    for _ in 0..resource_count {
        let mut scopes: Vec<ScopeSpans> = vec![];
        for _ in 0..scope_count {
            scopes.push(
                ScopeSpans::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .finish(),
                )
                .spans(spans(registry))
                .finish(),
            );
        }

        resources.push(
            ResourceSpans::build(Resource::default())
                .scope_spans(scopes)
                .finish(),
        );
    }

    TracesData::new(resources)
}

/// Generates LogsData with the specified resource/scope count and defined events (structured logs) in the registry
#[must_use]
pub fn fake_otlp_logs(
    resource_count: usize,
    scope_count: usize,
    registry: &ResolvedRegistry,
) -> LogsData {
    let mut resources: Vec<ResourceLogs> = vec![];

    for _ in 0..resource_count {
        let mut scopes: Vec<ScopeLogs> = vec![];
        for _ in 0..scope_count {
            scopes.push(
                ScopeLogs::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .finish(),
                )
                .log_records(logs(registry))
                .finish(),
            );
        }

        resources.push(
            ResourceSpans::build(Resource::default())
                .scope_spans(scopes)
                .finish(),
        );
    }

    LogsData::new(resources)
}

/// Generates MetricsData with the specified resource/scope count and defined metrics in the registry
#[must_use]
pub fn fake_otlp_metrics(
    resource_count: usize,
    scope_count: usize,
    registry: &ResolvedRegistry,
) -> MetricsData {
    let mut resources: Vec<ResourceMetrics> = vec![];

    for _ in 0..resource_count {
        let mut scopes: Vec<ScopeMetrics> = vec![];
        for _ in 0..scope_count {
            scopes.push(
                ScopeMetrics::build(
                    InstrumentationScope::build(get_scope_name())
                        .version(get_scope_version())
                        .finish(),
                )
                .metrics(metrics(registry))
                .finish(),
            );
        }

        resources.push(
            ResourceMetrics::build(Resource::default())
                .scope_spans(scopes)
                .finish(),
        );
    }

    MetricsData::new(resources)
}

/// generate each span defined in the resolved registry
#[must_use]
fn spans(registry: &ResolvedRegistry) -> Vec<Span> {
    // Emit each span to the OTLP receiver.
    let spans = vec![];
    for group in registry.groups.iter() {
        if group.r#type == GroupType::Span {
            let start_time = current_time();
            // todo add random delay (configurable via annotations?)
            let end_time = start_time + 10000;
            spans.push(
                Span::build(gen_trace_id(), gen_span_id(), group.id.clone(), start_time)
                    .attributes(
                        group
                            .attributes
                            .iter()
                            .map(get_attribute_name_value)
                            .collect::<Vec<_>>(),
                    )
                    .events(
                        group
                            .events
                            .iter()
                            .map(|event_name| Event::new(event_name, current_time()))
                            .collect::<Vec<_>>(),
                    )
                    .kind(group.span_kind.as_ref())
                    .end_time_unix_nano(end_time)
                    .finish(),
            );
        }
    }
    spans
}

/// generate each metric defined in the resolved registry
#[must_use]
fn metrics(registry: &ResolvedRegistry) -> Vec<Metric> {
    let metrics = vec![];
    for group in registry.groups.iter() {
        if group.r#type == GroupType::Metric {
            if let Some(instrument) = &group.instrument {
                let metric_name = group.metric_name.clone().unwrap_or("".to_owned());
                let unit = group.unit.clone().unwrap_or("".to_owned());
                let description = group.brief.clone();

                let attributes = group
                    .attributes
                    .iter()
                    .map(get_attribute_name_value)
                    .collect::<Vec<_>>();

                // build the metrics here
                // todo add configurable datapoint_count
                // todo add configurable value range, distrubution
                match instrument {
                    InstrumentSpec::UpDownCounter => {
                        let datapoints = vec![
                            NumberDataPoint::build_double(current_time(), 1.0)
                                .attributes(attributes)
                                .finish(),
                        ];
                        // is not monotonic
                        metrics.push(
                            Metric::new_sum(metric_name, Sum::new(, false, datapoints))
                                .description(description)
                                .unit(unit),
                        );
                    }
                    InstrumentSpec::Counter => {
                        let datapoints = vec![
                            NumberDataPoint::build_double(current_time(), 1.0)
                                .attributes(attributes)
                                .finish(),
                        ];
                        // is monotonic
                        metrics.push(
                            Metric::new_sum(metric_name, Sum::new(, true, datapoints))
                                .description(description)
                                .unit(unit),
                        );
                    }
                    InstrumentSpec::Gauge => {
                        let datapoints = vec![
                            NumberDataPoint::build_double(current_time(), 1.0)
                                .attributes(attributes)
                                .finish(),
                        ];

                        metrics.push(
                            Metric::new_gauge(metric_name, Gauge::new(datapoints))
                                .description(description)
                                .unit(unit),
                        );
                    }
                    InstrumentSpec::Histogram => {
                        let datapoints = vec![
                            HistogramDataPoint::build(current_time(), vec![], vec![])
                                .attributes(attributes)
                                .finish(),
                        ];
                        metrics.push(
                            Metric::new_histogram(metric_name, Histogram::new(datapoints))
                                .description(description)
                                .unit(unit),
                        );
                    }
                }
            }
        }
    }
    metrics
}

/// generate each span defined in the resolved registry
#[must_use]
fn logs(registry: &ResolvedRegistry) -> Vec<LogRecord> {
    let log_records = vec![];
    for group in registry.groups.iter() {
        // events are structured logs
        if group.r#type == GroupType::Event {
            let timestamp = current_time();
            // extract the body
            let body_text = match group.body {
                Some(body) => body.to_string(),
                None => "".to_string(),
            };
            
            log_records.push(LogRecord::build(
                timestamp,
                SeverityNumber::Unspecified,
                group.name.clone().unwrap_or("".to_owned),
            )
            .attributes(
                group
                    .attributes
                    .iter()
                    .map(get_attribute_name_value)
                    .collect::<Vec<_>>(),
            )
            .body(AnyValue::new_string(body_text))
            .observed_time_unix_nano(timestamp)
            .finish());
        }
    }
    log_records
}

/// map a SpanKindSpec to a SpanKind
#[must_use]
fn otel_span_kind(span_kind: Option<&SpanKindSpec>) -> SpanKind {
    match span_kind {
        Some(SpanKindSpec::Client) => SpanKind::Client,
        Some(SpanKindSpec::Server) => SpanKind::Server,
        Some(SpanKindSpec::Producer) => SpanKind::Producer,
        Some(SpanKindSpec::Consumer) => SpanKind::Consumer,
        Some(SpanKindSpec::Internal) | None => SpanKind::Internal,
    }
}
