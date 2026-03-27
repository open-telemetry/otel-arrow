// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public types for the experimental control-optimized channel.

use std::time::Instant;

use thiserror::Error;

/// Phase of the control channel lifecycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// Ordinary control delivery is active.
    #[default]
    Normal,
    /// `DrainIngress` has been latched and best-effort control is suppressed.
    IngressDrainLatched,
    /// `Shutdown` has been latched and the channel is draining retained work.
    ShutdownLatched,
}

/// Logical control class used by queue policy and backpressure reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlClass {
    /// One of the lifecycle tokens.
    Lifecycle,
    /// High-frequency completion traffic.
    Completion,
    /// Deferred retry work.
    DelayedData,
    /// Timer-driven work.
    TimerTick,
    /// Telemetry collection work.
    CollectTelemetry,
    /// Configuration updates.
    Config,
}

/// Configuration for a control-optimized channel instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlChannelConfig {
    /// Maximum number of completion messages retained in the queue.
    pub completion_msg_capacity: usize,
    /// Maximum number of completion messages returned in a single batch.
    pub completion_batch_max: usize,
    /// Maximum number of delayed-data messages retained in the queue.
    pub delayed_data_capacity: usize,
    /// Maximum number of distinct pending timer sources.
    pub timer_sources_capacity: usize,
    /// Maximum number of distinct pending telemetry sources.
    pub telemetry_sources_capacity: usize,
}

impl Default for ControlChannelConfig {
    fn default() -> Self {
        Self {
            completion_msg_capacity: 256,
            completion_batch_max: 32,
            delayed_data_capacity: 64,
            timer_sources_capacity: 8,
            telemetry_sources_capacity: 4,
        }
    }
}

impl ControlChannelConfig {
    /// Validates channel configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.completion_batch_max == 0 {
            return Err(ConfigError::ZeroCompletionBatchMax);
        }
        Ok(())
    }
}

/// Configuration validation errors.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// `completion_batch_max` must be strictly positive.
    #[error("completion_batch_max must be greater than zero")]
    ZeroCompletionBatchMax,
}

/// Deduplication key for timer-tick sources within one channel instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimerSourceId(pub u64);

/// Deduplication key for telemetry-collection sources within one channel instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TelemetrySourceId(pub u64);

/// Shutdown-drain lifecycle message for receivers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrainIngressMsg {
    /// Deadline after which shutdown is considered forced.
    pub deadline: Instant,
    /// Human-readable reason for the drain request.
    pub reason: String,
}

/// Terminal lifecycle message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShutdownMsg {
    /// Deadline after which shutdown is considered forced.
    pub deadline: Instant,
    /// Human-readable reason for the shutdown request.
    pub reason: String,
}

/// Completion success message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AckMsg<PData> {
    /// Accepted payload being returned upstream.
    pub accepted: Box<PData>,
}

impl<PData> AckMsg<PData> {
    /// Creates a new acknowledgment wrapper.
    pub fn new(accepted: PData) -> Self {
        Self {
            accepted: Box::new(accepted),
        }
    }
}

/// Completion failure message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NackMsg<PData> {
    /// Human-readable failure reason.
    pub reason: String,
    /// Refused payload being returned upstream.
    pub refused: Box<PData>,
    /// Whether the failure is permanent.
    pub permanent: bool,
}

impl<PData> NackMsg<PData> {
    /// Creates a new non-permanent negative acknowledgment.
    pub fn new<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self {
            reason: reason.into(),
            refused: Box::new(refused),
            permanent: false,
        }
    }

    /// Creates a new permanent negative acknowledgment.
    pub fn new_permanent<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self {
            reason: reason.into(),
            refused: Box::new(refused),
            permanent: true,
        }
    }
}

/// Deferred work item retained by the control channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelayedDataMsg<PData> {
    /// Target wakeup time for the deferred payload.
    pub when: Instant,
    /// Deferred payload.
    pub data: Box<PData>,
}

impl<PData> DelayedDataMsg<PData> {
    /// Creates a new delayed-data wrapper.
    pub fn new(when: Instant, data: PData) -> Self {
        Self {
            when,
            data: Box::new(data),
        }
    }
}

/// Completion message retained inside a batched completion queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionMsg<PData> {
    /// Positive completion.
    Ack(AckMsg<PData>),
    /// Negative completion.
    Nack(NackMsg<PData>),
}

/// Command submitted to the control-aware channel.
#[derive(Clone, Debug, PartialEq)]
pub enum ControlCmd<PData> {
    /// Lifecycle drain token.
    DrainIngress(DrainIngressMsg),
    /// Lifecycle shutdown token.
    Shutdown(ShutdownMsg),
    /// Completion success.
    Ack(AckMsg<PData>),
    /// Completion failure.
    Nack(NackMsg<PData>),
    /// Latest configuration update.
    Config {
        /// Resolved configuration payload.
        config: serde_json::Value,
    },
    /// Timer-driven control work.
    TimerTick {
        /// Deduplication source for the timer.
        source: TimerSourceId,
    },
    /// Telemetry-driven control work.
    CollectTelemetry {
        /// Deduplication source for telemetry collection.
        source: TelemetrySourceId,
    },
    /// Delayed retry work.
    DelayedData(DelayedDataMsg<PData>),
}

/// Event surfaced by the control-aware receiver.
#[derive(Clone, Debug, PartialEq)]
pub enum ControlEvent<PData> {
    /// Lifecycle drain token.
    DrainIngress(DrainIngressMsg),
    /// Batch of completions in arrival order.
    CompletionBatch(Vec<CompletionMsg<PData>>),
    /// One delayed-data item.
    DelayedData(DelayedDataMsg<PData>),
    /// Latest configuration update.
    Config {
        /// Resolved configuration payload.
        config: serde_json::Value,
    },
    /// Timer-driven control work.
    TimerTick {
        /// Source that requested the tick.
        source: TimerSourceId,
    },
    /// Telemetry-driven control work.
    CollectTelemetry {
        /// Source that requested collection.
        source: TelemetrySourceId,
    },
    /// Terminal lifecycle token.
    Shutdown(ShutdownMsg),
}

/// Result of a successful send attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendOutcome {
    /// The command mutated channel state and is now retained or pending delivery.
    Accepted,
    /// The command was coalesced with an already pending equivalent item.
    Coalesced,
    /// The command replaced an older pending item of the same class.
    Replaced,
    /// The lifecycle token was already latched earlier in the channel lifetime.
    DuplicateLifecycle,
    /// The command was intentionally dropped because the channel is draining.
    DroppedDuringDrain,
}

/// Send errors for the control-aware sender.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SendError {
    /// The channel has been closed.
    #[error("control channel is closed")]
    Closed,
    /// The bounded class-specific capacity has been reached.
    #[error("control channel capacity reached for {0:?}")]
    Full(ControlClass),
}

/// Snapshot of queue occupancy and lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlChannelStats {
    /// Current lifecycle phase.
    pub phase: Phase,
    /// Whether a drain-ingress token is still pending delivery.
    pub has_pending_drain_ingress: bool,
    /// Whether a shutdown token is still pending delivery.
    pub has_pending_shutdown: bool,
    /// Number of retained completion messages.
    pub completion_len: usize,
    /// Number of retained delayed-data items.
    pub delayed_len: usize,
    /// Whether a configuration update is pending.
    pub has_pending_config: bool,
    /// Number of pending timer sources.
    pub timer_sources_len: usize,
    /// Number of pending telemetry sources.
    pub telemetry_sources_len: usize,
    /// Whether the channel is closed for new sends.
    pub closed: bool,
}
