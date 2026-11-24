// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end topic flow tests for OTAP topic nodes.

use bytes::Bytes;
use otel_arrow_dfe_config::codecs::CodecsConfig;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::{SignalType, TopicName};
use otel_arrow_dfe_core_nodes::exporters::topic_exporter::{TOPIC_EXPORTER, TOPIC_EXPORTER_URN};
use otel_arrow_dfe_core_nodes::receivers::topic_receiver::{TOPIC_RECEIVER, TOPIC_RECEIVER_URN};
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::config::{ExporterConfig, ReceiverConfig};
use otel_arrow_dfe_engine::control::{
    Controllable, NackCause, NackMsg, NodeControlMsg, PipelineCompletionMsg,
    pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
};
use otel_arrow_dfe_engine::effect_handler::SourceTagging;
use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
use otel_arrow_dfe_engine::message::{Receiver as PDataReceiver, Sender as PDataSender};
use otel_arrow_dfe_engine::node::{NodeWithPDataReceiver, NodeWithPDataSender};
use otel_arrow_dfe_engine::runtime_services::PipelineRuntimeServices;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::testing::exporter::create_test_pipeline_context;
use otel_arrow_dfe_engine::testing::{
    create_not_send_channel, setup_test_runtime, test_node, test_pipeline_ctx_with_interests,
    test_pipeline_runtime_services,
};
use otel_arrow_dfe_engine::topic::{
    TopicBroadcastAckMode, TopicBroadcastOnLagPolicy, TopicBroker, TopicOptions, TopicSet,
};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_otap::testing::{TestCallData, next_nack};
use otel_arrow_dfe_pdata::encode::encode_logs_otap_batch;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
use otel_arrow_dfe_pdata::testing::fixtures::{
    logs_multiple_scopes_no_resource, traces_with_full_resource_and_scope,
};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtlpProtoBytes};
use otel_arrow_dfe_pdata_codec::builtins::syslog::{SyslogBatchBuilder, resolve_syslog};
use otel_arrow_dfe_pdata_codec::{
    CodecError, CodecMetadata, CodecOperation, CodecRegistration, DecodePolicy, PdataDecoder,
    PdataEncoding, PreparedCodecFactory, ResolvedCodec, register_pdata_codec,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use prost::Message as _;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn make_test_pdata() -> OtapPdata {
    let logs = otel_arrow_dfe_pdata::testing::fixtures::log_with_no_scope();
    let mut bytes = Vec::new();
    logs.encode(&mut bytes).expect("log payload should encode");
    OtapPdata::new_todo_context(OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)).into())
}

// A test codec whose binding selects one instrumentation scope from a log request.
const SCOPE_LOGS: PdataEncoding = PdataEncoding::new("test-scope-logs");
static METADATA: CodecMetadata = CodecMetadata::new(SCOPE_LOGS, &[SignalType::Logs]);

#[derive(Clone)]
struct ScopeSelector(String);

impl PreparedCodecFactory for ScopeSelector {
    fn create_decoder(
        &self,
        _: ResolvedCodec,
        _: DecodePolicy,
    ) -> Result<Box<dyn PdataDecoder>, CodecError> {
        Ok(Box::new(self.clone()))
    }
}

impl PdataDecoder for ScopeSelector {
    fn decode(&mut self, _: SignalType, bytes: &Bytes) -> Result<OtapArrowRecords, CodecError> {
        let mut logs = LogsData::decode(bytes.clone()).map_err(CodecError::invalid_input)?;
        for resource in &mut logs.resource_logs {
            resource.scope_logs.retain(|logs| {
                logs.scope
                    .as_ref()
                    .is_some_and(|scope| scope.name == self.0)
            });
        }
        encode_logs_otap_batch(&logs)
            .map_err(|e| CodecError::operation(&SCOPE_LOGS, CodecOperation::Decode, e))
    }
}

register_pdata_codec!(
    SCOPE_LOGS_CODEC,
    CodecRegistration::new(&METADATA)
        .with_default_for_encoding()
        .with_preparer(|config| {
            let scope = config
                .options
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("scope1");
            Ok(Arc::new(ScopeSelector(scope.to_owned())))
        })
        .with_decoder(|_| Box::new(ScopeSelector("scope1".to_owned())))
        .with_item_counter(|_, _| None),
);

register_pdata_codec!(
    FIXED_SCOPE_LOGS_CODEC,
    CodecRegistration::new(&METADATA)
        .with_implementation("test.fixed-scope")
        .with_cross_pipeline_forwarding()
        .with_decoder(|_| Box::new(ScopeSelector("scope1".to_owned())))
        .with_item_counter(|_, _| None),
);

static OTLP_LOGS: CodecMetadata = CodecMetadata::new(PdataEncoding::OTLP, &[SignalType::Logs]);
register_pdata_codec!(
    OUTPUT_ONLY_OTLP,
    CodecRegistration::new(&OTLP_LOGS)
        .with_implementation("test.output-only")
        .with_encoder(|_| panic!("topic publication must not instantiate an encoder")),
);

struct BusyDecoder;

impl PdataDecoder for BusyDecoder {
    fn decode(&mut self, _: SignalType, _: &Bytes) -> Result<OtapArrowRecords, CodecError> {
        Err(CodecError::ServiceBusy)
    }
}

static OTLP_TRACES: CodecMetadata = CodecMetadata::new(PdataEncoding::OTLP, &[SignalType::Traces]);
register_pdata_codec!(
    BUSY_OTLP_TRACES,
    CodecRegistration::new(&OTLP_TRACES)
        .with_implementation("test.busy-traces")
        .with_decoder(|_| Box::new(BusyDecoder))
        .with_item_counter(|_, _| None),
);

fn services_with_bindings(bindings: serde_json::Value) -> PipelineRuntimeServices {
    let config: CodecsConfig = serde_json::from_value(json!({"bindings": bindings}))
        .expect("valid test codec configuration");
    PipelineRuntimeServices::new(CodecsConfig::resolve([&config]))
        .expect("test codec bindings should resolve")
}

fn scope_logs(services: &PipelineRuntimeServices, bytes: Bytes) -> OtapPdata {
    let codec = services
        .codecs()
        .registry()
        .resolve(&SCOPE_LOGS)
        .expect("scope selector must be registered");
    let encoded = codec
        .admit(SignalType::Logs, bytes)
        .expect("scope selector must accept encoded logs");
    OtapPdata::new_default(encoded.into())
}

fn send_through_topic(
    input: OtapPdata,
    exporter_services: PipelineRuntimeServices,
    receiver_services: PipelineRuntimeServices,
    source_tagging: SourceTagging,
) -> (Result<OtapPdata, NackMsg<OtapPdata>>, TerminalState) {
    setup_test_runtime().block_on(async move {
        let broker = TopicBroker::<OtapPdata>::new();
        let topic_name = TopicName::parse("ingress").expect("topic name should parse");
        let handle = broker
            .create_in_memory_topic(
                topic_name.clone(),
                TopicOptions::Mixed {
                    balanced_capacity: 16,
                    broadcast_capacity: 16,
                    on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                    ack_mode: TopicBroadcastAckMode::First,
                },
            )
            .expect("topic should be created");

        let exporter_set = TopicSet::new("exporter-set");
        _ = exporter_set.insert(topic_name.clone(), handle.clone());
        let receiver_set = TopicSet::new("receiver-set");
        _ = receiver_set.insert(topic_name.clone(), handle);

        let (mut exporter_ctx, _) = test_pipeline_ctx_with_interests(
            Interests::NODE_ITEM_COUNTS | Interests::NODE_INPUT_METRICS,
        );
        exporter_ctx.set_topic_set(exporter_set);
        let mut receiver_ctx = create_test_pipeline_context();
        receiver_ctx.set_topic_set(receiver_set);

        let exporter_node = test_node("topic_exporter");
        let receiver_node = test_node("topic_receiver");

        let mut exporter_user_cfg = NodeUserConfig::new_exporter_config(TOPIC_EXPORTER_URN);
        exporter_user_cfg.config = json!({
            "topic": "ingress",
            "queue_on_full": "block"
        });
        let mut receiver_user_cfg = NodeUserConfig::new_receiver_config(TOPIC_RECEIVER_URN);
        receiver_user_cfg.config = json!({
            "topic": "ingress",
            "subscription": {
                "mode": "balanced",
                "group": "sut-workers"
            }
        });

        let mut exporter = (TOPIC_EXPORTER.create)(
            exporter_ctx,
            exporter_node.clone(),
            Arc::new(exporter_user_cfg),
            &ExporterConfig::new("topic_exporter"),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
        .expect("topic exporter should be created");

        let mut receiver = (TOPIC_RECEIVER.create)(
            receiver_ctx,
            receiver_node.clone(),
            Arc::new(receiver_user_cfg),
            &ReceiverConfig::new("topic_receiver"),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
        .expect("topic receiver should be created");

        receiver.set_source_tagging(source_tagging);

        let (exporter_input_tx, exporter_input_rx) = create_not_send_channel::<OtapPdata>(8);
        exporter
            .set_pdata_receiver(
                exporter_node.clone(),
                PDataReceiver::Local(LocalReceiver::mpsc(exporter_input_rx)),
            )
            .expect("exporter input channel should be wired");

        let (receiver_output_tx, receiver_output_rx) = create_not_send_channel::<OtapPdata>(8);
        receiver
            .set_pdata_sender(
                receiver_node.clone(),
                "".into(),
                PDataSender::Local(LocalSender::mpsc(receiver_output_tx)),
            )
            .expect("receiver output channel should be wired");

        let exporter_ctrl = exporter.control_sender();
        let receiver_ctrl = receiver.control_sender();
        let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel::<OtapPdata>(32);
        let (pipeline_completion_tx, mut pipeline_completion_rx) =
            pipeline_completion_msg_channel::<OtapPdata>(32);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
        let exporter_metrics = metrics_reporter.clone();
        let receiver_metrics = metrics_reporter.clone();
        let exporter_ctrl_tx = runtime_ctrl_tx.clone();
        let receiver_ctrl_tx = runtime_ctrl_tx.clone();
        let exporter_completion_tx = pipeline_completion_tx.clone();
        let receiver_completion_tx = pipeline_completion_tx.clone();
        let exporter_task = tokio::task::spawn_local(async move {
            exporter
                .start(
                    exporter_ctrl_tx,
                    exporter_completion_tx,
                    exporter_metrics,
                    Interests::empty(),
                    exporter_services,
                )
                .await
        });
        let receiver_task = tokio::task::spawn_local(async move {
            receiver
                .start(
                    receiver_ctrl_tx,
                    receiver_completion_tx,
                    receiver_metrics,
                    Interests::empty(),
                    receiver_services,
                )
                .await
        });

        exporter_input_tx.send(input).expect("send to topic exporter");
        let result = tokio::time::timeout(Duration::from_secs(30), async {
            tokio::select! {
                Ok(PipelineCompletionMsg::DeliverNack { nack }) = pipeline_completion_rx.recv() => Err(nack),
                delivery = receiver_output_rx.recv() => Ok(delivery.expect("receiver output should stay open")),
            }
        }).await.expect("timed out waiting for subscriber delivery or upstream NACK");

        let deadline = Instant::now() + Duration::from_secs(1);
        exporter_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("exporter shutdown should be sent");
        receiver_ctrl
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .await
            .expect("receiver shutdown should be sent");

        let exporter_result = exporter_task.await.expect("exporter task should join");
        let receiver_result = receiver_task.await.expect("receiver task should join");
        assert!(receiver_result.is_ok(), "receiver should stop cleanly");
        (result, exporter_result.expect("exporter should stop cleanly"))
    })
}

fn attempted_metric(terminal: &TerminalState, signal: &str, outcome: &str, name: &str) -> u64 {
    terminal
        .metrics()
        .iter()
        .filter(|snapshot| {
            snapshot.descriptor().name == "exporter.attempted"
                && snapshot.measurement_attribute_value("signal") == Some(signal)
                && snapshot.measurement_attribute_value("outcome") == Some(outcome)
        })
        .find_map(|snapshot| {
            snapshot
                .descriptor()
                .metrics
                .iter()
                .position(|metric| metric.name == name)
                .map(|index| snapshot.get_metrics()[index].to_u64_lossy())
        })
        .expect("missing exporter attempt metric")
}

/// Scenario: A publisher forwards syslog and OTLP logs, including with an encode-only OTLP binding and a traces-only alternative decoder.
/// Guarantees: The subscriber receives the original buffers in all cases, independent of encoder and unrelated signal registrations.
#[test]
fn topic_preserves_encoded_otlp_and_syslog() {
    let mut batch = SyslogBatchBuilder::new();
    batch
        .append_with_year(b"<13>Jan  1 00:00:00 host app: hello", 2026)
        .unwrap();
    let syslog = resolve_syslog()
        .unwrap()
        .admit_built(batch.finish(0).unwrap())
        .unwrap();
    for (input, publisher) in [
        (make_test_pdata(), test_pipeline_runtime_services()),
        (
            make_test_pdata(),
            services_with_bindings(json!({
                "otlp-bytes": {"implementation": "test.output-only"}
            })),
        ),
        (
            OtapPdata::new_default(syslog),
            test_pipeline_runtime_services(),
        ),
    ] {
        let original = input.payload_ref().encoded_bytes().unwrap().clone();
        let encoding = input.payload_ref().encoding().unwrap().clone();
        let (delivery, _) = send_through_topic(
            input.test_subscribe_to(Interests::NACKS, TestCallData::default().into(), 42),
            publisher,
            test_pipeline_runtime_services(),
            SourceTagging::Disabled,
        );
        let delivered = delivery.expect("eligible bytes must reach the subscriber");
        let encoded = delivered
            .payload_ref()
            .encoded_view()
            .expect("payload must stay encoded");
        assert_eq!(encoded.encoding(), &encoding);
        assert_eq!(encoded.bytes(), original.as_ref());
        assert_eq!(encoded.bytes().as_ptr(), original.as_ptr());
    }
}

/// Scenario: Publishers and subscribers select different scopes through either options or distinct decoder implementations.
/// Guarantees: Subscribers receive the publisher's selected record and the exporter counts it even when the wire count is unknown.
#[test]
fn topic_preserves_publisher_record_selection() {
    let second_scope = || {
        services_with_bindings(json!({
            "test-scope-logs": {"options": {"scope": "scope2"}}
        }))
    };
    let fixed_scope = services_with_bindings(json!({
        "test-scope-logs": {"implementation": "test.fixed-scope"}
    }));
    for (publisher, subscriber, selected_scope) in [
        (second_scope(), test_pipeline_runtime_services(), "scope2"),
        (fixed_scope, second_scope(), "scope1"),
    ] {
        let mut logs = logs_multiple_scopes_no_resource();
        let input = scope_logs(&publisher, logs.encode_to_vec().into());
        assert_eq!(input.payload_ref().known_item_count(), None);
        let (delivery, terminal) =
            send_through_topic(input, publisher, subscriber, SourceTagging::Disabled);
        logs.resource_logs[0]
            .scope_logs
            .retain(|scope_logs| scope_logs.scope.as_ref().unwrap().name == selected_scope);
        assert_eq!(
            delivery.unwrap().payload_ref().otap_ref(),
            Some(&encode_logs_otap_batch(&logs).unwrap()),
        );
        assert_eq!(attempted_metric(&terminal, "logs", "success", "items"), 1);
    }
}

/// Scenario: A publisher must materialize a malformed log request with an upstream NACK subscription.
/// Guarantees: A permanent client refusal returns the original call data and bytes and records one refused attempt.
#[test]
fn topic_materialization_failure_returns_original_delivery() {
    let services = test_pipeline_runtime_services();
    let bytes = Bytes::from_static(&[0xff]);
    let call_data = TestCallData::default();
    let input = scope_logs(&services, bytes.clone()).test_subscribe_to(
        Interests::NACKS | Interests::RETURN_DATA,
        call_data.clone().into(),
        42,
    );
    let (delivery, terminal) = send_through_topic(
        input,
        services,
        test_pipeline_runtime_services(),
        SourceTagging::Disabled,
    );
    let nack = delivery.expect_err("malformed input must be NACKed before publication");
    let (node, nack) = next_nack(nack).expect("original subscriber must receive the NACK");
    assert!(nack.permanent);
    assert_eq!(nack.cause, NackCause::Refused);
    assert_eq!(
        attempted_metric(&terminal, "logs", "refused", "messages"),
        1
    );
    assert_eq!(node, 42);
    assert_eq!(
        TestCallData::try_from(nack.unwind.route.calldata).unwrap(),
        call_data
    );
    let encoded = nack.refused.payload_ref().encoded_view().unwrap();
    assert_eq!(encoded.encoding(), &SCOPE_LOGS);
    assert_eq!(encoded.bytes(), bytes.as_ref());
}

/// Scenario: A traces publisher requires native records but its decoder is busy or its binding lacks a decoder.
/// Guarantees: Both errors count as server failures, and the busy codec returns a retryable NACK.
#[test]
fn topic_materialization_classifies_service_failures() {
    let bytes = Bytes::from(traces_with_full_resource_and_scope().encode_to_vec());
    for (implementation, permanent) in [("test.busy-traces", false), ("test.output-only", true)] {
        let publisher =
            services_with_bindings(json!({"otlp-bytes": {"implementation": implementation}}));
        let input =
            OtapPdata::new_default(OtlpProtoBytes::ExportTracesRequest(bytes.clone()).into())
                .test_subscribe_to(Interests::NACKS, TestCallData::default().into(), 42);
        let (delivery, terminal) = send_through_topic(
            input,
            publisher,
            test_pipeline_runtime_services(),
            SourceTagging::Disabled,
        );
        let nack = delivery.expect_err("materialization failed");
        assert_eq!(nack.cause, NackCause::Unspecified);
        assert_eq!(nack.permanent, permanent);
        assert_eq!(
            attempted_metric(&terminal, "traces", "failure", "messages"),
            1
        );
    }
}

/// Scenario: a topic receiver is configured to tag incoming pdata with its source.
/// Guarantees: received pdata contains the configured source tag.
#[test]
fn topic_receiver_applies_source_tag_when_enabled() {
    let (delivery, _) = send_through_topic(
        make_test_pdata(),
        test_pipeline_runtime_services(),
        test_pipeline_runtime_services(),
        SourceTagging::Enabled,
    );
    let delivered = delivery.unwrap();
    assert_eq!(
        delivered.get_source_node(),
        Some(test_node("topic_receiver").index)
    );
}
