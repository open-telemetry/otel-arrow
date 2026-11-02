// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bindings that derive output metadata from input metadata.
//!
//! Projection forms one output from one input plus values the node produces.
//! Merge forms one output from several inputs.

use super::{ContextDomain, ContextError, ContextLayoutIdentity, Metadata, PeerAddrMerger};
use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, ValueKind};
use otel_arrow_dfe_config::error::Error;

/// Compiled handle for one entry a node declared it produces.
///
/// Obtain once during node construction from [`ProjectionBinding::produced`].
/// A handle records the binding that issued it, so another binding rejects it
/// instead of writing a different entry.
#[derive(Debug, Clone)]
pub struct ProducedEntry {
    /// Identity of the issuing binding.
    binding: ContextLayoutIdentity,
    /// Position in the issuing binding's sorted entry list.
    index: usize,
}

impl PartialEq for ProducedEntry {
    fn eq(&self, other: &Self) -> bool {
        self.binding.matches(&other.binding) && self.index == other.index
    }
}

impl Eq for ProducedEntry {}

/// One value supplied for a produced entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducedValue<'a> {
    /// Produced entry handle issued by the node's projection binding.
    pub entry: &'a ProducedEntry,
    /// Text or binary representation.
    pub kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

/// Compiled projection for a node that produces context entries.
///
/// Equality is identity: clones of one compiled binding are equal and accept
/// each other's handles, while every other binding is not, even with the
/// same entries.
#[derive(Debug, Clone)]
pub struct ProjectionBinding {
    /// Identity shared by clones and by the handles this binding issues.
    ///
    /// The shared allocation is created once at compilation and cloned only
    /// when a node looks up handles during construction; `form` compares
    /// pointers and never touches the reference count.
    identity: ContextLayoutIdentity,
    /// Declared produced transport-header entries, sorted and unique.
    produced: Box<[ContextEntryName]>,
}

impl PartialEq for ProjectionBinding {
    fn eq(&self, other: &Self) -> bool {
        self.identity.matches(&other.identity)
    }
}

impl Eq for ProjectionBinding {}

impl ProjectionBinding {
    /// Compiles a projection for declared transport-header entries.
    pub(super) fn new(mut produced: Vec<ContextEntryName>) -> Self {
        produced.sort();
        produced.dedup();
        Self {
            identity: ContextLayoutIdentity::default(),
            produced: produced.into_boxed_slice(),
        }
    }

    /// Returns the handle for an entry this node declared it produces.
    ///
    /// Call once during node construction with the same domain and entry the
    /// node returned from its context declarations. Only transport-header
    /// entries can be declared as produced. Any other request is a mismatch
    /// between the node and its declarations and fails with
    /// [`Error::UndeclaredContextEntry`].
    pub fn produced(
        &self,
        domain: ContextDomain,
        entry: &ContextEntryName,
    ) -> Result<ProducedEntry, Error> {
        let undeclared = || Error::UndeclaredContextEntry {
            domain,
            entry: entry.clone(),
        };
        if domain != ContextDomain::TransportHeader {
            return Err(undeclared());
        }
        let index = self
            .produced
            .binary_search(entry)
            .map_err(|_| undeclared())?;
        Ok(ProducedEntry {
            binding: self.identity.clone(),
            index,
        })
    }

    /// Clones the input and appends each produced value as a transport header.
    ///
    /// Values are appended in slice order after any captured headers. Materialized
    /// composites, authorized identity, and peer address are retained unchanged,
    /// so produced values do not re-evaluate composites. Without produced values
    /// the output equals the input.
    ///
    /// Fails with [`ContextError::ContextMismatch`] when a value carries a
    /// handle issued by another binding.
    pub fn form(
        &self,
        input: &Metadata,
        produced: &[ProducedValue<'_>],
    ) -> Result<Metadata, ContextError> {
        let mut output = input.clone();
        if produced.is_empty() {
            return Ok(output);
        }
        let mut headers = output.take_transport_headers().unwrap_or_default();
        for value in produced {
            let name = value
                .entry
                .binding
                .matches(&self.identity)
                .then(|| self.produced.get(value.entry.index))
                .flatten()
                .ok_or(ContextError::ContextMismatch)?;
            headers.push(TransportHeader::new(name.clone(), value.kind, value.bytes));
        }
        output.set_transport_headers(headers);
        Ok(output)
    }
}

/// Compiled merge for a node that combines several inputs into one output.
///
/// Merge applies whenever an output stands for merged inputs, including a
/// merge of exactly one input, which keeps only that input's peer address.
/// An output that forwards one input's context unchanged is not a merge and
/// needs no binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeBinding {
    _compiled: (),
}

impl MergeBinding {
    pub(super) const fn new() -> Self {
        Self { _compiled: () }
    }

    /// Returns an empty accumulator for inputs observed one at a time.
    #[must_use]
    pub fn merger(&self) -> MetadataMerger {
        MetadataMerger {
            peer: PeerAddrMerger::new(),
        }
    }

    /// Merges every input into one output.
    ///
    /// Equivalent to pushing each input into [`Self::merger`] and finishing.
    pub fn merge<'a>(
        &self,
        inputs: impl IntoIterator<Item = &'a Metadata>,
    ) -> Result<Metadata, ContextError> {
        let mut merger = self.merger();
        for input in inputs {
            merger.push(input);
        }
        merger.finish()
    }
}

/// Owned accumulator for inputs merged into one output.
///
/// Retains the peer address only when every pushed input observed the same
/// peer. Transport headers, authorized identity, and materialized composites
/// are not retained. Finishing without inputs yields empty metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataMerger {
    peer: PeerAddrMerger,
}

impl MetadataMerger {
    /// Folds one contributing input into the running merge.
    pub fn push(&mut self, input: &Metadata) {
        self.peer.push(input.peer_addr());
    }

    /// Returns the merged metadata.
    ///
    /// Pushing inputs never fails. This fails when the merged metadata does
    /// not meet the node's context requirements, for example when an entry
    /// listed in its `required_out` policy is absent. `required_out` is not
    /// yet enforced, so the current implementation always succeeds.
    pub fn finish(self) -> Result<Metadata, ContextError> {
        let mut output = Metadata::default();
        if let Some(peer) = self.peer.finish() {
            output.set_peer_addr(peer);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::auth::AuthorizedIdentity;
    use crate::pdata_context::ContextMaterializationPlan;
    use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
    use otel_arrow_dfe_config::context::policy::{
        ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
    };
    use otel_arrow_dfe_config::context::transport_headers::TransportHeaders;
    use std::net::SocketAddr;

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid context name")
    }

    fn peer(value: &str) -> SocketAddr {
        value.parse().expect("peer address")
    }

    /// Builds metadata carrying every field: headers, identity, peer, and a composite.
    fn populated_metadata(peer_addr: Option<SocketAddr>) -> Metadata {
        let plan = ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
            scope: ContextScope::Pipeline("group".into(), "pipeline".into()),
            name: name("request"),
            definition: ContextEntryDefinition(vec![
                ContextEntryPart::TransportHeader {
                    name: name("tenant"),
                    store_as: None,
                },
                ContextEntryPart::Constant {
                    name: name("route"),
                    value: "fixed".to_owned(),
                },
            ]),
        }])
        .expect("materialization plan");
        let identity_policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("identity policy");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::captured(
            name("tenant"),
            "X-Tenant",
            true,
            ValueKind::Text,
            b"acme".as_slice(),
        ));
        let mut metadata = Metadata::default();
        metadata.set_transport_headers(headers);
        metadata.capture_authorized_identity(
            &identity_policy,
            &AuthorizedIdentity::new().with_subject("reader"),
        );
        if let Some(peer_addr) = peer_addr {
            metadata.set_peer_addr(peer_addr);
        }
        metadata.materialize_context(&plan);
        assert!(metadata.materialized_context().is_some());
        metadata
    }

    /// Applies the partition processor's in-place header update: clone, take, push, and set.
    fn partition_header_update(
        input: &Metadata,
        produced: &[(&ContextEntryName, ValueKind, &[u8])],
    ) -> Metadata {
        let mut output = input.clone();
        let mut headers = output.take_transport_headers().unwrap_or_default();
        for (name, kind, bytes) in produced {
            headers.push(TransportHeader::new((*name).clone(), *kind, *bytes));
        }
        output.set_transport_headers(headers);
        output
    }

    /// Scenario: partition-style projection appends text and binary values to empty and populated inputs.
    /// Guarantees: output equals the partition processor's header update, retaining identity, peer, and composites.
    #[test]
    fn projection_matches_partition_header_update() {
        let partition = name("partition");
        let shard = name("shard");
        let binding = ProjectionBinding::new(vec![shard.clone(), partition.clone()]);
        let partition_entry = binding
            .produced(ContextDomain::TransportHeader, &partition)
            .expect("partition handle");
        let shard_entry = binding
            .produced(ContextDomain::TransportHeader, &shard)
            .expect("shard handle");

        for input in [
            Metadata::default(),
            populated_metadata(None),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            let actual = binding
                .form(
                    &input,
                    &[ProducedValue {
                        entry: &partition_entry,
                        kind: ValueKind::Text,
                        bytes: b"0",
                    }],
                )
                .expect("projection forms");
            let expected =
                partition_header_update(&input, &[(&partition, ValueKind::Text, b"0".as_slice())]);
            assert_eq!(actual, expected);
            let headers = actual.transport_headers().expect("produced header");
            let header = headers
                .find_by_name("partition")
                .next()
                .expect("partition header");
            assert_eq!(header.value.value_kind, ValueKind::Text);
            assert_eq!(header.value.bytes, b"0");
            assert_eq!(header.wire_name(), "partition");

            let actual = binding
                .form(
                    &input,
                    &[
                        ProducedValue {
                            entry: &partition_entry,
                            kind: ValueKind::Binary,
                            bytes: &514i64.to_le_bytes(),
                        },
                        ProducedValue {
                            entry: &shard_entry,
                            kind: ValueKind::Text,
                            bytes: b"east",
                        },
                    ],
                )
                .expect("projection forms");
            let expected = partition_header_update(
                &input,
                &[
                    (
                        &partition,
                        ValueKind::Binary,
                        514i64.to_le_bytes().as_slice(),
                    ),
                    (&shard, ValueKind::Text, b"east".as_slice()),
                ],
            );
            assert_eq!(actual, expected);
            assert_eq!(actual.peer_addr(), input.peer_addr());
            assert_eq!(
                actual.authorized_identity_entries(),
                input.authorized_identity_entries()
            );
            assert_eq!(
                actual
                    .materialized_context()
                    .map(|context| context.entries()),
                input
                    .materialized_context()
                    .map(|context| context.entries())
            );
        }
    }

    /// Scenario: a projection is formed without any produced values.
    /// Guarantees: the output equals the input, including absent transport headers.
    #[test]
    fn projection_without_values_preserves_input() {
        let binding = ProjectionBinding::new(vec![name("partition")]);
        for input in [
            Metadata::default(),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            assert_eq!(binding.form(&input, &[]), Ok(input));
        }
        assert_eq!(
            binding
                .form(&Metadata::default(), &[])
                .expect("projection forms")
                .transport_headers()
                .map(|headers| headers.len()),
            None
        );
    }

    /// Scenario: construction-time lookups name an undeclared transport header or any authorized-identity entry.
    /// Guarantees: both fail with UndeclaredContextEntry naming the domain and entry instead of issuing a handle.
    #[test]
    fn produced_lookup_rejects_undeclared_entries() {
        let binding = ProjectionBinding::new(vec![name("partition")]);
        for (domain, entry) in [
            (ContextDomain::TransportHeader, "other"),
            (ContextDomain::AuthorizedIdentity, "partition"),
        ] {
            let Err(Error::UndeclaredContextEntry {
                domain: actual_domain,
                entry: actual_entry,
            }) = binding.produced(domain, &name(entry))
            else {
                panic!("{domain:?} `{entry}` is not a declared produced entry");
            };
            assert_eq!((actual_domain, actual_entry), (domain, name(entry)));
        }
    }

    /// Scenario: a node supplies a handle issued by another binding, including one compiled from identical entries.
    /// Guarantees: form fails with ContextMismatch instead of writing this binding's entry, while clones accept handles and compare equal.
    #[test]
    fn projection_rejects_mismatched_handles() {
        let partition = name("partition");
        let binding = ProjectionBinding::new(vec![partition.clone()]);
        fn value(entry: &ProducedEntry) -> ProducedValue<'_> {
            ProducedValue {
                entry,
                kind: ValueKind::Text,
                bytes: b"0",
            }
        }
        for other in [
            ProjectionBinding::new(vec![name("tenant")]),
            ProjectionBinding::new(vec![partition.clone()]),
        ] {
            assert_ne!(other, binding);
            let mismatched = other
                .produced(ContextDomain::TransportHeader, &other.produced[0])
                .expect("other handle");
            assert_eq!(
                binding.form(&Metadata::default(), &[value(&mismatched)]),
                Err(ContextError::ContextMismatch)
            );
        }

        let own = binding
            .produced(ContextDomain::TransportHeader, &partition)
            .expect("own handle");
        let clone = binding.clone();
        assert_eq!(clone, binding);
        assert_eq!(
            clone.form(&Metadata::default(), &[value(&own)]),
            binding.form(&Metadata::default(), &[value(&own)])
        );
        assert!(binding.form(&Metadata::default(), &[value(&own)]).is_ok());
    }

    /// Scenario: batch-style merges see no inputs, one input, unanimous peers, distinct peers, or a peerless input.
    /// Guarantees: output equals empty metadata plus the PeerAddrMerger result, and the incremental merger agrees.
    #[test]
    fn merge_matches_peer_addr_merger() {
        let a = Some(peer("10.0.0.1:1"));
        let b = Some(peer("10.0.0.2:2"));
        let binding = MergeBinding::new();
        let cases: [&[Option<SocketAddr>]; 8] = [
            &[],
            &[a],
            &[None],
            &[a, a, a],
            &[a, b],
            &[a, None],
            &[None, a],
            &[a, b, a],
        ];
        for peers in cases {
            let inputs = peers
                .iter()
                .map(|peer_addr| populated_metadata(*peer_addr))
                .collect::<Vec<_>>();
            let mut peers_merged = PeerAddrMerger::new();
            for input in &inputs {
                peers_merged.push(input.peer_addr());
            }
            let mut expected = Metadata::default();
            if let Some(peer_addr) = peers_merged.finish() {
                expected.set_peer_addr(peer_addr);
            }

            let merged = binding.merge(&inputs).expect("merge forms");
            assert_eq!(merged, expected, "inputs {peers:?}");
            assert!(merged.transport_headers().is_none());
            assert!(merged.authorized_identity_entries().is_none());
            assert!(merged.materialized_context().is_none());

            let mut merger = binding.merger();
            for input in &inputs {
                merger.push(input);
            }
            assert_eq!(
                merger.finish(),
                Ok(expected),
                "incremental inputs {peers:?}"
            );
        }
    }
}
