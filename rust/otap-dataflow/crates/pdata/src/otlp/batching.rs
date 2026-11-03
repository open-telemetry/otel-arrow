// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::OtlpProtoBytes;
use crate::batching::BatchingResult;
use crate::error::{Error, Result};
use crate::proto::consts::wire_types;
use crate::views::otlp::bytes::decode::{field_value_range, read_varint};
use otel_arrow_dfe_config::SignalType;
use std::num::NonZeroU64;
use std::ops::RangeInclusive;

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

/// An output paired with the original input indices consumed to produce it.
type OwnedBatch = (OtlpProtoBytes, RangeInclusive<usize>);

/// Encoded outputs and their input provenance, independent of duplicated headers.
pub type BytesBatches = BatchingResult<OtlpProtoBytes>;

/// Tracks origins while packing and splitting, before any output is exposed.
struct BatchWriter {
    signal: SignalType,
    input: usize,
    first: usize,
    last: usize,
    pending: Vec<u8>,
    batches: Vec<OwnedBatch>,
}

impl BatchWriter {
    fn append(&mut self, bytes: &[u8]) {
        if self.pending.is_empty() {
            self.first = self.input;
        }
        self.last = self.input;
        self.pending.extend_from_slice(bytes);
    }

    fn flush(&mut self) {
        if !self.pending.is_empty() {
            let bytes = std::mem::take(&mut self.pending);
            self.batches.push((
                OtlpProtoBytes::new_from_bytes(self.signal, bytes),
                self.first..=self.last,
            ));
        }
    }

    fn emit_top(&mut self, entry_bytes: &[u8]) {
        let mut batch = Vec::with_capacity(wrapped_len(RESOURCE_ENTRY_FIELD, entry_bytes.len()));
        write_len_delimited(&mut batch, RESOURCE_ENTRY_FIELD, entry_bytes);
        self.batches.push((
            OtlpProtoBytes::new_from_bytes(self.signal, batch),
            self.input..=self.input,
        ));
    }
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
///
/// The field-boundary computation is delegated to the shared
/// [`field_value_range`] decoder so the wire-format logic lives in one place;
/// this wrapper only re-derives the historical `payload_start` convention
/// (value start for LEN, `field_end` otherwise -- callers only read
/// `payload_start` under a `wire == LEN` guard).
fn next_field(buf: &[u8], pos: usize) -> Option<(u64, u64, usize, usize)> {
    let (tag, after_tag) = read_varint(buf, pos)?;
    let field = tag >> 3;
    let wire = tag & wire_types::PROTOBUF_TAG_BITMASK;
    let (value_start, field_end) = field_value_range(buf, wire, after_tag)?;
    let payload_start = if wire == wire_types::LEN {
        value_start
    } else {
        field_end
    };
    Some((field, wire, payload_start, field_end))
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

/// Greedily place an indivisible (non-resource or unparseable) top-level unit,
/// starting a new batch when it does not fit and emitting it on its own when it
/// exceeds `max_size` by itself.
fn push_opaque(writer: &mut BatchWriter, full: &[u8], max_size: usize) {
    if writer.pending.len() + full.len() <= max_size {
        writer.append(full);
        return;
    }
    writer.flush();
    writer.append(full);
    if writer.pending.len() > max_size {
        // An indivisible unit larger than max_size is emitted on its own.
        writer.flush();
    }
}

/// Place a top-level resource entry, descending into it only when it cannot fit
/// within `max_size` on its own. Every split fragment retains the current input index.
#[allow(clippy::too_many_arguments)]
fn push_resource_entry(
    writer: &mut BatchWriter,
    full: &[u8],
    payload: &[u8],
    max_size: usize,
    fragment_budget: usize,
    overhead_budget: usize,
    flush_fragment_budget: usize,
    budget_fallbacks: &mut u64,
) {
    if writer.pending.len() + full.len() <= max_size {
        writer.append(full);
        return;
    }
    if full.len() <= max_size {
        writer.flush();
        writer.append(full);
        return;
    }
    // A single resource entry exceeds max_size: split within it.
    writer.flush();
    if writer.batches.len() >= flush_fragment_budget {
        // Per-flush output ceiling already reached: emit this entry whole rather
        // than fan it out into more in-memory fragments. Bounds the flush's
        // total output allocation regardless of downstream slot accounting.
        writer.emit_top(payload);
        *budget_fallbacks += 1;
    } else {
        split_resource_entry(
            writer,
            payload,
            max_size,
            fragment_budget,
            overhead_budget,
            budget_fallbacks,
        );
    }
}

/// Split one oversize resource entry into multiple valid resource entries,
/// descending into individual scopes (and, when a scope is still too large,
/// into individual records). The resource header (`resource`, `schema_url` and
/// any unknown fields) is preserved and duplicated across the produced
/// fragments.
///
/// Two budgets guard the split, and because nothing is sent until the caller
/// finishes building the output vec, an over-budget entry is always collapsed
/// back to a single whole batch (best-effort, may exceed `max_size`) with
/// `budget_fallbacks` incremented -- never leaving some records already emitted:
///   1. `fragment_budget`: a cheap up-front ceiling on the worst-case fragment
///      count (one per record, or one per empty scope), bounding the work done
///      before a possible fallback. The real count is always smaller.
///   2. `overhead_budget`: the duplicated wrapper bytes, measured from the
///      *actual* greedy packing (total emitted fragment bytes minus a single
///      whole encoding) rather than a per-record worst case, so many small
///      records under a large header are not falsely collapsed when they pack
///      into only a few fragments.
fn split_resource_entry(
    writer: &mut BatchWriter,
    entry_payload: &[u8],
    max_size: usize,
    fragment_budget: usize,
    overhead_budget: usize,
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
                        writer.emit_top(entry_payload);
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
                writer.emit_top(entry_payload);
                return;
            }
        }
    }

    if scope_fulls.is_empty() {
        // Nothing to split (e.g. a resource entry with no scopes): emit as-is.
        writer.emit_top(entry_payload);
        return;
    }

    // Fragment-count ceiling: a cheap up-front guard, checked worst-case (one
    // fragment per record, or one for an empty scope). This bounds how much work
    // the greedy pack below can do before we would fall back, so falling back
    // here -- before any fragment is emitted -- never duplicates records already
    // placed in earlier fragments. The real fragment count can only be smaller
    // (records pack together), so it always stays within `fragment_budget`.
    let projected_fragments: usize = scope_payloads
        .iter()
        .map(|sp| count_records(sp).max(1))
        .sum();
    if projected_fragments > fragment_budget {
        writer.emit_top(entry_payload);
        *budget_fallbacks += 1;
        return;
    }

    // Greedily pack whole scopes into resource-entry fragments, tracking the
    // running duplicated-wrapper overhead so we can both (a) fall back to a
    // single whole batch when the *real* amplification exceeds `overhead_budget`
    // and (b) abort early -- bounding transient memory to about the budget rather
    // than letting up to `fragment_budget` header-duplicating fragments
    // materialize first. Overhead is measured from the actual packing (total
    // emitted fragment bytes minus one whole encoding), not a per-record worst
    // case, so many small records under a large header are not falsely collapsed.
    // Nothing is sent until the caller finishes the vec, so truncation is safe.
    let start = writer.batches.len();
    let whole_bytes = wrapped_len(RESOURCE_ENTRY_FIELD, entry_payload.len());
    let abort_at = whole_bytes.saturating_add(overhead_budget);
    let mut emitted: usize = 0;
    let mut over_budget = false;
    let mut frag: Vec<u8> = header.clone();
    for (i, scope_full) in scope_fulls.iter().enumerate() {
        let prospective = frag.len() + scope_full.len();
        if wrapped_len(RESOURCE_ENTRY_FIELD, prospective) <= max_size {
            frag.extend_from_slice(scope_full);
            continue;
        }
        // Flush what we have so far (if it carries at least one scope).
        if frag.len() > header.len() {
            writer.emit_top(&frag);
            emitted = emitted.saturating_add(
                writer
                    .batches
                    .last()
                    .map_or(0, |(payload, _)| payload.num_bytes()),
            );
            frag.truncate(header.len());
            if emitted > abort_at {
                over_budget = true;
                break;
            }
        }
        // Try the scope on its own.
        if wrapped_len(RESOURCE_ENTRY_FIELD, header.len() + scope_full.len()) <= max_size {
            frag.extend_from_slice(scope_full);
        } else {
            // The scope alone is still too large: split it by records.
            split_scope_entry(
                writer,
                &header,
                scope_payloads[i],
                max_size,
                abort_at,
                &mut emitted,
            );
            if emitted > abort_at {
                over_budget = true;
                break;
            }
        }
    }
    if !over_budget && frag.len() > header.len() {
        writer.emit_top(&frag);
        emitted = emitted.saturating_add(
            writer
                .batches
                .last()
                .map_or(0, |(payload, _)| payload.num_bytes()),
        );
        over_budget = emitted > abort_at;
    }

    if over_budget {
        writer.batches.truncate(start);
        writer.emit_top(entry_payload);
        *budget_fallbacks += 1;
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

/// Split one oversize scope entry into multiple resource-entry fragments, each
/// carrying `resource_header` + a scope wrapping a subset of the records. A
/// single record that is larger than `max_size` (with minimal wrappers) is
/// emitted on its own, exceeding the limit.
///
/// The caller (`split_resource_entry`) only descends here after verifying the
/// whole entry -- including this scope's payload -- is `fully_parseable`, so the
/// field scan always terminates cleanly at the end of the payload.
///
/// `emitted` accumulates the encoded length of every fragment produced (here and
/// by the caller); emission stops early once it exceeds `abort_at`, so a large
/// scope header re-encoded across many record fragments cannot amplify transient
/// memory far past the overhead budget before the caller rolls the entry back to
/// a single whole batch.
fn split_scope_entry(
    writer: &mut BatchWriter,
    resource_header: &[u8],
    scope_payload: &[u8],
    max_size: usize,
    abort_at: usize,
    emitted: &mut usize,
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

    let emit_frag = |recs: &[u8], writer: &mut BatchWriter| {
        let mut scope_inner = Vec::with_capacity(scope_header.len() + recs.len());
        scope_inner.extend_from_slice(&scope_header);
        scope_inner.extend_from_slice(recs);
        let mut entry = Vec::with_capacity(
            resource_header.len() + wrapped_len(CHILD_LIST_FIELD, scope_inner.len()),
        );
        entry.extend_from_slice(resource_header);
        write_len_delimited(&mut entry, CHILD_LIST_FIELD, &scope_inner);
        writer.emit_top(&entry);
    };

    if record_fulls.is_empty() {
        // Preserve an empty scope rather than dropping it.
        emit_frag(&[], writer);
        *emitted = emitted.saturating_add(
            writer
                .batches
                .last()
                .map_or(0, |(payload, _)| payload.num_bytes()),
        );
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
            emit_frag(&recs, writer);
            *emitted = emitted.saturating_add(
                writer
                    .batches
                    .last()
                    .map_or(0, |(payload, _)| payload.num_bytes()),
            );
            recs.clear();
            // Abort early so a large scope header re-encoded across many record
            // fragments cannot amplify transient memory past the budget; the
            // caller rolls the whole entry back to a single batch.
            if *emitted > abort_at {
                return;
            }
        }
        recs.extend_from_slice(rec);
        let alone_inner = scope_header.len() + rec.len();
        let alone_entry = resource_header.len() + wrapped_len(CHILD_LIST_FIELD, alone_inner);
        if wrapped_len(RESOURCE_ENTRY_FIELD, alone_entry) > max_size {
            // Indivisible record larger than max_size: emit it on its own.
            emit_frag(&recs, writer);
            *emitted = emitted.saturating_add(
                writer
                    .batches
                    .last()
                    .map_or(0, |(payload, _)| payload.num_bytes()),
            );
            recs.clear();
            if *emitted > abort_at {
                return;
            }
        }
    }
    if !recs.is_empty() {
        emit_frag(&recs, writer);
        *emitted = emitted.saturating_add(
            writer
                .batches
                .last()
                .map_or(0, |(payload, _)| payload.num_bytes()),
        );
    }
}

/// Combines OTLP content into size-bounded batches. Thin wrapper over
/// [`make_bytes_batches_owned`] that discards provenance and applies no
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
        make_bytes_batches_owned(signal, max_bytes, None, None, None, inputs)?
            .batches
            .into_iter()
            .map(|(batch, _inputs)| batch)
            .collect(),
    )
}

/// Like [`make_bytes_batches`] but also reports inclusive input ranges and
/// bounds split amplification. Provenance is captured while packing, so header
/// duplication never changes which inputs own a fragment.
///
/// `fragment_budget` caps how many fragments a single oversize resource entry
/// may split into -- a cheap worst-case ceiling projected up front (one fragment
/// per record) before any fragment is built. `overhead_budget` caps the
/// duplicated wrapper bytes (resource/scope headers re-encoded across
/// fragments), guarding against output-byte amplification from large headers
/// even when the fragment count stays low; unlike the fragment ceiling it is
/// *measured from the actual greedy packing* (emitted fragment bytes minus one
/// whole encoding) as the entry is split, aborting early once exceeded. An entry
/// that would exceed either budget is emitted whole (best-effort) and counted in
/// `budget_fallbacks`, so no records are ever partially emitted before falling
/// back.
///
/// `flush_fragment_budget` bounds the *cumulative* number of output batches this
/// whole call may build before splitting stops: once that many batches have been
/// produced, any further oversize resource entry is emitted whole instead of
/// split. This caps the in-memory output allocation for a flush containing many
/// large entries, independently of any downstream Ack/Nack slot accounting. It
/// is a greedy running check -- no reservation or look-ahead over later entries.
///
/// `None` means unbounded for any of the three budgets.
pub fn make_bytes_batches_owned(
    signal: SignalType,
    max_bytes: Option<NonZeroU64>,
    fragment_budget: Option<NonZeroU64>,
    overhead_budget: Option<NonZeroU64>,
    flush_fragment_budget: Option<NonZeroU64>,
    inputs: Vec<OtlpProtoBytes>,
) -> Result<BytesBatches> {
    if inputs.is_empty() {
        return Err(Error::EmptyBatch);
    }
    let total_size: usize = inputs.iter().map(|i| i.num_bytes()).sum();
    if total_size == 0 {
        return Err(Error::EmptyBatch);
    }

    let input_count = inputs.len();
    // The fast path consumes the complete input range without parsing.
    let single = |inputs: Vec<OtlpProtoBytes>| -> BytesBatches {
        if inputs.len() == 1 {
            let batch = inputs.into_iter().next().expect("one input");
            return BytesBatches {
                batches: vec![(batch, 0..=0)],
                completed_inputs: Vec::new(),
                budget_fallbacks: 0,
            };
        }
        let bytes = inputs
            .iter()
            .fold(Vec::with_capacity(total_size), |mut acc, record| {
                acc.extend_from_slice(record.as_bytes());
                acc
            });
        BytesBatches {
            batches: vec![(
                OtlpProtoBytes::new_from_bytes(signal, bytes),
                0..=input_count - 1,
            )],
            completed_inputs: Vec::new(),
            budget_fallbacks: 0,
        }
    };

    let max_size = match max_bytes {
        None => return Ok(single(inputs)),
        // A limit beyond the addressable size cannot constrain an in-memory batch.
        // Saturate at the platform bound rather than truncating it on 32-bit targets.
        Some(max_nz) => usize::try_from(max_nz.get()).unwrap_or(usize::MAX),
    };

    // Fast path: a single input that already fits is returned unchanged,
    // avoiding all parsing and copying.
    if inputs.len() == 1 && total_size <= max_size {
        return Ok(single(inputs));
    }

    // Budgets are upper bounds too: values above usize::MAX are effectively
    // unbounded for addressable data, not small budgets after truncation.
    let fragment_budget = fragment_budget
        .map(|n| usize::try_from(n.get()).unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);
    let overhead_budget = overhead_budget
        .map(|n| usize::try_from(n.get()).unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);
    let flush_fragment_budget = flush_fragment_budget
        .map(|n| usize::try_from(n.get()).unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);

    let mut writer = BatchWriter {
        signal,
        input: 0,
        first: 0,
        last: 0,
        pending: Vec::with_capacity(total_size.min(max_size)),
        batches: Vec::new(),
    };
    let mut budget_fallbacks: u64 = 0;

    for (index, input) in inputs.iter().enumerate() {
        writer.input = index;
        let buf = input.as_bytes();
        let mut pos = 0;
        while pos < buf.len() {
            match next_field(buf, pos) {
                Some((field, wire, payload_start, field_end)) => {
                    let full = &buf[pos..field_end];
                    pos = field_end;
                    if field == RESOURCE_ENTRY_FIELD && wire == wire_types::LEN {
                        push_resource_entry(
                            &mut writer,
                            full,
                            &buf[payload_start..field_end],
                            max_size,
                            fragment_budget,
                            overhead_budget,
                            flush_fragment_budget,
                            &mut budget_fallbacks,
                        );
                    } else {
                        push_opaque(&mut writer, full, max_size);
                    }
                }
                None => {
                    // Malformed field: treat the rest of the buffer as opaque.
                    let full = &buf[pos..];
                    pos = buf.len();
                    push_opaque(&mut writer, full, max_size);
                }
            }
        }
    }

    writer.flush();
    let mut completed_inputs = Vec::new();
    let mut covered = 0;
    for (_, inputs) in &writer.batches {
        completed_inputs.extend(covered..*inputs.start());
        covered = inputs.end() + 1;
    }
    completed_inputs.extend(covered..input_count);
    Ok(BytesBatches {
        batches: writer.batches,
        completed_inputs,
        budget_fallbacks,
    })
}
