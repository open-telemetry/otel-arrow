// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Opt-in compute-duration timing for processors.
//!
//! Processors that perform meaningful synchronous compute can add a
//! [`ComputeDuration`] field and call [`ComputeDuration::timed`] to
//! measure the wall-clock duration of that work.  Timing is gated on
//! the `COMPONENT_DURATION` interest at the detailed metric level or through
//! the node's `policies.telemetry.duration` opt-in.
//!
//! Duration is grouped by outcome so operators can distinguish compute time
//! from error-path time without defining separate instruments.
//!
//! This API complements, but is not required for, the engine's automatic
//! per-message flow_metric. [`ComputeDuration::timed`] provides the
//! outcome split for `processor.compute.duration`,
//! while the engine's `Instant`-marker timing on the EffectHandler captures
//! total wall-clock compute between sends for flow_metrics without processor
//! cooperation.
//!
//! The closure-based API structurally prevents timing from spanning
//! `.await` points.

use std::cell::RefCell;

use crate::Interests;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::instrument::{HistogramNormal, Timer};
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::context::PipelineContext;

/// Metric set containing processor compute duration grouped by outcome.
#[metric_set(
    name = "processor.compute",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ComputeDurationMetrics {
    /// Wall-clock duration of processor-local synchronous compute.
    #[metric(unit = "s")]
    pub duration: HistogramNormal,
}

/// Wrapper providing interests-gated duration recording and reporting.
pub struct ComputeDuration {
    metrics: MeasurementMetricSet<ComputeDurationMetrics>,
    /// Accumulator for successful durations.
    /// Uses `RefCell` for interior mutability so `timed` can take `&self`,
    /// allowing callers to hold shared borrows of sibling fields in the
    /// closure.
    acc_success: RefCell<HistogramNormal>,
    /// Accumulator for failed durations.
    acc_failed: RefCell<HistogramNormal>,
}

impl ComputeDuration {
    /// Register a new compute-duration metric set on the given pipeline.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            metrics: ComputeDurationMetrics::register(pipeline_ctx),
            acc_success: RefCell::new(HistogramNormal::default()),
            acc_failed: RefCell::new(HistogramNormal::default()),
        }
    }

    /// Time a synchronous, fallible closure for the process-duration outcome
    /// split if interests includes `COMPONENT_DURATION`, otherwise just call
    /// `f` directly.
    ///
    /// The elapsed time is recorded into the `success` or `failed`
    /// accumulator based on the closure's `Result` outcome. This feeds only
    /// the `processor.compute.duration` metric; flow_metric
    /// participation is handled separately by the engine's `Instant`-marker
    /// timing on the EffectHandler.
    ///
    /// The closure-based API structurally prevents the timer from
    /// being held across `.await` -- the closure is `FnOnce`, not
    /// async, so the compiler enforces that only synchronous work is
    /// measured.
    #[inline]
    pub fn timed<T, E>(
        &self,
        interests: Interests,
        f: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        if interests.contains(Interests::COMPONENT_DURATION) {
            let timer = Timer::start();
            let result = f();
            let elapsed_seconds = timer.elapsed_nanos() / 1e9;
            let acc = if result.is_ok() {
                &self.acc_success
            } else {
                &self.acc_failed
            };
            acc.borrow_mut().record(elapsed_seconds);
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
        let success = self.acc_success.replace(HistogramNormal::default());
        self.metrics
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .duration
            .merge(success);
        let failed = self.acc_failed.replace(HistogramNormal::default());
        self.metrics
            .with(OutcomeAttributes {
                outcome: Outcome::Failure,
            })
            .duration
            .merge(failed);
        let _ = reporter.report_measurement(&mut self.metrics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_pipeline_ctx;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    /// Scenario: timed processor compute succeeds twice and fails once.
    /// Guarantees: duration observations are accumulated in the corresponding outcome buckets.
    #[test]
    fn timed_splits_by_outcome() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);
        let active = Interests::COMPONENT_DURATION;

        // Two Ok results and one Err.
        let _ = cd.timed(active, || Ok::<_, &str>(std::hint::black_box(42)));
        let _ = cd.timed(active, || Ok::<_, &str>(std::hint::black_box(43)));
        let _ = cd.timed(active, || Err::<i32, _>("fail"));

        let (success_count, _, success_min, _) = cd.acc_success.borrow().get().summary();
        assert_eq!(success_count, 2);
        assert!(success_min >= 0.0);

        let (failed_count, _, failed_min, _) = cd.acc_failed.borrow().get().summary();
        assert_eq!(failed_count, 1);
        assert!(failed_min >= 0.0);
    }

    /// Scenario: processor compute timing runs without the component-duration interest.
    /// Guarantees: the closure executes without recording any duration observations.
    #[test]
    fn timed_noop_when_disabled() {
        let (ctx, _) = test_pipeline_ctx();
        let cd = ComputeDuration::new(&ctx);

        let _ = cd.timed(Interests::empty(), || Ok::<_, &str>(1));
        let _ = cd.timed(Interests::empty(), || Err::<i32, _>("fail"));

        assert_eq!(cd.acc_success.borrow().get().count(), 0);
        assert_eq!(cd.acc_failed.borrow().get().count(), 0);
    }

    /// Scenario: successful and failed processor compute observations are reported.
    /// Guarantees: one seconds-based processor.compute.duration histogram is emitted with bounded outcome dimensions.
    #[test]
    fn report_emits_expected_metric_names() {
        let (ctx, _) = test_pipeline_ctx();
        let mut cd = ComputeDuration::new(&ctx);
        let active = Interests::COMPONENT_DURATION;

        let _ = cd.timed(active, || Ok::<_, &str>(1));
        let _ = cd.timed(active, || Err::<i32, _>("fail"));

        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(4);
        cd.report(&mut reporter);
        let mut snapshots: Vec<_> = rx.try_iter().collect();
        snapshots.sort_by_key(|snapshot| {
            snapshot
                .measurement_attribute_value("outcome")
                .unwrap_or_default()
        });

        assert_eq!(snapshots.len(), 2);
        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| snapshot.measurement_attribute_value("outcome"))
                .collect::<Vec<_>>(),
            vec![Some("failure"), Some("success")]
        );
        for snapshot in snapshots {
            assert_eq!(snapshot.descriptor().name, "processor.compute");
            assert_eq!(snapshot.descriptor().metrics.len(), 1);
            assert_eq!(snapshot.descriptor().metrics[0].name, "duration");
            assert_eq!(snapshot.descriptor().metrics[0].unit, "s");
        }
    }
}
