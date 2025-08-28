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
    Array, ArrayRef, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float32Array, Float64Array,
    Int32Array, Int64Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
    UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
use futures::future::Lazy;
use otap_df_engine::error::Error;
use otap_df_otlp::proto::opentelemetry::metrics::v1::metric::Data;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts::ATTRIBUTE_SER;
use otel_arrow_rust::schema::{FieldExt, consts};

/// Transform the
// TODO comments
pub fn transform_to_known_schema(
    record_batch: &RecordBatch,
    payload_type: ArrowPayloadType,
) -> Result<RecordBatch, Error> {
    let current_schema = record_batch.schema_ref();
    let template_schema = get_template_schema(payload_type);

    let (new_columns, new_fields) = transform_to_known_schema_internal(
        record_batch.num_rows(),
        record_batch.columns(),
        current_schema.fields(),
        template_schema.fields(),
    )?;

    // safety: this shouldn't fail b/c we're creating a record batch where the columns all have
    // the correct length and their datatypes match the schema
    let new_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns)
        .expect("unexpected error creating record batch with known schema");

    Ok(new_rb)
}

fn transform_struct_to_known_schema(
    num_rows: usize,
    current_array: &StructArray,
    template_fields: &Fields,
) -> Result<StructArray, Error> {
    let (new_columns, new_fields) = transform_to_known_schema_internal(
        num_rows,
        current_array.columns(),
        current_array.fields(),
        template_fields,
    )?;

    Ok(StructArray::new(
        new_fields,
        new_columns,
        current_array.nulls().cloned(),
    ))
}

fn transform_to_known_schema_internal(
    num_rows: usize,
    current_columns: &[ArrayRef],
    current_fields: &Fields,
    template_fields: &Fields,
) -> Result<(Vec<ArrayRef>, Fields), Error> {
    let mut new_columns = Vec::with_capacity(template_fields.len());
    let mut new_fields = Vec::with_capacity(template_fields.len());

    for template_field in template_fields {
        // TODO -- the last 3 blocks of each of the some/none branches are the same here. We might
        // but first need to figure out if we need to preserve the metadata?
        match current_fields.find(template_field.name()) {
            Some((current_field_index, current_field)) => {
                // column exists, reuse the existing column..
                // let current_field = &current_schema.fields[current_field_index];
                let current_column = &current_columns[current_field_index];
                let new_column = if let DataType::Struct(_) = current_field.data_type() {
                    // handle struct column
                    let new_struct_arr = transform_struct_to_known_schema(
                        num_rows,
                        // safety: we've just checked the datatype
                        current_column
                            .as_any()
                            .downcast_ref()
                            .expect("can downcast to struct"),
                        // TODO -- need to return an error here if this is None b/c it means that
                        // the record batch we received had a struct column where it shouldn't
                        template_field.as_struct_fields().unwrap(),
                    )?;

                    Arc::new(new_struct_arr)
                } else {
                    // otherwise just keep the existing column
                    current_column.clone()
                };

                // TODO if the datatypes are the same here, there's no need to create a new Arc
                let new_field = current_field
                    .as_ref()
                    .clone()
                    .with_data_type(new_column.data_type().clone());
                new_columns.push(new_column);
                new_fields.push(Arc::new(new_field));
            }

            None => {
                // column doesn't exist, add a new "empty" column..
                let new_column = if template_field.is_nullable() {
                    get_all_null_column(template_field.data_type(), num_rows)?
                } else {
                    get_all_default_value_column(template_field.data_type(), num_rows)?
                };

                // TODO if the datatypes are the same here, there's no need to create a new Arc
                let new_field = template_field
                    .as_ref()
                    .clone()
                    .with_data_type(new_column.data_type().clone());
                new_columns.push(new_column);
                new_fields.push(Arc::new(new_field));
            }
        }
    }

    Ok((new_columns, Fields::from(new_fields)))
}

fn get_all_null_column(data_type: &DataType, length: usize) -> Result<ArrayRef, Error> {
    // TODO once run-end encoding, we can save some memory here by  allocating a single RunArray
    // with one null value, and one run-end of `length`. This would allow us to allocate a few
    // single length buffers instead of full-length empty buffers
    // https://github.com/apache/arrow-rs/issues/8016
    Ok(match data_type {
        DataType::Binary => Arc::new(BinaryArray::new_null(length)),
        DataType::Boolean => Arc::new(BooleanArray::new_null(length)),
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
        DataType::FixedSizeBinary(fsl_len) => Arc::new(
            FixedSizeBinaryArray::try_from_iter(repeat_n(vec![0; *fsl_len as usize], length))
                .expect("can create FSB array from iter of correct len"),
        ),
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
            TimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from_iter_values(repeat_n(
                0, length,
            ))),
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
fn get_struct_full_of_nulls_or_defaults(
    fields: &Fields,
    length: usize,
    struct_nullable: bool,
) -> Result<ArrayRef, Error> {
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

fn get_template_schema(payload_type: ArrowPayloadType) -> &'static Schema {
    match payload_type {
        ArrowPayloadType::Logs => &LOGS_TEMPLATE_SCHEMA,
        ArrowPayloadType::UnivariateMetrics | ArrowPayloadType::MultivariateMetrics => {
            &METRICS_TEMPLATE_SCHEMA
        }
        ArrowPayloadType::SummaryDataPoints => &SUMMARY_DP_TEMPLATE_SCHEMA,
        ArrowPayloadType::NumberDataPoints => &NUMBERS_DP_TEMPLATE_SCHEMA,
        ArrowPayloadType::HistogramDataPoints => &HISTOGRAM_DP_TEMPLATE_SCHEMA,
        ArrowPayloadType::ExpHistogramDataPoints => &EXP_HISTOGRAM_DP_TEMPLATE_SCHEMA,
        ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => &EXEMPLAR_TEMPLATE_SCHEMA,
        ArrowPayloadType::Spans => &SPANS_TEMPLATE_SCHEMA,
        ArrowPayloadType::SpanLinks => &SPAN_LINKS_TEMPLATE_SCHEMA,
        ArrowPayloadType::SpanEvents => &SPAN_EVENTS_TEMPLATE_SCHEMA,
        ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::LogAttrs => &ATTRS_16_TEMPLATE_SCHEMA,
        ArrowPayloadType::SpanLinkAttrs
        | ArrowPayloadType::SpanEventAttrs
        | ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => &ATTRS_32_TEMPLATE_SCHEMA,
        ArrowPayloadType::Unknown => {
            // TODO need to return error here?
            todo!()
        }
    }
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
            true,
        ),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, false),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::OBSERVED_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::SEVERITY_NUMBER, DataType::Int32, true),
        Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
        Field::new(
            consts::BODY,
            DataType::Struct(Fields::from(vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
                Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
                Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
            ])),
            true,
        ),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, false),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ])
});

static METRICS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
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
            false,
        ),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, false),
        Field::new(consts::METRIC_TYPE, DataType::UInt8, false),
        Field::new(consts::NAME, DataType::Utf8, false),
        Field::new(consts::DESCRIPTION, DataType::Utf8, false),
        Field::new(consts::UNIT, DataType::Utf8, false),
        Field::new(consts::AGGREGATION_TEMPORALITY, DataType::Int32, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
    ])
});

static NUMBERS_DP_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
        Field::new(consts::INT_VALUE, DataType::Int64, true),
        Field::new(consts::DOUBLE_VALUE, DataType::Float64, true),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ])
});

static SUMMARY_DP_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::SUMMARY_COUNT, DataType::UInt64, false),
        Field::new(consts::SUMMARY_SUM, DataType::Float64, false),
        Field::new(
            consts::SUMMARY_QUANTILE_VALUES,
            DataType::List(Arc::new(Field::new(
                "item",
                DataType::Struct(Fields::from(vec![
                    Field::new(consts::SUMMARY_QUANTILE, DataType::Float64, false),
                    Field::new(consts::SUMMARY_VALUE, DataType::Float64, false),
                ])),
                true,
            ))),
            false,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
    ])
});

static HISTOGRAM_DP_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
        Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
        Field::new(
            consts::HISTOGRAM_BUCKET_COUNTS,
            DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
            false,
        ),
        Field::new(
            consts::HISTOGRAM_EXPLICIT_BOUNDS,
            DataType::List(Arc::new(Field::new("item", DataType::Float64, true))),
            true,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
        Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
        Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
    ])
});

static EXP_HISTOGRAM_DP_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, false),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::HISTOGRAM_COUNT, DataType::UInt64, false),
        Field::new(consts::HISTOGRAM_SUM, DataType::Float64, true),
        Field::new(consts::EXP_HISTOGRAM_SCALE, DataType::Int32, false),
        Field::new(consts::EXP_HISTOGRAM_ZERO_COUNT, DataType::UInt64, false),
        Field::new(
            consts::EXP_HISTOGRAM_POSITIVE,
            DataType::Struct(Fields::from(vec![
                Field::new(consts::EXP_HISTOGRAM_OFFSET, DataType::Int32, false),
                Field::new(
                    consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                    DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                    false,
                ),
            ])),
            false,
        ),
        Field::new(
            consts::EXP_HISTOGRAM_NEGATIVE,
            DataType::Struct(Fields::from(vec![
                Field::new(consts::EXP_HISTOGRAM_OFFSET, DataType::Int32, false),
                Field::new(
                    consts::EXP_HISTOGRAM_BUCKET_COUNTS,
                    DataType::List(Arc::new(Field::new("item", DataType::UInt64, false))),
                    false,
                ),
            ])),
            false,
        ),
        Field::new(consts::FLAGS, DataType::UInt32, false),
        Field::new(consts::HISTOGRAM_MIN, DataType::Float64, true),
        Field::new(consts::HISTOGRAM_MAX, DataType::Float64, true),
    ])
});

static EXEMPLAR_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, true),
        Field::new(consts::PARENT_ID, DataType::UInt32, false),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(consts::INT_VALUE, DataType::Int64, false),
        Field::new(consts::DOUBLE_VALUE, DataType::Int64, false),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
    ])
});

static SPANS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt16, true),
        Field::new(
            consts::RESOURCE,
            DataType::Struct(RESOURCE_TEMPLATE_FIELDS.clone()),
            true,
        ),
        Field::new(
            consts::SCOPE,
            DataType::Struct(SCOPE_TEMPLATE_FIELDS.clone()),
            true,
        ),
        Field::new(consts::SCHEMA_URL, DataType::Utf8, false),
        Field::new(
            consts::START_TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new(
            consts::DURATION_TIME_UNIX_NANO,
            DataType::Duration(TimeUnit::Nanosecond),
            false,
        ),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), false),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), false),
        Field::new(consts::TRACE_STATE, DataType::Utf8, true),
        Field::new(consts::PARENT_SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::NAME, DataType::Utf8, false),
        Field::new(consts::KIND, DataType::Int32, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
        Field::new(consts::DROPPED_EVENTS_COUNT, DataType::UInt32, true),
        Field::new(consts::DROPPED_LINKS_COUNT, DataType::UInt32, true),
        Field::new(
            consts::STATUS,
            DataType::Struct(Fields::from(vec![
                Field::new(consts::STATUS_CODE, DataType::Int32, true),
                Field::new(consts::STATUS_MESSAGE, DataType::Utf8, true),
            ])),
            true,
        ),
    ])
});

static SPAN_EVENTS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, true),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(
            consts::TIME_UNIX_NANO,
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            true,
        ),
        Field::new(consts::NAME, DataType::Utf8, false),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
    ])
});

static SPAN_LINKS_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::ID, DataType::UInt32, true),
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), true),
        Field::new(consts::TRACE_STATE, DataType::Utf8, true),
        Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
    ])
});

static ATTRS_16_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ])
});

static ATTRS_32_TEMPLATE_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt32, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
        Field::new(consts::ATTRIBUTE_SER, DataType::Binary, true),
    ])
});

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{RecordBatch, RecordBatchOptions, UInt16Array};
    use pretty_assertions::assert_eq;

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
                    false,
                ),
                // TODO fill in other columns:
                // 1 - default null/values on FSL
                // 2 - nullable struct (body?)
                // 3 - keeps dicts
                // ...
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(StructArray::new(
                    Fields::from(vec![Field::new(consts::ID, DataType::UInt16, true)]),
                    vec![Arc::new(UInt16Array::from_iter(vec![
                        Some(0),
                        None,
                        Some(1),
                    ]))],
                    Some(NullBuffer::new_valid(3)),
                )),
            ],
        )
        .unwrap();

        let result =
            transform_to_known_schema(&log_attrs_record_batch, ArrowPayloadType::Logs).unwrap();

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
                    false,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(expected_scope_fields.clone()),
                    false,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 2, 3])),
                Arc::new(StructArray::new(
                    expected_resource_fields.clone(),
                    vec![
                        Arc::new(UInt16Array::new_null(3)),
                        Arc::new(StringArray::new_null(3)),
                        Arc::new(UInt32Array::new_null(3)),
                    ],
                    Some(NullBuffer::new_valid(3)),
                )),
                Arc::new(StructArray::new(
                    expected_scope_fields.clone(),
                    vec![
                        Arc::new(UInt16Array::from_iter(vec![Some(0), None, Some(1)])),
                        Arc::new(StringArray::new_null(3)),
                        Arc::new(StringArray::new_null(3)),
                        Arc::new(UInt32Array::new_null(3)),
                    ],
                    Some(NullBuffer::new_valid(3)),
                )),
            ],
        )
        .unwrap();

        assert_eq!(result, expected)
    }

    #[test]
    fn test_generate_all_null_record_batch() {
        // this is just a smoke test to ensure that all of our null/empty column generating code
        // isn't missing any types that are actually used in one of the template schemas
        for payload_type in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::MultivariateMetrics,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ] {
            let input = RecordBatch::try_new_with_options(
                Arc::new(Schema::empty()),
                vec![],
                &RecordBatchOptions::new().with_row_count(Some(1)),
            )
            .unwrap();
            _ = transform_to_known_schema(&input, payload_type).unwrap();
        }
    }
}
