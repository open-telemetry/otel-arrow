// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public admin API and SDK for the OTAP dataflow engine.

mod endpoint;
mod error;

#[cfg(feature = "http-client")]
mod client;
#[cfg(feature = "http-client")]
mod http_backend;

pub use otap_df_admin_types::{engine, operations, pipeline_groups, pipelines, telemetry};
pub use otap_df_config as config;

#[cfg(feature = "http-client")]
pub use crate::client::{
    AdminClient, AdminClientBuilder, EngineClient, HttpAdminClientSettings, PipelineGroupsClient,
    PipelinesClient, TelemetryClient,
};
pub use crate::endpoint::{AdminAuth, AdminEndpoint, AdminScheme};
pub use crate::error::{EndpointError, Error};
