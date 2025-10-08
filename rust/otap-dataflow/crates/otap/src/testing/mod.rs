// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ultra-minimal test utilities for OTAP components

use crate::pdata::{OtapPayload, OtapPdata, OtlpProtoBytes};
use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
use otap_df_engine::{
    ExporterFactory, Interests,
    control::{CallData, PipelineControlMsg},
};
use serde_json::Value;

/// TestCallData helps test the CallData type.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TestCallData {
    id0: u64,
    id1: usize,
}

impl TestCallData {
    /// Create test calldata
    pub fn new(id0: u64, id1: usize) -> Self {
        Self { id0, id1 }
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

/// Create a standard test calldata
fn create_test_data() -> TestCallData {
    TestCallData::new(123, 4567)
}

/// Create minimal empty test data
#[must_use]
pub fn empty_logs_pdata() -> OtapPdata {
    OtapPdata::new_todo_context(OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
        vec![],
    )))
}

/// Simple exporter test where there is NO subscribe_to() in the context.
pub fn test_exporter_no_subscription(factory: &ExporterFactory<OtapPdata>, config: Value) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config).unwrap();

    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            ctx.send_pdata(empty_logs_pdata()).await.unwrap();
            ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
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
pub fn test_exporter_with_subscription(factory: &ExporterFactory<OtapPdata>, config: Value) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config).unwrap();
    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            let mut req_data = empty_logs_pdata();

            req_data.test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                create_test_data().into(),
                654321,
            );
            ctx.send_pdata(req_data).await.unwrap();
            ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                .await
                .unwrap();
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("success");

            let mut pipeline_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
            match pipeline_rx.recv().await {
                Ok(PipelineControlMsg::DeliverAck { ack, node_id }) => {
                    assert_eq!(node_id, 654321);
                    let got_data: TestCallData = ack.calldata.try_into().unwrap();
                    assert_eq!(create_test_data(), got_data);
                }
                other => panic!("expected DeliverAck, got {other:?}"),
            }
        });
}
