// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ack/Nack bookkeeping for the experimental OTEL receiver.
//!
//! The code in this module owns the `AckRegistry` and related helpers that turn
//! pipeline notifications back into OTAP `BatchStatus` responses. Keeping the
//! state machine isolated here lets the higher-level router focus purely on gRPC
//! transport concerns while this module tracks slot lifetimes, wakeups, and
//! status message formatting.

use crate::otap_grpc::otlp::server::RouteResponse;
use crate::pdata::OtapPdata;
use otap_df_config::SignalType;
use otap_df_engine::control::{AckMsg, CallData, Context8u8, NackMsg};
use otap_df_pdata::proto::opentelemetry::arrow::v1::{BatchStatus, StatusCode as ProtoStatusCode};
use smallvec::smallvec;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::task::{Context as TaskContext, Poll, Waker};

pub(crate) enum AckPollResult {
    Ack,
    Nack(String),
    Cancelled,
}

#[derive(Clone)]
pub(crate) struct AckRegistry {
    inner: Rc<RefCell<AckRegistryInner>>,
}

struct AckRegistryInner {
    slots: Box<[AckSlot]>,
    free_stack: Vec<usize>,
}

impl AckRegistry {
    pub(crate) fn new(max_size: usize) -> Self {
        let mut slots = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            slots.push(AckSlot::new());
        }
        let mut free_stack = Vec::with_capacity(max_size);
        for idx in (0..max_size).rev() {
            free_stack.push(idx);
        }
        Self {
            inner: Rc::new(RefCell::new(AckRegistryInner {
                slots: slots.into_boxed_slice(),
                free_stack,
            })),
        }
    }

    pub(crate) fn allocate(&self) -> Option<AckToken> {
        let mut inner = self.inner.borrow_mut();
        let slot_index = inner.free_stack.pop()?;
        let slot = &mut inner.slots[slot_index];
        debug_assert!(matches!(slot.state, SlotState::Free));
        slot.generation = slot.generation.wrapping_add(1);
        slot.state = SlotState::Waiting(WaitingSlot::new());
        Some(AckToken {
            slot_index,
            generation: slot.generation,
        })
    }

    pub(crate) fn complete(&self, token: AckToken, result: Result<(), String>) -> RouteResponse {
        let mut inner = self.inner.borrow_mut();
        let Some(slot) = inner.slots.get_mut(token.slot_index) else {
            return RouteResponse::Invalid;
        };
        if slot.generation != token.generation {
            return RouteResponse::Expired;
        }
        match &mut slot.state {
            SlotState::Waiting(waiting) => {
                waiting.outcome = match result {
                    Ok(()) => AckOutcome::Ack,
                    Err(reason) => AckOutcome::Nack(reason),
                };
                if let Some(waker) = waiting.waker.take() {
                    waker.wake();
                }
                RouteResponse::Sent
            }
            SlotState::Free => RouteResponse::Expired,
        }
    }

    pub(crate) fn poll_slot(
        &self,
        token: AckToken,
        cx: &mut TaskContext<'_>,
    ) -> Poll<AckPollResult> {
        let mut inner = self.inner.borrow_mut();
        let Some(slot) = inner.slots.get_mut(token.slot_index) else {
            return Poll::Ready(AckPollResult::Cancelled);
        };
        if slot.generation != token.generation {
            return Poll::Ready(AckPollResult::Cancelled);
        }
        match &mut slot.state {
            SlotState::Waiting(waiting) => match &mut waiting.outcome {
                AckOutcome::Pending => {
                    let replace = match &waiting.waker {
                        Some(existing) => !existing.will_wake(cx.waker()),
                        None => true,
                    };
                    if replace {
                        waiting.waker = Some(cx.waker().clone());
                    }
                    Poll::Pending
                }
                AckOutcome::Ack => {
                    slot.state = SlotState::Free;
                    inner.free_stack.push(token.slot_index);
                    Poll::Ready(AckPollResult::Ack)
                }
                AckOutcome::Nack(reason) => {
                    let reason = mem::take(reason);
                    slot.state = SlotState::Free;
                    inner.free_stack.push(token.slot_index);
                    Poll::Ready(AckPollResult::Nack(reason))
                }
            },
            SlotState::Free => Poll::Ready(AckPollResult::Cancelled),
        }
    }

    pub(crate) fn cancel(&self, token: AckToken) {
        let mut inner = self.inner.borrow_mut();
        if let Some(slot) = inner.slots.get_mut(token.slot_index) {
            if slot.generation != token.generation {
                return;
            }
            if matches!(slot.state, SlotState::Waiting(_)) {
                slot.state = SlotState::Free;
                inner.free_stack.push(token.slot_index);
            }
        }
    }
}

struct AckSlot {
    generation: u32,
    state: SlotState,
}

impl AckSlot {
    fn new() -> Self {
        Self {
            generation: 0,
            state: SlotState::Free,
        }
    }
}

enum SlotState {
    Free,
    Waiting(WaitingSlot),
}

struct WaitingSlot {
    waker: Option<Waker>,
    outcome: AckOutcome,
}

impl WaitingSlot {
    fn new() -> Self {
        Self {
            waker: None,
            outcome: AckOutcome::Pending,
        }
    }
}

enum AckOutcome {
    Pending,
    Ack,
    Nack(String),
}

#[derive(Clone, Copy)]
pub(crate) struct AckToken {
    slot_index: usize,
    generation: u32,
}

impl AckToken {
    pub(crate) fn to_calldata(self) -> CallData {
        smallvec![
            Context8u8::from(self.slot_index as u64),
            Context8u8::from(self.generation as u64)
        ]
    }

    pub(crate) fn from_calldata(calldata: &CallData) -> Option<Self> {
        if calldata.len() < 2 {
            return None;
        }
        let slot_index = usize::try_from(u64::from(calldata[0])).ok()?;
        let generation = u64::from(calldata[1]) as u32;
        Some(Self {
            slot_index,
            generation,
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct AckRegistries {
    logs: Option<AckRegistry>,
    metrics: Option<AckRegistry>,
    traces: Option<AckRegistry>,
}

impl AckRegistries {
    pub(crate) fn new(
        logs: Option<AckRegistry>,
        metrics: Option<AckRegistry>,
        traces: Option<AckRegistry>,
    ) -> Self {
        Self {
            logs,
            metrics,
            traces,
        }
    }

    pub(crate) fn ack_registry_for_signal(&self, signal: SignalType) -> Option<&AckRegistry> {
        match signal {
            SignalType::Logs => self.logs.as_ref(),
            SignalType::Metrics => self.metrics.as_ref(),
            SignalType::Traces => self.traces.as_ref(),
        }
    }
}

pub(crate) fn route_local_ack_response(
    states: &AckRegistries,
    ack: AckMsg<OtapPdata>,
) -> RouteResponse {
    let Some(token) = AckToken::from_calldata(&ack.calldata) else {
        return RouteResponse::Invalid;
    };
    states
        .ack_registry_for_signal(ack.accepted.signal_type())
        .map(|state| state.complete(token, Ok(())))
        .unwrap_or(RouteResponse::None)
}

pub(crate) fn route_local_nack_response(
    states: &AckRegistries,
    nack: NackMsg<OtapPdata>,
) -> RouteResponse {
    let Some(token) = AckToken::from_calldata(&nack.calldata) else {
        return RouteResponse::Invalid;
    };
    states
        .ack_registry_for_signal(nack.refused.signal_type())
        .map(|state| state.complete(token, Err(nack.reason)))
        .unwrap_or(RouteResponse::None)
}

pub(crate) fn success_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }
}

pub(crate) fn nack_status(batch_id: i64, reason: String) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: format!("Pipeline processing failed: {reason}"),
    }
}

pub(crate) fn overloaded_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: "Pipeline processing failed: Too many concurrent requests".to_string(),
    }
}
