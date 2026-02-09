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

#![allow(unused_macros, unused_macro_rules)]

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
macro_rules! otel_debug {
    ($name:expr, $($fields:tt)+) => {
        tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), $($fields)+);
    };
    ($name:expr) => {
        tracing::debug!(name: $name, target: env!("CARGO_PKG_NAME"), "");
    };
}

// Make macros available within this crate only (no #[macro_export])
pub(crate) use otel_debug;
pub(crate) use otel_error;
pub(crate) use otel_info;
pub(crate) use otel_warn;

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_compile() {
        // These just verify the macros compile correctly with various argument patterns
        otel_info!("quiver.test.info_only");
        otel_info!("quiver.test.info_message", "a message");
        otel_info!("quiver.test.info_fields", key = 42);
        otel_info!("quiver.test.info_fields_message", key = 42, "with message");
        otel_info!("quiver.test.info_debug_format", key = ?vec![1, 2, 3]);
        otel_info!("quiver.test.info_display_format", key = %"display");
        otel_info!(
            "quiver.test.info_message_as_field",
            message = "human-readable message"
        );

        otel_warn!("quiver.test.warn_only");
        otel_warn!("quiver.test.warn_message", "warning message");
        otel_warn!("quiver.test.warn_fields", a = 1, b = 2);

        otel_error!("quiver.test.error_only");
        otel_error!("quiver.test.error_fields", error = %"some error");

        otel_debug!("quiver.test.debug_only");
        otel_debug!("quiver.test.debug_fields", count = 100);
    }

    #[test]
    fn test_shorthand_field() {
        let replayed = 42;
        otel_info!("quiver.test.shorthand", replayed, "replayed entries");
    }

    /// Ensures no source files use raw `tracing::` macros.
    ///
    /// All logging in quiver must use `otel_info!`, `otel_warn!`, etc. to
    /// enforce durable event names. This test scans the source tree to
    /// catch violations.
    #[test]
    fn no_raw_tracing_calls() {
        use std::path::Path;

        // Patterns indicating raw tracing macro usage
        const RAW_PATTERNS: &[&str] = &[
            "tracing::info!(",
            "tracing::warn!(",
            "tracing::error!(",
            "tracing::debug!(",
            "tracing::trace!(",
        ];

        fn scan_directory(dir: &Path, violations: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    scan_directory(&path, violations);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    // Skip this file (it defines the macros)
                    if path.file_name().is_some_and(|name| name == "logging.rs") {
                        continue;
                    }

                    let Ok(content) = std::fs::read_to_string(&path) else {
                        continue;
                    };

                    for (line_num, line) in content.lines().enumerate() {
                        // Skip comments
                        let trimmed = line.trim();
                        if trimmed.starts_with("//") {
                            continue;
                        }

                        for pattern in RAW_PATTERNS {
                            if line.contains(pattern) {
                                violations.push(format!(
                                    "{}:{}: {}",
                                    path.display(),
                                    line_num + 1,
                                    trimmed
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Find the src directory relative to this file's location
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let src_dir = Path::new(manifest_dir).join("src");

        let mut violations = Vec::new();
        scan_directory(&src_dir, &mut violations);

        assert!(
            violations.is_empty(),
            "Raw tracing calls found - use otel_info!, otel_warn!, etc. instead:\n{}",
            violations.join("\n")
        );
    }
}
