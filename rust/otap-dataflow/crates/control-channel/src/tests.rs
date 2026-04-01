// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    AckMsg, AdmissionClass, CompletionMsg, ConfigError, ControlChannelConfig, ControlCmd,
    DrainIngressMsg, LifecycleSendResult, NodeControlEvent, ReceiverControlEvent, SendError,
    SendOutcome, ShutdownMsg, TrySendError, node_channel, node_channel_with_meta, receiver_channel,
};
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, Instant};

fn test_config() -> ControlChannelConfig {
    ControlChannelConfig {
        completion_msg_capacity: 4,
        completion_batch_max: 2,
        completion_burst_limit: 2,
    }
}

fn ack(value: &str) -> ControlCmd<String> {
    ControlCmd::Ack(AckMsg::new(value.to_owned()))
}

fn nack(value: &str) -> ControlCmd<String> {
    ControlCmd::Nack(crate::NackMsg::new("nack", value.to_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TestMeta {
    route: &'static str,
    seq: u64,
}

fn shutdown(deadline: Instant) -> ShutdownMsg {
    ShutdownMsg {
        deadline,
        reason: "shutdown".to_owned(),
    }
}

fn drain(deadline: Instant) -> DrainIngressMsg {
    DrainIngressMsg {
        deadline,
        reason: "drain".to_owned(),
    }
}

async fn is_pending_once<F>(mut future: Pin<&mut F>) -> bool
where
    F: Future,
{
    poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx).is_pending())).await
}

#[derive(Default)]
struct CountingWake {
    wake_count: AtomicUsize,
}

impl CountingWake {
    fn count(&self) -> usize {
        self.wake_count.load(Ordering::SeqCst)
    }
}

impl Wake for CountingWake {
    fn wake(self: Arc<Self>) {
        let _ = self.wake_count.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.wake_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn zero_completion_capacity_is_rejected_at_validation_and_construction() {
    // Scenario: retained completion traffic needs at least one slot, so zero
    // completion capacity must be rejected before constructing the channel.
    // Guarantees: config validation and both channel constructors reject this
    // impossible configuration before any send path can block forever.
    let config = ControlChannelConfig {
        completion_msg_capacity: 0,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    };

    assert_eq!(
        config.validate(),
        Err(ConfigError::ZeroCompletionMsgCapacity)
    );
    assert!(matches!(
        node_channel::<String>(config.clone()),
        Err(ConfigError::ZeroCompletionMsgCapacity)
    ));
    assert!(matches!(
        receiver_channel::<String>(config),
        Err(ConfigError::ZeroCompletionMsgCapacity)
    ));
}

#[test]
fn completion_batch_max_cannot_exceed_completion_capacity() {
    // Scenario: a config claims one completion batch can exceed the total
    // number of retained completions.
    // Guarantees: validation rejects this meaningless setting early so batch
    // sizing remains bounded by an explicit, coherent config relation.
    let config = ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 2,
        completion_burst_limit: 1,
    };

    assert_eq!(
        config.validate(),
        Err(ConfigError::CompletionBatchMaxExceedsCapacity)
    );
    assert!(matches!(
        node_channel::<String>(config.clone()),
        Err(ConfigError::CompletionBatchMaxExceedsCapacity)
    ));
    assert!(matches!(
        receiver_channel::<String>(config),
        Err(ConfigError::CompletionBatchMaxExceedsCapacity)
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_tokens_remain_deliverable_under_backlog() {
    // Scenario: completion backlog is saturated, but reserved-capacity
    // lifecycle delivery still gets shutdown through to the receiver.
    // Guarantees: lifecycle tokens bypass completion-capacity pressure and
    // remain deliverable even when regular completion admission is full.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = node_channel(ControlChannelConfig {
        completion_msg_capacity: 2,
        completion_batch_max: 2,
        completion_burst_limit: 2,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(ack("ack-2")).unwrap(), SendOutcome::Accepted);
    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );

    let first = rx.recv().await.expect("completion batch should arrive");
    let second = rx.recv().await.expect("shutdown should arrive");

    assert_eq!(
        first,
        NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-2".to_owned())),
        ])
    );
    assert_eq!(
        second,
        NodeControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn drain_ingress_precedes_shutdown_even_if_shutdown_arrives_first() {
    // Scenario: the engine records shutdown before receiver drain, but the
    // receiver must still observe `DrainIngress` before `Shutdown`.
    // Guarantees: delivery order gives receiver drain precedence over shutdown
    // regardless of lifecycle recording order.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = receiver_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(
        tx.accept_drain_ingress(drain(deadline)),
        LifecycleSendResult::Accepted
    );

    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::DrainIngress(DrainIngressMsg {
            deadline,
            reason: "drain".to_owned(),
        }))
    );
    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn config_and_best_effort_work_are_rejected_after_drain_ingress() {
    // Scenario: once receiver drain starts, normal control work becomes stale
    // and must be dropped while completion traffic continues to drain.
    // Guarantees: drain clears normal control semantics, preserves completion
    // draining, and records dropped-normal counters for observability.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = receiver_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.accept_drain_ingress(drain(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"ignored": true}),
        })
        .unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.try_send(ControlCmd::TimerTick).unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.try_send(ControlCmd::CollectTelemetry).unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let stats = tx.stats();
    assert!(stats.drain_ingress_recorded);
    assert!(!stats.shutdown_recorded);
    assert_eq!(stats.normal_event_dropped_during_drain_total, 3);

    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::DrainIngress(DrainIngressMsg {
            deadline,
            reason: "drain".to_owned(),
        }))
    );
    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned()))
        ]))
    );
    assert_eq!(rx.try_recv(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn completion_batching_preserves_arrival_order() {
    // Scenario: mixed `Ack` and `Nack` messages are batched without reordering.
    // Guarantees: completion batching preserves FIFO arrival order and updates
    // the emitted-batch and emitted-message counters consistently.
    let (tx, mut rx) = node_channel::<String>(test_config()).unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(nack("nack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(ack("ack-2")).unwrap(), SendOutcome::Accepted);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Nack(crate::NackMsg::new("nack", "nack-1".to_owned())),
        ]))
    );
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-2".to_owned())
        )]))
    );

    let stats = tx.stats();
    assert_eq!(stats.completion_batch_emitted_total, 2);
    assert_eq!(stats.completion_message_emitted_total, 3);
}

#[tokio::test(flavor = "current_thread")]
async fn completion_burst_limit_forces_pending_normal_work() {
    // Scenario: pending normal control work must break up long completion
    // bursts once the configured burst limit is reached.
    // Guarantees: the completion burst limit forces a normal event before more
    // completion traffic can continue once the configured limit is reached.
    let (tx, mut rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 8,
        completion_batch_max: 2,
        completion_burst_limit: 2,
    })
    .unwrap();

    for value in ["ack-1", "ack-2", "ack-3", "ack-4"] {
        assert_eq!(tx.try_send(ack(value)).unwrap(), SendOutcome::Accepted);
    }
    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"version": 1}),
        })
        .unwrap(),
        SendOutcome::Accepted
    );

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-2".to_owned())),
        ]))
    );
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::Config {
            config: serde_json::json!({"version": 1}),
        })
    );
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-3".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-4".to_owned())),
        ]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pending_normal_work_is_cleared_when_shutdown_is_accepted() {
    // Scenario: stale normal control work is discarded when shutdown is
    // accepted so terminal progress is not delayed by obsolete tokens.
    // Guarantees: shutdown admission clears pending normal work immediately and
    // allows terminal progress without delivering stale config or best-effort tokens.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = node_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.try_send(ControlCmd::TimerTick).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::CollectTelemetry).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"ignored": true}),
        })
        .unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
    assert_eq!(rx.try_recv(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn best_effort_work_is_suppressed_once_shutdown_is_recorded() {
    // Scenario: after shutdown is latched, completion traffic may still drain
    // but normal control work must be rejected.
    // Guarantees: shutdown preserves completion draining while rejecting new
    // normal control work and accounting for those drops.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = node_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(
        tx.try_send(ControlCmd::TimerTick).unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.try_send(ControlCmd::CollectTelemetry).unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"ignored": true}),
        })
        .unwrap(),
        SendOutcome::DroppedDuringDrain
    );

    let stats = tx.stats();
    assert!(stats.shutdown_recorded);
    assert_eq!(stats.normal_event_dropped_during_drain_total, 3);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-1".to_owned())
        )]))
    );
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
    assert_eq!(rx.try_recv(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_deadline_forces_terminal_progress() {
    // Scenario: shutdown reaches its deadline while completion backlog still
    // exists, so the queue must force terminal progress and abandon the rest.
    // Guarantees: the force deadline bounds shutdown latency, abandons any
    // remaining completion backlog, and closes the channel afterward.
    let deadline = Instant::now() + Duration::from_millis(20);
    let (tx, mut rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 4,
        completion_batch_max: 2,
        completion_burst_limit: 2,
    })
    .unwrap();

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(ack("ack-2")).unwrap(), SendOutcome::Accepted);

    tokio::time::sleep(Duration::from_millis(40)).await;

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
    assert_eq!(rx.try_recv(), None);

    let stats = tx.stats();
    assert!(stats.shutdown_recorded);
    assert!(stats.shutdown_forced);
    assert_eq!(stats.completion_abandoned_on_forced_shutdown_total, 2);
    assert!(stats.closed);

    match tx.try_send(ack("ack-3")) {
        Err(TrySendError::Closed(ControlCmd::Ack(ack))) => {
            assert_eq!(*ack.accepted, "ack-3".to_owned());
        }
        other => panic!("expected closed after forced shutdown, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn late_drain_ingress_is_rejected_after_shutdown_is_forced() {
    // Scenario: shutdown is recorded first, its deadline expires, and only
    // then a receiver-drain token is offered.
    // Guarantees: once forced shutdown has begun, late `DrainIngress` no
    // longer reintroduces a graceful-drain step ahead of terminal shutdown.
    let deadline = Instant::now() + Duration::from_millis(20);
    let (tx, mut rx) = receiver_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );

    tokio::time::sleep(Duration::from_millis(40)).await;

    assert_eq!(
        tx.accept_drain_ingress(drain(deadline)),
        LifecycleSendResult::Closed
    );
    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
    assert_eq!(rx.try_recv(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn receiver_returns_none_once_last_sender_drops_and_queue_is_empty() {
    // Scenario: dropping the last sender closes the queue and lets the
    // receiver terminate once there is no buffered work left.
    // Guarantees: last-sender drop transitions the channel to closed and the
    // receiver eventually observes `None` after draining buffered state.
    let (tx, mut rx) = node_channel::<String>(test_config()).unwrap();
    drop(tx);

    let result = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("receiver should wake when the last sender drops");
    assert_eq!(result, None);
}

#[tokio::test(flavor = "current_thread")]
async fn blocking_send_waits_for_capacity_then_completes() {
    // Scenario: a blocking send waits for bounded completion capacity and then
    // succeeds after the receiver drains one batch.
    // Guarantees: blocking completion sends park only until capacity is freed
    // and then complete with the original send outcome.
    let (tx, mut rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let mut blocked = std::pin::pin!(tx.send(ack("ack-2")));
    assert!(is_pending_once(blocked.as_mut()).await);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-1".to_owned())
        )]))
    );
    assert_eq!(blocked.await.unwrap(), SendOutcome::Accepted);
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-2".to_owned())
        )]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn drain_ingress_does_not_wake_blocked_completion_senders() {
    // Scenario: receiver drain begins while completion capacity is full and
    // a completion sender is blocked waiting for one retained completion slot.
    // Guarantees: `DrainIngress` wakes the channel receiver so lifecycle
    // precedence is preserved, but blocked completion senders stay asleep
    // until a later completion batch actually frees capacity.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = receiver_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let wake_counter = Arc::new(CountingWake::default());
    let waker = Waker::from(wake_counter.clone());
    let mut cx = Context::from_waker(&waker);
    let mut blocked = std::pin::pin!(tx.send(ack("ack-2")));
    assert!(matches!(blocked.as_mut().poll(&mut cx), Poll::Pending));
    assert_eq!(wake_counter.count(), 0);

    assert_eq!(
        tx.accept_drain_ingress(drain(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(wake_counter.count(), 0);
    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::DrainIngress(DrainIngressMsg {
            deadline,
            reason: "drain".to_owned(),
        }))
    );
    assert_eq!(wake_counter.count(), 0);

    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned()))
        ]))
    );
    assert_eq!(wake_counter.count(), 1);

    assert_eq!(blocked.await.unwrap(), SendOutcome::Accepted);
    assert_eq!(
        rx.recv().await,
        Some(ReceiverControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-2".to_owned()))
        ]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_batch_wakes_as_many_blocked_senders_as_slots_freed() {
    // Scenario: draining a completion batch should wake one blocked sender per
    // freed completion slot, not just one sender for the whole batch.
    // Guarantees: blocked-sender wakeups scale with the number of released
    // completion slots rather than degenerating into one-wakeup-per-batch behavior.
    let (tx, mut rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 2,
        completion_batch_max: 2,
        completion_burst_limit: 2,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(ack("ack-2")).unwrap(), SendOutcome::Accepted);

    let tx_clone = tx.clone();
    let mut blocked_one = std::pin::pin!(tx.send(ack("ack-3")));
    let mut blocked_two = std::pin::pin!(tx_clone.send(ack("ack-4")));
    assert!(is_pending_once(blocked_one.as_mut()).await);
    assert!(is_pending_once(blocked_two.as_mut()).await);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-2".to_owned())),
        ]))
    );

    assert_eq!(blocked_one.await.unwrap(), SendOutcome::Accepted);
    assert_eq!(blocked_two.await.unwrap(), SendOutcome::Accepted);
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-3".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-4".to_owned())),
        ]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn canceled_blocked_sender_does_not_steal_next_capacity_wakeup() {
    // Scenario: canceling a blocked sender should unregister its waiter so the
    // next capacity release wakes a live blocked sender instead of a stale one.
    // Guarantees: canceled blocked sends do not leave stale waiters that can
    // consume a future capacity wakeup meant for a live sender.
    let (tx, mut rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    {
        let mut canceled = std::pin::pin!(tx.send(ack("ack-2")));
        assert!(is_pending_once(canceled.as_mut()).await);
    }

    let tx_clone = tx.clone();
    let mut live = std::pin::pin!(tx_clone.send(ack("ack-3")));
    assert!(is_pending_once(live.as_mut()).await);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-1".to_owned())
        )]))
    );
    assert_eq!(live.await.unwrap(), SendOutcome::Accepted);
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-3".to_owned())
        )]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn close_wakes_blocked_senders_with_closed() {
    // Scenario: closing the channel must wake blocked senders so they return
    // `Closed` with the original command instead of waiting forever.
    // Guarantees: terminal close wakes parked senders immediately and preserves
    // ownership of the original command in the returned error.
    let (tx, _rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let mut blocked = std::pin::pin!(tx.send(ack("ack-2")));
    assert!(is_pending_once(blocked.as_mut()).await);

    tx.close();

    match blocked.await {
        Err(SendError::Closed(ControlCmd::Ack(ack))) => {
            assert_eq!(*ack.accepted, "ack-2".to_owned());
        }
        other => panic!("expected closed send after close(), got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn receiver_drop_wakes_blocked_senders_with_closed() {
    // Scenario: the channel receiver is dropped while a completion sender is
    // blocked on bounded capacity.
    // Guarantees: receiver drop closes the channel, wakes blocked senders
    // immediately, and returns the original command as `Closed`.
    let (tx, rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let mut blocked = std::pin::pin!(tx.send(ack("ack-2")));
    assert!(is_pending_once(blocked.as_mut()).await);

    drop(rx);

    match blocked.await {
        Err(SendError::Closed(ControlCmd::Ack(ack))) => {
            assert_eq!(*ack.accepted, "ack-2".to_owned());
        }
        other => panic!("expected closed send after receiver drop, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn receiver_drop_makes_future_sends_fail_closed() {
    // Scenario: the channel receiver is dropped before a sender attempts more
    // work, leaving no consumer for queued control events.
    // Guarantees: receiver drop closes the channel for future sends so control
    // messages are not accepted into a channel with no consumer.
    let (tx, rx) = node_channel::<String>(test_config()).unwrap();

    drop(rx);

    match tx.try_send(ack("ack-1")) {
        Err(TrySendError::Closed(ControlCmd::Ack(ack))) => {
            assert_eq!(*ack.accepted, "ack-1".to_owned());
        }
        other => panic!("expected closed try_send after receiver drop, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn try_send_returns_full_with_the_original_command() {
    // Scenario: `try_send` preserves the original command when bounded
    // backpressured completion admission is full.
    // Guarantees: a full error reports the backpressured admission class and
    // returns the original command unchanged to the caller.
    let (tx, _rx) = node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    match tx.try_send(nack("nack-1")) {
        Err(TrySendError::Full {
            admission_class: AdmissionClass::Backpressured,
            cmd: ControlCmd::Nack(nack),
        }) => {
            assert_eq!(nack.reason, "nack");
            assert_eq!(*nack.refused, "nack-1".to_owned());
        }
        other => panic!("expected full completion error, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_metadata_survives_batching_and_full_errors() {
    // Scenario: explicit completion metadata must survive batching and full
    // backpressure errors so future engine integration can preserve unwind state.
    // Guarantees: completion metadata is preserved both when a completion is
    // delivered in a batch and when `try_send` returns the original command.
    let (tx, mut rx) = node_channel_with_meta::<String, TestMeta>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    let first_meta = TestMeta {
        route: "batch",
        seq: 1,
    };
    let second_meta = TestMeta {
        route: "full",
        seq: 2,
    };

    assert_eq!(
        tx.try_send(ControlCmd::Ack(AckMsg::with_meta(
            "ack-1".to_owned(),
            first_meta.clone(),
        )))
        .unwrap(),
        SendOutcome::Accepted
    );

    match tx.try_send(ControlCmd::Nack(crate::NackMsg::with_meta(
        "nack",
        "nack-1".to_owned(),
        second_meta.clone(),
    ))) {
        Err(TrySendError::Full {
            admission_class: AdmissionClass::Backpressured,
            cmd: ControlCmd::Nack(nack),
        }) => {
            assert_eq!(nack.reason, "nack");
            assert_eq!(*nack.refused, "nack-1".to_owned());
            assert_eq!(nack.meta, second_meta);
        }
        other => panic!("expected full completion error with preserved metadata, got {other:?}"),
    }

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::with_meta("ack-1".to_owned(), first_meta)
        )]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_duplicate_is_rejected_before_delivery_and_closed_after_terminal_shutdown() {
    // Scenario: duplicate lifecycle offers are rejected while pending, and the
    // lifecycle path closes after terminal shutdown has been delivered.
    // Guarantees: lifecycle recording is idempotent while pending and becomes
    // permanently closed once terminal shutdown delivery completes.
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = node_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(
        tx.accept_shutdown(shutdown(deadline + Duration::from_secs(1))),
        LifecycleSendResult::AlreadyAccepted
    );
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );

    assert_eq!(
        tx.accept_shutdown(shutdown(deadline + Duration::from_secs(1))),
        LifecycleSendResult::Closed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_accept_returns_closed_after_sender_close() {
    // Scenario: once senders close the channel, reserved lifecycle acceptance
    // also reports the closed state.
    // Guarantees: lifecycle admission shares the same terminal closed state as
    // regular send paths after sender-driven close.
    let (tx, _rx) = receiver_channel::<String>(test_config()).unwrap();
    tx.close();

    assert_eq!(
        tx.accept_drain_ingress(drain(Instant::now() + Duration::from_secs(1))),
        LifecycleSendResult::Closed
    );
    assert_eq!(
        tx.accept_shutdown(shutdown(Instant::now() + Duration::from_secs(1))),
        LifecycleSendResult::Closed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn stats_track_config_replacement_and_best_effort_coalescing() {
    // Scenario: stats expose replacement and coalescing counters while normal
    // control events remain pending.
    // Guarantees: stats reflect latest-wins config replacement, best-effort
    // coalescing, and the presence of still-pending normal control work.
    let (tx, _rx) = node_channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"version": 1}),
        })
        .unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::Config {
            config: serde_json::json!({"version": 2}),
        })
        .unwrap(),
        SendOutcome::Replaced
    );
    assert_eq!(
        tx.try_send(ControlCmd::TimerTick).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::TimerTick).unwrap(),
        SendOutcome::Coalesced
    );
    assert_eq!(
        tx.try_send(ControlCmd::CollectTelemetry).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.try_send(ControlCmd::CollectTelemetry).unwrap(),
        SendOutcome::Coalesced
    );

    let stats = tx.stats();
    assert_eq!(stats.config_replaced_total, 1);
    assert_eq!(stats.timer_tick_coalesced_total, 1);
    assert_eq!(stats.collect_telemetry_coalesced_total, 1);
    assert!(stats.has_pending_config);
    assert!(stats.has_pending_timer_tick);
    assert!(stats.has_pending_collect_telemetry);
}
