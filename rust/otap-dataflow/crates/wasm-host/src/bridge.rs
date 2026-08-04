// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `OtapPdata` <-> OTAP records bridge at the node boundary.
//!
//! The bridge converts payloads to [`OtapArrowRecords`], hands them to the
//! WASM host via a caller-provided closure, and reconstructs an [`OtapPdata`]
//! from the result. The pdata [`Context`] (Ack/Nack routing and transport
//! headers) is preserved so plugin-modified batches keep the same downstream
//! delivery semantics as unmodified data.

use otap_df_engine::error::Error as EngineError;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::TryIntoWithOptions;

/// Run `run` on `pdata` converted to OTAP records, preserving the pdata context.
///
/// - Returns `Ok(Some(pdata))` with the reconstructed message when the guest
///   returns records.
/// - Returns `Ok(None)` when the guest drops the pdata (`process` returned `none`).
///
/// The closure receives and returns full `OtapArrowRecords`. The host kernel
/// implementations (`filter_by_attribute_eq`, etc.) are trusted to produce
/// structurally valid OTAP output -- no additional schema re-validation is
/// performed on the returned records before forwarding downstream.
///
/// TODO: `OtlpBytes` payloads still flow through default conversion to
/// OTAP records; add native OTLP handling and explicit per-`ArrowPayloadType`
/// processing paths.
pub(crate) fn run_on_otap_records<F>(
    pdata: OtapPdata,
    run: F,
) -> Result<Option<OtapPdata>, EngineError>
where
    F: FnOnce(OtapArrowRecords) -> Result<Option<OtapArrowRecords>, EngineError>,
{
    let (context, payload) = pdata.into_parts();
    let records: OtapArrowRecords = payload.try_into_with_default()?;
    if records.root_record_batch().is_none() {
        // Nothing to process; forward unchanged (context preserved).
        return Ok(Some(OtapPdata::new(context, records.into())));
    }

    match run(records)? {
        Some(updated_records) => Ok(Some(OtapPdata::new(context, updated_records.into()))),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, StringArray};
    use arrow_select::filter::filter_record_batch;
    use otap_df_otap::pdata::Context;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, to_otap_logs};

    fn logs_pdata_with_severities(severities: &[&str]) -> OtapPdata {
        let records = to_otap_logs(
            severities
                .iter()
                .enumerate()
                .map(|(idx, severity)| {
                    LogRecord::build()
                        .severity_text(*severity)
                        .attributes(vec![KeyValue::new(
                            "k",
                            AnyValue::new_string(format!("v{idx}")),
                        )])
                        .finish()
                })
                .collect(),
        );
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
        let column = batch
            .column_by_name("severity_text")
            .expect("severity_text column");
        let utf8 = arrow_cast::cast(column, &arrow::datatypes::DataType::Utf8)
            .expect("cast severity_text to utf8");
        let strings = utf8
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("utf8 severity_text");
        (0..strings.len())
            .map(|i| strings.value(i).to_string())
            .collect()
    }

    /// Scenario: The payload has no root record batch for its signal type.
    /// Guarantees: The bridge forwards the payload unchanged and does not call the guest closure.
    #[test]
    fn skips_guest_call_when_root_batch_is_missing() {
        let input = OtapPdata::new(
            Context::default(),
            OtapArrowRecords::Logs(Logs::default()).into(),
        );
        let output = run_on_otap_records(input, |_records| {
            panic!("closure should not be called for empty/rootless payload")
        })
        .expect("run_on_otap_records should pass through empty payloads")
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

    /// Scenario: Guest `process` returns `none` for an input payload.
    /// Guarantees: The bridge reports a drop with `Ok(None)`.
    #[test]
    fn returns_none_when_guest_drops_the_batch() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO"]);
        let output = run_on_otap_records(input, |_records| Ok(None))
            .expect("guest-returned None is not an error");
        assert!(output.is_none(), "guest None must drop the input batch");
    }

    /// Scenario: Guest returns an updated OTAP records payload.
    /// Guarantees: The bridge forwards guest-updated records and preserves pdata context.
    #[test]
    fn replaces_root_batch_with_guest_output() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO", "ERROR"]);

        let output = run_on_otap_records(input, |mut records| {
            let root_type = records.root_payload_type();
            let batch = records
                .get(root_type)
                .expect("root batch present in logs payload");
            let keep = arrow::array::BooleanArray::from(vec![true, false, true]);
            let filtered = filter_record_batch(batch, &keep).expect("filter root logs batch");
            records
                .set(root_type, filtered)
                .expect("set filtered root batch");
            Ok(Some(records))
        })
        .expect("guest success should map to Ok")
        .expect("guest returned a replacement payload");

        assert_eq!(severities_of(output), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Guest processing closure returns an engine error.
    /// Guarantees: The bridge propagates the guest error unchanged.
    #[test]
    fn propagates_guest_errors() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO"]);
        let result = run_on_otap_records(input, |_records| {
            Err(EngineError::RuntimeMsgError {
                error: "guest failed".to_string(),
            })
        });

        assert!(
            matches!(result, Err(EngineError::RuntimeMsgError { .. })),
            "guest closure errors should propagate"
        );
    }

    /// Scenario: Guest updates logs whose attribute rows are linked by record IDs.
    /// Guarantees: The bridge round-trips OTAP records without corrupting per-record attributes.
    #[test]
    fn preserves_record_attributes_across_round_trip() {
        let input = logs_pdata_with_severities(&["ERROR", "INFO", "ERROR"]);
        let output = run_on_otap_records(input, |records| Ok(Some(records)))
            .expect("bridge run succeeds")
            .expect("payload is not dropped");

        let (_ctx, payload) = output.into_parts();
        let records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert payload to otap records");
        let otlp = otap_to_otlp(&records);

        let OtlpProtoMessage::Logs(logs) = otlp else {
            panic!("expected logs payload");
        };
        let attrs: Vec<String> = logs.resource_logs[0].scope_logs[0]
            .log_records
            .iter()
            .map(|record| record.attributes[0].value.as_ref().expect("any value"))
            .map(|v| match v.value.as_ref().expect("typed value") {
                otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                    s,
                ) => s.clone(),
                other => panic!("expected string attribute value, got {other:?}"),
            })
            .collect();
        assert_eq!(attrs, vec!["v0", "v1", "v2"]);
    }
}
