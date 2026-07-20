// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTLP receiver metric definitions.

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Transport protocol used to receive an OTLP request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum OtlpProtocol {
    /// OTLP over gRPC.
    Grpc,
    /// OTLP over HTTP.
    Http,
}

/// Bounded classification for requests rejected before pipeline admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum OtlpRejectionErrorType {
    /// The process is refusing ingress because memory pressure is active.
    MemoryPressure,
    /// No configured concurrency or acknowledgement slot is available.
    ConcurrencyLimit,
    /// The encoded or decoded payload exceeds the configured size limit.
    PayloadTooLarge,
    /// The request path, method, content type, encoding, or payload is invalid.
    InvalidRequest,
    /// Receiver state or an internal channel failed unexpectedly.
    Internal,
}

/// Signal and protocol dimensions for an accepted OTLP request.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpRequestAttributes {
    /// Signal carried by the request.
    pub signal: SignalType,
    /// OTLP transport used by the request.
    pub protocol: OtlpProtocol,
}

/// Protocol and bounded error type dimensions for a rejected request.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpRejectionAttributes {
    /// OTLP transport on which the request was rejected.
    pub protocol: OtlpProtocol,
    /// Reason the request was rejected.
    #[attribute_key = "error.type"]
    pub error_type: OtlpRejectionErrorType,
}

/// Protocol dimension for a transport-level receiver error.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpTransportAttributes {
    /// OTLP transport that surfaced the error.
    pub protocol: OtlpProtocol,
}

/// Lifecycle and payload metrics for accepted OTLP requests.
#[metric_set(
    name = "receiver.otlp.requests",
    measurement_attributes = OtlpRequestAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpRequestMetrics {
    /// Number of requests admitted to the pipeline send path.
    #[metric(unit = "{request}")]
    pub started: Counter<u64>,
    /// Number of admitted requests whose receiver work terminated.
    #[metric(unit = "{request}")]
    pub completed: Counter<u64>,
    /// Decompressed payload bytes for successfully decoded requests.
    #[metric(unit = "By")]
    pub payload_size: Counter<u64>,
}

/// Requests rejected before pipeline admission.
#[metric_set(
    name = "receiver.otlp.rejections",
    measurement_attributes = OtlpRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpRejectionMetrics {
    /// Number of rejected requests.
    #[metric(unit = "{request}")]
    pub requests: Counter<u64>,
}

/// Downstream acknowledgement routing results.
#[metric_set(
    name = "receiver.otlp.acknowledgements",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpAcknowledgementMetrics {
    /// Number of routed or invalid acknowledgement responses.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
}

/// Transport-level OTLP receiver errors.
#[metric_set(
    name = "receiver.otlp.transport",
    measurement_attributes = OtlpTransportAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpTransportMetrics {
    /// Number of transport-level server errors.
    #[metric(unit = "{error}")]
    pub errors: Counter<u64>,
}

/// Shared bounded-cardinality OTLP receiver metrics tracker.
#[derive(Debug)]
pub struct OtlpReceiverMetrics {
    requests: MeasurementMetricSet<OtlpRequestMetrics>,
    rejections: MeasurementMetricSet<OtlpRejectionMetrics>,
    acknowledgements: MeasurementMetricSet<OtlpAcknowledgementMetrics>,
    transport: MeasurementMetricSet<OtlpTransportMetrics>,
}

impl OtlpReceiverMetrics {
    /// Registers all OTLP receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            requests: OtlpRequestMetrics::register(pipeline_ctx),
            rejections: OtlpRejectionMetrics::register(pipeline_ctx),
            acknowledgements: OtlpAcknowledgementMetrics::register(pipeline_ctx),
            transport: OtlpTransportMetrics::register(pipeline_ctx),
        }
    }

    /// Records a request entering the pipeline send path.
    pub fn record_request_started(&mut self, signal: SignalType, protocol: OtlpProtocol) {
        self.requests
            .with(OtlpRequestAttributes { signal, protocol })
            .started
            .inc();
    }

    /// Records termination of receiver work for an admitted request.
    pub fn record_request_completed(&mut self, signal: SignalType, protocol: OtlpProtocol) {
        self.requests
            .with(OtlpRequestAttributes { signal, protocol })
            .completed
            .inc();
    }

    /// Records decompressed payload bytes for a successfully decoded request.
    pub fn record_request_payload_size(
        &mut self,
        signal: SignalType,
        protocol: OtlpProtocol,
        bytes: u64,
    ) {
        if bytes == 0 {
            return;
        }
        self.requests
            .with(OtlpRequestAttributes { signal, protocol })
            .payload_size
            .add(bytes);
    }

    /// Records a request rejected before pipeline admission.
    pub fn record_rejection(&mut self, protocol: OtlpProtocol, error_type: OtlpRejectionErrorType) {
        self.rejections
            .with(OtlpRejectionAttributes {
                protocol,
                error_type,
            })
            .requests
            .inc();
    }

    /// Records the outcome of routing an acknowledgement response.
    pub fn record_acknowledgement(&mut self, signal: SignalType, outcome: Outcome) {
        self.acknowledgements
            .with(SignalOutcomeAttributes { signal, outcome })
            .responses
            .inc();
    }

    /// Records a transport-level server error.
    pub fn record_transport_error(&mut self, protocol: OtlpProtocol) {
        self.transport
            .with(OtlpTransportAttributes { protocol })
            .errors
            .inc();
    }

    /// Returns a request bucket for inspection without marking it for export.
    #[must_use]
    pub fn requests_for(&self, signal: SignalType, protocol: OtlpProtocol) -> &OtlpRequestMetrics {
        self.requests
            .get(OtlpRequestAttributes { signal, protocol })
    }

    /// Returns a rejection bucket for inspection without marking it for export.
    #[must_use]
    pub fn rejections_for(
        &self,
        protocol: OtlpProtocol,
        error_type: OtlpRejectionErrorType,
    ) -> &OtlpRejectionMetrics {
        self.rejections.get(OtlpRejectionAttributes {
            protocol,
            error_type,
        })
    }

    /// Returns an acknowledgement bucket for inspection without marking it for export.
    #[must_use]
    pub fn acknowledgements_for(
        &self,
        signal: SignalType,
        outcome: Outcome,
    ) -> &OtlpAcknowledgementMetrics {
        self.acknowledgements
            .get(SignalOutcomeAttributes { signal, outcome })
    }

    /// Returns a transport bucket for inspection without marking it for export.
    #[must_use]
    pub fn transport_for(&self, protocol: OtlpProtocol) -> &OtlpTransportMetrics {
        self.transport.get(OtlpTransportAttributes { protocol })
    }

    /// Reports every touched OTLP receiver metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.requests)?;
        reporter.report_measurement(&mut self.rejections)?;
        reporter.report_measurement(&mut self.acknowledgements)?;
        reporter.report_measurement(&mut self.transport)
    }

    /// Takes every touched OTLP receiver metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.requests.terminal_snapshots();
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots.extend(self.acknowledgements.terminal_snapshots());
        snapshots.extend(self.transport.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> OtlpReceiverMetrics {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        OtlpReceiverMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Requests, rejections, acknowledgements, and transport errors span dimensions.
    /// Guarantees: Every counter is isolated by its bounded enum-based measurement attributes.
    #[test]
    fn receiver_metrics_are_partitioned_by_context() {
        let mut metrics = new_test_metrics();
        metrics.record_request_started(SignalType::Logs, OtlpProtocol::Grpc);
        metrics.record_request_payload_size(SignalType::Logs, OtlpProtocol::Grpc, 42);
        metrics.record_request_completed(SignalType::Logs, OtlpProtocol::Grpc);
        metrics.record_rejection(OtlpProtocol::Http, OtlpRejectionErrorType::InvalidRequest);
        metrics.record_acknowledgement(SignalType::Logs, Outcome::Refused);
        metrics.record_transport_error(OtlpProtocol::Grpc);

        let requests = metrics.requests_for(SignalType::Logs, OtlpProtocol::Grpc);
        assert_eq!(requests.started.get(), 1);
        assert_eq!(requests.completed.get(), 1);
        assert_eq!(requests.payload_size.get(), 42);
        assert_eq!(
            metrics
                .requests_for(SignalType::Logs, OtlpProtocol::Http)
                .started
                .get(),
            0
        );
        assert_eq!(
            metrics
                .rejections_for(OtlpProtocol::Http, OtlpRejectionErrorType::InvalidRequest)
                .requests
                .get(),
            1
        );
        assert_eq!(
            metrics
                .acknowledgements_for(SignalType::Logs, Outcome::Refused)
                .responses
                .get(),
            1
        );
        assert_eq!(metrics.transport_for(OtlpProtocol::Grpc).errors.get(), 1);
    }

    /// Scenario: OTLP receiver metrics are transferred into terminal snapshots twice.
    /// Guarantees: Touched buckets carry enum wire values once and are then cleared.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_test_metrics();
        metrics.record_request_started(SignalType::Metrics, OtlpProtocol::Http);
        metrics.record_rejection(OtlpProtocol::Grpc, OtlpRejectionErrorType::MemoryPressure);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otlp.requests"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("protocol") == Some("http")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otlp.rejections"
                && snapshot.measurement_attribute_value("protocol") == Some("grpc")
                && snapshot.measurement_attribute_value("error.type") == Some("memory_pressure")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
