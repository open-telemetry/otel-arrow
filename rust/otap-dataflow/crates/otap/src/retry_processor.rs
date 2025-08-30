// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Retry Processor with ACK/NACK Feedback Loop
//!
//! The retry processor implements reliable message delivery through an ACK/NACK feedback system.
//! Messages are tracked with unique IDs and retried on failure using exponential backoff.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐   ACK/NACK   ┌─────────────┐   ACK/NACK   ┌─────────────┐
//! │  Upstream   │◄─────────────│    Retry    │◄─────────────│ Downstream  │
//! │ Component   │              │  Processor  │              │ Component   │
//! └─────────────┘              └─────────────┘              └─────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Reliable Delivery**: Messages are tracked until acknowledged
//! - **Exponential Backoff**: Failed messages are retried with increasing delays
//! - **Backpressure**: When queue is full, sends NACK upstream instead of dropping messages
//! - **Automatic Cleanup**: Expired messages are periodically removed
//!
//! ## ACK/NACK Behavior
//!
//! - **ACK**: Remove message from pending retry queue (successful processing)
//! - **NACK**: Schedule message for retry with exponential backoff
//! - **Queue Full**: Send NACK upstream with ID=0 to signal backpressure
//!
//! ## Configuration
//!
//! The processor is configured via [`retry_processor::RetryConfig`] with parameters for:
//! - Maximum retry attempts
//! - Initial and maximum retry delays
//! - Backoff multiplier
//! - Queue capacity limits
//!
//! ## Example
//!
//! ```rust
//! use otap_df_otap::retry_processor::{RetryConfig, RetryProcessor};
//!
//! let config = RetryConfig {
//!     max_retries: 3,
//!     initial_retry_delay_ms: 1000,
//!     max_retry_delay_ms: 30000,
//!     backoff_multiplier: 2.0,
//! };
//! let processor = RetryProcessor::with_config(config);
//! ```

use crate::pdata::{OtapPdata, ReplyState};

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    ProcessorFactory,
    config::ProcessorConfig,
    control::{AckOrNack, NodeControlMsg},
    error::Error,
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

/// URN for the RetryProcessor processor
pub const RETRY_PROCESSOR_URN: &str = "urn:otap:processor:retry_processor";

/// Configuration for the retry processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts before dropping a message
    pub max_retries: usize,
    /// Initial delay in milliseconds before the first retry
    pub initial_retry_delay_ms: u64,
    /// Maximum delay in milliseconds between retries
    pub max_retry_delay_ms: u64,
    /// Multiplier applied to delay for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// OTAP RetryProcessor
#[allow(unsafe_code)]
#[distributed_slice(crate::OTAP_PROCESSOR_FACTORIES)]
pub static RETRY_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: RETRY_PROCESSOR_URN,
    create: create_retry_processor,
};

/// A processor that handles message retries with exponential backoff
///
/// The RetryProcessor maintains a queue of messages that have failed processing
/// and retries them according to the configured retry policy. It tracks each
/// message with a unique ID and implements exponential backoff for retry delays.
/// Register SignalTypeRouter as an OTAP processor factory
pub struct RetryProcessor {
    config: RetryConfig,
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_retry_processor(
    _pipeline_ctx: PipelineContext,
    node: NodeId,
    config: &Value,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    // Deserialize the (currently empty) router configuration
    let config: RetryConfig =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse retry configuration: {e}"),
        })?;

    // Create the retry processor
    let retry = RetryProcessor::with_config(config);

    // Create NodeUserConfig and wrap as local processor
    let user_config = Arc::new(NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN));

    Ok(ProcessorWrapper::local(
        retry,
        node,
        user_config,
        processor_config,
    ))
}

impl RetryProcessor {
    /// Creates a new RetryProcessor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Creates a new RetryProcessor with the specified configuration
    #[must_use]
    pub fn with_config(config: RetryConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
struct RetryState {
    retries: usize,   // r0
    last_ts: Instant, // r1
}

impl From<RetryState> for ReplyState {
    fn from(value: RetryState) -> Self {
        Self::new(value.retries.into(), value.last_ts.into())
    }
}

impl TryFrom<ReplyState> for RetryState {
    type Error = crate::pdata::error::Error;

    fn try_from(value: ReplyState) -> Result<Self, Self::Error> {
        Ok(Self {
            retries: value.r0.try_into()?,
            last_ts: value.r1.try_into()?,
        })
    }
}

#[async_trait(?Send)]
impl Processor<OtapPdata> for RetryProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::PData(mut data) => {
                let rstate = RetryState {
                    retries: 0,
                    last_ts: Instant::now(),
                };
                data.mut_context()
                    .reply_to(effect_handler.processor_id().index(), rstate.into());

                effect_handler.send_message(data).await?;
                Ok(())
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Ack(mut ack) => {
                    _ = ack.context.reply_to.pop().expect("has_rsvp");
                    let node_id = ack.context.reply_node_id();
                    effect_handler.reply(node_id, AckOrNack::Ack(ack)).await?;
                    Ok(())
                }
                NodeControlMsg::Nack(mut nack) => {
                    let mut rstate: RetryState = nack
                        .context
                        .reply_to
                        .pop()
                        .expect("has_reply")
                        .state
                        .try_into()?;

                    let node_id = nack.context.reply_node_id();

                    if rstate.retries >= self.config.max_retries || nack.payload.is_none() {
                        effect_handler.reply(node_id, AckOrNack::Nack(nack)).await?;
                        return Ok(());
                    }
                    // TODO: new Nack messages explaining what/why, instead of direct
                    // propagation as below?
                    //
                    // TODO: compute sleep; the check below should
                    // really test whether we expire before the
                    // backoff.

                    let expired = nack
                        .context
                        .deadline
                        .map(|dead| Instant::now().duration_since(dead).is_zero())
                        .unwrap_or(false);

                    if expired {
                        effect_handler.reply(node_id, AckOrNack::Nack(nack)).await?;
                        return Ok(());
                    }

                    // Increment the retry count
                    rstate.retries += 1;

                    nack.context
                        .reply_to(effect_handler.processor_id().index(), rstate.into());

                    effect_handler
                        .send_message(OtapPdata::new(
                            nack.context,
                            *nack.payload.expect("checked"),
                        ))
                        .await?;

                    Ok(())
                }
                NodeControlMsg::TimerTick { .. } => {
                    // Nothing
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry { .. } => {
                    // Retry processor has no telemetry collection to perform here.
                    Ok(())
                }
                NodeControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                NodeControlMsg::Shutdown { .. } => {
                    // Nothing
                    Ok(())
                }
            },
        }
    }
}

impl Default for RetryProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{SimpleDataGenOptions, create_simple_logs_arrow_record_batches};
    use crate::grpc::OtapArrowBytes;
    use crate::pdata::Context;
    use otap_df_channel::mpsc;
    use otap_df_config::experimental::SignalType;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::time::{Duration, sleep};

    fn create_test_channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::Channel::new(capacity)
    }

    fn create_test_processor() -> RetryProcessor {
        let config = RetryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 1000,
            backoff_multiplier: 2.0,
        };
        RetryProcessor::with_config(config)
    }

    fn create_test_data(_id: u64) -> OtapPdata {
        OtapPdata::OtapArrowBytes {
            context: Context::new(None),
            value: OtapArrowBytes::ArrowLogs(create_simple_logs_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows: 1,
                    ..Default::default()
                },
            )),
        }
    }

    /// num_rows is a placeholder for maybe a testing helper library for OTAP pdata?
    fn num_rows(pdata: &OtapPdata) -> usize {
        match pdata.signal_type() {
            SignalType::Logs => {
                let records: otel_arrow_rust::otap::OtapArrowRecords =
                    pdata.clone().try_into().map(|(_, v)| v).unwrap();
                records
                    .get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs)
                    .map_or(0, |batch| batch.num_rows())
            }
            SignalType::Traces => {
                let records: otel_arrow_rust::otap::OtapArrowRecords =
                    pdata.clone().try_into().map(|(_, v)| v).unwrap();
                records
                    .get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::Spans)
                    .map_or(0, |batch| batch.num_rows())
            }
            SignalType::Metrics => {
                let records: otel_arrow_rust::otap::OtapArrowRecords =
                    pdata.clone().try_into().map(|(_, v)| v).unwrap();
                records.get(otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType::UnivariateMetrics)
                    .map_or(0, |batch| batch.num_rows())
            }
        }
    }

    /// Test helper to compare two OtapPdata instances for equivalence.
    fn requests_match(expected: &OtapPdata, actual: &OtapPdata) -> bool {
        // ToDo: Implement full semantic equivalence checking similar to Go's assert.Equiv()
        num_rows(expected) == num_rows(actual)
    }

    #[test]
    fn test_factory_creation() {
        let config = json!({
            "max_retries": 5,
            "initial_retry_delay_ms": 500,
            "max_retry_delay_ms": 15000,
            "backoff_multiplier": 1.5,
            "max_pending_messages": 5000,
            "cleanup_interval_secs": 30
        });
        let processor_config = ProcessorConfig::new("test_retry");

        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let result = create_retry_processor(
            pipeline_ctx,
            test_node(processor_config.name.clone()),
            &config,
            &processor_config,
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_pdata_message() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        let test_data = create_test_data(1);
        let message = Message::PData(test_data.clone());

        processor
            .process(message, &mut effect_handler)
            .await
            .unwrap();

        // Should have one pending message
        assert_eq!(processor.pending_messages.len(), 1);

        // Should have sent message downstream
        let received = receiver.recv().await.unwrap();
        assert!(requests_match(&test_data, &received));
    }

    #[tokio::test]
    async fn test_ack_removes_pending() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add a message
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data.clone()), &mut effect_handler)
            .await
            .unwrap();
        assert_eq!(processor.pending_messages.len(), 1);

        // Consume the downstream message
        let _ = receiver.recv().await.unwrap();

        // ACK the message
        processor
            .process(
                Message::Control(NodeControlMsg::Ack { id: 1 }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Should be removed from pending
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_nack_schedules_retry() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add a message
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data.clone()), &mut effect_handler)
            .await
            .unwrap();
        let _ = receiver.recv().await.unwrap();

        // NACK the message
        processor
            .process(
                Message::Control(NodeControlMsg::Nack {
                    id: 1,
                    reason: "Test failure".to_string(),
                    pdata: None,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Should still have one pending message with incremented retry count
        assert_eq!(processor.pending_messages.len(), 1);
        let pending = processor.pending_messages.get(&1).unwrap();
        assert_eq!(pending.retry_count, 1);
        assert_eq!(pending.last_error, "Test failure");
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add a message
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data.clone()), &mut effect_handler)
            .await
            .unwrap();
        let _ = receiver.recv().await.unwrap();

        // NACK the message multiple times to exceed max retries
        for i in 1..=4 {
            processor
                .process(
                    Message::Control(NodeControlMsg::Nack {
                        id: 1,
                        reason: format!("Test failure {i}"),
                        pdata: None,
                    }),
                    &mut effect_handler,
                )
                .await
                .unwrap();
        }

        // Message should be dropped after exceeding max retries
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_timer_tick_retries_ready_messages() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add a message and NACK it
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data.clone()), &mut effect_handler)
            .await
            .unwrap();
        let _ = receiver.recv().await.unwrap();

        processor
            .process(
                Message::Control(NodeControlMsg::Nack {
                    id: 1,
                    reason: "Test failure".to_string(),
                    pdata: None,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Wait for retry delay to pass
        sleep(Duration::from_millis(150)).await;

        // Process timer tick
        processor
            .process(
                Message::Control(NodeControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Should have sent retry message
        let retry_data = receiver.recv().await.unwrap();
        assert!(requests_match(&test_data, &retry_data));
    }

    #[tokio::test]
    async fn test_queue_full_returns_error() {
        let config = RetryConfig {
            max_pending_messages: 2, // Small queue for testing
            ..Default::default()
        };
        let mut processor = RetryProcessor::with_config(config);
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Fill the queue
        for i in 1..=2 {
            let test_data = create_test_data(i);
            processor
                .process(Message::PData(test_data), &mut effect_handler)
                .await
                .unwrap();
            let _ = receiver.recv().await.unwrap();
        }

        // Try to add one more message - should fail
        let test_data = create_test_data(3);
        let result = processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await;
        assert!(result.is_err());

        if let Err(Error::ProcessorError { error, .. }) = result {
            assert!(error.contains("Retry queue is full"));
        } else {
            panic!("Expected ProcessorError");
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add a message
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();
        let _ = receiver.recv().await.unwrap();

        // NACK it to get first retry count
        processor
            .process(
                Message::Control(NodeControlMsg::Nack {
                    id: 1,
                    reason: "First failure".to_string(),
                    pdata: None,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        let first_retry_count = processor.pending_messages.get(&1).unwrap().retry_count;
        assert_eq!(first_retry_count, 1);

        // NACK it again to get second retry count
        processor
            .process(
                Message::Control(NodeControlMsg::Nack {
                    id: 1,
                    reason: "Second failure".to_string(),
                    pdata: None,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        let second_retry_count = processor.pending_messages.get(&1).unwrap().retry_count;
        assert_eq!(second_retry_count, 2);

        // Verify exponential backoff by checking the retry counts increase
        // This is more reliable than timing-based assertions
        assert!(second_retry_count > first_retry_count);
    }

    #[tokio::test]
    async fn test_shutdown_flushes_pending_messages() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Add multiple messages and NACK them
        for i in 1..=3 {
            let test_data = create_test_data(i);
            processor
                .process(Message::PData(test_data), &mut effect_handler)
                .await
                .unwrap();
            let _ = receiver.recv().await.unwrap();

            processor
                .process(
                    Message::Control(NodeControlMsg::Nack {
                        id: i,
                        reason: "Test failure".to_string(),
                        pdata: None,
                    }),
                    &mut effect_handler,
                )
                .await
                .unwrap();
        }

        assert_eq!(processor.pending_messages.len(), 3);

        // Shutdown should flush all pending messages
        processor
            .process(
                Message::Control(NodeControlMsg::Shutdown {
                    deadline: Duration::from_secs(5),
                    reason: "Test shutdown".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // All pending messages should be cleared
        assert_eq!(processor.pending_messages.len(), 0);

        // Should have sent all pending messages downstream
        for _ in 1..=3 {
            let _ = receiver.recv().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_update() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        let new_config = RetryConfig {
            max_retries: 5,
            initial_retry_delay_ms: 200,
            max_retry_delay_ms: 2000,
            backoff_multiplier: 3.0,
            max_pending_messages: 20,
            cleanup_interval_secs: 2,
        };

        let config_json = serde_json::to_value(new_config.clone()).unwrap();
        processor
            .process(
                Message::Control(NodeControlMsg::Config {
                    config: config_json,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        assert_eq!(processor.config.max_retries, new_config.max_retries);
        assert_eq!(
            processor.config.initial_retry_delay_ms,
            new_config.initial_retry_delay_ms
        );
        assert_eq!(
            processor.config.max_retry_delay_ms,
            new_config.max_retry_delay_ms
        );
        assert_eq!(
            processor.config.backoff_multiplier,
            new_config.backoff_multiplier
        );
        assert_eq!(
            processor.config.max_pending_messages,
            new_config.max_pending_messages
        );
        assert_eq!(
            processor.config.cleanup_interval_secs,
            new_config.cleanup_interval_secs
        );
    }
}
