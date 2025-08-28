// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for manipulating the arrow record batch [`RecordBatch`] so that it
//! has a schema that's compatible with the Parquet writer.
//!
//! The main reason we can't write the record batch verbatim is because OTAP considers some columns
//! optional, but parquet spec requires that each row group contain a column chunk for every field
//! in the schema (and the column chunks must be in the correct order). 
//! 
//! This means we can't receive two consecutive OTAP batches for some payload type and write them 
//! into the same writer. To  handle this, we insert all null columns for missing columns (or all 
//! default-value where the column is not nullable), and also arrange the columns so they're always
//! in the same order.
//!
//! Note that although we also switch between the Dictionary and Native encodings, we don't need to
//! actually convert the existing columns to all be the same type. Parquet writer is able to accept
//! logically compatible types, which includes compatibility between types like `T` (native array)
//! and `Dictionary<K, T>` (for any `K`).

use std::iter::repeat_n;
use std::sync::{Arc, LazyLock};

use arrow::array::{
    ArrayRef, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float32Array, Float64Array, Int32Array, Int64Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
use otap_df_engine::error::Error;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

/// Transform the
// TODO comments
pub fn transform_to_known_schema(
    record_batch: &RecordBatch,
    payload_type: ArrowPayloadType,
) -> Result<RecordBatch, Error> {
    let current_schema = record_batch.schema_ref();
    let template_schema = get_template_schema(payload_type);

    let mut new_columns = Vec::with_capacity(template_schema.fields.len());
    let mut new_fields = Vec::with_capacity(template_schema.fields.len());

    for template_field in template_schema.fields() {
        match current_schema.index_of(template_field.name()) {
            Ok(current_field_index) => {
                // TODO handle struct here
                new_columns.push(record_batch.column(current_field_index).clone());
                new_fields.push(current_schema.fields[current_field_index].clone());
            }
            Err(_) => {
                let new_column = if template_field.is_nullable() {
                    get_all_null_column(template_field.data_type(), record_batch.num_rows())?
                } else {
                    get_all_default_value_column(
                        template_field.data_type(),
                        record_batch.num_rows(),
                    )?
                };
                let new_field = template_field
                    .as_ref()
                    .clone()
                    .with_data_type(new_column.data_type().clone());

                new_columns.push(new_column);
                new_fields.push(Arc::new(new_field));
            }
        }
    }

    // safety: this shouldn't fail b/c we're creating a record batch where the columns all have
    // the correct length and their datatypes match the schema
    let new_rb = RecordBatch::try_new(
        Arc::new(Schema::new(new_fields)),
        new_columns,
    ).expect("unexpected error creating record batch with known schema");

    Ok(new_rb)
}

fn get_template_schema(payload_type: ArrowPayloadType) -> &'static Schema {
    match payload_type {
        ArrowPayloadType::Logs => &LOGS_TEMPLATE_SCHEMA,
        _ => {
            todo!()
        }
    }
}

fn get_all_null_column(data_type: &DataType, length: usize) -> Result<ArrayRef, Error> {
    // TODO once run-end encoding, we can save some memory here by  allocating a single RunArray
    // with one null value, and one run-end of `length`. This would allow us to allocate a few
    // single length buffers instead of full-length empty buffers
    // https://github.com/apache/arrow-rs/issues/8016
    Ok(match data_type {
        DataType::Binary => Arc::new(BinaryArray::new_null(length)),
        DataType::Boolean => Arc::new(BinaryArray::new_null(length)),
        DataType::FixedSizeBinary(fsl_len) => {
            Arc::new(FixedSizeBinaryArray::new_null(*fsl_len, length))
        }
        DataType::Float32 => Arc::new(Float32Array::new_null(length)),
        DataType::Float64 => Arc::new(Float64Array::new_null(length)),
        DataType::Int32 => Arc::new(Int32Array::new_null(length)),
        DataType::Int64 => Arc::new(Int64Array::new_null(length)),
        DataType::UInt8 => Arc::new(UInt8Array::new_null(length)),
        DataType::UInt16 => Arc::new(UInt16Array::new_null(length)),
        DataType::UInt32 => Arc::new(UInt32Array::new_null(length)),
        DataType::UInt64 => Arc::new(UInt64Array::new_null(length)),
        DataType::Utf8 => Arc::new(StringArray::new_null(length)),
        DataType::Timestamp(time_unit, _) => match *time_unit {
            TimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::new_null(length)),
            _ => {
                todo!()
            }
        },

        DataType::Struct(fields) => get_struct_full_of_nulls_or_defaults(fields, length, true)?,
        _ => {
            todo!()
        }
    })
}

fn get_all_default_value_column(data_type: &DataType, length: usize) -> Result<ArrayRef, Error> {
    // TODO once run-end encoding, we can save some memory here by  allocating a single RunArray
    // with one default value, and one run-end of `length`. This would allow us to allocate a few
    // single length buffers instead of full-length empty buffers
    // https://github.com/apache/arrow-rs/issues/8016
    Ok(match data_type {
        DataType::Binary => Arc::new(BinaryArray::from_iter_values(repeat_n(b"", length))),
        DataType::Boolean => Arc::new(BooleanArray::from_iter(repeat_n(Some(false), length))),
        DataType::FixedSizeBinary(fsl_len) => {
            Arc::new(FixedSizeBinaryArray::try_from_iter(repeat_n(vec![0; *fsl_len as usize], length)).expect("can create FSB array from iter of correct len"))
        },
        DataType::Float32 => Arc::new(Float32Array::from_iter_values(repeat_n(0.0, length))),
        DataType::Float64 => Arc::new(Float64Array::from_iter_values(repeat_n(0.0, length))),
        DataType::Int32 => Arc::new(Int32Array::from_iter_values(repeat_n(0, length))),
        DataType::Int64 => Arc::new(Int64Array::from_iter_values(repeat_n(0, length))),
        DataType::UInt8 => Arc::new(UInt8Array::from_iter_values(repeat_n(0, length))),
        DataType::UInt16 => Arc::new(UInt16Array::from_iter_values(repeat_n(0, length))),
        DataType::UInt32 => Arc::new(UInt32Array::from_iter_values(repeat_n(0, length))),
        DataType::UInt64 => Arc::new(UInt64Array::from_iter_values(repeat_n(0, length))),
        DataType::Utf8 => Arc::new(StringArray::from_iter(repeat_n(Some(""), length))),
        DataType::Timestamp(time_unit, _) => match *time_unit {
            TimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from_iter_values(repeat_n(0, length))),
            _ => {
                todo!()
            }
        },

        DataType::Struct(fields) => get_struct_full_of_nulls_or_defaults(fields, length, false)?,
        _ => {
            todo!()
        }
    })
}

/// creates a a struct where all the columns are all null, or all default value if non-nullable.
/// the intention is that this will be a stand-in for the struct column of a record batch that is
/// missing some struct column.
fn get_struct_full_of_nulls_or_defaults(fields: &Fields, length: usize, struct_nullable: bool) -> Result<ArrayRef, Error> {
    let mut new_fields = Vec::with_capacity(fields.len());
    let mut new_columns = Vec::with_capacity(fields.len());

    for field in fields {
        let new_column = if field.is_nullable() {
            get_all_null_column(field.data_type(), length)?
        } else {
            get_all_default_value_column(field.data_type(), length)?
        };
        new_fields.push(field.clone());
        new_columns.push(new_column);
    }

    let nulls = (!struct_nullable).then(|| NullBuffer::new_valid(length));
    let struct_array = StructArray::new(Fields::from(new_fields), new_columns, nulls);
    
    Ok(Arc::new(struct_array))
}

// template schemas for various data types:
//
// note: these shouldn't be interpreted a comprehensive reference for OTAP generally. while the
// schemas below do contain every field and their logical datatypes, they lack information such
// as where dictionary encoding is used and other metadata such as which fields are optional. a
// better reference would probably be whats in the otel-arrow go code in pkg/otel

static RESOURCE_TEMPLATE_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
    ])
});

static SCOPE_TEMPLATE_FIELDS: LazyLock<Fields> = LazyLock::new(|| {
    Fields::from(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(consts::NAME, DataType::Utf8, true),
        Field::new(consts::VERSION, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
    ])
});

static LOGS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(
            consts::RESOURCE,
            DataType::Struct(RESOURCE_TEMPLATE_FIELDS.clone()),
            false,
        ),
        Field::new(
            consts::SCOPE,
            DataType::Struct(SCOPE_TEMPLATE_FIELDS.clone()),
            true
        )
    ])
});

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{RecordBatch, UInt16Array};

    #[test]
    fn test_coalesces_new_columns_with_empty_columns() {
        let log_attrs_record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false),
                // "resource" is missing, so we should insert a new non-nullable struct
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(Fields::from(vec![
                        Field::new(consts::ID, DataType::UInt16, true),
                        // scope will have some missing columns (name, version, drop_attr's_count)
                        // so we should see them inserted
                    ])),
                    false
                )
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(StructArray::new(
                    Fields::from(vec![
                        Field::new(consts::ID, DataType::UInt16, true),
                    ]),
                    vec![
                        Arc::new(UInt16Array::from_iter(vec![Some(0), None, Some(1)]))
                    ],
                    Some(NullBuffer::new_valid(3))
                )),
            ]
        ).unwrap();

        let result = transform_to_known_schema(&log_attrs_record_batch, ArrowPayloadType::Logs).unwrap();

        let expected_resource_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true),
            Field::new(consts::SCHEMA_URL, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]);

        let expected_scope_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::VERSION, DataType::Utf8, true),
            Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        ]);

        let expected = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, false),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(expected_resource_fields.clone()),
                    false
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(expected_scope_fields.clone()),
                    false
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(
                    StructArray::new(
                        expected_resource_fields.clone(),
                        vec![
                            Arc::new(UInt16Array::new_null(3)),
                            Arc::new(StringArray::new_null(3)),
                            Arc::new(UInt32Array::new_null(3)),
                        ],
                        Some(NullBuffer::new_valid(3))
                    )
                ),
                Arc::new(
                    StructArray::new(
                        expected_scope_fields.clone(),
                        vec![
                            Arc::new(UInt16Array::from_iter(vec![Some(0), None, Some(1)])),
                            Arc::new(StringArray::new_null(3)),
                            Arc::new(StringArray::new_null(3)),
                            Arc::new(UInt32Array::new_null(3)),
                        ],
                        Some(NullBuffer::new_valid(3))
                    )
                )
            ]
        ).unwrap();

        assert_eq!(result, expected)
    }
}