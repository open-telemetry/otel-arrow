// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka receiver fixtures for tests that construct the node outside a pipeline.

use super::receiver::KAFKA_RECEIVER_URN;
use otel_arrow_dfe_config::context::transport_policy::HeaderCapturePolicy;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::testing::single_node_test_context;
use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;

/// The compiler validates Kafka settings before collecting the arrival declaration.
pub(crate) fn receiver_node(capture: Option<HeaderCapturePolicy>) -> NodeUserConfig {
    let mut node = NodeUserConfig::new_receiver_config(KAFKA_RECEIVER_URN);
    node.config = serde_json::json!({
        "brokers": "unused:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["traces"]},
    });
    node.header_capture = capture;
    node
}

/// Compiles the single receiver's arrival binding without a downstream consumer.
pub(crate) fn receiver_context(
    ctx: PipelineContext,
    capture: Option<HeaderCapturePolicy>,
) -> PipelineContext {
    single_node_test_context(
        ctx,
        &OTAP_PIPELINE_FACTORY,
        "kafka-receiver",
        &receiver_node(capture),
    )
    .expect("Kafka receiver bindings")
}
