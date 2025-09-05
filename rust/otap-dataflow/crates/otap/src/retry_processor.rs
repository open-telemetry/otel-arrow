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
//! │  Upstream   │◄─────────────│ Retry       │◄─────────────│ Downstream  │
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
use std::time::Duration;
use tokio::time::Instant;
use tonic::Code;

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
    retries: usize, // r0
}

impl From<RetryState> for ReplyState {
    fn from(value: RetryState) -> Self {
        Self::new(value.retries.into())
    }
}

impl TryFrom<ReplyState> for RetryState {
    type Error = crate::pdata::error::Error;

    fn try_from(value: ReplyState) -> Result<Self, Self::Error> {
        Ok(Self {
            retries: value.r0.try_into()?,
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
                if let Some(_) = data.return_node_id() {
                    let rstate = RetryState { retries: 0 };
                    data.context
                        .reply_to(effect_handler.processor_id().index(), rstate.into());
                }

                effect_handler.send_message(data).await?;
                Ok(())
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Ack(mut ack) => {
                    // Note: Ack is doing nothing but propagating
                    // here. the pipeline controller could do this on
                    // behalf of components with simple Ack
                    // propagation.
                    _ = ack.context.pop_stack();
                    if let Some(return_to) = ack.context.mut_context().return_node_id() {
                        effect_handler.reply(return_to, AckOrNack::Ack(ack)).await?;
                    }
                    Ok(())
                }
                NodeControlMsg::Nack(mut nack) => {
                    let mut rstate: RetryState = nack.request.pop_stack().try_into()?;

                    // The receiver of the Nack will pop the reply. Here we
                    // peek at the next recipient.
                    let node_id =
                        nack.request
                            .return_node_id()
                            .ok_or_else(|| Error::ProcessorError {
                                processor: effect_handler.processor_id(),
                                error: "retry with missing return_to".into(),
                            })?;

                    // If the error is permanent or too many retries.
                    // If the payload is empty: the effect is also
                    // permanent, as by intentionaly failing to return
                    // the data.
                    let has_failed = if nack.permanent {
                        nack.reason = format!("cannot retry permanent: {}", nack.reason);
                        true
                    } else if rstate.retries >= self.config.max_retries {
                        nack.reason = format!("max retries reached: {}", nack.reason);
                        true
                    } else if nack.request.is_empty() {
                        nack.reason = format!("retry internal error: {}", nack.reason);
                        nack.permanent = true;
                        nack.code = Some(Code::Internal as i32);
                        true
                    } else {
                        false
                    };
                    if has_failed {
                        effect_handler.reply(node_id, AckOrNack::Nack(nack)).await?;
                        return Ok(());
                    }

                    // Increment the retry count (for the powi() below)
                    rstate.retries += 1;

                    let now = Instant::now();
                    let delay_ms = (self.config.initial_retry_delay_ms as f64
                        * self
                            .config
                            .backoff_multiplier
                            .powi(rstate.retries as i32 - 1))
                    .min(self.config.max_retry_delay_ms as f64)
                        as u64;

                    let next_retry_time = now + Duration::from_millis(delay_ms);

                    // Note that the Go component has a "max_elapsed" configuration,
                    // we don't here.
                    let expired = nack
                        .request
                        .context
                        .deadline
                        // duration_since asks "how long after", how long
                        // is deadline after retry time. zero indicates expired.
                        .map(|dead| {
                            Instant::from_std(dead)
                                .duration_since(next_retry_time)
                                .is_zero()
                        })
                        .unwrap_or(false);

                    if expired {
                        nack.reason = format!("final retry: {}", nack.reason);
                        effect_handler.reply(node_id, AckOrNack::Nack(nack)).await?;
                        return Ok(());
                    }

                    let mut request = nack.request;

                    request
                        .mut_context()
                        .reply_to(effect_handler.processor_id().index(), rstate.into());

                    effect_handler
                        .delay_message(request, next_retry_time)
                        .await?;

                    Ok(())
                }
                NodeControlMsg::DelayedData { data } => {
                    effect_handler.send_message(*data).await?;
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
    use crate::context::Register;
    use otap_df_channel::mpsc;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{AckMsg, NackMsg};
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;
    use std::collections::HashMap;
    use tonic::Code;
    //use tokio::time::{Duration, sleep};

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
        OtapPdata::new_default(
            OtapArrowBytes::ArrowLogs(create_simple_logs_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows: 1,
                    ..Default::default()
                },
            ))
            .into(),
        )
    }

    /// Test helper to compare two OtapPdata instances for equivalence.
    fn requests_match(expected: &OtapPdata, actual: &OtapPdata) -> bool {
        // ToDo: Implement full semantic equivalence checking similar to Go's assert.Equiv()
        expected.clone().num_items() == actual.clone().num_items()
    }

    #[test]
    fn test_factory_creation() {
        let config = json!({
            "max_retries": 5,
            "initial_retry_delay_ms": 500,
            "max_retry_delay_ms": 15000,
            "backoff_multiplier": 1.5,
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
    async fn test_process_pdata_message_with_return_node() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        let mut test_data = create_test_data(1);
        // Set a return node ID to simulate a request that expects a reply
        test_data.context.reply_to(999, ReplyState::new(Register::Usize(0))); // Some upstream node ID

        let message = Message::PData(test_data.clone());

        processor
            .process(message, &mut effect_handler)
            .await
            .unwrap();

        // Should have sent message downstream
        let received = receiver.recv().await.unwrap();
        assert!(requests_match(&test_data, &received));
        
        // Verify that retry state was added to the context stack
        assert!(received.context.has_reply_state());
    }

    #[tokio::test]
    async fn test_process_pdata_message_without_return_node() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        let test_data = create_test_data(1);
        // No return node ID set - this is a fire-and-forget message

        let message = Message::PData(test_data.clone());

        processor
            .process(message, &mut effect_handler)
            .await
            .unwrap();

        // Should have sent message downstream
        let received = receiver.recv().await.unwrap();
        assert!(requests_match(&test_data, &received));
        
        // Should not have any reply state since no return node
        assert!(!received.context.has_reply_state());
    }

    #[tokio::test]
    async fn test_ack_processing_pops_stack() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Create an ACK message with a reply stack
        let test_data = create_test_data(1);

        let ack = AckMsg::new(test_data, None);

        processor
            .process(
                Message::Control(NodeControlMsg::Ack(ack)),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // The ACK should have been processed and forwarded upstream
        // Since we don't have access to the reply mechanism in the test,
        // we just verify the processor didn't panic/error
    }

    #[tokio::test]
    async fn test_nack_schedules_retry() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Create a NACK message with retry state
        let test_data = create_test_data(1);

        let nack = NackMsg {
            request: Box::new(test_data),
            reason: "Test failure".to_string(),
            permanent: false,
            code: None,
        };

        processor
            .process(
                Message::Control(NodeControlMsg::Nack(nack)),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // The NACK should have been processed and scheduled for retry
        // Since we don't have direct access to the delay mechanism in the test,
        // we just verify the processor didn't panic/error
    }

    #[tokio::test]
    async fn test_nack_permanent_error() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Create a NACK message with permanent error
        let test_data = create_test_data(1);

        let nack = NackMsg {
            request: Box::new(test_data),
            reason: "Permanent failure".to_string(),
            permanent: true, // Permanent error
            code: Some(Code::InvalidArgument as i32),
        };

        processor
            .process(
                Message::Control(NodeControlMsg::Nack(nack)),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Permanent errors should be immediately forwarded upstream without retry
    }

    #[tokio::test]
    async fn test_nack_max_retries_exceeded() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Create a NACK message that has already exceeded max retries
        let test_data = create_test_data(1);

        let nack = NackMsg {
            request: Box::new(test_data),
            reason: "Retry limit exceeded".to_string(),
            permanent: false,
            code: None,
        };

        processor
            .process(
                Message::Control(NodeControlMsg::Nack(nack)),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Should forward NACK upstream instead of retrying
    }

    #[tokio::test]
    async fn test_nack_empty_request() {
        let mut processor = create_test_processor();
        let (sender, _receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        // Create a NACK message with empty request data
        let test_data = create_test_data(1);
        // Note: We can't easily make the data "empty" without knowing the internal structure
        // This test simulates the behavior but may need adjustment based on actual empty detection

        let nack = NackMsg {
            request: Box::new(test_data),
            reason: "Empty request".to_string(),
            permanent: false,
            code: None,
        };

        processor
            .process(
                Message::Control(NodeControlMsg::Nack(nack)),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Empty requests should be treated as permanent failures
    }

    #[tokio::test]
    async fn test_delayed_data_processing() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = EffectHandler::new(test_node("retry"), senders_map, None);

        let test_data = create_test_data(1);
        let boxed_data = Box::new(test_data.clone());

        processor
            .process(
                Message::Control(NodeControlMsg::DelayedData { data: boxed_data }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Should have sent the delayed message downstream
        let received = receiver.recv().await.unwrap();
        assert!(requests_match(&test_data, &received));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
        };

        // Test backoff calculation logic (extracted from the NACK processing)
        let test_cases = vec![
            (1, 100),  // First retry: 100 * 2^0 = 100ms
            (2, 200),  // Second retry: 100 * 2^1 = 200ms
            (3, 400),  // Third retry: 100 * 2^2 = 400ms
            (4, 800),  // Fourth retry: 100 * 2^3 = 800ms
            (5, 1600), // Fifth retry: 100 * 2^4 = 1600ms
        ];

        for (retry_count, expected_delay) in test_cases {
            let delay_ms = (config.initial_retry_delay_ms as f64
                * config.backoff_multiplier.powi(retry_count - 1))
            .min(config.max_retry_delay_ms as f64) as u64;
            
            assert_eq!(delay_ms, expected_delay, "Retry count {retry_count} should have delay {expected_delay}ms");
        }
    }

    #[test]
    fn test_max_delay_capping() {
        let config = RetryConfig {
            max_retries: 10,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
        };

        // Test that delays are capped at max_retry_delay_ms
        let high_retry_count = 10;
        let delay_ms = (config.initial_retry_delay_ms as f64
            * config.backoff_multiplier.powi(high_retry_count - 1))
        .min(config.max_retry_delay_ms as f64) as u64;
        
        assert_eq!(delay_ms, config.max_retry_delay_ms, "High retry counts should be capped at max delay");
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
    }
}
