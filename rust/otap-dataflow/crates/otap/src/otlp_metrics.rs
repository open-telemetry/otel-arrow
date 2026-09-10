// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTLP receiver metric definitions.

use crate::metrics::ReceiverMetrics;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::{
    Outcome, ReceiverRejectionErrorType, SignalOutcomeAttributes,
};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Transport protocol used to receive an OTLP request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum OtlpProtocol {
    /// OTLP over gRPC.
    Grpc,
    /// OTLP over HTTP.
    Http,
}

/// Protocol and bounded error type dimensions for a rejected request.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpRejectionAttributes {
    /// OTLP transport on which the request was rejected.
    pub protocol: OtlpProtocol,
    /// Reason the request was rejected.
    #[attribute_key = "error.type"]
    pub error_type: ReceiverRejectionErrorType,
}

/// Signal and protocol dimensions for an accepted OTLP transport request.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpRequestAttributes {
    /// Signal carried by the request.
    pub signal: SignalType,
    /// OTLP transport used by the request.
    pub protocol: OtlpProtocol,
}

/// Protocol dimension for a transport-level receiver error.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OtlpTransportErrorAttributes {
    /// OTLP transport that surfaced the server error.
    pub protocol: OtlpProtocol,
}

/// Admitted OTLP requests.
#[metric_set(
    name = "receiver.otlp.requests",
    measurement_attributes = OtlpRequestAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpRequestMetrics {
    /// Number of OTLP requests admitted to the pipeline send path.
    #[metric(unit = "{request}")]
    pub accepted: Counter<u64>,
}

/// Requests rejected before pipeline admission.
#[metric_set(
    name = "receiver.otlp.requests",
    measurement_attributes = OtlpRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpRejectionMetrics {
    /// Number of rejected requests.
    #[metric(name = "rejected", unit = "{request}")]
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
    measurement_attributes = OtlpTransportErrorAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtlpTransportErrorMetrics {
    /// Number of transport-level server errors.
    #[metric(unit = "{error}")]
    pub errors: Counter<u64>,
}

/// Shared bounded-cardinality OTLP receiver metrics tracker.
#[derive(Debug)]
pub struct OtlpReceiverMetrics {
    /// Shared receiver boundary metrics.
    pub boundary: ReceiverMetrics,
    requests: MeasurementMetricSet<OtlpRequestMetrics>,
    rejections: MeasurementMetricSet<OtlpRejectionMetrics>,
    acknowledgements: MeasurementMetricSet<OtlpAcknowledgementMetrics>,
    transport_errors: MeasurementMetricSet<OtlpTransportErrorMetrics>,
}

impl OtlpReceiverMetrics {
    /// Registers all OTLP receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ReceiverMetrics::register(pipeline_ctx),
            requests: OtlpRequestMetrics::register(pipeline_ctx),
            rejections: OtlpRejectionMetrics::register(pipeline_ctx),
            acknowledgements: OtlpAcknowledgementMetrics::register(pipeline_ctx),
            transport_errors: OtlpTransportErrorMetrics::register(pipeline_ctx),
        }
    }

    /// Records an admitted OTLP request.
    pub fn record_request_admitted(&mut self, signal: SignalType, protocol: OtlpProtocol) {
        self.requests
            .with(OtlpRequestAttributes { signal, protocol })
            .accepted
            .inc();
    }

    /// Records a request rejected before pipeline admission.
    pub fn record_rejection(
        &mut self,
        protocol: OtlpProtocol,
        error_type: ReceiverRejectionErrorType,
    ) {
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
        self.transport_errors
            .with(OtlpTransportErrorAttributes { protocol })
            .errors
            .inc();
    }

    /// Returns a transport request bucket without marking it for export.
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
        error_type: ReceiverRejectionErrorType,
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
    pub fn transport_errors_for(&self, protocol: OtlpProtocol) -> &OtlpTransportErrorMetrics {
        self.transport_errors
            .get(OtlpTransportErrorAttributes { protocol })
    }

    /// Reports every touched OTLP receiver metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.requests)?;
        reporter.report_measurement(&mut self.rejections)?;
        reporter.report_measurement(&mut self.acknowledgements)?;
        reporter.report_measurement(&mut self.transport_errors)
    }

    /// Takes every touched OTLP receiver metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.requests.terminal_snapshots());
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots.extend(self.acknowledgements.terminal_snapshots());
        snapshots.extend(self.transport_errors.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

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
        let completed = metrics.boundary.processing().run(|processing| {
            processing.set_payload_size_with(|| 42);
            Ok::<_, crate::metrics::ErrorWithOutcome<()>>((SignalType::Logs, ()))
        });
        metrics.boundary.record(completed).unwrap();
        metrics.record_request_admitted(SignalType::Logs, OtlpProtocol::Grpc);
        metrics.record_rejection(
            OtlpProtocol::Http,
            ReceiverRejectionErrorType::InvalidRequest,
        );
        metrics.record_acknowledgement(SignalType::Logs, Outcome::Refused);
        metrics.record_transport_error(OtlpProtocol::Grpc);

        let snapshots = metrics.boundary.terminal_snapshots();
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|metric| metric.name == "messages")
                    .is_some_and(|index| snapshot.get_metrics()[index].to_u64_lossy() == 1)
        }));
        assert_eq!(
            metrics
                .rejections_for(
                    OtlpProtocol::Http,
                    ReceiverRejectionErrorType::InvalidRequest,
                )
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
        let requests = metrics.requests_for(SignalType::Logs, OtlpProtocol::Grpc);
        assert_eq!(requests.accepted.get(), 1);
        assert_eq!(
            metrics
                .transport_errors_for(OtlpProtocol::Grpc)
                .errors
                .get(),
            1
        );
    }

    /// Scenario: An admitted OTLP request has an empty decompressed payload.
    /// Guarantees: The shared receiver boundary still records the successful message.
    #[test]
    fn admitted_empty_request_still_records_received() {
        let mut metrics = new_test_metrics();
        let completed = metrics.boundary.processing().run(|processing| {
            processing.set_payload_size_with(|| 0);
            Ok::<_, crate::metrics::ErrorWithOutcome<()>>((SignalType::Logs, ()))
        });
        metrics.boundary.record(completed).unwrap();
        metrics.record_request_admitted(SignalType::Logs, OtlpProtocol::Http);

        let snapshots = metrics.boundary.terminal_snapshots();
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|metric| metric.name == "messages")
                    .is_some_and(|index| snapshot.get_metrics()[index].to_u64_lossy() == 1)
        }));
        assert_eq!(
            metrics
                .requests_for(SignalType::Logs, OtlpProtocol::Http)
                .accepted
                .get(),
            1
        );
    }

    /// Scenario: OTLP receiver metrics are transferred into terminal snapshots twice.
    /// Guarantees: Touched buckets carry enum wire values once and are then cleared.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_test_metrics();
        let completed = metrics
            .boundary
            .processing()
            .run(|_| Ok::<_, crate::metrics::ErrorWithOutcome<()>>((SignalType::Metrics, ())));
        metrics.boundary.record(completed).unwrap();
        metrics.record_request_admitted(SignalType::Metrics, OtlpProtocol::Http);
        metrics.record_rejection(
            OtlpProtocol::Grpc,
            ReceiverRejectionErrorType::MemoryPressure,
        );

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otlp.requests"
                && snapshot.measurement_attribute_value("signal") == Some("metrics")
                && snapshot.measurement_attribute_value("protocol") == Some("http")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otlp.requests"
                && snapshot.measurement_attribute_value("protocol") == Some("grpc")
                && snapshot.measurement_attribute_value("error.type") == Some("memory_pressure")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
