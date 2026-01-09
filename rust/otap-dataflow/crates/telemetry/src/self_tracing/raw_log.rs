// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Raw logging macros that bypass the tracing subscriber.
//!
//! These macros are used for logging when the subscriber infrastructure is not
//! available or may be in an inconsistent state (e.g., during shutdown, when
//! the channel is closed, or during early initialization).
//!
//! The macros temporarily install a dedicated subscriber that writes directly
//! to the console, then use standard tracing macros. This reuses the normal
//! tracing event creation and encoding path.

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
/// For reporting errors in the rrr rsr rs rsr rsr rs sr sr rs rs rs sr rs rs
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
