// SPDX-License-Identifier: Apache-2.0

//! Core SignalTypeRouter processor implementation

use crate::{SignalTypeRouterConfig, SignalTypeRouterError};
use async_trait::async_trait;
use otap_df_config::experimental::SignalType;
use otap_df_engine::{
    error::Error as EngineError,
    local::processor::{EffectHandler as LocalEffectHandler, Processor as LocalProcessor},
    message::Message,
    shared::processor::{EffectHandler as SharedEffectHandler, Processor as SharedProcessor},
};
use otap_df_otlp::grpc::OTLPData;

/// Signal type router processor
///
/// Routes incoming OpenTelemetry signals to different output ports based on their signal type.
/// The router identifies signal types efficiently using the `OTLPData` enum without deserializing
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

    /// Determines the signal type from any SignalDetectable data
    ///
    /// This method will be simplified once native signal_type() methods are available
    /// on all data types (see: https://github.com/open-telemetry/otel-arrow/pull/862)
    fn get_signal_type<T: SignalDetectable>(&self, data: &T) -> SignalType {
        data.signal_type()
    }

    /// Enhanced signal type detection that can work with both trait-based and native methods
    ///
    /// This method provides a migration path for when OTAP types get native signal_type() methods
    fn detect_signal_type(&self, data: &OTLPData) -> SignalType {
        // For now, use the trait-based approach
        // TODO: Once PR #862 lands, this can be simplified to direct method calls
        self.get_signal_type(data)
    }

    /// Routes the signal to the appropriate output port based on its type
    async fn route_signal<E: EffectHandlerTrait>(
        &mut self,
        data: OTLPData,
        effect_handler: &mut E,
    ) -> Result<(), SignalTypeRouterError> {
        let signal_type = self.detect_signal_type(&data);

        log::debug!("Routing signal of type: {signal_type:?}");

        // For now, we'll send to the default output port
        // TODO: Implement proper port routing based on signal type and configuration
        effect_handler
            .send_message(data)
            .await
            .map_err(|e| SignalTypeRouterError::Engine { source: e })?;

        Ok(())
    }
}

/// Trait to abstract over different effect handler types
trait EffectHandlerTrait {
    async fn send_message(&mut self, data: OTLPData) -> Result<(), EngineError<OTLPData>>;
}

impl EffectHandlerTrait for LocalEffectHandler<OTLPData> {
    async fn send_message(&mut self, data: OTLPData) -> Result<(), EngineError<OTLPData>> {
        LocalEffectHandler::send_message(self, data).await
    }
}

impl EffectHandlerTrait for SharedEffectHandler<OTLPData> {
    async fn send_message(&mut self, data: OTLPData) -> Result<(), EngineError<OTLPData>> {
        SharedEffectHandler::send_message(self, data)
            .await
            .map_err(EngineError::ChannelSendError)
    }
}

/// Trait for data types that can be detected for signal type
///
/// This trait provides a common interface for determining the OpenTelemetry signal type
/// of any data structure. It serves as a stopgap solution that will make future transitions
/// to new PData/OTLP/OTAPData implementations easier.
///
/// NOTE: This trait will become obsolete once native signal_type() methods are available
/// on all OTAP data types (see: https://github.com/open-telemetry/otel-arrow/pull/862)
pub trait SignalDetectable {
    /// Returns the signal type of this data
    fn signal_type(&self) -> SignalType;
}

/// Implementation of SignalDetectable for OTLPData
impl SignalDetectable for OTLPData {
    fn signal_type(&self) -> SignalType {
        match self {
            OTLPData::Traces(_) => SignalType::Traces,
            OTLPData::Metrics(_) => SignalType::Metrics,
            OTLPData::Logs(_) => SignalType::Logs,
            // Note: Profiles not available in experimental::SignalType yet
            // This will need to be handled when PR #862 lands
            OTLPData::Profiles(_) => {
                log::warn!(
                    "Profiles signal type not yet supported in config::experimental::SignalType"
                );
                // For now, treat as logs or add custom handling
                SignalType::Logs // Temporary fallback
            }
        }
    }
}

/// Implementation of SignalDetectable for OTAP data types
///
/// These implementations provide signal type detection for the Open Telemetry Arrow Protocol (OTAP)
/// data formats, enabling the SignalTypeRouter to work with both OTLP and OTAP data seamlessly.
// Implementation for OtapPdata (the main OTAP pipeline data container)
impl SignalDetectable for otap_df_otap::pdata::OtapPdata {
    fn signal_type(&self) -> SignalType {
        use otap_df_otap::pdata::OtapPdata;
        match self {
            OtapPdata::OtlpBytes(otlp_bytes) => otlp_bytes.signal_type(),
            OtapPdata::OtapArrowBytes(otap_bytes) => otap_bytes.signal_type(),
            OtapPdata::OtapArrowRecords(otap_records) => otap_records.signal_type(),
        }
    }
}

// Implementation for OtlpProtoBytes (OTLP bytes within OTAP)
impl SignalDetectable for otap_df_otap::pdata::OtlpProtoBytes {
    fn signal_type(&self) -> SignalType {
        use otap_df_otap::pdata::OtlpProtoBytes;
        match self {
            OtlpProtoBytes::ExportLogsRequest(_) => SignalType::Logs,
            OtlpProtoBytes::ExportMetricsRequest(_) => SignalType::Metrics,
            OtlpProtoBytes::ExportTracesRequest(_) => SignalType::Traces,
        }
    }
}

// Implementation for OtapArrowBytes (OTAP Arrow bytes)
impl SignalDetectable for otap_df_otap::grpc::OtapArrowBytes {
    fn signal_type(&self) -> SignalType {
        use otap_df_otap::grpc::OtapArrowBytes;
        match self {
            OtapArrowBytes::ArrowLogs(_) => SignalType::Logs,
            OtapArrowBytes::ArrowMetrics(_) => SignalType::Metrics,
            OtapArrowBytes::ArrowTraces(_) => SignalType::Traces,
        }
    }
}

// Implementation for OtapArrowRecords (OTAP Arrow records)
impl SignalDetectable for otel_arrow_rust::otap::OtapArrowRecords {
    fn signal_type(&self) -> SignalType {
        use otel_arrow_rust::otap::OtapArrowRecords;
        match self {
            OtapArrowRecords::Logs(_) => SignalType::Logs,
            OtapArrowRecords::Metrics(_) => SignalType::Metrics,
            OtapArrowRecords::Traces(_) => SignalType::Traces,
        }
    }
}

#[async_trait(?Send)]
impl LocalProcessor<OTLPData> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OTLPData>,
        effect_handler: &mut LocalEffectHandler<OTLPData>,
    ) -> Result<(), EngineError<OTLPData>> {
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
impl SharedProcessor<OTLPData> for SignalTypeRouter {
    async fn process(
        &mut self,
        msg: Message<OTLPData>,
        effect_handler: &mut SharedEffectHandler<OTLPData>,
    ) -> Result<(), EngineError<OTLPData>> {
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
    use otap_df_otlp::proto::opentelemetry::collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        profiles::v1development::ExportProfilesServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    };

    #[test]
    fn test_signal_type_detection() {
        let router = SignalTypeRouter::new(SignalTypeRouterConfig::default());

        // Test traces signal detection
        let traces_data = OTLPData::Traces(ExportTraceServiceRequest {
            resource_spans: vec![],
        });
        assert_eq!(router.get_signal_type(&traces_data), SignalType::Traces);

        // Test metrics signal detection
        let metrics_data = OTLPData::Metrics(ExportMetricsServiceRequest {
            resource_metrics: vec![],
        });
        assert_eq!(router.get_signal_type(&metrics_data), SignalType::Metrics);

        // Test logs signal detection
        let logs_data = OTLPData::Logs(ExportLogsServiceRequest {
            resource_logs: vec![],
        });
        assert_eq!(router.get_signal_type(&logs_data), SignalType::Logs);

        // Test profiles signal detection
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

    #[test]
    fn test_signal_detectable_trait() {
        // Test that SignalDetectable trait works directly on OTLPData
        let traces_data = OTLPData::Traces(ExportTraceServiceRequest {
            resource_spans: vec![],
        });
        assert_eq!(traces_data.signal_type(), SignalType::Traces);

        let metrics_data = OTLPData::Metrics(ExportMetricsServiceRequest {
            resource_metrics: vec![],
        });
        assert_eq!(metrics_data.signal_type(), SignalType::Metrics);

        let logs_data = OTLPData::Logs(ExportLogsServiceRequest {
            resource_logs: vec![],
        });
        assert_eq!(logs_data.signal_type(), SignalType::Logs);

        let profiles_data = OTLPData::Profiles(ExportProfilesServiceRequest {
            resource_profiles: vec![],
        });
        // Profiles currently falls back to Logs signal type
        assert_eq!(profiles_data.signal_type(), SignalType::Logs);
    }

    #[test]
    fn test_signal_detectable_generic() {
        // Test that we can use SignalDetectable as a generic constraint
        fn get_type_from_detectable<T: SignalDetectable>(data: &T) -> SignalType {
            data.signal_type()
        }

        let traces_data = OTLPData::Traces(ExportTraceServiceRequest {
            resource_spans: vec![],
        });
        assert_eq!(get_type_from_detectable(&traces_data), SignalType::Traces);
    }
}
