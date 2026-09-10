// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! DLQ record header construction.
//!
//! Builds the `dlq.*` error-context headers attached to every dead-lettered
//! record and copies through the original source message headers so the DLQ
//! record is a faithful, self-describing superset of the original.

use crate::common::kafka::sanitize_for_log;
use rdkafka::message::{Header, Headers, OwnedHeaders};

/// Header key: the error message that caused the dead-letter.
pub(crate) const DLQ_ERROR: &str = "dlq.error";
/// Header key: the failure category (`decode`, `unknown_topic`, `permanent_nack`).
pub(crate) const DLQ_REASON: &str = "dlq.reason";
/// Header key: the original source topic.
pub(crate) const DLQ_SOURCE_TOPIC: &str = "dlq.source.topic";
/// Header key: the original source partition.
pub(crate) const DLQ_SOURCE_PARTITION: &str = "dlq.source.partition";
/// Header key: the original source offset.
pub(crate) const DLQ_SOURCE_OFFSET: &str = "dlq.source.offset";
/// Header key: the signal (`traces`, `metrics`, `logs`, `unknown`).
pub(crate) const DLQ_SIGNAL: &str = "dlq.signal";
/// Header key: the time (unix millis) the message was dead-lettered.
pub(crate) const DLQ_TIMESTAMP: &str = "dlq.timestamp";

/// Immutable context describing why and where a message is being dead-lettered.
#[derive(Debug, Clone)]
pub(crate) struct DlqHeaderContext {
    /// The error/reason string (e.g. the decode error message).
    pub(crate) error: String,
    /// The failure category, as its wire string.
    pub(crate) reason: &'static str,
    /// The original source topic.
    pub(crate) source_topic: String,
    /// The original source partition.
    pub(crate) source_partition: i32,
    /// The original source offset.
    pub(crate) source_offset: i64,
    /// The signal string (`traces`, `metrics`, `logs`, `unknown`).
    pub(crate) signal: &'static str,
    /// The time (unix millis) the message was dead-lettered.
    pub(crate) timestamp_millis: i64,
}

/// Build the [`OwnedHeaders`] for a DLQ record: the `dlq.*` context headers plus
/// a passthrough of the original source headers.
///
/// The error and source-topic strings are sanitized (bounded, control chars
/// escaped) because they can carry adversarial, client-controlled bytes.
pub(crate) fn build_dlq_headers(
    ctx: &DlqHeaderContext,
    original: Option<&OwnedHeaders>,
) -> OwnedHeaders {
    let mut headers = OwnedHeaders::new();

    let error = sanitize_for_log(&ctx.error);
    let source_topic = sanitize_for_log(&ctx.source_topic);
    let partition = ctx.source_partition.to_string();
    let offset = ctx.source_offset.to_string();
    let timestamp = ctx.timestamp_millis.to_string();

    headers = headers.insert(Header {
        key: DLQ_ERROR,
        value: Some(error.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_REASON,
        value: Some(ctx.reason.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_SOURCE_TOPIC,
        value: Some(source_topic.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_SOURCE_PARTITION,
        value: Some(partition.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_SOURCE_OFFSET,
        value: Some(offset.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_SIGNAL,
        value: Some(ctx.signal.as_bytes()),
    });
    headers = headers.insert(Header {
        key: DLQ_TIMESTAMP,
        value: Some(timestamp.as_bytes()),
    });

    // Copy through the original source headers so the DLQ record is a faithful
    // superset. Skip any header that collides with a `dlq.*` key so the
    // injected context is authoritative.
    if let Some(original) = original {
        for i in 0..original.count() {
            let header = original.get(i);
            let key = header.key;
            if key.starts_with("dlq.") {
                continue;
            }
            headers = headers.insert(Header {
                key,
                value: header.value,
            });
        }
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> DlqHeaderContext {
        DlqHeaderContext {
            error: "boom".to_string(),
            reason: "decode",
            source_topic: "src".to_string(),
            source_partition: 3,
            source_offset: 42,
            signal: "traces",
            timestamp_millis: 1000,
        }
    }

    fn header_value(headers: &OwnedHeaders, key: &str) -> Option<Vec<u8>> {
        (0..headers.count())
            .map(|i| headers.get(i))
            .find(|h| h.key == key)
            .and_then(|h| h.value.map(<[u8]>::to_vec))
    }

    /// Scenario: build DLQ headers for a decode failure with no source headers.
    /// Guarantees: every dlq.* context header is present with the expected value.
    #[test]
    fn builds_all_context_headers() {
        let headers = build_dlq_headers(&ctx(), None);
        assert_eq!(
            header_value(&headers, DLQ_ERROR).as_deref(),
            Some(&b"boom"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_REASON).as_deref(),
            Some(&b"decode"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_SOURCE_TOPIC).as_deref(),
            Some(&b"src"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_SOURCE_PARTITION).as_deref(),
            Some(&b"3"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_SOURCE_OFFSET).as_deref(),
            Some(&b"42"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_SIGNAL).as_deref(),
            Some(&b"traces"[..])
        );
        assert_eq!(
            header_value(&headers, DLQ_TIMESTAMP).as_deref(),
            Some(&b"1000"[..])
        );
    }

    /// Scenario: an original source header is passed through, and a colliding
    /// dlq.* source header is dropped in favor of the injected context.
    /// Guarantees: source headers survive; injected dlq.* keys are authoritative.
    #[test]
    fn passes_through_source_headers_and_drops_dlq_collisions() {
        let original = OwnedHeaders::new()
            .insert(Header {
                key: "x-tenant",
                value: Some(&b"acme"[..]),
            })
            .insert(Header {
                key: "dlq.reason",
                value: Some(&b"attacker"[..]),
            });
        let headers = build_dlq_headers(&ctx(), Some(&original));
        assert_eq!(
            header_value(&headers, "x-tenant").as_deref(),
            Some(&b"acme"[..])
        );
        // Only the injected dlq.reason survives.
        let reasons: Vec<_> = (0..headers.count())
            .map(|i| headers.get(i))
            .filter(|h| h.key == DLQ_REASON)
            .collect();
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].value, Some(&b"decode"[..]));
    }

    /// Scenario: an adversarial error string with control characters is used.
    /// Guarantees: the rendered header value is sanitized (no raw newline).
    #[test]
    fn sanitizes_adversarial_error() {
        let mut c = ctx();
        c.error = "line1\nline2".to_string();
        let headers = build_dlq_headers(&c, None);
        let value = header_value(&headers, DLQ_ERROR).unwrap();
        assert!(!value.contains(&b'\n'), "raw newline must be escaped");
    }
}
