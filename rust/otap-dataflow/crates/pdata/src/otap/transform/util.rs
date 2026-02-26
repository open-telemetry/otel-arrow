// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, DictionaryArray, MutableArrayData, PrimitiveArray, RecordBatch, StructArray,
    UInt32Array, make_array,
};
use arrow::compute::SortColumn;
use arrow_schema::{DataType, FieldRef, Schema, SortOptions};

use crate::error::{Error, Result};
use crate::otap::{POSITION_LOOKUP, UNUSED_INDEX};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{ID, PARENT_ID};
use crate::schema::{FieldExt, consts};

use super::transport_optimize::{Encoding, RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH};

/// Dispatches on the `DataType` of an Arrow column, binding type parameters for
/// the concrete Arrow numeric types supported by ID columns.
///
/// # Expression Duplication
/// `$native_expr` is emitted once per native type arm (UInt16, UInt32), and
/// `$dict_expr` once per dictionary key/value combination. `$fallback` is emitted
/// for both the native and dictionary fallthrough arms.
macro_rules! id_column_dispatch {
    (
        $col:expr,
        Native[$T:ident] => $native_expr:expr,
        Dictionary[$KType:ident, $VType:ident] => $dict_expr:expr,
        _ => $fallback:expr $(,)?
    ) => {
        match $col.data_type() {
            ::arrow::datatypes::DataType::UInt16 => {
                type $T = ::arrow::datatypes::UInt16Type;
                $native_expr
            }
            ::arrow::datatypes::DataType::UInt32 => {
                type $T = ::arrow::datatypes::UInt32Type;
                $native_expr
            }
            ::arrow::datatypes::DataType::Dictionary(key_dt, val_dt) => {
                match (key_dt.as_ref(), val_dt.as_ref()) {
                    (::arrow::datatypes::DataType::UInt8, ::arrow::datatypes::DataType::UInt16) => {
                        type $KType = ::arrow::datatypes::UInt8Type;
                        type $VType = ::arrow::datatypes::UInt16Type;
                        $dict_expr
                    }
                    (::arrow::datatypes::DataType::UInt8, ::arrow::datatypes::DataType::UInt32) => {
                        type $KType = ::arrow::datatypes::UInt8Type;
                        type $VType = ::arrow::datatypes::UInt32Type;
                        $dict_expr
                    }
                    (
                        ::arrow::datatypes::DataType::UInt16,
                        ::arrow::datatypes::DataType::UInt32,
                    ) => {
                        type $KType = ::arrow::datatypes::UInt16Type;
                        type $VType = ::arrow::datatypes::UInt32Type;
                        $dict_expr
                    }
                    _ => $fallback,
                }
            }
            _ => $fallback,
        }
    };
}
pub(crate) use id_column_dispatch;

/// Create a new record batch by taking the specified ranges from the provided record batch.
pub(crate) fn take_record_batch_ranges(
    rb: &RecordBatch,
    ranges: &[Range<usize>],
) -> arrow::error::Result<RecordBatch> {
    let new_len = ranges.iter().map(|r| r.end - r.start).sum();
    let mut new_columns = Vec::with_capacity(rb.num_columns());
    for column in rb.columns() {
        let data = column.to_data();
        let mut new_data = MutableArrayData::new(vec![&data], false, new_len);
        for range in ranges {
            new_data.extend(0, range.start, range.end);
        }
        new_columns.push(make_array(new_data.freeze()));
    }

    RecordBatch::try_new(rb.schema(), new_columns)
}

/// Create a new record batch by removing the specified ranges from the provided record batch.
pub(crate) fn remove_record_batch_ranges(
    rb: &RecordBatch,
    ranges: &[Range<usize>],
) -> arrow::error::Result<RecordBatch> {
    let new_len = rb.num_rows() - ranges.iter().map(|r| r.end - r.start).sum::<usize>();
    let mut new_columns = Vec::with_capacity(rb.num_columns());
    for column in rb.columns() {
        let data = column.to_data();
        let mut new_data = MutableArrayData::new(vec![&data], false, new_len);
        let mut pos = 0;
        for range in ranges {
            new_data.extend(0, pos, range.start);
            pos = range.end;
        }
        new_data.extend(0, pos, rb.num_rows());
        new_columns.push(make_array(new_data.freeze()));
    }

    RecordBatch::try_new(rb.schema(), new_columns)
}

pub(crate) fn sort_otap_batch_by_parent_then_id<const N: usize>(
    batches: &mut [Option<RecordBatch>; N],
) -> Result<()> {
    for batch in batches.iter_mut() {
        if let Some(rb) = batch.take() {
            *batch = Some(sort_by_parent_then_id(rb)?);
        }
    }

    Ok(())
}

/// Sorts a record batch by `parent_id` (primary) then `id` (secondary) columns.
///
/// Dispatches to specialized sort paths based on the column types:
/// - No sort columns: returns batch unchanged.
/// - Single column (native): sort via primitive values slice with `is_sorted` early return.
/// - Single column (dictionary): resolve through dict keys→values, `is_sorted` early return.
/// - Two columns (both native): RowConverter-based sort with `is_sorted` early return.
/// - Two columns (dict parent_id + native id): pack resolved parent_id and id into a `Vec<u64>`,
///   sort that, with `is_sorted` early return. Avoids the RowConverter dictionary expansion.
#[cfg(feature = "bench")]
pub fn sort_by_parent_then_id(rb: RecordBatch) -> Result<RecordBatch> {
    sort_by_parent_then_id_impl(rb)
}

#[cfg(not(feature = "bench"))]
pub(crate) fn sort_by_parent_then_id(rb: RecordBatch) -> Result<RecordBatch> {
    sort_by_parent_then_id_impl(rb)
}

fn sort_by_parent_then_id_impl(rb: RecordBatch) -> Result<RecordBatch> {
    let schema = rb.schema();

    let parent_id_col = schema
        .column_with_name(PARENT_ID)
        .map(|(idx, _)| rb.column(idx).clone());
    let id_col = schema
        .column_with_name(ID)
        .map(|(idx, _)| rb.column(idx).clone());

    match (&parent_id_col, &id_col) {
        (None, None) => Ok(rb),
        (None, Some(col)) | (Some(col), None) => sort_single_column(rb, col),
        (Some(pid), Some(id)) => sort_two_columns(rb, pid, id),
    }
}

/// Single-column sort. Sorts native/dict columns directly via `sort_unstable_by_key`
/// on the primitive values, avoiding Arrow's `sort_to_indices` kernel overhead.
fn sort_single_column(rb: RecordBatch, col: &ArrayRef) -> Result<RecordBatch> {
    let len = rb.num_rows();

    id_column_dispatch!(
        col,
        Native[T] => {
            let vals = col.as_any().downcast_ref::<PrimitiveArray<T>>().unwrap().values();
            if vals.is_sorted() {
                return Ok(rb);
            }
            let mut indices: Vec<u32> = (0..len as u32).collect();
            indices.sort_unstable_by_key(|&i| vals[i as usize]);
            let indices_arr = UInt32Array::from(indices);
            return sort_record_batch_by_indices(rb, &indices_arr);
        },
        Dictionary[KType, VType] => {
            let dict = col.as_any().downcast_ref::<DictionaryArray<KType>>().unwrap();
            let keys = dict.keys().values();
            let vals = dict.values().as_any()
                .downcast_ref::<PrimitiveArray<VType>>().unwrap().values();

            let resolved_sorted = keys.windows(2).all(|w| {
                vals[w[0] as usize] <= vals[w[1] as usize]
            });
            if resolved_sorted {
                return Ok(rb);
            }

            let mut indices: Vec<u32> = (0..len as u32).collect();
            indices.sort_unstable_by_key(|&i| vals[keys[i as usize] as usize]);
            let indices_arr = UInt32Array::from(indices);
            return sort_record_batch_by_indices(rb, &indices_arr);
        },
        _ => {}
    );

    // Fallback for unsupported column types
    let indices = arrow::compute::sort_to_indices(
        col,
        Some(SortOptions {
            descending: false,
            nulls_first: true,
        }),
        None,
    )
    .map_err(|e| Error::Batching { source: e })?;

    if indices.values().is_sorted() {
        return Ok(rb);
    }

    sort_record_batch_by_indices(rb, &indices)
}

/// Two-column sort. Uses packed u64 sort for dict-encoded parent_id, RowConverter otherwise.
fn sort_two_columns(
    rb: RecordBatch,
    parent_id_col: &ArrayRef,
    id_col: &ArrayRef,
) -> Result<RecordBatch> {
    if parent_id_col.null_count() == 0
        && id_col.null_count() == 0
        && matches!(parent_id_col.data_type(), DataType::Dictionary(_, _))
    {
        return sort_two_columns_packed(rb, parent_id_col, id_col);
    }

    sort_two_columns_row_converter(rb, parent_id_col, id_col)
}

/// RowConverter-based two-column sort (handles native types and nullable columns).
fn sort_two_columns_row_converter(
    rb: RecordBatch,
    parent_id_col: &ArrayRef,
    id_col: &ArrayRef,
) -> Result<RecordBatch> {
    let options = Some(SortOptions {
        descending: false,
        nulls_first: true,
    });
    let sort_columns = vec![
        SortColumn {
            values: parent_id_col.clone(),
            options,
        },
        SortColumn {
            values: id_col.clone(),
            options,
        },
    ];
    let indices =
        super::sort_to_indices(&sort_columns).map_err(|e| Error::Batching { source: e })?;

    if indices.values().is_sorted() {
        return Ok(rb);
    }

    sort_record_batch_by_indices(rb, &indices)
}

/// Packed u64 sort for dict-encoded parent_id + native id.
///
/// Packs `resolved_parent_id` into the high bits and `id` into the low bits of a u64,
/// then sorts the packed values. This avoids the RowConverter's dictionary expansion.
fn sort_two_columns_packed(
    rb: RecordBatch,
    parent_id_col: &ArrayRef,
    id_col: &ArrayRef,
) -> Result<RecordBatch> {
    debug_assert_eq!(parent_id_col.null_count(), 0);
    debug_assert_eq!(id_col.null_count(), 0);

    let len = rb.num_rows();
    let id_shift = match id_col.data_type() {
        DataType::UInt16 => 16u32,
        DataType::UInt32 => 32u32,
        _ => return sort_two_columns_row_converter(rb, parent_id_col, id_col),
    };

    let mut packed: Vec<u64> = Vec::with_capacity(len);

    id_column_dispatch!(
        parent_id_col,
        Native[_T] => {
            // parent_id is native, not dict — caller should not have routed here
            return sort_two_columns_row_converter(rb, parent_id_col, id_col);
        },
        Dictionary[KType, VType] => {
            let dict = parent_id_col.as_any()
                .downcast_ref::<DictionaryArray<KType>>().unwrap();
            let pid_keys = dict.keys().values();
            let pid_vals = dict.values().as_any()
                .downcast_ref::<PrimitiveArray<VType>>().unwrap().values();

            id_column_dispatch!(
                id_col,
                Native[IdType] => {
                    let id_vals = id_col.as_any()
                        .downcast_ref::<PrimitiveArray<IdType>>().unwrap().values();
                    for i in 0..len {
                        let pid = pid_vals[pid_keys[i] as usize] as u64;
                        let id = id_vals[i] as u64;
                        packed.push((pid << id_shift) | id);
                    }
                },
                Dictionary[_K2, _V2] => {
                    // id column is never dict-encoded, but handle gracefully
                    return sort_two_columns_row_converter(rb, parent_id_col, id_col);
                },
                _ => return sort_two_columns_row_converter(rb, parent_id_col, id_col),
            );
        },
        _ => return sort_two_columns_row_converter(rb, parent_id_col, id_col),
    );

    debug_assert_eq!(packed.len(), len);

    if packed.is_sorted() {
        return Ok(rb);
    }

    let mut indices: Vec<u32> = (0..len as u32).collect();
    indices.sort_unstable_by_key(|&i| packed[i as usize]);
    let indices_arr = UInt32Array::from(indices);
    sort_record_batch_by_indices(rb, &indices_arr)
}

/// Extracts an ID column from a record batch
pub(crate) fn extract_id_column(rb: &RecordBatch, column_path: &str) -> Result<ArrayRef> {
    access_column(column_path, &rb.schema(), rb.columns()).ok_or_else(|| Error::ColumnNotFound {
        name: column_path.to_string(),
    })
}

pub(crate) fn sort_record_batch_by_indices(
    rb: RecordBatch,
    indices: &dyn Array,
) -> Result<RecordBatch> {
    assert_eq!(rb.num_rows(), indices.len());

    let (schema, columns, _) = rb.into_parts();
    let new_columns: Vec<_> = columns
        .iter()
        .map(|c| arrow::compute::take(c, indices, None))
        .collect::<arrow::error::Result<Vec<_>>>()
        .map_err(|e| Error::Batching { source: e })?;

    // safety: We did a valid tranformation on all columns
    Ok(RecordBatch::try_new(schema, new_columns).expect("valid record batch"))
}

/// Helper function for accessing the column associated for the (possibly nested) path
pub(crate) fn access_column(path: &str, schema: &Schema, columns: &[ArrayRef]) -> Option<ArrayRef> {
    // handle special case of accessing either the resource ID or scope ID which are nested
    // within a struct
    if let Some(struct_col_name) = struct_column_name(path) {
        let struct_col_idx = schema.index_of(struct_col_name).ok()?;
        let struct_col = columns
            .get(struct_col_idx)?
            .as_any()
            .downcast_ref::<StructArray>()?;
        return struct_col.column_by_name(ID).cloned();
    }

    // otherwise just return column by name
    let (column_idx, _) = schema.fields.find(path)?;
    columns.get(column_idx).cloned()
}

/// if configured to encode the ID column in the nested resource/scope struct array, this
/// helper function simply returns the name of the struct column, and otherwise returns `None`
pub(crate) fn struct_column_name(path: &str) -> Option<&'static str> {
    if path == RESOURCE_ID_COL_PATH {
        return Some(consts::RESOURCE);
    }

    if path == SCOPE_ID_COL_PATH {
        return Some(consts::SCOPE);
    }

    None
}

/// Replaces the column identified by `path` within the array of columns with the new column.
pub(crate) fn replace_column(
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
                if let Some((struct_idx, _)) = struct_column.fields().find(ID) {
                    // replace the encoding metadata on the struct field
                    let mut new_struct_fields = struct_column.fields().to_vec();
                    update_field_encoding_metadata(ID, encoding, &mut new_struct_fields);

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
pub(crate) fn update_field_encoding_metadata(
    path: &str,
    encoding: Option<Encoding>,
    fields: &mut [FieldRef],
) {
    if let Some(struct_col_name) = struct_column_name(path) {
        // replace the field metadata in some nested struct
        let found_field = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name().as_str() == struct_col_name);

        if let Some((idx, field)) = found_field {
            if let DataType::Struct(struct_fields) = field.data_type() {
                let mut new_struct_fields = struct_fields.to_vec();
                update_field_encoding_metadata(ID, encoding, &mut new_struct_fields);

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

#[derive(Debug)]
pub(crate) struct PayloadRelationInfo {
    pub primary_id: Option<PrimaryIdInfo>,
    pub relations: &'static [Relation],
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IdColumnType {
    U16,
    U32,
}

impl IdColumnType {
    pub(crate) fn max(&self) -> u64 {
        match self {
            IdColumnType::U16 => u16::MAX as u64,
            IdColumnType::U32 => u32::MAX as u64,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PrimaryIdInfo {
    pub name: &'static str,
    pub size: IdColumnType,
}

#[derive(Debug)]
pub(crate) struct Relation {
    pub key_col: &'static str,
    pub child_types: &'static [ArrowPayloadType],
}

/// Get the primary ID column info and foreign relations for the given payload type.
pub(crate) fn payload_relations(parent_type: ArrowPayloadType) -> PayloadRelationInfo {
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
        ArrowPayloadType::SpanLinks => PayloadRelationInfo {
            primary_id: None,
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::SpanLinkAttrs],
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

pub(crate) fn payload_to_idx(payload_type: ArrowPayloadType) -> usize {
    let pos = POSITION_LOOKUP[payload_type as usize];
    assert_ne!(pos, UNUSED_INDEX);
    pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_batch;
    use arrow::array::AsArray;
    use arrow::datatypes::UInt16Type;

    /// Helper: extract a column from the result batch as a u16 slice.
    fn col_u16(rb: &RecordBatch, name: &str) -> Vec<u16> {
        let col = rb.column_by_name(name).unwrap();
        col.as_primitive::<UInt16Type>().values().to_vec()
    }

    #[test]
    fn test_native_parent_id_and_id() {
        // parent_id: [2, 1, 2, 1], id: [4, 3, 2, 1]
        // expected order by (parent_id, id): (1,1), (1,3), (2,2), (2,4)
        let rb = record_batch!(
            ("parent_id", UInt16, [2, 1, 2, 1]),
            ("id",        UInt16, [4, 3, 2, 1])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb).unwrap();
        assert_eq!(col_u16(&sorted, "parent_id"), vec![1, 1, 2, 2]);
        assert_eq!(col_u16(&sorted, "id"), vec![1, 3, 2, 4]);
    }

    #[test]
    fn test_dict_parent_id_and_native_id() {
        // Dict<UInt8, UInt16> parent_id with values [10, 20], keys [1, 0, 1, 0]
        // So resolved parent_id = [20, 10, 20, 10], id = [4, 3, 2, 1]
        // expected: (10,1), (10,3), (20,2), (20,4)
        let rb = record_batch!(
            ("parent_id", (UInt8, UInt16), ([1, 0, 1, 0], [10, 20])),
            ("id",        UInt16,         [4, 3, 2, 1])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb).unwrap();
        // parent_id is still dict-encoded after take, so read via id column
        assert_eq!(col_u16(&sorted, "id"), vec![1, 3, 2, 4]);
    }

    #[test]
    fn test_only_id_column() {
        let rb = record_batch!(
            ("id", UInt16, [3, 1, 2])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb).unwrap();
        assert_eq!(col_u16(&sorted, "id"), vec![1, 2, 3]);
    }

    #[test]
    fn test_only_parent_id_column() {
        let rb = record_batch!(
            ("parent_id", UInt16, [3, 1, 2])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb).unwrap();
        assert_eq!(col_u16(&sorted, "parent_id"), vec![1, 2, 3]);
    }

    #[test]
    fn test_neither_column() {
        let rb = record_batch!(
            ("other", UInt16, [3, 1, 2])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb.clone()).unwrap();
        // Should be unchanged
        assert_eq!(col_u16(&sorted, "other"), vec![3, 1, 2]);
    }

    #[test]
    fn test_already_sorted() {
        let rb = record_batch!(
            ("parent_id", UInt16, [1, 1, 2, 2]),
            ("id",        UInt16, [1, 2, 1, 2])
        )
        .unwrap();

        let sorted = sort_by_parent_then_id(rb).unwrap();
        assert_eq!(col_u16(&sorted, "parent_id"), vec![1, 1, 2, 2]);
        assert_eq!(col_u16(&sorted, "id"), vec![1, 2, 1, 2]);
    }
}
