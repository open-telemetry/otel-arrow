#[cfg(test)]
mod test {
    use super::super::*;
    use otel_arrow_dfe_engine::{
        Interests,
        context::ControllerContext,
        control::{NackCause, NackMsg, pipeline_completion_msg_channel},
        testing::{
            processor::{TEST_OUT_PORT_NAME, TestRuntime},
            test_node,
        },
    };
    use otel_arrow_dfe_otap::testing::{TestCallData, next_ack, next_nack};
    use otel_arrow_dfe_pdata::{
        PayloadData, TryFromWithOptions,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
                trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData},
            },
        },
        testing::round_trip::{otap_to_otlp, otlp_to_otap, to_otap_logs},
    };
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn parsing_metric_points(registry: &TelemetryRegistryHandle) -> Vec<TransformMetricPoint> {
        metric_points_for(registry, "processor.log_parser")
            .into_iter()
            .filter(|point| point.field == "records")
            .collect()
    }

    /// Scenario: A parsed record has an unused sensitive value in an unrelated dictionary column.
    /// Guarantees: Default sanitization removes the unused value; opting out retains the original buffer.
    #[test]
    fn test_sanitization_removes_unused_dictionary_values() {
        use arrow::{
            array::{Array, DictionaryArray, RecordBatch, StringArray, UInt8Array},
            datatypes::{Field, Schema, UInt8Type},
        };
        use otel_arrow_dfe_pdata::{
            proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::consts,
        };
        for skip in [false, true] {
            let runtime = TestRuntime::<OtapPdata>::new();
            let mut config = parsing_config("json");
            let _ = config.as_object_mut().unwrap().remove("severity");
            config["skip_sanitize_result"] = skip.into();
            let processor = try_create_with_config(config, &runtime).unwrap();
            runtime
                .set_processor(processor)
                .run_test(move |mut ctx| async move {
                    let mut batch = to_otap_logs(vec![
                        LogRecord::build()
                            .body(AnyValue::new_string(
                                r#"{"raw-data":"body","ts":"1970-01-01T00:00:02Z"}"#,
                            ))
                            .severity_text("INFO")
                            .finish(),
                    ]);
                    let root = batch.get(ArrowPayloadType::Logs).unwrap();
                    let index = root.schema().index_of(consts::SEVERITY_TEXT).unwrap();
                    let dictionary = DictionaryArray::<UInt8Type>::try_new(
                        UInt8Array::from(vec![0]),
                        Arc::new(StringArray::from(vec!["INFO", "secret-unused-value"])),
                    )
                    .unwrap();
                    let mut fields = root.schema().fields().to_vec();
                    fields[index] = Arc::new(Field::new(
                        consts::SEVERITY_TEXT,
                        dictionary.data_type().clone(),
                        true,
                    ));
                    let mut columns = root.columns().to_vec();
                    columns[index] = Arc::new(dictionary);
                    let root =
                        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap();
                    batch.set(ArrowPayloadType::Logs, root).unwrap();
                    ctx.process(Message::PData(OtapPdata::new_default(batch.into())))
                        .await
                        .unwrap();
                    let mut outputs = ctx.drain_pdata().await;
                    assert_eq!(outputs.len(), 1);
                    let batch =
                        OtapArrowRecords::try_from_with_default(outputs.pop().unwrap().payload())
                            .unwrap();
                    let root = batch.get(ArrowPayloadType::Logs).unwrap();
                    let dictionary = root
                        .column_by_name(consts::SEVERITY_TEXT)
                        .unwrap()
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .unwrap();
                    let values = dictionary
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .unwrap();
                    assert_eq!(
                        values
                            .iter()
                            .flatten()
                            .any(|value| value == "secret-unused-value"),
                        skip
                    );
                })
                .validate(|_ctx| async {});
        }
    }

    /// Scenario: Processor-only settings, old wrappers and query settings reach both config paths.
    /// Guarantees: Sanitization is enabled by default and unsupported or mistyped settings fail early.
    #[test]
    fn test_strict_processor_configuration() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let config = parsing_config("json");
        assert!(parse_config(&config).unwrap().1);
        for skip in [false, true] {
            let mut config = config.clone();
            config["skip_sanitize_result"] = skip.into();
            assert_eq!(parse_config(&config).unwrap().1, !skip);
            assert!(validate_log_parser_config(&config).is_ok());
            assert!(try_create_with_config(config, &runtime).is_ok());
        }
        for key in [
            "inbound_request_limit",
            "outbound_request_limit",
            "filter_attribute_keys_case_sensitive",
            "opl_query",
            "parse_logs",
            "unknown",
        ] {
            let mut invalid = config.clone();
            invalid[key] = json!(1);
            assert!(validate_log_parser_config(&invalid).is_err(), "{key}");
            assert!(try_create_with_config(invalid, &runtime).is_err(), "{key}");
        }
        for invalid in [Value::Null, json!([]), json!({"parse_logs": config}), {
            let mut config = parsing_config("json");
            config["skip_sanitize_result"] = json!("false");
            config
        }] {
            assert!(validate_log_parser_config(&invalid).is_err());
            assert!(try_create_with_config(invalid, &runtime).is_err());
        }
    }

    /// Scenario: An empty logs payload arrives with an upstream delivery subscriber.
    /// Guarantees: It completes locally exactly once without forwarding empty data or requesting drop decisions.
    #[test]
    fn test_empty_batch_completes_locally() {
        use otel_arrow_dfe_engine::control::PipelineCompletionMsg;
        let runtime = TestRuntime::<OtapPdata>::new();
        let processor = try_create_with_config(parsing_config("json"), &runtime).unwrap();
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(2);
                ctx.set_pipeline_completion_sender(completion_tx);
                let pdata = create_pdata_with_subscriber(
                    to_otap_logs(Vec::new()),
                    Interests::ACKS | Interests::NACKS,
                    7,
                    999,
                );
                ctx.process(Message::PData(pdata)).await.unwrap();
                assert!(ctx.drain_pdata().await.is_empty());
                match completion_rx.try_recv().unwrap() {
                    PipelineCompletionMsg::DeliverAck { mut ack } => {
                        assert_eq!(ack.accepted.num_items(), 0);
                        assert_eq!(ack.accepted.signal_type(), SignalType::Logs);
                    }
                    other => panic!("unexpected completion {other:?}"),
                }
                assert!(completion_rx.try_recv().is_err());
            })
            .validate(|_ctx| async {});
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline = controller.pipeline_context_with("group".into(), "pipeline".into(), 0, 1, 0);
        let parser = LogParserProcessor::from_config(&pipeline, &parsing_config("json")).unwrap();
        assert!(!parser.runtime_requirements().makes_drop_decisions);
    }

    /// Scenario: Real parser and OPL factories process valid and malformed records in sequence.
    /// Guarantees: OPL filters the mapped body, malformed input survives, and delivery subscribers remain downstream-owned.
    #[test]
    fn test_parser_to_opl_pipeline() {
        use crate::processors::transform_processor::TRANSFORM_PROCESSOR_FACTORY;
        use std::{cell::RefCell, rc::Rc};
        let parsed = Rc::new(RefCell::new(None));
        let result = parsed.clone();
        let runtime = TestRuntime::<OtapPdata>::new();
        let processor = try_create_with_config(parsing_config("json"), &runtime).unwrap();
        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (sender, mut completion) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(sender);
                let records = [
                    r#"{"raw-data":"keep","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
                    r#"{"raw-data":"drop","ts":"1970-01-01T00:00:02Z","sev":"INFO"}"#,
                    "malformed",
                ]
                .into_iter()
                .map(|body| LogRecord::build().body(AnyValue::new_string(body)).finish())
                .collect();
                ctx.process(Message::PData(create_pdata_with_subscriber(
                    to_otap_logs(records),
                    Interests::ACKS | Interests::NACKS,
                    7,
                    999,
                )))
                .await
                .unwrap();
                assert!(completion.try_recv().is_err());
                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                *result.borrow_mut() = output.pop();
            })
            .validate(|_ctx| async {});
        let input = parsed.borrow_mut().take().unwrap();
        let runtime = TestRuntime::<OtapPdata>::new();
        let controller = ControllerContext::new(runtime.metrics_registry());
        let pipeline = controller.pipeline_context_with("group".into(), "pipeline".into(), 0, 1, 0);
        let mut node = NodeUserConfig::new_processor_config(TRANSFORM_PROCESSOR_FACTORY.name);
        node.config = json!({"opl_query": "logs | where body != \"drop\""});
        node.default_output = Some(TEST_OUT_PORT_NAME.into());
        let processor = (TRANSFORM_PROCESSOR_FACTORY.create)(
            pipeline,
            test_node("transform"),
            Arc::new(node),
            runtime.config(),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
        .unwrap();
        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (sender, mut completion) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(sender);
                ctx.process(Message::PData(input)).await.unwrap();
                assert!(completion.try_recv().is_err());
                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                let output = output.pop().unwrap();
                let batch =
                    OtapArrowRecords::try_from_with_default(output.clone().payload()).unwrap();
                let OtlpProtoMessage::Logs(logs) = otap_to_otlp(&batch) else {
                    panic!("expected logs")
                };
                let records = &logs.resource_logs[0].scope_logs[0].log_records;
                assert_eq!(records.len(), 2);
                assert_eq!(records[0].body, Some(AnyValue::new_string("keep")));
                assert_eq!(records[1].body, Some(AnyValue::new_string("malformed")));
                assert_eq!(next_ack(AckMsg::new(output.clone())).unwrap().0, 999);
                for permanent in [false, true] {
                    let nack = if permanent {
                        NackMsg::new_permanent_with_cause(
                            "downstream",
                            output.clone(),
                            NackCause::Refused,
                        )
                    } else {
                        NackMsg::new_with_cause("downstream", output.clone(), NackCause::Refused)
                    };
                    let (node, nack) = next_nack(nack).unwrap();
                    assert_eq!(node, 999);
                    assert_eq!(nack.permanent, permanent);
                }
            })
            .validate(|_ctx| async {});
    }

    fn set_pdata_sender(
        port_name: &'static str,
        processor: &mut ProcessorWrapper<OtapPdata>,
    ) -> otel_arrow_dfe_channel::mpsc::Receiver<OtapPdata> {
        use otel_arrow_dfe_engine::{
            local::message::LocalSender, message::Sender, node::NodeWithPDataSender,
        };
        let (sender, receiver) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
        processor
            .set_pdata_sender(
                test_node("downstream"),
                port_name.into(),
                Sender::Local(LocalSender::mpsc(sender)),
            )
            .unwrap();
        receiver
    }
    #[derive(Debug, PartialEq, Eq)]
    struct TransformMetricPoint {
        field: String,
        attributes: BTreeMap<String, String>,
        value: u64,
    }

    fn transform_metric_points(
        telemetry_registry: &TelemetryRegistryHandle,
    ) -> Vec<TransformMetricPoint> {
        metric_points_for(telemetry_registry, "processor.log_parser")
    }

    fn metric_points_for(
        telemetry_registry: &TelemetryRegistryHandle,
        metric_set: &str,
    ) -> Vec<TransformMetricPoint> {
        let mut points = Vec::new();
        telemetry_registry.visit_current_metrics_with_item_attrs(
            |descriptor, _entity_attributes, item_attributes, metrics| {
                if descriptor.name != metric_set {
                    return;
                }

                let attributes = item_attributes
                    .iter()
                    .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                    .collect::<BTreeMap<_, _>>();
                for (field, value) in metrics {
                    points.push(TransformMetricPoint {
                        field: field.name.to_string(),
                        attributes: attributes.clone(),
                        value: value.to_u64_lossy(),
                    });
                }
            },
            false,
        );
        points
    }

    fn transform_metric_value(
        points: &[TransformMetricPoint],
        field: &str,
        expected_attributes: &[(&str, &str)],
    ) -> Option<u64> {
        points
            .iter()
            .find(|point| {
                point.field == field
                    && expected_attributes.iter().all(|(key, value)| {
                        point.attributes.get(*key).map(String::as_str) == Some(*value)
                    })
            })
            .map(|point| point.value)
    }

    /// Helper to create pdata with subscribers for testing Ack/Nack
    fn create_pdata_with_subscriber(
        otap_batch: OtapArrowRecords,
        interests: Interests,
        call_data_id: u64,
        node_id: usize,
    ) -> OtapPdata {
        OtapPdata::new_default(otap_batch.into()).test_subscribe_to(
            interests,
            TestCallData::new_with(call_data_id, 0).into(),
            node_id,
        )
    }

    fn try_create_with_config(
        config: Value,
        runtime: &TestRuntime<OtapPdata>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut node_config = NodeUserConfig::new_processor_config(LOG_PARSER_PROCESSOR_URN);
        node_config.config = config;
        node_config.default_output = Some(TEST_OUT_PORT_NAME.into());

        let telemetry_registry_handle = runtime.metrics_registry();
        let controller_context = ControllerContext::new(telemetry_registry_handle);
        let pipeline_context = controller_context.pipeline_context_with(
            "group_id".into(),
            "pipeline_id".into(),
            0,
            1,
            0,
        );
        let node_id = test_node("log-parser-processor");
        create_log_parser_processor(
            pipeline_context,
            node_id,
            Arc::new(node_config),
            runtime.config(),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
    }

    fn parsing_config(format: &str) -> Value {
        let prefix = if format == "json" { "/" } else { "" };
        let mut parsing = json!({
            "format": format, "on_error": "preserve",
            "timestamp": {"source": format!("{prefix}ts"), "format": "rfc3339", "on_missing": "observed"},
            "severity": {"source": format!("{prefix}sev"), "mapping": {"ERROR":17,"INFO":9}},
            "limits": {"max_input_bytes":1048576,"max_scratch_bytes":8388608,
                "max_pattern_bytes":4096,"max_compiled_regex_bytes":1048576,"max_json_depth":32,"max_entries":4096}
        });
        match format {
            "regex" => {
                parsing["pattern"] = r"(?s)^START: (?P<ts>\S+) (?P<sev>\S+) (?P<message>.*)$".into()
            }
            "csv" => {
                parsing["columns"] = json!(["ts", "sev", "raw-data"]);
                parsing["delimiter"] = ",".into();
                parsing["header"] = "none".into();
                parsing["body"] = json!({"source":"raw-data"});
            }
            _ => parsing["body"] = json!({"source":"/raw-data"}),
        }
        parsing
    }

    /// Scenario: Each parsing format receives a good/bad/good batch with provenance and an ACK subscriber.
    /// Guarantees: Only valid records change, full logical metadata and order survive, and no early ACK is sent.
    #[test]
    fn test_parse_logs_framed_batches() {
        for (format, input, body, text, number) in [
            (
                "regex",
                "START: 1970-01-01T00:00:02Z ERROR failure\n detail",
                "START: 1970-01-01T00:00:02Z ERROR failure\n detail",
                "ERROR",
                17,
            ),
            (
                "json",
                r#"{"raw-data":"failure","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
                "failure",
                "ERROR",
                17,
            ),
            (
                "csv",
                "1970-01-01T00:00:02Z,INFO,\"hello, \"\"world\"\"\"",
                "hello, \"world\"",
                "INFO",
                9,
            ),
        ] {
            let runtime = TestRuntime::<OtapPdata>::new();
            let node = include_str!("README.md")
                .split("```yaml")
                .skip(1)
                .map(|block| {
                    serde_yaml::from_str::<Value>(block.split("```").next().unwrap()).unwrap()
                })
                .find(|node| node["config"]["format"] == format)
                .unwrap();
            let processor = try_create_with_config(node["config"].clone(), &runtime).unwrap();
            runtime
                .set_processor(processor)
                .run_test(move |mut ctx| async move {
                    let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                    ctx.set_pipeline_completion_sender(completion_tx);
                    let mut records = Vec::new();
                    for value in [input, "malformed", input] {
                        let mut record = LogRecord::build()
                            .body(AnyValue::new_string(value))
                            .finish();
                        record.observed_time_unix_nano = 3_000_000_000;
                        record.attributes = vec![
                            KeyValue::new("log.file.path", AnyValue::new_string("fixture.log")),
                            KeyValue::new("unrelated", AnyValue::new_string("retained")),
                        ];
                        records.push(record);
                    }
                    let mut expected = LogsData {
                        resource_logs: vec![ResourceLogs::new(
                            Resource {
                                attributes: vec![KeyValue::new(
                                    "service.name",
                                    AnyValue::new_string("fixture"),
                                )],
                                ..Default::default()
                            },
                            vec![ScopeLogs::new(
                                InstrumentationScope {
                                    name: "fixture-scope".into(),
                                    ..Default::default()
                                },
                                records,
                            )],
                        )],
                    };
                    let input_batch = otlp_to_otap(&OtlpProtoMessage::Logs(expected.clone()));
                    let pdata = create_pdata_with_subscriber(
                        input_batch,
                        Interests::ACKS | Interests::NACKS,
                        1,
                        999,
                    );
                    let (expected_context, payload) = pdata.into_parts();
                    let pdata = OtapPdata::new(expected_context.clone(), payload);
                    ctx.process(Message::PData(pdata)).await.unwrap();
                    assert!(
                        completion_rx.try_recv().is_err(),
                        "parsing must not complete upstream"
                    );
                    let output = ctx.drain_pdata().await;
                    assert_eq!(output.len(), 1);
                    let (actual_context, _) = output[0].clone().into_parts();
                    assert_eq!(actual_context, expected_context);
                    let (upstream, _) = next_ack(AckMsg::new(output[0].clone())).unwrap();
                    assert_eq!(upstream, 999);
                    for permanent in [false, true] {
                        let nack = if permanent {
                            NackMsg::new_permanent_with_cause(
                                "downstream",
                                output[0].clone(),
                                NackCause::Refused,
                            )
                        } else {
                            NackMsg::new_with_cause(
                                "downstream",
                                output[0].clone(),
                                NackCause::Refused,
                            )
                        };
                        let (upstream, nack) = next_nack(nack).unwrap();
                        assert_eq!(upstream, 999);
                        assert_eq!(nack.permanent, permanent);
                        assert_eq!(nack.cause, NackCause::Refused);
                    }
                    for index in [0, 2] {
                        let record =
                            &mut expected.resource_logs[0].scope_logs[0].log_records[index];
                        record.body = Some(AnyValue::new_string(body));
                        record.time_unix_nano = 2_000_000_000;
                        record.severity_text = text.into();
                        record.severity_number = number;
                    }
                    let batch = OtapArrowRecords::try_from_with_default(
                        output.into_iter().next().unwrap().payload(),
                    )
                    .unwrap();
                    match otap_to_otlp(&batch) {
                        OtlpProtoMessage::Logs(actual) => assert_eq!(actual, expected),
                        other => panic!("unexpected output {other:?}"),
                    }
                })
                .validate(|_ctx| async {});
        }
    }

    /// Scenario: Parsing is combined with a query or has unknown mappings and invalid limits.
    /// Guarantees: Invalid modes fail at construction and query configuration is rejected by the parser.
    #[test]
    fn test_parse_logs_configuration_gate() {
        let runtime = TestRuntime::<OtapPdata>::new();
        assert!(
            try_create_with_config(json!({"opl_query":"logs | where true"}), &runtime).is_err()
        );
        let mut config = parsing_config("json");
        config["opl_query"] = "logs".into();
        assert!(try_create_with_config(config, &runtime).is_err());
        let mut config = parsing_config("json");
        config["unknown"] = true.into();
        assert!(try_create_with_config(config, &runtime).is_err());
        let mut config = parsing_config("regex");
        config["limits"]["max_pattern_bytes"] = 1.into();
        assert!((LOG_PARSER_PROCESSOR_FACTORY.validate_config)(&config).is_err());
        assert!(try_create_with_config(config, &runtime).is_err());
    }

    /// Scenario: Every documented YAML parsing node is deserialized through the real configuration path.
    /// Guarantees: Documentation examples compile, select one format, and remain constructible processor configs.
    #[test]
    fn test_documented_parsing_yaml() {
        let documentation = include_str!("README.md");
        let runtime = TestRuntime::<OtapPdata>::new();
        let mut examples = 0;
        let mut composition_examples = 0;
        for block in documentation.split("```yaml").skip(1) {
            let yaml = block.split("```").next().unwrap();
            let node: Value = serde_yaml::from_str(yaml).unwrap();
            if let Some(nodes) = node.get("nodes") {
                use crate::processors::transform_processor::TRANSFORM_PROCESSOR_FACTORY;
                (LOG_PARSER_PROCESSOR_FACTORY.validate_config)(&nodes["parse"]["config"]).unwrap();
                (TRANSFORM_PROCESSOR_FACTORY.validate_config)(&nodes["filter"]["config"]).unwrap();
                assert!(try_create_with_config(nodes["parse"]["config"].clone(), &runtime).is_ok());
                composition_examples += 1;
            }
            if node["config"].get("format").is_some() {
                (LOG_PARSER_PROCESSOR_FACTORY.validate_config)(&node["config"]).unwrap();
                assert!(try_create_with_config(node["config"].clone(), &runtime).is_ok());
                examples += 1;
            }
        }
        assert_eq!(examples, 3);
        assert_eq!(composition_examples, 1);
    }

    /// Scenario: Framed JSON records exercise missing-time fallback and errors in later mappings.
    /// Guarantees: Full records and context survive, and only committed fallbacks emit a fallback count.
    #[test]
    fn test_parse_logs_fallback_records_and_counters() {
        for (timestamp, event, observed, policy, severity, expected_time, reason) in [
            (
                None,
                0,
                3_000_000_000,
                "observed",
                "ERROR",
                Some(3_000_000_000),
                Some("observed_fallback"),
            ),
            (
                Some(Value::Null),
                0,
                3_000_000_000,
                "observed",
                "ERROR",
                Some(3_000_000_000),
                Some("observed_fallback"),
            ),
            (
                None,
                2_000_000_000,
                3_000_000_000,
                "observed",
                "ERROR",
                Some(2_000_000_000),
                None,
            ),
            (
                Some(Value::Null),
                2_000_000_000,
                3_000_000_000,
                "observed",
                "ERROR",
                Some(2_000_000_000),
                None,
            ),
            (None, 0, 0, "observed", "ERROR", None, Some("timestamp")),
            (None, 0, 0, "preserve", "ERROR", Some(0), None),
            (Some(Value::Null), 0, 0, "preserve", "ERROR", Some(0), None),
            (
                Some(json!("")),
                2_000_000_000,
                3_000_000_000,
                "observed",
                "ERROR",
                None,
                Some("timestamp"),
            ),
            (
                Some(json!("malformed")),
                0,
                3_000_000_000,
                "observed",
                "ERROR",
                None,
                Some("timestamp"),
            ),
            (
                Some(json!("1970-01-01T00:00:02")),
                0,
                3_000_000_000,
                "observed",
                "ERROR",
                None,
                Some("timestamp"),
            ),
            (
                None,
                0,
                3_000_000_000,
                "observed",
                "unknown",
                None,
                Some("severity"),
            ),
        ] {
            let runtime = TestRuntime::<OtapPdata>::new();
            let registry = runtime.metrics_registry();
            let reporter = runtime.metrics_reporter();
            let mut config = parsing_config("json");
            config["timestamp"]["on_missing"] = policy.into();
            let processor = try_create_with_config(config, &runtime).unwrap();
            runtime
                .set_processor(processor)
                .run_test(move |mut ctx| async move {
                    let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                    ctx.set_pipeline_completion_sender(completion_tx);
                    let mut body = json!({"raw-data":"failure", "sev":severity});
                    if let Some(timestamp) = timestamp {
                        body["ts"] = timestamp;
                    }
                    let mut record = LogRecord::build()
                        .body(AnyValue::new_string(body.to_string()))
                        .finish();
                    record.time_unix_nano = event;
                    record.observed_time_unix_nano = observed;
                    record.severity_text = "INFO".into();
                    record.severity_number = 9;
                    record.attributes = vec![
                        KeyValue::new("log.file.path", AnyValue::new_string("fixture.log")),
                        KeyValue::new("unrelated", AnyValue::new_string("retained")),
                    ];
                    let mut expected = LogsData {
                        resource_logs: vec![ResourceLogs::new(
                            Resource {
                                attributes: vec![KeyValue::new(
                                    "service.name",
                                    AnyValue::new_string("fixture"),
                                )],
                                ..Default::default()
                            },
                            vec![ScopeLogs::new(
                                InstrumentationScope {
                                    name: "fixture-scope".into(),
                                    ..Default::default()
                                },
                                vec![record],
                            )],
                        )],
                    };
                    let batch = otlp_to_otap(&OtlpProtoMessage::Logs(expected.clone()));
                    let (context, payload) = create_pdata_with_subscriber(
                        batch,
                        Interests::ACKS | Interests::NACKS,
                        1,
                        999,
                    )
                    .into_parts();
                    ctx.process(Message::PData(OtapPdata::new(context.clone(), payload)))
                        .await
                        .unwrap();
                    assert!(completion_rx.try_recv().is_err());
                    let mut outputs = ctx.drain_pdata().await;
                    assert_eq!(outputs.len(), 1);
                    let (output_context, output) = outputs.pop().unwrap().into_parts();
                    assert_eq!(output_context, context);
                    if let Some(timestamp) = expected_time {
                        let record = &mut expected.resource_logs[0].scope_logs[0].log_records[0];
                        record.body = Some(AnyValue::new_string("failure"));
                        record.time_unix_nano = timestamp;
                        record.severity_text = "ERROR".into();
                        record.severity_number = 17;
                    }
                    let batch = OtapArrowRecords::try_from_with_default(output).unwrap();
                    match otap_to_otlp(&batch) {
                        OtlpProtoMessage::Logs(actual) => assert_eq!(actual, expected),
                        other => panic!("unexpected output {other:?}"),
                    }
                    ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: reporter,
                    }))
                    .await
                    .unwrap();
                })
                .validate(move |_ctx| async move {
                    registry.flush_pending_metrics().await.unwrap();
                    let points = parsing_metric_points(&registry);
                    match reason {
                        Some(reason) => {
                            assert_eq!(
                                points.len(),
                                1,
                                "exactly one reason, without a discarded fallback"
                            );
                            assert_eq!(
                                transform_metric_value(
                                    &points,
                                    "records",
                                    &[("format", "json"), ("reason", reason)]
                                ),
                                Some(1)
                            );
                            assert_eq!(points[0].attributes.len(), 2);
                        }
                        None => assert!(points.is_empty()),
                    }
                    let operations = transform_metric_points(&registry);
                    assert_eq!(
                        transform_metric_value(
                            &operations,
                            "operations",
                            &[("signal", "logs"), ("outcome", "success")]
                        ),
                        Some(1)
                    );
                    assert!(operations.iter().all(|point| point.field != "failures"));
                });
        }
    }

    fn assert_parsing_fixture(
        config: Value,
        bodies: Vec<AnyValue>,
        mapped_body: Option<&str>,
        mapped_time: u64,
        reason: Option<&'static str>,
    ) {
        let runtime = TestRuntime::<OtapPdata>::new();
        let registry = runtime.metrics_registry();
        let reporter = runtime.metrics_reporter();
        let format = config["format"].as_str().unwrap().to_owned();
        let processor = try_create_with_config(config, &runtime).unwrap();
        let mapped_body = mapped_body.map(str::to_owned);
        let count = bodies.len() as u64;
        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(completion_tx);
                for (index, body) in bodies.into_iter().enumerate() {
                    let mut record = LogRecord::build().body(body).finish();
                    record.observed_time_unix_nano = 3_000_000_000;
                    record.attributes = vec![
                        KeyValue::new(
                            "log.file.path",
                            AnyValue::new_string(if index % 2 == 0 {
                                "first.log"
                            } else {
                                "second.log"
                            }),
                        ),
                        KeyValue::new("fragment.reason", AnyValue::new_string("size_limit")),
                        KeyValue::new("unrelated", AnyValue::new_string("retained")),
                    ];
                    let mut expected = LogsData {
                        resource_logs: vec![ResourceLogs::new(
                            Resource {
                                attributes: vec![KeyValue::new(
                                    "service.name",
                                    AnyValue::new_string("fixture"),
                                )],
                                ..Default::default()
                            },
                            vec![ScopeLogs::new(
                                InstrumentationScope {
                                    name: "fixture-scope".into(),
                                    ..Default::default()
                                },
                                vec![record],
                            )],
                        )],
                    };
                    let batch = otlp_to_otap(&OtlpProtoMessage::Logs(expected.clone()));
                    let (context, payload) = create_pdata_with_subscriber(
                        batch,
                        Interests::ACKS | Interests::NACKS,
                        1,
                        999,
                    )
                    .into_parts();
                    ctx.process(Message::PData(OtapPdata::new(context.clone(), payload)))
                        .await
                        .unwrap();
                    assert!(completion_rx.try_recv().is_err());
                    let mut output = ctx.drain_pdata().await;
                    assert_eq!(output.len(), 1);
                    let (actual_context, payload) = output.pop().unwrap().into_parts();
                    assert_eq!(actual_context, context);
                    if let Some(body) = &mapped_body {
                        let record = &mut expected.resource_logs[0].scope_logs[0].log_records[0];
                        record.body = Some(AnyValue::new_string(body));
                        record.time_unix_nano = mapped_time;
                        record.severity_text = "ERROR".into();
                        record.severity_number = 17;
                    }
                    let batch = OtapArrowRecords::try_from_with_default(payload).unwrap();
                    match otap_to_otlp(&batch) {
                        OtlpProtoMessage::Logs(actual) => assert_eq!(actual, expected),
                        other => panic!("unexpected output {other:?}"),
                    }
                }
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }))
                .await
                .unwrap();
            })
            .validate(move |_ctx| async move {
                registry.flush_pending_metrics().await.unwrap();
                let points = parsing_metric_points(&registry);
                match reason {
                    Some(reason) => {
                        assert_eq!(points.len(), 1);
                        assert_eq!(
                            transform_metric_value(
                                &points,
                                "records",
                                &[("format", &format), ("reason", reason)]
                            ),
                            Some(count)
                        );
                        assert_eq!(points[0].attributes.len(), 2);
                    }
                    None => assert!(points.is_empty()),
                }
            });
    }

    /// Scenario: CSV rejection, interleaved sources and fresh instances process complete framed rows.
    /// Guarantees: Errors preserve full records and context; data rows map without header or source state.
    #[test]
    fn test_parse_logs_csv_acceptance_records() {
        for (input, reason) in [
            ("ts,sev,raw-data", "timestamp"),
            ("one,two", "extraction"),
            ("one,two,three,four", "extraction"),
            (",ERROR,body", "timestamp"),
            ("\"unterminated", "extraction"),
            ("one,two,\"bad\"tail", "extraction"),
            ("one,two,bad\"quote", "extraction"),
            ("one,two,\"embedded\nnewline\"", "extraction"),
            ("one,two,\"embedded\rnewline\"", "extraction"),
        ] {
            assert_parsing_fixture(
                parsing_config("csv"),
                vec![AnyValue::new_string(input)],
                None,
                0,
                Some(reason),
            );
        }
        for _restart in 0..2 {
            assert_parsing_fixture(
                parsing_config("csv"),
                vec![AnyValue::new_string("1970-01-01T00:00:02Z,ERROR,body"); 4],
                Some("body"),
                2_000_000_000,
                None,
            );
        }
    }

    /// Scenario: JSON shapes, escaped pointers, regex fragments and unsupported bodies enter the processor.
    /// Guarantees: Full logical records, fragment metadata, source identity, context and error counters remain correct.
    #[test]
    fn test_parse_logs_shape_and_fragment_records() {
        for input in [
            r#"{"a":1,"a":2}"#,
            r#"{"a":1,"\u0061":2}"#,
            "{} {}",
            "[]",
            "null",
        ] {
            assert_parsing_fixture(
                parsing_config("json"),
                vec![AnyValue::new_string(input)],
                None,
                0,
                Some("extraction"),
            );
        }
        assert_parsing_fixture(
            parsing_config("json"),
            vec![AnyValue::new_string(
                r#"{"raw-data":42,"ts":"bad","sev":"unknown"}"#,
            )],
            None,
            0,
            Some("body"),
        );
        for (pointer, input) in [
            (
                "/a~1b",
                r#"{"a/b":"body","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
            ),
            (
                "/nested/0/value",
                r#"{"nested":[{"value":"body"}],"ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
            ),
        ] {
            let mut config = parsing_config("json");
            config["body"]["source"] = pointer.into();
            assert_parsing_fixture(
                config,
                vec![AnyValue::new_string(input)],
                Some("body"),
                2_000_000_000,
                None,
            );
        }
        assert_parsing_fixture(
            parsing_config("regex"),
            vec![AnyValue::new_string("fragment without header")],
            None,
            0,
            Some("extraction"),
        );
        let input = "START: 1970-01-01T00:00:02Z ERROR fragment\n detail";
        assert_parsing_fixture(
            parsing_config("regex"),
            vec![AnyValue::new_string(input)],
            Some(input),
            2_000_000_000,
            None,
        );
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValueList, any_value};
        assert_parsing_fixture(
            parsing_config("json"),
            vec![
                AnyValue {
                    value: Some(any_value::Value::BytesValue(vec![0xff, 0])),
                },
                AnyValue {
                    value: Some(any_value::Value::KvlistValue(KeyValueList {
                        values: vec![KeyValue::new("field", AnyValue::new_string("value"))],
                    })),
                },
            ],
            None,
            0,
            Some("unsupported_body"),
        );
    }

    /// Scenario: Input, scratch, depth and entry limits are exact or exceeded for framed processor inputs.
    /// Guarantees: At-limit records transform and over-limit records preserve all fields and context with one limit counter.
    #[test]
    fn test_parse_logs_limit_records() {
        for (format, input, body) in [
            (
                "regex",
                "START: 1970-01-01T00:00:02Z ERROR body",
                "START: 1970-01-01T00:00:02Z ERROR body",
            ),
            (
                "json",
                r#"{"raw-data":"body","ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#,
                "body",
            ),
            ("csv", "1970-01-01T00:00:02Z,ERROR,body", "body"),
        ] {
            let config = parsing_config(format);
            let parser =
                parse_logs::Parser::new(serde_json::from_value(config.clone()).unwrap()).unwrap();
            for (limit, bound) in [
                ("max_input_bytes", input.len()),
                ("max_scratch_bytes", parser.scratch_bound(input).unwrap()),
            ] {
                for over in [false, true] {
                    let mut config = config.clone();
                    config["limits"][limit] = (bound - usize::from(over)).into();
                    assert_parsing_fixture(
                        config,
                        vec![AnyValue::new_string(input)],
                        (!over).then_some(body),
                        2_000_000_000,
                        over.then_some("limit"),
                    );
                }
            }
        }
        let input = r#"{"nested":{"body":"body"},"ts":"1970-01-01T00:00:02Z","sev":"ERROR"}"#;
        for (limit, bound) in [("max_json_depth", 2), ("max_entries", 4)] {
            for over in [false, true] {
                let mut config = parsing_config("json");
                config["body"]["source"] = "/nested/body".into();
                config["limits"][limit] = (bound - usize::from(over)).into();
                assert_parsing_fixture(
                    config,
                    vec![AnyValue::new_string(input)],
                    (!over).then_some("body"),
                    2_000_000_000,
                    over.then_some("limit"),
                );
            }
        }
        let mut config = parsing_config("csv");
        config["limits"]["max_entries"] = 3.into();
        assert_parsing_fixture(
            config.clone(),
            vec![AnyValue::new_string("1970-01-01T00:00:02Z,ERROR,body")],
            Some("body"),
            2_000_000_000,
            None,
        );
        assert_parsing_fixture(
            config,
            vec![AnyValue::new_string(
                "1970-01-01T00:00:02Z,ERROR,body,extra",
            )],
            None,
            0,
            Some("limit"),
        );
    }

    /// Scenario: RFC 3339 offsets, nanoseconds and prohibited values flow through native OTAP updates.
    /// Guarantees: Exact timestamps commit while invalid timestamps preserve the complete record and delivery context.
    #[test]
    fn test_parse_logs_timestamp_records() {
        for (timestamp, expected_time) in [
            ("1970-01-01T00:00:02Z", Some(2_000_000_000)),
            ("1970-01-01T01:00:02+01:00", Some(2_000_000_000)),
            ("1970-01-01T00:00:02.123456789Z", Some(2_123_456_789)),
            ("1969-12-31T23:59:59.999999999Z", None),
            ("2016-12-31T23:59:60Z", None),
            ("1970-01-01T00:00:02.1234567890Z", None),
            ("9999-12-31T23:59:59Z", None),
        ] {
            let input = json!({"raw-data":"body","sev":"ERROR","ts":timestamp}).to_string();
            assert_parsing_fixture(
                parsing_config("json"),
                vec![AnyValue::new_string(input)],
                expected_time.map(|_| "body"),
                expected_time.unwrap_or(0),
                expected_time.is_none().then_some("timestamp"),
            );
        }
    }

    /// Scenario: A nonempty trace batch passes through a log-parsing processor with delivery subscribers.
    /// Guarantees: The native payload and context are identical and no log parsing counters are emitted.
    #[test]
    fn test_parse_logs_non_log_passthrough() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let registry = runtime.metrics_registry();
        let reporter = runtime.metrics_reporter();
        let processor = try_create_with_config(parsing_config("json"), &runtime).unwrap();
        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(completion_tx);
                let batch = otlp_to_otap(&OtlpProtoMessage::Traces(TracesData {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue::new(
                                "service.name",
                                AnyValue::new_string("fixture"),
                            )],
                            ..Default::default()
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: "fixture-scope".into(),
                                ..Default::default()
                            }),
                            spans: vec![Span {
                                name: "untouched".into(),
                                trace_id: vec![1; 16],
                                span_id: vec![2; 8],
                                attributes: vec![KeyValue::new(
                                    "unrelated",
                                    AnyValue::new_string("retained"),
                                )],
                                ..Default::default()
                            }],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                }));
                let original = batch.clone();
                let (context, payload) =
                    create_pdata_with_subscriber(batch, Interests::ACKS | Interests::NACKS, 1, 999)
                        .into_parts();
                ctx.process(Message::PData(OtapPdata::new(context.clone(), payload)))
                    .await
                    .unwrap();
                assert!(completion_rx.try_recv().is_err());
                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                let (actual_context, payload) = output.pop().unwrap().into_parts();
                assert_eq!(actual_context, context);
                match payload.into_data() {
                    PayloadData::OtapArrowRecords(actual) => assert_eq!(actual, original),
                    other => panic!("unexpected output {other:?}"),
                }
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }))
                .await
                .unwrap();
            })
            .validate(move |_ctx| async move {
                registry.flush_pending_metrics().await.unwrap();
                assert!(parsing_metric_points(&registry).is_empty());
                assert!(transform_metric_points(&registry).is_empty());
            });
    }

    /// Scenario: Invalid parser references, severity values, columns and limits reach factory validation.
    /// Guarantees: Every invalid configuration is rejected before a processor accepts input.
    #[test]
    fn test_parse_logs_factory_rejects_invalid_settings() {
        for (format, pointer, value) in [
            ("regex", "/severity/source", json!("absent")),
            ("regex", "/pattern", json!("(")),
            ("regex", "/limits/max_compiled_regex_bytes", json!(1)),
            ("csv", "/columns", json!(["ts", "sev", "sev"])),
            ("csv", "/body/source", json!("absent")),
            ("csv", "/header", json!("infer")),
            ("csv", "/delimiter", json!("||")),
            ("json", "/body/source", json!("/bad~2")),
            ("json", "/severity/mapping/ERROR", json!(0)),
            ("json", "/severity/mapping/ERROR", json!(25)),
            ("json", "/limits/max_input_bytes", json!(0)),
            ("json", "/limits/max_scratch_bytes", json!(0)),
            ("json", "/limits/max_json_depth", json!(0)),
            ("json", "/limits/max_entries", json!(0)),
        ] {
            let mut config = parsing_config(format);
            *config.pointer_mut(pointer).unwrap() = value;
            assert!(
                (LOG_PARSER_PROCESSOR_FACTORY.validate_config)(&config).is_err(),
                "{pointer}"
            );
        }
    }

    /// Scenario: A parsing-mode processor's default downstream channel is closed before dispatch.
    /// Guarantees: The send fails through the processor operation path and no successful ACK is emitted.
    #[test]
    fn test_parse_logs_failed_send() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let controller = ControllerContext::new(runtime.metrics_registry());
        let pipeline = controller.pipeline_context_with("group".into(), "pipeline".into(), 0, 1, 0);
        let mut node = NodeUserConfig::new_processor_config(LOG_PARSER_PROCESSOR_URN);
        node.config = parsing_config("json");
        node.default_output = Some("closed".into());
        let mut processor = create_log_parser_processor(
            pipeline,
            test_node("log-parser-processor"),
            Arc::new(node),
            runtime.config(),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
        .unwrap();
        let receiver = set_pdata_sender("closed", &mut processor);
        drop(receiver);
        runtime
            .set_processor(processor)
            .run_test(|mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(completion_tx);
                let input = to_otap_logs(vec![
                    LogRecord::build()
                        .body(AnyValue::new_string("malformed"))
                        .finish(),
                ]);
                let pdata =
                    create_pdata_with_subscriber(input, Interests::ACKS | Interests::NACKS, 1, 999);
                assert!(ctx.process(Message::PData(pdata)).await.is_err());
                assert!(completion_rx.try_recv().is_err());
            })
            .validate(|_ctx| async {});
    }

    /// Scenario: An internal parser failure occurs after staging one valid record with a timestamp fallback.
    /// Guarantees: The processor returns an internal failure without output, ACK, fallback or malformed-input counters.
    #[test]
    fn test_parse_logs_internal_failure_after_staging() {
        let runtime = TestRuntime::<OtapPdata>::new();
        let registry = runtime.metrics_registry();
        let reporter = runtime.metrics_reporter();
        let controller = ControllerContext::new(registry.clone());
        let pipeline = controller.pipeline_context_with("group".into(), "pipeline".into(), 0, 1, 0);
        let mut node = NodeUserConfig::new_processor_config(LOG_PARSER_PROCESSOR_URN);
        node.config = parsing_config("json");
        node.default_output = Some(TEST_OUT_PORT_NAME.into());
        let mut processor = LogParserProcessor::from_config(&pipeline, &node.config).unwrap();
        processor.log_parser.fail_after = Some(1);
        let processor = ProcessorWrapper::local(
            processor,
            test_node("log-parser-processor"),
            Arc::new(node),
            runtime.config(),
        );
        runtime
            .set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(completion_tx);
                let mut record = LogRecord::build()
                    .body(AnyValue::new_string(
                        r#"{"raw-data":"failure","sev":"ERROR"}"#,
                    ))
                    .finish();
                record.observed_time_unix_nano = 3_000_000_000;
                record.attributes = vec![
                    KeyValue::new("log.file.path", AnyValue::new_string("fixture.log")),
                    KeyValue::new("unrelated", AnyValue::new_string("retained")),
                ];
                let expected = LogsData {
                    resource_logs: vec![ResourceLogs::new(
                        Resource {
                            attributes: vec![KeyValue::new(
                                "service.name",
                                AnyValue::new_string("fixture"),
                            )],
                            ..Default::default()
                        },
                        vec![ScopeLogs::new(
                            InstrumentationScope {
                                name: "fixture-scope".into(),
                                ..Default::default()
                            },
                            vec![record; 2],
                        )],
                    )],
                };
                let batch = otlp_to_otap(&OtlpProtoMessage::Logs(expected.clone()));
                let original = batch.clone();
                let pdata =
                    create_pdata_with_subscriber(batch, Interests::ACKS | Interests::NACKS, 1, 999);
                let error = ctx
                    .process(Message::PData(pdata))
                    .await
                    .expect_err("injected internal failure");
                assert!(error.to_string().contains("injected staging failure"));
                assert!(ctx.drain_pdata().await.is_empty());
                assert!(completion_rx.try_recv().is_err());
                match otap_to_otlp(&original) {
                    OtlpProtoMessage::Logs(actual) => assert_eq!(actual, expected),
                    other => panic!("unexpected retained input {other:?}"),
                }
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }))
                .await
                .unwrap();
            })
            .validate(move |_ctx| async move {
                registry.flush_pending_metrics().await.unwrap();
                assert!(parsing_metric_points(&registry).is_empty());
                let points = transform_metric_points(&registry);
                assert_eq!(
                    transform_metric_value(
                        &points,
                        "operations",
                        &[("signal", "logs"), ("outcome", "failure")]
                    ),
                    Some(1)
                );
                assert_eq!(
                    transform_metric_value(
                        &points,
                        "failures",
                        &[("signal", "logs"), ("error.type", "internal")]
                    ),
                    Some(1)
                );
                assert_eq!(points.len(), 2);
            });
    }
}
