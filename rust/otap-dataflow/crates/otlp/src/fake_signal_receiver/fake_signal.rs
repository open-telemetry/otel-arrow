// SPDX-License-Identifier: Apache-2.0

//!
//! The fake signal module provides methods for generating OTLP signals for testing
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object

use otel_arrow_rust::proto::opentelemetry::{
    common::v1::InstrumentationScope,
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
    metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
        metric::Data, number_data_point::Value,
    },
    trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData, span::SpanKind},
};

use crate::fake_signal_receiver::*;
use weaver_forge::ResolvedRegistry;
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

/// generate each span defined in the resolved registry
#[must_use]
fn spans(registry: &ResolvedRegistry) -> Vec<Span> {
    // Emit each span to the OTLP receiver.
    let spans = vec![];
    for group in registry.groups.iter() {
        if group.r#type == GroupType::Span {
            let start_time = current_time();
            let end_time = start_time + delay();

            // group.event_names has list of events
            spans.push(
                Span::build(gen_trace_id(), gen_span_id(), group.id.clone(), start_time)
                    .attributes(group.attributes.iter().map(get_attribute_name_value))
                    .kind(group.span_kind.as_ref())
                    .end_time_unix_nano(end_time)
                    .finish(),
            );
            // group.event
            // List of strings that specify the ids of event semantic conventions
            // associated with this span semantic convention.
            // Note: only valid if type is span
        }
    }
    spans
}

/// generate each metric defined in the resolved registry
#[must_use]
fn metrics(registry: &ReolvedRegistry) -> Vec<Metric> {
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

                        for _ in 0..4 {
                            datapoints.push(NumberDataPoint::build_double(insert timestmap, 1.0)
            .start_time_unix_nano()
            .attributes(attributes)
            .finish());
                        }
                        // is not monotonic
                        metrics.push(Metric::new_sum(metric_name, Sum::new(datapoints)).description(description).unit(unit));
                    }
                    InstrumentSpec::Counter => {
                        let datapoints = vec![];
                        for _ in 0..4 {
                            datapoints.push(NumberDataPoint::build_double(insert timestmap, 1.0)
            .start_time_unix_nano()
            .attributes(attributes)
            .finish());
                        }
                        -> sum
                        // is monotonic
                        metrics.push(Metric::new_sum(metric_name, Sum::new(datapoints)).description(description).unit(unit));
                    }
                    InstrumentSpec::Gauge => {
                        for _ in 0..4 {
                            datapoints.push(NumberDataPoint::build_double(insert timestmap, 1.0)
                            .start_time_unix_nano()
                            .attributes(attributes)
                            .finish());
                        }

                        metrics.push(Metric::new_gauge(metric_name, Gauge::new(datapoints)).description(description).unit(unit));
                    }
                    InstrumentSpec::Histogram => {

                             let mut datapoints = vec![];
                            for _ in 0..datapoint_count {
                                datapoints.push(
                                    HistogramDataPoint::build(
                                        get_time_unix_nano(),
                                        get_buckets_count(),
                                        get_explicit_bounds(),
                                    )
                                    .start_time_unix_nano(get_start_time_unix_nano())
                                    .attributes(attributes)
                                    .finish(),
                                )
                            }
                        metrics.push(Metric::new_histogram(metric_name, Histogram::new(datapoints)).description(description).unit(unit));

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
            // update body here
            let timestamp = gen_current_time();
            LogRecord::build(
                0u64,
                SeverityNumber::Unspecified,
                group.name.clone().unwrap_or("".to_owned),
            )
            .attributes(group.attributes.iter().map(get_attribute_name_value))
            .body(common).observed_time_unix_nano(timestamp).time_unix_nano(timestamp)
            .finish();
            log_records.push();
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

fn otel_any_value(any_value: Option<&AnyValueSpec>) -> AnyValue {
    match any_value {
        Some(AnyValueSpec::Bool)
    }
}


/ generate gauge datapoints
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