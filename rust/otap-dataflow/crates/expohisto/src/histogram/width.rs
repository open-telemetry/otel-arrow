// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Counter width for bucket data.

/// The current width of bucket counters, in bits.
///
/// Counters start at B1 and widen in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Width {
    /// 1-bit counters
    B1 = 0,
    /// 2-bit counters
    B2 = 1,
    /// 4-bit counters
    B4 = 2,
    /// 1-byte counters
    U8 = 3,
    /// 2-byte counters
    U16 = 4,
    /// 4-byte counters
    U32 = 5,
    /// 8-byte counters
    U64 = 6,
}

/// All counter widths in level order, for computed lookups.
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
pub struct SlotAddr {
    width: Width,
    word_index: i32,
    sub_offset: u32,
}

impl SlotAddr {
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

    /// Updates the counter, promised to fit.
    #[inline]
    pub(crate) const fn update_counter_in_word(&self, word: u64, count: u64) -> u64 {
        debug_assert!(count <= self.width.counter_max());

        let shift = self.shift_count();
        let sub_mask = self.width.counter_max() << shift;

        (word & !sub_mask) | (count << shift)
    }

    /// Adds `incr` to the counter, promised not to overflow its lane.
    ///
    /// Because the lane has room, the carry cannot reach the next one, so the
    /// whole word can be added to at once. That keeps the dependency from the
    /// loaded word to the stored word down to a single add, where clearing and
    /// reinserting the lane would put a shift, a mask and an or on it.
    #[inline]
    pub(crate) const fn add_counter_in_word(&self, word: u64, incr: u64) -> u64 {
        // Written as headroom so the assertion cannot itself overflow at U64,
        // where a counter spans the whole word.
        debug_assert!(
            incr <= self.width.counter_max() - self.retrieve_counter(word),
            "the caller must check the lane has room for incr"
        );

        word + (incr << self.shift_count())
    }

    /// Physical data index, offset so that `word_base` maps to slot 0.
    ///
    /// `word_base` always sits inside the active window and the window never
    /// exceeds `data_size` words, so the offset is within one turn of the ring
    /// and a single correction replaces a modulo.
    #[inline]
    pub(crate) const fn data_index(&self, data_size: usize, word_base: i32) -> usize {
        let size = data_size as i32;
        let offset = self.word_index - word_base;
        debug_assert!(
            offset > -size && offset < size,
            "word index is more than one turn from the base"
        );
        if offset < 0 {
            (offset + size) as usize
        } else {
            offset as usize
        }
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

// Note we use u32 and i32 for bucket addresses.
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
    pub(crate) const fn slot_addr(self, index: i32) -> SlotAddr {
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

    /// Lowest scale any histogram may use while its counters are this
    /// narrow.
    ///
    /// Every histogram has at least two u64 words, which at this width
    /// hold `2 * slots_per_u64()` buckets. This is the scale whose
    /// range still covers that many, i.e.
    /// `crate::mapping::min_scale_for(2 * self.slots_per_u64() as u64)`
    /// -- the range halves per scale step and bottoms out at two
    /// buckets, so the result is exactly `MIN_SCALE` plus the steps it
    /// takes to widen these counters to `Width::U64`.
    ///
    /// Equivalently: widening counters is paid for in scale steps, and
    /// a histogram must always be able to afford widening to
    /// `Width::U64`, since a counter overflow can demand it at any
    /// time.
    #[inline]
    #[must_use]
    pub const fn min_scale(self) -> i32 {
        crate::mapping::MIN_SCALE + self.to_u64_widen_steps() as i32
    }
}
