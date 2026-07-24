// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer utilities: single consumers, consumer groups, rebalancing, and
//! committed-offset inspection.
//!
//! Multiple [`TestConsumer`]s sharing a `group.id` model consumer-group
//! rebalancing; [`TestConsumer::assignment`] / [`TestConsumer::wait_for_assignment`]
//! let tests assert partition distribution. [`RebalanceTrigger`] forces a
//! deterministic revoke by joining an extra member and polling it until
//! assigned.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer, StreamConsumer};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};

use super::error::TestError;
use super::message::ConsumedMessage;
use super::wait::poll_until_async;

/// Default timeout for a single [`TestConsumer::recv`].
const DEFAULT_RECV_TIMEOUT: Duration = Duration::from_secs(30);
/// Default consumer `session.timeout.ms`.
const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(6);
/// Timeout used when probing committed offsets.
const COMMITTED_PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Monotonic counter so each builder gets a unique default `group.id`,
/// preventing accidental cross-test group sharing.
static GROUP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Builder for a [`TestConsumer`] bound to a cluster's bootstrap servers.
#[must_use]
pub(crate) struct TestConsumerBuilder {
    brokers: String,
    group_id: Option<String>,
    auto_offset_reset: String,
    enable_auto_commit: bool,
    session_timeout: Duration,
    overrides: Vec<(String, String)>,
}

impl TestConsumerBuilder {
    /// Creates a builder targeting `brokers`.
    pub(crate) fn new(brokers: &str) -> Self {
        Self {
            brokers: brokers.to_string(),
            group_id: None,
            auto_offset_reset: "earliest".to_string(),
            enable_auto_commit: false,
            session_timeout: DEFAULT_SESSION_TIMEOUT,
            overrides: Vec::new(),
        }
    }

    /// Sets the consumer group id (default: a unique per-builder id).
    pub(crate) fn group_id(mut self, id: &str) -> Self {
        self.group_id = Some(id.to_string());
        self
    }

    /// Sets `auto.offset.reset` (default `earliest`).
    pub(crate) fn auto_offset_reset(mut self, v: &str) -> Self {
        self.auto_offset_reset = v.to_string();
        self
    }

    /// Sets `enable.auto.commit` (default `false`).
    pub(crate) fn enable_auto_commit(mut self, on: bool) -> Self {
        self.enable_auto_commit = on;
        self
    }

    /// Sets `session.timeout.ms` (default 6s).
    pub(crate) fn session_timeout(mut self, d: Duration) -> Self {
        self.session_timeout = d;
        self
    }

    /// Sets an arbitrary raw rdkafka consumer config key.
    pub(crate) fn set(mut self, key: &str, val: &str) -> Self {
        self.overrides.push((key.to_string(), val.to_string()));
        self
    }

    fn resolved_group(&self) -> String {
        self.group_id.clone().unwrap_or_else(|| {
            let n = GROUP_SEQ.fetch_add(1, Ordering::Relaxed);
            format!("kafka-test-group-{n}")
        })
    }

    fn base_config(&self, group: &str) -> ClientConfig {
        let mut cfg = ClientConfig::new();
        let _ = cfg
            .set("bootstrap.servers", &self.brokers)
            .set("group.id", group)
            .set("auto.offset.reset", &self.auto_offset_reset)
            .set(
                "enable.auto.commit",
                if self.enable_auto_commit {
                    "true"
                } else {
                    "false"
                },
            )
            .set(
                "session.timeout.ms",
                self.session_timeout.as_millis().to_string(),
            );
        for (k, v) in &self.overrides {
            let _ = cfg.set(k, v);
        }
        cfg
    }

    /// Builds a consumer subscribed to `topics`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the consumer cannot be created or subscribed.
    pub(crate) fn subscribe(self, topics: &[&str]) -> TestConsumer {
        let group = self.resolved_group();
        let consumer: StreamConsumer = self
            .base_config(&group)
            .create()
            .expect("kafka-test: failed to create test consumer");
        consumer
            .subscribe(topics)
            .expect("kafka-test: failed to subscribe test consumer");
        TestConsumer {
            consumer,
            brokers: self.brokers,
            group,
            pending: RefCell::new(VecDeque::new()),
        }
    }

    /// Builds a consumer manually assigned to a single `(topic, partition)`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the consumer cannot be created or assigned.
    pub(crate) fn assign_partition(self, topic: &str, partition: i32) -> TestConsumer {
        let group = self.resolved_group();
        let consumer: StreamConsumer = self
            .base_config(&group)
            .create()
            .expect("kafka-test: failed to create test consumer");
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition(topic, partition);
        consumer
            .assign(&tpl)
            .expect("kafka-test: failed to assign partition");
        TestConsumer {
            consumer,
            brokers: self.brokers,
            group,
            pending: RefCell::new(VecDeque::new()),
        }
    }
}

/// A consumer wired to the mock broker.
pub(crate) struct TestConsumer {
    consumer: StreamConsumer,
    brokers: String,
    group: String,
    /// Records received while driving a rebalance (e.g. in
    /// [`TestConsumer::wait_for_assignment`]) are buffered here so a subsequent
    /// [`TestConsumer::recv`]/[`TestConsumer::try_recv`] returns them instead of
    /// silently dropping records the caller produced beforehand.
    pending: RefCell<VecDeque<ConsumedMessage>>,
}

impl TestConsumer {
    /// Receives one message within the default 30s timeout.
    ///
    /// # Panics
    ///
    /// Panics if no message arrives before the timeout or on a consume error.
    pub(crate) async fn recv(&self) -> ConsumedMessage {
        self.try_recv(DEFAULT_RECV_TIMEOUT)
            .await
            .expect("kafka-test: timed out waiting for a record")
    }

    /// Receives one message within `timeout`, returning `None` on timeout.
    ///
    /// Any records buffered while driving a rebalance (see
    /// [`TestConsumer::wait_for_assignment`]) are returned first, before polling
    /// the broker.
    ///
    /// # Panics
    ///
    /// Panics on a consume error (a broker/client failure, distinct from an
    /// empty poll).
    pub(crate) async fn try_recv(&self, timeout: Duration) -> Option<ConsumedMessage> {
        if let Some(buffered) = self.pending.borrow_mut().pop_front() {
            return Some(buffered);
        }
        match tokio::time::timeout(timeout, self.consumer.recv()).await {
            Ok(Ok(msg)) => Some(ConsumedMessage::from_borrowed(&msg)),
            Ok(Err(e)) => panic!("kafka-test: consume error: {e}"),
            Err(_) => None,
        }
    }

    /// Receives exactly `n` messages, each bounded by the default recv timeout.
    ///
    /// # Panics
    ///
    /// Panics if fewer than `n` messages arrive in time.
    pub(crate) async fn recv_n(&self, n: usize) -> Vec<ConsumedMessage> {
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let msg = self
                .try_recv(DEFAULT_RECV_TIMEOUT)
                .await
                .unwrap_or_else(|| panic!("kafka-test: timed out waiting for record {i} of {n}"));
            out.push(msg);
        }
        out
    }

    /// Collects messages until none arrives within `idle` (drain semantics).
    pub(crate) async fn collect_until_idle(&self, idle: Duration) -> Vec<ConsumedMessage> {
        let mut out = Vec::new();
        while let Some(msg) = self.try_recv(idle).await {
            out.push(msg);
        }
        out
    }

    /// Returns the consumer's current partition assignment as `(topic, partition)`.
    pub(crate) fn assignment(&self) -> Vec<(String, i32)> {
        self.consumer
            .assignment()
            .map(|tpl| {
                tpl.elements()
                    .iter()
                    .map(|e| (e.topic().to_string(), e.partition()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Drives the consumer until at least `min_partitions` are assigned or
    /// `timeout` elapses. Returns whether the assignment threshold was reached.
    ///
    /// Polling to advance group membership may surface application records; they
    /// are buffered (not discarded) so a subsequent
    /// [`TestConsumer::recv`]/[`TestConsumer::try_recv`] returns them. This
    /// preserves records the caller produced before calling this method.
    pub(crate) async fn wait_for_assignment(
        &self,
        min_partitions: usize,
        timeout: Duration,
    ) -> bool {
        poll_until_async(timeout, Duration::from_millis(250), || async {
            if self.assignment().len() >= min_partitions {
                return true;
            }
            // Poll to advance group membership. Buffer any record we receive so
            // it is not lost from later `recv`/`try_recv` assertions.
            if let Ok(Ok(msg)) =
                tokio::time::timeout(Duration::from_millis(250), self.consumer.recv()).await
            {
                self.pending
                    .borrow_mut()
                    .push_back(ConsumedMessage::from_borrowed(&msg));
            }
            self.assignment().len() >= min_partitions
        })
        .await
    }

    /// Returns the committed offset for `(topic, partition)` in this consumer's
    /// group, via an independent probe client.
    ///
    /// `Ok(None)` means no offset has been committed; `Err` means the probe
    /// itself failed (broker error/timeout) and is deliberately distinct from
    /// `Ok(None)` so a broken probe cannot masquerade as "uncommitted".
    ///
    /// # Errors
    ///
    /// Returns [`TestError::Consume`] if the committed-offset probe fails.
    pub(crate) fn committed_offset(
        &self,
        topic: &str,
        partition: i32,
    ) -> Result<Option<i64>, TestError> {
        committed_offset(&self.brokers, &self.group, topic, partition)
    }

    /// Returns committed offsets for several `(topic, partition)` pairs.
    ///
    /// # Errors
    ///
    /// Returns [`TestError::Consume`] if any committed-offset probe fails.
    pub(crate) fn committed_offsets(
        &self,
        tps: &[(&str, i32)],
    ) -> Result<Vec<Option<i64>>, TestError> {
        tps.iter()
            .map(|(t, p)| self.committed_offset(t, *p))
            .collect()
    }

    /// Asserts this consumer currently holds exactly `n` partitions.
    pub(crate) fn assert_assignment_count(&self, n: usize) -> &Self {
        assert_eq!(
            self.assignment().len(),
            n,
            "unexpected partition assignment count"
        );
        self
    }

    /// Asserts `(topic, partition)` is in this consumer's current assignment.
    pub(crate) fn assert_assigned(&self, topic: &str, partition: i32) -> &Self {
        assert!(
            self.assignment()
                .iter()
                .any(|(t, p)| t == topic && *p == partition),
            "expected {topic:?}/{partition} to be assigned"
        );
        self
    }

    /// Asserts `(topic, partition)` is not in this consumer's current
    /// assignment.
    pub(crate) fn assert_not_assigned(&self, topic: &str, partition: i32) -> &Self {
        assert!(
            !self
                .assignment()
                .iter()
                .any(|(t, p)| t == topic && *p == partition),
            "expected {topic:?}/{partition} not to be assigned"
        );
        self
    }

    /// Asserts the committed offset for `(topic, partition)` in this consumer's
    /// group equals `expected`.
    pub(crate) fn assert_committed_offset(
        &self,
        topic: &str,
        partition: i32,
        expected: Option<i64>,
    ) -> &Self {
        assert_eq!(
            self.committed_offset(topic, partition)
                .expect("committed-offset probe failed"),
            expected,
            "unexpected committed offset on {topic:?}/{partition}"
        );
        self
    }

    /// Asserts no further record arrives within `idle` (the stream is drained).
    ///
    /// # Panics
    ///
    /// Panics (reporting the stray record) if any record arrives before `idle`
    /// elapses.
    pub(crate) async fn assert_no_more_messages(&self, idle: Duration) {
        if let Some(msg) = self.try_recv(idle).await {
            panic!(
                "kafka-test: expected no more records, got one on {}/{} at offset {}",
                msg.topic, msg.partition, msg.offset
            );
        }
    }
}

/// Reads the committed offset for `(topic, partition)` in `group`, using an
/// independent [`BaseConsumer`] probe client.
///
/// `Ok(Some(offset))` when an offset has been committed, `Ok(None)` when none
/// has, and `Err` when the probe itself failed. Keeping the probe error
/// distinct from `Ok(None)` prevents a broker error/timeout from masquerading
/// as "no offset committed" (which would let negative assertions pass while
/// inspection is broken).
///
/// # Errors
///
/// Returns [`TestError::Consume`] if the committed-offset probe fails.
///
/// # Panics
///
/// Panics (with context) if the probe client cannot be created (a
/// test-environment setup bug).
pub(crate) fn committed_offset(
    brokers: &str,
    group: &str,
    topic: &str,
    partition: i32,
) -> Result<Option<i64>, TestError> {
    let consumer: BaseConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group)
        .set("enable.auto.commit", "false")
        .create()
        .expect("kafka-test: failed to create committed-offset probe consumer");

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition(topic, partition);

    let committed = consumer
        .committed_offsets(tpl, COMMITTED_PROBE_TIMEOUT)
        .map_err(|e| TestError::Consume(format!("committed-offset probe failed: {e}")))?;
    match committed.find_partition(topic, partition) {
        Some(elem) => match elem.offset() {
            Offset::Offset(o) => Ok(Some(o)),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

/// A lightweight extra group member used to force a rebalance.
///
/// It joins `group`, subscribes to `topics`, and is polled until it holds an
/// assignment (which drives the revoke on the other members). Keep it alive for
/// as long as the revoke must persist; drop it to let ownership revert.
///
/// This is a throwaway member whose only purpose is to trigger a rebalance on
/// the node/consumer under test; any records it receives while polling for its
/// assignment are intentionally discarded. Do not use it to consume application
/// data -- use a [`TestConsumer`] (which buffers records) for that.
pub(crate) struct RebalanceTrigger {
    consumer: StreamConsumer,
}

impl RebalanceTrigger {
    /// Joins `group` on `topics` and polls until assigned or `timeout` elapses.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the consumer cannot be created/subscribed or if
    /// it never receives an assignment before `timeout`.
    pub(crate) async fn join(
        cluster: &super::cluster::KafkaTestCluster,
        group: &str,
        topics: &[&str],
        timeout: Duration,
    ) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", cluster.bootstrap_servers())
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("kafka-test: failed to create rebalance-trigger consumer");
        consumer
            .subscribe(topics)
            .expect("kafka-test: rebalance-trigger subscribe failed");

        let trigger = Self { consumer };
        let assigned = poll_until_async(timeout, Duration::from_millis(250), || async {
            if !trigger.assignment().is_empty() {
                return true;
            }
            let _ = tokio::time::timeout(Duration::from_millis(500), trigger.consumer.recv()).await;
            !trigger.assignment().is_empty()
        })
        .await;
        assert!(
            assigned,
            "kafka-test: rebalance trigger never received an assignment; rebalance did not occur"
        );
        trigger
    }

    /// Returns the trigger consumer's current assignment.
    pub(crate) fn assignment(&self) -> Vec<(String, i32)> {
        self.consumer
            .assignment()
            .map(|tpl| {
                tpl.elements()
                    .iter()
                    .map(|e| (e.topic().to_string(), e.partition()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
