// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end test for `#[component_inventory]` (RFC 0001).
//!
//! Exercises the full link-time path: the macro emits `ComponentMeta` entries
//! into `otap_df_engine::inventory::COMPONENT_INVENTORY`, which we then read
//! back and assert on. This validates the cross-crate emission
//! (`::otap_df_engine::inventory::*` paths) and the `id`-from-`name` derivation.

use otap_df_engine::inventory::{Category, ComponentMeta, attrs, components};
use otap_df_engine_macros::component_inventory;

/// A URN const, mirroring how real factory statics reference their URN.
const TEST_RECEIVER_URN: &str = "urn:otel:receiver:component_inventory_test";

/// Minimal stand-in for a factory struct with a `name` field.
struct FakeFactory {
    #[allow(dead_code)]
    name: &'static str,
}

/// Factory-style static: `id` is derived from the `name` field (the URN const).
#[component_inventory(
    category = Receiver,
    description = "test receiver",
    attributes(port = "4317", protocol = "gRPC (HTTP/2)", auth = "mTLS (opt-in)"),
)]
static TEST_RECEIVER: FakeFactory = FakeFactory {
    name: TEST_RECEIVER_URN,
};

/// Non-factory item: requires an explicit URN-shaped `id`.
#[component_inventory(
    id = "urn:otel:extension:component_inventory_test",
    category = Extension,
)]
#[allow(dead_code)]
struct TestExtension;

fn find(id: &str) -> &'static ComponentMeta {
    components()
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("component `{id}` not found in COMPONENT_INVENTORY"))
}

/// Scenario: a factory-style `static` (with a `name` URN field) is annotated
/// with `#[component_inventory]` and no explicit `id`.
/// Guarantees: a `COMPONENT_INVENTORY` entry is registered at link time whose
/// `id` is the factory's URN, with the declared category, description,
/// attributes, and auto-populated `file`/`line`.
#[test]
fn factory_static_entry_is_registered_with_derived_urn_id() {
    // Touch the static so the linker keeps it (and its slice entry).
    assert_eq!(TEST_RECEIVER.name, TEST_RECEIVER_URN);

    let meta = find(TEST_RECEIVER_URN);
    assert_eq!(meta.category, Category::Receiver);
    assert_eq!(meta.description, Some("test receiver"));
    assert_eq!(meta.attribute(attrs::PORT), Some("4317"));
    assert_eq!(meta.attribute(attrs::PROTOCOL), Some("gRPC (HTTP/2)"));
    assert_eq!(meta.attribute(attrs::AUTH), Some("mTLS (opt-in)"));
    assert_eq!(meta.attribute("does-not-exist"), None);
    assert!(meta.file.ends_with("component_inventory_e2e.rs"));
    assert!(meta.line > 0);
}

/// Scenario: a non-factory `struct` (no `name` field) is annotated with an
/// explicit URN-shaped `id` and no attributes.
/// Guarantees: the entry is registered under the explicit `id`, with the
/// declared category, a `None` description, and an empty attributes slice.
#[test]
fn non_factory_entry_uses_explicit_id() {
    let meta = find("urn:otel:extension:component_inventory_test");
    assert_eq!(meta.category, Category::Extension);
    assert_eq!(meta.description, None);
    assert!(meta.attributes.is_empty());
}

/// Scenario: `Category::urn_segment()` is called for all category variants.
/// Guarantees: each variant (factory and non-factory) maps to its lowercase URN segment,
/// keeping the enum and the URN cross-check in the macro consistent.
#[test]
fn category_urn_segment_mapping() {
    assert_eq!(Category::Receiver.urn_segment(), "receiver");
    assert_eq!(Category::Exporter.urn_segment(), "exporter");
    assert_eq!(Category::Processor.urn_segment(), "processor");
    assert_eq!(Category::Extension.urn_segment(), "extension");
    assert_eq!(Category::Admin.urn_segment(), "admin");
    assert_eq!(Category::Controller.urn_segment(), "controller");
    assert_eq!(Category::Cli.urn_segment(), "cli");
    assert_eq!(Category::Subsystem.urn_segment(), "subsystem");
    assert_eq!(Category::Safety.urn_segment(), "safety");
}

/// A non-factory item annotated with `Category::Admin`.
#[component_inventory(
    id = "urn:otel:admin:test_server",
    category = Admin,
    description = "Test admin server",
)]
#[allow(dead_code)]
struct TestAdminServer;

/// A non-factory item annotated with `Category::Safety`.
#[component_inventory(
    id = "urn:otel:safety:test_limiter",
    category = Safety,
    description = "Test memory limiter",
)]
#[allow(dead_code)]
struct TestSafetyLimiter;

/// Scenario: non-factory components are annotated with `Admin` and `Safety` categories.
/// Guarantees: the entries are registered under their explicit URN IDs and matching categories in `COMPONENT_INVENTORY`.
#[test]
fn non_factory_categories_are_registered() {
    let admin_meta = find("urn:otel:admin:test_server");
    assert_eq!(admin_meta.category, Category::Admin);
    assert_eq!(admin_meta.description, Some("Test admin server"));

    let safety_meta = find("urn:otel:safety:test_limiter");
    assert_eq!(safety_meta.category, Category::Safety);
    assert_eq!(safety_meta.description, Some("Test memory limiter"));
}
