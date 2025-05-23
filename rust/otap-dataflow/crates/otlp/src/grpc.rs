// SPDX-License-Identifier: Apache-2.0

//!
//! Provides a set of structs and enums that interact with the gRPC Server
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo Modify OTLPData -> Optimize message transport
//!

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
use tonic::codec::CompressionEncoding;
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

/// Enum to represent various compression methods
#[derive(Debug)]
pub enum CompressionMethod {
    /// Fastest compression
    Zstd,
    /// Most compatible compression method
    Gzip,
    /// Used for legacy systems
    Deflate,
}

impl CompressionMethod {
    /// map the compression method to the proper tonic compression encoding equivalent
    /// use the CompressionMethod enum to abstract from tonic
    #[must_use]
    pub fn map_to_compression_encoding(&self) -> CompressionEncoding {
        match *self {
            CompressionMethod::Gzip => CompressionEncoding::Gzip,
            CompressionMethod::Zstd => CompressionEncoding::Zstd,
            CompressionMethod::Deflate => CompressionEncoding::Deflate,
        }
    }
}
