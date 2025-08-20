// SPDX-License-Identifier: Apache-2.0

//! Collector task for internal metrics.

use crate::error::Error;
use crate::metrics::MetricSetSnapshot;
use crate::pipeline::{LineProtocolPipeline, MetricsPipeline};
use crate::registry::MetricsRegistryHandle;
use crate::reporter::MetricsReporter;
use tokio::time::{interval, Duration};

/// Metrics collector.
///
/// In this project, metrics are multivariate, meaning that multiple metrics are reported together
/// sharing the timestamp and the same set of attributes.
pub struct MetricsCollector {
    /// The metrics registry where metrics are declared and aggregated.
    registry: MetricsRegistryHandle,

    /// Receiver for incoming metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_receiver: flume::Receiver<MetricSetSnapshot>,
}

impl MetricsCollector {
    /// Creates a new `MetricsAggregator`.
    pub(crate) fn new(registry: MetricsRegistryHandle) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) = flume::bounded::<MetricSetSnapshot>(100);

        (
            Self {
                registry,
                metrics_receiver,
            },
            MetricsReporter::new(metrics_sender),
        )
    }

    /// Collects metrics from the reporting channel and aggregates them into the `registry`.
    /// The collection runs indefinitely until the metrics channel is closed.
    pub async fn run_collection_loop(self) -> Result<(), Error> {
        let mut timer = interval(Duration::from_secs(1));
        let reporter = LineProtocolPipeline;

        loop {
            tokio::select! {
                // ToDo need to be moved into a CollectorExporter
                _ = timer.tick() => {
                    self.registry.for_each_changed_field_iter_and_zero(|measurement, field_iter, attrs| {
                        // Ignore individual report errors for now; could log.
                        let _ = reporter.report_iter(measurement, field_iter, attrs);
                    });
                }
                result = self.metrics_receiver.recv_async() => {
                    match result {
                        Ok(metrics) => {
                            self.registry.add_metrics(metrics.key, &metrics.metrics);
                        }
                        Err(_) => {
                            // Channel closed, exit the loop
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
