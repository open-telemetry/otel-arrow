// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read-only broker/group state inspection.
//!
//! `MockCluster` 0.38.0 exposes no getters, so inspection goes through a normal
//! client probe: topology from `fetch_metadata`, offsets from
//! `fetch_watermarks`, and committed group offsets from `committed_offsets`
//! (group-scoped probe). Each query creates a short-lived [`BaseConsumer`].

use std::time::Duration;

use rdkafka::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};

/// Default probe timeout for metadata/watermark/committed queries.
const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Summary of a topic's partition count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopicInfo {
    /// Topic name.
    pub(crate) name: String,
    /// Number of partitions.
    pub(crate) partition_count: usize,
}

/// Per-partition topology: leader, replicas, and in-sync replicas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PartitionInfo {
    /// Partition id.
    pub(crate) id: i32,
    /// Leader broker id.
    pub(crate) leader: i32,
    /// Replica broker ids.
    pub(crate) replicas: Vec<i32>,
    /// In-sync replica broker ids.
    pub(crate) isr: Vec<i32>,
}

/// A read-only inspector bound to a cluster's bootstrap servers.
///
/// Construct via [`super::cluster::KafkaTestCluster::inspect`] (topology +
/// watermarks) or [`super::cluster::KafkaTestCluster::inspect_group`] (adds
/// committed-offset queries).
pub(crate) struct BrokerInspector {
    brokers: String,
    group: Option<String>,
    probe_timeout: Duration,
}

impl BrokerInspector {
    /// Creates an inspector; pass `group` to enable committed-offset queries.
    pub(crate) fn new(brokers: &str, group: Option<String>) -> Self {
        Self {
            brokers: brokers.to_string(),
            group,
            probe_timeout: DEFAULT_PROBE_TIMEOUT,
        }
    }

    /// Overrides the probe timeout (default 10s).
    pub(crate) fn probe_timeout(mut self, d: Duration) -> Self {
        self.probe_timeout = d;
        self
    }

    fn probe(&self) -> BaseConsumer {
        let mut cfg = ClientConfig::new();
        let _ = cfg.set("bootstrap.servers", &self.brokers);
        // A group is required for committed-offset probes; harmless otherwise.
        let group = self.group.as_deref().unwrap_or("kafka-test-inspector");
        let _ = cfg
            .set("group.id", group)
            .set("enable.auto.commit", "false");
        cfg.create()
            .expect("kafka-test: failed to create inspector probe consumer")
    }

    /// Returns all topics and their partition counts.
    ///
    /// # Panics
    ///
    /// Panics (with context) if metadata cannot be fetched.
    pub(crate) fn topics(&self) -> Vec<TopicInfo> {
        let probe = self.probe();
        let md = probe
            .fetch_metadata(None, self.probe_timeout)
            .expect("kafka-test: fetch_metadata failed");
        md.topics()
            .iter()
            .map(|t| TopicInfo {
                name: t.name().to_string(),
                partition_count: t.partitions().len(),
            })
            .collect()
    }

    /// Returns the partition topology for `topic`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if metadata cannot be fetched.
    pub(crate) fn partitions(&self, topic: &str) -> Vec<PartitionInfo> {
        let probe = self.probe();
        let md = probe
            .fetch_metadata(Some(topic), self.probe_timeout)
            .expect("kafka-test: fetch_metadata failed");
        md.topics()
            .iter()
            .find(|t| t.name() == topic)
            .map(|t| {
                t.partitions()
                    .iter()
                    .map(|p| PartitionInfo {
                        id: p.id(),
                        leader: p.leader(),
                        replicas: p.replicas().to_vec(),
                        isr: p.isr().to_vec(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns whether `topic` exists.
    pub(crate) fn topic_exists(&self, topic: &str) -> bool {
        !self.partitions(topic).is_empty()
    }

    /// Returns the `(low, high)` watermark offsets for `(topic, partition)`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if watermarks cannot be fetched.
    pub(crate) fn watermarks(&self, topic: &str, partition: i32) -> (i64, i64) {
        let probe = self.probe();
        probe
            .fetch_watermarks(topic, partition, self.probe_timeout)
            .expect("kafka-test: fetch_watermarks failed")
    }

    /// Returns the number of messages currently in `(topic, partition)`
    /// (`high - low`).
    pub(crate) fn message_count(&self, topic: &str, partition: i32) -> i64 {
        let (low, high) = self.watermarks(topic, partition);
        high - low
    }

    /// Returns the committed offset for `(topic, partition)` in the inspector's
    /// group.
    ///
    /// # Panics
    ///
    /// Panics if the inspector was not created with a group (use
    /// [`super::cluster::KafkaTestCluster::inspect_group`]).
    pub(crate) fn committed_offset(&self, topic: &str, partition: i32) -> Option<i64> {
        assert!(
            self.group.is_some(),
            "kafka-test: committed_offset requires a group; use inspect_group(..)"
        );
        let probe = self.probe();
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition(topic, partition);
        let committed = probe.committed_offsets(tpl, self.probe_timeout).ok()?;
        match committed.find_partition(topic, partition)?.offset() {
            Offset::Offset(o) => Some(o),
            _ => None,
        }
    }

    /// Returns committed offsets for several `(topic, partition)` pairs.
    pub(crate) fn committed_offsets(&self, tps: &[(&str, i32)]) -> Vec<Option<i64>> {
        tps.iter()
            .map(|(t, p)| self.committed_offset(t, *p))
            .collect()
    }

    /// Asserts `topic` exists on the broker.
    pub(crate) fn assert_topic_exists(&self, topic: &str) -> &Self {
        assert!(
            self.topic_exists(topic),
            "expected topic {topic:?} to exist"
        );
        self
    }

    /// Asserts `topic` does not exist on the broker.
    pub(crate) fn assert_topic_absent(&self, topic: &str) -> &Self {
        assert!(
            !self.topic_exists(topic),
            "expected topic {topic:?} to be absent"
        );
        self
    }

    /// Asserts `topic` has exactly `count` partitions.
    pub(crate) fn assert_partition_count(&self, topic: &str, count: usize) -> &Self {
        assert_eq!(
            self.partitions(topic).len(),
            count,
            "unexpected partition count for topic {topic:?}"
        );
        self
    }

    /// Asserts `(topic, partition)` currently holds exactly `count` messages
    /// (`high - low` watermark delta).
    pub(crate) fn assert_message_count(&self, topic: &str, partition: i32, count: i64) -> &Self {
        assert_eq!(
            self.message_count(topic, partition),
            count,
            "unexpected message count on {topic:?}/{partition}"
        );
        self
    }

    /// Asserts `(topic, partition)` holds at least `count` messages.
    pub(crate) fn assert_message_count_at_least(
        &self,
        topic: &str,
        partition: i32,
        count: i64,
    ) -> &Self {
        let got = self.message_count(topic, partition);
        assert!(
            got >= count,
            "expected at least {count} messages on {topic:?}/{partition}, saw {got}"
        );
        self
    }

    /// Asserts the high watermark for `(topic, partition)` equals `high`.
    pub(crate) fn assert_high_watermark(&self, topic: &str, partition: i32, high: i64) -> &Self {
        let (_low, got) = self.watermarks(topic, partition);
        assert_eq!(
            got, high,
            "unexpected high watermark on {topic:?}/{partition}"
        );
        self
    }

    /// Asserts the configured leader broker for `(topic, partition)` is
    /// `broker_id`.
    ///
    /// Note: `MockCluster` does not perform leader election, so this reflects
    /// the leader the mock reports (as set up or after an explicit reassignment
    /// fault), not an elected one.
    pub(crate) fn assert_leader(&self, topic: &str, partition: i32, broker_id: i32) -> &Self {
        let leader = self
            .partitions(topic)
            .into_iter()
            .find(|p| p.id == partition)
            .map(|p| p.leader);
        assert_eq!(
            leader,
            Some(broker_id),
            "unexpected leader for {topic:?}/{partition}"
        );
        self
    }

    /// Asserts the committed offset for `(topic, partition)` in the inspector's
    /// group equals `expected`.
    ///
    /// # Panics
    ///
    /// Panics if the inspector was not created with a group (use
    /// [`super::cluster::KafkaTestCluster::inspect_group`]).
    pub(crate) fn assert_committed_offset(
        &self,
        topic: &str,
        partition: i32,
        expected: Option<i64>,
    ) -> &Self {
        assert_eq!(
            self.committed_offset(topic, partition),
            expected,
            "unexpected committed offset on {topic:?}/{partition}"
        );
        self
    }
}
