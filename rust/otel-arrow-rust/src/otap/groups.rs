// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Support for splitting and merging sequences of `OtapArrowRecords` in support of batching.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    iter::{once, repeat, repeat_n},
    num::NonZeroU64,
    ops::{Add, Range, RangeFrom, RangeInclusive},
    sync::Arc,
};

use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, BinaryArray, DictionaryArray, FixedSizeBinaryArray,
        GenericByteArray, PrimitiveArray, RecordBatch, StringArray, StructArray, UInt16Array,
        UInt32Array,
    },
    buffer::NullBuffer,
    compute::cast,
    datatypes::{
        ArrowNativeTypeOp, ByteArrayType, DataType, Field, Fields, Float32Type, Float64Type,
        Int32Type, Int64Type, Schema, SchemaBuilder, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
    },
};
use itertools::Itertools;
use smallvec::SmallVec;
use snafu::{OptionExt, ResultExt};

use crate::{
    otap::{
        DATA_POINTS_TYPES, Logs, Metrics, OtapArrowRecordTag, OtapArrowRecords, OtapBatchStore,
        POSITION_LOOKUP, Traces, batch_length, child_payload_types,
        error::{self, Result},
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::consts,
};

/// I logically represent a sequence of OtapArrowRecords that all share exactly the same tag.  I
/// maintain an invariant that the primary table for each telemetry type in each batch is not None
/// and has more than zero records.
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
    pub fn split_by_type(records: Vec<OtapArrowRecords>) -> [Self; 3] {
        let log_count = tag_count(&records, OtapArrowRecordTag::Logs);
        let mut log_records = Vec::with_capacity(log_count);

        let metric_count = tag_count(&records, OtapArrowRecordTag::Metrics);
        let mut metric_records = Vec::with_capacity(metric_count);

        let trace_count = tag_count(&records, OtapArrowRecordTag::Traces);
        let mut trace_records = Vec::with_capacity(trace_count);

        for records in records {
            match records {
                OtapArrowRecords::Logs(logs) => {
                    let batches = shrink(logs.into_batches());
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
                    let batches = shrink(traces.into_batches());
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

    /// Split `RecordBatch`es as need when they're larger than our threshold or when we need them in
    /// smaller pieces to concatenate together into our target size.
    pub fn split(self, max_output_batch: NonZeroU64) -> Result<Self> {
        Ok(match self {
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_split(
                items,
                max_output_batch,
                Logs::allowed_payload_types(),
                ArrowPayloadType::Logs,
            )?),
            RecordsGroup::Metrics(items) => RecordsGroup::Metrics(generic_split(
                items,
                max_output_batch,
                Metrics::allowed_payload_types(),
                ArrowPayloadType::UnivariateMetrics,
            )?),
            RecordsGroup::Traces(items) => RecordsGroup::Traces(generic_split(
                items,
                max_output_batch,
                Traces::allowed_payload_types(),
                ArrowPayloadType::Spans,
            )?),
        })
    }

    /// Merge `RecordBatch`es together so that they're no bigger than `max_output_batch`.
    pub fn concatenate(self, max_output_batch: Option<NonZeroU64>) -> Result<Self> {
        Ok(match self {
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_concatenate(
                items,
                Logs::allowed_payload_types(),
                max_output_batch,
            )?),
            RecordsGroup::Metrics(items) => RecordsGroup::Metrics(generic_concatenate(
                items,
                Metrics::allowed_payload_types(),
                max_output_batch,
            )?),
            RecordsGroup::Traces(items) => RecordsGroup::Traces(generic_concatenate(
                items,
                Traces::allowed_payload_types(),
                max_output_batch,
            )?),
        })
    }

    // FIXME: replace this with an Extend impl to avoid unnecessary allocations
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
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Logs(logs) => logs.is_empty(),
            Self::Metrics(metrics) => metrics.is_empty(),
            Self::Traces(traces) => traces.is_empty(),
        }
    }

    /// Find the number of OtapArrowRecords we've got.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Logs(logs) => logs.len(),
            Self::Metrics(metrics) => metrics.len(),
            Self::Traces(traces) => traces.len(),
        }
    }
}

// *************************************************************************************************
// Everything above this line is the public interface and everything below this line is internal
// implementation details.

// Some helpers for `RecordsGroup`...
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

/// In order to make `into_batches()` work, it has to return the maximally sized array of
/// `Option<RecordBatch>` padding out the end with `None`s. This function undoes that for
/// reintegrating the data into a generic context where we know exactly how big the array should be.
fn shrink<T, const BIGGER: usize, const SMALLER: usize>(
    array: [Option<T>; BIGGER],
) -> [Option<T>; SMALLER] {
    // Because the T we actually care about doesn't impl Copy, I think this is the simplest way to
    // verify that the tail is all None.
    for none in array[SMALLER..].iter() {
        assert!(none.is_none());
    }

    assert!(SMALLER < BIGGER);
    let mut iter = array.into_iter();
    // SAFETY: we've already verified that the iterator won't run out with the assert above.
    std::array::from_fn(|_| {
        iter.next()
            .expect("we will have the right number of elements")
    })
}

// Code for splitting batches
// *************************************************************************************************

fn generic_split<const N: usize>(
    mut batches: Vec<[Option<RecordBatch>; N]>,
    max_output_batch: NonZeroU64,
    allowed_payloads: &[ArrowPayloadType],
    primary_payload: ArrowPayloadType,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    assert_eq!(N, allowed_payloads.len());
    assert!(allowed_payloads.contains(&primary_payload));
    assert!(!batches.is_empty());

    // First, ensure that all RecordBatches are sorted by parent_id & id so that we can efficiently
    // pluck ranges from them.
    for batches in batches.iter_mut() {
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            if let Some(rb) = std::mem::take(&mut batches[i]) {
                let rb = sort_record_batch(rb, HowToSort::SortByParentIdAndId)?;
                batches[i] = Some(rb);
            }
        }
    }

    // Next, split the primary table
    let primary_offset = POSITION_LOOKUP[primary_payload as usize];
    let mut result = Vec::with_capacity(
        batches
            .iter()
            .map(batch_length)
            .map(|l| l as u64)
            .sum::<u64>()
            .div_ceil(max_output_batch.get()) as usize,
    );
    // SAFETY: on 32-bit archs, `as` conversion from u64 to usize can be wrong for values >=
    // u32::MAX, but we don't care about those cases because if they happen we'll only fail to avoid
    // a reallocation.

    let splits = if N == Metrics::COUNT {
        split_metric_batches(max_output_batch, &batches)?
    } else {
        split_non_metric_batches(max_output_batch, &batches)?
    };
    let groups = splits.into_iter().chunk_by(|(batch_index, _)| *batch_index);
    let mut splits = Vec::new();
    let mut lengths = Vec::new();
    let mut split_primary = Vec::new();

    for (batch_index, ranges) in &groups {
        splits.clear();
        lengths.clear();
        split_primary.clear();
        let batches = &mut batches[batch_index];

        if let Some(rb) = std::mem::take(&mut batches[primary_offset]) {
            let original_length = rb.num_rows();
            splits.extend(ranges.map(|pair| pair.1));
            for range in splits.iter() {
                let offset = range.start;
                let length = range.end - range.start;
                lengths.push(length);
                split_primary.push(rb.slice(offset, length));
            }
            assert_eq!(original_length, lengths.iter().sum::<usize>());
            assert_eq!(
                original_length,
                split_primary.iter().map(|rb| rb.num_rows()).sum::<usize>()
            );
            let ids = IDSeqs::from_col(IDColumn::extract(&rb, consts::ID)?, &lengths);

            // use ids to split the child tables: call split_child_record_batch
            let new_batch_count = split_primary.len();
            result.extend(repeat_n([const { None }; N], new_batch_count));
            let result_len = result.len(); // only here to avoid immutable borrowing overlapping mutable borrowing
            // this is where we're going to be writing the rest of this split batch into!
            let new_batch = &mut result[result_len - new_batch_count..];

            // Store the newly split primary tables into this function's result
            for (i, split_primary) in split_primary.drain(..).enumerate() {
                new_batch[i][primary_offset] = Some(split_primary);
            }
            for payload in allowed_payloads
                .iter()
                .filter(|payload| **payload != primary_payload)
                .copied()
            {
                generic_split_helper(batches, payload, primary_payload, &ids, new_batch)?;
            }
        } else {
            panic!("expected to have primary for every group");
        }

        // When we're done, the input should be an empty husk; if there's anything still left there,
        // that means we screwed up!
        assert_eq!(batches, &[const { None }; N]);
    }

    Ok(result)
}

// This is a recursive helper function; the depth of recursion is bounded by parent-child
// relationships described in `child_payload_types` so we won't blow the stack.
fn generic_split_helper<const N: usize>(
    input: &mut [Option<RecordBatch>; N],
    payload: ArrowPayloadType,
    primary_payload: ArrowPayloadType,
    parent_ids: &IDSeqs,
    output: &mut [[Option<RecordBatch>; N]],
) -> Result<()> {
    // First, do the splitting, but only for non-primary payloads since those have to be done
    // specially.
    assert_ne!(payload, primary_payload);

    let payload_offset = POSITION_LOOKUP[payload as usize];
    if let Some(table) = std::mem::take(&mut input[payload_offset]) {
        // `table` is a parent table with some children!
        let child_payloads = child_payload_types(payload);

        // Let's split it...
        let split_table_parts = parent_ids.split_child_record_batch(&table)?;
        assert_eq!(split_table_parts.len(), output.len());

        // ...and then recursively call ourself to do the same with our children
        if !child_payloads.is_empty()
            && child_payloads
                .iter()
                .any(|&payload| input[POSITION_LOOKUP[payload as usize]].is_some())
        {
            // We don't want to construct `id` unless there are children. Note that leaf nodes
            // won't have children and consequently won't have an `id` column so we shouldn't
            // construct `id` blindly!
            let id = IDSeqs::from_split_cols(&split_table_parts)?;

            for child_payload in child_payloads {
                generic_split_helper(input, *child_payload, primary_payload, &id, output)?;
            }
        }

        // ...and stash the result in `output`
        for (i, split_table) in split_table_parts.into_iter().enumerate() {
            output[i][payload_offset] = split_table;
        }
    }

    Ok(())
}

/// I'm a convenient wrapper for dealing with ID and PARENT_ID columns in generic code.
enum IDColumn<'rb> {
    U16(&'rb UInt16Array),
    U32(&'rb UInt32Array),
}

impl<'rb> IDColumn<'rb> {
    fn extract(input: &'rb RecordBatch, column_name: &'static str) -> Result<IDColumn<'rb>> {
        use snafu::OptionExt;
        let id = input
            .column_by_name(column_name)
            .context(error::ColumnNotFoundSnafu { name: column_name })?;

        Self::from_array(column_name, id)
    }

    fn from_array(column_name: &'static str, id: &'rb dyn Array) -> Result<IDColumn<'rb>> {
        match (
            id.as_any().downcast_ref::<UInt16Array>(),
            id.as_any().downcast_ref::<UInt32Array>(),
        ) {
            (Some(array), None) => Ok(Self::U16(array)),
            (None, Some(array)) => Ok(Self::U32(array)),
            (Some(_), Some(_)) => unreachable!(),
            (None, None) => {
                error::ColumnDataTypeMismatchSnafu {
                    name: column_name,
                    expect: DataType::UInt16, // Or UInt32, but we can only provide one
                    actual: id.data_type().clone(),
                }
                .fail()
            }
        }
    }
}

/// I describe ID column values for splitting
enum IDSeqs {
    RangeU16(Vec<Option<RangeInclusive<u16>>>),
    RangeU32(Vec<Option<RangeInclusive<u32>>>),
}

impl From<Vec<Option<RangeInclusive<u16>>>> for IDSeqs {
    fn from(r: Vec<Option<RangeInclusive<u16>>>) -> Self {
        Self::RangeU16(r)
    }
}

impl From<Vec<Option<RangeInclusive<u32>>>> for IDSeqs {
    fn from(r: Vec<Option<RangeInclusive<u32>>>) -> Self {
        Self::RangeU32(r)
    }
}

impl IDSeqs {
    fn from_col<'rb>(ids: IDColumn<'rb>, lengths: &[usize]) -> Self {
        match ids {
            IDColumn::U16(array) => Self::from_generic_array(array, lengths),
            IDColumn::U32(array) => Self::from_generic_array(array, lengths),
        }
    }

    fn from_generic_array<ArrowPrimitive>(
        array: &PrimitiveArray<ArrowPrimitive>,
        lengths: &[usize],
    ) -> Self
    where
        ArrowPrimitive: ArrowPrimitiveType,
        ArrowPrimitive::Native: ArrowNativeTypeOp
            + From<u16>
            + Copy
            + PartialEq
            + Eq
            + Add<Output = ArrowPrimitive::Native>,
        Self: From<Vec<Option<RangeInclusive<ArrowPrimitive::Native>>>>,
    {
        // Null-handling:
        // Again, we rely on the fact that we've sorted IDs with nulls coming first.

        let slice = &array.values()[..];
        let first_valid_index = match array.nulls() {
            // No nulls!
            None => Some(0),

            // All nulls...
            Some(_) if array.null_count() == array.len() => None,

            // Some, but not all nulls!
            Some(nulls) => {
                // SAFETY: unwrap is safe here because we've already verified that the entire array
                // isn't null, which means there has to be at least one valid index which means
                // .next() will return Some the first time we call it.
                Some(
                    nulls
                        .valid_indices()
                        .next()
                        .expect("a non-null must be here"),
                )
            }
        };

        let mut ranges = Vec::with_capacity(lengths.len());
        let mut start = 0;
        for length in lengths {
            // subslice[0] and subslice[len-1] expressions at the end of this block rely on the fact
            // that subslice is not empty!
            assert!(*length > 0);

            let end = start + length;
            let ids = match first_valid_index {
                Some(first_valid_index) if start >= first_valid_index => {
                    let subslice = &slice[start.max(first_valid_index)..end];
                    Some(subslice[0]..=subslice[subslice.len() - 1])
                }
                _ => None,
            };

            ranges.push(ids);
            start = end;
        }

        ranges.into()
    }

    fn from_split_cols(inputs: &[Option<RecordBatch>]) -> Result<Self> {
        let column_name = consts::ID;
        let lengths = inputs
            .iter()
            .flatten()
            .map(|input| input.num_rows())
            .collect::<Vec<_>>();

        let ids: Result<Vec<_>> = inputs
            .iter()
            .flatten()
            .map(|rb| IDColumn::extract(rb, column_name))
            .collect();
        let ids = ids?;
        let concatenated_array = match ids.first().expect("there should be at least one input") {
            IDColumn::U16(_) => {
                let mut refs = Vec::with_capacity(lengths.len());
                for id in ids {
                    if let IDColumn::U16(next_array) = id {
                        let next_array: &dyn Array = next_array;
                        refs.push(next_array);
                    } else {
                        panic!();
                    }
                }
                arrow::compute::concat(&refs)
            }
            IDColumn::U32(_) => {
                let mut refs = Vec::with_capacity(lengths.len());
                for id in ids {
                    if let IDColumn::U32(next_array) = id {
                        let next_array: &dyn Array = next_array;
                        refs.push(next_array);
                    } else {
                        panic!();
                    }
                }
                arrow::compute::concat(&refs)
            }
        }
        .context(error::BatchingSnafu)?;

        let ids = IDColumn::from_array(column_name, &concatenated_array)?;
        Ok(Self::from_col(ids, &lengths))
    }

    fn split_child_record_batch(&self, input: &RecordBatch) -> Result<Vec<Option<RecordBatch>>> {
        let id = IDColumn::extract(input, consts::PARENT_ID)?;
        Ok(match (self, id) {
            (IDSeqs::RangeU16(id_ranges), IDColumn::U16(parent_ids)) => {
                Self::generic_split_child_record_batch(id_ranges, parent_ids, input)
            }
            (IDSeqs::RangeU32(id_ranges), IDColumn::U32(parent_ids)) => {
                Self::generic_split_child_record_batch(id_ranges, parent_ids, input)
            }
            _ => {
                panic!();
            }
        })
    }

    fn generic_split_child_record_batch<T>(
        id_ranges: &[Option<RangeInclusive<T::Native>>],
        parent_ids: &PrimitiveArray<T>,
        input: &RecordBatch,
    ) -> Vec<Option<RecordBatch>>
    where
        T: ArrowPrimitiveType,
        T::Native: ArrowNativeTypeOp + From<u16> + Copy + PartialEq + Eq + Add<Output = T::Native>,
    {
        let one = T::Native::from(1u16);

        let slice = parent_ids.values();
        if parent_ids.null_count() == 0 {
            debug_assert!(slice.is_sorted());
        }

        let mut result = Vec::with_capacity(id_ranges.len());
        for range in id_ranges.iter().flatten() {
            // We're using `partition_point` instead of `.binary_search` because it returns a
            // deterministic result in cases where there are multiple results found.

            // the first index where id >= start
            let first_index = slice.partition_point(|id| id < range.start());
            if first_index >= slice.len() {
                // range doesn't show up in table
                result.push(None);
                continue;
            }

            // the last index where id <= end
            let last_index = slice
                .partition_point(|&id| id < *range.end() + one)
                .checked_sub(1)
                .unwrap_or(first_index);

            result.push(
                // slice record batch if the  id range doesn't show up in this table
                (range.contains(&slice[first_index]) && range.contains(&slice[last_index]))
                    .then(|| input.slice(first_index, 1 + (last_index - first_index))),
            );
        }

        result
    }
}

fn split_non_metric_batches<const N: usize>(
    max_output_batch: NonZeroU64,
    batches: &[[Option<RecordBatch>; N]],
) -> Result<Vec<(usize, Range<usize>)>> {
    let mut result = Vec::new();

    let mut total_records_seen: u64 = 0; // think of this like iter::single(0).chain(batch_sizes.iter()).cumsum()
    for (batch_index, batches) in batches.iter().enumerate() {
        let num_records = batch_length(batches);

        // SAFETY: % panics if the second arg is 0, but we're relying on NonZeroU64 to ensure
        // that can't happen.
        let prev_batch_size = total_records_seen % max_output_batch.get();
        let first_batch_size = (max_output_batch.get() - prev_batch_size) as usize;
        // FIXME: this calculation is broken for logs & traces since it doesn't take into account
        // how we have to limit batch size to accomodate the u16::MAX size limit for non-null IDs.

        if num_records > first_batch_size {
            let batch_sizes = once(first_batch_size).chain(repeat(max_output_batch.get() as usize));
            let mut offset = 0;
            for batch_size in batch_sizes {
                let batch_size = batch_size.min(num_records - offset);
                result.push((batch_index, offset..(offset + batch_size)));
                offset += batch_size;
                if offset >= num_records {
                    break;
                }
            }
        } else {
            result.push((batch_index, 0..num_records));
        }

        total_records_seen += num_records as u64;
    }

    Ok(result)
}

// Splitting batches of metrics requires special handling because for metrics, we don't measure
// batch size in terms of the primary table, but rather in terms of the total count of
// DataPoints. The one saving grace is that for Metrics and all the DataPoints tables, ID and
// PARENT_ID columns are not nullable!
fn split_metric_batches<const N: usize>(
    max_output_batch: NonZeroU64,
    batches: &[[Option<RecordBatch>; N]],
) -> Result<Vec<(usize, Range<usize>)>> {
    assert_eq!(N, Metrics::COUNT);
    // Because the metrics table has non-nullable 16-bit IDs, you can never have a batch with more
    // than u16::MAX metrics.

    const METRICS_INDEX: usize = POSITION_LOOKUP[ArrowPayloadType::UnivariateMetrics as usize];

    // `child_counts` maps for each ID in the metrics tables, how many DataPoints there are; we'll
    // use this data to figure out how to split `batches`
    let mut child_counts: Vec<u64> = Vec::new();
    // Why `u64`? Because a given batch can have at most `u16::MAX` metrics with children, but each
    // metrics can have up to `4 * u32::MAX` children (since there are 4 data points tables); the
    // worst case count of children is more than can fit in a `u32` and since on 32-bit
    // architectures, `usize` is a `u32`, we need a `u64`.
    let mut cumulative_child_counts: Vec<u64> = Vec::new();

    let mut result = Vec::new();
    let max_output_batch = max_output_batch.get() as usize;
    let mut batch_size = max_output_batch;

    for (batch_index, batches) in batches.iter().enumerate() {
        use arrow::array::as_primitive_array;

        let metrics = batches[METRICS_INDEX]
            .as_ref()
            .expect("we've alredy ensured that every batch has a non-null primary table");
        let metric_ids: &PrimitiveArray<UInt16Type> = as_primitive_array(
            metrics
                .column_by_name(consts::ID)
                .expect("ID column should be present"),
        );

        let metric_length = metric_ids.len();
        // SAFETY: indexing here is safe because we've already ensured that all primary tables are
        // non empty.
        let max_metric_id = metric_ids.values()[metric_length - 1];
        // These are sorted so the max will be at the end.

        // Note that `max_metric_id` can differ from `metric_length` because the values in the ID
        // column can have gaps.

        let batch_len = batch_length(batches);
        if batch_len <= batch_size {
            // We know that this batch is too small to split, so don't bother computing
            // `cumulative_child_counts`.
            batch_size -= batch_len;
            if batch_size == 0 {
                result.push((batch_index, 0..metric_length));
                batch_size = max_output_batch;
            }
        } else {
            child_counts.clear();
            child_counts.resize(max_metric_id as usize + 1, 0);
            for dpt in DATA_POINTS_TYPES {
                let child = batches[POSITION_LOOKUP[dpt as usize]].as_ref();
                if let Some(child) = child {
                    let parent_id: &PrimitiveArray<UInt16Type> = as_primitive_array(
                        child
                            .column_by_name(consts::PARENT_ID)
                            .expect("PARENT_ID column should be present"),
                    );
                    for (count, parent_id) in parent_id.values().iter().dedup_with_count() {
                        let parent_id = *parent_id as usize;
                        child_counts[parent_id] += count as u64;
                    }
                }
            }

            cumulative_child_counts.clear();
            cumulative_child_counts.extend(child_counts.iter().scan(0, |accumulator, &element| {
                *accumulator += element;
                Some(*accumulator)
            }));

            loop {
                // Find the index of the largest element of `cumulative_child_counts` that is
                // smaller than the desired batch size.
                let candidate_index = cumulative_child_counts
                    .partition_point(|&cum_child_count| cum_child_count < batch_size as u64);
                let candidate_count = cumulative_child_counts[candidate_index];
                // Some possibilities:
                // 1. candidate_count == batch_size -> we win!
                // 2. candidate_count < batch_size -> this shouldn't happen since it implies that our early termination in the other branch of the if statement should've fired
                // 3. candidate_count > batch_size && candidate_index==0 -> we overshot because the first entry is bigger than our target batch size
                // 4. candidate_count > batch_size -> we overshot, so try candidate_index-=1

                let starting_index = result
                    .last()
                    .filter(|(other_batch, _)| *other_batch == batch_index)
                    .map(|(_, range)| range.end)
                    .unwrap_or(0);
                let ending_index = (candidate_index + 1).min(metric_length);

                result.push((batch_index, starting_index..ending_index));
                let candidate_size = (candidate_count
                    + cumulative_child_counts
                        .get(candidate_count as usize + 1)
                        .copied()
                        .unwrap_or(0)
                    - cumulative_child_counts[starting_index])
                    as usize;
                batch_size = batch_size.saturating_sub(candidate_size);
                if batch_size == 0 {
                    batch_size = max_output_batch;
                }
                if ending_index >= metric_length {
                    break;
                }
            }
        }
    }
    Ok(result)
}

// Sorting `RecordBatch`es!
// *************************************************************************************************

enum HowToSort {
    SortByParentIdAndId,
    SortById,
}

/// Return a `RecordBatch` lexically sorted by either the `parent_id` column and secondarily by the
/// `id` column or just by the `id` column.
fn sort_record_batch(rb: RecordBatch, how: HowToSort) -> Result<RecordBatch> {
    let (schema, columns, _num_rows) = rb.into_parts();
    let id_column_index = schema.column_with_name(consts::ID).map(|pair| pair.0);
    let parent_id_column_index = schema
        .column_with_name(consts::PARENT_ID)
        .map(|pair| pair.0);

    use arrow::compute::{SortColumn, SortOptions, lexsort_to_indices, take};
    let options = Some(SortOptions {
        descending: false,
        nulls_first: true, // We rely on this heavily later on!
    });
    use HowToSort::*;
    let sort_columns: SmallVec<[SortColumn; 2]> =
        match (how, parent_id_column_index, id_column_index) {
            (SortByParentIdAndId, Some(parent_id), Some(id)) => {
                // FIXME: use row format for faster multicolumn sorts right here
                let parent_id_values = columns[parent_id].clone();
                let id_values = columns[id].clone();
                smallvec::smallvec![
                    SortColumn {
                        values: parent_id_values,
                        options,
                    },
                    SortColumn {
                        values: id_values,
                        options,
                    },
                ]
            }
            (_, None, Some(id)) => {
                let id_values = columns[id].clone();
                smallvec::smallvec![SortColumn {
                    values: id_values,
                    options,
                }]
            }
            _ => unreachable!(),
        };

    let indices = lexsort_to_indices(&sort_columns, None).context(error::BatchingSnafu)?;
    let input_was_already_sorted = indices.values().is_sorted();
    let columns = if input_was_already_sorted {
        // Don't bother with take if the input was already sorted as we need.
        columns
    } else {
        columns
            .iter()
            .map(|c| take(c.as_ref(), &indices, None))
            .collect::<arrow::error::Result<Vec<_>>>()
            .context(error::BatchingSnafu)?
    };

    RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)
}

// Code for merging batches (concatenation)
// *************************************************************************************************

fn generic_concatenate<const N: usize>(
    batches: Vec<[Option<RecordBatch>; N]>,
    allowed_payloads: &[ArrowPayloadType],
    max_output_batch: Option<NonZeroU64>,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    let mut result = Vec::new();

    let mut current = Vec::new();
    let mut current_batch_length = 0;
    for batches in batches {
        let emit_new_batch = max_output_batch
            .map(|max_output_batch| {
                (current_batch_length + batch_length(&batches)) as u64 >= max_output_batch.get()
            })
            .unwrap_or(false);
        if emit_new_batch {
            reindex(&mut current, allowed_payloads)?;
            result.push(generic_schemaless_concatenate(&mut current)?);
            current_batch_length = 0;
            for batches in current.iter() {
                assert_eq!(batches, &[const { None }; N]);
            }
            current.clear();
        } else {
            current_batch_length += batch_length(&batches);
            current.push(batches);
        }
    }

    if !current.is_empty() {
        reindex(&mut current, allowed_payloads)?;
        result.push(generic_schemaless_concatenate(&mut current)?);
        for batches in current.iter() {
            assert_eq!(batches, &[const { None }; N]);
        }
    }
    Ok(result)
}

fn generic_schemaless_concatenate<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
    unify(batches)?;
    let mut result = [const { None }; N];
    for i in 0..N {
        // ignore all the rows where every item is None
        if select(batches, i).next().is_some() {
            let schema = Arc::new(
                Schema::try_merge(select(batches, i).map(|rb| Arc::unwrap_or_clone(rb.schema())))
                    .context(error::BatchingSnafu)?,
            );

            let num_rows = select(batches, i).map(RecordBatch::num_rows).sum();
            let mut batcher = arrow::compute::BatchCoalescer::new(schema.clone(), num_rows);
            for row in batches.iter_mut() {
                if let Some(rb) = row[i].take() {
                    batcher
                        .push_batch(
                            rb.with_schema(schema.clone())
                                .context(error::BatchingSnafu)?,
                        )
                        .context(error::BatchingSnafu)?;
                }
            }
            batcher
                .finish_buffered_batch()
                .context(error::BatchingSnafu)?;
            let concatenated = batcher
                .next_completed_batch()
                .expect("by construction this should never be empty");
            result[i] = Some(concatenated);
        }
    }

    for batches in batches {
        assert_eq!(batches, &[const { None }; N]);
    }
    Ok(result)
}

/// This is basically a transpose view thats lets us look at a sequence of the `i`-th table given a
/// sequence of `RecordBatch` arrays.
fn select<const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    i: usize,
) -> impl Iterator<Item = &RecordBatch> {
    batches.iter().flat_map(move |batches| batches[i].as_ref())
}

// Concatenation requires that we solve two problems: reindexing and unification!

// Reindexing code
// *************************************************************************************************

fn reindex<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
    allowed_payloads: &[ArrowPayloadType],
) -> Result<()> {
    let mut starting_ids: [u32; N] = [0; N];
    for payload in allowed_payloads {
        let child_payloads = child_payload_types(*payload);
        if !child_payloads.is_empty() {
            for batches in batches.iter_mut() {
                let parent_offset = POSITION_LOOKUP[*payload as usize];
                let parent = batches[parent_offset].take();
                if let Some(mut parent) = parent {
                    let parent_starting_offset = starting_ids[parent_offset];

                    // When `parent` has both ID and PARENT_ID columns, resort by ID. Why? Because
                    // the reindexing code requires that the input be sorted. For all these cases,
                    // we've already reindexed by PARENT_ID in an earlier iteration of this loop.
                    if parent.column_by_name(consts::PARENT_ID).is_some()
                        && parent.column_by_name(consts::ID).is_some()
                    {
                        parent = sort_record_batch(parent, HowToSort::SortById)?;
                    }

                    let (parent, next_starting_id) =
                        reindex_record_batch(parent, consts::ID, parent_starting_offset)?;
                    starting_ids[parent_offset] += next_starting_id;
                    // return parent to batches since we took it!
                    let _ = batches[parent_offset].replace(parent);

                    for child in child_payloads {
                        let child_offset = POSITION_LOOKUP[*child as usize];
                        if let Some(child) = batches[child_offset].take() {
                            let (child, next_starting_id) = reindex_record_batch(
                                child,
                                consts::PARENT_ID,
                                parent_starting_offset,
                            )?;
                            starting_ids[child_offset] += next_starting_id;
                            // return child to batches since we took it!
                            let _ = batches[child_offset].replace(child);
                            // We don't have to reindex child's id column since we'll get to it in a
                            // later iteration of the loop if it exists.
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn reindex_record_batch(
    rb: RecordBatch,
    column_name: &'static str,
    mut next_starting_id: u32,
) -> Result<(RecordBatch, u32)> {
    let id = IDColumn::extract(&rb, column_name)?;

    let maybe_new_ids = match id {
        IDColumn::U16(array) => IDRange::<u16>::reindex_column(array, next_starting_id)?,
        IDColumn::U32(array) => IDRange::<u32>::reindex_column(array, next_starting_id)?,
    };

    // Sigh. There doesn't seemt to be a way to mutate the values of a single column of a
    // `RecordBatch` without taking it apart entirely and then putting it back together.
    let (schema, mut columns, _len) = rb.into_parts();

    if let Some((new_id_array, new_next_starting_id)) = maybe_new_ids {
        let column_index = schema
            .fields
            .find(column_name)
            .expect("we already extracted this column")
            .0;
        columns[column_index] = new_id_array;
        next_starting_id = new_next_starting_id;
    }

    Ok((
        RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)?,
        next_starting_id,
    ))
}

/// I describe the indices and values in an ID column for reindexing
struct IDRange<T> {
    indices: RangeInclusive<usize>,
    ids: RangeInclusive<T>,
    is_gap_free: bool,
}

impl<T> IDRange<T> {
    /// Maybe make a new `IDRange` from an array reference
    fn from_generic_array<Native, ArrowPrimitive>(
        array: &PrimitiveArray<ArrowPrimitive>,
    ) -> Option<IDRange<Native>>
    where
        Native: Add<Output = Native> + Copy + PartialEq + PartialOrd + ArrowNativeTypeOp,
        ArrowPrimitive: ArrowPrimitiveType<Native = Native>,
    {
        // Null-handling
        //
        // We're going to rely heavily on the fact that we sorted with nulls first. Why first and not last?
        // Because the more efficient PrimitiveArray.nulls().unwrap().valid_indices() iterator is not double ended
        // unlike PrimitiveArray.nulls().unwrap().iter() which is slower.

        let all_values = array.values();
        let len = array.len();
        use arrow::array::Array;

        let (non_null_slice, indices) = match array.nulls() {
            // There are no nulls at all, so everything is valid!
            None => (&all_values[..], Some(0..=len - 1)),

            // Everything is null, so nothing is valid
            Some(nulls) if nulls.null_count() == len => (&all_values[0..0], None),

            // There are some nulls but also some non-null values, so somethings are valid
            Some(nulls) => {
                // We rely on the fact that we've sorted so that nulls come at the front.

                // SAFETY: unwrap is safe here because we've already verified that the entire array
                // isn't null, which means there has to be at least one valid index which means
                // .next() will return Some the first time we call it.
                let first_valid_index = nulls
                    .valid_indices()
                    .next()
                    .expect("a non-null must be here");
                (
                    &all_values[first_valid_index..],
                    Some(first_valid_index..=len - 1),
                )
            }
        };
        let is_gap_free = non_null_slice
            .windows(2)
            .all(|pair| pair[0] == pair[1] || pair[1] == pair[0] + Native::ONE);

        indices.map(|indices| {
            let ids = non_null_slice[0]..=non_null_slice[non_null_slice.len() - 1];
            IDRange {
                ids,
                indices,
                is_gap_free,
            }
        })
    }

    fn reindex_column<Native, ArrowPrimitive>(
        array: &PrimitiveArray<ArrowPrimitive>,
        next_starting_id: u32,
    ) -> Result<Option<(Arc<dyn Array>, u32)>>
    where
        Native: Add<Output = Native>
            + Copy
            + PartialEq
            + PartialOrd
            + TryFrom<u32>
            + TryFrom<i64>
            + Into<i64>
            + Into<u32>
            + Clone
            + ArrowNativeTypeOp,
        RangeFrom<Native>: Iterator<Item = Native>,
        <Native as TryFrom<u32>>::Error: std::fmt::Debug,
        <Native as TryFrom<i64>>::Error: std::fmt::Debug,
        ArrowPrimitive: ArrowPrimitiveType<Native = Native>,
    {
        if let Some(id_range) = Self::from_generic_array(array) {
            // We do our bounds checking in `i64`-land because the only types we care about are `u32`
            // and `u16` and `i64` can repersent all those values and all offsets between them.
            let start: i64 = (*id_range.ids.start()).into();
            let end: i64 = (*id_range.ids.end()).into();
            let offset = (next_starting_id as i64) - start;
            let do_sub_offset = offset.signum() == -1;
            let offset = offset.abs();

            // If this statement works, then we know that all additions/subtractions will work
            // because we rely on the fact that the slice is sorted, so `start` is the smallest
            // possible value and `end` is the largest possible value.
            let _ = Native::try_from(if do_sub_offset {
                start - offset
            } else {
                end + offset
            })
            .expect("overflow occurred");

            let offset = Native::try_from(offset).expect("this should never happen");

            let array = if id_range.is_gap_free {
                // Whee! We can just do vectorized addition/subtraction!
                let scalar = PrimitiveArray::<ArrowPrimitive>::new_scalar(offset);

                // The normal add/sub kernels check for overflow which we don't want since we've already
                // verified that overflow can't happen, so we use the wrapping variants even though we
                // know no wrapping will occur to avoid the cost of overflow checks.
                let array = if do_sub_offset {
                    arrow::compute::kernels::numeric::sub_wrapping(array, &scalar)
                } else {
                    arrow::compute::kernels::numeric::add_wrapping(array, &scalar)
                };

                // FIXME: downcast_array will panic if the types aren't right; all it is doing is
                // PrimitiveArray<ArrowType>::from(input.data()); maybe try that with try_from instead?
                let array: PrimitiveArray<ArrowPrimitive> = arrow::array::downcast_array(
                    &array.expect("this array is of the expected type"),
                );
                array
            } else {
                // Ugh, there are gaps, so we need to do something slower and more complicated to
                // replace the sequence with a gap free version. This is complicated by the presence of
                // duplciates.
                use itertools::Itertools;

                let null_count = *(id_range.indices.start());
                let valid_ids = &array.values()[id_range.indices];
                let next_starting_id = Native::try_from(next_starting_id)
                    .expect("we can convert next_starting_id to our element type");

                let values = valid_ids
                    .iter()
                    // we convert the original values into a form of run-length encoding
                    .dedup_with_count()
                    // and combine it with a gap-free sequence of new integer values
                    .zip(next_starting_id..)
                    .flat_map(|((count, _old_id), new_id)| {
                        // swapping out old for new values, repeating items as needed
                        repeat_n(new_id, count)
                    });
                if null_count == 0 {
                    PrimitiveArray::from_iter_values(values)
                } else {
                    // First, we start with as many nulls as the original...
                    let nulls = repeat_n(None, null_count);
                    // ...then we add the non-null values
                    nulls.chain(values.map(Some)).collect()
                }
            };

            let last_id: u32 = array.values()[array.len() - 1].into();
            let next_starting_id = last_id.checked_add(1).expect("no overflow");
            Ok(Some((Arc::new(array), next_starting_id)))
        } else {
            Ok(None)
        }
    }
}

// Part of concatenation is unifying batches into a common schema and data...
// *************************************************************************************************

// There are two problems we need to solve when concatenating `RecordBatch`es:
//
// * The `AdaptiveArrayBuilder` code in `encode` will generate columns with different types: either
//   a flat array, a Dict8 array or a Dict16 array. When any dictionary arrays are being used here,
//   we need to convert them to flat arrays.
//
// * Optional columns will be missing entirely; if they're missing from only some batches, we
//   need to add all null columns from the batches where they're not present.

/// Modifies the record batches so that they all have the same set of columns.
///
/// # Arguments
/// * `batches` - this is a 2D array where each element in the outer array contains an inner array
///   and each element in the inner array contains the record batch for a given payload type
fn unify<const N: usize>(batches: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut schemas = Vec::with_capacity(batches.len());

    // FIXME: perhaps this whole function should coalesce operations against the same
    // `RecordBatch`es together? At least investigate the performance cost of calling
    // RecordBatch::into_parts/try_new repeatedly.

    // `field_name_to_batch_indices` maps column name to vector of col-indices
    let mut field_name_to_batch_indices: HashMap<String, HashSet<usize>> = HashMap::new();
    // FIXME: replace sets with a real bitset type, ideally one that stores small sets inline
    // without allocations.

    // `dict_fields` contains state needed to compute the size of the dictionary keys in the
    // unified record batch. This is so we don't overflow the dictionary when combining batches
    let mut dict_fields: BTreeMap<String, UnifiedDictionaryKeySelector> = BTreeMap::new();

    // map of struct columns where, keyed by the struct field name, and the values are a tuple
    // the first element in the tuple is the struct field definition, and the second element is
    // the a map of struct field name -> field definition + maybe dictionary state
    let mut struct_fields: BTreeMap<String, (Field, BTreeMap<String, StructFieldToUnify>)> =
        BTreeMap::new();

    let mut all_batch_indices: HashSet<usize> = HashSet::new();

    for payload_type_index in 0..N {
        schemas.clear(); // We're going to reuse this allocation across loop iterations
        schemas.extend(select(batches, payload_type_index).map(|batch| batch.schema()));

        if batches.is_empty() {
            return Ok(());
        }
        let len = batches.len();

        field_name_to_batch_indices.clear();
        all_batch_indices.clear();

        for (batch_index, schema) in schemas.iter().enumerate() {
            for field in schema.fields.iter() {
                if matches!(field.data_type(), DataType::Struct(_)) {
                    // skip struct fields, as they get unified in a code path that doesn't use the
                    // data structure we're populating here
                    continue;
                }

                let _ = field_name_to_batch_indices
                    .entry(field.name().clone())
                    .or_default()
                    .insert(batch_index);
            }
        }
        (0..len).for_each(|batch_index| {
            let _ = all_batch_indices.insert(batch_index);
        });

        // this will be used to initialize the dictionary key selector. There are certain
        // optimizations that can be made when choosing the correct key sizes using this value
        let total_batch_size = batches
            .iter()
            .filter_map(|batches| batches[payload_type_index].as_ref())
            .map(|rb| rb.num_rows())
            .sum();

        for batches in batches.iter() {
            // try to get the fields that should be in each struct column
            if let Some(rb) = &batches[payload_type_index] {
                try_record_dictionary_fields(rb, total_batch_size, &mut dict_fields)?;
                try_record_struct_fields(rb, total_batch_size, &mut struct_fields)?;
            }
        }

        for batches in batches.iter() {
            if let Some(rb) = &batches[payload_type_index] {
                try_count_dictionary_values(rb, &mut dict_fields)?;
                try_count_struct_dictionary_fields(rb, &mut struct_fields)?;
            }
        }

        // unify all the struct columns
        for batches in batches.iter_mut() {
            if let Some(rb) = batches[payload_type_index].take() {
                let rb = try_unify_dictionary_fields(&rb, &dict_fields)?;
                let rb = try_unify_struct_columns(&rb, &struct_fields)?;
                let _ = batches[payload_type_index].replace(rb);
            }
        }

        // repopulate the list of schemas in case any dictionary datatypes were changed
        schemas.clear();
        schemas.extend(select(batches, payload_type_index).map(|batch| batch.schema()));

        // Let's find missing optional columns; note that this must happen after we deal with the
        // dict columns since we rely on the assumption that all fields with the same name will have
        // the same type.
        for (missing_field_name, present_batch_indices) in field_name_to_batch_indices
            .iter()
            .filter(|(_, cols)| cols.len() != len)
        {
            // All the present columns should have the same Field definition, so just pick the first
            // one arbitrarily; we know there has to be at least one because if there were none, we
            // wouldn't have a mismatch to begin with.
            let field = Arc::new(
                schemas[*present_batch_indices
                    .iter()
                    .next()
                    .expect("there should be at least one schema")]
                .field_with_name(missing_field_name)
                .context(error::BatchingSnafu)?
                .clone(),
            );
            assert!(field.is_nullable());
            for missing_batch_index in all_batch_indices.difference(present_batch_indices).copied()
            {
                if let Some(batch) = batches[missing_batch_index][payload_type_index].take() {
                    let (schema, mut columns, num_rows) = batch.into_parts();
                    let schema = Arc::unwrap_or_clone(schema);
                    let mut builder = SchemaBuilder::from(&schema);
                    builder.push(field.clone());
                    let schema = Arc::new(builder.finish());

                    columns.push(arrow::array::new_null_array(field.data_type(), num_rows));
                    let batch =
                        RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)?;

                    let _ = batches[missing_batch_index][payload_type_index].replace(batch);
                }
            }
        }
    }
    Ok(())
}

struct StructFieldToUnify {
    field: Field,
    dictionary: Option<UnifiedDictionaryKeySelector>,
}

/// This will be used to inspect the values in some dictionary column to determine what key type
/// should be used.
///
/// To do this, it should first visit all the dictionary data types, for the column in every batch
/// whose schema is being unified, so it can decide what is the smallest key type allowed for the
/// column. To do this, call [`Self::visit_key_data_type`].
///
/// Next, it should visit the values, where it will try to calculate the total cardinality across
/// the combined columns until it figures out what key type will fit all the values. To invoke this
/// use the [`Self::visit_dictionary_values`].
struct UnifiedDictionaryKeySelector {
    smallest_key_data_type: DataType,
    total_batch_size: usize,
    total_values_visited: usize,
    state: ahash::RandomState,
    dedup: BTreeSet<u64>,
}

impl UnifiedDictionaryKeySelector {
    fn new(total_batch_size: usize, key_data_type: DataType) -> Self {
        Self {
            total_batch_size,
            smallest_key_data_type: key_data_type,
            total_values_visited: 0,
            state: Default::default(),
            dedup: BTreeSet::new(),
        }
    }

    /// Returns boolean of whether the type of key to use for this dictionary column can be known
    /// based on the available data.
    ///
    /// In the worst case scenario, this needs to look at every value in the dict column of all
    /// record batches whose schema are being unified. However, if certain conditions are met then
    /// we can make an early decision and avoid looking at so many values
    fn key_selected(&self) -> bool {
        // if this is true, we know the values can fit into whatever size dictionary we want
        if self.total_batch_size <= u8::MAX as usize {
            return true;
        }

        // we know the values can fit into a dict keyed by u16 because of the batch size, but
        // we also know there's no chance they'll fit into a u8, so the key decision is made
        if self.total_batch_size <= u16::MAX as usize && self.dedup.len() > u8::MAX as usize {
            return true;
        }

        // if this is true, we know we need to use the native array type
        if self.dedup.len() > u16::MAX as usize {
            return true;
        }

        // calculate the max cardinality based on the total batch size and the number of duplicate
        // values we've visited
        let duplicates_visited = self.total_values_visited - self.dedup.len();
        let possible_max_cardinality = self.total_batch_size - duplicates_visited;

        // if we can determine that the values would fit in the desired key size, we can return early
        if possible_max_cardinality < u16::MAX as usize
            && self.smallest_key_data_type == DataType::UInt16
        {
            return true;
        }

        if possible_max_cardinality <= u8::MAX as usize {
            return true;
        }

        false
    }

    fn visit_key_data_type(&mut self, key_data_type: &DataType) {
        if key_data_type == &DataType::UInt8 {
            self.smallest_key_data_type = DataType::UInt8
        }
    }

    fn visit_dictionary_values(&mut self, maybe_dict_column_arr: &ArrayRef) -> Result<()> {
        if self.key_selected() {
            // we've already seen enough values that we can decide the dictionary key size
            return Ok(());
        }

        // access the array containing the dictionary values ...
        let values_arr = match maybe_dict_column_arr.data_type() {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => {
                    let dict_col = maybe_dict_column_arr
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                        .expect("can cast array to data type");
                    dict_col.values()
                }
                DataType::UInt16 => {
                    let dict_col = maybe_dict_column_arr
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                        .expect("can cast array to data type");
                    dict_col.values()
                }
                key_type => {
                    return Err(error::UnsupportedDictionaryKeyTypeSnafu {
                        expect_oneof: vec![DataType::UInt8, DataType::UInt32],
                        actual: key_type.clone(),
                    }
                    .build());
                }
            },
            _ => {
                // assume it's native column
                maybe_dict_column_arr
            }
        };

        // visit the dictionary values until we decide on the key type ..
        match values_arr.data_type() {
            DataType::Binary => {
                let bin_array = values_arr
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    .expect("can cast array to data type");
                self.visit_bytes_dict_values(bin_array);
            }
            DataType::Utf8 => {
                let str_array = values_arr
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .expect("can cast array to data type");
                self.visit_bytes_dict_values(str_array);
            }
            DataType::FixedSizeBinary(_) => {
                let fsb_array = values_arr
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .expect("can cast array to data type");
                self.visit_fsb_dict_values(fsb_array);
            }
            DataType::UInt16 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<UInt16Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::UInt32 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<UInt32Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::UInt64 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<UInt64Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::Int16 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<Int64Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::Int32 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<Int32Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::Int64 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<Int64Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }
            DataType::Float32 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<Float32Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }

            DataType::Float64 => {
                let prim_array = values_arr
                    .as_any()
                    .downcast_ref::<PrimitiveArray<Float64Type>>()
                    .expect("can cast array to data type");
                self.visit_primitive_values(prim_array);
            }

            value_type => {
                return Err(error::UnsupportedDictionaryValueTypeSnafu {
                    expect_oneof: vec![
                        DataType::Binary,
                        DataType::Utf8,
                        DataType::FixedSizeBinary(16),
                        DataType::FixedSizeBinary(8),
                        DataType::UInt16,
                        DataType::UInt32,
                        DataType::UInt64,
                        DataType::Int16,
                        DataType::Int32,
                        DataType::Int64,
                        DataType::Float32,
                        DataType::Float64,
                    ],
                    actual: value_type.clone(),
                }
                .build());
            }
        };

        Ok(())
    }

    fn visit_bytes_dict_values<T: ByteArrayType>(&mut self, byte_array: &GenericByteArray<T>) {
        for value_native in byte_array.iter().flatten() {
            let value_bytes: &[u8] = value_native.as_ref();
            let hash = self.state.hash_one(value_bytes);
            _ = self.dedup.insert(hash);
            self.total_values_visited += 1;
            if self.key_selected() {
                break;
            }
        }
    }

    fn visit_fsb_dict_values(&mut self, fsb_array: &FixedSizeBinaryArray) {
        for value_bytes in fsb_array.iter().flatten() {
            let hash = self.state.hash_one(value_bytes);
            _ = self.dedup.insert(hash);
            self.total_values_visited += 1;
            if self.key_selected() {
                break;
            }
        }
    }

    fn visit_primitive_values<T: ArrowPrimitiveType>(
        &mut self,
        primitive_array: &PrimitiveArray<T>,
    ) {
        let size = size_of::<T::Native>();
        let values_buffer = primitive_array.values().inner();
        for i in 0..primitive_array.len() {
            if primitive_array.is_null(i) {
                continue;
            }

            let offset = i * size;
            let value_bytes = &values_buffer[offset..offset + size];
            let hash = self.state.hash_one(value_bytes);
            _ = self.dedup.insert(hash);
            self.total_values_visited += 1;
            if self.key_selected() {
                break;
            }
        }
    }

    /// Choose the key type to use for the dictionary. If it returns `None` it means
    /// use the native array type (not a dictionary array)
    fn choose_key_type(&self) -> Option<DataType> {
        if self.dedup.len() <= u8::MAX as usize {
            return Some(self.smallest_key_data_type.clone());
        }

        if self.dedup.len() <= u16::MAX as usize {
            return Some(DataType::UInt16);
        }

        None
    }
}

fn try_record_dictionary_fields(
    record_batch: &RecordBatch,
    total_batch_size: usize,
    dictionary_fields: &mut BTreeMap<String, UnifiedDictionaryKeySelector>,
) -> Result<()> {
    let schema = record_batch.schema_ref();

    for field in schema.fields() {
        if let DataType::Dictionary(dict_key_type, _) = field.data_type() {
            let field_name = field.name();
            if dictionary_fields.contains_key(field.name()) {
                dictionary_fields
                    .get_mut(field.name())
                    // safety: we've just checked that the ma contains the key
                    .expect("can access dict field")
                    .visit_key_data_type(dict_key_type.as_ref());
            } else {
                _ = dictionary_fields.insert(
                    field_name.clone(),
                    UnifiedDictionaryKeySelector::new(
                        total_batch_size,
                        dict_key_type.as_ref().clone(),
                    ),
                );
            }
        }
    }

    Ok(())
}

fn try_count_dictionary_values(
    record_batch: &RecordBatch,
    dictionary_fields: &mut BTreeMap<String, UnifiedDictionaryKeySelector>,
) -> Result<()> {
    for (field_name, dict_key_selector) in dictionary_fields.iter_mut() {
        if let Some(column) = record_batch.column_by_name(field_name) {
            dict_key_selector.visit_dictionary_values(column)?;
        }
    }

    Ok(())
}

fn try_unify_dictionary_fields(
    record_batch: &RecordBatch,
    dictionary_fields: &BTreeMap<String, UnifiedDictionaryKeySelector>,
) -> Result<RecordBatch> {
    let schema = record_batch.schema_ref();
    let mut columns = record_batch.columns().to_vec();
    let mut fields = schema.fields.to_vec();

    for (field_name, dict_key_selector) in dictionary_fields.iter() {
        if let Ok(field_index) = schema.index_of(field_name) {
            let column = &columns[field_index];
            let values_type = match column.data_type() {
                DataType::Dictionary(_, v) => v.as_ref().clone(),
                native => native.clone(),
            };

            // safety: casting the dictionary keys should be infallible here as we're
            // either casting to a native dict (which should be infallible), or we're
            // casting the keys to a size we've calculated will fit
            let new_column = match dict_key_selector.choose_key_type() {
                Some(key_type) => cast(
                    column,
                    &DataType::Dictionary(Box::new(key_type), Box::new(values_type)),
                ),
                None => cast(column, &values_type),
            }
            .expect("can cast dictionary column");

            let new_field = fields[field_index]
                .as_ref()
                .clone()
                .with_data_type(new_column.data_type().clone());
            fields[field_index] = Arc::new(new_field);
            columns[field_index] = new_column;
        }
    }

    // safety: should be safe to expect that building the record batch won't fail here. The schema
    // should match the columns and the columns should all have the correct length
    Ok(RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("can unify dict columns"))
}

fn try_record_struct_fields(
    record_batch: &RecordBatch,
    total_batch_size: usize,
    all_struct_fields: &mut BTreeMap<String, (Field, BTreeMap<String, StructFieldToUnify>)>,
) -> Result<()> {
    for rb_field in record_batch.schema_ref().fields() {
        if let DataType::Struct(struct_fields) = rb_field.data_type() {
            if !all_struct_fields.contains_key(rb_field.name()) {
                _ = all_struct_fields.insert(
                    rb_field.name().clone(),
                    (rb_field.as_ref().clone(), BTreeMap::new()),
                );
            }

            // this is the fields contained within the "field" struct array column on any record
            // batch for which we're unifying the schemas
            let (_, all_this_struct_fields) = all_struct_fields
                .get_mut(rb_field.name())
                .expect("struct fields should be initialized for this field name");

            for struct_field in struct_fields {
                if all_this_struct_fields.get(struct_field.name()).is_none() {
                    let dict_key_selector = match struct_field.data_type() {
                        DataType::Dictionary(dict_key_type, _) => {
                            Some(UnifiedDictionaryKeySelector::new(
                                total_batch_size,
                                dict_key_type.as_ref().clone(),
                            ))
                        }
                        _ => None,
                    };
                    _ = all_this_struct_fields.insert(
                        struct_field.name().clone(),
                        StructFieldToUnify {
                            field: struct_field.as_ref().clone(),
                            dictionary: dict_key_selector,
                        },
                    );
                }
            }
        }
    }

    Ok(())
}

fn try_count_struct_dictionary_fields(
    record_batch: &RecordBatch,
    all_struct_fields_defs: &mut BTreeMap<String, (Field, BTreeMap<String, StructFieldToUnify>)>,
) -> Result<()> {
    for (rb_field_name, (_, desired_struct_fields)) in all_struct_fields_defs {
        let struct_arr = match record_batch.column_by_name(rb_field_name) {
            Some(column) => column
                .as_any()
                .downcast_ref::<StructArray>()
                .with_context(|| error::InvalidListArraySnafu {
                    expect_oneof: vec![DataType::Struct(Fields::empty())],
                    actual: column.data_type().clone(),
                })?,
            None => {
                // the struct field isn't contained in this record batch, so we don't need to count
                // the values in its dictionary columns
                continue;
            }
        };

        for (struct_field_name, struct_field) in desired_struct_fields {
            let dict_key_selector = match struct_field.dictionary.as_mut() {
                Some(dict_key_selector) => dict_key_selector,
                // skip field if it's not a dict field
                None => continue,
            };

            if let Some(column) = struct_arr.column_by_name(struct_field_name) {
                dict_key_selector.visit_dictionary_values(column)?;
            }
        }
    }

    Ok(())
}

fn try_unify_struct_columns(
    record_batch: &RecordBatch,
    all_struct_fields_defs: &BTreeMap<String, (Field, BTreeMap<String, StructFieldToUnify>)>,
) -> Result<RecordBatch> {
    let schema = record_batch.schema_ref();
    let mut rb_fields = schema.fields.to_vec();
    let mut rb_columns = record_batch.columns().to_vec();

    for (rb_field_name, (rb_field, desired_struct_fields)) in all_struct_fields_defs {
        match schema.index_of(rb_field_name) {
            Ok(rb_field_index) => {
                let field = schema.field(rb_field_index);

                if let DataType::Struct(_) = field.data_type() {
                    // safety: we've just checked the column's data type
                    let rb_column = rb_columns[rb_field_index]
                        .as_any()
                        .downcast_ref::<StructArray>()
                        .expect("expect can downcast to struct");
                    let new_rb_column = try_unify_struct_fields(rb_column, desired_struct_fields)?;

                    let new_field = Arc::new(
                        field
                            .clone()
                            .with_data_type(new_rb_column.data_type().clone()),
                    );
                    rb_fields[rb_field_index] = new_field;
                    rb_columns[rb_field_index] = Arc::new(new_rb_column);
                }
            }

            Err(_) => {
                let len = record_batch.num_rows();
                let struct_fields = Fields::from(
                    desired_struct_fields
                        .values()
                        .map(|s| &s.field)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                let struct_columns = desired_struct_fields
                    .values()
                    .map(|s| arrow::array::new_null_array(s.field.data_type(), len))
                    .collect::<Vec<_>>();
                let struct_nulls = rb_field
                    .is_nullable()
                    .then(|| NullBuffer::from_iter(repeat_n(false, len)));
                let new_rb_column = StructArray::new(struct_fields, struct_columns, struct_nulls);
                let new_rb_field = Field::new(
                    rb_field.name(),
                    new_rb_column.data_type().clone(),
                    rb_field.is_nullable(),
                );
                rb_fields.push(Arc::new(new_rb_field));
                rb_columns.push(Arc::new(new_rb_column));
            }
        }
    }

    Ok(
        // Safety: here we should have an array of fields that match the types in the columns
        // and all the columns are same length, so it's safe to expect here
        RecordBatch::try_new(Arc::new(Schema::new(rb_fields)), rb_columns)
            .expect("could not new record batch with unified struct columns"),
    )
}

fn try_unify_struct_fields(
    current_array: &StructArray,
    desired_fields: &BTreeMap<String, StructFieldToUnify>,
) -> Result<StructArray> {
    let mut new_fields = Vec::with_capacity(desired_fields.len());
    let mut new_columns = Vec::with_capacity(desired_fields.len());
    let array_len = current_array.len();
    let curr_fields = current_array.fields();
    for (field_name, desired_struct_field) in desired_fields {
        // determine the correct data type for the column
        let data_type = match &desired_struct_field.dictionary {
            // if it's a dictionary column, use the key selector + the values type
            // to compute the type
            Some(dict_key_selector) => {
                let dict_values_type = match desired_struct_field.field.data_type() {
                    DataType::Dictionary(_, v) => v.as_ref().clone(),
                    native => native.clone(),
                };
                match dict_key_selector.choose_key_type() {
                    Some(key_type) => {
                        DataType::Dictionary(Box::new(key_type), Box::new(dict_values_type))
                    }
                    None => dict_values_type,
                }
            }

            // it's not a dictionary column, so the value type will just be the type specified
            // by the original schema
            None => desired_struct_field.field.data_type().clone(),
        };

        new_fields.push(
            desired_struct_field
                .field
                .clone()
                .with_data_type(data_type.clone()),
        );

        match curr_fields.find(field_name) {
            Some((field_index, current_field)) => {
                let current_column = current_array.column(field_index).clone();
                let new_column = match current_field.data_type() {
                    DataType::Dictionary(_, _) => {
                        // safety: casting the dictionary keys should be infallible here as we're
                        // either casting to a native dict (which should be infallible), or we're
                        // casting the keys to a size we've calculated will fit
                        cast(&current_column, &data_type).expect("can cast dictionary column")
                    }
                    _ => current_column,
                };

                new_columns.push(new_column);
            }

            None => {
                // create an all null array with the desired type
                let new_struct_column =
                    arrow::array::new_null_array(desired_struct_field.field.data_type(), array_len);
                new_columns.push(new_struct_column);
            }
        }
    }

    Ok(StructArray::new(
        Fields::from(new_fields),
        new_columns,
        current_array.nulls().cloned(),
    ))
}

#[cfg(test)]
mod test {
    use arrow::array::record_batch;
    use arrow::array::{
        ArrayRef, DictionaryArray, FixedSizeBinaryArray, Int32Array, Int64Array, RecordBatch,
        StringArray, StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt64Array,
    };
    use arrow::datatypes::{ArrowDictionaryKeyType, DataType, Field, Schema, TimeUnit, UInt8Type};
    use arrow_schema;

    use crate::otlp::metrics::MetricType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_unify_null_filling() {
        let a = record_batch!(
            ("id", Int32, [1, 2, 3]),
            ("b", Float64, [Some(4.0), Some(5.0), Some(6.0)])
        );
        let b = record_batch!(("id", Int32, [4, 5, 6]));

        let mut batches: [[Option<RecordBatch>; 1]; 2] = [[Some(a.unwrap())], [Some(b.unwrap())]];

        unify(&mut batches).unwrap();
        assert_eq!(
            batches[0][0],
            Some(
                record_batch!(
                    ("id", Int32, [1, 2, 3]),
                    ("b", Float64, [Some(4.0), Some(5.0), Some(6.0)])
                )
                .unwrap()
            )
        );
        assert_eq!(
            batches[1][0],
            Some(
                record_batch!(("id", Int32, [4, 5, 6]), ("b", Float64, [None, None, None]))
                    .unwrap()
            )
        )
    }

    #[test]
    fn test_unify_dict_handling_upgrades_keys_u8_to_u16() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        )]));

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{i}")),
                )),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{}", i + 100)),
                )),
            ))],
        )
        .unwrap();

        // add an empty batch to ensure we'll add an empty dict array as well
        let rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(0..200))],
        )
        .unwrap();

        // now we have created two record batches with a total of 300 different values across two
        // dictionaries keyed by u8. Test if the unify code will recognize that this won't fit in
        // a u8 keyed dictionary if they were combined, so we need to upgrade the key type ...
        let mut batches: [[Option<RecordBatch>; 1]; 3] = [[Some(rb_a)], [Some(rb_b)], [Some(rb_c)]];
        unify(&mut batches).unwrap();

        let expected_schema = Arc::new(Schema::new(vec![
            Field::new(
                "f1",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("f2", DataType::Int32, true),
        ]));

        let expected_rb_a = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values(0..200),
                    Arc::new(StringArray::from_iter_values(
                        (0..200).map(|i| format!("{i}")),
                    )),
                )),
                Arc::new(Int32Array::new_null(200)),
            ],
        )
        .unwrap();

        let expected_rb_b = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values(0..200),
                    Arc::new(StringArray::from_iter_values(
                        (0..200).map(|i| format!("{}", i + 100)),
                    )),
                )),
                Arc::new(Int32Array::new_null(200)),
            ],
        )
        .unwrap();

        let expected_rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new(
                    "f1",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(0..200)),
                Arc::new(DictionaryArray::new(
                    UInt16Array::new_null(200),
                    Arc::new(StringArray::from_iter_values(Vec::<String>::new())),
                )),
            ],
        )
        .unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
        assert_eq!(batches[2][0].as_ref().unwrap(), &expected_rb_c);
    }

    #[test]
    fn test_unify_dict_keeps_u8_if_fits() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        )]));

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{i}")),
                )),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{}", i + 20)),
                )),
            ))],
        )
        .unwrap();

        let rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(0..200))],
        )
        .unwrap();

        // now we have created two record batches with a total of 220 different values across two
        // dictionaries keyed by u8. Test if the unify code will recognize that this will fit in
        // a u8 keyed dictionary if they were combined, so no changes needed
        let mut batches: [[Option<RecordBatch>; 1]; 3] =
            [[Some(rb_a.clone())], [Some(rb_b.clone())], [Some(rb_c)]];
        unify(&mut batches).unwrap();

        let expected_schema = Arc::new(Schema::new(vec![
            Field::new(
                "f1",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("f2", DataType::Int32, true),
        ]));

        let expected_rb_a = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values(0..200),
                    Arc::new(StringArray::from_iter_values(
                        (0..200).map(|i| format!("{i}")),
                    )),
                )),
                Arc::new(Int32Array::new_null(200)),
            ],
        )
        .unwrap();

        let expected_rb_b = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values(0..200),
                    Arc::new(StringArray::from_iter_values(
                        (0..200).map(|i| format!("{}", i + 20)),
                    )),
                )),
                Arc::new(Int32Array::new_null(200)),
            ],
        )
        .unwrap();

        let expected_rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new(
                    "f1",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(0..200)),
                Arc::new(DictionaryArray::new(
                    UInt8Array::new_null(200),
                    Arc::new(StringArray::from_iter_values(Vec::<String>::new())),
                )),
            ],
        )
        .unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
        assert_eq!(batches[2][0].as_ref().unwrap(), &expected_rb_c);
    }

    #[test]
    fn test_unify_dict_handling_upgrades_keys_u16_to_native() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            true,
        )]));

        let num_rows = 40000;

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..num_rows),
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{i}")),
                )),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..num_rows),
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{}", i as u32 + 30000)),
                )),
            ))],
        )
        .unwrap();

        // add an empty batch to ensure we'll add an empty dict array as well
        let rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(0..num_rows as i32))],
        )
        .unwrap();

        // now we have created two record batches with a total of 70000 different values across two
        // dictionaries keyed by u16. Test if the unify code will recognize that this won't fit in
        // a u16 keyed dictionary if they were combined, so we need to upgrade the key type ...
        let mut batches: [[Option<RecordBatch>; 1]; 3] = [[Some(rb_a)], [Some(rb_b)], [Some(rb_c)]];
        unify(&mut batches).unwrap();

        let expected_schema = Arc::new(Schema::new(vec![
            Field::new("f1", DataType::Utf8, true),
            Field::new("f2", DataType::Int32, true),
        ]));

        let expected_rb_a = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{i}")),
                )),
                Arc::new(Int32Array::new_null(num_rows as usize)),
            ],
        )
        .unwrap();

        let expected_rb_b = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{}", i as u32 + 30000)),
                )),
                Arc::new(Int32Array::new_null(num_rows as usize)),
            ],
        )
        .unwrap();

        let expected_rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new("f1", DataType::Utf8, true),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(0..num_rows as i32)),
                Arc::new(StringArray::new_null(num_rows as usize)),
            ],
        )
        .unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
        assert_eq!(batches[2][0].as_ref().unwrap(), &expected_rb_c);
    }

    #[test]
    fn test_unify_dict_keeps_u16_if_fits() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            true,
        )]));

        let num_rows = 40000;

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..num_rows),
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{i}")),
                )),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..num_rows),
                Arc::new(StringArray::from_iter_values(
                    (0..num_rows).map(|i| format!("{}", i + 10000)),
                )),
            ))],
        )
        .unwrap();

        // add an empty batch to ensure we'll add an empty dict array as well
        let rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(0..num_rows as i32))],
        )
        .unwrap();

        // now we have created two record batches with a total of 50000 different values across two
        // dictionaries keyed by u16. Test if the unify code will recognize that this will fit in
        // a u16 keyed dictionary if they were combined, so we can keep the original type ..
        let mut batches: [[Option<RecordBatch>; 1]; 3] = [[Some(rb_a)], [Some(rb_b)], [Some(rb_c)]];
        unify(&mut batches).unwrap();

        let expected_schema = Arc::new(Schema::new(vec![
            Field::new(
                "f1",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("f2", DataType::Int32, true),
        ]));

        let expected_rb_a = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values(0..num_rows),
                    Arc::new(StringArray::from_iter_values(
                        (0..num_rows).map(|i| format!("{i}")),
                    )),
                )),
                Arc::new(Int32Array::new_null(num_rows as usize)),
            ],
        )
        .unwrap();

        let expected_rb_b = RecordBatch::try_new(
            expected_schema.clone(),
            vec![
                Arc::new(DictionaryArray::new(
                    UInt16Array::from_iter_values(0..num_rows),
                    Arc::new(StringArray::from_iter_values(
                        (0..num_rows).map(|i| format!("{}", i + 10000)),
                    )),
                )),
                Arc::new(Int32Array::new_null(num_rows as usize)),
            ],
        )
        .unwrap();

        let expected_rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new(
                    "f1",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(0..num_rows as i32)),
                Arc::new(DictionaryArray::new(
                    UInt16Array::new_null(num_rows as usize),
                    Arc::new(StringArray::from_iter_values(Vec::<String>::new())),
                )),
            ],
        )
        .unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
        assert_eq!(batches[2][0].as_ref().unwrap(), &expected_rb_c);
    }

    #[test]
    fn test_unify_dict_handling_does_not_downgrade_u16_keys() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            true,
        )]));

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{i}")),
                )),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(0..200),
                Arc::new(StringArray::from_iter_values(
                    (0..200).map(|i| format!("{}", i + 20)),
                )),
            ))],
        )
        .unwrap();

        // now we have created two record batches with a total of 220 different values across two
        // dictionaries keyed by u16. Test if the unify code will recognize that, although this
        // could fit in a u8 keyed dictionary, b/c we never saw a u8 keyed dict in the inputs
        // we assume that we need to keep the dict size as u16
        let mut batches: [[Option<RecordBatch>; 1]; 2] =
            [[Some(rb_a.clone())], [Some(rb_b.clone())]];
        unify(&mut batches).unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &rb_b);
    }

    #[test]
    fn test_unify_structs_adds_missing_fields() {
        let field_1 = Field::new("f1", DataType::UInt8, true);
        let field_2 = Field::new("f2", DataType::UInt8, true);

        let rb_a = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "s1",
                DataType::Struct(vec![field_1.clone()].into()),
                true,
            )])),
            vec![Arc::new(StructArray::new(
                vec![field_1.clone()].into(),
                vec![Arc::new(UInt8Array::from_iter_values(vec![1, 2, 3]))],
                None,
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "s1",
                DataType::Struct(vec![field_2.clone()].into()),
                true,
            )])),
            vec![Arc::new(StructArray::new(
                vec![field_2.clone()].into(),
                vec![Arc::new(UInt8Array::from_iter_values(vec![4, 5, 6]))],
                None,
            ))],
        )
        .unwrap();

        let mut batches: [[Option<RecordBatch>; 1]; 2] = [[Some(rb_a)], [Some(rb_b)]];
        unify(&mut batches).unwrap();

        let expected_fields = Fields::from(vec![field_1, field_2]);
        let expected_schema = Arc::new(Schema::new(vec![Field::new(
            "s1",
            DataType::Struct(expected_fields.clone()),
            true,
        )]));

        let expected_rb_a = RecordBatch::try_new(
            expected_schema.clone(),
            vec![Arc::new(StructArray::new(
                expected_fields.clone(),
                vec![
                    Arc::new(UInt8Array::from_iter_values(vec![1, 2, 3])),
                    Arc::new(UInt8Array::from_iter(vec![None, None, None])),
                ],
                None,
            ))],
        )
        .unwrap();

        let expected_rb_b = RecordBatch::try_new(
            expected_schema.clone(),
            vec![Arc::new(StructArray::new(
                expected_fields.clone(),
                vec![
                    Arc::new(UInt8Array::from_iter(vec![None, None, None])),
                    Arc::new(UInt8Array::from_iter_values(vec![4, 5, 6])),
                ],
                None,
            ))],
        )
        .unwrap();

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
    }

    #[test]
    fn test_unify_structs_handles_missing_struct() {
        // test that for nullable structs, if the entire struct is missing then we add a struct
        // where all the structs = null, otherwise we add a list of non null struct where all the
        // fields in the struct are null
        let field = Field::new("f1", DataType::UInt8, true);
        let schema = Arc::new(Schema::new(vec![
            Field::new("s1", DataType::Struct(vec![field.clone()].into()), true),
            Field::new("s2", DataType::Struct(vec![field.clone()].into()), false),
        ]));

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StructArray::new(
                    vec![field.clone()].into(),
                    vec![Arc::new(UInt8Array::from_iter_values(vec![1, 2, 3]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    vec![field.clone()].into(),
                    vec![Arc::new(UInt8Array::from_iter_values(vec![4, 5, 6]))],
                    None,
                )),
            ],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(vec![1, 2, 3]))],
        )
        .unwrap();

        let mut batches: [[Option<RecordBatch>; 1]; 2] = [[Some(rb_a.clone())], [Some(rb_b)]];
        unify(&mut batches).unwrap();

        let expected_rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new("s1", DataType::Struct(vec![field.clone()].into()), true),
                Field::new("s2", DataType::Struct(vec![field.clone()].into()), false),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(StructArray::new(
                    vec![field.clone()].into(),
                    vec![Arc::new(UInt8Array::from_iter(vec![None, None, None]))],
                    Some(NullBuffer::from_iter(vec![false, false, false])),
                )),
                Arc::new(StructArray::new(
                    vec![field.clone()].into(),
                    vec![Arc::new(UInt8Array::from_iter(vec![None, None, None]))],
                    None,
                )),
            ],
        )
        .unwrap();

        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
    }

    #[test]
    fn test_unify_struct_adds_missing_dict_columns() {
        let struct_field = Field::new(
            "f1",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            true,
        );

        let schema = Arc::new(Schema::new(vec![Field::new(
            "s1",
            DataType::Struct(vec![struct_field.clone()].into()),
            false,
        )]));

        let rb_a = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(StructArray::new(
                vec![struct_field.clone()].into(),
                vec![Arc::new(DictionaryArray::<UInt8Type>::from_iter(vec![
                    Some("a"),
                    Some("b"),
                ]))],
                None,
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("f2", DataType::Int32, true)])),
            vec![Arc::new(Int32Array::from_iter_values(vec![1, 2]))],
        )
        .unwrap();

        let mut batches: [[Option<RecordBatch>; 1]; 2] = [[Some(rb_a.clone())], [Some(rb_b)]];
        unify(&mut batches).unwrap();

        let expected_rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("f2", DataType::Int32, true),
                Field::new(
                    "s1",
                    DataType::Struct(vec![struct_field.clone()].into()),
                    false,
                ),
            ])),
            vec![
                Arc::new(Int32Array::from_iter_values(vec![1, 2])),
                Arc::new(StructArray::new(
                    Fields::from(vec![struct_field.clone()]),
                    vec![Arc::new(DictionaryArray::new(
                        UInt8Array::new_null(2),
                        Arc::new(StringArray::from_iter_values(Vec::<String>::new())),
                    ))],
                    None,
                )),
            ],
        )
        .unwrap();

        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
    }

    #[test]
    fn test_unify_struct_can_upgrade_dict_columns() {
        fn gen_struct_rb<T: ArrowDictionaryKeyType + ArrowPrimitiveType>(
            val_offset: usize,
        ) -> RecordBatch
        where
            T::Native: From<u8>,
        {
            let struct_field = Field::new(
                "f1",
                DataType::Dictionary(Box::new(T::DATA_TYPE), Box::new(DataType::Utf8)),
                true,
            );
            let schema = Arc::new(Schema::new(vec![Field::new(
                "f1",
                DataType::Struct(Fields::from(vec![struct_field.clone()])),
                true,
            )]));

            RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(StructArray::new(
                    Fields::from(vec![struct_field.clone()]),
                    vec![Arc::new(DictionaryArray::new(
                        PrimitiveArray::<T>::from_iter_values((0u8..200).map(T::Native::from)),
                        Arc::new(StringArray::from_iter_values(
                            (0..200).map(|i| format!("{}", i + val_offset)),
                        )),
                    ))],
                    None,
                ))],
            )
            .unwrap()
        }

        let rb_a = gen_struct_rb::<UInt8Type>(0);
        let rb_b = gen_struct_rb::<UInt8Type>(100);

        // now we have created two record batches with a total of 300 different values across two
        // dictionaries keyed by u8. Test if the unify code will recognize that this won't fit in
        // a u8 keyed dictionary if they were combined, so we need to upgrade the key type.
        let mut batches: [[Option<RecordBatch>; 1]; 2] = [[Some(rb_a)], [Some(rb_b)]];
        unify(&mut batches).unwrap();

        let expected_rb_a = gen_struct_rb::<UInt16Type>(0);
        let expected_rb_b = gen_struct_rb::<UInt16Type>(100);

        assert_eq!(batches[0][0].as_ref().unwrap(), &expected_rb_a);
        assert_eq!(batches[1][0].as_ref().unwrap(), &expected_rb_b);
    }

    #[test]
    fn unifying_dicts_can_downgrade_from_u16_and_native() {
        let rb_a = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "a",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            )])),
            vec![Arc::new(DictionaryArray::new(
                UInt8Array::from_iter_values(vec![0, 1, 2]),
                Arc::new(StringArray::from_iter_values(vec!["a", "b", "c"])),
            ))],
        )
        .unwrap();

        let rb_b = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "a",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                false,
            )])),
            vec![Arc::new(DictionaryArray::new(
                UInt16Array::from_iter_values(vec![0, 1, 2]),
                Arc::new(StringArray::from_iter_values(vec!["d", "e", "f"])),
            ))],
        )
        .unwrap();

        let rb_c = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("a", DataType::Utf8, false)])),
            vec![Arc::new(StringArray::from_iter_values(vec!["g", "h", "i"]))],
        )
        .unwrap();

        let mut batches: [[Option<RecordBatch>; 1]; 3] = [[Some(rb_a)], [Some(rb_b)], [Some(rb_c)]];
        unify(&mut batches).unwrap();

        fn gen_expected(str_vals: Vec<&str>) -> RecordBatch {
            RecordBatch::try_new(
                Arc::new(Schema::new(vec![Field::new(
                    "a",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                )])),
                vec![Arc::new(DictionaryArray::new(
                    UInt8Array::from_iter_values(vec![0, 1, 2]),
                    Arc::new(StringArray::from_iter_values(str_vals)),
                ))],
            )
            .unwrap()
        }

        assert_eq!(
            batches[0][0].as_ref().unwrap(),
            &gen_expected(vec!["a", "b", "c"])
        );
        assert_eq!(
            batches[1][0].as_ref().unwrap(),
            &gen_expected(vec!["d", "e", "f"])
        );
        assert_eq!(
            batches[2][0].as_ref().unwrap(),
            &gen_expected(vec!["g", "h", "i"])
        );
    }

    fn make_logs() -> OtapArrowRecords {
        let rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, true),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(
                        vec![
                            Field::new(consts::ID, DataType::UInt16, true),
                            Field::new(
                                "schema_url",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "scope",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "name",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "observed_time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "severity_number",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
            ])),
            vec![
                // id
                Arc::new(UInt16Array::from_iter(vec![Some(0), None, Some(1)])),
                // resource
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // resource.id
                        Arc::new(UInt16Array::from(vec![0, 0, 1])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "schema_url",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // resource.schema_url
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0, 0, 0]),
                            Arc::new(StringArray::from_iter_values(vec![
                                "https://schema.opentelemetry.io/resource_schema",
                            ])),
                        )) as ArrayRef,
                    ),
                ])),
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // scope.id
                        Arc::new(UInt16Array::from(vec![0, 1, 2])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "name",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.name
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0, 1, 0]),
                            Arc::new(StringArray::from(vec!["scope", "scope2"])),
                        )) as ArrayRef,
                    ),
                ])),
                // timestamps
                Arc::new(TimestampNanosecondArray::from(vec![0, 0, 0])),
                // observed_time_unix_nano
                Arc::new(TimestampNanosecondArray::from(vec![0i64, 0, 0])) as ArrayRef,
                // severity_number
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0, 1, 0]),
                    Arc::new(Int32Array::from(vec![5, 9, 5])),
                )) as ArrayRef,
            ],
        )
        .unwrap();
        let rb = sort_record_batch(rb, HowToSort::SortByParentIdAndId).unwrap();
        let mut batches: [Option<RecordBatch>; Logs::COUNT] = [const { None }; Logs::COUNT];
        batches[POSITION_LOOKUP[ArrowPayloadType::Logs as usize]] = Some(rb);
        OtapArrowRecords::Logs(Logs { batches })
    }

    #[test]
    fn test_simple_split_logs() {
        let [logs, _, _] = RecordsGroup::split_by_type(vec![make_logs()]);
        let original_logs = logs.clone();
        let split = logs.split(NonZeroU64::new(2).unwrap()).unwrap();
        assert_eq!(split.len(), 2);
        let [a, b] = split.into_otap_arrow_records().try_into().unwrap();
        assert_eq!(a.batch_length(), 2);
        assert_eq!(b.batch_length(), 1);

        let [logs, _, _] = RecordsGroup::split_by_type(vec![a, b]);
        let logs2 = logs.clone();
        let merged = logs.concatenate(Some(NonZeroU64::new(4).unwrap())).unwrap();
        let merged2 = logs2.concatenate(None).unwrap();
        assert_eq!(merged, merged2);
        assert_eq!(merged, original_logs);
    }

    fn make_traces() -> OtapArrowRecords {
        let spans_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, true),
                Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), false),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 2, 3])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        [1, 2, 3, 4].into_iter().map(u64::to_be_bytes),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let span_links_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![0, 1, 2, 3])),
                // create a parent ID range here where not all values of the parent's ID are
                // present to test the range is successfully handled when splitting child
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 1, 2])),
            ],
        )
        .unwrap();

        let span_events_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![0, 1, 2])),
                // create a range where all the ID values are only present in one split to test
                // that the other ranges will not contain a record batch for this payload type
                Arc::new(UInt16Array::from_iter_values(vec![3, 3, 3])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Traces(Traces::default());
        otap_batch.set(ArrowPayloadType::Spans, spans_rb);
        otap_batch.set(ArrowPayloadType::SpanLinks, span_links_rb);
        otap_batch.set(ArrowPayloadType::SpanEvents, span_events_rb);

        otap_batch
    }

    #[test]
    fn test_simple_split_traces() {
        let input = make_traces();
        let [_, _, traces] = RecordsGroup::split_by_type(vec![make_traces().clone()]);
        let split = traces.split(NonZeroU64::new(2).unwrap()).unwrap();

        let otap_batches = match split {
            RecordsGroup::Traces(batches) => batches,
            _ => {
                panic!("split returned wrong type of record group. Expecting traces")
            }
        };

        assert_eq!(otap_batches.len(), 2);

        let input_spans = input.get(ArrowPayloadType::Spans).unwrap();
        let input_span_links = input.get(ArrowPayloadType::SpanLinks).unwrap();
        let input_span_events = input.get(ArrowPayloadType::SpanEvents).unwrap();

        let batch0 = OtapArrowRecords::Traces(Traces {
            batches: otap_batches[0].clone(),
        });
        let batch0_spans = batch0.get(ArrowPayloadType::Spans).unwrap();
        assert_eq!(batch0_spans, &input_spans.slice(0, 2));
        let batch0_span_links = batch0.get(ArrowPayloadType::SpanLinks).unwrap();
        assert_eq!(batch0_span_links, &input_span_links.slice(0, 3));
        let batch0_span_events = batch0.get(ArrowPayloadType::SpanEvents);
        assert!(batch0_span_events.is_none());

        let batch1 = OtapArrowRecords::Traces(Traces {
            batches: otap_batches[1].clone(),
        });
        let batch1_spans = batch1.get(ArrowPayloadType::Spans).unwrap();
        assert_eq!(batch1_spans, &input_spans.slice(2, 2));
        let batch1_span_links = batch1.get(ArrowPayloadType::SpanLinks).unwrap();
        assert_eq!(batch1_span_links, &input_span_links.slice(3, 1));
        let batch1_span_events = batch1.get(ArrowPayloadType::SpanEvents).unwrap();
        // batch 1 events only contained parent IDs from the second spans batch:
        assert_eq!(batch1_span_events, input_span_events);
    }

    fn make_metrics() -> OtapArrowRecords {
        let metrics_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, true),
                Field::new(consts::METRIC_TYPE, DataType::UInt8, false),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 2])),
                Arc::new(UInt8Array::from_iter_values(vec![
                    MetricType::Gauge as u8,
                    MetricType::Gauge as u8,
                    MetricType::Summary as u8,
                ])),
            ],
        )
        .unwrap();

        let number_dp_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ID, DataType::UInt32, false),
                Field::new(consts::INT_VALUE, DataType::Int64, false),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0, 0, 1, 1])),
                Arc::new(UInt32Array::from_iter_values(vec![0, 1, 2, 3])),
                Arc::new(Int64Array::from_iter_values(vec![30, 50, 40, 60])),
            ],
        )
        .unwrap();

        let summary_db_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ID, DataType::UInt32, false),
                Field::new(consts::SUMMARY_COUNT, DataType::UInt64, false),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![2, 2, 2, 2])),
                Arc::new(UInt32Array::from_iter_values(vec![0, 1, 2, 3])),
                Arc::new(UInt64Array::from_iter_values(vec![8, 9, 10, 11])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::default());
        otap_batch.set(ArrowPayloadType::UnivariateMetrics, metrics_rb);
        otap_batch.set(ArrowPayloadType::NumberDataPoints, number_dp_rb);
        otap_batch.set(ArrowPayloadType::SummaryDataPoints, summary_db_rb);

        otap_batch
    }

    // ignoring testing metrics for now. It seems like there's an issue where we subtract with
    // underflow when calculating the splits.
    #[test]
    #[ignore = "this test currently does not pass"]
    fn test_simple_split_metrics() {
        let [_, metrics, _] = RecordsGroup::split_by_type(vec![make_metrics()]);

        let _original_metrics = metrics.clone();
        let _split = metrics.split(NonZeroU64::new(2).unwrap()).unwrap();
        todo!("assert results")
    }
}
