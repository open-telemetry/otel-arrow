// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Raw logging macros that bypass the tracing subscriber and write to
//! the console. A single `raw_error!(...)` API is provided.

use super::formatter::RawLoggingLayer;
use tracing_subscriber::prelude::*;

/// Create a subscriber that writes directly to console (bypassing channels).
fn raw_logging_subscriber() -> impl tracing::Subscriber {
    tracing_subscriber::registry().with(RawLoggingLayer::new(super::ConsoleWriter::no_color()))
}

/// Execute a closure with a raw logging subscriber that writes directly to console.
#[inline]
pub fn with_raw_logging<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    tracing::subscriber::with_default(raw_logging_subscriber(), f)
}

/// Log an error message directly to stderr, bypassing the tracing subscriber.
///
/// This should be used sparingly, only emergencies! This is a good
/// configuration for diagnosing internal other logging facilities,
/// because it is unbuffered and uses a dedicated
///
///
/// # Example
///
/// ```ignore
/// use otap_df_telemetry::raw_error;
/// raw_error!("Connection failed: {}", error);
/// ```
#[macro_export]
macro_rules! raw_error {
    ($($arg:tt)+) => {
        $crate::self_tracing::raw_log::with_raw_logging(|| {
            ::tracing::error!($($arg)+)
        })
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_raw_error() {
        raw_error!("test error message");
        raw_error!("test error with arg: {}", 42);
    }
}
