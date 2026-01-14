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
    pub use tracing::callsite::{Callsite, DefaultCallsite};
    pub use tracing::field::ValueSet;
    pub use tracing::metadata::Kind;
    pub use tracing::{Event, Level};
    pub use tracing::{callsite2, debug, error, info, valueset, warn};
}

/// Macro for logging informational messages.
///
/// # Arguments:
/// - First argument (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_info;
/// otel_info!("receiver.start", version = "1.0.0");
/// ```
// TODO: Remove `name` attribute duplication in logging macros below once `tracing::Fmt` supports displaying `name`.
// See issue: https://github.com/tokio-rs/tracing/issues/2774
#[macro_export]
macro_rules! otel_info {
    ($name:expr $(,)?) => {
        $crate::_private::info!( name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::info!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
    };
}

/// Macro for logging warning messages.
///
/// # Arguments:
/// - First argument (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_warn;
/// otel_warn!("channel.full", dropped_count = 10);
/// ```
#[macro_export]
macro_rules! otel_warn {
    ($name:expr $(,)?) => {
        $crate::_private::warn!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {
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
/// # Arguments:
/// - First argument (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_debug;
/// otel_debug!("processing.batch", batch_size = 100);
/// ```
#[macro_export]
macro_rules! otel_debug {
    ($name:expr $(,)?) => {
        $crate::_private::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::_private::debug!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, $($key = $value),+, "");
    };
}

/// Macro for logging error messages.
///
/// # Arguments:
/// - First argument (required): The OpenTelemetry Event name identifying the log event.
///   See [OpenTelemetry Event name specification](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
/// - Additional optional key-value pairs can be passed as attributes.
///
/// # Example:
/// ```ignore
/// use otap_df_telemetry::otel_error;
/// otel_error!("export.failure", error_code = 500);
/// ```
#[macro_export]
macro_rules! otel_error {
    ($name:expr $(,)?) => {
        $crate::_private::error!(name: $name, target: env!("CARGO_PKG_NAME"), name = $name, "");
    };
    ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {
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

/// Create a subscriber that writes directly to console (bypassing channels).
fn raw_logging_subscriber() -> impl tracing::Subscriber {
    use crate::self_tracing::{ConsoleWriter, RawLoggingLayer};
    use tracing_subscriber::layer::SubscriberExt;

    tracing_subscriber::registry().with(RawLoggingLayer::new(ConsoleWriter::no_color()))
}

/// Execute a closure with a raw logging subscriber that writes directly to console.
#[inline]
pub fn with_raw_logging<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    tracing::subscriber::with_default(raw_logging_subscriber(), f)
}

/// Log an error message directly to stderr, bypassing the tracing dispatcher.
/// TODO: the way this is written it supports the full tracing syntax for
/// debug and display formatting of field values. The macros above should
/// be extended similarly.
#[macro_export]
macro_rules! raw_error {
    ($name:expr $(, $($fields:tt)*)?) => {{
        use $crate::self_tracing::{ConsoleWriter, RawLoggingLayer};
        use $crate::_private::Callsite;

        static __CALLSITE: $crate::_private::DefaultCallsite = $crate::_private::callsite2! {
            name: $name,
            kind: $crate::_private::Kind::EVENT,
            target: module_path!(),
            level: $crate::_private::Level::ERROR,
            fields: $($($fields)*)?
        };

        let meta = __CALLSITE.metadata();
        let layer = RawLoggingLayer::new(ConsoleWriter::no_color());

        // Use closure to extend valueset lifetime (same pattern as tracing::event!)
        (|valueset: $crate::_private::ValueSet<'_>| {
            let event = $crate::_private::Event::new(meta, &valueset);
            layer.dispatch_event(&event);
        })($crate::_private::valueset!(meta.fields(), $($($fields)*)?));
    }};
}
