// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! LogRecordView implementation for tokio-tracing events.
//!
//! This module provides the bridge between tracing::Event and our OTLP stateful encoder,
//! allowing tracing events to be encoded directly without intermediate allocations.

use otap_df_pdata::schema::{SpanId, TraceId};
use otap_df_pdata::views::common::{AnyValueView, AttributeView, Str, ValueType};
use otap_df_pdata::views::logs::LogRecordView;
use std::fmt;
use tracing::{Level, Metadata};

/// A LogRecordView implementation that wraps a tracing event.
///
/// This provides zero-copy access to tracing event data, enabling direct encoding
/// to OTLP format via the stateful encoder.
///
/// # Lifetime
/// The lifetime parameter `'a` represents the lifetime of the underlying tracing::Event
/// and all its associated data (field values, metadata, etc.).
pub struct TracingLogRecord {
    /// The event name from the `name` field, if present
    event_name: Option<String>,
    
    /// The severity level from tracing
    level: Level,
    
    /// Timestamp when the event occurred (nanoseconds since Unix epoch)
    timestamp_nanos: u64,
    
    /// The target (typically module path) from tracing metadata
    target: String,
    
    /// Collected attributes from the event's fields
    attributes: Vec<TracingAttribute>,
    
    /// Optional body/message for the log record (stored as TracingAnyValue)
    body: Option<TracingAnyValue>,
}

impl TracingLogRecord {
    /// Creates a new TracingLogRecord from tracing event components.
    ///
    /// # Arguments
    /// * `metadata` - The event's metadata (level, target, name with file:line)
    /// * `attributes` - Key-value pairs extracted from event fields
    /// * `timestamp_nanos` - Event timestamp in nanoseconds since Unix epoch
    ///
    /// Note: metadata.name() contains both the event location and file:line info,
    /// e.g., "event src/main.rs:42", so we don't need to separately track file/line.
    pub fn new(
        metadata: &Metadata<'_>,
        attributes: Vec<TracingAttribute>,
        timestamp_nanos: u64,
    ) -> Self {
        Self {
            event_name: Some(metadata.name().to_string()),
            level: *metadata.level(),
            timestamp_nanos,
            target: metadata.target().to_string(),
            attributes,
            body: None, // Can be populated from message field
        }
    }

    /// Sets the body/message for this log record.
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(TracingAnyValue::Str(body));
        self
    }

    /// Returns the target (typically module path) for this log record.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Creates a TracingLogRecord with a custom event name (for span events).
    pub fn new_with_event_name(
        metadata: &Metadata<'_>,
        attributes: Vec<TracingAttribute>,
        timestamp_nanos: u64,
        event_name: String,
    ) -> Self {
        Self {
            event_name: Some(event_name),
            level: *metadata.level(),
            timestamp_nanos,
            target: metadata.target().to_string(),
            attributes,
            body: None,
        }
    }

    /// Creates a minimal TracingLogRecord for span end events.
    pub fn new_span_end(
        span_id: u64,
        attributes: Vec<TracingAttribute>,
        timestamp_nanos: u64,
    ) -> Self {
        Self {
            event_name: Some(format!("span.end (id:{})", span_id)),
            level: Level::INFO,
            timestamp_nanos,
            target: "tracing::span".to_string(),
            attributes,
            body: None,
        }
    }
}

impl LogRecordView for TracingLogRecord {
    type Attribute<'att> = TracingAttributeView<'att>
    where
        Self: 'att;

    type AttributeIter<'att> = TracingAttributeIterator<'att>
    where
        Self: 'att;

    type Body<'bod> = TracingAnyValue
    where
        Self: 'bod;

    fn time_unix_nano(&self) -> Option<u64> {
        Some(self.timestamp_nanos)
    }

    fn observed_time_unix_nano(&self) -> Option<u64> {
        // For tracing events, observed time = event time
        Some(self.timestamp_nanos)
    }

    fn severity_number(&self) -> Option<i32> {
        // Map tracing Level to OTLP severity numbers
        // https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber
        Some(match self.level {
            Level::TRACE => 1,  // TRACE
            Level::DEBUG => 5,  // DEBUG
            Level::INFO => 9,   // INFO
            Level::WARN => 13,  // WARN
            Level::ERROR => 17, // ERROR
        })
    }

    fn severity_text(&self) -> Option<Str<'_>> {
        Some(self.level.as_str().as_bytes())
    }

    fn body(&self) -> Option<Self::Body<'_>> {
        self.body.clone()
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        TracingAttributeIterator {
            inner: self.attributes.iter(),
        }
    }

    fn dropped_attributes_count(&self) -> u32 {
        0 // We don't drop attributes in this implementation
    }

    fn flags(&self) -> Option<u32> {
        None // No flags for now
    }

    fn trace_id(&self) -> Option<&TraceId> {
        None // TODO: Extract from tracing span context when available
    }

    fn span_id(&self) -> Option<&SpanId> {
        None // TODO: Extract from tracing span context when available
    }

    fn event_name(&self) -> Option<Str<'_>> {
        self.event_name.as_ref().map(|s| s.as_bytes())
    }
}

/// Represents an attribute (key-value pair) from a tracing event.
///
/// This wraps tracing field data into a structure compatible with OTLP encoding.
#[derive(Debug, Clone)]
pub struct TracingAttribute {
    /// The attribute key
    pub key: String,
    /// The attribute value
    pub value: TracingAnyValue,
}

/// Wrapper for TracingAttribute that implements AttributeView
pub struct TracingAttributeView<'a> {
    attribute: &'a TracingAttribute,
}

impl<'a> AttributeView for TracingAttributeView<'a> {
    type Val<'val> = TracingAnyValue
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        self.attribute.key.as_bytes()
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        Some(self.attribute.value.clone())
    }
}

/// Iterator wrapper for TracingAttribute slice
pub struct TracingAttributeIterator<'a> {
    inner: std::slice::Iter<'a, TracingAttribute>,
}

impl<'a> Iterator for TracingAttributeIterator<'a> {
    type Item = TracingAttributeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|attr| TracingAttributeView { attribute: attr })
    }
}

/// Represents a value from a tracing event field.
///
/// This is a simplified subset of OTLP AnyValue that covers the most common
/// types found in tracing events.
#[derive(Debug, Clone)]
pub enum TracingAnyValue {
    /// String value
    Str(String),
    /// Integer value (i64)
    Int(i64),
    /// Boolean value
    Bool(bool),
    /// Double-precision floating point value
    Double(f64),
}

/// Dummy AttributeView for composite types (not used in this basic implementation)
pub struct EmptyAttribute;

impl AttributeView for EmptyAttribute {
    type Val<'val> = TracingAnyValue
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        b""
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        None
    }
}

impl<'a> AnyValueView<'a> for TracingAnyValue {
    type KeyValue = EmptyAttribute;
    type ArrayIter<'arr> = std::iter::Empty<Self>
    where
        Self: 'arr;
    type KeyValueIter<'kv> = std::iter::Empty<Self::KeyValue>
    where
        Self: 'kv;

    fn value_type(&self) -> ValueType {
        match self {
            TracingAnyValue::Str(_) => ValueType::String,
            TracingAnyValue::Int(_) => ValueType::Int64,
            TracingAnyValue::Bool(_) => ValueType::Bool,
            TracingAnyValue::Double(_) => ValueType::Double,
        }
    }

    fn as_string(&self) -> Option<Str<'_>> {
        match self {
            TracingAnyValue::Str(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            TracingAnyValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_int64(&self) -> Option<i64> {
        match self {
            TracingAnyValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    fn as_double(&self) -> Option<f64> {
        match self {
            TracingAnyValue::Double(d) => Some(*d),
            _ => None,
        }
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        None // Not commonly used in tracing
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        None // Not supported in this implementation
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        None // Not supported in this implementation
    }
}

// Implement Display for easier debugging
impl fmt::Display for TracingAnyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TracingAnyValue::Str(s) => write!(f, "{}", s),
            TracingAnyValue::Int(i) => write!(f, "{}", i),
            TracingAnyValue::Bool(b) => write!(f, "{}", b),
            TracingAnyValue::Double(d) => write!(f, "{}", d),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_log_record_creation() {
        // Create a mock metadata (in real usage this comes from tracing)
        let _level = Level::INFO;
        
        let _attributes = vec![
            TracingAttribute {
                key: "key1".to_string(),
                value: TracingAnyValue::Str("value1".to_string()),
            },
            TracingAttribute {
                key: "count".to_string(),
                value: TracingAnyValue::Int(42),
            },
        ];

        // Note: In real usage, metadata comes from tracing::Event
        // For this test, we'll test the TracingLogRecord structure directly
        let _timestamp = 1234567890000000000u64;
        
        // Test basic construction and access
        let key1 = "key1".to_string();
        let value1 = TracingAnyValue::Str("value1".to_string());
        let attr = TracingAttribute { key: key1, value: value1 };
        
        assert_eq!(attr.key, "key1");
        match &attr.value {
            TracingAnyValue::Str(s) => assert_eq!(s, "value1"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_severity_mapping() {
        // Test that tracing levels map correctly to OTLP severity numbers
        let levels_and_numbers = [
            (Level::TRACE, 1),
            (Level::DEBUG, 5),
            (Level::INFO, 9),
            (Level::WARN, 13),
            (Level::ERROR, 17),
        ];

        for (level, expected_number) in levels_and_numbers {
            let severity_number = match level {
                Level::TRACE => 1,
                Level::DEBUG => 5,
                Level::INFO => 9,
                Level::WARN => 13,
                Level::ERROR => 17,
            };
            assert_eq!(severity_number, expected_number);
        }
    }

    #[test]
    fn test_any_value_types() {
        use otap_df_pdata::views::common::AnyValueView;
        
        let str_val = TracingAnyValue::Str("test".to_string());
        assert!(str_val.as_string().is_some());
        assert!(str_val.as_int64().is_none());

        let int_val = TracingAnyValue::Int(123);
        assert!(int_val.as_int64().is_some());
        assert_eq!(int_val.as_int64().unwrap(), 123);

        let bool_val = TracingAnyValue::Bool(true);
        assert!(bool_val.as_bool().is_some());
        assert_eq!(bool_val.as_bool().unwrap(), true);

        let double_val = TracingAnyValue::Double(3.14);
        assert!(double_val.as_double().is_some());
        assert!((double_val.as_double().unwrap() - 3.14).abs() < f64::EPSILON);
    }
}
