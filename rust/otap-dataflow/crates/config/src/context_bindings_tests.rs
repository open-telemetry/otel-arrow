// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::context_policy::ContextPolicy;
use crate::transport_headers_policy::{CompiledHeaderCapturePolicy, HeaderCapturePolicy};

fn pipeline() -> crate::PipelineKey {
    crate::PipelineKey::new("g".into(), "p".into())
}

fn reference(value: &str) -> ContextEntryRef {
    value.try_into().expect("valid reference")
}

fn capture_policy() -> HeaderCapturePolicy {
    serde_yaml::from_str(
        "headers:
  - match_names: [x-customer, x-workspace]
    store_as: request_identity
  - match_names: [environment]
",
    )
    .expect("capture policy")
}

fn definition(scope: ContextScope) -> BTreeSet<ContextEntryDeclaration> {
    let policy: ContextPolicy = serde_yaml::from_str(
        "entries:
  production_workspace:
    - type: transport_header
      name: request_identity:x-customer
      as: customer
    - type: transport_header
      name: request_identity:x-workspace
      as: workspace
    - type: transport_header_match
      name: environment
      value: production
      match: all
",
    )
    .expect("entry policy");
    policy
        .entries
        .into_iter()
        .map(|(name, definition)| ContextEntryDeclaration {
            scope: scope.clone(),
            name,
            definition,
        })
        .collect()
}

fn layout() -> Arc<ContextLayout> {
    ContextLayout::compile(
        capture_policy().context_primitives().collect(),
        definition(ContextScope::Engine),
    )
    .expect("compiled layout")
}

fn sink(
    layout: Arc<ContextLayout>,
    references: &[&str],
    naming: NameStrategy,
) -> CompiledHeaderPropagationPolicy {
    use crate::transport_headers_policy::{PropagationDefault, PropagationSelector};
    HeaderPropagationPolicy::new(
        PropagationDefault {
            selector: PropagationSelector {
                selector_type: PropagationSelectorType::Named,
                named: Some(references.iter().map(|value| reference(value)).collect()),
            },
            name: naming,
            ..Default::default()
        },
        vec![],
    )
    .compile(layout, &pipeline())
    .expect("compiled sink")
}

fn capture(
    layout: Arc<ContextLayout>,
    sink: &CompiledHeaderPropagationPolicy,
) -> CompiledHeaderCapturePolicy {
    let required: BTreeSet<_> = sink.original_name_fields().collect();
    capture_policy()
        .compile_bound(layout, &pipeline(), |field| required.contains(&field))
        .expect("compiled source")
}

fn message(capture: &CompiledHeaderCapturePolicy, input: &[(&str, &[u8])]) -> TransportHeaders {
    let mut headers = TransportHeaders::new();
    assert!(
        capture
            .capture_from_pairs(input.iter().copied(), &mut headers)
            .is_none()
    );
    headers
}

/// Scenario: a composite-qualified propagation rule receives complete, partial, or non-production inputs.
/// Guarantees: only a present composite emits its selected member; bare source presence is not a fallback.
#[test]
fn conditional_member_propagation_is_atomic() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(layout, &sink);
    for (customer, environment, expected) in [
        (true, "production", 2),
        (true, "development", 0),
        (false, "production", 0),
    ] {
        let mut input = vec![
            ("x-workspace", b"first".as_slice()),
            ("x-workspace", b"second".as_slice()),
            ("environment", environment.as_bytes()),
        ];
        if customer {
            input.push(("x-customer", b"acme"));
        }
        let context = message(&source, &input);
        let output: Vec<_> = sink.propagate(&context).expect("bound context").collect();
        assert_eq!(output.len(), expected);
        if expected != 0 {
            assert_eq!(output[0].header_name, "x-workspace");
            assert_eq!(output[0].value, b"first");
            assert_eq!(output[1].value, b"second");
        }
    }
}

/// Scenario: consumers bind qualified fields, whole capture groups, and undeclared bare names.
/// Guarantees: field access never searches composites implicitly or coerces a whole group into a field.
#[test]
fn references_are_schema_checked() {
    let layout = layout();
    for invalid in [
        "x-customer",
        "request_identity:unknown",
        "environment:environment",
        "unknown:field",
    ] {
        assert!(
            layout.bind_field(&reference(invalid), &pipeline()).is_err(),
            "{invalid}"
        );
    }
    assert!(
        layout
            .bind_field(&reference("request_identity"), &pipeline())
            .is_err()
    );
    assert!(
        layout
            .resolve(&reference("request_identity"), &pipeline())
            .is_ok()
    );
    assert!(
        layout
            .bind_field(&reference("environment"), &pipeline())
            .is_ok()
    );
    assert!(
        layout
            .bind_field(&reference("request_identity:x-customer"), &pipeline())
            .is_ok()
    );
}

/// Scenario: scalar and multi-value bindings access the same repeated field through a present composite.
/// Guarantees: propagation preserves every occurrence while scalar access reports multiplicity explicitly.
#[test]
fn cardinality_belongs_to_the_binding() {
    let layout = layout();
    let binding = layout
        .bind_field(&reference("production_workspace:workspace"), &pipeline())
        .unwrap();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::StoredName,
    );
    let source = capture(layout, &sink);
    let context = message(
        &source,
        &[
            ("x-customer", b"acme"),
            ("environment", b"production"),
            ("x-workspace", b""),
            ("x-workspace", b"two"),
        ],
    );
    assert_eq!(binding.values(&context).unwrap().count(), 2);
    assert!(matches!(
        binding.single(&context),
        Err(ContextAccessError::MultipleValues)
    ));
    let output: Vec<_> = sink.propagate(&context).unwrap().collect();
    assert_eq!(output.len(), 2);
    assert!(
        output
            .iter()
            .all(|header| header.header_name == "production_workspace")
    );
    assert_eq!(output[0].value, b"");
}

/// Scenario: an unconditional capture group receives one member and overlapping propagation selections.
/// Guarantees: partial capture groups remain usable and selector overlap does not duplicate occurrences.
#[test]
fn capture_groups_are_unconditional_and_selection_is_a_union() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["request_identity", "request_identity:x-workspace"],
        NameStrategy::StoredName,
    );
    let source = capture(layout, &sink);
    let context = message(&source, &[("x-workspace", b"one"), ("x-workspace", b"two")]);
    let output: Vec<_> = sink.propagate(&context).unwrap().collect();
    assert_eq!(output.len(), 2);
    assert!(
        output
            .iter()
            .all(|header| header.header_name == "request_identity")
    );
}

/// Scenario: a sink preserves just one qualified capture member's original name.
/// Guarantees: unrelated members retain their identity without allocating original-name strings.
#[test]
fn original_name_retention_follows_selected_source_fields() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(layout.clone(), &sink);
    let context = message(
        &source,
        &[
            ("X-Customer", b"acme"),
            ("X-Workspace", b"one"),
            ("environment", b"production"),
        ],
    );
    assert!(context.as_slice()[0].value.original_name.is_none());
    assert_eq!(
        context.as_slice()[1].value.original_name.as_deref(),
        Some("X-Workspace")
    );
    let customer = layout
        .bind_field(&reference("request_identity:x-customer"), &pipeline())
        .unwrap();
    assert_eq!(
        customer
            .single(&context)
            .unwrap()
            .unwrap()
            .value
            .bytes
            .as_ref(),
        b"acme"
    );
    assert_eq!(
        sink.propagate(&context)
            .unwrap()
            .next()
            .unwrap()
            .header_name,
        "X-Workspace"
    );
}

/// Scenario: a repeated environment field contains both production and development values.
/// Guarantees: explicit all-values guards reject the group without treating one matching value as sufficient.
#[test]
fn all_values_condition_is_not_implicit_any() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(layout, &sink);
    let context = message(
        &source,
        &[
            ("x-customer", b"acme"),
            ("x-workspace", b"one"),
            ("environment", b"production"),
            ("environment", b"development"),
        ],
    );
    assert_eq!(sink.propagate(&context).unwrap().count(), 0);
}

/// Scenario: a bound producer adds a contradictory guard value after a context was cloned.
/// Guarantees: mutation recomputes composite presence without changing the original context's shared snapshot.
#[test]
fn producer_mutation_recomputes_presence_copy_on_write() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(layout.clone(), &sink);
    let original = message(
        &source,
        &[
            ("x-customer", b"acme"),
            ("x-workspace", b"one"),
            ("environment", b"production"),
        ],
    );
    let mut modified = original.clone();
    let producer = layout
        .bind_producer(&"environment".try_into().unwrap(), &pipeline())
        .unwrap();
    producer
        .append(
            &mut modified,
            TransportHeader::text("environment".try_into().unwrap(), b"development").value,
        )
        .unwrap();
    assert_eq!(sink.propagate(&original).unwrap().count(), 1);
    assert_eq!(sink.propagate(&modified).unwrap().count(), 0);
}

/// Scenario: a new configuration assigns different slots while an old context remains in flight.
/// Guarantees: incompatible layouts report an error instead of reading a different field or treating it as missing.
#[test]
fn layout_changes_fail_closed() {
    let old = layout();
    let old_sink = sink(
        old.clone(),
        &["request_identity:x-workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(old, &old_sink);
    let context = message(&source, &[("x-workspace", b"one")]);
    let mut primitives: BTreeSet<_> = capture_policy().context_primitives().collect();
    let _ = primitives.insert(ContextPrimitive::standalone("aaa".try_into().unwrap()));
    let new = ContextLayout::compile(primitives, definition(ContextScope::Engine)).unwrap();
    let new_sink = sink(
        new,
        &["request_identity:x-workspace"],
        NameStrategy::Preserve,
    );
    assert!(matches!(
        new_sink.propagate(&context),
        Err(ContextAccessError::IncompatibleLayout)
    ));
}

/// Scenario: two overlapping policy scopes define the same entry name.
/// Guarantees: inherited entries cannot silently shadow broader definitions.
#[test]
fn overlapping_scope_definitions_are_rejected() {
    let definitions = definition(ContextScope::Engine)
        .into_iter()
        .chain(definition(ContextScope::Group("g".into())))
        .collect();
    assert!(
        ContextLayout::compile(capture_policy().context_primitives().collect(), definitions)
            .is_err()
    );
    let scoped = ContextLayout::compile(
        capture_policy().context_primitives().collect(),
        definition(ContextScope::Group("other".into())),
    )
    .unwrap();
    assert!(
        scoped
            .resolve(&reference("production_workspace:workspace"), &pipeline())
            .is_err()
    );
}

/// Scenario: a new sink requests original names that an earlier capture deliberately omitted.
/// Guarantees: present selections report missing retained information, while absent composites still emit nothing.
#[test]
fn original_name_requirements_cannot_reinterpret_an_old_capture() {
    let layout = layout();
    let source = capture_policy()
        .compile_bound(layout.clone(), &pipeline(), |_| false)
        .unwrap();
    let sink = sink(
        layout,
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    for environment in ["production", "development"] {
        let context = message(
            &source,
            &[
                ("x-customer", b"acme"),
                ("X-Workspace", b"one"),
                ("environment", environment.as_bytes()),
            ],
        );
        if environment == "production" {
            assert!(matches!(
                sink.propagate(&context),
                Err(ContextAccessError::OriginalNameUnavailable)
            ));
        } else {
            assert_eq!(sink.propagate(&context).unwrap().count(), 0);
        }
    }
}

/// Scenario: a message arrives outside the group that defines a conditional entry and crosses into that group.
/// Guarantees: a source cannot construct another scope's derived entries, even when primitive values match.
#[test]
fn arrival_construction_obeys_definition_scope() {
    let layout = ContextLayout::compile(
        capture_policy().context_primitives().collect(),
        definition(ContextScope::Group("g".into())),
    )
    .unwrap();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture_policy()
        .compile_bound(
            layout,
            &crate::PipelineKey::new("other".into(), "p".into()),
            |_| true,
        )
        .unwrap();
    let context = message(
        &source,
        &[
            ("x-customer", b"acme"),
            ("x-workspace", b"one"),
            ("environment", b"production"),
        ],
    );
    assert_eq!(sink.propagate(&context).unwrap().count(), 0);
}

/// Scenario: a whole-entry drop override precedes a qualified member propagation override.
/// Guarantees: declaration order wins over specificity, and unreachable preserve actions retain no original names.
#[test]
fn overrides_preserve_first_match_precedence() {
    use crate::transport_headers_policy::{PropagationMatch, PropagationOverride};
    let layout = layout();
    let policy = HeaderPropagationPolicy::new(
        Default::default(),
        vec![
            PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![reference("request_identity")],
                },
                action: PropagationAction::Drop,
                name: None,
                on_error: None,
            },
            PropagationOverride {
                match_rule: PropagationMatch {
                    stored_names: vec![reference("request_identity:x-workspace")],
                },
                action: PropagationAction::Propagate,
                name: Some(NameStrategy::Preserve),
                on_error: None,
            },
        ],
    )
    .compile(layout.clone(), &pipeline())
    .unwrap();
    assert_eq!(policy.original_name_fields().count(), 0);
    let source = capture(layout, &policy);
    let context = message(&source, &[("x-workspace", b"one")]);
    assert_eq!(policy.propagate(&context).unwrap().count(), 0);
}

/// Scenario: a scalar binding reads an empty header through present and absent composites.
/// Guarantees: an empty single value is present, but the same primitive value is hidden by a failed parent condition.
#[test]
fn empty_scalar_value_is_distinct_from_missing_composite() {
    let layout = layout();
    let binding = layout
        .bind_field(&reference("production_workspace:workspace"), &pipeline())
        .unwrap();
    let empty = TransportHeaders::new();
    assert!(binding.single(&empty).unwrap().is_none());
    assert_eq!(binding.values(&empty).unwrap().count(), 0);
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::StoredName,
    );
    let source = capture(layout, &sink);
    for environment in ["production", "development"] {
        let context = message(
            &source,
            &[
                ("x-customer", b"acme"),
                ("x-workspace", b""),
                ("environment", environment.as_bytes()),
            ],
        );
        let value = binding.single(&context).unwrap();
        assert_eq!(value.is_some(), environment == "production");
        if let Some(value) = value {
            assert!(value.value.bytes.is_empty());
        }
    }
}

/// Scenario: an existing raw-header API mutates a previously bound context.
/// Guarantees: compiled consumers reject invalidated indexes instead of observing stale composite presence.
#[test]
fn raw_mutation_invalidates_compiled_access() {
    let layout = layout();
    let sink = sink(
        layout.clone(),
        &["production_workspace:workspace"],
        NameStrategy::Preserve,
    );
    let source = capture(layout, &sink);
    let mut context = message(
        &source,
        &[
            ("x-customer", b"acme"),
            ("x-workspace", b"one"),
            ("environment", b"production"),
        ],
    );
    context.push(TransportHeader::text(
        "environment".try_into().unwrap(),
        b"development",
    ));
    assert!(matches!(
        sink.propagate(&context),
        Err(ContextAccessError::Unbound)
    ));
}

/// Scenario: the engine declares thousands of unrelated fields but an arrival captures one field.
/// Guarantees: message-local lookup and presence storage scale with the captured message, not the engine namespace.
#[test]
fn message_index_does_not_allocate_global_register_arrays() {
    let mut primitives: BTreeSet<_> = capture_policy().context_primitives().collect();
    primitives.extend(
        (0..4096)
            .map(|id| ContextPrimitive::standalone(format!("unrelated-{id}").try_into().unwrap())),
    );
    let layout = ContextLayout::compile(primitives, definition(ContextScope::Engine)).unwrap();
    let sink = sink(
        layout.clone(),
        &["request_identity:x-workspace"],
        NameStrategy::Preserve,
    );
    assert_eq!(sink.fields.len(), 1);
    let source = capture(layout.clone(), &sink);
    let context = message(&source, &[("x-workspace", b"one")]);
    let index = context.context_index(&layout).unwrap();
    assert_eq!(index.heads.len(), 1);
    assert_eq!(index.slots.len(), 1);
    assert_eq!(index.next.len(), 1);
    assert_eq!(index.names_preserved.len(), 1);
    assert!(index.present.is_empty());
    assert_eq!(sink.propagate(&context).unwrap().count(), 1);
}
