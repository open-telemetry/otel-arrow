// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verify suppression happens before subscribers receive exporter events.

use otel_arrow_dfe_telemetry::export_diagnostics::{DiagnosticTracker, ExportErrorKind};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<(&'static str, &'static str, Level)>>>);

impl<S: Subscriber> Layer<S> for Capture {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();
        self.0
            .lock()
            .expect("capture lock must not be poisoned")
            .push((metadata.name(), metadata.target(), *metadata.level()));
    }
}

/// Scenario: Two subscribers observe a busy outage, a summary, and confirmed recovery.
/// Guarantees: Both receive only bounded reports with the component target and correct severity.
#[test]
fn suppression_precedes_all_subscribers() {
    let first = Capture::default();
    let second = Capture::default();
    let subscriber = tracing_subscriber::registry()
        .with(first.clone())
        .with(second.clone());
    let formats = Cell::new(0);
    tracing::subscriber::with_default(subscriber, || {
        let start = Instant::now();
        let mut tracker = DiagnosticTracker::default();
        for second in 0..=60 {
            for _ in 0..100 {
                otel_arrow_dfe_telemetry::otel_export_diagnostic!(
                    target: "otel.exporter.test",
                    tracker.failure(start + Duration::from_secs(second), ExportErrorKind::Transport, || {
                        formats.set(formats.get() + 1);
                        "connection refused"
                    }),
                    stage = "delivery", signal = "logs"
                );
            }
        }
        otel_arrow_dfe_telemetry::otel_export_diagnostic!(
            target: "otel.exporter.test",
            tracker.success(start + Duration::from_secs(61), start + Duration::from_secs(90)),
            stage = "delivery", signal = "logs"
        );
    });
    let expected = vec![
        (
            "otelcol.node.export.degrade",
            "otel.exporter.test",
            Level::WARN,
        ),
        (
            "otelcol.node.export.report",
            "otel.exporter.test",
            Level::WARN,
        ),
        (
            "otelcol.node.export.resume",
            "otel.exporter.test",
            Level::INFO,
        ),
    ];
    assert_eq!(
        *first.0.lock().expect("capture lock must not be poisoned"),
        expected
    );
    assert_eq!(
        *second.0.lock().expect("capture lock must not be poisoned"),
        expected
    );
    assert_eq!(formats.get(), 2);
}
