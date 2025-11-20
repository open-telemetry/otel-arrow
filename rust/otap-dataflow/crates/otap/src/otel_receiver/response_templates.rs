use http::{HeaderValue, Response, StatusCode as HttpStatusCode};

use crate::compression::CompressionMethod;
use crate::otel_receiver::grpc::{self, grpc_encoding_token};

/// Prebuilt response headers for successful unary responses.
pub struct ResponseTemplates {
    pub ok_plain: Response<()>,
    pub ok_encoded: Vec<(CompressionMethod, Response<()>)>,
}

impl ResponseTemplates {
    pub fn new(accept_header: HeaderValue) -> Self {
        let ok_plain = build_ok_response(accept_header.clone(), None);
        Self {
            ok_plain,
            ok_encoded: Vec::new(),
        }
    }

    pub fn get_ok(&self, method: Option<CompressionMethod>) -> Option<Response<()>> {
        match method {
            None => Some(self.ok_plain.clone()),
            Some(method) => self
                .ok_encoded
                .iter()
                .find(|(m, _)| *m == method)
                .map(|(_, resp)| resp.clone()),
        }
    }

    pub fn with_method(mut self, method: CompressionMethod, accept_header: &HeaderValue) -> Self {
        let encoding = match method {
            CompressionMethod::Zstd => grpc::GrpcEncoding::Zstd,
            CompressionMethod::Gzip => grpc::GrpcEncoding::Gzip,
            CompressionMethod::Deflate => grpc::GrpcEncoding::Deflate,
        };
        if let Some(token) = grpc_encoding_token(encoding) {
            let encoded = build_ok_response(
                accept_header.clone(),
                Some(HeaderValue::from_static(token)),
            );
            self.ok_encoded.push((method, encoded));
        }
        self
    }
}

fn build_ok_response(
    accept_header: HeaderValue,
    encoding_token: Option<HeaderValue>,
) -> Response<()> {
    let mut builder = Response::builder()
        .status(HttpStatusCode::OK)
        .header("content-type", "application/grpc")
        .header("grpc-accept-encoding", accept_header);
    if let Some(token) = encoding_token {
        builder = builder.header("grpc-encoding", token);
    }
    builder.body(()).expect("response build must succeed")
}
