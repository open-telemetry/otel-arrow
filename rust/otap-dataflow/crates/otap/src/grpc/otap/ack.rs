// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ACK/NACK subscriptions for OTAP gRPC streams.

use crate::accessory::slots::{Key as SlotKey, State as SlotsState};
use crate::pdata::OtapPdata;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::control::{CallData, NackMsg};
use otel_arrow_dfe_pdata_codec::OtapPayload;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Shared state for binding requests with responses when wait-for-result is enabled.
#[derive(Clone)]
pub struct SharedState(
    // Transport tasks and receiver control handling may run on different threads.
    pub(crate) Arc<Mutex<SlotsState<oneshot::Sender<Result<(), NackMsg<OtapPdata>>>>>>,
);

impl SharedState {
    pub(crate) fn new(max_size: usize) -> Self {
        Self(Arc::new(Mutex::new(SlotsState::new(max_size))))
    }

    /// Returns true when there are no outstanding wait-for-result slots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.lock().map(|state| state.is_empty()).unwrap_or(true)
    }

    /// Completes all outstanding waiters with a shutdown Nack.
    pub fn force_shutdown(&self, signal: SignalType, reason: &str) {
        if let Ok(mut state) = self.0.lock() {
            state.drain(|sender| {
                let _ = sender.send(Err(NackMsg::new(
                    reason,
                    OtapPdata::new_todo_context(OtapPayload::empty(signal)),
                )));
            });
        }
    }

    /// Routes responses to subscription slots.
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
        let chan = self
            .0
            .lock()
            .map(|mut state| state.take(key))
            .ok()
            .flatten();

        if chan.and_then(|sender| sender.send(result).ok()).is_some() {
            RouteResponse::Sent
        } else {
            RouteResponse::Expired
        }
    }
}

/// The outcome from routing an ACK/NACK to an OTAP stream batch.
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

/// Cancels a subscription when Tonic drops a waiting future.
pub(crate) struct SlotGuard {
    pub(crate) key: SlotKey,
    pub(crate) state: SharedState,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.0.lock() {
            state.cancel(self.key);
        }
    }
}
