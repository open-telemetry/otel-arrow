// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The retry processor implements reliable message delivery through
//! ACK/NACK handling.  Retry state is stored in the Context. Retries are
//! issued using exponential backoff.
//!
//! The processor is configured via [`retry_processor::RetryConfig`] with
//! parameters for:
//! - Initial and maximum retry delays
//! - Maximum elapsed time
//! - Backoff multiplier
//! ```

use crate::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, ProcessorFactory, ProducerEffectHandlerExtension,
    config::ProcessorConfig,
    control::{AckMsg, CallData, NackMsg, NodeControlMsg},
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
use serde_with::{DurationSecondsWithFrac, formats::Flexible, serde_as};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// URN for the RetryProcessor processor
pub const RETRY_PROCESSOR_URN: &str = "urn:otap:processor:retry_processor";

/// Configuration for the retry processor. Modeled on
/// https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/exporterhelper/README.md#retry-on-failure.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    /// Initial delay in seconds before the first retry
    #[serde_as(as = "DurationSecondsWithFrac<f64, Flexible>")]
    pub initial_interval: Duration,

    /// Maximum delay in seconds between retries
    #[serde_as(as = "DurationSecondsWithFrac<f64, Flexible>")]
    pub max_interval: Duration,

    /// Maximum delay in seconds between retries
    #[serde_as(as = "DurationSecondsWithFrac<f64, Flexible>")]
    pub max_elapsed_time: Duration,

    /// Multiplier applied to delay for exponential backoff
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::new(5, 0),
            max_interval: Duration::new(30, 0),
            max_elapsed_time: Duration::new(300, 0),
            multiplier: 1.5,
        }
    }
}

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

/// OTAP RetryProcessor
#[allow(unsafe_code)]
#[distributed_slice(crate::OTAP_PROCESSOR_FACTORIES)]
pub static RETRY_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: RETRY_PROCESSOR_URN,
    create: create_retry_processor,
};

/// A processor that handles message retries with exponential backoff
///
/// This component only maintains state in the request context.
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
    let config: RetryConfig = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse retry configuration: {e}"),
        }
    })?;

    let router = RetryProcessor::with_pipeline_ctx(pipeline_ctx, config);

    let user_config = Arc::new(NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN));

    Ok(ProcessorWrapper::local(
        router,
        node,
        user_config,
        processor_config,
    ))
}

fn now_f64() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("epoch")
        .as_secs_f64()
}

/// RetryState is the count of retries
#[derive(Debug)]
struct RetryState {
    retries: u64,
    begin: f64,
}

impl RetryState {
    fn new() -> Self {
        Self {
            retries: 0,
            begin: now_f64(),
        }
    }
}

impl From<RetryState> for CallData {
    fn from(value: RetryState) -> Self {
        smallvec::smallvec![value.retries.into(), value.begin.into()]
    }
}

impl TryFrom<CallData> for RetryState {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(Error::InternalError {
                message: "invalid calldata".into(),
            });
        }

        Ok(Self {
            retries: value[0].into(),
            begin: value[1].into(),
        })
    }
}

impl RetryProcessor {
    /// Creates a new RetryProcessor with metrics registered via PipelineContext
    #[must_use]
    pub fn with_pipeline_ctx(pipeline_ctx: PipelineContext, config: RetryConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<RetryProcessorMetrics>();
        Self { config, metrics }
    }

    async fn handle_ack(
        &mut self,
        ack: AckMsg<OtapPdata>,
        _effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let _rstate: RetryState = ack.calldata.try_into()?;
        // @@@ METRICS
        Ok(())
    }

    async fn handle_nack(
        &mut self,
        nack: NackMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let _rstate: RetryState = nack.calldata.try_into()?;
        let _ = effect_handler
            .notify_nack(NackMsg::new(nack.reason, *nack.refused))
            .await?;

        Ok(())
    }

    async fn handle_delayed(
        &mut self,
        _when: Instant,
        _data: Box<OtapPdata>,
        _effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // TODO
        Ok(())
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
                // let signal = data.signal_type();
                // let items = data.num_items() as u64;
                effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    RetryState::new().into(),
                    &mut data,
                );
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
                NodeControlMsg::Ack(ack) => self.handle_ack(ack, effect_handler).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(nack, effect_handler).await,
                NodeControlMsg::DelayedData { when, data } => {
                    self.handle_delayed(when, data, effect_handler).await
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    let _ = metrics_reporter
                        .report(&mut self.metrics)
                        .map_err(|error| Error::Telemetry { error })?;
                    Ok(())
                }
                NodeControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                NodeControlMsg::TimerTick { .. } => {
                    unreachable!("unused");
                }
                NodeControlMsg::Shutdown { .. } => Ok(()),
            },
        }
    }
}

impl RetryProcessor {
    /// Creates a new RetryProcessor with default configuration
    #[must_use]
    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Creates a new RetryProcessor with the specified configuration
    #[must_use]
    #[cfg(test)]
    pub fn with_config(config: RetryConfig) -> Self {
        let handle = otap_df_telemetry::registry::MetricsRegistryHandle::default();
        let metrics: MetricSet<RetryProcessorMetrics> = handle.register(());

        Self { config, metrics }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    // Mock implementations for testing
    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<u64>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![0, 0], // Initialize with 2 values to match MOCK_METRICS_DESCRIPTOR
            }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self::new()
        }
    }

    #[test]
    fn test_default_config() {
        let cfg: RetryConfig = serde_json::from_value(json!({
            "initial_interval": 5.0,
            "max_interval": 30.0,
            "max_elapsed_time": 300.0,
            "multiplier": 1.5,
        }))
        .unwrap();
        assert_eq!(cfg, RetryConfig::default());
    }

    #[test]
    fn test_tiny_config() {
        let cfg: RetryConfig = serde_json::from_value(json!({
            "initial_interval": 0.5,
            "max_interval": 1.75,
            "max_elapsed_time": 9.9,
            "multiplier": 1.999,
        }))
        .unwrap();
        assert_eq!(
            cfg,
            RetryConfig {
                initial_interval: Duration::new(0, 500000000),
                max_interval: Duration::new(1, 750000000),
                max_elapsed_time: Duration::new(9, 900000000),
                multiplier: 1.999,
            }
        );
    }
}
