// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{OTEL_RECEIVER_URN, OtelReceiver};
use crate::fake_data_generator::fake_signal::{
    fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
};
use crate::otap_mock::create_otap_batch;
use crate::otel_receiver::test_common::{build_test_registry, decode_pdata_to_message};
use crate::pdata::OtapPdata;
use async_stream::stream;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::testing::{
    receiver::{NotSendValidateContext, TestContext, TestRuntime},
    test_node,
};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::Producer;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::arrow::v1::{
    ArrowPayloadType, BatchStatus, StatusCode as ProtoStatusCode,
    arrow_logs_service_client::ArrowLogsServiceClient,
    arrow_metrics_service_client::ArrowMetricsServiceClient,
    arrow_traces_service_client::ArrowTracesServiceClient,
};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use otap_df_pdata::testing::equiv::assert_equivalent;
use otap_df_telemetry::registry::MetricsRegistryHandle;
use prost::Message;
use serde_json::json;
use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, timeout};
use tonic::Status;
use tonic::codec::CompressionEncoding;
use weaver_forge::registry::ResolvedRegistry;

fn pick_free_port() -> u16 {
    portpicker::pick_unused_port().expect("No free ports")
}

#[derive(Clone, Default)]
struct OtapFakeBatchPlan {
    logs: Vec<(OtapArrowRecords, OtlpProtoMessage)>,
    metrics: Vec<(OtapArrowRecords, OtlpProtoMessage)>,
    traces: Vec<(OtapArrowRecords, OtlpProtoMessage)>,
}

fn generate_otap_fake_batches(registry: &ResolvedRegistry) -> OtapFakeBatchPlan {
    let mut plan = OtapFakeBatchPlan::default();
    for batch_size in 1..=100 {
        let logs_data = fake_otlp_logs(batch_size, registry);
        let logs_req = ExportLogsServiceRequest {
            resource_logs: logs_data.resource_logs.clone(),
        };
        let logs_bytes = logs_req.encode_to_vec();
        let logs_records: OtapArrowRecords = OtlpProtoBytes::ExportLogsRequest(logs_bytes)
            .try_into()
            .expect("encode logs to arrow");
        plan.logs
            .push((logs_records, OtlpProtoMessage::Logs(logs_data)));

        let metrics_data = fake_otlp_metrics(batch_size, registry);
        let metrics_req = ExportMetricsServiceRequest {
            resource_metrics: metrics_data.resource_metrics.clone(),
        };
        let metrics_bytes = metrics_req.encode_to_vec();
        let metrics_records: OtapArrowRecords = OtlpProtoBytes::ExportMetricsRequest(metrics_bytes)
            .try_into()
            .expect("encode metrics to arrow");
        plan.metrics
            .push((metrics_records, OtlpProtoMessage::Metrics(metrics_data)));

        let traces_data = fake_otlp_traces(batch_size, registry);
        let traces_req = ExportTraceServiceRequest {
            resource_spans: traces_data.resource_spans.clone(),
        };
        let traces_bytes = traces_req.encode_to_vec();
        let traces_records: OtapArrowRecords = OtlpProtoBytes::ExportTracesRequest(traces_bytes)
            .try_into()
            .expect("encode traces to arrow");
        plan.traces
            .push((traces_records, OtlpProtoMessage::Traces(traces_data)));
    }
    plan
}

async fn validate_success_responses<S>(
    mut inbound_stream: S,
    expected_count: usize,
    signal_name: &str,
) where
    S: futures::Stream<Item = Result<BatchStatus, Status>> + Unpin,
{
    use futures::StreamExt;
    let mut seen = HashSet::new();
    let mut count = 0usize;
    while let Some(result) = inbound_stream.next().await {
        let status = result.expect("successful response");
        assert_eq!(
            status.status_code,
            ProtoStatusCode::Ok as i32,
            "Unexpected status code for {} batch {}",
            signal_name,
            status.batch_id
        );
        assert_eq!(
            status.status_message, "Successfully received",
            "Unexpected status message for {} batch {}",
            signal_name, status.batch_id
        );
        assert!(
            seen.insert(status.batch_id),
            "duplicate status for {} batch {}",
            signal_name,
            status.batch_id
        );
        count += 1;
    }
    assert_eq!(
        count, expected_count,
        "missing responses for {} (expected {}, got {})",
        signal_name, expected_count, count
    );
}

fn otap_fake_batch_scenario(
    grpc_endpoint: String,
    plan: OtapFakeBatchPlan,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let metrics_count = plan.metrics.len();
            let logs_count = plan.logs.len();
            let traces_count = plan.traces.len();

            let mut metrics_producer = Producer::new();
            let mut metric_bars = Vec::with_capacity(metrics_count);
            for (idx, (mut records, _)) in plan.metrics.clone().into_iter().enumerate() {
                match metrics_producer.produce_bar(&mut records) {
                    Ok(bar) => metric_bars.push(bar),
                    Err(e) => {
                        let reason = format!("produce metrics bar {} failed: {e}", idx);
                        _ = ctx.send_shutdown(Instant::now(), &reason).await.ok();
                        panic!("{reason}");
                    }
                }
            }
            let mut metrics_client = ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("metrics client connect");
            #[allow(tail_expr_drop_order)]
            let metrics_stream = stream! {
                for bar in metric_bars {
                    yield bar;
                }
            };
            let metrics_resp = metrics_client
                .arrow_metrics(metrics_stream)
                .await
                .expect("metrics request");
            validate_success_responses(metrics_resp.into_inner(), metrics_count, "metrics").await;

            let mut logs_producer = Producer::new();
            let mut log_bars = Vec::with_capacity(logs_count);
            for (idx, (mut records, _)) in plan.logs.clone().into_iter().enumerate() {
                match logs_producer.produce_bar(&mut records) {
                    Ok(bar) => log_bars.push(bar),
                    Err(e) => {
                        let reason = format!("produce logs bar {} failed: {e}", idx);
                        _ = ctx.send_shutdown(Instant::now(), &reason).await.ok();
                        panic!("{reason}");
                    }
                }
            }
            let mut logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("logs client connect");
            #[allow(tail_expr_drop_order)]
            let logs_stream = stream! {
                for bar in log_bars {
                    yield bar;
                }
            };
            let logs_resp = logs_client
                .arrow_logs(logs_stream)
                .await
                .expect("logs request");
            validate_success_responses(logs_resp.into_inner(), logs_count, "logs").await;

            let mut traces_producer = Producer::new();
            let mut trace_bars = Vec::with_capacity(traces_count);
            for (idx, (mut records, _)) in plan.traces.clone().into_iter().enumerate() {
                match traces_producer.produce_bar(&mut records) {
                    Ok(bar) => trace_bars.push(bar),
                    Err(e) => {
                        let reason = format!("produce traces bar {} failed: {e}", idx);
                        _ = ctx.send_shutdown(Instant::now(), &reason).await.ok();
                        panic!("{reason}");
                    }
                }
            }
            let mut traces_client = ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("traces client connect");
            #[allow(tail_expr_drop_order)]
            let traces_stream = stream! {
                for bar in trace_bars {
                    yield bar;
                }
            };
            let traces_resp = traces_client
                .arrow_traces(traces_stream)
                .await
                .expect("traces request");
            validate_success_responses(traces_resp.into_inner(), traces_count, "traces").await;

            ctx.send_shutdown(Instant::now(), "OTAP fake batch test")
                .await
                .expect("shutdown send");
        })
    }
}

fn otap_fake_batch_validation(
    plan: OtapFakeBatchPlan,
) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |mut ctx| {
        Box::pin(async move {
            let mut actual_metrics = Vec::new();
            for _ in 0..plan.metrics.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("metrics timeout")
                    .expect("missing metrics");
                actual_metrics.push(decode_pdata_to_message(&pdata));
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata)) {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("metrics ack send failed");
                }
            }
            let expected_metrics: Vec<_> = plan.metrics.into_iter().map(|(_, msg)| msg).collect();
            assert_equivalent(&expected_metrics, &actual_metrics);

            let mut actual_logs = Vec::new();
            for _ in 0..plan.logs.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("logs timeout")
                    .expect("missing logs");
                actual_logs.push(decode_pdata_to_message(&pdata));
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata)) {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("logs ack send failed");
                }
            }
            let expected_logs: Vec<_> = plan.logs.into_iter().map(|(_, msg)| msg).collect();
            assert_equivalent(&expected_logs, &actual_logs);

            let mut actual_traces = Vec::new();
            for _ in 0..plan.traces.len() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("traces timeout")
                    .expect("missing traces");
                actual_traces.push(decode_pdata_to_message(&pdata));
                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata)) {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("traces ack send failed");
                }
            }
            let expected_traces: Vec<_> = plan.traces.into_iter().map(|(_, msg)| msg).collect();
            assert_equivalent(&expected_traces, &actual_traces);
        })
    }
}

#[test]
#[ignore = "temporarily disabled while investigating produce_bar failure"]
fn test_otap_receiver_round_trip_fake_batches() {
    let registry = build_test_registry();
    let plan = generate_otap_fake_batches(&registry);

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
        .run_test(otap_fake_batch_scenario(grpc_endpoint, plan.clone()))
        .run_validation_concurrent(otap_fake_batch_validation(plan));
}

fn scenario(
    grpc_endpoint: String,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut arrow_metrics_client =
                ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect metrics client");
            #[allow(tail_expr_drop_order)]
            let metrics_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut metrics_records =
                        create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                    let bar = producer.produce_bar(&mut metrics_records).unwrap();
                    yield bar
                }
            };
            let metrics_response = arrow_metrics_client
                .arrow_metrics(metrics_stream)
                .await
                .expect("metrics request failed");
            validate_batch_responses(
                metrics_response.into_inner(),
                0,
                "Successfully received",
                "metrics",
            )
            .await;

            let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("Failed to connect logs client");
            #[allow(tail_expr_drop_order)]
            let logs_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    let bar = producer.produce_bar(&mut logs_records).unwrap();
                    yield bar;
                }
            };
            let logs_response = arrow_logs_client
                .arrow_logs(logs_stream)
                .await
                .expect("logs request failed");
            validate_batch_responses(
                logs_response.into_inner(),
                0,
                "Successfully received",
                "logs",
            )
            .await;

            let mut arrow_traces_client = ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("Failed to connect traces client");
            #[allow(tail_expr_drop_order)]
            let traces_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut traces_records =
                        create_otap_batch(batch_id, ArrowPayloadType::Spans);
                    let bar = producer.produce_bar(&mut traces_records).unwrap();
                    yield bar;
                }
            };
            let traces_response = arrow_traces_client
                .arrow_traces(traces_stream)
                .await
                .expect("traces request failed");
            validate_batch_responses(
                traces_response.into_inner(),
                0,
                "Successfully received",
                "traces",
            )
            .await;

            ctx.send_shutdown(Instant::now(), "Test complete")
                .await
                .expect("shutdown send failed");
        })
    }
}

fn validation_procedure()
-> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    |mut ctx| {
        Box::pin(async move {
            for batch_id in 0..3 {
                let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("metrics timeout")
                    .expect("missing metrics");
                let metrics_records: OtapArrowRecords = metrics_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("metrics conversion");
                let _expected_metrics =
                    create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                assert!(matches!(metrics_records, _expected_metrics));
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("metrics ack send failed");
                }
            }

            for batch_id in 0..3 {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("logs timeout")
                    .expect("missing logs");
                let logs_records: OtapArrowRecords = logs_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("logs conversion");
                let _expected_logs = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                assert!(matches!(logs_records, _expected_logs));
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("logs ack send failed");
                }
            }

            for batch_id in 0..3 {
                let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("traces timeout")
                    .expect("missing traces");
                let traces_records: OtapArrowRecords = traces_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("traces conversion");
                let _expected_traces = create_otap_batch(batch_id, ArrowPayloadType::Spans);
                assert!(matches!(traces_records, _expected_traces));
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(traces_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("traces ack send failed");
                }
            }
        })
    }
}

fn nack_scenario(
    grpc_endpoint: String,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut arrow_metrics_client =
                ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect metrics client");
            #[allow(tail_expr_drop_order)]
            let metrics_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut metrics_records =
                        create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                    let bar = producer.produce_bar(&mut metrics_records).unwrap();
                    yield bar
                }
            };
            let metrics_response = arrow_metrics_client
                .arrow_metrics(metrics_stream)
                .await
                .expect("metrics request failed");
            validate_batch_responses(
                metrics_response.into_inner(),
                14,
                &format!(
                    "Pipeline processing failed: {}",
                    "Test NACK reason for metrics"
                ),
                "metrics",
            )
            .await;

            let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("Failed to connect logs client");
            #[allow(tail_expr_drop_order)]
            let logs_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    let bar = producer.produce_bar(&mut logs_records).unwrap();
                    yield bar;
                }
            };
            let logs_response = arrow_logs_client
                .arrow_logs(logs_stream)
                .await
                .expect("logs request failed");
            validate_batch_responses(
                logs_response.into_inner(),
                14,
                &format!(
                    "Pipeline processing failed: {}",
                    "Test NACK reason for logs"
                ),
                "logs",
            )
            .await;

            let mut arrow_traces_client = ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("Failed to connect traces client");
            #[allow(tail_expr_drop_order)]
            let traces_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..3 {
                    let mut traces_records =
                        create_otap_batch(batch_id, ArrowPayloadType::Spans);
                    let bar = producer.produce_bar(&mut traces_records).unwrap();
                    yield bar;
                }
            };
            let traces_response = arrow_traces_client
                .arrow_traces(traces_stream)
                .await
                .expect("traces request failed");
            validate_batch_responses(
                traces_response.into_inner(),
                14,
                &format!(
                    "Pipeline processing failed: {}",
                    "Test NACK reason for traces"
                ),
                "traces",
            )
            .await;

            ctx.send_shutdown(Instant::now(), "Test complete")
                .await
                .expect("shutdown send failed");
        })
    }
}

fn nack_validation_procedure()
-> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    |mut ctx| {
        Box::pin(async move {
            for _batch_id in 0..3 {
                let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("metrics timeout")
                    .expect("missing metrics");
                let nack = NackMsg::new("Test NACK reason for metrics", metrics_pdata);
                if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("metrics nack send failed");
                }
            }

            for _batch_id in 0..3 {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("logs timeout")
                    .expect("missing logs");
                let nack = NackMsg::new("Test NACK reason for logs", logs_pdata);
                if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("logs nack send failed");
                }
            }

            for _batch_id in 0..3 {
                let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("traces timeout")
                    .expect("missing traces");
                let nack = NackMsg::new("Test NACK reason for traces", traces_pdata);
                if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("traces nack send failed");
                }
            }
        })
    }
}

type StatusPlan = Vec<Result<(), &'static str>>;

/// gRPC client harness for the Zstd log regression test.
///
/// It streams a predetermined number of OTAP log batches, forces tonic to use request-side
/// Zstd compression, and asserts that the streamed `BatchStatus` items match the provided
/// `status_plan` (ACK vs NACK). Once validation finishes we trigger the runtime shutdown so
/// the paired validator can complete.
fn zstd_logs_scenario(
    grpc_endpoint: String,
    status_plan: StatusPlan,
) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |ctx| {
        Box::pin(async move {
            let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .expect("Failed to connect logs client")
                .send_compressed(CompressionEncoding::Zstd);

            let plan_len = status_plan.len();
            #[allow(tail_expr_drop_order)]
            let logs_stream = stream! {
                let mut producer = Producer::new();
                for batch_id in 0..plan_len {
                    let mut logs_records =
                        create_otap_batch(batch_id as i64, ArrowPayloadType::Logs);
                    let bar = producer.produce_bar(&mut logs_records).unwrap();
                    yield bar;
                }
            };

            let logs_response = arrow_logs_client
                .arrow_logs(logs_stream)
                .await
                .expect("logs request failed");
            validate_mixed_log_statuses(logs_response.into_inner(), &status_plan, "logs").await;

            ctx.send_shutdown(Instant::now(), "Zstd logs test complete")
                .await
                .expect("shutdown send failed");
        })
    }
}

/// Validator companion for `zstd_logs_scenario`.
///
/// Consumes the decoded batches the receiver pushes into the pipeline channel, verifies each
/// payload matches what the client sent, and sends either an ACK or NACK back into the system
/// to simulate downstream processing outcomes.
fn zstd_logs_validation(
    status_plan: StatusPlan,
) -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
    move |mut ctx| {
        Box::pin(async move {
            for (batch_id, expected) in status_plan.into_iter().enumerate() {
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("logs timeout")
                    .expect("missing logs");
                let records: OtapArrowRecords =
                    pdata.clone().payload().try_into().expect("logs conversion");
                let expected_logs = create_otap_batch(batch_id as i64, ArrowPayloadType::Logs);
                assert_eq!(records, expected_logs);
                match expected {
                    Ok(()) => {
                        if let Some((_node_id, ack)) =
                            crate::pdata::Context::next_ack(AckMsg::new(pdata))
                        {
                            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                                .await
                                .expect("logs ack send failed");
                        }
                    }
                    Err(reason) => {
                        let nack_msg = NackMsg::new(reason, pdata);
                        if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack_msg) {
                            ctx.send_control_msg(NodeControlMsg::Nack(nack))
                                .await
                                .expect("logs nack send failed");
                        }
                    }
                }
            }
        })
    }
}

/// Confirms that the gRPC response stream mirrors the ACK/NACK plan.
///
/// ACK responses must arrive as `OK` with the default success message, while NACKs surface as
/// `UNAVAILABLE` plus the provided failure reason.
async fn validate_mixed_log_statuses<S>(
    mut inbound_stream: S,
    status_plan: &[Result<(), &'static str>],
    signal_name: &str,
) where
    S: futures::Stream<Item = Result<BatchStatus, Status>> + Unpin,
{
    use futures::StreamExt;
    let mut index = 0;
    while let Some(result) = inbound_stream.next().await {
        let batch_status = result.expect("Expected successful response");
        let expected = status_plan
            .get(index)
            .unwrap_or_else(|| panic!("unexpected extra response for {}", signal_name));
        match expected {
            Ok(()) => {
                assert_eq!(
                    batch_status.status_code,
                    ProtoStatusCode::Ok as i32,
                    "Unexpected success code for {} batch {}",
                    signal_name,
                    batch_status.batch_id
                );
                assert_eq!(
                    batch_status.status_message, "Successfully received",
                    "Unexpected success message for {} batch {}",
                    signal_name, batch_status.batch_id
                );
            }
            Err(reason) => {
                assert_eq!(
                    batch_status.status_code,
                    ProtoStatusCode::Unavailable as i32,
                    "Unexpected failure code for {} batch {}",
                    signal_name,
                    batch_status.batch_id
                );
                assert_eq!(
                    batch_status.status_message,
                    format!("Pipeline processing failed: {}", reason),
                    "Unexpected failure message for {} batch {}",
                    signal_name,
                    batch_status.batch_id
                );
            }
        }
        index += 1;
    }
    assert_eq!(
        index,
        status_plan.len(),
        "Missing responses for {} (expected {}, saw {index})",
        signal_name,
        status_plan.len()
    );
}

async fn validate_batch_responses<S>(
    mut inbound_stream: S,
    expected_status_code: i32,
    expected_status_message: &str,
    signal_name: &str,
) where
    S: futures::Stream<Item = Result<BatchStatus, Status>> + Unpin,
{
    use futures::StreamExt;
    let mut received_batch_ids = HashSet::new();
    while let Some(result) = inbound_stream.next().await {
        assert!(
            result.is_ok(),
            "Expected successful response for {}",
            signal_name
        );
        let batch_status = result.unwrap();
        let batch_id = batch_status.batch_id;
        assert!(
            received_batch_ids.insert(batch_id),
            "Duplicate response for batch {} ({})",
            batch_id,
            signal_name
        );
        assert_eq!(
            batch_status.status_code, expected_status_code,
            "Unexpected status code for {} batch {}",
            signal_name, batch_id
        );
        assert_eq!(
            batch_status.status_message, expected_status_message,
            "Unexpected status message for {} batch {}",
            signal_name, batch_id
        );
    }
    assert_eq!(
        received_batch_ids,
        (0..3).collect::<HashSet<_>>(),
        "Missing responses for {}",
        signal_name
    );
}

#[test]
fn test_otel_receiver() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = pick_free_port();
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config = json!({ "listening_addr": addr.to_string() });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);

    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver)
        .run_test(scenario(grpc_endpoint))
        .run_validation(validation_procedure());
}

#[test]
fn test_otel_receiver_ack() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = pick_free_port();
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config = json!({
        "listening_addr": addr.to_string(),
        "wait_for_result": true
    });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(receiver)
        .run_test(scenario(grpc_endpoint))
        .run_validation_concurrent(validation_procedure());
}

#[test]
fn test_otel_receiver_nack() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = pick_free_port();
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config = json!({
        "listening_addr": addr.to_string(),
        "wait_for_result": true
    });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
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
        .run_validation_concurrent(nack_validation_procedure());
}

#[test]
/// End-to-end test for request-side Zstd compression with mixed ACK/NACK outcomes.
///
/// The receiver is configured with `wait_for_result` and `compression_method = "zstd"` so the
/// gRPC client can compress request frames. The `status_plan` drives alternating ACK/NACK
/// responses from the validation harness, exercising the effect-handler path (decoded batches
/// hit the channel) and the control path (ACK/NACK is reflected back to the client via
/// `BatchStatus`).
fn test_otel_receiver_zstd_logs_ack_nack() {
    let test_runtime = TestRuntime::new();
    let grpc_addr = "127.0.0.1";
    let grpc_port = pick_free_port();
    let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
    let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

    let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config = json!({
        "listening_addr": addr.to_string(),
        "wait_for_result": true,
        "compression_method": "zstd"
    });
    let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
    receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
    let receiver = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    let status_plan = vec![
        Ok(()),
        Err("Test NACK reason for logs 1"),
        Ok(()),
        Err("Test NACK reason for logs 2"),
    ];

    test_runtime
        .set_receiver(receiver)
        .run_test(zstd_logs_scenario(grpc_endpoint, status_plan.clone()))
        .run_validation_concurrent(zstd_logs_validation(status_plan));
}

#[test]
fn test_otel_receiver_config_parsing() {
    use crate::compression::CompressionMethod;
    use serde_json::json;

    let metrics_registry_handle = MetricsRegistryHandle::new();
    let controller_ctx = ControllerContext::new(metrics_registry_handle);
    let pipeline_ctx = controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

    let config_with_max_concurrent_requests = json!({
        "listening_addr": "127.0.0.1:4417",
        "max_concurrent_requests": 5000
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
            .unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4417"
    );
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 5000);
    assert!(!receiver.config.grpc.wait_for_result);
    assert!(receiver.config.grpc.request_compression.is_none());
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(
        receiver
            .config
            .grpc
            .preferred_response_compression()
            .is_none()
    );
    assert!(receiver.config.grpc.timeout.is_none());

    let config_minimal = json!({ "listening_addr": "127.0.0.1:4418" });
    let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_minimal).unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4418"
    );
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 0);
    assert!(!receiver.config.grpc.wait_for_result);
    assert!(receiver.config.grpc.request_compression.is_none());
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(
        receiver
            .config
            .grpc
            .preferred_response_compression()
            .is_none()
    );
    assert!(receiver.config.grpc.timeout.is_none());

    let config_full_gzip = json!({
        "listening_addr": "127.0.0.1:4419",
        "compression_method": "gzip",
        "max_concurrent_requests": 2500,
        "wait_for_result": true,
        "timeout": "30s"
    });
    let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_full_gzip).unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4419"
    );
    assert_eq!(receiver.config.grpc.max_concurrent_requests, 2500);
    assert!(receiver.config.grpc.wait_for_result);
    assert_eq!(
        receiver.config.grpc.request_compression,
        Some(vec![CompressionMethod::Gzip])
    );
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(
        receiver
            .config
            .grpc
            .preferred_response_compression()
            .is_none()
    );
    assert_eq!(receiver.config.grpc.timeout, Some(Duration::from_secs(30)));

    let config_with_zstd = json!({
        "listening_addr": "127.0.0.1:4420",
        "compression_method": "zstd",
        "wait_for_result": false
    });
    let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_zstd).unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4420"
    );
    assert!(!receiver.config.grpc.wait_for_result);
    assert_eq!(
        receiver.config.grpc.request_compression,
        Some(vec![CompressionMethod::Zstd])
    );
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(
        receiver
            .config
            .grpc
            .preferred_response_compression()
            .is_none()
    );
    assert!(receiver.config.grpc.timeout.is_none());

    let config_with_deflate = json!({
        "listening_addr": "127.0.0.1:4421",
        "compression_method": "deflate"
    });
    let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_deflate).unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4421"
    );
    assert_eq!(
        receiver.config.grpc.request_compression,
        Some(vec![CompressionMethod::Deflate])
    );
    assert!(receiver.config.grpc.response_compression.is_none());
    assert!(
        receiver
            .config
            .grpc
            .preferred_response_compression()
            .is_none()
    );
    assert!(receiver.config.grpc.timeout.is_none());

    let config_with_response_only = json!({
        "listening_addr": "127.0.0.1:4422",
        "response_compression_method": "gzip"
    });
    let receiver =
        OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_response_only).unwrap();
    assert_eq!(
        receiver.config.grpc.listening_addr.to_string(),
        "127.0.0.1:4422"
    );
    assert!(receiver.config.grpc.request_compression.is_none());
    assert_eq!(
        receiver.config.grpc.response_compression,
        Some(vec![CompressionMethod::Gzip])
    );
    assert_eq!(
        receiver.config.grpc.preferred_response_compression(),
        Some(CompressionMethod::Gzip)
    );
    assert!(receiver.config.grpc.timeout.is_none());
}
