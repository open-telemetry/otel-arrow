// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Add, AddAssign, Range, Sub, SubAssign};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowNativeTypeOp, ArrowPrimitiveType, AsArray, DictionaryArray,
    PrimitiveArray, RecordBatch,
};
use arrow::buffer::ScalarBuffer;
use arrow::compute::kernels::aggregate::{max_array, min_array};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, ArrowNumericType, DataType, UInt8Type, UInt16Type,
    UInt32Type,
};

use crate::error::{Error, Result};
use crate::otap::transform::transport_optimize::remove_transport_optimized_encodings;
use crate::otap::transform::util::{
    extract_id_column, payload_to_idx, remove_record_batch_ranges, replace_column,
    sort_record_batch_by_indices,
};
use crate::otap::{Logs, Metrics, OtapBatchStore, Traces};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{ID, PARENT_ID};

use super::util::{IdColumnType, PrimaryIdInfo, payload_relations};

/// Reindex the provided record batches in place such that all IDs are unique
/// for each payload type across all batches. This makes it safe to concatenate
/// these record batches.
///
/// Note: reindex also removes the transport optimized encoding.
/// Note: There are opportunities for optimization here, some of which are captured
/// in https://github.com/open-telemetry/otel-arrow/issues/1926
pub fn reindex<const N: usize>(batches: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    if batches.is_empty() || batches.len() == 1 {
        return Ok(());
    }

    match N {
        Logs::COUNT => reindex_logs::<{ N }>(batches),
        Metrics::COUNT => reindex_metrics::<{ N }>(batches),
        Traces::COUNT => reindex_traces::<{ N }>(batches),
        _ => unreachable!(),
    }
}

struct MultiBatchStore<'a, T, const N: usize> {
    batches: &'a mut [[Option<RecordBatch>; N]],
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: OtapBatchStore, const N: usize> MultiBatchStore<'a, T, N> {
    fn new(batches: &'a mut [[Option<RecordBatch>; N]]) -> Self {
        Self {
            batches,
            _phantom: std::marker::PhantomData,
        }
    }

    fn get_mut(&mut self, idx: usize) -> &mut [Option<RecordBatch>; N] {
        &mut self.batches[idx]
    }

    fn len(&self) -> usize {
        self.batches.len()
    }

    fn select(&self, payload_type: ArrowPayloadType) -> impl Iterator<Item = &RecordBatch> {
        self.batches
            .iter()
            .filter_map(move |batch| batch[payload_to_idx(payload_type)].as_ref())
    }

    fn remove_transport_optimized_encodings(
        &mut self,
        payload_type: ArrowPayloadType,
    ) -> Result<()> {
        for rb in self.select_mut(payload_type) {
            *rb = remove_transport_optimized_encodings(payload_type, rb)?;
        }
        Ok(())
    }

    fn select_mut(
        &mut self,
        payload_type: ArrowPayloadType,
    ) -> impl Iterator<Item = &mut RecordBatch> {
        self.batches
            .iter_mut()
            .filter_map(move |batch| batch[payload_to_idx(payload_type)].as_mut())
    }
}

pub fn reindex_logs<const N: usize>(logs: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut store = MultiBatchStore::<Logs, { N }>::new(logs);
    reindex_batch_store(&mut store)
}

pub fn reindex_metrics<const N: usize>(metrics: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut store = MultiBatchStore::<Metrics, { N }>::new(metrics);
    reindex_batch_store(&mut store)
}

pub fn reindex_traces<const N: usize>(traces: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut store = MultiBatchStore::<Traces, { N }>::new(traces);
    reindex_batch_store(&mut store)
}

/// Generic reindexing function that works for any OtapBatchStore
///
/// Iterates over all allowed payload types, gets their relations, and reindexes
/// each ID column and its corresponding parent_id columns in child tables.
fn reindex_batch_store<S, const N: usize>(store: &mut MultiBatchStore<'_, S, N>) -> Result<()>
where
    S: OtapBatchStore,
{
    for payload_type in S::allowed_payload_types() {
        store.remove_transport_optimized_encodings(*payload_type)?;
    }

    for &payload_type in S::allowed_payload_types() {
        let info = payload_relations(payload_type);

        // Check for obvious overflow on the primary ID column.
        if let Some(ref primary_id_info) = info.primary_id {
            check_primary_id_for_overflow(store, payload_type, primary_id_info)?;
        }

        for relation in info.relations {
            let is_primary = Some(relation.key_col) == info.primary_id.as_ref().map(|id| id.name);
            match relation.size {
                IdColumnType::U16 => {
                    reindex_id_column::<UInt16Type, S, N>(
                        store,
                        payload_type,
                        relation.child_types,
                        relation.key_col,
                        is_primary,
                        relation.size,
                    )?;
                }
                IdColumnType::U32 => {
                    reindex_id_column::<UInt32Type, S, N>(
                        store,
                        payload_type,
                        relation.child_types,
                        relation.key_col,
                        is_primary,
                        relation.size,
                    )?;
                }
            }
        }
    }

    Ok(())
}

// For a given primary id column, determine the count of Ids that exist across
// every record batch and determine if it will fit in the type for that column.
//
// # Returns
// * `Ok(())` if the primary id column will fit in the type
// * `Err` if the primary id column will not fit in the type
fn check_primary_id_for_overflow<S, const N: usize>(
    store: &MultiBatchStore<'_, S, N>,
    payload_type: ArrowPayloadType,
    id_info: &PrimaryIdInfo,
) -> Result<()>
where
    S: OtapBatchStore,
{
    let mut count: u64 = 0;
    for batch in store.select(payload_type) {
        let Ok(id_col) = extract_id_column(batch, id_info.name) else {
            continue;
        };
        count += id_col.len() as u64;
    }

    // TODO: Consider supporting u16::MAX + 1. This is a little tricky because we
    // do offset math with the Native type which causes us to overflow right
    // at the top. We could maybe try to do offset math with u64, but we will
    // have to constantly cast back and forth and it won't be as clear if we've
    // made a mistake somewhere. Only consequence is max batch size is 1 less.
    if count > id_info.size.max() {
        return Err(Error::TooManyItems {
            payload_type,
            count: count as usize,
            max: id_info.size.max(),
            message: "Too many items to reindex".to_string(),
        });
    }

    Ok(())
}

/// Two-pass reindexing for an ID column and its corresponding parent_id
/// columns in child tables.
///
/// Pass 1 (`gather_column_stats`): read-only scan to collect per-batch min/max,
/// length, and strategy (offset vs compact).
///
/// Pass 2 (greedy execution): for each batch, decide whether to apply a cheap
/// uniform offset or fall back to sort+compact. The decision uses per-batch
/// stats and a global budget: batches that *must* compact always do, and
/// remaining batches compact only when needed to stay within the ID type limit.
fn reindex_id_column<T, S, const N: usize>(
    store: &mut MultiBatchStore<'_, S, N>,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
    is_primary: bool,
    size: IdColumnType,
) -> Result<()>
where
    T: ArrowNumericType,
    T::Native: Ord
        + Copy
        + Add<Output = T::Native>
        + Sub<Output = T::Native>
        + AddAssign
        + SubAssign
        + From<u8>
        + ArrowNativeTypeOp,
    S: OtapBatchStore,
{
    // -- Pass 1: gather statistics --
    let stats = gather_column_stats::<T, S, N>(
        store,
        parent_payload_type,
        child_payload_types,
        id_column_path,
        is_primary,
    )?;

    // Compute the ID headroom budget. When total_ids_needed exceeds the
    // column-type limit we must compact enough batches to fit.
    let limit: u64 = size.max();

    let total_ids_needed: u64 = stats
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.max_ids_needed as u64)
        .sum();
    let need_to_save: i64 = total_ids_needed as i64 - limit as i64;

    // -- Pass 2: greedy execution --
    let mut offset = T::Native::from(0);
    let mut current_saved: i64 = 0;

    #[allow(clippy::needless_range_loop)]
    for i in 0..store.len() {
        let Some(ref stat) = stats[i] else {
            continue;
        };

        let must_compact = stat.strategy == ReindexStrategy::CompactOnly
            || (need_to_save > 0 && current_saved < need_to_save);

        if must_compact {
            let new_offset = apply_compact_reindex::<T, S, N>(
                store,
                i,
                parent_payload_type,
                child_payload_types,
                id_column_path,
                offset,
            )?;
            let ids_consumed = new_offset.as_usize() - offset.as_usize();
            current_saved += stat.max_ids_needed as i64 - ids_consumed as i64;
            offset = new_offset;
        } else {
            offset = apply_offset_reindex::<T, S, N>(
                store,
                i,
                parent_payload_type,
                child_payload_types,
                id_column_path,
                stat.min,
                stat.max,
                offset,
            )?;
        }
    }

    Ok(())
}

/// Returns (min, max) of an ID column, handling nulls and dictionary encoding.
///
/// Returns `None` if the column is empty or all null.
fn id_column_min_max<T>(col: &dyn Array) -> Result<Option<(T::Native, T::Native)>>
where
    T: ArrowNumericType,
    T::Native: ArrowNativeTypeOp,
{
    let values = materialize_id_values::<T>(col)?;
    let Some(min) = min_array::<T, _>(values) else {
        return Ok(None);
    };
    // SAFETY: presence of a min value implies at least one non-null element,
    // so max must also be Some.
    let max = max_array::<T, _>(values).expect("max must exist when min exists");
    Ok(Some((min, max)))
}

/// Fast path: apply a uniform offset to the ID column and all child parent_id
/// columns. Works for any column (primary or non-primary) as long as the values
/// array is offset-safe (dict-encoded columns remap values, preserving key
/// structure).
///
/// Returns the new offset (= old offset + span).
fn apply_offset_reindex<T, S, const N: usize>(
    store: &mut MultiBatchStore<'_, S, N>,
    batch_index: usize,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
    min: T::Native,
    max: T::Native,
    offset: T::Native,
) -> Result<T::Native>
where
    T: ArrowNumericType,
    T::Native: Ord
        + Copy
        + Add<Output = T::Native>
        + Sub<Output = T::Native>
        + AddAssign
        + SubAssign
        + From<u8>
        + ArrowNativeTypeOp,
    S: OtapBatchStore,
{
    let parent_idx = payload_to_idx(parent_payload_type);
    let parent_rb = store.get_mut(batch_index)[parent_idx]
        .take()
        .expect("batch must exist for non-None stat");

    let id_col = extract_id_column(&parent_rb, id_column_path)?;
    let id_values = materialize_id_values::<T>(id_col.as_ref())?;

    let (off, sign) = if min <= offset {
        (offset - min, Sign::Positive)
    } else {
        (min - offset, Sign::Negative)
    };

    let mut ids = id_values.values().to_vec();
    apply_uniform_offset(&mut ids, off, sign);
    let parent_rb = replace_id_column::<T>(parent_rb, id_column_path, ids)?;
    store.get_mut(batch_index)[parent_idx] = Some(parent_rb);

    for &child_payload_type in child_payload_types {
        let child_idx = payload_to_idx(child_payload_type);
        if let Some(child_rb) = store.get_mut(batch_index)[child_idx].take() {
            let child_rb = fast_path_reindex_child::<T>(child_rb, PARENT_ID, off, sign)?;
            store.get_mut(batch_index)[child_idx] = Some(child_rb);
        }
    }

    let span = max - min + T::Native::from(1);
    Ok(offset + span)
}

/// Slow path: sort the ID column, create old->new mappings, and remap both the
/// parent ID column and all child parent_id columns.
///
/// Returns the new offset after compaction.
fn apply_compact_reindex<T, S, const N: usize>(
    store: &mut MultiBatchStore<'_, S, N>,
    batch_index: usize,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
    offset: T::Native,
) -> Result<T::Native>
where
    T: ArrowNumericType,
    T::Native: Ord
        + Copy
        + Add<Output = T::Native>
        + Sub<Output = T::Native>
        + AddAssign
        + SubAssign
        + From<u8>
        + ArrowNativeTypeOp,
    S: OtapBatchStore,
{
    let parent_idx = payload_to_idx(parent_payload_type);
    let parent_rb = store.get_mut(batch_index)[parent_idx]
        .take()
        .expect("batch must exist for non-None stat");

    let id_col = extract_id_column(&parent_rb, id_column_path)?;
    let id_values = materialize_id_values::<T>(id_col.as_ref())?;
    let mut ids = id_values.values().to_vec();

    let (mappings, new_offset) = if ids.is_sorted() {
        let (m, o) = create_mappings::<T>(&ids, offset)?;
        assert!(apply_mappings::<T>(&mut ids, &m).is_none());
        (m, o)
    } else {
        let sort_indices = sort_vec_to_indices(&ids);
        let mut sorted_ids = vec![T::Native::default(); ids.len()];
        take_vec(&ids, &mut sorted_ids, &sort_indices);
        let (m, o) = create_mappings::<T>(&sorted_ids, offset)?;
        assert!(apply_mappings::<T>(&mut sorted_ids, &m).is_none());
        untake_vec(&sorted_ids, &mut ids, &sort_indices);
        (m, o)
    };

    let parent_rb = replace_id_column::<T>(parent_rb, id_column_path, ids)?;
    store.get_mut(batch_index)[parent_idx] = Some(parent_rb);

    for &child_payload_type in child_payload_types {
        let child_idx = payload_to_idx(child_payload_type);
        if let Some(child_rb) = store.get_mut(batch_index)[child_idx].take() {
            let child_rb = reindex_child_column::<T>(child_rb, PARENT_ID, &mappings)?;
            store.get_mut(batch_index)[child_idx] = Some(child_rb);
        }
    }

    Ok(new_offset)
}

/// Check if all of a child's parent_id values for batch_index are within
/// [min, max]. Returns false if the child has orphan references.
fn children_in_parent_range<T, S, const N: usize>(
    store: &MultiBatchStore<'_, S, N>,
    batch_index: usize,
    child_payload_type: ArrowPayloadType,
    parent_min: T::Native,
    parent_max: T::Native,
) -> Result<bool>
where
    T: ArrowNumericType,
    T::Native: Ord + Copy + ArrowNativeTypeOp,
    S: OtapBatchStore,
{
    let child_idx = payload_to_idx(child_payload_type);
    let Some(child_batch) = &store.batches[batch_index][child_idx] else {
        return Ok(true);
    };
    let Ok(child_col) = extract_id_column(child_batch, PARENT_ID) else {
        return Ok(true);
    };
    let Some((child_min, child_max)) = id_column_min_max::<T>(child_col.as_ref())? else {
        return Ok(true);
    };
    Ok(child_min >= parent_min && child_max <= parent_max)
}

/// Applies a uniform offset to all values in a slice.
fn apply_uniform_offset<T>(values: &mut [T], offset: T, sign: Sign)
where
    T: AddAssign + SubAssign + Copy,
{
    match sign {
        Sign::Positive => values.iter_mut().for_each(|v| *v += offset),
        Sign::Negative => values.iter_mut().for_each(|v| *v -= offset),
    }
}

/// Fast-path child reindexing: applies a uniform offset to the parent_id
/// column in a child record batch.
fn fast_path_reindex_child<T>(
    rb: RecordBatch,
    column_path: &str,
    offset: T::Native,
    sign: Sign,
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + AddAssign + SubAssign + ArrowNativeType,
{
    let id_col = extract_id_column(&rb, column_path)?;
    let id_values = materialize_id_values::<T>(id_col.as_ref())?;
    let mut new_values = id_values.values().to_vec();
    apply_uniform_offset(&mut new_values, offset, sign);
    replace_id_column::<T>(rb, column_path, new_values)
}

/// Reindexes a child id column in a record batch using the provided mappings.
/// Parent ids take a slightly different path because they need to separate the
/// creation of the mappings from applying those mappings to potentially multiple
/// child batches.
fn reindex_child_column<T>(
    rb: RecordBatch,
    column_path: &str,
    mappings: &[IdMapping<T::Native>],
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + AddAssign + SubAssign + ArrowNativeType,
{
    // Materialize the id values. In the case of a dictionary this is the
    // values array and does not include the keys.
    let id_col = extract_id_column(&rb, column_path)?;
    let id_values = materialize_id_values::<T>(id_col.as_ref())?;
    let mut id_values = id_values.values().to_vec();

    let value_sort_indices = sort_vec_to_indices(&id_values);
    let value_sort_indices = PrimitiveArray::from(value_sort_indices);
    let mut new_ids = vec![T::Native::default(); id_values.len()];
    take_vec(&id_values, &mut new_ids, value_sort_indices.values());
    if let Some(violations) = apply_mappings::<T>(&mut new_ids, mappings) {
        // We may have integrity violations in some number of ranges. We need to eliminate
        // them because we're on the reindexing path where we're squashing all ids
        // to contiguous ranges starting at 0, so any strays left behind may accidentally
        // be associated to ids in other batches if we apply some offset to them.
        //
        // For primitive columns the violation ranges correspond directly to rows in
        // the sorted record batch so we sort, remove the rows, compact new_ids, and
        // replace the column.
        //
        // For dictionary columns the violations are in the values array, not the
        // keys. In this case the violations could be for unreferenced dict values,
        // so we map value-level redactions to key-level redactions to see what
        // needs to be removed.
        match id_col.data_type() {
            DataType::Dictionary(key_type, _) => {
                // Determine which value violations correspond to actual rows.
                let key_redactions = match key_type.as_ref() {
                    DataType::UInt8 => map_value_redactions_to_key_redactions::<UInt8Type>(
                        id_col.as_ref(),
                        &violations,
                    ),
                    DataType::UInt16 => map_value_redactions_to_key_redactions::<UInt16Type>(
                        id_col.as_ref(),
                        &violations,
                    ),
                    _ => {
                        return Err(Error::UnsupportedDictionaryKeyType {
                            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                            actual: key_type.as_ref().clone(),
                        });
                    }
                };

                // Unsort the remapped values back to original order. Violation
                // positions contain garbage but no key references them.
                untake_vec(&new_ids, &mut id_values, value_sort_indices.values());

                let rb = if !key_redactions.is_empty() {
                    // Genuine violations - sort batch by the same key order
                    // used to produce the key redaction ranges, then remove.
                    let sort_indices = arrow::compute::sort_to_indices(&id_col, None, None)
                        .map_err(|e| Error::Batching { source: e })?;
                    let rb = sort_record_batch_by_indices(rb, &sort_indices)?;
                    remove_record_batch_ranges(&rb, &key_redactions)
                        .map_err(|e| Error::Batching { source: e })?
                } else {
                    rb
                };

                return replace_id_column::<T>(rb, column_path, id_values);
            }
            _ => {
                // Primitive column: sort batch, remove violation rows, compact.
                let rb = sort_record_batch_by_indices(rb, &value_sort_indices)?;
                let rb = remove_record_batch_ranges(&rb, &violations)
                    .map_err(|e| Error::Batching { source: e })?;
                remove_vec_ranges(&mut new_ids, &violations);
                return replace_id_column::<T>(rb, column_path, new_ids);
            }
        }
    }

    // Unsort the IDs. Note that since `take` and `untake` can't be done
    // in place, we re-use the original id vec as the destination.
    untake_vec(&new_ids, &mut id_values, value_sort_indices.values());
    replace_id_column::<T>(rb, column_path, id_values)
}

/// Removes elements at the given ranges from a vector in place.
/// Ranges must be sorted and non-overlapping.
fn remove_vec_ranges<T>(vec: &mut Vec<T>, ranges: &[Range<usize>]) {
    // Process in reverse so earlier indices remain valid
    for range in ranges.iter().rev() {
        drop(vec.drain(range.clone()));
    }
}

/// Maps value-level redaction ranges to key-level (row-level) redaction ranges
/// for dictionary-encoded columns.
///
/// # Background
///
/// When [reindex_child_column] processes a dictionary-encoded id column, it
/// operates on the dictionary **values** array rather
/// than the per-row keys. Not all dictionary values are necessarily referenced
/// by a key which is a problem because [apply_mappings] may flag values that
/// are not actually referenced in any row.
///
/// This function determines which, if any, flagged values are referenced by
/// keys and returns ranges of indices for the keys which need to be removed.
///
/// # Algorithm
///
/// Dictionary keys are indices into the values array, and the redaction ranges
/// are also indices into the values array. Both are directly comparable. We:
///
/// 1. Sort the keys. Since the redaction ranges are sorted and non-overlapping
///    by construction, we merge-scan them in a single pass.
/// 2. If a key falls inside a redaction range, that row has a genuine integrity
///    violation.
///
/// The output ranges are positions in the sorted-key order, which corresponds
/// to rows in the record batch after sorting by
/// `arrow::compute::sort_to_indices(&id_col)`.
///
/// # Example
///
/// ```text
/// Dictionary values array:  [0, 1, 2, 3, 4]   (indices 0..5)
/// Dictionary keys array:    [0, 2, 4, 1]      (4 rows)
/// Value redactions:         [3..5]            (values at indices 3,4 flagged)
///
/// ```
///
/// In this case:
///
/// Value index 3 was NOT referenced by any key (spurious).
/// Value index 4 WAS referenced by key 4 (genuine) -> Output is [3..4)
///
fn map_value_redactions_to_key_redactions<K>(
    id_col: &dyn Array,
    value_redactions: &[Range<usize>],
) -> Vec<Range<usize>>
where
    K: ArrowDictionaryKeyType,
    K::Native: Ord,
{
    debug_assert!(
        value_redactions.windows(2).all(|w| w[0].end <= w[1].start),
        "value_redactions must be sorted and non-overlapping"
    );

    if value_redactions.is_empty() {
        return Vec::new();
    }

    // safety: Caller checks the type before
    let dict = id_col.as_dictionary::<K>();

    // Keys are indices into the values array — directly comparable to the
    // redaction ranges which are also value-array indices. We keep the keys
    // in their native type (u8 or u16) and cast the range bounds to match.
    let mut sorted_keys: Vec<K::Native> = dict.keys().values().to_vec();
    sorted_keys.sort_unstable();

    // Merge-scan sorted keys against value redaction ranges.
    let mut key_redactions = Vec::new();
    let mut key_idx = 0;
    let mut redaction_idx = 0;
    let mut current_start: Option<usize> = None;

    while key_idx < sorted_keys.len() && redaction_idx < value_redactions.len() {
        let key = sorted_keys[key_idx];
        let redaction = &value_redactions[redaction_idx];
        // Cast range bounds to the key type. Safe because dictionary keys
        // index into the values array, so all indices fit in K::Native.
        // safety: K::Native is at most 16 bits, so we should be able to cast that into
        // a usize on any 32 bit or larger platform.
        let redaction_start = K::Native::from_usize(redaction.start).expect("usize > 16 bits");
        let redaction_end = K::Native::from_usize(redaction.end).expect("usize > 16 bits");

        // Key is before this redaction range - not a violation.
        if key < redaction_start {
            if let Some(start) = current_start.take() {
                key_redactions.push(start..key_idx);
            }

            key_idx += 1;
            continue;
        }

        // Key is past this redaction range - advance to the next range.
        if key >= redaction_end {
            if let Some(start) = current_start.take() {
                key_redactions.push(start..key_idx);
            }

            redaction_idx += 1;
            continue;
        }

        // Key is inside the redaction range - genuine violation.
        if current_start.is_none() {
            current_start = Some(key_idx);
        }
        key_idx += 1;
    }

    if let Some(start) = current_start {
        key_redactions.push(start..key_idx);
    }

    key_redactions
}

fn replace_id_column<T>(
    rb: RecordBatch,
    column_path: &str,
    new_ids: Vec<T::Native>,
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    T::Native: ArrowNativeType,
{
    let id_col = extract_id_column(&rb, column_path)?;
    let new_ids_array = PrimitiveArray::<T>::new(ScalarBuffer::from(new_ids), None);
    let (schema, mut columns, _) = rb.into_parts();
    let new_column = replace_ids::<T>(id_col.as_ref(), new_ids_array);
    replace_column(column_path, None, &schema, &mut columns, new_column);
    let rb =
        RecordBatch::try_new(schema, columns).map_err(|e| Error::UnexpectedRecordBatchState {
            reason: format!("Failed to create batch: {}", e),
        })?;

    Ok(rb)
}

/// Sorts a vector of values and returns the resulting sort indices
fn sort_vec_to_indices<T: Ord>(values: &[T]) -> Vec<u32> {
    let mut indices: Vec<u32> = (0u32..values.len() as u32).collect();
    indices.sort_unstable_by_key(|&i| &values[i as usize]);
    indices
}

/// Materializes an ID array from either a direct array or dictionary values.
///
/// For dictionary arrays, returns the VALUES array (unique dictionary entries), not the per-row
/// logical values. This is intentional: callers remap just the dictionary values, and the
/// dictionary keys preserve the per-row structure automatically.
fn materialize_id_values<T>(array: &dyn Array) -> Result<&PrimitiveArray<T>>
where
    T: ArrowPrimitiveType,
{
    let id_arr = match array.data_type() {
        data_type if data_type == &T::DATA_TYPE => array.as_primitive::<T>(),
        DataType::Dictionary(key_type, value_type) if value_type.as_ref() == &T::DATA_TYPE => {
            match key_type.as_ref() {
                DataType::UInt8 => array.as_dictionary::<UInt8Type>().values().as_primitive(),
                DataType::UInt16 => array.as_dictionary::<UInt16Type>().values().as_primitive(),
                _ => {
                    return Err(Error::UnsupportedDictionaryKeyType {
                        expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                        actual: key_type.as_ref().clone(),
                    });
                }
            }
        }
        _ => {
            return Err(Error::ColumnDataTypeMismatch {
                name: ID.to_string(),
                expect: T::DATA_TYPE,
                actual: array.data_type().clone(),
            });
        }
    };

    Ok(id_arr)
}

/// Creates a replacement ID column, preserving dictionary encoding if the original was
/// dictionary-encoded. For plain columns, returns the new values directly. For dictionary
/// columns, builds a new DictionaryArray with the original keys and the remapped values.
fn replace_ids<T>(original: &dyn Array, new_values: PrimitiveArray<T>) -> ArrayRef
where
    T: ArrowPrimitiveType,
{
    match original.data_type() {
        DataType::Dictionary(key_type, _) => match key_type.as_ref() {
            DataType::UInt8 => {
                let dict = original.as_dictionary::<UInt8Type>();
                Arc::new(DictionaryArray::new(
                    dict.keys().clone(),
                    Arc::new(new_values),
                ))
            }
            DataType::UInt16 => {
                let dict = original.as_dictionary::<UInt16Type>();
                Arc::new(DictionaryArray::new(
                    dict.keys().clone(),
                    Arc::new(new_values),
                ))
            }
            _ => Arc::new(new_values),
        },
        _ => Arc::new(new_values),
    }
}

/// Sign of an offset operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    /// Add the offset
    Positive,
    /// Subtract the offset
    Negative,
}

/// Represents a contiguous range of IDs with an offset to apply
#[derive(Debug, Clone)]
struct IdMapping<T> {
    /// The first ID value in this range
    start_id: T,
    /// The last ID value in this range
    end_id: T,
    /// Offset to add or subtract from IDs in this range
    offset: T,
    /// Sign of the offset operation
    sign: Sign,
}

/// Whether a batch can use the fast offset path or must compact (sort + remap).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReindexStrategy {
    /// Must use the slow path (sort + compact).
    CompactOnly,
    /// May use the fast path (uniform offset) if headroom allows.
    Any,
}

/// Per-batch statistics collected in the first pass of reindexing.
#[derive(Debug, Clone)]
struct ColumnStats<T> {
    min: T,
    max: T,
    /// Upper bound on IDs consumed if this batch uses the offset path.
    /// Equal to `span` for `Any`, unique value count for `CompactOnly`.
    max_ids_needed: usize,
    strategy: ReindexStrategy,
}

/// Read-only first pass: compute per-batch statistics for an ID column without
/// mutating any batches.
///
/// Returns a `Vec` of length `store.len()`. Entry `i` is `None` when batch `i`
/// has no parent payload, no ID column, or an empty/all-null ID column (same
/// batches the current code skips via `continue`).
fn gather_column_stats<T, S, const N: usize>(
    store: &MultiBatchStore<'_, S, N>,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
    is_primary: bool,
) -> Result<Vec<Option<ColumnStats<T::Native>>>>
where
    T: ArrowNumericType,
    T::Native: Ord + Copy + ArrowNativeTypeOp,
    S: OtapBatchStore,
{
    let parent_idx = payload_to_idx(parent_payload_type);
    let mut stats = Vec::with_capacity(store.len());

    for i in 0..store.len() {
        let Some(parent_rb) = &store.batches[i][parent_idx] else {
            stats.push(None);
            continue;
        };

        let id_col = match extract_id_column(parent_rb, id_column_path) {
            Ok(col) => col,
            Err(_) => {
                stats.push(None);
                continue;
            }
        };

        let Some((min, max)) = id_column_min_max::<T>(id_col.as_ref())? else {
            stats.push(None);
            continue;
        };

        let id_values = materialize_id_values::<T>(id_col.as_ref())?;
        let len = id_values.len();
        let span = max.as_usize() - min.as_usize() + 1;

        let children_ok = child_payload_types.iter().all(|&ct| {
            children_in_parent_range::<T, S, N>(store, i, ct, min, max).unwrap_or(false)
        });

        // We're computing the max number of ids that we will use after
        // reindexing according to the given strategy.
        let (strategy, max_ids_needed) = match is_primary {
            // In the primary case, we can either take the existing
            // span or we can compact and reduce the number of ids to
            // the length (which is equal to span if the ids are
            // contiguous already). We can only take the existing
            // span if the children are within the parent range otherwise
            // it's possible that we create integrity violations.
            true if children_ok => (ReindexStrategy::Any, span),
            true if !children_ok => (ReindexStrategy::CompactOnly, len),

            // The non-primary case is similar, but we have to be more
            // pessimistic for the CompactOnly case. In this case we don't
            // know how many unique ids there will be after we compact as
            // there could be gaps. This could be <= span, so we'll take the
            // span as the upper bound.
            false if children_ok => (ReindexStrategy::Any, span),
            false if !children_ok => (ReindexStrategy::CompactOnly, span),
            _ => unreachable!(),
        };

        stats.push(Some(ColumnStats {
            min,
            max,
            max_ids_needed,
            strategy,
        }));
    }

    Ok(stats)
}

/// Chunks the sorted ID column into consecutive ranges and creates mappings
///
/// Given a sorted slice of IDs, this identifies consecutive ranges (no gaps)
/// and calculates the offset needed to make them sequential starting from `offset`.
///
/// Returns (mappings, max_new_id)
fn create_mappings<T>(
    sorted_ids: &[T::Native],
    offset: T::Native,
) -> Result<(Vec<IdMapping<T::Native>>, T::Native)>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + Sub<Output = T::Native> + From<u8>,
{
    let mut mappings = Vec::new();
    let mut current_offset = offset;
    let one = T::Native::from(1);

    for chunk in sorted_ids.chunk_by(|a, b| *b == *a + one || *b == *a) {
        let start_id = chunk[0];
        let end_id = chunk[chunk.len() - 1];

        let (offset, sign) = if start_id <= current_offset {
            (current_offset - start_id, Sign::Positive)
        } else {
            (start_id - current_offset, Sign::Negative)
        };

        mappings.push(IdMapping {
            start_id,
            end_id,
            offset,
            sign,
        });

        // Calculate the next offset based on where this range ends
        let new_end = match sign {
            Sign::Positive => end_id + offset,
            Sign::Negative => end_id - offset,
        };
        current_offset = new_end + one;
    }

    Ok((mappings, current_offset))
}

/// Applies mappings to a sorted ID buffer that were produced by processing the
/// corresponding id column of the parent record batch.
///
/// Returns ranges of indices where referential integrity violations were found.
/// Referential integrity violations are when and ID is found in the child record
/// batch that is not a part of the primary ID column from the parent that defines
/// them.
///
/// For example if the parent record batch has idx [0, 1, 2] and the child
/// record batch has parent_ids [0, 1, 3] then the parent_id 3 at position 2
/// in the child record batch has a violation.
#[must_use]
fn apply_mappings<T>(
    sorted_ids: &mut [T::Native],
    mappings: &[IdMapping<T::Native>],
) -> Option<Vec<Range<usize>>>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + AddAssign + SubAssign,
{
    let mut violations = Vec::new();
    let mut remaining_slice = &mut sorted_ids[..];
    let mut idx = 0;
    for mapping in mappings.iter() {
        if remaining_slice.is_empty() {
            break;
        }

        // If there are elements left that come before the current mapping then these
        // were never a part of the parent column
        let map_start_idx = remaining_slice
            .iter()
            .position(|id| *id >= mapping.start_id)
            .unwrap_or(remaining_slice.len());
        if map_start_idx != 0 {
            violations.push(idx..idx + map_start_idx);
        }
        idx += map_start_idx;

        remaining_slice = &mut remaining_slice[map_start_idx..];
        let end_idx = remaining_slice
            .iter()
            .position(|id| *id > mapping.end_id)
            .unwrap_or(remaining_slice.len());

        let slice_to_map = &mut remaining_slice[0..end_idx];
        idx += slice_to_map.len();

        // TODO: Anything we need to do here to make sure this is vectorized?
        match mapping.sign {
            Sign::Positive => slice_to_map.iter_mut().for_each(|id| *id += mapping.offset),
            Sign::Negative => slice_to_map.iter_mut().for_each(|id| *id -= mapping.offset),
        }
        remaining_slice = &mut remaining_slice[end_idx..];
    }

    // If there are elements left after processing all mappings, these were also
    // never a part of the parent column
    if !remaining_slice.is_empty() {
        violations.push(idx..idx + remaining_slice.len());
    }

    match violations.is_empty() {
        true => None,
        false => Some(violations),
    }
}

/// Takes values from src to dst using indices
///
/// Copies values from src[indices[i]] to dst[i] for all i.
/// All three slices must have the same length.
fn take_vec<T: Copy>(src: &[T], dst: &mut [T], indices: &[u32]) {
    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len(), indices.len());

    for i in 0..indices.len() {
        let src_idx = indices[i] as usize;
        dst[i] = src[src_idx];
    }
}

/// Untakes (unsorts) values from src to dst using sort indices
///
/// Copies values from src[i] to dst[indices[i]] for all i.
/// All three slices must have the same length.
fn untake_vec<T: Copy>(src: &[T], dst: &mut [T], indices: &[u32]) {
    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len(), indices.len());

    for i in 0..indices.len() {
        let dst_idx = indices[i] as usize;
        dst[dst_idx] = src[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use arrow::array::RecordBatch;

    use crate::error::Error;
    use crate::otap::transform::testing::{
        self, assert_no_id_overlaps, extract_relation_fingerprints, find_parent_id_size, logs,
        metrics, traces,
    };
    use crate::otap::transform::transport_optimize::apply_transport_optimized_encodings;
    use crate::otap::transform::util::{IdColumnType, payload_relations, payload_to_idx};
    use crate::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::otap_to_otlp;

    const HALF_U16: u16 = (u16::MAX / 2) + 1;

    // ---- Logs tests ----

    #[test]
    fn test_logs_mismatched_id_types() {
        // Note: We may not have to support u32 here for logs. If this starts
        // failing then the right thing to do might be to remove this test.
        let mut batches = vec![
            logs!((Logs, ("id", UInt16, vec![0, 1]))),
            logs!((Logs, ("id", UInt32, vec![0, 1]))),
        ];
        let result = reindex_logs(&mut batches);
        assert!(matches!(result, Err(Error::ColumnDataTypeMismatch { .. })));
    }

    #[test]
    fn test_logs_greater_than_u16_max() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();
        let ids3 = vec![u16::MAX];

        let mut batches = vec![
            logs!((Logs, ("id", UInt16, ids))),
            logs!((Logs, ("id", UInt16, ids2))),
            logs!((Logs, ("id", UInt16, ids3))),
        ];
        let result = reindex_logs(&mut batches);
        assert!(matches!(result, Err(Error::TooManyItems { .. })));
    }

    #[test]
    fn test_logs_u16_max_items() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();

        let mut batches = vec![
            logs!((Logs, ("id", UInt16, ids))),
            logs!((Logs, ("id", UInt16, ids2))),
        ];
        test_reindex_logs(&mut batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_referential_integrity_violations() {
        // Referential integrity violations occur when a child has parent_ids that
        // don't exist in the parent's id column. These orphaned rows are removed
        // during reindexing.
        //
        // Three different violation positions:
        // - At the start (id 0 not in parent [1, 2, 4])
        // - In the middle (id 3 not in parent [1, 2, 4])
        // - At the end (id 5 not in parent [1, 2, 4])
        //
        // Valid children: ids 1, 2, 4 -> 3 valid rows out of 7.
        let parent_ids = vec![1u16, 2, 4];
        let child_ids  = vec![1u16, 0, 2, 2, 3, 3, 4, 5, 4];

        let parent_set: HashSet<u16> = parent_ids.iter().copied().collect();
        let expected_child_count = child_ids.iter().filter(|id| parent_set.contains(id)).count();

        let mut batches = vec![
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ];

        reindex_logs(&mut batches).unwrap();
        assert_no_id_overlaps::<Logs, { Logs::COUNT }>(&batches);

        // Verify orphaned rows were removed
        let child_idx = payload_to_idx(ArrowPayloadType::LogAttrs);
        for group in &batches {
            let child_batch = group[child_idx].as_ref().unwrap();
            assert_eq!(child_batch.num_rows(), expected_child_count);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_complex() {
        // Overlapping ranges, not in order, many to many child relations
        let parent_ids   = vec![0, 5, 3, 10, 7, 11];
        let parent_ids_2 = vec![2, 8, 5, 9, 11, 0];
        let child_ids    = vec![0, 10, 10, 10, 7, 7, 3, 11, 3, 11, 3, 11];
        let child_ids_2  = vec![2, 8, 8, 5, 9, 9, 0, 11, 11];

        // Log Attrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_range_gaps() {
        // IDs with gaps between consecutive ranges
        let parent_ids   = vec![0, 1, 10, 11, 15, 16];
        let parent_ids_2 = vec![5, 6, 12, 13, 20, 21];
        let child_ids    = vec![0, 1, 10, 11, 15, 16];
        let child_ids_2  = vec![5, 6, 12, 13, 20, 21];

        // Log Attrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_reindex_overlapping() {
        // Both batches use the same IDs, so reindexing must remap to avoid overlap
        let parent_ids   = vec![1, 0];
        let child_ids   = vec![0, 0, 1, 1];

        // LogAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_reindex_noop() {
        // IDs are not overlapping at all
        let parent_ids   = vec![0, 2, 1, 3];
        let parent_ids_2 = vec![4, 6, 5, 7];

        let child_ids   = vec![1, 2, 2, 0, 3];
        let child_ids_2 = vec![6, 6, 5, 5, 7, 4];

        // LogAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("scope.id", UInt16, parent_ids_2.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_logs(&mut[
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids_2.clone()), ("resource.id", UInt16, parent_ids_2.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_all_offset() {
        // Checking scenario where we skip all compaction becuase 
        // all batches have gaps (span > len) but sum(span) fits
        // in U16 range. Every batch qualifies for the offset fast-path.
        //
        // Per batch: span=6, len=3, sum(span)=18, need_to_save < 0.

        // id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 2, 5])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 5, 2]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![10u16, 13, 15])),
                (LogAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![20u16, 22, 25])),
                (LogAttrs, ("parent_id", UInt16, vec![20u16, 22, 25, 25]))
            ),
        ]);

        // resource.id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("resource.id", UInt16, vec![0u16, 0, 2, 5, 5])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 5]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![10u16, 10, 13, 15])),
                (ResourceAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![20u16, 22, 25, 25])),
                (ResourceAttrs, ("parent_id", UInt16, vec![20u16, 22, 25]))
            ),
        ]);

        // scope.id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("scope.id", UInt16, vec![0u16, 0, 2, 5, 5])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 5]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![10u16, 10, 13, 15])),
                (ScopeAttrs, ("parent_id", UInt16, vec![10u16, 13, 15]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![20u16, 22, 25, 25])),
                (ScopeAttrs, ("parent_id", UInt16, vec![20u16, 22, 25]))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_some_compact() {
        // Checking that we can handle partial compaction where some 
        // batches need to compact, but not all.
        //
        // Batch 0: span=40001, len=3
        // Batch 1: span=30001, len=3
        // Batch 2: span=6,     len=3
        // total_span=70008, need_to_save=4473. Batch 0 compacts (saves 39998).

        // id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 10000, 40000])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 10000, 30000])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 3, 5])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);

        // resource.id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("resource.id", UInt16, vec![0u16, 0, 10000, 40000, 40000])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![0u16, 0, 10000, 30000])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("resource.id", UInt16, vec![0u16, 3, 5, 5])),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);

        // scope.id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3, 4]),
                       ("scope.id", UInt16, vec![0u16, 0, 10000, 40000, 40000])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 10000, 40000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![0u16, 0, 10000, 30000])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 10000, 30000]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3]),
                       ("scope.id", UInt16, vec![0u16, 3, 5, 5])),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 3, 5]))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_logs_greedy_all_compact() {
        // Checking that we can perfectly compact to the max u16
        // range when every batch needs compaction.
        //
        // Batch 0: ids = 0,2,4,...,65534 (32768 items, span=65535)
        // Batch 1: ids = 0,2,4,...,65532 (32767 items, span=65533)
        // total_span=131068, need_to_save=65533. Both compact.

        let even_ids_0: Vec<u16> = (0..32768).map(|i| i * 2).collect();
        let even_ids_1: Vec<u16> = (0..32767).map(|i| i * 2).collect();

        // id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, even_ids_0.clone())),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, even_ids_1.clone())),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);

        // resource.id
        let contiguous_ids_0: Vec<u16> = (0..32768).collect();
        let contiguous_ids_1: Vec<u16> = (0..32767).collect();

        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_0.clone()),
                       ("resource.id", UInt16, even_ids_0.clone())),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_1.clone()),
                       ("resource.id", UInt16, even_ids_1.clone())),
                (ResourceAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);

        // scope.id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_0.clone()),
                       ("scope.id", UInt16, even_ids_0.clone())),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
            logs!(
                (Logs, ("id", UInt16, contiguous_ids_1.clone()),
                       ("scope.id", UInt16, even_ids_1.clone())),
                (ScopeAttrs, ("parent_id", UInt16, vec![0u16, 2, 4]))
            ),
        ]);
    }

    #[test]
    fn test_logs_greedy_violation_no_overflow() {
        // Exactly 65535 total logs with referential integrity violations in
        // child tables. Each child table has more rows than its parent and
        // includes parent_ids outside the parent's [min, max] range.
        //
        // The children_in_parent_range check detects these violations via
        // min/max statistics and forces CompactOnly strategy. Without this,
        // the offset fast-path would overflow u16 when adding the offset to
        // out-of-range parent_ids (e.g. batch 1 offset=32768, child
        // parent_id=u16::MAX -> 32768 + 65535 overflows).

        let ids_0: Vec<u16> = (0..32768).collect();
        let ids_1: Vec<u16> = (0..32767).collect();

        // let mut child_pids_0: Vec<u16> = (0..32768).collect();
        let mut child_pids_0: Vec<u16> = (0..65535).collect();
        child_pids_0.push(u16::MAX);
        // let mut child_pids_1: Vec<u16> = (0..32767).collect();
        let mut child_pids_1: Vec<u16> = (0..65535).collect();
        child_pids_1.push(u16::MAX);

        // id
        test_reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, ids_0.clone())),
                (LogAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, ids_1.clone())),
                (LogAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);

        test_reindex_logs(&mut [
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_0.clone()),
                    ("resource.id", UInt16, ids_0.clone())
                ),
                (ResourceAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_1.clone()),
                    ("resource.id", UInt16, ids_1.clone())
                ),
                (ResourceAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);

        // scope.id
        test_reindex_logs(&mut [
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_0.clone()),
                    ("scope.id", UInt16, ids_0.clone())
                ),
                (ScopeAttrs, ("parent_id", UInt16, child_pids_0.clone()))
            ),
            logs!(
                (
                    Logs,
                    ("id", UInt16, ids_1.clone()),
                    ("scope.id", UInt16, ids_1.clone())
                ),
                (ScopeAttrs, ("parent_id", UInt16, child_pids_1.clone()))
            ),
        ]);
    }

    // ---- Traces tests ----

    #[test]
    #[rustfmt::skip]
    fn test_traces_reindex_attrs() {
        // Test resource, scope, and span attrs reindexing with overlapping IDs
        let parent_ids = vec![1u16, 0];
        let child_ids  = vec![0u16, 0, 1, 1];

        // SpanAttrs
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone())),
                (SpanAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone())),
                (SpanAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    fn test_traces_greater_than_u16_max() {
        let ids = (0..HALF_U16).collect::<Vec<_>>();
        let ids2 = (HALF_U16..u16::MAX).collect::<Vec<_>>();
        let ids3 = vec![u16::MAX];

        let mut batches = vec![
            traces!((Spans, ("id", UInt16, ids))),
            traces!((Spans, ("id", UInt16, ids2))),
            traces!((Spans, ("id", UInt16, ids3))),
        ];
        let result = reindex_traces(&mut batches);
        assert!(matches!(result, Err(Error::TooManyItems { .. })));
    }

    #[test]
    fn test_traces_empty_batches() {
        let mut batches: Vec<[Option<RecordBatch>; Traces::COUNT]> = vec![];
        reindex_traces(&mut batches).unwrap();

        let mut batches = vec![traces!((Spans, ("id", UInt16, vec![0u16, 1])))];
        test_reindex_traces(&mut batches);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_span_events_and_links() {
        // Test new two level relationships:
        //
        // Spans -> SpanEvents (parent_id UInt16, id UInt32)
        // Spans -> SpanLinks (parent_id UInt16, id UInt32)

        let span_ids        = vec![0u16, 1, 2];
        let event_pids      = vec![0u16, 0, 1, 2, 2];
        let link_pids       = vec![1u16, 2];
        let event_ids       = vec![0u32, 1, 2, 3, 4];
        let link_ids        = vec![0u32, 1];

        let span_ids_2      = vec![1u16, 3, 4];
        let event_pids_2    = vec![1u16, 3, 3, 4];
        let event_ids_2     = vec![0u32, 1, 2, 3];
        let link_pids_2     = vec![3u16, 4, 4];
        let link_ids_2      = vec![0u32, 1, 2];

        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids)),
                (SpanEvents, ("id", UInt32, event_ids), ("parent_id", UInt16, event_pids)),
                (SpanLinks, ("id", UInt32, link_ids), ("parent_id", UInt16, link_pids))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2)),
                (SpanEvents, ("id", UInt32, event_ids_2), ("parent_id", UInt16, event_pids_2)),
                (SpanLinks, ("id", UInt32, link_ids_2), ("parent_id", UInt16, link_pids_2))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_span_events_with_attrs() {
        // Three-level relationship: Spans -> SpanEvents -> SpanEventAttrs
        // SpanEvents.id is UInt32, SpanEventAttrs.parent_id is UInt32
        //
        // UInt32 id columns are always plain (not dictionary encoded).
        // UInt32 parent_id columns may be dictionary encoded.
        // We test encoding variants for parent_id columns:
        // 1. Plain UInt32 parent_ids
        // 2. Dict<UInt8, UInt32> parent_ids
        // 3. Dict<UInt16, UInt32> parent_ids
        // 4. Mixed encodings across parent_id columns and batches
        let span_ids     = vec![0u16, 1];
        let span_ids_2   = vec![2u16, 3];
        let event_pids   = vec![0u16, 0, 1];
        let event_pids_2 = vec![2u16, 3, 3];

        // Plain UInt32 parent_ids
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 1, 1, 2]))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 2, 2]))
            ),
        ]);

        // Dict<UInt8, UInt32> parent_ids
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Dict<UInt16, UInt32> parent_ids
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Mixed: Dict<UInt16, UInt32> event attr parent_ids in first batch,
        // Dict<UInt8, UInt32> event attr parent_ids in second batch
        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_complex() {
        // Complex case with spans, events, event attrs, links, and span attrs all at once
        let span_ids          = vec![0u16, 5, 3, 10];
        let span_ids_2        = vec![2u16, 7, 4, 12];
        let event_pids        = vec![0u16, 0, 3, 10, 10];
        let event_pids_2      = vec![2u16, 7, 7, 12];
        let event_ids         = vec![0u32, 1, 2, 3, 4];
        let event_ids_2       = vec![0u32, 1, 2, 3];
        let event_attr_pids   = vec![0u32, 1, 2, 3, 4, 4];
        let event_attr_pids_2 = vec![0u32, 1, 2, 3];
        let link_pids         = vec![5u16, 3];
        let link_pids_2       = vec![7u16, 4, 12];
        let link_ids          = vec![0u32, 1];
        let link_ids_2        = vec![0u32, 1, 2];
        let span_attr_pids    = vec![0u16, 5, 3, 10, 10];
        let span_attr_pids_2  = vec![2u16, 7, 12];

        test_reindex_traces(&mut[
            traces!(
                (Spans, ("id", UInt16, span_ids.clone())),
                (SpanEvents, ("id", UInt32, event_ids.clone()), ("parent_id", UInt16, event_pids.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, event_attr_pids.clone())),
                (SpanLinks, ("id", UInt32, link_ids.clone()), ("parent_id", UInt16, link_pids.clone())),
                (SpanAttrs, ("parent_id", UInt16, span_attr_pids.clone()))
            ),
            traces!(
                (Spans, ("id", UInt16, span_ids_2.clone())),
                (SpanEvents, ("id", UInt32, event_ids_2.clone()), ("parent_id", UInt16, event_pids_2.clone())),
                (SpanEventAttrs, ("parent_id", UInt32, event_attr_pids_2.clone())),
                (SpanLinks, ("id", UInt32, link_ids_2.clone()), ("parent_id", UInt16, link_pids_2.clone())),
                (SpanAttrs, ("parent_id", UInt16, span_attr_pids_2.clone()))
            ),
        ]);
    }

    // ---- Metrics tests ----

    #[test]
    #[rustfmt::skip]
    fn test_metrics_reindex_attrs() {
        // Test resource, scope, and metric attrs reindexing with overlapping IDs
        let parent_ids = vec![1u16, 0];
        let child_ids  = vec![0u16, 0, 1, 1];

        // MetricAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone())),
                (MetricAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone())),
                (MetricAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ScopeAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("scope.id", UInt16, parent_ids.clone())),
                (ScopeAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);

        // ResourceAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, parent_ids.clone()), ("resource.id", UInt16, parent_ids.clone())),
                (ResourceAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_data_points() {
        // Test Metrics.id -> DataPoints.parent_id for each data point type
        // with overlapping IDs across batches
        let metric_ids    = vec![0u16, 1, 2];
        let metric_ids_2  = vec![1u16, 3, 4];
        let dp_pids       = vec![0u16, 0, 1, 2, 2];
        let dp_pids_2     = vec![1u16, 3, 3, 4];
        let dp_ids        = vec![0u32, 1, 2, 3, 4];
        let dp_ids_2      = vec![0u32, 1, 2, 3];

        // NumberDataPoints
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // SummaryDataPoints
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_dp_with_attrs() {
        // Three-level relationship: Metrics -> DataPoints -> DpAttrs
        let metric_ids     = vec![0u16, 1];
        let metric_ids_2   = vec![2u16, 3];
        let dp_pids        = vec![0u16, 0, 1];
        let dp_pids_2      = vec![2u16, 3, 3];
        let dp_ids         = vec![0u32, 1, 2];
        let dp_ids_2       = vec![0u32, 1, 2];
        let dp_attr_pids   = vec![0u32, 1, 1, 2];
        let dp_attr_pids_2 = vec![0u32, 2, 2];

        // NumberDataPoints -> NumberDpAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // SummaryDataPoints -> SummaryDpAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (SummaryDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (SummaryDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (SummaryDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpAttrs, ("parent_id", UInt32, dp_attr_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_dp_with_exemplars() {
        // Three-level relationship: Metrics -> DataPoints -> Exemplars
        // (Summary does not have exemplars)
        let metric_ids      = vec![0u16, 1];
        let metric_ids_2    = vec![2u16, 3];
        let dp_pids         = vec![0u16, 0, 1];
        let dp_pids_2       = vec![2u16, 3, 3];
        let dp_ids          = vec![0u32, 1, 2];
        let dp_ids_2        = vec![0u32, 1, 2];
        let exemplar_pids   = vec![0u32, 1, 1, 2];
        let exemplar_pids_2 = vec![0u32, 2, 2];
        let exemplar_ids    = vec![0u32, 1, 2, 3];
        let exemplar_ids_2  = vec![0u32, 1, 2];

        // NumberDataPoints -> NumberDpExemplars
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpExemplars
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpExemplars
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_exemplar_attrs() {
        // Four-level relationship: Metrics -> DataPoints -> Exemplars → ExemplarAttrs
        let metric_ids          = vec![0u16, 1];
        let metric_ids_2        = vec![2u16, 3];
        let dp_pids             = vec![0u16, 0, 1];
        let dp_pids_2           = vec![2u16, 3, 3];
        let dp_ids              = vec![0u32, 1, 2];
        let dp_ids_2            = vec![0u32, 1, 2];
        let exemplar_pids       = vec![0u32, 1, 2];
        let exemplar_pids_2     = vec![0u32, 1, 2];
        let exemplar_ids        = vec![0u32, 1, 2];
        let exemplar_ids_2      = vec![0u32, 1, 2];
        let exemplar_attr_pids  = vec![0u32, 1, 1, 2];
        let exemplar_attr_pids_2 = vec![0u32, 2, 2];

        // NumberDataPoints -> NumberDpExemplars -> NumberDpExemplarAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);

        // HistogramDataPoints -> HistogramDpExemplars -> HistogramDpExemplarAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (HistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (HistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);

        // ExpHistogramDataPoints -> ExpHistogramDpExemplars -> ExpHistogramDpExemplarAttrs
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids.clone()), ("parent_id", UInt16, dp_pids.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids.clone()), ("parent_id", UInt32, exemplar_pids.clone())),
                (ExpHistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (ExpHistogramDataPoints, ("id", UInt32, dp_ids_2.clone()), ("parent_id", UInt16, dp_pids_2.clone())),
                (ExpHistogramDpExemplars, ("id", UInt32, exemplar_ids_2.clone()), ("parent_id", UInt32, exemplar_pids_2.clone())),
                (ExpHistogramDpExemplarAttrs, ("parent_id", UInt32, exemplar_attr_pids_2.clone()))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_number_dp_chain_with_dicts() {
        // Four-level chain: Metrics -> NumberDataPoints -> NumberDpExemplars -> NumberDpExemplarAttrs
        //
        // UInt32 id columns are always plain (not dictionary encoded).
        // UInt32 parent_id columns may be dictionary encoded.
        // Tests dictionary encoding variants for parent_id columns:
        // 1. Dict<UInt8, UInt32> parent_ids
        // 2. Dict<UInt16, UInt32> parent_ids
        // 3. Mixed encodings across parent_id columns and batches
        let metric_ids           = vec![0u16, 1];
        let metric_ids_2         = vec![2u16, 3];
        let dp_pids              = vec![0u16, 0, 1];
        let dp_pids_2            = vec![2u16, 3, 3];

        // Dict<UInt8, UInt32> for all UInt32 parent_id columns
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Dict<UInt16, UInt32> for all UInt32 parent_id columns
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1], vec![0u32, 2])))
            ),
        ]);

        // Mixed: Dict<UInt16, UInt32> exemplar parent_ids and exemplar attr parent_ids in first batch,
        // Dict<UInt8, UInt32> exemplar attr parent_ids in second batch
        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 2], vec![0u32, 1, 2]))),
                (NumberDpExemplarAttrs, ("parent_id", (UInt16, UInt32), (vec![0u16, 1, 1, 2], vec![0u32, 1, 2])))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, dp_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt32, vec![0u32, 1, 2])),
                (NumberDpExemplarAttrs, ("parent_id", (UInt8, UInt32), (vec![0u8, 1, 1], vec![0u32, 2])))
            ),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_complex() {
        // Complex case: multiple DP types + attrs + exemplars + exemplar attrs
        let metric_ids            = vec![0u16, 5, 3, 10];
        let metric_ids_2          = vec![2u16, 7, 4, 12];

        // NumberDataPoints chain
        let num_dp_pids           = vec![0u16, 0, 3];
        let num_dp_pids_2         = vec![2u16, 7, 7];
        let num_dp_ids            = vec![0u32, 1, 2];
        let num_dp_ids_2          = vec![0u32, 1, 2];
        let num_dp_attr_pids      = vec![0u32, 1, 2];
        let num_dp_attr_pids_2    = vec![0u32, 1, 2];
        let num_exemplar_pids     = vec![0u32, 1];
        let num_exemplar_pids_2   = vec![0u32, 2];
        let num_exemplar_ids      = vec![0u32, 1];
        let num_exemplar_ids_2    = vec![0u32, 1];
        let num_ex_attr_pids      = vec![0u32, 1];
        let num_ex_attr_pids_2    = vec![0u32, 1];

        // HistogramDataPoints chain
        let hist_dp_pids          = vec![5u16, 10, 10];
        let hist_dp_pids_2        = vec![4u16, 12, 12];
        let hist_dp_ids           = vec![0u32, 1, 2];
        let hist_dp_ids_2         = vec![0u32, 1, 2];
        let hist_exemplar_pids    = vec![0u32, 2];
        let hist_exemplar_pids_2  = vec![1u32, 2];
        let hist_exemplar_ids     = vec![0u32, 1];
        let hist_exemplar_ids_2   = vec![0u32, 1];

        // MetricAttrs
        let metric_attr_pids      = vec![0u16, 5, 3, 10];
        let metric_attr_pids_2    = vec![2u16, 7, 12];

        test_reindex_metrics(&mut[
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids.clone())),
                (NumberDataPoints, ("id", UInt32, num_dp_ids.clone()), ("parent_id", UInt16, num_dp_pids.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, num_dp_attr_pids.clone())),
                (NumberDpExemplars, ("id", UInt32, num_exemplar_ids.clone()), ("parent_id", UInt32, num_exemplar_pids.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, num_ex_attr_pids.clone())),
                (HistogramDataPoints, ("id", UInt32, hist_dp_ids.clone()), ("parent_id", UInt16, hist_dp_pids.clone())),
                (HistogramDpExemplars, ("id", UInt32, hist_exemplar_ids.clone()), ("parent_id", UInt32, hist_exemplar_pids.clone())),
                (MetricAttrs, ("parent_id", UInt16, metric_attr_pids.clone()))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, metric_ids_2.clone())),
                (NumberDataPoints, ("id", UInt32, num_dp_ids_2.clone()), ("parent_id", UInt16, num_dp_pids_2.clone())),
                (NumberDpAttrs, ("parent_id", UInt32, num_dp_attr_pids_2.clone())),
                (NumberDpExemplars, ("id", UInt32, num_exemplar_ids_2.clone()), ("parent_id", UInt32, num_exemplar_pids_2.clone())),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, num_ex_attr_pids_2.clone())),
                (HistogramDataPoints, ("id", UInt32, hist_dp_ids_2.clone()), ("parent_id", UInt16, hist_dp_pids_2.clone())),
                (HistogramDpExemplars, ("id", UInt32, hist_exemplar_ids_2.clone()), ("parent_id", UInt32, hist_exemplar_pids_2.clone())),
                (MetricAttrs, ("parent_id", UInt16, metric_attr_pids_2.clone()))
            ),
        ]);
    }

    // ---- Transport optimized encoding tests ----

    #[test]
    #[rustfmt::skip]
    fn test_logs_transport_optimized() {
        let mut batches = vec![
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3])),
                (LogAttrs, ("parent_id", UInt16, vec![1u16, 2, 2, 0, 3, 3, 2, 2]))
            ),
            logs!(
                (Logs, ("id", UInt16, vec![0u16, 1, 2, 3])),
                (LogAttrs, ("parent_id", UInt16, vec![0u16, 1, 3, 3, 2, 2, 1, 1, 2, 2, 0, 0]))
            ),
        ];
        test_reindex_transport_optimized::<Logs, { Logs::COUNT }>(
            &mut batches, reindex_logs, |b| OtapArrowRecords::Logs(Logs { batches: b.clone() }),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_traces_transport_optimized() {
        let mut batches = vec![
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1, 2])),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 0, 1])),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 1, 1, 2])),
                (SpanLinks, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt16, vec![1u16, 2])),
                (SpanAttrs, ("parent_id", UInt16, vec![0u16, 1, 2]))
            ),
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1, 2])),
                (SpanEvents, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![1u16, 2, 2])),
                (SpanEventAttrs, ("parent_id", UInt32, vec![0u32, 2, 2])),
                (SpanLinks, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt16, vec![0u16, 1])),
                (SpanAttrs, ("parent_id", UInt16, vec![0u16, 1, 2]))
            ),
        ];
        test_reindex_transport_optimized::<Traces, { Traces::COUNT }>(
            &mut batches, reindex_traces, |b| OtapArrowRecords::Traces(Traces { batches: b.clone() }),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_metrics_transport_optimized() {
        let mut batches = vec![
            metrics!(
                (UnivariateMetrics, ("id", UInt16, vec![0u16, 1])),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 0, 1])),
                (NumberDpAttrs, ("parent_id", UInt32, vec![0u32, 1, 2])),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt32, vec![0u32, 2])),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (MetricAttrs, ("parent_id", UInt16, vec![0u16, 0, 1]))
            ),
            metrics!(
                (UnivariateMetrics, ("id", UInt16, vec![0u16, 1])),
                (NumberDataPoints, ("id", UInt32, vec![0u32, 1, 2]), ("parent_id", UInt16, vec![0u16, 1, 1])),
                (NumberDpAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (NumberDpExemplars, ("id", UInt32, vec![0u32, 1]), ("parent_id", UInt32, vec![1u32, 2])),
                (NumberDpExemplarAttrs, ("parent_id", UInt32, vec![0u32, 1])),
                (MetricAttrs, ("parent_id", UInt16, vec![0u16, 1]))
            ),
        ];
        test_reindex_transport_optimized::<Metrics, { Metrics::COUNT }>(
            &mut batches, reindex_metrics, |b| OtapArrowRecords::Metrics(Metrics { batches: b.clone() }),
        );
    }

    // ---- Primary ID bounds tests ----

    #[test]
    fn test_logs_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Logs, { Logs::COUNT }>(reindex_logs);
    }

    #[test]
    fn test_traces_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Traces, { Traces::COUNT }>(reindex_traces);
    }

    #[test]
    fn test_metrics_u16_primary_id_bounds() {
        test_u16_primary_id_bounds::<Metrics, { Metrics::COUNT }>(reindex_metrics);
    }

    /// Tests the overflow bounds for every U16 primary id column in a batch store.
    /// Currently we're not testing u32 because it's a lot of memory to do that.
    ///
    /// For each U16 payload type, verifies that u16::MAX total rows succeeds
    /// and u16::MAX + 1 fails with TooManyItems.
    fn test_u16_primary_id_bounds<S: OtapBatchStore, const N: usize>(
        reindex_fn: fn(&mut [[Option<RecordBatch>; N]]) -> Result<()>,
    ) {
        for &payload_type in S::allowed_payload_types() {
            let info = payload_relations(payload_type);
            let Some(id_info) = info.primary_id else {
                continue;
            };
            if matches!(id_info.size, IdColumnType::U32) {
                continue;
            }

            // Exactly u16::MAX rows split across two batches should succeed
            let half = (u16::MAX / 2) as usize;
            let other_half = u16::MAX as usize - half;
            let idx = payload_to_idx(payload_type);

            let mut ok_batches: Vec<[Option<RecordBatch>; N]> =
                vec![std::array::from_fn(|_| None), std::array::from_fn(|_| None)];
            ok_batches[0][idx] = Some(make_u16_id_batch::<S>(payload_type, half));
            ok_batches[1][idx] = Some(make_u16_id_batch::<S>(payload_type, other_half));
            reindex_fn(&mut ok_batches).unwrap_or_else(|e| {
                panic!(
                    "{:?}: u16::MAX rows should succeed but got: {}",
                    payload_type, e
                )
            });

            // u16::MAX + 1 rows should fail
            let mut fail_batches: Vec<[Option<RecordBatch>; N]> =
                vec![std::array::from_fn(|_| None), std::array::from_fn(|_| None)];
            fail_batches[0][idx] = Some(make_u16_id_batch::<S>(payload_type, half));
            fail_batches[1][idx] = Some(make_u16_id_batch::<S>(payload_type, other_half + 1));
            assert!(
                matches!(
                    reindex_fn(&mut fail_batches),
                    Err(Error::TooManyItems { .. })
                ),
                "{:?}: u16::MAX + 1 rows should fail with TooManyItems",
                payload_type,
            );
        }
    }

    /// Creates a minimal batch for a U16 primary id payload type containing an
    /// "id" column with `count` rows. Non-root types also get a "parent_id"
    /// column whose type is determined by looking up the parent's primary id
    /// size via `find_parent_id_size`.
    fn make_u16_id_batch<S: OtapBatchStore>(
        payload_type: ArrowPayloadType,
        count: usize,
    ) -> RecordBatch {
        let ids: Vec<u16> = (0..count as u16).collect();
        let parent_id_size = find_parent_id_size::<S>(payload_type);
        let batch = match parent_id_size {
            None => record_batch!(("id", UInt16, ids)).unwrap(),
            Some(IdColumnType::U16) => {
                let pids = vec![0u16; count];
                record_batch!(("id", UInt16, ids), ("parent_id", UInt16, pids)).unwrap()
            }
            Some(IdColumnType::U32) => {
                let pids = vec![0u32; count];
                record_batch!(("id", UInt16, ids), ("parent_id", UInt32, pids)).unwrap()
            }
        };
        testing::complete_batch(payload_type, batch, &[payload_type])
    }

    #[test]
    fn test_logs_wrong_id_size() {
        let mut batches = vec![
            logs!((Logs, ("id", UInt32, vec![0u32, 1]))),
            logs!((Logs, ("id", UInt32, vec![0u32, 1]))),
        ];
        let result = reindex_logs(&mut batches);
        assert!(
            matches!(result, Err(Error::ColumnDataTypeMismatch { .. })),
            "expected ColumnDataTypeMismatch, got: {:?}",
            result,
        );
    }

    #[test]
    fn test_traces_span_events_wrong_id_size() {
        let mut batches = vec![
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1])),
                (
                    SpanEvents,
                    ("id", UInt16, vec![0u16, 1]),
                    ("parent_id", UInt16, vec![0u16, 1])
                )
            ),
            traces!(
                (Spans, ("id", UInt16, vec![0u16, 1])),
                (
                    SpanEvents,
                    ("id", UInt16, vec![0u16, 1]),
                    ("parent_id", UInt16, vec![0u16, 1])
                )
            ),
        ];
        let result = reindex_traces(&mut batches);
        assert!(
            matches!(result, Err(Error::ColumnDataTypeMismatch { .. })),
            "expected ColumnDataTypeMismatch, got: {:?}",
            result,
        );
    }

    // ---- Test helpers ----

    fn test_reindex_logs(batches: &mut [[Option<RecordBatch>; Logs::COUNT]]) {
        test_reindex::<Logs, { Logs::COUNT }>(batches, reindex_logs, |b| {
            OtapArrowRecords::Logs(Logs { batches: b.clone() })
        });
    }

    fn test_reindex_traces(batches: &mut [[Option<RecordBatch>; Traces::COUNT]]) {
        test_reindex::<Traces, { Traces::COUNT }>(batches, reindex_traces, |b| {
            OtapArrowRecords::Traces(Traces { batches: b.clone() })
        });
    }

    fn test_reindex_metrics(batches: &mut [[Option<RecordBatch>; Metrics::COUNT]]) {
        test_reindex::<Metrics, { Metrics::COUNT }>(batches, reindex_metrics, |b| {
            OtapArrowRecords::Metrics(Metrics { batches: b.clone() })
        });
    }

    /// Validates reindexing for any signal:
    /// 1. Converts input to OTLP (before reindex)
    /// 2. Snapshots parent -> child relation fingerprints (before reindex)
    /// 3. Reindexes the batches
    /// 4. Asserts no ID overlaps across batch groups
    /// 5. Asserts relation fingerprints are unchanged
    /// 6. Converts output to OTLP (after reindex)
    /// 7. Asserts the OTLP data is equivalent
    fn test_reindex<S, const N: usize>(
        batches: &mut [[Option<RecordBatch>; N]],
        reindex_fn: fn(&mut [[Option<RecordBatch>; N]]) -> Result<()>,
        to_otap: impl Fn(&[Option<RecordBatch>; N]) -> OtapArrowRecords,
    ) where
        S: OtapBatchStore,
    {
        let before_otlp: Vec<_> = batches.iter().map(|b| otap_to_otlp(&to_otap(b))).collect();
        let before_relations = extract_relation_fingerprints::<S, N>(batches);

        reindex_fn(batches).unwrap();
        assert_no_id_overlaps::<S, N>(batches);

        let after_relations = extract_relation_fingerprints::<S, N>(batches);
        assert_eq!(
            before_relations, after_relations,
            "Parent-child relations changed after reindexing"
        );

        let after_otlp: Vec<_> = batches.iter().map(|b| otap_to_otlp(&to_otap(b))).collect();

        // Pretty print batches
        // pretty_print_otap_batches(&batches);

        assert_equivalent(&before_otlp, &after_otlp);
    }

    /// Reindexes transport-optimized batches and verifies OTLP equivalence.
    ///
    /// Fingerprinting doesn't work on transport-optimized batches (IDs are
    /// delta-encoded), so we only verify OTLP equivalence before and after.
    fn test_reindex_transport_optimized<S, const N: usize>(
        batches: &mut [[Option<RecordBatch>; N]],
        reindex_fn: fn(&mut [[Option<RecordBatch>; N]]) -> Result<()>,
        to_otap: impl Fn(&[Option<RecordBatch>; N]) -> OtapArrowRecords,
    ) where
        S: OtapBatchStore,
    {
        apply_transport_encodings::<S, N>(batches);

        // Convert to OTLP *after* encoding but *before* reindex to get the
        // reference. `otap_to_otlp` internally strips transport encodings.
        let before_otlp: Vec<_> = batches.iter().map(|b| otap_to_otlp(&to_otap(b))).collect();

        reindex_fn(batches).unwrap();

        let after_otlp: Vec<_> = batches.iter().map(|b| otap_to_otlp(&to_otap(b))).collect();
        assert_equivalent(&before_otlp, &after_otlp);
    }

    #[test]
    #[rustfmt::skip]
    fn test_reindex_dict_parent_id_values_longer_than_keys() {
        // 2 rows, but the dictionary values array has 4 entries.
        // keys=[0,1], values=[0,1,2,3]
        let batch_a = traces!(
            (Spans,
                ("id", UInt16, vec![0u16, 1])),
            (SpanEvents,
                ("id", UInt32, vec![0u32, 1]),
                ("parent_id", UInt16, vec![0u16, 1])),
            (SpanEventAttrs,
                ("parent_id", (UInt8, UInt32), (vec![0u8, 1], vec![0u32, 1, 2, 3])))
        );

        reindex_traces(&mut [batch_a]).unwrap();
    }

    /// Regression test: non-primary ID columns (resource.id) with duplicates must
    /// use the slow path even when span == len.
    ///
    /// resource.id = [0, 1, 1, 4, 4] has span=5, len=5, which looks contiguous.
    /// But only 3 unique values exist (0, 1, 4). The fast path would advance the
    /// offset by 5 (the span) instead of 3 (the unique count), causing the second
    /// batch's resource.id values to collide with the first batch after remapping.
    /// The is_primary check forces the slow path for non-primary columns.
    #[test]
    #[rustfmt::skip]
    fn test_logs_many_to_many_resource_id_with_gaps_and_duplicates() {
        // resource.id has duplicates and gaps -- span == len but NOT contiguous.
        // 5 log rows, 3 unique resources.
        let log_ids_1     = vec![0u16, 1, 2, 3, 4];
        let resource_ids_1 = vec![0u16, 1, 1, 4, 4];

        // Second batch: 2 log rows, 2 unique resources.
        let log_ids_2      = vec![0u16, 1];
        let resource_ids_2 = vec![0u16, 1];

        test_reindex_logs(&mut [
            logs!(
                (Logs,
                    ("id", UInt16, log_ids_1),
                    ("resource.id", UInt16, resource_ids_1)),
                (ResourceAttrs,
                    ("parent_id", UInt16, vec![0u16, 1, 4]))
            ),
            logs!(
                (Logs,
                    ("id", UInt16, log_ids_2),
                    ("resource.id", UInt16, resource_ids_2)),
                (ResourceAttrs,
                    ("parent_id", UInt16, vec![0u16, 1]))
            ),
        ]);
    }

    /// Applies transport optimized encodings to all payload types in each batch group.
    fn apply_transport_encodings<S: OtapBatchStore, const N: usize>(
        batches: &mut [[Option<RecordBatch>; N]],
    ) {
        for group in batches.iter_mut() {
            for &payload_type in S::allowed_payload_types() {
                let idx = payload_to_idx(payload_type);
                if let Some(rb) = group[idx].take() {
                    let (encoded, _) =
                        apply_transport_optimized_encodings(&payload_type, &rb).unwrap();
                    group[idx] = Some(encoded);
                }
            }
        }
    }
}
