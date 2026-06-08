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

        // Duplicate the event name as an attribute so it is preserved in
        // exporters whose schema does not include the OTel `event_name`
        // log record field (e.g. the Parquet exporter writes only the
        // standard OTAP columns and omits `event_name`).
        if !event.event_name.is_empty() {
            self.append_attr(
                "etw.event_name",
                AttrValue::Str(Cow::Borrowed(&event.event_name)),
            );
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

        // Provider GUID as hex string (e.g. "d2387720-2907-5677-8625-c1bdc4155197")
        let guid = &event.provider_id;
        let provider_guid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            guid[0],
            guid[1],
            guid[2],
            guid[3],
            guid[4],
            guid[5],
            guid[6],
            guid[7],
            guid[8],
            guid[9],
            guid[10],
            guid[11],
            guid[12],
            guid[13],
            guid[14],
            guid[15],
        );
        self.append_attr("etw.provider_id", AttrValue::Str(Cow::Owned(provider_guid)));

        // Activity ID — only emit when non-zero (provider set a correlation ID)
        if event.activity_id != [0u8; 16] {
            let aid = &event.activity_id;
            let activity = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                aid[0],
                aid[1],
                aid[2],
                aid[3],
                aid[4],
                aid[5],
                aid[6],
                aid[7],
                aid[8],
                aid[9],
                aid[10],
                aid[11],
                aid[12],
                aid[13],
                aid[14],
                aid[15],
            );
            self.append_attr("etw.activity_id", AttrValue::Str(Cow::Owned(activity)));
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
                // Encode raw bytes as hex string for observability
                let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
                self.log_attrs.any_values_builder.append_str(hex.as_bytes());
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
            user_data: vec![0u8; 16],
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
        // 8 header attrs (event_name, event_id, opcode, version, keywords,
        // process_id, thread_id, provider_id) + 2 decoded fields = 10 rows.
        // (activity_id is all zeros so it's omitted.)
        assert_eq!(attrs_rb.num_rows(), 10);
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
        // 8 header attributes (event_name, event_id, opcode, version,
        // keywords, process_id, thread_id, provider_id);
        // activity_id is zero → omitted
        assert_eq!(attrs_rb.num_rows(), 8);
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
        // 8 header attributes; the empty-named field is skipped
        assert_eq!(attrs_rb.num_rows(), 8);
    }
}
