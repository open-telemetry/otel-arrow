// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Counter width for bucket data.

/// The current width of bucket counters, in bits.
///
/// Counters start at 1-bit (maximizing initial bucket count) and widen
/// in place through the chain: 1→2→4→8→16→32→64 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Width {
    /// 1-bit counters (max 1 per bucket — presence bitmap).
    B1 = 0,
    /// 2-bit counters (max 3 per bucket).
    B2 = 1,
    /// 4-bit counters (max 15 per bucket).
    B4 = 2,
    /// 1-byte counters (max 255 per bucket).
    U8 = 3,
    /// 2-byte counters (max 65,535 per bucket).
    U16 = 4,
    /// 4-byte counters (max ~4 billion per bucket).
    U32 = 5,
    /// 8-byte counters.
    U64 = 6,
}

/// All counter widths in level order (excludes B0), for computed lookups.
pub(crate) const ALL_WIDTHS: [Width; 7] = [
    Width::B1,
    Width::B2,
    Width::B4,
    Width::U8,
    Width::U16,
    Width::U32,
    Width::U64,
];

/// Slot address for a specific width by word- and sub-index.
#[derive(Debug)]
pub struct SlotAddr<'a> {
    width: &'a Width,
    word_index: i32,
    sub_offset: u32,
}

impl<'a> SlotAddr<'a> {
    #[inline]
    pub(crate) const fn word_index(&self) -> i32 {
        self.word_index
    }

    /// Shift count for this slot.
    #[inline]
    const fn shift_count(&self) -> u32 {
        self.sub_offset * self.width.bits_per_slot()
    }

    /// Retrieves a counter.
    #[inline]
    pub(crate) const fn retrieve_counter(&self, word: u64) -> u64 {
        let shifted = word >> self.shift_count();
        shifted & self.width.counter_max()
    }

    /// Updtes the counter, promised to fit.
    #[inline]
    pub(crate) const fn update_counter_in_word(&self, word: u64, count: u64) -> u64 {
        debug_assert!(count <= self.width.counter_max());

        let shift = self.shift_count();
        let sub_mask = self.width.counter_max() << shift;

        (word & !sub_mask) | (count << shift)
    }

    /// Physical data index, offset so that `word_base` maps to slot 0.
    #[inline]
    pub(crate) const fn data_index(&self, data_size: usize, word_base: i32) -> usize {
        (self.word_index - word_base).rem_euclid(data_size as i32) as usize
    }

    /// Returns the next address, if valid.
    #[inline]
    pub(crate) fn next_addr(mut self, end_word_index: i32) -> Option<Self> {
        self.sub_offset += 1;
        if self.sub_offset == self.width.slots_per_u64() {
            self.word_index += 1;
            self.sub_offset = 0;
        }
        (self.word_index <= end_word_index).then_some(self)
    }
}

// Note we use u32 and i32 for bucket addresses. MAX_SCALE has been
// set to ensure i32 fits all valid values.
impl Width {
    /// Returns the log2 of bits.
    /// 0 through 6
    #[inline]
    #[must_use]
    pub(crate) const fn log2(self) -> u32 {
        self as u32
    }

    /// Number of in-word widening change steps possible.
    /// 6 through 0
    #[inline]
    #[must_use]
    pub(crate) const fn to_u64_widen_steps(self) -> u32 {
        Self::U64 as u32 - self as u32
    }

    /// Returns number of bits in one slot.
    /// 1 through 64
    #[inline]
    #[must_use]
    pub(crate) const fn bits_per_slot(self) -> u32 {
        1 << self.log2()
    }

    /// Number of slots per u64.
    /// 64 through 1
    #[inline]
    #[must_use]
    pub(crate) const fn slots_per_u64(self) -> u32 {
        1 << self.to_u64_widen_steps()
    }

    /// Maximum counter value at this width. The next value overflows.
    /// 0xFFFFFFFF through 1
    #[inline]
    #[must_use]
    pub(crate) const fn counter_max(self) -> u64 {
        // same as (1 << self.bits_per_slot()) - 1 without overflow
        u64::MAX >> (64 - self.bits_per_slot())
    }

    /// Mask for the sub-u64 index values at this width.
    /// 0x3F through 0
    #[inline]
    #[must_use]
    const fn slot_sub64_index_mask(self) -> i32 {
        self.slots_per_u64() as i32 - 1
    }

    /// Returns the (word_index, bit_shift, mask) for a slot index at this width.
    #[inline]
    #[must_use]
    pub(crate) const fn slot_addr(&self, index: i32) -> SlotAddr<'_> {
        SlotAddr {
            width: self,
            word_index: self.slot_to_word_index(index),
            sub_offset: (index & self.slot_sub64_index_mask()) as u32,
        }
    }

    /// Shifts a bucket index to its u64-word address.
    #[inline]
    #[must_use]
    pub(crate) const fn slot_to_word_index(self, index: i32) -> i32 {
        index >> self.to_u64_widen_steps()
    }

    /// Shifts a u64-word address to the first slot index.
    #[inline]
    #[must_use]
    pub(crate) const fn word_to_slot_index(self, index: i32) -> i32 {
        index << self.to_u64_widen_steps()
    }

    // /// Rounds a bucket index down to the first slot in its u64 word.
    // #[inline]
    // pub(crate) const fn slot_start_u64(self, index: i32) -> i32 {
    //     index & !self.slot_sub64_index_mask()
    // }

    // /// Rounds a bucket index up to the last slot in its u64 word.
    // #[inline]
    // pub(crate) const fn slot_end_u64(self, index: i32) -> i32 {
    //     index | self.slot_sub64_index_mask()
    // }

    /// Returns the next-wider counter width or None.
    #[inline]
    #[must_use]
    pub(crate) const fn wider_by(self, change: u32) -> Option<Width> {
        let value = self as usize + change as usize;
        if value > Self::U64 as usize {
            None
        } else {
            Some(ALL_WIDTHS[value])
        }
    }

    /// Returns width difference in steps.
    #[inline]
    #[must_use]
    pub(crate) const fn subtract(self, other: Width) -> i32 {
        self as i32 - other as i32
    }

    /// Returns the narrowest viable width.
    #[inline]
    #[must_use]
    pub(crate) const fn from_max_value(value: u64) -> Self {
        let leading = 64 - value.leading_zeros();
        let width = leading.next_power_of_two();
        ALL_WIDTHS[width.trailing_zeros() as usize]
    }
}
