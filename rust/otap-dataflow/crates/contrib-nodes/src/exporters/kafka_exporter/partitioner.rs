// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Partition key generation for Kafka messages.
//!
//! The request's packed tenant context is hashed into a fixed-size,
//! hex-encoded key that librdkafka's partitioner algorithm (configured via
//! [`PartitionerStrategy`]) then maps to a concrete partition number.
//!
//! [`PartitionerStrategy`]: super::config::PartitionerStrategy

use otap_df_otap::pdata::Context;
use xxhash_rust::xxh64::Xxh64;

/// Build a deterministic partition key from a packed tenant context.
///
/// Requests carrying the same tenant values are routed to the same partition,
/// which is what lets a downstream consumer see one tenant's data in order.
///
/// The packed layout is positional: a value's slot is fixed by the registry,
/// not by the order headers happened to arrive in, and the layout digest is
/// folded into the epoch so two registries cannot produce colliding shapes.
/// Equal tenant contexts are therefore byte-equal, and the sort-and-normalize
/// step the transport-header version needed to make its key order-independent
/// has no analogue here -- along with the `Vec` it allocated per request.
///
/// Words are folded in big-endian so the key does not depend on the producer's
/// architecture; two producers on different hosts must agree on the partition.
///
/// # Returns
/// A hex-encoded 16-character key, or `None` when the context carries nothing.
#[must_use]
pub fn partition_key_from_tenant(words: &[u64]) -> Option<String> {
    if words.is_empty() {
        return None;
    }
    let mut hasher = Xxh64::new(0);
    for word in words {
        hasher.update(&word.to_be_bytes());
    }
    Some(hex::encode(hasher.digest().to_be_bytes()))
}

/// Determine the partition key for a signal based on its per-signal config and
/// the pdata context.
///
/// Returns `None` when partitioning by tenant context is not enabled or the
/// request carries no tenant context, which leaves the Kafka key unset (null
/// key) and gives true round-robin partitioning under all
/// [`PartitionerStrategy`] variants.
///
/// [`PartitionerStrategy`]: super::config::PartitionerStrategy
#[must_use]
pub fn partition_key_for_signal(
    signal_config: &super::config::SignalConfig,
    context: &Context,
) -> Option<String> {
    if !signal_config.partition_by_tenant_context() {
        return None;
    }
    partition_key_from_tenant(context.tenant()?)
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

    /// Scenario: the same packed tenant context is hashed twice, and a context
    /// differing by a single word is hashed once.
    /// Guarantees: equal contexts produce the same 16-character key so a
    /// tenant's data lands on one partition, and a different context produces
    /// a different key so tenants are not silently merged onto one partition.
    #[test]
    fn key_is_deterministic_and_distinguishing() {
        let a = [0x0102_0304_0506_0708u64, 0x1112_1314_1516_1718];
        let b = [0x0102_0304_0506_0708u64, 0x1112_1314_1516_1719];

        let key_a = partition_key_from_tenant(&a).expect("non-empty");
        assert_eq!(key_a.len(), 16);
        assert_eq!(partition_key_from_tenant(&a), Some(key_a.clone()));
        assert_ne!(partition_key_from_tenant(&b), Some(key_a));
    }

    /// Scenario: a request arrives with no tenant context at all.
    /// Guarantees: no key is produced, so the Kafka key stays null and the
    /// record round-robins rather than piling every context-less request onto
    /// one partition.
    #[test]
    fn empty_context_has_no_key() {
        assert!(partition_key_from_tenant(&[]).is_none());
    }
}
