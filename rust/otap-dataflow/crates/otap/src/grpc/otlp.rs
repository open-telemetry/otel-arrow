//! crate containing GRPC Server implementations for the OTLP services that
//! convert the received OTLP signals into OTAP

use std::convert::Infallible;
use std::sync::Arc;

use futures::future::BoxFuture;
use http::Request;
use tonic::body::Body;
use tonic::server::{NamedService, UnaryService};
use tonic::{Response, Status};

/// implementation of OTLP bytes -> OTAP GRPC server for logs
#[derive(Clone)]
pub struct LogsServiceServer<T: Clone> {
    /// TODO make this not pub
    pub inner: Arc<T>
}

impl<T> tower_service::Service<Request<Body>> for LogsServiceServer<T> where T: Clone {
    type Response = http::Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        todo!()
    }

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }
}

/// TODO maybe we should import this
pub const LOGS_SERVICE_NAME: &str = "opentelemetry.proto.collector.logs.v1.LogsService";

impl<T> NamedService for LogsServiceServer<T> where T: Clone {
    const NAME: &'static str = LOGS_SERVICE_NAME;
}