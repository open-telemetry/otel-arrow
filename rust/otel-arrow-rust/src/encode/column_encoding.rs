// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for adding transport optimized encodings to the various columns in
//! OTAP record batches when converting to BatchArrowRecords. These types of encodings include
//! delta encoding for ID columns, and quasi-delta encoding for attribute parent IDs.
//!
//! The motivation behind adding these encodings to the columns is for better compression when,
//! for example, transmitting OTAP data via gRPC.

use std::{
    ops::{Add, AddAssign},
    sync::Arc,
};

use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, PrimitiveArray, RecordBatch, StructArray, UInt16Array,
        UInt32Array,
    },
    buffer::{MutableBuffer, ScalarBuffer},
    compute::{SortOptions, sort_to_indices, take_record_batch},
    datatypes::{ArrowNativeType, DataType, FieldRef, Schema, UInt16Type, UInt32Type},
    row::{RowConverter, SortField},
};
use snafu::OptionExt;

use crate::{
    error::{self, Result},
    otap::transform::{
        materialize_parent_id_for_attributes, materialize_parent_id_for_exemplars,
        materialize_parent_ids_by_columns, remove_delta_encoding,
        transport_encode_parent_id_for_attributes,
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::{FieldExt, consts},
};

/// identifier for column encoding
#[derive(Clone, Copy)]
enum Encoding {
    Delta,

    /// this is the transport optimized encoding that is applied to attribute's parent_id column
    /// where subsequent rows of matching type/key/value will have their parent_id field delta
    /// encoded
    AttributeQuasiDelta,
}

// for the majority of columns, we'll be able to identify the path within the record batch as
// the column name directly, but Resource ID and Scope ID, they're typically nested within a
// struct on the root record so we treat these as special cases.
/// TODO
pub const RESOURCE_ID_COL_PATH: &str = "resource.id";
/// TODO
pub const SCOPE_ID_COL_PATH: &str = "scope.id";

/// specification for encoding that should be applied to the column before it is IPC serialized
struct ColumnEncoding<'a> {
    /// path to the column within the record batch
    path: &'a str,

    /// the expected data type of the column
    data_type: DataType,

    /// identifier for how the column should be encoded
    encoding: Encoding,
}

impl<'a> ColumnEncoding<'a> {
    /// access the column associated with this [`ColumnEncoding`]
    // TODO - possibly refactor? This might not need to be a member of this struct
    fn access_column(&self, schema: &Schema, columns: &[ArrayRef]) -> Option<ArrayRef> {
        access_column(self.path, schema, columns)
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
    fn is_column_encoded(&self, schema: &Schema) -> Option<bool> {
        let field = if let Some(struct_col_name) = struct_column_name(self.path) {
            // get the ID field out of the struct column
            let struct_col = schema.field_with_name(struct_col_name).ok()?;
            if let DataType::Struct(fields) = struct_col.data_type() {
                fields.find(consts::ID).map(|(_, field)| field)?
            } else {
                return None;
            }
        } else {
            // otherwise just look at field with path == name
            schema.field_with_name(self.path).ok()?
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
}

/// access the column associated for the possibly nested path
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

    // otherwise just return column by nname
    let column_idx = schema.index_of(path).ok()?;
    columns.get(column_idx).cloned()
}

fn replace_column(
    path: &str,
    encoding: Encoding,
    schema: &Schema,
    columns: &mut [ArrayRef],
    new_column: ArrayRef,
) {
    // TODO write tests for this
    if let Some(struct_col_name) = struct_column_name(path) {
        let field_index = schema.index_of(struct_col_name).ok();
        if let Some(field_index) = field_index {
            let struct_column = columns[field_index].as_any().downcast_ref::<StructArray>();
            if let Some(struct_column) = struct_column {
                if let Some((struct_idx, _)) = struct_column.fields().find(consts::ID) {
                    // replace the encoding metadata on the struct field
                    let mut new_struct_fields = struct_column.fields().to_vec();
                    update_field_encoding_metadata(&mut new_struct_fields, consts::ID, encoding);

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

/// sets the encoding metadata on the field metadata for column at path
fn update_field_encoding_metadata(fields: &mut [FieldRef], path: &str, encoding: Encoding) {
    // TODO write tests for this

    if let Some(struct_col_name) = struct_column_name(path) {
        // replace the field metadata in some nested struct
        let found_field = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name().as_str() == struct_col_name);

        if let Some((idx, field)) = found_field {
            if let DataType::Struct(struct_fields) = field.data_type() {
                let mut new_struct_fields = struct_fields.to_vec();
                update_field_encoding_metadata(&mut new_struct_fields, consts::ID, encoding);

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
            Encoding::Delta => consts::metadata::encodings::DELTA,
            Encoding::AttributeQuasiDelta => consts::metadata::encodings::QUASI_DELTA,
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

/// helper for initializing [`ColumnEncoding`] as it will be initialized many times in the function
/// below, so this helps w/ brevity
macro_rules! col_encoding {
    ($path:expr, $dtype:ident, $enc:ident) => {
        ColumnEncoding {
            path: $path,
            data_type: DataType::$dtype,
            encoding: Encoding::$enc,
        }
    };
}

/// returns the list of transport-optimized encoding that should be applied to OTAP batches of a
/// given payload type
fn get_column_encodings(payload_type: &ArrowPayloadType) -> &'static [ColumnEncoding<'static>] {
    match payload_type {
        ArrowPayloadType::ResourceAttrs => &[col_encoding!(
            consts::PARENT_ID,
            UInt16,
            AttributeQuasiDelta
        )],
        ArrowPayloadType::ScopeAttrs => &[col_encoding!(
            consts::PARENT_ID,
            UInt16,
            AttributeQuasiDelta
        )],
        ArrowPayloadType::LogAttrs => &[col_encoding!(
            consts::PARENT_ID,
            UInt16,
            AttributeQuasiDelta
        )],
        // ArrowPayloadType::SpanAttrs => &[col_encoding!(
        //     consts::PARENT_ID,
        //     UInt16,
        //     AttributeQuasiDelta
        // )],
        // ArrowPayloadType::MetricAttrs => &[col_encoding!(
        //     consts::PARENT_ID,
        //     UInt16,
        //     AttributeQuasiDelta
        // )],
        ArrowPayloadType::Logs => &[
            col_encoding!(consts::ID, UInt16, Delta),
            col_encoding!(RESOURCE_ID_COL_PATH, UInt16, Delta),
            col_encoding!(SCOPE_ID_COL_PATH, UInt16, Delta),
        ],
        _ => &[],
    }
}

/// returns the list of columns that the OTAP record batch of this payload type should be sorted by
/// before applying any column encodings
fn get_sort_column_paths(payload_type: &ArrowPayloadType) -> &'static [&'static str] {
    match payload_type {
        // TODO fill in this for all the other types

        // TODO need to check that this gives the correct sort order ..
        ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs
        | ArrowPayloadType::LogAttrs => &[
            consts::ATTRIBUTE_TYPE,
            consts::ATTRIBUTE_KEY,
            consts::ATTRIBUTE_STR,
            consts::ATTRIBUTE_INT,
            consts::ATTRIBUTE_DOUBLE,
            consts::ATTRIBUTE_BOOL,
            consts::ATTRIBUTE_BYTES,
            consts::PARENT_ID,
        ],
        ArrowPayloadType::Logs => &[RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH, consts::TRACE_ID],
        _ => &[],
    }
}

/// sort the record batch with this payload type by columns that will hopefully give us the best
/// compression ratio
// TODO bench this ...
fn sort_record_batch(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    let sort_columns_paths = get_sort_column_paths(payload_type);
    // TODO, here maybe we should check if this is empty?
    // ^^ this would only ever be empty if there were no sort columns configured for some payload type

    let schema = record_batch.schema_ref();
    let mut sort_inputs = vec![];
    let columns = record_batch.columns();
    for path in sort_columns_paths {
        if let Some(column) = access_column(path, &schema, columns) {
            sort_inputs.push(column)
        }
    }

    if sort_inputs.len() == 0 {
        Ok(record_batch.clone())
    } else if sort_inputs.len() == 1 {
        let indices = sort_to_indices(
            &sort_inputs[0],
            Some(SortOptions {
                descending: false,
                nulls_first: false,
            }),
            None,
        )
        .unwrap();
        Ok(take_record_batch(record_batch, &indices).unwrap())
    } else {
        // multi-column sort .. using row converter to convert the rows.
        //
        // For multi-column sort, it is faster to use the row model for sorting and compare the
        // row bytes directly instead of having to fetch & compare the values from multiple columns
        // See here for more discussion:
        // https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-1/

        let sort_options = SortOptions {
            nulls_first: false,
            descending: false,
        };
        let sort_fields = sort_inputs
            .iter()
            .map(|col| SortField::new_with_options(col.data_type().clone(), sort_options.clone()))
            .collect();
        let converter = RowConverter::new(sort_fields).unwrap();
        let rows = converter.convert_columns(&sort_inputs).unwrap();

        let mut sort: Vec<_> = rows.iter().enumerate().collect();
        sort.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
        let indices = UInt32Array::from_iter_values(sort.iter().map(|(i, _)| *i as u32));
        Ok(take_record_batch(record_batch, &indices).unwrap())
    }
}

struct EncodedColumnResult<T: ArrowPrimitiveType> {
    new_column: ArrayRef,
    remapping: Option<Vec<T::Native>>,
}

/// TODO
fn create_new_delta_encoded_column_from<T>(
    column: &PrimitiveArray<T>,
) -> Result<EncodedColumnResult<T>>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: From<u8> + Add<Output = T::Native> + AddAssign + PartialOrd,
{
    // TODO need to check if column has at least one value
    // TODO we can do nothing if the column contains only one value?
    // TODO use a bunch of unsafe stuff here to build the new column

    // TODO - there's an important optimization that can be made here where, if all the IDs
    // from the original array equal their counterpart in the remappings, we can avoid returning
    // a new column altogether, which will mean we save time not having to remap the Parent IDs of
    // the child record batch

    let zero = T::Native::from(0u8);
    let one = T::Native::from(1u8);

    let remappings_len = one + arrow::compute::max(column).unwrap();
    let mut remappings = vec![zero; remappings_len.as_usize()];

    let mut curr_id: T::Native = zero;

    let mut new_buffer = MutableBuffer::with_capacity(column.len() * size_of::<T::Native>());

    let mut prev_value = None;
    for val in column.iter() {
        match val {
            Some(val) => match prev_value {
                Some(prev) => {
                    if val == prev {
                        new_buffer.push(zero);
                    } else {
                        new_buffer.push(one);

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
                new_buffer.push(zero);
            }
        }
    }

    let nulls = column.nulls().cloned();
    let new_buffer = ScalarBuffer::<T::Native>::new(new_buffer.into(), 0, column.len());
    let new_column = PrimitiveArray::<T>::new(new_buffer, nulls);

    Ok(EncodedColumnResult {
        new_column: Arc::new(new_column),
        remapping: Some(remappings),
    })
}

/// context required to remap parent IDs. After an ID column has been encoded, the IDs may change
/// so the parent IDs that pointed to these IDs also need to be renapped
/// TODO can you write better commnents on this?
#[derive(Debug, PartialEq)]
pub struct ParentIdRemapping {
    /// TODO commentaires
    pub column_path: &'static str,

    /// TODO commentaires
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

/// TODO comments
/// should explain what this field represents e.g. a vec where indices are the old
/// ids and values are the new ones
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum RemappedParentIds {
    UInt16(Vec<u16>),
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
    remapped_ids: &Vec<T::Native>,
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
pub fn apply_column_encodings(
    payload_type: &ArrowPayloadType,
    record_batch: &RecordBatch,
) -> Result<(RecordBatch, Option<Vec<ParentIdRemapping>>)> {
    let column_encodings = get_column_encodings(payload_type);

    // TODO revisit this check after filling in get_column_encodings b/c there might not be any
    // payload types that have no encoded columns in practice, in which case this check is useless
    // as it'll never be true.
    if column_encodings.is_empty() {
        // nothing to do
        return Ok((record_batch.clone(), None));
    }

    let schema = record_batch.schema();

    // determine which columns need to be encoded
    let mut to_apply = vec![];
    for column_encoding in column_encodings {
        let is_encoded = column_encoding.is_column_encoded(&schema);
        if is_encoded == Some(false) {
            to_apply.push(column_encoding)
        }
    }

    if to_apply.len() == 0 {
        // nothing to do
        return Ok((record_batch.clone(), None));
    }

    if to_apply.len() != column_encodings.len() {
        // TODO handle this ...
        // this would be a weird situation where only some of the columns are encoded and others aren't.
        // if we need to sort the record batch to do the encoding, this will not end well ....
    }

    // sort record batch before applying the encoding. This will give us the best compression ratio
    // for columns which may have many repeated sequences of the same value
    let record_batch = sort_record_batch(payload_type, record_batch)?;
    let mut columns = record_batch.columns().to_vec();
    let mut fields = schema.fields.to_vec();
    let mut remapped_parent_ids = Vec::with_capacity(column_encodings.len());

    for column_encoding in to_apply {
        let column = column_encoding.access_column(&schema, &columns).unwrap();

        let encoding_result = match &column_encoding.data_type {
            DataType::UInt16 => {
                let column =
                    column
                        .as_any()
                        .downcast_ref()
                        .context(error::InvalidListArraySnafu {
                            expect_oneof: vec![DataType::UInt16],
                            actual: column.data_type().clone(),
                        })?;
                match column_encoding.encoding {
                    Encoding::Delta => create_new_delta_encoded_column_from::<UInt16Type>(column),
                    Encoding::AttributeQuasiDelta => {
                        let new_col =
                            transport_encode_parent_id_for_attributes::<u16>(&record_batch)?;
                        Ok(EncodedColumnResult {
                            new_column: new_col,
                            remapping: None,
                        })
                    }
                }
            }
            _ => {
                todo!()
            }
        }?;
        if let Some(remapping) = encoding_result.remapping {
            remapped_parent_ids.push(ParentIdRemapping::new(
                column_encoding.path,
                remapping.into(),
            ));
        }
        // TODO the order of arguments between this and the next method call are not consistent
        replace_column(
            column_encoding.path,
            column_encoding.encoding,
            &schema,
            &mut columns,
            encoding_result.new_column,
        );
        // TODO could make this a method call on column_encoding?
        update_field_encoding_metadata(&mut fields, column_encoding.path, column_encoding.encoding);
    }

    // Safety: TODO
    let record_batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("TODO why we can expect here");

    // TODO need to write tests for this
    Ok((record_batch, Some(remapped_parent_ids)))
}

#[cfg(test)]
mod test {
    use std::vec;

    use arrow::{
        array::{
            FixedSizeBinaryArray, Int64Array, StringArray, StructArray, TimestampNanosecondArray,
            UInt8Array, UInt16Array,
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
            encoding: Encoding::Delta,
        };

        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
        assert_eq!(*column, *columns[0]);

        // assert what happens if the column isn't present
        column_encoding.path = "b";
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        )
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
            encoding: Encoding::Delta,
        };

        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
        assert_eq!(*column, resource_ids);

        column_encoding.path = SCOPE_ID_COL_PATH;
        let column = column_encoding
            .access_column(&schema, columns.as_ref())
            .unwrap();
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
            encoding: Encoding::Delta,
        };

        // assert what happens if the struct isn't present
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        );

        // assert what happens if the struct is present, but the ID field isn't present
        column_encoding.path = SCOPE_ID_COL_PATH;
        assert!(
            column_encoding
                .access_column(&schema, columns.as_ref())
                .is_none()
        );
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
            encoding: Encoding::Delta,
        };

        assert!(!column_encoding.is_column_encoded(&schema).unwrap());

        // ensure that no metadata means column is encoded
        column_encoding.path = "b";
        assert!(column_encoding.is_column_encoded(&schema).unwrap());
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

        let mut column_encoding = ColumnEncoding {
            path: RESOURCE_ID_COL_PATH,
            data_type: DataType::UInt16,
            encoding: Encoding::Delta,
        };

        assert!(!column_encoding.is_column_encoded(&schema).unwrap());
        column_encoding.path = SCOPE_ID_COL_PATH;
        assert!(column_encoding.is_column_encoded(&schema).unwrap());
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

            // TODO -- test sorting w/ nulls

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
                [0, 1, 2, 3, 4, 5, 6, 7],
                [0, 1, 1, 1, 1, 1, 1, 1],
                [0, 1, 2, 3, 4, 5, 6, 7],
            ), // TODO more test cases
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
            assert_eq!(&result.remapping.unwrap(), &test_case.2);
        }
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

        let result = apply_column_encodings(&ArrowPayloadType::Logs, &input).unwrap();
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

        let result = apply_column_encodings(&ArrowPayloadType::Logs, &input).unwrap();
        assert_eq!(result.0, input);

        // assert no parent ID remappings returned
        assert_eq!(result.1, None)
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
                        .map(|v| match v {
                            Some(v) => Some(T::Native::from(v)),
                            None => None,
                        }),
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
                        .map(|v| match v {
                            Some(v) => Some(T::Native::from(v)),
                            None => None,
                        }),
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
}
