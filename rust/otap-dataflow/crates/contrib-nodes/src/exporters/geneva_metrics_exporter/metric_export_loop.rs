// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::client::MetricsPublisher;
use super::otlp_to_geneva::Config;
use super::publication_preparation::prepare_publications;
use async_trait::async_trait;
use otel_arrow_dfe_config::{SignalFormat, SignalType};
use otel_arrow_dfe_engine::ConsumerEffectHandlerExtension;
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider as LocalBearerTokenProvider;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::otel_warn;

const UNSUPPORTED_SIGNAL_MESSAGE: &str = "Geneva metrics exporter accepts metrics only";
const UNSUPPORTED_FORMAT_MESSAGE: &str = "Geneva metrics exporter accepts OTLP only";

/// Pipeline exporter that converts OTLP metrics to Geneva protocol v6 packets.
pub struct GenevaMetricsExporter {
    mapping_config: Config,
    publisher: MetricsPublisher,
    token_provider: Box<dyn LocalBearerTokenProvider>,
}

impl GenevaMetricsExporter {
    pub(crate) fn new(
        mapping_config: Config,
        publisher: MetricsPublisher,
        token_provider: Box<dyn LocalBearerTokenProvider>,
    ) -> Self {
        Self {
            mapping_config,
            publisher,
            token_provider,
        }
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaMetricsExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    let signal_type = data.signal_type();
                    let signal_format = data.signal_format();
                    if signal_type != SignalType::Metrics {
                        let nack = NackMsg::new_permanent(UNSUPPORTED_SIGNAL_MESSAGE, data);
                        effect_handler.notify_nack(nack).await?;
                    } else if signal_format != SignalFormat::OtlpBytes {
                        let nack = NackMsg::new_permanent(UNSUPPORTED_FORMAT_MESSAGE, data);
                        effect_handler.notify_nack(nack).await?;
                    } else {
                        match prepare_publications(&data, &self.mapping_config) {
                            Ok(prepared) => {
                                if prepared.rejected_data_points > 0
                                    || prepared.cardinality_overflows > 0
                                {
                                    otel_warn!(
                                        "geneva_metrics_exporter.mapping.partial",
                                        rejected_data_points = prepared.rejected_data_points,
                                        cardinality_overflows = prepared.cardinality_overflows,
                                    );
                                }
                                if prepared.publications.is_empty() {
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                    continue;
                                }

                                let bearer_token = match self.token_provider.get_token().await {
                                    Ok(token) => token,
                                    Err(error) => {
                                        let nack = NackMsg::new(
                                            format!(
                                                "failed to acquire Geneva metrics bearer token: {error}"
                                            ),
                                            data,
                                        );
                                        effect_handler.notify_nack(nack).await?;
                                        continue;
                                    }
                                };
                                let mut failure = None;
                                for (monitoring_account, packet) in prepared.publications {
                                    if let Err(error) = self
                                        .publisher
                                        .publish(
                                            &monitoring_account,
                                            packet,
                                            bearer_token.expose_token(),
                                        )
                                        .await
                                    {
                                        failure = Some((monitoring_account, error));
                                        break;
                                    }
                                }

                                if let Some((monitoring_account, error)) = failure {
                                    let reason = format!(
                                        "failed to publish Geneva metrics for account {monitoring_account}: {error}"
                                    );
                                    let nack = if error.is_retryable() {
                                        NackMsg::new(reason, data)
                                    } else {
                                        NackMsg::new_permanent(reason, data)
                                    };
                                    effect_handler.notify_nack(nack).await?;
                                } else {
                                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                                }
                            }
                            Err(error) => {
                                let nack = NackMsg::new_permanent(error.to_string(), data);
                                effect_handler.notify_nack(nack).await?;
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
