// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLPMarshaler for converting OTLP messages to structured string reports.
//!

use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{ InstrumentationScope, KeyValue,},
    metrics::v1::{
        Exemplar, ExponentialHistogramDataPoint, HistogramDataPoint, NumberDataPoint,
        SummaryDataPoint, exemplar::Value as ExemplarValue, metric::Data,
        number_data_point::Value as NumberValue,
    },
};
use crate::debug_exporter::marshaler::OTLPMarshaler;

use std::fmt::Write;

/// The Detailed Marshaler takes OTLP messages and converts them to a string by extracting their informations
/// the finalized string will be the output for a detailed verbosity level
#[derive(Default)]
pub struct DetailedOTLPMarshaler;

impl OTLPMarshaler for DetailedOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            _ = writeln!(&mut report, "ResourceLog #{index}", index = resource_index);
            _ = writeln!(
                &mut report,
                "Resource SchemaURL: {schema_url}",
                schema_url = resource_log.schema_url
            );
            if let Some(resource) = &resource_log.resource {
                _ = writeln!(
                    &mut report,
                    "Resource attributes: {attributes}",
                    attributes = attributes_string_detailed(&resource.attributes)
                );
            }

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                _ = writeln!(&mut report, "ScopeLogs #{index}", index = scope_index);
                _ = writeln!(
                    &mut report,
                    "ScopeLogs SchemaURL: {schema_url}",
                    schema_url = scope_log.schema_url
                );
                if let Some(scope) = &scope_log.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (record_index, log_record) in scope_log.log_records.iter().enumerate() {
                    _ = writeln!(&mut report, "LogRecord #{index}", index = record_index);
                    _ = writeln!(
                        &mut report,
                        "ObservedTimestamp: {timestamp}",
                        timestamp = log_record.observed_time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "Timestamp: {timestamp}",
                        timestamp = log_record.time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "SeverityText: {severity}",
                        severity = log_record.severity_text
                    );
                    _ = writeln!(
                        &mut report,
                        "SeverityNumber: {severity_number}",
                        severity_number = log_record.severity_number
                    );

                    if !log_record.event_name.is_empty() {
                        _ = writeln!(
                            &mut report,
                            "EventName: {event_name}",
                            event_name = log_record.event_name
                        );
                    }
                    if let Some(body) = &log_record.body {
                        _ = writeln!(&mut report, "Body: {body}");
                    }
                    _ = writeln!(
                        &mut report,
                        "Attributes: {attributes}",
                        attributes = attributes_string_detailed(&log_record.attributes)
                    );
                    if let Ok(trace_id) = std::str::from_utf8(&log_record.trace_id) {
                        _ = writeln!(&mut report, "Trace ID: {trace_id}", trace_id = trace_id);
                    }

                    if let Ok(span_id) = std::str::from_utf8(&log_record.span_id) {
                        _ = writeln!(&mut report, "Span ID: {span_id}", span_id = span_id);
                    }

                    _ = writeln!(&mut report, "Flags: {flags}", flags = log_record.flags);
                }
            }
        }
        report
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "ResourceMetric #{index}",
                index = resource_index
            );
            _ = writeln!(
                &mut report,
                "Resource SchemaURL: {schema_url}",
                schema_url = resource_metric.schema_url
            );

            if let Some(resource) = &resource_metric.resource {
                _ = writeln!(
                    &mut report,
                    "Resource attributes: {attributes}",
                    attributes = attributes_string_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                _ = writeln!(&mut report, "ScopeMetrics #{index}", index = scope_index);
                _ = writeln!(
                    &mut report,
                    "ScopeMetrics SchemaURL: {schema_url}",
                    schema_url = scope_metric.schema_url
                );
                if let Some(scope) = &scope_metric.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (metric_index, metric) in scope_metric.metrics.iter().enumerate() {
                    _ = writeln!(&mut report, "Metric #{index}", index = metric_index);
                    _ = writeln!(&mut report, "Descriptor:");
                    _ = writeln!(&mut report, "     -> Name: {name}", name = metric.name);
                    _ = writeln!(
                        &mut report,
                        "     -> Description: {description}",
                        description = metric.description
                    );
                    _ = writeln!(&mut report, "     -> Unit: {unit}", unit = metric.unit);
                    if let Some(data) = &metric.data {
                        write_datapoints_detailed(&mut report, data);
                    }
                }
            }
        }
        report
    }
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            _ = writeln!(&mut report, "ResourceSpan #{index}", index = resource_index);
            _ = writeln!(
                &mut report,
                "Resource SchemaURL: {schema_url}",
                schema_url = resource_span.schema_url
            );
            if let Some(resource) = &resource_span.resource {
                _ = writeln!(
                    &mut report,
                    "Resource attributes {attributes}",
                    attributes = attributes_string_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                _ = writeln!(&mut report, "ScopeSpans #{index}", index = scope_index);
                _ = writeln!(
                    &mut report,
                    "ScopeSpans SchemaURL: {schema_url}",
                    schema_url = scope_span.schema_url
                );
                if let Some(scope) = &scope_span.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (span_index, span) in scope_span.spans.iter().enumerate() {
                    _ = writeln!(&mut report, "Span {index}", index = span_index);
                    if let Ok(trace_id) = std::str::from_utf8(&span.trace_id) {
                        _ = writeln!(&mut report, "Trace ID: {trace_id}", trace_id = trace_id);
                    }
                    if let Ok(parent_span_id) = std::str::from_utf8(&span.parent_span_id) {
                        _ = writeln!(
                            &mut report,
                            "Parent ID: {parent_span_id}",
                            parent_span_id = parent_span_id
                        );
                    }
                    if let Ok(span_id) = std::str::from_utf8(&span.span_id) {
                        _ = writeln!(&mut report, "ID: {span_id}", span_id = span_id);
                    }

                    _ = writeln!(&mut report, "Name: {name}", name = span.name);
                    _ = writeln!(&mut report, "Kind: {kind}", kind = span.kind);
                    if !span.trace_state.is_empty() {
                        _ = writeln!(
                            &mut report,
                            "TraceState: {trace_state}",
                            trace_state = span.trace_state
                        );
                    }

                    _ = writeln!(
                        &mut report,
                        "Start time: {start_time}",
                        start_time = span.start_time_unix_nano
                    );
                    _ = writeln!(
                        &mut report,
                        "End time: {end_time}",
                        end_time = span.end_time_unix_nano
                    );
                    if let Some(status) = &span.status {
                        _ = writeln!(
                            &mut report,
                            "Status code: {status_code}",
                            status_code = status.code
                        );
                        _ = writeln!(
                            &mut report,
                            "Status message: {status_message}",
                            status_message = status.message
                        );
                    }

                    _ = writeln!(
                        &mut report,
                        "Attributes: {attributes}",
                        attributes = attributes_string_detailed(&span.attributes)
                    );

                    if !span.events.is_empty() {
                        _ = writeln!(&mut report, "Events: ");
                        for (event_index, event) in span.events.iter().enumerate() {
                            _ = writeln!(&mut report, "SpanEvent {index}", index = event_index);
                            _ = writeln!(&mut report, "     -> Name: {name}", name = event.name);
                            _ = writeln!(
                                &mut report,
                                "     -> Timestamp: {timestamp}",
                                timestamp = event.time_unix_nano
                            );
                            _ = writeln!(
                                &mut report,
                                "     -> DroppedAttributesCount: {dropped_attributes_count}",
                                dropped_attributes_count = event.dropped_attributes_count
                            );
                            _ = writeln!(
                                &mut report,
                                "     -> Attributes: {attributes}",
                                attributes = attributes_string_detailed(&event.attributes)
                            );
                        }
                    }

                    if !span.links.is_empty() {
                        _ = writeln!(&mut report, "Links: ");
                        for (index, link) in span.links.iter().enumerate() {
                            _ = writeln!(&mut report, "SpanLink: {}", index);
                            if let Ok(trace_id) = std::str::from_utf8(&link.trace_id) {
                                _ = writeln!(&mut report, "     -> Trace ID: {}", trace_id);
                            }
                            if let Ok(span_id) = std::str::from_utf8(&link.span_id) {
                                _ = writeln!(&mut report, "     -> Span ID: {}", span_id);
                            }

                            _ = writeln!(&mut report, "     -> TraceState: {}", link.trace_state);
                            _ = writeln!(
                                &mut report,
                                "     -> DroppedAttributesCount: {}",
                                link.dropped_attributes_count
                            );
                            _ = writeln!(
                                &mut report,
                                "     -> Attributes: {}",
                                attributes_string_detailed(&link.attributes)
                            );
                        }
                    }
                }
            }
        }
        report
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        let mut report = String::new();

        // ToDo: Display profile mapping, profile location, profile functions,
        for (resource_index, resource_profile) in profiles.resource_profiles.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "ResourceProfile #{index}",
                index = resource_index
            );
            _ = writeln!(
                &mut report,
                "Resource SchemaURL: {schema_url}",
                schema_url = resource_profile.schema_url
            );
            if let Some(resource) = &resource_profile.resource {
                _ = writeln!(
                    &mut report,
                    "Resource attributes {attributes}",
                    attributes = attributes_string_detailed(&resource.attributes)
                );
            }
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                _ = writeln!(&mut report, "ScopeProfiles #{index}", index = scope_index);
                _ = writeln!(
                    &mut report,
                    "ScopeProfiles SchemaURL: {schema_url}",
                    schema_url = scope_profile.schema_url
                );
                if let Some(scope) = &scope_profile.scope {
                    write_instrumentation_scope(&mut report, scope);
                }

                for (profile_index, profile) in scope_profile.profiles.iter().enumerate() {
                    _ = writeln!(&mut report, "Profile {index}", index = profile_index);
                    if let Ok(profile_id) = std::str::from_utf8(&profile.profile_id) {
                        _ = writeln!(&mut report, "Profile ID: {}", profile_id);
                    }
                    _ = writeln!(
                        &mut report,
                        "Start time: {profile_start_time}",
                        profile_start_time = profile.time_nanos
                    );
                    _ = writeln!(
                        &mut report,
                        "Duration: {profile_duration}",
                        profile_duration = profile.duration_nanos
                    );
                    _ = writeln!(
                        &mut report,
                        "Dropped attributes count: {profile_dropped_attributes_count}",
                        profile_dropped_attributes_count = profile.dropped_attributes_count
                    );

                    _ = write!(
                        &mut report,
                        "Location indices: {location_indices:?}",
                        location_indices = profile.location_indices
                    );

                    // ToDo: display profile samples

                    if !profile.comment_strindices.is_empty() {
                        _ = writeln!(&mut report, "Comment: ");
                        for comment in profile.comment_strindices.iter() {
                            _ = writeln!(&mut report, "     -> {comment}");
                        }
                    }
                }
            }
        }
        report
    }
}

fn attributes_string_detailed(attributes: &[KeyValue]) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            let attribute_value = value.to_string();
            _ = write!(
                &mut attribute_string,
                "\n     -> {key}: {value}",
                key = attribute.key,
                value = attribute_value
            );
        }
    }

    attribute_string
}

fn write_datapoints_detailed(mut report: &mut String, data: &Data) {
    match data {
        Data::Gauge(gauge) => {
            _ = writeln!(&mut report, "     -> DataType: Gauge");
            write_number_datapoints_detailed(report, &gauge.data_points);
        }
        Data::Sum(sum) => {
            _ = writeln!(&mut report, "     -> DataType: Sum");
            _ = writeln!(&mut report, "     -> IsMonotonic: {}", sum.is_monotonic);
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {}",
                sum.aggregation_temporality
            );
            write_number_datapoints_detailed(report, &sum.data_points);
        }
        Data::Histogram(histogram) => {
            _ = writeln!(&mut report, "     -> DataType: Histogram");
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {}",
                histogram.aggregation_temporality
            );
            write_histogram_datapoints_detailed(report, &histogram.data_points);
        }
        Data::ExponentialHistogram(exponential_histogram) => {
            _ = writeln!(&mut report, "     -> DataType: Exponential Histogram");
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {}",
                exponential_histogram.aggregation_temporality
            );
            write_exponential_histogram_datapoints_detailed(
                report,
                &exponential_histogram.data_points,
            );
        }
        Data::Summary(summary) => {
            _ = writeln!(&mut report, "     -> DataType: Summary");
            write_summary_datapoints_detailed(report, &summary.data_points);
        }
    }
}

fn write_number_datapoints_detailed(mut report: &mut String, datapoints: &[NumberDataPoint]) {
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "NumberDataPoints #{index}",
            index = datapoint_index
        );
        _ = writeln!(
            &mut report,
            "Attributes: {attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes)
        );
        _ = writeln!(
            &mut report,
            "StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        if let Some(value) = &datapoint.value {
            match value {
                NumberValue::AsInt(int) => {
                    _ = writeln!(&mut report, "Value: {value}", value = int);
                }
                NumberValue::AsDouble(double) => {
                    _ = writeln!(&mut report, "Value: {value}", value = double);
                }
            }
        }

        write_exemplars(report, &datapoint.exemplars);
    }
}

fn write_histogram_datapoints_detailed(mut report: &mut String, datapoints: &[HistogramDataPoint]) {
    for (index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(&mut report, "HistogramDataPoints {}", index);
        _ = writeln!(
            &mut report,
            "Attributes: {}",
            attributes_string_detailed(&datapoint.attributes)
        );

        _ = writeln!(
            &mut report,
            "StartTimestamp: {}",
            datapoint.start_time_unix_nano
        );
        _ = writeln!(&mut report, "Timestamp: {}", datapoint.time_unix_nano);
        _ = writeln!(&mut report, "Count: {}", datapoint.count);

        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "Sum: {}", sum);
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "Min: {}", min);
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "Max: {}", max);
        }

        for (index, bound) in datapoint.explicit_bounds.iter().enumerate() {
            _ = writeln!(&mut report, "ExplicitBound {index}: {bound}",);
        }
        for (index, count) in datapoint.bucket_counts.iter().enumerate() {
            _ = writeln!(&mut report, "Buckets {index}, Count: {count}",);
        }

        write_exemplars(report, &datapoint.exemplars);
    }
}

fn write_exponential_histogram_datapoints_detailed(
    mut report: &mut String,
    datapoints: &[ExponentialHistogramDataPoint],
) {
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "ExponentialHistogramDataPoints #{index}",
            index = datapoint_index
        );
        _ = writeln!(
            &mut report,
            "Attributes: {}",
            attributes_string_detailed(&datapoint.attributes)
        );
        _ = writeln!(
            &mut report,
            "StartTimestamp: {}",
            datapoint.start_time_unix_nano
        );
        _ = writeln!(&mut report, "Timestamp: {}", datapoint.time_unix_nano);
        _ = writeln!(&mut report, "Count: {}", datapoint.count);
        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "Sum: {}", sum);
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "Min: {}", min);
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "Max: {}", max);
        }

        // calcualate the base -> 2^(2^(-scale)) -> e^(ln(2) * 2^(-scale))

        let base: f64 = (std::f64::consts::LN_2 * 2.0_f64.powf(-datapoint.scale as f64)).exp();

        if let Some(negative) = &datapoint.negative {
            let num_buckets = negative.bucket_counts.len();
            for position in 0..num_buckets {
                let updated_position = num_buckets - position - 1;

                let index: f64 = negative.offset as f64 + updated_position as f64;
                // calculate lower bound base^index
                let lower_bound = (index * base).exp();
                // calculate upper bound base^(index + 1)
                let upper_bound = ((index + 1.0) * base).exp();
                _ = writeln!(
                    report,
                    "Bucket [{}, {}), Count: {}",
                    -upper_bound, -lower_bound, negative.bucket_counts[updated_position]
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
                    "Bucket ({}, {}], Count: {}",
                    lower_bound, upper_bound, positive.bucket_counts[position]
                );
            }
        }

        if datapoint.zero_count != 0 {
            _ = writeln!(
                &mut report,
                "Bucket [0, 0], Count: {}",
                datapoint.zero_count
            );
        }

        write_exemplars(report, &datapoint.exemplars);
    }
}

fn write_summary_datapoints_detailed(mut report: &mut String, datapoints: &[SummaryDataPoint]) {
    for (datapoint_index, datapoint) in datapoints.iter().enumerate() {
        _ = writeln!(
            &mut report,
            "SummaryDataPoints {index}",
            index = datapoint_index
        );
        _ = writeln!(
            &mut report,
            "Attributes: {attributes}",
            attributes = attributes_string_detailed(&datapoint.attributes)
        );
        _ = writeln!(
            &mut report,
            "StartTimestamp: {timestamp}",
            timestamp = datapoint.start_time_unix_nano
        );
        _ = writeln!(
            &mut report,
            "Timestamp: {timestamp}",
            timestamp = datapoint.time_unix_nano
        );
        _ = writeln!(&mut report, "Count: {count}", count = datapoint.count);
        _ = writeln!(&mut report, "Sum: {sum}", sum = datapoint.sum);
        for (quantile_index, quantile) in datapoint.quantile_values.iter().enumerate() {
            _ = writeln!(
                &mut report,
                "QuantileValue {index}: Quantile {quantile}, Value {value}",
                index = quantile_index,
                quantile = quantile.quantile,
                value = quantile.value
            );
        }
    }
}

fn write_exemplars(mut report: &mut String, exemplars: &[Exemplar]) {
    if !exemplars.is_empty() {
        _ = writeln!(&mut report, "Exemplars: ");

        for (exemplar_index, exemplar) in exemplars.iter().enumerate() {
            _ = writeln!(&mut report, "Exemplar #{index}", index = exemplar_index);
            if let Ok(trace_id) = std::str::from_utf8(&exemplar.trace_id) {
                _ = writeln!(
                    &mut report,
                    "     -> Trace ID: {trace_id}",
                    trace_id = trace_id
                );
            }
            if let Ok(span_id) = std::str::from_utf8(&exemplar.span_id) {
                _ = writeln!(&mut report, "     -> Span ID: {span_id}", span_id = span_id);
            }
            _ = writeln!(
                &mut report,
                "     -> Timestamp: {timestamp}",
                timestamp = exemplar.time_unix_nano
            );
            if let Some(value) = &exemplar.value {
                match value {
                    ExemplarValue::AsInt(int) => {
                        _ = writeln!(&mut report, "     -> Value: {value}", value = int);
                    }
                    ExemplarValue::AsDouble(double) => {
                        _ = writeln!(&mut report, "     -> Value: {value}", value = double);
                    }
                }
            }
            _ = writeln!(
                &mut report,
                "     -> FilteredAttributes:\n{attributes}",
                attributes = attributes_string_detailed(&exemplar.filtered_attributes)
            );
        }
    }
}

fn write_instrumentation_scope(mut report: &mut String, scope: &InstrumentationScope) {
    _ = writeln!(
        &mut report,
        "Instrumentation Scope {name} @{version}",
        name = scope.name,
        version = scope.version
    );
    _ = writeln!(
        &mut report,
        "Instrumentation Scope Attributes: {attributes}",
        attributes = attributes_string_detailed(&scope.attributes)
    );
}
