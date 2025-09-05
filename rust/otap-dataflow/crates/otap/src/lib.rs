// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// Request context associated with OTAP data.
pub mod context;

/// Code for encoding OTAP batch from pdata view
pub mod encoder;
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

// OTAP batch processor
pub mod otap_batch_processor;

// Retry processor that is aware of the OTAP PData/context.
pub mod retry_processor;

/// Receiver that reads in syslog data
pub mod syslog_cef_receiver;

/// Generated protobuf files
pub mod proto;

pub mod pdata;

pub mod parquet_exporter;

pub mod perf_exporter;

pub mod fake_data_generator;

/// Implementation of debug processor that outputs received signals in a string format for user view
pub mod debug_processor;

/// Implementation of a noop exporter that acts as a exporter placeholder
pub mod noop_exporter;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(test)]
mod fixtures;

/// Signal-type router processor (OTAP-based)
pub mod signal_type_router;

/// Attributes processor (OTAP-based)
pub mod attributes_processor;
/// compression formats
pub mod compression;
mod metrics;
/// gRPC service implementation
pub mod otlp_grpc;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();
