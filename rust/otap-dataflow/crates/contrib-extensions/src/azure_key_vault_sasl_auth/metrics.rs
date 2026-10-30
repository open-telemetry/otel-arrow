// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lifecycle metrics for startup-only SASL credential publication.

use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::BackgroundProviderMetrics;

/// Lifecycle metrics recorded by the shared background provider.
#[metric_set(name = "extension.azure_key_vault_sasl_auth")]
#[derive(Debug, Default, Clone)]
pub struct AzureKeyVaultSaslAuthMetrics {
    /// Successful complete username/password acquisitions.
    #[metric(unit = "{acquisition}")]
    pub acquisition_successes: Counter<u64>,
    /// Failed complete acquisitions, with no partial credential publication.
    #[metric(unit = "{acquisition}")]
    pub acquisition_failures: Counter<u64>,
    /// Complete credentials published to consumers.
    #[metric(unit = "{credential}")]
    pub credential_publishes: Counter<u64>,
    /// Successful complete acquisition duration.
    #[metric(unit = "ms")]
    pub acquisition_success_latency: Mmsc,
}

impl BackgroundProviderMetrics for AzureKeyVaultSaslAuthMetrics {
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
