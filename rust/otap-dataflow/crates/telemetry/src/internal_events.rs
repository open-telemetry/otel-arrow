// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logging macros for the OTAP Dataflow Engine.
//!
//! This module provides logging macros (`otel_info!`, `otel_warn!`, `otel_debug!`, and `otel_error!`)
//! for internal use within the OTAP engine codebase. These macros wrap the `tracing` crate's
//! logging functionality and are intended for instrumenting engine internals, receivers,
//! processors, exporters, and other pipeline components.

#![allow(unused_macros)]

#[doc(hidden)]
pub mod _private {
    pub use tracing::{debug, error, info, warn};
}

/// Macro for logging informational messages.
///
/// # Fields:
/// - `name` (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_info;
/// otel_info!(name: "receiver_start", version = "1.0.0");
/// ```
// TODO: Remove `name` attribute duplication in logging macros below once `tracing::Fmt` supports displaying `name`.
// See issue: https://github.com/tokio-rs/tracing/issues/2774
#[macro_export]
macro_rules! otel_info {
    (name: $name:expr $(,)?) => {
        $crate::_private::info!( name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::info!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
    };
}

/// Macro for logging warning messages.
///
/// # Fields:
/// - `name` (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_warn;
/// otel_warn!(name: "channel_full", dropped_count = 10);
/// ```
#[macro_export]
macro_rules! otel_warn {
    (name: $name:expr $(,)?) => {
        $crate::_private::warn!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::warn!(name: $name,
                        target: env!("CARGO_PKG_NAME"),
                        name = $name,
                        $($key = {
                                $value
                        }),+,
                        ""
                )
    };
}

/// Macro for logging debug messages.
///
/// # Fields:
/// - `name` (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_debug;
/// otel_debug!(name: "processing_batch", batch_size = 100);
/// ```
#[macro_export]
macro_rules! otel_debug {
    (name: $name:expr $(,)?) => {
        $crate::_private::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
    };
}

/// Macro for logging error messages.
///
/// # Fields:
/// - `name` (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_error;
/// otel_error!(name: "export_failure", error_code = 500);
/// ```
#[macro_export]
macro_rules! otel_error {
    (name: $name:expr $(,)?) => {
        $crate::_private::error!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    (name: $name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::error!(name: $name,
                        target: env!("CARGO_PKG_NAME"),
                        name = $name,
                        $($key = {
                                $value
                        }),+,
                        ""
                )
    };
}
