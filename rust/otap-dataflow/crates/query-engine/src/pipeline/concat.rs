// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Module contains utilities for concatenating OTAP batches.
//!
//! This is used by pipeline stages that split/duplicate the batch, evaluate nested pipelines
//! on it, and concatenate the results to produce an output.

use arrow::array::RecordBatch;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::raw_batch_store::{
    LOGS_TYPE_MASK, METRICS_TYPE_MASK, POSITION_LOOKUP, RawBatchStore, TRACES_TYPE_MASK,
};
use otel_arrow_dfe_pdata::otap::transform::concatenate::{ConcatOptions, concatenate};
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::{Error, Result};
use crate::pipeline::expr::types::MetricDataPointType;

pub(crate) fn concat_generic<T, const TYPE_MASK: u64, const COUNT: usize>(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords>
where
    T: OtapBatchStore<BatchArray = [Option<RecordBatch>; COUNT]>
        + TryFrom<RawBatchStore<TYPE_MASK, COUNT>, Error = otel_arrow_dfe_pdata::error::Error>
        + TryFrom<OtapArrowRecords, Error = otel_arrow_dfe_pdata::error::Error>,
    OtapArrowRecords: From<T>,
{
    let mut batches = Vec::new();
    for branch_result in branch_results.drain(..) {
        let batch_store: T = branch_result.try_into()?;
        batches.push(batch_store.into_batches())
    }

    let concatenated_batches = concatenate(&mut batches, opts)?;
    let raw_store = RawBatchStore::<TYPE_MASK, COUNT>::from_batches(concatenated_batches);
    let result_store = T::try_from(raw_store)?;

    Ok(OtapArrowRecords::from(result_store))
}

pub(crate) fn concatenate_logs(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Logs, { LOGS_TYPE_MASK }, { Logs::COUNT }>(branch_results, opts)
}

pub(crate) fn concatenate_metrics(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Metrics, { METRICS_TYPE_MASK }, { Metrics::COUNT }>(branch_results, opts)
}

pub(crate) fn concatenate_traces(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Traces, { TRACES_TYPE_MASK }, { Traces::COUNT }>(branch_results, opts)
}

/// Payload types in a Metrics OTAP batch that are NOT data-point sub-batches.
///
/// These are stripped before concatenating data-point branches and restored from the original
/// OTAP batch afterwards.
const NON_DATA_POINT_PAYLOAD_TYPES: [ArrowPayloadType; 5] = [
    ArrowPayloadType::ResourceAttrs,
    ArrowPayloadType::ScopeAttrs,
    ArrowPayloadType::UnivariateMetrics,
    ArrowPayloadType::MultivariateMetrics,
    ArrowPayloadType::MetricAttrs,
];

/// Concatenate only the data-point sub-batches from multiple branch results back into
/// `otap_batch`.
///
/// Each branch result in `branch_results` is a metrics OTAP batch that was produced by
/// filtering the data points from the original `otap_batch` and running branch pipeline stages.
/// The non-data-point payloads (root metrics, resource/scope attrs, metric attrs) in each branch
/// result are ignored -- they are preserved from the original `otap_batch`.
///
/// The data-point sub-batches (data points, dp attrs, exemplars, exemplar attrs) from all
/// branches are concatenated using the OTAP concatenation infrastructure (which handles
/// dictionary coalescing and schema unification) with `preserve_ids` since the IDs are disjoint
/// pieces of the same original batch.
///
/// After concatenation, the resulting data-point sub-batches replace those in `otap_batch`.
pub(crate) fn concatenate_metric_data_point_batches(
    branch_results: &mut Vec<OtapArrowRecords>,
    otap_batch: &mut OtapArrowRecords,
) -> Result<()> {
    // Strip non-data-point payloads from each branch result so concatenation only
    // processes data-point sub-batches. The concatenate infrastructure skips None slots.
    for branch_result in branch_results.iter_mut() {
        for payload_type in NON_DATA_POINT_PAYLOAD_TYPES {
            _ = branch_result.remove(payload_type);
        }
    }

    let concatenated = concatenate_metrics(branch_results, ConcatOptions::preserve_ids())?;

    // Move concatenated data-point sub-batches into the original otap_batch,
    // replacing the old data-point payloads.
    for dp_type in MetricDataPointType::all() {
        for payload_type in dp_type.associated_payload_types() {
            // Remove the old payload from otap_batch regardless
            _ = otap_batch.remove(payload_type);
            // Set the concatenated payload if present
            if let Some(batch) = concatenated.get(payload_type) {
                otap_batch.set(payload_type, batch.clone())?;
            }
        }
    }

    Ok(())
}

pub(crate) fn concatenate_attrs_record_batches(
    branch_results: &mut Vec<RecordBatch>,
) -> Result<RecordBatch> {
    // to call schema aware `concatenate` on just the attributes record batch, we stick it in
    // a Logs OTAP batch and treat it as log attributes. This is a bit of a hack until we have
    // a better top-level interface for calling concatenate.
    //
    // The attribute batches are disjoint pieces of the same original batch so
    // their parent IDs must be preserved as-is.

    let mut otap_batches = branch_results
        .drain(..)
        .map(|attrs_record_batch| {
            let mut logs_record_batches = Logs::default();
            logs_record_batches.set(ArrowPayloadType::LogAttrs, attrs_record_batch)?;
            Ok(OtapArrowRecords::from(logs_record_batches))
        })
        .collect::<Result<Vec<_>>>()?;
    let concatenated_logs = concatenate_logs(&mut otap_batches, ConcatOptions::preserve_ids())?;
    let mut concatenated_logs_batches = Logs::try_from(concatenated_logs)?.into_batches();
    let concatenated_attrs_batch = concatenated_logs_batches
        [POSITION_LOOKUP[ArrowPayloadType::LogAttrs as usize]]
        .take()
        .ok_or_else(|| Error::ExecutionError {
            cause: "expected concatenate to produce non 'None' batch".into(),
        })?;

    Ok(concatenated_attrs_batch)
}
