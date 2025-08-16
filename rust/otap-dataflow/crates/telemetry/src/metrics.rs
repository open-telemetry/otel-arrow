// SPDX-License-Identifier: Apache-2.0

//! Declaration of all multivariate metrics types used in this system.
//!
//! Note: This module will be entirely generated from a telemetry schema and Weaver in the future.

use crate::attributes::NodeStaticAttrs;
use crate::counter::Counter;
use crate::descriptor::{MetricsDescriptor, MetricsField, MetricsKind};
use crate::error::Error;
use crate::registry::{MetricsRegistry, MetricsKey, MetricsRegistryHandle};

/// Type representing a snapshot of multivariate metrics.
pub type MetricsSnapshot = Box<dyn MultivariateMetrics + Send + Sync>;

/// Trait for types that can aggregate their metrics into a `MetricsRegistry`.
pub trait MultivariateMetrics {
    /// Register the current multivariate metrics into the metrics registry.
    #[doc(hidden)]
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs);

    /// Returns the descriptor for this set of metrics.
    fn descriptor(&self) -> &'static MetricsDescriptor;

    /// Iterate over (descriptor_field, current_value) pairs in defined order.
    fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_>;

    /// Aggregates the metrics of this type to the provided registry.
    fn aggregate_into(&self, registry: &mut MetricsRegistryHandle) -> Result<(), Error>;

    /// Resets all metrics to zero / default.
    fn zero(&mut self);

    /// Returns true if at least one metric has a non-zero value.
    fn has_non_zero(&self) -> bool {
        self.field_values().any(|(_, v)| v != 0)
    }
}

/// OTLP receiver metrics.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone)]
pub struct OtlpReceiverMetrics {
    /// A unique key set at the registration of these metrics.
    key: Option<MetricsKey>,

    /// Total bytes received by the receiver.
    pub bytes_received: Counter<u64>,
    /// Total messages received by the receiver.
    pub messages_received: Counter<u64>,
}

const OTLP_RECEIVER_METRICS_DESC: MetricsDescriptor = MetricsDescriptor {
    name: "otlp.receiver.metrics",
    fields: &[
        MetricsField {
            name: "bytes.received",
            unit: "By",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "messages.received",
            unit: "{msg}",
            kind: MetricsKind::Counter,
        },
    ],
};

/// OTLP exporter metrics.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone)]
pub struct OtlpExporterMetrics {
    /// A unique key set at the registration of these metrics.
    key: Option<MetricsKey>,

    /// Number of export log requests received by the exporter.
    pub export_logs_request_received: Counter<u64>,
    /// Number of export log requests that succeeded.
    pub export_logs_request_success: Counter<u64>,
    /// Number of export log requests that failed.
    pub export_logs_request_failure: Counter<u64>,

    /// Number of export metrics requests received by the exporter.
    pub export_metrics_request_received: Counter<u64>,
    /// Number of export metrics requests that succeeded.
    pub export_metrics_request_success: Counter<u64>,
    /// Number of export metrics requests that failed.
    pub export_metrics_request_failure: Counter<u64>,

    /// Number of export traces requests received by the exporter.
    pub export_traces_request_received: Counter<u64>,
    /// Number of export traces requests that succeeded.
    pub export_traces_request_success: Counter<u64>,
    /// Number of export traces requests that failed.
    pub export_traces_request_failure: Counter<u64>,
}

const OTLP_EXPORTER_METRICS_DESC: MetricsDescriptor = MetricsDescriptor {
    name: "otlp.exporter.metrics",
    fields: &[
        MetricsField {
            name: "export.logs.request.received",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.logs.request.success",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.logs.request.failure",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.metrics.request.received",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.metrics.request.success",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.metrics.request.failure",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.traces.request.received",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.traces.request.success",
            unit: "{req}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "export.traces.request.failure",
            unit: "{req}",
            kind: MetricsKind::Counter,
        }
    ],
};

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterPdataMetrics {
    /// A unique key set at the registration of these metrics.
    key: Option<MetricsKey>,

    /// Number of pdata batches received.
    pub batches: Counter<u64>,
    /// Number of invalid pdata batches received.
    pub invalid_batches: Counter<u64>,
    /// Number of arrow records received.
    pub arrow_records: Counter<u64>,
    /// Number of logs received.
    pub logs: Counter<u64>,
    /// Number of spans received.
    pub spans: Counter<u64>,
    /// Number of metrics received.
    pub metrics: Counter<u64>,
}

const PERF_EXPORTER_METRICS_DESC: MetricsDescriptor = MetricsDescriptor {
    name: "perf.exporter.pdata.metrics",
    fields: &[
        MetricsField {
            name: "batches",
            unit: "{msg}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "invalid.batches",
            unit: "{msg}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "arrow.records",
            unit: "{record}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "logs",
            unit: "{log}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "spans",
            unit: "{span}",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "metrics",
            unit: "{metric}",
            kind: MetricsKind::Counter,
        },
    ],
};

impl MultivariateMetrics for OtlpReceiverMetrics {
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
        let key = registry.otlp_receiver_metrics.insert((OtlpReceiverMetrics::default(), attrs));
        self.key = Some(key);
    }

    fn descriptor(&self) -> &'static MetricsDescriptor {
        &OTLP_RECEIVER_METRICS_DESC
    }

    fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> {
        let desc = self.descriptor();
        let values = [self.bytes_received.get(), self.messages_received.get()];
        Box::new(desc.fields.iter().zip(values.into_iter()).map(|(f, v)| (f, v)))
    }

    fn aggregate_into(&self, aggregator: &mut MetricsRegistryHandle) -> Result<(), Error> {
        if let Some(key) = self.key {
            aggregator.add_otlp_receiver_metrics(key, self);
            Ok(())
        } else {
            Err(Error::MetricsNotRegistered {
                descriptor: self.descriptor()
            })
        }
    }

    fn zero(&mut self) {
        self.bytes_received.set(0);
        self.messages_received.set(0);
    }

    fn has_non_zero(&self) -> bool { // override for efficiency (no iterator boxing)
        self.bytes_received.get() != 0 || self.messages_received.get() != 0
    }
}

impl MultivariateMetrics for OtlpExporterMetrics {
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
        let key = registry.otlp_exporter_metrics.insert((OtlpExporterMetrics::default(), attrs));
        self.key = Some(key);
    }

    fn descriptor(&self) -> &'static MetricsDescriptor {
        &OTLP_EXPORTER_METRICS_DESC
    }

    fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> {
        let desc = self.descriptor();
        let values = [
            self.export_logs_request_received.get(),
            self.export_logs_request_success.get(),
            self.export_logs_request_failure.get(),
            self.export_metrics_request_received.get(),
            self.export_metrics_request_success.get(),
            self.export_metrics_request_failure.get(),
            self.export_traces_request_received.get(),
            self.export_traces_request_success.get(),
            self.export_traces_request_failure.get(),
        ];
        Box::new(desc.fields.iter().zip(values.into_iter()).map(|(f, v)| (f, v)))
    }

    fn aggregate_into(&self, aggregator: &mut MetricsRegistryHandle) -> Result<(), Error> {
        if let Some(key) = self.key {
            aggregator.add_otlp_exporter_metrics(key, self);
            Ok(())
        } else {
            Err(Error::MetricsNotRegistered {
                descriptor: self.descriptor()
            })
        }
    }

    fn zero(&mut self) {
        self.export_logs_request_received.set(0);
        self.export_logs_request_success.set(0);
        self.export_logs_request_failure.set(0);
        self.export_metrics_request_received.set(0);
        self.export_metrics_request_success.set(0);
        self.export_metrics_request_failure.set(0);
        self.export_traces_request_received.set(0);
        self.export_traces_request_success.set(0);
        self.export_traces_request_failure.set(0);
    }

    fn has_non_zero(&self) -> bool { // override for efficiency (no iterator boxing)
        self.export_logs_request_received.get() != 0
            || self.export_logs_request_success.get() != 0
            || self.export_logs_request_failure.get() != 0
            || self.export_metrics_request_received.get() != 0
            || self.export_metrics_request_success.get() != 0
            || self.export_metrics_request_failure.get() != 0
            || self.export_traces_request_received.get() != 0
            || self.export_traces_request_success.get() != 0
            || self.export_traces_request_failure.get() != 0
    }
}

impl MultivariateMetrics for PerfExporterPdataMetrics {
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
        let key = registry.perf_exporter_metrics.insert((PerfExporterPdataMetrics::default(), attrs));
        self.key = Some(key);
    }

    fn descriptor(&self) -> &'static MetricsDescriptor {
        &PERF_EXPORTER_METRICS_DESC
    }

    fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> {
        let desc = self.descriptor();
        let values = [
            self.batches.get(),
            self.invalid_batches.get(),
            self.arrow_records.get(),
            self.logs.get(),
            self.spans.get(),
            self.metrics.get(),
        ];
        Box::new(desc.fields.iter().zip(values.into_iter()).map(|(f, v)| (f, v)))
    }

    fn aggregate_into(&self, aggregator: &mut MetricsRegistryHandle) -> Result<(), Error> {
        if let Some(key) = self.key {
            aggregator.add_perf_exporter_metrics(key, self);
            Ok(())
        } else {
            Err(Error::MetricsNotRegistered {
                descriptor: self.descriptor()
            })
        }
    }

    fn zero(&mut self) {
        self.batches.set(0);
        self.invalid_batches.set(0);
        self.arrow_records.set(0);
        self.logs.set(0);
        self.spans.set(0);
        self.metrics.set(0);
    }

    fn has_non_zero(&self) -> bool { // override for efficiency (no iterator boxing)
        self.batches.get() != 0
            || self.invalid_batches.get() != 0
            || self.arrow_records.get() != 0
            || self.logs.get() != 0
            || self.spans.get() != 0
            || self.metrics.get() != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::{PerfExporterPdataMetrics, OtlpReceiverMetrics, MultivariateMetrics};

    #[test]
    fn test_perf_exporter_metrics() {
        let mut metrics = PerfExporterPdataMetrics::default();
        assert!(!metrics.has_non_zero());

        metrics.logs.add(10);
        metrics.metrics.inc();

        metrics.logs.inc();

        assert!(metrics.has_non_zero());
        assert_eq!(metrics.logs.get(), 11);
        assert_eq!(metrics.metrics.get(), 1);

        metrics.zero();
        assert!(!metrics.has_non_zero());
    }

    #[test]
    fn test_receiver_metrics_has_non_zero() {
        let mut metrics = OtlpReceiverMetrics::default();
        assert!(!metrics.has_non_zero());
        metrics.bytes_received.add(5);
        assert!(metrics.has_non_zero());
        metrics.zero();
        assert!(!metrics.has_non_zero());
    }
}