// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::OtapPdata;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use otap_df_pdata::proto::opentelemetry::logs::v1::LogsData;
use otap_df_pdata::proto::opentelemetry::metrics::v1::MetricsData;
use otap_df_pdata::proto::opentelemetry::trace::v1::TracesData;
use prost::Message;
use weaver_forge::registry::{ResolvedGroup, ResolvedRegistry};
use weaver_resolved_schema::attribute::Attribute;
use weaver_semconv::attribute::{AttributeType, PrimitiveOrArrayTypeSpec, RequirementLevel};
use weaver_semconv::group::{GroupType, InstrumentSpec, SpanKindSpec};

/// Build a small semantic-conventions registry for generating fake signals in tests.
pub fn build_test_registry() -> ResolvedRegistry {
    let string_attribute = Attribute {
        name: "service.name".to_string(),
        r#type: AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::String),
        brief: "service name".to_string(),
        examples: None,
        tag: None,
        requirement_level: RequirementLevel::default(),
        sampling_relevant: None,
        note: String::new(),
        stability: None,
        deprecated: None,
        prefix: false,
        tags: None,
        annotations: None,
        value: None,
        role: None,
    };
    let int_attribute = Attribute {
        name: "example.counter".to_string(),
        r#type: AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Int),
        brief: "example counter".to_string(),
        examples: None,
        tag: None,
        requirement_level: RequirementLevel::default(),
        sampling_relevant: None,
        note: String::new(),
        stability: None,
        deprecated: None,
        prefix: false,
        tags: None,
        annotations: None,
        value: None,
        role: None,
    };

    let span_group = ResolvedGroup {
        id: "fake_span".to_string(),
        r#type: GroupType::Span,
        brief: "test span group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute.clone()],
        span_kind: Some(SpanKindSpec::Server),
        events: vec!["fake.event".to_string()],
        metric_name: None,
        instrument: None,
        unit: None,
        name: None,
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    let metric_group = ResolvedGroup {
        id: "fake_metric".to_string(),
        r#type: GroupType::Metric,
        brief: "test metric group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute.clone(), int_attribute],
        span_kind: None,
        events: Vec::new(),
        metric_name: Some("requests.count".to_string()),
        instrument: Some(InstrumentSpec::Counter),
        unit: Some("1".to_string()),
        name: None,
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    let event_group = ResolvedGroup {
        id: "fake_event".to_string(),
        r#type: GroupType::Event,
        brief: "test event group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute],
        span_kind: None,
        events: Vec::new(),
        metric_name: None,
        instrument: None,
        unit: None,
        name: Some("app.log".to_string()),
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    ResolvedRegistry {
        registry_url: "test://otlp_fake_registry".to_string(),
        groups: vec![span_group, metric_group, event_group],
    }
}

/// Decode an `OtapPdata` payload back into an OTLP message variant for equivalence checks.
pub fn decode_pdata_to_message(pdata: &OtapPdata) -> OtlpProtoMessage {
    let proto_bytes: OtlpProtoBytes = pdata
        .clone()
        .payload()
        .try_into()
        .expect("convert to proto bytes");
    match proto_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            let request =
                ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode logs payload");
            OtlpProtoMessage::Logs(LogsData {
                resource_logs: request.resource_logs,
            })
        }
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            let request = ExportMetricsServiceRequest::decode(bytes.as_ref())
                .expect("decode metrics payload");
            OtlpProtoMessage::Metrics(MetricsData {
                resource_metrics: request.resource_metrics,
            })
        }
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            let request =
                ExportTraceServiceRequest::decode(bytes.as_ref()).expect("decode traces payload");
            OtlpProtoMessage::Traces(TracesData {
                resource_spans: request.resource_spans,
            })
        }
    }
}
