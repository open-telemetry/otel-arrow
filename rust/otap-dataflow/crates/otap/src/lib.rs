// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// OTAP node components organized by type (receiver, processor, exporter)
pub mod nodes;

/// Contributed components organized by type
#[cfg(any(feature = "contrib-exporters", feature = "contrib-processors"))]
pub mod contrib;

// Re-export node components at the top level for backward compatibility
pub use nodes::exporter::error_exporter;
pub use nodes::exporter::noop_exporter;
pub use nodes::exporter::otap_exporter;
pub use nodes::exporter::otlp_exporter;
pub use nodes::exporter::parquet_exporter;
pub use nodes::exporter::perf_exporter;

pub use nodes::processor::attributes_processor;
pub use nodes::processor::batch_processor;
pub use nodes::processor::debug_processor;
pub use nodes::processor::filter_processor;
pub use nodes::processor::retry_processor;
pub use nodes::processor::signal_type_router;
pub use nodes::processor::transform_processor;

pub use nodes::receiver::fake_data_generator;
pub use nodes::receiver::otap_receiver;
pub use nodes::receiver::otlp_receiver;
pub use nodes::receiver::syslog_cef_receiver;

// Re-export contrib components for backward compatibility
#[cfg(feature = "azure-monitor-exporter")]
pub use contrib::exporter::azure_monitor_exporter;
#[cfg(feature = "geneva-exporter")]
pub use contrib::exporter::geneva_exporter;
#[cfg(feature = "condense-attributes-processor")]
pub use contrib::processor::condense_attributes_processor;
#[cfg(feature = "recordset-kql-processor")]
pub use contrib::processor::recordset_kql_processor;

/// Common component accessories (e.g., context-state management).
pub mod accessory;

pub mod pdata;

/// gRPC service implementation
pub mod otap_grpc;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(test)]
mod fixtures;

#[cfg(test)]
pub mod testing;

/// validation process to verify that encoding/decoding works properly with otlp request
#[cfg(test)]
pub mod validation;

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

/// Object storage utilities including integrations for different cloud
/// providers
pub mod object_store;
/// TLS utilities
#[cfg(feature = "experimental-tls")]
pub mod tls_utils;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
