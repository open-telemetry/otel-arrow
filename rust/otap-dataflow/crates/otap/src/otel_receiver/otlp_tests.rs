// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{OTEL_RECEIVER_URN, OtelReceiver};
use crate::compression::CompressionMethod;
use crate::fake_data_generator::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use crate::pdata::OtapPdata;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::testing::{
    receiver::{NotSendValidateContext, TestContext, TestRuntime},
    test_node,
};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use otap_df_pdata::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use otap_df_pdata::proto::opentelemetry::metrics::v1::{MetricsData, ResourceMetrics, ScopeMetrics};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, TracesData};
use otap_df_pdata::testing::equiv::assert_equivalent;
use otap_df_telemetry::registry::MetricsRegistryHandle;
use prost::Message;
use serde_json::json;
use std::future::Future;
use std::net::{SocketAddr, TcpListener};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::Code;
use weaver_forge::registry::{ResolvedGroup, ResolvedRegistry};
use weaver_resolved_schema::attribute::Attribute;
use weaver_semconv::attribute::{AttributeType, PrimitiveOrArrayTypeSpec, RequirementLevel};
use weaver_semconv::group::{GroupType, InstrumentSpec, SpanKindSpec};

fn create_logs_service_request() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "a".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    attributes: vec![KeyValue {
                        key: "b".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                log_records: vec![
                    LogRecord {
                        time_unix_nano: 1,
                        attributes: vec![KeyValue {
                            key: "c".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    LogRecord {
                        time_unix_nano: 2,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn create_metrics_service_request() -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn create_traces_service_request() -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: None,
            scope_spans: vec![
                ScopeSpans {
                    ..Default::default()
                },
                ScopeSpans {
                    ..Default::default()
                },
            ],
            schema_url: "opentelemetry.io/schema/traces".to_string(),
        }],
    }
}

#[test]
fn test_otlp_config_parsing() {
    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config_with_max_concurrent_requests = json!({
        "listening_addr": "127.0.0.1:4317",
        "max_concurrent_requests": 5000
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
            .unwrap();
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 5000);
    assert!(receiver.config.grpc.request_compression.is_none());
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(receiver.config.grpc.tcp_nodelay);
    assert_eq!(
        receiver.config.grpc.tcp_keepalive,
        Some(Duration::from_secs(45))
    );
    assert_eq!(
        receiver.config.grpc.tcp_keepalive_interval,
        Some(Duration::from_secs(15))
    );
    assert_eq!(receiver.config.grpc.tcp_keepalive_retries, Some(5));
    assert_eq!(receiver.config.grpc.transport_concurrency_limit, None);
    assert!(receiver.config.grpc.load_shed);
    assert_eq!(
        receiver.config.grpc.initial_stream_window_size,
        Some(8 * 1024 * 1024)
    );
    assert_eq!(
        receiver.config.grpc.initial_connection_window_size,
        Some(32 * 1024 * 1024)
    );
    assert_eq!(receiver.config.grpc.max_frame_size, Some(16 * 1024));
    assert_eq!(
        receiver.config.grpc.max_decoding_message_size,
        Some(4 * 1024 * 1024)
    );
    assert_eq!(receiver.config.grpc.max_concurrent_streams, None);

    let config_with_server_overrides = json!({
        "listening_addr": "127.0.0.1:4317",
        "max_concurrent_requests": 512,
        "tcp_nodelay": false,
        "tcp_keepalive": "60s",
        "tcp_keepalive_interval": "20s",
        "tcp_keepalive_retries": 3,
        "transport_concurrency_limit": 256,
        "load_shed": false,
        "initial_stream_window_size": "4MiB",
        "initial_connection_window_size": "16MiB",
        "max_frame_size": "8MiB",
        "max_decoding_message_size": "6MiB",
        "max_concurrent_streams": 1024,
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_server_overrides).unwrap();
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 512);
    assert!(!receiver.config.grpc.tcp_nodelay);
    assert_eq!(
        receiver.config.grpc.tcp_keepalive,
        Some(Duration::from_secs(60))
    );
    assert_eq!(
        receiver.config.grpc.tcp_keepalive_interval,
        Some(Duration::from_secs(20))
    );
    assert_eq!(receiver.config.grpc.tcp_keepalive_retries, Some(3));
    assert_eq!(receiver.config.grpc.transport_concurrency_limit, Some(256));
    assert!(!receiver.config.grpc.load_shed);
    assert_eq!(
        receiver.config.grpc.initial_stream_window_size,
        Some(4 * 1024 * 1024)
    );
    assert_eq!(
        receiver.config.grpc.initial_connection_window_size,
        Some(16 * 1024 * 1024)
    );
    assert_eq!(receiver.config.grpc.max_frame_size, Some(8 * 1024 * 1024));
    assert_eq!(
        receiver.config.grpc.max_decoding_message_size,
        Some(6 * 1024 * 1024)
    );
    assert_eq!(receiver.config.grpc.max_concurrent_streams, Some(1024));

    let config_with_compression_list = json!({
        "listening_addr": "127.0.0.1:4317",
        "compression_method": ["gzip","zstd"]
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_compression_list).unwrap();
    assert_eq!(
        receiver.config.grpc.request_compression,
        Some(vec![CompressionMethod::Gzip, CompressionMethod::Zstd])
    );

    let config_with_compression_none = json!({
        "listening_addr": "127.0.0.1:4317",
        "compression_method": "none"
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_compression_none).unwrap();
    assert_eq!(receiver.config.grpc.request_compression, Some(vec![]));

    let config_with_timeout = json!({
        "listening_addr": "127.0.0.1:4317",
        "timeout": "30s"
    });
    let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_timeout).unwrap();
    assert_eq!(receiver.config.grpc.timeout, Some(Duration::from_secs(30)));

    let config_with_timeout_ms = json!({
        "listening_addr": "127.0.0.1:4317",
        "timeout": "500ms"
    });
    let receiver = OtelReceiver::from_config(pipeline_ctx, &config_with_timeout_ms).unwrap();
    assert_eq!(
        receiver.config.grpc.timeout,
        Some(Duration::from_millis(500))
    );
}

#[test]
fn test_otlp_tune_max_concurrent_requests() {
    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config_default = json!({ "listening_addr": "127.0.0.1:4317" });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_default).unwrap();
    receiver.tune_max_concurrent_requests(128);
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 128);

    let config_small = json!({
        "listening_addr": "127.0.0.1:4317",
        "max_concurrent_requests": 32
    });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_small).unwrap();
    receiver.tune_max_concurrent_requests(128);
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 32);

    let config_zero = json!({
        "listening_addr": "127.0.0.1:4317",
        "max_concurrent_requests": 0
    });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config_zero).unwrap();
    receiver.tune_max_concurrent_requests(256);
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 256);
}

#[derive(Clone, Default, Debug)]
struct FakeBatchPlan {
    logs_requests: Vec<ExportLogsServiceRequest>,
    metrics_requests: Vec<ExportMetricsServiceRequest>,
    traces_requests: Vec<ExportTraceServiceRequest>,
    expected_logs: Vec<OtlpProtoMessage>,
    expected_metrics: Vec<OtlpProtoMessage>,
    expected_traces: Vec<OtlpProtoMessage>,
}

fn pick_free_port() -> u16 {
    for _ in 0..5 {
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            let port = listener.local_addr().expect("local addr").port();
            return port;
        }
        if let Some(port) = portpicker::pick_unused_port() {
            return port;
        }
    }
    panic!("free port");
}

fn build_test_registry() -> ResolvedRegistry {
    let string_attribute = Attribute {
        name: "service.name".to_string(),
        r#type: AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::String),
        brief: "service name".to_string(),
        examples: None,
        tag: None,
        requirement_level: RequirementLevel::default(),
        sampling_relevant: None,
        note: String::new(),
        stability: None,
        deprecated: None,
        prefix: false,
        tags: None,
        annotations: None,
        value: None,
        role: None,
    };
    let int_attribute = Attribute {
        name: "example.counter".to_string(),
        r#type: AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Int),
        brief: "example counter".to_string(),
        examples: None,
        tag: None,
        requirement_level: RequirementLevel::default(),
        sampling_relevant: None,
        note: String::new(),
        stability: None,
        deprecated: None,
        prefix: false,
        tags: None,
        annotations: None,
        value: None,
        role: None,
    };

    let span_group = ResolvedGroup {
        id: "fake_span".to_string(),
        r#type: GroupType::Span,
        brief: "test span group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute.clone()],
        span_kind: Some(SpanKindSpec::Server),
        events: vec!["fake.event".to_string()],
        metric_name: None,
        instrument: None,
        unit: None,
        name: None,
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    let metric_group = ResolvedGroup {
        id: "fake_metric".to_string(),
        r#type: GroupType::Metric,
        brief: "test metric group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute.clone(), int_attribute],
        span_kind: None,
        events: Vec::new(),
        metric_name: Some("requests.count".to_string()),
        instrument: Some(InstrumentSpec::Counter),
        unit: Some("1".to_string()),
        name: None,
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    let event_group = ResolvedGroup {
        id: "fake_event".to_string(),
        r#type: GroupType::Event,
        brief: "test event group".to_string(),
        note: String::new(),
        prefix: String::new(),
        extends: None,
        stability: None,
        deprecated: None,
        attributes: vec![string_attribute],
        span_kind: None,
        events: Vec::new(),
        metric_name: None,
        instrument: None,
        unit: None,
        name: Some("app.log".to_string()),
        lineage: None,
        display_name: None,
        body: None,
        entity_associations: Vec::new(),
        annotations: None,
    };

    ResolvedRegistry {
        registry_url: "test://otlp_fake_registry".to_string(),
        groups: vec![span_group, metric_group, event_group],
    }
}

fn generate_fake_batches(registry: &ResolvedRegistry) -> FakeBatchPlan {
    let mut plan = FakeBatchPlan::default();
    for batch_size in 1..=100 {
        let logs = fake_otlp_logs(batch_size, registry);
        plan.logs_requests.push(ExportLogsServiceRequest {
            resource_logs: logs.resource_logs.clone(),
        });
        plan.expected_logs.push(OtlpProtoMessage::Logs(logs));

        let metrics = fake_otlp_metrics(batch_size, registry);
        plan.metrics_requests.push(ExportMetricsServiceRequest {
            resource_metrics: metrics.resource_metrics.clone(),
        });
        plan.expected_metrics.push(OtlpProtoMessage::Metrics(metrics));

        let traces = fake_otlp_traces(batch_size, registry);
        plan.traces_requests.push(ExportTraceServiceRequest {
            resource_spans: traces.resource_spans.clone(),
        });
        plan.expected_traces.push(OtlpProtoMessage::Traces(traces));
    }
    plan
}

fn decode_pdata_to_message(pdata: &OtapPdata) -> OtlpProtoMessage {
    let proto_bytes: OtlpProtoBytes = pdata
        .clone()
        .payload()
        .try_into()
        .expect("convert to proto bytes");
    match proto_bytes {
        OtlpProtoBytes::ExportLogsRequest(bytes) => {
            let request = ExportLogsServiceRequest::decode(bytes.as_slice())
                .expect("decode logs payload");
            OtlpProtoMessage::Logs(LogsData {
                resource_logs: request.resource_logs,
            })
        }
        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
            let request = ExportMetricsServiceRequest::decode(bytes.as_slice())
                .expect("decode metrics payload");
            OtlpProtoMessage::Metrics(MetricsData {
                resource_metrics: request.resource_metrics,
            })
        }
        OtlpProtoBytes::ExportTracesRequest(bytes) => {
            let request = ExportTraceServiceRequest::decode(bytes.as_slice())
                .expect("decode traces payload");
            OtlpProtoMessage::Traces(TracesData {
                resource_spans: request.resource_spans,
            })
        }
    }
}

fn fake_batch_scenario(
    grpc_endpoint: String,
    plan: FakeBatchPlan,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("logs client connect");
            for request in plan.logs_requests {
                let response = logs_client
                    .export(request)
                    .await
                    .expect("logs request succeeds")
                    .into_inner();
                assert_eq!(
                    response,
                    ExportLogsServiceResponse {
                        partial_success: None
                    }
                );
            }

            let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("metrics client connect");
            for request in plan.metrics_requests {
                let response = metrics_client
                    .export(request)
                    .await
                    .expect("metrics request succeeds")
                    .into_inner();
                assert_eq!(
                    response,
                    ExportMetricsServiceResponse {
                        partial_success: None
                    }
                );
            }

            let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("traces client connect");
            for request in plan.traces_requests {
                let response = traces_client
                    .export(request)
                    .await
                    .expect("traces request succeeds")
                    .into_inner();
                assert_eq!(
                    response,
                    ExportTraceServiceResponse {
                        partial_success: None
                    }
                );
            }

            ctx.send_shutdown(Instant::now(), "OTLP fake batch test")
                .await
                .expect("shutdown send");
        })
    }
}

fn fake_batch_validation(
    plan: FakeBatchPlan,
) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |mut ctx| {
        Box::pin(async move {
            let mut actual_logs = Vec::new();
            for _ in 0..plan.expected_logs.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("logs timeout")
                    .expect("missing logs");
                let message = decode_pdata_to_message(&pdata);
                assert!(
                    matches!(message, OtlpProtoMessage::Logs(_)),
                    "expected logs payload, got {:?}",
                    message.signal_type()
                );
                actual_logs.push(message);
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("logs ack");
                }
            }

            let mut actual_metrics = Vec::new();
            for _ in 0..plan.expected_metrics.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("metrics timeout")
                    .expect("missing metrics");
                let message = decode_pdata_to_message(&pdata);
                assert!(
                    matches!(message, OtlpProtoMessage::Metrics(_)),
                    "expected metrics payload, got {:?}",
                    message.signal_type()
                );
                actual_metrics.push(message);
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("metrics ack");
                }
            }

            let mut actual_traces = Vec::new();
            for _ in 0..plan.expected_traces.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("traces timeout")
                    .expect("missing traces");
                let message = decode_pdata_to_message(&pdata);
                assert!(
                    matches!(message, OtlpProtoMessage::Traces(_)),
                    "expected traces payload, got {:?}",
                    message.signal_type()
                );
                actual_traces.push(message);
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("traces ack");
                }
            }

            assert_equivalent(&plan.expected_logs, &actual_logs);
            assert_equivalent(&plan.expected_metrics, &actual_metrics);
            assert_equivalent(&plan.expected_traces, &actual_traces);
        })
    }
}

#[test]
fn test_otlp_receiver_round_trip_fake_batches() {
    let registry = build_test_registry();
    let plan = generate_fake_batches(&registry);
    let grpc_addr = "127.0.0.1";
    let grpc_port = pick_free_port();
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let test_runtime = TestRuntime::new();
    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let mut receiver = OtelReceiver::from_config(
        pipeline_ctx,
        &json!({
            "listening_addr": addr.to_string(),
            "wait_for_result": true
        }),
    )
    .unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver)
        .run_test(fake_batch_scenario(grpc_endpoint, plan.clone()))
        .run_validation_concurrent(fake_batch_validation(plan));
}

fn otlp_scenario(
    grpc_endpoint: String,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("logs client connect");
            let logs_response = logs_client
                .export(create_logs_service_request())
                .await
                .expect("logs request succeeds")
                .into_inner();
            assert_eq!(
                logs_response,
                ExportLogsServiceResponse {
                    partial_success: None
                }
            );

            let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("metrics client connect");
            let metrics_response = metrics_client
                .export(create_metrics_service_request())
                .await
                .expect("metrics request succeeds")
                .into_inner();
            assert_eq!(
                metrics_response,
                ExportMetricsServiceResponse {
                    partial_success: None
                }
            );

            let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("traces client connect");
            let traces_response = traces_client
                .export(create_traces_service_request())
                .await
                .expect("traces request succeeds")
                .into_inner();
            assert_eq!(
                traces_response,
                ExportTraceServiceResponse {
                    partial_success: None
                }
            );

            ctx.send_shutdown(Instant::now(), "OTLP test")
                .await
                .expect("shutdown send");
        })
    }
}

fn validation_procedure()
-> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    |mut ctx| {
        Box::pin(async move {
            let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("logs timeout")
                .expect("missing logs");
            let logs_proto: OtlpProtoBytes = logs_pdata
                .clone()
                .payload()
                .try_into()
                .expect("logs conversion");
            assert!(matches!(logs_proto, OtlpProtoBytes::ExportLogsRequest(_)));
            let expected = create_logs_service_request();
            let mut expected_bytes = Vec::new();
            expected.encode(&mut expected_bytes).unwrap();
            assert_eq!(&expected_bytes, logs_proto.as_bytes());
            if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
            {
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("logs ack");
            }

            let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("metrics timeout")
                .expect("missing metrics");
            let metrics_proto: OtlpProtoBytes = metrics_pdata
                .clone()
                .payload()
                .try_into()
                .expect("metrics conversion");
            assert!(matches!(
                metrics_proto,
                OtlpProtoBytes::ExportMetricsRequest(_)
            ));
            let expected = create_metrics_service_request();
            let mut expected_bytes = Vec::new();
            expected.encode(&mut expected_bytes).unwrap();
            assert_eq!(&expected_bytes, metrics_proto.as_bytes());
            if let Some((_node_id, ack)) =
                crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
            {
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("metrics ack");
            }

            let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("traces timeout")
                .expect("missing traces");
            let traces_proto: OtlpProtoBytes = traces_pdata
                .clone()
                .payload()
                .try_into()
                .expect("traces conversion");
            assert!(matches!(
                traces_proto,
                OtlpProtoBytes::ExportTracesRequest(_)
            ));
            let expected = create_traces_service_request();
            let mut expected_bytes = Vec::new();
            expected.encode(&mut expected_bytes).unwrap();
            assert_eq!(&expected_bytes, traces_proto.as_bytes());
            if let Some((_node_id, ack)) =
                crate::pdata::Context::next_ack(AckMsg::new(traces_pdata))
            {
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("traces ack");
            }
        })
    }
}

#[test]
fn test_otlp_receiver_ack() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = portpicker::pick_unused_port().expect("free port");
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let mut receiver = OtelReceiver::from_config(
        pipeline_ctx,
        &json!({
            "listening_addr": addr.to_string(),
            "wait_for_result": true
        }),
    )
    .unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver)
        .run_test(otlp_scenario(grpc_endpoint))
        .run_validation_concurrent(validation_procedure());
}

fn nack_scenario(
    grpc_endpoint: String,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("logs client connect");
            let result = logs_client.export(create_logs_service_request()).await;
            assert!(result.is_err(), "nack should surface error");
            let status = result.unwrap_err();
            assert_eq!(status.code(), Code::Unavailable);
            assert!(
                status
                    .message()
                    .contains("Pipeline processing failed: Test nack reason")
            );

            ctx.send_shutdown(Instant::now(), "OTLP nack test")
                .await
                .expect("shutdown send");
        })
    }
}

fn nack_validation()
-> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    |mut ctx| {
        Box::pin(async move {
            let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("logs timeout")
                .expect("missing logs");
            let nack = NackMsg::new("Test nack reason", logs_pdata);
            if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                ctx.send_control_msg(NodeControlMsg::Nack(nack))
                    .await
                    .expect("send nack");
            }
        })
    }
}

#[test]
fn test_otlp_receiver_nack() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = portpicker::pick_unused_port().expect("free port");
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let mut receiver = OtelReceiver::from_config(
        pipeline_ctx,
        &json!({
            "listening_addr": addr.to_string(),
            "wait_for_result": true
        }),
    )
    .unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver)
        .run_test(nack_scenario(grpc_endpoint))
        .run_validation_concurrent(nack_validation());
}
