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
use otap_df_config::NodeId as NodeName;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::assert_equivalent;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

pub const ASSERT_VALID_EXPORTER_URN: &str = "urn:otel:assert_valid:exporter";

#[derive(Debug, Deserialize)]
struct AssertValidExporterConfig {
    suv_node: NodeName,
    control_node: NodeName,
    /// When true, a failing equivalence check is considered expected.
    #[serde(default)]
    expect_failure: bool,
}

#[metric_set(name = "validation.assert.exporter.metrics")]
#[derive(Debug, Default, Clone)]
struct AssertValidExporterMetrics {
    /// Number of comparisons that did not match expectation
    #[metric(name = "comparison.failed", unit = "{comparison}")]
    failed_comparisons: otap_df_telemetry::instrument::Counter<u64>,
    /// Number of comparisons that did match expectation
    #[metric(name = "comparison.passed", unit = "{comparison}")]
    passed_comparisons: otap_df_telemetry::instrument::Counter<u64>,
    /// The value of the last comparison result
    /// 0 -> not valid
    /// 1 -> valid
    valid: otap_df_telemetry::instrument::Gauge<u64>,
}

pub struct AssertValidExporter {
    config: AssertValidExporterConfig,
    control_msgs: Vec<OtlpProtoMessage>,
    suv_msgs: Vec<OtlpProtoMessage>,
    metrics: MetricSet<AssertValidExporterMetrics>,
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
            control_msgs: Vec::new(),
            suv_msgs: Vec::new(),
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
    name: ASSERT_VALID_EXPORTER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        create_assert_exporter(pipeline_ctx, node, node_config, exporter_config)
    },
};

impl AssertValidExporter {

    fn new() -> Self {
        Self {
            config,
            control_msgs: Vec::new(),
            suv_msgs: Vec::new(),
            metrics
        }
    }
    }
    /// compares control and suv msgs and updates the metrics
    fn assert(&mut self) {
        let equiv = std::panic::catch_unwind(AssertUnwindSafe(|| {
            assert_equivalent(&self.control_msgs, &self.suv_msgs)
        }))
        .is_ok();

        let passed = if self.config.expect_failure {
            !equiv
        } else {
            equiv
        };

        if passed {
            self.metrics.passed_comparisons.add(1);
            self.metrics.valid.set(1);
        } else {
            self.metrics.failed_comparisons.add(1);
            self.metrics.valid.set(0);
        }
    }
}

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
                    let (context, payload) = pdata.into_parts();
                    let source_node: Option<NodeName> = None;
                    let msg = OtlpProtoBytes::try_from(payload)
                        .ok()
                        .and_then(|bytes| OtlpProtoMessage::try_from(bytes).ok());

                    if let Some(msg) = msg && let Some(node_name) = source_node {
                        match node_name {
                            // match node name and update the vec of msgs and compare
                            name if name == self.config.suv_node => {
                                self.suv_msgs.push(msg);
                                self.assert();
                            }
                            name if name == self.config.control_node => {
                                self.control_msgs.push(msg);
                                self.assert();
                            }
                            _ => {

                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(TerminalState::default())
    }
}
