// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dispatch OTLP-encoded bytes as tracing events to the configured subscriber.
//!
//! This module provides integration with the existing tracing subscriber infrastructure
//! by reconstructing `tracing::Event` objects from OTLP bytes and dispatching them through
//! the global subscriber. This ensures that OTLP bytes are formatted using whatever
//! fmt layer configuration the application has set up (colors, timestamps, format, etc.).
//!
//! # Architecture
//!
//! ```text
//! OTLP bytes (from internal telemetry, channel, etc.)
//!     ↓
//! RawLogsData (LogsDataView) - zero-copy decode
//!     ↓
//! Reconstruct tracing::Event + Metadata
//!     ↓
//! dispatch::get_default(|d| d.event(&event))
//!     ↓
//! Existing subscriber stack (filters, fmt layer, etc.)
//!     ↓
//! Console output (using configured formatting)
//! ```
//!
//! # Benefits
//!
//! - **Uses existing fmt configuration**: Colors, timestamps, format - everything
//! - **Respects filters**: EnvFilter, level filters, target filters all work
//! - **Embedded-friendly**: Works with whatever the application configured
//! - **Zero duplication**: No parallel formatting logic
//!
//! # Example
//!
//! ```ignore
//! use otap_df_telemetry::tracing_integration::dispatch_otlp_bytes_as_events;
//!
//! // Somewhere in your app, configure tracing as usual:
//! tracing_subscriber::fmt()
//!     .with_ansi(true)
//!     .with_level(true)
//!     .init();
//!
//! // Later, when you receive OTLP bytes:
//! let otlp_bytes: Vec<u8> = /* from channel, receiver, etc. */;
//! dispatch_otlp_bytes_as_events(&otlp_bytes)?;
//! // ^ These will be formatted using the configured fmt layer!
//! ```

use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType, InstrumentationScopeView};
use tracing::Level;
use std::fmt;

/// Dispatch OTLP bytes as tracing events to the configured subscriber.
///
/// This function:
/// 1. Decodes OTLP bytes to LogsDataView (zero-copy)
/// 2. Reconstructs tracing events for each log record
/// 3. Uses tracing macros to dispatch through the global subscriber
/// 4. The configured fmt layer formats them as usual
///
/// # Arguments
/// * `otlp_bytes` - OTLP-encoded ExportLogsServiceRequest bytes
///
/// # Example
/// ```ignore
/// let otlp_bytes = vec![/* OTLP bytes */];
/// dispatch_otlp_bytes_as_events(&otlp_bytes)?;
/// ```
pub fn dispatch_otlp_bytes_as_events(otlp_bytes: &[u8]) -> Result<(), DispatchError> {
    // Decode OTLP bytes to LogsDataView (zero-copy)
    let logs_view = RawLogsData::new(otlp_bytes);
    
    // Iterate through the logs structure
    for resource_logs in logs_view.resources() {
        for scope_logs in resource_logs.scopes() {
            // Extract scope name (target) once for all records in this scope
            let target = if let Some(scope) = scope_logs.scope() {
                if let Some(name) = scope.name() {
                    std::str::from_utf8(name)
                        .ok()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "otap::internal".to_string())
                } else {
                    "otap::internal".to_string()
                }
            } else {
                "otap::internal".to_string()
            };
            
            for log_record in scope_logs.log_records() {
                dispatch_log_record_as_event(&log_record, &target)?;
            }
        }
    }
    
    Ok(())
}

/// Dispatch a single log record as a tracing event through the global dispatcher.
///
/// This uses tracing's dispatcher API directly, creating an Event that will be
/// processed by the configured subscriber (fmt layer, filters, etc.).
fn dispatch_log_record_as_event<L: LogRecordView>(
    log_record: &L,
    _target: &str,
) -> Result<(), DispatchError> {
    // Extract level from severity number
    let level = severity_number_to_level(log_record.severity_number());
    
    // Extract message from body
    let message = log_record.body()
        .map(|body| format_any_value(&body))
        .unwrap_or_default();
    
    // Build a formatted string with all attributes
    // This is the simplest approach - format everything as key=value pairs
    let mut attributes = Vec::new();
    for attr in log_record.attributes() {
        let key = String::from_utf8_lossy(attr.key());
        if let Some(value) = attr.value() {
            attributes.push(format!("{}={}", key, format_any_value(&value)));
        }
    }
    
    let attrs_str = attributes.join(", ");
    
    // Combine message and attributes
    let full_message = if attrs_str.is_empty() {
        message
    } else if message.is_empty() {
        attrs_str
    } else {
        format!("{} {}", message, attrs_str)
    };
    
    // Dispatch the event through the global dispatcher using the event! macro
    // This will be processed by the configured subscriber (fmt layer, filters, etc.)
    // The macro creates the proper callsite infrastructure, and the fmt layer
    // will format it exactly like a regular tracing event.
    match level {
        Level::ERROR => {
            tracing::event!(target: "otap::internal", Level::ERROR, message = %full_message);
        }
        Level::WARN => {
            tracing::event!(target: "otap::internal", Level::WARN, message = %full_message);
        }
        Level::INFO => {
            tracing::event!(target: "otap::internal", Level::INFO, message = %full_message);
        }
        Level::DEBUG => {
            tracing::event!(target: "otap::internal", Level::DEBUG, message = %full_message);
        }
        Level::TRACE => {
            tracing::event!(target: "otap::internal", Level::TRACE, message = %full_message);
        }
    }
    
    Ok(())
}

/// Convert OTLP severity number to tracing Level.
fn severity_number_to_level(severity: Option<i32>) -> Level {
    match severity {
        Some(1..=4) => Level::TRACE,
        Some(5..=8) => Level::DEBUG,
        Some(9..=12) => Level::INFO,
        Some(13..=16) => Level::WARN,
        Some(17..=24) => Level::ERROR,
        _ => Level::INFO, // Default
    }
}

/// Format an AnyValue to a string representation.
///
/// This recursively formats OTLP AnyValue types to human-readable strings
/// that will appear in the tracing output.
fn format_any_value<'a>(value: &impl AnyValueView<'a>) -> String {
    match value.value_type() {
        ValueType::String => {
            value.as_string()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_default()
        }
        ValueType::Int64 => {
            value.as_int64()
                .map(|i| i.to_string())
                .unwrap_or_default()
        }
        ValueType::Bool => {
            value.as_bool()
                .map(|b| b.to_string())
                .unwrap_or_default()
        }
        ValueType::Double => {
            value.as_double()
                .map(|d| format!("{:.6}", d))
                .unwrap_or_default()
        }
        ValueType::Bytes => {
            value.as_bytes()
                .map(|bytes| format!("{:?}", bytes))
                .unwrap_or_default()
        }
        ValueType::Array => {
            if let Some(array_iter) = value.as_array() {
                let items: Vec<String> = array_iter
                    .map(|item| format_any_value(&item))
                    .collect();
                format!("[{}]", items.join(", "))
            } else {
                "[]".to_string()
            }
        }
        ValueType::KeyValueList => {
            if let Some(kvlist_iter) = value.as_kvlist() {
                let items: Vec<String> = kvlist_iter
                    .map(|kv| {
                        let key = String::from_utf8_lossy(kv.key());
                        if let Some(val) = kv.value() {
                            format!("{}={}", key, format_any_value(&val))
                        } else {
                            key.to_string()
                        }
                    })
                    .collect();
                format!("{{{}}}", items.join(", "))
            } else {
                "{}".to_string()
            }
        }
        ValueType::Empty => String::new(),
    }
}

/// Error type for event dispatching.
#[derive(Debug)]
pub enum DispatchError {
    /// Failed to decode OTLP bytes
    DecodeFailed(String),
    /// Failed to create event
    EventCreationFailed(String),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::DecodeFailed(msg) => write!(f, "OTLP decode failed: {}", msg),
            DispatchError::EventCreationFailed(msg) => write!(f, "Event creation failed: {}", msg),
        }
    }
}

impl std::error::Error for DispatchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing_integration::{TracingLogRecord, TracingAttribute, TracingAnyValue};
    use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
    use tracing::Level as TracingLevel;
    
    #[test]
    fn test_dispatch_simple_event() {
        // Configure a test subscriber
        let _guard = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(TracingLevel::INFO)
            .try_init();
        
        // Create a simple TracingLogRecord
        let metadata = tracing::Metadata::new(
            "test_event",
            "test_target",
            TracingLevel::INFO,
            None,
            None,
            None,
            tracing::field::FieldSet::new(&[], tracing::callsite::Identifier(std::ptr::null())),
            tracing::metadata::Kind::EVENT,
        );
        
        let attributes = vec![
            TracingAttribute {
                key: "count".to_string(),
                value: TracingAnyValue::Int(42),
            },
            TracingAttribute {
                key: "name".to_string(),
                value: TracingAnyValue::Str("test".to_string()),
            },
        ];
        
        let log_record = TracingLogRecord::new(&metadata, attributes, 1234567890);
        
        // Encode to OTLP bytes
        let mut encoder = StatefulOtlpEncoder::new(4096);
        let resource_bytes = Vec::new();
        encoder.encode_log_record(&log_record, &resource_bytes, "test_target").unwrap();
        let otlp_bytes = encoder.flush();
        
        // Dispatch as events - should use configured fmt layer
        dispatch_otlp_bytes_as_events(otlp_bytes.as_ref()).unwrap();
        
        // If this doesn't panic, the dispatch worked
        // Actual output validation would require capturing the test writer output
    }
    
    #[test]
    fn test_severity_conversion() {
        assert_eq!(severity_number_to_level(Some(1)), Level::TRACE);
        assert_eq!(severity_number_to_level(Some(5)), Level::DEBUG);
        assert_eq!(severity_number_to_level(Some(9)), Level::INFO);
        assert_eq!(severity_number_to_level(Some(13)), Level::WARN);
        assert_eq!(severity_number_to_level(Some(17)), Level::ERROR);
        assert_eq!(severity_number_to_level(None), Level::INFO);
    }
}
