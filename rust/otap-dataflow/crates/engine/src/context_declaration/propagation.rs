// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Header propagation bindings compiled from configuration and visible context declarations.

use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
use otel_arrow_dfe_config::context_policy::{ContextEntryDeclaration, ContextEntryPart};
use otel_arrow_dfe_config::transport_headers::TransportHeaderSource;
use otel_arrow_dfe_config::transport_headers_policy::{
    HeaderPropagationPolicy, NameStrategy, PropagatedHeader, PropagationAction, PropagationDefault,
    PropagationOverride, PropagationSelectorType,
};
use smallvec::SmallVec;
use std::collections::HashMap;

use super::{ContextMaterializationPlan, ContextProjection, ContextValueKind, MaterializedContext};

/// An exporter's propagation policy with qualified context references validated.
#[doc(hidden)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PreparedHeaderPropagationPolicy {
    default: PropagationDefault,
    overrides: Vec<PropagationOverride>,
    prepared_named: Vec<PreparedNamedPropagation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PreparedNamedPropagation {
    composite_name: ContextEntryName,
    member_name: ContextEntryName,
    source_name: Option<ContextEntryName>,
    output_name: ContextEntryName,
}

/// Runtime propagation policy bound to the pipeline's materialization plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompiledHeaderPropagationPolicy {
    default: PropagationDefault,
    overrides: Vec<PropagationOverride>,
    materialization_plan: ContextMaterializationPlan,
    compiled_named: Vec<CompiledNamedPropagation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledNamedPropagation {
    source_name: Option<ContextEntryName>,
    output_name: ContextEntryName,
    projection: ContextProjection,
}

impl PreparedHeaderPropagationPolicy {
    /// Resolves qualified named selectors from visible composite declarations.
    ///
    /// This step validates every qualified `composite:member` reference and
    /// records the binding information needed to compile runtime projections.
    /// It is safe to call for policies containing only unqualified selectors.
    pub fn compile(
        policy: HeaderPropagationPolicy,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, String> {
        let mut policy = Self {
            default: policy.default,
            overrides: policy.overrides,
            prepared_named: Vec::new(),
        };
        let Some(references) = policy.default.selector.named.as_ref() else {
            return Ok(policy);
        };
        let mut selected_sources = HashMap::<Box<str>, ContextEntryRef>::new();

        for reference in references {
            let Some(composite_name) = reference.scope() else {
                register_named_source(&mut selected_sources, reference.name(), reference)?;
                continue;
            };
            let declaration = declarations
                .iter()
                .find(|declaration| declaration.name.as_str() == composite_name.as_str())
                .ok_or_else(|| format!("unknown composite context entry `{composite_name}`"))?;

            let mut source_name = None;
            let mut selected = false;
            for part in &declaration.definition.0 {
                match part {
                    ContextEntryPart::Constant { name, .. } => {
                        if name == reference.name() {
                            selected = true;
                        }
                    }
                    ContextEntryPart::Randomness { name, .. } => {
                        if name == reference.name() {
                            selected = true;
                        }
                    }
                    ContextEntryPart::TransportHeader { name, store_as } => {
                        if store_as.as_ref().unwrap_or(name) == reference.name() {
                            source_name = Some(name.clone());
                            selected = true;
                        }
                    }
                    ContextEntryPart::AuthorizedIdentity { name, store_as } => {
                        if store_as.as_ref().unwrap_or(name) == reference.name() {
                            return Err(format!(
                                "context entry reference `{reference}` selects authorized-identity member `{name}`, which cannot be propagated as a transport header"
                            ));
                        }
                    }
                    ContextEntryPart::TransportHeaderMatch { .. } => {}
                }
            }

            if !selected {
                return Err(format!(
                    "context entry reference `{reference}` does not select a value-bearing member"
                ));
            }
            if let Some(source_name) = &source_name {
                register_named_source(&mut selected_sources, source_name, reference)?;
            }
            policy.prepared_named.push(PreparedNamedPropagation {
                composite_name: composite_name.clone(),
                member_name: reference.name().clone(),
                source_name,
                output_name: reference.name().clone(),
            });
        }
        Ok(policy)
    }

    /// Binds qualified selectors to projections from the pipeline materialization plan.
    pub(super) fn bind(
        self,
        materialization_plan: &ContextMaterializationPlan,
    ) -> Result<CompiledHeaderPropagationPolicy, String> {
        let Self {
            default,
            overrides,
            prepared_named,
        } = self;
        let compiled_named = prepared_named
            .into_iter()
            .filter(|prepared| !binding_suppressed_by_override(prepared, &overrides))
            .map(|prepared| {
                let projection = materialization_plan
                    .layout()
                    .resolve_member(&prepared.composite_name, &prepared.member_name)
                    .map_err(|error| error.to_string())?;
                Ok(CompiledNamedPropagation {
                    source_name: prepared.source_name,
                    output_name: prepared.output_name,
                    projection,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(CompiledHeaderPropagationPolicy {
            default,
            overrides,
            materialization_plan: materialization_plan.clone(),
            compiled_named,
        })
    }

    /// Visits composite names selected by qualified propagation references.
    pub(super) fn visit_composite_names(&self, mut visit: impl FnMut(&ContextEntryName)) {
        for binding in &self.prepared_named {
            if !binding_suppressed_by_override(binding, &self.overrides) {
                visit(&binding.composite_name);
            }
        }
    }

    /// Returns whether this entry is propagated with its original name.
    #[must_use]
    pub fn propagates_original_name(&self, name: &ContextEntryName) -> bool {
        let (action, name_strategy) = self.resolve_static_action_for_name(name);
        action == PropagationAction::Propagate && name_strategy == NameStrategy::Preserve
    }

    /// Returns whether an otherwise-unmentioned captured header uses its original name.
    #[must_use]
    pub fn propagates_original_name_by_default(&self) -> bool {
        self.default.selector.selector_type == PropagationSelectorType::AllCaptured
            && self.default.action == PropagationAction::Propagate
            && self.default.name == NameStrategy::Preserve
    }

    /// Visits names whose original-name disposition may differ from the default.
    pub fn visit_original_name_requirement_names(&self, mut visit: impl FnMut(&ContextEntryName)) {
        if let Some(names) = &self.default.selector.named {
            for name in names {
                if name.scope().is_none() {
                    visit(name.name());
                }
            }
        }
        for binding in &self.prepared_named {
            if let Some(source_name) = &binding.source_name {
                visit(source_name);
            }
        }
        for override_policy in &self.overrides {
            for name in &override_policy.match_rule.stored_names {
                visit(name);
            }
        }
    }

    fn resolve_static_action_for_name(
        &self,
        name: &ContextEntryName,
    ) -> (PropagationAction, NameStrategy) {
        if let Some(override_policy) = find_override(&self.overrides, name.as_str()) {
            return (
                override_policy.action,
                override_policy.name.unwrap_or(self.default.name),
            );
        }
        if self.prepared_named.iter().any(|binding| {
            binding
                .source_name
                .as_ref()
                .is_some_and(|source_name| name.as_str().eq_ignore_ascii_case(source_name.as_str()))
        }) {
            return (self.default.action, self.default.name);
        }
        resolve_static_action_for_name(&self.default, &[], name)
    }
}

impl CompiledHeaderPropagationPolicy {
    /// Compiles a standalone runtime policy and its materialization plan.
    ///
    /// Pipeline construction binds prepared policies to one shared plan instead.
    pub fn compile_standalone(
        policy: HeaderPropagationPolicy,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, String> {
        let prepared = PreparedHeaderPropagationPolicy::compile(policy, declarations)?;
        let materialization_plan =
            ContextMaterializationPlan::compile(declarations).map_err(|error| error.to_string())?;
        prepared.bind(&materialization_plan)
    }

    /// Returns the materialization plan used by qualified propagation projections.
    #[must_use]
    pub fn materialization_plan(&self) -> &ContextMaterializationPlan {
        &self.materialization_plan
    }

    /// Returns whether this policy has qualified composite selections.
    #[must_use]
    pub fn has_materialized_selections(&self) -> bool {
        !self.compiled_named.is_empty()
    }

    /// Returns whether this primitive entry is propagated with its original name.
    #[must_use]
    pub fn propagates_original_name(&self, name: &ContextEntryName) -> bool {
        let (action, name_strategy) =
            resolve_static_action_for_name(&self.default, &self.overrides, name);
        action == PropagationAction::Propagate && name_strategy == NameStrategy::Preserve
    }

    /// Returns borrowed primitive headers selected for propagation.
    ///
    /// Qualified composite selections require [`Self::propagate_materialized`].
    pub fn propagate<'a, H>(&'a self, headers: &'a H) -> impl Iterator<Item = PropagatedHeader<'a>>
    where
        H: TransportHeaderSource + ?Sized,
    {
        self.propagate_primitive(headers)
    }

    /// Returns borrowed primitive and materialized headers selected for propagation.
    ///
    /// Primitive selections read captured transport headers directly.
    /// Qualified selections read the receiver-materialized composite snapshot.
    pub fn propagate_materialized<'a, H>(
        &'a self,
        headers: &'a H,
        materialized: Option<&'a MaterializedContext>,
    ) -> impl Iterator<Item = PropagatedHeader<'a>>
    where
        H: TransportHeaderSource + ?Sized,
    {
        self.propagate_primitive(headers)
            .chain(self.materialized_headers(materialized))
    }

    fn propagate_primitive<'a, H>(
        &'a self,
        headers: &'a H,
    ) -> impl Iterator<Item = PropagatedHeader<'a>>
    where
        H: TransportHeaderSource + ?Sized,
    {
        headers.transport_headers().filter_map(move |header| {
            let (action, name_strategy) = self.resolve_primitive_action(header.name.as_str());
            if action == PropagationAction::Drop {
                return None;
            }
            let header_name = match name_strategy {
                NameStrategy::StoredName => header.name.as_str(),
                NameStrategy::Preserve => header.wire_name(),
            };
            Some(PropagatedHeader {
                header_name,
                value_kind: header.value.value_kind,
                value: header.value.bytes,
            })
        })
    }

    /// Returns qualified headers when no captured primitive headers are present.
    pub fn propagate_materialized_only<'a>(
        &'a self,
        materialized: Option<&'a MaterializedContext>,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        self.materialized_headers(materialized).into_iter()
    }

    fn materialized_headers<'a>(
        &'a self,
        materialized: Option<&'a MaterializedContext>,
    ) -> SmallVec<[PropagatedHeader<'a>; 8]> {
        let mut propagated = SmallVec::new();
        if let Some(materialized) = materialized {
            for binding in &self.compiled_named {
                if self.default.action == PropagationAction::Drop {
                    continue;
                }
                let Some(projection) =
                    materialized.resolve(&self.materialization_plan, &binding.projection)
                else {
                    continue;
                };
                projection.visit_members(|member| {
                    for value in member.values().values() {
                        let header_name = match self.default.name {
                            NameStrategy::StoredName => binding.output_name.as_str(),
                            NameStrategy::Preserve => value
                                .original_name()
                                .or_else(|| {
                                    binding
                                        .source_name
                                        .as_ref()
                                        .map(|source_name| source_name.as_str())
                                })
                                .unwrap_or(binding.output_name.as_str()),
                        };
                        propagated.push(PropagatedHeader {
                            header_name,
                            value_kind: match value.kind() {
                                ContextValueKind::Text => {
                                    otel_arrow_dfe_config::transport_headers::ValueKind::Text
                                }
                                ContextValueKind::Binary => {
                                    otel_arrow_dfe_config::transport_headers::ValueKind::Binary
                                }
                            },
                            value: value.bytes(),
                        });
                    }
                });
            }
        }
        propagated
    }

    fn resolve_primitive_action(&self, name: &str) -> (PropagationAction, NameStrategy) {
        if let Some(override_policy) = self.override_for_str(name) {
            return (
                override_policy.action,
                override_policy.name.unwrap_or(self.default.name),
            );
        }
        if self.default.selector.selects_primitive_header(name) {
            (self.default.action, self.default.name)
        } else {
            (PropagationAction::Drop, self.default.name)
        }
    }

    fn override_for_str(&self, name: &str) -> Option<&PropagationOverride> {
        find_override(&self.overrides, name)
    }
}

fn binding_suppressed_by_override(
    binding: &PreparedNamedPropagation,
    overrides: &[PropagationOverride],
) -> bool {
    binding
        .source_name
        .as_ref()
        .is_some_and(|source_name| find_override(overrides, source_name.as_str()).is_some())
}

fn find_override<'a>(
    overrides: &'a [PropagationOverride],
    name: &str,
) -> Option<&'a PropagationOverride> {
    overrides.iter().find(|override_policy| {
        override_policy
            .match_rule
            .stored_names
            .iter()
            .any(|stored| name.eq_ignore_ascii_case(stored.as_str()))
    })
}

fn resolve_static_action_for_name(
    default: &PropagationDefault,
    overrides: &[PropagationOverride],
    name: &ContextEntryName,
) -> (PropagationAction, NameStrategy) {
    let name = name.as_str();
    if let Some(override_policy) = find_override(overrides, name) {
        let name_strategy = override_policy.name.unwrap_or(default.name);
        return (override_policy.action, name_strategy);
    }

    let selected = default.selector.selects_primitive_header(name);
    if selected {
        (default.action, default.name)
    } else {
        (PropagationAction::Drop, default.name)
    }
}

fn register_named_source(
    selected_sources: &mut HashMap<Box<str>, ContextEntryRef>,
    source_name: &ContextEntryName,
    reference: &ContextEntryRef,
) -> Result<(), String> {
    let key: Box<str> = source_name.as_str().to_ascii_lowercase().into();
    if let Some(previous) = selected_sources.get(&key) {
        if previous.scope().is_none() && reference.scope().is_none() {
            return Ok(());
        }
        return Err(format!(
            "named context entry references `{previous}` and `{reference}` resolve to the same transport-header entry `{source_name}`"
        ));
    }
    let _ = selected_sources.insert(key, reference.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_declaration::{
        ContextDomain, ContextFieldLayout, ContextValueCardinality, ContextValueSource,
        MaterializedContextValue, MaterializedContextValues,
    };
    use otel_arrow_dfe_config::transport_headers::{
        TransportHeaderRef, TransportHeaders, TransportHeadersIter, ValueKind,
    };
    use otel_arrow_dfe_config::transport_headers_policy::{
        CaptureDefaults, CaptureRule, HeaderCapturePolicy, PropagationMatch, PropagationSelector,
    };
    use otel_arrow_dfe_config::{context_policy, transport_headers};
    use std::cell::Cell;
    use uuid::Uuid;

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn header(
        normal: &str,
        wire_name: &str,
        value: impl AsRef<[u8]>,
    ) -> transport_headers::TransportHeader {
        transport_headers::TransportHeader::captured(
            context_name(normal),
            wire_name,
            true,
            ValueKind::Text,
            value.as_ref(),
        )
    }

    struct TestValueSource<'a> {
        headers: &'a TransportHeaders,
    }

    impl ContextValueSource for TestValueSource<'_> {
        fn values(&self, field: &ContextFieldLayout) -> Option<MaterializedContextValues> {
            let values = match field.domain {
                ContextDomain::TransportHeader => self
                    .headers
                    .iter()
                    .filter(|header| {
                        header
                            .name
                            .as_str()
                            .eq_ignore_ascii_case(field.name.as_str())
                    })
                    .map(|header| {
                        MaterializedContextValue::new(
                            header.value.original_name.map(Box::from),
                            match header.value.value_kind {
                                ValueKind::Text => ContextValueKind::Text,
                                ValueKind::Binary => ContextValueKind::Binary,
                            },
                            header.value.bytes,
                        )
                    })
                    .collect::<Vec<_>>(),
                ContextDomain::AuthorizedIdentity => vec![MaterializedContextValue::new(
                    None,
                    ContextValueKind::Text,
                    b"customer".as_slice(),
                )],
            };
            if values.is_empty() {
                return None;
            }
            let cardinality = if values.len() == 1 {
                ContextValueCardinality::One
            } else {
                ContextValueCardinality::Many
            };
            MaterializedContextValues::new(cardinality, values.into_boxed_slice())
        }
    }

    fn materialize(
        policy: &CompiledHeaderPropagationPolicy,
        headers: &TransportHeaders,
    ) -> MaterializedContext {
        let mut materialized = MaterializedContext::default();
        materialized.materialize(policy.materialization_plan(), &TestValueSource { headers });
        materialized
    }

    struct CountingHeaderSource {
        headers: TransportHeaders,
        reads: Cell<usize>,
    }

    struct CountingHeaderIter<'a> {
        inner: TransportHeadersIter<'a>,
        reads: &'a Cell<usize>,
    }

    impl<'a> Iterator for CountingHeaderIter<'a> {
        type Item = TransportHeaderRef<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            let header = self.inner.next()?;
            self.reads.set(self.reads.get() + 1);
            Some(header)
        }
    }

    impl TransportHeaderSource for CountingHeaderSource {
        type Iter<'a>
            = CountingHeaderIter<'a>
        where
            Self: 'a;

        fn transport_headers(&self) -> Self::Iter<'_> {
            CountingHeaderIter {
                inner: self.headers.iter(),
                reads: &self.reads,
            }
        }
    }

    /// Scenario: an exporter uses the default compiled policy or compiles default settings.
    /// Guarantees: both policies are equal and propagate no captured headers.
    #[test]
    fn default_propagation_policy_matches_compilation() {
        let policy = CompiledHeaderPropagationPolicy::default();
        assert_eq!(
            policy,
            CompiledHeaderPropagationPolicy::compile_standalone(
                HeaderPropagationPolicy::default(),
                &[]
            )
            .expect("default propagation policy compiles")
        );
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant", "X-Tenant", b"acme"));
        assert_eq!(policy.propagate(&headers).count(), 0);
    }

    /// Scenario: overrides change naming or drop selected headers.
    /// Guarantees: only propagated headers using `Preserve` require original names.
    #[test]
    fn propagation_policy_resolves_original_name_per_entry() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name: NameStrategy::Preserve,
                ..PropagationDefault::default()
            },
            vec![
                PropagationOverride {
                    match_rule: PropagationMatch {
                        stored_names: vec![context_name("stored")],
                    },
                    action: PropagationAction::Propagate,
                    name: Some(NameStrategy::StoredName),
                    on_error: None,
                },
                PropagationOverride {
                    match_rule: PropagationMatch {
                        stored_names: vec![context_name("dropped")],
                    },
                    action: PropagationAction::Drop,
                    name: None,
                    on_error: None,
                },
            ],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");

        assert!(policy.propagates_original_name(&context_name("preserved")));
        assert!(!policy.propagates_original_name(&context_name("stored")));
        assert!(!policy.propagates_original_name(&context_name("dropped")));
    }

    /// Scenario: a named selector references a composite transport-header member with a condition.
    /// Guarantees: names ignore ASCII case, values match exactly, all conditions pass, duplicate
    /// values use any-match semantics, and unrelated value members are not evaluated.
    #[test]
    fn composite_transport_header_propagation_requires_matching_conditions() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id]
  action: propagate
  name: stored_name
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");

        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("WORKSPACE"),
            b"acme",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("Environment"),
            b"staging",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"Production",
        ));
        let materialized = materialize(&policy, &headers);
        assert_eq!(
            policy
                .propagate_materialized(&headers, Some(&materialized))
                .count(),
            0
        );

        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"production",
        ));
        let materialized = materialize(&policy, &headers);
        assert_eq!(
            policy
                .propagate_materialized(&headers, Some(&materialized))
                .count(),
            0
        );

        headers.push(transport_headers::TransportHeader::text(
            context_name("REGION"),
            b"US-EAST",
        ));
        let materialized = materialize(&policy, &headers);
        assert_eq!(
            policy
                .propagate_materialized(&headers, Some(&materialized))
                .count(),
            0
        );

        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: every condition matches but the selected transport-header member is absent.
    /// Guarantees: the composite binding remains inactive and emits no header.
    #[test]
    fn composite_transport_header_propagation_requires_selected_member() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id]
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"production",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));

        let materialized = materialize(&policy, &headers);
        assert_eq!(
            policy
                .propagate_materialized(&headers, Some(&materialized))
                .count(),
            0
        );
    }

    /// Scenario: duplicate selected-source values are propagated across repeated calls.
    /// Guarantees: one activation result applies to every source value and is not retained
    /// after the propagation iterator is discarded.
    #[test]
    fn composite_transport_header_propagation_shares_activation_per_call() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id]
  name: stored_name
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"beta",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"production",
        ));

        let materialized = materialize(&policy, &headers);
        assert_eq!(
            policy
                .propagate_materialized(&headers, Some(&materialized))
                .count(),
            0
        );

        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();
        assert_eq!(propagated.len(), 2);
        assert!(
            propagated
                .iter()
                .all(|header| header.header_name == "workspace_id")
        );
        assert_eq!(propagated[0].value, b"acme");
        assert_eq!(propagated[1].value, b"beta");
    }

    /// Scenario: two selected members share one materialized composite presence gate.
    /// Guarantees: both values propagate from the same receiver-materialized snapshot.
    #[test]
    fn composite_transport_header_propagation_shares_conditions_across_bindings() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id, product_user:account_id]
"#,
        )
        .expect("valid propagation policy");
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: transport_header
      name: workspace
      store_as: workspace_id
    - type: transport_header
      name: account
      store_as: account_id
    - type: transport_header_match
      name: environment
      value: production
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            policy,
            &[ContextEntryDeclaration {
                scope: context_policy::ContextScope::Engine,
                name,
                definition,
            }],
        )
        .expect("composite selectors compile");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"production",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));
        headers.push(transport_headers::TransportHeader::text(
            context_name("account"),
            b"acct",
        ));
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].value, b"acme");
        assert_eq!(propagated[1].value, b"acct");
    }

    /// Scenario: an override selects a primitive source whose composite conditions are absent.
    /// Guarantees: override precedence propagates the primitive header independently.
    #[test]
    fn composite_transport_header_conditions_do_not_constrain_overrides() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id]
  name: stored_name
overrides:
  - match:
      stored_names: [workspace]
    action: propagate
    name: stored_name
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: a qualified transport member's primitive source has an explicit override.
    /// Guarantees: preparation omits the composite requirement and binding omits the qualified
    /// runtime projection because primitive propagation owns the source's configured outcome.
    #[test]
    fn primitive_override_suppresses_qualified_binding_during_compilation() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id]
overrides:
  - match:
      stored_names: [workspace]
    action: propagate
"#,
        )
        .expect("valid propagation policy");
        let prepared = PreparedHeaderPropagationPolicy::compile(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");
        let mut selected_composites = Vec::new();

        prepared.visit_composite_names(|name| selected_composites.push(name.clone()));
        let compiled = prepared
            .bind(&ContextMaterializationPlan::default())
            .expect("suppressed binding needs no materialization plan");

        assert!(selected_composites.is_empty());
        assert!(!compiled.has_materialized_selections());
    }

    fn conditional_product_user_declaration() -> ContextEntryDeclaration {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: authorized_identity
      name: customer_id
    - type: transport_header
      name: workspace
      store_as: workspace_id
    - type: transport_header_match
      name: environment
      value: production
    - type: transport_header_match
      name: region
      value: us-east
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        }
    }

    /// Scenario: a qualified propagation selector names an unknown composite.
    /// Guarantees: binding compilation rejects the unresolved reference before runtime.
    #[test]
    fn composite_transport_header_propagation_rejects_unknown_composite() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [missing:workspace]
"#,
        )
        .expect("valid propagation policy");

        let error = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect_err("unknown composite must fail");
        assert!(error.contains("unknown composite context entry `missing`"));
    }

    /// Scenario: a qualified selector names a header in a composite that also has a constant.
    /// Guarantees: the unrelated constant does not block compilation or header propagation.
    #[test]
    fn propagates_header_with_constant_sibling() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  route:
    - type: constant
      name: route_name
      value: otlp-http-json
    - type: transport_header
      name: workspace
      store_as: workspace_id
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [route:workspace_id]
  action: propagate
  name: stored_name
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("header member compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: a qualified transport member aliases a primitive whose wire and stored names
    /// are identical, so capture has no separate original name to retain.
    /// Guarantees: `preserve` falls back to the primitive source name instead of the member alias.
    #[test]
    fn qualified_preserve_falls_back_to_primitive_source_name() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  tenant:
    - type: transport_header
      name: workspace
      store_as: workspace_id
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [tenant:workspace_id]
  name: preserve
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("qualified transport member compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));
        let materialized = materialize(&policy, &headers);

        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: only a qualified transport member requires preserving a distinct ingress name.
    /// Guarantees: capture retains the wire name and qualified export emits it unchanged.
    #[test]
    fn qualified_preserve_retains_original_name_from_capture_to_export() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  tenant:
    - type: transport_header
      name: workspace
      store_as: workspace_id
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let propagation: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [tenant:workspace_id]
  name: preserve
"#,
        )
        .expect("valid propagation policy");
        let prepared = PreparedHeaderPropagationPolicy::compile(
            propagation,
            std::slice::from_ref(&declaration),
        )
        .expect("qualified propagation policy compiles");
        let capture = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("x-workspace")],
                store_as: Some(context_name("workspace")),
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|name| prepared.propagates_original_name(name));
        let mut headers = TransportHeaders::new();

        assert_eq!(
            capture.capture_from_pairs(
                [("X-Workspace", b"acme".as_slice())].into_iter(),
                &mut headers,
            ),
            None
        );
        assert_eq!(
            headers.get(0).expect("captured workspace").wire_name(),
            "X-Workspace"
        );

        let plan = ContextMaterializationPlan::compile(&[declaration])
            .expect("materialization plan compiles");
        let policy = prepared.bind(&plan).expect("prepared policy binds");
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Workspace");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: capture stores the same mixed-case name seen on the wire, while a qualified
    /// composite source references that header using different capitalization.
    /// Guarantees: case-insensitive lookup does not replace the captured spelling on export.
    #[test]
    fn qualified_preserve_retains_captured_stored_name_spelling() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  tenant:
    - type: transport_header
      name: x-workspace
      store_as: workspace_id
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let propagation: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [tenant:workspace_id]
  name: preserve
"#,
        )
        .expect("valid propagation policy");
        let prepared = PreparedHeaderPropagationPolicy::compile(
            propagation,
            std::slice::from_ref(&declaration),
        )
        .expect("qualified propagation policy compiles");
        let capture = HeaderCapturePolicy::new(
            CaptureDefaults::default(),
            vec![CaptureRule {
                match_names: vec![context_name("X-Workspace")],
                store_as: Some(context_name("X-Workspace")),
                sensitive: false,
                value_kind: None,
            }],
        )
        .compile(|name| prepared.propagates_original_name(name));
        let mut headers = TransportHeaders::new();

        assert_eq!(
            capture.capture_from_pairs(
                [("X-Workspace", b"acme".as_slice())].into_iter(),
                &mut headers,
            ),
            None
        );
        let captured = headers.get(0).expect("captured workspace");
        assert_eq!(captured.name.as_str(), "X-Workspace");
        assert_eq!(captured.value.original_name, None);

        let plan = ContextMaterializationPlan::compile(&[declaration])
            .expect("materialization plan compiles");
        let policy = prepared.bind(&plan).expect("prepared policy binds");
        let mut materialized = MaterializedContext::default();
        materialized.set_transport_headers(headers.clone());
        materialized
            .materialize_with_stored_transport(&plan, &TestValueSource { headers: &headers });
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Workspace");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: a qualified selector names a header in a composite with randomness.
    /// Guarantees: unselected randomness does not block compilation or header propagation.
    #[test]
    fn propagates_header_with_randomness_sibling() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  idempotency:
    - type: randomness
      name: id
      value: uuid7
    - type: transport_header
      name: workspace
      store_as: workspace_id
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [idempotency:workspace_id]
  action: propagate
  name: stored_name
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("header member compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: a qualified propagation selector names a configured constant member.
    /// Guarantees: the configured value propagates using the member name as its wire name.
    #[test]
    fn composite_transport_header_propagation_emits_constant_member() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  route:
    - type: constant
      name: route_name
      value: otlp-http-json
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [route:route_name]
"#,
        )
        .expect("valid propagation policy");

        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("constant member compiles");
        let headers = TransportHeaders::new();
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "route_name");
        assert_eq!(propagated[0].value, b"otlp-http-json");
    }

    /// Scenario: a qualified constant selector uses the default `drop` action.
    /// Guarantees: materialization does not bypass the configured propagation action.
    #[test]
    fn composite_transport_header_propagation_drops_constant_member() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  route:
    - type: constant
      name: route_name
      value: otlp-http-json
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [route:route_name]
  action: drop
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("constant member compiles");
        let headers = TransportHeaders::new();
        let materialized = materialize(&policy, &headers);

        assert_eq!(
            policy
                .propagate_materialized_only(Some(&materialized))
                .count(),
            0
        );
    }

    /// Scenario: a qualified propagation selector names a configured randomness member.
    /// Guarantees: one UUID-v7 value materializes and propagates under the member name.
    #[test]
    fn composite_transport_header_propagation_emits_randomness_member() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  idempotency:
    - type: randomness
      name: id
      value: uuid7
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [idempotency:id]
"#,
        )
        .expect("valid propagation policy");

        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("randomness member compiles");
        let headers = TransportHeaders::new();
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "id");
        let value = std::str::from_utf8(propagated[0].value).expect("UUID is UTF-8");
        assert_eq!(value.len(), 36);
        assert_eq!(value.as_bytes()[14], b'7');
    }

    /// Scenario: qualified generated members share names with primitive drop overrides.
    /// Guarantees: legacy stored-name overrides remain scoped to captured primitive headers,
    /// while selected constant and randomness members follow the default propagation action.
    #[test]
    fn generated_members_do_not_match_primitive_overrides() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  delivery:
    - type: constant
      name: route
      value: primary
    - type: randomness
      name: id
      value: uuid7
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [delivery:route, delivery:id]
overrides:
  - match:
      stored_names: [route, id]
    action: drop
"#,
        )
        .expect("valid propagation policy");

        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("generated members compile");
        let headers = TransportHeaders::new();
        let materialized = materialize(&policy, &headers);
        let propagated = policy
            .propagate_materialized_only(Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].header_name, "route");
        assert_eq!(propagated[0].value, b"primary");
        assert_eq!(propagated[1].header_name, "id");
    }

    /// Scenario: one policy combines primitive, qualified header, constant, and randomness
    /// selections with primitive drop and propagate overrides.
    /// Guarantees: primitive overrides take precedence, dropping a source suppresses its
    /// qualified representation, while selected constant and UUID members share the result.
    #[test]
    fn mixed_propagation_preserves_primitive_override_precedence() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  delivery:
    - type: transport_header
      name: workspace
      store_as: workspace_id
    - type: constant
      name: route
      value: primary
    - type: randomness
      name: id
      value: uuid7
"#,
        )
        .expect("valid context policy");
        let (name, definition) = context.entries.into_iter().next().expect("declaration");
        let declaration = ContextEntryDeclaration {
            scope: context_policy::ContextScope::Engine,
            name,
            definition,
        };
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [keep, delivery:workspace_id, delivery:route, delivery:id]
  action: propagate
  name: stored_name
overrides:
  - match:
      stored_names: [workspace]
    action: drop
  - match:
      stored_names: [forced]
    action: propagate
"#,
        )
        .expect("valid propagation policy");
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect("mixed propagation policy compiles");
        let mut headers = TransportHeaders::new();
        headers.push(header("keep", "X-Keep", b"kept"));
        headers.push(header("workspace", "X-Workspace", b"acme"));
        headers.push(header("forced", "X-Forced", b"forced"));
        headers.push(header("ignored", "X-Ignored", b"ignored"));
        let materialized = materialize(&policy, &headers);

        let propagated = policy
            .propagate_materialized(&headers, Some(&materialized))
            .collect::<Vec<_>>();

        assert_eq!(propagated.len(), 4);
        assert_eq!(propagated[0].header_name, "keep");
        assert_eq!(propagated[0].value, b"kept");
        assert_eq!(propagated[1].header_name, "forced");
        assert_eq!(propagated[1].value, b"forced");
        assert_eq!(propagated[2].header_name, "route");
        assert_eq!(propagated[2].value, b"primary");
        assert_eq!(propagated[3].header_name, "id");
        let uuid = std::str::from_utf8(propagated[3].value).expect("UUID is UTF-8");
        assert_eq!(
            Uuid::parse_str(uuid).expect("valid UUID").get_version_num(),
            7
        );
    }

    /// Scenario: a named selector repeats an unqualified header using identical and varied case.
    /// Guarantees: equivalent unconditional selections remain accepted for compatibility.
    #[test]
    fn transport_header_propagation_accepts_unqualified_duplicate_sources() {
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [workspace, WORKSPACE, workspace]
"#,
        )
        .expect("valid propagation policy");

        let _compiled = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("equivalent unqualified duplicates must remain valid");
    }

    /// Scenario: named selectors use an unqualified entry and a composite alias for its source.
    /// Guarantees: startup rejects the ambiguous duplicate source instead of bypassing conditions.
    #[test]
    fn composite_transport_header_propagation_rejects_mixed_duplicate_source() {
        let declaration = composite_declarations_for_duplicate_source()
            .into_iter()
            .next()
            .expect("tenant_a declaration");
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [workspace, tenant_a:workspace_id]
"#,
        )
        .expect("valid propagation policy");

        let error = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[declaration])
            .expect_err("duplicate source must fail");
        assert!(error.contains("`workspace` and `tenant_a:workspace_id`"));
        assert!(error.contains("same transport-header entry `workspace`"));
    }

    /// Scenario: two qualified selectors resolve to the same primitive header with varied case.
    /// Guarantees: startup detects the collision using transport-header name semantics.
    #[test]
    fn composite_transport_header_propagation_rejects_qualified_duplicate_source() {
        let declarations = composite_declarations_for_duplicate_source();
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [tenant_a:workspace_id, tenant_b:workspace_id]
"#,
        )
        .expect("valid propagation policy");

        let error = CompiledHeaderPropagationPolicy::compile_standalone(policy, &declarations)
            .expect_err("duplicate source must fail");
        assert!(error.contains("`tenant_a:workspace_id` and `tenant_b:workspace_id`"));
        assert!(error.contains("same transport-header entry `WORKSPACE`"));
    }

    fn composite_declarations_for_duplicate_source() -> Vec<ContextEntryDeclaration> {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  tenant_a:
    - type: transport_header
      name: workspace
      store_as: workspace_id
    - type: transport_header_match
      name: environment
      value: production
  tenant_b:
    - type: transport_header
      name: WORKSPACE
      store_as: workspace_id
    - type: transport_header_match
      name: region
      value: us-east
"#,
        )
        .expect("valid context policy");

        context
            .entries
            .into_iter()
            .map(|(name, definition)| ContextEntryDeclaration {
                scope: context_policy::ContextScope::Engine,
                name,
                definition,
            })
            .collect()
    }

    // -- Propagation policy tests --------------------------------------------

    /// Scenario: propagation selects all captured headers.
    /// Guarantees: every header keeps its original wire name.
    #[test]
    fn propagate_all_captured_default() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
        assert_eq!(propagated[1].header_name, "X-Request-Id");
    }

    /// Scenario: an all-captured primitive policy receives more headers than the materialized
    /// result buffer's inline capacity.
    /// Guarantees: creating the propagation iterator reads nothing, and each output is produced
    /// lazily without buffering the complete primitive result.
    #[test]
    fn primitive_propagation_remains_lazy_above_inline_capacity() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");
        let mut headers = TransportHeaders::new();
        for index in 0..12 {
            headers.push(header(
                &format!("header-{index}"),
                &format!("X-Header-{index}"),
                index.to_string().as_bytes(),
            ));
        }
        let source = CountingHeaderSource {
            headers,
            reads: Cell::new(0),
        };

        let mut propagated = policy.propagate(&source);

        assert_eq!(source.reads.get(), 0);
        assert_eq!(
            propagated
                .next()
                .expect("first propagated header")
                .header_name,
            "X-Header-0"
        );
        assert_eq!(source.reads.get(), 1);
        assert_eq!(propagated.count(), 11);
        assert_eq!(source.reads.get(), 12);
    }

    /// Scenario: a drop override excludes authorization.
    /// Guarantees: other headers still propagate with their original names.
    #[test]
    fn propagate_override_drops_auth() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("authorization")],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            }],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("authorization", "Authorization", b"Bearer secret"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: a `none` selector has one propagation override.
    /// Guarantees: only the overridden header is propagated.
    #[test]
    fn propagate_selector_none_drops_all_unless_override() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::None,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![context_name("tenant_id")],
                },
                action: PropagationAction::Propagate,
                name: None,
                on_error: None,
            }],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: propagation uses `StoredName` for a renamed header.
    /// Guarantees: the stored name replaces the original wire name.
    #[test]
    fn propagate_stored_name_strategy() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::AllCaptured,
                    named: None,
                },
                name: NameStrategy::StoredName,
                ..PropagationDefault::default()
            },
            vec![],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "tenant_id");
    }

    /// Scenario: a named selector lists one captured entry.
    /// Guarantees: only that entry is propagated.
    #[test]
    fn propagate_named_selector() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: Some(vec![context_name("tenant_id").into()]),
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let policy = CompiledHeaderPropagationPolicy::compile_standalone(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }
}
