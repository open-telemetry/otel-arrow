// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end integration test for the WASM host-kernel processor plugin.
//!
//! Builds standalone `wasm32-wasip3` guests with Rust nightly. The reference
//! `severity-filter` proves the normal end-to-end processor path and import
//! allowlist, while the purpose-built test guest exercises startup failures.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use arrow::array::{Array, RecordBatch, StringArray, UInt16Array};
use arrow::datatypes::{DataType, Field, Schema};

use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::message::Message;
use otel_arrow_dfe_engine::testing::{node::test_node, processor::TestRuntime};
use otel_arrow_dfe_otap::OTAP_PROCESSOR_FACTORIES;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::TryIntoWithOptions;
use otel_arrow_dfe_pdata::otap::Logs;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_wasm_host::WASM_PROCESSOR_URN;

/// Compile one standalone guest plugin to a `wasm32-wasip3` component.
fn build_guest_wasm(plugin_dir: &str, artifact: &str) -> PathBuf {
    let guest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(plugin_dir);

    let status = Command::new("rustup")
        .current_dir(&guest_dir)
        .args([
            "run",
            "nightly",
            "cargo",
            "build",
            "--release",
            "--target",
            "wasm32-wasip3",
        ])
        .status()
        .expect("failed to spawn rustup to build the guest plugin with nightly cargo");
    assert!(
        status.success(),
        "building guest plugin {plugin_dir} failed; ensure the \
         nightly toolchain and wasm32-wasip3 target are installed \
         (`rustup target add --toolchain nightly wasm32-wasip3`)"
    );

    let wasm = guest_dir
        .join("target/wasm32-wasip3/release")
        .join(artifact);
    assert!(wasm.exists(), "guest wasm not found at {wasm:?}");
    wasm
}

fn build_reference_guest_wasm() -> PathBuf {
    build_guest_wasm("plugins/severity-filter", "severity_filter_guest.wasm")
}

fn build_test_guest_wasm() -> PathBuf {
    build_guest_wasm("plugins/test-plugin", "wasm_host_test_guest.wasm")
}

/// Build a Logs `OtapPdata` with a `severity_text` column.
fn logs_with_severities(severities: &[&str]) -> OtapPdata {
    let ids: Vec<u16> = (0..severities.len() as u16).collect();
    let schema = Schema::new(vec![
        Field::new("id", DataType::UInt16, true),
        Field::new("severity_text", DataType::Utf8, true),
    ]);
    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(UInt16Array::from(ids)),
            Arc::new(StringArray::from(severities.to_vec())),
        ],
    )
    .expect("valid record batch");

    let mut records = OtapArrowRecords::Logs(Logs::default());
    records
        .set(ArrowPayloadType::Logs, record_batch)
        .expect("valid logs batch");

    OtapPdata::new(Context::default(), records.into())
}

/// Extract the `severity_text` values of the root Logs batch of an `OtapPdata`.
fn severities_of(pdata: OtapPdata) -> Vec<String> {
    let (_ctx, payload) = pdata.into_parts();
    let records: OtapArrowRecords = payload
        .try_into_with_default()
        .expect("convert payload to otap records");
    let batch = records
        .get(ArrowPayloadType::Logs)
        .expect("logs record batch");
    let column = batch
        .column_by_name("severity_text")
        .expect("severity_text column");
    let strings = column
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("utf8 severity_text");
    (0..strings.len())
        .map(|i| strings.value(i).to_string())
        .collect()
}

/// Scenario: A full pipeline node runs the reference guest end to end,
/// processing a batch of log records with mixed severities.
/// Guarantees: Only `severity_text == "ERROR"` records survive, proving the
/// wasm processor factory, guest instantiation (including the new
/// `initialize`/`shutdown` lifecycle calls), and the `otel-kernels` filter
/// kernel all work together against the real engine.
#[test]
fn wasm_processor_filters_error_severity_end_to_end() {
    let wasm_path = build_reference_guest_wasm();

    // Look the factory up through the engine's distributed_slice registry to
    // prove it is registered like any other processor node.
    let factory = OTAP_PROCESSOR_FACTORIES
        .iter()
        .find(|f| f.name == WASM_PROCESSOR_URN)
        .expect("WasmProcessor factory registered in OTAP_PROCESSOR_FACTORIES");

    let controller_ctx = ControllerContext::new(TelemetryRegistryHandle::new());
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

    let node = test_node("wasm-processor");
    let rt: TestRuntime<OtapPdata> = TestRuntime::new();

    let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
    node_config.config = serde_json::json!({ "wasm_path": wasm_path });

    let processor = (factory.create)(
        pipeline_ctx,
        node,
        Arc::new(node_config),
        rt.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    )
    .expect("create wasm processor");

    let phase = rt.set_processor(processor);

    phase
        .run_test(|mut ctx| async move {
            ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                .await
                .expect("process control");

            let input = logs_with_severities(&["ERROR", "INFO", "ERROR", "WARN"]);
            ctx.process(Message::PData(input))
                .await
                .expect("process pdata");

            let out = ctx.drain_pdata().await;
            let first = out.into_iter().next().expect("one output message");

            let severities = severities_of(first);
            assert_eq!(
                severities,
                vec!["ERROR".to_string(), "ERROR".to_string()],
                "only ERROR log records should survive the filter"
            );
        })
        .validate(|_| async move {});
}

/// Scenario: The processor factory is created against the test plugin whose
/// plugin-specific config requests `{"fail_init": true}`, so the guest's
/// `initialize` export returns an `init-error`.
/// Guarantees: Factory construction fails with `ConfigError::InvalidUserConfig`
/// (a pipeline *startup* error), not a panic, and no processor node is ever
/// produced -- proving misconfigured plugins fail fast before any `process`
/// call is attempted.
#[test]
fn wasm_processor_factory_rejects_guest_init_error() {
    let wasm_path = build_test_guest_wasm();

    let factory = OTAP_PROCESSOR_FACTORIES
        .iter()
        .find(|f| f.name == WASM_PROCESSOR_URN)
        .expect("WasmProcessor factory registered in OTAP_PROCESSOR_FACTORIES");

    let controller_ctx = ControllerContext::new(TelemetryRegistryHandle::new());
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

    let node = test_node("wasm-processor-bad-init");
    let rt: TestRuntime<OtapPdata> = TestRuntime::new();

    let mut node_config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
    node_config.config = serde_json::json!({
        "wasm_path": wasm_path,
        "config": { "fail_init": true },
    });

    let result = (factory.create)(
        pipeline_ctx,
        node,
        Arc::new(node_config),
        rt.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    );

    match result {
        Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig { error }) => {
            assert!(
                error.contains("fail_init"),
                "startup error should surface the guest's init-error reason: {error}"
            );
        }
        Ok(_) => panic!("plugin with fail_init=true must not construct a processor"),
        Err(other) => panic!("expected InvalidUserConfig, got {other:?}"),
    }
}

/// Scenario: the compiled reference guest component is inspected for the set of
/// component-model instance imports it declares to the host.
/// Guarantees: every import belongs to the plugin contract or the explicitly
/// allowed WASI CLI/clock interfaces required by `std`; filesystem, sockets,
/// random, and any future capability creep fail with the offending names.
#[test]
fn guest_imports_only_the_sandboxed_interfaces() {
    let wasm_path = build_reference_guest_wasm();

    let mut config = wasmtime::Config::new();
    let _ = config.wasm_component_model_async(true);
    let _ = config.wasm_component_model_more_async_builtins(true);
    let _ = config.wasm_component_model_async_stackful(true);
    let engine = wasmtime::Engine::new(&config).expect("valid wasm engine config");
    let component = wasmtime::component::Component::from_file(&engine, &wasm_path)
        .expect("reference guest should be a valid component");

    let imports: Vec<String> = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_string())
        .collect();

    let unexpected: Vec<&String> = imports
        .iter()
        .filter(|name| {
            !name.starts_with("otel:otap-dataflow-plugin/")
                && !(name.starts_with("wasi:cli/") && name.ends_with("@0.3.0"))
                && !(name.starts_with("wasi:clocks/") && name.ends_with("@0.3.0"))
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "guest imports must stay within the plugin, WASI CLI, and WASI clocks allowlist, \
         found: {unexpected:?} \
         (full import list: {imports:?})"
    );

    for required in ["otel-kernels", "host-services"] {
        assert!(
            imports.iter().any(|name| name.contains(required)),
            "expected the guest to import `{required}`, got: {imports:?}"
        );
    }
}
