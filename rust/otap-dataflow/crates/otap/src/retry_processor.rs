// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Retry Processor with ACK/NACK Feedback Loop
//!
//! The retry processor implements reliable message delivery through an ACK/NACK feedback system.
//! Messages are tracked with unique IDs and retried on failure using exponential backoff.
//!
//! ## RetryState Handler Responsibility
//!
//! The retry processor uses a **stateless design** where retry information is stored in the message
//! context stack rather than in the processor itself. This enables horizontal scaling and fault tolerance.
//!
//! ### Forward Path (Data Processing)
//! 1. When a PData message arrives, the processor checks if it has a return node ID
//! 2. If it does, the processor **pushes** a `RetryState` onto the context stack before forwarding
//! 3. The `RetryState` contains the current retry count (initially 0)
//! 4. The message is then sent downstream with this state attached
//!
//! ### Backward Path (ACK/NACK Processing)
//! 1. **Return Sender (this processor)**: When an ACK/NACK comes back, this processor **peeks** at
//!    the `RetryState` to determine retry count and decide whether to retry or fail permanently
//! 2. **Originator/Recipient**: The final recipient of the ACK/NACK **pops** the `RetryState` from
//!    the stack, completing the round-trip and cleaning up the context
//!
//! ### State Lifecycle
//! ```text
//! Request Flow:
//! [Upstream] --PData--> [RetryProcessor: PUSH RetryState] --PData+State--> [Downstream]
//!
//! Success Flow:
//! [Upstream] <--ACK-- [RetryProcessor: PEEK+POP State] <--ACK+State-- [Downstream]
//!
//! Retry Flow:
//! [Upstream] <--Delayed-- [RetryProcessor: PEEK State, increment retries] <--NACK+State-- [Downstream]
//!                         [RetryProcessor: PUSH updated State] --PData+State--> [Downstream]
//!
//! Failure Flow:
//! [Upstream] <--NACK-- [RetryProcessor: PEEK+POP State] <--NACK+State-- [Downstream]
//! ```
//!
//! This design ensures that:
//! - **Scalability**: No processor-local state to synchronize across instances
//! - **Fault Tolerance**: Retry state travels with the message, surviving processor restarts
//! - **Correctness**: Each retry attempt increments the count, preventing infinite loops
//! - **Clean Separation**: Stack operations clearly delineate responsibility boundaries
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

use crate::pdata::{ContextData, Interest, OtapPdata};

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    ProcessorFactory,
    config::ProcessorConfig,
    control::{NackMsg, NodeControlMsg},
    error::Error,
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
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
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    // Deserialize the (currently empty) router configuration
    let config: RetryConfig = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse retry configuration: {e}"),
        }
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
    retries: usize, // register
}

impl From<RetryState> for ContextData {
    fn from(value: RetryState) -> Self {
        Self::new(value.retries)
    }
}

//impl From<OtapArrowRecords> for OtapPayload {
//    fn from(value: OtapArrowRecords) -> Self {

impl From<ContextData> for RetryState {
    fn from(value: ContextData) -> Self {
        Self {
            retries: value.register0(),
        }
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
            Message::PData(data) => {
                match data.with_reply_to(
                    Interest::NackOnly,
                    effect_handler.processor_id().index(),
                    ContextData::new(0),
                ) {
                    (data, None) => {
                        effect_handler.send_message(data).await?;
                        Ok(())
                    }
                    (data, Some(err)) => {
                        effect_handler
                            .reply_nack(
                                data.return_node_id(),
                                NackMsg::new(data, err.to_string(), false, None),
                            )
                            .await?;
                        Ok(())
                    }
                }
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Ack(_) => {
                    // This component does not subscribe to Acks, they pass directly
                    // to the next interested component.
                    unreachable!();
                }
                NodeControlMsg::Nack(mut nack) => {
                    // Backward path: NACK processing - PEEK at retry state to decide action
                    let reply = nack.refused.take_reply();
                    let frame = reply
                        .ok_or_else(|| effect_handler.internal_error("missing reply state"))?;
                    let mut rstate: RetryState = frame.data.into();

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
                    } else if nack.refused.is_empty() {
                        nack.reason = format!("retry internal error: {}", nack.reason);
                        nack.permanent = true;
                        nack.code = Some(Code::Internal as i32);
                        true
                    } else {
                        false
                    };
                    let return_node_id = nack.refused.return_node_id();

                    if has_failed {
                        // Permanent failure: forward NACK upstream (originator will POP state)
                        effect_handler.reply_nack(return_node_id, nack).await?;
                        return Ok(());
                    }

                    // Retry attempt: increment counter and schedule delayed retry
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
                    let deadline = nack.refused.deadline();

                    let expired = deadline
                        .map(|dead| dead <= next_retry_time)
                        .unwrap_or(false);

                    if expired {
                        // Deadline expired: forward NACK upstream. Not permanent.
                        nack.reason = format!("final retry: {}", nack.reason);
                        effect_handler.reply_nack(return_node_id, nack).await?;
                        return Ok(());
                    }

                    // Updated RetryState back onto context for retry attempt
                    match nack.refused.with_reply_to(
                        Interest::NackOnly,
                        effect_handler.processor_id().index(),
                        rstate.into(),
                    ) {
                        (repeat_request, None) => {
                            effect_handler
                                .send_delayed(
                                    effect_handler.processor_id().index(),
                                    repeat_request,
                                    next_retry_time,
                                )
                                .await
                        }
                        (repeat_request, Some(err)) => {
                            effect_handler
                                .reply_nack(
                                    repeat_request.return_node_id(),
                                    NackMsg::new(repeat_request, err.to_string(), false, None),
                                )
                                .await
                        }
                    }
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
