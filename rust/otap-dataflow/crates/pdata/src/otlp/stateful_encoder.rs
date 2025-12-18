// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Stateful OTLP encoder for streaming single log records with automatic batching.
//!
//! This encoder maintains open `ResourceLogs` and `ScopeLogs` messages, appending individual
//! `LogRecord`s as they arrive. When the InstrumentationScope changes (via scope ID), it automatically
//! closes the previous scope and starts a new one. The Resource is pre-encoded and copied once.
//!
//! # Design
//! - **Resource**: Pre-encoded as `OtlpBytes` (includes protobuf field tag + length + content)
//! - **Scope**: Pre-encoded with unique ID for fast comparison
//! - **LogRecord**: Accepted as `LogRecordView` trait, encoded on-the-fly

use crate::error::Result;
use crate::otlp::common::{ProtoBuffer, encode_len_placeholder, patch_len_placeholder};
use crate::proto::consts::{field_num::logs::*, wire_types};
use crate::views::logs::LogRecordView;
use bytes::Bytes;

/// Pre-encoded OTLP bytes (includes protobuf field tag + length + message content)
///
/// These bytes are ready to be copied directly into the output buffer without further processing.
pub type OtlpBytes = Vec<u8>;
/// @@@ Remove me, use super::OtlpProtoBytes

/// Unique identifier for a scope encoding
///
/// This is typically a static reference to a constant or a pre-allocated ID.
pub type ScopeId = usize;
/// @@@ Encapsulate, not alias: pub struct ScopeId(usize)

/// Pre-encoded scope information with unique identifier
///
/// Each effect handler registers its scope once and receives a unique ID.
/// The encoder compares scope IDs (not hashes) for fast equality checking.
#[derive(Debug, Clone)]
pub struct ScopeEncoding {
    /// Unique identifier for this scope (used for fast comparison)
    pub id: ScopeId,
    /// Pre-encoded ScopeLogs.scope field (protobuf tag + length + InstrumentationScope message)
    pub encoded_bytes: OtlpBytes, // @@@ should be super::OtlpProtoBytes expecting ::ExportLogsRequest
}

/// Position marker for a length-delimited field that needs patching
///
/// @@@ Make this variable width. We want 2-byte padding for records
/// and 4-byte padding for the container messages ResourceLogs,
/// ScopeLogs, etc, because it is reasonable to insist on 16 KiB log
/// messages for a self-diagnostic library and we are able to drop
/// attributes to achieve this (OTLP has a dedicated field for this).
/// Using a <const WIDTH: usize> maybe, or a <T> for the primitive u16, u32.
#[derive(Debug, Clone, Copy)]
struct LengthPlaceholder {
    /// Position in buffer where the 4-byte length placeholder starts
    position: usize,
}

impl LengthPlaceholder {
    fn new(position: usize) -> Self {
        Self { position }
    }

    fn patch(self, buf: &mut ProtoBuffer) {
        let content_len = buf.len() - self.position - 4;
        patch_len_placeholder(buf, 4, content_len, self.position);
    }
}

/// Current state of the stateful encoder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncoderState {
    /// No messages open, ready to start new ResourceLogs
    Idle,
    /// ResourceLogs is open, ready to add ScopeLogs
    ResourceOpen,
    /// ResourceLogs and ScopeLogs are both open, ready to append LogRecords
    ScopeOpen,
}

/// Stateful OTLP encoder that maintains open ResourceLogs and ScopeLogs messages.
///
/// # Example
/// ```ignore
/// let mut encoder = StatefulOtlpEncoder::new(64 * 1024);
///
/// // Pre-encode resource once
/// let resource_bytes = encode_resource_to_otlp_bytes(&resource);
///
/// // Register scope once per effect handler
/// let scope_encoding = ScopeEncoding {
///     id: 42, // Unique ID for this scope
///     encoded_bytes: encode_scope_to_otlp_bytes(&scope),
/// };
///
/// // Encode multiple log records - automatically batched if scope ID matches
/// encoder.encode_log_record(&log_record_view, &resource_bytes, &scope_encoding)?;
/// encoder.encode_log_record(&log_record_view2, &resource_bytes, &scope_encoding)?;  // Batched
///
/// // Flush to get OTLP bytes
/// let otlp_bytes = encoder.flush();
/// ```
pub struct StatefulOtlpEncoder {
    /// Output buffer (reuses ProtoBuffer infrastructure)
    buf: ProtoBuffer,

    /// Current encoder state
    state: EncoderState,

    /// Length placeholder for the current ResourceLogs message
    resource_logs_placeholder: Option<LengthPlaceholder>,

    /// Length placeholder for the current ScopeLogs message
    scope_logs_placeholder: Option<LengthPlaceholder>,

    /// ID of the current scope for fast comparison
    current_scope_id: Option<ScopeId>,
}

impl StatefulOtlpEncoder {
    /// Create a new encoder with pre-allocated buffer capacity
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            buf: ProtoBuffer::with_capacity(capacity_bytes),
            state: EncoderState::Idle,
            resource_logs_placeholder: None,
            scope_logs_placeholder: None,
            current_scope_id: None,
        }
    }

    /// Get the current buffer size in bytes
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Encode a single log record with its pre-encoded Resource and Scope context.
    ///
    /// This method automatically handles batching:
    /// - If Scope ID matches the current batch, the LogRecord is appended
    /// - If Scope ID differs, the current ScopeLogs is closed and a new one started
    ///
    /// # Parameters
    /// - `log_record`: View of the log record to encode
    /// - `resource_bytes`: Pre-encoded Resource (includes protobuf field tag + length + content)
    /// - `scope_encoding`: Pre-encoded Scope with unique ID
    pub fn encode_log_record(
        &mut self,
        log_record: &impl LogRecordView,
        resource_bytes: &[u8], // @@@ Make super::OtlpProtoBytes, expecting ::ExportLogsRequest
        scope_encoding: &ScopeEncoding,
    ) -> Result<()> {
        let scope_id = scope_encoding.id;

        match self.state {
            EncoderState::Idle => {
                // Start new batch with Resource and Scope
                self.start_resource_logs(resource_bytes)?;
                self.start_scope_logs(scope_encoding)?;
                self.append_log_record(log_record)?;
            }

            EncoderState::ResourceOpen => {
                // Resource already open, start scope
                self.start_scope_logs(scope_encoding)?;
                self.append_log_record(log_record)?;
            }

            EncoderState::ScopeOpen => {
                if Some(scope_id) == self.current_scope_id {
                    // Same scope - just append LogRecord
                    self.append_log_record(log_record)?;
                } else {
                    // Different scope - close current and start new
                    self.close_scope_logs()?;
                    self.start_scope_logs(scope_encoding)?;
                    self.append_log_record(log_record)?;
                }
            }
        }

        Ok(())
    }

    /// Flush the encoder, closing all open messages and returning the accumulated OTLP bytes.
    ///
    /// After flushing, the encoder is reset and ready for new messages.
    pub fn flush(&mut self) -> Bytes {
        // Close any open messages
        if self.state == EncoderState::ScopeOpen {
            let _ = self.close_scope_logs();
        }
        if self.state == EncoderState::ResourceOpen || self.state == EncoderState::ScopeOpen {
            let _ = self.close_resource_logs();
        }

        // Take the bytes and reset the encoder
        let (bytes, capacity) = self.buf.take_into_bytes();

        // Reset state
        self.state = EncoderState::Idle;
        self.resource_logs_placeholder = None;
        self.scope_logs_placeholder = None;
        self.current_scope_id = None;

        // Ensure capacity is preserved for next use
        self.buf.ensure_capacity(capacity);

        bytes
    }

    // === Private state management methods ===

    fn start_resource_logs(&mut self, resource_bytes: &[u8]) -> Result<()> {
        // Encode LogsData.resource_logs field (tag 1, length-delimited)
        self.buf
            .encode_field_tag(LOGS_DATA_RESOURCE, wire_types::LEN);

        // Write 4-byte length placeholder
        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Copy pre-encoded Resource bytes (includes ResourceLogs.resource field)
        self.buf.extend_from_slice(resource_bytes);

        // Update state
        self.resource_logs_placeholder = Some(placeholder);
        self.state = EncoderState::ResourceOpen;

        Ok(())
    }

    fn start_scope_logs(&mut self, scope_encoding: &ScopeEncoding) -> Result<()> {
        // Encode ResourceLogs.scope_logs field (tag 2, length-delimited)
        self.buf
            .encode_field_tag(RESOURCE_LOGS_SCOPE_LOGS, wire_types::LEN);

        // Write 4-byte length placeholder
        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Copy pre-encoded Scope bytes (includes ScopeLogs.scope field)
        self.buf.extend_from_slice(&scope_encoding.encoded_bytes);

        // Update state
        self.scope_logs_placeholder = Some(placeholder);
        self.current_scope_id = Some(scope_encoding.id);
        self.state = EncoderState::ScopeOpen;

        Ok(())
    }

    fn append_log_record(&mut self, log_record: &impl LogRecordView) -> Result<()> {
        // Encode ScopeLogs.log_records field (tag 2, length-delimited)
        self.buf
            .encode_field_tag(SCOPE_LOGS_LOG_RECORDS, wire_types::LEN);

        // Use 4-byte padding for LogRecord
        let placeholder = LengthPlaceholder::new(self.buf.len());
        encode_len_placeholder(&mut self.buf);

        // Encode LogRecordView fields
        encode_log_record_view(log_record, &mut self.buf)?;

        // Patch the length
        placeholder.patch(&mut self.buf);

        Ok(())
    }

    fn close_scope_logs(&mut self) -> Result<()> {
        if let Some(placeholder) = self.scope_logs_placeholder.take() {
            placeholder.patch(&mut self.buf);
            self.state = EncoderState::ResourceOpen;
            self.current_scope_id = None;
        }
        Ok(())
    }

    fn close_resource_logs(&mut self) -> Result<()> {
        if let Some(placeholder) = self.resource_logs_placeholder.take() {
            placeholder.patch(&mut self.buf);
            self.state = EncoderState::Idle;
        }
        Ok(())
    }
}

// === Helper functions for encoding LogRecordView ===

/// Encode all fields of a LogRecordView
fn encode_log_record_view(log_record: &impl LogRecordView, buf: &mut ProtoBuffer) -> Result<()> {
    // time_unix_nano (field 1, fixed64)
    if let Some(time) = log_record.time_unix_nano() {
        buf.encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        buf.extend_from_slice(&time.to_le_bytes());
    }

    // severity_number (field 2, varint)
    if let Some(severity) = log_record.severity_number() {
        buf.encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        buf.encode_varint(severity as u64);
    }

    // severity_text (field 3, string)
    if let Some(text) = log_record.severity_text() {
        if !text.is_empty() {
            // Convert &[u8] to &str for encode_string
            if let Ok(text_str) = std::str::from_utf8(text) {
                buf.encode_string(LOG_RECORD_SEVERITY_TEXT, text_str);
            }
        }
    }

    // body (field 5, AnyValue) - encode from AnyValueView
    if let Some(body) = log_record.body() {
        encode_any_value_view_field(LOG_RECORD_BODY, &body, buf)?;
    }

    // attributes (field 6, repeated KeyValue) - encode from AttributeView iterator
    for attr in log_record.attributes() {
        encode_attribute_view(LOG_RECORD_ATTRIBUTES, &attr, buf)?;
    }

    // dropped_attributes_count (field 7, uint32)
    let dropped = log_record.dropped_attributes_count();
    if dropped > 0 {
        buf.encode_field_tag(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
        buf.encode_varint(dropped as u64);
    }

    // flags (field 8, fixed32)
    if let Some(flags) = log_record.flags() {
        buf.encode_field_tag(LOG_RECORD_FLAGS, wire_types::FIXED32);
        buf.extend_from_slice(&flags.to_le_bytes());
    }

    // trace_id (field 9, bytes)
    if let Some(trace_id) = log_record.trace_id() {
        buf.encode_bytes(LOG_RECORD_TRACE_ID, trace_id);
    }

    // span_id (field 10, bytes)
    if let Some(span_id) = log_record.span_id() {
        buf.encode_bytes(LOG_RECORD_SPAN_ID, span_id);
    }

    // observed_time_unix_nano (field 11, fixed64)
    if let Some(observed_time) = log_record.observed_time_unix_nano() {
        buf.encode_field_tag(LOG_RECORD_OBSERVED_TIME_UNIX_NANO, wire_types::FIXED64);
        buf.extend_from_slice(&observed_time.to_le_bytes());
    }

    Ok(())
}

/// Encode an AttributeView as a length-delimited field
fn encode_attribute_view(
    field_tag: u64,
    attr: &impl crate::views::common::AttributeView,
    buf: &mut ProtoBuffer,
) -> Result<()> {
    use crate::proto::consts::field_num::common::*;

    // Start KeyValue message
    buf.encode_field_tag(field_tag, wire_types::LEN);
    let placeholder = LengthPlaceholder::new(buf.len());
    encode_len_placeholder(buf);

    // Encode key
    let key = attr.key();
    if !key.is_empty() {
        // Convert &[u8] to &str for encode_string
        if let Ok(key_str) = std::str::from_utf8(key) {
            buf.encode_string(KEY_VALUE_KEY, key_str);
        }
    }

    // Encode value (if present)
    if let Some(value) = attr.value() {
        encode_any_value_view_field(KEY_VALUE_VALUE, &value, buf)?;
    }

    // Patch length
    placeholder.patch(buf);

    Ok(())
}

/// Encode an AnyValueView as a length-delimited field
fn encode_any_value_view_field<'a>(
    field_tag: u64,
    value: &impl crate::views::common::AnyValueView<'a>,
    buf: &mut ProtoBuffer,
) -> Result<()> {
    buf.encode_field_tag(field_tag, wire_types::LEN);
    let placeholder = LengthPlaceholder::new(buf.len());
    encode_len_placeholder(buf);

    encode_any_value_view_content(value, buf)?;

    placeholder.patch(buf);
    Ok(())
}

/// Encode the content of an AnyValueView (without the outer field tag)
fn encode_any_value_view_content<'a>(
    value: &impl crate::views::common::AnyValueView<'a>,
    buf: &mut ProtoBuffer,
) -> Result<()> {
    use crate::proto::consts::field_num::common::*;
    use crate::views::common::ValueType;

    match value.value_type() {
        ValueType::String => {
            if let Some(s) = value.as_string() {
                // Convert &[u8] to &str for encode_string
                if let Ok(s_str) = std::str::from_utf8(s) {
                    buf.encode_string(ANY_VALUE_STRING_VALUE, s_str);
                }
            }
        }
        ValueType::Bool => {
            if let Some(b) = value.as_bool() {
                buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                buf.encode_varint(if b { 1 } else { 0 });
            }
        }
        ValueType::Int64 => {
            if let Some(i) = value.as_int64() {
                buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                buf.encode_varint(i as u64);
            }
        }
        ValueType::Double => {
            if let Some(d) = value.as_double() {
                buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                buf.extend_from_slice(&d.to_le_bytes());
            }
        }
        ValueType::Bytes => {
            if let Some(bytes) = value.as_bytes() {
                buf.encode_bytes(ANY_VALUE_BYTES_VALUE, bytes);
            }
        }
        ValueType::Array => {
            if let Some(mut arr_iter) = value.as_array() {
                // Encode ArrayValue
                buf.encode_field_tag(ANY_VALUE_ARRAY_VALUE, wire_types::LEN);
                let placeholder = LengthPlaceholder::new(buf.len());
                encode_len_placeholder(buf);

                while let Some(val) = arr_iter.next() {
                    encode_any_value_view_field(ARRAY_VALUE_VALUES, &val, buf)?;
                }

                placeholder.patch(buf);
            }
        }
        ValueType::KeyValueList => {
            if let Some(mut kvlist_iter) = value.as_kvlist() {
                // Encode KeyValueList
                buf.encode_field_tag(ANY_VALUE_KVLIST_VALUE, wire_types::LEN);
                let placeholder = LengthPlaceholder::new(buf.len());
                encode_len_placeholder(buf);

                while let Some(kv) = kvlist_iter.next() {
                    encode_attribute_view(KEY_VALUE_LIST_VALUES, &kv, buf)?;
                }

                placeholder.patch(buf);
            }
        }
        ValueType::Empty => {
            // Empty AnyValue - valid according to spec
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::schema::{SpanId, TraceId};
    use crate::views::common::{AnyValueView, AttributeView, Str, ValueType};
    use crate::views::logs::LogRecordView;

    // Test helper: Simple LogRecordView implementation
    struct SimpleLogRecord {
        time_unix_nano: Option<u64>,
        severity_number: Option<i32>,
        severity_text: Option<&'static str>,
        body: Option<&'static str>,
        trace_id: Option<TraceId>,
        span_id: Option<SpanId>,
    }

    impl LogRecordView for SimpleLogRecord {
        type Attribute<'a>
            = SimpleAttribute
        where
            Self: 'a;
        type AttributeIter<'a>
            = std::iter::Empty<Self::Attribute<'a>>
        where
            Self: 'a;
        type Body<'a>
            = SimpleAnyValue
        where
            Self: 'a;

        fn time_unix_nano(&self) -> Option<u64> {
            self.time_unix_nano
        }

        fn observed_time_unix_nano(&self) -> Option<u64> {
            self.time_unix_nano // same for tests
        }

        fn severity_number(&self) -> Option<i32> {
            self.severity_number
        }

        fn severity_text(&self) -> Option<Str<'_>> {
            self.severity_text.map(|s| s.as_bytes())
        }

        fn body(&self) -> Option<Self::Body<'_>> {
            self.body.map(|s| SimpleAnyValue::String(s))
        }

        fn attributes(&self) -> Self::AttributeIter<'_> {
            std::iter::empty()
        }

        fn dropped_attributes_count(&self) -> u32 {
            0
        }

        fn flags(&self) -> Option<u32> {
            Some(0)
        }

        fn trace_id(&self) -> Option<&TraceId> {
            self.trace_id.as_ref()
        }

        fn span_id(&self) -> Option<&SpanId> {
            self.span_id.as_ref()
        }

        fn event_name(&self) -> Option<Str<'_>> {
            None
        }
    }

    #[derive(Clone)]
    enum SimpleAnyValue {
        String(&'static str),
    }

    impl<'a> AnyValueView<'a> for SimpleAnyValue {
        type KeyValue = SimpleAttribute;
        type ArrayIter<'arr>
            = std::iter::Empty<Self>
        where
            Self: 'arr;
        type KeyValueIter<'kv>
            = SimpleAttribute
        where
            Self: 'kv;

        fn value_type(&self) -> ValueType {
            match self {
                SimpleAnyValue::String(_) => ValueType::String,
            }
        }

        fn as_string(&self) -> Option<Str<'_>> {
            match self {
                SimpleAnyValue::String(s) => Some(s.as_bytes()),
            }
        }

        fn as_bool(&self) -> Option<bool> {
            None
        }

        fn as_int64(&self) -> Option<i64> {
            None
        }

        fn as_double(&self) -> Option<f64> {
            None
        }

        fn as_bytes(&self) -> Option<&[u8]> {
            None
        }

        fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
            None
        }

        fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
            None
        }
    }

    #[derive(Clone)]
    struct SimpleAttribute;

    impl AttributeView for SimpleAttribute {
        type Val<'val>
            = SimpleAnyValue
        where
            Self: 'val;

        fn key(&self) -> Str<'_> {
            "key".as_bytes()
        }

        fn value(&self) -> Option<Self::Val<'_>> {
            Some(SimpleAnyValue::String("value"))
        }
    }

    impl Iterator for SimpleAttribute {
        type Item = Self;

        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }

    // Helper: Pre-encode a Resource as OtlpBytes
    fn encode_resource_bytes(resource: &Resource) -> OtlpBytes {
        use crate::proto::consts::field_num::resource::*;
        let mut buf = ProtoBuffer::with_capacity(256);

        // Encode ResourceLogs.resource field (tag 1)
        buf.encode_field_tag(1, wire_types::LEN);
        let start = buf.len();
        encode_len_placeholder(&mut buf);

        // Encode attributes
        for attr in &resource.attributes {
            encode_attribute_proto(RESOURCE_ATTRIBUTES, attr, &mut buf);
        }

        // Patch length
        let content_len = buf.len() - start - 4;
        patch_len_placeholder(&mut buf, 4, content_len, start);

        buf.into_bytes().to_vec()
    }

    // Helper: Pre-encode a Scope as OtlpBytes
    fn encode_scope_bytes(scope: &InstrumentationScope) -> OtlpBytes {
        use crate::proto::consts::field_num::common::*;
        let mut buf = ProtoBuffer::with_capacity(256);

        // Encode ScopeLogs.scope field (tag 1)
        buf.encode_field_tag(1, wire_types::LEN);
        let start = buf.len();
        encode_len_placeholder(&mut buf);

        // Encode name
        if !scope.name.is_empty() {
            buf.encode_string(INSTRUMENTATION_SCOPE_NAME, &scope.name);
        }

        // Encode version
        if !scope.version.is_empty() {
            buf.encode_string(INSTRUMENTATION_SCOPE_VERSION, &scope.version);
        }

        // Patch length
        let content_len = buf.len() - start - 4;
        patch_len_placeholder(&mut buf, 4, content_len, start);

        buf.into_bytes().to_vec()
    }

    // Helper to encode protobuf KeyValue (for test helpers)
    fn encode_attribute_proto(field_tag: u64, attr: &KeyValue, buf: &mut ProtoBuffer) {
        use crate::proto::consts::field_num::common::*;
        buf.encode_field_tag(field_tag, wire_types::LEN);
        let start = buf.len();
        encode_len_placeholder(buf);

        if !attr.key.is_empty() {
            buf.encode_string(KEY_VALUE_KEY, &attr.key);
        }

        if let Some(ref value) = attr.value {
            encode_any_value_proto(KEY_VALUE_VALUE, value, buf);
        }

        let content_len = buf.len() - start - 4;
        patch_len_placeholder(buf, 4, content_len, start);
    }

    fn encode_any_value_proto(field_tag: u64, value: &AnyValue, buf: &mut ProtoBuffer) {
        use crate::proto::consts::field_num::common::*;
        buf.encode_field_tag(field_tag, wire_types::LEN);
        let start = buf.len();
        encode_len_placeholder(buf);

        match &value.value {
            Some(any_value::Value::StringValue(s)) => {
                buf.encode_string(ANY_VALUE_STRING_VALUE, s);
            }
            _ => {}
        }

        let content_len = buf.len() - start - 4;
        patch_len_placeholder(buf, 4, content_len, start);
    }

    #[test]
    fn test_encoder_state_machine() {
        let mut encoder = StatefulOtlpEncoder::new(1024);

        // Initial state
        assert_eq!(encoder.state, EncoderState::Idle);
        assert!(encoder.is_empty());

        // Pre-encode resource and scope
        let resource = Resource::default();
        let scope = InstrumentationScope::default();
        let resource_bytes = encode_resource_bytes(&resource);
        let scope_encoding = ScopeEncoding {
            id: 1,
            encoded_bytes: encode_scope_bytes(&scope),
        };

        // Simple log record
        let log_record = SimpleLogRecord {
            time_unix_nano: Some(1000),
            severity_number: Some(9),
            severity_text: Some("INFO"),
            body: Some("test message"),
            trace_id: None,
            span_id: None,
        };

        encoder
            .encode_log_record(&log_record, &resource_bytes, &scope_encoding)
            .unwrap();

        // Should have data now
        assert!(!encoder.is_empty());
        assert_eq!(encoder.state, EncoderState::ScopeOpen);

        // Flush should reset
        let bytes = encoder.flush();
        assert!(!bytes.is_empty());
        assert_eq!(encoder.state, EncoderState::Idle);
    }

    #[test]
    fn test_batching_same_scope() {
        let mut encoder = StatefulOtlpEncoder::new(1024);

        let resource = Resource::default();
        let scope = InstrumentationScope::default();
        let resource_bytes = encode_resource_bytes(&resource);
        let scope_encoding = ScopeEncoding {
            id: 1,
            encoded_bytes: encode_scope_bytes(&scope),
        };

        // Encode three records with same scope
        for i in 0..3 {
            let log_record = SimpleLogRecord {
                time_unix_nano: Some(i as u64),
                severity_number: Some(9),
                severity_text: Some("INFO"),
                body: Some("test"),
                trace_id: None,
                span_id: None,
            };
            encoder
                .encode_log_record(&log_record, &resource_bytes, &scope_encoding)
                .unwrap();
        }

        // Should be in ScopeOpen state (not closed between records)
        assert_eq!(encoder.state, EncoderState::ScopeOpen);

        let bytes = encoder.flush();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_different_scopes_close_and_reopen() {
        let mut encoder = StatefulOtlpEncoder::new(4096);

        let resource = Resource::default();
        let resource_bytes = encode_resource_bytes(&resource);

        let scope1 = InstrumentationScope {
            name: "scope1".into(),
            version: "1.0".into(),
            attributes: vec![],
            dropped_attributes_count: 0,
        };

        let scope2 = InstrumentationScope {
            name: "scope2".into(),
            version: "1.0".into(),
            attributes: vec![],
            dropped_attributes_count: 0,
        };

        let scope1_encoding = ScopeEncoding {
            id: 1,
            encoded_bytes: encode_scope_bytes(&scope1),
        };

        let scope2_encoding = ScopeEncoding {
            id: 2,
            encoded_bytes: encode_scope_bytes(&scope2),
        };

        let log_record = SimpleLogRecord {
            time_unix_nano: Some(1000),
            severity_number: Some(9),
            severity_text: Some("INFO"),
            body: Some("test"),
            trace_id: None,
            span_id: None,
        };

        // Encode with scope1
        encoder
            .encode_log_record(&log_record, &resource_bytes, &scope1_encoding)
            .unwrap();
        assert_eq!(encoder.state, EncoderState::ScopeOpen);

        // Encode with scope2 - should close scope1 and start scope2
        encoder
            .encode_log_record(&log_record, &resource_bytes, &scope2_encoding)
            .unwrap();
        assert_eq!(encoder.state, EncoderState::ScopeOpen);

        let bytes = encoder.flush();
        assert!(!bytes.is_empty());
    }
}
