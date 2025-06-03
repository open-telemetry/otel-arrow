// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This crate contains code for generating unique IDs for the various
//! record batches that are being exported to parquet

use std::sync::Arc;

use arrow::array::{RecordBatch, UInt32Array};
use arrow::compute::kernels::cast;
use arrow::compute::kernels::numeric::add;
use arrow::compute::max;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::error::ArrowError;
use otel_arrow_rust::otap::{OtapBatch, child_payload_types};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::{consts, update_schema_metadata};
use uuid::Uuid;

/// Definition of errors that could happen when generating an ID
#[derive(thiserror::Error, Debug)]
pub enum IdGeneratorError {
    #[error("Invalid record batch: {error}")]
    InvalidRecordBatch { error: String },

    #[error("overflow")]
    Overflow {},

    #[error("Unknown error occurred: {error}")]
    UnknownError { error: String },
}

/// This is an ID generator that converts the batch IDs (which are only unique within a given
/// OTAP Batch) into IDs that can be treated as unique.
///
/// This particular implementation only generates IDs that are unique within some partition.
/// It expects the writer to partition the parquet file by the partition ID, which it sets on
/// the schema metadata.
///
/// Within the partition, the IDs increment more or less sequentially until they would overflow
/// the datatype (u32).
pub struct PartitionSequenceIdGenerator {
    part_id: Uuid,

    // current max ID. New batches will have their ID columns incremented by at least
    // this value to maintain unique IDs within the partition
    curr_max: u32,
}

pub const PARTITION_METADATA_KEY: &str = "_part_id";

impl PartitionSequenceIdGenerator {
    pub fn new() -> Self {
        Self {
            part_id: Uuid::new_v4(),
            curr_max: 0,
        }
    }

    fn init(&mut self) {
        self.part_id = Uuid::new_v4();
        self.curr_max = 0;
    }

    pub fn generate_unique_ids(
        &mut self,
        otap_batch: &mut OtapBatch,
    ) -> Result<(), IdGeneratorError> {
        // decode the transport optimized IDs if they are not already decoded. This is needed
        // to ensure that we are not computing the sequence of IDs based on delta-encoded IDs.
        // If the IDs are already decoded, this is a no-op.
        otap_batch.decode_transport_optimized_ids().map_err(|e| {
            IdGeneratorError::InvalidRecordBatch {
                error: format!("failed to decode transport optimized IDs: {}", e),
            }
        })?;

        loop {
            match otap_batch {
                OtapBatch::Logs(_) => {
                    let logs_rb = match otap_batch.get(ArrowPayloadType::Logs) {
                        Some(rb) => rb,
                        // nothing to do -- this could mean an empty batch
                        None => break,
                    };

                    let new_logs_rb = match self.add_offset_to_column(consts::ID, logs_rb) {
                        Ok(rb) => rb,
                        Err(IdGeneratorError::Overflow {}) => {
                            // re-init the curr_offset with a new node_id
                            self.init();
                            continue;
                        }
                        Err(e) => return Err(e),
                    };

                    let new_logs_rb = update_schema_metadata(
                        new_logs_rb,
                        PARTITION_METADATA_KEY.into(),
                        format!("{}", self.part_id),
                    );

                    // hang onto the max_id so we can update current offset later
                    // safety: we've already added an ID column of type UInt32 to the record batch here
                    // in the call to add_offset_to_column
                    let id_col = new_logs_rb
                        .column_by_name(consts::ID)
                        .expect("expect id is in batch here");
                    let id_col = id_col
                        .as_any()
                        .downcast_ref::<UInt32Array>()
                        .expect("expect this is the datatype");
                    let max_id = match max(id_col) {
                        Some(max) => max,
                        // this would be null if the id_col was all nulls, in which case
                        // there shouldn't be any children and we can just break
                        None => break,
                    };

                    otap_batch.set(ArrowPayloadType::Logs, new_logs_rb);

                    for child_payload_type in child_payload_types(ArrowPayloadType::Logs) {
                        if let Some(rb) = otap_batch.get(*child_payload_type) {
                            let new_rb = self.add_offset_to_column(consts::PARENT_ID, rb)?;
                            otap_batch.set(*child_payload_type, new_rb);
                        }
                    }

                    self.curr_max += max_id;
                    self.curr_max += 1;
                }
                _ => {
                    // TODO -- for the other record batches, we'll need to refactor this to
                    // traverse the structure of the tree of record batches recursively transform
                    // the parent ID, update the children as well.
                    // https://github.com/open-telemetry/otel-arrow/issues/503
                    todo!("implement generate ids for other signal types")
                }
            }

            break;
        }

        Ok(())
    }

    fn add_offset_to_column(
        &self,
        column_name: &str,
        record_batch: &RecordBatch,
    ) -> Result<RecordBatch, IdGeneratorError> {
        let schema = record_batch.schema_ref();
        let id_column_index = match schema.index_of(column_name) {
            Ok(index) => index,
            Err(_) => {
                return Err(IdGeneratorError::InvalidRecordBatch {
                    error: format!("Did not find expected column {}", column_name),
                });
            }
        };
        let id_column = record_batch.column(id_column_index);

        // cast ID column to type of self.curr_offset and add offset
        let id_column = match cast(id_column, &DataType::UInt32) {
            Ok(id_col) => id_col,
            Err(err) => {
                return Err(IdGeneratorError::InvalidRecordBatch {
                    error: format!("could not cast ID column to UInt32: {}", err),
                });
            }
        };
        let new_id_col = match add(&UInt32Array::new_scalar(self.curr_max), &id_column) {
            Ok(col) => col,
            // return error if overflows
            Err(ArrowError::ArithmeticOverflow(_)) => return Err(IdGeneratorError::Overflow {}),
            Err(e) => {
                return Err(IdGeneratorError::UnknownError {
                    error: format!("{}", e),
                });
            }
        };

        // construct new record batch
        let new_columns = record_batch
            .columns()
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if i == id_column_index {
                    new_id_col.clone()
                } else {
                    col.clone()
                }
            })
            .collect::<Vec<_>>();

        let new_fields = schema
            .fields()
            .into_iter()
            .enumerate()
            .map(|(i, field)| {
                if i == id_column_index {
                    Arc::new(Field::new(column_name, DataType::UInt32, false))
                } else {
                    field.clone()
                }
            })
            .collect::<Vec<_>>();

        // safety: this call should succeed based on how we're constructing the record batch
        // e.g. using the same fields/columns, except for replacing ID with the correct type
        Ok(
            RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns)
                .expect("can construct record batch"),
        )
    }
}

#[cfg(test)]
pub mod test {
    use otel_arrow_rust::Consumer;
    use otel_arrow_rust::otap::from_record_messages;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::schema::get_schema_metadata;

    use crate::parquet_exporter::test::datagen::SimpleLogDataGenOptions;

    use super::super::test::datagen::create_single_arrow_record_batch;
    use super::*;

    #[test]
    fn test_partition_sequence_id_generator_logs() {
        let mut id_generator = PartitionSequenceIdGenerator::new();

        let mut log_batch = create_single_arrow_record_batch(SimpleLogDataGenOptions {
            id_offset: 0,
            num_rows: 2,
            ..Default::default()
        });
        let mut consumer = Consumer::default();
        let record_messages = consumer.consume_bar(&mut log_batch).unwrap();
        let mut otap_batch1 = OtapBatch::Logs(from_record_messages(record_messages));
        id_generator.generate_unique_ids(&mut otap_batch1).unwrap();
        let expected_ids = UInt32Array::from_iter_values(vec![0, 1]);

        let logs_rb = otap_batch1.get(ArrowPayloadType::Logs).unwrap();
        let id_col = logs_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(id_col, &expected_ids);

        // check that the schema metadata was correctly applied
        let val = get_schema_metadata(logs_rb.schema_ref(), PARTITION_METADATA_KEY).unwrap();
        assert_eq!(val, &format!("{}", id_generator.part_id));

        // expect that the parent IDs are also updated
        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ] {
            let rb = otap_batch1.get(payload_type).unwrap();
            let parent_id_col = rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref::<UInt32Array>()
                .unwrap();
            assert_eq!(parent_id_col, &expected_ids);
        }

        // add a second batch
        let mut log_batch = create_single_arrow_record_batch(SimpleLogDataGenOptions {
            id_offset: 0,
            num_rows: 2,
            ..Default::default()
        });
        let mut consumer = Consumer::default();
        let record_messages = consumer.consume_bar(&mut log_batch).unwrap();
        let mut otap_batch2 = OtapBatch::Logs(from_record_messages(record_messages));
        id_generator.generate_unique_ids(&mut otap_batch2).unwrap();
        let expected_ids = UInt32Array::from_iter_values(vec![2, 3]);

        // validate the IDs are now offset by the max of the previous IDs
        let logs_rb = otap_batch2.get(ArrowPayloadType::Logs).unwrap();
        let id_col = logs_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(id_col, &expected_ids);

        // expect that the parent IDs are also updated
        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ] {
            let rb = otap_batch2.get(payload_type).unwrap();
            let parent_id_col = rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref::<UInt32Array>()
                .unwrap();
            assert_eq!(parent_id_col, &expected_ids);
        }
    }

    #[test]
    fn test_partition_sequence_id_generator_logs_with_overflow() {
        let mut id_generator = PartitionSequenceIdGenerator::new();
        id_generator.curr_max = u32::MAX - 5;
        let curr_partition_id = id_generator.part_id;

        let mut log_batch = create_single_arrow_record_batch(SimpleLogDataGenOptions {
            id_offset: 0,
            num_rows: 10,
            ..Default::default()
        });
        let mut consumer = Consumer::default();
        let record_messages = consumer.consume_bar(&mut log_batch).unwrap();
        let mut otap_batch = OtapBatch::Logs(from_record_messages(record_messages));

        id_generator.generate_unique_ids(&mut otap_batch).unwrap();
        let expected_ids = UInt32Array::from_iter_values(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let id_col = logs_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(id_col, &expected_ids);

        // check that a new partition was created due to rollover
        assert_ne!(curr_partition_id, id_generator.part_id);
        let val = get_schema_metadata(logs_rb.schema_ref(), PARTITION_METADATA_KEY).unwrap();
        assert_eq!(val, &format!("{}", id_generator.part_id));
        assert_ne!(val, &format!("{}", curr_partition_id));
    }

    #[test]
    fn test_partition_sequence_id_generator_decodes_ids() {
        let mut id_generator = PartitionSequenceIdGenerator::new();

        let mut log_batch = create_single_arrow_record_batch(SimpleLogDataGenOptions {
            id_offset: 0,
            num_rows: 3,
            ids_decoded: false,
            ..Default::default()
        });
        let mut consumer = Consumer::default();
        let record_messages = consumer.consume_bar(&mut log_batch).unwrap();
        let mut otap_batch1 = OtapBatch::Logs(from_record_messages(record_messages));
        id_generator.generate_unique_ids(&mut otap_batch1).unwrap();
        let expected_ids = UInt32Array::from_iter_values(vec![1, 2, 3]);

        let logs_rb = otap_batch1.get(ArrowPayloadType::Logs).unwrap();
        let id_col = logs_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt32Array>()
            .unwrap();
        assert_eq!(id_col, &expected_ids);

        // check that the schema metadata was correctly applied
        let val = get_schema_metadata(logs_rb.schema_ref(), PARTITION_METADATA_KEY).unwrap();
        assert_eq!(val, &format!("{}", id_generator.part_id));

        // expect that the parent IDs are also updated
        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ] {
            let rb = otap_batch1.get(payload_type).unwrap();
            let parent_id_col = rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref::<UInt32Array>()
                .unwrap();
            assert_eq!(parent_id_col, &expected_ids);
        }
    }
}
