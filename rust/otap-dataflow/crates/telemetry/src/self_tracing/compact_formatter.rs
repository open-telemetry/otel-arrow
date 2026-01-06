// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compact log formatter - a minimal `fmt::layer()` alternative.
//!
//! This module provides a lightweight formatting layer for tokio-tracing events
//! that outputs human-readable log lines to stdout/stderr. It uses the same
//! `CompactLogRecord` structure that can later be extended for OTLP encoding.
//!
//! # Design
//!
//! The key insight is to separate:
//! - **Structural fields** (timestamp, severity, callsite_id) - kept as cheap values
//! - **Borrowed data** (body, attributes) - encoded to bytes during event capture
//! - **Static callsite info** (target, name, file, line) - cached at registration time
//!
//! This design allows for immediate formatting and output while preserving the
//! option to accumulate and batch-encode to OTLP later.
//!
//! # OTLP View Integration
//!
//! For decoding the pre-encoded body and attributes bytes, we reuse the pdata
//! View types (`RawAnyValue`, `RawKeyValue`) which provide zero-copy parsing
//! of OTLP protobuf bytes.

use bytes::Bytes;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use otap_df_pdata::proto::consts::field_num::logs::{LOG_RECORD_ATTRIBUTES, LOG_RECORD_BODY};
use otap_df_pdata::proto::consts::wire_types;
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata::views::otlp::bytes::common::{RawAnyValue, RawKeyValue};
use otap_df_pdata::views::otlp::bytes::decode::read_varint;

use tracing::{Event, Level, Subscriber};
use tracing::callsite::Identifier;
use tracing::span::{Attributes, Record};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use super::direct_encoder::{DirectFieldVisitor, ProtoBuffer};

// ============================================================================
// Core Data Structures
// ============================================================================

/// A compact log record with structural metadata and pre-encoded body/attributes.
///
/// Cheap-to-copy fields are kept in structural form for sorting/filtering.
/// Only borrowed data (body, attributes) is encoded to bytes.
/// Callsite details (target/name/file/line) are cached separately.
#[derive(Debug, Clone)]
pub struct CompactLogRecord {
    /// Callsite identifier - used to look up cached callsite info
    pub callsite_id: Identifier,
    
    /// Timestamp in nanoseconds since Unix epoch
    pub timestamp_ns: u64,
    
    /// Severity number: 1=TRACE, 5=DEBUG, 9=INFO, 13=WARN, 17=ERROR
    pub severity_number: u8,
    
    /// Severity text - &'static str from Level::as_str()
    pub severity_text: &'static str,
    
    /// Pre-encoded body and attributes (OTLP format for body field 5 + attrs field 6)
    pub body_attrs_bytes: Bytes,
}

/// Cached callsite information, populated via `register_callsite` hook.
#[derive(Debug, Clone)]
pub struct CachedCallsite {
    /// Target module path - &'static from Metadata
    pub target: &'static str,
    
    /// Event name - &'static from Metadata
    pub name: &'static str,
    
    /// Source file - &'static from Metadata
    pub file: Option<&'static str>,
    
    /// Source line
    pub line: Option<u32>,
}

/// Cache of callsite information, keyed by `Identifier`.
#[derive(Debug, Default)]
pub struct CallsiteCache {
    callsites: HashMap<Identifier, CachedCallsite>,
}

impl CallsiteCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a callsite from its metadata.
    pub fn register(&mut self, metadata: &'static tracing::Metadata<'static>) {
        let id = metadata.callsite();
        let _ = self.callsites.entry(id).or_insert_with(|| CachedCallsite {
            target: metadata.target(),
            name: metadata.name(),
            file: metadata.file(),
            line: metadata.line(),
        });
    }
    
    /// Get cached callsite info by identifier.
    pub fn get(&self, id: &Identifier) -> Option<&CachedCallsite> {
        self.callsites.get(id)
    }
}

// ============================================================================
// Formatting
// ============================================================================

/// Format a CompactLogRecord as a human-readable string.
///
/// Output format: `2026-01-06T10:30:45.123Z  INFO target::name: body [attr=value, ...]`
pub fn format_log_record(record: &CompactLogRecord, cache: &CallsiteCache) -> String {
    let callsite = cache.get(&record.callsite_id);
    
    let (target, name) = match callsite {
        Some(cs) => (cs.target, cs.name),
        None => ("<unknown>", "<unknown>"),
    };
    
    let body_attrs = format_body_attrs(&record.body_attrs_bytes);
    
    format!(
        "{}  {:5} {}::{}: {}",
        format_timestamp(record.timestamp_ns),
        record.severity_text,
        target,
        name,
        body_attrs,
    )
}

/// Format nanosecond timestamp as ISO 8601 (UTC).
fn format_timestamp(nanos: u64) -> String {
    let secs = nanos / 1_000_000_000;
    let subsec_millis = (nanos % 1_000_000_000) / 1_000_000;
    
    // Convert to datetime components
    // Days since Unix epoch
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    
    // Calculate year/month/day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days as i64);
    
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, subsec_millis
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    // Algorithm from Howard Hinnant's date library
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    
    (y as i32, m, d)
}

/// Format body+attrs bytes as readable string.
///
/// Uses the pdata View types (`RawAnyValue`, `RawKeyValue`) for zero-copy
/// parsing of the OTLP protobuf bytes. This is consistent with the decoding
/// approach used in `otlp_bytes_formatter.rs`.
fn format_body_attrs(bytes: &Bytes) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    
    // The bytes contain LogRecord fields:
    // - field 5 (LOG_RECORD_BODY): AnyValue message
    // - field 6 (LOG_RECORD_ATTRIBUTES): repeated KeyValue messages
    
    let mut body_str = String::new();
    let mut attrs = Vec::new();
    let data = bytes.as_ref();
    let mut pos = 0;
    
    while pos < data.len() {
        // Read field tag
        let (tag, next_pos) = match read_varint(data, pos) {
            Some(v) => v,
            None => break,
        };
        pos = next_pos;
        
        let field_num = tag >> 3;
        let wire_type = tag & 0x7;
        
        if wire_type != wire_types::LEN {
            // Skip non-length-delimited fields (shouldn't happen for body/attrs)
            break;
        }
        
        // Read length-delimited content
        let (len, next_pos) = match read_varint(data, pos) {
            Some(v) => v,
            None => break,
        };
        pos = next_pos;
        let end = pos + len as usize;
        
        if end > data.len() {
            break;
        }
        
        let field_bytes = &data[pos..end];
        
        if field_num == LOG_RECORD_BODY {
            // Body: parse as AnyValue using pdata View
            let any_value = RawAnyValue::new(field_bytes);
            body_str = format_any_value(&any_value);
        } else if field_num == LOG_RECORD_ATTRIBUTES {
            // Attribute: parse as KeyValue using pdata View
            let kv = RawKeyValue::new(field_bytes);
            let key = String::from_utf8_lossy(kv.key()).to_string();
            let value = match kv.value() {
                Some(v) => format_any_value(&v),
                None => "<?>".to_string(),
            };
            attrs.push(format!("{}={}", key, value));
        }
        
        pos = end;
    }
    
    if !attrs.is_empty() {
        body_str.push_str(" [");
        body_str.push_str(&attrs.join(", "));
        body_str.push(']');
    }
    
    body_str
}

/// Format an AnyValue for display.
///
/// This is based on the same logic used in `otlp_bytes_formatter.rs`, providing
/// consistent formatting across the crate.
fn format_any_value<'a>(value: &impl AnyValueView<'a>) -> String {
    match value.value_type() {
        ValueType::String => {
            if let Some(s) = value.as_string() {
                String::from_utf8_lossy(s).to_string()
            } else {
                String::new()
            }
        }
        ValueType::Int64 => {
            if let Some(i) = value.as_int64() {
                i.to_string()
            } else {
                String::new()
            }
        }
        ValueType::Bool => {
            if let Some(b) = value.as_bool() {
                b.to_string()
            } else {
                String::new()
            }
        }
        ValueType::Double => {
            if let Some(d) = value.as_double() {
                format!("{:.6}", d)
            } else {
                String::new()
            }
        }
        ValueType::Bytes => {
            if let Some(bytes) = value.as_bytes() {
                format!("{:?}", bytes)
            } else {
                String::new()
            }
        }
        ValueType::Array => {
            if let Some(array_iter) = value.as_array() {
                let parts: Vec<_> = array_iter.map(|item| format_any_value(&item)).collect();
                format!("[{}]", parts.join(", "))
            } else {
                "[]".to_string()
            }
        }
        ValueType::KeyValueList => {
            if let Some(kvlist_iter) = value.as_kvlist() {
                let parts: Vec<_> = kvlist_iter
                    .map(|kv| {
                        let key_str = String::from_utf8_lossy(kv.key()).to_string();
                        match kv.value() {
                            Some(val) => format!("{}={}", key_str, format_any_value(&val)),
                            None => key_str,
                        }
                    })
                    .collect();
                format!("{{{}}}", parts.join(", "))
            } else {
                "{}".to_string()
            }
        }
        ValueType::Empty => String::new(),
    }
}

// ============================================================================
// Writer
// ============================================================================

/// Output target for log lines.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputTarget {
    /// Write to standard error.
    #[default]
    Stderr,
    /// Write to standard output.
    Stdout,
}

/// Simple writer that outputs log lines to stdout or stderr.
#[derive(Debug)]
pub struct SimpleWriter {
    target: OutputTarget,
}

impl Default for SimpleWriter {
    fn default() -> Self {
        Self::stderr()
    }
}

impl SimpleWriter {
    /// Create a writer that outputs to stdout.
    pub fn stdout() -> Self {
        Self { target: OutputTarget::Stdout }
    }
    
    /// Create a writer that outputs to stderr.
    pub fn stderr() -> Self {
        Self { target: OutputTarget::Stderr }
    }
    
    /// Write a log line (with newline).
    pub fn write_line(&self, line: &str) {
        match self.target {
            OutputTarget::Stdout => {
                let _ = writeln!(io::stdout(), "{}", line);
            }
            OutputTarget::Stderr => {
                let _ = writeln!(io::stderr(), "{}", line);
            }
        }
    }
}

// ============================================================================
// Layer Implementation
// ============================================================================

/// A minimal formatting layer that outputs log records to stdout/stderr.
///
/// This is a lightweight alternative to `tracing_subscriber::fmt::layer()`.
pub struct CompactFormatterLayer {
    callsite_cache: RwLock<CallsiteCache>,
    writer: SimpleWriter,
}

impl CompactFormatterLayer {
    /// Create a new layer that writes to stderr.
    pub fn new() -> Self {
        Self {
            callsite_cache: RwLock::new(CallsiteCache::new()),
            writer: SimpleWriter::stderr(),
        }
    }
    
    /// Create a new layer that writes to stdout.
    pub fn stdout() -> Self {
        Self {
            callsite_cache: RwLock::new(CallsiteCache::new()),
            writer: SimpleWriter::stdout(),
        }
    }
    
    /// Create a new layer that writes to stderr.
    pub fn stderr() -> Self {
        Self::new()
    }
}

impl Default for CompactFormatterLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for CompactFormatterLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        self.callsite_cache.write().unwrap().register(metadata);
        tracing::subscriber::Interest::always()
    }
    
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        
        // Encode body and attributes to bytes
        let body_attrs_bytes = encode_body_and_attrs(event);
        
        // Get current timestamp
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        // Build compact record
        let record = CompactLogRecord {
            callsite_id: metadata.callsite(),
            timestamp_ns,
            severity_number: level_to_severity(metadata.level()),
            severity_text: metadata.level().as_str(),
            body_attrs_bytes,
        };
        
        // Format and write immediately
        let cache = self.callsite_cache.read().unwrap();
        let line = format_log_record(&record, &cache);
        self.writer.write_line(&line);
    }
    
    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans in MVP
    }
    
    fn on_record(&self, _span: &tracing::span::Id, _values: &Record<'_>, _ctx: Context<'_, S>) {
        // Not handling spans in MVP
    }
    
    fn on_enter(&self, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans in MVP
    }
    
    fn on_exit(&self, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans in MVP
    }
    
    fn on_close(&self, _id: tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans in MVP
    }
}

/// Encode only body and attributes from an event to OTLP bytes.
fn encode_body_and_attrs(event: &Event<'_>) -> Bytes {
    let mut buf = ProtoBuffer::with_capacity(256);
    
    // Visit fields to encode body (field 5) and attributes (field 6)
    let mut visitor = DirectFieldVisitor::new(&mut buf);
    event.record(&mut visitor);
    
    buf.into_bytes()
}

/// Convert tracing Level to OTLP severity number.
fn level_to_severity(level: &Level) -> u8 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;
    
    #[test]
    fn test_format_timestamp() {
        // 2026-01-06T10:30:45.123Z in nanoseconds
        // Let's use a known timestamp: 2024-01-01T00:00:00.000Z
        let nanos: u64 = 1704067200 * 1_000_000_000; // 2024-01-01 00:00:00 UTC
        let formatted = format_timestamp(nanos);
        assert_eq!(formatted, "2024-01-01T00:00:00.000Z");
        
        // Test with milliseconds
        let nanos_with_ms: u64 = 1704067200 * 1_000_000_000 + 123_000_000;
        let formatted = format_timestamp(nanos_with_ms);
        assert_eq!(formatted, "2024-01-01T00:00:00.123Z");
    }
    
    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01 is day 0
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        
        // 2024-01-01 is 19723 days after 1970-01-01
        assert_eq!(days_to_ymd(19723), (2024, 1, 1));
    }
    
    #[test]
    fn test_level_to_severity() {
        assert_eq!(level_to_severity(&Level::TRACE), 1);
        assert_eq!(level_to_severity(&Level::DEBUG), 5);
        assert_eq!(level_to_severity(&Level::INFO), 9);
        assert_eq!(level_to_severity(&Level::WARN), 13);
        assert_eq!(level_to_severity(&Level::ERROR), 17);
    }
    
    #[test]
    fn test_callsite_cache() {
        let cache = CallsiteCache::new();
        assert!(cache.callsites.is_empty());
    }
    
    #[test]
    fn test_simple_writer_creation() {
        let _stdout = SimpleWriter::stdout();
        let _stderr = SimpleWriter::stderr();
        let _default = SimpleWriter::default();
    }
    
    #[test]
    fn test_compact_formatter_layer_creation() {
        let _layer = CompactFormatterLayer::new();
        let _stdout = CompactFormatterLayer::stdout();
        let _stderr = CompactFormatterLayer::stderr();
        let _default = CompactFormatterLayer::default();
    }
    
    #[test]
    fn test_layer_integration() {
        // Create the layer and subscriber
        let layer = CompactFormatterLayer::stderr();
        let subscriber = tracing_subscriber::registry().with(layer);
        
        // Set as default for this thread temporarily
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);
        
        // Emit some events - these should be formatted and written to stderr
        tracing::info!("Test info message");
        tracing::warn!(count = 42, "Warning with attribute");
        tracing::error!(error = "something failed", "Error occurred");
        
        // The test verifies no panics occur; actual output goes to stderr
    }
}
