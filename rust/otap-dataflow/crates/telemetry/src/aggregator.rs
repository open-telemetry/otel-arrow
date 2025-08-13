// SPDX-License-Identifier: Apache-2.0

//! Aggregator for telemetry metrics.

use crate::metrics::MultivariateMetrics;
use crate::registry2::MetricsRegistry;
use tokio::time::{interval, Duration};

/// Multivariate metrics aggregator.
pub struct MetricsAggregator {
    /// The metrics registry that aggregates all multivariate metrics.
    registry: MetricsRegistry,

    metrics_receiver: flume::Receiver<Box<dyn MultivariateMetrics + Send + Sync>>,
}

/// A reporter for metrics
#[derive(Clone,Debug)]
pub struct MetricsReporter {
    metrics_sender: flume::Sender<Box<dyn MultivariateMetrics + Send + Sync>>,
}

impl MetricsAggregator {
    /// Creates a new `MetricsAggregator`.
    pub fn new() -> (
        Self,
        MetricsReporter,
    ) {
        let (metrics_sender, metrics_receiver) =
            flume::bounded::<Box<dyn MultivariateMetrics + Send + Sync>>(100);

        (
            Self {
                registry: MetricsRegistry::new(),
                metrics_receiver: metrics_receiver,
            },
            MetricsReporter::new(metrics_sender),
        )
    }

    /// Runs the aggregator, processing incoming metrics asynchronously.
    pub async fn run_forever(mut self) {
        let mut timer = interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    println!("{:#?}", self.registry);
                }
                result = self.metrics_receiver.recv_async() => {
                    match result {
                        Ok(metrics) => {
                            metrics.aggregate_into(&mut self.registry);
                        }
                        Err(_) => {
                            // Channel closed, exit the loop
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl MetricsReporter {
    fn new(metrics_sender: flume::Sender<Box<dyn MultivariateMetrics + Send + Sync>>) -> Self {
        Self { metrics_sender }
    }

    /// Report multivariate metrics and reset the metrics if successful.
    pub async fn report<M: MultivariateMetrics + Send + Sync + Clone + 'static>(&mut self, metrics: &mut M) {
        let snapshot = metrics.clone();
        if let Err(e) = self.metrics_sender.send_async(Box::new(snapshot)).await {
            eprintln!("Failed to send metrics: {:?}", e);
        } else {
            metrics.zero();
        }
    }
}