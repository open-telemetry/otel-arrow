// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Concatenation of multiple OTAP batches into one.
//!
//! Reindexing, casting to a unified schema, and concatenation are fused so
//! that every output column is written exactly once, directly from the input
//! arrays.
//!
//! # Algorithm
//!
//! - **P0** If there are 0 or 1 inputs, return them as-is.
//! - **P1** If reindexing, remove transport optimized encodings from every
//!   input (see [`reindex`]).
//! - **P2** For each payload type, index the fields present across inputs
//!   and select a unified schema (`index_records`, `select_schema`). This
//!   includes dictionary key width selection from the summed physical value
//!   counts, and nullability.
//! - **P3** If reindexing, plan ID rewrites without modifying inputs
//!   (`reindex::plan_ids`). This produces a row `Selection` and an
//!   `IdRemap` per ID column for every input.
//! - **P4** Compute the output row count per payload from the selections.
//! - **P5** For each target field, allocate a full-size destination and copy
//!   every selected range of every input into it, casting between native and
//!   dictionary encodings and applying ID remaps on the fly (see
//!   `write_column`).
//!
//! Only P5 writes column data (P1 only touches encoded columns, and P3 only
//! allocates scratch space for compacted ID columns).
//!
//! # Plan types
//!
//! The planner inspects the input batches without modifying them and
//! produces, per input and per payload, an `InputPlan`: a `Selection` of
//! rows that survive into the output plus an `IdRemap` for each ID column.
//! The column writers apply the selection and remaps on the fly.
//!
//! # Column writers
//!
//! Each output column is written exactly once: a destination buffer is sized
//! for the full output and every selected range of every input is copied (and,
//! when needed, cast and/or ID-remapped) directly into it. There is no
//! intermediate per-input converted array and no final coalescing copy.
//!
//! - `ValueBuilder` implementations know how to append values of one
//!   physical type (primitive, boolean, bytes, fixed size binary) from a
//!   source array, either by contiguous range or by gathering through
//!   dictionary keys.
//! - `write_native` drives a `ValueBuilder` to produce a plain (non
//!   dictionary) output column from native, dictionary, or missing inputs.
//! - `write_dict` drives a `ValueBuilder` for the dictionary values and
//!   builds output keys directly. Dictionary inputs append their whole values
//!   array and shift their keys; native inputs append their selected values
//!   with sequential keys. No hashing is performed.
//! - `write_struct` recurses into struct children.
//! - `write_fallback` handles anything else (currently only List columns)
//!   via `MutableArrayData`.
//!
//! Every writer honors array offsets (sliced inputs) for both values and
//! nulls, and null buffers are only materialized if a null is encountered.
//!
//! # TODO
//!
//! - TODO(dict-policy): Tune when dictionary vs. native output is selected and
//!   the key width, based on the least work, and possibly make it
//!   configurable. Today a column is a dictionary if any input is, when the
//!   summed physical value count fits.
//! - TODO(dict-dedupe): `write_dict` appends the entire values array of
//!   every dictionary input. Deduplicate values arrays shared between inputs
//!   (pointer identity, e.g. slices of the same split batch) and trim values
//!   not referenced by the selected keys.
//! - TODO(list-writer): List columns (metrics quantiles, histogram buckets)
//!   use the generic `MutableArrayData` fallback (`write_fallback`). Add a
//!   specialized writer that copies offsets and child values directly.
//! - TODO(bytes-gather-capacity): Byte capacity for dictionary inputs is sized
//!   from the whole values array, which over-allocates when the column is
//!   gathered into a native output. Size it from the selected keys instead.
//! - TODO(fused-decode): Fuse transport delta decoding into the ID statistics
//!   and write passes instead of decoding in a separate pre-pass.
//! - TODO(single-input): Pass a payload through unchanged (Arc reuse) when
//!   only one input contributes and its plan is identity.

use ahash::AHashSet;
use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, AsArray, DictionaryArray, OffsetSizeTrait, RecordBatch,
    StructArray,
};
use arrow::datatypes::{
    ArrowNativeType, DurationMicrosecondType, DurationMillisecondType, DurationNanosecondType,
    DurationSecondType, Float64Type, GenericBinaryType, Int64Type, TimestampMicrosecondType,
    TimestampMillisecondType, TimestampNanosecondType, TimestampSecondType, UInt8Type, UInt16Type,
    UInt64Type,
};
use arrow_schema::{DataType, Field, FieldRef, Schema, SchemaBuilder};
use itertools::Either;
use roaring::RoaringBitmap;
use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    ArrayData, ArrowNativeTypeOp, BooleanArray, BooleanBufferBuilder, FixedSizeBinaryArray,
    GenericByteArray, MutableArrayData, PrimitiveArray, make_array,
};
use arrow::buffer::{Buffer, NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow::compute::kernels::cast;
use arrow::datatypes::{
    ArrowDictionaryKeyType, BinaryType, ByteArrayType, Int32Type, UInt32Type, Utf8Type,
};
use arrow_schema::{ArrowError, Fields};

use crate::otap::transform::reindex;

use crate::error::Error;
use crate::otap::{Logs, Metrics, OtapBatchStore, Result, Traces};
use crate::schema::consts::metadata::COLUMN_ENCODING;
use crate::schema::consts::metadata::encodings::PLAIN;
use crate::schema::consts::{ID, PARENT_ID, RESOURCE, SCOPE};
use crate::schema::payloads;
use crate::schema::schema::{DictKeySize, Field as SchemaField, Schema as PayloadSchema};

/// These are one less than the maximum cardinality of the key type. We should be
/// able to go up to 256/65536 without overflow, but there is a bug in arrow-rs
/// for some value types.
///
/// See:
///     - https://github.com/apache/arrow-rs/issues/9366
///     - github.com/open-telemetry/otel-arrow/issues/1971
const MAX_U8_CARDINALITY: usize = 255;
const MAX_U16_CARDINALITY: usize = 65535;

/// Options controlling [concatenate].
#[derive(Debug, Clone, Copy, Default)]
pub struct ConcatOptions {
    /// Rewrite ID / PARENT_ID columns so that IDs from different inputs do not
    /// collide in the output. This also removes transport optimized encodings.
    ///
    /// Callers that concatenate disjoint pieces of the same original batch
    /// (whose IDs are already unique across pieces) may disable this.
    pub reindex: bool,
}

impl ConcatOptions {
    /// Options for concatenating unrelated batches: reindex enabled.
    #[must_use]
    pub const fn reindex() -> Self {
        Self { reindex: true }
    }

    /// Options for concatenating pieces with already-disjoint IDs.
    #[must_use]
    pub const fn preserve_ids() -> Self {
        Self { reindex: false }
    }
}

/// Concatenate the provided OTAP batches into a single batch.
///
/// See the module documentation for the algorithm. When `opts.reindex` is
/// set, transport optimized encodings are removed and ID columns are
/// rewritten so that IDs from different inputs cannot collide. Otherwise ID
/// columns are copied as-is, which is only correct when the inputs are
/// disjoint pieces of the same original batch.
///
/// The inputs are consumed: every slot in `items` is `None` on success.
///
/// # Errors
///
/// Returns [`Error::UnsupportedBatchStoreType`] if `N` is not the batch width
/// of a known signal (`Logs`, `Metrics`, or `Traces`), regardless of how many
/// batches are passed. The signal is selected by width alone, so a future
/// batch store that reuses an existing signal's width would be routed to that
/// signal's payload schemas.
pub fn concatenate<const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
    opts: ConcatOptions,
) -> Result<[Option<RecordBatch>; N]> {
    // Resolve the signal up front so an unsupported width is rejected even on
    // the empty and single-batch fast paths below.
    type ConcatSignal<const N: usize> =
        fn(&mut [[Option<RecordBatch>; N]], ConcatOptions) -> Result<[Option<RecordBatch>; N]>;
    let concat_signal: ConcatSignal<N> = match N {
        Logs::COUNT => concatenate_signal::<Logs, N>,
        Metrics::COUNT => concatenate_signal::<Metrics, N>,
        Traces::COUNT => concatenate_signal::<Traces, N>,
        _ => return Err(Error::UnsupportedBatchStoreType { batch_width: N }),
    };

    let mut result = [const { None }; N];
    if items.is_empty() {
        return Ok(result);
    }

    if items.len() == 1 {
        for (i, item) in result.iter_mut().enumerate().take(N) {
            *item = items[0][i].take();
        }
        return Ok(result);
    }

    concat_signal(items, opts)
}

fn concatenate_signal<S: OtapBatchStore, const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
    opts: ConcatOptions,
) -> Result<[Option<RecordBatch>; N]> {
    let mut result = [const { None }; N];

    // P1 + P3: decode transport encodings and plan the ID rewrites. Nothing
    // is copied here apart from decoding encoded columns and the scratch
    // values of compacted ID columns.
    let mut id_plan = if opts.reindex {
        reindex::remove_transport_encodings::<S, N>(items)?;
        Some(reindex::plan_ids::<S, N>(items)?)
    } else {
        None
    };

    #[allow(clippy::needless_range_loop)]
    for i in 0..N {
        let payload_def = payloads::get(S::payload_type_at_idx(i));

        // P2: index fields and select the unified schema.
        let index = index_records(select_all(items, i), payload_def)?;
        if index.batch_count == 0 {
            continue;
        }
        let selected = select_schema(&index)?;

        // P5: write every output column once.
        let mut batches: Vec<&RecordBatch> = Vec::with_capacity(index.batch_count);
        let mut plans: Vec<InputPlan> = Vec::with_capacity(index.batch_count);
        for (j, group) in items.iter().enumerate() {
            if let Some(rb) = group[i].as_ref() {
                batches.push(rb);
                plans.push(match id_plan.as_mut() {
                    Some(p) => std::mem::take(&mut p[i][j]),
                    None => InputPlan::default(),
                });
            }
        }
        result[i] = Some(write_payload(&batches, &plans, payload_def, selected)?);

        for payload in select_all_mut(items, i) {
            *payload = None;
        }
    }

    Ok(result)
}

/// Test helper: apply the reindex plan to every input independently, without
/// concatenating. This lets tests verify the planner using per-input
/// assertions (no overlaps across inputs, preserved relations).
#[cfg(test)]
pub(crate) fn reindex_in_place<S: OtapBatchStore, const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
) -> Result<()> {
    reindex::remove_transport_encodings::<S, N>(items)?;
    let mut id_plan = reindex::plan_ids::<S, N>(items)?;
    for (j, group) in items.iter_mut().enumerate() {
        for i in 0..N {
            let Some(rb) = group[i].as_ref() else {
                continue;
            };
            let payload_def = payloads::get(S::payload_type_at_idx(i));
            let index = index_records(std::iter::once(Some(rb)), payload_def)?;
            let selected = select_schema(&index)?;
            let plan = std::mem::take(&mut id_plan[i][j]);
            let out = write_payload(&[rb], &[plan], payload_def, selected)?;
            group[i] = Some(out);
        }
    }
    Ok(())
}

/// Write the output record batch for one payload type.
///
/// `batches[j]` is the j-th contributing input and `plans[j]` its row
/// selection and ID remaps. Every output column is written exactly once.
fn write_payload(
    batches: &[&RecordBatch],
    plans: &[InputPlan],
    payload_def: &PayloadSchema,
    selected: SelectedSchema,
) -> Result<RecordBatch> {
    debug_assert_eq!(batches.len(), plans.len());
    let schema = Arc::new(selected.schema);
    let target_fields = schema.fields();

    // Resolve, for every input, the source column for each target position.
    // This is O(fields) per input and avoids name lookups per column.
    let mut sources: Vec<Vec<Option<&ArrayRef>>> = Vec::with_capacity(batches.len());
    for rb in batches {
        let mut cols = vec![None; target_fields.len()];
        for (field, col) in rb.schema_ref().fields().iter().zip(rb.columns()) {
            let slot =
                payload_def
                    .slot_of(field.name())
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: DataType::Null,
                        actual: field.data_type().clone(),
                    })?;
            let target_idx = selected.slot_to_target[slot];
            debug_assert!(target_idx >= 0, "indexed field must have a target position");
            cols[target_idx as usize] = Some(col);
        }
        sources.push(cols);
    }

    let rows: usize = batches
        .iter()
        .zip(plans)
        .map(|(rb, plan)| plan.selection.count(rb.num_rows()))
        .sum();

    let mut columns = Vec::with_capacity(target_fields.len());
    let mut inputs: Vec<Input<'_>> = Vec::with_capacity(batches.len());
    for (target_idx, target) in target_fields.iter().enumerate() {
        inputs.clear();
        inputs.extend(
            batches
                .iter()
                .zip(plans)
                .zip(&sources)
                .map(|((rb, plan), cols)| Input {
                    column: cols[target_idx],
                    num_rows: rb.num_rows(),
                    plan,
                }),
        );
        let id_col = top_level_id_col(target.name());
        columns.push(write_column(target, &inputs, rows, id_col)?);
    }

    let options = arrow::array::RecordBatchOptions::new().with_row_count(Some(rows));
    RecordBatch::try_new_with_options(schema, columns, &options)
        .map_err(|source| Error::Batching { source })
}

/// The output of schema selection: the unified schema plus a map from each
/// payload spec slot to its index in that schema (or -1 if the field was absent
/// from every input batch).
struct SelectedSchema {
    schema: Schema,
    slot_to_target: [i16; MAX_SLOTS],
}

/// Select a unified schema in payload-spec declaration order, emitting only the
/// fields that were present in at least one input batch.
fn select_schema<'a>(index: &'a RecordIndex<'a>) -> Result<SelectedSchema> {
    let payload_def = index.fields.schema;
    let mut builder = SchemaBuilder::with_capacity(payload_def.fields().len());
    let mut slot_to_target = [-1i16; MAX_SLOTS];
    let mut next_target: i16 = 0;

    for (slot, def_field) in payload_def.fields().iter().enumerate() {
        let Some(info) = index.fields.slots[slot].as_ref() else {
            continue;
        };

        let typ = select_field_type(info, Some(def_field))?;
        let mut new_field = Field::new(def_field.name, typ, info.nullable);
        add_field_metadata(&mut new_field);
        slot_to_target[slot] = next_target;
        next_target += 1;
        builder.push(new_field);
    }

    Ok(SelectedSchema {
        schema: builder.finish(),
        slot_to_target,
    })
}

/// Select the final Arrow data type for an indexed field, resolving struct
/// children and dictionary key widths.
fn select_field_type(info: &IndexedField<'_>, field_def: Option<&SchemaField>) -> Result<DataType> {
    if let Some(struct_index) = info.struct_index.as_ref() {
        return select_struct_type(struct_index);
    }

    if info.is_dictionary {
        return select_dictionary_type(info, field_def);
    }

    Ok(info.value_type.clone())
}

/// Select the final data type for a struct field, in sub-schema declaration
/// order over the present children.
fn select_struct_type(struct_index: &FieldIndex<'_>) -> Result<DataType> {
    let sub_def = struct_index.schema;
    let mut fields = Vec::new();
    for (slot, def_field) in sub_def.fields().iter().enumerate() {
        let Some(info) = struct_index.slots[slot].as_ref() else {
            continue;
        };

        // Nested structs are rejected during indexing.
        debug_assert!(!matches!(info.value_type, DataType::Struct(_)));

        let typ = select_field_type(info, Some(def_field))?;
        let mut new_field = Field::new(def_field.name, typ, info.nullable);
        add_field_metadata(&mut new_field);
        fields.push(new_field);
    }

    Ok(DataType::Struct(fields.into()))
}

/// Add required metadata to the field. Currently we're requiring callers to
/// strip the transport optimized encoding before calling [concatenate], so
/// we need to mark ID fields as plain.
fn add_field_metadata(field: &mut Field) {
    match field.name().as_str() {
        ID | PARENT_ID => {
            _ = field
                .metadata_mut()
                .insert(COLUMN_ENCODING.to_string(), PLAIN.to_string());
        }
        _ => {}
    }
}

/// Upper bound on the number of fields in any OTAP payload schema, including
/// nested struct schemas. `FieldIndex` stores one slot per spec field in a
/// fixed-size array sized by this bound.
///
/// The widest schema is `spans`, with 17 top-level fields (`status` is slot 16),
/// followed by `logs` and `exp_histogram_data_points` with 14 each. The widest
/// struct child is the logs `body`, with 7. The `spec_index_tests` tests check
/// that every payload schema fits and that this bound stays tight.
pub(crate) const MAX_SLOTS: usize = 17;

#[derive(Debug)]
struct RecordIndex<'a> {
    batch_count: usize,
    row_count: usize,
    fields: FieldIndex<'a>,
}

/// A spec-indexed set of fields. Each slot corresponds positionally to a field
/// in `schema.fields()`; `None` means the field was absent from every batch.
#[derive(Debug)]
struct FieldIndex<'a> {
    schema: &'static PayloadSchema,
    slots: [Option<IndexedField<'a>>; MAX_SLOTS],
}

impl<'a> FieldIndex<'a> {
    fn new(schema: &'static PayloadSchema) -> Self {
        Self {
            schema,
            slots: [const { None }; MAX_SLOTS],
        }
    }
}

/// Accumulated information about a single field across input batches, used to
/// select the unified output type. Unlike the query-engine cardinality
/// estimator this deliberately does not retain the value arrays: dictionary
/// widths are chosen from `total_physical_value_count` and nullability from
/// `present_count`.
#[derive(Debug)]
struct IndexedField<'a> {
    // The value type of the column: a primitive/binary type, or a struct type.
    // Never a dictionary; dictionary columns store their value type here and set
    // `is_dictionary`.
    value_type: &'a DataType,
    // True if any batch was null in this column, or the column was absent from
    // some batch (determined in the finalize pass).
    nullable: bool,
    // True if any batch carried this column as a dictionary.
    is_dictionary: bool,
    // The number of batches that carried this column.
    present_count: usize,
    // The total number of physical values (including nulls) contributed across
    // all dictionary batches. Bounds the number of dictionary entries that Arrow
    // may append while coalescing, and hence the required key width.
    total_physical_value_count: usize,
    // For struct columns, the recursively-indexed children.
    struct_index: Option<Box<FieldIndex<'a>>>,
}

/// Create an index of fields, validating each against the payload spec and
/// computing the statistics needed to select the unified schema.
fn index_records<'a>(
    batches: impl Iterator<Item = Option<&'a RecordBatch>>,
    payload_def: &'static PayloadSchema,
) -> Result<RecordIndex<'a>> {
    let mut index = RecordIndex {
        batch_count: 0,
        row_count: 0,
        fields: FieldIndex::new(payload_def),
    };

    for rb in batches {
        let Some(rb) = rb else {
            continue;
        };

        index.batch_count += 1;
        index.row_count += rb.num_rows();

        let fields = rb.schema_ref().fields();
        let iter = fields.iter().zip(rb.columns());
        index_fields(&mut index.fields, iter)?;
    }

    // Finalize nullability: a field is nullable if it was null in any batch or
    // was absent from some batch. Struct children additionally inherit the
    // parent's nullability and are nullable if absent from some batch that
    // carried the parent.
    finalize_nullability(&mut index.fields, index.batch_count);

    Ok(index)
}

fn finalize_nullability(index: &mut FieldIndex<'_>, batch_count: usize) {
    for slot in index.slots.iter_mut() {
        let Some(field) = slot.as_mut() else {
            continue;
        };
        field.nullable = field.nullable || field.present_count != batch_count;

        if let Some(struct_index) = field.struct_index.as_mut() {
            let parent_present = field.present_count;
            let parent_nullable = field.nullable;
            for child_slot in struct_index.slots.iter_mut() {
                if let Some(child) = child_slot.as_mut() {
                    child.nullable =
                        child.nullable || parent_nullable || child.present_count != parent_present;
                }
            }
        }
    }
}

/// Index the fields of a single batch (or struct) into `index`.
///
/// Struct columns are unified recursively: different batches may carry different
/// subsets of the optional struct children, so each child is indexed into its
/// own sub-slot and the unified struct type is the union of the children seen
/// across batches. A struct child whose scalar type diverges between batches is
/// still rejected (at child granularity), as is a struct-vs-non-struct collision
/// on the same column.
fn index_fields<'a>(
    index: &mut FieldIndex<'a>,
    fields: impl Iterator<Item = (&'a FieldRef, &'a ArrayRef)>,
) -> Result<()> {
    let schema = index.schema;

    for (field, data) in fields {
        let name = field.name().as_str();
        let slot = schema
            .slot_of(name)
            .ok_or_else(|| Error::ColumnDataTypeMismatch {
                name: field.name().clone(),
                // No spec entry for this column name.
                expect: DataType::Null,
                actual: field.data_type().clone(),
            })?;

        let (array, value_type, is_dict) = match data.data_type() {
            DataType::Dictionary(_, v) => (get_dictionary_values(data)?, v.as_ref(), true),
            x => (data, x, false),
        };

        if index.slots[slot].is_none() {
            let struct_index = if matches!(value_type, DataType::Struct(_)) {
                let sub_def = schema.fields()[slot]
                    .data_type
                    .as_struct_schema()
                    .expect("spec struct field has a struct sub-schema");

                // safety: value_type is Struct
                let struct_array = data
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .expect("Struct array");

                let mut sub = FieldIndex::new(sub_def);
                let iter = struct_array.fields().iter().zip(struct_array.columns());
                index_fields(&mut sub, iter)?;
                Some(Box::new(sub))
            } else {
                None
            };

            index.slots[slot] = Some(IndexedField {
                value_type,
                nullable: data.null_count() > 0,
                is_dictionary: is_dict,
                present_count: 1,
                total_physical_value_count: array.len(),
                struct_index,
            });
            continue;
        }

        let existing = index.slots[slot].as_mut().expect("slot occupied");
        if let Some(struct_index) = existing.struct_index.as_mut() {
            // For structs we only check if the target is a struct and validate
            // each column when we recurse.
            if !matches!(value_type, DataType::Struct(_)) {
                return Err(Error::ColumnDataTypeMismatch {
                    name: field.name().clone(),
                    expect: existing.value_type.clone(),
                    actual: value_type.clone(),
                });
            }

            // safety: value_type is Struct (checked above)
            let struct_array = data
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Struct array");
            let iter = struct_array.fields().iter().zip(struct_array.columns());
            index_fields(struct_index, iter)?;
        } else {
            if existing.value_type != value_type {
                return Err(Error::ColumnDataTypeMismatch {
                    name: field.name().clone(),
                    expect: existing.value_type.clone(),
                    actual: value_type.clone(),
                });
            }

            existing.is_dictionary = existing.is_dictionary || is_dict;
        }

        existing.nullable = existing.nullable || data.null_count() > 0;
        existing.present_count += 1;
        existing.total_physical_value_count += array.len();
    }

    Ok(())
}

fn get_dictionary_values(array: &ArrayRef) -> Result<&ArrayRef> {
    let values = match array.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => {
                // safety: we checked the type
                let dict_col = array
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("we just checked the key type");
                dict_col.values()
            }
            DataType::UInt16 => {
                // safety: we checked the type
                let dict_col = array
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .expect("we just checked the key type");
                dict_col.values()
            }
            _ => {
                return Err(Error::UnsupportedDictionaryKeyType {
                    expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                    actual: *k.clone(),
                });
            }
        },
        _ => unreachable!(),
    };

    Ok(values)
}

fn select_dictionary_type(
    info: &IndexedField<'_>,
    field_def: Option<&SchemaField>,
) -> Result<DataType> {
    debug_assert!(info.is_dictionary);

    // If the column is in the definition but doesn't support dictionary
    // encoding, use the native type. If not in the definition at all (None),
    // fall through to physical-count-based selection.
    let min_key_size = match field_def {
        Some(def) => match def.data_type.min_dict_key_size() {
            Some(min) => Some(min),
            // Column is explicitly defined as not dictionary-encodable
            None => return Ok(info.value_type.clone()),
        },
        // Column not in definition, select from physical count with no minimum.
        None => None,
    };

    // Arrow does not deduplicate all dictionary value types while coalescing,
    // and its merge path does not guarantee unique output values. The summed
    // physical values length is therefore the safe upper bound for both paths.
    let total = info.total_physical_value_count;
    let (mut dict_key_size, within_u8) = if total <= MAX_U8_CARDINALITY {
        (DataType::UInt8, true)
    } else if total <= MAX_U16_CARDINALITY {
        (DataType::UInt16, false)
    } else {
        return Ok(info.value_type.clone());
    };

    // Upgrade key size if the spec requires a minimum of u16.
    if min_key_size == Some(DictKeySize::U16) && within_u8 {
        dict_key_size = DataType::UInt16;
    }

    Ok(DataType::Dictionary(
        Box::new(dict_key_size),
        Box::new(info.value_type.clone()),
    ))
}

/// Accumulated information about a single field across one or more arrays, used
/// as input to [`estimate_cardinality`].
#[derive(Debug)]
pub struct FieldInfo<'a> {
    // The value type of the column. This must be some primitive type; the
    // estimator does not support struct or dictionary value types directly.
    value_type: &'a DataType,
    // Set if this originated from a dictionary column. The query engine always
    // constructs from a plain array so this is currently always `None`, but it
    // is retained because `check_cardinality` uses it to decide when an early
    // exit is sound.
    smallest_key_type: Option<DataType>,
    // The total number of values, excluding nulls
    total_value_count: usize,
    // The values arrays for the type, some of these may come from dictionary array values.
    values: Vec<ArrayRef>,
}

impl<'a> FieldInfo<'a> {
    /// Construct a [`FieldInfo`] describing a single array.
    #[must_use]
    pub fn new_from_array(array: &'a ArrayRef) -> Self {
        Self {
            value_type: array.data_type(),
            smallest_key_type: None,
            total_value_count: array.len() - array.null_count(),
            values: vec![Arc::clone(array)],
        }
    }
}

/// Estimate of the cardinality of a field
#[derive(PartialEq)]
pub enum Cardinality {
    WithinU8,
    WithinU16,
    GreaterThanU16,
}

impl Cardinality {
    const fn from_exact(count: usize) -> Cardinality {
        match count {
            count if count <= MAX_U8_CARDINALITY => Cardinality::WithinU8,
            count if count <= MAX_U16_CARDINALITY => Cardinality::WithinU16,
            _ => Cardinality::GreaterThanU16,
        }
    }
}

/// Estimate the cardinality of a set of arrays
#[must_use]
pub fn estimate_cardinality<'a>(info: &FieldInfo<'a>) -> Cardinality {
    // Small types
    match info.value_type.primitive_width() {
        Some(1) => return estimate_cardinality_small_type::<u8>(info),
        Some(2) => return estimate_cardinality_small_type::<u16>(info),
        Some(4) => return estimate_cardinality_small_type::<u32>(info),
        _ => {}
    };

    // Large types
    match info.value_type {
        DataType::UInt64 => estimate_cardinality_primitive_type::<UInt64Type, 8>(info),
        DataType::Int64 => estimate_cardinality_primitive_type::<Int64Type, 8>(info),
        DataType::Duration(unit) => match unit {
            arrow_schema::TimeUnit::Second => {
                estimate_cardinality_primitive_type::<DurationSecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Millisecond => {
                estimate_cardinality_primitive_type::<DurationMillisecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Microsecond => {
                estimate_cardinality_primitive_type::<DurationMicrosecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Nanosecond => {
                estimate_cardinality_primitive_type::<DurationNanosecondType, 8>(info)
            }
        },
        DataType::Timestamp(unit, _) => match unit {
            arrow_schema::TimeUnit::Second => {
                estimate_cardinality_primitive_type::<TimestampSecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Millisecond => {
                estimate_cardinality_primitive_type::<TimestampMillisecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Microsecond => {
                estimate_cardinality_primitive_type::<TimestampMicrosecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Nanosecond => {
                estimate_cardinality_primitive_type::<TimestampNanosecondType, 8>(info)
            }
        },
        DataType::Float64 => estimate_cardinality_primitive_type::<Float64Type, 8>(info),
        DataType::FixedSizeBinary(8) => estimate_cardinality_fixed_size_type::<8>(info),
        DataType::FixedSizeBinary(16) => estimate_cardinality_fixed_size_type::<16>(info),
        DataType::Utf8 => estimate_cardinality_string_type::<i32>(info),
        DataType::LargeUtf8 => estimate_cardinality_string_type::<i64>(info),
        DataType::LargeBinary => {
            let iter = info
                .values
                .iter()
                .flat_map(|v| v.as_bytes::<GenericBinaryType<i64>>().iter().flatten());
            estimate_cardinality_generic(info, iter)
        }
        _ => unreachable!("Unexpected type: {:?}", info.value_type),
    }
}

fn estimate_cardinality_fixed_size_type<'a, const ELEMENT_WIDTH: usize>(
    info: &FieldInfo<'a>,
) -> Cardinality {
    estimate_cardinality_from_bytes::<ELEMENT_WIDTH>(info, |array| {
        array.as_fixed_size_binary().values()
    })
}

fn estimate_cardinality_primitive_type<'a, T, const ELEMENT_WIDTH: usize>(
    info: &FieldInfo<'a>,
) -> Cardinality
where
    T: ArrowPrimitiveType,
{
    estimate_cardinality_from_bytes::<ELEMENT_WIDTH>(info, |array| {
        array.as_primitive::<T>().values().inner()
    })
}

fn estimate_cardinality_from_bytes<'a, const ELEMENT_WIDTH: usize>(
    info: &'a FieldInfo<'a>,
    get_buffer: impl Fn(&'a ArrayRef) -> &'a [u8],
) -> Cardinality {
    let iter = info.values.iter().flat_map(|array| {
        let nulls = array.nulls();
        let buf = get_buffer(array);

        match nulls {
            Some(nulls) => Either::Left(nulls.valid_slices().flat_map(move |(start, end)| {
                let range = start * ELEMENT_WIDTH..end * ELEMENT_WIDTH;
                buf[range]
                    .as_chunks::<ELEMENT_WIDTH>()
                    .0
                    .iter()
                    .map(|chunk| chunk.as_slice())
            })),
            None => Either::Right(
                buf.as_chunks::<ELEMENT_WIDTH>()
                    .0
                    .iter()
                    .map(|chunk| chunk.as_slice()),
            ),
        }
    });

    estimate_cardinality_generic(info, iter)
}

fn estimate_cardinality_string_type<'a, T: OffsetSizeTrait>(info: &FieldInfo<'a>) -> Cardinality {
    let iter = info
        .values
        .iter()
        .flat_map(|v| v.as_string::<T>().iter().flatten().map(|s| s.as_bytes()));
    estimate_cardinality_generic(info, iter)
}

fn estimate_cardinality_generic<'a>(
    info: &FieldInfo<'a>,
    values: impl Iterator<Item = &'a [u8]>,
) -> Cardinality {
    // TODO: Consider re-use this across cardinality calculations
    let capacity = if info.total_value_count <= u8::MAX as usize {
        u8::MAX as usize
    } else {
        // TODO: is this too big?
        u16::MAX as usize
    };

    let mut set = AHashSet::with_capacity(capacity);
    let mut visited_element_count = 0;

    for value in values {
        _ = set.insert(value);
        visited_element_count += 1;

        let maybe_cardinality = check_cardinality(info, visited_element_count, set.len() as u64);
        if let Some(c) = maybe_cardinality {
            return c;
        }
    }

    Cardinality::from_exact(set.len())
}

fn estimate_cardinality_small_type<'a, T>(info: &FieldInfo<'a>) -> Cardinality
where
    T: ArrowNativeType + Into<u32>,
{
    // TODO: Play around with optimizing bitmap here
    let mut bitmap = RoaringBitmap::new();
    let mut visited_element_count = 0;

    for array in info.values.iter() {
        let value_data = array.to_data();
        let value_buf = value_data.buffer::<T>(0);
        match array.nulls() {
            Some(nulls) => {
                for (start, end) in nulls.valid_slices() {
                    let cardinality = visit_native_values(
                        &value_buf[start..end],
                        info,
                        &mut bitmap,
                        &mut visited_element_count,
                    );
                    if let Some(c) = cardinality {
                        return c;
                    }
                }
            }
            None => {
                let cardinality =
                    visit_native_values(value_buf, info, &mut bitmap, &mut visited_element_count);
                if let Some(c) = cardinality {
                    return c;
                }
            }
        }

        // TODO: Consider when to call bitmap.optimize(). This seemed to regress
        // things for high numbers of small batches. Maybe we can be smarter about
        // calling this only under certain conditions.
        // _ = bitmap.optimize();
    }

    Cardinality::from_exact(bitmap.len() as usize)
}

fn visit_native_values<'a, T>(
    values: &[T],
    info: &FieldInfo<'a>,
    bitmap: &mut RoaringBitmap,
    visited_element_count: &mut usize,
) -> Option<Cardinality>
where
    T: ArrowNativeType + Into<u32>,
{
    const CHUNK_SIZE: usize = 256;

    for chunk in values.chunks(CHUNK_SIZE) {
        bitmap.extend(chunk.iter().copied().map(|v| v.into()));
        *visited_element_count += chunk.len();

        let maybe_cardinality = check_cardinality(info, *visited_element_count, bitmap.len());
        if let Some(c) = maybe_cardinality {
            return Some(c);
        }
    }

    None
}

fn check_cardinality<'a>(
    info: &'a FieldInfo<'a>,
    visited_count: usize,
    current_cardinality: u64,
) -> Option<Cardinality> {
    if current_cardinality > MAX_U16_CARDINALITY as u64 {
        return Some(Cardinality::GreaterThanU16);
    }

    let duplicates_visited = visited_count - current_cardinality as usize;
    let max_possible_cardinality = info.total_value_count - duplicates_visited;

    // If the smallest key type is u8 then it's possible as we keep processing
    // values that we can reduce the size further, so we can't return.
    if max_possible_cardinality <= MAX_U16_CARDINALITY
        && info.smallest_key_type == Some(DataType::UInt16)
    {
        return Some(Cardinality::WithinU16);
    }

    if max_possible_cardinality <= MAX_U8_CARDINALITY
        && info.smallest_key_type == Some(DataType::UInt8)
    {
        return Some(Cardinality::WithinU8);
    }

    None
}

/// Select a specific record batch from every OtapArrowRecords
fn select_all<const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    i: usize,
) -> impl Iterator<Item = Option<&RecordBatch>> {
    batches.iter().map(move |batches| batches[i].as_ref())
}

/// Similar to [select], but does not filter out the `None` values.
fn select_all_mut<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
    i: usize,
) -> impl Iterator<Item = &mut Option<RecordBatch>> {
    batches.iter_mut().map(move |batches| &mut batches[i])
}

// ---------------------------------------------------------------------------
// Plan types
//
// Shared between the ID planner (`reindex`) and the column writers below.
// ---------------------------------------------------------------------------

/// The rows of a single input record batch that survive into the output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum Selection {
    /// Every row is kept. This is the common case.
    #[default]
    All,
    /// Only the rows in these sorted, disjoint, non-empty ranges are kept.
    Ranges(Vec<Range<usize>>),
}

impl Selection {
    /// Number of selected rows given the source length.
    #[must_use]
    pub(crate) fn count(&self, len: usize) -> usize {
        match self {
            Selection::All => len,
            Selection::Ranges(ranges) => ranges.iter().map(|r| r.len()).sum(),
        }
    }

    /// Iterate the selected ranges given the source length.
    pub(crate) fn ranges(&self, len: usize) -> impl Iterator<Item = Range<usize>> + '_ {
        let (all, ranges): (Option<Range<usize>>, &[Range<usize>]) = match self {
            Selection::All => ((len > 0).then_some(0..len), &[]),
            Selection::Ranges(ranges) => (None, ranges.as_slice()),
        };
        all.into_iter().chain(ranges.iter().cloned())
    }
}

/// How the values of one ID column of one input are transformed on the way to
/// the output.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IdRemap<T: ArrowNativeType> {
    /// Values are copied as-is.
    Identity,
    /// `out = in.wrapping_add(delta)`. Wrapping arithmetic is intentional:
    /// null slots may hold arbitrary values that must not panic.
    Offset(T),
    /// Replacement values in source order. Indexed like the source column, or
    /// like the dictionary values array for a dictionary-encoded column.
    Replace(ScalarBuffer<T>),
}

/// An [IdRemap] for either of the two ID widths used by OTAP.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnyRemap {
    U16(IdRemap<u16>),
    U32(IdRemap<u32>),
}

/// The ID columns that may be remapped within a payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdCol {
    Id = 0,
    ResourceId = 1,
    ScopeId = 2,
    ParentId = 3,
}

impl IdCol {
    pub(crate) const COUNT: usize = 4;
}

/// Plan for a single input record batch of a single payload type.
#[derive(Debug, Clone, Default)]
pub(crate) struct InputPlan {
    /// Rows that survive into the output.
    pub(crate) selection: Selection,
    /// Remaps for each ID column, indexed by [IdCol]. `None` means identity.
    pub(crate) remaps: [Option<AnyRemap>; IdCol::COUNT],
}

impl InputPlan {
    /// Look up the remap for an ID column.
    #[must_use]
    pub(crate) fn remap(&self, col: IdCol) -> Option<&AnyRemap> {
        self.remaps[col as usize].as_ref()
    }
}

// ---------------------------------------------------------------------------
// Column writers
// ---------------------------------------------------------------------------

/// One input's contribution to an output column.
#[derive(Clone, Copy)]
pub(crate) struct Input<'a> {
    /// The source column, or `None` if this input does not carry the field.
    pub(crate) column: Option<&'a ArrayRef>,
    /// Number of rows in the source record batch.
    pub(crate) num_rows: usize,
    /// Row selection and ID remaps for this input.
    pub(crate) plan: &'a InputPlan,
}

impl Input<'_> {
    fn selected_rows(&self) -> usize {
        self.plan.selection.count(self.num_rows)
    }

    fn ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        self.plan.selection.ranges(self.num_rows)
    }
}

/// Resolve which ID column (if any) a top-level field corresponds to.
#[must_use]
pub(crate) fn top_level_id_col(name: &str) -> Option<IdCol> {
    match name {
        ID => Some(IdCol::Id),
        PARENT_ID => Some(IdCol::ParentId),
        _ => None,
    }
}

fn struct_child_id_col(struct_name: &str, child_name: &str) -> Option<IdCol> {
    match (struct_name, child_name) {
        (RESOURCE, ID) => Some(IdCol::ResourceId),
        (SCOPE, ID) => Some(IdCol::ScopeId),
        _ => None,
    }
}

/// Write a single output column for `target` from `inputs`.
///
/// `rows` must equal the sum of selected rows across `inputs`.
pub(crate) fn write_column(
    target: &Field,
    inputs: &[Input<'_>],
    rows: usize,
    id_col: Option<IdCol>,
) -> Result<ArrayRef> {
    debug_assert_eq!(rows, inputs.iter().map(Input::selected_rows).sum::<usize>());

    match target.data_type() {
        DataType::Struct(fields) => write_struct(target.name(), fields, inputs, rows),
        DataType::Dictionary(key, value) => match key.as_ref() {
            DataType::UInt8 => dispatch_value::<DictDriver<UInt8Type>>(value, inputs, rows, id_col),
            DataType::UInt16 => {
                dispatch_value::<DictDriver<UInt16Type>>(value, inputs, rows, id_col)
            }
            _ => write_fallback(target.data_type(), inputs, rows),
        },
        value => dispatch_value::<NativeDriver>(value, inputs, rows, id_col),
    }
}

// ---------------------------------------------------------------------------
// Drivers
// ---------------------------------------------------------------------------

/// Abstraction over [write_native] and [write_dict] so value-type dispatch is
/// written once.
trait Driver {
    /// True if the output is dictionary encoded.
    const IS_DICT: bool;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef>;
}

struct NativeDriver;
impl Driver for NativeDriver {
    const IS_DICT: bool = false;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef> {
        write_native(builder, value_type, inputs, rows)
    }
}

struct DictDriver<K>(std::marker::PhantomData<K>);
impl<K: ArrowDictionaryKeyType> Driver for DictDriver<K> {
    const IS_DICT: bool = true;

    fn write<B: ValueBuilder>(
        builder: B,
        value_type: &DataType,
        inputs: &[Input<'_>],
        rows: usize,
    ) -> Result<ArrayRef> {
        write_dict::<K, B>(builder, value_type, inputs, rows)
    }
}

/// Upper bound on the number of values the builder will receive, used for
/// capacity. For native output this is `rows`; for dictionary output this is
/// the sum of dictionary value lengths plus native selected rows.
fn dict_values_capacity(inputs: &[Input<'_>]) -> usize {
    inputs
        .iter()
        .map(|inp| match inp.column {
            Some(col) => match col.data_type() {
                DataType::Dictionary(_, _) => dict_values(col).map(|v| v.len()).unwrap_or(0),
                _ => inp.selected_rows(),
            },
            None => 0,
        })
        .sum()
}

fn dispatch_value<D: Driver>(
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
    id_col: Option<IdCol>,
) -> Result<ArrayRef> {
    let cap = if D::IS_DICT {
        dict_values_capacity(inputs)
    } else {
        rows
    };

    macro_rules! prim {
        ($t:ty) => {
            D::write(
                PrimitiveBuilder::<$t>::new(cap, value_type.clone(), None),
                value_type,
                inputs,
                rows,
            )
        };
    }

    match value_type {
        DataType::UInt8 => prim!(UInt8Type),
        DataType::UInt16 => D::write(
            PrimitiveBuilder::<UInt16Type>::new(cap, value_type.clone(), id_col.map(u16_remap)),
            value_type,
            inputs,
            rows,
        ),
        DataType::UInt32 => D::write(
            PrimitiveBuilder::<UInt32Type>::new(cap, value_type.clone(), id_col.map(u32_remap)),
            value_type,
            inputs,
            rows,
        ),
        DataType::UInt64 => prim!(UInt64Type),
        DataType::Int32 => prim!(Int32Type),
        DataType::Int64 => prim!(Int64Type),
        DataType::Float64 => prim!(Float64Type),
        DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, _) => {
            prim!(TimestampNanosecondType)
        }
        DataType::Duration(arrow_schema::TimeUnit::Nanosecond) => prim!(DurationNanosecondType),
        DataType::Boolean => D::write(BoolBuilder::new(cap), value_type, inputs, rows),
        DataType::Utf8 => D::write(
            BytesBuilder::<Utf8Type>::new(cap, inputs),
            value_type,
            inputs,
            rows,
        ),
        DataType::Binary => D::write(
            BytesBuilder::<BinaryType>::new(cap, inputs),
            value_type,
            inputs,
            rows,
        ),
        DataType::FixedSizeBinary(w) => {
            D::write(FsbBuilder::new(cap, *w as usize), value_type, inputs, rows)
        }
        _ if D::IS_DICT => Err(Error::UnexpectedRecordBatchState {
            reason: format!("unsupported dictionary value type {value_type:?}"),
        }),
        _ => write_fallback(value_type, inputs, rows),
    }
}

/// Produce a plain output column.
fn write_native<B: ValueBuilder>(
    mut builder: B,
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut nulls = LazyNulls::new(rows);

    for inp in inputs {
        builder.begin_input(inp.plan);
        let Some(col) = inp.column else {
            let n = inp.selected_rows();
            builder.append_default(n);
            nulls.append_null(n);
            continue;
        };

        match col.data_type() {
            DataType::Dictionary(key, _) => match key.as_ref() {
                DataType::UInt8 => gather_dict::<UInt8Type, B>(&mut builder, &mut nulls, inp, col),
                DataType::UInt16 => {
                    gather_dict::<UInt16Type, B>(&mut builder, &mut nulls, inp, col)
                }
                k => return Err(unsupported_key(k)),
            },
            dt => {
                check_value_type(dt, value_type)?;
                let src = B::downcast(col.as_ref());
                for r in inp.ranges() {
                    builder.append_range(&src, r.clone());
                    nulls.append_from(col.nulls(), r);
                }
            }
        }
    }

    builder.finish(nulls.finish())
}

/// Gather values through dictionary keys into a native builder.
fn gather_dict<K: ArrowDictionaryKeyType, B: ValueBuilder>(
    builder: &mut B,
    nulls: &mut LazyNulls,
    inp: &Input<'_>,
    col: &ArrayRef,
) {
    let dict = col.as_dictionary::<K>();
    let values = dict.values();
    let keys = dict.keys().values();
    let values_len = values.len();
    let src = B::downcast(values.as_ref());

    for r in inp.ranges() {
        if values_len == 0 {
            // Every key must be null.
            builder.append_default(r.len());
        } else {
            // Null keys may hold arbitrary values; clamp so the gather is
            // always in bounds. The null buffer masks the result.
            let max = values_len - 1;
            builder.append_gather::<K::Native>(&src, &keys[r.clone()], max);
        }
        // A null value referenced by a valid key is also null in the output.
        match values.nulls() {
            Some(vn) if vn.null_count() > 0 => {
                for k in keys[r.clone()].iter().zip(r.clone()) {
                    let (key, row) = k;
                    let valid = dict.keys().is_valid(row)
                        && key.as_usize() < values_len
                        && vn.is_valid(key.as_usize());
                    if valid {
                        nulls.append_valid(1);
                    } else {
                        nulls.append_null(1);
                    }
                }
            }
            _ => nulls.append_from(dict.nulls(), r),
        }
    }
}

/// Produce a dictionary output column with key type `K`.
fn write_dict<K: ArrowDictionaryKeyType, B: ValueBuilder>(
    mut builder: B,
    value_type: &DataType,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut keys: Vec<K::Native> = Vec::with_capacity(rows);
    let mut key_nulls = LazyNulls::new(rows);
    let mut value_nulls = LazyNulls::new(dict_values_capacity(inputs));
    let mut vbase: usize = 0;

    for inp in inputs {
        builder.begin_input(inp.plan);
        let Some(col) = inp.column else {
            let n = inp.selected_rows();
            keys.resize(keys.len() + n, K::Native::usize_as(0));
            key_nulls.append_null(n);
            continue;
        };

        match col.data_type() {
            DataType::Dictionary(key, _) => match key.as_ref() {
                DataType::UInt8 => append_dict_input::<UInt8Type, K, B>(
                    &mut builder,
                    &mut keys,
                    &mut key_nulls,
                    &mut value_nulls,
                    &mut vbase,
                    inp,
                    col,
                    value_type,
                )?,
                DataType::UInt16 => append_dict_input::<UInt16Type, K, B>(
                    &mut builder,
                    &mut keys,
                    &mut key_nulls,
                    &mut value_nulls,
                    &mut vbase,
                    inp,
                    col,
                    value_type,
                )?,
                k => return Err(unsupported_key(k)),
            },
            dt => {
                check_value_type(dt, value_type)?;
                let src = B::downcast(col.as_ref());
                for r in inp.ranges() {
                    let n = r.len();
                    builder.append_range(&src, r.clone());
                    value_nulls.append_valid(n);
                    keys.extend((vbase..vbase + n).map(K::Native::usize_as));
                    key_nulls.append_from(col.nulls(), r);
                    vbase += n;
                }
            }
        }
    }

    // The schema selection bounds the total number of physical values so
    // that they fit the chosen key type. Guard anyway rather than wrapping.
    if vbase > 0 && K::Native::from_usize(vbase - 1).is_none() {
        return Err(Error::Batching {
            source: ArrowError::DictionaryKeyOverflowError,
        });
    }

    let values = builder.finish(value_nulls.finish())?;
    let keys = PrimitiveArray::<K>::new(ScalarBuffer::from(keys), key_nulls.finish());
    debug_assert!(DictionaryArray::<K>::try_new(keys.clone(), values.clone()).is_ok());

    // SAFETY: every valid key was produced as `vbase + k` where `k` is a
    // valid index into the values appended for that input (either a source
    // dictionary key, which the source array guarantees to be in bounds, or
    // a sequential index into the values just appended). Null keys may hold
    // arbitrary values, which Arrow permits.
    #[allow(unsafe_code)]
    let dict = unsafe { DictionaryArray::<K>::new_unchecked(keys, values) };
    Ok(Arc::new(dict))
}

#[allow(clippy::too_many_arguments)]
fn append_dict_input<Ks: ArrowDictionaryKeyType, K: ArrowDictionaryKeyType, B: ValueBuilder>(
    builder: &mut B,
    keys: &mut Vec<K::Native>,
    key_nulls: &mut LazyNulls,
    value_nulls: &mut LazyNulls,
    vbase: &mut usize,
    inp: &Input<'_>,
    col: &ArrayRef,
    value_type: &DataType,
) -> Result<()> {
    let dict = col.as_dictionary::<Ks>();
    let values = dict.values();
    check_value_type(values.data_type(), value_type)?;

    // Append the whole values array. This keeps key rewriting a pure add.
    let values_len = values.len();
    let src = B::downcast(values.as_ref());
    builder.append_range(&src, 0..values_len);
    value_nulls.append_from(values.nulls(), 0..values_len);

    let src_keys = dict.keys().values();
    let base = *vbase;
    for r in inp.ranges() {
        keys.extend(
            src_keys[r.clone()]
                .iter()
                .map(|k| K::Native::usize_as(k.as_usize() + base)),
        );
        key_nulls.append_from(dict.nulls(), r);
    }
    *vbase += values_len;
    Ok(())
}

/// Produce a struct output column, recursing into its children.
fn write_struct(
    struct_name: &str,
    fields: &Fields,
    inputs: &[Input<'_>],
    rows: usize,
) -> Result<ArrayRef> {
    let mut nulls = LazyNulls::new(rows);
    let structs: Vec<Option<&StructArray>> = inputs
        .iter()
        .map(|inp| inp.column.map(|c| c.as_struct()))
        .collect();

    for (inp, s) in inputs.iter().zip(&structs) {
        match s {
            Some(s) => {
                for r in inp.ranges() {
                    nulls.append_from(s.nulls(), r);
                }
            }
            None => nulls.append_null(inp.selected_rows()),
        }
    }

    let mut children = Vec::with_capacity(fields.len());
    let mut child_inputs: Vec<Input<'_>> = Vec::with_capacity(inputs.len());
    for field in fields.iter() {
        child_inputs.clear();
        child_inputs.extend(inputs.iter().zip(&structs).map(|(inp, s)| Input {
            column: s.and_then(|s| s.column_by_name(field.name())),
            num_rows: inp.num_rows,
            plan: inp.plan,
        }));
        let id_col = struct_child_id_col(struct_name, field.name());
        children.push(write_column(field, &child_inputs, rows, id_col)?);
    }

    let array = StructArray::try_new_with_length(fields.clone(), children, nulls.finish(), rows)
        .map_err(|source| Error::Batching { source })?;
    Ok(Arc::new(array))
}

/// Generic fallback using `MutableArrayData`. Used for List columns. Inputs of
/// a different type are cast to `target` first.
fn write_fallback(target: &DataType, inputs: &[Input<'_>], rows: usize) -> Result<ArrayRef> {
    let mut casted: Vec<ArrayRef> = Vec::with_capacity(inputs.len());
    for inp in inputs {
        if let Some(col) = inp.column {
            let col = if col.data_type() == target {
                Arc::clone(col)
            } else {
                cast(col.as_ref(), target).map_err(|source| Error::Batching { source })?
            };
            casted.push(col);
        }
    }

    let datas: Vec<ArrayData> = casted.iter().map(|a| a.to_data()).collect();
    let refs: Vec<&ArrayData> = datas.iter().collect();
    if refs.is_empty() {
        return Ok(arrow::array::new_null_array(target, rows));
    }

    let mut mutable = MutableArrayData::new(refs, true, rows);
    let mut src_idx = 0;
    for inp in inputs {
        match inp.column {
            Some(_) => {
                for r in inp.ranges() {
                    mutable.extend(src_idx, r.start, r.end);
                }
                src_idx += 1;
            }
            None => mutable.extend_nulls(inp.selected_rows()),
        }
    }

    Ok(make_array(mutable.freeze()))
}

fn check_value_type(actual: &DataType, expected: &DataType) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error::ColumnDataTypeMismatch {
            name: String::new(),
            expect: expected.clone(),
            actual: actual.clone(),
        })
    }
}

fn unsupported_key(k: &DataType) -> Error {
    Error::UnsupportedDictionaryKeyType {
        expect_oneof: vec![DataType::UInt8, DataType::UInt16],
        actual: k.clone(),
    }
}

fn dict_values(col: &ArrayRef) -> Option<&ArrayRef> {
    match col.data_type() {
        DataType::Dictionary(k, _) => match k.as_ref() {
            DataType::UInt8 => Some(col.as_dictionary::<UInt8Type>().values()),
            DataType::UInt16 => Some(col.as_dictionary::<UInt16Type>().values()),
            _ => None,
        },
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Null buffer
// ---------------------------------------------------------------------------

/// A null buffer builder that allocates only once a null is seen.
pub(crate) struct LazyNulls {
    len: usize,
    capacity: usize,
    builder: Option<BooleanBufferBuilder>,
}

impl LazyNulls {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            len: 0,
            capacity,
            builder: None,
        }
    }

    fn materialize(&mut self) -> &mut BooleanBufferBuilder {
        let (len, capacity) = (self.len, self.capacity);
        self.builder.get_or_insert_with(|| {
            let mut b = BooleanBufferBuilder::new(capacity.max(len));
            b.append_n(len, true);
            b
        })
    }

    pub(crate) fn append_valid(&mut self, n: usize) {
        if let Some(b) = self.builder.as_mut() {
            b.append_n(n, true);
        }
        self.len += n;
    }

    pub(crate) fn append_null(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.materialize().append_n(n, false);
        self.len += n;
    }

    /// Append validity for `range` (logical indices) of a source null buffer.
    pub(crate) fn append_from(&mut self, nulls: Option<&NullBuffer>, range: Range<usize>) {
        match nulls {
            Some(n) if n.null_count() > 0 => {
                let inner = n.inner();
                let offset = inner.offset();
                let len = range.len();
                self.materialize()
                    .append_packed_range(offset + range.start..offset + range.end, inner.values());
                self.len += len;
            }
            _ => self.append_valid(range.len()),
        }
    }

    pub(crate) fn finish(self) -> Option<NullBuffer> {
        self.builder
            .map(|mut b| NullBuffer::new(b.finish()))
            .filter(|n| n.null_count() > 0)
    }
}

// ---------------------------------------------------------------------------
// Value builders
// ---------------------------------------------------------------------------

/// Appends values of a single physical type into a pre-sized destination.
trait ValueBuilder {
    /// Typed view of a source array.
    type Src<'a>;

    fn downcast(array: &dyn Array) -> Self::Src<'_>;

    /// Called before each input is processed.
    fn begin_input(&mut self, _plan: &InputPlan) {}

    /// Append logical indices `range` of `src`.
    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>);

    /// Append `src[min(k, max)]` for each dictionary key `k` in `keys`.
    ///
    /// Keys are clamped to `max` (the last valid index) because null keys may
    /// hold arbitrary values; the null buffer masks those rows.
    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize);

    /// Append `n` placeholder values (masked by nulls).
    fn append_default(&mut self, n: usize);

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef>;
}

/// Extracts the typed remap for an ID column from an [InputPlan].
type RemapFn<T> = fn(&InputPlan, IdCol) -> Option<IdRemap<T>>;

fn u16_remap(col: IdCol) -> (IdCol, RemapFn<u16>) {
    (col, |plan, col| match plan.remap(col) {
        Some(AnyRemap::U16(r)) => Some(r.clone()),
        _ => None,
    })
}

fn u32_remap(col: IdCol) -> (IdCol, RemapFn<u32>) {
    (col, |plan, col| match plan.remap(col) {
        Some(AnyRemap::U32(r)) => Some(r.clone()),
        _ => None,
    })
}

/// Split dictionary keys into maximal runs of consecutive value indices and
/// call `f` with each run as a range of value indices, in key order.
///
/// Keys are clamped to `max` (see [ValueBuilder::append_gather]). A run of
/// length 1 is a single value.
#[inline]
fn for_each_key_run<K: ArrowNativeType>(keys: &[K], max: usize, mut f: impl FnMut(Range<usize>)) {
    let mut iter = keys.iter().map(|k| k.as_usize().min(max));
    let Some(first) = iter.next() else {
        return;
    };
    let mut start = first;
    let mut end = first + 1;
    for k in iter {
        if k == end {
            end += 1;
        } else {
            f(start..end);
            start = k;
            end = k + 1;
        }
    }
    f(start..end);
}

struct PrimitiveBuilder<T: ArrowPrimitiveType> {
    values: Vec<T::Native>,
    data_type: DataType,
    remap_source: Option<(IdCol, RemapFn<T::Native>)>,
    remap: IdRemap<T::Native>,
}

impl<T: ArrowPrimitiveType> PrimitiveBuilder<T> {
    fn new(
        capacity: usize,
        data_type: DataType,
        remap_source: Option<(IdCol, RemapFn<T::Native>)>,
    ) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            data_type,
            remap_source,
            remap: IdRemap::Identity,
        }
    }
}

impl<T: ArrowPrimitiveType> ValueBuilder for PrimitiveBuilder<T> {
    type Src<'a> = &'a [T::Native];

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_primitive::<T>().values()
    }

    fn begin_input(&mut self, plan: &InputPlan) {
        if let Some((col, f)) = self.remap_source {
            self.remap = f(plan, col).unwrap_or(IdRemap::Identity);
        }
    }

    #[inline]
    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        match &self.remap {
            IdRemap::Identity => self.values.extend_from_slice(&src[range]),
            IdRemap::Offset(d) => {
                let d = *d;
                self.values
                    .extend(src[range].iter().map(|v| v.add_wrapping(d)));
            }
            IdRemap::Replace(buf) => self.values.extend_from_slice(&buf[range]),
        }
    }

    #[inline]
    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        let indices = keys.iter().map(|k| k.as_usize().min(max));
        match &self.remap {
            IdRemap::Identity => self.values.extend(indices.map(|i| src[i])),
            IdRemap::Offset(d) => {
                let d = *d;
                self.values.extend(indices.map(|i| src[i].add_wrapping(d)));
            }
            IdRemap::Replace(buf) => self.values.extend(indices.map(|i| buf[i])),
        }
    }

    fn append_default(&mut self, n: usize) {
        self.values
            .resize(self.values.len() + n, T::Native::default());
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        let array = PrimitiveArray::<T>::new(ScalarBuffer::from(self.values), nulls)
            .with_data_type(self.data_type);
        Ok(Arc::new(array))
    }
}

struct BoolBuilder {
    values: BooleanBufferBuilder,
}

impl BoolBuilder {
    fn new(capacity: usize) -> Self {
        Self {
            values: BooleanBufferBuilder::new(capacity),
        }
    }
}

impl ValueBuilder for BoolBuilder {
    type Src<'a> = &'a BooleanArray;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_boolean()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let bits = src.values();
        let offset = bits.offset();
        self.values
            .append_packed_range(offset + range.start..offset + range.end, bits.values());
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        let bits = src.values();
        for k in keys {
            self.values.append(bits.value(k.as_usize().min(max)));
        }
    }

    fn append_default(&mut self, n: usize) {
        self.values.append_n(n, false);
    }

    fn finish(mut self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        Ok(Arc::new(BooleanArray::new(self.values.finish(), nulls)))
    }
}

struct BytesBuilder<T: ByteArrayType<Offset = i32>> {
    offsets: Vec<i32>,
    data: Vec<u8>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ByteArrayType<Offset = i32>> BytesBuilder<T> {
    fn new(capacity: usize, inputs: &[Input<'_>]) -> Self {
        // Exact byte capacity for native inputs; dictionary inputs are either
        // gathered (unknown, grows) or appended whole (exact).
        let mut bytes = 0usize;
        for inp in inputs {
            let Some(col) = inp.column else { continue };
            match col.data_type() {
                DataType::Dictionary(_, _) => {
                    if let Some(v) = dict_values(col) {
                        let o = v.as_bytes::<T>().value_offsets();
                        bytes += (o[o.len() - 1] - o[0]) as usize;
                    }
                }
                _ => {
                    let o = col.as_bytes::<T>().value_offsets();
                    for r in inp.ranges() {
                        bytes += (o[r.end] - o[r.start]) as usize;
                    }
                }
            }
        }

        let mut offsets = Vec::with_capacity(capacity + 1);
        offsets.push(0);
        Self {
            offsets,
            data: Vec::with_capacity(bytes),
            _phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    fn last(&self) -> i32 {
        // safety: always contains the initial 0
        *self.offsets.last().expect("non-empty offsets")
    }
}

impl<T: ByteArrayType<Offset = i32>> ValueBuilder for BytesBuilder<T> {
    type Src<'a> = &'a GenericByteArray<T>;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_bytes::<T>()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let o = src.value_offsets();
        let start = o[range.start];
        let end = o[range.end];
        self.data
            .extend_from_slice(&src.value_data()[start as usize..end as usize]);
        // Offsets may overflow i32 only if the output exceeds 2GiB; this is
        // checked in `finish`.
        let delta = self.last().wrapping_sub(start);
        self.offsets.extend(
            o[range.start + 1..=range.end]
                .iter()
                .map(|x| x.wrapping_add(delta)),
        );
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        // Runs of consecutive keys reference contiguous values, so each run
        // is copied like `append_range`: one memcpy for the bytes and one
        // shifted copy of the offsets.
        for_each_key_run(keys, max, |run| self.append_range(src, run));
    }

    fn append_default(&mut self, n: usize) {
        let last = self.last();
        self.offsets.resize(self.offsets.len() + n, last);
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        if i32::try_from(self.data.len()).is_err() {
            return Err(Error::Batching {
                source: ArrowError::OffsetOverflowError(self.data.len()),
            });
        }

        let offsets = ScalarBuffer::from(self.offsets);
        let values = Buffer::from_vec(self.data);
        #[cfg(debug_assertions)]
        {
            let checked = OffsetBuffer::new(offsets.clone());
            debug_assert!(
                GenericByteArray::<T>::try_new(checked, values.clone(), nulls.clone()).is_ok()
            );
        }

        // SAFETY: offsets start at 0, are monotonically non-decreasing (each
        // appended value has non-negative length), and the last offset equals
        // `values.len()` which fits in i32 (checked above). Every value was
        // copied whole from a valid source array of the same type `T`, so for
        // Utf8 each value is valid UTF-8 and boundaries fall on char
        // boundaries.
        #[allow(unsafe_code)]
        let array = unsafe {
            let offsets = OffsetBuffer::new_unchecked(offsets);
            GenericByteArray::<T>::new_unchecked(offsets, values, nulls)
        };
        Ok(Arc::new(array))
    }
}

struct FsbBuilder {
    width: usize,
    data: Vec<u8>,
}

impl FsbBuilder {
    fn new(capacity: usize, width: usize) -> Self {
        Self {
            width,
            data: Vec::with_capacity(capacity * width),
        }
    }
}

impl ValueBuilder for FsbBuilder {
    type Src<'a> = &'a FixedSizeBinaryArray;

    fn downcast(array: &dyn Array) -> Self::Src<'_> {
        array.as_fixed_size_binary()
    }

    fn append_range(&mut self, src: &Self::Src<'_>, range: Range<usize>) {
        let w = self.width;
        self.data
            .extend_from_slice(&src.value_data()[range.start * w..range.end * w]);
    }

    fn append_gather<K: ArrowNativeType>(&mut self, src: &Self::Src<'_>, keys: &[K], max: usize) {
        // Runs of consecutive keys reference contiguous values: one memcpy per
        // run.
        for_each_key_run(keys, max, |run| self.append_range(src, run));
    }

    fn append_default(&mut self, n: usize) {
        self.data.resize(self.data.len() + n * self.width, 0);
    }

    fn finish(self, nulls: Option<NullBuffer>) -> Result<ArrayRef> {
        let array =
            FixedSizeBinaryArray::try_new(self.width as i32, Buffer::from_vec(self.data), nulls)
                .map_err(|source| Error::Batching { source })?;
        Ok(Arc::new(array))
    }
}

/// Benchmark-only accessors that expose the concatenation stages
/// (`index_records`, `select_schema`, and writing the output) so their cost can
/// be measured separately.
///
/// Each stage consumes the previous stage's output, so each accessor runs a
/// cumulative prefix of the pipeline: `bench_index_records` runs indexing,
/// `bench_select_schema` runs indexing + selection, and `bench_write_payload`
/// runs all three. Per-stage cost is the difference between adjacent results.
///
/// These are gated behind the `bench` feature and are not part of the public
/// API. They exist purely so the `schema_unify` benchmark can attribute time to
/// each stage.
#[cfg(feature = "bench")]
pub mod bench_exports {
    use super::*;

    /// Run only the field-indexing stage for payload slot `i` across `items`.
    ///
    /// Returns the number of distinct fields discovered so the caller can keep
    /// the result observable and prevent the optimizer from eliding the work.
    pub fn bench_index_records<S: OtapBatchStore, const N: usize>(
        items: &[[Option<RecordBatch>; N]],
        i: usize,
    ) -> Result<usize> {
        let payload_def = payloads::get(S::payload_type_at_idx(i));
        let index = index_records(select_all(items, i), payload_def)?;
        Ok(index.fields.slots.iter().filter(|s| s.is_some()).count())
    }

    /// Run the indexing and schema-selection stages for payload slot `i`. The
    /// timing includes `index_records`, since selection consumes its output.
    pub fn bench_select_schema<S: OtapBatchStore, const N: usize>(
        items: &[[Option<RecordBatch>; N]],
        i: usize,
    ) -> Result<Schema> {
        let payload_def = payloads::get(S::payload_type_at_idx(i));
        let index = index_records(select_all(items, i), payload_def)?;
        Ok(select_schema(&index)?.schema)
    }

    /// Run the full per-payload pipeline (index + select + write) for payload
    /// slot `i`, returning the concatenated output batch. The timing includes
    /// `index_records` and `select_schema`.
    pub fn bench_write_payload<S: OtapBatchStore, const N: usize>(
        items: &[[Option<RecordBatch>; N]],
        i: usize,
    ) -> Result<RecordBatch> {
        let payload_def = payloads::get(S::payload_type_at_idx(i));
        let index = index_records(select_all(items, i), payload_def)?;
        let selected = select_schema(&index)?;
        let batches: Vec<&RecordBatch> = select_all(items, i).flatten().collect();
        let plans = vec![InputPlan::default(); batches.len()];
        write_payload(&batches, &plans, payload_def, selected)
    }
}

#[cfg(test)]
mod batch_width_tests {
    use super::*;

    /// Scenario: call `concatenate` with a batch width (1) that matches no
    /// signal, using zero, one, and two input batch arrays so the empty and
    /// single-batch fast paths are exercised as well as the general path.
    /// Guarantees: every call returns `UnsupportedBatchStoreType` carrying the
    /// width instead of panicking or silently succeeding.
    #[test]
    fn unsupported_batch_width_returns_error() {
        for num_batches in 0..=2 {
            let mut items: Vec<[Option<RecordBatch>; 1]> = vec![[None]; num_batches];
            let err = concatenate::<1>(&mut items, ConcatOptions::preserve_ids()).unwrap_err();
            assert!(
                matches!(err, Error::UnsupportedBatchStoreType { batch_width: 1 }),
                "{num_batches} batches: unexpected error: {err:?}"
            );
        }
    }

    /// Scenario: compare each signal store's `COUNT` with the number of payload
    /// types it allows.
    /// Guarantees: they are equal for Logs, Metrics, and Traces, so
    /// `concatenate_signal`'s `payload_type_at_idx(i)` lookup for every
    /// `i < COUNT` can never index out of bounds.
    #[test]
    fn signal_counts_match_payload_types() {
        assert_eq!(Logs::COUNT, Logs::allowed_payload_types().len());
        assert_eq!(Metrics::COUNT, Metrics::allowed_payload_types().len());
        assert_eq!(Traces::COUNT, Traces::allowed_payload_types().len());
    }
}

#[cfg(test)]
mod spec_index_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::schema::schema::DataType as SpecDataType;

    /// Every `ArrowPayloadType` variant, discovered by probing the prost
    /// `TryFrom<i32>` conversion so newly added variants are covered without
    /// maintaining a hand-written list.
    fn all_payload_types() -> Vec<ArrowPayloadType> {
        (0..=i32::from(u8::MAX))
            .filter_map(|v| ArrowPayloadType::try_from(v).ok())
            .collect()
    }

    /// Visit `schema` and every schema nested beneath it (struct fields and
    /// lists of structs), calling `visit` with a dotted field path and the
    /// number of fields declared at that level.
    fn visit_schemas(path: &str, schema: &PayloadSchema, visit: &mut impl FnMut(&str, usize)) {
        visit(path, schema.fields().len());
        for field in schema.fields() {
            let nested = match &field.data_type {
                SpecDataType::Struct(sub) => Some(*sub),
                SpecDataType::List(SpecDataType::Struct(sub)) => Some(*sub),
                _ => None,
            };
            if let Some(sub) = nested {
                visit_schemas(&format!("{path}.{}", field.name), sub, visit);
            }
        }
    }

    /// Scenario: enumerate every `ArrowPayloadType` variant (including
    /// `Unknown` and `MultivariateMetrics`) and walk its payload schema plus all
    /// nested struct and list-of-struct schemas.
    /// Guarantees: no schema at any nesting level declares more than
    /// `MAX_SLOTS` fields, so every spec field has an addressable slot in the
    /// fixed-size `FieldIndex` and no column is silently dropped.
    #[test]
    fn all_payload_schemas_fit_max_slots() {
        let types = all_payload_types();
        assert!(
            types.len() > 20,
            "expected to discover all payload types, found only {}",
            types.len()
        );

        for pt in types {
            visit_schemas(&format!("{pt:?}"), payloads::get(pt), &mut |path, len| {
                assert!(
                    len <= MAX_SLOTS,
                    "{path} declares {len} fields, exceeding MAX_SLOTS {MAX_SLOTS}"
                );
            });
        }
    }

    /// Scenario: compute the widest field count across every payload schema
    /// and all nested schemas.
    /// Guarantees: `MAX_SLOTS` equals that widest count, so the fixed-size
    /// `FieldIndex` is not over-allocated; if schemas change, this fails and
    /// reports the new maximum to set.
    #[test]
    fn max_slots_is_tight() {
        let mut widest = (String::new(), 0usize);
        for pt in all_payload_types() {
            visit_schemas(&format!("{pt:?}"), payloads::get(pt), &mut |path, len| {
                if len > widest.1 {
                    widest = (path.to_string(), len);
                }
            });
        }
        assert_eq!(
            widest.1, MAX_SLOTS,
            "widest payload schema is {} with {} fields; set MAX_SLOTS to match",
            widest.0, widest.1
        );
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::otap::raw_batch_store::LOGS_COUNT;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::{
        LogAttrs, Logs, ResourceAttrs, SpanEventAttrs,
    };
    use crate::record_batch;
    use crate::schema::consts::{
        ATTRIBUTE_INT, ATTRIBUTE_KEY, ATTRIBUTE_SER, ATTRIBUTE_STR, ATTRIBUTE_TYPE, BODY,
        SEVERITY_TEXT, SPAN_ID, TRACE_ID,
    };
    use arrow::array::{
        Array, DictionaryArray, FixedSizeBinaryArray, Int64Array, PrimitiveArray, StringArray,
        UInt8Array, UInt16Array,
    };
    use arrow::datatypes::DataType;
    use std::sync::Arc;

    /// Build a single-column record batch carrying `column` as a dictionary of
    /// the given keys/values. The column name must be a real OTAP field so the
    /// spec-indexed `index_records` accepts it.
    fn create_dict_batch<K: ArrowDictionaryKeyType>(
        name: &str,
        keys: PrimitiveArray<K>,
        values: Arc<dyn Array>,
        value_type: DataType,
    ) -> RecordBatch
    where
        PrimitiveArray<K>: From<Vec<K::Native>>,
    {
        let dict_array = DictionaryArray::<K>::try_new(keys, values).unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new(
            name,
            DataType::Dictionary(Box::new(K::DATA_TYPE), Box::new(value_type)),
            true,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap()
    }

    /// Position of the root Logs table in the Logs signal store's batch array
    /// (allowed_payload_types = [ResourceAttrs, ScopeAttrs, Logs, LogAttrs]).
    const LOGS_ROOT_IDX: usize = 2;

    /// Concatenate two root Logs batches through the real Logs signal path so the
    /// spec (`payloads::get(Logs)`) drives schema selection, and return the root
    /// output batch.
    fn concat_logs_root(batch1: RecordBatch, batch2: RecordBatch) -> RecordBatch {
        let mut a: [Option<RecordBatch>; LOGS_COUNT] = Default::default();
        let mut b: [Option<RecordBatch>; LOGS_COUNT] = Default::default();
        a[LOGS_ROOT_IDX] = Some(batch1);
        b[LOGS_ROOT_IDX] = Some(batch2);
        let mut batches = vec![a, b];
        let result =
            concatenate::<LOGS_COUNT>(&mut batches, ConcatOptions::preserve_ids()).unwrap();
        result[LOGS_ROOT_IDX]
            .as_ref()
            .expect("concatenated root logs batch")
            .clone()
    }

    /// Build a `trace_id` batch (spec: Dict(u16, FixedSizeBinary(16))) using u16
    /// keys so `count` distinct 16-byte values are physically present.
    fn create_fsb_dictionary_batch(start: usize, count: usize) -> RecordBatch {
        let keys = UInt16Array::from((0..count).map(|i| i as u16).collect::<Vec<_>>());
        let values = generate_fsb16_values(start, count);
        create_dict_batch(TRACE_ID, keys, values, DataType::FixedSizeBinary(16))
    }

    /// Build a Logs "body" struct whose `ser` child (spec: Dict(u16, Binary)) has
    /// `count` distinct physical values. Used to exercise the nested
    /// physical-bound / fallback path through a real struct column.
    fn create_struct_binary_dictionary_batch(start: usize, count: usize) -> RecordBatch {
        let keys = UInt16Array::from((0..count).map(|i| i as u16).collect::<Vec<_>>());
        let values = generate_binary_values(start, count);
        let dict_field = Field::new(
            ATTRIBUTE_SER,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
            false,
        );
        let dict_array = DictionaryArray::<UInt16Type>::try_new(keys, values).unwrap();
        let struct_array = StructArray::from(vec![(
            Arc::new(dict_field.clone()),
            Arc::new(dict_array) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            BODY,
            DataType::Struct(vec![dict_field].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    fn generate_fsb16_values(start: usize, count: usize) -> Arc<dyn Array> {
        use arrow::buffer::Buffer;
        let values: Vec<u8> = (start..start + count)
            .flat_map(|i| expected_fsb_16(i as u64))
            .collect();
        let buffer = Buffer::from_vec(values);
        Arc::new(FixedSizeBinaryArray::try_new(16, buffer, None).unwrap())
    }

    fn generate_binary_values(start: usize, count: usize) -> Arc<dyn Array> {
        use arrow::array::BinaryArray;
        let owned: Vec<[u8; 8]> = (start..start + count)
            .map(|i| (i as u64).to_le_bytes())
            .collect();
        Arc::new(BinaryArray::from_iter_values(
            owned.iter().map(|b| b.as_slice()),
        ))
    }

    fn expected_fsb_16(value: u64) -> [u8; 16] {
        let mut expected = [0; 16];
        expected[..8].copy_from_slice(&value.to_le_bytes());
        expected[8..].copy_from_slice(&value.to_le_bytes());
        expected
    }

    fn assert_overlapping_fsb_values(array: &ArrayRef, count: usize, overlap: usize) {
        let array = array
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .expect("plain FixedSizeBinary array");

        for (row, value) in [
            (0, 0),
            (count - 1, count - 1),
            (count, overlap),
            (2 * count - 1, overlap + count - 1),
        ] {
            assert_eq!(array.value(row), expected_fsb_16(value as u64).as_slice());
        }
    }

    /// Create a Dict(u8, Utf8) batch with low cardinality for a real Utf8 dict
    /// column (e.g. "str", "key", "severity_text").
    fn create_low_cardinality_u8_utf8_batch(col_name: &str, n_values: usize) -> RecordBatch {
        assert!(n_values <= 255);
        let keys = UInt8Array::from((0..n_values).map(|i| i as u8).collect::<Vec<_>>());
        let values: Arc<dyn Array> = Arc::new(StringArray::from(
            (0..n_values)
                .map(|i| format!("val_{}", i))
                .collect::<Vec<_>>(),
        ));
        let dict_array = DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new(
            col_name,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap()
    }

    /// Create a Dict(u8, Int64) batch with low cardinality for a real Int64 dict
    /// column (e.g. "int").
    fn create_low_cardinality_u8_int64_batch(col_name: &str, n_values: usize) -> RecordBatch {
        assert!(n_values <= 255);
        let keys = UInt8Array::from((0..n_values).map(|i| i as u8).collect::<Vec<_>>());
        let values: Arc<dyn Array> = Arc::new(Int64Array::from(
            (0..n_values).map(|i| i as i64).collect::<Vec<_>>(),
        ));
        let dict_array = DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new(
            col_name,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int64)),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap()
    }

    /// Create a batch with a real struct field (e.g. "body") whose named sub-field
    /// is a Dict(u8, Utf8) column with low cardinality.
    fn create_struct_with_u8_dict_batch(
        struct_name: &str,
        field_name: &str,
        n_values: usize,
    ) -> RecordBatch {
        assert!(n_values <= 255);
        let keys = UInt8Array::from((0..n_values).map(|i| i as u8).collect::<Vec<_>>());
        let values: Arc<dyn Array> = Arc::new(StringArray::from(
            (0..n_values)
                .map(|i| format!("val_{}", i))
                .collect::<Vec<_>>(),
        ));
        let dict_field = Field::new(
            field_name,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        );
        let dict_array = DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap();
        let struct_array = StructArray::from(vec![(
            Arc::new(dict_field.clone()),
            Arc::new(dict_array) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            struct_name,
            DataType::Struct(vec![dict_field].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Scenario: schema selection over an empty batch iterator with a real
    /// payload spec.
    /// Guarantees: an index with no batches yields an empty schema and reports a
    /// batch count of zero rather than erroring.
    #[test]
    fn test_empty_iterator() {
        let records: Vec<Option<&RecordBatch>> = vec![];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_eq!(index.batch_count, 0);
        assert_eq!(schema.schema.fields().len(), 0);
    }

    /// Scenario: schema selection when every batch slot is `None`.
    /// Guarantees: `None` slots are skipped, producing an empty schema and a zero
    /// batch count.
    #[test]
    fn test_none_batches() {
        let records: Vec<Option<&RecordBatch>> = vec![None, None, None];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_eq!(index.batch_count, 0);
        assert_eq!(schema.schema.fields().len(), 0);
    }

    /// Scenario: a single LogAttrs batch carrying all required columns is indexed
    /// and selected.
    /// Guarantees: every present column appears once in the selected schema, in
    /// payload-spec declaration order.
    #[test]
    fn test_single_batch() {
        let batch = create_log_attrs_batch();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let actual = select_schema(&index).unwrap();

        let names: Vec<&str> = actual
            .schema
            .fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect();
        assert_eq!(names, vec!["parent_id", "key", "type"]);
    }

    /// Scenario: two identical LogAttrs batches are indexed together.
    /// Guarantees: the shared columns are emitted exactly once and marked
    /// non-nullable because they are present in every batch.
    #[test]
    fn test_same_fields() {
        let batch1 = create_log_attrs_batch();
        let batch2 = create_log_attrs_batch();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let actual = select_schema(&index).unwrap();

        let names: Vec<&str> = actual
            .schema
            .fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect();
        assert_eq!(names, vec!["parent_id", "key", "type"]);
        for field in actual.schema.fields() {
            assert!(
                !field.is_nullable(),
                "field {} present in all batches must be non-nullable",
                field.name()
            );
        }
    }

    /// Scenario: real LogAttrs columns appear in different subsets across three
    /// batches (parent_id in all, str/int in some).
    /// Guarantees: the union of columns is emitted; columns absent from some
    /// batch are nullable while the ever-present column is not.
    #[test]
    fn test_mixed_fields_nullability() {
        let batch1 = record_batch!((ATTRIBUTE_STR, (UInt8, Utf8), ([0, 1], ["a", "b"]))).unwrap();
        let batch2 = record_batch!((ATTRIBUTE_INT, (UInt8, Int64), ([0, 1], [10i64, 20]))).unwrap();
        let batch3 = record_batch!(
            (ATTRIBUTE_STR, (UInt8, Utf8), ([0, 1], ["a", "b"])),
            (ATTRIBUTE_INT, (UInt8, Int64), ([0, 1], [10i64, 20]))
        )
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let actual = select_schema(&index).unwrap();

        // str is missing from batch2, int is missing from batch1: both nullable.
        let str_field = actual.schema.field_with_name(ATTRIBUTE_STR).unwrap();
        let int_field = actual.schema.field_with_name(ATTRIBUTE_INT).unwrap();
        assert!(str_field.is_nullable());
        assert!(int_field.is_nullable());
    }

    /// Scenario: one column (key) is present in all batches while other columns
    /// each appear in only one.
    /// Guarantees: the common column is non-nullable and the per-batch-unique
    /// columns are nullable, all emitted in spec order.
    #[test]
    fn test_multiple_batches_one_common_field() {
        let batch1 = record_batch!(
            (ATTRIBUTE_KEY, (UInt8, Utf8), ([0, 1], ["a", "b"])),
            (ATTRIBUTE_STR, (UInt8, Utf8), ([0, 1], ["a", "b"]))
        )
        .unwrap();
        let batch2 = record_batch!(
            (ATTRIBUTE_KEY, (UInt8, Utf8), ([0, 1], ["a", "b"])),
            (ATTRIBUTE_INT, (UInt8, Int64), ([0, 1], [10i64, 20]))
        )
        .unwrap();
        let batch3 = record_batch!(
            (ATTRIBUTE_KEY, (UInt8, Utf8), ([0, 1], ["a", "b"])),
            (ATTRIBUTE_SER, (UInt8, Utf8), ([0, 1], ["a", "b"]))
        )
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let actual = select_schema(&index).unwrap();

        let key_field = actual.schema.field_with_name(ATTRIBUTE_KEY).unwrap();
        assert!(!key_field.is_nullable(), "key present in all batches");
        assert!(
            actual
                .schema
                .field_with_name(ATTRIBUTE_STR)
                .unwrap()
                .is_nullable()
        );
        assert!(
            actual
                .schema
                .field_with_name(ATTRIBUTE_INT)
                .unwrap()
                .is_nullable()
        );
        assert!(
            actual
                .schema
                .field_with_name(ATTRIBUTE_SER)
                .unwrap()
                .is_nullable()
        );
    }

    /// Scenario: the same real dictionary column (str) is carried with u8 keys in
    /// two batches and u16 keys in a third.
    /// Guarantees: physical-value-count selection keeps the low-cardinality
    /// column at the spec-required u16 key without erroring on mixed input keys.
    #[test]
    fn test_cardinality_mixed_key_types() {
        let batch1 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 3);
        let batch2 = {
            let keys = UInt16Array::from(vec![0u16, 1]);
            let values: Arc<dyn Array> =
                Arc::new(StringArray::from(vec!["val_0".to_string(), "val_9".into()]));
            create_dict_batch(ATTRIBUTE_STR, keys, values, DataType::Utf8)
        };
        let batch3 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 2);

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let actual = select_schema(&index).unwrap();

        let field = actual.schema.field_with_name(ATTRIBUTE_STR).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
        );
    }

    /// Scenario: Two FixedSizeBinary dictionaries overlap enough for their distinct
    /// values to fit u16, but their summed physical value arrays exceed the u16 limit.
    /// Guarantees: Concatenation falls back to plain values instead of overflowing
    /// Arrow's dictionary keys or panicking.
    #[test]
    fn test_overlapping_fsb_dictionaries_fall_back_to_plain() {
        let count = (MAX_U16_CARDINALITY / 2) + 1;
        let overlap = count / 2;
        let batch1 = create_fsb_dictionary_batch(0, count);
        let batch2 = create_fsb_dictionary_batch(overlap, count);

        let batch = concat_logs_root(batch1, batch2);

        assert_eq!(batch.num_rows(), 2 * count);
        let trace_id = batch.schema().field_with_name(TRACE_ID).unwrap().clone();
        assert_eq!(trace_id.data_type(), &DataType::FixedSizeBinary(16));
        let column = batch.column_by_name(TRACE_ID).expect("trace_id column");
        assert_overlapping_fsb_values(column, count, overlap);
    }

    /// Scenario: A Binary dictionary nested in the Logs "body" struct has
    /// overlapping values whose summed physical length exceeds the u16 limit.
    /// Guarantees: Nested dictionary selection uses the same physical bound and
    /// concatenation produces a plain nested field without panicking.
    #[test]
    fn test_nested_overlapping_dictionaries_fall_back_to_plain() {
        let count = (MAX_U16_CARDINALITY / 2) + 1;
        let overlap = count / 2;
        let batch1 = create_struct_binary_dictionary_batch(0, count);
        let batch2 = create_struct_binary_dictionary_batch(overlap, count);

        let batch = concat_logs_root(batch1, batch2);
        let schema = batch.schema();
        let body = schema.field_with_name(BODY).unwrap();
        let DataType::Struct(fields) = body.data_type() else {
            panic!("expected struct field");
        };

        assert_eq!(batch.num_rows(), 2 * count);
        let ser_field = fields
            .iter()
            .find(|f| f.name() == ATTRIBUTE_SER)
            .expect("ser child");
        assert_eq!(ser_field.data_type(), &DataType::Binary);
    }

    /// Scenario: Dictionary value arrays contain one more physical slot than the
    /// u8 limit, but only that limit's number of non-null values, in a real
    /// FixedSizeBinary(8) column (span_id).
    /// Guarantees: Key selection counts the null slot and selects u16 rather than
    /// underestimating the physical dictionary length as fitting u8.
    #[test]
    fn test_dictionary_physical_bound_includes_null_slots() {
        let physical_count = MAX_U8_CARDINALITY + 1;
        let first_count = physical_count / 2;
        let second_count = physical_count - first_count;
        let raw1 = (0..first_count)
            .map(|i| (i as u64).to_le_bytes())
            .collect::<Vec<_>>();
        let values1 = FixedSizeBinaryArray::try_from_sparse_iter_with_size(
            raw1.iter()
                .enumerate()
                .map(|(i, value)| (i != 0).then_some(value.as_slice())),
            8,
        )
        .unwrap();
        let raw2 = (first_count..physical_count)
            .map(|i| (i as u64).to_le_bytes())
            .collect::<Vec<_>>();
        let values2 =
            FixedSizeBinaryArray::try_from_iter(raw2.iter().map(|value| value.as_slice())).unwrap();
        let keys1 = UInt8Array::from((0..first_count).map(|i| i as u8).collect::<Vec<_>>());
        let keys2 = UInt8Array::from((0..second_count).map(|i| i as u8).collect::<Vec<_>>());
        let batch1 = create_dict_batch(
            SPAN_ID,
            keys1,
            Arc::new(values1),
            DataType::FixedSizeBinary(8),
        );
        let batch2 = create_dict_batch(
            SPAN_ID,
            keys2,
            Arc::new(values2),
            DataType::FixedSizeBinary(8),
        );

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(SPAN_ID).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt16),
                Box::new(DataType::FixedSizeBinary(8))
            )
        );
    }

    /// Scenario: LogAttrs "str" column arrives as low-cardinality Dict(u8, Utf8)
    /// in every batch, but the spec requires a u16 minimum key.
    /// Guarantees: schema selection upgrades the key to u16 despite the physical
    /// value count fitting in u8.
    #[test]
    fn test_u16_attrs_str_column_enforces_u16_key() {
        let batch1 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 10);
        let batch2 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(ATTRIBUTE_STR).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            "U16 attrs 'str' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    /// Scenario: ResourceAttrs "int" column arrives as low-cardinality
    /// Dict(u8, Int64), but the spec requires a u16 minimum key.
    /// Guarantees: schema selection upgrades the key to u16.
    #[test]
    fn test_u16_attrs_int_column_enforces_u16_key() {
        let batch1 = create_low_cardinality_u8_int64_batch(ATTRIBUTE_INT, 10);
        let batch2 = create_low_cardinality_u8_int64_batch(ATTRIBUTE_INT, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(ResourceAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(ATTRIBUTE_INT).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
            "U16 attrs 'int' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    /// Scenario: the 32-bit-parent attribute schema (SpanEventAttrs) "str" column
    /// arrives as low-cardinality Dict(u8, Utf8).
    /// Guarantees: the u16 minimum key requirement holds for the 32-bit-parent
    /// attribute schema too.
    #[test]
    fn test_u32_attrs_str_column_enforces_u16_key() {
        let batch1 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 10);
        let batch2 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_STR, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(SpanEventAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(ATTRIBUTE_STR).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            "U32 attrs 'str' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    /// Scenario: the Logs "body" struct carries a low-cardinality Dict(u8, Utf8)
    /// "str" child.
    /// Guarantees: the u16 minimum key requirement is enforced on struct children.
    #[test]
    fn test_logs_body_str_enforces_u16_key() {
        let batch1 = create_struct_with_u8_dict_batch(BODY, ATTRIBUTE_STR, 10);
        let batch2 = create_struct_with_u8_dict_batch(BODY, ATTRIBUTE_STR, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let body_field = schema.schema.field_with_name(BODY).unwrap();
        if let DataType::Struct(fields) = body_field.data_type() {
            let str_field = fields
                .iter()
                .find(|f| f.name() == ATTRIBUTE_STR)
                .expect("str field should exist in body struct");
            assert_eq!(
                str_field.data_type(),
                &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                "LOGS body.str must use Dict(u16) key, got {:?}",
                str_field.data_type()
            );
        } else {
            panic!(
                "Expected body to be a struct, got {:?}",
                body_field.data_type()
            );
        }
    }

    /// Scenario: the Logs "body" struct carries a low-cardinality Dict(u8, Utf8)
    /// "ser" child whose spec value type is Binary.
    /// Guarantees: struct-child selection upgrades the key to u16 while keeping
    /// the incoming Utf8 value type recorded from the batch.
    #[test]
    fn test_logs_body_ser_enforces_u16_key() {
        let batch1 = create_struct_with_u8_dict_batch(BODY, ATTRIBUTE_SER, 10);
        let batch2 = create_struct_with_u8_dict_batch(BODY, ATTRIBUTE_SER, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let body_field = schema.schema.field_with_name(BODY).unwrap();
        if let DataType::Struct(fields) = body_field.data_type() {
            let ser_field = fields
                .iter()
                .find(|f| f.name() == ATTRIBUTE_SER)
                .expect("ser field should exist in body struct");
            assert_eq!(
                ser_field.data_type(),
                &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                "LOGS body.ser must use Dict(u16) key, got {:?}",
                ser_field.data_type()
            );
        } else {
            panic!(
                "Expected body to be a struct, got {:?}",
                body_field.data_type()
            );
        }
    }

    /// Scenario: LogAttrs "key" column arrives as low-cardinality Dict(u8, Utf8),
    /// and the spec permits a u8 minimum key.
    /// Guarantees: schema selection leaves the key at u8 rather than upgrading.
    #[test]
    fn test_attrs_key_column_allows_u8() {
        let batch1 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_KEY, 10);
        let batch2 = create_low_cardinality_u8_utf8_batch(ATTRIBUTE_KEY, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(ATTRIBUTE_KEY).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            "Attrs 'key' column may use Dict(u8), got {:?}",
            field.data_type()
        );
    }

    /// Scenario: the Logs "severity_text" column arrives as low-cardinality
    /// Dict(u8, Utf8), and the spec permits a u8 minimum key.
    /// Guarantees: schema selection leaves the key at u8.
    #[test]
    fn test_logs_severity_text_allows_u8() {
        let batch1 = create_low_cardinality_u8_utf8_batch(SEVERITY_TEXT, 10);
        let batch2 = create_low_cardinality_u8_utf8_batch(SEVERITY_TEXT, 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(SEVERITY_TEXT).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            "LOGS severity_text may use Dict(u8), got {:?}",
            field.data_type()
        );
    }

    /// Scenario: LogAttrs "type" column (spec: plain UInt8) is delivered
    /// dictionary-encoded as Dict(u8, UInt8).
    /// Guarantees: schema selection strips the dictionary and emits native UInt8
    /// because the spec column is not dictionary-encodable.
    #[test]
    fn test_non_dict_column_strips_dictionary() {
        let keys = UInt8Array::from(vec![0u8, 1, 0, 1]);
        let values: Arc<dyn Array> = Arc::new(UInt8Array::from(vec![1u8, 2]));
        let dict_array = DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new(
            ATTRIBUTE_TYPE,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt8)),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap();

        let keys2 = UInt8Array::from(vec![0u8, 1]);
        let values2: Arc<dyn Array> = Arc::new(UInt8Array::from(vec![3u8, 4]));
        let dict_array2 = DictionaryArray::<UInt8Type>::try_new(keys2, values2).unwrap();
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            ATTRIBUTE_TYPE,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt8)),
            false,
        )]));
        let batch2 = RecordBatch::try_new(schema2, vec![Arc::new(dict_array2)]).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let schema = select_schema(&index).unwrap();

        let field = schema.schema.field_with_name(ATTRIBUTE_TYPE).unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::UInt8,
            "'type' column should be native UInt8, got {:?}",
            field.data_type()
        );
    }

    /// Build a minimal spec-valid LogAttrs batch carrying its required columns
    /// (parent_id, key, type).
    fn create_log_attrs_batch() -> RecordBatch {
        let parent_id: ArrayRef = Arc::new(UInt16Array::from(vec![0u16, 1]));
        let key_keys = UInt8Array::from(vec![0u8, 1]);
        let key_values: Arc<dyn Array> =
            Arc::new(StringArray::from(vec!["k0".to_string(), "k1".into()]));
        let key: ArrayRef =
            Arc::new(DictionaryArray::<UInt8Type>::try_new(key_keys, key_values).unwrap());
        let type_col: ArrayRef = Arc::new(UInt8Array::from(vec![1u8, 2]));

        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, false),
            Field::new(
                ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new(ATTRIBUTE_TYPE, DataType::UInt8, false),
        ]));
        RecordBatch::try_new(schema, vec![parent_id, key, type_col]).unwrap()
    }

    /// Build a root-Logs batch whose "body" struct carries the required "type"
    /// child plus one optional value child. `value_child` is (name, array,
    /// arrow_type) so callers can vary which optional body column is present.
    fn logs_body_batch(rows: usize, value_child: (&str, ArrayRef, DataType)) -> RecordBatch {
        let (child_name, child_array, child_dt) = value_child;
        let type_array: ArrayRef =
            Arc::new(UInt8Array::from((0..rows).map(|_| 1u8).collect::<Vec<_>>()));
        let type_field = Field::new(ATTRIBUTE_TYPE, DataType::UInt8, false);
        let value_field = Field::new(child_name, child_dt, false);
        let struct_array = StructArray::from(vec![
            (Arc::new(type_field.clone()), type_array),
            (Arc::new(value_field.clone()), child_array),
        ]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            BODY,
            DataType::Struct(vec![type_field, value_field].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Scenario: two root-Logs batches carry the "body" struct with divergent
    /// optional children -- a string body ({type, str}) and an integer body
    /// ({type, int}) -- and are concatenated end to end through the Logs signal
    /// path.
    /// Guarantees: concatenation succeeds (no ColumnDataTypeMismatch), the unified
    /// body struct is the union {type, str, int} in spec order, row counts sum,
    /// and each optional child is null-padded for the batch that lacked it.
    #[test]
    fn test_concatenate_divergent_body_struct_children() {
        // batch1: string body -> body.str present, body.int absent.
        let str_keys = UInt8Array::from(vec![0u8, 1]);
        let str_values: Arc<dyn Array> =
            Arc::new(StringArray::from(vec!["a".to_string(), "b".into()]));
        let str_array: ArrayRef =
            Arc::new(DictionaryArray::<UInt8Type>::try_new(str_keys, str_values).unwrap());
        let batch1 = logs_body_batch(
            2,
            (
                ATTRIBUTE_STR,
                str_array,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            ),
        );

        // batch2: integer body -> body.int present, body.str absent.
        let int_keys = UInt8Array::from(vec![0u8, 1, 0]);
        let int_values: Arc<dyn Array> = Arc::new(Int64Array::from(vec![10i64, 20]));
        let int_array: ArrayRef =
            Arc::new(DictionaryArray::<UInt8Type>::try_new(int_keys, int_values).unwrap());
        let batch2 = logs_body_batch(
            3,
            (
                ATTRIBUTE_INT,
                int_array,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int64)),
            ),
        );

        let batch = concat_logs_root(batch1, batch2);

        assert_eq!(batch.num_rows(), 5, "row counts should sum");

        let body = batch.schema().field_with_name(BODY).unwrap().clone();
        let DataType::Struct(fields) = body.data_type() else {
            panic!("expected body struct, got {:?}", body.data_type());
        };
        let names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
        assert_eq!(
            names,
            vec![ATTRIBUTE_TYPE, ATTRIBUTE_STR, ATTRIBUTE_INT],
            "unified body struct should be the union of children in spec order"
        );

        let struct_array = batch
            .column_by_name(BODY)
            .unwrap()
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("body struct array");

        // str present for the first two rows (batch1), null-padded for the last
        // three (batch2 lacked it).
        let str_col = struct_array.column_by_name(ATTRIBUTE_STR).unwrap();
        assert_eq!(str_col.null_count(), 3, "str null-padded for int-body rows");
        assert!(str_col.is_valid(0) && str_col.is_valid(1));
        assert!(str_col.is_null(2) && str_col.is_null(3) && str_col.is_null(4));

        // int present for the last three rows (batch2), null-padded for the first
        // two (batch1 lacked it).
        let int_col = struct_array.column_by_name(ATTRIBUTE_INT).unwrap();
        assert_eq!(int_col.null_count(), 2, "int null-padded for str-body rows");
        assert!(int_col.is_null(0) && int_col.is_null(1));
        assert!(int_col.is_valid(2) && int_col.is_valid(3) && int_col.is_valid(4));
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::{LogAttrs, Logs};
    use crate::record_batch;
    use crate::schema::consts::{
        ATTRIBUTE_INT, ATTRIBUTE_STR, BODY, DROPPED_ATTRIBUTES_COUNT, FLAGS, SEVERITY_TEXT,
    };
    use arrow::array::{Int32Array, StructArray, UInt16Array, UInt32Array};
    use arrow::datatypes::Int32Type;
    use std::sync::Arc;

    /// Look up the indexed field for a spec column by name, panicking if absent.
    fn indexed_field<'a>(index: &'a RecordIndex<'a>, name: &str) -> &'a IndexedField<'a> {
        let slot = index.fields.schema.slot_of(name).expect("column in spec");
        index.fields.slots[slot]
            .as_ref()
            .unwrap_or_else(|| panic!("field '{}' missing from index", name))
    }

    /// Build a Logs "body" struct batch with a single Int32 "str"-ish child so we
    /// can force a struct-vs-non-struct collision on the real "body" column.
    fn logs_body_struct_batch() -> RecordBatch {
        let child = Field::new(ATTRIBUTE_STR, DataType::Utf8, true);
        let struct_array = StructArray::from(vec![(
            Arc::new(child.clone()),
            Arc::new(arrow::array::StringArray::from(vec!["a", "b", "c"])) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            BODY,
            DataType::Struct(vec![child].into()),
            true,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Scenario: the real Logs "body" column is a struct in one batch and a
    /// primitive in another.
    /// Guarantees: index_records surfaces a clean ColumnDataTypeMismatch rather
    /// than panicking during later casting.
    #[test]
    fn test_struct_to_non_struct_mismatch() {
        let batch1 = logs_body_struct_batch();
        let batch2 = record_batch!((BODY, Int32, [1, 2, 3])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(Logs));

        match result {
            Err(Error::ColumnDataTypeMismatch { name, actual, .. }) => {
                assert_eq!(name, BODY);
                assert_eq!(actual, DataType::Int32);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real Logs "body" column is a primitive in the first batch and
    /// a struct in a later batch.
    /// Guarantees: index_records surfaces a ColumnDataTypeMismatch naming the
    /// struct as the divergent type.
    #[test]
    fn test_non_struct_to_struct_mismatch() {
        let batch1 = record_batch!((BODY, Int32, [1, 2, 3])).unwrap();
        let batch2 = logs_body_struct_batch();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(Logs));

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, BODY);
                assert_eq!(expect, DataType::Int32);
                assert!(matches!(actual, DataType::Struct { .. }));
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real LogAttrs "str" column is a dictionary of Int64 values in
    /// one batch and Utf8 values in another.
    /// Guarantees: divergent dictionary value types are reported as
    /// ColumnDataTypeMismatch (the new spec-indexed path compares value types).
    #[test]
    fn test_dictionary_value_type_mismatch() {
        let batch1 = record_batch!((
            ATTRIBUTE_STR,
            (UInt8, Int64),
            ([0, 1, 2], [100i64, 200, 300])
        ))
        .unwrap();
        let batch2 = record_batch!((
            ATTRIBUTE_STR,
            (UInt8, Utf8),
            ([0, 1, 2], ["foo", "bar", "baz"])
        ))
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(LogAttrs));

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, ATTRIBUTE_STR);
                assert_eq!(expect, DataType::Int64);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real LogAttrs "str" dictionary column arrives with an
    /// unsupported Int32 key type.
    /// Guarantees: index_records rejects the key type with
    /// UnsupportedDictionaryKeyType listing the allowed u8/u16 keys.
    #[test]
    fn test_unsupported_dictionary_key_type() {
        let keys = Int32Array::from(vec![0, 1, 2]);
        let values = Arc::new(arrow::array::StringArray::from(vec!["foo", "bar", "baz"]));
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, values).unwrap();

        let schema = Arc::new(Schema::new(vec![Field::new(
            ATTRIBUTE_STR,
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            true,
        )]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap();

        let records = vec![Some(&batch)];
        let result = index_records(records.into_iter(), payloads::get(LogAttrs));

        match result {
            Err(Error::UnsupportedDictionaryKeyType {
                expect_oneof,
                actual,
            }) => {
                assert_eq!(expect_oneof, vec![DataType::UInt8, DataType::UInt16]);
                assert_eq!(actual, DataType::Int32);
            }
            _ => panic!(
                "Expected UnsupportedDictionaryKeyType error, got: {:?}",
                result
            ),
        }
    }

    /// Scenario: the real Logs "flags" column carries Int32 in one batch and Int64
    /// in another.
    /// Guarantees: primitive value-type divergence is reported as
    /// ColumnDataTypeMismatch.
    #[test]
    fn test_primitive_type_mismatch() {
        let batch1 = record_batch!((FLAGS, Int32, [1, 2, 3])).unwrap();
        let batch2 = record_batch!((FLAGS, Int64, [4, 5, 6])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(Logs));

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, FLAGS);
                assert_eq!(expect, DataType::Int32);
                assert_eq!(actual, DataType::Int64);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real LogAttrs "str" column is a dictionary of Int64 in the
    /// first batch and a plain Utf8 column in the next.
    /// Guarantees: the mismatch between the dictionary value type and the plain
    /// type is reported as ColumnDataTypeMismatch.
    #[test]
    fn test_dictionary_to_primitive_mismatch() {
        let batch1 = record_batch!((
            ATTRIBUTE_STR,
            (UInt8, Int64),
            ([0, 1, 2], [100i64, 200, 300])
        ))
        .unwrap();
        let batch2 = record_batch!((ATTRIBUTE_STR, Utf8, ["foo", "bar", "baz"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(LogAttrs));

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, ATTRIBUTE_STR);
                assert_eq!(expect, DataType::Int64);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real LogAttrs "str" column is a plain Int64 column in the
    /// first batch and a Utf8 dictionary in the next.
    /// Guarantees: the divergence is reported as ColumnDataTypeMismatch (the new
    /// path no longer emits a distinct DictionaryValueTypeMismatch here).
    #[test]
    fn test_primitive_to_dictionary_mismatch() {
        let batch1 = record_batch!((ATTRIBUTE_STR, Int64, [100i64, 200, 300])).unwrap();
        let batch2 = record_batch!((
            ATTRIBUTE_STR,
            (UInt8, Utf8),
            ([0, 1, 2], ["foo", "bar", "baz"])
        ))
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter(), payloads::get(LogAttrs));

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, ATTRIBUTE_STR);
                assert_eq!(expect, DataType::Int64);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    /// Scenario: the real LogAttrs "int" column arrives plain Int64 in one batch
    /// and as a Dict(u8, Int64) in another, with matching value types.
    /// Guarantees: index_records accepts the pair and schema selection emits a
    /// dictionary (u16 per the "int" spec minimum) rather than erroring.
    #[test]
    fn test_primitive_to_dictionary_upgrade_success() {
        let batch1 = record_batch!((ATTRIBUTE_INT, Int64, [100i64, 200, 300])).unwrap();
        let batch2 = record_batch!((
            ATTRIBUTE_INT,
            (UInt8, Int64),
            ([0, 1, 2], [100i64, 400, 500])
        ))
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter(), payloads::get(LogAttrs)).unwrap();
        let selected = select_schema(&index).unwrap();

        let field = selected.schema.field_with_name(ATTRIBUTE_INT).unwrap();
        assert!(
            matches!(
                field.data_type(),
                DataType::Dictionary(k, v) if **k == DataType::UInt16 && **v == DataType::Int64
            ),
            "Expected Dictionary(UInt16, Int64), got {:?}",
            field.data_type()
        );
    }

    /// Scenario: real Logs columns appear in different subsets of batches, some
    /// with null values and some slots wholly absent (None entries).
    /// Guarantees: the spec-indexed IndexedField accumulates present_count,
    /// total_physical_value_count, value_type, is_dictionary, and nullability
    /// exactly as the concatenation planner relies on.
    #[test]
    fn test_index_fields_with_mixed_types_and_none_batches() {
        // batch0: severity_text (dict u8, 3 rows), flags (3 rows)
        let batch0 = record_batch!(
            (
                SEVERITY_TEXT,
                (UInt8, Utf8),
                ([0, 1, 2], ["ok", "warn", "err"])
            ),
            (FLAGS, UInt32, [10u32, 20, 30])
        )
        .unwrap();

        // batch2: severity_text (dict u16, 2 rows), flags (2 rows, one null),
        // dropped_attributes_count (2 rows)
        let sev_keys = UInt16Array::from(vec![0u16, 1]);
        let sev_values: Arc<dyn Array> = Arc::new(arrow::array::StringArray::from(vec![
            "ok".to_string(),
            "warn".into(),
        ]));
        let sev = Arc::new(DictionaryArray::<UInt16Type>::try_new(sev_keys, sev_values).unwrap());
        let flags2: ArrayRef = Arc::new(UInt32Array::from(vec![Some(5u32), None]));
        let dropped2: ArrayRef = Arc::new(UInt32Array::from(vec![1u32, 2]));
        let schema2 = Arc::new(Schema::new(vec![
            Field::new(
                SEVERITY_TEXT,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new(FLAGS, DataType::UInt32, true),
            Field::new(DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]));
        let batch2 = RecordBatch::try_new(schema2, vec![sev, flags2, dropped2]).unwrap();

        // batch3: severity_text (dict u8, 4 rows)
        let batch3 = record_batch!((
            SEVERITY_TEXT,
            (UInt8, Utf8),
            ([0, 1, 2, 3], ["ok", "warn", "err", "dbg"])
        ))
        .unwrap();

        let records = vec![
            Some(&batch0),
            None,
            Some(&batch2),
            None,
            Some(&batch3),
            None,
        ];
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        assert_eq!(index.batch_count, 3, "batch_count mismatch");

        // severity_text: present in all 3 batches, dictionary, Utf8 values.
        // Physical value counts: 3 + 2 + 4 = 9 across the dictionaries.
        let sev = indexed_field(&index, SEVERITY_TEXT);
        assert_eq!(sev.value_type, &DataType::Utf8);
        assert!(sev.is_dictionary);
        assert_eq!(sev.present_count, 3);
        assert_eq!(sev.total_physical_value_count, 9);
        assert!(
            !sev.nullable,
            "severity_text present in every batch, no nulls"
        );

        // flags: present in batches 0 and 2 only -> nullable (absent from batch3),
        // and additionally has a null value in batch2.
        let flags = indexed_field(&index, FLAGS);
        assert_eq!(flags.value_type, &DataType::UInt32);
        assert!(!flags.is_dictionary);
        assert_eq!(flags.present_count, 2);
        assert_eq!(flags.total_physical_value_count, 5);
        assert!(flags.nullable);

        // dropped_attributes_count: present in batch2 only -> nullable.
        let dropped = indexed_field(&index, DROPPED_ATTRIBUTES_COUNT);
        assert_eq!(dropped.value_type, &DataType::UInt32);
        assert_eq!(dropped.present_count, 1);
        assert_eq!(dropped.total_physical_value_count, 2);
        assert!(dropped.nullable);
    }
}

#[cfg(test)]
mod nullability_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs;
    use crate::record_batch;
    use crate::schema::consts::{
        DROPPED_ATTRIBUTES_COUNT, FLAGS, ID, NAME, RESOURCE, SCHEMA_URL, SCOPE, SEVERITY_TEXT,
        VERSION,
    };
    use arrow::array::{StructArray, UInt16Array, UInt32Array};
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    /// Assert a top-level field's nullability in the selected schema.
    fn assert_field_nullable(schema: &Schema, field_name: &str, expected_nullable: bool) {
        let field = schema
            .field_with_name(field_name)
            .unwrap_or_else(|_| panic!("Field '{}' not found in schema", field_name));
        assert_eq!(
            field.is_nullable(),
            expected_nullable,
            "Field '{}' nullability mismatch: expected {}, got {}",
            field_name,
            expected_nullable,
            field.is_nullable()
        );
    }

    /// Select the unified Logs schema from the given batches.
    fn select_logs(records: Vec<Option<&RecordBatch>>) -> Schema {
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        select_schema(&index).unwrap().schema
    }

    /// Build a Logs "resource" struct batch carrying the named u16 "id" child and,
    /// optionally, a u32 "dropped_attributes_count" child.
    fn resource_struct_batch(children: &[&str]) -> RecordBatch {
        let mut fields = Vec::new();
        let mut arrays: Vec<ArrayRef> = Vec::new();
        for &name in children {
            let (dt, array): (DataType, ArrayRef) = match name {
                ID => (DataType::UInt16, Arc::new(UInt16Array::from(vec![1u16, 2]))),
                DROPPED_ATTRIBUTES_COUNT => (
                    DataType::UInt32,
                    Arc::new(UInt32Array::from(vec![10u32, 20])),
                ),
                SCHEMA_URL => {
                    let keys = arrow::array::UInt8Array::from(vec![0u8, 1]);
                    let values: Arc<dyn Array> =
                        Arc::new(arrow::array::StringArray::from(vec!["u0", "u1"]));
                    (
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        Arc::new(DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap()),
                    )
                }
                other => panic!("unsupported resource child '{}'", other),
            };
            fields.push(Field::new(name, dt, false));
            arrays.push(array);
        }
        let struct_fields: Vec<(Arc<Field>, ArrayRef)> =
            fields.iter().cloned().map(Arc::new).zip(arrays).collect();
        let struct_array = StructArray::from(struct_fields);
        let schema = Arc::new(Schema::new(vec![Field::new(
            RESOURCE,
            DataType::Struct(fields.into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Build a Logs "scope" struct batch carrying the named children.
    fn scope_struct_batch(children: &[&str]) -> RecordBatch {
        let mut fields = Vec::new();
        let mut arrays: Vec<ArrayRef> = Vec::new();
        for &name in children {
            let (dt, array): (DataType, ArrayRef) = match name {
                ID => (DataType::UInt16, Arc::new(UInt16Array::from(vec![1u16]))),
                DROPPED_ATTRIBUTES_COUNT => {
                    (DataType::UInt32, Arc::new(UInt32Array::from(vec![10u32])))
                }
                NAME | VERSION => {
                    let keys = arrow::array::UInt8Array::from(vec![0u8]);
                    let values: Arc<dyn Array> =
                        Arc::new(arrow::array::StringArray::from(vec!["x"]));
                    (
                        DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                        Arc::new(DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap()),
                    )
                }
                other => panic!("unsupported scope child '{}'", other),
            };
            fields.push(Field::new(name, dt, false));
            arrays.push(array);
        }
        let struct_fields: Vec<(Arc<Field>, ArrayRef)> =
            fields.iter().cloned().map(Arc::new).zip(arrays).collect();
        let struct_array = StructArray::from(struct_fields);
        let schema = Arc::new(Schema::new(vec![Field::new(
            SCOPE,
            DataType::Struct(fields.into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Scenario: the real Logs "severity_text" column is present in two batches and
    /// absent from a third, while "id" is present in all three.
    /// Guarantees: a column missing from some batch is nullable, and a column
    /// present in every batch stays non-nullable.
    #[test]
    fn test_field_nullable_when_missing_in_some_batches() {
        let batch1 = record_batch!(
            (ID, UInt16, [1u16, 2]),
            (SEVERITY_TEXT, (UInt8, Utf8), ([0, 1], ["a", "b"]))
        )
        .unwrap();
        let batch2 = record_batch!((ID, UInt16, [3u16, 4])).unwrap();
        let batch3 = record_batch!(
            (ID, UInt16, [5u16, 6]),
            (SEVERITY_TEXT, (UInt8, Utf8), ([0, 1], ["c", "d"]))
        )
        .unwrap();

        let schema = select_logs(vec![Some(&batch1), Some(&batch2), Some(&batch3)]);

        assert_field_nullable(&schema, SEVERITY_TEXT, true);
        assert_field_nullable(&schema, ID, false);
    }

    /// Scenario: the real Logs "flags" column has a null value in one batch.
    /// Guarantees: a column containing a null value is marked nullable even when
    /// present in every batch.
    #[test]
    fn test_field_nullable_with_null_values_in_array() {
        let flags1: ArrayRef = Arc::new(UInt32Array::from(vec![Some(1u32), Some(2)]));
        let flags2: ArrayRef = Arc::new(UInt32Array::from(vec![Some(3u32), None]));
        let schema_def = Arc::new(Schema::new(vec![Field::new(FLAGS, DataType::UInt32, true)]));
        let batch1 = RecordBatch::try_new(schema_def.clone(), vec![flags1]).unwrap();
        let batch2 = RecordBatch::try_new(schema_def, vec![flags2]).unwrap();

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        assert_field_nullable(&schema, FLAGS, true);
    }

    /// Scenario: real Logs columns appear in different subsets across three
    /// batches (id in all, flags in two, severity_text in two).
    /// Guarantees: only the ever-present column is non-nullable; the others are
    /// nullable.
    #[test]
    fn test_multiple_fields_nullable_combinations() {
        let batch1 = record_batch!(
            (ID, UInt16, [1u16, 2]),
            (FLAGS, UInt32, [3u32, 4]),
            (SEVERITY_TEXT, (UInt8, Utf8), ([0, 1], ["a", "b"]))
        )
        .unwrap();
        let batch2 = record_batch!((ID, UInt16, [7u16, 8]), (FLAGS, UInt32, [9u32, 10])).unwrap();
        let batch3 = record_batch!(
            (ID, UInt16, [11u16, 12]),
            (SEVERITY_TEXT, (UInt8, Utf8), ([0, 1], ["c", "d"]))
        )
        .unwrap();

        let schema = select_logs(vec![Some(&batch1), Some(&batch2), Some(&batch3)]);

        assert_field_nullable(&schema, ID, false);
        assert_field_nullable(&schema, FLAGS, true);
        assert_field_nullable(&schema, SEVERITY_TEXT, true);
    }

    /// Scenario: the real Logs "resource" struct is present in two batches and
    /// absent from a third that carries only "id".
    /// Guarantees: the struct column and its children are nullable when the struct
    /// is missing from some batch.
    #[test]
    fn test_struct_field_nullable_when_struct_missing_from_batches() {
        let batch1 = resource_struct_batch(&[ID]);
        let batch2 = record_batch!((ID, UInt16, [1u16, 2])).unwrap();
        let batch3 = resource_struct_batch(&[ID]);

        let schema = select_logs(vec![Some(&batch1), Some(&batch2), Some(&batch3)]);

        assert_field_nullable(&schema, RESOURCE, true);
        assert_field_nullable(&schema, ID, true);

        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            let id_field = fields
                .iter()
                .find(|f| f.name() == ID)
                .expect("id child should exist");
            assert!(
                id_field.is_nullable(),
                "resource.id should be nullable when parent struct missing from batches"
            );
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: a single Logs batch carries a "resource" struct with two present
    /// children.
    /// Guarantees: struct-child selection emits every present child in the unified
    /// struct type.
    #[test]
    fn test_struct_field_nullability_basic() {
        let batch = resource_struct_batch(&[ID, DROPPED_ATTRIBUTES_COUNT]);

        let schema = select_logs(vec![Some(&batch)]);

        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");
            assert!(fields.iter().any(|f| f.name() == ID));
            assert!(fields.iter().any(|f| f.name() == DROPPED_ATTRIBUTES_COUNT));
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: the same "resource" struct with the same two children appears in
    /// two batches.
    /// Guarantees: struct children present in every instance stay non-nullable and
    /// are emitted once.
    #[test]
    fn test_struct_fields_accumulated_across_batches() {
        let batch1 = resource_struct_batch(&[ID, DROPPED_ATTRIBUTES_COUNT]);
        let batch2 = resource_struct_batch(&[ID, DROPPED_ATTRIBUTES_COUNT]);

        let index = index_records(
            vec![Some(&batch1), Some(&batch2)].into_iter(),
            payloads::get(Logs),
        )
        .unwrap();

        // The resource field should have been indexed from both batches.
        let slot = index.fields.schema.slot_of(RESOURCE).unwrap();
        let resource_info = index.fields.slots[slot].as_ref().expect("resource indexed");
        assert_eq!(resource_info.present_count, 2);
        let struct_index = resource_info
            .struct_index
            .as_ref()
            .expect("resource has struct children");
        for child in struct_index.slots.iter().flatten() {
            assert_eq!(
                child.present_count, 2,
                "each present struct child should be seen in both batches"
            );
        }

        let schema = select_schema(&index).unwrap().schema;
        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");
            let id_field = fields.iter().find(|f| f.name() == ID).expect("id exists");
            let dropped_field = fields
                .iter()
                .find(|f| f.name() == DROPPED_ATTRIBUTES_COUNT)
                .expect("dropped exists");
            assert!(!id_field.is_nullable());
            assert!(!dropped_field.is_nullable());
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: the "scope" struct carries {id, dropped_attributes_count} in one
    /// batch and {id, name} in another, i.e. divergent subsets of its optional
    /// children.
    /// Guarantees: struct children are unified across batches -- the selected
    /// struct is the union {id, dropped_attributes_count, name}, with the
    /// ever-present child non-nullable and children absent from some batch made
    /// nullable.
    #[test]
    fn test_struct_children_unified_across_batches() {
        let batch1 = scope_struct_batch(&[ID, DROPPED_ATTRIBUTES_COUNT]);
        let batch2 = scope_struct_batch(&[ID, NAME]);

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        let scope = schema.field_with_name(SCOPE).unwrap();
        if let DataType::Struct(fields) = scope.data_type() {
            let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
            assert_eq!(field_names.len(), 3, "expected union of children");
            assert!(field_names.contains(&ID));
            assert!(field_names.contains(&DROPPED_ATTRIBUTES_COUNT));
            assert!(field_names.contains(&NAME));

            let id_field = fields.iter().find(|f| f.name() == ID).expect("id exists");
            assert!(!id_field.is_nullable(), "id present in every scope struct");

            let dropped_field = fields
                .iter()
                .find(|f| f.name() == DROPPED_ATTRIBUTES_COUNT)
                .expect("dropped exists");
            let name_field = fields
                .iter()
                .find(|f| f.name() == NAME)
                .expect("name exists");
            assert!(dropped_field.is_nullable(), "dropped absent from batch2");
            assert!(name_field.is_nullable(), "name absent from batch1");
        } else {
            panic!("Expected Struct type for 'scope' field");
        }
    }

    /// Scenario: two "scope" structs with the same children but a null value in
    /// one child of one batch.
    /// Guarantees: within a consistent struct shape, a null child value makes that
    /// child nullable while its non-null sibling stays non-nullable.
    #[test]
    fn test_struct_child_nullability_from_null_values() {
        let batch1 =
            scope_struct_batch_with_nulls(&[(ID, false), (DROPPED_ATTRIBUTES_COUNT, false)]);
        let batch2 =
            scope_struct_batch_with_nulls(&[(ID, false), (DROPPED_ATTRIBUTES_COUNT, true)]);

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        let scope = schema.field_with_name(SCOPE).unwrap();
        if let DataType::Struct(fields) = scope.data_type() {
            let id_field = fields.iter().find(|f| f.name() == ID).expect("id exists");
            let dropped_field = fields
                .iter()
                .find(|f| f.name() == DROPPED_ATTRIBUTES_COUNT)
                .expect("dropped exists");
            assert!(!id_field.is_nullable(), "id has no null values");
            assert!(dropped_field.is_nullable(), "dropped had a null value");
        } else {
            panic!("Expected Struct type for 'scope' field");
        }
    }

    /// Build a "scope" struct where each child is present but may carry a null
    /// value in its single row.
    fn scope_struct_batch_with_nulls(children: &[(&str, bool)]) -> RecordBatch {
        let mut fields = Vec::new();
        let mut arrays: Vec<ArrayRef> = Vec::new();
        for &(name, null_value) in children {
            let (dt, array): (DataType, ArrayRef) = match name {
                ID => {
                    let v = if null_value { None } else { Some(1u16) };
                    (DataType::UInt16, Arc::new(UInt16Array::from(vec![v])))
                }
                DROPPED_ATTRIBUTES_COUNT => {
                    let v = if null_value { None } else { Some(10u32) };
                    (DataType::UInt32, Arc::new(UInt32Array::from(vec![v])))
                }
                other => panic!("unsupported scope child '{}'", other),
            };
            fields.push(Field::new(name, dt, true));
            arrays.push(array);
        }
        let struct_fields: Vec<(Arc<Field>, ArrayRef)> =
            fields.iter().cloned().map(Arc::new).zip(arrays).collect();
        let struct_array = StructArray::from(struct_fields);
        let schema = Arc::new(Schema::new(vec![Field::new(
            SCOPE,
            DataType::Struct(fields.into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::{Logs, SpanEvents};
    use crate::record_batch;
    use crate::schema::consts::metadata::COLUMN_ENCODING;
    use crate::schema::consts::metadata::encodings::PLAIN;
    use crate::schema::consts::{DROPPED_ATTRIBUTES_COUNT, ID, NAME, PARENT_ID, RESOURCE};
    use arrow::array::{StructArray, UInt16Array, UInt32Array};
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    /// Assert a field carries (or lacks) the expected metadata value.
    fn assert_field_metadata(
        schema: &Schema,
        field_name: &str,
        key: &str,
        expected_value: Option<&str>,
    ) {
        let field = schema
            .field_with_name(field_name)
            .unwrap_or_else(|_| panic!("Field '{}' not found in schema", field_name));
        let actual_value = field.metadata().get(key);
        match (actual_value, expected_value) {
            (Some(actual), Some(expected)) => {
                assert_eq!(
                    actual, expected,
                    "Field '{}' metadata '{}' mismatch: expected '{}', got '{}'",
                    field_name, key, expected, actual
                );
            }
            (None, None) => {}
            (Some(actual), None) => {
                panic!(
                    "Field '{}' has unexpected metadata '{}' = '{}'",
                    field_name, key, actual
                );
            }
            (None, Some(expected)) => {
                panic!(
                    "Field '{}' missing expected metadata '{}' = '{}'",
                    field_name, key, expected
                );
            }
        }
    }

    fn select_span_events(records: Vec<Option<&RecordBatch>>) -> Schema {
        let index = index_records(records.into_iter(), payloads::get(SpanEvents)).unwrap();
        select_schema(&index).unwrap().schema
    }

    /// Scenario: the real SpanEvents "id" column (spec UInt32) is concatenated.
    /// Guarantees: the "id" column is stamped with the PLAIN column-encoding
    /// metadata that downstream transport decoding relies on.
    #[test]
    fn test_metadata_added_to_id_field() {
        let batch1 = record_batch!((ID, UInt32, [1u32, 2])).unwrap();
        let batch2 = record_batch!((ID, UInt32, [3u32, 4])).unwrap();

        let schema = select_span_events(vec![Some(&batch1), Some(&batch2)]);

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
    }

    /// Scenario: the real SpanEvents "parent_id" column (spec UInt16) is
    /// concatenated.
    /// Guarantees: the "parent_id" column is stamped with the PLAIN encoding
    /// metadata.
    #[test]
    fn test_metadata_added_to_parent_id_field() {
        let batch1 = record_batch!((PARENT_ID, UInt16, [0u16, 1])).unwrap();
        let batch2 = record_batch!((PARENT_ID, UInt16, [2u16, 3])).unwrap();

        let schema = select_span_events(vec![Some(&batch1), Some(&batch2)]);

        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }

    /// Scenario: the real SpanEvents "name" and "dropped_attributes_count" columns
    /// are concatenated.
    /// Guarantees: non-ID columns carry no column-encoding metadata.
    #[test]
    fn test_metadata_not_added_to_regular_fields() {
        let batch1 = record_batch!(
            (NAME, (UInt8, Utf8), ([0, 1], ["a", "b"])),
            (DROPPED_ATTRIBUTES_COUNT, UInt32, [1u32, 2])
        )
        .unwrap();
        let batch2 = record_batch!(
            (NAME, (UInt8, Utf8), ([0, 1], ["c", "d"])),
            (DROPPED_ATTRIBUTES_COUNT, UInt32, [3u32, 4])
        )
        .unwrap();

        let schema = select_span_events(vec![Some(&batch1), Some(&batch2)]);

        assert_field_metadata(&schema, NAME, COLUMN_ENCODING, None);
        assert_field_metadata(&schema, DROPPED_ATTRIBUTES_COUNT, COLUMN_ENCODING, None);
    }

    /// Scenario: the real SpanEvents "id" column (spec UInt32, not dictionary
    /// encodable) arrives dictionary-encoded.
    /// Guarantees: the dictionary is stripped to native UInt32 and the "id" column
    /// still receives the PLAIN encoding metadata.
    #[test]
    fn test_metadata_added_to_dictionary_id_field() {
        let batch1 = record_batch!((ID, (UInt8, UInt32), ([0, 1], [100u32, 200]))).unwrap();
        let batch2 = record_batch!((ID, (UInt8, UInt32), ([0, 1], [300u32, 400]))).unwrap();

        let schema = select_span_events(vec![Some(&batch1), Some(&batch2)]);

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
        assert_eq!(
            schema.field_with_name(ID).unwrap().data_type(),
            &DataType::UInt32,
            "spec-plain id column should be stripped to native UInt32"
        );
    }

    /// Scenario: the real Logs "resource" struct contains an "id" child alongside
    /// a regular child.
    /// Guarantees: struct children named "id" get the PLAIN encoding metadata
    /// while regular struct children do not.
    #[test]
    fn test_metadata_added_to_struct_id_fields() {
        let id_field = Field::new(ID, DataType::UInt16, false);
        let dropped_field = Field::new(DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, false);
        let struct_array = StructArray::from(vec![
            (
                Arc::new(id_field.clone()),
                Arc::new(UInt16Array::from(vec![1u16, 2])) as ArrayRef,
            ),
            (
                Arc::new(dropped_field.clone()),
                Arc::new(UInt32Array::from(vec![10u32, 20])) as ArrayRef,
            ),
        ]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            RESOURCE,
            DataType::Struct(vec![id_field, dropped_field].into()),
            false,
        )]));
        let batch = RecordBatch::try_new(schema1, vec![Arc::new(struct_array)]).unwrap();

        let index = index_records(vec![Some(&batch)].into_iter(), payloads::get(Logs)).unwrap();
        let schema = select_schema(&index).unwrap().schema;

        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            let id_field = fields
                .iter()
                .find(|f| f.name() == ID)
                .expect("id field should exist");
            assert_eq!(
                id_field.metadata().get(COLUMN_ENCODING),
                Some(&PLAIN.to_string()),
                "struct subfield 'id' should have PLAIN encoding metadata"
            );

            let dropped_field = fields
                .iter()
                .find(|f| f.name() == DROPPED_ATTRIBUTES_COUNT)
                .expect("dropped field should exist");
            assert_eq!(
                dropped_field.metadata().get(COLUMN_ENCODING),
                None,
                "regular struct child should not have encoding metadata"
            );
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: the real SpanEvents schema carries both "id" and "parent_id".
    /// Guarantees: both ID-like columns receive the PLAIN encoding metadata.
    #[test]
    fn test_metadata_on_both_id_and_parent_id() {
        let batch1 =
            record_batch!((PARENT_ID, UInt16, [0u16, 1]), (ID, UInt32, [1u32, 2])).unwrap();
        let batch2 =
            record_batch!((PARENT_ID, UInt16, [2u16, 3]), (ID, UInt32, [3u32, 4])).unwrap();

        let schema = select_span_events(vec![Some(&batch1), Some(&batch2)]);

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }
}

#[cfg(test)]
mod struct_field_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs;
    use crate::record_batch;
    use crate::schema::consts::{ID, RESOURCE, SCHEMA_URL, SCOPE};
    use arrow::array::{DictionaryArray, StringArray, StructArray, UInt8Array, UInt16Array};
    use arrow::datatypes::UInt8Type;
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    fn select_logs(records: Vec<Option<&RecordBatch>>) -> Schema {
        let index = index_records(records.into_iter(), payloads::get(Logs)).unwrap();
        select_schema(&index).unwrap().schema
    }

    /// Build a Logs "resource" struct whose "schema_url" child is a Dict(u8, Utf8)
    /// with the given values.
    fn resource_with_schema_url(values: Vec<&str>) -> RecordBatch {
        let key_array = UInt8Array::from((0..values.len() as u8).collect::<Vec<_>>());
        let value_array = Arc::new(StringArray::from(values));
        let dict_array = DictionaryArray::<UInt8Type>::new(key_array, value_array);

        let child = Field::new(
            SCHEMA_URL,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        );
        let struct_array = StructArray::from(vec![(
            Arc::new(child.clone()),
            Arc::new(dict_array) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            RESOURCE,
            DataType::Struct(vec![child].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Build a Logs "resource" struct whose "id" child is a plain UInt16.
    fn resource_with_id(values: Vec<u16>) -> RecordBatch {
        let child = Field::new(ID, DataType::UInt16, false);
        let struct_array = StructArray::from(vec![(
            Arc::new(child.clone()),
            Arc::new(UInt16Array::from(values)) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            RESOURCE,
            DataType::Struct(vec![child].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    /// Scenario: the Logs "resource" struct carries a "schema_url" dictionary child
    /// whose spec permits a u8 key.
    /// Guarantees: struct-child dictionary selection keeps the u8 key type.
    #[test]
    fn test_struct_with_dictionary_field_u8() {
        let batch1 = resource_with_schema_url(vec!["a", "b"]);
        let batch2 = resource_with_schema_url(vec!["c", "d"]);

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            let schema_url = fields
                .iter()
                .find(|f| f.name() == SCHEMA_URL)
                .expect("schema_url field should exist");
            assert!(
                matches!(
                    schema_url.data_type(),
                    DataType::Dictionary(k, v) if **k == DataType::UInt8 && **v == DataType::Utf8
                ),
                "Expected Dictionary(UInt8, Utf8), got {:?}",
                schema_url.data_type()
            );
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: the Logs "resource" struct carries a plain UInt16 "id" child in
    /// two batches.
    /// Guarantees: struct-child selection emits the native primitive type.
    #[test]
    fn test_struct_with_primitive_field() {
        let batch1 = resource_with_id(vec![1, 2, 3]);
        let batch2 = resource_with_id(vec![4, 5, 6]);

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        let resource = schema.field_with_name(RESOURCE).unwrap();
        if let DataType::Struct(fields) = resource.data_type() {
            let id_field = fields
                .iter()
                .find(|f| f.name() == ID)
                .expect("id field should exist");
            assert_eq!(
                id_field.data_type(),
                &DataType::UInt16,
                "Expected UInt16 type"
            );
        } else {
            panic!("Expected Struct type for 'resource' field");
        }
    }

    /// Scenario: the Logs "resource" struct is present in one batch and absent from
    /// another that carries only "id".
    /// Guarantees: the struct column is nullable when missing from some batch.
    #[test]
    fn test_struct_completely_missing_from_batch() {
        let batch1 = resource_with_id(vec![1, 2]);
        let batch2 = record_batch!((ID, UInt16, [3u16, 4])).unwrap();

        let schema = select_logs(vec![Some(&batch1), Some(&batch2)]);

        let resource = schema.field_with_name(RESOURCE).unwrap();
        assert!(
            resource.is_nullable(),
            "Struct should be nullable when missing from some batches"
        );
    }

    /// Scenario: a single Logs batch carries both the "resource" and "scope"
    /// struct columns.
    /// Guarantees: multiple struct columns are each emitted in the selected schema.
    #[test]
    fn test_multiple_struct_fields_in_schema() {
        let resource_child = Field::new(ID, DataType::UInt16, false);
        let resource_array = StructArray::from(vec![(
            Arc::new(resource_child.clone()),
            Arc::new(UInt16Array::from(vec![1u16, 2])) as ArrayRef,
        )]);

        let scope_child = Field::new(ID, DataType::UInt16, false);
        let scope_array = StructArray::from(vec![(
            Arc::new(scope_child.clone()),
            Arc::new(UInt16Array::from(vec![3u16, 4])) as ArrayRef,
        )]);

        let schema = Arc::new(Schema::new(vec![
            Field::new(
                RESOURCE,
                DataType::Struct(vec![resource_child].into()),
                false,
            ),
            Field::new(SCOPE, DataType::Struct(vec![scope_child].into()), false),
        ]));
        let batch = RecordBatch::try_new(
            schema,
            vec![Arc::new(resource_array), Arc::new(scope_array)],
        )
        .unwrap();

        let schema = select_logs(vec![Some(&batch)]);

        assert!(schema.field_with_name(RESOURCE).is_ok());
        assert!(schema.field_with_name(SCOPE).is_ok());
    }
}

#[cfg(test)]
mod write_tests {
    use super::*;
    use arrow::array::{
        BinaryArray, Float64Array, ListArray, StringArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::Int32Type as I32;

    fn plan_all() -> InputPlan {
        InputPlan::default()
    }

    fn plan_ranges(ranges: Vec<Range<usize>>) -> InputPlan {
        InputPlan {
            selection: Selection::Ranges(ranges),
            ..Default::default()
        }
    }

    fn plan_remap(col: IdCol, remap: AnyRemap) -> InputPlan {
        let mut plan = InputPlan::default();
        plan.remaps[col as usize] = Some(remap);
        plan
    }

    fn dict_u8(keys: Vec<Option<u8>>, values: ArrayRef) -> ArrayRef {
        Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(keys),
            values,
        ))
    }

    fn dict_u16(keys: Vec<Option<u16>>, values: ArrayRef) -> ArrayRef {
        Arc::new(DictionaryArray::<UInt16Type>::new(
            UInt16Array::from(keys),
            values,
        ))
    }

    fn write(
        target: DataType,
        cols: &[(Option<ArrayRef>, usize, InputPlan)],
        id_col: Option<IdCol>,
    ) -> ArrayRef {
        let field = Field::new("f", target, true);
        let inputs: Vec<Input<'_>> = cols
            .iter()
            .map(|(c, n, p)| Input {
                column: c.as_ref(),
                num_rows: *n,
                plan: p,
            })
            .collect();
        let rows = inputs.iter().map(Input::selected_rows).sum();
        let out = write_column(&field, &inputs, rows, id_col).unwrap();
        assert_eq!(out.len(), rows);
        assert_eq!(out.data_type(), field.data_type());
        out.to_data().validate_full().unwrap();
        out
    }

    /// Decode any (possibly dictionary) column to its logical value array.
    fn logical(array: &ArrayRef) -> ArrayRef {
        match array.data_type() {
            DataType::Dictionary(_, v) => cast(array.as_ref(), v).unwrap(),
            _ => Arc::clone(array),
        }
    }

    fn utf8(v: Vec<Option<&str>>) -> ArrayRef {
        Arc::new(StringArray::from(v))
    }

    /// Scenario: every source encoding (native, Dict(u8), Dict(u16), missing)
    /// is written into each target encoding (native, Dict(u8), Dict(u16)) for
    /// a Utf8 column.
    /// Guarantees: the logical values and nulls of the output match the
    /// concatenation of the inputs regardless of input/output encoding.
    #[test]
    fn test_encoding_matrix_utf8() {
        let native = utf8(vec![Some("a"), None, Some("b")]);
        let d8 = dict_u8(
            vec![Some(1), None, Some(0)],
            utf8(vec![Some("x"), Some("y")]),
        );
        let d16 = dict_u16(vec![Some(0), Some(0)], utf8(vec![Some("z")]));
        let expected = utf8(vec![
            Some("a"),
            None,
            Some("b"),
            Some("y"),
            None,
            Some("x"),
            Some("z"),
            Some("z"),
            None,
            None,
        ]);

        for target in [
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
        ] {
            let out = write(
                target.clone(),
                &[
                    (Some(native.clone()), 3, plan_all()),
                    (Some(d8.clone()), 3, plan_all()),
                    (Some(d16.clone()), 2, plan_all()),
                    (None, 2, plan_all()),
                ],
                None,
            );
            assert_eq!(logical(&out).as_ref(), expected.as_ref(), "{target:?}");
        }
    }

    /// Scenario: primitive, boolean, binary and fixed size binary columns with
    /// sliced (non-zero offset) sources and nulls are concatenated.
    /// Guarantees: writers honor array offsets for both values and null
    /// buffers.
    #[test]
    fn test_sliced_sources() {
        let prim: ArrayRef = Arc::new(Float64Array::from(vec![
            Some(0.0),
            None,
            Some(2.0),
            Some(3.0),
            None,
        ]));
        let out = write(
            DataType::Float64,
            &[
                (Some(prim.slice(1, 3)), 3, plan_all()),
                (Some(prim.slice(3, 2)), 2, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(Float64Array::from(vec![
            None,
            Some(2.0),
            Some(3.0),
            Some(3.0),
            None,
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let b: ArrayRef = Arc::new(BooleanArray::from(vec![
            Some(true),
            None,
            Some(false),
            Some(true),
            Some(true),
            Some(false),
            None,
            Some(true),
            Some(false),
            Some(true),
        ]));
        let out = write(
            DataType::Boolean,
            &[
                (Some(b.slice(3, 7)), 7, plan_all()),
                (Some(b.slice(1, 3)), 3, plan_all()),
            ],
            None,
        );
        let mut expected: Vec<Option<bool>> = b.as_boolean().slice(3, 7).iter().collect();
        expected.extend(b.as_boolean().slice(1, 3).iter());
        let expected: ArrayRef = Arc::new(BooleanArray::from(expected));
        assert_eq!(out.as_ref(), expected.as_ref());

        let bin: ArrayRef = Arc::new(BinaryArray::from(vec![
            Some(&b"aa"[..]),
            None,
            Some(b"ccc"),
            Some(b""),
            Some(b"e"),
        ]));
        let out = write(
            DataType::Binary,
            &[
                (Some(bin.slice(2, 3)), 3, plan_all()),
                (Some(bin.slice(0, 2)), 2, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(BinaryArray::from(vec![
            Some(&b"ccc"[..]),
            Some(b""),
            Some(b"e"),
            Some(b"aa"),
            None,
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let fsb: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                vec![Some([1u8, 1]), None, Some([3, 3]), Some([4, 4])].into_iter(),
                2,
            )
            .unwrap(),
        );
        let out = write(
            DataType::FixedSizeBinary(2),
            &[
                (Some(fsb.slice(1, 3)), 3, plan_all()),
                (None, 1, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                vec![None, Some([3u8, 3]), Some([4, 4]), None].into_iter(),
                2,
            )
            .unwrap(),
        );
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: inputs carry `Selection::Ranges` for native, dictionary, and
    /// struct columns.
    /// Guarantees: only rows inside the selected ranges are written, in order,
    /// and nulls follow their rows.
    #[test]
    fn test_selection_ranges() {
        let native: ArrayRef = Arc::new(UInt8Array::from(vec![
            Some(0),
            Some(1),
            None,
            Some(3),
            Some(4),
        ]));
        let out = write(
            DataType::UInt8,
            &[(Some(native.clone()), 5, plan_ranges(vec![0..1, 2..4]))],
            None,
        );
        let expected: ArrayRef = Arc::new(UInt8Array::from(vec![Some(0), None, Some(3)]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let dict = dict_u8(
            vec![Some(0), Some(1), None, Some(0)],
            utf8(vec![Some("a"), Some("b")]),
        );
        for target in [
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
        ] {
            let out = write(
                target,
                &[(
                    Some(dict.clone()),
                    4,
                    plan_ranges(std::iter::once(1..3).collect()),
                )],
                None,
            );
            assert_eq!(logical(&out).as_ref(), utf8(vec![Some("b"), None]).as_ref());
        }

        let child = Field::new("c", DataType::UInt8, true);
        let s: ArrayRef = Arc::new(StructArray::new(
            vec![child.clone()].into(),
            vec![native],
            Some(NullBuffer::from(vec![true, false, true, true, true])),
        ));
        let target = DataType::Struct(vec![child].into());
        let out = write(target, &[(Some(s), 5, plan_ranges(vec![1..2, 3..5]))], None);
        let out = out.as_struct();
        assert!(out.is_null(0));
        assert!(out.is_valid(1));
        let expected: ArrayRef = Arc::new(UInt8Array::from(vec![Some(1), Some(3), Some(4)]));
        assert_eq!(out.column(0).as_ref(), expected.as_ref());
    }

    /// Scenario: a struct target has children that are absent from some
    /// inputs, and one input lacks the struct entirely.
    /// Guarantees: missing children and missing structs are written as nulls
    /// of the correct length without affecting present data.
    #[test]
    fn test_missing_struct_children() {
        let a = Field::new("a", DataType::Int32, true);
        let b = Field::new("b", DataType::Utf8, true);
        let s1: ArrayRef = Arc::new(StructArray::new(
            vec![a.clone()].into(),
            vec![Arc::new(PrimitiveArray::<I32>::from(vec![1, 2]))],
            None,
        ));
        let s2: ArrayRef = Arc::new(StructArray::new(
            vec![b.clone()].into(),
            vec![utf8(vec![Some("x")])],
            None,
        ));
        let target = DataType::Struct(vec![a, b].into());
        let out = write(
            target,
            &[
                (Some(s1), 2, plan_all()),
                (Some(s2), 1, plan_all()),
                (None, 2, plan_all()),
            ],
            None,
        );
        let out = out.as_struct();
        assert_eq!(out.null_count(), 2);
        let expected_a: ArrayRef = Arc::new(PrimitiveArray::<I32>::from(vec![
            Some(1),
            Some(2),
            None,
            None,
            None,
        ]));
        assert_eq!(out.column(0).as_ref(), expected_a.as_ref());
        assert_eq!(
            out.column(1).as_ref(),
            utf8(vec![None, None, Some("x"), None, None]).as_ref()
        );
    }

    /// Scenario: inputs without nulls are written; then a null appears only in
    /// the last input.
    /// Guarantees: no null buffer is allocated when no input has nulls, and
    /// earlier rows are backfilled as valid when the first null appears late.
    #[test]
    fn test_lazy_nulls() {
        let a: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2]));
        let out = write(
            DataType::UInt32,
            &[
                (Some(a.clone()), 2, plan_all()),
                (Some(a.clone()), 2, plan_all()),
            ],
            None,
        );
        assert!(out.nulls().is_none());

        let b: ArrayRef = Arc::new(UInt32Array::from(vec![None, Some(9)]));
        let out = write(
            DataType::UInt32,
            &[(Some(a), 2, plan_all()), (Some(b), 2, plan_all())],
            None,
        );
        let expected: ArrayRef = Arc::new(UInt32Array::from(vec![Some(1), Some(2), None, Some(9)]));
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: ID remaps (Offset including wraparound on a null slot, and
    /// Replace) are applied to a native u16 id column and a dictionary u32
    /// parent_id column.
    /// Guarantees: remaps apply to values (dictionary values for dictionary
    /// columns) in both native and dictionary output, null slots never panic,
    /// and nulls are preserved.
    #[test]
    fn test_id_remaps() {
        let ids: ArrayRef = Arc::new(UInt16Array::from(vec![Some(5), None, Some(6)]));
        let out = write(
            DataType::UInt16,
            &[
                (
                    Some(ids.clone()),
                    3,
                    plan_remap(
                        IdCol::Id,
                        AnyRemap::U16(IdRemap::Offset(0u16.wrapping_sub(5))),
                    ),
                ),
                (
                    Some(ids),
                    3,
                    plan_remap(
                        IdCol::Id,
                        AnyRemap::U16(IdRemap::Replace(vec![10u16, 0, 11].into())),
                    ),
                ),
            ],
            Some(IdCol::Id),
        );
        let expected: ArrayRef = Arc::new(UInt16Array::from(vec![
            Some(0),
            None,
            Some(1),
            Some(10),
            None,
            Some(11),
        ]));
        assert_eq!(out.as_ref(), expected.as_ref());

        let values: ArrayRef = Arc::new(UInt32Array::from(vec![100, 200]));
        let pid = dict_u8(vec![Some(1), Some(0), Some(1)], values);
        let plan = plan_remap(IdCol::ParentId, AnyRemap::U32(IdRemap::Offset(7)));
        let expected: ArrayRef = Arc::new(UInt32Array::from(vec![207, 107, 207]));
        for target in [
            DataType::UInt32,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32)),
        ] {
            let out = write(
                target,
                &[(Some(pid.clone()), 3, plan.clone())],
                Some(IdCol::ParentId),
            );
            assert_eq!(logical(&out).as_ref(), expected.as_ref());
        }
    }

    /// Scenario: a remap is present in the plan for `id` but the column being
    /// written is a non-ID column of the same type.
    /// Guarantees: remaps are only applied to the ID column they target.
    #[test]
    fn test_remap_not_applied_to_other_columns() {
        let a: ArrayRef = Arc::new(UInt32Array::from(vec![1, 2]));
        let out = write(
            DataType::UInt32,
            &[(
                Some(a.clone()),
                2,
                plan_remap(IdCol::Id, AnyRemap::U32(IdRemap::Offset(10))),
            )],
            None,
        );
        assert_eq!(out.as_ref(), a.as_ref());
    }

    /// Scenario: List columns are concatenated via the generic fallback with
    /// a selection and a missing input.
    /// Guarantees: the fallback honors selections and null-pads missing
    /// inputs.
    #[test]
    fn test_list_fallback() {
        let list: ArrayRef = Arc::new(ListArray::from_iter_primitive::<Float64Type, _, _>(vec![
            Some(vec![Some(1.0)]),
            Some(vec![Some(2.0), Some(3.0)]),
            None,
        ]));
        let target = list.data_type().clone();
        let out = write(
            target,
            &[
                (Some(list), 3, plan_ranges(std::iter::once(1..3).collect())),
                (None, 1, plan_all()),
            ],
            None,
        );
        let expected: ArrayRef =
            Arc::new(ListArray::from_iter_primitive::<Float64Type, _, _>(vec![
                Some(vec![Some(2.0), Some(3.0)]),
                None,
                None,
            ]));
        assert_eq!(out.as_ref(), expected.as_ref());
    }

    /// Scenario: dictionary inputs whose keys contain sequential runs,
    /// descending and repeated keys, null keys with out-of-range values, and
    /// a sliced key array are gathered into native Utf8 and FixedSizeBinary
    /// outputs, with and without a row selection.
    /// Guarantees: run-coalesced gathering produces exactly the logical
    /// values of the inputs, in row order, with nulls preserved.
    #[test]
    fn test_dict_gather_runs() {
        let values: Vec<String> = (0..12).map(|i| format!("v{i}{}", "x".repeat(i))).collect();
        let str_values: ArrayRef = Arc::new(StringArray::from(values.clone()));
        let fsb_values: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_iter((0u8..12).map(|i| [i, i.wrapping_mul(7)])).unwrap(),
        );

        // Runs [2,3,4], [9], descending [8,7], repeats [5,5], a null key that
        // holds an out-of-range value, then a run to the end [10,11].
        let key_data: Vec<Option<u8>> = vec![
            Some(2),
            Some(3),
            Some(4),
            Some(9),
            Some(8),
            Some(7),
            Some(5),
            Some(5),
            None,
            Some(10),
            Some(11),
        ];
        let mut keys = UInt8Array::from(key_data.clone());
        // Give the null slot an out-of-range key value.
        let (dt, mut raw, nulls) = keys.into_parts();
        let mut v = raw.to_vec();
        v[8] = 200;
        raw = v.into();
        keys = UInt8Array::new(raw, nulls).with_data_type(dt);

        for (values, target) in [
            (str_values, DataType::Utf8),
            (fsb_values, DataType::FixedSizeBinary(2)),
        ] {
            let dict: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
                keys.clone(),
                values.clone(),
            ));
            let expected_full = cast(dict.as_ref(), &target).unwrap();

            // Whole input.
            let out = write(
                target.clone(),
                &[(Some(dict.clone()), 11, plan_all())],
                None,
            );
            assert_eq!(out.as_ref(), expected_full.as_ref(), "{target:?} full");

            // Sliced keys (non-zero key offset) plus a selection.
            let sliced = dict.slice(1, 9);
            let out = write(
                target.clone(),
                &[(Some(sliced.clone()), 9, plan_ranges(vec![0..3, 5..9]))],
                None,
            );
            let exp = cast(sliced.as_ref(), &target).unwrap();
            let exp = arrow::compute::concat(&[&exp.slice(0, 3), &exp.slice(5, 4)]).unwrap();
            assert_eq!(out.as_ref(), exp.as_ref(), "{target:?} sliced");
        }
    }

    /// Scenario: a dictionary source has a null in its values array that is
    /// referenced by a valid key, gathered into a native output.
    /// Guarantees: the row is null in the output.
    #[test]
    fn test_dict_null_value_gather() {
        let d = dict_u8(vec![Some(0), Some(1)], utf8(vec![None, Some("a")]));
        let out = write(DataType::Utf8, &[(Some(d), 2, plan_all())], None);
        assert_eq!(out.as_ref(), utf8(vec![None, Some("a")]).as_ref());
    }
}
