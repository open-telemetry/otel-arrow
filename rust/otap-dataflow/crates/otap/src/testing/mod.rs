// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ultra-minimal test utilities for OTAP components

use crate::pdata::OtapPdata;
use bytes::Bytes;
use otap_df_engine::control::{AckMsg, NackMsg, UnwindData, nanos_since_birth};
use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
use otap_df_engine::{
    ExporterFactory, Interests,
    control::{CallData, PipelineResultMsg},
};
use otap_df_pdata::OtlpProtoBytes;
use prost::Message;
use serde_json::Value;
use std::ops::Add;
use std::time::Instant;

/// Consume frames to locate the most recent subscriber with ACKS
/// interest in test scenarios, simulating the runtime control manager.
#[must_use]
pub fn next_ack(mut ack: AckMsg<OtapPdata>) -> Option<(usize, AckMsg<OtapPdata>)> {
    let frame = ack
        .accepted
        .context_mut()
        .drain_to_next_subscriber(Interests::ACKS);
    frame.map(|frame| {
        if (frame.interests & Interests::RETURN_DATA).is_empty() {
            let _drop = ack.accepted.take_payload();
        }
        ack.unwind = UnwindData::new(frame.route, nanos_since_birth());
        (frame.node_id, ack)
    })
}

/// Consume frames to locate the most recent subscriber with NACKS
/// interest in test scenarios, simulating the runtime control manager.
#[must_use]
pub fn next_nack(mut nack: NackMsg<OtapPdata>) -> Option<(usize, NackMsg<OtapPdata>)> {
    let frame = nack
        .refused
        .context_mut()
        .drain_to_next_subscriber(Interests::NACKS);
    frame.map(|frame| {
        if (frame.interests & Interests::RETURN_DATA).is_empty() {
            let _drop = nack.refused.take_payload();
        }
        nack.unwind = UnwindData::new(frame.route, nanos_since_birth());
        (frame.node_id, nack)
    })
}

/// TestCallData helps test the RouteData type.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
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

/// Create minimal test pdata
#[must_use]
pub fn create_test_pdata() -> OtapPdata {
    // Note this has to be one log record for existing tests.
    let otlp_service_req = otap_df_pdata::testing::fixtures::log_with_no_scope();
    let mut otlp_bytes = vec![];
    otlp_service_req
        .encode(&mut otlp_bytes)
        .expect("failed to encode test OTLP ExportLogsServiceRequest");

    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::from(otlp_bytes)).into())
}

/// Simple exporter test where there is NO subscribe_to() in the context.
pub fn test_exporter_no_subscription(factory: &ExporterFactory<OtapPdata>, config: Value) {
    let test_runtime = TestRuntime::new();
    let exporter = create_exporter_from_factory(factory, config)
        .expect("failed to create exporter from factory in no-subscription test");

    test_runtime
        .set_exporter(exporter)
        .run_test(|ctx| async move {
            ctx.send_pdata(create_test_pdata())
                .await
                .expect("failed to send test pdata in no-subscription test");
            ctx.send_shutdown(
                Instant::now().add(std::time::Duration::from_secs(1)),
                "test shutdown",
            )
            .await
            .expect("failed to send shutdown in no-subscription test");
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("success");

            let mut pipeline_rx = ctx
                .take_pipeline_result_receiver()
                .expect("pipeline result receiver should be present in no-subscription test");

            match pipeline_rx.recv().await {
                Ok(received_msg) => {
                    panic!("expected no runtime control messages, received: {received_msg:?}");
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
    let exporter = create_exporter_from_factory(factory, config)
        .expect("failed to create exporter from factory in subscription test");
    test_runtime
        .set_exporter(exporter)
        .run_test(move |ctx| async move {
            let req_data = create_test_pdata()
                .test_subscribe_to(subscribe_interests, TestCallData::default().into(), 654321);
            ctx.send_pdata(req_data)
                .await
                .expect("failed to send subscribed test pdata");
            ctx.send_shutdown(Instant::now().add(std::time::Duration::from_secs(1)), "test shutdown")
                .await
                .expect("failed to send shutdown in subscription test");
        })
        .run_validation(|mut ctx, result| async move {
            result.expect("success");

            let mut pipeline_rx = ctx
                .take_pipeline_result_receiver()
                .expect("pipeline result receiver should be present in subscription test");
            // Loop to skip acks/nacks that have no matching subscriber
            // (e.g., exporter sends ack but subscription is NACKS-only).
            // In the real controller, unwind_ack would consume these silently.
            let (trigger, calldata, reqdata, reason) = loop {
                match pipeline_rx.recv().await {
                    Ok(PipelineResultMsg::DeliverAck { ack }) => {
                        if let Some((node_id, ack)) = next_ack(ack) {
                            assert_eq!(node_id, 654321);
                            break (Interests::ACKS, ack.unwind.route.calldata, Some(ack.accepted), "success".into());
                        }
                        // No ACKS subscriber — skip, like the controller would.
                    }
                    Ok(PipelineResultMsg::DeliverNack { nack }) => {
                        if let Some((node_id, nack)) = next_nack(nack) {
                            assert_eq!(node_id, 654321);
                            break (Interests::NACKS, nack.unwind.route.calldata, Some(nack.refused), nack.reason);
                        }
                        // No NACKS subscriber — skip, like the controller would.
                    }
                    Err(err) => break (
                        Interests::empty(),
                        CallData::default(),
                        None,
                        format!("error {err:?}"),
                    ),
                }
            };
            assert_eq!(expect_interest&Interests::ACKS_OR_NACKS, trigger);

            if !trigger.is_empty() {
                let got: TestCallData = calldata
                    .try_into()
                    .expect("failed to parse TestCallData from ack/nack calldata");
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
