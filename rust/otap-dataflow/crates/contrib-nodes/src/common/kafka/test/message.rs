// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Owned view of a consumed Kafka record plus fluent assertions.
//!
//! [`ConsumedMessage`] copies the borrowed record data out eagerly (topic,
//! partition, offset, key, payload, headers) so assertions do not fight
//! rdkafka's `BorrowedMessage` lifetimes.

use std::collections::HashMap;

use rdkafka::Message;
use rdkafka::message::{BorrowedMessage, Headers};

use crate::common::kafka::{MSG_FORMAT_HEADER, MSG_FORMAT_OTAP, MSG_FORMAT_OTLP};

/// An owned snapshot of a single consumed Kafka record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConsumedMessage {
    /// Topic the record was read from.
    pub(crate) topic: String,
    /// Partition the record landed on.
    pub(crate) partition: i32,
    /// Offset of the record within its partition.
    pub(crate) offset: i64,
    /// Record key, if any.
    pub(crate) key: Option<Vec<u8>>,
    /// Record payload, if any.
    pub(crate) payload: Option<Vec<u8>>,
    /// Record headers as (key, optional value) pairs, in wire order.
    pub(crate) headers: Vec<(String, Option<Vec<u8>>)>,
}

impl ConsumedMessage {
    /// Builds an owned [`ConsumedMessage`] from a borrowed rdkafka message.
    pub(crate) fn from_borrowed(msg: &BorrowedMessage<'_>) -> Self {
        let headers = msg.headers().map_or_else(Vec::new, |hs| {
            (0..hs.count())
                .map(|i| {
                    let h = hs.get(i);
                    (h.key.to_string(), h.value.map(<[u8]>::to_vec))
                })
                .collect()
        });
        Self {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            key: msg.key().map(<[u8]>::to_vec),
            payload: msg.payload().map(<[u8]>::to_vec),
            headers,
        }
    }

    /// Returns the value of the first header with `key`, if present.
    pub(crate) fn header(&self, key: &str) -> Option<&[u8]> {
        self.headers
            .iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| v.as_deref())
    }

    /// Returns the message-format header value (`MessageFormat`), if present.
    pub(crate) fn message_format(&self) -> Option<&[u8]> {
        self.header(MSG_FORMAT_HEADER)
    }

    /// Asserts the record was read from `topic`.
    pub(crate) fn assert_topic(&self, topic: &str) -> &Self {
        assert_eq!(self.topic, topic, "unexpected topic");
        self
    }

    /// Asserts the record payload equals `bytes`.
    pub(crate) fn assert_payload(&self, bytes: &[u8]) -> &Self {
        assert_eq!(
            self.payload.as_deref(),
            Some(bytes),
            "unexpected record payload"
        );
        self
    }

    /// Asserts the record key equals `key`.
    pub(crate) fn assert_key(&self, key: &[u8]) -> &Self {
        assert_eq!(self.key.as_deref(), Some(key), "unexpected record key");
        self
    }

    /// Asserts the record was read from `partition`.
    pub(crate) fn assert_partition(&self, partition: i32) -> &Self {
        assert_eq!(self.partition, partition, "unexpected partition");
        self
    }

    /// Asserts the record sits at `offset` within its partition.
    pub(crate) fn assert_offset(&self, offset: i64) -> &Self {
        assert_eq!(self.offset, offset, "unexpected offset");
        self
    }

    /// Asserts the record carries no key.
    pub(crate) fn assert_no_key(&self) -> &Self {
        assert_eq!(self.key, None, "expected record to have no key");
        self
    }

    /// Asserts the record carries no payload (e.g. a tombstone).
    pub(crate) fn assert_no_payload(&self) -> &Self {
        assert_eq!(self.payload, None, "expected record to have no payload");
        self
    }

    /// Asserts the record payload is `len` bytes long.
    pub(crate) fn assert_payload_len(&self, len: usize) -> &Self {
        assert_eq!(
            self.payload.as_deref().map(<[u8]>::len),
            Some(len),
            "unexpected payload length"
        );
        self
    }

    /// Asserts the record carries header `key` with value `val`.
    pub(crate) fn assert_header(&self, key: &str, val: &[u8]) -> &Self {
        assert_eq!(
            self.header(key),
            Some(val),
            "unexpected value for header {key:?}"
        );
        self
    }

    /// Asserts the record carries header `key` with any (or no) value.
    pub(crate) fn assert_has_header(&self, key: &str) -> &Self {
        assert!(
            self.headers.iter().any(|(k, _)| k == key),
            "expected record to carry header {key:?}"
        );
        self
    }

    /// Asserts the record does not carry header `key`.
    pub(crate) fn assert_no_header(&self, key: &str) -> &Self {
        assert!(
            !self.headers.iter().any(|(k, _)| k == key),
            "expected record not to carry header {key:?}"
        );
        self
    }

    /// Asserts the message-format header equals `fmt`.
    pub(crate) fn assert_format(&self, fmt: &[u8]) -> &Self {
        assert_eq!(
            self.message_format(),
            Some(fmt),
            "unexpected message-format header"
        );
        self
    }

    /// Asserts the message-format header marks an OTLP payload.
    pub(crate) fn assert_format_otlp(&self) -> &Self {
        self.assert_format(MSG_FORMAT_OTLP)
    }

    /// Asserts the message-format header marks an OTAP payload.
    pub(crate) fn assert_format_otap(&self) -> &Self {
        self.assert_format(MSG_FORMAT_OTAP)
    }
}

/// Returns the number of messages per topic.
// Retained as a batch-tally helper alongside `count_by_partition`; no in-tree
// caller on the branch that only finalizes the test suite.
#[allow(dead_code)]
pub(crate) fn count_by_topic(msgs: &[ConsumedMessage]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for m in msgs {
        *counts.entry(m.topic.clone()).or_insert(0) += 1;
    }
    counts
}

/// Returns the number of messages per (topic, partition).
pub(crate) fn count_by_partition(msgs: &[ConsumedMessage]) -> HashMap<(String, i32), usize> {
    let mut counts = HashMap::new();
    for m in msgs {
        *counts.entry((m.topic.clone(), m.partition)).or_insert(0) += 1;
    }
    counts
}
