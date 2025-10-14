// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ultra-minimal test utilities for OTAP components

use crate::pdata::{OtapPdata, OtlpProtoBytes};
use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
use otap_df_engine::{
    ExporterFactory, Interests,
    control::{CallData, PipelineControlMsg},
};
use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_rust::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
    resource::v1::Resource,
};
use prost::Message;
use serde_json::Value;
use std::ops::Add;
use std::time::Instant;

/// TestCallData helps test the CallData type.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TestCallData {
    id0: u64,
    id1: usize,
}

impl TestCallData {
    /// Create test calldata
    #[must_use]
    pub fn new_with(id0: u64, id1: usize) -> Self {
        Self { id0, id1 }
    }
}

impl Default for TestCallData {
    fn default() -> TestCallData {
        TestCallData::new_with(123, 4567)
    }
}

impl From<TestCallData> for CallData {
    fn from(value: TestCallData) -> Self {
        smallvec::smallvec![value.id0.into(), value.id1.into()]
    }
}

impl TryFrom<CallData> for TestCallData {
    type Error = otap_df_engine::error::Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Self::Error::InternalError {
                message: "invalid calldata".into(),
            });
        }
        Ok(Self {
            id0: value[0].into(),
            id1: value[1].try_into()?,
        })
    }
}

/// Create minimal test data
#[must_use]
pub fn create_test_logs() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest::new(vec![
        ResourceLogs::build(Resource::default())
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::default())
                    .log_records(vec![
                        LogRecord::build(2u64, SeverityNumber::Info, "event")
                            .attributes(vec![KeyValue::new("key", AnyValue::new_string("val"))])
                            .finish(),
                    ])
                    .finish(),
            ])
            .finish(),
    ])
}

/// Create minimal test pdata
#[must_use]
pub fn create_test_pdata() -> OtapPdata {
    let otlp_service_req = create_test_logs();
    let mut otlp_bytes = vec![];
    otlp_service_req.encode(&mut otlp_bytes).unwrap();

    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(otlp_bytes).into())
}

/// Simple exporter test where there is NO subscribe_to() in the context.
pub fn test_exporter_no_subscription(factory: &ExporterFactory<OtapPdata>, config: Value) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config).unwrap();

    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            ctx.send_pdata(create_test_pdata()).await.unwrap();
            ctx.send_shutdown(
                Instant::now().add(std::time::Duration::from_secs(1)),
                "test shutdown",
            )
            .await
            .unwrap();
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("success");

            let mut pipeline_rx = ctx.take_pipeline_ctrl_receiver().unwrap();

            match pipeline_rx.recv().await {
                Ok(received_msg) => {
                    panic!("expected no pipeline control messages, received: {received_msg:?}");
                }
                Err(err) => {
                    assert!(err.to_string().contains("channel is closed"));
                }
            }
        });
}

/// Simple exporter test where there is a subscribe_to() in the context.
pub fn test_exporter_with_subscription(
    factory: &ExporterFactory<OtapPdata>,
    config: Value,
    subscribe_interests: Interests,
    expect_interest: Interests,
) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config).unwrap();
    test_runtime
        .set_exporter(exporter)
        .run_test(move |ctx| async move {
            let req_data = create_test_pdata()
                .test_subscribe_to(subscribe_interests, TestCallData::default().into(), 654321);
            ctx.send_pdata(req_data).await.unwrap();
            ctx.send_shutdown(Instant::now().add(std::time::Duration::from_secs(1)), "test shutdown")
                .await
                .unwrap();
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("success");

            let mut pipeline_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
            let (trigger, calldata, reqdata, reason) = match pipeline_rx.recv().await {
                Ok(PipelineControlMsg::DeliverAck { ack, node_id }) => {
                    assert_eq!(node_id, 654321);
                    (Interests::ACKS, ack.calldata, Some(ack.accepted), "success".into())
                }
                Ok(PipelineControlMsg::DeliverNack { nack, node_id }) => {
                    assert_eq!(node_id, 654321);
                    (Interests::NACKS, nack.calldata, Some(nack.refused), nack.reason)
                }
                Ok(other) => (
                    Interests::empty(),
                    CallData::default(),
                    None,
                    format!("other message {other:?}"),
                ),
                Err(err) => (
                    Interests::empty(),
                    CallData::default(),
                    None,
                    format!("error {err:?}"),
                ),
            };
            assert_eq!(expect_interest&Interests::ACKS_OR_NACKS, trigger);

            if !trigger.is_empty() {
                let got: TestCallData = calldata.try_into().unwrap();
                assert_eq!(TestCallData::default(), got);
                assert_eq!(
                    reason,
                    if trigger == Interests::NACKS { "THIS specific error" } else { "success" },
                );

                assert_eq!(reqdata.expect("has payload").num_items(),
                           if (subscribe_interests & Interests::RETURN_DATA).is_empty() {
                               0
                           } else {
                               1
                           });

            } else {
                assert!(
                    reason.contains("Closed"),
                    "subscribed {subscribe_interests:?}: expecting {expect_interest:?}: trigger {trigger:?}: failed reason {reason}",
                );
                assert_eq!(calldata.len(), 0);
            }
        });
}
