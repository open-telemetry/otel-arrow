// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for producing BatchArrowRecords from OTAP Batches.
//!
//! `BatchArrowRecords` is the protobuf type that contains the Arrow IPC serialized messages.

use std::{collections::HashMap, io::Cursor};

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow::ipc::writer::StreamWriter;
use prost::bytes::Buf;
use snafu::ResultExt;

use crate::error::{self, Result};
use crate::otap::OtapBatch;
use crate::otap::schema::schema_id;
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
        self.stream_writer.write(record_batch).unwrap(); // TODO no unwrap
        let cursor = self.stream_writer.get_mut();
        let mut results = Vec::with_capacity(cursor.position() as usize);
        cursor.copy_to_slice(&mut results);
        cursor.set_position(0);
        Ok(results)
    }
}

/// Produces OTAP `BatchArrowRecords` from OTAP Batches
#[derive(Default)]
struct Producer {
    next_batch_id: i64,
    next_schema_id: i64,
    stream_producers: HashMap<String, StreamProducer>,
}

impl Producer {
    /// produce `BatchArrowRecords` protobuf message from `OtapBatch`
    pub fn produce_bar(&mut self, otap_batch: OtapBatch) -> Result<BatchArrowRecords> {
        let allowed_payloads = otap_batch.allowed_payload_types();
        let mut arrow_payloads = Vec::<ArrowPayload>::with_capacity(allowed_payloads.len());

        for payload_type in allowed_payloads {
            let record_batch = match otap_batch.get(*payload_type) {
                Some(rb) => rb,
                None => continue,
            };
            let schema = record_batch.schema();

            let schema_id = schema_id(&schema);
            let stream_producer = match self.stream_producers.get_mut(&schema_id) {
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
                        .entry(schema_id)
                        .or_insert(StreamProducer::try_new(
                            *payload_type,
                            schema,
                            payload_schema_id,
                        )?)
                }
                Some(s) => s,
            };

            let serialized_rb = stream_producer.serialize_batch(&record_batch)?;
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
            batch_id: batch_id,
            arrow_payloads,
            ..Default::default()
        })
    }
}
