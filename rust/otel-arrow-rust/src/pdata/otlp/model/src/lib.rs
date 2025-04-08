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

use std::sync::LazyLock;

enum Kind {
    Normal,
    Oneof,
    Value,
}

struct Detail {
    name: &'static str,
    kind: Kind,
}

static DETAILS: LazyLock<Vec<Detail>> = LazyLock::new(|| {
    vec![
        Detail {
            name: "opentelemetry.proto.trace.v1.Span",
            kind: Kind::Normal,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue",
            kind: Kind::Value,
        },
        Detail {
            name: "opentelemetry.proto.metrics.v1.Metric.data",
            kind: Kind::Oneof,
        },
        // Detail {
        //     name: "opentelemetry.proto.common.v1.AnyValue.value",
        //     kind: Kind::Oneof,
        // },
    ]
});

pub fn add_type_attributes(mut builder: tonic_build::Builder) -> tonic_build::Builder {
    for det in DETAILS.iter() {
        match det.kind {
            Kind::Value => {
                builder =
                    builder.type_attribute(det.name, r#"#[derive(crate::pdata::otlp::Value)]"#);
            }
            Kind::Normal => {
                builder =
                    builder.type_attribute(det.name, r#"#[derive(crate::pdata::otlp::Message)]"#);
            }
            Kind::Oneof => {
                builder =
                    builder.type_attribute(det.name, r#"#[derive(crate::pdata::otlp::Oneof)]"#);
            }
        }
    }
    builder
}
