// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Arrow encoding for Windows ETW logs.
//!
//! Converts [`EtwEventData`] into OTAP Arrow log record batches, following
//! the same builder pattern used by the Linux `user_events_receiver`.

use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use otap_df_pdata::encode::Result;
use otap_df_pdata::encode::record::{
    attributes::StrKeysAttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder,
};
use otap_df_pdata::otap::{Logs, OtapArrowRecords};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::session::{EtwAttributeValue, EtwEventData};

// ── ETW level → OTel severity number mapping ─────────────────────────────────

/// Map an ETW event level to the closest OpenTelemetry severity number.
///
/// ETW levels:
/// - 1 = Critical  → OTEL FATAL  (21)
/// - 2 = Error     → OTEL ERROR  (17)
/// - 3 = Warning   → OTEL WARN   (13)
/// - 4 = Info      → OTEL INFO   (9)
/// - 5 = Verbose   → OTEL DEBUG  (5)
///
/// Unknown levels map to OTEL UNSPECIFIED (0).
const fn etw_level_to_otel_severity(level: u8) -> i32 {
    match level {
        1 => 21, // FATAL
        2 => 17, // ERROR
        3 => 13, // WARN
        4 => 9,  // INFO
        5 => 5,  // DEBUG
        _ => 0,  // UNSPECIFIED
    }
}

/// Map an ETW level to the conventional OTel severity text.
const fn etw_level_to_severity_text(level: u8) -> Option<&'static str> {
    match level {
        1 => Some("FATAL"),
        2 => Some("ERROR"),
        3 => Some("WARN"),
        4 => Some("INFO"),
        5 => Some("DEBUG"),
        _ => None,
    }
}

// ── Decoded field → attribute value conversion ───────────────────────────────

/// Typed attribute value for Arrow encoding.
enum AttrValue<'a> {
    Str(Cow<'a, str>),
    Int(i64),
    Double(f64),
    Bool(bool),
    Bytes(&'a [u8]),
}

/// Lowercase hex digits used when formatting GUID byte arrays.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Format a 16-byte GUID/UUID into a fixed 36-byte stack buffer as
/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`, avoiding a heap allocation.
///
/// Returns the buffer; callers turn it into a `&str` (the output is always
/// valid ASCII/UTF-8) for attribute encoding.
fn format_guid(guid: &[u8; 16]) -> [u8; 36] {
    // Positions of the four '-' separators in the canonical GUID layout.
    const DASH_AT: [usize; 4] = [8, 13, 18, 23];

    let mut out = [b'-'; 36];
    let mut byte_idx = 0;
    let mut out_idx = 0;
    while out_idx < out.len() {
        if DASH_AT.contains(&out_idx) {
            // Leave the pre-filled '-' in place and advance past it.
            out_idx += 1;
            continue;
        }
        let byte = guid[byte_idx];
        out[out_idx] = HEX_DIGITS[(byte >> 4) as usize];
        out[out_idx + 1] = HEX_DIGITS[(byte & 0x0f) as usize];
        byte_idx += 1;
        out_idx += 2;
    }
    out
}

/// Map a decoder-produced [`EtwAttributeValue`] to the encoder's borrowing
/// [`AttrValue`].
///
/// All type interpretation already happened in the decoder
/// (`session::interpret_field_value`), so this is an exhaustive, allocation-free
/// match — adding a new [`EtwAttributeValue`] variant is a compile error here.
fn decode_field_value(value: &EtwAttributeValue) -> AttrValue<'_> {
    match value {
        EtwAttributeValue::Str(s) => AttrValue::Str(Cow::Borrowed(s)),
        EtwAttributeValue::Int(i) => AttrValue::Int(*i),
        EtwAttributeValue::Double(d) => AttrValue::Double(*d),
        EtwAttributeValue::Bool(b) => AttrValue::Bool(*b),
        EtwAttributeValue::Bytes(b) => AttrValue::Bytes(b),
    }
}

// ── Arrow records builder ────────────────────────────────────────────────────

/// Builder for creating Arrow record batches from ETW events.
pub(super) struct EtwArrowRecordsBuilder {
    curr_log_id: u16,
    logs: LogsRecordBatchBuilder,
    log_attrs: StrKeysAttributesRecordBatchBuilder<u16>,
}

impl Default for EtwArrowRecordsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EtwArrowRecordsBuilder {
    /// Creates a new builder.
    #[must_use]
    pub(super) fn new() -> Self {
        Self {
            curr_log_id: 0,
            logs: LogsRecordBatchBuilder::new(),
            log_attrs: StrKeysAttributesRecordBatchBuilder::<u16>::new(),
        }
    }

    /// Returns the number of buffered log records.
    #[must_use]
    pub(super) const fn len(&self) -> u16 {
        self.curr_log_id
    }

    /// Returns true when the builder is empty.
    #[must_use]
    pub(super) const fn is_empty(&self) -> bool {
        self.curr_log_id == 0
    }

    /// Appends an ETW event as an OTAP log record.
    ///
    /// The ETW event timestamp has already been converted from QPC ticks to
    /// Unix epoch nanoseconds by the session callback thread using a
    /// reference point captured at session start.
    pub(super) fn append(&mut self, event: &EtwEventData) {
        let timestamp = event.timestamp as i64;

        self.logs.append_time_unix_nano(timestamp);

        // Body: for ETW events we leave body empty (no single "message"
        // field in the general case).  Decoded fields go into attributes.
        self.logs.body.append_null();

        // Severity from ETW level
        let severity = etw_level_to_otel_severity(event.level);
        if severity > 0 {
            self.logs.append_severity_number(Some(severity));
            self.logs
                .append_severity_text(etw_level_to_severity_text(event.level).map(str::as_bytes));
        } else {
            self.logs.append_severity_number(None);
            self.logs.append_severity_text(None);
        }

        self.logs.append_id(Some(self.curr_log_id));

        // Event name: prefer the TDH event name (e.g. "AppStarted") over
        // the numeric event ID.  Fall back to "etw.<event_id>" for
        // manifest-based events where TDH doesn't provide a name.
        if event.event_name.is_empty() {
            let fallback = format!("etw.{}", event.event_id);
            self.logs.append_event_name(Some(fallback.as_bytes()));
        } else {
            self.logs
                .append_event_name(Some(event.event_name.as_bytes()));
        }

        // Attributes: ETW header metadata
        self.append_attr("etw.event_id", AttrValue::Int(i64::from(event.event_id)));
        self.append_attr("etw.opcode", AttrValue::Int(i64::from(event.opcode)));
        self.append_attr("etw.version", AttrValue::Int(i64::from(event.version)));
        self.append_attr(
            "etw.keywords",
            AttrValue::Int(event.keywords.min(i64::MAX as u64) as i64),
        );
        self.append_attr(
            "etw.process_id",
            AttrValue::Int(i64::from(event.process_id)),
        );
        self.append_attr("etw.thread_id", AttrValue::Int(i64::from(event.thread_id)));

        // Provider GUID as hex string (e.g. "d2387720-2907-5677-8625-c1bdc4155197").
        // Format into a stack buffer to avoid a per-event heap allocation.
        let provider_guid = format_guid(&event.provider_id);
        // safety: `format_guid` only writes ASCII hex digits and '-'.
        let provider_guid =
            std::str::from_utf8(&provider_guid).expect("GUID buffer is valid ASCII");
        self.append_attr(
            "etw.provider_id",
            AttrValue::Str(Cow::Borrowed(provider_guid)),
        );

        // Activity ID — only emit when non-zero (provider set a correlation ID)
        if event.activity_id != [0u8; 16] {
            let activity = format_guid(&event.activity_id);
            // safety: `format_guid` only writes ASCII hex digits and '-'.
            let activity =
                std::str::from_utf8(&activity).expect("activity id buffer is valid ASCII");
            self.append_attr("etw.activity_id", AttrValue::Str(Cow::Borrowed(activity)));
        }

        // Attributes: TDH-decoded fields
        for field in &event.decoded_fields {
            if field.name.is_empty() {
                continue;
            }
            let value = decode_field_value(&field.value);
            self.append_attr(&field.name, value);
        }

        self.curr_log_id += 1;
    }

    /// Append a single attribute key-value pair.
    fn append_attr(&mut self, key: &str, value: AttrValue<'_>) {
        self.log_attrs.append_key(key);
        match value {
            AttrValue::Str(s) => self.log_attrs.any_values_builder.append_str(s.as_bytes()),
            AttrValue::Int(i) => self.log_attrs.any_values_builder.append_int(i),
            AttrValue::Double(d) => self.log_attrs.any_values_builder.append_double(d),
            AttrValue::Bool(b) => self.log_attrs.any_values_builder.append_bool(b),
            AttrValue::Bytes(b) => {
                // Encode raw bytes as a lowercase hex string for observability.
                // Manual lookup into a single buffer avoids the per-byte
                // `format!` allocation and the intermediate `String`.
                let mut hex = Vec::with_capacity(b.len() * 2);
                for &byte in b {
                    hex.push(HEX_DIGITS[(byte >> 4) as usize]);
                    hex.push(HEX_DIGITS[(byte & 0x0f) as usize]);
                }
                self.log_attrs.any_values_builder.append_str(&hex);
            }
        }
        self.log_attrs.append_parent_id(&self.curr_log_id);
    }

    /// Builds the Arrow record batch from buffered ETW events.
    pub(super) fn build(mut self) -> Result<OtapArrowRecords> {
        let log_record_count = self.curr_log_id.into();

        // Set observed_time to the current wall-clock time at build (flush)
        // time, matching the syslog receiver pattern.  This represents when
        // the collector processed the batch, not when the events occurred.
        let observed_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        self.logs
            .append_observed_time_unix_nano_n(observed_time, log_record_count);

        // Batch-fill fields that are uniform across all records in the batch.
        self.logs.append_flags_n(None, log_record_count);
        _ = self.logs.append_trace_id_n(None, log_record_count);
        _ = self.logs.append_span_id_n(None, log_record_count);

        // All logs belong to the same resource and scope.  Fill in the
        // required row-aligned placeholder arrays.
        self.logs.resource.append_id_n(0, log_record_count);
        self.logs
            .resource
            .append_schema_url_n(None, log_record_count);
        self.logs
            .resource
            .append_dropped_attributes_count_n(0, log_record_count);

        self.logs.scope.append_id_n(0, log_record_count);
        self.logs.scope.append_name_n(None, log_record_count);
        self.logs.scope.append_version_n(None, log_record_count);
        self.logs
            .scope
            .append_dropped_attributes_count_n(0, log_record_count);

        self.logs.append_schema_url_n(None, log_record_count);
        self.logs
            .append_dropped_attributes_count_n(0, log_record_count);

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, self.logs.finish()?)?;

        let log_attrs_rb = self.log_attrs.finish()?;
        if log_attrs_rb.num_rows() > 0 {
            otap_batch.set(ArrowPayloadType::LogAttrs, log_attrs_rb)?;
        }

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::etw_receiver::session::{DecodedField, EtwAttributeValue, EtwEventData};

    fn test_event() -> EtwEventData {
        EtwEventData {
            provider_id: [0u8; 16],
            timestamp: 123456789,
            process_id: 1234,
            thread_id: 5678,
            event_id: 42,
            opcode: 0,
            version: 1,
            level: 4, // Information
            keywords: 0xFF,
            event_name: "TestEvent".to_string(),
            activity_id: [0u8; 16],
            decoded_fields: vec![
                DecodedField {
                    name: "ProcessId".to_string(),
                    value: EtwAttributeValue::Int(1234),
                },
                DecodedField {
                    name: "FileName".to_string(),
                    value: EtwAttributeValue::Str("test.exe".to_string()),
                },
            ],
        }
    }

    #[test]
    fn build_creates_logs_and_attributes_batches() {
        let mut builder = EtwArrowRecordsBuilder::new();
        builder.append(&test_event());

        let batch = builder.build().expect("build succeeds");
        let logs_rb = batch
            .get(ArrowPayloadType::Logs)
            .expect("logs batch present");
        let attrs_rb = batch
            .get(ArrowPayloadType::LogAttrs)
            .expect("attrs batch present");

        assert_eq!(logs_rb.num_rows(), 1);
        // 7 header attrs (event_id, opcode, version, keywords, process_id,
        // thread_id, provider_id) + 2 decoded fields = 9 rows.
        // (event_name is carried in the OTAP `event_name` log-record column,
        // not duplicated as an attribute; activity_id is all zeros so it's
        // omitted.)
        assert_eq!(attrs_rb.num_rows(), 9);
    }

    #[test]
    fn severity_mapping() {
        assert_eq!(etw_level_to_otel_severity(1), 21); // FATAL
        assert_eq!(etw_level_to_otel_severity(2), 17); // ERROR
        assert_eq!(etw_level_to_otel_severity(3), 13); // WARN
        assert_eq!(etw_level_to_otel_severity(4), 9); // INFO
        assert_eq!(etw_level_to_otel_severity(5), 5); // DEBUG
        assert_eq!(etw_level_to_otel_severity(0), 0); // UNSPECIFIED
        assert_eq!(etw_level_to_otel_severity(255), 0); // Unknown
    }

    #[test]
    fn empty_builder_is_empty() {
        let builder = EtwArrowRecordsBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn len_increments_on_append() {
        let mut builder = EtwArrowRecordsBuilder::new();
        builder.append(&test_event());
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());
        builder.append(&test_event());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn empty_decoded_fields_still_has_header_attrs() {
        let mut builder = EtwArrowRecordsBuilder::new();
        let mut event = test_event();
        event.decoded_fields.clear();
        builder.append(&event);

        let batch = builder.build().expect("build succeeds");
        let attrs_rb = batch
            .get(ArrowPayloadType::LogAttrs)
            .expect("attrs batch present");
        // 7 header attributes (event_id, opcode, version, keywords,
        // process_id, thread_id, provider_id); event_name is carried in the
        // OTAP `event_name` log-record column; activity_id is zero → omitted
        assert_eq!(attrs_rb.num_rows(), 7);
    }

    #[test]
    fn skip_empty_field_names() {
        let mut builder = EtwArrowRecordsBuilder::new();
        let mut event = test_event();
        event.decoded_fields = vec![DecodedField {
            name: String::new(),
            value: EtwAttributeValue::Int(42),
        }];
        builder.append(&event);

        let batch = builder.build().expect("build succeeds");
        let attrs_rb = batch
            .get(ArrowPayloadType::LogAttrs)
            .expect("attrs batch present");
        // 7 header attributes; the empty-named field is skipped
        assert_eq!(attrs_rb.num_rows(), 7);
    }
}
