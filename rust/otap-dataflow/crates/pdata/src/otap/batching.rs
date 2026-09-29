// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use otel_arrow_dfe_config::SignalType;
use std::num::NonZeroU64;
use std::ops::RangeInclusive;

/// One output of [`make_item_batches`].
#[derive(Debug, Clone)]
pub struct ItemBatch {
    /// The batched records.
    pub records: OtapArrowRecords,
    /// Indices (into the `records` argument of [`make_item_batches`]) of the
    /// inputs that contributed rows to this output.
    ///
    /// Outputs are produced in input order, so consecutive outputs' ranges
    /// are non-decreasing and share at most one index (an input that was cut
    /// across them). An input that contributes no rows to any output (for
    /// example, because all of its rows were malformed and dropped) may be
    /// covered by no range, or covered by a range while contributing
    /// nothing.
    pub inputs: RangeInclusive<usize>,
}

/// Rebatch records to the appropriate size in a single pass, measured
/// in items.  Requires all inputs have the same signal type.
///
/// Each output reports which inputs it was built from; see [`ItemBatch`].
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<ItemBatch>> {
    // Separate by signal type.
    let mut records = match signal {
        SignalType::Logs => RecordsGroup::separate_logs(records),
        SignalType::Metrics => RecordsGroup::separate_metrics(records),
        SignalType::Traces => RecordsGroup::separate_traces(records),
    }?;

    // Split large batches so they can be reassembled into
    // limited-size batches.
    if let Some(limit) = max_items {
        records = records.split(limit)?;
    }

    // Join batches in sequence.
    Ok(records
        .concatenate(max_items)?
        .into_iter()
        .map(|o| ItemBatch {
            records: o.records,
            inputs: o.inputs,
        })
        .collect())
}
