// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read adapters retain the existing lookup and materialization semantics.

use super::metadata::{AuthorizedClaimValue, AuthorizedIdentityEntry};
use super::{
    ContextDomain, ContextEntrySelector, ContextEntryTarget, ContextMaterializationPlan,
    ContextNameMatching, ContextProjection, ContextValueCardinality, ContextValueKind,
    MaterializedContextProjection, MaterializedContextValue, Metadata,
};
use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context::transport_headers::{
    TransportHeaderRef, TransportHeadersFindIter, TransportHeadersIter, ValueKind,
};
use otel_arrow_dfe_config::error::Error;

/// One declared named read, resolved once by the context compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueRead {
    selector: ContextEntrySelector,
    backend: ValueReadBackend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Composite reads retain a shared plan and projection without another heap allocation.
#[allow(variant_size_differences)]
enum ValueReadBackend {
    TransportHeader {
        name: ContextEntryName,
        matching: ContextNameMatching,
    },
    AuthorizedIdentity {
        name: ContextEntryName,
        matching: ContextNameMatching,
    },
    Composite {
        plan: ContextMaterializationPlan,
        projection: ContextProjection,
    },
}

impl ContextNameMatching {
    fn matches(self, stored: &str, configured: &str) -> bool {
        match self {
            Self::Exact => stored == configured,
            Self::AsciiCaseInsensitive => stored.eq_ignore_ascii_case(configured),
        }
    }
}

impl ValueRead {
    pub(super) fn compile(
        selector: &ContextEntrySelector,
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        let backend = match &selector.target {
            ContextEntryTarget::Primitive {
                domain,
                name,
                matching,
            } => {
                let (name, matching) = (name.clone(), *matching);
                match domain {
                    ContextDomain::TransportHeader => {
                        ValueReadBackend::TransportHeader { name, matching }
                    }
                    ContextDomain::AuthorizedIdentity => {
                        ValueReadBackend::AuthorizedIdentity { name, matching }
                    }
                }
            }
            ContextEntryTarget::CompositeMember { composite, member } => {
                ValueReadBackend::Composite {
                    plan: plan.clone(),
                    projection: plan.layout().resolve_member(composite, member)?,
                }
            }
            ContextEntryTarget::Composite { name } => ValueReadBackend::Composite {
                plan: plan.clone(),
                projection: plan.layout().resolve_composite(name)?,
            },
        };
        Ok(Self {
            selector: selector.clone(),
            backend,
        })
    }

    /// Returns the declared selector this read was compiled from.
    #[must_use]
    pub fn selector(&self) -> &ContextEntrySelector {
        &self.selector
    }

    /// Applies a compiled stored-name lookup or composite projection without allocating.
    ///
    /// Missing values return `None`. An identity claim read retains the claim's
    /// cardinality, so an empty multi-valued claim is present. A composite read
    /// retains the whole entry's presence gate and source cardinalities. Each
    /// member yields at most the number of values the selector's `take` requests.
    #[must_use]
    pub fn read<'a>(&'a self, metadata: &'a Metadata) -> Option<Read<'a>> {
        let backend = match &self.backend {
            ValueReadBackend::TransportHeader { name, matching } => {
                let headers = metadata.transport_headers()?;
                let name = name.as_str();
                let mut remaining = match matching {
                    ContextNameMatching::Exact => HeaderMatches::Exact(headers.find_by_name(name)),
                    ContextNameMatching::AsciiCaseInsensitive => {
                        HeaderMatches::AsciiCaseInsensitive {
                            headers: headers.iter(),
                            name,
                        }
                    }
                };
                ReadBackend::Headers {
                    first: remaining.next()?,
                    remaining,
                }
            }
            ValueReadBackend::AuthorizedIdentity { name, matching } => {
                let name = name.as_str();
                ReadBackend::AuthorizedIdentity(
                    metadata
                        .authorized_identity_entries()?
                        .iter()
                        .find(|entry| matching.matches(entry.name(), name))?,
                )
            }
            ValueReadBackend::Composite { plan, projection } => {
                ReadBackend::Composite(metadata.resolve_materialized_context(plan, projection)?)
            }
        };
        Some(Read {
            backend,
            limit: self.selector.take.limit(),
        })
    }
}

/// A present borrowed primitive or composite selection.
pub struct Read<'a> {
    backend: ReadBackend<'a>,
    limit: usize,
}

enum ReadBackend<'a> {
    Headers {
        first: TransportHeaderRef<'a>,
        remaining: HeaderMatches<'a>,
    },
    AuthorizedIdentity(AuthorizedIdentityEntry<'a>),
    Composite(MaterializedContextProjection<'a>),
}

impl<'a> Read<'a> {
    /// Visits selected members in existing projection order, including empty multi-valued members.
    ///
    /// Primitive headers form one multi-valued member; duplicate values retain
    /// capture order. Names and original wire names remain available regardless
    /// of the requested selector form. Values are limited by the selector's `take`.
    pub fn visit_members(self, mut visit: impl FnMut(ReadMember<'a>)) {
        let limit = self.limit;
        match self.backend {
            ReadBackend::Headers { first, remaining } => visit(ReadMember {
                name: first.name.as_str(),
                cardinality: ContextValueCardinality::Many,
                values: ReadValues::Headers {
                    first: Some(first),
                    remaining,
                },
                limit,
            }),
            ReadBackend::AuthorizedIdentity(entry) => visit(ReadMember::identity(entry, limit)),
            ReadBackend::Composite(projection) => projection.visit_members(|member| {
                visit(ReadMember {
                    name: member.name().as_str(),
                    cardinality: member.values().cardinality(),
                    values: ReadValues::Materialized {
                        name: member.name().as_str(),
                        values: member.values().values().iter(),
                    },
                    limit,
                });
            }),
        }
    }
}

/// One selected member and its borrowed values.
pub struct ReadMember<'a> {
    /// Stored primitive name or configured composite member alias.
    pub name: &'a str,
    /// Source cardinality, including explicitly empty multi-valued claims.
    pub cardinality: ContextValueCardinality,
    values: ReadValues<'a>,
    limit: usize,
}

impl<'a> ReadMember<'a> {
    fn identity(entry: AuthorizedIdentityEntry<'a>, limit: usize) -> Self {
        let claim = entry.value();
        Self {
            name: entry.name(),
            cardinality: if claim.is_many() {
                ContextValueCardinality::Many
            } else {
                ContextValueCardinality::One
            },
            values: ReadValues::AuthorizedIdentity {
                name: entry.name(),
                claim,
                next: 0,
            },
            limit,
        }
    }

    /// Iterates raw values without flattening, joining, or decoding them.
    ///
    /// Yields no values for `Exists`, at most one for `First`, and all for `All`;
    /// unrequested values are never scanned.
    pub fn values(self) -> impl Iterator<Item = ReadValue<'a>> {
        self.values.take(self.limit)
    }
}

/// A borrowed value and its retained names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ReadValue<'a> {
    /// Stored primitive name or configured composite member alias.
    pub name: &'a str,
    /// Original transport wire name, when retained.
    pub original_name: Option<&'a str>,
    /// Text or binary representation.
    pub kind: ContextValueKind,
    /// Raw value bytes.
    pub bytes: &'a [u8],
}

impl std::fmt::Debug for ReadValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadValue")
            .field("name", &self.name)
            .field("original_name", &self.original_name)
            .field("kind", &self.kind)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl<'a> From<TransportHeaderRef<'a>> for ReadValue<'a> {
    fn from(header: TransportHeaderRef<'a>) -> Self {
        Self {
            name: header.name.as_str(),
            original_name: header.value.original_name,
            kind: match header.value.value_kind {
                ValueKind::Text => ContextValueKind::Text,
                ValueKind::Binary => ContextValueKind::Binary,
            },
            bytes: header.value.bytes,
        }
    }
}

/// A borrowed stored transport header returned by an all-captured read.
pub type HeaderValue<'a> = ReadValue<'a>;

enum HeaderMatches<'a> {
    Exact(TransportHeadersFindIter<'a>),
    AsciiCaseInsensitive {
        headers: TransportHeadersIter<'a>,
        name: &'a str,
    },
}

impl<'a> Iterator for HeaderMatches<'a> {
    type Item = TransportHeaderRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Exact(headers) => headers.next(),
            Self::AsciiCaseInsensitive { headers, name } => {
                headers.find(|header| header.name.as_str().eq_ignore_ascii_case(name))
            }
        }
    }
}

enum ReadValues<'a> {
    Headers {
        first: Option<TransportHeaderRef<'a>>,
        remaining: HeaderMatches<'a>,
    },
    AuthorizedIdentity {
        name: &'a str,
        claim: AuthorizedClaimValue<'a>,
        next: usize,
    },
    Materialized {
        name: &'a str,
        values: std::slice::Iter<'a, MaterializedContextValue>,
    },
}

impl<'a> Iterator for ReadValues<'a> {
    type Item = ReadValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Headers { first, remaining } => {
                first.take().or_else(|| remaining.next()).map(Into::into)
            }
            Self::AuthorizedIdentity { name, claim, next } => {
                let value = claim.get(*next)?;
                *next += 1;
                Some(ReadValue {
                    name,
                    original_name: None,
                    kind: ContextValueKind::Text,
                    bytes: value.as_bytes(),
                })
            }
            Self::Materialized { name, values } => values.next().map(|value| ReadValue {
                name,
                original_name: value.original_name(),
                kind: value.kind(),
                bytes: value.bytes(),
            }),
        }
    }
}

/// A declared all-captured read over existing transport-header storage.
///
/// Obtain this binding from [`super::NodeBindings::headers`].
///
/// Scenario: a node tries to construct an all-captured header read directly.
/// Guarantees: nodes cannot create this binding without a compiled declaration.
/// ```compile_fail
/// use otel_arrow_dfe_engine::pdata_context::HeadersRead;
///
/// let _ = HeadersRead;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadersRead(());

impl HeadersRead {
    pub(super) const fn new() -> Self {
        Self(())
    }

    /// Iterates stored headers in capture order, preserving duplicates and binary bytes.
    pub fn read<'a>(&self, metadata: &'a Metadata) -> impl Iterator<Item = HeaderValue<'a>> {
        metadata
            .transport_headers()
            .into_iter()
            .flat_map(|headers| headers.iter())
            .map(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::auth::{AuthorizedIdentity, ClaimValue};
    use crate::pdata_context::{
        Arrival, ArrivalHeaders, CompiledContextBindings, ContextConsumerSelector,
        ContextDeclaration, ContextEntrySelectorForm, ContextEntryTake, ContextRuntimeRequirements,
        NodeBindings, NodeContextDeclarations, PreparedNodeContextDeclarations,
    };
    use otel_arrow_dfe_config::context::ContextEntryName;
    use otel_arrow_dfe_config::context::policy::{
        ContextEntryDeclaration, ContextEntryDefinition, ContextEntryPart, ContextScope,
    };
    use otel_arrow_dfe_config::context::transport_headers::{TransportHeader, TransportHeaders};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid name")
    }

    fn primitive(value: &str, matching: ContextNameMatching) -> ContextEntrySelector {
        ContextEntrySelector {
            target: ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name: name(value),
                matching,
            },
            form: ContextEntrySelectorForm::Value,
            take: ContextEntryTake::All,
        }
    }

    fn compile(
        declarations: NodeContextDeclarations,
        composites: &[ContextEntryDeclaration],
    ) -> Arc<NodeBindings> {
        let pipeline = otel_arrow_dfe_config::PipelineKey::new("group".into(), "pipeline".into());
        let node = otel_arrow_dfe_config::NodeId::from("node");
        let prepared = HashMap::from([(
            pipeline.clone(),
            HashMap::from([(
                node.clone(),
                PreparedNodeContextDeclarations::new(declarations, composites)
                    .expect("valid declarations"),
            )]),
        )]);
        let requirements = ContextRuntimeRequirements::compile(&prepared);
        CompiledContextBindings::compile(prepared, &requirements)
            .expect("bindings compile")
            .node_bindings(&pipeline, &node)
            .expect("node bindings")
            .clone()
    }

    fn consumes(entries: Vec<ContextEntrySelector>) -> ContextDeclaration {
        ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::Entries {
                entries: entries.into_boxed_slice(),
            },
        }
    }

    fn values<'a>(read: Option<Read<'a>>) -> Vec<ReadValue<'a>> {
        let mut result = Vec::new();
        if let Some(read) = read {
            read.visit_members(|member| result.extend(member.values()));
        }
        result
    }

    /// Scenario: one repeated header is declared with `Exists`, `First`, and `All` takes.
    /// Guarantees: every take reports the same presence, and members yield no values, only the
    /// first value, or every value in capture order, respectively.
    #[test]
    fn take_limits_values_without_changing_presence() {
        let selector = |take| ContextEntrySelector {
            target: ContextEntryTarget::Primitive {
                domain: ContextDomain::TransportHeader,
                name: name("tenant"),
                matching: ContextNameMatching::AsciiCaseInsensitive,
            },
            form: ContextEntrySelectorForm::Value,
            take,
        };
        let takes = [
            ContextEntryTake::Exists,
            ContextEntryTake::First,
            ContextEntryTake::All,
        ];
        let bindings = compile(
            [consumes(takes.into_iter().map(selector).collect())]
                .into_iter()
                .collect(),
            &[],
        );
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(name("Tenant"), b"first"));
        headers.push(TransportHeader::text(name("other"), b"ignored"));
        headers.push(TransportHeader::text(name("tenant"), b"second"));
        let mut metadata = Metadata::default();
        metadata.set_transport_headers(headers);
        let expected: [&[&[u8]]; 3] = [&[], &[b"first"], &[b"first", b"second"]];
        for (take, expected) in takes.into_iter().zip(expected) {
            let read = bindings.value(&selector(take)).expect("declared read");
            assert!(read.read(&Metadata::default()).is_none());
            assert!(read.read(&metadata).is_some());
            let bytes = values(read.read(&metadata))
                .into_iter()
                .map(|value| value.bytes)
                .collect::<Vec<_>>();
            assert_eq!(bytes, expected, "{take:?}");
        }
    }

    /// Scenario: consumers declare exact and case-insensitive reads of repeated mixed-case headers.
    /// Guarantees: compiled reads match legacy lookup order, spelling, kinds, bytes, and absence.
    #[test]
    fn primitive_reads_preserve_matching_and_values() {
        let declared = [
            ("Topic", ContextNameMatching::Exact),
            ("Topic", ContextNameMatching::AsciiCaseInsensitive),
            ("missing", ContextNameMatching::Exact),
        ];
        let bindings = compile(
            [consumes(
                declared
                    .iter()
                    .map(|(entry, matching)| primitive(entry, *matching))
                    .collect(),
            )]
            .into_iter()
            .collect(),
            &[],
        );
        let mut headers = TransportHeaders::new();
        for (stored, wire, kind, value) in [
            ("topic", "X-Topic", ValueKind::Text, b"first".as_slice()),
            ("Topic", "X-Topic", ValueKind::Binary, b"\xff\0".as_slice()),
            ("other", "Other", ValueKind::Text, b"ignored".as_slice()),
            ("Topic", "X-Topic", ValueKind::Text, b"last".as_slice()),
        ] {
            headers.push(TransportHeader::captured(
                name(stored),
                wire,
                true,
                kind,
                value,
            ));
        }
        let mut metadata = Metadata::default();
        metadata.set_transport_headers(headers);
        let headers = metadata.transport_headers().expect("headers");
        for (entry, matching) in declared {
            let read = bindings
                .value(&primitive(entry, matching))
                .expect("declared read");
            let expected: Vec<ReadValue<'_>> = match matching {
                ContextNameMatching::Exact => headers.find_by_name(entry).map(Into::into).collect(),
                ContextNameMatching::AsciiCaseInsensitive => headers
                    .iter()
                    .filter(|header| header.name.as_str().eq_ignore_ascii_case("Topic"))
                    .map(Into::into)
                    .collect(),
            };
            assert_eq!(values(read.read(&metadata)), expected);
            assert_eq!(read.read(&metadata).is_some(), !expected.is_empty());
            assert!(read.read(&Metadata::default()).is_none());
        }
    }

    /// Scenario: all-captured reads see captured headers and a subsequently appended header.
    /// Guarantees: the adapter matches legacy iteration without sorting, deduplication, or composites.
    #[test]
    fn all_captured_matches_capture_and_overlay_iteration() {
        let bindings = compile(
            [
                ContextDeclaration::HeaderCapture {
                    policy: serde_json::from_value(serde_json::json!({
                        "headers": [
                            {"match_names": ["x-tenant"], "store_as": "Tenant"},
                            {"match_names": ["x-token-bin"], "store_as": "Token"}
                        ]
                    }))
                    .expect("capture policy"),
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllCaptured {
                        domain: ContextDomain::TransportHeader,
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllCaptured {
                        domain: ContextDomain::AuthorizedIdentity,
                    },
                },
            ]
            .into_iter()
            .collect(),
            &[],
        );
        let (mut metadata, _) = bindings.arrival().expect("arrival").form(&Arrival {
            headers: ArrivalHeaders::Pairs(&[
                ("x-tenant", b"first"),
                ("x-token-bin", b"\xff\0"),
                ("x-tenant", b"last"),
            ]),
            identity: None,
            peer: None,
        });
        let mut headers = metadata.take_transport_headers().expect("captured headers");
        headers.push(TransportHeader::text(name("partition"), b"produced"));
        metadata.set_transport_headers(headers);
        let read = bindings.headers().expect("all captured read");
        let expected = metadata
            .transport_headers()
            .expect("headers")
            .iter()
            .map(ReadValue::from)
            .collect::<Vec<_>>();
        assert_eq!(read.read(&metadata).collect::<Vec<_>>(), expected);
        assert_eq!(expected.len(), 4);
        assert_eq!(read.read(&Metadata::default()).count(), 0);
    }

    /// Scenario: composite reads select aliases, identity claims, and constants from one arrival.
    /// Guarantees: compiled projections match the legacy resolver, including empty-many cardinality.
    #[test]
    fn composite_reads_match_projections_and_presence_gates() {
        let composite = ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("request"),
            definition: ContextEntryDefinition(vec![
                ContextEntryPart::TransportHeader {
                    name: name("tenant"),
                    store_as: Some(name("customer")),
                },
                ContextEntryPart::AuthorizedIdentity {
                    name: name("roles"),
                    store_as: None,
                },
                ContextEntryPart::Constant {
                    name: name("route"),
                    value: "fixed".into(),
                },
                ContextEntryPart::TransportHeaderMatch {
                    name: name("environment"),
                    value: "production".into(),
                },
            ]),
        };
        let selectors = vec![
            ContextEntrySelector {
                target: ContextEntryTarget::Composite {
                    name: name("request"),
                },
                form: ContextEntrySelectorForm::Value,
                take: ContextEntryTake::All,
            },
            ContextEntrySelector {
                target: ContextEntryTarget::CompositeMember {
                    composite: name("request"),
                    member: name("customer"),
                },
                form: ContextEntrySelectorForm::OriginalKeyValue,
                take: ContextEntryTake::All,
            },
        ];
        let bindings = compile(
            [
                consumes(selectors.clone()),
                ContextDeclaration::HeaderCapture {
                    policy: serde_json::from_value(serde_json::json!({
                        "headers": [
                            {"match_names": ["x-tenant"], "store_as": "tenant"},
                            {"match_names": ["environment"]}
                        ]
                    }))
                    .expect("capture policy"),
                },
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: serde_json::from_value(serde_json::json!([
                        {"claim": "roles", "store_as": "roles"}
                    ]))
                    .expect("identity policy"),
                },
            ]
            .into_iter()
            .collect(),
            std::slice::from_ref(&composite),
        );
        let identity = AuthorizedIdentity::new().with_claim("roles", ClaimValue::Many(vec![]));
        let arrival = Arrival {
            headers: ArrivalHeaders::Pairs(&[
                ("x-tenant", b"acme"),
                ("environment", b"production"),
            ]),
            identity: Some(&identity),
            peer: None,
        };
        let binding = bindings.arrival().expect("arrival");
        let (metadata, _) = binding.form(&arrival);
        for selector in &selectors {
            let read = bindings.value(selector).expect("declared read");
            let projection = match &selector.target {
                ContextEntryTarget::Composite { name } => {
                    binding.materialization.layout().resolve_composite(name)
                }
                ContextEntryTarget::CompositeMember { composite, member } => binding
                    .materialization
                    .layout()
                    .resolve_member(composite, member),
                _ => panic!("composite read"),
            }
            .expect("legacy projection");
            let mut expected = Vec::new();
            metadata
                .resolve_materialized_context(&binding.materialization, &projection)
                .expect("legacy read")
                .visit_members(|member| {
                    expected.push((
                        member.name().as_str(),
                        member.values().cardinality(),
                        member
                            .values()
                            .values()
                            .iter()
                            .map(|value| ReadValue {
                                name: member.name().as_str(),
                                original_name: value.original_name(),
                                kind: value.kind(),
                                bytes: value.bytes(),
                            })
                            .collect::<Vec<_>>(),
                    ));
                });
            let mut actual = Vec::new();
            read.read(&metadata)
                .expect("adapter read")
                .visit_members(|member| {
                    actual.push((
                        member.name,
                        member.cardinality,
                        member.values().collect::<Vec<_>>(),
                    ));
                });
            assert_eq!(actual, expected);
            if matches!(selector.target, ContextEntryTarget::Composite { .. }) {
                assert!(actual.iter().any(|(name, cardinality, values)| {
                    *name == "roles"
                        && *cardinality == ContextValueCardinality::Many
                        && values.is_empty()
                }));
            } else {
                assert_eq!(
                    actual.iter().map(|(name, ..)| *name).collect::<Vec<_>>(),
                    ["customer"]
                );
            }

            let (missing, _) = binding.form(&Arrival {
                headers: ArrivalHeaders::Pairs(&[
                    ("x-tenant", b"acme"),
                    ("environment", b"production"),
                ]),
                identity: None,
                peer: None,
            });
            assert!(read.read(&missing).is_none());
            let (missing, _) = binding.form(&Arrival {
                headers: ArrivalHeaders::Pairs(&[]),
                identity: Some(&identity),
                peer: None,
            });
            assert!(read.read(&missing).is_none());
            let (mismatch, _) = binding.form(&Arrival {
                headers: ArrivalHeaders::Pairs(&[
                    ("x-tenant", b"acme"),
                    ("environment", b"development"),
                ]),
                identity: Some(&identity),
                peer: None,
            });
            assert!(read.read(&mismatch).is_none());

            let mut changed = composite.clone();
            changed.definition.0.push(ContextEntryPart::Constant {
                name: name("new_member"),
                value: "new".into(),
            });
            let changed_plan =
                ContextMaterializationPlan::compile(&[changed]).expect("changed materialization");
            let changed_read =
                ValueRead::compile(read.selector(), &changed_plan).expect("changed read");
            assert!(changed_read.read(&metadata).is_none());
        }
    }

    /// Scenario: a node reads identity claims by name alongside an all-captured header read.
    /// Guarantees: identity reads return captured claim values with their cardinality, never
    /// header values, and the all-captured header read never yields identity claims.
    #[test]
    fn identity_reads_return_captured_claims() {
        let identity_read = |value: &str, matching| ContextEntrySelector {
            target: ContextEntryTarget::Primitive {
                domain: ContextDomain::AuthorizedIdentity,
                name: name(value),
                matching,
            },
            form: ContextEntrySelectorForm::Value,
            take: ContextEntryTake::All,
        };
        let bindings = compile(
            [
                consumes(vec![
                    identity_read("subject", ContextNameMatching::Exact),
                    identity_read("Subject", ContextNameMatching::AsciiCaseInsensitive),
                    identity_read("Subject", ContextNameMatching::Exact),
                    identity_read("roles", ContextNameMatching::Exact),
                ]),
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllCaptured {
                        domain: ContextDomain::TransportHeader,
                    },
                },
            ]
            .into_iter()
            .collect(),
            &[],
        );
        let mut metadata = Metadata::default();
        let policy = serde_json::from_value(serde_json::json!([
            {"claim": "sub", "store_as": "subject"},
            {"claim": "roles", "store_as": "roles"},
            {"claim": "groups", "store_as": "groups"}
        ]))
        .expect("identity policy");
        metadata.capture_authorized_identity(
            &policy,
            &AuthorizedIdentity::new()
                .with_subject("verified")
                .with_claim("roles", ClaimValue::Many(vec![]))
                .with_claim("groups", ClaimValue::Many(vec!["a".into(), "b".into()])),
        );
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(name("subject"), b"untrusted"));
        metadata.set_transport_headers(headers);

        type Member<'a> = (
            &'a str,
            ContextValueCardinality,
            Vec<(ContextValueKind, &'a [u8])>,
        );
        fn members(read: Option<Read<'_>>) -> Vec<Member<'_>> {
            let mut result = Vec::new();
            if let Some(read) = read {
                read.visit_members(|member| {
                    result.push((
                        member.name,
                        member.cardinality,
                        member
                            .values()
                            .map(|value| (value.kind, value.bytes))
                            .collect::<Vec<_>>(),
                    ));
                });
            }
            result
        }
        let subject = vec![(
            "subject",
            ContextValueCardinality::One,
            vec![(ContextValueKind::Text, b"verified".as_slice())],
        )];
        let read = |value, matching| {
            bindings
                .value(&identity_read(value, matching))
                .expect("declared read")
        };
        let exact = read("subject", ContextNameMatching::Exact);
        assert_eq!(members(exact.read(&metadata)), subject);
        assert_eq!(
            members(read("Subject", ContextNameMatching::AsciiCaseInsensitive).read(&metadata)),
            subject
        );
        assert!(
            read("Subject", ContextNameMatching::Exact)
                .read(&metadata)
                .is_none()
        );
        assert_eq!(
            members(read("roles", ContextNameMatching::Exact).read(&metadata)),
            vec![("roles", ContextValueCardinality::Many, vec![])]
        );
        assert!(exact.read(&Metadata::default()).is_none());

        assert_eq!(
            bindings
                .headers()
                .expect("all captured headers")
                .read(&metadata)
                .map(|header| header.bytes)
                .collect::<Vec<_>>(),
            vec![b"untrusted".as_slice()]
        );
    }

    /// Scenario: debug formatting is applied to primitive and composite reads of an identity claim.
    /// Guarantees: diagnostics retain value metadata but omit both text and byte representations of claims.
    #[test]
    fn read_value_debug_redacts_identity_and_composite_payloads() {
        let selectors = [
            ContextEntrySelector {
                target: ContextEntryTarget::Primitive {
                    domain: ContextDomain::AuthorizedIdentity,
                    name: name("subject"),
                    matching: ContextNameMatching::Exact,
                },
                form: ContextEntrySelectorForm::Value,
                take: ContextEntryTake::All,
            },
            ContextEntrySelector {
                target: ContextEntryTarget::Composite {
                    name: name("request"),
                },
                form: ContextEntrySelectorForm::Value,
                take: ContextEntryTake::All,
            },
        ];
        let bindings = compile(
            [
                consumes(selectors.to_vec()),
                ContextDeclaration::AuthorizedIdentityCapture {
                    policy: serde_json::from_value(serde_json::json!([
                        {"claim": "sub", "store_as": "subject"}
                    ]))
                    .expect("identity policy"),
                },
            ]
            .into_iter()
            .collect(),
            &[ContextEntryDeclaration {
                scope: ContextScope::Engine,
                name: name("request"),
                definition: ContextEntryDefinition(vec![ContextEntryPart::AuthorizedIdentity {
                    name: name("subject"),
                    store_as: Some(name("account")),
                }]),
            }],
        );
        let secret = "private-subject";
        let identity = AuthorizedIdentity::new().with_subject(secret);
        let (metadata, _) = bindings.arrival().expect("arrival").form(&Arrival {
            headers: ArrivalHeaders::Pairs(&[]),
            identity: Some(&identity),
            peer: None,
        });
        for (selector, member_name) in selectors.iter().zip(["subject", "account"]) {
            let selected = values(
                bindings
                    .value(selector)
                    .expect("declared read")
                    .read(&metadata),
            );
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].bytes, secret.as_bytes());
            assert_eq!(
                format!("{:?}", selected[0]),
                format!(
                    "ReadValue {{ name: {member_name:?}, original_name: None, kind: Text, byte_len: {} }}",
                    secret.len()
                ),
            );
        }
    }

    /// Scenario: a node asks its bindings for a read it never declared.
    /// Guarantees: selection fails loudly at construction instead of returning an absent read.
    #[test]
    fn undeclared_read_is_a_construction_error() {
        let bindings = compile(
            [consumes(vec![primitive(
                "tenant",
                ContextNameMatching::Exact,
            )])]
            .into_iter()
            .collect(),
            &[],
        );
        assert!(
            bindings
                .value(&primitive("tenant", ContextNameMatching::Exact))
                .is_ok()
        );
        for undeclared in [
            primitive("other", ContextNameMatching::Exact),
            primitive("tenant", ContextNameMatching::AsciiCaseInsensitive),
        ] {
            assert!(bindings.value(&undeclared).is_err());
        }
    }
}
