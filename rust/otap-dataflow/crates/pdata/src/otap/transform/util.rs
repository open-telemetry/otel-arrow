// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, MutableArrayData, RecordBatch, StructArray, UInt32Array, make_array,
};
use arrow::compute::partition_validity;
use arrow::datatypes::{ArrowNativeType, UInt16Type, UInt32Type};
use arrow_schema::{DataType, FieldRef, Schema, SortOptions};

use crate::error::{Error, Result};
use crate::otap::raw_batch_store::{POSITION_LOOKUP, UNUSED_INDEX};
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
pub fn take_record_batch_ranges(
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
/// - Single column (native): sort via the normal arrow sort_to_indices with an
///   early return check if we're already sorted.
/// - Two columns (both native): Sort via a specialized two column sort that packs
///   parent_id into the high bits and id into the low bits of a u64 and sorts that
///   instead.
pub fn sort_by_parent_then_id(rb: RecordBatch) -> Result<RecordBatch> {
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

/// Single-column sort. Checks `is_sorted` for early return, then delegates to
/// `arrow::compute::sort_to_indices` for the actual sort.
fn sort_single_column(rb: RecordBatch, col: &ArrayRef) -> Result<RecordBatch> {
    // Fast path: check if already sorted (only when non-null, to avoid resolving nullability).
    id_column_dispatch!(
        col,
        Native[T] => {
            if col.null_count() == 0 && col.as_primitive::<T>().values().is_sorted() {
                return Ok(rb);
            }
        },
        Dictionary[KType, VType] => {
            if col.null_count() == 0 {
                // safety: id_column_dispatch! matched the dictionary key/value types
                let dict = col.as_dictionary::<KType>();
                let keys = dict.keys().values();
                let vals = dict.values().as_primitive::<VType>().values();
                if keys.windows(2).all(|w| vals[w[0].as_usize()] <= vals[w[1].as_usize()]) {
                    return Ok(rb);
                }
            }
        },
        _ => {
            return Err(Error::UnsupportedParentIdType {
                actual: col.data_type().clone(),
            });
        },
    );

    let options = Some(SortOptions {
        descending: false,
        nulls_first: true,
    });
    let indices = arrow::compute::sort_to_indices(col, options, None)
        .map_err(|e| Error::Batching { source: e })?;

    if indices.values().is_sorted() {
        return Ok(rb);
    }

    sort_record_batch_by_indices(rb, &indices)
}

/// Two-column sort. Packs parent_id and id into u64 sort keys for a single-pass sort.
fn sort_two_columns(
    rb: RecordBatch,
    parent_id_col: &ArrayRef,
    id_col: &ArrayRef,
) -> Result<RecordBatch> {
    if parent_id_col.null_count() != 0 {
        return Err(Error::NullParentId);
    }

    id_column_dispatch!(
        parent_id_col,
        Native[PType] => {
            // safety: id_column_dispatch! matched the native data type
            let pid_vals = parent_id_col.as_primitive::<PType>().values();
            dispatch_id_and_pack(rb, |i| pid_vals[i as usize] as u64, id_col)
        },
        Dictionary[KType, VType] => {
            // safety: id_column_dispatch! matched the dictionary key/value types
            let dict = parent_id_col.as_dictionary::<KType>();
            let pid_keys = dict.keys().values();
            let pid_vals = dict.values().as_primitive::<VType>().values();
            dispatch_id_and_pack(
                rb,
                |i| pid_vals[pid_keys[i as usize].as_usize()] as u64,
                id_col,
            )
        },
        _ => {
            Err(Error::UnsupportedParentIdType {
                actual: parent_id_col.data_type().clone(),
            })
        },
    )
}

/// Dispatches on the id column's native type, then delegates to packed sort.
fn dispatch_id_and_pack(
    rb: RecordBatch,
    resolve_pid: impl Fn(u32) -> u64,
    id_col: &ArrayRef,
) -> Result<RecordBatch> {
    match id_col.data_type() {
        DataType::UInt16 => {
            let id_vals = id_col.as_primitive::<UInt16Type>().values();
            sort_two_columns_packed(rb, resolve_pid, id_col.as_ref(), id_vals)
        }
        DataType::UInt32 => {
            let id_vals = id_col.as_primitive::<UInt32Type>().values();
            sort_two_columns_packed(rb, resolve_pid, id_col.as_ref(), id_vals)
        }
        other => Err(Error::UnsupportedParentIdType {
            actual: other.clone(),
        }),
    }
}

/// Pack parent_id and id into u64 sort keys and sort.
///
/// High 32 bits hold the resolved `parent_id` value, low 32 bits hold the `id` value
///
/// Parent_id must be non-null (enforced by caller). When the id column has nulls,
/// null-id rows cannot be packed (no valid id value), so we partition into two groups,
/// sort each independently, then merge them with nulls_first semantics for id.
fn sort_two_columns_packed<T: ArrowNativeType>(
    rb: RecordBatch,
    resolve_pid: impl Fn(u32) -> u64,
    id_col: &dyn Array,
    id_vals: &[T],
) -> Result<RecordBatch> {
    let len = id_vals.len();

    let pack = |i: u32| -> u64 { (resolve_pid(i) << 32) | id_vals[i as usize].as_usize() as u64 };

    // Partition by id nullability. When there are no nulls, null_indices is empty
    // and valid_indices contains all row indices - the merge becomes a pass-through.
    let (mut valid_indices, mut null_indices) = partition_validity(id_col);

    // Sort null-id rows by parent_id only.
    null_indices.sort_unstable_by_key(|&i| resolve_pid(i));

    // Precompute packed keys for sorting. Only valid row slots are populated
    let mut packed = vec![0u64; len];
    for &i in &valid_indices {
        packed[i as usize] = pack(i);
    }
    valid_indices.sort_unstable_by_key(|&i| packed[i as usize]);

    // Merge: null-id rows first at equal parent_id (nulls_first for id column).
    let mut indices: Vec<u32> = Vec::with_capacity(len);
    let (mut ni, mut vi) = (0, 0);
    while ni < null_indices.len() && vi < valid_indices.len() {
        let parent_id = packed[valid_indices[vi] as usize] >> 32;
        let null_parent_id = resolve_pid(null_indices[ni]);
        if null_parent_id <= parent_id {
            indices.push(null_indices[ni]);
            ni += 1;
        } else {
            indices.push(valid_indices[vi]);
            vi += 1;
        }
    }

    // Add any leftovers
    indices.extend_from_slice(&null_indices[ni..]);
    indices.extend_from_slice(&valid_indices[vi..]);

    if indices.is_sorted() {
        return Ok(rb);
    }

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
    pub size: IdColumnType,
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
                    size: IdColumnType::U16,
                },
                Relation {
                    key_col: SCOPE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ScopeAttrs],
                    size: IdColumnType::U16,
                },
                Relation {
                    key_col: ID,
                    child_types: &[ArrowPayloadType::LogAttrs],
                    size: IdColumnType::U16,
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
                    size: IdColumnType::U16,
                },
                Relation {
                    key_col: SCOPE_ID_COL_PATH,
                    child_types: &[ArrowPayloadType::ScopeAttrs],
                    size: IdColumnType::U16,
                },
                Relation {
                    key_col: ID,
                    child_types: &[
                        ArrowPayloadType::SpanAttrs,
                        ArrowPayloadType::SpanEvents,
                        ArrowPayloadType::SpanLinks,
                    ],
                    size: IdColumnType::U16,
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
                size: IdColumnType::U32,
            }],
        },
        ArrowPayloadType::SpanLinks => PayloadRelationInfo {
            primary_id: None,
            relations: &[Relation {
                key_col: ID,
                child_types: &[ArrowPayloadType::SpanLinkAttrs],
                size: IdColumnType::U32,
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
                        size: IdColumnType::U16,
                    },
                    Relation {
                        key_col: SCOPE_ID_COL_PATH,
                        child_types: &[ArrowPayloadType::ScopeAttrs],
                        size: IdColumnType::U16,
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
                        size: IdColumnType::U16,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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
                size: IdColumnType::U32,
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

    use arrow::array::{DictionaryArray, UInt16Array};
    use arrow::compute::SortColumn;
    use arrow::datatypes::{Field, UInt8Type};
    use arrow_schema::SortOptions;
    use std::sync::Arc;

    /// Oracle: the legacy RowConverter-based sort. Used as ground truth for all tests.
    fn sort_oracle(rb: RecordBatch) -> RecordBatch {
        let schema = rb.schema();
        let cols = rb.columns();
        let options = Some(SortOptions {
            descending: false,
            nulls_first: true,
        });

        let mut sort_columns: Vec<SortColumn> = Vec::new();
        if let Some((idx, _)) = schema.column_with_name("parent_id") {
            sort_columns.push(SortColumn {
                values: cols[idx].clone(),
                options,
            });
        }
        if let Some((idx, _)) = schema.column_with_name("id") {
            sort_columns.push(SortColumn {
                values: cols[idx].clone(),
                options,
            });
        }

        if sort_columns.is_empty() {
            return rb;
        }

        let indices = super::super::sort_to_indices(&sort_columns).expect("oracle sort failed");
        let (schema, columns, _) = rb.into_parts();
        let new_columns: Vec<_> = columns
            .iter()
            .map(|c| arrow::compute::take(c, &indices, None).expect("take failed"))
            .collect();
        RecordBatch::try_new(schema, new_columns).expect("valid record batch")
    }

    /// Assert that our optimized sort produces the same result as the oracle.
    fn assert_matches_oracle(rb: RecordBatch) {
        let expected = sort_oracle(rb.clone());
        let actual = sort_by_parent_then_id(rb).expect("optimized sort failed");
        assert_eq!(expected.num_columns(), actual.num_columns());
        for i in 0..expected.num_columns() {
            assert_eq!(
                expected.column(i).as_ref(),
                actual.column(i).as_ref(),
                "column {} mismatch",
                expected.schema().field(i).name(),
            );
        }
    }

    /// Helper: build a RecordBatch from pre-built (name, array) pairs.
    fn batch_from_arrays(cols: Vec<(&str, ArrayRef)>) -> RecordBatch {
        let schema = Arc::new(Schema::new(
            cols.iter()
                .map(|(name, arr)| Field::new(*name, arr.data_type().clone(), true))
                .collect::<Vec<_>>(),
        ));
        let arrays: Vec<ArrayRef> = cols.into_iter().map(|(_, a)| a).collect();
        RecordBatch::try_new(schema, arrays).expect("valid record batch")
    }

    #[test]
    fn test_dict_u8_u32() {
        let rb = record_batch!(
            ("parent_id", (UInt8, UInt32), ([1, 0, 1, 0], [10, 20])),
            ("id", UInt16, [4, 3, 2, 1])
        )
        .unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_dict_16_u32() {
        let rb = record_batch!(
            ("parent_id", (UInt16, UInt32), ([1, 0, 1, 0], [10, 20])),
            ("id", UInt16, [4, 3, 2, 1])
        )
        .unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_only_id_column() {
        let rb = record_batch!(("id", UInt16, [3, 1, 2])).unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_only_parent_id_column() {
        let rb = record_batch!(("parent_id", UInt16, [3, 1, 2])).unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_neither_column() {
        let rb = record_batch!(("other", UInt16, [3, 1, 2])).unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_already_sorted() {
        let rb = record_batch!(
            ("parent_id", UInt16, [1, 1, 2, 2]),
            ("id", UInt16, [1, 2, 1, 2])
        )
        .unwrap();
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_native_id_with_nulls() {
        let id: ArrayRef = Arc::new(UInt16Array::from(vec![Some(3), None, Some(1)]));
        let rb = batch_from_arrays(vec![("id", id)]);
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_native_parent_id_with_nulls_returns_error() {
        let pid: ArrayRef = Arc::new(UInt16Array::from(vec![Some(2), None, Some(1)]));
        let id: ArrayRef = Arc::new(UInt16Array::from(vec![Some(10), Some(20), Some(30)]));
        let rb = batch_from_arrays(vec![("parent_id", pid), ("id", id)]);

        let result = sort_by_parent_then_id(rb);
        assert!(matches!(result, Err(Error::NullParentId)));
    }

    #[test]
    fn test_dict_parent_id_and_native_id_with_null_ids() {
        let pid: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            arrow::array::UInt8Array::from(vec![1, 0, 1, 0]),
            Arc::new(UInt16Array::from(vec![10, 20])),
        ));
        let id: ArrayRef = Arc::new(UInt16Array::from(vec![None, Some(3), Some(2), Some(1)]));
        let rb = batch_from_arrays(vec![("parent_id", pid), ("id", id)]);
        assert_matches_oracle(rb);
    }

    #[test]
    fn test_null_parent_id_returns_error() {
        let pid: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            arrow::array::UInt8Array::from(vec![Some(1), None, Some(1), Some(0)]),
            Arc::new(UInt16Array::from(vec![10, 20])),
        ));
        let id: ArrayRef = Arc::new(UInt16Array::from(vec![4, 3, 2, 1]));
        let rb = batch_from_arrays(vec![("parent_id", pid), ("id", id)]);

        let result = sort_by_parent_then_id(rb);
        assert!(matches!(result, Err(Error::NullParentId)));
    }
}
