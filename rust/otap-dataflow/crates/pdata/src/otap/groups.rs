// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Support for splitting and merging sequences of `OtapArrowRecords` in support of batching.
use std::num::{NonZeroU32, NonZeroU64};
use std::ops::RangeInclusive;

use crate::{
    otap::{
        Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces,
        error::{Error, Result},
        num_items, raw_batch_store,
        raw_batch_store::POSITION_LOOKUP,
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};
use arrow::array::RecordBatch;
use otel_arrow_dfe_config::SignalType;

use super::transform::{
    concatenate::{ConcatOptions, concatenate},
    split,
};

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
    pub(crate) fn split(self, max_items: NonZeroU64, origins: &mut Vec<usize>) -> Result<Self> {
        // The splitter indexes rows with u32. A larger requested upper bound
        // must not wrap into a small split size; cap it at the internal maximum.
        let max_items = NonZeroU32::try_from(max_items).unwrap_or(NonZeroU32::MAX);
        Ok(match self {
            RecordsGroup::Logs(mut items) => {
                RecordsGroup::Logs(split_with_origins::<{ Logs::COUNT }>(
                    &mut items, max_items, origins,
                )?)
            }
            RecordsGroup::Metrics(mut items) => {
                RecordsGroup::Metrics(split_with_origins::<{ Metrics::COUNT }>(
                    &mut items, max_items, origins,
                )?)
            }
            RecordsGroup::Traces(mut items) => {
                RecordsGroup::Traces(split_with_origins::<{ Traces::COUNT }>(
                    &mut items, max_items, origins,
                )?)
            }
        })
    }

    /// Merge `RecordBatch`es together so that they're no bigger than `max_items`.
    ///
    /// TODO: The maximum is optional, but there is usually an ID- or
    /// PARENT_ID-width that imposes some kind of limit.
    pub(crate) fn concatenate(
        self,
        max_items: Option<NonZeroU64>,
        origins: Vec<usize>,
    ) -> Result<(Self, Vec<RangeInclusive<usize>>)> {
        let (group, ranges) = match self {
            RecordsGroup::Logs(items) => {
                let (items, ranges) = generic_concatenate(items, max_items, origins)?;
                (RecordsGroup::Logs(items), ranges)
            }
            RecordsGroup::Metrics(items) => {
                let (items, ranges) = generic_concatenate(items, max_items, origins)?;
                (RecordsGroup::Metrics(items), ranges)
            }
            RecordsGroup::Traces(items) => {
                let (items, ranges) = generic_concatenate(items, max_items, origins)?;
                (RecordsGroup::Traces(items), ranges)
            }
        };
        Ok((group, ranges))
    }

    // FIXME: replace this with an Extend impl to avoid unnecessary allocations
    /// Convert into a sequence of `OtapArrowRecords`
    pub(crate) fn into_otap_arrow_records(self) -> Result<Vec<OtapArrowRecords>> {
        match self {
            RecordsGroup::Logs(items) => items
                .into_iter()
                .map(|batches| {
                    let raw = raw_batch_store::RawLogsStore::from_batches(batches);
                    Logs::try_from(raw).map(OtapArrowRecords::Logs)
                })
                .collect(),
            RecordsGroup::Metrics(items) => items
                .into_iter()
                .map(|batches| {
                    let raw = raw_batch_store::RawMetricsStore::from_batches(batches);
                    Metrics::try_from(raw).map(OtapArrowRecords::Metrics)
                })
                .collect(),
            RecordsGroup::Traces(items) => items
                .into_iter()
                .map(|batches| {
                    let raw = raw_batch_store::RawTracesStore::from_batches(batches);
                    Traces::try_from(raw).map(OtapArrowRecords::Traces)
                })
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

// Code for merging batches (concatenation)
// *************************************************************************************************

fn split_with_origins<const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
    limit: NonZeroU32,
    origins: &mut Vec<usize>,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    let mut split_origins = Vec::with_capacity(origins.len());
    let result = split::split_with_sources(items, limit, |index, count| {
        split_origins.extend(std::iter::repeat_n(origins[index], count));
    })?;
    *origins = split_origins;
    Ok(result)
}

type GroupedBatches<const N: usize> = (Vec<[Option<RecordBatch>; N]>, Vec<RangeInclusive<usize>>);

fn generic_concatenate<const N: usize>(
    batches: Vec<[Option<RecordBatch>; N]>,
    max_items: Option<NonZeroU64>,
    origins: Vec<usize>,
) -> Result<GroupedBatches<N>> {
    let mut result = Vec::new();

    let mut current = Vec::new();
    let mut current_num_items = 0;
    let mut ranges = Vec::new();
    let mut first = 0;
    let mut last = 0;

    for (input, origin) in batches.into_iter().zip(origins) {
        let blen = num_items(&input);

        if !current.is_empty() && size_over_limit(max_items, current_num_items + blen) {
            concatenate_emitter(&mut current, &mut result)?;
            ranges.push(first..=last);
            current_num_items = 0;
        }

        if current.is_empty() {
            first = origin;
        }
        last = origin;
        current_num_items += blen;
        current.push(input);
    }

    if !current.is_empty() {
        concatenate_emitter(&mut current, &mut result)?;
        ranges.push(first..=last);
    }
    Ok((result, ranges))
}

fn concatenate_emitter<const N: usize>(
    current: &mut Vec<[Option<RecordBatch>; N]>,
    result: &mut Vec<[Option<RecordBatch>; N]>,
) -> Result<()> {
    result.push(concatenate(current, ConcatOptions::reindex())?);
    assert_all_empty(current);
    current.clear();
    Ok(())
}

fn size_over_limit(max_items: Option<NonZeroU64>, size: usize) -> bool {
    max_items
        .map(|limit| size as u64 > limit.get())
        .unwrap_or(false)
}
