// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry Metrics exporter that outputs metrics to the system standard output

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use chrono::{DateTime, Utc};

use crate::metrics::dispatcher::{MetricsEntrySnapshot, MetricsExporter};
use crate::error::Error;

/// Metrics exporter that outputs metrics to the system standard output.
pub struct LoggingMetricsExporter {
    invocation_count: AtomicUsize,
}

impl MetricsExporter for LoggingMetricsExporter {
    fn name(&self) -> &str {
        "LoggingMetricsExporter"
    }

    fn export_metrics(&self, metric_sets: Vec<MetricsEntrySnapshot>) -> Result<(), Error> {
        let prev_value = self.invocation_count.fetch_add(1, Ordering::SeqCst);
        for metric_set in metric_sets {
            let now: DateTime<Utc> = SystemTime::now().into();
            let formatted_now = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let as_json = serde_json::to_string(&metric_set)
                .map_err(|e| Error::SerializationError(format!("JSON serialization failed: {}", e)))?;
            println!("{} MetricsExporter({}): {}", formatted_now, prev_value, as_json);
        }
        Ok(())
    }
}

impl LoggingMetricsExporter {
    /// Creates a new instance of the LoggingMetricsExporter.
    pub fn new() -> Self {
        LoggingMetricsExporter {
            invocation_count: AtomicUsize::new(0),
        }
    }
}