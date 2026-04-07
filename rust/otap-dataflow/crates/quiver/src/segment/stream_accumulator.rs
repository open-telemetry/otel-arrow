// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Accumulates Arrow `RecordBatch`es for a single stream within a segment.
//!
//! A `StreamAccumulator` buffers batches in memory during segment accumulation.
//! When the segment is written to disk, the accumulator streams Arrow IPC file
//! bytes that can be memory-mapped for zero-copy reads.
//!
//! # Lifecycle
//!
//! 1. Create with [`StreamAccumulator::new`], providing the stream's schema.
//! 2. Append batches via [`StreamAccumulator::append`]; each call returns the
//!    chunk index for manifest bookkeeping.
//! 3. The accumulator is consumed by [`SegmentWriter::write_segment`] which
//!    streams the IPC data directly to disk.
//!
//! [`SegmentWriter::write_segment`]: super::SegmentWriter::write_segment
//!
//! # Dictionary Handling
//!
//! When writing multiple batches to IPC file format, dictionary-encoded columns
//! are unified so all batches share the same dictionary values per column. This
//! is required because Arrow IPC file format mandates a single dictionary per
//! field; the `FileWriter` rejects dictionary replacement across batches.
//!
//! Unification handles both top-level dictionary columns and dictionary fields
//! nested inside `Struct` columns (e.g., OTAP's `resource.schema_url`,
//! `scope.name`, `body.str`). For struct columns, each dictionary child is
//! unified independently while non-dictionary children pass through unchanged.
//!
//! The unification strategy concatenates all per-batch dictionary values arrays
//! into one unified values array, then offsets each batch's dictionary keys
//! accordingly. Since all batches share the same unified dictionary (via `Arc`),
//! Arrow's `DictionaryTracker` writes it once in the IPC file rather than
//! repeating it per batch.
//!
//! If the combined number of dictionary values exceeds the original key type's
//! capacity (e.g., more than 256 values for `UInt8` keys), the key type is
//! widened (e.g., `UInt8` -> `UInt16`). This schema change is preferred over
//! data loss from failed segment finalization.

use std::sync::Arc;

use arrow_array::types::{
    Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrayRef, DictionaryArray, RecordBatch, StructArray};
use arrow_buffer::ArrowNativeType;
use arrow_cast::cast;
use arrow_ipc::writer::{FileWriter, IpcWriteOptions};
use arrow_schema::{DataType, FieldRef, Fields, Schema, SchemaRef};
use arrow_select::concat::concat;

use super::error::SegmentError;
use super::types::{ChunkIndex, MAX_CHUNKS_PER_STREAM, StreamId, StreamKey, StreamMetadata};
use crate::logging::{otel_debug, otel_warn};
use crate::record_bundle::{SchemaFingerprint, SlotId};

/// Accumulates `RecordBatch`es for a single `(slot, schema)` stream.
///
/// This is an in-memory buffer that collects batches during segment
/// accumulation. On finalization, it produces Arrow IPC file bytes
/// suitable for memory-mapped reads.
pub struct StreamAccumulator {
    /// Unique identifier for this stream within the segment.
    stream_id: StreamId,
    /// The slot this stream serves.
    slot_id: SlotId,
    /// Schema fingerprint for routing verification.
    schema_fingerprint: SchemaFingerprint,
    /// Arrow schema for all batches in this stream.
    schema: SchemaRef,
    /// Buffered batches awaiting finalization.
    batches: Vec<RecordBatch>,
    /// Running count of rows across all batches.
    row_count: u64,
    /// Running total of buffer memory used by accumulated batches.
    buffer_size: usize,
    /// Whether finalize() has been called.
    finalized: bool,
}

impl StreamAccumulator {
    /// Creates a new stream accumulator.
    ///
    /// # Arguments
    ///
    /// * `stream_id` - Unique identifier for this stream within the segment.
    /// * `slot_id` - The payload slot this stream serves.
    /// * `schema_fingerprint` - 256-bit hash of the schema for routing.
    /// * `schema` - Arrow schema that all appended batches must match.
    #[must_use]
    pub const fn new(
        stream_id: StreamId,
        slot_id: SlotId,
        schema_fingerprint: SchemaFingerprint,
        schema: SchemaRef,
    ) -> Self {
        Self {
            stream_id,
            slot_id,
            schema_fingerprint,
            schema,
            batches: Vec::new(),
            row_count: 0,
            buffer_size: 0,
            finalized: false,
        }
    }

    /// Returns the stream's unique identifier.
    #[must_use]
    pub const fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    /// Returns the slot this stream serves.
    #[must_use]
    pub const fn slot_id(&self) -> SlotId {
        self.slot_id
    }

    /// Returns the schema fingerprint for this stream.
    #[must_use]
    pub const fn schema_fingerprint(&self) -> SchemaFingerprint {
        self.schema_fingerprint
    }

    /// Returns the stream key for this accumulator.
    #[must_use]
    pub const fn stream_key(&self) -> StreamKey {
        (self.slot_id, self.schema_fingerprint)
    }

    /// Returns the Arrow schema for this stream.
    #[must_use]
    pub const fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// Returns the number of chunks (batches) accumulated so far.
    #[must_use]
    pub const fn chunk_count(&self) -> u32 {
        self.batches.len() as u32
    }

    /// Returns the total row count across all accumulated batches.
    #[must_use]
    pub const fn row_count(&self) -> u64 {
        self.row_count
    }

    /// Returns the total buffer memory used by accumulated batches.
    ///
    /// This is the sum of `get_array_memory_size()` for all appended batches,
    /// representing the actual Arrow buffer memory consumption.
    #[must_use]
    pub const fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Returns true if this accumulator has no batches.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Returns true if finalize() has been called.
    #[must_use]
    pub const fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Appends a `RecordBatch` to this stream.
    ///
    /// Returns the chunk index assigned to this batch, which should be
    /// recorded in the batch manifest.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if the accumulator
    /// has already been finalized.
    /// Returns [`SegmentError::InvalidFormat`] if adding this batch would
    /// exceed the chunk limit.
    /// Returns [`SegmentError::SchemaMismatch`] if the batch schema doesn't
    /// match the stream's expected schema.
    pub fn append(&mut self, batch: RecordBatch) -> Result<ChunkIndex, SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }

        if self.batches.len() >= MAX_CHUNKS_PER_STREAM {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "stream {:?} already has {} chunks, cannot exceed limit of {}",
                    self.stream_id,
                    self.batches.len(),
                    MAX_CHUNKS_PER_STREAM
                ),
            });
        }

        // Validate schema matches - this catches routing bugs or data corruption
        if batch.schema() != self.schema {
            return Err(SegmentError::SchemaMismatch {
                stream_id: self.stream_id,
                expected: format!("{:?}", self.schema),
                actual: format!("{:?}", batch.schema()),
            });
        }

        let chunk_index = ChunkIndex::new(self.batches.len() as u32);
        self.row_count += batch.num_rows() as u64;
        self.buffer_size += batch.get_array_memory_size();
        self.batches.push(batch);

        Ok(chunk_index)
    }

    /// Writes all accumulated batches directly to a writer in Arrow IPC file format.
    ///
    /// Streams IPC bytes directly to the destination without buffering the entire
    /// serialized output in memory. After this call, the accumulator is consumed.
    ///
    /// # Arguments
    ///
    /// * `writer` - Destination for the Arrow IPC bytes.
    /// * `byte_offset` - The byte offset where this stream starts in the segment
    ///   file. Used for metadata only.
    ///
    /// # Returns
    ///
    /// The stream metadata including the actual byte length written.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if already finalized.
    /// Returns [`SegmentError::Arrow`] if IPC serialization fails.
    pub fn write_to<W: std::io::Write>(
        mut self,
        writer: &mut W,
        byte_offset: u64,
    ) -> Result<StreamMetadata, SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }
        self.finalized = true;

        let chunk_count = self.chunk_count();
        let row_count = self.row_count;

        // Wrap in CountingWriter to track bytes written
        let mut counting = CountingWriter::new(writer);
        self.write_ipc_to(&mut counting)?;
        let byte_length = counting.bytes_written() as u64;

        // Note: schema_fingerprint reflects the *ingestion* schema, which may
        // differ from the IPC file schema after dictionary unification
        // (key widening or native-type fallback). This is intentional -- the
        // fingerprint is used for write-path routing only.
        Ok(StreamMetadata::new(
            self.stream_id,
            self.slot_id,
            self.schema_fingerprint,
            byte_offset,
            byte_length,
            row_count,
            chunk_count,
        ))
    }

    /// Writes all accumulated batches to Arrow IPC file format.
    ///
    /// When multiple batches contain dictionary-encoded columns, their
    /// dictionaries are unified so all batches share the same dictionary
    /// values per column. This is required because Arrow IPC file format
    /// mandates a single dictionary per field across all record batches.
    fn write_ipc_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), SegmentError> {
        let options = IpcWriteOptions::default();

        // When there are multiple batches with dictionary-encoded columns, unify
        // the dictionaries before writing to avoid FileWriter's dictionary
        // replacement rejection.
        if self.batches.len() > 1 && has_dict_columns(&self.schema) {
            let (unified_schema, unified_batches) = unify_batch_dicts(&self.schema, &self.batches)?;
            let mut ipc_writer =
                FileWriter::try_new_with_options(writer, &unified_schema, options)?;
            for batch in &unified_batches {
                ipc_writer.write(batch)?;
            }
            ipc_writer.finish()?;
        } else {
            let mut ipc_writer = FileWriter::try_new_with_options(writer, &self.schema, options)?;
            for batch in &self.batches {
                ipc_writer.write(batch)?;
            }
            ipc_writer.finish()?;
        }

        Ok(())
    }
}

/// A writer wrapper that counts bytes written.
struct CountingWriter<W> {
    inner: W,
    bytes_written: usize,
}

impl<W> CountingWriter<W> {
    const fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
        }
    }

    const fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W: std::io::Write> std::io::Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes_written += n;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dictionary unification helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum dictionary key width that downstream readers support.
///
/// When dictionary unification produces a key type wider than this, the
/// dictionary encoding is stripped and columns are stored as the native
/// value type instead. This prevents writing segments that cannot be read
/// back by the OTAP reader stack (which only supports `UInt8` / `UInt16`
/// dictionary keys).
///
/// TODO: Make this configurable via `QuiverConfig` so non-OTAP embeddings
/// can set a different limit (e.g., `UInt32` or `UInt64`).
const MAX_DICT_KEY_TYPE: DataType = DataType::UInt16;

/// Returns `true` if `key_type` exceeds the maximum supported dictionary
/// key width.
fn exceeds_max_dict_key(key_type: &DataType) -> bool {
    key_type_capacity(key_type) > key_type_capacity(&MAX_DICT_KEY_TYPE)
}

/// Returns `true` if any field in the schema uses dictionary encoding,
/// including dictionary fields nested inside `Struct` columns.
fn has_dict_columns(schema: &SchemaRef) -> bool {
    schema
        .fields()
        .iter()
        .any(|f| field_has_dict(f.data_type()))
}

/// Checks whether a data type contains dictionary encoding at the top level
/// or one level deep inside a `Struct`.
fn field_has_dict(dt: &DataType) -> bool {
    match dt {
        DataType::Dictionary(_, _) => true,
        DataType::Struct(fields) => fields
            .iter()
            .any(|f| matches!(f.data_type(), DataType::Dictionary(_, _))),
        _ => false,
    }
}

/// Unifies dictionary columns across batches so all batches share the same
/// dictionary values for each dictionary-encoded field.
///
/// For each dictionary column, the values arrays from all batches are
/// concatenated into a single unified values array. Each batch's dictionary
/// keys are then offset to index into this unified array. If the total
/// number of values exceeds the original key type's capacity, the key type
/// is widened.
fn unify_batch_dicts(
    schema: &SchemaRef,
    batches: &[RecordBatch],
) -> Result<(SchemaRef, Vec<RecordBatch>), SegmentError> {
    let num_cols = schema.fields().len();
    let num_batches = batches.len();

    // For each column, build the unified columns across all batches.
    // column_data[col_idx][batch_idx] = the new ArrayRef for that position.
    let mut column_data: Vec<Vec<ArrayRef>> = Vec::with_capacity(num_cols);
    let mut new_fields: Vec<FieldRef> = Vec::with_capacity(num_cols);

    for col_idx in 0..num_cols {
        let field = &schema.fields()[col_idx];

        if let DataType::Dictionary(key_type, value_type) = field.data_type() {
            let (columns, new_field) =
                unify_dict_column(col_idx, field, key_type, value_type, batches)?;
            column_data.push(columns);
            new_fields.push(new_field);
        } else if let DataType::Struct(struct_fields) = field.data_type() {
            if struct_fields.iter().any(|f| field_has_dict(f.data_type())) {
                let (columns, new_field) =
                    unify_struct_column(col_idx, field, struct_fields, batches)?;
                column_data.push(columns);
                new_fields.push(new_field);
            } else {
                // Struct without any dict children: pass through unchanged.
                let cols: Vec<ArrayRef> = batches
                    .iter()
                    .map(|b| Arc::clone(b.column(col_idx)))
                    .collect();
                column_data.push(cols);
                new_fields.push(Arc::clone(field));
            }
        } else {
            // Non-dict, non-struct column: pass through unchanged.
            let cols: Vec<ArrayRef> = batches
                .iter()
                .map(|b| Arc::clone(b.column(col_idx)))
                .collect();
            column_data.push(cols);
            new_fields.push(Arc::clone(field));
        }
    }

    let new_schema = Arc::new(Schema::new_with_metadata(
        new_fields,
        schema.metadata().clone(),
    ));

    let new_batches = (0..num_batches)
        .map(|batch_idx| {
            let columns: Vec<ArrayRef> = (0..num_cols)
                .map(|col_idx| Arc::clone(&column_data[col_idx][batch_idx]))
                .collect();
            RecordBatch::try_new(Arc::clone(&new_schema), columns)
                .map_err(|e| SegmentError::Arrow { source: e })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((new_schema, new_batches))
}

/// Unifies a single dictionary column across all batches.
///
/// Returns the new per-batch columns and the (possibly updated) field.
///
/// If the unified dictionary cardinality exceeds the maximum supported key
/// width ([`MAX_DICT_KEY_TYPE`]), the dictionary encoding is stripped and
/// columns are cast to the native value type.
fn unify_dict_column(
    col_idx: usize,
    field: &FieldRef,
    key_type: &DataType,
    value_type: &DataType,
    batches: &[RecordBatch],
) -> Result<(Vec<ArrayRef>, FieldRef), SegmentError> {
    // Collect per-batch values arrays and compute offsets.
    let mut value_refs: Vec<&dyn Array> = Vec::with_capacity(batches.len());
    let mut batch_offsets: Vec<usize> = Vec::with_capacity(batches.len());
    let mut total_values: usize = 0;

    for batch in batches {
        let col = batch.column(col_idx);
        let values = dict_values(col, key_type)?;
        batch_offsets.push(total_values);
        total_values += values.len();
        value_refs.push(values);
    }

    // Concatenate all values into a unified array.
    let unified_values: ArrayRef = if value_refs.is_empty() || total_values == 0 {
        arrow_array::new_empty_array(value_type)
    } else {
        concat(&value_refs)?
    };

    // Determine the key type that can address all unified values.
    let effective_key_type = widen_key_type(key_type, total_values)?;

    // If the effective key type exceeds the maximum supported width, fall
    // back to the native value type by casting each batch's dictionary
    // column. This avoids writing segments that cannot be read back.
    if exceeds_max_dict_key(&effective_key_type) {
        otel_warn!(
            "quiver.dict.native_fallback",
            field = field.name().as_str(),
            original_key_type = ?key_type,
            total_values = total_values,
            message = "dictionary cardinality exceeds max key width; \
                       falling back to native type. Consider reducing \
                       segment.target_size or segment.max_open_duration",
        );
        let native_type = value_type.clone();
        let columns: Vec<ArrayRef> = batches
            .iter()
            .map(|batch| {
                cast(batch.column(col_idx), &native_type)
                    .map_err(|e| SegmentError::Arrow { source: e })
            })
            .collect::<Result<_, _>>()?;
        let new_field = Arc::new(field.as_ref().clone().with_data_type(native_type));
        return Ok((columns, new_field));
    }

    // Build remapped columns for each batch.
    let columns: Vec<ArrayRef> = batches
        .iter()
        .zip(batch_offsets.iter())
        .map(|(batch, &offset)| {
            remap_dict_keys(
                batch.column(col_idx),
                key_type,
                &effective_key_type,
                &unified_values,
                offset,
            )
        })
        .collect::<Result<_, _>>()?;

    // Update the field if the key type was widened.
    let new_field = if effective_key_type != *key_type {
        otel_debug!(
            "quiver.dict.key_widened",
            field = field.name().as_str(),
            original_key_type = ?key_type,
            widened_key_type = ?effective_key_type,
            total_values = total_values,
        );
        Arc::new(field.as_ref().clone().with_data_type(DataType::Dictionary(
            Box::new(effective_key_type),
            Box::new(value_type.clone()),
        )))
    } else {
        Arc::clone(field)
    };

    Ok((columns, new_field))
}

/// Unifies dictionary children inside a `Struct` column across all batches.
///
/// Each dictionary child is unified independently (same strategy as top-level
/// dictionary columns). Non-dictionary children are passed through unchanged.
/// The struct is then rebuilt with the unified children for each batch.
fn unify_struct_column(
    col_idx: usize,
    field: &FieldRef,
    struct_fields: &Fields,
    batches: &[RecordBatch],
) -> Result<(Vec<ArrayRef>, FieldRef), SegmentError> {
    let num_children = struct_fields.len();
    let num_batches = batches.len();

    // Extract StructArrays from each batch for this column.
    let struct_arrays: Vec<&StructArray> = batches
        .iter()
        .map(|b| {
            b.column(col_idx)
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| SegmentError::InvalidFormat {
                    message: format!(
                        "expected StructArray at column {col_idx}, got {:?}",
                        b.column(col_idx).data_type()
                    ),
                })
        })
        .collect::<Result<_, _>>()?;

    // For each child field, either unify its dictionary or pass through.
    // child_data[child_idx][batch_idx] = unified ArrayRef
    let mut child_data: Vec<Vec<ArrayRef>> = Vec::with_capacity(num_children);
    let mut new_child_fields: Vec<FieldRef> = Vec::with_capacity(num_children);

    for child_idx in 0..num_children {
        let child_field = &struct_fields[child_idx];

        if let DataType::Dictionary(key_type, value_type) = child_field.data_type() {
            // Collect the child arrays and unify them.
            let child_arrays: Vec<ArrayRef> = struct_arrays
                .iter()
                .map(|sa| Arc::clone(sa.column(child_idx)))
                .collect();

            let mut value_refs: Vec<&dyn Array> = Vec::with_capacity(num_batches);
            let mut batch_offsets: Vec<usize> = Vec::with_capacity(num_batches);
            let mut total_values: usize = 0;

            for arr in &child_arrays {
                let values = dict_values(arr, key_type)?;
                batch_offsets.push(total_values);
                total_values += values.len();
                value_refs.push(values);
            }

            let unified_values: ArrayRef = if value_refs.is_empty() || total_values == 0 {
                arrow_array::new_empty_array(value_type)
            } else {
                concat(&value_refs)?
            };

            let effective_key_type = widen_key_type(key_type, total_values)?;

            // If the effective key type exceeds the max supported width,
            // fall back to native value type for this struct child.
            if exceeds_max_dict_key(&effective_key_type) {
                otel_warn!(
                    "quiver.dict.native_fallback",
                    field = %format!("{}.{}", field.name(), child_field.name()),
                    original_key_type = ?key_type,
                    total_values = total_values,
                    message = "dictionary cardinality exceeds max key width; \
                               falling back to native type",
                );
                let native_type: DataType = *value_type.clone();
                let native_children: Vec<ArrayRef> = child_arrays
                    .iter()
                    .map(|arr| {
                        cast(arr.as_ref(), &native_type)
                            .map_err(|e| SegmentError::Arrow { source: e })
                    })
                    .collect::<Result<_, _>>()?;
                child_data.push(native_children);
                new_child_fields.push(Arc::new(
                    child_field.as_ref().clone().with_data_type(native_type),
                ));
            } else {
                let unified_children: Vec<ArrayRef> = child_arrays
                    .iter()
                    .zip(batch_offsets.iter())
                    .map(|(arr, &offset)| {
                        remap_dict_keys(arr, key_type, &effective_key_type, &unified_values, offset)
                    })
                    .collect::<Result<_, _>>()?;

                child_data.push(unified_children);

                if effective_key_type != **key_type {
                    otel_debug!(
                        "quiver.dict.key_widened",
                        field = %format!("{}.{}", field.name(), child_field.name()),
                        original_key_type = ?key_type,
                        widened_key_type = ?effective_key_type,
                        total_values = total_values,
                    );
                    new_child_fields.push(Arc::new(child_field.as_ref().clone().with_data_type(
                        DataType::Dictionary(Box::new(effective_key_type), value_type.clone()),
                    )));
                } else {
                    new_child_fields.push(Arc::clone(child_field));
                }
            }
        } else {
            // Non-dict child: pass through from each struct.
            let cols: Vec<ArrayRef> = struct_arrays
                .iter()
                .map(|sa| Arc::clone(sa.column(child_idx)))
                .collect();
            child_data.push(cols);
            new_child_fields.push(Arc::clone(child_field));
        }
    }

    // Rebuild StructArrays for each batch with the unified children.
    let columns: Vec<ArrayRef> = (0..num_batches)
        .map(|batch_idx| {
            let nulls = struct_arrays[batch_idx].nulls().cloned();
            let sa = StructArray::try_new(
                Fields::from(new_child_fields.clone()),
                (0..num_children)
                    .map(|ci| Arc::clone(&child_data[ci][batch_idx]))
                    .collect(),
                nulls,
            )
            .map_err(|e| SegmentError::Arrow { source: e })?;
            Ok(Arc::new(sa) as ArrayRef)
        })
        .collect::<Result<_, SegmentError>>()?;

    // Update the field's data type if any child field changed.
    let new_struct_fields = Fields::from(new_child_fields.clone());
    let new_field = if *struct_fields != new_struct_fields {
        Arc::new(
            field
                .as_ref()
                .clone()
                .with_data_type(DataType::Struct(new_struct_fields)),
        )
    } else {
        Arc::clone(field)
    };

    Ok((columns, new_field))
}

/// Dispatches a dictionary operation across all supported Arrow dictionary key
/// types. The macro matches a `&DataType` expression and invokes `$body` with
/// `$arrow_ty` bound to the concrete `ArrowDictionaryKeyType`.
macro_rules! dispatch_dict_key {
    ($key_type:expr, $arrow_ty:ident => $body:expr, $err:expr) => {
        match $key_type {
            DataType::Int8 => {
                type $arrow_ty = Int8Type;
                $body
            }
            DataType::Int16 => {
                type $arrow_ty = Int16Type;
                $body
            }
            DataType::Int32 => {
                type $arrow_ty = Int32Type;
                $body
            }
            DataType::Int64 => {
                type $arrow_ty = Int64Type;
                $body
            }
            DataType::UInt8 => {
                type $arrow_ty = UInt8Type;
                $body
            }
            DataType::UInt16 => {
                type $arrow_ty = UInt16Type;
                $body
            }
            DataType::UInt32 => {
                type $arrow_ty = UInt32Type;
                $body
            }
            DataType::UInt64 => {
                type $arrow_ty = UInt64Type;
                $body
            }
            _ => $err,
        }
    };
}

/// Extracts the values array from a dictionary column.
fn dict_values<'a>(col: &'a ArrayRef, key_type: &DataType) -> Result<&'a dyn Array, SegmentError> {
    dispatch_dict_key!(key_type, K => {
        col.as_any()
            .downcast_ref::<DictionaryArray<K>>()
            .map(|d| d.values().as_ref())
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("expected DictionaryArray<{}>", stringify!(K)),
            })
    }, Err(SegmentError::InvalidFormat {
        message: format!("unsupported dictionary key type: {key_type:?}"),
    }))
}

/// Maximum number of dictionary values addressable by a given key type.
///
/// For signed types, only the non-negative range is usable since Arrow
/// dictionary keys are zero-based indices (e.g., `Int8` → 128 values,
/// not 256).
fn key_type_capacity(dt: &DataType) -> usize {
    match dt {
        DataType::Int8 => i8::MAX as usize + 1,
        DataType::Int16 => i16::MAX as usize + 1,
        DataType::Int32 => i32::MAX as usize + 1,
        DataType::Int64 => i64::MAX as usize + 1,
        DataType::UInt8 => u8::MAX as usize + 1,
        DataType::UInt16 => u16::MAX as usize + 1,
        DataType::UInt32 => u32::MAX as usize + 1,
        DataType::UInt64 => usize::MAX, // effectively unlimited on 64-bit
        _ => 0,
    }
}

/// Returns the smallest key type that can address `total_values` entries,
/// widening from the `original` type if necessary. Signed key types widen
/// within the signed family; unsigned within the unsigned family.
fn widen_key_type(original: &DataType, total_values: usize) -> Result<DataType, SegmentError> {
    if total_values <= key_type_capacity(original) {
        return Ok(original.clone());
    }
    // Widening chain preserves signedness.
    let chain: &[DataType] = match original {
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
            &[DataType::Int16, DataType::Int32, DataType::Int64]
        }
        _ => &[DataType::UInt16, DataType::UInt32, DataType::UInt64],
    };
    for candidate in chain {
        if total_values <= key_type_capacity(candidate) {
            return Ok(candidate.clone());
        }
    }
    Err(SegmentError::InvalidFormat {
        message: format!(
            "dictionary has {total_values} values, exceeding maximum addressable keys"
        ),
    })
}

/// Extracts dictionary keys from `col` as `u64` values, each offset by
/// `offset`. Null keys remain `None`.
fn extract_offset_keys(
    col: &ArrayRef,
    key_type: &DataType,
    offset: usize,
) -> Result<Vec<Option<u64>>, SegmentError> {
    dispatch_dict_key!(key_type, K => {
        let dict = col
            .as_any()
            .downcast_ref::<DictionaryArray<K>>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("expected DictionaryArray<{}>", stringify!(K)),
            })?;
        Ok(dict.keys().iter().map(|k| k.map(|v| {
            // Dictionary keys are always non-negative indices, so the
            // widening cast to u64 is lossless for all key types.
            debug_assert!(v.as_usize() < usize::MAX / 2, "dictionary key is negative");
            let wide = v.as_usize() as u64;
            wide + offset as u64
        })).collect())
    }, Err(SegmentError::InvalidFormat {
        message: format!("unsupported dictionary key type: {key_type:?}"),
    }))
}

/// Builds a `DictionaryArray` with the specified `target_key_type` from
/// pre-computed `u64` keys and a unified values array.
fn build_dict_array(
    keys: &[Option<u64>],
    target_key_type: &DataType,
    unified_values: &ArrayRef,
) -> Result<ArrayRef, SegmentError> {
    // Safety of the `as` narrowing casts below: `widen_key_type` has already
    // verified that all key values fit within the target type's range.
    dispatch_dict_key!(target_key_type, K => {
        let typed_keys: arrow_array::PrimitiveArray<K> = keys
            .iter()
            .map(|k| k.map(|v| <K as arrow_array::ArrowPrimitiveType>::Native::usize_as(v as usize)))
            .collect();
        Ok(Arc::new(DictionaryArray::new(typed_keys, Arc::clone(unified_values))))
    }, Err(SegmentError::InvalidFormat {
        message: format!("unsupported target dictionary key type: {target_key_type:?}"),
    }))
}

/// Remaps dictionary keys to reference a unified values array, applying an
/// offset and optionally widening the key type.
fn remap_dict_keys(
    col: &ArrayRef,
    original_key_type: &DataType,
    target_key_type: &DataType,
    unified_values: &ArrayRef,
    offset: usize,
) -> Result<ArrayRef, SegmentError> {
    let keys = extract_offset_keys(col, original_key_type, offset)?;
    build_dict_array(&keys, target_key_type, unified_values)
}

impl std::fmt::Debug for StreamAccumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamAccumulator")
            .field("stream_id", &self.stream_id)
            .field("slot_id", &self.slot_id)
            .field("chunk_count", &self.chunk_count())
            .field("row_count", &self.row_count)
            .field("finalized", &self.finalized)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch, StringArray};
    use arrow_ipc::reader::FileReader;
    use arrow_schema::{DataType, Field, Schema};

    use super::*;
    use crate::segment::test_utils::{make_batch, test_fingerprint, test_schema};

    #[test]
    fn new_accumulator_is_empty() {
        let schema = test_schema();
        let acc =
            StreamAccumulator::new(StreamId::new(0), SlotId::new(0), test_fingerprint(), schema);

        assert!(acc.is_empty());
        assert_eq!(acc.chunk_count(), 0);
        assert_eq!(acc.row_count(), 0);
        assert!(!acc.is_finalized());
    }

    #[test]
    fn append_returns_sequential_chunk_indices() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3], &["c"]);
        let batch3 = make_batch(&schema, &[4, 5, 6], &["d", "e", "f"]);

        assert_eq!(acc.append(batch1).unwrap(), ChunkIndex::new(0));
        assert_eq!(acc.append(batch2).unwrap(), ChunkIndex::new(1));
        assert_eq!(acc.append(batch3).unwrap(), ChunkIndex::new(2));

        assert_eq!(acc.chunk_count(), 3);
        assert_eq!(acc.row_count(), 6); // 2 + 1 + 3
    }

    #[test]
    fn append_after_finalize_fails() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch = make_batch(&schema, &[1], &["a"]);
        let _ = acc.append(batch).unwrap();

        // Finalize consumes self, so we need a new accumulator to test the error
        let mut acc2 = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );
        acc2.finalized = true; // Simulate already finalized

        let batch2 = make_batch(&schema, &[2], &["b"]);
        let result = acc2.append(batch2);
        assert!(matches!(result, Err(SegmentError::AccumulatorFinalized)));
    }

    #[test]
    fn write_to_produces_valid_arrow_ipc() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(5),
            SlotId::new(2),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_batch(&schema, &[1, 2], &["alice", "bob"]);
        let batch2 = make_batch(&schema, &[3, 4, 5], &["charlie", "diana", "eve"]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        // Write to a buffer using write_to
        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 1024).unwrap();

        // Verify metadata
        assert_eq!(metadata.id, StreamId::new(5));
        assert_eq!(metadata.slot_id, SlotId::new(2));
        assert_eq!(metadata.schema_fingerprint, test_fingerprint());
        assert_eq!(metadata.byte_offset, 1024);
        assert!(metadata.byte_length > 0);
        assert_eq!(metadata.row_count, 5);
        assert_eq!(metadata.chunk_count, 2);

        // Verify IPC bytes are readable
        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // Schema should match
        assert_eq!(reader.schema(), schema);

        // Read all batches
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify batch contents
        assert_eq!(batches[0].num_rows(), 2);
        assert_eq!(batches[1].num_rows(), 3);
    }

    #[test]
    fn write_to_empty_accumulator_produces_valid_ipc() {
        let schema = test_schema();
        let acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            schema.clone(),
        );

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();

        assert_eq!(metadata.chunk_count, 0);
        assert_eq!(metadata.row_count, 0);

        // Empty IPC file should still be readable
        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        assert_eq!(reader.schema(), schema);

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert!(batches.is_empty());
    }

    #[test]
    fn stream_key_matches_constructor_args() {
        let schema = test_schema();
        let fp = [0x42u8; 32];
        const SLOT_ID: u16 = 3;
        let acc = StreamAccumulator::new(StreamId::new(7), SlotId::new(SLOT_ID), fp, schema);

        let key = acc.stream_key();
        assert_eq!(key.0, SlotId::new(SLOT_ID));
        assert_eq!(key.1, fp);
    }

    #[test]
    fn accessors_return_expected_values() {
        let schema = test_schema();
        let fp = [0x99u8; 32];
        let acc =
            StreamAccumulator::new(StreamId::new(10), SlotId::new(5), fp, Arc::clone(&schema));

        assert_eq!(acc.stream_id(), StreamId::new(10));
        assert_eq!(acc.slot_id(), SlotId::new(5));
        assert_eq!(acc.schema_fingerprint(), fp);
        assert_eq!(acc.schema(), &schema);
    }

    #[test]
    fn debug_impl_does_not_panic() {
        let schema = test_schema();
        let acc =
            StreamAccumulator::new(StreamId::new(0), SlotId::new(0), test_fingerprint(), schema);
        let debug_str = format!("{:?}", acc);
        assert!(debug_str.contains("StreamAccumulator"));
        assert!(debug_str.contains("stream_id"));
    }

    #[test]
    fn append_rejects_schema_mismatch() {
        let schema1 = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));

        let schema2 = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("extra", DataType::Utf8, true),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema1),
        );

        // Create a batch with a different schema
        let batch = RecordBatch::try_new(
            Arc::clone(&schema2),
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["a", "b"])),
            ],
        )
        .unwrap();

        let result = acc.append(batch);

        match result {
            Err(SegmentError::SchemaMismatch {
                stream_id,
                expected,
                actual,
            }) => {
                assert_eq!(stream_id, StreamId::new(0));
                // Expected schema has 1 field, actual has 2
                assert!(expected.contains("id"));
                assert!(!expected.contains("extra"));
                assert!(actual.contains("id"));
                assert!(actual.contains("extra"));
            }
            other => panic!("Expected SchemaMismatch error, got: {:?}", other),
        }
    }

    /// Helper: creates a schema with a dictionary-encoded string column plus an int column.
    fn dict_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "label",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
        ]))
    }

    /// Helper: creates a RecordBatch with nullable Dict(UInt8, Utf8) labels.
    /// `labels` entries of `None` produce null dictionary keys.
    fn make_dict_batch(schema: &SchemaRef, ids: &[i32], labels: &[Option<&str>]) -> RecordBatch {
        use arrow_array::builder::StringDictionaryBuilder;

        let id_array = Int32Array::from(ids.to_vec());
        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        for label in labels {
            match label {
                Some(v) => dict_builder.append_value(v),
                None => dict_builder.append_null(),
            }
        }
        let dict_array = dict_builder.finish();

        RecordBatch::try_new(
            Arc::clone(schema),
            vec![Arc::new(id_array), Arc::new(dict_array)],
        )
        .expect("valid batch")
    }

    /// Reads back dict string values (including nulls) from a batch column.
    fn read_dict_strings(batch: &RecordBatch, col_idx: usize) -> Vec<Option<String>> {
        // The key type may have been widened, so try multiple key types.
        let col = batch.column(col_idx);
        let dt = col.data_type();
        match dt {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                DataType::UInt16 => col
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                DataType::Int8 => col
                    .as_any()
                    .downcast_ref::<DictionaryArray<Int8Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                DataType::Int16 => col
                    .as_any()
                    .downcast_ref::<DictionaryArray<Int16Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                other => panic!("unexpected dict key type in test: {other:?}"),
            },
            other => panic!("expected dictionary column, got {other:?}"),
        }
    }

    #[test]
    fn write_to_with_dict_columns_varying_values() {
        // Core bug scenario: multiple batches with different dictionary values.
        // Before the fix, FileWriter rejected this with
        // "Dictionary replacement detected".
        let schema = dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_dict_batch(&schema, &[1, 2], &[Some("hello"), Some("world")]);
        let batch2 = make_dict_batch(&schema, &[3, 4], &[Some("foo"), Some("bar")]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();

        assert_eq!(metadata.row_count, 4);
        assert_eq!(metadata.chunk_count, 2);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify dict values round-trip in each batch.
        assert_eq!(
            read_dict_strings(&batches[0], 1),
            vec![Some("hello".into()), Some("world".into())]
        );
        assert_eq!(
            read_dict_strings(&batches[1], 1),
            vec![Some("foo".into()), Some("bar".into())]
        );

        // Verify the non-dict column also round-trips correctly.
        let ids0: Vec<i32> = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap()
            .values()
            .to_vec();
        assert_eq!(ids0, vec![1, 2]);
        let ids1: Vec<i32> = batches[1]
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap()
            .values()
            .to_vec();
        assert_eq!(ids1, vec![3, 4]);
    }

    #[test]
    fn write_to_with_dict_columns_single_batch_unchanged() {
        // A single batch skips unification entirely.
        let schema = dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch = make_dict_batch(&schema, &[1, 2], &[Some("hello"), Some("world")]);
        let _ = acc.append(batch).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.chunk_count, 1);
        assert_eq!(metadata.row_count, 2);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        // Schema unchanged (still Dict(UInt8, Utf8))
        assert_eq!(
            reader.schema().field(1).data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );
    }

    #[test]
    fn write_to_with_dict_null_keys() {
        // Null dictionary keys must survive unification.
        let schema = dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_dict_batch(&schema, &[1, 2], &[Some("a"), None]);
        let batch2 = make_dict_batch(&schema, &[3, 4], &[None, Some("b")]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        assert_eq!(
            read_dict_strings(&batches[0], 1),
            vec![Some("a".into()), None]
        );
        assert_eq!(
            read_dict_strings(&batches[1], 1),
            vec![None, Some("b".into())]
        );
    }

    #[test]
    fn write_to_with_three_dict_batches_offsets_accumulate() {
        // Three batches ensure offsets accumulate beyond the first pair.
        let schema = dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_dict_batch(&schema, &[1], &[Some("alpha")]);
        let batch2 = make_dict_batch(&schema, &[2], &[Some("beta")]);
        let batch3 = make_dict_batch(&schema, &[3], &[Some("gamma")]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();
        let _ = acc.append(batch3).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.chunk_count, 3);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 3);

        assert_eq!(
            read_dict_strings(&batches[0], 1),
            vec![Some("alpha".into())]
        );
        assert_eq!(read_dict_strings(&batches[1], 1), vec![Some("beta".into())]);
        assert_eq!(
            read_dict_strings(&batches[2], 1),
            vec![Some("gamma".into())]
        );
    }

    #[test]
    fn write_to_with_many_unique_dict_values_widens_key_type() {
        // Total dictionary values across batches exceeds UInt8 capacity (256),
        // forcing widening to UInt16.
        let schema = dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        // 200 + 200 = 400 total values > 256
        let labels1: Vec<String> = (0..200).map(|i| format!("val_a_{i}")).collect();
        let labels2: Vec<String> = (0..200).map(|i| format!("val_b_{i}")).collect();

        let opts1: Vec<Option<&str>> = labels1.iter().map(|s| Some(s.as_str())).collect();
        let opts2: Vec<Option<&str>> = labels2.iter().map(|s| Some(s.as_str())).collect();

        let ids1: Vec<i32> = (0..200).collect();
        let ids2: Vec<i32> = (200..400).collect();

        let batch1 = make_dict_batch(&schema, &ids1, &opts1);
        let batch2 = make_dict_batch(&schema, &ids2, &opts2);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // Key type widened to UInt16.
        assert_eq!(
            reader.schema().field(1).data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        );

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].num_rows(), 200);
        assert_eq!(batches[1].num_rows(), 200);

        // Spot-check first and last values in each batch.
        let vals0 = read_dict_strings(&batches[0], 1);
        assert_eq!(vals0[0].as_deref(), Some("val_a_0"));
        assert_eq!(vals0[199].as_deref(), Some("val_a_199"));

        let vals1 = read_dict_strings(&batches[1], 1);
        assert_eq!(vals1[0].as_deref(), Some("val_b_0"));
        assert_eq!(vals1[199].as_deref(), Some("val_b_199"));
    }

    #[test]
    fn write_to_with_dict_overflow_falls_back_to_native_type() {
        // Total dictionary values across batches exceeds UInt16 capacity
        // (65536), triggering the native type fallback. The dictionary
        // column should be stored as plain Utf8 instead of Dict.
        //
        // Two batches with 40000 unique values each = 80000 total > 65536.
        // UInt8 builder can only hold 256, so we need to build these manually
        // with UInt8 keys that wrap around (all mapping to unique values).
        // Actually, we need to use a schema with UInt16 keys to hold 40K values.
        let schema_u16 = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "label",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema_u16),
        );

        let count = 40_000;
        let labels1: Vec<String> = (0..count).map(|i| format!("a_{i}")).collect();
        let labels2: Vec<String> = (0..count).map(|i| format!("b_{i}")).collect();

        let values1 = StringArray::from(labels1.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let keys1 =
            arrow_array::PrimitiveArray::<UInt16Type>::from((0..count as u16).collect::<Vec<_>>());
        let dict1 = DictionaryArray::new(keys1, Arc::new(values1));
        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema_u16),
            vec![
                Arc::new(Int32Array::from((0..count as i32).collect::<Vec<_>>())),
                Arc::new(dict1),
            ],
        )
        .unwrap();

        let values2 = StringArray::from(labels2.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let keys2 =
            arrow_array::PrimitiveArray::<UInt16Type>::from((0..count as u16).collect::<Vec<_>>());
        let dict2 = DictionaryArray::new(keys2, Arc::new(values2));
        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema_u16),
            vec![
                Arc::new(Int32Array::from(
                    (count as i32..2 * count as i32).collect::<Vec<_>>(),
                )),
                Arc::new(dict2),
            ],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.row_count, 2 * count as u64);
        assert_eq!(metadata.chunk_count, 2);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // The label column should have fallen back to plain Utf8 (not Dict).
        assert_eq!(
            reader.schema().field(1).data_type(),
            &DataType::Utf8,
            "dictionary should fall back to native Utf8 when cardinality exceeds UInt16"
        );

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].num_rows(), count);
        assert_eq!(batches[1].num_rows(), count);

        // Spot-check values are correct after fallback.
        let col0 = batches[0]
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("should be plain StringArray after fallback");
        assert_eq!(col0.value(0), "a_0");
        assert_eq!(col0.value(count - 1), &format!("a_{}", count - 1));

        let col1 = batches[1]
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("should be plain StringArray after fallback");
        assert_eq!(col1.value(0), "b_0");
        assert_eq!(col1.value(count - 1), &format!("b_{}", count - 1));
    }

    #[test]
    fn write_to_with_signed_dict_key_type() {
        // Exercises signed (Int8) dictionary key type.
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "label",
                DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        // Build batches using Int8 dictionary keys.
        use arrow_array::builder::StringDictionaryBuilder;

        let mut b1 = StringDictionaryBuilder::<Int8Type>::new();
        b1.append_value("x");
        b1.append_value("y");
        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(b1.finish()),
            ],
        )
        .unwrap();

        let mut b2 = StringDictionaryBuilder::<Int8Type>::new();
        b2.append_value("z");
        b2.append_null();
        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Int32Array::from(vec![3, 4])),
                Arc::new(b2.finish()),
            ],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // Key type should remain Int8 (4 values fits).
        assert_eq!(
            reader.schema().field(1).data_type(),
            &DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8))
        );

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        assert_eq!(
            read_dict_strings(&batches[0], 1),
            vec![Some("x".into()), Some("y".into())]
        );
        assert_eq!(
            read_dict_strings(&batches[1], 1),
            vec![Some("z".into()), None]
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Struct-with-dictionary tests (OTAP schemas embed Dict inside Struct)
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a schema resembling OTAP's resource/scope pattern:
    /// a top-level int column plus a Struct column whose children include
    /// both a dictionary field and a plain field.
    fn struct_dict_schema() -> SchemaRef {
        use arrow_schema::Field;
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "scope",
                DataType::Struct(Fields::from(vec![
                    Field::new(
                        "name",
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        true,
                    ),
                    Field::new("version_num", DataType::Int32, true),
                ])),
                true,
            ),
        ]))
    }

    /// Builds a batch with a Struct(Dict(UInt8,Utf8), Int32) column.
    fn make_struct_dict_batch(
        schema: &SchemaRef,
        ids: &[i32],
        names: &[Option<&str>],
        version_nums: &[Option<i32>],
    ) -> RecordBatch {
        use arrow_array::builder::StringDictionaryBuilder;
        use arrow_schema::Field;

        let id_array = Int32Array::from(ids.to_vec());

        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        for name in names {
            match name {
                Some(v) => dict_builder.append_value(v),
                None => dict_builder.append_null(),
            }
        }
        let dict_array: ArrayRef = Arc::new(dict_builder.finish());

        let version_array: ArrayRef = Arc::new(Int32Array::from(version_nums.to_vec()));

        let struct_fields = Fields::from(vec![
            Field::new(
                "name",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("version_num", DataType::Int32, true),
        ]);

        let struct_array =
            StructArray::try_new(struct_fields, vec![dict_array, version_array], None)
                .expect("valid struct");

        RecordBatch::try_new(
            Arc::clone(schema),
            vec![Arc::new(id_array), Arc::new(struct_array)],
        )
        .expect("valid batch")
    }

    /// Reads dict string values from a dictionary child inside a struct column.
    fn read_struct_dict_strings(
        batch: &RecordBatch,
        struct_col_idx: usize,
        child_idx: usize,
    ) -> Vec<Option<String>> {
        let struct_arr = batch
            .column(struct_col_idx)
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("struct column");
        let child = struct_arr.column(child_idx);
        // Create a temporary single-column batch to reuse read_dict_strings.
        let dt = child.data_type();
        match dt {
            DataType::Dictionary(k, _) => match k.as_ref() {
                DataType::UInt8 => child
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                DataType::UInt16 => child
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .unwrap()
                    .downcast_dict::<StringArray>()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.map(String::from))
                    .collect(),
                other => panic!("unexpected dict key type in struct child: {other:?}"),
            },
            other => panic!("expected dictionary child, got {other:?}"),
        }
    }

    #[test]
    fn write_to_with_struct_containing_dict_varying_values() {
        // Core case: dictionary inside a Struct column with different values
        // across batches. Without struct-aware unification, FileWriter rejects
        // this with "dictionary replacement detected".
        let schema = struct_dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_struct_dict_batch(
            &schema,
            &[1, 2],
            &[Some("alpha"), Some("beta")],
            &[Some(10), Some(20)],
        );
        let batch2 = make_struct_dict_batch(
            &schema,
            &[3, 4],
            &[Some("gamma"), Some("delta")],
            &[Some(30), None],
        );

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.row_count, 4);
        assert_eq!(metadata.chunk_count, 2);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify dict values inside struct round-trip correctly.
        assert_eq!(
            read_struct_dict_strings(&batches[0], 1, 0),
            vec![Some("alpha".into()), Some("beta".into())]
        );
        assert_eq!(
            read_struct_dict_strings(&batches[1], 1, 0),
            vec![Some("gamma".into()), Some("delta".into())]
        );

        // Verify the non-dict struct child also round-trips.
        let struct0 = batches[0]
            .column(1)
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let versions0: Vec<Option<i32>> = struct0
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap()
            .iter()
            .collect();
        assert_eq!(versions0, vec![Some(10), Some(20)]);

        let struct1 = batches[1]
            .column(1)
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let versions1: Vec<Option<i32>> = struct1
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap()
            .iter()
            .collect();
        assert_eq!(versions1, vec![Some(30), None]);
    }

    #[test]
    fn write_to_with_struct_dict_null_keys() {
        // Null dictionary keys inside struct must survive unification.
        let schema = struct_dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 =
            make_struct_dict_batch(&schema, &[1, 2], &[Some("a"), None], &[Some(1), Some(2)]);
        let batch2 =
            make_struct_dict_batch(&schema, &[3, 4], &[None, Some("b")], &[Some(3), Some(4)]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        assert_eq!(
            read_struct_dict_strings(&batches[0], 1, 0),
            vec![Some("a".into()), None]
        );
        assert_eq!(
            read_struct_dict_strings(&batches[1], 1, 0),
            vec![None, Some("b".into())]
        );
    }

    #[test]
    fn write_to_with_struct_dict_single_batch_unchanged() {
        // A single batch with struct-dict skips unification; schema unchanged.
        let schema = struct_dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch = make_struct_dict_batch(
            &schema,
            &[1, 2],
            &[Some("x"), Some("y")],
            &[Some(10), Some(20)],
        );
        let _ = acc.append(batch).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        // Schema unchanged (still Dict(UInt8, Utf8) inside Struct).
        let struct_type = reader.schema().field(1).data_type().clone();
        if let DataType::Struct(fields) = &struct_type {
            assert_eq!(
                fields[0].data_type(),
                &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
            );
        } else {
            panic!("expected Struct type, got {struct_type:?}");
        }
    }

    #[test]
    fn write_to_with_struct_dict_widens_key_type() {
        // Total dict values across batches exceeds UInt8 capacity, forcing
        // widening of the dict key type inside the struct.
        let schema = struct_dict_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let labels1: Vec<String> = (0..200).map(|i| format!("name_a_{i}")).collect();
        let labels2: Vec<String> = (0..200).map(|i| format!("name_b_{i}")).collect();
        let opts1: Vec<Option<&str>> = labels1.iter().map(|s| Some(s.as_str())).collect();
        let opts2: Vec<Option<&str>> = labels2.iter().map(|s| Some(s.as_str())).collect();
        let ids1: Vec<i32> = (0..200).collect();
        let ids2: Vec<i32> = (200..400).collect();
        let vers1: Vec<Option<i32>> = (0..200).map(Some).collect();
        let vers2: Vec<Option<i32>> = (200..400).map(Some).collect();

        let batch1 = make_struct_dict_batch(&schema, &ids1, &opts1, &vers1);
        let batch2 = make_struct_dict_batch(&schema, &ids2, &opts2, &vers2);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // Dict key type inside struct should be widened to UInt16.
        let struct_type = reader.schema().field(1).data_type().clone();
        if let DataType::Struct(fields) = &struct_type {
            assert_eq!(
                fields[0].data_type(),
                &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                "dict key type inside struct should be widened to UInt16"
            );
        } else {
            panic!("expected Struct type, got {struct_type:?}");
        }

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Spot-check values.
        let vals0 = read_struct_dict_strings(&batches[0], 1, 0);
        assert_eq!(vals0[0].as_deref(), Some("name_a_0"));
        assert_eq!(vals0[199].as_deref(), Some("name_a_199"));

        let vals1 = read_struct_dict_strings(&batches[1], 1, 0);
        assert_eq!(vals1[0].as_deref(), Some("name_b_0"));
        assert_eq!(vals1[199].as_deref(), Some("name_b_199"));
    }

    #[test]
    fn write_to_with_struct_dict_overflow_falls_back_to_native_type() {
        // Dict values inside a struct exceed UInt16 capacity, triggering
        // native type fallback for the struct child. The struct child
        // should become plain Utf8 while other struct children remain
        // unchanged.
        let schema_u16 = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "scope",
                DataType::Struct(Fields::from(vec![
                    Field::new(
                        "name",
                        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                        true,
                    ),
                    Field::new("version_num", DataType::Int32, true),
                ])),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema_u16),
        );

        let count = 40_000usize;

        // Helper to build a struct batch with UInt16-keyed dict.
        let make_batch = |start: usize, prefix: &str| -> RecordBatch {
            let labels: Vec<String> = (0..count).map(|i| format!("{prefix}_{i}")).collect();
            let values = StringArray::from(labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            let keys = arrow_array::PrimitiveArray::<UInt16Type>::from(
                (0..count as u16).collect::<Vec<_>>(),
            );
            let dict_arr: ArrayRef = Arc::new(DictionaryArray::new(keys, Arc::new(values)));
            let ver_arr: ArrayRef = Arc::new(Int32Array::from(
                (start..start + count).map(|i| i as i32).collect::<Vec<_>>(),
            ));

            let struct_fields = Fields::from(vec![
                Field::new(
                    "name",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new("version_num", DataType::Int32, true),
            ]);
            let struct_arr =
                StructArray::try_new(struct_fields, vec![dict_arr, ver_arr], None).unwrap();

            RecordBatch::try_new(
                Arc::clone(&schema_u16),
                vec![
                    Arc::new(Int32Array::from(
                        (start..start + count).map(|i| i as i32).collect::<Vec<_>>(),
                    )),
                    Arc::new(struct_arr),
                ],
            )
            .unwrap()
        };

        let _ = acc.append(make_batch(0, "a")).unwrap();
        let _ = acc.append(make_batch(count, "b")).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.row_count, 2 * count as u64);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // The dict child should have fallen back to plain Utf8.
        let struct_type = reader.schema().field(1).data_type().clone();
        if let DataType::Struct(fields) = &struct_type {
            assert_eq!(
                fields[0].data_type(),
                &DataType::Utf8,
                "struct dict child should fall back to native Utf8"
            );
            // Non-dict child should be unchanged.
            assert_eq!(fields[1].data_type(), &DataType::Int32);
        } else {
            panic!("expected Struct type, got {struct_type:?}");
        }

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify values via plain StringArray (not dict).
        let struct0 = batches[0]
            .column(1)
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();
        let names0 = struct0
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("should be plain StringArray after fallback");
        assert_eq!(names0.value(0), "a_0");
        assert_eq!(names0.value(count - 1), &format!("a_{}", count - 1));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Non-Utf8 dictionary value type tests
    //
    // The unification logic is value-type-agnostic (uses arrow::concat),
    // but these tests verify that Dict with Int32, Binary, and
    // FixedSizeBinary values all round-trip correctly through the
    // multi-batch unification path. These cover OTAP fields like
    // severity_number (Dict<UInt8,Int32>), trace_id (Dict<UInt8,FSB(16)>),
    // and bytes/ser (Dict<UInt16,Binary>).
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn write_to_with_dict_int32_values() {
        // Exercises Dict(UInt8, Int32) — used by OTAP severity_number,
        // kind, status.code.
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "code",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        // Build batches using UInt8-keyed Int32 dictionaries.
        let vals1 = Int32Array::from(vec![200, 404]);
        let keys1 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let codes1 = DictionaryArray::new(keys1, Arc::new(vals1));
        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![1, 2])), Arc::new(codes1)],
        )
        .unwrap();

        let vals2 = Int32Array::from(vec![500, 0]); // 0 is placeholder; key is null
        let keys2 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), None]);
        let codes2 = DictionaryArray::new(keys2, Arc::new(vals2));
        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![3, 4])), Arc::new(codes2)],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Read back Int32 dict values.
        let read_int32_dict = |batch: &RecordBatch, col: usize| -> Vec<Option<i32>> {
            let arr = batch.column(col);
            // Key type may have been widened, but UInt8 fits here.
            arr.as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                .unwrap()
                .downcast_dict::<Int32Array>()
                .unwrap()
                .into_iter()
                .collect()
        };

        assert_eq!(read_int32_dict(&batches[0], 1), vec![Some(200), Some(404)]);
        assert_eq!(read_int32_dict(&batches[1], 1), vec![Some(500), None]);
    }

    #[test]
    fn write_to_with_dict_fixed_size_binary_values() {
        // Exercises Dict(UInt8, FixedSizeBinary(16)) — used by OTAP
        // trace_id. This is the field that originally triggered the
        // dictionary replacement bug.
        use arrow_array::FixedSizeBinaryArray;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "trace_id",
                DataType::Dictionary(
                    Box::new(DataType::UInt8),
                    Box::new(DataType::FixedSizeBinary(16)),
                ),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        // Two batches with different 16-byte trace IDs.
        let trace1a = [1u8; 16];
        let trace1b = [2u8; 16];
        let values1 = FixedSizeBinaryArray::try_from_iter(
            vec![trace1a.as_ref(), trace1b.as_ref()].into_iter(),
        )
        .unwrap();
        let keys1 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let dict1 = DictionaryArray::new(keys1, Arc::new(values1));

        let trace2a = [3u8; 16];
        let trace2b = [4u8; 16];
        let values2 = FixedSizeBinaryArray::try_from_iter(
            vec![trace2a.as_ref(), trace2b.as_ref()].into_iter(),
        )
        .unwrap();
        let keys2 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let dict2 = DictionaryArray::new(keys2, Arc::new(values2));

        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![1, 2])), Arc::new(dict1)],
        )
        .unwrap();
        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![3, 4])), Arc::new(dict2)],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Read back FSB(16) dict values as raw bytes.
        let read_fsb_dict = |batch: &RecordBatch, col: usize| -> Vec<Vec<u8>> {
            let arr = batch.column(col);
            let dict = arr
                .as_any()
                .downcast_ref::<DictionaryArray<UInt8Type>>()
                .unwrap();
            let values = dict
                .values()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            dict.keys()
                .iter()
                .map(|k| values.value(k.unwrap() as usize).to_vec())
                .collect()
        };

        assert_eq!(
            read_fsb_dict(&batches[0], 1),
            vec![trace1a.to_vec(), trace1b.to_vec()]
        );
        assert_eq!(
            read_fsb_dict(&batches[1], 1),
            vec![trace2a.to_vec(), trace2b.to_vec()]
        );
    }

    #[test]
    fn write_to_with_dict_binary_values() {
        // Exercises Dict(UInt16, Binary) — used by OTAP bytes/ser
        // attribute columns.
        use arrow_array::BinaryArray;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "payload",
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let values1 = BinaryArray::from(vec![b"hello".as_ref(), b"world".as_ref()]);
        let keys1 = arrow_array::PrimitiveArray::<UInt16Type>::from(vec![Some(0u16), Some(1)]);
        let dict1 = DictionaryArray::new(keys1, Arc::new(values1));

        let values2 = BinaryArray::from(vec![b"foo".as_ref(), b"bar".as_ref()]);
        let keys2 = arrow_array::PrimitiveArray::<UInt16Type>::from(vec![Some(0u16), Some(1)]);
        let dict2 = DictionaryArray::new(keys2, Arc::new(values2));

        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![1, 2])), Arc::new(dict1)],
        )
        .unwrap();
        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![3, 4])), Arc::new(dict2)],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let _ = acc.write_to(&mut buffer, 0).unwrap();

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Read back Binary dict values.
        let read_binary_dict = |batch: &RecordBatch, col: usize| -> Vec<Vec<u8>> {
            let arr = batch.column(col);
            let dict = arr
                .as_any()
                .downcast_ref::<DictionaryArray<UInt16Type>>()
                .unwrap();
            let values = dict
                .values()
                .as_any()
                .downcast_ref::<BinaryArray>()
                .unwrap();
            dict.keys()
                .iter()
                .map(|k| values.value(k.unwrap() as usize).to_vec())
                .collect()
        };

        assert_eq!(
            read_binary_dict(&batches[0], 1),
            vec![b"hello".to_vec(), b"world".to_vec()]
        );
        assert_eq!(
            read_binary_dict(&batches[1], 1),
            vec![b"foo".to_vec(), b"bar".to_vec()]
        );
    }

    #[test]
    fn write_to_with_struct_containing_non_utf8_dict() {
        // Exercises OTAP's body struct pattern: a Struct column with
        // Dict(UInt16, Int64) and Dict(UInt16, Binary) children alongside
        // a plain primitive child. This is the most complex OTAP nesting
        // pattern (body.int + body.bytes + body.type).
        use arrow_array::BinaryArray;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new(
                "body",
                DataType::Struct(Fields::from(vec![
                    Field::new("type", DataType::UInt8, false),
                    Field::new(
                        "int",
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int64)),
                        true,
                    ),
                    Field::new(
                        "bytes",
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Binary)),
                        true,
                    ),
                ])),
                true,
            ),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        // Build batch 1.
        let body_type1: ArrayRef = Arc::new(arrow_array::UInt8Array::from(vec![1u8, 2]));
        let int_vals1 = arrow_array::Int64Array::from(vec![100i64, 200]);
        let int_keys1 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let int_dict1 = DictionaryArray::new(int_keys1, Arc::new(int_vals1));
        let bytes_vals1 = BinaryArray::from(vec![b"abc".as_ref(), b"def".as_ref()]);
        let bytes_keys1 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let bytes_dict1 = DictionaryArray::new(bytes_keys1, Arc::new(bytes_vals1));

        let struct_fields = Fields::from(vec![
            Field::new("type", DataType::UInt8, false),
            Field::new(
                "int",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int64)),
                true,
            ),
            Field::new(
                "bytes",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Binary)),
                true,
            ),
        ]);

        let struct1 = StructArray::try_new(
            struct_fields.clone(),
            vec![body_type1, Arc::new(int_dict1), Arc::new(bytes_dict1)],
            None,
        )
        .unwrap();

        let batch1 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![1, 2])), Arc::new(struct1)],
        )
        .unwrap();

        // Build batch 2 with different dict values.
        let body_type2: ArrayRef = Arc::new(arrow_array::UInt8Array::from(vec![3u8, 4]));
        let int_vals2 = arrow_array::Int64Array::from(vec![300i64, 400]);
        let int_keys2 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let int_dict2 = DictionaryArray::new(int_keys2, Arc::new(int_vals2));
        let bytes_vals2 = BinaryArray::from(vec![b"ghi".as_ref(), b"jkl".as_ref()]);
        let bytes_keys2 = arrow_array::PrimitiveArray::<UInt8Type>::from(vec![Some(0u8), Some(1)]);
        let bytes_dict2 = DictionaryArray::new(bytes_keys2, Arc::new(bytes_vals2));

        let struct2 = StructArray::try_new(
            struct_fields,
            vec![body_type2, Arc::new(int_dict2), Arc::new(bytes_dict2)],
            None,
        )
        .unwrap();

        let batch2 = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(vec![3, 4])), Arc::new(struct2)],
        )
        .unwrap();

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();
        assert_eq!(metadata.row_count, 4);
        assert_eq!(metadata.chunk_count, 2);

        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify Int64 dict values inside struct.
        let read_struct_int64_dict =
            |batch: &RecordBatch, struct_col: usize, child_idx: usize| -> Vec<Option<i64>> {
                let sa = batch
                    .column(struct_col)
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .unwrap();
                let child = sa.column(child_idx);
                child
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .unwrap()
                    .downcast_dict::<arrow_array::Int64Array>()
                    .unwrap()
                    .into_iter()
                    .collect()
            };

        assert_eq!(
            read_struct_int64_dict(&batches[0], 1, 1),
            vec![Some(100), Some(200)]
        );
        assert_eq!(
            read_struct_int64_dict(&batches[1], 1, 1),
            vec![Some(300), Some(400)]
        );

        // Verify Binary dict values inside struct.
        let read_struct_binary_dict =
            |batch: &RecordBatch, struct_col: usize, child_idx: usize| -> Vec<Vec<u8>> {
                let sa = batch
                    .column(struct_col)
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .unwrap();
                let child = sa.column(child_idx);
                let dict = child
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .unwrap();
                let values = dict
                    .values()
                    .as_any()
                    .downcast_ref::<BinaryArray>()
                    .unwrap();
                dict.keys()
                    .iter()
                    .map(|k| values.value(k.unwrap() as usize).to_vec())
                    .collect()
            };

        assert_eq!(
            read_struct_binary_dict(&batches[0], 1, 2),
            vec![b"abc".to_vec(), b"def".to_vec()]
        );
        assert_eq!(
            read_struct_binary_dict(&batches[1], 1, 2),
            vec![b"ghi".to_vec(), b"jkl".to_vec()]
        );

        // Verify the non-dict struct child (UInt8 type field) passes through.
        let read_struct_u8 =
            |batch: &RecordBatch, struct_col: usize, child_idx: usize| -> Vec<u8> {
                let sa = batch
                    .column(struct_col)
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .unwrap();
                sa.column(child_idx)
                    .as_any()
                    .downcast_ref::<arrow_array::UInt8Array>()
                    .unwrap()
                    .values()
                    .to_vec()
            };

        assert_eq!(read_struct_u8(&batches[0], 1, 0), vec![1, 2]);
        assert_eq!(read_struct_u8(&batches[1], 1, 0), vec![3, 4]);
    }
}
