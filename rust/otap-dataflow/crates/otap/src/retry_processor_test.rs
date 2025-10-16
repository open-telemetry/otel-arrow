// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for the retry processor using the common test harness.

use crate::pdata::{Context, OtapPdata};
use crate::retry_processor::RETRY_PROCESSOR_URN;
use crate::testing::{TestCallData, create_test_pdata};
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    AckMsg, NackMsg, NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel,
};
use otap_df_engine::testing::node::test_node;
use otap_df_engine::testing::processor::TestRuntime;
use otap_df_engine::{Interests, message::Message};
use otap_df_telemetry::registry::MetricsRegistryHandle;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

/// Creates a test pipeline context for testing
fn create_test_pipeline_context() -> PipelineContext {
    let metrics_registry = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry);
    controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 0)
}

/// Test the full retry flow with NACKs and eventual success.
#[test]
fn test_retry_processor_full_flow_with_retries() {
    let pipeline_ctx = create_test_pipeline_context();
    let node = test_node("retry-processor-full-test");
    let rt: TestRuntime<OtapPdata> = TestRuntime::new();

    // Configure with very short intervals for fast testing
    let config = json!({
        "initial_interval": 0.05,    // 50ms initial delay
        "max_interval": 0.15,         // 150ms max delay
        "max_elapsed_time": 0.3,      // 300ms total timeout
        "multiplier": 2.0,            // Double each retry
    });

    let mut node_config = NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN);
    node_config.config = config;

    let proc = crate::retry_processor::create_retry_processor(
        pipeline_ctx,
        node,
        Arc::new(node_config),
        rt.config(),
    )
    .expect("create processor");

    let phase = rt.set_processor(proc);

    phase
        .run_test(|mut ctx| async move {
            // Set up pipeline control channel to capture ACK/NACK messages sent upstream
            let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
            ctx.set_pipeline_ctrl_sender(pipeline_tx);

            let mut retry_count: usize = 0;
            let pdata_in = create_test_pdata().test_subscribe_to(
                Interests::ACKS | Interests::RETURN_DATA,
                TestCallData::default().into(),
                4444,
            );

            ctx.process(Message::PData(pdata_in))
                .await
                .expect("process initial message");

            // Verify the processor forwarded the data downstream
            let mut output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            let first_attempt = output.remove(0);
            assert_eq!(first_attempt.num_items(), 1);

            // Simulate downstream failures and retry
            let mut current_data = first_attempt;
            for _ in 0..3 {
                let nack = NackMsg::new("simulated downstream failure", current_data.clone());

                let (_, nack_ctx) = Context::next_nack(nack).unwrap();

                ctx.process(Message::nack_ctrl_msg(nack_ctx)).await.unwrap();

                // The processor should schedule a delayed retry via DelayData
                if let Ok(msg) = pipeline_rx.recv().await {
                    match msg {
                        PipelineControlMsg::DelayData { when, data, .. } => {
                            retry_count += 1;
                            // Deliver immediately (0 delay for test)
                            ctx.process(Message::Control(NodeControlMsg::DelayedData {
                                when,
                                data,
                            }))
                            .await
                            .unwrap();

                            // The retry was sent downstream
                            let mut retry_output = ctx.drain_pdata().await;
                            assert_eq!(retry_output.len(), 1);
                            current_data = retry_output.remove(0);
                        }
                        other => {
                            panic!("unexpected pipeline control message: {:?}", other);
                        }
                    }
                } else {
                    panic!("expected DelayData message from retry processor");
                }
            }

            // Send final ACK to complete successfully, emulating success:
            let ack = AckMsg::new(current_data);
            let (_, ack_ctx) = Context::next_ack(ack).unwrap();
            ctx.process(Message::ack_ctrl_msg(ack_ctx)).await.unwrap();

            // Verify the processor sent the ACK upstream
            let msg = tokio::time::timeout(Duration::from_secs(1), pipeline_rx.recv())
                .await
                .expect("timeout waiting for final DeliverAck")
                .expect("channel closed");

            match msg {
                PipelineControlMsg::DeliverAck { node_id, ack } => {
                    assert_eq!(node_id, 4444);
                    let ackdata: TestCallData = ack.calldata.try_into().expect("my calldata");
                    assert_eq!(TestCallData::default(), ackdata);

                    // Requested RETURN_DATA, check item count match
                    assert_eq!(create_test_pdata().num_items(), ack.accepted.num_items());
                }
                other => {
                    panic!("expected DeliverAck but got: {:?}", other);
                }
            }

            assert_eq!(retry_count, 3);
        })
        .validate(|ctx| async move {
            // Verify no unexpected control message processing
            let counters = ctx.counters();
            counters.assert(0, 0, 0, 0);
        });
}
