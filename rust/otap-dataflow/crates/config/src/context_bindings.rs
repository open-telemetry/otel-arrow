// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compiled entry layouts and typed access. Configuration names are resolved before binding.

use crate::context::{ContextEntryName, ContextEntryRef};
use crate::context_policy::{
    ContextEntryDeclaration, ContextEntryPart, ContextMatchMultiplicity, ContextScope,
};
use crate::error::Error;
use crate::transport_headers::{TransportHeader, TransportHeaders, ValueKind};
use crate::transport_headers_policy::{
    HeaderPropagationPolicy, NameStrategy, PropagatedHeader, PropagationAction,
    PropagationSelectorType,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[cfg(test)]
#[path = "context_bindings_tests.rs"]
mod tests;

/// An index into the immutable primitive-field layout carried with a context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextFieldId(pub(crate) usize);

/// An index into the immutable entry layout carried with a context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextEntryId(pub(crate) usize);

/// A primitive transport field declared by capture or a component producer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextPrimitive {
    /// Logical entry name (the capture rule's `store_as`, when supplied).
    pub entry: ContextEntryName,
    /// Normalized member identity, independent of retained wire-name spelling.
    pub field: ContextEntryName,
    /// Whether field access requires qualification, even for one-member groups.
    pub grouped: bool,
}

impl ContextPrimitive {
    /// Declares a standalone, singleton-field transport entry.
    #[must_use]
    pub fn standalone(name: ContextEntryName) -> Self {
        Self {
            entry: name.clone(),
            field: name,
            grouped: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextMember {
    pub(crate) name: ContextEntryName,
    pub(crate) source: ContextFieldId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextGuard {
    field: ContextFieldId,
    value: Box<[u8]>,
    multiplicity: ContextMatchMultiplicity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextEntryLayout {
    pub(crate) name: ContextEntryName,
    scope: ContextScope,
    pub(crate) members: Box<[ContextMember]>,
    grouped: bool,
    derived: bool,
    guards: Box<[ContextGuard]>,
}

/// Immutable layout shared by construction and consumer bindings.
///
/// Contexts retain this layout so an in-flight message cannot be interpreted
/// using unrelated slot numbers after live reconfiguration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextLayout {
    pub(crate) fields: Box<[ContextPrimitive]>,
    pub(crate) entries: Box<[ContextEntryLayout]>,
}

pub(crate) fn config_error(message: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: message.into(),
    }
}

impl ContextLayout {
    /// Returns the stable primitive identity assigned to a compiled field.
    #[must_use]
    pub fn primitive(&self, field: ContextFieldId) -> &ContextPrimitive {
        &self.fields[field.0]
    }

    pub(crate) fn visible_entries(&self, pipeline: &crate::PipelineKey) -> Arc<[ContextEntryId]> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.derived
                    && entry
                        .scope
                        .visible_from(pipeline.pipeline_group_id(), pipeline.pipeline_id())
            })
            .map(|(id, _)| ContextEntryId(id))
            .collect()
    }

    pub(crate) fn construction_entries(
        &self,
        pipeline: &crate::PipelineKey,
        available: &BTreeSet<ContextFieldId>,
    ) -> Arc<[ContextEntryId]> {
        self.visible_entries(pipeline)
            .iter()
            .copied()
            .filter(|id| {
                let entry = &self.entries[id.0];
                entry
                    .members
                    .iter()
                    .all(|member| available.contains(&member.source))
                    && entry
                        .guards
                        .iter()
                        .all(|guard| available.contains(&guard.field))
            })
            .collect()
    }
    /// Resolves complete primitive and scoped entry declarations into a deterministic layout.
    pub fn compile(
        primitives: BTreeSet<ContextPrimitive>,
        definitions: BTreeSet<ContextEntryDeclaration>,
    ) -> Result<Arc<Self>, Error> {
        let fields: Box<[_]> = primitives.into_iter().collect();
        let mut primitive_entries: BTreeMap<ContextEntryName, (bool, Vec<ContextMember>)> =
            BTreeMap::new();
        for (index, primitive) in fields.iter().enumerate() {
            if !primitive.grouped && primitive.entry != primitive.field {
                return Err(config_error(
                    "standalone context entries must have exactly their own named field",
                ));
            }
            if primitive.entry.contains(':') || primitive.field.contains(':') {
                return Err(config_error(
                    "context entry and field names must not contain `:`",
                ));
            }
            let (grouped, members) = primitive_entries
                .entry(primitive.entry.clone())
                .or_insert_with(|| (primitive.grouped, Vec::new()));
            if *grouped != primitive.grouped {
                return Err(config_error(format!(
                    "context entry `{}` is declared as both standalone and composite",
                    primitive.entry
                )));
            }
            members.push(ContextMember {
                name: primitive.field.clone(),
                source: ContextFieldId(index),
            });
        }
        let mut layout = Self {
            fields,
            entries: primitive_entries
                .into_iter()
                .map(|(name, (grouped, members))| ContextEntryLayout {
                    name,
                    scope: ContextScope::Engine,
                    members: members.into_boxed_slice(),
                    grouped,
                    derived: false,
                    guards: Box::default(),
                })
                .collect(),
        };
        let mut entries = layout.entries.to_vec();
        for declaration in definitions {
            if declaration.name.contains(':') {
                return Err(config_error(format!(
                    "context entry name `{}` must not contain `:`",
                    declaration.name
                )));
            }
            if entries.iter().any(|entry| {
                entry.name == declaration.name && scopes_overlap(&entry.scope, &declaration.scope)
            }) {
                return Err(config_error(format!(
                    "conflicting visible context entry `{}` in {:?}",
                    declaration.name, declaration.scope
                )));
            }
            let mut members = Vec::new();
            let mut guards = Vec::new();
            let mut names = BTreeSet::new();
            for part in &declaration.definition.0 {
                match part {
                    ContextEntryPart::TransportHeader { name, alias } => {
                        let (_, source) = layout.resolve_primitive_field(name)?;
                        let member_name = alias
                            .as_ref()
                            .unwrap_or_else(|| name.field().unwrap_or(name.entry()));
                        if member_name.contains(':') || !names.insert(member_name.clone()) {
                            return Err(config_error(format!(
                                "invalid or duplicate member `{member_name}` in `{}`",
                                declaration.name
                            )));
                        }
                        members.push(ContextMember {
                            name: member_name.clone(),
                            source,
                        });
                    }
                    ContextEntryPart::TransportHeaderMatch {
                        name,
                        value,
                        multiplicity,
                    } => {
                        let (_, field) = layout.resolve_primitive_field(name)?;
                        guards.push(ContextGuard {
                            field,
                            value: value.as_bytes().into(),
                            multiplicity: *multiplicity,
                        });
                    }
                }
            }
            if members.is_empty() {
                return Err(config_error(format!(
                    "context entry `{}` must contain at least one field",
                    declaration.name
                )));
            }
            entries.push(ContextEntryLayout {
                name: declaration.name,
                scope: declaration.scope,
                members: members.into_boxed_slice(),
                grouped: true,
                derived: true,
                guards: guards.into_boxed_slice(),
            });
        }
        layout.entries = entries.into_boxed_slice();
        Ok(Arc::new(layout))
    }

    fn resolve_primitive_field(
        &self,
        reference: &ContextEntryRef,
    ) -> Result<(ContextEntryId, ContextFieldId), Error> {
        let entry_id = self.entries.iter().position(|entry| {
            !entry.derived && &entry.name == reference.entry()
        }).ok_or_else(|| config_error(format!(
            "unknown primitive context entry `{}`; derived entries cannot be composite inputs",
            reference.entry()
        )))?;
        let field = self.resolve_member(ContextEntryId(entry_id), reference)?;
        Ok((ContextEntryId(entry_id), field))
    }

    /// Resolves an exact entry or member selection visible from one pipeline.
    pub fn resolve(
        &self,
        reference: &ContextEntryRef,
        pipeline: &crate::PipelineKey,
    ) -> Result<(ContextEntryId, Vec<ContextFieldId>), Error> {
        let index = self
            .entries
            .iter()
            .position(|entry| {
                &entry.name == reference.entry()
                    && entry
                        .scope
                        .visible_from(pipeline.pipeline_group_id(), pipeline.pipeline_id())
            })
            .ok_or_else(|| {
                config_error(format!(
                    "unknown or out-of-scope context entry `{}` in `{}`",
                    reference.entry(),
                    pipeline.as_string()
                ))
            })?;
        let entry = ContextEntryId(index);
        let fields = if reference.field().is_some() {
            vec![self.resolve_member(entry, reference)?]
        } else {
            self.entries[index]
                .members
                .iter()
                .map(|member| member.source)
                .collect()
        };
        Ok((entry, fields))
    }

    fn resolve_member(
        &self,
        entry: ContextEntryId,
        reference: &ContextEntryRef,
    ) -> Result<ContextFieldId, Error> {
        let layout = &self.entries[entry.0];
        match reference.field() {
            Some(name) => {
                if !layout.grouped {
                    return Err(config_error(format!(
                        "`{reference}` qualifies a standalone entry; use `{}`",
                        layout.name
                    )));
                }
                layout
                    .members
                    .iter()
                    .find(|member| &member.name == name)
                    .map(|member| member.source)
                    .ok_or_else(|| config_error(format!("unknown context member `{reference}`")))
            }
            None if !layout.grouped => Ok(layout.members[0].source),
            None => Err(config_error(format!(
                "`{reference}` selects a composite; a field binding requires `entry:field`"
            ))),
        }
    }

    /// Binds one field; multiplicity is handled by the consumer's access operation.
    pub fn bind_field(
        self: &Arc<Self>,
        reference: &ContextEntryRef,
        pipeline: &crate::PipelineKey,
    ) -> Result<ContextFieldBinding, Error> {
        let (entry, _) = self.resolve(reference, pipeline)?;
        let field = self.resolve_member(entry, reference)?;
        Ok(ContextFieldBinding {
            layout: self.clone(),
            entry,
            field,
        })
    }

    /// Resolves a primitive schema identity during source construction.
    pub fn primitive_field(&self, primitive: &ContextPrimitive) -> Result<ContextFieldId, Error> {
        self.fields
            .binary_search(primitive)
            .map(ContextFieldId)
            .map_err(|_| {
                config_error(format!(
                    "undeclared context field `{}:{}`",
                    primitive.entry, primitive.field
                ))
            })
    }

    /// Binds a component's declared standalone transport-field output.
    pub fn bind_producer(
        self: &Arc<Self>,
        name: &ContextEntryName,
        pipeline: &crate::PipelineKey,
    ) -> Result<ContextProducerBinding, Error> {
        let field = self.primitive_field(&ContextPrimitive::standalone(name.clone()))?;
        Ok(ContextProducerBinding {
            layout: self.clone(),
            field,
            entries: self.visible_entries(pipeline),
        })
    }

    /// Indexes prebuilt standalone headers at construction time, not in a message hot path.
    pub fn bind_headers(
        self: &Arc<Self>,
        headers: &mut TransportHeaders,
        pipeline: &crate::PipelineKey,
    ) -> Result<(), Error> {
        let fields = headers
            .iter()
            .map(|header| self.primitive_field(&ContextPrimitive::standalone(header.name.clone())))
            .collect::<Result<Vec<_>, _>>()?;
        let names_preserved: BTreeSet<_> = fields.iter().copied().collect();
        headers.bind_fields(
            self.clone(),
            self.visible_entries(pipeline),
            fields,
            names_preserved.into_iter().collect(),
        );
        Ok(())
    }
}

fn scopes_overlap(left: &ContextScope, right: &ContextScope) -> bool {
    match (left, right) {
        (ContextScope::Engine, _) | (_, ContextScope::Engine) => true,
        (ContextScope::Group(a), ContextScope::Group(b)) => a == b,
        (ContextScope::Group(a), ContextScope::Pipeline(b, _))
        | (ContextScope::Pipeline(b, _), ContextScope::Group(a)) => a == b,
        (ContextScope::Pipeline(ag, ap), ContextScope::Pipeline(bg, bp)) => ag == bg && ap == bp,
    }
}

/// Runtime access errors do not collapse into missing context.
#[derive(Debug, thiserror::Error)]
pub enum ContextAccessError {
    /// A context was not constructed through its source binding.
    #[error("context has no compiled field layout")]
    Unbound,
    /// Context and binding describe different schema generations.
    #[error("context layout is incompatible with this binding's configuration generation")]
    IncompatibleLayout,
    /// A newer consumer requires information omitted by the arrival's capture program.
    #[error("context binding requires an original header name that capture did not retain")]
    OriginalNameUnavailable,
    /// A scalar consumer was given a repeated field.
    #[error("single-value context binding received multiple values")]
    MultipleValues,
}

/// A component producer can append only the field that it declared.
#[derive(Debug, Clone)]
pub struct ContextProducerBinding {
    layout: Arc<ContextLayout>,
    field: ContextFieldId,
    entries: Arc<[ContextEntryId]>,
}

impl ContextProducerBinding {
    /// Appends one typed value and recomputes atomic composite presence.
    pub fn append(
        &self,
        context: &mut TransportHeaders,
        value: crate::transport_headers::TransportHeaderValue,
    ) -> Result<(), ContextAccessError> {
        if !context.is_empty() {
            let _ = context.context_index(&self.layout)?;
        }
        context.append_bound(
            self.layout.clone(),
            self.entries.clone(),
            self.field,
            TransportHeader {
                name: self.layout.fields[self.field.0].entry.clone(),
                value,
            },
        );
        Ok(())
    }
}

/// A typed field access retaining its parent entry's presence gate.
#[derive(Debug, Clone)]
pub struct ContextFieldBinding {
    layout: Arc<ContextLayout>,
    entry: ContextEntryId,
    field: ContextFieldId,
}

impl ContextFieldBinding {
    /// Returns all values in input order, or none when the parent entry is absent.
    pub fn values<'a>(
        &self,
        context: &'a TransportHeaders,
    ) -> Result<impl Iterator<Item = &'a TransportHeader>, ContextAccessError> {
        let index = if context.is_empty() {
            None
        } else {
            Some(context.context_index(&self.layout)?)
        };
        Ok(index.into_iter().flat_map(move |index| {
            index
                .values(self.field, context.as_slice())
                .filter(move |_| index.is_present(self.entry))
        }))
    }

    /// Returns one value, distinguishes absence, and rejects repeated values.
    pub fn single<'a>(
        &self,
        context: &'a TransportHeaders,
    ) -> Result<Option<&'a TransportHeader>, ContextAccessError> {
        let mut values = self.values(context)?;
        let first = values.next();
        if values.next().is_some() {
            return Err(ContextAccessError::MultipleValues);
        }
        Ok(first)
    }
}

/// An immutable message index shares primitive values instead of copying them into composites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextIndex {
    pub(crate) layout: Arc<ContextLayout>,
    pub(crate) fields: Vec<ContextFieldId>,
    pub(crate) names_preserved: Arc<[ContextFieldId]>,
    slots: Vec<ContextFieldId>,
    heads: Vec<usize>,
    next: Vec<usize>,
    present: Vec<ContextEntryId>,
}

impl ContextIndex {
    pub(crate) fn build(
        layout: Arc<ContextLayout>,
        entries: &[ContextEntryId],
        fields: Vec<ContextFieldId>,
        names_preserved: Arc<[ContextFieldId]>,
        headers: &[TransportHeader],
    ) -> Self {
        let mut slots = fields.clone();
        slots.sort_unstable();
        slots.dedup();
        let mut index = Self {
            heads: vec![usize::MAX; slots.len()],
            slots,
            next: vec![usize::MAX; headers.len()],
            fields,
            names_preserved,
            present: Vec::new(),
            layout,
        };
        // Reverse construction preserves the wire order within each field.
        for (offset, field) in index.fields.iter().enumerate().rev() {
            // Slots are constructed from exactly this field sequence.
            let slot = index
                .slots
                .binary_search(field)
                .expect("indexed source field");
            index.next[offset] = index.heads[slot];
            index.heads[slot] = offset;
        }
        for id in entries {
            let entry = &index.layout.entries[id.0];
            let fields_present = entry
                .members
                .iter()
                .all(|member| index.head(member.source).is_some());
            let present = fields_present
                && entry.guards.iter().all(|guard| {
                    let mut values = index.values(guard.field, headers).peekable();
                    if values.peek().is_none() {
                        return false;
                    }
                    let matches = |header: &TransportHeader| {
                        header.value.value_kind == ValueKind::Text
                            && header.value.bytes == guard.value
                    };
                    match guard.multiplicity {
                        ContextMatchMultiplicity::Any => values.any(matches),
                        ContextMatchMultiplicity::All => values.all(matches),
                    }
                });
            if present {
                index.present.push(*id);
            }
        }
        index
    }

    fn head(&self, field: ContextFieldId) -> Option<usize> {
        let slot = self.slots.binary_search(&field).ok()?;
        (self.heads[slot] != usize::MAX).then_some(self.heads[slot])
    }

    fn is_present(&self, entry: ContextEntryId) -> bool {
        // Primitive selections are tested only while visiting one of their values.
        !self.layout.entries[entry.0].derived || self.present.binary_search(&entry).is_ok()
    }

    fn values<'a>(
        &'a self,
        field: ContextFieldId,
        headers: &'a [TransportHeader],
    ) -> impl Iterator<Item = &'a TransportHeader> {
        std::iter::successors(self.head(field), |offset| {
            (self.next[*offset] != usize::MAX).then_some(self.next[*offset])
        })
        .map(|offset| &headers[offset])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PropagationStep {
    entry: ContextEntryId,
    action: PropagationAction,
    naming: NameStrategy,
}

/// A propagation binding resolves references, precedence, and naming before message processing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompiledHeaderPropagationPolicy {
    layout: Arc<ContextLayout>,
    fields: Box<[(ContextFieldId, Box<[PropagationStep]>)]>,
    original_names: Box<[ContextFieldId]>,
}

impl HeaderPropagationPolicy {
    /// Compiles an exact, multi-valued entry/member consumer.
    pub fn compile(
        &self,
        layout: Arc<ContextLayout>,
        pipeline: &crate::PipelineKey,
    ) -> Result<CompiledHeaderPropagationPolicy, Error> {
        self.validate().map_err(config_error)?;
        let mut fields = BTreeMap::<ContextFieldId, Vec<PropagationStep>>::new();
        let mut append = |reference: &ContextEntryRef, action, naming| -> Result<(), Error> {
            let (entry, selected) = layout.resolve(reference, pipeline)?;
            for field in selected {
                let steps = fields.entry(field).or_default();
                // A primitive selection is unconditional whenever this source field exists.
                if !steps
                    .iter()
                    .any(|step| !layout.entries[step.entry.0].derived)
                {
                    let step = PropagationStep {
                        entry,
                        action,
                        naming,
                    };
                    if !steps.contains(&step) {
                        steps.push(step);
                    }
                }
            }
            Ok(())
        };
        for rule in &self.overrides {
            for reference in &rule.match_rule.stored_names {
                append(
                    reference,
                    rule.action,
                    rule.name.unwrap_or(self.default.name),
                )?;
            }
        }
        match self.default.selector.selector_type {
            PropagationSelectorType::None => {}
            PropagationSelectorType::Named => {
                for reference in self.default.selector.named.iter().flatten() {
                    append(reference, self.default.action, self.default.name)?;
                }
            }
            PropagationSelectorType::AllCaptured => {
                for entry in layout.entries.iter().filter(|entry| !entry.derived) {
                    append(
                        &ContextEntryRef::try_from(entry.name.as_str())?,
                        self.default.action,
                        self.default.name,
                    )?;
                }
            }
        }
        let original_names = fields
            .iter()
            .filter_map(|(id, steps)| {
                steps
                    .iter()
                    .any(|step| {
                        step.action == PropagationAction::Propagate
                            && step.naming == NameStrategy::Preserve
                    })
                    .then_some(*id)
            })
            .collect();
        Ok(CompiledHeaderPropagationPolicy {
            layout,
            fields: fields
                .into_iter()
                .map(|(field, steps)| (field, steps.into_boxed_slice()))
                .collect(),
            original_names,
        })
    }
}

impl CompiledHeaderPropagationPolicy {
    fn actions(&self, field: ContextFieldId) -> Option<&[PropagationStep]> {
        self.fields
            .binary_search_by_key(&field, |(field, _)| *field)
            .ok()
            .map(|index| self.fields[index].1.as_ref())
    }

    /// Source fields whose effective output may require the original wire name.
    pub fn original_name_fields(&self) -> impl Iterator<Item = ContextFieldId> + '_ {
        self.original_names.iter().copied()
    }

    /// Emits each selected occurrence once, preserving duplicates and input order.
    pub fn propagate<'a>(
        &'a self,
        headers: &'a TransportHeaders,
    ) -> Result<impl Iterator<Item = PropagatedHeader<'a>>, ContextAccessError> {
        let index = if headers.is_empty() || self.fields.is_empty() {
            None
        } else {
            Some(headers.context_index(&self.layout)?)
        };
        if let Some(index) = index {
            for field in &index.slots {
                if index.names_preserved.binary_search(field).is_err() {
                    let step = self
                        .actions(*field)
                        .into_iter()
                        .flatten()
                        .find(|step| index.is_present(step.entry));
                    if step.is_some_and(|step| {
                        step.action == PropagationAction::Propagate
                            && step.naming == NameStrategy::Preserve
                    }) {
                        return Err(ContextAccessError::OriginalNameUnavailable);
                    }
                }
            }
        }
        Ok(headers
            .iter()
            .enumerate()
            .filter_map(move |(offset, header)| {
                let index = index?;
                let field = index.fields[offset];
                let step = self
                    .actions(field)?
                    .iter()
                    .find(|step| index.is_present(step.entry))?;
                if step.action == PropagationAction::Drop {
                    return None;
                }
                let header_name = match step.naming {
                    NameStrategy::StoredName if self.layout.entries[step.entry.0].derived => {
                        self.layout.entries[step.entry.0].name.as_str()
                    }
                    NameStrategy::StoredName => header.name.as_str(),
                    NameStrategy::Preserve => header
                        .value
                        .original_name
                        .as_deref()
                        .unwrap_or(header.name.as_str()),
                };
                Some(PropagatedHeader {
                    header_name,
                    value_kind: &header.value.value_kind,
                    value: &header.value.bytes,
                })
            }))
    }
}
