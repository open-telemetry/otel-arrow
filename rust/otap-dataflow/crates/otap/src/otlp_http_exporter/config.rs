// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;

use serde::Deserialize;

use crate::otlp_exporter::default_max_in_flight;
use crate::otlp_http::client_settings::HttpClientSettings;

/// Configuration for OTLP HTTP Exporter
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Configuration for the HTTP client that will be used by this exporter
    pub http: HttpClientSettings,

    /// The endpoint to which the exporter will send OTLP HTTP requests. This should include the
    /// scheme, host and port, but not the paths (/v1/logs) as these will be appended to requests
    /// automatically for each batch of signals depending on the signal type.
    ///
    /// Example: "http://localhost:4318" or "https://otel-collector:4318"
    pub endpoint: String,

    /// The size of the pool of HTTP clients to use for sending export requests. This allows for
    /// multiple concurrent connections to the OTLP server, which can help with load balancing when
    /// there are multiple receiver instances running on the same port (using SO_REUSEPORT).
    pub client_pool_size: NonZeroUsize,

    /// Maximum number of concurrent in-flight export requests.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
}
