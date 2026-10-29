// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read adapters retain the existing lookup and materialization semantics.

use super::{
    ContextDomain, ContextEntrySelector, ContextEntryTarget, ContextMaterializationPlan,
    ContextNameMatching, ContextProjection, ContextValueCardinality, ContextValueKind,
    MaterializedContextProjection, MaterializedContextValue, Metadata,
};
use otel_arrow_dfe_config::context::ContextEntryRef;
use otel_arrow_dfe_config::context::transport_headers::{
    TransportHeaderRef, TransportHeadersFindIter, TransportHeadersIter, ValueKind,
};
use otel_arrow_dfe_config::error::Error;

/// One declared named read, resolved once by the context compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueRead {
    reference: ContextEntryRef,
    selector: ContextEntrySelector,
    backend: ValueReadBackend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Composite reads retain a shared plan and projection without another heap allocation.
#[allow(variant_size_differences)]
enum ValueReadBackend {
    TransportHeader(ContextNameMatching),
    Composite {
        plan: ContextMaterializationPlan,
        projection: ContextProjection,
    },
    Unsupported,
}

impl ValueRead {
    pub(super) fn compile(
        selector: &ContextEntrySelector,
        plan: &ContextMaterializationPlan,
    ) -> Result<Self, Error> {
        let (reference, backend) = match &selector.target {
            ContextEntryTarget::Primitive {
                domain,
                name,
                matching,
            } => (
                name.clone().into(),
                match domain {
                    ContextDomain::TransportHeader => ValueReadBackend::TransportHeader(*matching),
                    ContextDomain::AuthorizedIdentity => ValueReadBackend::Unsupported,
                },
            ),
            ContextEntryTarget::CompositeMember { composite, member } => (
                format!("{composite}:{member}").try_into()?,
                ValueReadBackend::Composite {
                    plan: plan.clone(),
                    projection: plan.layout().resolve_member(composite, member)?,
                },
            ),
            ContextEntryTarget::Composite { name } => (
                name.clone().into(),
                ValueReadBackend::Composite {
                    plan: plan.clone(),
                    projection: plan.layout().resolve_composite(name)?,
                },
            ),
        };
        Ok(Self {
            reference,
            selector: selector.clone(),
            backend,
        })
    }

    /// Returns the configured reference for construction-time selection.
    #[must_use]
    pub fn reference(&self) -> &ContextEntryRef {
        &self.reference
    }

    /// Disambiguates reads with the same reference but different targets, forms, or matching.
    #[must_use]
    pub fn selector(&self) -> &ContextEntrySelector {
        &self.selector
    }

    /// Applies a compiled stored-name lookup or composite projection without allocating.
    ///
    /// Missing values and unsupported primitive domains return `None`. A composite
    /// read retains the whole entry's presence gate and source cardinalities.
    #[must_use]
    pub fn read<'a>(&'a self, metadata: &'a Metadata) -> Option<Read<'a>> {
        let backend = match &self.backend {
            ValueReadBackend::TransportHeader(matching) => {
                let headers = metadata.transport_headers()?;
                let name = self.reference.name().as_str();
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
            ValueReadBackend::Composite { plan, projection } => {
                ReadBackend::Composite(metadata.resolve_materialized_context(plan, projection)?)
            }
            ValueReadBackend::Unsupported => return None,
        };
        Some(Read { backend })
    }
}

/// A present borrowed primitive or composite selection.
pub struct Read<'a> {
    backend: ReadBackend<'a>,
}

enum ReadBackend<'a> {
    Headers {
        first: TransportHeaderRef<'a>,
        remaining: HeaderMatches<'a>,
    },
    Composite(MaterializedContextProjection<'a>),
}

impl<'a> Read<'a> {
    /// Visits selected members in existing projection order, including empty multi-valued members.
    ///
    /// Primitive headers form one multi-valued member; duplicate values retain
    /// capture order. Names and original wire names remain available regardless
    /// of the requested selector form.
    pub fn visit_members(self, mut visit: impl FnMut(ReadMember<'a>)) {
        match self.backend {
            ReadBackend::Headers { first, remaining } => visit(ReadMember {
                name: first.name.as_str(),
                cardinality: ContextValueCardinality::Many,
                values: ReadValues::Headers {
                    first: Some(first),
                    remaining,
                },
            }),
            ReadBackend::Composite(projection) => projection.visit_members(|member| {
                visit(ReadMember {
                    name: member.name().as_str(),
                    cardinality: member.values().cardinality(),
                    values: ReadValues::Materialized {
                        name: member.name().as_str(),
                        values: member.values().values().iter(),
                    },
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
}

impl<'a> ReadMember<'a> {
    /// Iterates raw values without flattening, joining, or decoding them.
    pub fn values(self) -> impl Iterator<Item = ReadValue<'a>> {
        self.values
    }
}

/// A borrowed value and its retained names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A borrowed stored transport header returned by an all-stored read.
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
            Self::Materialized { name, values } => values.next().map(|value| ReadValue {
                name,
                original_name: value.original_name(),
                kind: value.kind(),
                bytes: value.bytes(),
            }),
        }
    }
}

/// A declared all-stored read over existing transport-header storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadersRead {
    domain: ContextDomain,
}

impl HeadersRead {
    pub(super) fn new(domain: ContextDomain) -> Self {
        Self { domain }
    }

    /// Iterates stored headers in capture order, preserving duplicates and binary bytes.
    ///
    /// Unsupported domains remain empty rather than becoming new compilation errors.
    pub fn read<'a>(&self, metadata: &'a Metadata) -> impl Iterator<Item = HeaderValue<'a>> {
        (self.domain == ContextDomain::TransportHeader)
            .then(|| metadata.transport_headers())
            .flatten()
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
        ContextDeclaration, ContextEntrySelectorForm, ContextRuntimeRequirements, NodeBindings,
        NodeContextDeclarations, PreparedNodeContextDeclarations,
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

    /// Scenario: consumers declare exact and case-insensitive reads of repeated mixed-case headers.
    /// Guarantees: compiled reads match legacy lookup order, spelling, kinds, bytes, and absence.
    #[test]
    fn primitive_reads_preserve_matching_and_values() {
        let bindings = compile(
            [consumes(vec![
                primitive("Topic", ContextNameMatching::Exact),
                primitive("Topic", ContextNameMatching::AsciiCaseInsensitive),
                primitive("missing", ContextNameMatching::Exact),
            ])]
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
        assert_eq!(bindings.values().len(), 3);
        for read in bindings.values() {
            let ContextEntryTarget::Primitive { matching, .. } = read.selector().target else {
                panic!("primitive read");
            };
            let expected: Vec<ReadValue<'_>> = match matching {
                ContextNameMatching::Exact => headers
                    .find_by_name(read.reference().name().as_str())
                    .map(Into::into)
                    .collect(),
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

    /// Scenario: all-stored reads see captured headers and a subsequently appended header.
    /// Guarantees: the adapter matches legacy iteration without sorting, deduplication, or composites.
    #[test]
    fn all_stored_matches_capture_and_overlay_iteration() {
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
                    selector: ContextConsumerSelector::AllStored {
                        domain: ContextDomain::TransportHeader,
                    },
                },
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllStored {
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
        let read = bindings.headers().expect("all stored read");
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
            },
            ContextEntrySelector {
                target: ContextEntryTarget::CompositeMember {
                    composite: name("request"),
                    member: name("customer"),
                },
                form: ContextEntrySelectorForm::OriginalKeyValue,
            },
        ];
        let bindings = compile(
            [
                consumes(selectors),
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
        for read in bindings.values() {
            let projection = match &read.selector().target {
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
            if read.reference().scope().is_none() {
                assert!(actual.iter().any(|(name, cardinality, values)| {
                    *name == "roles"
                        && *cardinality == ContextValueCardinality::Many
                        && values.is_empty()
                }));
            } else {
                assert_eq!(read.reference().to_string(), "request:customer");
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

    /// Scenario: declarations request primitive identity values and all-stored identity entries.
    /// Guarantees: unsupported read backends remain absent without adding compilation failures.
    #[test]
    fn unsupported_domains_do_not_add_compile_failures() {
        let bindings = compile(
            [
                consumes(vec![ContextEntrySelector {
                    target: ContextEntryTarget::Primitive {
                        domain: ContextDomain::AuthorizedIdentity,
                        name: name("subject"),
                        matching: ContextNameMatching::Exact,
                    },
                    form: ContextEntrySelectorForm::Value,
                }]),
                ContextDeclaration::Consumes {
                    selector: ContextConsumerSelector::AllStored {
                        domain: ContextDomain::AuthorizedIdentity,
                    },
                },
            ]
            .into_iter()
            .collect(),
            &[],
        );
        let mut metadata = Metadata::default();
        let policy = serde_json::from_value(serde_json::json!([
            {"claim": "sub", "store_as": "subject"}
        ]))
        .expect("identity policy");
        metadata.capture_authorized_identity(
            &policy,
            &AuthorizedIdentity::new().with_subject("verified"),
        );
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(name("subject"), b"untrusted"));
        metadata.set_transport_headers(headers);
        assert!(bindings.values()[0].read(&metadata).is_none());
        assert_eq!(
            bindings
                .headers()
                .expect("all stored")
                .read(&metadata)
                .count(),
            0
        );
    }
}
