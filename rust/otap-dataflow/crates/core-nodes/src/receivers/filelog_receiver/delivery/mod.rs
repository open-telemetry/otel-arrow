// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single retained-batch delivery. See `../docs/delivery.md` for worker boundaries.

use super::{
    config::{RetryExhaustion, ValidatedInputs},
    identity::{IdentityError, IdentityOwner},
};
use otel_arrow_dfe_channel::error::SendError;
use otel_arrow_dfe_engine::{
    Interests, MessageSourceLocalEffectHandlerExtension, ProducerEffectHandlerExtension,
    control::{CallData, Context8u8},
    error::TypedError,
};
use otel_arrow_dfe_filelog_checkpoint::{self as cp, store::Durability};
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use std::{
    future::Future,
    time::{Duration, Instant},
};

/// Borrowed view of the batch builder's retained output. No ownership transfer.
pub(crate) struct BatchView<'a> {
    pub records: &'a OtapArrowRecords,
    pub progress: &'a [cp::UpdateProgress],
}

/// Batch-builder access used by delivery without transferring retained ownership.
/// Implement with `retained()` and `release_applied()`; carry-over stays in that builder.
pub(crate) trait RetainedBatch {
    type Error: std::error::Error + Send + Sync + 'static;
    fn failure(&self) -> Option<Self::Error>;
    fn retained(&self) -> Option<BatchView<'_>>;
    fn release_applied(
        &mut self,
        applied: &[cp::UpdateProgress],
        now: Instant,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DeliveryError {
    #[error("unsupported filelog explicit-loss delivery policy")]
    UnsupportedLossPolicy,
    #[error("filelog delivery invariant violated")]
    Invariant,
    #[error("filelog delivery counter or deadline exhausted")]
    Exhausted,
    #[error("filelog delivery retry budget exhausted")]
    RetryExhausted,
    #[error("filelog batch builder failed")]
    Batch(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Checkpoint(#[from] IdentityError),
}

/// The component responsible for resolving an outstanding authority write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingOwner {
    Batch,
    Growth,
    Host,
}

/// Recoverable blocking, optionally carrying the failure that established it.
#[derive(Debug)]
pub(crate) struct Blocked {
    pub owner: PendingOwner,
    pub cause: Option<IdentityError>,
}
impl Blocked {
    fn waiting(owner: PendingOwner) -> Self {
        Self { owner, cause: None }
    }
}

/// Errors are terminal; a blocked result keeps the original batch owned and live.
#[derive(Debug)]
pub(crate) enum CheckpointOutcome {
    Applied(Vec<cp::UpdateProgress>),
    NotReady,
    Blocked(Blocked),
}

/// Required synchronization and optional maintenance have distinct outcomes.
#[derive(Debug)]
pub(crate) enum SyncOutcome {
    NotReady,
    Synced,
    Blocked(Blocked),
    MaintenanceRejected(IdentityError),
}

/// Foreign work needs a host wakeup; only growth initiated here has a local retry timer.
#[derive(Clone, Copy, Debug)]
enum StoreBlock {
    Host,
    Growth(Instant),
}

/// Opaque identity carried through the engine, not inferred from payload attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Correlation {
    batch: u64,
    attempt: u64,
}
impl Correlation {
    pub(crate) fn calldata(self) -> CallData {
        [Context8u8::from(self.batch), Context8u8::from(self.attempt)]
            .into_iter()
            .collect()
    }
    fn from_calldata(data: &CallData) -> Option<Self> {
        (data.len() == 2).then(|| Self {
            batch: data[0].into(),
            attempt: data[1].into(),
        })
    }
}

/// One shared-buffer clone for the bounded worker-to-pipeline handoff slot.
pub(crate) struct Attempt {
    pub correlation: Correlation,
    records: OtapArrowRecords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SendOutcome {
    Accepted,
    Retryable,
    Cancelled,
}

/// Run on the async pipeline task. Subscription precedes publication. Cancellation
/// drops only this attempt's clone; the worker continues owning unresolved data.
/// The caller must service lifecycle/control while this future is pending.
pub(crate) async fn send_attempt<H>(
    handler: &H,
    attempt: Attempt,
    cancel: impl Future<Output = ()>,
) -> Result<SendOutcome, TypedError<OtapPdata>>
where
    H: ProducerEffectHandlerExtension<OtapPdata>
        + MessageSourceLocalEffectHandlerExtension<OtapPdata>,
{
    let mut data = OtapPdata::new(Context::default(), attempt.records.into());
    handler.subscribe_to(
        Interests::ACKS | Interests::NACKS,
        attempt.correlation.calldata(),
        &mut data,
    );
    tokio::select! {
        biased;
        _ = cancel => Ok(SendOutcome::Cancelled),
        result = handler.send_message_with_source_node(data) => match result {
            Ok(()) => Ok(SendOutcome::Accepted),
            Err(TypedError::ChannelSendError(SendError::Closed(_))) => Ok(SendOutcome::Retryable),
            Err(error) => Err(error),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Sending,
    Awaiting,
    Backoff(Instant),
    Authorized,
    Resolving(Instant),
    Stopped,
}

/// Worker-local state; all checkpoint methods execute on the read/checkpoint worker.
/// No timers, channel queues or per-attempt history are allocated here. The host
/// services `next_deadline` even when sources are idle and gates reads on `builder_mut`.
pub(crate) struct Delivery<B> {
    builder: B,
    phase: Phase,
    correlation: Correlation,
    used: u64,
    max_attempts: u64,
    delay: Duration,
    initial_backoff: Duration,
    max_backoff: Duration,
    exhaustion: RetryExhaustion,
    sync_interval: Duration,
    sync_due: Option<Instant>,
    store_block: Option<StoreBlock>,
    pub stale_completions: u64,
}

impl<B: RetainedBatch> Delivery<B> {
    pub fn new(builder: B, inputs: &ValidatedInputs) -> Result<Self, DeliveryError> {
        let config = inputs.settings();
        // Loss authorization needs its own observable policy integration before use.
        if config.on_retry_exhaustion == RetryExhaustion::DropAndContinue {
            return Err(DeliveryError::UnsupportedLossPolicy);
        }
        Ok(Self {
            builder,
            phase: Phase::Idle,
            correlation: Correlation {
                batch: 0,
                attempt: 0,
            },
            used: 0,
            max_attempts: config.retry.max_attempts,
            delay: config.retry.initial_backoff,
            initial_backoff: config.retry.initial_backoff,
            max_backoff: config.retry.max_backoff,
            exhaustion: config.on_retry_exhaustion,
            sync_interval: config.checkpoint.sync_interval,
            sync_due: None,
            store_block: None,
            stale_completions: 0,
        })
    }

    pub fn is_stopped(&self) -> bool {
        self.phase == Phase::Stopped
    }

    /// Inspect unresolved builder state for scheduler pins without permitting source work.
    pub fn builder(&self) -> &B {
        &self.builder
    }

    /// Only the quiescent host may feed the builder or resume source scheduling.
    pub fn builder_mut(&mut self, now: Instant) -> Option<&mut B> {
        (self.phase == Phase::Idle
            && self.store_block.is_none()
            && self.builder.failure().is_none()
            && self.builder.retained().is_none()
            && self.sync_due.is_none_or(|at| now < at))
        .then_some(&mut self.builder)
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        if self.phase == Phase::Stopped {
            return None;
        }
        match self.store_block {
            Some(StoreBlock::Host) => return None,
            Some(StoreBlock::Growth(at)) => return Some(at),
            None => {}
        }
        let retry = match self.phase {
            Phase::Backoff(at) | Phase::Resolving(at) => Some(at),
            _ => None,
        };
        // An unresolved append must settle before sync can run; do not spin on an overdue sync.
        if matches!(self.phase, Phase::Resolving(_)) {
            return retry;
        }
        retry.into_iter().chain(self.sync_due).min()
    }

    /// Cancel is terminal for this coordinator and never releases the builder.
    /// Worker teardown may drop it only after all worker access has ceased.
    pub fn cancel(&mut self) {
        self.phase = Phase::Stopped;
    }

    fn fail<T>(&mut self, error: DeliveryError) -> Result<T, DeliveryError> {
        self.phase = Phase::Stopped;
        Err(error)
    }

    /// Creates at most one outstanding attempt. A pending send/completion cannot time out into Ack.
    pub fn next_attempt(&mut self, now: Instant) -> Result<Option<Attempt>, DeliveryError> {
        if self.is_stopped() {
            return Ok(None);
        }
        if let Some(error) = self.builder.failure() {
            return self.fail(DeliveryError::Batch(Box::new(error)));
        }
        if self.store_block.is_some() || self.sync_due.is_some_and(|at| now >= at) {
            return Ok(None);
        }
        match self.phase {
            Phase::Idle => {
                if self.builder.retained().is_none() {
                    return Ok(None);
                }
                let Some(batch) = self.correlation.batch.checked_add(1) else {
                    return self.fail(DeliveryError::Exhausted);
                };
                self.correlation = Correlation { batch, attempt: 0 };
                self.used = 0;
                self.delay = self.initial_backoff;
            }
            Phase::Backoff(at) if now >= at => {}
            _ => return Ok(None),
        }
        let Some(attempt) = self.correlation.attempt.checked_add(1) else {
            return self.fail(DeliveryError::Exhausted);
        };
        self.correlation.attempt = attempt;
        self.used = self.used.saturating_add(1).min(self.max_attempts);
        let Some(batch) = self.builder.retained() else {
            return self.fail(DeliveryError::Invariant);
        };
        if batch.progress.is_empty() || batch.progress.len() > cp::WAL_MAX_OPS_PER_TX as usize {
            return self.fail(DeliveryError::Invariant);
        }
        let records = batch.records.clone();
        self.phase = Phase::Sending;
        Ok(Some(Attempt {
            correlation: self.correlation,
            records,
        }))
    }

    /// Host forwards publication outcome before forwarding completions for this attempt.
    pub fn sent(
        &mut self,
        id: Correlation,
        outcome: SendOutcome,
        now: Instant,
    ) -> Result<(), DeliveryError> {
        if self.phase != Phase::Sending || self.correlation != id {
            return self.fail(DeliveryError::Invariant);
        }
        match outcome {
            SendOutcome::Accepted => self.phase = Phase::Awaiting,
            SendOutcome::Retryable => return self.retry(now),
            SendOutcome::Cancelled => self.cancel(),
        }
        Ok(())
    }

    /// Forward only engine aggregate Ack/Nack calldata, never individual destination outcomes.
    pub fn complete(
        &mut self,
        calldata: &CallData,
        ack: bool,
        now: Instant,
    ) -> Result<bool, DeliveryError> {
        let matching = Correlation::from_calldata(calldata) == Some(self.correlation);
        if matching && self.phase == Phase::Sending {
            return self.fail(DeliveryError::Invariant);
        }
        if self.phase != Phase::Awaiting || !matching {
            self.stale_completions = self.stale_completions.saturating_add(1);
            return Ok(false);
        }
        if ack {
            self.phase = Phase::Authorized;
        } else {
            self.retry(now)?;
        }
        Ok(true)
    }

    fn retry(&mut self, now: Instant) -> Result<(), DeliveryError> {
        let delay = if self.used >= self.max_attempts {
            if self.exhaustion == RetryExhaustion::Fail {
                return self.fail(DeliveryError::RetryExhausted);
            }
            self.max_backoff
        } else {
            self.delay
        };
        let Some(at) = now.checked_add(delay) else {
            return self.fail(DeliveryError::Exhausted);
        };
        self.delay = self
            .delay
            .checked_mul(2)
            .unwrap_or(self.max_backoff)
            .min(self.max_backoff);
        self.phase = Phase::Backoff(at);
        Ok(())
    }

    /// Observe authority state before source admission and after host-owned mutations.
    /// Never infer transaction ownership from its returned admissions: ownership was
    /// recorded when the operation started. The host must not resolve our writes.
    fn observe_authority(
        &mut self,
        owner: &IdentityOwner,
    ) -> Result<Option<PendingOwner>, DeliveryError> {
        if owner.is_stopped() {
            return self.fail(IdentityError::Stopped.into());
        }
        let ours = if matches!(self.phase, Phase::Resolving(_)) {
            Some(PendingOwner::Batch)
        } else if matches!(self.store_block, Some(StoreBlock::Growth(_))) {
            Some(PendingOwner::Growth)
        } else {
            None
        };
        if let Some(ours) = ours {
            if !owner.has_pending_write() {
                return self.fail(DeliveryError::Invariant);
            }
            return Ok(Some(ours));
        }
        if owner.has_pending_write() {
            self.store_block = Some(StoreBlock::Host);
            Ok(Some(PendingOwner::Host))
        } else {
            self.store_block = None;
            Ok(None)
        }
    }

    /// Store retries currently use the configured maximum delivery backoff. This
    /// is a bounded worker wakeup, independent of the delivery attempt budget.
    fn store_retry_at(&mut self, now: Instant) -> Result<Instant, DeliveryError> {
        match now.checked_add(self.max_backoff) {
            Some(at) => Ok(at),
            None => self.fail(DeliveryError::Exhausted),
        }
    }

    /// Append once after authorization, or resolve exactly that pending append.
    /// Applied progress is returned for scheduler evidence refresh before resumption.
    /// Only terminal outcomes return Err; recoverable failures preserve their cause.
    pub fn checkpoint(
        &mut self,
        owner: &mut IdentityOwner,
        now: Instant,
    ) -> Result<CheckpointOutcome, DeliveryError> {
        if self.is_stopped() {
            return Ok(CheckpointOutcome::NotReady);
        }
        if let Some(on @ (PendingOwner::Host | PendingOwner::Growth)) =
            self.observe_authority(owner)?
        {
            return Ok(CheckpointOutcome::Blocked(Blocked::waiting(on)));
        }
        let resolving = match self.phase {
            Phase::Authorized => false,
            Phase::Resolving(at) if now >= at => true,
            Phase::Resolving(_) => {
                return Ok(CheckpointOutcome::Blocked(Blocked::waiting(
                    PendingOwner::Batch,
                )));
            }
            _ => return Ok(CheckpointOutcome::NotReady),
        };
        if let Some(error) = self.builder.failure() {
            return self.fail(DeliveryError::Batch(Box::new(error)));
        }
        let Some(batch) = self.builder.retained() else {
            return self.fail(DeliveryError::Invariant);
        };
        let progress = batch.progress.to_vec();
        let sync_due = if self.sync_interval.is_zero() || self.sync_due.is_some() {
            self.sync_due
        } else {
            match now.checked_add(self.sync_interval) {
                Some(at) => Some(at),
                None => return self.fail(DeliveryError::Exhausted),
            }
        };
        let result = if resolving {
            owner.resolve_pending().and_then(|resolved| {
                if resolved.admissions.is_empty() {
                    Ok(resolved.receipt)
                } else {
                    Err(IdentityError::StalePlan)
                }
            })
        } else {
            owner.append_checkpoint(
                progress
                    .iter()
                    .cloned()
                    .map(cp::Operation::UpdateProgress)
                    .collect(),
                if self.sync_interval.is_zero() {
                    Durability::Sync
                } else {
                    Durability::Deferred
                },
            )
        };
        match result {
            Ok(receipt) => {
                if self.sync_interval.is_zero() && !receipt.synced {
                    return self.fail(DeliveryError::Invariant);
                }
                self.sync_due = sync_due;
                // Application succeeded exactly once. Never reappend if release fails.
                self.phase = Phase::Stopped;
                self.builder
                    .release_applied(&progress, now)
                    .map_err(|error| DeliveryError::Batch(Box::new(error)))?;
                self.phase = Phase::Idle;
                Ok(CheckpointOutcome::Applied(progress))
            }
            Err(error) if !owner.is_stopped() && owner.has_pending_write() => {
                self.phase = Phase::Resolving(self.store_retry_at(now)?);
                Ok(CheckpointOutcome::Blocked(Blocked {
                    owner: PendingOwner::Batch,
                    cause: Some(error),
                }))
            }
            Err(error) => self.fail(error.into()),
        }
    }

    /// Observe host-owned work even without a due sync, and resolve only growth
    /// initiated by this coordinator. Call after host mutations, before source work.
    /// A successful required barrier clears its deadline even if optional growth fails.
    /// After a non-blocked success, service checkpoint and next_attempt before waiting:
    /// an Ack received during a growth block may have left immediately authorized work.
    pub fn sync(
        &mut self,
        owner: &mut IdentityOwner,
        now: Instant,
    ) -> Result<SyncOutcome, DeliveryError> {
        if self.is_stopped() {
            return Ok(SyncOutcome::NotReady);
        }
        match self.observe_authority(owner)? {
            Some(on @ (PendingOwner::Host | PendingOwner::Batch)) => {
                return Ok(SyncOutcome::Blocked(Blocked::waiting(on)));
            }
            Some(PendingOwner::Growth) => {
                let Some(StoreBlock::Growth(at)) = self.store_block else {
                    unreachable!()
                };
                if now < at {
                    return Ok(SyncOutcome::Blocked(Blocked::waiting(PendingOwner::Growth)));
                }
                return match owner.resolve_pending() {
                    Ok(resolved) if resolved.admissions.is_empty() => {
                        self.store_block = None;
                        Ok(SyncOutcome::Synced)
                    }
                    Ok(_) => self.fail(DeliveryError::Invariant),
                    Err(error) if !owner.is_stopped() && owner.has_pending_write() => {
                        self.block_growth(error, now)
                    }
                    Err(error) => self.fail(error.into()),
                };
            }
            None => {}
        }
        if self.sync_due.is_none_or(|at| now < at) {
            return Ok(SyncOutcome::NotReady);
        }
        match owner.sync_checkpoint() {
            Ok(outcome) => {
                self.sync_due = None;
                match outcome.growth {
                    Ok(_) => Ok(SyncOutcome::Synced),
                    Err(error) if owner.is_stopped() => self.fail(error.into()),
                    Err(error) if owner.has_pending_write() => self.block_growth(error, now),
                    Err(error) => Ok(SyncOutcome::MaintenanceRejected(error)),
                }
            }
            Err(error) => self.fail(error.into()),
        }
    }

    fn block_growth(
        &mut self,
        error: IdentityError,
        now: Instant,
    ) -> Result<SyncOutcome, DeliveryError> {
        self.store_block = Some(StoreBlock::Growth(self.store_retry_at(now)?));
        Ok(SyncOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            cause: Some(error),
        }))
    }
}

#[cfg(test)]
pub(crate) mod tests;
