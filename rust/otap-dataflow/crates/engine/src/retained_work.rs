// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-local retained-work accounting primitives.
//!
//! This module owns accounting state only. Runtime installation, attribution,
//! telemetry export, and production charge sites are layered on separately.

use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

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
/// use otap_df_engine::retained_work::LocalRetainedAccount;
///
/// let account = LocalRetainedAccount::new();
/// std::thread::spawn(move || drop(account));
/// ```
#[derive(Debug, Default)]
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
        Rc::new(Self::default())
    }

    /// Starts one retained-work interval and returns its ownership ticket.
    ///
    /// `Some(bytes)` records a known logical size. `None` records one
    /// unknown-size item without guessing its byte size.
    pub fn charge(
        self: &Rc<Self>,
        bytes: Option<u64>,
    ) -> Result<LocalRetainedTicket, LocalAccountingError> {
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

        Ok(LocalRetainedTicket {
            account: Rc::clone(self),
            charge,
            active: true,
        })
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
#[derive(Debug)]
#[must_use = "the ticket must be completed normally or dropped as abandoned"]
pub struct LocalRetainedTicket {
    account: Rc<LocalRetainedAccount>,
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

    /// Completes this retention interval normally and refunds its charge.
    pub fn complete(mut self) -> Result<(), LocalAccountingError> {
        self.active = false;
        self.account.settle(self.charge)
    }
}

impl Drop for LocalRetainedTicket {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let _ = self.account.settle(self.charge);
        self.account.record_abandonment(self.charge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a known-size ticket reaches its normal terminal path.
    /// Guarantees: completion refunds the bytes once and records no abandonment.
    #[test]
    fn known_charge_completes_normally() {
        let account = LocalRetainedAccount::new();
        let ticket = account.charge(Some(42)).expect("charge should fit");

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
        let ticket = account.charge(None).expect("charge should fit");

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
        let ticket = account.charge(Some(17)).expect("charge should fit");

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
        let ticket = account.charge(None).expect("charge should fit");

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
        let ticket = account.charge(Some(9)).expect("charge should fit");

        ticket.complete().expect("completion should settle");

        assert_eq!(account.snapshot(), LocalRetainedSnapshot::default());
    }

    /// Scenario: a known-size charge would overflow the byte counter.
    /// Guarantees: the charge is rejected without mutation and corruption is visible.
    #[test]
    fn charge_overflow_is_rejected_and_recorded() {
        let account = LocalRetainedAccount::new();
        account.retained_bytes.set(u64::MAX);

        let error = account
            .charge(Some(1))
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
        let ticket = account.charge(Some(5)).expect("charge should fit");
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
}
