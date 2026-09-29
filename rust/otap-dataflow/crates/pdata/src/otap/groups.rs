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
    concatenate::{ConcatOptions, concatenate_batches},
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
    Logs(Tracked<{ Logs::COUNT }>),
    /// OTAP metrics
    Metrics(Tracked<{ Metrics::COUNT }>),
    /// OTAP traces
    Traces(Tracked<{ Traces::COUNT }>),
}

/// A sequence of batches together with, for each batch, the index of the
/// original input (as passed to `separate_*`) that it was derived from.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Tracked<const N: usize> {
    pub(crate) batches: Vec<[Option<RecordBatch>; N]>,
    pub(crate) sources: Vec<usize>,
}

impl<const N: usize> Tracked<N> {
    fn with_capacity(n: usize) -> Self {
        Self {
            batches: Vec::with_capacity(n),
            sources: Vec::with_capacity(n),
        }
    }

    fn push(&mut self, batch: [Option<RecordBatch>; N], source: usize) {
        self.batches.push(batch);
        self.sources.push(source);
    }

    const fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }
}

/// One output of [`RecordsGroup::concatenate`]: the batch and the inclusive
/// range of original input indices it was built from.
pub(crate) struct GroupOutput {
    pub(crate) records: OtapArrowRecords,
    pub(crate) inputs: RangeInclusive<usize>,
}

impl RecordsGroup {
    /// Convert a sequence of `OtapArrowRecords` into three `RecordsGroup` objects.
    /// This is a sanity check. In practice, we expect the higher-level batching
    /// component to separate data by signal type. The public APIs for separating
    /// by expected signal type enforce this.
    #[must_use]
    fn separate_by_type(records: Vec<OtapArrowRecords>) -> [Self; 3] {
        let log_count = signal_count(&records, SignalType::Logs);
        let mut log_records = Tracked::with_capacity(log_count);

        let metric_count = signal_count(&records, SignalType::Metrics);
        let mut metric_records = Tracked::with_capacity(metric_count);

        let trace_count = signal_count(&records, SignalType::Traces);
        let mut trace_records = Tracked::with_capacity(trace_count);

        for (idx, records) in records.into_iter().enumerate() {
            match records {
                OtapArrowRecords::Logs(logs) => {
                    let batches = logs.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        log_records.push(batches, idx);
                    }
                }
                OtapArrowRecords::Metrics(metrics) => {
                    let batches = metrics.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        metric_records.push(batches, idx);
                    }
                }
                OtapArrowRecords::Traces(traces) => {
                    let batches = traces.into_batches();
                    if primary_table(&batches)
                        .map(|batch| batch.num_rows() > 0)
                        .unwrap_or(false)
                    {
                        trace_records.push(batches, idx);
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
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_split(items, max_items)?),
            RecordsGroup::Metrics(items) => RecordsGroup::Metrics(generic_split(items, max_items)?),
            RecordsGroup::Traces(items) => RecordsGroup::Traces(generic_split(items, max_items)?),
        })
    }

    /// Merge `RecordBatch`es together so that they're no bigger than `max_items`.
    ///
    /// TODO: The maximum is optional, but there is usually an ID- or
    /// PARENT_ID-width that imposes some kind of limit.
    ///
    /// Each output carries the inclusive range of original input indices it
    /// was built from, so callers can correlate outputs with inputs without
    /// counting items.
    pub(crate) fn concatenate(self, max_items: Option<NonZeroU64>) -> Result<Vec<GroupOutput>> {
        match self {
            RecordsGroup::Logs(items) => generic_concatenate(items, max_items, |b| {
                Logs::try_from(raw_batch_store::RawLogsStore::from_batches(b))
                    .map(OtapArrowRecords::Logs)
            }),
            RecordsGroup::Metrics(items) => generic_concatenate(items, max_items, |b| {
                Metrics::try_from(raw_batch_store::RawMetricsStore::from_batches(b))
                    .map(OtapArrowRecords::Metrics)
            }),
            RecordsGroup::Traces(items) => generic_concatenate(items, max_items, |b| {
                Traces::try_from(raw_batch_store::RawTracesStore::from_batches(b))
                    .map(OtapArrowRecords::Traces)
            }),
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

fn generic_split<const N: usize>(
    mut items: Tracked<N>,
    max_items: NonZeroU32,
) -> Result<Tracked<N>> {
    let mut pieces = Vec::new();
    let batches = split::split_tracked::<N>(&mut items.batches, max_items, &mut pieces)?;
    // Map piece -> index into `items` -> original input index.
    let sources = pieces.into_iter().map(|i| items.sources[i]).collect();
    Ok(Tracked { batches, sources })
}

fn generic_concatenate<const N: usize>(
    items: Tracked<N>,
    max_items: Option<NonZeroU64>,
    build: impl Fn([Option<RecordBatch>; N]) -> Result<OtapArrowRecords>,
) -> Result<Vec<GroupOutput>> {
    let mut result = Vec::new();

    let mut current = Vec::new();
    let mut current_num_items = 0;
    let mut first_source = 0;
    let mut last_source = 0;

    for (input, source) in items.batches.into_iter().zip(items.sources) {
        let blen = num_items(&input);

        if !current.is_empty() && size_over_limit(max_items, current_num_items + blen) {
            result.push(GroupOutput {
                records: build(concatenate_emitter(&mut current)?)?,
                inputs: first_source..=last_source,
            });
            current_num_items = 0;
        }

        if current.is_empty() {
            first_source = source;
        }
        last_source = source;
        current_num_items += blen;
        current.push(input);
    }

    if !current.is_empty() {
        result.push(GroupOutput {
            records: build(concatenate_emitter(&mut current)?)?,
            inputs: first_source..=last_source,
        });
    }
    Ok(result)
}

fn concatenate_emitter<const N: usize>(
    current: &mut Vec<[Option<RecordBatch>; N]>,
) -> Result<[Option<RecordBatch>; N]> {
    let out = concatenate_batches(current, ConcatOptions::reindex())?;
    assert_all_empty(current);
    current.clear();
    Ok(out)
}

fn size_over_limit(max_items: Option<NonZeroU64>, size: usize) -> bool {
    max_items
        .map(|limit| size as u64 > limit.get())
        .unwrap_or(false)
}
