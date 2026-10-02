// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compiled transport-header propagation and composite presence gates.

use super::{ContextEntryId, ContextLayout, ContextNameId, ContextValues};
use otel_arrow_dfe_config::context_policy::{ContextDomain, ContextEntryDeclaration};
use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use otel_arrow_dfe_config::transport_headers_policy::{
    HeaderPropagationPolicy, NameStrategy, PropagatedHeader, PropagationAction, PropagationDefault,
    PropagationOverride, PropagationSelectorType,
};
use otel_arrow_dfe_config::{ContextEntryName, ContextEntryRef};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;

/// A propagation policy with every composite selector resolved before runtime.
///
/// Configuration alone cannot propagate headers:
///
/// ```compile_fail
/// use otel_arrow_dfe_config::transport_headers::TransportHeaders;
/// use otel_arrow_dfe_config::transport_headers_policy::HeaderPropagationPolicy;
///
/// let policy = HeaderPropagationPolicy::default();
/// let _ = policy.propagate(&TransportHeaders::new());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompiledHeaderPropagationPolicy {
    default: PropagationDefault,
    overrides: Vec<PropagationOverride>,
    compiled_named: Vec<CompiledNamedPropagation>,
    // Immutable compiled state is shared when bindings are cloned for runtime instances.
    layout: Arc<ContextLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompiledNamedPropagation {
    source_name: ContextEntryName,
    output_name: ContextEntryName,
    entry: ContextEntryId,
}

type ConditionMatchCache = SmallVec<[(ContextEntryId, bool); 4]>;

impl CompiledHeaderPropagationPolicy {
    /// Validates and resolves propagation against the visible composite declarations.
    pub fn compile(
        policy: HeaderPropagationPolicy,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, String> {
        policy.validate()?;
        let references = policy.default.selector.named.as_ref();
        let layout = Arc::new(
            ContextLayout::for_references(declarations, references.into_iter().flatten())
                .map_err(|error| error.to_string())?,
        );
        let mut compiled_named = Vec::new();
        let mut selected_sources = HashMap::<Box<str>, ContextEntryRef>::new();

        for reference in references.into_iter().flatten() {
            let Some(_) = reference.scope() else {
                register_named_source(&mut selected_sources, reference.name(), reference)?;
                continue;
            };
            let projection = layout
                .resolve(reference)
                .map_err(|error| error.to_string())?;
            let ContextNameId::Composite(entry) = projection.presence() else {
                unreachable!("qualified references resolve to composites");
            };
            let field = &layout.fields()[projection.fields()[0].index()];
            let source_name = field.name.clone();
            if field.domain != ContextDomain::TransportHeader {
                return Err(format!(
                    "context entry reference `{reference}` selects authorized-identity member `{source_name}`, which cannot be propagated as a transport header"
                ));
            }
            register_named_source(&mut selected_sources, &source_name, reference)?;
            compiled_named.push(CompiledNamedPropagation {
                source_name,
                output_name: reference.name().clone(),
                entry,
            });
        }
        Ok(Self {
            default: policy.default,
            overrides: policy.overrides,
            compiled_named,
            layout,
        })
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
    ///
    /// All selectors and their composite presence gates are resolved before use.
    pub fn propagate<'a>(
        &'a self,
        context: &'a impl ContextValues,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        let mut condition_matches = ConditionMatchCache::new();
        context
            .transport_headers()
            .into_iter()
            .flat_map(TransportHeaders::iter)
            .filter_map(move |header| {
                let (action, name_strategy, selected_name) = self.resolve_action_for_header(
                    context,
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
        context: &'a impl ContextValues,
        name: &str,
        condition_matches: &mut ConditionMatchCache,
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
            let present = match condition_matches
                .iter()
                .find(|(entry, _)| *entry == binding.entry)
            {
                Some((_, present)) => *present,
                None => {
                    let present = self.layout.is_present(binding.entry, context);
                    condition_matches.push((binding.entry, present));
                    present
                }
            };
            if present {
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
    /// values use any-match semantics, and unselected identity members must also be present.
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
        for (name, value, expected) in [
            ("region", "us-east", 0),
            ("environment", "Production", 0),
            ("environment", "production", 1),
        ] {
            headers.push(transport_headers::TransportHeader::text(
                context_name(name),
                value.as_bytes(),
            ));
            let context = IdentityContext {
                headers: &headers,
                identity: &["customer_id"],
            };
            assert_eq!(policy.propagate(&context).count(), expected);
        }
        assert_eq!(policy.propagate(&headers).count(), 0);
        let context = IdentityContext {
            headers: &headers,
            identity: &["customer_id"],
        };
        let propagated = policy.propagate(&context).collect::<Vec<_>>();
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

        let context = IdentityContext {
            headers: &headers,
            identity: &["customer_id"],
        };
        assert_eq!(policy.propagate(&context).count(), 0);
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

        let context = IdentityContext {
            headers: &headers,
            identity: &["customer_id"],
        };
        assert_eq!(policy.propagate(&context).count(), 0);

        headers.push(transport_headers::TransportHeader::text(
            context_name("region"),
            b"us-east",
        ));
        let context = IdentityContext {
            headers: &headers,
            identity: &["customer_id"],
        };
        let propagated = policy.propagate(&context).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 2);
        assert!(
            propagated
                .iter()
                .all(|header| header.header_name == "workspace_id")
        );
        assert_eq!(propagated[0].value, b"acme");
        assert_eq!(propagated[1].value, b"beta");
    }

    /// Scenario: two selected members and duplicate values share one composite presence gate.
    /// Guarantees: successful and failed gates are evaluated once per call, then reevaluated
    /// on the next call while preserving output names and order.
    #[test]
    fn composite_transport_header_propagation_shares_conditions_across_bindings() {
        use std::cell::Cell;

        struct CountingContext {
            headers: TransportHeaders,
            identity_present: Cell<bool>,
            identity_checks: Cell<usize>,
        }

        impl ContextValues for CountingContext {
            fn transport_headers(&self) -> Option<&TransportHeaders> {
                Some(&self.headers)
            }

            fn has_authorized_identity(&self, name: &ContextEntryName) -> bool {
                assert_eq!(name.as_str(), "customer");
                self.identity_checks.set(self.identity_checks.get() + 1);
                self.identity_present.get()
            }
        }

        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            r#"
default:
  selector:
    type: named
    named: [product_user:workspace_id, product_user:account_id]
  name: stored_name
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
    - type: authorized_identity
      name: customer
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
        for name in ["workspace", "account", "workspace"] {
            headers.push(transport_headers::TransportHeader::text(
                context_name(name),
                b"present",
            ));
        }
        let context = CountingContext {
            headers,
            identity_present: Cell::new(true),
            identity_checks: Cell::new(0),
        };
        for (call, present) in [true, false, true].into_iter().enumerate() {
            context.identity_present.set(present);
            let output = policy
                .propagate(&context)
                .map(|header| (header.header_name, header.value))
                .collect::<Vec<_>>();
            let expected = if present {
                vec![
                    ("workspace_id", b"present".as_slice()),
                    ("account_id", b"present".as_slice()),
                    ("workspace_id", b"present".as_slice()),
                ]
            } else {
                vec![]
            };
            assert_eq!(output, expected);
            assert_eq!(context.identity_checks.get(), call + 1);
        }
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

    /// Scenario: programmatic configuration has a named selector without its required list.
    /// Guarantees: compilation rejects malformed configuration before creating a runtime binding.
    #[test]
    fn compilation_rejects_invalid_selector_shape() {
        let policy = HeaderPropagationPolicy::new(
            PropagationDefault {
                selector: PropagationSelector {
                    selector_type: PropagationSelectorType::Named,
                    named: None,
                },
                ..PropagationDefault::default()
            },
            vec![],
        );
        let error = CompiledHeaderPropagationPolicy::compile(policy, &[])
            .expect_err("invalid selector must not create a binding");
        assert!(error.contains("'named' list is required"));
    }

    /// Scenario: two composites have identical conditions but require different identities.
    /// Guarantees: cached presence never allows one composite's identity to satisfy another.
    #[test]
    fn composite_presence_cache_distinguishes_members() {
        let context: context_policy::ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  first:
    - {type: transport_header, name: workspace}
    - {type: authorized_identity, name: customer}
    - {type: transport_header_match, name: environment, value: prod}
  second:
    - {type: transport_header, name: account}
    - {type: authorized_identity, name: missing_identity}
    - {type: transport_header_match, name: environment, value: prod}
"#,
        )
        .expect("context policy");
        let declarations = context
            .entries
            .into_iter()
            .map(|(name, definition)| ContextEntryDeclaration {
                scope: context_policy::ContextScope::Engine,
                name,
                definition,
            })
            .collect::<Vec<_>>();
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            "default:\n  selector: {type: named, named: ['first:workspace', 'second:account']}",
        )
        .expect("propagation policy");
        let policy =
            CompiledHeaderPropagationPolicy::compile(policy, &declarations).expect("compiled");
        let mut headers = TransportHeaders::new();
        for (name, value) in [
            ("workspace", "acme"),
            ("account", "beta"),
            ("environment", "prod"),
        ] {
            headers.push(transport_headers::TransportHeader::text(
                context_name(name),
                value.as_bytes(),
            ));
        }
        let context = IdentityContext {
            headers: &headers,
            identity: &["customer"],
        };
        let propagated = policy.propagate(&context).collect::<Vec<_>>();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "workspace");
    }

    struct IdentityContext<'a> {
        headers: &'a TransportHeaders,
        identity: &'a [&'a str],
    }

    impl ContextValues for IdentityContext<'_> {
        fn transport_headers(&self) -> Option<&TransportHeaders> {
            Some(self.headers)
        }

        fn has_authorized_identity(&self, name: &ContextEntryName) -> bool {
            self.identity.contains(&name.as_str())
        }
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
