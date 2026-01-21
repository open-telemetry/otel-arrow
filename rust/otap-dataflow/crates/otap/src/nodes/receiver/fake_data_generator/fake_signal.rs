// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

use crate::fake_data_generator::attributes::get_attribute_name_value;
use crate::fake_data_generator::fake_data::{
    current_time, delay, gen_span_id, gen_trace_id, get_scope_name, get_scope_version,
};
use otap_df_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    metrics::v1::{
        AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
        NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    },
    resource::v1::Resource,
    trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData, span::Event, span::SpanKind},
};
use weaver_forge::registry::ResolvedRegistry;
use weaver_semconv::group::{GroupType, InstrumentSpec, SpanKindSpec};

/// Generates TracesData with the specified resource/scope count and defined spans in the registry
#[must_use]
pub fn fake_otlp_traces(signal_count: usize, registry: &ResolvedRegistry) -> TracesData {
    let scopes: Vec<ScopeSpans> = vec![ScopeSpans::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        spans(signal_count, registry),
    )];

    let resources: Vec<ResourceSpans> = vec![ResourceSpans::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "fake_data_generator",
                AnyValue::new_string("v1"),
            )])
            .finish(),
        scopes,
    )];
    TracesData::new(resources)
}

/// Generates LogsData with the specified resource/scope count and defined events (structured logs) in the registry
#[must_use]
pub fn fake_otlp_logs(signal_count: usize, registry: &ResolvedRegistry) -> LogsData {
    let scopes: Vec<ScopeLogs> = vec![ScopeLogs::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        logs(signal_count, registry),
    )];

    let resources: Vec<ResourceLogs> = vec![ResourceLogs::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "fake_data_generator",
                AnyValue::new_string("v1"),
            )])
            .finish(),
        scopes,
    )];

    LogsData::new(resources)
}

/// Generates MetricsData with the specified resource/scope count and defined metrics in the registry
#[must_use]
pub fn fake_otlp_metrics(signal_count: usize, registry: &ResolvedRegistry) -> MetricsData {
    let scopes: Vec<ScopeMetrics> = vec![ScopeMetrics::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        metrics(signal_count, registry),
    )];

    let resources: Vec<ResourceMetrics> = vec![ResourceMetrics::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "fake_data_generator",
                AnyValue::new_string("v1"),
            )])
            .finish(),
        scopes,
    )];

    MetricsData::new(resources)
}

/// generate each span defined in the resolved registry
#[must_use]
fn spans(signal_count: usize, registry: &ResolvedRegistry) -> Vec<Span> {
    // Emit each span to the OTLP receiver.
    let mut spans = vec![];
    for group in registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Span)
        .cycle()
        .take(signal_count)
    {
        let start_time = current_time();
        // todo add random delay (configurable via annotations?)
        let end_time = start_time + delay();
        spans.push(
            Span::build()
                .trace_id(gen_trace_id())
                .span_id(gen_span_id())
                .name(group.id.clone())
                .start_time_unix_nano(start_time)
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
                        .map(|event_name| {
                            Event::build()
                                .name(event_name.clone())
                                .time_unix_nano(current_time())
                                .finish()
                        })
                        .collect::<Vec<_>>(),
                )
                .kind(otel_span_kind(group.span_kind.as_ref()))
                .end_time_unix_nano(end_time)
                .finish(),
        );
    }
    spans
}

/// generate each metric defined in the resolved registry
#[must_use]
fn metrics(signal_count: usize, registry: &ResolvedRegistry) -> Vec<Metric> {
    let mut metrics = vec![];

    for group in registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Metric)
        .cycle()
        .take(signal_count)
    {
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
                        NumberDataPoint::build()
                            .time_unix_nano(current_time())
                            .value_double(1.0)
                            .attributes(attributes)
                            .finish(),
                    ];
                    // is not monotonic
                    metrics.push(
                        Metric::build()
                            .name(metric_name)
                            .data_sum(Sum::new(
                                AggregationTemporality::Unspecified,
                                false,
                                datapoints,
                            ))
                            .description(description)
                            .unit(unit)
                            .finish(),
                    );
                }
                InstrumentSpec::Counter => {
                    let datapoints = vec![
                        NumberDataPoint::build()
                            .time_unix_nano(current_time())
                            .value_double(1.0)
                            .attributes(attributes)
                            .finish(),
                    ];
                    // is monotonic
                    metrics.push(
                        Metric::build()
                            .name(metric_name)
                            .data_sum(Sum::new(
                                AggregationTemporality::Unspecified,
                                true,
                                datapoints,
                            ))
                            .description(description)
                            .unit(unit)
                            .finish(),
                    );
                }
                InstrumentSpec::Gauge => {
                    let datapoints = vec![
                        NumberDataPoint::build()
                            .time_unix_nano(current_time())
                            .value_double(1.0)
                            .attributes(attributes)
                            .finish(),
                    ];

                    metrics.push(
                        Metric::build()
                            .name(metric_name)
                            .data_gauge(Gauge::new(datapoints))
                            .description(description)
                            .unit(unit)
                            .finish(),
                    );
                }
                InstrumentSpec::Histogram => {
                    let datapoints = vec![
                        HistogramDataPoint::build()
                            .time_unix_nano(current_time())
                            .bucket_counts(vec![])
                            .explicit_bounds(vec![])
                            .attributes(attributes)
                            .finish(),
                    ];
                    metrics.push(
                        Metric::build()
                            .name(metric_name)
                            .data_histogram(Histogram::new(
                                AggregationTemporality::Unspecified,
                                datapoints,
                            ))
                            .description(description)
                            .unit(unit)
                            .finish(),
                    );
                }
            }
        }
    }
    metrics
}

/// generate each span defined in the resolved registry
#[must_use]
fn logs(signal_count: usize, registry: &ResolvedRegistry) -> Vec<LogRecord> {
    let mut log_records = vec![];
    for group in registry
        .groups
        .iter()
        .filter(|g| g.r#type == GroupType::Event)
        .cycle()
        .take(signal_count)
    {
        // events are structured logs
        let timestamp = current_time();
        // extract the body
        let body_text = match &group.body {
            Some(body) => body.to_string(),
            None => "".to_string(),
        };

        log_records.push(
            LogRecord::build()
                .time_unix_nano(timestamp)
                .severity_number(SeverityNumber::Unspecified)
                .event_name(group.name.clone().unwrap_or("".to_owned()))
                .attributes(
                    group
                        .attributes
                        .iter()
                        .map(get_attribute_name_value)
                        .collect::<Vec<_>>(),
                )
                .body(AnyValue::new_string(body_text))
                .observed_time_unix_nano(timestamp)
                .finish(),
        );
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
