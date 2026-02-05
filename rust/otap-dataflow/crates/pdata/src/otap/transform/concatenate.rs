// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use ahash::AHashSet;
use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, AsArray, DictionaryArray, OffsetSizeTrait, RecordBatch,
    StructArray,
};
use arrow::compute::kernels::cast;
use arrow::datatypes::{
    ArrowNativeType, Float64Type, GenericBinaryType, Int64Type, UInt8Type, UInt16Type, UInt64Type,
};
use arrow_schema::{DataType, Field, FieldRef, Fields, Schema, SchemaBuilder};
use itertools::Either;
use roaring::RoaringBitmap;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::Error;
use crate::otap::Result;
use crate::schema::consts::metadata::COLUMN_ENCODING;
use crate::schema::consts::metadata::encodings::PLAIN;
use crate::schema::consts::{ID, PARENT_ID};

/// These are one less than the maximum cardinality of the key type. We should be
/// able to go up to 256/65536 without overflow, but there is a bug in arrow-rs
/// for some value types.
///
/// See:
///     - https://github.com/apache/arrow-rs/issues/9366
///     - github.com/open-telemetry/otel-arrow/issues/1971
const MAX_U8_CARDINALITY: usize = 255;
const MAX_U16_CARDINALITY: usize = 65535;

/// Concatenate the provided OtapArrowRecords into a single batch.
///
/// # Preconditions
///
/// Currently the caller is responsible for satisfying the following:
///
///   1. Remove the transport optimized encodings from the columns, if any
///   2. Reindex the ID columns so that the parent child relationships are
///   consistent after the concatenation
///
/// These will be handled internally in the future as we refine the API, see
/// https://github.com/open-telemetry/otel-arrow/issues/1926.
///
/// # General Algorithm
///
/// Concatenating multiple OtapArrowRecords involves three steps:
///
///   1. Reindexing the ID columns so that the parent child relationships are
///   consistent after the concatenation
///   2. Selecting a common schema and converting every record batch to that
///   schema. This includes several steps:
///       a. Indexing all fields for the same ArrowPayloadType across every batch
///       b. Estimating the cardinality for each dictionary field and selecting
///       the key type.
///       c. Determining nullability for each field in the final batch
///   3. Casting every record batch to the final schema, including casting individual
///   arrays as well as reordering the columns to match the schema.
///
/// # Future optimizations
///
/// - TODO: Re-indexing probably should not be a separate operation. We should decide
/// within this function whether or not to do it and ensure it happens if required.
/// This is deferred until we totally remove the old implementation in groups.rs
/// due to interface incompatibility.
///
/// - TODO: Consider using new_unchecked for record batch construction if we're
/// confident in it. We mostly unwrap those operations a lot, so skipping the
/// checks or moving similar checks to debug asserts may be reasonable.
pub fn concatenate<const N: usize>(
    mut items: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
    let mut result = [const { None }; N];
    if items.is_empty() {
        return Ok(result);
    }

    if items.len() == 1 {
        for i in 0..N {
            result[i] = items[0][i].take();
        }
        return Ok(result);
    }

    for i in 0..N {
        let index = index_records(select_all(&items, i))?;
        if index.batch_count == 0 {
            continue;
        }

        let new_schema = Arc::from(select_schema(&index)?);
        let mut batcher = arrow::compute::BatchCoalescer::new(new_schema.clone(), index.row_count);
        for payload in select_all_mut(&mut items, i) {
            let Some(rb) = payload.take() else {
                continue;
            };

            let (curr_schema, columns, num_rows) = rb.into_parts();
            let converted_columns =
                convert(columns, num_rows, &curr_schema.fields, &new_schema.fields)?;

            // safety: Unless we have a bug, we've satisfied all the preconditions
            // for try_new and push_batch by converting everything to a unified
            // schema.
            let converted = RecordBatch::try_new(new_schema.clone(), converted_columns)
                .expect("Valid construction");
            batcher.push_batch(converted).expect("Compatible schemas");
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

/// Convert the columns from one schema to another. The arguments deal in in
/// fields and columns rather than schemas and record batches so that this
/// code can work with either struct arrays or record batches.
fn convert(
    columns: Vec<Arc<dyn Array>>,
    num_rows: usize,
    curr_fields: &Fields,
    target_fields: &Fields,
) -> Result<Vec<Arc<dyn Array>>> {
    assert_eq!(columns.len(), curr_fields.len());

    let mut new_columns: Vec<Arc<dyn Array>> = Vec::with_capacity(target_fields.len());

    for target_field in target_fields.iter() {
        // TODO: We can probably eliminate this find call by adding a map from
        // batch number -> field position for every field to the index.
        match curr_fields.find(target_field.name()) {
            Some((curr_idx, curr_field)) => {
                if curr_field.data_type() != target_field.data_type() {
                    if let DataType::Struct(target_struct_fields) = target_field.data_type() {
                        let struct_array = columns[curr_idx].clone();
                        // TODO: Figure out how to avoid the clone here. as_any just returns a
                        // ref, so we cannot downcast_mut. Since we can't downcast_mut, we can't
                        // break into parts. The clone isn't that bad for now since it's only a
                        // Vec<ArrayRef>.
                        let struct_array = struct_array
                            .as_any()
                            .downcast_ref::<StructArray>()
                            .unwrap()
                            .clone();

                        let (struct_fields, struct_columns, nulls) = struct_array.into_parts();

                        // Recursively convert the struct, depth is bounded to 1
                        // since we don't support nested structs which is checked
                        // by [index_fields].
                        let struct_columns = convert(
                            struct_columns,
                            num_rows,
                            &struct_fields,
                            target_struct_fields,
                        )?;

                        // safety: Unless we have a bug, we've satisfied all
                        // the preconditions laid out in this function and
                        // the columns, schema, and row count are valid.
                        let struct_array = StructArray::try_new_with_length(
                            target_struct_fields.clone(),
                            struct_columns,
                            nulls,
                            num_rows,
                        )
                        .expect("valid struct array");

                        new_columns.push(Arc::new(struct_array))
                    } else {
                        // safety: Unless we have a bug, we've satisfied all
                        // the preconditions laid out in this function and
                        // the columns, schema, and row count are valid.
                        let new_data = cast(columns[curr_idx].as_ref(), target_field.data_type())
                            .expect("Compatible types");

                        new_columns.push(new_data)
                    }
                } else {
                    new_columns.push(columns[curr_idx].clone());
                }
            }
            None => {
                // TODO: Can we optimize here with REE support?
                new_columns.push(arrow::array::new_null_array(
                    target_field.data_type(),
                    num_rows,
                ))
            }
        }
    }

    Ok(new_columns)
}

/// Select a unified schema that will satisfy all fields in the
/// RecordIndex.
fn select_schema<'a>(index: &'a RecordIndex<'a>) -> Result<Schema> {
    let mut builder = SchemaBuilder::with_capacity(index.fields.len());
    for (field_name, field_info) in index.fields.iter() {
        // Presence of smallest key type indicates dictionary
        let mut typ = field_info.value_type.clone();
        if field_info.smallest_key_type.is_some() {
            assert!(field_info.struct_index.is_none());
            typ = select_dictionary_type(&field_info)?
        } else if let Some(ref struct_index) = field_info.struct_index {
            typ = select_struct_type(struct_index)?;
        }

        let mut new_field = Field::new(*field_name, typ, field_info.nullable);
        add_field_metadata(&mut new_field);
        builder.push(new_field);
    }

    Ok(builder.finish())
}

/// Select the final data type for a struct field.
fn select_struct_type<'a>(struct_index: &'a FieldIndex<'a>) -> Result<DataType> {
    let mut fields = Vec::with_capacity(struct_index.len());
    for (field_name, field_info) in struct_index.iter() {
        // Presence of smallest key type indicates dictionary
        let mut typ = field_info.value_type.clone();
        if field_info.smallest_key_type.is_some() {
            typ = select_dictionary_type(&field_info)?
        }

        // This should have been detected by the indexing logic
        assert!(!matches!(field_info.value_type, DataType::Struct(_)));

        let mut new_field = Field::new(*field_name, typ, field_info.nullable);
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

#[derive(Debug)]
struct RecordIndex<'a> {
    batch_count: usize,
    row_count: usize,
    fields: FieldIndex<'a>,
}

type FieldIndex<'a> = BTreeMap<&'a str, FieldInfo<'a>>;

#[derive(Debug)]
struct FieldInfo<'a> {
    // The value type of the column, note that this must be some primitive or
    // struct type, it will never be a dictionary. In the case of a struct, it
    // is not necessarily the final struct type as we need to do more processing
    // of the struct_index to determine the final type.
    value_type: &'a DataType,
    // Indicates if this is nullable, determined by if any of the values are null
    // in any batch or if any batch is missing the field
    nullable: bool,
    // Set if this is a dictionary
    smallest_key_type: Option<DataType>,
    // Set if this is a struct
    struct_index: Option<FieldIndex<'a>>,
    // The number of total elements including nulls
    total_element_count: usize,
    // The total number of values, excluding nulls
    total_value_count: usize,
    // The size of the largest individual array in values
    largest_value_count: usize,
    // The values arrays for the type, some of these may come from dictionary array values.
    values: Vec<ArrayRef>,
}

/// Create an index of fields while checking which type corresponds to each, that the
/// value types are compatible, and computing basic statistics for each field.
fn index_records<'a>(
    batches: impl Iterator<Item = Option<&'a RecordBatch>>,
) -> Result<RecordIndex<'a>> {
    let mut index = RecordIndex {
        batch_count: 0,
        row_count: 0,
        fields: BTreeMap::new(),
    };

    for rb in batches {
        let Some(rb) = rb else {
            continue;
        };

        index.batch_count += 1;
        index.row_count += rb.num_rows();

        let fields = rb.schema_ref().fields();
        let iter = fields.iter().zip(rb.columns());
        index_fields(&mut index.fields, iter, None)?;
    }

    // We need a final pass to see if any fields were not present in any batch
    // and similarly for structs to see if any struct fields were missing.
    // When we coerce to the same schema, we have to appent nulls for missing fields.
    for field in index.fields.values_mut() {
        field.nullable = field.nullable || field.values.len() != index.batch_count;
        if let Some(struct_index) = field.struct_index.as_mut() {
            for struct_field in struct_index.values_mut() {
                struct_field.nullable = struct_field.nullable
                    || field.nullable
                    || struct_field.values.len() != field.values.len()
            }
        }
    }

    Ok(index)
}

/// Index the fields for some stream columns.
fn index_fields<'a>(
    index: &mut FieldIndex<'a>,
    fields: impl Iterator<Item = (&'a FieldRef, &'a ArrayRef)>,
    parent: Option<&'a str>,
) -> Result<()> {
    for (field, data) in fields {
        let (array, value_type, key_type) = match data.data_type() {
            DataType::Dictionary(k, v) => (
                get_dictionary_values(&data)?,
                v.as_ref(),
                Some(k.as_ref().clone()),
            ),
            x => (data, x, None),
        };

        let Some(existing) = index.get_mut(field.name().as_str()) else {
            let values_count = array.len() - array.null_count();

            // If this is a struct type, we need to index its fields
            let struct_index = if matches!(value_type, DataType::Struct(_)) {
                // safety: we checked the type
                let struct_array = data
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .expect("Struct array");

                let mut struct_index = BTreeMap::new();
                let iter = struct_array.fields().iter().zip(struct_array.columns());
                index_fields(&mut struct_index, iter, Some(field.name().as_str()))?;
                Some(struct_index)
            } else {
                None
            };

            let _ = index.insert(
                field.name().as_str(),
                FieldInfo {
                    value_type: value_type,
                    nullable: field.is_nullable(),
                    smallest_key_type: key_type,
                    struct_index,
                    total_element_count: data.len(),
                    largest_value_count: values_count,
                    total_value_count: values_count,
                    values: vec![array.clone()],
                },
            );
            continue;
        };

        let values = match (&existing.value_type, field.data_type()) {
            // If the existing value type is a struct, the new value type
            // must also be a struct
            (DataType::Struct(_), x) => {
                if let Some(parent) = parent {
                    return Err(Error::InvalidDataTypeForStruct {
                        parent: parent.to_string(),
                        name: field.name().clone(),
                        data_type: x.clone(),
                    });
                }

                if !matches!(x, DataType::Struct(_)) {
                    return Err(Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: existing.value_type.clone(),
                        actual: x.clone(),
                    });
                }

                // Recursively index this struct. This has a maximum depth of 1
                // because we forbid nested structs since valid otap batches
                // do not have them.
                //
                // safety: we checked the type
                let struct_array = data
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .expect("Struct array");

                let struct_index = existing.struct_index.get_or_insert_with(BTreeMap::new);
                let iter = struct_array.fields().iter().zip(struct_array.columns());
                index_fields(struct_index, iter, Some(field.name().as_str()))?;

                data
            }

            // Cannot change to struct from anything else
            (x, DataType::Struct(_)) => {
                return Err(Error::ColumnDataTypeMismatch {
                    name: field.name().clone(),
                    expect: (*x).clone(),
                    actual: field.data_type().clone(),
                });
            }

            // Upgrading from a native type to a dictionary is allowed
            // as long as the value type matches.
            (v1, DataType::Dictionary(k2, v2)) => {
                if **v1 != **v2 {
                    return Err(Error::DictionaryValueTypeMismatch {
                        name: field.name().clone(),
                        expect: (*v1).clone(),
                        actual: v2.as_ref().clone(),
                    });
                }

                match **k2 {
                    DataType::UInt8 => {
                        existing.smallest_key_type = Some(DataType::UInt8);
                    }
                    DataType::UInt16 => {}
                    _ => {
                        return Err(Error::UnsupportedDictionaryKeyType {
                            expect_oneof: vec![DataType::UInt8, DataType::UInt16],
                            actual: k2.as_ref().clone(),
                        });
                    }
                }

                get_dictionary_values(&data)?
            }

            (v1, v2) => {
                if **v1 != *v2 {
                    return Err(Error::ColumnDataTypeMismatch {
                        name: field.name().clone(),
                        expect: (*v1).clone(),
                        actual: v2.clone(),
                    });
                }
                data
            }
        };

        let _ = existing.values.push(values.clone());
        let values_count = values.len() - values.null_count();
        existing.nullable = existing.nullable || field.is_nullable();
        existing.total_element_count += data.len();
        existing.total_value_count += values_count;
        existing.largest_value_count = existing.largest_value_count.max(values_count);
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

fn select_dictionary_type<'a>(info: &FieldInfo<'a>) -> Result<DataType> {
    assert!(info.smallest_key_type.is_some());

    // If the lower bound is above u16::MAX, then cardinality is too large to dictionary encode.
    if info.largest_value_count > MAX_U16_CARDINALITY {
        return Ok(info.value_type.clone());
    }

    // If the upper bound is below u8::MAX, then so is the lower bound and we can select u8.
    if info.total_value_count <= MAX_U8_CARDINALITY {
        return Ok(DataType::Dictionary(
            Box::new(DataType::UInt8),
            Box::new(info.value_type.clone()),
        ));
    }

    // If we're between (u8, u16], then we can select u16
    if info.total_value_count <= MAX_U16_CARDINALITY
        && info.largest_value_count > MAX_U8_CARDINALITY
    {
        return Ok(DataType::Dictionary(
            Box::new(DataType::UInt16),
            Box::new(info.value_type.clone()),
        ));
    }

    let value_type = info.value_type.clone();
    let cardinality = estimate_cardinality(&info);
    match cardinality {
        Cardinality::WithinU8 => Ok(DataType::Dictionary(
            Box::new(DataType::UInt8),
            Box::new(value_type),
        )),
        Cardinality::WithinU16 => Ok(DataType::Dictionary(
            Box::new(DataType::UInt16),
            Box::new(value_type),
        )),
        Cardinality::GreaterThanU16 => Ok(value_type),
    }
}

enum Cardinality {
    WithinU8,
    WithinU16,
    GreaterThanU16,
}

impl Cardinality {
    fn from_exact(count: usize) -> Cardinality {
        match count {
            count if count <= MAX_U8_CARDINALITY => Cardinality::WithinU8,
            count if count <= MAX_U16_CARDINALITY => Cardinality::WithinU16,
            _ => Cardinality::GreaterThanU16,
        }
    }
}

fn estimate_cardinality<'a>(info: &FieldInfo<'a>) -> Cardinality {
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
        DataType::Float64 => estimate_cardinality_primitive_type::<Float64Type, 8>(info),
        DataType::FixedSizeBinary(8) => estimate_cardinality_fixed_size_type::<8>(info),
        DataType::FixedSizeBinary(16) => estimate_cardinality_fixed_size_type::<16>(info),
        DataType::Utf8 => estimate_cardinality_string_type::<i32>(info),
        DataType::LargeUtf8 => estimate_cardinality_string_type::<i64>(info),
        DataType::LargeBinary => {
            let iter = info
                .values
                .iter()
                .map(|v| v.as_bytes::<GenericBinaryType<i64>>().iter().flatten())
                .flatten();
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
                buf[range].chunks_exact(ELEMENT_WIDTH)
            })),
            None => Either::Right(buf.chunks_exact(ELEMENT_WIDTH)),
        }
    });

    estimate_cardinality_generic(info, iter)
}

fn estimate_cardinality_string_type<'a, T: OffsetSizeTrait>(info: &FieldInfo<'a>) -> Cardinality {
    let iter = info
        .values
        .iter()
        .map(|v| v.as_string::<T>().iter().flatten().map(|s| s.as_bytes()))
        .flatten();
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
                        &info,
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
                    visit_native_values(value_buf, &info, &mut bitmap, &mut visited_element_count);
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

#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::record_batch;
    use arrow::array::{Array, DictionaryArray, PrimitiveArray, UInt8Array, UInt16Array};
    use rand::Rng;
    use std::sync::Arc;

    #[test]
    fn test_empty_iterator() {
        let records: Vec<Option<&RecordBatch>> = vec![];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn test_none_batches() {
        let records: Vec<Option<&RecordBatch>> = vec![None, None, None];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn test_single_batch() {
        let batch = create_all_types_batch();
        let expected = batch.schema().clone();

        let records = vec![Some(&batch)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index).unwrap();

        validate_schema(&actual, &expected);
    }

    #[test]
    fn test_same_fields() {
        let batch1 = create_all_types_batch();
        let batch2 = create_all_types_batch();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let actual = select_schema(&index).unwrap();

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
        let actual = select_schema(&index).unwrap();

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
        let actual = select_schema(&index).unwrap();

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
        let actual = select_schema(&index).unwrap();

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
        //
        // // 4+ byte types
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
        let batch_refs: Vec<Option<&RecordBatch>> = batches.iter().map(|b| Some(b)).collect();

        // Test schema selection
        let index = index_records(batch_refs.into_iter())
            .unwrap_or_else(|e| panic!("Failed to index records {}: {:?}", test_context, e));
        let actual_schema = select_schema(&index)
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
                    create_batch_u16_dict(offset, cardinality, value_type)
                } else {
                    create_batch_u8_dict(offset, cardinality, value_type)
                };
                offset += cardinality;
                batch
            })
            .collect()
    }

    /// Create a batch with UInt8 dictionary keys
    fn create_batch_u8_dict(
        start_value: usize,
        cardinality: usize,
        value_type: &DataType,
    ) -> RecordBatch {
        let keys: Vec<u8> = (0..cardinality).map(|i| i as u8).collect();
        let values = generate_values_for_type(start_value, cardinality, value_type);

        create_dict_batch("data", UInt8Array::from(keys), values, value_type.clone())
    }

    /// Create a batch with UInt16 dictionary keys
    fn create_batch_u16_dict(
        start_value: usize,
        cardinality: usize,
        value_type: &DataType,
    ) -> RecordBatch {
        let keys: Vec<u16> = (0..cardinality).map(|i| i as u16).collect();
        let values = generate_values_for_type(start_value, cardinality, value_type);

        create_dict_batch("data", UInt16Array::from(keys), values, value_type.clone())
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
            // UTF-8 string
            DataType::Utf8 => Arc::new(StringArray::from(
                (start..end)
                    .map(|i| format!("value_{}", i))
                    .collect::<Vec<_>>(),
            )),
            // Large UTF-8 string (i64 offsets)
            DataType::LargeUtf8 => Arc::new(LargeStringArray::from(
                (start..end)
                    .map(|i| format!("value_{}", i))
                    .collect::<Vec<_>>(),
            )),
            // Large binary (i64 offsets)
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

#[cfg(test)]
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
        let schema = select_schema(&index).unwrap();

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

#[cfg(test)]
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
        let schema = select_schema(&index).unwrap();

        // "name" is missing from batch2, so it should be nullable
        assert_field_nullable(&schema, "name", true);
        // "id" is present in all batches
        assert_field_nullable(&schema, "id", true); // Still nullable because field schema marks it nullable
    }

    #[test]
    fn test_field_nullable_with_null_values_in_array() {
        let batch1 = record_batch!(("value", Int32, [Some(1), Some(2)])).unwrap();
        let batch2 = record_batch!(("value", Int32, [Some(3), None])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

        // Field has null values, so it should be nullable
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
        let schema = select_schema(&index).unwrap();

        assert_field_nullable(&schema, "a", true);
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
        let schema = select_schema(&index).unwrap();

        // Struct "data" is missing from batch2, so it should be nullable
        assert_field_nullable(&schema, "data", true);

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
        let schema = select_schema(&index).unwrap();

        // Verify struct fields are present
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");
            assert!(
                fields.iter().any(|f| f.name() == "a"),
                "Field 'a' should exist"
            );
            assert!(
                fields.iter().any(|f| f.name() == "b"),
                "Field 'b' should exist"
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }

    #[test]
    fn test_struct_fields_accumulated_across_batches() {
        // CRITICAL TEST: Verifies that when the same struct appears in multiple batches,
        // the struct fields' values are properly accumulated (not just indexed once).
        // This catches bugs where struct fields from first batch aren't updated when
        // the same struct appears in subsequent batches.

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

        // Batch 2: same struct schema with same fields "a" and "b"
        let struct_fields2 = vec![
            Field::new("a", DataType::Int32, false),
            Field::new("b", DataType::Int32, false),
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

        // Verify internal state: struct fields should be accumulated
        if let Some(data_field_info) = index.fields.get("data") {
            assert_eq!(
                data_field_info.values.len(),
                2,
                "Parent struct field should have 2 values (one from each batch)"
            );

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

        let schema = select_schema(&index).unwrap();

        // Both fields should be present and not nullable (present in all instances)
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            assert_eq!(fields.len(), 2, "Should have 2 struct fields");

            let a_field = fields.iter().find(|f| f.name() == "a").expect("a exists");
            let b_field = fields.iter().find(|f| f.name() == "b").expect("b exists");

            assert!(
                !a_field.is_nullable(),
                "Field 'a' should not be nullable (present in all struct instances)"
            );
            assert!(
                !b_field.is_nullable(),
                "Field 'b' should not be nullable (present in all struct instances)"
            );
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
        let schema = select_schema(&index).unwrap();

        // All three fields (a, b, c) from both batches should be present
        let struct_field = schema.field_with_name("data").unwrap();
        if let DataType::Struct(fields) = struct_field.data_type() {
            let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();

            // All fields should be present
            assert_eq!(
                field_names.len(),
                3,
                "Should have 3 fields (a, b, c) in struct"
            );
            assert!(
                field_names.contains(&"a"),
                "Field 'a' should exist (present in both batches)"
            );
            assert!(
                field_names.contains(&"b"),
                "Field 'b' should exist (present in batch 1)"
            );
            assert!(
                field_names.contains(&"c"),
                "Field 'c' should exist (present in batch 2)"
            );

            // Fields b and c should be nullable since they don't appear in all struct instances
            let b_field = fields.iter().find(|f| f.name() == "b").expect("b exists");
            let c_field = fields.iter().find(|f| f.name() == "c").expect("c exists");
            assert!(
                b_field.is_nullable(),
                "Field 'b' should be nullable (missing from batch 2)"
            );
            assert!(
                c_field.is_nullable(),
                "Field 'c' should be nullable (missing from batch 1)"
            );

            // Field a present in all instances should not be nullable
            let a_field = fields.iter().find(|f| f.name() == "a").expect("a exists");
            assert!(
                !a_field.is_nullable(),
                "Field 'a' should not be nullable (present in all instances)"
            );
        } else {
            panic!("Expected Struct type for 'data' field");
        }
    }
}

#[cfg(test)]
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
        let schema = select_schema(&index).unwrap();

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
    }

    #[test]
    fn test_metadata_added_to_parent_id_field() {
        let batch1 = record_batch!(("parent_id", Int32, [0, 1])).unwrap();
        let batch2 = record_batch!(("parent_id", Int32, [2, 3])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }

    #[test]
    fn test_metadata_not_added_to_regular_fields() {
        let batch1 = record_batch!(("value", Int32, [1, 2]), ("name", Utf8, ["a", "b"])).unwrap();
        let batch2 = record_batch!(("value", Int32, [3, 4]), ("name", Utf8, ["c", "d"])).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

        assert_field_metadata(&schema, "value", COLUMN_ENCODING, None);
        assert_field_metadata(&schema, "name", COLUMN_ENCODING, None);
    }

    #[test]
    fn test_metadata_added_to_dictionary_id_field() {
        let batch1 = record_batch!(("id", (UInt8, Int32), ([0, 1], [100, 200]))).unwrap();
        let batch2 = record_batch!(("id", (UInt8, Int32), ([0, 1], [300, 400]))).unwrap();

        let records = vec![Some(&batch1), Some(&batch2)];
        let index = index_records(records.into_iter()).unwrap();
        let schema = select_schema(&index).unwrap();

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
        let schema = select_schema(&index).unwrap();

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
        let schema = select_schema(&index).unwrap();

        assert_field_metadata(&schema, ID, COLUMN_ENCODING, Some(PLAIN));
        assert_field_metadata(&schema, PARENT_ID, COLUMN_ENCODING, Some(PLAIN));
    }
}

#[cfg(test)]
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
        let schema = select_schema(&index).unwrap();

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
        let schema = select_schema(&index).unwrap();

        // Check struct field is Int32
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
        let schema = select_schema(&index).unwrap();

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
        let schema = select_schema(&index).unwrap();

        assert!(schema.field_with_name("struct1").is_ok());
        assert!(schema.field_with_name("struct2").is_ok());
    }
}
