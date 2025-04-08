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
pub enum Kind {
    Message, // ordinary message
    Value,   // special case for AnyValue

    #[default]
    Ignore, // special case for oneof fields
}

#[derive(Clone, Debug, Default)]
pub struct Field {
    pub name: &'static str,
    pub ty: &'static str,
    pub bound: &'static str,
    pub get: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct Detail {
    pub name: &'static str,
    pub kind: Kind,
    pub params: Option<Vec<Field>>,
}

pub static ALL_KNOWN_TYPES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        // Common types
        "opentelemetry.proto.common.v1.AnyValue",
        "opentelemetry.proto.common.v1.KeyValue",
        "opentelemetry.proto.common.v1.InstrumentationScope",
        // types unused b/c these are just vectors
        // "opentelemetry.proto.common.v1.ArrayValue",
        // "opentelemetry.proto.common.v1.KeyValueList",

        // Resource types
        "opentelemetry.proto.resource.v1.Resource",

        // Log types
        "opentelemetry.proto.logs.v1.LogRecord",
        "opentelemetry.proto.logs.v1.LogsData",
        "opentelemetry.proto.logs.v1.ResourceLogs",
        "opentelemetry.proto.logs.v1.ScopeLogs",
        "opentelemetry.proto.logs.v1.SeverityNumber",

        // Trace types
        "opentelemetry.proto.trace.v1.Span",
        "opentelemetry.proto.trace.v1.TracesData",
        "opentelemetry.proto.trace.v1.ResourceSpans",
        "opentelemetry.proto.trace.v1.ScopeSpans",
        "opentelemetry.proto.trace.v1.SpanKind",
        "opentelemetry.proto.trace.v1.Status",
        "opentelemetry.proto.trace.v1.Status.StatusCode",

        // Metric types
        "opentelemetry.proto.metrics.v1.Metric",
        "opentelemetry.proto.metrics.v1.MetricsData",
        "opentelemetry.proto.metrics.v1.ResourceMetrics",
        "opentelemetry.proto.metrics.v1.ScopeMetrics",
        "opentelemetry.proto.metrics.v1.Gauge",
        "opentelemetry.proto.metrics.v1.Sum",
        "opentelemetry.proto.metrics.v1.Histogram",
        "opentelemetry.proto.metrics.v1.ExponentialHistogram",
        "opentelemetry.proto.metrics.v1.Summary",
        "opentelemetry.proto.metrics.v1.NumberDataPoint",
        "opentelemetry.proto.metrics.v1.HistogramDataPoint",
        "opentelemetry.proto.metrics.v1.ExponentialHistogramDataPoint",
        "opentelemetry.proto.metrics.v1.SummaryDataPoint",
        "opentelemetry.proto.metrics.v1.AggregationTemporality",
    ]
});

pub static FIELD_TYPE_OVERRIDES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "opentelemetry.proto.logs.v1.LogRecord.flags",
            "LogRecordFlags",
        );
        m
    });

pub static DETAILS: LazyLock<Vec<Detail>> = LazyLock::new(|| {
    vec![
        // Logs
        Detail {
            name: "opentelemetry.proto.logs.v1.LogRecord",
            kind: Kind::Message,
            params: Some(vec![
                Field {
                    name: "time_unix_nano",
                    ty: "u64",
                    bound: "",
                    get: "",
                },
                Field {
                    name: "severity_number",
                    ty: "SeverityNumber",
                    bound: "",
                    get: "into()",
                },
                Field {
                    name: "event_name",
                    ty: "S",
                    bound: "AsRef<str>",
                    get: "as_ref().to_string()",
                },
            ]),
        },
        // Trace
        Detail {
            name: "opentelemetry.proto.trace.v1.Span",
            kind: Kind::Message,
            params: None,
        },
        // Common
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue",
            kind: Kind::Value,
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue.value",
            kind: Kind::Ignore,
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.ArrayValue",
            kind: Kind::Ignore,
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValueList",
            kind: Kind::Ignore,
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValue",
            kind: Kind::Message,
            params: Some(vec![
                Field {
                    name: "key",
                    ty: "S",
                    bound: "AsRef<str>",
                    get: "as_ref().to_string()",
                },
                Field {
                    name: "value",
                    ty: "AnyValue",
                    bound: "",
                    get: "",
                },
            ]),
        },
    ]
});

/// This is the entry point from build.rs where we configure prost/tonic.
pub fn add_type_attributes(mut builder: tonic_build::Builder) -> tonic_build::Builder {
    for name in ALL_KNOWN_TYPES.iter() {
        // Add the fully qualified protobuf type name as an attribute
        builder = builder.type_attribute(
            name,
            &format!(r#"#[crate::pdata::otlp::qualified("{}")]"#, name),
        );
	if *name == "opentelemetry.proto.common.v1.AnyValue" {
                builder = builder.type_attribute(
                    name,
		    r#"#[derive(crate::pdata::otlp::Value)]"#,
		)
	} else {
                builder = builder.type_attribute(
                    name,
		    r#"#[derive(crate::pdata::otlp::Message)]"#,
		)
	}
    }
    builder
}
