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
//! - Slots 0-45: Direct ArrowPayloadType values (shared between signal types)
//! - Slot 60: OTLP Logs opaque binary storage  
//! - Slot 61: OTLP Traces opaque binary storage
//! - Slot 62: OTLP Metrics opaque binary storage
//!
//! The signal type is determined by which ArrowPayloadType values are present:
//! - LOGS (30), LOG_ATTRS (31) => Logs signal
//! - SPANS (40), SPAN_* => Traces signal
//! - UNIVARIATE_METRICS (10), etc. => Metrics signal

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

/// Convert payload type to a slot ID (direct mapping)
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

/// Convert a slot ID back to signal type and payload type (Arrow format only)
fn from_slot_id(slot: SlotId) -> Option<(SignalType, ArrowPayloadType)> {
    let raw = slot.raw();

    // Check if it's an OTLP slot
    if raw >= otlp_slots::OTLP_LOGS {
        return None;
    }

    // Convert directly back to ArrowPayloadType
    let payload_type = ArrowPayloadType::try_from(raw as i32).ok()?;

    // Determine signal type from the payload type value
    let signal_type = match raw {
        30..=31 => SignalType::Logs,    // LOGS (30), LOG_ATTRS (31)
        40..=45 => SignalType::Traces,  // SPANS (40) through SPAN_LINK_ATTRS (45)
        10..=26 => SignalType::Metrics, // UNIVARIATE_METRICS (10) through METRIC_ATTRS (26)
        1..=2 => SignalType::Logs,      // RESOURCE_ATTRS (1), SCOPE_ATTRS (2) - default to Logs
        _ => SignalType::Logs,          // Unknown - default to Logs
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
        let (signal_type, payload_type) = from_slot_id(slot)?;

        // Only return payload if signal type matches
        if signal_type != self.signal_type {
            return None;
        }

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
        if let Some((SignalType::Logs, payload_type)) = from_slot_id(*slot_id) {
            logs.set(payload_type, batch.clone());
        }
    }

    OtapArrowRecords::Logs(logs)
}

/// Create Traces records from payloads.
fn create_traces_records(payloads: &HashMap<SlotId, RecordBatch>) -> OtapArrowRecords {
    let mut traces = Traces::default();

    for (slot_id, batch) in payloads {
        if let Some((SignalType::Traces, payload_type)) = from_slot_id(*slot_id) {
            traces.set(payload_type, batch.clone());
        }
    }

    OtapArrowRecords::Traces(traces)
}

/// Create Metrics records from payloads.
fn create_metrics_records(payloads: &HashMap<SlotId, RecordBatch>) -> OtapArrowRecords {
    let mut metrics = Metrics::default();

    for (slot_id, batch) in payloads {
        if let Some((SignalType::Metrics, payload_type)) = from_slot_id(*slot_id) {
            metrics.set(payload_type, batch.clone());
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
        // Test that unique payload types roundtrip correctly
        // Note: ResourceAttrs/ScopeAttrs are shared across signal types, so they
        // default to Logs when reverse-mapped. The signal type context is not
        // preserved in the slot ID since we use direct ArrowPayloadType mapping.
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
            // Shared types default to Logs
            (
                SignalType::Logs,
                ArrowPayloadType::ResourceAttrs,
                SignalType::Logs,
            ),
            (
                SignalType::Traces,
                ArrowPayloadType::ResourceAttrs,
                SignalType::Logs,
            ), // Defaults to Logs
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
}
