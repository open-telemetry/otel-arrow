// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Debug, Default)]
pub struct Detail {
    pub name: &'static str,
    pub params: Option<Vec<&'static str>>,
}

#[derive(Clone, Debug, Default)]
pub struct OneofCase {
    pub name: &'static str,
    pub type_param: &'static str,
    pub value_variant: &'static str,
    pub extra_call: Option<&'static str>,
}

#[derive(Clone, Debug, Default)]
pub struct OneofMapping {
    pub field: &'static str,
    pub cases: Vec<OneofCase>,
}

fn oneof(name: &'static str, type_param: &'static str, value_variant: &'static str, extra_call: Option<&'static str>) -> OneofCase {
    OneofCase{name, type_param, value_variant, extra_call}
}

pub static ONEOF_MAPPINGS: LazyLock<Vec<OneofMapping>> = LazyLock::new(|| {
    vec![
	OneofMapping{
	    field: "opentelemetry.proto.common.v1.AnyValue.value",
	    cases: vec![
		oneof("string", "::prost::alloc::string::String", "any_value::Value::StringValue", None ),
		oneof("bool", "bool", "any_value::Value::BoolValue", None),
		oneof("int", "i64", "any_value::Value::IntValue", None),
		oneof("double", "f64", "any_value::Value::DoubleValue", None),
		oneof("kvlist", "Vec<KeyValue>", "any_value::Value::KvlistValue", Some("KeyValueList::new")),
		oneof("array", "Vec<AnyValue>", "any_value::Value::ArrayValue", Some("ArrayValue::new")),
		oneof("bytes", "Vec<u8>", "any_value::Value::BytesValue", None),
	    ],
	},
    ]
});

pub static ALL_KNOWN_TYPES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        // Common types
        "opentelemetry.proto.common.v1.AnyValue",
        "opentelemetry.proto.common.v1.EntityRef",
        "opentelemetry.proto.common.v1.InstrumentationScope",
        "opentelemetry.proto.common.v1.KeyValue",
        "opentelemetry.proto.common.v1.KeyValueList",
        "opentelemetry.proto.common.v1.ArrayValue",
        // Resource types
        "opentelemetry.proto.resource.v1.Resource",
        // Log types
        "opentelemetry.proto.logs.v1.LogsData",
        "opentelemetry.proto.logs.v1.ResourceLogs",
        "opentelemetry.proto.logs.v1.ScopeLogs",
        "opentelemetry.proto.logs.v1.LogRecord",
        // Trace types
        "opentelemetry.proto.trace.v1.TracesData",
        "opentelemetry.proto.trace.v1.ResourceSpans",
        "opentelemetry.proto.trace.v1.ScopeSpans",
        "opentelemetry.proto.trace.v1.Span",
        "opentelemetry.proto.trace.v1.Span.Kind",
        "opentelemetry.proto.trace.v1.Span.Link",
        "opentelemetry.proto.trace.v1.Span.Event",
        "opentelemetry.proto.trace.v1.Status",
        // Metric types
        "opentelemetry.proto.metrics.v1.MetricsData",
        "opentelemetry.proto.metrics.v1.ResourceMetrics",
        "opentelemetry.proto.metrics.v1.ScopeMetrics",
        "opentelemetry.proto.metrics.v1.Metric",
        "opentelemetry.proto.metrics.v1.Sum",
        "opentelemetry.proto.metrics.v1.Gauge",
        "opentelemetry.proto.metrics.v1.Histogram",
        "opentelemetry.proto.metrics.v1.ExponentialHistogram",
        "opentelemetry.proto.metrics.v1.Summary",
        "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint",
        "opentelemetry.proto.metrics.v1.HistogramDataPoint",
        "opentelemetry.proto.metrics.v1.NumberDataPoint",
        "opentelemetry.proto.metrics.v1.SummaryDataPoint",
    ]
});

pub struct Override {
    pub datatype: &'static str,
    pub fieldtype: &'static str,
}

pub static FIELD_TYPE_OVERRIDES: LazyLock<HashMap<&'static str, Override>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "opentelemetry.proto.logs.v1.LogRecord.flags",
        Override {
            datatype: "LogRecordFlags",
            fieldtype: "u32",
        },
    );
    m.insert(
        "opentelemetry.proto.trace.v1.Span.flags",
        Override {
            datatype: "SpanFlags",
            fieldtype: "u32",
        },
    );
    m.insert(
        "opentelemetry.proto.trace.v1.Span.kind",
        Override {
            datatype: "span::SpanKind",
            fieldtype: "i32",
        },
    );
    m.insert(
        "opentelemetry.proto.trace.v1.Status.code",
        Override {
            datatype: "status::StatusCode",
            fieldtype: "i32",
        },
    );
    m.insert(
        "opentelemetry.proto.metrics.v1.Sum.aggregation_temporality",
        Override {
            datatype: "AggregationTemporality",
            fieldtype: "i32",
        },
    );
    m.insert(
        "opentelemetry.proto.metrics.v1.Histogram.aggregation_temporality",
        Override {
            datatype: "AggregationTemporality",
            fieldtype: "i32",
        },
    );
    m.insert(
        "opentelemetry.proto.metrics.v1.ExponentialHistogram.aggregation_temporality",
        Override {
            datatype: "AggregationTemporality",
            fieldtype: "i32",
        },
    );
    m
});

pub static DETAILS: LazyLock<Vec<Detail>> = LazyLock::new(|| {
    vec![
	// Common
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue",
            params: Some(vec!["value"]),
        },
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValue",
            params: Some(vec!["key", "value"]),
        },
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValueList",
            params: Some(vec!["values"]),
        },
        Detail {
            name: "opentelemetry.proto.common.v1.ArrayValue",
            params: Some(vec!["values"]),
        },
        Detail {
            name: "opentelemetry.proto.common.v1.InstrumentationScope",
            params: Some(vec!["name"]),
        },
        Detail {
            name: "opentelemetry.proto.common.v1.EntityRef",
            params: Some(vec!["r#type"]),
        },
        // Resource
        Detail {
            name: "opentelemetry.proto.resource.v1.Resource",
            params: Some(vec!["attributes"]),
        },
        // Logs
        Detail {
            name: "opentelemetry.proto.logs.v1.LogRecord",
            params: Some(vec!["time_unix_nano", "severity_number", "event_name"]),
        },
        Detail {
            name: "opentelemetry.proto.logs.v1.ScopeLogs",
            params: Some(vec!["scope"]),
        },
        Detail {
            name: "opentelemetry.proto.logs.v1.ResourceLogs",
            params: Some(vec!["resource"]),
        },
        Detail {
            name: "opentelemetry.proto.logs.v1.LogsData",
            params: Some(vec!["resource_logs"]),
        },
        // Traces
        Detail {
            name: "opentelemetry.proto.trace.v1.Span",
            params: Some(vec!["trace_id", "span_id", "name", "start_time_unix_nano"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.ScopeSpans",
            params: Some(vec!["scope"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.ResourceSpans",
            params: Some(vec!["resource"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.TracesData",
            params: Some(vec!["resource_spans"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.Status",
            params: Some(vec!["message", "code"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.Span.Link",
            params: Some(vec!["trace_id", "span_id"]),
        },
        Detail {
            name: "opentelemetry.proto.trace.v1.Span.Event",
            params: Some(vec!["name", "time_unix_nano"]),
        },
        // Metrics
        Detail {
            name: "opentelemetry.proto.metrics.v1.ScopeMetrics",
            params: Some(vec!["scope"]),
        },
        Detail {
            name: "opentelemetry.proto.metrics.v1.ResourceMetrics",
            params: Some(vec!["resource"]),
        },
        Detail {
            name: "opentelemetry.proto.metrics.v1.MetricsData",
            params: Some(vec!["resource_metrics"]),
        },
        Detail {
            name: "opentelemetry.proto.metrics.v1.Metric",
            params: Some(vec!["name", "data"]),
        },
    ]
});

/// This is the entry point from build.rs where we configure prost/tonic.
pub fn add_type_attributes(mut builder: tonic_build::Builder) -> tonic_build::Builder {
    for name in ALL_KNOWN_TYPES.iter() {
        builder = builder.type_attribute(
            name,
            &format!(r#"#[crate::pdata::otlp::qualified("{}")]"#, name),
        );
        builder = builder.type_attribute(name, r#"#[derive(crate::pdata::otlp::Message)]"#);
    }
    builder
}
