// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    ArrowPrimitiveType, FixedSizeBinaryArray, PrimitiveArray, RecordBatch, StringArray,
    TimestampNanosecondArray, UInt8Array,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit, UInt16Type, UInt32Type};
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

    pub metrics_options: Option<SimpleMetricsDataGenOptions>,
    pub traces_options: Option<SimpleTracesDataGenOptions>,
}
pub struct SimpleTracesDataGenOptions {
    pub with_span_links: bool,
    pub with_span_links_attrs: bool,
    pub with_span_events: bool,
    pub with_span_events_attrs: bool,
}

pub struct SimpleMetricsDataGenOptions {
    pub with_numbers_dp: bool,
    pub with_numbers_dp_attrs: bool,
    pub with_numbers_dp_exemplars: bool,
    pub with_numbers_dp_exemplar_attrs: bool,

    pub with_summary_dp: bool,
    pub with_summary_attrs: bool,

    pub with_histogram_dp: bool,
    pub with_histogram_dp_attrs: bool,
    pub with_histogram_dp_exemplars: bool,
    pub with_histogram_dp_exemplars_attrs: bool,

    pub with_exp_histogram_dp: bool,
    pub with_exp_histogram_dp_attrs: bool,
    pub with_exp_histogram_dp_exemplars: bool,
    pub with_exp_histogram_dp_exemplars_attrs: bool,
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
            metrics_options: None,
        }
    }
}

impl Default for SimpleTracesDataGenOptions {
    fn default() -> Self {
        Self {
            with_span_events: true,
            with_span_events_attrs: true,
            with_span_links: true,
            with_span_links_attrs: true,
        }
    }
}

impl Default for SimpleMetricsDataGenOptions {
    fn default() -> Self {
        Self {
            with_numbers_dp: true,
            with_numbers_dp_attrs: true,
            with_numbers_dp_exemplars: true,
            with_numbers_dp_exemplar_attrs: true,
            with_summary_dp: true,
            with_summary_attrs: true,
            with_histogram_dp: true,
            with_histogram_dp_attrs: true,
            with_histogram_dp_exemplars: true,
            with_histogram_dp_exemplars_attrs: true,
            with_exp_histogram_dp: true,
            with_exp_histogram_dp_attrs: true,
            with_exp_histogram_dp_exemplars: true,
            with_exp_histogram_dp_exemplars_attrs: true,
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
        let log_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "log_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::LogAttrs as i32,
            record: serialize(&log_attrs_batch),
        });
    }

    if options.with_resource_attrs {
        let resource_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
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

pub fn create_simple_trace_arrow_record_batches(
    options: SimpleDataGenOptions,
) -> BatchArrowRecords {
    let mut arrow_payloads = vec![];

    let spans_batch = create_main_record_batch(&options);
    arrow_payloads.push(ArrowPayload {
        schema_id: "spans_schema_1".to_string(),
        r#type: ArrowPayloadType::Spans as i32,
        record: serialize(&spans_batch),
    });

    if options.with_main_record_attrs {
        let log_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "span_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::SpanAttrs as i32,
            record: serialize(&log_attrs_batch),
        });
    }

    if options.with_resource_attrs {
        let resource_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "scope_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ScopeAttrs as i32,
            record: serialize(&scope_attrs_batch),
        });
    }

    if let Some(trace_options) = options.traces_options.as_ref() {
        if trace_options.with_span_events {
            let span_events_schema = Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true)
                    .with_metadata(create_ids_metadata(&options)),
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_metadata(create_ids_metadata(&options)),
                Field::new(consts::NAME, DataType::Utf8, false),
            ]));

            let span_events_batch = RecordBatch::try_new(
                span_events_schema,
                vec![
                    create_ids_array::<UInt32Type>(&options),
                    create_ids_array::<UInt16Type>(&options),
                    Arc::new(StringArray::from_iter_values(
                        (0..options.num_rows).map(|i| format!("name{}", i)),
                    )),
                ],
            )
            .unwrap();

            arrow_payloads.push(ArrowPayload {
                schema_id: "span_events_schema_1".to_string(),
                r#type: ArrowPayloadType::SpanEvents as i32,
                record: serialize(&span_events_batch),
            });

            if trace_options.with_span_events_attrs {
                let span_events_attrs = create_attributes_records_batch::<UInt32Type>(&options);
                arrow_payloads.push(ArrowPayload {
                    schema_id: "span_events_attrs_schema_1".to_string(),
                    r#type: ArrowPayloadType::SpanEventAttrs as i32,
                    record: serialize(&span_events_attrs),
                });
            }
        }

        if trace_options.with_span_links {
            let span_events_schema = Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true)
                    .with_metadata(create_ids_metadata(&options)),
                Field::new(consts::PARENT_ID, DataType::UInt16, false)
                    .with_metadata(create_ids_metadata(&options)),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ]));

            let trace_ids = FixedSizeBinaryArray::try_from_iter(
                (0..options.num_rows).map(|_| (0..16).collect::<Vec<_>>()),
            )
            .unwrap();

            let span_events_batch = RecordBatch::try_new(
                span_events_schema,
                vec![
                    create_ids_array::<UInt32Type>(&options),
                    create_ids_array::<UInt16Type>(&options),
                    Arc::new(trace_ids),
                ],
            )
            .unwrap();

            arrow_payloads.push(ArrowPayload {
                schema_id: "span_links_schema_1".to_string(),
                r#type: ArrowPayloadType::SpanLinks as i32,
                record: serialize(&span_events_batch),
            });

            if trace_options.with_span_links_attrs {
                let span_links_attrs = create_attributes_records_batch::<UInt32Type>(&options);
                arrow_payloads.push(ArrowPayload {
                    schema_id: "span_links_attrs_schema_1".to_string(),
                    r#type: ArrowPayloadType::SpanLinkAttrs as i32,
                    record: serialize(&span_links_attrs),
                });
            }
        }
    }

    BatchArrowRecords {
        batch_id: 0,
        arrow_payloads,
        headers: Vec::default(),
    }
}

pub fn create_simple_metrics_arrow_record_batches(
    options: SimpleDataGenOptions,
) -> BatchArrowRecords {
    let mut arrow_payloads = vec![];

    let metrics_batch = create_main_record_batch(&options);
    arrow_payloads.push(ArrowPayload {
        schema_id: "metrics_schema_1".to_string(),
        r#type: ArrowPayloadType::UnivariateMetrics as i32,
        record: serialize(&metrics_batch),
    });

    if options.with_resource_attrs {
        let resource_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "resource_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ResourceAttrs as i32,
            record: serialize(&resource_attrs_batch),
        });
    }

    if options.with_scope_attrs {
        let scope_attrs_batch = create_attributes_records_batch::<UInt16Type>(&options);
        arrow_payloads.push(ArrowPayload {
            schema_id: "scope_attrs_schema_1".to_string(),
            r#type: ArrowPayloadType::ScopeAttrs as i32,
            record: serialize(&scope_attrs_batch),
        });
    }

    if let Some(metrics_options) = options.metrics_options.as_ref() {
        append_payloads_for_metrics_type(
            &mut arrow_payloads,
            &options,
            metrics_options
                .with_summary_dp
                .then_some(ArrowPayloadType::SummaryDataPoints),
            metrics_options
                .with_summary_attrs
                .then_some(ArrowPayloadType::SummaryDpAttrs),
            None,
            None,
        );
    }

    if let Some(metrics_options) = options.metrics_options.as_ref() {
        append_payloads_for_metrics_type(
            &mut arrow_payloads,
            &options,
            metrics_options
                .with_numbers_dp
                .then_some(ArrowPayloadType::NumberDataPoints),
            metrics_options
                .with_numbers_dp_attrs
                .then_some(ArrowPayloadType::NumberDpAttrs),
            metrics_options
                .with_numbers_dp_exemplars
                .then_some(ArrowPayloadType::NumberDpExemplars),
            metrics_options
                .with_numbers_dp_exemplar_attrs
                .then_some(ArrowPayloadType::NumberDpExemplarAttrs),
        );
    }

    if let Some(metrics_options) = options.metrics_options.as_ref() {
        append_payloads_for_metrics_type(
            &mut arrow_payloads,
            &options,
            metrics_options
                .with_histogram_dp
                .then_some(ArrowPayloadType::HistogramDataPoints),
            metrics_options
                .with_histogram_dp_attrs
                .then_some(ArrowPayloadType::HistogramDpAttrs),
            metrics_options
                .with_histogram_dp_exemplars
                .then_some(ArrowPayloadType::HistogramDpExemplars),
            metrics_options
                .with_histogram_dp_exemplars_attrs
                .then_some(ArrowPayloadType::HistogramDpExemplarAttrs),
        );
    }

    if let Some(metrics_options) = options.metrics_options.as_ref() {
        append_payloads_for_metrics_type(
            &mut arrow_payloads,
            &options,
            metrics_options
                .with_exp_histogram_dp
                .then_some(ArrowPayloadType::ExpHistogramDataPoints),
            metrics_options
                .with_exp_histogram_dp_attrs
                .then_some(ArrowPayloadType::ExpHistogramDpAttrs),
            metrics_options
                .with_exp_histogram_dp_exemplars
                .then_some(ArrowPayloadType::ExpHistogramDpExemplars),
            metrics_options
                .with_exp_histogram_dp_exemplars_attrs
                .then_some(ArrowPayloadType::ExpHistogramDpExemplarAttrs),
        );
    }

    // Placeholder for metrics data generation logic
    BatchArrowRecords {
        batch_id: 0,
        arrow_payloads,
        headers: Vec::default(),
    }
}

fn append_payloads_for_metrics_type(
    arrow_payloads: &mut Vec<ArrowPayload>,
    options: &SimpleDataGenOptions,
    data_point_payload_type: Option<ArrowPayloadType>,
    data_point_attrs_payload_type: Option<ArrowPayloadType>,
    exemplar_payload_type: Option<ArrowPayloadType>,
    exemplar_attrs_payload_type: Option<ArrowPayloadType>,
) {
    if let Some(payload_type) = data_point_payload_type {
        arrow_payloads.push(ArrowPayload {
            schema_id: format!("{:?}_schema_1", payload_type.as_str_name().to_lowercase()),
            r#type: payload_type as i32,
            record: serialize(&create_metrics_data_point_record_batch(options)),
        })
    }

    if let Some(payload_type) = data_point_attrs_payload_type {
        arrow_payloads.push(ArrowPayload {
            schema_id: format!("{:?}_schema_1", payload_type.as_str_name().to_lowercase()),
            r#type: payload_type as i32,
            record: serialize(&create_attributes_records_batch::<UInt32Type>(options)),
        });
    }

    if let Some(payload_type) = exemplar_payload_type {
        arrow_payloads.push(ArrowPayload {
            schema_id: format!("{:?}_schema_1", payload_type.as_str_name().to_lowercase()),
            r#type: payload_type as i32,
            record: serialize(&create_exemplars_record_batch(options)),
        });
    }

    if let Some(payload_type) = exemplar_attrs_payload_type {
        arrow_payloads.push(ArrowPayload {
            schema_id: format!("{:?}_schema_1", payload_type.as_str_name().to_lowercase()),
            r#type: payload_type as i32,
            record: serialize(&create_attributes_records_batch::<UInt32Type>(options)),
        });
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

/// creates the IDs for the generic type. if options.ids_decoded is passed, it creates
/// a sequence 0..num_rows, otherwise it is all 1 (which is the same sequence with
/// delta encoding).
pub fn create_ids_array<T>(options: &SimpleDataGenOptions) -> Arc<PrimitiveArray<T>>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: From<u16>,
{
    Arc::new(PrimitiveArray::<T>::from_iter_values(
        (0..options.num_rows).map(|i| {
            let id = if options.ids_decoded {
                i as u16 + options.id_offset
            } else {
                1
            };
            <T as ArrowPrimitiveType>::Native::from(id)
        }),
    ))
}

pub fn create_timestamp_ns_array(options: &SimpleDataGenOptions) -> Arc<TimestampNanosecondArray> {
    Arc::new(TimestampNanosecondArray::from_iter_values(
        (0..options.num_rows).map(|_| 1748297321 * 1_000_000_000),
    ))
}

pub fn create_main_record_batch(options: &SimpleDataGenOptions) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true).with_metadata(create_ids_metadata(options)),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
    ]));

    RecordBatch::try_new(
        schema.clone(),
        vec![
            create_ids_array::<UInt16Type>(options),
            create_timestamp_ns_array(options),
        ],
    )
    .unwrap()
}

pub fn create_attributes_records_batch<ParentIdType>(options: &SimpleDataGenOptions) -> RecordBatch
where
    ParentIdType: ArrowPrimitiveType,
    <ParentIdType as ArrowPrimitiveType>::Native: From<u16>,
{
    let attr_schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, ParentIdType::DATA_TYPE, false)
            .with_metadata(create_ids_metadata(options)),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
    ]));

    RecordBatch::try_new(
        attr_schema.clone(),
        vec![
            create_ids_array::<ParentIdType>(options),
            Arc::new(UInt8Array::from_iter_values(
                vec![AttributeValueType::Str; options.num_rows]
                    .iter()
                    .map(|i| *i as u8),
            )),
            Arc::new(StringArray::from_iter_values(
                (0..options.num_rows).map(|_| "attr"),
            )),
            Arc::new(StringArray::from_iter(
                (0..options.num_rows).map(|_| Some("val")),
            )),
        ],
    )
    .unwrap()
}

pub fn create_metrics_data_point_record_batch(options: &SimpleDataGenOptions) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            create_ids_array::<UInt32Type>(options),
            create_ids_array::<UInt16Type>(options),
            create_timestamp_ns_array(options),
        ],
    )
    .unwrap()
}

pub fn create_exemplars_record_batch(options: &SimpleDataGenOptions) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt32, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            create_ids_array::<UInt32Type>(options),
            create_ids_array::<UInt32Type>(options),
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
