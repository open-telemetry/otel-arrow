// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Code and utilities for decoding OTAP Logs directly into proto bytes

// TODO remove this
#![allow(dead_code)]

use arrow::array::{
    Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt16Array,
};

use crate::arrays::NullableArrayAccessor;
use crate::decode::proto_bytes::resource::ResourceProtoBytesEncoder;
use crate::decode::proto_bytes::{
    RootColumnSorter, encode_field_tag, encode_fixed64, encode_len_placeholder, encode_varint,
    patch_len_placeholder, wire_types,
};
use crate::error::Result;
use crate::otap::OtapArrowRecords;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;

/// TODO comments
pub struct LogsProtoBytesEncoder {
    root_column_sorter: RootColumnSorter,
    root_columns_indices_sorted: Vec<usize>,
    root_column_sorted_index: usize,

    resource_encoder: ResourceProtoBytesEncoder,
}

impl LogsProtoBytesEncoder {
    fn new() -> Self {
        // TODO -- is there any way to estimate capacity here?
        Self {
            root_column_sorter: RootColumnSorter::new(),
            root_columns_indices_sorted: Vec::new(),
            root_column_sorted_index: 0,

            resource_encoder: ResourceProtoBytesEncoder::new(),
        }
    }

    /// TODO comments
    pub fn encode(
        &mut self,
        otap_batch: &OtapArrowRecords,
        result_buf: &mut Vec<u8>,
    ) -> Result<()> {
        // TODO -- do we need to ensure the buf is empty?

        // TODO nounwrap
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();

        // get the list of indices in the root record to visit in order
        self.root_columns_indices_sorted.clear();
        self.root_column_sorter
            .set_root_indices_to_visit_in_order(logs_rb, &mut self.root_columns_indices_sorted);
        self.root_column_sorted_index = 0;

        while self.root_column_sorted_index < logs_rb.num_rows() {
            // write the tag
            encode_field_tag(1, wire_types::LEN, result_buf);
            let len_start_pos = result_buf.len();
            encode_len_placeholder(5, result_buf);
            self.encode_resource_log(logs_rb, result_buf);
            let len = result_buf.len() - len_start_pos - 5;
            patch_len_placeholder(5, result_buf, len, len_start_pos);
        }

        Ok(())
    }

    fn encode_resource_log(&mut self, logs_rb: &RecordBatch, result_buf: &mut Vec<u8>) {
        let index = self.root_columns_indices_sorted[self.root_column_sorted_index];

        if let Some(resources_col) = logs_rb.column_by_name(consts::RESOURCE) {
            let resource_col = resources_col
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            // self.resource_encoder.encode(resource_col, index, &mut self.resource_logs_buf);
        }

        encode_field_tag(2, wire_types::LEN, result_buf);
        let len_start_pos = result_buf.len();
        encode_len_placeholder(5, result_buf);
        self.encode_scope_logs(logs_rb, result_buf);
        let len = result_buf.len() - len_start_pos - 5;
        patch_len_placeholder(5, result_buf, len, len_start_pos);
    }

    fn encode_scope_logs(&mut self, logs_rb: &RecordBatch, result_buf: &mut Vec<u8>) {
        let mut index = self.root_columns_indices_sorted[self.root_column_sorted_index];

        let mut scope_id = None;
        if let Some(scopes_col) = logs_rb.column_by_name(consts::SCOPE) {
            let scopes_col = scopes_col.as_any().downcast_ref::<StructArray>().unwrap();
            // TODO instrumentation scopes fields

            // get a reference to the scope_id we're working with
            if let Some(id_col) = scopes_col.column_by_name(consts::ID) {
                let id_col = id_col.as_any().downcast_ref::<UInt16Array>().unwrap();
                scope_id = id_col.value_at(index)
            }
        }

        while self.root_column_sorted_index < logs_rb.num_rows() {
            encode_field_tag(2, wire_types::LEN, result_buf);
            let len_start_pos = result_buf.len();
            encode_len_placeholder(5, result_buf);
            self.encode_log_record(logs_rb, result_buf);
            let len = result_buf.len() - len_start_pos - 5;
            patch_len_placeholder(5, result_buf, len, len_start_pos);
            self.root_column_sorted_index += 1;

            // TODO check if scope IDs match, then break
        }
    }

    fn encode_log_record(&mut self, logs_rb: &RecordBatch, result_buf: &mut Vec<u8>) {
        let index = self.root_columns_indices_sorted[self.root_column_sorted_index];

        if let Some(time_unix_nano_col) = logs_rb.column_by_name(consts::TIME_UNIX_NANO) {
            let time_unix_nano_col = time_unix_nano_col
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            if time_unix_nano_col.is_valid(index) {
                let val = time_unix_nano_col.value(index);
                encode_field_tag(1, wire_types::FIXED64, result_buf);
                encode_fixed64(val as u64, result_buf);
            }
        }

        if let Some(severity_text_col) = logs_rb.column_by_name(consts::SEVERITY_TEXT) {
            let severity_text_col = severity_text_col
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            if severity_text_col.is_valid(index) {
                let val = severity_text_col.value(index);
                encode_field_tag(3, wire_types::LEN, result_buf);
                encode_varint(val.len() as u64, result_buf);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
    use prost::Message;
    use std::sync::Arc;

    use crate::{otap::Logs, proto::opentelemetry::logs::v1::LogsData};

    use super::*;

    #[test]
    fn albert_smoke_test() {
        let struct_fields = Fields::from(vec![Field::new(consts::ID, DataType::UInt16, true)]);

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 0, 0]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 0, 0]))],
                    None,
                )),
                Arc::new(TimestampNanosecondArray::from_iter_values([1, 2, 3])),
                Arc::new(StringArray::from_iter_values(vec![
                    "ERROR", "INFO", "DEBUG",
                ])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, record_batch);
        let mut result_buf = vec![];
        let mut encoder = LogsProtoBytesEncoder::new();
        encoder.encode(&otap_batch, &mut result_buf).unwrap();

        println!("{:?}", result_buf);
        let result = LogsData::decode(result_buf.as_ref()).unwrap();

        println!("{:#?}", result);
    }
}
