// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logical layouts and projections for context declarations.
//!
//! A projection identifies selected values and the enclosing composite whose
//! presence a consumer must establish. This module compiles and resolves that
//! model and evaluates atomic presence against existing message context.
//!
//! Layout-local IDs and whole-composite projections provide a foundation for
//! subsequent consumers. They are not offsets into message storage. Presence
//! is evaluated against existing header and identity storage at read time;
//! ingestion-time materialization and precomputed hashes are separate work.
//! Presence plans deduplicate header requirements at compilation and check
//! conditions before unconditional members, without read-time allocation.

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

/// A compact lookup for the small sets of configured propagation names.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(super) struct HeaderLookup<T> {
    /// Bindings in deterministic configured-name order.
    entries: Box<[HeaderLookupEntry<T>]>,
}

/// One case-insensitive stored-header-name binding.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderLookupEntry<T> {
    /// Configured stored name, matched using ASCII case-insensitive semantics.
    name: String,
    /// Value bound to the configured name.
    value: T,
}

impl<T> Default for HeaderLookup<T> {
    fn default() -> Self {
        Self {
            entries: Box::new([]),
        }
    }
}

impl<T> HeaderLookup<T> {
    pub(super) fn new(entries: BTreeMap<String, T>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(name, value)| HeaderLookupEntry { name, value })
                .collect(),
        }
    }

    #[inline]
    pub(super) fn get(&self, name: &str) -> Option<&T> {
        if let [entry] = self.entries.as_ref() {
            return entry
                .name
                .eq_ignore_ascii_case(name)
                .then_some(&entry.value);
        }
        if self.entries.is_empty() {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
            .map(|entry| &entry.value)
    }
}

/// One transport-header member or exact-value condition required for presence.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderRequirement {
    /// Canonical stored header name, matched using ASCII case-insensitive semantics.
    name: ContextEntryName,
    /// Required bytes for a condition, or `None` when any value proves presence.
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
    fn compile_presence_requirements(
        entry: &ContextEntryLayout,
        fields: &[ContextFieldLayout],
    ) -> Self {
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
    /// Members in canonical name order; field sources must be present, while constants always are.
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

/// Read-only presence information from a message's separate authority domains.
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
    /// Compiles composites selected by qualified references, leaving unused definitions inert.
    ///
    /// Only selected definitions contribute fields. Missing runtime values make
    /// a composite absent rather than invalidating startup.
    pub(crate) fn for_references<'a>(
        declarations: &[ContextEntryDeclaration],
        references: impl IntoIterator<Item = &'a ContextEntryRef>,
    ) -> Result<Self, Error> {
        let requested = references
            .into_iter()
            .filter_map(ContextEntryRef::scope)
            .collect::<BTreeSet<_>>();
        for name in &requested {
            if !declarations
                .iter()
                .any(|declaration| &declaration.name == *name)
            {
                return Err(invalid(format!("unknown composite context entry `{name}`")));
            }
        }
        let selected = declarations
            .iter()
            .filter(|declaration| requested.contains(&declaration.name))
            .cloned()
            .collect::<Vec<_>>();
        Self::for_declarations(&selected)
    }

    /// Compiles all supplied composite declarations into one canonical layout.
    pub(crate) fn for_declarations(
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, Error> {
        let required = declarations
            .iter()
            .flat_map(|declaration| &declaration.definition.0)
            .filter_map(|part| {
                part.referenced_source()
                    .map(|(domain, name)| ContextFieldLayout {
                        name: name.clone(),
                        domain,
                    })
            })
            .collect::<BTreeSet<_>>();
        let mut fields = BTreeSet::new();
        for field in required {
            if !fields.iter().any(|existing: &ContextFieldLayout| {
                existing.domain == field.domain && existing.matches_name(&field.name)
            }) {
                _ = fields.insert(field);
            }
        }
        Self::compile_layout(fields, declarations)
    }

    /// Merges node-local layouts into one canonical pipeline layout.
    pub(crate) fn merge<'a>(
        layouts: impl IntoIterator<Item = &'a ContextLayout>,
    ) -> Result<Self, Error> {
        let layouts = layouts.into_iter().collect::<Vec<_>>();
        let candidates = layouts
            .iter()
            .flat_map(|layout| layout.fields.iter().cloned())
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
        let remap_field =
            |layout: &ContextLayout, field: ContextFieldId| -> Result<ContextFieldId, Error> {
                let source = &layout.fields[field.index()];
                fields
                    .iter()
                    .position(|candidate| {
                        candidate.domain == source.domain && candidate.matches_name(&source.name)
                    })
                    .map(ContextFieldId)
                    .ok_or_else(|| {
                        invalid(format!(
                            "pipeline context layout lost {:?} field `{}` while merging",
                            source.domain, source.name
                        ))
                    })
            };

        let mut merged_entries = BTreeMap::<ContextEntryName, ContextEntryLayout>::new();
        for layout in layouts {
            for entry in &layout.entries {
                let members = entry
                    .members
                    .iter()
                    .map(|member| {
                        Ok(ContextMember {
                            name: member.name.clone(),
                            source: match &member.source {
                                ContextMemberSource::Field(field) => {
                                    ContextMemberSource::Field(remap_field(layout, *field)?)
                                }
                                ContextMemberSource::Constant(value) => {
                                    ContextMemberSource::Constant(value.clone())
                                }
                            },
                        })
                    })
                    .collect::<Result<Box<[_]>, Error>>()?;
                let conditions = entry
                    .conditions
                    .iter()
                    .map(|condition| {
                        Ok(ContextCondition {
                            field: remap_field(layout, condition.field)?,
                            value: condition.value.clone(),
                        })
                    })
                    .collect::<Result<Box<[_]>, Error>>()?;
                let merged = ContextEntryLayout {
                    name: entry.name.clone(),
                    scope: entry.scope.clone(),
                    members,
                    conditions,
                };
                match merged_entries.entry(entry.name.clone()) {
                    std::collections::btree_map::Entry::Vacant(vacant) => {
                        _ = vacant.insert(merged);
                    }
                    std::collections::btree_map::Entry::Occupied(occupied)
                        if occupied.get() == &merged => {}
                    std::collections::btree_map::Entry::Occupied(_) => {
                        return Err(invalid(format!(
                            "pipeline context entry `{}` has conflicting layouts",
                            entry.name
                        )));
                    }
                }
            }
        }

        let entries = merged_entries
            .into_values()
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let field_names = fields
            .iter()
            .enumerate()
            .map(|(index, field)| ((field.domain, field.name.clone()), ContextFieldId(index)))
            .collect();
        let entry_names = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.name.clone(), ContextEntryId(index)))
            .collect();
        let presence = entries
            .iter()
            .map(|entry| EntryPresence::compile_presence_requirements(entry, &fields))
            .collect();
        Ok(Self {
            fields,
            entries,
            field_names,
            entry_names,
            presence,
        })
    }

    /// Compiles a binding's fields and entry declarations into a logical layout.
    pub fn compile_layout(
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
        let presence = entries
            .iter()
            .map(|entry| EntryPresence::compile_presence_requirements(entry, &fields))
            .collect();
        Ok(Self {
            fields,
            entries: entries.into_boxed_slice(),
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
    use otel_arrow_dfe_config::ContextEntryRef;
    use otel_arrow_dfe_config::context_policy::ContextEntryDefinition;

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid name")
    }

    fn reference(value: &str) -> ContextEntryRef {
        value.try_into().expect("valid reference")
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

    fn compile_layout(
        fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: &[ContextEntryDeclaration],
    ) -> ContextLayout {
        ContextLayout::compile_layout(fields, declarations).expect("valid layout")
    }

    fn assert_compile_error(
        fields: impl IntoIterator<Item = ContextFieldLayout>,
        declarations: &[ContextEntryDeclaration],
        expected: &str,
    ) {
        let error =
            ContextLayout::compile_layout(fields, declarations).expect_err("compilation must fail");
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
            let layout = compile_layout(conditional_fields(), &[declaration]);
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
        let expected = compile_layout(fields(), &declarations);
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
            assert_eq!(compile_layout(sources, &entries), expected, "{permutation}");
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
        let layout = compile_layout(fields(), &[entry()]);
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
        let layout = compile_layout(fields, &[declaration]);
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

        let layout = compile_layout(fields(), &[conditional]);
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

        let first = compile_layout(conditional_fields(), &[conditional]);
        let second = compile_layout(conditional_fields(), &[reordered]);
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
        let layout = compile_layout(fields(), &[declaration.clone()]);
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

    /// Scenario: node-local layouts arrive in different node iteration orders.
    /// Guarantees: pipeline merging assigns identical field and entry IDs independent of order.
    #[test]
    fn merged_pipeline_layout_is_canonical() {
        let first = compile_layout(fields(), &[entry()]);
        let mut other = entry();
        other.name = name("other");
        let second = compile_layout(fields(), &[other]);

        let forward = ContextLayout::merge([&first, &second]).expect("forward merge");
        let reverse = ContextLayout::merge([&second, &first]).expect("reverse merge");

        assert_eq!(forward, reverse);
        assert_eq!(forward.entries().len(), 2);
        assert_ne!(
            forward
                .resolve_composite(&name("product_user"))
                .expect("product user")
                .presence(),
            forward
                .resolve_composite(&name("other"))
                .expect("other")
                .presence()
        );
    }

    /// Scenario: a pipeline consumes one composite while another has unrelated source members.
    /// Guarantees: only live references activate definitions, and unused source slots remain absent.
    #[test]
    fn consumer_selection_leaves_unused_definitions_inert() {
        let mut unused = entry();
        unused.name = name("unused");
        unused.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: name("other_identity"),
            store_as: None,
        };
        let selected = reference("product_user:workspace");
        let layout = ContextLayout::for_references(&[entry(), unused], [&selected])
            .expect("only selected entry compiles");
        assert_eq!(layout.entries().len(), 1);
        assert_eq!(layout.fields().len(), 2);
        let ContextNameId::Composite(entry) = layout.resolve(&selected).expect("member").presence()
        else {
            panic!("expected composite");
        };
        assert!(!layout.is_present(entry, &TransportHeaders::new()));
    }

    /// Scenario: member and condition spellings differ in case and their declaration order changes.
    /// Guarantees: reserved field names, condition gates, and resulting layouts are deterministic.
    #[test]
    fn inferred_transport_fields_are_canonical() {
        let mut declaration = entry();
        declaration
            .definition
            .0
            .push(ContextEntryPart::TransportHeaderMatch {
                name: name("WORKSPACE"),
                value: "production".to_owned(),
            });
        let selected = reference("product_user:workspace");
        let first =
            ContextLayout::for_references(&[declaration.clone()], [&selected]).expect("original");
        declaration.definition.0.reverse();
        let second = ContextLayout::for_references(&[declaration], [&selected]).expect("reordered");
        assert_eq!(first, second);
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
            let layout = compile_layout(
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

        let layout = compile_layout([], &[declaration]);
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

        let layout = compile_layout(
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
            compile_layout(fields(), &[declaration.clone()]),
            compile_layout(fields(), &[reordered])
        );

        let mut changed = declaration.clone();
        let ContextEntryPart::Constant { value, .. } =
            changed.definition.0.last_mut().expect("constant member")
        else {
            panic!("last member must be constant");
        };
        *value = "Bearer".to_owned();
        assert_ne!(
            compile_layout(fields(), &[declaration]),
            compile_layout(fields(), &[changed])
        );
    }
}
