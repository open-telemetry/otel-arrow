// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `OtapPdata` <-> Arrow `RecordBatch` bridge at the node boundary.
//!
//! The bridge extracts the root [`RecordBatch`] of an [`OtapPdata`] message,
//! hands it to the WASM guest via a caller-provided closure, and reconstructs
//! an [`OtapPdata`] from the result. The pdata [`Context`] (Ack/Nack routing
//! and transport headers) is preserved so plugin-modified batches keep the same
//! downstream delivery semantics as unmodified data.

use arrow::array::RecordBatch;
use otap_df_engine::error::Error as EngineError;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::TryIntoWithOptions;

/// Run `run` on the root record batch of `pdata`, preserving the pdata context.
///
/// - Returns `Ok(Some(pdata))` with the reconstructed message when the guest
///   returns a batch.
/// - Returns `Ok(None)` when the guest drops the batch (`process` returned
///   `none`).
///
/// The reconstructed batch is validated against the OTAP schema invariants by
/// [`OtapArrowRecords::set`] before it is forwarded downstream, giving the
/// host-side "valid OTel data" guarantee for free.
///
/// TODO: `OtlpBytes` payloads are converted to OTAP records via the
/// default conversion; native OTLP handling and per-`ArrowPayloadType`
/// processing (including child attribute batches) are deferred.
pub(crate) fn run_on_root_batch<F>(
    pdata: OtapPdata,
    run: F,
) -> Result<Option<OtapPdata>, EngineError>
where
    F: FnOnce(RecordBatch) -> Result<Option<RecordBatch>, EngineError>,
{
    let (context, payload) = pdata.into_parts();
    let mut records: OtapArrowRecords = payload.try_into_with_default()?;

    let root_type = records.root_payload_type();
    let Some(root_batch) = records.get(root_type).cloned() else {
        // Nothing to process; forward unchanged (context preserved).
        return Ok(Some(OtapPdata::new(context, records.into())));
    };

    match run(root_batch)? {
        Some(filtered) => {
            records.set(root_type, filtered)?;
            Ok(Some(OtapPdata::new(context, records.into())))
        }
        None => Ok(None),
    }
}
