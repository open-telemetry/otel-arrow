// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experiment harness: runs a full engine with a fixed synthetic workload and
//! captures every internal-telemetry OTLP metrics request emitted by the
//! observability pipeline.
//!
//! The test is ignored by default. Run it with:
//!
//! ```text
//! ENTITY_SCOPE_OUT=/some/dir ENTITY_SCOPE_LABEL=baseline-1 \
//!   cargo test -p otel-arrow-dfe-validation --release \
//!   --test entity_scope_volume -- --ignored --nocapture
//! ```
//!
//! `ENTITY_SCOPE_WORKLOAD` selects the workload: `pipeline` (default) or
//! `topology`.
//!
//! Output written to `$ENTITY_SCOPE_OUT/$ENTITY_SCOPE_LABEL/`:
//! - `req-NNNN.pb`: the raw `ExportMetricsServiceRequest` bytes, one file per
//!   captured request.
//! - `points.jsonl`: one JSON object per data point with its scope name, scope
//!   attributes, metric name, data-point attributes, kind, and values. The
//!   comparison script canonicalizes these so the before and after shapes can be
//!   compared without the harness knowing about either shape.

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_admin_api::{
    AdminClient, AdminEndpoint, HttpAdminClientSettings, engine::ProbeStatus,
    operations::OperationOptions,
};
use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_controller::Controller;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_otap::{OTAP_EXPORTER_FACTORIES, OTAP_PIPELINE_FACTORY};
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
use parking_lot::Mutex;
use prost::Message as _;
use serde_json::{Value, json};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

// Link the node factories referenced by URN in the workload YAML.
use otel_arrow_dfe_core_nodes as _;
use otel_arrow_dfe_dev_nodes as _;

const CAPTURE_EXPORTER_URN: &str = "urn:otel:exporter:entity_scope_volume_capture";

/// How long the engine runs before the harness requests shutdown. The traffic
/// generator finishes well before this, so the tail windows show idle telemetry.
const RUN_DURATION: Duration = Duration::from_secs(12);

static CAPTURED: LazyLock<Mutex<Vec<Vec<u8>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

struct CaptureExporter;

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
                    if let Some(bytes) = data.payload_ref().encoded_bytes() {
                        CAPTURED.lock().push(bytes.to_vec());
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
    _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<OtapPdata>, otel_arrow_dfe_config::error::Error> {
    Ok(ExporterWrapper::local(
        CaptureExporter,
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
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otel_arrow_dfe_config::validation::no_config,
};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

/// Workload: a generator pipeline exports over OTLP gRPC to a sink pipeline in
/// the same engine, with detailed runtime metrics and one flow metric so that
/// node, channel, flow, receiver, exporter, pipeline, and engine metric sets all
/// report.
fn workload_yaml(admin_port: u16, otlp_port: u16) -> String {
    format!(
        r#"
version: otel_dataflow/v1
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
engine:
  http_admin:
    bind_address: "127.0.0.1:{admin_port}"
  telemetry:
    reporting_interval: 1s
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
        telemetry:
          type: receiver:internal_telemetry
          config:
            signals: [metrics]
            metrics:
              interval: 1s
        capture:
          type: "{CAPTURE_EXPORTER_URN}"
          config: {{}}
      connections:
        - from: telemetry
          to: capture
groups:
  default:
    pipelines:
      gen:
        policies:
          telemetry:
            pipeline_metrics: true
            tokio_metrics: true
            runtime_metrics: detailed
            flow_metrics:
              - id: enrich
                bounds:
                  start_node: attributes
                  end_node: batch
                duration_distribution: basic
                purpose: enrich
        nodes:
          generator:
            type: receiver:traffic_generator
            config:
              data_source: synthetic
              traffic_config:
                max_signal_count: 6000
                max_batch_size: 100
                signals_per_second: 1000
                log_weight: 70
                metric_weight: 30
                trace_weight: 0
          attributes:
            type: processor:attribute
            config:
              actions:
                - action: insert
                  key: deployment.environment
                  value: experiment
          batch:
            type: processor:batch
            config:
              otap:
                min_size: 500
                sizer: items
              max_batch_duration: 500ms
          exporter:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://127.0.0.1:{otlp_port}"
        connections:
          - from: generator
            to: attributes
          - from: attributes
            to: batch
          - from: batch
            to: exporter
      sink:
        policies:
          telemetry:
            pipeline_metrics: true
            tokio_metrics: true
            runtime_metrics: detailed
        nodes:
          receiver:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:{otlp_port}"
          debug:
            type: processor:debug
            config:
              verbosity: basic
              signals: []
          noop:
            type: exporter:noop
            config: {{}}
        connections:
          - from: receiver
            to: debug
          - from: debug
            to: noop
"#
    )
}

/// Topology workload: exercises the cases where endpoint and port dimensions
/// can vary within one entity.
///
/// - fan-in: two generators feed the router (one channel, two senders), and
///   two filters feed `sink_logs` (one channel, two senders);
/// - multi-port node: `router` has `logs` and `metrics` output ports;
/// - competing consumers: `router["logs"]` feeds `[filter_a, filter_b]`
///   (one MPMC channel, two receivers);
/// - decision nodes: the `logs_a` flow spans `filter_a` -> `filter_c`, two
///   filters that drop records.
fn topology_workload_yaml(admin_port: u16) -> String {
    format!(
        r#"
version: otel_dataflow/v1
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
engine:
  http_admin:
    bind_address: "127.0.0.1:{admin_port}"
  telemetry:
    reporting_interval: 1s
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
        telemetry:
          type: receiver:internal_telemetry
          config:
            signals: [metrics]
            metrics:
              interval: 1s
        capture:
          type: "{CAPTURE_EXPORTER_URN}"
          config: {{}}
      connections:
        - from: telemetry
          to: capture
groups:
  default:
    pipelines:
      topo:
        policies:
          telemetry:
            pipeline_metrics: true
            tokio_metrics: true
            runtime_metrics: detailed
            flow_metrics:
              - id: logs_a
                bounds:
                  start_node: filter_a
                  end_node: filter_c
                duration_distribution: basic
                purpose: filter
        nodes:
          gen_logs:
            type: receiver:traffic_generator
            config:
              data_source: synthetic
              traffic_config:
                max_signal_count: 3000
                max_batch_size: 50
                signals_per_second: 500
                log_weight: 100
                metric_weight: 0
                trace_weight: 0
          gen_mixed:
            type: receiver:traffic_generator
            config:
              data_source: synthetic
              traffic_config:
                max_signal_count: 3000
                max_batch_size: 50
                signals_per_second: 500
                log_weight: 50
                metric_weight: 50
                trace_weight: 0
          router:
            type: processor:type_router
            outputs: ["logs", "metrics"]
            config: {{}}
          filter_a:
            type: processor:filter
            config:
              logs:
                include:
                  match_type: strict
                  severity_texts: [INFO, WARN]
          filter_b:
            type: processor:filter
            config:
              logs:
                include:
                  match_type: strict
                  severity_texts: [WARN, ERROR]
          filter_c:
            type: processor:filter
            config:
              logs:
                exclude:
                  match_type: strict
                  severity_texts: [WARN]
          attrs:
            type: processor:attribute
            config:
              actions:
                - action: insert
                  key: deployment.environment
                  value: experiment
          sink_logs:
            type: exporter:noop
            config: {{}}
          sink_metrics:
            type: exporter:noop
            config: {{}}
        connections:
          - from: gen_logs
            to: router
          - from: gen_mixed
            to: router
          - from: router["logs"]
            to: [filter_a, filter_b]
          - from: router["metrics"]
            to: attrs
          - from: filter_a
            to: filter_c
          - from: filter_c
            to: sink_logs
          - from: filter_b
            to: sink_logs
          - from: attrs
            to: sink_metrics
"#
    )
}

/// Scenario: a fixed synthetic workload runs for a fixed duration.
/// Guarantees: every internal-telemetry metrics request is captured and dumped
/// for offline before/after comparison.
#[test]
#[ignore = "experiment harness; run explicitly with ENTITY_SCOPE_OUT set"]
fn capture_internal_metrics_volume() {
    let out_root = std::env::var("ENTITY_SCOPE_OUT").expect("ENTITY_SCOPE_OUT must be set");
    let label = std::env::var("ENTITY_SCOPE_LABEL").unwrap_or_else(|_| "run".to_owned());
    let out_dir = PathBuf::from(out_root).join(label);
    if out_dir.exists() {
        std::fs::remove_dir_all(&out_dir).expect("clear previous output");
    }
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let admin_port = free_port();
    let otlp_port = free_port();
    let workload = std::env::var("ENTITY_SCOPE_WORKLOAD").unwrap_or_else(|_| "pipeline".to_owned());
    let yaml = match workload.as_str() {
        "pipeline" => workload_yaml(admin_port, otlp_port),
        "topology" => topology_workload_yaml(admin_port),
        other => panic!("unknown ENTITY_SCOPE_WORKLOAD `{other}`"),
    };
    let spec =
        OtelDataflowSpec::from_yaml(&yaml).expect("workload config should parse and validate");

    let engine_thread = std::thread::spawn(move || {
        Controller::new(&OTAP_PIPELINE_FACTORY)
            .run_till_shutdown(spec)
            .map_err(|e| e.to_string())
    });

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(drive_engine(admin_port));

    let engine_result = engine_thread.join().expect("engine thread should join");
    assert!(
        engine_result.is_ok(),
        "engine run failed: {engine_result:?}"
    );

    let captured = std::mem::take(&mut *CAPTURED.lock());
    assert!(!captured.is_empty(), "no internal metrics were captured");
    dump(&out_dir, &captured);
}

async fn drive_engine(admin_port: u16) {
    let endpoint =
        AdminEndpoint::from_url(&format!("http://127.0.0.1:{admin_port}")).expect("admin endpoint");
    let client = AdminClient::builder()
        .http(HttpAdminClientSettings::new(endpoint))
        .build()
        .expect("admin client");

    let mut ready = false;
    for _ in 0..100 {
        if let Ok(resp) = client.engine().readyz().await
            && resp.status == ProbeStatus::Ok
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(ready, "engine did not become ready");

    tokio::time::sleep(RUN_DURATION).await;
    let _ = client
        .groups()
        .shutdown(&OperationOptions::default())
        .await
        .expect("shutdown request should be accepted");
}

fn dump(out_dir: &std::path::Path, captured: &[Vec<u8>]) {
    let mut points =
        std::fs::File::create(out_dir.join("points.jsonl")).expect("create points.jsonl");
    let mut requests =
        std::fs::File::create(out_dir.join("requests.jsonl")).expect("create requests.jsonl");
    for (index, bytes) in captured.iter().enumerate() {
        std::fs::write(out_dir.join(format!("req-{index:04}.pb")), bytes).expect("write request");
        let request =
            ExportMetricsServiceRequest::decode(bytes.as_slice()).expect("valid OTLP metrics");
        let scopes: usize = request
            .resource_metrics
            .iter()
            .map(|rm| rm.scope_metrics.len())
            .sum();
        let metrics: usize = request
            .resource_metrics
            .iter()
            .flat_map(|rm| rm.scope_metrics.iter())
            .map(|sm| sm.metrics.len())
            .sum();
        serde_json::to_writer(
            &mut requests,
            &json!({"request": index, "bytes": bytes.len(), "scopes": scopes, "metrics": metrics}),
        )
        .expect("write request summary");
        requests.write_all(b"\n").expect("write newline");
        for resource_metrics in request.resource_metrics {
            for scope_metrics in resource_metrics.scope_metrics {
                let (scope_name, scope_attrs) = scope_metrics
                    .scope
                    .map(|scope| (scope.name, attrs_json(&scope.attributes)))
                    .unwrap_or_default();
                for metric in scope_metrics.metrics {
                    for point in metric_points(metric.data) {
                        let mut record = json!({
                            "request": index,
                            "scope": scope_name,
                            "scope_attrs": scope_attrs,
                            "metric": metric.name,
                            "unit": metric.unit,
                        });
                        let record_map = record.as_object_mut().expect("object");
                        if let Value::Object(fields) = point {
                            record_map.extend(fields);
                        }
                        serde_json::to_writer(&mut points, &record).expect("write point");
                        points.write_all(b"\n").expect("write newline");
                    }
                }
            }
        }
    }
}

fn metric_points(data: Option<metric::Data>) -> Vec<Value> {
    let Some(data) = data else {
        return Vec::new();
    };
    match data {
        metric::Data::Sum(sum) => sum
            .data_points
            .iter()
            .map(|p| {
                json!({
                    "kind": "sum",
                    "temporality": sum.aggregation_temporality,
                    "monotonic": sum.is_monotonic,
                    "attrs": attrs_json(&p.attributes),
                    "value": number_value(p.value.as_ref()),
                })
            })
            .collect(),
        metric::Data::Gauge(gauge) => gauge
            .data_points
            .iter()
            .map(|p| {
                json!({
                    "kind": "gauge",
                    "attrs": attrs_json(&p.attributes),
                    "value": number_value(p.value.as_ref()),
                })
            })
            .collect(),
        metric::Data::Histogram(hist) => hist
            .data_points
            .iter()
            .map(|p| {
                json!({
                    "kind": "histogram",
                    "temporality": hist.aggregation_temporality,
                    "attrs": attrs_json(&p.attributes),
                    "count": p.count,
                    "sum": p.sum,
                    "min": p.min,
                    "max": p.max,
                })
            })
            .collect(),
        metric::Data::ExponentialHistogram(hist) => hist
            .data_points
            .iter()
            .map(|p| {
                json!({
                    "kind": "exp_histogram",
                    "temporality": hist.aggregation_temporality,
                    "attrs": attrs_json(&p.attributes),
                    "count": p.count,
                    "sum": p.sum,
                    "min": p.min,
                    "max": p.max,
                })
            })
            .collect(),
        metric::Data::Summary(summary) => summary
            .data_points
            .iter()
            .map(|p| {
                json!({
                    "kind": "summary",
                    "attrs": attrs_json(&p.attributes),
                    "count": p.count,
                    "sum": p.sum,
                })
            })
            .collect(),
    }
}

fn number_value(value: Option<&number_data_point::Value>) -> Value {
    match value {
        Some(number_data_point::Value::AsInt(v)) => json!(v),
        Some(number_data_point::Value::AsDouble(v)) => json!(v),
        None => Value::Null,
    }
}

fn attrs_json(attrs: &[KeyValue]) -> Value {
    let mut map = serde_json::Map::with_capacity(attrs.len());
    for kv in attrs {
        let _ = map.insert(kv.key.clone(), any_value_string(kv.value.as_ref()).into());
    }
    Value::Object(map)
}

fn any_value_string(value: Option<&AnyValue>) -> String {
    let Some(value) = value.and_then(|v| v.value.as_ref()) else {
        return String::new();
    };
    match value {
        any_value::Value::StringValue(s) => s.clone(),
        any_value::Value::BoolValue(b) => b.to_string(),
        any_value::Value::IntValue(i) => i.to_string(),
        any_value::Value::DoubleValue(d) => d.to_string(),
        any_value::Value::BytesValue(b) => format!("{b:?}"),
        any_value::Value::ArrayValue(a) => format!("{a:?}"),
        any_value::Value::KvlistValue(kv) => format!("{kv:?}"),
    }
}
