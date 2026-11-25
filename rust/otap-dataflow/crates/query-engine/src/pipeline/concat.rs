// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Module contains utilities for concatenating OTAP batches.
//!
//! This is used by pipeline stages that split/duplicate the batch, evaluate nested pipelines
//! on it, and concatenate the results to produce an output.

use arrow::array::{BooleanArray, RecordBatch};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::raw_batch_store::{
    LOGS_TYPE_MASK, METRICS_TYPE_MASK, POSITION_LOOKUP, RawBatchStore, TRACES_TYPE_MASK,
};
use otel_arrow_dfe_pdata::otap::transform::concatenate::{
    ConcatOptions, concatenate_with_selections,
};
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::{Error, Result};

/// Row masks for one branch result, one per payload slot in batch store order
/// (see [`POSITION_LOOKUP`]). A `None` slot drops that payload.
pub(crate) type PayloadSelections = Vec<Option<BooleanArray>>;

fn concat_generic<T, const TYPE_MASK: u64, const COUNT: usize>(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
    selections: Option<Vec<Option<PayloadSelections>>>,
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

    let masks = selections
        .map(|selections| {
            selections
                .into_iter()
                .map(|entry| entry.map(positional_masks::<COUNT>).transpose())
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;
    let concatenated_batches = concatenate_with_selections(&mut batches, opts, masks.as_deref())?;
    let raw_store = RawBatchStore::<TYPE_MASK, COUNT>::from_batches(concatenated_batches);
    let result_store = T::try_from(raw_store)?;

    Ok(OtapArrowRecords::from(result_store))
}

/// Convert one input's slot-ordered masks into the batch store's array.
fn positional_masks<const COUNT: usize>(
    masks: PayloadSelections,
) -> Result<[Option<BooleanArray>; COUNT]> {
    masks
        .try_into()
        .map_err(|masks: PayloadSelections| Error::ExecutionError {
            cause: format!(
                "expected {COUNT} payload selections for this signal, got {}",
                masks.len()
            ),
        })
}

pub(crate) fn concatenate_logs(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Logs, { LOGS_TYPE_MASK }, { Logs::COUNT }>(branch_results, opts, None)
}

pub(crate) fn concatenate_metrics(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Metrics, { METRICS_TYPE_MASK }, { Metrics::COUNT }>(branch_results, opts, None)
}

pub(crate) fn concatenate_traces(
    branch_results: &mut Vec<OtapArrowRecords>,
    opts: ConcatOptions,
) -> Result<OtapArrowRecords> {
    concat_generic::<Traces, { TRACES_TYPE_MASK }, { Traces::COUNT }>(branch_results, opts, None)
}

/// Concatenate branch results of the signal with the given root payload type.
///
/// `selections`, when supplied, has one entry per branch result. A `None` entry
/// keeps that result whole.
pub(crate) fn concatenate_selected(
    branch_results: &mut Vec<OtapArrowRecords>,
    root_type: ArrowPayloadType,
    opts: ConcatOptions,
    selections: Option<Vec<Option<PayloadSelections>>>,
) -> Result<OtapArrowRecords> {
    match root_type {
        ArrowPayloadType::Logs => concat_generic::<Logs, { LOGS_TYPE_MASK }, { Logs::COUNT }>(
            branch_results,
            opts,
            selections,
        ),
        ArrowPayloadType::Spans => {
            concat_generic::<Traces, { TRACES_TYPE_MASK }, { Traces::COUNT }>(
                branch_results,
                opts,
                selections,
            )
        }
        _ => concat_generic::<Metrics, { METRICS_TYPE_MASK }, { Metrics::COUNT }>(
            branch_results,
            opts,
            selections,
        ),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A branch's masks are sized for a different signal than logs.
    /// Guarantees: The masks are rejected rather than applied to the wrong slots.
    #[test]
    fn positional_masks_reject_wrong_slot_count() {
        let root = POSITION_LOOKUP[ArrowPayloadType::Logs as usize];
        let mut masks = vec![None; Traces::COUNT];
        masks[root] = Some(BooleanArray::from(vec![true]));
        assert!(positional_masks::<{ Logs::COUNT }>(masks).is_err());

        let mut masks = vec![None; Logs::COUNT];
        masks[root] = Some(BooleanArray::from(vec![true]));
        let positional = positional_masks::<{ Logs::COUNT }>(masks).unwrap();
        assert!(positional[root].is_some());
        assert_eq!(positional.iter().flatten().count(), 1);
    }
}
