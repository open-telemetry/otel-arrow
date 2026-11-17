// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transport-independent request subscriptions and ACK/NACK routing.

use crate::accessory::slots::{Key as SlotKey, State as SlotsState};
use crate::pdata::OtapPdata;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::control::{AckMsg, CallData, NackMsg};
use otel_arrow_dfe_pdata_codec::OtapPayload;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Tracks outstanding request subscriptions for a single signal.
#[derive(Clone)]
pub struct AckSlot(
    // Transport tasks and the receiver event loop route completions across threads.
    // parking_lot preserves the existing non-poisoning behavior.
    pub(crate) Arc<Mutex<SlotsState<oneshot::Sender<Result<(), NackMsg<OtapPdata>>>>>>,
);

/// Receiver side of a wait-for-result subscription slot.
pub type AckSlotReceiver = oneshot::Receiver<Result<(), NackMsg<OtapPdata>>>;

impl AckSlot {
    /// Build a new per-signal slot map sized for the configured concurrency.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self(Arc::new(Mutex::new(SlotsState::new(max_size))))
    }

    /// Returns true when there are no outstanding wait-for-result slots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.lock().is_empty()
    }

    /// Completes all outstanding waiters with a shutdown Nack.
    pub fn force_shutdown(&self, signal: SignalType, reason: &str) {
        self.0.lock().drain(|sender| {
            let _ = sender.send(Err(NackMsg::new(
                reason,
                OtapPdata::new_todo_context(OtapPayload::empty(signal)),
            )));
        });
    }

    /// Allocate a raw slot key plus its paired receiver.
    pub(crate) fn allocate_slot(&self) -> Option<(SlotKey, AckSlotReceiver)> {
        let mut guard = self.0.lock();
        guard.allocate(oneshot::channel)
    }

    /// Allocate a wait-for-result subscription in calldata form.
    #[must_use]
    pub fn allocate_waiter(&self) -> Option<(CallData, AckSlotReceiver)> {
        self.allocate_slot().map(|(key, rx)| (key.into(), rx))
    }

    /// Cancel an outstanding wait-for-result subscription.
    pub(crate) fn cancel_slot(&self, key: SlotKey) {
        self.0.lock().cancel(key);
    }

    /// Routes the final outcome into the registered slot matching the provided `CallData`.
    #[must_use]
    pub fn route_response(
        &self,
        calldata: CallData,
        result: Result<(), NackMsg<OtapPdata>>,
    ) -> RouteResponse {
        let key: SlotKey = match calldata.try_into() {
            Ok(data) => data,
            Err(_) => return RouteResponse::Invalid,
        };
        let chan = self.0.lock().take(key);

        if chan.and_then(|sender| sender.send(result).ok()).is_some() {
            RouteResponse::Sent
        } else {
            RouteResponse::Expired
        }
    }
}

/// The outcome from routing an ACK/NACK to a request.
pub enum RouteResponse {
    /// The Ack/Nack was sent.
    Sent,
    /// The Ack/Nack may have timed out.
    Expired,
    /// No subscription was found.
    None,
    /// The Ack/Nack had invalid call data.
    Invalid,
}

/// Cancels a subscription when its waiting request is dropped.
pub(crate) struct SlotGuard {
    pub(crate) key: SlotKey,
    pub(crate) state: AckSlot,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.state.cancel_slot(self.key);
    }
}

/// Aggregates the per-signal ACK subscription maps that let us route responses back to callers.
#[derive(Clone, Default)]
pub struct AckRegistry {
    /// Subscription map for log acknowledgements.
    pub logs: Option<AckSlot>,
    /// Subscription map for metric acknowledgements.
    pub metrics: Option<AckSlot>,
    /// Subscription map for trace acknowledgements.
    pub traces: Option<AckSlot>,
}

impl AckRegistry {
    /// Creates a new bundle of optional subscription maps.
    #[must_use]
    pub const fn new(
        logs: Option<AckSlot>,
        metrics: Option<AckSlot>,
        traces: Option<AckSlot>,
    ) -> Self {
        Self {
            logs,
            metrics,
            traces,
        }
    }

    /// Returns true when all subscription maps are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.logs.as_ref().is_none_or(AckSlot::is_empty)
            && self.metrics.as_ref().is_none_or(AckSlot::is_empty)
            && self.traces.as_ref().is_none_or(AckSlot::is_empty)
    }

    /// Completes all outstanding wait-for-result slots with a shutdown Nack.
    pub fn force_shutdown(&self, reason: &str) {
        if let Some(slot) = &self.logs {
            slot.force_shutdown(SignalType::Logs, reason);
        }
        if let Some(slot) = &self.metrics {
            slot.force_shutdown(SignalType::Metrics, reason);
        }
        if let Some(slot) = &self.traces {
            slot.force_shutdown(SignalType::Traces, reason);
        }
    }
}

/// Routes an Ack message to the appropriate signal's subscription map.
#[must_use]
pub fn route_ack_response(states: &AckRegistry, ack: AckMsg<OtapPdata>) -> RouteResponse {
    let calldata = ack.unwind.route.calldata;
    let resp = Ok(());
    let state = match ack.accepted.signal_type() {
        SignalType::Logs => states.logs.as_ref(),
        SignalType::Metrics => states.metrics.as_ref(),
        SignalType::Traces => states.traces.as_ref(),
    };

    state
        .map(|s| s.route_response(calldata, resp))
        .unwrap_or(RouteResponse::None)
}

/// Routes a Nack message to the appropriate shared state.
#[must_use]
pub fn route_nack_response(states: &AckRegistry, mut nack: NackMsg<OtapPdata>) -> RouteResponse {
    let calldata = std::mem::take(&mut nack.unwind.route.calldata);
    let signal_type = nack.refused.signal_type();
    let resp = Err(nack);
    let state = match signal_type {
        SignalType::Logs => states.logs.as_ref(),
        SignalType::Metrics => states.metrics.as_ref(),
        SignalType::Traces => states.traces.as_ref(),
    };

    state
        .map(|s| s.route_response(calldata, resp))
        .unwrap_or(RouteResponse::None)
}

/// Handles the outcome from routing an Ack/Nack response.
pub fn handle_route_response<T, F, G>(
    resp: RouteResponse,
    state: &mut T,
    mut on_sent: F,
    mut on_expired_or_invalid: G,
) where
    F: FnMut(&mut T),
    G: FnMut(&mut T),
{
    match resp {
        RouteResponse::Sent => on_sent(state),
        RouteResponse::Expired | RouteResponse::Invalid => on_expired_or_invalid(state),
        RouteResponse::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a waiting transport request is dropped and its slot is reused.
    /// Guarantees: cancellation frees capacity and a late response cannot complete the new waiter.
    #[test]
    fn cancelled_slot_does_not_route_to_replacement() {
        let state = AckSlot::new(1);
        let (key, mut cancelled_rx) = state.allocate_slot().unwrap();
        let cancelled_data: CallData = key.into();
        let guard = SlotGuard {
            key,
            state: state.clone(),
        };
        assert!(state.allocate_waiter().is_none());
        drop(guard);
        assert_eq!(
            cancelled_rx.try_recv().unwrap_err(),
            oneshot::error::TryRecvError::Closed
        );

        let (replacement_data, mut replacement_rx) = state.allocate_waiter().unwrap();
        assert!(matches!(
            state.route_response(cancelled_data, Ok(())),
            RouteResponse::Expired
        ));
        assert_eq!(
            replacement_rx.try_recv().unwrap_err(),
            oneshot::error::TryRecvError::Empty
        );
        assert!(matches!(
            state.route_response(replacement_data, Ok(())),
            RouteResponse::Sent
        ));
        assert!(replacement_rx.try_recv().unwrap().is_ok());
        assert!(state.is_empty());
    }

    /// Scenario: a receiver shuts down with outstanding subscriptions for every signal.
    /// Guarantees: all waiters receive a NACK with their signal and reason, and the registry drains.
    #[test]
    fn registry_shutdown_completes_all_signals() {
        let registry = AckRegistry::new(
            Some(AckSlot::new(1)),
            Some(AckSlot::new(1)),
            Some(AckSlot::new(1)),
        );
        let mut waiters = [
            (SignalType::Logs, registry.logs.as_ref().unwrap()),
            (SignalType::Metrics, registry.metrics.as_ref().unwrap()),
            (SignalType::Traces, registry.traces.as_ref().unwrap()),
        ]
        .map(|(signal, slot)| (signal, slot.allocate_waiter().unwrap().1));

        assert!(!registry.is_empty());
        registry.force_shutdown("receiver stopped");
        assert!(registry.is_empty());
        for (signal, rx) in &mut waiters {
            let nack = rx.try_recv().unwrap().unwrap_err();
            assert_eq!(nack.refused.signal_type(), *signal);
            assert_eq!(nack.reason, "receiver stopped");
        }
    }
}
