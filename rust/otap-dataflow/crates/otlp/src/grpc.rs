// SPDX-License-Identifier: Apache-2.0

//! Expose the OTLP gRPC services.
//!
//! Provides a set of structs and enums that interact with the gRPC Server
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo Modify OTLPData -> Optimize message transport
use otel_arrow_rust::proto::opentelemetry::{collector::{
    logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse, logs_service_server::LogsService
    },
    metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
        metrics_service_server::MetricsService
    },
    profiles::v1development::{
        ExportProfilesServiceRequest, ExportProfilesServiceResponse,
        profiles_service_server::ProfilesService
    },
    trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse, trace_service_server::TraceService
    }, logs::v1::LogsData, metrics::v1::MetricsData, trace::v1::TracesData}
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
            .send_message(OTLPData::Logs(LogsData::new(request.into_inner().resource_logs.clone())))
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
            .send_message(OTLPData::Metrics(MetricsData::new(request.into_inner().resource_metrics.clone())))
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
            .send_message(OTLPData::Traces(TracesData::new(request.into_inner().resource_spans.clone())))
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
    Logs(LogsData),
    /// Metrics Object
    Metrics(MetricsData),
    /// Traces/Span Object
    Traces(TracesData),
    /// Profiles Object
    Profiles(ExportProfilesServiceRequest),
}
