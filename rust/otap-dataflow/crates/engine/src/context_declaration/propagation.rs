// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Header propagation bindings compiled from configuration and visible context declarations.

use otel_arrow_dfe_config::context::{ContextEntryName, ContextEntryRef};
use otel_arrow_dfe_config::context_policy::{ContextEntryDeclaration, ContextEntryPart};
use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use otel_arrow_dfe_config::transport_headers_policy::{
    HeaderPropagationPolicy, NameStrategy, PropagatedHeader, PropagationAction, PropagationDefault,
    PropagationOverride, PropagationSelectorType,
};
use smallvec::SmallVec;
use std::collections::HashMap;

/// An exporter's propagation policy with qualified context references resolved.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompiledHeaderPropagationPolicy {
    default: PropagationDefault,
    overrides: Vec<PropagationOverride>,
    compiled_named: Vec<CompiledNamedPropagation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompiledNamedPropagation {
    source_name: ContextEntryName,
    output_name: ContextEntryName,
    conditions: Vec<CompiledTransportHeaderMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompiledTransportHeaderMatch {
    name: ContextEntryName,
    value: Box<[u8]>,
}

type ConditionMatchCache<'a> = SmallVec<[(&'a [CompiledTransportHeaderMatch], bool); 4]>;

impl CompiledHeaderPropagationPolicy {
    /// Resolves qualified named selectors from visible composite declarations.
    ///
    /// This step validates every qualified `composite:member` reference and
    /// installs the primitive transport-header bindings used by propagation.
    /// It is safe to call for policies containing only unqualified selectors.
    pub fn compile(
        policy: HeaderPropagationPolicy,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, String> {
        let mut policy = Self {
            default: policy.default,
            overrides: policy.overrides,
            compiled_named: Vec::new(),
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
            let mut conditions = Vec::new();
            for part in &declaration.definition.0 {
                match part {
                    ContextEntryPart::Constant { name, .. } => {
                        if name == reference.name() {
                            return Err(format!(
                                "context entry reference `{reference}` selects constant member `{name}`, which cannot be propagated until constant runtime integration is available"
                            ));
                        }
                    }
                    ContextEntryPart::Randomness { name, .. } => {
                        if name == reference.name() {
                            return Err(format!(
                                "context entry reference `{reference}` selects randomness member `{name}`, which cannot be propagated until randomness runtime integration is available"
                            ));
                        }
                    }
                    ContextEntryPart::TransportHeader { name, store_as } => {
                        if store_as.as_ref().unwrap_or(name) == reference.name() {
                            source_name = Some(name.clone());
                        }
                    }
                    ContextEntryPart::AuthorizedIdentity { name, store_as } => {
                        if store_as.as_ref().unwrap_or(name) == reference.name() {
                            return Err(format!(
                                "context entry reference `{reference}` selects authorized-identity member `{name}`, which cannot be propagated as a transport header"
                            ));
                        }
                    }
                    ContextEntryPart::TransportHeaderMatch { name, value } => {
                        conditions.push(CompiledTransportHeaderMatch {
                            name: name.clone(),
                            value: value.as_bytes().into(),
                        });
                    }
                }
            }
            conditions.sort_unstable();

            let source_name = source_name.ok_or_else(|| {
                format!(
                    "context entry reference `{reference}` does not select a transport-header member"
                )
            })?;
            register_named_source(&mut selected_sources, &source_name, reference)?;
            policy.compiled_named.push(CompiledNamedPropagation {
                source_name,
                output_name: reference.name().clone(),
                conditions,
            });
        }
        Ok(policy)
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
        for binding in &self.compiled_named {
            visit(&binding.source_name);
        }
        for override_policy in &self.overrides {
            for name in &override_policy.match_rule.stored_names {
                visit(name);
            }
        }
    }

    /// Returns borrowed headers selected for propagation.
    /// [`NameStrategy`] selects each header's original or stored name.
    /// Headers with [`PropagationAction::Drop`] are omitted.
    pub fn propagate<'a>(
        &'a self,
        headers: &'a TransportHeaders,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        let mut condition_matches = ConditionMatchCache::new();
        headers.iter().filter_map(move |header| {
            let (action, name_strategy, selected_name) = self.resolve_action_for_header(
                headers,
                header.name.as_str(),
                &mut condition_matches,
            );
            if action == PropagationAction::Drop {
                return None;
            }
            let header_name = match name_strategy {
                NameStrategy::StoredName => selected_name.unwrap_or(header.name.as_str()),
                NameStrategy::Preserve => header.wire_name(),
            };
            Some(PropagatedHeader {
                header_name,
                value_kind: header.value.value_kind,
                value: header.value.bytes,
            })
        })
    }

    fn resolve_static_action_for_name(
        &self,
        name: &ContextEntryName,
    ) -> (PropagationAction, NameStrategy) {
        let name = name.as_str();
        // Check overrides first.
        for ov in &self.overrides {
            if ov
                .match_rule
                .stored_names
                .iter()
                .any(|stored| name.eq_ignore_ascii_case(stored.as_str()))
            {
                let name_strategy = ov.name.unwrap_or(self.default.name);
                return (ov.action, name_strategy);
            }
        }

        let selected = self.default.selector.selects_primitive_header(name)
            || self
                .compiled_named
                .iter()
                .any(|binding| name.eq_ignore_ascii_case(binding.source_name.as_str()));

        if selected {
            (self.default.action, self.default.name)
        } else {
            (PropagationAction::Drop, self.default.name)
        }
    }

    fn resolve_action_for_header<'a>(
        &'a self,
        headers: &'a TransportHeaders,
        name: &str,
        condition_matches: &mut ConditionMatchCache<'a>,
    ) -> (PropagationAction, NameStrategy, Option<&'a str>) {
        for ov in &self.overrides {
            if ov
                .match_rule
                .stored_names
                .iter()
                .any(|stored| name.eq_ignore_ascii_case(stored.as_str()))
            {
                let name_strategy = ov.name.unwrap_or(self.default.name);
                return (ov.action, name_strategy, None);
            }
        }

        if self.default.selector.selects_primitive_header(name) {
            return (self.default.action, self.default.name, None);
        }
        for binding in &self.compiled_named {
            if !name.eq_ignore_ascii_case(binding.source_name.as_str()) {
                continue;
            }
            if binding.matches_cached(headers, condition_matches) {
                return (
                    self.default.action,
                    self.default.name,
                    Some(binding.output_name.as_str()),
                );
            }
        }
        (PropagationAction::Drop, self.default.name, None)
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

impl CompiledNamedPropagation {
    fn matches_cached<'a>(
        &'a self,
        headers: &TransportHeaders,
        condition_matches: &mut ConditionMatchCache<'a>,
    ) -> bool {
        condition_matches
            .iter()
            .find_map(|(conditions, matches)| {
                (*conditions == self.conditions.as_slice()).then_some(*matches)
            })
            .unwrap_or_else(|| {
                let matches = self.matches(headers);
                condition_matches.push((self.conditions.as_slice(), matches));
                matches
            })
    }

    fn matches(&self, headers: &TransportHeaders) -> bool {
        self.conditions.iter().all(|condition| {
            headers.iter().any(|header| {
                header
                    .name
                    .as_str()
                    .eq_ignore_ascii_case(condition.name.as_str())
                    && header.value.bytes == condition.value.as_ref()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::transport_headers::ValueKind;
    use otel_arrow_dfe_config::transport_headers_policy::{PropagationMatch, PropagationSelector};
    use otel_arrow_dfe_config::{context_policy, transport_headers};

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

    /// Scenario: an exporter uses the default compiled policy or compiles default settings.
    /// Guarantees: both policies are equal and propagate no captured headers.
    #[test]
    fn default_propagation_policy_matches_compilation() {
        let policy = CompiledHeaderPropagationPolicy::default();
        assert_eq!(
            policy,
            CompiledHeaderPropagationPolicy::compile(HeaderPropagationPolicy::default(), &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(
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
        assert_eq!(policy.propagate(&headers).count(), 0);

        headers.push(transport_headers::TransportHeader::text(
            context_name("environment"),
            b"production",
        ));
        assert_eq!(policy.propagate(&headers).count(), 0);

        headers.push(transport_headers::TransportHeader::text(
            context_name("REGION"),
            b"US-EAST",
        ));
        assert_eq!(policy.propagate(&headers).count(), 0);

        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));
        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
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
        let policy = CompiledHeaderPropagationPolicy::compile(
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

        assert_eq!(policy.propagate(&headers).count(), 0);
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
        let policy = CompiledHeaderPropagationPolicy::compile(
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

        assert_eq!(policy.propagate(&headers).count(), 0);

        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));
        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 2);
        assert!(
            propagated
                .iter()
                .all(|header| header.header_name == "workspace_id")
        );
        assert_eq!(propagated[0].value, b"acme");
        assert_eq!(propagated[1].value, b"beta");
    }

    /// Scenario: two selected composite members share the same condition set.
    /// Guarantees: propagation evaluates and caches that condition set only once per call.
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
        let policy = CompiledHeaderPropagationPolicy::compile(
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
        let mut condition_matches = ConditionMatchCache::new();

        assert_eq!(policy.compiled_named.len(), 2);
        assert!(policy.compiled_named[0].matches_cached(&headers, &mut condition_matches));
        assert_eq!(condition_matches.len(), 1);
        assert!(policy.compiled_named[1].matches_cached(&headers, &mut condition_matches));
        assert_eq!(condition_matches.len(), 1);
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
        let policy = CompiledHeaderPropagationPolicy::compile(
            policy,
            &[conditional_product_user_declaration()],
        )
        .expect("composite selector compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace");
        assert_eq!(propagated[0].value, b"acme");
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

        let error = CompiledHeaderPropagationPolicy::compile(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[declaration])
            .expect("header member compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[declaration])
            .expect("header member compiles");
        let mut headers = TransportHeaders::new();
        headers.push(transport_headers::TransportHeader::text(
            context_name("workspace"),
            b"acme",
        ));

        let propagated = policy.propagate(&headers).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace_id");
        assert_eq!(propagated[0].value, b"acme");
    }

    /// Scenario: a qualified propagation selector names a configured constant member.
    /// Guarantees: pre-integration compilation fails explicitly instead of silently dropping it.
    #[test]
    fn composite_transport_header_propagation_rejects_constant_member() {
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

        let error = CompiledHeaderPropagationPolicy::compile(policy, &[declaration])
            .expect_err("constant propagation must wait for runtime integration");
        assert!(error.contains("selects constant member `route_name`"));
        assert!(error.contains("constant runtime integration"));
    }

    /// Scenario: a qualified propagation selector names a configured randomness member.
    /// Guarantees: pre-integration compilation fails explicitly instead of silently dropping it.
    #[test]
    fn composite_transport_header_propagation_rejects_randomness_member() {
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

        let error = CompiledHeaderPropagationPolicy::compile(policy, &[declaration])
            .expect_err("randomness propagation must wait for runtime integration");
        assert!(error.contains("selects randomness member `id`"));
        assert!(error.contains("randomness runtime integration"));
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

        let _compiled = CompiledHeaderPropagationPolicy::compile(policy, &[])
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

        let error = CompiledHeaderPropagationPolicy::compile(policy, &[declaration])
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

        let error = CompiledHeaderPropagationPolicy::compile(policy, &declarations)
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
            .expect("propagation policy compiles");
        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 2);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
        assert_eq!(propagated[1].header_name, "X-Request-Id");
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }
}
