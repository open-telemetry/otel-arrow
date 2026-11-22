// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lazily allocated dense bitset and reusable scratch buffer pool for predicate and rule masks.

use arrow::buffer::{BooleanBuffer, Buffer};

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

    /// Returns a new bitset containing `self & other`.
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

/// Reusable pool of `Vec<u64>` backing buffers for [`DenseBitset`].
///
/// Recycled across stages and batches inside `EvaluationScratch` so untouched predicates
/// allocate zero heap bytes while active predicates reuse pre-allocated buffers.
#[derive(Debug, Default)]
pub(crate) struct BitsetBufferPool {
    free_buffers: Vec<Vec<u64>>,
}

impl BitsetBufferPool {
    /// Pops a recycled buffer (or allocates a new one) zero-initialized to `len` words.
    pub(crate) fn take_zeroed(&mut self, len: usize) -> Vec<u64> {
        if let Some(mut buf) = self.free_buffers.pop() {
            buf.clear();
            buf.resize(len, 0);
            buf
        } else {
            vec![0u64; len]
        }
    }

    /// Returns all allocated backing buffers in `bitsets` to the pool for reuse.
    pub(crate) fn recycle_vec(&mut self, bitsets: &mut Vec<DenseBitset>) {
        for bs in bitsets.drain(..) {
            if bs.words.capacity() > 0 {
                self.free_buffers.push(bs.words);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn free_len(&self) -> usize {
        self.free_buffers.len()
    }
}
