// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Testing utilities for constructing OTAP batches.
//!
//! Provides macros and helpers for building test batches with minimal boilerplate.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float64Array, Int32Array,
    Int64Array, ListArray, RecordBatch, StringArray, StructArray, UInt8Array, UInt16Array,
    UInt32Array, UInt64Array,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};

use crate::otap::OtapBatchStore;
use crate::otap::transform::util::{
    access_column, payload_to_idx, replace_column, update_field_encoding_metadata,
};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::schema::{DataType as OtapDataType, SimpleType};

/// Known ID column paths that need plain encoding metadata.
pub const ID_COLUMN_PATHS: &[&str] = &["id", "resource.id", "scope.id", "parent_id"];

/// Build a [`Logs`] store from payload‑type / column specs.
///
/// Missing required columns are automatically filled in using the schema spec.
/// Use `.into()` to convert to [`OtapArrowRecords`], or `.into_batches()` to
/// get the raw `[Option<RecordBatch>; Logs::COUNT]` array.
///
/// ```ignore
/// let logs: Logs = logs!(
///     (Logs, ("id", UInt16, vec![0u16, 1, 2])),
///     (LogAttrs, ("parent_id", UInt16, vec![0u16, 1, 2])),
/// );
/// let records: OtapArrowRecords = logs.into();
/// ```
#[macro_export]
macro_rules! logs {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Logs;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::testing::make_test_batch::<Logs, { Logs::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub use logs;

/// Build a [`Traces`] store from payload‑type / column specs.
///
/// Missing required columns are automatically filled in using the schema spec.
/// Use `.into()` to convert to [`OtapArrowRecords`], or `.into_batches()` to
/// get the raw `[Option<RecordBatch>; Traces::COUNT]` array.
#[macro_export]
macro_rules! traces {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Traces;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::testing::make_test_batch::<Traces, { Traces::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub use traces;

/// Build a [`Metrics`] store from payload‑type / column specs.
///
/// Missing required columns are automatically filled in using the schema spec.
/// Use `.into()` to convert to [`OtapArrowRecords`], or `.into_batches()` to
/// get the raw `[Option<RecordBatch>; Metrics::COUNT]` array.
#[macro_export]
macro_rules! metrics {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Metrics;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::testing::make_test_batch::<Metrics, { Metrics::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub use metrics;

/// Pre-process a test batch before spec-driven completion.
///
/// 1. Groups dotted column names (e.g., "resource.id", "resource.schema_url") by prefix
///    and converts them into StructArrays.
/// 2. For UnivariateMetrics: adds "metric_type" column if missing, inferred from
///    sibling payload types.
fn normalize_test_batch(
    payload_type: ArrowPayloadType,
    batch: RecordBatch,
    all_payload_types: &[ArrowPayloadType],
) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

    // Step 1: Group dotted columns into structs
    let mut struct_groups: HashMap<String, Vec<(String, Arc<Field>, ArrayRef)>> = HashMap::new();
    let mut indices_to_remove = Vec::new();

    for (idx, field) in schema.fields().iter().enumerate() {
        let name = field.name().as_str();
        if let Some(dot_pos) = name.find('.') {
            let (prefix, suffix) = name.split_at(dot_pos);
            let suffix = &suffix[1..]; // skip the '.'
            struct_groups.entry(prefix.to_string()).or_default().push((
                suffix.to_string(),
                field.clone(),
                columns[idx].clone(),
            ));
            indices_to_remove.push(idx);
        }
    }

    // Remove flat dotted columns (in reverse order to preserve indices)
    for &idx in indices_to_remove.iter().rev() {
        let _ = fields.remove(idx);
        let _ = columns.remove(idx);
    }

    // Add struct columns
    for (struct_name, mut sub_fields_data) in struct_groups {
        // Sort by sub-field name for deterministic ordering
        sub_fields_data.sort_by(|a, b| a.0.cmp(&b.0));

        let sub_fields: Vec<Field> = sub_fields_data
            .iter()
            .map(|(name, field, _)| {
                Field::new(name, field.data_type().clone(), field.is_nullable())
            })
            .collect();
        let sub_arrays: Vec<ArrayRef> = sub_fields_data
            .iter()
            .map(|(_, _, arr)| arr.clone())
            .collect();

        let struct_array = StructArray::new(sub_fields.clone().into(), sub_arrays, None);
        fields.push(Arc::new(Field::new(
            &struct_name,
            DataType::Struct(sub_fields.into()),
            true,
        )));
        columns.push(Arc::new(struct_array));
    }

    let mut batch = RecordBatch::try_new(Arc::new(Schema::new(fields.clone())), columns)
        .expect("Failed to create normalized batch");

    // For UnivariateMetrics, add metric_type if missing. We can pick based on
    // the data points tables if there are any.
    if payload_type == ArrowPayloadType::UnivariateMetrics
        && batch.schema().column_with_name("metric_type").is_none()
    {
        let metric_type = infer_metric_type(all_payload_types);
        let (schema, mut columns, _) = batch.into_parts();
        let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
        fields.push(Arc::new(Field::new("metric_type", DataType::UInt8, false)));
        columns.push(Arc::new(UInt8Array::from(vec![metric_type; num_rows])));
        batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
            .expect("Failed to add metric_type column");
    }

    batch
}

/// Create an `OtapBatchStore` (`Logs`, `Metrics`, or `Traces`) from payload-type / column specs.
///
/// Each batch is normalized (dotted columns -> structs), completed (missing columns
/// filled from spec), and marked with encoding metadata. For metrics, missing data
/// point tables are auto-generated based on metric_type.
#[must_use]
pub fn make_test_batch<S: OtapBatchStore, const N: usize>(
    inputs: Vec<(ArrowPayloadType, RecordBatch)>,
) -> S {
    let allowed = S::allowed_payload_types();
    let mut result: [Option<RecordBatch>; N] = std::array::from_fn(|_| None);
    let all_payload_types: Vec<ArrowPayloadType> = inputs.iter().map(|(pt, _)| *pt).collect();

    for (payload_type, batch) in inputs {
        assert!(
            allowed.contains(&payload_type),
            "Payload type {:?} is not allowed for this store",
            payload_type
        );

        let idx = payload_to_idx(payload_type);
        assert!(
            result[idx].is_none(),
            "Duplicate payload type {:?}",
            payload_type
        );

        let batch = normalize_test_batch(payload_type, batch, &all_payload_types);
        let batch = complete_batch(payload_type, batch);
        let batch = mark_id_columns_plain(batch);

        result[idx] = Some(batch);
    }

    // Auto-generate missing data point tables for metrics
    generate_missing_dp_tables(&mut result);

    // Move the completed batches into a new store
    let mut store = S::new();
    let batches_mut = store.batches_mut();
    for (idx, batch) in result.into_iter().enumerate() {
        batches_mut[idx] = batch;
    }
    store
}

/// Fill in missing columns for a test batch using the OTAP schema spec.
///
/// For each field defined in the schema that's not present in the batch,
/// adds a default-valued column of the correct type.
fn complete_batch(payload_type: ArrowPayloadType, batch: RecordBatch) -> RecordBatch {
    let num_rows = batch.num_rows();
    let spec = crate::schema::payloads::get(payload_type);
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

    // For each field in the spec, if it's missing from the batch, add a default column
    for spec_field in spec.fields() {
        if schema.column_with_name(spec_field.name).is_none() {
            let (arrow_dt, default_arr) = default_array(&spec_field.data_type, num_rows);
            fields.push(Arc::new(Field::new(
                spec_field.name,
                arrow_dt,
                !spec_field.required,
            )));
            columns.push(default_arr);
        }
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed batch")
}

/// Create a default-valued Arrow array for the given OTAP data type with `num_rows` rows.
///
/// - `Simple` types → concrete zero-valued arrays
/// - `Dictionary` types → produce native (non-dictionary) form
/// - `Struct` types → recursively build all sub-fields
/// - `List` types → create `num_rows` empty lists
fn default_array(data_type: &OtapDataType, num_rows: usize) -> (DataType, ArrayRef) {
    match data_type {
        OtapDataType::Simple(s) | OtapDataType::Dictionary { value_type: s, .. } => {
            simple_type_array(s, num_rows)
        }
        OtapDataType::Struct(sub_schema) => {
            let mut sub_fields = Vec::with_capacity(sub_schema.fields().len());
            let mut sub_arrays: Vec<ArrayRef> = Vec::with_capacity(sub_schema.fields().len());

            for field in sub_schema.fields() {
                let (arrow_dt, array) = default_array(&field.data_type, num_rows);
                sub_fields.push(Field::new(field.name, arrow_dt, !field.required));
                sub_arrays.push(array);
            }

            let struct_array = StructArray::new(sub_fields.clone().into(), sub_arrays, None);
            (DataType::Struct(sub_fields.into()), Arc::new(struct_array))
        }
        OtapDataType::List(inner) => {
            // Create num_rows empty lists
            let (item_dt, _) = default_array(inner, 0);
            let item_field = Arc::new(Field::new("item", item_dt.clone(), true));
            let offsets = OffsetBuffer::from_lengths(vec![0; num_rows]);
            let values = arrow::array::new_null_array(&item_dt, 0);
            let list_array = ListArray::new(item_field.clone(), offsets, values, None);
            (DataType::List(item_field), Arc::new(list_array))
        }
    }
}

/// Create a concrete Arrow array for a `SimpleType` with `num_rows` zero/default values.
fn simple_type_array(simple_type: &SimpleType, num_rows: usize) -> (DataType, ArrayRef) {
    match simple_type {
        SimpleType::Boolean => (
            DataType::Boolean,
            Arc::new(BooleanArray::from(vec![false; num_rows])),
        ),
        SimpleType::UInt8 => (
            DataType::UInt8,
            Arc::new(UInt8Array::from(vec![0u8; num_rows])),
        ),
        SimpleType::UInt16 => (
            DataType::UInt16,
            Arc::new(UInt16Array::from(vec![0u16; num_rows])),
        ),
        SimpleType::UInt32 => (
            DataType::UInt32,
            Arc::new(UInt32Array::from(vec![0u32; num_rows])),
        ),
        SimpleType::UInt64 => (
            DataType::UInt64,
            Arc::new(UInt64Array::from(vec![0u64; num_rows])),
        ),
        SimpleType::Int32 => (
            DataType::Int32,
            Arc::new(Int32Array::from(vec![0i32; num_rows])),
        ),
        SimpleType::Int64 => (
            DataType::Int64,
            Arc::new(Int64Array::from(vec![0i64; num_rows])),
        ),
        SimpleType::Float64 => (
            DataType::Float64,
            Arc::new(Float64Array::from(vec![0.0; num_rows])),
        ),
        SimpleType::Utf8 => (
            DataType::Utf8,
            Arc::new(StringArray::from(vec![""; num_rows])),
        ),
        SimpleType::Binary => (
            DataType::Binary,
            Arc::new(BinaryArray::from(vec![b"" as &[u8]; num_rows])),
        ),
        SimpleType::FixedSizeBinary(size) => {
            let byte_width = *size as usize;
            let values = vec![0u8; num_rows * byte_width];
            let array = FixedSizeBinaryArray::try_new(*size, values.into(), None)
                .expect("Failed to create FixedSizeBinaryArray");
            (DataType::FixedSizeBinary(*size), Arc::new(array))
        }
        SimpleType::TimestampNanosecond => (
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            Arc::new(arrow::array::TimestampNanosecondArray::from(vec![
                0i64;
                num_rows
            ])),
        ),
        SimpleType::DurationNanosecond => (
            DataType::Duration(TimeUnit::Nanosecond),
            Arc::new(arrow::array::DurationNanosecondArray::from(vec![
                0i64;
                num_rows
            ])),
        ),
    }
}

fn infer_metric_type(payload_types: &[ArrowPayloadType]) -> u8 {
    for pt in payload_types {
        match pt {
            ArrowPayloadType::HistogramDataPoints
            | ArrowPayloadType::HistogramDpAttrs
            | ArrowPayloadType::HistogramDpExemplars
            | ArrowPayloadType::HistogramDpExemplarAttrs => return 3,
            ArrowPayloadType::ExpHistogramDataPoints
            | ArrowPayloadType::ExpHistogramDpAttrs
            | ArrowPayloadType::ExpHistogramDpExemplars
            | ArrowPayloadType::ExpHistogramDpExemplarAttrs => return 4,
            ArrowPayloadType::SummaryDataPoints | ArrowPayloadType::SummaryDpAttrs => return 5,
            ArrowPayloadType::NumberDataPoints
            | ArrowPayloadType::NumberDpAttrs
            | ArrowPayloadType::NumberDpExemplars
            | ArrowPayloadType::NumberDpExemplarAttrs => return 1,
            _ => {}
        }
    }
    1 // Default: Gauge
}

/// Map metric_type value to corresponding data point payload type.
fn metric_type_to_dp_payload(metric_type: u8) -> ArrowPayloadType {
    match metric_type {
        1 | 2 => ArrowPayloadType::NumberDataPoints, // Gauge or Sum
        3 => ArrowPayloadType::HistogramDataPoints,
        4 => ArrowPayloadType::ExpHistogramDataPoints,
        5 => ArrowPayloadType::SummaryDataPoints,
        _ => panic!("Invalid metric_type value: {}", metric_type),
    }
}

/// Auto-generate missing data point tables for metrics based on metric_type column.
///
/// For each distinct metric type value in UnivariateMetrics, if the corresponding
/// data point table is not already present, creates a minimal table with parent_id
/// values matching the metric IDs.
fn generate_missing_dp_tables<const N: usize>(result: &mut [Option<RecordBatch>; N]) {
    // Only run for Metrics signal type (Logs/Traces don't have UnivariateMetrics)
    // Check if the array size matches Metrics::COUNT
    use crate::otap::Metrics;
    if N != Metrics::COUNT {
        return; // Not a Metrics batch array
    }

    // Find UnivariateMetrics batch
    let metrics_idx = payload_to_idx(ArrowPayloadType::UnivariateMetrics);
    let Some(metrics_batch) = &result[metrics_idx] else {
        return; // No metrics, nothing to do
    };

    // Get metric_type and id columns
    let Some(_) = metrics_batch.column_by_name("metric_type") else {
        return; // No metric_type column, nothing to auto-generate
    };

    // Get metric_type and id columns
    let Some(metric_type_col) = metrics_batch.column_by_name("metric_type") else {
        panic!("UnivariateMetrics batch missing metric_type column");
    };
    let metric_type_array = metric_type_col
        .as_any()
        .downcast_ref::<UInt8Array>()
        .expect("metric_type column should be UInt8Array");

    let Some(id_col) = metrics_batch.column_by_name("id") else {
        panic!("UnivariateMetrics batch missing id column");
    };
    let id_array = id_col
        .as_any()
        .downcast_ref::<UInt16Array>()
        .expect("id column should be UInt16Array");

    // Group metric IDs by metric_type
    let mut type_to_ids: HashMap<u8, Vec<u16>> = HashMap::new();
    for i in 0..metrics_batch.num_rows() {
        let mt = metric_type_array.value(i);
        let id = id_array.value(i);
        type_to_ids.entry(mt).or_default().push(id);
    }

    // For each distinct metric type, generate data point table if missing
    for (mt, parent_ids) in type_to_ids {
        let dp_payload_type = metric_type_to_dp_payload(mt);
        let dp_idx = payload_to_idx(dp_payload_type);

        if result[dp_idx].is_some() {
            continue; // User already provided this table
        }

        // Create minimal data point batch with parent_id and id
        let num_rows = parent_ids.len();
        let ids: Vec<u32> = (0..num_rows as u32).collect();
        let schema = Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::UInt16, true),
            Field::new("id", DataType::UInt32, false),
        ]));
        let columns: Vec<ArrayRef> = vec![
            Arc::new(UInt16Array::from(parent_ids)),
            Arc::new(UInt32Array::from(ids)),
        ];
        let batch = RecordBatch::try_new(schema, columns)
            .expect("Failed to create minimal data point batch");

        // Complete and mark as plain
        let batch = complete_batch(dp_payload_type, batch);
        let batch = mark_id_columns_plain(batch);

        result[dp_idx] = Some(batch);
    }
}

/// Marks all id/parent_id columns with plain encoding metadata.
#[must_use]
pub fn mark_id_columns_plain(batch: RecordBatch) -> RecordBatch {
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields = schema.fields.to_vec();

    for &path in ID_COLUMN_PATHS {
        if let Some(col) = access_column(path, &schema, &columns) {
            replace_column(path, None, &schema, &mut columns, col);
            update_field_encoding_metadata(path, None, &mut fields);
        }
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to mark id columns as plain")
}
