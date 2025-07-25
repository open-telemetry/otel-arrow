// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

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
        span::SpanKind
    },
};
use std::collections::HashMap;
use weaver_forge::ResolvedRegistry;
use weaver_semconv::group::{GroupType, SpanKindSpec, InstrumentSpec};

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


#[must_use]
pub fn fake_otlp_metrics(
    resource_count: usize,
    scope_count: usize,
    registry: &ResolvedRegistry,
) -> MetricsData {
    let mut resources: Vec<ResourcesMetrics> = vec![];

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


fn spans(registry: &ResolvedRegistry) -> Vec<Span> {
// Emit each span to the OTLP receiver.
    let spans = vec![];
    for group in registry.groups.iter() {
        if group.r#type == GroupType::Span {
            let start_time = 1619712000000000000u64;
            let end_time = 1619712001000000000u64;
            let trace_id = TraceID::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
            let span_id = SpanID::new(&[1, 2, 3, 4, 5, 6, 7, 8]);
            spans.push(Span::build(trace_id, span_id, group.id.clone(), start_time).attributes(group.attributes.iter().map(get_attribute_name_value)).kind(group.span_kind.as_ref()).finish())
            // group.event 
                /// List of strings that specify the ids of event semantic conventions
    /// associated with this span semantic convention.
    /// Note: only valid if type is span
        }
    }
    spans
}




fn metrics(registry: &ReolvedRegistry) -> {
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
                match instrument {
                    InstrumentSpec::UpDownCounter => {
                        let up_down_counter = Metric::new_
                            .f64_up_down_counter(metric_name)
                            .with_unit(unit)
                            .with_description(description)
                            .build();
                        up_down_counter.add(1.0, &attributes);
                    }
                    InstrumentSpec::Counter => {
                        let counter = meter
                            .f64_counter(metric_name)
                            .with_unit(unit)
                            .with_description(description)
                            .build();
                        counter.add(1.0, &attributes);
                    }
                    InstrumentSpec::Gauge => {
                        let gauge = meter
                            .f64_gauge(metric_name)
                            .with_unit(unit)
                            .with_description(description)
                            .build();
                        gauge.record(1.0, &attributes);
                    }
                    InstrumentSpec::Histogram => {
                        let histogram = meter
                            .f64_histogram(metric_name)
                            .with_unit(unit)
                            .with_description(description)
                            .build();
                        histogram.record(1.0, &attributes);
                    }
                }
            }
        }
    }
    metrics
}

fn logs(registry: &ResolvedRegistry) -> Vec<LogRecord>{
    let log_records = vec![];
        for group in registry.groups.iter() {
            if group.r#type == GroupType::Event {
                let start_time = 1619712000000000000u64;
                let end_time = 1619712001000000000u64;
                let trace_id = TraceID::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
                let span_id = SpanID::new(&[1, 2, 3, 4, 5, 6, 7, 8]);
                LogRecord::build(2_000_000_000u64, , group.name.clone().unwrap_or("".to_owned)).attributes(group.attributes.iter().map(get_attribute_name_value)).body(common.)finish();
                log_records.push();
            }
        }
    log_records
}

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

fn otel_any_value(any_value: Option<&AnyValueSpec>) -> AnyValue {
    match any_value {
        Some(AnyValueSpec::Bool)
    }
}