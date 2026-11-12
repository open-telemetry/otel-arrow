// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Translation and application of policy transforms to OTAP Arrow batches.
//!
//! # Signal type generalization (TODO)
//!
//! All transform functions in this module are currently hardcoded for
//! `LogSignal` / `LogFieldSelector`. To support metrics and traces:
//! - `classify_field` needs variants for `MetricFieldSelector` and
//!   `TraceFieldSelector` (mapping to their respective attribute payload
//!   types like `MetricAttrs`, `SpanAttrs`, `NumberDpAttrs`, etc.).
//! - `apply_policy_transforms` needs to be generic over `S: Signal` or
//!   dispatched per signal type.
//! - `concat_log_batches` needs metric and trace equivalents (or a single
//!   generic concat that dispatches on the `OtapArrowRecords` variant).
//! - The `simple_field_name` mapping in `translate.rs` needs equivalents
//!   for metric and trace root fields.
//!
//! # Transform optimization (TODO)
//!
//! When a policy has multiple transforms of the same kind (e.g. several
//! renames, or several removes), they should be combined into a single
//! `AttributesTransform` call rather than applied one at a time. For example:
//! - Multiple renames -> one `RenameTransform` with all entries
//! - Multiple removes -> one `DeleteTransform` with all keys
//! - Multiple redacts on attributes -> one `AssignPipelineStage` with all
//!   assignments batched together
//!
//! Similarly, a remove followed by a rename (within the same policy) could
//! be combined into a single `AttributesTransform` with both a `delete` and
//! a `rename` field set, since `apply_attribute_transform` handles both in
//! one pass.

use std::collections::{BTreeMap, BTreeSet};

use arrow::array::RecordBatch;
use otel_arrow_dfe_pdata::otap::raw_batch_store::{LOGS_TYPE_MASK, RawBatchStore};
use otel_arrow_dfe_pdata::otap::transform::concatenate::{ConcatOptions, concatenate};
use otel_arrow_dfe_pdata::otap::transform::{
    AttributesTransform, DeleteTransform, RenameTransform, apply_attribute_transform,
};
use otel_arrow_dfe_pdata::otap::{Logs, OtapBatchStore};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use policy_rs::engine::{CompiledTransform, LogSignal, TransformOp};
use policy_rs::LogFieldSelector;
use policy_rs::proto::tero::policy::v1::LogField;

use crate::engine::translate::simple_field_name;

/// Apply all transform operations from a compiled policy transform to an
/// OTAP batch.
///
/// Operations are applied in the spec-mandated order: remove, redact,
/// rename, add. The `CompiledTransform` already stores ops in this order.
pub(crate) fn apply_policy_transforms(
    batch: &mut OtapArrowRecords,
    transform: &CompiledTransform<LogSignal>,
) -> Result<(), otel_arrow_dfe_pdata::error::Error> {
    for op in &transform.ops {
        match op {
            TransformOp::Remove { field } => {
                apply_remove(batch, field)?;
            }
            TransformOp::Redact { .. } => {
                // TODO: implement redact transform
            }
            TransformOp::Rename { from, to, upsert: _ } => {
                // TODO: handle the upsert flag. Currently we always rename
                // (overwrite). Per spec, if upsert is false and the target
                // key already exists, the rename should be a no-op.
                apply_rename(batch, from, to)?;
            }
            TransformOp::Add { .. } => {
                // TODO: implement add transform
            }
        }
    }
    Ok(())
}

/// Classify a `LogFieldSelector` as either an attribute operation (with key
/// and payload type) or a simple field operation (with column name).
enum FieldTarget<'a> {
    /// An attribute field: the key to match and the Arrow payload type
    /// for that attribute batch.
    Attribute(&'a str, ArrowPayloadType),
    /// A simple root column field (body, severity_text, etc.).
    SimpleField(&'static str),
}

/// Map a `LogFieldSelector` to a `FieldTarget`.
///
/// Returns `None` if the field selector uses a nested attribute path
/// (path length > 1), which is not yet supported for transforms.
///
/// TODO: support nested attribute paths (e.g., `["http", "request", "method"]`).
/// This requires operating on serialized map/slice values within the
/// attribute's value column, not just matching on the top-level key.
fn classify_field(field: &LogFieldSelector) -> Option<FieldTarget<'_>> {
    match field {
        LogFieldSelector::LogAttribute(path) => {
            if path.len() != 1 {
                // TODO: nested attribute paths not yet supported for transforms
                return None;
            }
            let key = path.first().map(|s| s.as_str())?;
            Some(FieldTarget::Attribute(key, ArrowPayloadType::LogAttrs))
        }
        LogFieldSelector::ResourceAttribute(path) => {
            if path.len() != 1 {
                return None;
            }
            let key = path.first().map(|s| s.as_str())?;
            Some(FieldTarget::Attribute(key, ArrowPayloadType::ResourceAttrs))
        }
        LogFieldSelector::ScopeAttribute(path) => {
            if path.len() != 1 {
                return None;
            }
            let key = path.first().map(|s| s.as_str())?;
            Some(FieldTarget::Attribute(key, ArrowPayloadType::ScopeAttrs))
        }
        LogFieldSelector::Simple(log_field) => {
            Some(FieldTarget::SimpleField(simple_field_name(log_field)))
        }
    }
}

/// Apply a single remove operation to an OTAP batch.
///
/// For attribute fields: uses `DeleteTransform` to remove matching rows
/// from the attribute batch.
///
/// For simple fields: removes the column from the root record batch
/// (following the same approach as `AssignPipelineStage::assign_null_record_column`).
/// If the field does not exist, this is a no-op.
fn apply_remove(
    batch: &mut OtapArrowRecords,
    field: &LogFieldSelector,
) -> Result<(), otel_arrow_dfe_pdata::error::Error> {
    let Some(target) = classify_field(field) else {
        return Ok(());
    };

    match target {
        FieldTarget::Attribute(key, payload_type) => {
            let mut set = BTreeSet::new();
            let _ = set.insert(key.to_string());
            let delete = DeleteTransform::new(set);
            let transform = AttributesTransform::default().with_delete(delete);
            let _ = apply_attribute_transform(batch, payload_type, &transform, false)?;
        }
        FieldTarget::SimpleField(column_name) => {
            let root_payload_type = batch.root_payload_type();
            let Some(root_batch) = batch.root_record_batch() else {
                return Ok(());
            };

            let schema = root_batch.schema_ref();
            if let Some((column_index, field)) = schema.fields().find(column_name) {
                if field.is_nullable() {
                    let mut new_rb = root_batch.clone();
                    let _ = new_rb.remove_column(column_index);
                    batch.set(root_payload_type, new_rb)?;
                }
                // Non-nullable fields cannot be removed -- no-op.
            }
            // Column not found -- no-op per spec.
        }
    }

    Ok(())
}

/// Apply a single attribute rename to an OTAP batch.
///
/// Renaming simple fields (body, severity_text, etc.) is a no-op per spec
/// and the reference implementations (Go, Zig).
fn apply_rename(
    batch: &mut OtapArrowRecords,
    from: &LogFieldSelector,
    to: &str,
) -> Result<(), otel_arrow_dfe_pdata::error::Error> {
    let Some(target) = classify_field(from) else {
        return Ok(());
    };

    match target {
        FieldTarget::Attribute(key, payload_type) => {
            let mut map = BTreeMap::new();
            let _ = map.insert(key.to_string(), to.to_string());
            let rename = RenameTransform::new(map);
            let transform = AttributesTransform::default().with_rename(rename);
            let _ = apply_attribute_transform(batch, payload_type, &transform, false)?;
        }
        // Renaming simple fields is a no-op per spec.
        FieldTarget::SimpleField(_) => {}
    }

    Ok(())
}

/// Concatenate two OTAP log batches that are disjoint pieces of the same
/// original batch. IDs are preserved (not reindexed).
///
/// TODO: generalize to metrics and traces signal types.
pub(crate) fn concat_log_batches(
    a: OtapArrowRecords,
    b: OtapArrowRecords,
) -> Result<OtapArrowRecords, otel_arrow_dfe_pdata::error::Error> {
    let a_logs: Logs = a.try_into()?;
    let b_logs: Logs = b.try_into()?;

    let mut batches = vec![a_logs.into_batches(), b_logs.into_batches()];

    let concatenated = concatenate(&mut batches, ConcatOptions::preserve_ids())?;
    let raw_store = RawBatchStore::<LOGS_TYPE_MASK, { Logs::COUNT }>::from_batches(concatenated);
    let result: Logs = raw_store.try_into()?;

    Ok(OtapArrowRecords::from(result))
}
