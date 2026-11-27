// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable machinery for extensions with background fetching.

mod metrics;
mod provider;

#[cfg(test)]
mod tests;

pub use metrics::{BackgroundProviderMetrics, BackgroundProviderMetricsTracker};
#[allow(unused_imports)]
pub use provider::{
    BackgroundFetchOutcome, BackgroundProviderExtension, BackgroundProviderRefreshPolicy,
    BackgroundProviderSource,
};
