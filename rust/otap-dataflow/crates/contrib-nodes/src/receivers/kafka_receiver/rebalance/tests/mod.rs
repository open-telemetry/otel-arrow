// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalance tests, split by concern. Shared imports and
//! helpers (including the scripted resume-consumer double) live here; each
//! concern submodule pulls them in via `use super::*`.

use super::*;
use crate::common::kafka::test::cluster::KafkaTestCluster;
use crate::common::kafka::test::with_cluster;
use rdkafka::ClientConfig;
use std::collections::VecDeque;
use std::sync::Arc;

// ---- Shared test helpers ----

/// Builds a real `BaseConsumer` wired to the receiver's rebalance context
/// against the mock `cluster`, so the private `handle_assign` / `handle_revoke`
/// methods can be driven for real while assertions stay on the deterministic
/// in-memory rebalance state (not the async broker outcome).
fn mock_base_consumer(
    cluster: &KafkaTestCluster,
    group: &str,
    state: Arc<RebalanceState>,
) -> BaseConsumer<RebalancingConsumerContext> {
    let ctx = RebalancingConsumerContext::Default(state);
    ClientConfig::new()
        .set("bootstrap.servers", cluster.bootstrap_servers())
        .set("group.id", group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create_with_context(ctx)
        .expect("failed to create mock base consumer")
}

struct ScriptedResumeConsumer {
    results: Mutex<VecDeque<Result<(), rdkafka::error::KafkaError>>>,
    attempts: AtomicU64,
}

impl ScriptedResumeConsumer {
    fn new(results: Vec<Result<(), rdkafka::error::KafkaError>>) -> Self {
        Self {
            results: Mutex::new(results.into()),
            attempts: AtomicU64::new(0),
        }
    }
}

impl PartitionResumeOperations for ScriptedResumeConsumer {
    fn resume_partitions(
        &self,
        _tpl: &TopicPartitionList,
    ) -> Result<(), rdkafka::error::KafkaError> {
        let _ = self.attempts.fetch_add(1, Ordering::Relaxed);
        lock_ignore_poison(&self.results)
            .pop_front()
            .unwrap_or(Ok(()))
    }
}

mod rebalancing;
