use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, ArrowPrimitiveType, AsArray, PrimitiveArray, RecordBatch};
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::{ArrowNativeType, DataType, UInt8Type, UInt16Type, UInt32Type};

use crate::error::{Error, Result};
use crate::otap::transform::transport_optimize::{
    access_column, remove_transport_optimized_encodings, replace_column,
};
use crate::otap::{Logs, Metrics, OtapBatchStore, POSITION_LOOKUP, Traces, UNUSED_INDEX};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::PARENT_ID;

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

    // Validate that we don't exceed u16::MAX items
    let log_count: usize = store
        .select(Logs::ROOT_PAYLOAD_TYPE)
        .map(|rb| rb.num_rows())
        .sum();

    if log_count > u16::MAX as usize + 1 {
        return Err(Error::TooManyItems {
            signal: "Logs".to_string(),
            count: log_count,
            max: u16::MAX as usize,
            message: "Too many items to reindex".to_string(),
        });
    }

    reindex_batch_store(&mut store)
}

pub fn reindex_metrics<const N: usize>(metrics: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut store = MultiBatchStore::<Metrics, { N }>::new(metrics);

    // Validate that we don't exceed u16::MAX items
    // FIXME [JD]: This is not right
    let metric_count: usize = store
        .select(Metrics::ROOT_PAYLOAD_TYPE)
        .map(|rb| rb.num_rows())
        .sum();

    if metric_count > u16::MAX as usize + 1 {
        return Err(Error::TooManyItems {
            signal: "Metrics".to_string(),
            count: metric_count,
            max: u16::MAX as usize,
            message: "Too many items to reindex".to_string(),
        });
    }

    reindex_batch_store(&mut store)
}

pub fn reindex_traces<const N: usize>(traces: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut store = MultiBatchStore::<Traces, { N }>::new(traces);

    // Validate that we don't exceed u16::MAX items
    let trace_count: usize = store
        .select(Traces::ROOT_PAYLOAD_TYPE)
        .map(|rb| rb.num_rows())
        .sum();

    if trace_count > u16::MAX as usize + 1 {
        return Err(Error::TooManyItems {
            signal: "Traces".to_string(),
            count: trace_count,
            max: u16::MAX as usize,
            message: "Too many items to reindex".to_string(),
        });
    }

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
    use crate::otap::payload_relations;

    // Iterate over all allowed payload types for this signal
    for &payload_type in S::allowed_payload_types() {
        for rb in store.select_mut(payload_type) {
            *rb = remove_transport_optimized_encodings(payload_type, rb)?;
        }

        // Get all relations (parent-child relationships) for this payload type
        for relation in payload_relations(payload_type) {
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
        let Some(parent_batch) = store.get_mut(i)[parent_idx].take() else {
            continue;
        };

        // Extract ID column - if it doesn't exist, skip reindexing for this batch
        // TODO [JD]: Return an error here or unwrap because id is a required column
        let id_col = match extract_id_column(&parent_batch, id_column_path) {
            Ok(col) => col,
            Err(_) => {
                // No ID column, put the batch back and continue
                store.get_mut(i)[parent_idx] = Some(parent_batch);
                continue;
            }
        };

        // Create mappings for the parent IDs
        let ids = materialize_id_column::<T>(id_col.as_ref())?;
        let ids = ids.values().to_vec();
        let sort_indices = sort_vec_to_indices(&ids);
        let mut sorted_ids = vec![T::Native::default(); ids.len()];
        take_vec(&ids, &mut sorted_ids, &sort_indices);

        let (mappings, new_offset) = create_mappings::<T>(&sorted_ids, offset)?;
        offset = new_offset;

        // Reindex parent batch using the mappings
        let parent_batch = reindex_batch_column::<T>(parent_batch, id_column_path, &mappings)?;

        // Put parent batch back
        store.get_mut(i)[parent_idx] = Some(parent_batch);

        // Apply mappings to each child batch one at a time
        for &child_payload_type in child_payload_types {
            let child_idx = payload_to_idx(child_payload_type);

            if let Some(child_batch) = store.get_mut(i)[child_idx].take() {
                let child_batch = reindex_batch_column::<T>(child_batch, PARENT_ID, &mappings)?;
                store.get_mut(i)[child_idx] = Some(child_batch);
            }
        }
    }

    Ok(())
}

/// Reindexes a single ID column in a record batch using the provided mappings
///
/// Returns the updated batch
fn reindex_batch_column<T>(
    batch: RecordBatch,
    column_path: &str,
    mappings: &[IdMapping<T::Native>],
) -> Result<RecordBatch>
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + AddAssign + SubAssign + ArrowNativeType,
{
    // Extract ID column
    let id_col = extract_id_column(&batch, column_path)?;
    let ids = materialize_id_column::<T>(id_col.as_ref())?;
    let mut ids = ids.values().to_vec();

    // Sort the ids
    let sort_indices = sort_vec_to_indices(&ids);
    let mut new_ids = vec![T::Native::default(); ids.len()];
    take_vec(&ids, &mut new_ids, &sort_indices);

    // Apply the mappings
    apply_mappings::<T>(&mut new_ids, mappings);

    // Unsort the IDs. Note that since `take` and `untake` can't be done
    // in place, we re-use the original id vec as the destination.
    untake_vec(&new_ids, &mut ids, &sort_indices);
    let new_ids_array = PrimitiveArray::<T>::new(ScalarBuffer::from(ids), None);

    // Replace ID column in batch
    let (schema, mut columns, _) = batch.into_parts();
    replace_column(
        column_path,
        None,
        &schema,
        &mut columns,
        Arc::new(new_ids_array),
    );
    let batch =
        RecordBatch::try_new(schema, columns).map_err(|e| Error::UnexpectedRecordBatchState {
            reason: format!("Failed to create batch: {}", e),
        })?;

    Ok(batch)
}

/// Extracts an ID column from a record batch
/// TODO [JD]: Probably get rid of this
fn extract_id_column(batch: &RecordBatch, column_path: &str) -> Result<ArrayRef> {
    access_column(column_path, &batch.schema(), batch.columns()).ok_or_else(|| {
        Error::ColumnNotFound {
            name: column_path.to_string(),
        }
    })
}

fn sort_vec_to_indices<T: Ord>(values: &Vec<T>) -> Vec<u32> {
    let mut indices: Vec<u32> = (0u32..values.len() as u32).collect();
    indices.sort_by_key(|&i| &values[i as usize]);
    indices
}

/// Materializes an ID array from either a direct array or dictionary  values
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

/// Applies mappings to a sorted ID buffer by scanning and applying offsets
///
/// This function scans through the sorted IDs and applies the appropriate offset
/// from the mappings. The mappings are processed in order.
fn apply_mappings<T>(sorted_ids: &mut [T::Native], mappings: &[IdMapping<T::Native>])
where
    T: ArrowPrimitiveType,
    T::Native: Ord + Copy + Add<Output = T::Native> + AddAssign + SubAssign,
{
    let mut remaining_slice = &mut sorted_ids[..];
    for mapping in mappings.iter() {
        if remaining_slice.len() == 0 {
            break;
        }

        // We don't have stuff in this mapping range, which is a little weird
        // but not impossible if items were filtered for example.
        if remaining_slice[0] < mapping.start_id {
            continue;
        }

        let end_idx = remaining_slice
            .iter()
            .position(|id| *id > mapping.end_id)
            .unwrap_or(remaining_slice.len());

        let slice_to_map = &mut remaining_slice[0..end_idx];
        // TODO [JD]: Anything we need to do here to make sure this is vectorized?
        // Might be beneficial to use chunks which vectorizes better.
        match mapping.sign {
            Sign::Positive => slice_to_map.iter_mut().for_each(|id| *id += mapping.offset),
            Sign::Negative => slice_to_map.iter_mut().for_each(|id| *id -= mapping.offset),
        }
        remaining_slice = &mut remaining_slice[end_idx..];
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use arrow::array::{
        Array, ArrayRef, AsArray, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, UInt16Type, UInt32Type};

    use crate::otap::transform::reindex::{payload_to_idx, reindex_logs};
    use crate::otap::transform::transport_optimize::access_column;
    use crate::otap::{Logs, OtapArrowRecords, OtapBatchStore};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::record_batch;
    use crate::schema::FieldExt;
    use crate::testing::equiv::assert_equivalent;
    use crate::testing::round_trip::otap_to_otlp;

    /// Known ID column names that need plain encoding metadata
    const ID_COLUMNS: &[&str] = &["id", "resource.id", "scope.id", "parent_id"];

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

    // ---- Tests ----

    #[test]
    fn test_logs_reindex_u16() {
        test_reindex_logs(vec![
            logs!(
                (Logs, ("id", UInt16, [1, 0])),
                (LogAttrs, ("parent_id", UInt16, [0, 0, 1, 1]))
            ),
            logs!(
                (Logs, ("id", UInt16, [1, 0])),
                (LogAttrs, ("parent_id", UInt16, [0, 0, 1, 1]))
            ),
        ]);
    }

    #[test]
    fn test_logs_reindex_u16_noop() {
        test_reindex_logs(vec![
            logs!(
                (Logs, ("id", UInt16, [0, 2, 1, 3])),
                (LogAttrs, ("parent_id", UInt16, [1, 2, 2, 0, 3]))
            ),
            logs!(
                (Logs, ("id", UInt16, [4, 6, 5, 7])),
                (LogAttrs, ("parent_id", UInt16, [6, 6, 5, 5, 7, 4]))
            ),
        ]);
    }

    // ---- Test helpers (callstack order) ----

    /// Validates reindexing for logs:
    /// 1. Converts input to OTLP (before reindex)
    /// 2. Reindexes the batches
    /// 3. Asserts no ID overlaps across batch groups
    /// 4. Converts output to OTLP (after reindex)
    /// 5. Asserts the OTLP data is equivalent
    fn test_reindex_logs(mut batches: Vec<[Option<RecordBatch>; Logs::COUNT]>) {
        // Convert to OTLP before reindexing
        let before_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&OtapArrowRecords::Logs(Logs { batches: b.clone() })))
            .collect();

        // Reindex
        reindex_logs(&mut batches).unwrap();

        // Validate no ID overlaps
        assert_no_id_overlaps::<Logs, { Logs::COUNT }>(&batches);

        // Convert to OTLP after reindexing
        let after_otlp: Vec<_> = batches
            .iter()
            .map(|b| otap_to_otlp(&OtapArrowRecords::Logs(Logs { batches: b.clone() })))
            .collect();

        // Assert equivalence
        assert_equivalent(&before_otlp, &after_otlp);
    }

    /// Validates that no ID column has overlapping values across batch groups.
    /// Uses payload_relations to discover all ID columns that should be unique.
    fn assert_no_id_overlaps<S: OtapBatchStore, const N: usize>(
        batches: &[[Option<RecordBatch>; N]],
    ) {
        use crate::otap::payload_relations;

        for &payload_type in S::allowed_payload_types() {
            let idx = payload_to_idx(payload_type);

            for relation in payload_relations(payload_type) {
                let mut seen = HashSet::new();

                for (group_idx, group) in batches.iter().enumerate() {
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
                            "Overlapping ID {} in column '{}' for {:?} (group {})",
                            id,
                            relation.key_col,
                            payload_type,
                            group_idx
                        );
                    }
                }
            }
        }
    }

    /// Extracts all ID values from a column as u64, handling UInt16 and UInt32 types
    fn collect_ids(col: &dyn Array) -> Vec<u64> {
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
            _ => vec![],
        }
    }

    fn make_test_batch<S: OtapBatchStore, const N: usize>(
        inputs: Vec<(ArrowPayloadType, RecordBatch)>,
    ) -> [Option<RecordBatch>; N] {
        let allowed = S::allowed_payload_types();
        let mut result: [Option<RecordBatch>; N] = std::array::from_fn(|_| None);

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

            result[idx] = Some(complete_batch(payload_type, batch));
        }

        result
    }

    fn complete_batch(payload_type: ArrowPayloadType, batch: RecordBatch) -> RecordBatch {
        let batch = match payload_type {
            ArrowPayloadType::Logs => complete_logs_batch(batch),
            ArrowPayloadType::LogAttrs
            | ArrowPayloadType::ResourceAttrs
            | ArrowPayloadType::ScopeAttrs => complete_attrs_batch(batch),
            _ => batch,
        };
        mark_id_columns_plain(batch)
    }

    fn complete_logs_batch(batch: RecordBatch) -> RecordBatch {
        let num_rows = batch.num_rows();
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

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

    fn mark_id_columns_plain(batch: RecordBatch) -> RecordBatch {
        let (schema, columns, _) = batch.into_parts();
        let fields: Vec<Arc<Field>> = schema
            .fields()
            .iter()
            .map(|f| {
                if ID_COLUMNS.contains(&f.name().as_str()) {
                    Arc::new(f.as_ref().clone().with_plain_encoding())
                } else {
                    f.clone()
                }
            })
            .collect();
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to mark id columns as plain")
    }
}
