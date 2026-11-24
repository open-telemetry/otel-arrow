// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lazily allocated dense bitset and reusable scratch buffer pool for predicate and rule masks.

use arrow::array::{Array, BooleanArray};
use arrow::buffer::{BooleanBuffer, Buffer, NullBuffer};

/// Lazily allocated dense bitset backed by `Vec<u64>` words.
///
/// Remains empty (0 heap bytes) until the first bit is inserted. Supports O(1) random-order
/// inserts, word-wise bitwise set operations, and fast `trailing_zeros` iteration.
#[derive(Debug, Clone, Default)]
pub(crate) struct DenseBitset {
    words: Vec<u64>,
}

impl PartialEq for DenseBitset {
    fn eq(&self, other: &Self) -> bool {
        let min_len = self.words.len().min(other.words.len());
        self.words[..min_len] == other.words[..min_len]
            && self.words[min_len..].iter().all(|&w| w == 0)
            && other.words[min_len..].iter().all(|&w| w == 0)
    }
}

impl Eq for DenseBitset {}

impl DenseBitset {
    /// Creates an empty bitset with zero heap allocation.
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self { words: Vec::new() }
    }

    /// Creates a bitset with all indices in `0..num_rows` set.
    #[must_use]
    pub(crate) fn full(num_rows: usize) -> Self {
        if num_rows == 0 {
            return Self::new();
        }
        let num_words = num_rows.div_ceil(64);
        let mut words = vec![u64::MAX; num_words];
        let rem = num_rows & 63;
        if rem != 0
            && let Some(last) = words.last_mut()
        {
            *last = (1u64 << rem) - 1;
        }
        Self { words }
    }

    /// Creates a dense bitset with all bits in `0..num_rows` set, reusing a buffer from `pool`.
    #[must_use]
    pub(crate) fn full_with_pool(num_rows: usize, pool: &mut BitsetBufferPool) -> Self {
        if num_rows == 0 {
            return Self::new();
        }
        let num_words = num_rows.div_ceil(64);
        let mut words = pool.take_filled(num_words, u64::MAX);
        let rem = num_rows & 63;
        if rem != 0
            && let Some(last) = words.last_mut()
        {
            *last = (1u64 << rem) - 1;
        }
        Self { words }
    }

    /// Creates a bitset containing all indices from `iter` (used in unit tests).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_indices(iter: impl IntoIterator<Item = u32>) -> Self {
        let mut bs = Self::new();
        for idx in iter {
            bs.insert(idx);
        }
        bs
    }

    /// Returns `true` if no bits are set.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Returns the number of set bits.
    #[inline]
    #[must_use]
    pub(crate) fn len(&self) -> u64 {
        self.words.iter().map(|w| u64::from(w.count_ones())).sum()
    }

    /// Returns `true` if `idx` is set.
    #[inline]
    #[must_use]
    pub(crate) fn contains(&self, idx: u32) -> bool {
        let word_idx = (idx as usize) >> 6;
        self.words
            .get(word_idx)
            .is_some_and(|&w| (w & (1u64 << (idx & 63))) != 0)
    }

    /// Sets bit `idx`, growing the backing vector as needed.
    #[inline]
    pub(crate) fn insert(&mut self, idx: u32) {
        let word_idx = (idx as usize) >> 6;
        if word_idx >= self.words.len() {
            self.words.resize(word_idx + 1, 0);
        }
        self.words[word_idx] |= 1u64 << (idx & 63);
    }

    /// Sets bit `idx`, drawing a pre-zeroed backing vector of at least `min_words` from `pool`
    /// on first insertion.
    #[inline]
    pub(crate) fn insert_with_pool(
        &mut self,
        idx: u32,
        min_words: usize,
        pool: &mut BitsetBufferPool,
    ) {
        let word_idx = (idx as usize) >> 6;
        if word_idx < self.words.len() {
            self.words[word_idx] |= 1u64 << (idx & 63);
            return;
        }
        self.insert_grow_with_pool(idx, min_words, pool);
    }

    #[cold]
    fn insert_grow_with_pool(&mut self, idx: u32, min_words: usize, pool: &mut BitsetBufferPool) {
        let word_idx = (idx as usize) >> 6;
        let needed = min_words.max(word_idx + 1);
        if self.words.is_empty() {
            self.words = pool.take_zeroed(needed);
        } else {
            self.words.resize(needed, 0);
        }
        self.words[word_idx] |= 1u64 << (idx & 63);
    }

    /// Returns the underlying `u64` words slice.
    #[inline]
    #[must_use]
    pub(crate) fn words(&self) -> &[u64] {
        &self.words
    }

    /// Computes `self |= other` in place, drawing a backing buffer from `pool` when `self` is empty.
    #[inline]
    pub(crate) fn or_assign_with_pool(&mut self, other: &Self, pool: &mut BitsetBufferPool) {
        if other.words.is_empty() {
            return;
        }
        if self.words.is_empty() {
            let mut words = pool.take_zeroed(other.words.len());
            words.copy_from_slice(&other.words);
            self.words = words;
            return;
        }
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (a, &b) in self.words.iter_mut().zip(&other.words) {
            *a |= b;
        }
    }

    /// Builds a [`DenseBitset`] from an Arrow [`BooleanArray`] intersected with an optional `mask`.
    ///
    /// When `nulls_match` is `false`, null entries evaluate to `0`. When `nulls_match` is `true`,
    /// null entries within `0..arr.len()` evaluate to `1` (used for OTAP `default_values_optional`
    /// columns whose null slots encode a matching default value). Remains empty (zero allocation)
    /// if no bits survive.
    #[must_use]
    pub(crate) fn from_boolean_array(
        arr: &BooleanArray,
        nulls_match: bool,
        mask: Option<&Self>,
        pool: &mut BitsetBufferPool,
    ) -> Self {
        let mut out = Self::new();
        out.or_boolean_array_with_pool(arr, nulls_match, mask, pool);
        out
    }

    /// Bitwise-ORs an Arrow [`BooleanArray`] (intersected with optional `mask`) into `self`.
    ///
    /// Lazily draws a backing buffer from `pool` only when at least one non-zero word is produced.
    pub(crate) fn or_boolean_array_with_pool(
        &mut self,
        arr: &BooleanArray,
        nulls_match: bool,
        mask: Option<&Self>,
        pool: &mut BitsetBufferPool,
    ) {
        self.or_boolean_array_impl(arr, false, nulls_match, mask, pool);
    }

    /// Bitwise-ORs valid `false` slots of an Arrow [`BooleanArray`] (intersected with optional
    /// `mask`) into `self`. Null entries evaluate to `0`.
    pub(crate) fn or_negated_boolean_array_with_pool(
        &mut self,
        arr: &BooleanArray,
        mask: Option<&Self>,
        pool: &mut BitsetBufferPool,
    ) {
        self.or_boolean_array_impl(arr, true, false, mask, pool);
    }

    fn or_boolean_array_impl(
        &mut self,
        arr: &BooleanArray,
        invert: bool,
        nulls_match: bool,
        mask: Option<&Self>,
        pool: &mut BitsetBufferPool,
    ) {
        let len = arr.len();
        if len == 0 || mask.is_some_and(Self::is_empty) {
            return;
        }
        let num_words = len.div_ceil(64);
        let max_words = match mask {
            Some(m) => num_words.min(m.words.len()),
            None => num_words,
        };
        if max_words == 0 {
            return;
        }
        let val_chunks = arr.values().bit_chunks();
        let mut val_iter = val_chunks.iter_padded();
        let null_chunks = arr.nulls().map(|n| n.inner().bit_chunks());
        let mut null_iter = null_chunks.as_ref().map(|c| c.iter_padded());
        let rem = len & 63;
        let last_word_idx = num_words - 1;
        let last_mask = if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        };

        for wi in 0..max_words {
            let v = val_iter.next().unwrap_or(0);
            let n = null_iter
                .as_mut()
                .map_or(u64::MAX, |it| it.next().unwrap_or(0));
            let m = mask.map_or(u64::MAX, |bs| bs.words[wi]);
            let tail = if wi == last_word_idx {
                last_mask
            } else {
                u64::MAX
            };
            let bits = if invert {
                !v & n
            } else if nulls_match {
                v | !n
            } else {
                v & n
            };
            let w = bits & tail & m;
            if w != 0 {
                self.or_word_with_pool(wi, w, num_words, pool);
            }
        }
    }

    /// Builds a [`DenseBitset`] of valid row indices in `0..num_rows` by word-wise intersecting
    /// `col_nulls`, `parent_nulls`, and an optional `mask`.
    #[must_use]
    pub(crate) fn from_validity_buffers(
        num_rows: usize,
        col_nulls: Option<&NullBuffer>,
        parent_nulls: Option<&NullBuffer>,
        mask: Option<&Self>,
        pool: &mut BitsetBufferPool,
    ) -> Self {
        let mut out = Self::new();
        if num_rows == 0 || mask.is_some_and(Self::is_empty) {
            return out;
        }
        let num_words = num_rows.div_ceil(64);
        let max_words = match mask {
            Some(m) => num_words.min(m.words.len()),
            None => num_words,
        };
        if max_words == 0 {
            return out;
        }
        let col_chunks = col_nulls.map(|n| n.inner().bit_chunks());
        let mut col_iter = col_chunks.as_ref().map(|c| c.iter_padded());
        let parent_chunks = parent_nulls.map(|n| n.inner().bit_chunks());
        let mut parent_iter = parent_chunks.as_ref().map(|c| c.iter_padded());
        let rem = num_rows & 63;
        let last_word_idx = num_words - 1;
        let last_mask = if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        };

        for wi in 0..max_words {
            let c = col_iter
                .as_mut()
                .map_or(u64::MAX, |it| it.next().unwrap_or(0));
            let p = parent_iter
                .as_mut()
                .map_or(u64::MAX, |it| it.next().unwrap_or(0));
            let m = mask.map_or(u64::MAX, |bs| bs.words[wi]);
            let tail = if wi == last_word_idx {
                last_mask
            } else {
                u64::MAX
            };
            let w = c & p & m & tail;
            if w != 0 {
                out.or_word_with_pool(wi, w, num_words, pool);
            }
        }
        out
    }

    /// Sets bits from `word` at `word_idx`, lazily drawing a backing vector from `pool` when empty.
    #[inline]
    pub(crate) fn or_word_with_pool(
        &mut self,
        word_idx: usize,
        word: u64,
        min_words: usize,
        pool: &mut BitsetBufferPool,
    ) {
        if word == 0 {
            return;
        }
        if word_idx >= self.words.len() {
            let needed = min_words.max(word_idx + 1);
            if self.words.is_empty() {
                self.words = pool.take_zeroed(needed);
            } else {
                self.words.resize(needed, 0);
            }
        }
        self.words[word_idx] |= word;
    }

    /// Computes `self |= other` in place.
    #[inline]
    pub(crate) fn or_assign(&mut self, other: &Self) {
        if other.words.is_empty() {
            return;
        }
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (a, &b) in self.words.iter_mut().zip(&other.words) {
            *a |= b;
        }
    }

    /// Computes `self &= other` in place.
    #[inline]
    pub(crate) fn and_assign(&mut self, other: &Self) {
        if self.words.is_empty() {
            return;
        }
        let common = self.words.len().min(other.words.len());
        if common == 0 {
            self.words.clear();
            return;
        }
        for (a, &b) in self.words[..common].iter_mut().zip(&other.words[..common]) {
            *a &= b;
        }
        self.words.truncate(common);
    }

    /// Computes `self &= !other` (`self - other`) in place.
    #[inline]
    pub(crate) fn sub_assign(&mut self, other: &Self) {
        if self.words.is_empty() || other.words.is_empty() {
            return;
        }
        for (a, &b) in self.words.iter_mut().zip(&other.words) {
            *a &= !b;
        }
    }

    /// Clones `self` using a recycled backing buffer from `pool` when non-empty.
    #[must_use]
    pub(crate) fn clone_with_pool(&self, pool: &mut BitsetBufferPool) -> Self {
        if self.is_empty() {
            return Self::new();
        }
        let mut words = pool.take_zeroed(self.words.len());
        words.copy_from_slice(&self.words);
        Self { words }
    }

    /// Returns a new bitset containing `self & other`, drawing its buffer from `pool`.
    #[must_use]
    pub(crate) fn and_with_pool(&self, other: &Self, pool: &mut BitsetBufferPool) -> Self {
        let common = self.words.len().min(other.words.len());
        if common == 0 {
            return Self::new();
        }
        let mut out = Self::new();
        for (wi, (&a, &b)) in self.words[..common]
            .iter()
            .zip(&other.words[..common])
            .enumerate()
        {
            let v = a & b;
            if v != 0 {
                out.or_word_with_pool(wi, v, common, pool);
            }
        }
        out
    }

    /// Returns a new bitset containing `self - other`, drawing its buffer from `pool`.
    #[must_use]
    pub(crate) fn sub_with_pool(&self, other: &Self, pool: &mut BitsetBufferPool) -> Self {
        if self.words.is_empty() {
            return Self::new();
        }
        let mut out = Self::new();
        for (wi, &a) in self.words.iter().enumerate() {
            let b = other.words.get(wi).copied().unwrap_or(0);
            let v = a & !b;
            if v != 0 {
                out.or_word_with_pool(wi, v, self.words.len(), pool);
            }
        }
        out
    }

    /// Returns a new bitset containing `self & other`.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn and(&self, other: &Self) -> Self {
        let common = self.words.len().min(other.words.len());
        if common == 0 {
            return Self::new();
        }
        let mut words = Vec::with_capacity(common);
        let mut any = false;
        for (&a, &b) in self.words[..common].iter().zip(&other.words[..common]) {
            let v = a & b;
            any |= v != 0;
            words.push(v);
        }
        if any { Self { words } } else { Self::new() }
    }

    /// Returns a new bitset containing `self - other`.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn sub(&self, other: &Self) -> Self {
        if self.words.is_empty() {
            return Self::new();
        }
        let mut out = self.clone();
        out.sub_assign(other);
        out
    }

    /// Returns a new bitset containing `self | other`.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn or(&self, other: &Self) -> Self {
        let mut out = self.clone();
        out.or_assign(other);
        out
    }

    /// Returns the number of bits set in both `self` and `other` without allocating.
    #[inline]
    #[must_use]
    pub(crate) fn intersection_len(&self, other: &Self) -> u64 {
        self.words
            .iter()
            .zip(&other.words)
            .map(|(&a, &b)| u64::from((a & b).count_ones()))
            .sum()
    }

    /// Iterates over all set bit indices in ascending order.
    #[inline]
    pub(crate) fn iter(&self) -> DenseBitsetIter<'_> {
        DenseBitsetIter {
            words: &self.words,
            word_idx: 0,
            current: 0,
        }
    }

    /// Converts the lowest `num_rows` bits into an Apache Arrow `BooleanBuffer`.
    #[must_use]
    pub(crate) fn to_boolean_buffer(&self, num_rows: usize) -> BooleanBuffer {
        let num_words = num_rows.div_ceil(64);
        let mut bytes = Vec::with_capacity(num_words * 8);
        for i in 0..num_words {
            let w = self.words.get(i).copied().unwrap_or(0);
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        BooleanBuffer::new(Buffer::from_vec(bytes), 0, num_rows)
    }
}

/// Ascending iterator over set bit indices in a [`DenseBitset`].
pub(crate) struct DenseBitsetIter<'a> {
    words: &'a [u64],
    word_idx: usize,
    current: u64,
}

impl Iterator for DenseBitsetIter<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        while self.current == 0 {
            self.current = *self.words.get(self.word_idx)?;
            self.word_idx += 1;
        }
        let tz = self.current.trailing_zeros();
        self.current &= self.current - 1;
        Some((((self.word_idx - 1) << 6) as u32) + tz)
    }
}

/// Maximum number of recycled `Vec<u64>` buffers retained by [`BitsetBufferPool`].
pub(crate) const MAX_POOLED_BUFFERS: usize = 256;

/// Maximum per-buffer word capacity retained by [`BitsetBufferPool`] (`1024` words = 8 KiB,
/// covering up to 65,536 rows without retaining transient burst allocations indefinitely).
pub(crate) const MAX_POOLED_BUFFER_WORDS: usize = 1024;

/// Bounded reusable pool of `Vec<u64>` backing buffers for [`DenseBitset`].
///
/// Recycled across stages and batches inside `EvaluationScratch` so untouched predicates
/// allocate zero heap bytes while active predicates reuse pre-allocated buffers.
#[derive(Debug, Default)]
pub(crate) struct BitsetBufferPool {
    free_buffers: Vec<Vec<u64>>,
}

impl BitsetBufferPool {
    /// Pops a recycled buffer (or allocates a new one) filled with `fill` to `num_words` words.
    fn take_filled(&mut self, num_words: usize, fill: u64) -> Vec<u64> {
        if let Some(mut buf) = self.free_buffers.pop() {
            buf.clear();
            buf.resize(num_words, fill);
            buf
        } else {
            vec![fill; num_words]
        }
    }

    /// Pops a recycled buffer (or allocates a new one) zero-initialized to `num_words` words.
    fn take_zeroed(&mut self, num_words: usize) -> Vec<u64> {
        self.take_filled(num_words, 0)
    }

    /// Returns a single bitset's backing buffer to the pool if within pool count and size bounds.
    #[inline]
    pub(crate) fn recycle(&mut self, bs: DenseBitset) {
        let cap = bs.words.capacity();
        if cap > 0 && cap <= MAX_POOLED_BUFFER_WORDS && self.free_buffers.len() < MAX_POOLED_BUFFERS
        {
            self.free_buffers.push(bs.words);
        }
    }

    /// Returns all eligible backing buffers in `bitsets` to the pool for reuse.
    pub(crate) fn recycle_vec(&mut self, bitsets: &mut Vec<DenseBitset>) {
        for bs in bitsets.drain(..) {
            self.recycle(bs);
        }
    }

    #[cfg(test)]
    pub(crate) fn free_len(&self) -> usize {
        self.free_buffers.len()
    }
}
