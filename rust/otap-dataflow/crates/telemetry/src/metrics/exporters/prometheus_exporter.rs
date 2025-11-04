// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry Prometheus Metrics exporter that serves metrics as Prometheus format.

use serde_json::Value;

use crate::metrics::dispatcher::{MetricsEntrySnapshot, MetricsExporter};
use crate::error::Error;

/// Prometheus metrics exporter.
pub struct PrometheusMetricsExporter {
    host: String,
    port: u16,
}

impl MetricsExporter for PrometheusMetricsExporter {
    fn name(&self) -> &str {
        "PrometheusMetricsExporter"
    }

    fn export_metrics(&self, _metric_sets: Vec<MetricsEntrySnapshot>) -> Result<(), Error> {
        // TODO: Implement Prometheus metrics exporting logic here.
        println!("Accumulating metrics for prometheus exposed at {}:{} (not implemented yet)", self.host, self.port);
        Ok(())
    }
}

impl PrometheusMetricsExporter {
    /// Creates a new instance of the PrometheusMetricsExporter.
    pub fn new(config: Value) -> Self {
        let host = config.get("host")
            .and_then(Value::as_str)
            .unwrap_or("localhost")
            .to_string();
            
        let port = config.get("port")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8888);
        println!("Prometheus service starting at {}:{}", host, port);

        PrometheusMetricsExporter { host, port }
    }
}