// SPDX-License-Identifier: Apache-2.0
//! Implementation of the OTLP nodes (receiver, exporter, processor).

mod batch_processor;

/// gRPC service implementation
pub mod grpc;

/// Generated protobuf files
pub mod proto;
