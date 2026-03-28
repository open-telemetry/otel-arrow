// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::local;
use crate::shared;
use crate::{
    AckMsg, AdmissionClass, CompletionMsg, ControlChannelConfig, ControlCmd, DrainIngressMsg,
    LifecycleSendResult, NackMsg, NodeControlEvent, ReceiverControlEvent, SendOutcome, ShutdownMsg,
    TrySendError,
};
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
    ControlCmd::Nack(NackMsg::new("nack", value.to_owned()))
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

async fn collect_local_node_events(
    mut rx: local::LocalNodeControlReceiver<String>,
) -> Vec<NodeControlEvent<String>> {
    let mut events = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(Duration::from_millis(20), rx.recv()).await {
        events.push(event);
    }
    events
}

async fn collect_shared_node_events(
    mut rx: shared::SharedNodeControlReceiver<String>,
) -> Vec<NodeControlEvent<String>> {
    let mut events = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(Duration::from_millis(20), rx.recv()).await {
        events.push(event);
    }
    events
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_tokens_remain_deliverable_under_backlog() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::node_channel(ControlChannelConfig {
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
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::receiver_channel::<String>(test_config()).unwrap();

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
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::receiver_channel::<String>(test_config()).unwrap();

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
    let (tx, mut rx) = local::node_channel::<String>(test_config()).unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(nack("nack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.try_send(ack("ack-2")).unwrap(), SendOutcome::Accepted);

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Nack(NackMsg::new("nack", "nack-1".to_owned())),
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
    let (tx, mut rx) = local::node_channel::<String>(ControlChannelConfig {
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
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::node_channel::<String>(test_config()).unwrap();

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
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::node_channel::<String>(test_config()).unwrap();

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
    let deadline = Instant::now() + Duration::from_millis(20);
    let (tx, mut rx) = shared::node_channel::<String>(ControlChannelConfig {
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
async fn local_and_shared_variants_produce_the_same_event_sequence() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let script = vec![
        ack("ack-1"),
        nack("nack-1"),
        ControlCmd::TimerTick,
        ControlCmd::CollectTelemetry,
        ControlCmd::Config {
            config: serde_json::json!({"version": 1}),
        },
        ControlCmd::Config {
            config: serde_json::json!({"version": 2}),
        },
        ack("ack-2"),
    ];

    let (local_tx, local_rx) = local::node_channel::<String>(test_config()).unwrap();
    let (shared_tx, shared_rx) = shared::node_channel::<String>(test_config()).unwrap();

    for cmd in script {
        let local_result = local_tx.try_send(cmd.clone()).unwrap();
        let shared_result = shared_tx.try_send(cmd).unwrap();
        assert_eq!(local_result, shared_result);
    }

    assert_eq!(
        local_tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );
    assert_eq!(
        shared_tx.accept_shutdown(shutdown(deadline)),
        LifecycleSendResult::Accepted
    );

    local_tx.close();
    shared_tx.close();

    let local_events = collect_local_node_events(local_rx).await;
    let shared_events = collect_shared_node_events(shared_rx).await;

    assert_eq!(local_events, shared_events);
}

#[tokio::test(flavor = "current_thread")]
async fn receiver_returns_none_once_last_sender_drops_and_queue_is_empty() {
    let (tx, mut rx) = shared::node_channel::<String>(test_config()).unwrap();
    drop(tx);

    let result = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("receiver should wake when the last sender drops");
    assert_eq!(result, None);
}

#[tokio::test(flavor = "current_thread")]
async fn blocking_send_waits_for_capacity_then_completes() {
    let (tx, mut rx) = shared::node_channel::<String>(ControlChannelConfig {
        completion_msg_capacity: 1,
        completion_batch_max: 1,
        completion_burst_limit: 1,
    })
    .unwrap();

    assert_eq!(tx.try_send(ack("ack-1")).unwrap(), SendOutcome::Accepted);

    let blocked = tokio::spawn({
        let tx = tx.clone();
        async move { tx.send(ack("ack-2")).await }
    });

    tokio::task::yield_now().await;
    assert!(!blocked.is_finished());

    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-1".to_owned())
        )]))
    );
    assert_eq!(blocked.await.unwrap().unwrap(), SendOutcome::Accepted);
    assert_eq!(
        rx.recv().await,
        Some(NodeControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-2".to_owned())
        )]))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn try_send_returns_full_with_the_original_command() {
    let (tx, _rx) = local::node_channel::<String>(ControlChannelConfig {
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
async fn lifecycle_duplicate_is_rejected_before_delivery_and_closed_after_terminal_shutdown() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::node_channel::<String>(test_config()).unwrap();

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
    let (tx, _rx) = shared::receiver_channel::<String>(test_config()).unwrap();
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
    let (tx, _rx) = local::node_channel::<String>(test_config()).unwrap();

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
