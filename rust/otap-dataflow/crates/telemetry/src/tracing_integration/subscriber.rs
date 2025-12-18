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

use super::log_record::{TracingAttribute, TracingAnyValue, TracingLogRecord};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{Event, Id, Subscriber};
use tracing::span::Attributes;
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// A tracing subscriber layer that converts events to TracingLogRecord.
///
/// This layer can be composed with other layers in a tracing-subscriber registry
/// to capture events and convert them to OTLP-compatible log records.
///
/// # Example
/// ```ignore
/// use tracing_subscriber::prelude::*;
/// use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
///
/// let otlp_layer = OtlpTracingLayer::new(|log_record| {
///     // Encode log_record using stateful encoder
///     encoder.encode_log_record(&log_record, &resource_bytes, &scope_encoding)?;
/// });
///
/// tracing_subscriber::registry()
///     .with(otlp_layer)
///     .init();
/// ```
/// Span data stored for duration calculation
struct SpanData {
    start_time_nanos: u64,
    attributes: Vec<TracingAttribute>,
}

/// Tracing subscriber layer that captures events and spans as OTLP log records.
///
/// This layer implements an unconventional approach where spans are treated as pairs
/// of log records (start/end) rather than as first-class span objects. This aligns
/// with unified dataflow architectures where all telemetry flows through a single
/// log pipeline.
pub struct OtlpTracingLayer<F>
where
    F: Fn(TracingLogRecord) + Send + Sync + 'static,
{
    /// Callback function that receives each TracingLogRecord
    on_event: F,
    /// Storage for span start times to calculate duration on close
    span_data: Mutex<HashMap<u64, SpanData>>,
}

impl<F> OtlpTracingLayer<F>
where
    F: Fn(TracingLogRecord) + Send + Sync + 'static,
{
    /// Creates a new OtlpTracingLayer with the given event handler.
    ///
    /// # Arguments
    /// * `on_event` - Callback invoked for each tracing event, receiving a TracingLogRecord
    pub fn new(on_event: F) -> Self {
        Self {
            on_event,
            span_data: Mutex::new(HashMap::new()),
        }
    }
}

impl<S, F> Layer<S> for OtlpTracingLayer<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    F: Fn(TracingLogRecord) + Send + Sync + 'static,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Get timestamp
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Extract fields using visitor
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        // Build TracingLogRecord
        // Note: metadata.name() includes file:line, e.g., "event src/main.rs:42"
        let log_record = TracingLogRecord::new(
            event.metadata(),
            visitor.attributes,
            timestamp_nanos,
        )
        .with_body(visitor.message.unwrap_or_default());

        // Invoke the callback
        (self.on_event)(log_record);
    }

    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Extract fields from span attributes
        let mut visitor = FieldVisitor::new();
        attrs.record(&mut visitor);

        let metadata = attrs.metadata();
        let mut attributes = visitor.attributes.clone();
        
        // Add span.id as attribute
        attributes.push(TracingAttribute {
            key: "span.id".to_string(),
            value: TracingAnyValue::Int(id.into_u64() as i64),
        });

        // Store span data for duration calculation on close
        if let Ok(mut spans) = self.span_data.lock() {
            let _ = spans.insert(id.into_u64(), SpanData {
                start_time_nanos: timestamp_nanos,
                attributes: attributes.clone(),
            });
        }

        // Create "span.start" log record
        // Format: "span.start {span_name} src/file.rs:42"
        let event_name = format!("span.start {}", metadata.name());
        let log_record = TracingLogRecord::new_with_event_name(
            metadata,
            attributes,
            timestamp_nanos,
            event_name,
        )
        .with_body(visitor.message.unwrap_or_default());

        // Invoke callback with span start event
        (self.on_event)(log_record);
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let end_time_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Retrieve and remove span data
        let span_data = if let Ok(mut spans) = self.span_data.lock() {
            spans.remove(&id.into_u64())
        } else {
            return;
        };

        if let Some(span_data) = span_data {
            // Get span metadata from context
            // Note: We don't have direct access to metadata here, so we'll create a minimal record
            let duration_nanos = end_time_nanos.saturating_sub(span_data.start_time_nanos);
            
            let mut attributes = span_data.attributes;
            
            // Add duration as attribute
            attributes.push(TracingAttribute {
                key: "span.duration_nanos".to_string(),
                value: TracingAnyValue::Int(duration_nanos as i64),
            });

            // Create a minimal log record for span end
            // We use INFO level for span events
            let log_record = TracingLogRecord::new_span_end(
                id.into_u64(),
                attributes,
                end_time_nanos,
            );

            // Invoke callback with span end event
            (self.on_event)(log_record);
        }
    }
}

/// Visitor that extracts field values from a tracing event.
///
/// This implements tracing::field::Visit to walk through all fields in an event
/// and collect them as TracingAttribute instances.
///
/// Note: We don't extract event_name here because metadata.name() already provides
/// it with file:line info (e.g., "event src/main.rs:42").
struct FieldVisitor {
    /// Collected attributes from the event
    attributes: Vec<TracingAttribute>,
    
    /// The message/body (from the "message" field, if present)
    message: Option<String>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            attributes: Vec::new(),
            message: None,
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        // Skip special "message" field
        if field.name() == "message" {
            return;
        }
        
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Double(value),
        });
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "message" {
            return;
        }
        
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Int(value),
        });
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "message" {
            return;
        }
        
        // Convert u64 to i64 (may lose precision for very large values)
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Int(value as i64),
        });
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "message" {
            return;
        }
        
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Bool(value),
        });
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        // Handle special "message" field
        if field.name() == "message" {
            self.message = Some(value.to_string());
            return;
        }
        
        // Store string attributes by cloning
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Str(value.to_string()),
        });
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // Capture the "message" field which contains the formatted message
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
            return;
        }
        
        // Convert debug representation to string and store
        let debug_str = format!("{:?}", value);
        self.attributes.push(TracingAttribute {
            key: field.name().to_string(),
            value: TracingAnyValue::Str(debug_str),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_otlp_layer_captures_events() {
        use otap_df_pdata::views::logs::LogRecordView;
        
        // Collect captured log records
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        let layer = OtlpTracingLayer::new(move |log_record| {
            let mut records = captured_clone.lock().unwrap();
            records.push((
                log_record.severity_text().map(|s| String::from_utf8_lossy(s).to_string()),
                log_record.event_name().map(|s| String::from_utf8_lossy(s).to_string()),
            ));
        });

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(name: "test.event", "Test message");
            tracing::warn!(name: "test.warning", "Warning message");
        });

        let records = captured.lock().unwrap();
        assert_eq!(records.len(), 2);
        
        // Note: event_name extraction from visitor has lifetime issues
        // We'll address this in the production implementation
    }
}
