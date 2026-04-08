// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parquet-exporter-local batch storage.
//!
//! [`OtapParquetRecords`] mirrors [`OtapArrowRecords`] but wraps the underlying
//! [`RawBatchStore`] directly, so batches can be read and written without OTAP
//! schema validation. This is needed because the parquet exporter legitimately
//! transforms batches in ways that may not conform to the OTAP wire-protocol
//! schema (e.g. widening ID columns to `UInt32` for partition-unique IDs).

use arrow::array::RecordBatch;
use otap_df_pdata::{
    otap::{
        Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces,
        raw_batch_store::{RawLogsStore, RawMetricsStore, RawTracesStore},
    },
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

/// Parquet-exporter batch storage backed by unvalidated [`RawBatchStore`]s.
///
/// Converted from [`OtapArrowRecords`] at the exporter boundary via
/// `From<OtapArrowRecords>`.
#[allow(clippy::large_enum_variant)]
pub enum OtapParquetRecords {
    /// Logs signal batches.
    Logs(RawLogsStore),
    /// Metrics signal batches.
    Metrics(RawMetricsStore),
    /// Traces signal batches.
    Traces(RawTracesStore),
}

impl From<OtapArrowRecords> for OtapParquetRecords {
    fn from(records: OtapArrowRecords) -> Self {
        match records {
            OtapArrowRecords::Logs(l) => Self::Logs(l.into_raw()),
            OtapArrowRecords::Metrics(m) => Self::Metrics(m.into_raw()),
            OtapArrowRecords::Traces(t) => Self::Traces(t.into_raw()),
        }
    }
}

impl OtapParquetRecords {
    /// Get a reference to the batch for the given payload type, if present.
    #[must_use]
    pub fn get(&self, payload_type: ArrowPayloadType) -> Option<&RecordBatch> {
        match self {
            Self::Logs(s) => s.get(payload_type),
            Self::Metrics(s) => s.get(payload_type),
            Self::Traces(s) => s.get(payload_type),
        }
    }

    /// Set the batch for the given payload type without schema validation.
    ///
    /// Callers should only pass types obtained from [`Self::allowed_payload_types()`].
    pub fn set(&mut self, payload_type: ArrowPayloadType, record_batch: RecordBatch) {
        match self {
            Self::Logs(s) => s.set(payload_type, record_batch),
            Self::Metrics(s) => s.set(payload_type, record_batch),
            Self::Traces(s) => s.set(payload_type, record_batch),
        }
    }

    /// Return the list of allowed payload types for this signal.
    #[must_use]
    pub fn allowed_payload_types(&self) -> &'static [ArrowPayloadType] {
        match self {
            Self::Logs(_) => Logs::allowed_payload_types(),
            Self::Metrics(_) => Metrics::allowed_payload_types(),
            Self::Traces(_) => Traces::allowed_payload_types(),
        }
    }
}
