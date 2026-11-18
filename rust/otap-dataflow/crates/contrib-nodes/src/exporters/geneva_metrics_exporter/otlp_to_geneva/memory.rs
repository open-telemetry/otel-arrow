// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use super::MappingError;

pub(crate) const DEFAULT_MAX_MAPPING_BYTES: usize = 128 * 1024 * 1024;

/// Request-local string sharing and a conservative allocation budget. Charges are
/// not refunded: vector/hash-table growth and discarded intermediate values also
/// consume the budget. This is not a measurement of allocator overhead or RSS.
pub(super) struct MappingMemory {
    strings: RefCell<HashSet<Rc<str>>>,
    remaining: Cell<usize>,
    maximum: usize,
}

impl MappingMemory {
    pub(super) fn new(maximum: usize) -> Self {
        Self {
            strings: RefCell::default(),
            remaining: Cell::new(maximum),
            maximum,
        }
    }

    pub(super) fn charge(&self, bytes: usize) -> Result<(), MappingError> {
        let remaining =
            self.remaining
                .get()
                .checked_sub(bytes)
                .ok_or(MappingError::MemoryLimit {
                    maximum: self.maximum,
                })?;
        self.remaining.set(remaining);
        Ok(())
    }

    pub(super) fn charge_items<T>(&self, count: usize) -> Result<(), MappingError> {
        // Allow for spare capacity and old/new storage coexisting during growth.
        let bytes = count
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_mul(4))
            .ok_or(MappingError::MemoryLimit {
                maximum: self.maximum,
            })?;
        self.charge(bytes)
    }

    pub(super) fn intern(&self, value: &str) -> Result<Rc<str>, MappingError> {
        let mut strings = self.strings.borrow_mut();
        if let Some(value) = strings.get(value) {
            return Ok(Rc::clone(value));
        }
        // Include hash-table control bytes as well as the stored string handle.
        self.charge_items::<(Rc<str>, u8)>(1)?;
        self.charge(2 * size_of::<usize>())?;
        self.charge(value.len())?;
        let value: Rc<str> = value.into();
        let _ = strings.insert(Rc::clone(&value));
        Ok(value)
    }
}

#[cfg(test)]
impl Default for MappingMemory {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_MAPPING_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A request repeatedly references one large string, then introduces another distinct string.
    /// Guarantees: Repeated references reuse the budgeted allocation, while unique strings cannot grow the interner beyond its budget.
    #[test]
    fn shares_strings_without_allowing_unbounded_unique_storage() {
        let memory = MappingMemory::new(1024);
        let value = "v".repeat(512);
        let first = memory.intern(&value).expect("one string should fit");
        for _ in 0..256 {
            assert!(Rc::ptr_eq(
                &first,
                &memory.intern(&value).expect("shared string should fit")
            ));
        }
        assert!(matches!(
            memory.intern(&"w".repeat(512)),
            Err(MappingError::MemoryLimit { maximum: 1024 })
        ));
    }
}
