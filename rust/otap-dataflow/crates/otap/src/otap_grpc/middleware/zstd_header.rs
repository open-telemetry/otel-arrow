// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Middlewares for adapting gRPC encoding request header

use async_trait::async_trait;
use http::{HeaderName, HeaderValue, Request, Response};
use tonic::body::Body;
use tonic_middleware::{Middleware, ServiceBound};

static ENCODING_HEADER: HeaderName = HeaderName::from_static("grpc-encoding");
static ZSTD_HEADER_VALUE: HeaderValue = HeaderValue::from_static("zstd");

/// Tonic middleware implementation that will replace the `grpc-encoding` header that the golang
/// exporter produces (which has the format zstdarrow[0-9]) with the value that tonic expects,
/// which is just "zstd"
#[derive(Clone, Default)]
pub struct ZstdRequestHeaderAdapter {}

#[async_trait]
impl<S> Middleware<S> for ZstdRequestHeaderAdapter
where
    S: ServiceBound,
    S::Future: Send,
{
    async fn call(
        &self,
        mut req: Request<Body>,
        mut service: S,
    ) -> Result<Response<Body>, S::Error> {
        let headers = req.headers_mut();
        if let Some(header_val) = headers.get(&ENCODING_HEADER) {
            let header_bytes = header_val.as_bytes();
            if header_bytes.starts_with(b"zstdarrow") {
                _ = headers.insert(&ENCODING_HEADER, ZSTD_HEADER_VALUE.clone());
            }
        }

        let result = service.call(req).await?;
        Ok(result)
    }
}
