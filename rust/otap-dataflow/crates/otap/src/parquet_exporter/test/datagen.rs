// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{RecordBatch, StringArray, TimestampNanosecondArray, UInt8Array, UInt16Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow_ipc::writer::StreamWriter;
use otel_arrow_rust::otlp::attributes::store::AttributeValueType;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType};
use otel_arrow_rust::schema::consts::{self, metadata};

pub struct SimpleLogDataGenOptions {
    pub id_offset: u16,
    pub num_rows: usize,

    pub with_log_attrs: bool,
    pub with_resource_attrs: bool,
    pub with_scope_attrs: bool,

    pub ids_decoded: bool,
}

impl Default for SimpleLogDataGenOptions {
    fn default() -> Self {
        Self {
            id_offset: 0,
            num_rows: 1,
            with_log_attrs: true,
            with_resource_attrs: true,
            with_scope_attrs: true,
            ids_decoded: true,
        }
    }
}

pub fn create_single_arrow_record_batch(options: SimpleLogDataGenOptions) -> BatchArrowRecords {
    let mut arrow_payloads = vec![];

    let ids_metadata = if options.ids_decoded {
        HashMap::from_iter(vec![(
            metadata::COLUMN_ENCODING.to_string(),
            metadata::encodings::PLAIN.to_string(),
        )])
    } else {
        HashMap::<String, String>::new()
    };

    let logs_schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true).with_metadata(ids_metadata.clone()),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None),
            true,
        ),
    ]));

    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_metadata(ids_metadata.clone()),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));

    let ids_array = Arc::new(UInt16Array::from_iter((0..options.num_rows).map(|i| {
        if options.ids_decoded {
            i as u16 + options.id_offset
        } else {
            1
        }
    })));

    let logs_batch = RecordBatch::try_new(
        logs_schema.clone(),
        vec![
            ids_array.clone(),
            Arc::new(TimestampNanosecondArray::from_iter_values(
                (0..options.num_rows).map(|_| 1748297321 * 1_000_000_000),
            )),
        ],
    )
    .unwrap();

    arrow_payloads.push(ArrowPayload {
        schema_id: "logs_schema_1".to_string(),
        r#type: ArrowPayloadType::Logs as i32,
        record: serialize(&logs_batch),
    });

    if options.with_log_attrs {
        let log_attrs_batch = RecordBatch::try_new(
            attr_schema.clone(),
            vec![
                ids_array.clone(),
                Arc::new(UInt8Array::from_iter_values(
                    vec![AttributeValueType::Str; options.num_rows]
                        .iter()
                        .map(|i| *i as u8),
                )),
                Arc::new(StringArray::from_iter_values(
                    (0..options.num_rows).map(|_| "log_attr"),
                )),
                Arc::new(StringArray::from_iter(
                    (0..options.num_rows).map(|_| Some("log_val")),
                )),
            ],
        )
        .unwrap();

        arrow_payloads.push(ArrowPayload {
            schema_id: "log_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::LogAttrs as i32,
            record: serialize(&log_attrs_batch),
        });
    }

    if options.with_resource_attrs {
        let resource_attrs_batch = RecordBatch::try_new(
            attr_schema.clone(),
            vec![
                ids_array.clone(),
                Arc::new(UInt8Array::from_iter_values(
                    vec![AttributeValueType::Str; options.num_rows]
                        .iter()
                        .map(|i| *i as u8),
                )),
                Arc::new(StringArray::from_iter_values(
                    (0..options.num_rows).map(|_| "resource_attr"),
                )),
                Arc::new(StringArray::from_iter(
                    (0..options.num_rows).map(|_| Some("resource_val")),
                )),
            ],
        )
        .unwrap();

        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = RecordBatch::try_new(
            attr_schema,
            vec![
                ids_array.clone(),
                Arc::new(UInt8Array::from_iter_values(
                    vec![AttributeValueType::Str; options.num_rows]
                        .iter()
                        .map(|i| *i as u8),
                )),
                Arc::new(StringArray::from_iter_values(
                    (0..options.num_rows).map(|_| "scope_attr"),
                )),
                Arc::new(StringArray::from_iter(
                    (0..options.num_rows).map(|_| Some("scope_val")),
                )),
            ],
        )
        .unwrap();

        arrow_payloads.push(ArrowPayload {
            schema_id: "scope_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ScopeAttrs as i32,
            record: serialize(&scope_attrs_batch),
        });
    }

    BatchArrowRecords {
        batch_id: 0,
        arrow_payloads,
        headers: Vec::default(),
    }
}

fn serialize(record_batch: &RecordBatch) -> Vec<u8> {
    let mut buff = Vec::<u8>::new();
    {
        let mut writer = StreamWriter::try_new(&mut buff, record_batch.schema_ref()).unwrap();
        writer.write(record_batch).unwrap();
        writer.finish().unwrap();
    };

    buff
}
