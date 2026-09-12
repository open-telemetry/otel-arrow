// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka partition keys derived from transport headers.
//!
//! Headers are hashed into a fixed-size hexadecimal key.
//! The configured [`PartitionerStrategy`] selects the partition.
//!
//! [`PartitionerStrategy`]: super::config::PartitionerStrategy

use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use std::hash::{Hash, Hasher};
use xxhash_rust::xxh64::Xxh64;

/// Builds a deterministic key from exact stored header names and values.
///
/// Header order and original wire-name casing do not affect the key. The
/// configured casing of a custom stored name does affect the key.
/// Returns a 16-character hexadecimal key, or `None` for empty headers.
#[must_use]
pub fn partition_key_from_transport_headers(headers: &TransportHeaders) -> Option<String> {
    if headers.is_empty() {
        return None;
    }

    // Sort by exact stored name, then value. Ignore original wire-name casing
    // and input order.
    let mut sorted: Vec<&_> = headers.iter().collect();
    sorted.sort_unstable_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.value.bytes.cmp(&b.value.bytes))
    });

    let mut hasher = Xxh64::new(0);
    for header in sorted {
        header.name.hash(&mut hasher);
        header.value.bytes.hash(&mut hasher);
    }
    let hash = hasher.finish();

    Some(hex::encode(hash.to_be_bytes()))
}

/// Determine the partition key for a signal based on its per-signal config and
/// the pdata context.
///
/// This is the main entry point called by the exporter for each message. It
/// inspects the signal's partitioning options and delegates to the appropriate
/// key-generation function:
///
/// 1. If [`SignalConfig::partition_by_transport_headers`] is `true` and the
///    context carries transport headers, all headers are serialized into a key
///    via [`partition_key_from_transport_headers`].
/// 2. Otherwise, returns `None` which leaves the Kafka key unset (null key),
///    ensuring true round-robin partitioning under all [`PartitionerStrategy`]
///    variants.
///
/// [`SignalConfig::partition_by_transport_headers`]: super::config::SignalConfig::partition_by_transport_headers
/// [`PartitionerStrategy`]: super::config::PartitionerStrategy
#[must_use]
pub fn partition_key_for_signal(
    signal_config: &super::config::SignalConfig,
    context: &otel_arrow_dfe_otap::pdata::Context,
) -> Option<String> {
    if signal_config.partition_by_transport_headers()
        && let Some(headers) = context.transport_headers()
    {
        return partition_key_from_transport_headers(headers);
    }

    None
}

// TODO: Explore `partition_by_trace_id` -- partition traces by hex-encoded trace ID.
//   Trace IDs are 16-byte FixedSizeBinary values in the OTAP Arrow Spans schema.
//   A single OTAP batch can contain spans with different trace IDs, so implementing
//   this requires splitting the batch into sub-batches grouped by trace ID (returning
//   something like `Vec<(String, RoaringBitmap)>`) before sending each sub-batch to
//   Kafka with its own partition key. Empty/zero trace IDs should map to an empty
//   partition key (round-robin).

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::{TransportHeader, ValueKind};

    fn context_name(raw: &str) -> otel_arrow_dfe_config::ContextEntryName {
        raw.try_into().expect("valid test context entry name")
    }

    fn transport_header_text(
        normalized_name: &str,
        wire_name: &str,
        value: impl Into<Vec<u8>>,
    ) -> TransportHeader {
        TransportHeader::captured(
            context_name(normalized_name),
            wire_name,
            true,
            ValueKind::Text,
            value.into(),
        )
    }

    fn transport_header_binary(
        normalized_name: &str,
        wire_name: &str,
        value: impl Into<Vec<u8>>,
    ) -> TransportHeader {
        TransportHeader::captured(
            context_name(normalized_name),
            wire_name,
            true,
            ValueKind::Binary,
            value.into(),
        )
    }

    // ---- Transport header partition key tests ----

    #[test]
    fn test_empty_transport_headers_returns_none() {
        let headers = TransportHeaders::new();
        let key = partition_key_from_transport_headers(&headers);
        assert!(key.is_none());
    }

    #[test]
    fn test_transport_headers_key_is_deterministic() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));
        headers.push(transport_header_text("x_region", "X-Region", b"us-east-1"));

        let key1 = partition_key_from_transport_headers(&headers);
        let key2 = partition_key_from_transport_headers(&headers);

        assert_eq!(key1, key2);
        assert!(key1.is_some());
    }

    #[test]
    fn test_transport_headers_different_values_produce_different_keys() {
        let mut headers1 = TransportHeaders::new();
        headers1.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-a",
        ));

        let mut headers2 = TransportHeaders::new();
        headers2.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-b",
        ));

        let key1 = partition_key_from_transport_headers(&headers1);
        let key2 = partition_key_from_transport_headers(&headers2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_transport_headers_different_names_produce_different_keys() {
        let mut headers1 = TransportHeaders::new();
        headers1.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"value",
        ));

        let mut headers2 = TransportHeaders::new();
        headers2.push(transport_header_text("x_region", "X-Region", b"value"));

        let key1 = partition_key_from_transport_headers(&headers1);
        let key2 = partition_key_from_transport_headers(&headers2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_transport_headers_key_is_fixed_size_hex() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // The key is the hex encoding of a u64 hash (8 bytes -> 16 hex chars),
        // regardless of how large the header set is.
        assert_eq!(key.len(), 16);
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()),
            "key should be lowercase hex ASCII"
        );
    }

    #[test]
    fn test_transport_headers_binary_values_produce_fixed_size_hex() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_binary(
            "x_binary",
            "X-Binary",
            [0x01, 0x02, 0x03, 0xFF],
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // Binary header values are folded into the same fixed-size hex key.
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Transport headers pushed in different order must produce the same key.
    #[test]
    fn test_transport_headers_order_independent() {
        let mut headers_ab = TransportHeaders::new();
        headers_ab.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));
        headers_ab.push(transport_header_text("x_region", "X-Region", b"us-east-1"));

        // Same headers, reversed insertion order.
        let mut headers_ba = TransportHeaders::new();
        headers_ba.push(transport_header_text("x_region", "X-Region", b"us-east-1"));
        headers_ba.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key_ab = partition_key_from_transport_headers(&headers_ab);
        let key_ba = partition_key_from_transport_headers(&headers_ba);

        assert_eq!(
            key_ab, key_ba,
            "partition key must be independent of header insertion order"
        );
    }

    // ---- Hashing regression test ----
    //
    // Pins the hashing pipeline for a known input. The key is the hex encoding
    // of the big-endian `u64` XXH64 hash produced by folding the sorted headers
    // into a single hasher. If the hashing structure or hasher changes it would
    // change which partition a given header set maps to, causing a Kafka
    // partition rebalance. We recompute the expected hash from the documented
    // pipeline so a change in either is caught.

    #[test]
    fn test_transport_headers_hash_matches_documented_pipeline() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // Reconstruct the documented pipeline for a single header:
        //   sort -> single hasher -> [name, value]
        //   name = "x_tenant_id", value = "tenant-123"
        let mut hasher = Xxh64::new(0);
        "x_tenant_id".hash(&mut hasher);
        b"tenant-123".to_vec().hash(&mut hasher);
        let expected = hex::encode(hasher.finish().to_be_bytes());

        assert_eq!(
            key, expected,
            "partition key no longer matches the documented hash pipeline \u{2014} \
             this will cause a Kafka partition rebalance"
        );
        assert_eq!(key.len(), 16);
    }

    // ---- partition_key_for_signal tests ----

    use crate::common::kafka::MessageFormat;
    use crate::exporters::kafka_exporter::config::SignalConfig;
    use otel_arrow_dfe_otap::pdata::Context;

    #[test]
    fn test_partition_key_for_signal_disabled_returns_none() {
        let config = SignalConfig::new("topic".into(), MessageFormat::OtlpProto);
        let context = Context::default();
        let key = partition_key_for_signal(&config, &context);
        assert!(key.is_none());
    }

    #[test]
    fn test_partition_key_for_signal_enabled_no_headers_returns_none() {
        let config = SignalConfig::new("topic".into(), MessageFormat::OtlpProto)
            .with_partition_by_transport_headers(true);
        let context = Context::default();
        let key = partition_key_for_signal(&config, &context);
        assert!(key.is_none());
    }

    #[test]
    fn test_partition_key_for_signal_enabled_with_headers() {
        let config = SignalConfig::new("topic".into(), MessageFormat::OtlpProto)
            .with_partition_by_transport_headers(true);

        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let mut context = Context::default();
        context.set_transport_headers(headers.clone());
        let key = partition_key_for_signal(&config, &context);

        // Should match what partition_key_from_transport_headers produces.
        let expected = partition_key_from_transport_headers(&headers);
        assert_eq!(key, expected);
        assert!(key.is_some());
    }

    // ---- Security: header values are never emitted in plaintext ----

    /// Scenario: derive a partition key from a sensitive header value (an auth
    /// token) and a sensitive header name.
    /// Guarantees: the emitted key never contains the raw header value or name
    /// as a substring -- it is a fixed-size hash, so a tenant/token identifier
    /// is not exposed in plaintext via the Kafka record key.
    #[test]
    fn partition_key_never_contains_raw_header_value_or_name() {
        let secret_value = "Bearer-super-secret-token-abcdef0123456789";
        let sensitive_name = "authorization";
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            sensitive_name,
            "Authorization",
            secret_value.as_bytes(),
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers produce a key");

        assert!(
            !key.contains(secret_value),
            "partition key must not leak the raw header value: {key}"
        );
        assert!(
            !key.contains("secret") && !key.contains("token"),
            "partition key must not leak fragments of the header value: {key}"
        );
        assert!(
            !key.contains(sensitive_name),
            "partition key must not leak the raw header name: {key}"
        );
        // It is a fixed-size lowercase-hex hash, not the plaintext.
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Scenario: the same sensitive header set is hashed on two separate
    /// requests.
    /// Guarantees: the key is stable/deterministic (co-location works) while
    /// remaining a non-plaintext hash -- the recorded fingerprinting tradeoff.
    #[test]
    fn partition_key_is_stable_hash_for_same_sensitive_header() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-super-secret",
        ));
        let k1 = partition_key_from_transport_headers(&headers).expect("key");
        let k2 = partition_key_from_transport_headers(&headers).expect("key");
        assert_eq!(k1, k2, "same headers must produce the same (stable) key");
        assert!(!k1.contains("secret"), "key must not be plaintext: {k1}");
    }
}
