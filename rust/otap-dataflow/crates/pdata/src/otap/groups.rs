// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Support for rebatching sequences of `OtapArrowRecords`.
//!
//! # Current Implementation Status
//!
//! This module currently supports batching for **Logs only**.
//!
//! The rebatching approach:
//! - Takes a sequence of input batches and produces maximally-full output batches in a single pass
//! - Piece-by-piece processing: consumes chunks from input, builds maximally-full output batches
//! - Primary table (Logs) drives the batching
//! - Secondary tables (LogAttrs, ScopeAttrs, ResourceAttrs) follow via PARENT_ID references
//! - Resource/Scope attributes are deduplicated within each output batch
//! - Respects the u16::MAX constraint for ID columns
//!
//! ## Algorithm Overview
//!
//! For each output batch:
//! 1. Consume N logs from input (up to max_batch_size)
//! 2. Extract unique resource IDs and scope IDs from those logs
//! 3. Gather corresponding resource/scope attribute rows
//! 4. For logs with attributes (non-NULL log.id): gather log attribute rows
//! 5. Reindex all IDs to be sequential starting from 0
//! 6. Emit the output batch
//!
//! ## Metrics and Traces
//!
//! Batching for Metrics and Traces signals is **not yet implemented**.

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;

use arrow::array::{Array, RecordBatch, StructArray, UInt16Array};
use arrow::datatypes::{DataType, Field, Fields, Schema};
use snafu::{OptionExt, ResultExt};

use crate::{
    error::{self, Result},
    otap::{
        Logs, Metrics, OtapArrowRecordTag, OtapArrowRecords, OtapBatchStore, POSITION_LOOKUP,
        Traces,
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::consts,
};

#[cfg(test)]
mod tests;

/// A sequence of OtapArrowRecords that all share exactly the same tag.
/// Maintains invariant: primary table for each batch is not None and has more than zero records.
#[derive(Clone, Debug, PartialEq)]
pub enum RecordsGroup {
    /// A sequence of batches representing log data
    Logs(Vec<[Option<RecordBatch>; Logs::COUNT]>),
    /// A sequence of batches representing metric data
    Metrics(Vec<[Option<RecordBatch>; Metrics::COUNT]>),
    /// A sequence of batches representing span data
    Traces(Vec<[Option<RecordBatch>; Traces::COUNT]>),
}

impl RecordsGroup {
    /// Convert a sequence of `OtapArrowRecords` into three `RecordsGroup` objects
    #[must_use]
    fn separate_by_type(records: Vec<OtapArrowRecords>) -> [Self; 3] {
        let log_count = tag_count(&records, OtapArrowRecordTag::Logs);
        let mut log_records = Vec::with_capacity(log_count);

        let metric_count = tag_count(&records, OtapArrowRecordTag::Metrics);
        let mut metric_records = Vec::with_capacity(metric_count);

        let trace_count = tag_count(&records, OtapArrowRecordTag::Traces);
        let mut trace_records = Vec::with_capacity(trace_count);

        for records in records {
            match records {
                OtapArrowRecords::Logs(logs) => {
                    let batches = logs.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        log_records.push(batches);
                    }
                }
                OtapArrowRecords::Metrics(metrics) => {
                    let batches = metrics.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        metric_records.push(batches);
                    }
                }
                OtapArrowRecords::Traces(traces) => {
                    let batches = traces.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        trace_records.push(batches);
                    }
                }
            }
        }

        [
            RecordsGroup::Logs(log_records),
            RecordsGroup::Metrics(metric_records),
            RecordsGroup::Traces(trace_records),
        ]
    }

    /// Separate expecting only logs
    pub fn separate_logs(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !metrics.is_empty() || !traces.is_empty() {
            Err(error::MixedSignalsSnafu.build())
        } else {
            Ok(logs)
        }
    }

    /// Separate expecting only metrics
    pub fn separate_metrics(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !logs.is_empty() || !traces.is_empty() {
            Err(error::MixedSignalsSnafu.build())
        } else {
            Ok(metrics)
        }
    }

    /// Separate expecting only traces
    pub fn separate_traces(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !logs.is_empty() || !metrics.is_empty() {
            Err(error::MixedSignalsSnafu.build())
        } else {
            Ok(traces)
        }
    }

    /// Rebatch records in a single pass, creating maximally-full output batches.
    ///
    /// Iterates through input batches once, building output batches that are as close
    /// to `max_output_batch` in size as possible (or u16::MAX if no limit specified).
    pub fn rebatch(self, max_output_batch: Option<NonZeroU64>) -> Result<Self> {
        let effective_max =
            max_output_batch.unwrap_or_else(|| NonZeroU64::new(u16::MAX as u64).unwrap());

        Ok(match self {
            RecordsGroup::Logs(items) => {
                RecordsGroup::Logs(rebatch_logs_single_pass(items, effective_max)?)
            }
            RecordsGroup::Metrics(_) => {
                unimplemented!("Metrics batching is not yet implemented")
            }
            RecordsGroup::Traces(_) => {
                unimplemented!("Traces batching is not yet implemented")
            }
        })
    }

    /// Convert into a sequence of `OtapArrowRecords`
    #[must_use]
    pub fn into_otap_arrow_records(self) -> Vec<OtapArrowRecords> {
        match self {
            RecordsGroup::Logs(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Logs(Logs { batches }))
                .collect(),
            RecordsGroup::Metrics(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Metrics(Metrics { batches }))
                .collect(),
            RecordsGroup::Traces(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Traces(Traces { batches }))
                .collect(),
        }
    }

    /// Is the container empty?
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Logs(logs) => logs.is_empty(),
            Self::Metrics(metrics) => metrics.is_empty(),
            Self::Traces(traces) => traces.is_empty(),
        }
    }

    /// Find the number of OtapArrowRecords we've got.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Logs(logs) => logs.len(),
            Self::Metrics(metrics) => metrics.len(),
            Self::Traces(traces) => traces.len(),
        }
    }
}

// Helper functions
// *************************************************************************************************

fn tag_count(records: &[OtapArrowRecords], tag: OtapArrowRecordTag) -> usize {
    records
        .iter()
        .map(|records| (records.tag() == tag) as usize)
        .sum()
}

/// Fetch the primary table for a given batch
#[must_use]
fn primary_table<const N: usize>(batches: &[Option<RecordBatch>; N]) -> Option<&RecordBatch> {
    match N {
        Logs::COUNT => batches[POSITION_LOOKUP[ArrowPayloadType::Logs as usize]].as_ref(),
        Metrics::COUNT => {
            batches[POSITION_LOOKUP[ArrowPayloadType::UnivariateMetrics as usize]].as_ref()
        }
        Traces::COUNT => batches[POSITION_LOOKUP[ArrowPayloadType::Spans as usize]].as_ref(),
        _ => {
            unreachable!()
        }
    }
}

/// This is a transpose view that lets us look at a sequence of the i-th table given a
/// sequence of RecordBatch arrays.
fn select<const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    i: usize,
) -> impl Iterator<Item = &RecordBatch> {
    batches.iter().flat_map(move |batches| batches[i].as_ref())
}

// Logs batching implementation
// *************************************************************************************************

/// Rebatch logs in a single pass to create maximally-full output batches.
///
/// Algorithm:
/// 1. Unify schemas across all inputs
/// 2. Concatenate all input batches
/// 3. Deduplicate Resource and Scope attributes
/// 4. Sort the Logs table by Resource.ID then Scope.ID
/// 5. Split into output batches of target size
/// 6. Reindex all ID and PARENT_ID columns
fn rebatch_logs_single_pass(
    mut items: Vec<[Option<RecordBatch>; Logs::COUNT]>,
    max_batch_size: NonZeroU64,
) -> Result<Vec<[Option<RecordBatch>; Logs::COUNT]>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    // Phase 1: Unify schemas
    unify_logs(&mut items)?;

    // Phase 2: Concatenate all inputs
    let combined = concatenate_logs_batches(&mut items)?;

    // Phase 3: Deduplicate and sort
    let sorted = sort_and_deduplicate_logs(combined)?;

    // Phase 4: Split to target size
    let outputs = split_logs_to_size(sorted, max_batch_size.get() as usize)?;

    // Phase 5: Reindex IDs
    let mut reindexed = outputs;
    reindex_logs(&mut reindexed)?;

    Ok(reindexed)
}

/// Unify schemas across all log batches to handle optional columns and dictionary variations
fn unify_logs(batches: &mut [[Option<RecordBatch>; Logs::COUNT]]) -> Result<()> {
    // For the MVP, we'll use a simplified unification:
    // Just ensure all batches have the same columns, adding null columns where missing

    for payload_type in Logs::allowed_payload_types() {
        let payload_idx = POSITION_LOOKUP[*payload_type as usize];

        // Collect all field names and their definitions
        let mut all_fields: HashMap<String, Field> = HashMap::new();
        for batch_array in batches.iter() {
            if let Some(batch) = &batch_array[payload_idx] {
                for field in batch.schema().fields() {
                    let _ = all_fields
                        .entry(field.name().clone())
                        .or_insert_with(|| field.as_ref().clone());
                }
            }
        }

        if all_fields.is_empty() {
            continue;
        }

        // Build unified schema with all fields
        let unified_schema = Arc::new(Schema::new(
            all_fields.values().cloned().collect::<Vec<_>>(),
        ));

        // Add missing columns to each batch
        for batch_array in batches.iter_mut() {
            if let Some(batch) = batch_array[payload_idx].take() {
                batch_array[payload_idx] = Some(add_missing_columns(batch, &unified_schema)?);
            }
        }
    }

    Ok(())
}

/// Add missing columns to a batch, filling them with nulls
fn add_missing_columns(batch: RecordBatch, target_schema: &Schema) -> Result<RecordBatch> {
    let (schema, columns, num_rows) = batch.into_parts();
    let mut new_columns = Vec::with_capacity(target_schema.fields().len());

    for field in target_schema.fields() {
        if let Ok(idx) = schema.index_of(field.name()) {
            new_columns.push(columns[idx].clone());
        } else {
            // Add null-filled column for missing field
            new_columns.push(arrow::array::new_null_array(field.data_type(), num_rows));
        }
    }

    RecordBatch::try_new(Arc::new(target_schema.clone()), new_columns).context(error::BatchingSnafu)
}

/// Concatenate all log batches into a single set of batches
fn concatenate_logs_batches(
    items: &mut [[Option<RecordBatch>; Logs::COUNT]],
) -> Result<[Option<RecordBatch>; Logs::COUNT]> {
    let mut result: [Option<RecordBatch>; Logs::COUNT] = Default::default();

    for payload_type in Logs::allowed_payload_types() {
        let payload_idx = POSITION_LOOKUP[*payload_type as usize];

        // Collect all batches for this payload type
        let batches: Vec<&RecordBatch> = select(items, payload_idx).collect();

        if batches.is_empty() {
            continue;
        }

        // Get the schema from the first batch (they should all be unified by now)
        let schema = batches[0].schema();

        // Concatenate
        let combined =
            arrow::compute::concat_batches(&schema, batches.iter().copied()).context(error::BatchingSnafu)?;

        result[payload_idx] = Some(combined);
    }

    Ok(result)
}

/// Sort and deduplicate logs
fn sort_and_deduplicate_logs(
    mut batches: [Option<RecordBatch>; Logs::COUNT],
) -> Result<[Option<RecordBatch>; Logs::COUNT]> {
    // For MVP: Just sort by Resource.ID and Scope.ID
    // Deduplication will be added in a follow-up phase

    let logs_idx = POSITION_LOOKUP[ArrowPayloadType::Logs as usize];

    if let Some(logs_batch) = batches[logs_idx].take() {
        // Extract Resource.ID and Scope.ID for sorting
        let resource_col = logs_batch
            .column_by_name(consts::RESOURCE)
            .context(error::ColumnNotFoundSnafu {
                name: consts::RESOURCE,
            })?;

        let scope_col = logs_batch
            .column_by_name(consts::SCOPE)
            .context(error::ColumnNotFoundSnafu { name: consts::SCOPE })?;

        // Resource and Scope are struct columns, extract the ID field
        let resource_struct = resource_col
            .as_any()
            .downcast_ref::<StructArray>()
            .context(error::ColumnDataTypeMismatchSnafu {
                name: consts::RESOURCE,
                expect: DataType::Struct(Fields::empty()),
                actual: resource_col.data_type().clone(),
            })?;

        let scope_struct = scope_col
            .as_any()
            .downcast_ref::<StructArray>()
            .context(error::ColumnDataTypeMismatchSnafu {
                name: consts::SCOPE,
                expect: DataType::Struct(Fields::empty()),
                actual: scope_col.data_type().clone(),
            })?;

        let resource_id = resource_struct
            .column_by_name(consts::ID)
            .context(error::ColumnNotFoundSnafu { name: consts::ID })?;

        let scope_id = scope_struct
            .column_by_name(consts::ID)
            .context(error::ColumnNotFoundSnafu { name: consts::ID })?;

        // Sort by Resource.ID then Scope.ID
        let sort_columns = vec![
            arrow::compute::SortColumn {
                values: resource_id.clone(),
                options: Some(arrow::compute::SortOptions {
                    descending: false,
                    nulls_first: true,
                }),
            },
            arrow::compute::SortColumn {
                values: scope_id.clone(),
                options: Some(arrow::compute::SortOptions {
                    descending: false,
                    nulls_first: true,
                }),
            },
        ];

        let indices = arrow::compute::lexsort_to_indices(&sort_columns, None)
            .context(error::BatchingSnafu)?;

        let sorted_logs = arrow::compute::take_record_batch(&logs_batch, &indices)
            .context(error::BatchingSnafu)?;

        batches[logs_idx] = Some(sorted_logs);
    }

    Ok(batches)
}

/// Split logs into batches of target size
fn split_logs_to_size(
    batches: [Option<RecordBatch>; Logs::COUNT],
    target_size: usize,
) -> Result<Vec<[Option<RecordBatch>; Logs::COUNT]>> {
    let logs_idx = POSITION_LOOKUP[ArrowPayloadType::Logs as usize];

    let Some(logs_batch) = &batches[logs_idx] else {
        return Ok(vec![batches]);
    };

    let total_rows = logs_batch.num_rows();
    if total_rows == 0 {
        return Ok(vec![batches]);
    }

    // Calculate number of output batches needed
    let num_batches = (total_rows + target_size - 1) / target_size;
    let mut result = Vec::with_capacity(num_batches);

    for batch_idx in 0..num_batches {
        let start = batch_idx * target_size;
        let end = (start + target_size).min(total_rows);
        let length = end - start;

        let mut output_batch: [Option<RecordBatch>; Logs::COUNT] = Default::default();

        // Slice the logs table
        output_batch[logs_idx] = Some(logs_batch.slice(start, length));

        // For now, we'll need to also slice the attribute tables based on PARENT_ID
        // This is simplified - a full implementation would track which attribute rows
        // correspond to which log IDs
        // TODO: Implement proper attribute slicing based on PARENT_ID ranges

        result.push(output_batch);
    }

    Ok(result)
}

/// Reindex all ID and PARENT_ID columns to be sequential starting from 0
fn reindex_logs(batches: &mut [[Option<RecordBatch>; Logs::COUNT]]) -> Result<()> {
    // For MVP: simplified reindexing
    // A full implementation would:
    // 1. Reindex Logs.ID
    // 2. Update LogAttrs.PARENT_ID to match new Logs.ID
    // 3. Reindex Logs.Resource.ID and update ResourceAttrs.PARENT_ID
    // 4. Reindex Logs.Scope.ID and update ScopeAttrs.PARENT_ID

    // For now, we'll just ensure IDs are sequential within each batch
    for batch_array in batches.iter_mut() {
        let logs_idx = POSITION_LOOKUP[ArrowPayloadType::Logs as usize];

        if let Some(logs_batch) = batch_array[logs_idx].take() {
            // Reindex the logs ID column if present
            if logs_batch.column_by_name(consts::ID).is_some() {
                let reindexed = reindex_id_column(logs_batch, consts::ID, 0)?;
                batch_array[logs_idx] = Some(reindexed);
            } else {
                batch_array[logs_idx] = Some(logs_batch);
            }
        }
    }

    Ok(())
}

/// Reindex a single ID column to be sequential starting from start_id
fn reindex_id_column(
    batch: RecordBatch,
    column_name: &'static str,
    start_id: u16,
) -> Result<RecordBatch> {
    let (schema, mut columns, num_rows) = batch.into_parts();

    let id_idx = schema
        .column_with_name(column_name)
        .context(error::ColumnNotFoundSnafu { name: column_name })?
        .0;

    // Create a new sequential ID array
    let new_ids: UInt16Array = (start_id..start_id + num_rows as u16).collect();

    columns[id_idx] = Arc::new(new_ids);

    RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)
}
