// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared queue core for the experimental control-aware channel.

use crate::{
    CompletionMsg, ControlChannelConfig, ControlChannelStats, ControlClass, ControlCmd,
    ControlEvent, DelayedDataMsg, DrainIngressMsg, Phase, SendError, SendOutcome, ShutdownMsg,
    TelemetrySourceId, TimerSourceId,
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RetainedCursor {
    #[default]
    Completion,
    DelayedData,
}

pub(crate) struct Inner<PData> {
    pub(crate) config: ControlChannelConfig,
    pub(crate) phase: Phase,
    pub(crate) closed: bool,
    pub(crate) version: u64,
    drain_ingress: Option<DrainIngressMsg>,
    shutdown: Option<ShutdownMsg>,
    completion: VecDeque<CompletionMsg<PData>>,
    delayed: VecDeque<DelayedDataMsg<PData>>,
    latest_config: Option<serde_json::Value>,
    pending_timer_ticks: VecDeque<TimerSourceId>,
    pending_telemetry_ticks: VecDeque<TelemetrySourceId>,
    next_retained: RetainedCursor,
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
            completion: VecDeque::new(),
            delayed: VecDeque::new(),
            latest_config: None,
            pending_timer_ticks: VecDeque::new(),
            pending_telemetry_ticks: VecDeque::new(),
            next_retained: RetainedCursor::default(),
        }
    }

    pub(crate) fn stats(&self) -> ControlChannelStats {
        ControlChannelStats {
            phase: self.phase,
            has_pending_drain_ingress: self.drain_ingress.is_some(),
            has_pending_shutdown: self.shutdown.is_some(),
            completion_len: self.completion.len(),
            delayed_len: self.delayed.len(),
            has_pending_config: self.latest_config.is_some(),
            timer_sources_len: self.pending_timer_ticks.len(),
            telemetry_sources_len: self.pending_telemetry_ticks.len(),
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

    pub(crate) fn send(&mut self, cmd: ControlCmd<PData>) -> Result<SendOutcome, SendError> {
        if self.closed {
            return Err(SendError::Closed);
        }

        match cmd {
            ControlCmd::DrainIngress(msg) => self.send_drain_ingress(msg),
            ControlCmd::Shutdown(msg) => self.send_shutdown(msg),
            ControlCmd::Ack(ack) => self.send_completion(CompletionMsg::Ack(ack)),
            ControlCmd::Nack(nack) => self.send_completion(CompletionMsg::Nack(nack)),
            ControlCmd::Config { config } => self.send_config(config),
            ControlCmd::TimerTick { source } => self.send_timer_tick(source),
            ControlCmd::CollectTelemetry { source } => self.send_telemetry_tick(source),
            ControlCmd::DelayedData(msg) => self.send_delayed_data(msg),
        }
    }

    pub(crate) fn pop_event(&mut self) -> Option<ControlEvent<PData>> {
        if let Some(msg) = self.drain_ingress.take() {
            self.bump_version();
            return Some(ControlEvent::DrainIngress(msg));
        }

        if let Some(event) = self.pop_retained_event() {
            return Some(event);
        }

        if let Some(msg) = self.shutdown.take() {
            self.bump_version();
            return Some(ControlEvent::Shutdown(msg));
        }

        if self.phase != Phase::Normal {
            return None;
        }

        if let Some(config) = self.latest_config.take() {
            self.bump_version();
            return Some(ControlEvent::Config { config });
        }

        if let Some(source) = self.pending_timer_ticks.pop_front() {
            self.bump_version();
            return Some(ControlEvent::TimerTick { source });
        }

        if let Some(source) = self.pending_telemetry_ticks.pop_front() {
            self.bump_version();
            return Some(ControlEvent::CollectTelemetry { source });
        }

        None
    }

    fn send_drain_ingress(&mut self, msg: DrainIngressMsg) -> Result<SendOutcome, SendError> {
        if self.drain_ingress.is_some() {
            return Ok(SendOutcome::DuplicateLifecycle);
        }

        self.drain_ingress = Some(msg);
        if self.phase == Phase::Normal {
            self.phase = Phase::IngressDrainLatched;
        }
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn send_shutdown(&mut self, msg: ShutdownMsg) -> Result<SendOutcome, SendError> {
        if self.shutdown.is_some() {
            return Ok(SendOutcome::DuplicateLifecycle);
        }

        self.shutdown = Some(msg);
        self.phase = Phase::ShutdownLatched;
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn send_completion(&mut self, msg: CompletionMsg<PData>) -> Result<SendOutcome, SendError> {
        if self.completion.len() >= self.config.completion_msg_capacity {
            return Err(SendError::Full(ControlClass::Completion));
        }

        self.completion.push_back(msg);
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn send_config(&mut self, config: serde_json::Value) -> Result<SendOutcome, SendError> {
        if self.phase != Phase::Normal {
            return Ok(SendOutcome::DroppedDuringDrain);
        }

        let outcome = if self.latest_config.is_some() {
            SendOutcome::Replaced
        } else {
            SendOutcome::Accepted
        };
        self.latest_config = Some(config);
        self.bump_version();
        Ok(outcome)
    }

    fn send_timer_tick(&mut self, source: TimerSourceId) -> Result<SendOutcome, SendError> {
        if self.phase != Phase::Normal {
            return Ok(SendOutcome::DroppedDuringDrain);
        }

        if self.pending_timer_ticks.contains(&source) {
            return Ok(SendOutcome::Coalesced);
        }
        if self.pending_timer_ticks.len() >= self.config.timer_sources_capacity {
            return Err(SendError::Full(ControlClass::TimerTick));
        }

        self.pending_timer_ticks.push_back(source);
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn send_telemetry_tick(&mut self, source: TelemetrySourceId) -> Result<SendOutcome, SendError> {
        if self.phase != Phase::Normal {
            return Ok(SendOutcome::DroppedDuringDrain);
        }

        if self.pending_telemetry_ticks.contains(&source) {
            return Ok(SendOutcome::Coalesced);
        }
        if self.pending_telemetry_ticks.len() >= self.config.telemetry_sources_capacity {
            return Err(SendError::Full(ControlClass::CollectTelemetry));
        }

        self.pending_telemetry_ticks.push_back(source);
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn send_delayed_data(&mut self, msg: DelayedDataMsg<PData>) -> Result<SendOutcome, SendError> {
        if self.delayed.len() >= self.config.delayed_data_capacity {
            return Err(SendError::Full(ControlClass::DelayedData));
        }

        self.delayed.push_back(msg);
        self.bump_version();
        Ok(SendOutcome::Accepted)
    }

    fn pop_retained_event(&mut self) -> Option<ControlEvent<PData>> {
        let completion_ready = !self.completion.is_empty();
        let delayed_ready = !self.delayed.is_empty();

        match (completion_ready, delayed_ready, self.next_retained) {
            (false, false, _) => None,
            (true, false, _) => Some(self.take_completion_batch()),
            (false, true, _) => Some(self.take_delayed_item()),
            (true, true, RetainedCursor::Completion) => Some(self.take_completion_batch()),
            (true, true, RetainedCursor::DelayedData) => Some(self.take_delayed_item()),
        }
    }

    fn take_completion_batch(&mut self) -> ControlEvent<PData> {
        let batch_len = self.completion.len().min(self.config.completion_batch_max);
        let mut batch = Vec::with_capacity(batch_len);
        for _ in 0..batch_len {
            let msg = self
                .completion
                .pop_front()
                .expect("completion batch length checked");
            batch.push(msg);
        }
        self.next_retained = RetainedCursor::DelayedData;
        self.bump_version();
        ControlEvent::CompletionBatch(batch)
    }

    fn take_delayed_item(&mut self) -> ControlEvent<PData> {
        let msg = self.delayed.pop_front().expect("delayed item exists");
        self.next_retained = RetainedCursor::Completion;
        self.bump_version();
        ControlEvent::DelayedData(msg)
    }

    fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}
