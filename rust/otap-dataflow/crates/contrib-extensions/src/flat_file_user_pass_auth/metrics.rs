// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the flat file user pass extension.

use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::BackgroundProviderMetrics;

/// Telemetry metrics for the flat file user pass extension.
#[metric_set(name = "extension.flat_file_user_pass_auth")]
#[derive(Debug, Default, Clone)]
pub struct FlatFileUserPassAuthMetrics {
    /// Number of successful credential acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_successes: Counter<u64>,
    /// Number of failed credential acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_failures: Counter<u64>,
    /// Number of credentials published to consumers via the watch channel.
    #[metric(unit = "{credential}")]
    pub auth_publishes: Counter<u64>,
    /// Latency of successful acquisitions in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub auth_success_latency: Mmsc,
}

impl BackgroundProviderMetrics for FlatFileUserPassAuthMetrics {
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
