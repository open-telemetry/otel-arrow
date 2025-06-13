// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::LazyLock;

/// This provides detail specific to each oneof value type.
#[derive(Clone, Debug, Default)]
pub struct OneofCase {
    pub name: &'static str,
    pub type_param: &'static str,
    pub value_variant: &'static str,
    pub is_primitive: bool,
    pub tag: u32,
    pub proto_type: &'static str,
}

fn oneof(
    name: &'static str,
    type_param: &'static str,
    value_variant: &'static str,
    tag: u32,
    proto_type: &'static str,
) -> OneofCase {
    let is_primitive = matches!(
        type_param,
        "bool"
            | "i32"
            | "i64"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "::prost::alloc::string::String"
            | "Vec<u8>"
    );

    OneofCase {
        name,
        type_param,
        value_variant,
        is_primitive,
        tag,
        proto_type,
    }
}

pub type OneofMapping = Option<(String, Vec<OneofCase>)>;

/// This provides detail about the underlying type of protobuf enum values.
#[derive(Clone, Debug, Default)]
pub struct EnumField {
    pub datatype: &'static str,
    pub fieldtype: &'static str,
}

fn enumfield(dt: &'static str, ft: &'static str) -> EnumField {
    EnumField {
        datatype: dt,
        fieldtype: ft,
    }
}

/// This is required to contain an entry for each builder type with at
/// least one new or builder parameter.
pub static REQUIRED_PARAMS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            // Common
            ("opentelemetry.proto.common.v1.AnyValue", vec!["value"]),
            (
                "opentelemetry.proto.common.v1.KeyValue",
                vec!["key", "value"],
            ),
            ("opentelemetry.proto.common.v1.KeyValueList", vec!["values"]),
            ("opentelemetry.proto.common.v1.ArrayValue", vec!["values"]),
            (
                "opentelemetry.proto.common.v1.InstrumentationScope",
                vec!["name"],
            ),
            ("opentelemetry.proto.common.v1.EntityRef", vec!["r#type"]),
            // Resource
            (
                "opentelemetry.proto.resource.v1.Resource",
                vec!["attributes"],
            ),
            // Logs
            (
                "opentelemetry.proto.logs.v1.LogRecord",
                vec!["time_unix_nano", "severity_number", "event_name"],
            ),
            ("opentelemetry.proto.logs.v1.ScopeLogs", vec!["scope"]),
            ("opentelemetry.proto.logs.v1.ResourceLogs", vec!["resource"]),
            (
                "opentelemetry.proto.logs.v1.LogsData",
                vec!["resource_logs"],
            ),
            // Traces
            (
                "opentelemetry.proto.trace.v1.Span",
                vec!["trace_id", "span_id", "name", "start_time_unix_nano"],
            ),
            ("opentelemetry.proto.trace.v1.ScopeSpans", vec!["scope"]),
            (
                "opentelemetry.proto.trace.v1.ResourceSpans",
                vec!["resource"],
            ),
            (
                "opentelemetry.proto.trace.v1.TracesData",
                vec!["resource_spans"],
            ),
            (
                "opentelemetry.proto.trace.v1.Status",
                vec!["message", "code"],
            ),
            (
                "opentelemetry.proto.trace.v1.Span.Link",
                vec!["trace_id", "span_id"],
            ),
            (
                "opentelemetry.proto.trace.v1.Span.Event",
                vec!["name", "time_unix_nano"],
            ),
            // Metrics
            ("opentelemetry.proto.metrics.v1.ScopeMetrics", vec!["scope"]),
            (
                "opentelemetry.proto.metrics.v1.ResourceMetrics",
                vec!["resource"],
            ),
            (
                "opentelemetry.proto.metrics.v1.MetricsData",
                vec!["resource_metrics"],
            ),
            (
                "opentelemetry.proto.metrics.v1.Metric",
                vec!["name", "data"],
            ),
            (
                "opentelemetry.proto.metrics.v1.Sum",
                vec!["aggregation_temporality", "is_monotonic", "data_points"],
            ),
            ("opentelemetry.proto.metrics.v1.Gauge", vec!["data_points"]),
            (
                "opentelemetry.proto.metrics.v1.Histogram",
                vec!["aggregation_temporality", "data_points"],
            ),
            (
                "opentelemetry.proto.metrics.v1.ExponentialHistogram",
                vec!["aggregation_temporality", "data_points"],
            ),
            (
                "opentelemetry.proto.metrics.v1.Summary",
                vec!["data_points"],
            ),
            (
                "opentelemetry.proto.metrics.v1.NumberDataPoint",
                vec!["time_unix_nano", "value"],
            ),
            (
                "opentelemetry.proto.metrics.v1.HistogramDataPoint",
                vec!["time_unix_nano", "bucket_counts", "explicit_bounds"],
            ),
            (
                "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint",
                vec!["time_unix_nano", "scale", "positive"],
            ),
            (
                "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint.Buckets",
                vec!["offset", "bucket_counts"],
            ),
            (
                "opentelemetry.proto.metrics.v1.SummaryDataPoint",
                vec!["time_unix_nano", "quantile_values"],
            ),
            (
                "opentelemetry.proto.metrics.v1.SummaryDataPoint.ValueAtQuantile",
                vec!["quantile", "value"],
            ),
            (
                "opentelemetry.proto.metrics.v1.Exemplar",
                vec!["time_unix_nano", "value"],
            ),
            // Service
            (
                "opentelemetry.proto.collector.logs.v1.ExportLogsServiceRequest",
                vec!["resource_logs"],
            ),
            (
                "opentelemetry.proto.collector.logs.v1.ExportLogsServiceResponse",
                vec!["partial_success"],
            ),
            (
                "opentelemetry.proto.collector.logs.v1.ExportLogsPartialSuccess",
                vec!["rejected_log_records"],
            ),
            (
                "opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceRequest",
                vec!["resource_metrics"],
            ),
            (
                "opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceResponse",
                vec!["partial_success"],
            ),
            (
                "opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess",
                vec!["rejected_data_points"],
            ),
            (
                "opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest",
                vec!["resource_spans"],
            ),
            (
                "opentelemetry.proto.collector.trace.v1.ExportTraceServiceResponse",
                vec!["partial_success"],
            ),
            (
                "opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess",
                vec!["rejected_spans"],
            ),
        ])
    });

/// This lists all the known oneof fields in OpenTelemetry, with their cases.
pub static ONEOF_MAPPINGS: LazyLock<HashMap<String, Vec<OneofCase>>> = LazyLock::new(|| {
    HashMap::from([
        (
            "opentelemetry.proto.common.v1.AnyValue.value".into(),
            vec![
                oneof(
                    "string",
                    "::prost::alloc::string::String",
                    "any_value::Value::StringValue",
                    1,
                    "string",
                ),
                oneof("bool", "bool", "any_value::Value::BoolValue", 2, "bool"),
                oneof("int", "i64", "any_value::Value::IntValue", 3, "int64"),
                oneof(
                    "double",
                    "f64",
                    "any_value::Value::DoubleValue",
                    4,
                    "double",
                ),
                oneof(
                    "array",
                    "ArrayValue",
                    "any_value::Value::ArrayValue",
                    5,
                    "message",
                ),
                oneof(
                    "kvlist",
                    "KeyValueList",
                    "any_value::Value::KvlistValue",
                    6,
                    "message",
                ),
                oneof(
                    "bytes",
                    "Vec<u8>",
                    "any_value::Value::BytesValue",
                    7,
                    "bytes",
                ),
            ],
        ),
        (
            "opentelemetry.proto.metrics.v1.Metric.data".into(),
            vec![
                oneof("sum", "Sum", "metric::Data::Sum", 5, "message"),
                oneof("gauge", "Gauge", "metric::Data::Gauge", 6, "message"),
                oneof(
                    "histogram",
                    "Histogram",
                    "metric::Data::Histogram",
                    7,
                    "message",
                ),
                oneof(
                    "exponential_histogram",
                    "ExponentialHistogram",
                    "metric::Data::ExponentialHistogram",
                    8,
                    "message",
                ),
                oneof("summary", "Summary", "metric::Data::Summary", 11, "message"),
            ],
        ),
        (
            "opentelemetry.proto.metrics.v1.NumberDataPoint.value".into(),
            vec![
                oneof(
                    "int",
                    "i64",
                    "number_data_point::Value::AsInt",
                    4,
                    "sfixed64",
                ),
                oneof(
                    "double",
                    "f64",
                    "number_data_point::Value::AsDouble",
                    5,
                    "double",
                ),
            ],
        ),
        (
            "opentelemetry.proto.metrics.v1.Exemplar.value".into(),
            vec![
                oneof("int", "i64", "exemplar::Value::AsInt", 4, "sfixed64"),
                oneof("double", "f64", "exemplar::Value::AsDouble", 5, "double"),
            ],
        ),
    ])
});

/// This lists all the enumerated types and their underlying
/// representation type.
pub static FIELD_TYPE_OVERRIDES: LazyLock<HashMap<&'static str, EnumField>> = LazyLock::new(|| {
    HashMap::from([
        (
            "opentelemetry.proto.logs.v1.LogRecord.flags",
            enumfield("LogRecordFlags", "u32"),
        ),
        (
            "opentelemetry.proto.trace.v1.Span.flags",
            enumfield("SpanFlags", "u32"),
        ),
        (
            "opentelemetry.proto.trace.v1.Span.kind",
            enumfield("span::SpanKind", "i32"),
        ),
        (
            "opentelemetry.proto.trace.v1.Status.code",
            enumfield("status::StatusCode", "i32"),
        ),
        (
            "opentelemetry.proto.metrics.v1.Sum.aggregation_temporality",
            enumfield("AggregationTemporality", "i32"),
        ),
        (
            "opentelemetry.proto.metrics.v1.Histogram.aggregation_temporality",
            enumfield("AggregationTemporality", "i32"),
        ),
        (
            "opentelemetry.proto.metrics.v1.ExponentialHistogram.aggregation_temporality",
            enumfield("AggregationTemporality", "i32"),
        ),
    ])
});

/// This is the entry point from build.rs where we configure prost/tonic.
pub fn add_type_attributes(mut builder: tonic_build::Builder) -> tonic_build::Builder {
    for (name, _) in REQUIRED_PARAMS.iter() {
        let attr = format!(r#"#[crate::pdata::otlp::qualified("{}")]"#, name);
        builder = builder.type_attribute(name, attr);
        builder = builder.type_attribute(name, r#"#[derive(crate::pdata::otlp::Message)]"#);
    }
    builder
}
