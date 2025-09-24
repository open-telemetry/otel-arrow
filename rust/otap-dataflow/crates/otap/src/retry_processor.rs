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

    /// Number of items produced (logs)
    #[metric(unit = "{item}")]
    pub retried_items_logs: Counter<u64>,
    /// Number of items produced (metrics)
    #[metric(unit = "{item}")]
    pub retried_items_metrics: Counter<u64>,
    /// Number of items retried (traces)
    #[metric(unit = "{item}")]
    pub retried_items_traces: Counter<u64>,
}

impl RetryProcessorMetrics {
    /// Increment consumed.items with outcome=success for the given signal by n
    fn add_consumed_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_success.add(n),
            SignalType::Metrics => self.consumed_items_metrics_success.add(n),
            SignalType::Traces => self.consumed_items_traces_success.add(n),
        }
    }
    /// Increment consumed.items with outcome=failure for the given signal by n
    fn add_consumed_failure(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_failure.add(n),
            SignalType::Metrics => self.consumed_items_metrics_failure.add(n),
            SignalType::Traces => self.consumed_items_traces_failure.add(n),
        }
    }
    /// Increment consumed.items with outcome=refused for the given signal by n
    fn add_consumed_refused(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_refused.add(n),
            SignalType::Metrics => self.consumed_items_metrics_refused.add(n),
            SignalType::Traces => self.consumed_items_traces_refused.add(n),
        }
    }

    /// Increment produced.items with outcome=success for the given signal by n
    fn add_produced_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_success.add(n),
            SignalType::Metrics => self.produced_items_metrics_success.add(n),
            SignalType::Traces => self.produced_items_traces_success.add(n),
        }
    }
    /// Increment produced.items with outcome=refused for the given signal by n
    fn add_produced_refused(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_refused.add(n),
            SignalType::Metrics => self.produced_items_metrics_refused.add(n),
            SignalType::Traces => self.produced_items_traces_refused.add(n),
        }
    }

    /// Increment retried.items for the given signal by n
    fn add_retried(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.retried_items_logs.add(n),
            SignalType::Metrics => self.retried_items_metrics.add(n),
            SignalType::Traces => self.retried_items_traces.add(n),
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
    metrics: MetricSet<RetryProcessorMetrics>,
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
    /// Creates a new RetryProcessor with metrics registered via PipelineContext
    #[must_use]
    pub fn with_pipeline_ctx(pipeline_ctx: PipelineContext, config: RetryConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<RetryProcessorMetrics>();
        Self { config, metrics }
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
                    .subscribe_to(
                        Interests::NACKS | Interests::ACKS,
                        RetryState::new().into(),
                        &mut data,
                    )
                    .await;
                match effect_handler.send_message(data).await {
                    Ok(()) => {
                        // Request control flows downstream.
                        Ok(())
                    }
                    Err(TypedError::ChannelSendError(sent)) => {
                        let data = sent.inner();
                        let signal = data.signal_type();
                        let items = data.num_items();
                        self.metrics.add_consumed_failure(signal, items as u64);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Nack(mut nack) => {
                    let signal = nack.refused.signal_type();
                    let items = nack.refused.num_items();

                    // The producer side returned one refusal.
                    self.metrics.add_produced_refused(signal, items as u64);

                    if nack.permanent {
                        // Passthrough permanent nacks.
                        effect_handler.notify_nack(nack).await?;
                        self.metrics.add_consumed_refused(signal, items as u64);
                        return Ok(());
                    } else if nack.context.is_none() || nack.refused.is_empty() {
                        // We might retry, but how could we?
                        nack.reason = format!("retry internal error: {}", nack.reason);
                        nack.permanent = true;
                        effect_handler.notify_nack(nack).await?;
                        self.metrics.add_consumed_failure(signal, items as u64);
                        return Ok(());
                    }

                    let mut rstate: RetryState = nack.context.take().expect("some").into();

                    if rstate.retries >= self.config.max_retries {
                        // The final refusal counts to the consumer.
                        nack.reason = format!("max retries reached: {}", nack.reason);
                        effect_handler.notify_nack(nack).await?;
                        self.metrics.add_consumed_refused(signal, items as u64);
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

                    // Compute the delay.
                    let next_retry_time = now + Duration::from_millis(delay_ms);

                    // E.g., check deadline-before-retry
                    // let deadline = nack.refused.deadline();
                    // let expired = deadline
                    //     .map(|dead| dead <= next_retry_time)
                    //     .unwrap_or(false);
                    // if expired {
                    //     // Deadline expired: forward NACK upstream. Not permanent.
                    //     nack.reason = format!("final retry: {}", nack.reason);
                    //     effect_handler.notify_nack(nack).await?;
                    //     self.metrics.add_consumed_failure(signal, items as u64);
                    //     return Ok(());
                    // }

                    // Updated RetryState back onto context for retry attempt
                    let mut rereq = nack.refused;
                    effect_handler
                        .subscribe_to(Interests::NACKS, rstate.into(), &mut rereq)
                        .await;

                    // Enter a delay, we'll continue in the
                    // DelayedData branch next.
                    let res = effect_handler
                        .delay_message(rereq, next_retry_time)
                        .await
                        .map_err(|e| e.into());
                    match res {
                        Ok(_) => {
                            self.metrics.add_retried(signal, items as u64);
                        }
                        Err(_) => {
                            self.metrics.add_retried(signal, items as u64);
                        }
                    }
                    res
                }
                NodeControlMsg::DelayedData { data } => {
                    // Control flow follows from delay_message() above.
                    match effect_handler.send_message(*data).await {
                        Ok(()) => Ok(()),
                        Err(TypedError::ChannelSendError(sent)) => {
                            // TODO: Note we remove full vs closed info here.
                            let data = sent.inner();
                            let signal = data.signal_type();
                            let items = data.num_items();

                            self.metrics.add_consumed_failure(signal, items as u64);
                            Ok(())
                        }
                        Err(e) => {
                            // Here we have consumed_failure but we don't expect this
                            // and haven't saved the count for metric purposes.
                            Err(e.into())
                        }
                    }
                }
                NodeControlMsg::Ack(ack) => {
                    let signal = ack.accepted.signal_type();
                    let items = ack.accepted.num_items();

                    self.metrics.add_produced_success(signal, items as u64);
                    self.metrics.add_consumed_success(signal, items as u64);

                    effect_handler.notify_ack(ack).await.map_err(|e| e.into())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => metrics_reporter
                    .report(&mut self.metrics)
                    .map_err(|error| Error::Telemetry { error }),
                NodeControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                NodeControlMsg::TimerTick { .. } => {
                    unreachable!("impossible");
                }
                NodeControlMsg::Shutdown { .. } => Ok(()),
            },
        }
    }
}
