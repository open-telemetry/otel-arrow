// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ERROR_EXPORTER_URN: &str = "urn:otel:error:exporter";

/// The error exporter is an exporter that does nothing (like
/// noop_exporter) but returns a NACK with a configurable message.
struct ErrorExporter {
    message: String,
}

/// Configuration for the error exporter.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorExporterConfig {
    /// The error message.
    pub message: String,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
static ERROR_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: ERROR_EXPORTER_URN,
    create: ErrorExporter::create_exporter,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<ErrorExporterConfig>,
};

impl ErrorExporter {
    fn create_exporter(
        _pipeline: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        exporter_config: &ExporterConfig,
    ) -> Result<ExporterWrapper<OtapPdata>, otap_df_config::error::Error> {
        let config: ErrorExporterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Failed to parse error-exporter configuration: {e}"),
            })?;

        let exporter = ErrorExporter::from_config(config);

        Ok(ExporterWrapper::local(
            exporter,
            node,
            node_config,
            exporter_config,
        ))
    }

    fn from_config(config: ErrorExporterConfig) -> Self {
        Self {
            message: config.message,
        }
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ErrorExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    effect_handler
                        .notify_nack(NackMsg::new(&self.message, data))
                        .await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(TerminalState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use otap_df_engine::Interests;
    use serde_json::json;

    #[test]
    fn test_error_exporter_no_subscription() {
        test_exporter_no_subscription(&ERROR_EXPORTER, json!({"message": "Test error"}));
    }

    #[test]
    fn test_error_exporter_with_subscription() {
        let config = json!({"message": "THIS specific error"});
        test_exporter_with_subscription(
            &ERROR_EXPORTER,
            config.clone(),
            Interests::NACKS,
            Interests::NACKS,
        );
        test_exporter_with_subscription(
            &ERROR_EXPORTER,
            config.clone(),
            Interests::NACKS,
            Interests::NACKS | Interests::RETURN_DATA,
        );
        test_exporter_with_subscription(
            &ERROR_EXPORTER,
            config,
            Interests::ACKS,
            Interests::empty(),
        );
    }
}
