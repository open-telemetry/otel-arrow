// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A processor that introduces an artificial delay before forwarding data.
//!
//! This is useful for simulating slow pipeline stages in testing and
//! benchmarking scenarios (e.g., backpressure, inflight limits, timeouts).
//!
//! # Configuration
//!
//! ```yaml
//! delay-node:
//!   type: urn:otel:processor:delay
//!   config:
//!     delay: 500ms
//! ```
//!
//! The `delay` field uses humantime format (e.g., "100ms", "1s", "2s500ms").

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{MessageSourceLocalEffectHandlerExtension as _, ProcessorFactory};
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// The URN for the delay processor.
pub const DELAY_PROCESSOR_URN: &str = "urn:otel:processor:delay";

/// Configuration for the delay processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayConfig {
    /// How long to sleep before forwarding each message.
    /// Format: humantime (e.g., "500ms", "1s", "2s").
    #[serde(with = "humantime_serde")]
    pub delay: Duration,
}

/// Register the delay processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static DELAY_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: DELAY_PROCESSOR_URN,
    create: create_delay_processor,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<DelayConfig>,
};

/// Factory function to create a DelayProcessor.
pub fn create_delay_processor(
    _pipeline_ctx: otap_df_engine::context::PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: DelayConfig = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse delay processor configuration: {e}"),
        }
    })?;

    Ok(ProcessorWrapper::local(
        DelayProcessor {
            delay: config.delay,
        },
        node,
        node_config,
        processor_config,
    ))
}

/// A processor that sleeps for a configured duration before forwarding data.
pub struct DelayProcessor {
    delay: Duration,
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for DelayProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::PData(pdata) => {
                tokio::time::sleep(self.delay).await;
                effect_handler.send_message_with_source_node(pdata).await?;
                Ok(())
            }
            Message::Control(NodeControlMsg::CollectTelemetry { .. }) => Ok(()),
            Message::Control(NodeControlMsg::Shutdown { .. }) => Ok(()),
            Message::Control(NodeControlMsg::Config { .. }) => Ok(()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::testing::node::test_node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_otap::testing::create_test_pdata;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_config_parsing() {
        let config: DelayConfig = serde_json::from_value(json!({
            "delay": "500ms"
        }))
        .unwrap();
        assert_eq!(config.delay, Duration::from_millis(500));

        let config: DelayConfig = serde_json::from_value(json!({
            "delay": "2s"
        }))
        .unwrap();
        assert_eq!(config.delay, Duration::from_secs(2));
    }

    #[test]
    fn test_config_missing_delay_fails() {
        let result = serde_json::from_value::<DelayConfig>(json!({}));
        assert!(result.is_err());
    }

    fn create_test_pipeline_context() -> otap_df_engine::context::PipelineContext {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
    }

    #[test]
    fn test_delay_processor_forwards_data() {
        let pipeline_ctx = create_test_pipeline_context();
        let node = test_node("delay-processor-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();

        let mut node_config = NodeUserConfig::new_processor_config(DELAY_PROCESSOR_URN);
        node_config.config = json!({ "delay": "1ms" });

        let proc = create_delay_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
            .expect("create processor");

        let phase = rt.set_processor(proc);

        phase
            .run_test(move |mut ctx| async move {
                let pdata = create_test_pdata();

                let start = std::time::Instant::now();
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process message");
                let elapsed = start.elapsed();

                // Verify the delay was applied (at least ~1ms).
                assert!(
                    elapsed >= Duration::from_micros(500),
                    "expected delay of at least ~1ms, got {elapsed:?}"
                );

                // Verify the data was forwarded downstream.
                let output = ctx.drain_pdata().await;
                assert_eq!(output.len(), 1, "expected exactly one forwarded message");
            })
            .validate(|ctx| async move {
                let counters = ctx.counters();
                counters.assert(0, 0, 0, 0);
            });
    }
}
