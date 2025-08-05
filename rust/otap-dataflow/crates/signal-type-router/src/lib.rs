// SPDX-License-Identifier: Apache-2.0

//! # SignalTypeRouter Processor
//!
//! The SignalTypeRouter is a processor that efficiently routes OpenTelemetry signals (traces, metrics, logs)
//! to different output ports based on their signal type. It implements zero-copy routing by working with
//! signal references rather than cloning data.
//!
//! ## Key Features
//!
//! - **Zero-copy routing**: Routes signal references without cloning or deserializing telemetry data
//! - **Efficient signal detection**: Uses `OTLPData` enum matching for fast signal type identification
//! - **Multi-port routing**: Routes different signal types to different output ports
//! - **Flexible dispatch strategies**: Supports broadcast, round-robin, random, and least-loaded routing
//!
//! ## Explicit Configuration
//!
//! The SignalTypeRouter requires **explicit configuration** for all instances. All routing behavior must be completely specified:
//!
//! ```yaml
//! type: otap
//! description: "Explicit signal type routing pipeline"
//! settings:
//!   default_control_channel_size: 100
//!   default_pdata_channel_size: 100
//! nodes:
//!   # All nodes must be explicitly configured
//!   signal_router:
//!     kind: processor
//!     plugin_urn: "urn:otap:processor:signal_type_router"
//!     description: "Routes signals by type with zero-copy forwarding"
//!     out_ports:
//!       traces_out:
//!         dispatch_strategy: round_robin
//!         destinations: [traces_processor_1, traces_processor_2]
//!       metrics_out:
//!         dispatch_strategy: broadcast
//!         destinations: [metrics_storage, metrics_analytics]
//!       logs_out:
//!         dispatch_strategy: random
//!         destinations: [logs_storage_1, logs_storage_2]
//!       profiles_out:
//!         dispatch_strategy: least_loaded
//!         destinations: [profiles_processor_1, profiles_processor_2]
//!     config:
//!       drop_unknown_signals: false
//!
//!   # All referenced destinations must be explicitly configured
//!   traces_processor_1:
//!     kind: processor
//!     plugin_urn: "urn:otap:processor:batch"
//!     config:
//!       timeout: "1s"
//!       send_batch_size: 512
//!
//!   # ... additional explicit configurations for all referenced nodes
//! ```
//!
//! ## Configuration Requirements
//!
//! - **No Default Routing**: All signal types must have explicitly defined output ports
//! - **Named Destinations**: All `destinations` must reference explicitly configured nodes
//! - **Complete Specification**: Partial or simplified configurations are not supported
//! - **Validation**: Configuration validation ensures all references are resolvable

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::{ProcessorFactory, error::Error as EngineError};
use otap_df_otlp::grpc::OTLPData;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod processor;
pub mod routing;

pub use processor::{SignalDetectable, SignalTypeRouter};
// Re-export SignalType from config for consistency
pub use otap_df_config::experimental::SignalType;

/// Configuration for the SignalTypeRouter processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalTypeRouterConfig {
    /// Whether to drop signals of unknown types
    /// If false, unknown signals are forwarded to the default output (first port if available)
    #[serde(default = "default_drop_unknown_signals")]
    pub drop_unknown_signals: bool,
}

fn default_drop_unknown_signals() -> bool {
    false
}

impl Default for SignalTypeRouterConfig {
    fn default() -> Self {
        Self {
            drop_unknown_signals: default_drop_unknown_signals(),
        }
    }
}

/// Error types for the SignalTypeRouter
#[derive(Debug, thiserror::Error)]
pub enum SignalTypeRouterError {
    /// Configuration parsing error
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
    },

    /// Signal routing error
    #[error("Routing error: {message}")]
    Routing {
        /// Error message
        message: String,
    },

    /// Engine-related errors
    #[error("Engine error: {source}")]
    Engine {
        /// Source engine error
        #[from]
        source: EngineError<OTLPData>,
    },
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_signal_type_router(
    config: &Value,
    processor_config: &otap_df_engine::config::ProcessorConfig,
) -> Result<otap_df_engine::processor::ProcessorWrapper<OTLPData>, ConfigError> {
    // Deserialize the router-specific configuration
    let router_config: SignalTypeRouterConfig =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse SignalTypeRouter configuration: {e}"),
        })?;

    log::info!(
        "Creating SignalTypeRouter processor '{}' with config: {:?}",
        processor_config.name,
        router_config
    );

    // Create the router processor
    let router = SignalTypeRouter::new(router_config);

    // For now, we'll default to creating a local processor
    // In the future, this could be configurable based on the processor requirements
    let user_config = std::rc::Rc::new(otap_df_config::node::NodeUserConfig::new_processor_config(
        "urn:otap:processor:signal_type_router",
    ));

    Ok(otap_df_engine::processor::ProcessorWrapper::local(
        router,
        user_config,
        processor_config,
    ))
}

/// Signal type router processor factory
pub const SIGNAL_TYPE_ROUTER_FACTORY: ProcessorFactory<OTLPData> = ProcessorFactory {
    name: "signal_type_router",
    create: create_signal_type_router,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_deserialization() {
        // Test default configuration
        let config_json = json!({});
        let config: SignalTypeRouterConfig = serde_json::from_value(config_json).unwrap();
        assert!(!config.drop_unknown_signals);

        // Test explicit configuration
        let config_json = json!({
            "drop_unknown_signals": true
        });
        let config: SignalTypeRouterConfig = serde_json::from_value(config_json).unwrap();
        assert!(config.drop_unknown_signals);
    }

    #[test]
    fn test_factory_creation() {
        let config = json!({
            "drop_unknown_signals": false
        });

        let processor_config = otap_df_engine::config::ProcessorConfig::new("test_router");

        let result = create_signal_type_router(&config, &processor_config);
        assert!(result.is_ok());
    }
}
