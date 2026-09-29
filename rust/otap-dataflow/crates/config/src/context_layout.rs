// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deterministic, configuration-only context layout compilation.
//!
//! Runtime construction and hashing remain separate: the compiler resolves
//! source provenance and ordered presence requirements without changing the
//! current request-context storage or activation of unused declarations.

use std::collections::{BTreeMap, BTreeSet};

use crate::context::{ContextEntryName, ContextEntryRef};
use crate::context_policy::{ContextEntryDeclaration, ContextEntryPart, ContextScope};
use crate::error::Error;

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: message.into(),
    }
}

/// Trusted provenance and physical storage domain of a primitive field.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContextSource {
    /// Untrusted transport metadata.
    TransportHeader,
    /// Verified authorization claim.
    AuthorizedIdentity,
}

/// Name and provenance assigned by a capture policy or component producer.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContextPrimitive {
    /// Exact configured stored name.
    pub name: ContextEntryName,
    /// Physical storage domain.
    pub source: ContextSource,
}

/// Dense primitive identity within one compiled pipeline layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextFieldId(usize);

impl ContextFieldId {
    /// Returns the index into the layout's primitive fields.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Dense entry identity within one compiled pipeline layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContextEntryId(usize);

impl ContextEntryId {
    /// Returns the index into the layout's entries.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// A grouping member and its resolved primitive.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMember {
    /// Output member name (after applying `store_as`).
    pub name: ContextEntryName,
    /// Referenced primitive field.
    pub field: ContextFieldId,
}

/// One exact condition controlling atomic entry presence.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ContextCondition {
    /// Referenced transport-header field.
    pub field: ContextFieldId,
    /// Required exact byte value.
    pub value: Box<[u8]>,
}

/// One atomic logical entry, including members and presence conditions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextEntryLayout {
    /// Logical entry name.
    pub name: ContextEntryName,
    /// Declaring scope, or `None` for a primitive entry.
    pub scope: Option<ContextScope>,
    /// Members in definition order; all must be present for this entry to exist.
    pub members: Box<[ContextMember]>,
    /// Canonically ordered conditions; all must match for this entry to exist.
    pub conditions: Box<[ContextCondition]>,
}

/// Immutable, deterministic source and grouping layout for one pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayout {
    fields: Box<[ContextPrimitive]>,
    entries: Box<[ContextEntryLayout]>,
    by_name: BTreeMap<ContextEntryName, ContextEntryId>,
}

/// A compiled entry or qualified member reference.
///
/// `presence` always identifies the parent entry. Reading one member never
/// bypasses that entry's all-member presence requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBinding {
    /// Parent entry whose atomic presence must be checked.
    pub presence: ContextEntryId,
    /// Fields in their configured projection order.
    pub fields: Box<[ContextFieldId]>,
}

impl ContextLayout {
    /// Compile one pipeline's visible definitions and known primitive sources.
    ///
    /// Callers supply source names from the *effective* capture/producer
    /// declarations; unused context policies need not be compiled yet.
    pub fn compile(
        sources: impl IntoIterator<Item = ContextPrimitive>,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, Error> {
        let fields: Box<[_]> = sources
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut entries = Vec::with_capacity(fields.len() + declarations.len());
        let mut by_name = BTreeMap::new();
        let mut primitive_sources = BTreeMap::new();
        for (index, field) in fields.iter().enumerate() {
            let name = field.name.clone();
            if let Some(existing) = primitive_sources.insert(name.clone(), field.source) {
                return Err(invalid(format!(
                    "context source `{name}` is produced as both {existing:?} and {:?}",
                    field.source
                )));
            }
            _ = by_name.insert(name.clone(), ContextEntryId(index));
            entries.push(ContextEntryLayout {
                name: name.clone(),
                scope: None,
                members: Box::new([ContextMember {
                    name,
                    field: ContextFieldId(index),
                }]),
                conditions: Box::new([]),
            });
        }

        let mut ordered = declarations.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| {
            left.scope
                .cmp(&right.scope)
                .then_with(|| left.name.cmp(&right.name))
        });
        for declaration in ordered {
            if primitive_sources.contains_key(&declaration.name)
                || by_name.contains_key(&declaration.name)
            {
                return Err(invalid(format!(
                    "context entry `{}` conflicts with a visible source or grouping entry",
                    declaration.name
                )));
            }
            if declaration.definition.0.is_empty() {
                return Err(invalid(format!(
                    "context entry `{}` must contain at least one member",
                    declaration.name
                )));
            }
            let mut names = BTreeSet::new();
            let mut members = Vec::with_capacity(declaration.definition.0.len());
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                let (source, condition_value) = match part {
                    ContextEntryPart::TransportHeader { .. } => {
                        (ContextSource::TransportHeader, None)
                    }
                    ContextEntryPart::AuthorizedIdentity { .. } => {
                        (ContextSource::AuthorizedIdentity, None)
                    }
                    ContextEntryPart::TransportHeaderMatch { value, .. } => {
                        (ContextSource::TransportHeader, Some(value))
                    }
                };
                let reference = part.source();
                if reference.scope().is_some() {
                    return Err(invalid(format!(
                        "context entry `{}` cannot use nested reference `{reference}`",
                        declaration.name
                    )));
                }
                let field = fields
                    .binary_search(&ContextPrimitive {
                        name: reference.name().clone(),
                        source,
                    })
                    .map(ContextFieldId)
                    .map_err(|_| {
                        invalid(format!(
                            "context entry `{}` requires unavailable {:?} source `{reference}`",
                            declaration.name, source
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
                if !names.insert(name.clone()) {
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
                        "context entry `{}` repeats source `{reference}`",
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
            _ = by_name.insert(declaration.name.clone(), id);
            entries.push(ContextEntryLayout {
                name: declaration.name.clone(),
                scope: Some(declaration.scope.clone()),
                members: members.into_boxed_slice(),
                conditions: conditions.into_boxed_slice(),
            });
        }
        Ok(Self {
            fields,
            entries: entries.into_boxed_slice(),
            by_name,
        })
    }

    /// Primitive fields in stable source-and-name order.
    #[must_use]
    pub fn fields(&self) -> &[ContextPrimitive] {
        &self.fields
    }

    /// Entries in stable primitive order, then scope-and-name order.
    #[must_use]
    pub fn entries(&self) -> &[ContextEntryLayout] {
        &self.entries
    }

    /// Resolve a whole entry or qualified member with its parent's presence gate.
    pub fn bind(&self, reference: &ContextEntryRef) -> Result<ContextBinding, Error> {
        let (entry_name, member_name) = match reference.scope() {
            Some(entry) => (entry, Some(reference.name())),
            None => (reference.name(), None),
        };
        let id = *self
            .by_name
            .get(entry_name)
            .ok_or_else(|| invalid(format!("unknown context entry `{entry_name}`")))?;
        let entry = &self.entries[id.0];
        let fields = match member_name {
            Some(member) => {
                if entry.scope.is_none() {
                    return Err(invalid(format!(
                        "standalone context entry `{entry_name}` has no qualified members"
                    )));
                }
                vec![
                    entry
                        .members
                        .iter()
                        .find(|candidate| &candidate.name == member)
                        .ok_or_else(|| invalid(format!("unknown context member `{reference}`")))?
                        .field,
                ]
            }
            None => entry.members.iter().map(|member| member.field).collect(),
        };
        Ok(ContextBinding {
            presence: id,
            fields: fields.into_boxed_slice(),
        })
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

    fn sources() -> Vec<ContextPrimitive> {
        vec![
            ContextPrimitive {
                name: name("workspace"),
                source: ContextSource::TransportHeader,
            },
            ContextPrimitive {
                name: name("customer"),
                source: ContextSource::AuthorizedIdentity,
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
    /// Guarantees: whole and qualified bindings use one atomic presence gate.
    #[test]
    fn mixed_group_and_member_share_presence() {
        let layout = ContextLayout::compile(sources(), &[entry()]).expect("valid layout");
        let whole = layout
            .bind(&reference("product_user"))
            .expect("whole entry");
        let member = layout
            .bind(&reference("product_user:customer_id"))
            .expect("qualified member");
        assert_eq!(whole.presence, member.presence);
        assert_eq!(whole.fields.len(), 2);
        assert_eq!(&whole.fields[..1], &member.fields[..]);
        assert_eq!(
            layout.fields()[whole.fields[0].index()].source,
            ContextSource::AuthorizedIdentity
        );
        assert_eq!(
            layout.fields()[whole.fields[1].index()].source,
            ContextSource::TransportHeader
        );
    }

    /// Scenario: source declarations are supplied in different iteration orders.
    /// Guarantees: compiled IDs and selected field order stay deterministic.
    #[test]
    fn source_input_order_does_not_change_layout() {
        let mut reverse = sources();
        reverse.reverse();
        assert_eq!(
            ContextLayout::compile(sources(), &[entry()]).expect("layout"),
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
            ContextLayout::compile(sources(), &declarations).expect("layout"),
            ContextLayout::compile(sources(), &reversed).expect("layout")
        );
    }

    /// Scenario: a grouping references a name in the wrong source domain.
    /// Guarantees: compilation fails rather than interpreting a trusted claim as a header.
    #[test]
    fn provenance_mismatch_is_rejected() {
        let only_header = [ContextPrimitive {
            name: name("customer"),
            source: ContextSource::TransportHeader,
        }];
        assert!(ContextLayout::compile(only_header, &[entry()]).is_err());
    }

    /// Scenario: a visible derived entry conflicts with an existing source name.
    /// Guarantees: source and derived entry IDs cannot shadow each other.
    #[test]
    fn visible_name_collision_is_rejected() {
        let mut conflict = entry();
        conflict.name = name("workspace");
        assert!(ContextLayout::compile(sources(), &[conflict]).is_err());
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
        assert!(ContextLayout::compile(sources(), &[nested]).is_err());
    }

    /// Scenario: a qualified reference selects a member not defined by its parent.
    /// Guarantees: missing members never fall back to a standalone field.
    #[test]
    fn unknown_qualified_member_is_rejected() {
        let layout = ContextLayout::compile(sources(), &[entry()]).expect("layout");
        assert!(layout.bind(&reference("product_user:missing")).is_err());
        assert!(layout.bind(&reference("workspace:customer")).is_err());
    }

    /// Scenario: transport-header and authorized-identity capture produce the same stored name.
    /// Guarantees: layout compilation rejects the collision instead of requiring consumer aliases.
    #[test]
    fn source_domain_collision_is_rejected() {
        let fields = [
            ContextPrimitive {
                name: name("customer"),
                source: ContextSource::AuthorizedIdentity,
            },
            ContextPrimitive {
                name: name("customer"),
                source: ContextSource::TransportHeader,
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
        let mut fields = sources();
        fields.extend([
            ContextPrimitive {
                name: name("environment"),
                source: ContextSource::TransportHeader,
            },
            ContextPrimitive {
                name: name("region"),
                source: ContextSource::TransportHeader,
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
        let entry = &first.entries()[first
            .bind(&reference("product_user"))
            .unwrap()
            .presence
            .index()];
        assert_eq!(entry.members.len(), 2);
        assert_eq!(entry.conditions.len(), 2);
    }
}
