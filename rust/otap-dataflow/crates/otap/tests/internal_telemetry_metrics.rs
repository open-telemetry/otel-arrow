// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for registry-backed metrics routed through the real
//! internal telemetry receiver factory and an in-process exporter.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::DeployedPipelineKey;
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_core_nodes::receivers::internal_telemetry_receiver::INTERNAL_TELEMETRY_RECEIVER;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    AckMsg, NodeControlMsg, RuntimeControlMsg, pipeline_completion_msg_channel,
    runtime_ctrl_msg_channel,
};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_otap::pdata::OtapPdata;
use otap_df_otap::{OTAP_EXPORTER_FACTORIES, OTAP_PIPELINE_FACTORY};
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, metric, number_data_point,
};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::{InternalTelemetrySystem, LogContext};
use otap_df_telemetry_macros::{attribute_set, metric_set};
use parking_lot::Mutex;
use prost::Message as _;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const CAPTURE_EXPORTER_URN: &str = "urn:otel:exporter:internal_telemetry_test_capture";
const VIEWED_METRIC_NAME: &str = "test_pipeline_events";
const VIEWED_METRIC_DESCRIPTION: &str =
    "Number of test events emitted through the internal telemetry pipeline.";
static CAPTURE_SENDER: LazyLock<Mutex<Option<SyncSender<Vec<u8>>>>> =
    LazyLock::new(|| Mutex::new(None));

struct CaptureExporter {
    sender: SyncSender<Vec<u8>>,
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for CaptureExporter {
    async fn start(
        self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match inbox.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    if let OtapPayload::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) =
                        data.payload_ref()
                    {
                        let _ = self.sender.try_send(bytes.to_vec());
                    }
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {}
            }
        }
        Ok(TerminalState::default())
    }
}

fn create_capture_exporter(
    _pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &ExporterConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<OtapPdata>, otap_df_config::error::Error> {
    let sender = CAPTURE_SENDER.lock().take().ok_or_else(|| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: "internal telemetry test capture sender was not registered".to_owned(),
        }
    })?;
    Ok(ExporterWrapper::local(
        CaptureExporter { sender },
        node,
        node_config,
        exporter_config,
    ))
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
static CAPTURE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CAPTURE_EXPORTER_URN,
    create: create_capture_exporter,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::no_config,
};

#[metric_set(name = "test.its.pipeline")]
#[derive(Debug, Default)]
struct TestMetrics {
    /// Number of events routed through the test pipeline.
    #[metric(unit = "{event}")]
    emitted: Counter<u64>,
}

#[attribute_set(scope, name = "test.its.pipeline.attrs")]
#[derive(Debug, Clone)]
struct TestAttributes {
    /// Test route used to select one metric-set entity.
    #[attribute_key = "test.route"]
    route: String,
}

fn decode_test_metrics(bytes: &[u8]) -> ((String, String, i64), i64) {
    let request = ExportMetricsServiceRequest::decode(bytes).expect("valid OTLP metrics request");
    let mut viewed = None;
    let mut unviewed = None;
    for resource_metrics in request.resource_metrics {
        for scope_metrics in resource_metrics.scope_metrics {
            let Some(scope) = scope_metrics.scope else {
                continue;
            };
            if scope.name != "test.its.pipeline" {
                continue;
            }
            for metric in scope_metrics.metrics {
                if metric.name != VIEWED_METRIC_NAME && metric.name != "emitted" {
                    continue;
                }
                let metric_name = metric.name;
                let metric_description = metric.description;
                let Some(metric::Data::Sum(sum)) = metric.data else {
                    panic!("test metric must be encoded as a sum")
                };
                assert!(sum.is_monotonic);
                assert_eq!(
                    sum.aggregation_temporality,
                    AggregationTemporality::Delta as i32
                );
                let [point] = sum.data_points.as_slice() else {
                    panic!("expected exactly one test metric data point")
                };
                let Some(number_data_point::Value::AsInt(value)) = point.value else {
                    panic!("test metric must contain an integer value")
                };
                if metric_name == VIEWED_METRIC_NAME {
                    viewed = Some((metric_name, metric_description, value));
                } else {
                    unviewed = Some(value);
                }
            }
        }
    }
    (
        viewed.expect("captured request must contain the viewed test metric"),
        unviewed.expect("captured request must contain the unviewed test metric"),
    )
}

#[test]
fn applies_receiver_interval_and_view_in_the_internal_telemetry_pipeline() {
    let spec = OtelDataflowSpec::from_yaml(&format!(
        r#"
version: otel_dataflow/v1
engine:
  telemetry:
    reporting_interval: 60s
    metrics:
      provider: its
    logs:
      providers:
        global: noop
        engine: noop
        internal: noop
        admin: noop
  observability:
    pipeline:
      policies:
        telemetry:
          pipeline_metrics: false
          tokio_metrics: false
          runtime_metrics: none
      nodes:
        internal:
          type: receiver:internal_telemetry
          config:
            metrics:
              interval: 25ms
              views:
                - selector:
                    scope_name: test.its.pipeline
                    scope_attributes:
                      test.route: selected
                    instrument_name: emitted
                  stream:
                    name: {VIEWED_METRIC_NAME}
                    description: {VIEWED_METRIC_DESCRIPTION}
        capture:
          type: "{CAPTURE_EXPORTER_URN}"
          config: {{}}
      connections:
        - from: internal
          to: capture
groups: {{}}
"#
    ))
    .expect("ITS observability config should parse and validate");
    let (engine, regular_pipelines, observability_pipeline) = spec.resolve().into_parts();
    assert!(regular_pipelines.is_empty());
    let observability_pipeline =
        observability_pipeline.expect("resolved config should contain observability pipeline");

    // Force-link and verify the real ITR and the in-process exporter before
    // PipelineFactory::build exercises the settings injection/factory path.
    assert!(
        OTAP_PIPELINE_FACTORY
            .get_receiver_factory_map()
            .contains_key(INTERNAL_TELEMETRY_RECEIVER.name)
    );
    assert!(
        OTAP_PIPELINE_FACTORY
            .get_exporter_factory_map()
            .contains_key(CAPTURE_EXPORTER_URN)
    );

    let registry = TelemetryRegistryHandle::new();
    let telemetry_system = InternalTelemetrySystem::new(
        &engine.telemetry,
        registry.clone(),
        None,
        SendPolicy::default(),
        LogContext::new,
        None,
    )
    .expect("ITS telemetry system should initialize");
    let internal_settings = telemetry_system
        .internal_telemetry_settings()
        .expect("ITS metrics should create receiver settings");

    let collector_cancel = CancellationToken::new();
    // Constructing the run future synchronously publishes collector readiness,
    // matching controller startup's readiness barrier.
    let collector_task = telemetry_system.collector().run(collector_cancel.clone());
    let collector_thread = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("collector runtime")
            .block_on(collector_task)
    });

    // Exercise the real hot metric set -> reporter -> collector path rather
    // than inserting values directly into the registry.
    let mut selected_metric_set = registry.register_metric_set::<TestMetrics>(TestAttributes {
        route: "selected".to_owned(),
    });
    selected_metric_set.emitted.add(3);
    telemetry_system
        .reporter()
        .report(&mut selected_metric_set)
        .expect("metric snapshot should enter the collector channel");
    assert_eq!(
        selected_metric_set.emitted.get(),
        0,
        "successful report resets the hot set"
    );
    let mut unselected_metric_set = registry.register_metric_set::<TestMetrics>(TestAttributes {
        route: "unselected".to_owned(),
    });
    unselected_metric_set.emitted.add(5);
    telemetry_system
        .reporter()
        .report(&mut unselected_metric_set)
        .expect("unselected metric snapshot should enter the collector channel");

    let (capture_tx, capture_rx) = sync_channel(4);
    assert!(
        CAPTURE_SENDER.lock().replace(capture_tx).is_none(),
        "capture sender should not leak across tests"
    );

    let pipeline_group_id = observability_pipeline.pipeline_group_id.clone();
    let pipeline_id = observability_pipeline.pipeline_id.clone();
    let channel_capacity = observability_pipeline.policies.channel_capacity.clone();
    let telemetry_policy = observability_pipeline.policies.telemetry.clone();
    let controller_context = ControllerContext::new(registry.clone());
    let pipeline_context = controller_context.pipeline_context_with(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        0,
        1,
        0,
    );
    let pipeline_entity_key = pipeline_context.register_pipeline_entity();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_context.clone(),
            observability_pipeline.pipeline,
            channel_capacity.clone(),
            telemetry_policy,
            None,
            None,
            Some(internal_settings),
        )
        .expect("real internal telemetry receiver should build with injected settings");

    let (runtime_ctrl_tx, runtime_ctrl_rx) =
        runtime_ctrl_msg_channel(channel_capacity.control.pipeline);
    let (pipeline_completion_tx, pipeline_completion_rx) =
        pipeline_completion_msg_channel(channel_capacity.control.completion);
    let shutdown_tx = runtime_ctrl_tx.clone();
    let capture_thread = std::thread::spawn(move || {
        let captured = capture_rx.recv_timeout(Duration::from_secs(5));
        let _ = shutdown_tx.try_send(RuntimeControlMsg::Shutdown {
            deadline: Instant::now() + Duration::from_secs(1),
            reason: "test metric captured".to_owned(),
        });
        captured
    });

    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let event_reporter = observed_state_store.reporter(SendPolicy::default());
    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id,
        pipeline_id,
        core_id: 0,
        deployment_generation: 0,
    };
    let (_memory_pressure_tx, memory_pressure_rx) = tokio::sync::watch::channel(
        otap_df_engine::memory_limiter::MemoryPressureChanged::initial(),
    );
    let _pipeline_entity_guard = otap_df_engine::entity_context::set_pipeline_entity_key(
        pipeline_context.metrics_registry(),
        pipeline_entity_key,
    );
    let run_result = runtime_pipeline.run_forever(
        pipeline_key,
        pipeline_context,
        event_reporter,
        telemetry_system.reporter(),
        engine.telemetry.reporting_interval,
        memory_pressure_rx,
        runtime_ctrl_tx,
        runtime_ctrl_rx,
        pipeline_completion_tx,
        pipeline_completion_rx,
    );

    let captured = capture_thread
        .join()
        .expect("capture thread should join")
        .expect("timed out waiting for ITS metrics at the real exporter");
    collector_cancel.cancel();
    let collector_result = collector_thread
        .join()
        .expect("collector thread should join");

    assert!(
        run_result.is_ok(),
        "observability pipeline should shut down cleanly: {run_result:?}"
    );
    collector_result.expect("collector should shut down cleanly");
    assert_eq!(
        decode_test_metrics(&captured),
        (
            (
                VIEWED_METRIC_NAME.to_owned(),
                VIEWED_METRIC_DESCRIPTION.to_owned(),
                3,
            ),
            5,
        ),
    );
    telemetry_system
        .shutdown_otel()
        .expect("ITS mode has no SDK shutdown failure");
}
