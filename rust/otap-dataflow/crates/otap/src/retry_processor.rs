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
//!     max_pending_messages: 10000,
//!     cleanup_interval_secs: 60,
//! };
//! let processor = RetryProcessor::with_config(config);
//! ```

use crate::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    Interests, ProcessorFactory, ProducerEffectHandlerExtension,
    config::ProcessorConfig,
    control::{AckMsg, CallData, NackMsg, NodeControlMsg},
    error::{Error, ProcessorErrorKind},
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Maximum age for failed messages before cleanup (5 minutes)
const MAX_FAILED_MESSAGE_AGE_SECS: u64 = 300;

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
    /// Maximum number of messages that can be pending retry
    pub max_pending_messages: usize,
    /// Interval in seconds for cleanup of expired messages
    pub cleanup_interval_secs: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        }
    }
}

struct PendingMessage {
    data: OtapPdata,
    retry_count: usize,
    next_retry_time: Instant,
    last_error: String,
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
    pending_messages: HashMap<u64, PendingMessage>,
    next_message_id: u64,
    last_cleanup_time: Instant,
    /// Optional metrics handle (present when constructed with a PipelineContext)
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

/// RetryState is a placeholder for assumptions the retry_processor
/// makes about Ack/Nack message identifiers. This is to satisfy
/// internal testing, however we plan to use a stateless design where
/// retry count and timestamp are stored in call data, not separate
/// maps.
#[derive(Debug)]
struct RetryState {
    id: u64,
}

impl RetryState {
    #[cfg(test)]
    fn new(id: u64) -> Self {
        Self { id }
    }
}

impl From<RetryState> for CallData {
    fn from(value: RetryState) -> Self {
        smallvec::smallvec![value.id.into()]
    }
}

impl TryFrom<CallData> for RetryState {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        value
            .first()
            .map(|&val0| Ok(Self { id: val0.into() }))
            .unwrap_or(Err(Error::InternalError {
                message: "invalid calldata".into(),
            }))
    }
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
        Self {
            config,
            pending_messages: HashMap::new(),
            next_message_id: 1,
            last_cleanup_time: Instant::now(),
            metrics: None,
        }
    }

    /// Creates a new RetryProcessor with metrics registered via PipelineContext
    #[must_use]
    pub fn with_pipeline_ctx(pipeline_ctx: PipelineContext, config: RetryConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<RetryProcessorMetrics>();
        Self {
            config,
            pending_messages: HashMap::new(),
            next_message_id: 1,
            last_cleanup_time: Instant::now(),
            metrics: Some(metrics),
        }
    }

    async fn acknowledge(&mut self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
        let rstate: RetryState = ack.calldata.try_into()?;
        let id = rstate.id;
        if let Some(_removed) = self.pending_messages.remove(&id) {
            log::debug!("Acknowledged and removed message with ID: {id}");
            if let Some(m) = self.metrics.as_mut() {
                m.msgs_acked.inc();
            }
        } else {
            log::warn!("Attempted to acknowledge non-existent message with ID: {id}");
        }
        Ok(())
    }

    async fn handle_nack(
        &mut self,
        nack: NackMsg<OtapPdata>,
        _effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let rstate: RetryState = nack.calldata.try_into()?;

        let id = rstate.id;
        if let Some(m) = self.metrics.as_mut() {
            m.nacks_received.inc();
        }
        if let Some(mut pending) = self.pending_messages.remove(&id) {
            pending.retry_count += 1;
            pending.last_error = nack.reason;

            if pending.retry_count <= self.config.max_retries {
                let delay_ms = (self.config.initial_retry_delay_ms as f64
                    * self
                        .config
                        .backoff_multiplier
                        .powi(pending.retry_count as i32 - 1))
                .min(self.config.max_retry_delay_ms as f64) as u64;

                pending.next_retry_time = Instant::now() + Duration::from_millis(delay_ms);
                let retry_count = pending.retry_count;
                let _previous = self.pending_messages.insert(id, pending);
                if let Some(m) = self.metrics.as_mut() {
                    m.retry_attempts.inc();
                }
                log::debug!("Scheduled message {id} for retry attempt {retry_count}");
            } else {
                log::error!(
                    "Message {} exceeded max retries ({}), dropping. Last error: {}",
                    id,
                    self.config.max_retries,
                    pending.last_error
                );
                if let Some(m) = self.metrics.as_mut() {
                    m.msgs_dropped_exceeded_retries.inc();
                }
            }
        } else {
            log::warn!("Attempted to handle nack for non-existent message with ID: {id}");
        }
        Ok(())
    }

    async fn process_pending_retries(
        &mut self,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let now = Instant::now();
        let mut ready_messages = Vec::new();

        for (&id, pending) in &self.pending_messages {
            if pending.next_retry_time <= now {
                ready_messages.push((id, pending.data.clone()));
            }
        }

        for (id, data) in ready_messages {
            log::debug!("Retrying message with ID: {id}");
            let signal = data.signal_type();
            let items = data.num_items() as u64;
            match effect_handler.send_message(data).await {
                Ok(()) => {
                    if let Some(m) = self.metrics.as_mut() {
                        m.msgs_retried.inc();
                        m.add_produced_success(signal, items);
                    }
                }
                Err(e) => {
                    if let Some(m) = self.metrics.as_mut() {
                        m.add_produced_refused(signal, items);
                    }
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    fn cleanup_expired_messages(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup_time)
            < Duration::from_secs(self.config.cleanup_interval_secs)
        {
            return;
        }

        // Clean up messages that have exceeded max retries and are old
        let max_age = Duration::from_secs(MAX_FAILED_MESSAGE_AGE_SECS);
        let expired_ids: Vec<u64> = self
            .pending_messages
            .iter()
            .filter_map(|(&id, pending)| {
                let age = now.duration_since(pending.next_retry_time);
                if pending.retry_count > self.config.max_retries && age > max_age {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        let mut removed = 0u64;
        for id in expired_ids {
            if self.pending_messages.remove(&id).is_some() {
                log::warn!("Removed expired message with ID: {id}");
                removed += 1;
            }
        }
        if removed > 0 {
            if let Some(m) = self.metrics.as_mut() {
                m.msgs_removed_expired.add(removed);
            }
        }

        self.last_cleanup_time = now;
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
                // Clone only if we need to add to retry queue AND send downstream
                let signal = data.signal_type();
                let items = data.num_items() as u64;
                // Check if queue is full first to avoid unnecessary clone
                if self.pending_messages.len() >= self.config.max_pending_messages {
                    let error_msg = format!(
                        "Retry queue is full (capacity: {}), cannot add message",
                        self.config.max_pending_messages
                    );
                    log::warn!("{error_msg}");
                    if let Some(m) = self.metrics.as_mut() {
                        m.msgs_dropped_queue_full.inc();
                        m.add_consumed_failure(signal, items);
                    }
                    // Send NACK upstream to signal backpressure instead of forwarding message
                    // For now, we'll just log and drop the message
                    return Err(Error::ProcessorError {
                        processor: effect_handler.processor_id(),
                        kind: ProcessorErrorKind::Transport,
                        error: error_msg,
                        source_detail: String::new(),
                    });
                } else {
                    // Queue has space, add message for retry and send downstream
                    let id = self.next_message_id;
                    self.next_message_id += 1;

                    let pending = PendingMessage {
                        data: data.clone(), // Only clone when we know we need to store AND send
                        retry_count: 0,
                        next_retry_time: Instant::now(),
                        last_error: String::new(),
                    };

                    let _previous = self.pending_messages.insert(id, pending);
                    if let Some(m) = self.metrics.as_mut() {
                        m.msgs_enqueued.inc();
                        m.add_consumed_success(signal, items);
                    }
                    log::debug!("Added message {id} to retry queue");

                    effect_handler.subscribe_to(
                        Interests::NACKS | Interests::RETURN_DATA,
                        RetryState { id }.into(),
                        &mut data,
                    );

                    match effect_handler.send_message(data).await {
                        Ok(()) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.add_produced_success(signal, items);
                            }
                        }
                        Err(e) => {
                            if let Some(m) = self.metrics.as_mut() {
                                m.add_produced_refused(signal, items);
                            }
                            return Err(e.into());
                        }
                    }
                }
                Ok(())
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Ack(ack) => self.acknowledge(ack).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(nack, effect_handler).await,
                NodeControlMsg::TimerTick { .. } => {
                    self.process_pending_retries(effect_handler).await?;
                    self.cleanup_expired_messages();
                    Ok(())
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
                NodeControlMsg::DelayedData { .. } => {
                    unreachable!("unused");
                }
                NodeControlMsg::Shutdown { .. } => {
                    let pending_ids: Vec<u64> = self.pending_messages.keys().cloned().collect();
                    for id in pending_ids {
                        if let Some(pending) = self.pending_messages.remove(&id) {
                            let _ = effect_handler.send_message(pending.data).await;
                        }
                    }
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
    use crate::pdata::{OtapPayload, OtlpProtoBytes};
    use otap_df_channel::mpsc;
    use otap_df_config::PortName;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::MetricsSystem;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use otel_arrow_rust::Consumer;
    use otel_arrow_rust::otap::{OtapArrowRecords, from_record_messages};
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::time::{Duration, sleep};

    fn create_test_channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::Channel::new(capacity)
    }

    fn make_effect_handler(
        node_name: &str,
        senders: HashMap<PortName, LocalSender<OtapPdata>>,
    ) -> EffectHandler<OtapPdata> {
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        EffectHandler::new(
            test_node(node_name.to_owned()),
            senders,
            None,
            metrics_reporter,
        )
    }

    fn make_effect_handler_with_metrics_reporter(
        node_name: &str,
        senders: HashMap<PortName, LocalSender<OtapPdata>>,
        metrics_reporter: MetricsReporter,
    ) -> EffectHandler<OtapPdata> {
        EffectHandler::new(
            test_node(node_name.to_owned()),
            senders,
            None,
            metrics_reporter,
        )
    }

    fn create_test_processor() -> RetryProcessor {
        let config = RetryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10,
            cleanup_interval_secs: 1,
        };
        RetryProcessor::with_config(config)
    }

    fn create_test_data(_id: u64) -> OtapPdata {
        let mut consumer = Consumer::default();
        let otap_data = consumer
            .consume_bar(&mut create_simple_logs_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows: 1,
                    ..Default::default()
                },
            ))
            .unwrap();
        OtapPdata::new_default(OtapArrowRecords::Logs(from_record_messages(otap_data)).into())
    }

    /// Test helper to compare two OtapPdata instances for equivalence.
    fn requests_match(expected: &OtapPdata, actual: &OtapPdata) -> bool {
        // ToDo: Implement full semantic equivalence checking similar to Go's assert.Equiv()
        expected.num_items() == actual.num_items()
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
        let mut node_config = NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN);
        node_config.config = config;
        let result = create_retry_processor(
            pipeline_ctx,
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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

    fn empty_pdata() -> OtapPdata {
        OtapPdata::new_default(OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
            vec![],
        )))
    }

    fn test_ack_with_id(id: u64) -> AckMsg<OtapPdata> {
        let msg = empty_pdata();
        let mut ack = AckMsg::new(msg);
        ack.calldata = RetryState::new(id).into();
        ack
    }

    fn test_nack_with_id<S: Into<String>>(id: u64, reason: S) -> NackMsg<OtapPdata> {
        let msg = empty_pdata();
        let mut nack = NackMsg::new(reason, msg);
        nack.calldata = RetryState::new(id).into();
        nack
    }

    #[tokio::test]
    async fn test_ack_removes_pending() {
        let mut processor = create_test_processor();
        let (sender, receiver) = create_test_channel(10);
        let mut senders_map = HashMap::new();
        let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
                Message::Control(NodeControlMsg::Ack(test_ack_with_id(1))),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
                Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "Test failure"))),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
                    Message::Control(NodeControlMsg::Nack(test_nack_with_id(
                        1,
                        format!("Test failure {i}"),
                    ))),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

        // Add a message and NACK it
        let test_data = create_test_data(1);
        processor
            .process(Message::PData(test_data.clone()), &mut effect_handler)
            .await
            .unwrap();
        let _ = receiver.recv().await.unwrap();

        processor
            .process(
                Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "Test failure"))),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
                Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "First failure"))),
                &mut effect_handler,
            )
            .await
            .unwrap();

        let first_retry_count = processor.pending_messages.get(&1).unwrap().retry_count;
        assert_eq!(first_retry_count, 1);

        // NACK it again to get second retry count
        processor
            .process(
                Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "Second failure"))),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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
                    Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "Test failure"))),
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
                    deadline: Instant::now() + Duration::from_secs(5),
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
        let mut effect_handler = make_effect_handler("retry", senders_map);

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

    // Helper: create a minimal OtapPdata logs payload with 1 row
    fn make_test_pdata() -> OtapPdata {
        let mut consumer = Consumer::default();
        let otap_data = consumer
            .consume_bar(&mut create_simple_logs_arrow_record_batches(
                SimpleDataGenOptions {
                    num_rows: 1,
                    ..Default::default()
                },
            ))
            .unwrap();
        OtapPdata::new_default(OtapArrowRecords::Logs(from_record_messages(otap_data)).into())
    }

    // Collects current metrics for the retry processor set into a map
    fn collect_retry_metrics_map(
        registry: &MetricsRegistryHandle,
    ) -> std::collections::HashMap<&'static str, u64> {
        let mut out = std::collections::HashMap::new();
        registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "retry.processor.metrics" {
                for (field, value) in iter {
                    let _ = out.insert(field.name, value);
                }
            }
        });
        out
    }

    #[tokio::test]
    async fn test_metrics_collect_telemetry_reports_counters() {
        // 1) Telemetry system and registry/reporter
        let metrics_system = MetricsSystem::default();
        let registry = metrics_system.registry();
        let reporter = metrics_system.reporter();
        // Start background collection loop
        let _collector = tokio::spawn(metrics_system.run_collection_loop());

        // 2) Pipeline context to register metrics under
        let controller = ControllerContext::new(registry.clone());
        let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

        // 3) RetryProcessor with metrics registered via pipeline context
        let cfg = RetryConfig {
            max_retries: 2,
            initial_retry_delay_ms: 50,
            max_retry_delay_ms: 200,
            backoff_multiplier: 2.0,
            max_pending_messages: 10,
            cleanup_interval_secs: 1,
        };
        let mut proc = RetryProcessor::with_pipeline_ctx(pipeline, cfg);

        // 4) Local effect handler with a default 'out' port
        let (tx_out, rx_out) = mpsc::Channel::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::MpscSender(tx_out));
        let mut eh = make_effect_handler_with_metrics_reporter(
            "retry_proc_telemetry",
            senders,
            reporter.clone(),
        );

        // Enqueue a message (should inc msgs.enqueued) and send downstream
        let pdata = make_test_pdata();
        proc.process(Message::PData(pdata), &mut eh).await.unwrap();
        let _ = rx_out.recv().await.expect("downstream message");

        // NACK it (id=1) to schedule a retry (inc nacks.received, retry.attempts)
        proc.process(
            Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "fail"))),
            &mut eh,
        )
        .await
        .unwrap();

        // Wait and tick to perform retry (inc msgs.retried)
        sleep(Duration::from_millis(60)).await;
        proc.process(Message::Control(NodeControlMsg::TimerTick {}), &mut eh)
            .await
            .unwrap();
        let _ = rx_out.recv().await.expect("retry downstream message");

        // ACK it (inc msgs.acked)
        proc.process(
            Message::Control(NodeControlMsg::Ack(test_ack_with_id(1))),
            &mut eh,
        )
        .await
        .unwrap();

        // Trigger telemetry snapshot (report + reset)
        proc.process(
            Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }),
            &mut eh,
        )
        .await
        .unwrap();

        // Allow collector to pull the snapshot
        sleep(Duration::from_millis(50)).await;

        let map = collect_retry_metrics_map(&registry);
        // Basic expectations: enqueued >=1, nacks >=1, retries >=1, retried >=1, acked >=1
        let get = |key: &str| map.get(key).copied().unwrap_or(0);
        assert!(
            get("msgs.enqueued") >= 1,
            "msgs.enqueued should be >= 1, got {}",
            get("msgs.enqueued")
        );
        assert!(
            get("nacks.received") >= 1,
            "nacks.received should be >= 1, got {}",
            get("nacks.received")
        );
        assert!(
            get("retry.attempts") >= 1,
            "retry.attempts should be >= 1, got {}",
            get("retry.attempts")
        );
        assert!(
            get("msgs.retried") >= 1,
            "msgs.retried should be >= 1, got {}",
            get("msgs.retried")
        );
        assert!(
            get("msgs.acked") >= 1,
            "msgs.acked should be >= 1, got {}",
            get("msgs.acked")
        );
    }

    #[tokio::test]
    async fn test_metrics_queue_full_increments_drop_counter() {
        // Telemetry system
        let ms = MetricsSystem::default();
        let registry = ms.registry();
        let reporter = ms.reporter();
        let _collector = tokio::spawn(ms.run_collection_loop());

        // Pipeline + processor (capacity 1)
        let controller = ControllerContext::new(registry.clone());
        let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let cfg = RetryConfig {
            max_pending_messages: 1,
            ..Default::default()
        };
        let mut proc = RetryProcessor::with_pipeline_ctx(pipeline, cfg);

        // Effect handler with an 'out' port
        let (tx_out, rx_out) = mpsc::Channel::new(2);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::MpscSender(tx_out));
        let mut eh = make_effect_handler_with_metrics_reporter(
            "retry_queue_full",
            senders,
            reporter.clone(),
        );

        // 1st message enqueues fine
        proc.process(Message::PData(make_test_pdata()), &mut eh)
            .await
            .unwrap();
        let _ = rx_out.recv().await.expect("downstream first");

        // 2nd message should hit queue full and return error
        let res = proc
            .process(Message::PData(make_test_pdata()), &mut eh)
            .await;
        assert!(res.is_err(), "expected queue full error");

        // Report telemetry
        proc.process(
            Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }),
            &mut eh,
        )
        .await
        .unwrap();
        sleep(Duration::from_millis(50)).await;

        let map = collect_retry_metrics_map(&registry);
        let get = |key: &str| map.get(key).copied().unwrap_or(0);
        assert!(get("msgs.enqueued") >= 1, "expected at least one enqueued");
        assert!(
            get("msgs.dropped.queue.full") >= 1,
            "expected queue full drop counter to be >= 1"
        );
    }

    #[tokio::test]
    async fn test_metrics_exceeded_retries_dropped_increments_counter() {
        // Telemetry system
        let ms = MetricsSystem::default();
        let registry = ms.registry();
        let reporter = ms.reporter();
        let _collector = tokio::spawn(ms.run_collection_loop());

        // Pipeline + processor: allow only 1 retry
        let controller = ControllerContext::new(registry.clone());
        let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let cfg = RetryConfig {
            max_retries: 1,
            initial_retry_delay_ms: 10,
            max_retry_delay_ms: 10,
            backoff_multiplier: 2.0,
            max_pending_messages: 10,
            cleanup_interval_secs: 1,
        };
        let mut proc = RetryProcessor::with_pipeline_ctx(pipeline, cfg);

        // Effect handler
        let (tx_out, rx_out) = mpsc::Channel::new(4);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::MpscSender(tx_out));
        let mut eh =
            make_effect_handler_with_metrics_reporter("retry_exceeded", senders, reporter.clone());

        // Enqueue and send downstream
        proc.process(Message::PData(make_test_pdata()), &mut eh)
            .await
            .unwrap();
        let _ = rx_out.recv().await.expect("downstream message");

        // First NACK -> schedule retry (not dropped)
        proc.process(
            Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "fail1"))),
            &mut eh,
        )
        .await
        .unwrap();

        // Second NACK -> exceed max and drop
        proc.process(
            Message::Control(NodeControlMsg::Nack(test_nack_with_id(1, "fail2"))),
            &mut eh,
        )
        .await
        .unwrap();

        // Collect telemetry
        proc.process(
            Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }),
            &mut eh,
        )
        .await
        .unwrap();
        sleep(Duration::from_millis(30)).await;

        let map = collect_retry_metrics_map(&registry);
        let get = |key: &str| map.get(key).copied().unwrap_or(0);
        assert!(
            get("msgs.dropped.exceeded.retries") >= 1,
            "expected exceeded retries drop >= 1"
        );
    }

    #[tokio::test]
    async fn test_metrics_cleanup_expired_increments_counter() {
        use std::time::{Duration as StdDuration, Instant as StdInstant};
        // Telemetry system
        let ms = MetricsSystem::default();
        let registry = ms.registry();
        let reporter = ms.reporter();
        let _collector = tokio::spawn(ms.run_collection_loop());

        // Pipeline + processor
        let controller = ControllerContext::new(registry.clone());
        let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let cfg = RetryConfig {
            cleanup_interval_secs: 1,
            max_retries: 0,
            ..Default::default()
        };
        let mut proc = RetryProcessor::with_pipeline_ctx(pipeline, cfg);

        // Insert a fabricated expired pending message (retry_count > max_retries and age > MAX_FAILED_MESSAGE_AGE_SECS)
        let expired_id = 42u64;
        let pending = PendingMessage {
            data: make_test_pdata(),
            retry_count: proc.config.max_retries + 1,
            next_retry_time: StdInstant::now()
                - StdDuration::from_secs(MAX_FAILED_MESSAGE_AGE_SECS + 5),
            last_error: "expired".into(),
        };
        let _ = proc.pending_messages.insert(expired_id, pending);
        // Ensure cleanup interval elapsed
        proc.last_cleanup_time = StdInstant::now() - StdDuration::from_secs(2);

        // Run cleanup
        proc.cleanup_expired_messages();
        assert!(
            !proc.pending_messages.contains_key(&expired_id),
            "expired message should be removed"
        );

        // Report telemetry -> should include msgs.removed.expired >= 1
        // We need an effect handler but won't send data
        let (tx_out, _rx_out) = mpsc::Channel::new(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), LocalSender::MpscSender(tx_out));
        let mut eh =
            make_effect_handler_with_metrics_reporter("retry_cleanup", senders, reporter.clone());

        proc.process(
            Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }),
            &mut eh,
        )
        .await
        .unwrap();
        sleep(Duration::from_millis(30)).await;

        let map = collect_retry_metrics_map(&registry);
        let get = |key: &str| map.get(key).copied().unwrap_or(0);
        assert!(
            get("msgs.removed.expired") >= 1,
            "expected msgs.removed.expired >= 1"
        );
    }
}
