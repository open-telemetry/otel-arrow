// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for manipulating the arrow record batch [`RecordBatch`] so that it
//! has a schema that's compatible with the Parquet writer.
//!
//! The main reason we can't write the record batch verbatim is because OTAP considers some columns
//! optional, but parquet spec requires that each row group contain a column chunk for every field
//! in the schema. This means we can't receive two consecutive OTAP batches for some payload type
//! and write them into the same writer.
//!
//! Note that although we also switch between the Dictionary and Native encodings, we don't need to
//! actually convert the existing columns to all be the same type. Parquet writer is able to accept
//! logically compatible types, which includes compatibility between types like `T` (native array)
//! and `Dictionary<K, T>` (for any `K`).
//!
//! TODO do we also need to handle the mixed-up column order?
//!
//! To handle this, we need to add all-null columns for all the optional columns.

use std::sync::{Arc, LazyLock};

use arrow::array::{
    ArrayRef, BinaryArray, FixedSizeBinaryArray, Float32Array, Float64Array, Int64Array,
    RecordBatch, StringArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
    UInt64Array,
};
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

    todo!()
}

fn get_template_schema(payload_type: ArrowPayloadType) -> &'static Schema {
    match payload_type {
        ArrowPayloadType::Logs => &LOGS_TEMPLATE_SCHEMA,
        _ => {
            todo!()
        }
    }
}

static LOGS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(
            consts::RESOURCE,
            DataType::Struct(Fields::from(vec![Field::new(
                consts::ID,
                DataType::UInt16,
                true,
            )])),
            false,
        ),
    ])
});

fn get_all_null_column(data_type: &DataType, length: usize) -> Result<ArrayRef, Error> {
    // TODO once run-end encoding, we can save some memory here by  allocating a single RunArray
    // with one null value, and one run-end of `length`.
    // https://github.com/apache/arrow-rs/issues/8016
    Ok(match data_type {
        DataType::Binary => Arc::new(BinaryArray::new_null(length)),
        DataType::Boolean => Arc::new(BinaryArray::new_null(length)),
        DataType::FixedSizeBinary(fsl_len) => {
            Arc::new(FixedSizeBinaryArray::new_null(*fsl_len, length))
        }
        DataType::Float32 => Arc::new(Float32Array::new_null(length)),
        DataType::Float64 => Arc::new(Float64Array::new_null(length)),
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

        DataType::Struct(fields) => get_struct_full_of_nulls(fields, true)?,
        _ => {
            todo!()
        }
    })
}

fn get_all_default_value_column(data_type: &DataType, length: usize) -> Result<ArrayRef, Error> {
    // TODO once run-end encoding, we can save some memory here by  allocating a single RunArray
    // with one default value, and one run-end of `length`.
    // https://github.com/apache/arrow-rs/issues/8016
    match data_type {
        DataType::Struct(fields) => get_struct_full_of_nulls(fields, false),
        _ => {
            todo!()
        }
    }
}

fn get_struct_full_of_nulls(data_type: &Fields, struct_nullable: bool) -> Result<ArrayRef, Error> {
    todo!()
}
