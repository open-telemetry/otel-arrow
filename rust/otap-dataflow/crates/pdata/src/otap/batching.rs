// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use crate::batching::BatchingResult;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_config::SignalType;
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass, measured
/// in items.  Requires all inputs have the same signal type.
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
    Ok(make_item_batches_with_ranges(signal, max_items, records)?
        .batches
        .into_iter()
        .map(|(records, _)| records)
        .collect())
}

/// Rebatch with input provenance captured before splitting or integrity repair.
/// Inputs lacking a nonempty primary table are completed without output.
pub fn make_item_batches_with_ranges(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<BatchingResult<OtapArrowRecords>> {
    let input_count = records.len();
    let root = match signal {
        SignalType::Logs => ArrowPayloadType::Logs,
        SignalType::Metrics => ArrowPayloadType::UnivariateMetrics,
        SignalType::Traces => ArrowPayloadType::Spans,
    };
    let mut origins: Vec<_> = records
        .iter()
        .enumerate()
        .filter(|(_, records)| records.get(root).is_some_and(|root| root.num_rows() > 0))
        .map(|(index, _)| index)
        .collect();
    // Separate by signal type.
    let mut records = match signal {
        SignalType::Logs => RecordsGroup::separate_logs(records),
        SignalType::Metrics => RecordsGroup::separate_metrics(records),
        SignalType::Traces => RecordsGroup::separate_traces(records),
    }?;

    // Split large batches so they can be reassembled into
    // limited-size batches.
    if let Some(limit) = max_items {
        records = records.split(limit, &mut origins)?;
    }

    // Join batches in sequence.
    let (records, ranges) = records.concatenate(max_items, origins)?;
    let batches: Vec<_> = records
        .into_otap_arrow_records()?
        .into_iter()
        .zip(ranges)
        .filter(|(records, _)| records.num_items() != 0)
        .collect();
    let mut completed_inputs = Vec::new();
    let mut covered = 0;
    for (_, inputs) in &batches {
        completed_inputs.extend(covered..*inputs.start());
        covered = inputs.end() + 1;
    }
    completed_inputs.extend(covered..input_count);
    Ok(BatchingResult {
        batches,
        completed_inputs,
        budget_fallbacks: 0,
    })
}
