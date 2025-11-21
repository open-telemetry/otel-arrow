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
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context as TaskContext, Poll, Waker};

/// Result returned when polling a registry slot for completion.
///
/// This is used by both the streaming and unary paths:
/// - `Ack` means the pipeline processed the batch successfully,
/// - `Nack(reason)` means it failed with the provided reason, and
/// - `Cancelled` means the slot was reclaimed without a concrete outcome
///   (for example because the wait future was dropped).
pub(crate) enum AckPollResult {
    Ack,
    Nack(String),
    Cancelled,
}

/// Fixed size slab of wait slots used to correlate `Ack` and `Nack` control messages
/// with in flight requests.
///
/// The registry:
/// - lives entirely on the single threaded runtime (`Rc<RefCell<_>>`),
/// - provides O(1) allocation and free via an intrusive free list, and
/// - participates directly in backpressure, since `allocate` returns `None`
///   when the slab is full.
#[derive(Clone)]
pub(crate) struct AckRegistry {
    inner: Rc<RefCell<AckRegistryInner>>,
}

/// We pre-allocate a boxed slice of slots so indices remain stable and
/// `AckToken` can cheaply refer to one.
/// `head_free` points to the index of the first available slot in the intrusive list.
struct AckRegistryInner {
    slots: Box<[AckSlot]>,
    head_free: Option<usize>,
}

impl AckRegistry {
    pub(crate) fn new(max_size: usize) -> Self {
        let mut slots = Vec::with_capacity(max_size);

        // Initialize the intrusive list.
        // Each slot points to the next one: 0 -> 1 -> 2 ...
        // The last slot points to None.
        for i in 0..max_size {
            let next_free = if i < max_size - 1 { Some(i + 1) } else { None };
            slots.push(AckSlot {
                generation: 0,
                state: SlotState::Free { next_free },
            });
        }

        let head_free = if max_size > 0 { Some(0) } else { None };

        Self {
            inner: Rc::new(RefCell::new(AckRegistryInner {
                slots: slots.into_boxed_slice(),
                head_free,
            })),
        }
    }

    /// Attempts to allocate a free slot, returning its token on success.
    /// O(1) operation: pops from the head of the intrusive linked list.
    pub(crate) fn allocate(&self) -> Option<AckToken> {
        let mut inner = self.inner.borrow_mut();

        // Check if we have any free slots available
        let slot_index = inner.head_free?;

        // 1. Extract the next pointer from the current free slot.
        // We peek at the slot state. We CANNOT hold 'slot' mutable ref here while writing to head_free later.
        let next_free = match &inner.slots[slot_index].state {
            SlotState::Free { next_free } => *next_free,
            _ => unreachable!("Corrupted AckRegistry: head_free pointed to a non-free slot"),
        };

        // 2. Update the head pointer.
        inner.head_free = next_free;

        // 3. Initialize the slot for use.
        // Now we can borrow 'slots' mutably again.
        let slot = &mut inner.slots[slot_index];
        slot.generation = slot.generation.wrapping_add(1);
        slot.state = SlotState::Waiting(WaitingSlot::new());

        Some(AckToken {
            slot_index,
            generation: slot.generation,
        })
    }

    /// Marks the slot as completed with the provided outcome, waking any waiter.
    pub(crate) fn complete(&self, token: AckToken, result: Result<(), String>) -> RouteResponse {
        let waker_opt = {
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
                    waiting.waker.take()
                }
                SlotState::Free { .. } => return RouteResponse::Expired,
            }
        };

        if let Some(waker) = waker_opt {
            waker.wake();
        }

        RouteResponse::Sent
    }

    /// Polls the slot, registering the waker if it is still pending.
    /// If the slot is finished (Ack/Nack), it returns Ready and immediately frees the slot
    /// back to the intrusive list.
    pub(crate) fn poll_slot(
        &self,
        token: AckToken,
        cx: &mut TaskContext<'_>,
    ) -> Poll<AckPollResult> {
        let mut inner = self.inner.borrow_mut();

        // 1. Check the state of the slot.
        // We scope this block so the mutable borrow of `inner.slots` ends before we call `free_slot_inner`.
        let result_to_process = {
            let slot = match inner.slots.get_mut(token.slot_index) {
                Some(s) => s,
                None => return Poll::Ready(AckPollResult::Cancelled),
            };

            if slot.generation != token.generation {
                return Poll::Ready(AckPollResult::Cancelled);
            }

            match &mut slot.state {
                SlotState::Free { .. } => return Poll::Ready(AckPollResult::Cancelled),
                SlotState::Waiting(waiting) => match &mut waiting.outcome {
                    AckOutcome::Pending => {
                        // Still pending: update waker and return.
                        let replace = match &waiting.waker {
                            Some(existing) => !existing.will_wake(cx.waker()),
                            None => true,
                        };
                        if replace {
                            waiting.waker = Some(cx.waker().clone());
                        }
                        return Poll::Pending;
                    }
                    // Completed: Return the result so we can free outside this block.
                    AckOutcome::Ack => Ok(()),
                    AckOutcome::Nack(reason) => Err(mem::take(reason)),
                },
            }
        };

        // 2. If we are here, the slot is done (Ack or Nack). We must free it.
        // The previous borrow of `inner.slots` (via `slot`) is dropped.
        Self::free_slot_inner(&mut inner, token.slot_index);

        match result_to_process {
            Ok(()) => Poll::Ready(AckPollResult::Ack),
            Err(reason) => Poll::Ready(AckPollResult::Nack(reason)),
        }
    }

    /// Cancels the slot if it is still waiting (e.g. drop without completion).
    pub(crate) fn cancel(&self, token: AckToken) {
        let mut inner = self.inner.borrow_mut();

        // 1. Check if we need to free.
        // We use a read-only check first to avoid conflicts, though we hold `mut inner` anyway.
        // The key is that we don't hold a reference to `slots` when calling `free_slot_inner`.
        let should_free = if let Some(slot) = inner.slots.get(token.slot_index) {
            if slot.generation != token.generation {
                false
            } else {
                matches!(slot.state, SlotState::Waiting(_))
            }
        } else {
            false
        };

        if should_free {
            Self::free_slot_inner(&mut inner, token.slot_index);
        }
    }

    /// Helper: transitions a slot at `index` to Free and pushes it onto the head
    /// of the free list (LIFO).
    fn free_slot_inner(inner: &mut AckRegistryInner, index: usize) {
        let old_head = inner.head_free;
        inner.slots[index].state = SlotState::Free {
            next_free: old_head,
        };
        inner.head_free = Some(index);
    }
}

/// Future that resolves once the provided slot receives an ACK/NACK (or is cancelled).
pub(crate) struct AckCompletionFuture {
    token: AckToken,
    state: AckRegistry,
    completed: bool,
}

impl AckCompletionFuture {
    pub(crate) fn new(token: AckToken, state: AckRegistry) -> Self {
        Self {
            token,
            state,
            completed: false,
        }
    }
}

impl Future for AckCompletionFuture {
    type Output = AckPollResult;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.state.poll_slot(this.token, cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                this.completed = true;
                Poll::Ready(result)
            }
        }
    }
}

impl Drop for AckCompletionFuture {
    fn drop(&mut self) {
        if !self.completed {
            self.state.cancel(self.token);
        }
    }
}

/// Individual slot that may be free or waiting for a result.
struct AckSlot {
    generation: u32,
    state: SlotState,
}

/// Tracks whether a slot is unused (pointing to next free) or actively waiting.
enum SlotState {
    Free { next_free: Option<usize> },
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

/// Compact handle that identifies a single registry slot.
///
/// The token is passed downstream as `CallData` and later reconstructed when
/// an `Ack` or `Nack` control message arrives.
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
pub(crate) fn route_ack_response(states: &AckRegistries, ack: AckMsg<OtapPdata>) -> RouteResponse {
    let Some(token) = AckToken::from_calldata(&ack.calldata) else {
        return RouteResponse::Invalid;
    };
    states
        .ack_registry_for_signal(ack.accepted.signal_type())
        .map(|state| state.complete(token, Ok(())))
        .unwrap_or(RouteResponse::None)
}

/// Routes a Nack control message back into the appropriate registry.
pub(crate) fn route_nack_response(
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
