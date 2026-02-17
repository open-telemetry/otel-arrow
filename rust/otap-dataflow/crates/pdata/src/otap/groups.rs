// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Support for splitting and merging sequences of `OtapArrowRecords` in support of batching.
use std::{
    iter::{once, repeat, repeat_n},
    num::{NonZeroU32, NonZeroU64},
    ops::{Add, Range, RangeInclusive},
};

use crate::{
    otap::{
        DATA_POINTS_TYPES, Logs, Metrics, OtapArrowRecords, OtapBatchStore, POSITION_LOOKUP,
        Traces, child_payload_types,
        error::{Error, Result},
        num_items,
        transform::sort_to_indices,
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::consts,
};
use arrow::{
    array::{
        Array, ArrowPrimitiveType, PrimitiveArray, RecordBatch, UInt16Array, UInt32Array,
        as_primitive_array,
    },
    datatypes::{ArrowNativeTypeOp, DataType, UInt16Type},
};
use itertools::Itertools;
use otap_df_config::SignalType;
use smallvec::SmallVec;

use super::transform::{concatenate::concatenate, split};

/// Represents a sequence of OtapArrowRecords that all share exactly
/// the same signal.  Invarients:
///
/// - the data has num_items() >= 1
/// - the primary table (Spans, LogRecords, UnivariateMetrics) has >= 1 rows
///
/// The higher-level component is expected to check for empty payloads.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RecordsGroup {
    /// OTAP logs
    Logs(Vec<[Option<RecordBatch>; Logs::COUNT]>),
    /// OTAP metrics
    Metrics(Vec<[Option<RecordBatch>; Metrics::COUNT]>),
    /// OTAP traces
    Traces(Vec<[Option<RecordBatch>; Traces::COUNT]>),
}

impl RecordsGroup {
    /// Convert a sequence of `OtapArrowRecords` into three `RecordsGroup` objects.
    /// This is a sanity check. In practice, we expect the higher-level batching
    /// component to separate data by signal type. The public APIs for separating
    /// by expected signal type enforce this.
    #[must_use]
    fn separate_by_type(records: Vec<OtapArrowRecords>) -> [Self; 3] {
        let log_count = signal_count(&records, SignalType::Logs);
        let mut log_records = Vec::with_capacity(log_count);

        let metric_count = signal_count(&records, SignalType::Metrics);
        let mut metric_records = Vec::with_capacity(metric_count);

        let trace_count = signal_count(&records, SignalType::Traces);
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

    /// Separate, expecting only logs.
    pub(crate) fn separate_logs(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !metrics.is_empty() || !traces.is_empty() {
            Err(Error::MixedSignals)
        } else {
            Ok(logs)
        }
    }

    /// Separate, expecting only metrics.
    pub(crate) fn separate_metrics(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !logs.is_empty() || !traces.is_empty() {
            Err(Error::MixedSignals)
        } else {
            Ok(metrics)
        }
    }

    /// Separate, expecting only traces.
    pub(crate) fn separate_traces(records: Vec<OtapArrowRecords>) -> Result<Self> {
        let [logs, metrics, traces] = RecordsGroup::separate_by_type(records);
        if !logs.is_empty() || !metrics.is_empty() {
            Err(Error::MixedSignals)
        } else {
            Ok(traces)
        }
    }

    /// Split `RecordBatch`es as needed when they're larger than our threshold or when we need them in
    /// smaller pieces to concatenate together into our target size.
    pub(crate) fn split(self, max_items: NonZeroU64) -> Result<Self> {
        let max_items = NonZeroU32::new(max_items.get() as u32)
            .unwrap_or(NonZeroU32::try_from(u32::MAX).expect("u32::MAX is not 0"));
        Ok(match self {
            RecordsGroup::Logs(mut items) => {
                RecordsGroup::Logs(split::split::<{ Logs::COUNT }>(&mut items, max_items)?)
            }
            RecordsGroup::Metrics(mut items) => {
                RecordsGroup::Metrics(split::split::<{ Metrics::COUNT }>(&mut items, max_items)?)
            }
            RecordsGroup::Traces(mut items) => {
                RecordsGroup::Traces(split::split::<{ Traces::COUNT }>(&mut items, max_items)?)
            }
        })
    }

    /// Merge `RecordBatch`es together so that they're no bigger than `max_items`.
    ///
    /// TODO: The maximum is optional, but there is usually an ID- or
    /// PARENT_ID-width that imposes some kind of limit.
    pub(crate) fn concatenate(self, max_items: Option<NonZeroU64>) -> Result<Self> {
        Ok(match self {
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_concatenate(items, max_items)?),
            RecordsGroup::Metrics(items) => {
                RecordsGroup::Metrics(generic_concatenate(items, max_items)?)
            }
            RecordsGroup::Traces(items) => {
                RecordsGroup::Traces(generic_concatenate(items, max_items)?)
            }
        })
    }

    // FIXME: replace this with an Extend impl to avoid unnecessary allocations
    /// Convert into a sequence of `OtapArrowRecords`
    #[must_use]
    pub(crate) fn into_otap_arrow_records(self) -> Vec<OtapArrowRecords> {
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
    pub(crate) const fn is_empty(&self) -> bool {
        match self {
            Self::Logs(logs) => logs.is_empty(),
            Self::Metrics(metrics) => metrics.is_empty(),
            Self::Traces(traces) => traces.is_empty(),
        }
    }
}

// *************************************************************************************************
// Everything above this line is the public interface and everything below this line is internal
// implementation details.

// Some helpers for `RecordsGroup`...
// *************************************************************************************************

/// Count the batches by matching signal type, used in separate().
fn signal_count(records: &[OtapArrowRecords], signal: SignalType) -> usize {
    records
        .iter()
        .map(|records| (records.signal_type() == signal) as usize)
        .sum()
}

/// Fetch the primary table for a given batch.
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

// Checks that we have taken all the RecordBatches after batching, that the data is all None.
fn assert_empty<const N: usize>(data: &[Option<RecordBatch>; N]) {
    assert_eq!(data, &[const { None }; N]);
}

// Calls assert_empty for all data in a batch.
fn assert_all_empty<const N: usize>(data: &[[Option<RecordBatch>; N]]) {
    for rec in data.iter() {
        assert_empty(rec);
    }
}

// Code for splitting batches
// *************************************************************************************************

/// Splits the input batches so they are no larger than max_items.
/// There is always an upper bound due to ID column width, such as a u16 limit.
fn generic_split<const N: usize>(
    mut batches: Vec<[Option<RecordBatch>; N]>,
    max_items: NonZeroU64,
    allowed_payloads: &[ArrowPayloadType],
    primary_payload: ArrowPayloadType,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    assert_eq!(N, allowed_payloads.len());
    assert!(allowed_payloads.contains(&primary_payload));

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
            .map(num_items)
            .map(|l| l as u64)
            .sum::<u64>()
            .div_ceil(max_items.get()) as usize,
    );
    // SAFETY: on 32-bit archs, `as` conversion from u64 to usize can be wrong for values >=
    // u32::MAX, but we don't care about those cases because if they happen we'll only fail to avoid
    // a reallocation.

    let splits = if N == Metrics::COUNT {
        split_metric_batches(max_items, &batches)?
    } else {
        split_non_metric_batches(max_items, &batches)?
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

            // Extract IDs only if the column exists, for splitting child tables.
            let ids_opt = rb
                .column_by_name(consts::ID)
                .map(|col| IDColumn::from_array(consts::ID, col))
                .transpose()?
                .map(|ids| IDSeqs::from_col(ids, &lengths));

            // use ids to split the child tables: call split_child_record_batch
            let new_batch_count = split_primary.len();
            result.extend(repeat_n([const { None }; N], new_batch_count));
            let result_len = result.len();
            // this is where we're going to be writing the rest of this split batch into!
            let new_batch = &mut result[result_len - new_batch_count..];

            // Store the newly split primary tables into this function's result
            for (i, split_primary) in split_primary.drain(..).enumerate() {
                new_batch[i][primary_offset] = Some(split_primary);
            }

            // Only process child tables if we have IDs
            if let Some(ids) = ids_opt.as_ref() {
                for payload in allowed_payloads
                    .iter()
                    .filter(|payload| **payload != primary_payload)
                    .copied()
                {
                    parent_child_split(batches, payload, primary_payload, ids, new_batch)?;
                }
            }
        } else {
            panic!("expected to have primary for every group");
        }

        assert_empty(batches);
    }

    Ok(result)
}

// This is a recursive helper function; the depth of recursion is bounded by parent-child
// relationships described in `child_payload_types` so we won't blow the stack.
fn parent_child_split<const N: usize>(
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
                parent_child_split(input, *child_payload, primary_payload, &id, output)?;
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
        let id = input
            .column_by_name(column_name)
            .ok_or_else(|| Error::ColumnNotFound {
                name: column_name.into(),
            })?;

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
                Err(Error::ColumnDataTypeMismatch {
                    name: column_name.into(),
                    expect: DataType::UInt16, // Or UInt32, but we can only provide one
                    actual: id.data_type().clone(),
                })
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
        .map_err(|e| Error::Batching { source: e })?;

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
    max_items: NonZeroU64,
    batches: &[[Option<RecordBatch>; N]],
) -> Result<Vec<(usize, Range<usize>)>> {
    let mut result = Vec::new();

    let mut total_records_seen: u64 = 0; // think of this like iter::single(0).chain(batch_sizes.iter()).cumsum()
    for (batch_index, batches) in batches.iter().enumerate() {
        let num_records = num_items(batches);

        // SAFETY: % panics if the second arg is 0, but we're relying on NonZeroU64 to ensure
        // that can't happen.
        let prev_batch_size = total_records_seen % max_items.get();
        let first_batch_size = (max_items.get() - prev_batch_size) as usize;
        // FIXME: this calculation is broken for logs & traces since it doesn't take into account
        // how we have to limit batch size to accomodate the u16::MAX size limit for non-null IDs.

        if num_records > first_batch_size {
            let batch_sizes = once(first_batch_size).chain(repeat(max_items.get() as usize));
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
//
// However, note that we do not split the list of DataPoints within a
// metric, despite our max_items parameter being a count of
// items, we are only combining data at the Metric level. This makes
// it possible that one batch will exceed the batch size, happening
// when an individual slice of data points exceeds the limit.
fn split_metric_batches<const N: usize>(
    max_items: NonZeroU64,
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
    let max_items = max_items.get() as usize;

    // Note! this function counts down, carrying a "remaining"
    // available size in the current batch, which makes it different
    // from the other similar methods in this file (e.g.,
    // split_non_metric_batches, generic_concatenate).
    //
    // This variable is carried across the for loop and meant to
    // assist the next stage of batching, generic_concatenate. The
    // carrying of batch_size_remaining, ensures that as
    // generic_concatenate iterates through batches in the same order
    // it can accumulate up to the limit for each output, reproducing
    // the size which are merely being calculated here in advance.
    let mut batch_size_remaining = max_items;

    for (batch_index, batch) in batches.iter().enumerate() {
        // If zero items, reset. Both branches of this loop subtract from the remaining,
        // so reset on loop entry.
        if batch_size_remaining == 0 {
            batch_size_remaining = max_items;
        }

        let metrics = batch[METRICS_INDEX]
            .as_ref()
            .expect("we've alredy ensured that every batch has a non-null primary table");
        let metric_ids: &PrimitiveArray<UInt16Type> = as_primitive_array(
            metrics
                .column_by_name(consts::ID)
                .expect("ID column should be present"),
        );

        let metric_length = metric_ids.len();
        // SAFETY: indexing here is safe because we've already ensured that all primary tables are
        // non empty. These are sorted so the max will be at the end (in generic_split).
        let max_metric_id = metric_ids.values()[metric_length - 1];

        // Note that `max_metric_id` can differ from `metric_length` because the values in the ID
        // column can have gaps. TODO: Address a secondary safety issue: we're sizing a vector
        // to the max_metric_id below, what if the ID range is not contiguous?
        let batch_len = num_items(batch);

        // If the whole batch fits the available space, take a simple path.
        if batch_len <= batch_size_remaining {
            // We know that this batch is small enough to include
            // whole in the current output.
            batch_size_remaining -= batch_len;
            result.push((batch_index, 0..metric_length));
            continue;
        }

        // Compute a cumulative count of data points by metric in the batch.
        child_counts.clear();
        child_counts.resize(max_metric_id as usize + 1, 0);
        for dpt in DATA_POINTS_TYPES {
            let child = batch[POSITION_LOOKUP[dpt as usize]].as_ref();
            // TODO: If the child is None, do we consider it corruption?
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

        // Compute a cumluative size, for partitioning with below.
        cumulative_child_counts.clear();
        cumulative_child_counts.extend(child_counts.iter().scan(0, |accumulator, &element| {
            *accumulator += element;
            Some(*accumulator)
        }));
        // SAFETY: batch_len <= batch_size_remaining takes branch
        // above; batch_size_remaining != 0.
        assert!(!cumulative_child_counts.is_empty());

        // We want to partition `cumulative_child_counts` into chunks where the difference
        // between the first and last value of each chunk is as close to but less than
        // the available batch size.

        // We store the last cumulative data point count following each loop iteration.
        let mut last_cumulative = 0;
        // We store the position of the first unprocessed Metric.
        let mut starting_index = 0;

        // If the first metric in the batch doesn't fit in remaining space AND
        // we've already used some space, start a fresh output batch.
        let first_count = *cumulative_child_counts.first().expect("not empty") as usize;
        if first_count > batch_size_remaining && batch_size_remaining < max_items {
            batch_size_remaining = max_items;
        }

        loop {
            // candidate index is the index of the first metric (in order)
            // that will not join this batch.
            let candidate_index = cumulative_child_counts.partition_point(|&cum_child_count| {
                cum_child_count <= last_cumulative + batch_size_remaining as u64
            });

            // ending_index is checked: max() below ensures we
            // make progress, even allowing larger-than-the-limit
            // metrics to pass unsplit.
            //
            // TODO: Support splitting a metric with over-limit
            // point count.
            let ending_index = candidate_index.max(starting_index + 1);

            // The partition_point call ensures ending_index is
            // in-range.
            debug_assert!(ending_index <= metric_length);

            // We should always make forward progress; we should not
            // enter the loop when there are no points.
            debug_assert!(ending_index > starting_index);

            // Record the number of points from this batch iteration,
            // the position within cumulative_child_counts.
            let next_cumulative = cumulative_child_counts
                .get(ending_index - 1)
                .copied()
                .expect("index-1 < length");
            let split_count = next_cumulative - last_cumulative;

            // We have to make progress.
            debug_assert!(split_count > 0);

            // Emit and update the loop state.
            result.push((batch_index, starting_index..ending_index));
            starting_index = ending_index;
            last_cumulative = next_cumulative;

            // Break the loop after consuming all metrics.
            if ending_index == metric_length {
                // The last loop body updates the remaining count.
                batch_size_remaining = batch_size_remaining.saturating_sub(split_count as usize);
                break;
            }

            // Continuing means there is a next-metric in this batch
            // that would exceed the limit.  Start a new batch.
            batch_size_remaining = max_items;
        }
    }
    Ok(result)
}

// Sorting `RecordBatch`es!
// *************************************************************************************************

#[derive(Debug)]
enum HowToSort {
    SortByParentIdAndId,
}

/// Return a `RecordBatch` lexically sorted by either the `parent_id` column and secondarily by the
/// `id` column or just by the `id` column.
fn sort_record_batch(rb: RecordBatch, how: HowToSort) -> Result<RecordBatch> {
    use HowToSort::*;
    use arrow::compute::{SortColumn, SortOptions, take};

    let (schema, columns, _num_rows) = rb.into_parts();
    let id_column_index = schema.column_with_name(consts::ID).map(|pair| pair.0);
    let parent_id_column_index = schema
        .column_with_name(consts::PARENT_ID)
        .map(|pair| pair.0);

    let options = Some(SortOptions {
        descending: false,
        nulls_first: true, // We rely on this heavily later on!
    });
    let sort_columns: SmallVec<[SortColumn; 2]> =
        match (how, parent_id_column_index, id_column_index) {
            (SortByParentIdAndId, Some(parent_id), Some(id)) => {
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
            (_, _, Some(id)) => {
                // Comment reproduced from the call site, explaining why the parent_id
                // column may be Some(_) or None:
                //
                // When `parent` has both ID and PARENT_ID columns, resort by ID. Why? Because
                // the reindexing code requires that the input be sorted. For all these cases,
                // we've already reindexed by PARENT_ID in an earlier iteration of this loop.
                let id_values = columns[id].clone();
                smallvec::smallvec![SortColumn {
                    values: id_values,
                    options,
                }]
            }
            (_, _, None) => {
                // No id or parent_id columns, no sorting required.
                return RecordBatch::try_new(schema, columns)
                    .map_err(|e| Error::Batching { source: e });
            }
        };

    // safety: [`sort_to_indices`] will only return an error if the passed columns aren't supported
    // by either row converter or arrow's sort kernel, both of which should be OK for Id columns.
    let indices = sort_to_indices(&sort_columns).expect("can sort IDs");
    let input_was_already_sorted = indices.values().is_sorted();
    let columns = if input_was_already_sorted {
        // Don't bother with take if the input was already sorted as we need.
        columns
    } else {
        columns
            .iter()
            .map(|c| take(c.as_ref(), &indices, None))
            .collect::<arrow::error::Result<Vec<_>>>()
            .map_err(|e| Error::Batching { source: e })?
    };

    RecordBatch::try_new(schema, columns).map_err(|e| Error::Batching { source: e })
}

// Code for merging batches (concatenation)
// *************************************************************************************************

fn generic_concatenate<const N: usize>(
    batches: Vec<[Option<RecordBatch>; N]>,
    max_items: Option<NonZeroU64>,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    let mut result = Vec::new();

    let mut current = Vec::new();
    let mut current_num_items = 0;

    for input in batches {
        let blen = num_items(&input);

        if !current.is_empty() && size_over_limit(max_items, current_num_items + blen) {
            concatenate_emitter(&mut current, &mut result)?;
            current_num_items = 0;
        }

        current_num_items += blen;
        current.push(input);
    }

    if !current.is_empty() {
        concatenate_emitter(&mut current, &mut result)?;
    }
    Ok(result)
}

fn concatenate_emitter<const N: usize>(
    current: &mut Vec<[Option<RecordBatch>; N]>,
    result: &mut Vec<[Option<RecordBatch>; N]>,
) -> Result<()> {
    super::transform::reindex::reindex(current)?;
    result.push(concatenate(current)?);
    assert_all_empty(current);
    current.clear();
    Ok(())
}

fn size_over_limit(max_items: Option<NonZeroU64>, size: usize) -> bool {
    max_items
        .map(|limit| size as u64 > limit.get())
        .unwrap_or(false)
}
