// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end episode start/end cost with WARN and INFO enabled and a no-op subscriber.
//! Both variants perform identical observation, capture, counting and field encoding;
//! only the end's saved-event representation differs. No console or channel is timed.
//!
//! Run: `cargo bench -p otel-arrow-dfe-telemetry --features testing --bench diagnostics`.
//! Each iteration is one WARN start plus one INFO recovery, not one log event.
//!
//! Example results on an Intel Core Ultra 7 165H, Linux x86_64, pinned to CPU 2:
//!
//! | Input bytes | Map, us/pair (95% CI) | Escaped text, us/pair (95% CI) |
//! | --- | --- | --- |
//! | 64 | 1.774 (1.598-1.940) | 4.932 (4.687-5.237) |
//! | 256 | 2.300 (2.119-2.571) | 8.700 (8.126-9.487) |
//! | 1024 | 2.704 (2.491-2.926) | 25.648 (24.437-27.128) |
//! | 4096 | 2.857 (2.697-3.015) | 39.446 (35.674-42.833) |
//!
//! This was a shared, busy host: absolute timings varied substantially between
//! runs. The map path was faster at every tested size, but this is not a claim
//! about exporter throughput. Both paths are bounded and may truncate differently.
//! The map path avoids the escaped path's intermediate formatted String.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_telemetry::diagnostics::{
    DiagnosticEmission, DiagnosticErrorKind, EpisodeSampler,
};
use otel_arrow_dfe_telemetry::self_tracing::{LogContext, format_log_record_to_string};
use otel_arrow_dfe_telemetry::tracing_init::StructuredLogEmitter;
use otel_arrow_dfe_telemetry::{Logger, Sampler};
use std::hint::black_box;
use std::time::{Duration, Instant};
use tracing::{Dispatch, Event, Metadata};
use tracing_subscriber::{filter::LevelFilter, prelude::*};

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = "urn:otel:exporter:diagnostic_bench",
    target = "otel.exporter.diagnostic_bench",
);

struct Measured<'a, const ESCAPED: bool>(DiagnosticEmission<'a, DiagnosticErrorKind>);

impl<const ESCAPED: bool> Sampler for Measured<'_, ESCAPED> {
    fn should_sample(&mut self, metadata: &Metadata<'_>) -> bool {
        self.0.should_sample(metadata)
    }
}

impl<const ESCAPED: bool> Logger for Measured<'_, ESCAPED> {
    fn emit(&mut self, event: &Event<'_>, dispatch: &Dispatch) {
        let record = self
            .0
            .encode_with_snapshot(event, LogContext::new(), |visitor, saved| {
                if ESCAPED {
                    let formatted = format_log_record_to_string(None, saved);
                    visitor.write_display("episode.start_event", &formatted.escape_debug());
                } else {
                    visitor.record_log_record("episode.start_event", saved);
                }
            });
        // The subscriber intentionally ignores events; keep encoding observable.
        let _ = black_box(record);
        dispatch.event(event);
    }
}

fn completion<const ESCAPED: bool>(
    state: &mut EpisodeSampler<DiagnosticErrorKind>,
    started: Instant,
    now: Instant,
    result: Result<(), DiagnosticErrorKind>,
    message: &str,
) {
    if tracing::Level::WARN <= tracing::level_filters::STATIC_MAX_LEVEL {
        let emission = state.observe(result, started, now);
        if emission.is_recovery() {
            otel_info!(
                logger: Measured::<ESCAPED>(emission),
                "bench.recovered",
                signal = "logs",
                message = "export recovered"
            );
        } else if emission.report().is_some() {
            otel_warn!(
                logger: Measured::<ESCAPED>(emission),
                "bench.failure",
                signal = "logs",
                retryable = true,
                message = %message
            );
        }
    }
}

fn episode_logging(c: &mut Criterion) {
    let subscriber = tracing_subscriber::registry().with(LevelFilter::INFO);
    tracing::subscriber::with_default(subscriber, || {
        let mut group = c.benchmark_group("episode_logging");
        _ = group.sample_size(40);
        _ = group.warm_up_time(Duration::from_secs(1));
        _ = group.measurement_time(Duration::from_secs(2));
        let start = Instant::now();
        for size in [64, 256, 1024, 4096] {
            let prefix = "connection failed\n";
            let message = format!("{prefix}{}", "x".repeat(size - prefix.len()));
            assert_eq!(message.len(), size);
            _ = group.throughput(Throughput::Elements(2));
            let mut state = EpisodeSampler::new(StructuredLogEmitter::default());
            _ = group.bench_with_input(BenchmarkId::new("map", size), &message, |b, text| {
                b.iter(|| {
                    completion::<false>(
                        &mut state,
                        start,
                        start,
                        Err(DiagnosticErrorKind::Transport),
                        black_box(text),
                    );
                    completion::<false>(
                        &mut state,
                        start + Duration::from_secs(1),
                        start + Duration::from_secs(30),
                        Ok(()),
                        "",
                    );
                });
            });
            let mut state = EpisodeSampler::new(StructuredLogEmitter::default());
            _ = group.bench_with_input(BenchmarkId::new("escaped", size), &message, |b, text| {
                b.iter(|| {
                    completion::<true>(
                        &mut state,
                        start,
                        start,
                        Err(DiagnosticErrorKind::Transport),
                        black_box(text),
                    );
                    completion::<true>(
                        &mut state,
                        start + Duration::from_secs(1),
                        start + Duration::from_secs(30),
                        Ok(()),
                        "",
                    );
                });
            });
        }
        group.finish();
    });
}

criterion_group!(benches, episode_logging);
criterion_main!(benches);
