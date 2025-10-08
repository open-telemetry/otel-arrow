// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ultra-minimal test utilities for OTAP components

use crate::pdata::{OtapPayload, OtapPdata, OtlpProtoBytes};
use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
use otap_df_engine::{ExporterFactory, Interests, control::CallData};
use serde_json::Value;

/// TestCallData helps test the CallData type.
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TestCallData {
    id0: u64,
    id1: usize,
}

impl TestCallData {
    /// Create test calldata
    pub fn new_with(id0: u64, id1: usize) -> Self {
        Self { id0, id1 }
    }

    /// Create a standard test calldata
    pub fn new() -> TestCallData {
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
    let exporter = create_exporter_from_factory(factory, config)
        .expect("Failed to create exporter from factory");

    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            ctx.send_pdata(empty_logs_pdata())
                .await
                .expect("Failed to send pdata");
            ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                .await
                .expect("Failed to send shutdown");
        })
        .run_validation(|_ctx, result| async move {
            result.expect("Exporter should succeed with no subscription");
        });
}

/// Simple exporter test where there is a subscribe_to() in the context.
pub fn test_exporter_with_subscription(factory: &ExporterFactory<OtapPdata>, config: Value) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config)
        .expect("Failed to create exporter from factory");

    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            let mut test_data = empty_logs_pdata();
            test_data.test_subscribe_to(Interests::ACKS | Interests::NACKS, CallData::new(), 1);
            ctx.send_pdata(test_data)
                .await
                .expect("Failed to send pdata");
            ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                .await
                .expect("Failed to send shutdown");
        })
        .run_validation(|_ctx, result| async move {
            // The EffectHandlerCore::route_{ack,nack} implementations return a fixed
            // SendError::Closed as a placeholder because there is no routing implementation.
            let err = result.expect_err(
                "Exporter should fail when subscription exists but notify_* not implemented",
            );
            let err_str = format!("{:?}", err);
            assert!(
                err_str.contains("NodeControlMsgSendError")
                    && err_str.contains("Channel is closed"),
                "Expected NodeControlMsgSendError with closed channel, got: {:?}",
                err
            );
        });
}
