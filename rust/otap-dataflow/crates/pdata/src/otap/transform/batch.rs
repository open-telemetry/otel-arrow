// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fused split + concatenate for batching OTAP data by item count.
//!
//! The batch processor needs to turn a sequence of inputs into outputs of at
//! most `max_items` items. Historically this was done by splitting every
//! input into pieces (sorting and copying every table), then concatenating
//! the pieces (copying again). This module plans one output at a time as a
//! contiguous range over the inputs and hands the range to the fused
//! concatenation kernel, which copies every selected row exactly once.
//!
//! # Ranges and segments
//!
//! A position in the inputs is a cursor `(input, root row)`. An output is
//! the half-open range between two cursors. Only the inputs at the two ends
//! of a range can be cut; every input in between is taken whole, without any
//! sorting, joining or planning.
//!
//! ```text
//!  inputs:   [   0   ][    1    ][ 2 ][     3     ]
//!  range:        s--------------------------e
//!                ^ cut      whole  whole    ^ cut
//! ```
//!
//! # Cutting an input
//!
//! A cut input is described by a row range of its root table. The rows of
//! every other payload are derived top-down through the parent -> child
//! relations: the keys of the selected parent rows are collected into a
//! [KeySet], and a [Join] on the child's `parent_id` returns the child rows
//! with those keys. Children whose parent is not selected (including
//! dangling references) are never selected, so a cut segment never contains
//! referential integrity violations.
//!
//! A [Join] argsorts the child rows by `parent_id` without moving any column
//! data. When the child table is already sorted by `parent_id` no sort is
//! needed and a key range maps to a contiguous range of rows, which the
//! kernel can slice without copying. Joins are built lazily, only for inputs
//! that are actually cut, and their buffers are reused across inputs.
//!
//! For metrics, a root row's item count is the number of its data points,
//! which is answered by the same joins that are later used to cut. Metrics
//! are packed greedily at metric boundaries: an output takes whole metrics
//! (from the next input too) until the next metric does not fit, so it may
//! close below the limit, and a single metric larger than the limit is
//! emitted alone.
//!
//! # TODO
//!
//! - TODO(null-parent-cut): Child rows with a null `parent_id` belong to no
//!   parent, so they are dropped from inputs that are cut. They are kept
//!   when an input is taken whole.
//! - TODO(id-budget): With no `max_items`, an output whose root ids exceed
//!   the ID space is an error. The planner could cut instead.
//! - TODO(residual-cursor): The batch processor re-buffers an undersized
//!   trailing output as a materialized batch, which is copied again on the
//!   next flush. It could carry a cursor into the original input instead.
//! - TODO(join-strategy): Pick between argsort, counting sort (dense keys),
//!   and a filter scan (inputs cut once or twice) using benchmarks.
//! - TODO(nested-keyset): KeySets for nested relations (span events, data
//!   points) sort their keys; a bitmap over the parent's id span would avoid
//!   the sort.
//! - TODO(metrics-sizing): Metric item counts binary search the data points
//!   join per metric row; a merge walk would be linear.
//! - TODO(split-metric): A metric with more data points than the limit is
//!   not divided across outputs.

use std::borrow::Cow;
use std::ops::{Range, RangeInclusive};

use arrow::array::{Array, ArrayRef, AsArray, RecordBatch};
use arrow::datatypes::{ArrowNativeType, DataType, UInt8Type, UInt16Type, UInt32Type};

use crate::error::{Error, Result};
use crate::otap::transform::concatenate::{ConcatOptions, Selection, concatenate_segments};
use crate::otap::transform::reindex::{Segment, remove_transport_encodings};
use crate::otap::transform::util::{payload_relations, payload_to_idx};
use crate::otap::{Logs, Metrics, OtapBatchStore, Traces};
use crate::otlp::metrics::MetricType;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{METRIC_TYPE, PARENT_ID};

/// One output of [batch_items]: the output batch and the inclusive range of
/// input indices that contributed rows to it.
pub(crate) struct BatchOutput<const N: usize> {
    pub(crate) batches: [Option<RecordBatch>; N],
    pub(crate) inputs: RangeInclusive<usize>,
}

/// Batch `inputs` into outputs of at most `max_items` items each (no limit
/// when `None`), in input order.
///
/// Logs and traces outputs hold exactly `max_items` items except the last.
/// Metrics are cut only at metric boundaries: a metric with more data points
/// than `max_items` is emitted in an output of its own, and an output closes
/// early when the next metric does not fit.
///
/// Inputs with an empty (or missing) root table contribute nothing and are
/// skipped.
pub(crate) fn batch_items<S: OtapBatchStore, const N: usize>(
    inputs: Vec<[Option<RecordBatch>; N]>,
    max_items: Option<usize>,
) -> Result<Vec<BatchOutput<N>>> {
    Batcher::<S, N>::new(inputs)?.run(max_items)
}

/// A position in the inputs: `row` is a row of `input`'s root table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Cursor {
    input: usize,
    row: usize,
}

struct Batcher<S, const N: usize> {
    inputs: Vec<[Option<RecordBatch>; N]>,
    /// Root row count of each input.
    root_rows: Vec<usize>,
    /// Whether each input has had transport encodings removed.
    decoded: Vec<bool>,
    root_idx: usize,
    joins: JoinCache<N>,
    _signal: std::marker::PhantomData<S>,
}

impl<S: OtapBatchStore, const N: usize> Batcher<S, N> {
    fn new(inputs: Vec<[Option<RecordBatch>; N]>) -> Result<Self> {
        let root_type = match N {
            Logs::COUNT => ArrowPayloadType::Logs,
            Metrics::COUNT => ArrowPayloadType::UnivariateMetrics,
            Traces::COUNT => ArrowPayloadType::Spans,
            _ => return Err(Error::UnsupportedBatchStoreType { batch_width: N }),
        };
        let root_idx = payload_to_idx(root_type);
        let root_rows = inputs
            .iter()
            .map(|b| b[root_idx].as_ref().map_or(0, RecordBatch::num_rows))
            .collect();
        let n = inputs.len();
        Ok(Self {
            inputs,
            root_rows,
            decoded: vec![false; n],
            root_idx,
            joins: JoinCache::new(),
            _signal: std::marker::PhantomData,
        })
    }

    fn run(mut self, max_items: Option<usize>) -> Result<Vec<BatchOutput<N>>> {
        let mut outputs = Vec::new();
        let mut cur = self.skip_empty(Cursor { input: 0, row: 0 });
        while cur.input < self.inputs.len() {
            let end = match max_items {
                None => Cursor {
                    input: self.inputs.len(),
                    row: 0,
                },
                Some(max) => self.seek(cur, max.max(1))?,
            };
            debug_assert!(end > cur, "every range must make progress");
            outputs.push(self.emit(cur, end)?);
            cur = self.skip_empty(end);
        }
        Ok(outputs)
    }

    /// Advance past inputs with no remaining root rows.
    fn skip_empty(&self, mut c: Cursor) -> Cursor {
        while c.input < self.inputs.len() && c.row >= self.root_rows[c.input] {
            c = Cursor {
                input: c.input + 1,
                row: 0,
            };
        }
        c
    }

    /// Greedy item walk: the furthest cursor from `from` such that the range
    /// holds at most `max` items, taking at least one root row.
    fn seek(&mut self, from: Cursor, max: usize) -> Result<Cursor> {
        let mut budget = max;
        let mut c = from;
        let mut taken_any = false;
        while c.input < self.inputs.len() {
            let rows = self.root_rows[c.input];
            if c.row >= rows {
                c = Cursor {
                    input: c.input + 1,
                    row: 0,
                };
                continue;
            }

            // Whole remaining input fits: O(1) for inputs we start at row 0.
            if c.row == 0 {
                let items = crate::otap::num_items(&self.inputs[c.input]);
                if items <= budget {
                    budget -= items;
                    taken_any = true;
                    c = Cursor {
                        input: c.input + 1,
                        row: 0,
                    };
                    continue;
                }
            }

            // The budget ends inside this input.
            let end_row = if N == Metrics::COUNT {
                self.seek_metrics(c, &mut budget, !taken_any)?
            } else {
                let end = (c.row + budget)
                    .min(rows)
                    .max(c.row + usize::from(!taken_any));
                budget = budget.saturating_sub(end - c.row);
                end
            };
            if end_row < rows {
                c.row = end_row;
                return Ok(c);
            }
            // The rest of the input fit after all (metrics rows without data
            // points); keep absorbing following content that costs nothing.
            taken_any = true;
            c = Cursor {
                input: c.input + 1,
                row: 0,
            };
        }
        Ok(c)
    }

    /// Walk metric rows of `c.input` from `c.row`, consuming `budget`, and
    /// return the first row that does not fit. If `force_one` the first
    /// metric is always taken, even if it alone exceeds the budget.
    fn seek_metrics(&mut self, c: Cursor, budget: &mut usize, force_one: bool) -> Result<usize> {
        let input = c.input;
        self.prepare(input)?;
        for dp in DATA_POINT_TYPES {
            let idx = payload_to_idx(dp);
            self.joins.ensure(input, idx, &self.inputs[input][idx])?;
        }

        let root = self.inputs[input][self.root_idx]
            .as_ref()
            .expect("non-empty root");
        let ids = KeyColumn::new(root, crate::schema::consts::ID)?;
        let types = root
            .column_by_name(METRIC_TYPE)
            .ok_or_else(|| Error::ColumnNotFound {
                name: METRIC_TYPE.to_string(),
            })?
            .as_primitive_opt::<UInt8Type>()
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: METRIC_TYPE.to_string(),
                expect: DataType::UInt8,
                actual: DataType::Null,
            })?
            .clone();

        let rows = self.root_rows[input];
        let mut row = c.row;
        let mut force_one = force_one;
        while row < rows {
            let points = match (ids.get(row), dp_payload_for(types.value(row))) {
                (Some(id), Some(dp)) => self
                    .joins
                    .get(input, payload_to_idx(dp))
                    .map_or(0, |j| j.count(id)),
                _ => 0,
            };
            // Metrics without data points cost nothing and ride along with
            // their neighbors. When forced, the first metric that carries
            // data points is taken even if it alone exceeds the budget.
            if points > 0 {
                if points > *budget && !force_one {
                    break;
                }
                force_one = false;
            }
            *budget = budget.saturating_sub(points);
            row += 1;
        }
        Ok(row)
    }

    /// Remove transport encodings from an input (idempotent).
    fn prepare(&mut self, input: usize) -> Result<()> {
        if !self.decoded[input] {
            remove_transport_encodings::<S, N>(std::slice::from_mut(&mut self.inputs[input]))?;
            self.decoded[input] = true;
        }
        Ok(())
    }

    /// Produce the output for the range `[start, end)`.
    fn emit(&mut self, start: Cursor, end: Cursor) -> Result<BatchOutput<N>> {
        // Segments: (input, root rows) for every input with rows in range.
        let mut spans: Vec<(usize, Range<usize>)> = Vec::new();
        let last = if end.row == 0 {
            end.input
        } else {
            end.input + 1
        };
        for i in start.input..last.min(self.inputs.len()) {
            let lo = if i == start.input { start.row } else { 0 };
            let hi = if i == end.input {
                end.row
            } else {
                self.root_rows[i]
            };
            if lo < hi {
                spans.push((i, lo..hi));
            }
        }
        let first_input = spans.first().map_or(start.input, |s| s.0);
        let last_input = spans.last().map_or(start.input, |s| s.0);

        // Pass a single whole input through untouched.
        if let [(i, rows)] = spans.as_slice()
            && rows.start == 0
            && rows.end == self.root_rows[*i]
        {
            let batches = std::mem::replace(&mut self.inputs[*i], [const { None }; N]);
            return Ok(BatchOutput {
                batches,
                inputs: *i..=*i,
            });
        }

        // Decode every input in the output, then build the joins that cut
        // inputs need.
        for &(i, _) in &spans {
            self.prepare(i)?;
        }
        let cut: Vec<bool> = spans
            .iter()
            .map(|(i, r)| r.start != 0 || r.end != self.root_rows[*i])
            .collect();
        for ((i, _), &is_cut) in spans.iter().zip(&cut) {
            if is_cut {
                self.build_joins(*i)?;
            }
        }

        // Derive selections for cut inputs.
        let mut keys = KeySetScratch::default();
        let mut selections: Vec<Option<[Selection<'_>; N]>> = Vec::with_capacity(spans.len());
        for ((i, rows), &is_cut) in spans.iter().zip(&cut) {
            selections.push(if is_cut {
                Some(derive_selections::<S, N>(
                    &self.inputs[*i],
                    self.root_idx,
                    rows.clone(),
                    self.joins.for_input(*i),
                    &mut keys,
                )?)
            } else {
                None
            });
        }
        let segments: Vec<Segment<'_, N>> = spans
            .iter()
            .zip(&selections)
            .map(|((i, _), sel)| Segment {
                batches: &self.inputs[*i],
                cut: sel.as_ref(),
            })
            .collect();

        let batches = concatenate_segments::<S, N>(&segments, ConcatOptions::reindex())?;
        drop(segments);
        drop(selections);

        // Release inputs the cursor has fully passed.
        for &(i, ref rows) in &spans {
            if rows.end == self.root_rows[i] {
                self.inputs[i] = [const { None }; N];
                self.joins.release(i);
            }
        }

        Ok(BatchOutput {
            batches,
            inputs: first_input..=last_input,
        })
    }

    /// Ensure joins exist for every child payload of input `i`.
    fn build_joins(&mut self, i: usize) -> Result<()> {
        let _ = self.joins.slot_for(i);
        for &pt in S::allowed_payload_types() {
            for rel in payload_relations(pt).relations {
                for &child in rel.child_types {
                    let idx = payload_to_idx(child);
                    self.joins.ensure(i, idx, &self.inputs[i][idx])?;
                }
            }
        }
        Ok(())
    }
}

const DATA_POINT_TYPES: [ArrowPayloadType; 4] = [
    ArrowPayloadType::NumberDataPoints,
    ArrowPayloadType::SummaryDataPoints,
    ArrowPayloadType::HistogramDataPoints,
    ArrowPayloadType::ExpHistogramDataPoints,
];

fn dp_payload_for(metric_type: u8) -> Option<ArrowPayloadType> {
    match MetricType::try_from(metric_type).ok()? {
        MetricType::Gauge | MetricType::Sum => Some(ArrowPayloadType::NumberDataPoints),
        MetricType::Histogram => Some(ArrowPayloadType::HistogramDataPoints),
        MetricType::ExponentialHistogram => Some(ArrowPayloadType::ExpHistogramDataPoints),
        MetricType::Summary => Some(ArrowPayloadType::SummaryDataPoints),
        MetricType::Empty => None,
    }
}

/// Derive the row selection of every payload of a cut input whose root rows
/// are `root_rows`.
fn derive_selections<'j, S: OtapBatchStore, const N: usize>(
    batches: &[Option<RecordBatch>; N],
    root_idx: usize,
    root_rows: Range<usize>,
    joins: &'j [Option<Join>; N],
    keys: &mut KeySetScratch,
) -> Result<[Selection<'j>; N]> {
    // Payloads not reached from the root keep an empty selection.
    let mut sel: [Selection<'j>; N] = std::array::from_fn(|_| Selection::Ranges(Vec::new()));
    sel[root_idx] = Selection::Ranges(vec![root_rows]);

    // Parents are always visited before their children: walk the payload
    // tree breadth-first from the root.
    let mut queue: Vec<ArrowPayloadType> = vec![S::payload_type_at_idx(root_idx)];
    let mut qi = 0;
    while qi < queue.len() {
        let parent = queue[qi];
        qi += 1;
        let parent_idx = payload_to_idx(parent);
        let Some(parent_rb) = batches[parent_idx].as_ref() else {
            continue;
        };
        for rel in payload_relations(parent).relations {
            let present: Vec<ArrowPayloadType> = rel
                .child_types
                .iter()
                .copied()
                .filter(|c| batches[payload_to_idx(*c)].is_some())
                .collect();
            if present.is_empty() {
                continue;
            }
            let Some(col) = KeyColumn::try_new(parent_rb, rel.key_col)? else {
                continue;
            };
            let set = keys.build(&col, &sel[parent_idx], parent_rb.num_rows());
            for child in present {
                let idx = payload_to_idx(child);
                sel[idx] = match joins[idx].as_ref() {
                    Some(join) => join.select(set),
                    None => Selection::Ranges(Vec::new()),
                };
                queue.push(child);
            }
        }
    }
    Ok(sel)
}

// ---------------------------------------------------------------------------
// Key columns and key sets
// ---------------------------------------------------------------------------

/// Typed, row-indexed view of an ID column (native or dictionary encoded).
enum KeyColumn<'a> {
    U16(&'a [u16], Option<&'a arrow::buffer::NullBuffer>),
    U32(&'a [u32], Option<&'a arrow::buffer::NullBuffer>),
    /// Dictionary: keys resolved through values. Stored materialized since
    /// dictionary-encoded id columns are small and rare.
    Resolved(Vec<Option<u32>>),
}

impl<'a> KeyColumn<'a> {
    fn new(rb: &'a RecordBatch, path: &str) -> Result<Self> {
        Self::try_new(rb, path)?.ok_or_else(|| Error::ColumnNotFound {
            name: path.to_string(),
        })
    }

    fn try_new(rb: &'a RecordBatch, path: &str) -> Result<Option<Self>> {
        let Some(col_ref) = column_ref(rb, path) else {
            return Ok(None);
        };
        Ok(Some(match col_ref.data_type() {
            DataType::UInt16 => {
                let a = col_ref.as_primitive::<UInt16Type>();
                KeyColumn::U16(a.values(), a.nulls().filter(|n| n.null_count() > 0))
            }
            DataType::UInt32 => {
                let a = col_ref.as_primitive::<UInt32Type>();
                KeyColumn::U32(a.values(), a.nulls().filter(|n| n.null_count() > 0))
            }
            _ => Self::resolve(col_ref)?,
        }))
    }

    fn resolve(col: &ArrayRef) -> Result<Self> {
        let out: Vec<Option<u32>> = match col.data_type() {
            DataType::UInt16 => col
                .as_primitive::<UInt16Type>()
                .iter()
                .map(|v| v.map(u32::from))
                .collect(),
            DataType::UInt32 => col.as_primitive::<UInt32Type>().iter().collect(),
            DataType::Dictionary(k, v) => {
                macro_rules! dict {
                    ($k:ty) => {{
                        let d = col.as_dictionary::<$k>();
                        let vals: Vec<Option<u32>> = match v.as_ref() {
                            DataType::UInt16 => d
                                .values()
                                .as_primitive::<UInt16Type>()
                                .iter()
                                .map(|x| x.map(u32::from))
                                .collect(),
                            DataType::UInt32 => {
                                d.values().as_primitive::<UInt32Type>().iter().collect()
                            }
                            other => {
                                return Err(Error::UnsupportedParentIdType {
                                    actual: other.clone(),
                                });
                            }
                        };
                        d.keys()
                            .iter()
                            .map(|k| k.and_then(|k| vals.get(k.as_usize()).copied().flatten()))
                            .collect()
                    }};
                }
                match k.as_ref() {
                    DataType::UInt8 => dict!(UInt8Type),
                    DataType::UInt16 => dict!(UInt16Type),
                    other => {
                        return Err(Error::UnsupportedDictionaryKeyType {
                            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                            actual: other.clone(),
                        });
                    }
                }
            }
            other => {
                return Err(Error::UnsupportedParentIdType {
                    actual: other.clone(),
                });
            }
        };
        Ok(KeyColumn::Resolved(out))
    }

    #[inline]
    fn get(&self, row: usize) -> Option<u32> {
        match self {
            KeyColumn::U16(v, n) => n.is_none_or(|n| n.is_valid(row)).then(|| u32::from(v[row])),
            KeyColumn::U32(v, n) => n.is_none_or(|n| n.is_valid(row)).then(|| v[row]),
            KeyColumn::Resolved(v) => v[row],
        }
    }

    fn len(&self) -> usize {
        match self {
            KeyColumn::U16(v, _) => v.len(),
            KeyColumn::U32(v, _) => v.len(),
            KeyColumn::Resolved(v) => v.len(),
        }
    }
}

/// Borrow the (possibly struct-nested) column at `path` from `rb`.
fn column_ref<'a>(rb: &'a RecordBatch, path: &str) -> Option<&'a ArrayRef> {
    use crate::otap::transform::util::struct_column_name;
    if let Some(struct_name) = struct_column_name(path) {
        let s = rb.column_by_name(struct_name)?.as_struct_opt()?;
        return s.column_by_name(crate::schema::consts::ID);
    }
    rb.column_by_name(path)
}

/// Distinct sorted keys of the selected parent rows, as inclusive runs.
#[derive(Default)]
struct KeySetScratch {
    values: Vec<u32>,
    runs: Vec<RangeInclusive<u32>>,
}

/// A set of keys as sorted, disjoint, non-adjacent inclusive runs.
type KeySet<'s> = &'s [RangeInclusive<u32>];

/// Append key `v` to runs built from non-decreasing keys, extending the last
/// run when `v` repeats or directly follows it. Returns false (leaving the
/// runs unchanged) if `v` is smaller than the previous key.
#[inline]
fn push_run(runs: &mut Vec<RangeInclusive<u32>>, v: u32) -> bool {
    match runs.last_mut() {
        Some(last) if v < *last.end() => false,
        Some(last) if v <= last.end().saturating_add(1) => {
            *last = *last.start()..=v;
            true
        }
        _ => {
            runs.push(v..=v);
            true
        }
    }
}

impl KeySetScratch {
    fn build<'s>(&'s mut self, col: &KeyColumn<'_>, sel: &Selection<'_>, len: usize) -> KeySet<'s> {
        debug_assert_eq!(col.len(), len);
        self.runs.clear();

        // Fast path: keys at the selected rows are non-decreasing (typical
        // for ids in row order), so runs are built in one streaming pass
        // without collecting or sorting.
        let mut sorted = true;
        macro_rules! stream {
            ($values:expr, $nulls:expr) => {{
                let values = $values;
                let nulls = $nulls;
                sel.ranges(len).for_each(|r| {
                    if !sorted {
                        return;
                    }
                    match nulls {
                        None => {
                            let vals = &values[r];
                            // Dense ids in row order (each key one more than
                            // the previous) form a single run: check that
                            // with a tight loop and push it at once.
                            if let (Some(&first), Some(&last)) = (vals.first(), vals.last())
                                && vals
                                    .windows(2)
                                    .all(|w| u32::from(w[1]) == u32::from(w[0]) + 1)
                                && push_run(&mut self.runs, u32::from(first))
                            {
                                let top = self.runs.last_mut().expect("just pushed");
                                *top = *top.start()..=u32::from(last);
                                return;
                            }
                            for &v in vals {
                                if !push_run(&mut self.runs, u32::from(v)) {
                                    sorted = false;
                                    return;
                                }
                            }
                        }
                        Some(n) => {
                            for i in r {
                                if n.is_valid(i) && !push_run(&mut self.runs, u32::from(values[i]))
                                {
                                    sorted = false;
                                    return;
                                }
                            }
                        }
                    }
                });
            }};
        }
        match col {
            KeyColumn::U16(v, n) => stream!(*v, *n),
            KeyColumn::U32(v, n) => stream!(*v, *n),
            KeyColumn::Resolved(_) => sorted = false,
        }
        if sorted {
            return &self.runs;
        }

        // Slow path: collect, sort, and dedupe.
        self.runs.clear();
        self.values.clear();
        sel.for_each_row(len, |r| {
            if let Some(k) = col.get(r) {
                self.values.push(k);
            }
        });
        self.values.sort_unstable();
        for &v in &self.values {
            let ok = push_run(&mut self.runs, v);
            debug_assert!(ok);
        }
        &self.runs
    }
}

// ---------------------------------------------------------------------------
// Joins
// ---------------------------------------------------------------------------

/// A lookup from parent key to the child rows that reference it.
#[derive(Default)]
struct Join {
    mode: JoinMode,
    /// Argsort mode: the sorted keys, parallel to `rows`.
    keys: Vec<u32>,
    /// Argsort mode: child row indices ordered by (key, row).
    rows: Vec<u32>,
    /// Argsort scratch, reused across builds.
    packed: Vec<u64>,
}

#[derive(Default)]
enum JoinMode {
    /// The child `parent_id` column is sorted with no nulls: rows with keys
    /// in a range are a contiguous range of rows. The column is kept (an
    /// `Arc` clone) and searched directly, so nothing is copied.
    Sorted(SortedKeys),
    /// Rows are argsorted by key; nulls are excluded.
    #[default]
    Argsort,
}

/// A sorted native key column.
enum SortedKeys {
    U16(arrow::buffer::ScalarBuffer<u16>),
    U32(arrow::buffer::ScalarBuffer<u32>),
}

impl SortedKeys {
    #[inline]
    fn span(&self, run: &RangeInclusive<u32>) -> Range<usize> {
        match self {
            SortedKeys::U16(v) => {
                let lo = u16::try_from(*run.start()).unwrap_or(u16::MAX);
                let s = if *run.start() > u32::from(u16::MAX) {
                    v.len()
                } else {
                    v.partition_point(|&k| k < lo)
                };
                let e = v.partition_point(|&k| u32::from(k) <= *run.end());
                s..e.max(s)
            }
            SortedKeys::U32(v) => {
                let s = v.partition_point(|&k| k < *run.start());
                let e = v.partition_point(|&k| k <= *run.end());
                s..e
            }
        }
    }
}

impl Join {
    /// (Re)build the join over `rb`'s `parent_id`, reusing buffers.
    fn build(&mut self, rb: &RecordBatch) -> Result<()> {
        self.keys.clear();
        self.rows.clear();
        let Some(col_ref) = rb.column_by_name(PARENT_ID) else {
            return Err(Error::ColumnNotFound {
                name: PARENT_ID.to_string(),
            });
        };

        // Sorted native columns need no index at all.
        if col_ref.null_count() == 0 {
            match col_ref.data_type() {
                DataType::UInt16 => {
                    let v = col_ref.as_primitive::<UInt16Type>().values();
                    if v.is_sorted() {
                        self.mode = JoinMode::Sorted(SortedKeys::U16(v.clone()));
                        return Ok(());
                    }
                }
                DataType::UInt32 => {
                    let v = col_ref.as_primitive::<UInt32Type>().values();
                    if v.is_sorted() {
                        self.mode = JoinMode::Sorted(SortedKeys::U32(v.clone()));
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        // Argsort (key << 32 | row). Rows in the low bits make the unstable
        // sort stable within a key.
        let col = KeyColumn::new(rb, PARENT_ID)?;
        let n = col.len();
        self.mode = JoinMode::Argsort;
        self.packed.clear();
        self.packed.reserve(n);
        macro_rules! pack {
            ($values:expr, $nulls:expr) => {{
                let values = $values;
                match $nulls {
                    None => self.packed.extend(
                        values
                            .iter()
                            .enumerate()
                            .map(|(r, &k)| (u64::from(k) << 32) | r as u64),
                    ),
                    Some(nulls) => self.packed.extend(
                        nulls
                            .valid_indices()
                            .map(|r| (u64::from(values[r]) << 32) | r as u64),
                    ),
                }
            }};
        }
        match &col {
            KeyColumn::U16(v, nulls) => pack!(*v, *nulls),
            KeyColumn::U32(v, nulls) => pack!(*v, *nulls),
            KeyColumn::Resolved(v) => self.packed.extend(
                v.iter()
                    .enumerate()
                    .filter_map(|(r, k)| k.map(|k| (u64::from(k) << 32) | r as u64)),
            ),
        }
        self.packed.sort_unstable();
        self.keys
            .extend(self.packed.iter().map(|p| (p >> 32) as u32));
        self.rows.extend(self.packed.iter().map(|p| *p as u32));
        Ok(())
    }

    /// Positions (rows in sorted mode, indices into `rows` in argsort mode)
    /// holding keys in `run`.
    #[inline]
    fn span(&self, run: &RangeInclusive<u32>) -> Range<usize> {
        match &self.mode {
            JoinMode::Sorted(keys) => keys.span(run),
            JoinMode::Argsort => {
                let s = self.keys.partition_point(|&k| k < *run.start());
                let e = self.keys.partition_point(|&k| k <= *run.end());
                s..e
            }
        }
    }

    /// Number of child rows with parent key `key`.
    fn count(&self, key: u32) -> usize {
        self.span(&(key..=key)).len()
    }

    /// Child rows whose parent key is in `keys`.
    fn select(&self, keys: KeySet<'_>) -> Selection<'_> {
        let mut spans = keys
            .iter()
            .map(|run| self.span(run))
            .filter(|r| !r.is_empty());
        match &self.mode {
            JoinMode::Sorted(_) => {
                // Adjacent spans (keys with no children in between) merge.
                let mut ranges: Vec<Range<usize>> = Vec::new();
                for s in spans {
                    match ranges.last_mut() {
                        Some(last) if last.end == s.start => last.end = s.end,
                        _ => ranges.push(s),
                    }
                }
                Selection::Ranges(ranges)
            }
            JoinMode::Argsort => match (spans.next(), spans.next()) {
                (None, _) => Selection::Ranges(Vec::new()),
                (Some(one), None) => Selection::Gather(Cow::Borrowed(&self.rows[one])),
                (Some(a), Some(b)) => {
                    let mut rows = Vec::new();
                    rows.extend_from_slice(&self.rows[a]);
                    rows.extend_from_slice(&self.rows[b]);
                    for s in spans {
                        rows.extend_from_slice(&self.rows[s]);
                    }
                    Selection::Gather(Cow::Owned(rows))
                }
            },
        }
    }
}

/// Joins for at most two inputs at a time: a range cuts at most two inputs
/// (its start and end), and when a range is emitted the end input becomes
/// the next range's start. Buffers are recycled between inputs.
struct JoinCache<const N: usize> {
    slots: [(Option<usize>, [Option<Join>; N]); 2],
    spare: Vec<Join>,
}

impl<const N: usize> JoinCache<N> {
    fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| (None, std::array::from_fn(|_| None))),
            spare: Vec::new(),
        }
    }

    fn slot_of(&self, input: usize) -> Option<usize> {
        self.slots.iter().position(|(i, _)| *i == Some(input))
    }

    fn slot_for(&mut self, input: usize) -> usize {
        if let Some(s) = self.slot_of(input) {
            return s;
        }
        let s = match self.slots.iter().position(|(i, _)| i.is_none()) {
            Some(s) => s,
            None => {
                // Evict the lower input; the cursor has moved past it.
                let s = if self.slots[0].0 < self.slots[1].0 {
                    0
                } else {
                    1
                };
                self.clear_slot(s);
                s
            }
        };
        self.slots[s].0 = Some(input);
        s
    }

    fn clear_slot(&mut self, s: usize) {
        self.slots[s].0 = None;
        for j in self.slots[s].1.iter_mut() {
            if let Some(j) = j.take() {
                self.spare.push(j);
            }
        }
    }

    fn ensure(&mut self, input: usize, idx: usize, rb: &Option<RecordBatch>) -> Result<()> {
        let Some(rb) = rb.as_ref() else {
            return Ok(());
        };
        let s = self.slot_for(input);
        if self.slots[s].1[idx].is_none() {
            let mut join = self.spare.pop().unwrap_or_default();
            join.build(rb)?;
            self.slots[s].1[idx] = Some(join);
        }
        Ok(())
    }

    fn get(&self, input: usize, idx: usize) -> Option<&Join> {
        self.slots[self.slot_of(input)?].1[idx].as_ref()
    }

    /// The joins of a cut input. Joins are always built (via
    /// [Self::ensure]) before selections are derived, which claims a slot.
    fn for_input(&self, input: usize) -> &[Option<Join>; N] {
        let s = self
            .slot_of(input)
            .expect("joins are built before selections are derived");
        &self.slots[s].1
    }

    fn release(&mut self, input: usize) {
        if let Some(s) = self.slot_of(input) {
            self.clear_slot(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::otap::sealed::OtapBatchStore as _;
    use crate::otap::transform::testing::collect_row_ids;
    use crate::otap::transform::util::access_column;
    use crate::otap::{OtapArrowRecords, num_items};
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::fixtures::{DataGenerator, LogsConfig};
    use crate::testing::round_trip::{otap_to_otlp, otlp_to_otap};
    use crate::{logs, metrics, record_batch, traces};

    // ---- Logs tests ----
    // TODO test all the referential integrity violation variants

    /// Scenario: logs with shared resources/scopes are batched at every max size from 1 to items+1.
    /// Guarantees: outputs respect the limit, keep every root id and item, satisfy referential integrity, report contiguous input ranges, and are OTLP-equivalent to the input.
    #[test]
    #[rustfmt::skip]
    fn test_batch_logs() {
        check_batching_stores(&[logs!(
            (Logs,
                ("id", UInt16, vec![0u16, 1, 2, 3, 4, 5]),
                ("scope.id", UInt16, vec![0u16, 0, 0, 1, 1, 2]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1, 2, 3])),
            (LogAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 2, 3, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 2, 2, 2])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 2, 3, 3]))
        )]);
    }

    /// Scenario: a single log record is batched with max_items = 1.
    /// Guarantees: exactly one output holding the one item is produced.
    #[test]
    #[rustfmt::skip]
    fn test_batch_logs_singleton() {
        let batch = logs!(
            (Logs,
                ("id", UInt16, vec![0u16]),
                ("scope.id", UInt16, vec![0u16]),
                ("resource.id", UInt16, vec![0u16])),
            (LogAttrs,
                ("parent_id", UInt16, vec![0u16])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16]))
        ).into_batches();

        let result = batch_items::<Logs, { Logs::COUNT }>(vec![batch], Some(1)).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0].batches), 1);
    }

    /// Scenario: batching is invoked with no inputs for every signal.
    /// Guarantees: no outputs are produced and no error is returned.
    #[test]
    fn test_batch_empty_batches() {
        let result = batch_items::<Logs, { Logs::COUNT }>(vec![], Some(2)).unwrap();
        assert_eq!(result.len(), 0);
        let result = batch_items::<Metrics, { Metrics::COUNT }>(vec![], Some(2)).unwrap();
        assert_eq!(result.len(), 0);
        let result = batch_items::<Traces, { Traces::COUNT }>(vec![], Some(2)).unwrap();
        assert_eq!(result.len(), 0);
    }

    // ---- Traces tests ----

    /// Scenario: traces with span attrs, events (with attrs) and links are batched at every size, with plain and dictionary-encoded u32 parent ids.
    /// Guarantees: every nested child table is cut consistently with its parent and the output is OTLP-equivalent to the input.
    #[test]
    #[rustfmt::skip]
    fn test_batch_traces() {
        let span_ids        = vec![0u16, 1, 2, 3];
        let scope_ids       = vec![0u16, 0, 1, 1];
        let resource_ids    = vec![0u16, 0, 0, 1];
        let span_attr_pids  = vec![0, 1, 2, 3, 3];
        let event_ids       = vec![0u32, 1, 2, 3];
        let event_pids      = vec![0u16, 1, 2, 3];
        let event_attr_pids = vec![0u32, 1, 2, 3];
        let link_ids        = vec![0u32, 1];
        let link_pids       = vec![1u16, 3];
        let scope_pids      = vec![0u16, 0, 1];
        let resource_pids   = vec![0u16, 0, 1, 1];

        // Plain parent_ids
        check_batching_stores(&[traces!(
            (Spans,
                ("id", UInt16, span_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (SpanAttrs,
                ("parent_id", UInt16, span_attr_pids.clone())),
            (SpanEvents,
                ("id", UInt32, event_ids.clone()),
                ("parent_id", UInt16, event_pids.clone())),
            (SpanEventAttrs,
                ("parent_id", UInt32, event_attr_pids.clone())),
            (SpanLinks,
                ("id", UInt32, link_ids.clone()),
                ("parent_id", UInt16, link_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone()))
        )]);

        // Dict<UInt8, UInt32> parent_ids for u32 columns
        check_batching_stores(&[traces!(
            (Spans,
                ("id", UInt16, span_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (SpanAttrs,
                ("parent_id", UInt16, span_attr_pids.clone())),
            (SpanEvents,
                ("id", UInt32, event_ids.clone()),
                ("parent_id", UInt16, event_pids.clone())),
            (SpanEventAttrs,
                ("parent_id", (UInt8, UInt32), (vec![3u8, 2, 1, 0], event_attr_pids.clone()))),
            (SpanLinks,
                ("id", UInt32, link_ids.clone()),
                ("parent_id", UInt16, link_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone()))
        )]);

        // Dict<UInt16, UInt32> parent_ids for u32 columns
        check_batching_stores(&[traces!(
            (Spans,
                ("id", UInt16, span_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (SpanAttrs,
                ("parent_id", UInt16, span_attr_pids.clone())),
            (SpanEvents,
                ("id", UInt32, event_ids.clone()),
                ("parent_id", UInt16, event_pids.clone())),
            (SpanEventAttrs,
                ("parent_id", (UInt16, UInt32), (vec![3u16, 2, 1, 0], event_attr_pids.clone()))),
            (SpanLinks,
                ("id", UInt32, link_ids.clone()),
                ("parent_id", UInt16, link_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone()))
        )]);
    }

    /// Scenario: a single span with two events is batched with max_items = 1.
    /// Guarantees: the span and all of its descendants land in one output.
    #[test]
    #[rustfmt::skip]
    fn test_batch_traces_singleton() {
        let batch = traces!(
            (Spans,
                ("id", UInt16, vec![0u16])),
            (SpanAttrs,
                ("parent_id", UInt16, vec![0u16])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 0])),
            (SpanEventAttrs,
                ("parent_id", UInt32, vec![0u32, 1]))
        ).into_batches();

        let result = batch_items::<Traces, { Traces::COUNT }>(vec![batch], Some(1)).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0].batches), 1);
    }

    /// Scenario: span events are not sorted by parent and their ids are not in parent order.
    /// Guarantees: the argsort join selects every event (and event attr) of each selected span exactly once.
    #[test]
    #[rustfmt::skip]
    fn test_batch_overlapping_parent_ranges() {
        // Tests the scenario where some different child ids are associated to the same
        // parent.
        check_batching_stores(&[traces!(
            (Spans,
                ("id", UInt16, vec![0u16, 1, 2, 3])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 7, 6]),
                ("parent_id", UInt16, vec![1u16, 1, 0, 0, 0, 1, 1, 3])),
            (SpanEventAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 0, 1, 1, 0, 3, 2]))
        )]);
    }

    // ---- Metrics tests ----

    /// Scenario: gauge metrics with number data points, dp attrs, exemplars and exemplar attrs are batched at every size, with plain and dictionary parent ids.
    /// Guarantees: cuts happen at metric boundaries, the whole tree follows its metric, and output is OTLP-equivalent.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_number_dp() {
        let metric_ids       = vec![0u16, 1, 2, 3];
        let scope_ids        = vec![0u16, 0, 1, 1];
        let resource_ids     = vec![0u16, 0, 0, 1];
        let metric_attr_pids = vec![0u16, 1, 2, 3];
        let scope_pids       = vec![0u16, 0, 1];
        let resource_pids    = vec![0u16, 0, 1, 1];
        let dp_ids           = vec![0u32, 1, 2, 3, 4, 5, 6, 7];
        let dp_pids          = vec![0u16, 0, 1, 1, 1, 2, 3, 3];
        let dp_attr_pids     = vec![0u32, 1, 2, 3, 4, 5, 6, 7];
        let ex_ids           = vec![0u32, 1, 2, 3];
        let ex_pids          = vec![0u32, 2, 4, 7];
        let ex_attr_pids     = vec![0u32, 1, 2, 3];

        let gauge_types = vec![MetricType::Gauge as u8; 4];

        // Plain parent_ids
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
                ("metric_type", UInt8, gauge_types.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone())),
            (NumberDataPoints,
                ("id", UInt32, dp_ids.clone()),
                ("parent_id", UInt16, dp_pids.clone())),
            (NumberDpAttrs,
                ("parent_id", UInt32, dp_attr_pids.clone())),
            (NumberDpExemplars,
                ("id", UInt32, ex_ids.clone()),
                ("parent_id", UInt32, ex_pids.clone())),
            (NumberDpExemplarAttrs,
                ("parent_id", UInt32, ex_attr_pids.clone()))
        )]);

        // Dict<UInt8, UInt32> parent_ids for u32 columns
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
                ("metric_type", UInt8, gauge_types.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone())),
            (NumberDataPoints,
                ("id", UInt32, dp_ids.clone()),
                ("parent_id", UInt16, dp_pids.clone())),
            (NumberDpAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3, 4, 5, 6, 7], dp_attr_pids.clone()))),
            (NumberDpExemplars,
                ("id", UInt32, ex_ids.clone()),
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3], ex_pids.clone()))),
            (NumberDpExemplarAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3], ex_attr_pids.clone())))
        )]);

        // Dict<UInt16, UInt32> parent_ids for u32 columns
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
                ("metric_type", UInt8, gauge_types.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids.clone())),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids.clone())),
            (NumberDataPoints,
                ("id", UInt32, dp_ids.clone()),
                ("parent_id", UInt16, dp_pids.clone())),
            (NumberDpAttrs,
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3, 4, 5, 6, 7], dp_attr_pids.clone()))),
            (NumberDpExemplars,
                ("id", UInt32, ex_ids.clone()),
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3], ex_pids.clone()))),
            (NumberDpExemplarAttrs,
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3], ex_attr_pids.clone())))
        )]);
    }

    /// Scenario: histogram metrics with data points, attrs and exemplars are batched at every size.
    /// Guarantees: histogram data point subtrees follow their metric through every cut.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_histogram_dp() {
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1, 2, 3]),
                ("metric_type", UInt8, vec![MetricType::Histogram as u8; 4]),
                ("scope.id", UInt16, vec![0u16, 0, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 1])),
            (HistogramDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7]),
                ("parent_id", UInt16, vec![0u16, 0, 1, 1, 1, 2, 3, 3])),
            (HistogramDpAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7])),
            (HistogramDpExemplars,
                ("id", UInt32, vec![0u32, 1, 2, 3]),
                ("parent_id", UInt32, vec![0u32, 2, 4, 7])),
            (HistogramDpExemplarAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2, 3]))
        )]);
    }

    /// Scenario: exponential histogram metrics are batched at every size.
    /// Guarantees: exp histogram data point subtrees follow their metric through every cut.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_exp_histogram_dp() {
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1, 2, 3]),
                ("metric_type", UInt8, vec![MetricType::ExponentialHistogram as u8; 4]),
                ("scope.id", UInt16, vec![0u16, 0, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 1])),
            (ExpHistogramDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7]),
                ("parent_id", UInt16, vec![0u16, 0, 1, 1, 1, 2, 3, 3])),
            (ExpHistogramDpAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7])),
            (ExpHistogramDpExemplars,
                ("id", UInt32, vec![0u32, 1, 2, 3]),
                ("parent_id", UInt32, vec![0u32, 2, 4, 7])),
            (ExpHistogramDpExemplarAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2, 3]))
        )]);
    }

    /// Scenario: summary metrics with data point attrs are batched at every size.
    /// Guarantees: summary data point subtrees follow their metric through every cut.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_summary_dp() {
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1, 2, 3]),
                ("metric_type", UInt8, vec![MetricType::Summary as u8; 4]),
                ("scope.id", UInt16, vec![0u16, 0, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 1])),
            (SummaryDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7]),
                ("parent_id", UInt16, vec![0u16, 0, 1, 1, 1, 2, 3, 3])),
            (SummaryDpAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2, 3, 4, 5, 6, 7]))
        )]);
    }

    /// Scenario: one metric has 6 data points and max_items = 5.
    /// Guarantees: the metric cannot be divided, so it is emitted alone in one output of 6 items.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_oversized_singleton() {
        // A single metric with 6 data points at max_items=5.
        // Reproduces the batching_tests::test_comprehensive_batch_metrics
        // "over_limit_5" case. The metric cannot be split further so it
        // must be emitted as a singleton batch that exceeds the limit.
        let batch = metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16]),
                ("metric_type", UInt8, vec![MetricType::Gauge as u8])),
            (NumberDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5]),
                ("parent_id", UInt16, vec![0u16, 0, 0, 0, 0, 0]))
        ).into_batches();

        let result = batch_items::<Metrics, { Metrics::COUNT }>(vec![batch], Some(5)).unwrap();

        // The oversized metric is emitted as a single batch.
        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0].batches), 6);
    }

    /// Scenario: one metric of each data point type is batched at every size.
    /// Guarantees: metric item counts are taken from the data point table matching each metric type.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_mixed_types() {
        // One metric of each type in the same batch:
        //   metric 0 -> Gauge        (2 dp)
        //   metric 1 -> Histogram    (2 dp)
        //   metric 2 -> ExpHistogram (2 dp)
        //   metric 3 -> Summary      (2 dp)
        check_batching_stores(&[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1, 2, 3]),
                ("metric_type", UInt8, vec![
                    MetricType::Gauge as u8,
                    MetricType::Histogram as u8,
                    MetricType::ExponentialHistogram as u8,
                    MetricType::Summary as u8,
                ]),
                ("scope.id", UInt16, vec![0u16, 0, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1, 1])),
            (NumberDataPoints,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 0])),
            (HistogramDataPoints,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![1u16, 1])),
            (ExpHistogramDataPoints,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![2u16, 2])),
            (SummaryDataPoints,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![3u16, 3]))
        )]);
    }

    /// Scenario: two logs inputs with overlapping ids are batched at every size.
    /// Guarantees: cuts across input boundaries reindex correctly and ranges correlate outputs to inputs.
    #[test]
    #[rustfmt::skip]
    fn test_batch_logs_multi_batch() {
        let batch1 = logs!(
            (Logs,
                ("id", UInt16, vec![0u16, 1, 2]),
                ("scope.id", UInt16, vec![0u16, 0, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 1])),
            (LogAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 1]))
        );
        let batch2 = logs!(
            (Logs,
                ("id", UInt16, vec![0u16, 1, 2, 3]),
                ("scope.id", UInt16, vec![0u16, 0, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 0, 1])),
            (LogAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2, 3])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 0, 1]))
        );
        check_batching_stores(&[batch1, batch2]);
    }

    /// Scenario: two traces inputs, only one with links, are batched at every size.
    /// Guarantees: payloads present in only some inputs are handled across cut and whole segments.
    #[test]
    #[rustfmt::skip]
    fn test_batch_traces_multi_batch() {
        let batch1 = traces!(
            (Spans,
                ("id", UInt16, vec![0u16, 1]),
                ("scope.id", UInt16, vec![0u16, 0]),
                ("resource.id", UInt16, vec![0u16, 0])),
            (SpanAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 1])),
            (SpanEventAttrs,
                ("parent_id", UInt32, vec![0u32, 1]))
        );
        let batch2 = traces!(
            (Spans,
                ("id", UInt16, vec![0u16, 1, 2]),
                ("scope.id", UInt16, vec![0u16, 1, 1]),
                ("resource.id", UInt16, vec![0u16, 0, 1])),
            (SpanAttrs,
                ("parent_id", UInt16, vec![0u16, 1, 2])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1, 2]),
                ("parent_id", UInt16, vec![0u16, 1, 2])),
            (SpanEventAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 2])),
            (SpanLinks,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![1u16, 2]))
        );
        check_batching_stores(&[batch1, batch2]);
    }

    /// Scenario: two metrics inputs are batched at every size.
    /// Guarantees: cut and whole metric segments combine into OTLP-equivalent outputs.
    #[test]
    #[rustfmt::skip]
    fn test_batch_metrics_multi_batch() {
        let batch1 = metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1]),
                ("metric_type", UInt8, vec![MetricType::Gauge as u8; 2]),
                ("scope.id", UInt16, vec![0u16, 0]),
                ("resource.id", UInt16, vec![0u16, 0])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16])),
            (NumberDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3]),
                ("parent_id", UInt16, vec![0u16, 0, 1, 1]))
        );
        let batch2 = metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16, 1]),
                ("metric_type", UInt8, vec![MetricType::Gauge as u8; 2]),
                ("scope.id", UInt16, vec![0u16, 1]),
                ("resource.id", UInt16, vec![0u16, 1])),
            (MetricAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (ScopeAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (ResourceAttrs,
                ("parent_id", UInt16, vec![0u16, 1])),
            (NumberDataPoints,
                ("id", UInt32, vec![0u32, 1, 2]),
                ("parent_id", UInt16, vec![0u16, 0, 1]))
        );
        check_batching_stores(&[batch1, batch2]);
    }

    /// Batch `batches` at every interesting max size and check the output.
    fn check_batching<S: OtapBatchStore, const N: usize>(
        to_otap: &dyn Fn(&[Option<RecordBatch>; N]) -> OtapArrowRecords,
        batches: &[[Option<RecordBatch>; N]],
    ) {
        let root_idx = root_idx::<N>();
        let root_type = S::payload_type_at_idx(root_idx);

        let input_otlp: Vec<_> = batches.iter().map(|b| otap_to_otlp(&to_otap(b))).collect();
        let mut expected_ids: Vec<u32> = Vec::new();
        for batch in batches {
            if let Some(rb) = &batch[root_idx] {
                expected_ids.extend(root_ids(rb));
            }
        }
        let expected_items: usize = batches.iter().map(|b| num_items(b)).sum();

        for max in get_split_sizes(expected_items) {
            let result = batch_items::<S, N>(batches.to_vec(), Some(max)).unwrap();

            // Every root row is emitted exactly once (ids per output are
            // reindexed, so count rows).
            let out_rows: usize = result
                .iter()
                .map(|o| {
                    o.batches[root_idx]
                        .as_ref()
                        .map_or(0, RecordBatch::num_rows)
                })
                .sum();
            assert_eq!(out_rows, expected_ids.len(), "max={max}");
            let result_items: usize = result.iter().map(|o| num_items(&o.batches)).sum();
            assert_eq!(result_items, expected_items, "max={max}");

            let mut next = 0usize;
            for (k, out) in result.iter().enumerate() {
                let items = num_items(&out.batches);
                if items > max {
                    assert_eq!(N, Metrics::COUNT, "non-metric output exceeded {max}");
                    let rows = out.batches[root_idx].as_ref().unwrap().num_rows();
                    assert_eq!(rows, 1, "oversized metric output must be a singleton");
                }
                if N != Metrics::COUNT && k + 1 < result.len() {
                    assert_eq!(items, max, "logs/traces outputs are full except the last");
                }
                assert_referential_integrity::<N>(&out.batches, root_type);

                let (s, e) = (*out.inputs.start(), *out.inputs.end());
                assert!(
                    s == next || (next > 0 && s == next - 1),
                    "ranges are contiguous"
                );
                next = e + 1;
            }

            let output_otlp: Vec<_> = result
                .iter()
                .map(|o| otap_to_otlp(&to_otap(&o.batches)))
                .collect();
            assert_equivalent(&input_otlp, &output_otlp);
        }
    }

    fn check_batching_stores<S, const N: usize>(stores: &[S])
    where
        S: OtapBatchStore<BatchArray = [Option<RecordBatch>; N]> + Into<OtapArrowRecords> + Clone,
    {
        let batches: Vec<[Option<RecordBatch>; N]> =
            stores.iter().cloned().map(|s| s.into_batches()).collect();
        let to_otap = |b: &[Option<RecordBatch>; N]| -> OtapArrowRecords {
            let mut store = S::new();
            store.batches_mut().clone_from_slice(b);
            store.into()
        };
        check_batching::<S, N>(&to_otap, &batches);
    }

    fn root_idx<const N: usize>() -> usize {
        payload_to_idx(match N {
            Logs::COUNT => ArrowPayloadType::Logs,
            Metrics::COUNT => ArrowPayloadType::UnivariateMetrics,
            Traces::COUNT => ArrowPayloadType::Spans,
            _ => unreachable!("unsupported batch size {N}"),
        })
    }

    /// A representative set of max sizes: small counts exhaustively, and
    /// boundary values for larger ones.
    fn get_split_sizes(total_items: usize) -> Vec<usize> {
        if total_items == 0 {
            return vec![1];
        }
        if total_items < 10 {
            return (1..=total_items + 1).collect();
        }
        let sqrt = total_items.isqrt();
        let half = total_items / 2;
        let mut sizes: Vec<usize> = (1..=sqrt).collect();
        sizes.extend([
            half - 1,
            half,
            half + 1,
            total_items - 1,
            total_items,
            total_items + 1,
        ]);
        sizes
    }

    fn root_ids(rb: &RecordBatch) -> Vec<u32> {
        let col = access_column("id", rb.schema_ref(), rb.columns()).expect("missing id column");
        collect_row_ids(col.as_ref())
    }

    /// Assert that every child parent_id in the batch is present in the
    /// parent's key column, recursively.
    fn assert_referential_integrity<const N: usize>(
        batch: &[Option<RecordBatch>; N],
        parent_type: ArrowPayloadType,
    ) {
        let parent_idx = payload_to_idx(parent_type);
        let Some(parent_rb) = batch[parent_idx].as_ref() else {
            return;
        };
        for relation in payload_relations(parent_type).relations {
            let Some(parent_col) = access_column(
                relation.key_col,
                parent_rb.schema_ref(),
                parent_rb.columns(),
            ) else {
                continue;
            };
            let parent_ids: HashSet<u32> =
                collect_row_ids(parent_col.as_ref()).into_iter().collect();
            for &child_type in relation.child_types {
                let Some(child_rb) = &batch[payload_to_idx(child_type)] else {
                    continue;
                };
                let Some(child_col) = child_rb.column_by_name(PARENT_ID) else {
                    continue;
                };
                for pid in collect_row_ids(child_col.as_ref()) {
                    assert!(
                        parent_ids.contains(&pid),
                        "{child_type:?} parent_id {pid} missing from {parent_type:?}.{}",
                        relation.key_col
                    );
                }
                assert_referential_integrity(batch, child_type);
            }
        }
    }

    // ---- Transport encodings, compaction, dictionaries ----

    fn generated_logs(n: usize, count: usize) -> Vec<Logs> {
        let mut datagen = DataGenerator::with_logs_config(
            LogsConfig::new(n)
                .with_resources(2)
                .with_scopes_per_resource(2)
                .with_resource_attrs(2)
                .with_scope_attrs(2)
                .with_log_attrs(3),
        );
        (0..count)
            .map(
                |_| match otlp_to_otap(&datagen.generate_logs_from_config().into()) {
                    OtapArrowRecords::Logs(l) => l,
                    _ => unreachable!(),
                },
            )
            .collect()
    }

    fn gauge_metrics(metrics: usize, points: usize) -> [Option<RecordBatch>; Metrics::COUNT] {
        use crate::testing::fixtures::MetricsConfig;
        let mut datagen = DataGenerator::with_metrics_config(
            MetricsConfig::new().with_gauges(vec![points; metrics]),
        );
        match otlp_to_otap(&datagen.generate_metrics_from_config().into()) {
            OtapArrowRecords::Metrics(m) => m.into_batches(),
            _ => unreachable!(),
        }
    }

    /// Scenario: three metrics inputs of 600 data points (60 metrics of 10
    /// points each) are batched with max_items = 1000.
    /// Guarantees: outputs are filled to the limit by cutting the next input
    /// at a metric boundary, rather than emitting each input on its own:
    /// [1000, 800] with ranges [0..=1, 1..=2].
    #[test]
    fn test_batch_metrics_fill_to_max() {
        let inputs = vec![
            gauge_metrics(60, 10),
            gauge_metrics(60, 10),
            gauge_metrics(60, 10),
        ];
        let out = batch_items::<Metrics, { Metrics::COUNT }>(inputs, Some(1000)).unwrap();
        let sizes: Vec<_> = out.iter().map(|o| num_items(&o.batches)).collect();
        let ranges: Vec<_> = out.iter().map(|o| o.inputs.clone()).collect();
        assert_eq!(sizes, vec![1000, 800]);
        assert_eq!(ranges, vec![0..=1, 1..=2]);
    }

    /// Scenario: metrics of 30 data points each are batched with
    /// max_items = 100 after an input of 60 points, so the next metric does
    /// not fit exactly.
    /// Guarantees: metrics are never divided; an output closes below the
    /// limit when the next whole metric does not fit, and no output exceeds
    /// the limit.
    #[test]
    fn test_batch_metrics_closes_at_metric_boundary() {
        let inputs = vec![gauge_metrics(2, 30), gauge_metrics(4, 30)];
        let out = batch_items::<Metrics, { Metrics::COUNT }>(inputs, Some(100)).unwrap();
        let sizes: Vec<_> = out.iter().map(|o| num_items(&o.batches)).collect();
        assert_eq!(sizes, vec![90, 90]);
        assert!(out.iter().all(|o| num_items(&o.batches).is_multiple_of(30)));
    }

    /// Scenario: transport-optimized logs inputs (delta-encoded ids, attrs
    /// sorted by key/value so parent ids are scattered) are cut into many
    /// outputs.
    /// Guarantees: inputs are decoded before they are cut or combined, the
    /// argsort join finds scattered children, and the outputs are
    /// OTLP-equivalent to the decoded inputs.
    #[test]
    fn test_batch_transport_encoded_inputs() {
        let plain = generated_logs(10, 3);
        let expected: Vec<_> = plain
            .iter()
            .map(|l| otap_to_otlp(&OtapArrowRecords::Logs(l.clone())))
            .collect();
        let encoded: Vec<[Option<RecordBatch>; Logs::COUNT]> = plain
            .into_iter()
            .map(|l| {
                let mut rec = OtapArrowRecords::Logs(l);
                rec.encode_transport_optimized().unwrap();
                match rec {
                    OtapArrowRecords::Logs(l) => l.into_batches(),
                    _ => unreachable!(),
                }
            })
            .collect();
        for max in [1, 7, 13, 40, 1000] {
            let out = batch_items::<Logs, { Logs::COUNT }>(encoded.clone(), Some(max)).unwrap();
            let got: Vec<_> = out
                .iter()
                .map(|o| {
                    let mut store = Logs::new();
                    store.batches_mut().clone_from_slice(&o.batches);
                    otap_to_otlp(&OtapArrowRecords::Logs(store))
                })
                .collect();
            assert_equivalent(&expected, &got);
        }
    }

    /// Scenario: every id of two logs inputs is doubled (leaving gaps), so an
    /// output that combines a cut piece of each exceeds the u16 id budget
    /// under uniform offsets and must compact the cut segments' ids.
    /// Guarantees: cut segments are compacted like whole inputs and the
    /// output stays OTLP-equivalent.
    #[test]
    #[rustfmt::skip]
    fn test_batch_compacts_cut_segments() {
        // 30k rows per input with ids 0, 2, 4, ... (a span of ~60k).
        let gapped = || {
            let ids: Vec<u16> = (0..30_000u16).map(|i| i * 2).collect();
            logs!(
                (Logs,
                    ("id", UInt16, ids.clone())),
                (LogAttrs,
                    ("parent_id", UInt16, ids))
            )
        };
        let (a, b) = (gapped(), gapped());
        let expected: Vec<_> =
            [&a, &b].iter().map(|l| otap_to_otlp(&OtapArrowRecords::Logs((*l).clone()))).collect();
        let inputs = vec![a.into_batches(), b.into_batches()];
        // Outputs: [a + b 0..10k], [b 10k..30k]. The first combines a whole
        // input (id span 60k) with a cut segment (id span 20k): 80k ids
        // with uniform offsets, which only fits the u16 id space compacted.
        let out = batch_items::<Logs, { Logs::COUNT }>(inputs, Some(40_000)).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].inputs, 0..=1);
        assert_eq!(out[1].inputs, 1..=1);
        let got: Vec<_> = out
            .iter()
            .map(|o| {
                let mut store = Logs::new();
                store.batches_mut().clone_from_slice(&o.batches);
                otap_to_otlp(&OtapArrowRecords::Logs(store))
            })
            .collect();
        assert_equivalent(&expected, &got);
    }

    /// Scenario: an output is built from two cut segments (the tail of one
    /// input, the head of the next) whose ids are sparse (spacing 4), so
    /// their id spans total ~80k and overflow the u16 id space unless the
    /// cut segments themselves are compacted.
    /// Guarantees: cut segments are compacted over their selected rows only,
    /// and the output stays OTLP-equivalent.
    #[test]
    #[rustfmt::skip]
    fn test_batch_compacts_two_cut_segments() {
        // a: 20k dense ids then 10k ids spaced by 4; b: the reverse.
        let a_ids: Vec<u16> = (0..20_000u16).chain((0..10_000u16).map(|i| 20_000 + i * 4)).collect();
        let b_ids: Vec<u16> = (0..10_000u16).map(|i| i * 4).chain(40_001..60_001u16).collect();
        let mk = |ids: Vec<u16>| logs!(
            (Logs, ("id", UInt16, ids.clone())),
            (LogAttrs, ("parent_id", UInt16, ids))
        );
        let (a, b) = (mk(a_ids), mk(b_ids));
        let expected: Vec<_> =
            [&a, &b].iter().map(|l| otap_to_otlp(&OtapArrowRecords::Logs((*l).clone()))).collect();
        let out = batch_items::<Logs, { Logs::COUNT }>(
            vec![a.into_batches(), b.into_batches()],
            Some(20_000),
        )
        .unwrap();
        let ranges: Vec<_> = out.iter().map(|o| o.inputs.clone()).collect();
        assert_eq!(ranges, vec![0..=0, 0..=1, 1..=1]);
        let got: Vec<_> = out
            .iter()
            .map(|o| {
                let mut store = Logs::new();
                store.batches_mut().clone_from_slice(&o.batches);
                otap_to_otlp(&OtapArrowRecords::Logs(store))
            })
            .collect();
        assert_equivalent(&expected, &got);
    }

    /// Scenario: a sorted logs input is cut into several outputs, each a
    /// single segment.
    /// Guarantees: payloads that map to one contiguous range are zero-copy
    /// slices of the input (they share the input's buffers).
    #[test]
    fn test_batch_single_segment_is_zero_copy() {
        let input = generated_logs(50, 1).remove(0).into_batches();
        let root = payload_to_idx(ArrowPayloadType::Logs);
        let src_ptr = input[root].as_ref().unwrap().column(0).to_data().buffers()[0].as_ptr();
        let out = batch_items::<Logs, { Logs::COUNT }>(vec![input], Some(60)).unwrap();
        assert!(out.len() > 1);
        let out_ptr = out[0].batches[root]
            .as_ref()
            .unwrap()
            .column(0)
            .to_data()
            .buffers()[0]
            .as_ptr();
        assert_eq!(src_ptr, out_ptr, "first piece must slice the input");
        for o in &out {
            assert_eq!(o.inputs, 0..=0);
        }
    }

    /// Scenario: a transport-optimized input with a dictionary-encoded
    /// attribute column is cut into several outputs that each hold one
    /// segment, gathering scattered attribute rows.
    /// Guarantees: every output shares the input's dictionary values array
    /// rather than copying it.
    #[test]
    fn test_batch_single_segment_shares_dictionary_values() {
        // Transport encoding sorts attributes by key/value, so a cut selects
        // scattered rows and takes the gather path rather than a slice.
        let mut rec = OtapArrowRecords::Logs(generated_logs(50, 1).remove(0));
        rec.encode_transport_optimized().unwrap();
        let input = match rec {
            OtapArrowRecords::Logs(l) => l.into_batches(),
            _ => unreachable!(),
        };
        let attrs = payload_to_idx(ArrowPayloadType::LogAttrs);
        let rb = input[attrs].as_ref().unwrap();
        let (dict_idx, _) = rb
            .schema()
            .fields()
            .iter()
            .enumerate()
            .find(|(_, f)| matches!(f.data_type(), DataType::Dictionary(_, _)))
            .map(|(i, f)| (i, f.clone()))
            .expect("a dictionary column");
        let values_of =
            |rb: &RecordBatch| rb.column(dict_idx).to_data().child_data()[0].buffers()[0].as_ptr();
        let src = values_of(rb);
        let out = batch_items::<Logs, { Logs::COUNT }>(vec![input], Some(60)).unwrap();
        assert!(out.len() > 1);
        for o in &out {
            let orb = o.batches[attrs].as_ref().unwrap();
            assert_eq!(values_of(orb), src, "dictionary values must be shared");
        }
    }
}
