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
//! Event names should follow the pattern: `quiver.<component>.<action>`
//!
//! Examples:
//! - `quiver.wal.replay_complete`
//! - `quiver.segment.recovered`
//! - `quiver.subscriber.progress_load_failed`
//!
//! ## Usage
//!
//! ```ignore
//! otel_info!("quiver.segment.recovered", segment_count = 5, "recovered segments from previous run");
//! otel_warn!("quiver.wal.cursor_decode_failed", error = %e);
//! otel_debug!("quiver.wal.replay_entry");
//! ```

#![allow(unused_macros, unused_macro_rules)]

/// Macro for logging informational messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or trailing message literal.
///
/// # Example
/// ```ignore
/// otel_info!("quiver.engine.startup", version = "1.0.0");
/// otel_info!("quiver.segment.finalized", segment_id = 42, "segment finalized successfully");
/// ```
macro_rules! otel_info {
    ($name:expr, $($fields:tt)+) => {
        tracing::info!(name: $name, target: "quiver", $($fields)+);
    };
    ($name:expr) => {
        tracing::info!(name: $name, target: "quiver", "");
    };
}

/// Macro for logging warning messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or trailing message literal.
///
/// # Example
/// ```ignore
/// otel_warn!("quiver.wal.cursor_missing", "cursor file not found, starting from beginning");
/// otel_warn!("quiver.segment.delete_failed", error = %e, segment_id = 42);
/// ```
macro_rules! otel_warn {
    ($name:expr, $($fields:tt)+) => {
        tracing::warn!(name: $name, target: "quiver", $($fields)+);
    };
    ($name:expr) => {
        tracing::warn!(name: $name, target: "quiver", "");
    };
}

/// Macro for logging error messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or trailing message literal.
///
/// # Example
/// ```ignore
/// otel_error!("quiver.wal.corruption_detected", error = %e, "WAL corruption during replay");
/// ```
macro_rules! otel_error {
    ($name:expr, $($fields:tt)+) => {
        tracing::error!(name: $name, target: "quiver", $($fields)+);
    };
    ($name:expr) => {
        tracing::error!(name: $name, target: "quiver", "");
    };
}

/// Macro for logging debug messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or trailing message literal.
///
/// # Example
/// ```ignore
/// otel_debug!("quiver.wal.replay_started", cursor_position = 0);
/// otel_debug!("quiver.segment.scan_entry", path = %path.display());
/// ```
macro_rules! otel_debug {
    ($name:expr, $($fields:tt)+) => {
        tracing::debug!(name: $name, target: "quiver", $($fields)+);
    };
    ($name:expr) => {
        tracing::debug!(name: $name, target: "quiver", "");
    };
}

/// Macro for logging trace messages with a required event name.
///
/// # Arguments
/// - First argument (required): The event name identifying the log event.
/// - Additional optional key-value pairs and/or trailing message literal.
///
/// # Example
/// ```ignore
/// otel_trace!("quiver.segment.file_in_use", segment = seq.raw());
/// ```
macro_rules! otel_trace {
    ($name:expr, $($fields:tt)+) => {
        tracing::trace!(name: $name, target: "quiver", $($fields)+);
    };
    ($name:expr) => {
        tracing::trace!(name: $name, target: "quiver", "");
    };
}

// Make macros available within this crate only (no #[macro_export])
pub(crate) use otel_debug;
pub(crate) use otel_error;
pub(crate) use otel_info;
pub(crate) use otel_trace;
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

        otel_trace!("quiver.test.trace_only");
        otel_trace!("quiver.test.trace_fields", detail = "fine-grained");
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
