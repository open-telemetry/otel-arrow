// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{Array, ArrayRef, DictionaryArray, RecordBatch, StructArray};
use arrow::compute::kernels::cast;
use arrow::datatypes::{UInt8Type, UInt16Type};
use arrow_schema::{DataType, Field, FieldRef, Fields, Schema, SchemaBuilder};
use std::sync::Arc;

use crate::error::Error;
use crate::otap::transform::cardinality::{MAX_U8_CARDINALITY, MAX_U16_CARDINALITY};
use crate::otap::{Logs, Metrics, OtapBatchStore, Result, Traces};
use crate::schema::consts::metadata::COLUMN_ENCODING;
use crate::schema::consts::metadata::encodings::PLAIN;
use crate::schema::consts::{ID, PARENT_ID};
use crate::schema::payloads;
use crate::schema::schema::{DictKeySize, Field as SchemaField, Schema as PayloadSchema};

/// Concatenate the provided OtapArrowRecords into a single batch.
///
/// # Preconditions
///
/// Currently the caller is responsible for satisfying the following:
///
///   1. Remove the transport optimized encodings from the columns, if any
///   2. Reindex the ID columns so that the parent child relationships are
///      consistent after the concatenation
///
/// These will be handled internally in the future as we refine the API, see
/// https://github.com/open-telemetry/otel-arrow/issues/1926.
///
/// # General Algorithm
///
/// Concatenating multiple OtapArrowRecords involves three steps:
///
///   1. Reindexing the ID columns so that the parent child relationships are
///      consistent after the concatenation
///   2. Selecting a common schema and converting every record batch to that
///      schema. This includes several steps:
///         * Indexing all fields for the same ArrowPayloadType across every batch
///         * Selecting a safe key type for each dictionary field from the
///           physical number of dictionary values that Arrow may concatenate.
///         * Determining nullability for each field in the final batch
///   3. Casting every record batch to the final schema, including casting individual
///      arrays as well as reordering the columns to match the schema.
///
/// # Future optimizations
///
/// - TODO: Re-indexing probably should not be a separate operation. We should decide
///   within this function whether or not to do it and ensure it happens if required.
///   This is deferred until we totally remove the old implementation in groups.rs
///   due to interface incompatibility.
///
/// - TODO: Consider using new_unchecked for record batch construction if we're
///   confident in it. We mostly unwrap those operations a lot, so skipping the
///   checks or moving similar checks to debug asserts may be reasonable.
pub fn concatenate<const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
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

    match N {
        Logs::COUNT => concatenate_signal::<Logs, N>(items),
        Metrics::COUNT => concatenate_signal::<Metrics, N>(items),
        Traces::COUNT => concatenate_signal::<Traces, N>(items),
        // FIXME: This is a hack for now to avoid having to rewrite a lot of
        // the tests. We can make the tests a lot better now that we have payload
        // definitions.
        _ => concatenate_without_spec(items),
    }
}

fn concatenate_without_spec<const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
    concatenate_with_def(items, |_| &PayloadSchema::EMPTY)
}

fn concatenate_signal<S: OtapBatchStore, const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
    concatenate_with_def(items, |i| {
        let payload_type = S::payload_type_at_idx(i);
        payloads::get(payload_type)
    })
}

fn concatenate_with_def<const N: usize>(
    items: &mut [[Option<RecordBatch>; N]],
    get_def: impl Fn(usize) -> &'static PayloadSchema,
) -> Result<[Option<RecordBatch>; N]> {
    let mut result = [const { None }; N];

    #[allow(clippy::needless_range_loop)]
    for i in 0..N {
        let payload_def = get_def(i);

        let index = index_records(select_all(items, i), payload_def)?;
        if index.batch_count == 0 {
            continue;
        }

        let selected = select_schema(&index)?;
        let new_schema: Arc<Schema> = Arc::from(selected.schema);
        let mut batcher = arrow::compute::BatchCoalescer::new(new_schema.clone(), index.row_count);
        for payload in select_all_mut(items, i) {
            let Some(rb) = payload.take() else {
                continue;
            };

            let (curr_schema, columns, num_rows) = rb.into_parts();
            let converted_columns = convert(
                columns,
                num_rows,
                &curr_schema.fields,
                &new_schema.fields,
                payload_def,
                &selected.slot_to_target,
            )?;

            // safety: Unless we have a bug, we've satisfied all the preconditions
            // for try_new and push_batch by converting everything to a unified
            // schema.
            let converted = RecordBatch::try_new(new_schema.clone(), converted_columns)
                .expect("Valid construction");
            batcher
                .push_batch(converted)
                .map_err(|source| Error::Batching { source })?;
        }

        batcher
            .finish_buffered_batch()
            .map_err(|e| Error::Batching { source: e })?;

        // safety: If if finish_buffered_batch succeeded then we can expect
        // next_completed_batch to succeed.
        assert!(batcher.has_completed_batch());
        let batch = batcher.next_completed_batch().expect("complete batch");
        result[i] = Some(batch);
    }

    Ok(result)
}

/// Convert the columns of a single input batch to the unified `target_fields`.
///
/// The input batch conforms to `payload_def`, so instead of scanning
/// `target_fields` for every current field (an O(fields^2) search), we look each
/// current field up in the payload spec to get its slot, then map that slot to
/// its position in the target schema via `slot_to_target`. Fields present in the
/// target but absent from this batch are filled with nulls afterward.
fn convert(
    columns: Vec<Arc<dyn Array>>,
    num_rows: usize,
    curr_fields: &Fields,
    target_fields: &Fields,
    payload_def: &PayloadSchema,
    slot_to_target: &[i16; MAX_SLOTS],
) -> Result<Vec<Arc<dyn Array>>> {
    assert_eq!(columns.len(), curr_fields.len());

    // Pre-fill so every target position is initialized; positions not written
    // by an input column are missing fields and are null-padded below.
    let mut new_columns: Vec<Option<Arc<dyn Array>>> = vec![None; target_fields.len()];

    for (curr_idx, curr_field) in curr_fields.iter().enumerate() {
        let slot = payload_def.slot_of(curr_field.name()).ok_or_else(|| {
            Error::ColumnDataTypeMismatch {
                name: curr_field.name().clone(),
                expect: DataType::Null,
                actual: curr_field.data_type().clone(),
            }
        })?;
        let target_idx = slot_to_target[slot];
        debug_assert!(target_idx >= 0, "indexed field must have a target position");
        let target_idx = target_idx as usize;
        let target_field = &target_fields[target_idx];

        let converted = if curr_field.data_type() == target_field.data_type() {
            columns[curr_idx].clone()
        } else if let DataType::Struct(target_struct_fields) = target_field.data_type() {
            let sub_def = payload_def
                .get(curr_field.name())
                .and_then(|f| f.data_type.as_struct_schema())
                .expect("struct field must have a struct sub-schema");
            let sub_map = struct_slot_map(sub_def, target_struct_fields);

            // TODO: Figure out how to avoid the clone here. as_any just returns
            // a ref, so we cannot downcast_mut and break into parts. The clone
            // is only a Vec<ArrayRef>.
            let struct_array = columns[curr_idx]
                .as_any()
                .downcast_ref::<StructArray>()
                .expect("Struct array")
                .clone();
            let (struct_fields, struct_columns, nulls) = struct_array.into_parts();

            // Recursively convert the struct; depth is bounded to 1 since valid
            // OTAP batches do not have nested structs.
            let struct_columns = convert(
                struct_columns,
                num_rows,
                &struct_fields,
                target_struct_fields,
                sub_def,
                &sub_map,
            )?;

            // safety: preconditions satisfied by construction above.
            Arc::new(
                StructArray::try_new_with_length(
                    target_struct_fields.clone(),
                    struct_columns,
                    nulls,
                    num_rows,
                )
                .expect("valid struct array"),
            )
        } else {
            // safety: the selected type is cast-compatible by construction.
            cast(columns[curr_idx].as_ref(), target_field.data_type()).expect("Compatible types")
        };

        new_columns[target_idx] = Some(converted);
    }

    // Fill any target field that this batch did not carry with nulls, reusing
    // the already-allocated Vec rather than collecting into a second one.
    let out = new_columns
        .into_iter()
        .enumerate()
        .map(|(idx, col)| match col {
            Some(col) => col,
            // TODO: Can we optimize here with REE support?
            None => arrow::array::new_null_array(target_fields[idx].data_type(), num_rows),
        })
        .collect();

    Ok(out)
}

/// Build the spec-slot -> target-index map for a struct sub-schema, matching
/// child fields by name against the already-selected target struct fields.
fn struct_slot_map(sub_def: &PayloadSchema, target_fields: &Fields) -> [i16; MAX_SLOTS] {
    let mut map = [-1i16; MAX_SLOTS];
    for (target_idx, field) in target_fields.iter().enumerate() {
        if let Some(slot) = sub_def.slot_of(field.name()) {
            map[slot] = target_idx as i16;
        }
    }
    map
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

/// The widest OTAP payload schema (`logs`) has 14 top-level fields, and its
/// widest struct child (`body`) has 7. All schemas fit within this bound; a
/// compile-time assertion in `schema::payloads` enforces it as schemas evolve.
pub(crate) const MAX_SLOTS: usize = 32;

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
                // No spec entry; surface the offending column's actual type.
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
                    .ok_or_else(|| Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: DataType::Null,
                        actual: field.data_type().clone(),
                    })?;

                // safety: we checked the type
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

        match (existing.value_type, field.data_type()) {
            // If the existing value type is a struct, the new value type
            // must also be a struct.
            (DataType::Struct(_), x) => {
                if !matches!(x, DataType::Struct(_)) {
                    return Err(Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: existing.value_type.clone(),
                        actual: x.clone(),
                    });
                }

                // safety: we checked the type
                let struct_array = data
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .expect("Struct array");

                let struct_index = existing
                    .struct_index
                    .as_mut()
                    .expect("struct field must have a struct index");
                let iter = struct_array.fields().iter().zip(struct_array.columns());
                index_fields(struct_index, iter)?;
            }

            // Cannot change to struct from anything else.
            (x, DataType::Struct(_)) => {
                return Err(Error::ColumnDataTypeMismatch {
                    name: field.name().clone(),
                    expect: x.clone(),
                    actual: field.data_type().clone(),
                });
            }

            // Upgrading from a native type to a dictionary is allowed as long as
            // the value type matches.
            (v1, DataType::Dictionary(k2, v2)) => {
                if *v1 != **v2 {
                    return Err(Error::DictionaryValueTypeMismatch {
                        name: field.name().clone(),
                        expect: v1.clone(),
                        actual: v2.as_ref().clone(),
                    });
                }

                match **k2 {
                    DataType::UInt8 | DataType::UInt16 => {}
                    _ => {
                        return Err(Error::UnsupportedDictionaryKeyType {
                            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                            actual: k2.as_ref().clone(),
                        });
                    }
                }

                existing.is_dictionary = true;
            }

            (v1, v2) => {
                if *v1 != *v2 {
                    return Err(Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: v1.clone(),
                        actual: v2.clone(),
                    });
                }
            }
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

/// Benchmark-only accessors that expose the individual schema-unification
/// stages (`index_records`, `select_schema`, and `convert`) so their cost can
/// be measured in isolation from the row-copying performed by the coalescer.
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

    /// Run the indexing and schema-selection stages for payload slot `i`.
    pub fn bench_select_schema<S: OtapBatchStore, const N: usize>(
        items: &[[Option<RecordBatch>; N]],
        i: usize,
    ) -> Result<Schema> {
        let payload_def = payloads::get(S::payload_type_at_idx(i));
        let index = index_records(select_all(items, i), payload_def)?;
        Ok(select_schema(&index)?.schema)
    }

    /// Run the full schema unification (index + select + convert) for payload
    /// slot `i`, returning the converted columns for every input batch. This
    /// isolates the schema-unification work from the `BatchCoalescer` row copy.
    pub fn bench_convert_all<S: OtapBatchStore, const N: usize>(
        items: &[[Option<RecordBatch>; N]],
        i: usize,
    ) -> Result<Vec<Vec<ArrayRef>>> {
        let payload_def = payloads::get(S::payload_type_at_idx(i));
        let index = index_records(select_all(items, i), payload_def)?;
        let selected = select_schema(&index)?;
        let new_schema = Arc::new(selected.schema);

        let mut converted = Vec::new();
        for payload in select_all(items, i).flatten() {
            let columns = payload.columns().to_vec();
            let num_rows = payload.num_rows();
            let curr_fields = payload.schema_ref().fields.clone();
            converted.push(convert(
                columns,
                num_rows,
                &curr_fields,
                &new_schema.fields,
                payload_def,
                &selected.slot_to_target,
            )?);
        }
        Ok(converted)
    }
}

#[cfg(test)]
mod spec_index_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    /// Every payload schema, including its struct children, must fit within
    /// `MAX_SLOTS` so the flat spec-indexed `FieldIndex` cannot overflow.
    ///
    /// Scenario: iterate every payload type's schema and its struct children.
    /// Guarantees: no schema declares more fields than `MAX_SLOTS`, so a field
    /// slot is always addressable and the index never silently drops a column.
    #[test]
    fn all_payload_schemas_fit_max_slots() {
        // All payload types referenced by the signal stores.
        let types = [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ];

        for pt in types {
            let schema = payloads::get(pt);
            assert!(
                schema.fields().len() <= MAX_SLOTS,
                "{pt:?} top-level fields {} exceed MAX_SLOTS {MAX_SLOTS}",
                schema.fields().len()
            );
            for field in schema.fields() {
                if let Some(sub) = field.data_type.as_struct_schema() {
                    assert!(
                        sub.fields().len() <= MAX_SLOTS,
                        "{pt:?} struct field {} sub-fields {} exceed MAX_SLOTS {MAX_SLOTS}",
                        field.name,
                        sub.fields().len()
                    );
                }
            }
        }
    }
}

// TODO(schema-unify): These modules exercise index_records/select_schema with
// synthetic field names against PayloadSchema::EMPTY, relying on the previous
// BTreeMap behavior that accepted any field name. The spec-indexed
// implementation validates every field against the payload schema, so these
// tests must be rewritten to use real OTAP payload schemas. They are disabled
// via `cfg(any())` (never compiled) until that rewrite lands. See the plan's
// Phase 2 test-rewrite follow-up.
#[cfg(all(test, any()))]
mod schema_tests {
    use super::*;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::{LogAttrs, Logs};
    use crate::record_batch;
    use crate::schema::schema::{
        DataType as SchemaDataType, DictKeySize, Field as SchemaField, Schema as PayloadSchema,
        SimpleType,
    };
    use arrow::array::{
        Array, DictionaryArray, FixedSizeBinaryArray, Int64Array, PrimitiveArray, StringArray,
        UInt8Array, UInt16Array,
    };
    use arrow::datatypes::DataType;
    use rand::RngExt;
    use std::sync::Arc;

    /// A test-only schema definition that has a single "data" column allowing
    /// Dict(u8). Used by the generic cardinality tests which don't test spec
    /// enforcement behavior.
    static TEST_DATA_DEF: PayloadSchema = PayloadSchema {
        fields: &[SchemaField {
            name: "data",
            data_type: SchemaDataType::Dictionary {
                min_key_size: DictKeySize::U8,
                value_type: SimpleType::Utf8,
            },
            required: false,
        }],
        idx: |name| match name {
            "data" => Some(0),
            _ => None,
        },
    };

    #[test]
    fn test_empty_iterator() {
        let records: Vec<Option<&RecordBatch>> = vec![];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn test_none_batches() {
        let records: Vec<Option<&RecordBatch>> = vec![None, None, None];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn test_single_batch() {
        let batch = create_all_types_batch();
        let expected = batch.schema().clone();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        validate_schema(&actual, &expected);
    }

    #[test]
    fn test_same_fields() {
        let batch1 = create_all_types_batch();
        let batch2 = create_all_types_batch();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let expected = batch1.schema();

        validate_schema(&actual, &expected);
    }

    #[test]
    fn test_mixed_fields_nullability() {
        let batch1 = record_batch!(("id", Int32, [1, 2]), ("name", Utf8, ["a", "b"])).unwrap();
        let batch2 = record_batch!(("id", Int32, [3, 4]), ("age", Int32, [25, 30])).unwrap();
        let batch3 = record_batch!(("name", Utf8, ["c", "d"]), ("age", Int32, [35, 40])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let expected = Schema::new(vec![
            Field::new("age", DataType::Int32, true),
            Field::new("id", DataType::Int32, true),
            Field::new("name", DataType::Utf8, true),
        ]);

        validate_schema(&actual, &expected);
    }

    #[test]
    fn test_multiple_batches_one_common_field() {
        let batch1 = record_batch!(("common", Int32, [1]), ("a", Utf8, ["x"])).unwrap();
        let batch2 = record_batch!(("common", Int32, [2]), ("b", Utf8, ["y"])).unwrap();
        let batch3 = record_batch!(("common", Int32, [3]), ("c", Utf8, ["z"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let expected = Schema::new(vec![
            Field::new("a", DataType::Utf8, true),        // in 1/3
            Field::new("b", DataType::Utf8, true),        // in 1/3
            Field::new("c", DataType::Utf8, true),        // in 1/3
            Field::new("common", DataType::Int32, false), // in all
        ]);

        validate_schema(&actual, &expected);
    }

    #[test]
    fn test_cardinality_mixed_key_types() {
        let batch1 =
            record_batch!(("data", (UInt8, UInt16), ([0, 1, 2], [100u16, 200, 300]))).unwrap();
        let batch2 = record_batch!(("data", (UInt16, UInt16), ([0, 1], [100u16, 400]))).unwrap();
        let batch3 = record_batch!(("data", (UInt8, UInt16), ([0, 1], [200u16, 300]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let expected = Schema::new(vec![Field::new(
            "data",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt16)),
            false,
        )]);

        validate_schema(&actual, &expected);
    }

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

    fn create_fsb_dictionary_batch(start: usize, count: usize) -> RecordBatch {
        let keys = UInt16Array::from((0..count).map(|i| i as u16).collect::<Vec<_>>());
        let values = generate_values_for_type(start, count, &DataType::FixedSizeBinary(16));
        create_dict_batch("data", keys, values, DataType::FixedSizeBinary(16))
    }

    fn create_struct_fsb_dictionary_batch(start: usize, count: usize) -> RecordBatch {
        let batch = create_fsb_dictionary_batch(start, count);
        let dict_field = batch.schema().field(0).clone();
        let struct_array = StructArray::from(vec![(
            Arc::new(dict_field.clone()),
            Arc::clone(batch.column(0)),
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            "parent",
            DataType::Struct(vec![dict_field].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
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
        let mut batches = vec![[Some(batch1)], [Some(batch2)]];

        let result = concatenate::<1>(&mut batches).unwrap();
        let batch = result[0].as_ref().expect("concatenated batch");

        assert_eq!(batch.num_rows(), 2 * count);
        assert_eq!(
            batch.schema().field(0).data_type(),
            &DataType::FixedSizeBinary(16)
        );
        assert_overlapping_fsb_values(batch.column(0), count, overlap);
    }

    /// Scenario: A FixedSizeBinary dictionary nested in a supported struct has
    /// overlapping values whose summed physical length exceeds the u16 limit.
    /// Guarantees: Nested dictionary selection uses the same physical bound and
    /// concatenation produces a plain nested field without panicking.
    #[test]
    fn test_nested_overlapping_fsb_dictionaries_fall_back_to_plain() {
        let count = (MAX_U16_CARDINALITY / 2) + 1;
        let overlap = count / 2;
        let batch1 = create_struct_fsb_dictionary_batch(0, count);
        let batch2 = create_struct_fsb_dictionary_batch(overlap, count);
        let mut batches = vec![[Some(batch1)], [Some(batch2)]];

        let result = concatenate::<1>(&mut batches).unwrap();
        let batch = result[0].as_ref().expect("concatenated batch");
        let schema = batch.schema();
        let DataType::Struct(fields) = schema.field(0).data_type() else {
            panic!("expected struct field");
        };

        assert_eq!(batch.num_rows(), 2 * count);
        assert_eq!(fields[0].data_type(), &DataType::FixedSizeBinary(16));
        let struct_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("struct array");
        assert_overlapping_fsb_values(struct_array.column(0), count, overlap);
    }

    /// Scenario: Dictionary value arrays contain one more physical slot than the
    /// u8 limit, but only that limit's number of non-null values.
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
            "data",
            keys1,
            Arc::new(values1),
            DataType::FixedSizeBinary(8),
        );
        let batch2 = create_dict_batch(
            "data",
            keys2,
            Arc::new(values2),
            DataType::FixedSizeBinary(8),
        );

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &TEST_DATA_DEF).unwrap();

        assert_eq!(
            schema.field(0).data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt16),
                Box::new(DataType::FixedSizeBinary(8))
            )
        );
    }

    fn validate_schema(actual: &Schema, expected: &Schema) {
        let merged = Schema::try_merge(vec![actual.clone(), expected.clone()])
            .expect("schemas have compatible types");

        assert_eq!(
            merged.fields().len(),
            expected.fields().len(),
            "Merged schema has different number of fields than expected"
        );
    }

    /// Helper to create a record batch with all supported data types
    fn create_all_types_batch() -> RecordBatch {
        let mut rng = rand::rng();
        #[rustfmt::skip]
        let batch = record_batch!(
            ("f_int8", Int8, [1, 2, 3, rng.random()]),
            ("f_int16", Int16, [5, 6, 7, rng.random()]),
            ("f_int32", Int32, [-1, -2, -3, rng.random()]),
            ("f_int64", Int64, [-5, -6, -7, rng.random()]),
            ("f_uint8", UInt8, [0, 1, 2, rng.random()]),
            ("f_uint16", UInt16, [4, 5, 6, rng.random()]),
            ("f_uint32", UInt32, [8, 9, 10, rng.random()]),
            ("f_uint64", UInt64, [12, 13, 15, rng.random()]),
            ("f_float32", Float32, [16.0, 18.0, 19.0, rng.random()]),
            ("f_float64", Float64, [20.0, 22.0, 23.0, rng.random()]),
            ("f_utf8", Utf8, ["foo", "bar", "baz", "qux"]),
            ("f_dict_u8_utf8", (UInt8, Utf8), ([0, 1, 2, 1], ["a", "b", "c"])),
            ("f_dict_u16_i32", (UInt8, Int32), ([0, 1, 2, 0], [1000000, 0, 1, rng.random()])),
        )
        .unwrap();

        batch
    }

    #[test]
    fn test_above_u16_below_u32() {
        test_cardinality_helper(&[1000], Some(DataType::UInt16));
    }

    #[test]
    fn test_cardinality_at_u16_boundary() {
        // TODO: This should be [30000, 35536]
        // See: https://github.com/open-telemetry/otel-arrow/issues/1971
        test_cardinality_helper(&[30000, 35535], Some(DataType::UInt16));
    }

    #[test]
    fn test_cardinality_above_u16_boundary() {
        // TODO: This should be [30000, 35537]
        // See: https://github.com/open-telemetry/otel-arrow/issues/1971
        test_cardinality_helper(&[30000, 35536], None);
    }

    #[test]
    fn test_cardinality_at_u8_boundary() {
        // TODO: This should be [128, 128]
        // See: https://github.com/open-telemetry/otel-arrow/issues/1971
        test_cardinality_helper(&[128, 127], Some(DataType::UInt8));
    }

    #[test]
    fn test_cardinality_just_above_u8_boundary() {
        // TODO: This should be [128, 128, 1]
        // See: https://github.com/open-telemetry/otel-arrow/issues/1971
        test_cardinality_helper(&[128, 128], Some(DataType::UInt16));
    }

    #[test]
    fn test_cardinality_mixed_batch_sizes() {
        test_cardinality_helper(&[250, 10], Some(DataType::UInt16));
    }

    #[test]
    fn test_cardinality_duration_all_time_units() {
        for unit in [
            arrow_schema::TimeUnit::Second,
            arrow_schema::TimeUnit::Millisecond,
            arrow_schema::TimeUnit::Microsecond,
            arrow_schema::TimeUnit::Nanosecond,
        ] {
            test_cardinality_for_type(&[250], &DataType::Duration(unit), Some(DataType::UInt8));
        }
    }

    #[test]
    fn test_cardinality_duration_all_time_units_above_u8_boundary() {
        for unit in [
            arrow_schema::TimeUnit::Second,
            arrow_schema::TimeUnit::Millisecond,
            arrow_schema::TimeUnit::Microsecond,
            arrow_schema::TimeUnit::Nanosecond,
        ] {
            test_cardinality_for_type(
                &[250, 10],
                &DataType::Duration(unit),
                Some(DataType::UInt16),
            );
        }
    }

    #[test]
    fn test_cardinality_timestamp_all_time_units() {
        for unit in [
            arrow_schema::TimeUnit::Second,
            arrow_schema::TimeUnit::Millisecond,
            arrow_schema::TimeUnit::Microsecond,
            arrow_schema::TimeUnit::Nanosecond,
        ] {
            test_cardinality_for_type(
                &[250],
                &DataType::Timestamp(unit, None),
                Some(DataType::UInt8),
            );
        }
    }

    #[test]
    fn test_cardinality_timestamp_all_time_units_above_u8_boundary() {
        for unit in [
            arrow_schema::TimeUnit::Second,
            arrow_schema::TimeUnit::Millisecond,
            arrow_schema::TimeUnit::Microsecond,
            arrow_schema::TimeUnit::Nanosecond,
        ] {
            test_cardinality_for_type(
                &[250, 10],
                &DataType::Timestamp(unit, None),
                Some(DataType::UInt16),
            );
        }
    }

    #[test]
    fn test_cardinality_timestamp_with_timezone() {
        test_cardinality_for_type(
            &[250],
            &DataType::Timestamp(
                arrow_schema::TimeUnit::Microsecond,
                Some(Arc::<str>::from("UTC")),
            ),
            Some(DataType::UInt8),
        );
    }

    #[test]
    fn test_cardinality_timestamp_with_timezone_above_u8_boundary() {
        test_cardinality_for_type(
            &[250, 10],
            &DataType::Timestamp(
                arrow_schema::TimeUnit::Microsecond,
                Some(Arc::<str>::from("UTC")),
            ),
            Some(DataType::UInt16),
        );
    }

    #[test]
    fn test_generate_timestamp_values_preserves_timezone_datatype() {
        let value_type = DataType::Timestamp(
            arrow_schema::TimeUnit::Nanosecond,
            Some(Arc::<str>::from("UTC")),
        );

        let values = generate_values_for_type(0, 3, &value_type);
        assert_eq!(values.data_type(), &value_type);
    }

    /// Create a Dict(u8, Utf8) batch with low cardinality for a given column name.
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

    /// Create a Dict(u8, Int64) batch with low cardinality for a given column name.
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

    /// Create a struct batch with a Dict(u8, Utf8) sub-field inside a struct.
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

    #[test]
    fn test_u16_attrs_str_column_enforces_u16_key() {
        let def = payloads::get(LogAttrs);

        // Create two batches with low cardinality Dict(u8, Utf8) for the "str" column
        let batch1 = create_low_cardinality_u8_utf8_batch("str", 10);
        let batch2 = create_low_cardinality_u8_utf8_batch("str", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        // Without spec enforcement this would be Dict(u8, Utf8).
        // With spec enforcement it must be Dict(u16, Utf8).
        let field = schema.field_with_name("str").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            "U16 attrs 'str' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_u16_attrs_int_column_enforces_u16_key() {
        let def =
            payloads::get(crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::ResourceAttrs);

        let batch1 = create_low_cardinality_u8_int64_batch("int", 10);
        let batch2 = create_low_cardinality_u8_int64_batch("int", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let field = schema.field_with_name("int").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
            "U16 attrs 'int' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_u32_attrs_str_column_enforces_u16_key() {
        let def =
            payloads::get(crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::SpanEventAttrs);

        let batch1 = create_low_cardinality_u8_utf8_batch("str", 10);
        let batch2 = create_low_cardinality_u8_utf8_batch("str", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let field = schema.field_with_name("str").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            "U32 attrs 'str' column must use Dict(u16) key, got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_logs_body_str_enforces_u16_key() {
        let def = payloads::get(Logs);

        let batch1 = create_struct_with_u8_dict_batch("body", "str", 10);
        let batch2 = create_struct_with_u8_dict_batch("body", "str", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let body_field = schema.field_with_name("body").unwrap();
        if let DataType::Struct(fields) = body_field.data_type() {
            let str_field = fields
                .iter()
                .find(|f| f.name() == "str")
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

    #[test]
    fn test_logs_body_ser_enforces_u16_key() {
        let def = payloads::get(Logs);

        let batch1 = create_struct_with_u8_dict_batch("body", "ser", 10);
        let batch2 = create_struct_with_u8_dict_batch("body", "ser", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let body_field = schema.field_with_name("body").unwrap();
        if let DataType::Struct(fields) = body_field.data_type() {
            let ser_field = fields
                .iter()
                .find(|f| f.name() == "ser")
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

    #[test]
    fn test_attrs_key_column_allows_u8() {
        let def = payloads::get(LogAttrs);

        let batch1 = create_low_cardinality_u8_utf8_batch("key", 10);
        let batch2 = create_low_cardinality_u8_utf8_batch("key", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let field = schema.field_with_name("key").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            "Attrs 'key' column may use Dict(u8), got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_logs_severity_text_allows_u8() {
        let def = payloads::get(Logs);

        let batch1 = create_low_cardinality_u8_utf8_batch("severity_text", 10);
        let batch2 = create_low_cardinality_u8_utf8_batch("severity_text", 5);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let field = schema.field_with_name("severity_text").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            "LOGS severity_text may use Dict(u8), got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_non_dict_column_strips_dictionary() {
        let def = payloads::get(LogAttrs);

        // "type" column is UInt8 with no dictionary support in the spec.
        // If input has it as Dict(u8, UInt8), select_schema should strip the
        // dictionary and return plain UInt8.
        let keys = UInt8Array::from(vec![0u8, 1, 0, 1]);
        let values: Arc<dyn Array> = Arc::new(UInt8Array::from(vec![1u8, 2]));
        let dict_array = DictionaryArray::<UInt8Type>::try_new(keys, values).unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new(
            "type",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt8)),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap();

        let keys2 = UInt8Array::from(vec![0u8, 1]);
        let values2: Arc<dyn Array> = Arc::new(UInt8Array::from(vec![3u8, 4]));
        let dict_array2 = DictionaryArray::<UInt8Type>::try_new(keys2, values2).unwrap();
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            "type",
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt8)),
            false,
        )]));
        let batch2 = RecordBatch::try_new(schema2, vec![Arc::new(dict_array2)]).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, def).unwrap();

        let field = schema.field_with_name("type").unwrap();
        assert_eq!(
            field.data_type(),
            &DataType::UInt8,
            "'type' column should be native UInt8, got {:?}",
            field.data_type()
        );
    }

    /// Helper function to test cardinality selection with specified parameters
    /// Tests all supported value types automatically, skipping types that can't
    /// represent the required cardinality.
    ///
    /// # Arguments
    /// * `cardinalities` - List of unique value counts per batch
    /// * `expected_dict_key` - Expected dictionary key type (None for primitive output)
    fn test_cardinality_helper(cardinalities: &[usize], expected_dict_key: Option<DataType>) {
        let total_cardinality: usize = cardinalities.iter().sum();

        // Get list of value types to test based on cardinality
        let value_types = get_testable_value_types(total_cardinality);

        for value_type in value_types {
            test_cardinality_for_type(cardinalities, &value_type, expected_dict_key.clone());
        }
    }

    /// Get the list of value types that can represent the given cardinality when
    /// used as values. One and two byte types have to stick within their limits
    fn get_testable_value_types(cardinality: usize) -> Vec<DataType> {
        let mut types = vec![];

        // 1 byte types
        if cardinality < MAX_U8_CARDINALITY {
            types.push(DataType::UInt8);
            types.push(DataType::Int8);
        }

        // 2 byte types
        if cardinality < MAX_U16_CARDINALITY {
            types.push(DataType::UInt16);
            types.push(DataType::Int16);
            types.push(DataType::Float16);
        }

        // 4+ byte types
        types.push(DataType::UInt32);
        types.push(DataType::UInt64);
        types.push(DataType::Int32);
        types.push(DataType::Int64);
        types.push(DataType::Float32);
        types.push(DataType::Float64);
        types.push(DataType::FixedSizeBinary(8));
        types.push(DataType::FixedSizeBinary(16));
        types.push(DataType::Utf8);
        types.push(DataType::LargeUtf8);
        types.push(DataType::LargeBinary);

        types
    }

    /// Test cardinality selection for a specific value type
    fn test_cardinality_for_type(
        cardinalities: &[usize],
        value_type: &DataType,
        expected_dict_key: Option<DataType>,
    ) {
        let total_cardinality: usize = cardinalities.iter().sum();
        let key_type_str = match &expected_dict_key {
            Some(dt) => format!("{:?}", dt),
            None => "None".to_string(),
        };
        let test_context = format!(
            "[value_type={:?}, total_cardinality={}, cardinalities={:?}, expected_key={}]",
            value_type, total_cardinality, cardinalities, key_type_str
        );

        let batches = generate_batches_with_cardinality(cardinalities, value_type);
        let batch_refs: Vec<Option<&RecordBatch>> = batches.iter().map(Some).collect();

        // Test schema selection
        let index = index_records(batch_refs.into_iter())
            .unwrap_or_else(|e| panic!("Failed to index records {}: {:?}", test_context, e));
        let actual_schema = select_schema(&index, &TEST_DATA_DEF)
            .unwrap_or_else(|e| panic!("Failed to select schema {}: {:?}", test_context, e));

        let expected_field_type = match expected_dict_key.clone() {
            Some(key_type) => {
                DataType::Dictionary(Box::new(key_type), Box::new(value_type.clone()))
            }
            None => value_type.clone(),
        };

        let expected_schema =
            Schema::new(vec![Field::new("data", expected_field_type.clone(), false)]);

        validate_schema(&actual_schema, &expected_schema);

        // Test actual concatenation
        let mut batches_for_concat: Vec<[Option<RecordBatch>; 1]> =
            batches.into_iter().map(|batch| [Some(batch)]).collect();

        let result = concatenate::<1>(&mut batches_for_concat)
            .unwrap_or_else(|e| panic!("Concatenation failed {}: {:?}", test_context, e));

        // Verify concatenated result
        assert_eq!(
            result.len(),
            1,
            "Should have one output batch {}",
            test_context
        );
        let concatenated_batch = result[0]
            .as_ref()
            .unwrap_or_else(|| panic!("Output batch should exist {}", test_context));

        // Verify schema matches expected
        let output_schema = concatenated_batch.schema();
        assert_eq!(
            output_schema.fields().len(),
            1,
            "Should have exactly one field {}",
            test_context
        );
        let output_field = &output_schema.fields()[0];
        assert_eq!(
            output_field.name(),
            "data",
            "Field name should be 'data' {}",
            test_context
        );
        assert_eq!(
            output_field.data_type(),
            &expected_field_type,
            "Output field type mismatch {}: expected {:?}, got {:?}",
            test_context,
            expected_field_type,
            output_field.data_type()
        );

        // Verify row count matches sum of input cardinalities
        let expected_rows: usize = cardinalities.iter().sum();
        assert_eq!(
            concatenated_batch.num_rows(),
            expected_rows,
            "Row count mismatch {}: expected {} rows, got {}",
            test_context,
            expected_rows,
            concatenated_batch.num_rows()
        );
    }

    /// Generate record batches with specified cardinalities for a given value type
    fn generate_batches_with_cardinality(
        cardinalities: &[usize],
        value_type: &DataType,
    ) -> Vec<RecordBatch> {
        let mut offset = 0;
        let total_cardinality: usize = cardinalities.iter().sum();

        // Use U16 keys if total cardinality exceeds what U8 can index
        let use_u16_keys = total_cardinality > MAX_U8_CARDINALITY;
        cardinalities
            .iter()
            .map(|&cardinality| {
                let batch = if use_u16_keys {
                    let keys: Vec<u16> = (0..cardinality).map(|i| i as u16).collect();
                    let values = generate_values_for_type(offset, cardinality, value_type);

                    create_dict_batch("data", UInt16Array::from(keys), values, value_type.clone())
                } else {
                    let keys: Vec<u8> = (0..cardinality).map(|i| i as u8).collect();
                    let values = generate_values_for_type(offset, cardinality, value_type);

                    create_dict_batch("data", UInt8Array::from(keys), values, value_type.clone())
                };
                offset += cardinality;
                batch
            })
            .collect()
    }

    /// Generate an array of values for a specific type starting at a given offset
    fn generate_values_for_type(
        start: usize,
        count: usize,
        value_type: &DataType,
    ) -> Arc<dyn Array> {
        use arrow::array::*;

        let end = start + count;
        match value_type {
            DataType::UInt8 => Arc::new(UInt8Array::from(
                (start..end).map(|i| (i % 256) as u8).collect::<Vec<_>>(),
            )),
            DataType::Int8 => Arc::new(Int8Array::from(
                (start..end)
                    .map(|i| ((i % 256) as i16 - 128) as i8)
                    .collect::<Vec<_>>(),
            )),
            DataType::UInt16 => Arc::new(UInt16Array::from(
                (start..end).map(|i| (i % 65536) as u16).collect::<Vec<_>>(),
            )),
            DataType::Int16 => Arc::new(Int16Array::from(
                (start..end)
                    .map(|i| ((i % 65536) as i32 - 32768) as i16)
                    .collect::<Vec<_>>(),
            )),
            DataType::Float16 => {
                use arrow::buffer::Buffer;
                use arrow::datatypes::Float16Type;
                // Generate unique Float16 values
                let values: Vec<u16> = (start..end).map(|i| (i % 65536) as u16).collect();
                let buffer = Buffer::from_slice_ref(&values);
                Arc::new(PrimitiveArray::<Float16Type>::new(buffer.into(), None))
            }
            DataType::UInt32 => Arc::new(UInt32Array::from(
                (start..end).map(|i| i as u32).collect::<Vec<_>>(),
            )),
            DataType::Int32 => Arc::new(Int32Array::from(
                (start..end).map(|i| i as i32).collect::<Vec<_>>(),
            )),
            DataType::Float32 => Arc::new(Float32Array::from(
                (start..end).map(|i| i as f32 + 0.5).collect::<Vec<_>>(),
            )),
            DataType::UInt64 => Arc::new(UInt64Array::from(
                (start..end).map(|i| i as u64).collect::<Vec<_>>(),
            )),
            DataType::Int64 => Arc::new(Int64Array::from(
                (start..end).map(|i| i as i64).collect::<Vec<_>>(),
            )),
            DataType::Float64 => Arc::new(Float64Array::from(
                (start..end).map(|i| i as f64 + 0.5).collect::<Vec<_>>(),
            )),
            DataType::Duration(unit) => {
                let values = (start..end).map(|i| i as i64).collect::<Vec<_>>();
                match unit {
                    arrow_schema::TimeUnit::Second => Arc::new(DurationSecondArray::from(values)),
                    arrow_schema::TimeUnit::Millisecond => {
                        Arc::new(DurationMillisecondArray::from(values))
                    }
                    arrow_schema::TimeUnit::Microsecond => {
                        Arc::new(DurationMicrosecondArray::from(values))
                    }
                    arrow_schema::TimeUnit::Nanosecond => {
                        Arc::new(DurationNanosecondArray::from(values))
                    }
                }
            }
            DataType::Timestamp(unit, tz) => {
                // The full value_type (including unit and tz) is forwarded to
                // cast() below; binding them here documents intent.
                let _ = (unit, tz);
                let values = (start..end).map(|i| i as i64).collect::<Vec<_>>();
                let values: ArrayRef = Arc::new(Int64Array::from(values));
                cast(values.as_ref(), value_type).unwrap()
            }
            DataType::FixedSizeBinary(8) => {
                use arrow::buffer::Buffer;
                let values: Vec<u8> = (start..end)
                    .flat_map(|i| (i as u64).to_le_bytes())
                    .collect();
                let buffer = Buffer::from_vec(values);
                let array = FixedSizeBinaryArray::try_new(8, buffer, None).unwrap();
                Arc::new(array)
            }
            DataType::FixedSizeBinary(16) => {
                use arrow::buffer::Buffer;
                let values: Vec<u8> = (start..end)
                    .flat_map(|i| {
                        let mut bytes = [0u8; 16];
                        bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
                        bytes[8..16].copy_from_slice(&(i as u64).to_le_bytes());
                        bytes
                    })
                    .collect();
                let buffer = Buffer::from_vec(values);
                Arc::new(FixedSizeBinaryArray::try_new(16, buffer, None).unwrap())
            }
            DataType::Utf8 => Arc::new(StringArray::from(
                (start..end)
                    .map(|i| format!("value_{}", i))
                    .collect::<Vec<_>>(),
            )),
            DataType::LargeUtf8 => Arc::new(LargeStringArray::from(
                (start..end)
                    .map(|i| format!("value_{}", i))
                    .collect::<Vec<_>>(),
            )),
            DataType::LargeBinary => {
                use arrow::array::GenericBinaryBuilder;
                let mut builder = GenericBinaryBuilder::<i64>::new();
                for i in start..end {
                    let mut bytes = format!("binary_{}", i).into_bytes();
                    // Add index bytes to ensure uniqueness
                    bytes.extend_from_slice(&(i as u64).to_le_bytes());
                    builder.append_value(&bytes);
                }
                Arc::new(builder.finish())
            }
            _ => panic!("Unsupported value type for test: {:?}", value_type),
        }
    }
}

#[cfg(all(test, any()))]
mod index_tests {
    use super::*;
    use crate::record_batch;
    use arrow::array::{Int32Array, StructArray};
    use arrow::datatypes::Int32Type;
    use std::sync::Arc;

    #[test]
    fn test_struct_to_non_struct_mismatch() {
        let struct_field = Field::new("value", DataType::Int32, true);
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field),
            Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef,
        )]);
        let expected_struct_type =
            DataType::Struct(vec![Field::new("value", DataType::Int32, true)].into());
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            expected_struct_type.clone(),
            true,
        )]));
        let batch1 = RecordBatch::try_new(schema1, vec![Arc::new(struct_array)]).unwrap();
        let batch2 = record_batch!(("data", Int32, [1, 2, 3])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "data");
                assert_eq!(expect, expected_struct_type);
                assert_eq!(actual, DataType::Int32);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_non_struct_to_struct_mismatch() {
        let batch1 = record_batch!(("data", Int32, [1, 2, 3])).unwrap();

        let struct_field = Field::new("value", DataType::Int32, true);
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field),
            Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef,
        )]);
        let expected_struct_type =
            DataType::Struct(vec![Field::new("value", DataType::Int32, true)].into());
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            "data",
            expected_struct_type.clone(),
            true,
        )]));
        let batch2 = RecordBatch::try_new(schema2, vec![Arc::new(struct_array)]).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "data");
                assert_eq!(expect, DataType::Int32);
                assert!(matches!(actual, DataType::Struct { .. }));
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_dictionary_value_type_mismatch() {
        let batch1 =
            record_batch!(("status", (UInt8, Int32), ([0, 1, 2], [100, 200, 300]))).unwrap();
        let batch2 =
            record_batch!(("status", (UInt8, Utf8), ([0, 1, 2], ["foo", "bar", "baz"]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::DictionaryValueTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "status");
                assert_eq!(expect, DataType::Int32);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnValueTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_unsupported_dictionary_key_type() {
        let keys = Int32Array::from(vec![0, 1, 2]);
        let values = Arc::new(arrow::array::StringArray::from(vec!["foo", "bar", "baz"]));
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, values).unwrap();

        let schema = Arc::new(Schema::new(vec![Field::new(
            "category",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            true,
        )]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(dict_array)]).unwrap();

        let records = vec![Some(&batch)];
        let result = index_records(records.into_iter());

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

    #[test]
    fn test_primitive_type_mismatch() {
        let batch1 = record_batch!(("value", Int32, [1, 2, 3])).unwrap();
        let batch2 = record_batch!(("value", Int64, [4, 5, 6])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "value");
                assert_eq!(expect, DataType::Int32);
                assert_eq!(actual, DataType::Int64);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_dictionary_to_primitive_mismatch() {
        let batch1 = record_batch!(("data", (UInt8, Int32), ([0, 1, 2], [100, 200, 300]))).unwrap();
        let batch2 = record_batch!(("data", Utf8, ["foo", "bar", "baz"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::ColumnDataTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "data");
                assert_eq!(expect, DataType::Int32);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnDataTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_primitive_to_dictionary_mismatch() {
        let batch1 = record_batch!(("data", Int32, [100, 200, 300])).unwrap();
        let batch2 =
            record_batch!(("data", (UInt8, Utf8), ([0, 1, 2], ["foo", "bar", "baz"]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let result = index_records(records.into_iter());

        match result {
            Err(Error::DictionaryValueTypeMismatch {
                name,
                expect,
                actual,
            }) => {
                assert_eq!(name, "data");
                assert_eq!(expect, DataType::Int32);
                assert_eq!(actual, DataType::Utf8);
            }
            _ => panic!("Expected ColumnValueTypeMismatch error, got: {:?}", result),
        }
    }

    #[test]
    fn test_primitive_to_dictionary_upgrade_success() {
        let batch1 = record_batch!(("data", Int32, [100, 200, 300])).unwrap();
        let batch2 = record_batch!(("data", (UInt8, Int32), ([0, 1, 2], [100, 400, 500]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let field = schema.field_with_name("data").unwrap();
        assert!(
            matches!(
                field.data_type(),
                DataType::Dictionary(k, v) if **k == DataType::UInt8 && **v == DataType::Int32
            ),
            "Expected Dictionary(UInt8, Int32), got {:?}",
            field.data_type()
        );
    }

    #[test]
    fn test_index_fields_with_mixed_types_and_none_batches() {
        #[rustfmt::skip]
        let batch0 = record_batch!(
            ("status", (UInt8, Utf8), ([0, 1, 2], ["ok", "error", "pending"])),
            ("count", Int32, [10, 20, 30]),
            ("name", Utf8, ["alice", "bob", "charlie"])
        ).unwrap();

        #[rustfmt::skip]
        let batch2 = record_batch!(
            ("status", (UInt16, Utf8), ([0, 1], ["ok", "error"])),
            ("count", Int32, [5, 15]),
            ("age", Int32, [Some(25), None])
        ).unwrap();

        #[rustfmt::skip]
        let batch4 = record_batch!(
            ("status", (UInt8, Utf8), ([0, 1, 2, 3], ["ok", "error", "pending", "skipped"])),
            ("name", Utf8, ["dave", "eve", "frank", "grace"])
        ).unwrap();

        let batch6 = record_batch!(
            ("count", Int32, [Some(100), None, Some(200)]),
            ("age", Int32, [30, 40, 50])
        )
        .unwrap();

        let records = vec![
            Some(&batch0),
            None,
            Some(&batch2),
            None,
            Some(&batch4),
            None,
            None,
            Some(&batch6),
        ];

        let result = index_records(records.into_iter()).unwrap();
        assert_eq!(result.batch_count, 4, "batch_count mismatch");
        assert_eq!(result.fields.len(), 4, "Expected 4 fields in index");

        // Validate "status" field
        // Present in batches 0, 2, 4 (indices 0, 2, 4)
        // Total elements: 3 + 2 + 4 = 9
        // Total values: 3 + 2 + 4 = 9 (no nulls)
        // Largest value count: 4 (from batch 4)
        // Value type: Utf8
        // Smallest key type: UInt8 (batch 0 and 4 use UInt8, batch 2 uses UInt16)
        let status_field = result.fields.get("status").expect("status field missing");
        validate_field(
            "status",
            status_field,
            &DataType::Utf8,
            Some(DataType::UInt8),
            9, // total_element_count
            9, // total_value_count
            4, // largest_value_count
        );

        // Validate "count" field
        // Present in batches 0, 2, 6 (indices 0, 2, 6)
        // Total elements: 3 + 2 + 3 = 8
        // Total values: 3 + 2 + 2 = 7 (one null in batch 6)
        // Largest value count: 3 (from batch 0)
        // Value type: Int32
        // Smallest key type: None (not a dictionary)
        let count_field = result.fields.get("count").expect("count field missing");
        validate_field(
            "count",
            count_field,
            &DataType::Int32,
            None,
            8, // total_element_count
            7, // total_value_count
            3, // largest_value_count
        );

        // Validate "name" field
        // Present in batches 0, 4 (indices 0, 4)
        // Total elements: 3 + 4 = 7
        // Total values: 3 + 4 = 7 (no nulls)
        // Largest value count: 4 (from batch 4)
        // Value type: Utf8
        // Smallest key type: None (not a dictionary)
        let name_field = result.fields.get("name").expect("name field missing");
        validate_field(
            "name",
            name_field,
            &DataType::Utf8,
            None,
            7, // total_element_count
            7, // total_value_count
            4, // largest_value_count
        );

        // Validate "age" field
        // Present in batches 2, 6 (indices 2, 6)
        // Total elements: 2 + 3 = 5
        // Total values: 1 + 3 = 4 (one null in batch 2)
        // Largest value count: 3 (from batch 6)
        // Value type: Int32
        // Smallest key type: None (not a dictionary)
        let age_field = result.fields.get("age").expect("age field missing");
        validate_field(
            "age",
            age_field,
            &DataType::Int32,
            None,
            5, // total_element_count
            4, // total_value_count
            3, // largest_value_count
        );
    }

    /// Helper function to validate a single field from the index
    fn validate_field<'a>(
        field_name: &str,
        field_info: &FieldInfo<'a>,
        expected_value_type: &DataType,
        expected_smallest_key_type: Option<DataType>,
        expected_total_element_count: usize,
        expected_total_value_count: usize,
        expected_largest_value_count: usize,
    ) {
        assert_eq!(
            field_info.value_type, expected_value_type,
            "Field '{}': value_type mismatch",
            field_name
        );
        assert_eq!(
            field_info.smallest_key_type, expected_smallest_key_type,
            "Field '{}': smallest_key_type mismatch",
            field_name
        );
        assert_eq!(
            field_info.total_element_count, expected_total_element_count,
            "Field '{}': total_element_count mismatch",
            field_name
        );
        assert_eq!(
            field_info.total_value_count, expected_total_value_count,
            "Field '{}': total_value_count mismatch",
            field_name
        );
        assert_eq!(
            field_info.largest_value_count, expected_largest_value_count,
            "Field '{}': largest_value_count mismatch",
            field_name
        );
    }
}

#[cfg(all(test, any()))]
mod nullability_tests {
    use super::*;
    use crate::record_batch;
    use arrow::array::{Int32Array, StructArray};
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    /// Helper to assert a field's nullability in a schema
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

    /// Helper to create a simple struct batch
    fn create_struct_batch(struct_name: &str, field_name: &str, values: Vec<i32>) -> RecordBatch {
        let struct_field = Field::new(field_name, DataType::Int32, false);
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field),
            Arc::new(Int32Array::from(values)) as ArrayRef,
        )]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            struct_name,
            DataType::Struct(vec![Field::new(field_name, DataType::Int32, false)].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    #[test]
    fn test_field_nullable_when_missing_in_some_batches() {
        let batch1 = record_batch!(("id", Int32, [1, 2]), ("name", Utf8, ["a", "b"])).unwrap();
        let batch2 = record_batch!(("id", Int32, [3, 4])).unwrap();
        let batch3 = record_batch!(("id", Int32, [5, 6]), ("name", Utf8, ["c", "d"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_nullable(&schema, "name", true);
        assert_field_nullable(&schema, "id", false);
    }

    #[test]
    fn test_field_nullable_with_null_values_in_array() {
        let batch1 = record_batch!(("value", Int32, [Some(1), Some(2)])).unwrap();
        let batch2 = record_batch!(("value", Int32, [Some(3), None])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_nullable(&schema, "value", true);
    }

    #[test]
    fn test_multiple_fields_nullable_combinations() {
        let batch1 = record_batch!(
            ("a", Int32, [1, 2]),
            ("b", Int32, [3, 4]),
            ("c", Int32, [5, 6])
        )
        .unwrap();
        let batch2 = record_batch!(("a", Int32, [7, 8]), ("b", Int32, [9, 10])).unwrap();
        let batch3 = record_batch!(("a", Int32, [11, 12]), ("c", Int32, [13, 14])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_nullable(&schema, "a", false);
        assert_field_nullable(&schema, "b", true);
        assert_field_nullable(&schema, "c", true);
    }

    #[test]
    fn test_struct_field_nullable_when_struct_missing_from_batches() {
        let batch1 = create_struct_batch("data", "value", vec![1, 2, 3]);
        let batch2 = record_batch!(("other", Int32, [1, 2])).unwrap();
        let batch3 = create_struct_batch("data", "value", vec![4, 5, 6]);

        let records = vec![Some(&batch1), Some(&batch2), Some(&batch3)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Struct "data" is missing from batch2, so it should be nullable
        assert_field_nullable(&schema, "data", true);
        assert_field_nullable(&schema, "other", true);

        // Check the struct field itself
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let value_field = fields
                .iter()
                .find(|f| f.name() == "value")
                .expect("value field should exist");
            assert!(
                value_field.is_nullable(),
                "Struct field 'value' should be nullable when parent struct missing from batches"
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_field_nullability_basic() {
        // Simple test with single batch containing struct with non-nullable fields
        let struct_fields = vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
        ];
        let struct_array = StructArray::from(vec![
            (
                Arc::new(struct_fields[0].clone()),
                Arc::new(Int32Array::from(vec![1, 2])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields[1].clone()),
                Arc::new(Int32Array::from(vec![3, 4])) as ArrayRef,
            ),
        ]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields.into()),
            false,
        )]));
        let batch = RecordBatch::try_new(schema1, vec![Arc::new(struct_array)]).unwrap();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Verify struct fields are present
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");
            assert!(fields.iter().any(|f| f.name() == "a"),);
            assert!(fields.iter().any(|f| f.name() == "b"),);
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_fields_accumulated_across_batches() {
        let struct_fields = vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
        ];
        let struct_array1 = StructArray::from(vec![
            (
                Arc::new(struct_fields[0].clone()),
                Arc::new(Int32Array::from(vec![1])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields[1].clone()),
                Arc::new(Int32Array::from(vec![2])) as ArrayRef,
            ),
        ]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields.clone().into()),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema1, vec![Arc::new(struct_array1)]).unwrap();

        // Batch 2: same struct schema with same fields "a" and "b"
        let struct_fields2 = struct_fields.clone();
        let struct_array2 = StructArray::from(vec![
            (
                Arc::new(struct_fields2[0].clone()),
                Arc::new(Int32Array::from(vec![3])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields2[1].clone()),
                Arc::new(Int32Array::from(vec![4])) as ArrayRef,
            ),
        ]);

        let schema2 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields2.into()),
            false,
        )]));
        let batch2 = RecordBatch::try_new(schema2, vec![Arc::new(struct_array2)]).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();

        // Verify internal state: struct fields should be accumulated
        if let Some(data_field_info) = index.fields.get("data") {
            assert_eq!(data_field_info.values.len(), 2,);

            if let Some(struct_index) = &data_field_info.struct_index {
                for (name, field_info) in struct_index.iter() {
                    assert_eq!(
                        field_info.values.len(),
                        2,
                        "Struct field '{}' should have 2 values (accumulated from both batches), but has {}",
                        name,
                        field_info.values.len()
                    );
                }
            }
        }

        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Both fields should be present and not nullable (present in all instances)
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");

            let a_field = fields.iter().find(|f| f.name() == "a").expect("a exists");
            let b_field = fields.iter().find(|f| f.name() == "b").expect("b exists");

            assert!(!a_field.is_nullable(),);
            assert!(!b_field.is_nullable(),);
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_field_union_behavior() {
        // Test verifies that struct fields from different batches are properly unioned.
        // When struct instances have different fields across batches, all fields should
        // be included in the final schema and marked nullable when not present in all instances.

        // Batch 1: struct with fields "a" and "b"
        let struct_fields1 = vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
        ];
        let struct_array1 = StructArray::from(vec![
            (
                Arc::new(struct_fields1[0].clone()),
                Arc::new(Int32Array::from(vec![1])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields1[1].clone()),
                Arc::new(Int32Array::from(vec![2])) as ArrayRef,
            ),
        ]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields1.into()),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema1, vec![Arc::new(struct_array1)]).unwrap();

        // Batch 2: struct with fields "a" and "c"
        let struct_fields2 = vec![
            Field::new("a", DataType::Int32, false),
            Field::new("c", DataType::Int32, false),
        ];
        let struct_array2 = StructArray::from(vec![
            (
                Arc::new(struct_fields2[0].clone()),
                Arc::new(Int32Array::from(vec![3])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields2[1].clone()),
                Arc::new(Int32Array::from(vec![4])) as ArrayRef,
            ),
        ]);
        let schema2 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields2.into()),
            false,
        )]));
        let batch2 = RecordBatch::try_new(schema2, vec![Arc::new(struct_array2)]).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // All three fields (a, b, c) from both batches should be present
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();

            // All fields should be present
            assert_eq!(field_names.len(), 3,);
            assert!(field_names.contains(&"a"),);
            assert!(field_names.contains(&"b"),);
            assert!(field_names.contains(&"c"),);

            // Fields b and c should be nullable since they don't appear in all struct instances
            let b_field = fields.iter().find(|f| f.name() == "b").expect("b exists");
            let c_field = fields.iter().find(|f| f.name() == "c").expect("c exists");
            assert!(b_field.is_nullable(),);
            assert!(c_field.is_nullable(),);

            // Field a present in all instances should not be nullable
            let a_field = fields.iter().find(|f| f.name() == "a").expect("a exists");
            assert!(!a_field.is_nullable(),);
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }
}

#[cfg(all(test, any()))]
mod metadata_tests {
    use super::*;
    use crate::record_batch;
    use crate::schema::consts::metadata::COLUMN_ENCODING;
    use crate::schema::consts::metadata::encodings::PLAIN;
    use crate::schema::consts::{ID, PARENT_ID};
    use arrow::array::{Int32Array, StructArray};
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    /// Helper to assert field has expected metadata
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

    #[test]
    fn test_metadata_added_to_id_field() {
        let batch1 = record_batch!(("id", Int32, [1, 2])).unwrap();
        let batch2 = record_batch!(("id", Int32, [3, 4])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
    }

    #[test]
    fn test_metadata_added_to_parent_id_field() {
        let batch1 = record_batch!(("parent_id", Int32, [0, 1])).unwrap();
        let batch2 = record_batch!(("parent_id", Int32, [2, 3])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }

    #[test]
    fn test_metadata_not_added_to_regular_fields() {
        let batch1 = record_batch!(("value", Int32, [1, 2]), ("name", Utf8, ["a", "b"])).unwrap();
        let batch2 = record_batch!(("value", Int32, [3, 4]), ("name", Utf8, ["c", "d"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_metadata(&schema, "value", COLUMN_ENCODING, None);
        assert_field_metadata(&schema, "name", COLUMN_ENCODING, None);
    }

    #[test]
    fn test_metadata_added_to_dictionary_id_field() {
        let batch1 = record_batch!(("id", (UInt8, Int32), ([0, 1], [100, 200]))).unwrap();
        let batch2 = record_batch!(("id", (UInt8, Int32), ([0, 1], [300, 400]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // ID field should get PLAIN metadata even when dictionary-encoded
        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
    }

    #[test]
    fn test_metadata_added_to_struct_id_fields() {
        // Test verifies that "id" and "parent_id" fields within structs also get PLAIN metadata,
        // just like top-level fields with those names.

        // Create struct with "id" and "value" fields
        let struct_fields = vec![
            Field::new(ID, DataType::Int32, false),
            Field::new("value", DataType::Int32, false),
        ];
        let struct_array = StructArray::from(vec![
            (
                Arc::new(struct_fields[0].clone()),
                Arc::new(Int32Array::from(vec![1, 2])) as ArrayRef,
            ),
            (
                Arc::new(struct_fields[1].clone()),
                Arc::new(Int32Array::from(vec![10, 20])) as ArrayRef,
            ),
        ]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(struct_fields.into()),
            false,
        )]));
        let batch = RecordBatch::try_new(schema1, vec![Arc::new(struct_array)]).unwrap();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Struct subfields named "id" should get PLAIN metadata
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let id_field = fields
                .iter()
                .find(|f| f.name() == ID)
                .expect("id field should exist");
            let metadata_value = id_field.metadata().get(COLUMN_ENCODING);
            assert_eq!(
                metadata_value,
                Some(&PLAIN.to_string()),
                "Struct subfield 'id' should have PLAIN encoding metadata"
            );

            let value_field = fields
                .iter()
                .find(|f| f.name() == "value")
                .expect("value field should exist");
            let value_metadata = value_field.metadata().get(COLUMN_ENCODING);
            assert_eq!(
                value_metadata, None,
                "Struct field 'value' should not have encoding metadata"
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_metadata_on_both_id_and_parent_id() {
        let batch1 = record_batch!(("id", Int32, [1, 2]), ("parent_id", Int32, [0, 1])).unwrap();
        let batch2 = record_batch!(("id", Int32, [3, 4]), ("parent_id", Int32, [2, 3])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }
}

#[cfg(all(test, any()))]
mod struct_field_tests {
    use super::*;
    use crate::record_batch;
    use arrow::array::{DictionaryArray, Int32Array, StringArray, StructArray, UInt8Array};
    use arrow::datatypes::UInt8Type;
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    /// Helper to create a struct batch with a dictionary field
    fn create_struct_with_dict_field(
        struct_name: &str,
        field_name: &str,
        keys: Vec<u8>,
        values: Vec<&str>,
    ) -> RecordBatch {
        let key_array = UInt8Array::from(keys);
        let value_array = Arc::new(StringArray::from(values));
        let dict_array = DictionaryArray::<UInt8Type>::new(key_array, value_array);

        let struct_field = Field::new(
            field_name,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
            false,
        );
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field.clone()),
            Arc::new(dict_array) as ArrayRef,
        )]);

        let schema = Arc::new(Schema::new(vec![Field::new(
            struct_name,
            DataType::Struct(vec![struct_field].into()),
            false,
        )]));
        RecordBatch::try_new(schema, vec![Arc::new(struct_array)]).unwrap()
    }

    #[test]
    fn test_struct_with_dictionary_field_u8() {
        let batch1 = create_struct_with_dict_field("data", "status", vec![0, 1], vec!["a", "b"]);
        let batch2 = create_struct_with_dict_field("data", "status", vec![0, 1], vec!["c", "d"]);

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Check struct field type
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let status_field = fields
                .iter()
                .find(|f| f.name() == "status")
                .expect("status field should exist");
            assert!(
                matches!(
                    status_field.data_type(),
                    DataType::Dictionary(k, v) if **k == DataType::UInt8 && **v == DataType::Utf8
                ),
                "Expected Dictionary(UInt8, Utf8), got {:?}",
                status_field.data_type()
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_with_primitive_field() {
        // Create struct with simple Int32 field
        let struct_field = Field::new("value", DataType::Int32, false);
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field.clone()),
            Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef,
        )]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(vec![struct_field].into()),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema1, vec![Arc::new(struct_array.clone())]).unwrap();
        let batch2 = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "data",
                DataType::Struct(vec![Field::new("value", DataType::Int32, false)].into()),
                false,
            )])),
            vec![Arc::new(struct_array)],
        )
        .unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let value_field = fields
                .iter()
                .find(|f| f.name() == "value")
                .expect("value field should exist");
            assert_eq!(
                value_field.data_type(),
                &DataType::Int32,
                "Expected Int32 type"
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_completely_missing_from_batch() {
        // Create struct batch
        let struct_field = Field::new("value", DataType::Int32, false);
        let struct_array = StructArray::from(vec![(
            Arc::new(struct_field.clone()),
            Arc::new(Int32Array::from(vec![1, 2])) as ArrayRef,
        )]);
        let schema1 = Arc::new(Schema::new(vec![Field::new(
            "data",
            DataType::Struct(vec![struct_field].into()),
            false,
        )]));
        let batch1 = RecordBatch::try_new(schema1, vec![Arc::new(struct_array)]).unwrap();

        // Batch without struct
        let batch2 = record_batch!(("other", Int32, [3, 4])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        // Struct field should be nullable when missing from batch
        let struct_field = schema.field_with_name("data").unwrap();
        assert!(
            struct_field.is_nullable(),
            "Struct should be nullable when missing from some batches"
        );
    }

    #[test]
    fn test_multiple_struct_fields_in_schema() {
        // Create batch with two different struct fields
        let struct_field1 = Field::new("value", DataType::Int32, false);
        let struct_array1 = StructArray::from(vec![(
            Arc::new(struct_field1.clone()),
            Arc::new(Int32Array::from(vec![1, 2])) as ArrayRef,
        )]);

        let struct_field2 = Field::new("name", DataType::Utf8, false);
        let struct_array2 = StructArray::from(vec![(
            Arc::new(struct_field2.clone()),
            Arc::new(StringArray::from(vec!["a", "b"])) as ArrayRef,
        )]);

        let schema1 = Arc::new(Schema::new(vec![
            Field::new(
                "struct1",
                DataType::Struct(vec![struct_field1].into()),
                false,
            ),
            Field::new(
                "struct2",
                DataType::Struct(vec![struct_field2].into()),
                false,
            ),
        ]));
        let batch = RecordBatch::try_new(
            schema1,
            vec![Arc::new(struct_array1), Arc::new(struct_array2)],
        )
        .unwrap();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index, &PayloadSchema::EMPTY).unwrap();

        assert!(schema.field_with_name("struct1").is_ok());
        assert!(schema.field_with_name("struct2").is_ok());
    }
}
