// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pure decoding of a journald entry's raw `name=value` byte slices into owned
//! [`JournalField`]s.
//!
//! This logic used to live inline in the `sd-journal` FFI reader
//! (`journal.rs::imp::current_entry`), where it could not be unit-tested (it
//! required a live journal) and could not be benchmarked. Extracting it into a
//! pure function over an iterator of borrowed byte slices lets the FFI layer
//! stay a thin adapter (enumerate slices -> feed them here) while the
//! size-limit, drop, and `MESSAGE`-body selection semantics are exercised by
//! ordinary tests and a criterion benchmark.
//!
//! The function borrows each field's bytes and copies exactly once (into the
//! owned [`JournalField::value`]); it never clones the `MESSAGE` payload a
//! second time for the log body. Instead it records the index of the first
//! kept `MESSAGE` field so the body can read it back from `fields`.

use super::arrow_records_encoder::JournalField;
use super::config::{ExtractionConfig, LargeFieldPolicy};

/// Extra bytes added to `max_field_bytes` when deriving the
/// `sd_journal_set_data_threshold` value, so a field whose *value* is exactly at
/// the limit is not misclassified as truncated purely because of its name and
/// `=` separator overhead.
const FIELD_NAME_THRESHOLD_HEADROOM_BYTES: u64 = 4096;

/// The data-threshold, in `u64`, used both to program
/// `sd_journal_set_data_threshold` and to detect possibly-truncated fields
/// during decode. Kept as a pure function so the FFI `size_t` caller and the
/// decoder agree on one definition.
pub(crate) fn extraction_data_threshold_u64(extraction: &ExtractionConfig) -> u64 {
    extraction
        .max_field_bytes
        .saturating_add(FIELD_NAME_THRESHOLD_HEADROOM_BYTES)
        .min(extraction.max_entry_bytes)
        .max(1)
}

/// The owned fields decoded from one journald entry, plus the metadata the
/// encoder needs to project them.
pub(crate) struct DecodedFields {
    /// Kept fields, in journal order.
    pub(crate) fields: Vec<JournalField>,
    /// Index into `fields` of the first kept `MESSAGE` field, or `None` if no
    /// `MESSAGE` was kept. Note this is `None` when the *first* `MESSAGE` was
    /// dropped for size even if a later `MESSAGE` survived -- matching the
    /// encoder's body contract.
    pub(crate) message_body_index: Option<usize>,
    /// Count of fields dropped for exceeding the extraction limits (or for a
    /// non-UTF-8 field name).
    pub(crate) dropped_fields: u64,
}

/// Incremental decoder for one journald entry's fields.
///
/// Fed one raw `name=value` slice at a time via [`FieldDecoder::feed`], it lets
/// the FFI reader copy each `sd_journal_enumerate_data` slice out *before*
/// pulling the next one -- those slices are only valid until the next enumerate
/// call -- so no journald-owned pointer is ever held across a call.
/// [`FieldDecoder::finish`] yields the owned result.
///
/// For each field, [`FieldDecoder::feed`] applies, in order:
/// 1. `MESSAGE` detection -- the first field named `MESSAGE` is marked the body
///    candidate (regardless of whether it is then dropped).
/// 2. Drop check -- dropped and counted if possibly truncated
///    (`field_len >= threshold`), its value exceeds `max_field_bytes`, the kept
///    field count already reached `max_fields_per_entry`, or keeping it would
///    push the entry past `max_entry_bytes`.
/// 3. UTF-8 name check -- a field whose name is not valid UTF-8 is dropped and
///    counted.
///
/// The body index is set only when the first `MESSAGE` is actually *kept*; a
/// dropped first `MESSAGE` leaves the body unset even if a later `MESSAGE` is
/// kept.
pub(crate) struct FieldDecoder<'e> {
    extraction: &'e ExtractionConfig,
    threshold: u64,
    fields: Vec<JournalField>,
    copied_entry_bytes: u64,
    copied_field_count: usize,
    dropped_fields: u64,
    message_seen: bool,
    message_body_index: Option<usize>,
}

impl<'e> FieldDecoder<'e> {
    pub(crate) fn new(extraction: &'e ExtractionConfig) -> Self {
        Self {
            extraction,
            threshold: extraction_data_threshold_u64(extraction),
            fields: Vec::with_capacity(extraction.max_fields_per_entry.min(64)),
            copied_entry_bytes: 0,
            copied_field_count: 0,
            dropped_fields: 0,
            message_seen: false,
            message_body_index: None,
        }
    }

    /// Process one raw `name=value` field. `bytes` only needs to be valid for
    /// the duration of the call -- kept fields are copied out before returning.
    pub(crate) fn feed(&mut self, bytes: &[u8]) {
        let Some(eq) = bytes.iter().position(|b| *b == b'=') else {
            return;
        };
        let is_first_message = if !self.message_seen && &bytes[..eq] == b"MESSAGE" {
            // Setting `message_seen` here -- before the drop check -- intentionally
            // poisons the body index for the whole entry if this first MESSAGE is
            // then dropped (e.g. oversized): a later, smaller MESSAGE chunk must
            // not masquerade as the (truncated) body.
            self.message_seen = true;
            true
        } else {
            false
        };
        let value_len = bytes.len().saturating_sub(eq + 1) as u64;
        let field_len = bytes.len() as u64;
        let possibly_truncated = field_len >= self.threshold;
        let would_exceed_entry =
            self.copied_entry_bytes.saturating_add(field_len) > self.extraction.max_entry_bytes;
        let should_drop = possibly_truncated
            || value_len > self.extraction.max_field_bytes
            || self.copied_field_count >= self.extraction.max_fields_per_entry
            || would_exceed_entry;
        if should_drop {
            match self.extraction.large_field_policy {
                LargeFieldPolicy::DropAndCount => {
                    self.dropped_fields = self.dropped_fields.saturating_add(1);
                    return;
                }
            }
        }
        let name = match std::str::from_utf8(&bytes[..eq]) {
            Ok(name) => name.to_owned(),
            Err(_) => {
                self.dropped_fields = self.dropped_fields.saturating_add(1);
                return;
            }
        };
        let value = bytes[eq + 1..].to_vec();
        if is_first_message {
            // Record the index of the MESSAGE field instead of cloning its
            // value: the body reads the bytes back from `fields`
            // (JournalEntry::message_body), so the message payload is stored
            // once per entry, not twice.
            self.message_body_index = Some(self.fields.len());
        }
        self.fields.push(JournalField { name, value });
        self.copied_entry_bytes = self.copied_entry_bytes.saturating_add(field_len);
        self.copied_field_count = self.copied_field_count.saturating_add(1);
    }

    pub(crate) fn finish(self) -> DecodedFields {
        DecodedFields {
            fields: self.fields,
            message_body_index: self.message_body_index,
            dropped_fields: self.dropped_fields,
        }
    }
}

/// Decode an entry's raw `name=value` field slices into owned [`JournalField`]s.
///
/// A convenience wrapper over [`FieldDecoder`] for sources whose items are all
/// simultaneously valid (tests, benchmarks). The FFI reader instead drives
/// [`FieldDecoder`] directly, feeding one `sd_journal_enumerate_data` slice at a
/// time so no journald pointer is aliased across an enumerate call. Gated to its
/// only callers so a production build (which uses `FieldDecoder` directly) does
/// not flag it as dead code.
#[cfg(any(test, feature = "bench"))]
pub(crate) fn decode_journal_fields<'a, I>(
    raw_fields: I,
    extraction: &ExtractionConfig,
) -> DecodedFields
where
    I: IntoIterator<Item = &'a [u8]>,
{
    let mut decoder = FieldDecoder::new(extraction);
    for bytes in raw_fields {
        decoder.feed(bytes);
    }
    decoder.finish()
}

/// Reference decode used by the out-of-crate `journald_decode` benchmark. It
/// runs the real [`decode_journal_fields`] over one entry's raw fields and
/// returns the kept-field count so the caller can `black_box` the result and the
/// optimizer cannot elide the decode work. Exposed only behind the `bench`
/// feature (and under `cfg(test)`); it is not part of the crate's public API.
#[cfg(any(test, feature = "bench"))]
#[must_use]
pub fn bench_reference_decode(raw: &[&[u8]]) -> usize {
    let extraction = ExtractionConfig {
        max_entry_bytes: 1 << 30,
        max_field_bytes: 1 << 30,
        max_fields_per_entry: 4096,
        large_field_policy: LargeFieldPolicy::DropAndCount,
    };
    let decoded = decode_journal_fields(raw.iter().copied(), &extraction);
    let count = decoded.fields.len();
    // Force the decoded buffers (including each field's copied value) to be
    // materialized so the decode cost is measured rather than optimized away.
    let _ = std::hint::black_box(&decoded);
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an [`ExtractionConfig`] with generous limits so tests opt into the
    /// specific limit they exercise.
    fn cfg(
        max_entry_bytes: u64,
        max_field_bytes: u64,
        max_fields_per_entry: usize,
    ) -> ExtractionConfig {
        ExtractionConfig {
            max_entry_bytes,
            max_field_bytes,
            max_fields_per_entry,
            large_field_policy: LargeFieldPolicy::DropAndCount,
        }
    }

    /// Loose config: nothing gets dropped for size in the small test entries.
    fn loose() -> ExtractionConfig {
        cfg(1 << 20, 1 << 16, 1024)
    }

    fn field<'a>(decoded: &'a DecodedFields, name: &str) -> Option<&'a JournalField> {
        decoded.fields.iter().find(|f| f.name == name)
    }

    #[test]
    fn keeps_all_fields_and_indexes_first_message() {
        let raw: Vec<&[u8]> = vec![b"_PID=42", b"MESSAGE=hello", b"PRIORITY=6"];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.fields.len(), 3);
        assert_eq!(decoded.dropped_fields, 0);
        // MESSAGE is the second kept field -> index 1.
        assert_eq!(decoded.message_body_index, Some(1));
        assert_eq!(
            decoded.fields[decoded.message_body_index.unwrap()].value,
            b"hello".to_vec()
        );
    }

    #[test]
    fn no_message_leaves_body_index_none() {
        let raw: Vec<&[u8]> = vec![b"_PID=42", b"PRIORITY=6"];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.fields.len(), 2);
        assert_eq!(decoded.message_body_index, None);
    }

    #[test]
    fn message_index_accounts_for_earlier_dropped_fields() {
        // The oversized first field is dropped, so MESSAGE becomes kept-index 0
        // even though it is the second raw field.
        let big = format!("BIG={}", "x".repeat(200));
        let raw: Vec<&[u8]> = vec![big.as_bytes(), b"MESSAGE=hi"];
        let decoded = decode_journal_fields(raw, &cfg(1 << 20, 64, 1024));
        assert_eq!(decoded.dropped_fields, 1);
        assert_eq!(decoded.fields.len(), 1);
        assert_eq!(decoded.message_body_index, Some(0));
        assert_eq!(decoded.fields[0].name, "MESSAGE");
    }

    #[test]
    fn dropped_first_message_leaves_body_unset_even_if_later_message_kept() {
        // Pins the encoder contract (`leaves_body_unset_when_first_message_was_dropped`):
        // once the first MESSAGE is seen, no later MESSAGE can become the body.
        let big = format!("MESSAGE={}", "x".repeat(200));
        let raw: Vec<&[u8]> = vec![big.as_bytes(), b"MESSAGE=small"];
        let decoded = decode_journal_fields(raw, &cfg(1 << 20, 64, 1024));
        assert_eq!(decoded.dropped_fields, 1);
        // The small later MESSAGE is kept as a field...
        assert_eq!(decoded.fields.len(), 1);
        assert_eq!(decoded.fields[0].value, b"small".to_vec());
        // ...but the body index stays None because the FIRST MESSAGE was dropped.
        assert_eq!(decoded.message_body_index, None);
    }

    #[test]
    fn duplicate_message_indexes_the_first() {
        let raw: Vec<&[u8]> = vec![b"MESSAGE=first", b"OTHER=x", b"MESSAGE=second"];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.fields.len(), 3);
        assert_eq!(decoded.message_body_index, Some(0));
        assert_eq!(decoded.fields[0].value, b"first".to_vec());
    }

    #[test]
    fn oversized_value_is_dropped_and_counted() {
        let big = format!("DATA={}", "y".repeat(500));
        let raw: Vec<&[u8]> = vec![b"_PID=42", big.as_bytes()];
        let decoded = decode_journal_fields(raw, &cfg(1 << 20, 128, 1024));
        assert_eq!(decoded.dropped_fields, 1);
        assert!(field(&decoded, "DATA").is_none());
        assert!(field(&decoded, "_PID").is_some());
    }

    #[test]
    fn non_utf8_name_is_dropped_and_counted() {
        let raw: Vec<&[u8]> = vec![b"\xff\xfeBAD=value", b"OK=1"];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.dropped_fields, 1);
        assert_eq!(decoded.fields.len(), 1);
        assert_eq!(decoded.fields[0].name, "OK");
    }

    #[test]
    fn field_without_equals_is_ignored_not_dropped() {
        let raw: Vec<&[u8]> = vec![b"NOEQUALSHERE", b"OK=1"];
        let decoded = decode_journal_fields(raw, &loose());
        // Not counted as a drop, just skipped.
        assert_eq!(decoded.dropped_fields, 0);
        assert_eq!(decoded.fields.len(), 1);
        assert_eq!(decoded.fields[0].name, "OK");
    }

    #[test]
    fn max_fields_per_entry_caps_kept_fields() {
        let raw: Vec<&[u8]> = vec![b"A=1", b"B=2", b"C=3", b"D=4"];
        let decoded = decode_journal_fields(raw, &cfg(1 << 20, 1 << 16, 2));
        assert_eq!(decoded.fields.len(), 2);
        assert_eq!(decoded.dropped_fields, 2);
        assert_eq!(decoded.fields[0].name, "A");
        assert_eq!(decoded.fields[1].name, "B");
    }

    #[test]
    fn max_entry_bytes_caps_total_copied() {
        // Each field is 5 bytes ("A=xxx"); a 12-byte cap keeps two, drops the third.
        let raw: Vec<&[u8]> = vec![b"A=xxx", b"B=xxx", b"C=xxx"];
        let decoded = decode_journal_fields(raw, &cfg(12, 1 << 16, 1024));
        assert_eq!(decoded.fields.len(), 2);
        assert_eq!(decoded.dropped_fields, 1);
    }

    #[test]
    fn empty_iterator_yields_empty_decode() {
        let raw: Vec<&[u8]> = vec![];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.fields.len(), 0);
        assert_eq!(decoded.dropped_fields, 0);
        assert_eq!(decoded.message_body_index, None);
    }

    #[test]
    fn threshold_is_field_headroom_bounded_by_entry() {
        // max_field_bytes + headroom, clamped to max_entry_bytes, min 1.
        let c = cfg(1 << 20, 100, 1024);
        assert_eq!(extraction_data_threshold_u64(&c), 100 + 4096);
        let clamped = cfg(50, 100, 1024);
        assert_eq!(extraction_data_threshold_u64(&clamped), 50);
        let zeroed = cfg(0, 0, 1024);
        assert_eq!(extraction_data_threshold_u64(&zeroed), 1);
    }

    #[test]
    fn empty_message_value_is_kept_and_indexed() {
        let raw: Vec<&[u8]> = vec![b"MESSAGE="];
        let decoded = decode_journal_fields(raw, &loose());
        assert_eq!(decoded.fields.len(), 1);
        assert_eq!(decoded.message_body_index, Some(0));
        assert_eq!(decoded.fields[0].value, Vec::<u8>::new());
    }

    #[test]
    fn bench_reference_decode_covers_helper() {
        // Covers the `bench_reference_decode` helper (used by the out-of-crate
        // `journald_decode` benchmark) so it is not reported as uncovered.
        let owned: Vec<Vec<u8>> = vec![b"_PID=1".to_vec(), b"MESSAGE=hi".to_vec()];
        let raw: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        assert_eq!(bench_reference_decode(&raw), 2);
    }
}
