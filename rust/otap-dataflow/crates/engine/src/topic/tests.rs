// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests covering all subscription modes, ack/nack, backpressure,
//! multi-threaded correctness, topic lifecycle, and TopicSet.
//!
//! Tests are grouped by section headers. The test names follow the pattern
//! `<feature>_<scenario>` and are designed to be self-documenting.
//!
//! # Key Properties Verified
//!
//! - **Balanced**: exactly-once delivery within a group, per-subscriber ordering,
//!   no cross-group interference, backpressure blocking.
//! - **Broadcast**: all-receive, in-order delivery, lag reporting for slow
//!   subscribers, slow subscribers don't block fast ones.
//! - **Mode enforcement**: `BalancedOnly` rejects broadcast (and vice versa),
//!   `BalancedOnly` enforces single consumer group.
//! - **Ack/nack**: correct per-publisher routing, channel-full handling,
//!   disabled-when-not-configured.
//! - **Lifecycle**: `remove_topic` closes the topic (publish fails, recv gets
//!   Closed), removed topics can be recreated independently, `close_all` shuts
//!   everything down.
//! - **TopicSet**: insert/get/remove semantics, overwrite returns previous,
//!   remove-does-not-close, per-set ack routing, clone-shares-state.

use crate::error::Error;
use crate::topic::backend::InMemoryBackend;
use crate::topic::types::{
    AckEvent, AckStatus, RecvItem, SubscriberOptions, SubscriptionMode, TopicOptions,
};
use crate::topic::{TopicBroker, TopicSet};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

// =========================================================================
// Balanced mode – single group
// =========================================================================

// Two subscribers in the same consumer group receive all 100 messages between
// them with no duplicates and no losses.
#[tokio::test]
async fn balanced_single_group_all_messages_delivered_exactly_once() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic("test", TopicOptions::default())
        .unwrap();

    let mut sub1 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    let n = 100u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut received = HashSet::new();
    let mut order1 = Vec::new();
    let mut order2 = Vec::new();

    loop {
        tokio::select! {
            r = sub1.recv() => {
                match r {
                    Ok(RecvItem::Message(env)) => {
                        assert!(received.insert(env.id), "duplicate id={}", env.id);
                        order1.push(*env.payload);
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
            r = sub2.recv() => {
                match r {
                    Ok(RecvItem::Message(env)) => {
                        assert!(received.insert(env.id), "duplicate id={}", env.id);
                        order2.push(*env.payload);
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
        }
    }

    // Drain remaining from both.
    while let Ok(RecvItem::Message(env)) = sub1.recv().await {
        assert!(received.insert(env.id), "duplicate id={}", env.id);
        order1.push(*env.payload);
    }
    while let Ok(RecvItem::Message(env)) = sub2.recv().await {
        assert!(received.insert(env.id), "duplicate id={}", env.id);
        order2.push(*env.payload);
    }

    // All messages delivered.
    let all_payloads: HashSet<u64> = order1.iter().chain(order2.iter()).copied().collect();
    assert_eq!(all_payloads.len(), n as usize);
    for i in 0..n {
        assert!(all_payloads.contains(&i), "missing payload {i}");
    }
}

// Each subscriber within a consumer group receives its subset of messages in
// strictly increasing ID order.
#[tokio::test]
async fn balanced_single_group_preserves_order_per_subscriber() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic("test-order", TopicOptions::default())
        .unwrap();

    let mut sub1 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    let n = 200u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut ids1 = Vec::new();
    let mut ids2 = Vec::new();

    while let Ok(RecvItem::Message(env)) = sub1.recv().await {
        ids1.push(env.id);
    }
    while let Ok(RecvItem::Message(env)) = sub2.recv().await {
        ids2.push(env.id);
    }

    // Each subscriber's stream must be in strictly increasing id order.
    for w in ids1.windows(2) {
        assert!(w[0] < w[1], "out of order in sub1: {} >= {}", w[0], w[1]);
    }
    for w in ids2.windows(2) {
        assert!(w[0] < w[1], "out of order in sub2: {} >= {}", w[0], w[1]);
    }
}

// Four subscribers in one group collectively receive all 500 messages with no
// duplicate IDs.
#[tokio::test]
async fn balanced_no_duplicates_within_group() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic("dup-test", TopicOptions::default())
        .unwrap();

    let num_subs = 4;
    let mut subs: Vec<_> = (0..num_subs)
        .map(|_| {
            topic
                .subscribe(
                    SubscriptionMode::Balanced { group: "g1".into() },
                    SubscriberOptions::default(),
                )
                .unwrap()
        })
        .collect();

    let n = 500u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut all_ids = HashSet::new();
    for sub in subs.iter_mut() {
        while let Ok(RecvItem::Message(env)) = sub.recv().await {
            assert!(all_ids.insert(env.id), "duplicate id={}", env.id);
        }
    }
    assert_eq!(all_ids.len(), n as usize);
}

// =========================================================================
// Balanced mode – multiple groups
// =========================================================================

// Two consumer groups on the same topic each receive all messages independently,
// within each group delivery is split across subscribers with ordering preserved.
#[tokio::test]
async fn balanced_multiple_groups_independent() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic("multi-group", TopicOptions::default())
        .unwrap();

    let mut sub_g1_a = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: "group-A".into(),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub_g1_b = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: "group-A".into(),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub_g2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: "group-B".into(),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let n = 100u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    // Group A: messages split between sub_g1_a and sub_g1_b.
    let mut ga_ids = HashSet::new();
    let mut ga_a_ids = Vec::new();
    let mut ga_b_ids = Vec::new();
    while let Ok(RecvItem::Message(env)) = sub_g1_a.recv().await {
        ga_a_ids.push(env.id);
        _ = ga_ids.insert(env.id);
    }
    while let Ok(RecvItem::Message(env)) = sub_g1_b.recv().await {
        ga_b_ids.push(env.id);
        assert!(ga_ids.insert(env.id), "duplicate in group-A");
    }
    assert_eq!(ga_ids.len(), n as usize);

    // Group B: sub_g2 gets all messages.
    let mut gb_ids = HashSet::new();
    let mut gb_order = Vec::new();
    while let Ok(RecvItem::Message(env)) = sub_g2.recv().await {
        _ = gb_ids.insert(env.id);
        gb_order.push(env.id);
    }
    assert_eq!(gb_ids.len(), n as usize);

    // Group B ordering preserved.
    for w in gb_order.windows(2) {
        assert!(w[0] < w[1]);
    }

    // Both groups ordered independently.
    for w in ga_a_ids.windows(2) {
        assert!(w[0] < w[1]);
    }
    for w in ga_b_ids.windows(2) {
        assert!(w[0] < w[1]);
    }
}

// =========================================================================
// Broadcast mode
// =========================================================================

// Three broadcast subscribers each receive all 100 messages in the exact
// published order.
#[tokio::test]
async fn broadcast_all_subscribers_see_all_messages_in_order() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic(
            "broadcast-test",
            TopicOptions::Mixed {
                balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                broadcast_capacity: 1024,
            },
        )
        .unwrap();

    let mut sub1 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut sub2 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut sub3 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    let n = 100u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    for sub in [&mut sub1, &mut sub2, &mut sub3] {
        let mut received = Vec::new();
        while let Ok(RecvItem::Message(env)) = sub.recv().await {
            received.push(*env.payload);
        }
        assert_eq!(received.len(), n as usize);
        for (i, &val) in received.iter().enumerate() {
            assert_eq!(val, i as u64);
        }
    }
}

// A slow broadcast subscriber with a small ring buffer (8 slots) receives a
// Lagged notification when the publisher overwrites unread slots.
#[tokio::test]
async fn broadcast_lag_reported_on_slow_subscriber() {
    let broker = TopicBroker::new();
    // Small broadcast buffer to force lag.
    let topic = broker
        .create_in_memory_topic(
            "broadcast-lag",
            TopicOptions::Mixed {
                balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                broadcast_capacity: 8,
            },
        )
        .unwrap();

    let mut sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    // Publish more messages than the buffer can hold without consuming.
    let n = 50u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut messages = Vec::new();
    let mut total_lagged = 0u64;
    loop {
        match sub.recv().await {
            Ok(RecvItem::Message(env)) => messages.push(*env.payload),
            Ok(RecvItem::Lagged { missed }) => total_lagged += missed,
            Err(_) => break,
        }
    }

    // We should have fewer messages than n because some were dropped.
    assert!(
        messages.len() < n as usize,
        "expected lag but got all {} messages",
        n
    );
    assert!(total_lagged > 0, "expected lag notification");

    // The messages we did receive should be in order.
    for w in messages.windows(2) {
        assert!(w[0] < w[1], "out of order: {} >= {}", w[0], w[1]);
    }
}

// A slow subscriber that never reads does not block the publisher or prevent a
// fast subscriber from receiving every message in real time.
#[tokio::test]
async fn broadcast_slow_subscriber_does_not_block_fast_subscriber() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic(
            "broadcast-no-block",
            TopicOptions::Mixed {
                balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                broadcast_capacity: 4,
            },
        )
        .unwrap();

    let _slow_sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut fast_sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    // Publish enough to overflow slow_sub buffer, but fast_sub consumes immediately.
    for i in 0..20u64 {
        topic.publish(Arc::new(i)).await.unwrap();
        // Fast subscriber consumes each message right away.
        match fast_sub.recv().await {
            Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, i),
            other => panic!("unexpected: {:?}", other),
        }
    }
    // If slow_sub blocked the publisher, we'd never get here.
}

// In Mixed mode, broadcast delivery remains non-blocking even if balanced
// consumer-group backpressure stalls publish completion.
#[tokio::test]
async fn mixed_broadcast_not_blocked_by_balanced_backpressure() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "mixed-bcast-priority",
            TopicOptions::Mixed {
                balanced_capacity: 1,
                broadcast_capacity: 16,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut balanced = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    // Fill the balanced queue.
    topic.publish(Arc::new(1u64)).await.unwrap();

    // Second publish should block on balanced delivery, but broadcast should
    // still observe it promptly.
    let topic_clone = topic.clone();
    let second_publish = tokio::spawn(async move {
        topic_clone.publish(Arc::new(2u64)).await.unwrap();
    });

    // First message.
    match broadcast.recv().await {
        Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected: {:?}", other),
    }

    // Second message should arrive before balanced queue is drained.
    let second = tokio::time::timeout(std::time::Duration::from_millis(200), async {
        loop {
            match broadcast.recv().await {
                Ok(RecvItem::Message(env)) => break *env.payload,
                Ok(RecvItem::Lagged { .. }) => continue,
                Err(e) => panic!("unexpected recv error: {e:?}"),
            }
        }
    })
    .await
    .expect("broadcast delivery should not wait on balanced backpressure");
    assert_eq!(second, 2);

    // Publisher task should still be blocked on balanced queue.
    assert!(
        !second_publish.is_finished(),
        "second publish should still be blocked by balanced backpressure"
    );

    // Drain balanced queue to unblock publisher.
    _ = balanced.recv().await.unwrap();
    _ = balanced.recv().await.unwrap();
    second_publish.await.unwrap();
    topic.close();
}

// =========================================================================
// Subscriber choice – publisher API identical
// =========================================================================

// A single publish call delivers the message to both a balanced and a broadcast
// subscriber, proving the publisher API is mode-agnostic.
#[tokio::test]
async fn publisher_api_identical_for_all_subscriber_modes() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("choice-test", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    // Mix of balanced and broadcast subscribers.
    let mut balanced = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    // Publisher doesn't know or care about subscriber modes.
    topic.publish(Arc::new("hello".to_string())).await.unwrap();
    topic.close();

    // Both receive the message.
    match balanced.recv().await {
        Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, "hello"),
        other => panic!("unexpected: {:?}", other),
    }
    match broadcast.recv().await {
        Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, "hello"),
        other => panic!("unexpected: {:?}", other),
    }
}

// =========================================================================
// Ack/Nack
// =========================================================================

// Publishing via a handle created with `with_ack_sender` routes ack and nack
// events to the registered channel with correct status, message ID, and topic.
#[tokio::test]
async fn ack_nack_events_received_when_enabled() {
    let (ack_tx, mut ack_rx) = mpsc::channel::<AckEvent>(64);

    let broker = TopicBroker::new();
    let base = broker
        .create_topic("ack-test", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let topic = base.with_ack_sender(ack_tx);

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    topic.publish(Arc::new(42)).await.unwrap();
    topic.publish(Arc::new(43)).await.unwrap();

    let env1 = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };
    let env2 = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    sub.ack(env1.id).unwrap();
    sub.nack(env2.id, "bad data").unwrap();

    let evt1 = ack_rx.recv().await.unwrap();
    assert_eq!(evt1.message_id, env1.id);
    assert_eq!(evt1.status, AckStatus::Ack);
    assert!(evt1.reason.is_none());
    assert_eq!(evt1.topic.as_ref(), "ack-test");
    assert_ne!(evt1.publisher_id, 0); // per-publisher

    let evt2 = ack_rx.recv().await.unwrap();
    assert_eq!(evt2.message_id, env2.id);
    assert_eq!(evt2.status, AckStatus::Nack);
    assert_eq!(&*evt2.reason.unwrap(), "bad data");
    assert_ne!(evt2.publisher_id, 0); // per-publisher
}

// Publishing without with_ack_sender (publisher_id=0) means ack() returns
// AckError::NotEnabled.
#[tokio::test]
async fn ack_nack_disabled_when_no_ack_sender() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("no-ack", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    // No with_ack_sender() call → publisher_id=0 → NotEnabled.
    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    topic.publish(Arc::new(1)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    match sub.ack(env.id) {
        Err(Error::AckNotEnabled) => {}
        other => panic!("expected NotEnabled, got {:?}", other),
    }
}

// When the ack channel (capacity=1) is full, subsequent ack() calls return
// AckChannelFull instead of blocking.
#[tokio::test]
async fn ack_channel_full_returns_error() {
    // Tiny ack channel to force overflow.
    let (ack_tx, _ack_rx) = mpsc::channel::<AckEvent>(1);

    let broker = TopicBroker::new();
    let base = broker
        .create_topic("ack-full", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let topic = base.with_ack_sender(ack_tx);

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    for i in 0..10u64 {
        topic.publish(Arc::new(i)).await.unwrap();
    }

    // Consume and ack all without draining ack channel.
    let mut full_count = 0;
    for _ in 0..10 {
        let env = match sub.recv().await.unwrap() {
            RecvItem::Message(e) => e,
            _ => panic!(),
        };
        if sub.ack(env.id).is_err() {
            full_count += 1;
        }
    }

    // At least some should have been dropped due to channel full.
    assert!(full_count > 0, "expected some ack events to be dropped");
}

// =========================================================================
// Backpressure
// =========================================================================

// A balanced topic with capacity=2 blocks the third publish until the
// subscriber drains a message, verifying backpressure.
#[tokio::test]
async fn balanced_backpressure_blocks_publisher() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "backpressure",
            TopicOptions::Mixed {
                balanced_capacity: 2,
                broadcast_capacity: TopicOptions::DEFAULT_BROADCAST_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Fill the buffer.
    topic.publish(Arc::new(1)).await.unwrap();
    topic.publish(Arc::new(2)).await.unwrap();

    // Next publish should block because buffer is full.
    let topic_clone = topic.clone();
    let handle = tokio::spawn(async move {
        topic_clone.publish(Arc::new(3)).await.unwrap();
    });

    // Give the publisher task a moment to block.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(!handle.is_finished(), "publish should be blocked");

    // Consume one message to unblock.
    let _ = sub.recv().await.unwrap();

    // Now the publisher should complete.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(handle.is_finished(), "publish should have completed");
}

// =========================================================================
// Multi-threaded correctness
// =========================================================================

// Four concurrent publisher tasks send 250 messages each; two subscribers in
// the same group collect all 1000 with no duplicates across real OS threads.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn balanced_multi_threaded_no_duplicates() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("mt-balanced", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let num_publishers = 4;
    let msgs_per_publisher = 250;
    let total = num_publishers * msgs_per_publisher;

    // Create subscribers BEFORE publishers so the consumer group exists.
    let mut sub1 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Spawn publishers on different tasks (potentially different threads).
    let mut pub_handles = Vec::new();
    for p in 0..num_publishers {
        let t = topic.clone();
        pub_handles.push(tokio::spawn(async move {
            for i in 0..msgs_per_publisher {
                let val = (p * msgs_per_publisher + i) as u64;
                t.publish(Arc::new(val)).await.unwrap();
            }
        }));
    }

    // Wait for publishers.
    for h in pub_handles {
        h.await.unwrap();
    }
    topic.close();

    let mut all_ids = HashSet::new();
    while let Ok(RecvItem::Message(env)) = sub1.recv().await {
        assert!(all_ids.insert(env.id));
    }
    while let Ok(RecvItem::Message(env)) = sub2.recv().await {
        assert!(all_ids.insert(env.id));
    }
    assert_eq!(all_ids.len(), total);
}

// A publisher task and two subscriber tasks run on separate OS threads; both
// subscribers receive all 500 messages in order with a large enough ring buffer.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn broadcast_multi_threaded_all_receive() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "mt-broadcast",
            TopicOptions::Mixed {
                balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                broadcast_capacity: 2048,
            },
            InMemoryBackend,
        )
        .unwrap();

    let n = 500u64;

    let mut sub1 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut sub2 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    let t = topic.clone();
    let pub_handle = tokio::spawn(async move {
        for i in 0..n {
            t.publish(Arc::new(i)).await.unwrap();
        }
        t.close();
    });

    let h1 = tokio::spawn(async move {
        let mut received = Vec::new();
        loop {
            match sub1.recv().await {
                Ok(RecvItem::Message(env)) => received.push(*env.payload),
                Ok(RecvItem::Lagged { .. }) => {}
                Err(_) => break,
            }
        }
        received
    });

    let h2 = tokio::spawn(async move {
        let mut received = Vec::new();
        loop {
            match sub2.recv().await {
                Ok(RecvItem::Message(env)) => received.push(*env.payload),
                Ok(RecvItem::Lagged { .. }) => {}
                Err(_) => break,
            }
        }
        received
    });

    pub_handle.await.unwrap();
    let r1 = h1.await.unwrap();
    let r2 = h2.await.unwrap();

    // Both should receive all messages (buffer large enough).
    assert_eq!(r1.len(), n as usize);
    assert_eq!(r2.len(), n as usize);

    // In order.
    for w in r1.windows(2) {
        assert!(w[0] < w[1]);
    }
    for w in r2.windows(2) {
        assert!(w[0] < w[1]);
    }
}

// =========================================================================
// Edge: no subscribers
// =========================================================================

// Publishing to a topic with zero subscribers succeeds silently (messages are
// dropped rather than causing an error).
#[tokio::test]
async fn publish_with_no_subscribers_succeeds() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("empty", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    // Publishing to a topic with no subscribers should succeed (messages are dropped).
    topic.publish(Arc::new(42)).await.unwrap();
}

// =========================================================================
// Edge: subscriber receives only after subscribe
// =========================================================================

// Messages published before a subscriber joins are not delivered; only messages
// published after subscribe are received.
#[tokio::test]
async fn subscriber_only_receives_messages_after_subscribe() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("late-sub", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    // Publish before subscribing.
    topic.publish(Arc::new(1)).await.unwrap();
    topic.publish(Arc::new(2)).await.unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Publish after subscribing.
    topic.publish(Arc::new(3)).await.unwrap();
    topic.close();

    let mut received = Vec::new();
    while let Ok(RecvItem::Message(env)) = sub.recv().await {
        received.push(*env.payload);
    }
    // Should only have message 3 (published after subscribe).
    assert_eq!(received, vec![3]);
}

// =========================================================================
// BalancedOnly mode
// =========================================================================

// A BalancedOnly topic delivers 50 messages in order to a single subscriber.
#[tokio::test]
async fn balanced_only_basic_delivery() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bo-basic",
            TopicOptions::BalancedOnly {
                capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    let n = 50u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut received = Vec::new();
    while let Ok(RecvItem::Message(env)) = sub.recv().await {
        received.push(*env.payload);
    }
    assert_eq!(received.len(), n as usize);
    for (i, &val) in received.iter().enumerate() {
        assert_eq!(val, i as u64);
    }
}

// A BalancedOnly topic returns BroadcastNotSupported when a broadcast
// subscription is attempted.
#[tokio::test]
async fn balanced_only_rejects_broadcast() {
    let broker: TopicBroker<u64> = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bo-no-bcast",
            TopicOptions::BalancedOnly {
                capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    match topic.subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default()) {
        Err(Error::SubscribeBroadcastNotSupported) => {}
        Ok(_) => panic!("expected BroadcastNotSupported, got Ok"),
        Err(e) => panic!("expected BroadcastNotSupported, got Err({e:?})"),
    }
}

// A BalancedOnly topic allows multiple subscribers in the same group but
// returns SingleGroupViolation for a second group name.
#[tokio::test]
async fn balanced_only_rejects_second_group() {
    let broker: TopicBroker<u64> = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bo-single-group",
            TopicOptions::BalancedOnly {
                capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    // First group succeeds.
    let _sub1 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Same group succeeds (adds another subscriber).
    let _sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Different group fails.
    match topic.subscribe(
        SubscriptionMode::Balanced { group: "g2".into() },
        SubscriberOptions::default(),
    ) {
        Err(Error::SubscribeSingleGroupViolation) => {}
        Ok(_) => panic!("expected SingleGroupViolation, got Ok"),
        Err(e) => panic!("expected SingleGroupViolation, got Err({e:?})"),
    }
}

// A closed BalancedOnly topic rejects new balanced subscriptions.
#[tokio::test]
async fn balanced_only_rejects_balanced_subscribe_after_close() {
    let broker: TopicBroker<u64> = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bo-closed-subscribe",
            TopicOptions::BalancedOnly {
                capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();
    topic.close();

    match topic.subscribe(
        SubscriptionMode::Balanced { group: "g1".into() },
        SubscriberOptions::default(),
    ) {
        Err(Error::TopicClosed) => {}
        Ok(_) => panic!("expected TopicClosed, got Ok"),
        Err(e) => panic!("expected TopicClosed, got Err({e:?})"),
    }
}

// A closed Mixed topic rejects new balanced subscriptions.
#[tokio::test]
async fn mixed_rejects_balanced_subscribe_after_close() {
    let broker: TopicBroker<u64> = TopicBroker::new();
    let topic = broker
        .create_topic(
            "mixed-closed-subscribe",
            TopicOptions::Mixed {
                balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                broadcast_capacity: TopicOptions::DEFAULT_BROADCAST_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();
    topic.close();

    match topic.subscribe(
        SubscriptionMode::Balanced { group: "g1".into() },
        SubscriberOptions::default(),
    ) {
        Err(Error::TopicClosed) => {}
        Ok(_) => panic!("expected TopicClosed, got Ok"),
        Err(e) => panic!("expected TopicClosed, got Err({e:?})"),
    }
}

// Two subscribers in a BalancedOnly topic's single group collectively receive
// all 200 messages with no duplicates.
#[tokio::test]
async fn balanced_only_no_messages_lost() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bo-no-loss",
            TopicOptions::BalancedOnly {
                capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub1 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    let n = 200u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut all_ids = HashSet::new();
    while let Ok(RecvItem::Message(env)) = sub1.recv().await {
        assert!(all_ids.insert(env.id), "duplicate id={}", env.id);
    }
    while let Ok(RecvItem::Message(env)) = sub2.recv().await {
        assert!(all_ids.insert(env.id), "duplicate id={}", env.id);
    }
    assert_eq!(all_ids.len(), n as usize);
}

// =========================================================================
// BroadcastOnly mode
// =========================================================================

// A BroadcastOnly topic delivers all 50 messages in order to each of two
// subscribers.
#[tokio::test]
async fn broadcast_only_basic_delivery() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bro-basic",
            TopicOptions::BroadcastOnly { capacity: 1024 },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub1 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut sub2 = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    let n = 50u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    for sub in [&mut sub1, &mut sub2] {
        let mut received = Vec::new();
        while let Ok(RecvItem::Message(env)) = sub.recv().await {
            received.push(*env.payload);
        }
        assert_eq!(received.len(), n as usize);
        for (i, &val) in received.iter().enumerate() {
            assert_eq!(val, i as u64);
        }
    }
}

// A BroadcastOnly topic returns BalancedNotSupported when a balanced
// subscription is attempted.
#[tokio::test]
async fn broadcast_only_rejects_balanced() {
    let broker: TopicBroker<u64> = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bro-no-balanced",
            TopicOptions::BroadcastOnly {
                capacity: TopicOptions::DEFAULT_BROADCAST_CAPACITY,
            },
            InMemoryBackend,
        )
        .unwrap();

    match topic.subscribe(
        SubscriptionMode::Balanced { group: "g1".into() },
        SubscriberOptions::default(),
    ) {
        Err(Error::SubscribeBalancedNotSupported) => {}
        Ok(_) => panic!("expected BalancedNotSupported, got Ok"),
        Err(e) => panic!("expected BalancedNotSupported, got Err({e:?})"),
    }
}

// A BroadcastOnly topic with a small ring buffer (8 slots) reports lag when
// the publisher overwrites unread slots.
#[tokio::test]
async fn broadcast_only_lag_reported() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bro-lag",
            TopicOptions::BroadcastOnly { capacity: 8 },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    let n = 50u64;
    for i in 0..n {
        topic.publish(Arc::new(i)).await.unwrap();
    }
    topic.close();

    let mut messages = Vec::new();
    let mut total_lagged = 0u64;
    loop {
        match sub.recv().await {
            Ok(RecvItem::Message(env)) => messages.push(*env.payload),
            Ok(RecvItem::Lagged { missed }) => total_lagged += missed,
            Err(_) => break,
        }
    }

    assert!(
        messages.len() < n as usize,
        "expected lag but got all {} messages",
        n
    );
    assert!(total_lagged > 0, "expected lag notification");

    for w in messages.windows(2) {
        assert!(w[0] < w[1], "out of order: {} >= {}", w[0], w[1]);
    }
}

// =========================================================================
// Per-publisher ack routing
// =========================================================================

// Two publisher handles with separate ack channels publish one message each,
// acks are routed to the correct per-publisher channel.
#[tokio::test]
async fn per_publisher_ack_routing() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic("per-pub-ack", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let (ack_tx_a, mut ack_rx_a) = mpsc::channel::<AckEvent>(64);
    let (ack_tx_b, mut ack_rx_b) = mpsc::channel::<AckEvent>(64);

    let handle_a = base.with_ack_sender(ack_tx_a);
    let handle_b = base.with_ack_sender(ack_tx_b);

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Publisher A sends a message.
    handle_a.publish(Arc::new(100u64)).await.unwrap();
    // Publisher B sends a message.
    handle_b.publish(Arc::new(200u64)).await.unwrap();

    let env1 = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };
    let env2 = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    // Ack both.
    sub.ack(env1.id).unwrap();
    sub.ack(env2.id).unwrap();

    // Ack for publisher A's message goes to ack_rx_a.
    let evt_a = ack_rx_a.recv().await.unwrap();
    assert_eq!(evt_a.status, AckStatus::Ack);
    assert_eq!(*env1.payload, 100);

    // Ack for publisher B's message goes to ack_rx_b.
    let evt_b = ack_rx_b.recv().await.unwrap();
    assert_eq!(evt_b.status, AckStatus::Ack);
    assert_eq!(*env2.payload, 200);

    // Neither channel has extra events.
    assert!(ack_rx_a.try_recv().is_err());
    assert!(ack_rx_b.try_recv().is_err());
}

// Per-publisher ack routing works with BroadcastOnly topics: a message
// published via with_ack_sender is acked back to the correct channel.
#[tokio::test]
async fn per_publisher_ack_broadcast_mode() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic(
            "per-pub-bcast",
            TopicOptions::BroadcastOnly { capacity: 1024 },
            InMemoryBackend,
        )
        .unwrap();

    let (ack_tx, mut ack_rx) = mpsc::channel::<AckEvent>(64);
    let handle = base.with_ack_sender(ack_tx);

    let mut sub = base
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    handle.publish(Arc::new(42u64)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    sub.ack(env.id).unwrap();

    let evt = ack_rx.recv().await.unwrap();
    assert_eq!(evt.status, AckStatus::Ack);
    assert_eq!(evt.message_id, env.id);
    assert_ne!(evt.publisher_id, 0); // per-publisher, not default
}

// =========================================================================
// Broker lifecycle methods
// =========================================================================

// get_topic returns a handle to a previously created topic by name.
#[tokio::test]
async fn get_topic_returns_existing() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("my-topic", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    let retrieved = broker.get_topic("my-topic");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name().as_ref(), handle.name().as_ref());
}

// get_topic returns None for a name that was never created.
#[tokio::test]
async fn get_topic_returns_none_for_missing() {
    let broker = TopicBroker::<u64>::new();
    assert!(broker.get_topic("nonexistent").is_none());
}

// has_topic returns false before creation and true after.
#[tokio::test]
async fn has_topic_reflects_state() {
    let broker = TopicBroker::<u64>::new();
    assert!(!broker.has_topic("t1"));
    _ = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    assert!(broker.has_topic("t1"));
}

// remove_topic closes the topic (publish returns Closed, recv returns Closed)
// and removes it from the broker.
#[tokio::test]
async fn remove_topic_closes_and_removes() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("removable", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let mut sub = handle
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    handle.publish(Arc::new(1)).await.unwrap();

    assert!(broker.remove_topic("removable"));
    assert!(!broker.has_topic("removable"));

    // Publish should fail with Closed.
    match handle.publish(Arc::new(2)).await {
        Err(Error::TopicClosed) => {}
        other => panic!("expected PublishError::Closed, got {:?}", other),
    }

    // Drain any buffered message, then expect Closed.
    loop {
        match sub.recv().await {
            Ok(RecvItem::Message(_)) => continue,
            Err(Error::SubscriptionClosed) => break,
            other => panic!("expected RecvError::Closed, got {:?}", other),
        }
    }
}

// remove_topic returns false when the topic does not exist.
#[tokio::test]
async fn remove_topic_returns_false_for_missing() {
    let broker = TopicBroker::<u64>::new();
    assert!(!broker.remove_topic("nonexistent"));
}

// After removing a topic, recreating it with the same name yields an
// independent topic that does not share state with the old one.
#[tokio::test]
async fn remove_topic_allows_recreation_with_same_name() {
    let broker = TopicBroker::<u64>::new();
    let old_handle = broker
        .create_topic("reuse", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    old_handle.publish(Arc::new(1)).await.unwrap();
    _ = broker.remove_topic("reuse");

    // Recreate — should be an independent topic.
    let new_handle = broker
        .create_topic("reuse", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    let mut sub = new_handle
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    new_handle.publish(Arc::new(42)).await.unwrap();
    new_handle.close();

    let mut received = Vec::new();
    while let Ok(RecvItem::Message(env)) = sub.recv().await {
        received.push(*env.payload);
    }
    assert_eq!(received, vec![42]);
}

// topic_names returns the names of all currently registered topics.
#[tokio::test]
async fn topic_names_returns_all_names() {
    let broker = TopicBroker::<u64>::new();
    _ = broker
        .create_topic("alpha", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    _ = broker
        .create_topic("beta", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    _ = broker
        .create_topic("gamma", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let mut names: Vec<String> = broker
        .topic_names()
        .into_iter()
        .map(|n| n.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

// close_all closes every topic in the broker; subsequent publishes return
// Closed and the broker is empty.
#[tokio::test]
async fn close_all_closes_every_topic() {
    let broker = TopicBroker::<u64>::new();
    let h1 = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    let h2 = broker
        .create_topic("t2", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    broker.close_all();

    match h1.publish(Arc::new(1)).await {
        Err(Error::TopicClosed) => {}
        other => panic!("expected Closed, got {:?}", other),
    }
    match h2.publish(Arc::new(2)).await {
        Err(Error::TopicClosed) => {}
        other => panic!("expected Closed, got {:?}", other),
    }

    // Broker should be empty.
    assert!(broker.topic_names().is_empty());
}

// =========================================================================
// TopicSet
// =========================================================================

// Inserting a handle into a TopicSet and retrieving it by local name returns
// the correct underlying topic.
#[tokio::test]
async fn topic_set_insert_and_get() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("pipeline-1");
    _ = set.insert("output", handle);

    let retrieved = set.get("output");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name().as_ref(), "t1");
}

// Getting a name that was never inserted into the TopicSet returns None.
#[tokio::test]
async fn topic_set_get_missing_returns_none() {
    let set = TopicSet::<u64>::new("empty-set");
    assert!(set.get("nonexistent").is_none());
}

// Removing an entry from a TopicSet returns the handle and makes subsequent
// get calls return None.
#[tokio::test]
async fn topic_set_remove() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    _ = set.insert("output", handle);
    assert!(set.contains("output"));

    let removed = set.remove("output");
    assert!(removed.is_some());
    assert!(!set.contains("output"));
}

// Removing a topic from a TopicSet does not close the underlying topic; the
// original handle can still publish.
#[tokio::test]
async fn topic_set_remove_does_not_close_topic() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    _ = set.insert("output", handle.clone());

    // Remove from set.
    _ = set.remove("output");

    // Topic should still be usable via the original handle.
    handle.publish(Arc::new(42)).await.unwrap();
}

// Inserting a second handle under the same local name replaces the first and
// returns the previous handle.
#[tokio::test]
async fn topic_set_insert_overwrites() {
    let broker = TopicBroker::<u64>::new();
    let h1 = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    let h2 = broker
        .create_topic("t2", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    assert!(set.insert("output", h1).is_none());

    let prev = set.insert("output", h2);
    assert!(prev.is_some());
    assert_eq!(prev.unwrap().name().as_ref(), "t1");

    // Current should point to t2.
    assert_eq!(set.get("output").unwrap().name().as_ref(), "t2");
}

// topic_names on a TopicSet returns the local alias names, not the underlying
// topic names.
#[tokio::test]
async fn topic_set_topic_names() {
    let broker = TopicBroker::<u64>::new();
    let h1 = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();
    let h2 = broker
        .create_topic("t2", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    _ = set.insert("alpha", h1);
    _ = set.insert("beta", h2);

    let mut names: Vec<String> = set
        .topic_names()
        .into_iter()
        .map(|n| n.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

// A TopicSet created with an ack sender automatically wraps inserted handles
// so that ack events are routed to the set-level channel.
#[tokio::test]
async fn topic_set_with_ack_sender() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let (ack_tx, mut ack_rx) = mpsc::channel::<AckEvent>(64);
    let set = TopicSet::with_ack_sender("p1", ack_tx);
    _ = set.insert("output", handle);

    let pub_handle = set.get("output").unwrap();

    let mut sub = pub_handle
        .subscribe(
            SubscriptionMode::Balanced { group: "g1".into() },
            SubscriberOptions::default(),
        )
        .unwrap();

    pub_handle.publish(Arc::new(99)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    sub.ack(env.id).unwrap();

    let evt = ack_rx.recv().await.unwrap();
    assert_eq!(evt.status, AckStatus::Ack);
    assert_eq!(evt.message_id, env.id);
    assert_ne!(evt.publisher_id, 0); // per-publisher ack routing via set
}

// A TopicSet's name matches the value provided at construction.
#[tokio::test]
async fn topic_set_name() {
    let set = TopicSet::<u64>::new("my-pipeline");
    assert_eq!(set.name(), "my-pipeline");
}

// Cloning a TopicSet shares the underlying state: an insert on one clone is
// visible from the other.
#[tokio::test]
async fn topic_set_clone_shares_state() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set1 = TopicSet::new("p1");
    let set2 = set1.clone();

    _ = set1.insert("output", handle);

    // set2 should see the insert from set1.
    assert!(set2.contains("output"));
    assert_eq!(set2.len(), 1);
}

// len() and is_empty() accurately reflect the number of entries in the
// TopicSet.
#[tokio::test]
async fn topic_set_len_and_is_empty() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    assert!(set.is_empty());
    assert_eq!(set.len(), 0);

    _ = set.insert("output", handle);
    assert!(!set.is_empty());
    assert_eq!(set.len(), 1);
}
