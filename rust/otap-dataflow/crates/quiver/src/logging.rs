// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Crate-internal logging macros that enforce durable event names.
//!
//! These macros wrap `tracing` to enforce a `name:` parameter (the
//! [OpenTelemetry Event name](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname))
//! as the first argument. This ensures all log events have stable, machine-readable
//! identifiers separate from human-readable messages.
//!
//! ## Note on Naming
//!
//! These macros use `otel_*` names to match the API of `otap_df_telemetry` macros,
//! providing a familiar interface for developers working across the codebase.
//! However, these are **crate-private** macros that do not depend on
//! `otap_df_telemetry` - they are simple wrappers around `tracing` with
//! enforced event names.
//!
//! ## Keeping in Sync with `otap_df_telemetry`
//!
//! The macro implementations here should stay aligned with
//! [`otap_df_telemetry::internal_events`](../../telemetry/src/internal_events.rs).
//! If the telemetry crate changes how it invokes `tracing` (e.g., field
//! wrapping, target strategy), apply the same changes here so that quiver
//! log output is consistent with the rest of the engine.
//!
//! ## Event Naming Convention
//!
//! Event names follow the [Events Guide] pattern:
//! `quiver.<entity>[.<thing>].<verb>`
//!
//! [Events Guide]: ../../docs/telemetry/events-guide.md
//!
//! Names describe the **operation** (verb), not the outcome. Use attributes for
//! outcome details (e.g., `reason`, `phase`, `scope`, `status`):
//!
//! - `reason`: Why the operation happened (`"expired"`, `"force_drop"`, `"corruption"`)
//! - `phase`: When it happened (`"deferred"`, `"scan"`, `"runtime"`)
//! - `scope`: What was affected (`"entry"`, `"slot"`)
//! - `status`: Outcome detail (`"stopped_incomplete"`)
//!
//! Verbs are drawn from the guide's recommended set (`init`, `stop`, `flush`,
//! `drop`, `backpressure`, `scan`, `replay`, `rotate`, `tick`, `load`, `decode`).
//!
//! The log level carries success vs failure (info = success, warn/error = failure).
//!
//! ## Message Convention
//!
//! Most events need **no message** — the event name, level, and structured
//! attributes are sufficient. Only add a `message` attribute when it provides
//! context that cannot be inferred from the other fields (e.g., consequences,
//! recovery actions, or remediation taken):
//!
//! ```ignore
//! // No message needed — event name + attrs say it all
//! otel_info!("quiver.wal.rotate", rotation_id = 3, rotated_file_count = 2);
//! otel_debug!("quiver.segment.flush", segment = 7, bytes_written = 4096);
//!
//! // Message adds valuable context (consequence / recovery action)
//! otel_warn!("quiver.wal.cursor.load",
//!     error = %e,
//!     error_type = "decode",
//!     reason = "decode_failed",
//!     message = "replaying from start, duplicates may occur",
//! );
//! ```
//!
//! Use `message = "..."` (named field) rather than a trailing string literal.
//!
//! ## Error Classification
//!
//! When logging an error with `error = %e`, add an `error_type` attribute with
//! a low-cardinality classifier for filtering and alerting:
//!
//! - `"io"` — filesystem or network I/O failures
//! - `"decode"` — deserialization or format-parsing failures
//! - `"config"` — configuration validation errors
//! - `"corruption"` — data integrity or CRC failures
//!
//! ```ignore
//! otel_error!("quiver.segment.flush",
//!     segment = seq.raw(),
//!     error = %e,
//!     error_type = "io",
//!     message = "data may only be recoverable via WAL replay",
//! );
//! ```
//!
//! ## Event Name Examples
//!
//! - `quiver.wal.replay` — all WAL replay events (level + fields distinguish outcomes)
//! - `quiver.segment.drop` — all segment removal (reason + phase attrs differentiate)
//! - `quiver.segment.scan` — startup segment scanning
//! - `quiver.wal.cursor.load` — cursor sidecar loading (reason attr for failure type)
//! - `quiver.wal.file.init` — WAL file open/seek (file_type attr for variant)
//! - `quiver.wal.drop.flush` — drop-time flush (reason attr for skip/failure cause)
//! - `quiver.subscriber.progress.load` — subscriber progress file loading
//!
//! ## Usage
//!
//! ```ignore
//! // Attributes only (preferred when event name is self-describing)
//! otel_info!("quiver.segment.drop", segment = 5, reason = "expired");
//!
//! // With error_type classifier
//! otel_warn!("quiver.wal.cursor.load",
//!     error = %e, error_type = "decode", reason = "decode_failed",
//!     message = "replaying from start, duplicates may occur",
//! );
//!
//! // Attributes only, no message
//! otel_debug!("quiver.wal.replay", status = "stopped_incomplete");
//! ```

/// Macro for logging informational messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or a `message = "..."` attribute.
///
/// # Example
/// ```ignore
/// otel_info!("quiver.engine.init", version = "1.0.0");
/// otel_info!("quiver.wal.rotate", rotation_id = 3, rotated_file_count = 2);
/// ```
#[allow(unused_macro_rules)]
macro_rules! otel_info {
    ($name:expr, $($fields:tt)+) => {
        tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
    };
    ($name:expr) => {
        tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), "");
    };
}

/// Macro for logging warning messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or a `message = "..."` attribute.
///
/// # Example
/// ```ignore
/// otel_warn!("quiver.wal.cursor.load",
///     error = %e, error_type = "io", reason = "read_failed",
///     message = "replaying from start, duplicates may occur",
/// );
/// otel_warn!("quiver.budget.backpressure", policy = "backpressure");
/// ```
#[allow(unused_macro_rules)]
macro_rules! otel_warn {
    ($name:expr, $($fields:tt)+) => {
        tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
    };
    ($name:expr) => {
        tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), "");
    };
}

/// Macro for logging error messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or a `message = "..."` attribute.
///
/// # Example
/// ```ignore
/// otel_error!("quiver.wal.replay",
///     error = %e, error_type = "corruption", reason = "corruption",
///     message = "stopping replay at corruption boundary",
/// );
/// ```
#[allow(unused_macro_rules)]
macro_rules! otel_error {
    ($name:expr, $($fields:tt)+) => {
        tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
    };
    ($name:expr) => {
        tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), "");
    };
}

/// Macro for logging debug messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or a `message = "..."` attribute.
///
/// # Example
/// ```ignore
/// otel_debug!("quiver.wal.replay", status = "stopped_incomplete");
/// otel_debug!("quiver.segment.flush", segment = 7, bytes_written = 4096);
/// ```
#[allow(unused_macro_rules)]
macro_rules! otel_debug {
    ($name:expr, $($fields:tt)+) => {
        tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
    };
    ($name:expr) => {
        tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), "");
    };
}

/// Macro for logging messages at a runtime-selectable severity level.
///
/// Accepts a [`tracing::Level`] value as its first argument so that the severity
/// can be determined at runtime (e.g., from a variable or computed expression),
/// without requiring a separate call site for each level.
///
/// # Arguments
/// - First argument (required): A [`tracing::Level`] value.
/// - Second argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or a `message = "..."` attribute.
///
/// # Example
/// ```ignore
/// let level = tracing::Level::WARN;
/// otel_log!(level, "quiver.wal.replay", status = "stopped_incomplete");
///
/// otel_log!(tracing::Level::INFO, "quiver.segment.drop", segment = 5, reason = "expired");
/// ```
#[allow(unused_macro_rules)]
// otel_log! may not be used yet in production code within this crate; suppress the lint.
#[allow(unused_macros)]
macro_rules! otel_log {
    ($level:expr, $name:expr, $($fields:tt)+) => {
        match $level {
            tracing::Level::TRACE => {
                tracing::trace!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
            }
            tracing::Level::DEBUG => {
                tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
            }
            tracing::Level::INFO => {
                tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
            }
            tracing::Level::WARN => {
                tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
            }
            _ => {
                tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
            }
        }
    };
    ($level:expr, $name:expr) => {
        match $level {
            tracing::Level::TRACE => {
                tracing::trace!(name: $name, target: env!("CARGO_PKG_NAME"), "");
            }
            tracing::Level::DEBUG => {
                tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), "");
            }
            tracing::Level::INFO => {
                tracing::info!(name: $name, target: env!("CARGO_PKG_NAME"), "");
            }
            tracing::Level::WARN => {
                tracing::warn!(name: $name, target: env!("CARGO_PKG_NAME"), "");
            }
            _ => {
                tracing::error!(name: $name, target: env!("CARGO_PKG_NAME"), "");
            }
        }
    };
}

// Make macros available within this crate only (no #[macro_export])
pub(crate) use otel_debug;
pub(crate) use otel_error;
pub(crate) use otel_info;
#[allow(unused_imports)]
pub(crate) use otel_log;
pub(crate) use otel_warn;

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_compile() {
        // Verify macros compile with various argument patterns.
        // Tests model best practice: use `message = "..."` (named field)
        // rather than trailing string literals.
        otel_info!("quiver.test.info_only");
        otel_info!("quiver.test.info_fields", key = 42);
        otel_info!(
            "quiver.test.info_fields_message",
            key = 42,
            message = "with context"
        );
        otel_info!("quiver.test.info_debug_format", key = ?vec![1, 2, 3]);
        otel_info!("quiver.test.info_display_format", key = %"display");
        otel_info!(
            "quiver.test.info_message_as_field",
            message = "human-readable message",
        );

        otel_warn!("quiver.test.warn_only");
        otel_warn!("quiver.test.warn_fields", a = 1, b = 2);
        otel_warn!("quiver.test.warn_message", message = "warning context");

        otel_error!("quiver.test.error_only");
        otel_error!("quiver.test.error_fields", error = %"some error", error_type = "io");

        otel_debug!("quiver.test.debug_only");
        otel_debug!("quiver.test.debug_fields", count = 100);
    }

    #[test]
    fn test_shorthand_field() {
        let replayed = 42;
        otel_info!("quiver.test.shorthand", replayed);
    }

    #[test]
    fn test_otel_log_runtime_level() {
        // Verify otel_log! compiles and dispatches correctly with runtime levels.
        for level in [
            tracing::Level::TRACE,
            tracing::Level::DEBUG,
            tracing::Level::INFO,
            tracing::Level::WARN,
            tracing::Level::ERROR,
        ] {
            otel_log!(level, "quiver.test.log_fields", key = 42);
            otel_log!(level, "quiver.test.log_only");
        }
    }

    #[test]
    fn test_otel_log_const_level() {
        otel_log!(tracing::Level::DEBUG, "quiver.test.log_const", key = 1);
        otel_log!(tracing::Level::INFO, "quiver.test.log_const");
    }
}
