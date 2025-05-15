// SPDX-License-Identifier: Apache-2.0

//!
//! Defines the necessary service traits that could be used in a test gRPC server to confirm client activity
//!
//! Uses a tokio channel to confirm that the gRPC server has received data from a client
//!

use crate::grpc::OTLPData;
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
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};

/// struct that implements the Log Service trait
pub struct LogsServiceMock {
    sender: Sender<OTLPData>,
}

impl LogsServiceMock {
    /// creates a new mock logs service
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Metrics Service trait
pub struct MetricsServiceMock {
    sender: Sender<OTLPData>,
}

impl MetricsServiceMock {
    /// creates a new mock metrics service
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Trace Service trait
pub struct TraceServiceMock {
    sender: Sender<OTLPData>,
}

impl TraceServiceMock {
    /// creates a new mock trace service
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the Profiles Service trait
pub struct ProfilesServiceMock {
    sender: Sender<OTLPData>,
}

impl ProfilesServiceMock {
    /// creates a new mock profiles service
    pub fn new(sender: Sender<OTLPData>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl LogsService for LogsServiceMock {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Logs(request.into_inner()))
            .await
            .expect("Logs failed to be sent through channel");
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceMock {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Metrics(request.into_inner()))
            .await
            .expect("Metrics failed to be sent through channel");
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceMock {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Traces(request.into_inner()))
            .await
            .expect("Traces failed to be sent through channel");
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl ProfilesService for ProfilesServiceMock {
    async fn export(
        &self,
        request: Request<ExportProfilesServiceRequest>,
    ) -> Result<Response<ExportProfilesServiceResponse>, Status> {
        self.sender
            .send(OTLPData::Profiles(request.into_inner()))
            .await
            .expect("Profiles failed to be sent through channel");
        Ok(Response::new(ExportProfilesServiceResponse {
            partial_success: None,
        }))
    }
}
