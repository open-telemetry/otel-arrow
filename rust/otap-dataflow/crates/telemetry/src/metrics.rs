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
    /// Aggregates the metrics of this type to the provided registry.
    fn aggregate_into(&self, registry: &mut MetricsRegistryHandle) -> Result<(), Error>;

    /// Resets all metrics to zero / default.
    fn zero(&mut self);

    /// Returns the descriptor for this set of metrics.
    fn descriptor(&self) -> &'static MetricsDescriptor;

    /// Register the current multivariate metrics into the metrics registry.
    #[doc(hidden)]
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs);

    /// Iterate over (descriptor_field, current_value) pairs in defined order.
    fn field_values(&self) -> Box<dyn Iterator<Item = (&'static MetricsField, u64)> + '_>;
}

/// Multivariate metrics for receivers.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone)]
pub struct ReceiverMetrics {
    /// A unique key set at the registration of these metrics.
    key: Option<MetricsKey>,

    /// Total bytes received by the receiver.
    pub bytes_received: Counter<u64>,
    /// Total messages received by the receiver.
    pub messages_received: Counter<u64>,
}

const RECEIVER_METRICS_DESC: MetricsDescriptor = MetricsDescriptor {
    name: "receiver_metrics",
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

/// Multivariate metrics for the OTAP PerfExporter node.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterMetrics {
    /// A unique key set at the registration of these metrics.
    key: Option<MetricsKey>,

    /// Total bytes processed by the perf exporter.
    pub bytes_total: Counter<u64>,
    /// Number of pdata messages handled.
    pub pdata_msgs: Counter<u64>,
    /// Number of invalid pdata messages
    pub invalid_pdata_msgs: Counter<u64>,
    /// Number of logs processed.
    pub logs: Counter<u64>,
    /// Number of spans processed.
    pub spans: Counter<u64>,
    /// Number of metrics processed.
    pub metrics: Counter<u64>,
}

const PERF_EXPORTER_METRICS_DESC: MetricsDescriptor = MetricsDescriptor {
    name: "perf_exporter_metrics",
    fields: &[
        MetricsField {
            name: "bytes.total",
            unit: "By",
            kind: MetricsKind::Counter,
        },
        MetricsField {
            name: "pdata.messages",
            unit: "{msg}",
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

impl MultivariateMetrics for ReceiverMetrics {
    fn aggregate_into(&self, aggregator: &mut MetricsRegistryHandle) -> Result<(), Error> {
        if let Some(key) = self.key {
            aggregator.add_receiver_metrics(key, self);
            Ok(())
        } else {
            Err(Error::MetricsNotRegistered{
                descriptor: self.descriptor()
            })
        }
    }

    fn zero(&mut self) {
        self.bytes_received.set(0);
        self.messages_received.set(0);
    }

    fn descriptor(&self) -> &'static MetricsDescriptor {
        &RECEIVER_METRICS_DESC
    }

    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
        let key = registry.receiver_metrics.insert((ReceiverMetrics::default(), attrs));
        self.key = Some(key);
    }

    fn field_values(&self) -> Box<dyn Iterator<Item = (&'static MetricsField, u64)> + '_> {
        let desc = self.descriptor();
        let values = [self.bytes_received.get(), self.messages_received.get()];
        Box::new(desc.fields.iter().zip(values.into_iter()).map(|(f,v)| (f, v)))
    }
}

impl MultivariateMetrics for PerfExporterMetrics {
    fn aggregate_into(&self, aggregator: &mut MetricsRegistryHandle) -> Result<(), Error> {
        if let Some(key) = self.key {
            aggregator.add_perf_exporter_metrics(key, self);
            Ok(())
        } else {
            Err(Error::MetricsNotRegistered{
                descriptor: self.descriptor()
            })
        }
    }

    fn zero(&mut self) {
        self.bytes_total.set(0);
        self.pdata_msgs.set(0);
        self.invalid_pdata_msgs.set(0);
        self.logs.set(0);
        self.spans.set(0);
        self.metrics.set(0);
    }

    fn descriptor(&self) -> &'static MetricsDescriptor {
        &PERF_EXPORTER_METRICS_DESC
    }

    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
        let key = registry.perf_exporter_metrics.insert((PerfExporterMetrics::default(), attrs));
        self.key = Some(key);
    }

    fn field_values(&self) -> Box<dyn Iterator<Item = (&'static MetricsField, u64)> + '_> {
        let desc = self.descriptor();
        let values = [
            self.bytes_total.get(),
            self.pdata_msgs.get(),
            self.logs.get(),
            self.spans.get(),
            self.metrics.get(),
        ];
        Box::new(desc.fields.iter().zip(values.into_iter()).map(|(f,v)| (f, v)))
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::PerfExporterMetrics;

    #[test]
    fn test_perf_exporter_metrics() {
        let mut metrics = PerfExporterMetrics::default();

        metrics.bytes_total.inc();
        metrics.logs.add(10);
        metrics.metrics.inc();

        metrics.logs.inc();

        assert_eq!(metrics.bytes_total.get(), 1);
        assert_eq!(metrics.logs.get(), 11);
        assert_eq!(metrics.metrics.get(), 1);
    }
}