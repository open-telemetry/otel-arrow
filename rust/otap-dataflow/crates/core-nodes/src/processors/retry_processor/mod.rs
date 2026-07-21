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

// ToDo: Consider adding a jitter mechanism.

use otap_df_otap::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
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
use otap_df_telemetry::common_attributes::{Outcome, SignalAttributes, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MeasurementMetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// URN for the RetryProcessor processor
pub const RETRY_PROCESSOR_URN: &str = "urn:otel:processor:retry";

/// Configuration for the retry processor. Modeled exactly on
/// https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/exporterhelper/README.md#retry-on-failure.
///
/// The calculated delay is:
///
///   min(max_interval, initial_interval * multiplier.pow(retry_number))
///
/// Retries will be attempted until max_elapsed_time has passed
/// from the initial attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Initial retry interval in seconds. This is how long the
    /// first delay will be following the first NACK response.
    /// This interval is multiplied by the multiplier on subsequent
    /// retries, until it exceeds max_interval.
    #[serde(with = "humantime_serde", default = "default_initial_interval")]
    pub initial_interval: Duration,

    /// Maximum retry interval in seconds. This is a limit on
    /// individual delays in the retry processor following a single
    /// NACK failure. Prevents exponential growth when the initial
    /// interval times the exponentiated multiplier reaches this
    /// value.
    #[serde(with = "humantime_serde", default = "default_max_interval")]
    pub max_interval: Duration,

    /// Maximum elapsed time in seconds.  This is the maximum elapsed
    /// wall time for the entire request, beginning when the retry
    /// processor first sees it. Retries will not be scheduled if they
    /// would begin after this many seconds from the start.
    #[serde(with = "humantime_serde", default = "default_max_elapsed_time")]
    pub max_elapsed_time: Duration,

    /// Multiplier for the retry interval.
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

// These defaults are copied from the Collector (exporterhelper) retry sender.

const fn default_max_interval() -> Duration {
    Duration::from_secs(30)
}

const fn default_initial_interval() -> Duration {
    Duration::from_secs(5)
}

const fn default_max_elapsed_time() -> Duration {
    Duration::from_secs(300)
}

const fn default_multiplier() -> f64 {
    1.5
}

/// This prevents absurd configurations due to very small multipliers
/// or very long max_elapsed_time. There will be an error indicating to
/// raise the multiplier or increase initial_interval, etc.
const fn hard_retry_growth_limit() -> usize {
    1000
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_interval: default_max_interval(),
            initial_interval: default_initial_interval(),
            max_elapsed_time: default_max_elapsed_time(),
            multiplier: default_multiplier(),
        }
    }
}

impl RetryConfig {
    /// Computes the maximum retry count by simulation. The configuration
    /// combines exponential growth with a limit, making it difficult to
    /// reach a closed form. Returns the retry-count limit and vector of
    /// durations; note that the vector length covers only the exponential
    /// portion, subsequent retries use max_interval.
    fn compute_retry_delays(config: &RetryConfig) -> Result<(usize, Vec<Duration>), ConfigError> {
        let mut count = 0;
        let mut delays: Vec<Duration> = Vec::new();
        let mut total_elapsed = 0.0;
        let mut accum_multiplier = 1.0;

        let single_multiplier = config.multiplier;
        let initial_interval = config.initial_interval.as_secs_f64();
        let max_interval = config.max_interval.as_secs_f64();
        let limit_total_elapsed = config.max_elapsed_time.as_secs_f64();

        loop {
            let mult_d = accum_multiplier * initial_interval;
            let use_mult = mult_d < max_interval;
            let this_d = if use_mult { mult_d } else { max_interval };

            if this_d + total_elapsed >= limit_total_elapsed {
                break;
            }

            if use_mult {
                accum_multiplier *= single_multiplier;

                let limit = hard_retry_growth_limit();
                if delays.len() >= limit {
                    return Err(ConfigError::InvalidUserConfig {
                        error:
                            "retry growth: limit {limit}: raise multiplier or modify an interval"
                                .into(),
                    });
                }

                delays.push(Duration::from_secs_f64(this_d));
                total_elapsed += this_d;
                count += 1;
            } else {
                // Remaining intervals are identical: divide, round up.
                let remain = limit_total_elapsed - total_elapsed;
                count += ((remain + max_interval) / max_interval).ceil() as usize;
                break;
            }
        }

        Ok((count, delays))
    }

    /// Checks the parameters and returns pre-computed (retry limit,
    /// growth-phase delays vector)
    fn validate_retries(&self) -> Result<(usize, Vec<Duration>), ConfigError> {
        if self.multiplier < 1.0 {
            return Err(ConfigError::InvalidUserConfig {
                error: "multiplier must be >= 1".into(),
            });
        }
        if self.max_interval == Duration::from_secs(0) {
            return Err(ConfigError::InvalidUserConfig {
                error: "max_interval cannot be zero".into(),
            });
        }
        if self.initial_interval == Duration::from_secs(0) {
            return Err(ConfigError::InvalidUserConfig {
                error: "initial_interval cannot be zero".into(),
            });
        }
        if self.max_elapsed_time == Duration::from_secs(0) {
            return Err(ConfigError::InvalidUserConfig {
                error: "max_elapsed_time cannot be zero".into(),
            });
        }
        let (retry_limit, delays) = Self::compute_retry_delays(self)?;

        Ok((retry_limit, delays))
    }
}

/// Item outcomes recorded by the retry processor.
#[metric_set(
    name = "processor.retry.items",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct RetryItemMetrics {
    /// Number of items whose retry processing reached a terminal outcome.
    #[metric(unit = "{item}")]
    pub consumed: Counter<u64>,
    /// Number of items returned by downstream delivery attempts.
    #[metric(unit = "{item}")]
    pub produced: Counter<u64>,
}

/// Retry attempts scheduled after a downstream refusal.
#[metric_set(
    name = "processor.retry.attempts",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct RetryAttemptMetrics {
    /// Number of retry attempts scheduled.
    #[metric(unit = "{retry}")]
    pub scheduled: Counter<u64>,
}

/// Bounded-cardinality metrics emitted by a retry processor.
#[derive(Debug)]
pub struct RetryProcessorMetrics {
    items: MeasurementMetricSet<RetryItemMetrics>,
    attempts: MeasurementMetricSet<RetryAttemptMetrics>,
}

impl RetryProcessorMetrics {
    /// Registers all retry processor metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            items: RetryItemMetrics::register(pipeline_ctx),
            attempts: RetryAttemptMetrics::register(pipeline_ctx),
        }
    }

    /// Records items whose retry processing reached a terminal outcome.
    pub fn record_consumed(&mut self, signal: SignalType, outcome: Outcome, count: u64) {
        if count == 0 {
            return;
        }
        self.items
            .with(SignalOutcomeAttributes { signal, outcome })
            .consumed
            .add(count);
    }

    /// Records items returned by a downstream delivery attempt.
    pub fn record_produced(&mut self, signal: SignalType, outcome: Outcome, count: u64) {
        if count == 0 {
            return;
        }
        self.items
            .with(SignalOutcomeAttributes { signal, outcome })
            .produced
            .add(count);
    }

    /// Records a retry attempt scheduled after a downstream refusal.
    pub fn record_retry_scheduled(&mut self, signal: SignalType) {
        self.attempts
            .with(SignalAttributes { signal })
            .scheduled
            .inc();
    }

    /// Returns an item bucket for inspection without marking it for export.
    #[must_use]
    pub fn items_for(&self, signal: SignalType, outcome: Outcome) -> &RetryItemMetrics {
        self.items.get(SignalOutcomeAttributes { signal, outcome })
    }

    /// Returns a retry-attempt bucket for inspection without marking it for export.
    #[must_use]
    pub fn attempts_for(&self, signal: SignalType) -> &RetryAttemptMetrics {
        self.attempts.get(SignalAttributes { signal })
    }

    /// Reports every touched retry processor metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.items)?;
        reporter.report_measurement(&mut self.attempts)
    }
}

/// OTAP RetryProcessor
#[allow(unsafe_code)]
#[distributed_slice(otap_df_otap::OTAP_PROCESSOR_FACTORIES)]
pub static RETRY_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: RETRY_PROCESSOR_URN,
    create: create_retry_processor,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<RetryConfig>,
};

/// A processor that handles message retries with exponential backoff
///
/// This component only maintains state in the request context.
pub struct RetryProcessor {
    /// This is how many retries we can attempt in the worst case, and
    /// this is enforced so that retries would not repeat forever if the
    /// clock stopped.
    retry_limit: usize,

    /// Delays stores all the exponentially-scaled durations so that
    /// we do not repeat the f64::pow() operation for each request.
    delays: Vec<Duration>,

    config: RetryConfig,
    metrics: RetryProcessorMetrics,
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_retry_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: RetryConfig = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse retry configuration: {e}"),
        }
    })?;

    let retry = RetryProcessor::with_pipeline_ctx(pipeline_ctx, config)?;

    Ok(ProcessorWrapper::local(
        retry,
        node,
        node_config,
        processor_config,
    ))
}

fn systemtime_f64(st: SystemTime) -> f64 {
    st.duration_since(SystemTime::UNIX_EPOCH)
        .expect("epoch")
        .as_secs_f64()
}

fn now_f64() -> f64 {
    systemtime_f64(SystemTime::now())
}

/// State tracking for retry attempts, sized for Context8u8.
#[derive(Debug, Clone)]
struct RetryState {
    /// Number of retry attempts so far (0 = first attempt, 1+ = retries).
    retries: u64,

    /// Deadline for the retry operation.  Note this is an f64 because
    /// it's the only 64-bit value we can get that is lossless. The
    /// SystemTime::as_millis() and other APIs for integer fractional
    /// seconds return u128.
    deadline: f64,

    /// Item count
    num_items: u64,
}

impl RetryState {
    const fn new(num_items: u64, deadline: f64) -> Self {
        Self {
            retries: 0,
            deadline,
            num_items,
        }
    }
}

impl From<RetryState> for CallData {
    fn from(value: RetryState) -> Self {
        smallvec::smallvec![
            value.retries.into(),
            value.deadline.into(),
            value.num_items.into()
        ]
    }
}

impl TryFrom<CallData> for RetryState {
    type Error = Error;

    fn try_from(value: CallData) -> Result<Self, Self::Error> {
        if value.len() != 3 {
            return Err(Error::InternalError {
                message: "invalid calldata".into(),
            });
        }

        Ok(Self {
            retries: value[0].into(),
            deadline: value[1].into(),
            num_items: value[2].into(),
        })
    }
}

impl RetryProcessor {
    /// Creates a new RetryProcessor with metrics registered via PipelineContext
    pub fn with_pipeline_ctx(
        pipeline_ctx: PipelineContext,
        config: RetryConfig,
    ) -> Result<Self, ConfigError> {
        let metrics = RetryProcessorMetrics::register(&pipeline_ctx);

        let (retry_limit, delays) = config.validate_retries()?;

        Ok(Self {
            retry_limit,
            delays,
            config,
            metrics,
        })
    }

    async fn handle_ack(
        &mut self,
        ack: AckMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = ack.accepted.signal_type();
        let calldata = ack.unwind.route.calldata.clone();

        let num_items = match calldata.try_into() {
            Err(_err) => {
                // We don't know how many items,
                // ToDo: this shouldn't happen; how to count?
                0
            }
            Ok(RetryState { num_items, .. }) => num_items,
        };

        // The producer and consumer both succeed.
        self.metrics
            .record_produced(signal, Outcome::Success, num_items);
        self.metrics
            .record_consumed(signal, Outcome::Success, num_items);
        effect_handler.notify_ack(ack).await?;
        Ok(())
    }

    async fn handle_nack(
        &mut self,
        mut nack: NackMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = nack.refused.signal_type();

        let mut rstate: RetryState = match nack.unwind.route.calldata.clone().try_into() {
            Err(_err) => {
                // Malformed context error: we don't know what this is.
                effect_handler.notify_nack(nack).await?;
                return Ok(());
            }
            Ok(retry) => retry,
        };

        // Regardless of the next step, this NACK counts as a producer
        // refusal.
        self.metrics
            .record_produced(signal, Outcome::Refused, rstate.num_items);

        // Permanent errors should not be retried, notify the next recipient.
        if nack.permanent {
            self.metrics
                .record_consumed(signal, Outcome::Refused, rstate.num_items);
            effect_handler.notify_nack(nack).await?;
            return Ok(());
        }

        // Check for missing payload, we won't retry an empty request.
        if nack.refused.is_empty() {
            // The downstream refused the request and did not give us
            // back data to retry.
            nack.reason = format!("retry lost payload: {}", nack.reason);
            self.metrics
                .record_consumed(signal, Outcome::Refused, rstate.num_items);
            effect_handler.notify_nack(nack).await?;
            return Ok(());
        }

        // Compute the delay.
        // Limited is defined by the worst-case, where exports take 0 time.
        // If the clock is working, the deadlock check will agree with
        // this check, but this check is less expensive.
        let limited = (rstate.retries as usize) >= self.retry_limit;
        let delay = self
            .delays
            .get(rstate.retries as usize)
            .unwrap_or(&self.config.max_interval);

        if limited || rstate.deadline <= now_f64() + delay.as_secs_f64() {
            // The caller has refused, as often as we'll let them.
            nack.reason = format!("final retry: {}", nack.reason);
            self.metrics
                .record_consumed(signal, Outcome::Refused, rstate.num_items);
            effect_handler.notify_nack(nack).await?;
            return Ok(());
        }

        let now_i = Instant::now();
        let next_retry_time_i = now_i + *delay;

        rstate.retries += 1;

        // Save for use below.
        let num_items = rstate.num_items;

        // Updated RetryState back onto context for retry attempt
        let mut rereq = nack.refused;
        effect_handler.subscribe_to(
            Interests::NACKS | Interests::ACKS | Interests::RETURN_DATA,
            rstate.into(),
            &mut rereq,
        );

        // Requeue the data onto this node, we'll continue in the DelayedData branch next.
        match effect_handler.requeue_later(next_retry_time_i, rereq) {
            Ok(_) => {
                self.metrics.record_retry_scheduled(signal);
                Ok(())
            }
            Err(refused) => {
                // This component failed before it could schedule the retry.
                self.metrics
                    .record_consumed(signal, Outcome::Failure, num_items);
                effect_handler
                    .notify_nack(NackMsg::new("cannot requeue", refused))
                    .await?;
                Ok(())
            }
        }
    }

    async fn handle_delayed(
        &mut self,
        _when: Instant,
        data: Box<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
        num_items: u64,
    ) -> Result<(), Error> {
        self.send_or_nack(*data, effect_handler, num_items).await
    }

    async fn send_or_nack(
        &mut self,
        data: OtapPdata,
        effect_handler: &mut EffectHandler<OtapPdata>,
        num_items: u64,
    ) -> Result<(), Error> {
        let signal = data.signal_type();
        match effect_handler.send_message_with_source_node(data).await {
            Ok(()) => {
                // Request control flows downstream.
                Ok(())
            }
            Err(TypedError::ChannelSendError(sent)) => {
                let reason = sent.to_string();
                let data = sent.inner();
                // This component failed before it could produce the items.
                self.metrics
                    .record_consumed(signal, Outcome::Failure, num_items);
                effect_handler
                    .notify_nack(NackMsg::new(reason, data))
                    .await?;
                Ok(())
            }
            Err(e) => {
                self.metrics
                    .record_consumed(signal, Outcome::Failure, num_items);
                Err(e.into())
            }
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
            Message::PData(mut data) => {
                let num_items = data.num_items() as u64;
                if num_items == 0 {
                    // Immediately Ack an empty request. Otherwise
                    // looks like a failure to return data in the Nack
                    // code path.
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                    return Ok(());
                }

                let deadline = now_f64() + self.config.max_elapsed_time.as_secs_f64();
                effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
                    RetryState::new(num_items, deadline).into(),
                    &mut data,
                );
                self.send_or_nack(data, effect_handler, num_items).await
            }
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::Ack(ack) => self.handle_ack(ack, effect_handler).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(nack, effect_handler).await,
                NodeControlMsg::DelayedData { when, data } => {
                    if let Some(calldata) = data.source_route() {
                        let rstate: RetryState = calldata.calldata.try_into()?;
                        let _ = self
                            .handle_delayed(when, data, effect_handler, rstate.num_items)
                            .await?;
                    }
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => self
                    .metrics
                    .report(&mut metrics_reporter)
                    .map_err(|e| Error::InternalError {
                        message: e.to_string(),
                    }),
                NodeControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                NodeControlMsg::TimerTick { .. } => {
                    unreachable!("unused");
                }
                NodeControlMsg::Wakeup { .. } => Ok(()),
                NodeControlMsg::MemoryPressureChanged { .. } => Ok(()),
                NodeControlMsg::DrainIngress { .. } => Ok(()),
                NodeControlMsg::Shutdown { .. } => Ok(()),
            },
        }
    }
}

impl RetryProcessor {
    /// Creates a new RetryProcessor with the specified configuration
    #[must_use]
    #[cfg(test)]
    pub fn with_config(config: RetryConfig) -> Self {
        let telemetry_registry = otap_df_telemetry::registry::TelemetryRegistryHandle::default();
        let controller = otap_df_engine::context::ControllerContext::new(telemetry_registry);
        let pipeline_ctx = controller.pipeline_context_with("test".into(), "retry".into(), 0, 1, 0);
        let metrics = RetryProcessorMetrics::register(&pipeline_ctx);

        let (retry_limit, delays) = config.validate_retries().expect("valid");
        Self {
            retry_limit,
            delays,
            config,
            metrics,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Outcome, RETRY_PROCESSOR_URN, RetryConfig, RetryProcessorMetrics, SignalType};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_engine::control::{
        AckMsg, NackMsg, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
    };
    use otap_df_engine::testing::liveness::next_completion;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::{Interests, message::Message};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::{TestCallData, create_test_pdata, next_ack, next_nack};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Scenario: Retry metrics record terminal item outcomes and scheduled attempts across signals.
    /// Guarantees: Counts are isolated by the shared signal and outcome enum attributes.
    #[test]
    fn retry_metrics_are_partitioned_by_signal_and_outcome() {
        let mut metrics = RetryProcessorMetrics::register(&create_test_pipeline_context());
        metrics.record_consumed(SignalType::Logs, Outcome::Success, 3);
        metrics.record_consumed(SignalType::Logs, Outcome::Refused, 2);
        metrics.record_consumed(SignalType::Metrics, Outcome::Failure, 1);
        metrics.record_produced(SignalType::Logs, Outcome::Success, 3);
        metrics.record_produced(SignalType::Logs, Outcome::Refused, 4);
        metrics.record_retry_scheduled(SignalType::Traces);
        metrics.record_retry_scheduled(SignalType::Traces);

        assert_eq!(
            metrics
                .items_for(SignalType::Logs, Outcome::Success)
                .consumed
                .get(),
            3
        );
        assert_eq!(
            metrics
                .items_for(SignalType::Logs, Outcome::Success)
                .produced
                .get(),
            3
        );
        assert_eq!(
            metrics
                .items_for(SignalType::Logs, Outcome::Refused)
                .consumed
                .get(),
            2
        );
        assert_eq!(
            metrics
                .items_for(SignalType::Logs, Outcome::Refused)
                .produced
                .get(),
            4
        );
        assert_eq!(
            metrics
                .items_for(SignalType::Metrics, Outcome::Failure)
                .consumed
                .get(),
            1
        );
        assert_eq!(
            metrics
                .items_for(SignalType::Traces, Outcome::Success)
                .consumed
                .get(),
            0
        );
        assert_eq!(metrics.attempts_for(SignalType::Traces).scheduled.get(), 2);
        assert_eq!(metrics.attempts_for(SignalType::Logs).scheduled.get(), 0);
    }

    /// Scenario: Retry metric buckets are converted to snapshots after recording several dimensions.
    /// Guarantees: Snapshot descriptors carry the expected bounded enum wire values.
    #[test]
    fn retry_metric_snapshots_preserve_enum_attribute_values() {
        let mut metrics = RetryProcessorMetrics::register(&create_test_pipeline_context());
        metrics.record_consumed(SignalType::Metrics, Outcome::Failure, 4);
        metrics.record_produced(SignalType::Logs, Outcome::Refused, 2);
        metrics.record_retry_scheduled(SignalType::Traces);

        let mut snapshots = metrics.items.terminal_snapshots();
        snapshots.extend(metrics.attempts.terminal_snapshots());

        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "processor.retry.items"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("outcome") == Some("failure")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "processor.retry.items"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("refused")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "processor.retry.attempts"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
        }));
    }

    /// Scenario: An empty retry configuration is deserialized.
    /// Guarantees: Every retry setting uses its documented default value.
    #[test]
    fn test_default_config() {
        let cfg: RetryConfig = serde_json::from_value(json!({})).unwrap();
        assert_eq!(cfg, RetryConfig::default());
    }

    /// Scenario: Retry intervals contain valid fractional-second values.
    /// Guarantees: Deserialization preserves subsecond precision and the configured multiplier.
    #[test]
    fn test_tiny_config() {
        let cfg: RetryConfig = serde_json::from_value(json!({
            "initial_interval": "0.5s",
            "max_interval": "1.75s",
            "max_elapsed_time": "9.9s",
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

    /// Scenario: Retry configurations contain zero intervals, a small multiplier, or excessive growth.
    /// Guarantees: Validation rejects each invalid configuration with a relevant error.
    #[test]
    fn test_invalid_config() {
        for (value, expect) in [
            (
                json!({
                    "initial_interval": "0s",
                }),
                "initial",
            ),
            (
                json!({
                    "max_interval": "0h",
                }),
                "max",
            ),
            (
                json!({
                    "max_elapsed_time": "0m",
                }),
                "elapsed",
            ),
            (
                json!({
                    "multiplier": 0.75,
                }),
                "multiplier",
            ),
            (
                json!({
                    "initial_interval": "1s",
                    "max_interval": "1m",
                    "max_elapsed_time": "1h",
                    "multiplier": 1.0001,
                }),
                "retry growth",
            ),
        ] {
            let res = serde_json::from_value::<RetryConfig>(value)
                .unwrap()
                .validate_retries();
            let err = res.expect_err("has error");
            assert!(
                err.to_string().contains(expect),
                "{err:?} should contain {expect}"
            );
        }
    }

    /// Creates a test pipeline context for testing
    fn create_test_pipeline_context() -> PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
    }

    fn create_test_config() -> serde_json::Value {
        // These settings are designed for 3 retries:
        // 1st retry: +0.05=+0.05 retry_count=1
        // 2nd retry: +0.10=+0.15 retry_count=2
        // 3nd retry: +0.20=+0.35 retry_count=3
        // 4nd retry: +0.40=+0.75 max_elapsed reached
        json!({
            "initial_interval": "0.05s",     // 50ms initial delay
            "max_interval": "0.40s",         // 400ms max delay
            "max_elapsed_time": "0.5s",      // 500ms total timeout
            "multiplier": 2.0,            // Double
        })
    }

    /// Scenario: Zero to two transient NACKs precede a downstream ACK with working or stalled time.
    /// Guarantees: Every request eventually succeeds when it remains within the retry limit.
    #[test]
    fn test_retry_processor_nacks_then_success_time() {
        // For the success case, we expect success with or without a
        // working clock.  Test both ways.
        for i in 0..3 {
            test_retry_processor(create_test_config(), i, None, true, false)
        }
        for i in 0..3 {
            test_retry_processor(create_test_config(), i, None, false, false)
        }
    }

    /// Scenario: Transient downstream NACKs exhaust the configured elapsed-time window.
    /// Guarantees: The processor returns a terminal NACK instead of scheduling another retry.
    #[test]
    fn test_retry_processor_nacks_then_timeout() {
        test_retry_processor(
            create_test_config(),
            4,
            Some("final retry: simulated downstream".into()),
            true,  // working clock
            false, // retryable
        )
    }

    /// Scenario: Downstream returns a permanent NACK on the first delivery attempt.
    /// Guarantees: The processor forwards the NACK without scheduling a retry.
    #[test]
    fn test_retry_processor_permanent_error_not_retried() {
        test_retry_processor(
            create_test_config(),
            1,
            Some("simulated permanent".into()),
            true,
            true, // permanent error
        )
    }

    /// Scenario: A stalled wall clock allows NACKs to reach the computed retry-count limit.
    /// Guarantees: The hard retry limit still produces a terminal NACK.
    #[test]
    fn test_retry_processor_nacks_then_limit() {
        test_retry_processor(
            create_test_config(),
            4,
            Some("final retry: simulated".into()),
            // this places emphasis on the logical limit, not the
            // max-elapsed walltime.
            false, // broken clock
            false, // retryable
        )
    }

    /// Scenario: A transient downstream NACK does not return the original payload.
    /// Guarantees: The processor emits a terminal NACK immediately without retrying empty data.
    #[test]
    fn test_retry_processor_missing_return_data_nacks_without_retry() {
        let pipeline_ctx = create_test_pipeline_context();
        let node = test_node("retry-processor-missing-return-data");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();

        let mut node_config = NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN);
        node_config.config = create_test_config();

        let proc = crate::processors::retry_processor::create_retry_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
            &otap_df_engine::capability::registry::Capabilities::empty(),
        )
        .expect("create processor");

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let pdata_in = create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    4444,
                );

                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process initial message");

                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                let mut first_attempt = output.remove(0);
                let _ = first_attempt.take_payload();

                let (_, nack_msg) = next_nack(NackMsg::new("missing payload", first_attempt))
                    .expect("expected nack subscriber");
                ctx.process(Message::nack_ctrl_msg(nack_msg))
                    .await
                    .expect("process nack");

                match next_completion(
                    &mut pipeline_completion_rx,
                    Duration::from_secs(1),
                    "retry processor terminal nack for missing payload",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        let (node_id, nack) = next_nack(nack).expect("expected nack subscriber");
                        assert_eq!(node_id, 4444);
                        assert!(
                            nack.reason.contains("retry lost payload"),
                            "unexpected reason: {}",
                            nack.reason
                        );
                        let calldata: TestCallData =
                            nack.unwind.route.calldata.try_into().expect("my calldata");
                        assert_eq!(TestCallData::default(), calldata);
                    }
                    other => panic!("expected terminal nack, got {other:?}"),
                }
            })
            .validate(|ctx| async move {
                ctx.counters().assert(0, 0, 0, 0);
            });
    }

    /// Scenario: retry receives a transient downstream NACK while the
    /// processor-local delayed-resume queue is already full.
    /// Guarantees: the processor emits a terminal NACK and counts only the retry that was scheduled.
    #[test]
    fn test_retry_processor_cannot_requeue_becomes_terminal_nack() {
        let pipeline_ctx = create_test_pipeline_context();
        let node = test_node("retry-processor-cannot-delay");
        let rt: TestRuntime<OtapPdata> = TestRuntime::with_channel_capacities(10, 1);

        let mut node_config = NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN);
        node_config.config = create_test_config();

        let proc = crate::processors::retry_processor::create_retry_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
            &otap_df_engine::capability::registry::Capabilities::empty(),
        )
        .expect("create processor");

        rt.set_processor(proc)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let pdata_in = create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
                    TestCallData::default().into(),
                    4444,
                );

                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process initial message");

                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                let first_attempt = output.remove(0);

                let (_, nack_msg) =
                    next_nack(NackMsg::new("simulated downstream failure", first_attempt))
                        .expect("expected nack subscriber");
                ctx.process(Message::nack_ctrl_msg(nack_msg))
                    .await
                    .expect("process nack");

                assert!(
                    ctx.next_local_control_deadline().is_some(),
                    "first retry should occupy the only local requeue slot"
                );

                let second_input = create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
                    TestCallData::default().into(),
                    4444,
                );
                ctx.process(Message::PData(second_input))
                    .await
                    .expect("process second input");

                let mut output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1);
                let second_attempt = output.remove(0);

                let (_, nack_msg) =
                    next_nack(NackMsg::new("simulated downstream failure", second_attempt))
                        .expect("expected nack subscriber");
                ctx.process(Message::nack_ctrl_msg(nack_msg))
                    .await
                    .expect("process second nack");

                match next_completion(
                    &mut pipeline_completion_rx,
                    Duration::from_secs(1),
                    "retry processor terminal nack when requeue_later fails",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        let (_node_id, nack) = next_nack(nack).expect("expected nack subscriber");
                        assert!(
                            nack.reason.contains("cannot requeue"),
                            "unexpected reason: {}",
                            nack.reason
                        );
                    }
                    other => panic!("expected terminal nack, got {other:?}"),
                }

                let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(3);
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect retry metrics");

                let mut scheduled = 0;
                let mut failed_items = 0;
                let mut refused_items = 0;
                for snapshot in metrics_rx.try_iter() {
                    if snapshot.descriptor().name == "processor.retry.attempts"
                        && snapshot.measurement_attribute_value("signal") == Some("logs")
                    {
                        scheduled += snapshot.get_metrics()[0].to_u64_lossy();
                    }
                    if snapshot.descriptor().name == "processor.retry.items"
                        && snapshot.measurement_attribute_value("signal") == Some("logs")
                    {
                        match snapshot.measurement_attribute_value("outcome") {
                            Some("failure") => {
                                failed_items += snapshot.get_metrics()[0].to_u64_lossy();
                            }
                            Some("refused") => {
                                refused_items += snapshot.get_metrics()[1].to_u64_lossy();
                            }
                            _ => {}
                        }
                    }
                }
                assert_eq!(scheduled, 1);
                assert_eq!(failed_items, 1);
                assert_eq!(refused_items, 2);
            })
            .validate(|ctx| async move {
                ctx.counters().assert(0, 0, 0, 0);
            });
    }

    fn test_retry_processor(
        config: serde_json::Value,
        number_of_nacks: usize,
        outcome_failure: Option<String>,
        working_clock: bool,
        permanent_error: bool,
    ) {
        let pipeline_ctx = create_test_pipeline_context();
        let node = test_node("retry-processor-full-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();

        let mut node_config = NodeUserConfig::new_processor_config(RETRY_PROCESSOR_URN);
        node_config.config = config;

        let proc = crate::processors::retry_processor::create_retry_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
            &otap_df_engine::capability::registry::Capabilities::empty(),
        )
        .expect("create processor");

        let phase = rt.set_processor(proc);

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut retry_count: usize = 0;
                let pdata_in = create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
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
                // have_pmsg is the first non-requeue completion message
                // received in the loop, this will happen when
                // number_of_nacks is 4, i.e., the nack before the
                // final retry attempt.
                let mut have_pmsg: Option<PipelineCompletionMsg<OtapPdata>> = None;
                let mut nacks_delivered = 0;
                while nacks_delivered < number_of_nacks {
                    let nack = if permanent_error {
                        NackMsg::new_permanent("simulated permanent failure", current_data.clone())
                    } else {
                        NackMsg::new("simulated downstream failure", current_data.clone())
                    };

                    let (_, nack_msg) = next_nack(nack).unwrap();

                    ctx.process(Message::nack_ctrl_msg(nack_msg)).await.unwrap();
                    nacks_delivered += 1;

                    let resp = if let Some(when) = ctx.next_local_control_deadline() {
                        retry_count += 1;

                        if working_clock {
                            ctx.sleep(
                                when.checked_duration_since(Instant::now())
                                    .unwrap_or_default(),
                            )
                            .await;
                        }

                        let control = ctx
                            .take_due_local_control(when)
                            .expect("scheduled local control");
                        assert!(
                            matches!(control, NodeControlMsg::DelayedData { .. }),
                            "retry should requeue retained pdata as DelayedData"
                        );
                        ctx.process(Message::Control(control)).await.unwrap();

                        let mut retry_output = ctx.drain_pdata().await;
                        assert_eq!(retry_output.len(), 1);
                        current_data = retry_output.remove(0);
                        None
                    } else {
                        Some(
                            pipeline_completion_rx
                                .recv()
                                .await
                                .expect("pipeline-completion channel closed unexpectedly"),
                        )
                    };
                    have_pmsg = have_pmsg.or(resp);
                }

                if have_pmsg.is_none() {
                    // Send final ACK or NACK
                    if let Some(message) = &outcome_failure {
                        let nack = NackMsg::new(format!("TEST {} FAILED", message), current_data);
                        let (_, nack_msg) = next_nack(nack).unwrap();
                        ctx.process(Message::nack_ctrl_msg(nack_msg)).await.unwrap();
                    } else {
                        let ack = AckMsg::new(current_data);
                        let (_, ack_msg) = next_ack(ack).unwrap();
                        ctx.process(Message::ack_ctrl_msg(ack_msg)).await.unwrap();
                    }

                    // Verify the processor sent the ACK or NACK upstream
                    have_pmsg = Some(
                        tokio::time::timeout(Duration::from_secs(1), pipeline_completion_rx.recv())
                            .await
                            .expect("timeout waiting for final DeliverAck")
                            .expect("channel closed"),
                    );
                }

                match have_pmsg.expect("retry replied") {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        let (node_id, ack) = next_ack(ack).expect("expected ack subscriber");
                        assert!(
                            outcome_failure.is_none(),
                            "expecting Nack {outcome_failure:?}, got Ack"
                        );
                        assert_eq!(node_id, 4444);

                        let ackdata: TestCallData =
                            ack.unwind.route.calldata.try_into().expect("my calldata");
                        assert_eq!(TestCallData::default(), ackdata);

                        // Requested RETURN_DATA, check item count match
                        assert_eq!(create_test_pdata().num_items(), ack.accepted.num_items());
                    }
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        let (node_id, nack) = next_nack(nack).expect("expected nack subscriber");
                        assert!(
                            nack.reason
                                .contains(&outcome_failure.expect("expecting nack"))
                        );
                        assert_eq!(node_id, 4444);

                        let nackdata: TestCallData =
                            nack.unwind.route.calldata.try_into().expect("my calldata");
                        assert_eq!(TestCallData::default(), nackdata);

                        // Requested RETURN_DATA, check item count match
                        assert_eq!(create_test_pdata().num_items(), nack.refused.num_items());
                    }
                }

                // With 0-3 Nacks, we retry every time. On the 4th Nack, this changes.
                // Permanent errors are never retried.
                let expected_retries = if permanent_error {
                    0
                } else {
                    std::cmp::min(nacks_delivered, 3)
                };
                assert_eq!(expected_retries, retry_count);
                assert_eq!(nacks_delivered, number_of_nacks);
            })
            .validate(|ctx| async move {
                // Verify no unexpected control message processing
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }
}
