// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Retention policy for a payload with a single, unmodified contributor.

use arrow::array::{Array, ArrayRef, AsArray, DictionaryArray, RecordBatch, StructArray};
use arrow::datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type};
use arrow_schema::{ArrowError, DataType};
use arrow_select::dictionary::garbage_collect_any_dictionary;
use std::mem::size_of_val;
use std::sync::Arc;

use super::{InputPlan, PayloadSchema, SelectedSchema, write_payload};
use crate::error::Error;
use crate::otap::memory::{CountedAllocations, array_slice_memory_size, record_batch_pinned_bytes};

pub(super) fn write_single_payload(
    batch: &RecordBatch,
    plan: &InputPlan,
    payload_def: &PayloadSchema,
    selected: SelectedSchema,
) -> crate::otap::Result<RecordBatch> {
    let trimmed;
    let source = if is_compact_batch(batch) {
        if batch.schema_ref().as_ref() == &selected.schema {
            return Ok(batch.clone());
        }
        batch
    } else {
        // The writer copies list ranges, but appends entire dictionary value
        // tables. Prune them first, keeping the schema already selected for output.
        let columns = batch
            .columns()
            .iter()
            .map(trim_column)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| Error::Batching { source })?;
        trimmed = RecordBatch::try_new(batch.schema(), columns)
            .map_err(|source| Error::Batching { source })?;
        &trimmed
    };
    write_payload(&[source], std::slice::from_ref(plan), payload_def, selected)
}

fn is_compact_batch(batch: &RecordBatch) -> bool {
    let pinned = record_batch_pinned_bytes(batch, &mut CountedAllocations::default());
    if pinned <= 4096 {
        return true;
    }
    // Stop as soon as the live data justifies retaining the whole allocation.
    let mut needed = pinned.div_ceil(4);
    for column in batch.columns() {
        let Ok(bytes) = active_bytes(column.as_ref(), needed) else {
            return false;
        };
        needed = needed.saturating_sub(bytes);
        if needed == 0 {
            return true;
        }
    }
    false
}

// Arrow's logical size counts a list's entire child and a dictionary's entire
// values table, even when the slice references only a few entries.
fn active_bytes(array: &dyn Array, needed: usize) -> Result<usize, ArrowError> {
    let validity = array.nulls().map_or(0, |_| array.len().div_ceil(8));
    match array.data_type() {
        DataType::List(_) => {
            let list = array.as_list::<i32>();
            let offsets = list.value_offsets();
            let start = offsets[0] as usize;
            let end = offsets[list.len()] as usize;
            // OTAP list children (histogram buckets, bounds and quantiles) are primitives.
            let child = list.values().slice(start, end - start);
            Ok(validity + size_of_val(offsets) + array_slice_memory_size(child.as_ref())?)
        }
        DataType::Dictionary(key, _) => match key.as_ref() {
            DataType::UInt8 => dictionary_bytes(array.as_dictionary::<UInt8Type>(), needed),
            DataType::UInt16 => dictionary_bytes(array.as_dictionary::<UInt16Type>(), needed),
            other => Err(ArrowError::InvalidArgumentError(format!(
                "unexpected OTAP dictionary key {other}"
            ))),
        },
        DataType::Struct(_) => {
            let mut bytes = validity;
            for child in array.as_struct().columns() {
                if bytes >= needed {
                    break;
                }
                bytes += active_bytes(child.as_ref(), needed - bytes)?;
            }
            Ok(bytes)
        }
        _ => array_slice_memory_size(array),
    }
}

fn dictionary_bytes<K: ArrowDictionaryKeyType>(
    dict: &DictionaryArray<K>,
    needed: usize,
) -> Result<usize, ArrowError> {
    let mut bytes = array_slice_memory_size(dict.keys())?;
    if bytes >= needed {
        return Ok(bytes);
    }
    let used = dict.occupancy();
    let values = dict.values();
    let count = used.count_set_bits();
    let offsets = match values.data_type() {
        DataType::Utf8 => Some(values.as_string::<i32>().value_offsets()),
        DataType::Binary => Some(values.as_binary::<i32>().value_offsets()),
        _ => None,
    };
    if let Some(offsets) = offsets {
        // Count offsets once. Consult the original offsets directly so that a
        // fragmented set of keys does not allocate an Arrow slice per run.
        bytes += (count + 1) * size_of::<i32>();
        for (start, end) in used.set_slices() {
            bytes += (offsets[end] - offsets[start]) as usize;
            if bytes >= needed {
                break;
            }
        }
    } else {
        // All other supported OTAP dictionary values are fixed width.
        bytes += array_slice_memory_size(values.slice(0, count).as_ref())?;
    }
    Ok(bytes)
}

fn trim_column(array: &ArrayRef) -> Result<ArrayRef, ArrowError> {
    match array.data_type() {
        DataType::Dictionary(_, _) => garbage_collect_any_dictionary(array.as_any_dictionary()),
        DataType::Struct(fields) => {
            let children = array
                .as_struct()
                .columns()
                .iter()
                .map(trim_column)
                .collect::<Result<_, _>>()?;
            Ok(Arc::new(StructArray::try_new_with_length(
                fields.clone(),
                children,
                array.nulls().cloned(),
                array.len(),
            )?))
        }
        _ => Ok(array.clone()),
    }
}
