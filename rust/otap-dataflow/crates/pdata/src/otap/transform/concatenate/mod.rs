// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
use std::sync::Arc;

pub(crate) mod plan;
mod write;

use crate::otap::transform::reindex;

use plan::InputPlan;
use write::Input;

use crate::error::Error;
use crate::otap::{Logs, Metrics, OtapBatchStore, Result, Traces};
use crate::schema::consts::metadata::COLUMN_ENCODING;
use crate::schema::consts::metadata::encodings::PLAIN;
use crate::schema::consts::{ID, PARENT_ID};
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
        let id_col = write::top_level_id_col(target.name());
        columns.push(write::write_column(target, &inputs, rows, id_col)?);
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
    fn create_dict_batch<K: arrow::datatypes::ArrowDictionaryKeyType>(
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
