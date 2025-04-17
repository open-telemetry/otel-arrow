use grpc_stubs::proto::collector::logs::v1::logs_service_server::LogsService;
use grpc_stubs::proto::collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use grpc_stubs::proto::collector::metrics::v1::metrics_service_server::MetricsService;
use grpc_stubs::proto::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use grpc_stubs::proto::collector::trace::v1::trace_service_server::TraceService;
use grpc_stubs::proto::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use tonic::{Request, Response, Status};
use otap_df_engine::receiver::EffectHandler;

/// Expose the OTLP gRPC services.
/// See the build.rs file for more information.
// #[path = "../../grpc_stubs"]
pub mod grpc_stubs {
    #[path = ""]
    pub mod proto {
        #[path = ""]
        pub mod collector {
            #[path = ""]
            pub mod logs {
                #[allow(unused_qualifications)]
                #[allow(unused_results)]
                #[allow(clippy::enum_variant_names)]
                #[allow(rustdoc::invalid_html_tags)]
                #[path = "opentelemetry.proto.collector.logs.v1.rs"]
                pub mod v1;
            }
            #[path = ""]
            pub mod metrics {
                #[allow(unused_qualifications)]
                #[allow(unused_results)]
                #[allow(clippy::enum_variant_names)]
                #[allow(rustdoc::invalid_html_tags)]
                #[path = "opentelemetry.proto.collector.metrics.v1.rs"]
                pub mod v1;
            }
            #[path = ""]
            pub mod trace {
                #[allow(unused_qualifications)]
                #[allow(unused_results)]
                #[allow(clippy::enum_variant_names)]
                #[allow(rustdoc::invalid_html_tags)]
                #[path = "opentelemetry.proto.collector.trace.v1.rs"]
                pub mod v1;
            }
        }

        #[path = ""]
        pub mod logs {
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.logs.v1.rs"]
            pub mod v1;
        }

        #[path = ""]
        pub mod metrics {
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.metrics.v1.rs"]
            pub mod v1;
        }

        #[path = ""]
        pub mod trace {
            #[allow(rustdoc::invalid_html_tags)]
            #[path = "opentelemetry.proto.trace.v1.rs"]
            pub mod v1;
        }

        #[path = ""]
        pub mod common {
            #[allow(clippy::enum_variant_names)]
            #[path = "opentelemetry.proto.common.v1.rs"]
            pub mod v1;
        }

        #[path = ""]
        pub mod resource {
            #[path = "opentelemetry.proto.resource.v1.rs"]
            pub mod v1;
        }
    }
}


pub struct LogsServiceImpl {
    effect_handler: EffectHandler<OTLPRequest>,
}
pub struct MetricsServiceImpl {
    effect_handler: EffectHandler<OTLPRequest>,
}
pub struct TraceServiceImpl {
    effect_handler: EffectHandler<OTLPRequest>,
}

#[tonic::async_trait]
impl LogsService for LogsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        tokio_spawn local
        tokio::task::spawn_local(async move {
            self.effect_handler.send_message(OTLPRequest::Logs(request.into_inner())).await;
        }).await;
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
        tokio::task::spawn_local(async move {
            self.effect_handler.send_message(OTLPRequest::Metrics(request.into_inner())).await;
        }).await;
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
        tokio::task::spawn_local(async move {
            self.effect_handler.send_message(OTLPRequest::Traces(request.into_inner())).await;
        }).await;
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

// Enum to represent received OTLP requests.
#[derive(Debug)]
pub enum OTLPRequest {
    Logs(ExportLogsServiceRequest),
    Metrics(ExportMetricsServiceRequest),
    Traces(ExportTraceServiceRequest),
}

