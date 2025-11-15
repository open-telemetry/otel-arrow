// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ack/Nack bookkeeping for the experimental OTAP receiver.
//!
//! The receiver runs in the thread-per-core engine where tasks, channels, and
//! effect handlers are NOT `Send`. Instead of locking, we rely on `Rc` +
//! `RefCell` and the fact that each registry is only ever touched from the same
//! single-threaded async runtime. The registry acts like a tiny slab of "wait
//! slots", and we bound the number of slots to participate in backpressure: if
//! no slots remain, new batches get an immediate "Too many concurrent requests"
//! status, preventing runaway resource use. When a request wants an ACK/NACK,
//! it allocates a slot, subscribes, and passes the token downstream. Later, the
//! pipeline produces a `NodeControlMsg::Ack/Nack`, the token is recovered, and
//! the waiting future resolves back into a `BatchStatus`. Keeping this state
//! machine isolated here lets the router focus purely on gRPC/H2 plumbing while
//! this module tracks slot lifetimes, wakers, and status formatting.
//!
//! General design goals:
//! - O(1) allocate/free so the hot path stays predictable.
//! - No locking or atomic work, since nothing ever leaves the local executor.
//! - Strict control over memory/concurrency so we can apply backpressure when
//!   the pipeline can't keep up.

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

/// Result returned when polling a registry slot for completion.
pub(crate) enum AckPollResult {
    Ack,
    Nack(String),
    Cancelled,
}

/// Manages the fixed-size set of wait slots used to correlate ACK/NACK responses.
/// Relies on `Rc<RefCell<_>>` because it only lives within the single-threaded receiver task.
#[derive(Clone)]
pub(crate) struct AckRegistry {
    inner: Rc<RefCell<AckRegistryInner>>,
}

/// We pre-allocate a boxed slice of slots so indices remain stable and
/// `AckToken` can cheaply refer to one.  `free_stack` is a simple LIFO of
/// currently unused slot indices: allocate = `pop()`, complete/cancel =
/// `push()`.  Because the registry is tiny and single-threaded, this gives us
/// O(1) operations with minimal bookkeeping.
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

    /// Attempts to allocate a free slot, returning its token on success.
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

    /// Marks the slot as completed with the provided outcome, waking any waiter.
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

    /// Polls the slot, registering the waker if it is still pending.
    ///
    /// This simply inspects the slot outcome and, when still pending, stores the
    /// current task’s waker so the eventual `complete` call can wake the same
    /// future. The returned `AckPollResult` drives the higher-level
    /// `AckWaitFuture` to emit the correct `BatchStatus`.
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

    /// Cancels the slot if it is still waiting (e.g. drop without completion).
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

/// Individual slot that may be free or waiting for a result.
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

/// Tracks whether a slot is unused or actively waiting.
enum SlotState {
    Free,
    Waiting(WaitingSlot),
}

/// Carrier for a waiting slot's waker and eventual outcome.
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

/// Final disposition for a slot once the pipeline responds.
enum AckOutcome {
    Pending,
    Ack,
    Nack(String),
}

/// Handle that flows through the pipeline to identify an Ack slot.
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

/// Convenience holder for the three per-signal registries.
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

/// Routes an Ack control message back into the appropriate registry.
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

/// Routes a Nack control message back into the appropriate registry.
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

/// Helper to produce the canonical success status used across signals.
pub(crate) fn success_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }
}

/// Helper to produce a nack status with the provided reason.
pub(crate) fn nack_status(batch_id: i64, reason: String) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: format!("Pipeline processing failed: {reason}"),
    }
}

/// Helper to produce the status returned when the registry runs out of slots.
pub(crate) fn overloaded_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: "Pipeline processing failed: Too many concurrent requests".to_string(),
    }
}
