// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-process mock Kafka cluster used as the root of the test suite.
//!
//! [`KafkaTestCluster`] wraps librdkafka's [`MockCluster`], which is `!Send`
//! (it holds a raw pointer and must live on its creation thread for the whole
//! broker lifetime; dropping it tears the broker down). Because of that the
//! cluster is always driven from a current-thread `LocalSet` (see the
//! `with_cluster` / `run_on_local_set` helpers in the parent module).
//!
//! `MockCluster` in rdkafka 0.38.0 exposes only *setters/mutators* plus
//! `bootstrap_servers`; there is no query API. Read-only inspection therefore
//! goes through a normal client probe, encapsulated in [`super::inspect`].

use rdkafka::mocking::MockCluster;
use rdkafka::producer::DefaultProducerContext;

use super::consumer::TestConsumerBuilder;
use super::faults::BrokerFaults;
use super::inspect::BrokerInspector;
use super::producer::TestProducerBuilder;

/// Declarative description of a topic to pre-create on the mock broker.
///
/// The mock only auto-creates single-partition topics on produce, so any test
/// that needs a deterministic partition layout (or more than one partition)
/// must pre-create the topic through the builder.
#[derive(Debug, Clone)]
pub(crate) struct TopicSpec {
    /// Topic name.
    pub(crate) name: String,
    /// Number of partitions.
    pub(crate) partitions: i32,
    /// Replication factor.
    pub(crate) replication: i32,
}

/// Builder for a [`KafkaTestCluster`].
///
/// Defaults to a single broker with no topics. Topics added here are created
/// eagerly in [`KafkaTestClusterBuilder::build`].
#[must_use]
pub(crate) struct KafkaTestClusterBuilder {
    broker_count: i32,
    topics: Vec<TopicSpec>,
}

impl Default for KafkaTestClusterBuilder {
    fn default() -> Self {
        Self {
            broker_count: 1,
            topics: Vec::new(),
        }
    }
}

impl KafkaTestClusterBuilder {
    /// Sets the number of brokers in the mock cluster (default 1).
    pub(crate) fn broker_count(mut self, n: i32) -> Self {
        self.broker_count = n;
        self
    }

    /// Adds a topic with a single partition and replication factor 1.
    pub(crate) fn topic(self, name: impl Into<String>) -> Self {
        self.topic_with(name, 1, 1)
    }

    /// Adds a topic with an explicit partition count and replication factor.
    pub(crate) fn topic_with(
        mut self,
        name: impl Into<String>,
        partitions: i32,
        replication: i32,
    ) -> Self {
        self.topics.push(TopicSpec {
            name: name.into(),
            partitions,
            replication,
        });
        self
    }

    /// Builds the cluster, creating the mock broker and all declared topics.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the mock cluster or any topic cannot be
    /// created. Cluster setup failure is a test-environment bug, so panicking
    /// is preferred over threading a `Result` through every call site.
    pub(crate) fn build(self) -> KafkaTestCluster {
        let mock = MockCluster::new(self.broker_count)
            .expect("kafka-test: failed to create mock Kafka cluster");
        for spec in &self.topics {
            mock.create_topic(&spec.name, spec.partitions, spec.replication)
                .unwrap_or_else(|e| {
                    panic!(
                        "kafka-test: failed to create topic {:?} ({} partitions): {e}",
                        spec.name, spec.partitions
                    )
                });
        }
        let bootstrap_servers = mock.bootstrap_servers();
        KafkaTestCluster {
            mock,
            bootstrap_servers,
        }
    }
}

/// An in-process mock Kafka cluster bound to its bootstrap-servers string.
///
/// Hand this to [`super::node_harness`] wrappers, [`TestProducerBuilder`],
/// consumers, [`BrokerFaults`], and [`BrokerInspector`] to build a full
/// integration test with no external broker.
pub(crate) struct KafkaTestCluster {
    mock: MockCluster<'static, DefaultProducerContext>,
    bootstrap_servers: String,
}

impl KafkaTestCluster {
    /// Returns a builder for configuring broker count and topics.
    pub(crate) fn builder() -> KafkaTestClusterBuilder {
        KafkaTestClusterBuilder::default()
    }

    /// Convenience constructor for a cluster with `broker_count` brokers and no
    /// pre-created topics.
    pub(crate) fn new(broker_count: i32) -> Self {
        KafkaTestClusterBuilder::default()
            .broker_count(broker_count)
            .build()
    }

    /// Returns the `bootstrap.servers` string for wiring components/clients.
    pub(crate) fn bootstrap_servers(&self) -> &str {
        &self.bootstrap_servers
    }

    /// Creates a topic on the running broker after construction.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the topic cannot be created.
    pub(crate) fn create_topic(&self, name: &str, partitions: i32, replication: i32) {
        self.mock
            .create_topic(name, partitions, replication)
            .unwrap_or_else(|e| panic!("kafka-test: failed to create topic {name:?}: {e}"));
    }

    /// Returns a producer builder bound to this cluster.
    pub(crate) fn producer(&self) -> TestProducerBuilder {
        TestProducerBuilder::new(&self.bootstrap_servers)
    }

    /// Returns a consumer builder bound to this cluster.
    pub(crate) fn consumer(&self) -> TestConsumerBuilder {
        TestConsumerBuilder::new(&self.bootstrap_servers)
    }

    /// Returns broker-state mutators (fault injection) for this cluster.
    pub(crate) fn faults(&self) -> BrokerFaults<'_> {
        BrokerFaults::new(&self.mock)
    }

    /// Returns a read-only inspector for topology and watermarks.
    pub(crate) fn inspect(&self) -> BrokerInspector {
        BrokerInspector::new(&self.bootstrap_servers, None)
    }

    /// Returns a read-only inspector scoped to a consumer group, adding
    /// committed-offset queries.
    pub(crate) fn inspect_group(&self, group: &str) -> BrokerInspector {
        BrokerInspector::new(&self.bootstrap_servers, Some(group.to_string()))
    }

    /// Raw escape hatch to the underlying [`MockCluster`] for advanced cases
    /// not yet covered by the test-suite API.
    pub(crate) fn mock(&self) -> &MockCluster<'static, DefaultProducerContext> {
        &self.mock
    }
}
