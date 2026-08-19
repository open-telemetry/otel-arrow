// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Channel-oriented metrics for the OTAP engine.
//!
//! Metrics are split by endpoint role (sender vs receiver). All metrics are scoped
//! using channel endpoint attributes and can be correlated using `channel.id`
//! and `channel.kind`.

use crate::attributes::ChannelKind;
use otap_df_config::SignalType;
use otap_df_telemetry::attributes::AttributeEnum as _;
use otap_df_telemetry::common_attributes::{
    Outcome, OutcomeAttributes, SignalAttributes, SignalOutcomeAttributes,
};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otap_df_telemetry::registry::MetricSetKey;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

/// Represents the outcome of a request for metrics recording.
///
/// Used to consolidate success/failure/refused counter updates into a single
/// method call, reducing code duplication in both producer and consumer metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestOutcome {
    /// Request completed successfully (ack received).
    Success,
    /// Request failed but may be retried (transient / retryable, non-permanent nack).
    Failure,
    /// Request was permanently refused (permanent nack).
    Refused,
}

/// Actionable category for a channel send failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(crate) enum ChannelSendErrorType {
    /// The bounded channel had no capacity for a non-blocking send.
    Full,
    /// The receiving endpoint was closed.
    Closed,
}

/// Error classification for a control-channel send failure.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ChannelSendFailureAttributes {
    /// Bounded failure category.
    #[attribute_key = "error.type"]
    pub(crate) error_type: ChannelSendErrorType,
}

/// Signal and error classification for a PData channel send failure.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct SignalChannelSendFailureAttributes {
    /// Pipeline signal carried by the refused or failed message.
    pub(crate) signal: SignalType,
    /// Bounded failure category.
    #[attribute_key = "error.type"]
    pub(crate) error_type: ChannelSendErrorType,
}

/// PData channel send attempts grouped by signal and terminal local outcome.
#[metric_set(
    name = "channel.sender",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ChannelSenderMetrics {
    /// Messages whose immediate channel send attempt terminated.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// PData channel send failures grouped by signal and actionable error category.
#[metric_set(
    name = "channel.sender",
    measurement_attributes = SignalChannelSendFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub(crate) struct ChannelSenderFailureMetrics {
    /// Failed or refused immediate send attempts.
    #[metric(unit = "{message}")]
    pub(crate) failures: Counter<u64>,
}

/// Control-channel send attempts grouped by terminal local outcome.
#[metric_set(
    name = "channel.sender",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub(crate) struct ControlChannelSenderMetrics {
    /// Control messages whose immediate channel send attempt terminated.
    #[metric(unit = "{message}")]
    pub(crate) messages: Counter<u64>,
}

/// Control-channel send failures grouped by actionable error category.
#[metric_set(
    name = "channel.sender",
    measurement_attributes = ChannelSendFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub(crate) struct ControlChannelSenderFailureMetrics {
    /// Failed or refused immediate control-message send attempts.
    #[metric(unit = "{message}")]
    pub(crate) failures: Counter<u64>,
}

/// Successfully dequeued PData messages grouped by signal.
#[metric_set(
    name = "channel.receiver",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ChannelReceiverMetrics {
    /// Messages successfully dequeued from the channel.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// Current state of a PData channel queue from the receiver endpoint perspective.
#[metric_set(name = "channel.receiver")]
#[derive(Debug, Default, Clone)]
pub(crate) struct ChannelReceiverStateMetrics {
    /// Current number of messages buffered in the channel.
    #[metric(unit = "{message}")]
    pub(crate) queue_depth: Gauge<u64>,
    /// Configured maximum number of buffered messages.
    #[metric(unit = "{message}")]
    pub(crate) capacity: Gauge<u64>,
}

/// Successfully dequeued control messages and current queue state.
#[metric_set(name = "channel.receiver")]
#[derive(Debug, Default, Clone)]
pub(crate) struct ControlChannelReceiverMetrics {
    /// Control messages successfully dequeued from the channel.
    #[metric(unit = "{message}")]
    pub(crate) messages: Counter<u64>,
    /// Current number of control messages buffered in the channel.
    #[metric(unit = "{message}")]
    pub(crate) queue_depth: Gauge<u64>,
    /// Configured maximum number of buffered control messages.
    #[metric(unit = "{message}")]
    pub(crate) capacity: Gauge<u64>,
}

/// Ack/nack metrics for consumed messages, owned exclusively by the runtime control manager.
/// Registered under the input channel entity key so they share the same
/// channel attributes as the transport metrics.
#[metric_set(
    name = "node.consumer",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ConsumedMetrics {
    /// Duration from entry until the corresponding ack or nack is
    /// routed, in nanoseconds. This is reported at the detailed level.
    ///
    /// TODO: make this Option<Box<Mmsc or Histogram>>.
    #[metric(name = "consumed.duration", unit = "ns")]
    pub consumed_duration_ns: Mmsc,
    /// Consumed messages, grouped by `signal` and `outcome` datapoint attributes.
    #[metric(name = "consumed.messages", unit = "{message}")]
    pub consumed_messages: Counter<u64>,
}

/// Optional per-signal item metrics for a node input channel.
#[metric_set(
    name = "node.consumer",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ConsumedItemMetrics {
    /// Consumed signal items, grouped by the `signal` datapoint attribute.
    #[metric(name = "consumed.items", unit = "{item}")]
    pub consumed_items: Counter<u64>,
}

/// Ack/nack metrics for produced messages, owned exclusively by the runtime control manager.
/// Registered under the output channel entity key so they share the same
/// channel attributes as the transport metrics.
#[metric_set(
    name = "node.producer",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ProducedMetrics {
    /// Duration from production until the corresponding ack or nack is
    /// routed, in nanoseconds. This is reported at the detailed level,
    /// only in receivers. Processors report `consumed.messages`.
    ///
    /// TODO: make this Option<Box<Mmsc or Histogram>>.
    #[metric(name = "produced.duration", unit = "ns")]
    pub produced_duration_ns: Mmsc,
    /// Produced messages, grouped by `signal` and `outcome` datapoint attributes.
    #[metric(name = "produced.messages", unit = "{message}")]
    pub produced_messages: Counter<u64>,
}

/// Optional per-signal item metrics for a node output channel.
#[metric_set(
    name = "node.producer",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ProducedItemMetrics {
    /// Produced signal items, grouped by the `signal` datapoint attribute.
    #[metric(name = "produced.items", unit = "{item}")]
    pub produced_items: Counter<u64>,
}

pub(crate) fn control_channel_id(name: &str) -> Cow<'static, str> {
    format!("{}:{}", name, ChannelKind::Control.as_str()).into()
}

pub(crate) struct PdataChannelSenderMetricSets {
    pub(crate) messages: MeasurementMetricSet<ChannelSenderMetrics>,
    pub(crate) failures: MeasurementMetricSet<ChannelSenderFailureMetrics>,
}

impl PdataChannelSenderMetricSets {
    pub(crate) fn metric_set_keys(&self) -> [MetricSetKey; 2] {
        [
            self.messages.metric_set_key(),
            self.failures.metric_set_key(),
        ]
    }
}

pub(crate) struct ControlChannelSenderMetricSets {
    pub(crate) messages: MeasurementMetricSet<ControlChannelSenderMetrics>,
    pub(crate) failures: MeasurementMetricSet<ControlChannelSenderFailureMetrics>,
}

pub(crate) struct PdataChannelReceiverMetricSets {
    pub(crate) messages: MeasurementMetricSet<ChannelReceiverMetrics>,
    pub(crate) state: MetricSet<ChannelReceiverStateMetrics>,
}

impl PdataChannelReceiverMetricSets {
    pub(crate) fn metric_set_keys(&self) -> [MetricSetKey; 2] {
        [self.messages.metric_set_key(), self.state.metric_set_key()]
    }
}

pub(crate) struct ControlChannelReceiverMetricSets {
    pub(crate) metrics: MetricSet<ControlChannelReceiverMetrics>,
}

pub(crate) enum ChannelSenderMetricSets {
    Pdata(PdataChannelSenderMetricSets),
    Control(ControlChannelSenderMetricSets),
}

pub(crate) enum ChannelReceiverMetricSets {
    Pdata(PdataChannelReceiverMetricSets),
    Control(ControlChannelReceiverMetricSets),
}

pub(crate) struct ChannelSenderMetricsState {
    metrics: ChannelSenderMetricSets,
}

impl ChannelSenderMetricsState {
    pub(crate) const fn new(metrics: ChannelSenderMetricSets) -> Self {
        Self { metrics }
    }

    #[inline]
    pub(crate) fn record_send_ok(&mut self, signal: Option<SignalType>) {
        match &mut self.metrics {
            ChannelSenderMetricSets::Pdata(metrics) => {
                if let Some(signal) = signal {
                    metrics
                        .messages
                        .with(SignalOutcomeAttributes {
                            signal,
                            outcome: Outcome::Success,
                        })
                        .messages
                        .inc();
                }
            }
            ChannelSenderMetricSets::Control(metrics) => metrics
                .messages
                .with(OutcomeAttributes {
                    outcome: Outcome::Success,
                })
                .messages
                .inc(),
        }
    }

    #[inline]
    pub(crate) fn record_send_error(
        &mut self,
        signal: Option<SignalType>,
        error_type: ChannelSendErrorType,
    ) {
        let outcome = match error_type {
            ChannelSendErrorType::Full => Outcome::Refused,
            ChannelSendErrorType::Closed => Outcome::Failure,
        };
        match &mut self.metrics {
            ChannelSenderMetricSets::Pdata(metrics) => {
                if let Some(signal) = signal {
                    metrics
                        .messages
                        .with(SignalOutcomeAttributes { signal, outcome })
                        .messages
                        .inc();
                    metrics
                        .failures
                        .with(SignalChannelSendFailureAttributes { signal, error_type })
                        .failures
                        .inc();
                }
            }
            ChannelSenderMetricSets::Control(metrics) => {
                metrics
                    .messages
                    .with(OutcomeAttributes { outcome })
                    .messages
                    .inc();
                metrics
                    .failures
                    .with(ChannelSendFailureAttributes { error_type })
                    .failures
                    .inc();
            }
        }
    }

    #[inline]
    pub(crate) fn report(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        match &mut self.metrics {
            ChannelSenderMetricSets::Pdata(metrics) => {
                metrics_reporter.report_measurement(&mut metrics.messages)?;
                metrics_reporter.report_measurement(&mut metrics.failures)
            }
            ChannelSenderMetricSets::Control(metrics) => {
                metrics_reporter.report_measurement(&mut metrics.messages)?;
                metrics_reporter.report_measurement(&mut metrics.failures)
            }
        }
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = Vec::new();
        match &mut self.metrics {
            ChannelSenderMetricSets::Pdata(metrics) => {
                snapshots.extend(metrics.messages.terminal_snapshots());
                snapshots.extend(metrics.failures.terminal_snapshots());
            }
            ChannelSenderMetricSets::Control(metrics) => {
                snapshots.extend(metrics.messages.terminal_snapshots());
                snapshots.extend(metrics.failures.terminal_snapshots());
            }
        }
        snapshots
    }
}

pub(crate) trait ChannelQueueDepth: Clone {
    fn record_send(&self);
    fn record_receive(&self);
    fn current(&self) -> u64;
}

#[derive(Clone, Default)]
pub(crate) struct LocalChannelQueueDepth(Rc<Cell<i64>>);

impl ChannelQueueDepth for LocalChannelQueueDepth {
    fn record_send(&self) {
        self.0.set(self.0.get().saturating_add(1));
    }

    fn record_receive(&self) {
        self.0.set(self.0.get().saturating_sub(1));
    }

    fn current(&self) -> u64 {
        self.0.get().max(0) as u64
    }
}

#[derive(Clone, Default)]
pub(crate) struct SharedChannelQueueDepth(Arc<AtomicI64>);

impl ChannelQueueDepth for SharedChannelQueueDepth {
    fn record_send(&self) {
        let _ = self.0.fetch_add(1, Ordering::Relaxed);
    }

    fn record_receive(&self) {
        let _ = self.0.fetch_sub(1, Ordering::Relaxed);
    }

    fn current(&self) -> u64 {
        self.0.load(Ordering::Relaxed).max(0) as u64
    }
}

pub(crate) struct ChannelReceiverMetricsState<Q> {
    metrics: ChannelReceiverMetricSets,
    capacity: u64,
    queue_depth: Q,
}

impl<Q: ChannelQueueDepth> ChannelReceiverMetricsState<Q> {
    pub(crate) const fn new(
        metrics: ChannelReceiverMetricSets,
        capacity: u64,
        queue_depth: Q,
    ) -> Self {
        Self {
            metrics,
            capacity,
            queue_depth,
        }
    }

    #[inline]
    pub(crate) fn record_recv_ok(&mut self, signal: Option<SignalType>) {
        match &mut self.metrics {
            ChannelReceiverMetricSets::Pdata(metrics) => {
                if let Some(signal) = signal {
                    metrics
                        .messages
                        .with(SignalAttributes { signal })
                        .messages
                        .inc();
                }
            }
            ChannelReceiverMetricSets::Control(metrics) => metrics.metrics.messages.inc(),
        }
    }

    fn update_state_metrics(&mut self) {
        let queue_depth = self.queue_depth.current().min(self.capacity);
        match &mut self.metrics {
            ChannelReceiverMetricSets::Pdata(metrics) => {
                metrics.state.queue_depth.set(queue_depth);
                metrics.state.capacity.set(self.capacity);
            }
            ChannelReceiverMetricSets::Control(metrics) => {
                metrics.metrics.queue_depth.set(queue_depth);
                metrics.metrics.capacity.set(self.capacity);
            }
        }
    }

    #[inline]
    pub(crate) fn report(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        self.update_state_metrics();
        match &mut self.metrics {
            ChannelReceiverMetricSets::Pdata(metrics) => {
                metrics_reporter.report_measurement(&mut metrics.messages)?;
                metrics_reporter.report(&mut metrics.state)
            }
            ChannelReceiverMetricSets::Control(metrics) => {
                metrics_reporter.report(&mut metrics.metrics)
            }
        }
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.update_state_metrics();
        match &mut self.metrics {
            ChannelReceiverMetricSets::Pdata(metrics) => {
                let mut snapshots = metrics.messages.terminal_snapshots();
                snapshots.extend(metrics.state.terminal_snapshots());
                snapshots
            }
            ChannelReceiverMetricSets::Control(metrics) => metrics.metrics.terminal_snapshots(),
        }
    }
}

pub(crate) type LocalChannelSenderMetricsHandle = Rc<RefCell<ChannelSenderMetricsState>>;
pub(crate) type LocalChannelReceiverMetricsHandle =
    Rc<RefCell<ChannelReceiverMetricsState<LocalChannelQueueDepth>>>;
pub(crate) type SharedChannelSenderMetricsHandle = Arc<Mutex<ChannelSenderMetricsState>>;
pub(crate) type SharedChannelReceiverMetricsHandle =
    Arc<Mutex<ChannelReceiverMetricsState<SharedChannelQueueDepth>>>;

#[derive(Clone)]
pub(crate) enum ChannelMetricsHandle {
    LocalSender(LocalChannelSenderMetricsHandle),
    SharedSender(SharedChannelSenderMetricsHandle),
    LocalReceiver(LocalChannelReceiverMetricsHandle),
    SharedReceiver(SharedChannelReceiverMetricsHandle),
}

impl ChannelMetricsHandle {
    #[inline]
    pub(crate) fn report(
        &self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        match self {
            ChannelMetricsHandle::LocalSender(metrics) => match metrics.try_borrow_mut() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::SharedSender(metrics) => match metrics.try_lock() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::LocalReceiver(metrics) => match metrics.try_borrow_mut() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
            ChannelMetricsHandle::SharedReceiver(metrics) => match metrics.try_lock() {
                Ok(mut metrics) => metrics.report(metrics_reporter),
                Err(_) => Ok(()),
            },
        }
    }

    pub(crate) fn terminal_snapshots(&self) -> Vec<MetricSetSnapshot> {
        match self {
            ChannelMetricsHandle::LocalSender(metrics) => metrics
                .try_borrow_mut()
                .map_or_else(|_| Vec::new(), |mut metrics| metrics.terminal_snapshots()),
            ChannelMetricsHandle::SharedSender(metrics) => metrics
                .try_lock()
                .map_or_else(|_| Vec::new(), |mut metrics| metrics.terminal_snapshots()),
            ChannelMetricsHandle::LocalReceiver(metrics) => metrics
                .try_borrow_mut()
                .map_or_else(|_| Vec::new(), |mut metrics| metrics.terminal_snapshots()),
            ChannelMetricsHandle::SharedReceiver(metrics) => metrics
                .try_lock()
                .map_or_else(|_| Vec::new(), |mut metrics| metrics.terminal_snapshots()),
        }
    }
}

#[derive(Default)]
pub(crate) struct ChannelMetricsRegistry {
    handles: Vec<ChannelMetricsHandle>,
}

impl ChannelMetricsRegistry {
    pub(crate) fn register(&mut self, handle: ChannelMetricsHandle) {
        self.handles.push(handle);
    }

    pub(crate) fn into_handles(self) -> Vec<ChannelMetricsHandle> {
        self.handles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{ChannelImplementation, ChannelMode, ChannelType};
    use crate::context::{ControllerContext, PipelineContext};
    use crate::local::message::{LocalReceiver, LocalSender};
    use otap_df_channel::error::{RecvError, SendError};
    use otap_df_channel::mpsc;
    use otap_df_config::node::NodeKind;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct TestMessage {
        signal: SignalType,
    }

    fn test_signal(message: &TestMessage) -> Option<SignalType> {
        Some(message.signal)
    }

    fn test_context() -> PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx
            .pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0)
            .with_node_context(
                "node".into(),
                "urn:test:receiver:example".into(),
                NodeKind::Receiver,
                HashMap::new(),
            )
    }

    fn pdata_sender_metrics(
        pipeline_ctx: &PipelineContext,
        entity_key: otap_df_telemetry::registry::EntityKey,
    ) -> ChannelSenderMetricSets {
        ChannelSenderMetricSets::Pdata(PdataChannelSenderMetricSets {
            messages: pipeline_ctx.register_measurement_metric_set_for_entity(entity_key),
            failures: pipeline_ctx.register_measurement_metric_set_for_entity(entity_key),
        })
    }

    fn pdata_receiver_metrics(
        pipeline_ctx: &PipelineContext,
        entity_key: otap_df_telemetry::registry::EntityKey,
    ) -> ChannelReceiverMetricSets {
        ChannelReceiverMetricSets::Pdata(PdataChannelReceiverMetricSets {
            messages: pipeline_ctx.register_measurement_metric_set_for_entity(entity_key),
            state: pipeline_ctx.register_metric_set_for_entity(entity_key),
        })
    }

    fn take_local_sender_handle(
        handles: &[ChannelMetricsHandle],
    ) -> LocalChannelSenderMetricsHandle {
        handles
            .iter()
            .find_map(|handle| match handle {
                ChannelMetricsHandle::LocalSender(handle) => Some(handle.clone()),
                _ => None,
            })
            .expect("missing local sender metrics handle")
    }

    fn take_local_receiver_handle(
        handles: &[ChannelMetricsHandle],
    ) -> LocalChannelReceiverMetricsHandle {
        handles
            .iter()
            .find_map(|handle| match handle {
                ChannelMetricsHandle::LocalReceiver(handle) => Some(handle.clone()),
                _ => None,
            })
            .expect("missing local receiver metrics handle")
    }

    /// Scenario: PData sends succeed, encounter capacity backpressure, and find a closed receiver.
    /// Guarantees: Send outcomes and failure categories remain isolated by bounded signal buckets.
    #[test]
    fn channel_sender_metrics_record_send_outcomes() {
        let pipeline_ctx = test_context();
        let mut registry = ChannelMetricsRegistry::default();
        let (sender, receiver) = mpsc::Channel::new(1);
        let channel_entity_key = pipeline_ctx.register_node_channel_entity(
            "test:sender".into(),
            "out".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        let queue_depth = LocalChannelQueueDepth::default();
        let sender = LocalSender::mpsc_with_metrics(
            sender,
            &mut registry,
            pdata_sender_metrics(&pipeline_ctx, channel_entity_key),
            queue_depth,
            Some(test_signal),
        );
        sender
            .try_send(TestMessage {
                signal: SignalType::Logs,
            })
            .unwrap();
        assert!(matches!(
            sender.try_send(TestMessage {
                signal: SignalType::Metrics,
            }),
            Err(SendError::Full(_))
        ));
        drop(receiver);
        assert!(matches!(
            sender.try_send(TestMessage {
                signal: SignalType::Traces,
            }),
            Err(SendError::Closed(_))
        ));

        let handles = registry.into_handles();
        let sender_handle = take_local_sender_handle(&handles);
        let metrics = sender_handle.borrow();
        let ChannelSenderMetricSets::Pdata(metrics) = &metrics.metrics else {
            panic!("expected pdata sender metrics");
        };
        assert_eq!(
            metrics
                .messages
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .messages
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Metrics,
                    outcome: Outcome::Refused,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .messages
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Traces,
                    outcome: Outcome::Failure,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(SignalChannelSendFailureAttributes {
                    signal: SignalType::Metrics,
                    error_type: ChannelSendErrorType::Full,
                })
                .failures
                .get(),
            1
        );
        assert_eq!(
            metrics
                .failures
                .get(SignalChannelSendFailureAttributes {
                    signal: SignalType::Traces,
                    error_type: ChannelSendErrorType::Closed,
                })
                .failures
                .get(),
            1
        );
    }

    /// Scenario: A PData receiver observes an empty poll, a queued message, and channel closure.
    /// Guarantees: Only dequeued messages are counted and queue state reports depth and capacity.
    #[test]
    fn channel_receiver_metrics_record_messages_and_queue_state() {
        let pipeline_ctx = test_context();
        let mut registry = ChannelMetricsRegistry::default();
        let (sender, receiver) = mpsc::Channel::new(1);
        let sender_entity_key = pipeline_ctx.register_node_channel_entity(
            "receiver:test".into(),
            "out".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        let receiver_entity_key = pipeline_ctx.register_node_channel_entity(
            "receiver:test".into(),
            "input".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        let queue_depth = LocalChannelQueueDepth::default();
        let sender = LocalSender::mpsc_with_metrics(
            sender,
            &mut registry,
            pdata_sender_metrics(&pipeline_ctx, sender_entity_key),
            queue_depth.clone(),
            Some(test_signal),
        );
        let mut receiver = LocalReceiver::mpsc_with_metrics(
            receiver,
            &mut registry,
            pdata_receiver_metrics(&pipeline_ctx, receiver_entity_key),
            1,
            queue_depth,
            Some(test_signal),
        );

        assert!(matches!(receiver.try_recv(), Err(RecvError::Empty)));
        sender
            .try_send(TestMessage {
                signal: SignalType::Logs,
            })
            .unwrap();

        let handles = registry.into_handles();
        let receiver_handle = take_local_receiver_handle(&handles);
        {
            let mut metrics = receiver_handle.borrow_mut();
            metrics.update_state_metrics();
            let ChannelReceiverMetricSets::Pdata(metrics) = &metrics.metrics else {
                panic!("expected pdata receiver metrics");
            };
            assert_eq!(metrics.state.queue_depth.get(), 1);
            assert_eq!(metrics.state.capacity.get(), 1);
        }

        let received = receiver.try_recv().unwrap();
        assert_eq!(received.signal, SignalType::Logs);
        drop(sender);
        assert!(matches!(receiver.try_recv(), Err(RecvError::Closed)));

        let metrics = receiver_handle.borrow();
        let ChannelReceiverMetricSets::Pdata(metrics) = &metrics.metrics else {
            panic!("expected pdata receiver metrics");
        };
        assert_eq!(
            metrics
                .messages
                .get(SignalAttributes {
                    signal: SignalType::Logs,
                })
                .messages
                .get(),
            1
        );
    }
}
