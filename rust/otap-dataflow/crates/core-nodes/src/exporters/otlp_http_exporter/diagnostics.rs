// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HTTP event schemas and metadata associated with representative failures.

use super::metrics::OtlpHttpExporterErrorType;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_telemetry::export_diagnostics::{
    DiagnosticReport, DiagnosticTracker, ExportErrorKind, ReportKind,
};
use std::fmt::Display;
use std::time::Instant;

#[cfg(test)]
mod tests;

/// Delivery scopes retain protocol metadata alongside the shared tracker.
#[derive(Default)]
pub(super) struct DeliveryDiagnostics {
    signals: [DeliveryDiagnostic; 3],
}

impl DeliveryDiagnostics {
    pub(super) fn signal(&mut self, signal: SignalType) -> &mut DeliveryDiagnostic {
        &mut self.signals[match signal {
            SignalType::Logs => 0,
            SignalType::Metrics => 1,
            SignalType::Traces => 2,
        }]
    }
}

/// Retryability always describes the retained error, including on stale successes.
#[derive(Default)]
pub(super) struct DeliveryDiagnostic {
    tracker: DiagnosticTracker<OtlpHttpExporterErrorType>,
    sample_retryable: bool,
}

impl DeliveryDiagnostic {
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

    pub(super) fn success(
        &mut self,
        started_at: Instant,
        now: Instant,
    ) -> Option<DiagnosticReport<OtlpHttpExporterErrorType>> {
        self.tracker.success(started_at, now)
    }

    pub(super) fn emit(
        &self,
        report: Option<DiagnosticReport<OtlpHttpExporterErrorType>>,
        signal: SignalType,
    ) {
        let Some(report) = report else { return };
        if report.kind == ReportKind::Recovered {
            otel_arrow_dfe_telemetry::otel_export_diagnostic!(
                target: "otel.exporter.otlp_http", level: otel_info,
                name: "otlp.exporter.http.export_recovered", report: &report,
                diagnostic_kind = "recovery", signal = ?signal, stage = "delivery",
                message = "OTLP HTTP export recovered",
                error = report.detail.as_str()
            );
        } else {
            otel_arrow_dfe_telemetry::otel_export_diagnostic!(
                target: "otel.exporter.otlp_http", level: otel_warn,
                name: "otlp.exporter.http.export_error", report: &report,
                diagnostic_kind = diagnostic_kind(report.kind), signal = ?signal, stage = "delivery",
                message = report.detail.as_str(), retryable = self.sample_retryable
            );
        }
    }
}

fn diagnostic_kind(kind: ReportKind) -> &'static str {
    match kind {
        ReportKind::Degraded => "first_failure",
        ReportKind::Summary => "summary",
        ReportKind::Recovered => "recovery",
    }
}

pub(super) fn emit_preparation(
    report: Option<DiagnosticReport<OtlpHttpExporterErrorType>>,
    signal: SignalType,
) {
    if let Some(report) = report {
        otel_arrow_dfe_telemetry::otel_export_diagnostic!(
            target: "otel.exporter.otlp_http", level: otel_warn,
            name: "otlp.exporter.http.preparation_error", report: &report,
            diagnostic_kind = diagnostic_kind(report.kind), signal = ?signal, stage = "preparation",
            message = "Failed to prepare OTLP HTTP export", error = report.detail.as_str()
        );
    }
}

#[derive(Clone, Copy)]
pub(super) enum NotificationOperation {
    Ack,
    Nack,
}

pub(super) fn emit_notification(
    report: Option<DiagnosticReport<ExportErrorKind>>,
    signal: SignalType,
    operation: NotificationOperation,
) {
    if let Some(report) = report {
        let message = match operation {
            NotificationOperation::Ack => "Failed to route the terminal OTLP HTTP Ack notification",
            NotificationOperation::Nack => {
                "Failed to route the terminal OTLP HTTP Nack notification"
            }
        };
        otel_arrow_dfe_telemetry::otel_export_diagnostic!(
            target: "otel.exporter.otlp_http", level: otel_warn,
            name: "otlp.exporter.http.notification_error", report: &report,
            diagnostic_kind = diagnostic_kind(report.kind), signal = ?signal, stage = "notification",
            message = message, error = report.detail.as_str()
        );
    }
}
