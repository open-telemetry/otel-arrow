// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the ViewMarshaler for converting View messages to structured string reports.

use super::marshaler::ViewMarshaler;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::KeyValue,
    logs::v1::LogsData,
    metrics::v1::{
        ExponentialHistogramDataPoint, HistogramDataPoint, Metric, MetricsData, NumberDataPoint,
        SummaryDataPoint, metric::Data, number_data_point::Value as NumberValue,
    },
    trace::v1::TracesData,
};
use std::fmt::Write;

/// The Normal Marshaler takes View messages and converts them to a string by extracting their information
/// the finalized string will be the output for a normal verbosity level
#[derive(Default)]
pub struct NormalViewMarshaler;

impl ViewMarshaler for NormalViewMarshaler {
    fn marshal_logs(&self, logs: LogsData) -> String {
        let mut report = String::new();
        for (resource_index, resource_log) in logs.resource_logs.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_log.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }

            _ = writeln!(
                &mut report,
                "ResourceLog #{resource_index}, Schema:[{schema}], Attributes: {attributes}",
                schema = resource_log.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_log) in resource_log.scope_logs.iter().enumerate() {
                if let Some(scope) = &scope_log.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeLog #{scope_index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        name = scope.name,
                        version = scope.version,
                        schema = scope_log.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeLog #{scope_index}, Schema:, Schema: [{schema}]",
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
    fn marshal_metrics(&self, metrics: MetricsData) -> String {
        let mut report = String::new();
        for (resource_index, resource_metric) in metrics.resource_metrics.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_metric.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }

            _ = writeln!(
                &mut report,
                "ResourceMetric #{resource_index}, Schema:[{schema}], Attributes: {attributes}",
                schema = resource_metric.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_metric) in resource_metric.scope_metrics.iter().enumerate() {
                if let Some(scope) = &scope_metric.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeMetric #{scope_index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        name = scope.name,
                        version = scope.version,
                        schema = scope_metric.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeMetric #{scope_index}, Schema: [{schema}]",
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
    fn marshal_traces(&self, traces: TracesData) -> String {
        let mut report = String::new();
        for (resource_index, resource_span) in traces.resource_spans.iter().enumerate() {
            let mut resource_attributes = String::new();
            if let Some(resource) = &resource_span.resource {
                resource_attributes = attributes_string_normal(&resource.attributes);
            }
            _ = writeln!(
                &mut report,
                "ResourceSpan #{resource_index}, Schema:[{schema}], Attributes: {attributes}",
                schema = resource_span.schema_url,
                attributes = resource_attributes
            );

            for (scope_index, scope_span) in resource_span.scope_spans.iter().enumerate() {
                if let Some(scope) = &scope_span.scope {
                    _ = writeln!(
                        &mut report,
                        "   ScopeSpan #{scope_index}, Name: {name}, Version: @{version}, Schema: [{schema}], Attributes: {attributes}",
                        name = scope.name,
                        version = scope.version,
                        schema = scope_span.schema_url,
                        attributes = attributes_string_normal(&scope.attributes)
                    );
                } else {
                    _ = writeln!(
                        &mut report,
                        "   ScopeSpan #{scope_index}, Schema: [{schema}]",
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
            .expect("Failed to write quantile value");
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

    use crate::debug_processor::marshaler::ViewMarshaler;
    use crate::debug_processor::normal_marshaler::NormalViewMarshaler;
    use otel_arrow_rust::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        metrics::v1::{
             ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
            HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
            ScopeMetrics, Sum, Summary, SummaryDataPoint, 
            exponential_histogram_data_point::Buckets, 
            summary_data_point::ValueAtQuantile,
        },
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span,  TracesData, 
            
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
                        999u64,
                    )
                    .attributes(vec![KeyValue::new(
                        "hostname",
                        AnyValue::new_string("host4.gov"),
                    )])
                    .parent_span_id(Vec::from("7271ee06d7e5925f".as_bytes()))
                    .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);

        let marshaler = NormalViewMarshaler;

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
        let metrics = MetricsData::new(vec![
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
                    Metric::build_sum(
                        "system.cpu.time",
                        Sum::new(
                            0,
                            false,
                            vec![NumberDataPoint::build_int(0u64, 0i64).finish()],
                        ),
                    )
                    .finish(),
                    Metric::build_exponential_histogram(
                        "system.cpu.time",
                        ExponentialHistogram::new(
                            3,
                            vec![
                                ExponentialHistogramDataPoint::build(
                                    345u64,
                                    67,
                                    Buckets::new(2, vec![34, 45, 67]),
                                )
                                .attributes(vec![KeyValue::new(
                                    "freq",
                                    AnyValue::new_string("3GHz"),
                                )])
                                .start_time_unix_nano(23u64)
                                .count(0u64)
                                .sum(56)
                                .zero_count(7u64)
                                .flags(5u32)
                                .min(12)
                                .max(100.1)
                                .zero_threshold(-1.1)
                                .finish(),
                            ],
                        ),
                    )
                    .finish(),
                    Metric::build_histogram(
                        "system.cpu.time",
                        Histogram::new(
                            1,
                            vec![
                                HistogramDataPoint::build(
                                    567u64,
                                    vec![0],
                                    vec![94.17542094619048, 65.66722851519177],
                                )
                                .attributes(vec![KeyValue::new(
                                    "freq",
                                    AnyValue::new_string("3GHz"),
                                )])
                                .start_time_unix_nano(23u64)
                                .count(0u64)
                                .sum(56)
                                .flags(1u32)
                                .min(12)
                                .max(100.1)
                                .finish(),
                            ],
                        ),
                    )
                    .finish(),
                    Metric::build_gauge(
                        "system.cpu.time",
                        Gauge::new(vec![
                            NumberDataPoint::build_int(0u64, 0i64)
                                .attributes(vec![KeyValue::new(
                                    "cpu_logical_processors",
                                    AnyValue::new_string("8"),
                                )])
                                .start_time_unix_nano(456u64)
                                .flags(1u32)
                                .finish(),
                        ]),
                    )
                    .finish(),
                    Metric::build_summary(
                        "system.cpu.time",
                        Summary::new(vec![
                            SummaryDataPoint::build(765u64, vec![ValueAtQuantile::new(0., 0.)])
                                .attributes(vec![KeyValue::new(
                                    "cpu_cores",
                                    AnyValue::new_string("4"),
                                )])
                                .start_time_unix_nano(543u64)
                                .count(0u64)
                                .sum(56.0)
                                .flags(2u32)
                                .finish(),
                        ]),
                    )
                    .finish(),
                ])
                .finish(),
            ])
            .schema_url("http://schema.opentelemetry.io")
            .finish(),
        ]);

        let marshaler = NormalViewMarshaler;
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
                        .observed_time_unix_nano(3_000_000_000u64)
                        .severity_text("Info")
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
        let marshaler = NormalViewMarshaler;
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
}
