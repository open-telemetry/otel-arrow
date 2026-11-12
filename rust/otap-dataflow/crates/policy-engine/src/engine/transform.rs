// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Translation and application of policy transforms to OTAP Arrow batches.
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

use std::collections::BTreeMap;

use arrow::array::RecordBatch;
use otel_arrow_dfe_pdata::otap::raw_batch_store::{
    LOGS_TYPE_MASK, RawBatchStore,
};
use otel_arrow_dfe_pdata::otap::transform::concatenate::{ConcatOptions, concatenate};
use otel_arrow_dfe_pdata::otap::transform::{
    AttributesTransform, RenameTransform, apply_attribute_transform,
};
use otel_arrow_dfe_pdata::otap::{Logs, OtapBatchStore};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use policy_rs::engine::{CompiledTransform, LogSignal, TransformOp};
use policy_rs::LogFieldSelector;

use crate::engine::policy_match::PolicyMatch;

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
            TransformOp::Rename { from, to, upsert: _ } => {
                // TODO: handle the upsert flag. Currently we always rename
                // (overwrite). Per spec, if upsert is false and the target
                // key already exists, the rename should be a no-op.
                apply_rename(batch, from, to)?;
            }
            TransformOp::Remove { .. } => {
                // TODO: implement remove transform
            }
            TransformOp::Redact { .. } => {
                // TODO: implement redact transform
            }
            TransformOp::Add { .. } => {
                // TODO: implement add transform
            }
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
    let (attr_key, payload_type) = match from {
        LogFieldSelector::LogAttribute(path) => {
            let key = path.first().map(|s| s.as_str()).unwrap_or("");
            (key, ArrowPayloadType::LogAttrs)
        }
        LogFieldSelector::ResourceAttribute(path) => {
            let key = path.first().map(|s| s.as_str()).unwrap_or("");
            (key, ArrowPayloadType::ResourceAttrs)
        }
        LogFieldSelector::ScopeAttribute(path) => {
            let key = path.first().map(|s| s.as_str()).unwrap_or("");
            (key, ArrowPayloadType::ScopeAttrs)
        }
        // Renaming simple fields is a no-op per spec.
        LogFieldSelector::Simple(_) => return Ok(()),
    };

    if attr_key.is_empty() {
        return Ok(());
    }

    let mut map = BTreeMap::new();
    let _ = map.insert(attr_key.to_string(), to.to_string());
    let rename = RenameTransform::new(map);
    let transform = AttributesTransform::default().with_rename(rename);

    let _ = apply_attribute_transform(batch, payload_type, &transform, false)?;

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
