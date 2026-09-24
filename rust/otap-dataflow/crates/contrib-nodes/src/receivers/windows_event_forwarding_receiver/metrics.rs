//! WEF protocol diagnostics alongside the shared receiver boundary metrics.
//!
//! # Measurement boundaries
//!
//! [`WefMetrics`] combines shared receiver ingress/processing instrumentation with
//! protocol counters in `receiver.windows_event_forwarding`. Protocol requests
//! are counted once when their [`RequestObservation`] is dropped, including on
//! handler cancellation. Only requests that enter the authenticated HTTP handler
//! create observations: TLS failures and HTTP parser failures before dispatch
//! are not included.
//!
//! Request counts describe local handling, not event counts or successful receipt
//! of a response by Windows. A locally successful Ack can still be lost on the
//! connection. Event batches, heartbeats, enumeration, and termination requests
//! all contribute to the request counter; synthetic markers do not emit logs.
//!
//! # Outcome classification
//!
//! [`RequestObservation::complete`] classifies the handler's response as follows,
//! in priority order:
//!
//! | Condition | Outcome |
//! | --- | --- |
//! | Any 2xx status | Success |
//! | Explicit admission refusal or any 4xx status | Refused |
//! | Any other status | Failure |
//!
//! Thus an admission-related 503 is Refused, while a downstream Nack or unavailable
//! queue/feedback returning 503 is Failure. A feedback deadline returning 504 is
//! Failure and separately increments `feedback_timeouts`. Dropping an observation
//! before `complete` retains its default Failure outcome. In particular, an outer
//! request/connection timeout does not automatically count as a feedback timeout
//! or as the 408 response generated outside the cancelled handler.
//!
//! # Cardinality and reporting
//!
//! Requests have bounded `action` and `outcome` measurement attributes. Unknown
//! action URIs, or failures before action classification, use `Other`. Raw action
//! URIs, source identities, subscription IDs, and error text are not dimensions.
//! Diagnostic counters have no request measurement attributes and aggregate over
//! the receiver. Counters record deltas for reporting; terminal snapshots collect
//! pending measurements for the engine's final telemetry handoff.

use super::wsman::messages::Action;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ReceiverMetrics;
use otel_arrow_dfe_telemetry::{
    common_attributes::Outcome,
    error::Error,
    instrument::Counter,
    metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot},
    reporter::MetricsReporter,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::sync::{Arc, Mutex};

/// Fixed SOAP action labels, independent of untrusted request cardinality.
#[derive(Debug, Clone, Copy, AttributeEnum)]
pub(super) enum ProtocolAction {
    Enumerate,
    Heartbeat,
    Events,
    SubscriptionEnd,
    End,
    /// Unknown action, or no action classified before the request ended.
    Other,
}

impl From<Action> for ProtocolAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Enumerate => Self::Enumerate,
            Action::Heartbeat => Self::Heartbeat,
            Action::Events => Self::Events,
            Action::SubscriptionEnd => Self::SubscriptionEnd,
            Action::End => Self::End,
        }
    }
}

impl ProtocolAction {
    /// Match known action URIs exactly, collapsing all other values into `Other`.
    pub fn from_uri(uri: &str) -> Self {
        [
            Action::Enumerate,
            Action::Heartbeat,
            Action::Events,
            Action::SubscriptionEnd,
            Action::End,
        ]
        .into_iter()
        .find(|action| action.uri() == uri)
        .map(Self::from)
        .unwrap_or(Self::Other)
    }
}

/// Bounded dimensions attached only to protocol request measurements.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct RequestAttributes {
    action: ProtocolAction,
    outcome: Outcome,
}

/// Request counts partitioned by local action and outcome, not by event or source.
#[metric_set(name = "receiver.windows_event_forwarding", measurement_attributes = RequestAttributes)]
#[derive(Debug, Default, Clone)]
struct RequestMetrics {
    /// Authenticated HTTP requests completed locally or cancelled, by bounded SOAP action and outcome.
    #[metric(unit = "{request}")]
    requests: Counter<u64>,
}

/// Receiver-wide protocol milestones recorded from observation flags on drop.
#[metric_set(name = "receiver.windows_event_forwarding")]
#[derive(Debug, Default, Clone)]
struct DiagnosticMetrics {
    /// Subscription advertisements encoded in successful Enumerate responses, including repeated advertisements.
    /// Counts local response construction, not confirmed receipt or unique subscriptions.
    #[metric(unit = "{subscription}")]
    subscription_advertisements: Counter<u64>,
    /// Validated marker-only Events batches accepted without emitting application logs.
    /// Set only after local delivery/bookmark resolution succeeds, not merely after decoding.
    #[metric(unit = "{batch}")]
    marker_only_batches: Counter<u64>,
    /// Events requests for which the configured downstream feedback deadline expired.
    /// Excludes cancellation by the outer request/connection deadline or shutdown.
    #[metric(unit = "{request}")]
    feedback_timeouts: Counter<u64>,
}

/// Registered shared boundary metrics and WEF-specific protocol counters.
///
/// HTTP observations share this state through `Arc<Mutex<_>>`; the receiver task
/// reports it periodically and extracts terminal snapshots during shutdown.
pub(super) struct WefMetrics {
    /// Standard receiver metrics, recorded at ingress and processing boundaries.
    pub boundary: ReceiverMetrics,
    requests: MeasurementMetricSet<RequestMetrics>,
    diagnostics: MetricSet<DiagnosticMetrics>,
}

impl WefMetrics {
    /// Register all metric sets with the receiver's pipeline context.
    pub fn register(context: &PipelineContext) -> Self {
        Self {
            boundary: ReceiverMetrics::register(context),
            requests: RequestMetrics::register(context),
            diagnostics: DiagnosticMetrics::register(context),
        }
    }

    /// Report boundary metrics, request measurements, then aggregate diagnostics.
    ///
    /// Returns the first reporting error. Reporting is sequential, so earlier
    /// metric sets may already have been handed off when a later report fails.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.requests)?;
        reporter.report(&mut self.diagnostics)
    }

    /// Extract remaining measurements for terminal handoff, without a reporter.
    ///
    /// Consumes pending deltas and omits empty diagnostic snapshots. Repeated
    /// calls without new measurements do not recount earlier observations.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.requests.terminal_snapshots());
        if !self.diagnostics.is_empty() {
            snapshots.extend(self.diagnostics.terminal_snapshots());
        }
        snapshots
    }
}

/// Drop-recorded accounting for one authenticated HTTP handler invocation.
///
/// The handler updates classification and milestone flags as work proceeds, then
/// calls `complete` on its normal response path. Drop records one request and each
/// enabled diagnostic flag; flags are independent of the final outcome. An absent
/// metrics sink makes recording a no-op without changing handler control flow.
pub(super) struct RequestObservation {
    metrics: Option<Arc<Mutex<WefMetrics>>>,
    /// Recognized SOAP action; remains `Other` until the envelope is classified.
    pub action: ProtocolAction,
    /// Local response classification, defaulting to Failure for cancellation.
    pub outcome: Outcome,
    /// A nonempty subscription advertisement was successfully encoded.
    pub advertised: bool,
    /// A marker-only batch completed local delivery/bookmark resolution.
    pub marker_only: bool,
    /// The dedicated feedback timeout elapsed, rather than an outer deadline.
    pub feedback_timeout: bool,
    /// Admission explicitly refused work; distinguishes refusal from other 503s.
    pub refused: bool,
}

impl RequestObservation {
    /// Start an unclassified observation whose default outcome is cancellation-safe.
    pub fn new(metrics: Option<Arc<Mutex<WefMetrics>>>) -> Self {
        Self {
            metrics,
            action: ProtocolAction::Other,
            outcome: Outcome::Failure,
            advertised: false,
            marker_only: false,
            feedback_timeout: false,
            refused: false,
        }
    }

    /// Classify a locally produced response; recording still occurs only on drop.
    ///
    /// Success takes precedence over `refused`, followed by explicit refusal or
    /// client error, then Failure. This does not confirm network transmission and
    /// does not set or clear any diagnostic milestone flags.
    pub fn complete(&mut self, status: hyper::StatusCode) {
        self.outcome = if status.is_success() {
            Outcome::Success
        } else if self.refused || status.is_client_error() {
            Outcome::Refused
        } else {
            Outcome::Failure
        };
    }
}

impl Drop for RequestObservation {
    fn drop(&mut self) {
        if let Some(metrics) = &self.metrics {
            let mut metrics = metrics.lock().expect("WEF metrics lock poisoned");
            metrics
                .requests
                .with(RequestAttributes {
                    action: self.action,
                    outcome: self.outcome,
                })
                .requests
                .inc();
            if self.advertised {
                metrics.diagnostics.subscription_advertisements.inc();
            }
            if self.marker_only {
                metrics.diagnostics.marker_only_batches.inc();
            }
            if self.feedback_timeout {
                metrics.diagnostics.feedback_timeouts.inc();
            }
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub fn registered() -> Arc<Mutex<WefMetrics>> {
        let (context, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx();
        Arc::new(Mutex::new(WefMetrics::register(&context)))
    }

    pub fn assert_counts(
        metrics: &Arc<Mutex<WefMetrics>>,
        expected: &[(&str, &str, u64)],
        diagnostics: [u64; 3],
    ) {
        let mut metrics = metrics.lock().unwrap();
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(64);
        metrics.report(&mut reporter).unwrap();
        let snapshots: Vec<_> = receiver
            .try_iter()
            .filter(|snapshot| snapshot.descriptor().name == "receiver.windows_event_forwarding")
            .collect();
        let requests: Vec<_> = snapshots
            .iter()
            .filter(|snapshot| snapshot.measurement_attribute_value("action").is_some())
            .collect();
        assert_eq!(requests.len(), expected.len());
        for &(action, outcome, count) in expected {
            let snapshot = requests
                .iter()
                .find(|snapshot| {
                    snapshot.measurement_attribute_value("action") == Some(action)
                        && snapshot.measurement_attribute_value("outcome") == Some(outcome)
                })
                .unwrap_or_else(|| panic!("missing {action}/{outcome}"));
            assert_eq!(
                snapshot.get_metrics()[0].to_u64_lossy(),
                count,
                "{action}/{outcome}"
            );
        }
        let actual = snapshots
            .iter()
            .find(|snapshot| snapshot.measurement_attribute_value("action").is_none())
            .map(|snapshot| {
                snapshot
                    .get_metrics()
                    .iter()
                    .map(|metric| metric.to_u64_lossy())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![0; 3]);
        assert_eq!(actual, diagnostics);
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: authenticated requests are cancelled, refused, or fail after SOAP action classification.
    /// Guarantees: each observation records once with bounded labels, and cancellation never claims a feedback timeout.
    #[test]
    fn request_outcomes_and_cancellation() {
        let metrics = registered();
        drop(RequestObservation::new(Some(Arc::clone(&metrics))));
        for (action, status, refused) in [
            (Action::Events, hyper::StatusCode::SERVICE_UNAVAILABLE, true),
            (
                Action::Events,
                hyper::StatusCode::SERVICE_UNAVAILABLE,
                false,
            ),
            (
                Action::SubscriptionEnd,
                hyper::StatusCode::BAD_REQUEST,
                false,
            ),
            (Action::Heartbeat, hyper::StatusCode::OK, false),
        ] {
            let mut observation = RequestObservation::new(Some(Arc::clone(&metrics)));
            observation.action = ProtocolAction::from_uri(action.uri());
            observation.refused = refused;
            observation.complete(status);
        }
        assert!(matches!(
            ProtocolAction::from_uri("urn:arbitrary:action"),
            ProtocolAction::Other
        ));
        assert_counts(
            &metrics,
            &[
                ("other", "failure", 1),
                ("events", "refused", 1),
                ("events", "failure", 1),
                ("subscription_end", "refused", 1),
                ("heartbeat", "success", 1),
            ],
            [0, 0, 0],
        );
        let mut observation = RequestObservation::new(Some(Arc::clone(&metrics)));
        observation.action = ProtocolAction::End;
        observation.complete(hyper::StatusCode::NO_CONTENT);
        drop(observation);
        let mut metrics = metrics.lock().unwrap();
        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].get_metrics()[0].to_u64_lossy(), 1);
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
