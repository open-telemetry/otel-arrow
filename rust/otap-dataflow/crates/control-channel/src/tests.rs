// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::local;
use crate::shared;
use crate::{
    AckMsg, CompletionMsg, ControlChannelConfig, ControlCmd, ControlEvent, DelayedDataMsg,
    DrainIngressMsg, NackMsg, SendOutcome, ShutdownMsg, TelemetrySourceId, TimerSourceId,
};
use std::time::{Duration, Instant};

fn test_config() -> ControlChannelConfig {
    ControlChannelConfig {
        completion_msg_capacity: 4,
        completion_batch_max: 2,
        delayed_data_capacity: 2,
        timer_sources_capacity: 4,
        telemetry_sources_capacity: 4,
    }
}

fn ack(value: &str) -> ControlCmd<String> {
    ControlCmd::Ack(AckMsg::new(value.to_owned()))
}

fn nack(value: &str) -> ControlCmd<String> {
    ControlCmd::Nack(NackMsg::new("nack", value.to_owned()))
}

fn delayed(value: &str, when: Instant) -> ControlCmd<String> {
    ControlCmd::DelayedData(DelayedDataMsg::new(when, value.to_owned()))
}

fn shutdown(deadline: Instant) -> ControlCmd<String> {
    ControlCmd::Shutdown(ShutdownMsg {
        deadline,
        reason: "shutdown".to_owned(),
    })
}

fn drain(deadline: Instant) -> ControlCmd<String> {
    ControlCmd::DrainIngress(DrainIngressMsg {
        deadline,
        reason: "drain".to_owned(),
    })
}

async fn collect_local_events(
    mut rx: local::LocalControlReceiver<String>,
) -> Vec<ControlEvent<String>> {
    let mut events = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(Duration::from_millis(20), rx.recv()).await {
        events.push(event);
    }
    events
}

async fn collect_shared_events(
    mut rx: shared::SharedControlReceiver<String>,
) -> Vec<ControlEvent<String>> {
    let mut events = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(Duration::from_millis(20), rx.recv()).await {
        events.push(event);
    }
    events
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_tokens_remain_deliverable_under_backlog() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let when = Instant::now() + Duration::from_secs(5);
    let (tx, mut rx) = local::channel(ControlChannelConfig {
        completion_msg_capacity: 2,
        completion_batch_max: 2,
        delayed_data_capacity: 1,
        timer_sources_capacity: 1,
        telemetry_sources_capacity: 1,
    })
    .unwrap();

    assert_eq!(tx.send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.send(ack("ack-2")).unwrap(), SendOutcome::Accepted);
    assert_eq!(
        tx.send(delayed("retry-1", when)).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(tx.send(shutdown(deadline)).unwrap(), SendOutcome::Accepted);

    let first = rx.recv().await.expect("completion batch should arrive");
    let second = rx.recv().await.expect("delayed data should arrive");
    let third = rx.recv().await.expect("shutdown should arrive");

    assert_eq!(
        first,
        ControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Ack(AckMsg::new("ack-2".to_owned())),
        ])
    );
    assert_eq!(
        second,
        ControlEvent::DelayedData(DelayedDataMsg::new(when, "retry-1".to_owned()))
    );
    assert_eq!(
        third,
        ControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn drain_ingress_precedes_shutdown_even_if_shutdown_arrives_first() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::channel::<String>(test_config()).unwrap();

    assert_eq!(tx.send(shutdown(deadline)).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.send(drain(deadline)).unwrap(), SendOutcome::Accepted);

    let first = rx.recv().await.expect("drain ingress should arrive");
    let second = rx.recv().await.expect("shutdown should arrive");

    assert_eq!(
        first,
        ControlEvent::DrainIngress(DrainIngressMsg {
            deadline,
            reason: "drain".to_owned(),
        })
    );
    assert_eq!(
        second,
        ControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_batching_preserves_arrival_order() {
    let (tx, mut rx) = local::channel::<String>(test_config()).unwrap();

    assert_eq!(tx.send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.send(nack("nack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.send(ack("ack-2")).unwrap(), SendOutcome::Accepted);

    let first = rx.recv().await.expect("first batch should arrive");
    let second = rx.recv().await.expect("second batch should arrive");

    assert_eq!(
        first,
        ControlEvent::CompletionBatch(vec![
            CompletionMsg::Ack(AckMsg::new("ack-1".to_owned())),
            CompletionMsg::Nack(NackMsg::new("nack", "nack-1".to_owned())),
        ])
    );
    assert_eq!(
        second,
        ControlEvent::CompletionBatch(vec![CompletionMsg::Ack(AckMsg::new("ack-2".to_owned()))])
    );
}

#[tokio::test(flavor = "current_thread")]
async fn timer_telemetry_and_config_are_coalesced_or_replaced() {
    let (tx, mut rx) = local::channel::<String>(test_config()).unwrap();

    assert_eq!(
        tx.send(ControlCmd::TimerTick {
            source: TimerSourceId(7),
        })
        .unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.send(ControlCmd::TimerTick {
            source: TimerSourceId(7),
        })
        .unwrap(),
        SendOutcome::Coalesced
    );
    assert_eq!(
        tx.send(ControlCmd::CollectTelemetry {
            source: TelemetrySourceId(9),
        })
        .unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.send(ControlCmd::CollectTelemetry {
            source: TelemetrySourceId(9),
        })
        .unwrap(),
        SendOutcome::Coalesced
    );
    assert_eq!(
        tx.send(ControlCmd::Config {
            config: serde_json::json!({"v": 1}),
        })
        .unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.send(ControlCmd::Config {
            config: serde_json::json!({"v": 2}),
        })
        .unwrap(),
        SendOutcome::Replaced
    );

    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::Config {
            config: serde_json::json!({"v": 2}),
        })
    );
    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::TimerTick {
            source: TimerSourceId(7),
        })
    );
    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::CollectTelemetry {
            source: TelemetrySourceId(9),
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn best_effort_work_is_suppressed_once_shutdown_is_latched() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let when = Instant::now() + Duration::from_secs(5);
    let (tx, mut rx) = local::channel::<String>(test_config()).unwrap();

    assert_eq!(tx.send(shutdown(deadline)).unwrap(), SendOutcome::Accepted);
    assert_eq!(tx.send(ack("ack-1")).unwrap(), SendOutcome::Accepted);
    assert_eq!(
        tx.send(delayed("retry", when)).unwrap(),
        SendOutcome::Accepted
    );
    assert_eq!(
        tx.send(ControlCmd::TimerTick {
            source: TimerSourceId(1),
        })
        .unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.send(ControlCmd::CollectTelemetry {
            source: TelemetrySourceId(2),
        })
        .unwrap(),
        SendOutcome::DroppedDuringDrain
    );
    assert_eq!(
        tx.send(ControlCmd::Config {
            config: serde_json::json!({"ignored": true}),
        })
        .unwrap(),
        SendOutcome::DroppedDuringDrain
    );

    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::CompletionBatch(vec![CompletionMsg::Ack(
            AckMsg::new("ack-1".to_owned())
        )]))
    );
    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::DelayedData(DelayedDataMsg::new(
            when,
            "retry".to_owned()
        )))
    );
    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
    assert_eq!(rx.try_recv(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn duplicate_lifecycle_messages_are_rejected_without_replacing_the_first() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let (tx, mut rx) = local::channel::<String>(test_config()).unwrap();

    assert_eq!(tx.send(shutdown(deadline)).unwrap(), SendOutcome::Accepted);
    assert_eq!(
        tx.send(shutdown(deadline + Duration::from_secs(1)))
            .unwrap(),
        SendOutcome::DuplicateLifecycle
    );

    assert_eq!(
        rx.recv().await,
        Some(ControlEvent::Shutdown(ShutdownMsg {
            deadline,
            reason: "shutdown".to_owned(),
        }))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn local_and_shared_variants_produce_the_same_event_sequence() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let when = Instant::now() + Duration::from_secs(5);
    let script = vec![
        ack("ack-1"),
        nack("nack-1"),
        ControlCmd::TimerTick {
            source: TimerSourceId(1),
        },
        ControlCmd::CollectTelemetry {
            source: TelemetrySourceId(2),
        },
        ControlCmd::Config {
            config: serde_json::json!({"version": 1}),
        },
        ControlCmd::Config {
            config: serde_json::json!({"version": 2}),
        },
        delayed("retry-1", when),
        drain(deadline),
        ack("ack-2"),
        shutdown(deadline),
    ];

    let (local_tx, local_rx) = local::channel::<String>(test_config()).unwrap();
    let (shared_tx, shared_rx) = shared::channel::<String>(test_config()).unwrap();

    for cmd in script {
        let local_result = local_tx.send(cmd.clone()).unwrap();
        let shared_result = shared_tx.send(cmd).unwrap();
        assert_eq!(local_result, shared_result);
    }

    local_tx.close();
    shared_tx.close();

    let local_events = collect_local_events(local_rx).await;
    let shared_events = collect_shared_events(shared_rx).await;

    assert_eq!(local_events, shared_events);
}

#[tokio::test(flavor = "current_thread")]
async fn receiver_returns_none_once_last_sender_drops_and_queue_is_empty() {
    let (tx, mut rx) = shared::channel::<String>(test_config()).unwrap();
    drop(tx);

    let result = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("receiver should wake when the last sender drops");
    assert_eq!(result, None);
}
