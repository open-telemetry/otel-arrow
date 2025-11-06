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
    eprintln!("  [make_output_batches] Input: signal={:?}, {} records, max_output_batch={:?}", 
        signal, records.len(), max_output_batch);
    for (i, rec) in records.iter().enumerate() {
        eprintln!("    Input[{}]: batch_length={}", i, rec.batch_length());
    }
    
    // Separate by signal type and rebatch in one pass
    let mut records = match signal {
        SignalType::Logs => RecordsGroup::separate_logs(records),
        SignalType::Metrics => RecordsGroup::separate_metrics(records),
        SignalType::Traces => RecordsGroup::separate_traces(records),
    }?;
    
    eprintln!("  [make_output_batches] After separate: RecordsGroup.len()={}", records.len());

    // Rebatch: iterate through inputs once, building maximally-full output batches
    if let Some(limit) = max_output_batch {
        eprintln!("  [make_output_batches] Calling split with limit={}", limit);
        records = records.split(limit)?;
        eprintln!("  [make_output_batches] After split: RecordsGroup.len()={}", records.len());
    }
    
    eprintln!("  [make_output_batches] Calling concatenate with max_output_batch={:?}", max_output_batch);
    records = records.concatenate(max_output_batch)?;
    eprintln!("  [make_output_batches] After concatenate: RecordsGroup.len()={}", records.len());

    let result = records.into_otap_arrow_records();
    eprintln!("  [make_output_batches] Final output: {} records", result.len());
    for (i, rec) in result.iter().enumerate() {
        eprintln!("    Output[{}]: batch_length={}", i, rec.batch_length());
    }
    
    Ok(result)
}
