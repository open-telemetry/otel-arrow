// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for adding transport optimized encodings to the various columns in
//! OTAP record batches when converting to BatchArrowRecords. These types of encodings include
//! delta encoding for ID columns, and quasi-delta encoding for attribute parent IDs.
//!
//! The motivation behind adding these encodings to the columns is for better compression when,
//! for example, transmitting OTAP data via gRPC.

use std::{
    ops::{Add, AddAssign, Sub},
    sync::Arc,
};

use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, BooleanArray, PrimitiveArray, PrimitiveBuilder,
        RecordBatch, StructArray, UInt16Array, UInt32Array,
    },
    buffer::{MutableBuffer, ScalarBuffer},
    compute::{SortColumn, SortOptions, and, take_record_batch},
    datatypes::{ArrowNativeType, DataType, FieldRef, Schema, UInt16Type, UInt32Type},
};
use snafu::{OptionExt, ResultExt};

use crate::{
    arrays::{NullableArrayAccessor, get_u8_array},
    error::{self, Result},
    otap::transform::{
        create_next_element_equality_array, create_next_eq_array_for_array,
        materialize_parent_id_for_attributes, materialize_parent_id_for_exemplars,
        materialize_parent_ids_by_columns, remove_delta_encoding,
        remove_delta_encoding_from_column, sort_to_indices,
    },
    otlp::attributes::{parent_id::ParentId, store::AttributeValueType},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::{FieldExt, consts, get_field_metadata},
};

/// identifier for column encoding
#[derive(Clone, Copy)]
enum Encoding {
    /// Delta encoding. Note that to use this encoding, the column must already be sorted.
    /// Otherwise `DeltaRemapped` is more appropriate
    Delta,

    /// This encoding creates a new delta encoded ID column, but also create new IDs instead of
    /// simply delta encoding the original IDs. This is used to add delta encoding to columns that
    /// are not already sorted by the ID, since most ID columns are unsigned ints which means no
    /// negative deltas are allowed. The parent IDs which point at IDs that have been replaced will
    /// need to be remapped.
    DeltaRemapped,

    /// This is the transport optimized encoding that is applied to the parent_id column
    /// of attribute record batches where where subsequent rows of matching attribute type,
    /// key and value will have delta encoded parent IDs
    AttributeQuasiDelta,

    /// This is similar to AttributeQuasiDelta, but applied to non-attribute record batches like
    /// Span Events and Span Links. In this case, instead of looking at attribute columns to
    /// determine runs of delta encoding, we consider the values in arbitrary columns (the names of
    /// which are contained within this enum variant).
    ColumnarQuasiDelta(&'static [&'static str]),
}

// For the majority of columns, we'll be able to identify the path within the record batch as
// the column name directly, but Resource ID and Scope ID, they're typically nested within a
// struct on the root record so we treat these as special cases:/
/// path within the record batch to the resource ID column
pub const RESOURCE_ID_COL_PATH: &str = "resource.id";
/// path within the record batch to the scope ID column
pub const SCOPE_ID_COL_PATH: &str = "scope.id";

/// specification for encoding that should be applied to some column
struct ColumnEncoding<'a> {
    /// path to the column within the record batch
    path: &'a str,

    /// the expected data type of the column
    data_type: DataType,

    /// identifier for how the column should be encoded
    encoding: Encoding,
}

/// checks if the column associated with this [`ColumnEncoding`] has already had the column
/// encoding applied.
///
/// this is done by inspecting the field metadata, and specifically checking that the column
/// encoding is not 'plain'. if there is no field metadata, we assume the column is already
/// encoded. we make this assumption because it probably means we received this OTAP data from
/// the golang OTAP exporter, which always encodes the columns and never adds metadata
///
/// returns `None` if the field associated with `self.path` isn't found in passed schema
fn is_column_encoded(path: &str, schema: &Schema) -> Option<bool> {
    let field = if let Some(struct_col_name) = struct_column_name(path) {
        // get the ID field out of the struct column
        let struct_col = schema.field_with_name(struct_col_name).ok()?;
        if let DataType::Struct(fields) = struct_col.data_type() {
            fields.find(consts::ID).map(|(_, field)| field)?
        } else {
            return None;
        }
    } else {
        // otherwise just look at field with path == name
        schema.field_with_name(path).ok()?
    };

    // check the field metadata to determine if field is encoded
    let field_metadata = field.metadata();
    let is_encoded = match field_metadata.get(consts::metadata::COLUMN_ENCODING) {
        Some(encoding) => encoding != consts::metadata::encodings::PLAIN,

        // assume if there is no metadata, then the column is already encoded
        None => true,
    };

    Some(is_encoded)
}

/// Helper function for accessing the column associated for the (possibly nested) path
fn access_column(path: &str, schema: &Schema, columns: &[ArrayRef]) -> Option<ArrayRef> {
    // handle special case of accessing either the resource ID or scope ID which are nested
    // within a struct
    if let Some(struct_col_name) = struct_column_name(path) {
        let struct_col_idx = schema.index_of(struct_col_name).ok()?;
        let struct_col = columns
            .get(struct_col_idx)?
            .as_any()
            .downcast_ref::<StructArray>()?;
        return struct_col.column_by_name(consts::ID).cloned();
    }

    // otherwise just return column by name
    let column_idx = schema.index_of(path).ok()?;
    columns.get(column_idx).cloned()
}

/// Replaces the column identified by `path` within the array of columns with the new column.
fn replace_column(
    path: &str,
    encoding: Option<Encoding>,
    schema: &Schema,
    columns: &mut [ArrayRef],
    new_column: ArrayRef,
) {
    if let Some(struct_col_name) = struct_column_name(path) {
        let field_index = schema.index_of(struct_col_name).ok();
        if let Some(field_index) = field_index {
            let struct_column = columns[field_index].as_any().downcast_ref::<StructArray>();
            if let Some(struct_column) = struct_column {
                if let Some((struct_idx, _)) = struct_column.fields().find(consts::ID) {
                    // replace the encoding metadata on the struct field
                    let mut new_struct_fields = struct_column.fields().to_vec();
                    update_field_encoding_metadata(consts::ID, encoding, &mut new_struct_fields);

                    // build new struct array
                    let mut new_struct_columns = struct_column.columns().to_vec();
                    new_struct_columns[struct_idx] = new_column;
                    let new_struct_array = Arc::new(StructArray::new(
                        new_struct_fields.into(),
                        new_struct_columns,
                        struct_column.nulls().cloned(),
                    ));

                    // replace the original struct column with the new one
                    columns[field_index] = new_struct_array;
                }
            }
        }
        return;
    }

    let field_index = schema.index_of(path).ok();
    if let Some(field_index) = field_index {
        columns[field_index] = new_column
    }
}

/// Sets the encoding metadata on the field metadata for column at path.
///
/// # Arguments
/// - encoding: if `Some`, then the encoding metadata on the field will be updated to reflect the
///   new encoding. `None` will be interpreted  as plain encoding.
fn update_field_encoding_metadata(path: &str, encoding: Option<Encoding>, fields: &mut [FieldRef]) {
    if let Some(struct_col_name) = struct_column_name(path) {
        // replace the field metadata in some nested struct
        let found_field = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name().as_str() == struct_col_name);

        if let Some((idx, field)) = found_field {
            if let DataType::Struct(struct_fields) = field.data_type() {
                let mut new_struct_fields = struct_fields.to_vec();
                update_field_encoding_metadata(consts::ID, encoding, &mut new_struct_fields);

                let new_field = field
                    .as_ref()
                    .clone()
                    .with_data_type(DataType::Struct(new_struct_fields.into()));
                fields[idx] = Arc::new(new_field)
            }
        }
    }

    // not a field nested within a struct, so just replace the metadata on field where name == path
    let found_field = fields
        .iter()
        .enumerate()
        .find(|(_, f)| f.name().as_str() == path);

    if let Some((idx, field)) = found_field {
        let encoding = match encoding {
            None => consts::metadata::encodings::PLAIN,
            Some(Encoding::Delta | Encoding::DeltaRemapped) => consts::metadata::encodings::DELTA,
            Some(Encoding::AttributeQuasiDelta | Encoding::ColumnarQuasiDelta(_)) => {
                consts::metadata::encodings::QUASI_DELTA
            }
        };
        let new_field = field.as_ref().clone().with_encoding(encoding);
        fields[idx] = Arc::new(new_field)
    }
}

/// if configured to encode the ID column in the nested resource/scope struct array, this
/// helper function simply returns the name of the struct column, and otherwise returns `None`
fn struct_column_name(path: &str) -> Option<&'static str> {
    if path == RESOURCE_ID_COL_PATH {
        return Some(consts::RESOURCE);
    }

    if path == SCOPE_ID_COL_PATH {
        return Some(consts::SCOPE);
    }

    None
}

/// returns the list of transport-optimized encoding that should be applied to OTAP batches of a
/// given payload type
const fn get_column_encodings(
    payload_type: &ArrowPayloadType,
) -> &'static [ColumnEncoding<'static>] {
    match payload_type {
        ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::LogAttrs => &[ColumnEncoding {
            path: consts::PARENT_ID,
            data_type: DataType::UInt16,
            encoding: Encoding::AttributeQuasiDelta,
        }],
        ArrowPayloadType::SpanEventAttrs
        | ArrowPayloadType::SpanLinkAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => &[ColumnEncoding {
            path: consts::PARENT_ID,
            data_type: DataType::UInt32,
            encoding: Encoding::AttributeQuasiDelta,
        }],
        ArrowPayloadType::SummaryDataPoints
        | ArrowPayloadType::NumberDataPoints
        | ArrowPayloadType::HistogramDataPoints
        | ArrowPayloadType::ExpHistogramDataPoints => &[
            ColumnEncoding {
                path: consts::ID,
                data_type: DataType::UInt32,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: consts::PARENT_ID,
                data_type: DataType::UInt16,
                encoding: Encoding::Delta,
            },
        ],

        ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => &[
            ColumnEncoding {
                path: consts::ID,
                data_type: DataType::UInt32,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: consts::PARENT_ID,
                data_type: DataType::UInt32,
                encoding: Encoding::ColumnarQuasiDelta(&[consts::INT_VALUE, consts::DOUBLE_VALUE]),
            },
        ],

        ArrowPayloadType::SpanEvents => &[
            ColumnEncoding {
                path: consts::ID,
                data_type: DataType::UInt32,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: consts::PARENT_ID,
                data_type: DataType::UInt16,
                encoding: Encoding::ColumnarQuasiDelta(&[consts::NAME]),
            },
        ],
        ArrowPayloadType::SpanLinks => &[
            ColumnEncoding {
                path: consts::ID,
                data_type: DataType::UInt32,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: consts::PARENT_ID,
                data_type: DataType::UInt16,
                encoding: Encoding::ColumnarQuasiDelta(&[consts::TRACE_ID]),
            },
        ],
        ArrowPayloadType::Logs
        | &ArrowPayloadType::UnivariateMetrics
        | ArrowPayloadType::Spans
        | ArrowPayloadType::MultivariateMetrics => &[
            ColumnEncoding {
                path: consts::ID,
                data_type: DataType::UInt16,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: RESOURCE_ID_COL_PATH,
                data_type: DataType::UInt16,
                encoding: Encoding::DeltaRemapped,
            },
            ColumnEncoding {
                path: SCOPE_ID_COL_PATH,
                data_type: DataType::UInt16,
                encoding: Encoding::DeltaRemapped,
            },
        ],
        ArrowPayloadType::Unknown => &[],
    }
}

/// returns the list of columns that the OTAP record batch of this payload type should be sorted by
/// before applying any column encodings
const fn get_sort_column_paths(payload_type: &ArrowPayloadType) -> &'static [&'static str] {
    match payload_type {
        ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::LogAttrs
        | ArrowPayloadType::SpanEventAttrs
        | ArrowPayloadType::SpanLinkAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => &[
            consts::ATTRIBUTE_TYPE,
            consts::ATTRIBUTE_KEY,
            consts::ATTRIBUTE_STR,
            consts::ATTRIBUTE_INT,
            consts::ATTRIBUTE_DOUBLE,
            consts::ATTRIBUTE_BOOL,
            consts::ATTRIBUTE_BYTES,
            consts::PARENT_ID,
        ],
        ArrowPayloadType::SummaryDataPoints
        | ArrowPayloadType::NumberDataPoints
        | ArrowPayloadType::HistogramDataPoints
        | ArrowPayloadType::ExpHistogramDataPoints => &[consts::PARENT_ID],

        ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => {
            &[consts::INT_VALUE, consts::DOUBLE_VALUE, consts::PARENT_ID]
        }

        ArrowPayloadType::SpanEvents => &[consts::NAME, consts::PARENT_ID],
        ArrowPayloadType::SpanLinks => &[consts::TRACE_ID, consts::PARENT_ID],
        ArrowPayloadType::Logs => &[RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH, consts::TRACE_ID],
        ArrowPayloadType::Spans => &[
            RESOURCE_ID_COL_PATH,
            SCOPE_ID_COL_PATH,
            consts::NAME,
            consts::TRACE_ID,
        ],
        ArrowPayloadType::UnivariateMetrics | ArrowPayloadType::MultivariateMetrics => &[
            RESOURCE_ID_COL_PATH,
            SCOPE_ID_COL_PATH,
            consts::METRIC_TYPE,
            consts::NAME,
        ],
        ArrowPayloadType::Unknown => &[],
    }
}

/// Sort the record batch with this payload type by columns that will hopefully give us the best
/// compression ratio.
fn sort_record_batch(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    let sort_columns_paths = get_sort_column_paths(payload_type);

    // choose which columns to sort by -- only sort by the columns that are present
    let schema = record_batch.schema_ref();
    let mut sort_inputs = vec![];
    let columns = record_batch.columns();
    for path in sort_columns_paths {
        if let Some(column) = access_column(path, schema, columns) {
            sort_inputs.push(column)
        }
    }

    if sort_inputs.is_empty() {
        Ok(record_batch.clone())
    } else {
        let sort_columns = sort_inputs
            .iter()
            .map(|array| SortColumn {
                values: array.clone(),
                options: Some(SortOptions {
                    descending: false,
                    nulls_first: false,
                }),
            })
            .collect::<Vec<_>>();

        // safety: this will only panic if there's some columns we pass that aren't supported
        // either in row converter or by arrow's sort_to_indices kernel, but based on the columns
        // we`re choosing in [`get_sort_column_paths`] this shouldn't happen
        let indices = sort_to_indices(&sort_columns).expect("error sorting to column indices");

        if indices.values().is_sorted() {
            Ok(record_batch.clone())
        } else {
            // safety: take_record_batch will only error here if we're taking indices not outside the
            // range of the batch. But since we've calculated the indices by sorting the existing
            // indices this shouldn't happen
            Ok(take_record_batch(record_batch, &indices).expect("error taking sort result"))
        }
    }
}

struct EncodedColumnResult {
    new_column: ArrayRef,
    remapping: Option<RemappedParentIds>,
}

/// Creates a new array of the same length where the array values are delta encoded.
///
/// This is typically used on the ID columns of various record batches. Because the OTAP record
/// are typically not sorted by these ID and the ID columns are unsigned types, we cannot just
/// apply delta encoding directly else we'd end up with negative deltas. To solve this, we create
/// a new ID column and also possibly return a mapping of old IDs to new IDs (only if some IDs
/// were assigned new values).
fn create_new_delta_encoded_column_from<T>(
    column: &PrimitiveArray<T>,
) -> Result<EncodedColumnResult>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: From<u8> + Add<Output = T::Native> + AddAssign + PartialOrd,
    RemappedParentIds: From<Vec<<T as ArrowPrimitiveType>::Native>>,
{
    // check some early return conditions where we don't need to compute new column:
    // - empty column
    // - all nulls column
    if column.is_empty() || column.null_count() == column.len() {
        return Ok(EncodedColumnResult {
            new_column: Arc::new(column.clone()),
            remapping: None,
        });
    }

    let zero = T::Native::from(0u8);
    let one = T::Native::from(1u8);

    // safety: max will only return an error here if the array is all nulls or empty, which we've
    // already validated that it is not
    let remappings_len =
        one + arrow::compute::max(column).expect("error computing size of remappings");
    let mut remappings = vec![zero; remappings_len.as_usize()];

    let mut curr_id: T::Native = zero;
    let mut new_buffer = MutableBuffer::with_capacity(column.len() * size_of::<T::Native>());

    let mut prev_value = None;
    for val in column.iter() {
        match val {
            Some(val) => match prev_value {
                Some(prev) => {
                    if val == prev {
                        // Safety: we've allocated enough space for the new value so calling
                        // push_unchecked here is safe and saves us having to check the buffer
                        // reservation on each push.
                        #[allow(unsafe_code)]
                        unsafe {
                            new_buffer.push_unchecked(zero)
                        };
                    } else {
                        // Safety: see comments on push_unchecked call above
                        #[allow(unsafe_code)]
                        unsafe {
                            new_buffer.push_unchecked(one)
                        };

                        curr_id += one;
                        remappings[val.as_usize()] = curr_id;
                        prev_value = Some(val);
                    }
                }
                None => {
                    new_buffer.push(zero);
                    prev_value = Some(val);
                }
            },
            None => {
                // push the default value & we'll clone the null buffer later
                // Safety: see comments on push_unchecked call above
                #[allow(unsafe_code)]
                unsafe {
                    new_buffer.push_unchecked(zero)
                };
            }
        }
    }

    // Check if all the new IDs we've assigned are the same as the old IDs. This could happen
    // if the passed column was already a sorted, incrementing sequence. This is an important
    // optimization because it allows us to avoid creating a new column for the child's parent_id
    // column if there were no changes
    let mut all_ids_same = true;
    for (i, remapping) in remappings.iter().enumerate() {
        if remapping.as_usize() != i {
            all_ids_same = false;
            break;
        }
    }
    let remapping = if all_ids_same {
        None
    } else {
        Some(RemappedParentIds::from(remappings))
    };

    let nulls = column.nulls().cloned();
    let new_buffer = ScalarBuffer::<T::Native>::new(new_buffer.into(), 0, column.len());
    let new_column = PrimitiveArray::<T>::new(new_buffer, nulls);

    Ok(EncodedColumnResult {
        new_column: Arc::new(new_column),
        remapping,
    })
}

/// Context required to remap parent IDs.
///
/// After an ID column has been encoded, the IDs may change so the parent ID columns on child
/// batches will need to be remapped. This provides information to the caller about the new IDs
/// and also which ID column they correspond to.
#[derive(Debug, PartialEq)]
pub struct ParentIdRemapping {
    /// The path to the column that contains the parent IDs that have changed
    pub column_path: &'static str,

    /// The remapped parent IDs
    pub remapped_ids: RemappedParentIds,
}

impl ParentIdRemapping {
    fn new(column_path: &'static str, remapped_ids: RemappedParentIds) -> Self {
        Self {
            column_path,
            remapped_ids,
        }
    }
}

/// This represents a mapping between ID values which may have been reassigned during encoding.
/// Internally, the representation is a vector where the index would bethe old ID and the value
/// at that index would be the new ID.
///
/// For example, if we had the following remappings of old ID -> new ID
/// - 1 -> 2
/// - 2 -> 0
/// - 0 -> 1
///
/// Then the values of the remapping would be `[1, 2, 0]`
///
#[derive(Debug, PartialEq)]
pub enum RemappedParentIds {
    /// remapping of IDs for a u16 type ID column
    UInt16(Vec<u16>),
    /// remapping of IDs for a u32 type ID column
    UInt32(Vec<u32>),
}

impl From<Vec<u32>> for RemappedParentIds {
    fn from(ids: Vec<u32>) -> Self {
        Self::UInt32(ids)
    }
}

impl From<Vec<u16>> for RemappedParentIds {
    fn from(ids: Vec<u16>) -> Self {
        Self::UInt16(ids)
    }
}

/// Creates a new parent_id column on the passed record batch according to the passed remapping.
///
/// The remapping should contain a vector that has the same type as the parent ID column. For
/// example if the parent_id column is a UInt32Array, then remapping should be the UInt32 variant.
///
/// The payload type should match the type of record batch that is passed. This is because this
/// argument is used to determine the proper way to materialize possibly encoded parent IDs before
/// remapping them.
pub fn remap_parent_ids(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
    remapping: &RemappedParentIds,
) -> Result<RecordBatch> {
    let schema = record_batch.schema_ref();
    let field_idx = match schema.index_of(consts::PARENT_ID) {
        Ok(idx) => idx,
        _ => {
            // No parent ID column, so nothing to remap
            return Ok(record_batch.clone());
        }
    };

    // Check if the column is plain encoded. If not, we'll need to remove any encoding
    // before we remap the IDs
    let field = schema.field(field_idx);
    let metadata = field.metadata();
    let is_plain_encoded = matches!(
        metadata.get(consts::metadata::COLUMN_ENCODING).map(String::as_str),
        Some(enc) if enc == consts::metadata::encodings::PLAIN
    );
    let record_batch = if is_plain_encoded {
        record_batch.clone()
    } else {
        remove_parent_id_column_encoding(payload_type, record_batch)?
    };

    // reassign schema to the new record batch schema, which may have updated metadata due to
    // having maybe materialized the encoded parent IDs
    let schema = record_batch.schema();

    let parent_id_col = record_batch.column(field_idx);
    let new_parent_ids = match remapping {
        RemappedParentIds::UInt16(ids) => {
            let parent_id_col = parent_id_col
                .as_any()
                .downcast_ref::<UInt16Array>()
                .with_context(|| error::InvalidListArraySnafu {
                    expect_oneof: vec![DataType::UInt16],
                    actual: parent_id_col.data_type().clone(),
                })?;
            Arc::new(remap_parent_id_col(parent_id_col, ids)?) as ArrayRef
        }
        RemappedParentIds::UInt32(ids) => {
            let parent_id_col = parent_id_col
                .as_any()
                .downcast_ref::<UInt32Array>()
                .with_context(|| error::InvalidListArraySnafu {
                    expect_oneof: vec![DataType::UInt32],
                    actual: parent_id_col.data_type().clone(),
                })?;
            Arc::new(remap_parent_id_col(parent_id_col, ids)?) as ArrayRef
        }
    };

    let new_columns = record_batch.columns().iter().enumerate().map(|(i, col)| {
        if i == field_idx {
            new_parent_ids.clone()
        } else {
            col.clone()
        }
    });

    // Safety: this is safe because we have replaced a column with another column of the same type
    // (which means the schema matches the columns), and the column we're replacing has the same
    // length as the replacement
    Ok(RecordBatch::try_new(schema, new_columns.collect())
        .expect("could not create new record batch"))
}

fn remap_parent_id_col<T: ArrowPrimitiveType>(
    parent_ids: &PrimitiveArray<T>,
    remapped_ids: &[T::Native],
) -> Result<PrimitiveArray<T>> {
    let mut new_parent_ids =
        MutableBuffer::with_capacity(parent_ids.len() * size_of::<T::Native>());
    for val in parent_ids {
        match val {
            Some(id) => {
                let remapped_id = remapped_ids[id.as_usize()];
                // Safety: we've preallocated the buffer with the correct length, so we're safe to
                // call push_unchecked here and avoid the cost of checking the capacity reservation
                // for every array
                #[allow(unsafe_code)]
                unsafe {
                    new_parent_ids.push_unchecked(remapped_id)
                };
            }
            None => {
                // Safety: see the comment in the block above about why this is safe
                #[allow(unsafe_code)]
                unsafe {
                    new_parent_ids.push_unchecked(T::default_value())
                }
            }
        }
    }

    Ok(PrimitiveArray::<T>::new(
        ScalarBuffer::new(new_parent_ids.into(), 0, parent_ids.len()),
        parent_ids.nulls().cloned(),
    ))
}

fn remove_parent_id_column_encoding(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    match payload_type {
        ArrowPayloadType::LogAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::ScopeAttrs => materialize_parent_id_for_attributes::<u16>(record_batch),

        ArrowPayloadType::SpanLinkAttrs
        | ArrowPayloadType::SpanEventAttrs
        | ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => {
            materialize_parent_id_for_attributes::<u32>(record_batch)
        }

        ArrowPayloadType::SpanEvents => {
            materialize_parent_ids_by_columns::<u16>(record_batch, [consts::NAME])
        }
        ArrowPayloadType::SpanLinks => {
            materialize_parent_ids_by_columns::<u16>(record_batch, [consts::TRACE_ID])
        }

        ArrowPayloadType::NumberDataPoints
        | ArrowPayloadType::SummaryDataPoints
        | ArrowPayloadType::HistogramDataPoints
        | ArrowPayloadType::ExpHistogramDataPoints => {
            remove_delta_encoding::<UInt32Type>(record_batch, consts::PARENT_ID)
        }

        ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => {
            materialize_parent_id_for_exemplars::<u32>(record_batch)
        }

        ArrowPayloadType::Logs
        | ArrowPayloadType::UnivariateMetrics
        | ArrowPayloadType::MultivariateMetrics
        | ArrowPayloadType::Spans
        | ArrowPayloadType::Unknown => {
            // nothing to do b/c there are no parent ID field for these payload types
            Ok(record_batch.clone())
        }
    }
}

/// apply transport-optimized encodings to the record batch's columns
pub fn apply_transport_optimized_encodings(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<(RecordBatch, Option<Vec<ParentIdRemapping>>)> {
    let column_encodings = get_column_encodings(payload_type);

    let schema = record_batch.schema();

    // determine which columns need to be encoded, and which are already encoded..
    let mut count_to_apply = 0;
    let mut count_already_encoded = 0;
    for column_encoding in column_encodings {
        match is_column_encoded(column_encoding.path, &schema) {
            Some(true) => count_already_encoded += 1,
            Some(false) => count_to_apply += 1,
            None => {}
        }
    }

    if count_to_apply == 0 {
        // nothing to do - the entire record batch already has the columns transport-optimized
        return Ok((record_batch.clone(), None));
    }

    // sort record batch before applying the encoding. This will give us the best compression ratio
    // for columns which may have many repeated sequences of the same value
    let record_batch = if count_already_encoded > 0 {
        // if some of the columns are encoded, we need to  materialize them, otherwise we'll end up
        // with mixed up sequences of delta encoding during sorting
        let rb = remove_transport_optimized_encodings(*payload_type, record_batch)?;
        sort_record_batch(payload_type, &rb)
    } else {
        sort_record_batch(payload_type, record_batch)
    }?;

    let mut columns = record_batch.columns().to_vec();
    let mut fields = schema.fields.to_vec();
    let mut remapped_parent_ids = Vec::with_capacity(column_encodings.len());

    for column_encoding in column_encodings {
        let encoding_result = match &column_encoding.data_type {
            DataType::UInt16 => {
                apply_encoding_for_id_type::<UInt16Type>(&record_batch, column_encoding)?
            }
            DataType::UInt32 => {
                apply_encoding_for_id_type::<UInt32Type>(&record_batch, column_encoding)?
            }
            _ => {
                return Err(error::InvalidListArraySnafu {
                    expect_oneof: vec![DataType::UInt16, DataType::UInt32],
                    actual: column_encoding.data_type.clone(),
                }
                .build());
            }
        };

        if let Some(encoding_result) = encoding_result {
            if let Some(remapping) = encoding_result.remapping {
                remapped_parent_ids.push(ParentIdRemapping::new(column_encoding.path, remapping));
            }

            replace_column(
                column_encoding.path,
                Some(column_encoding.encoding),
                &schema,
                &mut columns,
                encoding_result.new_column,
            );
            update_field_encoding_metadata(
                column_encoding.path,
                Some(column_encoding.encoding),
                &mut fields,
            );
        }
    }

    // Safety: we should have a valid schema with matching columns where all columns are the
    // same length, so it should be safe to call expect here
    let record_batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("error constructing transport encoded record batch");

    Ok((record_batch, Some(remapped_parent_ids)))
}

/// Applies the specified column encoding to the record batch. Generic over the ID column type
fn apply_encoding_for_id_type<T>(
    record_batch: &RecordBatch,
    column_encoding: &ColumnEncoding<'_>,
) -> Result<Option<EncodedColumnResult>>
where
    T: ArrowPrimitiveType,
    T::Native: From<u8> + ParentId + Sub<Output = T::Native>,
    RemappedParentIds: From<Vec<T::Native>>,
{
    let column = match access_column(
        column_encoding.path,
        record_batch.schema_ref(),
        record_batch.columns(),
    ) {
        Some(column) => column,
        None => return Ok(None),
    };

    let column = column
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .context(error::InvalidListArraySnafu {
            expect_oneof: vec![T::DATA_TYPE],
            actual: column.data_type().clone(),
        })?;

    let result = match column_encoding.encoding {
        Encoding::DeltaRemapped => create_new_delta_encoded_column_from::<T>(column),

        Encoding::AttributeQuasiDelta => {
            let new_column = transport_encode_parent_id_for_attributes::<T>(record_batch)?;
            Ok(EncodedColumnResult {
                new_column,
                remapping: None,
            })
        }
        Encoding::ColumnarQuasiDelta(columns) => {
            let new_column = transport_encode_parent_id_for_columns::<T>(record_batch, columns)?;
            Ok(EncodedColumnResult {
                new_column,
                remapping: None,
            })
        }
        Encoding::Delta => {
            // Adding delta encoding here by repurposing adding quasi-delta encoding except that
            // there are no columns to check equality between rows which would  break the sequence
            // of delta encodings.
            //
            // TODO we might get slightly better performance by to creating a dedicated function to
            // do the delta encoding directly.
            let new_column = transport_encode_parent_id_for_columns::<T>(record_batch, &[])?;
            Ok(EncodedColumnResult {
                new_column,
                remapping: None,
            })
        }
    };

    Some(result).transpose()
}

pub fn remove_transport_optimized_encodings(
    payload_type: ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    match payload_type {
        ArrowPayloadType::Logs
        | ArrowPayloadType::UnivariateMetrics
        | ArrowPayloadType::MultivariateMetrics
        | ArrowPayloadType::Spans => {
            // remove delta encoding from ID column on struct arrays ..
            let schema = record_batch.schema_ref();
            let mut columns = record_batch.columns().to_vec();
            let mut fields = schema.fields.to_vec();
            for struct_id_path in [RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH] {
                if is_column_encoded(struct_id_path, schema) == Some(false) {
                    continue;
                }

                if let Some(struct_ids) = access_column(struct_id_path, schema, &columns) {
                    let struct_ids = struct_ids
                        .as_any()
                        .downcast_ref::<UInt16Array>()
                        .with_context(|| error::InvalidListArraySnafu {
                            expect_oneof: vec![DataType::UInt16],
                            actual: struct_ids.data_type().clone(),
                        })?;

                    let new_struct_ids = remove_delta_encoding_from_column(struct_ids);
                    replace_column(
                        struct_id_path,
                        None,
                        schema,
                        &mut columns,
                        Arc::new(new_struct_ids),
                    );
                    update_field_encoding_metadata(struct_id_path, None, &mut fields);
                }
            }

            // safety: we should have a valid schema with matching columns where all columns are the
            // same length, so it should be safe to call expect here
            let schema = Schema::new(fields);
            let rb = RecordBatch::try_new(Arc::new(schema), columns)
                .expect("could not create record batch after removing struct ID encodings");

            // remove the delta encoding on the ID column and return
            remove_delta_encoding::<UInt16Type>(&rb, consts::ID)
        }

        ArrowPayloadType::LogAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs => materialize_parent_id_for_attributes::<u16>(record_batch),

        ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => {
            materialize_parent_id_for_attributes::<u32>(record_batch)
        }

        ArrowPayloadType::SpanEvents => {
            let rb = remove_delta_encoding::<UInt32Type>(record_batch, consts::ID)?;
            materialize_parent_ids_by_columns::<u16>(&rb, [consts::NAME])
        }
        ArrowPayloadType::SpanLinks => {
            let rb = remove_delta_encoding::<UInt32Type>(record_batch, consts::ID)?;
            materialize_parent_ids_by_columns::<u16>(&rb, [consts::TRACE_ID])
        }
        ArrowPayloadType::SpanEventAttrs | ArrowPayloadType::SpanLinkAttrs => {
            materialize_parent_id_for_attributes::<u32>(record_batch)
        }
        ArrowPayloadType::NumberDataPoints
        | ArrowPayloadType::SummaryDataPoints
        | ArrowPayloadType::HistogramDataPoints
        | ArrowPayloadType::ExpHistogramDataPoints => {
            let rb = remove_delta_encoding::<UInt32Type>(record_batch, consts::ID)?;
            remove_delta_encoding::<UInt16Type>(&rb, consts::PARENT_ID)
        }

        ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => {
            let rb = remove_delta_encoding::<UInt32Type>(record_batch, consts::ID)?;
            materialize_parent_id_for_exemplars::<u32>(&rb)
        }

        ArrowPayloadType::Unknown => {
            // do nothing
            Ok(record_batch.clone())
        }
    }
}

/// Adds the quasi-delta/transport optimized encoding to the encoding to the parent ID column.
///
/// See the comments on [`super::materialize_parent_id_for_attributes`] for more information about
/// the encoding scheme.
pub fn transport_encode_parent_id_for_attributes<T>(record_batch: &RecordBatch) -> Result<ArrayRef>
where
    T: ArrowPrimitiveType,
    T::Native: ParentId + Sub<Output = T::Native>,
{
    // downcast parent ID into an array of the primitive type
    let parent_id_arr = T::Native::get_parent_id_column(record_batch)?
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        // safety: this should be OK because we're basically recasting the primitive array to the
        // same type, where it contains T::Native instead of <T as ParentId>::ArrayType::Native
        // this cast avoids some messy type constraints on the method signature
        .expect("unable to downcast to type");

    if record_batch.num_rows() == 0 {
        // safety: we've already called `T::get_parent_id_column`, which checks that the column
        // so it should be safe to call expect here
        return Ok(record_batch
            .column_by_name(consts::PARENT_ID)
            .expect("error accessing parent ID column")
            .clone());
    }

    // check that the column hasn't already been encoded. If so, we want to avoid re-encoding
    let column_encoding = get_field_metadata(
        record_batch.schema_ref(),
        consts::PARENT_ID,
        consts::metadata::COLUMN_ENCODING,
    );
    if let Some(consts::metadata::encodings::QUASI_DELTA) = column_encoding {
        // safety: we've already called `T::get_parent_id_column`, which checks that the column
        // so it should be safe to call expect here
        return Ok(record_batch
            .column_by_name(consts::PARENT_ID)
            .expect("error accessing parent ID column")
            .clone());
    }

    let keys_arr =
        record_batch
            .column_by_name(consts::ATTRIBUTE_KEY)
            .context(error::ColumnNotFoundSnafu {
                name: consts::ATTRIBUTE_KEY,
            })?;
    let key_eq_next = create_next_eq_array_for_array(keys_arr);

    let type_arr = record_batch
        .column_by_name(consts::ATTRIBUTE_TYPE)
        .context(error::ColumnNotFoundSnafu {
            name: consts::ATTRIBUTE_TYPE,
        })?;
    let types_eq_next = create_next_element_equality_array(type_arr)?;
    let type_arr = get_u8_array(record_batch, consts::ATTRIBUTE_TYPE)?;

    let val_str_arr = record_batch.column_by_name(consts::ATTRIBUTE_STR);
    let val_int_arr = record_batch.column_by_name(consts::ATTRIBUTE_INT);
    let val_double_arr = record_batch.column_by_name(consts::ATTRIBUTE_DOUBLE);
    let val_bool_arr = record_batch.column_by_name(consts::ATTRIBUTE_BOOL);
    let val_bytes_arr = record_batch.column_by_name(consts::ATTRIBUTE_BYTES);

    let mut encoded_parent_ids = PrimitiveArray::<T>::builder(record_batch.num_rows());

    // below we're iterating through the record batch and each time we find a contiguous range
    // where all the types & attribute keys are the same, we use the "eq" compute kernel to
    // compare all the values. Then we use the resulting next-element equality array for the
    // values to determine if there should be delta encoding
    let mut curr_range_start = 0;
    for idx in 0..record_batch.num_rows() {
        // check if we've found the end of a range of where all the type & attribute are the same
        let found_range_end = if idx == types_eq_next.len() {
            true // end of list
        } else {
            !types_eq_next.value(idx) || !key_eq_next.value(idx)
        };

        // when we find the range end, decode the parent ID values
        if found_range_end {
            let value_type = AttributeValueType::try_from(type_arr.value(curr_range_start))
                .context(error::UnrecognizedAttributeValueTypeSnafu)?;
            let value_arr = match value_type {
                AttributeValueType::Str => val_str_arr,
                AttributeValueType::Int => val_int_arr,
                AttributeValueType::Bool => val_bool_arr,
                AttributeValueType::Bytes => val_bytes_arr,
                AttributeValueType::Double => val_double_arr,

                // These types are always considered not equal for purposes of determining
                // whether to delta encode parent ID
                AttributeValueType::Map | AttributeValueType::Slice | AttributeValueType::Empty => {
                    None
                }
            };

            // add the first value from this range to the parent IDs
            let first_parent_id = parent_id_arr
                .value_at(curr_range_start)
                // safety: there's a check at the beginning of this function to ensure that
                // the batch is not empty
                .expect("expect the batch not to be empty");
            encoded_parent_ids.append_value(first_parent_id);

            if let Some(value_arr) = value_arr {
                // if we have a value array here, we want the parent ID to be delta encoded
                let range_length = idx + 1 - curr_range_start;
                let values_range = value_arr.slice(curr_range_start, range_length);
                let values_eq_next = create_next_element_equality_array(&values_range)?;

                for batch_idx in (curr_range_start + 1)..=idx {
                    let curr_parent_id = parent_id_arr.value_at_or_default(batch_idx);
                    let prev_value_range_idx = batch_idx - 1 - curr_range_start;

                    let parent_id_or_delta = if values_eq_next.value(prev_value_range_idx)
                        && !values_eq_next.is_null(prev_value_range_idx)
                    {
                        // value at current index equals previous so we're delta encoded
                        let prev_parent_id = parent_id_arr.value_at_or_default(batch_idx - 1);
                        curr_parent_id - prev_parent_id
                    } else {
                        // otherwise, we break the delta encoding
                        curr_parent_id
                    };
                    encoded_parent_ids.append_value(parent_id_or_delta);
                }
            } else {
                // if we're here, we've determined that the parent ID values are not delta encoded
                // because the type doesn't support it
                for batch_idx in (curr_range_start + 1)..(idx + 1) {
                    encoded_parent_ids.append_value(parent_id_arr.value_at_or_default(batch_idx));
                }
            }

            curr_range_start = idx + 1;
        }
    }

    let encoded_parent_ids = Arc::new(encoded_parent_ids.finish());
    Ok(encoded_parent_ids)
}

/// This function adds the quasi-delta encoding to the parent ID column. Subsequent parent IDs with
/// column values that are equal to the previous row's column values will be delta encoded. The
/// list of columns to check for equality is specified by `equality_column_names`.
pub fn transport_encode_parent_id_for_columns<T>(
    record_batch: &RecordBatch,
    equality_column_names: &[&str],
) -> Result<ArrayRef>
where
    T: ArrowPrimitiveType,
    T::Native: ParentId + Sub<Output = T::Native>,
{
    // downcast parent ID into an array of the primitive type
    let parent_ids = T::Native::get_parent_id_column(record_batch)?
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        // safety: this should be OK because we're basically recasting the primitive array to the
        // same type, where it contains T::Native instead of <T as ParentId>::ArrayType::Native
        // this cast avoids some messy type constraints on the method signature
        .expect("unable to downcast to own type");

    // if the record batch is empty, nothing to decode so return early
    if record_batch.num_rows() == 0 {
        // safety: we've already called `T::get_parent_id_column`, which checks that the column
        // so it should be safe to call expect here
        return Ok(record_batch
            .column_by_name(consts::PARENT_ID)
            .expect("error accessing parent ID column")
            .clone());
    }

    // check that the column hasn't already been encoded. If so, we want to avoid re-encoding
    let column_encoding = get_field_metadata(
        record_batch.schema_ref(),
        consts::PARENT_ID,
        consts::metadata::COLUMN_ENCODING,
    );
    if let Some(consts::metadata::encodings::QUASI_DELTA) = column_encoding {
        // safety: we've already called `T::get_parent_id_column`, which checks that the column
        // so it should be safe to call expect here
        return Ok(record_batch
            .column_by_name(consts::PARENT_ID)
            .expect("error accessing parent ID column")
            .clone());
    }

    // here we're building up the next-element equality array for multiple columns by 'and'ing
    // the equality array for each column together. This gives us an array that is true if all
    // the values in each column are equal (index offset by 1). If some column doesn't exist, in
    // this case assume this means null values which are equal

    // we start off with an array full of `true`, indicating all rows are equal. This is the
    // assumption to make if none of the equality columns are present on the record batch.
    let mut eq_next: BooleanArray = std::iter::repeat_n(Some(true), parent_ids.len() - 1).collect();

    for column_name in equality_column_names {
        if let Some(column) = record_batch.column_by_name(column_name) {
            let eq_next_column = create_next_element_equality_array(column)?;
            eq_next =
                and(&eq_next_column, &eq_next).expect("can 'and' arrays together of same length")
        }
    }

    let mut encoded_parent_ids = PrimitiveBuilder::<T>::with_capacity(record_batch.num_rows());

    // safety: there's a check at the beginning of this method that the batch is not empty
    let first_parent_id = parent_ids
        .value_at(0)
        .expect("expect batch not to be empty");
    encoded_parent_ids.append_value(first_parent_id);

    for i in 1..record_batch.num_rows() {
        let curr_parent_id = parent_ids.value(i);
        let parent_id_or_delta = if eq_next.value(i - 1) {
            let prev_parent_id = parent_ids.value_at_or_default(i - 1);
            curr_parent_id - prev_parent_id
        } else {
            curr_parent_id
        };
        encoded_parent_ids.append_value(parent_id_or_delta);
    }

    let encoded_parent_ids = Arc::new(encoded_parent_ids.finish());
    Ok(encoded_parent_ids)
}

#[cfg(test)]
mod test {
    use std::vec;

    use arrow::{
        array::{
            FixedSizeBinaryArray, Float64Array, Int64Array, StringArray, StructArray,
            TimestampNanosecondArray, UInt8Array, UInt16Array,
        },
        datatypes::{Field, Fields, TimeUnit},
    };

    use crate::{otlp::attributes::store::AttributeValueType, schema::FieldExt};

    use super::*;

    #[test]
    fn test_access_column_basic() {
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, true)]);
        let columns = vec![Arc::new(UInt8Array::from_iter_values([1, 2])) as ArrayRef];

        let mut column_encoding = ColumnEncoding {
            path: "a",
            data_type: DataType::UInt8,
            encoding: Encoding::DeltaRemapped,
        };

        let column = access_column(column_encoding.path, &schema, columns.as_ref()).unwrap();
        assert_eq!(*column, *columns[0]);

        // assert what happens if the column isn't present
        column_encoding.path = "b";
        assert!(access_column(column_encoding.path, &schema, columns.as_ref()).is_none())
    }

    #[test]
    fn test_access_column_nested() {
        let struct_fields: Fields = vec![Field::new("id", DataType::UInt16, true)].into();
        let schema = Schema::new(vec![
            Field::new("resource", DataType::Struct(struct_fields.clone()), true),
            Field::new("scope", DataType::Struct(struct_fields.clone()), true),
        ]);

        let resource_ids = UInt16Array::from_iter_values(vec![1]);
        let scope_ids = UInt16Array::from_iter_values(vec![2]);

        let columns = vec![
            Arc::new(StructArray::new(
                struct_fields.clone(),
                vec![Arc::new(resource_ids.clone())],
                None,
            )) as ArrayRef,
            Arc::new(StructArray::new(
                struct_fields.clone(),
                vec![Arc::new(scope_ids.clone())],
                None,
            )) as ArrayRef,
        ];

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt8,
            encoding: Encoding::DeltaRemapped,
        };

        let column = access_column(column_encoding.path, &schema, columns.as_ref()).unwrap();
        assert_eq!(*column, resource_ids);

        column_encoding.path = SCOPE_ID_COL_PATH;
        let column = access_column(column_encoding.path, &schema, columns.as_ref()).unwrap();
        assert_eq!(*column, scope_ids)
    }

    #[test]
    fn test_access_column_not_present_nested() {
        let schema = Schema::new(vec![Field::new(
            "scope",
            DataType::Struct(Vec::<Field>::new().into()),
            true,
        )]);

        let columns = vec![Arc::new(StructArray::new_empty_fields(1, None)) as ArrayRef];

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt8,
            encoding: Encoding::DeltaRemapped,
        };

        // assert what happens if the struct isn't present
        assert!(access_column(column_encoding.path, &schema, columns.as_ref()).is_none());

        // assert what happens if the struct is present, but the ID field isn't present
        column_encoding.path = SCOPE_ID_COL_PATH;
        assert!(access_column(column_encoding.path, &schema, columns.as_ref()).is_none());
    }

    #[test]
    fn test_is_column_encoded_basic() {
        let schema = Schema::new(vec![
            Field::new("a", DataType::Utf8, false).with_plain_encoding(),
            Field::new("b", DataType::UInt16, false),
        ]);

        let mut column_encoding = ColumnEncoding {
            path: "a",
            data_type: DataType::UInt16,
            encoding: Encoding::DeltaRemapped,
        };

        assert!(!is_column_encoded("a", &schema).unwrap());

        // ensure that no metadata means column is encoded
        column_encoding.path = "b";
        assert!(is_column_encoded("b", &schema).unwrap());
    }

    #[test]
    fn test_is_column_encoded_nested() {
        let schema = Schema::new(vec![
            Field::new(
                "resource",
                DataType::Struct(
                    vec![Field::new("id", DataType::UInt16, true).with_plain_encoding()].into(),
                ),
                true,
            ),
            Field::new(
                "scope",
                DataType::Struct(vec![Field::new("id", DataType::UInt16, true)].into()),
                true,
            ),
        ]);

        assert!(!is_column_encoded(RESOURCE_ID_COL_PATH, &schema).unwrap());
        assert!(is_column_encoded(SCOPE_ID_COL_PATH, &schema).unwrap());
    }

    #[test]
    fn test_sort_columns_multi_column() {
        let input_data = vec![(2, 1, 1), (2, 0, 1), (2, 1, 0)];

        let expected_data = vec![(2, 0, 1), (2, 1, 0), (2, 1, 1)];

        fn to_batch(data: Vec<(u16, u16, u128)>) -> RecordBatch {
            let struct_fields: Fields = vec![Field::new(consts::ID, DataType::UInt16, true)].into();

            let schema = Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), false),
            ]));
            let resource_ids = UInt16Array::from_iter_values(data.iter().map(|d| d.0));
            let scope_ids = UInt16Array::from_iter_values(data.iter().map(|d| d.1));
            let trace_ids =
                FixedSizeBinaryArray::try_from_iter(data.iter().map(|d| u128::to_be_bytes(d.2)))
                    .unwrap();

            RecordBatch::try_new(
                schema.clone(),
                vec![
                    // resource
                    Arc::new(StructArray::new(
                        struct_fields.clone(),
                        vec![Arc::new(resource_ids)],
                        None,
                    )),
                    Arc::new(StructArray::new(
                        struct_fields.clone(),
                        vec![Arc::new(scope_ids)],
                        None,
                    )),
                    Arc::new(trace_ids),
                ],
            )
            .unwrap()
        }

        let result = sort_record_batch(&ArrowPayloadType::Logs, &to_batch(input_data)).unwrap();
        let expected = to_batch(expected_data);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_columns_none_present() {
        // Sometimes the sorting columns in the record batch will be optional, in which case
        // there might be none present and so the sorting should be basically a noop

        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt8, true),
            Field::new(
                consts::TIME_UNIX_NANO,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
        ]));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![4, 1, 8])),
                Arc::new(TimestampNanosecondArray::from_iter_values(vec![5, 1, 4])),
            ],
        )
        .unwrap();

        let result = sort_record_batch(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result, input)
    }

    #[test]
    fn test_sort_columns_single_column() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt8, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![4, 1, 8])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![5u128, 1, 4].into_iter().map(|i| i.to_be_bytes()),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        // this should now be sorted by trace_id
        let result = sort_record_batch(&ArrowPayloadType::Logs, &input).unwrap();

        let expected = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![1, 8, 4])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![1u128, 4, 5].into_iter().map(|i| i.to_be_bytes()),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_multi_columns_with_nulls() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        ]));
        let data = vec![
            (AttributeValueType::Str as u8, "a", Some("a"), None),
            (AttributeValueType::Str as u8, "a", None, None),
            (AttributeValueType::Int as u8, "a", None, Some(1)),
            (AttributeValueType::Int as u8, "a", None, Some(2)),
            (AttributeValueType::Str as u8, "b", Some("c"), None),
            (AttributeValueType::Int as u8, "a", None, None),
            (AttributeValueType::Str as u8, "b", Some("a"), None),
            (AttributeValueType::Str as u8, "b", Some("b"), None),
            (AttributeValueType::Str as u8, "b", None, None),
            (AttributeValueType::Str as u8, "a", Some("c"), None),
            (AttributeValueType::Int as u8, "b", None, Some(1)),
            (AttributeValueType::Str as u8, "a", Some("b"), None),
            (AttributeValueType::Int as u8, "b", None, Some(2)),
            (AttributeValueType::Int as u8, "a", None, Some(3)),
            (AttributeValueType::Int as u8, "b", None, Some(3)),
            (AttributeValueType::Int as u8, "b", None, None),
        ];

        let attr_types = UInt8Array::from_iter_values(data.iter().map(|d| d.0));
        let attr_keys = StringArray::from_iter_values(data.iter().map(|d| d.1));
        let attr_strs = StringArray::from_iter(data.iter().map(|d| d.2));
        let attr_ints = Int64Array::from_iter(data.iter().map(|d| d.3));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(attr_types),
                Arc::new(attr_keys),
                Arc::new(attr_strs),
                Arc::new(attr_ints),
            ],
        )
        .unwrap();

        let result = sort_record_batch(&ArrowPayloadType::ResourceAttrs, &input).unwrap();

        let expected_data = vec![
            (AttributeValueType::Str as u8, "a", Some("a"), None),
            (AttributeValueType::Str as u8, "a", Some("b"), None),
            (AttributeValueType::Str as u8, "a", Some("c"), None),
            (AttributeValueType::Str as u8, "a", None, None),
            (AttributeValueType::Str as u8, "b", Some("a"), None),
            (AttributeValueType::Str as u8, "b", Some("b"), None),
            (AttributeValueType::Str as u8, "b", Some("c"), None),
            (AttributeValueType::Str as u8, "b", None, None),
            (AttributeValueType::Int as u8, "a", None, Some(1)),
            (AttributeValueType::Int as u8, "a", None, Some(2)),
            (AttributeValueType::Int as u8, "a", None, Some(3)),
            (AttributeValueType::Int as u8, "a", None, None),
            (AttributeValueType::Int as u8, "b", None, Some(1)),
            (AttributeValueType::Int as u8, "b", None, Some(2)),
            (AttributeValueType::Int as u8, "b", None, Some(3)),
            (AttributeValueType::Int as u8, "b", None, None),
        ];
        let attr_types = UInt8Array::from_iter_values(expected_data.iter().map(|d| d.0));
        let attr_keys = StringArray::from_iter_values(expected_data.iter().map(|d| d.1));
        let attr_strs = StringArray::from_iter(expected_data.iter().map(|d| d.2));
        let attr_ints = Int64Array::from_iter(expected_data.iter().map(|d| d.3));

        let expected = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(attr_types),
                Arc::new(attr_keys),
                Arc::new(attr_strs),
                Arc::new(attr_ints),
            ],
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_single_column_with_nulls() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt8, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        let input = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![4, 1, 8, 3])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                        vec![Some(5u128), None, Some(1), Some(4)]
                            .into_iter()
                            .map(|i| i.map(|i| i.to_be_bytes())),
                        16,
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        // this should now be sorted by trace_id
        let result = sort_record_batch(&ArrowPayloadType::Logs, &input).unwrap();

        let expected = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![8, 3, 4, 1])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                        vec![Some(1u128), Some(4), Some(5), None]
                            .into_iter()
                            .map(|i| i.map(|i| i.to_be_bytes())),
                        16,
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_delta_encoded_column() {
        let test_cases = vec![
            (
                // simple case:
                vec![3, 1, 0, 2],
                vec![0, 1, 1, 1],
                Some(vec![2u16, 1, 3, 0]),
            ),
            (
                // simple case with no remapping .. in this case, we can delta encode the column
                // without remapping the IDs because it is already an incrementing sequence
                // starting at 0
                vec![0, 1, 2, 3],
                vec![0, 1, 1, 1],
                None,
            ),
            (
                // test batch of single length
                vec![0],
                vec![0],
                None,
            ),
            (
                // test batch of single length with non zero start
                vec![1],
                vec![0],
                Some(vec![0, 0]),
            ),
            (
                // test empty batch
                vec![],
                vec![],
                None,
            ),
        ];

        for test_case in test_cases {
            let input = UInt16Array::from_iter_values(test_case.0);
            let result = create_new_delta_encoded_column_from(&input).unwrap();
            let result_col = result
                .new_column
                .as_any()
                .downcast_ref::<UInt16Array>()
                .expect("Expected UInt16Array");
            let expected_column = UInt16Array::from_iter_values(test_case.1);
            assert_eq!(result_col, &expected_column);

            match test_case.2 {
                Some(remapping) => {
                    assert_eq!(
                        &result.remapping.unwrap(),
                        &RemappedParentIds::from(remapping.to_vec())
                    );
                }
                None => {
                    assert!(result.remapping.is_none())
                }
            }
        }
    }

    #[test]
    fn test_create_delta_encoded_column_all_nulls() {
        let input = UInt16Array::from_iter(vec![None, None, None, None]);
        let result = create_new_delta_encoded_column_from(&input).unwrap();
        let result_col = result
            .new_column
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("Expected UInt16Array");

        assert_eq!(result_col, &input);
        assert!(result.remapping.is_none());
    }

    #[test]
    fn test_apply_column_encodings_is_noop_if_all_columns_already_encoded() {
        let struct_fields: Fields = vec![Field::new(consts::ID, DataType::UInt16, true)].into();
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
            ])),
            vec![
                // id:
                Arc::new(UInt16Array::from_iter_values(vec![1])),
                // resource:
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1]))],
                    None,
                )),
                // scope:
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1]))],
                    None,
                )),
            ],
        )
        .unwrap();

        let result = apply_transport_optimized_encodings(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result.0, input);

        // assert no parent ID remappings returned
        assert_eq!(result.1, None)
    }

    #[test]
    fn test_apply_column_encodings_is_noop_if_all_columns_not_present() {
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::SCHEMA_URL,
                DataType::Utf8,
                false,
            )])),
            vec![Arc::new(StringArray::from_iter_values(vec!["a"]))],
        )
        .unwrap();

        let result = apply_transport_optimized_encodings(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result.0, input);

        // assert no parent ID remappings returned
        assert_eq!(result.1, None)
    }

    #[test]
    fn test_materializes_encoded_columns_before_sorting() {
        // create a record batch where the ID column is delta encoded, but the resource ID column is not.
        // In this case, the delta encoding on the ID column will need to be removed in order to sort
        let struct_fields: Fields =
            vec![Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding()].into();
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::DELTA),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ])),
            vec![
                // delta encoded IDs representing IDs 1, 3, 4, 6
                Arc::new(UInt16Array::from_iter_values(vec![1, 2, 1, 2])),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![1, 1, 0, 0]))],
                    None,
                )),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![6u128, 5, 4, 3].into_iter().map(|i| i.to_be_bytes()),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let result = apply_transport_optimized_encodings(&ArrowPayloadType::Logs, &input).unwrap();

        // Once the IDs are decoded and we sort, we should end up with:
        //
        // res_id  trace_id  id -> remapped id
        //  0       3        6  -> 0
        //  0       4        4  -> 1
        //  1       5        3  -> 2
        //  1       6        1  -> 3

        let expected_struct_fields: Fields = vec![
            Field::new(consts::ID, DataType::UInt16, true)
                .with_encoding(consts::metadata::encodings::DELTA),
        ]
        .into();
        let expected_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false)
                    .with_encoding(consts::metadata::encodings::DELTA),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(expected_struct_fields.clone()),
                    true,
                ),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ])),
            vec![
                // decoded IDs representing IDs 1, 3, 4, 6
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 1, 1])),
                Arc::new(StructArray::new(
                    expected_struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values(vec![0, 0, 1, 0]))],
                    None,
                )),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![3u128, 4, 5, 6].into_iter().map(|i| i.to_be_bytes()),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        assert_eq!(result.0, expected_batch);

        let expected_remappings = vec![ParentIdRemapping {
            column_path: consts::ID,
            // TODO consider if these 0s are correct ... this should probably be either a vec of Options or an array with null buffer ...
            remapped_ids: RemappedParentIds::UInt16(vec![0, 3, 0, 2, 1, 0, 0]),
        }];

        assert_eq!(result.1.unwrap(), expected_remappings);
    }

    #[test]
    fn test_remap_parent_ids_plain_encoded() {
        fn do_test_generic<T: ArrowPrimitiveType>(payload_type: ArrowPayloadType)
        where
            Vec<<T as ArrowPrimitiveType>::Native>: Into<RemappedParentIds>,
            <T as ArrowPrimitiveType>::Native: From<u8>,
        {
            let schema = Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, T::DATA_TYPE, false).with_plain_encoding(),
            ]));

            let record_batch = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(PrimitiveArray::<T>::from_iter_values(
                    vec![5, 0, 3, 1, 2, 4].into_iter().map(T::Native::from),
                ))],
            )
            .unwrap();

            // represents remapped IDs like
            // original -> new
            // 0 -> 4,
            // 1 -> 2,
            // 2 -> 1,
            // 3 -> 0,
            // 4 -> 5,
            // 5 -> 3
            let remapping: RemappedParentIds = vec![4, 2, 1, 0, 5, 3]
                .into_iter()
                .map(T::Native::from)
                .collect::<Vec<_>>()
                .into();

            let result = remap_parent_ids(&payload_type, &record_batch, &remapping).unwrap();

            let expected = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(PrimitiveArray::<T>::from_iter_values(
                    vec![3, 4, 0, 2, 1, 5].into_iter().map(T::Native::from),
                ))],
            )
            .unwrap();

            assert_eq!(result, expected);
        }

        do_test_generic::<UInt16Type>(ArrowPayloadType::LogAttrs);
        do_test_generic::<UInt32Type>(ArrowPayloadType::SpanLinkAttrs);
    }

    #[test]
    fn test_remap_parent_ids_with_nulls_plain_encoded() {
        fn do_test_generic<T: ArrowPrimitiveType>(payload_type: ArrowPayloadType)
        where
            Vec<<T as ArrowPrimitiveType>::Native>: Into<RemappedParentIds>,
            <T as ArrowPrimitiveType>::Native: From<u8>,
        {
            let schema = Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, T::DATA_TYPE, true).with_plain_encoding(),
            ]));

            let record_batch = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(PrimitiveArray::<T>::from_iter(
                    vec![Some(3), None, Some(2), Some(0), Some(1), None]
                        .into_iter()
                        .map(|v| v.map(T::Native::from)),
                ))],
            )
            .unwrap();

            // represents remapped IDs like
            // original -> new
            // 0 -> 3,
            // 1 -> 2,
            // 2 -> 1,
            // 3 -> 0,
            let remapping: RemappedParentIds = vec![3, 2, 1, 0]
                .into_iter()
                .map(T::Native::from)
                .collect::<Vec<_>>()
                .into();

            let result = remap_parent_ids(&payload_type, &record_batch, &remapping).unwrap();

            let expected = RecordBatch::try_new(
                schema.clone(),
                vec![Arc::new(PrimitiveArray::<T>::from_iter(
                    vec![Some(0), None, Some(1), Some(3), Some(2), None]
                        .into_iter()
                        .map(|v| v.map(T::Native::from)),
                ))],
            )
            .unwrap();

            assert_eq!(result, expected);
        }

        do_test_generic::<UInt16Type>(ArrowPayloadType::LogAttrs);
        do_test_generic::<UInt32Type>(ArrowPayloadType::SpanLinkAttrs);
    }

    #[test]
    fn test_remap_parent_ids_with_decode_attributes() {
        // in order to remap the parent IDs, if the column is delta encoded then we need to turn it
        // back into a plain encoded column. This test ensures we do that properly

        fn do_test_generic<T: ArrowPrimitiveType>(payload_type: ArrowPayloadType)
        where
            Vec<<T as ArrowPrimitiveType>::Native>: Into<RemappedParentIds>,
            <T as ArrowPrimitiveType>::Native: From<u8>,
        {
            let schema = Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, T::DATA_TYPE, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ]));

            // this delta encoded column represents
            // 0, 1, 2, 3, 4, 5
            let parent_ids = PrimitiveArray::<T>::from_iter_values(
                [0, 1, 1, 1, 1, 1].into_iter().map(T::Native::from),
            );
            let attr_types =
                UInt8Array::from_iter_values(std::iter::repeat_n(AttributeValueType::Str as u8, 6));
            let attr_keys = StringArray::from_iter_values(std::iter::repeat_n("a", 6));
            let attr_vals = StringArray::from_iter_values(std::iter::repeat_n("a", 6));
            let record_batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(parent_ids.clone()),
                    Arc::new(attr_types.clone()),
                    Arc::new(attr_keys.clone()),
                    Arc::new(attr_vals.clone()),
                ],
            )
            .unwrap();

            // represents remapped IDs like
            // original -> new
            // 0 -> 4,
            // 1 -> 2,
            // 2 -> 1,
            // 3 -> 0,
            // 4 -> 5,
            // 5 -> 3
            let remapping: RemappedParentIds = vec![4, 2, 1, 0, 5, 3]
                .into_iter()
                .map(T::Native::from)
                .collect::<Vec<_>>()
                .into();
            let result = remap_parent_ids(&payload_type, &record_batch, &remapping).unwrap();

            let expected_schema = Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, T::DATA_TYPE, false).with_plain_encoding(),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ]));
            let expected_parent_ids = PrimitiveArray::<T>::from_iter_values(
                [4, 2, 1, 0, 5, 3].into_iter().map(T::Native::from),
            );
            let expected = RecordBatch::try_new(
                expected_schema,
                vec![
                    Arc::new(expected_parent_ids),
                    Arc::new(attr_types.clone()),
                    Arc::new(attr_keys.clone()),
                    Arc::new(attr_vals.clone()),
                ],
            )
            .unwrap();

            assert_eq!(result, expected)
        }

        // check for every attribute type to make sure we've covered them all with the correct
        // decoding function
        let u16_test_cases = [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::SpanAttrs,
        ];
        for payload_type in u16_test_cases {
            do_test_generic::<UInt16Type>(payload_type);
        }

        let u32_test_cases = [
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ];
        for payload_type in u32_test_cases {
            do_test_generic::<UInt32Type>(payload_type);
        }
    }

    #[test]
    fn test_remap_parent_ids_with_decode_span_events() {
        // this is the name & quasi-delta encoded parent ID
        let data = [
            ("a", 0), // id = 0
            ("a", 1), // id = 1
            ("a", 1), // id = 2
            ("b", 0), // id = 0
            ("b", 1), // id = 1
            ("b", 2), // id = 3
        ];

        let names = StringArray::from_iter_values(data.iter().map(|d| d.0));
        let parent_ids = UInt16Array::from_iter_values(data.iter().map(|d| d.1));
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, true),
                Field::new(consts::NAME, DataType::Utf8, true),
            ])),
            vec![Arc::new(parent_ids), Arc::new(names)],
        )
        .unwrap();

        // remappings:
        // 0 -> 1
        // 1 -> 0
        // 2 -> 3
        // 3 -> 2
        let remapings = RemappedParentIds::from(vec![1u16, 0, 3, 2]);
        let result = remap_parent_ids(&ArrowPayloadType::SpanEvents, &input, &remapings).unwrap();

        let expected_data = [("a", 1), ("a", 0), ("a", 3), ("b", 1), ("b", 0), ("b", 2)];

        let names = StringArray::from_iter_values(expected_data.iter().map(|d| d.0));
        let parent_ids = UInt16Array::from_iter_values(expected_data.iter().map(|d| d.1));
        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::NAME, DataType::Utf8, true),
            ])),
            vec![Arc::new(parent_ids), Arc::new(names)],
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_remap_parent_ids_with_decode_span_links() {
        // this is the name & quasi-delta encoded parent ID
        let data = [
            (0, 0), // id = 0
            (0, 1), // id = 1
            (0, 1), // id = 2
            (1, 0), // id = 0
            (1, 1), // id = 1
            (1, 2), // id = 3
        ];

        let trace_ids =
            FixedSizeBinaryArray::try_from_iter(data.iter().map(|d| u128::to_be_bytes(d.0)))
                .unwrap();
        let parent_ids = UInt16Array::from_iter_values(data.iter().map(|d| d.1));
        let input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, true),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ])),
            vec![Arc::new(parent_ids), Arc::new(trace_ids)],
        )
        .unwrap();

        // remappings:
        // 0 -> 1
        // 1 -> 0
        // 2 -> 3
        // 3 -> 2
        let remapings = RemappedParentIds::from(vec![1u16, 0, 3, 2]);
        let result = remap_parent_ids(&ArrowPayloadType::SpanLinks, &input, &remapings).unwrap();

        let expected_data = [(0, 1), (0, 0), (0, 3), (1, 1), (1, 0), (1, 2)];

        let trace_ids = FixedSizeBinaryArray::try_from_iter(
            expected_data.iter().map(|d| u128::to_be_bytes(d.0)),
        )
        .unwrap();
        let parent_ids = UInt16Array::from_iter_values(expected_data.iter().map(|d| d.1));
        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ])),
            vec![Arc::new(parent_ids), Arc::new(trace_ids)],
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_remap_parent_ids_with_decode_exemplars() {
        fn do_test(payload_type: ArrowPayloadType) {
            let data = vec![
                (None, None, 0),      // id = 0
                (None, None, 1),      // id = 1
                (None, None, 1),      // id = 2
                (Some(1), None, 0),   // id = 0
                (Some(1), None, 2),   // id = 2
                (Some(1), None, 1),   // id = 3
                (Some(2), None, 0),   // id = 0
                (None, Some(1.0), 1), // id = 1
                (None, Some(1.0), 1), // id = 2
                (None, Some(2.0), 0), // id = 0
            ];

            let int_values = Int64Array::from_iter(data.iter().map(|d| d.0));
            let double_values = Float64Array::from_iter(data.iter().map(|d| d.1));
            let parent_ids = UInt32Array::from_iter_values(data.iter().map(|d| d.2));

            let input = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt32, true),
                    Field::new(consts::INT_VALUE, DataType::Int64, true),
                    Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
                ])),
                vec![
                    Arc::new(parent_ids),
                    Arc::new(int_values),
                    Arc::new(double_values),
                ],
            )
            .unwrap();

            // remappings:
            // 0 -> 1
            // 1 -> 0
            // 2 -> 3
            // 3 -> 2
            let remapings = RemappedParentIds::from(vec![1u32, 0, 3, 2]);
            let result = remap_parent_ids(&payload_type, &input, &remapings).unwrap();

            let expected_data = vec![
                (None, None, 1),
                (None, None, 0),
                (None, None, 3),
                (Some(1), None, 1),
                (Some(1), None, 3),
                (Some(1), None, 2),
                (Some(2), None, 1),
                (None, Some(1.0), 0),
                (None, Some(1.0), 3),
                (None, Some(2.0), 1),
            ];
            let expected = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new(consts::PARENT_ID, DataType::UInt32, true).with_plain_encoding(),
                    Field::new(consts::INT_VALUE, DataType::Int64, true),
                    Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
                ])),
                vec![
                    Arc::new(UInt32Array::from_iter_values(
                        expected_data.iter().map(|d| d.2),
                    )),
                    Arc::new(Int64Array::from_iter(expected_data.iter().map(|d| d.0))),
                    Arc::new(Float64Array::from_iter(expected_data.iter().map(|d| d.1))),
                ],
            )
            .unwrap();

            assert_eq!(result, expected);
        }

        for payload_type in [
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
        ] {
            do_test(payload_type);
        }
    }

    #[test]
    fn test_transport_encode_parent_ids_for_attributes_empty_batch() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            consts::PARENT_ID,
            DataType::UInt16,
            true,
        )]));
        let input = RecordBatch::new_empty(schema);

        let result = transport_encode_parent_id_for_attributes::<UInt16Type>(&input).unwrap();
        assert_eq!(&result, input.column_by_name(consts::PARENT_ID).unwrap());
    }

    #[test]
    fn test_transport_encode_parent_ids_for_attributes_no_double_encoding() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, true)
                .with_encoding(consts::metadata::encodings::QUASI_DELTA),
        ]));
        let input = RecordBatch::try_new(
            schema,
            vec![Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3]))],
        )
        .unwrap();

        let result = transport_encode_parent_id_for_attributes::<UInt16Type>(&input).unwrap();
        assert_eq!(&result, input.column_by_name(consts::PARENT_ID).unwrap());
    }

    #[test]
    fn test_transport_encode_parent_id_for_columns_empty_batch() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            consts::PARENT_ID,
            DataType::UInt16,
            true,
        )]));
        let input = RecordBatch::new_empty(schema);

        let result =
            transport_encode_parent_id_for_columns::<UInt16Type>(&input, &["name"]).unwrap();
        assert_eq!(&result, input.column_by_name(consts::PARENT_ID).unwrap());
    }

    #[test]
    fn test_transport_encode_parent_id_for_columns_no_double_encoding() {
        let schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, DataType::UInt16, true)
                .with_encoding(consts::metadata::encodings::QUASI_DELTA),
        ]));
        let input = RecordBatch::try_new(
            schema,
            vec![Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3]))],
        )
        .unwrap();

        let result =
            transport_encode_parent_id_for_columns::<UInt16Type>(&input, &["name"]).unwrap();
        assert_eq!(&result, input.column_by_name(consts::PARENT_ID).unwrap());
    }
}
