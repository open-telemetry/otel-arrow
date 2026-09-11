// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-local retained-work accounting primitives.
//!
//! This module owns accounting state and immutable bounded attribution. Runtime
//! installation, telemetry export, and production charge sites are layered on
//! separately.

use otel_arrow_dfe_config::{DeployedPipelineKey, NodeId, PipelineKey};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::rc::Rc;

/// Compact identity for one trusted retained-work owner.
///
/// Owners are allocated by [`WorkOwnerRegistry`]. The two reserved values keep
/// mixed and over-capacity ownership explicit without introducing unbounded
/// request-derived labels.
/// Registered IDs are meaningful only within the registry that assigned them;
/// independent registries may assign the same number to different pipelines.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct WorkOwnerId(u32);

impl WorkOwnerId {
    /// Work combined from more than one registered owner.
    pub const MIXED: Self = Self(0);

    /// Work whose trusted owner could not be registered within the bound.
    pub const UNREGISTERED: Self = Self(1);

    const FIRST_REGISTERED: u32 = 2;

    /// Returns the compact numeric representation.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Bounded registry of trusted pipeline owners.
///
/// Only configuration-derived pipeline identities belong here. Raw request or
/// tenant identity must never be registered because it would turn request
/// cardinality into retained-work cardinality.
#[derive(Debug)]
pub struct WorkOwnerRegistry {
    max_registered_owners: usize,
    owners_by_key: HashMap<PipelineKey, WorkOwnerId>,
    keys_by_owner: Vec<PipelineKey>,
}

impl WorkOwnerRegistry {
    /// Creates a registry that stores at most `max_registered_owners` owners.
    #[must_use]
    pub fn new(max_registered_owners: NonZeroUsize) -> Self {
        Self {
            max_registered_owners: max_registered_owners.get().min((u32::MAX - 1) as usize),
            owners_by_key: HashMap::new(),
            keys_by_owner: Vec::new(),
        }
    }

    /// Returns the stable owner for one trusted configured pipeline.
    ///
    /// Once the bound is reached, new identities deterministically map to
    /// [`WorkOwnerId::UNREGISTERED`]. Previously registered identities retain
    /// their original IDs.
    pub fn register_pipeline(&mut self, pipeline: &PipelineKey) -> WorkOwnerId {
        if let Some(owner) = self.owners_by_key.get(pipeline) {
            return *owner;
        }

        if self.keys_by_owner.len() >= self.max_registered_owners {
            return WorkOwnerId::UNREGISTERED;
        }

        let index =
            u32::try_from(self.keys_by_owner.len()).expect("owner registry bound must fit in u32");
        let owner = WorkOwnerId(index + WorkOwnerId::FIRST_REGISTERED);
        self.keys_by_owner.push(pipeline.clone());
        let previous = self.owners_by_key.insert(pipeline.clone(), owner);
        debug_assert!(previous.is_none());
        owner
    }

    /// Resolves a registered owner to its configured pipeline identity.
    #[must_use]
    pub fn pipeline(&self, owner: WorkOwnerId) -> Option<&PipelineKey> {
        let index = owner.0.checked_sub(WorkOwnerId::FIRST_REGISTERED)? as usize;
        self.keys_by_owner.get(index)
    }

    /// Returns the number of registered owners, excluding sentinels.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys_by_owner.len()
    }

    /// Returns whether the registry contains no configured owners.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys_by_owner.is_empty()
    }
}

/// Immutable attribution required for every retained-work charge.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RetainedWorkScopeId {
    /// Configured pipeline runtime, including group, core, and generation.
    pub deployed_pipeline: DeployedPipelineKey,
    /// Component retaining the work.
    pub component_id: NodeId,
    /// Trusted bounded owner of the retained work.
    pub owner: WorkOwnerId,
}

/// Static retention site carried by each ticket rather than its scope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RetainedWorkSite {
    /// Payload waiting for retry delivery.
    RetryBuffer,
    /// Payload held in a batch processor's pending batch.
    BatchPending,
}

#[derive(Debug)]
struct LocalRetainedScopeInner {
    account: Rc<LocalRetainedAccount>,
    id: RetainedWorkScopeId,
}

/// Runtime-local attributed entrypoint for retained-work charges.
///
/// The scope caches its runtime account and immutable attribution once. Charge
/// sites clone only this local `Rc`; they do not look up runtime state or copy
/// string fields per item.
#[derive(Clone, Debug)]
pub struct LocalRetainedScope {
    inner: Rc<LocalRetainedScopeInner>,
}

impl LocalRetainedScope {
    /// Binds immutable attribution to one runtime-local account.
    #[must_use]
    pub fn new(account: Rc<LocalRetainedAccount>, id: RetainedWorkScopeId) -> Self {
        Self {
            inner: Rc::new(LocalRetainedScopeInner { account, id }),
        }
    }

    /// Returns this scope's immutable attribution.
    #[must_use]
    pub fn id(&self) -> &RetainedWorkScopeId {
        &self.inner.id
    }

    /// Starts one attributed retained-work interval.
    ///
    /// `Some(bytes)` records a known logical size. `None` records one
    /// unknown-size item without guessing its byte size.
    ///
    /// Accounting failures are diagnostics, not data-path failures. A charge
    /// site must continue processing the retained work without a ticket if this
    /// returns an error; observe-only accounting must never reject or drop it.
    #[inline]
    pub fn charge(
        &self,
        site: RetainedWorkSite,
        bytes: Option<u64>,
    ) -> Result<LocalRetainedTicket, LocalAccountingError> {
        let charge = self.inner.account.start_charge(bytes)?;
        Ok(LocalRetainedTicket {
            scope: Rc::clone(&self.inner),
            site,
            charge,
            active: true,
        })
    }
}

/// Identifies the counter whose checked arithmetic failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalAccountingCounter {
    /// Logical bytes retained by known-size items.
    RetainedBytes,
    /// Number of retained items whose size is unknown.
    UnknownItems,
    /// Number of tickets dropped without explicit completion.
    AbandonedItems,
    /// Known bytes dropped without explicit completion.
    AbandonedBytes,
}

/// Describes an accounting arithmetic failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalAccountingError {
    /// Adding a charge or diagnostic would exceed the counter range.
    Overflow(LocalAccountingCounter),
    /// Settling a ticket would reduce a counter below zero.
    Underflow(LocalAccountingCounter),
}

impl fmt::Display for LocalAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow(counter) => write!(formatter, "{counter:?} counter overflow"),
            Self::Underflow(counter) => write!(formatter, "{counter:?} counter underflow"),
        }
    }
}

impl std::error::Error for LocalAccountingError {}

/// Point-in-time values from one runtime-local retained-work account.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LocalRetainedSnapshot {
    /// Logical bytes retained by known-size items.
    pub retained_bytes: u64,
    /// Number of retained items whose size is unknown.
    pub unknown_items: u64,
    /// Number of tickets dropped without explicit completion.
    pub abandoned_items: u64,
    /// Known bytes dropped without explicit completion.
    pub abandoned_bytes: u64,
    /// Number of detected accounting arithmetic failures.
    pub corruption_count: u64,
}

/// Accounting state owned by one pinned runtime.
///
/// The account is deliberately neither `Send` nor `Sync`. A later runtime
/// wiring layer is responsible for creating and exposing one on its owner
/// runtime.
///
/// ```compile_fail
/// use otel_arrow_dfe_engine::retained_work::LocalRetainedAccount;
///
/// let account = LocalRetainedAccount::new();
/// std::thread::spawn(move || drop(account));
/// ```
#[derive(Debug)]
pub struct LocalRetainedAccount {
    retained_bytes: Cell<u64>,
    unknown_items: Cell<u64>,
    abandoned_items: Cell<u64>,
    abandoned_bytes: Cell<u64>,
    corruption_count: Cell<u64>,
    _not_send: PhantomData<Rc<()>>,
}

impl LocalRetainedAccount {
    /// Creates an empty runtime-local account.
    #[must_use]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            retained_bytes: Cell::new(0),
            unknown_items: Cell::new(0),
            abandoned_items: Cell::new(0),
            abandoned_bytes: Cell::new(0),
            corruption_count: Cell::new(0),
            _not_send: PhantomData,
        })
    }

    #[inline]
    fn start_charge(
        &self,
        bytes: Option<u64>,
    ) -> Result<LocalRetainedCharge, LocalAccountingError> {
        let charge = match bytes {
            Some(bytes) => {
                self.checked_add(
                    &self.retained_bytes,
                    bytes,
                    LocalAccountingCounter::RetainedBytes,
                )?;
                LocalRetainedCharge::Known(bytes)
            }
            None => {
                self.checked_add(&self.unknown_items, 1, LocalAccountingCounter::UnknownItems)?;
                LocalRetainedCharge::Unknown
            }
        };

        Ok(charge)
    }

    /// Returns the current local counters.
    #[must_use]
    pub fn snapshot(&self) -> LocalRetainedSnapshot {
        LocalRetainedSnapshot {
            retained_bytes: self.retained_bytes.get(),
            unknown_items: self.unknown_items.get(),
            abandoned_items: self.abandoned_items.get(),
            abandoned_bytes: self.abandoned_bytes.get(),
            corruption_count: self.corruption_count.get(),
        }
    }

    fn settle(&self, charge: LocalRetainedCharge) -> Result<(), LocalAccountingError> {
        match charge {
            LocalRetainedCharge::Known(bytes) => self.checked_sub(
                &self.retained_bytes,
                bytes,
                LocalAccountingCounter::RetainedBytes,
            ),
            LocalRetainedCharge::Unknown => {
                self.checked_sub(&self.unknown_items, 1, LocalAccountingCounter::UnknownItems)
            }
        }
    }

    fn record_abandonment(&self, charge: LocalRetainedCharge) {
        let _ = self.checked_add(
            &self.abandoned_items,
            1,
            LocalAccountingCounter::AbandonedItems,
        );
        if let LocalRetainedCharge::Known(bytes) = charge {
            let _ = self.checked_add(
                &self.abandoned_bytes,
                bytes,
                LocalAccountingCounter::AbandonedBytes,
            );
        }
    }

    fn checked_add(
        &self,
        cell: &Cell<u64>,
        value: u64,
        counter: LocalAccountingCounter,
    ) -> Result<(), LocalAccountingError> {
        let Some(next) = cell.get().checked_add(value) else {
            self.record_corruption();
            return Err(LocalAccountingError::Overflow(counter));
        };
        cell.set(next);
        Ok(())
    }

    fn checked_sub(
        &self,
        cell: &Cell<u64>,
        value: u64,
        counter: LocalAccountingCounter,
    ) -> Result<(), LocalAccountingError> {
        let Some(next) = cell.get().checked_sub(value) else {
            self.record_corruption();
            return Err(LocalAccountingError::Underflow(counter));
        };
        cell.set(next);
        Ok(())
    }

    fn record_corruption(&self) {
        if let Some(next) = self.corruption_count.get().checked_add(1) {
            self.corruption_count.set(next);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LocalRetainedCharge {
    Known(u64),
    Unknown,
}

/// Owns one charge in a [`LocalRetainedAccount`].
///
/// Call [`complete`](Self::complete) on every normal terminal path. Dropping
/// an active ticket still refunds its charge, but records the interval as
/// abandoned. The ticket is deliberately neither `Send` nor `Sync` because it
/// holds an `Rc` to runtime-local state.
///
/// ```compile_fail
/// use otel_arrow_dfe_config::DeployedPipelineKey;
/// use otel_arrow_dfe_engine::retained_work::{
///     LocalRetainedAccount, LocalRetainedScope, RetainedWorkScopeId,
///     RetainedWorkSite, WorkOwnerId,
/// };
///
/// let scope = LocalRetainedScope::new(
///     LocalRetainedAccount::new(),
///     RetainedWorkScopeId {
///         deployed_pipeline: DeployedPipelineKey {
///             pipeline_group_id: "group".into(),
///             pipeline_id: "pipeline".into(),
///             core_id: 0,
///             deployment_generation: 0,
///         },
///         component_id: "processor".into(),
///         owner: WorkOwnerId::UNREGISTERED,
///     },
/// );
/// let ticket = scope
///     .charge(RetainedWorkSite::RetryBuffer, Some(1))
///     .expect("charge should fit");
/// std::thread::spawn(move || drop(ticket));
/// ```
#[derive(Debug)]
#[must_use = "the ticket must be completed normally or dropped as abandoned"]
pub struct LocalRetainedTicket {
    scope: Rc<LocalRetainedScopeInner>,
    site: RetainedWorkSite,
    charge: LocalRetainedCharge,
    active: bool,
}

impl LocalRetainedTicket {
    /// Returns the known logical byte charge, or `None` for an unknown size.
    #[must_use]
    pub const fn bytes(&self) -> Option<u64> {
        match self.charge {
            LocalRetainedCharge::Known(bytes) => Some(bytes),
            LocalRetainedCharge::Unknown => None,
        }
    }

    /// Returns the immutable attribution for this charge.
    #[must_use]
    pub fn scope(&self) -> &RetainedWorkScopeId {
        &self.scope.id
    }

    /// Returns the static site retaining this charge.
    #[must_use]
    pub const fn site(&self) -> RetainedWorkSite {
        self.site
    }

    /// Completes this retention interval normally and refunds its charge.
    #[inline]
    pub fn complete(mut self) -> Result<(), LocalAccountingError> {
        self.active = false;
        self.scope.account.settle(self.charge)
    }
}

impl Drop for LocalRetainedTicket {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let _ = self.scope.account.settle(self.charge);
        self.scope.account.record_abandonment(self.charge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_scope(account: Rc<LocalRetainedAccount>) -> LocalRetainedScope {
        LocalRetainedScope::new(
            account,
            RetainedWorkScopeId {
                deployed_pipeline: DeployedPipelineKey {
                    pipeline_group_id: "group".into(),
                    pipeline_id: "pipeline".into(),
                    core_id: 3,
                    deployment_generation: 7,
                },
                component_id: "processor".into(),
                owner: WorkOwnerId(2),
            },
        )
    }

    /// Scenario: a known-size ticket reaches its normal terminal path.
    /// Guarantees: completion refunds the bytes once and records no abandonment.
    #[test]
    fn known_charge_completes_normally() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::RetryBuffer, Some(42))
            .expect("charge should fit");

        assert_eq!(ticket.bytes(), Some(42));
        assert_eq!(account.snapshot().retained_bytes, 42);
        ticket.complete().expect("completion should settle");
        assert_eq!(account.snapshot(), LocalRetainedSnapshot::default());
    }

    /// Scenario: an unknown-size ticket reaches its normal terminal path.
    /// Guarantees: completion decrements the unknown count without guessing bytes.
    #[test]
    fn unknown_charge_completes_normally() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::RetryBuffer, None)
            .expect("charge should fit");

        assert_eq!(ticket.bytes(), None);
        assert_eq!(account.snapshot().unknown_items, 1);
        ticket.complete().expect("completion should settle");
        assert_eq!(account.snapshot(), LocalRetainedSnapshot::default());
    }

    /// Scenario: a known-size ticket is dropped without explicit completion.
    /// Guarantees: drop refunds once and records both the abandoned item and bytes.
    #[test]
    fn unresolved_known_ticket_refunds_and_records_abandonment() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::BatchPending, Some(17))
            .expect("charge should fit");

        drop(ticket);

        assert_eq!(
            account.snapshot(),
            LocalRetainedSnapshot {
                abandoned_items: 1,
                abandoned_bytes: 17,
                ..LocalRetainedSnapshot::default()
            }
        );
    }

    /// Scenario: an unknown-size ticket is dropped without explicit completion.
    /// Guarantees: drop refunds the unknown item and records no invented byte count.
    #[test]
    fn unresolved_unknown_ticket_refunds_and_records_abandonment() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::BatchPending, None)
            .expect("charge should fit");

        drop(ticket);

        assert_eq!(
            account.snapshot(),
            LocalRetainedSnapshot {
                abandoned_items: 1,
                ..LocalRetainedSnapshot::default()
            }
        );
    }

    /// Scenario: normal completion consumes a ticket and then runs its destructor.
    /// Guarantees: the destructor neither refunds twice nor records abandonment.
    #[test]
    fn completion_prevents_drop_from_settling_twice() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::RetryBuffer, Some(9))
            .expect("charge should fit");

        ticket.complete().expect("completion should settle");

        assert_eq!(account.snapshot(), LocalRetainedSnapshot::default());
    }

    /// Scenario: a known-size charge would overflow the byte counter.
    /// Guarantees: the charge is rejected without mutation and corruption is visible.
    #[test]
    fn charge_overflow_is_rejected_and_recorded() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        account.retained_bytes.set(u64::MAX);

        let error = scope
            .charge(RetainedWorkSite::RetryBuffer, Some(1))
            .expect_err("overflowing charge must fail");

        assert_eq!(
            error,
            LocalAccountingError::Overflow(LocalAccountingCounter::RetainedBytes)
        );
        assert_eq!(account.snapshot().retained_bytes, u64::MAX);
        assert_eq!(account.snapshot().corruption_count, 1);
    }

    /// Scenario: corrupted account state cannot cover a ticket's known-byte refund.
    /// Guarantees: completion reports underflow, leaves state bounded, and records corruption.
    #[test]
    fn settlement_underflow_is_reported_and_recorded() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let ticket = scope
            .charge(RetainedWorkSite::RetryBuffer, Some(5))
            .expect("charge should fit");
        account.retained_bytes.set(0);

        let error = ticket
            .complete()
            .expect_err("corrupted settlement must fail");

        assert_eq!(
            error,
            LocalAccountingError::Underflow(LocalAccountingCounter::RetainedBytes)
        );
        assert_eq!(account.snapshot().retained_bytes, 0);
        assert_eq!(account.snapshot().corruption_count, 1);
        assert_eq!(account.snapshot().abandoned_items, 0);
    }

    /// Scenario: a configured pipeline is registered more than once.
    /// Guarantees: repeated registration returns one stable compact owner ID.
    #[test]
    fn owner_registration_is_stable() {
        let mut registry = WorkOwnerRegistry::new(NonZeroUsize::new(2).expect("non-zero bound"));
        let pipeline = PipelineKey::new("group".into(), "pipeline".into());

        let first = registry.register_pipeline(&pipeline);
        let repeated = registry.register_pipeline(&pipeline);

        assert_eq!(first, repeated);
        assert_eq!(first.as_u32(), WorkOwnerId::FIRST_REGISTERED);
        assert_eq!(registry.pipeline(first), Some(&pipeline));
        assert_eq!(registry.len(), 1);
    }

    /// Scenario: equal pipeline names exist in different configured groups.
    /// Guarantees: the registry treats the full group and pipeline pair as identity.
    #[test]
    fn owner_registration_includes_pipeline_group() {
        let mut registry = WorkOwnerRegistry::new(NonZeroUsize::new(2).expect("non-zero bound"));
        let first_pipeline = PipelineKey::new("group-a".into(), "pipeline".into());
        let second_pipeline = PipelineKey::new("group-b".into(), "pipeline".into());

        let first = registry.register_pipeline(&first_pipeline);
        let second = registry.register_pipeline(&second_pipeline);

        assert_ne!(first, second);
        assert_eq!(registry.pipeline(first), Some(&first_pipeline));
        assert_eq!(registry.pipeline(second), Some(&second_pipeline));
    }

    /// Scenario: more configured owners are observed than the registry permits.
    /// Guarantees: excess identities use the deterministic Unregistered sentinel
    /// while existing owners remain stable and the registry stays bounded.
    #[test]
    fn owner_registration_is_bounded() {
        let mut registry = WorkOwnerRegistry::new(NonZeroUsize::new(1).expect("non-zero bound"));
        let first_pipeline = PipelineKey::new("group".into(), "first".into());
        let second_pipeline = PipelineKey::new("group".into(), "second".into());

        let registered = registry.register_pipeline(&first_pipeline);
        let overflow = registry.register_pipeline(&second_pipeline);

        assert_eq!(overflow, WorkOwnerId::UNREGISTERED);
        assert_eq!(registry.register_pipeline(&first_pipeline), registered);
        assert_eq!(registry.register_pipeline(&second_pipeline), overflow);
        assert_eq!(registry.pipeline(WorkOwnerId::MIXED), None);
        assert_eq!(registry.pipeline(WorkOwnerId::UNREGISTERED), None);
        assert_eq!(registry.len(), 1);
    }

    /// Scenario: owner sentinel values are exported or persisted as compact IDs.
    /// Guarantees: Mixed and Unregistered stay reserved below registered owners.
    #[test]
    fn owner_sentinel_values_are_stable() {
        let mut registry = WorkOwnerRegistry::new(NonZeroUsize::new(1).expect("non-zero bound"));
        let pipeline = PipelineKey::new("group".into(), "pipeline".into());

        assert_eq!(WorkOwnerId::MIXED.as_u32(), 0);
        assert_eq!(WorkOwnerId::UNREGISTERED.as_u32(), 1);
        assert_eq!(registry.register_pipeline(&pipeline).as_u32(), 2);
    }

    /// Scenario: a component charges retained work through an attributed scope.
    /// Guarantees: the ticket carries the complete immutable scope and its
    /// per-charge retention site without copying those fields into the account.
    #[test]
    fn ticket_carries_scope_and_retention_site() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));

        let ticket = scope
            .charge(RetainedWorkSite::BatchPending, Some(11))
            .expect("charge should fit");

        assert_eq!(ticket.scope(), scope.id());
        assert_eq!(ticket.site(), RetainedWorkSite::BatchPending);
        assert_eq!(
            ticket.scope().deployed_pipeline.pipeline_group_id.as_ref(),
            "group"
        );
        assert_eq!(
            ticket.scope().deployed_pipeline.pipeline_id.as_ref(),
            "pipeline"
        );
        assert_eq!(ticket.scope().deployed_pipeline.core_id, 3);
        assert_eq!(ticket.scope().deployed_pipeline.deployment_generation, 7);
        assert_eq!(ticket.scope().component_id.as_ref(), "processor");
        assert_eq!(ticket.scope().owner, WorkOwnerId(2));
        ticket.complete().expect("completion should settle");
    }

    /// Scenario: outstanding tickets outlive the scope handle that created them.
    /// Guarantees: attribution survives until settlement, both terminal paths
    /// refund once, and the scope is freed after the last ticket is settled.
    #[test]
    fn tickets_keep_scope_alive_until_settlement() {
        let account = LocalRetainedAccount::new();
        let scope = test_scope(Rc::clone(&account));
        let expected_id = scope.id().clone();
        let weak_scope = Rc::downgrade(&scope.inner);
        let completed = scope
            .charge(RetainedWorkSite::RetryBuffer, Some(11))
            .expect("charge should fit");
        let abandoned = scope
            .charge(RetainedWorkSite::BatchPending, None)
            .expect("charge should fit");

        drop(scope);
        assert_eq!(completed.scope(), &expected_id);
        assert_eq!(abandoned.scope(), &expected_id);
        completed.complete().expect("completion should settle");
        assert!(weak_scope.upgrade().is_some());
        drop(abandoned);

        assert!(weak_scope.upgrade().is_none());
        assert_eq!(
            account.snapshot(),
            LocalRetainedSnapshot {
                abandoned_items: 1,
                ..LocalRetainedSnapshot::default()
            }
        );
    }

    /// Scenario: different owners and deployment generations share an account.
    /// Guarantees: each ticket retains its own attribution and settling one
    /// scope's work leaves the other scope's outstanding charge intact.
    #[test]
    fn scopes_preserve_independent_attribution_and_settlement() {
        let account = LocalRetainedAccount::new();
        let first_scope = test_scope(Rc::clone(&account));
        let mut second_id = first_scope.id().clone();
        second_id.deployed_pipeline.deployment_generation += 1;
        second_id.component_id = "batch_processor".into();
        second_id.owner = WorkOwnerId::MIXED;
        let second_scope = LocalRetainedScope::new(Rc::clone(&account), second_id);
        let first = first_scope
            .charge(RetainedWorkSite::RetryBuffer, Some(11))
            .expect("charge should fit");
        let second = second_scope
            .charge(RetainedWorkSite::BatchPending, Some(23))
            .expect("charge should fit");

        assert_ne!(first.scope(), second.scope());
        assert_eq!(first.scope(), first_scope.id());
        assert_eq!(second.scope(), second_scope.id());
        assert_eq!(first.site(), RetainedWorkSite::RetryBuffer);
        assert_eq!(second.site(), RetainedWorkSite::BatchPending);
        assert_eq!(account.snapshot().retained_bytes, 34);

        first.complete().expect("completion should settle");
        assert_eq!(account.snapshot().retained_bytes, 23);
        assert_eq!(second.scope(), second_scope.id());
        second.complete().expect("completion should settle");
        assert_eq!(account.snapshot(), LocalRetainedSnapshot::default());
    }
}
