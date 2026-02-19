// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;
use std::time::Duration;

use serde::Deserialize;

use crate::otlp_exporter::default_max_in_flight;
use crate::otlp_http::client_settings::HttpClientSettings;

/// Configuration for OTLP HTTP Exporter
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Configuration for the HTTP client that will be used by this exporter
    pub http: HttpClientSettings,

    pub endpoint: String,

    // TODO comments
    // TODO default
    pub client_pool_size: NonZeroUsize,

    /// Maximum number of concurrent in-flight export requests.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
}
