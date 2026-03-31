// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Queue core for the standalone control-aware channel.

use crate::types::CoreControlEvent;
use crate::{
    AdmissionClass, CompletionMsg, ControlChannelConfig, ControlChannelStats, ControlCmd,
    DrainIngressMsg, LifecycleSendResult, Phase, SendOutcome, ShutdownMsg, TrySendError,
};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy)]
enum NormalEventClass {
    /// Latest-wins config updates. Only the newest pending config matters.
    Config,
    /// Coalesced timer wakeup emitted at most once while pending.
    TimerTick,
    /// Coalesced telemetry collection token emitted at most once while pending.
    CollectTelemetry,
}

impl NormalEventClass {
    /// Advances the round-robin cursor used to provide bounded fairness across
    /// normal non-completion control work.
    fn next(self) -> Self {
        match self {
            Self::Config => Self::TimerTick,
            Self::TimerTick => Self::CollectTelemetry,
            Self::CollectTelemetry => Self::Config,
        }
    }
}

pub(crate) struct Inner<PData> {
    /// Immutable admission and fairness policy for this channel instance.
    pub(crate) config: ControlChannelConfig,
    /// Current lifecycle phase. This drives which classes can still be
    /// admitted and which events may be delivered next.
    pub(crate) phase: Phase,
    /// Terminal close flag. Once set, no new sends are accepted and the
    /// receiver returns `None` after buffered work is drained.
    pub(crate) closed: bool,
    // Generation counter used with `Notify` to avoid check-then-sleep races in
    // sender/receiver wait loops. Any state transition that can unblock a
    // waiter must bump this value before notifications are observed.
    pub(crate) version: u64,
    /// Reserved lifecycle tokens. They bypass bounded completion capacity and
    /// are delivered ahead of normal control work.
    drain_ingress: Option<DrainIngressMsg>,
    shutdown: Option<ShutdownMsg>,
    /// Deadline after which shutdown stops waiting for completion backlog and
    /// forces terminal progress.
    shutdown_deadline: Option<Instant>,
    /// Sticky lifecycle observability flags kept even after the corresponding
    /// token has been delivered.
    drain_ingress_recorded: bool,
    shutdown_recorded: bool,
    shutdown_forced: bool,
    /// Lossless backpressured completion backlog (`Ack`/`Nack`).
    completion: VecDeque<CompletionMsg<PData>>,
    /// Latest-wins normal control state.
    latest_config: Option<serde_json::Value>,
    /// Coalesced best-effort normal control state.
    pending_timer_tick: bool,
    pending_collect_telemetry: bool,
    /// Number of completion messages emitted since the last normal control
    /// event. This enforces bounded fairness between completion traffic and
    /// normal control work.
    completion_burst_len: usize,
    /// Monotonic counters used for observability and tests.
    completion_batch_emitted_total: u64,
    completion_message_emitted_total: u64,
    config_replaced_total: u64,
    timer_tick_coalesced_total: u64,
    collect_telemetry_coalesced_total: u64,
    normal_event_dropped_during_drain_total: u64,
    completion_abandoned_on_forced_shutdown_total: u64,
    /// Round-robin cursor for normal control events. This avoids fixed-priority
    /// starvation among config, timer, and telemetry work.
    next_normal_event: NormalEventClass,
}

impl<PData> Inner<PData> {
    /// Creates an empty queue core. The outer channel wrapper adds waiting
    /// behavior around this state machine.
    pub(crate) fn new(config: ControlChannelConfig) -> Self {
        Self {
            config,
            phase: Phase::Normal,
            closed: false,
            version: 0,
            drain_ingress: None,
            shutdown: None,
            shutdown_deadline: None,
            drain_ingress_recorded: false,
            shutdown_recorded: false,
            shutdown_forced: false,
            completion: VecDeque::new(),
            latest_config: None,
            pending_timer_tick: false,
            pending_collect_telemetry: false,
            completion_burst_len: 0,
            completion_batch_emitted_total: 0,
            completion_message_emitted_total: 0,
            config_replaced_total: 0,
            timer_tick_coalesced_total: 0,
            collect_telemetry_coalesced_total: 0,
            normal_event_dropped_during_drain_total: 0,
            completion_abandoned_on_forced_shutdown_total: 0,
            next_normal_event: NormalEventClass::Config,
        }
    }

    /// Returns a point-in-time snapshot of queue occupancy, lifecycle flags,
    /// and cumulative counters.
    pub(crate) fn stats(&self) -> ControlChannelStats {
        ControlChannelStats {
            phase: self.phase,
            drain_ingress_recorded: self.drain_ingress_recorded,
            shutdown_recorded: self.shutdown_recorded,
            has_pending_drain_ingress: self.drain_ingress.is_some(),
            has_pending_shutdown: self.shutdown.is_some(),
            completion_len: self.completion.len(),
            has_pending_config: self.latest_config.is_some(),
            has_pending_timer_tick: self.pending_timer_tick,
            has_pending_collect_telemetry: self.pending_collect_telemetry,
            completion_burst_len: self.completion_burst_len,
            completion_batch_emitted_total: self.completion_batch_emitted_total,
            completion_message_emitted_total: self.completion_message_emitted_total,
            config_replaced_total: self.config_replaced_total,
            timer_tick_coalesced_total: self.timer_tick_coalesced_total,
            collect_telemetry_coalesced_total: self.collect_telemetry_coalesced_total,
            normal_event_dropped_during_drain_total: self.normal_event_dropped_during_drain_total,
            completion_abandoned_on_forced_shutdown_total: self
                .completion_abandoned_on_forced_shutdown_total,
            shutdown_forced: self.shutdown_forced,
            closed: self.closed,
        }
    }

    /// Closes the queue core. This is triggered either explicitly by a
    /// sender or implicitly when the last sender handle drops.
    pub(crate) fn close(&mut self) -> bool {
        if self.closed {
            return false;
        }
        self.closed = true;
        self.bump_version();
        true
    }

    /// Returns the next deadline that senders/receivers must wait on in
    /// addition to notifications. Only shutdown currently arms an internal
    /// deadline.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        if self.closed || self.shutdown_forced {
            return None;
        }
        if self.phase == Phase::ShutdownRecorded {
            return self.shutdown_deadline;
        }
        None
    }

    /// Attempts to admit a non-lifecycle control command using the current
    /// phase and bounded-capacity policy.
    pub(crate) fn try_send(
        &mut self,
        cmd: ControlCmd<PData>,
    ) -> Result<SendOutcome, TrySendError<PData>> {
        if self.phase == Phase::ShutdownRecorded {
            self.refresh_shutdown_force(Instant::now());
        }
        if self.closed || self.shutdown_forced {
            return Err(TrySendError::Closed(cmd));
        }

        match cmd {
            ControlCmd::Ack(ack) => {
                if self.completion.len() >= self.config.completion_msg_capacity {
                    return Err(TrySendError::Full {
                        admission_class: AdmissionClass::Backpressured,
                        cmd: ControlCmd::Ack(ack),
                    });
                }
                Ok(self.push_completion(CompletionMsg::Ack(ack)))
            }
            ControlCmd::Nack(nack) => {
                if self.completion.len() >= self.config.completion_msg_capacity {
                    return Err(TrySendError::Full {
                        admission_class: AdmissionClass::Backpressured,
                        cmd: ControlCmd::Nack(nack),
                    });
                }
                Ok(self.push_completion(CompletionMsg::Nack(nack)))
            }
            ControlCmd::Config { config } => Ok(self.send_config(config)),
            ControlCmd::TimerTick => Ok(self.send_timer_tick()),
            ControlCmd::CollectTelemetry => Ok(self.send_telemetry_tick()),
        }
    }

    /// Records the receiver-only drain lifecycle token. The token is reserved
    /// capacity, delivered ahead of bounded control traffic, and clears pending
    /// normal control work that no longer matters once drain has started.
    pub(crate) fn record_drain_ingress(&mut self, msg: DrainIngressMsg) -> LifecycleSendResult {
        if self.closed {
            return LifecycleSendResult::Closed;
        }
        if self.drain_ingress_recorded {
            return LifecycleSendResult::AlreadyAccepted;
        }

        self.clear_normal_pending();
        self.drain_ingress = Some(msg);
        self.drain_ingress_recorded = true;
        if self.phase == Phase::Normal {
            self.phase = Phase::IngressDrainRecorded;
        }
        self.bump_version();
        LifecycleSendResult::Accepted
    }

    /// Records shutdown with its terminal deadline. Shutdown admission is
    /// reserved-capacity and flips the queue into terminal-progress mode.
    pub(crate) fn record_shutdown(&mut self, msg: ShutdownMsg) -> LifecycleSendResult {
        if self.closed {
            return LifecycleSendResult::Closed;
        }
        if self.shutdown_recorded {
            return LifecycleSendResult::AlreadyAccepted;
        }

        self.clear_normal_pending();
        self.shutdown_deadline = Some(msg.deadline);
        self.shutdown = Some(msg);
        self.shutdown_recorded = true;
        self.phase = Phase::ShutdownRecorded;
        self.refresh_shutdown_force(Instant::now());
        self.bump_version();
        LifecycleSendResult::Accepted
    }

    /// Pops the next deliverable event according to lifecycle precedence,
    /// bounded fairness, and deadline-bounded shutdown rules.
    ///
    /// Delivery order is:
    /// 1. `DrainIngress` if pending
    /// 2. shutdown-mode completion draining, subject to deadline forcing
    /// 3. in normal phase, bounded alternation between completion batches and
    ///    round-robin normal control events
    pub(crate) fn pop_event(&mut self) -> Option<CoreControlEvent<PData>> {
        if self.phase == Phase::ShutdownRecorded {
            self.refresh_shutdown_force(Instant::now());
        }

        if let Some(msg) = self.drain_ingress.take() {
            self.completion_burst_len = 0;
            self.bump_version();
            return Some(CoreControlEvent::DrainIngress(msg));
        }

        match self.phase {
            Phase::ShutdownRecorded => {
                if self.shutdown_forced {
                    return self.finalize_shutdown(true);
                }
                if !self.completion.is_empty() {
                    return Some(self.take_completion_batch(None));
                }
                return self.finalize_shutdown(false);
            }
            Phase::IngressDrainRecorded => {
                if !self.completion.is_empty() {
                    return Some(self.take_completion_batch(None));
                }
                return None;
            }
            Phase::Normal => {}
        }

        let has_pending_normal_event = self.has_pending_normal_event();

        if !has_pending_normal_event {
            if !self.completion.is_empty() {
                return Some(self.take_completion_batch(None));
            }
            return None;
        }

        if self.completion_burst_len >= self.config.completion_burst_limit {
            if let Some(event) = self.take_next_normal_event() {
                return Some(event);
            }
        }

        if !self.completion.is_empty() {
            return Some(
                self.take_completion_batch(Some(
                    self.config
                        .completion_burst_limit
                        .saturating_sub(self.completion_burst_len),
                )),
            );
        }

        if let Some(event) = self.take_next_normal_event() {
            return Some(event);
        }

        None
    }

    /// Checks whether shutdown has crossed its force deadline and, if so,
    /// flips the queue into forced terminal progress.
    fn refresh_shutdown_force(&mut self, now: Instant) {
        if self.shutdown_forced || self.phase != Phase::ShutdownRecorded {
            return;
        }

        if let Some(deadline) = self.shutdown_deadline {
            if now >= deadline {
                self.shutdown_forced = true;
                self.bump_version();
            }
        }
    }

    /// Appends one completion message to the lossless backpressured backlog.
    fn push_completion(&mut self, msg: CompletionMsg<PData>) -> SendOutcome {
        self.completion.push_back(msg);
        self.bump_version();
        SendOutcome::Accepted
    }

    /// Accepts or replaces the latest pending config while the queue remains in
    /// normal phase. Config is dropped once drain or shutdown has started.
    fn send_config(&mut self, config: serde_json::Value) -> SendOutcome {
        if self.phase != Phase::Normal {
            self.normal_event_dropped_during_drain_total = self
                .normal_event_dropped_during_drain_total
                .saturating_add(1);
            return SendOutcome::DroppedDuringDrain;
        }

        let outcome = if self.latest_config.is_some() {
            self.config_replaced_total = self.config_replaced_total.saturating_add(1);
            SendOutcome::Replaced
        } else {
            SendOutcome::Accepted
        };
        self.latest_config = Some(config);
        self.bump_version();
        outcome
    }

    /// Accepts one pending timer tick token. Repeated offers coalesce until
    /// the pending token is delivered.
    fn send_timer_tick(&mut self) -> SendOutcome {
        if self.phase != Phase::Normal {
            self.normal_event_dropped_during_drain_total = self
                .normal_event_dropped_during_drain_total
                .saturating_add(1);
            return SendOutcome::DroppedDuringDrain;
        }

        if self.pending_timer_tick {
            self.timer_tick_coalesced_total = self.timer_tick_coalesced_total.saturating_add(1);
            return SendOutcome::Coalesced;
        }
        self.pending_timer_tick = true;
        self.bump_version();
        SendOutcome::Accepted
    }

    /// Accepts one pending telemetry collection token. Repeated offers
    /// coalesce until the pending token is delivered.
    fn send_telemetry_tick(&mut self) -> SendOutcome {
        if self.phase != Phase::Normal {
            self.normal_event_dropped_during_drain_total = self
                .normal_event_dropped_during_drain_total
                .saturating_add(1);
            return SendOutcome::DroppedDuringDrain;
        }

        if self.pending_collect_telemetry {
            self.collect_telemetry_coalesced_total =
                self.collect_telemetry_coalesced_total.saturating_add(1);
            return SendOutcome::Coalesced;
        }

        self.pending_collect_telemetry = true;
        self.bump_version();
        SendOutcome::Accepted
    }

    /// Drops pending normal control state when drain/shutdown begins or after
    /// terminal shutdown delivery.
    fn clear_normal_pending(&mut self) {
        self.latest_config = None;
        self.pending_timer_tick = false;
        self.pending_collect_telemetry = false;
        self.completion_burst_len = 0;
    }

    fn has_pending_normal_event(&self) -> bool {
        self.latest_config.is_some() || self.pending_timer_tick || self.pending_collect_telemetry
    }

    /// Picks the next pending normal control event using the round-robin
    /// cursor. Delivering any normal event resets the completion burst counter.
    fn take_next_normal_event(&mut self) -> Option<CoreControlEvent<PData>> {
        for _ in 0..3 {
            let candidate = self.next_normal_event;
            self.next_normal_event = candidate.next();

            let event = match candidate {
                NormalEventClass::Config => self
                    .latest_config
                    .take()
                    .map(|config| CoreControlEvent::Config { config }),
                NormalEventClass::TimerTick => self.pending_timer_tick.then(|| {
                    self.pending_timer_tick = false;
                    CoreControlEvent::TimerTick
                }),
                NormalEventClass::CollectTelemetry => self.pending_collect_telemetry.then(|| {
                    self.pending_collect_telemetry = false;
                    CoreControlEvent::CollectTelemetry
                }),
            };

            if let Some(event) = event {
                self.completion_burst_len = 0;
                self.bump_version();
                return Some(event);
            }
        }

        None
    }

    /// Emits one bounded completion batch. When fairness is active, the batch
    /// size is further capped so at least one normal event can run before more
    /// completion traffic is emitted.
    fn take_completion_batch(&mut self, fairness_budget: Option<usize>) -> CoreControlEvent<PData> {
        let mut batch_len = self.completion.len().min(self.config.completion_batch_max);
        if let Some(limit) = fairness_budget {
            batch_len = batch_len.min(limit.max(1));
        }

        let mut batch = Vec::with_capacity(batch_len);
        for _ in 0..batch_len {
            let msg = self
                .completion
                .pop_front()
                .expect("completion batch length checked");
            batch.push(msg);
        }
        self.completion_burst_len = self.completion_burst_len.saturating_add(batch_len);
        self.completion_batch_emitted_total = self.completion_batch_emitted_total.saturating_add(1);
        self.completion_message_emitted_total = self
            .completion_message_emitted_total
            .saturating_add(batch_len as u64);
        self.bump_version();
        CoreControlEvent::CompletionBatch(batch)
    }

    /// Emits the terminal shutdown event. Forced shutdown abandons any
    /// remaining completion backlog; graceful shutdown emits only after that
    /// backlog has drained.
    fn finalize_shutdown(&mut self, forced: bool) -> Option<CoreControlEvent<PData>> {
        let msg = self.shutdown.take()?;
        if forced {
            self.completion_abandoned_on_forced_shutdown_total = self
                .completion_abandoned_on_forced_shutdown_total
                .saturating_add(self.completion.len() as u64);
            self.completion.clear();
        }

        self.clear_normal_pending();
        self.shutdown_deadline = None;
        self.shutdown_forced = false;
        self.closed = true;
        self.bump_version();
        Some(CoreControlEvent::Shutdown(msg))
    }

    /// Bumps the waiter generation after any transition that may unblock a
    /// sender or receiver.
    fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}
