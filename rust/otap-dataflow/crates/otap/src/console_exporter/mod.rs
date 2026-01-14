// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Console exporter that prints OTLP data with hierarchical formatting.
//!
//! This exporter displays logs (and future support for traces/metrics) with
//! resource and scope grouping using tree-style output:
//!
//! ```text
//! <timestamp> RESOURCE {service.name=my-service, ...}
//! │ <timestamp> SCOPE {name=my-scope, version=1.0}
//! │ ├─ INFO  event_name: message [attr=value]
//! │ ├─ WARN  event_name: warning message
//! │ └─ ERROR event_name: error message [code=500]
//! ```

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::SignalType;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_pdata::OtapPayload;
use std::sync::Arc;

mod formatter;

use formatter::HierarchicalFormatter;

/// The URN for the console exporter
pub const CONSOLE_EXPORTER_URN: &str = "urn:otap:console:exporter";

/// Configuration for the console exporter
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ConsoleExporterConfig {
    /// Whether to use ANSI colors in output (default: true)
    #[serde(default = "default_color")]
    pub color: bool,
    /// Whether to use Unicode box-drawing characters (default: true)
    #[serde(default = "default_unicode")]
    pub unicode: bool,
}

fn default_color() -> bool {
    true
}

fn default_unicode() -> bool {
    true
}

/// Console exporter that prints OTLP data with hierarchical formatting
pub struct ConsoleExporter {
    formatter: HierarchicalFormatter,
}

impl ConsoleExporter {
    /// Create a new console exporter with the given configuration.
    #[must_use]
    pub fn new(config: ConsoleExporterConfig) -> Self {
        Self {
            formatter: HierarchicalFormatter::new(config.color, config.unicode),
        }
    }
}

/// Declare the Console Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CONSOLE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CONSOLE_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        let config: ConsoleExporterConfig =
            serde_json::from_value(node_config.config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!("Failed to parse console exporter config: {}", e),
                }
            })?;
        Ok(ExporterWrapper::local(
            ConsoleExporter::new(config),
            node,
            node_config,
            exporter_config,
        ))
    },
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ConsoleExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    self.export(&data);
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(TerminalState::default())
    }
}

impl ConsoleExporter {
    fn export(&self, data: &OtapPdata) {
        let (_, payload) = data.clone().into_parts();
        match payload.signal_type() {
            SignalType::Logs => self.export_logs(&payload),
            SignalType::Traces => self.export_traces(&payload),
            SignalType::Metrics => self.export_metrics(&payload),
        }
    }

    fn export_logs(&self, payload: &OtapPayload) {
        match payload {
            OtapPayload::OtlpBytes(bytes) => {
                self.formatter.format_logs_bytes(bytes);
            }
            OtapPayload::OtapArrowRecords(_records) => {
                // TODO: Support Arrow format
                eprintln!("Console exporter: Arrow format not yet supported for logs");
            }
        }
    }

    fn export_traces(&self, _payload: &OtapPayload) {
        // TODO: Implement traces formatting
        eprintln!("Console exporter: Traces formatting not yet implemented");
    }

    fn export_metrics(&self, _payload: &OtapPayload) {
        // TODO: Implement metrics formatting
        eprintln!("Console exporter: Metrics formatting not yet implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use otap_df_engine::Interests;
    use serde_json::json;

    #[test]
    fn test_console_exporter_no_subscription() {
        test_exporter_no_subscription(&CONSOLE_EXPORTER, json!({}));
    }

    #[test]
    fn test_console_exporter_with_subscription() {
        test_exporter_with_subscription(
            &CONSOLE_EXPORTER,
            json!({}),
            Interests::ACKS,
            Interests::ACKS,
        );
    }

    #[test]
    fn test_console_exporter_config_defaults() {
        let config: ConsoleExporterConfig = serde_json::from_value(json!({})).unwrap();
        assert!(config.color);
        assert!(config.unicode);
    }

    #[test]
    fn test_console_exporter_config_custom() {
        let config: ConsoleExporterConfig =
            serde_json::from_value(json!({"color": false, "unicode": false})).unwrap();
        assert!(!config.color);
        assert!(!config.unicode);
    }
}
