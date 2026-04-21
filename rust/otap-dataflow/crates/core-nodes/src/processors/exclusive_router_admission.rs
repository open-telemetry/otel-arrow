// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared selected-route admission state for exclusive-routing processors.
//!
//! `content_router` and `signal_type_router` both select exactly one output
//! for each inbound message. They differ in route selection and telemetry, but
//! the blocked-route scheduling mechanics are the same:
//!
//! - `reject_immediately` has no local parked state
//! - `backpressure` parks at most one message per blocked output port
//! - one engine wakeup slot is always armed to the earliest next probe across
//!   whatever local parked state currently exists
//!
//! The parked pdata stays local to the router instead of being requeued into
//! the normal pdata inbox. That keeps blocked-route work separate from fresh
//! ingress and makes the policy explicit in router-local state.

use otap_df_config::PortName;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::control::{WakeupRevision, WakeupSlot};
use otap_df_engine::local::processor::EffectHandler;
use otap_df_engine::{ProcessorRuntimeRequirements, WakeupError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Shared wakeup slot used by exclusive-router admission scheduling.
pub(crate) const EXCLUSIVE_ROUTER_WAKEUP_SLOT: WakeupSlot = WakeupSlot(0);

/// Sparse re-probe cadence for blocked selected routes.
pub(crate) const PROBE_INTERVAL: Duration = Duration::from_millis(200);

/// Policy for selected-route `Full` admission outcomes.
///
/// Both policies avoid head-of-line blocking by never awaiting the selected
/// send in the main router task. They differ in how much route-local state the
/// router keeps before pushing back on upstream work.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnFullPolicy {
    /// If route A is full while route B is healthy, NACK the message selected
    /// for route A immediately and keep admitting traffic for route B.
    #[default]
    RejectImmediately,
    /// If route A is full while route B is healthy, park one message for route
    /// A and keep admitting traffic for route B. The router only pauses pdata
    /// admission once every selectable route currently has a parked full
    /// message.
    Backpressure,
}

/// Router-local selected-route admission policy shared by exclusive routers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedRouteAdmissionPolicy {
    /// Handling for selected-route `Full`.
    #[serde(default)]
    pub on_full: OnFullPolicy,
}

impl SelectedRouteAdmissionPolicy {
    /// Validate the policy-local config contract.
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }

    /// Runtime requirements implied by the selected policy.
    #[must_use]
    pub const fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        match self.on_full {
            OnFullPolicy::RejectImmediately => ProcessorRuntimeRequirements::none(),
            OnFullPolicy::Backpressure => ProcessorRuntimeRequirements::with_local_wakeups(1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArmedWakeup {
    when: Instant,
    revision: WakeupRevision,
}

/// One locally parked selected-route admission.
pub(crate) struct PendingRoute<PData, Meta> {
    pub(crate) port: PortName,
    pub(crate) data: PData,
    pub(crate) meta: Meta,
    next_probe_at: Instant,
}

impl<PData, Meta> PendingRoute<PData, Meta> {
    fn new(port: PortName, data: PData, meta: Meta, now: Instant) -> Self {
        Self {
            port,
            data,
            meta,
            next_probe_at: now + PROBE_INTERVAL,
        }
    }

    fn is_due(&self, now: Instant) -> bool {
        self.next_probe_at <= now
    }

    pub(crate) fn into_parts(self) -> (PortName, PData, Meta) {
        (self.port, self.data, self.meta)
    }

    pub(crate) fn from_retry_parts(port: PortName, data: PData, meta: Meta, now: Instant) -> Self {
        Self::new(port, data, meta, now)
    }
}

/// Result of handling a selected-route `Full` outcome.
pub(crate) enum FullRouteHandling<PData> {
    /// Reject the message immediately.
    ImmediateNack(PData),
    /// The message was parked locally and will be resumed by wakeup.
    Parked,
}

/// Shared parked-state scheduler for exclusive routers.
pub(crate) struct ExclusiveRouteScheduler<PData, Meta> {
    policy: SelectedRouteAdmissionPolicy,
    pause_candidate_ports: HashSet<PortName>,
    pending_by_port: HashMap<PortName, PendingRoute<PData, Meta>>,
    armed_wakeup: Option<ArmedWakeup>,
}

impl<PData, Meta> ExclusiveRouteScheduler<PData, Meta> {
    #[must_use]
    pub fn new(policy: SelectedRouteAdmissionPolicy) -> Self {
        Self {
            policy,
            pause_candidate_ports: HashSet::new(),
            pending_by_port: HashMap::new(),
            armed_wakeup: None,
        }
    }

    #[must_use]
    pub const fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        self.policy.runtime_requirements()
    }

    /// Record the set of outputs that can currently be selected by the router.
    pub fn observe_pause_candidate_ports<I>(&mut self, ports: I)
    where
        I: IntoIterator<Item = PortName>,
    {
        if self.policy.on_full != OnFullPolicy::Backpressure {
            return;
        }

        self.pause_candidate_ports.clear();
        self.pause_candidate_ports.extend(ports);
    }

    /// Return whether the router should accept new pdata right now.
    #[must_use]
    pub fn accept_pdata(&self) -> bool {
        match self.policy.on_full {
            OnFullPolicy::RejectImmediately => true,
            OnFullPolicy::Backpressure => !self.all_pause_candidates_blocked(),
        }
    }

    /// Handle a selected-route `Full` from the immediate routing path.
    pub fn handle_selected_route_full(
        &mut self,
        port: PortName,
        data: PData,
        meta: Meta,
        effect_handler: &EffectHandler<PData>,
    ) -> Result<FullRouteHandling<PData>, WakeupError> {
        let now = Instant::now();

        match self.policy.on_full {
            OnFullPolicy::RejectImmediately => Ok(FullRouteHandling::ImmediateNack(data)),
            OnFullPolicy::Backpressure => {
                if self.pending_by_port.contains_key(&port) {
                    return Ok(FullRouteHandling::ImmediateNack(data));
                }

                let pending = PendingRoute::new(port.clone(), data, meta, now);
                let _ = self.pending_by_port.insert(port.clone(), pending);
                if let Err(error) = self.sync_armed_wakeup(effect_handler) {
                    let pending = self
                        .pending_by_port
                        .remove(&port)
                        // Safe: the pending route was inserted for this same
                        // port immediately above, and no await or re-entrant
                        // scheduler call can remove it before this rollback.
                        .expect("inserted pending route should be present");
                    return match error {
                        WakeupError::ShuttingDown => {
                            let _ = self.pending_by_port.insert(port, pending);
                            Ok(FullRouteHandling::Parked)
                        }
                        other => Err(other),
                    };
                }
                Ok(FullRouteHandling::Parked)
            }
        }
    }

    /// Accept the current wakeup if it matches the armed slot/revision and
    /// return every parked route whose retry is now due.
    pub fn take_due_routes(
        &mut self,
        slot: WakeupSlot,
        revision: WakeupRevision,
        now: Instant,
    ) -> Vec<PendingRoute<PData, Meta>> {
        if !self.accept_wakeup(slot, revision) {
            return Vec::new();
        }

        match self.policy.on_full {
            OnFullPolicy::RejectImmediately => Vec::new(),
            OnFullPolicy::Backpressure => {
                let due_ports: Vec<PortName> = self
                    .pending_by_port
                    .iter()
                    .filter_map(|(port, pending)| pending.is_due(now).then_some(port.clone()))
                    .collect();
                due_ports
                    .into_iter()
                    .filter_map(|port| self.pending_by_port.remove(&port))
                    .collect()
            }
        }
    }

    /// Re-park a still-full selected route after wakeup re-probing.
    pub fn repark_after_full(&mut self, pending: PendingRoute<PData, Meta>) {
        if self.policy.on_full == OnFullPolicy::Backpressure {
            let _ = self.pending_by_port.insert(pending.port.clone(), pending);
        }
    }

    /// Re-arm or cancel the shared wakeup to match current parked state.
    pub fn sync_armed_wakeup(
        &mut self,
        effect_handler: &EffectHandler<PData>,
    ) -> Result<(), WakeupError> {
        let Some(when) = self.desired_wakeup_at() else {
            if self.armed_wakeup.is_some() {
                let _ = effect_handler.cancel_wakeup(EXCLUSIVE_ROUTER_WAKEUP_SLOT);
                self.armed_wakeup = None;
            }
            return Ok(());
        };

        if self.armed_wakeup.is_some_and(|armed| armed.when == when) {
            return Ok(());
        }

        match effect_handler.set_wakeup(EXCLUSIVE_ROUTER_WAKEUP_SLOT, when) {
            Ok(outcome) => {
                self.armed_wakeup = Some(ArmedWakeup {
                    when,
                    revision: outcome.revision(),
                });
                Ok(())
            }
            Err(WakeupError::ShuttingDown) => {
                self.armed_wakeup = None;
                Ok(())
            }
            Err(other) => Err(other),
        }
    }

    /// Drain all locally parked messages for shutdown entry.
    pub fn drain_for_shutdown(
        &mut self,
        effect_handler: &EffectHandler<PData>,
    ) -> Vec<PendingRoute<PData, Meta>> {
        let _ = effect_handler.cancel_wakeup(EXCLUSIVE_ROUTER_WAKEUP_SLOT);
        self.armed_wakeup = None;

        self.pending_by_port
            .drain()
            .map(|(_, pending)| pending)
            .collect()
    }

    fn all_pause_candidates_blocked(&self) -> bool {
        !self.pause_candidate_ports.is_empty()
            && self
                .pause_candidate_ports
                .iter()
                .all(|port| self.pending_by_port.contains_key(port))
    }

    fn desired_wakeup_at(&self) -> Option<Instant> {
        match self.policy.on_full {
            OnFullPolicy::RejectImmediately => None,
            OnFullPolicy::Backpressure => self
                .pending_by_port
                .values()
                .map(|pending| pending.next_probe_at)
                .min(),
        }
    }

    fn accept_wakeup(&mut self, slot: WakeupSlot, revision: WakeupRevision) -> bool {
        if slot != EXCLUSIVE_ROUTER_WAKEUP_SLOT {
            return false;
        }

        let Some(armed_wakeup) = self.armed_wakeup else {
            return false;
        };

        if armed_wakeup.revision != revision {
            return false;
        }

        self.armed_wakeup = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_reject_immediately() {
        assert_eq!(
            SelectedRouteAdmissionPolicy::default().on_full,
            OnFullPolicy::RejectImmediately
        );
    }

    #[test]
    fn backpressure_requests_local_wakeup() {
        let policy = SelectedRouteAdmissionPolicy {
            on_full: OnFullPolicy::Backpressure,
        };
        assert_eq!(
            policy.runtime_requirements(),
            ProcessorRuntimeRequirements::with_local_wakeups(1)
        );
    }
}
