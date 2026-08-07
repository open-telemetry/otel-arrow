// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::OtlpProtoBytes;
use crate::error::{Error, Result};
use crate::proto::consts::wire_types;
use crate::views::otlp::bytes::decode::{read_len_delim, read_varint};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

// OTLP export requests share a uniform nesting across all three signals:
//
//   ExportXServiceRequest { repeated ResourceX resource_x = 1 }
//   ResourceX { Resource resource = 1; repeated ScopeX scope_x = 2; string schema_url = 3 }
//   ScopeX    { InstrumentationScope scope = 1; repeated Record records = 2; string schema_url = 3 }
//
// so the field numbers used when splitting are identical for logs, traces and
// metrics: the top-level repeated resource entry is field 1, and both the
// repeated scope list (within a resource entry) and the repeated record list
// (within a scope entry) are field 2.
const RESOURCE_ENTRY_FIELD: u64 = 1;
const CHILD_LIST_FIELD: u64 = 2;

/// An output batch paired with its ownership weight: the number of *input* bytes
/// it represents. Unlike the batch's own encoded length, ownership weights sum
/// to exactly the input byte total even when wrapper headers are duplicated
/// across split fragments. The batch processor apportions Ack/Nack ownership by
/// this weight so every fragment of a split input is attributed back to that
/// input (see `make_bytes_batches_owned`).
type OwnedBatch = (OtlpProtoBytes, usize);

/// Result of byte batching: the output batches (each with its ownership weight)
/// plus the counts of oversize entries that were emitted whole instead of split.
pub struct BytesBatches {
    /// Output batches, each paired with the input-byte ownership weight it
    /// represents. The weights sum to the input byte total.
    pub batches: Vec<OwnedBatch>,
    /// Number of resource entries emitted whole (best-effort) instead of split
    /// because their projected fragment count or duplicated wrapper bytes
    /// exceeded the per-entry `fragment_budget` / `overhead_budget`.
    pub budget_fallbacks: u64,
    /// Number of resource entries emitted whole (best-effort) instead of split
    /// because splitting them would have produced more output batches than the
    /// flush's remaining outbound capacity (`output_budget`) could track.
    pub capacity_fallbacks: u64,
}

/// Number of bytes needed to encode `v` as a protobuf varint.
fn varint_len(mut v: u64) -> usize {
    let mut n = 1;
    while v >= 0x80 {
        v >>= 7;
        n += 1;
    }
    n
}

/// Append `v` to `buf` as a protobuf varint.
fn write_varint(buf: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        buf.push(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Append a LEN-delimited field (`tag` + length + `payload`) to `buf`.
fn write_len_delimited(buf: &mut Vec<u8>, field: u64, payload: &[u8]) {
    write_varint(buf, (field << 3) | wire_types::LEN);
    write_varint(buf, payload.len() as u64);
    buf.extend_from_slice(payload);
}

/// On-wire size of a LEN-delimited field carrying `payload_len` bytes.
fn wrapped_len(field: u64, payload_len: usize) -> usize {
    varint_len((field << 3) | wire_types::LEN) + varint_len(payload_len as u64) + payload_len
}

/// Parse one protobuf field starting at `pos`.
///
/// Returns `(field_number, wire_type, payload_start, field_end)`. For LEN
/// fields, `[payload_start, field_end)` is the value bytes; for other wire
/// types `payload_start == field_end`. Returns `None` when the field cannot be
/// parsed (truncated or invalid), which signals the caller to treat the rest of
/// the buffer as an opaque unit (matching the previous skip-to-EOF behavior).
fn next_field(buf: &[u8], pos: usize) -> Option<(u64, u64, usize, usize)> {
    let (tag, after_tag) = read_varint(buf, pos)?;
    let field = tag >> 3;
    let wire = tag & wire_types::PROTOBUF_TAG_BITMASK;
    match wire {
        wire_types::LEN => {
            let (payload, end) = read_len_delim(buf, after_tag)?;
            Some((field, wire, end - payload.len(), end))
        }
        wire_types::VARINT => {
            let (_, end) = read_varint(buf, after_tag)?;
            Some((field, wire, end, end))
        }
        wire_types::FIXED64 => {
            let end = after_tag.checked_add(8)?;
            if end > buf.len() {
                return None;
            }
            Some((field, wire, end, end))
        }
        wire_types::FIXED32 => {
            let end = after_tag.checked_add(4)?;
            if end > buf.len() {
                return None;
            }
            Some((field, wire, end, end))
        }
        _ => None,
    }
}

/// Counts top-level repeated resource entries (`RESOURCE_ENTRY_FIELD`, LEN
/// wire type) in `buf`, skipping payloads. A malformed tail stops the count.
/// Used to reserve one outbound slot per not-yet-processed entry when bounding
/// a split against the flush's remaining outbound capacity.
fn count_resource_entries(buf: &[u8]) -> usize {
    let mut pos = 0;
    let mut count = 0;
    while pos < buf.len() {
        match next_field(buf, pos) {
            Some((field, wire, _, field_end)) => {
                if field == RESOURCE_ENTRY_FIELD && wire == wire_types::LEN {
                    count += 1;
                }
                pos = field_end;
            }
            None => break,
        }
    }
    count
}

/// Returns `true` when every field in `buf` parses cleanly through to the end
/// (no truncated or invalid field). Used to decide whether a resource entry can
/// be safely split: if any part is malformed, the entry is emitted whole rather
/// than reconstructed (which could reorder or duplicate bytes).
fn fully_parseable(buf: &[u8]) -> bool {
    let mut pos = 0;
    while pos < buf.len() {
        match next_field(buf, pos) {
            Some((_, _, _, field_end)) => pos = field_end,
            None => return false,
        }
    }
    true
}

/// Move the accumulated bytes in `cur` into a new output batch, if any. A
/// packed/opaque batch carries no duplicated headers, so its ownership weight
/// equals its own encoded length.
fn flush(signal: SignalType, cur: &mut Vec<u8>, batches: &mut Vec<OwnedBatch>) {
    if !cur.is_empty() {
        let bytes = std::mem::take(cur);
        let weight = bytes.len();
        batches.push((OtlpProtoBytes::new_from_bytes(signal, bytes), weight));
    }
}

/// Wrap `entry_bytes` (a resource-entry payload) in the top-level repeated
/// field and push it as a standalone batch. Its ownership weight is its own
/// encoded length; callers that produce header-duplicating fragments overwrite
/// these weights afterwards via `rescale_ownership`.
fn emit_top(signal: SignalType, entry_bytes: &[u8], batches: &mut Vec<OwnedBatch>) {
    let mut batch = Vec::with_capacity(wrapped_len(RESOURCE_ENTRY_FIELD, entry_bytes.len()));
    write_len_delimited(&mut batch, RESOURCE_ENTRY_FIELD, entry_bytes);
    let weight = batch.len();
    batches.push((OtlpProtoBytes::new_from_bytes(signal, batch), weight));
}

/// Redistribute `total` input-byte ownership evenly across the batches in
/// `frags`, so their weights sum to exactly `total` (the split entry's input
/// size) with each fragment receiving at least one unit. Used after a split
/// duplicates wrapper headers, which would otherwise make the fragments' own
/// encoded lengths sum to more than the input they represent.
fn rescale_ownership(frags: &mut [OwnedBatch], total: usize) {
    let n = frags.len();
    if n == 0 {
        return;
    }
    // A genuine split always carries at least one record per fragment, so
    // `total >= n` and `base >= 1`; the debug assert guards the invariant.
    debug_assert!(total >= n, "each fragment must own at least one input byte");
    let base = total / n;
    let rem = total % n;
    for (i, frag) in frags.iter_mut().enumerate() {
        frag.1 = base + usize::from(i < rem);
    }
}

/// Greedily place an indivisible (non-resource or unparseable) top-level unit,
/// starting a new batch when it does not fit and emitting it on its own when it
/// exceeds `max_size` by itself.
fn push_opaque(
    signal: SignalType,
    full: &[u8],
    max_size: usize,
    cur: &mut Vec<u8>,
    batches: &mut Vec<OwnedBatch>,
) {
    if cur.len() + full.len() <= max_size {
        cur.extend_from_slice(full);
        return;
    }
    flush(signal, cur, batches);
    cur.extend_from_slice(full);
    if cur.len() > max_size {
        // An indivisible unit larger than max_size is emitted on its own.
        flush(signal, cur, batches);
    }
}

/// Place a top-level resource entry, descending into it only when it cannot fit
/// within `max_size` on its own. When a split occurs, the produced fragments'
/// ownership weights are rescaled to sum to the entry's input size (`full`).
#[allow(clippy::too_many_arguments)]
fn push_resource_entry(
    signal: SignalType,
    full: &[u8],
    payload: &[u8],
    max_size: usize,
    fragment_budget: usize,
    overhead_budget: usize,
    output_budget: usize,
    entries_after: usize,
    cur: &mut Vec<u8>,
    batches: &mut Vec<OwnedBatch>,
    budget_fallbacks: &mut u64,
    capacity_fallbacks: &mut u64,
) {
    if cur.len() + full.len() <= max_size {
        cur.extend_from_slice(full);
        return;
    }
    if full.len() <= max_size {
        flush(signal, cur, batches);
        cur.extend_from_slice(full);
        return;
    }
    // A single resource entry exceeds max_size: split within it.
    flush(signal, cur, batches);
    let start = batches.len();
    split_resource_entry(
        signal,
        payload,
        max_size,
        fragment_budget,
        overhead_budget,
        batches,
        budget_fallbacks,
    );
    // Bound cumulative fan-out across the whole flush. `batches.len()` is the
    // running total of committed outputs (the `cur` accumulator was just
    // flushed, so nothing is pending). Reserve one outbound slot for every
    // resource entry still to be processed (`entries_after`): even if each of
    // them falls back to a single whole batch, they need a slot, so this entry
    // may only keep its split if the running total plus those reservations still
    // fits `output_budget` -- the flush's remaining outbound capacity. Without
    // the reservation a greedy early entry could consume the whole budget and
    // force later entries to be Nacked. Collapsing back to a single whole entry
    // keeps the input's Ack/Nack coherent: a real split would create more
    // subscribed fragments than the outbound pool can track, so a later fragment
    // would be Nacked while its siblings are already in flight. Nothing has been
    // sent yet (the caller builds the entire output vec first), so truncation is
    // safe.
    if batches.len() - start > 1 && batches.len().saturating_add(entries_after) > output_budget {
        batches.truncate(start);
        emit_top(signal, payload, batches);
        *capacity_fallbacks += 1;
    }
    // Every fragment produced by this split re-encodes the resource/scope
    // headers, so their encoded lengths sum to more than the input entry.
    // Reattribute the entry's input bytes across the fragments (or the single
    // whole batch above) so Ack/Nack ownership sums back to the input.
    rescale_ownership(&mut batches[start..], full.len());
}

/// Split one oversize resource entry into multiple valid resource entries,
/// descending into individual scopes (and, when a scope is still too large,
/// into individual records). The resource header (`resource`, `schema_url` and
/// any unknown fields) is preserved and duplicated across the produced
/// fragments.
///
/// Before emitting anything, an upper bound on the fragment count is projected
/// (worst case: one fragment per record, plus one per empty scope) and an upper
/// bound on the duplicated wrapper bytes (resource header re-encoded per
/// fragment plus each scope header re-encoded per record). If either exceeds its
/// budget (`fragment_budget` / `overhead_budget`), the entry is emitted whole
/// (best-effort, may exceed `max_size`) and `budget_fallbacks` is incremented,
/// so the caller can always fall back without having already emitted -- and thus
/// duplicated -- some of the entry's records.
fn split_resource_entry(
    signal: SignalType,
    entry_payload: &[u8],
    max_size: usize,
    fragment_budget: usize,
    overhead_budget: usize,
    batches: &mut Vec<OwnedBatch>,
    budget_fallbacks: &mut u64,
) {
    let mut header: Vec<u8> = Vec::new();
    let mut scope_fulls: Vec<&[u8]> = Vec::new();
    let mut scope_payloads: Vec<&[u8]> = Vec::new();
    let mut pos = 0;
    while pos < entry_payload.len() {
        match next_field(entry_payload, pos) {
            Some((field, wire, payload_start, field_end)) => {
                let full = &entry_payload[pos..field_end];
                if field == CHILD_LIST_FIELD && wire == wire_types::LEN {
                    let scope_payload = &entry_payload[payload_start..field_end];
                    if !fully_parseable(scope_payload) {
                        // A malformed field inside a scope: emitting a rebuilt
                        // entry would reorder trailing non-scope fields (e.g.
                        // schema_url) and, with multiple scopes, split the entry
                        // across fragments. Emit the entry byte-for-byte instead
                        // (best-effort, may exceed max_size).
                        emit_top(signal, entry_payload, batches);
                        return;
                    }
                    scope_fulls.push(full);
                    scope_payloads.push(scope_payload);
                } else {
                    header.extend_from_slice(full);
                }
                pos = field_end;
            }
            None => {
                // A malformed field at the resource level: do not attempt to
                // split it. Splitting would fold the corrupt tail into the
                // duplicated header, reordering it ahead of every fragment's
                // scopes and possibly breaking the decode of otherwise-valid
                // fragments. Emit the entry byte-for-byte as a single batch
                // instead (best-effort, may exceed max_size).
                emit_top(signal, entry_payload, batches);
                return;
            }
        }
    }

    if scope_fulls.is_empty() {
        // Nothing to split (e.g. a resource entry with no scopes): emit as-is.
        emit_top(signal, entry_payload, batches);
        return;
    }

    // Bound fan-out before emitting anything. Two projections, both worst-case
    // (one fragment per record), so falling back here -- before any fragment is
    // emitted -- never duplicates records already placed in earlier fragments:
    //
    //   1. fragment count: one per record (or one for an empty scope);
    //   2. duplicated wrapper bytes: the resource header is re-encoded once per
    //      fragment, and each scope header is re-encoded once per record in that
    //      scope. A large header split across many records amplifies output far
    //      beyond the input even when the fragment count stays low.
    let per_scope_records: Vec<usize> = scope_payloads
        .iter()
        .map(|sp| count_records(sp).max(1))
        .collect();
    let projected_fragments: usize = per_scope_records.iter().sum();
    let projected_overhead: usize = projected_fragments
        .saturating_mul(header.len())
        .saturating_add(
            scope_payloads
                .iter()
                .zip(&per_scope_records)
                .map(|(sp, records)| scope_header_len(sp).saturating_mul(*records))
                .fold(0usize, |acc, n| acc.saturating_add(n)),
        );
    if projected_fragments > fragment_budget || projected_overhead > overhead_budget {
        emit_top(signal, entry_payload, batches);
        *budget_fallbacks += 1;
        return;
    }

    // Greedily pack whole scopes into a resource-entry fragment.
    let mut frag: Vec<u8> = header.clone();
    for (i, scope_full) in scope_fulls.iter().enumerate() {
        let prospective = frag.len() + scope_full.len();
        if wrapped_len(RESOURCE_ENTRY_FIELD, prospective) <= max_size {
            frag.extend_from_slice(scope_full);
            continue;
        }
        // Flush what we have so far (if it carries at least one scope).
        if frag.len() > header.len() {
            emit_top(signal, &frag, batches);
            frag.truncate(header.len());
        }
        // Try the scope on its own.
        if wrapped_len(RESOURCE_ENTRY_FIELD, header.len() + scope_full.len()) <= max_size {
            frag.extend_from_slice(scope_full);
        } else {
            // The scope alone is still too large: split it by records.
            split_scope_entry(signal, &header, scope_payloads[i], max_size, batches);
        }
    }
    if frag.len() > header.len() {
        emit_top(signal, &frag, batches);
    }
}

/// Count the record entries (repeated field 2) in a fully-parseable scope
/// payload. Used to project an upper bound on fragment count before splitting.
fn count_records(scope_payload: &[u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while let Some((field, wire, _payload_start, field_end)) = next_field(scope_payload, pos) {
        if field == CHILD_LIST_FIELD && wire == wire_types::LEN {
            count += 1;
        }
        pos = field_end;
    }
    count
}

/// Total encoded length of a scope's non-record (header) fields -- the
/// `InstrumentationScope`, `schema_url`, and any unknown fields, i.e. everything
/// except the repeated records (field 2). These bytes are re-encoded into every
/// record-fragment when a scope is split, so they drive the byte-amplification
/// projection in [`split_resource_entry`].
fn scope_header_len(scope_payload: &[u8]) -> usize {
    let mut len = 0;
    let mut pos = 0;
    while let Some((field, wire, _payload_start, field_end)) = next_field(scope_payload, pos) {
        if !(field == CHILD_LIST_FIELD && wire == wire_types::LEN) {
            len += field_end - pos;
        }
        pos = field_end;
    }
    len
}

/// Split one oversize scope entry into multiple resource-entry fragments, each
/// carrying `resource_header` + a scope wrapping a subset of the records. A
/// single record that is larger than `max_size` (with minimal wrappers) is
/// emitted on its own, exceeding the limit.
///
/// The caller (`split_resource_entry`) only descends here after verifying the
/// whole entry -- including this scope's payload -- is `fully_parseable`, so the
/// field scan always terminates cleanly at the end of the payload.
fn split_scope_entry(
    signal: SignalType,
    resource_header: &[u8],
    scope_payload: &[u8],
    max_size: usize,
    batches: &mut Vec<OwnedBatch>,
) {
    let mut scope_header: Vec<u8> = Vec::new();
    let mut record_fulls: Vec<&[u8]> = Vec::new();
    let mut pos = 0;
    // The caller only descends here for a fully-parseable scope, so the loop
    // always terminates via the end-of-buffer `None` after consuming every
    // field (no malformed-field handling is needed here).
    while let Some((field, wire, _payload_start, field_end)) = next_field(scope_payload, pos) {
        let full = &scope_payload[pos..field_end];
        if field == CHILD_LIST_FIELD && wire == wire_types::LEN {
            record_fulls.push(full);
        } else {
            scope_header.extend_from_slice(full);
        }
        pos = field_end;
    }

    let emit_frag = |recs: &[u8], batches: &mut Vec<OwnedBatch>| {
        let mut scope_inner = Vec::with_capacity(scope_header.len() + recs.len());
        scope_inner.extend_from_slice(&scope_header);
        scope_inner.extend_from_slice(recs);
        let mut entry = Vec::with_capacity(
            resource_header.len() + wrapped_len(CHILD_LIST_FIELD, scope_inner.len()),
        );
        entry.extend_from_slice(resource_header);
        write_len_delimited(&mut entry, CHILD_LIST_FIELD, &scope_inner);
        emit_top(signal, &entry, batches);
    };

    if record_fulls.is_empty() {
        // Preserve an empty scope rather than dropping it.
        emit_frag(&[], batches);
        return;
    }

    let mut recs: Vec<u8> = Vec::new();
    for rec in &record_fulls {
        let scope_inner_len = scope_header.len() + recs.len() + rec.len();
        let entry_len = resource_header.len() + wrapped_len(CHILD_LIST_FIELD, scope_inner_len);
        if wrapped_len(RESOURCE_ENTRY_FIELD, entry_len) <= max_size {
            recs.extend_from_slice(rec);
            continue;
        }
        if !recs.is_empty() {
            emit_frag(&recs, batches);
            recs.clear();
        }
        recs.extend_from_slice(rec);
        let alone_inner = scope_header.len() + rec.len();
        let alone_entry = resource_header.len() + wrapped_len(CHILD_LIST_FIELD, alone_inner);
        if wrapped_len(RESOURCE_ENTRY_FIELD, alone_entry) > max_size {
            // Indivisible record larger than max_size: emit it on its own.
            emit_frag(&recs, batches);
            recs.clear();
        }
    }
    if !recs.is_empty() {
        emit_frag(&recs, batches);
    }
}

/// Combines OTLP content into size-bounded batches. Thin wrapper over
/// [`make_bytes_batches_owned`] that discards ownership weights and applies no
/// fragment budget; kept for callers that only need the output payloads.
///
/// With no limit, inputs are concatenated into a single batch (correct because
/// the top-level field is repeated). With a byte limit, whole resource entries
/// are packed greedily by concatenation (the cheap, byte-exact fast path); when
/// a single resource entry exceeds `max_bytes`, it is split within the entry --
/// descending to scopes and, when needed, to individual records -- re-encoding
/// the resource and scope wrapper headers so every output batch is a valid
/// `ExportXServiceRequest`. Records are never dropped, duplicated or reordered;
/// unknown wrapper fields are preserved. Any indivisible unit whose minimal
/// encoding still exceeds `max_bytes` -- a lone record, an opaque/unparseable
/// field, or a wrapper-only (header/empty-scope) fragment -- is emitted alone,
/// exceeding the limit. If a resource entry that must be split is malformed
/// (any field truncated or invalid, at the resource level or within one of its
/// scopes), it is emitted whole and unchanged rather than reconstructed, so its
/// original bytes and field ordering are preserved.
///
/// Limitation: the smallest metric unit is a whole `Metric` (all of its data
/// points), not an individual data point, so a single metric with many points
/// may still exceed `max_bytes`. Data point-level splitting is a follow-up.
pub fn make_bytes_batches(
    signal: SignalType,
    max_bytes: Option<NonZeroU64>,
    inputs: Vec<OtlpProtoBytes>,
) -> Result<Vec<OtlpProtoBytes>> {
    Ok(
        make_bytes_batches_owned(signal, max_bytes, None, None, usize::MAX, inputs)?
            .batches
            .into_iter()
            .map(|(batch, _weight)| batch)
            .collect(),
    )
}

/// Like [`make_bytes_batches`] but also returns, per output, the input-byte
/// ownership weight it represents (summing to the input total), and bounds
/// fan-out with `fragment_budget` and byte amplification with `overhead_budget`.
///
/// The batch processor uses the ownership weights -- not each batch's own
/// encoded length -- to apportion Ack/Nack subscribers, so every fragment of a
/// split input is attributed back to that input even though header duplication
/// makes the fragments' encoded lengths sum to more than the input.
///
/// `fragment_budget` caps how many fragments a single oversize resource entry
/// may split into. `overhead_budget` caps the projected duplicated wrapper
/// bytes (resource/scope headers re-encoded across fragments), guarding against
/// output-byte amplification from large headers even when the fragment count
/// stays low. Both projections are checked up front; an entry that would exceed
/// either budget is emitted whole (best-effort) and counted in
/// `budget_fallbacks`, so no records are ever partially emitted before falling
/// back. `None` means unbounded.
///
/// `output_budget` caps the *cumulative* number of output batches across the
/// whole call, tying split fan-out to the caller's remaining outbound capacity.
/// When deciding whether to keep an entry's split, one slot is reserved for
/// every resource entry still to be processed (each may need at least a whole
/// batch), so an early greedy split cannot consume the whole budget and starve
/// later entries. If keeping the split would push the running total (plus those
/// reservations) past this bound, the entry is instead emitted whole and counted
/// in `capacity_fallbacks`, so a single input never fans out into more subscribed
/// fragments than the caller can track (which would otherwise Nack a late
/// fragment while its siblings are in flight). Pass `usize::MAX` for unbounded.
pub fn make_bytes_batches_owned(
    signal: SignalType,
    max_bytes: Option<NonZeroU64>,
    fragment_budget: Option<NonZeroU64>,
    overhead_budget: Option<NonZeroU64>,
    output_budget: usize,
    inputs: Vec<OtlpProtoBytes>,
) -> Result<BytesBatches> {
    if inputs.is_empty() {
        return Err(Error::EmptyBatch);
    }
    let total_size: usize = inputs.iter().map(|i| i.num_bytes()).sum();
    if total_size == 0 {
        return Err(Error::EmptyBatch);
    }

    // Emit a single input (or the whole concatenation) as one batch whose
    // ownership weight equals its encoded length (no headers are duplicated).
    let single = |inputs: Vec<OtlpProtoBytes>| -> BytesBatches {
        if inputs.len() == 1 {
            let batch = inputs.into_iter().next().expect("one input");
            let weight = batch.num_bytes();
            return BytesBatches {
                batches: vec![(batch, weight)],
                budget_fallbacks: 0,
                capacity_fallbacks: 0,
            };
        }
        let bytes = inputs
            .iter()
            .fold(Vec::with_capacity(total_size), |mut acc, record| {
                acc.extend_from_slice(record.as_bytes());
                acc
            });
        let weight = bytes.len();
        BytesBatches {
            batches: vec![(OtlpProtoBytes::new_from_bytes(signal, bytes), weight)],
            budget_fallbacks: 0,
            capacity_fallbacks: 0,
        }
    };

    let max_size = match max_bytes {
        None => return Ok(single(inputs)),
        Some(max_nz) => max_nz.get() as usize,
    };

    // Fast path: a single input that already fits is returned unchanged,
    // avoiding all parsing and copying.
    if inputs.len() == 1 && total_size <= max_size {
        return Ok(single(inputs));
    }

    let fragment_budget = fragment_budget
        .map(|n| n.get() as usize)
        .unwrap_or(usize::MAX);
    let overhead_budget = overhead_budget
        .map(|n| n.get() as usize)
        .unwrap_or(usize::MAX);

    let mut batches: Vec<OwnedBatch> = Vec::new();
    // Reserve for the common top-level packing path; a batch never exceeds
    // `max_size` unless a single indivisible unit forces it.
    let mut cur: Vec<u8> = Vec::with_capacity(total_size.min(max_size));
    let mut budget_fallbacks: u64 = 0;
    let mut capacity_fallbacks: u64 = 0;

    // Total number of top-level resource entries across all inputs. Used to
    // reserve one outbound slot per not-yet-processed entry when bounding a
    // split against `output_budget`, so an early greedy split cannot starve
    // later entries. Only meaningful when `output_budget` is finite; the count
    // is cheap (field tags only, payloads skipped).
    let total_entries: usize = if output_budget == usize::MAX {
        0
    } else {
        inputs
            .iter()
            .map(|input| count_resource_entries(input.as_bytes()))
            .sum()
    };
    let mut entries_seen: usize = 0;

    for input in &inputs {
        let buf = input.as_bytes();
        let mut pos = 0;
        while pos < buf.len() {
            match next_field(buf, pos) {
                Some((field, wire, payload_start, field_end)) => {
                    let full = &buf[pos..field_end];
                    pos = field_end;
                    if field == RESOURCE_ENTRY_FIELD && wire == wire_types::LEN {
                        entries_seen += 1;
                        let entries_after = total_entries.saturating_sub(entries_seen);
                        push_resource_entry(
                            signal,
                            full,
                            &buf[payload_start..field_end],
                            max_size,
                            fragment_budget,
                            overhead_budget,
                            output_budget,
                            entries_after,
                            &mut cur,
                            &mut batches,
                            &mut budget_fallbacks,
                            &mut capacity_fallbacks,
                        );
                    } else {
                        push_opaque(signal, full, max_size, &mut cur, &mut batches);
                    }
                }
                None => {
                    // Malformed field: treat the rest of the buffer as opaque.
                    let full = &buf[pos..];
                    pos = buf.len();
                    push_opaque(signal, full, max_size, &mut cur, &mut batches);
                }
            }
        }
    }

    flush(signal, &mut cur, &mut batches);
    Ok(BytesBatches {
        batches,
        budget_fallbacks,
        capacity_fallbacks,
    })
}
