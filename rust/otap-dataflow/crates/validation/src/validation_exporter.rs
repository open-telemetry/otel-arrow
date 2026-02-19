// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter used in validation pipelines that compares control and
//! system-under-validation outputs and records pass/fail metrics.

use crate::ValidationInstructions;
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
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_error;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// URN that identifies the validation exporter within OTAP pipelines.
pub const VALIDATION_EXPORTER_URN: &str = "urn:otel:validation:exporter";

#[derive(Debug, Deserialize)]
struct ValidationExporterConfig {
    suv_input: NodeName,
    control_input: NodeName,
    /// Validation rules to run. Defaults to a single equivalence check.
    #[serde(default = "ValidationExporterConfig::default_validations")]
    validations: Vec<ValidationInstructions>,
}

impl ValidationExporterConfig {
    fn default_validations() -> Vec<ValidationInstructions> {
        vec![ValidationInstructions::Equivalence]
    }
}

#[metric_set(name = "validation.exporter.metrics")]
#[derive(Debug, Default, Clone)]
struct ValidationExporterMetrics {
    /// Number of validation checks that did not match expectation
    #[metric(name = "check.failed", unit = "{check}")]
    failed_checks: otap_df_telemetry::instrument::Counter<u64>,
    /// Number of validation checks that did match expectation
    #[metric(name = "check.passed", unit = "{check}")]
    passed_checks: otap_df_telemetry::instrument::Counter<u64>,
    /// The value of the last comparison result
    /// 0 -> not valid
    /// 1 -> valid
    #[metric(unit = "{input}")]
    valid: otap_df_telemetry::instrument::Gauge<u64>,
}

/// Exporter that compares control and suv pipeline outputs and reports equivalence metrics.
pub struct ValidationExporter {
    suv_index: usize,
    control_index: usize,
    validations: Vec<ValidationInstructions>,
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<ValidationExporterConfig>,
};

impl ValidationExporter {
    /// Run the configured validations and update metrics.
    fn validate_and_record(&mut self, received_suv_msg: OtlpProtoMessage, time_elapsed: Duration) {
        let mut valid = true;
        for validate in &self.validations {
            valid &= validate.validate(
                &self.control_msgs,
                &self.suv_msgs,
                &received_suv_msg,
                &time_elapsed,
            );
        }

        if valid {
            self.metrics.passed_checks.add(1);
        } else {
            self.metrics.failed_checks.add(1);
        }
        self.metrics.valid.set(valid as u64);
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
        let suv_node = pipeline_ctx
            .node_by_name(&config.suv_input)
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: format!("unknown node name for suv_input: {}", config.suv_input),
            })?;
        let control_node = pipeline_ctx
            .node_by_name(&config.control_input)
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: format!(
                    "unknown node name for control_input: {}",
                    config.control_input
                ),
            })?;
        Ok(Self {
            suv_index: suv_node.index,
            control_index: control_node.index,
            validations: config.validations,
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
        let mut time = Instant::now();
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
                    let time_elapsed = time.elapsed();
                    let (context, payload) = pdata.into_parts();
                    let source_node = context.source_node();
                    let msg = OtlpProtoBytes::try_from(payload)
                        .ok()
                        .and_then(|bytes| OtlpProtoMessage::try_from(bytes).ok());

                    if let Some(msg) = msg
                        && let Some(node_index) = source_node
                    {
                        if node_index == self.suv_index {
                            self.suv_msgs.push(msg.clone());
                            self.validate_and_record(msg, time_elapsed);
                            time = Instant::now();
                        } else if node_index == self.control_index {
                            self.control_msgs.push(msg);
                        }
                    } else {
                        otel_error!("validation.missing.source");
                    }
                }
                _ => {}
            }
        }
    }
}
