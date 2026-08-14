// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Borrowed pdata views over serialized OTLP log protobuf messages.
//!
//! # Purpose
//!
//! The types in this module expose the pdata log view traits without first decoding the protobuf
//! into an owned Prost object tree. Field values remain slices of the caller-owned byte buffer,
//! and nested messages are represented by lightweight borrowed views. This is useful on paths
//! that only need selected fields or want to transform OTLP bytes directly into another format.
//!
//! The view hierarchy follows the OTLP logs schema:
//!
//! ```text
//! RawLogsData
//!   `- ResourceLogsIter -> RawResourceLogs
//!        `- ScopeLogsIter -> RawScopeLogs
//!             `- LogRecordsIter -> RawLogRecord
//! ```
//!
//! Resource, instrumentation-scope, attribute, and `AnyValue` submessages reuse the raw views in
//! the sibling `resource` and `common` modules.
//!
//! # Validation and lazy access
//!
//! [`RawLogsData::try_new`] performs an allocation-free validation pass over the top-level message
//! before exposing the view. It validates top-level protobuf framing, while length-delimited nested
//! messages remain opaque until a consumer accesses them. Unknown fields retain protobuf forward
//! compatibility and are skipped according to their framing.
//!
//! Child views use [`ProtoBytesParser`] to discover fields lazily. Parser clones and repeated-field
//! iterators share scan progress and cached byte ranges through `Rc<Cell<_>>`. This avoids eagerly
//! indexing every field, but means these views are intentionally not `Send` and are not designed
//! for concurrent access. Malformed or wrongly typed nested fields may appear absent or stop the
//! affected iterator, matching the best-effort behavior of the other raw OTLP byte views.
//!
//! [`RawLogsData::new`] and [`RawLogRecord::new`] are unchecked constructors for trusted or
//! internal bytes when the caller does not need top-level framing validation.

use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::OtlpProtoBytes;
use crate::error::Error;
use crate::proto::consts::field_num::logs::{
    LOG_RECORD_ATTRIBUTES, LOG_RECORD_BODY, LOG_RECORD_DROPPED_ATTRIBUTES_COUNT,
    LOG_RECORD_EVENT_NAME, LOG_RECORD_FLAGS, LOG_RECORD_OBSERVED_TIME_UNIX_NANO,
    LOG_RECORD_SEVERITY_NUMBER, LOG_RECORD_SEVERITY_TEXT, LOG_RECORD_SPAN_ID,
    LOG_RECORD_TIME_UNIX_NANO, LOG_RECORD_TRACE_ID, LOGS_DATA_RESOURCE, RESOURCE_LOGS_RESOURCE,
    RESOURCE_LOGS_SCHEMA_URL, RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOG_SCOPE, SCOPE_LOGS_LOG_RECORDS,
    SCOPE_LOGS_SCHEMA_URL,
};
use crate::proto::consts::wire_types;
use crate::schema::{SpanId, TraceId};
use crate::views::otlp::bytes::common::{
    KeyValueIter, RawAnyValue, RawInstrumentationScope, RawKeyValue,
};
use crate::views::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, RepeatedFieldProtoBytesParser,
    from_option_nonzero_range_to_primitive, read_dropped_count, read_len_delim, read_varint,
    to_nonzero_range,
};
use crate::views::otlp::bytes::resource::RawResource;
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};

/// Root borrowed view over a serialized OTLP logs request.
///
/// `LogsData` and `ExportLogsServiceRequest` have the same top-level repeated resource-logs field,
/// so this view accepts either wire representation. It owns no payload data.
pub struct RawLogsData<'a> {
    /// Bytes of the serialized message.
    buf: &'a [u8],
}

impl<'a> RawLogsData<'a> {
    /// Create a root view without validating the serialized message.
    ///
    /// This constructor is intended only for trusted or already validated internal data. Prefer
    /// [`Self::try_new`] at an ingestion boundary.
    #[must_use]
    pub const fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    /// Construct a [`RawLogsData`] after validating top-level protobuf wire framing.
    ///
    /// Cost: a single linear walk of `buf` with no allocations. Each top-level field tag is
    /// decoded, and each value is bounds-checked against the end of `buf`. Length-delimited nested
    /// messages are not recursively validated; their fields are interpreted lazily on access.
    pub fn try_new(buf: &'a [u8]) -> Result<Self, Error> {
        let mut pos = 0;
        while pos < buf.len() {
            let (tag, next) = read_varint(buf, pos).ok_or(Error::InvalidProtobufWireFormat)?;
            let field_num = tag >> 3;
            let wire_type = tag & 7;

            if field_num == 0 {
                return Err(Error::InvalidProtobufWireFormat);
            }

            pos = match wire_type {
                wire_types::VARINT => {
                    let (_, p) = read_varint(buf, next).ok_or(Error::InvalidProtobufWireFormat)?;
                    p
                }
                wire_types::LEN => {
                    let (len, p) =
                        read_varint(buf, next).ok_or(Error::InvalidProtobufWireFormat)?;
                    let end = p
                        .checked_add(
                            usize::try_from(len).map_err(|_| Error::InvalidProtobufWireFormat)?,
                        )
                        .ok_or(Error::InvalidProtobufWireFormat)?;
                    if end > buf.len() {
                        return Err(Error::InvalidProtobufWireFormat);
                    }
                    end
                }
                wire_types::FIXED64 => {
                    let end = next
                        .checked_add(8)
                        .ok_or(Error::InvalidProtobufWireFormat)?;
                    if end > buf.len() {
                        return Err(Error::InvalidProtobufWireFormat);
                    }
                    end
                }
                wire_types::FIXED32 => {
                    let end = next
                        .checked_add(4)
                        .ok_or(Error::InvalidProtobufWireFormat)?;
                    if end > buf.len() {
                        return Err(Error::InvalidProtobufWireFormat);
                    }
                    end
                }
                _ => return Err(Error::InvalidProtobufWireFormat),
            };
        }

        Ok(Self { buf })
    }
}

impl<'a> TryFrom<&'a OtlpProtoBytes> for RawLogsData<'a> {
    type Error = Error;

    fn try_from(bytes: &'a OtlpProtoBytes) -> Result<Self, Self::Error> {
        match bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => Self::try_new(bytes),
            _ => Err(Error::LogRecordNotFound),
        }
    }
}

/// Borrowed `ResourceLogsView` backed by a serialized `ResourceLogs` message.
pub struct RawResourceLogs<'a> {
    byte_parser: ProtoBytesParser<'a, ResourceLogsFieldOffsets>,
}

/// Lazily cached byte ranges for fields used from a `ResourceLogs` message.
///
/// Only the first repeated `scope_logs` range is cached; its iterator scans subsequent values.
pub struct ResourceLogsFieldOffsets {
    resource: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_scope_logs: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ResourceLogsFieldOffsets {
    fn new() -> Self {
        Self {
            resource: Cell::new(None),
            schema_url: Cell::new(None),
            first_scope_logs: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            RESOURCE_LOGS_RESOURCE => self.resource.get(),
            RESOURCE_LOGS_SCHEMA_URL => self.schema_url.get(),
            RESOURCE_LOGS_SCOPE_LOGS => self.first_scope_logs.get(),
            _ => None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };

        if wire_type == wire_types::LEN {
            match field_num {
                RESOURCE_LOGS_RESOURCE => self.resource.set(range),
                RESOURCE_LOGS_SCHEMA_URL => self.schema_url.set(range),
                RESOURCE_LOGS_SCOPE_LOGS if self.first_scope_logs.get().is_none() => {
                    self.first_scope_logs.set(range);
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Borrowed `ScopeLogsView` backed by a serialized `ScopeLogs` message.
pub struct RawScopeLogs<'a> {
    byte_parser: ProtoBytesParser<'a, ScopeLogsFieldOffsets>,
}

/// Lazily cached byte ranges for fields used from a `ScopeLogs` message.
///
/// Only the first repeated log-record range is cached; its iterator scans subsequent values.
pub struct ScopeLogsFieldOffsets {
    scope: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_log_record: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ScopeLogsFieldOffsets {
    fn new() -> Self {
        Self {
            scope: Cell::new(None),
            schema_url: Cell::new(None),
            first_log_record: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SCOPE_LOG_SCOPE => self.scope.get(),
            SCOPE_LOGS_SCHEMA_URL => self.schema_url.get(),
            SCOPE_LOGS_LOG_RECORDS => self.first_log_record.get(),
            _ => None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };
        if wire_type == wire_types::LEN {
            match field_num {
                SCOPE_LOG_SCOPE => self.scope.set(range),
                SCOPE_LOGS_SCHEMA_URL => self.schema_url.set(range),
                SCOPE_LOGS_LOG_RECORDS if self.first_log_record.get().is_none() => {
                    self.first_log_record.set(range)
                }
                _ => { /* ignore unknown field */ }
            }
        }
    }
}

/// Borrowed `LogRecordView` backed by a serialized `LogRecord` message.
pub struct RawLogRecord<'a> {
    bytes_parser: ProtoBytesParser<'a, LogFieldOffsets>,
}

impl<'a> RawLogRecord<'a> {
    /// Create an unchecked view of an internally generated serialized log record.
    ///
    /// This is exposed specifically for records that encode body and attributes as OTLP bytes.
    /// External request bytes should be validated through [`RawLogsData::try_new`] before child
    /// views are constructed.
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            bytes_parser: ProtoBytesParser::new(buf),
        }
    }
}

/// Lazily cached byte ranges for fields used from a `LogRecord` message.
///
/// Scalar ranges share a field-number-indexed array. Only the first attribute range is cached;
/// the attribute iterator continues scanning subsequent values as needed.
pub struct LogFieldOffsets {
    scalar_fields: [Cell<Option<(NonZeroUsize, NonZeroUsize)>>; 13],
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for LogFieldOffsets {
    fn new() -> Self {
        Self {
            scalar_fields: std::array::from_fn(|_| Cell::new(None)),
            first_attribute: Cell::new(None),
        }
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        const WIRE_TYPES: [u64; 13] = [
            0,                   // unused
            wire_types::FIXED64, // time_unix_nano = 1
            wire_types::VARINT,  // severity_number = 2
            wire_types::LEN,     // severity_text = 3
            0,                   // unused
            wire_types::LEN,     // body = 5
            wire_types::LEN,     // attributes = 6
            wire_types::VARINT,  // dropped_attributes_count = 7
            wire_types::FIXED32, // flags = 8
            wire_types::LEN,     // trace_id = 9
            wire_types::LEN,     // span_id = 10
            wire_types::FIXED64, // observed_time_unix_nano = 11
            wire_types::LEN,     // event_name = 12
        ];

        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };

        if field_num == LOG_RECORD_ATTRIBUTES {
            if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                self.first_attribute.set(range);
            }
        } else if field_num < 13 {
            let idx = field_num as usize;
            if wire_type == WIRE_TYPES[idx] {
                self.scalar_fields[idx].set(range);
            }
        }
    }

    #[inline]
    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = if field_num == LOG_RECORD_ATTRIBUTES {
            self.first_attribute.get()
        } else {
            self.scalar_fields
                .get(field_num as usize)
                .and_then(|c| c.get())
        };

        from_option_nonzero_range_to_primitive(range)
    }
}

/* ----------------------------- ADAPTER ITERATORS ----------------------- */

/// Iterator of borrowed resource-log views in their protobuf wire order.
pub struct ResourceLogsIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for ResourceLogsIter<'a> {
    type Item = RawResourceLogs<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buf.len() {
            let (tag, next_pos) = read_varint(self.buf, self.pos)?;
            self.pos = next_pos;
            let field = tag >> 3;
            let wire_type = tag & 7;
            if field == LOGS_DATA_RESOURCE && wire_type == wire_types::LEN {
                let (slice, next_pos) = read_len_delim(self.buf, self.pos)?;
                self.pos = next_pos;
                return Some(RawResourceLogs {
                    byte_parser: ProtoBytesParser::new(slice),
                });
            }
        }

        None
    }
}

/// Iterator of borrowed scope-log views in their parent resource's protobuf wire order.
pub struct ScopeLogsIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ResourceLogsFieldOffsets>,
}

impl<'a> Iterator for ScopeLogsIter<'a> {
    type Item = RawScopeLogs<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawScopeLogs {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of borrowed log-record views in their parent scope's protobuf wire order.
pub struct LogRecordsIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ScopeLogsFieldOffsets>,
}

impl<'a> Iterator for LogRecordsIter<'a> {
    type Item = RawLogRecord<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(RawLogRecord::new(self.byte_parser.next()?))
    }
}

/* ----------------------------- TRAIT IMPLEMENTATIONS ------------------- */

impl LogsDataView for RawLogsData<'_> {
    type ResourceLogs<'a>
        = RawResourceLogs<'a>
    where
        Self: 'a;

    type ResourcesIter<'a>
        = ResourceLogsIter<'a>
    where
        Self: 'a;

    #[inline]
    fn resources(&self) -> Self::ResourcesIter<'_> {
        ResourceLogsIter {
            buf: self.buf,
            pos: 0,
        }
    }
}

impl ResourceLogsView for RawResourceLogs<'_> {
    type Resource<'res>
        = RawResource<'res>
    where
        Self: 'res;
    type ScopeLogs<'scp>
        = RawScopeLogs<'scp>
    where
        Self: 'scp;
    type ScopesIter<'scp>
        = ScopeLogsIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_LOGS_RESOURCE)?;

        Some(RawResource::new(ProtoBytesParser::new(slice)))
    }

    #[inline]
    fn schema_url(&self) -> Option<otap_df_pdata_views::views::common::Str<'_>> {
        self.byte_parser
            .advance_to_find_field(RESOURCE_LOGS_SCHEMA_URL)
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeLogsIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                RESOURCE_LOGS_SCOPE_LOGS,
                wire_types::LEN,
            ),
        }
    }
}

impl ScopeLogsView for RawScopeLogs<'_> {
    type LogRecord<'rec>
        = RawLogRecord<'rec>
    where
        Self: 'rec;
    type LogRecordsIter<'rec>
        = LogRecordsIter<'rec>
    where
        Self: 'rec;
    type Scope<'scp>
        = RawInstrumentationScope<'scp>
    where
        Self: 'scp;

    #[inline]
    fn log_records(&self) -> Self::LogRecordsIter<'_> {
        LogRecordsIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SCOPE_LOGS_LOG_RECORDS,
                wire_types::LEN,
            ),
        }
    }

    #[inline]
    fn schema_url(&self) -> Option<otap_df_pdata_views::views::common::Str<'_>> {
        self.byte_parser
            .advance_to_find_field(SCOPE_LOGS_SCHEMA_URL)
    }

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        let slice = self.byte_parser.advance_to_find_field(SCOPE_LOG_SCOPE)?;
        Some(RawInstrumentationScope::new(ProtoBytesParser::new(slice)))
    }
}

impl LogRecordView for RawLogRecord<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, LogFieldOffsets>
    where
        Self: 'att;

    type Body<'bod>
        = RawAnyValue<'bod>
    where
        Self: 'bod;

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            LOG_RECORD_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    #[inline]
    fn body(&self) -> Option<Self::Body<'_>> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_BODY)
            .map(RawAnyValue::new)
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT);
        read_dropped_count(slice)
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        let slice = self.bytes_parser.advance_to_find_field(LOG_RECORD_FLAGS)?;
        let byte_arr: [u8; 4] = slice.try_into().ok()?;
        Some(u32::from_le_bytes(byte_arr))
    }

    #[inline]
    fn observed_time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_OBSERVED_TIME_UNIX_NANO)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    #[inline]
    fn severity_number(&self) -> Option<i32> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_SEVERITY_NUMBER)?;
        let (val, _) = read_varint(slice, 0)?;
        Some(val as i32)
    }

    #[inline]
    fn severity_text(&self) -> Option<otap_df_pdata_views::views::common::Str<'_>> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_SEVERITY_TEXT)
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_SPAN_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_TIME_UNIX_NANO)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_TRACE_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn event_name(&self) -> Option<otap_df_pdata_views::views::common::Str<'_>> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_EVENT_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: an empty byte slice represents an empty OTLP logs request.
    /// Guarantees: validation accepts the request and exposes no resources.
    #[test]
    fn try_new_accepts_valid_empty_request() {
        let logs = RawLogsData::try_new(&[]).expect("empty request is valid protobuf");
        assert_eq!(logs.resources().count(), 0);
    }

    /// Scenario: a top-level protobuf tag is an unterminated varint.
    /// Guarantees: validation rejects malformed top-level wire framing.
    #[test]
    fn try_new_rejects_malformed_varint() {
        assert!(matches!(
            RawLogsData::try_new(b"\xff"),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a top-level length-delimited field extends beyond the request buffer.
    /// Guarantees: validation rejects the truncated field before a view can traverse it.
    #[test]
    fn try_new_rejects_truncated_length_delimited_field() {
        assert!(matches!(
            RawLogsData::try_new(&[0x0a, 0x05, 0x00]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a top-level field declares a length larger than a 32-bit address space.
    /// Guarantees: validation rejects lengths that overflow `usize` or exceed the input buffer.
    #[test]
    fn try_new_rejects_oversized_length_delimited_field() {
        assert!(matches!(
            RawLogsData::try_new(&[0x0a, 0x80, 0x80, 0x80, 0x80, 0x10]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a top-level varint value is truncated at the end of the request.
    /// Guarantees: validation rejects trailing partial values.
    #[test]
    fn try_new_rejects_trailing_partial_varint() {
        assert!(matches!(
            RawLogsData::try_new(&[0x08, 0x80]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a protobuf tag uses the reserved field number zero.
    /// Guarantees: validation rejects invalid field numbers.
    #[test]
    fn try_new_rejects_field_number_zero() {
        assert!(matches!(
            RawLogsData::try_new(&[0x00]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a valid unknown top-level length-delimited field is present.
    /// Guarantees: validation preserves protobuf forward compatibility for unknown fields.
    #[test]
    fn try_new_accepts_unknown_fields() {
        assert!(RawLogsData::try_new(&[0xa2, 0x06, 0x00]).is_ok());
    }

    /// Scenario: a nested ScopeLogs field declares a length beyond its ResourceLogs boundary.
    /// Guarantees: top-level construction succeeds and lazy scope traversal ignores the field.
    #[test]
    fn try_new_accepts_truncated_nested_field() {
        let logs = RawLogsData::try_new(&[0x0a, 0x03, 0x1a, 0x05, 0x00])
            .expect("top-level framing is valid");
        let resource_logs = logs.resources().next().expect("resource logs");

        assert_eq!(resource_logs.scopes().count(), 0);
    }

    /// Scenario: a known nested ResourceLogs field uses the wrong protobuf wire type.
    /// Guarantees: top-level construction defers interpretation to the lazy nested view.
    #[test]
    fn try_new_accepts_wrong_wire_type_for_nested_field() {
        assert!(RawLogsData::try_new(&[0x0a, 0x02, 0x08, 0x01]).is_ok());
    }
}
