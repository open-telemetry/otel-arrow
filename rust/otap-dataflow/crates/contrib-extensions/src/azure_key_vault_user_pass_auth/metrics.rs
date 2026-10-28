// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup acquisition/publication telemetry, separate from database login.

use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::BackgroundProviderMetrics;

/// Lifecycle metrics recorded by the shared background provider.
#[metric_set(name = "extension.azure_key_vault_user_pass_auth")]
#[derive(Debug, Default, Clone)]
pub struct AzureKeyVaultUserPassAuthMetrics {
    /// Successful complete username/password acquisitions.
    #[metric(unit = "{acquisition}")]
    pub acquisition_successes: Counter<u64>,
    /// Failed acquisitions, without partial credential publication.
    #[metric(unit = "{acquisition}")]
    pub acquisition_failures: Counter<u64>,
    /// Complete credentials published to consumers.
    #[metric(unit = "{credential}")]
    pub credential_publishes: Counter<u64>,
    /// Successful complete acquisition duration.
    #[metric(unit = "ms")]
    pub acquisition_success_latency: Mmsc,
}

impl BackgroundProviderMetrics for AzureKeyVaultUserPassAuthMetrics {
    fn successes(&mut self) -> &mut Counter<u64> {
        &mut self.acquisition_successes
    }

    fn failures(&mut self) -> &mut Counter<u64> {
        &mut self.acquisition_failures
    }

    fn publishes(&mut self) -> &mut Counter<u64> {
        &mut self.credential_publishes
    }

    fn success_latency(&mut self) -> &mut Mmsc {
        &mut self.acquisition_success_latency
    }
}
