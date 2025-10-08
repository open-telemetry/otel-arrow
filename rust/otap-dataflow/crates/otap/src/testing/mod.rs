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
            let mut test_data = empty_logs_pdata();
            test_data.test_subscribe_to(Interests::ACKS | Interests::NACKS, CallData::new(), 1);
            ctx.send_pdata(test_data)
                .await
                .expect("Failed to send pdata");
            ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                .await
                .expect("Failed to send shutdown");
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("Exporter should succeed when subscription exists");

            // Verify that a DeliverAck message was sent
            let mut pipeline_rx = ctx
                .take_pipeline_ctrl_receiver()
                .expect("pipeline ctrl receiver should be available");

            let msg = tokio::time::timeout(std::time::Duration::from_millis(500), async {
                pipeline_rx.recv().await
            })
            .await
            .expect("timed out waiting for DeliverAck")
            .expect("pipeline ctrl channel closed");

            match msg {
                PipelineControlMsg::DeliverAck { .. } => {
                    // Success - this is what we expect
                }
                other => panic!("Expected DeliverAck, got {other:?}"),
            }
        });
}
