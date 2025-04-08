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

pub enum Kind {
    Message, // ordinary message
    Value,   // special case for AnyValue
    Ignore,  // special case for oneof fields
}

pub struct Field {
    pub name: &'static str,
    pub ty: &'static str,
    pub bound: &'static str,
    pub get: &'static str,
}

pub struct Detail {
    pub name: &'static str,
    pub kind: Kind,
    pub direct: bool,
    pub derive: Option<&'static str>,
    pub params: Option<Vec<Field>>,
}

pub static DETAILS: LazyLock<Vec<Detail>> = LazyLock::new(|| {
    vec![
        Detail {
            name: "opentelemetry.proto.trace.v1.Span",
            kind: Kind::Message,
	    direct: false,
	    derive: None,
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue",
            kind: Kind::Value,
	    direct: true,
	    derive: None, // Some("Eq"),
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.AnyValue.value",
            kind: Kind::Ignore,
	    direct: false,
	    derive: None, // Some("Eq"),
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.ArrayValue",
            kind: Kind::Message,
	    direct: false,
	    derive: None, // Some("Eq"),
            params: None,
        },
        Detail {
            name: "opentelemetry.proto.common.v1.KeyValue",
            kind: Kind::Message,
	    direct: true,
	    derive: None, // Some("Eq"),
            params: Some(vec![
                Field{
		    name: "key",
		    ty: "S",
		    bound: "AsRef<str>",
		    get: "as_ref().to_string()",
		},
                Field{
                    name: "value",
		    ty:"AnyValue",
		    bound: "",
		    get: "",
		},
            ]),
        },
    ]
});

pub fn add_type_attributes(mut builder: tonic_build::Builder) -> tonic_build::Builder {
    for det in DETAILS.iter() {
        match det.kind {
            Kind::Ignore => {
                builder =
                    builder.type_attribute(det.name,
					   format!(r#"#[derive({})]"#,
						   det.derive.unwrap_or(""),
					   ));
	    }
            Kind::Value => {
                builder =
                    builder.type_attribute(det.name,
					   format!(r#"#[derive(crate::pdata::otlp::Value,{})]"#,
						   det.derive.unwrap_or(""),
					   ));
            }
            Kind::Message => {
                builder =
                    builder.type_attribute(det.name,
					   format!(r#"#[derive(crate::pdata::otlp::Message,{})]"#,
						   det.derive.unwrap_or(""),
					   ));
            }
        }
    }
    builder
}
