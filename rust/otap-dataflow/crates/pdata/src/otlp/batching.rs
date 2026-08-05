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

/// Move the accumulated bytes in `cur` into a new output batch, if any.
fn flush(signal: SignalType, cur: &mut Vec<u8>, batches: &mut Vec<OtlpProtoBytes>) {
    if !cur.is_empty() {
        batches.push(OtlpProtoBytes::new_from_bytes(signal, std::mem::take(cur)));
    }
}

/// Wrap `entry_bytes` (a resource-entry payload) in the top-level repeated
/// field and push it as a standalone batch.
fn emit_top(signal: SignalType, entry_bytes: &[u8], batches: &mut Vec<OtlpProtoBytes>) {
    let mut batch = Vec::with_capacity(wrapped_len(RESOURCE_ENTRY_FIELD, entry_bytes.len()));
    write_len_delimited(&mut batch, RESOURCE_ENTRY_FIELD, entry_bytes);
    batches.push(OtlpProtoBytes::new_from_bytes(signal, batch));
}

/// Greedily place an indivisible (non-resource or unparseable) top-level unit,
/// starting a new batch when it does not fit and emitting it on its own when it
/// exceeds `max_size` by itself.
fn push_opaque(
    signal: SignalType,
    full: &[u8],
    max_size: usize,
    cur: &mut Vec<u8>,
    batches: &mut Vec<OtlpProtoBytes>,
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
/// within `max_size` on its own.
fn push_resource_entry(
    signal: SignalType,
    full: &[u8],
    payload: &[u8],
    max_size: usize,
    cur: &mut Vec<u8>,
    batches: &mut Vec<OtlpProtoBytes>,
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
    split_resource_entry(signal, payload, max_size, batches);
}

/// Split one oversize resource entry into multiple valid resource entries,
/// descending into individual scopes (and, when a scope is still too large,
/// into individual records). The resource header (`resource`, `schema_url` and
/// any unknown fields) is preserved and duplicated across the produced
/// fragments.
fn split_resource_entry(
    signal: SignalType,
    entry_payload: &[u8],
    max_size: usize,
    batches: &mut Vec<OtlpProtoBytes>,
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
                    scope_fulls.push(full);
                    scope_payloads.push(&entry_payload[payload_start..field_end]);
                } else {
                    header.extend_from_slice(full);
                }
                pos = field_end;
            }
            None => {
                // A malformed field inside the resource entry: do not attempt
                // to split it. Splitting would fold the corrupt tail into the
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

/// Split one oversize scope entry into multiple resource-entry fragments, each
/// carrying `resource_header` + a scope wrapping a subset of the records. A
/// single record that is larger than `max_size` (with minimal wrappers) is
/// emitted on its own, exceeding the limit.
fn split_scope_entry(
    signal: SignalType,
    resource_header: &[u8],
    scope_payload: &[u8],
    max_size: usize,
    batches: &mut Vec<OtlpProtoBytes>,
) {
    let mut scope_header: Vec<u8> = Vec::new();
    let mut record_fulls: Vec<&[u8]> = Vec::new();
    let mut pos = 0;
    while pos < scope_payload.len() {
        match next_field(scope_payload, pos) {
            Some((field, wire, _payload_start, field_end)) => {
                let full = &scope_payload[pos..field_end];
                if field == CHILD_LIST_FIELD && wire == wire_types::LEN {
                    record_fulls.push(full);
                } else {
                    scope_header.extend_from_slice(full);
                }
                pos = field_end;
            }
            None => {
                // A malformed field inside the scope: emit the whole scope
                // unmodified as a single fragment rather than reordering the
                // corrupt tail ahead of the records. This preserves the scope
                // bytes exactly (best-effort, may exceed max_size).
                let mut entry = Vec::with_capacity(
                    resource_header.len() + wrapped_len(CHILD_LIST_FIELD, scope_payload.len()),
                );
                entry.extend_from_slice(resource_header);
                write_len_delimited(&mut entry, CHILD_LIST_FIELD, scope_payload);
                emit_top(signal, &entry, batches);
                return;
            }
        }
    }

    let emit_frag = |recs: &[u8], batches: &mut Vec<OtlpProtoBytes>| {
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

/// Combines OTLP content into size-bounded batches.
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
/// exceeding the limit.
pub fn make_bytes_batches(
    signal: SignalType,
    max_bytes: Option<NonZeroU64>,
    inputs: Vec<OtlpProtoBytes>,
) -> Result<Vec<OtlpProtoBytes>> {
    if inputs.is_empty() {
        return Err(Error::EmptyBatch);
    }
    let total_size: usize = inputs.iter().map(|i| i.num_bytes()).sum();
    if total_size == 0 {
        return Err(Error::EmptyBatch);
    }

    let max_size = match max_bytes {
        None => {
            if inputs.len() == 1 {
                return Ok(inputs);
            }
            return Ok(vec![OtlpProtoBytes::new_from_bytes(
                signal,
                inputs
                    .into_iter()
                    .fold(Vec::with_capacity(total_size), |mut acc, record| {
                        acc.extend_from_slice(record.as_bytes());
                        acc
                    }),
            )]);
        }
        Some(max_nz) => max_nz.get() as usize,
    };

    let mut batches: Vec<OtlpProtoBytes> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();

    for input in &inputs {
        let buf = input.as_bytes();
        let mut pos = 0;
        while pos < buf.len() {
            match next_field(buf, pos) {
                Some((field, wire, payload_start, field_end)) => {
                    let full = &buf[pos..field_end];
                    pos = field_end;
                    if field == RESOURCE_ENTRY_FIELD && wire == wire_types::LEN {
                        push_resource_entry(
                            signal,
                            full,
                            &buf[payload_start..field_end],
                            max_size,
                            &mut cur,
                            &mut batches,
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
    Ok(batches)
}
