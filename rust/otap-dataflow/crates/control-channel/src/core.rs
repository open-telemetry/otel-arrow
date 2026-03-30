// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared queue core for the experimental control-aware channel.

use crate::types::CoreControlEvent;
use crate::{
    AdmissionClass, CompletionMsg, ControlChannelConfig, ControlChannelStats, ControlCmd,
    DrainIngressMsg, LifecycleSendResult, Phase, SendOutcome, ShutdownMsg, TrySendError,
};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy)]
enum NormalEventClass {
    Config,
    TimerTick,
    CollectTelemetry,
}

impl NormalEventClass {
    fn next(self) -> Self {
        match self {
            Self::Config => Self::TimerTick,
            Self::TimerTick => Self::CollectTelemetry,
            Self::CollectTelemetry => Self::Config,
        }
    }
}

pub(crate) struct Inner<PData> {
    pub(crate) config: ControlChannelConfig,
    pub(crate) phase: Phase,
    pub(crate) closed: bool,
    // Generation counter used with `Notify` to avoid check-then-sleep races in
    // sender/receiver wait loops. Any state transition that can unblock a
    // waiter must bump this value before notifications are observed.
    pub(crate) version: u64,
    drain_ingress: Option<DrainIngressMsg>,
    shutdown: Option<ShutdownMsg>,
    shutdown_deadline: Option<Instant>,
    drain_ingress_recorded: bool,
    shutdown_recorded: bool,
    shutdown_forced: bool,
    completion: VecDeque<CompletionMsg<PData>>,
    latest_config: Option<serde_json::Value>,
    pending_timer_tick: bool,
    pending_collect_telemetry: bool,
    completion_burst_len: usize,
    completion_batch_emitted_total: u64,
    completion_message_emitted_total: u64,
    config_replaced_total: u64,
    timer_tick_coalesced_total: u64,
    collect_telemetry_coalesced_total: u64,
    normal_event_dropped_during_drain_total: u64,
    completion_abandoned_on_forced_shutdown_total: u64,
    next_normal_event: NormalEventClass,
}

impl<PData> Inner<PData> {
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

    pub(crate) fn close(&mut self) -> bool {
        if self.closed {
            return false;
        }
        self.closed = true;
        self.bump_version();
        true
    }

    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        if self.closed || self.shutdown_forced {
            return None;
        }
        if self.phase == Phase::ShutdownRecorded {
            return self.shutdown_deadline;
        }
        None
    }

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

    fn push_completion(&mut self, msg: CompletionMsg<PData>) -> SendOutcome {
        self.completion.push_back(msg);
        self.bump_version();
        SendOutcome::Accepted
    }

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

    fn clear_normal_pending(&mut self) {
        self.latest_config = None;
        self.pending_timer_tick = false;
        self.pending_collect_telemetry = false;
        self.completion_burst_len = 0;
    }

    fn has_pending_normal_event(&self) -> bool {
        self.latest_config.is_some() || self.pending_timer_tick || self.pending_collect_telemetry
    }

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

    fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}
