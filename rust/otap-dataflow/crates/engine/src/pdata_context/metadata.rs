// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Message metadata carried by pdata: request-scoped [`Metadata`], verified
//! identity storage, the value source used to materialize composites, and
//! peer-address merging.
//!
//! Delivery state (Ack/Nack frames, flow metrics, signal) belongs to the pdata
//! type that carries this metadata.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use super::Claims;
use crate::capability::auth::ClaimValue;
use crate::pdata_context::{
    ContextDomain, ContextFieldLayout, ContextMaterializationPlan, ContextProjection,
    ContextValueCardinality, ContextValueKind, ContextValueSource, MaterializedContext,
    MaterializedContextProjection, MaterializedContextValue, MaterializedContextValues,
};
use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeaders, TransportHeadersData};

/// Request-scoped message metadata.
///
/// Holds what a message carries about its request, as opposed to delivery
/// state (Ack/Nack frames, flow metrics, signal):
///
/// - **Transport headers**: Protocol-neutral metadata captured from inbound
///   transport headers.
/// - **Authorized identity**: Verified claims selected by policy and kept
///   separate from untrusted transport headers.
/// - **Materialized entries**: Selected composite snapshots whose values are
///   fixed for the message's lifetime.
/// - **Peer address**: Optional socket address observed by the receiving
///   socket at request acceptance time. Populated by receivers that have a
///   real socket (OTLP gRPC/HTTP, OTAP gRPC, syslog/CEF) and left `None` by
///   sourceless receivers (file-based, journald).
///
/// All of it is preserved across transport boundaries and copied to split
/// outputs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metadata {
    /// Verified authorization claims selected by policy.
    authorized_identity: AuthorizedIdentityEntries,
    /// Captured transport headers and selected materialized composites.
    materialized: MaterializedContext,
    /// Peer address observed by the receiving socket at request acceptance
    /// time. `None` for receivers without a real socket.
    peer_addr: Option<SocketAddr>,
}

impl Metadata {
    /// Returns a reference to the captured transport headers, if any.
    #[must_use]
    pub fn transport_headers(&self) -> Option<&TransportHeadersData> {
        self.materialized.transport_headers()
    }

    /// Takes and returns the captured transport headers, if any.
    #[must_use]
    pub fn take_transport_headers(&mut self) -> Option<TransportHeaders> {
        self.materialized.take_transport_headers()
    }

    /// Sets the captured transport headers.
    pub fn set_transport_headers(&mut self, headers: TransportHeaders) {
        self.materialized.set_transport_headers(headers);
    }

    /// Returns the authorization-derived context entries, if any.
    #[must_use]
    pub fn authorized_identity_entries(&self) -> Option<&AuthorizedIdentityEntries> {
        (!self.authorized_identity.is_empty()).then_some(&self.authorized_identity)
    }

    /// Replaces the authorized identity entries with the claims `policy`
    /// selects from a verified identity.
    pub fn capture_authorized_identity(
        &mut self,
        policy: &AuthorizedIdentityPolicy,
        identity: &(impl Claims + ?Sized),
    ) {
        self.authorized_identity =
            AuthorizedIdentityEntries::capture(policy, identity).unwrap_or_default();
    }

    /// Materializes selected composite snapshots exactly once.
    pub fn materialize_context(&mut self, plan: &ContextMaterializationPlan) {
        let source = PdataContextValueSource {
            authorized_identity: &self.authorized_identity,
        };
        self.materialized
            .materialize_with_stored_transport(plan, &source);
    }

    /// Returns the materialized composite snapshots, if any.
    #[must_use]
    pub fn materialized_context(&self) -> Option<&MaterializedContext> {
        (!self.materialized.is_empty()).then_some(&self.materialized)
    }

    /// Resolves an existing materialized composite projection.
    #[must_use]
    pub fn resolve_materialized_context<'a>(
        &'a self,
        plan: &'a ContextMaterializationPlan,
        projection: &'a ContextProjection,
    ) -> Option<MaterializedContextProjection<'a>> {
        self.materialized.resolve(plan, projection)
    }

    /// Returns the peer address observed by the receiving socket, if any.
    #[must_use]
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }

    /// Sets the peer address.
    pub fn set_peer_addr(&mut self, addr: SocketAddr) {
        self.peer_addr = Some(addr);
    }
}

const AUTHORIZED_ENTRY_LEN: usize = 20;
const AUTHORIZED_VALUE_LEN: usize = 8;

/// A verified authorization claim stored under a configured context entry name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AuthorizedIdentityEntry<'a> {
    name: &'a str,
    value: AuthorizedClaimValue<'a>,
}

impl fmt::Debug for AuthorizedIdentityEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedIdentityEntry")
            .field("name", &self.name)
            .field("value_count", &self.value.len())
            .finish()
    }
}

impl<'a> AuthorizedIdentityEntry<'a> {
    /// Returns the configured context entry name.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the verified claim value without flattening its cardinality.
    #[must_use]
    pub const fn value(&self) -> AuthorizedClaimValue<'a> {
        self.value
    }
}

/// Borrowed single- or multi-valued authorized claim.
#[derive(Clone, Copy)]
pub struct AuthorizedClaimValue<'a> {
    storage: &'a PackedAuthorizedIdentity,
    first_value: usize,
    value_count: usize,
    many: bool,
}

impl PartialEq for AuthorizedClaimValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.many == other.many
            && self.value_count == other.value_count
            && self.values().eq(other.values())
    }
}

impl Eq for AuthorizedClaimValue<'_> {}

impl fmt::Debug for AuthorizedClaimValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedClaimValue")
            .field("value_count", &self.value_count)
            .field("many", &self.many)
            .finish()
    }
}

impl<'a> AuthorizedClaimValue<'a> {
    /// Returns the single value, or `None` when the source claim was multi-valued.
    #[must_use]
    pub fn as_str(&self) -> Option<&'a str> {
        (!self.many && self.value_count == 1).then(|| self.storage.decode_value(self.first_value))
    }

    /// Iterates values in source order.
    pub fn values(&self) -> impl Iterator<Item = &'a str> + '_ {
        (0..self.value_count)
            .map(move |offset| self.storage.decode_value(self.first_value + offset))
    }

    /// Returns the number of values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.value_count
    }

    /// Returns whether the claim contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.value_count == 0
    }

    /// Returns whether the source claim used multi-valued cardinality.
    #[must_use]
    pub const fn is_many(&self) -> bool {
        self.many
    }
}

#[derive(PartialEq, Eq)]
struct PackedAuthorizedIdentity {
    bytes: Box<[u8]>,
    entry_count: usize,
    value_count: usize,
}

/// Immutable authorization-derived context entries.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct AuthorizedIdentityEntries {
    packed: Option<Arc<PackedAuthorizedIdentity>>,
}

struct AuthorizedIdentityEntriesDebug<'a>(&'a AuthorizedIdentityEntries);

impl fmt::Debug for AuthorizedIdentityEntriesDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl fmt::Debug for AuthorizedIdentityEntries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorizedIdentityEntries")
            .field("entries", &AuthorizedIdentityEntriesDebug(self))
            .finish()
    }
}

impl AuthorizedIdentityEntries {
    fn capture(
        policy: &AuthorizedIdentityPolicy,
        identity: &(impl Claims + ?Sized),
    ) -> Option<Self> {
        PackedAuthorizedIdentity::capture(policy, identity).map(|packed| Self {
            packed: Some(Arc::new(packed)),
        })
    }

    /// Returns the number of captured entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.packed.as_ref().map_or(0, |packed| packed.entry_count)
    }

    /// Returns whether no entries were captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates over captured entries in policy order.
    pub fn iter(&self) -> impl Iterator<Item = AuthorizedIdentityEntry<'_>> {
        self.packed
            .as_deref()
            .into_iter()
            .flat_map(|packed| (0..packed.entry_count).map(|index| packed.decode_entry(index)))
    }

    /// Finds an entry by exact configured name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<AuthorizedIdentityEntry<'_>> {
        self.iter().find(|entry| entry.name == name)
    }
}

impl PackedAuthorizedIdentity {
    fn capture(
        policy: &AuthorizedIdentityPolicy,
        identity: &(impl Claims + ?Sized),
    ) -> Option<Self> {
        let mut entry_count = 0usize;
        let mut value_count = 0usize;
        let mut blob_len = 0usize;
        for projection in policy.iter() {
            let Some(claim) = identity.claim(&projection.claim) else {
                continue;
            };
            entry_count = entry_count
                .checked_add(1)
                .expect("authorized identity entry count overflow");
            value_count = value_count
                .checked_add(claim.as_slice().len())
                .expect("authorized identity value count overflow");
            blob_len = claim
                .as_slice()
                .iter()
                .try_fold(
                    blob_len
                        .checked_add(projection.store_as.as_str().len())
                        .expect("authorized identity blob length overflow"),
                    |total, value| total.checked_add(value.len()),
                )
                .expect("authorized identity blob length overflow");
        }
        if entry_count == 0 {
            return None;
        }
        let entry_bytes = entry_count
            .checked_mul(AUTHORIZED_ENTRY_LEN)
            .expect("authorized identity entry descriptor length overflow");
        let value_bytes = value_count
            .checked_mul(AUTHORIZED_VALUE_LEN)
            .expect("authorized identity value descriptor length overflow");
        let descriptor_len = entry_bytes
            .checked_add(value_bytes)
            .expect("authorized identity descriptor length overflow");
        let mut bytes = vec![
            0;
            descriptor_len
                .checked_add(blob_len)
                .expect("authorized identity packed length overflow")
        ];
        let mut blob_at = descriptor_len;
        let mut value_index = 0;

        let mut entry_index = 0;
        for projection in policy.iter() {
            let Some(claim) = identity.claim(&projection.claim) else {
                continue;
            };
            let entry_at = entry_index * AUTHORIZED_ENTRY_LEN;
            let name_range = write_context_blob(
                &mut bytes,
                &mut blob_at,
                projection.store_as.as_str().as_bytes(),
            )
            .expect("preallocated authorized identity name range");
            write_context_range(&mut bytes, entry_at, name_range)
                .expect("authorized identity name range fits descriptor");
            write_context_u32(&mut bytes, entry_at + 8, value_index)
                .expect("authorized identity first value fits descriptor");
            write_context_u32(&mut bytes, entry_at + 12, claim.as_slice().len())
                .expect("authorized identity value count fits descriptor");
            bytes[entry_at + 16] = u8::from(matches!(claim, ClaimValue::Many(_)));

            for value in claim.as_slice() {
                let range = write_context_blob(&mut bytes, &mut blob_at, value.as_bytes())
                    .expect("preallocated authorized identity value range");
                let value_at = entry_bytes + value_index * AUTHORIZED_VALUE_LEN;
                write_context_range(&mut bytes, value_at, range)
                    .expect("authorized identity value range fits descriptor");
                value_index += 1;
            }
            entry_index += 1;
        }

        Some(Self {
            bytes: bytes.into_boxed_slice(),
            entry_count,
            value_count,
        })
    }

    fn decode_entry(&self, index: usize) -> AuthorizedIdentityEntry<'_> {
        debug_assert!(
            index < self.entry_count,
            "packed authorized identity index must be in bounds"
        );
        self.try_decode_entry(index)
            .expect("in-bounds packed authorized identity entry must decode")
    }

    fn try_decode_entry(&self, index: usize) -> Option<AuthorizedIdentityEntry<'_>> {
        let at = index.checked_mul(AUTHORIZED_ENTRY_LEN)?;
        let name = read_context_str(&self.bytes, read_context_range(&self.bytes, at)?)?;
        let first_value = read_context_u32(&self.bytes, at + 8)?;
        let value_count = read_context_u32(&self.bytes, at + 12)?;
        let _ = first_value
            .checked_add(value_count)
            .filter(|end| *end <= self.value_count)?;
        let many = match *self.bytes.get(at + 16)? {
            0 => false,
            1 => true,
            _ => return None,
        };
        Some(AuthorizedIdentityEntry {
            name,
            value: AuthorizedClaimValue {
                storage: self,
                first_value,
                value_count,
                many,
            },
        })
    }

    fn value(&self, index: usize) -> Option<&str> {
        if index >= self.value_count {
            return None;
        }
        let values_at = self.entry_count.checked_mul(AUTHORIZED_ENTRY_LEN)?;
        let at = values_at.checked_add(index.checked_mul(AUTHORIZED_VALUE_LEN)?)?;
        read_context_str(&self.bytes, read_context_range(&self.bytes, at)?)
    }

    fn decode_value(&self, index: usize) -> &str {
        debug_assert!(
            index < self.value_count,
            "packed authorized identity value index must be in bounds"
        );
        self.value(index)
            .expect("in-bounds packed authorized identity value must decode")
    }
}

fn write_context_blob(bytes: &mut [u8], at: &mut usize, value: &[u8]) -> Option<(usize, usize)> {
    let start = *at;
    let end = start.checked_add(value.len())?;
    bytes.get_mut(start..end)?.copy_from_slice(value);
    *at = end;
    Some((start, value.len()))
}

fn write_context_range(bytes: &mut [u8], at: usize, range: (usize, usize)) -> Option<()> {
    write_context_u32(bytes, at, range.0)?;
    write_context_u32(bytes, at + 4, range.1)
}

fn write_context_u32(bytes: &mut [u8], at: usize, value: usize) -> Option<()> {
    bytes
        .get_mut(at..at.checked_add(4)?)?
        .copy_from_slice(&u32::try_from(value).ok()?.to_le_bytes());
    Some(())
}

fn read_context_range(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    Some((
        read_context_u32(bytes, at)?,
        read_context_u32(bytes, at + 4)?,
    ))
}

fn read_context_u32(bytes: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)
}

fn read_context_str(bytes: &[u8], range: (usize, usize)) -> Option<&str> {
    std::str::from_utf8(bytes.get(range.0..range.0.checked_add(range.1)?)?).ok()
}

struct PdataContextValueSource<'a> {
    authorized_identity: &'a AuthorizedIdentityEntries,
}

impl ContextValueSource for PdataContextValueSource<'_> {
    fn values(&self, field: &ContextFieldLayout) -> Option<MaterializedContextValues> {
        match field.domain {
            ContextDomain::TransportHeader => None,
            ContextDomain::AuthorizedIdentity => {
                let entry = self.authorized_identity.get(field.name.as_str())?;
                let claim = entry.value();
                let cardinality = if claim.is_many() {
                    ContextValueCardinality::Many
                } else {
                    ContextValueCardinality::One
                };
                let values = claim
                    .values()
                    .map(|value| {
                        MaterializedContextValue::new(
                            None,
                            ContextValueKind::Text,
                            value.as_bytes().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>();
                MaterializedContextValues::new(cardinality, values.into_boxed_slice())
            }
        }
    }
}

/// Incremental merge of the peer addresses of contributing inputs, without
/// allocating a `Vec` of intermediate peer addresses.
///
/// The merged result is `Some(addr)` only when every contributing input
/// observed the same peer address; it is `None` when no inputs were pushed,
/// any input was peerless, or distinct peers contributed.
///
/// Use when contributing inputs are observed one-at-a-time as part of an
/// existing pass over the input list (e.g. a processor's flush loop). Push
/// each input's `peer_addr` exactly once with [`PeerAddrMerger::push`], then
/// call [`PeerAddrMerger::finish`] to obtain the merged result.
///
/// Modeled as an enum so illegal states (e.g. a peer address present
/// alongside a poisoned flag) cannot be constructed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PeerAddrMerger {
    /// No inputs pushed yet.
    #[default]
    Empty,
    /// Every input pushed so far agreed on this address.
    Single(SocketAddr),
    /// At least one input was peerless or distinct from a previously agreed
    /// address. The merge stays in this state regardless of later pushes.
    Poisoned,
}

impl PeerAddrMerger {
    /// Creates an empty merger. `finish` on an unused merger returns `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self::Empty
    }

    /// Folds one contributing input's `peer_addr` into the running merge.
    pub fn push(&mut self, peer_addr: Option<SocketAddr>) {
        *self = match (*self, peer_addr) {
            (Self::Empty, None) => Self::Poisoned,
            (Self::Empty, Some(addr)) => Self::Single(addr),
            (Self::Single(_), None) => Self::Poisoned,
            (Self::Single(existing), Some(addr)) if existing == addr => Self::Single(existing),
            (Self::Single(_), Some(_)) => Self::Poisoned,
            (Self::Poisoned, _) => Self::Poisoned,
        };
    }

    /// Returns the merged peer address, or `None` if no inputs were pushed or
    /// the merge was poisoned by a peerless or distinct contributor.
    #[must_use]
    pub const fn finish(self) -> Option<SocketAddr> {
        match self {
            Self::Single(addr) => Some(addr),
            Self::Empty | Self::Poisoned => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::auth::AuthorizedIdentity;

    /// Scenario: default metadata is given a peer address.
    /// Guarantees: the peer address starts absent and reads back as set.
    #[test]
    fn metadata_peer_addr_round_trips() {
        let mut metadata = Metadata::default();
        assert_eq!(metadata.peer_addr(), None);

        let addr: SocketAddr = "10.0.0.1:5005".parse().expect("valid address");
        metadata.set_peer_addr(addr);
        assert_eq!(metadata.peer_addr(), Some(addr));
    }

    /// Scenario: an identity contains one selected multi-valued claim while
    /// another configured claim is absent.
    /// Guarantees: claim cardinality is preserved and missing claims do not
    /// create empty context entries.
    #[test]
    fn authorized_identity_capture_preserves_many_and_omits_missing() {
        let policy: AuthorizedIdentityPolicy = serde_json::from_value(serde_json::json!([
            {"claim": "groups", "store_as": "access_groups"},
            {"claim": "missing", "store_as": "missing_entry"}
        ]))
        .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new().with_claim_values("groups", ["reader", "writer"]);
        let mut metadata = Metadata::default();

        metadata.capture_authorized_identity(&policy, &identity);

        let entries = metadata
            .authorized_identity_entries()
            .expect("selected claim captured");
        assert_eq!(entries.len(), 1);
        let groups = entries
            .get("access_groups")
            .expect("groups destination exists")
            .value();
        assert_eq!(groups.values().collect::<Vec<_>>(), ["reader", "writer"]);
        assert!(groups.is_many());
        assert!(entries.get("missing_entry").is_none());
    }

    /// Scenario: one claim is encoded as `One` and another as one-element `Many`.
    /// Guarantees: packed storage preserves cardinality instead of inferring it from count.
    #[test]
    fn authorized_identity_capture_preserves_single_value_cardinality() {
        let policy: AuthorizedIdentityPolicy = serde_json::from_value(serde_json::json!([
            {"claim": "sub", "store_as": "subject"},
            {"claim": "groups", "store_as": "groups"}
        ]))
        .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new()
            .with_subject("reader")
            .with_claim_values("groups", ["reader"]);
        let mut metadata = Metadata::default();

        metadata.capture_authorized_identity(&policy, &identity);

        let entries = metadata
            .authorized_identity_entries()
            .expect("selected claims captured");
        let subject = entries.get("subject").expect("subject entry").value();
        let groups = entries.get("groups").expect("groups entry").value();
        assert_eq!(subject.as_str(), Some("reader"));
        assert!(!subject.is_many());
        assert_eq!(groups.as_str(), None);
        assert!(groups.is_many());
        assert_eq!(groups.values().collect::<Vec<_>>(), ["reader"]);
    }

    /// Scenario: equal claims occupy different offsets in identities with different unrelated data.
    /// Guarantees: claim and entry equality use cardinality and values, not packed storage identity.
    #[test]
    fn authorized_claim_equality_is_value_based() {
        let subject_first: AuthorizedIdentityPolicy = serde_json::from_value(serde_json::json!([
            {"claim": "sub", "store_as": "subject"},
            {"claim": "groups", "store_as": "groups"}
        ]))
        .expect("valid authorized identity policy");
        let groups_first: AuthorizedIdentityPolicy = serde_json::from_value(serde_json::json!([
            {"claim": "groups", "store_as": "groups"},
            {"claim": "sub", "store_as": "subject"}
        ]))
        .expect("valid authorized identity policy");
        let left_identity = AuthorizedIdentity::new()
            .with_subject("reader")
            .with_claim_values("groups", ["left-group"]);
        let right_identity = AuthorizedIdentity::new()
            .with_subject("reader")
            .with_claim_values("groups", ["right-group"]);

        let left = AuthorizedIdentityEntries::capture(&subject_first, &left_identity)
            .expect("left identity captured");
        let right = AuthorizedIdentityEntries::capture(&groups_first, &right_identity)
            .expect("right identity captured");
        let left_subject = left.get("subject").expect("left subject");
        let right_subject = right.get("subject").expect("right subject");

        assert_eq!(left_subject.value(), right_subject.value());
        assert_eq!(left_subject, right_subject);

        let one_reader = left_subject.value();
        let many_reader_identity =
            AuthorizedIdentity::new().with_claim_values("groups", ["reader"]);
        let many_reader_entries =
            AuthorizedIdentityEntries::capture(&subject_first, &many_reader_identity)
                .expect("multi-valued identity captured");
        let many_reader = many_reader_entries
            .get("groups")
            .expect("groups entry")
            .value();
        assert_ne!(one_reader, many_reader);
    }

    /// Scenario: a packed authorized identity descriptor is corrupted below its declared count.
    /// Guarantees: iteration detects the violated decode invariant instead of omitting it silently.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "in-bounds packed authorized identity entry must decode")]
    fn authorized_identity_decode_failure_fails_fast() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new().with_subject("reader");
        let mut entries =
            AuthorizedIdentityEntries::capture(&policy, &identity).expect("captured identity");
        let packed = Arc::get_mut(entries.packed.as_mut().expect("packed storage is present"))
            .expect("packed storage is uniquely owned");
        packed.bytes[0..4].copy_from_slice(&u32::MAX.to_le_bytes());

        let _ = entries.iter().next();
    }

    /// Scenario: a packed authorized identity descriptor has an invalid cardinality flag.
    /// Guarantees: decoding rejects values outside the encoded single-or-many domain.
    #[test]
    #[should_panic(expected = "in-bounds packed authorized identity entry must decode")]
    fn authorized_identity_invalid_cardinality_fails_fast() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new().with_subject("reader");
        let mut entries =
            AuthorizedIdentityEntries::capture(&policy, &identity).expect("captured identity");
        let packed = Arc::get_mut(entries.packed.as_mut().expect("packed storage is present"))
            .expect("packed storage is uniquely owned");
        packed.bytes[16] = 2;

        let _ = entries.iter().next();
    }

    fn corrupt_first_authorized_identity_value(entries: &mut AuthorizedIdentityEntries) {
        let packed = Arc::get_mut(entries.packed.as_mut().expect("packed storage is present"))
            .expect("packed storage is uniquely owned");
        let value_at = packed.entry_count * AUTHORIZED_ENTRY_LEN;
        packed.bytes[value_at..value_at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    }

    /// Scenario: a single-valued authorized claim has a corrupted value descriptor.
    /// Guarantees: scalar access detects the violated decode invariant instead of returning `None`.
    #[test]
    #[should_panic(expected = "in-bounds packed authorized identity value must decode")]
    fn authorized_identity_scalar_value_decode_failure_fails_fast() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new().with_subject("reader");
        let mut entries =
            AuthorizedIdentityEntries::capture(&policy, &identity).expect("captured identity");
        corrupt_first_authorized_identity_value(&mut entries);

        let value = entries.get("subject").expect("subject entry").value();
        let _ = value.as_str();
    }

    /// Scenario: a multi-valued authorized claim has a corrupted value descriptor.
    /// Guarantees: iteration detects the violated decode invariant instead of omitting the value.
    #[test]
    #[should_panic(expected = "in-bounds packed authorized identity value must decode")]
    fn authorized_identity_iterated_value_decode_failure_fails_fast() {
        let policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "groups", "store_as": "groups"}]))
                .expect("valid authorized identity policy");
        let identity = AuthorizedIdentity::new().with_claim_values("groups", ["reader", "writer"]);
        let mut entries =
            AuthorizedIdentityEntries::capture(&policy, &identity).expect("captured identity");
        corrupt_first_authorized_identity_value(&mut entries);

        let value = entries.get("groups").expect("groups entry").value();
        let _ = value.values().next();
    }

    /// Scenario: peer addresses from contributing inputs are pushed into a
    /// merger in various combinations, including after a conflict.
    /// Guarantees: the merge yields an address only when every input agreed
    /// on it, and a peerless or distinct input poisons the merge for good.
    #[test]
    fn peer_addr_merger_requires_unanimous_peer() {
        let a: SocketAddr = "10.0.0.1:1".parse().unwrap();
        let b: SocketAddr = "10.0.0.2:2".parse().unwrap();

        // Empty.
        let m = PeerAddrMerger::new();
        assert_eq!(m.finish(), None);

        // Single Some.
        let mut m = PeerAddrMerger::new();
        m.push(Some(a));
        assert_eq!(m.finish(), Some(a));

        // All identical.
        let mut m = PeerAddrMerger::new();
        m.push(Some(a));
        m.push(Some(a));
        m.push(Some(a));
        assert_eq!(m.finish(), Some(a));

        // Any None poisons subsequent pushes.
        let mut m = PeerAddrMerger::new();
        m.push(Some(a));
        m.push(None);
        m.push(Some(a));
        assert_eq!(m.finish(), None);

        // Distinct Somes.
        let mut m = PeerAddrMerger::new();
        m.push(Some(a));
        m.push(Some(b));
        assert_eq!(m.finish(), None);

        // After conflict, further identical pushes do not "heal" the result.
        let mut m = PeerAddrMerger::new();
        m.push(Some(a));
        m.push(Some(b));
        m.push(Some(a));
        assert_eq!(m.finish(), None);
    }
}
