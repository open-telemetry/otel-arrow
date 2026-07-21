// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric declarations for the durable buffer processor.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::error::Error;
use otap_df_telemetry::instrument::{Counter, Gauge, ObserveCounter};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Metrics that do not have a bounded item dimension.
#[metric_set(name = "processor.durable_buffer")]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferOperationalMetrics {
    /// Number of read errors.
    #[metric(unit = "{error}")]
    pub(super) read_errors: Counter<u64>,
    /// Current bytes used by persistent storage.
    #[metric(unit = "By")]
    pub(super) storage_bytes_used: Gauge<u64>,
    /// Configured storage capacity cap.
    #[metric(unit = "By")]
    pub(super) storage_bytes_cap: Gauge<u64>,
    /// Number of retry attempts scheduled.
    #[metric(unit = "{retry}")]
    pub(super) retries_scheduled: Counter<u64>,
    /// Current number of bundles in flight to downstream.
    #[metric(unit = "{bundle}")]
    pub(super) in_flight: Gauge<u64>,
    /// Number of segment finalization failures.
    #[metric(unit = "{error}")]
    pub(super) flush_failures: Counter<u64>,
    /// Current storage utilization ratio.
    #[metric(unit = "{1}")]
    pub(super) storage_utilization: Gauge<f64>,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub(super) enum BundleOutcome {
    Acked,
    Deferred,
    PermanentlyRejected,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct BundleOutcomeAttributes {
    pub(super) outcome: BundleOutcome,
}

/// Bundle resolution metrics partitioned by downstream outcome.
#[metric_set(
    name = "processor.durable_buffer.bundles",
    measurement_attributes = BundleOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferBundleMetrics {
    /// Number of bundles resolved by downstream outcome.
    #[metric(unit = "{bundle}")]
    pub(super) resolved: Counter<u64>,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub(super) enum IngestFailure {
    Error,
    Backpressure,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct IngestFailureAttributes {
    pub(super) failure: IngestFailure,
}

/// Ingest failures partitioned by failure kind.
#[metric_set(
    name = "processor.durable_buffer.ingest",
    measurement_attributes = IngestFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferIngestMetrics {
    /// Number of failed ingest attempts.
    #[metric(unit = "{failure}")]
    pub(super) failures: Counter<u64>,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct SignalAttributes {
    pub(super) signal: SignalType,
}

/// Item operation metrics partitioned by OpenTelemetry signal.
#[metric_set(
    name = "processor.durable_buffer.items",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferItemMetrics {
    /// Number of items in permanently rejected bundles.
    #[metric(unit = "{item}")]
    pub(super) rejected: Counter<u64>,
    /// Number of items ingested to durable storage.
    #[metric(unit = "{item}")]
    pub(super) consumed: Counter<u64>,
    /// Number of items sent downstream.
    #[metric(unit = "{item}")]
    pub(super) produced: Counter<u64>,
    /// Number of items requeued for retry after a NACK.
    #[metric(unit = "{item}")]
    pub(super) requeued: Counter<u64>,
    /// Current number of items queued in durable storage.
    #[metric(unit = "{item}")]
    pub(super) queued: Gauge<u64>,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub(super) enum LossReason {
    DropOldest,
    Expired,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct LossAttributes {
    pub(super) reason: LossReason,
}

/// Aggregate storage loss metrics partitioned by retention reason.
#[metric_set(
    name = "processor.durable_buffer.loss",
    measurement_attributes = LossAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferLossMetrics {
    /// Number of segments lost.
    #[metric(unit = "{segment}")]
    pub(super) segments: ObserveCounter<u64>,
    /// Number of bundles lost.
    #[metric(unit = "{bundle}")]
    pub(super) bundles: ObserveCounter<u64>,
    /// Number of items lost.
    #[metric(unit = "{item}")]
    pub(super) items: ObserveCounter<u64>,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct SignalLossAttributes {
    pub(super) signal: SignalType,
    pub(super) reason: LossReason,
}

/// Item loss metrics partitioned by signal and retention reason.
#[metric_set(
    name = "processor.durable_buffer.item_loss",
    measurement_attributes = SignalLossAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct DurableBufferItemLossMetrics {
    /// Number of items lost.
    #[metric(unit = "{item}")]
    pub(super) items: Counter<u64>,
}

/// Metric sets emitted by a durable buffer processor.
pub(super) struct DurableBufferMetrics {
    pub(super) operational_metrics: MetricSet<DurableBufferOperationalMetrics>,
    pub(super) bundle_metrics: MeasurementMetricSet<DurableBufferBundleMetrics>,
    pub(super) ingest_metrics: MeasurementMetricSet<DurableBufferIngestMetrics>,
    pub(super) item_metrics: MeasurementMetricSet<DurableBufferItemMetrics>,
    pub(super) loss_metrics: MeasurementMetricSet<DurableBufferLossMetrics>,
    pub(super) item_loss_metrics: MeasurementMetricSet<DurableBufferItemLossMetrics>,
}

impl DurableBufferMetrics {
    pub(super) fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operational_metrics: DurableBufferOperationalMetrics::register(pipeline_ctx),
            bundle_metrics: DurableBufferBundleMetrics::register(pipeline_ctx),
            ingest_metrics: DurableBufferIngestMetrics::register(pipeline_ctx),
            item_metrics: DurableBufferItemMetrics::register(pipeline_ctx),
            loss_metrics: DurableBufferLossMetrics::register(pipeline_ctx),
            item_loss_metrics: DurableBufferItemLossMetrics::register(pipeline_ctx),
        }
    }

    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report(&mut self.operational_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.bundle_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.ingest_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.item_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.loss_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.item_loss_metrics))
    }

    pub(super) fn bundles_for(
        &mut self,
        outcome: BundleOutcome,
    ) -> &mut DurableBufferBundleMetrics {
        self.bundle_metrics
            .with(BundleOutcomeAttributes { outcome })
    }

    pub(super) fn ingest_for(&mut self, failure: IngestFailure) -> &mut DurableBufferIngestMetrics {
        self.ingest_metrics
            .with(IngestFailureAttributes { failure })
    }

    pub(super) fn items_for_signal(&mut self, signal: SignalType) -> &mut DurableBufferItemMetrics {
        self.item_metrics.with(SignalAttributes { signal })
    }

    pub(super) fn loss_for(&mut self, reason: LossReason) -> &mut DurableBufferLossMetrics {
        self.loss_metrics.with(LossAttributes { reason })
    }

    pub(super) fn item_loss_for(
        &mut self,
        signal: SignalType,
        reason: LossReason,
    ) -> &mut DurableBufferItemLossMetrics {
        self.item_loss_metrics
            .with(SignalLossAttributes { signal, reason })
    }
}
