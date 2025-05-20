// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

/// gRPC service implementation
pub mod grpc;
/// Implementation of OTLP Receiver that implements the receiver trait
// pub mod otlp_receiver;
/// Generated protobuf files
pub mod proto;

/// otlp exporter implementation
pub mod otlp_exporter;

/// grpc mock server for testing
pub mod mock;
