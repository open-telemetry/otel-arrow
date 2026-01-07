// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use std::fmt::Write as FmtWrite;
use tracing::Level;
use super::{LogRecord, SavedCallsite};
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

    /// Reset the underlying buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Encode a tracing Event as a complete LogRecord message.
    ///
    /// Returns the number of bytes written.
    pub fn encode_log_record(&mut self, record: LogRecord, callsite: &SavedCallsite) -> usize {
        let start_len = self.buf.len();
        
        // Encode time_unix_nano (field 1, fixed64)
        self.buf.encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        self.buf.extend_from_slice(&record.timestamp_ns.to_le_bytes());

        // Note: the next two fields could be pre-encoded by Level
        
        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(&callsite.level);
        self.buf.encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        self.buf.encode_varint(severity as u64);
        
        // Encode severity_text (field 3, string)
        self.buf.encode_string(LOG_RECORD_SEVERITY_TEXT, callsite.level.as_str());

        self.buf.extend_from_slice(&record.body_attrs_bytes);
        
        self.buf.len() - start_len
    }
}

/// Visitor that directly encodes tracing fields to protobuf.
pub struct DirectFieldVisitor<'buf> {
    buf: &'buf mut ProtoBuffer,
}

impl<'buf> FmtWrite for DirectFieldVisitor<'buf> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
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
        let _ = write!(self, "{:?}", value);
        
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
        let _ = write!(self, "{:?}", value);
        
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
pub fn level_to_severity_number(level: &Level) -> u8 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}
