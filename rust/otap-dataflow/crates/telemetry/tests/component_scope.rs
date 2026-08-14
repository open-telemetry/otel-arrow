// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cross-crate coverage for component-aware event targets.

use std::sync::{Arc, Mutex};

use tracing::{Event, Subscriber};
use tracing_subscriber::{EnvFilter, Layer, layer::Context, prelude::*};

#[derive(Clone, Default)]
struct TargetCapture {
    targets: Arc<Mutex<Vec<&'static str>>>,
}

impl<S> Layer<S> for TargetCapture
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        self.targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event.metadata().target());
    }
}

mod scoped_component {
    const COMPONENT_URN: &str = "urn:otel:processor:scope_test";

    otap_df_telemetry::otel_component_scope!(
        urn = COMPONENT_URN,
        target = "otel.processor.scope_test",
    );

    pub(super) fn emit_from_root() {
        otel_info!("test.component.root");
    }

    pub(super) fn emit_from_child() {
        child::emit();
    }

    pub(super) fn emit_all_helpers() {
        otel_debug!("test.component.debug");
        otel_info!("test.component.info");
        otel_warn!("test.component.warn");
        otel_error!("test.component.error");
        otel_event!(tracing::Level::TRACE, "test.component.trace");
    }

    pub(super) mod child {
        pub(super) fn emit() {
            otel_warn!("test.component.child", value = 1);
        }
    }
}

mod second_scoped_component {
    const COMPONENT_URN: &str = "urn:otel:processor:second_scope_test";

    otap_df_telemetry::otel_component_scope!(
        urn = COMPONENT_URN,
        target = "otel.processor.second_scope_test",
    );

    pub(super) fn emit() {
        otel_debug!("test.component.second");
    }
}

mod prefix_collision_component {
    const COMPONENT_URN: &str = "urn:otel:processor:scope_test_extra";

    otap_df_telemetry::otel_component_scope!(
        urn = COMPONENT_URN,
        target = "otel.processor.scope_test_extra",
    );

    pub(super) fn emit() {
        otel_info!("test.component.prefix_collision");
    }
}

mod namespaced_component {
    const COMPONENT_URN: &str = "urn:microsoft:processor:scope_test";

    otap_df_telemetry::otel_component_scope!(
        urn = COMPONENT_URN,
        target = "microsoft.processor.scope_test",
    );

    pub(super) fn emit() {
        otel_info!("test.component.namespaced");
    }
}

/// Scenario: a component scope emits events from its root and a child module.
/// Guarantees: every event inherits the stable target derived from the component URN.
#[test]
fn component_scope_applies_to_module_subtree() {
    let capture = TargetCapture::default();
    let targets = Arc::clone(&capture.targets);
    let subscriber = tracing_subscriber::registry().with(capture);

    tracing::subscriber::with_default(subscriber, || {
        scoped_component::emit_from_root();
        scoped_component::emit_from_child();
    });

    assert_eq!(
        *targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        ["otel.processor.scope_test", "otel.processor.scope_test",]
    );
}

/// Scenario: every event helper is used inside a component telemetry scope.
/// Guarantees: all generated helpers compile and apply the same component target.
#[test]
fn component_scope_covers_every_event_helper() {
    let capture = TargetCapture::default();
    let targets = Arc::clone(&capture.targets);
    let subscriber = tracing_subscriber::registry().with(capture);

    tracing::subscriber::with_default(subscriber, scoped_component::emit_all_helpers);

    let targets = targets
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(targets.len(), 5);
    assert!(
        targets
            .iter()
            .all(|target| *target == "otel.processor.scope_test")
    );
}

/// Scenario: callers use the base macro with and without an explicit target.
/// Guarantees: explicit targets are honored while the legacy form retains its package target.
#[test]
fn base_macros_support_explicit_and_default_targets() {
    let capture = TargetCapture::default();
    let targets = Arc::clone(&capture.targets);
    let subscriber = tracing_subscriber::registry().with(capture);

    tracing::subscriber::with_default(subscriber, || {
        otap_df_telemetry::otel_info!(target: "test::explicit", "test.explicit");
        otap_df_telemetry::otel_info!("test.default");
    });

    assert_eq!(
        *targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        ["test::explicit", "otap-df-telemetry"]
    );
}

/// Scenario: target directives select a component kind or component target prefix.
/// Guarantees: EnvFilter prefix semantics include components with prefix-colliding names.
#[test]
fn component_targets_support_hierarchical_prefix_filtering() {
    let kind_capture = TargetCapture::default();
    let kind_targets = Arc::clone(&kind_capture.targets);
    let kind_filter =
        EnvFilter::try_new("off,otel.processor=debug").expect("kind filter should parse");
    let kind_subscriber = tracing_subscriber::registry()
        .with(kind_capture)
        .with(kind_filter);

    tracing::subscriber::with_default(kind_subscriber, || {
        scoped_component::emit_from_root();
        second_scoped_component::emit();
        otap_df_telemetry::otel_info!("test.unscoped");
    });

    assert_eq!(
        *kind_targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        [
            "otel.processor.scope_test",
            "otel.processor.second_scope_test",
        ]
    );

    let component_capture = TargetCapture::default();
    let component_targets = Arc::clone(&component_capture.targets);
    let component_filter = EnvFilter::try_new("off,otel.processor.scope_test=info")
        .expect("component filter should parse");
    let component_subscriber = tracing_subscriber::registry()
        .with(component_capture)
        .with(component_filter);

    tracing::subscriber::with_default(component_subscriber, || {
        scoped_component::emit_from_root();
        prefix_collision_component::emit();
        second_scoped_component::emit();
    });

    assert_eq!(
        *component_targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        [
            "otel.processor.scope_test",
            "otel.processor.scope_test_extra",
        ]
    );
}

/// Scenario: package and component targets are selected by independent directives.
/// Guarantees: a package filter does not accidentally enable component-scoped events.
#[test]
fn package_filter_does_not_match_component_targets() {
    let capture = TargetCapture::default();
    let targets = Arc::clone(&capture.targets);
    let filter =
        EnvFilter::try_new("off,otap-df-telemetry=info").expect("package filter should parse");
    let subscriber = tracing_subscriber::registry().with(capture).with(filter);

    tracing::subscriber::with_default(subscriber, || {
        scoped_component::emit_from_root();
        otap_df_telemetry::otel_info!("test.unscoped");
    });

    assert_eq!(
        *targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        ["otap-df-telemetry"]
    );
}

/// Scenario: two components share kind and name but use different URN namespaces.
/// Guarantees: namespace remains part of the target and prevents identity collisions.
#[test]
fn component_target_preserves_urn_namespace() {
    let capture = TargetCapture::default();
    let targets = Arc::clone(&capture.targets);
    let subscriber = tracing_subscriber::registry().with(capture);

    tracing::subscriber::with_default(subscriber, || {
        scoped_component::emit_from_root();
        namespaced_component::emit();
    });

    assert_eq!(
        *targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        [
            "otel.processor.scope_test",
            "microsoft.processor.scope_test"
        ]
    );
}
