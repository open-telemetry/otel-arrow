// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic exporter.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::TopicName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::topic::TopicQueueOnFullPolicy;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::topic::{PublishOutcome, TopicHandle};
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// URN for the topic exporter.
pub const TOPIC_EXPORTER_URN: &str = "urn:otel:topic:exporter";

/// Telemetry metrics for the topic exporter.
#[metric_set(name = "topic.exporter.metrics")]
#[derive(Debug, Default, Clone)]
pub struct TopicExporterMetrics {
    /// Number of messages published to the topic.
    #[metric(unit = "{item}")]
    pub published_messages: Counter<u64>,
    /// Number of messages dropped due to queue full policy.
    #[metric(unit = "{item}")]
    pub dropped_messages_on_full: Counter<u64>,
}

/// Topic exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicExporterConfig {
    /// Topic name to publish to.
    pub topic: TopicName,
    /// Optional local override for publish behavior when topic queue is full.
    /// If omitted, runtime falls back to the topic declaration policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_on_full: Option<TopicQueueOnFullPolicy>,
}

/// Exporter for topic publishing.
pub struct TopicExporter {
    topic: TopicHandle<OtapPdata>,
    queue_on_full: TopicQueueOnFullPolicy,
    metrics: MetricSet<TopicExporterMetrics>,
}

/// Declares the topic exporter as a local exporter factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static TOPIC_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: TOPIC_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        let config = TopicExporter::parse_config(&node_config.config)?;
        let queue_on_full = config
            .queue_on_full
            .clone()
            .unwrap_or(TopicQueueOnFullPolicy::Block);
        let topic_set = pipeline
            .topic_set::<OtapPdata>()
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: "Topic set is not available in pipeline context".to_owned(),
            })?;
        let topic = topic_set
            .get(config.topic.as_ref())
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: format!(
                    "Unknown topic `{}` for topic exporter (pipeline `{}`/`{}`)",
                    config.topic,
                    pipeline.pipeline_group_id(),
                    pipeline.pipeline_id(),
                ),
            })?;
        let metrics = pipeline.register_metrics::<TopicExporterMetrics>();
        Ok(ExporterWrapper::local(
            TopicExporter {
                topic,
                queue_on_full,
                metrics,
            },
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |config| TopicExporter::parse_config(config).map(|_| ()),
};

impl TopicExporter {
    /// Parses and validates topic exporter configuration.
    pub fn parse_config(config: &Value) -> Result<TopicExporterConfig, ConfigError> {
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse topic exporter config: {e}"),
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for TopicExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let TopicExporter {
            topic,
            queue_on_full,
            mut metrics,
        } = *self;
        let exporter_id = effect_handler.exporter_id();
        otel_info!(
            "topic_exporter.start",
            node = exporter_id.name.as_ref(),
            topic = topic.name().as_ref(),
            queue_on_full = format!("{queue_on_full:?}"),
            message = "Topic exporter started"
        );
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let run_result: Result<(), Error> = async {
            loop {
                match msg_chan.recv().await? {
                    Message::Control(NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    }) => {
                        _ = metrics_reporter.report(&mut metrics);
                    }
                    Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                    Message::PData(data) => {
                        // Topic hop is a transport boundary: do not propagate in-process
                        // Ack/Nack routing state (node ids/call data) across pipelines.
                        let published = Arc::new(data.clone_without_context());
                        let publish_result = match queue_on_full {
                            TopicQueueOnFullPolicy::Block => {
                                topic.publish(published).await?;
                                Ok(PublishOutcome::Published)
                            }
                            TopicQueueOnFullPolicy::DropNewest => topic.try_publish(published),
                        };

                        match publish_result? {
                            PublishOutcome::Published => {
                                metrics.published_messages.add(1);
                                effect_handler.notify_ack(AckMsg::new(data)).await?;
                            }
                            PublishOutcome::DroppedOnFull => {
                                metrics.dropped_messages_on_full.add(1);
                                let exporter_id = effect_handler.exporter_id();
                                otel_warn!(
                                    "topic_exporter.drop_newest",
                                    node = exporter_id.name.as_ref(),
                                    topic = topic.name().as_ref(),
                                    message = "Dropping message because topic queue is full"
                                );
                                effect_handler
                                    .notify_nack(NackMsg::new("topic queue full: dropped newest", data))
                                    .await?;
                            }
                        }
                        tokio::task::consume_budget().await;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        .await;

        _ = telemetry_cancel_handle.cancel().await;
        run_result?;
        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::TopicExporter;
    use otap_df_config::topic::TopicQueueOnFullPolicy;
    use serde_json::json;

    #[test]
    fn parse_config_accepts_minimal_topic() {
        let cfg = TopicExporter::parse_config(&json!({"topic": "raw"})).expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert!(cfg.queue_on_full.is_none());
    }

    #[test]
    fn parse_config_accepts_local_queue_on_full_override() {
        let cfg = TopicExporter::parse_config(&json!({
            "topic": "raw",
            "queue_on_full": "drop_newest"
        }))
        .expect("valid config");
        assert_eq!(cfg.topic.as_ref(), "raw");
        assert_eq!(cfg.queue_on_full, Some(TopicQueueOnFullPolicy::DropNewest));
    }

    #[test]
    fn parse_config_rejects_unknown_queue_on_full_variant() {
        let err = TopicExporter::parse_config(&json!({
            "topic": "raw",
            "queue_on_full": "unknown_variant"
        }))
        .expect_err("unknown queue_on_full should fail");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn parse_config_rejects_empty_topic() {
        let err = TopicExporter::parse_config(&json!({"topic": "   "}))
            .expect_err("empty topic should fail");
        assert!(err.to_string().contains("topic name must be non-empty"));
    }
}
