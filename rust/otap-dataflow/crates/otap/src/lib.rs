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

/// Batch processor
pub mod batch_processor;

// Retry processor that is aware of the OTAP PData/context.
pub mod retry_processor;

/// Receiver that reads in syslog data
pub mod syslog_cef_receiver;

/// Common component accessories (e.g., context-state management).
pub mod accessory;

pub mod pdata;

pub mod parquet_exporter;

pub mod perf_exporter;

pub mod fake_data_generator;

/// Implementation of debug processor that outputs received signals in a string format for user view
pub mod debug_processor;

pub mod filter_processor;

/// Implementation of a noop exporter that acts as a exporter placeholder
pub mod noop_exporter;

/// Fan-out processor to clone data to multiple downstream outputs.
pub mod fanout_processor;

/// An error-exporter returns a static error.
pub mod error_exporter;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(test)]
mod fixtures;

#[cfg(any(test, feature = "test-utils"))]
pub mod testing;

/// Signal-type router processor (OTAP-based)
pub mod signal_type_router;

/// Content-based router processor (routes by resource attribute value)
pub mod content_router;

/// Attributes processor (OTAP-based)
pub mod attributes_processor;

pub mod transform_processor;

/// compression formats
pub mod compression;

pub mod metrics;

pub(crate) mod socket_options;

/// Shared concurrency limiting across protocol servers
pub(crate) mod shared_concurrency;

/// gRPC service implementation
pub mod otlp_grpc;

/// OTLP/HTTP receiver support.
pub mod otlp_http;

/// Cloud specific auth utilities
pub mod cloud_auth;

/// Internal telemetry receiver
pub mod internal_telemetry_receiver;

pub mod topic_receiver;

pub mod topic_exporter;

/// Object storage utilities including integrations for different cloud
/// providers
pub mod object_store;
/// TLS utilities
#[cfg(feature = "experimental-tls")]
pub mod tls_utils;

/// Install the rustls crypto provider selected by the active feature flag.
///
/// Safe to call multiple times — subsequent calls are no-ops.
/// Panics if no crypto feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`) is enabled.
#[cfg(feature = "experimental-tls")]
pub fn install_crypto_provider() {
    let _ = try_install_crypto_provider();
}

/// Try to install the rustls crypto provider selected by the active feature flag.
///
/// Returns `Ok(())` on success or if already installed, `Err` on failure.
#[cfg(feature = "experimental-tls")]
pub fn try_install_crypto_provider() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "crypto-ring")]
    {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|e| format!("Failed to install ring crypto provider: {e:?}"))?;
    }
    #[cfg(feature = "crypto-aws-lc")]
    {
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|e| format!("Failed to install aws-lc-rs crypto provider: {e:?}"))?;
    }
    #[cfg(feature = "crypto-openssl")]
    {
        rustls_openssl::default_provider()
            .install_default()
            .map_err(|e| format!("Failed to install OpenSSL crypto provider: {e:?}"))?;
    }
    Ok(())
}

/// Console exporter similar using built-in OTLP-bytes formatting.
pub mod console_exporter;

/// Durable buffer processor for crash-resilient buffering via Quiver
pub mod durable_buffer_processor;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
