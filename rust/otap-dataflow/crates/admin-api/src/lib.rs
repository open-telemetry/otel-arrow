// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public admin API and SDK for the OTAP dataflow engine.

mod endpoint;
mod error;

#[cfg(feature = "http-client")]
mod client;
#[cfg(feature = "http-client")]
mod http_backend;

pub use otel_arrow_dfe_admin_types::{engine, groups, operations, pipelines, telemetry};
pub use otel_arrow_dfe_config as config;

#[cfg(feature = "http-client")]
pub use crate::client::{
    AdminClient, AdminClientBuilder, EngineClient, GroupsClient, HttpAdminClientSettings,
    PipelinesClient, TelemetryClient,
};
pub use crate::endpoint::{AdminAuth, AdminEndpoint, AdminScheme};
pub use crate::error::{EndpointError, Error};
