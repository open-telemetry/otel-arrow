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
///
/// TODO: Update to use valueset! for full `tracing` syntax, see raw_error!
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
///
/// TODO: Update to use valueset! for full `tracing` syntax, see raw_error!
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
///
/// TODO: Update to use valueset! for full `tracing` syntax, see raw_error!
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

/// Log an error message directly to stderr, bypassing the tracing dispatcher.
///
/// Note! the way this is written, it supports the full `tracing` syntax for
/// debug and display formatting of field values, following tracing::valueset!
/// where ? signifies debug and % signifies display.
///
/// ```ignore
/// use otap_df_telemetry::raw_error;
/// raw_error!("logging.write.failed", error = ?err, thing = %display);
/// ```
#[macro_export]
macro_rules! raw_error {
    ($name:expr $(, $($fields:tt)*)?) => {{
        use $crate::self_tracing::ConsoleWriter;
        let now = std::time::SystemTime::now();
        let record = $crate::error_event!($name $(, $($fields)*)?);
        ConsoleWriter::no_color().print_log_record(now, &record);
    }};
}

/// Internal macro that constructs a `LogRecord` from a static callsite.
/// This is shared by the level-specific record macros.
#[doc(hidden)]
#[macro_export]
macro_rules! __log_record_impl {
    ($level:expr, $name:expr $(, $($fields:tt)*)?) => {{
        use $crate::_private::Callsite;
        use $crate::self_tracing::LogRecord;

        static __CALLSITE: $crate::_private::DefaultCallsite = $crate::_private::callsite2! {
            name: $name,
            kind: $crate::_private::Kind::EVENT,
            target: module_path!(),
            level: $level,
            fields: $($($fields)*)?
        };

        let meta = __CALLSITE.metadata();

        // Use closure to extend valueset lifetime (same pattern as tracing::event!)
        (|valueset: $crate::_private::ValueSet<'_>| {
            let event = $crate::_private::Event::new(meta, &valueset);
            LogRecord::new(&event)
        })($crate::_private::valueset!(meta.fields(), $($($fields)*)?))
    }};
}

/// Construct an INFO-level `LogRecord` without dispatching it.
///
/// Returns a `LogRecord` that can be used in `EventMessage::Log`.
#[macro_export]
macro_rules! info_event {
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::__log_record_impl!($crate::_private::Level::INFO, $name $(, $($fields)*)?)
    };
}

/// Construct an ERROR-level `LogRecord` without dispatching it.
///
/// Returns a `LogRecord` that can be used in `EventMessage::Log`.
#[macro_export]
macro_rules! error_event {
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::__log_record_impl!($crate::_private::Level::ERROR, $name $(, $($fields)*)?)
    };
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use tracing::Level;

    #[test]
    fn test_raw_error() {
        let err = Error::ConfigurationError("bad config".into());
        raw_error!("raw error message", error = ?err);
        raw_error!("simple error message");
    }

    #[test]
    fn test_log_record_macros() {
        // Test info_event! with attributes
        let record = info_event!("test.event", count = 42i64, name = "test");
        let callsite = record.callsite();
        assert_eq!(*callsite.level(), Level::INFO);
        assert_eq!(callsite.name(), "test.event");

        // Test the formatted output contains body and attributes
        let formatted = record.format();
        assert!(formatted.contains("count=42"), "got: {}", formatted);
        assert!(formatted.contains("name=test"), "got: {}", formatted);

        // Test error_event!
        let err = Error::ConfigurationError("bad config".into());
        let record = error_event!("error.event", error = ?err); // I AM LINE 226
        assert_eq!(*record.callsite().level(), Level::ERROR);
        assert_eq!(record.callsite().line(), Some(226u32));
    }
}
