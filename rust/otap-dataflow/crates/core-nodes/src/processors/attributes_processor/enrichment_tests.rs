// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_config::ContextEntryName;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
};
use otel_arrow_dfe_config::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
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
        pipeline_context(),
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
        json!({"action":"upsert","key":"x","value":{"from_headers":[]}}),
        json!({"action":"insert","key":"x","value":{"from_headers":[""]}}),
        json!({"action":"insert","key":"x","value":{"from_context":" "}}),
        json!({"action":"insert","key":"x","value":{"from_context":"x","transform":"unknown"}}),
        json!({"action":"insert","key":"x","value":{"from_context":"x","value_type":"int"}}),
        json!({"action":"insert","key":"x","value":{"from_context":"x","transform":null}}),
        json!({"action":"insert","key":"x","value":{"from_headers":["host"],"transform":"authority_host","selection":"middle"}}),
        json!({"action":"insert","key":"x","value":{"from_headers":["host"],"transform":"authority_host","selection":null}}),
        json!({"action":"insert","key":"x","value":{"from_headers":["host"],"selection":"first"}}),
        json!({"action":"insert","key":"x","value":{"from_context":"x","transform":"int","selection":"last"}}),
        json!({"action":"insert","key":"x","value":{"string":"host","selection":"last"}}),
        json!({"action":"insert","key":"x","value":{"from_headers":["host"],"transform":"unknown"}}),
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
                assert_eq!(parsed.literal(), Some(expected.clone()));
                assert_eq!(
                    parsed.resolve(&Context::default()).unwrap(),
                    Some(expected.clone())
                );
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
        json!({"string":"prod","from_headers":["host"]}),
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
/// Guarantees: Stored-name priority wins and binary, empty, malformed, and invalid UTF-8 values fall through.
#[test]
fn ordered_headers_skip_unusable_candidates() {
    let context = header_context(&[
        ("host", ValueKind::Text, b"last.example:80"),
        ("preferred", ValueKind::Text, b"not an authority"),
        ("preferred", ValueKind::Binary, b"binary.example"),
        ("preferred", ValueKind::Text, &[0xff]),
        ("preferred", ValueKind::Text, b""),
        ("preferred", ValueKind::Text, b"first.example:443"),
        ("preferred", ValueKind::Text, b"second.example:443"),
    ]);
    let source: ValueSource = serde_json::from_value(json!({
        "from_headers":["absent", "preferred", "host"], "transform":"authority_host"
    }))
    .unwrap();
    assert_eq!(
        source.resolve(&context).unwrap(),
        Some(LiteralValue::Str("first.example".into()))
    );
    let fallback: ValueSource = serde_json::from_value(json!({
        "from_headers":["missing", "host"], "transform":"authority_host"
    }))
    .unwrap();
    assert_eq!(
        fallback.resolve(&context).unwrap(),
        Some(LiteralValue::Str("last.example".into()))
    );
    let text: ValueSource = serde_json::from_value(json!({"from_headers":["host"]})).unwrap();
    assert_eq!(
        text.resolve(&context).unwrap(),
        Some(LiteralValue::Str("last.example:80".into()))
    );
}

/// Scenario: Header and context text use every shared transform with valid and invalid inputs.
/// Guarantees: Both sources produce identical scalar types; invalid headers fall through and invalid context errors.
#[test]
fn shared_source_transforms() {
    for (transform, text, invalid, expected) in [
        ("string", "123", "", LiteralValue::Str("123".into())),
        ("bool", "false", "not-bool", LiteralValue::Bool(false)),
        ("int", "-42", "18446744073709551615", LiteralValue::Int(-42)),
        ("double", "1.25", "not-double", LiteralValue::Double(1.25)),
        (
            "authority_host",
            "[::1]:443",
            "not a host",
            LiteralValue::Str("[::1]".into()),
        ),
    ] {
        let header: ValueSource = serde_json::from_value(json!({
            "from_headers":["preferred","fallback"],"transform":transform
        }))
        .unwrap();
        let context_source: ValueSource = serde_json::from_value(json!({
            "from_context":"request:value","transform":transform
        }))
        .unwrap();
        header.validate().unwrap();
        context_source.validate().unwrap();
        let headers = header_context(&[
            ("preferred", ValueKind::Text, invalid.as_bytes()),
            ("fallback", ValueKind::Text, text.as_bytes()),
        ]);
        assert_eq!(header.resolve(&headers).unwrap(), Some(expected.clone()));
        let mut context = Context::default();
        store_context_text(&mut context, "request", text.into());
        assert_eq!(context_source.resolve(&context).unwrap(), Some(expected));
        store_context_text(&mut context, "request", invalid.into());
        if transform == "string" {
            assert_eq!(
                context_source.resolve(&context).unwrap(),
                Some(LiteralValue::Str(String::new()))
            );
        } else {
            assert!(context_source.resolve(&context).is_err());
        }
    }
}

/// Scenario: Host projection encounters DNS, IPv4, IPv6, forwarding lists, and malformed authorities.
/// Guarantees: The first authority loses only its port, while invalid authorities remain unusable.
#[test]
fn authority_host_parsing() {
    for (input, expected) in [
        ("example.com:443", Some("example.com")),
        ("127.0.0.1:8080", Some("127.0.0.1")),
        ("[2001:db8::1]:443", Some("[2001:db8::1]")),
        ("[::1]", Some("[::1]")),
        (
            " first.example:80, second.example:90 ",
            Some("first.example"),
        ),
        ("https://example.com", None),
        ("not a host", None),
        ("", None),
        (",example.com", None),
        ("user@example.com", None),
        ("example.com:99999", None),
        ("example.com:abc", None),
    ] {
        let context = header_context(&[("host", ValueKind::Text, input.as_bytes())]);
        let source: ValueSource = serde_json::from_value(json!({
            "from_headers":["host"],"transform":"authority_host"
        }))
        .unwrap();
        assert_eq!(
            source.resolve(&context).unwrap(),
            expected.map(|host| LiteralValue::Str(host.into())),
            "{input}"
        );
    }
}

/// Scenario: Authority lists select the configured endpoint for headers and context text.
/// Guarantees: First remains the default, whitespace and IPv6 work, and invalid selected elements never select another list element.
#[test]
fn authority_host_selection() {
    for selection in [None, Some("first"), Some("last")] {
        let last = selection == Some("last");
        for (text, first, final_host) in [
            (
                " first.example:80, middle.example:90, [::1]:443 ",
                Some("first.example"),
                Some("[::1]"),
            ),
            ("127.0.0.1:80", Some("127.0.0.1"), Some("127.0.0.1")),
            (
                "[2001:db8::1]",
                Some("[2001:db8::1]"),
                Some("[2001:db8::1]"),
            ),
            ("", None, None),
            ("  ", None, None),
            (",last.example", None, Some("last.example")),
            ("first.example,", Some("first.example"), None),
            ("bad host,last.example", None, Some("last.example")),
            ("first.example,bad host", Some("first.example"), None),
            ("user@first.example,last.example:99999", None, None),
            ("first.example:abc,https://last.example", None, None),
        ] {
            let expected =
                if last { final_host } else { first }.map(|host| LiteralValue::Str(host.into()));
            let mut header_config = json!({"from_headers":["host"],"transform":"authority_host"});
            let mut context_config =
                json!({"from_context":"request:value","transform":"authority_host"});
            if let Some(selection) = selection {
                header_config["selection"] = json!(selection);
                context_config["selection"] = json!(selection);
            }
            for config in [header_config, context_config] {
                let source: ValueSource = serde_json::from_value(config.clone()).unwrap();
                source.validate().unwrap();
                assert_eq!(serde_json::to_value(&source).unwrap(), config);
                let mut context = header_context(&[("host", ValueKind::Text, text.as_bytes())]);
                store_context_text(&mut context, "request", text.into());
                let actual = source.resolve(&context);
                if source.from_context.is_some() && expected.is_none() {
                    assert!(actual.is_err(), "{config} {text}");
                } else {
                    assert_eq!(actual.unwrap(), expected, "{config} {text}");
                }
            }
        }
    }
}

/// Scenario: Last-value selection meets invalid repeated headers and competing source names.
/// Guarantees: Selection only affects comma-list endpoints; repeated-value and source-name priority remain unchanged.
#[test]
fn authority_selection_preserves_header_fallback() {
    let context = header_context(&[
        ("fallback", ValueKind::Text, b"fallback.example"),
        ("preferred", ValueKind::Text, b"valid.example,"),
        (
            "preferred",
            ValueKind::Text,
            b"ignored.example, chosen.example:443",
        ),
        (
            "preferred",
            ValueKind::Text,
            b"ignored.example, later.example:80",
        ),
    ]);
    let source: ValueSource = serde_json::from_value(json!({
        "from_headers":["missing","preferred","fallback"],
        "transform":"authority_host","selection":"last"
    }))
    .unwrap();
    assert_eq!(
        source.resolve(&context).unwrap(),
        Some(LiteralValue::Str("chosen.example".into()))
    );
    let context = header_context(&[
        ("preferred", ValueKind::Text, b"valid.example,"),
        ("fallback", ValueKind::Text, b"fallback.example:80"),
    ]);
    assert_eq!(
        source.resolve(&context).unwrap(),
        Some(LiteralValue::Str("fallback.example".into()))
    );
}

/// Scenario: First and last authority selection enrich every signal and attribute domain in both encodings.
/// Guarantees: Insert/upsert/update use the selected host without changing existing action behavior or context.
#[test]
fn authority_selection_otlp_otap_parity() {
    for selection in ["first", "last"] {
        for signal in [SignalType::Logs, SignalType::Metrics, SignalType::Traces] {
            for native in [false, true] {
                let mut payload = fixture(signal, &[0]);
                if native {
                    payload = payload
                        .try_into_otap(&CodecService::new().unwrap())
                        .unwrap()
                        .into();
                }
                let mut context =
                    header_context(&[("host", ValueKind::Text, b"first.example:80, [::1]:443")]);
                store_context_text(
                    &mut context,
                    "request",
                    "first.example:80, [::1]:443".into(),
                );
                let original_context = context.clone();
                run(
                    json!({
                        "apply_to":["resource","scope","signal"],
                        "actions":[
                            {"action":"insert","key":"insert_new","value":{"from_headers":["host"],"transform":"authority_host","selection":selection}},
                            {"action":"insert","key":"insert_existing","value":{"from_headers":["host"],"transform":"authority_host","selection":selection}},
                            {"action":"upsert","key":"upsert_existing","value":{"from_context":"request:value","transform":"authority_host","selection":selection}},
                            {"action":"update","key":"update_existing","value":{"from_headers":["host"],"transform":"authority_host","selection":selection}},
                            {"action":"update","key":"update_missing","value":{"from_context":"request:value","transform":"authority_host","selection":selection}}
                        ]
                    }),
                    vec![Message::PData(OtapPdata::new(context, payload))],
                    move |mut output| {
                        assert_eq!(output.len(), 1);
                        let (context, payload) = output.remove(0).into_parts();
                        assert_eq!(context.clone_detached(), original_context);
                        let expected = AnyValue::new_string(if selection == "first" {
                            "first.example"
                        } else {
                            "[::1]"
                        });
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
                {"action":"insert","key":"x","value":{"from_headers":["missing"]}}, other
            ]}))
            .is_err()
        );
    }
    validate_config(&json!({"actions":[
        {"action":"insert","key":"x","value":{"from_context":"x"},"signals":["logs"]},
        {"action":"upsert","key":"x","value":1,"signals":["metrics","traces"]}
    ]}))
    .unwrap();
    let config: Config = serde_json::from_value(json!({"actions":[
        {"action":"insert","key":"x","value":1},
        {"action":"insert","key":"x","value":{"from_headers":["missing"]}}
    ]}))
    .unwrap();
    let plans = compile_config(&config).unwrap();
    assert!(plan::is_noop(&plans[0].transform));
    assert!(plan::is_noop(
        &plans[0].resolve(&Context::default()).unwrap()
    ));
    let reversed: Config = serde_json::from_value(json!({"actions":[
        {"action":"insert","key":"x","value":{"from_headers":["missing"]}},
        {"action":"insert","key":"x","value":1}
    ]}))
    .unwrap();
    assert!(!plan::is_noop(
        &compile_config(&reversed).unwrap()[0].transform
    ));
}

/// Scenario: Request integers reach the boundaries of signed OTAP attributes.
/// Guarantees: Zero and i64::MAX are exact; larger values error without wrapping or saturation.
#[test]
fn checked_context_attribute_range() {
    let source: ValueSource =
        serde_json::from_value(json!({"from_context":"received:value","transform":"int"})).unwrap();
    for integer in [0, i64::MAX as u64, i64::MAX as u64 + 1, u64::MAX] {
        let mut context = Context::default();
        store_context_text(&mut context, "received", integer.to_string());
        let attr = source.resolve(&context);
        if let Ok(expected) = i64::try_from(integer) {
            assert_eq!(attr.unwrap(), Some(LiteralValue::Int(expected)));
        } else {
            assert!(attr.is_err());
        }
    }
    let context = Context::default();
    assert_eq!(source.resolve(&context).unwrap(), None);
}

/// Scenario: Explicit context conversions drive insert, upsert, and update for all signals and encodings.
/// Guarantees: All attribute domains receive requested scalar types, including empty strings, false, and signed limits.
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
        let expected = match &scalar {
            LiteralValue::Str(value) => AnyValue::new_string(value),
            LiteralValue::Bool(value) => AnyValue::new_bool(*value),
            LiteralValue::Int(value) => AnyValue::new_int(*value),
            LiteralValue::Double(value) => AnyValue::new_double(*value),
        };
        let (text, transform) = match &scalar {
            LiteralValue::Str(value) => (value.clone(), "string"),
            LiteralValue::Bool(value) => (value.to_string(), "bool"),
            LiteralValue::Int(value) => (value.to_string(), "int"),
            LiteralValue::Double(value) => (value.to_string(), "double"),
        };
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
                            {"action":"insert","key":"insert_new","value":{"from_context":"scalar:value","transform":transform}},
                            {"action":"insert","key":"insert_existing","value":{"from_context":"scalar","transform":transform}},
                            {"action":"upsert","key":"upsert_existing","value":{"from_context":"scalar","transform":transform}},
                            {"action":"update","key":"update_existing","value":{"from_context":"scalar","transform":transform}},
                            {"action":"update","key":"update_missing","value":{"from_context":"scalar","transform":transform}}
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
                        {"action":"insert","key":"insert_existing","value":{"from_headers":["host"]}},
                        {"action":"insert","key":"insert_new","value":{"from_headers":["host"],"transform":"authority_host"}},
                        {"action":"upsert","key":"upsert_existing","value":{"from_headers":["host"]}},
                        {"action":"update","key":"update_existing","value":{"from_context":"received","transform":"int"}},
                        {"action":"update","key":"update_missing","value":{"from_context":"received","transform":"int"}},
                        {"action":"insert","key":"missing","value":{"from_headers":["uncaptured"]}},
                        {"action":"insert","key":"logs_only","value":{"from_context":"received"},"signals":["logs"]},
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
                                Some(&AnyValue::new_string("new.example"))
                            );
                            assert_eq!(
                                value(&attributes, "upsert_existing"),
                                Some(&AnyValue::new_string("new.example:443"))
                            );
                            assert_eq!(
                                value(&attributes, "update_existing"),
                                Some(&AnyValue::new_int(456))
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
                {"action":"insert","key":"server.address","value":{"from_headers":["host"]}},
                {"action":"insert","key":"missing","value":{"from_context":"missing"}}
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
        json!({"actions":[{"action":"insert","key":"x","value":{"from_context":"absent"}}]}),
        json!({"actions":[{"action":"insert","key":"x","value":{"from_headers":["host"]}}]}),
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
                    {"action":"insert","key":"invalid","value":{"from_headers":[]}}
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

/// Scenario: Live updates switch authority selection and then supply an invalid selection.
/// Guarantees: Default first selection changes to last atomically and an invalid update preserves last.
#[test]
fn authority_selection_configuration_updates() {
    let config = |selection: &str| {
        json!({"actions":[
            {"action":"insert","key":"selected","value":{
                "from_headers":["host"],"transform":"authority_host","selection":selection
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
            "from_headers":["host"],"transform":"authority_host"
        }}]}),
        vec![
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("last"),
            }),
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("middle"),
            }),
            pdata(),
            Message::Control(NodeControlMsg::Config {
                config: config("first"),
            }),
            pdata(),
        ],
        |output| {
            assert_eq!(output.len(), 4);
            for (payload, expected) in output.into_iter().zip([
                "first.example",
                "last.example",
                "last.example",
                "first.example",
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
        serde_json::from_value::<ValueSource>(json!({
            "from_context":name,"transform":"authority_host","selection":"last"
        }))
        .unwrap()
    };
    let mut context = Context::default();
    assert_eq!(source("request:value").resolve(&context).unwrap(), None);
    store_context_text(&mut context, "request", "first.example,last.example".into());
    assert_eq!(source("absent").resolve(&context).unwrap(), None);
    assert_eq!(source("absent:value").resolve(&context).unwrap(), None);
    assert_eq!(source("request:absent").resolve(&context).unwrap(), None);
    assert_eq!(
        source("request").resolve(&context).unwrap(),
        Some(LiteralValue::Str("last.example".into()))
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
        source("request")
            .resolve(&context)
            .unwrap_err()
            .to_string()
            .contains("entry:member")
    );
    assert_eq!(
        source("request:last").resolve(&context).unwrap(),
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
        source("request:host")
            .resolve(&context)
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
            source("request:host")
                .resolve(&context)
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
}

/// Scenario: Dynamic request metadata exceeds the representable range in an otherwise valid message.
/// Guarantees: The processor sends a permanent Nack with the original payload and request context.
#[test]
fn context_range_error_permanently_nacks_original_message() {
    let rt = TestRuntime::new();
    let mut config = NodeUserConfig::new_processor_config(ATTRIBUTES_PROCESSOR_URN);
    config.config = json!({"actions":[{"action":"insert","key":"received","value":{"from_context":"received","transform":"int"}}]});
    let processor = create_attributes_processor(
        pipeline_context(),
        test_node("range-nack"),
        Arc::new(config),
        rt.config(),
    )
    .unwrap();
    let mut context = Context::default();
    store_context_text(&mut context, "received", u64::MAX.to_string());
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
            assert!(nack.reason.contains("transform"));
            assert_eq!(
                nack.refused.payload_ref().encoded_bytes().unwrap(),
                &original
            );
            let (context, _) = nack.refused.into_parts();
            assert_eq!(context.clone_detached(), original_context);
        })
        .validate(|_| async {});
}
