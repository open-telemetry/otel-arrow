// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for producing BatchArrowRecords from OTAP Batches.
//!
//! `BatchArrowRecords` is the protobuf type that contains the Arrow IPC serialized messages.

// TODO we will eventually use this code as part of the implementation of PDATA for OTAP pipelines
// https://github.com/open-telemetry/otel-arrow/issues/728
#![allow(dead_code)]

use std::{collections::HashMap, io::Cursor};

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow::ipc::writer::StreamWriter;
use snafu::ResultExt;

use crate::error::{self, Result};
use crate::otap::OtapBatch;
use crate::otap::schema::SchemaIdBuilder;
use crate::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType, BatchArrowRecords};

/// handles serializing the stream of record batches for some payload type
struct StreamProducer {
    payload_type: ArrowPayloadType,
    stream_writer: StreamWriter<Cursor<Vec<u8>>>,
    schema: SchemaRef,
    schema_id: i64,
}

impl StreamProducer {
    fn try_new(payload_type: ArrowPayloadType, schema: SchemaRef, schema_id: i64) -> Result<Self> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let stream_writer =
            StreamWriter::try_new(cursor, &schema).context(error::BuildStreamWriterSnafu)?;

        Ok(Self {
            payload_type,
            stream_writer,
            schema,
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

/// Produces OTAP `BatchArrowRecords` from OTAP Batches
struct Producer {
    next_batch_id: i64,
    next_schema_id: i64,
    stream_producers: HashMap<String, StreamProducer>,
    schema_id_builder: SchemaIdBuilder,
}

impl Producer {
    /// create a new instance of `Producer`
    pub fn new() -> Self {
        Self {
            next_batch_id: 0,
            next_schema_id: 0,
            stream_producers: HashMap::new(),
            schema_id_builder: SchemaIdBuilder::new(),
        }
    }

    /// produce `BatchArrowRecords` protobuf message from `OtapBatch`
    pub fn produce_bar(&mut self, otap_batch: &OtapBatch) -> Result<BatchArrowRecords> {
        let allowed_payloads = otap_batch.allowed_payload_types();
        let mut arrow_payloads = Vec::<ArrowPayload>::with_capacity(allowed_payloads.len());

        for payload_type in allowed_payloads {
            let record_batch = match otap_batch.get(*payload_type) {
                Some(rb) => rb,
                None => continue,
            };
            let schema = record_batch.schema();

            let schema_id = self.schema_id_builder.build_id(&schema);
            let stream_producer = match self.stream_producers.get_mut(schema_id) {
                None => {
                    // cleanup previous stream producer if any that have the same ArrowPayloadType.
                    // The reasoning is that if we have a new schema ID (i.e. schema change) we
                    // should no longer use the previous stream producer for this PayloadType as
                    // schema changes are only additive.
                    self.stream_producers
                        .retain(|_, v| v.payload_type != *payload_type);
                    let payload_schema_id = self.next_schema_id;
                    self.next_schema_id += 1;
                    self.stream_producers
                        .entry(schema_id.to_string())
                        .or_insert(StreamProducer::try_new(
                            *payload_type,
                            schema,
                            payload_schema_id,
                        )?)
                }
                Some(s) => s,
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

#[cfg(test)]
mod test {
    use crate::Consumer;
    use crate::otap::{Logs, from_record_messages};

    use super::*;
    use std::sync::Arc;

    use arrow::array::{StringArray, UInt16Array, UInt32Array};
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

        let mut input = OtapBatch::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&input).unwrap();
        let result = OtapBatch::Logs(from_record_messages(
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

        let mut input = OtapBatch::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&input).unwrap();
        let result = OtapBatch::Logs(from_record_messages(
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

        let mut input = OtapBatch::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&input).unwrap();
        let result = OtapBatch::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        assert_eq!(input, result);
    }
}
