// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`
//!
//!

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass.
/// Returns error if not the same signal type.
pub fn make_output_batches(
    signal: SignalType,
    records: Vec<OtapArrowRecords>,
    max_output_batch: Option<NonZeroU64>,
) -> Result<Vec<OtapArrowRecords>> {
    // Separate by signal type and rebatch in one pass
    let mut records = match signal {
        SignalType::Logs => RecordsGroup::separate_logs(records),
        SignalType::Metrics => RecordsGroup::separate_metrics(records),
        SignalType::Traces => RecordsGroup::separate_traces(records),
    }?;

    // Rebatch: iterate through inputs once, building maximally-full output batches
    if let Some(limit) = max_output_batch {
        records = records.split(limit)?;
    }
    records = records.concatenate(max_output_batch)?;

    Ok(records.into_otap_arrow_records())
}
