// Copyright The OpenTelemetry Authors SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;
use std::ops::{Range, RangeInclusive};

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, DictionaryArray, PrimitiveArray, RecordBatch,
};
use arrow::datatypes::{ArrowDictionaryKeyType, DataType, UInt8Type, UInt16Type, UInt32Type};

use crate::error::{Error, Result};
use crate::otap::transform::util::sort_otap_batch_by_parent_then_id;
use crate::otap::{Logs, Metrics, OtapBatchStore, POSITION_LOOKUP, Traces};
use crate::otlp::metrics::MetricType;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{METRIC_TYPE, PARENT_ID};

use super::util::{access_column, extract_id_column, payload_relations, take_ranges};

type SplitResult<const N: usize> = Result<Vec<[Option<RecordBatch>; N]>>;

/// Perform a naive split of the provided record batches ensuring that each
/// batch has no more than `max_items` items.
///
/// Oversized batches are split in place: the original entry keeps the
/// "leftover" (last chunk) and the earlier chunks are appended to `batches`.
pub fn split<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
    max_items: NonZeroU32,
) -> SplitResult<N> {
    if batches.is_empty() {
        return Ok(vec![]);
    }

    let root_type = match N {
        Logs::COUNT => ArrowPayloadType::Logs,
        Metrics::COUNT => ArrowPayloadType::UnivariateMetrics,
        Traces::COUNT => ArrowPayloadType::Spans,
        _ => unreachable!(),
    };
    let root_idx = POSITION_LOOKUP[root_type as usize];

    let original_len = batches.len();
    let mut output = Vec::with_capacity(original_len);
    #[allow(clippy::needless_range_loop)]
    for i in 0..original_len {
        // TODO: Can we make this impossible by OtapBatchStore guarantees?
        // TODO: Test this scenario
        if batches[i][root_idx].is_none() {
            continue;
        };

        // Sort
        let mut batch = std::mem::replace(&mut batches[i], std::array::from_fn(|_| None));
        sort_otap_batch_by_parent_then_id(&mut batch)?;

        // Plan
        let root_rb = batch[root_idx].as_ref().expect("root");
        let ranges = match N {
            Metrics::COUNT => plan_metrics_split(root_rb, &batch, max_items)?,
            Logs::COUNT | Traces::COUNT => plan_split(root_rb, max_items)?,
            _ => unreachable!(),
        };

        // Execute
        execute_split(batch, root_type, &ranges, &mut output)?;
    }

    Ok(output)
}

/// Compute split boundaries as row ranges over the sorted root table, requires
/// that all record batches in the input are sorted first by parent_id (if present)
/// then id. Also requires the root record batch is present.
///
///  Returns `N` non-overlapping,
/// sorted ranges that cover the entire root.
fn plan_split(root_rb: &RecordBatch, max_items: NonZeroU32) -> Result<Vec<Range<usize>>> {
    let n_rows = root_rb.num_rows();
    let max = max_items.get() as usize;
    let n_chunks = n_rows.div_ceil(max);

    let mut ranges = Vec::with_capacity(n_chunks);
    for i in 0..n_chunks {
        let start = i * max;
        let end = (start + max).min(n_rows);
        ranges.push(start..end);
    }

    Ok(ranges)
}

/// Compute split boundaries for metrics based on data point counts.
///
/// Unlike logs/traces where each root row is one item, each metric row may
/// correspond to many data points. The `metric_type` column indicates which
/// data points table to look in, and [`find_rows_by_parent_id_range`] counts
/// matching rows.
///
/// Dispatches on the id column's native type (UInt16/UInt32), following the
/// same pattern as [`get_contiguous_id_ranges`].
fn plan_metrics_split(
    root_rb: &RecordBatch,
    batch: &[Option<RecordBatch>],
    max_items: NonZeroU32,
) -> Result<Vec<Range<usize>>> {
    let id_col = extract_id_column(root_rb, "id")?;

    match id_col.data_type() {
        DataType::UInt16 => {
            plan_metrics_split_impl::<UInt16Type>(root_rb, &id_col, batch, max_items)
        }
        DataType::UInt32 => {
            plan_metrics_split_impl::<UInt32Type>(root_rb, &id_col, batch, max_items)
        }
        other => Err(Error::ColumnDataTypeMismatch {
            name: "id".to_string(),
            expect: DataType::UInt16,
            actual: other.clone(),
        }),
    }
}

/// Type-specialized implementation of [`plan_split_metrics`]. Works directly
/// with the native id values buffer — no allocation or conversion.
fn plan_metrics_split_impl<T>(
    root_rb: &RecordBatch,
    id_col: &ArrayRef,
    batch: &[Option<RecordBatch>],
    max_items: NonZeroU32,
) -> Result<Vec<Range<usize>>>
where
    T: ArrowPrimitiveType,
    T::Native: Into<u32> + Copy,
{
    let row_count = root_rb.num_rows();
    if row_count == 0 {
        return Ok(vec![]);
    }

    let max_items = max_items.get() as usize;

    // safety: id column type verified by caller
    let id_values = id_col
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .expect("id column type verified by caller")
        .values();

    let metric_types =
        root_rb
            .column_by_name(METRIC_TYPE)
            .ok_or_else(|| Error::ColumnNotFound {
                name: METRIC_TYPE.to_string(),
            })?;

    // safety: Assuming this property will be uphelp in the future by OtapBatchStore
    // guarantees of being spec compliant.
    let metric_types = metric_types
        .as_any()
        .downcast_ref::<PrimitiveArray<UInt8Type>>()
        .expect("metric_type is UInt8")
        .values();

    let mut ranges = Vec::new();
    let mut range_start = 0;
    let mut item_count: usize = 0;

    for row in 0..row_count {
        let metric_id: u32 = id_values[row].into();
        let metric_type = metric_types[row];

        let dp_type = payload_from_metric_type(metric_type)?;
        let dp_idx = POSITION_LOOKUP[dp_type as usize];

        // Count data points for this metric.
        let mut points = 0;
        if let Some(dp_rb) = &batch[dp_idx] {
            let range = find_rows_by_parent_id_range(dp_rb, &(metric_id..=metric_id))?;
            points = range.end - range.start;
        }

        // If adding this metric would exceed the limit, cut the current range
        // and start a new one.
        //
        // Note: A single metric that exceeds the limit will always cause
        // item_count + points > max_items on the next iteration. This causes
        // the metric to be emitted as its own range.
        //
        // FIXME: We should split metrics that exceed the limit into multiple metrics.
        if item_count + points > max_items && row > range_start {
            ranges.push(range_start..row);
            range_start = row;
            item_count = 0;
        }

        item_count += points;
    }

    // Push the final range.
    if range_start < row_count {
        ranges.push(range_start..row_count);
    }

    Ok(ranges)
}

/// Execute a split according to the given ranges.
///
/// The batch **must** already be sorted (e.g. by [`sort_otap_batch_by_parent_then_id`]).
/// Produces one `[Option<RecordBatch>; N]` per range, pushed to `output` in
/// the same order as `ranges`.
fn execute_split<const N: usize>(
    mut batch: [Option<RecordBatch>; N],
    root_type: ArrowPayloadType,
    ranges: &[Range<usize>],
    output: &mut Vec<[Option<RecordBatch>; N]>,
) -> Result<()> {
    // Entire batch fits into the required limit — push as-is.
    if ranges.len() == 1 {
        output.push(batch);
        return Ok(());
    }

    let root_idx = POSITION_LOOKUP[root_type as usize];
    let root = batch[root_idx].take().expect("root must be present");

    // Range preconditions
    debug_assert!(!ranges.is_empty());
    debug_assert!(ranges.iter().all(|r| r.start < r.end));
    debug_assert!(ranges.windows(2).all(|w| w[0].end == w[1].start));
    debug_assert_eq!(ranges.first().expect("one range").start, 0);
    debug_assert_eq!(ranges.last().expect("one range").end, root.num_rows());

    // Process one range at a time: slice the root and all children for this
    // range, assemble a completereange output batch, and push it.
    for range in ranges {
        let root_slice = root.slice(range.start, range.end - range.start);
        let mut out_batch: [Option<RecordBatch>; N] = std::array::from_fn(|_| None);
        out_batch[root_idx] = Some(root_slice.clone());
        slice_children(&batch, &mut out_batch, &root_slice, root_type)?;
        output.push(out_batch);
    }

    Ok(())
}

/// Slice the children of `parent_type` from `source` that match the given
/// `parent_slice`, placing results into `out`. Recurses for children that
/// have their own child relations (e.g. SpanEvents -> SpanEventAttrs).
///
/// `source` contains the full sorted tables (borrowed, not consumed).
/// `out` is the single output batch being assembled for one range.
fn slice_children<const N: usize>(
    source_batch: &[Option<RecordBatch>; N],
    output_batch: &mut [Option<RecordBatch>; N],
    parent_slice: &RecordBatch,
    parent_type: ArrowPayloadType,
) -> Result<()> {
    let parent_relations = payload_relations(parent_type);

    for relation in parent_relations.relations {
        let key_col = relation.key_col;

        for &child_type in relation.child_types {
            let child_idx = POSITION_LOOKUP[child_type as usize];

            let Some(child_rb) = &source_batch[child_idx] else {
                continue;
            };

            let key_ranges = get_contiguous_id_ranges(parent_slice, key_col)?;
            let child_slice = take_child_rows(child_rb, &key_ranges)?;

            // Recurse before placing into `out` so we can borrow child_slice
            // without conflicting with the mutable borrow of `out`.
            if let Some(ref cs) = child_slice {
                let child_relation_info = payload_relations(child_type);
                if !child_relation_info.relations.is_empty() {
                    slice_children(source_batch, output_batch, cs, child_type)?;
                }
            }

            output_batch[child_idx] = child_slice;
        }
    }

    Ok(())
}

/// Map a `metric_type` column value to the corresponding data points
/// `ArrowPayloadType`.
fn payload_from_metric_type(metric_type: u8) -> Result<ArrowPayloadType> {
    let mt = MetricType::try_from(metric_type).map_err(|e| Error::UnrecognizedMetricType {
        metric_type: metric_type as i32,
        error: e,
    })?;
    match mt {
        MetricType::Gauge | MetricType::Sum => Ok(ArrowPayloadType::NumberDataPoints),
        MetricType::Histogram => Ok(ArrowPayloadType::HistogramDataPoints),
        MetricType::ExponentialHistogram => Ok(ArrowPayloadType::ExpHistogramDataPoints),
        MetricType::Summary => Ok(ArrowPayloadType::SummaryDataPoints),
        MetricType::Empty => Err(Error::EmptyMetricType),
    }
}

/// Extract the set of ids in use within a column. Returned as a list of inclusive
/// ranges where each range is a disjoint subset of the ids found.
///
/// As a precondiction, OTAP child tables are sorted by `parent_id` then `id`.
/// This means the `id` column is only sorted *within* each parent group and may
/// restart across groups. For example, consider an arbitrary child table with
/// two parents.
///
/// ```text
/// Example child record batch:
///
///   +-----------+----+
///   | parent_id | id |
///   +-----------+----+
///   |         0 |  1 |
///   |         0 |  2 |
///   |         1 |  0 |  <-- id restarts here
///   |         1 |  3 |
///   +-----------+----+
/// ```
///
/// In this case we'll initially produce two ranges, one for `[1, 2]` and one
/// for `[0, 3]`, which are overlapping. We need to reduce these to a single range
/// of `[0, 3]`.
///
/// # General Algorithm
///
/// 1. Start by creating windows for chunks of contiguous values.
/// 2. Verify that the chunks are contiguous and non-overlapping, if not, proceed
///    to the next step.
/// 3. Reduce and merge the ranges via [`reduce_ranges`]
///
/// Note: id columns are never dictionary-encoded; this dispatches on plain
/// UInt16/UInt32 only.
///
/// See [`test_split_traces_non_unique_child_ids`] for the exact scenario
/// that exercises this path.
fn get_contiguous_id_ranges(rb: &RecordBatch, id_path: &str) -> Result<Vec<RangeInclusive<u32>>> {
    let Some(col) = access_column(id_path, rb.schema_ref(), rb.columns()) else {
        return Ok(vec![]);
    };

    if col.is_empty() {
        return Ok(vec![]);
    }

    match col.data_type() {
        DataType::UInt16 => Ok(chunk_contiguous_generic::<UInt16Type>(&col)),
        DataType::UInt32 => Ok(chunk_contiguous_generic::<UInt32Type>(&col)),
        _ => Err(Error::UnsupportedParentIdType {
            actual: col.data_type().clone(),
        }),
    }
}

/// Type-specialized implementation of [`get_contiguous_ranges`]. The caller
/// must have verified that the column's `DataType` matches `T`.
fn chunk_contiguous_generic<T>(col: &ArrayRef) -> Vec<RangeInclusive<u32>>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Into<u32> + std::ops::Add<Output = T::Native> + From<u8>,
{
    let values = col
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .expect("Primitive array")
        .values();
    let one = T::Native::from(1u8);

    // Produce ranges naively from contiguous runs in buffer order.
    let mut ranges: Vec<RangeInclusive<T::Native>> = values
        .chunk_by(|a, b| *b == *a + one || *b == *a)
        .map(|chunk| chunk[0]..=chunk[chunk.len() - 1])
        .collect();

    // Check if they are already sorted and non-overlapping.
    let needs_fix = ranges
        .windows(2)
        .any(|w| *w[1].start() <= *w[0].end() + one);
    if needs_fix {
        reduce_ranges(&mut ranges, one);
    }

    ranges
        .into_iter()
        .map(|r| (*r.start()).into()..=(*r.end()).into())
        .collect()
}

/// Sort ranges by start then end and merge overlapping or adjacent entries
/// in place.
///
/// Uses a read/write cursor over the sorted vec. The `write` cursor tracks
/// the last merged range. For each `read` range, if it overlaps with or is
/// adjacent to the `write` range (i.e. `read.start <= write.end + 1`), we
/// extend `write.end`. Otherwise we advance `write` and copy `read` into
/// the new position. Finally, truncate to discard the leftover tail.
///
/// # Example
///
/// Given the naive chunks from `get_contiguous_ranges`:
///
/// ```text
///   input:  [1..=2, 0..=3]
///
///   sort:   [0..=3, 1..=2]
///
///   merge:  write = 0..=3
///           read  = 1..=2 -> 1 <= 3 + 1, so extend write
///                            2 < 3, so write stays 0..=3
///
///   result: [0..=3]
/// ```
fn reduce_ranges<N>(ranges: &mut Vec<RangeInclusive<N>>, one: N)
where
    N: Ord + Copy + std::ops::Add<Output = N>,
{
    ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()).then(a.end().cmp(b.end())));

    let mut write_idx = 0;
    for read_idx in 1..ranges.len() {
        if *ranges[read_idx].start() <= *ranges[write_idx].end() + one {
            // Overlapping or adjacent — extend the current range.
            if ranges[read_idx].end() > ranges[write_idx].end() {
                ranges[write_idx] = *ranges[write_idx].start()..=*ranges[read_idx].end();
            }
        } else {
            write_idx += 1;
            ranges[write_idx] = ranges[read_idx].clone();
        }
    }
    ranges.truncate(write_idx + 1);
}

/// Given a child table and a set of contiguous key ranges from the parent,
/// extract the matching rows. Uses `slice` for a single contiguous range
/// and `take_ranges` for multiple non-contiguous ranges.
fn take_child_rows(
    child_rb: &RecordBatch,
    key_ranges: &[RangeInclusive<u32>],
) -> Result<Option<RecordBatch>> {
    if key_ranges.is_empty() {
        return Ok(None);
    }

    let mut row_ranges: Vec<Range<usize>> = Vec::with_capacity(key_ranges.len());
    for key_range in key_ranges {
        let r = find_rows_by_parent_id_range(child_rb, key_range)?;
        if !r.is_empty() {
            row_ranges.push(r);
        }
    }

    match row_ranges.len() {
        0 => Ok(None),
        1 => {
            let r = &row_ranges[0];
            Ok(Some(child_rb.slice(r.start, r.end - r.start)))
        }
        _ => take_ranges(child_rb, &row_ranges)
            .map(Some)
            .map_err(|e| Error::Batching { source: e }),
    }
}

/// Binary-search the sorted `parent_id` column in `child_rb` for rows where
/// parent_id is in the range [key_min, key_max] (inclusive).
///
/// Returns `(start_row, end_row)` as a half-open range `[start_row, end_row)`.
/// The child table **must** be sorted by `parent_id` ascending.
///
/// Handles both plain (UInt16, UInt32) and dictionary-encoded parent_id columns.
/// For dictionary arrays the rows are still sorted by logical value, so binary
/// search is performed using the logical comparison via `row_partition_point`.
fn find_rows_by_parent_id_range(
    child_rb: &RecordBatch,
    key_range: &RangeInclusive<u32>,
) -> Result<Range<usize>> {
    let parent_id_col =
        child_rb
            .column_by_name(PARENT_ID)
            .ok_or_else(|| Error::ColumnNotFound {
                name: PARENT_ID.to_string(),
            })?;

    match parent_id_col.data_type() {
        DataType::UInt16 => find_rows::<UInt16Type>(parent_id_col, key_range),
        DataType::UInt32 => find_rows::<UInt32Type>(parent_id_col, key_range),
        DataType::Dictionary(key_dt, val_dt) => match (key_dt.as_ref(), val_dt.as_ref()) {
            (DataType::UInt8, DataType::UInt16) => {
                find_rows_dict::<UInt8Type, UInt16Type>(parent_id_col, key_range)
            }
            (DataType::UInt8, DataType::UInt32) => {
                find_rows_dict::<UInt8Type, UInt32Type>(parent_id_col, key_range)
            }
            (DataType::UInt16, DataType::UInt32) => {
                find_rows_dict::<UInt16Type, UInt32Type>(parent_id_col, key_range)
            }
            _ => Err(Error::ColumnDataTypeMismatch {
                name: PARENT_ID.to_string(),
                expect: DataType::UInt32,
                actual: parent_id_col.data_type().clone(),
            }),
        },
        other => Err(Error::ColumnDataTypeMismatch {
            name: PARENT_ID.to_string(),
            expect: DataType::UInt32,
            actual: other.clone(),
        }),
    }
}

fn find_rows<T>(col: &ArrayRef, key_range: &RangeInclusive<u32>) -> Result<Range<usize>>
where
    T: ArrowPrimitiveType,
    T::Native: Into<u32>,
{
    // safety: We checked the type of the column before calling this function
    let arr = col
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .expect("correct primitive type");
    let values = arr.values();
    let start = values.partition_point(|&v| v.into() < *key_range.start());
    let end = values.partition_point(|&v| v.into() <= *key_range.end());
    Ok(start..end)
}

/// Binary-search a dictionary-encoded `parent_id` column for the half-open
/// row range `[start, end)` where the logical parent_id is in
/// `[key_min, key_max]`.
///
/// Rows must be sorted by logical parent_id value (guaranteed by the sort
/// step). The physical dictionary keys are not necessarily sorted themselves,
/// so we binary-search on row indices using the logical comparison
/// `values[keys[i]]`.
fn find_rows_dict<K, V>(col: &ArrayRef, key_range: &RangeInclusive<u32>) -> Result<Range<usize>>
where
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
    V::Native: Into<u32>,
{
    let dict = col
        .as_any()
        .downcast_ref::<DictionaryArray<K>>()
        .expect("confirmed DictionaryArray");
    let vals = dict
        .values()
        .as_any()
        .downcast_ref::<PrimitiveArray<V>>()
        .expect("confirmed value type");
    let n = dict.len();

    // Dereference each row's dictionary key to its logical value.
    let logical_val = |i: usize| -> u32 {
        let key_idx = dict.key(i).expect("non-null parent_id");
        vals.value(key_idx).into()
    };

    let start = row_partition_point(n, |i| logical_val(i) < *key_range.start());
    let end = row_partition_point(n, |i| logical_val(i) <= *key_range.end());
    Ok(start..end)
}

/// Binary search over row indices `0..n`, finding the first row where `pred`
/// is false. Equivalent to `slice.partition_point` but without requiring a
/// concrete slice.
fn row_partition_point(n: usize, mut pred: impl FnMut(usize) -> bool) -> usize {
    let mut lo = 0;
    let mut hi = n;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use arrow::array::RecordBatch;

    use super::*;
    use crate::otap::transform::concatenate::concatenate;
    use crate::otap::transform::reindex::reindex;
    use crate::otap::transform::testing::{collect_row_ids, logs, metrics, payload_to_idx, traces};
    use crate::otap::transform::util::access_column;
    use crate::otap::{Logs, Metrics, OtapArrowRecords, Traces, num_items};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::otap_to_otlp;

    // ---- Logs tests ----
    // TODO test all the referential integrity violation variants

    #[test]
    #[rustfmt::skip]
    fn test_split_logs() {
        let log_ids       = vec![0, 1, 2, 3, 4, 5];
        let scope_ids     = vec![0, 0, 0, 1, 1, 2];
        let resource_ids  = vec![0, 0, 0, 1, 2, 3];
        let log_pids      = vec![0, 0, 1, 2, 3, 3];
        let scope_pids    = vec![0, 0, 1, 2, 2, 2];
        let resource_pids = vec![0, 0, 1, 2, 3, 3];

        // Plain UInt16 parent_ids
        test_split::<{ Logs::COUNT }>(&to_otap_logs, &[logs!(
            (Logs,
                ("id", UInt16, log_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (LogAttrs, 
                ("parent_id", UInt16, log_pids.clone())),
            (ScopeAttrs, 
                ("parent_id", UInt16, scope_pids.clone())),
            (ResourceAttrs, 
                ("parent_id", UInt16, resource_pids.clone()))
        )]);

        // Dict<UInt8, UInt16> parent_ids
        test_split::<{ Logs::COUNT }>(&to_otap_logs, &[logs!(
            (Logs,
                ("id", UInt16, log_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (LogAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3, 4, 5], log_pids.clone()))),
            (ScopeAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3, 4, 5], scope_pids.clone()))),
            (ResourceAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3, 4, 5], resource_pids.clone())))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_logs_singleton() {
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
        );

        let result = split::<{ Logs::COUNT }>(
            &mut [batch], NonZeroU32::new(1).unwrap(),
        ).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0]), 1);
    }

    #[test]
    fn test_split_empty_batches() {
        let mut logs: Vec<[Option<RecordBatch>; Logs::COUNT]> = vec![];
        let result = split::<{ Logs::COUNT }>(&mut logs, NonZeroU32::new(2).unwrap()).unwrap();
        assert_eq!(result.len(), 0);

        let mut metrics: Vec<[Option<RecordBatch>; Metrics::COUNT]> = vec![];
        let result =
            split::<{ Metrics::COUNT }>(&mut metrics, NonZeroU32::new(2).unwrap()).unwrap();
        assert_eq!(result.len(), 0);

        let mut traces: Vec<[Option<RecordBatch>; Traces::COUNT]> = vec![];
        let result = split::<{ Traces::COUNT }>(&mut traces, NonZeroU32::new(2).unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    // ---- Traces tests ----

    #[test]
    #[rustfmt::skip]
    fn test_split_traces() {
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
        test_split::<{ Traces::COUNT }>(&to_otap_traces, &[traces!(
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

        // Dict<UInt8, UInt16> / Dict<UInt8, UInt32> parent_ids
        test_split::<{ Traces::COUNT }>(&to_otap_traces, &[traces!(
            (Spans,
                ("id", UInt16, span_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (SpanAttrs,
                ("parent_id", (UInt8, UInt16), (vec![4u8, 3, 2, 1, 0], span_attr_pids.clone()))),
            (SpanEvents,
                ("id", UInt32, event_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![3u8, 2, 1, 0], event_pids.clone()))),
            (SpanEventAttrs,
                ("parent_id", (UInt8, UInt32), (vec![3u8, 2, 1, 0], event_attr_pids.clone()))),
            (SpanLinks,
                ("id", UInt32, link_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![1u8, 0], link_pids.clone()))),
            (ScopeAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2], scope_pids.clone()))),
            (ResourceAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], resource_pids.clone())))
        )]);

        // Dict<UInt16, UInt32> parent_ids for u32 columns
        test_split::<{ Traces::COUNT }>(&to_otap_traces, &[traces!(
            (Spans,
                ("id", UInt16, span_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (SpanAttrs,
                ("parent_id", (UInt8, UInt16), (vec![4u8, 3, 2, 1, 0], span_attr_pids.clone()))),
            (SpanEvents,
                ("id", UInt32, event_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![3u8, 2, 1, 0], event_pids.clone()))),
            (SpanEventAttrs,
                ("parent_id", (UInt16, UInt32), (vec![3u16, 2, 1, 0], event_attr_pids.clone()))),
            (SpanLinks,
                ("id", UInt32, link_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![1u8, 0], link_pids.clone()))),
            (ScopeAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2], scope_pids.clone()))),
            (ResourceAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], resource_pids.clone())))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_traces_singleton() {
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
        );

        let result = split::<{ Traces::COUNT }>(
            &mut [batch], NonZeroU32::new(1).unwrap(),
        ).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0]), 1);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_overlapping_parent_ranges() {
        // Tests the scenario where some different child ids are associated to the same
        // parent.
        let span_ids = vec![0u16, 1, 2, 3];
        let batches = vec![traces!(
            (Spans,
                ("id", UInt16, span_ids.clone())),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1, 2, 1, 2, 2, 3, 2]),
                ("parent_id", UInt16, vec![0u16, 0, 0, 1, 1, 1, 2, 3])),
            (SpanEventAttrs,
                ("parent_id", UInt32, vec![0u32, 1, 0, 1, 1, 0, 3, 2]))
        )];

        test_split::<{ Traces::COUNT }>(&to_otap_traces, &batches);
    }

    // ---- Metrics tests ----

    #[test]
    #[rustfmt::skip]
    fn test_split_metrics_number_dp() {
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

        // Plain parent_ids
        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
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
        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (MetricAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], metric_attr_pids.clone()))),
            (ScopeAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2], scope_pids.clone()))),
            (ResourceAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], resource_pids.clone()))),
            (NumberDataPoints,
                ("id", UInt32, dp_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3, 4, 5, 6, 7], dp_pids.clone()))),
            (NumberDpAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3, 4, 5, 6, 7], dp_attr_pids.clone()))),
            (NumberDpExemplars,
                ("id", UInt32, ex_ids.clone()),
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3], ex_pids.clone()))),
            (NumberDpExemplarAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2, 3], ex_attr_pids.clone())))
        )]);

        // Dict<UInt16, UInt32> parent_ids for u32 columns
        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids.clone()),
                ("scope.id", UInt16, scope_ids.clone()),
                ("resource.id", UInt16, resource_ids.clone())),
            (MetricAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], metric_attr_pids.clone()))),
            (ScopeAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2], scope_pids.clone()))),
            (ResourceAttrs,
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3], resource_pids.clone()))),
            (NumberDataPoints,
                ("id", UInt32, dp_ids.clone()),
                ("parent_id", (UInt8, UInt16), (vec![0u8, 1, 2, 3, 4, 5, 6, 7], dp_pids.clone()))),
            (NumberDpAttrs,
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3, 4, 5, 6, 7], dp_attr_pids.clone()))),
            (NumberDpExemplars,
                ("id", UInt32, ex_ids.clone()),
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3], ex_pids.clone()))),
            (NumberDpExemplarAttrs,
                ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2, 3], ex_attr_pids.clone())))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_metrics_histogram_dp() {
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

        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids),
                ("scope.id", UInt16, scope_ids),
                ("resource.id", UInt16, resource_ids)),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids)),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids)),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids)),
            (HistogramDataPoints,
                ("id", UInt32, dp_ids),
                ("parent_id", UInt16, dp_pids)),
            (HistogramDpAttrs,
                ("parent_id", UInt32, dp_attr_pids)),
            (HistogramDpExemplars,
                ("id", UInt32, ex_ids),
                ("parent_id", UInt32, ex_pids)),
            (HistogramDpExemplarAttrs,
                ("parent_id", UInt32, ex_attr_pids))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_metrics_exp_histogram_dp() {
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

        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids),
                ("scope.id", UInt16, scope_ids),
                ("resource.id", UInt16, resource_ids)),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids)),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids)),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids)),
            (ExpHistogramDataPoints,
                ("id", UInt32, dp_ids),
                ("parent_id", UInt16, dp_pids)),
            (ExpHistogramDpAttrs,
                ("parent_id", UInt32, dp_attr_pids)),
            (ExpHistogramDpExemplars,
                ("id", UInt32, ex_ids),
                ("parent_id", UInt32, ex_pids)),
            (ExpHistogramDpExemplarAttrs,
                ("parent_id", UInt32, ex_attr_pids))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_metrics_summary_dp() {
        let metric_ids       = vec![0u16, 1, 2, 3];
        let scope_ids        = vec![0u16, 0, 1, 1];
        let resource_ids     = vec![0u16, 0, 0, 1];
        let metric_attr_pids = vec![0u16, 1, 2, 3];
        let scope_pids       = vec![0u16, 0, 1];
        let resource_pids    = vec![0u16, 0, 1, 1];
        let dp_ids           = vec![0u32, 1, 2, 3, 4, 5, 6, 7];
        let dp_pids          = vec![0u16, 0, 1, 1, 1, 2, 3, 3];
        let dp_attr_pids     = vec![0u32, 1, 2, 3, 4, 5, 6, 7];

        test_split::<{ Metrics::COUNT }>(&to_otap_metrics, &[metrics!(
            (UnivariateMetrics,
                ("id", UInt16, metric_ids),
                ("scope.id", UInt16, scope_ids),
                ("resource.id", UInt16, resource_ids)),
            (MetricAttrs,
                ("parent_id", UInt16, metric_attr_pids)),
            (ScopeAttrs,
                ("parent_id", UInt16, scope_pids)),
            (ResourceAttrs,
                ("parent_id", UInt16, resource_pids)),
            (SummaryDataPoints,
                ("id", UInt32, dp_ids),
                ("parent_id", UInt16, dp_pids)),
            (SummaryDpAttrs,
                ("parent_id", UInt32, dp_attr_pids))
        )]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_split_metrics_oversized_singleton() {
        // A single metric with 6 data points at max_items=5.
        // Reproduces the batching_tests::test_comprehensive_batch_metrics
        // "over_limit_5" case. The metric cannot be split further so it
        // must be emitted as a singleton batch that exceeds the limit.
        let batch = metrics!(
            (UnivariateMetrics,
                ("id", UInt16, vec![0u16])),
            (NumberDataPoints,
                ("id", UInt32, vec![0u32, 1, 2, 3, 4, 5]),
                ("parent_id", UInt16, vec![0u16, 0, 0, 0, 0, 0]))
        );

        let result = split::<{ Metrics::COUNT }>(
            &mut [batch],
            NonZeroU32::new(5).unwrap(),
        )
        .unwrap();

        // The oversized metric is emitted as a single batch.
        assert_eq!(result.len(), 1);
        assert_eq!(num_items(&result[0]), 6);
    }

    fn test_split<const N: usize>(
        to_otap: &dyn Fn(&[Option<RecordBatch>; N]) -> OtapArrowRecords,
        batches: &[[Option<RecordBatch>; N]],
    ) {
        let root_type = root_type_for::<N>();
        let root_idx = POSITION_LOOKUP[root_type as usize];

        // Reindex + concatenate input into a single OTLP message for equivalence.
        let input_otlp = {
            let mut input_clone = batches.to_vec();
            reindex::<N>(&mut input_clone).unwrap();
            let input_combined = concatenate::<N>(&mut input_clone).unwrap();
            otap_to_otlp(&to_otap(&input_combined))
        };

        // Extract expected root IDs and total item count from the inputs.
        let mut expected_ids: HashSet<u32> = HashSet::new();
        for batch in batches {
            if let Some(rb) = &batch[root_idx] {
                expected_ids.extend(root_ids(rb));
            }
        }
        let expected_items: usize = batches.iter().map(num_items).sum();

        for i in get_split_sizes(expected_items) {
            let mut result =
                split::<N>(&mut batches.to_vec(), NonZeroU32::new(i as u32).unwrap()).unwrap();

            // Collect root ids across all output batches.
            let mut split_ids: HashSet<u32> = HashSet::new();
            for batch in &result {
                let rb = batch[root_idx].as_ref().expect("root batch present");
                split_ids.extend(root_ids(rb));
            }
            assert_eq!(split_ids, expected_ids);

            // Total item count must be preserved.
            let result_items: usize = result.iter().map(num_items).sum();
            assert_eq!(result_items, expected_items);

            // For metrics, verify data point row counts are preserved per type.
            if N == Metrics::COUNT {
                for dp_type in [
                    ArrowPayloadType::NumberDataPoints,
                    ArrowPayloadType::SummaryDataPoints,
                    ArrowPayloadType::HistogramDataPoints,
                    ArrowPayloadType::ExpHistogramDataPoints,
                ] {
                    let idx = POSITION_LOOKUP[dp_type as usize];
                    let input_rows: usize = batches
                        .iter()
                        .map(|b| b[idx].as_ref().map_or(0, |rb| rb.num_rows()))
                        .sum();
                    let output_rows: usize = result
                        .iter()
                        .map(|b| b[idx].as_ref().map_or(0, |rb| rb.num_rows()))
                        .sum();
                    assert_eq!(
                        input_rows, output_rows,
                        "data point row count mismatch for {:?}",
                        dp_type
                    );
                }
            }

            // Referential integrity for every output batch.
            for batch in &result {
                assert_referential_integrity::<N>(batch, root_type);
            }

            // Reindex + concatenate output and assert OTLP equivalence.
            {
                reindex::<N>(&mut result).unwrap();
                let output_combined = concatenate::<N>(&mut result).unwrap();
                let output_otlp = otap_to_otlp(&to_otap(&output_combined));
                assert_equivalent(&[input_otlp.clone()], &[output_otlp]);
            }
        }
    }

    fn to_otap_logs(batch: &[Option<RecordBatch>; Logs::COUNT]) -> OtapArrowRecords {
        OtapArrowRecords::Logs(Logs {
            batches: batch.clone(),
        })
    }

    fn to_otap_metrics(batch: &[Option<RecordBatch>; Metrics::COUNT]) -> OtapArrowRecords {
        OtapArrowRecords::Metrics(Metrics {
            batches: batch.clone(),
        })
    }

    fn to_otap_traces(batch: &[Option<RecordBatch>; Traces::COUNT]) -> OtapArrowRecords {
        OtapArrowRecords::Traces(Traces {
            batches: batch.clone(),
        })
    }

    /// Derive the root `ArrowPayloadType` from the const generic batch size.
    fn root_type_for<const N: usize>() -> ArrowPayloadType {
        match N {
            Logs::COUNT => ArrowPayloadType::Logs,
            Metrics::COUNT => ArrowPayloadType::UnivariateMetrics,
            Traces::COUNT => ArrowPayloadType::Spans,
            _ => unreachable!("unsupported batch size {N}"),
        }
    }

    /// Compute a representative set of split sizes to exercise for a given
    /// root id list. Covers small counts exhaustively and samples boundary
    /// values for larger ones.
    fn get_split_sizes(total_items: usize) -> Vec<usize> {
        if total_items == 0 {
            return vec![1];
        }

        if total_items == 1 {
            return vec![1, 2];
        }

        if total_items < 10 {
            return (1..=total_items + 1).collect();
        }

        let sqrt = total_items.isqrt();
        let half = total_items / 2;
        let mut split_sizes: Vec<usize> = Vec::with_capacity(sqrt + 1 + 6);
        split_sizes.extend(1..=sqrt);
        split_sizes.push(half - 1);
        split_sizes.push(half);
        split_sizes.push(half + 1);
        split_sizes.push(total_items - 1);
        split_sizes.push(total_items);
        split_sizes.push(total_items + 1);
        split_sizes
    }

    /// Collect the per-row values from the "id" column of a RecordBatch.
    fn root_ids(rb: &RecordBatch) -> Vec<u32> {
        let col = access_column("id", rb.schema_ref(), rb.columns()).expect("missing id column");
        collect_row_ids(col.as_ref())
    }

    /// Assert that every child parent_id in the batch is present in the parent's
    /// key column.  Works for both flat (id) and nested (resource.id, scope.id).
    fn assert_referential_integrity<const N: usize>(
        batch: &[Option<RecordBatch>; N],
        parent_type: ArrowPayloadType,
    ) {
        use crate::otap::transform::util::payload_relations;

        let parent_idx = payload_to_idx(parent_type);
        let parent_rb = batch[parent_idx].as_ref().unwrap();

        for relation in payload_relations(parent_type).relations {
            let Some(parent_col) = access_column(
                relation.key_col,
                parent_rb.schema_ref(),
                parent_rb.columns(),
            ) else {
                continue;
            };

            let parent_id_set: HashSet<u32> =
                collect_row_ids(parent_col.as_ref()).into_iter().collect();

            for &child_type in relation.child_types {
                let child_idx = payload_to_idx(child_type);
                let Some(child_rb) = &batch[child_idx] else {
                    continue;
                };
                let Some(child_col) = child_rb.column_by_name("parent_id") else {
                    continue;
                };
                for pid in collect_row_ids(child_col.as_ref()) {
                    assert!(
                        parent_id_set.contains(&pid),
                        "Child {:?} has parent_id {} not present in parent {:?} key column '{}'",
                        child_type,
                        pid,
                        parent_type,
                        relation.key_col,
                    );
                }

                // Recurse for deeper relations (e.g. SpanEvents -> SpanEventAttrs)
                let child_relations = payload_relations(child_type);
                if !child_relations.relations.is_empty() {
                    assert_referential_integrity(batch, child_type);
                }
            }
        }
    }
}
