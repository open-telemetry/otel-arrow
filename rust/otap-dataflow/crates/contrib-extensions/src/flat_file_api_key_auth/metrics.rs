// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the flat file API key authentication extension.

use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::BackgroundProviderMetrics;

/// Telemetry metrics for the flat file API key authentication extension.
#[metric_set(name = "extension.flat_file_api_key_auth")]
#[derive(Debug, Default, Clone)]
pub struct FlatFileApiKeyAuthMetrics {
    /// Number of successful API key acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_successes: Counter<u64>,
    /// Number of failed API key acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_failures: Counter<u64>,
    /// Number of API keys published to consumers.
    #[metric(unit = "{credential}")]
    pub auth_publishes: Counter<u64>,
    /// Latency of successful acquisitions in milliseconds.
    #[metric(unit = "ms")]
    pub auth_success_latency: Mmsc,
}

impl BackgroundProviderMetrics for FlatFileApiKeyAuthMetrics {
    fn successes(&mut self) -> &mut Counter<u64> {
        &mut self.auth_successes
    }

    fn failures(&mut self) -> &mut Counter<u64> {
        &mut self.auth_failures
    }

    fn publishes(&mut self) -> &mut Counter<u64> {
        &mut self.auth_publishes
    }

    fn success_latency(&mut self) -> &mut Mmsc {
        &mut self.auth_success_latency
    }
}
