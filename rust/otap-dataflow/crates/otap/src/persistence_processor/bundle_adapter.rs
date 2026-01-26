// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Adapter implementing Quiver's `RecordBundle` trait for OTAP data types.
//!
//! This module bridges the OTAP data model (`OtapArrowRecords`) with Quiver's
//! persistence layer by implementing the `RecordBundle` trait.
//!
//! # Slot ID Design
//!
//! Quiver's WAL uses a 64-bit bitmap for slot presence, so slot IDs must be < 64.
//! Since each bundle contains only one signal type and ArrowPayloadType values
//! go up to ~45, we use the ArrowPayloadType value directly as the slot ID.
//!
//! Slot IDs map directly to `ArrowPayloadType` enum values defined in
//! `proto/opentelemetry/proto/experimental/arrow/v1/arrow_service.proto`:
//!
//! | Signal  | Payload Types (slot IDs)                                    | Table Count |
//! |---------|-------------------------------------------------------------|-------------|
//! | Shared  | RESOURCE_ATTRS (1), SCOPE_ATTRS (2)                         | 2           |
//! | Metrics | UNIVARIATE_METRICS (10) through METRIC_ATTRS (26)           | 17          |
//! | Logs    | LOGS (30), LOG_ATTRS (31)                                   | 2           |
//! | Traces  | SPANS (40) through SPAN_LINK_ATTRS (45)                     | 6           |
//!
//! Total tables per signal (including shared RESOURCE_ATTRS and SCOPE_ATTRS):
//! - Logs: 4 tables (LOGS, LOG_ATTRS, RESOURCE_ATTRS, SCOPE_ATTRS)
//! - Traces: 8 tables (SPANS, SPAN_ATTRS, SPAN_EVENTS, SPAN_LINKS,
//!   SPAN_EVENT_ATTRS, SPAN_LINK_ATTRS, RESOURCE_ATTRS, SCOPE_ATTRS)
//! - Metrics: 19 tables (UNIVARIATE_METRICS, MULTIVARIATE_METRICS, 4 data point
//!   types, 4 DP attrs, 3 exemplar types, 3 exemplar attrs, METRIC_ATTRS,
//!   RESOURCE_ATTRS, SCOPE_ATTRS)
//!
//! Reserved slots for OTLP pass-through (opaque binary storage):
//! - Slot 60: OTLP Logs
//! - Slot 61: OTLP Traces
//! - Slot 62: OTLP Metrics
//!
//! Note: Slots identify payload *types*, not schemas. Within each slot, Quiver
//! supports many different schemas via `(slot_id, schema_fingerprint)` pairs
//! that create distinct "streams" (up to 100,000 per segment).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use arrow::array::{BinaryArray, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema};
use quiver::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};
use quiver::segment::ReconstructedBundle;

use otap_df_config::SignalType;
use otap_df_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};

use crate::pdata::{Context, OtapPdata};

// ─────────────────────────────────────────────────────────────────────────────
// Slot ID Constants
// ─────────────────────────────────────────────────────────────────────────────

/// OTLP opaque binary storage slots (for pass-through mode)
/// These are in the 60-62 range to stay within the 64-slot WAL bitmap limit
mod otlp_slots {
    pub const OTLP_LOGS: u16 = 60;
    pub const OTLP_TRACES: u16 = 61;
    pub const OTLP_METRICS: u16 = 62;
}

/// Convert payload type to a slot ID (direct mapping).
///
/// Note: The slot ID is determined solely by the payload type, not the signal type.
/// Signal type is passed for consistency with the API but is not used in the mapping.
fn to_slot_id(_signal_type: SignalType, payload_type: ArrowPayloadType) -> SlotId {
    // ArrowPayloadType values are 0-45, which fits directly in the 64-slot limit
    SlotId::new(payload_type as u16)
}

/// Convert signal type to OTLP slot ID (for opaque binary storage)
fn to_otlp_slot_id(signal_type: SignalType) -> SlotId {
    SlotId::new(match signal_type {
        SignalType::Logs => otlp_slots::OTLP_LOGS,
        SignalType::Traces => otlp_slots::OTLP_TRACES,
        SignalType::Metrics => otlp_slots::OTLP_METRICS,
    })
}

/// Check if a slot ID is an OTLP opaque binary slot
fn is_otlp_slot(slot: SlotId) -> Option<SignalType> {
    match slot.raw() {
        otlp_slots::OTLP_LOGS => Some(SignalType::Logs),
        otlp_slots::OTLP_TRACES => Some(SignalType::Traces),
        otlp_slots::OTLP_METRICS => Some(SignalType::Metrics),
        _ => None,
    }
}

/// Convert a slot ID back to payload type only (Arrow format only).
///
/// Returns the `ArrowPayloadType` for the given slot, or `None` if the slot
/// is not a valid Arrow payload slot (e.g., OTLP opaque slots 60-62).
fn slot_to_payload_type(slot: SlotId) -> Option<ArrowPayloadType> {
    let raw = slot.raw();

    // Check if it's an OTLP slot (60-62 range, not an ArrowPayloadType)
    if raw >= otlp_slots::OTLP_LOGS {
        return None;
    }

    ArrowPayloadType::try_from(raw as i32).ok()
}

/// Check if a slot ID represents a shared payload type (RESOURCE_ATTRS, SCOPE_ATTRS).
///
/// These slots are used by ALL signal types, so their presence alone cannot
/// determine the signal type of a bundle.
fn is_shared_slot(slot: SlotId) -> bool {
    matches!(slot.raw(), 1 | 2) // RESOURCE_ATTRS (1), SCOPE_ATTRS (2)
}

/// Convert a slot ID back to signal type and payload type (Arrow format only).
///
/// This reverse mapping is used during WAL recovery to reconstruct the signal type
/// from persisted slot IDs. The ranges correspond to `ArrowPayloadType` values
/// from `arrow_service.proto`.
///
/// # Returns
/// - `Some((signal_type, payload_type))` for signal-specific slots
/// - `None` for shared slots (RESOURCE_ATTRS, SCOPE_ATTRS) since they're used by all signals
/// - `None` for OTLP opaque slots (60-62) or invalid slot IDs
fn from_slot_id(slot: SlotId) -> Option<(SignalType, ArrowPayloadType)> {
    let payload_type = slot_to_payload_type(slot)?;
    let raw = slot.raw();

    // Determine signal type from the ArrowPayloadType value ranges.
    // See arrow_service.proto for the enum definitions:
    //   - Metrics: 10-26 (UNIVARIATE_METRICS through METRIC_ATTRS)
    //   - Logs: 30-31 (LOGS, LOG_ATTRS)
    //   - Traces: 40-45 (SPANS through SPAN_LINK_ATTRS)
    //   - Shared: 1-2 (RESOURCE_ATTRS, SCOPE_ATTRS) - used by all signals, return None
    let signal_type = match raw {
        10..=26 => SignalType::Metrics, // UNIVARIATE_METRICS (10) through METRIC_ATTRS (26)
        30..=31 => SignalType::Logs,    // LOGS (30), LOG_ATTRS (31)
        40..=45 => SignalType::Traces,  // SPANS (40) through SPAN_LINK_ATTRS (45)
        1..=2 => return None,           // Shared slots - cannot determine signal type
        _ => return None,               // Unknown slots
    };

    Some((signal_type, payload_type))
}

/// Get the slot label for a signal and payload type
fn slot_label(signal_type: SignalType, payload_type: ArrowPayloadType) -> Cow<'static, str> {
    let signal_prefix = match signal_type {
        SignalType::Logs => "Log",
        SignalType::Traces => "Trace",
        SignalType::Metrics => "Metric",
    };
    Cow::Owned(format!("{}:{}", signal_prefix, payload_type.as_str_name()))
}

/// Get the slot label for an OTLP opaque slot
fn otlp_slot_label(signal_type: SignalType) -> Cow<'static, str> {
    Cow::Borrowed(match signal_type {
        SignalType::Logs => "OtlpLogs",
        SignalType::Traces => "OtlpTraces",
        SignalType::Metrics => "OtlpMetrics",
    })
}

/// Schema for OTLP opaque binary storage (single binary column)
fn otlp_binary_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![Field::new(
        "data",
        DataType::Binary,
        false,
    )]))
}

/// Schema fingerprint for OTLP binary storage (stable across all OTLP slots)
fn otlp_schema_fingerprint() -> SchemaFingerprint {
    // Use a fixed fingerprint since all OTLP slots have the same schema
    let hash = blake3::hash(b"otlp_binary_v1");
    *hash.as_bytes()
}

/// Compute schema fingerprint from a RecordBatch using blake3.
fn compute_schema_fingerprint(batch: &RecordBatch) -> SchemaFingerprint {
    let schema = batch.schema();
    // Create a deterministic string representation of the schema
    let schema_str = format!("{:?}", schema);
    let hash = blake3::hash(schema_str.as_bytes());
    *hash.as_bytes()
}

// ─────────────────────────────────────────────────────────────────────────────
// OtapRecordBundleAdapter
// ─────────────────────────────────────────────────────────────────────────────

/// Adapter that wraps OtapArrowRecords and implements Quiver's RecordBundle trait.
pub struct OtapRecordBundleAdapter {
    /// The underlying OTAP arrow records
    records: OtapArrowRecords,
    /// The signal type (Logs, Traces, or Metrics)
    signal_type: SignalType,
    /// Cached bundle descriptor
    descriptor: BundleDescriptor,
    /// Ingestion timestamp
    ingestion_time: SystemTime,
}

impl OtapRecordBundleAdapter {
    /// Create a new adapter for the given OtapArrowRecords.
    #[must_use]
    pub fn new(records: OtapArrowRecords) -> Self {
        let signal_type = match &records {
            OtapArrowRecords::Logs(_) => SignalType::Logs,
            OtapArrowRecords::Traces(_) => SignalType::Traces,
            OtapArrowRecords::Metrics(_) => SignalType::Metrics,
        };
        let descriptor = Self::build_descriptor(&records, signal_type);
        Self {
            records,
            signal_type,
            descriptor,
            ingestion_time: SystemTime::now(),
        }
    }

    /// Build the bundle descriptor from the OTAP payload.
    fn build_descriptor(records: &OtapArrowRecords, signal_type: SignalType) -> BundleDescriptor {
        let mut slot_descriptors = Vec::new();

        for payload_type in records.allowed_payload_types() {
            if records.get(*payload_type).is_some() {
                let slot_id = to_slot_id(signal_type, *payload_type);
                slot_descriptors.push(SlotDescriptor::new(
                    slot_id,
                    slot_label(signal_type, *payload_type),
                ));
            }
        }

        BundleDescriptor::new(slot_descriptors)
    }
}

impl RecordBundle for OtapRecordBundleAdapter {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        // Get the payload type from the slot ID
        let payload_type = slot_to_payload_type(slot)?;

        // For signal-specific slots, verify the signal type matches
        if let Some((signal_type, _)) = from_slot_id(slot) {
            if signal_type != self.signal_type {
                return None;
            }
        }
        // Shared slots (RESOURCE_ATTRS, SCOPE_ATTRS) are allowed for any signal type

        let batch = self.records.get(payload_type)?;

        Some(PayloadRef {
            schema_fingerprint: compute_schema_fingerprint(batch),
            batch,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OtlpBytesAdapter (for pass-through mode)
// ─────────────────────────────────────────────────────────────────────────────

/// Adapter that wraps OtlpProtoBytes and implements Quiver's RecordBundle trait.
///
/// This stores OTLP data as opaque binary for efficient pass-through pipelines.
pub struct OtlpBytesAdapter {
    /// The signal type (Logs, Traces, or Metrics)
    signal_type: SignalType,
    /// The record batch containing the binary data
    batch: RecordBatch,
    /// Cached bundle descriptor
    descriptor: BundleDescriptor,
    /// Ingestion timestamp
    ingestion_time: SystemTime,
}

impl OtlpBytesAdapter {
    /// Create a new adapter for the given OtlpProtoBytes.
    #[must_use]
    pub fn new(bytes: OtlpProtoBytes) -> Self {
        let signal_type = match &bytes {
            OtlpProtoBytes::ExportLogsRequest(_) => SignalType::Logs,
            OtlpProtoBytes::ExportMetricsRequest(_) => SignalType::Metrics,
            OtlpProtoBytes::ExportTracesRequest(_) => SignalType::Traces,
        };

        // Create a record batch with a single binary column containing the OTLP bytes
        let binary_array = BinaryArray::from_vec(vec![bytes.as_bytes()]);
        let batch = RecordBatch::try_new(otlp_binary_schema(), vec![Arc::new(binary_array)])
            .expect("valid schema and array");

        let slot_id = to_otlp_slot_id(signal_type);
        let descriptor = BundleDescriptor::new(vec![SlotDescriptor::new(
            slot_id,
            otlp_slot_label(signal_type),
        )]);

        Self {
            signal_type,
            batch,
            descriptor,
            ingestion_time: SystemTime::now(),
        }
    }
}

impl RecordBundle for OtlpBytesAdapter {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        // Check if this is our OTLP slot
        let slot_signal_type = is_otlp_slot(slot)?;
        if slot_signal_type != self.signal_type {
            return None;
        }

        Some(PayloadRef {
            schema_fingerprint: otlp_schema_fingerprint(),
            batch: &self.batch,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ReconstructedBundle to OtapPdata conversion
// ─────────────────────────────────────────────────────────────────────────────

/// Error returned when converting a bundle to OtapPdata fails.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BundleConversionError {
    /// The bundle contains no payloads.
    #[error("empty bundle")]
    EmptyBundle,
    /// Could not determine the signal type from the slots present.
    #[error("could not determine signal type from slots")]
    UnknownSignalType,
    /// Failed to extract binary data from OTLP slot.
    #[error("failed to extract OTLP bytes: {0}")]
    OtlpExtractionError(String),
}

/// Check if the bundle contains an OTLP opaque slot.
fn find_otlp_slot(payloads: &HashMap<SlotId, RecordBatch>) -> Option<(SignalType, &RecordBatch)> {
    for (slot_id, batch) in payloads {
        if let Some(signal_type) = is_otlp_slot(*slot_id) {
            return Some((signal_type, batch));
        }
    }
    None
}

/// Determine signal type from the Arrow slots present in the bundle.
fn determine_signal_type(
    payloads: &HashMap<SlotId, RecordBatch>,
) -> Result<SignalType, BundleConversionError> {
    for slot_id in payloads.keys() {
        if let Some((signal_type, _)) = from_slot_id(*slot_id) {
            return Ok(signal_type);
        }
    }
    Err(BundleConversionError::UnknownSignalType)
}

/// Extract OTLP bytes from an OTLP bundle's record batch.
fn extract_otlp_bytes(
    signal_type: SignalType,
    batch: &RecordBatch,
) -> Result<OtlpProtoBytes, BundleConversionError> {
    // The batch should have a single binary column
    if batch.num_columns() != 1 {
        return Err(BundleConversionError::OtlpExtractionError(format!(
            "expected 1 column, got {}",
            batch.num_columns()
        )));
    }
    if batch.num_rows() != 1 {
        return Err(BundleConversionError::OtlpExtractionError(format!(
            "expected 1 row, got {}",
            batch.num_rows()
        )));
    }

    let column = batch.column(0);
    let binary_array = column
        .as_any()
        .downcast_ref::<BinaryArray>()
        .ok_or_else(|| {
            BundleConversionError::OtlpExtractionError("expected BinaryArray".to_string())
        })?;

    let bytes = binary_array.value(0).to_vec();
    Ok(OtlpProtoBytes::new_from_bytes(signal_type, bytes))
}

/// Convert a ReconstructedBundle back to OtapPdata.
///
/// This reconstructs the original OTAP telemetry data from Quiver's storage format.
/// It handles both Arrow-format bundles and OTLP opaque bundles.
pub fn convert_bundle_to_pdata(
    bundle: &ReconstructedBundle,
) -> Result<OtapPdata, BundleConversionError> {
    let payloads = bundle.payloads();

    if payloads.is_empty() {
        return Err(BundleConversionError::EmptyBundle);
    }

    // First, check if this is an OTLP opaque bundle
    if let Some((signal_type, batch)) = find_otlp_slot(payloads) {
        let otlp_bytes = extract_otlp_bytes(signal_type, batch)?;
        let payload = OtapPayload::OtlpBytes(otlp_bytes);
        return Ok(OtapPdata::new(Context::default(), payload));
    }

    // Otherwise, it's an Arrow-format bundle
    let signal_type = determine_signal_type(payloads)?;

    // Create the appropriate OtapArrowRecords variant
    let records = match signal_type {
        SignalType::Logs => create_logs_records(payloads),
        SignalType::Traces => create_traces_records(payloads),
        SignalType::Metrics => create_metrics_records(payloads),
    };

    // Wrap in OtapPayload and OtapPdata
    let payload = OtapPayload::OtapArrowRecords(records);
    Ok(OtapPdata::new(Context::default(), payload))
}

/// Create Logs records from payloads.
fn create_logs_records(payloads: &HashMap<SlotId, RecordBatch>) -> OtapArrowRecords {
    let mut logs = Logs::default();

    for (slot_id, batch) in payloads {
        // Include signal-specific slots for Logs
        if let Some((SignalType::Logs, payload_type)) = from_slot_id(*slot_id) {
            logs.set(payload_type, batch.clone());
        }
        // Also include shared slots (RESOURCE_ATTRS, SCOPE_ATTRS)
        else if let Some(payload_type) = slot_to_payload_type(*slot_id) {
            if is_shared_slot(*slot_id) {
                logs.set(payload_type, batch.clone());
            }
        }
    }

    OtapArrowRecords::Logs(logs)
}

/// Create Traces records from payloads.
fn create_traces_records(payloads: &HashMap<SlotId, RecordBatch>) -> OtapArrowRecords {
    let mut traces = Traces::default();

    for (slot_id, batch) in payloads {
        // Include signal-specific slots for Traces
        if let Some((SignalType::Traces, payload_type)) = from_slot_id(*slot_id) {
            traces.set(payload_type, batch.clone());
        }
        // Also include shared slots (RESOURCE_ATTRS, SCOPE_ATTRS)
        else if let Some(payload_type) = slot_to_payload_type(*slot_id) {
            if is_shared_slot(*slot_id) {
                traces.set(payload_type, batch.clone());
            }
        }
    }

    OtapArrowRecords::Traces(traces)
}

/// Create Metrics records from payloads.
fn create_metrics_records(payloads: &HashMap<SlotId, RecordBatch>) -> OtapArrowRecords {
    let mut metrics = Metrics::default();

    for (slot_id, batch) in payloads {
        // Include signal-specific slots for Metrics
        if let Some((SignalType::Metrics, payload_type)) = from_slot_id(*slot_id) {
            metrics.set(payload_type, batch.clone());
        }
        // Also include shared slots (RESOURCE_ATTRS, SCOPE_ATTRS)
        else if let Some(payload_type) = slot_to_payload_type(*slot_id) {
            if is_shared_slot(*slot_id) {
                metrics.set(payload_type, batch.clone());
            }
        }
    }

    OtapArrowRecords::Metrics(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int64Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));
        let array = Int64Array::from(vec![1, 2, 3]);
        RecordBatch::try_new(schema, vec![Arc::new(array)]).unwrap()
    }

    #[test]
    fn test_slot_id_roundtrip() {
        // Test that signal-specific payload types roundtrip correctly
        let test_cases = [
            (SignalType::Logs, ArrowPayloadType::Logs, SignalType::Logs),
            (
                SignalType::Logs,
                ArrowPayloadType::LogAttrs,
                SignalType::Logs,
            ),
            (
                SignalType::Traces,
                ArrowPayloadType::Spans,
                SignalType::Traces,
            ),
            (
                SignalType::Traces,
                ArrowPayloadType::SpanAttrs,
                SignalType::Traces,
            ),
            (
                SignalType::Metrics,
                ArrowPayloadType::UnivariateMetrics,
                SignalType::Metrics,
            ),
            (
                SignalType::Metrics,
                ArrowPayloadType::NumberDataPoints,
                SignalType::Metrics,
            ),
        ];

        for (signal_type, payload_type, expected_signal) in test_cases {
            let slot = to_slot_id(signal_type, payload_type);
            let (recovered_signal, recovered_payload) =
                from_slot_id(slot).expect("should reverse map");
            assert_eq!(
                expected_signal, recovered_signal,
                "signal type mismatch for {:?}",
                payload_type
            );
            assert_eq!(payload_type, recovered_payload, "payload type mismatch");
        }
    }

    #[test]
    fn test_shared_slots_return_none() {
        // Shared slots (RESOURCE_ATTRS, SCOPE_ATTRS) should return None from from_slot_id
        // because they're used by ALL signal types and we can't determine which one.
        let shared_slots = [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ];

        for payload_type in shared_slots {
            let slot = to_slot_id(SignalType::Logs, payload_type);
            assert!(
                from_slot_id(slot).is_none(),
                "shared slot {:?} should return None",
                payload_type
            );
            assert!(
                is_shared_slot(slot),
                "slot {:?} should be identified as shared",
                payload_type
            );

            // But we can still get the payload type
            let recovered_payload = slot_to_payload_type(slot).expect("should get payload type");
            assert_eq!(payload_type, recovered_payload);
        }
    }

    #[test]
    fn test_signal_type_detection() {
        let mut payloads = HashMap::new();
        let _ = payloads.insert(
            to_slot_id(SignalType::Logs, ArrowPayloadType::Logs),
            create_test_batch(),
        );

        let signal = determine_signal_type(&payloads).unwrap();
        assert_eq!(signal, SignalType::Logs);
    }

    #[test]
    fn test_adapter_descriptor() {
        let mut logs = Logs::default();
        logs.set(ArrowPayloadType::Logs, create_test_batch());

        let records = OtapArrowRecords::Logs(logs);
        let adapter = OtapRecordBundleAdapter::new(records);

        let descriptor = adapter.descriptor();
        assert_eq!(descriptor.slots.len(), 1);

        let expected_slot = to_slot_id(SignalType::Logs, ArrowPayloadType::Logs);
        assert!(descriptor.get(expected_slot).is_some());
    }

    #[test]
    fn test_adapter_payload_access() {
        let mut logs = Logs::default();
        logs.set(ArrowPayloadType::Logs, create_test_batch());

        let records = OtapArrowRecords::Logs(logs);
        let adapter = OtapRecordBundleAdapter::new(records);

        let logs_slot = to_slot_id(SignalType::Logs, ArrowPayloadType::Logs);
        let payload = adapter.payload(logs_slot);
        assert!(payload.is_some());

        let spans_slot = to_slot_id(SignalType::Traces, ArrowPayloadType::Spans);
        let payload = adapter.payload(spans_slot);
        assert!(payload.is_none());
    }

    #[test]
    fn test_otlp_slot_id_roundtrip() {
        let test_cases = [SignalType::Logs, SignalType::Traces, SignalType::Metrics];

        for signal_type in test_cases {
            let slot = to_otlp_slot_id(signal_type);
            let recovered_signal = is_otlp_slot(slot).expect("should be OTLP slot");
            assert_eq!(signal_type, recovered_signal);

            // Verify it's not an Arrow slot
            assert!(from_slot_id(slot).is_none());
        }
    }

    #[test]
    fn test_otlp_bytes_adapter() {
        let test_bytes = b"test OTLP protobuf data".to_vec();
        let otlp = OtlpProtoBytes::new_from_bytes(SignalType::Logs, test_bytes.clone());

        let adapter = OtlpBytesAdapter::new(otlp);

        // Check descriptor
        let descriptor = adapter.descriptor();
        assert_eq!(descriptor.slots.len(), 1);

        let expected_slot = to_otlp_slot_id(SignalType::Logs);
        assert!(descriptor.get(expected_slot).is_some());

        // Check payload
        let payload = adapter.payload(expected_slot);
        assert!(payload.is_some());

        let payload_ref = payload.unwrap();
        assert_eq!(payload_ref.batch.num_rows(), 1);
        assert_eq!(payload_ref.batch.num_columns(), 1);

        // Verify the binary data is stored correctly
        let column = payload_ref.batch.column(0);
        let binary_array = column.as_any().downcast_ref::<BinaryArray>().unwrap();
        assert_eq!(binary_array.value(0), test_bytes.as_slice());

        // Wrong signal type slot should return None
        let wrong_slot = to_otlp_slot_id(SignalType::Traces);
        assert!(adapter.payload(wrong_slot).is_none());
    }

    #[test]
    fn test_extract_otlp_bytes() {
        let original_bytes = b"original OTLP data".to_vec();
        let otlp = OtlpProtoBytes::new_from_bytes(SignalType::Traces, original_bytes.clone());

        // Store in adapter
        let adapter = OtlpBytesAdapter::new(otlp);

        // Get the batch from the adapter
        let slot = to_otlp_slot_id(SignalType::Traces);
        let payload = adapter.payload(slot).unwrap();

        // Extract bytes back
        let extracted = extract_otlp_bytes(SignalType::Traces, payload.batch).unwrap();
        assert_eq!(extracted.as_bytes(), original_bytes.as_slice());
    }

    #[test]
    fn test_find_otlp_slot() {
        // Test with OTLP bundle
        let otlp = OtlpProtoBytes::new_from_bytes(SignalType::Metrics, b"data".to_vec());
        let adapter = OtlpBytesAdapter::new(otlp);
        let slot = to_otlp_slot_id(SignalType::Metrics);
        let payload = adapter.payload(slot).unwrap();

        let mut payloads = HashMap::new();
        let _ = payloads.insert(slot, payload.batch.clone());

        let result = find_otlp_slot(&payloads);
        assert!(result.is_some());
        let (signal_type, _) = result.unwrap();
        assert_eq!(signal_type, SignalType::Metrics);

        // Test with Arrow bundle (should return None)
        let mut arrow_payloads = HashMap::new();
        let _ = arrow_payloads.insert(
            to_slot_id(SignalType::Logs, ArrowPayloadType::Logs),
            create_test_batch(),
        );

        assert!(find_otlp_slot(&arrow_payloads).is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Roundtrip tests: Simulating WAL persist → recover cycle
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_traces_bundle_with_shared_slots_roundtrip() {
        // Simulate a Traces bundle that includes shared slots (RESOURCE_ATTRS, SCOPE_ATTRS)
        // This is the realistic case - bundles always have shared slots alongside signal-specific ones

        let mut traces = Traces::default();
        traces.set(ArrowPayloadType::Spans, create_test_batch());
        traces.set(ArrowPayloadType::SpanAttrs, create_test_batch());
        traces.set(ArrowPayloadType::ResourceAttrs, create_test_batch()); // shared
        traces.set(ArrowPayloadType::ScopeAttrs, create_test_batch()); // shared

        let records = OtapArrowRecords::Traces(traces);
        let adapter = OtapRecordBundleAdapter::new(records);

        // Simulate what Quiver does: extract slot→batch pairs
        let mut recovered_payloads: HashMap<SlotId, RecordBatch> = HashMap::new();
        for slot_desc in &adapter.descriptor().slots {
            if let Some(payload) = adapter.payload(slot_desc.id) {
                let _ = recovered_payloads.insert(slot_desc.id, payload.batch.clone());
            }
        }

        // Verify we have all 4 slots
        assert_eq!(recovered_payloads.len(), 4);

        // Now simulate recovery: determine signal type
        let signal_type = determine_signal_type(&recovered_payloads).unwrap();
        assert_eq!(signal_type, SignalType::Traces);

        // Reconstruct the records
        let reconstructed = create_traces_records(&recovered_payloads);

        // Verify all tables were recovered, including shared slots
        if let OtapArrowRecords::Traces(traces) = reconstructed {
            assert!(
                traces.get(ArrowPayloadType::Spans).is_some(),
                "SPANS should be recovered"
            );
            assert!(
                traces.get(ArrowPayloadType::SpanAttrs).is_some(),
                "SPAN_ATTRS should be recovered"
            );
            assert!(
                traces.get(ArrowPayloadType::ResourceAttrs).is_some(),
                "RESOURCE_ATTRS (shared) should be recovered"
            );
            assert!(
                traces.get(ArrowPayloadType::ScopeAttrs).is_some(),
                "SCOPE_ATTRS (shared) should be recovered"
            );
        } else {
            panic!("Expected Traces variant");
        }
    }

    #[test]
    fn test_metrics_bundle_with_shared_slots_roundtrip() {
        // Metrics bundle with shared slots
        let mut metrics = Metrics::default();
        metrics.set(ArrowPayloadType::UnivariateMetrics, create_test_batch());
        metrics.set(ArrowPayloadType::NumberDataPoints, create_test_batch());
        metrics.set(ArrowPayloadType::ResourceAttrs, create_test_batch()); // shared
        metrics.set(ArrowPayloadType::ScopeAttrs, create_test_batch()); // shared

        let records = OtapArrowRecords::Metrics(metrics);
        let adapter = OtapRecordBundleAdapter::new(records);

        // Extract slot→batch pairs (simulating WAL storage)
        let mut recovered_payloads: HashMap<SlotId, RecordBatch> = HashMap::new();
        for slot_desc in &adapter.descriptor().slots {
            if let Some(payload) = adapter.payload(slot_desc.id) {
                let _ = recovered_payloads.insert(slot_desc.id, payload.batch.clone());
            }
        }

        // Signal type should be detected from UNIVARIATE_METRICS or NUMBER_DATA_POINTS
        let signal_type = determine_signal_type(&recovered_payloads).unwrap();
        assert_eq!(signal_type, SignalType::Metrics);

        // Reconstruct and verify shared slots are included
        let reconstructed = create_metrics_records(&recovered_payloads);

        if let OtapArrowRecords::Metrics(metrics) = reconstructed {
            assert!(metrics.get(ArrowPayloadType::UnivariateMetrics).is_some());
            assert!(metrics.get(ArrowPayloadType::NumberDataPoints).is_some());
            assert!(
                metrics.get(ArrowPayloadType::ResourceAttrs).is_some(),
                "RESOURCE_ATTRS (shared) should be recovered for Metrics"
            );
            assert!(
                metrics.get(ArrowPayloadType::ScopeAttrs).is_some(),
                "SCOPE_ATTRS (shared) should be recovered for Metrics"
            );
        } else {
            panic!("Expected Metrics variant");
        }
    }

    #[test]
    fn test_logs_bundle_with_shared_slots_roundtrip() {
        // Logs bundle with shared slots
        let mut logs = Logs::default();
        logs.set(ArrowPayloadType::Logs, create_test_batch());
        logs.set(ArrowPayloadType::LogAttrs, create_test_batch());
        logs.set(ArrowPayloadType::ResourceAttrs, create_test_batch()); // shared
        logs.set(ArrowPayloadType::ScopeAttrs, create_test_batch()); // shared

        let records = OtapArrowRecords::Logs(logs);
        let adapter = OtapRecordBundleAdapter::new(records);

        // Extract slot→batch pairs
        let mut recovered_payloads: HashMap<SlotId, RecordBatch> = HashMap::new();
        for slot_desc in &adapter.descriptor().slots {
            if let Some(payload) = adapter.payload(slot_desc.id) {
                let _ = recovered_payloads.insert(slot_desc.id, payload.batch.clone());
            }
        }

        let signal_type = determine_signal_type(&recovered_payloads).unwrap();
        assert_eq!(signal_type, SignalType::Logs);

        let reconstructed = create_logs_records(&recovered_payloads);

        if let OtapArrowRecords::Logs(logs) = reconstructed {
            assert!(logs.get(ArrowPayloadType::Logs).is_some());
            assert!(logs.get(ArrowPayloadType::LogAttrs).is_some());
            assert!(
                logs.get(ArrowPayloadType::ResourceAttrs).is_some(),
                "RESOURCE_ATTRS (shared) should be recovered for Logs"
            );
            assert!(
                logs.get(ArrowPayloadType::ScopeAttrs).is_some(),
                "SCOPE_ATTRS (shared) should be recovered for Logs"
            );
        } else {
            panic!("Expected Logs variant");
        }
    }

    #[test]
    fn test_bundle_with_only_shared_slots_fails() {
        // Edge case: A bundle that somehow only has shared slots
        // (shouldn't happen in practice, but we should handle it gracefully)
        let mut payloads = HashMap::new();
        let _ = payloads.insert(
            to_slot_id(SignalType::Logs, ArrowPayloadType::ResourceAttrs),
            create_test_batch(),
        );
        let _ = payloads.insert(
            to_slot_id(SignalType::Logs, ArrowPayloadType::ScopeAttrs),
            create_test_batch(),
        );

        // Should fail to determine signal type since shared slots return None
        let result = determine_signal_type(&payloads);
        assert!(
            result.is_err(),
            "Should fail when only shared slots are present"
        );
    }

    #[test]
    fn test_signal_type_detection_priority() {
        // When multiple signal-specific slots are present, the first one found determines the type
        // In practice, bundles only contain one signal type, but let's verify behavior

        // HashMap iteration order is not guaranteed, but all slots should be from same signal
        let mut payloads = HashMap::new();
        let _ = payloads.insert(
            to_slot_id(SignalType::Traces, ArrowPayloadType::Spans),
            create_test_batch(),
        );
        let _ = payloads.insert(
            to_slot_id(SignalType::Traces, ArrowPayloadType::SpanEvents),
            create_test_batch(),
        );

        let signal = determine_signal_type(&payloads).unwrap();
        assert_eq!(signal, SignalType::Traces);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge case and boundary tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_invalid_slot_ids_return_none() {
        // Test that gap/invalid slot IDs are handled correctly
        // These are slots that don't correspond to any ArrowPayloadType
        let invalid_slots = [
            0,  // UNKNOWN
            3,  // Gap between SCOPE_ATTRS (2) and UNIVARIATE_METRICS (10)
            9,  // Just before metrics range
            27, // Just after METRIC_ATTRS (26)
            29, // Just before LOGS (30)
            32, // Just after LOG_ATTRS (31)
            39, // Just before SPANS (40)
            46, // Just after SPAN_LINK_ATTRS (45)
            50, // Well outside any range
            59, // Just before OTLP range
        ];

        for raw in invalid_slots {
            let slot = SlotId::new(raw);

            // Should not be identified as shared
            if raw != 1 && raw != 2 {
                assert!(!is_shared_slot(slot), "slot {} should not be shared", raw);
            }

            // Should not map to a signal type (except 0 which is UNKNOWN)
            assert!(
                from_slot_id(slot).is_none(),
                "slot {} should return None from from_slot_id",
                raw
            );

            // slot_to_payload_type depends on whether prost accepts the value
            // For truly invalid values (gaps), it should return None
            if raw == 3
                || raw == 9
                || raw == 27
                || raw == 29
                || raw == 32
                || raw == 39
                || raw == 46
                || raw == 50
                || raw == 59
            {
                assert!(
                    slot_to_payload_type(slot).is_none(),
                    "slot {} should return None from slot_to_payload_type",
                    raw
                );
            }
        }
    }

    #[test]
    fn test_adapter_rejects_wrong_signal_slots() {
        // A Traces adapter should reject Logs and Metrics slots
        let mut traces = Traces::default();
        traces.set(ArrowPayloadType::Spans, create_test_batch());

        let records = OtapArrowRecords::Traces(traces);
        let adapter = OtapRecordBundleAdapter::new(records);

        // Should reject Logs slot
        let logs_slot = to_slot_id(SignalType::Logs, ArrowPayloadType::Logs);
        assert!(
            adapter.payload(logs_slot).is_none(),
            "Traces adapter should reject Logs slot"
        );

        // Should reject Metrics slot
        let metrics_slot = to_slot_id(SignalType::Metrics, ArrowPayloadType::UnivariateMetrics);
        assert!(
            adapter.payload(metrics_slot).is_none(),
            "Traces adapter should reject Metrics slot"
        );

        // Should accept its own Traces slot
        let traces_slot = to_slot_id(SignalType::Traces, ArrowPayloadType::Spans);
        assert!(
            adapter.payload(traces_slot).is_some(),
            "Traces adapter should accept Traces slot"
        );
    }

    #[test]
    fn test_shared_slots_work_for_all_signal_adapters() {
        // RESOURCE_ATTRS and SCOPE_ATTRS should be accessible from any signal adapter
        // that has them set

        // Test with Logs
        let mut logs = Logs::default();
        logs.set(ArrowPayloadType::Logs, create_test_batch());
        logs.set(ArrowPayloadType::ResourceAttrs, create_test_batch());
        let logs_adapter = OtapRecordBundleAdapter::new(OtapArrowRecords::Logs(logs));

        let resource_slot = to_slot_id(SignalType::Logs, ArrowPayloadType::ResourceAttrs);
        assert!(
            logs_adapter.payload(resource_slot).is_some(),
            "Logs adapter should return RESOURCE_ATTRS"
        );

        // Test with Traces
        let mut traces = Traces::default();
        traces.set(ArrowPayloadType::Spans, create_test_batch());
        traces.set(ArrowPayloadType::ResourceAttrs, create_test_batch());
        let traces_adapter = OtapRecordBundleAdapter::new(OtapArrowRecords::Traces(traces));

        assert!(
            traces_adapter.payload(resource_slot).is_some(),
            "Traces adapter should return RESOURCE_ATTRS"
        );

        // Test with Metrics
        let mut metrics = Metrics::default();
        metrics.set(ArrowPayloadType::UnivariateMetrics, create_test_batch());
        metrics.set(ArrowPayloadType::ResourceAttrs, create_test_batch());
        let metrics_adapter = OtapRecordBundleAdapter::new(OtapArrowRecords::Metrics(metrics));

        assert!(
            metrics_adapter.payload(resource_slot).is_some(),
            "Metrics adapter should return RESOURCE_ATTRS"
        );
    }

    #[test]
    fn test_empty_bundle_conversion_fails() {
        // Empty payloads should fail conversion
        let payloads: HashMap<SlotId, RecordBatch> = HashMap::new();

        // Wrap in a mock ReconstructedBundle-like structure
        let result = determine_signal_type(&payloads);
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_with_no_tables() {
        // An adapter with no tables set should have empty descriptor
        let logs = Logs::default();
        let records = OtapArrowRecords::Logs(logs);
        let adapter = OtapRecordBundleAdapter::new(records);

        assert_eq!(adapter.descriptor().slots.len(), 0);
    }

    #[test]
    fn test_slot_id_boundary_values() {
        // Test the boundary values for each signal type range
        let boundary_cases = [
            // (slot_raw, expected_signal, expected_payload)
            (
                10,
                Some(SignalType::Metrics),
                ArrowPayloadType::UnivariateMetrics,
            ),
            (26, Some(SignalType::Metrics), ArrowPayloadType::MetricAttrs),
            (30, Some(SignalType::Logs), ArrowPayloadType::Logs),
            (31, Some(SignalType::Logs), ArrowPayloadType::LogAttrs),
            (40, Some(SignalType::Traces), ArrowPayloadType::Spans),
            (
                45,
                Some(SignalType::Traces),
                ArrowPayloadType::SpanLinkAttrs,
            ),
        ];

        for (raw, expected_signal, expected_payload) in boundary_cases {
            let slot = SlotId::new(raw);

            if let Some(expected) = expected_signal {
                let (signal, payload) = from_slot_id(slot)
                    .unwrap_or_else(|| panic!("slot {} should map to {:?}", raw, expected));
                assert_eq!(signal, expected, "signal mismatch for slot {}", raw);
                assert_eq!(
                    payload, expected_payload,
                    "payload mismatch for slot {}",
                    raw
                );
            }
        }
    }

    #[test]
    fn test_otlp_slots_are_not_arrow_slots() {
        // OTLP slots (60-62) should not be treated as Arrow payload slots
        for raw in [60, 61, 62] {
            let slot = SlotId::new(raw);

            // Should be identified as OTLP slot
            assert!(is_otlp_slot(slot).is_some(), "slot {} should be OTLP", raw);

            // Should NOT be an Arrow payload type
            assert!(
                slot_to_payload_type(slot).is_none(),
                "slot {} should not be Arrow payload",
                raw
            );

            // Should NOT return signal type from from_slot_id
            assert!(
                from_slot_id(slot).is_none(),
                "slot {} should not map via from_slot_id",
                raw
            );

            // Should NOT be a shared slot
            assert!(!is_shared_slot(slot), "slot {} should not be shared", raw);
        }
    }
}
