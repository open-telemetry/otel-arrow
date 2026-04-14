// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Declarative test framework for temporal reaggregation processor ack/nack
//! scenarios.

use std::sync::Arc;

use otap_df_config::SignalType;
pub(super) use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::Interests;
use otap_df_engine::context::{ControllerContext, PipelineContext};
use otap_df_engine::control::{
    AckMsg, CallData, NackMsg, NodeControlMsg, PipelineCompletionMsg, RouteData, UnwindData,
    pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
};
use otap_df_engine::message::Message;
pub(super) use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::testing::node::test_node;
pub(super) use otap_df_engine::testing::processor::TestRuntime;
use otap_df_otap::pdata::OtapPdata;
use otap_df_otap::testing::{TestCallData, next_ack, next_nack};
use otap_df_pdata::OtapPayload;
pub(super) use otap_df_pdata::otap::OtapArrowRecords;
pub(super) use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
use otap_df_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
pub(super) use otap_df_pdata::proto::opentelemetry::metrics::v1::exponential_histogram_data_point::Buckets;
pub(super) use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
    HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    Summary, SummaryDataPoint,
};
pub(super) use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::testing::equiv::assert_equivalent;
pub(super) use otap_df_pdata::testing::round_trip::{
    otap_to_otlp, otlp_message_to_bytes, otlp_to_otap,
};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use serde_json;

use super::{TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY, TEMPORAL_REAGGREGATION_PROCESSOR_URN};

/// An action in a declarative test scenario.
#[allow(clippy::large_enum_variant)]
pub(super) enum Action {
    /// Send pdata with optional ack/nack subscriber interest.
    /// `Interests::empty()` skips subscriber tracking.
    SendPdata {
        interests: Interests,
        payload: OtapPayload,
    },

    /// Send a message to the processor
    SendControl(NodeControlMsg<OtapPdata>),

    /// Fire the next pending wakeup from the processor's local scheduler.
    FireWakeup,

    /// Drain one output from the processor and apply actions to it.
    /// Use multiple `DrainPdata` in sequence to drain multiple outputs.
    DrainPdata { actions: Vec<PdataAction> },

    /// Asserts that there is no pdata to drain
    AssertNoPdata,

    /// Assert on upstream ack/nack messages received via the pipeline
    /// completion channel.
    AssertUpstream(UpstreamExpectation),
}

/// An action to perform on a single drained output.
#[allow(variant_size_differences)]
#[allow(clippy::large_enum_variant)]
pub(super) enum PdataAction {
    /// Assert that the drained data is equivalent to this data
    AssertEquivalent(OtapPayload),
    /// Assert whether this output has subscriber interest.
    AssertSubscribers(bool),
    /// Run a custom assertion on the drained output.
    AssertCustom(Box<dyn FnOnce(&OtapPdata)>),
    /// Nack this output with the given reason.
    Nack(&'static str),
    /// Ack this output.
    Ack,
}

/// Assertion on the cumulative upstream ack/nack counts.
pub(super) enum UpstreamExpectation {
    /// Expect exactly N upstream acks received so far.
    AckCount(usize),
    /// Expect exactly N upstream nacks received so far.
    NackCount(usize),
}

/// Run a declarative test scenario against the temporal reaggregation processor.
pub(super) fn run_test(config: serde_json::Value, actions: Vec<Action>) {
    let (rt, proc) = try_create_processor(config).expect("valid config");
    rt.set_processor(proc)
        .run_test(move |mut ctx| async move {
            let (runtime_ctrl_tx, mut runtime_ctrl_rx) = runtime_ctrl_msg_channel(8);
            let (pipeline_completion_tx, mut pipeline_completion_rx) =
                pipeline_completion_msg_channel(8);
            ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
            ctx.set_pipeline_completion_sender(pipeline_completion_tx);

            let mut input_idx: u64 = 0;
            let mut output_buffer: Vec<OtapPdata> = Vec::new();
            let mut upstream_ack_count: usize = 0;
            let mut upstream_nack_count: usize = 0;

            for action in actions {
                match action {
                    Action::SendPdata { interests, payload } => {
                        let mut pdata = OtapPdata::new_default(payload);
                        if !interests.is_empty() {
                            pdata = pdata.test_subscribe_to(
                                interests,
                                TestCallData::new_with(input_idx, 0).into(),
                                1,
                            );
                        }
                        ctx.process(Message::PData(pdata))
                            .await
                            .expect("process pdata");
                        input_idx += 1;
                    }
                    Action::DrainPdata { actions } => {
                        if output_buffer.is_empty() {
                            output_buffer = ctx.drain_pdata().await;
                        }
                        assert!(
                            !output_buffer.is_empty(),
                            "DrainPdata: expected at least one output, got none"
                        );
                        let output = output_buffer.remove(0);

                        for pdata_action in actions {
                            match pdata_action {
                                PdataAction::AssertSubscribers(expected) => {
                                    assert_eq!(
                                        output.has_subscribers(),
                                        expected,
                                        "has_subscribers mismatch"
                                    );
                                }
                                PdataAction::Nack(reason) => {
                                    let (_, nack_msg) =
                                        next_nack(NackMsg::new(reason.to_string(), output.clone()))
                                            .expect("output should have nack subscribers");
                                    ctx.process(Message::nack_ctrl_msg(nack_msg))
                                        .await
                                        .expect("process nack");
                                }
                                PdataAction::Ack => {
                                    let (_, ack_msg) = next_ack(AckMsg::new(output.clone()))
                                        .expect("output should have ack subscribers");
                                    ctx.process(Message::ack_ctrl_msg(ack_msg))
                                        .await
                                        .expect("process ack");
                                }
                                PdataAction::AssertCustom(f) => {
                                    f(&output);
                                }
                                PdataAction::AssertEquivalent(expected_payload) => {
                                    let actual = payload_to_otlp(output.payload_ref());
                                    let expected = payload_to_otlp(&expected_payload);
                                    assert_equivalent(&[actual], &[expected]);
                                }
                            }
                        }

                        // Drain control channels to keep them clear
                        drain_upstream(
                            &mut runtime_ctrl_rx,
                            &mut pipeline_completion_rx,
                            &mut upstream_ack_count,
                            &mut upstream_nack_count,
                        );
                    }
                    Action::SendControl(ctrl) => {
                        ctx.process(Message::Control(ctrl))
                            .await
                            .expect("process control message");
                    }
                    Action::FireWakeup => {
                        let _ = ctx.fire_wakeup().await.expect("fire wakeup");
                    }
                    Action::AssertUpstream(expectation) => {
                        // Drain any pending upstream messages first
                        drain_upstream(
                            &mut runtime_ctrl_rx,
                            &mut pipeline_completion_rx,
                            &mut upstream_ack_count,
                            &mut upstream_nack_count,
                        );

                        match expectation {
                            UpstreamExpectation::AckCount(expected) => {
                                assert_eq!(
                                    upstream_ack_count, expected,
                                    "upstream ack count mismatch"
                                );
                            }
                            UpstreamExpectation::NackCount(expected) => {
                                assert_eq!(
                                    upstream_nack_count, expected,
                                    "upstream nack count mismatch"
                                );
                            }
                        }
                    }
                    Action::AssertNoPdata => {
                        assert!(output_buffer.is_empty());
                        assert!(ctx.drain_pdata().await.is_empty());
                    }
                }
            }
        })
        .validate(|_ctx| async {});
}

/// Drain the runtime control and pipeline completion channels, accumulating
/// upstream ack/nack counts.
pub(super) fn drain_upstream(
    runtime_ctrl_rx: &mut otap_df_engine::control::RuntimeCtrlMsgReceiver<OtapPdata>,
    pipeline_completion_rx: &mut otap_df_engine::control::PipelineCompletionMsgReceiver<OtapPdata>,
    ack_count: &mut usize,
    nack_count: &mut usize,
) {
    // Drain runtime control (e.g. DelayData) -- just consume for now
    while runtime_ctrl_rx.try_recv().is_ok() {}

    // Drain pipeline completion (upstream ack/nack delivery)
    loop {
        match pipeline_completion_rx.try_recv() {
            Ok(PipelineCompletionMsg::DeliverAck { ack }) => {
                if next_ack(ack).is_some() {
                    *ack_count += 1;
                }
            }
            Ok(PipelineCompletionMsg::DeliverNack { nack }) => {
                if next_nack(nack).is_some() {
                    *nack_count += 1;
                }
            }
            Err(_) => break,
        }
    }
}

pub(super) fn try_create_processor(
    config: serde_json::Value,
) -> Result<(TestRuntime<OtapPdata>, ProcessorWrapper<OtapPdata>), ConfigError> {
    let pipeline_ctx = create_test_pipeline_context();
    let rt: TestRuntime<OtapPdata> = TestRuntime::new();
    let node = test_node("temporal-reaggregation-config-test");

    let mut node_config =
        NodeUserConfig::new_processor_config(TEMPORAL_REAGGREGATION_PROCESSOR_URN);
    node_config.config = config;

    (TEMPORAL_REAGGREGATION_PROCESSOR_FACTORY.create)(
        pipeline_ctx,
        node,
        Arc::new(node_config),
        rt.config(),
    )
    .map(|proc| (rt, proc))
}

/// Wrap [`OtapArrowRecords`] in an [`OtapPdata`].
pub(super) fn make_pdata(records: OtapArrowRecords) -> OtapPdata {
    OtapPdata::new_default(OtapPayload::OtapArrowRecords(records))
}

/// Convert OTLP [`MetricsData`] into an [`OtapPdata`] via OTAP encoding.
pub(super) fn make_otlp_pdata(metrics_data: MetricsData) -> OtapPdata {
    let otap_records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
        metrics_data,
    ));
    OtapPdata::new_default(OtapPayload::OtapArrowRecords(otap_records))
}

pub(super) fn create_traces_payload() -> OtapPayload {
    let mut datagen = otap_df_pdata::testing::fixtures::DataGenerator::new(3);
    let traces_data = datagen.generate_traces();
    otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Traces(traces_data)).into()
}

pub(super) fn create_logs_payload() -> OtapPayload {
    let mut datagen = otap_df_pdata::testing::fixtures::DataGenerator::new(3);
    let logs_data = datagen.generate_logs();
    otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Logs(logs_data)).into()
}

pub(super) fn create_test_pipeline_context() -> PipelineContext {
    let telemetry_registry = TelemetryRegistryHandle::new();
    let controller_ctx = ControllerContext::new(telemetry_registry);
    controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
}

/// Convert any [`OtapPayload`] variant into an [`OtlpProtoMessage`] for
/// equivalence comparison.
fn payload_to_otlp(payload: &OtapPayload) -> otap_df_pdata::proto::OtlpProtoMessage {
    match payload {
        OtapPayload::OtapArrowRecords(records) => otap_to_otlp(records),
        OtapPayload::OtlpBytes(bytes) => {
            otap_df_pdata::testing::round_trip::otlp_bytes_to_message(bytes.clone())
        }
    }
}

/// Assert that the processor output is semantically equivalent to the
/// expected set of [`OtapArrowRecords`] batches combined.
pub(super) fn assert_output_equivalent(output: &OtapPdata, expected: &[OtapArrowRecords]) {
    let actual = match output.payload_ref() {
        OtapPayload::OtapArrowRecords(r) => r,
        _ => panic!("expected OtapArrowRecords payload"),
    };
    let expected_msgs: Vec<_> = expected.iter().map(otap_to_otlp).collect();
    assert_equivalent(&[otap_to_otlp(actual)], &expected_msgs);
}

/// Assert that the processor output is semantically equivalent to the
/// expected OTLP [`MetricsData`].
pub(super) fn assert_output_otlp_equivalent(output: &OtapPdata, expected: MetricsData) {
    let actual = match output.payload_ref() {
        OtapPayload::OtapArrowRecords(r) => r,
        _ => panic!("expected OtapArrowRecords payload"),
    };
    assert_equivalent(
        &[otap_to_otlp(actual)],
        &[otap_df_pdata::proto::OtlpProtoMessage::Metrics(expected)],
    );
}

/// Encode OTLP [`MetricsData`] into serialized protobuf bytes and wrap
/// as an [`OtapPdata`] with an [`OtlpBytes`] payload.
pub(super) fn make_otlp_bytes_pdata(metrics_data: MetricsData) -> OtapPdata {
    let msg = otap_df_pdata::proto::OtlpProtoMessage::Metrics(metrics_data);
    let otlp_bytes = otlp_message_to_bytes(&msg);
    OtapPdata::new_default(OtapPayload::OtlpBytes(otlp_bytes))
}

/// Build an OTLP [`MetricsData`] with `n` unique gauge metrics, each with
/// a single data point. All metrics share one resource and one scope.
pub(super) fn make_gauge(name: &str, value: f64) -> Metric {
    Metric::build()
        .name(name)
        .data_gauge(Gauge::new(vec![
            NumberDataPoint::build()
                .time_unix_nano(1000u64)
                .value_double(value)
                .finish(),
        ]))
        .finish()
}

pub(super) fn make_sum(
    name: &str,
    aggregatable: bool,
    value: i64,
    exemplars: Vec<Exemplar>,
) -> Metric {
    let temporality = if aggregatable {
        AggregationTemporality::Cumulative
    } else {
        AggregationTemporality::Delta
    };
    Metric::build()
        .name(name)
        .data_sum(Sum::new(
            temporality,
            true,
            vec![
                NumberDataPoint::build()
                    .time_unix_nano(1000u64)
                    .value_int(value)
                    .exemplars(exemplars)
                    .finish(),
            ],
        ))
        .finish()
}

pub(super) fn make_histogram(name: &str, aggregatable: bool, exemplars: Vec<Exemplar>) -> Metric {
    let temporality = if aggregatable {
        AggregationTemporality::Cumulative
    } else {
        AggregationTemporality::Delta
    };
    Metric::build()
        .name(name)
        .data_histogram(Histogram::new(
            temporality,
            vec![
                HistogramDataPoint::build()
                    .time_unix_nano(1000u64)
                    .count(10u64)
                    .sum(100.0f64)
                    .bucket_counts(vec![2, 3, 5])
                    .explicit_bounds(vec![10.0, 50.0])
                    .exemplars(exemplars)
                    .finish(),
            ],
        ))
        .finish()
}

pub(super) fn make_exp_histogram(
    name: &str,
    aggregatable: bool,
    exemplars: Vec<Exemplar>,
) -> Metric {
    let temporality = if aggregatable {
        AggregationTemporality::Cumulative
    } else {
        AggregationTemporality::Delta
    };
    Metric::build()
        .name(name)
        .data_exponential_histogram(ExponentialHistogram::new(
            temporality,
            vec![
                ExponentialHistogramDataPoint::build()
                    .time_unix_nano(1000u64)
                    .count(5u64)
                    .scale(2i32)
                    .zero_count(1u64)
                    .positive(Buckets::new(0, vec![1, 2]))
                    .negative(Buckets::new(0, vec![1, 1]))
                    .exemplars(exemplars)
                    .finish(),
            ],
        ))
        .finish()
}

pub(super) fn make_n_gauge_metrics(n: usize) -> MetricsData {
    make_n_gauge_metrics_with_offset(n, 0)
}

/// Build an OTLP [`MetricsData`] with `n` unique gauge metrics starting from
/// `offset`. Metric names are `metric_{offset}`, `metric_{offset+1}`, etc.
pub(super) fn make_n_gauge_metrics_with_offset(n: usize, offset: usize) -> MetricsData {
    let metrics: Vec<_> = (offset..offset + n)
        .map(|i| {
            Metric::build()
                .name(format!("metric_{i}"))
                .data_gauge(Gauge::new(vec![
                    NumberDataPoint::build()
                        .time_unix_nano(1000u64)
                        .value_double(i as f64)
                        .finish(),
                ]))
                .finish()
        })
        .collect();
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().finish(),
            metrics,
        )],
    )])
}

/// Build a mixed metrics batch with both aggregatable (gauge) and
/// non-aggregatable (delta sum) metrics to trigger SomeAggregations.
pub(super) fn make_mixed_metrics() -> MetricsData {
    let aggregatable = make_gauge("cpu.gauge", 42.0);
    let passthrough = make_sum("requests.delta", false, 50, vec![]);
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build().finish(),
            vec![aggregatable, passthrough],
        )],
    )])
}

/// Convert OTLP [`MetricsData`] into an [`OtapPayload`] via OTAP encoding.
pub(super) fn make_otap_payload_from_metrics(metrics_data: MetricsData) -> OtapPayload {
    let records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
        metrics_data,
    ));
    OtapPayload::OtapArrowRecords(records)
}

/// Convert OTLP [`MetricsData`] into an [`OtapPayload`] via OTAP encoding.
pub(super) fn make_otlp_payload_from_metrics(metrics_data: MetricsData) -> OtapPayload {
    let msg = otap_df_pdata::proto::OtlpProtoMessage::Metrics(metrics_data);
    let otlp_bytes = otlp_message_to_bytes(&msg);
    OtapPayload::OtlpBytes(otlp_bytes)
}

/// Build a mixed metrics OtapPayload (aggregatable + non-aggregatable).
pub(super) fn make_mixed_metrics_payload() -> OtapPayload {
    let records = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Metrics(
        make_mixed_metrics(),
    ));
    OtapPayload::OtapArrowRecords(records)
}

/// Assert that the total number of metrics in the processor output matches
/// `expected`. Works with both OTAP and OTLP payload variants.
pub(super) fn assert_output_metric_count(output: &OtapPdata, expected: usize) {
    let otlp = payload_to_otlp(output.payload_ref());
    let md = match otlp {
        otap_df_pdata::proto::OtlpProtoMessage::Metrics(md) => md,
        other => panic!("expected Metrics, got {other:?}"),
    };
    let count: usize = md
        .resource_metrics
        .iter()
        .flat_map(|rm| &rm.scope_metrics)
        .map(|sm| sm.metrics.len())
        .sum();
    assert_eq!(count, expected, "metric count mismatch");
}

/// Helper to build a `ResourceMetrics` with `resource: None`.
pub(super) fn resource_metrics_without_resource(
    scope_metrics: Vec<ScopeMetrics>,
) -> ResourceMetrics {
    ResourceMetrics {
        resource: None,
        scope_metrics,
        schema_url: String::new(),
    }
}

/// Helper to build a `ScopeMetrics` with `scope: None`.
pub(super) fn scope_metrics_without_scope(metrics: Vec<Metric>) -> ScopeMetrics {
    ScopeMetrics {
        scope: None,
        metrics,
        schema_url: String::new(),
    }
}

/// Build an [`AckMsg`] with the given calldata, bypassing the normal
/// subscriber-unwinding path. Used to test invalid-calldata handling.
pub(super) fn ack_with_calldata(calldata: CallData) -> AckMsg<OtapPdata> {
    AckMsg {
        accepted: Box::new(OtapPdata::new_default(OtapPayload::empty(
            SignalType::Metrics,
        ))),
        unwind: UnwindData {
            route: RouteData {
                calldata,
                entry_time_ns: 0,
                output_port_index: 0,
            },
            return_time_ns: 0,
        },
    }
}

/// Build a [`NackMsg`] with the given calldata, bypassing the normal
/// subscriber-unwinding path. Used to test invalid-calldata handling.
pub(super) fn nack_with_calldata(calldata: CallData, reason: &str) -> NackMsg<OtapPdata> {
    NackMsg {
        reason: reason.to_string(),
        refused: Box::new(OtapPdata::new_default(OtapPayload::empty(
            SignalType::Metrics,
        ))),
        unwind: UnwindData {
            route: RouteData {
                calldata,
                entry_time_ns: 0,
                output_port_index: 0,
            },
            return_time_ns: 0,
        },
        permanent: false,
    }
}
