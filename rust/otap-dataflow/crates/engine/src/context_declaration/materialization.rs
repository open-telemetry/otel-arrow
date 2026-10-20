// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime materialization for selected constant and randomness composites.
//!
//! This module resolves the existing logical [`ContextProjection`] model. It
//! intentionally does not provide general consumer bindings, indexed lookups,
//! projector transforms, or outbound propagation.

use std::fmt;
use std::sync::Arc;

use otel_arrow_dfe_config::context::ContextEntryName;
use otel_arrow_dfe_config::context_policy::{
    ContextEntryDeclaration, ContextEntryPart, ContextRandomnessKind,
};
use otel_arrow_dfe_config::error::Error;
use uuid::{NoContext, Timestamp, Uuid};

use super::{
    ContextDomain, ContextEntryId, ContextEntryLayout, ContextFieldLayout, ContextLayout,
    ContextMemberSource, ContextProjection,
};

/// Immutable plan for materializing selected composites with domainless members.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextMaterializationPlan {
    layout: Arc<ContextLayout>,
    declarations: Arc<[Arc<ContextEntryDeclaration>]>,
}

impl ContextMaterializationPlan {
    /// Compiles selected constant/randomness composites and their referenced fields.
    pub fn compile(declarations: &[ContextEntryDeclaration]) -> Result<Self, Error> {
        let mut declarations = declarations
            .iter()
            .filter(|declaration| {
                declaration.definition.0.iter().any(|part| {
                    matches!(
                        part,
                        ContextEntryPart::Constant { .. } | ContextEntryPart::Randomness { .. }
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        declarations.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        let fields = declarations
            .iter()
            .flat_map(|declaration| declaration.definition.0.iter())
            .filter_map(ContextEntryPart::referenced_source)
            .map(|(domain, name)| ContextFieldLayout {
                name: match domain {
                    ContextDomain::TransportHeader => name
                        .as_str()
                        .to_ascii_lowercase()
                        .try_into()
                        .expect("lowercase context entry name remains valid"),
                    ContextDomain::AuthorizedIdentity => name.clone(),
                },
                domain,
            });
        let layout = ContextLayout::compile(fields, &declarations)?;
        let declarations = declarations
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>()
            .into();

        Ok(Self {
            layout: Arc::new(layout),
            declarations,
        })
    }

    /// Returns whether this plan has no composites to materialize.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.layout.entries().is_empty()
    }

    /// Returns the logical layout used by this plan.
    #[must_use]
    pub fn layout(&self) -> &ContextLayout {
        &self.layout
    }

    fn entries(
        &self,
    ) -> impl Iterator<Item = (&ContextEntryLayout, &Arc<ContextEntryDeclaration>)> {
        self.layout.entries().iter().zip(self.declarations.iter())
    }

    fn declaration(&self, entry: ContextEntryId) -> Option<&Arc<ContextEntryDeclaration>> {
        self.declarations.get(entry.index())
    }
}

/// Text or binary representation retained for one materialized value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextValueKind {
    /// UTF-8 text.
    Text,
    /// Arbitrary binary data.
    Binary,
}

/// Whether a source value was scalar or explicitly multi-valued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextValueCardinality {
    /// A scalar source value.
    One,
    /// A multi-valued source, including a one-element collection.
    Many,
}

/// One owned value retained in a materialized context.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedContextValue {
    original_name: Option<Box<str>>,
    kind: ContextValueKind,
    bytes: Box<[u8]>,
}

impl fmt::Debug for MaterializedContextValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterializedContextValue")
            .field("original_name", &self.original_name)
            .field("kind", &self.kind)
            .field("byte_len", &self.bytes.len())
            .finish()
    }
}

impl MaterializedContextValue {
    /// Creates one materialized value.
    #[must_use]
    pub fn new(
        original_name: Option<Box<str>>,
        kind: ContextValueKind,
        bytes: impl Into<Box<[u8]>>,
    ) -> Self {
        Self {
            original_name,
            kind,
            bytes: bytes.into(),
        }
    }

    /// Returns the original wire name when retained.
    #[must_use]
    pub fn original_name(&self) -> Option<&str> {
        self.original_name.as_deref()
    }

    /// Returns whether the value is text or binary.
    #[must_use]
    pub const fn kind(&self) -> ContextValueKind {
        self.kind
    }

    /// Returns the raw value bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the value as UTF-8 when valid.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }
}

/// Values supplied by one primitive or materialized member.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedContextValues {
    cardinality: ContextValueCardinality,
    values: Box<[MaterializedContextValue]>,
}

impl fmt::Debug for MaterializedContextValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterializedContextValues")
            .field("cardinality", &self.cardinality)
            .field("value_count", &self.values.len())
            .finish()
    }
}

impl MaterializedContextValues {
    /// Creates a cardinality-preserving value collection.
    ///
    /// Multi-valued sources may be present with no values. Scalar sources must
    /// contain exactly one value.
    #[must_use]
    pub fn new(
        cardinality: ContextValueCardinality,
        values: impl Into<Box<[MaterializedContextValue]>>,
    ) -> Option<Self> {
        let values = values.into();
        match cardinality {
            ContextValueCardinality::One if values.len() != 1 => None,
            ContextValueCardinality::One | ContextValueCardinality::Many => Some(Self {
                cardinality,
                values,
            }),
        }
    }

    fn text(value: impl Into<Box<[u8]>>) -> Self {
        Self {
            cardinality: ContextValueCardinality::One,
            values: Box::new([MaterializedContextValue::new(
                None,
                ContextValueKind::Text,
                value,
            )]),
        }
    }

    /// Returns the source cardinality.
    #[must_use]
    pub const fn cardinality(&self) -> ContextValueCardinality {
        self.cardinality
    }

    /// Returns the retained values.
    #[must_use]
    pub fn values(&self) -> &[MaterializedContextValue] {
        &self.values
    }
}

/// Supplies primitive field values to the materializer.
pub trait ContextValueSource {
    /// Copies the values for one logical primitive field.
    fn values(&self, field: &ContextFieldLayout) -> Option<MaterializedContextValues>;
}

/// One named member of a materialized composite.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedContextMember {
    name: ContextEntryName,
    values: MaterializedContextValues,
}

impl fmt::Debug for MaterializedContextMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterializedContextMember")
            .field("name", &self.name)
            .field("values", &self.values)
            .finish()
    }
}

impl MaterializedContextMember {
    /// Returns the configured member name.
    #[must_use]
    pub const fn name(&self) -> &ContextEntryName {
        &self.name
    }

    /// Returns the member values.
    #[must_use]
    pub const fn values(&self) -> &MaterializedContextValues {
        &self.values
    }
}

/// One present composite and its materialized members.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedContextEntry {
    name: ContextEntryName,
    members: Box<[MaterializedContextMember]>,
    provenance: Arc<ContextEntryDeclaration>,
}

impl fmt::Debug for MaterializedContextEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterializedContextEntry")
            .field("name", &self.name)
            .field("members", &self.members)
            .finish()
    }
}

impl MaterializedContextEntry {
    /// Returns the composite name.
    #[must_use]
    pub const fn name(&self) -> &ContextEntryName {
        &self.name
    }

    /// Returns all materialized members in canonical name order.
    #[must_use]
    pub fn members(&self) -> &[MaterializedContextMember] {
        &self.members
    }

    /// Finds one materialized member by configured name.
    #[must_use]
    pub fn member(&self, name: &ContextEntryName) -> Option<&MaterializedContextMember> {
        self.members
            .binary_search_by(|member| member.name.cmp(name))
            .ok()
            .map(|index| &self.members[index])
    }

    fn matches_declaration(&self, declaration: &Arc<ContextEntryDeclaration>) -> bool {
        Arc::ptr_eq(&self.provenance, declaration)
            || self.provenance.as_ref() == declaration.as_ref()
    }
}

#[derive(PartialEq, Eq)]
struct MaterializedContextStorage {
    entries: Box<[MaterializedContextEntry]>,
}

/// Immutable composites materialized for one pdata context.
///
/// The empty form is one nullable thin pointer and allocates only when at
/// least one selected composite is present.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct MaterializedContext {
    storage: Option<Arc<MaterializedContextStorage>>,
}

impl fmt::Debug for MaterializedContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterializedContext")
            .field("entries", &self.entries())
            .finish()
    }
}

impl MaterializedContext {
    /// Returns whether no composites are materialized.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_none()
    }

    /// Returns all present composites in canonical name order.
    #[must_use]
    pub fn entries(&self) -> &[MaterializedContextEntry] {
        self.storage
            .as_deref()
            .map_or(&[], |storage| storage.entries.as_ref())
    }

    /// Finds one present composite.
    #[must_use]
    pub fn entry(&self, name: &ContextEntryName) -> Option<&MaterializedContextEntry> {
        self.entries()
            .binary_search_by(|entry| entry.name.cmp(name))
            .ok()
            .map(|index| &self.entries()[index])
    }

    fn is_materialized(
        &self,
        entry: &ContextEntryLayout,
        declaration: &Arc<ContextEntryDeclaration>,
    ) -> bool {
        self.entry(&entry.name)
            .is_some_and(|materialized| materialized.matches_declaration(declaration))
    }

    /// Materializes composites not already present under the same declaration.
    ///
    /// Existing entries are retained only when their originating declaration
    /// matches the destination plan. A same-named entry with different
    /// provenance is removed before the destination presence gate is evaluated.
    ///
    /// A retained entry is a snapshot: repeated calls with the same declaration
    /// do not reread source values or re-evaluate its presence gate.
    pub fn materialize(
        &mut self,
        plan: &ContextMaterializationPlan,
        source: &impl ContextValueSource,
    ) {
        if plan.is_empty() {
            return;
        }
        if plan
            .entries()
            .all(|(entry, declaration)| self.is_materialized(entry, declaration))
        {
            return;
        }

        let fields = plan
            .layout
            .fields()
            .iter()
            .map(|field| source.values(field))
            .collect::<Vec<_>>();
        let mut entries = self.entries().to_vec();
        for (entry, declaration) in plan.entries() {
            if let Some(index) = entries
                .iter()
                .position(|materialized| materialized.name == entry.name)
            {
                if entries[index].matches_declaration(declaration) {
                    continue;
                }
                _ = entries.remove(index);
            }
            if !entry.conditions.iter().all(|condition| {
                fields[condition.field.index()]
                    .as_ref()
                    .is_some_and(|values| {
                        values
                            .values
                            .iter()
                            .any(|value| value.bytes.as_ref() == condition.value.as_ref())
                    })
            }) {
                continue;
            }
            if !entry.members.iter().all(|member| match member.source {
                ContextMemberSource::Field(field) => fields[field.index()].is_some(),
                ContextMemberSource::Constant(_) | ContextMemberSource::Randomness(_) => true,
            }) {
                continue;
            }

            let mut members = Vec::with_capacity(entry.members.len());
            for member in &entry.members {
                let values = match &member.source {
                    ContextMemberSource::Field(field) => fields[field.index()]
                        .clone()
                        .expect("required field presence was checked"),
                    ContextMemberSource::Constant(value) => {
                        MaterializedContextValues::text(value.as_bytes().to_vec())
                    }
                    ContextMemberSource::Randomness(ContextRandomnessKind::Uuid7) => {
                        MaterializedContextValues::text(
                            Uuid::new_v7(Timestamp::now(NoContext))
                                .to_string()
                                .into_bytes(),
                        )
                    }
                };
                members.push(MaterializedContextMember {
                    name: member.name.clone(),
                    values,
                });
            }
            entries.push(MaterializedContextEntry {
                name: entry.name.clone(),
                members: members.into_boxed_slice(),
                provenance: Arc::clone(declaration),
            });
        }
        entries.sort_unstable_by(|left, right| left.name.cmp(&right.name));
        self.storage = (!entries.is_empty()).then(|| {
            Arc::new(MaterializedContextStorage {
                entries: entries.into_boxed_slice(),
            })
        });
    }

    /// Resolves a materialized composite projection.
    ///
    /// Primitive projections are outside this resolver's intentionally narrow
    /// scope and return `None`.
    #[must_use]
    pub fn resolve<'a>(
        &'a self,
        plan: &'a ContextMaterializationPlan,
        projection: &'a ContextProjection,
    ) -> Option<MaterializedContextProjection<'a>> {
        let ContextProjection::Composite {
            layout,
            entry,
            members,
        } = projection
        else {
            return None;
        };
        let layout = plan.layout.projected_entry(layout, *entry, members)?;
        let declaration = plan.declaration(*entry)?;
        let materialized = self.entry(&layout.name)?;
        if !materialized.matches_declaration(declaration) {
            return None;
        }
        members
            .iter()
            .all(|member| materialized.member(&member.name).is_some())
            .then_some(MaterializedContextProjection {
                entry: materialized,
                members,
            })
    }
}

/// Borrowed minimal resolver result for an existing composite projection.
#[derive(Clone, Copy, Debug)]
pub struct MaterializedContextProjection<'a> {
    entry: &'a MaterializedContextEntry,
    members: &'a [super::ContextMember],
}

impl<'a> MaterializedContextProjection<'a> {
    /// Returns the enclosing composite name.
    #[must_use]
    pub const fn entry_name(&self) -> &ContextEntryName {
        &self.entry.name
    }

    /// Visits selected members in projection order.
    pub fn visit_members(&self, mut visit: impl FnMut(&'a MaterializedContextMember)) {
        for selected in self.members {
            visit(
                self.entry
                    .member(&selected.name)
                    .expect("resolved projection member is materialized"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::context_policy::{ContextEntryDefinition, ContextScope};
    use std::collections::BTreeMap;

    fn name(value: &str) -> ContextEntryName {
        value.try_into().expect("valid context entry name")
    }

    fn declaration(parts: Vec<ContextEntryPart>) -> ContextEntryDeclaration {
        ContextEntryDeclaration {
            scope: ContextScope::Engine,
            name: name("request"),
            definition: ContextEntryDefinition(parts),
        }
    }

    #[derive(Default)]
    struct TestSource {
        values: BTreeMap<(ContextDomain, ContextEntryName), MaterializedContextValues>,
    }

    impl ContextValueSource for TestSource {
        fn values(&self, field: &ContextFieldLayout) -> Option<MaterializedContextValues> {
            self.values
                .get(&(field.domain, field.name.clone()))
                .cloned()
        }
    }

    fn text(value: &str) -> MaterializedContextValues {
        MaterializedContextValues::text(value.as_bytes().to_vec())
    }

    /// Scenario: a selected composite contains only configured domainless values.
    /// Guarantees: constants and UUID-v7 randomness materialize without primitive inputs.
    #[test]
    fn materializes_domainless_members() {
        let plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::Constant {
                name: name("route"),
                value: "otlp-http-json".to_owned(),
            },
            ContextEntryPart::Randomness {
                name: name("id"),
                value: ContextRandomnessKind::Uuid7,
            },
        ])])
        .expect("plan compiles");
        let mut context = MaterializedContext::default();

        context.materialize(&plan, &TestSource::default());

        let entry = context.entry(&name("request")).expect("entry is present");
        assert_eq!(
            entry
                .member(&name("route"))
                .expect("constant member")
                .values()
                .values()[0]
                .as_str(),
            Some("otlp-http-json")
        );
        let generated = entry
            .member(&name("id"))
            .expect("random member")
            .values()
            .values()[0]
            .as_str()
            .expect("UUID is text");
        assert_eq!(
            Uuid::parse_str(generated)
                .expect("valid UUID")
                .get_version_num(),
            7
        );
    }

    /// Scenario: a random member is materialized repeatedly for one context.
    /// Guarantees: the generated UUID remains stable for the context lifetime.
    #[test]
    fn retains_generated_value() {
        let plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::Randomness {
                name: name("id"),
                value: ContextRandomnessKind::Uuid7,
            },
        ])])
        .expect("plan compiles");
        let mut context = MaterializedContext::default();
        context.materialize(&plan, &TestSource::default());
        let first = context.clone();

        context.materialize(&plan, &TestSource::default());

        assert_eq!(context, first);
    }

    /// Scenario: separately compiled plans contain the same randomness declaration.
    /// Guarantees: structural declaration provenance retains the generated UUID across plans.
    #[test]
    fn retains_generated_value_across_identical_plans() {
        let declaration = declaration(vec![ContextEntryPart::Randomness {
            name: name("id"),
            value: ContextRandomnessKind::Uuid7,
        }]);
        let first_plan = ContextMaterializationPlan::compile(std::slice::from_ref(&declaration))
            .expect("first plan compiles");
        let second_plan =
            ContextMaterializationPlan::compile(&[declaration]).expect("second plan compiles");
        let mut context = MaterializedContext::default();
        context.materialize(&first_plan, &TestSource::default());
        let first = context.clone();

        context.materialize(&second_plan, &TestSource::default());

        assert_eq!(context, first);
    }

    /// Scenario: source values and a satisfied presence gate change after materialization.
    /// Guarantees: the same declaration retains its original member snapshot and presence.
    #[test]
    fn retains_source_snapshot_for_same_declaration() {
        let plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::TransportHeader {
                name: name("workspace"),
                store_as: None,
            },
            ContextEntryPart::TransportHeaderMatch {
                name: name("environment"),
                value: "production".to_owned(),
            },
            ContextEntryPart::Constant {
                name: name("route"),
                value: "dedicated".to_owned(),
            },
        ])])
        .expect("plan compiles");
        let mut source = TestSource::default();
        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("workspace")),
            text("acme"),
        );
        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("environment")),
            text("production"),
        );
        let mut context = MaterializedContext::default();
        context.materialize(&plan, &source);

        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("workspace")),
            text("other"),
        );
        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("environment")),
            text("staging"),
        );
        context.materialize(&plan, &source);

        let entry = context
            .entry(&name("request"))
            .expect("snapshot remains present");
        assert_eq!(
            entry
                .member(&name("workspace"))
                .expect("workspace member")
                .values()
                .values()[0]
                .as_str(),
            Some("acme")
        );
    }

    /// Scenario: a detached context reaches a same-named destination declaration with a new gate.
    /// Guarantees: mismatched provenance removes the old value and re-evaluates the destination.
    #[test]
    fn re_evaluates_same_name_entry_with_different_provenance() {
        let upstream_plan =
            ContextMaterializationPlan::compile(&[declaration(vec![ContextEntryPart::Constant {
                name: name("route"),
                value: "upstream".to_owned(),
            }])])
            .expect("upstream plan compiles");
        let destination_plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::TransportHeaderMatch {
                name: name("environment"),
                value: "production".to_owned(),
            },
            ContextEntryPart::Constant {
                name: name("route"),
                value: "destination".to_owned(),
            },
        ])])
        .expect("destination plan compiles");
        let projection = destination_plan
            .layout()
            .resolve_member(&name("request"), &name("route"))
            .expect("destination projection resolves");
        let mut context = MaterializedContext::default();
        context.materialize(&upstream_plan, &TestSource::default());
        let mut detached = context.clone();

        assert!(detached.resolve(&destination_plan, &projection).is_none());
        detached.materialize(&destination_plan, &TestSource::default());

        assert!(detached.entry(&name("request")).is_none());
        assert!(detached.resolve(&destination_plan, &projection).is_none());

        let mut source = TestSource::default();
        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("environment")),
            text("production"),
        );
        detached.materialize(&destination_plan, &source);

        let entry = detached
            .entry(&name("request"))
            .expect("destination entry is present");
        assert_eq!(
            entry
                .member(&name("route"))
                .expect("destination route")
                .values()
                .values()[0]
                .as_str(),
            Some("destination")
        );
    }

    /// Scenario: two independent pdata contexts use the same randomness plan.
    /// Guarantees: each context receives its own UUID-v7 value.
    #[test]
    fn generates_distinct_values_per_context() {
        let plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::Randomness {
                name: name("id"),
                value: ContextRandomnessKind::Uuid7,
            },
        ])])
        .expect("plan compiles");
        let mut first = MaterializedContext::default();
        let mut second = MaterializedContext::default();

        first.materialize(&plan, &TestSource::default());
        second.materialize(&plan, &TestSource::default());

        assert_ne!(first, second);
    }

    /// Scenario: a constant member has one required field and one matching condition.
    /// Guarantees: missing fields or failed conditions omit the whole composite.
    #[test]
    fn enforces_composite_presence_gate() {
        let declaration = declaration(vec![
            ContextEntryPart::TransportHeader {
                name: name("workspace"),
                store_as: None,
            },
            ContextEntryPart::TransportHeaderMatch {
                name: name("environment"),
                value: "production".to_owned(),
            },
            ContextEntryPart::Constant {
                name: name("route"),
                value: "dedicated".to_owned(),
            },
        ]);
        let plan = ContextMaterializationPlan::compile(&[declaration]).expect("plan compiles");
        let mut source = TestSource::default();
        let mut context = MaterializedContext::default();

        context.materialize(&plan, &source);
        assert!(context.is_empty());

        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("workspace")),
            text("acme"),
        );
        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("environment")),
            text("staging"),
        );
        context.materialize(&plan, &source);
        assert!(context.is_empty());

        _ = source.values.insert(
            (ContextDomain::TransportHeader, name("environment")),
            text("production"),
        );
        context.materialize(&plan, &source);
        assert!(context.entry(&name("request")).is_some());
    }

    /// Scenario: a caller resolves one member from a materialized composite.
    /// Guarantees: the minimal resolver returns only the existing projection selection.
    #[test]
    fn resolves_existing_composite_projection() {
        let plan = ContextMaterializationPlan::compile(&[declaration(vec![
            ContextEntryPart::Constant {
                name: name("route"),
                value: "otlp-http-json".to_owned(),
            },
            ContextEntryPart::Randomness {
                name: name("id"),
                value: ContextRandomnessKind::Uuid7,
            },
        ])])
        .expect("plan compiles");
        let projection = plan
            .layout()
            .resolve_member(&name("request"), &name("route"))
            .expect("projection resolves");
        let mut context = MaterializedContext::default();
        context.materialize(&plan, &TestSource::default());
        let resolved = context
            .resolve(&plan, &projection)
            .expect("projection is materialized");
        let mut selected = Vec::new();

        resolved.visit_members(|member| selected.push(member.name().clone()));

        assert_eq!(selected, [name("route")]);
    }

    /// Scenario: a projection from an independently compiled layout is resolved against this plan.
    /// Guarantees: layout-local entry identifiers cannot select an unrelated materialized entry.
    #[test]
    fn rejects_projection_from_another_layout() {
        let declaration = declaration(vec![ContextEntryPart::Constant {
            name: name("route"),
            value: "otlp-http-json".to_owned(),
        }]);
        let plan = ContextMaterializationPlan::compile(std::slice::from_ref(&declaration))
            .expect("plan compiles");
        let other_plan =
            ContextMaterializationPlan::compile(&[declaration]).expect("other plan compiles");
        let projection = other_plan
            .layout()
            .resolve_member(&name("request"), &name("route"))
            .expect("projection resolves");
        let mut context = MaterializedContext::default();
        context.materialize(&plan, &TestSource::default());

        assert!(context.resolve(&plan, &projection).is_none());
    }

    /// Scenario: a caller alters the descriptors in a projection from the correct layout.
    /// Guarantees: resolution accepts only canonical members selected from that layout.
    #[test]
    fn rejects_projection_with_tampered_members() {
        let plan =
            ContextMaterializationPlan::compile(&[declaration(vec![ContextEntryPart::Constant {
                name: name("route"),
                value: "otlp-http-json".to_owned(),
            }])])
            .expect("plan compiles");
        let projection = plan
            .layout()
            .resolve_member(&name("request"), &name("route"))
            .expect("projection resolves");
        let ContextProjection::Composite { layout, entry, .. } = projection else {
            panic!("member projection must be composite");
        };
        let projection = ContextProjection::Composite {
            layout,
            entry,
            members: Box::new([crate::context_declaration::ContextMember {
                name: name("route"),
                source: ContextMemberSource::Constant("altered".into()),
            }]),
        };
        let mut context = MaterializedContext::default();
        context.materialize(&plan, &TestSource::default());

        assert!(context.resolve(&plan, &projection).is_none());
    }
}
