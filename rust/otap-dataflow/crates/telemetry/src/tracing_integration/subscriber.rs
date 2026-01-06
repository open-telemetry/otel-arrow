// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tracing subscriber layer that captures events as TracingLogRecord instances.
//!
//! This layer integrates with the tracing-subscriber ecosystem, allowing us to:
//! 1. Capture all tracing events (from tokio macros and 3rd-party libraries)
//! 2. Convert them to TracingLogRecord (which implements LogRecordView)
//! 3. Encode them using our stateful OTLP encoder
//!
//! The layer uses a visitor pattern to extract field values from events and
//! constructs TracingLogRecord instances that can be encoded directly.
//!
//! **Important**: This layer is designed for single-threaded use. The callback
//! should encode the log record to OTLP bytes immediately - only bytes should
//! cross thread boundaries.
//!
//! **Zero-copy design**: All attribute keys and values are borrowed with lifetimes
//! tied to the tracing event callback. The `FieldVisitor` uses a string arena to
//! hold any formatted strings that need allocation.

use super::log_record::{TracingAnyValue, TracingAttribute, TracingLogRecord};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::span::Attributes;
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// Owned value type for span storage (spans outlive individual events).
#[derive(Clone)]
enum OwnedValue {
    Str(String),
    Int(i64),
    Bool(bool),
    Double(f64),
}

impl OwnedValue {
    /// Convert to a borrowed TracingAnyValue given a lifetime.
    fn as_borrowed(&self) -> TracingAnyValue<'_> {
        match self {
            OwnedValue::Str(s) => TracingAnyValue::Str(s.as_str()),
            OwnedValue::Int(i) => TracingAnyValue::Int(*i),
            OwnedValue::Bool(b) => TracingAnyValue::Bool(*b),
            OwnedValue::Double(d) => TracingAnyValue::Double(*d),
        }
    }
}

/// Tracing subscriber layer that captures events and spans as OTLP log records.
///
/// This layer implements an unconventional approach where spans are treated as pairs
/// of log records (start/end) rather than as first-class span objects. This aligns
/// with unified dataflow architectures where all telemetry flows through a single
/// log pipeline.
///
/// **Note**: This layer is `!Send` because it uses `RefCell` internally. The callback
/// should encode log records to OTLP bytes immediately - only bytes cross thread
/// boundaries.
pub struct OtlpTracingLayer<F>
where
    F: for<'a> Fn(TracingLogRecord<'a>) + 'static,
{
    /// Callback function that receives each TracingLogRecord
    on_event: F,
}

impl<F> OtlpTracingLayer<F>
where
    F: for<'a> Fn(TracingLogRecord<'a>) + 'static,
{
    /// Creates a new OtlpTracingLayer with the given event handler.
    ///
    /// # Arguments
    /// * `on_event` - Callback invoked for each tracing event, receiving a TracingLogRecord
    pub fn new(on_event: F) -> Self {
        Self { on_event }
    }
}

impl<S, F> Layer<S> for OtlpTracingLayer<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    F: for<'a> Fn(TracingLogRecord<'a>) + 'static,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Get timestamp
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Extract fields using visitor with string arena
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        // Build attributes from collected data
        // The visitor's arena holds any allocated strings
        let attributes: Vec<TracingAttribute<'_>> = visitor
            .attr_keys
            .iter()
            .zip(visitor.attr_values.iter())
            .map(|(key, value)| TracingAttribute {
                key,
                value: value.as_borrowed(),
            })
            .collect();

        // Build TracingLogRecord with borrowed message
        let message_ref = visitor.message.as_deref().unwrap_or("");
        let log_record = TracingLogRecord::new(event.metadata(), attributes, timestamp_nanos)
            .with_body(message_ref);

        // Invoke the callback
        (self.on_event)(log_record);
    }

    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        // TODO

        // let timestamp_nanos = SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap_or_default()
        //     .as_nanos() as u64;

        // // Extract fields from span attributes
        // let mut visitor = FieldVisitor::new();
        // attrs.record(&mut visitor);

        // let metadata = attrs.metadata();
        // let span_id = id.into_u64();

        // let mut attributes: Vec<TracingAttribute<'_>> = data
        //     .attr_keys
        //     .iter()
        //     .zip(data.attr_values.iter())
        //     .map(|(key, value)| TracingAttribute {
        //         key,
        //         value: value.as_borrowed(),
        //     })
        //     .collect();

        // // Add span.id and span.name as attributes
        // let span_id_value = TracingAnyValue::Int(span_id as i64);
        // let span_name_value = TracingAnyValue::Str(data.name);

        // attributes.push(TracingAttribute {
        //     key: "span.id",
        //     value: span_id_value,
        // });
        // attributes.push(TracingAttribute {
        //     key: "span.name",
        //     value: span_name_value,
        // });

        // // Create "span.start" log record
        // let message_ref = visitor.message.as_deref().unwrap_or("");
        // let log_record = TracingLogRecord::new_with_event_name(
        //     metadata,
        //     attributes,
        //     timestamp_nanos,
        //     "span.start",
        // )
        // .with_body(message_ref);

        // // Invoke callback with span start event
        // (self.on_event)(log_record);
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        // TODO
    }
}

/// Visitor that extracts field values from a tracing event.
///
/// This implements tracing::field::Visit to walk through all fields in an event
/// and collect them as attribute key-value pairs.
///
/// **Zero-copy design**: Field names are `&'static str` from tracing.
/// String values that need allocation (debug formatting) are stored in owned form.
struct FieldVisitor {
    /// Attribute keys (all &'static str from field.name())
    attr_keys: Vec<&'static str>,
    /// Attribute values (owned to support debug formatting)
    attr_values: Vec<OwnedValue>,
    /// The message/body (from the "message" field, if present)
    message: Option<String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            attr_keys: Vec::new(),
            attr_values: Vec::new(),
            message: None,
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "message" {
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values.push(OwnedValue::Double(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "message" {
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values.push(OwnedValue::Int(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "message" {
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values.push(OwnedValue::Int(value as i64));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "message" {
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values.push(OwnedValue::Bool(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values.push(OwnedValue::Str(value.to_string()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
            return;
        }
        self.attr_keys.push(field.name());
        self.attr_values
            .push(OwnedValue::Str(format!("{:?}", value)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_otlp_layer_captures_events() {
        use otap_df_pdata::views::logs::LogRecordView;

        // Thread-local storage for captured log records (no Send needed)
        thread_local! {
            static CAPTURED: RefCell<Vec<(Option<String>, Option<String>)>> = const { RefCell::new(Vec::new()) };
        }

        let layer = OtlpTracingLayer::new(|log_record| {
            CAPTURED.with(|captured| {
                captured.borrow_mut().push((
                    log_record
                        .severity_text()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    log_record
                        .event_name()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                ));
            });
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        // Use Dispatch and set_default for thread-local subscriber (no Send+Sync required)
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);

        tracing::info!(name: "test.event", "Test message");
        tracing::warn!(name: "test.warning", "Warning message");

        // Verify captured records
        CAPTURED.with(|captured| {
            let records = captured.borrow();
            assert_eq!(records.len(), 2);
        });
    }
}
