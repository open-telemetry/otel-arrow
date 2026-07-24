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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use arrow::array::{Array, RecordBatch, StringArray, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow_select::filter::filter_record_batch;
    use otap_df_otap::pdata::Context;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    fn logs_pdata_with_severities(severities: &[&str]) -> OtapPdata {
        let schema = Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new("severity_text", DataType::Utf8, true),
        ]);
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt16Array::from(
                    (0..severities.len() as u16).collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(severities.to_vec())),
            ],
        )
        .expect("valid logs record batch");

        let mut records = OtapArrowRecords::Logs(Logs::default());
        records
            .set(ArrowPayloadType::Logs, record_batch)
            .expect("set logs root batch");
        OtapPdata::new(Context::default(), records.into())
    }

    fn severities_of(pdata: OtapPdata) -> Vec<String> {
        let (_ctx, payload) = pdata.into_parts();
        let records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert payload to otap records");
        let batch = records
            .get(ArrowPayloadType::Logs)
            .expect("logs root record batch");
        let strings = batch
            .column_by_name("severity_text")
            .expect("severity_text column")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("utf8 severity_text");
        (0..strings.len())
            .map(|i| strings.value(i).to_string())
            .collect()
    }

    #[test]
    fn skips_guest_call_when_root_batch_is_missing() {
        let input = OtapPdata::new(
            Context::default(),
            OtapArrowRecords::Logs(Logs::default()).into(),
        );
        let output = run_on_root_batch(input, |_batch| {
            panic!("closure should not be called for empty/rootless payload")
        })
        .expect("run_on_root_batch should pass through empty payloads")
        .expect("empty payload should be forwarded, not dropped");

        let (_ctx, payload) = output.into_parts();
        let records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert payload to otap records");
        assert!(
            records.get(ArrowPayloadType::Logs).is_none(),
            "empty logs payload should remain empty"
        );
    }

    #[test]
    fn returns_none_when_guest_drops_the_batch() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO"]);
        let output = run_on_root_batch(input, |_batch| Ok(None))
            .expect("guest-returned None is not an error");
        assert!(output.is_none(), "guest None must drop the input batch");
    }

    #[test]
    fn replaces_root_batch_with_guest_output() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO", "ERROR"]);

        let output = run_on_root_batch(input, |batch| {
            let keep = arrow::array::BooleanArray::from(vec![true, false, true]);
            let filtered = filter_record_batch(&batch, &keep).expect("filter root logs batch");
            Ok(Some(filtered))
        })
        .expect("guest success should map to Ok")
        .expect("guest returned a replacement batch");

        assert_eq!(severities_of(output), vec!["ERROR", "ERROR"]);
    }

    #[test]
    fn propagates_guest_errors() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO"]);
        let result = run_on_root_batch(input, |_batch| {
            Err(EngineError::RuntimeMsgError {
                error: "guest failed".to_string(),
            })
        });

        assert!(
            matches!(result, Err(EngineError::RuntimeMsgError { .. })),
            "guest closure errors should propagate"
        );
    }
}
