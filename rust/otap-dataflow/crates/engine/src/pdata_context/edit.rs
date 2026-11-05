// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Edits and Merges bindings for processors that form output context from input context.

use super::{ContextAction, ContextDomain, ContextError, ContextLayoutIdentity, Metadata};
use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, ValueKind};
use otel_arrow_dfe_config::error::Error;

/// Handle for one entry an edit binding adds, valid only with the issuing binding.
#[derive(Debug, Clone)]
pub struct EditEntry {
    /// Identity of the issuing binding.
    binding: ContextLayoutIdentity,
    /// Position in the binding's sorted entry list.
    index: usize,
}

impl PartialEq for EditEntry {
    fn eq(&self, other: &Self) -> bool {
        self.binding.matches(&other.binding) && self.index == other.index
    }
}

impl Eq for EditEntry {}

/// One value supplied for added entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditValue<'a> {
    /// Handle issued by the node's edit binding.
    pub entry: &'a EditEntry,
    /// Text or binary representation.
    pub kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

/// Compiled 1:1 edit for a processor that declares `Edits`.
#[derive(Debug, Clone)]
pub struct EditBinding {
    /// Identity shared by the handles this binding issues.
    identity: ContextLayoutIdentity,
    /// Added transport-header entries, sorted and unique.
    added: Box<[ContextEntryName]>,
}

impl PartialEq for EditBinding {
    fn eq(&self, other: &Self) -> bool {
        self.identity.matches(&other.identity)
    }
}

impl Eq for EditBinding {}

impl EditBinding {
    /// Compiles an edit from declared actions, rejecting duplicate entries.
    pub(super) fn compile(actions: &[ContextAction]) -> Result<Self, Error> {
        let mut added = Vec::with_capacity(actions.len());
        for action in actions {
            match action {
                ContextAction::Add { entry } => added.push(entry.clone()),
            }
        }
        added.sort();
        if let Some(pair) = added.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(Error::InvalidUserConfig {
                error: format!("context entry `{}` is added more than once", pair[0]),
            });
        }
        Ok(Self {
            identity: ContextLayoutIdentity::default(),
            added: added.into_boxed_slice(),
        })
    }

    /// Returns the handle for an entry this node declared it adds.
    pub fn edit_entry(
        &self,
        domain: ContextDomain,
        entry: &ContextEntryName,
    ) -> Result<EditEntry, Error> {
        let undeclared = || Error::UndeclaredContextEntry {
            domain,
            entry: entry.clone(),
        };
        if domain != ContextDomain::TransportHeader {
            return Err(undeclared());
        }
        let index = self.added.binary_search(entry).map_err(|_| undeclared())?;
        Ok(EditEntry {
            binding: self.identity.clone(),
            index,
        })
    }

    /// Consumes the input and returns it with each value appended as a transport header, in order.
    pub fn form(
        &self,
        mut input: Metadata,
        derived: &[EditValue<'_>],
    ) -> Result<Metadata, ContextError> {
        if derived.is_empty() {
            return Ok(input);
        }
        let mut headers = input.take_transport_headers().unwrap_or_default();
        for value in derived {
            let name = self.name_of(value.entry)?;
            headers.push(TransportHeader::new(name.clone(), value.kind, value.bytes));
        }
        input.set_transport_headers(headers);
        Ok(input)
    }

    /// Returns the entry name for a handle, failing when another binding issued it.
    fn name_of(&self, entry: &EditEntry) -> Result<&ContextEntryName, ContextError> {
        if !entry.binding.matches(&self.identity) {
            return Err(ContextError::ContextMismatch);
        }
        self.added
            .get(entry.index)
            .ok_or(ContextError::ContextMismatch)
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

    /// Compiles a plan for composite `request` built from the `tenant` header and a constant.
    fn request_plan() -> ContextMaterializationPlan {
        ContextMaterializationPlan::compile(&[ContextEntryDeclaration {
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
        .expect("materialization plan")
    }

    /// Builds metadata carrying every field: headers, identity, peer, and a composite.
    fn populated_metadata(peer_addr: Option<SocketAddr>) -> Metadata {
        tenant_metadata(b"acme", peer_addr)
    }

    /// Builds populated metadata whose `tenant` header and `request` composite carry `tenant`.
    fn tenant_metadata(tenant: &[u8], peer_addr: Option<SocketAddr>) -> Metadata {
        let identity_policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("identity policy");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::captured(
            name("tenant"),
            "X-Tenant",
            true,
            ValueKind::Text,
            tenant,
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
        metadata.materialize_context(&request_plan());
        assert!(metadata.materialized_context().is_some());
        metadata
    }

    /// Compiles an edit that only adds `adds`, in order.
    fn edit(adds: &[ContextEntryName]) -> EditBinding {
        let actions = adds.iter().cloned().map(add).collect::<Vec<_>>();
        EditBinding::compile(&actions).expect("edit compiles")
    }

    fn add(entry: ContextEntryName) -> ContextAction {
        ContextAction::Add { entry }
    }

    /// Applies the partition processor's in-place header update: clone, take, push, and set.
    fn partition_header_update(
        input: &Metadata,
        derived: &[(&ContextEntryName, ValueKind, &[u8])],
    ) -> Metadata {
        let mut output = input.clone();
        let mut headers = output.take_transport_headers().unwrap_or_default();
        for (name, kind, bytes) in derived {
            headers.push(TransportHeader::new((*name).clone(), *kind, *bytes));
        }
        output.set_transport_headers(headers);
        output
    }

    /// Scenario: partition-style edit appends text and binary values to empty and populated inputs.
    /// Guarantees: output equals the partition processor's header update, retaining identity, peer, and composites.
    #[test]
    fn edit_matches_partition_header_update() {
        let partition = name("partition");
        let shard = name("shard");
        let binding = edit(&[partition.clone(), shard.clone()]);
        let partition_entry = binding
            .edit_entry(ContextDomain::TransportHeader, &partition)
            .expect("partition handle");
        let shard_entry = binding
            .edit_entry(ContextDomain::TransportHeader, &shard)
            .expect("shard handle");

        for input in [
            Metadata::default(),
            populated_metadata(None),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            let actual = binding
                .form(
                    input.clone(),
                    &[EditValue {
                        entry: &partition_entry,
                        kind: ValueKind::Text,
                        bytes: b"0",
                    }],
                )
                .expect("edit forms");
            let expected =
                partition_header_update(&input, &[(&partition, ValueKind::Text, b"0".as_slice())]);
            assert_eq!(actual, expected);
            let headers = actual.transport_headers().expect("derived header");
            let header = headers
                .find_by_name("partition")
                .next()
                .expect("partition header");
            assert_eq!(header.value.value_kind, ValueKind::Text);
            assert_eq!(header.value.bytes, b"0");
            assert_eq!(header.wire_name(), "partition");

            let actual = binding
                .form(
                    input.clone(),
                    &[
                        EditValue {
                            entry: &partition_entry,
                            kind: ValueKind::Binary,
                            bytes: &514i64.to_le_bytes(),
                        },
                        EditValue {
                            entry: &shard_entry,
                            kind: ValueKind::Text,
                            bytes: b"east",
                        },
                    ],
                )
                .expect("edit forms");
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

    /// Scenario: an edit is formed without any derived values.
    /// Guarantees: the output equals the input, including absent transport headers.
    #[test]
    fn edit_without_values_preserves_input() {
        let binding = edit(&[name("partition")]);
        for input in [
            Metadata::default(),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            assert_eq!(binding.form(input.clone(), &[]), Ok(input));
        }
        assert_eq!(
            binding
                .form(Metadata::default(), &[])
                .expect("edit forms")
                .transport_headers()
                .map(|headers| headers.len()),
            None
        );
    }

    /// Scenario: construction-time lookups name an undeclared transport header or any authorized-identity entry.
    /// Guarantees: both fail with UndeclaredContextEntry naming the domain and entry instead of issuing a handle.
    #[test]
    fn edit_entry_lookup_rejects_undeclared_entries() {
        let binding = edit(&[name("partition")]);
        for (domain, entry) in [
            (ContextDomain::TransportHeader, "other"),
            (ContextDomain::AuthorizedIdentity, "partition"),
        ] {
            let Err(Error::UndeclaredContextEntry {
                domain: actual_domain,
                entry: actual_entry,
            }) = binding.edit_entry(domain, &name(entry))
            else {
                panic!("{domain:?} `{entry}` is not a declared derived entry");
            };
            assert_eq!((actual_domain, actual_entry), (domain, name(entry)));
        }
    }

    /// Scenario: a node supplies a handle issued by another binding, including one compiled from identical entries.
    /// Guarantees: form fails with ContextMismatch instead of writing this binding's entry, while clones accept handles and compare equal.
    #[test]
    fn edit_rejects_mismatched_handles() {
        let partition = name("partition");
        let binding = edit(std::slice::from_ref(&partition));
        fn value(entry: &EditEntry) -> EditValue<'_> {
            EditValue {
                entry,
                kind: ValueKind::Text,
                bytes: b"0",
            }
        }
        for other in [
            edit(&[name("tenant")]),
            edit(std::slice::from_ref(&partition)),
        ] {
            assert_ne!(other, binding);
            let mismatched = other
                .edit_entry(ContextDomain::TransportHeader, &other.added[0])
                .expect("other handle");
            assert_eq!(
                binding.form(Metadata::default(), &[value(&mismatched)]),
                Err(ContextError::ContextMismatch)
            );
        }

        let own = binding
            .edit_entry(ContextDomain::TransportHeader, &partition)
            .expect("own handle");
        let clone = binding.clone();
        assert_eq!(clone, binding);
        assert_eq!(
            clone.form(Metadata::default(), &[value(&own)]),
            binding.form(Metadata::default(), &[value(&own)])
        );
        assert!(binding.form(Metadata::default(), &[value(&own)]).is_ok());
    }

    /// Scenario: an edit declares the same Add entry twice.
    /// Guarantees: compilation fails naming the entry instead of silently collapsing the actions.
    #[test]
    fn duplicate_adds_are_rejected_at_compilation() {
        let actions = [add(name("partition")), add(name("partition"))];
        let message = EditBinding::compile(&actions)
            .expect_err("duplicate add is rejected")
            .to_string();
        assert!(
            message.contains("`partition` is added more than once"),
            "{message}"
        );
    }
}
