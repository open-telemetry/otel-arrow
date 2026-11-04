// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bindings that form a processor's output metadata from its input metadata.
//!
//! An edit forms one output from one input. A merge forms one output from
//! several inputs. Both apply declared actions in order: an edit to its
//! input, a merge to the merged output.

use super::{
    ContextAction, ContextDomain, ContextEntryTarget, ContextError, ContextLayoutIdentity,
    ContextMaterializationPlan, ContextNameMatching, MaterializedContextEntry, Metadata,
    PeerAddrMerger,
};
use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{
    TransportHeader, TransportHeaders, ValueKind,
};
use otel_arrow_dfe_config::error::Error;
use std::sync::Arc;

/// Compiled handle for one entry a node declared it produces.
///
/// Obtain once during node construction from [`EditBinding::produced`] or
/// [`MergeBinding::produced`]. A handle records the binding that issued it,
/// so another binding rejects it instead of writing a different entry.
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
    /// Produced entry handle issued by the node's edit or merge binding.
    pub entry: &'a ProducedEntry,
    /// Text or binary representation.
    pub kind: ValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

/// Declared produced entries and the identity carried by their handles.
#[derive(Debug, Clone)]
struct ProducedEntries {
    /// Identity shared by clones and by the handles this binding issues.
    ///
    /// The shared allocation is created once at compilation and cloned only
    /// when a node looks up handles during construction; forming compares
    /// pointers and never touches the reference count.
    identity: ContextLayoutIdentity,
    /// Declared produced transport-header entries, sorted and unique.
    names: Box<[ContextEntryName]>,
}

impl ProducedEntries {
    /// Fails when an entry is added more than once.
    fn new(names: Vec<ContextEntryName>) -> Result<Self, Error> {
        let mut names = names;
        names.sort();
        if let Some(pair) = names.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(Error::InvalidUserConfig {
                error: format!("context entry `{}` is added more than once", pair[0]),
            });
        }
        Ok(Self {
            identity: ContextLayoutIdentity::default(),
            names: names.into_boxed_slice(),
        })
    }

    fn handle(
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
        let index = self.names.binary_search(entry).map_err(|_| undeclared())?;
        Ok(ProducedEntry {
            binding: self.identity.clone(),
            index,
        })
    }

    /// Fails unless this binding issued `entry`.
    fn check(&self, entry: &ProducedEntry) -> Result<(), ContextError> {
        if entry.binding.matches(&self.identity) && entry.index < self.names.len() {
            Ok(())
        } else {
            Err(ContextError::ContextMismatch)
        }
    }
}

/// A compiled edit or merge target: transport headers by name, or one whole composite.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ContextTarget {
    TransportHeader {
        name: ContextEntryName,
        matching: ContextNameMatching,
    },
    Composite(ContextEntryName),
}

impl ContextTarget {
    /// Compiles declared targets in canonical order without duplicates.
    fn compile_all(
        targets: &[ContextEntryTarget],
        plan: &ContextMaterializationPlan,
    ) -> Result<Box<[Self]>, Error> {
        let mut targets = targets.to_vec();
        targets.sort();
        targets.dedup();
        targets
            .iter()
            .map(|target| Self::compile(target, plan))
            .collect()
    }

    fn compile(
        target: &ContextEntryTarget,
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        let invalid = |error: String| Error::InvalidUserConfig { error };
        match target {
            ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name,
                matching,
            } => Ok(Self::TransportHeader {
                name: name.clone(),
                matching: *matching,
            }),
            ContextEntryTarget::Primitive {
                domain: ContextDomain::AuthorizedIdentity,
                name,
                ..
            } => Err(invalid(format!(
                "authorized-identity entry `{name}` cannot be edited or merged; only \
                 transport headers and whole composites can"
            ))),
            ContextEntryTarget::CompositeMember { composite, member } => Err(invalid(format!(
                "composite member `{composite}:{member}` cannot be edited or merged; \
                 composites are atomic, so name the whole composite `{composite}`"
            ))),
            ContextEntryTarget::Composite { name } => {
                _ = plan.layout().resolve_composite(name)?;
                Ok(Self::Composite(name.clone()))
            }
        }
    }

    fn matches_header(&self, header: &str) -> bool {
        match self {
            Self::TransportHeader { name, matching } => matching.matches(header, name.as_str()),
            Self::Composite(_) => false,
        }
    }
}

/// One compiled action, in declaration order.
#[derive(Debug, Clone)]
enum Step {
    /// Appends the produced values whose handle has this index.
    Add {
        index: usize,
        name: ContextEntryName,
    },
    Delete(ContextTarget),
}

/// Compiled `Edits` or `Merges` actions.
#[derive(Debug, Clone)]
struct ContextEdit {
    produced: ProducedEntries,
    steps: Box<[Step]>,
}

impl ContextEdit {
    fn compile(
        actions: &[ContextAction],
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        let produced = ProducedEntries::new(
            actions
                .iter()
                .filter_map(|action| match action {
                    ContextAction::Add { entry } => Some(entry.clone()),
                    ContextAction::Delete { .. } => None,
                })
                .collect(),
        )?;
        let steps = actions
            .iter()
            .map(|action| match action {
                ContextAction::Add { entry } => Ok(Step::Add {
                    index: produced
                        .handle(ContextDomain::TransportHeader, entry)?
                        .index,
                    name: entry.clone(),
                }),
                ContextAction::Delete { target } => {
                    ContextTarget::compile(target, plan).map(Step::Delete)
                }
            })
            .collect::<Result<_, Error>>()?;
        Ok(Self { produced, steps })
    }

    /// Applies each action to `output` in declaration order.
    ///
    /// Added values are appended after the existing headers. Neither
    /// deletions nor added values re-evaluate composites. Fails with
    /// [`ContextError::ContextMismatch`] when a value carries a handle issued
    /// by another binding.
    fn apply(
        &self,
        output: &mut Metadata,
        produced: &[ProducedValue<'_>],
    ) -> Result<(), ContextError> {
        for value in produced {
            self.produced.check(value.entry)?;
        }
        let mut headers: Option<TransportHeaders> = None;
        for step in &self.steps {
            match step {
                Step::Add { index, name } => {
                    let mut values = produced
                        .iter()
                        .filter(|value| value.entry.index == *index)
                        .peekable();
                    if values.peek().is_none() {
                        continue;
                    }
                    let headers = headers
                        .get_or_insert_with(|| output.take_transport_headers().unwrap_or_default());
                    for value in values {
                        headers.push(TransportHeader::new(name.clone(), value.kind, value.bytes));
                    }
                }
                Step::Delete(target @ ContextTarget::TransportHeader { .. }) => {
                    let current = headers
                        .take()
                        .or_else(|| output.take_transport_headers())
                        .unwrap_or_default();
                    let mut retained = TransportHeaders::with_capacity(current.len());
                    for header in current
                        .iter()
                        .filter(|header| !target.matches_header(header.name.as_str()))
                    {
                        retained.push(header.into());
                    }
                    headers = Some(retained);
                }
                Step::Delete(ContextTarget::Composite(name)) => {
                    output.retain_materialized_entries(|entry| entry != name);
                }
            }
        }
        if let Some(headers) = headers {
            output.set_transport_headers(headers);
        }
        Ok(())
    }
}

/// Compiled 1:1 edit for a processor that declares `Edits`.
///
/// The processor forms each output from one input. A processor emitting
/// several outputs per input forms one and clones it.
///
/// Equality is identity: clones of one compiled binding are equal and accept
/// each other's handles, while every other binding is not, even with the
/// same declaration.
#[derive(Debug, Clone)]
pub struct EditBinding {
    edit: ContextEdit,
}

impl PartialEq for EditBinding {
    fn eq(&self, other: &Self) -> bool {
        self.edit
            .produced
            .identity
            .matches(&other.edit.produced.identity)
    }
}

impl Eq for EditBinding {}

impl EditBinding {
    /// Compiles an edit for declared actions.
    pub(super) fn compile(
        actions: &[ContextAction],
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        Ok(Self {
            edit: ContextEdit::compile(actions, plan)?,
        })
    }

    /// Returns the handle for an entry this node declared it adds.
    ///
    /// Call once during node construction with the same entry the node
    /// returned from its context declarations. Only transport-header entries
    /// can be added. Any other request is a mismatch between the node and its
    /// declarations and fails with [`Error::UndeclaredContextEntry`].
    pub fn produced(
        &self,
        domain: ContextDomain,
        entry: &ContextEntryName,
    ) -> Result<ProducedEntry, Error> {
        self.edit.produced.handle(domain, entry)
    }

    /// Clones the input and applies the declared actions in order.
    ///
    /// Each `Add` appends the supplied values for its entry, in slice order,
    /// after the existing headers. Authorized identity and peer address are
    /// retained. Without deletions or produced values the output equals the
    /// input.
    ///
    /// Fails with [`ContextError::ContextMismatch`] when a value carries a
    /// handle issued by another binding.
    pub fn form(
        &self,
        input: &Metadata,
        produced: &[ProducedValue<'_>],
    ) -> Result<Metadata, ContextError> {
        let mut output = input.clone();
        self.edit.apply(&mut output, produced)?;
        Ok(output)
    }
}

/// Compiled N:1 merge for a processor that declares `Merges`.
///
/// Merge applies whenever an output stands for merged inputs, including a
/// merge of exactly one input. An output that forwards one input's context
/// unchanged is not a merge and needs no binding.
///
/// The output keeps a listed entry only when every input carries it with
/// equal values, and the peer address only when every input observed the
/// same peer. Everything else is dropped; the declared actions are then
/// applied to the merged output. Equality is identity, as for [`EditBinding`].
#[derive(Debug, Clone)]
pub struct MergeBinding {
    plan: Arc<MergePlan>,
}

#[derive(Debug)]
struct MergePlan {
    /// Targets kept when all inputs agree, in canonical order.
    keeps: Box<[ContextTarget]>,
    /// Actions applied to the merged output.
    edit: ContextEdit,
}

impl PartialEq for MergeBinding {
    fn eq(&self, other: &Self) -> bool {
        self.plan
            .edit
            .produced
            .identity
            .matches(&other.plan.edit.produced.identity)
    }
}

impl Eq for MergeBinding {}

impl MergeBinding {
    /// Compiles a merge for declared kept entries and actions.
    pub(super) fn compile(
        keeps: &[ContextEntryTarget],
        actions: &[ContextAction],
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        Ok(Self {
            plan: Arc::new(MergePlan {
                keeps: ContextTarget::compile_all(keeps, plan)?,
                edit: ContextEdit::compile(actions, plan)?,
            }),
        })
    }

    /// Returns the handle for an entry this node declared it adds; see
    /// [`EditBinding::produced`].
    pub fn produced(
        &self,
        domain: ContextDomain,
        entry: &ContextEntryName,
    ) -> Result<ProducedEntry, Error> {
        self.plan.edit.produced.handle(domain, entry)
    }

    /// Returns an empty accumulator for inputs observed one at a time.
    #[must_use]
    pub fn merger(&self) -> MetadataMerger {
        MetadataMerger {
            plan: Arc::clone(&self.plan),
            peer: PeerAddrMerger::new(),
            kept: self.plan.keeps.iter().map(|_| Kept::Pending).collect(),
        }
    }

    /// Merges every input into one output.
    ///
    /// Equivalent to pushing each input into [`Self::merger`] and finishing.
    pub fn merge<'a>(
        &self,
        inputs: impl IntoIterator<Item = &'a Metadata>,
        produced: &[ProducedValue<'_>],
    ) -> Result<Metadata, ContextError> {
        let mut merger = self.merger();
        for input in inputs {
            merger.push(input);
        }
        merger.finish(produced)
    }
}

/// Agreement state for one kept target.
#[derive(Debug, Clone)]
enum Kept {
    /// No input pushed yet.
    Pending,
    /// Every input so far carried these transport headers.
    Headers(Vec<TransportHeader>),
    /// Every input so far carried this composite.
    Composite(MaterializedContextEntry),
    /// An input lacked the target or disagreed.
    Erased,
}

impl Kept {
    fn push(&mut self, target: &ContextTarget, input: &Metadata) {
        *self = match (std::mem::replace(self, Self::Erased), target) {
            (Self::Erased, _) => Self::Erased,
            (state, ContextTarget::TransportHeader { .. }) => {
                let values = || {
                    input
                        .transport_headers()
                        .into_iter()
                        .flat_map(|headers| headers.iter())
                        .filter(|header| target.matches_header(header.name.as_str()))
                };
                match state {
                    Self::Pending => {
                        let headers = values().map(TransportHeader::from).collect::<Vec<_>>();
                        if headers.is_empty() {
                            Self::Erased
                        } else {
                            Self::Headers(headers)
                        }
                    }
                    Self::Headers(headers) if values().eq(headers.iter()) => Self::Headers(headers),
                    _ => Self::Erased,
                }
            }
            (state, ContextTarget::Composite(name)) => {
                let entry = input
                    .materialized_context()
                    .and_then(|context| context.entry(name));
                match (state, entry) {
                    (Self::Pending, Some(entry)) => Self::Composite(entry.clone()),
                    (Self::Composite(kept), Some(entry)) if kept == *entry => Self::Composite(kept),
                    _ => Self::Erased,
                }
            }
        };
    }
}

/// Owned accumulator for inputs merged into one output.
///
/// Retains each kept entry and the peer address only when every pushed input
/// agreed on it; see [`MergeBinding`]. Finishing without inputs applies the
/// actions to empty metadata.
#[derive(Debug, Clone)]
pub struct MetadataMerger {
    plan: Arc<MergePlan>,
    peer: PeerAddrMerger,
    /// Agreement state, one per kept target.
    kept: Box<[Kept]>,
}

impl MetadataMerger {
    /// Folds one contributing input into the running merge.
    pub fn push(&mut self, input: &Metadata) {
        self.peer.push(input.peer_addr());
        for (target, kept) in self.plan.keeps.iter().zip(self.kept.iter_mut()) {
            kept.push(target, input);
        }
    }

    /// Returns the merged metadata with the declared actions applied.
    ///
    /// Fails with [`ContextError::ContextMismatch`] when a value carries a
    /// handle issued by another binding. The merged metadata must also meet
    /// the node's context requirements, for example an entry listed in its
    /// `required_out` policy; `required_out` is not yet enforced.
    pub fn finish(self, produced: &[ProducedValue<'_>]) -> Result<Metadata, ContextError> {
        let mut headers = TransportHeaders::new();
        let mut entries = Vec::new();
        for kept in self.kept {
            match kept {
                Kept::Headers(values) => values.into_iter().for_each(|header| headers.push(header)),
                Kept::Composite(entry) => entries.push(entry),
                Kept::Pending | Kept::Erased => {}
            }
        }
        let mut output = Metadata::default();
        output.set_transport_headers(headers);
        output.set_materialized_entries(entries);
        if let Some(peer) = self.peer.finish() {
            output.set_peer_addr(peer);
        }
        self.plan.edit.apply(&mut output, produced)?;
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
        EditBinding::compile(&actions, &ContextMaterializationPlan::default())
            .expect("edit compiles")
    }

    fn add(entry: ContextEntryName) -> ContextAction {
        ContextAction::Add { entry }
    }

    fn delete(target: ContextEntryTarget) -> ContextAction {
        ContextAction::Delete { target }
    }

    /// Returns the header names in order.
    fn header_names(metadata: &Metadata) -> Vec<String> {
        metadata
            .transport_headers()
            .map(|headers| {
                headers
                    .iter()
                    .map(|header| header.name.as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Transport-header target matched exactly.
    fn header(value: &str) -> ContextEntryTarget {
        ContextEntryTarget::Primitive {
            domain: ContextDomain::TransportHeader,
            name: name(value),
            matching: ContextNameMatching::Exact,
        }
    }

    /// Whole-composite target.
    fn composite(value: &str) -> ContextEntryTarget {
        ContextEntryTarget::Composite { name: name(value) }
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

    /// Scenario: partition-style edit appends text and binary values to empty and populated inputs.
    /// Guarantees: output equals the partition processor's header update, retaining identity, peer, and composites.
    #[test]
    fn edit_matches_partition_header_update() {
        let partition = name("partition");
        let shard = name("shard");
        let binding = edit(&[partition.clone(), shard.clone()]);
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
                .expect("edit forms");
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

    /// Scenario: an edit without deletions is formed without any produced values.
    /// Guarantees: the output equals the input, including absent transport headers.
    #[test]
    fn edit_without_values_preserves_input() {
        let binding = edit(&[name("partition")]);
        for input in [
            Metadata::default(),
            populated_metadata(Some(peer("10.0.0.1:1"))),
        ] {
            assert_eq!(binding.form(&input, &[]), Ok(input));
        }
        assert_eq!(
            binding
                .form(&Metadata::default(), &[])
                .expect("edit forms")
                .transport_headers()
                .map(|headers| headers.len()),
            None
        );
    }

    /// Scenario: construction-time lookups name an undeclared transport header or any authorized-identity entry.
    /// Guarantees: both fail with UndeclaredContextEntry naming the domain and entry instead of issuing a handle.
    #[test]
    fn produced_lookup_rejects_undeclared_entries() {
        let binding = edit(&[name("partition")]);
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
    fn edit_rejects_mismatched_handles() {
        let partition = name("partition");
        let binding = edit(std::slice::from_ref(&partition));
        fn value(entry: &ProducedEntry) -> ProducedValue<'_> {
            ProducedValue {
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
                .produced(
                    ContextDomain::TransportHeader,
                    &other.edit.produced.names[0],
                )
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

    /// Scenario: batch-style merges with no keeps see no inputs, one input, unanimous peers, distinct peers, or a peerless input.
    /// Guarantees: output equals empty metadata plus the PeerAddrMerger result, and the incremental merger agrees.
    #[test]
    fn merge_matches_peer_addr_merger() {
        let a = Some(peer("10.0.0.1:1"));
        let b = Some(peer("10.0.0.2:2"));
        let binding = MergeBinding::compile(&[], &[], &ContextMaterializationPlan::default())
            .expect("merge compiles");
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

            let merged = binding.merge(&inputs, &[]).expect("merge forms");
            assert_eq!(merged, expected, "inputs {peers:?}");
            assert!(merged.transport_headers().is_none());
            assert!(merged.authorized_identity_entries().is_none());
            assert!(merged.materialized_context().is_none());

            let mut merger = binding.merger();
            for input in &inputs {
                merger.push(input);
            }
            assert_eq!(
                merger.finish(&[]),
                Ok(expected),
                "incremental inputs {peers:?}"
            );
        }
    }

    /// Scenario: an edit deletes the `tenant` header and the `request` composite and adds `partition`.
    /// Guarantees: only the deleted entries are removed, the added header follows the remaining headers, and identity and peer are retained.
    #[test]
    fn edit_deletes_headers_and_composites() {
        let partition = name("partition");
        let binding = EditBinding::compile(
            &[
                delete(header("tenant")),
                delete(composite("request")),
                add(partition.clone()),
            ],
            &request_plan(),
        )
        .expect("edit compiles");
        let entry = binding
            .produced(ContextDomain::TransportHeader, &partition)
            .expect("partition handle");
        let input = populated_metadata(Some(peer("10.0.0.1:1")));

        let output = binding
            .form(
                &input,
                &[ProducedValue {
                    entry: &entry,
                    kind: ValueKind::Text,
                    bytes: b"0",
                }],
            )
            .expect("edit forms");
        let headers = output.transport_headers().expect("added header");
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers.iter().next().map(|h| h.name.as_str()),
            Some("partition")
        );
        assert!(output.materialized_context().is_none());
        assert_eq!(output.peer_addr(), input.peer_addr());
        assert_eq!(
            output.authorized_identity_entries(),
            input.authorized_identity_entries()
        );

        let deleted = binding.form(&input, &[]).expect("edit forms");
        assert!(deleted.transport_headers().is_none());
        assert!(deleted.materialized_context().is_none());
    }

    /// Scenario: a merge keeping `tenant` and `request` sees agreeing, conflicting, partially absent, and single inputs.
    /// Guarantees: a kept entry survives only when every input carries it with equal values; added values follow kept headers.
    #[test]
    fn merge_keeps_only_agreeing_entries() {
        let partition = name("partition");
        let binding = MergeBinding::compile(
            &[header("tenant"), composite("request")],
            &[add(partition.clone())],
            &request_plan(),
        )
        .expect("merge compiles");
        let entry = binding
            .produced(ContextDomain::TransportHeader, &partition)
            .expect("partition handle");
        let added = [ProducedValue {
            entry: &entry,
            kind: ValueKind::Text,
            bytes: b"0",
        }];
        let acme = tenant_metadata(b"acme", None);
        let other = tenant_metadata(b"other", None);
        let empty = Metadata::default();
        let kept = |inputs: &[&Metadata]| {
            let merged = binding
                .merge(inputs.iter().copied(), &added)
                .expect("merge forms");
            let headers = header_names(&merged);
            let composites = merged
                .materialized_context()
                .map_or(0, |context| context.entries().len());
            assert!(merged.authorized_identity_entries().is_none());
            (headers, composites)
        };
        let both = (vec!["tenant".to_owned(), "partition".to_owned()], 1);
        let neither = (vec!["partition".to_owned()], 0);

        assert_eq!(kept(&[&acme]), both);
        assert_eq!(kept(&[&acme, &acme, &acme]), both);
        assert_eq!(kept(&[&acme, &other]), neither);
        assert_eq!(kept(&[&acme, &empty]), neither);
        assert_eq!(kept(&[&empty, &acme]), neither);
        assert_eq!(kept(&[]), neither);
        let merged = binding.merge([&acme, &acme], &[]).expect("merge forms");
        let tenant = merged
            .transport_headers()
            .and_then(|headers| headers.find_by_name("tenant").next())
            .expect("kept tenant");
        assert_eq!(tenant.value.bytes, b"acme");
        assert_eq!(tenant.wire_name(), "X-Tenant");
    }

    /// Scenario: edit and merge targets name an identity primitive, a composite member, or an unknown composite.
    /// Guarantees: compilation fails instead of partially editing atomic composites or packed identity entries.
    #[test]
    fn unsupported_targets_are_rejected_at_compilation() {
        let plan = request_plan();
        for (target, expected) in [
            (
                ContextEntryTarget::Primitive {
                    domain: ContextDomain::AuthorizedIdentity,
                    name: name("subject"),
                    matching: ContextNameMatching::Exact,
                },
                "authorized-identity entry `subject`",
            ),
            (
                ContextEntryTarget::CompositeMember {
                    composite: name("request"),
                    member: name("route"),
                },
                "composite member `request:route`",
            ),
            (composite("missing"), "missing"),
        ] {
            for error in [
                EditBinding::compile(&[delete(target.clone())], &plan).err(),
                MergeBinding::compile(std::slice::from_ref(&target), &[], &plan).err(),
                MergeBinding::compile(&[], &[delete(target.clone())], &plan).err(),
            ] {
                let message = error.expect("target is rejected").to_string();
                assert!(message.contains(expected), "{message}");
            }
        }
    }

    /// Scenario: edits order Delete and Add of `tenant` both ways, and a merge deletes a kept entry.
    /// Guarantees: actions apply in declaration order, and merge actions apply to the merged output.
    #[test]
    fn actions_apply_in_declaration_order() {
        let tenant = name("tenant");
        let value = |entry| ProducedValue {
            entry,
            kind: ValueKind::Text,
            bytes: b"new",
        };
        let input = tenant_metadata(b"old", None);
        let plan = request_plan();
        let tenant_values = |metadata: &Metadata| {
            metadata
                .transport_headers()
                .into_iter()
                .flat_map(|headers| headers.find_by_name("tenant"))
                .map(|header| header.value.bytes.to_vec())
                .collect::<Vec<_>>()
        };

        let replace = EditBinding::compile(&[delete(header("tenant")), add(tenant.clone())], &plan)
            .expect("edit compiles");
        let entry = replace
            .produced(ContextDomain::TransportHeader, &tenant)
            .expect("tenant handle");
        let output = replace.form(&input, &[value(&entry)]).expect("edit forms");
        assert_eq!(tenant_values(&output), [b"new".to_vec()]);

        let remove = EditBinding::compile(&[add(tenant.clone()), delete(header("tenant"))], &plan)
            .expect("edit compiles");
        let entry = remove
            .produced(ContextDomain::TransportHeader, &tenant)
            .expect("tenant handle");
        let output = remove.form(&input, &[value(&entry)]).expect("edit forms");
        assert!(tenant_values(&output).is_empty());

        let merge = MergeBinding::compile(
            &[header("tenant"), composite("request")],
            &[delete(composite("request"))],
            &plan,
        )
        .expect("merge compiles");
        let merged = merge.merge([&input, &input], &[]).expect("merge forms");
        assert_eq!(tenant_values(&merged), [b"old".to_vec()]);
        assert!(merged.materialized_context().is_none());
    }

    /// Scenario: an edit or merge declares the same Add entry twice.
    /// Guarantees: compilation fails naming the entry instead of silently collapsing the actions.
    #[test]
    fn duplicate_adds_are_rejected_at_compilation() {
        let plan = ContextMaterializationPlan::default();
        let actions = [add(name("partition")), add(name("partition"))];
        for error in [
            EditBinding::compile(&actions, &plan).err(),
            MergeBinding::compile(&[], &actions, &plan).err(),
        ] {
            let message = error.expect("duplicate add is rejected").to_string();
            assert!(
                message.contains("`partition` is added more than once"),
                "{message}"
            );
        }
    }
}
