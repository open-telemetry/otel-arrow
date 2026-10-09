// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logical layouts and projections for context declarations.
//!
//! A projection identifies selected values and the enclosing composite whose
//! presence a consumer must establish. This module compiles and resolves that
//! model; runtime materialization is implemented by the sibling
//! `materialization` module, while header propagation remains separate.
//!
//! Layout-local IDs are not offsets into message storage. General consumer
//! bindings and precomputed hashes remain separate work.

use std::collections::{BTreeMap, BTreeSet};

use otel_arrow_dfe_config::context::ContextEntryName;
pub use otel_arrow_dfe_config::context_policy::ContextDomain;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryPart, ContextRandomnessKind, ContextScope,
};
use otel_arrow_dfe_config::error::Error;

/// A deterministic logical layout of primitive fields and composite entries.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextLayout {
    /// Primitive elements from each domain.
    fields: Box<[ContextFieldLayout]>,
    /// Composite entries from `policies::context::entries`.
    entries: Box<[ContextEntryLayout]>,
    /// Primitive names are local to their authority domain.
    field_names: BTreeMap<(ContextDomain, ContextEntryName), ContextFieldId>,
    /// Composite names are independent of primitive source names.
    entry_names: BTreeMap<ContextEntryName, ContextEntryId>,
}

/// A primitive field is a named element in one authority domain.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextFieldLayout {
    /// Stored name.
    pub name: ContextEntryName,
    /// Authority domain.
    pub domain: ContextDomain,
}

/// Corresponds with the position of a field in the layout `fields`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextFieldId(usize);

impl ContextFieldId {
    /// Returns the index into `fields`.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Corresponds with the position of a composite entry in `entries`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextEntryId(usize);

impl ContextEntryId {
    /// Returns the index into `entries`.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Identity of one presence gate in the layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContextNameId {
    /// A primitive field.
    Primitive(ContextFieldId),
    /// A composite entry, addressable as a whole or by qualified member.
    Composite(ContextEntryId),
}

/// Member of a composite entry.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextMember {
    /// Canonical member name.
    ///
    /// Field-backed members use `store_as` or the referenced primitive name;
    /// domainless members use their configured name.
    pub name: ContextEntryName,
    /// Value source for this member.
    pub source: ContextMemberSource,
}

/// Logical source of one composite member value.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContextMemberSource {
    /// Value supplied by one domain-scoped primitive field.
    Field(ContextFieldId),
    /// Inline UTF-8 value supplied by configuration.
    Constant(Box<str>),
    /// Random value generated according to the configured kind.
    Randomness(ContextRandomnessKind),
}

/// One conditional element
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextCondition {
    /// Condition field
    pub field: ContextFieldId,
    /// Condition value.
    pub value: Box<[u8]>,
}

/// One composite entry.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextEntryLayout {
    /// Composite entry name.
    pub name: ContextEntryName,
    /// Declaring scope.
    pub scope: ContextScope,
    /// Members in canonical name order; fields must be present, while domainless sources always are.
    pub members: Box<[ContextMember]>,
    /// Canonically ordered conditions; all must match for this entry to exist.
    pub conditions: Box<[ContextCondition]>,
}

/// Selected context values and their atomic presence gate.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContextProjection {
    /// One independent primitive field.
    Primitive(ContextFieldId),
    /// All or selected members from a composite entry.
    Composite {
        /// Composite whose members and conditions determine presence.
        entry: ContextEntryId,
        /// Members in canonical member-name order.
        members: Box<[ContextMember]>,
    },
}

impl ContextProjection {
    /// Returns the primitive or composite whose presence gates this projection.
    #[must_use]
    pub const fn presence(&self) -> ContextNameId {
        match self {
            Self::Primitive(field) => ContextNameId::Primitive(*field),
            Self::Composite { entry, .. } => ContextNameId::Composite(*entry),
        }
    }

    /// Returns projected composite members in canonical name order.
    #[must_use]
    pub fn members(&self) -> Option<&[ContextMember]> {
        match self {
            Self::Primitive(_) => None,
            Self::Composite { members, .. } => Some(members),
        }
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: message.into(),
    }
}

pub(super) fn validate_definition(declaration: &ContextEntryDeclaration) -> Result<(), Error> {
    let errors = declaration
        .definition
        .validation_errors(&format!("context entry `{}`", declaration.name));
    if !errors.is_empty() {
        return Err(invalid(errors.join("; ")));
    }
    Ok(())
}

impl ContextLayout {
    /// Compiles a binding's fields and entry declarations into a logical layout.
    pub fn compile(
        fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, Error> {
        let fields: Box<[_]> = fields
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut entries = Vec::with_capacity(declarations.len());
        let field_names = fields
            .iter()
            .enumerate()
            .map(|(index, field)| ((field.domain, field.name.clone()), ContextFieldId(index)))
            .collect();
        let mut entry_names = BTreeMap::new();

        let mut ordered = declarations.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        for declaration in ordered {
            let vacant_name = match entry_names.entry(declaration.name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => entry,
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(invalid(format!(
                        "duplicate composite context entry `{}`",
                        declaration.name
                    )));
                }
            };
            validate_definition(declaration)?;
            let mut members = Vec::with_capacity(declaration.definition.0.len());
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                match part {
                    ContextEntryPart::Constant { name, value } => {
                        members.push(ContextMember {
                            name: name.clone(),
                            source: ContextMemberSource::Constant(value.clone().into_boxed_str()),
                        });
                        continue;
                    }
                    ContextEntryPart::Randomness { name, value } => {
                        members.push(ContextMember {
                            name: name.clone(),
                            source: ContextMemberSource::Randomness(*value),
                        });
                        continue;
                    }
                    ContextEntryPart::TransportHeader { .. }
                    | ContextEntryPart::AuthorizedIdentity { .. }
                    | ContextEntryPart::TransportHeaderMatch { .. } => {}
                }
                let (domain, source_name) = part
                    .referenced_source()
                    .expect("referenced context part has a domain and source name");
                let mut matching = fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| field.domain == domain && field.matches_name(source_name));
                let field = matching
                    .next()
                    .map(|(index, _)| ContextFieldId(index))
                    .ok_or_else(|| {
                        invalid(format!(
                            "context entry `{}` requires unavailable {:?} domain `{source_name}`",
                            declaration.name, domain
                        ))
                    })?;
                if matching.next().is_some() {
                    return Err(invalid(format!(
                        "context entry `{}` has ambiguous {:?} reference `{source_name}`",
                        declaration.name, domain
                    )));
                }
                if let ContextEntryPart::TransportHeaderMatch { value, .. } = part {
                    let condition = ContextCondition {
                        field,
                        value: value.as_bytes().into(),
                    };
                    conditions.push(condition);
                    continue;
                }
                let name = part
                    .member_name()
                    .expect("value-bearing context part has a member name")
                    .clone();
                if members.iter().any(|member: &ContextMember| {
                    member.source == ContextMemberSource::Field(field)
                }) {
                    return Err(invalid(format!(
                        "context entry `{}` repeats `{source_name}`",
                        declaration.name
                    )));
                }
                members.push(ContextMember {
                    name,
                    source: ContextMemberSource::Field(field),
                });
            }
            members.sort_unstable_by(|left, right| left.name.cmp(&right.name));
            conditions.sort_unstable();
            conditions.dedup();
            let id = ContextEntryId(entries.len());
            _ = vacant_name.insert(id);
            entries.push(ContextEntryLayout {
                name: declaration.name.clone(),
                scope: declaration.scope.clone(),
                members: members.into_boxed_slice(),
                conditions: conditions.into_boxed_slice(),
            });
        }
        Ok(Self {
            fields,
            entries: entries.into_boxed_slice(),
            field_names,
            entry_names,
        })
    }

    /// Primitive fields in stable name-and-domain order.
    #[must_use]
    pub fn fields(&self) -> &[ContextFieldLayout] {
        &self.fields
    }

    /// Composite entries in stable name-and-scope order.
    #[must_use]
    pub fn entries(&self) -> &[ContextEntryLayout] {
        &self.entries
    }

    /// Resolves an exact stored primitive name within its source domain.
    pub fn resolve_primitive(
        &self,
        domain: ContextDomain,
        name: &ContextEntryName,
    ) -> Result<ContextProjection, Error> {
        self.field_names
            .get(&(domain, name.clone()))
            .copied()
            .map(ContextProjection::Primitive)
            .ok_or_else(|| invalid(format!("unknown {domain:?} context entry `{name}`")))
    }

    /// Resolves a member using the domain recorded by its composite definition.
    ///
    /// The selected member retains the entire composite's presence gate.
    pub fn resolve_member(
        &self,
        composite: &ContextEntryName,
        member: &ContextEntryName,
    ) -> Result<ContextProjection, Error> {
        let entry = self.entry_id(composite)?;
        let member = self.entries[entry.index()]
            .members
            .iter()
            .find(|candidate| &candidate.name == member)
            .ok_or_else(|| invalid(format!("unknown context member `{composite}:{member}`")))?
            .clone();
        Ok(ContextProjection::Composite {
            entry,
            members: Box::new([member]),
        })
    }

    /// Resolves a whole composite, preserving member order and source domains.
    pub fn resolve_composite(
        &self,
        composite: &ContextEntryName,
    ) -> Result<ContextProjection, Error> {
        let entry = self.entry_id(composite)?;
        Ok(ContextProjection::Composite {
            entry,
            members: self.entries[entry.index()].members.clone(),
        })
    }

    fn entry_id(&self, name: &ContextEntryName) -> Result<ContextEntryId, Error> {
        self.entry_names
            .get(name)
            .copied()
            .ok_or_else(|| invalid(format!("unknown composite context entry `{name}`")))
    }
}

impl ContextFieldLayout {
    fn matches_name(&self, name: &ContextEntryName) -> bool {
        match self.domain {
            ContextDomain::TransportHeader => {
                self.name.as_str().eq_ignore_ascii_case(name.as_str())
            }
            ContextDomain::AuthorizedIdentity => self.name == *name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::context_policy::ContextEntryDefinition;

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid name")
    }

    fn field(value: &str, domain: ContextDomain) -> ContextFieldLayout {
        ContextFieldLayout {
            name: name(value),
            domain,
        }
    }

    fn fields() -> Vec<ContextFieldLayout> {
        vec![
            field("workspace", ContextDomain::TransportHeader),
            field("customer", ContextDomain::AuthorizedIdentity),
        ]
    }

    fn entry() -> ContextEntryDeclaration {
        ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("product_user"),
            definition: ContextEntryDefinition(vec![
                ContextEntryPart::AuthorizedIdentity {
                    name: name("customer"),
                    store_as: Some(name("customer_id")),
                },
                ContextEntryPart::TransportHeader {
                    name: name("workspace"),
                    store_as: None,
                },
            ]),
        }
    }

    fn conditional_fields() -> Vec<ContextFieldLayout> {
        let mut result = fields();
        result.extend([
            field("environment", ContextDomain::TransportHeader),
            field("region", ContextDomain::TransportHeader),
        ]);
        result
    }

    fn conditional_entry() -> ContextEntryDeclaration {
        let mut result = entry();
        result.definition.0.extend([
            ContextEntryPart::TransportHeaderMatch {
                name: name("region"),
                value: "west".to_owned(),
            },
            ContextEntryPart::TransportHeaderMatch {
                name: name("environment"),
                value: "production".to_owned(),
            },
        ]);
        result
    }

    fn compile(
        fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: &[ContextEntryDeclaration],
    ) -> ContextLayout {
        ContextLayout::compile(fields, declarations).expect("valid layout")
    }

    fn assert_compile_error(
        fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: &[ContextEntryDeclaration],
        expected: &str,
    ) {
        let error =
            ContextLayout::compile(fields, declarations).expect_err("compilation must fail");
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?} in {error}"
        );
    }

    fn projection_names<'a>(
        layout: &'a ContextLayout,
        projection: &'a ContextProjection,
    ) -> Vec<&'a str> {
        match projection {
            ContextProjection::Primitive(field) => {
                vec![layout.fields()[field.index()].name.as_str()]
            }
            ContextProjection::Composite { members, .. } => members
                .iter()
                .map(|member| match &member.source {
                    ContextMemberSource::Field(field) => {
                        layout.fields()[field.index()].name.as_str()
                    }
                    ContextMemberSource::Constant(_) | ContextMemberSource::Randomness(_) => {
                        member.name.as_str()
                    }
                })
                .collect(),
        }
    }

    fn projection_sources(projection: &ContextProjection) -> Vec<ContextMemberSource> {
        match projection {
            ContextProjection::Primitive(field) => {
                vec![ContextMemberSource::Field(*field)]
            }
            ContextProjection::Composite { members, .. } => {
                members.iter().map(|member| member.source.clone()).collect()
            }
        }
    }

    /// Scenario: primitive, whole-composite, and qualified names select conditional or plain entries.
    /// Guarantees: primitives stay independent; members retain their entire composite gate,
    /// aliases resolve to source fields, and conditions never become projected members.
    #[test]
    fn projections_preserve_fields_and_presence_gates() {
        for declaration in [entry(), conditional_entry()] {
            let layout = compile(conditional_fields(), &[declaration]);
            let primitive = layout
                .resolve_primitive(ContextDomain::TransportHeader, &name("workspace"))
                .expect("primitive");
            let ContextProjection::Primitive(field) = primitive else {
                panic!("workspace must be primitive");
            };
            assert_eq!(primitive.presence(), ContextNameId::Primitive(field));
            assert_eq!(projection_names(&layout, &primitive), ["workspace"]);
            assert_eq!(layout.entries().len(), 1);
            for (projection, expected) in [
                (
                    layout.resolve_composite(&name("product_user")),
                    vec!["customer", "workspace"],
                ),
                (
                    layout.resolve_member(&name("product_user"), &name("customer_id")),
                    vec!["customer"],
                ),
                (
                    layout.resolve_member(&name("product_user"), &name("workspace")),
                    vec!["workspace"],
                ),
            ] {
                let projection = projection.expect("projection");
                assert_eq!(
                    projection.presence(),
                    ContextNameId::Composite(ContextEntryId(0)),
                );
                assert_eq!(projection_names(&layout, &projection), expected);
            }
        }
    }

    /// Scenario: field, member, and declaration input order vary independently.
    /// Guarantees: IDs and member order stay canonical, with names taking precedence over scopes.
    #[test]
    fn input_order_does_not_change_layout() {
        let mut alpha = entry();
        alpha.scope = ContextScope::Group("group".into());
        alpha.name = name("alpha");
        let mut zeta = entry();
        zeta.name = name("zeta");
        let declarations = [zeta, alpha];
        let expected = compile(fields(), &declarations);
        for permutation in 0..8 {
            let mut sources = fields();
            let mut entries = declarations.clone();
            if permutation & 1 != 0 {
                sources.reverse();
            }
            if permutation & 2 != 0 {
                entries.reverse();
            }
            if permutation & 4 != 0 {
                for entry in &mut entries {
                    entry.definition.0.reverse();
                }
            }
            assert_eq!(compile(sources, &entries), expected, "{permutation}");
        }
        assert_eq!(
            expected
                .fields()
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["customer", "workspace"]
        );
        assert_eq!(
            expected
                .entries()
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        assert_eq!(
            expected.entries()[0]
                .members
                .iter()
                .map(|member| member.name.as_str())
                .collect::<Vec<_>>(),
            ["customer_id", "workspace"]
        );
    }

    /// Scenario: a grouping references a name in the wrong source domain.
    /// Guarantees: compilation fails rather than interpreting a trusted claim as a header.
    #[test]
    fn provenance_mismatch_is_rejected() {
        assert_compile_error(
            [field("customer", ContextDomain::TransportHeader)],
            &[entry()],
            "requires unavailable AuthorizedIdentity domain `customer`",
        );
    }

    /// Scenario: declarations repeat composite names or have no members.
    /// Guarantees: the compiler rejects ambiguous namespaces and invokes definition validation.
    #[test]
    fn invalid_declarations_are_rejected() {
        let mut second = entry();
        second.scope = ContextScope::Group("group".into());
        let mut empty = entry();
        empty.definition.0.clear();
        for (declarations, expected) in [
            (
                vec![entry(), second],
                "duplicate composite context entry `product_user`",
            ),
            (vec![empty], "must contain at least one member"),
        ] {
            assert_compile_error(fields(), &declarations, expected);
        }
    }

    /// Scenario: selections use missing names, the wrong domain, or a primitive as a composite.
    /// Guarantees: explicit resolution never falls back to another domain or namespace.
    #[test]
    fn invalid_projections_are_rejected() {
        let layout = compile(fields(), &[entry()]);
        for (result, expected) in [
            (
                layout.resolve_primitive(ContextDomain::TransportHeader, &name("missing")),
                "unknown TransportHeader context entry `missing`",
            ),
            (
                layout.resolve_primitive(ContextDomain::AuthorizedIdentity, &name("workspace")),
                "unknown AuthorizedIdentity context entry `workspace`",
            ),
            (
                layout.resolve_primitive(ContextDomain::TransportHeader, &name("product_user")),
                "unknown TransportHeader context entry `product_user`",
            ),
            (
                layout.resolve_composite(&name("missing")),
                "unknown composite context entry `missing`",
            ),
            (
                layout.resolve_member(&name("product_user"), &name("missing")),
                "unknown context member `product_user:missing`",
            ),
            (
                layout.resolve_member(&name("workspace"), &name("customer")),
                "unknown composite context entry `workspace`",
            ),
        ] {
            let error = result.expect_err("invalid reference");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    /// Scenario: both source domains and a composite share a name, with aliased composite members.
    /// Guarantees: explicit lookups distinguish all three and qualified members retain provenance.
    #[test]
    fn source_domains_and_composites_have_independent_namespaces() {
        let fields = [
            field("customer", ContextDomain::AuthorizedIdentity),
            field("customer", ContextDomain::TransportHeader),
        ];
        let mut declaration = entry();
        declaration.name = name("customer");
        declaration.definition.0[1] = ContextEntryPart::TransportHeader {
            name: name("customer"),
            store_as: Some(name("header")),
        };
        let layout = compile(fields, &[declaration]);
        let identity = layout
            .resolve_primitive(ContextDomain::AuthorizedIdentity, &name("customer"))
            .expect("identity");
        let header = layout
            .resolve_primitive(ContextDomain::TransportHeader, &name("customer"))
            .expect("header");
        assert_ne!(projection_sources(&identity), projection_sources(&header));
        let whole = layout
            .resolve_composite(&name("customer"))
            .expect("composite");
        for (member, primitive) in [("customer_id", identity), ("header", header)] {
            let projection = layout
                .resolve_member(&name("customer"), &name(member))
                .expect("member");
            assert_eq!(
                projection_sources(&projection),
                projection_sources(&primitive)
            );
            assert_eq!(projection.presence(), whole.presence());
            assert_ne!(projection.presence(), primitive.presence());
        }
        assert_eq!(
            whole
                .members()
                .expect("composite members")
                .iter()
                .map(|member| match &member.source {
                    ContextMemberSource::Field(field) => layout.fields()[field.index()].domain,
                    ContextMemberSource::Constant(_) | ContextMemberSource::Randomness(_) => {
                        panic!("unexpected domainless member")
                    }
                })
                .collect::<Vec<_>>(),
            [
                ContextDomain::AuthorizedIdentity,
                ContextDomain::TransportHeader
            ],
        );
    }

    /// Scenario: one conditional composite requires two distinct values from the same field.
    /// Guarantees: distinct same-field conditions remain expressible and canonical.
    #[test]
    fn distinct_conditions_on_one_field_are_retained() {
        let mut conditional = entry();
        conditional.definition.0.extend([
            ContextEntryPart::TransportHeaderMatch {
                name: name("workspace"),
                value: "production".to_owned(),
            },
            ContextEntryPart::TransportHeaderMatch {
                name: name("workspace"),
                value: "staging".to_owned(),
            },
        ]);

        let layout = compile(fields(), &[conditional]);
        assert_eq!(layout.entries()[0].conditions.len(), 2);
        assert_eq!(
            layout.entries()[0]
                .conditions
                .iter()
                .map(|condition| condition.value.as_ref())
                .collect::<Vec<_>>(),
            [b"production".as_slice(), b"staging".as_slice()]
        );
    }

    /// Scenario: equivalent conditional composites declare their conditions in different orders.
    /// Guarantees: conditions compile canonically and do not become projected value members.
    #[test]
    fn conditional_entries_compile_canonical_presence_requirements() {
        let conditional = conditional_entry();
        let mut reordered = conditional.clone();
        reordered.definition.0.swap(2, 3);

        let first = compile(conditional_fields(), &[conditional]);
        let second = compile(conditional_fields(), &[reordered]);
        assert_eq!(first, second);
        let projection = first
            .resolve_composite(&name("product_user"))
            .expect("composite entry");
        let ContextNameId::Composite(entry_id) = projection.presence() else {
            panic!("product_user must have composite presence");
        };
        let entry = &first.entries()[entry_id.index()];
        assert_eq!(
            entry
                .conditions
                .iter()
                .map(|condition| first.fields()[condition.field.index()].name.as_str())
                .collect::<Vec<_>>(),
            ["environment", "region"]
        );
        assert_eq!(
            projection_names(&first, &projection),
            ["customer", "workspace"]
        );
    }

    /// Scenario: a header reference varies in case while an identity reference does not.
    /// Guarantees: transport matching preserves stored spelling without folding identity names.
    #[test]
    fn reference_matching_respects_source_domains() {
        let mut declaration = entry();
        declaration.definition.0[1] = ContextEntryPart::TransportHeader {
            name: name("WORKSPACE"),
            store_as: None,
        };
        let layout = compile(fields(), &[declaration.clone()]);
        let projection = layout
            .resolve_member(&name("product_user"), &name("WORKSPACE"))
            .expect("member");
        assert_eq!(projection_names(&layout, &projection), ["workspace"]);
        declaration.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: name("CUSTOMER"),
            store_as: None,
        };
        assert_compile_error(fields(), &[declaration], "unavailable AuthorizedIdentity");
    }

    /// Scenario: distinct stored header names differ only by ASCII case.
    /// Guarantees: a case-insensitive composite reference reports ambiguity rather than picking one.
    #[test]
    fn ambiguous_transport_reference_is_rejected() {
        let mut sources = fields();
        sources.push(field("WORKSPACE", ContextDomain::TransportHeader));
        assert_compile_error(sources, &[entry()], "ambiguous TransportHeader reference");
    }

    /// Scenario: a constant-only entry compiles without any primitive fields.
    /// Guarantees: the literal is projected as an always-available member with composite presence.
    #[test]
    fn constant_only_entry_compiles_without_fields() {
        let declaration = ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("route"),
            definition: ContextEntryDefinition(vec![ContextEntryPart::Constant {
                name: name("route_name"),
                value: "otlp-http-json".to_owned(),
            }]),
        };

        let layout = compile([], &[declaration]);
        assert!(layout.fields().is_empty());
        assert!(layout.entries()[0].conditions.is_empty());
        let projection = layout
            .resolve_member(&name("route"), &name("route_name"))
            .expect("constant member");
        assert_eq!(
            projection.members().expect("composite members"),
            [ContextMember {
                name: name("route_name"),
                source: ContextMemberSource::Constant("otlp-http-json".into()),
            }]
        );
        assert_eq!(
            projection.presence(),
            ContextNameId::Composite(ContextEntryId(0))
        );
    }

    /// Scenario: a UUID v7 randomness-only entry compiles without primitive fields.
    /// Guarantees: the generator is projected as an always-available domainless member.
    #[test]
    fn randomness_only_entry_compiles_without_fields() {
        let declaration = ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("idempotency"),
            definition: ContextEntryDefinition(vec![ContextEntryPart::Randomness {
                name: name("id"),
                value: ContextRandomnessKind::Uuid7,
            }]),
        };

        let layout = compile([], &[declaration]);
        assert!(layout.fields().is_empty());
        assert!(layout.entries()[0].conditions.is_empty());
        let projection = layout
            .resolve_member(&name("idempotency"), &name("id"))
            .expect("randomness member");
        assert_eq!(
            projection.members().expect("composite members"),
            [ContextMember {
                name: name("id"),
                source: ContextMemberSource::Randomness(ContextRandomnessKind::Uuid7),
            }]
        );
        assert_eq!(
            projection.presence(),
            ContextNameId::Composite(ContextEntryId(0))
        );
    }

    /// Scenario: a randomness member is guarded by a transport-header match condition.
    /// Guarantees: generated values remain available without bypassing the composite condition.
    #[test]
    fn randomness_member_retains_composite_condition_gate() {
        let declaration = ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("idempotency"),
            definition: ContextEntryDefinition(vec![
                ContextEntryPart::Randomness {
                    name: name("id"),
                    value: ContextRandomnessKind::Uuid7,
                },
                ContextEntryPart::TransportHeaderMatch {
                    name: name("environment"),
                    value: "production".to_owned(),
                },
            ]),
        };

        let layout = compile(
            [field("environment", ContextDomain::TransportHeader)],
            &[declaration],
        );
        let projection = layout
            .resolve_member(&name("idempotency"), &name("id"))
            .expect("randomness member");
        let ContextNameId::Composite(entry_id) = projection.presence() else {
            panic!("randomness member must retain composite presence");
        };
        let entry = &layout.entries()[entry_id.index()];
        assert_eq!(
            projection.members().expect("composite members"),
            [ContextMember {
                name: name("id"),
                source: ContextMemberSource::Randomness(ContextRandomnessKind::Uuid7),
            }]
        );
        assert_eq!(entry.conditions.len(), 1);
        assert_eq!(
            layout.fields()[entry.conditions[0].field.index()]
                .name
                .as_str(),
            "environment"
        );
        assert_eq!(entry.conditions[0].value.as_ref(), b"production");
    }

    /// Scenario: a constant member is guarded by a transport-header match condition.
    /// Guarantees: the constant remains available without bypassing the composite condition gate.
    #[test]
    fn constant_member_retains_composite_condition_gate() {
        let declaration = ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("route"),
            definition: ContextEntryDefinition(vec![
                ContextEntryPart::Constant {
                    name: name("route_name"),
                    value: "otlp-http-json".to_owned(),
                },
                ContextEntryPart::TransportHeaderMatch {
                    name: name("environment"),
                    value: "production".to_owned(),
                },
            ]),
        };

        let layout = compile(
            [field("environment", ContextDomain::TransportHeader)],
            &[declaration],
        );
        let projection = layout
            .resolve_member(&name("route"), &name("route_name"))
            .expect("constant member");
        let ContextNameId::Composite(entry_id) = projection.presence() else {
            panic!("constant member must retain composite presence");
        };
        let entry = &layout.entries()[entry_id.index()];
        assert_eq!(
            projection.members().expect("composite members"),
            [ContextMember {
                name: name("route_name"),
                source: ContextMemberSource::Constant("otlp-http-json".into()),
            }]
        );
        assert_eq!(entry.conditions.len(), 1);
        assert_eq!(
            layout.fields()[entry.conditions[0].field.index()]
                .name
                .as_str(),
            "environment"
        );
        assert_eq!(entry.conditions[0].value.as_ref(), b"production");
    }

    /// Scenario: equivalent mixed composites reorder constant and field members.
    /// Guarantees: constant values compile canonically while remaining binding-significant.
    #[test]
    fn constants_are_canonical_and_binding_significant() {
        let mut declaration = entry();
        declaration.definition.0.push(ContextEntryPart::Constant {
            name: name("scheme"),
            value: "ApiKey".to_owned(),
        });
        let mut reordered = declaration.clone();
        reordered.definition.0.reverse();
        assert_eq!(
            compile(fields(), &[declaration.clone()]),
            compile(fields(), &[reordered])
        );

        let mut changed = declaration.clone();
        let ContextEntryPart::Constant { value, .. } =
            changed.definition.0.last_mut().expect("constant member")
        else {
            panic!("last member must be constant");
        };
        *value = "Bearer".to_owned();
        assert_ne!(
            compile(fields(), &[declaration]),
            compile(fields(), &[changed])
        );
    }
}
