// SPDX-License-Identifier: Apache-2.0

//! Signal type router processor for OTAP pipelines.
//!
//! Current behavior: pass-through. All signals are forwarded unchanged.
//! Multi-port routing will be added later at the effect handler level.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::PortName;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ProcessorFactory;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::processor::ProcessorWrapper;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// URN for the SignalTypeRouter processor
pub const SIGNAL_TYPE_ROUTER_URN: &str = "urn:otap:processor:signal_type_router";

/// Configuration for the SignalTypeRouter processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalTypeRouterConfig {
    /// Named out ports to route traces signals to (broadcast if more than one).
    #[serde(default)]
    pub traces_ports: Vec<PortName>,
    /// Named out ports to route metrics signals to (broadcast if more than one).
    #[serde(default)]
    pub metrics_ports: Vec<PortName>,
    /// Named out ports to route logs signals to (broadcast if more than one).
    #[serde(default)]
    pub logs_ports: Vec<PortName>,
    /// Optional default out port name to use when a mapping is not defined.
    /// If not set, the engine's single-port fallback or configured default is used.
    #[serde(default)]
    pub default_port: Option<PortName>,
    /// Whether to drop signals of unknown or unmapped types.
    /// If false, undefined/unknown signals may be forwarded to a default/first port
    /// as determined by the pipeline engine.
    #[serde(default = "default_drop_unknown_signals")]
    pub drop_unknown_signals: bool,
}

fn default_drop_unknown_signals() -> bool {
    false
}

impl Default for SignalTypeRouterConfig {
    fn default() -> Self {
        Self {
            traces_ports: Vec::new(),
            metrics_ports: Vec::new(),
            logs_ports: Vec::new(),
            default_port: None,
            drop_unknown_signals: default_drop_unknown_signals(),
        }
    }
}

/// The SignalTypeRouter processor (local, !Send)
pub struct SignalTypeRouter {
    /// Router configuration
    #[allow(dead_code)] // todo Remove this once full routing is implemented
    config: SignalTypeRouterConfig,
}

impl SignalTypeRouter {
    /// Creates a new SignalTypeRouter with the given configuration
    #[must_use]
    pub fn new(config: SignalTypeRouterConfig) -> Self {
        Self { config }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        match msg {
            Message::Control(_ctrl) => {
                // No specific control handling required currently.
                Ok(())
            }
            Message::PData(data) => {
                // Determine signal type and route to the configured port list if present.
                let signal_type = data.signal_type();
                let ports: &[PortName] = match signal_type {
                    SignalType::Traces => &self.config.traces_ports,
                    SignalType::Metrics => &self.config.metrics_ports,
                    SignalType::Logs => &self.config.logs_ports,
                };

                match ports.len() {
                    n if n > 1 => {
                        // Broadcast: send a shallow clone per port.
                        for p in ports {
                            // Clone is expected to be shallow (Arrow/OTLP bytes)
                            effect_handler
                                .send_message_to(p.clone(), data.clone())
                                .await?;
                        }
                        Ok(())
                    }
                    1 => effect_handler.send_message_to(ports[0].clone(), data).await,
                    _ => {
                        // No mapping for this type.
                        if self.config.drop_unknown_signals {
                            return Ok(());
                        }
                        if let Some(default_port) = self.config.default_port.clone() {
                            return effect_handler.send_message_to(default_port, data).await;
                        }
                        // Engine default: single-port fallback or configured node-level default.
                        effect_handler.send_message(data).await
                    }
                }
            }
        }
    }
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_signal_type_router(
    config: &Value,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    // Deserialize the router-specific configuration
    let router_config: SignalTypeRouterConfig =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse SignalTypeRouter configuration: {e}"),
        })?;

    // Create the router processor
    let router = SignalTypeRouter::new(router_config);

    // Create NodeUserConfig and wrap as local processor
    let user_config = Arc::new(NodeUserConfig::new_processor_config(SIGNAL_TYPE_ROUTER_URN));

    Ok(ProcessorWrapper::local(
        router,
        user_config,
        processor_config,
    ))
}

/// Register SignalTypeRouter as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static SIGNAL_TYPE_ROUTER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: SIGNAL_TYPE_ROUTER_URN,
    create: |config: &Value, proc_cfg: &ProcessorConfig| {
        create_signal_type_router(config, proc_cfg)
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::testing::processor::TestRuntime;
    use serde_json::json;

    #[test]
    fn test_config_deserialization_defaults() {
        let config_json = json!({});
        let cfg: SignalTypeRouterConfig = serde_json::from_value(config_json).unwrap();
        assert!(!cfg.drop_unknown_signals);
    }

    #[test]
    fn test_factory_creation_ok() {
        let config = json!({ "drop_unknown_signals": false });
        let processor_config = ProcessorConfig::new("test_router");
        let result = create_signal_type_router(&config, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_creation_bad_config() {
        let config = json!({ "drop_unknown_signals": "not-a-bool" });
        let processor_config = ProcessorConfig::new("test_router");
        let result = create_signal_type_router(&config, &processor_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_messages_pass_through() {
        use otap_df_config::node::NodeUserConfig;
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::Arc;

        let test_runtime = TestRuntime::new();
        let user_cfg = Arc::new(NodeUserConfig::new_processor_config("sig_router_test"));
        let wrapper = ProcessorWrapper::local(
            SignalTypeRouter::new(SignalTypeRouterConfig::default()),
            user_cfg,
            test_runtime.config(),
        );

        let validation = test_runtime.set_processor(wrapper).run_test(|mut ctx| {
            Box::pin(async move {
                // Control message is handled and produces no output
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("control processing failed");
                assert!(ctx.drain_pdata().await.is_empty());

                // Data message is forwarded
                use crate::grpc::OtapArrowBytes;
                use otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords;
                let data = OtapArrowBytes::ArrowLogs(BatchArrowRecords::default());
                ctx.process(Message::data_msg(data.into()))
                    .await
                    .expect("data processing failed");
                let forwarded = ctx.drain_pdata().await;
                assert_eq!(forwarded.len(), 1);
            }) as Pin<Box<dyn Future<Output = ()>>>
        });

        // No-op validation closure
        validation.validate(|_| async {});
    }
}
