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
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::_private::info!(name: $name, target: env!("CARGO_PKG_NAME"), { $($($fields)*)? });
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
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::_private::warn!(name: $name, target: env!("CARGO_PKG_NAME"), { $($($fields)*)? });
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
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::_private::debug!(name: $name, target: env!("CARGO_PKG_NAME"), { $($($fields)*)? });
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
    ($name:expr $(, $($fields:tt)*)?) => {
        $crate::_private::error!(name: $name, target: env!("CARGO_PKG_NAME"), { $($($fields)*)? });
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
        let record = $crate::__log_record_impl!($crate::_private::Level::ERROR, $name $(, $($fields)*)?);
        ConsoleWriter::no_color().print_log_record(now, &record);
    }};
}

/// Internal macro that constructs a `LogRecord` from a static callsite.
#[doc(hidden)]
#[macro_export]
macro_rules! __log_record_impl {
    ($level:expr, $name:expr $(, $($fields:tt)*)?) => {{
        use $crate::_private::Callsite;
        use $crate::self_tracing::LogRecord;

        static __CALLSITE: $crate::_private::DefaultCallsite = $crate::_private::callsite2! {
            name: $name,
            kind: $crate::_private::Kind::EVENT,
            target: env!("CARGO_PKG_NAME"),
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

#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn test_raw_error() {
        let err = Error::ConfigurationError("bad config".into());
        raw_error!("raw error message", error = ?err);
        raw_error!("simple error message");
    }
}
