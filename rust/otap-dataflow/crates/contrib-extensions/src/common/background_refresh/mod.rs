// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable machinery for extensions with background fetching.

mod metrics;
mod provider;

#[cfg(test)]
mod tests;

pub use metrics::{BackgroundProviderMetrics, BackgroundProviderMetricsTracker};
pub use provider::{BackgroundProviderExtension, BackgroundProviderSource};
