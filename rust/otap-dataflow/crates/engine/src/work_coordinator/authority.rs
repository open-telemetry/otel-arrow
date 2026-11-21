// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{CoordinatorError, ParticipantSession, WorkId};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

/// Unique authority command within a host incarnation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationId(pub u64);

/// Exact authority identity. It cannot be used to manufacture an executable grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantIdentity {
    /// Random host incarnation, never reused across host restarts.
    pub incarnation: uuid::Uuid,
    /// Work being assigned.
    pub work_id: WorkId,
    /// Exact participant registration.
    pub session: ParticipantSession,
    /// Settings revision; Basic uses revision zero throughout its lifetime.
    pub revision: u64,
    /// Monotonic term within the incarnation, never reused after retirement.
    pub term: u64,
}

/// A definite or unresolved backend command result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendOutcome {
    /// The requested change is confirmed.
    Confirmed,
    /// The command definitively did not apply.
    Rejected,
    /// Cancellation/timeout does not establish whether the command applied.
    Unknown,
}

/// Assignment authority extension point, independent of allocation policy.
///
/// Commands are serialized by the host. An unknown outcome must be resolved
/// before another owner can activate. Basic fails closed on unresolved outcomes.
/// Durable progress will require a writer in this same authority domain; this
/// trait alone does not promise durable source progress or fenced takeover.
#[async_trait(?Send)]
pub trait AssignmentBackend: 'static {
    /// Activate exactly this grant, after an offer has been accepted.
    async fn activate(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError>;
    /// Retire exactly this grant, after the participant proves release.
    async fn retire(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError>;
    /// Resolve the most recent command after an ambiguous outcome.
    async fn resolve(&mut self, operation: OperationId)
    -> Result<BackendOutcome, CoordinatorError>;
}

/// Nonpersistent authority for Basic consumers, with bounded active grants.
/// Only the most recent command receipt is retained; older receipts are unknown.
pub struct InMemoryBackend {
    active: HashMap<WorkId, GrantIdentity>,
    last: Option<(OperationId, bool, GrantIdentity, BackendOutcome)>,
    capacity: usize,
}

impl InMemoryBackend {
    /// Create an empty authority, bounded by the catalog item limit.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            active: HashMap::new(),
            last: None,
            capacity,
        }
    }

    fn replay(
        &self,
        operation: OperationId,
        activating: bool,
        grant: &GrantIdentity,
    ) -> Option<BackendOutcome> {
        self.last
            .as_ref()
            .and_then(|(op, kind, identity, outcome)| {
                if operation.0 < op.0 {
                    Some(BackendOutcome::Rejected)
                } else if operation == *op {
                    Some(if activating == *kind && grant == identity {
                        *outcome
                    } else {
                        BackendOutcome::Rejected
                    })
                } else {
                    None
                }
            })
    }

    fn finish(
        &mut self,
        operation: OperationId,
        activating: bool,
        grant: &GrantIdentity,
        outcome: BackendOutcome,
    ) -> BackendOutcome {
        self.last = Some((operation, activating, grant.clone(), outcome));
        outcome
    }
}

#[async_trait(?Send)]
impl AssignmentBackend for InMemoryBackend {
    async fn activate(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError> {
        if let Some(outcome) = self.replay(operation, true, grant) {
            return Ok(outcome);
        }
        let outcome = match self.active.get(&grant.work_id) {
            Some(current) if current == grant => BackendOutcome::Confirmed,
            Some(_) => BackendOutcome::Rejected,
            None if self.active.len() < self.capacity => {
                let _ = self.active.insert(grant.work_id.clone(), grant.clone());
                BackendOutcome::Confirmed
            }
            None => return Err(CoordinatorError::Capacity),
        };
        Ok(self.finish(operation, true, grant, outcome))
    }

    async fn retire(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError> {
        if let Some(outcome) = self.replay(operation, false, grant) {
            return Ok(outcome);
        }
        let outcome = match self.active.get(&grant.work_id) {
            Some(current) if current == grant => {
                let _ = self.active.remove(&grant.work_id);
                BackendOutcome::Confirmed
            }
            _ => BackendOutcome::Rejected,
        };
        Ok(self.finish(operation, false, grant, outcome))
    }

    async fn resolve(
        &mut self,
        operation: OperationId,
    ) -> Result<BackendOutcome, CoordinatorError> {
        Ok(self
            .last
            .as_ref()
            .filter(|(op, _, _, _)| *op == operation)
            .map_or(BackendOutcome::Unknown, |(_, _, _, outcome)| *outcome))
    }
}

#[derive(Debug, Default)]
pub(super) struct AdmissionState(AtomicU8);

impl AdmissionState {
    const DRAINING: u8 = 1;
    const INVALID: u8 = 2;

    pub(super) fn begin_shutdown(&self) {
        let _ = self.0.fetch_or(Self::DRAINING, Ordering::AcqRel);
    }
    pub(super) fn is_stopping(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::DRAINING != 0
    }
    fn check_admission(&self) -> Result<(), CoordinatorError> {
        match self.0.load(Ordering::Acquire) {
            0 => Ok(()),
            Self::DRAINING => Err(CoordinatorError::Stale),
            _ => Err(CoordinatorError::Unavailable),
        }
    }
    pub(super) fn is_valid(&self) -> bool {
        self.0.load(Ordering::Acquire) & Self::INVALID == 0
    }
    /// Return true only for the first unexpected loss, linearized with shutdown.
    pub(super) fn invalidate(&self) -> bool {
        self.0.fetch_or(Self::INVALID, Ordering::AcqRel) == 0
    }
}

#[derive(Debug, Default)]
pub(super) struct Lease {
    pub accepting: bool,
    pub tickets: usize,
}

/// Activated permission to execute one responsibility, checked without a host RPC.
#[derive(Clone, Debug)]
pub struct AssignmentGrant {
    pub(super) identity: Arc<GrantIdentity>,
    pub(super) admission: Arc<AdmissionState>,
    pub(super) lease: Arc<Mutex<Lease>>,
}

impl AssignmentGrant {
    /// Exact identity to use in activation/release acknowledgements.
    #[must_use]
    pub fn identity(&self) -> &GrantIdentity {
        &self.identity
    }

    /// Atomically validate authority and reserve an operation, such as one scrape.
    /// Revocation stops new admissions while existing tickets can settle.
    pub fn try_admit(&self) -> Result<AdmissionTicket, CoordinatorError> {
        let mut lease = self.lease.lock();
        self.admission.check_admission()?;
        if !lease.accepting {
            return Err(CoordinatorError::Stale);
        }
        lease.tickets = lease
            .tickets
            .checked_add(1)
            .ok_or(CoordinatorError::Capacity)?;
        Ok(AdmissionTicket {
            grant: self.clone(),
        })
    }
}

/// Reservation for an admitted operation and its publication/completion path.
///
/// Keep this alive through completion, including downstream Ack/Nack when relevant.
/// Dropping it releases the reservation; it is neither an Ack nor proof that a
/// detached worker has stopped. The participant must join workers before release.
#[derive(Debug)]
pub struct AdmissionTicket {
    grant: AssignmentGrant,
}

impl AdmissionTicket {
    /// Whether the authority domain is still alive. Cooperative revocation allows
    /// already admitted operations to settle; host failure invalidates all tickets.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.grant.admission.is_valid()
    }

    /// Grant that admitted this operation.
    #[must_use]
    pub fn grant(&self) -> &GrantIdentity {
        &self.grant.identity
    }
}

impl Drop for AdmissionTicket {
    fn drop(&mut self) {
        self.grant.lease.lock().tickets -= 1;
    }
}
