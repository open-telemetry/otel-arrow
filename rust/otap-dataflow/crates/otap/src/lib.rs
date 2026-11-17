// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTAP/OTLP runtime, transport, and protocol infrastructure.

use crate::pdata::OtapPdata;
use otel_arrow_dfe_engine::{PipelineFactory, build_factory};
use otel_arrow_dfe_engine_macros::pipeline_factory;

pub mod grpc;
pub mod http;
pub mod otlp;
pub mod receiver;

mod defaults;

/// Common component accessories (e.g., context-state management).
pub mod accessory;

pub mod pdata;

mod pdata_conversions;

pub mod metrics;

mod bearer_authorization;

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

/// Shared ingress shedding based on receiver-local rate limits.
pub mod rate_limit_layer;

/// HTTP Client authentication provider.
pub mod http_client_auth;

/// Cloud specific auth utilities
pub mod cloud_auth;

/// Redacted string type for sensitive values.
pub mod opaque_string;

/// Object storage utilities including integrations for different cloud
/// providers
pub mod object_store;

/// Cryptographic provider initialization (see [`crypto::install_crypto_provider`]).
pub mod crypto;

/// Transport-header capture and propagation tests.
#[cfg(test)]
mod transport_headers;

/// TLS utilities
pub mod tls_utils;

/// Bounded hot-reload of client TLS material for OTLP exporters.
pub(crate) mod client_tls_reload;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata, pdata::materialize_context)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
