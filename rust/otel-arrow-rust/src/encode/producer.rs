// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for producing BatchArrowRecords from OTAP Batches.
//!
//! `BatchArrowRecords` is the protobuf type that contains the Arrow IPC serialized messages.

use std::io::Cursor;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow::ipc::writer::StreamWriter;
use arrow_ipc::CompressionType;
use arrow_ipc::writer::{DictionaryHandling, IpcWriteOptions};
use snafu::ResultExt;

use crate::error::{self, Result};
use crate::otap::OtapArrowRecords;
use crate::otap::schema::SchemaIdBuilder;
use crate::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType, BatchArrowRecords};

/// handles serializing the stream of record batches for some payload type
struct StreamProducer {
    stream_writer: StreamWriter<Cursor<Vec<u8>>>,
    schema_id: i64,
}

impl StreamProducer {
    fn try_new(
        schema: SchemaRef,
        schema_id: i64,
        ipc_write_options: IpcWriteOptions,
    ) -> Result<Self> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let stream_writer = StreamWriter::try_new_with_options(cursor, &schema, ipc_write_options)
            .context(error::BuildStreamWriterSnafu)?;

        Ok(Self {
            stream_writer,
            schema_id,
        })
    }

    fn serialize_batch(&mut self, record_batch: &RecordBatch) -> Result<Vec<u8>> {
        self.stream_writer
            .write(record_batch)
            .context(error::WriteRecordBatchSnafu)?;
        let cursor = self.stream_writer.get_mut();
        let pos = cursor.position() as usize;
        let result = cursor.get_ref()[..pos].to_vec();
        cursor.set_position(0);
        Ok(result)
    }
}

const PAYLOAD_TYPE_COUNT: usize = 28; // 28 variants total

// Compile-time lookup table for O(1) conversion
const PAYLOAD_TYPE_TO_INDEX: [Option<u8>; 46] = {
    let mut table = [None; 46];
    table[0] = Some(0); // Unknown
    table[1] = Some(1); // ResourceAttrs
    table[2] = Some(2); // ScopeAttrs
    table[10] = Some(3); // UnivariateMetrics
    table[11] = Some(4); // NumberDataPoints
    table[12] = Some(5); // SummaryDataPoints
    table[13] = Some(6); // HistogramDataPoints
    table[14] = Some(7); // ExpHistogramDataPoints
    table[15] = Some(8); // NumberDpAttrs
    table[16] = Some(9); // SummaryDpAttrs
    table[17] = Some(10); // HistogramDpAttrs
    table[18] = Some(11); // ExpHistogramDpAttrs
    table[19] = Some(12); // NumberDpExemplars
    table[20] = Some(13); // HistogramDpExemplars
    table[21] = Some(14); // ExpHistogramDpExemplars
    table[22] = Some(15); // NumberDpExemplarAttrs
    table[23] = Some(16); // HistogramDpExemplarAttrs
    table[24] = Some(17); // ExpHistogramDpExemplarAttrs
    table[25] = Some(18); // MultivariateMetrics
    table[26] = Some(19); // MetricAttrs
    table[30] = Some(20); // Logs
    table[31] = Some(21); // LogAttrs
    table[40] = Some(22); // Spans
    table[41] = Some(23); // SpanAttrs
    table[42] = Some(24); // SpanEvents
    table[43] = Some(25); // SpanLinks
    table[44] = Some(26); // SpanEventAttrs
    table[45] = Some(27); // SpanLinkAttrs
    table
};

impl ArrowPayloadType {
    #[inline]
    const fn to_index(self) -> usize {
        // Safety: Generated enum values are always mapped in the `PAYLOAD_TYPE_TO_INDEX` table.
        PAYLOAD_TYPE_TO_INDEX[self as usize]
            .expect("`PAYLOAD_TYPE_TO_INDEX` should cover all ArrowPayloadType variants")
            as usize
    }
}

/// Storage for a stream producer with its schema ID
struct ProducerEntry {
    schema_id: String,
    producer: StreamProducer,
}

/// Produces OTAP `BatchArrowRecords` from OTAP Batches
pub struct Producer {
    next_batch_id: i64,
    next_schema_id: i64,
    stream_producers: [Option<ProducerEntry>; PAYLOAD_TYPE_COUNT],
    schema_id_builder: SchemaIdBuilder,
    ipc_write_options: IpcWriteOptions,
}

/// Options for creating [`Producer`]
pub struct ProducerOptions {
    /// compression method for IPC batches. default = zstd
    pub ipc_compression: Option<CompressionType>,
}

impl Default for ProducerOptions {
    fn default() -> Self {
        Self {
            ipc_compression: Some(CompressionType::ZSTD),
        }
    }
}

impl Producer {
    /// create a new instance of `Producer`
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_options(ProducerOptions::default())
    }

    /// create a new instance of `Producer` with the given options for the IPC stream writer
    #[must_use]
    pub fn new_with_options(options: ProducerOptions) -> Self {
        Self {
            next_batch_id: 0,
            next_schema_id: 0,
            stream_producers: [const { None }; PAYLOAD_TYPE_COUNT],
            schema_id_builder: SchemaIdBuilder::new(),
            ipc_write_options: IpcWriteOptions::default()
                .with_dictionary_handling(DictionaryHandling::Delta)
                // safety: this will only fail if the writer options's metadata version has been
                // configured to version before V5, which we're not doing here.
                .try_with_compression(options.ipc_compression)
                .expect("can configure compression"),
        }
    }

    /// produce `BatchArrowRecords` protobuf message from `OtapBatch`
    pub fn produce_bar(&mut self, otap_batch: &mut OtapArrowRecords) -> Result<BatchArrowRecords> {
        // apply transport optimized encoding so we can get better compression ratio when
        // transmitting the serialized data
        otap_batch.encode_transport_optimized()?;

        let allowed_payloads = otap_batch.allowed_payload_types();
        let mut arrow_payloads = Vec::<ArrowPayload>::with_capacity(allowed_payloads.len());

        for payload_type in allowed_payloads {
            let record_batch = match otap_batch.get(*payload_type) {
                Some(rb) => rb,
                None => continue,
            };

            let schema = record_batch.schema();
            let schema_id = self.schema_id_builder.build_id(&schema);
            let idx = payload_type.to_index();
            let stream_producer = match &mut self.stream_producers[idx] {
                Some(entry) if entry.schema_id == schema_id => {
                    // Schema hasn't changed, use existing producer
                    &mut entry.producer
                }
                _ => {
                    // Schema changed or no producer yet, create new one
                    let payload_schema_id = self.next_schema_id;
                    self.next_schema_id += 1;

                    let new_producer = StreamProducer::try_new(
                        schema,
                        payload_schema_id,
                        self.ipc_write_options.clone(),
                    )?;

                    // cleanup previous stream producer if any that have the same ArrowPayloadType.
                    // The reasoning is that if we have a new schema ID (i.e. schema change) we
                    // should no longer use the previous stream producer for this PayloadType as
                    // schema changes are only additive.
                    self.stream_producers[idx] = Some(ProducerEntry {
                        schema_id: schema_id.to_string(),
                        producer: new_producer,
                    });

                    &mut self.stream_producers[idx]
                        .as_mut()
                        .expect("ProducerEntry should exist")
                        .producer
                }
            };

            let serialized_rb = stream_producer.serialize_batch(record_batch)?;
            arrow_payloads.push(ArrowPayload {
                schema_id: format!("{}", stream_producer.schema_id),
                r#type: *payload_type as i32,
                record: serialized_rb,
            });
        }

        // increment next batch id
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        Ok(BatchArrowRecords {
            batch_id,
            arrow_payloads,
            ..Default::default()
        })
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::Consumer;
    use crate::otap::{Logs, from_record_messages};
    use crate::otlp::attributes::AttributeValueType;
    use crate::schema::{FieldExt, consts};

    use super::*;
    use std::sync::Arc;

    use arrow::array::{StringArray, UInt8Array, UInt16Array, UInt32Array};
    use arrow::datatypes::{DataType, Field, Schema};

    #[test]
    fn test_round_trip_batch_arrow_records() {
        let mut producer = Producer::new();
        let mut consumer = Consumer::default();

        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::UInt32, false),
            Field::new("b", DataType::UInt16, false),
            Field::new("c", DataType::Utf8, true),
        ]));
        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(UInt16Array::from_iter_values(vec![4, 5, 6])),
                Arc::new(StringArray::from_iter(vec![
                    Some("terry"),
                    None,
                    Some("john"),
                ])),
            ],
        )
        .unwrap();

        let mut input = OtapArrowRecords::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&mut input).unwrap();
        let result = OtapArrowRecords::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        assert_eq!(input, result);

        // write a second batch with the same schema to test that we can reuse the same
        // StreamProducer & StreamConsumer
        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![5, 6, 7])),
                Arc::new(UInt16Array::from_iter_values(vec![9, 7, 8])),
                Arc::new(StringArray::from_iter(vec![
                    Some("fox"),
                    None,
                    Some("frog"),
                ])),
            ],
        )
        .unwrap();

        let mut input = OtapArrowRecords::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&mut input).unwrap();
        let result = OtapArrowRecords::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        assert_eq!(input, result);

        // send another batch with a different schema, but same payload type
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::UInt32, false),
            Field::new("b", DataType::UInt16, false),
        ]));
        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![5, 6, 7])),
                Arc::new(UInt16Array::from_iter_values(vec![9, 7, 8])),
            ],
        )
        .unwrap();

        let mut input = OtapArrowRecords::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&mut input).unwrap();
        let result = OtapArrowRecords::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        assert_eq!(input, result);
    }

    #[test]
    fn test_it_encodes_batches_with_transport_optimization() {
        let data = [("a", 0), ("b", 0), ("a", 1), ("b", 1), ("a", 2), ("b", 2)];
        let schema = Arc::new(Schema::new(vec![
            // because the IDs have a plain encoding, this signals to the otap batch that it will
            // need to be transport_encoded
            Field::new(consts::PARENT_ID, DataType::UInt16, false).with_plain_encoding(),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));

        let attr_types = Arc::new(UInt8Array::from_iter_values(std::iter::repeat_n(
            AttributeValueType::Str as u8,
            data.len(),
        )));
        let attr_keys = Arc::new(StringArray::from_iter_values(std::iter::repeat_n(
            "key",
            data.len(),
        )));
        let attr_vals = Arc::new(StringArray::from_iter_values(data.iter().map(|d| d.0)));
        let parent_id = Arc::new(UInt16Array::from_iter_values(data.iter().map(|d| d.1)));

        let log_attrs = RecordBatch::try_new(
            schema,
            vec![parent_id, attr_types.clone(), attr_keys.clone(), attr_vals],
        )
        .unwrap();

        let mut input = OtapArrowRecords::Logs(Logs::default());
        input.set(ArrowPayloadType::LogAttrs, log_attrs);

        let mut producer = Producer::new();
        let mut bar = producer.produce_bar(&mut input).unwrap();

        let mut consumer = Consumer::default();
        let result = OtapArrowRecords::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        let result_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();

        let expected_data = [("a", 0), ("a", 1), ("a", 1), ("b", 0), ("b", 1), ("b", 1)];
        let attr_vals = Arc::new(StringArray::from_iter_values(
            expected_data.iter().map(|d| d.0),
        ));
        let parent_id = Arc::new(UInt16Array::from_iter_values(
            expected_data.iter().map(|d| d.1),
        ));
        let expected_schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, false)
                .with_encoding(consts::metadata::encodings::QUASI_DELTA),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));

        let expected_rb = RecordBatch::try_new(
            expected_schema,
            vec![parent_id, attr_types, attr_keys, attr_vals],
        )
        .unwrap();

        assert_eq!(result_attrs, &expected_rb);
    }

    #[test]
    fn test_all_arrow_payload_types_have_valid_index() {
        // This function will fail to compile if new variants are added to ArrowPayloadType
        // without being included in this match statement
        // Anytime a new variant is added, it must be added to the `PAYLOAD_TYPE_TO_INDEX` table
        const fn exhaustive_check(payload_type: ArrowPayloadType) -> usize {
            // The match must be exhaustive - compiler will error if any variant is missing
            match payload_type {
                ArrowPayloadType::Unknown => 0,
                ArrowPayloadType::ResourceAttrs => 1,
                ArrowPayloadType::ScopeAttrs => 2,
                ArrowPayloadType::UnivariateMetrics => 3,
                ArrowPayloadType::NumberDataPoints => 4,
                ArrowPayloadType::SummaryDataPoints => 5,
                ArrowPayloadType::HistogramDataPoints => 6,
                ArrowPayloadType::ExpHistogramDataPoints => 7,
                ArrowPayloadType::NumberDpAttrs => 8,
                ArrowPayloadType::SummaryDpAttrs => 9,
                ArrowPayloadType::HistogramDpAttrs => 10,
                ArrowPayloadType::ExpHistogramDpAttrs => 11,
                ArrowPayloadType::NumberDpExemplars => 12,
                ArrowPayloadType::HistogramDpExemplars => 13,
                ArrowPayloadType::ExpHistogramDpExemplars => 14,
                ArrowPayloadType::NumberDpExemplarAttrs => 15,
                ArrowPayloadType::HistogramDpExemplarAttrs => 16,
                ArrowPayloadType::ExpHistogramDpExemplarAttrs => 17,
                ArrowPayloadType::MultivariateMetrics => 18,
                ArrowPayloadType::MetricAttrs => 19,
                ArrowPayloadType::Logs => 20,
                ArrowPayloadType::LogAttrs => 21,
                ArrowPayloadType::Spans => 22,
                ArrowPayloadType::SpanAttrs => 23,
                ArrowPayloadType::SpanEvents => 24,
                ArrowPayloadType::SpanLinks => 25,
                ArrowPayloadType::SpanEventAttrs => 26,
                ArrowPayloadType::SpanLinkAttrs => 27,
                // No wildcard pattern - compiler will error if new variants are added
            }
        }

        // Test each variant
        for variant in [
            ArrowPayloadType::Unknown,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            ArrowPayloadType::MultivariateMetrics,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ] {
            let expected_index = exhaustive_check(variant);
            let actual_index = variant.to_index();

            assert_eq!(
                actual_index, expected_index,
                "ArrowPayloadType::{:?} index mismatch",
                variant
            );

            // Verify index is within bounds
            assert!(
                actual_index < PAYLOAD_TYPE_COUNT,
                "Index {} for ArrowPayloadType::{:?} exceeds PAYLOAD_TYPE_COUNT",
                actual_index,
                variant
            );
        }
    }
}
