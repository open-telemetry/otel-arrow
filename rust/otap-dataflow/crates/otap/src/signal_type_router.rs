// SPDX-License-Identifier: Apache-2.0

//! Signal type router processor for OTAP pipelines.
//!
//! Simplest behavior: pass-through using engine wiring.
//! All signals are forwarded unchanged via the engine-provided default out port
//! (or error if multiple ports are connected without a default).

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ProcessorFactory;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::node::NodeUnique;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::processor::ProcessorWrapper;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// URN for the SignalTypeRouter processor
pub const SIGNAL_TYPE_ROUTER_URN: &str = "urn:otap:processor:signal_type_router";

/// Well-known out port names for type-based routing
/// Name of the out port used for trace signals
pub const PORT_TRACES: &str = "traces";
/// Name of the out port used for metric signals
pub const PORT_METRICS: &str = "metrics";
/// Name of the out port used for log signals
pub const PORT_LOGS: &str = "logs";

/// Minimal configuration for the SignalTypeRouter processor
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalTypeRouterConfig {}

/// The SignalTypeRouter processor (local, !Send)
pub struct SignalTypeRouter {
    /// Router configuration (currently unused, kept for forward compatibility)
    #[allow(dead_code)]
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
                // Determine desired out port by signal type
                let desired_port = match data.signal_type() {
                    otap_df_config::experimental::SignalType::Traces => PORT_TRACES,
                    otap_df_config::experimental::SignalType::Metrics => PORT_METRICS,
                    otap_df_config::experimental::SignalType::Logs => PORT_LOGS,
                };

                let connected = effect_handler.connected_ports();
                let has_port = connected.iter().any(|p| p.as_ref() == desired_port);

                // ToDo [LQ] send_message_to should returns a dedicated error when the port is not found so we can avoid to call the `connected.iter().any(...)`.
                if has_port {
                    effect_handler.send_message_to(desired_port, data).await
                } else {
                    // No matching named port: fall back to engine default behavior
                    effect_handler.send_message(data).await
                }
            }
        }
    }
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_signal_type_router(
    node: NodeUnique,
    config: &Value,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    // Deserialize the (currently empty) router configuration
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
        node,
        user_config,
        processor_config,
    ))
}

/// Register SignalTypeRouter as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static SIGNAL_TYPE_ROUTER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: SIGNAL_TYPE_ROUTER_URN,
    create: |node: NodeUnique, config: &Value, proc_cfg: &ProcessorConfig| {
        create_signal_type_router(node, config, proc_cfg)
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
        let _cfg: SignalTypeRouterConfig = serde_json::from_value(config_json).unwrap();
    }

    #[test]
    fn test_factory_creation_ok() {
        let config = json!({});
        let processor_config = ProcessorConfig::new("test_router");
        let test_runtime = TestRuntime::<()>::new();
        let result =
            create_signal_type_router(test_runtime.test_node(), &config, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_creation_bad_config() {
        // An invalid type (e.g., number instead of object) should error
        let config = json!(42);
        let processor_config = ProcessorConfig::new("test_router");
        let test_runtime = TestRuntime::<()>::new();
        let result =
            create_signal_type_router(test_runtime.test_node(), &config, &processor_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_messages_pass_through() {
        use otap_df_config::node::NodeUserConfig;
        use std::sync::Arc;

        let test_runtime = TestRuntime::new();
        let user_cfg = Arc::new(NodeUserConfig::new_processor_config("sig_router_test"));
        let wrapper = ProcessorWrapper::local(
            SignalTypeRouter::new(SignalTypeRouterConfig::default()),
            test_runtime.test_node(),
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
            })
        });

        // No-op validation closure
        validation.validate(|_| async {});
    }
}
