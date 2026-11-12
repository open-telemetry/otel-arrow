// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Merge bindings for processors that form output context from
//! multiple input contexts.

use super::{Metadata, PeerAddrMerger};

/// Compiled N:1 merge for a processor that declares `Merges`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MergeBinding;

impl MergeBinding {
    /// Returns an empty accumulator for inputs observed one at a time.
    #[must_use]
    pub fn merger(&self) -> MetadataMerger {
        MetadataMerger {
            peer: PeerAddrMerger::new(),
        }
    }
}

/// Accumulator that keeps the peer address only when every input observed the same peer.
#[derive(Debug, Clone)]
pub struct MetadataMerger {
    peer: PeerAddrMerger,
}

impl MetadataMerger {
    /// Folds one input into the running merge.
    pub fn combine(&mut self, input: &Metadata) {
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
    use otel_arrow_dfe_config::context::ContextEntryName;
    use otel_arrow_dfe_config::context::authorized_identity::AuthorizedIdentityPolicy;
    use otel_arrow_dfe_config::context::policy::{
        ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
    };
    use otel_arrow_dfe_config::context::transport_headers::{
        TransportHeader, TransportHeaders, ValueKind,
    };
    use std::net::SocketAddr;

    fn peer(value: &str) -> SocketAddr {
        value.parse().expect("peer address")
    }

    fn metadata(peer_addr: Option<SocketAddr>) -> Metadata {
        let mut metadata = Metadata::default();
        let header_name = ContextEntryName::try_from("test-header").unwrap();
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::new(
            header_name.clone(),
            ValueKind::Text,
            *b"hello",
        ));
        metadata.set_transport_headers(headers);
        let identity_policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("identity policy");
        metadata.capture_authorized_identity(
            &identity_policy,
            &AuthorizedIdentity::new().with_subject("reader"),
        );
        metadata.materialize_context(
            &ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
                scope: ContextScope::Pipeline("group".into(), "pipeline".into()),
                name: ContextEntryName::try_from("request").unwrap(),
                definition: ContextEntryDefinition(vec![ContextEntryPart::TransportHeader {
                    name: header_name,
                    store_as: None,
                }]),
            }])
            .expect("materialization plan"),
        );
        if let Some(peer_addr) = peer_addr {
            metadata.set_peer_addr(peer_addr);
        }
        assert!(metadata.transport_headers().is_some());
        assert!(metadata.authorized_identity_entries().is_some());
        assert!(metadata.materialized_context().is_some());
        metadata
    }

    /// Scenario: zero inputs, unanimous peers, distinct peers, and missing peers are merged.
    /// Guarantees: only unanimous present peers survive; headers, identity, and composites are discarded.
    #[test]
    fn merges_peer_addr() {
        let a = Some(peer("10.0.0.1:1"));
        let b = Some(peer("10.0.0.2:2"));
        let binding = MergeBinding;
        let cases: [(&[Option<SocketAddr>], Option<SocketAddr>); _] = [
            (&[], None),
            (&[a], a),
            (&[None], None),
            (&[a, a, a], a),
            (&[a, b], None),
            (&[a, None], None),
            (&[None, a], None),
            (&[a, b, a], None),
        ];
        for (peers, expected) in cases {
            let mut merger = binding.merger();
            for peer_addr in peers {
                merger.combine(&metadata(*peer_addr));
            }
            let merged = merger.finish();

            assert_eq!(merged.peer_addr(), expected);
            assert!(merged.transport_headers().is_none());
            assert!(merged.authorized_identity_entries().is_none());
            assert!(merged.materialized_context().is_none());
        }
    }
}
