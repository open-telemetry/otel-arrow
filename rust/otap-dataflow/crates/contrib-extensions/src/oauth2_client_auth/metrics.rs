// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the OAuth 2.0 Client Auth extension.

use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry_macros::metric_set;

use crate::common::token_refresh::TokenProviderMetrics;

/// Telemetry metrics for the OAuth 2.0 Client Auth extension.
#[metric_set(name = "extension.oauth2_client_auth")]
#[derive(Debug, Default, Clone)]
pub struct OAuth2ClientAuthMetrics {
    /// Number of successful token acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_successes: Counter<u64>,
    /// Number of failed token acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_failures: Counter<u64>,
    /// Number of tokens published to consumers via the watch channel.
    #[metric(unit = "{token}")]
    pub auth_token_publish: Counter<u64>,
    /// Latency of successful acquisitions in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub auth_success_latency: Mmsc,
}

impl TokenProviderMetrics for OAuth2ClientAuthMetrics {
    fn successes(&mut self) -> &mut Counter<u64> {
        &mut self.auth_successes
    }

    fn failures(&mut self) -> &mut Counter<u64> {
        &mut self.auth_failures
    }

    fn publishes(&mut self) -> &mut Counter<u64> {
        &mut self.auth_token_publish
    }

    fn success_latency(&mut self) -> &mut Mmsc {
        &mut self.auth_success_latency
    }
}
