// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::NodeId as NodeName;
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
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::assert_equivalent;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tokio::time::Duration;

/// URN that identifies the validation exporter within OTAP pipelines.
pub const VALIDATION_EXPORTER_URN: &str = "urn:otel:validation:exporter";

#[derive(Debug, Deserialize)]
struct ValidationExporterConfig {
    suv_input: NodeName,
    control_input: NodeName,
    /// When true, a failing equivalence check is considered expected.
    #[serde(default)]
    expect_failure: bool,
}

#[metric_set(name = "validation.exporter.metrics")]
#[derive(Debug, Default, Clone)]
struct ValidationExporterMetrics {
    /// Number of comparisons that did not match expectation
    #[metric(name = "comparison.failed", unit = "{comparison}")]
    failed_comparisons: otap_df_telemetry::instrument::Counter<u64>,
    /// Number of comparisons that did match expectation
    #[metric(name = "comparison.passed", unit = "{comparison}")]
    passed_comparisons: otap_df_telemetry::instrument::Counter<u64>,
    /// The value of the last comparison result
    /// 0 -> not valid
    /// 1 -> valid
    #[metric(unit = "{input}")]
    valid: otap_df_telemetry::instrument::Gauge<u64>,
}

/// Exporter that compares control and suv pipeline outputs and reports equivalence metrics.
pub struct ValidationExporter {
    config: ValidationExporterConfig,
    control_msgs: Vec<OtlpProtoMessage>,
    suv_msgs: Vec<OtlpProtoMessage>,
    metrics: MetricSet<ValidationExporterMetrics>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
/// Distributed-slice factory that registers the validation exporter with the engine.
pub static VALIDATION_EXPORTER_FACTORY: ExporterFactory<OtapPdata> = ExporterFactory {
    name: VALIDATION_EXPORTER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            ValidationExporter::from_config(pipeline_ctx, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl ValidationExporter {
    /// compares control and suv msgs and updates the metrics
    fn compare_and_record(&mut self) {
        let equiv = std::panic::catch_unwind(AssertUnwindSafe(|| {
            assert_equivalent(&self.control_msgs, &self.suv_msgs)
        }))
        .is_ok();

        // we do this if we expect the data to be altered due
        // to a transformative processor in the pipeline
        // passed is equal to !equiv if expect_failure is true
        // passed is equal to equiv if expect_failure is false

        if equiv {
            self.metrics.passed_comparisons.add(1);
        } else {
            self.metrics.failed_comparisons.add(1);
        }
        let passed = equiv ^ self.config.expect_failure;
        self.metrics.valid.set(passed as u64);
    }

    /// Build a new exporter instance from user configuration embedded in the pipeline.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<ValidationExporterMetrics>();
        let config: ValidationExporterConfig =
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?;
        Ok(Self {
            config,
            metrics,
            control_msgs: Vec::new(),
            suv_msgs: Vec::new(),
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ValidationExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    return Ok(TerminalState::new(deadline, [self.metrics]));
                }
                Message::PData(pdata) => {
                    let (context, payload) = pdata.into_parts();
                    let source_node = context.source_node();
                    let msg = OtlpProtoBytes::try_from(payload)
                        .ok()
                        .and_then(|bytes| OtlpProtoMessage::try_from(bytes).ok());

                    if let Some(msg) = msg
                        && let Some(node_name) = source_node
                    {
                        match node_name {
                            // match node name and update the vec of msgs and compare
                            name if name == self.config.suv_input => {
                                self.suv_msgs.push(msg);
                                self.compare_and_record();
                            }
                            name if name == self.config.control_input => {
                                self.control_msgs.push(msg);
                                self.compare_and_record();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
