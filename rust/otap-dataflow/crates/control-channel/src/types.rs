// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public types for the standalone control-aware channel.

use std::time::Instant;

use thiserror::Error;

/// Phase of the control channel lifecycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// Ordinary control delivery is active.
    #[default]
    Normal,
    /// `DrainIngress` has been recorded and best-effort control is suppressed.
    IngressDrainRecorded,
    /// `Shutdown` has been recorded and the channel is draining retained work.
    ShutdownRecorded,
}

/// Admission class used by queue policy and backpressure reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionClass {
    /// Backpressured retained traffic such as completion messages (`Ack`/`Nack`).
    Backpressured,
    /// Coalesced best-effort work such as timer and telemetry ticks.
    BestEffort,
}

/// Configuration for a control-optimized channel instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlChannelConfig {
    /// Maximum number of completion messages retained in the queue.
    pub completion_msg_capacity: usize,
    /// Maximum number of completion messages returned in a single batch.
    pub completion_batch_max: usize,
    /// Maximum number of completion messages delivered consecutively before the
    /// scheduler must surface one pending non-completion event.
    pub completion_burst_limit: usize,
}

impl Default for ControlChannelConfig {
    fn default() -> Self {
        Self {
            completion_msg_capacity: 256,
            completion_batch_max: 32,
            completion_burst_limit: 32,
        }
    }
}

impl ControlChannelConfig {
    /// Validates channel configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.completion_msg_capacity == 0 {
            return Err(ConfigError::ZeroCompletionMsgCapacity);
        }
        if self.completion_batch_max == 0 {
            return Err(ConfigError::ZeroCompletionBatchMax);
        }
        if self.completion_burst_limit == 0 {
            return Err(ConfigError::ZeroCompletionBurstLimit);
        }
        Ok(())
    }
}

/// Configuration validation errors.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// `completion_msg_capacity` must be strictly positive.
    #[error("completion_msg_capacity must be greater than zero")]
    ZeroCompletionMsgCapacity,
    /// `completion_batch_max` must be strictly positive.
    #[error("completion_batch_max must be greater than zero")]
    ZeroCompletionBatchMax,
    /// `completion_burst_limit` must be strictly positive.
    #[error("completion_burst_limit must be greater than zero")]
    ZeroCompletionBurstLimit,
}

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
pub struct AckMsg<PData, Meta = ()> {
    /// Accepted payload being returned upstream.
    pub accepted: Box<PData>,
    /// Explicit completion metadata carried with the returned payload.
    /// Future engine integration can use this for unwind state such as
    /// `UnwindData`.
    pub meta: Meta,
}

impl<PData> AckMsg<PData, ()> {
    /// Creates a new acknowledgment wrapper without additional metadata.
    pub fn new(accepted: PData) -> Self {
        Self::with_meta(accepted, ())
    }
}

impl<PData, Meta> AckMsg<PData, Meta> {
    /// Creates a new acknowledgment wrapper with explicit metadata.
    pub fn with_meta(accepted: PData, meta: Meta) -> Self {
        Self {
            accepted: Box::new(accepted),
            meta,
        }
    }
}

/// Completion failure message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NackMsg<PData, Meta = ()> {
    /// Human-readable failure reason.
    pub reason: String,
    /// Refused payload being returned upstream.
    pub refused: Box<PData>,
    /// Explicit completion metadata carried with the returned payload.
    /// Future engine integration can use this for unwind state such as
    /// `UnwindData`.
    pub meta: Meta,
    /// Whether the failure is permanent.
    pub permanent: bool,
}

impl<PData> NackMsg<PData, ()> {
    /// Creates a new non-permanent negative acknowledgment without additional metadata.
    pub fn new<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self::with_meta(reason, refused, ())
    }

    /// Creates a new permanent negative acknowledgment without additional metadata.
    pub fn new_permanent<T: Into<String>>(reason: T, refused: PData) -> Self {
        Self::with_meta_permanent(reason, refused, ())
    }
}

impl<PData, Meta> NackMsg<PData, Meta> {
    /// Creates a new non-permanent negative acknowledgment with explicit metadata.
    pub fn with_meta<T: Into<String>>(reason: T, refused: PData, meta: Meta) -> Self {
        Self::new_internal(reason, refused, meta, false)
    }

    /// Creates a new permanent negative acknowledgment with explicit metadata.
    pub fn with_meta_permanent<T: Into<String>>(reason: T, refused: PData, meta: Meta) -> Self {
        Self::new_internal(reason, refused, meta, true)
    }

    fn new_internal<T: Into<String>>(
        reason: T,
        refused: PData,
        meta: Meta,
        permanent: bool,
    ) -> Self {
        Self {
            reason: reason.into(),
            refused: Box::new(refused),
            meta,
            permanent,
        }
    }
}

/// Completion message retained inside a batched completion queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionMsg<PData, Meta = ()> {
    /// Positive completion.
    Ack(AckMsg<PData, Meta>),
    /// Negative completion.
    Nack(NackMsg<PData, Meta>),
}

/// Command submitted to the control-aware channel.
#[derive(Clone, Debug, PartialEq)]
pub enum ControlCmd<PData, Meta = ()> {
    /// Completion success.
    Ack(AckMsg<PData, Meta>),
    /// Completion failure.
    Nack(NackMsg<PData, Meta>),
    /// Latest configuration update.
    Config {
        /// Resolved configuration payload.
        config: serde_json::Value,
    },
    /// Timer-driven control work.
    TimerTick,
    /// Telemetry-driven control work.
    CollectTelemetry,
}

/// Event surfaced by receiver-role control channels.
#[derive(Clone, Debug, PartialEq)]
pub enum ReceiverControlEvent<PData, Meta = ()> {
    /// Lifecycle drain token.
    DrainIngress(DrainIngressMsg),
    /// Batch of completions in arrival order.
    CompletionBatch(Vec<CompletionMsg<PData, Meta>>),
    /// Latest configuration update.
    Config {
        /// Resolved configuration payload.
        config: serde_json::Value,
    },
    /// Timer-driven control work.
    TimerTick,
    /// Telemetry-driven control work.
    CollectTelemetry,
    /// Terminal lifecycle token.
    Shutdown(ShutdownMsg),
}

/// Event surfaced by non-receiver node control channels.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeControlEvent<PData, Meta = ()> {
    /// Batch of completions in arrival order.
    CompletionBatch(Vec<CompletionMsg<PData, Meta>>),
    /// Latest configuration update.
    Config {
        /// Resolved configuration payload.
        config: serde_json::Value,
    },
    /// Timer-driven control work.
    TimerTick,
    /// Telemetry-driven control work.
    CollectTelemetry,
    /// Terminal lifecycle token.
    Shutdown(ShutdownMsg),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CoreControlEvent<PData, Meta = ()> {
    DrainIngress(DrainIngressMsg),
    CompletionBatch(Vec<CompletionMsg<PData, Meta>>),
    Config { config: serde_json::Value },
    TimerTick,
    CollectTelemetry,
    Shutdown(ShutdownMsg),
}

impl<PData, Meta> ReceiverControlEvent<PData, Meta> {
    pub(crate) fn from_core(event: CoreControlEvent<PData, Meta>) -> Self {
        match event {
            CoreControlEvent::DrainIngress(msg) => Self::DrainIngress(msg),
            CoreControlEvent::CompletionBatch(batch) => Self::CompletionBatch(batch),
            CoreControlEvent::Config { config } => Self::Config { config },
            CoreControlEvent::TimerTick => Self::TimerTick,
            CoreControlEvent::CollectTelemetry => Self::CollectTelemetry,
            CoreControlEvent::Shutdown(msg) => Self::Shutdown(msg),
        }
    }
}

impl<PData, Meta> NodeControlEvent<PData, Meta> {
    pub(crate) fn from_core(event: CoreControlEvent<PData, Meta>) -> Self {
        match event {
            CoreControlEvent::DrainIngress(_) => {
                panic!("DrainIngress must not be delivered on node control channels")
            }
            CoreControlEvent::CompletionBatch(batch) => Self::CompletionBatch(batch),
            CoreControlEvent::Config { config } => Self::Config { config },
            CoreControlEvent::TimerTick => Self::TimerTick,
            CoreControlEvent::CollectTelemetry => Self::CollectTelemetry,
            CoreControlEvent::Shutdown(msg) => Self::Shutdown(msg),
        }
    }
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
    /// The command was intentionally dropped because the channel is draining.
    DroppedDuringDrain,
}

/// Result of submitting a lifecycle token to the sender API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleSendResult {
    /// The lifecycle token was accepted for the first time.
    Accepted,
    /// The lifecycle token had already been accepted earlier in the channel lifetime.
    AlreadyAccepted,
    /// The channel is closed.
    Closed,
}

/// Non-blocking send errors for the control-aware sender.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TrySendError<PData, Meta = ()> {
    /// The channel has been closed.
    #[error("control channel is closed")]
    Closed(ControlCmd<PData, Meta>),
    /// The bounded class-specific capacity has been reached.
    #[error("control channel capacity reached for {admission_class:?}")]
    Full {
        /// The admission class whose bounded capacity is saturated.
        admission_class: AdmissionClass,
        /// The command that could not be enqueued.
        cmd: ControlCmd<PData, Meta>,
    },
}

/// Blocking send errors for the control-aware sender.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum SendError<PData, Meta = ()> {
    /// The channel closed before the command could be enqueued.
    #[error("control channel is closed")]
    Closed(ControlCmd<PData, Meta>),
}

/// Snapshot of queue occupancy and lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlChannelStats {
    /// Current lifecycle phase.
    pub phase: Phase,
    /// Whether `DrainIngress` has been accepted at least once during the channel lifetime.
    pub drain_ingress_recorded: bool,
    /// Whether `Shutdown` has been accepted at least once during the channel lifetime.
    pub shutdown_recorded: bool,
    /// Whether a drain-ingress token is still pending delivery.
    pub has_pending_drain_ingress: bool,
    /// Whether a shutdown token is still pending delivery.
    pub has_pending_shutdown: bool,
    /// Number of retained completion messages.
    pub completion_len: usize,
    /// Whether a configuration update is pending.
    pub has_pending_config: bool,
    /// Whether a timer tick is pending.
    pub has_pending_timer_tick: bool,
    /// Whether a telemetry-collection request is pending.
    pub has_pending_collect_telemetry: bool,
    /// Number of completion messages delivered since the last non-completion
    /// event. This is the fairness budget currently consumed.
    pub completion_burst_len: usize,
    /// Total number of completion batches emitted by the receiver side.
    pub completion_batch_emitted_total: u64,
    /// Total number of completion messages emitted across all batches.
    pub completion_message_emitted_total: u64,
    /// Total number of pending config replacements.
    pub config_replaced_total: u64,
    /// Total number of timer ticks coalesced into an already pending tick.
    pub timer_tick_coalesced_total: u64,
    /// Total number of telemetry-collection requests coalesced into an already pending request.
    pub collect_telemetry_coalesced_total: u64,
    /// Total number of normal control events dropped during drain or shutdown.
    pub normal_event_dropped_during_drain_total: u64,
    /// Total number of retained completions abandoned when forced shutdown fires.
    pub completion_abandoned_on_forced_shutdown_total: u64,
    /// Whether the shutdown deadline has already forced terminal progress.
    pub shutdown_forced: bool,
    /// Whether the channel is closed for new sends.
    pub closed: bool,
}
