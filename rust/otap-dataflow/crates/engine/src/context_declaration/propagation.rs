// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compiled transport-header propagation and composite presence gates.

use super::{
    ContextEntryId, ContextFieldLayout, ContextLayout, ContextMemberSource, ContextNameId,
    ContextValues,
};
use otel_arrow_dfe_config::context_policy::{ContextDomain, ContextEntryDeclaration};
use otel_arrow_dfe_config::transport_headers::{
    TransportHeaderRef, TransportHeaders, TransportHeadersIter,
};
use otel_arrow_dfe_config::transport_headers_policy::{
    HeaderPropagationPolicy, NameStrategy, PropagatedHeader, PropagationAction, PropagationDefault,
    PropagationOverride, PropagationSelectorType,
};
use otel_arrow_dfe_config::{ContextEntryName, ContextEntryRef};
use smallvec::SmallVec;
use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    actions: HeaderLookup,
    single_entry: Option<ContextEntryId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompiledAction {
    action: PropagationAction,
    name: NameStrategy,
    binding: Option<usize>,
}

/// A compact lookup for the small set of compiled propagation actions.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderLookup {
    /// Actions in deterministic stored-name order.
    entries: Box<[HeaderLookupEntry]>,
}

/// One case-insensitive stored-header-name action.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HeaderLookupEntry {
    /// Canonical stored name, matched using ASCII case-insensitive semantics.
    name: ContextEntryName,
    /// Action compiled for the stored name.
    action: CompiledAction,
}

impl Default for HeaderLookup {
    fn default() -> Self {
        Self {
            entries: Box::new([]),
        }
    }
}

impl HeaderLookup {
    fn new(entries: BTreeMap<ContextEntryName, CompiledAction>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(name, action)| HeaderLookupEntry { name, action })
                .collect(),
        }
    }

    #[inline]
    fn get(&self, name: &str) -> Option<&CompiledAction> {
        if let [entry] = self.entries.as_ref() {
            return entry
                .name
                .eq_ignore_ascii_case(name)
                .then_some(&entry.action);
        }
        if self.entries.is_empty() {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
            .map(|entry| &entry.action)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompiledNamedPropagation {
    source_name: ContextEntryName,
    output_name: ContextEntryName,
    entry: ContextEntryId,
    requires_presence: bool,
}

type ConditionMatchCache = SmallVec<[(ContextEntryId, bool); 4]>;

/// Explicit primitive headers mentioned by a propagation policy.
pub(super) fn primitive_fields(
    policy: &HeaderPropagationPolicy,
) -> impl Iterator<Item = ContextFieldLayout> + '_ {
    policy
        .default
        .selector
        .named
        .iter()
        .flatten()
        .filter(|reference| reference.scope().is_none())
        .map(ContextEntryRef::name)
        .chain(
            policy
                .overrides
                .iter()
                .flat_map(|policy| &policy.match_rule.stored_names),
        )
        .map(|name| ContextFieldLayout {
            name: name.clone(),
            domain: ContextDomain::TransportHeader,
        })
}

impl CompiledHeaderPropagationPolicy {
    /// Compiles a standalone policy using the same layout compiler as pipeline construction.
    ///
    /// Pipeline construction instead binds each policy to its already-compiled shared layout.
    pub fn compile_propagation_policy(
        policy: HeaderPropagationPolicy,
        declarations: &[ContextEntryDeclaration],
    ) -> Result<Self, String> {
        policy.validate()?;
        let selected = policy
            .default
            .selector
            .named
            .iter()
            .flatten()
            .filter_map(ContextEntryRef::scope)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|name| super::composite_declaration(name, declarations))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let layout = Arc::new(
            ContextLayout::compile_layout(primitive_fields(&policy), selected)
                .map_err(|error| error.to_string())?,
        );
        Self::bind_propagation_policy_to_layout(policy, layout)
    }

    /// Resolves propagation against a layout shared by every node in one pipeline.
    pub(super) fn bind_propagation_policy_to_layout(
        policy: HeaderPropagationPolicy,
        layout: Arc<ContextLayout>,
    ) -> Result<Self, String> {
        policy.validate()?;
        let references = policy.default.selector.named.as_ref();
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
            let member = projection
                .members()
                .and_then(|members| members.first())
                .expect("qualified references resolve to one composite member");
            let source_name = match member.source {
                ContextMemberSource::Field(field) => {
                    let field = &layout.fields()[field.index()];
                    if field.domain != ContextDomain::TransportHeader {
                        return Err(format!(
                            "context entry reference `{reference}` selects authorized-identity member `{}`, which cannot be propagated as a transport header",
                            field.name
                        ));
                    }
                    field.name.clone()
                }
                ContextMemberSource::Constant(_) => {
                    return Err(format!(
                        "context entry reference `{reference}` selects constant member `{}`, which cannot be propagated until constant runtime integration is available",
                        member.name
                    ));
                }
            };
            register_named_source(&mut selected_sources, &source_name, reference)?;
            let entry_layout = &layout.entries()[entry.index()];
            // The selected header proves a sole, unconditional member is present.
            compiled_named.push(CompiledNamedPropagation {
                source_name,
                output_name: reference.name().clone(),
                entry,
                requires_presence: entry_layout.members.len() != 1
                    || !entry_layout.conditions.is_empty(),
            });
        }
        let mut actions = BTreeMap::new();
        for override_policy in &policy.overrides {
            for name in &override_policy.match_rule.stored_names {
                _ = actions
                    .entry(name.to_ascii_lowercase())
                    .or_insert(CompiledAction {
                        action: override_policy.action,
                        name: override_policy.name.unwrap_or(policy.default.name),
                        binding: None,
                    });
            }
        }
        for reference in references
            .into_iter()
            .flatten()
            .filter(|reference| reference.scope().is_none())
        {
            _ = actions
                .entry(reference.name().to_ascii_lowercase())
                .or_insert(CompiledAction {
                    action: policy.default.action,
                    name: policy.default.name,
                    binding: None,
                });
        }
        for (index, binding) in compiled_named.iter().enumerate() {
            _ = actions
                .entry(binding.source_name.to_ascii_lowercase())
                .or_insert(CompiledAction {
                    action: policy.default.action,
                    name: policy.default.name,
                    binding: Some(index),
                });
        }
        // A failed gate can end iteration only when every action depends on it.
        let selected_entry = compiled_named.first().map(|binding| binding.entry);
        let single_entry = if policy.default.action == PropagationAction::Propagate
            && actions.values().all(|action| action.binding.is_some())
            && selected_entry.is_some()
            && compiled_named
                .iter()
                .all(|binding| Some(binding.entry) == selected_entry)
        {
            selected_entry
        } else {
            None
        };
        Ok(Self {
            default: policy.default,
            overrides: policy.overrides,
            compiled_named,
            layout,
            actions: HeaderLookup::new(actions),
            single_entry,
        })
    }

    /// Returns the layout used to resolve this policy's field and entry IDs.
    #[cfg(test)]
    pub(super) fn layout(&self) -> &Arc<ContextLayout> {
        &self.layout
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
    #[inline]
    pub fn propagate<'a>(
        &'a self,
        context: &'a impl ContextValues,
    ) -> impl Iterator<Item = PropagatedHeader<'a>> {
        let headers = context
            .transport_headers()
            .map(TransportHeaders::iter)
            .unwrap_or_default();
        if self.compiled_named.is_empty() {
            PropagationIter::Direct {
                headers,
                policy: self,
            }
        } else if let Some(entry) = self.single_entry {
            PropagationIter::Single {
                headers,
                policy: self,
                context,
                entry,
                present: !self.compiled_named[0].requires_presence,
            }
        } else {
            PropagationIter::Composite {
                headers,
                policy: self,
                context,
                condition_matches: ConditionMatchCache::new(),
            }
        }
    }

    #[inline]
    fn single_next<'a>(
        &'a self,
        context: &'a impl ContextValues,
        headers: &mut TransportHeadersIter<'a>,
        entry: ContextEntryId,
        present: &mut bool,
    ) -> Option<PropagatedHeader<'a>> {
        for header in headers.by_ref() {
            let Some(action) = self.actions.get(header.name.as_str()) else {
                continue;
            };
            if !*present {
                if !self.layout.is_present(entry, context) {
                    *headers = TransportHeadersIter::default();
                    return None;
                }
                *present = true;
            }
            let binding = &self.compiled_named[action.binding.expect("single composite binding")];
            return Self::output_header(
                header,
                action.action,
                action.name,
                Some(binding.output_name.as_str()),
            );
        }
        None
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

    #[inline]
    fn resolve_action_for_header<'a>(
        &'a self,
        context: &'a impl ContextValues,
        header: TransportHeaderRef<'a>,
        condition_matches: &mut ConditionMatchCache,
    ) -> (PropagationAction, NameStrategy, Option<&'a str>) {
        let Some(action) = self.actions.get(header.name.as_str()) else {
            return (
                if self.default.selector.selector_type == PropagationSelectorType::AllCaptured {
                    self.default.action
                } else {
                    PropagationAction::Drop
                },
                self.default.name,
                None,
            );
        };
        if let Some(index) = action.binding {
            let binding = &self.compiled_named[index];
            let present = !binding.requires_presence
                || match condition_matches
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
            if !present {
                return (PropagationAction::Drop, action.name, None);
            }
            return (
                action.action,
                action.name,
                Some(binding.output_name.as_str()),
            );
        }
        (action.action, action.name, None)
    }

    #[inline]
    fn direct_header<'a>(&'a self, header: TransportHeaderRef<'a>) -> Option<PropagatedHeader<'a>> {
        let (action, name) = match self.actions.get(header.name.as_str()) {
            Some(action) => (action.action, action.name),
            None => (
                if self.default.selector.selector_type == PropagationSelectorType::AllCaptured {
                    self.default.action
                } else {
                    PropagationAction::Drop
                },
                self.default.name,
            ),
        };
        Self::output_header(header, action, name, None)
    }

    #[inline]
    fn composite_header<'a>(
        &'a self,
        context: &'a impl ContextValues,
        header: TransportHeaderRef<'a>,
        matches: &mut ConditionMatchCache,
    ) -> Option<PropagatedHeader<'a>> {
        let (action, name, selected) = self.resolve_action_for_header(context, header, matches);
        Self::output_header(header, action, name, selected)
    }

    #[inline]
    fn output_header<'a>(
        header: TransportHeaderRef<'a>,
        action: PropagationAction,
        name: NameStrategy,
        selected: Option<&'a str>,
    ) -> Option<PropagatedHeader<'a>> {
        (action == PropagationAction::Propagate).then(|| PropagatedHeader {
            header_name: match name {
                NameStrategy::Preserve => header.wire_name(),
                NameStrategy::StoredName => selected.unwrap_or(header.name.as_str()),
            },
            value_kind: header.value.value_kind,
            value: header.value.bytes,
        })
    }
}

enum PropagationIter<'a, C> {
    Direct {
        headers: TransportHeadersIter<'a>,
        policy: &'a CompiledHeaderPropagationPolicy,
    },
    Single {
        headers: TransportHeadersIter<'a>,
        policy: &'a CompiledHeaderPropagationPolicy,
        context: &'a C,
        entry: ContextEntryId,
        present: bool,
    },
    Composite {
        headers: TransportHeadersIter<'a>,
        policy: &'a CompiledHeaderPropagationPolicy,
        context: &'a C,
        condition_matches: ConditionMatchCache,
    },
}

impl<'a, C: ContextValues> Iterator for PropagationIter<'a, C> {
    type Item = PropagatedHeader<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct { headers, policy } => {
                headers.find_map(|header| policy.direct_header(header))
            }
            Self::Single {
                headers,
                policy,
                context,
                entry,
                present,
            } => policy.single_next(*context, headers, *entry, present),
            Self::Composite {
                headers,
                policy,
                context,
                condition_matches,
            } => headers
                .find_map(|header| policy.composite_header(*context, header, condition_matches)),
        }
    }

    #[inline]
    fn fold<B, F>(self, initial: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Direct { headers, policy } => headers
                .filter_map(|header| policy.direct_header(header))
                .fold(initial, fold),
            Self::Single {
                mut headers,
                policy,
                context,
                entry,
                mut present,
            } => std::iter::from_fn(|| {
                policy.single_next(context, &mut headers, entry, &mut present)
            })
            .fold(initial, fold),
            Self::Composite {
                headers,
                policy,
                context,
                mut condition_matches,
            } => headers
                .filter_map(|header| {
                    policy.composite_header(context, header, &mut condition_matches)
                })
                .fold(initial, fold),
        }
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
            CompiledHeaderPropagationPolicy::compile_propagation_policy(
                HeaderPropagationPolicy::default(),
                &[],
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(
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
    /// on the next call, preserving names and order across mixed `next` and `fold` consumption.
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(
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
        for (call, (present, consumed)) in [true, false, true]
            .into_iter()
            .flat_map(|present| (0..=3).map(move |consumed| (present, consumed)))
            .enumerate()
        {
            context.identity_present.set(present);
            let mut iter = policy.propagate(&context);
            let mut output = Vec::new();
            for _ in 0..consumed {
                if let Some(header) = iter.next() {
                    output.push((header.header_name, header.value));
                }
            }
            let output = iter.fold(output, |mut output, header| {
                output.push((header.header_name, header.value));
                output
            });
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(
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
        let error = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
            CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &declarations)
                .expect("compiled");
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

        let error = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
            .expect_err("unknown composite must fail");
        assert!(error.contains("unknown composite context entry `missing`"));
    }

    /// Scenario: standalone propagation receives two visible definitions of a selected name.
    /// Guarantees: duplicate configuration declarations are rejected before layout compilation,
    /// even though repeated references to one declaration may share a compiled entry.
    #[test]
    fn standalone_propagation_rejects_duplicate_composite_definitions() {
        let declaration = conditional_product_user_declaration();
        let policy: HeaderPropagationPolicy = serde_yaml::from_str(
            "default:\n  selector: {type: named, named: ['product_user:workspace_id']}",
        )
        .expect("propagation policy");
        let error = CompiledHeaderPropagationPolicy::compile_propagation_policy(
            policy,
            &[declaration.clone(), declaration],
        )
        .expect_err("duplicate declarations must fail");
        assert!(error.contains("duplicate composite context entry `product_user`"));
    }

    /// Scenario: a qualified selector names a header in a composite that also has a constant.
    /// Guarantees: the unrelated constant does not block compilation or header propagation.
    #[test]
    fn composite_transport_header_propagation_ignores_unselected_constant() {
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
        let policy =
            CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[declaration])
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

        let error =
            CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[declaration])
                .expect_err("constant propagation must wait for runtime integration");
        assert!(error.contains("selects constant member `route_name`"));
        assert!(error.contains("constant runtime integration"));
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

        let _compiled = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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

        let error =
            CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[declaration])
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

        let error =
            CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &declarations)
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
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
        let policy = CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
            .expect("propagation policy compiles");

        let mut headers = TransportHeaders::new();
        headers.push(header("tenant_id", "X-Tenant-Id", b"t-1"));
        headers.push(header("request_id", "X-Request-Id", b"r-1"));

        let propagated: Vec<_> = policy.propagate(&headers).collect();
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].header_name, "X-Tenant-Id");
    }

    /// Scenario: indexed selectors and overlapping overrides use names of different lengths/case.
    /// Guarantees: propagation agrees with static first-override-wins resolution for every
    /// selector type, including small sets and first/last-byte lookup-key collisions.
    #[test]
    fn indexed_actions_preserve_selector_and_override_precedence() {
        for count in [1, 2, 3, 4, 5, 32] {
            let names = (0..count)
                .map(|index| context_name(&format!("field_{index}")))
                .collect::<Vec<_>>();
            for selector_type in [
                PropagationSelectorType::None,
                PropagationSelectorType::AllCaptured,
                PropagationSelectorType::Named,
            ] {
                let named = (selector_type == PropagationSelectorType::Named)
                    .then(|| names.iter().cloned().map(Into::into).collect());
                let policy = HeaderPropagationPolicy::new(
                    PropagationDefault {
                        selector: PropagationSelector {
                            selector_type,
                            named,
                        },
                        ..PropagationDefault::default()
                    },
                    vec![
                        PropagationOverride {
                            match_rule: PropagationMatch {
                                stored_names: names.clone(),
                            },
                            action: PropagationAction::Propagate,
                            name: Some(NameStrategy::StoredName),
                            on_error: None,
                        },
                        PropagationOverride {
                            match_rule: PropagationMatch {
                                stored_names: vec![context_name("FIELD_0")],
                            },
                            action: PropagationAction::Drop,
                            name: None,
                            on_error: None,
                        },
                    ],
                );
                let policy =
                    CompiledHeaderPropagationPolicy::compile_propagation_policy(policy, &[])
                        .expect("compiled");
                for name in names.iter().map(|name| name.as_str()).chain(["unknown"]) {
                    let upper = name.to_ascii_uppercase();
                    let mut headers = TransportHeaders::new();
                    headers.push(header(&upper, "original", b"value"));
                    let (action, strategy) =
                        policy.resolve_static_action_for_name(&context_name(&upper));
                    let output = policy.propagate(&headers).collect::<Vec<_>>();
                    assert_eq!(policy.propagate(&headers).count(), output.len());
                    assert_eq!(
                        output.len(),
                        usize::from(action == PropagationAction::Propagate)
                    );
                    if let Some(output) = output.first() {
                        assert_eq!(
                            output.header_name,
                            if strategy == NameStrategy::Preserve {
                                "original"
                            } else {
                                &upper
                            }
                        );
                    }
                }
            }
        }
    }
}
