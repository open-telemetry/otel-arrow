// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the ViewMarshaler for converting VIEW messages to structured string reports.

use super::marshaler::ViewMarshaler;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, LogsData},
    metrics::v1::{
        Exemplar, ExponentialHistogramDataPoint, HistogramDataPoint, Metric, MetricsData,
        NumberDataPoint, SummaryDataPoint, exemplar::Value as ExemplarValue, metric::Data,
        number_data_point::Value as NumberValue,
    },
    trace::v1::{Span, TracesData},
};

use std::fmt::Write;

/// The Detailed Marshaler takes VIEW Views and converts them to a string by extracting their information
/// the finalized string will be the output for a detailed verbosity level
#[derive(Default)]
pub struct DetailedViewMarshaler;

impl ViewMarshaler for DetailedViewMarshaler {
    fn marshal_logs(&self, logs: LogsData) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            _ = writeln!(&mut report, "ResourceLog #{resource_index}",);
            _ = writeln!(
                &mut report,
                "   -> Resource SchemaURL: {schema_url}",
                schema_url = resource_log.schema_url
            );
            if let Some(resource) = &resource_log.resource {
                _ = writeln!(
                    &mut report,
                    "   -> Resource Attributes:{attributes}",
                    attributes = attributes_string_detailed(&resource.attributes, "      ->"),
                );
            }

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                _ = writeln!(&mut report, "   ScopeLogs #{scope_index}",);
                _ = writeln!(
                    &mut report,
                    "      -> ScopeLogs SchemaURL: {schema_url}",
                    schema_url = scope_log.schema_url
                );
                if let Some(scope) = &scope_log.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (record_index, log_record) in scope_log.log_records.iter().enumerate() {
                    _ = writeln!(&mut report, "      LogRecord #{record_index}",);
                    _ = writeln!(
                        &mut report,
                        "         -> ObservedTimestamp: {timestamp}",
                        timestamp = log_record.observed_time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "         -> Timestamp: {timestamp}",
                        timestamp = log_record.time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "         -> SeverityText: {severity}",
                        severity = log_record.severity_text
                    );
                    _ = writeln!(
                        &mut report,
                        "         -> SeverityNumber: {severity_number}",
                        severity_number = log_record.severity_number
                    );

                    if !log_record.event_name.is_empty() {
                        _ = writeln!(
                            &mut report,
                            "         -> EventName: {event_name}",
                            event_name = log_record.event_name
                        );
                    }
                    if let Some(body) = &log_record.body {
                        _ = writeln!(&mut report, "         -> Body: {body}");
                    }
                    _ = writeln!(
                        &mut report,
                        "         -> Attributes:{attributes}",
                        attributes =
                            attributes_string_detailed(&log_record.attributes, "            ->"),
                    );
                    if let Ok(trace_id) = std::str::from_utf8(&log_record.trace_id) {
                        _ = writeln!(&mut report, "         -> Trace ID: {trace_id}",);
                    }

                    if let Ok(span_id) = std::str::from_utf8(&log_record.span_id) {
                        _ = writeln!(&mut report, "         -> Span ID: {span_id}",);
                    }

                    _ = writeln!(
                        &mut report,
                        "         -> Flags: {flags}",
                        flags = log_record.flags
                    );
                }
            }
        }
        report
    }
    fn marshal_metrics(&self, metrics: MetricsData) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            _ = writeln!(&mut report, "ResourceMetric #{resource_index}",);
            _ = writeln!(
                &mut report,
                "   -> Resource SchemaURL: {schema_url}",
                schema_url = resource_metric.schema_url
            );

            if let Some(resource) = &resource_metric.resource {
                _ = writeln!(
                    &mut report,
                    "   -> Resource Attributes:{attributes}",
                    attributes = attributes_string_detailed(&resource.attributes, "         ->"),
                );
            }
            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                _ = writeln!(&mut report, "   ScopeMetrics #{scope_index}",);
                _ = writeln!(
                    &mut report,
                    "      -> ScopeMetrics SchemaURL: {schema_url}",
                    schema_url = scope_metric.schema_url
                );
                if let Some(scope) = &scope_metric.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (metric_index, metric) in scope_metric.metrics.iter().enumerate() {
                    _ = writeln!(&mut report, "      Metric #{metric_index}",);
                    _ = writeln!(&mut report, "         -> Name: {name}", name = metric.name);
                    _ = writeln!(
                        &mut report,
                        "         -> Description: {description}",
                        description = metric.description
                    );
                    _ = writeln!(&mut report, "         -> Unit: {unit}", unit = metric.unit);
                    if let Some(data) = &metric.data {
                        write_datapoints_detailed(&mut report, data, "         ");
                    }
                }
            }
        }
        report
    }
    fn marshal_traces(&self, traces: TracesData) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            _ = writeln!(&mut report, "ResourceSpan #{resource_index}",);
            _ = writeln!(
                &mut report,
                "   -> Resource SchemaURL: {schema_url}",
                schema_url = resource_span.schema_url
            );
            if let Some(resource) = &resource_span.resource {
                _ = writeln!(
                    &mut report,
                    "   -> Resource Attributes:{attributes}",
                    attributes = attributes_string_detailed(&resource.attributes, "      ->"),
                );
            }
            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                _ = writeln!(&mut report, "   ScopeSpans #{scope_index}",);
                _ = writeln!(
                    &mut report,
                    "      -> ScopeSpans SchemaURL: {schema_url}",
                    schema_url = scope_span.schema_url
                );
                if let Some(scope) = &scope_span.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (span_index, span) in scope_span.spans.iter().enumerate() {
                    _ = writeln!(&mut report, "      Span #{span_index}",);
                    if let Ok(trace_id) = std::str::from_utf8(&span.trace_id) {
                        _ = writeln!(&mut report, "         -> Trace ID: {trace_id}",);
                    }
                    if let Ok(parent_span_id) = std::str::from_utf8(&span.parent_span_id) {
                        _ = writeln!(&mut report, "         -> Parent ID: {parent_span_id}",);
                    }
                    if let Ok(span_id) = std::str::from_utf8(&span.span_id) {
                        _ = writeln!(&mut report, "         -> ID: {span_id}",);
                    }

                    _ = writeln!(&mut report, "         -> Name: {name}", name = span.name);
                    _ = writeln!(&mut report, "         -> Kind: {kind}", kind = span.kind);
                    if !span.trace_state.is_empty() {
                        _ = writeln!(
                            &mut report,
                            "         -> TraceState: {trace_state}",
                            trace_state = span.trace_state
                        );
                    }

                    _ = writeln!(
                        &mut report,
                        "         -> Start time: {start_time}",
                        start_time = span.start_time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "         -> End time: {end_time}",
                        end_time = span.end_time_unix_nano
                    );
                    if let Some(status) = &span.status {
                        _ = writeln!(
                            &mut report,
                            "         -> Status code: {status_code}",
                            status_code = status.code
                        );
                        _ = writeln!(
                            &mut report,
                            "         -> Status message: {status_message}",
                            status_message = status.message
                        );
                    }

                    _ = writeln!(
                        &mut report,
                        "         -> Attributes:{attributes}",
                        attributes = attributes_string_detailed(&span.attributes, "            ->"),
                    );

                    if !span.events.is_empty() {
                        _ = writeln!(&mut report, "         -> Events:");
                        for (event_index, event) in span.events.iter().enumerate() {
                            _ = writeln!(&mut report, "            SpanEvent #{event_index}",);
                            _ = writeln!(
                                &mut report,
                                "               -> Name: {name}",
                                name = event.name
                            );
                            _ = writeln!(
                                &mut report,
                                "               -> Timestamp: {timestamp}",
                                timestamp = event.time_unix_nano
                            );
                            _ = writeln!(
                                &mut report,
                                "               -> DroppedAttributesCount: {dropped_attributes_count}",
                                dropped_attributes_count = event.dropped_attributes_count
                            );
                            _ = writeln!(
                                &mut report,
                                "               -> Attributes:{attributes}",
                                attributes = attributes_string_detailed(
                                    &event.attributes,
                                    "                  ->"
                                ),
                            );
                        }
                    }

                    if !span.links.is_empty() {
                        _ = writeln!(&mut report, "         -> Links:");
                        for (index, link) in span.links.iter().enumerate() {
                            _ = writeln!(&mut report, "            SpanLink: #{index}");
                            if let Ok(trace_id) = std::str::from_utf8(&link.trace_id) {
                                _ = writeln!(&mut report, "               -> Trace ID: {trace_id}");
                            }
                            if let Ok(span_id) = std::str::from_utf8(&link.span_id) {
                                _ = writeln!(&mut report, "               -> Span ID: {span_id}");
                            }

                            _ = writeln!(
                                &mut report,
                                "               -> TraceState: {state}",
                                state = link.trace_state
                            );
                            _ = writeln!(
                                &mut report,
                                "               -> DroppedAttributesCount: {count}",
                                count = link.dropped_attributes_count
                            );
                            _ = writeln!(
                                &mut report,
                                "               -> Attributes:{attributes}",
                                attributes = attributes_string_detailed(
                                    &link.attributes,
                                    "                  ->"
                                ),
                            );
                        }
                    }
                }
            }
        }
        report
    }

    fn marshal_log_signal(&self, log_record: &LogRecord, index: usize) -> String {
        let mut report = String::new();
        _ = writeln!(&mut report, "LogRecord #{index}:",);
        _ = writeln!(
            &mut report,
            "   -> ObservedTimestamp: {timestamp}",
            timestamp = log_record.observed_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "   -> Timestamp: {timestamp}",
            timestamp = log_record.time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "   -> SeverityText: {severity}",
            severity = log_record.severity_text
        );
        _ = writeln!(
            &mut report,
            "   -> SeverityNumber: {severity_number}",
            severity_number = log_record.severity_number
        );

        if !log_record.event_name.is_empty() {
            _ = writeln!(
                &mut report,
                "   -> EventName: {event_name}",
                event_name = log_record.event_name
            );
        }
        if let Some(body) = &log_record.body {
            _ = writeln!(&mut report, "   -> Body: {body}");
        }
        _ = writeln!(
            &mut report,
            "   -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&log_record.attributes, "      ->"),
        );
        if let Ok(trace_id) = std::str::from_utf8(&log_record.trace_id) {
            _ = writeln!(&mut report, "   -> Trace ID: {trace_id}",);
        }

        if let Ok(span_id) = std::str::from_utf8(&log_record.span_id) {
            _ = writeln!(&mut report, "   -> Span ID: {span_id}",);
        }

        _ = writeln!(
            &mut report,
            "   -> Flags: {flags}",
            flags = log_record.flags
        );
        report
    }
    fn marshal_metric_signal(&self, metric: &Metric, index: usize) -> String {
        let mut report = String::new();

        _ = writeln!(&mut report, "Metric #{index}",);
        _ = writeln!(&mut report, "   -> Name: {name}", name = &metric.name);
        _ = writeln!(
            &mut report,
            "   -> Description: {description}",
            description = &metric.description
        );
        _ = writeln!(&mut report, "   -> Unit: {unit}", unit = &metric.unit);
        if let Some(data) = &metric.data {
            write_datapoints_detailed(&mut report, data, "   ");
        }

        report
    }

    fn marshal_span_signal(&self, span: &Span, index: usize) -> String {
        let mut report = String::new();

        _ = writeln!(&mut report, "Span #{index}:",);
        if let Ok(trace_id) = std::str::from_utf8(&span.trace_id) {
            _ = writeln!(&mut report, "   -> Trace ID: {trace_id}",);
        }
        if let Ok(parent_span_id) = std::str::from_utf8(&span.parent_span_id) {
            _ = writeln!(&mut report, "   -> Parent ID: {parent_span_id}",);
        }
        if let Ok(span_id) = std::str::from_utf8(&span.span_id) {
            _ = writeln!(&mut report, "   -> ID: {span_id}",);
        }

        _ = writeln!(&mut report, "   -> Name: {name}", name = span.name);
        _ = writeln!(&mut report, "   -> Kind: {kind}", kind = span.kind);
        if !span.trace_state.is_empty() {
            _ = writeln!(
                &mut report,
                "   -> TraceState: {trace_state}",
                trace_state = span.trace_state
            );
        }

        _ = writeln!(
            &mut report,
            "   -> Start time: {start_time}",
            start_time = span.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "   -> End time: {end_time}",
            end_time = span.end_time_unix_nano
        );
        if let Some(status) = &span.status {
            _ = writeln!(
                &mut report,
                "   -> Status code: {status_code}",
                status_code = status.code
            );
            _ = writeln!(
                &mut report,
                "   -> Status message: {status_message}",
                status_message = status.message
            );
        }

        _ = writeln!(
            &mut report,
            "   -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&span.attributes, "      ->"),
        );

        if !span.events.is_empty() {
            _ = writeln!(&mut report, "   -> Events:");
            for (event_index, event) in span.events.iter().enumerate() {
                _ = writeln!(&mut report, "      SpanEvent #{event_index}",);
                _ = writeln!(&mut report, "         -> Name: {name}", name = event.name);
                _ = writeln!(
                    &mut report,
                    "         -> Timestamp: {timestamp}",
                    timestamp = event.time_unix_nano
                );
                _ = writeln!(
                    &mut report,
                    "         -> DroppedAttributesCount: {dropped_attributes_count}",
                    dropped_attributes_count = event.dropped_attributes_count
                );
                _ = writeln!(
                    &mut report,
                    "         -> Attributes:{attributes}",
                    attributes = attributes_string_detailed(&event.attributes, "            ->"),
                );
            }
        }

        if !span.links.is_empty() {
            _ = writeln!(&mut report, "   -> Links:");
            for (index, link) in span.links.iter().enumerate() {
                _ = writeln!(&mut report, "      SpanLink: #{index}");
                if let Ok(trace_id) = std::str::from_utf8(&link.trace_id) {
                    _ = writeln!(&mut report, "         -> Trace ID: {trace_id}");
                }
                if let Ok(span_id) = std::str::from_utf8(&link.span_id) {
                    _ = writeln!(&mut report, "         -> Span ID: {span_id}");
                }

                _ = writeln!(
                    &mut report,
                    "         -> TraceState: {state}",
                    state = link.trace_state
                );
                _ = writeln!(
                    &mut report,
                    "         -> DroppedAttributesCount: {count}",
                    count = link.dropped_attributes_count
                );
                _ = writeln!(
                    &mut report,
                    "         -> Attributes:{attributes}",
                    attributes = attributes_string_detailed(&link.attributes, "            ->"),
                );
            }
        }

        report
    }
}

fn attributes_string_detailed(attributes: &[KeyValue], prefix: &str) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            _ = write!(
                &mut attribute_string,
                "\n{prefix} {key}: {value}",
                key = attribute.key,
            );
        }
    }

    attribute_string
}

fn write_datapoints_detailed(mut report: &mut String, data: &Data, prefix: &str) {
    match data {
        Data::Gauge(gauge) => {
            _ = writeln!(&mut report, "{prefix}-> DataType: Gauge");
            write_number_datapoints_detailed(report, &gauge.data_points, prefix);
        }
        Data::Sum(sum) => {
            _ = writeln!(&mut report, "{prefix}-> DataType: Sum");
            _ = writeln!(
                &mut report,
                "{prefix}-> IsMonotonic: {is_monotonic}",
                is_monotonic = sum.is_monotonic
            );
            _ = writeln!(
                &mut report,
                "{prefix}-> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = sum.aggregation_temporality
            );
            write_number_datapoints_detailed(report, &sum.data_points, prefix);
        }
        Data::Histogram(histogram) => {
            _ = writeln!(&mut report, "{prefix}-> DataType: Histogram");
            _ = writeln!(
                &mut report,
                "{prefix}-> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = histogram.aggregation_temporality
            );
            write_histogram_datapoints_detailed(report, &histogram.data_points, prefix);
        }
        Data::ExponentialHistogram(exponential_histogram) => {
            _ = writeln!(&mut report, "{prefix}-> DataType: Exponential Histogram");
            _ = writeln!(
                &mut report,
                "{prefix}-> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = exponential_histogram.aggregation_temporality
            );
            write_exponential_histogram_datapoints_detailed(
                report,
                &exponential_histogram.data_points,
                prefix,
            );
        }
        Data::Summary(summary) => {
            _ = writeln!(&mut report, "{prefix}-> DataType: Summary");
            write_summary_datapoints_detailed(report, &summary.data_points, prefix);
        }
    }
}

fn write_number_datapoints_detailed(
    mut report: &mut String,
    datapoints: &[NumberDataPoint],
    prefix: &str,
) {
    let attribute_prefix = format!("{prefix}            ->");
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "{prefix}   NumberDataPoints #{datapoint_index}",
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes, &attribute_prefix),
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        if let Some(value) = &datapoint.value {
            match value {
                NumberValue::AsInt(value) => {
                    _ = writeln!(&mut report, "{prefix}      -> Value: {value}");
                }
                NumberValue::AsDouble(value) => {
                    _ = writeln!(&mut report, "{prefix}      -> Value: {value}",);
                }
            }
        }

        write_exemplars(report, &datapoint.exemplars, prefix);
    }
}

fn write_histogram_datapoints_detailed(
    mut report: &mut String,
    datapoints: &[HistogramDataPoint],
    prefix: &str,
) {
    let attribute_prefix = format!("{prefix}            ->");
    for (index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(&mut report, "{prefix}   HistogramDataPoints #{index}");
        _ = writeln!(
            &mut report,
            "{prefix}      -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes, &attribute_prefix),
        );

        _ = writeln!(
            &mut report,
            "{prefix}      -> StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Count: {count}",
            count = datapoint.count
        );

        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "{prefix}      -> Sum: {sum}");
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "{prefix}      -> Min: {min}");
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "{prefix}      -> Max: {max}");
        }

        for (index, bound) in datapoint.explicit_bounds.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "{prefix}      -> ExplicitBound #{index}: {bound}",
            );
        }
        for (index, count) in datapoint.bucket_counts.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "{prefix}      -> Buckets #{index}, Count: {count}",
            );
        }

        write_exemplars(report, &datapoint.exemplars, prefix);
    }
}

fn write_exponential_histogram_datapoints_detailed(
    mut report: &mut String,
    datapoints: &[ExponentialHistogramDataPoint],
    prefix: &str,
) {
    let attribute_prefix = format!("{prefix}            ->");
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "{prefix}   ExponentialHistogramDataPoints #{datapoint_index}",
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes, &attribute_prefix),
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Count: {count}",
            count = datapoint.count
        );
        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "{prefix}      -> Sum: {sum}");
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "{prefix}      -> Min: {min}");
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "{prefix}      -> Max: {max}");
        }

        // calcualate the base -> 2^(2^(-scale)) -> e^(ln(2) * 2^(-scale))

        let base: f64 = (std::f64::consts::LN_2 * 2.0_f64.powf(-datapoint.scale as f64)).exp();

        if let Some(negative) = &datapoint.negative {
            let num_buckets = negative.bucket_counts.len();
            for position in 0..num_buckets {
                let updated_position = num_buckets - position - 1;

                let index: f64 = negative.offset as f64 + updated_position as f64;
                // calculate lower bound base^index
                let lower_bound = -(index * base).exp();
                // calculate upper bound base^(index + 1)
                let upper_bound = -((index + 1.0) * base).exp();
                _ = writeln!(
                    report,
                    "{prefix}      -> Bucket [{upper_bound}, {lower_bound}), Count: {count}",
                    count = negative.bucket_counts[updated_position]
                );
            }
        }
        if let Some(positive) = &datapoint.positive {
            let num_buckets = positive.bucket_counts.len();

            for position in 0..num_buckets {
                let index: f64 = positive.offset as f64 + position as f64;
                let lower_bound = (index * base).exp();
                let upper_bound = ((index + 1.0) * base).exp();
                _ = writeln!(
                    report,
                    "{prefix}      -> Bucket ({lower_bound}, {upper_bound}], Count: {count}",
                    count = positive.bucket_counts[position]
                );
            }
        }

        if datapoint.zero_count != 0 {
            _ = writeln!(
                &mut report,
                "{prefix}      -> Bucket [0, 0], Count: {count}",
                count = datapoint.zero_count
            );
        }

        write_exemplars(report, &datapoint.exemplars, prefix);
    }
}

fn write_summary_datapoints_detailed(
    mut report: &mut String,
    datapoints: &[SummaryDataPoint],
    prefix: &str,
) {
    let attribute_prefix = format!("{prefix}            ->");
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "{prefix}   SummaryDataPoints #{datapoint_index}",
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Attributes:{attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes, &attribute_prefix),
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Count: {count}",
            count = datapoint.count
        );
        _ = writeln!(
            &mut report,
            "{prefix}      -> Sum: {sum}",
            sum = datapoint.sum
        );
        for (quantile_index, quantile) in datapoint.quantile_values.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "{prefix}      -> QuantileValue #{quantile_index}: Quantile {quantile}, Value {value}",
                quantile = quantile.quantile,
                value = quantile.value
            );
        }
    }
}

fn write_exemplars(mut report: &mut String, exemplars: &[Exemplar], prefix: &str) {
    if !exemplars.is_empty() {
        _ = writeln!(&mut report, "{prefix}      -> Exemplars:");
        let attribute_prefix = format!("{prefix}               ->");
        for (exemplar_index, exemplar) in exemplars.iter().enumerate() {
            _ = writeln!(&mut report, "{prefix}         Exemplar #{exemplar_index}",);
            if let Ok(trace_id) = std::str::from_utf8(&exemplar.trace_id) {
                _ = writeln!(&mut report, "{prefix}            -> Trace ID: {trace_id}",);
            }
            if let Ok(span_id) = std::str::from_utf8(&exemplar.span_id) {
                _ = writeln!(&mut report, "{prefix}            -> Span ID: {span_id}",);
            }
            _ = writeln!(
                &mut report,
                "{prefix}            -> Timestamp: {timestamp}",
                timestamp = exemplar.time_unix_nano
            );
            if let Some(value) = &exemplar.value {
                match value {
                    ExemplarValue::AsInt(value) => {
                        _ = writeln!(&mut report, "{prefix}            -> Value: {value}",);
                    }
                    ExemplarValue::AsDouble(value) => {
                        _ = writeln!(&mut report, "{prefix}            -> Value: {value}",);
                    }
                }
            }
            _ = writeln!(
                &mut report,
                "{prefix}            -> FilteredAttributes:{attributes}",
                attributes =
                    attributes_string_detailed(&exemplar.filtered_attributes, &attribute_prefix)
            );
        }
    }
}

fn write_instrumentation_scope(mut report: &mut String, scope: &InstrumentationScope) {
    _ = writeln!(
        &mut report,
        "      -> Instrumentation Scope {name} @{version}",
        name = scope.name,
        version = scope.version
    );
    _ = writeln!(
        &mut report,
        "      -> Instrumentation Scope Attributes:{attributes}",
        attributes = attributes_string_detailed(&scope.attributes, "         ->")
    );
}

#[cfg(test)]
mod tests {

    use crate::debug_processor::detailed_marshaler::DetailedViewMarshaler;
    use crate::debug_processor::marshaler::ViewMarshaler;
    use otel_arrow_rust::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        metrics::v1::{
            Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
            HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
            ScopeMetrics, Sum, Summary, SummaryDataPoint,
            exponential_histogram_data_point::Buckets, summary_data_point::ValueAtQuantile,
        },
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span, Status, TracesData, span::Event, span::Link,
            status::StatusCode,
        },
    };

    #[test]
    fn test_marshal_traces() {
        let trace = TracesData::new(vec![
            ResourceSpans::build(
                Resource::build(vec![KeyValue::new(
                    "ip",
                    AnyValue::new_string("192.168.0.1"),
                )])
                .dropped_attributes_count(123u32),
            )
            .schema_url("http://schema.opentelemetry.io")
            .scope_spans(vec![
                ScopeSpans::build(
                    InstrumentationScope::build("library")
                        .version("v1")
                        .attributes(vec![KeyValue::new(
                            "hostname",
                            AnyValue::new_string("host5.retailer.com"),
                        )])
                        .finish(),
                )
                .schema_url("http://schema.opentelemetry.io")
                .spans(vec![
                    Span::build(
                        Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                        Vec::from("7271ee06d7e5925f".as_bytes()),
                        "user-account",
                        1647648000000000106u64,
                    )
                    .attributes(vec![KeyValue::new(
                        "hostname",
                        AnyValue::new_string("host4.gov"),
                    )])
                    .parent_span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                    .end_time_unix_nano(1647648000000000104u64)
                    .status(Status::new("Error", StatusCode::Error))
                    .trace_state("ended")
                    .events(vec![
                        Event::build("message-receive", 1647648000000000108u64)
                            .attributes(vec![KeyValue::new(
                                "hostname",
                                AnyValue::new_string("host5.retailer.com"),
                            )])
                            .dropped_attributes_count(0u32)
                            .finish(),
                    ])
                    .links(vec![
                        Link::build(
                            Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                            Vec::from("7271ee06d7e5925f".as_bytes()),
                        )
                        .trace_state("ended")
                        .dropped_attributes_count(0u32)
                        .attributes(vec![KeyValue::new(
                            "hostname",
                            AnyValue::new_string("host2.org"),
                        )])
                        .finish(),
                    ])
                    .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);
        let marshaler = DetailedViewMarshaler;
        let marshaled_trace = marshaler.marshal_traces(trace);
        let mut output_lines = Vec::new();
        for line in marshaled_trace.lines() {
            output_lines.push(line);
        }
        assert_eq!(output_lines[0], "ResourceSpan #0");
        assert_eq!(
            output_lines[1],
            "   -> Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "   -> Resource Attributes:");
        assert_eq!(output_lines[3], "      -> ip: 192.168.0.1");
        assert_eq!(output_lines[4], "   ScopeSpans #0");
        assert_eq!(
            output_lines[5],
            "      -> ScopeSpans SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(
            output_lines[6],
            "      -> Instrumentation Scope library @v1"
        );
        assert_eq!(
            output_lines[7],
            "      -> Instrumentation Scope Attributes:"
        );
        assert_eq!(output_lines[8], "         -> hostname: host5.retailer.com");
        assert_eq!(output_lines[9], "      Span #0");
        assert_eq!(
            output_lines[10],
            "         -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[11], "         -> Parent ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[12], "         -> ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[13], "         -> Name: user-account");
        assert_eq!(output_lines[14], "         -> Kind: 0");
        assert_eq!(output_lines[15], "         -> TraceState: ended");
        assert_eq!(
            output_lines[16],
            "         -> Start time: 1647648000000000106"
        );
        assert_eq!(
            output_lines[17],
            "         -> End time: 1647648000000000104"
        );
        assert_eq!(output_lines[18], "         -> Status code: 2");
        assert_eq!(output_lines[19], "         -> Status message: Error");
        assert_eq!(output_lines[20], "         -> Attributes:");
        assert_eq!(output_lines[21], "            -> hostname: host4.gov");
        assert_eq!(output_lines[22], "         -> Events:");
        assert_eq!(output_lines[23], "            SpanEvent #0");
        assert_eq!(output_lines[24], "               -> Name: message-receive");
        assert_eq!(
            output_lines[25],
            "               -> Timestamp: 1647648000000000108"
        );
        assert_eq!(
            output_lines[26],
            "               -> DroppedAttributesCount: 0"
        );
        assert_eq!(output_lines[27], "               -> Attributes:");
        assert_eq!(
            output_lines[28],
            "                  -> hostname: host5.retailer.com"
        );
        assert_eq!(output_lines[29], "         -> Links:");
        assert_eq!(output_lines[30], "            SpanLink: #0");
        assert_eq!(
            output_lines[31],
            "               -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[32],
            "               -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(output_lines[33], "               -> TraceState: ended");
        assert_eq!(
            output_lines[34],
            "               -> DroppedAttributesCount: 0"
        );
        assert_eq!(output_lines[35], "               -> Attributes:");
        assert_eq!(output_lines[36], "                  -> hostname: host2.org");
    }

    #[test]
    fn test_marshal_metrics() {
        let metric = MetricsData::new(vec![
            ResourceMetrics::build(
                Resource::build(vec![KeyValue::new(
                    "ip",
                    AnyValue::new_string("192.168.0.2"),
                )])
                .finish(),
            )
            .scope_metrics(vec![
                ScopeMetrics::build(
                    InstrumentationScope::build("library")
                        .version("v1")
                        .attributes(vec![KeyValue::new(
                            "instrumentation_scope_k1",
                            AnyValue::new_string("k1 value"),
                        )])
                        .finish(),
                )
                .schema_url("http://schema.opentelemetry.io")
                .metrics(vec![
                    Metric::build_gauge(
                        "system.cpu.time",
                        Gauge::new(vec![
                            NumberDataPoint::build_int(1663718400000001400u64, 0i64)
                                .start_time_unix_nano(1650499200000000100u64)
                                .flags(1u32)
                                .finish(),
                        ]),
                    )
                    .description("time cpu has ran")
                    .unit("s")
                    .metadata(vec![])
                    .finish(),
                    Metric::build_exponential_histogram(
                        "system.cpu.time",
                        ExponentialHistogram::new(
                            4,
                            vec![
                                ExponentialHistogramDataPoint::build(
                                    1663718400000001400u64,
                                    1,
                                    Buckets::new(0, vec![0]),
                                )
                                .attributes(vec![KeyValue::new(
                                    "freq",
                                    AnyValue::new_string("3GHz"),
                                )])
                                .exemplars(vec![
                                    Exemplar::build_double(1663718400000001400u64, 22.2)
                                        .filtered_attributes(vec![KeyValue::new(
                                            "cpu",
                                            AnyValue::new_string("0"),
                                        )])
                                        .trace_id(Vec::from(
                                            "4327e52011a22f9662eac217d77d1ec0".as_bytes(),
                                        ))
                                        .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                        .finish(),
                                ])
                                .start_time_unix_nano(1650499200000000000u64)
                                .count(0u64)
                                .sum(56)
                                .negative(Buckets::new(0, vec![0]))
                                .flags(5u32)
                                .min(12)
                                .max(100.1)
                                .zero_threshold(0.0)
                                .finish(),
                            ],
                        ),
                    )
                    .description("time cpu has ran")
                    .unit("s")
                    .finish(),
                    Metric::build_histogram(
                        "system.cpu.time",
                        Histogram::new(
                            4,
                            vec![
                                HistogramDataPoint::build(
                                    1663718400000001400u64,
                                    vec![0],
                                    vec![94.17542094619048, 65.66722851519177],
                                )
                                .attributes(vec![KeyValue::new(
                                    "freq",
                                    AnyValue::new_string("3GHz"),
                                )])
                                .start_time_unix_nano(1650499200000000000u64)
                                .count(0u64)
                                .exemplars(vec![
                                    Exemplar::build_double(1663718400000001400u64, 22.2)
                                        .filtered_attributes(vec![KeyValue::new(
                                            "cpu",
                                            AnyValue::new_string("0"),
                                        )])
                                        .trace_id(Vec::from(
                                            "4327e52011a22f9662eac217d77d1ec0".as_bytes(),
                                        ))
                                        .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                        .finish(),
                                ])
                                .sum(56)
                                .flags(0u32)
                                .min(12)
                                .max(100.1)
                                .finish(),
                            ],
                        ),
                    )
                    .description("time cpu has ran")
                    .unit("s")
                    .finish(),
                    Metric::build_sum(
                        "system.cpu.time",
                        Sum::new(
                            4,
                            true,
                            vec![
                                NumberDataPoint::build_int(1663718400000001400u64, 0i64)
                                    .start_time_unix_nano(1650499200000000000u64)
                                    .attributes(vec![KeyValue::new(
                                        "cpu_logical_processors",
                                        AnyValue::new_string("8"),
                                    )])
                                    .exemplars(vec![
                                        Exemplar::build_double(1663718400000001400u64, 22.2)
                                            .filtered_attributes(vec![KeyValue::new(
                                                "************",
                                                AnyValue::new_bool(true),
                                            )])
                                            .trace_id(Vec::from(
                                                "4327e52011a22f9662eac217d77d1ec0".as_bytes(),
                                            ))
                                            .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                            .finish(),
                                    ])
                                    .finish(),
                            ],
                        ),
                    )
                    .description("time cpu has ran")
                    .unit("s")
                    .finish(),
                    Metric::build_summary(
                        "system.cpu.time",
                        Summary::new(vec![
                            SummaryDataPoint::build(
                                1663718400000001400u64,
                                vec![ValueAtQuantile::new(0., 0.)],
                            )
                            .attributes(vec![KeyValue::new("cpu_cores", AnyValue::new_string("4"))])
                            .start_time_unix_nano(1650499200000000100u64)
                            .count(0u64)
                            .sum(56.0)
                            .flags(0u32)
                            .finish(),
                        ]),
                    )
                    .description("time cpu has ran")
                    .unit("s")
                    .finish(),
                ])
                .finish(),
            ])
            .schema_url("http://schema.opentelemetry.io")
            .finish(),
        ]);
        let marshaler = DetailedViewMarshaler;
        let marshaled_metric = marshaler.marshal_metrics(metric);
        let mut output_lines = Vec::new();
        for line in marshaled_metric.lines() {
            output_lines.push(line);
        }
        assert_eq!(output_lines[0], "ResourceMetric #0");
        assert_eq!(
            output_lines[1],
            "   -> Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "   -> Resource Attributes:");
        assert_eq!(output_lines[3], "         -> ip: 192.168.0.2");
        assert_eq!(output_lines[4], "   ScopeMetrics #0");
        assert_eq!(
            output_lines[5],
            "      -> ScopeMetrics SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(
            output_lines[6],
            "      -> Instrumentation Scope library @v1"
        );
        assert_eq!(
            output_lines[7],
            "      -> Instrumentation Scope Attributes:"
        );
        assert_eq!(
            output_lines[8],
            "         -> instrumentation_scope_k1: k1 value"
        );
        assert_eq!(output_lines[9], "      Metric #0");
        assert_eq!(output_lines[10], "         -> Name: system.cpu.time");
        assert_eq!(
            output_lines[11],
            "         -> Description: time cpu has ran"
        );
        assert_eq!(output_lines[12], "         -> Unit: s");
        assert_eq!(output_lines[13], "         -> DataType: Gauge");
        assert_eq!(output_lines[14], "            NumberDataPoints #0");
        assert_eq!(output_lines[15], "               -> Attributes:");
        assert_eq!(
            output_lines[16],
            "               -> StartTimestamp: 1650499200000000100"
        );
        assert_eq!(
            output_lines[17],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[18], "               -> Value: 0");
        assert_eq!(output_lines[19], "      Metric #1");
        assert_eq!(output_lines[20], "         -> Name: system.cpu.time");
        assert_eq!(
            output_lines[21],
            "         -> Description: time cpu has ran"
        );
        assert_eq!(output_lines[22], "         -> Unit: s");
        assert_eq!(
            output_lines[23],
            "         -> DataType: Exponential Histogram"
        );
        assert_eq!(output_lines[24], "         -> AggregationTemporality: 4");
        assert_eq!(
            output_lines[25],
            "            ExponentialHistogramDataPoints #0"
        );
        assert_eq!(output_lines[26], "               -> Attributes:");
        assert_eq!(output_lines[27], "                     -> freq: 3GHz");
        assert_eq!(
            output_lines[28],
            "               -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[29],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[30], "               -> Count: 0");
        assert_eq!(output_lines[31], "               -> Sum: 56");
        assert_eq!(output_lines[32], "               -> Min: 12");
        assert_eq!(output_lines[33], "               -> Max: 100.1");
        assert_eq!(
            output_lines[34],
            "               -> Bucket [-4.113250378782927, -1), Count: 0"
        );
        assert_eq!(
            output_lines[35],
            "               -> Bucket (1, 4.113250378782927], Count: 0"
        );
        assert_eq!(output_lines[36], "               -> Exemplars:");
        assert_eq!(output_lines[37], "                  Exemplar #0");
        assert_eq!(
            output_lines[38],
            "                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[39],
            "                     -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[40],
            "                     -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[41], "                     -> Value: 22.2");
        assert_eq!(
            output_lines[42],
            "                     -> FilteredAttributes:"
        );
        assert_eq!(output_lines[43], "                        -> cpu: 0");
        assert_eq!(output_lines[44], "      Metric #2");
        assert_eq!(output_lines[45], "         -> Name: system.cpu.time");
        assert_eq!(
            output_lines[46],
            "         -> Description: time cpu has ran"
        );
        assert_eq!(output_lines[47], "         -> Unit: s");
        assert_eq!(output_lines[48], "         -> DataType: Histogram");
        assert_eq!(output_lines[49], "         -> AggregationTemporality: 4");
        assert_eq!(output_lines[50], "            HistogramDataPoints #0");
        assert_eq!(output_lines[51], "               -> Attributes:");
        assert_eq!(output_lines[52], "                     -> freq: 3GHz");
        assert_eq!(
            output_lines[53],
            "               -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[54],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[55], "               -> Count: 0");
        assert_eq!(output_lines[56], "               -> Sum: 56");
        assert_eq!(output_lines[57], "               -> Min: 12");
        assert_eq!(output_lines[58], "               -> Max: 100.1");
        assert_eq!(
            output_lines[59],
            "               -> ExplicitBound #0: 94.17542094619048"
        );
        assert_eq!(
            output_lines[60],
            "               -> ExplicitBound #1: 65.66722851519177"
        );
        assert_eq!(output_lines[61], "               -> Buckets #0, Count: 0");
        assert_eq!(output_lines[62], "               -> Exemplars:");
        assert_eq!(output_lines[63], "                  Exemplar #0");
        assert_eq!(
            output_lines[64],
            "                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[65],
            "                     -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[66],
            "                     -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[67], "                     -> Value: 22.2");
        assert_eq!(
            output_lines[68],
            "                     -> FilteredAttributes:"
        );
        assert_eq!(output_lines[69], "                        -> cpu: 0");
        assert_eq!(output_lines[70], "      Metric #3");
        assert_eq!(output_lines[71], "         -> Name: system.cpu.time");
        assert_eq!(
            output_lines[72],
            "         -> Description: time cpu has ran"
        );
        assert_eq!(output_lines[73], "         -> Unit: s");
        assert_eq!(output_lines[74], "         -> DataType: Sum");
        assert_eq!(output_lines[75], "         -> IsMonotonic: true");
        assert_eq!(output_lines[76], "         -> AggregationTemporality: 4");
        assert_eq!(output_lines[77], "            NumberDataPoints #0");
        assert_eq!(output_lines[78], "               -> Attributes:");
        assert_eq!(
            output_lines[79],
            "                     -> cpu_logical_processors: 8"
        );
        assert_eq!(
            output_lines[80],
            "               -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[81],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[82], "               -> Value: 0");
        assert_eq!(output_lines[83], "               -> Exemplars:");
        assert_eq!(output_lines[84], "                  Exemplar #0");
        assert_eq!(
            output_lines[85],
            "                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[86],
            "                     -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[87],
            "                     -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[88], "                     -> Value: 22.2");
        assert_eq!(
            output_lines[89],
            "                     -> FilteredAttributes:"
        );
        assert_eq!(
            output_lines[90],
            "                        -> ************: true"
        );
        assert_eq!(output_lines[91], "      Metric #4");
        assert_eq!(output_lines[92], "         -> Name: system.cpu.time");
        assert_eq!(
            output_lines[93],
            "         -> Description: time cpu has ran"
        );
        assert_eq!(output_lines[94], "         -> Unit: s");
        assert_eq!(output_lines[95], "         -> DataType: Summary");
        assert_eq!(output_lines[96], "            SummaryDataPoints #0");
        assert_eq!(output_lines[97], "               -> Attributes:");
        assert_eq!(output_lines[98], "                     -> cpu_cores: 4");
        assert_eq!(
            output_lines[99],
            "               -> StartTimestamp: 1650499200000000100"
        );
        assert_eq!(
            output_lines[100],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[101], "               -> Count: 0");
        assert_eq!(output_lines[102], "               -> Sum: 56");
        assert_eq!(
            output_lines[103],
            "               -> QuantileValue #0: Quantile 0, Value 0"
        );
    }
    #[test]
    fn test_marshal_logs() {
        let logs = LogsData::new(vec![
            ResourceLogs::build(Resource::build(vec![KeyValue::new(
                "version",
                AnyValue::new_string("2.0"),
            )]))
            .schema_url("http://schema.opentelemetry.io")
            .scope_logs(vec![
                ScopeLogs::build(
                    InstrumentationScope::build("library")
                        .version("v1")
                        .attributes(vec![KeyValue::new(
                            "hostname",
                            AnyValue::new_string("host5.retailer.com"),
                        )])
                        .finish(),
                )
                .schema_url("http://schema.opentelemetry.io")
                .log_records(vec![
                    LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                        .observed_time_unix_nano(1663718400000001300u64)
                        .severity_text("INFO")
                        .trace_id(Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()))
                        .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                        .attributes(vec![KeyValue::new(
                            "hostname",
                            AnyValue::new_string("host3.thedomain.edu"),
                        )])
                        .flags(LogRecordFlags::TraceFlagsMask)
                        .body(AnyValue::new_string(
                            "Sint impedit non ut eligendi nisi neque harum maxime adipisci.",
                        ))
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);
        let marshaler = DetailedViewMarshaler;
        let marshaled_logs = marshaler.marshal_logs(logs);
        let mut output_lines = Vec::new();
        for line in marshaled_logs.lines() {
            output_lines.push(line);
        }

        assert_eq!(output_lines[0], "ResourceLog #0");
        assert_eq!(
            output_lines[1],
            "   -> Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "   -> Resource Attributes:");
        assert_eq!(output_lines[3], "      -> version: 2.0");
        assert_eq!(output_lines[4], "   ScopeLogs #0");
        assert_eq!(
            output_lines[5],
            "      -> ScopeLogs SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(
            output_lines[6],
            "      -> Instrumentation Scope library @v1"
        );
        assert_eq!(
            output_lines[7],
            "      -> Instrumentation Scope Attributes:"
        );
        assert_eq!(output_lines[8], "         -> hostname: host5.retailer.com");
        assert_eq!(output_lines[9], "      LogRecord #0");
        assert_eq!(
            output_lines[10],
            "         -> ObservedTimestamp: 1663718400000001300"
        );
        assert_eq!(output_lines[11], "         -> Timestamp: 2000000000");
        assert_eq!(output_lines[12], "         -> SeverityText: INFO");
        assert_eq!(output_lines[13], "         -> SeverityNumber: 9");
        assert_eq!(output_lines[14], "         -> EventName: event1");
        assert_eq!(
            output_lines[15],
            "         -> Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci."
        );
        assert_eq!(output_lines[16], "         -> Attributes:");
        assert_eq!(
            output_lines[17],
            "            -> hostname: host3.thedomain.edu"
        );
        assert_eq!(
            output_lines[18],
            "         -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[19], "         -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[20], "         -> Flags: 255");
    }

    #[test]
    fn test_marshal_log_signal() {
        let log = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
            .observed_time_unix_nano(1663718400000001300u64)
            .severity_text("INFO")
            .trace_id(Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()))
            .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
            .attributes(vec![KeyValue::new(
                "hostname",
                AnyValue::new_string("host3.thedomain.edu"),
            )])
            .flags(LogRecordFlags::TraceFlagsMask)
            .body(AnyValue::new_string(
                "Sint impedit non ut eligendi nisi neque harum maxime adipisci.",
            ))
            .finish();
        let marshaler = DetailedViewMarshaler;
        let marshaled_logs = marshaler.marshal_log_signal(&log, 0);
        let mut output_lines = Vec::new();
        for line in marshaled_logs.lines() {
            output_lines.push(line);
        }
        assert_eq!(output_lines[0], "LogRecord #0:");
        assert_eq!(
            output_lines[1],
            "   -> ObservedTimestamp: 1663718400000001300"
        );
        assert_eq!(output_lines[2], "   -> Timestamp: 2000000000");
        assert_eq!(output_lines[3], "   -> SeverityText: INFO");
        assert_eq!(output_lines[4], "   -> SeverityNumber: 9");
        assert_eq!(output_lines[5], "   -> EventName: event1");
        assert_eq!(
            output_lines[6],
            "   -> Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci."
        );
        assert_eq!(output_lines[7], "   -> Attributes:");
        assert_eq!(output_lines[8], "      -> hostname: host3.thedomain.edu");
        assert_eq!(
            output_lines[9],
            "   -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[10], "   -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[11], "   -> Flags: 255");
    }

    #[test]
    fn test_marshal_span_signal() {
        let span = Span::build(
            Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
            Vec::from("7271ee06d7e5925f".as_bytes()),
            "user-account",
            1647648000000000106u64,
        )
        .attributes(vec![KeyValue::new(
            "hostname",
            AnyValue::new_string("host4.gov"),
        )])
        .parent_span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
        .end_time_unix_nano(1647648000000000104u64)
        .status(Status::new("Error", StatusCode::Error))
        .trace_state("ended")
        .events(vec![
            Event::build("message-receive", 1647648000000000108u64)
                .attributes(vec![KeyValue::new(
                    "hostname",
                    AnyValue::new_string("host5.retailer.com"),
                )])
                .dropped_attributes_count(0u32)
                .finish(),
        ])
        .links(vec![
            Link::build(
                Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()),
                Vec::from("7271ee06d7e5925f".as_bytes()),
            )
            .trace_state("ended")
            .dropped_attributes_count(0u32)
            .attributes(vec![KeyValue::new(
                "hostname",
                AnyValue::new_string("host2.org"),
            )])
            .finish(),
        ])
        .finish();

        let marshaler = DetailedViewMarshaler;
        let marshaled_trace = marshaler.marshal_span_signal(&span, 0);
        let mut output_lines = Vec::new();
        for line in marshaled_trace.lines() {
            output_lines.push(line);
        }
        assert_eq!(output_lines[0], "Span #0:");
        assert_eq!(
            output_lines[1],
            "   -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[2], "   -> Parent ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[3], "   -> ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[4], "   -> Name: user-account");
        assert_eq!(output_lines[5], "   -> Kind: 0");
        assert_eq!(output_lines[6], "   -> TraceState: ended");
        assert_eq!(output_lines[7], "   -> Start time: 1647648000000000106");
        assert_eq!(output_lines[8], "   -> End time: 1647648000000000104");
        assert_eq!(output_lines[9], "   -> Status code: 2");
        assert_eq!(output_lines[10], "   -> Status message: Error");
        assert_eq!(output_lines[11], "   -> Attributes:");
        assert_eq!(output_lines[12], "      -> hostname: host4.gov");
        assert_eq!(output_lines[13], "   -> Events:");
        assert_eq!(output_lines[14], "      SpanEvent #0");
        assert_eq!(output_lines[15], "         -> Name: message-receive");
        assert_eq!(
            output_lines[16],
            "         -> Timestamp: 1647648000000000108"
        );
        assert_eq!(output_lines[17], "         -> DroppedAttributesCount: 0");
        assert_eq!(output_lines[18], "         -> Attributes:");
        assert_eq!(
            output_lines[19],
            "            -> hostname: host5.retailer.com"
        );
        assert_eq!(output_lines[20], "   -> Links:");
        assert_eq!(output_lines[21], "      SpanLink: #0");
        assert_eq!(
            output_lines[22],
            "         -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[23], "         -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[24], "         -> TraceState: ended");
        assert_eq!(output_lines[25], "         -> DroppedAttributesCount: 0");
        assert_eq!(output_lines[26], "         -> Attributes:");
        assert_eq!(output_lines[27], "            -> hostname: host2.org");
    }

    #[test]
    fn test_marshal_metric_signal() {
        let metrics = vec![
            Metric::build_gauge(
                "system.cpu.time",
                Gauge::new(vec![
                    NumberDataPoint::build_int(1663718400000001400u64, 0i64)
                        .start_time_unix_nano(1650499200000000100u64)
                        .flags(1u32)
                        .finish(),
                ]),
            )
            .description("time cpu has ran")
            .unit("s")
            .metadata(vec![])
            .finish(),
            Metric::build_exponential_histogram(
                "system.cpu.time",
                ExponentialHistogram::new(
                    4,
                    vec![
                        ExponentialHistogramDataPoint::build(
                            1663718400000001400u64,
                            1,
                            Buckets::new(0, vec![0]),
                        )
                        .attributes(vec![KeyValue::new("freq", AnyValue::new_string("3GHz"))])
                        .exemplars(vec![
                            Exemplar::build_double(1663718400000001400u64, 22.2)
                                .filtered_attributes(vec![KeyValue::new(
                                    "cpu",
                                    AnyValue::new_string("0"),
                                )])
                                .trace_id(Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()))
                                .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                .finish(),
                        ])
                        .start_time_unix_nano(1650499200000000000u64)
                        .count(0u64)
                        .sum(56)
                        .negative(Buckets::new(0, vec![0]))
                        .flags(5u32)
                        .min(12)
                        .max(100.1)
                        .zero_threshold(0.0)
                        .finish(),
                    ],
                ),
            )
            .description("time cpu has ran")
            .unit("s")
            .finish(),
            Metric::build_histogram(
                "system.cpu.time",
                Histogram::new(
                    4,
                    vec![
                        HistogramDataPoint::build(
                            1663718400000001400u64,
                            vec![0],
                            vec![94.17542094619048, 65.66722851519177],
                        )
                        .attributes(vec![KeyValue::new("freq", AnyValue::new_string("3GHz"))])
                        .start_time_unix_nano(1650499200000000000u64)
                        .count(0u64)
                        .exemplars(vec![
                            Exemplar::build_double(1663718400000001400u64, 22.2)
                                .filtered_attributes(vec![KeyValue::new(
                                    "cpu",
                                    AnyValue::new_string("0"),
                                )])
                                .trace_id(Vec::from("4327e52011a22f9662eac217d77d1ec0".as_bytes()))
                                .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                .finish(),
                        ])
                        .sum(56)
                        .flags(0u32)
                        .min(12)
                        .max(100.1)
                        .finish(),
                    ],
                ),
            )
            .description("time cpu has ran")
            .unit("s")
            .finish(),
            Metric::build_sum(
                "system.cpu.time",
                Sum::new(
                    4,
                    true,
                    vec![
                        NumberDataPoint::build_int(1663718400000001400u64, 0i64)
                            .start_time_unix_nano(1650499200000000000u64)
                            .attributes(vec![KeyValue::new(
                                "cpu_logical_processors",
                                AnyValue::new_string("8"),
                            )])
                            .exemplars(vec![
                                Exemplar::build_double(1663718400000001400u64, 22.2)
                                    .filtered_attributes(vec![KeyValue::new(
                                        "************",
                                        AnyValue::new_bool(true),
                                    )])
                                    .trace_id(Vec::from(
                                        "4327e52011a22f9662eac217d77d1ec0".as_bytes(),
                                    ))
                                    .span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                                    .finish(),
                            ])
                            .finish(),
                    ],
                ),
            )
            .description("time cpu has ran")
            .unit("s")
            .finish(),
            Metric::build_summary(
                "system.cpu.time",
                Summary::new(vec![
                    SummaryDataPoint::build(
                        1663718400000001400u64,
                        vec![ValueAtQuantile::new(0., 0.)],
                    )
                    .attributes(vec![KeyValue::new("cpu_cores", AnyValue::new_string("4"))])
                    .start_time_unix_nano(1650499200000000100u64)
                    .count(0u64)
                    .sum(56.0)
                    .flags(0u32)
                    .finish(),
                ]),
            )
            .description("time cpu has ran")
            .unit("s")
            .finish(),
        ];
        let marshaler = DetailedViewMarshaler;
        let mut output_lines = Vec::new();
        for (index, metric) in metrics.iter().enumerate() {
            let marshaled_metrics = marshaler.marshal_metric_signal(metric, index);
            for line in marshaled_metrics.lines() {
                output_lines.push(line.to_owned());
            }
        }
        assert_eq!(output_lines[0], "Metric #0");
        assert_eq!(output_lines[1], "   -> Name: system.cpu.time");
        assert_eq!(output_lines[2], "   -> Description: time cpu has ran");
        assert_eq!(output_lines[3], "   -> Unit: s");
        assert_eq!(output_lines[4], "   -> DataType: Gauge");
        assert_eq!(output_lines[5], "      NumberDataPoints #0");
        assert_eq!(output_lines[6], "         -> Attributes:");
        assert_eq!(
            output_lines[7],
            "         -> StartTimestamp: 1650499200000000100"
        );
        assert_eq!(
            output_lines[8],
            "         -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[9], "         -> Value: 0");
        assert_eq!(output_lines[10], "Metric #1");
        assert_eq!(output_lines[11], "   -> Name: system.cpu.time");
        assert_eq!(output_lines[12], "   -> Description: time cpu has ran");
        assert_eq!(output_lines[13], "   -> Unit: s");
        assert_eq!(output_lines[14], "   -> DataType: Exponential Histogram");
        assert_eq!(output_lines[15], "   -> AggregationTemporality: 4");
        assert_eq!(output_lines[16], "      ExponentialHistogramDataPoints #0");
        assert_eq!(output_lines[17], "         -> Attributes:");
        assert_eq!(output_lines[18], "               -> freq: 3GHz");
        assert_eq!(
            output_lines[19],
            "         -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[20],
            "         -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[21], "         -> Count: 0");
        assert_eq!(output_lines[22], "         -> Sum: 56");
        assert_eq!(output_lines[23], "         -> Min: 12");
        assert_eq!(output_lines[24], "         -> Max: 100.1");
        assert_eq!(
            output_lines[25],
            "         -> Bucket [-4.113250378782927, -1), Count: 0"
        );
        assert_eq!(
            output_lines[26],
            "         -> Bucket (1, 4.113250378782927], Count: 0"
        );
        assert_eq!(output_lines[27], "         -> Exemplars:");
        assert_eq!(output_lines[28], "            Exemplar #0");
        assert_eq!(
            output_lines[29],
            "               -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[30],
            "               -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[31],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[32], "               -> Value: 22.2");
        assert_eq!(output_lines[33], "               -> FilteredAttributes:");
        assert_eq!(output_lines[34], "                  -> cpu: 0");
        assert_eq!(output_lines[35], "Metric #2");
        assert_eq!(output_lines[36], "   -> Name: system.cpu.time");
        assert_eq!(output_lines[37], "   -> Description: time cpu has ran");
        assert_eq!(output_lines[38], "   -> Unit: s");
        assert_eq!(output_lines[39], "   -> DataType: Histogram");
        assert_eq!(output_lines[40], "   -> AggregationTemporality: 4");
        assert_eq!(output_lines[41], "      HistogramDataPoints #0");
        assert_eq!(output_lines[42], "         -> Attributes:");
        assert_eq!(output_lines[43], "               -> freq: 3GHz");
        assert_eq!(
            output_lines[44],
            "         -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[45],
            "         -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[46], "         -> Count: 0");
        assert_eq!(output_lines[47], "         -> Sum: 56");
        assert_eq!(output_lines[48], "         -> Min: 12");
        assert_eq!(output_lines[49], "         -> Max: 100.1");
        assert_eq!(
            output_lines[50],
            "         -> ExplicitBound #0: 94.17542094619048"
        );
        assert_eq!(
            output_lines[51],
            "         -> ExplicitBound #1: 65.66722851519177"
        );
        assert_eq!(output_lines[52], "         -> Buckets #0, Count: 0");
        assert_eq!(output_lines[53], "         -> Exemplars:");
        assert_eq!(output_lines[54], "            Exemplar #0");
        assert_eq!(
            output_lines[55],
            "               -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[56],
            "               -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[57],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[58], "               -> Value: 22.2");
        assert_eq!(output_lines[59], "               -> FilteredAttributes:");
        assert_eq!(output_lines[60], "                  -> cpu: 0");
        assert_eq!(output_lines[61], "Metric #3");
        assert_eq!(output_lines[62], "   -> Name: system.cpu.time");
        assert_eq!(output_lines[63], "   -> Description: time cpu has ran");
        assert_eq!(output_lines[64], "   -> Unit: s");
        assert_eq!(output_lines[65], "   -> DataType: Sum");
        assert_eq!(output_lines[66], "   -> IsMonotonic: true");
        assert_eq!(output_lines[67], "   -> AggregationTemporality: 4");
        assert_eq!(output_lines[68], "      NumberDataPoints #0");
        assert_eq!(output_lines[69], "         -> Attributes:");
        assert_eq!(
            output_lines[70],
            "               -> cpu_logical_processors: 8"
        );
        assert_eq!(
            output_lines[71],
            "         -> StartTimestamp: 1650499200000000000"
        );
        assert_eq!(
            output_lines[72],
            "         -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[73], "         -> Value: 0");
        assert_eq!(output_lines[74], "         -> Exemplars:");
        assert_eq!(output_lines[75], "            Exemplar #0");
        assert_eq!(
            output_lines[76],
            "               -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(
            output_lines[77],
            "               -> Span ID: 7271ee06d7e5925f"
        );
        assert_eq!(
            output_lines[78],
            "               -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[79], "               -> Value: 22.2");
        assert_eq!(output_lines[80], "               -> FilteredAttributes:");
        assert_eq!(output_lines[81], "                  -> ************: true");
        assert_eq!(output_lines[82], "Metric #4");
        assert_eq!(output_lines[83], "   -> Name: system.cpu.time");
        assert_eq!(output_lines[84], "   -> Description: time cpu has ran");
        assert_eq!(output_lines[85], "   -> Unit: s");
        assert_eq!(output_lines[86], "   -> DataType: Summary");
        assert_eq!(output_lines[87], "      SummaryDataPoints #0");
        assert_eq!(output_lines[88], "         -> Attributes:");
        assert_eq!(output_lines[89], "               -> cpu_cores: 4");
        assert_eq!(
            output_lines[90],
            "         -> StartTimestamp: 1650499200000000100"
        );
        assert_eq!(
            output_lines[91],
            "         -> Timestamp: 1663718400000001400"
        );
        assert_eq!(output_lines[92], "         -> Count: 0");
        assert_eq!(output_lines[93], "         -> Sum: 56");
        assert_eq!(
            output_lines[94],
            "         -> QuantileValue #0: Quantile 0, Value 0"
        );
    }
}
