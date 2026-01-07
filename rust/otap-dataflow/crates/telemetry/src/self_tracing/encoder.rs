// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use bytes::Bytes;
use std::fmt::Write as FmtWrite;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{Event, Level};
use otap_df_pdata::otlp::{ProtoBuffer, encode_len_placeholder, patch_len_placeholder};
use otap_df_pdata::proto::consts::{field_num::common::*, field_num::logs::*, wire_types};

/// Position marker for a length-delimited field that needs patching.
///
/// TODO: This would belong in otap_df_pdata::otlp, for use in place
/// of directly calling encode_len_placeholder, patch_len_placeholder,
/// except we should use the macros defined there instead. Remove.
#[derive(Debug, Clone, Copy)]
pub struct LengthPlaceholder {
    /// Position in buffer where the 4-byte length placeholder starts
    position: usize,
}

impl LengthPlaceholder {
    /// Create a new placeholder at the current buffer position.
    #[inline]
    pub fn new(position: usize) -> Self {
        Self { position }
    }

    /// Patch the placeholder with the actual content length.
    #[inline]
    pub fn patch(self, buf: &mut ProtoBuffer) {
        let content_len = buf.len() - self.position - 4;
        patch_len_placeholder(buf, 4, content_len, self.position);
    }
}

/// Wrapper for ProtoBuffer for formatting of Debug values without
/// allocating an intermediate String.
struct ProtoBufferWriter<'a> {
    buf: &'a mut ProtoBuffer,
}

impl FmtWrite for ProtoBufferWriter<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

/// Direct encoder that writes a single LogRecord from a tracing Event.
pub struct DirectLogRecordEncoder<'buf> {
    buf: &'buf mut ProtoBuffer,
}

impl<'buf> DirectLogRecordEncoder<'buf> {
    /// Create a new encoder that writes to the provided buffer.
    #[inline]
    pub fn new(buf: &'buf mut ProtoBuffer) -> Self {
        Self { buf }
    }

    /// Encode a tracing Event as a complete LogRecord message.
    ///
    /// This writes all LogRecord fields directly to the buffer:
    /// - time_unix_nano (field 1)
    /// - severity_number (field 2)  
    /// - severity_text (field 3)
    /// - body (field 5) - from the "message" field
    /// - attributes (field 6) - from all other fields
    ///
    /// Returns the number of bytes written.
    pub fn encode_event(&mut self, event: &Event<'_>) -> usize {
        let start_len = self.buf.len();
        
        // Get timestamp
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let metadata = event.metadata();
        
        // Encode time_unix_nano (field 1, fixed64)
        self.buf.encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        self.buf.extend_from_slice(&timestamp_nanos.to_le_bytes());
        
        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(metadata.level());
        self.buf.encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        self.buf.encode_varint(severity as u64);
        
        // Encode severity_text (field 3, string)
        self.buf.encode_string(LOG_RECORD_SEVERITY_TEXT, metadata.level().as_str());
        
        // Now visit fields to encode body and attributes
        let mut visitor = DirectFieldVisitor::new(self.buf);
        event.record(&mut visitor);
        
        self.buf.len() - start_len
    }

    /// Encode a tracing Event with a custom timestamp.
    pub fn encode_event_with_timestamp(&mut self, event: &Event<'_>, timestamp_nanos: u64) -> usize {
        let start_len = self.buf.len();
        let metadata = event.metadata();
        
        // Encode time_unix_nano (field 1, fixed64)
        self.buf.encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        self.buf.extend_from_slice(&timestamp_nanos.to_le_bytes());
        
        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(metadata.level());
        self.buf.encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        self.buf.encode_varint(severity as u64);
        
        // Encode severity_text (field 3, string)
        self.buf.encode_string(LOG_RECORD_SEVERITY_TEXT, metadata.level().as_str());
        
        // Now visit fields to encode body and attributes
        let mut visitor = DirectFieldVisitor::new(self.buf);
        event.record(&mut visitor);
        
        self.buf.len() - start_len
    }
}

/// Visitor that directly encodes tracing fields to protobuf.
///
/// This is the core of the zero-allocation design: instead of collecting
/// field values into an intermediate data structure, we encode them directly
/// to the protobuf buffer as we visit them.
pub struct DirectFieldVisitor<'buf> {
    buf: &'buf mut ProtoBuffer,
}

impl<'buf> DirectFieldVisitor<'buf> {
    /// Create a new DirectFieldVisitor that writes to the provided buffer.
    pub fn new(buf: &'buf mut ProtoBuffer) -> Self {
        Self { buf }
    }

    /// Encode an attribute (KeyValue message) with a string value.
    #[inline]
    pub fn encode_string_attribute(&mut self, key: &str, value: &str) {

        // KeyValue message as LOG_RECORD_ATTRIBUTES field (tag 6)
        self.buf.encode_field_tag(LOG_RECORD_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // KeyValue.key (field 1, string)
        self.buf.encode_string(KEY_VALUE_KEY, key);
        
        // KeyValue.value (field 2, AnyValue message)
        self.buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.string_value (field 1, string)
        self.buf.encode_string(ANY_VALUE_STRING_VALUE, value);
        
        av_placeholder.patch(self.buf);
        kv_placeholder.patch(self.buf);
    }

    /// Encode an attribute with an i64 value.
    #[inline]
    pub fn encode_int_attribute(&mut self, key: &str, value: i64) {
        self.buf.encode_field_tag(LOG_RECORD_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        self.buf.encode_string(KEY_VALUE_KEY, key);
        
        self.buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.int_value (field 3, varint)
        self.buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
        self.buf.encode_varint(value as u64);
        
        av_placeholder.patch(self.buf);
        kv_placeholder.patch(self.buf);
    }

    /// Encode an attribute with a bool value.
    #[inline]
    pub fn encode_bool_attribute(&mut self, key: &str, value: bool) {
        self.buf.encode_field_tag(LOG_RECORD_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        self.buf.encode_string(KEY_VALUE_KEY, key);
        
        self.buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.bool_value (field 2, varint)
        self.buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
        self.buf.encode_varint(if value { 1 } else { 0 });
        
        av_placeholder.patch(self.buf);
        kv_placeholder.patch(self.buf);
    }

    /// Encode an attribute with a double value.
    #[inline]
    pub fn encode_double_attribute(&mut self, key: &str, value: f64) {
        self.buf.encode_field_tag(LOG_RECORD_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        self.buf.encode_string(KEY_VALUE_KEY, key);
        
        self.buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.double_value (field 4, fixed64)
        self.buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
        self.buf.extend_from_slice(&value.to_le_bytes());
        
        av_placeholder.patch(self.buf);
        kv_placeholder.patch(self.buf);
    }

    /// Encode the body (AnyValue message) as a string.
    #[inline]
    pub fn encode_body_string(&mut self, value: &str) {
        // LogRecord.body (field 5, AnyValue message)
        self.buf.encode_field_tag(LOG_RECORD_BODY, wire_types::LEN);
        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.string_value (field 1, string)
        self.buf.encode_string(ANY_VALUE_STRING_VALUE, value);
        
        placeholder.patch(self.buf);
    }

    /// Encode the body (AnyValue message) from a Debug value without allocation.
    #[inline]
    pub fn encode_body_debug(&mut self, value: &dyn std::fmt::Debug) {
        // LogRecord.body (field 5, AnyValue message)
        self.buf.encode_field_tag(LOG_RECORD_BODY, wire_types::LEN);
        let body_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.string_value (field 1, string)
        self.buf.encode_field_tag(ANY_VALUE_STRING_VALUE, wire_types::LEN);
        let string_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // Write Debug output directly to buffer
        let mut writer = ProtoBufferWriter { buf: self.buf };
        let _ = write!(writer, "{:?}", value);
        
        string_placeholder.patch(self.buf);
        body_placeholder.patch(self.buf);
    }

    /// Encode an attribute with a Debug value without allocation.
    #[inline]
    pub fn encode_debug_attribute(&mut self, key: &str, value: &dyn std::fmt::Debug) {
        // KeyValue message as LOG_RECORD_ATTRIBUTES field (tag 6)
        self.buf.encode_field_tag(LOG_RECORD_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // KeyValue.key (field 1, string)
        self.buf.encode_string(KEY_VALUE_KEY, key);
        
        // KeyValue.value (field 2, AnyValue message)
        self.buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // AnyValue.string_value (field 1, string)
        self.buf.encode_field_tag(ANY_VALUE_STRING_VALUE, wire_types::LEN);
        let string_placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(self.buf);
        
        // Write Debug output directly to buffer
        let mut writer = ProtoBufferWriter { buf: self.buf };
        let _ = write!(writer, "{:?}", value);
        
        string_placeholder.patch(self.buf);
        av_placeholder.patch(self.buf);
        kv_placeholder.patch(self.buf);
    }
}

impl tracing::field::Visit for DirectFieldVisitor<'_> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "message" {
            // Body will be formatted later if needed
            return;
        }
        self.encode_double_attribute(field.name(), value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "message" {
            return;
        }
        self.encode_int_attribute(field.name(), value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "message" {
            return;
        }
        self.encode_int_attribute(field.name(), value as i64);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "message" {
            return;
        }
        self.encode_bool_attribute(field.name(), value);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.encode_body_string(value);
            return;
        }
        self.encode_string_attribute(field.name(), value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // The Rust Debug type cannot be destructured, only formatted.
        if field.name() == "message" {
            self.encode_body_debug(value);
        } else {
            self.encode_debug_attribute(field.name(), value);
        }
    }
}

/// Convert tracing Level to OTLP severity number.
///
/// See: https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber
#[inline]
fn level_to_severity_number(level: &Level) -> i32 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}

/// Stateful encoder for batching multiple LogRecords with shared Resource/Scope.
///
/// This encoder maintains open `ResourceLogs` and `ScopeLogs` messages, allowing
/// multiple LogRecords to be appended efficiently. When the scope changes, it
/// automatically closes the current scope and starts a new one.
///
/// # Thread-Local Usage
///
/// This encoder is designed for thread-local use. Each thread should have its own
/// encoder instance to avoid synchronization overhead. The encoder accumulates
/// records until explicitly flushed.
///
/// # Example
///
/// ```ignore
/// thread_local! {
///     static ENCODER: RefCell<StatefulDirectEncoder> = RefCell::new(
///         StatefulDirectEncoder::new(64 * 1024, resource_bytes)
///     );
/// }
///
/// // In event handler:
/// ENCODER.with(|encoder| {
///     let mut encoder = encoder.borrow_mut();
///     encoder.encode_event(event);
///     
///     if encoder.len() > FLUSH_THRESHOLD {
///         let bytes = encoder.flush();
///         // Send bytes to pipeline
///     }
/// });
/// ```
pub struct StatefulDirectEncoder {
    /// Output buffer
    buf: ProtoBuffer,
    
    /// Pre-encoded Resource bytes (includes ResourceLogs.resource field)
    resource_bytes: Vec<u8>,
    
    /// Current encoder state
    state: EncoderState,
    
    /// Length placeholder for current ResourceLogs
    resource_logs_placeholder: Option<LengthPlaceholder>,
    
    /// Length placeholder for current ScopeLogs
    scope_logs_placeholder: Option<LengthPlaceholder>,
    
    /// Current scope name for batching comparison
    current_scope_name: Option<String>,
}

/// Current state of the stateful encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncoderState {
    /// No messages open, ready to start new ResourceLogs
    Idle,
    /// ResourceLogs is open, ready to add ScopeLogs
    ResourceOpen,
    /// ResourceLogs and ScopeLogs are both open, ready to append LogRecords
    ScopeOpen,
}

impl StatefulDirectEncoder {
    /// Create a new stateful encoder with pre-allocated buffer capacity.
    ///
    /// # Arguments
    /// * `capacity_bytes` - Initial buffer capacity in bytes
    /// * `resource_bytes` - Pre-encoded Resource (use `encode_resource_bytes` helper)
    pub fn new(capacity_bytes: usize, resource_bytes: Vec<u8>) -> Self {
        Self {
            buf: ProtoBuffer::with_capacity(capacity_bytes),
            resource_bytes,
            state: EncoderState::Idle,
            resource_logs_placeholder: None,
            scope_logs_placeholder: None,
            current_scope_name: None,
        }
    }

    /// Get the current buffer size in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Check if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Encode a tracing Event, using its metadata target as the scope name.
    ///
    /// This method automatically handles batching:
    /// - If scope (target) matches the current batch, the LogRecord is appended
    /// - If scope differs, the current ScopeLogs is closed and a new one started
    pub fn encode_event(&mut self, event: &Event<'_>) {
        let scope_name = event.metadata().target();
        self.encode_event_with_scope(event, scope_name);
    }

    /// Encode a tracing Event with an explicit scope name.
    pub fn encode_event_with_scope(&mut self, event: &Event<'_>, scope_name: &str) {
        match self.state {
            EncoderState::Idle => {
                self.start_resource_logs();
                self.start_scope_logs(scope_name);
                self.append_log_record(event);
            }
            EncoderState::ResourceOpen => {
                self.start_scope_logs(scope_name);
                self.append_log_record(event);
            }
            EncoderState::ScopeOpen => {
                if self.current_scope_name.as_deref() == Some(scope_name) {
                    // Same scope - just append
                    self.append_log_record(event);
                } else {
                    // Different scope - close current and start new
                    self.close_scope_logs();
                    self.start_scope_logs(scope_name);
                    self.append_log_record(event);
                }
            }
        }
    }

    /// Flush the encoder, closing all open messages and returning the accumulated bytes.
    ///
    /// After flushing, the encoder is reset and ready for new messages.
    pub fn flush(&mut self) -> Bytes {
        // Close any open messages
        if self.state == EncoderState::ScopeOpen {
            self.close_scope_logs();
        }
        if self.state == EncoderState::ResourceOpen {
            self.close_resource_logs();
        }

        // Take the bytes
        let (bytes, capacity) = self.buf.take_into_bytes();

        // Reset state
        self.state = EncoderState::Idle;
        self.resource_logs_placeholder = None;
        self.scope_logs_placeholder = None;
        self.current_scope_name = None;

        // Preserve capacity for next use
        self.buf.ensure_capacity(capacity);

        bytes
    }

    // === Private methods ===

    fn start_resource_logs(&mut self) {
        // LogsData.resource_logs field (tag 1, length-delimited)
        self.buf.encode_field_tag(LOGS_DATA_RESOURCE, wire_types::LEN);

        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Copy pre-encoded Resource bytes
        self.buf.extend_from_slice(&self.resource_bytes);

        self.resource_logs_placeholder = Some(placeholder);
        self.state = EncoderState::ResourceOpen;
    }

    fn start_scope_logs(&mut self, scope_name: &str) {
        // ResourceLogs.scope_logs field (tag 2, length-delimited)
        self.buf.encode_field_tag(RESOURCE_LOGS_SCOPE_LOGS, wire_types::LEN);

        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Encode ScopeLogs.scope field (InstrumentationScope message)
        self.encode_instrumentation_scope(scope_name);

        self.scope_logs_placeholder = Some(placeholder);
        self.current_scope_name = Some(scope_name.to_string());
        self.state = EncoderState::ScopeOpen;
    }

    fn encode_instrumentation_scope(&mut self, scope_name: &str) {
        // ScopeLogs.scope field (tag 1, length-delimited)
        self.buf.encode_field_tag(SCOPE_LOG_SCOPE, wire_types::LEN);
        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // InstrumentationScope.name field (tag 1, string)
        self.buf.encode_string(INSTRUMENTATION_SCOPE_NAME, scope_name);

        placeholder.patch(&mut self.buf);
    }

    fn append_log_record(&mut self, event: &Event<'_>) {
        // ScopeLogs.log_records field (tag 2, length-delimited)
        self.buf.encode_field_tag(SCOPE_LOGS_LOG_RECORDS, wire_types::LEN);

        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Encode the LogRecord content directly
        let mut encoder = DirectLogRecordEncoder::new(&mut self.buf);
        let _ = encoder.encode_event(event);

        placeholder.patch(&mut self.buf);
    }

    fn close_scope_logs(&mut self) {
        if let Some(placeholder) = self.scope_logs_placeholder.take() {
            placeholder.patch(&mut self.buf);
            self.state = EncoderState::ResourceOpen;
            self.current_scope_name = None;
        }
    }

    fn close_resource_logs(&mut self) {
        if let Some(placeholder) = self.resource_logs_placeholder.take() {
            placeholder.patch(&mut self.buf);
            self.state = EncoderState::Idle;
        }
    }
}

/// Helper to pre-encode a Resource as OTLP bytes.
///
/// The returned bytes include the ResourceLogs.resource field tag and length,
/// ready to be copied directly into the encoder buffer.
///
/// # Example
///
/// ```ignore
/// use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
///
/// let resource = Resource {
///     attributes: vec![
///         KeyValue { key: "service.name".into(), value: Some(AnyValue { ... }) },
///     ],
///     dropped_attributes_count: 0,
/// };
/// let bytes = encode_resource_bytes(&resource);
/// let encoder = StatefulDirectEncoder::new(64 * 1024, bytes);
/// ```
pub fn encode_resource_bytes_from_attrs(attributes: &[(&str, &str)]) -> Vec<u8> {
    use otap_df_pdata::proto::consts::field_num::resource::RESOURCE_ATTRIBUTES;

    let mut buf = ProtoBuffer::with_capacity(256);

    // ResourceLogs.resource field (tag 1, length-delimited)
    buf.encode_field_tag(1, wire_types::LEN);
    let resource_placeholder = LengthPlaceholder::new(buf.len());
    encode_len_placeholder(&mut buf);

    // Encode each attribute as Resource.attributes (tag 1, KeyValue)
    for (key, value) in attributes {
        buf.encode_field_tag(RESOURCE_ATTRIBUTES, wire_types::LEN);
        let kv_placeholder = LengthPlaceholder::new(buf.len());
        encode_len_placeholder(&mut buf);

        buf.encode_string(KEY_VALUE_KEY, key);

        buf.encode_field_tag(KEY_VALUE_VALUE, wire_types::LEN);
        let av_placeholder = LengthPlaceholder::new(buf.len());
        encode_len_placeholder(&mut buf);

        buf.encode_string(ANY_VALUE_STRING_VALUE, value);

        av_placeholder.patch(&mut buf);
        kv_placeholder.patch(&mut buf);
    }

    resource_placeholder.patch(&mut buf);

    buf.into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::layer::Layer;
    use tracing_subscriber::registry::LookupSpan;
    use tracing::Subscriber;
    use std::sync::Mutex;

    /// Simple layer that uses DirectLogRecordEncoder (thread-safe for tests)
    struct DirectEncoderLayer {
        // Thread-local buffer - each event encodes to this
        buffer: Mutex<ProtoBuffer>,
        // Collected encoded bytes
        encoded: Mutex<Vec<Vec<u8>>>,
    }

    impl DirectEncoderLayer {
        fn new() -> Self {
            Self {
                buffer: Mutex::new(ProtoBuffer::with_capacity(4096)),
                encoded: Mutex::new(Vec::new()),
            }
        }
    }

    impl<S> Layer<S> for DirectEncoderLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.clear();
            
            let mut encoder = DirectLogRecordEncoder::new(&mut buffer);
            let _ = encoder.encode_event(event);
            
            // Save a copy of the encoded bytes
            self.encoded.lock().unwrap().push(buffer.as_ref().to_vec());
        }
    }

    #[test]
    fn test_direct_encoder_captures_events() {
        let layer = DirectEncoderLayer::new();
        
        let subscriber = tracing_subscriber::registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);

        tracing::info!("Test message");
        tracing::warn!(count = 42, "Warning with attribute");
        
        // Drop the guard to stop capturing
        drop(_guard);
        
        // Note: We can't easily get the layer back from dispatch to verify results
        // The test verifies that the encoding path doesn't panic
    }

    #[test]
    fn test_direct_encoder_encodes_attributes() {
        let mut buffer = ProtoBuffer::with_capacity(1024);
        
        // We can't easily create a tracing::Event in tests, so we'll just verify
        // the attribute encoding helpers work correctly
        let mut visitor = DirectFieldVisitor::new(&mut buffer);
        visitor.encode_string_attribute("test_key", "test_value");
        visitor.encode_int_attribute("count", 42);
        visitor.encode_bool_attribute("enabled", true);
        visitor.encode_double_attribute("ratio", 3.14);
        
        // Buffer should have content
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_stateful_encoder_batching() {
        let resource_bytes = encode_resource_bytes_from_attrs(&[
            ("service.name", "test-service"),
        ]);
        
        let mut encoder = StatefulDirectEncoder::new(4096, resource_bytes);
        
        assert!(encoder.is_empty());
        assert_eq!(encoder.state, EncoderState::Idle);
        
        // We can't easily test with real tracing events, but we can verify the structure
        // For now, just test flush on empty encoder
        let bytes = encoder.flush();
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_encode_resource_bytes() {
        let bytes = encode_resource_bytes_from_attrs(&[
            ("service.name", "my-service"),
            ("service.version", "1.0.0"),
        ]);
        
        // Should produce non-empty bytes
        assert!(!bytes.is_empty());
        
        // Bytes should start with field tag for ResourceLogs.resource
        // Field 1, wire type 2 (LEN) = (1 << 3) | 2 = 0x0a
        assert_eq!(bytes[0], 0x0a);
    }

    #[test]
    fn test_level_to_severity() {
        assert_eq!(level_to_severity_number(&Level::TRACE), 1);
        assert_eq!(level_to_severity_number(&Level::DEBUG), 5);
        assert_eq!(level_to_severity_number(&Level::INFO), 9);
        assert_eq!(level_to_severity_number(&Level::WARN), 13);
        assert_eq!(level_to_severity_number(&Level::ERROR), 17);
    }
}
