// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use super::structured::{ArrayWriter, MAX_DEPTH, MapWriter, Structured, ValueWriter};
use super::{LogRecord, SavedCallsite};
use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, EncodeFailure, EncodeResult};
use otel_arrow_dfe_pdata::proto::consts::{field_num::common::*, field_num::logs::*, wire_types};
use std::time::SystemTime;
use tracing::Level;

/// Direct encoder that writes a single LogRecord from a tracing Event.
pub struct DirectLogRecordEncoder<'buf, B: BoundedBuf> {
    buf: &'buf mut B,
}

impl<'buf, B: BoundedBuf> DirectLogRecordEncoder<'buf, B> {
    /// Create a new encoder that writes to the provided buffer.
    #[inline]
    pub const fn new(buf: &'buf mut B) -> Self {
        Self { buf }
    }

    /// Reset the underlying buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Encode a tracing Event as a complete LogRecord message.
    ///
    /// Returns the number of bytes written.
    pub fn encode_log_record(&mut self, time: SystemTime, record: &LogRecord) -> usize {
        let start_len = self.buf.len();

        // Convert SystemTime to nanoseconds since UNIX epoch
        let timestamp_ns = time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Encode time_unix_nano (field 1, fixed64)
        let _ = self
            .buf
            .encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        let _ = self.buf.extend_from_slice(&timestamp_ns.to_le_bytes());

        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(record.callsite().level());
        let _ = self
            .buf
            .encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        let _ = self.buf.encode_varint(severity as u64);

        // Note we skip encoding severity_text (field 3, string)

        // Encode event_name (field 12, string) - format: "target::name (file:line)"
        let _ = encode_event_name(self.buf, record.callsite());

        // Pre-encoded body and attributes.
        let _ = self.buf.extend_from_slice(&record.body_attrs_bytes);

        // Encode dropped_attributes_count (field 7, varint) if any were dropped.
        if record.dropped_attributes_count > 0 {
            let _ = self
                .buf
                .encode_field_tag(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
            let _ = self
                .buf
                .encode_varint(record.dropped_attributes_count as u64);
        }

        self.buf.len() - start_len
    }
}

/// Encode the event name from callsite metadata.
///
/// Emits the bare `callsite.name()` (the first argument to `otel_info!` /
/// `otel_warn!` / ...). The tracing target (`callsite.target()`) is **not**
/// prefixed here; it identifies the static logical software unit and is
/// conveyed separately through
/// `InstrumentationScope.name` by [`encode_export_logs_request`] so that
/// `event.name` on the wire matches the value declared in the
/// `rust/otap-dataflow/semconv/` registry and validated by
/// `weaver registry live-check`.
fn encode_event_name<B: BoundedBuf>(buf: &mut B, callsite: SavedCallsite) -> EncodeResult {
    buf.encode_len_delimited(LOG_RECORD_EVENT_NAME, |buf| {
        buf.try_extend(callsite.name().as_bytes())
    })
}

/// Visitor that directly encodes tracing fields to protobuf.
pub struct DirectFieldVisitor<'buf, B: BoundedBuf> {
    buf: &'buf mut B,
    dropped_count: u32,
}

impl<'buf, B: BoundedBuf> DirectFieldVisitor<'buf, B> {
    /// Create a new DirectFieldVisitor that writes to the provided buffer.
    pub const fn new(buf: &'buf mut B) -> Self {
        Self {
            buf,
            dropped_count: 0,
        }
    }

    /// Returns the number of fields dropped due to size limits.
    #[must_use]
    pub const fn dropped_count(&self) -> u32 {
        self.dropped_count
    }

    /// Encodes one attribute within the per-attribute budget, counting a hard
    /// failure or a truncation as one dropped attribute. A hard failure rolls
    /// back any partial bytes; `Ok(true)` from `encode` reports truncation.
    fn write_attribute(&mut self, encode: impl FnOnce(&mut B) -> Result<bool, EncodeFailure>) {
        if bounded(self.buf, encode).unwrap_or(true) {
            self.dropped_count += 1;
        }
    }

    /// Encodes a structured value as one attribute within the per-attribute budget.
    pub fn write_structured(&mut self, key: &str, value: &Structured) {
        self.write_value_attribute(key, |w| value.value().write(w));
    }

    /// Encodes the field as an attribute; scalar values of the `message`
    /// field have no body representation and are ignored.
    fn record_attribute(
        &mut self,
        field: &tracing::field::Field,
        f: impl FnMut(&mut dyn ValueWriter),
    ) {
        if field.name() != "message" {
            self.write_value_attribute(field.name(), f);
        }
    }

    /// Encodes one attribute whose value is written by `f`.
    fn write_value_attribute(&mut self, key: &str, mut f: impl FnMut(&mut dyn ValueWriter)) {
        self.write_attribute(|buf| write_key_value(buf, LOG_RECORD_ATTRIBUTES, key, 0, &mut f));
    }

    /// Encode the body as a string (empty strings are skipped), truncating
    /// safely via [`BoundedBuf::encode_string_truncating`].
    #[inline]
    pub fn encode_body_string(&mut self, value: &str) {
        if !value.is_empty() {
            self.write_body(|w| w.write_str(value));
        }
    }

    /// Encode the body from a Debug value without allocation, truncating
    /// safely (see [`Self::encode_body_string`]).
    #[inline]
    pub fn encode_body_debug(&mut self, value: &dyn std::fmt::Debug) {
        self.write_body(|w| w.write_debug(value));
    }

    /// Encodes the body, written by `f`, rolling it back if it cannot fit.
    fn write_body(&mut self, mut f: impl FnMut(&mut dyn ValueWriter)) {
        let _ = self
            .buf
            .try_encode(|buf| write_value(buf, LOG_RECORD_BODY, 0, &mut f).map(drop));
    }
}

/// Adapter that lets `write!` format directly into a [`BoundedBuf`] without
/// an intermediate `String`.
///
/// When the buffer fills up, the adapter backtracks to make room for a
/// `[...]` truncation suffix and silently discards all subsequent writes.
/// The `truncated` flag records whether truncation occurred so callers can
/// report the partial value instead of dropping it entirely.
struct BoundedBufFmt<'a, B: BoundedBuf> {
    buf: &'a mut B,
    /// Buffer position where formatted content starts (after protobuf framing).
    content_start: usize,
    /// Set to `true` once truncation has occurred.
    truncated: bool,
}

impl<'a, B: BoundedBuf> BoundedBufFmt<'a, B> {
    /// Create a new adapter. `content_start` is recorded as the current
    /// buffer length so the backtracking logic knows which bytes are ours.
    #[inline]
    fn new(buf: &'a mut B) -> Self {
        let content_start = buf.len();
        Self {
            buf,
            content_start,
            truncated: false,
        }
    }

    /// Backtrack from the end of the buffer to make room for `TRUNCATION_SUFFIX`,
    /// append it, and mark ourselves as truncated.
    ///
    /// If there is not enough content to free space for the suffix the buffer
    /// is left at `content_start` (empty content) and the caller should treat
    /// this as a hard drop.
    fn truncate_with_suffix(&mut self) {
        use otel_arrow_dfe_pdata::otlp::common::TRUNCATION_SUFFIX;

        let suffix_len = TRUNCATION_SUFFIX.len();
        let cur = self.buf.len();

        // Target position: back up `suffix_len` bytes from the current end,
        // then walk backwards (at most 3 extra bytes) to a UTF-8 char boundary.
        // UTF-8 continuation bytes have the bit pattern 10xxxxxx, so we skip
        // them until we land on a leading byte.
        let target = cur.saturating_sub(suffix_len).max(self.content_start);
        let mut pos = target;
        let slice = self.buf.as_slice();
        while pos > self.content_start && (slice[pos] & 0xC0) == 0x80 {
            pos -= 1;
        }
        self.buf.truncate(pos);

        // Try to append the suffix. If even this fails (degenerate budget),
        // leave the buffer at whatever position truncate() set.
        let _ = self.buf.try_extend(TRUNCATION_SUFFIX);
        self.truncated = true;
    }
}

impl<B: BoundedBuf> std::fmt::Write for BoundedBufFmt<'_, B> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.truncated {
            // Already truncated -- silently discard further output so the
            // formatter finishes cleanly.
            return Ok(());
        }
        match self.buf.try_extend(s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(_) => {
                // The full chunk did not fit. Write as much as we can, then
                // backtrack to append the truncation suffix.
                let remaining = self.buf.remaining();
                if remaining > 0 {
                    // Find the last UTF-8 char boundary within the portion
                    // of `s` that fits.
                    let mut end = remaining;
                    while end > 0 && !s.is_char_boundary(end) {
                        end -= 1;
                    }
                    if end > 0 {
                        // This extend cannot fail -- we checked remaining.
                        let _ = self.buf.try_extend(&s.as_bytes()[..end]);
                    }
                }
                self.truncate_with_suffix();
                Ok(())
            }
        }
    }
}

/// Encode a `Debug` value as a protobuf string field with truncation support.
///
/// If the formatted output fits, returns `Ok(false)`. If it overflows the
/// buffer, the value is truncated with a `[...]` suffix and `Ok(true)` is
/// returned. `Err(Dropped)` is returned only when even a minimal truncated
/// form cannot fit (nothing useful was written).
#[inline]
fn encode_debug_string<B: BoundedBuf>(
    buf: &mut B,
    value: &dyn std::fmt::Debug,
) -> Result<bool, EncodeFailure> {
    let mut truncated = false;
    buf.encode_len_delimited_partial(ANY_VALUE_STRING_VALUE, |buf| {
        use std::fmt::Write as _;
        let mut adapter = BoundedBufFmt::new(buf);
        // Ignore the fmt result -- truncation is handled internally by the
        // adapter; the write!() call always returns Ok because the adapter
        // silently discards overflow bytes after appending the suffix.
        let _ = write!(adapter, "{:?}", value);
        // Extract state before dropping the adapter (which borrows buf).
        let was_truncated = adapter.truncated;
        let content_start = adapter.content_start;
        truncated = was_truncated;
        if was_truncated && buf.len() <= content_start {
            // Nothing useful was written (not even the suffix fit).
            return Err(EncodeFailure::Dropped);
        }
        Ok(())
    })?;
    Ok(truncated)
}

/// Compute the per-attribute budget: at most half of remaining space, but
/// at least 1 byte (to ensure a chance of a tag byte being written and
/// triggering an atomic drop). Intentionally halves for every attribute,
/// including the last -- this keeps the policy single-pass without requiring
/// pre-counting the field set, while still preventing one large value from
/// consuming the entire remaining buffer.
#[inline]
fn attr_budget<B: BoundedBuf>(buf: &B) -> usize {
    buf.remaining().div_ceil(2)
}

/// Runs `encode` transactionally within the per-attribute budget. `Ok(true)`
/// means it succeeded but truncated; `Err` means it was rolled back.
fn bounded<B: BoundedBuf>(
    buf: &mut B,
    encode: impl FnOnce(&mut B) -> Result<bool, EncodeFailure>,
) -> Result<bool, EncodeFailure> {
    let budget = attr_budget(buf);
    let mut truncated = false;
    buf.with_max_remaining(budget, |buf| {
        buf.try_encode(|buf| {
            truncated = encode(buf)?;
            Ok::<_, EncodeFailure>(())
        })
    })?;
    Ok(truncated)
}

/// Encodes `f`'s value as a length-delimited `field` at `depth`. `Ok(true)`
/// means part of the value was lost; `Err` means it could not be written and
/// must be rolled back.
fn write_value<B: BoundedBuf>(
    buf: &mut B,
    field: u64,
    depth: usize,
    f: &mut dyn FnMut(&mut dyn ValueWriter),
) -> Result<bool, EncodeFailure> {
    let mut lost = false;
    buf.encode_len_delimited(field, |buf| {
        let mut writer = ValueEncoder {
            buf,
            depth,
            lost: false,
            failed: false,
        };
        f(&mut writer);
        lost = writer.lost;
        if writer.failed {
            Err(EncodeFailure::Dropped)
        } else {
            Ok(())
        }
    })?;
    Ok(lost)
}

/// Encodes a `KeyValue` as a length-delimited `field`; see [`write_value`].
fn write_key_value<B: BoundedBuf>(
    buf: &mut B,
    field: u64,
    key: &str,
    depth: usize,
    f: &mut dyn FnMut(&mut dyn ValueWriter),
) -> Result<bool, EncodeFailure> {
    let mut lost = false;
    buf.encode_len_delimited(field, |buf| {
        buf.encode_string(KEY_VALUE_KEY, key)?;
        lost = write_value(buf, KEY_VALUE_VALUE, depth, f)?;
        Ok::<_, EncodeFailure>(())
    })?;
    Ok(lost)
}

/// Writes one AnyValue into a bounded buffer.
struct ValueEncoder<'a, B: BoundedBuf> {
    buf: &'a mut B,
    depth: usize,
    lost: bool,
    failed: bool,
}

impl<B: BoundedBuf> ValueEncoder<'_, B> {
    fn put(&mut self, result: Result<bool, EncodeFailure>) {
        match result {
            Ok(truncated) => self.lost |= truncated,
            Err(_) => self.failed = true,
        }
    }

    /// Writes a length-delimited container at the next depth; `body` returns
    /// whether part of its content was lost. Containers beyond [`MAX_DEPTH`]
    /// are dropped.
    fn nested(&mut self, field: u64, body: impl FnOnce(&mut B, usize) -> bool) {
        if self.depth >= MAX_DEPTH {
            self.lost = true;
            return;
        }
        let depth = self.depth + 1;
        let mut lost = false;
        let result = self.buf.encode_len_delimited(field, |buf| {
            lost = body(buf, depth);
            Ok::<_, EncodeFailure>(())
        });
        self.put(result.map(|()| lost));
    }

    fn put_tagged(
        &mut self,
        field: u64,
        wire_type: u64,
        body: impl FnOnce(&mut B) -> EncodeResult,
    ) {
        let result = self
            .buf
            .encode_field_tag(field, wire_type)
            .and_then(|()| body(self.buf))
            .map(|()| false);
        self.put(result);
    }
}

impl<B: BoundedBuf> ValueWriter for ValueEncoder<'_, B> {
    fn write_str(&mut self, value: &str) {
        let result = self
            .buf
            .encode_string_truncating(ANY_VALUE_STRING_VALUE, value);
        self.put(result);
    }

    fn write_debug(&mut self, value: &dyn std::fmt::Debug) {
        let result = encode_debug_string(self.buf, value);
        self.put(result);
    }

    fn write_i64(&mut self, value: i64) {
        self.put_tagged(ANY_VALUE_INT_VALUE, wire_types::VARINT, |buf| {
            buf.encode_varint(value as u64)
        });
    }

    fn write_f64(&mut self, value: f64) {
        self.put_tagged(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64, |buf| {
            buf.extend_from_slice(&value.to_le_bytes())
        });
    }

    fn write_bool(&mut self, value: bool) {
        self.put_tagged(ANY_VALUE_BOOL_VALUE, wire_types::VARINT, |buf| {
            buf.encode_varint(u64::from(value))
        });
    }

    fn write_map(&mut self, f: &mut dyn FnMut(&mut dyn MapWriter)) {
        self.nested(ANY_VALUE_KVLIST_VALUE, |buf, depth| {
            let mut writer = MapEncoder {
                buf,
                depth,
                lost: false,
            };
            f(&mut writer);
            writer.lost
        });
    }

    fn write_array(&mut self, f: &mut dyn FnMut(&mut dyn ArrayWriter)) {
        self.nested(ANY_VALUE_ARRAY_VALUE, |buf, depth| {
            let mut writer = ArrayEncoder {
                buf,
                depth,
                lost: false,
                stopped: false,
            };
            f(&mut writer);
            writer.lost
        });
    }
}

/// Writes the entries of a map; `depth` is the depth of its values.
struct MapEncoder<'a, B: BoundedBuf> {
    buf: &'a mut B,
    depth: usize,
    lost: bool,
}

impl<B: BoundedBuf> MapWriter for MapEncoder<'_, B> {
    fn write_entry(&mut self, key: &str, f: &mut dyn FnMut(&mut dyn ValueWriter)) {
        let result = bounded(self.buf, |buf| {
            write_key_value(buf, KEY_VALUE_LIST_VALUES, key, self.depth, f)
        });
        self.lost |= result.unwrap_or(true);
    }
}

/// Writes the elements of an array; `depth` is the depth of its values.
struct ArrayEncoder<'a, B: BoundedBuf> {
    buf: &'a mut B,
    depth: usize,
    lost: bool,
    stopped: bool,
}

impl<B: BoundedBuf> ArrayWriter for ArrayEncoder<'_, B> {
    fn write_element(&mut self, f: &mut dyn FnMut(&mut dyn ValueWriter)) {
        if self.stopped {
            return;
        }
        let result = bounded(self.buf, |buf| {
            write_value(buf, ARRAY_VALUE_VALUES, self.depth, f)
        });
        // Keep a prefix rather than shifting later element positions.
        self.stopped = result.is_err();
        self.lost |= result.unwrap_or(true);
    }
}

impl<B: BoundedBuf> tracing::field::Visit for DirectFieldVisitor<'_, B> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.record_attribute(field, |w| w.write_f64(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.record_attribute(field, |w| w.write_i64(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.record_i64(field, value as i64);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.record_attribute(field, |w| w.write_bool(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.encode_body_string(value);
        } else {
            self.write_value_attribute(field.name(), |w| w.write_str(value));
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.encode_body_debug(value);
        } else {
            self.write_value_attribute(field.name(), |w| w.write_debug(value));
        }
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        match value.downcast_ref::<Structured>() {
            Some(structured) => self.write_structured(field.name(), structured),
            None => self.record_debug(field, &format_args!("{value}")),
        }
    }
}

/// Convert tracing Level to OTLP severity number.
///
/// See: <https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber>
#[inline]
#[must_use]
pub const fn level_to_severity_number(level: &Level) -> u8 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::__log_record_impl;
    use crate::LogContext;
    use crate::self_tracing::StructuredValue;
    use otel_arrow_dfe_pdata::otlp::common::ProtoBuffer;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use prost::Message;
    use std::sync::Arc;
    use tracing::Level;

    /// Verify the fair-budget halving policy for string attributes.
    ///
    /// With a 256-byte inline buffer and 4 attributes whose values are 1000
    /// bytes each, every attribute should consume roughly half of whatever
    /// remained, leaving each successive value smaller than the previous.
    /// All four values are truncated, so `dropped_attributes_count` is 4.
    #[test]
    fn record_str_halves_remaining_budget_across_attributes() {
        use crate::self_tracing::encoder::DirectFieldVisitor;
        use otel_arrow_dfe_pdata::otlp::common::TRUNCATION_SUFFIX;
        use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, StackProtoBuffer};
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        const INLINE: usize = 256;
        let big = "x".repeat(1000);

        let meta = &BUDGET_TEST_METADATA;
        let fields = meta.fields();
        let a = fields.field("a").unwrap();
        let b = fields.field("b").unwrap();
        let c = fields.field("c").unwrap();
        let d = fields.field("d").unwrap();

        // Drive the visitor directly with each (field, value) pair, which is
        // what `event.record(visitor)` does internally.
        let mut buf = StackProtoBuffer::<INLINE>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_str(&a, big.as_str());
        visitor.record_str(&b, big.as_str());
        visitor.record_str(&c, big.as_str());
        visitor.record_str(&d, big.as_str());
        let dropped = visitor.dropped_count();

        // All four values should be truncated.
        assert_eq!(dropped, 4, "expected 4 truncated attributes");

        // Buffer should not exceed the inline limit.
        assert!(
            buf.len() <= INLINE,
            "buffer len {} exceeds INLINE {}",
            buf.len(),
            INLINE
        );

        // Decode each attribute as a KeyValue (field tag = LOG_RECORD_ATTRIBUTES = 6,
        // wire type LEN). Walk the buffer manually.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let mut decoded: Vec<(String, String)> = Vec::new();
        while !cursor.is_empty() {
            let (tag, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let field_num = tag >> 3;
            let wire_type = tag & 0x7;
            assert_eq!(wire_type, wire_types::LEN, "expected LEN wire type");
            assert_eq!(field_num, LOG_RECORD_ATTRIBUTES, "expected ATTRIBUTES tag");
            let (len, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let len = len as usize;
            let kv_bytes = &cursor[..len];
            cursor = &cursor[len..];
            let kv = ProtoKeyValue::decode(kv_bytes).unwrap();
            let val = kv
                .value
                .as_ref()
                .and_then(|v| v.value.as_ref())
                .map(|v| match v {
                    otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => s.clone(),
                    _ => panic!("expected string value"),
                })
                .unwrap();
            decoded.push((kv.key, val));
        }

        assert_eq!(decoded.len(), 4, "expected 4 decoded attributes");

        // Each value should end with the truncation suffix.
        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        for (k, v) in &decoded {
            assert!(
                v.ends_with(suffix),
                "attribute {k} value should end with {suffix}, got len {}",
                v.len()
            );
        }

        // Each successive value should be smaller (halving budget). We compare
        // value lengths rather than exact sizes to avoid brittleness.
        let lens: Vec<usize> = decoded.iter().map(|(_, v)| v.len()).collect();
        assert!(
            lens.windows(2).all(|w| w[0] > w[1]),
            "expected strictly decreasing value lengths, got {lens:?}"
        );

        // Sanity-check the rough magnitudes from the user's spec
        // (~100 / ~50 / ~25 / ~10), accommodating overhead variation.
        // Observed: ~[118, 55, 23, 7].
        assert!(
            lens[0] > 60 && lens[0] < 140,
            "1st value len {} out of expected range",
            lens[0]
        );
        assert!(
            lens[1] > 25 && lens[1] < 80,
            "2nd value len {} out of expected range",
            lens[1]
        );
        assert!(
            lens[2] > 10 && lens[2] < 50,
            "3rd value len {} out of expected range",
            lens[2]
        );
        // Last value may be just the suffix (~5 bytes) up to ~25 bytes.
        assert!(
            lens[3] >= suffix.len() && lens[3] < 30,
            "4th value len {} out of expected range",
            lens[3]
        );
    }

    /// Verify `record_debug` formats a Debug value directly into the buffer
    /// (via the `BoundedBufFmt` adapter) and produces a decodable KeyValue
    /// whose string matches `format!("{:?}", value)` -- i.e. no truncation
    /// when the value comfortably fits.
    #[test]
    fn record_debug_writes_fmt_output_without_intermediate_string() {
        use otel_arrow_dfe_pdata::otlp::common::StackProtoBuffer;
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        #[derive(Debug)]
        #[allow(dead_code)]
        struct Payload {
            id: u32,
            label: &'static str,
        }
        let value = Payload {
            id: 7,
            label: "hello",
        };
        let expected_repr = format!("{:?}", value);

        let meta = &DEBUG_TEST_METADATA;
        let fields = meta.fields();
        let dbg = fields.field("dbg").unwrap();

        let mut buf = StackProtoBuffer::<256>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&dbg, &value);
        assert_eq!(visitor.dropped_count(), 0, "no fields should be dropped");

        // Decode the single KeyValue from the buffer.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let (tag, n) = read_varint(cursor);
        cursor = &cursor[n..];
        assert_eq!(tag >> 3, LOG_RECORD_ATTRIBUTES);
        assert_eq!(tag & 0x7, wire_types::LEN);
        let (len, n) = read_varint(cursor);
        cursor = &cursor[n..];
        let kv = ProtoKeyValue::decode(&cursor[..len as usize]).unwrap();
        assert_eq!(kv.key, "dbg");
        let s = match kv.value.as_ref().unwrap().value.as_ref().unwrap() {
            otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                s.clone()
            }
            other => panic!("expected StringValue, got {other:?}"),
        };
        assert_eq!(s, expected_repr);
    }

    /// Scenario: a Debug value overflows the available buffer.
    /// Guarantees: the value is truncated with a `[...]` suffix instead of
    /// being dropped entirely, `dropped_count` is incremented, and the
    /// encoded KeyValue is decodable with the suffix at the end.
    #[test]
    fn record_debug_overflow_truncates_with_suffix() {
        use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, StackProtoBuffer, TRUNCATION_SUFFIX};
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        // A Debug impl that writes far more than the buffer can hold.
        struct Huge;
        impl std::fmt::Debug for Huge {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // 4 KiB of output into a 64-byte buffer.
                for _ in 0..512 {
                    f.write_str("xxxxxxxx")?;
                }
                Ok(())
            }
        }

        let meta = &DEBUG_TEST_METADATA;
        let fields = meta.fields();
        let dbg = fields.field("dbg").unwrap();

        let mut buf = StackProtoBuffer::<64>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&dbg, &Huge);
        assert_eq!(
            visitor.dropped_count(),
            1,
            "the single oversized debug field should be counted as dropped (truncated)"
        );

        // Buffer should contain a truncated KeyValue, not be empty.
        assert!(
            buf.len() > 0,
            "buffer should contain a truncated attribute, not be empty"
        );

        // Decode the KeyValue and verify the value ends with the suffix.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let (tag, n) = read_varint(cursor);
        cursor = &cursor[n..];
        assert_eq!(tag >> 3, LOG_RECORD_ATTRIBUTES);
        assert_eq!(tag & 0x7, wire_types::LEN);
        let (len, n) = read_varint(cursor);
        cursor = &cursor[n..];
        let kv = ProtoKeyValue::decode(&cursor[..len as usize]).unwrap();
        assert_eq!(kv.key, "dbg");
        let s = match kv.value.as_ref().unwrap().value.as_ref().unwrap() {
            otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                s.clone()
            }
            other => panic!("expected StringValue, got {other:?}"),
        };
        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        assert!(
            s.ends_with(suffix),
            "truncated debug value should end with {suffix}, got: {s}"
        );
        // The value should contain some content before the suffix.
        assert!(
            s.len() > suffix.len(),
            "truncated value should have content before the suffix"
        );
    }

    /// Scenario: a Debug value cannot fit even in truncated form because the
    /// buffer is too small for key + suffix + protobuf overhead.
    /// Guarantees: the attribute is hard-dropped (buffer rolled back to
    /// pre-call length) and `dropped_count` is incremented.
    #[test]
    fn record_debug_hard_drop_when_budget_too_small() {
        use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, StackProtoBuffer};
        use tracing::field::Visit;

        struct Tiny;
        impl std::fmt::Debug for Tiny {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("x")
            }
        }

        let meta = &DEBUG_TEST_METADATA;
        let fields = meta.fields();
        let dbg = fields.field("dbg").unwrap();

        // A buffer so tiny that even the key "dbg" + protobuf overhead
        // cannot fit, forcing a hard drop.
        let mut buf = StackProtoBuffer::<8>::default();
        let before_len = buf.len();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&dbg, &Tiny);
        assert_eq!(
            visitor.dropped_count(),
            1,
            "the field should be counted as dropped"
        );
        assert_eq!(
            buf.len(),
            before_len,
            "buffer must be rolled back to its pre-call length on hard drop"
        );
    }

    /// Scenario: a bare `&str` `message` body overflows the available buffer.
    /// Guarantees: the decoded body ends with `[...]` without incrementing
    /// OTLP `dropped_attributes_count`.
    #[test]
    fn record_str_message_overflow_truncates_with_suffix() {
        use otel_arrow_dfe_pdata::otlp::common::{StackProtoBuffer, TRUNCATION_SUFFIX};
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value;
        use prost::Message;
        use tracing::field::Visit;

        let meta = &MESSAGE_TEST_METADATA;
        let fields = meta.fields();
        let message = fields.field("message").unwrap();

        // 4 KiB of content into a 64-byte buffer.
        let huge = "x".repeat(4096);

        let mut buf = StackProtoBuffer::<64>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_str(&message, &huge);
        assert_eq!(
            visitor.dropped_count(),
            0,
            "body truncation must not be counted as a dropped attribute"
        );

        assert!(
            buf.len() > 0,
            "buffer should contain a truncated body, not be empty"
        );

        let record = LogRecord::decode(buf.as_ref()).expect("log record should decode");
        assert_eq!(record.dropped_attributes_count, 0);
        let s = match record.body.unwrap().value.unwrap() {
            Value::StringValue(s) => s,
            other => panic!("expected StringValue, got {other:?}"),
        };
        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        assert!(
            s.ends_with(suffix),
            "truncated body should end with {suffix}, got: {s}"
        );
        assert!(
            s.len() > suffix.len(),
            "truncated body should have content before the suffix"
        );
    }

    /// Scenario: a Debug-formatted `message` body overflows the available buffer.
    /// Guarantees: the decoded body ends with `[...]` without incrementing
    /// OTLP `dropped_attributes_count`.
    #[test]
    fn record_debug_message_overflow_truncates_with_suffix() {
        use otel_arrow_dfe_pdata::otlp::common::{StackProtoBuffer, TRUNCATION_SUFFIX};
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value;
        use prost::Message;
        use tracing::field::Visit;

        struct Huge;
        impl std::fmt::Debug for Huge {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for _ in 0..512 {
                    f.write_str("xxxxxxxx")?;
                }
                Ok(())
            }
        }

        let fields = MESSAGE_TEST_METADATA.fields();
        let message = fields.field("message").unwrap();
        let mut buf = StackProtoBuffer::<64>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&message, &Huge);
        assert_eq!(
            visitor.dropped_count(),
            0,
            "body truncation must not be counted as a dropped attribute"
        );

        let record = LogRecord::decode(buf.as_ref()).expect("log record should decode");
        assert_eq!(record.dropped_attributes_count, 0);
        let s = match record.body.unwrap().value.unwrap() {
            Value::StringValue(s) => s,
            other => panic!("expected StringValue, got {other:?}"),
        };
        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        assert!(
            s.ends_with(suffix),
            "truncated body should end with {suffix}, got: {s}"
        );
        assert!(
            s.len() > suffix.len(),
            "truncated body should have content before the suffix"
        );
    }

    static MESSAGE_TEST_CALLSITE: MessageTestCallsite = MessageTestCallsite;

    struct MessageTestCallsite;

    impl tracing::Callsite for MessageTestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &MESSAGE_TEST_METADATA
        }
    }

    static MESSAGE_TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "message_test",
        "otel-arrow-dfe-telemetry",
        Level::INFO,
        Some(file!()),
        Some(line!()),
        Some(module_path!()),
        tracing::field::FieldSet::new(
            &["message"],
            tracing::callsite::Identifier(&MESSAGE_TEST_CALLSITE),
        ),
        tracing::metadata::Kind::EVENT,
    );

    /// Scenario: four Debug values compete for a 256-byte buffer with the
    /// halving budget policy.
    /// Guarantees: all four values are truncated (each ends with `[...]`),
    /// each successive value is shorter than the previous, and the buffer
    /// does not exceed the inline limit.
    #[test]
    fn record_debug_halves_remaining_budget_across_attributes() {
        use otel_arrow_dfe_pdata::otlp::common::TRUNCATION_SUFFIX;
        use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, StackProtoBuffer};
        use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        const INLINE: usize = 256;

        // A Debug impl that writes 1000 bytes.
        struct Big;
        impl std::fmt::Debug for Big {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for _ in 0..125 {
                    f.write_str("xxxxxxxx")?;
                }
                Ok(())
            }
        }

        let meta = &BUDGET_TEST_METADATA;
        let fields = meta.fields();
        let a = fields.field("a").unwrap();
        let b = fields.field("b").unwrap();
        let c = fields.field("c").unwrap();
        let d = fields.field("d").unwrap();

        let mut buf = StackProtoBuffer::<INLINE>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&a, &Big);
        visitor.record_debug(&b, &Big);
        visitor.record_debug(&c, &Big);
        visitor.record_debug(&d, &Big);
        let dropped = visitor.dropped_count();

        assert_eq!(dropped, 4, "expected 4 truncated attributes");
        assert!(
            buf.len() <= INLINE,
            "buffer len {} exceeds INLINE {}",
            buf.len(),
            INLINE
        );

        // Decode each attribute.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let mut decoded: Vec<(String, String)> = Vec::new();
        while !cursor.is_empty() {
            let (tag, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let field_num = tag >> 3;
            let wire_type = tag & 0x7;
            assert_eq!(wire_type, wire_types::LEN, "expected LEN wire type");
            assert_eq!(field_num, LOG_RECORD_ATTRIBUTES, "expected ATTRIBUTES tag");
            let (len, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let len = len as usize;
            let kv_bytes = &cursor[..len];
            cursor = &cursor[len..];
            let kv = ProtoKeyValue::decode(kv_bytes).unwrap();
            let val = kv
                .value
                .as_ref()
                .and_then(|v| v.value.as_ref())
                .map(|v| match v {
                    otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => s.clone(),
                    _ => panic!("expected string value"),
                })
                .unwrap();
            decoded.push((kv.key, val));
        }

        assert_eq!(decoded.len(), 4, "expected 4 decoded attributes");

        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        for (k, v) in &decoded {
            assert!(
                v.ends_with(suffix),
                "attribute {k} value should end with {suffix}, got len {}",
                v.len()
            );
        }

        // Each successive value should be no longer than the previous one
        // as the remaining budget is halved across attributes.
        let lens: Vec<usize> = decoded.iter().map(|(_, v)| v.len()).collect();
        assert!(
            lens.windows(2).all(|w| w[0] >= w[1]),
            "expected non-increasing value lengths, got {lens:?}"
        );
    }

    static DEBUG_TEST_CALLSITE: DebugTestCallsite = DebugTestCallsite;

    struct DebugTestCallsite;

    impl tracing::Callsite for DebugTestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &DEBUG_TEST_METADATA
        }
    }

    static DEBUG_TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "debug_test",
        "otel-arrow-dfe-telemetry",
        Level::INFO,
        Some(file!()),
        Some(line!()),
        Some(module_path!()),
        tracing::field::FieldSet::new(
            &["dbg"],
            tracing::callsite::Identifier(&DEBUG_TEST_CALLSITE),
        ),
        tracing::metadata::Kind::EVENT,
    );

    /// Read an unsigned varint, returning (value, bytes_consumed).
    fn read_varint(bytes: &[u8]) -> (u64, usize) {
        let mut value = 0u64;
        let mut shift = 0;
        for (i, &b) in bytes.iter().enumerate() {
            value |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return (value, i + 1);
            }
            shift += 7;
        }
        panic!("truncated varint");
    }

    /// A plain struct that writes itself as a nested map.
    struct Probe {
        name: String,
        count: i64,
        tags: Vec<String>,
        child: Option<Box<Probe>>,
    }

    impl StructuredValue for Probe {
        fn write(&self, w: &mut dyn ValueWriter) {
            w.map(|m| {
                m.entry("name", |v| v.write_str(&self.name));
                m.entry("count", |v| v.write_i64(self.count));
                m.entry("tags", |v| {
                    v.array(|a| {
                        for tag in &self.tags {
                            a.element(|v| v.write_str(tag));
                        }
                    })
                });
                if let Some(child) = &self.child {
                    m.entry("child", |v| child.write(v));
                }
            });
        }
    }

    fn probe(depth: usize) -> Probe {
        Probe {
            name: "n".into(),
            count: -7,
            tags: vec!["a".into(), "b".into()],
            child: (depth > 0).then(|| Box::new(probe(depth - 1))),
        }
    }

    fn encode_probe(limit: usize, value: Probe) -> (ProtoBuffer, u32) {
        let structured = Structured::from_arc(Arc::new(value));
        let mut buf = ProtoBuffer::with_capacity_and_limit(limit, limit);
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.write_structured("probe", &structured);
        let dropped = visitor.dropped_count();
        (buf, dropped)
    }

    /// Scenario: one Arc is emitted in several events and then updated in place.
    /// Guarantees: emitting leaves the caller as sole owner, so Arc::get_mut succeeds and later events see the new value.
    #[test]
    fn shared_value_can_be_updated_in_place_between_events() {
        let mut slot = Arc::new(probe(0));
        for count in [1, 2] {
            Arc::get_mut(&mut slot).expect("sole owner").count = count;
            let structured = Structured::from_arc(slot.clone());
            let record = __log_record_impl!(
                Level::INFO,
                "test.structured",
                probe = structured.as_value()
            )
            .into_record(LogContext::new());
            drop(structured);
            assert_eq!(record.dropped_attributes_count, 0);
            let decoded =
                LogRecord::decode(record.body_attrs_bytes.as_ref()).expect("valid record");
            let Some(otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::KvlistValue(map)) =
                decoded.attributes[0].value.as_ref().and_then(|v| v.value.as_ref())
            else {
                panic!("expected map");
            };
            let got = map.values[1].value.as_ref().and_then(|v| v.value.as_ref());
            assert!(
                matches!(got, Some(otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::IntValue(n)) if *n == count)
            );
        }
    }

    /// Scenario: a structured value meets small budgets and excessive nesting.
    /// Guarantees: output never exceeds the limit, stays decodable, and counts one dropped attribute per partial value.
    #[test]
    fn structured_value_is_bounded_and_reports_truncation() {
        for limit in [8, 32, 64, 128] {
            let (buf, dropped) = encode_probe(limit, probe(2));
            assert!(buf.len() <= limit);
            let _ = LogRecord::decode(buf.as_ref()).expect("valid truncated record");
            assert_eq!(dropped, 1, "limit {limit}");
        }

        let (buf, dropped) = encode_probe(16384, probe(2));
        assert_eq!(dropped, 0);
        assert_eq!(
            LogRecord::decode(buf.as_ref())
                .expect("valid")
                .attributes
                .len(),
            1
        );

        let (buf, dropped) = encode_probe(16384, probe(40));
        assert_eq!(dropped, 1);
        let _ = LogRecord::decode(buf.as_ref()).expect("valid depth-limited record");
    }

    static BUDGET_TEST_CALLSITE: BudgetTestCallsite = BudgetTestCallsite;

    struct BudgetTestCallsite;

    impl tracing::Callsite for BudgetTestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &BUDGET_TEST_METADATA
        }
    }

    static BUDGET_TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "budget_test",
        "otel-arrow-dfe-telemetry",
        Level::INFO,
        Some(file!()),
        Some(line!()),
        Some(module_path!()),
        tracing::field::FieldSet::new(
            &["a", "b", "c", "d"],
            tracing::callsite::Identifier(&BUDGET_TEST_CALLSITE),
        ),
        tracing::metadata::Kind::EVENT,
    );
}
