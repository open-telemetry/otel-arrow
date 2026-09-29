// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-safe receiver-local identities used by offset and replay state.

/// Identifies one Kafka partition ownership period between rebalances.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct OwnershipGeneration(u64);

impl OwnershipGeneration {
    /// Wrap a generation supplied by rebalance bookkeeping.
    #[must_use]
    pub(crate) const fn from_raw(generation: u64) -> Self {
        Self(generation)
    }

    /// Return the raw generation for APIs that still store ownership as an integer.
    #[must_use]
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

/// Identifies deliveries that share the same replay-valid feedback window.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DeliveryGeneration(u64);

impl DeliveryGeneration {
    /// First generation allocated by a retry manager.
    pub(crate) const FIRST: Self = Self(1);

    /// Decode a generation carried through pipeline call data.
    #[must_use]
    pub(crate) const fn from_raw(generation: u64) -> Self {
        Self(generation)
    }

    /// Return the raw value for call data and telemetry attributes.
    #[must_use]
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }

    /// Advance to the next representable generation without wrapping.
    #[must_use]
    pub(crate) const fn saturating_next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
