use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, MutableArrayData, PrimitiveArray, RecordBatch, StructArray, make_array,
};
use arrow::compute::{SortColumn, sort_to_indices};
use arrow::datatypes::UInt32Type;
use arrow_schema::{DataType, FieldRef, Schema, SortOptions};

use crate::error::{Error, Result};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts::{ID, PARENT_ID};
use crate::schema::{FieldExt, consts};

use super::transport_optimize::{Encoding, RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH};

// Maps a DataType variant name to its corresponding Arrow primitive type.
// #[macro_export]
// #[rustfmt::skip]
// macro_rules! arrow_primitive_type {
//     (UInt8)   => { ::arrow_array::types::UInt8Type   };
//     (UInt16)  => { ::arrow_array::types::UInt16Type  };
//     (UInt32)  => { ::arrow_array::types::UInt32Type  };
//     (UInt64)  => { ::arrow_array::types::UInt64Type  };
//     (Int8)    => { ::arrow_array::types::Int8Type    };
//     (Int16)   => { ::arrow_array::types::Int16Type   };
//     (Int32)   => { ::arrow_array::types::Int32Type   };
//     (Int64)   => { ::arrow_array::types::Int64Type   };
//     (Float32) => { ::arrow_array::types::Float32Type };
//     (Float64) => { ::arrow_array::types::Float64Type };
// }

/// Dispatches on an Arrow DataType, transparently handling Dictionary-encoded variants.
///
/// Each matched arm binds `T` to the corresponding Arrow primitive type. The `_` arm
/// is used as the fallback for both unmatched plain types and unmatched Dictionary
/// value types.
#[macro_export]
macro_rules! value_type_dispatch {
    (
        $data_type:expr,
        {
            $( $( $variant:ident )|+ => $body:block ),+
            $(,)?
            _ => $default:block $(,)?
        }
    ) => {
        match $data_type {
            $($(
                ::arrow_schema::DataType::$variant => {
                    ::paste::paste! {
                        type T = ::arrow::datatypes::[<$variant Type>];
                        $body
                    }
                },
            )+)+
            ::arrow_schema::DataType::Dictionary(_, __value_type) => match __value_type.as_ref() {
                $($(
                    ::arrow_schema::DataType::$variant => {
                        ::paste::paste! {
                            type T = ::arrow::datatypes::[<$variant Type>];
                            $body
                        }
                    },
                )+)+
                _ => $default,
            },
            _ => $default,
        }
    };
}

pub(crate) fn take_ranges(
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

pub(crate) fn remove_ranges(
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
            new_data.extend(0, pos, pos + range.start);
            pos = range.end;
        }
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

// TODO [JD]: Move any tests for this stuff over to here
// TODO [JD]: Optimize this because doing a column sort with dictionary encoded
// causes complete expansion of the dictionary. We can probably do this faster
// for a two column sort. Transport_optimize has some optimizations for this
// and we can steal some techniques.
pub(crate) fn sort_by_parent_then_id(rb: RecordBatch) -> Result<RecordBatch> {
    let schema = rb.schema();
    let cols = rb.columns();

    let options = Some(SortOptions {
        descending: false,
        nulls_first: true,
    });

    let mut sort_columns: Vec<SortColumn> = Vec::new();

    if let Some((parent_id_idx, _)) = schema.column_with_name(PARENT_ID) {
        sort_columns.push(SortColumn {
            values: cols[parent_id_idx].clone(),
            options,
        });
    }

    if let Some((id_idx, _)) = schema.column_with_name(ID) {
        sort_columns.push(SortColumn {
            values: cols[id_idx].clone(),
            options,
        });
    }

    if sort_columns.is_empty() {
        return Ok(rb);
    }

    let indices =
        super::sort_to_indices(&sort_columns).map_err(|e| Error::Batching { source: e })?;
    sort_record_batch_by_indices(rb, &indices)
}

pub(crate) fn sort_record_batch_by_id_col(
    rb: RecordBatch,
    col: &str,
) -> Result<(RecordBatch, PrimitiveArray<UInt32Type>)> {
    let Ok(id_col) = extract_id_column(&rb, col) else {
        return Err(Error::ColumnNotFound {
            name: col.to_string(),
        });
    };

    let indices =
        sort_to_indices(&id_col, None, None).map_err(|e| Error::Batching { source: e })?;
    let rb = sort_record_batch_by_indices(rb, &indices)?;
    Ok((rb, indices))

    // TODO: Consider some kind of dispatch macro for this that lets us
    // dispatch based on the value type of the IDColumn so we don't have to have
    // this kind of struct all over the place.
    // match id_col.data_type() {
    //     DataType::UInt16 => {
    //         todo!()
    //     }
    //     DataType::UInt32 => {
    //         todo!()
    //     }
    //     DataType::Dictionary(_, value_type) => match value_type.as_ref() {
    //         DataType::UInt16 => {
    //             todo!()
    //         }
    //         DataType::UInt32 => {
    //             todo!()
    //         }
    //         _ => Err(Error::UnsupportedDictionaryValueType {
    //             expect_oneof: vec![DataType::UInt16, DataType::UInt32],
    //             actual: value_type.as_ref().clone(),
    //         }),
    //     },
    //     _ => {
    //         Err(Error::ColumnDataTypeMismatch {
    //             name: col.to_string(),
    //             expect: DataType::UInt16, // or UInt32
    //             actual: id_col.data_type().clone(),
    //         })
    //     }
    // }
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
    let (schema, columns, _) = rb.into_parts();
    let new_columns: Vec<_> = columns
        .iter()
        .map(|c| arrow::compute::take(c, indices, None))
        .collect::<arrow::error::Result<Vec<_>>>()
        .map_err(|e| Error::Batching { source: e })?;

    // safety: We did a valid tranformation on all columns
    Ok(RecordBatch::try_new(schema, new_columns).expect("valid record batch"))
}

/// Removes rows at the given ranges from a record batch by slicing out the
/// valid (non-violation) ranges and concatenating them back together.
/// TODO [JD]: We can make this a lot faster
pub(crate) fn remove_record_batch_ranges(
    rb: RecordBatch,
    ranges: &[Range<usize>],
) -> Result<RecordBatch> {
    let total_len = rb.num_rows();
    let schema = rb.schema();

    let mut valid_ranges = Vec::new();
    let mut pos = 0;
    for r in ranges {
        if pos < r.start {
            valid_ranges.push(pos..r.start);
        }
        pos = r.end;
    }
    if pos < total_len {
        valid_ranges.push(pos..total_len);
    }

    if valid_ranges.is_empty() {
        return Ok(RecordBatch::new_empty(schema));
    }

    let slices: Vec<RecordBatch> = valid_ranges
        .iter()
        .map(|r| rb.slice(r.start, r.end - r.start))
        .collect();

    arrow::compute::concat_batches(&schema, &slices).map_err(|e| Error::Batching { source: e })
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
