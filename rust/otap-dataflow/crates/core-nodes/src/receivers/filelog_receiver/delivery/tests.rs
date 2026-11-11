// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::filelog_receiver::config::Config;
use otel_arrow_dfe_pdata::{logs, otap::OtapBatchStore, record_batch};
use std::{
    cell::Cell,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};

#[derive(Clone, Debug, thiserror::Error)]
#[error("builder test failure")]
pub(crate) struct BuilderError;

/// A builder double retains the original payload, progress and exact carry evidence.
pub(crate) struct Builder {
    pub records: OtapArrowRecords,
    pub progress: Vec<cp::UpdateProgress>,
    pub carry: Option<Vec<u8>>,
    pub promoted: Option<Vec<u8>>,
    pub released: bool,
    pub fail_release: bool,
    pub failed: bool,
    pub reseal: Option<cp::UpdateProgress>,
    pub drops: Rc<Cell<usize>>,
}
impl Drop for Builder {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
impl RetainedBatch for Builder {
    type Error = BuilderError;
    fn failure(&self) -> Option<Self::Error> {
        self.failed.then_some(BuilderError)
    }

    fn retained(&self) -> Option<BatchView<'_>> {
        (!self.released).then_some(BatchView {
            records: &self.records,
            progress: &self.progress,
        })
    }
    fn release_applied(
        &mut self,
        applied: &[cp::UpdateProgress],
        _: Instant,
    ) -> Result<(), Self::Error> {
        if self.fail_release {
            return Err(BuilderError);
        }
        assert_eq!(applied, self.progress);
        assert!(!self.released);
        self.released = true;
        self.promoted = self.carry.take();
        if let Some(next) = self.reseal.take() {
            self.progress = vec![next];
            self.released = false;
        }
        Ok(())
    }
}
pub(crate) fn builder(progress: Vec<cp::UpdateProgress>) -> Builder {
    Builder {
        records: OtapArrowRecords::Logs(logs!((
            Logs,
            ("id", UInt16, vec![0]),
            ("observed_time_unix_nano", TimestampNs, vec![42]),
            ("body.type", UInt8, vec![2]),
            ("body.str", Utf8, vec!["abc"])
        ))),
        progress,
        carry: Some(vec![0, 255, 13, 10, 42]),
        promoted: None,
        released: false,
        fail_release: false,
        failed: false,
        reseal: None,
        drops: Rc::new(Cell::new(0)),
    }
}
fn inputs(change: impl FnOnce(&mut Config)) -> ValidatedInputs {
    let mut config = Config {
        include: vec!["/tmp/*.log".into()],
        ..Config::default()
    };
    change(&mut config);
    config
        .compile("group", "pipeline", "filelog", std::path::Path::new("/tmp"))
        .unwrap()
        .into_parts()
        .0
}
fn delivery() -> Delivery<Builder> {
    let p = cp::UpdateProgress {
        file_id: cp::FileId::from_bytes([1; 16]),
        expected_file_epoch: 0,
        expected_committed_offset: 0,
        new_committed_offset: 3,
        new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc").unwrap(),
        new_framing_resume: cp::FramingResume::Clean,
        new_last_seen_time_unix_nano: 1,
        finalize: false,
    };
    Delivery::new(builder(vec![p]), &inputs(|c| c.retry.max_attempts = 2)).unwrap()
}
fn accepted(d: &mut Delivery<Builder>, now: Instant) -> Correlation {
    let a = d.next_attempt(now).unwrap().unwrap();
    d.sent(a.correlation, SendOutcome::Accepted, now).unwrap();
    a.correlation
}

/// Scenario: A Nack and a failed publication exhaust the initial budget, then a paused attempt succeeds.
/// Guarantees: Retries retain exact payload/progress/carry, use distinct identities and bounded deadlines, and never authorize progress before matching Ack.
#[test]
fn nack_and_pause_keep_exact_batch() {
    let mut d = delivery();
    let now = Instant::now();
    let original = d.builder.records.clone();
    let progress = d.builder.progress.clone();
    let carry = d.builder.carry.clone();
    let first = accepted(&mut d, now);
    assert!(d.complete(&first.calldata(), false, now).unwrap());
    let retry_at = d.next_deadline().unwrap();
    assert_eq!(retry_at.duration_since(now), Duration::from_millis(100));
    assert!(d.next_attempt(now).unwrap().is_none());
    let second = d.next_attempt(retry_at).unwrap().unwrap();
    assert_eq!(second.records, original);
    assert_ne!(second.correlation, first);
    d.sent(second.correlation, SendOutcome::Retryable, retry_at)
        .unwrap();
    let paused_at = d.next_deadline().unwrap();
    assert_eq!(paused_at.duration_since(retry_at), Duration::from_secs(5));
    assert_eq!(d.builder.progress, progress);
    assert_eq!(d.builder.carry, carry);
    assert!(!d.builder.released);
    assert!(d.builder_mut(now).is_none());
    assert!(!d.complete(&first.calldata(), true, paused_at).unwrap());
    let third = accepted(&mut d, paused_at);
    assert_eq!(third.attempt, 3);
    assert!(d.complete(&third.calldata(), true, paused_at).unwrap());
    assert_eq!(d.phase, Phase::Authorized);
    assert!(!d.builder.released);
}

/// Scenario: Malformed, wrong-batch, duplicate and superseded completion tuples arrive around sends.
/// Guarantees: Only the accepted current attempt can authorize checkpoint application.
#[test]
fn invalid_completions_cannot_authorize() {
    let mut d = delivery();
    let now = Instant::now();
    let a = d.next_attempt(now).unwrap().unwrap();
    d.sent(a.correlation, SendOutcome::Accepted, now).unwrap();
    for data in [
        CallData::new(),
        Correlation {
            batch: 20,
            attempt: 1,
        }
        .calldata(),
        Correlation {
            batch: 1,
            attempt: 2,
        }
        .calldata(),
    ] {
        assert!(!d.complete(&data, true, now).unwrap());
        assert_eq!(d.phase, Phase::Awaiting);
    }
    assert!(d.complete(&a.correlation.calldata(), true, now).unwrap());
    assert!(!d.complete(&a.correlation.calldata(), false, now).unwrap());
    assert_eq!(d.phase, Phase::Authorized);
    assert_eq!(d.stale_completions, 4);
}

/// Scenario: Cancellation arrives while waiting for completion or a retry deadline.
/// Guarantees: No new attempt, carry promotion or release occurs; teardown drops the owner exactly once.
#[test]
fn cancellation_retains_until_teardown() {
    for nack in [false, true] {
        let mut d = delivery();
        let drops = d.builder.drops.clone();
        let now = Instant::now();
        let id = accepted(&mut d, now);
        if nack {
            assert!(d.complete(&id.calldata(), false, now).unwrap());
        }
        d.cancel();
        assert!(d.next_deadline().is_none());
        assert!(
            d.next_attempt(now + Duration::from_secs(60))
                .unwrap()
                .is_none()
        );
        assert!(!d.complete(&id.calldata(), true, now).unwrap());
        assert!(!d.builder.released);
        assert!(d.builder.promoted.is_none());
        assert_eq!(drops.get(), 0);
        drop(d);
        assert_eq!(drops.get(), 1);
    }
}

/// Scenario: The next attempt identity cannot be represented.
/// Guarantees: Delivery stops without wrapping into an identity a stale completion could match.
#[test]
fn attempt_exhaustion_fails_closed() {
    let mut d = delivery();
    let now = Instant::now();
    d.phase = Phase::Backoff(now);
    d.correlation.attempt = u64::MAX;
    assert!(matches!(d.next_attempt(now), Err(DeliveryError::Exhausted)));
    assert_eq!(d.phase, Phase::Stopped);
}

pub(crate) fn handler(
    tx: otel_arrow_dfe_channel::mpsc::Sender<OtapPdata>,
) -> otel_arrow_dfe_engine::local::receiver::EffectHandler<OtapPdata> {
    use otel_arrow_dfe_engine::{
        control::runtime_ctrl_msg_channel,
        local::message::LocalSender,
        message::Sender,
        testing::{test_node, test_pipeline_runtime_services},
    };
    let (ctrl, _) = runtime_ctrl_msg_channel(1);
    let (_, metrics) =
        otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
    otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
        test_node("filelog"),
        [("out".into(), Sender::Local(LocalSender::mpsc(tx)))]
            .into_iter()
            .collect(),
        Some("out".into()),
        ctrl,
        metrics,
        test_pipeline_runtime_services(),
    )
}

/// Scenario: The engine output queue fills and a second subscribed send is cancelled while pending.
/// Guarantees: Publication carries both correlation fields and Ack/Nack interest; backpressure admits no extra message and cancellation preserves worker ownership.
#[tokio::test(flavor = "current_thread")]
async fn engine_send_subscribes_and_backpressure_is_cancellable() {
    let (tx, rx) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
    let handler = handler(tx);
    let mut d = delivery();
    let now = Instant::now();
    let attempt = d.next_attempt(now).unwrap().unwrap();
    let id = attempt.correlation;
    assert_eq!(
        send_attempt(&handler, attempt, std::future::pending())
            .await
            .unwrap(),
        SendOutcome::Accepted
    );
    d.sent(id, SendOutcome::Accepted, now).unwrap();
    // Keep the output full while arranging a retry, as a downstream could Nack before freeing capacity.
    assert!(d.complete(&id.calldata(), false, now).unwrap());
    let retry = d.next_attempt(d.next_deadline().unwrap()).unwrap().unwrap();
    let retry_id = retry.correlation;
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    let mut send = Box::pin(send_attempt(&handler, retry, async {
        let _ = cancel_rx.await;
    }));
    assert!(matches!(
        send.as_mut()
            .poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Pending
    ));
    assert!(d.next_attempt(now).unwrap().is_none());
    cancel_tx.send(()).unwrap();
    let outcome = send.await.unwrap();
    assert_eq!(outcome, SendOutcome::Cancelled);
    d.sent(retry_id, outcome, now).unwrap();
    let mut received = rx.try_recv().unwrap();
    let frame = received.context_mut().pop_frame().unwrap();
    assert_eq!(frame.route.calldata, id.calldata());
    assert!(frame.interests.contains(Interests::ACKS | Interests::NACKS));
    assert!(rx.try_recv().is_err());
    assert!(!d.builder.released);
}

/// Scenario: Explicit fail policy reaches its initial attempt budget without an Ack.
/// Guarantees: Exhaustion stops delivery without releasing the retained batch or promoting carry-over.
#[test]
fn fail_policy_never_authorizes_loss() {
    let mut d = delivery();
    d.exhaustion = RetryExhaustion::Fail;
    d.max_attempts = 1;
    let now = Instant::now();
    let id = accepted(&mut d, now);
    assert!(matches!(
        d.complete(&id.calldata(), false, now),
        Err(DeliveryError::RetryExhausted)
    ));
    assert_eq!(d.phase, Phase::Stopped);
    assert!(!d.builder.released);
    assert!(d.builder.promoted.is_none());
}

/// Scenario: An operator-selected loss policy reaches this internal delivery implementation.
/// Guarantees: Construction rejects unsupported loss authorization before any send or progress application.
#[test]
fn unsupported_loss_policy_is_rejected() {
    let config = inputs(|c| c.on_retry_exhaustion = RetryExhaustion::DropAndContinue);
    assert!(matches!(
        Delivery::new(builder(Vec::new()), &config),
        Err(DeliveryError::UnsupportedLossPolicy)
    ));
}

/// Scenario: A matching Ack or Nack is forwarded before the accepted-send result.
/// Guarantees: The host ordering violation stops delivery visibly without releasing or authorizing the batch.
#[test]
fn completion_before_send_result_is_terminal() {
    for ack in [true, false] {
        let mut d = delivery();
        let now = Instant::now();
        let a = d.next_attempt(now).unwrap().unwrap();
        let wrong = Correlation {
            batch: 99,
            attempt: 1,
        }
        .calldata();
        assert!(!d.complete(&wrong, ack, now).unwrap());
        assert!(!d.is_stopped());
        assert!(matches!(
            d.complete(&a.correlation.calldata(), ack, now),
            Err(DeliveryError::Invariant)
        ));
        assert!(d.is_stopped());
        assert!(!d.builder.released);
        assert!(d.next_attempt(now).unwrap().is_none());
        assert!(d.sent(a.correlation, SendOutcome::Accepted, now).is_err());
    }
}

/// Scenario: A builder reports a terminal failure before delivery selects its retained data.
/// Guarantees: Source access is blocked and the original builder error is preserved without publication.
#[test]
fn builder_failure_stops_before_send() {
    let mut d = delivery();
    d.builder.failed = true;
    let now = Instant::now();
    assert!(d.builder_mut(now).is_none());
    let Err(DeliveryError::Batch(error)) = d.next_attempt(now) else {
        panic!("expected builder failure")
    };
    assert!(error.downcast_ref::<BuilderError>().is_some());
    assert!(d.is_stopped());
}
