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
//! - **Ack/nack**: tracked publish outcomes, per-publisher independence,
//!   bounded in-flight tracking, disabled-when-not-tracked.
//! - **Lifecycle**: `remove_topic` closes the topic (publish fails, recv gets
//!   Closed), removed topics can be recreated independently, `close_all` shuts
//!   everything down.
//! - **TopicSet**: insert/get/remove semantics, overwrite returns previous,
//!   remove-does-not-close, clone-shares-state.

use crate::error::Error;
use crate::topic::backend::{InMemoryBackend, SubscriptionBackend};
use crate::topic::subscription::{DeliveryBackend, DeliveryStorageKind};
use crate::topic::types::{
    Envelope, PublishOutcome, RecvItem, SubscriberOptions, SubscriptionMode, TopicOptions,
    TopicPublishOutcomeConfig, TrackedPublishOutcome, TrackedPublishPermit, TrackedPublishTracker,
    TrackedTryPublishOutcome,
};
use crate::topic::{Delivery, RecvDelivery, Subscription, TopicBroker, TopicSet};
use otap_df_config::topic::TopicBroadcastOnLagPolicy;
use otap_df_config::{SubscriptionGroupName, TopicName};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::Semaphore;

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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
                    SubscriptionMode::Balanced {
                        group: SubscriptionGroupName::from("g1"),
                    },
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
                group: SubscriptionGroupName::from("group-A"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub_g1_b = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("group-A"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub_g2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("group-B"),
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
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

// A lagging broadcast subscriber configured with disconnect receives one final
// lag notification and is then closed on the next recv call.
#[tokio::test]
async fn broadcast_disconnects_slow_subscriber_on_lag() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "broadcast-disconnect",
            TopicOptions::BroadcastOnly {
                capacity: 8,
                on_lag: TopicBroadcastOnLagPolicy::Disconnect,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    for i in 0..50u64 {
        topic.publish(Arc::new(i)).await.unwrap();
    }

    match sub.recv().await {
        Ok(RecvItem::Lagged { missed }) => assert!(missed > 0, "expected missed count > 0"),
        other => panic!("expected lag notification before disconnect, got {other:?}"),
    }
    assert!(matches!(sub.recv().await, Err(Error::SubscriptionClosed)));
}

// In Mixed mode, async publish now waits for full topic admission. Balanced
// backpressure therefore delays both publish completion and broadcast
// visibility, while `try_publish` remains the non-blocking API.
#[tokio::test]
async fn mixed_async_publish_waits_for_balanced_admission_before_broadcast() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "mixed-bcast-priority",
            TopicOptions::Mixed {
                balanced_capacity: 1,
                broadcast_capacity: 16,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut balanced = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    // Fill the balanced queue.
    topic.publish(Arc::new(1u64)).await.unwrap();

    // Second publish should block on balanced delivery, so broadcast should
    // not observe it until balanced admission succeeds.
    let topic_clone = topic.clone();
    let second_publish = tokio::spawn(async move {
        topic_clone.publish(Arc::new(2u64)).await.unwrap();
    });

    // First message.
    match broadcast.recv().await {
        Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected: {:?}", other),
    }

    // Second message should not arrive before balanced queue is drained.
    assert!(
        tokio::time::timeout(Duration::from_millis(200), async {
            loop {
                match broadcast.recv().await {
                    Ok(RecvItem::Message(env)) => break *env.payload,
                    Ok(RecvItem::Lagged { .. }) => continue,
                    Err(e) => panic!("unexpected recv error: {e:?}"),
                }
            }
        })
        .await
        .is_err(),
        "broadcast delivery should wait for balanced admission on async publish"
    );

    // Publisher task should still be blocked on balanced queue.
    assert!(
        !second_publish.is_finished(),
        "second publish should still be blocked by balanced backpressure"
    );

    // Drain balanced queue to unblock publisher and then observe the second
    // broadcast message.
    _ = balanced.recv().await.unwrap();
    second_publish.await.unwrap();

    let second = tokio::time::timeout(Duration::from_millis(200), async {
        loop {
            match broadcast.recv().await {
                Ok(RecvItem::Message(env)) => break *env.payload,
                Ok(RecvItem::Lagged { .. }) => continue,
                Err(e) => panic!("unexpected recv error: {e:?}"),
            }
        }
    })
    .await
    .expect("broadcast delivery should resume after balanced admission succeeds");
    assert_eq!(second, 2);

    _ = balanced.recv().await.unwrap();
    topic.close();
}

// A blocked mixed publish must not reserve capacity on balanced groups that
// were otherwise ready to admit the message.
#[tokio::test]
async fn mixed_async_publish_does_not_reserve_free_groups_while_waiting() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "mixed-no-convoy",
            TopicOptions::Mixed {
                balanced_capacity: 1,
                broadcast_capacity: 16,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut fast = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("fast"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut slow = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("slow"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    topic.publish(Arc::new(1u64)).await.unwrap();

    match fast.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected first fast item: {other:?}"),
    }

    let topic_clone = topic.clone();
    let blocked_publish = tokio::spawn(async move {
        topic_clone.publish(Arc::new(2u64)).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        !blocked_publish.is_finished(),
        "publish should still be blocked by the slow group"
    );

    let mut permits = topic.debug_balanced_available_permits();
    permits.sort_by(|(left, _), (right, _)| left.as_ref().cmp(right.as_ref()));
    assert_eq!(
        permits,
        vec![
            (SubscriptionGroupName::from("fast"), 1),
            (SubscriptionGroupName::from("slow"), 0),
        ],
        "blocked mixed publish should not hold the fast group's permit"
    );

    match slow.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected first slow item: {other:?}"),
    }

    blocked_publish.await.unwrap();

    match fast.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 2),
        other => panic!("unexpected second fast item: {other:?}"),
    }
    match slow.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 2),
        other => panic!("unexpected second slow item: {other:?}"),
    }

    topic.close();
}

// Dropping a blocked async mixed-topic publish must not partially publish to
// broadcast subscribers before balanced admission succeeds.
#[tokio::test]
async fn mixed_async_publish_drop_does_not_publish_to_broadcast() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "mixed-cancel-no-broadcast",
            TopicOptions::Mixed {
                balanced_capacity: 1,
                broadcast_capacity: 16,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut balanced = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    topic.publish(Arc::new(1u64)).await.unwrap();

    let mut publish = Box::pin(topic.publish(Arc::new(2u64)));
    assert!(
        tokio::time::timeout(Duration::from_millis(200), publish.as_mut())
            .await
            .is_err(),
        "second publish should block while balanced capacity is full"
    );

    match broadcast.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected first broadcast item: {other:?}"),
    }
    drop(publish);

    assert!(
        tokio::time::timeout(Duration::from_millis(200), async {
            loop {
                match broadcast.recv().await {
                    Ok(RecvItem::Message(env)) => break *env.payload,
                    Ok(RecvItem::Lagged { .. }) => continue,
                    Err(e) => panic!("unexpected recv error: {e:?}"),
                }
            }
        })
        .await
        .is_err(),
        "dropping a blocked publish must not leak a broadcast delivery"
    );

    match balanced.recv().await.unwrap() {
        RecvItem::Message(env) => assert_eq!(*env.payload, 1),
        other => panic!("unexpected balanced item: {other:?}"),
    }
}

// Dropping a blocked tracked publish must release the publisher's in-flight
// slot so later non-blocking attempts fail only on topic capacity, not leaked
// tracked publisher capacity.
#[tokio::test]
async fn blocked_tracked_publish_drop_releases_in_flight_capacity() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "tracked-publish-drop",
            TopicOptions::BalancedOnly { capacity: 1 },
            InMemoryBackend,
        )
        .unwrap();

    let _balanced = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    topic.publish(Arc::new(1u64)).await.unwrap();

    let tracked = topic.tracked_publisher_with_config(TopicPublishOutcomeConfig {
        max_in_flight: 1,
        timeout: Duration::from_secs(30),
    });
    let mut blocked = Box::pin(tracked.publish(Arc::new(2u64)));
    assert!(
        tokio::time::timeout(Duration::from_millis(200), blocked.as_mut())
            .await
            .is_err(),
        "tracked publish should block while balanced capacity is full"
    );
    drop(blocked);

    assert!(
        matches!(
            tracked.try_publish(Arc::new(3u64)).unwrap(),
            TrackedTryPublishOutcome::DroppedOnFull
        ),
        "dropping the blocked tracked publish should release the in-flight slot"
    );
}

// Broadcast delivery permits keep ownership of one message until the caller
// commits it, preventing the subscription from advancing past the permitted item.
#[tokio::test]
async fn broadcast_delivery_commit_advances_to_next_message() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "broadcast-delivery-permit",
            TopicOptions::BroadcastOnly {
                capacity: 16,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    topic.publish(Arc::new(1u64)).await.unwrap();
    topic.publish(Arc::new(2u64)).await.unwrap();

    let first = match sub.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };
    assert_eq!(first.message_id(), 1);
    assert_eq!(*first.envelope().payload, 1);

    assert!(
        tokio::time::timeout(Duration::from_millis(200), sub.recv_delivery())
            .await
            .is_err(),
        "second delivery should stay blocked while the first delivery permit is held"
    );

    first.commit();

    let second = match sub.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };
    assert_eq!(second.message_id(), 2);
    assert_eq!(*second.envelope().payload, 2);
    second.commit();
}

// Balanced deliveries from the in-memory backend use the specialized inline
// finalizer path instead of the opaque fallback.
#[tokio::test]
async fn balanced_delivery_uses_specialized_inline_storage() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "balanced-inline-delivery",
            TopicOptions::default(),
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    topic.publish(Arc::new(7u64)).await.unwrap();

    let delivery = match sub.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };
    assert_eq!(delivery.storage_kind(), DeliveryStorageKind::Balanced);
    delivery.commit();
}

// Broadcast deliveries from the in-memory backend use the specialized inline
// finalizer path instead of the opaque fallback.
#[tokio::test]
async fn broadcast_delivery_uses_specialized_inline_storage() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "broadcast-inline-delivery",
            TopicOptions::BroadcastOnly {
                capacity: 16,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    topic.publish(Arc::new(11u64)).await.unwrap();

    let delivery = match sub.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };
    assert_eq!(delivery.storage_kind(), DeliveryStorageKind::Broadcast);
    delivery.commit();
}

// Custom backends still work through the opaque delivery fallback even though
// the in-memory backend now uses specialized inline storage.
#[tokio::test]
async fn opaque_delivery_fallback_still_works() {
    struct OpaqueDelivery {
        envelope: Envelope<u64>,
        aborted: Arc<AtomicBool>,
    }

    impl DeliveryBackend<u64> for OpaqueDelivery {
        fn envelope(&self) -> &Envelope<u64> {
            &self.envelope
        }

        fn commit(&mut self) {}

        fn abort(&mut self, _reason: Arc<str>) -> Result<(), Error> {
            self.aborted.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn abandon(&mut self) {}
    }

    struct OpaqueSubscription {
        yielded: bool,
        aborted: Arc<AtomicBool>,
    }

    impl SubscriptionBackend<u64> for OpaqueSubscription {
        fn poll_recv_delivery(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<RecvDelivery<u64>, Error>> {
            if self.yielded {
                Poll::Ready(Err(Error::SubscriptionClosed))
            } else {
                self.yielded = true;
                Poll::Ready(Ok(RecvDelivery::Message(Delivery::new_opaque(Box::new(
                    OpaqueDelivery {
                        envelope: Envelope {
                            id: 41,
                            payload: Arc::new(99u64),
                            tracked: false,
                        },
                        aborted: Arc::clone(&self.aborted),
                    },
                )))))
            }
        }

        fn ack(&self, _id: u64) -> Result<(), Error> {
            Ok(())
        }

        fn nack(&self, _id: u64, _reason: Arc<str>) -> Result<(), Error> {
            Ok(())
        }
    }

    let aborted = Arc::new(AtomicBool::new(false));
    let mut subscription = Subscription::new(Box::new(OpaqueSubscription {
        yielded: false,
        aborted: Arc::clone(&aborted),
    }));

    let delivery = match subscription.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };
    assert_eq!(delivery.storage_kind(), DeliveryStorageKind::Opaque);
    delivery.abort("opaque fallback abort").unwrap();
    assert!(aborted.load(Ordering::SeqCst));
}

// Aborting an unforwarded tracked delivery must resolve its tracked publish as
// a Nack owned by the delivery object itself.
#[tokio::test]
async fn tracked_delivery_abort_resolves_publish_outcome() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "tracked-delivery-abort",
            TopicOptions::default(),
            InMemoryBackend,
        )
        .unwrap();

    let tracked = topic.tracked_publisher();
    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let receipt = tracked.publish(Arc::new(42u64)).await.unwrap();
    let delivery = match sub.recv_delivery().await.unwrap() {
        RecvDelivery::Message(delivery) => delivery,
        RecvDelivery::Lagged { .. } => panic!("unexpected lag notification"),
    };

    delivery
        .abort("downstream rejected before forward")
        .unwrap();
    assert_eq!(
        receipt.wait_for_outcome().await,
        TrackedPublishOutcome::Nack {
            reason: Arc::from("downstream rejected before forward"),
        }
    );
}

// In Mixed mode, a lagging broadcast subscriber can be disconnected without
// affecting fast broadcast subscribers or balanced delivery.
#[tokio::test]
async fn mixed_disconnects_only_lagging_broadcast_subscriber() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic(
            "mixed-broadcast-disconnect",
            TopicOptions::Mixed {
                balanced_capacity: 32,
                broadcast_capacity: 4,
                on_lag: TopicBroadcastOnLagPolicy::Disconnect,
            },
        )
        .unwrap();

    let mut balanced = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("workers"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut slow_broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();
    let mut fast_broadcast = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    for i in 0..20u64 {
        topic.publish(Arc::new(i)).await.unwrap();
        match fast_broadcast.recv().await {
            Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, i),
            other => panic!("fast broadcast subscriber should stay current: {other:?}"),
        }
    }

    match slow_broadcast.recv().await {
        Ok(RecvItem::Lagged { missed }) => assert!(missed > 0, "expected lagged slow subscriber"),
        other => panic!("expected lag notification for slow broadcast subscriber, got {other:?}"),
    }
    assert!(matches!(
        slow_broadcast.recv().await,
        Err(Error::SubscriptionClosed)
    ));

    match balanced.recv().await {
        Ok(RecvItem::Message(env)) => assert_eq!(*env.payload, 0),
        other => panic!("balanced subscriber should still receive messages: {other:?}"),
    }
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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

// Tracked publishes resolve to the ack/nack reported by downstream subscribers.
#[tokio::test]
async fn tracked_publish_outcomes_received_when_enabled() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic("ack-test", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let topic = base.tracked_publisher();

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let receipt1 = topic.publish(Arc::new(42u64)).await.unwrap();
    let receipt2 = topic.publish(Arc::new(43u64)).await.unwrap();

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

    assert_eq!(
        receipt1.wait_for_outcome().await,
        TrackedPublishOutcome::Ack
    );
    assert_eq!(
        receipt2.wait_for_outcome().await,
        TrackedPublishOutcome::Nack {
            reason: Arc::from("bad data"),
        }
    );
}

// Publishing without the tracked publisher means ack/nack returns MessageNotTracked.
#[tokio::test]
async fn ack_nack_fail_when_message_is_not_tracked() {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("no-ack", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    topic.publish(Arc::new(1)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    match sub.ack(env.id) {
        Err(Error::MessageNotTracked) => {}
        other => panic!("expected MessageNotTracked, got {:?}", other),
    }
}

// Tracked publishers expose a bounded in-flight limit on try_publish.
#[tokio::test]
async fn tracked_try_publish_respects_max_in_flight() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic("ack-full", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let topic = base.tracked_publisher_with_config(TopicPublishOutcomeConfig {
        max_in_flight: 1,
        timeout: Duration::from_secs(30),
    });

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let receipt = topic.publish(Arc::new(1u64)).await.unwrap();
    assert!(matches!(
        topic.try_publish(Arc::new(2u64)).unwrap(),
        TrackedTryPublishOutcome::MaxInFlightReached
    ));

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };
    sub.ack(env.id).unwrap();

    assert_eq!(receipt.wait_for_outcome().await, TrackedPublishOutcome::Ack);
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let mut sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!handle.is_finished(), "publish should be blocked");

    // Consume one message to unblock.
    let _ = sub.recv().await.unwrap();

    // Now the publisher should complete.
    tokio::time::sleep(Duration::from_millis(50)).await;
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Same group succeeds (adds another subscriber).
    let _sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    // Different group fails.
    match topic.subscribe(
        SubscriptionMode::Balanced {
            group: SubscriptionGroupName::from("g2"),
        },
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
        SubscriptionMode::Balanced {
            group: SubscriptionGroupName::from("g1"),
        },
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();
    topic.close();

    match topic.subscribe(
        SubscriptionMode::Balanced {
            group: SubscriptionGroupName::from("g1"),
        },
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut sub2 = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
            TopicOptions::BroadcastOnly {
                capacity: 1024,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
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
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    match topic.subscribe(
        SubscriptionMode::Balanced {
            group: SubscriptionGroupName::from("g1"),
        },
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
            TopicOptions::BroadcastOnly {
                capacity: 8,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
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

// Two tracked publishers can publish independently and receive separate outcomes.
#[tokio::test]
async fn tracked_publishers_resolve_independently() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic("per-pub-ack", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let handle_a = base.tracked_publisher();
    let handle_b = base.tracked_publisher();

    let mut sub = base
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let receipt_a = handle_a.publish(Arc::new(100u64)).await.unwrap();
    let receipt_b = handle_b.publish(Arc::new(200u64)).await.unwrap();

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

    assert_eq!(
        receipt_a.wait_for_outcome().await,
        TrackedPublishOutcome::Ack
    );
    assert_eq!(
        receipt_b.wait_for_outcome().await,
        TrackedPublishOutcome::Ack
    );
}

// Broadcast subscribers can acknowledge tracked publishes too.
#[tokio::test]
async fn tracked_publish_broadcast_mode() {
    let broker = TopicBroker::new();
    let base = broker
        .create_topic(
            "per-pub-bcast",
            TopicOptions::BroadcastOnly {
                capacity: 1024,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let handle = base.tracked_publisher();

    let mut sub = base
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    let receipt = handle.publish(Arc::new(42u64)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    sub.ack(env.id).unwrap();

    assert_eq!(receipt.wait_for_outcome().await, TrackedPublishOutcome::Ack);
}

// =========================================================================
// Non-blocking publish
// =========================================================================

// try_publish reports DroppedOnFull for balanced queues without awaiting.
#[tokio::test]
async fn try_publish_balanced_only_reports_drop_on_full() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "nb-balanced",
            TopicOptions::BalancedOnly { capacity: 1 },
            InMemoryBackend,
        )
        .unwrap();

    let _sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    assert_eq!(
        topic.try_publish(Arc::new(1)).unwrap(),
        PublishOutcome::Published
    );
    assert_eq!(
        topic.try_publish(Arc::new(2)).unwrap(),
        PublishOutcome::DroppedOnFull
    );
}

// On mixed topics, try_publish is all-or-nothing across balanced and
// broadcast delivery. A balanced drop-on-full result must not leak a message
// to broadcast subscribers.
#[tokio::test]
async fn try_publish_mixed_rejects_broadcast_when_balanced_is_full() {
    let broker = TopicBroker::<u64>::new();
    let topic = broker
        .create_topic(
            "nb-mixed",
            TopicOptions::Mixed {
                balanced_capacity: 1,
                broadcast_capacity: 8,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
            },
            InMemoryBackend,
        )
        .unwrap();

    let _balanced_sub = topic
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();
    let mut broadcast_sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .unwrap();

    assert_eq!(
        topic.try_publish(Arc::new(10)).unwrap(),
        PublishOutcome::Published
    );
    assert_eq!(
        topic.try_publish(Arc::new(20)).unwrap(),
        PublishOutcome::DroppedOnFull
    );

    topic.close();

    let mut received = Vec::new();
    while let Ok(item) = broadcast_sub.recv().await {
        if let RecvItem::Message(env) = item {
            received.push(*env.payload);
        }
    }
    assert_eq!(received, vec![10]);
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

// get_topic_required returns UnknownTopic when missing.
#[tokio::test]
async fn get_topic_required_returns_error_for_missing() {
    let broker = TopicBroker::<u64>::new();
    let missing = "missing";
    match broker.get_topic_required(missing) {
        Err(Error::UnknownTopic { topic }) => assert_eq!(topic, missing),
        _ => panic!("expected UnknownTopic"),
    }
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

// create_topics declares multiple topics in one call.
#[tokio::test]
async fn create_topics_batch_success() {
    let broker = TopicBroker::<u64>::new();
    let declarations = vec![
        (TopicName::parse("t-a").unwrap(), TopicOptions::default()),
        (TopicName::parse("t-b").unwrap(), TopicOptions::default()),
    ];
    let handles = broker.create_topics(declarations, InMemoryBackend).unwrap();
    assert_eq!(handles.len(), 2);
    assert!(broker.has_topic("t-a"));
    assert!(broker.has_topic("t-b"));
}

// create_topics is atomic for duplicate checks: duplicate names in the batch
// fail and no topic from the call is inserted.
#[tokio::test]
async fn create_topics_batch_duplicate_is_rejected_atomically() {
    let broker = TopicBroker::<u64>::new();
    let dup = TopicName::parse("dup").unwrap();
    let result = broker.create_topics(
        vec![
            (dup.clone(), TopicOptions::default()),
            (dup.clone(), TopicOptions::default()),
        ],
        InMemoryBackend,
    );
    match result {
        Err(Error::TopicAlreadyExists { topic }) => assert_eq!(topic, dup),
        _ => panic!("expected TopicAlreadyExists"),
    }
    assert!(!broker.has_topic("dup"));
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
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

// Registering with a closed tracker resolves immediately as TopicClosed
// instead of leaving a pending entry behind without a timeout worker.
#[tokio::test]
async fn tracked_publish_tracker_register_after_close_resolves_immediately() {
    let tracker = TrackedPublishTracker::new();
    tracker.close_all();

    let permit = TrackedPublishPermit::from_tokio_owned(
        Arc::new(Semaphore::new(1))
            .acquire_owned()
            .await
            .expect("semaphore should not be closed"),
    );

    let receipt = tracker.register(1, Duration::from_secs(30), permit);

    assert_eq!(receipt.message_id(), 1);
    assert_eq!(
        receipt.wait_for_outcome().await,
        TrackedPublishOutcome::TopicClosed
    );
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

// get_required on TopicSet returns UnknownTopic when local alias is missing.
#[tokio::test]
async fn topic_set_get_required_returns_error_for_missing() {
    let set = TopicSet::<u64>::new("empty-set");
    let missing = "missing-local";
    match set.get_required(missing) {
        Err(Error::UnknownTopic { topic }) => assert_eq!(topic, missing),
        _ => panic!("expected UnknownTopic"),
    }
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

// Handles fetched from a TopicSet can still create tracked publishers.
#[tokio::test]
async fn topic_set_tracked_publisher() {
    let broker = TopicBroker::<u64>::new();
    let handle = broker
        .create_topic("t1", TopicOptions::default(), InMemoryBackend)
        .unwrap();

    let set = TopicSet::new("p1");
    _ = set.insert("output", handle);

    let handle = set.get("output").unwrap();
    let pub_handle = handle.tracked_publisher();

    let mut sub = handle
        .subscribe(
            SubscriptionMode::Balanced {
                group: SubscriptionGroupName::from("g1"),
            },
            SubscriberOptions::default(),
        )
        .unwrap();

    let receipt = pub_handle.publish(Arc::new(99u64)).await.unwrap();

    let env = match sub.recv().await.unwrap() {
        RecvItem::Message(e) => e,
        _ => panic!(),
    };

    sub.ack(env.id).unwrap();

    assert_eq!(receipt.wait_for_outcome().await, TrackedPublishOutcome::Ack);
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
