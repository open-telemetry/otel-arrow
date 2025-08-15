// SPDX-License-Identifier: Apache-2.0

//! Collector task for internal metrics.

use crate::metrics::MetricsSnapshot;
use crate::registry::MetricsRegistryHandle;
use tokio::time::{interval, Duration};
use crate::error::Error;
use crate::pipeline::{LineProtocolPipeline, MetricsPipeline};
use crate::reporter::MetricsReporter;

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
    metrics_receiver: flume::Receiver<MetricsSnapshot>,
}

impl MetricsCollector {
    /// Creates a new `MetricsAggregator`.
    pub(crate) fn new(registry: MetricsRegistryHandle) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) =
            flume::bounded::<MetricsSnapshot>(100);

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
    pub async fn run_collection_loop(mut self) -> Result<(), Error> {
        let mut timer = interval(Duration::from_secs(10));
        let reporter = LineProtocolPipeline;
        
        loop {
            tokio::select! {
                // ToDo need to be moved into a CollectorExporter
                _ = timer.tick() => {
                    println!("Collecting metrics... {}", self.registry.len());
                    self.registry.for_each_metrics(|m, attrs| {
                        // Ignore individual report errors for now; could log.
                        let _ = reporter.report(m, attrs.clone());
                    });
                }
                result = self.metrics_receiver.recv_async() => {
                    match result {
                        Ok(metrics) => {
                            metrics.aggregate_into(&mut self.registry)?;
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
