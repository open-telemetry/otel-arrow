// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP HTTP wire types and endpoint paths shared by clients and servers.

use prost_types::Any;

pub mod server;

/// OTLP protobuf content type.
pub const PROTOBUF_CONTENT_TYPE: &str = "application/x-protobuf";

/// OTLP HTTP path for logs endpoint.
pub const LOGS_PATH: &str = "/v1/logs";

/// OTLP HTTP path for metrics endpoint.
pub const METRICS_PATH: &str = "/v1/metrics";

/// OTLP HTTP path for traces endpoint.
pub const TRACES_PATH: &str = "/v1/traces";

/// Google RPC status format for HTTP error responses.
///
/// Encodes status codes and messages in protobuf format for consistency with
/// gRPC error handling.
/// See: https://github.com/googleapis/googleapis/blob/master/google/rpc/status.proto
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcStatus {
    /// gRPC-compatible numeric status code.
    #[prost(int32, tag = "1")]
    pub code: i32,
    /// Human-readable status message.
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<Any>,
}
