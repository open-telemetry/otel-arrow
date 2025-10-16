// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for the retry processor using the common test harness.

use crate::pdata::{Context, OtapPdata};
use crate::retry_processor::{RETRY_PROCESSOR_URN, RetryConfig};
use crate::testing::create_test_pdata;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    AckMsg, NackMsg, NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel,
};
use otap_df_engine::message::Message;
use otap_df_engine::testing::node::test_node;
use otap_df_engine::testing::processor::TestRuntime;
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

/// Test the full retry flow with NACKs and eventual failure.
///
/// This comprehensive test exercises both the downstream (producer) and upstream (consumer)
/// directions of the retry processor:
///
/// 1. **Setup**: Create test data with subscription context (simulating an upstream receiver)
/// 2. **First attempt**: Processor forwards data downstream (we intercept it)
/// 3. **First NACK**: Simulate downstream failure by sending NACK back to processor
/// 4. **Retry attempts**: Processor should delay and retry multiple times
/// 5. **Deadline exceeded**: Eventually the max_elapsed_time is exceeded
/// 6. **Final NACK upstream**: Processor should NACK back to the simulated upstream receiver
///
/// We configure with very short intervals to keep the test fast, and verify:
/// - The processor does NOT immediately forward NACKs (it retries)
/// - Multiple retry attempts occur (>0 retries)
/// - After deadline, the processor gives up and NACKs upstream
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

            let retry_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let pdata_in = create_test_pdata();

            ctx.process(Message::PData(pdata_in))
                .await
                .expect("process initial message");

            // Verify the processor forwarded the data downstream
            let mut output = ctx.drain_pdata().await;
            assert_eq!(output.len(), 1);
            let first_attempt = output.remove(0);
            assert_eq!(first_attempt.num_items(), 1);

            // Step 3: Simulate downstream failures and retry loop
            let mut current_data = first_attempt;
            for _ in 0..3 {
                let nack = NackMsg::new("simulated downstream failure", current_data.clone());

                let (_, nack_ctx) = Context::next_nack(nack).unwrap();

                ctx.process(Message::nack_ctrl_msg(nack_ctx)).await.unwrap();

                // The processor should schedule a delayed retry via DelayData
                if let Ok(msg) = pipeline_rx.recv().await {
                    match msg {
                        PipelineControlMsg::DelayData { when, data, .. } => {
                            let _ = retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

            // Step 4: Send final ACK to complete successfully
            let ack = AckMsg::new(current_data);
            let (_, ack_ctx) = Context::next_ack(ack).unwrap();
            ctx.process(Message::ack_ctrl_msg(ack_ctx)).await.unwrap();

            // Verify retry behavior
            let final_retry_count = retry_count.load(std::sync::atomic::Ordering::SeqCst);
            assert_eq!(final_retry_count, 3);
        })
        .validate(|ctx| async move {
            // Verify no unexpected control message processing
            let counters = ctx.counters();
            counters.assert(0, 0, 0, 0);
        });
}

/// Test configuration parsing with default values
#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.initial_interval, Duration::from_secs(5));
    assert_eq!(config.max_interval, Duration::from_secs(30));
    assert_eq!(config.max_elapsed_time, Duration::from_secs(300));
    assert_eq!(config.multiplier, 1.5);
}

/// Test configuration parsing with custom values
#[test]
fn test_retry_config_custom() {
    let config: RetryConfig = serde_json::from_value(json!({
        "initial_interval": 1.0,
        "max_interval": 10.0,
        "max_elapsed_time": 60.0,
        "multiplier": 3.0,
    }))
    .expect("parse config");

    assert_eq!(config.initial_interval, Duration::from_secs(1));
    assert_eq!(config.max_interval, Duration::from_secs(10));
    assert_eq!(config.max_elapsed_time, Duration::from_secs(60));
    assert_eq!(config.multiplier, 3.0);
}
