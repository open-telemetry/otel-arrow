// SPDX-License-Identifier: Apache-2.0

//! Errors for the telemetry system.

use crate::descriptor::MetricsDescriptor;

/// All errors that can occur in the telemetry system.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The metrics key was not set before use.
    /// This should never happen if the metrics are registered properly.
    #[error("Metrics not registered before use ({descriptor:?})")]
    MetricsNotRegistered {
        /// The metric descriptor
        descriptor: &'static MetricsDescriptor,
    },
}