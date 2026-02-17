// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared test helpers for the transform submodules (reindex, split, etc.).

use std::collections::HashSet;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, Int64Array, RecordBatch, StringArray, StructArray, UInt8Array,
};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Field, Schema, UInt8Type, UInt16Type,
    UInt32Type,
};

use crate::otap::transform::util::{
    IdColumnType, access_column, payload_relations, replace_column, struct_column_name,
    update_field_encoding_metadata,
};
use crate::otap::{OtapBatchStore, POSITION_LOOKUP, UNUSED_INDEX};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

/// Known ID column paths that need plain encoding metadata.
pub const ID_COLUMN_PATHS: &[&str] = &["id", "resource.id", "scope.id", "parent_id"];

// ---------------------------------------------------------------------------
// Convenience macros for constructing test batches
// ---------------------------------------------------------------------------

/// Build a `[Option<RecordBatch>; Logs::COUNT]` from payload‑type / column specs.
///
/// ```ignore
/// logs!(
///     (Logs, ("id", UInt16, vec![0u16, 1, 2])),
///     (LogAttrs, ("parent_id", UInt16, vec![0u16, 1, 2])),
/// )
/// ```
macro_rules! logs {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Logs;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::transform::testing::make_test_batch::<Logs, { Logs::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub(crate) use logs;

/// Build a `[Option<RecordBatch>; Traces::COUNT]` from payload‑type / column specs.
macro_rules! traces {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Traces;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::transform::testing::make_test_batch::<Traces, { Traces::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub(crate) use traces;

/// Build a `[Option<RecordBatch>; Metrics::COUNT]` from payload‑type / column specs.
macro_rules! metrics {
    ($(($payload:ident, $($record_batch_args:tt)*)),* $(,)?) => {
        {
            use $crate::otap::Metrics;
            use $crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

            $crate::otap::transform::testing::make_test_batch::<Metrics, { Metrics::COUNT }>(vec![
                $((
                    ArrowPayloadType::$payload,
                    $crate::record_batch!($($record_batch_args)*).unwrap(),
                ),)*
            ])
        }
    };
}
pub(crate) use metrics;

// ---------------------------------------------------------------------------
// Batch construction helpers
// ---------------------------------------------------------------------------

/// Convert `ArrowPayloadType` -> position index, panicking if unused.
pub fn payload_to_idx(payload_type: ArrowPayloadType) -> usize {
    let pos = POSITION_LOOKUP[payload_type as usize];
    assert_ne!(pos, UNUSED_INDEX);
    pos
}

/// Create a `[Option<RecordBatch>; N]` from a list of (payload_type, batch) pairs.
///
/// Each batch is "completed" (struct‑wrapped id columns, body/name filler,
/// encoding metadata) so it can be round‑tripped through OTLP.
pub fn make_test_batch<S: OtapBatchStore, const N: usize>(
    inputs: Vec<(ArrowPayloadType, RecordBatch)>,
) -> [Option<RecordBatch>; N] {
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

        result[idx] = Some(complete_batch(payload_type, batch, &all_payload_types));
    }

    result
}

/// Post‑process a test `RecordBatch` so it has the minimal columns needed for
/// a valid OTAP payload of the given type and has encoding metadata set.
pub fn complete_batch(
    payload_type: ArrowPayloadType,
    batch: RecordBatch,
    all_payload_types: &[ArrowPayloadType],
) -> RecordBatch {
    let batch = match payload_type {
        ArrowPayloadType::Logs => complete_logs_batch(batch),
        ArrowPayloadType::Spans => complete_spans_batch(batch),
        ArrowPayloadType::SpanEvents => complete_span_events_batch(batch),
        ArrowPayloadType::SpanLinks => complete_span_links_batch(batch),

        // Root metrics table
        ArrowPayloadType::UnivariateMetrics => {
            complete_metrics_batch(batch, infer_metric_type(all_payload_types))
        }

        // Attrs (adds key/type/int columns if missing)
        ArrowPayloadType::LogAttrs
        | ArrowPayloadType::SpanAttrs
        | ArrowPayloadType::MetricAttrs
        | ArrowPayloadType::ResourceAttrs
        | ArrowPayloadType::ScopeAttrs
        | ArrowPayloadType::SpanEventAttrs
        | ArrowPayloadType::SpanLinkAttrs
        | ArrowPayloadType::NumberDpAttrs
        | ArrowPayloadType::SummaryDpAttrs
        | ArrowPayloadType::HistogramDpAttrs
        | ArrowPayloadType::ExpHistogramDpAttrs
        | ArrowPayloadType::NumberDpExemplarAttrs
        | ArrowPayloadType::HistogramDpExemplarAttrs
        | ArrowPayloadType::ExpHistogramDpExemplarAttrs => complete_attrs_batch(batch),

        // Data points and exemplars: only id/parent_id needed
        ArrowPayloadType::NumberDataPoints
        | ArrowPayloadType::SummaryDataPoints
        | ArrowPayloadType::HistogramDataPoints
        | ArrowPayloadType::ExpHistogramDataPoints
        | ArrowPayloadType::NumberDpExemplars
        | ArrowPayloadType::HistogramDpExemplars
        | ArrowPayloadType::ExpHistogramDpExemplars => batch,

        _ => batch,
    };
    mark_id_columns_plain(batch)
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

fn complete_metrics_batch(batch: RecordBatch, metric_type: u8) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    wrap_struct_id_columns(&schema, &mut fields, &mut columns);

    if schema.fields.find("metric_type").is_none() {
        fields.push(Arc::new(Field::new("metric_type", DataType::UInt8, false)));
        columns.push(Arc::new(UInt8Array::from(vec![metric_type; num_rows])));
    }

    if schema.fields.find("name").is_none() {
        fields.push(Arc::new(Field::new("name", DataType::Utf8, false)));
        columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed metrics batch")
}

fn complete_logs_batch(batch: RecordBatch) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    wrap_struct_id_columns(&schema, &mut fields, &mut columns);

    if schema.fields.find("body").is_none() {
        let body_fields = vec![
            Field::new("type", DataType::UInt8, false),
            Field::new("int", DataType::Int64, true),
        ];
        let body_arrays: Vec<ArrayRef> = vec![
            Arc::new(UInt8Array::from(vec![0u8; num_rows])),
            Arc::new(Int64Array::from(vec![0i64; num_rows])),
        ];
        let body = StructArray::try_new(body_fields.clone().into(), body_arrays, None)
            .expect("Failed to create body struct");

        fields.push(Arc::new(Field::new(
            "body",
            DataType::Struct(body_fields.into()),
            true,
        )));
        columns.push(Arc::new(body));
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed logs batch")
}

fn complete_attrs_batch(batch: RecordBatch) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

    if schema.fields.find("key").is_none() {
        fields.push(Arc::new(Field::new("key", DataType::Utf8, false)));
        columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
    }

    if schema.fields.find("type").is_none() {
        fields.push(Arc::new(Field::new("type", DataType::UInt8, false)));
        columns.push(Arc::new(UInt8Array::from(vec![0u8; num_rows])));
    }

    if schema.fields.find("int").is_none() {
        fields.push(Arc::new(Field::new("int", DataType::Int64, true)));
        columns.push(Arc::new(Int64Array::from(vec![0i64; num_rows])));
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed attrs batch")
}

fn complete_spans_batch(batch: RecordBatch) -> RecordBatch {
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    wrap_struct_id_columns(&schema, &mut fields, &mut columns);

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed spans batch")
}

/// SpanEvents batch: needs a "name" column for transport decode
fn complete_span_events_batch(batch: RecordBatch) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

    if schema.fields.find("name").is_none() {
        fields.push(Arc::new(Field::new("name", DataType::Utf8, true)));
        columns.push(Arc::new(StringArray::from(vec![""; num_rows])));
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed span events batch")
}

/// SpanLinks batch: needs a "trace_id" column for transport decode
fn complete_span_links_batch(batch: RecordBatch) -> RecordBatch {
    let num_rows = batch.num_rows();
    let (schema, mut columns, _) = batch.into_parts();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();

    if schema.fields.find("trace_id").is_none() {
        fields.push(Arc::new(Field::new(
            "trace_id",
            DataType::FixedSizeBinary(16),
            true,
        )));
        let empty_bytes = vec![0u8; num_rows * 16];
        columns.push(Arc::new(
            arrow::array::FixedSizeBinaryArray::try_new(16, empty_bytes.into(), None)
                .expect("Failed to create trace_id array"),
        ));
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .expect("Failed to create completed span links batch")
}

/// Wraps flat `"resource.id"` / `"scope.id"` columns into struct columns
/// (e.g. `"resource.id" UInt16` → `"resource" Struct { "id": UInt16 }`).
pub fn wrap_struct_id_columns(
    schema: &Schema,
    fields: &mut Vec<Arc<Field>>,
    columns: &mut Vec<ArrayRef>,
) {
    for &path in ID_COLUMN_PATHS {
        let Some(struct_name) = struct_column_name(path) else {
            continue;
        };
        let Some((idx, _)) = schema.fields.find(path) else {
            continue;
        };
        let id_col = columns.remove(idx);
        let _ = fields.remove(idx);
        let id_field = Field::new("id", id_col.data_type().clone(), true);
        let struct_col = StructArray::try_new(vec![id_field.clone()].into(), vec![id_col], None)
            .expect("Failed to create struct");
        fields.push(Arc::new(Field::new(
            struct_name,
            DataType::Struct(vec![id_field].into()),
            true,
        )));
        columns.push(Arc::new(struct_col));
    }
}

/// Marks all id/parent_id columns with plain encoding metadata.
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

// ---------------------------------------------------------------------------
// Assertion / validation helpers
// ---------------------------------------------------------------------------

/// For each batch group, relation, and child type, records which child row
/// indices map to each parent by ordinal position (smallest parent ID = 0,
/// second smallest = 1, etc).  This captures structural relationships
/// independent of actual ID values, so it should be identical before and
/// after any ID‑rewriting transform.
pub fn extract_relation_fingerprints<S: OtapBatchStore, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
) -> Vec<Vec<Vec<usize>>> {
    let mut fingerprints = Vec::new();

    for group in batches.iter() {
        for &payload_type in S::allowed_payload_types() {
            let parent_idx = payload_to_idx(payload_type);
            let Some(parent_batch) = &group[parent_idx] else {
                continue;
            };

            for relation in payload_relations(payload_type).relations {
                let Some(parent_col) = access_column(
                    relation.key_col,
                    &parent_batch.schema(),
                    parent_batch.columns(),
                ) else {
                    continue;
                };
                let parent_ids = collect_row_ids(parent_col.as_ref());

                // Sort and deduplicate to get ordinal mapping
                let mut unique_sorted: Vec<u32> = parent_ids.clone();
                unique_sorted.sort();
                unique_sorted.dedup();

                for &child_type in relation.child_types {
                    let child_idx = payload_to_idx(child_type);
                    let Some(child_batch) = &group[child_idx] else {
                        continue;
                    };

                    let Some(child_col) =
                        access_column("parent_id", &child_batch.schema(), child_batch.columns())
                    else {
                        continue;
                    };
                    let child_parent_ids = collect_row_ids(child_col.as_ref());

                    // For each parent ordinal, record which child row indices reference it
                    let mut ordinal_to_children: Vec<Vec<usize>> =
                        vec![vec![]; unique_sorted.len()];
                    for (child_row, &child_pid) in child_parent_ids.iter().enumerate() {
                        if let Ok(ordinal) = unique_sorted.binary_search(&child_pid) {
                            ordinal_to_children[ordinal].push(child_row);
                        }
                    }

                    fingerprints.push(ordinal_to_children);
                }
            }
        }
    }

    fingerprints
}

/// Validates that no ID column has overlapping values across batch groups.
/// Uses `payload_relations` to discover all ID columns that should be unique.
pub fn assert_no_id_overlaps<S: OtapBatchStore, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
) {
    for &payload_type in S::allowed_payload_types() {
        let idx = payload_to_idx(payload_type);

        for relation in payload_relations(payload_type).relations {
            let mut seen = HashSet::new();

            for group in batches.iter() {
                let Some(batch) = &group[idx] else {
                    continue;
                };

                let Some(col) = access_column(relation.key_col, &batch.schema(), batch.columns())
                else {
                    continue;
                };

                let ids = collect_row_ids(col.as_ref());
                for id in ids {
                    assert!(
                        seen.insert(id),
                        "Overlapping ID in column '{}'",
                        relation.key_col,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ID extraction helpers
// ---------------------------------------------------------------------------

/// Collects per-row logical ID values as `u32`, resolving dictionary encoding.
/// Handles plain UInt16, UInt32, and dictionary-encoded columns.
pub fn collect_row_ids(col: &dyn Array) -> Vec<u32> {
    match col.data_type() {
        DataType::UInt16 => col
            .as_primitive::<UInt16Type>()
            .values()
            .iter()
            .map(|&v| v as u32)
            .collect(),
        DataType::UInt32 => col
            .as_primitive::<UInt32Type>()
            .values()
            .iter()
            .copied()
            .collect(),
        DataType::Dictionary(key_type, _) => match key_type.as_ref() {
            DataType::UInt8 => collect_dict_row_ids::<UInt8Type>(col),
            DataType::UInt16 => collect_dict_row_ids::<UInt16Type>(col),
            _ => unreachable!("Unsupported dictionary key type"),
        },
        _ => unreachable!("Unsupported column type: {:?}", col.data_type()),
    }
}

fn collect_dict_row_ids<K>(col: &dyn Array) -> Vec<u32>
where
    K: ArrowDictionaryKeyType,
    K::Native: ArrowNativeType,
{
    let dict = col.as_dictionary::<K>();
    let values = dict.values();
    match values.data_type() {
        DataType::UInt16 => {
            let vals = values.as_primitive::<UInt16Type>();
            dict.keys()
                .values()
                .iter()
                .map(|k: &K::Native| vals.value(k.as_usize()) as u32)
                .collect()
        }
        DataType::UInt32 => {
            let vals = values.as_primitive::<UInt32Type>();
            dict.keys()
                .values()
                .iter()
                .map(|k: &K::Native| vals.value(k.as_usize()))
                .collect()
        }
        _ => unreachable!("Unsupported dictionary value type"),
    }
}

/// Searches all allowed payload types in a batch store to find if
/// `child_type` appears as a child in any relation.  If found, returns the
/// parent's primary id column size (which determines the parent_id column
/// type).  Returns `None` for root types that have no parent.
pub fn find_parent_id_size<S: OtapBatchStore>(
    child_type: ArrowPayloadType,
) -> Option<IdColumnType> {
    for &pt in S::allowed_payload_types() {
        let info = payload_relations(pt);
        let Some(primary_id) = info.primary_id else {
            continue;
        };
        for relation in info.relations {
            if relation.child_types.contains(&child_type) {
                return Some(primary_id.size);
            }
        }
    }
    None
}

/// Utility for pretty printing a bunch of otap batches, nice for debugging
#[allow(dead_code)]
pub fn pretty_print_otap_batches<const N: usize>(batches: &[[Option<RecordBatch>; N]]) {
    for (idx, b) in batches.iter().enumerate() {
        use arrow::util::pretty;
        eprintln!("-----Batch #{}------", idx);
        for rb in b.iter().flatten().cloned() {
            eprintln!("{}", pretty::pretty_format_batches(&[rb]).unwrap());
        }
        eprintln!("-----End Batch #{}------", idx);
    }
}
