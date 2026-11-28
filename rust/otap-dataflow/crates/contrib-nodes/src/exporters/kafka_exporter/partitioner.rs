// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka partition keys derived from transport headers.
//!
//! Headers are hashed into a fixed-size hexadecimal key.
//! The configured [`PartitionerStrategy`] selects the partition.
//!
//! [`PartitionerStrategy`]: super::config::PartitionerStrategy

use otel_arrow_dfe_engine::pdata_context::{HeaderValue, HeadersRead, Metadata};
use std::hash::{Hash, Hasher};
use xxhash_rust::xxh64::Xxh64;

/// Builds a deterministic key from exact stored header names and values.
///
/// Header order and original wire-name casing do not affect the key. The
/// configured casing of a custom stored name does affect the key.
/// Returns a 16-character hexadecimal key, or `None` for empty headers.
#[must_use]
pub fn partition_key_from_transport_headers<'a>(
    headers: impl IntoIterator<Item = HeaderValue<'a>>,
) -> Option<String> {
    // Sort by exact stored name, then value. Ignore original wire-name casing
    // and input order.
    let mut sorted: Vec<_> = headers.into_iter().collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_unstable_by(|a, b| a.name.cmp(b.name).then_with(|| a.bytes.cmp(b.bytes)));

    let mut hasher = Xxh64::new(0);
    for header in sorted {
        header.name.hash(&mut hasher);
        header.bytes.hash(&mut hasher);
    }
    let hash = hasher.finish();

    Some(hex::encode(hash.to_be_bytes()))
}

/// Determine the partition key for a signal from its compiled all-headers
/// read and the pdata context metadata.
///
/// This is the main entry point called by the exporter for each message:
///
/// 1. If the signal enables [`SignalConfig::partition_by_transport_headers`]
///    (so `headers` is bound) and the context carries transport headers, all
///    headers are hashed into a key via [`partition_key_from_transport_headers`].
/// 2. Otherwise, returns `None` which leaves the Kafka key unset (null key),
///    ensuring true round-robin partitioning under all [`PartitionerStrategy`]
///    variants.
///
/// [`SignalConfig::partition_by_transport_headers`]: super::config::SignalConfig::partition_by_transport_headers
/// [`PartitionerStrategy`]: super::config::PartitionerStrategy
#[must_use]
pub fn partition_key_for_signal(
    headers: Option<&HeadersRead>,
    metadata: &Metadata,
) -> Option<String> {
    partition_key_from_transport_headers(headers?.read(metadata))
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
    use crate::common::kafka::MessageFormat;
    use crate::exporters::kafka_exporter::config::SignalConfig;
    use crate::exporters::kafka_exporter::exporter::test_support::{
        exporter_context, kafka_test_config_builder, pipeline_context,
    };
    use otel_arrow_dfe_config::context::transport_headers::{
        TransportHeader, TransportHeaders, ValueKind,
    };

    fn context_name(raw: &str) -> otel_arrow_dfe_config::ContextEntryName {
        raw.try_into().expect("valid test context entry name")
    }

    fn transport_header_text(
        stored_name: &str,
        wire_name: &str,
        value: impl Into<Vec<u8>>,
    ) -> TransportHeader {
        TransportHeader::captured(
            context_name(stored_name),
            wire_name,
            true,
            ValueKind::Text,
            value.into(),
        )
    }

    fn transport_header_binary(
        stored_name: &str,
        wire_name: &str,
        value: impl Into<Vec<u8>>,
    ) -> TransportHeader {
        TransportHeader::captured(
            context_name(stored_name),
            wire_name,
            true,
            ValueKind::Binary,
            value.into(),
        )
    }

    fn key_of(headers: &TransportHeaders) -> Option<String> {
        partition_key_from_transport_headers(headers.iter().map(HeaderValue::from))
    }

    /// Returns the all-headers read the compiler binds for a signal that
    /// partitions by transport headers.
    fn headers_read() -> HeadersRead {
        let config = kafka_test_config_builder("unused:9092")
            .with_logs(
                SignalConfig::new("logs".into(), MessageFormat::OtlpProto)
                    .with_partition_by_transport_headers(true),
            )
            .try_into()
            .expect("partitioning test config");
        exporter_context(pipeline_context(), &config)
            .node_bindings()
            .and_then(|bindings| bindings.headers().cloned())
            .expect("headers read")
    }

    // ---- Transport header partition key tests ----

    #[test]
    fn test_empty_transport_headers_returns_none() {
        let headers = TransportHeaders::new();
        let key = key_of(&headers);
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

        let key1 = key_of(&headers);
        let key2 = key_of(&headers);

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

        let key1 = key_of(&headers1);
        let key2 = key_of(&headers2);

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

        let key1 = key_of(&headers1);
        let key2 = key_of(&headers2);

        assert_ne!(key1, key2);
    }

    /// Scenario: explicit stored header names differ only by ASCII case.
    /// Guarantees: partition keys preserve the configured stored-name identity.
    #[test]
    fn transport_headers_stored_name_case_affects_partition_key() {
        let mut mixed_case = TransportHeaders::new();
        mixed_case.push(transport_header_text("Tenant", "X-Tenant-Id", b"tenant-a"));

        let mut lowercase = TransportHeaders::new();
        lowercase.push(transport_header_text("tenant", "X-Tenant-Id", b"tenant-a"));

        assert_ne!(key_of(&mixed_case), key_of(&lowercase));
    }

    #[test]
    fn test_transport_headers_key_is_fixed_size_hex() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key = key_of(&headers).expect("non-empty headers should produce a key");

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

        let key = key_of(&headers).expect("non-empty headers should produce a key");

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

        let key_ab = key_of(&headers_ab);
        let key_ba = key_of(&headers_ba);

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

        let key = key_of(&headers).expect("non-empty headers should produce a key");

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

    use otel_arrow_dfe_engine::testing::context::headers_metadata;
    use otel_arrow_dfe_otap::pdata::Context;

    /// Scenario: a context carries captured headers but the signal does not
    /// partition by transport headers (no compiled headers read).
    /// Guarantees: no partition key is derived from the captured headers.
    #[test]
    fn test_partition_key_for_signal_disabled_returns_none() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text("x_tenant_id", "X-Tenant-Id", b"t"));
        let metadata = headers_metadata(&headers);
        let key = partition_key_for_signal(None, &metadata);
        assert!(key.is_none());
    }

    #[test]
    fn test_partition_key_for_signal_enabled_no_headers_returns_none() {
        let context = Context::default();
        let key = partition_key_for_signal(Some(&headers_read()), context.metadata());
        assert!(key.is_none());
    }

    /// Scenario: a signal partitions by transport headers and the context
    /// carries a captured header.
    /// Guarantees: the key derived through the compiled headers read equals
    /// the key computed directly from the same headers.
    #[test]
    fn test_partition_key_for_signal_enabled_with_headers() {
        let mut headers = TransportHeaders::new();
        headers.push(transport_header_text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let metadata = headers_metadata(&headers);
        let key = partition_key_for_signal(Some(&headers_read()), &metadata);

        // Should match what partition_key_from_transport_headers produces.
        let expected = key_of(&headers);
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

        let key = key_of(&headers).expect("non-empty headers produce a key");

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
        let k1 = key_of(&headers).expect("key");
        let k2 = key_of(&headers).expect("key");
        assert_eq!(k1, k2, "same headers must produce the same (stable) key");
        assert!(!k1.contains("secret"), "key must not be plaintext: {k1}");
    }
}
