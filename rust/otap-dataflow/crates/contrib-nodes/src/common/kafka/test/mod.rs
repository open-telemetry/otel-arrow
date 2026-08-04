// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-process Kafka test suite built on `rdkafka::mocking::MockCluster`.
//!
//! The test suite lets a test stand up an in-process mock broker, connect clients
//! (or, via [`super::node_harness`], the Kafka exporter/receiver), produce and
//! consume messages, inspect broker state, and drive failure/rebalance
//! scenarios -- all without Docker or an external broker.
//!
//! # Threading
//!
//! [`cluster::KafkaTestCluster`] wraps a `!Send` `MockCluster` that must live on
//! its creation thread for the broker's whole lifetime. Tests therefore run on
//! a current-thread `LocalSet`; use [`with_cluster`] or [`run_on_local_set`],
//! which own the `LocalSet`, build the cluster, and keep the mock alive for the
//! duration of the supplied closure. Clients and node harnesses spawned via
//! `spawn_local` share the cluster through an [`Rc`].
//!
//! # Usage
//!
//! ```ignore
//! use crate::common::kafka::test::cluster::KafkaTestCluster;
//! use crate::common::kafka::test::with_cluster;
//!
//! with_cluster(KafkaTestCluster::builder().topic("it-logs"), |cluster| async move {
//!     let consumer = cluster.consumer().subscribe(&["it-logs"]);
//!     let producer = cluster.producer().build();
//!     producer.send("it-logs", b"payload").await.unwrap();
//!     consumer.recv().await.assert_topic("it-logs").assert_payload(b"payload");
//! })
//! .await;
//! ```

pub(crate) mod cluster;
pub(crate) mod consumer;
pub(crate) mod error;
pub(crate) mod faults;
pub(crate) mod inspect;
pub(crate) mod message;
pub(crate) mod producer;
pub(crate) mod wait;

use std::future::Future;
use std::rc::Rc;

use tokio::task::LocalSet;

use cluster::{KafkaTestCluster, KafkaTestClusterBuilder};

/// Runs `f` on a current-thread `LocalSet` with a live cluster built from
/// `builder`. The cluster is shared as an [`Rc`] so multiple clients/harnesses
/// spawned with `spawn_local` can reference it, and it is kept alive until `f`
/// completes (which keeps the `!Send` mock broker serving).
pub(crate) async fn with_cluster<F, Fut, T>(builder: KafkaTestClusterBuilder, f: F) -> T
where
    F: FnOnce(Rc<KafkaTestCluster>) -> Fut,
    Fut: Future<Output = T>,
{
    let cluster = Rc::new(builder.build());
    let local = LocalSet::new();
    local.run_until(f(cluster)).await
}

/// Convenience for the common `#[tokio::test]` shape: builds a default
/// single-broker cluster (no pre-created topics) and runs `f` on a `LocalSet`.
pub(crate) async fn run_on_local_set<F, Fut, T>(f: F) -> T
where
    F: FnOnce(Rc<KafkaTestCluster>) -> Fut,
    Fut: Future<Output = T>,
{
    with_cluster(KafkaTestCluster::builder(), f).await
}

#[cfg(test)]
mod demo_tests;
