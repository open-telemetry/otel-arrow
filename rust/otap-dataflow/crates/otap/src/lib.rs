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

/// Implementation of OTLP exporter that implements the exporter trait
pub mod otlp_exporter;

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

/// An error-exporter returns a static error.
pub mod error_exporter;

/// Experimental exporters and processors
#[cfg(any(feature = "experimental-exporters", feature = "kql-processor"))]
pub mod experimental;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(test)]
mod fixtures;

#[cfg(test)]
pub mod testing;

/// Signal-type router processor (OTAP-based)
pub mod signal_type_router;

/// Attributes processor (OTAP-based)
pub mod attributes_processor;
/// compression formats
pub mod compression;

pub mod metrics;

/// gRPC service implementation
pub mod otlp_grpc;

/// Cloud specific auth utilities
pub mod cloud_auth;

/// Object storage utilities including integrations for different cloud
/// providers
pub mod object_store;
/// TLS utilities
#[cfg(feature = "experimental-tls")]
pub mod tls_utils;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
