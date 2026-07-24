// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Producer utilities for feeding the mock broker.
//!
//! The receiver *feeds* the broker, so the producer is a first-class utility
//! (not just an exporter afterthought). It supports keys, headers, and explicit
//! target partitions, plus the receiver rebalance-test pattern of producing N
//! records to each of P partitions ([`TestProducer::produce_per_partition`]).

use std::time::Duration;

use rdkafka::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;

use super::error::TestError;

/// Default `message.timeout.ms` for the test producer (matches the receiver
/// tests' historical value).
const DEFAULT_MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);

/// Builder for a [`TestProducer`] bound to a cluster's bootstrap servers.
#[must_use]
pub(crate) struct TestProducerBuilder {
    brokers: String,
    client_id: Option<String>,
    message_timeout: Duration,
    overrides: Vec<(String, String)>,
}

impl TestProducerBuilder {
    /// Creates a builder targeting `brokers`.
    pub(crate) fn new(brokers: &str) -> Self {
        Self {
            brokers: brokers.to_string(),
            client_id: None,
            message_timeout: DEFAULT_MESSAGE_TIMEOUT,
            overrides: Vec::new(),
        }
    }

    /// Sets the producer `client.id`.
    pub(crate) fn client_id(mut self, id: &str) -> Self {
        self.client_id = Some(id.to_string());
        self
    }

    /// Overrides the `message.timeout.ms` (default 20s).
    pub(crate) fn message_timeout(mut self, d: Duration) -> Self {
        self.message_timeout = d;
        self
    }

    /// Sets an arbitrary raw rdkafka producer config key.
    pub(crate) fn set(mut self, key: &str, val: &str) -> Self {
        self.overrides.push((key.to_string(), val.to_string()));
        self
    }

    /// Builds the [`TestProducer`].
    ///
    /// # Panics
    ///
    /// Panics (with context) if the producer client cannot be created.
    pub(crate) fn build(self) -> TestProducer {
        let mut cfg = ClientConfig::new();
        let _ = cfg.set("bootstrap.servers", &self.brokers).set(
            "message.timeout.ms",
            self.message_timeout.as_millis().to_string(),
        );
        if let Some(id) = &self.client_id {
            let _ = cfg.set("client.id", id);
        }
        for (k, v) in &self.overrides {
            let _ = cfg.set(k, v);
        }
        let inner: FutureProducer = cfg
            .create()
            .expect("kafka-test: failed to create test producer");
        TestProducer {
            inner,
            message_timeout: self.message_timeout,
        }
    }
}

/// Fluent single-record spec (payload + optional key/headers/partition).
#[must_use]
pub(crate) struct SendRecord<'a> {
    topic: &'a str,
    payload: &'a [u8],
    key: Option<&'a [u8]>,
    partition: Option<i32>,
    headers: Vec<(String, Vec<u8>)>,
}

impl<'a> SendRecord<'a> {
    /// Creates a record targeting `topic` with `payload`.
    pub(crate) fn new(topic: &'a str, payload: &'a [u8]) -> Self {
        Self {
            topic,
            payload,
            key: None,
            partition: None,
            headers: Vec::new(),
        }
    }

    /// Sets the record key.
    pub(crate) fn key(mut self, key: &'a [u8]) -> Self {
        self.key = Some(key);
        self
    }

    /// Pins the record to a specific partition.
    pub(crate) fn partition(mut self, partition: i32) -> Self {
        self.partition = Some(partition);
        self
    }

    /// Adds a header.
    pub(crate) fn header(mut self, key: &str, value: &[u8]) -> Self {
        self.headers.push((key.to_string(), value.to_vec()));
        self
    }
}

/// A producer wired to the mock broker.
pub(crate) struct TestProducer {
    inner: FutureProducer,
    message_timeout: Duration,
}

impl TestProducer {
    /// Produces a single payload to `topic` (no key/headers/partition).
    ///
    /// # Errors
    ///
    /// Returns [`TestError::Produce`] if delivery fails or times out.
    pub(crate) async fn send(&self, topic: &str, payload: &[u8]) -> Result<(), TestError> {
        self.send_full(SendRecord::new(topic, payload)).await
    }

    /// Produces a fully-specified record.
    ///
    /// # Errors
    ///
    /// Returns [`TestError::Produce`] if delivery fails or times out.
    pub(crate) async fn send_full(&self, rec: SendRecord<'_>) -> Result<(), TestError> {
        let mut headers = OwnedHeaders::new();
        for (k, v) in &rec.headers {
            headers = headers.insert(Header {
                key: k,
                value: Some(v.as_slice()),
            });
        }

        let mut record = FutureRecord::to(rec.topic).payload(rec.payload);
        if let Some(key) = rec.key {
            record = record.key(key);
        }
        if let Some(partition) = rec.partition {
            record = record.partition(partition);
        }
        if !rec.headers.is_empty() {
            record = record.headers(headers);
        }

        self.inner
            .send(record, Timeout::After(self.message_timeout))
            .await
            .map(|_delivery| ())
            .map_err(|(e, _msg)| TestError::Produce(e.to_string()))
    }

    /// Produces `payload` to a specific partition of `topic`.
    ///
    /// # Errors
    ///
    /// Returns [`TestError::Produce`] if delivery fails or times out.
    pub(crate) async fn send_to_partition(
        &self,
        topic: &str,
        partition: i32,
        payload: &[u8],
    ) -> Result<(), TestError> {
        self.send_full(SendRecord::new(topic, payload).partition(partition))
            .await
    }

    /// Produces each payload in `payloads` to `topic` in order.
    ///
    /// # Panics
    ///
    /// Panics if any send fails; batch helpers are used where a failure is a
    /// test bug.
    pub(crate) async fn send_n(&self, topic: &str, payloads: &[&[u8]]) {
        for (i, payload) in payloads.iter().enumerate() {
            self.send(topic, payload)
                .await
                .unwrap_or_else(|e| panic!("kafka-test: failed to send record {i}: {e}"));
        }
    }

    /// Produces `count` records to each partition in `0..partitions`, keyed
    /// `k-{partition}-{i}`. This folds the receiver rebalance-test pattern into
    /// a single call.
    ///
    /// # Panics
    ///
    /// Panics if any send fails.
    pub(crate) async fn produce_per_partition(
        &self,
        topic: &str,
        partitions: i32,
        count: i32,
        payload: &[u8],
    ) {
        for partition in 0..partitions {
            for i in 0..count {
                let key = format!("k-{partition}-{i}");
                self.send_full(
                    SendRecord::new(topic, payload)
                        .key(key.as_bytes())
                        .partition(partition),
                )
                .await
                .unwrap_or_else(|e| {
                    panic!("kafka-test: failed to send record {i} to partition {partition}: {e}")
                });
            }
        }
    }

    /// Flushes any buffered records, waiting up to `timeout`.
    pub(crate) fn flush(&self, timeout: Duration) {
        let _ = self.inner.flush(Timeout::After(timeout));
    }
}
