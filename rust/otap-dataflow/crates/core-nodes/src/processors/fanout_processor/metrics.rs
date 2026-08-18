// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the fan-out processor.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{attribute_set, metric_set};

/// Fixed destination context for fan-out timeout metrics.
#[attribute_set(item, registration)]
#[derive(Debug, Clone)]
struct DestinationAttributes {
    destination: String,
}

#[metric_set(name = "processor.fanout")]
#[derive(Debug, Default, Clone)]
struct FanoutOperationalMetrics {
    /// Current number of active pipeline messages tracked by the processor.
    #[metric(unit = "{message}")]
    active: Gauge<u64>,
    /// Configured max_inflight value (0 means unlimited).
    #[metric(unit = "{message}")]
    max_inflight_config: Gauge<u64>,
    /// 1 when fanout is currently refusing new pdata via accept_pdata(), else 0.
    #[metric(unit = "1")]
    throttled: Gauge<u64>,
    /// Increments on transition from not-throttled to throttled.
    #[metric(unit = "{episode}")]
    throttle_episodes: Counter<u64>,
}

#[metric_set(
    name = "processor.fanout",
    registration_attributes = DestinationAttributes,
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
struct FanoutTimeoutMetrics {
    /// Destination message attempts that timed out.
    #[metric(unit = "{message}")]
    timed_out: Counter<u64>,
}

/// Metric sets emitted directly by a fan-out processor.
pub(super) struct FanoutMetrics {
    operational: MetricSet<FanoutOperationalMetrics>,
    timeouts: Vec<MeasurementMetricSet<FanoutTimeoutMetrics>>,
}

impl FanoutMetrics {
    /// Registers fan-out metrics.
    pub(super) fn register(
        pipeline_ctx: &PipelineContext,
        max_inflight: usize,
        destinations: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut operational = pipeline_ctx.register_metrics::<FanoutOperationalMetrics>();
        operational.max_inflight_config.set(max_inflight as u64);
        let timeouts = destinations
            .into_iter()
            .map(|destination| {
                FanoutTimeoutMetrics::register(pipeline_ctx, &DestinationAttributes { destination })
            })
            .collect();
        Self {
            operational,
            timeouts,
        }
    }

    /// Updates the current active pipeline message count.
    pub(super) fn set_active(&mut self, active: usize) {
        self.operational.active.set(active as u64);
    }

    /// Updates the current throttling state.
    pub(super) fn set_throttled(&mut self, throttled: bool) {
        self.operational.throttled.set(u64::from(throttled));
    }

    /// Records a transition into the throttled state.
    pub(super) fn record_throttle_episode(&mut self) {
        self.operational.throttle_episodes.inc();
    }

    /// Records a destination timeout for a signal.
    pub(super) fn record_timeout(&mut self, destination_index: usize, signal: SignalType) {
        self.timeouts[destination_index]
            .with(SignalAttributes { signal })
            .timed_out
            .inc();
    }

    /// Reports all fan-out metric sets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.operational)?;
        for timeouts in &mut self.timeouts {
            reporter.report_measurement(timeouts)?;
        }
        Ok(())
    }
}
