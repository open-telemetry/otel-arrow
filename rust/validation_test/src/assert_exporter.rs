// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter that buffers inputs from two ports and asserts their equivalence.
//!
//! NOTE: Multi-port exporters are not yet fully wired in the engine. The port
//! selection logic here is a placeholder that will be replaced once the engine
//! supplies port metadata on inbound messages.

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::assert_equivalent;
use otap_df_pdata::testing::round_trip::otap_to_otlp;
use otap_df_pdata::{OtapPayload, otlp::OtlpProtoBytes};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

pub const ASSERT_EXPORTER_URN: &str = "urn:otel:assert:exporter";

#[derive(Debug, Deserialize)]
struct AssertExporterConfig {
    /// When true, a failing equivalence check is considered expected.
    #[serde(default)]
    expect_failure: bool,
}

#[metric_set(name = "validation.assert.exporter.metrics")]
#[derive(Debug, Default, Clone)]
struct AssertExporterMetrics {
    /// Number of comparisons executed.
    #[metric(unit = "{comparison}")]
    comparisons: otap_df_telemetry::instrument::Counter<u64>,
    /// Number of comparisons that did not match expectation.
    #[metric(unit = "{comparison}")]
    mismatches: otap_df_telemetry::instrument::Counter<u64>,
}

pub struct AssertExporter {
    config: AssertExporterConfig,
    left: Vec<OtlpProtoMessage>,
    right: Vec<OtlpProtoMessage>,
    next_side: bool,
    metrics: MetricSet<AssertExporterMetrics>,
}

fn to_otlp(msg: OtapPdata) -> Option<OtlpProtoMessage> {
    let (_ctx, payload) = msg.into_parts();
    match payload {
        OtapPayload::OtlpBytes(bytes) => OtlpProtoMessage::try_from(bytes).ok(),
        OtapPayload::OtapArrowRecords(records) => Some(otap_to_otlp(&records)),
    }
}

pub fn create_assert_exporter(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &ExporterConfig,
) -> Result<ExporterWrapper<OtapPdata>, ConfigError> {
    let config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    let metrics = pipeline_ctx.register_metrics::<AssertExporterMetrics>();
    Ok(ExporterWrapper::local(
        AssertExporter {
            config,
            left: Vec::new(),
            right: Vec::new(),
            next_side: false,
            metrics,
        },
        node,
        node_config,
        exporter_config,
    ))
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static ASSERT_EXPORTER_FACTORY: ExporterFactory<OtapPdata> = ExporterFactory {
    name: ASSERT_EXPORTER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        create_assert_exporter(pipeline_ctx, node, node_config, exporter_config)
    },
};

impl AssertExporter {
    fn pick_side(&mut self, _msg: &Message<OtapPdata>) -> bool {
        // TODO: replace with real port-based routing once Message carries port info.
        let side = self.next_side;
        self.next_side = !self.next_side;
        side
    }

    fn compare(&mut self) {
        if self.left.is_empty() || self.right.is_empty() {
            return;
        }
        self.metrics.comparisons.inc();

        let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
            assert_equivalent(&self.left, &self.right);
        }));

        let comparison_failed = res.is_err();
        let expectation_met = if self.config.expect_failure {
            comparison_failed
        } else {
            !comparison_failed
        };

        if !expectation_met {
            self.metrics.mismatches.inc();
        }
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AssertExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        _effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    let _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    self.compare();
                    break;
                }
                Message::PData(pdata) => {
                    if let Some(proto) = to_otlp(pdata) {
                        if self.pick_side(&Message::PData(OtapPdata::new_todo_context(
                            OtapPayload::empty(proto.signal_type()),
                        ))) {
                            self.right.push(proto);
                        } else {
                            self.left.push(proto);
                        }
                        self.compare();
                    }
                }
            }
        }

        Ok(TerminalState::default())
    }
}
