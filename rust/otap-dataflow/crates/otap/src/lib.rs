// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// gRPC service implementation
pub mod grpc;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;
/// Generated protobuf files
pub mod proto;

pub mod parquet_exporter;

pub mod perf_exporter;

/// testing utilities
#[cfg(test)]
mod mock;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAPData)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();
