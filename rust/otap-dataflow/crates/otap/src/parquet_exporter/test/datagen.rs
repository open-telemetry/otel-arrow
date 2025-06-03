// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{ArrowPrimitiveType, NativeAdapter, PrimitiveArray, RecordBatch, StringArray, TimestampNanosecondArray, UInt16Array, UInt8Array};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use arrow_ipc::writer::StreamWriter;
use otel_arrow_rust::otlp::attributes::store::AttributeValueType;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType};
use otel_arrow_rust::schema::consts::{self, metadata};

pub struct SimpleDataGenOptions {
    pub id_offset: u16,
    pub num_rows: usize,

    pub with_main_record_attrs: bool,
    pub with_resource_attrs: bool,
    pub with_scope_attrs: bool,

    pub ids_decoded: bool,

    pub traces_options: Option<SimpleTracesDataGenOptions>,
}
pub struct SimpleTracesDataGenOptions {
    pub with_span_links: bool,
    pub with_span_links_attrs: bool,
    pub with_span_events: bool,
    pub with_span_events_attrs: bool,
}

impl Default for SimpleDataGenOptions {
    fn default() -> Self {
        Self {
            id_offset: 0,
            num_rows: 1,
            with_main_record_attrs: true,
            with_resource_attrs: true,
            with_scope_attrs: true,
            ids_decoded: true,
            traces_options: None,
        }
    }
}

impl Default for SimpleTracesDataGenOptions {
    fn default() -> Self {
        Self {
            with_span_events: true,
            with_span_events_attrs: true,
            with_span_links: true,
            with_span_links_attrs: true
        }
    }
}



pub fn create_simple_logs_arrow_record_batches(options: SimpleDataGenOptions) -> BatchArrowRecords {
    let mut arrow_payloads = vec![];

    let logs_batch = create_main_record_batch(&options);
    arrow_payloads.push(ArrowPayload {
        schema_id: "logs_schema_1".to_string(),
        r#type: ArrowPayloadType::Logs as i32,
        record: serialize(&logs_batch),
    });

    if options.with_main_record_attrs {
        let log_attrs_batch = create_attributes_records_batch(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "log_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::LogAttrs as i32,
            record: serialize(&log_attrs_batch),
        });
    }

    if options.with_resource_attrs {
        let resource_attrs_batch = create_attributes_records_batch(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = create_attributes_records_batch(&options);
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

pub fn create_simple_trace_arrow_record_batches(options: SimpleDataGenOptions) -> BatchArrowRecords {
    let mut arrow_payloads = vec![];

    let spans_batch = create_main_record_batch(&options);
    arrow_payloads.push(ArrowPayload {
        schema_id: "spans_schema_1".to_string(),
        r#type: ArrowPayloadType::Spans as i32,
        record: serialize(&spans_batch),
    });

    if options.with_main_record_attrs {
        let log_attrs_batch = create_attributes_records_batch(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "log_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::SpanAttrs as i32,
            record: serialize(&log_attrs_batch),
        });
    }

    if options.with_resource_attrs {
        let resource_attrs_batch = create_attributes_records_batch(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = create_attributes_records_batch(&options);
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

pub fn create_ids_metadata(options: &SimpleDataGenOptions) -> HashMap<String, String> {
    if options.ids_decoded {
        HashMap::from_iter(vec![(
            metadata::COLUMN_ENCODING.to_string(),
            metadata::encodings::PLAIN.to_string(),
        )])
    } else {
        HashMap::<String, String>::new()
    }
}

pub fn create_ids_array<T>(options: &SimpleDataGenOptions) -> Arc<PrimitiveArray<T>>
where T: ArrowPrimitiveType,
    NativeAdapter<T>: From<u16> {
    Arc::new(PrimitiveArray::<T>::from_iter((0..options.num_rows).map(|i| {
        if options.ids_decoded {
            i as u16 + options.id_offset
        } else {
            1
        }
    })))
}

pub fn create_main_record_batch(options: &SimpleDataGenOptions) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true).with_metadata(create_ids_metadata(options)),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None),
            true,
        ),
    ]));

    RecordBatch::try_new(
        schema.clone(),
        vec![
            create_ids_array::<UInt16Type>(options),
            Arc::new(TimestampNanosecondArray::from_iter_values(
                (0..options.num_rows).map(|_| 1748297321 * 1_000_000_000),
            )),
        ],
    )
    .unwrap()
}


pub fn create_attributes_records_batch<T>(options: &SimpleDataGenOptions) -> RecordBatch
where 
    T: ArrowPrimitiveType,
    NativeAdapter<T>: From<u16> 
{
    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false).with_metadata(create_ids_metadata(&options)),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));

    RecordBatch::try_new(
        attr_schema.clone(),
        vec![
            create_ids_array::<T>(&options),
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
    .unwrap()
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
