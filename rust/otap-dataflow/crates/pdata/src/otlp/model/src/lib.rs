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

/// Configuration for required parameters in constructor/builder pattern
#[derive(Clone, Debug, Default)]
pub struct ParamConfig {
    /// Fields that are required parameters (either in new() or via builder)
    pub required: Vec<&'static str>,
    /// Fields that are explicitly ignored (not required, not in builder)
    pub ignored: Vec<&'static str>,
}

/// Create a parameter config with required fields
fn simple(fields: Vec<&'static str>) -> ParamConfig {
    ParamConfig {
        required: fields,
        ignored: vec![],
    }
}

/// Create a parameter config with no required fields (all via builder)
fn detailed() -> ParamConfig {
    ParamConfig {
        required: vec![],
        ignored: vec![],
    }
}

/// Create a parameter config with some required and some ignored fields
fn some_simple(required: Vec<&'static str>, ignored: Vec<&'static str>) -> ParamConfig {
    ParamConfig { required, ignored }
}

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
pub static REQUIRED_PARAMS: LazyLock<HashMap<&'static str, ParamConfig>> = LazyLock::new(|| {
    HashMap::from([
        // Common
        //
        // SIMPLE
        //
        (
            "opentelemetry.proto.common.v1.AnyValue",
            simple(vec!["value"]),
        ),
        (
            "opentelemetry.proto.common.v1.KeyValue",
            simple(vec!["key", "value"]),
        ),
        (
            "opentelemetry.proto.common.v1.KeyValueList",
            simple(vec!["values"]),
        ),
        (
            "opentelemetry.proto.common.v1.ArrayValue",
            simple(vec!["values"]),
        ),
        //
        // Common
        //
        // DETAILED
        //
        (
            "opentelemetry.proto.common.v1.InstrumentationScope",
            detailed(),
        ),
        ("opentelemetry.proto.common.v1.EntityRef", detailed()),
        //
        // Resource
        //
        // DETAILED
        //
        ("opentelemetry.proto.resource.v1.Resource", detailed()),
        //
        // Logs
        //
        // SIMPLE
        //
        (
            "opentelemetry.proto.logs.v1.ScopeLogs",
            some_simple(vec!["scope", "log_records"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.logs.v1.ResourceLogs",
            some_simple(vec!["resource", "scope_logs"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.logs.v1.LogsData",
            simple(vec!["resource_logs"]),
        ),
        //
        // Logs
        //
        // DETAILED
        //
        ("opentelemetry.proto.logs.v1.LogRecord", detailed()),
        //
        // Traces
        //
        // SIMPLE
        //
        (
            "opentelemetry.proto.trace.v1.ScopeSpans",
            some_simple(vec!["scope", "spans"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.trace.v1.ResourceSpans",
            some_simple(vec!["resource", "scope_spans"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.trace.v1.TracesData",
            simple(vec!["resource_spans"]),
        ),
        (
            "opentelemetry.proto.trace.v1.Status",
            simple(vec!["code", "message"]),
        ),
        //
        // Traces
        //
        // DETAILED
        //
        ("opentelemetry.proto.trace.v1.Span", detailed()),
        ("opentelemetry.proto.trace.v1.Span.Link", detailed()),
        ("opentelemetry.proto.trace.v1.Span.Event", detailed()),
        //
        // Metrics
        //
        // SIMPLE
        //
        (
            "opentelemetry.proto.metrics.v1.ScopeMetrics",
            some_simple(vec!["scope", "metrics"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.ResourceMetrics",
            some_simple(vec!["resource", "scope_metrics"], vec!["schema_url"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.MetricsData",
            simple(vec!["resource_metrics"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.Sum",
            simple(vec![
                "aggregation_temporality",
                "is_monotonic",
                "data_points",
            ]),
        ),
        (
            "opentelemetry.proto.metrics.v1.Gauge",
            simple(vec!["data_points"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.Histogram",
            simple(vec!["aggregation_temporality", "data_points"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.ExponentialHistogram",
            simple(vec!["aggregation_temporality", "data_points"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.Summary",
            simple(vec!["data_points"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint.Buckets",
            simple(vec!["offset", "bucket_counts"]),
        ),
        (
            "opentelemetry.proto.metrics.v1.SummaryDataPoint.ValueAtQuantile",
            simple(vec!["quantile", "value"]),
        ),
        //
        // Metrics
        //
        // DETAILED
        //
        ("opentelemetry.proto.metrics.v1.Metric", detailed()),
        ("opentelemetry.proto.metrics.v1.NumberDataPoint", detailed()),
        (
            "opentelemetry.proto.metrics.v1.HistogramDataPoint",
            detailed(),
        ),
        (
            "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint",
            detailed(),
        ),
        (
            "opentelemetry.proto.metrics.v1.SummaryDataPoint",
            detailed(),
        ),
        ("opentelemetry.proto.metrics.v1.Exemplar", detailed()),
        //
        // Service
        //
        // SIMPLE
        //
        (
            "opentelemetry.proto.collector.logs.v1.ExportLogsServiceRequest",
            simple(vec!["resource_logs"]),
        ),
        (
            "opentelemetry.proto.collector.logs.v1.ExportLogsServiceResponse",
            simple(vec!["partial_success"]),
        ),
        (
            "opentelemetry.proto.collector.logs.v1.ExportLogsPartialSuccess",
            simple(vec!["rejected_log_records", "error_message"]),
        ),
        (
            "opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceRequest",
            simple(vec!["resource_metrics"]),
        ),
        (
            "opentelemetry.proto.collector.metrics.v1.ExportMetricsServiceResponse",
            simple(vec!["partial_success"]),
        ),
        (
            "opentelemetry.proto.collector.metrics.v1.ExportMetricsPartialSuccess",
            simple(vec!["rejected_data_points", "error_message"]),
        ),
        (
            "opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest",
            simple(vec!["resource_spans"]),
        ),
        (
            "opentelemetry.proto.collector.trace.v1.ExportTraceServiceResponse",
            simple(vec!["partial_success"]),
        ),
        (
            "opentelemetry.proto.collector.trace.v1.ExportTracePartialSuccess",
            simple(vec!["rejected_spans", "error_message"]),
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
pub fn add_type_attributes(mut builder: tonic_prost_build::Builder) -> tonic_prost_build::Builder {
    for (name, _) in REQUIRED_PARAMS.iter() {
        let attr = format!(r#"#[crate::otlp::qualified("{}")]"#, name);
        builder = builder.type_attribute(name, attr);
        builder = builder.type_attribute(name, r#"#[derive(crate::otlp::Message)]"#);
    }
    builder
}
