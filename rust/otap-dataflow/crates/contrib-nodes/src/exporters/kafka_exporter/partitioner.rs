// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Partition key generation for Kafka messages.
//!
//! This module provides functions to generate deterministic partition keys for
//! Kafka messages based on transport headers. The generated keys are
//! length-prefixed byte buffers that librdkafka's partitioner algorithm
//! (configured via [`PartitionerStrategy`]) hashes to select a concrete
//! partition number.
//!
//! [`PartitionerStrategy`]: super::config::PartitionerStrategy

use otap_df_otap::transport_headers::TransportHeaders;

/// Build a deterministic partition key from transport headers.
///
/// All headers from the pdata context are serialized (by transport header name
/// and raw value) into a length-prefixed byte buffer. This ensures that requests
/// carrying the same set of transport headers (e.g., same tenant ID, same auth
/// token) produce the same key and are therefore routed to the same Kafka
/// partition by librdkafka's partitioner.
///
/// Using the transport header name means that
/// headers differing only in casing or formatting (e.g. `X-Tenant-Id` vs
/// `x-tenant-id`) produce the same key.
///
/// - **Order-independent**: Headers are sorted by `(name, value)` before
///   serialization, so the same logical set of headers always produces the
///   same key regardless of insertion order.
/// - **Unambiguous**: Each field is length-prefixed (`u32` big-endian), so
///   there is no delimiter collision risk and distinct header sets always
///   produce distinct keys (zero collision risk).
/// - **Single hash**: The byte buffer is passed directly as the Kafka message
///   key. Only librdkafka hashes it (once) to select a partition — there is
///   no application-level pre-hash.
///
/// ## Wire format
///
/// ```text
/// For each header in sorted (name, value) order:
///   [4 bytes: name length (u32 BE)][name bytes][4 bytes: value length (u32 BE)][value bytes]
/// ```
///
/// # Arguments
/// * `headers` - Transport headers captured from the inbound request.
///
/// # Returns
/// A length-prefixed byte buffer suitable for use as a Kafka partition key.
/// Returns `None` if no headers are present, which leaves the Kafka key unset
/// (null key) and ensures true round-robin partitioning under all
/// [`PartitionerStrategy`] variants.
///
/// [`PartitionerStrategy`]: super::config::PartitionerStrategy
#[must_use]
pub fn partition_key_from_transport_headers(headers: &TransportHeaders) -> Option<Vec<u8>> {
    if headers.is_empty() {
        return None;
    }

    // Sort by (name, value) to ensure order-independent serialization.
    // TransportHeaders is backed by a Vec, so iteration order depends on
    // insertion order; sorting removes that dependency.
    // We use the normalized `name` (not `wire_name`) so that headers differing
    // only in original casing produce the same partition key.
    let mut sorted: Vec<_> = headers.iter().collect();
    sorted.sort_unstable_by(|a, b| a.name.cmp(&b.name).then_with(|| a.value.cmp(&b.value)));

    let mut buf = Vec::new();
    for header in &sorted {
        buf.extend_from_slice(&(header.name.len() as u32).to_be_bytes());
        buf.extend_from_slice(header.name.as_bytes());
        buf.extend_from_slice(&(header.value.len() as u32).to_be_bytes());
        buf.extend_from_slice(&header.value);
    }

    Some(buf)
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
    context: &otap_df_otap::pdata::Context,
) -> Option<Vec<u8>> {
    if signal_config.partition_by_transport_headers() {
        if let Some(headers) = context.transport_headers() {
            return partition_key_from_transport_headers(headers);
        }
    }

    None
}

// TODO: Explore `partition_by_trace_id` — partition traces by hex-encoded trace ID.
//   Trace IDs are 16-byte FixedSizeBinary values in the OTAP Arrow Spans schema.
//   A single OTAP batch can contain spans with different trace IDs, so implementing
//   this requires splitting the batch into sub-batches grouped by trace ID (returning
//   something like `Vec<(String, RoaringBitmap)>`) before sending each sub-batch to
//   Kafka with its own partition key. Empty/zero trace IDs should map to an empty
//   partition key (round-robin).

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_otap::transport_headers::TransportHeader;

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
        headers.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));
        headers.push(TransportHeader::text("x_region", "X-Region", b"us-east-1"));

        let key1 = partition_key_from_transport_headers(&headers);
        let key2 = partition_key_from_transport_headers(&headers);

        assert_eq!(key1, key2);
        assert!(key1.is_some());
    }

    #[test]
    fn test_transport_headers_different_values_produce_different_keys() {
        let mut headers1 = TransportHeaders::new();
        headers1.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-a",
        ));

        let mut headers2 = TransportHeaders::new();
        headers2.push(TransportHeader::text(
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
        headers1.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"value",
        ));

        let mut headers2 = TransportHeaders::new();
        headers2.push(TransportHeader::text("x_region", "X-Region", b"value"));

        let key1 = partition_key_from_transport_headers(&headers1);
        let key2 = partition_key_from_transport_headers(&headers2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_transport_headers_byte_format() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // Expected format:
        //   [4 bytes: name_len][name][4 bytes: value_len][value]
        //   name = "x_tenant_id" (11 bytes)
        //   value = "tenant-123" (10 bytes)
        //   total = 4 + 11 + 4 + 10 = 29 bytes
        assert_eq!(key.len(), 29);

        // Verify name length prefix (big-endian u32 = 11)
        assert_eq!(&key[0..4], &11_u32.to_be_bytes());
        // Verify name bytes
        assert_eq!(&key[4..15], b"x_tenant_id");
        // Verify value length prefix (big-endian u32 = 10)
        assert_eq!(&key[15..19], &10_u32.to_be_bytes());
        // Verify value bytes
        assert_eq!(&key[19..29], b"tenant-123");
    }

    #[test]
    fn test_transport_headers_binary_values_in_key() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::binary(
            "x_binary",
            "X-Binary",
            [0x01, 0x02, 0x03, 0xFF],
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // name = "x_binary" (8 bytes), value = 4 bytes
        // total = 4 + 8 + 4 + 4 = 20 bytes
        assert_eq!(key.len(), 20);

        // Binary value should appear directly in the key (not hashed)
        assert_eq!(&key[16..20], &[0x01, 0x02, 0x03, 0xFF]);
    }

    /// Transport headers pushed in different order must produce the same key.
    #[test]
    fn test_transport_headers_order_independent() {
        let mut headers_ab = TransportHeaders::new();
        headers_ab.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));
        headers_ab.push(TransportHeader::text("x_region", "X-Region", b"us-east-1"));

        // Same headers, reversed insertion order.
        let mut headers_ba = TransportHeaders::new();
        headers_ba.push(TransportHeader::text("x_region", "X-Region", b"us-east-1"));
        headers_ba.push(TransportHeader::text(
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

    // ---- Golden-value regression test ----
    //
    // Pins the exact byte output for a known input. If the value changes
    // it means the serialization format changed, which would cause a Kafka
    // partition rebalance. Investigate before updating.

    #[test]
    fn test_transport_headers_golden_value() {
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(
            "x_tenant_id",
            "X-Tenant-Id",
            b"tenant-123",
        ));

        let key = partition_key_from_transport_headers(&headers)
            .expect("non-empty headers should produce a key");

        // name = "x_tenant_id" (len 11), value = "tenant-123" (len 10)
        let mut expected = Vec::new();
        expected.extend_from_slice(&11_u32.to_be_bytes());
        expected.extend_from_slice(b"x_tenant_id");
        expected.extend_from_slice(&10_u32.to_be_bytes());
        expected.extend_from_slice(b"tenant-123");

        assert_eq!(
            key, expected,
            "golden value changed — this will cause a Kafka partition rebalance"
        );
    }

    // ---- partition_key_for_signal tests ----

    use crate::common::kafka::MessageFormat;
    use crate::exporters::kafka_exporter::config::SignalConfig;
    use otap_df_otap::pdata::Context;

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
        headers.push(TransportHeader::text(
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
}
