//! Bounded handoff from HTTP event delivery to the receiver's pipeline task.
//!
//! [`DeliveryBridge::prepare`] checks framework admission before decoding, then
//! reserves source progress and, for event-bearing batches, a feedback slot.
//! [`PreparedDelivery::deliver`] queues the records and waits for whole-batch
//! downstream feedback before resolving the bookmark reservation. Queueing alone
//! never advances progress. Validated marker-only batches commit locally without
//! entering the pipeline, but still reserve progress to preserve source ordering.
//!
//! The HTTP caller owns delivery timeouts. Dropping a prepared delivery or its
//! running future releases its feedback slot and Nacks unresolved progress.
//! Generation-tagged feedback keys prevent late responses from resolving a reused
//! slot. Cancellation cannot retract records already handed to the pipeline, so
//! a source retry may produce duplicates. Committed bookmarks remain memory-only.

use super::{
    bookmark::{BatchOutcome, BookmarkStore, PendingBatch},
    config::Limits,
    identity::SourceIdentity,
    metrics::{RequestObservation, WefMetrics},
};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::control::CallData;
use otel_arrow_dfe_engine::{
    admission::{AdmissionContext, AdmissionDecision, SharedAdmissionGate},
    memory_limiter::SharedReceiverAdmissionState,
};
use otel_arrow_dfe_otap::accessory::slots::{Key, State};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

/// Shared admission, progress, and feedback state for concurrent HTTP requests.
///
/// Clones share the bounded queue and registries; they do not create independent
/// source ordering domains. Mutex guards are held only for synchronous operations.
#[derive(Clone)]
pub(super) struct DeliveryBridge {
    sender: mpsc::Sender<Delivery>,
    progress: Arc<Mutex<BookmarkStore>>,
    metrics: Option<Arc<Mutex<WefMetrics>>>,
    admission: SharedReceiverAdmissionState,
    rate_limiter: Option<SharedAdmissionGate>,
    /// Signals HTTP lifecycle cancellation; `prepare` and `deliver` do not poll it.
    pub force_stop: tokio_util::sync::CancellationToken,
    /// Routes engine Ack/Nack call data back to the waiting HTTP delivery.
    pub feedback: FeedbackRegistry,
}

/// Bounded one-shot completion slots indexed by generation-tagged keys.
#[derive(Clone)]
pub(super) struct FeedbackRegistry(Arc<Mutex<State<oneshot::Sender<BatchOutcome>>>>);

impl FeedbackRegistry {
    /// Consume a matching slot and notify its waiter of the downstream outcome.
    ///
    /// Malformed, stale, duplicate, or cancelled keys are ignored. Notification
    /// alone does not commit progress; the waiting delivery resolves its ticket.
    pub fn resolve(&self, calldata: CallData, outcome: BatchOutcome) {
        if let Ok(key) = Key::try_from(calldata)
            && let Some(sender) = self.0.lock().expect("WEF feedback lock poisoned").take(key)
        {
            let _ = sender.send(outcome);
        }
    }
}

/// Cloneable completion handle for one delivery, not an owner of its reservation.
///
/// All clones address the same slot. Only the first successful resolution can
/// notify the waiter; dropping a handle does not cancel the slot.
#[derive(Clone)]
pub(super) struct Feedback {
    registry: FeedbackRegistry,
    key: Key,
}

impl Feedback {
    /// Encode the slot and generation for the engine's Ack/Nack return route.
    pub fn calldata(&self) -> CallData {
        self.key.into()
    }

    /// Whether the slot is gone or its receiving delivery has been dropped.
    ///
    /// This is a snapshot used to skip cancelled queued work, not a guarantee
    /// that the delivery will remain active during a subsequent pipeline send.
    pub fn is_closed(&self) -> bool {
        self.registry
            .0
            .lock()
            .expect("WEF feedback lock poisoned")
            .get(self.key)
            .is_none_or(oneshot::Sender::is_closed)
    }

    /// Resolve directly, returning an error if the slot or waiter is unavailable.
    pub fn send(self, outcome: BatchOutcome) -> Result<(), ()> {
        self.registry
            .0
            .lock()
            .expect("WEF feedback lock poisoned")
            .take(self.key)
            .ok_or(())?
            .send(outcome)
            .map_err(|_| ())
    }
}

/// Owns the receiving half and cancels any unresolved slot when dropped.
struct FeedbackWaiter {
    feedback: Feedback,
    result: oneshot::Receiver<BatchOutcome>,
}

impl Drop for FeedbackWaiter {
    fn drop(&mut self) {
        self.feedback
            .registry
            .0
            .lock()
            .expect("WEF feedback lock poisoned")
            .cancel(self.feedback.key);
    }
}

/// Event records queued for the pipeline together with their completion route.
pub(super) struct Delivery {
    pub records: OtapArrowRecords,
    pub feedback: Feedback,
}

#[derive(Debug)]
/// Preparation failures kept distinct so HTTP handling can select a response.
pub(super) enum PreparationError {
    /// Decoding or validation failed before a progress reservation was acquired.
    Invalid(String),
    /// Source ordering, progress bounds, or feedback capacity refused the batch.
    Refused(String),
    /// Framework admission rejected work with a retry delay expressed in seconds.
    Throttled { retry_after_secs: u32 },
    /// The request exceeds the admission gate's payload budget.
    Oversized,
}

/// Decoded batch holding source progress and optional downstream feedback capacity.
///
/// Preparation does not reserve a channel permit. Delivery uses a nonblocking
/// enqueue and releases its reservations if the queue is full or closed.
pub(super) struct PreparedDelivery {
    reservation: Reservation,
    records: Option<OtapArrowRecords>,
    sender: mpsc::Sender<Delivery>,
    waiter: Option<FeedbackWaiter>,
}

/// Drop guard ensuring every pending bookmark ticket receives a terminal outcome.
struct Reservation {
    progress: Arc<Mutex<BookmarkStore>>,
    ticket: Option<PendingBatch>,
}

impl Reservation {
    /// Consume the ticket exactly once and apply its outcome to source progress.
    fn finish(&mut self, outcome: BatchOutcome) -> Result<(), String> {
        self.progress
            .lock()
            .expect("WEF progress lock poisoned")
            .finish(self.ticket.take().expect("unresolved WEF ticket"), outcome)
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if self.ticket.is_some() {
            let _ = self.finish(BatchOutcome::Nack);
        }
    }
}

impl PreparedDelivery {
    /// Whether decoding produced no records and no downstream feedback is needed.
    pub fn is_marker_only(&self) -> bool {
        self.records.is_none()
    }

    /// Commit a marker locally or enqueue records and await downstream acceptance.
    ///
    /// Success means the reservation was resolved with Ack, not merely queued.
    /// Nack and enqueue/feedback failures return errors without advancing progress.
    /// There is no internal deadline: callers bound this future and cancellation
    /// drops its reservation and waiter, releasing both capacities.
    pub async fn deliver(mut self) -> Result<(), String> {
        let Some(records) = self.records.take() else {
            return self.reservation.finish(BatchOutcome::Ack);
        };
        let mut waiter = self.waiter.take().expect("events reserve a feedback slot");
        self.sender
            .try_send(Delivery {
                records,
                feedback: waiter.feedback.clone(),
            })
            .map_err(|_| "WEF delivery queue unavailable")?;
        let outcome = (&mut waiter.result)
            .await
            .map_err(|_| "WEF downstream feedback unavailable")?;
        self.reservation.finish(outcome)?;
        match outcome {
            BatchOutcome::Ack => Ok(()),
            BatchOutcome::Nack => Err("WEF batch rejected downstream".into()),
        }
    }
}

impl DeliveryBridge {
    /// Create shared progress state, feedback slots, and the pipeline-facing queue.
    ///
    /// Queue and feedback capacities use `max_in_flight_batches`. Limits must
    /// already be validated, including a nonzero in-flight capacity.
    pub fn new(limits: &Limits) -> (Self, mpsc::Receiver<Delivery>) {
        let (sender, receiver) = mpsc::channel(limits.max_in_flight_batches);
        (
            Self {
                sender,
                progress: Arc::new(Mutex::new(BookmarkStore::new(limits))),
                metrics: None,
                admission: SharedReceiverAdmissionState::default(),
                rate_limiter: None,
                force_stop: tokio_util::sync::CancellationToken::new(),
                feedback: FeedbackRegistry(Arc::new(Mutex::new(State::new(
                    limits.max_in_flight_batches,
                )))),
            },
            receiver,
        )
    }

    /// Attach shared request and processing telemetry to this bridge.
    pub fn with_metrics(mut self, metrics: Arc<Mutex<WefMetrics>>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Start a request observation using the configured optional telemetry sink.
    pub fn observe_request(&self) -> RequestObservation {
        RequestObservation::new(self.metrics.clone())
    }

    /// Attach framework memory-pressure state and an optional byte-rate gate.
    pub fn with_admission(
        mut self,
        admission: SharedReceiverAdmissionState,
        rate_limiter: Option<SharedAdmissionGate>,
    ) -> Self {
        self.admission = admission;
        self.rate_limiter = rate_limiter;
        self
    }

    /// Return the retry delay when memory-pressure policy requires ingress shedding.
    pub fn pressure_retry_after(&self) -> Option<u32> {
        self.admission
            .should_shed_ingress()
            .then(|| self.admission.retry_after_secs())
    }

    /// Admit and decode a batch, then reserve progress and any feedback slot.
    ///
    /// `payload_size` is the request-byte cost supplied to rate admission and
    /// processing telemetry. Enforced admission refusals never invoke `decode`;
    /// observe-only rate decisions allow work to proceed. The closure returns the
    /// optional bookmark and records after validating the entire batch. `None`
    /// records means a marker-only delivery, not a downstream empty record batch.
    ///
    /// Decoding precedes bookmark and feedback reservations. Progress reservation
    /// enforces source ordering even for markers; if feedback allocation then
    /// fails, the reservation's drop guard releases progress with Nack.
    pub fn prepare(
        &self,
        source: SourceIdentity,
        payload_size: usize,
        decode: impl FnOnce() -> Result<(Option<String>, Option<OtapArrowRecords>), String>,
    ) -> Result<PreparedDelivery, PreparationError> {
        let work = || {
            if let Some(retry_after_secs) = self.pressure_retry_after() {
                return Err(PreparationError::Throttled { retry_after_secs });
            }
            if let Some(gate) = &self.rate_limiter {
                if gate.refuse_if_instance_saturated() {
                    return Err(PreparationError::Throttled {
                        retry_after_secs: 1,
                    });
                }
                match gate.admit(
                    payload_size as u64,
                    AdmissionContext::for_signal(SignalType::Logs),
                ) {
                    AdmissionDecision::Admit | AdmissionDecision::WouldThrottle => {}
                    AdmissionDecision::Throttle { retry_after_secs } => {
                        return Err(PreparationError::Throttled { retry_after_secs });
                    }
                    AdmissionDecision::Oversized => return Err(PreparationError::Oversized),
                }
            }
            let (bookmark, records) = decode().map_err(PreparationError::Invalid)?;
            let ticket = self
                .progress
                .lock()
                .expect("WEF progress lock poisoned")
                .begin(source, bookmark)
                .map_err(PreparationError::Refused)?;
            let reservation = Reservation {
                progress: Arc::clone(&self.progress),
                ticket: Some(ticket),
            };
            let waiter = if records.is_some() {
                let (key, result) = self
                    .feedback
                    .0
                    .lock()
                    .expect("WEF feedback lock poisoned")
                    .allocate(oneshot::channel)
                    .ok_or_else(|| {
                        PreparationError::Refused("WEF feedback capacity exceeded".into())
                    })?;
                Some(FeedbackWaiter {
                    feedback: Feedback {
                        registry: self.feedback.clone(),
                        key,
                    },
                    result,
                })
            } else {
                None
            };
            Ok(PreparedDelivery {
                reservation,
                records,
                sender: self.sender.clone(),
                waiter,
            })
        };
        let Some(metrics) = &self.metrics else {
            return work();
        };
        let processing = metrics
            .lock()
            .expect("WEF metrics lock poisoned")
            .boundary
            .processing();
        let completed = processing.run(|processing| {
            processing.set_payload_size_with(|| payload_size);
            work()
                .map(|value| (SignalType::Logs, value))
                .map_err(|error| processing.refused(SignalType::Logs, error))
        });
        metrics
            .lock()
            .expect("WEF metrics lock poisoned")
            .boundary
            .record(completed)
    }

    /// Copy the last committed bookmark, excluding any pending candidate.
    pub fn bookmark(&self, source: &SourceIdentity) -> Option<String> {
        self.progress
            .lock()
            .expect("WEF progress lock poisoned")
            .bookmark(source)
            .map(str::to_owned)
    }

    #[cfg(test)]
    /// Exercise preparation and delivery with already-decoded test data.
    pub async fn deliver(
        &self,
        source: SourceIdentity,
        bookmark: Option<String>,
        records: Option<OtapArrowRecords>,
    ) -> Result<(), String> {
        self.prepare(source, 0, || Ok((bookmark, records)))
            .map_err(|error| format!("{error:?}"))?
            .deliver()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::AuthConfig;
    use super::*;
    use otel_arrow_dfe_pdata::otap::Logs;
    use rcgen::{CertificateParams, KeyPair};

    fn source() -> SourceIdentity {
        let certificate = CertificateParams::new(vec!["host.example".to_owned()])
            .unwrap()
            .self_signed(&KeyPair::generate().unwrap())
            .unwrap();
        SourceIdentity::from_verified_certificate(
            certificate.der(),
            &AuthConfig {
                allowed_sources: vec!["host.example".into()],
            },
        )
        .unwrap()
    }

    fn records() -> Option<OtapArrowRecords> {
        Some(OtapArrowRecords::Logs(Logs::default()))
    }

    /// Scenario: memory pressure changes and byte-rate admission runs in enforce or observe-only mode.
    /// Guarantees: enforced refusals precede decoding/reservation, recovery resumes admission, and observe-only stays nonblocking.
    #[test]
    fn framework_admission_controls_processing() {
        use otel_arrow_dfe_config::policy::{
            MemoryLimiterMode, RateLimitAggregation, RateLimitEnforcement, RateLimitPressure,
            RateLimitUnit, RateLimiterPolicy, TokenBucketPolicy,
        };
        use otel_arrow_dfe_engine::{
            admission::{AdmissionBinder, AdmissionDimension},
            memory_limiter::{
                MemoryPressureBehaviorConfig, MemoryPressureChanged, MemoryPressureLevel,
                MemoryPressureState,
            },
        };
        let source = source();
        for mode in [MemoryLimiterMode::Enforce, MemoryLimiterMode::ObserveOnly] {
            let process = MemoryPressureState::default();
            process.configure(MemoryPressureBehaviorConfig {
                mode,
                retry_after_secs: 7,
                fail_readiness_on_hard: true,
            });
            let admission = SharedReceiverAdmissionState::from_process_state(&process);
            admission.apply(MemoryPressureChanged {
                generation: 1,
                level: MemoryPressureLevel::Hard,
                retry_after_secs: 7,
                usage_bytes: 100,
            });
            let (bridge, _) = DeliveryBridge::new(&Limits::default());
            let bridge = bridge.with_admission(admission.clone(), None);
            if mode == MemoryLimiterMode::Enforce {
                assert_eq!(bridge.pressure_retry_after(), Some(7));
                assert!(matches!(
                    bridge.prepare(source.clone(), 8, || panic!(
                        "must not decode under hard pressure"
                    )),
                    Err(PreparationError::Throttled {
                        retry_after_secs: 7
                    })
                ));
            } else {
                assert!(
                    bridge
                        .prepare(source.clone(), 8, || Ok((None, None)))
                        .is_ok()
                );
            }
            admission.apply(MemoryPressureChanged {
                generation: 2,
                level: MemoryPressureLevel::Normal,
                retry_after_secs: 7,
                usage_bytes: 0,
            });
            assert_eq!(bridge.pressure_retry_after(), None);
            assert!(
                bridge
                    .prepare(source.clone(), 8, || Ok((None, None)))
                    .is_ok()
            );
        }
        for enforcement in [
            RateLimitEnforcement::Enforce,
            RateLimitEnforcement::ObserveOnly,
        ] {
            let admission = SharedReceiverAdmissionState::default();
            admission.apply(MemoryPressureChanged {
                generation: 1,
                level: MemoryPressureLevel::Soft,
                retry_after_secs: 1,
                usage_bytes: 100,
            });
            let gate = AdmissionBinder::configured(
                "wef-test",
                RateLimiterPolicy {
                    enforcement,
                    aggregation: RateLimitAggregation::ReceiverInstance,
                    unit: RateLimitUnit::RequestBytes,
                    pressure: RateLimitPressure::Soft,
                    token_bucket: TokenBucketPolicy {
                        allow: 1,
                        interval: std::time::Duration::from_secs(3600),
                        burst: Some(8),
                    },
                },
            )
            .bind_shared(AdmissionDimension::Bytes, admission.clone())
            .unwrap();
            let (bridge, _) = DeliveryBridge::new(&Limits::default());
            let bridge = bridge.with_admission(admission, gate);
            if enforcement == RateLimitEnforcement::Enforce {
                assert!(matches!(
                    bridge.prepare(source.clone(), 9, || panic!("oversized must not decode")),
                    Err(PreparationError::Oversized)
                ));
                drop(
                    bridge
                        .prepare(source.clone(), 8, || Ok((None, None)))
                        .unwrap(),
                );
                assert!(matches!(
                    bridge.prepare(source.clone(), 8, || panic!("throttled must not decode")),
                    Err(PreparationError::Throttled { .. })
                ));
            } else {
                assert!(
                    bridge
                        .prepare(source.clone(), 9, || Ok((None, None)))
                        .is_ok()
                );
                assert!(
                    bridge
                        .prepare(source.clone(), 9, || Ok((None, None)))
                        .is_ok()
                );
            }
        }
    }

    /// Scenario: a cancelled waiter's slot is reused and late or malformed feedback arrives.
    /// Guarantees: generation-safe shared keys cannot resolve the replacement request or advance its bookmark.
    #[tokio::test]
    async fn stale_feedback_cannot_resolve_reused_slot() {
        let (bridge, mut receiver) = DeliveryBridge::new(&Limits::default());
        let source = source();
        let prepared = bridge
            .prepare(source.clone(), 0, || Ok((Some("old".into()), records())))
            .unwrap();
        let stale = prepared.waiter.as_ref().unwrap().feedback.calldata();
        drop(prepared);
        let mut delivery = Box::pin(bridge.deliver(source.clone(), Some("new".into()), records()));
        let queued = tokio::select! {
            queued = receiver.recv() => queued.unwrap(),
            _ = &mut delivery => panic!("must wait for feedback"),
        };
        bridge.feedback.resolve(stale, BatchOutcome::Ack);
        bridge.feedback.resolve(CallData::new(), BatchOutcome::Ack);
        assert!(!queued.feedback.is_closed());
        assert!(bridge.bookmark(&source).is_none());
        bridge
            .feedback
            .resolve(queued.feedback.calldata(), BatchOutcome::Ack);
        delivery.await.unwrap();
        assert_eq!(bridge.bookmark(&source).as_deref(), Some("new"));
        assert!(bridge.feedback.0.lock().unwrap().is_empty());
    }

    /// Scenario: a delivery is queued, acknowledged, rejected, or cancelled while awaiting feedback.
    /// Guarantees: enqueue cannot commit; only Ack advances progress, while failures release source capacity.
    #[tokio::test]
    async fn feedback_gates_progress_and_cancellation_releases_slot() {
        let (bridge, mut receiver) = DeliveryBridge::new(&Limits::default());
        let source = source();
        for outcome in [BatchOutcome::Ack, BatchOutcome::Nack] {
            let delivery = bridge.deliver(source.clone(), Some(format!("{outcome:?}")), records());
            tokio::pin!(delivery);
            let queued = tokio::select! {
                queued = receiver.recv() => queued.unwrap(),
                _ = &mut delivery => panic!("must wait for feedback"),
            };
            assert_eq!(
                bridge.bookmark(&source).as_deref(),
                if outcome == BatchOutcome::Ack {
                    None
                } else {
                    Some("Ack")
                }
            );
            assert!(
                bridge
                    .deliver(source.clone(), None, records())
                    .await
                    .is_err()
            );
            queued.feedback.send(outcome).unwrap();
            assert_eq!(delivery.await.is_ok(), outcome == BatchOutcome::Ack);
            assert_eq!(bridge.bookmark(&source).as_deref(), Some("Ack"));
        }
        let mut delivery =
            Box::pin(bridge.deliver(source.clone(), Some("cancelled".into()), records()));
        let queued = tokio::select! {
            queued = receiver.recv() => queued.unwrap(),
            _ = &mut delivery => panic!("must wait for feedback"),
        };
        drop(delivery);
        assert!(queued.feedback.is_closed());
        assert_eq!(bridge.bookmark(&source).as_deref(), Some("Ack"));
        bridge.deliver(source.clone(), None, None).await.unwrap();
        assert_eq!(bridge.bookmark(&source).as_deref(), Some("Ack"));
    }

    /// Scenario: marker-only batches arrive before and during an unresolved ordinary batch.
    /// Guarantees: markers emit no pdata and commit locally, but cannot overtake pending events.
    #[tokio::test]
    async fn bookmark_only_batches_respect_source_ordering() {
        let (bridge, mut receiver) = DeliveryBridge::new(&Limits::default());
        let source = source();
        bridge
            .deliver(source.clone(), Some("initial".into()), None)
            .await
            .unwrap();
        assert!(receiver.try_recv().is_err());
        let delivery = bridge.deliver(source.clone(), Some("mixed".into()), records());
        tokio::pin!(delivery);
        let queued = tokio::select! {
            queued = receiver.recv() => queued.unwrap(),
            _ = &mut delivery => panic!("ordinary events must wait for feedback"),
        };
        assert!(
            bridge
                .deliver(source.clone(), Some("overtaking".into()), None)
                .await
                .is_err()
        );
        queued.feedback.send(BatchOutcome::Nack).unwrap();
        assert!(delivery.await.is_err());
        assert_eq!(bridge.bookmark(&source).as_deref(), Some("initial"));
        bridge
            .deliver(source.clone(), Some("next".into()), None)
            .await
            .unwrap();
        assert_eq!(bridge.bookmark(&source).as_deref(), Some("next"));
        assert!(receiver.try_recv().is_err());
    }

    /// Scenario: validation and bookmark admission finish before handoff, including overlapping requests.
    /// Guarantees: shared metrics report admission refusals, export deltas once, and release unqueued reservations.
    #[test]
    fn local_admission_metrics_and_cancellation() {
        let (context, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests(
            otel_arrow_dfe_engine::Interests::NODE_OUTPUT_METRICS,
        );
        let metrics = Arc::new(Mutex::new(WefMetrics::register(&context)));
        let (bridge, _) = DeliveryBridge::new(&Limits::default());
        let bridge = bridge.with_metrics(Arc::clone(&metrics));
        let source = source();
        let prepared = bridge
            .prepare(source.clone(), 24, || Ok((None, records())))
            .unwrap();
        assert!(matches!(
            bridge.prepare(source.clone(), 24, || Ok((None, records()))),
            Err(PreparationError::Refused(_))
        ));
        assert!(matches!(
            bridge.prepare(source.clone(), 32, || Err("invalid".into())),
            Err(PreparationError::Invalid(_))
        ));
        let (receiver, mut reporter) =
            otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(16);
        metrics.lock().unwrap().report(&mut reporter).unwrap();
        let snapshots: Vec<_> = receiver.try_iter().collect();
        for (outcome, expected) in [("success", 1), ("refused", 2)] {
            let snapshot = snapshots
                .iter()
                .find(|snapshot| {
                    snapshot.descriptor().name == "receiver.received"
                        && snapshot.measurement_attribute_value("outcome") == Some(outcome)
                })
                .unwrap();
            assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), expected);
        }
        assert!(
            metrics
                .lock()
                .unwrap()
                .boundary
                .terminal_snapshots()
                .is_empty()
        );
        drop(prepared);
        assert!(bridge.prepare(source, 24, || Ok((None, records()))).is_ok());
    }
}
