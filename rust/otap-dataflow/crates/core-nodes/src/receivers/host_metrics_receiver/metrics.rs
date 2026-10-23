// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the host metrics receiver node.
//!
//! ### Counter Algebra and Operational Semantics
//!
//! The outcome population on `receiver.host_metrics.scrapes` (`scrapes`) applies strictly
//! to **accepted** scrapes (each accepted scrape starts with `receiver.host_metrics.scrapes.started`):
//!
//! ```text
//! receiver.host_metrics.scrapes.started == sum(
//!     receiver.host_metrics.scrapes{outcome="completed"},
//!     receiver.host_metrics.scrapes{outcome="failed"},
//!     receiver.host_metrics.scrapes{outcome="timeout"},
//!     receiver.host_metrics.scrapes{outcome="dropped"},
//!     receiver.host_metrics.scrapes{outcome="cancelled"}
//! )
//! ```
//!
//! Ticks that could not be accepted because the worker was already busy processing a prior
//! scrape are counted separately in `receiver.host_metrics.scrapes.skipped` and do not enter
//! the accepted outcome population. Worker-start failures are recorded under
//! `receiver.host_metrics.errors{error.type="scraper_failure"}`.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Outcome of an accepted host metrics scrape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ScrapeOutcome {
    /// Scrape tick built and sent a metrics batch downstream.
    Completed,
    /// Fatal scrape failure occurred during read, encode, or delivery.
    Failed,
    /// Accepted scrape timed out before completion.
    Timeout,
    /// Scrape batch was dropped before delivery due to downstream channel backpressure.
    Dropped,
    /// In-flight scrape was cancelled during receiver shutdown or drain.
    Cancelled,
}

/// Attributes for scrape outcome metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ScrapeOutcomeAttributes {
    /// Terminal outcome of the scrape tick.
    pub outcome: ScrapeOutcome,
}

/// Scrape outcome counters for the host metrics receiver.
#[metric_set(
    name = "receiver.host_metrics.scrapes",
    measurement_attributes = ScrapeOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct HostMetricsScrapeMetrics {
    /// Number of scrape ticks by outcome.
    #[metric(unit = "{scrape}")]
    pub scrapes: Counter<u64>,
}

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
    /// Wall-clock scrape duration.
    #[metric(unit = "ns")]
    pub scrape_duration_ns: Mmsc,
    /// Delay between scheduled and actual scrape start.
    #[metric(unit = "ns")]
    pub scrape_lag_ns: Mmsc,
}

/// Shared bounded-cardinality host metrics receiver metrics tracker.
pub struct HostMetricsReceiverMetrics {
    /// Scrape outcome metrics.
    pub scrapes: MeasurementMetricSet<HostMetricsScrapeMetrics>,
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
            scrapes: HostMetricsScrapeMetrics::register(pipeline_ctx),
            errors: HostMetricsErrorMetrics::register(pipeline_ctx),
            general: HostMetricsGeneralMetrics::register(pipeline_ctx),
        }
    }

    /// Records one scrape tick with the given outcome.
    pub fn record_scrape_outcome(&mut self, outcome: ScrapeOutcome) {
        self.scrapes
            .with(ScrapeOutcomeAttributes { outcome })
            .scrapes
            .inc();
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
        reporter.report_measurement(&mut self.scrapes)?;
        reporter.report_measurement(&mut self.errors)?;
        reporter.report(&mut self.general)?;
        Ok(())
    }

    /// Collects terminal snapshots for all metric sets.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.scrapes.terminal_snapshots();
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
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    /// Scenario: Host metrics receiver metric sets are recorded and drained into terminal snapshots.
    /// Guarantees: Scrape outcomes, error categories, and general counters emit valid snapshots and clear hot values.
    #[test]
    fn test_host_metrics_receiver_metrics_lifecycle() {
        let registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut metrics = HostMetricsReceiverMetrics::register(&pipeline_ctx);

        // Record accepted scrape outcomes
        metrics.record_scrape_outcome(ScrapeOutcome::Completed);
        metrics.record_scrape_outcome(ScrapeOutcome::Failed);
        metrics.record_scrape_outcome(ScrapeOutcome::Timeout);
        metrics.record_scrape_outcome(ScrapeOutcome::Dropped);
        metrics.record_scrape_outcome(ScrapeOutcome::Cancelled);

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
        metrics.general.scrape_duration_ns.record(1_500_000.0);
        metrics.general.scrape_lag_ns.record(250_000.0);

        let snapshots = metrics.terminal_snapshots();
        assert!(
            !snapshots.is_empty(),
            "terminal snapshots should contain recorded metric sets"
        );

        struct Expected<'a> {
            scope: &'a str,
            instrument: &'a str,
            unit: &'a str,
            attr: Option<(&'a str, &'a str)>,
            value: u64,
        }

        let cases = [
            Expected {
                scope: "receiver.host_metrics.scrapes",
                instrument: "scrapes",
                unit: "{scrape}",
                attr: Some(("outcome", "completed")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.scrapes",
                instrument: "scrapes",
                unit: "{scrape}",
                attr: Some(("outcome", "failed")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.scrapes",
                instrument: "scrapes",
                unit: "{scrape}",
                attr: Some(("outcome", "timeout")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.scrapes",
                instrument: "scrapes",
                unit: "{scrape}",
                attr: Some(("outcome", "dropped")),
                value: 1,
            },
            Expected {
                scope: "receiver.host_metrics.scrapes",
                instrument: "scrapes",
                unit: "{scrape}",
                attr: Some(("outcome", "cancelled")),
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

        // Subsequent snapshot should be empty since hot values were flushed
        let empty_snapshots = metrics.terminal_snapshots();
        assert!(
            empty_snapshots.is_empty(),
            "subsequent terminal snapshots should be empty"
        );
    }

    /// Scenario: Host metrics receiver metrics are reported through a MetricsReporter.
    /// Guarantees: All metric sets report without error and clear dirty tracking.
    #[test]
    fn test_host_metrics_receiver_metrics_reporting() {
        let registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut metrics = HostMetricsReceiverMetrics::register(&pipeline_ctx);
        let (_rx, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        metrics.record_scrape_outcome(ScrapeOutcome::Completed);
        metrics.record_scrape_outcome(ScrapeOutcome::Dropped);
        metrics.record_scrape_outcome(ScrapeOutcome::Timeout);
        metrics.record_error(HostMetricsErrorType::SourceReadError);
        metrics.general.scrapes_started.add(1);
        metrics.general.scrapes_skipped.add(1);

        assert!(metrics.report(&mut reporter).is_ok());
    }
}
