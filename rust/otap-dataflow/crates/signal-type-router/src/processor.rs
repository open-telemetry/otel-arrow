// SPDX-License-Identifier: Apache-2.0

//! Core SignalTypeRouter processor implementation
//!
//! This module contains the main [`SignalTypeRouter`] processor that performs zero-copy routing
//! of OpenTelemetry signals based on their signal type. The processor uses the native
//! `signal_type()` method on [`OtapPdata`] for efficient signal type identification.
//!
//! ## Signal Type Detection
//!
//! The router uses the following approach for signal type detection:
//!
//! ```rust
//! use otap_df_otap::pdata::{OtapPdata, OtlpProtoBytes};
//! use otap_df_config::experimental::SignalType;
//!
//! // Create sample signals using OtapPdata
//! let traces_signal = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportTracesRequest(vec![]));
//! let metrics_signal = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(vec![]));
//! let logs_signal = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(vec![]));
//!
//! // Signal type detection using native method
//! assert_eq!(traces_signal.signal_type(), SignalType::Traces);
//! assert_eq!(metrics_signal.signal_type(), SignalType::Metrics);
//! assert_eq!(logs_signal.signal_type(), SignalType::Logs);
//! ```
//!
//! ## Zero-Copy Routing
//!
//! The router implements zero-copy routing by passing signal references directly to
//! downstream processors without cloning the underlying telemetry data:
//!
//! 1. **Signal Identification**: Uses `data.signal_type()` to determine the signal type
//! 2. **Port Selection**: Maps signal type to the appropriate output port
//! 3. **Reference Forwarding**: Passes the original signal reference to effect handlers
//!
//! This approach ensures minimal memory allocation and CPU overhead during routing operations.

use crate::{SignalTypeRouterConfig, SignalTypeRouterError};
use async_trait::async_trait;
use otap_df_engine::{
    error::Error as EngineError,
    local::processor::{EffectHandler as LocalEffectHandler, Processor as LocalProcessor},
    message::Message,
    shared::processor::{EffectHandler as SharedEffectHandler, Processor as SharedProcessor},
};
use otap_df_otap::pdata::OtapPdata;

/// Signal type router processor
///
/// Routes incoming OpenTelemetry signals to different output ports based on their signal type.
/// The router identifies signal types efficiently using the `OtapPdata` enum without deserializing
/// the underlying telemetry data.
///
/// ## Explicit Configuration Requirements
///
/// This processor requires **explicit configuration** for all routing behavior:
/// - All signal types must have explicitly defined output ports
/// - All destination nodes must be completely configured
/// - No implicit or default routing behaviors are supported
/// - Configuration validation ensures all references are resolvable
pub struct SignalTypeRouter {
    /// Router configuration
    #[allow(dead_code)] // Will be used when full routing is implemented
    config: SignalTypeRouterConfig,
}

impl SignalTypeRouter {
    /// Creates a new SignalTypeRouter with the given configuration
    #[must_use]
    pub fn new(config: SignalTypeRouterConfig) -> Self {
        Self { config }
    }

    /// Routes the signal to the appropriate output port based on its type
    ///
    /// **Current Implementation**: This is a pass-through implementation that forwards
    /// all signals to the default output regardless of signal type. Multi-port routing
    /// is planned for future implementation.
    async fn route_signal<E: EffectHandlerTrait>(
        &mut self,
        data: OtapPdata,
        effect_handler: &mut E,
    ) -> Result<(), SignalTypeRouterError> {
        let signal_type = data.signal_type();

        log::debug!("Routing signal of type: {signal_type:?} (current: pass-through mode)");

        // Current implementation: Forward all signals to default output
        // TODO: Implement multi-port routing based on signal type
        //
        // The actual routing logic will likely be implemented in the effect handler layer,
        // which has access to the pipeline configuration including out_ports definitions
        // and dispatch strategies. The effect handlers can:
        // 1. Parse the out_ports configuration for this processor
        // 2. Map signal types to appropriate output port names
        // 3. Apply the configured dispatch strategy for each port
        // 4. Route signals to the correct downstream destinations
        effect_handler
            .send_message(data)
            .await
            .map_err(|e| SignalTypeRouterError::Engine {
                source: Box::new(e),
            })?;

        Ok(())
    }
}

/// Trait to abstract over different effect handler types
trait EffectHandlerTrait {
    async fn send_message(&mut self, data: OtapPdata) -> Result<(), EngineError<OtapPdata>>;
}

impl EffectHandlerTrait for LocalEffectHandler<OtapPdata> {
    async fn send_message(&mut self, data: OtapPdata) -> Result<(), EngineError<OtapPdata>> {
        LocalEffectHandler::send_message(self, data).await
    }
}

impl EffectHandlerTrait for SharedEffectHandler<OtapPdata> {
    async fn send_message(&mut self, data: OtapPdata) -> Result<(), EngineError<OtapPdata>> {
        SharedEffectHandler::send_message(self, data)
            .await
            .map_err(EngineError::ChannelSendError)
    }
}

#[async_trait(?Send)]
impl LocalProcessor<OtapPdata> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut LocalEffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        match msg {
            Message::Control(ctrl_msg) => {
                log::debug!("SignalTypeRouter received control message: {ctrl_msg:?}");
                // Handle control messages (shutdown, config updates, etc.)
                // For now, we don't need to process control messages specifically
                Ok(())
            }
            Message::PData(data) => {
                log::trace!("SignalTypeRouter processing signal data");
                self.route_signal(data, effect_handler).await.map_err(|e| {
                    EngineError::ProcessorError {
                        processor: "SignalTypeRouter".into(),
                        error: e.to_string(),
                    }
                })
            }
        }
    }
}

#[async_trait]
impl SharedProcessor<OtapPdata> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut SharedEffectHandler<OtapPdata>,
    ) -> Result<(), EngineError<OtapPdata>> {
        match msg {
            Message::Control(ctrl_msg) => {
                log::debug!("SignalTypeRouter received control message: {ctrl_msg:?}");
                // Handle control messages (shutdown, config updates, etc.)
                // For now, we don't need to process control messages specifically
                Ok(())
            }
            Message::PData(data) => {
                log::trace!("SignalTypeRouter processing signal data");
                self.route_signal(data, effect_handler).await.map_err(|e| {
                    EngineError::ProcessorError {
                        processor: "SignalTypeRouter".into(),
                        error: e.to_string(),
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::experimental::SignalType;
    use otap_df_otap::pdata::OtlpProtoBytes;

    #[test]
    fn test_signal_type_detection() {
        // Test traces signal detection
        let traces_data = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportTracesRequest(vec![]));
        assert_eq!(traces_data.signal_type(), SignalType::Traces);

        // Test metrics signal detection
        let metrics_data = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(vec![]));
        assert_eq!(metrics_data.signal_type(), SignalType::Metrics);

        // Test logs signal detection
        let logs_data = OtapPdata::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(vec![]));
        assert_eq!(logs_data.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_router_creation() {
        let config = SignalTypeRouterConfig {
            drop_unknown_signals: true,
        };
        let router = SignalTypeRouter::new(config.clone());
        assert_eq!(
            router.config.drop_unknown_signals,
            config.drop_unknown_signals
        );
    }
}
