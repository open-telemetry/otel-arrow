// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal prost/tonic bindings for Splunk STEF's `destination.proto`.

use http::uri::PathAndQuery;
use std::marker::PhantomData;
use tonic::body::Body;
use tonic::client::{Grpc, GrpcService};
use tonic::codec::CompressionEncoding;
use tonic::{GrpcMethod, Status};

pub const STEF_DESTINATION_SERVICE_NAME: &str = "STEFDestination";
pub const STEF_DESTINATION_STREAM_PATH: &str = "/STEFDestination/Stream";

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefClientMessage {
    #[prost(message, optional, tag = "1")]
    pub first_message: Option<StefClientFirstMessage>,
    #[prost(bytes = "vec", tag = "2")]
    pub stef_bytes: Vec<u8>,
    #[prost(bool, tag = "3")]
    pub is_end_of_chunk: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefClientFirstMessage {
    #[prost(string, tag = "1")]
    pub root_struct_name: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefDestinationCapabilities {
    #[prost(message, optional, tag = "1")]
    pub dictionary_limits: Option<StefDictionaryLimits>,
    #[prost(bytes = "vec", tag = "2")]
    pub schema: Vec<u8>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefDictionaryLimits {
    #[prost(uint64, tag = "2")]
    pub max_dict_bytes: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefServerMessage {
    #[prost(oneof = "stef_server_message::Message", tags = "1, 2")]
    pub message: Option<stef_server_message::Message>,
}

pub mod stef_server_message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Message {
        #[prost(message, tag = "1")]
        Capabilities(super::StefDestinationCapabilities),
        #[prost(message, tag = "2")]
        Response(super::StefDataResponse),
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefDataResponse {
    #[prost(uint64, tag = "1")]
    pub ack_record_id: u64,
    #[prost(message, repeated, tag = "2")]
    pub bad_data_record_id_ranges: Vec<StefIdRange>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StefIdRange {
    #[prost(uint64, tag = "1")]
    pub from_id: u64,
    #[prost(uint64, tag = "2")]
    pub to_id: u64,
}

#[derive(Debug, Clone)]
pub struct StefDestinationClient<T> {
    inner: Grpc<T>,
    _pd: PhantomData<T>,
}

impl<T> StefDestinationClient<T>
where
    T: GrpcService<Body>,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T::ResponseBody: Send + 'static,
    <T::ResponseBody as tonic::transport::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner: Grpc::new(inner),
            _pd: PhantomData,
        }
    }

    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.send_compressed(encoding);
        self
    }

    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.accept_compressed(encoding);
        self
    }

    pub async fn stream(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = StefClientMessage>,
    ) -> Result<tonic::Response<tonic::codec::Streaming<StefServerMessage>>, Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| Status::unknown(format!("service was not ready: {}", e.into())))?;

        let codec = tonic_prost::ProstCodec::default();
        let path = PathAndQuery::from_static(STEF_DESTINATION_STREAM_PATH);
        let mut req = request.into_streaming_request();
        _ = req
            .extensions_mut()
            .insert(GrpcMethod::new(STEF_DESTINATION_SERVICE_NAME, "Stream"));
        self.inner.streaming(req, path, codec).await
    }
}
