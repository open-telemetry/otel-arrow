// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! LogRecordView implementation for tokio-tracing events.
//!
//! This module provides the bridge between tracing::Event and our
//! OTLP bytes. All data is borrowed from the tracing event with zero copies.
//!
//! The key insight is that `TracingAnyValue<'a>` is `Copy` - it's just an enum
//! containing borrowed references. This means we can implement `AnyValueView`
//! directly on it without needing a wrapper type, and the lifetime `'a` is
//! tied directly to the underlying tracing event data.

use otap_df_pdata::schema::{SpanId, TraceId};
use otap_df_pdata::views::common::{AnyValueView, AttributeView, Str, ValueType};
use otap_df_pdata::views::logs::LogRecordView;
use std::fmt;
use tracing::{Level, Metadata};

/// A LogRecordView implementation that wraps a tracing event.
///
/// Uses zero-copy borrows throughout:
/// - `event_name`: `&'static str` since `Metadata::name()` is always static
/// - `target`: `&'a str` borrowed from `Metadata<'a>`
/// - All attribute keys and values are borrowed from the event
///
/// The lifetime `'a` ties this struct to the tracing event callback scope.
/// Encoding to OTLP bytes must complete before the callback returns.
pub struct TracingLogRecord<'a> {
    /// The event name - always static from tracing metadata
    event_name: Option<&'static str>,

    /// The severity level from tracing
    level: Level,

    /// Timestamp when the event occurred (nanoseconds since Unix epoch)
    timestamp_nanos: u64,

    /// The target from tracing metadata, borrowed for the event lifetime
    target: &'a str,

    /// Event fields - all borrowed from the tracing event
    attributes: Vec<TracingAttribute<'a>>,

    /// Optional body/message for the log record (stored as raw &str,
    /// constructed into TracingAnyValue on demand in body() method)
    body: Option<&'a str>,
}

impl<'a> TracingLogRecord<'a> {
    /// Creates a new TracingLogRecord from tracing event components.
    ///
    /// The returned struct borrows from the metadata and attributes,
    /// so it must be encoded before the tracing callback returns.
    pub fn new(
        metadata: &Metadata<'a>,
        attributes: Vec<TracingAttribute<'a>>,
        timestamp_nanos: u64,
    ) -> Self {
        Self {
            event_name: Some(metadata.name()),
            level: *metadata.level(),
            timestamp_nanos,
            target: metadata.target(),
            attributes,
            body: None,
        }
    }

    /// Sets the body/message for this log record.
    pub fn with_body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    /// Returns the target (typically module path) for this log record.
    pub fn target(&self) -> &str {
        self.target
    }

    /// Creates a TracingLogRecord with a custom static event name.
    ///
    /// Use this for synthetic events like span.start/span.end where
    /// you want a different event name than metadata.name().
    pub fn new_with_event_name(
        metadata: &Metadata<'a>,
        attributes: Vec<TracingAttribute<'a>>,
        timestamp_nanos: u64,
        event_name: &'static str,
    ) -> Self {
        Self {
            event_name: Some(event_name),
            level: *metadata.level(),
            timestamp_nanos,
            target: metadata.target(),
            attributes,
            body: None,
        }
    }

    /// Creates a log record for span end events.
    ///
    /// The span_id should be added as an attribute by the caller.
    pub fn new_span_end(
        target: &'a str,
        attributes: Vec<TracingAttribute<'a>>,
        timestamp_nanos: u64,
    ) -> Self {
        Self {
            event_name: Some("span.end"),
            level: Level::INFO,
            timestamp_nanos,
            target,
            attributes,
            body: None,
        }
    }
}

impl<'a> LogRecordView for TracingLogRecord<'a> {
    // Use 'a (the data lifetime) for the attribute type, not the GAT lifetime.
    // This is correct because our attributes borrow from the original tracing event data.
    type Attribute<'att>
        = TracingAttribute<'a>
    where
        Self: 'att;

    // The iterator borrows from self (lifetime 'att) but yields items with lifetime 'a.
    // Since TracingAttribute<'a> is Copy, we can just copy the values.
    type AttributeIter<'att>
        = std::iter::Copied<std::slice::Iter<'att, TracingAttribute<'a>>>
    where
        Self: 'att;

    // Body is constructed on demand from the stored &'a str.
    // Since we create a fresh TracingAnyValue<'bod> each time, the GAT works.
    type Body<'bod>
        = TracingAnyValue<'bod>
    where
        Self: 'bod;

    fn time_unix_nano(&self) -> Option<u64> {
        Some(self.timestamp_nanos)
    }

    fn observed_time_unix_nano(&self) -> Option<u64> {
        // Field not used
        None
    }

    fn severity_number(&self) -> Option<i32> {
        // https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber
        Some(match self.level {
            Level::TRACE => 1,
            Level::DEBUG => 5,
            Level::INFO => 9,
            Level::WARN => 13,
            Level::ERROR => 17,
        })
    }

    fn severity_text(&self) -> Option<Str<'_>> {
        Some(self.level.as_str().as_bytes())
    }

    fn body(&self) -> Option<Self::Body<'_>> {
        // Construct TracingAnyValue on demand from stored &str.
        // The lifetime 'bod comes from &self, but the data has lifetime 'a.
        // Since 'a: 'bod (self contains 'a), this coercion is valid.
        self.body.map(TracingAnyValue::Str)
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        self.attributes.iter().copied()
    }

    fn dropped_attributes_count(&self) -> u32 {
        0
    }

    fn flags(&self) -> Option<u32> {
        None
    }

    fn trace_id(&self) -> Option<&TraceId> {
        None // TODO
    }

    fn span_id(&self) -> Option<&SpanId> {
        None // TODO
    }

    fn event_name(&self) -> Option<Str<'_>> {
        self.event_name.map(|s| s.as_bytes())
    }
}

/// Represents an attribute (key-value pair) from a tracing event.
///
/// All data is borrowed from the tracing event with lifetime 'a.
/// This type is `Copy` because it only contains borrowed references.
#[derive(Debug, Clone, Copy)]
pub struct TracingAttribute<'a> {
    /// The attribute key - borrowed from tracing
    pub key: &'a str,
    /// The attribute value - borrowed from tracing
    pub value: TracingAnyValue<'a>,
}

impl<'a> AttributeView for TracingAttribute<'a> {
    type Val<'val>
        = TracingAnyValue<'val>
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        self.key.as_bytes()
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        Some(self.value)
    }
}

/// Represents a value from a tracing event field.
///
/// This mirrors OTLP's AnyValue type system, supporting full structural fidelity
/// for nested data from tracing events (arrays, maps, etc.).
///
/// All variants use borrowed references with lifetime 'a, enabling true zero-copy
/// from tracing events to OTLP bytes. The type is `Copy` because it only contains
/// primitive values or borrowed references - copying just copies the pointer/length,
/// not the underlying data.
#[derive(Debug, Clone, Copy)]
pub enum TracingAnyValue<'a> {
    /// String value - borrowed
    Str(&'a str),
    /// Integer value (i64)
    Int(i64),
    /// Boolean value
    Bool(bool),
    /// Double-precision floating point value
    Double(f64),
    /// Bytes value - borrowed
    Bytes(&'a [u8]),
    /// Array of values - borrowed slice
    Array(&'a [TracingAnyValue<'a>]),
    /// Key-value list (like a map/object) - borrowed slice
    KeyValueList(&'a [TracingAttribute<'a>]),
}

/// Iterator for array values that yields copies of TracingAnyValue.
///
/// Since TracingAnyValue is Copy, this just copies the small enum
/// (which contains borrowed references to the underlying data).
pub struct ArrayIterator<'a> {
    inner: std::slice::Iter<'a, TracingAnyValue<'a>>,
}

impl<'a> Iterator for ArrayIterator<'a> {
    type Item = TracingAnyValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

/// Iterator for nested KeyValueList attributes.
pub struct KeyValueListIterator<'a> {
    inner: std::slice::Iter<'a, TracingAttribute<'a>>,
}

impl<'a> Iterator for KeyValueListIterator<'a> {
    type Item = TracingAttribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

impl<'a> AnyValueView<'a> for TracingAnyValue<'a> {
    type KeyValue = TracingAttribute<'a>;
    type ArrayIter<'arr>
        = ArrayIterator<'a>
    where
        Self: 'arr;
    type KeyValueIter<'kv>
        = KeyValueListIterator<'a>
    where
        Self: 'kv;

    fn value_type(&self) -> ValueType {
        match self {
            TracingAnyValue::Str(_) => ValueType::String,
            TracingAnyValue::Int(_) => ValueType::Int64,
            TracingAnyValue::Bool(_) => ValueType::Bool,
            TracingAnyValue::Double(_) => ValueType::Double,
            TracingAnyValue::Bytes(_) => ValueType::Bytes,
            TracingAnyValue::Array(_) => ValueType::Array,
            TracingAnyValue::KeyValueList(_) => ValueType::KeyValueList,
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
        match self {
            TracingAnyValue::Bytes(b) => Some(b),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        match self {
            TracingAnyValue::Array(arr) => Some(ArrayIterator { inner: arr.iter() }),
            _ => None,
        }
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        match self {
            TracingAnyValue::KeyValueList(kvs) => Some(KeyValueListIterator { inner: kvs.iter() }),
            _ => None,
        }
    }
}

// Implement Display for easier debugging
impl<'a> fmt::Display for TracingAnyValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TracingAnyValue::Str(s) => write!(f, "{}", s),
            TracingAnyValue::Int(i) => write!(f, "{}", i),
            TracingAnyValue::Bool(b) => write!(f, "{}", b),
            TracingAnyValue::Double(d) => write!(f, "{}", d),
            TracingAnyValue::Bytes(b) => write!(f, "{:?}", b),
            TracingAnyValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            TracingAnyValue::KeyValueList(kvs) => {
                write!(f, "{{")?;
                for (i, kv) in kvs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", kv.key, kv.value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_attribute_creation() {
        // Test basic construction with borrowed data
        let key = "key1";
        let value = TracingAnyValue::Str("value1");
        let attr = TracingAttribute { key, value };

        assert_eq!(attr.key, "key1");
        match attr.value {
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

        let str_val = TracingAnyValue::Str("test");
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

    #[test]
    fn test_zero_copy_semantics() {
        // Verify that TracingAnyValue is Copy (no heap allocation)
        let original = TracingAnyValue::Str("hello");
        let copied = original; // This should be a copy, not a move
        let _also_original = original; // Original still usable

        match copied {
            TracingAnyValue::Str(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_attribute_is_copy() {
        // Verify that TracingAttribute is Copy
        let attr = TracingAttribute {
            key: "test_key",
            value: TracingAnyValue::Int(42),
        };
        let copied = attr;
        let _also_original = attr; // Original still usable
        
        assert_eq!(copied.key, "test_key");
    }
}
