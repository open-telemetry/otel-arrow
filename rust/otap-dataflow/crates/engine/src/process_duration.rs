// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in compute-duration timing for processors.
//!
//! Processors that perform meaningful synchronous compute can add a
//! [`ComputeDuration`] field and call [`ComputeDuration::timed`] to
//! measure the wall-clock duration of that work.  Timing is gated on
//! the `PROCESS_DURATION` interest at the normal metric level.
//!
//! Duration is split by outcome: successful and failed operations are
//! recorded into separate instruments so operators can distinguish
//! compute time from error-path time.
//!
//! This API complements, but is not required for, the engine's automatic
//! per-message flow_metric. [`ComputeDuration::timed`] provides the
//! success/failed outcome split for `processor.compute.{success,failed}.duration`,
//! while the engine's `Instant`-marker timing on the EffectHandler captures
//! total wall-clock compute between sends for flow_metrics without processor
//! cooperation.
//!
//! The closure-based API structurally prevents timing from spanning
//! `.await` points.

use std::cell::Cell;

use crate::Interests;
use otap_df_telemetry::instrument::{Mmsc, Timer};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

use crate::context::PipelineContext;

/// Metric set containing compute-duration instruments split by outcome.
#[metric_set(name = "processor.compute")]
#[derive(Debug, Default, Clone)]
pub struct ComputeDurationMetrics {
    /// Wall-clock duration of successful compute sections, in nanoseconds.
    #[metric(name = "compute.success.duration", unit = "ns")]
    pub compute_duration_success: Mmsc,
    /// Wall-clock duration of failed compute sections, in nanoseconds.
    #[metric(name = "compute.failed.duration", unit = "ns")]
    pub compute_duration_failed: Mmsc,
}

/// Wrapper providing interests-gated duration recording and reporting.
pub struct ComputeDuration {
    metrics: MetricSet<ComputeDurationMetrics>,
    /// Accumulator for successful durations.
    /// Uses `Cell` for interior mutability so `timed` can take `&self`,
    /// allowing callers to hold shared borrows of sibling fields in the
    /// closure.  `Cell` works here because `Mmsc` is `Copy`.
    acc_success: Cell<Mmsc>,
    /// Accumulator for failed durations.
    acc_failed: Cell<Mmsc>,
}

impl ComputeDuration {
    /// Register a new compute-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: pipeline_ctx.register_metrics::<ComputeDurationMetrics>(),
            acc_success: Cell::new(Mmsc::default()),
            acc_failed: Cell::new(Mmsc::default()),
        }
    }

    /// Time a synchronous, fallible closure for the process-duration outcome
    /// split if interests includes `PROCESS_DURATION`, otherwise just call
    /// `f` directly.
    ///
    /// The elapsed time is recorded into the `success` or `failed`
    /// accumulator based on the closure's `Result` outcome. This feeds only
    /// the `processor.compute.{success,failed}.duration` metric; flow_metric
    /// participation is handled separately by the engine's `Instant`-marker
    /// timing on the EffectHandler.
    ///
    /// The closure-based API structurally prevents the timer from
    /// being held across `.await` — the closure is `FnOnce`, not
    /// async, so the compiler enforces that only synchronous work is
    /// measured.
    #[inline]
    pub fn timed<T, E>(
        &self,
        interests: Interests,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        if interests.contains(Interests::PROCESS_DURATION) {
            let timer = Timer::start();
            let result = f();
            let elapsed = timer.elapsed_nanos();
            let acc = if result.is_ok() {
                &self.acc_success
            } else {
                &self.acc_failed
            };
            let mut val = acc.get();
            val.record(elapsed);
            acc.set(val);
            result
        } else {
            f()
        }
    }

    /// Report accumulated duration metrics to the collector.
    ///
    /// Drains both accumulators into the metric set, then reports
    /// and resets as usual.
    pub fn report(&mut self, reporter: &mut MetricsReporter) {
        let success = self.acc_success.replace(Mmsc::default());
        self.metrics.compute_duration_success.merge(success);
        let failed = self.acc_failed.replace(Mmsc::default());
        self.metrics.compute_duration_failed.merge(failed);
        let _ = reporter.report(&mut self.metrics);
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::testing::test_pipeline_ctx;
    use otap_df_telemetry::reporter::MetricsReporter;

    /// timed() routes duration into success/failed accumulators based on outcome.
    #[test]
    fn timed_splits_by_outcome() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let active = Interests::PROCESS_DURATION;

        // Two Ok results and one Err.
        let _ = cd.timed(active, || Ok::<_, &str>(std::hint::black_box(42)));
        let _ = cd.timed(active, || Ok::<_, &str>(std::hint::black_box(43)));
        let _ = cd.timed(active, || Err::<i32, _>("fail"));

        let success_snap = cd.acc_success.get().get();
        assert_eq!(success_snap.count, 2);
        assert!(success_snap.min >= 0.0);

        let failed_snap = cd.acc_failed.get().get();
        assert_eq!(failed_snap.count, 1);
        assert!(failed_snap.min >= 0.0);
    }

    /// timed() is a no-op when interests are disabled.
    #[test]
    fn timed_noop_when_disabled() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);

        let _ = cd.timed(Interests::empty(), || Ok::<_, &str>(1));
        let _ = cd.timed(Interests::empty(), || Err::<i32, _>("fail"));

        assert_eq!(cd.acc_success.get().get().count, 0);
        assert_eq!(cd.acc_failed.get().get().count, 0);
    }

    /// End-to-end: report() drains accumulators into the registry under
    /// the expected metric set and field names.
    #[test]
    fn report_emits_expected_metric_names() {
        let (ctx, registry) = test_pipeline_ctx();
        let mut cd = ComputeDuration::new(&ctx);
        let active = Interests::PROCESS_DURATION;

        let _ = cd.timed(active, || Ok::<_, &str>(1));
        let _ = cd.timed(active, || Err::<i32, _>("fail"));

        // report() sends a snapshot through the channel; drain it back
        // into the registry so visit_current_metrics can see the values.
        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
        cd.report(&mut reporter);
        while let Ok(snapshot) = rx.try_recv() {
            registry.accumulate_metric_set_snapshot(snapshot.key(), snapshot.get_metrics());
        }

        let mut found_set = false;
        let mut field_names = Vec::new();
        registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "processor.compute" {
                found_set = true;
                for (field, _value) in iter {
                    field_names.push(field.name.to_owned());
                }
            }
        });

        assert!(found_set, "metric set 'processor.compute' not found");
        assert!(
            field_names.contains(&"compute.success.duration".to_owned()),
            "missing compute.success.duration, found: {field_names:?}"
        );
        assert!(
            field_names.contains(&"compute.failed.duration".to_owned()),
            "missing compute.failed.duration, found: {field_names:?}"
        );
    }
}
