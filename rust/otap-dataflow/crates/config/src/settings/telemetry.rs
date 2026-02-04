// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-internal telemetry settings.
//!
//! This module contains configuration for internal logging and metrics.
//!
//! Note these types are for engine-specific internal telemetry configuration.
//!
//! The settings in service::telemetry are OpenTelemetry-SDK specific, the path
//! taken from the Collector's configuration model (which has a service::pipelines
//! nearby), and based on the OpenTelemetry decolarative configuration model.

pub mod logs;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for pipeline-internal telemetry capture.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelemetrySettings {
    /// Enable capture of per-pipeline internal metrics.
    ///
    /// When disabled, the engine does not update or report the `pipeline.metrics` metric set.
    #[serde(default = "default_true")]
    pub pipeline_metrics: bool,

    /// Enable capture of Tokio runtime internal metrics.
    ///
    /// When disabled, the engine does not update or report the `tokio.runtime.metrics` metric set.
    #[serde(default = "default_true")]
    pub tokio_metrics: bool,

    /// Enable capture of channel-level metrics.
    ///
    /// When disabled, the engine does not report channel sender/receiver metrics.
    #[serde(default = "default_true")]
    pub channel_metrics: bool,
}

const fn default_true() -> bool {
    true
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            pipeline_metrics: true,
            tokio_metrics: true,
            channel_metrics: true,
        }
    }
}
