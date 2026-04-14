// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Arrow encoding for Linux userevents logs.

use chrono::Utc;
use otap_df_pdata::encode::Result;
use otap_df_pdata::encode::record::{
    attributes::StrKeysAttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder,
};
use otap_df_pdata::otap::{Logs, OtapArrowRecords};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::decoder::DecodedUsereventsRecord;

const ATTR_TRACEPOINT: &str = "linux.userevents.tracepoint";
const ATTR_CPU: &str = "linux.userevents.cpu";
const ATTR_PID: &str = "process.pid";
const ATTR_TID: &str = "thread.id";
const ATTR_SAMPLE_ID: &str = "linux.userevents.sample_id";
const ATTR_PAYLOAD_SIZE: &str = "linux.userevents.payload_size";
const ATTR_ENCODING: &str = "linux.userevents.body_encoding";
const BODY_ENCODING_BASE64: &str = "base64";

/// Builder for creating Arrow record batches from decoded userevents messages.
pub(super) struct ArrowRecordsBuilder {
    curr_log_id: u16,
    logs: LogsRecordBatchBuilder,
    log_attrs: StrKeysAttributesRecordBatchBuilder<u16>,
}

impl Default for ArrowRecordsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrowRecordsBuilder {
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

    /// Appends a decoded userevents record.
    pub(super) fn append(&mut self, record: DecodedUsereventsRecord) {
        self.logs.append_time_unix_nano(record.time_unix_nano);
        self.logs.body.append_str(record.body.as_bytes());
        self.logs.append_severity_number(record.severity_number);
        self.logs
            .append_severity_text(record.severity_text.map(str::as_bytes));
        self.logs.append_id(Some(self.curr_log_id));

        self.append_attr(ATTR_TRACEPOINT, &record.tracepoint);
        self.append_attr(ATTR_CPU, &record.cpu.to_string());
        self.append_attr(ATTR_PID, &record.pid.to_string());
        self.append_attr(ATTR_TID, &record.tid.to_string());
        self.append_attr(ATTR_SAMPLE_ID, &record.sample_id.to_string());
        self.append_attr(ATTR_PAYLOAD_SIZE, &record.payload_size.to_string());
        self.append_attr(ATTR_ENCODING, BODY_ENCODING_BASE64);
        for (key, value) in record.attributes {
            self.append_attr(&key, &value);
        }

        self.curr_log_id += 1;
    }

    fn append_attr(&mut self, key: &str, value: &str) {
        self.log_attrs.append_key(key);
        self.log_attrs
            .any_values_builder
            .append_str(value.as_bytes());
        self.log_attrs.append_parent_id(&self.curr_log_id);
    }

    /// Builds the Arrow records from the buffered userevents logs.
    pub(super) fn build(mut self) -> Result<OtapArrowRecords> {
        let log_record_count = self.curr_log_id.into();

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

        let observed_time = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        self.logs
            .append_observed_time_unix_nano_n(observed_time, log_record_count);
        self.logs.append_schema_url_n(None, log_record_count);
        self.logs
            .append_dropped_attributes_count_n(0, log_record_count);
        self.logs.append_flags_n(None, log_record_count);
        _ = self.logs.append_trace_id_n(None, log_record_count);
        _ = self.logs.append_span_id_n(None, log_record_count);

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
    use crate::receivers::userevents_receiver::decoder::DecodedUsereventsRecord;
    use arrow::array::{AsArray, DictionaryArray, StringArray, UInt8Array, UInt16Array};

    #[test]
    fn build_creates_logs_and_attributes_batches() {
        let mut builder = ArrowRecordsBuilder::new();
        builder.append(DecodedUsereventsRecord {
            tracepoint: "user_events:Example".to_owned(),
            time_unix_nano: 1234,
            cpu: 2,
            pid: 11,
            tid: 12,
            sample_id: 77,
            body: "QUJD".to_owned(),
            event_name: Some("example-event".to_owned()),
            payload_size: 3,
            severity_number: Some(17),
            severity_text: Some("ERROR"),
            attributes: vec![
                ("event.provider".to_owned(), "example".to_owned()),
                ("event.name".to_owned(), "example-event".to_owned()),
            ],
        });

        let batch = builder.build().expect("build succeeds");
        let logs_rb = batch
            .get(ArrowPayloadType::Logs)
            .expect("logs batch present");
        let attrs_rb = batch
            .get(ArrowPayloadType::LogAttrs)
            .expect("attrs batch present");

        assert_eq!(logs_rb.num_rows(), 1);
        assert_eq!(attrs_rb.num_rows(), 9);

        let time_col = logs_rb
            .column_by_name("time_unix_nano")
            .expect("time column");
        let time_values = time_col.as_primitive::<arrow::datatypes::Int64Type>();
        assert_eq!(time_values.value(0), 1234);

        let body_col = logs_rb.column_by_name("body").expect("body column");
        let body_dict = body_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Array>>()
            .expect("body dictionary");
        let body_values = body_dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("body string values");
        let body_idx = body_dict.keys().value(0) as usize;
        assert_eq!(body_values.value(body_idx), "QUJD");

        let severity_col = logs_rb
            .column_by_name("severity_number")
            .expect("severity number column")
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Array>>()
            .expect("severity dictionary");
        let severity_values = severity_col
            .values()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("severity values");
        let severity_idx = severity_col.keys().value(0) as usize;
        assert_eq!(severity_values.value(severity_idx), 17);

        let parent_col = attrs_rb
            .column_by_name("parent_id")
            .expect("parent id column")
            .as_primitive::<arrow::datatypes::UInt16Type>();
        for row in 0..attrs_rb.num_rows() {
            assert_eq!(parent_col.value(row), 0);
        }
    }
}
