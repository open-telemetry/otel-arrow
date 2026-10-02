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

use std::collections::{BTreeMap, BTreeSet};

use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
pub use otel_arrow_dfe_config::context_policy::ContextDomain;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryPart, ContextScope,
};
use otel_arrow_dfe_config::error::Error;
use otel_arrow_dfe_config::transport_headers::TransportHeaders;

/// A deterministic logical layout of primitive fields and composite entries.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextLayout {
    /// Primitive elements from each domain.
    fields: Box<[ContextFieldLayout]>,
    /// Composite entries from `policies::context::entries`.
    entries: Box<[ContextEntryLayout]>,
    /// Namespace containing both primitives and composites.
    names: BTreeMap<ContextEntryName, ContextNameId>,
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
        Ok(Self {
            fields,
            entries: entries.into_boxed_slice(),
            names,
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
        let entry = &self.entries[entry.index()];
        let headers = context.transport_headers();
        entry.members.iter().all(|member| {
            let field = &self.fields[member.field.index()];
            match field.domain {
                ContextDomain::TransportHeader => headers.is_some_and(|headers| {
                    headers.iter().any(|header| {
                        header
                            .name
                            .as_str()
                            .eq_ignore_ascii_case(field.name.as_str())
                    })
                }),
                ContextDomain::AuthorizedIdentity => context.has_authorized_identity(&field.name),
            }
        }) && entry.conditions.iter().all(|condition| {
            let field = &self.fields[condition.field.index()];
            headers.is_some_and(|headers| {
                headers.iter().any(|header| {
                    header
                        .name
                        .as_str()
                        .eq_ignore_ascii_case(field.name.as_str())
                        && header.value.bytes == condition.value.as_ref()
                })
            })
        })
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

    fn fields() -> Vec<ContextFieldLayout> {
        vec![
            ContextFieldLayout {
                domain: ContextDomain::TransportHeader,
                name: name("workspace"),
            },
            ContextFieldLayout {
                domain: ContextDomain::AuthorizedIdentity,
                name: name("customer"),
            },
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
            ContextFieldLayout {
                name: name("environment"),
                domain: ContextDomain::TransportHeader,
            },
            ContextFieldLayout {
                name: name("region"),
                domain: ContextDomain::TransportHeader,
            },
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

    /// Scenario: an unqualified primitive name is resolved directly.
    /// Guarantees: primitive projections select one field and create no synthetic composite.
    #[test]
    fn primitive_projection_uses_field_presence() {
        let layout = compile(fields(), &[entry()]);
        let projection = layout
            .resolve(&reference("workspace"))
            .expect("primitive entry");

        let ContextProjection::Primitive(field) = &projection else {
            panic!("workspace must resolve to a primitive projection");
        };
        assert_eq!(projection.fields().len(), 1);
        assert_eq!(projection.presence(), ContextNameId::Primitive(*field));
        assert_eq!(projection_names(&layout, &projection), ["workspace"]);
        assert_eq!(layout.entries().len(), 1);
    }

    /// Scenario: an unqualified composite name is resolved as a whole.
    /// Guarantees: the projection selects every member in canonical member-name order.
    #[test]
    fn whole_composite_projection_selects_all_members() {
        let layout = compile(fields(), &[entry()]);
        let projection = layout
            .resolve(&reference("product_user"))
            .expect("whole composite");

        let ContextProjection::Composite {
            entry: entry_id,
            fields: projected,
        } = &projection
        else {
            panic!("product_user must resolve to a composite projection");
        };
        assert_eq!(*entry_id, ContextEntryId(0));
        assert_eq!(projected.len(), 2);
        assert_eq!(
            projection_names(&layout, &projection),
            ["customer", "workspace"]
        );
    }

    /// Scenario: one qualified member of a composite is resolved.
    /// Guarantees: it selects one field while retaining the whole composite's presence gate.
    #[test]
    fn qualified_projection_retains_composite_presence() {
        let layout = compile(fields(), &[entry()]);
        let whole = layout
            .resolve(&reference("product_user"))
            .expect("whole composite");
        let member = layout
            .resolve(&reference("product_user:customer_id"))
            .expect("qualified member");

        assert_eq!(member.presence(), whole.presence());
        assert_eq!(projection_names(&layout, &member), ["customer"]);
    }

    /// Scenario: field declarations are supplied in different iteration orders.
    /// Guarantees: compiled IDs and selected field order stay deterministic.
    #[test]
    fn field_input_order_does_not_change_layout() {
        let mut reverse = fields();
        reverse.reverse();
        let first = compile(fields(), &[entry()]);
        let second = compile(reverse, &[entry()]);

        assert_eq!(first, second);
        assert_eq!(
            first
                .fields()
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["customer", "workspace"]
        );
    }

    /// Scenario: equivalent composites declare their value-bearing members in different orders.
    /// Guarantees: member order does not change the layout or canonical compiled member order.
    #[test]
    fn member_input_order_does_not_change_layout() {
        let original = entry();
        let mut reversed = original.clone();
        reversed.definition.0.reverse();

        let first = compile(fields(), &[original]);
        let second = compile(fields(), &[reversed]);
        assert_eq!(first, second);
        assert_eq!(
            first.entries()[0]
                .members
                .iter()
                .map(|member| member.name.as_str())
                .collect::<Vec<_>>(),
            ["customer_id", "workspace"]
        );
    }

    /// Scenario: declarations with opposing scope and name order are supplied in either order.
    /// Guarantees: entry IDs use name-first canonical order rather than scope or caller order.
    #[test]
    fn declaration_input_order_does_not_change_layout() {
        let mut alpha = entry();
        alpha.scope = ContextScope::Group("group".into());
        alpha.name = name("alpha");
        let mut zeta = entry();
        zeta.scope = ContextScope::Engine;
        zeta.name = name("zeta");
        let declarations = [zeta, alpha];
        let reversed = [declarations[1].clone(), declarations[0].clone()];
        let first = compile(fields(), &declarations);
        let second = compile(fields(), &reversed);

        assert_eq!(first, second);
        assert_eq!(
            first
                .entries()
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
    }

    /// Scenario: a grouping references a name in the wrong source domain.
    /// Guarantees: compilation fails rather than interpreting a trusted claim as a header.
    #[test]
    fn provenance_mismatch_is_rejected() {
        let only_header = [ContextFieldLayout {
            name: name("customer"),
            domain: ContextDomain::TransportHeader,
        }];
        assert_compile_error(
            only_header,
            &[entry()],
            "requires unavailable AuthorizedIdentity domain `customer`",
        );
    }

    /// Scenario: a visible derived entry conflicts with an existing source name.
    /// Guarantees: source and derived entry IDs cannot shadow each other.
    #[test]
    fn visible_name_collision_is_rejected() {
        let mut conflict = entry();
        conflict.name = name("workspace");
        assert_compile_error(
            fields(),
            &[conflict],
            "context entry `workspace` conflicts with a primitive or composite entry",
        );
    }

    /// Scenario: two visible composite declarations use one name at different scopes.
    /// Guarantees: the flat top-level namespace rejects duplicate composite names.
    #[test]
    fn duplicate_composite_name_is_rejected() {
        let first = entry();
        let mut second = first.clone();
        second.scope = ContextScope::Group("group".into());
        assert_compile_error(
            fields(),
            &[first, second],
            "context entry `product_user` conflicts with a primitive or composite entry",
        );
    }

    /// Scenario: a derived entry is used as a member source.
    /// Guarantees: nested derived entries are explicitly unsupported.
    #[test]
    fn nested_reference_is_rejected() {
        let mut nested = entry();
        nested.definition.0[0] = ContextEntryPart::AuthorizedIdentity {
            name: reference("other:customer"),
            store_as: None,
        };
        assert_compile_error(
            fields(),
            &[nested],
            "cannot use nested reference `other:customer`",
        );
    }

    /// Scenario: an unqualified reference names no primitive or composite.
    /// Guarantees: resolution reports the unknown top-level name.
    #[test]
    fn unknown_name_is_rejected() {
        let layout = compile(fields(), &[entry()]);
        let error = layout
            .resolve(&reference("missing"))
            .expect_err("unknown name must fail");
        assert!(
            error
                .to_string()
                .contains("unknown context entry `missing`")
        );
    }

    /// Scenario: a qualified reference selects a member not defined by its composite.
    /// Guarantees: missing members never fall back to a primitive field.
    #[test]
    fn unknown_composite_member_is_rejected() {
        let layout = compile(fields(), &[entry()]);
        let error = layout
            .resolve(&reference("product_user:missing"))
            .expect_err("unknown member must fail");
        assert!(
            error
                .to_string()
                .contains("unknown context member `product_user:missing`")
        );
    }

    /// Scenario: a qualified reference uses a primitive as its namespace.
    /// Guarantees: primitives cannot expose qualified members.
    #[test]
    fn primitive_cannot_be_qualified() {
        let layout = compile(fields(), &[entry()]);
        let error = layout
            .resolve(&reference("workspace:customer"))
            .expect_err("qualified primitive must fail");
        assert!(
            error
                .to_string()
                .contains("primitive context entry `workspace` has no qualified members")
        );
    }

    /// Scenario: transport-header and authorized-identity capture produce the same stored name.
    /// Guarantees: layout compilation rejects the collision instead of requiring consumer aliases.
    #[test]
    fn source_domain_collision_is_rejected() {
        let fields = [
            ContextFieldLayout {
                name: name("customer"),
                domain: ContextDomain::AuthorizedIdentity,
            },
            ContextFieldLayout {
                name: name("customer"),
                domain: ContextDomain::TransportHeader,
            },
        ];
        assert_compile_error(
            fields,
            &[],
            "context source `customer` is produced as both TransportHeader and AuthorizedIdentity",
        );
    }

    /// Scenario: programmatic declarations bypass policy-level semantic validation.
    /// Guarantees: compilation invokes the shared definition validator before resolving fields.
    #[test]
    fn compilation_validates_definition_shape() {
        let mut empty = entry();
        empty.definition.0.clear();
        assert_compile_error(fields(), &[empty], "must contain at least one member");
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

    /// Scenario: a qualified member is selected from a conditional composite.
    /// Guarantees: the member retains the same condition-bearing presence gate as the whole entry.
    #[test]
    fn qualified_projection_retains_conditional_presence() {
        let layout = compile(conditional_fields(), &[conditional_entry()]);
        let whole = layout
            .resolve(&reference("product_user"))
            .expect("whole composite");
        let member = layout
            .resolve(&reference("product_user:workspace"))
            .expect("qualified member");

        assert_eq!(member.presence(), whole.presence());
        let ContextNameId::Composite(entry_id) = member.presence() else {
            panic!("qualified member must have composite presence");
        };
        assert_eq!(layout.entries()[entry_id.index()].conditions.len(), 2);
        assert_eq!(projection_names(&layout, &member), ["workspace"]);
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
        sources.push(ContextFieldLayout {
            name: name("WORKSPACE"),
            domain: ContextDomain::TransportHeader,
        });
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
}
