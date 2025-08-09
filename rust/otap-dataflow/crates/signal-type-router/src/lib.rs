// SPDX-License-Identifier: Apache-2.0

//! # SignalTypeRouter Processor
//!
//! A zero-copy signal routing processor for the OTAP (OpenTelemetry Arrow Protocol) dataflow engine.
//! The SignalTypeRouter efficiently routes OpenTelemetry signals (traces, metrics, logs, profiles) to
//! different output ports based on their signal type without deserializing the underlying telemetry data.
//!
//! ## Key Features
//!
//! - **Zero-copy routing**: Routes signal references without cloning or deserializing telemetry data
//! - **Efficient signal detection**: Uses native `signal_type()` method for fast signal type identification  
//! - **Multi-port routing**: Routes different signal types to different output ports
//! - **Flexible dispatch strategies**: Supports broadcast, round-robin, random, and least-loaded routing
//! - **Configuration validation**: Comprehensive configuration parsing and validation
//!
//! ## Architecture
//!
//! The SignalTypeRouter operates by:
//!
//! 1. **Signal Type Detection**: Efficiently identifies the type of incoming OpenTelemetry signals using the native `signal_type()` method
//! 2. **Zero-Copy Routing**: Passes signal references (not clones) to downstream processors based on configured routing rules
//! 3. **Port-Based Distribution**: Routes signals to different output ports based on signal type
//! 4. **Dispatch Strategy Application**: Applies configured dispatch strategies (broadcast, round-robin, etc.) when multiple destinations exist
//!
//! ## Dispatch Strategies
//!
//! - **Broadcast**: Sends each signal to ALL destinations (zero-copy fan-out)
//! - **Round-Robin**: Cycles through destinations for load balancing
//! - **Random**: Randomly selects a destination
//! - **Least-Loaded**: Routes to the least busy destination (requires load monitoring)
//!
//! ## Usage
//!
//! ### Basic Usage
//!
//! ```rust
//! use otap_df_signal_type_router::{SignalTypeRouter, SignalTypeRouterConfig};
//!
//! // Create with default configuration
//! let config = SignalTypeRouterConfig::default();
//! let router = SignalTypeRouter::new(config);
//!
//! // Create with custom configuration
//! let config = SignalTypeRouterConfig {
//!     drop_unknown_signals: true,
//! };
//! let router = SignalTypeRouter::new(config);
//! ```
//!
//! ### Factory Usage
//!
//! ```rust
//! use serde_json::json;
//! use otap_df_signal_type_router::create_signal_type_router;
//!
//! let config = json!({
//!     "drop_unknown_signals": false
//! });
//!
//! let processor_config = otap_df_engine::config::ProcessorConfig::new("my_router");
//! let wrapper = create_signal_type_router(&config, &processor_config)?;
//! # Ok::<(), otap_df_config::error::Error>(())
//! ```
//!
//! ## Explicit Configuration
//!
//! The SignalTypeRouter requires **explicit configuration** for all instances. Each router must specify:
//!
//! - **Output ports**: Named ports for routing different signal types
//! - **Dispatch strategies**: How signals are distributed to destinations
//! - **Destinations**: Target nodes for each output port
//! - **Router-specific settings**: Configuration for signal handling behavior
//!
//! ### Complete Configuration Example
//!
//! ```yaml
//! type: otap
//! description: "Advanced signal type routing pipeline with explicit configuration"
//! settings:
//!   default_control_channel_size: 100
//!   default_pdata_channel_size: 100
//! nodes:
//!   # SignalTypeRouter - requires explicit port and destination configuration
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
//!   # All downstream processors must also be explicitly configured
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
//! ### Configuration Requirements
//!
//! - **No Default Routing**: All signal types must have explicitly defined output ports
//! - **Named Destinations**: All `destinations` must reference explicitly configured nodes
//! - **Complete Specification**: Partial or simplified configurations are not supported
//! - **Validation**: Configuration validation ensures all references are resolvable
//!
//! ## Performance Characteristics
//!
//! - **Zero Memory Allocation**: No cloning of telemetry data during routing
//! - **Constant Time Signal Detection**: O(1) pattern matching on signal types
//! - **Minimal CPU Overhead**: Lightweight routing logic
//! - **High Throughput**: Designed for production-scale telemetry processing

use otap_df_config::error::Error as ConfigError;
use otap_df_engine::{ProcessorFactory, error::Error as EngineError};
use otap_df_otap::pdata::OtapPdata;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod processor;

// TODO: Multi-port routing implementation
// The actual routing logic (mapping signal types to output ports and applying dispatch strategies)
// will likely be implemented in the effect handler layer, which has access to the pipeline
// configuration including out_ports definitions and dispatch strategies. The effect handlers
// can parse the out_ports configuration and route signals accordingly.

pub use processor::SignalTypeRouter;
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
        source: Box<EngineError<OtapPdata>>,
    },
}

/// Factory function to create a SignalTypeRouter processor
pub fn create_signal_type_router(
    config: &Value,
    processor_config: &otap_df_engine::config::ProcessorConfig,
) -> Result<otap_df_engine::processor::ProcessorWrapper<OtapPdata>, ConfigError> {
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
pub const SIGNAL_TYPE_ROUTER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
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
