// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass, measured
/// in items.  Requires all inputs have the same signal type.
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
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
    records = records.concatenate(max_items)?;

    Ok(records.into_otap_arrow_records())
}
