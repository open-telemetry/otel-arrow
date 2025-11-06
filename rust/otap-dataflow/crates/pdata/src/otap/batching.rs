// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`
//!
//!

use super::{OtapArrowRecordTag, OtapArrowRecords, error::Result, groups::RecordsGroup};
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass.
/// Returns error if not the same signal type.
pub fn make_output_batches(
    signal: OtapArrowRecordTag,
    records: Vec<OtapArrowRecords>,
    max_output_batch: Option<NonZeroU64>,
) -> Result<Vec<OtapArrowRecords>> {
    // Separate by signal type and rebatch in one pass
    let records = match signal {
        OtapArrowRecordTag::Logs => RecordsGroup::separate_logs(records),
        OtapArrowRecordTag::Metrics => RecordsGroup::separate_metrics(records),
        OtapArrowRecordTag::Traces => RecordsGroup::separate_traces(records),
    }?;

    // Rebatch: iterate through inputs once, building maximally-full output batches
    let rebatched = records.rebatch(max_output_batch)?;

    Ok(rebatched.into_otap_arrow_records())
}
