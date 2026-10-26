// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Source-derived program metadata and scratch model for the audited regex engine.

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
    /// Conservative retained program allowance including metadata and reference-counted wrappers.
    pub program_heap_bound: usize,
    /// Heap report from a newly constructed meta cache, before deferred fallback.
    pub initial_cache_reported_bytes: usize,
    /// Upper bound on caches that can grow on this half-search API path.
    pub growing_lazy_caches: usize,
    /// Capacity configured for each growing lazy cache.
    pub lazy_cache_bytes: usize,
    /// Bound for PikeVM's fixed arrays and epsilon-stack growth overlap.
    pub pikevm_heap_bound: usize,
    /// Peak requested heap allowance for one worker's scratch, excluding input.
    pub worker_heap_bound: usize,
}

pub(super) struct NfaScratch {
    pike: usize,
}

fn add(a: usize, b: usize) -> Result<usize, PatternError> {
    a.checked_add(b).ok_or(PatternError::MemoryOverflow)
}
fn mul(a: usize, b: usize) -> Result<usize, PatternError> {
    a.checked_mul(b).ok_or(PatternError::MemoryOverflow)
}

impl NfaScratch {
    pub(super) fn from_nfa(nfa: &thompson::NFA) -> Result<Self, PatternError> {
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
        Ok(Self { pike })
    }

    pub(super) fn estimate(
        self,
        payload: usize,
        initial: usize,
        growing: usize,
        cache: usize,
        patterns: usize,
    ) -> Result<MemoryEstimate, PatternError> {
        // The pinned lazy cache counts some lengths rather than capacities.
        // 4 covers: <=3x vector old/new overlap, hash buckets/control bytes at
        // the audited load factor, and serialized states' unreported Arc headers.
        // It is applied per reachable growing cache, NOT to the NFA program.
        let lazy = mul(4, add(initial, mul(growing, cache)?)?)?;
        // Meta's report omits its implicit Captures buffer and wrapper storage.
        let bookkeeping = add(size_of::<meta::Cache>(), mul(4, size_of::<usize>())?)?;
        let bound = add(add(lazy, self.pike)?, bookkeeping)?;
        // RegexInfo stores one property record per pattern plus a union. Account
        // for the Vec's spare capacity as well as Core/strategy wrappers, Arc
        // headers and the boxed (unused) pool factory. Captures are implicit or
        // disabled; indexed glob queries never allocate capture tables.
        // Properties is a Box, not the property record itself. Its private
        // record has three Option<usize> fields, five LookSets, a usize and
        // three booleans; sixteen words cover those fields and padding on
        // both 32-bit and 64-bit targets. Count the box payload explicitly.
        let property_record = add(
            size_of::<regex_syntax::hir::Properties>(),
            mul(16, size_of::<usize>())?,
        )?;
        let metadata = add(
            add(
                mul(16, size_of::<meta::Config>())?,
                mul(add(16, mul(4, patterns)?)?, property_record)?,
            )?,
            mul(128, size_of::<usize>())?,
        )?;
        Ok(MemoryEstimate {
            program_payload_bytes: payload,
            program_heap_bound: add(payload, metadata)?,
            initial_cache_reported_bytes: initial,
            growing_lazy_caches: growing,
            lazy_cache_bytes: if growing == 0 { 0 } else { cache },
            pikevm_heap_bound: self.pike,
            worker_heap_bound: bound,
        })
    }
}

/// Bound byte-glob automata by expanded syntax rather than granting 10 MiB
/// to every short pattern. This is an engine program limit, not a reservation
/// for the compiler's HIR, temporary vectors, or allocator overhead.
pub(super) fn glob_program_bound(hirs: &[regex_syntax::hir::Hir]) -> Result<usize, PatternError> {
    use regex_syntax::hir::{Class, Hir, HirKind};
    fn units(hir: &Hir) -> Result<usize, PatternError> {
        let children = match hir.kind() {
            HirKind::Empty | HirKind::Look(_) => 0,
            HirKind::Literal(literal) => literal.0.len(),
            HirKind::Class(Class::Bytes(class)) => class.ranges().len(),
            HirKind::Class(Class::Unicode(_)) => {
                return Err(PatternError::Compile(
                    "glob program must use byte classes".into(),
                ));
            }
            HirKind::Capture(capture) => add(2, units(&capture.sub)?)?,
            HirKind::Concat(parts) | HirKind::Alternation(parts) => parts
                .iter()
                .try_fold(parts.len(), |sum, part| add(sum, units(part)?))?,
            HirKind::Repetition(rep) => {
                let copies = rep.max.unwrap_or(rep.min.max(1)) as usize;
                mul(copies, add(3, units(&rep.sub)?)?)?
            }
        };
        add(1, children)
    }
    // The pinned Thompson compiler uses one state per literal byte, sparse
    // transitions per byte-class range, and at most three wrapper states per
    // repetition. Alternative arms add a union edge and two wrapper states.
    // Eight final-state widths plus sixteen StateIDs per expanded unit cover
    // builder states (32 bytes on 64-bit), both forward/reverse programs,
    // union/sparse edges, and their capacity growth. The fixed per-set allowance
    // covers engine tables/wrappers; per-pattern starts/captures are units too.
    let unit_bytes = add(mul(8, size_of::<State>())?, mul(16, size_of::<StateID>())?)?;
    let expanded = hirs
        .iter()
        .try_fold(0usize, |sum, hir| add(sum, add(8, units(hir)?)?))?;
    add(16 * 1024, mul(expanded, unit_bytes)?)
}
