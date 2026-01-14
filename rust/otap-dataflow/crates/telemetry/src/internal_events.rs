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
    pub use tracing_core;
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
///
/// This macro creates a real tracing Event with proper Metadata, then dispatches
/// it directly to `RawLoggingLayer::dispatch_event`, bypassing the global
/// dispatcher. This is safe to call from within tracing subscriber callbacks
/// (e.g., `on_event`) where using `tracing::subscriber::with_default` would
/// cause a RefCell panic.
///
/// Output format matches the standard log format:
/// `2026-01-06T10:30:45.123Z  ERROR  target::name: message [key=value, ...]`
#[macro_export]
macro_rules! raw_error {
    ($name:expr $(,)?) => {{
        use $crate::self_tracing::{ConsoleWriter, RawLoggingLayer};
        use $crate::_private::tracing_core::{Event, Metadata, Level, field::FieldSet, callsite::DefaultCallsite};

        static CALLSITE: DefaultCallsite = DefaultCallsite::new(&META);
        static META: Metadata<'static> = Metadata::new(
            $name,
            env!("CARGO_PKG_NAME"),
            Level::ERROR,
            Some(file!()),
            Some(line!()),
            Some(env!("CARGO_PKG_NAME")),
            FieldSet::new(&[], $crate::_private::tracing_core::callsite::Identifier(&CALLSITE)),
            $crate::_private::tracing_core::metadata::Kind::EVENT,
        );

        let layer = RawLoggingLayer::new(ConsoleWriter::no_color());
        let valueset = META.fields().value_set(&[]);
        let event = Event::new(&META, &valueset);
        layer.dispatch_event(&event);
    }};
    ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        use $crate::self_tracing::{ConsoleWriter, RawLoggingLayer};
        use $crate::_private::tracing_core::{Event, Metadata, Level, field::FieldSet, callsite::DefaultCallsite};

        // Define field names as static strings
        static FIELD_NAMES: &[&str] = &[$(stringify!($key)),+];

        static CALLSITE: DefaultCallsite = DefaultCallsite::new(&META);
        static META: Metadata<'static> = Metadata::new(
            $name,
            env!("CARGO_PKG_NAME"),
            Level::ERROR,
            Some(file!()),
            Some(line!()),
            Some(env!("CARGO_PKG_NAME")),
            FieldSet::new(FIELD_NAMES, $crate::_private::tracing_core::callsite::Identifier(&CALLSITE)),
            $crate::_private::tracing_core::metadata::Kind::EVENT,
        );

        let layer = RawLoggingLayer::new(ConsoleWriter::no_color());

        // Bind values to extend their lifetimes - use Debug formatting
        $(
            let $key = format!("{:?}", $value);
        )+

        // Create fixed-size array of field-value pairs (the repetition creates N elements)
        let field_values = &[
            $((
                &META.fields().field(stringify!($key)).expect("field exists"),
                Some(&$key as &dyn $crate::_private::tracing_core::field::Value)
            )),+
        ];
        let valueset = META.fields().value_set(field_values);
        let event = Event::new(&META, &valueset);
        layer.dispatch_event(&event);
    }};
}
