//! This crate contains code for generating unique IDs for the various
//! record batches that are being exported to clickhouse. It creates a dynamic "TransformPlan"
//! which includes steps for adding an offset ot ID columns and inserting a dynamic partition ID column.
//! All transformations are applied in one shot by the batch transformer.

// TODO: Testing for this is hard since there's no easy way to generate test otap batches outside
// the main project.

use arrow::compute::max;
use arrow::datatypes::{DataType, UInt16Type, UInt32Type};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;
use std::collections::HashMap;
use uuid::Uuid;

use crate::clickhouse_exporter::consts as ch_consts;
use crate::clickhouse_exporter::transform::transform_plan::TransformationPlan;

use super::error::ClickhouseExporterError;

/// This is an ID generator that converts the batch IDs (which are only unique within a given
/// OTAP Batch) into IDs that can be treated as unique.
///
/// Within the partition, the IDs increment more or less sequentially until they would overflow
/// the datatype (u32).
pub struct PartitionSequenceIdGenerator {
    // Bytes representation of a uuid that provides a globally unique ID when combined
    // with one of our ID columns.
    part_bytes: [u8; 32],

    // current max ID. New batches will have their ID columns incremented by at least
    // this value to maintain unique IDs within the partition
    curr_max: u32,

    // Configure which payload types need to have IDs since inlined attributes don't need them.
    // TODO: [Optimization] per @alockett: since ArrowPayloadType has a bounded max value, it might be possible to use
    // either a vec of booleans, or even a u64 treated as a bitmask. This would avoid having to hash the payload
    // type every time we do a lookup here.
    enabled_payload_types: HashMap<ArrowPayloadType, bool>,
}

impl PartitionSequenceIdGenerator {
    pub fn new(enabled_payload_types: HashMap<ArrowPayloadType, bool>) -> Self {
        Self {
            part_bytes: Self::get_uuid_byte_slice(),
            curr_max: 0,
            enabled_payload_types,
        }
    }

    fn init(&mut self) {
        self.part_bytes = Self::get_uuid_byte_slice();
        self.curr_max = 0;
    }

    fn get_uuid_byte_slice() -> [u8; 32] {
        let part_id = Uuid::new_v4();
        let s = part_id.simple().to_string(); // 32‐char hex form
        let bytes = s.as_bytes();
        bytes.try_into().expect("UUID string must be 32 bytes")
    }

    /// Create a transformation plan for ID and PART_ID columns to allow for globally unique rows.
    pub fn generate_id_transform_plan(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
    ) -> Result<HashMap<&ArrowPayloadType, TransformationPlan>, ClickhouseExporterError> {
        // decode the transport optimized IDs if they are not already decoded. This is needed
        // to ensure that we are not computing the sequence of IDs based on delta-encoded IDs.
        // If the IDs are already decoded, this is a no-op.
        otap_batch.decode_transport_optimized_ids().map_err(|e| {
            ClickhouseExporterError::InvalidRecordBatch {
                error: format!("failed to decode transport optimized IDs: {e}"),
            }
        })?;

        let payload_types = otap_batch.allowed_payload_types();

        // find the max ID in any of the record batches. We'll use this later on to increment
        // curr_max and also check for overflows before incrementing all the IDs
        let mut max_id = 0;
        for payload_type in payload_types {
            // Skip this payload type if its represented without ID columns in storage
            if !self
                .enabled_payload_types
                .get(payload_type)
                .unwrap_or(&false)
            {
                continue;
            }
            if let Some(id_column) = otap_batch
                .get(*payload_type)
                .and_then(|rb| rb.column_by_name(consts::ID))
            {
                let record_batch_max_id = match id_column.data_type() {
                    DataType::UInt16 => max::<UInt16Type>(
                        id_column
                            .as_any()
                            .downcast_ref()
                            .expect("can downcast to UInt16Array"),
                    )
                    .map(|max| max as u32),
                    DataType::UInt32 => max::<UInt32Type>(
                        id_column
                            .as_any()
                            .downcast_ref()
                            .expect("can downcast to UInt32Array"),
                    ),
                    _ => None,
                };
                if let Some(record_batch_max_id) = record_batch_max_id {
                    max_id = max_id.max(record_batch_max_id)
                }
            }
        }

        // check if we'll overflow when we increment the IDs
        if max_id.checked_add(self.curr_max).is_none() {
            // if adding curr_max to some record batch would cause the id sequence to overflow,
            // re-init self to begin writing to new partition
            self.init();
        }

        let mut transformation_plans = HashMap::new();
        let insert_ts = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
        // update IDs for all record batch types
        for payload_type in payload_types {
            // Skip this payload type if its represented without ID columns in storage
            if !self.enabled_payload_types.contains_key(payload_type) {
                continue;
            }
            match payload_type {
                ArrowPayloadType::Logs => {
                    let mut tp = TransformationPlan::new();
                    // All ID fields will have been cast to Uint32 in the static transform plan.
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::LogAttrs)
                    {
                        tp.column_ops.add_offset(consts::ID, self.curr_max);
                        // This is a new column, we attach the op to ID so we generate the correct length
                        // if any flattening of struct arrays has occured.
                        tp.column_ops.add_uuid_bytes(consts::ID, self.part_bytes);
                    } else {
                        // We have to attach the uuid to a a column that will be present, since the
                        // log atttributes are known to be inlined, we use that final column name from the multi-op transform.
                        tp.column_ops
                            .add_uuid_bytes(ch_consts::CH_LOG_ATTRIBUTES, self.part_bytes);
                    }
                    // If the resource attributes aren't inlined, we need to offset them
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::ResourceAttrs)
                    {
                        tp.column_ops
                            .add_offset(ch_consts::RESOURCE_ID, self.curr_max);
                    }
                    // If the scope attributes aren't inlined, we need to offset them
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::ScopeAttrs)
                    {
                        tp.column_ops.add_offset(ch_consts::SCOPE_ID, self.curr_max);
                    }
                    let _ = transformation_plans.insert(payload_type, tp);
                }
                ArrowPayloadType::Spans => {
                    let mut tp = TransformationPlan::new();
                    // All ID fields will have been cast to Uint32 in the static transform plan.
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::SpanAttrs)
                    {
                        tp.column_ops.add_offset(consts::ID, self.curr_max);
                        // This is a new column, we attach the op to ID so we generate the correct length
                        // if any flattening of struct arrays has occured.
                        tp.column_ops.add_uuid_bytes(consts::ID, self.part_bytes);
                    } else {
                        // We have to attach the uuid to a a column that will be present, since the
                        // log atttributes are known to be inlined, we use that final column name from the multi-op transform.
                        tp.column_ops
                            .add_uuid_bytes(ch_consts::CH_SPAN_ATTRIBUTES, self.part_bytes);
                    }
                    // If the resource attributes aren't inlined, we need to offset them
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::ResourceAttrs)
                    {
                        tp.column_ops
                            .add_offset(ch_consts::RESOURCE_ID, self.curr_max);
                    }
                    // If the scope attributes aren't inlined, we need to offset them
                    if self
                        .enabled_payload_types
                        .contains_key(&ArrowPayloadType::ScopeAttrs)
                    {
                        tp.column_ops.add_offset(ch_consts::SCOPE_ID, self.curr_max);
                    }
                    let _ = transformation_plans.insert(payload_type, tp);
                }
                ArrowPayloadType::LogAttrs
                | ArrowPayloadType::SpanAttrs
                | ArrowPayloadType::ResourceAttrs
                | ArrowPayloadType::ScopeAttrs => {
                    let mut tp = TransformationPlan::new();
                    // All ID fields will have been cast to Uint32 in the static transform plan.
                    tp.column_ops.add_offset(consts::PARENT_ID, self.curr_max);
                    // This is a new column, we attach the op to ID so we generate the correct length
                    // if any flattening of struct arrays has occured.
                    tp.column_ops
                        .add_uuid_bytes(consts::PARENT_ID, self.part_bytes);
                    // The clickhouse tables expire old data based on a timestamp field. This needs to be
                    // added to the attribute tables since they don't otherwise include a timestamp value.
                    tp.column_ops
                        .add_insert_timestamp(consts::PARENT_ID, insert_ts);
                    let _ = transformation_plans.insert(payload_type, tp);
                }
                // TODO: [support_new_signal] Other types not implemented
                _ => {
                    return Err(ClickhouseExporterError::UnsupportedPayload {
                        error: format!(
                            "Payload type {:?} not implemented for idgen plan",
                            payload_type
                        ),
                    });
                }
            }
        }

        self.curr_max += max_id;
        // try incrementing curr_max by 1 so the next batch won't have
        // any overlapping IDs (batch IDs could start at 0)
        match self.curr_max.checked_add(1) {
            Some(new_curr_max) => {
                self.curr_max = new_curr_max;
            }
            None => {
                self.init();
            }
        }

        Ok(transformation_plans)
    }
}
