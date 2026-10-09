// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logical layouts and projections for context declarations.
//!
//! A projection identifies the set of values and composite presence that
//! are logically evaluated at construction. The current implementation
//! evaluates conditions when contexts are read, instead.

use std::collections::{BTreeMap, BTreeSet};

use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
pub use otel_arrow_dfe_config::context_policy::ContextDomain;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryPart, ContextScope,
};
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::transport_headers::{TransportHeaderRef, TransportHeaders};

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
    /// Precompiled runtime presence requirements for each composite entry.
    presence: Box<[EntryPresence]>,
}

/// One header requirement, either presence of value equality.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderRequirement {
    /// Canonical stored header name.
    name: ContextEntryName,
    /// Optional condition bytes.
    value: Option<Box<[u8]>>,
}

impl HeaderRequirement {
    #[inline]
    fn matches(&self, header: TransportHeaderRef<'_>) -> bool {
        self.name.eq_ignore_ascii_case(header.name.as_str())
            && self
                .value
                .as_deref()
                .is_none_or(|value| value == header.value.bytes)
    }
}

/// Runtime requirements that must all hold for one composite entry to be present.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct EntryPresence {
    /// Required transport-header members and exact-value conditions.
    headers: Box<[HeaderRequirement]>,
    /// Required authorized-identity member names.
    identities: Box<[ContextEntryName]>,
}

impl EntryPresence {
    // Compute presence requirements.
    fn compile(entry: &ContextEntryLayout, fields: &[ContextFieldLayout]) -> Self {
        let mut headers = BTreeMap::<ContextEntryName, BTreeSet<Option<Box<[u8]>>>>::new();
        let mut identities = Vec::new();
        for member in &entry.members {
            let ContextMemberSource::Field(field) = member.source else {
                continue;
            };
            let field = &fields[field.index()];
            match field.domain {
                ContextDomain::TransportHeader => {
                    _ = headers
                        .entry(field.name.to_ascii_lowercase())
                        .or_default()
                        .insert(None);
                }
                ContextDomain::AuthorizedIdentity => identities.push(field.name.clone()),
            }
        }
        for condition in &entry.conditions {
            let values = headers
                .entry(fields[condition.field.index()].name.to_ascii_lowercase())
                .or_default();
            // A matching value also proves that its member exists.
            _ = values.remove(&None);
            _ = values.insert(Some(condition.value.clone()));
        }
        let mut headers = headers
            .into_iter()
            .flat_map(|(name, values)| {
                values.into_iter().map(move |value| HeaderRequirement {
                    name: name.clone(),
                    value,
                })
            })
            .collect::<Vec<_>>();
        // Stable partitioning retains canonical condition order and rejects failed
        // conditions before spending work on unconditional members.
        headers.sort_by_key(|requirement| requirement.value.is_none());
        Self {
            headers: headers.into_boxed_slice(),
            identities: identities.into_boxed_slice(),
        }
    }

    // Evaluate presence requirements. Note this is currently called during
    // projection, while we would prefer it to happen at construction.
    fn is_present(&self, context: &impl ContextValues) -> bool {
        if !self
            .identities
            .iter()
            .all(|name| context.has_authorized_identity(name))
        {
            return false;
        }
        if self.headers.is_empty() {
            return true;
        }
        let Some(headers) = context.transport_headers() else {
            return false;
        };
        if headers.len() == 1 {
            let header = headers.get(0).expect("one header");
            return self
                .headers
                .iter()
                .all(|requirement| requirement.matches(header));
        }
        if headers.len() <= 5 && self.headers.len() > 1 {
            let mut iter = headers.iter();
            let Some(first) = iter.next() else {
                return false;
            };
            // Decode small packed inputs once rather than once per requirement.
            let mut captured = [first; 5];
            for (slot, header) in captured[1..].iter_mut().zip(iter) {
                *slot = header;
            }
            return self.headers.iter().all(|requirement| {
                captured[..headers.len()]
                    .iter()
                    .any(|header| requirement.matches(*header))
            });
        }
        self.headers
            .iter()
            .all(|requirement| headers.iter().any(|header| requirement.matches(header)))
    }
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
    /// constants use their configured name.
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
    /// Members in canonical name order.
    pub members: Box<[ContextMember]>,
    /// Ordered conditions.
    pub conditions: Box<[ContextCondition]>,
}

/// Selected context values and conditions.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContextProjection {
    /// One primitive field.
    Primitive(ContextFieldId),
    /// A composite entry.
    Composite {
        /// Composite that determines presence.
        entry: ContextEntryId,
        /// Members in canonical member_name order.
        members: Box<[ContextMember]>,
    },
}

impl ContextProjection {
    /// Returns the primitive or composite that must be present.
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

/// Read-only values from specific domains. NOTE: This is a bridge
/// while we still have separate Arc<_> holding transport headers.
/// and authorized fields separately.
pub trait ContextValues {
    /// Captured or produced transport headers.
    fn transport_headers(&self) -> Option<&TransportHeaders>;

    /// Whether an exact stored authorized-identity name is present.
    fn has_authorized_identity(&self, name: &ContextEntryName) -> bool;
}

impl ContextValues for TransportHeaders {
    fn transport_headers(&self) -> Option<&TransportHeaders> {
        Some(self)
    }

    fn has_authorized_identity(&self, _name: &ContextEntryName) -> bool {
        false
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
    /// Compiles explicit primitive fields and selected composites into a layout.
    pub fn compile<'a>(
        primitive_fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: impl IntoIterator<Item = &'a ContextEntryDeclaration>,
    ) -> Result<Self, Error> {
        let mut declarations = declarations.into_iter().collect::<Vec<_>>();
        declarations.sort_unstable();
        declarations.dedup();

        let referenced_fields = declarations
            .iter()
            .flat_map(|declaration| &declaration.definition.0)
            .filter_map(|part| {
                part.referenced_source()
                    .map(|(domain, name)| ContextFieldLayout {
                        name: name.clone(),
                        domain,
                    })
            });
        let candidates = primitive_fields
            .into_iter()
            .chain(referenced_fields)
            .collect::<BTreeSet<_>>();
        let mut fields = Vec::new();
        for candidate in candidates {
            if !fields.iter().any(|field: &ContextFieldLayout| {
                field.domain == candidate.domain && field.matches_name(&candidate.name)
            }) {
                fields.push(candidate);
            }
        }
        let fields = fields.into_boxed_slice();
        let field_names = fields
            .iter()
            .enumerate()
            .map(|(index, field)| ((field.domain, field.name.clone()), ContextFieldId(index)))
            .collect();
        let mut entries = BTreeMap::<ContextEntryName, ContextEntryLayout>::new();
        for declaration in declarations {
            validate_definition(declaration)?;
            let mut members = Vec::with_capacity(declaration.definition.0.len());
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                if let ContextEntryPart::Constant { name, value } = part {
                    members.push(ContextMember {
                        name: name.clone(),
                        source: ContextMemberSource::Constant(value.clone().into_boxed_str()),
                    });
                    continue;
                }
                let (domain, source_name) = part
                    .referenced_source()
                    .expect("non-constant context part has a referenced source");
                let field = fields
                    .iter()
                    .position(|field| field.domain == domain && field.matches_name(source_name))
                    .map(ContextFieldId)
                    .expect("all referenced sources were collected before assigning field IDs");
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
            let entry = ContextEntryLayout {
                name: declaration.name.clone(),
                scope: declaration.scope.clone(),
                members: members.into_boxed_slice(),
                conditions: conditions.into_boxed_slice(),
            };
            match entries.entry(declaration.name.clone()) {
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    _ = vacant.insert(entry);
                }
                std::collections::btree_map::Entry::Occupied(occupied)
                    if occupied.get() == &entry => {}
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(invalid(format!(
                        "conflicting definitions for composite context entry `{}`",
                        declaration.name
                    )));
                }
            }
        }
        let entries: Box<[_]> = entries.into_values().collect();
        let entry_names = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.name.clone(), ContextEntryId(index)))
            .collect();
        let presence = entries
            .iter()
            .map(|entry| EntryPresence::compile(entry, &fields))
            .collect();
        Ok(Self {
            fields,
            entries,
            field_names,
            entry_names,
            presence,
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

    /// Evaluates a composite atomically, including members not selected for output.
    #[must_use]
    pub fn is_present(&self, entry: ContextEntryId, context: &impl ContextValues) -> bool {
        self.presence[entry.index()].is_present(context)
    }

    /// Resolves a primitive, whole composite, or qualified composite member.
    pub fn resolve(&self, reference: &ContextEntryRef) -> Result<ContextProjection, Error> {
        match reference.scope() {
            Some(composite) => self.resolve_member(composite, reference.name()),
            None => {
                if let Some(entry) = self.entry_names.get(reference.name()) {
                    return self.resolve_composite(&self.entries[entry.index()].name);
                }
                let mut fields = self
                    .field_names
                    .iter()
                    .filter(|((_, name), _)| name == reference.name())
                    .map(|(_, field)| *field);
                let field = fields
                    .next()
                    .ok_or_else(|| invalid(format!("unknown context entry `{reference}`")))?;
                if fields.next().is_some() {
                    return Err(invalid(format!(
                        "context entry `{reference}` requires an explicit source domain"
                    )));
                }
                Ok(ContextProjection::Primitive(field))
            }
        }
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
                    ContextMemberSource::Constant(_) => member.name.as_str(),
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

    /// Scenario: repeated fields and mixed-case member/condition references arrive in varied orders.
    /// Guarantees: IDs, members, and conditions stay canonical, with names preceding scopes.
    #[test]
    fn input_order_does_not_change_layout() {
        let mut alpha = conditional_entry();
        alpha
            .definition
            .0
            .push(ContextEntryPart::TransportHeaderMatch {
                name: name("WORKSPACE"),
                value: "production".to_owned(),
            });
        let mut zeta = alpha.clone();
        alpha.scope = ContextScope::Group("group".into());
        alpha.name = name("alpha");
        zeta.name = name("zeta");
        let declarations = [zeta, alpha];
        let expected = compile(fields(), &declarations);
        for permutation in 0..8 {
            let mut sources = fields();
            sources.extend(fields());
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
            ["WORKSPACE", "customer", "environment", "region"]
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

    /// Scenario: a composite requires an identity whose name is also an explicit header field.
    /// Guarantees: all member and condition fields are collected in their declared domains,
    /// without allowing the header field to substitute for the identity.
    #[test]
    fn composite_sources_are_collected_in_their_declared_domains() {
        let layout = compile(
            [field("customer", ContextDomain::TransportHeader)],
            &[conditional_entry()],
        );
        assert_eq!(layout.fields().len(), 5);
        let header = layout
            .resolve_primitive(ContextDomain::TransportHeader, &name("customer"))
            .expect("explicit header");
        let identity = layout
            .resolve_primitive(ContextDomain::AuthorizedIdentity, &name("customer"))
            .expect("inferred identity");
        let member = layout
            .resolve_member(&name("product_user"), &name("customer_id"))
            .expect("identity member");
        assert_ne!(projection_sources(&header), projection_sources(&identity));
        assert_eq!(projection_sources(&member), projection_sources(&identity));
        assert_eq!(
            layout.entries()[0]
                .conditions
                .iter()
                .map(|condition| layout.fields()[condition.field.index()].name.as_str())
                .collect::<Vec<_>>(),
            ["environment", "region"]
        );
    }

    /// Scenario: declarations conflict in scope, members, or conditions, or have no members.
    /// Guarantees: conflicting definitions are rejected instead of choosing one, and empty
    /// definitions still fail validation.
    #[test]
    fn invalid_declarations_are_rejected() {
        let mut second = entry();
        second.scope = ContextScope::Group("group".into());
        let mut changed_member = entry();
        changed_member.definition.0[1] = ContextEntryPart::TransportHeader {
            name: name("account"),
            store_as: Some(name("workspace")),
        };
        let mut empty = entry();
        empty.definition.0.clear();
        for (declarations, expected) in [
            (
                vec![entry(), second],
                "conflicting definitions for composite context entry `product_user`",
            ),
            (
                vec![entry(), changed_member],
                "conflicting definitions for composite context entry `product_user`",
            ),
            (
                vec![entry(), conditional_entry()],
                "conflicting definitions for composite context entry `product_user`",
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
                    ContextMemberSource::Constant(_) => panic!("unexpected constant"),
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

    /// Scenario: explicit and referenced source names differ in ASCII case in both domains.
    /// Guarantees: headers use one deterministic stored spelling, while identities stay distinct.
    #[test]
    fn reference_matching_respects_source_domains() {
        let mut declaration = entry();
        declaration.definition.0[1] = ContextEntryPart::TransportHeader {
            name: name("WORKSPACE"),
            store_as: None,
        };
        declaration.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: name("CUSTOMER"),
            store_as: None,
        };
        let layout = compile(fields(), &[declaration]);
        assert_eq!(layout.fields().len(), 3);
        let header = layout
            .resolve_member(&name("product_user"), &name("WORKSPACE"))
            .expect("header member");
        assert_eq!(projection_names(&layout, &header), ["WORKSPACE"]);
        let identity = layout
            .resolve_member(&name("product_user"), &name("CUSTOMER"))
            .expect("identity member");
        let lower = layout
            .resolve_primitive(ContextDomain::AuthorizedIdentity, &name("customer"))
            .expect("explicit identity");
        let upper = layout
            .resolve_primitive(ContextDomain::AuthorizedIdentity, &name("CUSTOMER"))
            .expect("inferred identity");
        assert_ne!(projection_sources(&lower), projection_sources(&upper));
        assert_eq!(projection_sources(&identity), projection_sources(&upper));
    }

    /// Scenario: several nodes select the same composite with reordered members and conditions.
    /// Guarantees: repeated declarations share one entry, field set, and complete presence plan.
    #[test]
    fn repeated_declarations_share_one_entry() {
        let mut declaration = conditional_entry();
        declaration.definition.0.push(ContextEntryPart::Constant {
            name: name("scheme"),
            value: "ApiKey".to_owned(),
        });
        let expected = compile([], &[declaration.clone()]);
        let mut reordered = declaration.clone();
        reordered.definition.0.reverse();
        assert_eq!(
            compile([], &[declaration.clone(), reordered, declaration]),
            expected
        );
    }

    /// Scenario: one composite selects the same header twice using different case and aliases.
    /// Guarantees: field canonicalization does not permit repeated references within a composite.
    #[test]
    fn repeated_composite_header_source_is_rejected() {
        let mut declaration = entry();
        declaration
            .definition
            .0
            .push(ContextEntryPart::TransportHeader {
                name: name("WORKSPACE"),
                store_as: Some(name("another_workspace")),
            });
        assert_compile_error([], &[declaration], "repeats `WORKSPACE`");
    }

    /// Scenario: small and larger composites require multiple values from the same header.
    /// Guarantees: every member and distinct value is required; duplicate and mixed-case headers
    /// cannot satisfy missing requirements.
    #[test]
    fn compiled_presence_handles_duplicate_values_and_member_counts() {
        use otel_arrow_dfe_config::transport_headers::TransportHeader;

        for count in [1, 2, 3, 4, 5, 32] {
            let sources = (0..count)
                .map(|index| ContextFieldLayout {
                    name: name(&format!("field_{index}")),
                    domain: ContextDomain::TransportHeader,
                })
                .collect::<Vec<_>>();
            let mut parts = sources
                .iter()
                .map(|field| ContextEntryPart::TransportHeader {
                    name: field.name.clone(),
                    store_as: None,
                })
                .collect::<Vec<_>>();
            parts.extend(
                ["one", "two"].map(|value| ContextEntryPart::TransportHeaderMatch {
                    name: name("field_0"),
                    value: value.into(),
                }),
            );
            let layout = compile(
                sources,
                &[ContextEntryDeclaration {
                    scope: ContextScope::Engine,
                    name: name("composite"),
                    definition: ContextEntryDefinition(parts),
                }],
            );
            let assert_present = |headers: &TransportHeaders, expected| {
                assert_eq!(
                    layout.is_present(ContextEntryId(0), headers),
                    expected,
                    "{count}"
                );
            };
            let mut headers = TransportHeaders::new();
            for index in 0..count {
                assert_present(&headers, false);
                headers.push(TransportHeader::text(
                    name(&format!("FIELD_{index}")),
                    b"one",
                ));
            }
            headers.push(TransportHeader::text(name("field_0"), b"one"));
            assert_present(&headers, false);
            headers.push(TransportHeader::text(name("field_0"), b"TWO"));
            assert_present(&headers, false);
            headers.push(TransportHeader::text(name("field_0"), b"two"));
            assert_present(&headers, true);
        }
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
    /// Guarantees: constant values compile canonically, remain binding-significant, and conflicting
    /// values cannot share one composite entry.
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
        assert_compile_error(
            fields(),
            &[declaration.clone(), changed.clone()],
            "conflicting definitions for composite context entry `product_user`",
        );
        assert_ne!(
            compile(fields(), &[declaration]),
            compile(fields(), &[changed])
        );
    }
}
