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

pub static ALL_KNOWN_TYPES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        // Common types
        "opentelemetry.proto.common.v1.AnyValue",
        "opentelemetry.proto.common.v1.KeyValue",
        "opentelemetry.proto.common.v1.InstrumentationScope",
        "opentelemetry.proto.common.v1.EntityRef",

        // Resource types
        "opentelemetry.proto.resource.v1.Resource",

        // Log types
        "opentelemetry.proto.logs.v1.LogsData",
        "opentelemetry.proto.logs.v1.ResourceLogs",
        "opentelemetry.proto.logs.v1.ScopeLogs",
        "opentelemetry.proto.logs.v1.LogRecord",

        // Trace types
        "opentelemetry.proto.trace.v1.Span",
        "opentelemetry.proto.trace.v1.TracesData",
        "opentelemetry.proto.trace.v1.ResourceSpans",
        "opentelemetry.proto.trace.v1.ScopeSpans",
        "opentelemetry.proto.trace.v1.SpanKind",
        "opentelemetry.proto.trace.v1.Status",

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
    ]
});

pub struct Override {
    pub datatype: &'static str,
    pub fieldtype: &'static str,
}

pub static FIELD_TYPE_OVERRIDES: LazyLock<HashMap<&'static str, Override>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "opentelemetry.proto.logs.v1.LogRecord.flags",
            Override{
		datatype: "LogRecordFlags",
		fieldtype: "u32",
	    },
        );
        m
    });

pub static DETAILS: LazyLock<Vec<Detail>> = LazyLock::new(|| {
    vec![
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

	// Common: Note: AnyValue is a special case.
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValue",
            params: Some(vec!["key", "value"]),
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
