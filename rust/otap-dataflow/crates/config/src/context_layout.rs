// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Context layout support.

use std::collections::{BTreeMap, BTreeSet};

use crate::context::{ContextEntryName, ContextEntryRef};
use crate::context_policy::{ContextEntryDeclaration, ContextEntryPart, ContextScope};
use crate::error::Error;

/// A deterministic layout of primitive fields and composite entries.
///
/// Fields and entries use canonical ordering; entry members retain declaration
/// order because that order defines their projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayout {
    /// Primitive elements from each domain.
    fields: Box<[ContextFieldLayout]>,
    /// Composite entries from `policies::context::entries`.
    entries: Box<[ContextEntryLayout]>,
    /// Namespace containing both primitives and composites.
    names: BTreeMap<ContextEntryName, ContextNameId>,
}

/// Context domains are separate areas of configuration and authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextDomain {
    /// Untrusted transport metadata.
    TransportHeader,
    /// Verified authorization claim.
    AuthorizedIdentity,
}

/// A primitive field is a single named element.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContextFieldLayout {
    /// Storage domain.
    pub domain: ContextDomain,
    /// Stored name.
    pub name: ContextEntryName,
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

/// Identity of one name in the layout's top-level namespace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextNameId {
    /// Independently addressable primitive field.
    Primitive(ContextFieldId),
    /// Composite entry, addressable as a whole or by qualified member.
    Composite(ContextEntryId),
}

/// Member of a composite entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMember {
    /// Name is this member's store_as, falls back to the field's
    /// primitive name.
    pub name: ContextEntryName,
    /// Primitive field
    pub field: ContextFieldId,
}

/// One conditional element
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContextCondition {
    /// Condition field
    pub field: ContextFieldId,
    /// Condition value.
    pub value: Box<[u8]>,
}

/// One atomic logical entry, including members and presence conditions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextEntryLayout {
    /// Logical entry name.
    pub name: ContextEntryName,
    /// Declaring scope.
    pub scope: ContextScope,
    /// Members in definition order; all must be present for this entry to exist.
    pub members: Box<[ContextMember]>,
    /// Canonically ordered conditions; all must match for this entry to exist.
    pub conditions: Box<[ContextCondition]>,
}

/// Selected context values and their atomic presence gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextProjection {
    /// One independently present primitive field.
    Primitive(ContextFieldId),
    /// All or selected fields gated by one composite entry.
    Composite {
        /// Composite whose members and conditions determine presence.
        entry: ContextEntryId,
        /// Fields in configured projection order.
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

    /// Returns fields in configured projection order.
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

impl ContextLayout {
    /// Compile one pipeline's fields and entry declarations.
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
            left.scope
                .cmp(&right.scope)
                .then_with(|| left.name.cmp(&right.name))
        });
        for declaration in ordered {
            if names.contains_key(&declaration.name) {
                return Err(invalid(format!(
                    "context entry `{}` conflicts with a primitive or composite entry",
                    declaration.name
                )));
            }
            if declaration.definition.0.is_empty() {
                return Err(invalid(format!(
                    "context entry `{}` must contain at least one member",
                    declaration.name
                )));
            }
            let mut member_names = BTreeSet::new();
            let mut members = Vec::with_capacity(declaration.definition.0.len());
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                let (domain, condition_value) = match part {
                    ContextEntryPart::TransportHeader { .. } => {
                        (ContextDomain::TransportHeader, None)
                    }
                    ContextEntryPart::AuthorizedIdentity { .. } => {
                        (ContextDomain::AuthorizedIdentity, None)
                    }
                    ContextEntryPart::TransportHeaderMatch { value, .. } => {
                        (ContextDomain::TransportHeader, Some(value))
                    }
                };
                let reference = part.reference();
                if reference.scope().is_some() {
                    return Err(invalid(format!(
                        "context entry `{}` cannot use nested reference `{reference}`",
                        declaration.name
                    )));
                }
                let field = fields
                    .binary_search(&ContextFieldLayout {
                        name: reference.name().clone(),
                        domain,
                    })
                    .map(ContextFieldId)
                    .map_err(|_| {
                        invalid(format!(
                            "context entry `{}` requires unavailable {:?} domain `{reference}`",
                            declaration.name, domain
                        ))
                    })?;
                if let Some(value) = condition_value {
                    let condition = ContextCondition {
                        field,
                        value: value.as_bytes().into(),
                    };
                    if conditions.contains(&condition) {
                        return Err(invalid(format!(
                            "context entry `{}` repeats condition for `{reference}`",
                            declaration.name
                        )));
                    }
                    conditions.push(condition);
                    continue;
                }
                let name = part
                    .member_name()
                    .expect("value-bearing context part has a member name")
                    .clone();
                if !member_names.insert(name.clone()) {
                    return Err(invalid(format!(
                        "context entry `{}` repeats member `{name}`",
                        declaration.name
                    )));
                }
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
            if members.is_empty() {
                return Err(invalid(format!(
                    "context entry `{}` must contain at least one value-bearing member",
                    declaration.name
                )));
            }
            conditions.sort_unstable();
            let id = ContextEntryId(entries.len());
            _ = names.insert(declaration.name.clone(), ContextNameId::Composite(id));
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

    /// Primitive fields in stable source-and-name order.
    #[must_use]
    pub fn fields(&self) -> &[ContextFieldLayout] {
        &self.fields
    }

    /// Composite entries in stable scope-and-name order.
    #[must_use]
    pub fn entries(&self) -> &[ContextEntryLayout] {
        &self.entries
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_policy::ContextEntryDefinition;

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

    /// Scenario: a mixed-source grouping is compiled in declaration order.
    /// Guarantees: whole and qualified projections use one atomic presence gate.
    #[test]
    fn mixed_group_and_member_share_presence() {
        let layout = ContextLayout::compile(fields(), &[entry()]).expect("valid layout");
        let whole = layout
            .resolve(&reference("product_user"))
            .expect("whole entry");
        let member = layout
            .resolve(&reference("product_user:customer_id"))
            .expect("qualified member");
        assert_eq!(whole.presence(), member.presence());
        assert_eq!(whole.fields().len(), 2);
        assert_eq!(&whole.fields()[..1], member.fields());
        assert!(matches!(
            whole.presence(),
            ContextNameId::Composite(ContextEntryId(0))
        ));
        assert_eq!(layout.entries().len(), 1);
        assert_eq!(
            layout.fields()[whole.fields()[0].index()].domain,
            ContextDomain::AuthorizedIdentity
        );
        assert_eq!(
            layout.fields()[whole.fields()[1].index()].domain,
            ContextDomain::TransportHeader
        );
    }

    /// Scenario: an unqualified primitive name is resolved directly.
    /// Guarantees: primitive projections contain one field and create no synthetic composite.
    #[test]
    fn primitive_projection_uses_field_presence() {
        let layout = ContextLayout::compile(fields(), &[entry()]).expect("valid layout");
        let projection = layout
            .resolve(&reference("workspace"))
            .expect("primitive entry");

        let ContextProjection::Primitive(field) = &projection else {
            panic!("workspace must resolve to a primitive projection");
        };
        assert_eq!(projection.fields().len(), 1);
        assert_eq!(projection.presence(), ContextNameId::Primitive(*field));
        assert_eq!(layout.entries().len(), 1);
    }

    /// Scenario: field declarations are supplied in different iteration orders.
    /// Guarantees: compiled IDs and selected field order stay deterministic.
    #[test]
    fn field_input_order_does_not_change_layout() {
        let mut reverse = fields();
        reverse.reverse();
        assert_eq!(
            ContextLayout::compile(fields(), &[entry()]).expect("layout"),
            ContextLayout::compile(reverse, &[entry()]).expect("layout")
        );
    }

    /// Scenario: a group declaration precedes an engine declaration in the input.
    /// Guarantees: derived entry IDs follow scope-and-name order, not caller order.
    #[test]
    fn declaration_input_order_does_not_change_layout() {
        let mut group = entry();
        group.scope = ContextScope::Group("alpha".into());
        group.name = name("group_entry");
        let declarations = [group, entry()];
        let reversed = [declarations[1].clone(), declarations[0].clone()];
        assert_eq!(
            ContextLayout::compile(fields(), &declarations).expect("layout"),
            ContextLayout::compile(fields(), &reversed).expect("layout")
        );
    }

    /// Scenario: a grouping references a name in the wrong source domain.
    /// Guarantees: compilation fails rather than interpreting a trusted claim as a header.
    #[test]
    fn provenance_mismatch_is_rejected() {
        let only_header = [ContextFieldLayout {
            domain: ContextDomain::TransportHeader,
            name: name("customer"),
        }];
        assert!(ContextLayout::compile(only_header, &[entry()]).is_err());
    }

    /// Scenario: a visible derived entry conflicts with an existing source name.
    /// Guarantees: source and derived entry IDs cannot shadow each other.
    #[test]
    fn visible_name_collision_is_rejected() {
        let mut conflict = entry();
        conflict.name = name("workspace");
        assert!(ContextLayout::compile(fields(), &[conflict]).is_err());
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
        assert!(ContextLayout::compile(fields(), &[nested]).is_err());
    }

    /// Scenario: a qualified reference selects a member not defined by its parent.
    /// Guarantees: missing members never fall back to a standalone field.
    #[test]
    fn unknown_qualified_member_is_rejected() {
        let layout = ContextLayout::compile(fields(), &[entry()]).expect("layout");
        assert!(layout.resolve(&reference("product_user:missing")).is_err());
        assert!(layout.resolve(&reference("workspace:customer")).is_err());
    }

    /// Scenario: transport-header and authorized-identity capture produce the same stored name.
    /// Guarantees: layout compilation rejects the collision instead of requiring consumer aliases.
    #[test]
    fn source_domain_collision_is_rejected() {
        let fields = [
            ContextFieldLayout {
                domain: ContextDomain::AuthorizedIdentity,
                name: name("customer"),
            },
            ContextFieldLayout {
                domain: ContextDomain::TransportHeader,
                name: name("customer"),
            },
        ];
        let error = ContextLayout::compile(fields, &[]).expect_err("collision must fail");
        let message = error.to_string();
        assert!(message.contains("context source `customer`"));
        assert!(message.contains("TransportHeader"));
        assert!(message.contains("AuthorizedIdentity"));
    }

    /// Scenario: equivalent conditional composites declare their conditions in different orders.
    /// Guarantees: conditions compile canonically and do not become projected value members.
    #[test]
    fn conditional_entries_compile_canonical_presence_requirements() {
        let mut fields = fields();
        fields.extend([
            ContextFieldLayout {
                domain: ContextDomain::TransportHeader,
                name: name("environment"),
            },
            ContextFieldLayout {
                domain: ContextDomain::TransportHeader,
                name: name("region"),
            },
        ]);
        let mut conditional = entry();
        conditional.definition.0.extend([
            ContextEntryPart::TransportHeaderMatch {
                name: reference("region"),
                value: "west".to_owned(),
            },
            ContextEntryPart::TransportHeaderMatch {
                name: reference("environment"),
                value: "production".to_owned(),
            },
        ]);
        let mut reordered = conditional.clone();
        reordered.definition.0.swap(2, 3);

        let first = ContextLayout::compile(fields.clone(), &[conditional]).expect("layout");
        let second = ContextLayout::compile(fields, &[reordered]).expect("layout");
        assert_eq!(first, second);
        let projection = first
            .resolve(&reference("product_user"))
            .expect("composite entry");
        let ContextProjection::Composite {
            entry: entry_id, ..
        } = projection
        else {
            panic!("product_user must resolve to a composite projection");
        };
        let entry = &first.entries()[entry_id.index()];
        assert_eq!(entry.members.len(), 2);
        assert_eq!(entry.conditions.len(), 2);
    }
}
