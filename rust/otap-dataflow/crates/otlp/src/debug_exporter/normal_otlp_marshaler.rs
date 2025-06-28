// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLPMarshaler for converting OTLP messages to structured string reports.
//!

use crate::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::KeyValue,
    metrics::v1::{
        ExponentialHistogramDataPoint, HistogramDataPoint, Metric, NumberDataPoint,
        SummaryDataPoint, metric::Data,
        number_data_point::Value as NumberValue,
    },
};
use crate::debug_exporter::marshaler::OTLPMarshaler;
use std::fmt::Write;

/// The Normal Marshaler takes OTLP messages and converts them to a string by extracting their informations
/// the finalized string will be the output for a normal verbosity level
#[derive(Default)]
pub struct NormalOTLPMarshaler;

impl OTLPMarshaler for NormalOTLPMarshaler {
    fn marshal_logs(&self, logs: ExportLogsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_log.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }

            _ = writeln!(
                &mut report,
                "ResourceLog #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_log.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                if let Some(scope) = &scope_log.scope {
                    _ = writeln!(
                        &mut report,
                        "ScopeLog #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_log.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "ScopeLog #{index}, Schema:, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_log.schema_url,
                    );
                }

                for log_record in scope_log.log_records.iter() {
                    let mut log_body = String::new();
                    if let Some(body) = &log_record.body {
                        log_body = body.to_string();
                    }

                    _ = writeln!(
                        &mut report,
                        "Body: {body}, Attributes {attributes}",
                        body = log_body,
                        attributes = attributes_string_normal(&log_record.attributes)
                    );
                }
            }
        }
        report
    }
    fn marshal_metrics(&self, metrics: ExportMetricsServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_metric.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }

            _ = writeln!(
                &mut report,
                "ResourceLog #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_metric.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                if let Some(scope) = &scope_metric.scope {
                    _ = writeln!(
                        &mut report,
                        "ScopeMetric #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_metric.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "ScopeMetric #{index}, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_metric.schema_url,
                    );
                }

                for metric in scope_metric.metrics.iter() {
                    if let Some(data) = &metric.data {
                        match data {
                            Data::Gauge(gauge) => write_number_datapoints_normal(
                                &mut report,
                                metric,
                                &gauge.data_points,
                            ),
                            Data::Sum(sum) => write_number_datapoints_normal(
                                &mut report,
                                metric,
                                &sum.data_points,
                            ),
                            Data::Histogram(histogram) => write_histogram_datapoints_normal(
                                &mut report,
                                metric,
                                &histogram.data_points,
                            ),
                            Data::ExponentialHistogram(exponential_histogram) => {
                                write_exponential_histogram_datapoints_normal(
                                    &mut report,
                                    metric,
                                    &exponential_histogram.data_points,
                                )
                            }
                            Data::Summary(summary) => write_summary_datapoints_normal(
                                &mut report,
                                metric,
                                &summary.data_points,
                            ),
                        }
                    }
                }
            }
        }
        report
    }
    fn marshal_traces(&self, traces: ExportTraceServiceRequest) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_span.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }
            _ = writeln!(
                &mut report,
                "ResourceLog #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_span.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                if let Some(scope) = &scope_span.scope {
                    _ = writeln!(
                        &mut report,
                        "ScopeSpan #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_span.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "ScopeSpan #{index}, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_span.schema_url,
                    );
                }

                for span in scope_span.spans.iter() {
                    // write line " {name} {trace_id} {span_id} {attributes}"
                    _ = write!(&mut report, "Name: {name}, ", name = &span.name,);
                    if let Ok(trace_id) = String::from_utf8(span.trace_id.clone()) {
                        _ = write!(&mut report, "Trace ID: {trace_id}, ", trace_id = trace_id);
                    }
                    if let Ok(span_id) = String::from_utf8(span.span_id.clone()) {
                        _ = write!(&mut report, "Span ID: {span_id}, ", span_id = span_id);
                    }

                    _ = writeln!(
                        &mut report,
                        "Attributes: {attributes}",
                        attributes = attributes_string_normal(&span.attributes)
                    );
                }
            }
        }
        report
    }
    fn marshal_profiles(&self, profiles: ExportProfilesServiceRequest) -> String {
        // marshal_profiles to string based on verbosity
        let mut report = String::new();
        for (resource_index, resource_profile) in profiles.resource_profiles.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_profile.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }

            _ = writeln!(
                &mut report,
                "ResourceLog #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_profile.schema_url,
                attributes = resource_attributes
            );
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                if let Some(scope) = &scope_profile.scope {
                    _ = writeln!(
                        &mut report,
                        "ScopeProfile #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_profile.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "ScopeProfile #{index}:, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_profile.schema_url,
                    );
                }

                for _ in scope_profile.profiles.iter() {
                    // Todo: use the attributes indicies from the profile object to get the attributes from the attribute table
                }
            }
        }
        report
    }
}

pub fn attributes_string_normal(attributes: &[KeyValue]) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            let attribute_value = value.to_string();
            _ = write!(
                &mut attribute_string,
                "{key}={value} ",
                key = attribute.key,
                value = attribute_value
            );
        }
    }

    attribute_string
}

fn write_number_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[NumberDataPoint],
) {
    for datapoint in datapoints.iter() {
        let datapoint_attributes = attributes_string_normal(&datapoint.attributes);
        if let Some(value) = datapoint.value {
            match value {
                NumberValue::AsDouble(double) => {
                    _ = writeln!(
                        &mut report,
                        "{name} {attributes} {value}",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = double
                    );
                }
                NumberValue::AsInt(int) => {
                    _ = writeln!(
                        &mut report,
                        "{name} {attributes} {value}",
                        name = metric.name,
                        attributes = datapoint_attributes,
                        value = int
                    );
                }
            }
        }
    }
}

fn write_histogram_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[HistogramDataPoint],
) {
    for datapoint in datapoints.iter() {
        let datapoint_attributes = attributes_string_normal(&datapoint.attributes);
        let mut values = String::new();
        _ = write!(&mut values, "count={} ", datapoint.count);
        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={} ", sum);
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={} ", min);
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={} ", max);
        }

        for (i, bucket) in datapoint.bucket_counts.iter().enumerate() {
            let mut bucket_bound = String::new();
            if i < datapoint.explicit_bounds.len() {
                bucket_bound = format!("le{}=", datapoint.explicit_bounds[i]);
            }
            _ = write!(&mut values, "{}{} ", bucket_bound, bucket);
        }

        _ = writeln!(
            &mut report,
            "{name} {attributes} {values}",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }
}

fn write_exponential_histogram_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[ExponentialHistogramDataPoint],
) {
    for datapoint in datapoints.iter() {
        let datapoint_attributes = attributes_string_normal(&datapoint.attributes);

        let mut values = String::new();
        _ = write!(&mut values, "count={} ", datapoint.count);

        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={} ", sum);
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={} ", min);
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={} ", max);
        }

        _ = writeln!(
            &mut report,
            "{name} {attributes} {values}",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }
}

fn write_summary_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[SummaryDataPoint],
) {
    for datapoint in datapoints.iter() {
        let datapoint_attributes = attributes_string_normal(&datapoint.attributes);
        let mut values = String::new();

        _ = write!(&mut values, "count={} ", datapoint.count);
        _ = write!(&mut values, "sum={} ", datapoint.sum);

        for quantile in datapoint.quantile_values.iter() {
            write!(&mut values, "q{}={} ", quantile.quantile, quantile.value).unwrap();
        }

        _ = writeln!(
            &mut report,
            "{name} {attributes} {values}",
            name = metric.name,
            attributes = datapoint_attributes,
            values = values
        );
    }
}

