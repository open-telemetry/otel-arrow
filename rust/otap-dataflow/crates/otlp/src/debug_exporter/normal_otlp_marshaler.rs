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
    common::v1::KeyValue,
    metrics::v1::{
        ExponentialHistogramDataPoint, HistogramDataPoint, Metric, NumberDataPoint,
        SummaryDataPoint, metric::Data, number_data_point::Value as NumberValue,
    },
};
use std::fmt::Write;

/// The Normal Marshaler takes OTLP messages and converts them to a string by extracting their information
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
                        "   ScopeLog #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_log.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeLog #{index}, Schema:, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_log.schema_url,
                    );
                }

                for log_record in scope_log.log_records.iter() {
                    if let Some(body) = &log_record.body {
                        _ = write!(&mut report, "      Body: {body}, ");
                    }
                    // TODO
                    _ = writeln!(
                        &mut report,
                        "Attributes: {attributes}",
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
                "ResourceMetric #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_metric.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                if let Some(scope) = &scope_metric.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeMetric #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_metric.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeMetric #{index}, Schema: [{schema}]",
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
                "ResourceSpan #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_span.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                if let Some(scope) = &scope_span.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeSpan #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_span.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeSpan #{index}, Schema: [{schema}]",
                        index = scope_index,
                        schema = scope_span.schema_url,
                    );
                }

                for span in scope_span.spans.iter() {
                    // write line " {name} {trace_id} {span_id} {attributes}"
                    _ = write!(&mut report, "      Name: {name}, ", name = &span.name,);
                    if let Ok(trace_id) = String::from_utf8(span.trace_id.clone()) {
                        _ = write!(&mut report, "Trace ID: {trace_id}, ");
                    }
                    if let Ok(span_id) = String::from_utf8(span.span_id.clone()) {
                        _ = write!(&mut report, "Span ID: {span_id}, ");
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
                "ResourceProfile #{index}, Schema:[{schema}], Attributes: {attributes}",
                index = resource_index,
                schema = resource_profile.schema_url,
                attributes = resource_attributes
            );
            for (scope_index, scope_profile) in resource_profile.scope_profiles.iter().enumerate() {
                if let Some(scope) = &scope_profile.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeProfile #{index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        index = scope_index,
                        name = scope.name,
                        version = scope.version,
                        schema = scope_profile.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeProfile #{index}:, Schema: [{schema}]",
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

fn attributes_string_normal(attributes: &[KeyValue]) -> String {
    let mut attribute_string = String::new();
    for attribute in attributes.iter() {
        if let Some(value) = &attribute.value {
            _ = write!(&mut attribute_string, "{key}={value} ", key = attribute.key,);
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
                NumberValue::AsDouble(value) => {
                    _ = writeln!(
                        &mut report,
                        "      {name} {attributes}{value}",
                        name = metric.name,
                        attributes = datapoint_attributes,
                    );
                }
                NumberValue::AsInt(value) => {
                    _ = writeln!(
                        &mut report,
                        "      {name} {attributes}{value}",
                        name = metric.name,
                        attributes = datapoint_attributes,
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
        let mut values = String::new();
        _ = write!(&mut values, "count={count} ", count = datapoint.count);
        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={sum} ");
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={min} ");
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={max} ");
        }

        for (i, bucket) in datapoint.bucket_counts.iter().enumerate() {
            let mut bucket_bound = String::new();
            if i < datapoint.explicit_bounds.len() {
                bucket_bound = format!("le{bound}=", bound = datapoint.explicit_bounds[i]);
            }
            _ = write!(&mut values, "{bucket_bound}{bucket} ");
        }

        _ = writeln!(
            &mut report,
            "      {name} {attributes}{values}",
            name = metric.name,
            attributes = attributes_string_normal(&datapoint.attributes),
        );
    }
}

fn write_exponential_histogram_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[ExponentialHistogramDataPoint],
) {
    for datapoint in datapoints.iter() {
        let mut values = String::new();
        _ = write!(&mut values, "count={count} ", count = datapoint.count);

        if let Some(sum) = datapoint.sum {
            _ = write!(&mut values, "sum={sum} ");
        }
        if let Some(min) = datapoint.min {
            _ = write!(&mut values, "min={min} ");
        }
        if let Some(max) = datapoint.max {
            _ = write!(&mut values, "max={max} ");
        }

        _ = writeln!(
            &mut report,
            "      {name} {attributes}{values}",
            name = metric.name,
            attributes = attributes_string_normal(&datapoint.attributes),
        );
    }
}

fn write_summary_datapoints_normal(
    mut report: &mut String,
    metric: &Metric,
    datapoints: &[SummaryDataPoint],
) {
    for datapoint in datapoints.iter() {
        let mut values = String::new();

        _ = write!(&mut values, "count={count} ", count = datapoint.count);
        _ = write!(&mut values, "sum={sum} ", sum = datapoint.sum);

        for quantile in datapoint.quantile_values.iter() {
            write!(
                &mut values,
                "q{quantile}={value} ",
                quantile = quantile.quantile,
                value = quantile.value
            )
            .unwrap();
        }

        _ = writeln!(
            &mut report,
            "      {name} {attributes}{values}",
            name = metric.name,
            attributes = attributes_string_normal(&datapoint.attributes),
        );
    }
}

#[cfg(test)]
mod tests {

    use crate::debug_exporter::marshaler::OTLPMarshaler;
    use crate::debug_exporter::normal_otlp_marshaler::NormalOTLPMarshaler;
    use crate::mock::{
        create_otlp_log, create_otlp_metric, create_otlp_profile, create_otlp_trace,
    };

    #[test]
    fn test_marshal_traces() {
        let trace = create_otlp_trace(1, 1, 1, 1, 1);

        let marshaler = NormalOTLPMarshaler::default();

        let marshaled_trace = marshaler.marshal_traces(trace);

        let mut output_lines = Vec::new();
        for line in marshaled_trace.lines() {
            output_lines.push(line);
        }

        assert_eq!(
            output_lines[0],
            "ResourceSpan #0, Schema:[http://schema.opentelemetry.io], Attributes: ip=192.168.0.1 "
        );
        assert_eq!(
            output_lines[1],
            "   ScopeSpan #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com "
        );
        assert_eq!(
            output_lines[2],
            "      Name: user-account, Trace ID: 4327e52011a22f9662eac217d77d1ec0, Span ID: 7271ee06d7e5925f, Attributes: hostname=host4.gov "
        )
    }

    #[test]
    fn test_marshal_metrics() {
        let metrics = create_otlp_metric(1, 1, 5, 1);
        let marshaler = NormalOTLPMarshaler::default();
        let marshaled_metrics = marshaler.marshal_metrics(metrics);
        let mut output_lines = Vec::new();
        for line in marshaled_metrics.lines() {
            output_lines.push(line);
        }

        assert_eq!(
            output_lines[0],
            "ResourceMetric #0, Schema:[http://schema.opentelemetry.io], Attributes: ip=192.168.0.2 "
        );
        assert_eq!(
            output_lines[1],
            "   ScopeMetric #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: instrumentation_scope_k1=k1 value "
        );
        assert_eq!(output_lines[2], "      system.cpu.time 0");
        assert_eq!(
            output_lines[3],
            "      system.cpu.time freq=3GHz count=0 sum=56 min=12 max=100.1 "
        );
        assert_eq!(
            output_lines[4],
            "      system.cpu.time freq=3GHz count=0 sum=56 min=12 max=100.1 le94.17542094619048=0 "
        );
        assert_eq!(
            output_lines[5],
            "      system.cpu.time cpu_logical_processors=8 0"
        );
        assert_eq!(
            output_lines[6],
            "      system.cpu.time cpu_cores=4 count=0 sum=56 q0=0 "
        );
    }

    #[test]
    fn test_marshal_logs() {
        let logs = create_otlp_log(1, 1, 1);
        let marshaler = NormalOTLPMarshaler::default();
        let marshaled_logs = marshaler.marshal_logs(logs);
        let mut output_lines = Vec::new();
        for line in marshaled_logs.lines() {
            output_lines.push(line);
        }

        assert_eq!(
            output_lines[0],
            "ResourceLog #0, Schema:[http://schema.opentelemetry.io], Attributes: version=2.0 "
        );
        assert_eq!(
            output_lines[1],
            "   ScopeLog #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com "
        );
        assert_eq!(
            output_lines[2],
            "      Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci., Attributes: hostname=host3.thedomain.edu "
        );
    }

    #[test]
    fn test_marshal_profiles() {
        let profiles = create_otlp_profile(1, 1, 1);
        let marshaler = NormalOTLPMarshaler::default();
        let marshaled_profiles = marshaler.marshal_profiles(profiles);
        let mut output_lines = Vec::new();
        for line in marshaled_profiles.lines() {
            output_lines.push(line);
        }

        assert_eq!(
            output_lines[0],
            "ResourceProfile #0, Schema:[http://schema.opentelemetry.io], Attributes: hostname=host7.com "
        );
        assert_eq!(
            output_lines[1],
            "   ScopeProfile #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com "
        );
    }
}
