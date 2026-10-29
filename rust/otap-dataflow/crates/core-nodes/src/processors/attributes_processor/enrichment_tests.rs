// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::enrichment::{context_entries, resolve_source, validate_source};
use super::*;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
};
use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
use otel_arrow_dfe_config::transport_headers_policy::PropagationSelector;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::context_declaration::ContextMaterializationPlan;
use otel_arrow_dfe_engine::control::{
    CallData, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
};
use otel_arrow_dfe_engine::testing::{node::test_node, processor::TestRuntime};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    metrics::v1::{Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric::Data},
    resource::v1::Resource,
    trace::v1::{ResourceSpans, ScopeSpans, Span},
};
use otel_arrow_dfe_pdata_codec::{CodecService, OtapPayload};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use prost::Message as _;
use serde_json::json;

fn pipeline_context() -> PipelineContext {
    ControllerContext::new(TelemetryRegistryHandle::new()).pipeline_context_with(
        "test".into(),
        "attribute".into(),
        0,
        1,
        0,
    )
}

fn declared_pipeline_context(config: &Value) -> PipelineContext {
    use otel_arrow_dfe_engine::context_declaration::{
        ContextConsumerSelector, ContextDeclaration, ContextEntryTarget,
    };
    let parsed: Config = serde_json::from_value(config.clone()).unwrap();
    let mut entries = BTreeMap::<String, BTreeSet<String>>::new();
    for declaration in parsed.context_declarations() {
        let ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::Entries { entries: selected },
        } = declaration
        else {
            continue;
        };
        for selected in selected {
            match selected.target {
                ContextEntryTarget::Composite { name } => {
                    _ = entries
                        .entry(name.to_string())
                        .or_default()
                        .insert("value".into());
                }
                ContextEntryTarget::CompositeMember { composite, member } => {
                    _ = entries
                        .entry(composite.to_string())
                        .or_default()
                        .insert(member.to_string());
                }
                _ => {}
            }
        }
    }
    let entries: BTreeMap<_, _> = entries
        .into_iter()
        .map(|(name, members)| {
            (
                name,
                members
                    .into_iter()
                    .map(|member| json!({"type":"constant","name":member,"value":"test"}))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let prepared = prepare_context(config, json!(entries)).unwrap();
    let mut ctx = pipeline_context().with_node_context(
        "attributes".into(),
        ATTRIBUTES_PROCESSOR_URN.into(),
        otel_arrow_dfe_config::node::NodeKind::Processor,
        Default::default(),
    );
    ctx.set_compiled_context_bindings(prepared.bindings);
    ctx
}

fn prepare_context(
    config: &Value,
    entries: Value,
) -> Result<otel_arrow_dfe_engine::context_declaration::PreparedContext, EngineError> {
    let spec: otel_arrow_dfe_config::engine::OtelDataflowSpec = serde_json::from_value(json!({
        "version":"otel_dataflow/v1", "engine":{},
        "policies":{"context":{"entries":entries}},
        "groups":{"test":{"pipelines":{"attribute":{
            "nodes":{
                "receiver":{"type":"receiver:otlp","config":{"protocols":{"http":{"listening_addr":"127.0.0.1:0"}}}},
                "attributes":{"type":"processor:attribute","config":config},
                "exporter":{"type":"exporter:noop","config":{}}
            },
            "connections":[{"from":"receiver","to":"attributes"},{"from":"attributes","to":"exporter"}]
        }}}}
    })).unwrap();
    let factory = otel_arrow_dfe_engine::PipelineFactory::new(
        &otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES,
        &OTAP_PROCESSOR_FACTORIES,
        &otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES,
        &[],
    );
    factory.compile_initial_context(&spec.resolve())
}

fn header_context(headers: &[(&str, ValueKind, &[u8])]) -> Context {
    let mut context = Context::default();
    let mut captured = TransportHeaders::new();
    for (name, kind, value) in headers {
        captured.push(TransportHeader::new(
            ContextEntryName::try_from(*name).unwrap(),
            *kind,
            value.to_vec(),
        ));
    }
    context.set_transport_headers(captured);
    context
}

fn store_context_text(context: &mut Context, name: &str, text: String) {
    let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
        scope: ContextScope::Engine,
        name: ContextEntryName::try_from(name).unwrap(),
        definition: ContextEntryDefinition(vec![ContextEntryPart::Constant {
            name: ContextEntryName::try_from("value").unwrap(),
            value: text,
        }]),
    }])
    .unwrap();
    context.materialize_context(&plan);
}

fn run(
    config: Value,
    messages: Vec<Message<OtapPdata>>,
    check: impl FnOnce(Vec<OtapPdata>) + 'static,
) {
    let rt = TestRuntime::new();
    let mut node_config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
    node_config.config = config;
    let processor = create_attributes_processor(
        declared_pipeline_context(&node_config.config),
        test_node("attribute-enrichment"),
        Arc::new(node_config),
        rt.config(),
    )
    .unwrap();
    rt.set_processor(processor)
        .run_test(|mut ctx| async move {
            for message in messages {
                ctx.process(message).await.unwrap();
            }
            check(ctx.drain_pdata().await);
        })
        .validate(|_| async {});
}

fn attrs() -> Vec<KeyValue> {
    ["insert_existing", "upsert_existing", "update_existing"]
        .into_iter()
        .map(|key| KeyValue::new(key, AnyValue::new_string("old")))
        .collect()
}

fn fixture(signal: SignalType, observed: &[u64]) -> OtapPayload {
    let resource = Some(Resource {
        attributes: attrs(),
        ..Default::default()
    });
    let scope = Some(InstrumentationScope {
        attributes: attrs(),
        ..Default::default()
    });
    match signal {
        SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource,
                    scope_logs: vec![ScopeLogs {
                        scope,
                        log_records: observed
                            .iter()
                            .map(|observed| LogRecord {
                                time_unix_nano: 123,
                                observed_time_unix_nano: *observed,
                                attributes: attrs(),
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec()
            .into(),
        )
        .into(),
        SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(
            ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    resource,
                    scope_metrics: vec![ScopeMetrics {
                        scope,
                        metrics: vec![Metric {
                            name: "test".into(),
                            data: Some(Data::Gauge(Gauge {
                                data_points: vec![NumberDataPoint {
                                    attributes: attrs(),
                                    ..Default::default()
                                }],
                            })),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec()
            .into(),
        )
        .into(),
        SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(
            ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans {
                    resource,
                    scope_spans: vec![ScopeSpans {
                        scope,
                        spans: vec![Span {
                            trace_id: vec![1; 16],
                            span_id: vec![2; 8],
                            attributes: attrs(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
            .encode_to_vec()
            .into(),
        )
        .into(),
    }
}

fn domains(payload: OtapPayload) -> Vec<Vec<KeyValue>> {
    let bytes: OtlpProtoBytes = payload.try_into_with_default().unwrap();
    match bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            let mut data = ExportLogsServiceRequest::decode(bytes).unwrap();
            let mut resource = data.resource_logs.remove(0);
            let mut scope = resource.scope_logs.remove(0);
            vec![
                resource.resource.unwrap().attributes,
                scope.scope.unwrap().attributes,
                scope.log_records.remove(0).attributes,
            ]
        }
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            let mut data = ExportMetricsServiceRequest::decode(bytes).unwrap();
            let mut resource = data.resource_metrics.remove(0);
            let mut scope = resource.scope_metrics.remove(0);
            let Some(Data::Gauge(mut gauge)) = scope.metrics.remove(0).data else {
                panic!("gauge")
            };
            vec![
                resource.resource.unwrap().attributes,
                scope.scope.unwrap().attributes,
                gauge.data_points.remove(0).attributes,
            ]
        }
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            let mut data = ExportTraceServiceRequest::decode(bytes).unwrap();
            let mut resource = data.resource_spans.remove(0);
            let mut scope = resource.scope_spans.remove(0);
            vec![
                resource.resource.unwrap().attributes,
                scope.scope.unwrap().attributes,
                scope.spans.remove(0).attributes,
            ]
        }
    }
}

fn value<'a>(attrs: &'a [KeyValue], key: &str) -> Option<&'a AnyValue> {
    attrs
        .iter()
        .find(|attr| attr.key == key)
        .and_then(|attr| attr.value.as_ref())
}

/// Scenario: Source fields, transforms, and signal selections are invalid.
/// Guarantees: Both factory validation and construction reject invalid or ambiguous configuration.
#[test]
fn rejects_invalid_enrichment_configuration() {
    for action in [
        json!({"action":"insert","key":"x"}),
        json!({"action":"insert","key":"x","value":{"literal":1}}),
        json!({"action":"insert","key":"x","value":null}),
        json!({"action":"update","key":"x","value":{"from_context":"x","from_headers":["host"]}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"none"}}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"all_captured"}}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["entry:member"]}}}),
        json!({"action":"upsert","key":"x","value":{"source":{"type":"transport_header","named":[]}}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":[""]}}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"named","named":[" "]}}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"named","named":["x"]},"transform":"unknown"}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"named","named":["x"]},"value_type":"int"}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"named","named":["x"]},"transform":null}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["host"]},"transform":"authority_host","selection":"middle"}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["host"]},"transform":"authority_host","selection":null}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["host"]},"selection":"first"}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"named","named":["x"]},"transform":"int","selection":"last"}}),
        json!({"action":"insert","key":"x","value":{"string":"host","selection":"last"}}),
        json!({"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["host"]},"transform":"unknown"}}),
        json!({"action":"insert","key":"x","value":1,"signals":[]}),
        json!({"action":"delete","key":"x","signals":["unknown"]}),
        json!({"action":"insert","key":"x","signals":null}),
    ] {
        let config = json!({"actions":[action]});
        assert!(validate_config(&config).is_err(), "{config}");
        assert!(
            AttributesProcessor::from_config(pipeline_context(), &config).is_err(),
            "{config}"
        );
    }
    for literal in [json!(true), json!(42), json!(1.5), json!("text")] {
        validate_config(&json!({"actions":[{"action":"insert","key":"x","value":literal}]}))
            .unwrap();
    }
}

/// Scenario: Documented typed literals and existing scalar literals configure each value action.
/// Guarantees: Both forms preserve scalar types through every signal, domain, and encoding; ambiguous objects fail.
#[test]
fn documented_typed_literals_preserve_scalar_behavior() {
    for (typed, scalar, expected) in [
        (
            json!({"string":"prod"}),
            json!("prod"),
            LiteralValue::Str("prod".into()),
        ),
        (
            json!({"string":""}),
            json!(""),
            LiteralValue::Str(String::new()),
        ),
        (
            json!({"string":"123"}),
            json!("123"),
            LiteralValue::Str("123".into()),
        ),
        (
            json!({"bool":false}),
            json!(false),
            LiteralValue::Bool(false),
        ),
        (
            json!({"int":i64::MIN}),
            json!(i64::MIN),
            LiteralValue::Int(i64::MIN),
        ),
        (
            json!({"int":i64::MAX}),
            json!(i64::MAX),
            LiteralValue::Int(i64::MAX),
        ),
        (
            json!({"double":1.25}),
            json!(1.25),
            LiteralValue::Double(1.25),
        ),
    ] {
        for action in ["insert", "upsert", "update"] {
            for value in [&typed, &scalar] {
                let config = json!({"actions":[{"action":action,"key":"x","value":value}]});
                validate_config(&config).unwrap();
                let parsed: ConfiguredValue = serde_json::from_value(value.clone()).unwrap();
                assert_eq!(serde_json::to_value(&parsed).unwrap(), *value);
                match parsed {
                    ConfiguredValue::Literal(value) | ConfiguredValue::Typed(value) => {
                        assert_eq!(value, expected);
                    }
                    ConfiguredValue::Dynamic { .. } => panic!("expected literal"),
                }
            }
        }
        let expected = match expected {
            LiteralValue::Str(value) => AnyValue::new_string(value),
            LiteralValue::Bool(value) => AnyValue::new_bool(value),
            LiteralValue::Int(value) => AnyValue::new_int(value),
            LiteralValue::Double(value) => AnyValue::new_double(value),
        };
        for configured_value in [&typed, &scalar] {
            for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
                for native in [false, true] {
                    let mut payload = fixture(signal, &[0]);
                    if native {
                        payload = payload
                            .try_into_otap(&CodecService::new().unwrap())
                            .unwrap()
                            .into();
                    }
                    let expected = expected.clone();
                    run(
                        json!({
                            "apply_to":["resource","scope","signal"],
                            "actions":[
                                {"action":"insert","key":"insert_new","value":configured_value},
                                {"action":"insert","key":"insert_existing","value":configured_value},
                                {"action":"upsert","key":"upsert_existing","value":configured_value},
                                {"action":"update","key":"update_existing","value":configured_value},
                                {"action":"update","key":"update_missing","value":configured_value}
                            ]
                        }),
                        vec![Message::PData(OtapPdata::new(Context::default(), payload))],
                        move |mut output| {
                            assert_eq!(output.len(), 1);
                            for attributes in domains(output.remove(0).into_parts().1) {
                                for key in ["insert_new", "upsert_existing", "update_existing"] {
                                    assert_eq!(
                                        value(&attributes, key),
                                        Some(&expected),
                                        "{signal:?} {native} {key}"
                                    );
                                }
                                assert_eq!(
                                    value(&attributes, "insert_existing"),
                                    Some(&AnyValue::new_string("old"))
                                );
                                assert_eq!(value(&attributes, "update_missing"), None);
                            }
                        },
                    );
                }
            }
        }
    }
    for value in [
        json!({"string":"prod","source":{"type":"transport_header","named":["host"]}}),
        json!({"string":"prod","transform":"authority_host"}),
        json!({"string":"prod","int":1}),
        json!({"int":"1"}),
        json!({"bool":null}),
        json!({"int":u64::MAX}),
    ] {
        assert!(
            validate_config(&json!({"actions":[
                {"action":"insert","key":"x","value":value}
            ]}))
            .is_err()
        );
    }
}

/// Scenario: Captured headers arrive out of preference order and include unusable repeated values.
/// Guarantees: Stored-name priority wins and binary, empty, and invalid UTF-8 values fall through without normalizing text.
#[test]
fn ordered_headers_skip_unusable_candidates() {
    let context = header_context(&[
        ("host", ValueKind::Text, b"last.example:80"),
        ("preferred", ValueKind::Binary, b"binary.example"),
        ("preferred", ValueKind::Text, &[0xff]),
        ("preferred", ValueKind::Text, b""),
        ("preferred", ValueKind::Text, b"first.example:443"),
        ("preferred", ValueKind::Text, b"second.example:443"),
    ]);
    let source: PropagationSelector = serde_json::from_value(json!({
        "type":"transport_header","named":["absent", "preferred", "host"]
    }))
    .unwrap();
    assert_eq!(
        resolve_source(&source, &context).unwrap(),
        Some(LiteralValue::Str("first.example:443".into()))
    );
    let fallback: PropagationSelector = serde_json::from_value(json!({
        "type":"transport_header","named":["missing", "host"]
    }))
    .unwrap();
    assert_eq!(
        resolve_source(&fallback, &context).unwrap(),
        Some(LiteralValue::Str("last.example:80".into()))
    );
    let text: PropagationSelector =
        serde_json::from_value(json!({"type":"transport_header","named":["host"]})).unwrap();
    assert_eq!(
        resolve_source(&text, &context).unwrap(),
        Some(LiteralValue::Str("last.example:80".into()))
    );
}

/// Scenario: Stored header text enriches every signal and attribute domain in both encodings.
/// Guarantees: Insert/upsert/update project text verbatim, including ports and lists, without changing context snapshots.
#[test]
fn stored_header_text_otlp_otap_parity() {
    for text in ["first.example", "[::1]", "first.example:80, [::1]:443"] {
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            for native in [false, true] {
                let mut payload = fixture(signal, &[0]);
                if native {
                    payload = payload
                        .try_into_otap(&CodecService::new().unwrap())
                        .unwrap()
                        .into();
                }
                let mut context = header_context(&[("host", ValueKind::Text, text.as_bytes())]);
                let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
                    scope: ContextScope::Engine,
                    name: ContextEntryName::try_from("request").unwrap(),
                    definition: ContextEntryDefinition(vec![ContextEntryPart::TransportHeader {
                        name: ContextEntryName::try_from("host").unwrap(),
                        store_as: Some(ContextEntryName::try_from("value").unwrap()),
                    }]),
                }])
                .unwrap();
                context.materialize_context(&plan);
                let original_context = context.clone();
                run(
                    json!({
                        "apply_to":["resource","scope","signal"],
                        "actions":[
                            {"action":"insert","key":"insert_new","value":{"source":{"type":"transport_header","named":["host"]}}},
                            {"action":"insert","key":"insert_existing","value":{"source":{"type":"transport_header","named":["host"]}}},
                            {"action":"upsert","key":"upsert_existing","value":{"source":{"type":"named","named":["request:value"]}}},
                            {"action":"update","key":"update_existing","value":{"source":{"type":"transport_header","named":["host"]}}},
                            {"action":"update","key":"update_missing","value":{"source":{"type":"named","named":["request:value"]}}}
                        ]
                    }),
                    vec![Message::PData(OtapPdata::new(context, payload))],
                    move |mut output| {
                        assert_eq!(output.len(), 1);
                        let (context, payload) = output.remove(0).into_parts();
                        assert_eq!(context.clone_detached(), original_context);
                        let expected = AnyValue::new_string(text);
                        for attributes in domains(payload) {
                            for key in ["insert_new", "upsert_existing", "update_existing"] {
                                assert_eq!(
                                    value(&attributes, key),
                                    Some(&expected),
                                    "{signal:?} {native} {key}"
                                );
                            }
                            assert_eq!(
                                value(&attributes, "insert_existing"),
                                Some(&AnyValue::new_string("old"))
                            );
                            assert_eq!(value(&attributes, "update_missing"), None);
                        }
                    },
                );
            }
        }
    }
}

/// Scenario: Repeated literal actions select values in configuration order for every encoding and signal.
/// Guarantees: The existing last-value precedence and insert/upsert/update behavior remain unchanged.
#[test]
fn literal_action_order_is_preserved() {
    for reversed in [false, true] {
        let values = if reversed { [2, 1] } else { [1, 2] };
        let actions: Vec<_> = values
            .iter()
            .flat_map(|value| {
                [
                    json!({"action":"insert","key":"insert_new","value":value}),
                    json!({"action":"insert","key":"insert_existing","value":value}),
                    json!({"action":"upsert","key":"upsert_existing","value":value}),
                    json!({"action":"update","key":"update_existing","value":value}),
                    json!({"action":"update","key":"update_missing","value":value}),
                ]
            })
            .collect();
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            for native in [false, true] {
                let mut payload = fixture(signal, &[0]);
                if native {
                    payload = payload
                        .try_into_otap(&CodecService::new().unwrap())
                        .unwrap()
                        .into();
                }
                run(
                    json!({"apply_to":["resource","scope","signal"],"actions":actions}),
                    vec![Message::PData(OtapPdata::new(Context::default(), payload))],
                    move |mut output| {
                        assert_eq!(output.len(), 1);
                        for attributes in domains(output.remove(0).into_parts().1) {
                            for key in ["insert_new", "upsert_existing", "update_existing"] {
                                assert_eq!(
                                    value(&attributes, key),
                                    Some(&AnyValue::new_int(values[1])),
                                    "{signal:?} {native} {key}"
                                );
                            }
                            assert_eq!(
                                value(&attributes, "insert_existing"),
                                Some(&AnyValue::new_string("old"))
                            );
                            assert_eq!(value(&attributes, "update_missing"), None);
                        }
                    },
                );
            }
        }
    }
}

/// Scenario: Literal actions conflict across operation kinds or form invalid rename maps in either list order.
/// Guarantees: Previously rejected configurations remain rejected rather than gaining sequential semantics.
#[test]
fn literal_action_conflicts_are_unchanged() {
    for conflicting in [
        json!({"action":"delete","key":"x"}),
        json!({"action":"upsert","key":"x","value":2}),
        json!({"action":"update","key":"x","value":2}),
        json!({"action":"hash","key":"x"}),
        json!({"action":"rename","source_key":"x","destination_key":"y"}),
        json!({"action":"rename","source_key":"y","destination_key":"x"}),
    ] {
        let literal = json!({"action":"insert","key":"x","value":1});
        for actions in [json!([literal, conflicting]), json!([conflicting, literal])] {
            let config = json!({"actions":actions});
            assert!(validate_config(&config).is_err(), "{config}");
            assert!(AttributesProcessor::from_config(pipeline_context(), &config).is_err());
        }
    }
    for actions in [
        json!([
            {"action":"rename","source_key":"a","destination_key":"b"},
            {"action":"rename","source_key":"b","destination_key":"c"}
        ]),
        json!([
            {"action":"rename","source_key":"a","destination_key":"c"},
            {"action":"rename","source_key":"b","destination_key":"c"}
        ]),
    ] {
        assert!(validate_config(&json!({"actions":actions})).is_err());
    }
}

/// Scenario: Static and dynamic actions share keys within or across action kinds and signals.
/// Guarantees: Same-kind last-wins composition is preserved and intersecting cross-kind collisions fail early.
#[test]
fn static_dynamic_composition() {
    for other in [
        json!({"action":"delete","key":"x"}),
        json!({"action":"upsert","key":"x","value":1}),
        json!({"action":"rename","source_key":"old","destination_key":"x"}),
        json!({"action":"rename","source_key":"x","destination_key":"new"}),
        json!({"action":"hash","key":"x"}),
    ] {
        assert!(
            validate_config(&json!({"actions":[
                {"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["missing"]}}}, other
            ]}))
            .is_err()
        );
    }
    validate_config(&json!({"actions":[
        {"action":"insert","key":"x","value":{"source":{"type":"named","named":["x"]}},"signals":["logs"]},
        {"action":"upsert","key":"x","value":1,"signals":["metrics","traces"]}
    ]}))
    .unwrap();
    let config: Config = serde_json::from_value(json!({"actions":[
        {"action":"insert","key":"x","value":1},
        {"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["missing"]}}}
    ]}))
    .unwrap();
    let plans = compile_config(&config).unwrap();
    assert!(plan::is_noop(&plans[0].transform));
    assert!(plan::is_noop(
        &plans[0].resolve(&Context::default()).unwrap()
    ));
    let reversed: Config = serde_json::from_value(json!({"actions":[
        {"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["missing"]}}},
        {"action":"insert","key":"x","value":1}
    ]}))
    .unwrap();
    assert!(!plan::is_noop(
        &compile_config(&reversed).unwrap()[0].transform
    ));
}

/// Scenario: Numeric-looking context strings include values outside signed OTAP integer range.
/// Guarantees: Projection preserves the original text without interpreting, wrapping, or rejecting it.
#[test]
fn checked_context_attribute_range() {
    let source: PropagationSelector =
        serde_json::from_value(json!({"type":"named","named":["received:value"]})).unwrap();
    for integer in [0, i64::MAX as u64, i64::MAX as u64 + 1, u64::MAX] {
        let mut context = Context::default();
        store_context_text(&mut context, "received", integer.to_string());
        assert_eq!(
            resolve_source(&source, &context).unwrap(),
            Some(LiteralValue::Str(integer.to_string()))
        );
    }
    let context = Context::default();
    assert_eq!(resolve_source(&source, &context).unwrap(), None);
}

/// Scenario: Context text drives insert, upsert, and update for all signals and encodings.
/// Guarantees: All attribute domains retain strings, including empty, numeric, and boolean-looking values.
#[test]
fn typed_context_actions_otlp_otap_parity() {
    for scalar in [
        LiteralValue::Str(String::new()),
        LiteralValue::Str("tenant-a".into()),
        LiteralValue::Bool(false),
        LiteralValue::Bool(true),
        LiteralValue::Int(i64::MIN),
        LiteralValue::Int(i64::MAX),
        LiteralValue::Double(-1.25),
        LiteralValue::Double(0.0),
    ] {
        let text = match &scalar {
            LiteralValue::Str(value) => value.clone(),
            LiteralValue::Bool(value) => value.to_string(),
            LiteralValue::Int(value) => value.to_string(),
            LiteralValue::Double(value) => value.to_string(),
        };
        let expected = AnyValue::new_string(&text);
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            for native in [false, true] {
                let mut payload = fixture(signal, &[0]);
                if native {
                    payload = payload
                        .try_into_otap(&CodecService::new().unwrap())
                        .unwrap()
                        .into();
                }
                let mut context = Context::default();
                store_context_text(&mut context, "scalar", text.clone());
                let original_context = context.clone();
                let expected = expected.clone();
                run(
                    json!({
                        "apply_to":["resource","scope","signal"],
                        "actions":[
                            {"action":"insert","key":"insert_new","value":{"source":{"type":"named","named":["scalar:value"]}}},
                            {"action":"insert","key":"insert_existing","value":{"source":{"type":"named","named":["scalar"]}}},
                            {"action":"upsert","key":"upsert_existing","value":{"source":{"type":"named","named":["scalar"]}}},
                            {"action":"update","key":"update_existing","value":{"source":{"type":"named","named":["scalar"]}}},
                            {"action":"update","key":"update_missing","value":{"source":{"type":"named","named":["scalar"]}}}
                        ]
                    }),
                    vec![Message::PData(OtapPdata::new(context, payload))],
                    move |mut output| {
                        assert_eq!(output.len(), 1);
                        let (context, payload) = output.remove(0).into_parts();
                        assert_eq!(context.clone_detached(), original_context);
                        for attributes in domains(payload) {
                            for key in ["insert_new", "upsert_existing", "update_existing"] {
                                assert_eq!(
                                    value(&attributes, key),
                                    Some(&expected),
                                    "{signal:?} {native} {key}"
                                );
                            }
                            assert_eq!(
                                value(&attributes, "insert_existing"),
                                Some(&AnyValue::new_string("old"))
                            );
                            assert_eq!(value(&attributes, "update_missing"), None);
                        }
                    },
                );
            }
        }
    }
}

/// Scenario: Each signal arrives in OTLP and native OTAP form with all three attribute domains.
/// Guarantees: Dynamic insert/upsert/update, literals, domain selection, and per-action signal filters agree.
#[test]
fn dynamic_actions_otlp_otap_signal_domain_parity() {
    for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
        for native in [false, true] {
            for selected_domain in ["resource", "scope", "signal"] {
                let mut payload = fixture(signal, &[0]);
                if native {
                    payload = payload
                        .try_into_otap(&CodecService::new().unwrap())
                        .unwrap()
                        .into();
                }
                let mut context = header_context(&[("host", ValueKind::Text, b"new.example:443")]);
                store_context_text(&mut context, "received", "456".into());
                let config = json!({
                    "apply_to":[selected_domain],
                    "actions":[
                        {"action":"insert","key":"literal","value":true},
                        {"action":"insert","key":"insert_existing","value":{"source":{"type":"transport_header","named":["host"]}}},
                        {"action":"insert","key":"insert_new","value":{"source":{"type":"transport_header","named":["host"]}}},
                        {"action":"upsert","key":"upsert_existing","value":{"source":{"type":"transport_header","named":["host"]}}},
                        {"action":"update","key":"update_existing","value":{"source":{"type":"named","named":["received"]}}},
                        {"action":"update","key":"update_missing","value":{"source":{"type":"named","named":["received"]}}},
                        {"action":"insert","key":"missing","value":{"source":{"type":"transport_header","named":["uncaptured"]}}},
                        {"action":"insert","key":"logs_only","value":{"source":{"type":"named","named":["received"]}},"signals":["logs"]},
                        {"action":"insert","key":"traces_only","value":true,"signals":["traces"]}
                    ]
                });
                run(
                    config,
                    vec![Message::PData(OtapPdata::new(context, payload))],
                    move |mut output| {
                        assert_eq!(output.len(), 1);
                        for (index, attributes) in
                            domains(output.remove(0).payload()).into_iter().enumerate()
                        {
                            if ["resource", "scope", "signal"][index] != selected_domain {
                                assert_eq!(attributes, attrs());
                                continue;
                            }
                            assert_eq!(
                                value(&attributes, "literal"),
                                Some(&AnyValue::new_bool(true))
                            );
                            assert_eq!(
                                value(&attributes, "insert_existing"),
                                Some(&AnyValue::new_string("old"))
                            );
                            assert_eq!(
                                value(&attributes, "insert_new"),
                                Some(&AnyValue::new_string("new.example:443"))
                            );
                            assert_eq!(
                                value(&attributes, "upsert_existing"),
                                Some(&AnyValue::new_string("new.example:443"))
                            );
                            assert_eq!(
                                value(&attributes, "update_existing"),
                                Some(&AnyValue::new_string("456"))
                            );
                            assert_eq!(value(&attributes, "update_missing"), None);
                            assert_eq!(value(&attributes, "missing"), None);
                            assert_eq!(
                                value(&attributes, "logs_only").is_some(),
                                signal == SignalType::Logs
                            );
                            assert_eq!(
                                value(&attributes, "traces_only").is_some(),
                                signal == SignalType::Traces
                            );
                        }
                    },
                );
            }
        }
    }
}

/// Scenario: Attribute actions process logs with zero and nonzero observed times in either encoding.
/// Guarantees: Event and observed timestamps are unchanged and missing context entries are not generated.
#[test]
fn attribute_actions_preserve_log_timestamps_and_context() {
    for native in [false, true] {
        let mut payload = fixture(SignalType::Logs, &[0, 999]);
        if native {
            payload = payload
                .try_into_otap(&CodecService::new().unwrap())
                .unwrap()
                .into();
        }
        let context = header_context(&[("host", ValueKind::Text, b"example.com")]);
        let original_context = context.clone();
        run(
            json!({"apply_to":["resource"],"actions":[
                {"action":"insert","key":"server.address","value":{"source":{"type":"transport_header","named":["host"]}}},
                {"action":"insert","key":"missing","value":{"source":{"type":"named","named":["missing"]}}}
            ]}),
            vec![Message::PData(OtapPdata::new(context, payload))],
            move |mut output| {
                assert_eq!(output.len(), 1);
                let (context, payload) = output.remove(0).into_parts();
                assert_eq!(context.clone_detached(), original_context);
                let OtlpProtoBytes::ExportLogsRequest(bytes) =
                    payload.try_into_with_default().unwrap()
                else {
                    panic!("logs");
                };
                let decoded = ExportLogsServiceRequest::decode(bytes).unwrap();
                let resource = &decoded.resource_logs[0];
                assert_eq!(
                    value(
                        &resource.resource.as_ref().unwrap().attributes,
                        "server.address"
                    ),
                    Some(&AnyValue::new_string("example.com"))
                );
                let logs = &resource.scope_logs[0].log_records;
                assert_eq!(
                    logs.iter()
                        .map(|log| log.observed_time_unix_nano)
                        .collect::<Vec<_>>(),
                    vec![0, 999]
                );
                assert!(logs.iter().all(|log| log.time_unix_nano == 123));
            },
        );
    }
}

/// Scenario: Missing sources or signal filters leave malformed encoded pdata untouched.
/// Guarantees: No-op messages are forwarded without invoking payload conversion.
#[test]
fn noop_retains_unconverted_payload() {
    for config in [
        json!({}),
        json!({"actions":[{"action":"insert","key":"x","value":{"source":{"type":"named","named":["absent"]}}}]}),
        json!({"actions":[{"action":"insert","key":"x","value":{"source":{"type":"transport_header","named":["host"]}}}]}),
        json!({"actions":[{"action":"insert","key":"x","value":1,"signals":["metrics"]}]}),
    ] {
        let bytes = bytes::Bytes::from_static(b"not valid protobuf");
        let payload = OtlpProtoBytes::ExportLogsRequest(bytes.clone()).into();
        run(
            config,
            vec![Message::PData(OtapPdata::new(Context::default(), payload))],
            move |mut output| {
                assert_eq!(output.len(), 1);
                let OtlpProtoBytes::ExportLogsRequest(actual) =
                    output.remove(0).payload().try_into_with_default().unwrap()
                else {
                    panic!("logs")
                };
                assert_eq!(actual, bytes);
            },
        );
    }
}

/// Scenario: A valid live configuration is followed by an invalid update.
/// Guarantees: Updates are atomic and an invalid replacement retains the last validated plan.
#[test]
fn configuration_update_preserves_last_valid_plan() {
    run(
        json!({}),
        vec![
            Message::Control(NodeControlMsg::Config {
                config: json!({"actions":[
                    {"action":"insert","key":"updated","value":true}
                ]}),
            }),
            Message::Control(NodeControlMsg::Config {
                config: json!({"actions":[
                    {"action":"insert","key":"invalid","value":{"source":{"type":"transport_header","named":[]}}}
                ]}),
            }),
            Message::PData(OtapPdata::new(
                Context::default(),
                fixture(SignalType::Logs, &[0]),
            )),
        ],
        |mut output| {
            let attributes = domains(output.remove(0).payload());
            assert_eq!(
                value(&attributes[2], "updated"),
                Some(&AnyValue::new_bool(true))
            );
            assert_eq!(value(&attributes[2], "invalid"), None);
        },
    );
}

/// Scenario: A live update attempts to configure authority normalization in the attribute processor.
/// Guarantees: Removed transform options are rejected and the installed raw-text projection remains active.
#[test]
fn removed_source_transform_updates_are_rejected() {
    let config = |transform: &str| {
        json!({"actions":[
            {"action":"insert","key":"selected","value":{
                "source":{"type":"transport_header","named":["host"]},"transform":transform
            }}
        ]})
    };
    let pdata = || {
        Message::PData(OtapPdata::new(
            header_context(&[("host", ValueKind::Text, b"first.example, last.example")]),
            fixture(SignalType::Logs, &[0]),
        ))
    };
    run(
        json!({"actions":[{"action":"insert","key":"selected","value":{
            "source":{"type":"transport_header","named":["host"]}
        }}]}),
        vec![
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("authority_host"),
            }),
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("int"),
            }),
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("string"),
            }),
            pdata(),
        ],
        |output| {
            assert_eq!(output.len(), 4);
            for (payload, expected) in output.into_iter().zip([
                "first.example, last.example",
                "first.example, last.example",
                "first.example, last.example",
                "first.example, last.example",
            ]) {
                assert_eq!(
                    value(&domains(payload.payload())[2], "selected"),
                    Some(&AnyValue::new_string(expected))
                );
            }
        },
    );
}

/// Scenario: Context lookups encounter absent entries, absent members, ambiguous entries, and multi-valued headers.
/// Guarantees: Missing values remain no-ops while nonscalar and ambiguous values produce explicit errors.
#[test]
fn context_lookup_shape_validation() {
    let source = |name: &str| {
        serde_json::from_value::<PropagationSelector>(json!({
            "type":"named","named":[name]
        }))
        .unwrap()
    };
    let mut context = Context::default();
    assert_eq!(
        resolve_source(&source("request:value"), &context).unwrap(),
        None
    );
    store_context_text(&mut context, "request", "first.example,last.example".into());
    assert_eq!(resolve_source(&source("absent"), &context).unwrap(), None);
    assert_eq!(
        resolve_source(&source("absent:value"), &context).unwrap(),
        None
    );
    assert_eq!(
        resolve_source(&source("request:absent"), &context).unwrap(),
        None
    );
    assert_eq!(
        resolve_source(&source("request"), &context).unwrap(),
        Some(LiteralValue::Str("first.example,last.example".into()))
    );
    let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
        scope: ContextScope::Engine,
        name: ContextEntryName::try_from("request").unwrap(),
        definition: ContextEntryDefinition(vec![
            ContextEntryPart::Constant {
                name: ContextEntryName::try_from("first").unwrap(),
                value: "first.example".into(),
            },
            ContextEntryPart::Constant {
                name: ContextEntryName::try_from("last").unwrap(),
                value: "last.example".into(),
            },
        ]),
    }])
    .unwrap();
    let mut context = Context::default();
    context.materialize_context(&plan);
    assert!(
        resolve_source(&source("request"), &context)
            .unwrap_err()
            .to_string()
            .contains("entry:member")
    );
    assert_eq!(
        resolve_source(&source("request:last"), &context).unwrap(),
        Some(LiteralValue::Str("last.example".into()))
    );
    let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
        scope: ContextScope::Engine,
        name: ContextEntryName::try_from("request").unwrap(),
        definition: ContextEntryDefinition(vec![ContextEntryPart::TransportHeader {
            name: ContextEntryName::try_from("host").unwrap(),
            store_as: None,
        }]),
    }])
    .unwrap();
    let mut context = header_context(&[
        ("host", ValueKind::Text, b"first.example"),
        ("host", ValueKind::Text, b"last.example"),
    ]);
    context.materialize_context(&plan);
    assert!(
        resolve_source(&source("request:host"), &context)
            .unwrap_err()
            .to_string()
            .contains("scalar")
    );
    for (kind, bytes, reason) in [
        (ValueKind::Binary, b"binary".as_slice(), "requires text"),
        (ValueKind::Text, b"\xff".as_slice(), "invalid UTF-8"),
    ] {
        let mut context = header_context(&[("host", kind, bytes)]);
        context.materialize_context(&plan);
        assert!(
            resolve_source(&source("request:host"), &context)
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
}

/// Scenario: Dynamic request metadata has nonscalar context in an otherwise valid message.
/// Guarantees: The processor sends a permanent Nack with the original payload and request context.
#[test]
fn context_shape_error_permanently_nacks_original_message() {
    let rt = TestRuntime::new();
    let mut config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
    config.config = json!({"actions":[{"action":"insert","key":"received","value":{"source":{"type":"named","named":["received"]}}}]});
    let processor = create_attributes_processor(
        declared_pipeline_context(&config.config),
        test_node("range-nack"),
        Arc::new(config),
        rt.config(),
    )
    .unwrap();
    let mut context = Context::default();
    let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
        scope: ContextScope::Engine,
        name: ContextEntryName::try_from("received").unwrap(),
        definition: ContextEntryDefinition(vec![
            ContextEntryPart::Constant {
                name: ContextEntryName::try_from("first").unwrap(),
                value: "one".into(),
            },
            ContextEntryPart::Constant {
                name: ContextEntryName::try_from("second").unwrap(),
                value: "two".into(),
            },
        ]),
    }])
    .unwrap();
    context.materialize_context(&plan);
    let original_context = context.clone();
    let input = OtapPdata::new(context, fixture(SignalType::Logs, &[0])).test_subscribe_to(
        Interests::NACKS | Interests::RETURN_DATA,
        CallData::new(),
        11,
    );
    let original = input.payload_ref().encoded_bytes().unwrap().clone();
    rt.set_processor(processor)
        .run_test(|mut ctx| async move {
            let (sender, mut receiver) = pipeline_completion_msg_channel(1);
            ctx.set_pipeline_completion_sender(sender);
            ctx.process(Message::PData(input)).await.unwrap();
            assert!(ctx.drain_pdata().await.is_empty());
            let PipelineCompletionMsg::DeliverNack { nack } = receiver.try_recv().unwrap() else {
                panic!("expected Nack");
            };
            assert!(nack.permanent);
            assert!(nack.reason.contains("entry:member"));
            assert_eq!(
                nack.refused.payload_ref().encoded_bytes().unwrap(),
                &original
            );
            let (context, _) = nack.refused.into_parts();
            assert_eq!(context.clone_detached(), original_context);
        })
        .validate(|_| async {});
}

/// Scenario: encoded input resolves successfully but a fault-injected transform fails after decoding.
/// Guarantees: RETURN_DATA recovers the original encoded payload and context, with one failure duration.
#[test]
fn encoded_transform_failure_returns_original_and_records_one_failure() {
    use otel_arrow_dfe_telemetry::metrics::MetricValue;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    let rt = TestRuntime::new();
    let mut config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
    config.config = json!({"actions":[{"action":"insert","key":"added","value":"literal"}]});
    let mut processor =
        AttributesProcessor::from_config(pipeline_context(), &config.config).unwrap();
    // Inject a post-compilation collision to exercise mutation failure independently of decoding.
    processor.plans[0].transform.delete =
        Some(DeleteTransform::new(BTreeSet::from(["added".to_string()])));
    let payload = fixture(SignalType::Logs, &[0]);
    let mut decoded = payload
        .clone()
        .try_into_otap(&CodecService::new().unwrap())
        .unwrap();
    let expected_error = processor
        .apply_transform_with_stats(
            &mut decoded,
            SignalType::Logs,
            &AttributesTransform::default(),
        )
        .unwrap_err()
        .to_string();
    let context = header_context(&[("host", ValueKind::Text, b"example.test")]);
    let original_context = context.clone();
    let input = OtapPdata::new(context, payload).test_subscribe_to(
        Interests::NACKS | Interests::RETURN_DATA,
        CallData::new(),
        11,
    );
    let original = input.payload_ref().encoded_bytes().unwrap().clone();
    let processor = ProcessorWrapper::local(
        processor,
        test_node("transform-failure"),
        Arc::new(config),
        rt.config(),
    );
    rt.set_processor(processor)
        .run_test_with_interests(Interests::NODE_LOCAL_DURATION, |mut ctx| async move {
            let (sender, mut receiver) = pipeline_completion_msg_channel(1);
            ctx.set_pipeline_completion_sender(sender);
            ctx.process(Message::PData(input)).await.unwrap();
            assert!(ctx.drain_pdata().await.is_empty());
            let PipelineCompletionMsg::DeliverNack { nack } = receiver.try_recv().unwrap() else {
                panic!("expected Nack");
            };
            assert!(nack.permanent);
            assert_eq!(nack.reason, expected_error);
            assert_eq!(
                nack.refused.payload_ref().encoded_bytes().unwrap(),
                &original
            );
            let (context, _) = nack.refused.into_parts();
            assert_eq!(context.clone_detached(), original_context);
            let (receiver, metrics_reporter) = MetricsReporter::create_new_and_receiver(32);
            ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter,
            }))
            .await
            .unwrap();
            let snapshots: Vec<_> = receiver
                .try_iter()
                .filter(|snapshot| snapshot.descriptor().name == "processor.compute")
                .collect();
            assert_eq!(snapshots.len(), 1);
            assert_eq!(
                snapshots[0].measurement_attribute_value("outcome"),
                Some("failure")
            );
            let MetricValue::Distribution(duration) = &snapshots[0].get_metrics()[0] else {
                panic!("expected duration distribution");
            };
            assert_eq!(duration.count(), 1);
        })
        .validate(|_| async {});
}

/// Scenario: literal, dynamic, and no-op messages run with duration telemetry enabled.
/// Guarantees: successful mutations record one sample, while no-op messages record none and stay encoded.
#[test]
fn compute_duration_records_once_or_skips_noop() {
    use otel_arrow_dfe_telemetry::metrics::MetricValue;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    for (config_value, expected_count) in [
        (json!({"actions":[]}), 0),
        (
            json!({"actions":[{"action":"insert","key":"added","value":{"source":{"type":"transport_header","named":["missing"]}}}]}),
            0,
        ),
        (
            json!({"actions":[{"action":"insert","key":"added","value":"literal"}]}),
            1,
        ),
        (
            json!({"actions":[{"action":"insert","key":"added","value":{"source":{"type":"transport_header","named":["host"]}}}]}),
            1,
        ),
    ] {
        let rt = TestRuntime::new();
        let mut config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
        config.config = config_value;
        let processor = create_attributes_processor(
            declared_pipeline_context(&config.config),
            test_node("duration"),
            Arc::new(config),
            rt.config(),
        )
        .unwrap();
        let input = OtapPdata::new(
            header_context(&[("host", ValueKind::Text, b"example.test")]),
            fixture(SignalType::Logs, &[0]),
        );
        rt.set_processor(processor)
            .run_test_with_interests(Interests::NODE_LOCAL_DURATION, move |mut ctx| async move {
                ctx.process(Message::PData(input)).await.unwrap();
                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                assert_eq!(
                    output[0].payload_ref().encoded_bytes().is_some(),
                    expected_count == 0
                );
                let (receiver, metrics_reporter) = MetricsReporter::create_new_and_receiver(32);
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .unwrap();
                let snapshots: Vec<_> = receiver
                    .try_iter()
                    .filter(|snapshot| snapshot.descriptor().name == "processor.compute")
                    .collect();
                assert_eq!(snapshots.len(), expected_count);
                if expected_count == 1 {
                    assert_eq!(
                        snapshots[0].measurement_attribute_value("outcome"),
                        Some("success")
                    );
                    let MetricValue::Distribution(duration) = &snapshots[0].get_metrics()[0] else {
                        panic!("expected duration distribution");
                    };
                    assert_eq!(duration.count(), 1);
                }
            })
            .validate(|_| async {});
    }
}

/// Scenario: a named selector references conditional and qualified composites in a pipeline policy.
/// Guarantees: declarations materialize only selected composites, preserve fallback order, and reject unknown references.
#[test]
fn named_selector_declares_and_materializes_required_composites() {
    let source = json!({"source":{"type":"named","named":["absent","request:number"]}});
    let config = json!({"actions":[{"action":"insert","key":"sequence","value":source}]});
    let entries = json!({
        "absent":[{"type":"transport_header","name":"missing"}],
        "request":[
            {"type":"constant","name":"number","value":"42"},
            {"type":"constant","name":"other","value":"unused"}
        ],
        "unselected":[{"type":"constant","name":"value","value":"not materialized"}]
    });
    let prepared = prepare_context(&config, entries.clone()).unwrap();
    let pipeline = pipeline_context().pipeline_key();
    let plan = prepared
        .bindings
        .context_materialization_plan(&pipeline)
        .unwrap();
    let mut context = Context::default();
    context.materialize_context(plan);
    let materialized = context.materialized_context().unwrap();
    assert!(
        materialized
            .entry(&ContextEntryName::try_from("request").unwrap())
            .is_some()
    );
    assert!(
        materialized
            .entry(&ContextEntryName::try_from("absent").unwrap())
            .is_none()
    );
    assert!(
        materialized
            .entry(&ContextEntryName::try_from("unselected").unwrap())
            .is_none()
    );
    let selected: PropagationSelector = serde_json::from_value(source["source"].clone()).unwrap();
    assert_eq!(
        resolve_source(&selected, &context).unwrap(),
        Some(LiteralValue::Str("42".into()))
    );
    run(
        config,
        vec![Message::PData(OtapPdata::new(
            context,
            fixture(SignalType::Logs, &[0]),
        ))],
        |mut output| {
            assert_eq!(
                value(&domains(output.remove(0).payload())[2], "sequence"),
                Some(&AnyValue::new_string("42"))
            );
        },
    );
    for reference in ["unknown", "request:unknown"] {
        let config = json!({"actions":[{"action":"insert","key":"x","value":{"source":{"type":"named","named":[reference]}}}]});
        assert!(prepare_context(&config, entries.clone()).is_err());
    }
}

/// Scenario: primitive identity selectors read verified claims captured under stored aliases.
/// Guarantees: aliases are case-sensitive, missing names fall through, scalar text is unchanged, and lists are rejected.
#[test]
fn authorized_identity_selector_reads_only_captured_scalar_claims() {
    use otel_arrow_dfe_engine::capability::auth::AuthorizedIdentity;
    let identity = AuthorizedIdentity::new()
        .with_subject("42")
        .with_claim_str("uncaptured", "secret")
        .with_claim_values("groups", ["one"]);
    let policy = serde_json::from_value(json!([
        {"claim":"sub","store_as":"UserId"},
        {"claim":"groups","store_as":"groups"}
    ]))
    .unwrap();
    let input = otel_arrow_dfe_otap::testing::with_authorized_identity(
        OtapPdata::new_default(fixture(SignalType::Logs, &[0])),
        &policy,
        &identity,
    );
    let (context, payload) = input.into_parts();
    for (names, expected) in [
        (vec!["userid", "uncaptured"], None),
        (
            vec!["missing", "UserId"],
            Some(LiteralValue::Str("42".into())),
        ),
    ] {
        let source: PropagationSelector = serde_json::from_value(json!({
            "type":"authorized_identity","named":names
        }))
        .unwrap();
        validate_source(&source).unwrap();
        assert_eq!(resolve_source(&source, &context).unwrap(), expected);
    }
    let groups: PropagationSelector = serde_json::from_value(json!({
        "type":"authorized_identity","named":["groups","UserId"]
    }))
    .unwrap();
    assert!(
        resolve_source(&groups, &context)
            .unwrap_err()
            .to_string()
            .contains("scalar")
    );
    let config = json!({"actions":[{"action":"insert","key":"user.id","value":{
        "source":{"type":"authorized_identity","named":["missing","UserId"]}
    }}]});
    run(
        config,
        vec![Message::PData(OtapPdata::new(context, payload))],
        |mut output| {
            assert_eq!(
                value(&domains(output.remove(0).payload())[2], "user.id"),
                Some(&AnyValue::new_string("42"))
            );
        },
    );
}

/// Scenario: every source domain declares its reads and a live update requests a new primitive.
/// Guarantees: declarations use value-only domain-scoped targets and an unbound update retains the installed plan.
#[test]
fn selector_declarations_and_live_update_binding_validation() {
    use otel_arrow_dfe_engine::context_declaration::{
        ContextDomain, ContextEntrySelectorForm, ContextEntryTarget,
    };
    for (kind, expected) in [
        (
            "transport_header",
            ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name: ContextEntryName::try_from("x").unwrap(),
            },
        ),
        (
            "authorized_identity",
            ContextEntryTarget::Primitive {
                domain: ContextDomain::AuthorizedIdentity,
                name: ContextEntryName::try_from("x").unwrap(),
            },
        ),
        (
            "named",
            ContextEntryTarget::Composite {
                name: ContextEntryName::try_from("x").unwrap(),
            },
        ),
    ] {
        let source: PropagationSelector =
            serde_json::from_value(json!({"type":kind,"named":["x"]})).unwrap();
        let entries: Vec<_> = context_entries(&source).collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].target, expected);
        assert_eq!(entries[0].form, ContextEntrySelectorForm::Value);
    }
    let config = |name| {
        json!({"actions":[{"action":"insert","key":"selected","value":{
            "source":{"type":"transport_header","named":[name]}
        }}]})
    };
    assert!(AttributesProcessor::from_config(pipeline_context(), &config("host")).is_err());
    run(
        config("host"),
        vec![
            Message::Control(NodeControlMsg::Config {
                config: config("other"),
            }),
            Message::PData(OtapPdata::new(
                header_context(&[
                    ("host", ValueKind::Text, b"original"),
                    ("other", ValueKind::Text, b"unbound"),
                ]),
                fixture(SignalType::Logs, &[0]),
            )),
        ],
        |mut output| {
            assert_eq!(
                value(&domains(output.remove(0).payload())[2], "selected"),
                Some(&AnyValue::new_string("original"))
            );
        },
    );
}

/// Scenario: each value operation has a context source overridden on all or only some signals.
/// Guarantees: fully discarded sources need no definitions or materialization; sources still used by any signal remain required.
#[test]
fn context_declarations_follow_effective_value_actions() {
    for (operation, key) in [
        ("insert", "added"),
        ("upsert", "upsert_existing"),
        ("update", "update_existing"),
    ] {
        let source = json!({"source":{"type":"named","named":["request"]}});
        let config = json!({"actions":[
            {"action":operation,"key":key,"value":source},
            {"action":operation,"key":key,"value":"literal"}
        ]});
        let parsed: Config = serde_json::from_value(config.clone()).unwrap();
        assert!(parsed.context_declarations().is_empty());
        for entries in [
            json!({}),
            json!({"request":[{"type":"constant","name":"value","value":"unused"}]}),
        ] {
            let prepared = prepare_context(&config, entries).unwrap();
            assert!(
                prepared
                    .bindings
                    .context_materialization_plan(&pipeline_context().pipeline_key())
                    .is_none()
            );
        }
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            run(
                config.clone(),
                vec![Message::PData(OtapPdata::new_default(fixture(
                    signal,
                    &[0],
                )))],
                move |mut output| {
                    assert_eq!(
                        value(&domains(output.remove(0).payload())[2], key),
                        Some(&AnyValue::new_string("literal"))
                    );
                },
            );
        }

        let partial = json!({"actions":[
            {"action":operation,"key":key,"value":source},
            {"action":operation,"key":key,"value":"literal","signals":["logs"]}
        ]});
        assert!(prepare_context(&partial, json!({})).is_err());
        let prepared = prepare_context(
            &partial,
            json!({"request":[{"type":"constant","name":"value","value":"context"}]}),
        )
        .unwrap();
        let pipeline = pipeline_context().pipeline_key();
        let plan = prepared
            .bindings
            .context_materialization_plan(&pipeline)
            .unwrap();
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            let mut context = Context::default();
            context.materialize_context(plan);
            run(
                partial.clone(),
                vec![Message::PData(OtapPdata::new(
                    context,
                    fixture(signal, &[0]),
                ))],
                move |mut output| {
                    let expected = if matches!(signal, SignalType::Logs) {
                        "literal"
                    } else {
                        "context"
                    };
                    assert_eq!(
                        value(&domains(output.remove(0).payload())[2], key),
                        Some(&AnyValue::new_string(expected))
                    );
                },
            );
        }
    }
}

/// Scenario: a live update reorders, repeats, and regroups already declared header candidates.
/// Guarantees: equivalent read requirements retain valid bindings while the new per-action fallback order takes effect.
#[test]
fn live_update_preserves_bindings_when_candidate_order_and_grouping_change() {
    let initial = json!({"actions":[
        {"action":"insert","key":"first","value":{
            "source":{"type":"transport_header","named":["forwarded","host"]}}},
        {"action":"insert","key":"second","value":{
            "source":{"type":"transport_header","named":["host"]}}}
    ]});
    let updated = json!({"actions":[
        {"action":"insert","key":"first","value":{
            "source":{"type":"transport_header","named":["host","forwarded","host"]}}},
        {"action":"insert","key":"second","value":{
            "source":{"type":"transport_header","named":["forwarded","host"]}}}
    ]});
    let parsed_initial: Config = serde_json::from_value(initial.clone()).unwrap();
    let parsed_updated: Config = serde_json::from_value(updated.clone()).unwrap();
    assert_eq!(
        parsed_initial.context_declarations(),
        parsed_updated.context_declarations()
    );
    for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
        run(
            initial.clone(),
            vec![
                Message::Control(NodeControlMsg::Config {
                    config: updated.clone(),
                }),
                Message::PData(OtapPdata::new(
                    header_context(&[
                        ("forwarded", ValueKind::Text, b"proxy.example"),
                        ("host", ValueKind::Text, b"direct.example"),
                    ]),
                    fixture(signal, &[0]),
                )),
            ],
            |mut output| {
                let attrs = domains(output.remove(0).payload());
                assert_eq!(
                    value(&attrs[2], "first"),
                    Some(&AnyValue::new_string("direct.example"))
                );
                assert_eq!(
                    value(&attrs[2], "second"),
                    Some(&AnyValue::new_string("proxy.example"))
                );
            },
        );
    }
}
