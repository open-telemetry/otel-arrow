// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end integration test for the WASM host-kernel processor plugin.
//!
//! Builds the reference `severity-filter` guest plugin from source (targeting
//! `wasm32-wasip2`), looks the WASM processor factory up through the engine's
//! `distributed_slice` registry, runs it as a real processor node on a small
//! pipeline via the engine test harness, and asserts that only log records with
//! `severity_text == "ERROR"` survive.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use arrow::array::{Array, RecordBatch, StringArray, UInt16Array};
use arrow::datatypes::{DataType, Field, Schema};

use otap_df_config::node::NodeUserConfig;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::message::Message;
use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::otap::Logs;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_wasm_host::WASM_PROCESSOR_URN;

/// Compile the reference guest plugin to a `wasm32-wasip2` component and return
/// the path to the produced `.wasm` file.
fn build_guest_wasm() -> PathBuf {
    let guest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins/severity-filter");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let status = Command::new(cargo)
        .current_dir(&guest_dir)
        .args(["build", "--release", "--target", "wasm32-wasip2"])
        .status()
        .expect("failed to spawn cargo to build the guest plugin");
    assert!(
        status.success(),
        "building the severity-filter guest plugin failed; ensure the \
         wasm32-wasip2 target is installed (`rustup target add wasm32-wasip2`)"
    );

    let wasm = guest_dir.join("target/wasm32-wasip2/release/severity_filter_guest.wasm");
    assert!(wasm.exists(), "guest wasm not found at {wasm:?}");
    wasm
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

#[test]
fn wasm_processor_filters_error_severity_end_to_end() {
    let wasm_path = build_guest_wasm();

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
        &otap_df_engine::capability::registry::Capabilities::empty(),
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
