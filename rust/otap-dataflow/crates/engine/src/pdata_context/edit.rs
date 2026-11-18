// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Edit bindings for processors that form output context from input context,
//! and production bindings for receivers that form context from produced values.

use super::{
    ContextAction, ContextDomain, ContextError, ContextLayoutIdentity, ContextMaterializationPlan,
    Metadata,
};
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

/// One value supplied for an added entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditValue<'a> {
    /// Handle issued by the node's edit binding.
    pub entry: &'a EditEntry,
    /// Text or binary representation.
    pub kind: ValueKind,
    /// Raw value bytes, moved into the output header without a copy.
    pub bytes: Box<[u8]>,
}

/// Compiled 1:1 edit for a processor that declares `Edits`.
///
/// Edits do not materialize composites: for now only receiver bindings
/// materialize context.
/// TODO(4541): settle which composites edits complete, with topic routing.
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
        let added = actions
            .iter()
            .map(|action| match action {
                ContextAction::Add { entry } => entry.clone(),
            })
            .collect();
        Self::from_entries(added, "added")
    }

    /// Builds a binding over `entries`, rejecting duplicates; `verb` names
    /// how the declaring node supplies them, for the error message.
    fn from_entries(mut added: Vec<ContextEntryName>, verb: &str) -> Result<Self, Error> {
        added.sort();
        if let Some(pair) = added.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(Error::InvalidUserConfig {
                error: format!("context entry `{}` is {verb} more than once", pair[0]),
            });
        }
        Ok(Self {
            identity: ContextLayoutIdentity::default(),
            added: added.into_boxed_slice(),
        })
    }

    /// Returns the handle for an entry this node declared it adds.
    pub fn entry(&self, entry: &ContextEntryName) -> Result<EditEntry, Error> {
        let undeclared = || Error::UndeclaredContextEntry {
            domain: ContextDomain::TransportHeader,
            entry: entry.clone(),
        };
        let index = self.added.binary_search(entry).map_err(|_| undeclared())?;
        Ok(EditEntry {
            binding: self.identity.clone(),
            index,
        })
    }

    /// Returns a new output context: the input with each value appended as a
    /// transport header, in order. Materialized composites in the input are
    /// kept as they are.
    pub fn form<'a>(
        &self,
        mut input: Metadata,
        values: impl IntoIterator<Item = EditValue<'a>>,
    ) -> Result<Metadata, ContextError> {
        let mut values = values.into_iter().peekable();
        if values.peek().is_none() {
            return Ok(input);
        }
        let mut headers = input.take_transport_headers().unwrap_or_default();
        for value in values {
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

/// Compiled formation for a receiver that declares `Produces`: each context is
/// formed whole from the produced values, then composites are materialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBinding {
    /// Produced transport-header entries, applied to an empty input.
    produced: EditBinding,
    /// Composites materialized after the produced values are set, since a
    /// produced entry can complete a composite.
    materialization: ContextMaterializationPlan,
}

impl ProductionBinding {
    /// Compiles a production from declared entries, rejecting duplicates.
    pub(super) fn compile(
        entries: &[ContextEntryName],
        materialization: ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        Ok(Self {
            produced: EditBinding::from_entries(entries.to_vec(), "produced")?,
            materialization,
        })
    }

    /// Returns the handle for an entry this receiver declared it produces.
    pub fn entry(&self, entry: &ContextEntryName) -> Result<EditEntry, Error> {
        self.produced.entry(entry)
    }

    /// Returns a new context holding the produced values as transport headers,
    /// in order, with composites materialized.
    pub fn form<'a>(
        &self,
        values: impl IntoIterator<Item = EditValue<'a>>,
    ) -> Result<Metadata, ContextError> {
        let mut metadata = self.produced.form(Metadata::default(), values)?;
        metadata.materialize_context(&self.materialization);
        Ok(metadata)
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
        let identity_policy: AuthorizedIdentityPolicy =
            serde_json::from_value(serde_json::json!([{"claim": "sub", "store_as": "subject"}]))
                .expect("identity policy");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::captured(
            name("tenant"),
            "X-Tenant",
            true,
            ValueKind::Text,
            *b"acme",
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
        values: &[(&ContextEntryName, ValueKind, &[u8])],
    ) -> Metadata {
        let mut output = input.clone();
        let mut headers = output.take_transport_headers().unwrap_or_default();
        for (name, kind, bytes) in values {
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
        let partition_entry = binding.entry(&partition).expect("partition handle");
        let shard_entry = binding.entry(&shard).expect("shard handle");

        for input in [
            Metadata::default(),
            populated_metadata(None),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            let actual = binding
                .form(
                    input.clone(),
                    [EditValue {
                        entry: &partition_entry,
                        kind: ValueKind::Text,
                        bytes: b"0".as_slice().into(),
                    }],
                )
                .expect("edit forms");
            let expected =
                partition_header_update(&input, &[(&partition, ValueKind::Text, b"0".as_slice())]);
            assert_eq!(actual, expected);
            let headers = actual.transport_headers().expect("added header");
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
                    [
                        EditValue {
                            entry: &partition_entry,
                            kind: ValueKind::Binary,
                            bytes: 514i64.to_le_bytes().into(),
                        },
                        EditValue {
                            entry: &shard_entry,
                            kind: ValueKind::Text,
                            bytes: b"east".as_slice().into(),
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
        }
    }

    /// Scenario: an edit is formed without any added values.
    /// Guarantees: the output equals the input, including absent transport headers.
    #[test]
    fn edit_without_values_preserves_input() {
        let binding = edit(&[name("partition")]);
        for input in [
            Metadata::default(),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            assert_eq!(binding.form(input.clone(), []), Ok(input));
        }
    }

    /// Scenario: a construction-time lookup names an undeclared transport header.
    /// Guarantees: it fails with UndeclaredContextEntry naming the transport-header domain and entry instead of issuing a handle.
    #[test]
    fn edit_entry_lookup_rejects_undeclared_entries() {
        let binding = edit(&[name("partition")]);
        let Err(Error::UndeclaredContextEntry { domain, entry }) = binding.entry(&name("other"))
        else {
            panic!("`other` is not a declared added entry");
        };
        assert_eq!(
            (domain, entry),
            (ContextDomain::TransportHeader, name("other"))
        );
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
                bytes: b"0".as_slice().into(),
            }
        }
        for other in [
            edit(&[name("tenant")]),
            edit(std::slice::from_ref(&partition)),
        ] {
            assert_ne!(other, binding);
            let mismatched = other.entry(&other.added[0]).expect("other handle");
            assert_eq!(
                binding.form(Metadata::default(), [value(&mismatched)]),
                Err(ContextError::ContextMismatch)
            );
        }

        let own = binding.entry(&partition).expect("own handle");
        let clone = binding.clone();
        assert_eq!(clone, binding);
        assert_eq!(
            clone.form(Metadata::default(), [value(&own)]),
            binding.form(Metadata::default(), [value(&own)])
        );
        assert!(binding.form(Metadata::default(), [value(&own)]).is_ok());
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

    /// Scenario: a production declares `tenant`, which a composite needs, and forms a context from it.
    /// Guarantees: the formed context holds the produced header and the composite it completes, exactly
    /// as if the header had been set and then the plan applied.
    #[test]
    fn production_materializes_composites_completed_by_produced_values() {
        let tenant = name("tenant");
        let binding = ProductionBinding::compile(std::slice::from_ref(&tenant), request_plan())
            .expect("production compiles");
        let entry = binding.entry(&tenant).expect("tenant handle");

        let actual = binding
            .form([EditValue {
                entry: &entry,
                kind: ValueKind::Text,
                bytes: b"acme".as_slice().into(),
            }])
            .expect("production forms");

        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::new(tenant, ValueKind::Text, *b"acme"));
        let mut expected = Metadata::default();
        expected.set_transport_headers(headers);
        expected.materialize_context(&request_plan());
        assert!(expected.materialized_context().is_some());
        assert_eq!(actual, expected);
    }

    /// Scenario: a receiver declares an empty production list and forms a context with no values.
    /// Guarantees: the context is still materialized against the plan, so the declaration is not
    /// conditional on having produced entries.
    #[test]
    fn empty_production_still_materializes() {
        let binding =
            ProductionBinding::compile(&[], request_plan()).expect("empty production compiles");
        let actual = binding.form([]).expect("production forms");
        let mut expected = Metadata::default();
        expected.materialize_context(&request_plan());
        assert_eq!(actual, expected);
    }

    /// Scenario: a production lists the same entry twice.
    /// Guarantees: compilation fails naming the entry as produced twice, rather than producing
    /// an ambiguous handle.
    #[test]
    fn production_rejects_duplicate_entries() {
        let tenant = name("tenant");
        let Err(error) = ProductionBinding::compile(&[tenant.clone(), tenant], request_plan())
        else {
            panic!("duplicate produced entries must be rejected");
        };
        let message = error.to_string();
        assert!(
            message.contains("`tenant` is produced more than once"),
            "{message}"
        );
    }
}
