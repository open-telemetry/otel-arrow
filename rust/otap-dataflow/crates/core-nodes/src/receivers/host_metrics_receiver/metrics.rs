// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the host metrics receiver node.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ReceiverPullMetrics;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Bounded classification for errors encountered during host metrics scraping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum HostMetricsErrorType {
    /// Scraper background worker stopped or failed.
    ScraperFailure,
    /// Source read error skipped because other families succeeded.
    PartialError,
    /// Source read error seen during scrapes.
    SourceReadError,
    /// Downstream send error or channel failure.
    SendFailure,
}

/// Attributes for error metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct HostMetricsErrorAttributes {
    /// Classification of the error.
    #[attribute_key = "error.type"]
    pub error_type: HostMetricsErrorType,
}

/// Error counters grouped by error type.
#[metric_set(
    name = "receiver.host_metrics.errors",
    measurement_attributes = HostMetricsErrorAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct HostMetricsErrorMetrics {
    /// Number of errors seen during scraping and delivery.
    #[metric(unit = "{error}")]
    pub errors: Counter<u64>,
}

/// Non-dimensioned operational and output metrics for the host metrics receiver.
#[metric_set(name = "receiver.host_metrics")]
#[derive(Debug, Default, Clone)]
pub struct HostMetricsGeneralMetrics {
    /// Number of scrape ticks started.
    #[metric(unit = "{scrape}")]
    pub scrapes_started: Counter<u64>,
    /// Number of scheduled scrape ticks skipped because the worker was busy.
    #[metric(unit = "{scrape}")]
    pub scrapes_skipped: Counter<u64>,
    /// Number of due metric families processed.
    #[metric(unit = "{family}")]
    pub families_scraped: Counter<u64>,
    /// Number of batches sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Delay between scheduled and actual scrape start.
    #[metric(unit = "ns")]
    pub scrape_lag_ns: Mmsc,
}

/// Shared bounded-cardinality host metrics receiver metrics tracker.
pub struct HostMetricsReceiverMetrics {
    /// Shared pull boundary tracking collection attempts and duration.
    pub boundary: ReceiverPullMetrics,
    /// Error metrics by classification.
    pub errors: MeasurementMetricSet<HostMetricsErrorMetrics>,
    /// General operational and timing metrics.
    pub general: MetricSet<HostMetricsGeneralMetrics>,
}

impl HostMetricsReceiverMetrics {
    /// Registers the metric sets with the pipeline context.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ReceiverPullMetrics::register(pipeline_ctx),
            errors: HostMetricsErrorMetrics::register(pipeline_ctx),
            general: HostMetricsGeneralMetrics::register(pipeline_ctx),
        }
    }

    /// Records a single occurrence of an error classification.
    pub fn record_error(&mut self, error_type: HostMetricsErrorType) {
        self.errors
            .with(HostMetricsErrorAttributes { error_type })
            .errors
            .inc();
    }

    /// Records multiple occurrences of an error classification.
    pub fn record_error_count(&mut self, error_type: HostMetricsErrorType, count: u64) {
        self.errors
            .with(HostMetricsErrorAttributes { error_type })
            .errors
            .add(count);
    }

    /// Reports touched metric buckets to the given reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.errors)?;
        reporter.report(&mut self.general)?;
        Ok(())
    }

    /// Collects terminal snapshots for all metric sets.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.errors.terminal_snapshots());
        if self.general.needs_flush() {
            snapshots.extend(self.general.terminal_snapshots());
        }
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;

    struct Expected<'a> {
        scope: &'a str,
        instrument: &'a str,
        unit: &'a str,
        attr: Option<(&'a str, &'a str)>,
        value: u64,
    }

    fn assert_snapshot_cases(snapshots: &[MetricSetSnapshot], cases: &[Expected<'_>]) {
        for c in cases {
            let snap = snapshots
                .iter()
                .find(|s| {
                    s.descriptor().name == c.scope
                        && match c.attr {
                            Some((k, v)) => s.measurement_attribute_value(k) == Some(v),
                            None => true,
                        }
                        && s.descriptor()
                            .metrics
                            .iter()
                            .any(|m| m.name == c.instrument)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "snapshot `{}` ({}) with attr {:?} not found",
                        c.scope, c.instrument, c.attr
                    )
                });

            let idx = snap
                .descriptor()
                .metrics
                .iter()
                .position(|m| m.name == c.instrument)
                .unwrap();

            assert_eq!(snap.descriptor().metrics[idx].unit, c.unit);
            assert_eq!(snap.get_metrics()[idx].to_u64_lossy(), c.value);
        }
    }

    /// Scenario: Host metrics receiver metric sets are recorded and drained into
    /// terminal snapshots.
    /// Guarantees: Collection attempts, error categories, and general counters
    /// emit valid snapshots and clear hot values.
    #[test]
    fn test_host_metrics_receiver_metrics_lifecycle() {
        let (pipeline_ctx, _registry) =
            test_pipeline_ctx_with_interests(Interests::NODE_OUTPUT_METRICS);

        let mut metrics = HostMetricsReceiverMetrics::register(&pipeline_ctx);

        // Record accepted scrape attempts through boundary
        let success = metrics.boundary.processing();
        success.success();

        let failure = metrics.boundary.processing();
        drop(failure);

        // Record error categories
        metrics.record_error(HostMetricsErrorType::ScraperFailure);
        metrics.record_error_count(HostMetricsErrorType::PartialError, 3);
        metrics.record_error_count(HostMetricsErrorType::SourceReadError, 2);
        metrics.record_error(HostMetricsErrorType::SendFailure);

        // Record general counters and timings
        metrics.general.scrapes_started.add(5);
        metrics.general.scrapes_skipped.add(1);
        metrics.general.families_scraped.add(9);
        metrics.general.batches_sent.add(4);
        metrics.general.scrape_lag_ns.record(250_000.0);

        let snapshots = metrics.terminal_snapshots();
        assert!(
            !snapshots.is_empty(),
            "terminal snapshots should contain recorded metric sets"
        );

        let cases = [
            Expected {
                scope: "receiver.pulled",
                instrument: "attempts",
                unit: "{attempt}",
                attr: Some(("outcome", "success")),
                value: 1,
            },
            Expected {
                scope: "receiver.pulled",
                instrument: "attempts",
                unit: "{attempt}",
                attr: Some(("outcome", "failure")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.errors",
                instrument: "errors",
                unit: "{error}",
                attr: Some(("error.type", "scraper_failure")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.errors",
                instrument: "errors",
                unit: "{error}",
                attr: Some(("error.type", "partial_error")),
                value: 3,
            },
            Expected {
                scope: "receiver.host_metrics.errors",
                instrument: "errors",
                unit: "{error}",
                attr: Some(("error.type", "source_read_error")),
                value: 2,
            },
            Expected {
                scope: "receiver.host_metrics.errors",
                instrument: "errors",
                unit: "{error}",
                attr: Some(("error.type", "send_failure")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "scrapes.started",
                unit: "{scrape}",
                attr: None,
                value: 5,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "scrapes.skipped",
                unit: "{scrape}",
                attr: None,
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "families.scraped",
                unit: "{family}",
                attr: None,
                value: 9,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "batches.sent",
                unit: "{batch}",
                attr: None,
                value: 4,
            },
        ];

        assert_snapshot_cases(&snapshots, &cases);

        // Subsequent snapshot should be empty since hot values were flushed
        let empty_snapshots = metrics.terminal_snapshots();
        assert!(
            empty_snapshots.is_empty(),
            "subsequent terminal snapshots should be empty"
        );
    }

    /// Scenario: Host metrics receiver metrics are reported through a MetricsReporter.
    /// Guarantees: Every touched metric bucket is exported exactly once and hot
    /// values are cleared afterward.
    #[test]
    fn test_host_metrics_receiver_metrics_reporting() {
        let (pipeline_ctx, _registry) =
            test_pipeline_ctx_with_interests(Interests::NODE_OUTPUT_METRICS);

        let mut metrics = HostMetricsReceiverMetrics::register(&pipeline_ctx);
        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        let success = metrics.boundary.processing();
        success.success();

        let failure = metrics.boundary.processing();
        drop(failure);

        metrics.record_error(HostMetricsErrorType::SourceReadError);
        metrics.general.scrapes_started.add(1);
        metrics.general.scrapes_skipped.add(1);

        assert!(metrics.report(&mut reporter).is_ok());

        let snapshots: Vec<_> = rx.try_iter().collect();
        // Two scrape attempt outcome buckets, one error bucket, and one general bucket.
        assert_eq!(
            snapshots.len(),
            4,
            "all touched metric buckets should be exported exactly once"
        );

        let cases = [
            Expected {
                scope: "receiver.pulled",
                instrument: "attempts",
                unit: "{attempt}",
                attr: Some(("outcome", "success")),
                value: 1,
            },
            Expected {
                scope: "receiver.pulled",
                instrument: "attempts",
                unit: "{attempt}",
                attr: Some(("outcome", "failure")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.errors",
                instrument: "errors",
                unit: "{error}",
                attr: Some(("error.type", "source_read_error")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "scrapes.started",
                unit: "{scrape}",
                attr: None,
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics",
                instrument: "scrapes.skipped",
                unit: "{scrape}",
                attr: None,
                value: 1,
            },
        ];

        assert_snapshot_cases(&snapshots, &cases);

        assert!(
            metrics.terminal_snapshots().is_empty(),
            "reported metric sets should not retain hot values"
        );
    }
}
