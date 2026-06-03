// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains implementation of the Views traits for OTAP Arrow RecordBatches.
//!
//! This provides zero-copy views over OTAP columnar data, abstracting away the internal
//! structure of Arrow RecordBatches. It enables direct iteration over the data using
//! a hierarchical OTLP-like interface (Resource -> Scope -> LogRecord) without exposing
//! the complexity of the raw Arrow batches or requiring conversion to intermediate formats.

pub mod common;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod traces;
#[cfg(test)]
pub(crate) mod transport_guard_test_util;

use crate::error::Result;
use crate::otap::OtapArrowRecords;

pub use logs::{DecodedOtapLogsResources, OtapLogsResourcesView, OtapLogsView};
pub use metrics::{OtapMetricsView, otap_metrics_have_aggregatable_metrics};
pub use traces::OtapTracesView;

/// Owns OTAP Arrow records after transport-optimized IDs have been decoded.
///
/// OTAP views borrow from decoded records, so callers that only have a borrowed
/// payload can use this wrapper to preserve the original payload while keeping
/// the cloned, decoded records alive for view construction.
///
/// Prefer [`DecodedOtapLogsResources`] when a logs caller only needs
/// resource-level data.
pub struct DecodedOtapArrowRecords {
    records: OtapArrowRecords,
}

impl DecodedOtapArrowRecords {
    /// Clone borrowed OTAP Arrow records and decode transport-optimized IDs.
    pub fn clone_and_decode(records: &OtapArrowRecords) -> Result<Self> {
        let mut records = records.clone();
        records.decode_transport_optimized_ids()?;
        Ok(Self { records })
    }

    /// Create a logs view over the decoded records.
    pub fn logs_view(&self) -> Result<OtapLogsView<'_>> {
        OtapLogsView::try_from(&self.records)
    }

    /// Create a metrics view over the decoded records.
    pub fn metrics_view(&self) -> Result<OtapMetricsView<'_>> {
        OtapMetricsView::try_from(&self.records)
    }
}
