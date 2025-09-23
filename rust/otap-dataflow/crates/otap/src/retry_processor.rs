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

use crate::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    EffectHandlerExtension, Interests, ProcessorFactory,
    config::ProcessorConfig,
    control::{CtxData, NodeControlMsg},
    error::{Error, TypedError},
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the RetryProcessor processor
pub const RETRY_PROCESSOR_URN: &str = "urn:otap:processor:retry_processor";

/// Telemetry metrics for the RetryProcessor (RFC-aligned items + component counters).
#[metric_set(name = "retry.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct RetryProcessorMetrics {
    // RFC-aligned: consumed items by signal and outcome
    /// Number of items consumed (logs) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_logs_success: Counter<u64>,
    /// Number of items consumed (metrics) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_metrics_success: Counter<u64>,
    /// Number of items consumed (traces) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_traces_success: Counter<u64>,

    /// Number of items consumed (logs) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_logs_failure: Counter<u64>,
    /// Number of items consumed (metrics) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_metrics_failure: Counter<u64>,
    /// Number of items consumed (traces) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_traces_failure: Counter<u64>,

    /// Number of items consumed (logs) with outcome=refused
    #[metric(unit = "{item}")]
    pub consumed_items_logs_refused: Counter<u64>,
    /// Number of items consumed (metrics) with outcome=refused
    #[metric(unit = "{item}")]
    pub consumed_items_metrics_refused: Counter<u64>,
    /// Number of items consumed (traces) with outcome=refused
    #[metric(unit = "{item}")]
    pub consumed_items_traces_refused: Counter<u64>,

    // RFC-aligned: produced items by signal and outcome
    /// Number of items produced (logs) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_logs_success: Counter<u64>,
    /// Number of items produced (metrics) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_metrics_success: Counter<u64>,
    /// Number of items produced (traces) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_traces_success: Counter<u64>,

    /// Number of items produced (logs) with outcome=refused (downstream error)
    #[metric(unit = "{item}")]
    pub produced_items_logs_refused: Counter<u64>,
    /// Number of items produced (metrics) with outcome=refused (downstream error)
    #[metric(unit = "{item}")]
    pub produced_items_metrics_refused: Counter<u64>,
    /// Number of items produced (traces) with outcome=refused (downstream error)
    #[metric(unit = "{item}")]
    pub produced_items_traces_refused: Counter<u64>,

    /// Number of items produced (logs) with outcome=failure (originating here)
    #[metric(unit = "{item}")]
    pub produced_items_logs_failure: Counter<u64>,
    /// Number of items produced (metrics) with outcome=failure
    #[metric(unit = "{item}")]
    pub produced_items_metrics_failure: Counter<u64>,
    /// Number of items produced (traces) with outcome=failure
    #[metric(unit = "{item}")]
    pub produced_items_traces_failure: Counter<u64>,

    // Component-specific counters
    /// Number of messages added to the pending retry queue.
    #[metric(unit = "{msg}")]
    pub msgs_enqueued: Counter<u64>,

    /// Number of ACKs received that removed a message from the pending queue.
    #[metric(unit = "{msg}")]
    pub msgs_acked: Counter<u64>,

    /// Number of NACK control messages processed.
    #[metric(unit = "{msg}")]
    pub nacks_received: Counter<u64>,

    /// Number of retry attempts scheduled as a result of NACKs.
    #[metric(unit = "{event}")]
    pub retry_attempts: Counter<u64>,

    /// Number of messages re-sent due to a retry.
    #[metric(unit = "{msg}")]
    pub msgs_retried: Counter<u64>,

    /// Number of messages dropped because the queue was full.
    #[metric(unit = "{msg}")]
    pub msgs_dropped_queue_full: Counter<u64>,

    /// Number of messages dropped after exceeding the maximum retries.
    #[metric(unit = "{msg}")]
    pub msgs_dropped_exceeded_retries: Counter<u64>,

    /// Number of expired messages removed during cleanup.
    #[metric(unit = "{msg}")]
    pub msgs_removed_expired: Counter<u64>,
}

impl RetryProcessorMetrics {
    /// Increment consumed.items with outcome=success for the given signal by n
    pub fn add_consumed_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_success.add(n),
            SignalType::Metrics => self.consumed_items_metrics_success.add(n),
            SignalType::Traces => self.consumed_items_traces_success.add(n),
        }
    }
    /// Increment consumed.items with outcome=failure for the given signal by n
    pub fn add_consumed_failure(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_failure.add(n),
            SignalType::Metrics => self.consumed_items_metrics_failure.add(n),
            SignalType::Traces => self.consumed_items_traces_failure.add(n),
        }
    }
    /// Increment consumed.items with outcome=refused for the given signal by n
    pub fn add_consumed_refused(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_refused.add(n),
            SignalType::Metrics => self.consumed_items_metrics_refused.add(n),
            SignalType::Traces => self.consumed_items_traces_refused.add(n),
        }
    }

    /// Increment produced.items with outcome=success for the given signal by n
    pub fn add_produced_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_success.add(n),
            SignalType::Metrics => self.produced_items_metrics_success.add(n),
            SignalType::Traces => self.produced_items_traces_success.add(n),
        }
    }
    /// Increment produced.items with outcome=refused for the given signal by n
    pub fn add_produced_refused(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_refused.add(n),
            SignalType::Metrics => self.produced_items_metrics_refused.add(n),
            SignalType::Traces => self.produced_items_traces_refused.add(n),
        }
    }
    /// Increment produced.items with outcome=failure for the given signal by n
    pub fn add_produced_failure(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_failure.add(n),
            SignalType::Metrics => self.produced_items_metrics_failure.add(n),
            SignalType::Traces => self.produced_items_traces_failure.add(n),
        }
    }
}

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

    /// Optional metrics handle
    metrics: Option<MetricSet<RetryProcessorMetrics>>,
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_retry_processor(
    pipeline_ctx: PipelineContext,
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

    // Create the router processor with metrics registered for this node
    let router = RetryProcessor::with_pipeline_ctx(pipeline_ctx, config);

    // Create NodeUserConfig and wrap as local processor
    let user_config = Arc::new(NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN));

    Ok(ProcessorWrapper::local(
        router,
        node,
        user_config,
        processor_config,
    ))
}

// impl Default for RetryProcessor {
//     fn default() -> Self {
//         Self::new()
//     }
// }

#[derive(Debug)]
struct RetryState {
    retries: usize, // register
}

impl RetryState {
    fn new() -> Self {
        Self { retries: 0 }
    }
}

impl From<RetryState> for CtxData {
    fn from(value: RetryState) -> Self {
        smallvec::smallvec![value.retries.into()]
    }
}

impl From<CtxData> for RetryState {
    fn from(value: CtxData) -> Self {
        Self {
            retries: value[0].into(),
        }
    }
}

impl RetryProcessor {
    // /// Creates a new RetryProcessor with default configuration
    // #[must_use]
    // pub fn new() -> Self {
    //     Self::with_config(RetryConfig::default())
    // }

    /// Creates a new RetryProcessor with metrics registered via PipelineContext
    #[must_use]
    pub fn with_pipeline_ctx(pipeline_ctx: PipelineContext, config: RetryConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<RetryProcessorMetrics>();
        Self {
            config,
            metrics: Some(metrics),
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
        // TODO Can we add automatic metric for the unsubscribed Ack
        // for metrics on the return path?
        match msg {
            Message::PData(mut data) => {
                effect_handler
                    .subscribe_to(Interests::NACKS, RetryState::new().into(), &mut data)
                    .await;
                match effect_handler.send_message(data).await {
                    Ok(()) => {
                        // Request control flows downstream.
                        Ok(())
                    }
                    Err(TypedError::ChannelSendError(sent)) => {
                        // TODO: Note we remove full vs closed info here.
                        let data = sent.inner();
                        let signal = data.signal_type();
                        let items = data.num_items();
                        // TODO: Note that if we fail to send, we drop.
                        // How would we delay or Nack when the outbound queue is full?
                        if let Some(m) = self.metrics.as_mut() {
                            m.add_produced_failure(signal, items as u64);
                        }
                        // TODO: What do we return?
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Nack(mut nack) => {
                    // If the error is permanent or too many retries.
                    // If the payload is empty: the effect is also
                    // permanent, as by intentionaly failing to return
                    // the data.
                    if nack.permanent {
                        // Passthrough
                        effect_handler.notify_nack(nack).await;
                        return Ok(());
                    } else if nack.context.is_none() || nack.refused.is_empty() {
                        nack.reason = format!("retry internal error: {}", nack.reason);
                        nack.permanent = true;
                        effect_handler.notify_nack(nack).await;
                        return Ok(());
                    }

                    let mut rstate: RetryState = nack.context.take().expect("some").into();

                    if rstate.retries >= self.config.max_retries {
                        nack.reason = format!("max retries reached: {}", nack.reason);
                        effect_handler.notify_nack(nack).await;
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

                    // TODO: implement delay!
                    let next_retry_time = now + Duration::from_millis(delay_ms);
                    let _ = next_retry_time;

                    // E.g., check deadline-before-retry
                    // let deadline = nack.refused.deadline();
                    // let expired = deadline
                    //     .map(|dead| dead <= next_retry_time)
                    //     .unwrap_or(false);
                    // if expired {
                    //     // Deadline expired: forward NACK upstream. Not permanent.
                    //     nack.reason = format!("final retry: {}", nack.reason);
                    //     effect_handler.notify_nack(nack).await?;
                    //     return Ok(());
                    // }

                    // Updated RetryState back onto context for retry attempt
                    let mut rereq = nack.refused;
                    effect_handler
                        .subscribe_to(Interests::NACKS, rstate.into(), &mut rereq)
                        .await;

                    match effect_handler.send_message(*rereq).await {
                        Ok(()) => Ok(()),
                        Err(TypedError::ChannelSendError(sent)) => {
                            // TODO: Note we remove full vs closed info here.
                            let data = sent.inner();
                            let signal = data.signal_type();
                            let items = data.num_items();

                            // TODO: Note that if we fail to send, we drop.
                            // Can we delay or Nack when the outbound queue is full?
                            if let Some(m) = self.metrics.as_mut() {
                                m.add_produced_failure(signal, items as u64);
                            }
                            Ok(())
                        }
                        Err(e) => Err(e.into()),
                    }
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    if let Some(metrics) = self.metrics.as_mut() {
                        let _ = metrics_reporter.report(metrics);
                    }
                    Ok(())
                }
                NodeControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                NodeControlMsg::Ack(_) | NodeControlMsg::TimerTick { .. } => {
                    unreachable!("impossible");
                }
                NodeControlMsg::Shutdown { .. } => {
                    // @@@ TODO Stop retrying; should just work b/c
                    // SendError<T> will tell you a permanent closed?
                    Ok(())
                }
            },
        }
    }
}
