// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! State stores

pub mod conditions;
pub mod error;
pub mod phase;
mod pipeline_rt_status;
pub mod pipeline_status;
pub mod reporter;
pub mod store;

// Re-export event types from telemetry crate.
pub use otap_df_telemetry::event;
pub use otap_df_telemetry::event::{DeployedPipelineKey, PipelineKey};

/// Type alias for CPU core identifier.
pub(crate) type CoreId = usize;
