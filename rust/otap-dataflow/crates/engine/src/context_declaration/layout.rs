// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resolved projections used by the engine's context declarations and bindings.
//!
//! A projection selects values without weakening its composite's presence gate:
//! every member must exist in its declared domain and every condition must match.
//! Header propagation is the first consumer of this shared contract.
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
    /// Namespace containing both primitives and composites.
    names: BTreeMap<ContextEntryName, ContextNameId>,
    presence: Box<[EntryPresence]>,
}

/// A compact lookup for the small sets of configured propagation names.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(super) struct HeaderLookup<T>(Box<[(u64, String, T)]>);

fn header_key(name: &str) -> u64 {
    // This is only a rejection key; callers still compare the full names.
    let bytes = name.as_bytes();
    ((name.len() as u64) << 16)
        | (u64::from(bytes.first().copied().unwrap_or(0).to_ascii_lowercase()) << 8)
        | u64::from(bytes.last().copied().unwrap_or(0).to_ascii_lowercase())
}

impl<T> Default for HeaderLookup<T> {
    fn default() -> Self {
        Self(Box::new([]))
    }
}

impl<T> HeaderLookup<T> {
    pub(super) fn new(entries: BTreeMap<String, T>) -> Self {
        Self(
            entries
                .into_iter()
                .map(|(name, value)| (header_key(&name), name, value))
                .collect(),
        )
    }

    #[inline]
    pub(super) fn get(&self, name: &str) -> Option<&T> {
        if let [(_, stored, value)] = self.0.as_ref() {
            return stored.eq_ignore_ascii_case(name).then_some(value);
        }
        if self.0.is_empty() {
            return None;
        }
        let key = header_key(name);
        self.0
            .iter()
            .find(|(stored_key, stored, _)| *stored_key == key && stored.eq_ignore_ascii_case(name))
            .map(|(_, _, value)| value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderRequirement {
    name: String,
    key: u64,
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct EntryPresence {
    headers: Box<[HeaderRequirement]>,
    identities: Box<[ContextEntryName]>,
}

impl EntryPresence {
    fn compile(entry: &ContextEntryLayout, fields: &[ContextFieldLayout]) -> Self {
        let mut headers = BTreeMap::<String, BTreeSet<Option<Box<[u8]>>>>::new();
        let mut identities = Vec::new();
        for member in &entry.members {
            let field = &fields[member.field.index()];
            match field.domain {
                ContextDomain::TransportHeader => {
                    _ = headers
                        .entry(field.name.as_str().to_ascii_lowercase())
                        .or_default()
                        .insert(None);
                }
                ContextDomain::AuthorizedIdentity => identities.push(field.name.clone()),
            }
        }
        for condition in &entry.conditions {
            let values = headers
                .entry(
                    fields[condition.field.index()]
                        .name
                        .as_str()
                        .to_ascii_lowercase(),
                )
                .or_default();
            // A matching value also proves that its member exists.
            _ = values.remove(&None);
            _ = values.insert(Some(condition.value.clone()));
        }
        let mut headers = headers
            .into_iter()
            .flat_map(|(name, values)| {
                values.into_iter().map(move |value| HeaderRequirement {
                    key: header_key(&name),
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

    fn is_present(
        &self,
        context: &impl ContextValues,
        observed: Option<TransportHeaderRef<'_>>,
    ) -> bool {
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
            let header = observed.or_else(|| headers.get(0)).expect("one header");
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
            let mut captured = [(header_key(first.name.as_str()), first); 5];
            for (slot, header) in captured[1..].iter_mut().zip(iter) {
                *slot = (header_key(header.name.as_str()), header);
            }
            return self.headers.iter().all(|requirement| {
                captured[..headers.len()]
                    .iter()
                    .any(|(key, header)| requirement.key == *key && requirement.matches(*header))
            });
        }
        self.headers.iter().all(|requirement| {
            observed.is_some_and(|header| requirement.matches(header))
                || headers.iter().any(|header| requirement.matches(header))
        })
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

/// Identity of one top-level name in the layout.
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
    /// Name is this member's store_as, falls back to the field's
    /// primitive name.
    pub name: ContextEntryName,
    /// Primitive field
    pub field: ContextFieldId,
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
    /// Members in canonical name order; all must be present for this entry to exist.
    pub members: Box<[ContextMember]>,
    /// Canonically ordered conditions; all must match for this entry to exist.
    pub conditions: Box<[ContextCondition]>,
}

/// Selected context values and their atomic presence gate.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContextProjection {
    /// One independent primitive field.
    Primitive(ContextFieldId),
    /// All or selected fields from a composite entry.
    Composite {
        /// Composite whose members and conditions determine presence.
        entry: ContextEntryId,
        /// Fields in canonical member-name order.
        fields: Box<[ContextFieldId]>,
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

    /// Returns fields in canonical projection order.
    #[must_use]
    pub fn fields(&self) -> &[ContextFieldId] {
        match self {
            Self::Primitive(field) => std::slice::from_ref(field),
            Self::Composite { fields, .. } => fields,
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

impl ContextLayout {
    /// Compiles composites selected by qualified references, leaving unused definitions inert.
    ///
    /// Only the selected definitions contribute fields. Capture and producer
    /// policies keep their existing semantics; context can also arrive from
    /// another pipeline. Missing message values mean absence, not a startup error.
    pub(crate) fn for_references<'a>(
        declarations: &[ContextEntryDeclaration],
        references: impl IntoIterator<Item = &'a ContextEntryRef>,
    ) -> Result<Self, Error> {
        let mut fields = BTreeSet::<ContextFieldLayout>::new();
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
        let required = selected
            .iter()
            .flat_map(|declaration| &declaration.definition.0)
            .map(|part| ContextFieldLayout {
                name: part.reference().name().clone(),
                domain: part.domain(),
            })
            .collect::<BTreeSet<_>>();
        for field in required {
            if !fields.iter().any(|existing| {
                existing.domain == field.domain && existing.matches_name(&field.name)
            }) {
                _ = fields.insert(field);
            }
        }
        Self::compile(fields, &selected)
    }

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
        let mut names = BTreeMap::new();
        for (index, field) in fields.iter().enumerate() {
            let name = field.name.clone();
            let field_id = ContextFieldId(index);
            match names.insert(name.clone(), ContextNameId::Primitive(field_id)) {
                None => {}
                Some(ContextNameId::Primitive(existing)) => {
                    return Err(invalid(format!(
                        "context source `{name}` is produced as both {:?} and {:?}",
                        fields[existing.index()].domain,
                        field.domain,
                    )));
                }
                Some(ContextNameId::Composite(_)) => {
                    return Err(invalid(format!(
                        "context source `{name}` conflicts with a composite entry"
                    )));
                }
            }
        }

        let mut ordered = declarations.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        for declaration in ordered {
            let vacant_name = match names.entry(declaration.name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => entry,
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(invalid(format!(
                        "context entry `{}` conflicts with a primitive or composite entry",
                        declaration.name
                    )));
                }
            };
            let errors = declaration
                .definition
                .validation_errors(&format!("context entry `{}`", declaration.name));
            if !errors.is_empty() {
                return Err(invalid(errors.join("; ")));
            }
            let mut members = Vec::with_capacity(declaration.definition.0.len());
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                let domain = part.domain();
                let reference = part.reference();
                if reference.scope().is_some() {
                    return Err(invalid(format!(
                        "context entry `{}` cannot use nested reference `{reference}`",
                        declaration.name
                    )));
                }
                let mut matching = fields.iter().enumerate().filter(|(_, field)| {
                    field.domain == domain && field.matches_name(reference.name())
                });
                let field = matching
                    .next()
                    .map(|(index, _)| ContextFieldId(index))
                    .ok_or_else(|| {
                        invalid(format!(
                            "context entry `{}` requires unavailable {:?} domain `{reference}`",
                            declaration.name, domain
                        ))
                    })?;
                if matching.next().is_some() {
                    return Err(invalid(format!(
                        "context entry `{}` has ambiguous {:?} reference `{reference}`",
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
                if members
                    .iter()
                    .any(|member: &ContextMember| member.field == field)
                {
                    return Err(invalid(format!(
                        "context entry `{}` repeats `{reference}`",
                        declaration.name
                    )));
                }
                members.push(ContextMember { name, field });
            }
            members.sort_unstable_by(|left, right| left.name.cmp(&right.name));
            conditions.sort_unstable();
            conditions.dedup();
            let id = ContextEntryId(entries.len());
            _ = vacant_name.insert(ContextNameId::Composite(id));
            entries.push(ContextEntryLayout {
                name: declaration.name.clone(),
                scope: declaration.scope.clone(),
                members: members.into_boxed_slice(),
                conditions: conditions.into_boxed_slice(),
            });
        }
        let presence = entries
            .iter()
            .map(|entry| EntryPresence::compile(entry, &fields))
            .collect();
        Ok(Self {
            fields,
            entries: entries.into_boxed_slice(),
            names,
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
        self.presence[entry.index()].is_present(context, None)
    }

    pub(super) fn is_present_with_header(
        &self,
        entry: ContextEntryId,
        context: &impl ContextValues,
        header: TransportHeaderRef<'_>,
    ) -> bool {
        self.presence[entry.index()].is_present(context, Some(header))
    }

    /// Resolve a primitive, whole composite, or qualified composite member.
    pub fn resolve(&self, reference: &ContextEntryRef) -> Result<ContextProjection, Error> {
        match reference.scope() {
            None => {
                let name = *self
                    .names
                    .get(reference.name())
                    .ok_or_else(|| invalid(format!("unknown context entry `{reference}`")))?;
                match name {
                    ContextNameId::Primitive(field) => Ok(ContextProjection::Primitive(field)),
                    ContextNameId::Composite(entry) => Ok(ContextProjection::Composite {
                        entry,
                        fields: self.entries[entry.index()]
                            .members
                            .iter()
                            .map(|member| member.field)
                            .collect(),
                    }),
                }
            }
            Some(entry_name) => {
                let name = *self
                    .names
                    .get(entry_name)
                    .ok_or_else(|| invalid(format!("unknown context entry `{entry_name}`")))?;
                let ContextNameId::Composite(entry) = name else {
                    return Err(invalid(format!(
                        "primitive context entry `{entry_name}` has no qualified members"
                    )));
                };
                let field = self.entries[entry.index()]
                    .members
                    .iter()
                    .find(|candidate| &candidate.name == reference.name())
                    .ok_or_else(|| invalid(format!("unknown context member `{reference}`")))?
                    .field;
                Ok(ContextProjection::Composite {
                    entry,
                    fields: Box::new([field]),
                })
            }
        }
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
                    name: reference("customer"),
                    store_as: Some(name("customer_id")),
                },
                ContextEntryPart::TransportHeader {
                    name: reference("workspace"),
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
                name: reference("region"),
                value: "west".to_owned(),
            },
            ContextEntryPart::TransportHeaderMatch {
                name: reference("environment"),
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
        projection: &ContextProjection,
    ) -> Vec<&'a str> {
        projection
            .fields()
            .iter()
            .map(|field| layout.fields()[field.index()].name.as_str())
            .collect()
    }

    /// Scenario: primitive, whole-composite, and qualified names select conditional or plain entries.
    /// Guarantees: primitives stay independent; members retain their entire composite gate,
    /// aliases resolve to source fields, and conditions never become projected members.
    #[test]
    fn projections_preserve_fields_and_presence_gates() {
        for declaration in [entry(), conditional_entry()] {
            let layout = compile(conditional_fields(), &[declaration]);
            let primitive = layout.resolve(&reference("workspace")).expect("primitive");
            let ContextProjection::Primitive(field) = primitive else {
                panic!("workspace must be primitive");
            };
            assert_eq!(primitive.presence(), ContextNameId::Primitive(field));
            assert_eq!(projection_names(&layout, &primitive), ["workspace"]);
            assert_eq!(layout.entries().len(), 1);
            for (raw, expected) in [
                ("product_user", vec!["customer", "workspace"]),
                ("product_user:customer_id", vec!["customer"]),
                ("product_user:workspace", vec!["workspace"]),
            ] {
                let projection = layout.resolve(&reference(raw)).expect("projection");
                assert_eq!(
                    projection.presence(),
                    ContextNameId::Composite(ContextEntryId(0)),
                    "{raw}"
                );
                assert_eq!(projection_names(&layout, &projection), expected, "{raw}");
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

    /// Scenario: declarations shadow sources/entries, nest derived references, or have no members.
    /// Guarantees: the compiler rejects ambiguous namespaces and invokes definition validation.
    #[test]
    fn invalid_declarations_are_rejected() {
        let mut conflict = entry();
        conflict.name = name("workspace");
        let mut second = entry();
        second.scope = ContextScope::Group("group".into());
        let mut nested = entry();
        nested.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: reference("other:customer"),
            store_as: None,
        };
        let mut empty = entry();
        empty.definition.0.clear();
        for (declarations, expected) in [
            (
                vec![conflict],
                "context entry `workspace` conflicts with a primitive or composite entry",
            ),
            (
                vec![entry(), second],
                "context entry `product_user` conflicts with a primitive or composite entry",
            ),
            (vec![nested], "cannot use nested reference `other:customer`"),
            (vec![empty], "must contain at least one member"),
        ] {
            assert_compile_error(fields(), &declarations, expected);
        }
    }

    /// Scenario: references name missing entries/members or qualify a primitive.
    /// Guarantees: resolution reports the specific failure instead of falling back to another field.
    #[test]
    fn invalid_projections_are_rejected() {
        let layout = compile(fields(), &[entry()]);
        for (raw, expected) in [
            ("missing", "unknown context entry `missing`"),
            (
                "product_user:missing",
                "unknown context member `product_user:missing`",
            ),
            (
                "workspace:customer",
                "primitive context entry `workspace` has no qualified members",
            ),
        ] {
            let error = layout
                .resolve(&reference(raw))
                .expect_err("invalid reference");
            assert!(error.to_string().contains(expected), "{raw}: {error}");
        }
    }

    /// Scenario: transport-header and authorized-identity capture produce the same stored name.
    /// Guarantees: layout compilation rejects the collision instead of requiring consumer aliases.
    #[test]
    fn source_domain_collision_is_rejected() {
        let fields = [
            field("customer", ContextDomain::AuthorizedIdentity),
            field("customer", ContextDomain::TransportHeader),
        ];
        assert_compile_error(
            fields,
            &[],
            "context source `customer` is produced as both TransportHeader and AuthorizedIdentity",
        );
    }

    /// Scenario: one conditional composite requires two distinct values from the same field.
    /// Guarantees: distinct same-field conditions remain expressible and canonical.
    #[test]
    fn distinct_conditions_on_one_field_are_retained() {
        let mut conditional = entry();
        conditional.definition.0.extend([
            ContextEntryPart::TransportHeaderMatch {
                name: reference("workspace"),
                value: "production".to_owned(),
            },
            ContextEntryPart::TransportHeaderMatch {
                name: reference("workspace"),
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
            .resolve(&reference("product_user"))
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
            name: reference("WORKSPACE"),
            store_as: None,
        };
        let layout = compile(fields(), &[declaration.clone()]);
        let projection = layout
            .resolve(&reference("product_user:WORKSPACE"))
            .expect("member");
        assert_eq!(projection_names(&layout, &projection), ["workspace"]);
        declaration.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: reference("CUSTOMER"),
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

    /// Scenario: a pipeline consumes one composite while another has unsupported nested references.
    /// Guarantees: only live references activate definitions, and external source slots may be absent.
    #[test]
    fn consumer_selection_leaves_unused_definitions_inert() {
        let mut unused = entry();
        unused.name = name("unused");
        unused.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: reference("other:identity"),
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
                name: reference("WORKSPACE"),
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
    /// cannot satisfy missing requirements, even when an observed header proves one requirement.
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
                    name: field.name.clone().into(),
                    store_as: None,
                })
                .collect::<Vec<_>>();
            parts.extend(
                ["one", "two"].map(|value| ContextEntryPart::TransportHeaderMatch {
                    name: reference("field_0"),
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
                for header in headers.iter() {
                    assert_eq!(
                        layout.is_present_with_header(ContextEntryId(0), headers, header),
                        expected,
                        "{count}: {}",
                        header.name.as_str()
                    );
                }
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
}
