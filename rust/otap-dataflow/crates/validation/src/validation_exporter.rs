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
use crate::checks::{check_min_batch_size, check_signal_drop};
use crate::ValidationKind;
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
    /// Validation rules to run. Defaults to a single equivalence check.
    #[serde(default = "ValidationExporterConfig::default_validations")]
    validations: Vec<ValidationKind>,
}

impl ValidationExporterConfig {
    fn default_validations() -> Vec<ValidationKind> {
        vec![ValidationKind::Equivalence]
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
    /// Run the configured validations and update metrics.
    fn evaluate_and_record(&mut self) {
        let mut valid = true;
        for validate in &self.config.validations {
            valid &= self.evaluate(validate);
        }

        if valid {
            self.metrics.passed_checks.add(1);
        } else {
            self.metrics.failed_checks.add(1);
        }
        self.metrics.valid.set(valid as u64);
    }

    /// Evaluate a single rule; always returns a pass/fail boolean.
    fn evaluate(&self, kind: &ValidationKind) -> bool {
        match kind {
            ValidationKind::Equivalence => {
                let equiv = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    assert_equivalent(&self.control_msgs, &self.suv_msgs)
                }))
                .is_ok();
                equiv
            }
            ValidationKind::SignalDrop => {
                check_signal_drop(&self.control_msgs, &self.suv_msgs)
            }
            ValidationKind::Batch {
                min_batch_size,
            } => {
                check_min_batch_size(&self.suv_msgs, *min_batch_size)
            }
            ValidationKind::Attributes {
                config,
            } => {
                config.check(&self.suv_msgs)
            }
        }
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
                                self.evaluate_and_record();
                            }
                            name if name == self.config.control_input => {
                                self.control_msgs.push(msg);
                                self.evaluate_and_record();
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
