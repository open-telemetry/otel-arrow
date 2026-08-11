// Copyright The OpenTelemetry Authors
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

    /// The metrics channel was closed unexpectedly.
    #[error("Metrics channel was closed")]
    MetricsChannelClosed,

    /// The metrics collector is not running, so it cannot acknowledge a flush barrier.
    #[error("Metrics collector is not running")]
    MetricsCollectorNotRunning,

    /// More than one task attempted to consume the internal metrics channel.
    #[error("Metrics collector is already running")]
    MetricsCollectorAlreadyRunning,

    /// Error during shutdown of a component.
    #[error("Shutdown error: {0}")]
    ShutdownError(String),

    /// Error during configuration of a component.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
