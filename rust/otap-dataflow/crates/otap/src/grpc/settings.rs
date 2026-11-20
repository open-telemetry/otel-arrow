// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration and validation for gRPC clients and servers.

mod client;
mod server;

pub use client::{GrpcClientSettings, GrpcEndpointError, StartupCheck, validate_grpc_metadata_key};
pub use server::{GrpcServerSettings, tune_max_concurrent_requests};
