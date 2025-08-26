//! Batching for `OtapArrowRecords`
//!
//!

use std::num::NonZeroU64;
use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};

/// merge and combine batches to the appropriate size
pub fn make_output_batches(
    max_output_batch: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
    // We have to deal with three complications here:
    // * batches that are too small
    // * batches that are too big
    // * cases where we have different types (logs/metrics/traces) intermingled

    // We deal with the last issue first, by splitting the input into three lists of the appropriate
    // types.
    let [mut logs, mut metrics, mut traces] = RecordsGroup::split_by_type(records);

    if let Some(max_output_batch) = max_output_batch {
        logs = logs.split(max_output_batch)?;
        metrics = metrics.split(max_output_batch)?;
        traces = traces.split(max_output_batch)?;
    }
    logs = logs.concatenate(max_output_batch)?;
    metrics = metrics.concatenate(max_output_batch)?;
    traces = traces.concatenate(max_output_batch)?;

    let mut result = Vec::new();
    result.extend(logs.into_otap_arrow_records());
    result.extend(metrics.into_otap_arrow_records());
    result.extend(traces.into_otap_arrow_records());

    // By splitting into 3 different lists, we've probably scrambled the ordering. We can't really
    // fix that problem in a general sense because each `OtapArrowRecords` will contain many rows ot
    // different times, but we can improve matters slightly by sorting on the smallest record time.

    // FIXME: sort here
    Ok(result)
}
