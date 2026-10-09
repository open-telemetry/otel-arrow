// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Source-derived scratch model for the audited regex-automata implementation.

use super::PatternError;
use regex_automata::{
    meta,
    nfa::thompson::{self, State},
    util::primitives::StateID,
};
use std::mem::size_of;

/// Diagnostics and requested-heap bounds, not a receiver reservation or RSS limit.
#[derive(Clone, Copy, Debug)]
pub struct MemoryEstimate {
    /// Aggregate engine-reported immutable payload. Allocator bookkeeping is separate.
    pub program_payload_bytes: usize,
    /// Heap report from a newly constructed meta cache, before deferred fallback.
    pub initial_cache_reported_bytes: usize,
    /// Upper bound on caches that can grow on this half-search API path.
    pub growing_lazy_caches: usize,
    /// Capacity configured for each growing lazy cache.
    pub lazy_cache_bytes: usize,
    /// Bound for PikeVM's fixed arrays and epsilon-stack growth overlap.
    pub pikevm_heap_bound: usize,
    /// Candidate backtracker bound; zero in normal library construction.
    pub backtracker_heap_bound: usize,
    /// Peak requested heap allowance for one worker's scratch, excluding input.
    pub worker_heap_bound: usize,
}

pub(super) struct NfaScratch {
    pike: usize,
    backtrack: usize,
}

fn add(a: usize, b: usize) -> Result<usize, PatternError> {
    a.checked_add(b).ok_or(PatternError::MemoryOverflow)
}
fn mul(a: usize, b: usize) -> Result<usize, PatternError> {
    a.checked_mul(b).ok_or(PatternError::MemoryOverflow)
}

impl NfaScratch {
    pub(super) fn from_nfa(nfa: &thompson::NFA, backtrack: bool) -> Result<Self, PatternError> {
        let states = nfa.states().len();
        let slots = nfa.group_info().slot_len();
        // Both active-state sets have dense/sparse StateID arrays and a slot
        // table of states * slots + slots. They are sized once, from empty.
        let sets = mul(mul(4, states.max(4))?, size_of::<StateID>())?;
        let table_slots = add(mul(states, slots)?, slots)?.max(4);
        let tables = mul(mul(2, table_slots)?, size_of::<usize>())?;
        // Each state is visited at most once per epsilon closure. A union
        // pushes its remaining arms; each capture may push one restore frame.
        let mut pushes = 1usize;
        for state in nfa.states() {
            pushes = add(
                pushes,
                match state {
                    State::Union { alternates } => alternates.len().saturating_sub(1),
                    State::BinaryUnion { .. } | State::Capture { .. } => 1,
                    _ => 0,
                },
            )?;
        }
        // Audited private FollowEpsilon/Frame layouts fit three machine words.
        // A doubling Vec can coexist with its old allocation: 3 * max length,
        // including the minimum four-element allocation for these element sizes.
        let frame_bytes = mul(3, size_of::<usize>())?;
        let stack = mul(mul(3, pushes.max(4))?, frame_bytes)?;
        let pike = add(add(sets, tables)?, stack)?;
        let backtrack = if backtrack {
            // Qualification only: meta's earliest search skips backtracking
            // beyond 128 bytes. Account for the visited matrix AND its stack.
            let visited_cap = thompson::backtrack::Config::new().get_visited_capacity();
            let positions = (mul(visited_cap, 8)? / states.max(1)).min(129);
            let bits = mul(states, positions)?;
            let words = add(bits, usize::BITS as usize - 1)? / usize::BITS as usize;
            let visited = mul(mul(3, words.max(4))?, size_of::<usize>())?;
            let frames = mul(pushes, positions)?.max(4);
            add(visited, mul(mul(3, frames)?, frame_bytes)?)?
        } else {
            0
        };
        Ok(Self { pike, backtrack })
    }

    pub(super) fn estimate(
        self,
        payload: usize,
        initial: usize,
        growing: usize,
        cache: usize,
    ) -> Result<MemoryEstimate, PatternError> {
        // The pinned lazy cache counts some lengths rather than capacities.
        // 4 covers: <=3x vector old/new overlap, hash buckets/control bytes at
        // the audited load factor, and serialized states' unreported Arc headers.
        // It is applied per reachable growing cache, NOT to the NFA program.
        let lazy = mul(4, add(initial, mul(growing, cache)?)?)?;
        // Meta's report omits its implicit Captures buffer and wrapper storage.
        let bookkeeping = add(size_of::<meta::Cache>(), mul(4, size_of::<usize>())?)?;
        let bound = add(add(add(lazy, self.pike)?, self.backtrack)?, bookkeeping)?;
        Ok(MemoryEstimate {
            program_payload_bytes: payload,
            initial_cache_reported_bytes: initial,
            growing_lazy_caches: growing,
            lazy_cache_bytes: if growing == 0 { 0 } else { cache },
            pikevm_heap_bound: self.pike,
            backtracker_heap_bound: self.backtrack,
            worker_heap_bound: bound,
        })
    }
}
