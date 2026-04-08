// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// gRPC service implementation
pub mod otap_grpc;

/// Common component accessories (e.g., context-state management).
pub mod accessory;

pub mod pdata;

mod pdata_conversions;

pub mod metrics;

/// Shared OTLP receiver metric definitions used by OTLP protocol support.
pub mod otlp_metrics;

/// testing utilities
#[cfg(any(test, feature = "test-utils"))]
pub mod otap_mock;
#[cfg(any(test, feature = "test-utils"))]
pub mod otlp_mock;

#[cfg(any(test, feature = "test-utils"))]
pub mod testing;

/// compression formats
pub mod compression;

pub(crate) mod socket_options;

/// Shared concurrency limiting across protocol servers
pub mod shared_concurrency;

/// Shared ingress shedding based on process-wide memory pressure.
pub mod memory_pressure_layer;

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

/// Protocol-neutral transport header abstraction for end-to-end header
/// propagation through the pipeline.
pub mod transport_headers;

/// TLS utilities
#[cfg(feature = "experimental-tls")]
pub mod tls_utils;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
