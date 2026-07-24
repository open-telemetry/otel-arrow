// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Broker-state mutators and fault injection.
//!
//! These are thin, documented wrappers over the raw [`MockCluster`] state
//! methods. They are all *mutators*; read-only inspection lives in
//! [`super::inspect`] because `MockCluster` exposes no getters.
//!
//! rdkafka 0.38.0 caveats to keep in mind:
//! - `broker_down`/`broker_up` do **not** trigger leader election; reassign
//!   leaders manually with [`BrokerFaults::set_partition_leader`].
//! - The CreateTopics admin API is unsupported; create topics via the cluster
//!   builder / [`super::cluster::KafkaTestCluster::create_topic`].

use std::time::Duration;

use rdkafka::mocking::{MockCluster, MockCoordinator};
use rdkafka::producer::DefaultProducerContext;
use rdkafka::types::{RDKafkaApiKey, RDKafkaRespErr};

/// Broker-state mutators bound to a cluster's [`MockCluster`].
pub(crate) struct BrokerFaults<'a> {
    mock: &'a MockCluster<'static, DefaultProducerContext>,
}

impl<'a> BrokerFaults<'a> {
    /// Creates a mutator handle over `mock`.
    pub(crate) fn new(mock: &'a MockCluster<'static, DefaultProducerContext>) -> Self {
        Self { mock }
    }

    /// Marks broker `broker_id` down (`-1` = all brokers).
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn broker_down(&self, broker_id: i32) {
        self.mock
            .broker_down(broker_id)
            .unwrap_or_else(|e| panic!("kafka-test: broker_down({broker_id}) failed: {e}"));
    }

    /// Marks broker `broker_id` up (`-1` = all brokers).
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn broker_up(&self, broker_id: i32) {
        self.mock
            .broker_up(broker_id)
            .unwrap_or_else(|e| panic!("kafka-test: broker_up({broker_id}) failed: {e}"));
    }

    /// Marks all brokers down.
    pub(crate) fn all_brokers_down(&self) {
        self.broker_down(-1);
    }

    /// Marks all brokers up.
    pub(crate) fn all_brokers_up(&self) {
        self.broker_up(-1);
    }

    /// Injects a per-request round-trip delay for broker `broker_id`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn round_trip_time(&self, broker_id: i32, d: Duration) {
        self.mock
            .broker_round_trip_time(broker_id, d)
            .unwrap_or_else(|e| {
                panic!("kafka-test: broker_round_trip_time({broker_id}) failed: {e}")
            });
    }

    /// Sets (or clears, with `None`) the leader broker for `(topic, partition)`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn set_partition_leader(&self, topic: &str, partition: i32, broker: Option<i32>) {
        self.mock
            .partition_leader(topic, partition, broker)
            .unwrap_or_else(|e| {
                panic!("kafka-test: partition_leader({topic}, {partition}) failed: {e}")
            });
    }

    /// Sets the group coordinator broker for `group`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn set_group_coordinator(&self, group: &str, broker: i32) {
        self.mock
            .coordinator(MockCoordinator::Group(group.to_string()), broker)
            .unwrap_or_else(|e| panic!("kafka-test: coordinator(Group {group}) failed: {e}"));
    }

    /// Injects a sequence of response errors for `api` requests.
    pub(crate) fn inject_request_errors(&self, api: RDKafkaApiKey, errors: &[RDKafkaRespErr]) {
        self.mock.request_errors(api, errors);
    }

    /// Clears previously injected request errors for `api`.
    pub(crate) fn clear_request_errors(&self, api: RDKafkaApiKey) {
        self.mock.clear_request_errors(api);
    }

    /// Sets a topic-level error for `topic`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn set_topic_error(&self, topic: &str, err: RDKafkaRespErr) {
        self.mock
            .topic_error(topic, err)
            .unwrap_or_else(|e| panic!("kafka-test: topic_error({topic}) failed: {e}"));
    }

    /// Restricts the advertised API-version range for `api`.
    ///
    /// # Panics
    ///
    /// Panics (with context) if the operation fails at the FFI boundary.
    pub(crate) fn set_api_version(&self, api: RDKafkaApiKey, min: Option<i16>, max: Option<i16>) {
        self.mock
            .apiversion(api, min, max)
            .unwrap_or_else(|e| panic!("kafka-test: apiversion({api:?}) failed: {e}"));
    }

    // ------------------------------------------------------------------
    // Composite recipes
    //
    // Higher-level fault sequences that compose the primitives above into
    // an intention-revealing operation. These exist because the raw mock
    // knobs are easy to misuse (e.g. `broker_down`/`broker_up` do not elect
    // a new leader on their own).
    // ------------------------------------------------------------------

    /// Simulates a broker restart while keeping `(topic, partition)` served, by
    /// taking `broker_id` down, moving that partition's leadership to
    /// `new_leader`, and bringing `broker_id` back up.
    ///
    /// The mock's `broker_down`/`broker_up` do **not** trigger leader election,
    /// so a naive down/up would leave the partition leaderless. This recipe
    /// reassigns the leader explicitly so a restart/leader-election scenario is
    /// deterministic and non-flaky.
    ///
    /// # Panics
    ///
    /// Panics (with context) if any underlying mock operation fails.
    pub(crate) fn restart_broker_reassigning_leader(
        &self,
        broker_id: i32,
        topic: &str,
        partition: i32,
        new_leader: i32,
    ) {
        self.broker_down(broker_id);
        self.set_partition_leader(topic, partition, Some(new_leader));
        self.broker_up(broker_id);
    }

    /// Makes the broker fail consumer offset-commit requests with the given
    /// sequence of `errors` (one per request, in order), so a test can exercise
    /// the receiver's commit-failure path (`offset_commit_errors`).
    ///
    /// Clear with [`BrokerFaults::clear_offset_commit_failures`].
    pub(crate) fn fail_offset_commits(&self, errors: &[RDKafkaRespErr]) {
        self.inject_request_errors(RDKafkaApiKey::OffsetCommit, errors);
    }

    /// Clears any injected offset-commit failures.
    pub(crate) fn clear_offset_commit_failures(&self) {
        self.clear_request_errors(RDKafkaApiKey::OffsetCommit);
    }

    /// Makes the broker fail produce requests with the given sequence of
    /// `errors` (one per request, in order), so a test can exercise the
    /// exporter's transient-nack path.
    ///
    /// Clear with [`BrokerFaults::clear_produce_failures`].
    pub(crate) fn fail_produce(&self, errors: &[RDKafkaRespErr]) {
        self.inject_request_errors(RDKafkaApiKey::Produce, errors);
    }

    /// Clears any injected produce failures.
    pub(crate) fn clear_produce_failures(&self) {
        self.clear_request_errors(RDKafkaApiKey::Produce);
    }

    /// Makes the broker fail fetch requests with the given sequence of `errors`
    /// (one per request, in order), so a test can exercise the receiver's
    /// non-fatal transport-error path (`transport_errors`).
    ///
    /// Clear with [`BrokerFaults::clear_fetch_failures`].
    pub(crate) fn fail_fetch(&self, errors: &[RDKafkaRespErr]) {
        self.inject_request_errors(RDKafkaApiKey::Fetch, errors);
    }

    /// Clears any injected fetch failures.
    pub(crate) fn clear_fetch_failures(&self) {
        self.clear_request_errors(RDKafkaApiKey::Fetch);
    }
}
