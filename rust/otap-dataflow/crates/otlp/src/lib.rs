// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

/// gRPC service implementation
pub mod grpc;
/// Generated protobuf files
pub mod proto;

/// otlp exporter implementation
pub mod otlp_exporter;

/// grpc mock server for testing
pub mod mock;
