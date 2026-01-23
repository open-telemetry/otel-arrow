// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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

pub const ASSERT_VALID_EXPORTER_URN: &str = "urn:otel:assert_valid:exporter";

#[derive(Debug, Deserialize)]
struct AssertValidExporterConfig {
    /// When true, a failing equivalence check is considered expected.
    #[serde(default)]
    expect_failure: bool,
}

#[metric_set(name = "validation.assert.exporter.metrics")]
#[derive(Debug, Default, Clone)]
struct AssertValidExporterMetrics {
    /// Number of comparisons that did not match expectation.
    /// 0 -> not valid
    /// 1 -> valid
    valid: otap_df_telemetry::instrument::Gauge<u64>,
}

pub struct AssertValidExporter {
    config: AssertValidExporterConfig,
    control_msg: Vec<OtlpProtoMessage>,
    suv_msg: Vec<OtlpProtoMessage>,
    metrics: MetricSet<AssertValidExporterMetrics>,
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
    let metrics = pipeline_ctx.register_metrics::<AssertValidExporterMetrics>();
    Ok(ExporterWrapper::local(
        AssertValidExporter {
            config,
            left: Vec::new(),
            right: Vec::new(),
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

#[async_trait(?Send)]
impl Exporter<OtapPdata> for AssertValidExporter {
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
                    break;
                }
                Message::PData(pdata) => {
                    // if message read in is from SUV update suv vector
                    // if message read in is from control update control vector
                    // compare and update metric
                    // if we expect_failure -> suv pipeline contains processors that can alter the data then we use the inverted result of assert_equivlent
                    // otherwise use assert_equivelent 
                }
            }
        }

        Ok(TerminalState::default())
    }
}
