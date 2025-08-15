// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for producing BatchArrowRecords from OTAP Batches.
//!
//! `BatchArrowRecords` is the protobuf type that contains the Arrow IPC serialized messages.

use std::{collections::HashMap, io::Cursor};

use arrow::array::{ArrayRef, ArrowPrimitiveType, RecordBatch, UInt32Array};
use arrow::compute::{SortColumn, SortOptions, lexsort_to_indices, take_record_batch};
use arrow::datatypes::{DataType, Schema, SchemaRef};
use arrow::ipc::writer::StreamWriter;
use arrow::row::{RowConverter, SortField};
use snafu::ResultExt;

use crate::error::{self, Result};
use crate::otap::OtapArrowRecords;
use crate::otap::schema::SchemaIdBuilder;
use crate::proto::opentelemetry::arrow::v1::{ArrowPayload, ArrowPayloadType, BatchArrowRecords};
use crate::proto::opentelemetry::metrics::v1::metric::Data;
use crate::schema::consts;

/// handles serializing the stream of record batches for some payload type
struct StreamProducer {
    payload_type: ArrowPayloadType,
    stream_writer: StreamWriter<Cursor<Vec<u8>>>,
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
pub struct Producer {
    next_batch_id: i64,
    next_schema_id: i64,
    stream_producers: HashMap<String, StreamProducer>,
    schema_id_builder: SchemaIdBuilder,
}

impl Producer {
    /// create a new instance of `Producer`
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_batch_id: 0,
            next_schema_id: 0,
            stream_producers: HashMap::new(),
            schema_id_builder: SchemaIdBuilder::new(),
        }
    }

    /// produce `BatchArrowRecords` protobuf message from `OtapBatch`
    pub fn produce_bar(&mut self, otap_batch: &OtapArrowRecords) -> Result<BatchArrowRecords> {
        let allowed_payloads = otap_batch.allowed_payload_types();
        let mut arrow_payloads = Vec::<ArrowPayload>::with_capacity(allowed_payloads.len());

        for payload_type in allowed_payloads {
            let record_batch = match otap_batch.get(*payload_type) {
                Some(rb) => rb,
                None => continue,
            };

            // apply any column encodings to optimize transport
            // let record_batch = apply_column_encodings(payload_type, record_batch);

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

// const LOG_RESOURCE_ID_COL: &str = "resource.id";
// const LOG_SCOPE_ID_COL: &str = "scope.id";

// fn apply_column_encodings(
//     payload_type: &ArrowPayloadType,
//     record_batch: &RecordBatch,
// ) -> RecordBatch {
//     let column_encodings = get_column_encodings(payload_type);
//     if column_encodings.is_empty() {
//         return record_batch.clone();
//     }

//     let mut schema = record_batch.schema().as_ref().clone();

//     // determine which columns need to be encoded
//     let mut to_apply = vec![];
//     for column_encoding in column_encodings {
//         if !is_encoded(column_encoding.path, &schema, column_encoding.encoding) {
//             to_apply.push(column_encoding);
//         }
//     }

//     // nothing to do
//     if to_apply.len() == 0 {
//         return record_batch.clone();
//     }

//     // Prepare to apply sorting ...
//     //
//     // Below we should apply the correct sort order before before we delta-encode the columns.
//     // This is because the columns we're applying the encoding to are unsigned integers, so the
//     // delta cannot be negative. If encoding only been applied to some of the columns already,
//     // (this would be unusual) we might need to remove it before we can sort.
//     if to_apply.len() != column_encodings.len() {
//         // TODO
//         todo!()
//     }

//     // sort the record batch
//     let columns = record_batch.columns();
//     let mut sort_inputs = vec![];
//     for path in get_sort_columns(payload_type) {
//         if let Some(column) = access_column(path, &schema, columns) {
//             sort_inputs.push(column);
//         }
//     }

//     let record_batch = if sort_inputs.len() > 0 {
//         // use row convert to convert to rows
//         // TODO link docs
//         // TODO benchmark against lex_sort_to_indices kernel
//         let sort_fields = sort_inputs
//             .iter()
//             .map(|col| SortField::new(col.data_type().clone()))
//             .collect();
//         let converter = RowConverter::new(sort_fields).unwrap();
//         let rows = converter.convert_columns(&sort_inputs).unwrap();

//         let mut sort: Vec<_> = rows.iter().enumerate().collect();
//         sort.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
//         let indices = UInt32Array::from_iter_values(sort.iter().map(|(i, _)| *i as u32));

//         take_record_batch(record_batch, &indices).unwrap()
//     } else {
//         record_batch.clone()
//     };

//     let mut columns = record_batch.columns().to_vec();
//     for column_encoding in column_encodings {
//         if let Some(column) = access_column(column_encoding.path, &schema, &columns) {
//             let new_column = match column_encoding.encoding {
//                 Encoding::Delta => apply_delta_encoding(column, &column_encoding.data_type),
//                 _ => {
//                     // TODO
//                     todo!()
//                 }
//             };

//             replace_column(column_encoding.path, new_column, &mut schema, &mut columns);
//             update_field_metadata(column_encoding.path, &mut schema, column_encoding.encoding);
//         }
//     }

//     todo!()
// }

// fn is_encoded(path: &str, schema: &Schema, encoding: Encoding) -> bool {
//     todo!()
// }

// fn access_column(path: &str, schema: &Schema, columns: &[ArrayRef]) -> Option<ArrayRef> {
//     todo!();
// }

// fn apply_delta_encoding(column: ArrayRef, data_type: &DataType) -> ArrayRef {
//     todo!();
// }

// fn replace_column(
//     path: &str,
//     new_column: ArrayRef,
//     schema: &mut Schema,
//     columns: &mut Vec<ArrayRef>,
// ) {
//     todo!();
// }

// fn update_field_metadata(path: &str, schema: &mut Schema, encoding: Encoding) {
//     todo!();
// }

// fn get_column_encodings(payload_type: &ArrowPayloadType) -> &'static [ColumnEncoding<'static>] {
//     match payload_type {
//         ArrowPayloadType::LogAttrs => {
//             &[
//                 // TODO could turn this into a macro
//                 ColumnEncoding {
//                     path: consts::ID,
//                     data_type: DataType::UInt16,
//                     encoding: Encoding::Delta,
//                 },
//             ]
//         }
//         _ => &[],
//     }
// }

// fn get_sort_columns(payload_type: &ArrowPayloadType) -> &'static [&'static str] {
//     match payload_type {
//         &ArrowPayloadType::LogAttrs => &[consts::ID, LOG_RESOURCE_ID_COL, LOG_RESOURCE_ID_COL],
//         _ => &[],
//     }
// }

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

        let mut input = OtapArrowRecords::Logs(Logs::default());
        input.set(ArrowPayloadType::Logs, record_batch);
        let mut bar = producer.produce_bar(&input).unwrap();
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
        let mut bar = producer.produce_bar(&input).unwrap();
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
        let mut bar = producer.produce_bar(&input).unwrap();
        let result = OtapArrowRecords::Logs(from_record_messages(
            consumer.consume_bar(&mut bar).unwrap(),
        ));
        assert_eq!(input, result);
    }
}
