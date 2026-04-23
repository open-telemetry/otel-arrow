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
use otap_df_pdata::schema::{SpanId, TraceId};

use super::decoder::{DecodedAttrValue, DecodedUsereventsRecord};

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
            .append_severity_text(record.severity_text.as_deref().map(str::as_bytes));
        self.logs.append_id(Some(self.curr_log_id));
        self.logs.append_flags(record.flags);
        self.logs
            .append_event_name(record.event_name.as_deref().map(str::as_bytes));
        _ = self
            .logs
            .append_trace_id(record.trace_id.as_ref() as Option<&TraceId>);
        _ = self
            .logs
            .append_span_id(record.span_id.as_ref() as Option<&SpanId>);

        // Receiver-internal transport/diagnostic fields are intentionally not
        // represented on decoded records or emitted as downstream attributes:
        // the Ingestion backend treats OTLP log attributes as backend columns,
        // so surfacing per-record diagnostics there would pollute the
        // application schema.
        for (key, value) in record.attributes {
            self.log_attrs.append_key(key.as_ref());
            match value {
                DecodedAttrValue::Str(s) => {
                    self.log_attrs.any_values_builder.append_str(s.as_bytes())
                }
                DecodedAttrValue::Int(i) => self.log_attrs.any_values_builder.append_int(i),
                DecodedAttrValue::Bool(b) => self.log_attrs.any_values_builder.append_bool(b),
                DecodedAttrValue::Double(d) => self.log_attrs.any_values_builder.append_double(d),
            }
            self.log_attrs.append_parent_id(&self.curr_log_id);
        }

        self.curr_log_id += 1;
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
    use crate::receivers::userevents_receiver::decoder::{
        DecodedAttrValue, DecodedUsereventsRecord,
    };
    use arrow::array::{
        Array, AsArray, DictionaryArray, Int32Array, StringArray, StructArray, UInt32Array,
    };
    use arrow::datatypes::{TimestampNanosecondType, UInt8Type, UInt16Type};

    #[test]
    fn build_creates_logs_and_attributes_batches() {
        let mut builder = ArrowRecordsBuilder::new();
        builder.append(DecodedUsereventsRecord {
            time_unix_nano: 1234,
            body: "QUJD".to_owned(),
            event_name: Some("example-event".to_owned()),
            severity_number: Some(17),
            severity_text: Some("ERROR".into()),
            flags: None,
            trace_id: None,
            span_id: None,
            attributes: vec![(
                "user_name".into(),
                DecodedAttrValue::Str("example".to_owned()),
            )],
        });

        let batch = builder.build().expect("build succeeds");
        let logs_rb = batch
            .get(ArrowPayloadType::Logs)
            .expect("logs batch present");
        let attrs_rb = batch
            .get(ArrowPayloadType::LogAttrs)
            .expect("attrs batch present");

        assert_eq!(logs_rb.num_rows(), 1);
        // Only caller/application attributes are emitted downstream. Receiver
        // transport diagnostics are intentionally not encoded as log attrs.
        assert_eq!(attrs_rb.num_rows(), 1);

        let time_col = logs_rb
            .column_by_name("time_unix_nano")
            .expect("time column");
        let time_values = time_col.as_primitive::<TimestampNanosecondType>();
        assert_eq!(time_values.value(0), 1234);

        let body_col = logs_rb.column_by_name("body").expect("body column");
        let body_struct = body_col
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("body struct");
        let body_dict = body_struct
            .column_by_name("str")
            .expect("body string field")
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .expect("body string dictionary");
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
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .expect("severity dictionary");
        let severity_values = severity_col
            .values()
            .as_any()
            .downcast_ref::<Int32Array>()
            .expect("severity values");
        let severity_idx = severity_col.keys().value(0) as usize;
        assert_eq!(severity_values.value(severity_idx), 17);

        assert!(
            logs_rb.column_by_name("flags").is_none(),
            "all-null flags column should be omitted"
        );

        let parent_col = attrs_rb
            .column_by_name("parent_id")
            .expect("parent id column")
            .as_primitive::<UInt16Type>();
        for row in 0..attrs_rb.num_rows() {
            assert_eq!(parent_col.value(row), 0);
        }
    }

    #[test]
    fn build_preserves_non_null_flags() {
        let mut builder = ArrowRecordsBuilder::new();
        builder.append(DecodedUsereventsRecord {
            time_unix_nano: 1234,
            body: "text".to_owned(),
            event_name: Some("example-event".to_owned()),
            severity_number: Some(9),
            severity_text: Some("INFO".into()),
            flags: Some(1),
            trace_id: None,
            span_id: None,
            attributes: vec![],
        });

        let batch = builder.build().expect("build succeeds");
        let logs_rb = batch
            .get(ArrowPayloadType::Logs)
            .expect("logs batch present");
        let flags_col = logs_rb
            .column_by_name("flags")
            .expect("flags column")
            .as_any()
            .downcast_ref::<UInt32Array>()
            .expect("flags values");
        assert_eq!(flags_col.value(0), 1);
    }
}
