// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// gRPC service implementation
pub mod otap_grpc;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;

/// This receiver receives OTLP bytes from the grpc service request and
/// produce for the pipeline OTAP PData
pub mod otlp_receiver;

/// Implementation of OTLP gRPC exporter that implements the exporter trait
pub mod otlp_grpc_exporter;

pub mod otlp_http_exporter;

/// Common component accessories (e.g., context-state management).
pub mod accessory;

pub mod pdata;

mod pdata_conversions;

pub mod metrics;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(any(test, feature = "test-utils"))]
pub mod testing;

/// compression formats
pub mod compression;

pub(crate) mod socket_options;

/// Shared concurrency limiting across protocol servers
pub(crate) mod shared_concurrency;

/// gRPC service implementation
pub mod otlp_grpc;

/// OTLP/HTTP receiver support.
pub mod otlp_http;

/// Cloud specific auth utilities
pub mod cloud_auth;

/// Object storage utilities including integrations for different cloud
/// providers
pub mod object_store;

/// Cryptographic provider initialization (see [`crypto::install_crypto_provider`]).
pub mod crypto;

/// TLS utilities
#[cfg(feature = "experimental-tls")]
pub mod tls_utils;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
