// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP event schemas and metadata associated with representative failures.
//!
//! The shared tracker owns suppression, counters, and recovery timing. This
//! adapter preserves HTTP event names and attributes, including retryability
//! associated with the retained error sample. Delivery is observed before
//! Ack/Nack routing; preparation and notification failures use separate trackers.

use super::metrics::OtlpHttpExporterErrorType;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_telemetry::attributes::AttributeEnum;
use otel_arrow_dfe_telemetry::diagnostics::{
    DiagnosticErrorKind, DiagnosticReport, DiagnosticTracker, ReportKind,
};
use std::fmt::Display;
use std::time::Instant;

#[cfg(test)]
mod tests;

/// A delivery episode and its HTTP-specific sample metadata for one signal.
///
/// Retryability describes the representative error selected by the tracker,
/// which may differ from the most recent failure or completion.
#[derive(Default)]
pub(super) struct DeliveryDiagnostic {
    /// Shared episode state, counters, and bounded representative error.
    tracker: DiagnosticTracker<OtlpHttpExporterErrorType>,
    /// Nack retry decision for that error; meaningful after selecting a failure.
    sample_retryable: bool,
}

impl DeliveryDiagnostic {
    /// Records a failed export at its monotonic completion time, `now`.
    ///
    /// `retryable` must use the same authentication-aware decision as Nack
    /// routing. The tracker evaluates `detail` only when selecting a first
    /// failure or summary; only then is its matching retryability replaced.
    /// Suppressed failures return `None` and leave both sample values intact.
    pub(super) fn failure<D: Display>(
        &mut self,
        now: Instant,
        category: OtlpHttpExporterErrorType,
        retryable: bool,
        detail: impl FnOnce() -> D,
    ) -> Option<DiagnosticReport<OtlpHttpExporterErrorType>> {
        let report = self.tracker.failure(now, category, detail);
        if report.is_some() {
            self.sample_retryable = retryable;
        }
        report
    }

    /// Records successful delivery and selects a summary or recovery if due.
    ///
    /// `started_at` is the export attempt's start, and `now` is its completion.
    /// Recovery requires the shared failure-free interval and an attempt begun
    /// after the latest failure. An older success can summarize unreported
    /// failures but cannot confirm recovery. Success-triggered reports reuse
    /// the last sampled error; successes do not replace its retryability.
    /// `None` means no diagnostic is due.
    pub(super) fn success(
        &mut self,
        started_at: Instant,
        now: Instant,
    ) -> Option<DiagnosticReport<OtlpHttpExporterErrorType>> {
        self.tracker.success(started_at, now)
    }

    /// Emits a selected delivery report using the HTTP event contract.
    ///
    /// Warnings retain `export_error` with a string `message` and boolean
    /// `retryable` describing the sample. Recovery uses `export_recovered` at
    /// INFO, retains the error sample, and omits `retryable`. Operation fields
    /// precede shared counters so bounded ITS encoding preserves the error.
    /// `None` emits nothing, keeping suppression ahead of all log subscribers.
    ///
    /// Emit immediately after observation, before recording another completion,
    /// so the report and stored sample retryability remain paired. `signal`
    /// must identify the scope from which this report was selected.
    pub(super) fn emit(
        &self,
        report: Option<DiagnosticReport<OtlpHttpExporterErrorType>>,
        signal: SignalType,
    ) {
        let Some(report) = report else { return };
        if report.kind == ReportKind::Recovered {
            otel_arrow_dfe_telemetry::otel_diagnostic_report!(
                target: "otel.exporter.otlp_http", emit: otel_info,
                name: "otlp.exporter.http.export_recovered", report: &report,
                error = report.detail.as_str(), signal = signal.as_str(),
                diagnostic_kind = "recovery"
            );
        } else {
            otel_arrow_dfe_telemetry::otel_diagnostic_report!(
                target: "otel.exporter.otlp_http", emit: otel_warn,
                name: "otlp.exporter.http.export_error", report: &report,
                signal = signal.as_str(), retryable = self.sample_retryable,
                diagnostic_kind = diagnostic_kind(report.kind),
                message = %report.detail.as_str()
            );
        }
    }
}

/// Maps a shared report kind to the HTTP `diagnostic_kind` attribute value.
fn diagnostic_kind(kind: ReportKind) -> &'static str {
    match kind {
        ReportKind::Degraded => "first_failure",
        ReportKind::Summary => "summary",
        ReportKind::Recovered => "recovery",
    }
}

/// Emits a WARN selected by the independent encoding/compression failure tracker.
///
/// The report's bounded sample becomes `error`; the event name supplies the
/// operation context without a redundant message or stage. Pass `None` for
/// suppressed failures. Preparation reports do not participate in delivery
/// recovery and make no claim about destination availability.
pub(super) fn emit_preparation(
    report: Option<DiagnosticReport<OtlpHttpExporterErrorType>>,
    signal: SignalType,
) {
    if let Some(report) = report {
        otel_arrow_dfe_telemetry::otel_diagnostic_report!(
            target: "otel.exporter.otlp_http", emit: otel_warn,
            name: "otlp.exporter.http.preparation_error", report: &report,
            error = report.detail.as_str(), signal = signal.as_str(),
            diagnostic_kind = diagnostic_kind(report.kind)
        );
    }
}

/// Upstream notification whose routing failure supplied the report's sample.
#[derive(Clone, Copy)]
pub(super) enum NotificationOperation {
    /// Acknowledgement of a successful export.
    Ack,
    /// Negative acknowledgement of a failed or refused export.
    Nack,
}

/// Emits a WARN selected by the independent Ack/Nack notification failure tracker.
///
/// `operation` must match the sampled failure and is emitted as a compact
/// bounded attribute alongside `error`. `None` emits nothing. Notification
/// reporting does not change the recorded delivery result or its recovery state.
pub(super) fn emit_notification(
    report: Option<DiagnosticReport<DiagnosticErrorKind>>,
    signal: SignalType,
    operation: NotificationOperation,
) {
    if let Some(report) = report {
        let operation = match operation {
            NotificationOperation::Ack => "ack",
            NotificationOperation::Nack => "nack",
        };
        otel_arrow_dfe_telemetry::otel_diagnostic_report!(
            target: "otel.exporter.otlp_http", emit: otel_warn,
            name: "otlp.exporter.http.notification_error", report: &report,
            error = report.detail.as_str(), operation = operation,
            signal = signal.as_str(), diagnostic_kind = diagnostic_kind(report.kind)
        );
    }
}
