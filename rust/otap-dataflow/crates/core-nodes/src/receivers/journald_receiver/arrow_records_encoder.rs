// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP log record construction for journald entries.

use chrono::Utc;
use otap_df_pdata::{
    encode::Result,
    encode::record::{
        attributes::StrKeysAttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder,
    },
    otap::{Logs, OtapArrowRecords},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

use crate::receivers::journald_receiver::config::severity_number_from_priority;

/// One field from a journald entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JournalField {
    pub(crate) name: String,
    pub(crate) value: Vec<u8>,
}

/// A decoded journald entry ready for OTAP projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JournalEntry {
    pub(crate) cursor: String,
    /// Index into `fields` of the `MESSAGE` field used as the log body. This is
    /// the entry's FIRST `MESSAGE`, but only if it survived extraction: if that
    /// first `MESSAGE` was dropped (e.g. oversized), this stays `None` even when
    /// a later `MESSAGE` remains in `fields`. Stored as an index so the message
    /// payload is held once (in `fields`) instead of being cloned into a second
    /// buffer.
    pub(crate) message_body_index: Option<usize>,
    pub(crate) realtime_unix_nano: u64,
    pub(crate) fields: Vec<JournalField>,
    pub(crate) dropped_fields: u64,
}

impl JournalEntry {
    pub(crate) fn priority(&self) -> Option<u8> {
        self.field("PRIORITY")
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|v| v.parse::<u8>().ok())
    }

    fn field(&self, key: &str) -> Option<&[u8]> {
        self.fields
            .iter()
            .find(|field| field.name == key)
            .map(|field| field.value.as_slice())
    }

    /// Returns the log body: the value at `message_body_index` — the entry's
    /// first `MESSAGE` field if it was kept, otherwise `None` (including the
    /// case where the first `MESSAGE` was dropped but a later `MESSAGE` survives
    /// in `fields`). Reads the bytes back from `fields`, so the message payload
    /// is not duplicated.
    pub(crate) fn message_body(&self) -> Option<&[u8]> {
        self.message_body_index.and_then(|index| {
            let field = self.fields.get(index)?;
            debug_assert_eq!(
                field.name, "MESSAGE",
                "message_body_index must point at the entry's MESSAGE field; a refactor \
                 reordered or dropped fields between decode and encode"
            );
            Some(field.value.as_slice())
        })
    }
}

/// Builder for creating Arrow record batches from journald entries.
pub(crate) struct JournaldArrowRecordsBuilder {
    curr_log_id: u16,
    logs: LogsRecordBatchBuilder,
    log_attrs: StrKeysAttributesRecordBatchBuilder<u16>,
}

impl JournaldArrowRecordsBuilder {
    pub(crate) fn new() -> Self {
        Self {
            curr_log_id: 0,
            logs: LogsRecordBatchBuilder::new(),
            log_attrs: StrKeysAttributesRecordBatchBuilder::<u16>::new(),
        }
    }

    pub(crate) const fn len(&self) -> u16 {
        self.curr_log_id
    }

    pub(crate) fn append(&mut self, entry: &JournalEntry) {
        let time_unix_nano = i64::try_from(entry.realtime_unix_nano).unwrap_or(i64::MAX);
        self.logs.append_time_unix_nano(time_unix_nano);
        self.logs.append_severity_number(
            entry
                .priority()
                .and_then(severity_number_from_priority)
                .map(i32::from),
        );
        self.logs.append_severity_text(None);

        match entry.message_body() {
            Some(message) if std::str::from_utf8(message).is_ok() => {
                self.logs.body.append_str(message);
            }
            Some(message) => {
                self.logs.body.append_bytes(message);
            }
            None => self.logs.body.append_null(),
        }

        for field in &entry.fields {
            if field.name == "__CURSOR" {
                continue;
            }
            self.log_attrs.append_parent_id(&self.curr_log_id);
            self.log_attrs.append_key(&field.name);
            if let Ok(value) = std::str::from_utf8(&field.value) {
                self.log_attrs
                    .any_values_builder
                    .append_str(value.as_bytes());
            } else {
                self.log_attrs.any_values_builder.append_bytes(&field.value);
            }
        }

        self.logs.append_id(Some(self.curr_log_id));
        self.curr_log_id = self
            .curr_log_id
            .checked_add(1)
            .expect("journald batch contains more than u16::MAX log records");
    }

    pub(crate) fn build(mut self) -> Result<OtapArrowRecords> {
        let log_record_count = usize::from(self.curr_log_id);

        self.logs.resource.append_id_n(0, log_record_count);
        self.logs
            .resource
            .append_schema_url_n(None, log_record_count);
        self.logs
            .resource
            .append_dropped_attributes_count_n(0, log_record_count);

        self.logs.scope.append_id_n(0, log_record_count);
        self.logs
            .scope
            .append_name_n(Some(b"otap-df-core-nodes/journald"), log_record_count);
        self.logs
            .scope
            .append_version_n(Some(env!("CARGO_PKG_VERSION").as_bytes()), log_record_count);
        self.logs
            .scope
            .append_dropped_attributes_count_n(0, log_record_count);

        let observed_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        self.logs
            .append_observed_time_unix_nano_n(observed_time, log_record_count);
        self.logs.append_schema_url_n(None, log_record_count);
        self.logs
            .append_dropped_attributes_count_n(0, log_record_count);
        self.logs.append_flags_n(None, log_record_count);
        self.logs.append_trace_id_n(None, log_record_count)?;
        self.logs.append_span_id_n(None, log_record_count)?;

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
    use otap_df_pdata::{
        otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
        proto::opentelemetry::{
            collector::logs::v1::ExportLogsServiceRequest, common::v1::any_value::Value,
        },
    };
    use prost::Message;

    #[test]
    fn projects_message_priority_time_and_attributes() {
        let entry = JournalEntry {
            cursor: "s=1".to_owned(),
            message_body_index: Some(0),
            realtime_unix_nano: 123,
            fields: vec![
                JournalField {
                    name: "MESSAGE".to_owned(),
                    value: b"hello".to_vec(),
                },
                JournalField {
                    name: "PRIORITY".to_owned(),
                    value: b"4".to_vec(),
                },
                JournalField {
                    name: "_SYSTEMD_UNIT".to_owned(),
                    value: b"sshd.service".to_vec(),
                },
            ],
            dropped_fields: 0,
        };

        let mut builder = JournaldArrowRecordsBuilder::new();
        builder.append(&entry);
        let mut records = builder.build().unwrap();
        let mut encoder = LogsProtoBytesEncoder::new();
        let mut buffer = ProtoBuffer::default();
        encoder.encode(&mut records, &mut buffer).unwrap();
        let req = ExportLogsServiceRequest::decode(buffer.as_ref()).unwrap();
        let log = &req.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(log.time_unix_nano, 123);
        assert_eq!(log.severity_number, 13);
        assert!(matches!(
            log.body.as_ref().and_then(|v| v.value.as_ref()),
            Some(Value::StringValue(v)) if v == "hello"
        ));
        assert_eq!(log.attributes.len(), 3);
        assert!(matches!(
            log.attributes
                .iter()
                .find(|kv| kv.key == "MESSAGE")
                .and_then(|kv| kv.value.as_ref())
                .and_then(|v| v.value.as_ref()),
            Some(Value::StringValue(v)) if v == "hello"
        ));
        assert!(matches!(
            log.attributes
                .iter()
                .find(|kv| kv.key == "PRIORITY")
                .and_then(|kv| kv.value.as_ref())
                .and_then(|v| v.value.as_ref()),
            Some(Value::StringValue(v)) if v == "4"
        ));
    }

    #[test]
    fn leaves_body_unset_when_first_message_was_dropped() {
        let entry = JournalEntry {
            cursor: "s=2".to_owned(),
            message_body_index: None,
            realtime_unix_nano: 456,
            fields: vec![JournalField {
                name: "MESSAGE".to_owned(),
                value: b"surviving duplicate".to_vec(),
            }],
            dropped_fields: 1,
        };

        let mut builder = JournaldArrowRecordsBuilder::new();
        builder.append(&entry);
        let mut records = builder.build().unwrap();
        let mut encoder = LogsProtoBytesEncoder::new();
        let mut buffer = ProtoBuffer::default();
        encoder.encode(&mut records, &mut buffer).unwrap();
        let req = ExportLogsServiceRequest::decode(buffer.as_ref()).unwrap();
        let log = &req.resource_logs[0].scope_logs[0].log_records[0];
        assert!(log.body.is_none());
        assert!(matches!(
            log.attributes
                .iter()
                .find(|kv| kv.key == "MESSAGE")
                .and_then(|kv| kv.value.as_ref())
                .and_then(|v| v.value.as_ref()),
            Some(Value::StringValue(v)) if v == "surviving duplicate"
        ));
    }
}
