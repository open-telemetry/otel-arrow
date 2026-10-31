// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projection and merge adapters retain the existing derived-metadata semantics.
//!
//! Projection forms one output from one input plus values the node produces.
//! Merge forms one output from several inputs.

use super::{ContextDomain, Metadata, PeerAddrMerger};
use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, ValueKind};
use otel_arrow_dfe_config::error::Error;

/// Compiled handle for one entry a node declared it produces.
///
/// Obtain once during node construction from [`ProjectionBinding::produced`];
/// handles are valid only with the binding that issued them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducedEntry(usize);

/// One value supplied for a produced entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducedValue<'a> {
    /// Produced entry handle issued by the node's projection binding.
    pub entry: ProducedEntry,
    /// Text or binary representation.
    pub kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProducedTarget {
    domain: ContextDomain,
    name: ContextEntryName,
}

/// Compiled projection for a node that produces context entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionBinding {
    /// Declared produced entries, sorted by domain and name and unique.
    produced: Box<[ProducedTarget]>,
}

impl ProjectionBinding {
    pub(super) fn new(mut produced: Vec<(ContextDomain, ContextEntryName)>) -> Self {
        produced.sort();
        produced.dedup();
        Self {
            produced: produced
                .into_iter()
                .map(|(domain, name)| ProducedTarget { domain, name })
                .collect(),
        }
    }

    /// Returns the handle for an entry this node declared it produces.
    ///
    /// Call once during node construction with the same domain and entry the
    /// node returned from its context declarations. An undeclared entry, or a
    /// declared entry in a domain the existing implementation cannot write, is
    /// a configuration error.
    pub fn produced(
        &self,
        domain: ContextDomain,
        entry: &ContextEntryName,
    ) -> Result<ProducedEntry, Error> {
        let index = self
            .produced
            .binary_search_by(|target| (target.domain, &target.name).cmp(&(domain, entry)))
            .map_err(|_| Error::InvalidUserConfig {
                error: format!(
                    "produced context entry `{entry}` in domain {domain:?} was not declared by this node"
                ),
            })?;
        match domain {
            ContextDomain::TransportHeader => Ok(ProducedEntry(index)),
            ContextDomain::AuthorizedIdentity => Err(Error::InvalidUserConfig {
                error: format!(
                    "produced context entry `{entry}` cannot be written; only transport-header entries can be produced"
                ),
            }),
        }
    }

    /// Clones the input and appends each produced value as a transport header.
    ///
    /// Values are appended in slice order after any captured headers. Materialized
    /// composites, authorized identity, and peer address are retained unchanged,
    /// so produced values do not re-evaluate composites. Without produced values
    /// the output equals the input.
    ///
    /// # Panics
    ///
    /// Panics if a produced value carries a handle issued by another binding.
    #[must_use]
    pub fn form(&self, input: &Metadata, produced: &[ProducedValue<'_>]) -> Metadata {
        let mut output = input.clone();
        if produced.is_empty() {
            return output;
        }
        let mut headers = output.take_transport_headers().unwrap_or_default();
        for value in produced {
            let target = self
                .produced
                .get(value.entry.0)
                .expect("produced entry handle was issued by this projection binding");
            debug_assert_eq!(target.domain, ContextDomain::TransportHeader);
            headers.push(TransportHeader::new(
                target.name.clone(),
                value.kind,
                value.bytes,
            ));
        }
        output.set_transport_headers(headers);
        output
    }
}

/// Compiled merge for a node that combines several inputs into one output.
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
    #[must_use]
    pub fn merge<'a>(&self, inputs: impl IntoIterator<Item = &'a Metadata>) -> Metadata {
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
    #[must_use]
    pub fn finish(self) -> Metadata {
        let mut output = Metadata::default();
        if let Some(peer) = self.peer.finish() {
            output.set_peer_addr(peer);
        }
        output
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

    /// Builds metadata carrying every legacy field: headers, identity, peer, and a composite.
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

    /// Applies the partition processor's existing in-place header update.
    fn legacy_partition_update(
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
    /// Guarantees: output equals the legacy clone, take, push, and set sequence, retaining identity, peer, and composites.
    #[test]
    fn projection_matches_legacy_partition_update() {
        let partition = name("partition");
        let shard = name("shard");
        let binding = ProjectionBinding::new(vec![
            (ContextDomain::TransportHeader, shard.clone()),
            (ContextDomain::TransportHeader, partition.clone()),
        ]);
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
            let actual = binding.form(
                &input,
                &[ProducedValue {
                    entry: partition_entry,
                    kind: ValueKind::Text,
                    bytes: b"0",
                }],
            );
            let expected =
                legacy_partition_update(&input, &[(&partition, ValueKind::Text, b"0".as_slice())]);
            assert_eq!(actual, expected);
            let headers = actual.transport_headers().expect("produced header");
            let header = headers
                .find_by_name("partition")
                .next()
                .expect("partition header");
            assert_eq!(header.value.value_kind, ValueKind::Text);
            assert_eq!(header.value.bytes, b"0");
            assert_eq!(header.wire_name(), "partition");

            let actual = binding.form(
                &input,
                &[
                    ProducedValue {
                        entry: partition_entry,
                        kind: ValueKind::Binary,
                        bytes: &514i64.to_le_bytes(),
                    },
                    ProducedValue {
                        entry: shard_entry,
                        kind: ValueKind::Text,
                        bytes: b"east",
                    },
                ],
            );
            let expected = legacy_partition_update(
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
        let binding =
            ProjectionBinding::new(vec![(ContextDomain::TransportHeader, name("partition"))]);
        for input in [
            Metadata::default(),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            assert_eq!(binding.form(&input, &[]), input);
        }
        assert_eq!(
            binding
                .form(&Metadata::default(), &[])
                .transport_headers()
                .map(|headers| headers.len()),
            None
        );
    }

    /// Scenario: construction-time lookups name an undeclared entry or a declared identity entry.
    /// Guarantees: both fail loudly instead of issuing a handle that would write the wrong domain or name.
    #[test]
    fn produced_lookup_rejects_undeclared_and_unwritable_entries() {
        let binding = ProjectionBinding::new(vec![
            (ContextDomain::TransportHeader, name("partition")),
            (ContextDomain::AuthorizedIdentity, name("subject")),
        ]);
        assert!(
            binding
                .produced(ContextDomain::TransportHeader, &name("other"))
                .is_err()
        );
        assert!(
            binding
                .produced(ContextDomain::AuthorizedIdentity, &name("partition"))
                .is_err()
        );
        let Err(Error::InvalidUserConfig { error }) =
            binding.produced(ContextDomain::AuthorizedIdentity, &name("subject"))
        else {
            panic!("identity entries cannot be produced");
        };
        assert!(error.contains("only transport-header entries can be produced"));
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
            let mut legacy = PeerAddrMerger::new();
            for input in &inputs {
                legacy.push(input.peer_addr());
            }
            let mut expected = Metadata::default();
            if let Some(peer_addr) = legacy.finish() {
                expected.set_peer_addr(peer_addr);
            }

            let merged = binding.merge(&inputs);
            assert_eq!(merged, expected, "inputs {peers:?}");
            assert!(merged.transport_headers().is_none());
            assert!(merged.authorized_identity_entries().is_none());
            assert!(merged.materialized_context().is_none());

            let mut merger = binding.merger();
            for input in &inputs {
                merger.push(input);
            }
            assert_eq!(merger.finish(), expected, "incremental inputs {peers:?}");
        }
    }
}
