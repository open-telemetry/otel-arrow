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
use otel_arrow_dfe_engine::{
    Interests, ProducerEffectHandlerExtension,
    control::{AckMsg, NackMsg, PipelineCompletionMsg},
    testing::processor::running::RunningProcessor,
};
use otel_arrow_dfe_otap::OTAP_PROCESSOR_FACTORIES;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::TryIntoWithOptions;
use otel_arrow_dfe_pdata::otap::Logs;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_telemetry::metrics::MetricValue;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_wasm_host::WASM_PROCESSOR_URN;

/// Compile one standalone guest plugin to a `wasm32-wasip3` component.
fn build_guest_wasm(plugin_dir: &str, artifact: &str) -> PathBuf {
    let guest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(plugin_dir);

    let status = Command::new("rustup")
        .current_dir(&guest_dir)
        // Host coverage flags and wrappers must not instrument the WASM
        // guest, whose target does not provide the profiler runtime.
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
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
/// wasm processor factory, guest instantiation and initialization, and the
/// `otel-kernels` filter kernel all work together against the real engine.
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
        .run_test(move |mut ctx| async move {
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
            !(matches!(
                name.as_str(),
                "otel:otap-dataflow-plugin/otel-kernels@0.2.0"
                    | "otel:otap-dataflow-plugin/host-services@0.2.0"
            ) || (name.starts_with("wasi:cli/") && name.ends_with("@0.3.0"))
                || (name.starts_with("wasi:clocks/") && name.ends_with("@0.3.0")))
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "guest imports must stay within the plugin, WASI CLI, and WASI clocks allowlist, \
         found: {unexpected:?} \
         (full import list: {imports:?})"
    );

    for required in [
        "otel:otap-dataflow-plugin/otel-kernels@0.2.0",
        "otel:otap-dataflow-plugin/host-services@0.2.0",
    ] {
        assert!(
            imports.iter().any(|name| name == required),
            "expected the guest to import `{required}`, got: {imports:?}"
        );
    }
}

/// Scenario: A real guest returns zero, one, or several outputs using default and named routing.
/// Guarantees: The factory enforces configured output limits and successful calls reclaim handles across repeated invocations.
#[test]
fn wasm_processor_multi_output_contract_end_to_end() {
    use otel_arrow_dfe_engine::testing::processor::TEST_OUT_PORT_NAME;

    let wasm_path = build_test_guest_wasm();
    for count in [0, 1, 4, 5] {
        let controller = ControllerContext::new(TelemetryRegistryHandle::new());
        let pipeline = controller.pipeline_context_with("grp".into(), "multi".into(), 0, 1, 0);
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
        let ports: Vec<Option<&str>> = (0..count)
            .map(|i| (i % 2 == 1).then_some(TEST_OUT_PORT_NAME))
            .collect();
        config.config = serde_json::json!({
            "wasm_path": wasm_path,
            "max_outputs_per_call": 4,
            "max_inflight_inputs": 1,
            "config": { "outputs": ports },
        });
        let factory = OTAP_PROCESSOR_FACTORIES
            .iter()
            .find(|f| f.name == WASM_PROCESSOR_URN)
            .unwrap();
        let processor = (factory.create)(
            pipeline,
            test_node("multi"),
            Arc::new(config),
            rt.config(),
            &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
        )
        .unwrap();
        rt.set_processor(processor)
            .run_test(move |mut ctx| async move {
                for _ in 0..2 {
                    let result = ctx
                        .process(Message::PData(logs_with_severities(&["ERROR", "INFO"])))
                        .await;
                    let outputs = ctx.drain_pdata().await;
                    if count > 4 {
                        assert!(
                            result
                                .unwrap_err()
                                .to_string()
                                .contains("max_outputs_per_call")
                        );
                        assert!(outputs.is_empty());
                        break;
                    }

                    result.unwrap();
                    assert_eq!(outputs.len(), count);
                    for output in outputs {
                        assert_eq!(severities_of(output), ["ERROR", "INFO"]);
                    }
                    assert!(ctx.accept_pdata());
                }
            })
            .validate(|_| async {});
    }
}

/// Scenario: A factory-built guest discovers actual wiring instead of extra declared or plugin-configured ports.
/// Guarantees: The snapshot reports the configured host output limit and sole connected default before the first guest process.
#[test]
fn wasm_processor_discovers_host_output_info() {
    use otel_arrow_dfe_engine::testing::processor::TEST_OUT_PORT_NAME;

    let controller = ControllerContext::new(TelemetryRegistryHandle::new());
    let pipeline = controller.pipeline_context_with("grp".into(), "discovery".into(), 0, 1, 0);
    let rt: TestRuntime<OtapPdata> = TestRuntime::new();
    let mut config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
    config.outputs = vec![TEST_OUT_PORT_NAME.into(), "unconnected".into()];
    config.config = serde_json::json!({
        "wasm_path": build_test_guest_wasm(),
        "max_outputs_per_call": 1,
        "config": {
            "connected_ports": ["forged"],
            "max_outputs_per_call": 99,
            "discover_outputs": {
                "connected_ports": [TEST_OUT_PORT_NAME],
                "default_port": TEST_OUT_PORT_NAME,
                "max_outputs_per_call": 1
            }
        }
    });
    let factory = OTAP_PROCESSOR_FACTORIES
        .iter()
        .find(|f| f.name == WASM_PROCESSOR_URN)
        .unwrap();
    let processor = (factory.create)(
        pipeline,
        test_node("discovery"),
        Arc::new(config),
        rt.config(),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    )
    .unwrap();
    rt.set_processor(processor)
        .run_test(|mut ctx| async move {
            for _ in 0..2 {
                ctx.process(Message::PData(logs_with_severities(&["ERROR", "INFO"])))
                    .await
                    .unwrap();
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1);
                assert_eq!(severities_of(outputs.pop().unwrap()), ["ERROR", "INFO"]);
            }
        })
        .validate(|_| async {});
}
fn running_guest(
    wasm_path: &std::path::Path,
    outputs: &[Option<&str>],
    telemetry: bool,
) -> (RunningProcessor<OtapPdata>, TelemetryRegistryHandle) {
    let interests = if telemetry {
        Interests::NODE_INPUT_METRICS | Interests::NODE_OUTPUT_METRICS | Interests::NODE_ITEM_COUNTS
    } else {
        Interests::empty()
    };
    let (context, registry) =
        otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests(interests);
    let mut config = NodeUserConfig::new_processor_config(WASM_PROCESSOR_URN);
    config.outputs = vec!["a".into(), "b".into()];
    config.default_output = Some("b".into());
    config.config = serde_json::json!({
        "wasm_path": wasm_path,
        "max_outputs_per_call": 3,
        "max_inflight_inputs": 1,
        "config": {"outputs": outputs},
    });
    let factory = OTAP_PROCESSOR_FACTORIES
        .iter()
        .find(|f| f.name == WASM_PROCESSOR_URN)
        .expect("WASM factory registered");
    let processor = (factory.create)(
        context.clone(),
        test_node("test_node"),
        Arc::new(config),
        &otel_arrow_dfe_engine::config::ProcessorConfig::new("test_node"),
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    )
    .expect("construct real guest processor");
    let policy = if telemetry {
        serde_json::from_value(serde_json::json!({
            "runtime_metrics": "detailed",
            "flow_metrics": [{
                "id": "guest", "bounds": {"start_node": "test_node", "end_node": "test_node"}
            }]
        }))
        .expect("valid telemetry policy")
    } else {
        Default::default()
    };
    (
        RunningProcessor::new(
            processor,
            context,
            &["a".into(), "b".into()],
            interests,
            policy,
        ),
        registry,
    )
}

fn subscribe_upstream(input: &mut OtapPdata, id: u64) {
    let (_, reporter) =
        otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(1);
    let effect = otel_arrow_dfe_engine::local::processor::EffectHandler::new(
        otel_arrow_dfe_engine::node::NodeId {
            index: 1,
            name: "upstream".into(),
        },
        Default::default(),
        None,
        reporter,
        otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
    );
    effect.subscribe_to(
        Interests::ACKS_OR_NACKS | Interests::RETURN_DATA,
        [id.into()].into_iter().collect(),
        input,
    );
}

async fn next_output(h: &mut RunningProcessor<OtapPdata>) -> OtapPdata {
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        tokio::select! {
            output = h.output.recv() => output.expect("downstream remains open"),
            result = &mut h.runner => panic!("processor stopped before output: {result:?}"),
        }
    })
    .await
    .expect("processor must emit the next output")
}

async fn complete(h: &mut RunningProcessor<OtapPdata>, data: OtapPdata, outcome: u8) {
    let completion = match outcome {
        0 => PipelineCompletionMsg::DeliverAck {
            ack: AckMsg::new(data),
        },
        1 => PipelineCompletionMsg::DeliverNack {
            nack: NackMsg::new("transient", data),
        },
        2 => PipelineCompletionMsg::DeliverNack {
            nack: NackMsg::new_permanent("permanent", data),
        },
        _ => unreachable!(),
    };
    h.completion
        .send(completion)
        .await
        .expect("return dispatcher remains open");
    // Unwind the child, handle its completion, then unwind the original input.
    assert!(futures::poll!(&mut h.dispatcher).is_pending());
    assert!(futures::poll!(&mut h.runner).is_pending());
    assert!(futures::poll!(&mut h.dispatcher).is_pending());
}

struct ObservedMetric {
    name: String,
    attributes: Vec<(String, String)>,
    value: MetricValue,
}

async fn finish_running(
    h: RunningProcessor<OtapPdata>,
    registry: &TelemetryRegistryHandle,
) -> Vec<ObservedMetric> {
    let node_snapshots = h.node_snapshots();
    // Completions must settle before input closes (engine issue #2477).
    drop(h.input);
    h.runner.await.expect("processor cleanup succeeds");
    drop(h.completion);
    h.dispatcher
        .await
        .expect("return dispatcher cleanup succeeds");
    for snapshot in node_snapshots.into_iter().chain(h.measurements.try_iter()) {
        registry.accumulate_metric_set_snapshot(
            snapshot.key(),
            snapshot.bucket(),
            snapshot.get_metrics(),
        );
    }
    let mut measurements = Vec::new();
    registry.visit_current_metrics_with_item_attrs(
        |descriptor, scope, item, values| {
            let attributes: Vec<_> = scope
                .iter_attributes()
                .map(|(key, value)| (key.to_string(), value.to_string_value()))
                .chain(
                    item.iter()
                        .map(|(key, value)| (key.to_string(), value.to_string())),
                )
                .collect();
            for (field, value) in values {
                measurements.push(ObservedMetric {
                    name: format!("{}.{}", descriptor.name, field.name),
                    attributes: attributes.clone(),
                    value: value.clone(),
                });
            }
        },
        false,
    );
    measurements
}

/// Scenario: A real guest fills a one-input tracker while another input waits on the production inbox.
/// Guarantees: All children settle before admission resumes, with one correctly classified upstream outcome.
#[tokio::test]
async fn wasm_runner_admission_and_completion() {
    let path = build_test_guest_wasm();
    for (outcomes, permanent) in [
        ([0, 0, 0], None),
        ([1, 1, 1], Some(false)),
        ([0, 1, 0], Some(true)),
        ([1, 2, 1], Some(true)),
    ] {
        let (mut h, registry) = running_guest(&path, &[None, Some("a"), None], true);
        let mut first = logs_with_severities(&["FIRST", "ORIGINAL"]);
        subscribe_upstream(&mut first, 11);
        h.input.send(first).unwrap();
        let a = next_output(&mut h).await;
        let b = next_output(&mut h).await;
        let c = next_output(&mut h).await;
        let mut second = logs_with_severities(&["SECOND"]);
        subscribe_upstream(&mut second, 22);
        h.input.send(second).unwrap();
        assert!(futures::poll!(&mut h.runner).is_pending());
        assert!(
            h.output.try_recv().is_err(),
            "full tracker must gate queued input"
        );
        for (child, outcome) in [(c, outcomes[2]), (a, outcomes[0])] {
            complete(&mut h, child, outcome).await;
            assert!(
                h.outcomes.try_recv().is_err(),
                "partial fanout cannot settle input"
            );
            assert!(
                h.output.try_recv().is_err(),
                "partial completion cannot release capacity"
            );
        }
        complete(&mut h, b, outcomes[1]).await;
        match (h.outcomes.try_recv().unwrap(), permanent) {
            (NodeControlMsg::Ack(ack), None) => {
                assert_eq!(u64::from(ack.unwind.route.calldata[0]), 11)
            }
            (NodeControlMsg::Nack(nack), Some(permanent)) => {
                assert_eq!(nack.permanent, permanent);
                assert_eq!(u64::from(nack.unwind.route.calldata[0]), 11);
                assert_eq!(severities_of(*nack.refused), ["FIRST", "ORIGINAL"]);
            }
            other => panic!("unexpected aggregate outcome: {other:?}"),
        }
        assert!(
            h.outcomes.try_recv().is_err(),
            "exactly one original outcome"
        );
        for _ in 0..3 {
            let child = next_output(&mut h).await;
            assert_eq!(severities_of(child.clone()), ["SECOND"]);
            complete(&mut h, child, 0).await;
        }
        let NodeControlMsg::Ack(ack) = h.outcomes.try_recv().unwrap() else {
            panic!("second input ack")
        };
        assert_eq!(u64::from(ack.unwind.route.calldata[0]), 22);
        assert!(h.outcomes.try_recv().is_err());
        let metrics = finish_running(h, &registry).await;
        let original_outcome = match permanent {
            None => "success",
            Some(false) => "failure",
            Some(true) => "refused",
        };
        assert_eq!(
            counter(&metrics, "node.input.messages", &[("signal", "logs")]),
            2
        );
        assert_eq!(
            counter(&metrics, "node.input.items", &[("signal", "logs")]),
            3
        );
        assert_eq!(
            counter(
                &metrics,
                "node.input.messages",
                &[("outcome", original_outcome)]
            ),
            if permanent.is_none() { 2 } else { 1 }
        );
        for (code, outcome) in [(0, "success"), (1, "failure"), (2, "refused")] {
            let count = outcomes.iter().filter(|&&value| value == code).count() as u64;
            assert_eq!(
                counter(
                    &metrics,
                    "node.output.messages",
                    &[("signal", "logs"), ("outcome", outcome)]
                ),
                count + if code == 0 { 3 } else { 0 }
            );
        }
    }
}

fn counter(metrics: &[ObservedMetric], name: &str, attributes: &[(&str, &str)]) -> u64 {
    metrics
        .iter()
        .filter(|metric| {
            metric.name == name
                && attributes.iter().all(|(key, value)| {
                    metric
                        .attributes
                        .iter()
                        .any(|(k, v)| k == key && v == value)
                })
        })
        .map(|metric| {
            let MetricValue::U64(value) = metric.value else {
                panic!("expected counter {name}")
            };
            value
        })
        .sum()
}

/// Scenario: Zero, single, and repeated-port fanout outputs run with enabled node and flow telemetry.
/// Guarantees: Signal/port metrics count each output, input is counted once, and empty output records a drop.
#[tokio::test]
async fn wasm_runner_observable_telemetry() {
    let path = build_test_guest_wasm();
    for ports in [vec![], vec![None], vec![None, Some("a"), None]] {
        let (mut h, registry) = running_guest(&path, &ports, true);
        // No explicit upstream subscriber: node metric interests alone must retain ownership.
        h.input
            .send(logs_with_severities(&["ERROR", "INFO"]))
            .unwrap();
        if ports.is_empty() {
            assert!(futures::poll!(&mut h.runner).is_pending());
            assert!(futures::poll!(&mut h.dispatcher).is_pending());
        } else {
            for _ in &ports {
                let child = next_output(&mut h).await;
                complete(&mut h, child, 0).await;
            }
        }
        assert!(h.output.try_recv().is_err());
        assert!(h.outcomes.try_recv().is_err());
        let metrics = finish_running(h, &registry).await;
        let signal = [("signal", "logs")];
        for (name, expected) in [
            ("flow.input.messages", 1),
            ("flow.input.items", 2),
            ("flow.output.messages", ports.len() as u64),
            ("flow.output.items", 2 * ports.len() as u64),
            ("flow.dropped.items", if ports.is_empty() { 2 } else { 0 }),
            ("node.input.messages", 1),
            ("node.input.items", 2),
            ("node.output.messages", ports.len() as u64),
            ("node.output.items", 2 * ports.len() as u64),
            ("processor.wasm_processor.pdata.records.records.in", 2),
            (
                "processor.wasm_processor.pdata.records.records.out",
                2 * ports.len() as u64,
            ),
        ] {
            assert_eq!(
                counter(&metrics, name, &signal),
                expected,
                "{name}, outputs={}",
                ports.len()
            );
        }
        for (port, count) in [
            ("a", u64::from(ports.len() == 3)),
            ("b", ports.len() as u64 - u64::from(ports.len() == 3)),
        ] {
            assert_eq!(
                counter(
                    &metrics,
                    "node.output.messages",
                    &[
                        ("signal", "logs"),
                        ("outcome", "success"),
                        ("node.port", port)
                    ]
                ),
                count
            );
        }
        let mut durations = 0;
        for metric in &metrics {
            assert_ne!(
                metric.name, "processor.compute.duration",
                "flow timing is not synchronous processor compute timing"
            );
            if metric.name == "flow.compute.duration" {
                assert!(
                    metric
                        .attributes
                        .contains(&("signal".into(), "logs".into()))
                );
                let MetricValue::Distribution(value) = &metric.value else {
                    panic!("flow duration distribution")
                };
                let (count, sum, min, max) = value.summary();
                assert!(sum.is_finite() && min >= 0.0 && max >= min);
                durations += count;
            }
        }
        assert_eq!(durations, ports.len().max(1) as u64);
    }
}

/// Scenario: A real guest produces three outputs but the downstream channel is already closed.
/// Guarantees: Guest rows are counted without emitted rows; the input gets one retryable route-closed Nack.
#[tokio::test]
async fn wasm_runner_guest_rows_are_not_emitted_rows() {
    let path = build_test_guest_wasm();
    let (mut h, registry) = running_guest(&path, &[None, Some("a"), None], true);
    let (_, disconnected) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
    drop(std::mem::replace(&mut h.output, disconnected));
    let mut input = logs_with_severities(&["ERROR", "INFO"]);
    subscribe_upstream(&mut input, 44);
    h.input.send(input).unwrap();
    assert!(futures::poll!(&mut h.runner).is_pending());
    assert!(futures::poll!(&mut h.dispatcher).is_pending());
    let NodeControlMsg::Nack(nack) = h.outcomes.try_recv().unwrap() else {
        panic!("closed route must nack")
    };
    assert!(!nack.permanent);
    assert_eq!(severities_of(*nack.refused), ["ERROR", "INFO"]);
    assert!(h.outcomes.try_recv().is_err());
    assert!(h.output.try_recv().is_err());
    let metrics = finish_running(h, &registry).await;
    assert_eq!(
        counter(
            &metrics,
            "processor.wasm_processor.pdata.records.records.out",
            &[("signal", "logs")]
        ),
        6
    );
    assert_eq!(
        counter(&metrics, "node.output.items", &[("signal", "logs")]),
        0
    );
    assert_eq!(
        counter(&metrics, "node.output.messages", &[("signal", "logs")]),
        0
    );
    assert_eq!(
        counter(
            &metrics,
            "node.input.messages",
            &[("signal", "logs"), ("outcome", "failure")]
        ),
        1
    );
}

/// Scenario: A real guest clones encoded traces onto default, named, and repeated destinations, then one child fails.
/// Guarantees: Fanout preserves order and metadata; one Nack returns the original encoded allocation.
#[tokio::test]
async fn wasm_runner_traces_fanout_recovery() {
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_pdata::testing::{
        fixtures::traces_with_full_resource_and_scope,
        round_trip::{decode_traces, otlp_message_to_bytes},
    };
    let path = build_test_guest_wasm();
    let (mut h, registry) = running_guest(&path, &[None, Some("a"), None], false);
    let traces = traces_with_full_resource_and_scope();
    let payload: otel_arrow_dfe_pdata_codec::OtapPayload =
        otlp_message_to_bytes(&traces.clone().into()).into();
    let original = payload.encoded_bytes().unwrap().clone();
    let address = "127.0.0.1:4317".parse().unwrap();
    let mut input = OtapPdata::new(Context::default(), payload).with_peer_addr(address);
    subscribe_upstream(&mut input, 33);
    h.input.send(input).unwrap();
    let mut outputs = Vec::new();
    for port in [1, 0, 1] {
        let child = next_output(&mut h).await;
        assert_eq!(child.signal_type(), SignalType::Traces);
        assert_eq!(child.peer_addr(), Some(address));
        let route = child.source_route().unwrap();
        assert_eq!(child.get_source_node(), Some(0));
        assert_eq!(route.output_port_index, port);
        let (_, payload) = child.clone().into_parts();
        let records = payload.try_into_with_default().unwrap();
        assert_eq!(decode_traces(records), traces);
        outputs.push(child);
    }
    complete(&mut h, outputs.pop().unwrap(), 1).await;
    complete(&mut h, outputs.remove(0), 0).await;
    assert!(h.outcomes.try_recv().is_err());
    complete(&mut h, outputs.pop().unwrap(), 0).await;
    let NodeControlMsg::Nack(nack) = h.outcomes.try_recv().unwrap() else {
        panic!("mixed delivery must nack")
    };
    assert!(nack.permanent);
    assert_eq!(u64::from(nack.unwind.route.calldata[0]), 33);
    assert_eq!(nack.refused.peer_addr(), Some(address));
    assert_eq!(nack.refused.signal_type(), SignalType::Traces);
    let recovered = nack.refused.payload_ref().encoded_bytes().unwrap();
    assert_eq!(recovered, &original);
    assert_eq!(recovered.as_ptr(), original.as_ptr());
    assert!(h.outcomes.try_recv().is_err());
    let _ = finish_running(h, &registry).await;
}
