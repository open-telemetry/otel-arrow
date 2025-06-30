// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLPMarshaler for converting OTLP messages to structured string reports.
//!

use crate::debug_exporter::marshaler::OTLPMarshaler;
use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{InstrumentationScope, KeyValue},
    metrics::v1::{
        Exemplar, ExponentialHistogramDataPoint, HistogramDataPoint, NumberDataPoint,
        SummaryDataPoint, exemplar::Value as ExemplarValue, metric::Data,
        number_data_point::Value as NumberValue,
    },
};

use std::fmt::Write;

/// The Detailed Marshaler takes OTLP messages and converts them to a string by extracting their information
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
                            _ = writeln!(&mut report, "SpanLink: {index}");
                            if let Ok(trace_id) = std::str::from_utf8(&link.trace_id) {
                                _ = writeln!(&mut report, "     -> Trace ID: {trace_id}");
                            }
                            if let Ok(span_id) = std::str::from_utf8(&link.span_id) {
                                _ = writeln!(&mut report, "     -> Span ID: {span_id}");
                            }

                            _ = writeln!(
                                &mut report,
                                "     -> TraceState: {state}",
                                state = link.trace_state
                            );
                            _ = writeln!(
                                &mut report,
                                "     -> DroppedAttributesCount: {count}",
                                count = link.dropped_attributes_count
                            );
                            _ = writeln!(
                                &mut report,
                                "     -> Attributes: {attributes}",
                                attributes = attributes_string_detailed(&link.attributes)
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
                        _ = writeln!(&mut report, "Profile ID: {profile_id}");
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

                    _ = writeln!(
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
            _ = write!(
                &mut attribute_string,
                "\n     -> {key}: {value}",
                key = attribute.key,
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
            _ = writeln!(
                &mut report,
                "     -> IsMonotonic: {is_monotonic}",
                is_monotonic = sum.is_monotonic
            );
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = sum.aggregation_temporality
            );
            write_number_datapoints_detailed(report, &sum.data_points);
        }
        Data::Histogram(histogram) => {
            _ = writeln!(&mut report, "     -> DataType: Histogram");
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = histogram.aggregation_temporality
            );
            write_histogram_datapoints_detailed(report, &histogram.data_points);
        }
        Data::ExponentialHistogram(exponential_histogram) => {
            _ = writeln!(&mut report, "     -> DataType: Exponential Histogram");
            _ = writeln!(
                &mut report,
                "     -> AggregationTemporality: {aggregation_temporality}",
                aggregation_temporality = exponential_histogram.aggregation_temporality
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
        _ = writeln!(&mut report, "HistogramDataPoints {index}");
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

        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "Sum: {sum}");
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "Min: {min}");
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "Max: {max}");
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
        if let Some(sum) = &datapoint.sum {
            _ = writeln!(&mut report, "Sum: {sum}");
        }
        if let Some(min) = &datapoint.min {
            _ = writeln!(&mut report, "Min: {min}");
        }
        if let Some(max) = &datapoint.max {
            _ = writeln!(&mut report, "Max: {max}");
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
                    "Bucket [{upper_bound}, {lower_bound}), Count: {count}",
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
                    "Bucket ({lower_bound}, {upper_bound}], Count: {count}",
                    count = positive.bucket_counts[position]
                );
            }
        }

        if datapoint.zero_count != 0 {
            _ = writeln!(
                &mut report,
                "Bucket [0, 0], Count: {count}",
                count = datapoint.zero_count
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
                "     -> FilteredAttributes: {attributes}",
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

#[cfg(test)]
mod tests {

    use crate::debug_exporter::detailed_otlp_marshaler::DetailedOTLPMarshaler;
    use crate::debug_exporter::marshaler::OTLPMarshaler;
    use crate::mock::{
        create_otlp_log, create_otlp_metric, create_otlp_profile, create_otlp_trace,
    };

    #[test]
    fn test_marshal_traces() {
        let trace = create_otlp_trace(1, 1, 1, 1, 1);
        let marshaler = DetailedOTLPMarshaler::default();
        let marshaled_trace = marshaler.marshal_traces(trace);
        let mut output_lines = Vec::new();
        for line in marshaled_trace.lines() {
            output_lines.push(line);
        }

        assert_eq!(output_lines[0], "ResourceSpan #0");
        assert_eq!(
            output_lines[1],
            "Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "Resource attributes ");
        assert_eq!(output_lines[3], "     -> ip: 192.168.0.1");
        assert_eq!(output_lines[4], "ScopeSpans #0");
        assert_eq!(
            output_lines[5],
            "ScopeSpans SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[6], "Instrumentation Scope library @v1");
        assert_eq!(output_lines[7], "Instrumentation Scope Attributes: ");
        assert_eq!(output_lines[8], "     -> hostname: host5.retailer.com");
        assert_eq!(output_lines[9], "Span 0");
        assert_eq!(
            output_lines[10],
            "Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[11], "Parent ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[12], "ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[13], "Name: user-account");
        assert_eq!(output_lines[14], "Kind: 4");
        assert_eq!(output_lines[15], "TraceState: ended");
        assert_eq!(output_lines[16], "Start time: 1647648000000000106");
        assert_eq!(output_lines[17], "End time: 1647648000000000104");
        assert_eq!(output_lines[18], "Status code: 2");
        assert_eq!(output_lines[19], "Status message: Error");
        assert_eq!(output_lines[20], "Attributes: ");
        assert_eq!(output_lines[21], "     -> hostname: host4.gov");
        assert_eq!(output_lines[22], "Events: ");
        assert_eq!(output_lines[23], "SpanEvent 0");
        assert_eq!(output_lines[24], "     -> Name: message-receive");
        assert_eq!(output_lines[25], "     -> Timestamp: 1647648000000000108");
        assert_eq!(output_lines[26], "     -> DroppedAttributesCount: 0");
        assert_eq!(output_lines[27], "     -> Attributes: ");
        assert_eq!(output_lines[28], "     -> hostname: host5.retailer.com");
        assert_eq!(output_lines[29], "Links: ");
        assert_eq!(output_lines[30], "SpanLink: 0");
        assert_eq!(
            output_lines[31],
            "     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[32], "     -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[33], "     -> TraceState: ended");
        assert_eq!(output_lines[34], "     -> DroppedAttributesCount: 0");
        assert_eq!(output_lines[35], "     -> Attributes: ");
        assert_eq!(output_lines[36], "     -> hostname: host2.org");
    }
    #[test]
    fn test_marshal_metrics() {
        let metric = create_otlp_metric(1, 1, 5, 1);

        let marshaler = DetailedOTLPMarshaler::default();

        let marshaled_metrics = marshaler.marshal_metrics(metric);

        let mut output_lines = Vec::new();
        for line in marshaled_metrics.lines() {
            output_lines.push(line);
        }

        assert_eq!(output_lines[0], "ResourceMetric #0");
        assert_eq!(
            output_lines[1],
            "Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "Resource attributes: ");
        assert_eq!(output_lines[3], "     -> ip: 192.168.0.2");
        assert_eq!(output_lines[4], "ScopeMetrics #0");
        assert_eq!(
            output_lines[5],
            "ScopeMetrics SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[6], "Instrumentation Scope library @v1");
        assert_eq!(output_lines[7], "Instrumentation Scope Attributes: ");
        assert_eq!(
            output_lines[8],
            "     -> instrumentation_scope_k1: k1 value"
        );
        assert_eq!(output_lines[9], "Metric #0");
        assert_eq!(output_lines[10], "Descriptor:");
        assert_eq!(output_lines[11], "     -> Name: system.cpu.time");
        assert_eq!(output_lines[12], "     -> Description: time cpu has ran");
        assert_eq!(output_lines[13], "     -> Unit: s");
        assert_eq!(output_lines[14], "     -> DataType: Gauge");
        assert_eq!(output_lines[15], "NumberDataPoints #0");
        assert_eq!(output_lines[16], "Attributes: ");
        assert_eq!(output_lines[17], "StartTimestamp: 1650499200000000100");
        assert_eq!(output_lines[18], "Timestamp: 1663718400000001400");
        assert_eq!(output_lines[19], "Value: 0");
        assert_eq!(output_lines[20], "Metric #1");
        assert_eq!(output_lines[21], "Descriptor:");
        assert_eq!(output_lines[22], "     -> Name: system.cpu.time");
        assert_eq!(output_lines[23], "     -> Description: time cpu has ran");
        assert_eq!(output_lines[24], "     -> Unit: s");
        assert_eq!(output_lines[25], "     -> DataType: Exponential Histogram");
        assert_eq!(output_lines[26], "     -> AggregationTemporality: 4");
        assert_eq!(output_lines[27], "ExponentialHistogramDataPoints #0");
        assert_eq!(output_lines[28], "Attributes: ");
        assert_eq!(output_lines[29], "     -> freq: 3GHz");
        assert_eq!(output_lines[30], "StartTimestamp: 1650499200000000000");
        assert_eq!(output_lines[31], "Timestamp: 1663718400000001400");
        assert_eq!(output_lines[32], "Count: 0");
        assert_eq!(output_lines[33], "Sum: 56");
        assert_eq!(output_lines[34], "Min: 12");
        assert_eq!(output_lines[35], "Max: 100.1");
        assert_eq!(
            output_lines[36],
            "Bucket [-4.113250378782927, -1), Count: 0"
        );
        assert_eq!(output_lines[37], "Bucket (1, 4.113250378782927], Count: 0");
        assert_eq!(output_lines[38], "Exemplars: ");
        assert_eq!(output_lines[39], "Exemplar #0");
        assert_eq!(
            output_lines[40],
            "     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[41], "     -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[42], "     -> Timestamp: 1663718400000001400");
        assert_eq!(output_lines[43], "     -> Value: 22.2");
        assert_eq!(output_lines[44], "     -> FilteredAttributes: ");
        assert_eq!(output_lines[45], "     -> cpu: 0");
        assert_eq!(output_lines[46], "Metric #2");
        assert_eq!(output_lines[47], "Descriptor:");
        assert_eq!(output_lines[48], "     -> Name: system.cpu.time");
        assert_eq!(output_lines[49], "     -> Description: time cpu has ran");
        assert_eq!(output_lines[50], "     -> Unit: s");
        assert_eq!(output_lines[51], "     -> DataType: Histogram");
        assert_eq!(output_lines[52], "     -> AggregationTemporality: 4");
        assert_eq!(output_lines[53], "HistogramDataPoints 0");
        assert_eq!(output_lines[54], "Attributes: ");
        assert_eq!(output_lines[55], "     -> freq: 3GHz");
        assert_eq!(output_lines[56], "StartTimestamp: 1650499200000000000");
        assert_eq!(output_lines[57], "Timestamp: 1663718400000001400");
        assert_eq!(output_lines[58], "Count: 0");
        assert_eq!(output_lines[59], "Sum: 56");
        assert_eq!(output_lines[60], "Min: 12");
        assert_eq!(output_lines[61], "Max: 100.1");
        assert_eq!(output_lines[62], "ExplicitBound 0: 94.17542094619048");
        assert_eq!(output_lines[63], "ExplicitBound 1: 65.66722851519177");
        assert_eq!(output_lines[64], "Buckets 0, Count: 0");
        assert_eq!(output_lines[65], "Exemplars: ");
        assert_eq!(output_lines[66], "Exemplar #0");
        assert_eq!(
            output_lines[67],
            "     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[68], "     -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[69], "     -> Timestamp: 1663718400000001400");
        assert_eq!(output_lines[70], "     -> Value: 22.2");
        assert_eq!(output_lines[71], "     -> FilteredAttributes: ");
        assert_eq!(output_lines[72], "     -> cpu: 0");
        assert_eq!(output_lines[73], "Metric #3");
        assert_eq!(output_lines[74], "Descriptor:");
        assert_eq!(output_lines[75], "     -> Name: system.cpu.time");
        assert_eq!(output_lines[76], "     -> Description: time cpu has ran");

        assert_eq!(output_lines[77], "     -> Unit: s");
        assert_eq!(output_lines[78], "     -> DataType: Sum");
        assert_eq!(output_lines[79], "     -> IsMonotonic: true");
        assert_eq!(output_lines[80], "     -> AggregationTemporality: 4");
        assert_eq!(output_lines[81], "NumberDataPoints #0");
        assert_eq!(output_lines[82], "Attributes: ");
        assert_eq!(output_lines[83], "     -> cpu_logical_processors: 8");
        assert_eq!(output_lines[84], "StartTimestamp: 1650499200000000000");
        assert_eq!(output_lines[85], "Timestamp: 1663718400000001400");
        assert_eq!(output_lines[86], "Value: 0");
        assert_eq!(output_lines[87], "Exemplars: ");
        assert_eq!(output_lines[88], "Exemplar #0");
        assert_eq!(
            output_lines[89],
            "     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[90], "     -> Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[91], "     -> Timestamp: 1663718400000001400");
        assert_eq!(output_lines[92], "     -> Value: 22.2");
        assert_eq!(output_lines[93], "     -> FilteredAttributes: ");
        assert_eq!(output_lines[94], "     -> ************: true");
        assert_eq!(output_lines[95], "Metric #4");
        assert_eq!(output_lines[96], "Descriptor:");

        assert_eq!(output_lines[97], "     -> Name: system.cpu.time");
        assert_eq!(output_lines[98], "     -> Description: time cpu has ran");
        assert_eq!(output_lines[99], "     -> Unit: s");
        assert_eq!(output_lines[100], "     -> DataType: Summary");
        assert_eq!(output_lines[101], "SummaryDataPoints 0");
        assert_eq!(output_lines[102], "Attributes: ");
        assert_eq!(output_lines[103], "     -> cpu_cores: 4");
        assert_eq!(output_lines[104], "StartTimestamp: 1650499200000000100");
        assert_eq!(output_lines[105], "Timestamp: 1663718400000001400");
        assert_eq!(output_lines[106], "Count: 0");
        assert_eq!(output_lines[107], "Sum: 56");
        assert_eq!(output_lines[108], "QuantileValue 0: Quantile 0, Value 0");
    }

    #[test]
    fn test_marshal_logs() {
        let logs = create_otlp_log(1, 1, 1);
        let marshaler = DetailedOTLPMarshaler::default();
        let marshaled_logs = marshaler.marshal_logs(logs);
        let mut output_lines = Vec::new();
        for line in marshaled_logs.lines() {
            output_lines.push(line);
        }

        assert_eq!(output_lines[0], "ResourceLog #0");
        assert_eq!(
            output_lines[1],
            "Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "Resource attributes: ");
        assert_eq!(output_lines[3], "     -> version: 2.0");
        assert_eq!(output_lines[4], "ScopeLogs #0");
        assert_eq!(
            output_lines[5],
            "ScopeLogs SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[6], "Instrumentation Scope library @v1");
        assert_eq!(output_lines[7], "Instrumentation Scope Attributes: ");
        assert_eq!(output_lines[8], "     -> hostname: host5.retailer.com");
        assert_eq!(output_lines[9], "LogRecord #0");
        assert_eq!(output_lines[10], "ObservedTimestamp: 1663718400000001300");
        assert_eq!(output_lines[11], "Timestamp: 2000000000");
        assert_eq!(output_lines[12], "SeverityText: INFO");
        assert_eq!(output_lines[13], "SeverityNumber: 2");
        assert_eq!(output_lines[14], "EventName: event1");
        assert_eq!(
            output_lines[15],
            "Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci."
        );
        assert_eq!(output_lines[16], "Attributes: ");
        assert_eq!(output_lines[17], "     -> hostname: host3.thedomain.edu");
        assert_eq!(
            output_lines[18],
            "Trace ID: 4327e52011a22f9662eac217d77d1ec0"
        );
        assert_eq!(output_lines[19], "Span ID: 7271ee06d7e5925f");
        assert_eq!(output_lines[20], "Flags: 8");
    }

    #[test]
    fn test_marshal_profiles() {
        let profiles = create_otlp_profile(1, 1, 1);
        let marshaler = DetailedOTLPMarshaler::default();
        let marshaled_profiles = marshaler.marshal_profiles(profiles);
        let mut output_lines = Vec::new();
        for line in marshaled_profiles.lines() {
            output_lines.push(line);
        }

        assert_eq!(output_lines[0], "ResourceProfile #0");
        assert_eq!(
            output_lines[1],
            "Resource SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[2], "Resource attributes ");
        assert_eq!(output_lines[3], "     -> hostname: host7.com");
        assert_eq!(output_lines[4], "ScopeProfiles #0");
        assert_eq!(
            output_lines[5],
            "ScopeProfiles SchemaURL: http://schema.opentelemetry.io"
        );
        assert_eq!(output_lines[6], "Instrumentation Scope library @v1");
        assert_eq!(output_lines[7], "Instrumentation Scope Attributes: ");
        assert_eq!(output_lines[8], "     -> hostname: host5.retailer.com");
        assert_eq!(output_lines[9], "Profile 0");
        assert_eq!(output_lines[10], "Profile ID: ");
        assert_eq!(output_lines[11], "Start time: 0");
        assert_eq!(output_lines[12], "Duration: 0");
        assert_eq!(output_lines[13], "Dropped attributes count: 0");
        assert_eq!(output_lines[14], "Location indices: []");
    }
}
