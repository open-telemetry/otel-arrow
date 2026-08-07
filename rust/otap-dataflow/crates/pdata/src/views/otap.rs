// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of the Views traits for OTAP Arrow RecordBatches.
//!
//! This provides zero-copy views over OTAP columnar data, abstracting away the internal
//! structure of Arrow RecordBatches. It enables direct iteration over the data using
//! a hierarchical OTLP-like interface (Resource -> Scope -> LogRecord) without exposing
//! the complexity of the raw Arrow batches or requiring conversion to intermediate formats.

use std::borrow::Cow;

pub mod common;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod traces;
#[cfg(test)]
pub(crate) mod transport_guard_test_util;

use crate::error::Result;
use crate::otap::OtapArrowRecords;
use crate::otap::transform::transport_optimize::{RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH};
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::schema::consts;
use crate::views::otap::common::ensure_record_plain_encoded_columns;

pub use logs::{DecodedOtapLogsResources, OtapLogsResourcesView, OtapLogsView};
pub use metrics::{OtapMetricsView, otap_metrics_have_aggregatable_metrics};
pub use traces::OtapTracesView;

const ROOT_ID_COLUMNS: &[&str] = &[consts::ID, RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH];
const PARENT_ID_COLUMNS: &[&str] = &[consts::PARENT_ID];
const ID_AND_PARENT_ID_COLUMNS: &[&str] = &[consts::ID, consts::PARENT_ID];

/// Keeps OTAP Arrow records available after transport-optimized IDs have been decoded.
///
/// OTAP views borrow from decoded records, so callers that only have a borrowed
/// payload can use this wrapper to preserve the original payload. Plain records
/// are borrowed directly; transport-optimized records are cloned and decoded.
///
/// Prefer [`DecodedOtapLogsResources`] when a logs caller only needs
/// resource-level data.
pub struct DecodedOtapArrowRecords<'a> {
    records: Cow<'a, OtapArrowRecords>,
}

impl<'a> DecodedOtapArrowRecords<'a> {
    /// Borrow plain OTAP Arrow records, or clone and decode transport-optimized IDs.
    pub fn clone_and_decode(records: &'a OtapArrowRecords) -> Result<Self> {
        if transport_ids_are_plain(records) {
            return Ok(Self {
                records: Cow::Borrowed(records),
            });
        }

        let mut records = records.clone();
        records.decode_transport_optimized_ids()?;
        Ok(Self {
            records: Cow::Owned(records),
        })
    }

    /// Create a logs view over the decoded records.
    pub fn logs_view(&self) -> Result<OtapLogsView<'_>> {
        OtapLogsView::try_from(self.records.as_ref())
    }

    /// Create a metrics view over the decoded records.
    pub fn metrics_view(&self) -> Result<OtapMetricsView<'_>> {
        OtapMetricsView::try_from(self.records.as_ref())
    }

    #[cfg(test)]
    fn is_borrowed(&self) -> bool {
        matches!(&self.records, Cow::Borrowed(_))
    }

    #[cfg(test)]
    fn is_owned(&self) -> bool {
        matches!(&self.records, Cow::Owned(_))
    }
}

fn transport_ids_are_plain(records: &OtapArrowRecords) -> bool {
    match records {
        OtapArrowRecords::Logs(_) => logs_transport_ids_are_plain(records),
        OtapArrowRecords::Metrics(_) => metrics_transport_ids_are_plain(records),
        OtapArrowRecords::Traces(_) => traces_transport_ids_are_plain(records),
    }
}

fn record_plain_encoded_columns(
    records: &OtapArrowRecords,
    payload_type: ArrowPayloadType,
    columns: &[&str],
) -> bool {
    ensure_record_plain_encoded_columns(records, payload_type, columns).is_ok()
}

fn records_plain_encoded_columns(
    records: &OtapArrowRecords,
    payload_types: &[ArrowPayloadType],
    columns: &[&str],
) -> bool {
    payload_types
        .iter()
        .copied()
        .all(|payload_type| record_plain_encoded_columns(records, payload_type, columns))
}

fn logs_transport_ids_are_plain(records: &OtapArrowRecords) -> bool {
    record_plain_encoded_columns(records, ArrowPayloadType::Logs, ROOT_ID_COLUMNS)
        && records_plain_encoded_columns(
            records,
            &[
                ArrowPayloadType::ResourceAttrs,
                ArrowPayloadType::ScopeAttrs,
                ArrowPayloadType::LogAttrs,
            ],
            PARENT_ID_COLUMNS,
        )
}

fn metrics_transport_ids_are_plain(records: &OtapArrowRecords) -> bool {
    records_plain_encoded_columns(
        records,
        &[
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::MultivariateMetrics,
        ],
        ROOT_ID_COLUMNS,
    ) && records_plain_encoded_columns(
        records,
        &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ],
        PARENT_ID_COLUMNS,
    ) && records_plain_encoded_columns(
        records,
        &[
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
        ],
        ID_AND_PARENT_ID_COLUMNS,
    )
}

fn traces_transport_ids_are_plain(records: &OtapArrowRecords) -> bool {
    record_plain_encoded_columns(records, ArrowPayloadType::Spans, ROOT_ID_COLUMNS)
        && records_plain_encoded_columns(
            records,
            &[
                ArrowPayloadType::ResourceAttrs,
                ArrowPayloadType::ScopeAttrs,
                ArrowPayloadType::SpanAttrs,
                ArrowPayloadType::SpanEventAttrs,
                ArrowPayloadType::SpanLinkAttrs,
            ],
            PARENT_ID_COLUMNS,
        )
        && records_plain_encoded_columns(
            records,
            &[ArrowPayloadType::SpanEvents, ArrowPayloadType::SpanLinks],
            ID_AND_PARENT_ID_COLUMNS,
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otap::OtapBatchStore;
    use crate::record_batch;

    fn metrics_records() -> OtapArrowRecords {
        crate::metrics!(
            (
                UnivariateMetrics,
                ("id", UInt16, [1u16]),
                ("resource.id", UInt16, [1u16]),
                ("scope.id", UInt16, [1u16]),
                ("metric_type", UInt8, [1u8])
            ),
            (
                NumberDataPoints,
                ("id", UInt32, [1u32]),
                ("parent_id", UInt16, [1u16])
            ),
        )
        .into()
    }

    /// Scenario: Clone-and-decode receives OTAP records whose transport IDs are already plain.
    /// Guarantees: The wrapper borrows the records and can construct a metrics view from them.
    #[test]
    fn clone_and_decode_borrows_plain_records() {
        let records = metrics_records();

        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).unwrap();

        assert!(decoded.is_borrowed());
        let _view = decoded.metrics_view().unwrap();
    }

    /// Scenario: Clone-and-decode receives transport-optimized OTAP records.
    /// Guarantees: The wrapper owns a decoded copy that can construct a metrics view.
    #[test]
    fn clone_and_decode_owns_transport_optimized_records() {
        let mut records = metrics_records();

        records.encode_transport_optimized().unwrap();
        let decoded = DecodedOtapArrowRecords::clone_and_decode(&records).unwrap();

        assert!(decoded.is_owned());
        let _view = decoded.metrics_view().unwrap();
    }
}
