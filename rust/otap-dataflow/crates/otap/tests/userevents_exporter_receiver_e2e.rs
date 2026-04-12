// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux-only end-to-end smoke test for:
//! `opentelemetry-user-events-logs` -> kernel `user_events` -> `receiver:userevents`.

#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use opentelemetry::logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::LoggerProviderBuilder;
use opentelemetry_user_events_logs::Processor;
use otap_df_config::node::NodeUserConfig;
use otap_df_core_nodes::receivers::userevents_receiver::{
    USEREVENTS_RECEIVER, USEREVENTS_RECEIVER_URN,
};
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::testing::receiver::{NotSendValidateContext, TestRuntime};
use otap_df_engine::testing::test_node;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use prost::Message;
use serde_json::json;

fn otap_logs_to_otlp(mut pdata: OtapPdata) -> ExportLogsServiceRequest {
    let payload = pdata.take_payload();
    let mut records: OtapArrowRecords = payload.try_into().expect("otap arrow payload");
    let mut encoder = LogsProtoBytesEncoder::new();
    let mut buffer = ProtoBuffer::new();
    encoder.encode(&mut records, &mut buffer).expect("encode logs");
    ExportLogsServiceRequest::decode(buffer.as_ref()).expect("decode export logs request")
}

fn log_attributes_as_strings(
    request: &ExportLogsServiceRequest,
) -> HashMap<String, String> {
    let log_record = &request.resource_logs[0].scope_logs[0].log_records[0];
    log_record
        .attributes
        .iter()
        .filter_map(|kv| {
            let value = kv.value.as_ref()?;
            let inner = value.value.as_ref()?;
            let rendered = match inner {
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(v) => {
                    v.clone()
                }
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::IntValue(v) => {
                    v.to_string()
                }
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::BoolValue(v) => {
                    v.to_string()
                }
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::DoubleValue(v) => {
                    v.to_string()
                }
                _ => return None,
            };
            Some((kv.key.clone(), rendered))
        })
        .collect()
}

fn scenario() -> impl FnOnce(
    otap_df_engine::testing::receiver::TestContext<OtapPdata>,
) -> Pin<Box<dyn Future<Output = ()>>> {
    |ctx| {
        Box::pin(async move {
            ctx.sleep(Duration::from_millis(200)).await;

            let processor = Processor::builder("myprovider")
                .build()
                .expect("create user_events processor");
            let provider = LoggerProviderBuilder::default()
                .with_resource(Resource::builder().with_service_name("myrolename").build())
                .with_log_processor(processor)
                .build();
            let logger = provider.logger("userevents-e2e");

            let mut record = logger.create_log_record();
            record.set_event_name("my-event-name");
            record.set_severity_number(Severity::Error);
            record.set_body(AnyValue::from("This is a test message"));
            record.add_attribute("user_name", "otel user");
            logger.emit(record);

            ctx.sleep(Duration::from_millis(800)).await;
            ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                .await
                .expect("shutdown receiver");
            drop(provider);
        })
    }
}

fn validation() -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    |mut ctx| {
        Box::pin(async move {
            let received = ctx.recv().await.expect("receiver output");
            let export_request = otap_logs_to_otlp(received);
            let log_record = &export_request.resource_logs[0].scope_logs[0].log_records[0];
            let attrs = log_attributes_as_strings(&export_request);

            assert_eq!(log_record.severity_number, 17);
            assert_eq!(log_record.severity_text, "ERROR");
            let body = log_record
                .body
                .as_ref()
                .and_then(|value| value.value.as_ref())
                .and_then(|inner| match inner {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(text) => Some(text.as_str()),
                    _ => None,
                })
                .expect("string log body");
            assert_eq!(body, "This is a test message");
            assert!(attrs
                .get("linux.userevents.tracepoint")
                .expect("tracepoint attribute")
                .contains("user_events:myprovider_L2K1"));
            assert_eq!(attrs.get("event.provider").map(String::as_str), Some("myprovider"));
            assert_eq!(attrs.get("event.name").map(String::as_str), Some("my-event-name"));
            assert_eq!(attrs.get("cs.part_b.name").map(String::as_str), Some("my-event-name"));
            assert_eq!(
                attrs.get("cs.part_b.body").map(String::as_str),
                Some("This is a test message")
            );
            assert!(!attrs
                .get("linux.userevents.decode.mode")
                .expect("decode mode")
                .is_empty());
            assert_eq!(attrs.get("user_name").map(String::as_str), Some("otel user"));
        })
    }
}

#[ignore = "requires Linux kernel user_events support and write permissions to tracing/user_events_data"]
#[test]
fn exporter_to_userevents_receiver_smoke_test() {
    let test_runtime = TestRuntime::<OtapPdata>::new();

    let mut node_config = NodeUserConfig::new_receiver_config(USEREVENTS_RECEIVER_URN);
    node_config.config = json!({
        "tracepoint": "user_events:myprovider_L2K1",
        "format": { "type": "common_schema_otel_logs" },
        "session": {
            "late_registration": {
                "enabled": true,
                "poll_interval_ms": 100
            }
        },
        "batching": {
            "max_size": 1,
            "max_duration": "10ms"
        }
    });

    let telemetry_registry_handle = TelemetryRegistryHandle::new();
    let controller_ctx = ControllerContext::new(telemetry_registry_handle);
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

    let receiver = (USEREVENTS_RECEIVER.create)(
        pipeline_ctx,
        test_node("userevents_receiver"),
        Arc::new(node_config),
        &ReceiverConfig::new("userevents_receiver"),
    )
    .expect("create userevents receiver");

    test_runtime
        .set_receiver(receiver)
        .run_test(scenario())
        .run_validation(validation());
}
