// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass, measured in
/// items. Requires all inputs to have the same signal type.
///
/// A single non-empty input within the configured limit is returned unchanged.
/// Besides avoiding grouping work, this preserves every Arrow buffer and schema
/// `Arc`; that is the dominant batch-processor case when upstream batches are
/// already sized appropriately.
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
    // A single non-empty payload already satisfying the output limit requires
    // neither splitting nor concatenation. Keep its Arrow arrays intact and
    // avoid converting through RecordsGroup only to reconstruct the same
    // payload. Empty payloads retain the existing behavior of being dropped,
    // regardless of their signal variant.
    if let [record] = records.as_slice() {
        let item_count = record.num_items();
        if item_count == 0 {
            return Ok(Vec::new());
        }
        if record.signal_type() != signal {
            return Err(super::error::Error::MixedSignals);
        }

        let within_limit = max_items.is_none_or(|limit| {
            let effective_limit = limit.get().min(u32::MAX as u64);
            u64::try_from(item_count).is_ok_and(|count| count <= effective_limit)
        });
        if within_limit {
            return Ok(records);
        }
    }

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
    records.into_otap_arrow_records()
}
