// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP diagnostic sampling state.
//!
//! Delivery, preparation, and notification failures use independent bounded
//! trackers. Selected failures are emitted directly at their ordinary callsite.

use super::metrics::OtlpHttpExporterErrorType;
use otel_arrow_dfe_telemetry::diagnostics::{DiagnosticReport, DiagnosticTracker};
use std::time::Instant;

#[cfg(test)]
mod tests;

/// A delivery failure episode for one signal.
#[derive(Default)]
pub(super) struct DeliveryDiagnostic {
    tracker: DiagnosticTracker<OtlpHttpExporterErrorType>,
}

impl DeliveryDiagnostic {
    /// Records a failed export and selects its ordinary warning when due.
    pub(super) fn failure(
        &mut self,
        now: Instant,
        category: OtlpHttpExporterErrorType,
    ) -> Option<DiagnosticReport<OtlpHttpExporterErrorType>> {
        self.tracker.failure(now, category)
    }

    /// Records successful delivery and selects confirmed recovery when due.
    pub(super) fn success(
        &mut self,
        started_at: Instant,
        now: Instant,
    ) -> Option<DiagnosticReport<OtlpHttpExporterErrorType>> {
        self.tracker.success(started_at, now)
    }
}
