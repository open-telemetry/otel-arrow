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
    use otel_arrow_dfe_config::context::ContextEntryName;
    use otel_arrow_dfe_config::context::transport_headers::{
        TransportHeader, TransportHeaders, ValueKind,
    };
    use std::net::SocketAddr;

    fn peer(value: &str) -> SocketAddr {
        value.parse().expect("peer address")
    }

    fn metadata(peer_addr: Option<SocketAddr>) -> Metadata {
        let mut metadata = Metadata::default();
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::new(
            ContextEntryName::try_from("test-header").unwrap(),
            ValueKind::Text,
            *b"hello",
        ));
        metadata.set_transport_headers(headers);
        if let Some(peer_addr) = peer_addr {
            metadata.set_peer_addr(peer_addr);
        }
        metadata
    }

    /// Scenario: a Merges binding sees various configurations.
    /// Guarantees: output correctly sets peer addr and clears other fields.
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
            let inputs = peers
                .iter()
                .map(|peer_addr| metadata(*peer_addr))
                .collect::<Vec<_>>();
            let mut merger = binding.merger();
            for input in &inputs {
                assert!(input.transport_headers().is_some());
                merger.combine(input);
            }
            let merged = merger.finish();

            assert_eq!(merged.peer_addr(), expected);
            assert!(merged.transport_headers().is_none());
            assert!(merged.authorized_identity_entries().is_none());
            assert!(merged.materialized_context().is_none());
        }
    }
}
