// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Acknowledged, bounded controller/provider reconciliation messages.

use super::{CoordinatorError, OperationId, ParticipantSession};
use parking_lot::{Condvar, Mutex};
use serde_json::Value;
use std::{sync::Arc, time::Instant};
use tokio::sync::oneshot;

/// Immutable coordinator settings revision, independent of runtime generations.
pub type CoordinatorRevision = u64;

/// A participant required by a prospective deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticipantSpec {
    /// Assigned core.
    pub core_id: usize,
    /// Runtime generation, which may differ across retained cores after recovery.
    pub generation: u64,
}

/// Reconciliation phase exposed without source identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationPhase {
    /// Candidate settings are being initialized alongside committed settings.
    Preparing,
    /// Settings are staged; candidates can report readiness without ownership.
    Prepared,
    /// Ownership is being released and installed.
    Activating,
    /// Required ownership changes are acknowledged.
    Activated,
    /// The controller has committed; abort is no longer allowed.
    CommitDecided,
    /// Obsolete settings and discovery workers have been retired.
    Committed,
    /// Candidate authority and settings are being retired.
    Aborting,
    /// Prior settings and eligibility have been restored with fresh grants.
    Aborted,
}

/// Definite result of an engine lifecycle command.
#[derive(Clone, Debug)]
pub enum ReconciliationOutcome {
    /// Candidate settings revision is ready.
    Prepared(CoordinatorRevision),
    /// A fresh session was registered.
    Registered(ParticipantSession),
    /// Required releases and activations completed.
    Activated,
    /// Commit finalization completed.
    Committed,
    /// Abort and restoration completed.
    Aborted,
}

pub(super) struct Completion {
    value: Mutex<Option<Result<ReconciliationOutcome, CoordinatorError>>>,
    changed: Condvar,
    asynchronous: tokio::sync::Notify,
}

pub(super) struct ReceiptCompleter(pub ReconciliationReceipt);
impl ReceiptCompleter {
    pub fn finish(&self, result: Result<ReconciliationOutcome, CoordinatorError>) {
        self.0.finish(result);
    }
    pub fn operation(&self) -> OperationId {
        self.0.operation()
    }
}
impl Drop for ReceiptCompleter {
    fn drop(&mut self) {
        self.0.finish(Err(CoordinatorError::Unavailable));
    }
}

/// Queryable acknowledgement. Losing a wakeup or timing out does not undo a command.
#[derive(Clone)]
pub struct ReconciliationReceipt {
    operation: OperationId,
    completion: Arc<Completion>,
}

impl ReconciliationReceipt {
    pub(super) fn pair(operation: OperationId) -> (Self, ReceiptCompleter) {
        let receipt = Self::new(operation);
        (receipt.clone(), ReceiptCompleter(receipt))
    }
    pub(super) fn new(operation: OperationId) -> Self {
        Self {
            operation,
            completion: Arc::new(Completion {
                value: Mutex::new(None),
                changed: Condvar::new(),
                asynchronous: tokio::sync::Notify::new(),
            }),
        }
    }
    pub(super) fn finish(&self, result: Result<ReconciliationOutcome, CoordinatorError>) {
        let mut value = self.completion.value.lock();
        if value.is_none() {
            *value = Some(result);
            let _ = self.completion.changed.notify_all();
            self.completion.asynchronous.notify_waiters();
        }
    }
    /// Idempotency/correlation key for the whole reconciliation.
    #[must_use]
    pub fn operation(&self) -> OperationId {
        self.operation
    }
    /// Resolve a lost acknowledgement without resubmitting a mutation.
    #[must_use]
    pub fn resolve(&self) -> Option<Result<ReconciliationOutcome, CoordinatorError>> {
        self.completion.value.lock().clone()
    }
    /// Await an acknowledgement without blocking. Cancellation does not undo
    /// the operation; the same receipt can be queried or awaited again.
    pub async fn wait(&self) -> Result<ReconciliationOutcome, CoordinatorError> {
        loop {
            let changed = self.completion.asynchronous.notified();
            tokio::pin!(changed);
            let _ = changed.as_mut().enable();
            if let Some(result) = self.resolve() {
                return result;
            }
            changed.await;
        }
    }
    /// Wait on a controller worker, never on a pipeline or provider runtime thread.
    /// Timeout is an unknown outcome; query this receipt or explicitly abort.
    pub fn wait_until(&self, deadline: Instant) -> Result<ReconciliationOutcome, CoordinatorError> {
        let mut value = self.completion.value.lock();
        loop {
            if let Some(result) = &*value {
                return result.clone();
            }
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(CoordinatorError::Timeout)?;
            let _ = self.completion.changed.wait_for(&mut value, remaining);
        }
    }
}

/// Provider-side receipt. Acknowledge only after staging or worker cleanup is complete.
pub struct ProviderReceipt {
    sender: oneshot::Sender<Result<(), CoordinatorError>>,
}
impl ProviderReceipt {
    pub(super) fn channel() -> (Self, oneshot::Receiver<Result<(), CoordinatorError>>) {
        let (sender, receiver) = oneshot::channel();
        (Self { sender }, receiver)
    }
    /// Send a definite outcome. Failed preparation must clean up its staged workers.
    pub fn acknowledge(self, result: Result<(), CoordinatorError>) -> Result<(), CoordinatorError> {
        self.sender
            .send(result)
            .map_err(|_| CoordinatorError::Stale)
    }
}

/// Engine-defined provider lifecycle. Custom notices cannot impersonate these events.
pub enum CoordinatorLifecycleEvent {
    /// Stage discovery/catalogs without changing behavior for committed participants.
    Prepare {
        /// Lifecycle operation.
        operation: OperationId,
        /// Candidate immutable settings revision.
        revision: CoordinatorRevision,
        /// Validated node-specific settings, bounded to 8 MiB per revision.
        settings: Arc<Value>,
        /// Acknowledge readiness after the candidate catalog has been published.
        receipt: ProviderReceipt,
    },
    /// The controller committed. Stop and join obsolete discovery workers.
    Commit {
        /// Lifecycle operation.
        operation: OperationId,
        /// Winning settings revision.
        revision: CoordinatorRevision,
        /// Acknowledge cleanup; a lost receipt never authorizes rollback.
        receipt: ProviderReceipt,
    },
    /// Stop and join candidate discovery workers, preserving committed settings.
    Abort {
        /// Lifecycle operation.
        operation: OperationId,
        /// Settings revision being abandoned.
        revision: CoordinatorRevision,
        /// Acknowledge candidate cleanup.
        receipt: ProviderReceipt,
    },
}

/// Side-effect-free settings compatibility rule supplied by the node implementer.
pub type SettingsCompatibility = fn(&Value, &Value) -> bool;

pub(super) fn settings_bytes(value: &Value) -> Result<usize, CoordinatorError> {
    let mut pending = vec![value];
    let mut bytes = 0usize;
    while let Some(value) = pending.pop() {
        bytes = bytes.saturating_add(size_of::<Value>());
        match value {
            Value::String(value) => bytes = bytes.saturating_add(value.capacity()),
            Value::Array(values) => {
                bytes = bytes.saturating_add(values.capacity().saturating_mul(size_of::<Value>()));
                if bytes > 8 * 1024 * 1024 {
                    return Err(CoordinatorError::Capacity);
                }
                pending.extend(values);
            }
            Value::Object(values) => {
                bytes = bytes.saturating_add(
                    values
                        .len()
                        .saturating_mul(size_of::<(String, Value)>() + 32),
                );
                if bytes > 8 * 1024 * 1024 {
                    return Err(CoordinatorError::Capacity);
                }
                for (key, value) in values {
                    bytes = bytes.saturating_add(key.capacity());
                    pending.push(value);
                }
            }
            _ => {}
        }
        if bytes > 8 * 1024 * 1024 {
            return Err(CoordinatorError::Capacity);
        }
    }
    Ok(bytes)
}
