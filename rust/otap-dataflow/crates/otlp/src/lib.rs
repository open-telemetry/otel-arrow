// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

/// gRPC service implementation
pub mod grpc;
/// Generated protobuf files
pub mod grpc_stubs;


pub mod otlp_exporter;
pub mod mock;