// SPDX-License-Identifier: Apache-2.0

//! Expose the OTLP gRPC services.
//!
//! Provides a set of structs and enums that interact with the gRPC Server
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo Modify OTLPData -> Optimize message transport
use crate::proto::opentelemetry::collector::{
    logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse, logs_service_server::LogsService,
    },
    metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
        metrics_service_server::MetricsService,
    },
    profiles::v1development::{
        ExportProfilesServiceRequest, ExportProfilesServiceResponse,
        profiles_service_server::ProfilesService,
    },
    trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse, trace_service_server::TraceService,
    },
};

use otap_df_engine::shared::receiver as shared;
use tonic::{Request, Response, Status};

/// struct that implements the Log Service trait
pub struct LogsServiceImpl {
    effect_handler: shared::EffectHandler<OTLPData>,
}

impl LogsServiceImpl {
    /// Create a LogsServiceImpl with a sendable Effect Handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTLPData>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the Metric Service trait
pub struct MetricsServiceImpl {
    effect_handler: shared::EffectHandler<OTLPData>,
}

impl MetricsServiceImpl {
    /// Create a MetricsServiceImpl with a sendable Effect Handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTLPData>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceImpl {
    effect_handler: shared::EffectHandler<OTLPData>,
}

impl TraceServiceImpl {
    /// Create a TraceServiceImpl with a sendable Effect Handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTLPData>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the Profile Service trait
pub struct ProfilesServiceImpl {
    effect_handler: shared::EffectHandler<OTLPData>,
}

impl ProfilesServiceImpl {
    /// create a ProfileServiceImpl with a sendable Effect Handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTLPData>) -> Self {
        Self { effect_handler }
    }
}

#[tonic::async_trait]
impl LogsService for LogsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        _ = self
            .effect_handler
            .send_message(OTLPData::Logs(request.into_inner()))
            .await;
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        _ = self
            .effect_handler
            .send_message(OTLPData::Metrics(request.into_inner()))
            .await;
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceImpl {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        _ = self
            .effect_handler
            .send_message(OTLPData::Traces(request.into_inner()))
            .await;
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl ProfilesService for ProfilesServiceImpl {
    async fn export(
        &self,
        request: Request<ExportProfilesServiceRequest>,
    ) -> Result<Response<ExportProfilesServiceResponse>, Status> {
        _ = self
            .effect_handler
            .send_message(OTLPData::Profiles(request.into_inner()))
            .await;
        Ok(Response::new(ExportProfilesServiceResponse {
            partial_success: None,
        }))
    }
}

/// Enum to represent received OTLP requests.
#[derive(Debug, Clone)]
pub enum OTLPData {
    /// Logs Object
    Logs(ExportLogsServiceRequest),
    /// Metrics Object
    Metrics(ExportMetricsServiceRequest),
    /// Traces/Span Object
    Traces(ExportTraceServiceRequest),
    /// Profiles Object
    Profiles(ExportProfilesServiceRequest),
}

impl otap_df_traits::Retryable for OTLPData {
    fn id(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use prost::Message;
        
        let mut hasher = DefaultHasher::new();
        
        // Include enum variant discriminant for uniqueness across variants
        match self {
            OTLPData::Logs(request) => {
                "logs".hash(&mut hasher);
                request.encode_to_vec().hash(&mut hasher);
            }
            OTLPData::Metrics(request) => {
                "metrics".hash(&mut hasher);
                request.encode_to_vec().hash(&mut hasher);
            }
            OTLPData::Traces(request) => {
                "traces".hash(&mut hasher);
                request.encode_to_vec().hash(&mut hasher);
            }
            OTLPData::Profiles(request) => {
                "profiles".hash(&mut hasher);
                request.encode_to_vec().hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }
    
    fn deadline(&self) -> Option<std::time::Instant> {
        // OTLP messages don't have built-in deadlines
        // Could be extended to extract deadline from headers/metadata in the future
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_traits::Retryable;

    #[test]
    fn test_otlp_data_retryable_id_deterministic() {
        let logs_request = ExportLogsServiceRequest::default();
        let data1 = OTLPData::Logs(logs_request.clone());
        let data2 = OTLPData::Logs(logs_request);
        
        // Same data should produce same ID
        assert_eq!(data1.id(), data2.id());
    }
    
    #[test]
    fn test_otlp_data_retryable_id_unique_across_variants() {
        let logs_request = ExportLogsServiceRequest::default();
        let metrics_request = ExportMetricsServiceRequest::default();
        
        let logs_data = OTLPData::Logs(logs_request);
        let metrics_data = OTLPData::Metrics(metrics_request);
        
        // Different variants should produce different IDs
        assert_ne!(logs_data.id(), metrics_data.id());
    }
    
    #[test]
    fn test_otlp_data_retryable_deadline() {
        let logs_request = ExportLogsServiceRequest::default();
        let data = OTLPData::Logs(logs_request);
        
        // Should return None for deadline (not implemented yet)
        assert!(data.deadline().is_none());
    }
    
    #[test]
    fn test_otlp_data_retryable_trait_bounds() {
        // Test that OTLPData satisfies Retryable trait bounds
        fn requires_retryable<T: Retryable>(_: T) {}
        
        let logs_request = ExportLogsServiceRequest::default();
        let data = OTLPData::Logs(logs_request);
        
        requires_retryable(data);
    }
}
