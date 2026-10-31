// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offline integration of real AKV extension and database receiver factories.
//! No vault access, database connection, or native Oracle initialization occurs.
//! Opt in with `akv-database-bindings`; the sample binary stays provider-agnostic.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use otel_arrow_dfe_config::{extension::ExtensionUserConfig, node::NodeUserConfig};
use otel_arrow_dfe_contrib_extensions::azure_key_vault_user_pass_auth::{
    AZURE_KEY_VAULT_USER_PASS_AUTH_EXTENSION, AZURE_KEY_VAULT_USER_PASS_AUTH_URN,
};
use otel_arrow_dfe_contrib_nodes::receivers::{
    oracle_receiver::ORACLE_RECEIVER, postgresql_receiver::POSTGRESQL_RECEIVER,
    sql_server_receiver::SQL_SERVER_RECEIVER,
};
use otel_arrow_dfe_engine::{
    ReceiverFactory,
    capability::{
        auth::basic_auth_provider::BasicAuthProvider,
        registry::{Capabilities, CapabilityRegistry},
    },
    config::ExtensionConfig,
    context::ControllerContext,
    testing::{
        capability::resolve_bindings_for_test, receiver::TestRuntime, test_extension_ctx, test_node,
    },
};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use serde_json::Value;

fn check_binding(example: &str, factory: &ReceiverFactory<OtapPdata>) {
    let document: Value = serde_yaml::from_str(example).expect("example parses as YAML");
    let pipeline = &document["groups"]["default"]["pipelines"]["main"];
    let extension_config = &pipeline["extensions"]["database-credentials"];
    assert_eq!(extension_config["type"], AZURE_KEY_VAULT_USER_PASS_AUTH_URN);
    let extension_factory = &AZURE_KEY_VAULT_USER_PASS_AUTH_EXTENSION;
    (extension_factory.validate_config)(&extension_config["config"])
        .expect("example AKV config is valid");
    let (context, _metrics) = test_extension_ctx();
    let name = "database-credentials".into();
    let bundle = (extension_factory.create)(
        &context,
        name,
        Arc::new(ExtensionUserConfig::new(
            extension_factory.name.into(),
            extension_config["config"].clone(),
        )),
        &ExtensionConfig::new("database-credentials"),
    )
    .expect("AKV factory constructs offline");
    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(extension_factory.capabilities.as_ref(), &mut registry)
        .expect("AKV BasicAuthProvider registers");
    let known = HashSet::from(["database-credentials".into()]);
    let bindings = serde_json::from_value(pipeline["nodes"]["database"]["capabilities"].clone())
        .expect("example capability bindings deserialize");
    let resolved = resolve_bindings_for_test(&bindings, &registry, &known)
        .expect("local capability binding resolves");
    // Production receiver factories request the local adapter of a shared provider.
    assert!(resolved.require_local::<BasicAuthProvider>().is_ok());
    let resolved = resolve_bindings_for_test(&bindings, &registry, &known)
        .expect("shared capability binding resolves");
    assert!(resolved.require_shared::<BasicAuthProvider>().is_ok());
    assert!(
        resolve_bindings_for_test(
            &HashMap::from([("basic_auth_provider".into(), "missing".into())]),
            &registry,
            &known,
        )
        .is_err()
    );
    assert!(
        resolve_bindings_for_test(
            &HashMap::from([(
                "sasl_credential_provider".into(),
                "database-credentials".into()
            )]),
            &registry,
            &known,
        )
        .is_err()
    );

    let directory = tempfile::tempdir().expect("isolated checkpoint directory");
    let mut node = NodeUserConfig::new_receiver_config(factory.name);
    assert_eq!(pipeline["nodes"]["database"]["type"], factory.name);
    node.config = pipeline["nodes"]["database"]["config"].clone();
    node.config["checkpoint"]["directory"] = serde_json::json!(directory.path());
    node.capabilities = bindings.clone();
    (factory.validate_config)(&node.config).expect("example receiver config is valid");
    let node = Arc::new(node);
    let runtime = TestRuntime::<OtapPdata>::new();
    let context = || {
        ControllerContext::new(TelemetryRegistryHandle::new()).pipeline_context_with(
            "group".into(),
            "pipeline".into(),
            0,
            1,
            0,
        )
    };
    let build = |capabilities: &Capabilities| {
        (factory.create)(
            context(),
            test_node("database"),
            Arc::clone(&node),
            runtime.config(),
            capabilities,
        )
    };
    assert!(
        build(&Capabilities::empty()).is_err(),
        "binding is required"
    );
    let resolve = || {
        resolve_bindings_for_test(&bindings, &registry, &known)
            .expect("fresh receiver capability binding resolves")
    };
    let receiver = build(&resolve()).expect("real receiver accepts shared AKV BasicAuthProvider");
    assert!(
        build(&resolve()).is_err(),
        "source ownership remains exclusive"
    );
    drop(receiver);
    let _restarted = build(&resolve()).expect("ownership is released for a restart");
}

/// Scenario: The Oracle example binds the real AKV extension through BasicAuthProvider.
/// Guarantees: Factory construction and restart succeed offline, missing/wrong bindings fail, and source ownership is retained.
#[test]
fn oracle_akv_binding() {
    check_binding(
        include_str!("../../../configs/oracle-akv-console.yaml"),
        &ORACLE_RECEIVER,
    );
}

/// Scenario: The PostgreSQL example binds the real AKV extension through BasicAuthProvider.
/// Guarantees: Factory construction and restart succeed offline, missing/wrong bindings fail, and source ownership is retained.
#[test]
fn postgresql_akv_binding() {
    check_binding(
        include_str!("../../../configs/postgresql-akv-console.yaml"),
        &POSTGRESQL_RECEIVER,
    );
}

/// Scenario: The SQL Server example binds the real AKV extension through BasicAuthProvider.
/// Guarantees: Factory construction and restart succeed offline, missing/wrong bindings fail, and source ownership is retained.
#[test]
fn sql_server_akv_binding() {
    check_binding(
        include_str!("../../../configs/sql-server-akv-console.yaml"),
        &SQL_SERVER_RECEIVER,
    );
}
