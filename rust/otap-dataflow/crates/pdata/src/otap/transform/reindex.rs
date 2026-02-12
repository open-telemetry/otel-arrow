// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::{Add, AddAssign, Range, RangeInclusive, Sub, SubAssign};
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, AsArray, DictionaryArray, PrimitiveArray, RecordBatch,
};
use arrow::buffer::ScalarBuffer;
use arrow::compute::kernels::take;
use arrow::datatypes::{ArrowNativeType, DataType, UInt8Type, UInt16Type, UInt32Type};

use crate::error::{Error, Result};
use crate::otap::transform::transport_optimize::{
    access_column, remove_transport_optimized_encodings, replace_column,
};
use crate::otap::{Logs, Metrics, OtapBatchStore, POSITION_LOOKUP, Traces, UNUSED_INDEX};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{ID, PARENT_ID};

use super::transport_optimize::{RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH};

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
    let primary_item_count: usize = store
        .select(S::ROOT_PAYLOAD_TYPE)
        .map(|rb| rb.num_rows())
        .sum();

    // TODO: Consider supporting u16::MAX + 1. This is a little tricky because we
    // do offset math with the Native type which causes us to overflow right
    // at the top. We could maybe try to do offset math with u64, but we will
    // have to constantly cast back and forth and it won't be as clear if we've
    // made a mistake somewhere. Only consequence is max batch size is 1 less.
    if primary_item_count > u16::MAX as usize {
        return Err(Error::TooManyItems {
            payload_type: S::ROOT_PAYLOAD_TYPE,
            count: primary_item_count,
            max: u16::MAX as usize,
            message: "Too many items to reindex".to_string(),
        });
    }

    for payload_type in S::allowed_payload_types() {
        store.remove_transport_optimized_encodings(*payload_type)?;
    }

    // Iterate over all allowed payload types for this signal
    for &payload_type in S::allowed_payload_types() {
        // Get all relations (parent-child relationships) for this payload type
        let info = payload_relations(payload_type);
        for relation in info.relations {
            reindex_id_column_dynamic(store, payload_type, relation.child_types, relation.key_col)?;
        }
    }

    Ok(())
}

/// Helper function that inspects the ID column type and dispatches to the appropriate generic function
fn reindex_id_column_dynamic<S, const N: usize>(
    store: &mut MultiBatchStore<'_, S, N>,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
) -> Result<()>
where
    S: OtapBatchStore,
{
    // Find the first batch with the parent payload to inspect the ID column type
    let parent_idx = payload_to_idx(parent_payload_type);

    for i in 0..store.len() {
        if let Some(parent_batch) = &store.batches[i][parent_idx] {
            if let Ok(id_col) = extract_id_column(parent_batch, id_column_path) {
                // Inspect the column type and dispatch to the appropriate generic function
                let data_type = id_col.data_type();
                match data_type {
                    DataType::UInt16 => {
                        return reindex_id_column::<UInt16Type, S, N>(
                            store,
                            parent_payload_type,
                            child_payload_types,
                            id_column_path,
                        );
                    }
                    DataType::UInt32 => {
                        return reindex_id_column::<UInt32Type, S, N>(
                            store,
                            parent_payload_type,
                            child_payload_types,
                            id_column_path,
                        );
                    }
                    DataType::Dictionary(_, value_type) => match value_type.as_ref() {
                        DataType::UInt16 => {
                            return reindex_id_column::<UInt16Type, S, N>(
                                store,
                                parent_payload_type,
                                child_payload_types,
                                id_column_path,
                            );
                        }
                        DataType::UInt32 => {
                            return reindex_id_column::<UInt32Type, S, N>(
                                store,
                                parent_payload_type,
                                child_payload_types,
                                id_column_path,
                            );
                        }
                        _ => {
                            return Err(Error::UnsupportedDictionaryValueType {
                                expect_oneof: vec![DataType::UInt16, DataType::UInt32],
                                actual: value_type.as_ref().clone(),
                            });
                        }
                    },
                    _ => {
                        return Err(Error::ColumnDataTypeMismatch {
                            name: id_column_path.to_string(),
                            expect: DataType::UInt16, // or UInt32
                            actual: data_type.clone(),
                        });
                    }
                }
            }
        }
    }

    // No batches found with this ID column - that's okay, nothing to reindex
    Ok(())
}

/// Generic function to reindex an ID column and its corresponding parent_id columns in child tables
///
/// # Type Parameters
/// * `T` - The Arrow primitive type for the ID (e.g., UInt16Type or UInt32Type)
/// * `S` - The OtapBatchStore type (e.g., Logs, Metrics, Traces)
/// * `N` - The number of batches (const generic)
///
/// # Arguments
/// * `store` - The batch store containing the record batches
/// * `parent_payload_type` - The payload type of the parent table (e.g., Logs)
/// * `child_payload_types` - The payload types of the child tables (e.g., [LogAttrs, SpanEvents])
/// * `id_column_path` - The path to the ID column in the parent table (e.g., "id", "resource.id")
fn reindex_id_column<T, S, const N: usize>(
    store: &mut MultiBatchStore<'_, S, N>,
    parent_payload_type: ArrowPayloadType,
    child_payload_types: &[ArrowPayloadType],
    id_column_path: &str,
) -> Result<()>
where
    T: ArrowPrimitiveType,
    T::Native: Ord
        + Copy
        + Add<Output = T::Native>
        + Sub<Output = T::Native>
        + AddAssign
        + SubAssign
        + From<u8>
        + ArrowNativeType,
    S: OtapBatchStore,
{
    let parent_idx = payload_to_idx(parent_payload_type);
    let mut offset = T::Native::from(0);

    for i in 0..store.len() {
        let Some(parent_rb) = store.get_mut(i)[parent_idx].take() else {
            continue;
        };

        // TODO: Consider unwrapping if we feel like id being present is an invariant.
        // that needs to be upheld at this point.
        // Extract ID column - if it doesn't exist, skip reindexing for this batch
        let id_col = match extract_id_column(&parent_rb, id_column_path) {
            Ok(col) => col,
            Err(_) => {
                // No ID column, put the batch back and continue
                store.get_mut(i)[parent_idx] = Some(parent_rb);
                continue;
            }
        };

        // TODO: We can optimize here by reusing some storage:
        //
        //  - The vectors to store the sort indices and to hold the sorted Ids is
        //  while we create mappings is just scratch space and reusasble. We may
        //  need one scratch buffer per Native type
        //  - The vector to hold the mappings is also reusable and just scratch
        //  space
        //
        // Create mappings for the parent IDs
        let mut ids = materialize_id_column::<T>(id_col.as_ref())?
            .values()
            .to_vec();
        let sort_indices = sort_vec_to_indices(&ids);
        let mut sorted_ids = vec![T::Native::default(); ids.len()];
        take_vec(&ids, &mut sorted_ids, &sort_indices);

        let (mappings, new_offset) = create_mappings::<T>(&sorted_ids, offset)?;
        offset = new_offset;

        // safety: Mappings should always be valid when applied to the ids that
        // generated them. If not then we've made a serious error.
        assert!(apply_mappings::<T>(&mut sorted_ids, &mappings).is_none());

        untake_vec(&sorted_ids, &mut ids, &sort_indices);
        let parent_rb = replace_id_column::<T>(parent_rb, id_column_path, ids)?;

        // Put parent batch back
        store.get_mut(i)[parent_idx] = Some(parent_rb);

        // Apply mappings to each child record batch one at a time
        for &child_payload_type in child_payload_types {
            let child_idx = payload_to_idx(child_payload_type);

            if let Some(child_rb) = store.get_mut(i)[child_idx].take() {
                let child_rb = reindex_child_column::<T>(child_rb, PARENT_ID, &mappings)?;

                // Put child batch back
                store.get_mut(i)[child_idx] = Some(child_rb);
            }
        }
    }

    Ok(())
}

/// Reindexes a child id column in a record batch using the provided mappings.
/// Parent ids take a slightly different path because they need to separate the
/// creation of the mappings from applying those mappings to potentially multiple
/// child batches.
fn reindex_child_column<T>(
    mut rb: RecordBatch,
    column_path: &str,
    mappings: &[IdMapping<T::Native>],
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + AddAssign + SubAssign + ArrowNativeType,
{
    // Extract ID column
    let id_col = extract_id_column(&rb, column_path)?;
    let ids = materialize_id_column::<T>(id_col.as_ref())?;
    let mut ids = ids.values().to_vec();

    // Sort the ids. If we already did this in the parent batch, we can skip it here
    // and reuse the same allocation.
    let sort_indices = sort_vec_to_indices(&ids);
    let sort_indices = PrimitiveArray::from(sort_indices);
    let mut new_ids = vec![T::Native::default(); ids.len()];
    take_vec(&ids, &mut new_ids, sort_indices.values());
    if let Some(violations) = apply_mappings::<T>(&mut new_ids, mappings) {
        // We have integrity violations in some number of ranges. We need to eliminate
        // them because we're on the reindexing path where we're squashing all ids
        // to contiguous ranges starting at 0, so any strays left behind may accidentally
        // be associated to ids in other batches.
        //
        // Process is as follows:
        // 1. Sort the entire record batch by the indices so that the ranges correspond to
        // the violation ranges.
        // 2. Remove all rows in those ranges and reconstruct the record batch.
        rb = sort_record_batch_by_indices(rb, &sort_indices)?;
    }

    // Unsort the IDs. Note that since `take` and `untake` can't be done
    // in place, we re-use the original id vec as the destination.
    untake_vec(&new_ids, &mut ids, sort_indices.values());
    let new_rb = replace_id_column::<T>(rb, column_path, ids)?;
    Ok(new_rb)
}

fn sort_record_batch_by_indices(rb: RecordBatch, indices: &dyn Array) -> Result<RecordBatch> {
    let (schema, columns, _) = rb.into_parts();
    let new_columns: Vec<_> = columns
        .iter()
        .map(|c| arrow::compute::take(c, indices, None))
        .collect::<arrow::error::Result<Vec<_>>>()
        .map_err(|e| Error::Batching { source: e })?;

    // safety: We did a valid tranformation on all columns
    Ok(RecordBatch::try_new(schema, new_columns).expect("valid record batch"))
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

/// Extracts an ID column from a record batch
fn extract_id_column(rb: &RecordBatch, column_path: &str) -> Result<ArrayRef> {
    access_column(column_path, &rb.schema(), rb.columns()).ok_or_else(|| Error::ColumnNotFound {
        name: column_path.to_string(),
    })
}

/// Sorts a vector of values and returns the resulting sort indices
fn sort_vec_to_indices<T: Ord>(values: &[T]) -> Vec<u32> {
    let mut indices: Vec<u32> = (0u32..values.len() as u32).collect();
    indices.sort_by_key(|&i| &values[i as usize]);
    indices
}

/// Materializes an ID array from either a direct array or dictionary values.
///
/// For dictionary arrays, returns the VALUES array (unique dictionary entries), not the per-row
/// logical values. This is intentional: callers remap just the dictionary values, and the
/// dictionary keys preserve the per-row structure automatically.
fn materialize_id_column<T>(array: &dyn Array) -> Result<&PrimitiveArray<T>>
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
                name: "id".to_string(),
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
    dbg!(&mappings);
    dbg!(&sorted_ids);
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
        idx += map_start_idx;
        if map_start_idx != 0 {
            violations.push(idx..idx + map_start_idx);
        }

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

fn payload_to_idx(payload_type: ArrowPayloadType) -> usize {
    let pos = POSITION_LOOKUP[payload_type as usize];
    assert_ne!(pos, UNUSED_INDEX);
    pos
}

fn idx_to_payload_type<T: OtapBatchStore>(idx: usize) -> ArrowPayloadType {
    T::allowed_payload_types()[idx]
}

struct PayloadRelationInfo {
    primary_id: Option<PrimaryIdInfo>,
    relations: &'static [Relation],
}

enum IdColumnType {
    U16,
    U32,
}

struct PrimaryIdInfo {
    name: &'static str,
    size: IdColumnType,
}

struct Relation {
    key_col: &'static str,
    child_types: &'static [ArrowPayloadType],
}

/// Get the primary ID column info and foreign relations for the given payload type.
fn payload_relations(parent_type: ArrowPayloadType) -> PayloadRelationInfo {
    match parent_type {
        // Logs
        ArrowPayloadType::Logs => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U16,
            }),
            relations: &[
                Relation {
                    key_col: RESOURCE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ResourceAttrs],
                },
                Relation {
                    key_col: SCOPE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ScopeAttrs],
                },
                Relation {
                    key_col: ID,
                    child_types: &[ArrowPayloadType::LogAttrs],
                },
            ],
        },
        // Traces
        ArrowPayloadType::Spans => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U16,
            }),
            relations: &[
                Relation {
                    key_col: RESOURCE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ResourceAttrs],
                },
                Relation {
                    key_col: SCOPE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ScopeAttrs],
                },
                Relation {
                    key_col: ID,
                    child_types: &[
                        ArrowPayloadType::SpanAttrs,
                        ArrowPayloadType::SpanEvents,
                        ArrowPayloadType::SpanLinks,
                    ],
                },
            ],
        },
        ArrowPayloadType::SpanEvents => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::SpanEventAttrs],
            }],
        },
        ArrowPayloadType::SpanLinkAttrs => PayloadRelationInfo {
            primary_id: None,
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::SpanEventAttrs],
            }],
        },

        // Metrics
        ArrowPayloadType::UnivariateMetrics | ArrowPayloadType::MultivariateMetrics => {
            PayloadRelationInfo {
                primary_id: Some(PrimaryIdInfo {
                    name: ID,
                    size: IdColumnType::U16,
                }),
                relations: &[
                    Relation {
                        key_col: RESOURCE_ID_COL_PATH,
                        child_types: &[ArrowPayloadType::ResourceAttrs],
                    },
                    Relation {
                        key_col: SCOPE_ID_COL_PATH,
                        child_types: &[ArrowPayloadType::ScopeAttrs],
                    },
                    Relation {
                        key_col: ID,
                        child_types: &[
                            ArrowPayloadType::MetricAttrs,
                            ArrowPayloadType::NumberDataPoints,
                            ArrowPayloadType::SummaryDataPoints,
                            ArrowPayloadType::HistogramDataPoints,
                            ArrowPayloadType::ExpHistogramDataPoints,
                        ],
                    },
                ],
            }
        }

        ArrowPayloadType::NumberDataPoints => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[
                    ArrowPayloadType::NumberDpAttrs,
                    ArrowPayloadType::NumberDpExemplars,
                ],
            }],
        },
        ArrowPayloadType::NumberDpExemplars => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::NumberDpExemplarAttrs],
            }],
        },
        ArrowPayloadType::SummaryDataPoints => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::SummaryDpAttrs],
            }],
        },
        ArrowPayloadType::HistogramDataPoints => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[
                    ArrowPayloadType::HistogramDpAttrs,
                    ArrowPayloadType::HistogramDpExemplars,
                ],
            }],
        },
        ArrowPayloadType::HistogramDpExemplars => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::HistogramDpExemplarAttrs],
            }],
        },
        ArrowPayloadType::ExpHistogramDataPoints => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[
                    ArrowPayloadType::ExpHistogramDpAttrs,
                    ArrowPayloadType::ExpHistogramDpExemplars,
                ],
            }],
        },
        ArrowPayloadType::ExpHistogramDpExemplars => PayloadRelationInfo {
            primary_id: Some(PrimaryIdInfo {
                name: ID,
                size: IdColumnType::U32,
            }),
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::ExpHistogramDpExemplarAttrs],
            }],
        },

        _ => PayloadRelationInfo {
            primary_id: None,
            relations: &[],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    use arrow::array::{
        Array, ArrayRef, AsArray, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    };
    use arrow::datatypes::{
        ArrowDictionaryKeyType, DataType, Field, Schema, UInt16Type, UInt32Type,
    };

    use crate::error::Error;
    use crate::otap::transform::transport_optimize::{
        access_column, apply_transport_optimized_encodings, replace_column, struct_column_name,
        update_field_encoding_metadata,
    };
    use crate::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::otap_to_otlp;

    /// Known ID column paths that need plain encoding metadata
    const ID_COLUMN_PATHS: &[&str] = &["id", "resource.id", "scope.id", "parent_id"];
    const HALF_U16: u16 = (u16::MAX / 2) + 1;

    macro_rules! logs {
        ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
            {
                use $crate::otap::Logs;
                use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                make_test_batch::<Logs, { Logs::COUNT }>(vec![
                    $((
                        ArrowPayloadType::$payload,
                        $crate::record_batch!($($record_batch_args)*).unwrap(),
                    ),)*
                ])
            }
        };
    }

    macro_rules! traces {
        ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
            {
                use $crate::otap::Traces;
                use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                make_test_batch::<Traces, { Traces::COUNT }>(vec![
                    $((
                        ArrowPayloadType::$payload,
                        $crate::record_batch!($($record_batch_args)*).unwrap(),
                    ),)*
                ])
            }
        };
    }

    macro_rules! metrics {
        ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
            {
                use $crate::otap::Metrics;
                use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                make_test_batch::<Metrics, { Metrics::COUNT }>(vec![
                    $((
                        ArrowPayloadType::$payload,
                        $crate::record_batch!($($record_batch_args)*).unwrap(),
                    ),)*
                ])
            }
        };
    }

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
        // Referential integrity violations can cause problems because we may end up
        // encountering Ids that are not in any mapped range. We need to make
        // sure that in such cases all valid ids are remapped and what happens to
        // the others is undefined for now.
        //
        // FIXME [JD]: Is this the right behavior or should we error? If we touch the
        // invalid ids by mapping them accidentally, we might add attributes to
        // some logs that didn't previously exist. More of a problem in a multi-tenant
        // scenario. This might be very expensive to detect for missing ids in the
        // middle and would require us to go through the motions for a full
        // join.
        //
        // Three different violations here:
        //
        // - One that is in the middle, meaning
        // it will probably just get remapped since it's a part of one of the
        // valid ranges
        // - One that is at the start, so it can't get remapped at all
        // - One that is at the end, so it also can't get remapped
        //
        // Since orphan child IDs have undefined remapping behavior, we only
        // verify that reindexing does not error. We do not assert relation
        // fingerprint preservation here.
        let parent_ids = vec![1, 2, 4];
        let child_ids  = vec![1, 0, 2, 3, 3, 5, 4];
        reindex_logs(&mut [
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
            logs!(
                (Logs, ("id", UInt16, parent_ids.clone())),
                (LogAttrs, ("parent_id", UInt16, child_ids.clone()))
            ),
        ]).unwrap();
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
        // Useful for debugging, keep this in and uncomment when running a single
        // test along with `-- --no-capture`
        // FIXME: Comment out
        for (idx, b) in batches.iter().enumerate() {
            use arrow::util::pretty;
            eprintln!("-----Batch #{}------", idx);
            for rb in b.iter().flatten().cloned() {
                eprintln!("{}", pretty::pretty_format_batches(&[rb]).unwrap());
            }
            eprintln!("-----End Batch #{}------", idx);
        }

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

    /// For each batch group, relation, and child type, records which child row
    /// indices map to each parent by ordinal position (smallest parent ID = 0,
    /// second smallest = 1, etc). This captures structural relationships
    /// independent of actual ID values, so it should be identical before and
    /// after reindexing.
    fn extract_relation_fingerprints<S: OtapBatchStore, const N: usize>(
        batches: &[[Option<RecordBatch>; N]],
    ) -> Vec<Vec<Vec<usize>>> {
        let mut fingerprints = Vec::new();

        for group in batches.iter() {
            for &payload_type in S::allowed_payload_types() {
                let parent_idx = payload_to_idx(payload_type);
                let Some(parent_batch) = &group[parent_idx] else {
                    continue;
                };

                for relation in payload_relations(payload_type).relations {
                    let Some(parent_col) = access_column(
                        relation.key_col,
                        &parent_batch.schema(),
                        parent_batch.columns(),
                    ) else {
                        continue;
                    };
                    let parent_ids = collect_row_ids(parent_col.as_ref());

                    // Sort and deduplicate to get ordinal mapping
                    let mut unique_sorted: Vec<u64> = parent_ids.clone();
                    unique_sorted.sort();
                    unique_sorted.dedup();

                    for &child_type in relation.child_types {
                        let child_idx = payload_to_idx(child_type);
                        let Some(child_batch) = &group[child_idx] else {
                            continue;
                        };

                        let Some(child_col) = access_column(
                            "parent_id",
                            &child_batch.schema(),
                            child_batch.columns(),
                        ) else {
                            continue;
                        };
                        let child_parent_ids = collect_row_ids(child_col.as_ref());

                        // For each parent ordinal, record which child row indices reference it
                        let mut ordinal_to_children: Vec<Vec<usize>> =
                            vec![vec![]; unique_sorted.len()];
                        for (child_row, &child_pid) in child_parent_ids.iter().enumerate() {
                            if let Ok(ordinal) = unique_sorted.binary_search(&child_pid) {
                                ordinal_to_children[ordinal].push(child_row);
                            }
                        }

                        fingerprints.push(ordinal_to_children);
                    }
                }
            }
        }

        fingerprints
    }

    /// Validates that no ID column has overlapping values across batch groups.
    /// Uses payload_relations to discover all ID columns that should be unique.
    fn assert_no_id_overlaps<S: OtapBatchStore, const N: usize>(
        batches: &[[Option<RecordBatch>; N]],
    ) {
        for &payload_type in S::allowed_payload_types() {
            let idx = payload_to_idx(payload_type);

            for relation in payload_relations(payload_type).relations {
                let mut seen = HashSet::new();

                for group in batches.iter() {
                    let Some(batch) = &group[idx] else {
                        continue;
                    };

                    let Some(col) =
                        access_column(relation.key_col, &batch.schema(), batch.columns())
                    else {
                        continue;
                    };

                    let ids = collect_ids(col.as_ref());
                    for id in ids {
                        assert!(
                            seen.insert(id),
                            "Overlapping ID in column '{}'",
                            relation.key_col,
                        );
                    }
                }
            }
        }
    }

    /// Collects unique ID values from a column (for dictionary arrays, returns the
    /// dictionary VALUES, not per-row values). Used by `assert_no_id_overlaps`.
    fn collect_ids(col: &dyn Array) -> Vec<u64> {
        match col.data_type() {
            DataType::Dictionary(ktype, _) => match ktype.as_ref() {
                DataType::UInt8 => {
                    let values = col.as_dictionary::<UInt8Type>().values();
                    collect_ids(values)
                }
                DataType::UInt16 => {
                    let values = col.as_dictionary::<UInt16Type>().values();
                    collect_ids(values)
                }
                _ => unreachable!(),
            },
            DataType::UInt16 => col
                .as_primitive::<UInt16Type>()
                .values()
                .iter()
                .map(|&v| v as u64)
                .collect(),
            DataType::UInt32 => col
                .as_primitive::<UInt32Type>()
                .values()
                .iter()
                .map(|&v| v as u64)
                .collect(),
            _ => unreachable!(),
        }
    }

    /// Collects per-row logical ID values, resolving dictionary encoding.
    fn collect_row_ids(col: &dyn Array) -> Vec<u64> {
        match col.data_type() {
            DataType::UInt16 => col
                .as_primitive::<UInt16Type>()
                .values()
                .iter()
                .map(|&v| v as u64)
                .collect(),
            DataType::UInt32 => col
                .as_primitive::<UInt32Type>()
                .values()
                .iter()
                .map(|&v| v as u64)
                .collect(),
            DataType::Dictionary(key_type, _) => match key_type.as_ref() {
                DataType::UInt8 => collect_dict_row_ids::<UInt8Type>(col),
                DataType::UInt16 => collect_dict_row_ids::<UInt16Type>(col),
                _ => unreachable!("Unsupported dictionary key type"),
            },
            _ => unreachable!("Unsupported column type: {:?}", col.data_type()),
        }
    }

    fn collect_dict_row_ids<K>(col: &dyn Array) -> Vec<u64>
    where
        K: ArrowDictionaryKeyType,
        K::Native: ArrowNativeType,
    {
        let dict = col.as_dictionary::<K>();
        let values = dict.values();
        match values.data_type() {
            DataType::UInt16 => {
                let vals = values.as_primitive::<UInt16Type>();
                dict.keys()
                    .values()
                    .iter()
                    .map(|k: &K::Native| vals.value(k.as_usize()) as u64)
                    .collect()
            }
            DataType::UInt32 => {
                let vals = values.as_primitive::<UInt32Type>();
                dict.keys()
                    .values()
                    .iter()
                    .map(|k: &K::Native| vals.value(k.as_usize()) as u64)
                    .collect()
            }
            _ => unreachable!("Unsupported dictionary value type"),
        }
    }

    fn make_test_batch<S: OtapBatchStore, const N: usize>(
        inputs: Vec<(ArrowPayloadType, RecordBatch)>,
    ) -> [Option<RecordBatch>; N] {
        let allowed = S::allowed_payload_types();
        let mut result: [Option<RecordBatch>; N] = std::array::from_fn(|_| None);
        let all_payload_types: Vec<ArrowPayloadType> = inputs.iter().map(|(pt, _)| *pt).collect();

        for (payload_type, batch) in inputs {
            assert!(
                allowed.contains(&payload_type),
                "Payload type {:?} is not allowed for this store",
                payload_type
            );

            let idx = payload_to_idx(payload_type);
            assert!(
                result[idx].is_none(),
                "Duplicate payload type {:?}",
                payload_type
            );

            result[idx] = Some(complete_batch(payload_type, batch, &all_payload_types));
        }

        result
    }

    fn complete_batch(
        payload_type: ArrowPayloadType,
        batch: RecordBatch,
        all_payload_types: &[ArrowPayloadType],
    ) -> RecordBatch {
        let batch = match payload_type {
            ArrowPayloadType::Logs => complete_logs_batch(batch),
            ArrowPayloadType::Spans => complete_spans_batch(batch),
            ArrowPayloadType::SpanEvents => complete_span_events_batch(batch),
            ArrowPayloadType::SpanLinks => complete_span_links_batch(batch),

            // Root metrics table
            ArrowPayloadType::UnivariateMetrics => {
                complete_metrics_batch(batch, infer_metric_type(all_payload_types))
            }

            // Attrs (adds key/type/int columns if missing)
            ArrowPayloadType::LogAttrs
            | ArrowPayloadType::SpanAttrs
            | ArrowPayloadType::MetricAttrs
            | ArrowPayloadType::ResourceAttrs
            | ArrowPayloadType::ScopeAttrs
            | ArrowPayloadType::SpanEventAttrs
            | ArrowPayloadType::SpanLinkAttrs
            | ArrowPayloadType::NumberDpAttrs
            | ArrowPayloadType::SummaryDpAttrs
            | ArrowPayloadType::HistogramDpAttrs
            | ArrowPayloadType::ExpHistogramDpAttrs
            | ArrowPayloadType::NumberDpExemplarAttrs
            | ArrowPayloadType::HistogramDpExemplarAttrs
            | ArrowPayloadType::ExpHistogramDpExemplarAttrs => complete_attrs_batch(batch),

            // Data points and exemplars: only id/parent_id needed
            ArrowPayloadType::NumberDataPoints
            | ArrowPayloadType::SummaryDataPoints
            | ArrowPayloadType::HistogramDataPoints
            | ArrowPayloadType::ExpHistogramDataPoints
            | ArrowPayloadType::NumberDpExemplars
            | ArrowPayloadType::HistogramDpExemplars
            | ArrowPayloadType::ExpHistogramDpExemplars => batch,

            _ => batch,
        };
        mark_id_columns_plain(batch)
    }

    fn infer_metric_type(payload_types: &[ArrowPayloadType]) -> u8 {
        for pt in payload_types {
            match pt {
                ArrowPayloadType::HistogramDataPoints
                | ArrowPayloadType::HistogramDpAttrs
                | ArrowPayloadType::HistogramDpExemplars
                | ArrowPayloadType::HistogramDpExemplarAttrs => return 3,
                ArrowPayloadType::ExpHistogramDataPoints
                | ArrowPayloadType::ExpHistogramDpAttrs
                | ArrowPayloadType::ExpHistogramDpExemplars
                | ArrowPayloadType::ExpHistogramDpExemplarAttrs => return 4,
                ArrowPayloadType::SummaryDataPoints | ArrowPayloadType::SummaryDpAttrs => return 5,
                ArrowPayloadType::NumberDataPoints
                | ArrowPayloadType::NumberDpAttrs
                | ArrowPayloadType::NumberDpExemplars
                | ArrowPayloadType::NumberDpExemplarAttrs => return 1,
                _ => {}
            }
        }
        1 // Default: Gauge
    }

    fn complete_metrics_batch(batch: RecordBatch, metric_type: u8) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
        wrap_struct_id_columns(&schema, &mut fields, &mut columns);

        if schema.fields.find("metric_type").is_none() {
            fields.push(Arc::new(Field::new("metric_type", DataType::UInt8, false)));
            columns.push(Arc::new(UInt8Array::from(vec![metric_type; num_rows])));
        }

        if schema.fields.find("name").is_none() {
            fields.push(Arc::new(Field::new("name", DataType::Utf8, false)));
            columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed metrics batch")
    }

    fn complete_logs_batch(batch: RecordBatch) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
        wrap_struct_id_columns(&schema, &mut fields, &mut columns);

        if schema.fields.find("body").is_none() {
            let body_fields = vec![
                Field::new("type", DataType::UInt8, false),
                Field::new("int", DataType::Int64, true),
            ];
            let body_arrays: Vec<ArrayRef> = vec![
                Arc::new(UInt8Array::from(vec![0u8; num_rows])),
                Arc::new(Int64Array::from(vec![0i64; num_rows])),
            ];
            let body = StructArray::try_new(body_fields.clone().into(), body_arrays, None)
                .expect("Failed to create body struct");

            fields.push(Arc::new(Field::new(
                "body",
                DataType::Struct(body_fields.into()),
                true,
            )));
            columns.push(Arc::new(body));
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed logs batch")
    }

    fn complete_attrs_batch(batch: RecordBatch) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

        if schema.fields.find("key").is_none() {
            fields.push(Arc::new(Field::new("key", DataType::Utf8, false)));
            columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
        }

        if schema.fields.find("type").is_none() {
            fields.push(Arc::new(Field::new("type", DataType::UInt8, false)));
            columns.push(Arc::new(UInt8Array::from(vec![0u8; num_rows])));
        }

        if schema.fields.find("int").is_none() {
            fields.push(Arc::new(Field::new("int", DataType::Int64, true)));
            columns.push(Arc::new(Int64Array::from(vec![0i64; num_rows])));
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed attrs batch")
    }

    fn complete_spans_batch(batch: RecordBatch) -> RecordBatch {
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
        wrap_struct_id_columns(&schema, &mut fields, &mut columns);

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed spans batch")
    }

    /// SpanEvents batch: needs a "name" column for transport decode
    fn complete_span_events_batch(batch: RecordBatch) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

        if schema.fields.find("name").is_none() {
            fields.push(Arc::new(Field::new("name", DataType::Utf8, true)));
            columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed span events batch")
    }

    /// SpanLinks batch: needs a "trace_id" column for transport decode
    fn complete_span_links_batch(batch: RecordBatch) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

        if schema.fields.find("trace_id").is_none() {
            fields.push(Arc::new(Field::new(
                "trace_id",
                DataType::FixedSizeBinary(16),
                true,
            )));
            let empty_bytes = vec![0u8; num_rows * 16];
            columns.push(Arc::new(
                arrow::array::FixedSizeBinaryArray::try_new(16, empty_bytes.into(), None)
                    .expect("Failed to create trace_id array"),
            ));
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to create completed span links batch")
    }

    /// Wraps flat "resource.id" / "scope.id" columns into struct columns
    /// (e.g. "resource.id" UInt16 -> "resource" Struct { "id": UInt16 })
    fn wrap_struct_id_columns(
        schema: &Schema,
        fields: &mut Vec<Arc<Field>>,
        columns: &mut Vec<ArrayRef>,
    ) {
        for &path in ID_COLUMN_PATHS {
            let Some(struct_name) = struct_column_name(path) else {
                continue;
            };
            let Some((idx, _)) = schema.fields.find(path) else {
                continue;
            };
            let id_col = columns.remove(idx);
            let _ = fields.remove(idx);
            let id_field = Field::new("id", id_col.data_type().clone(), true);
            let struct_col =
                StructArray::try_new(vec![id_field.clone()].into(), vec![id_col], None)
                    .expect("Failed to create struct");
            fields.push(Arc::new(Field::new(
                struct_name,
                DataType::Struct(vec![id_field].into()),
                true,
            )));
            columns.push(Arc::new(struct_col));
        }
    }

    fn mark_id_columns_plain(batch: RecordBatch) -> RecordBatch {
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields = schema.fields.to_vec();

        for &path in ID_COLUMN_PATHS {
            if let Some(col) = access_column(path, &schema, &columns) {
                replace_column(path, None, &schema, &mut columns, col);
                update_field_encoding_metadata(path, None, &mut fields);
            }
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to mark id columns as plain")
    }
}
