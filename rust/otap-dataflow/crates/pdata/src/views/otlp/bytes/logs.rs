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
//! [`RawLogsData::try_new`] performs an allocation-free validation pass before exposing the view.
//! It validates protobuf framing, the wire types of known fields, and the complete known nested
//! logs hierarchy. Unknown fields retain protobuf forward compatibility: their framing is checked,
//! but length-delimited unknown payloads remain opaque. Semantic constraints such as UTF-8 and ID
//! lengths are left to typed accessors and consumers.
//!
//! After validation, child views use [`ProtoBytesParser`] to discover fields lazily. Parser clones
//! and repeated-field iterators share scan progress and cached byte ranges through `Rc<Cell<_>>`.
//! This avoids eagerly indexing every field, but means these views are intentionally not `Send`
//! and are not designed for concurrent access. A validated request therefore incurs one framing
//! scan followed by lazy field scans, while still avoiding an owned decoded representation.
//!
//! [`RawLogsData::new`] and [`RawLogRecord::new`] are unchecked constructors for trusted or
//! already validated internal bytes. Untrusted serialized requests should enter through
//! [`RawLogsData::try_new`]; otherwise malformed nested fields can appear indistinguishable from
//! absent fields to the optional view accessors.
//!
//! # Schema-aware wire validation
//!
//! Protobuf's length-delimited wire type is shared by strings, bytes, and embedded messages, so
//! wire framing alone cannot determine which payloads require recursive validation.
//! [`MessageKind`] supplies that schema context. For each known field it records the expected wire
//! type and, for embedded messages, the kind to validate recursively. This table duplicates the
//! portion of the OTLP protobuf schema traversed by these raw views and must stay synchronized with
//! `logs.proto`, `common.proto`, `resource.proto`, and the accessors below.

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
use crate::proto::consts::field_num::{common as common_fields, resource as resource_fields};
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

/// Maximum number of embedded known messages accepted by recursive wire validation.
const MAX_NESTED_MESSAGE_DEPTH: usize = 64;

/// Schema context for one protobuf message reachable from an OTLP logs request.
///
/// This is not a protobuf field value or an OTLP signal discriminator. It tells
/// [`validate_message`] how to interpret field numbers whose wire representation alone is
/// ambiguous. See [`Self::field_spec`] for the schema table.
#[derive(Clone, Copy)]
enum MessageKind {
    /// `LogsData` or the wire-compatible `ExportLogsServiceRequest` envelope.
    LogsData,
    /// One `ResourceLogs` message.
    ResourceLogs,
    /// One `Resource` message from `resource.proto`.
    Resource,
    /// One `ScopeLogs` message.
    ScopeLogs,
    /// One `InstrumentationScope` message from `common.proto`.
    InstrumentationScope,
    /// One `LogRecord` message.
    LogRecord,
    /// One attribute `KeyValue` message from `common.proto`.
    KeyValue,
    /// One `AnyValue` message from `common.proto`.
    AnyValue,
    /// The embedded message used by the `AnyValue.array_value` variant.
    ArrayValue,
    /// The embedded message used by the `AnyValue.kvlist_value` variant.
    KeyValueList,
}

impl MessageKind {
    /// Return the expected wire type and optional nested message kind for a known field.
    ///
    /// `None` means the field number is unknown in this message and is accepted after generic
    /// framing validation. `Some((wire_type, None))` identifies a known scalar, string, or byte
    /// field. `Some((wire_type, Some(kind)))` identifies an embedded message whose contents must
    /// be recursively validated as `kind`.
    fn field_spec(self, field_num: u64) -> Option<(u64, Option<Self>)> {
        use MessageKind::{
            AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList, LogRecord,
            Resource, ResourceLogs, ScopeLogs,
        };

        let spec = match (self, field_num) {
            (Self::LogsData, LOGS_DATA_RESOURCE) => (wire_types::LEN, Some(ResourceLogs)),
            (ResourceLogs, RESOURCE_LOGS_RESOURCE) => (wire_types::LEN, Some(Resource)),
            (ResourceLogs, RESOURCE_LOGS_SCOPE_LOGS) => (wire_types::LEN, Some(ScopeLogs)),
            (ResourceLogs, RESOURCE_LOGS_SCHEMA_URL) => (wire_types::LEN, None),
            (Resource, resource_fields::RESOURCE_ATTRIBUTES) => (wire_types::LEN, Some(KeyValue)),
            (Resource, resource_fields::RESOURCE_DROPPED_ATTRIBUTES_COUNT) => {
                (wire_types::VARINT, None)
            }
            (ScopeLogs, SCOPE_LOG_SCOPE) => (wire_types::LEN, Some(InstrumentationScope)),
            (ScopeLogs, SCOPE_LOGS_LOG_RECORDS) => (wire_types::LEN, Some(LogRecord)),
            (ScopeLogs, SCOPE_LOGS_SCHEMA_URL) => (wire_types::LEN, None),
            (InstrumentationScope, common_fields::INSTRUMENTATION_SCOPE_NAME)
            | (InstrumentationScope, common_fields::INSTRUMENTATION_SCOPE_VERSION) => {
                (wire_types::LEN, None)
            }
            (InstrumentationScope, common_fields::INSTRUMENTATION_SCOPE_ATTRIBUTES) => {
                (wire_types::LEN, Some(KeyValue))
            }
            (InstrumentationScope, common_fields::INSTRUMENTATION_DROPPED_ATTRIBUTES_COUNT) => {
                (wire_types::VARINT, None)
            }
            (LogRecord, LOG_RECORD_TIME_UNIX_NANO)
            | (LogRecord, LOG_RECORD_OBSERVED_TIME_UNIX_NANO) => (wire_types::FIXED64, None),
            (LogRecord, LOG_RECORD_SEVERITY_NUMBER)
            | (LogRecord, LOG_RECORD_DROPPED_ATTRIBUTES_COUNT) => (wire_types::VARINT, None),
            (LogRecord, LOG_RECORD_SEVERITY_TEXT)
            | (LogRecord, LOG_RECORD_TRACE_ID)
            | (LogRecord, LOG_RECORD_SPAN_ID)
            | (LogRecord, LOG_RECORD_EVENT_NAME) => (wire_types::LEN, None),
            (LogRecord, LOG_RECORD_BODY) => (wire_types::LEN, Some(AnyValue)),
            (LogRecord, LOG_RECORD_ATTRIBUTES) => (wire_types::LEN, Some(KeyValue)),
            (LogRecord, LOG_RECORD_FLAGS) => (wire_types::FIXED32, None),
            (KeyValue, common_fields::KEY_VALUE_KEY) => (wire_types::LEN, None),
            (KeyValue, common_fields::KEY_VALUE_VALUE) => (wire_types::LEN, Some(AnyValue)),
            (AnyValue, common_fields::ANY_VALUE_STRING_VALUE)
            | (AnyValue, common_fields::ANY_VALUE_BYTES_VALUE) => (wire_types::LEN, None),
            (AnyValue, common_fields::ANY_VALUE_BOOL_VALUE)
            | (AnyValue, common_fields::ANY_VALUE_INT_VALUE) => (wire_types::VARINT, None),
            (AnyValue, common_fields::ANY_VALUE_DOUBLE_VALUE) => (wire_types::FIXED64, None),
            (AnyValue, common_fields::ANY_VALUE_ARRAY_VALUE) => (wire_types::LEN, Some(ArrayValue)),
            (AnyValue, common_fields::ANY_VALUE_KVLIST_VALUE) => {
                (wire_types::LEN, Some(KeyValueList))
            }
            (ArrayValue, common_fields::ARRAY_VALUE_VALUES) => (wire_types::LEN, Some(AnyValue)),
            (KeyValueList, common_fields::KEY_VALUE_LIST_VALUES) => {
                (wire_types::LEN, Some(KeyValue))
            }
            _ => return None,
        };
        Some(spec)
    }
}

/// Recursively validate framing and known-field wire types for one serialized message.
///
/// The walk allocates no heap data and rejects invalid tags, truncated values, unsupported wire
/// types, known fields encoded with the wrong wire type, and excessive known-message nesting.
/// Unknown fields are skipped according to their encoded wire type.
fn validate_message(buf: &[u8], message: MessageKind, depth: usize) -> Result<(), Error> {
    if depth > MAX_NESTED_MESSAGE_DEPTH {
        return Err(Error::InvalidProtobufWireFormat);
    }

    let mut pos = 0;
    while pos < buf.len() {
        let (tag, next) = read_varint(buf, pos).ok_or(Error::InvalidProtobufWireFormat)?;
        let field_num = tag >> 3;
        let wire_type = tag & 7;
        if field_num == 0 {
            return Err(Error::InvalidProtobufWireFormat);
        }

        let (start, end) =
            crate::views::otlp::bytes::decode::field_value_range(buf, wire_type, next)
                .ok_or(Error::InvalidProtobufWireFormat)?;
        if let Some((expected_wire_type, nested_message)) = message.field_spec(field_num) {
            if wire_type != expected_wire_type {
                return Err(Error::InvalidProtobufWireFormat);
            }
            if let Some(nested_message) = nested_message {
                validate_message(&buf[start..end], nested_message, depth + 1)?;
            }
        }
        pos = end;
    }
    Ok(())
}

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

    /// Construct a [`RawLogsData`] after validating protobuf wire framing.
    ///
    /// The recursive, allocation-free validation walk covers the known OTLP logs hierarchy.
    /// Unknown fields remain opaque but their wire framing is validated. Nesting is capped to
    /// keep stack usage bounded.
    pub fn try_new(buf: &'a [u8]) -> Result<Self, Error> {
        validate_message(buf, MessageKind::LogsData, 0)?;
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

    fn length_delimited_field(tag: u8, payload: &[u8]) -> Vec<u8> {
        let mut encoded = vec![tag];
        let mut len = payload.len();
        loop {
            let mut byte = (len & 0x7f) as u8;
            len >>= 7;
            if len != 0 {
                byte |= 0x80;
            }
            encoded.push(byte);
            if len == 0 {
                break;
            }
        }
        encoded.extend_from_slice(payload);
        encoded
    }

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

    /// Scenario: a ResourceLogs string declares a length beyond its nested message boundary.
    /// Guarantees: validation rejects malformed nested framing instead of exposing unsafe ranges.
    #[test]
    fn try_new_rejects_truncated_nested_field() {
        assert!(matches!(
            RawLogsData::try_new(&[0x0a, 0x03, 0x1a, 0x05, 0x00]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: a known nested ResourceLogs field uses the wrong protobuf wire type.
    /// Guarantees: validation rejects data that the typed raw view cannot interpret faithfully.
    #[test]
    fn try_new_rejects_wrong_wire_type_for_known_field() {
        assert!(matches!(
            RawLogsData::try_new(&[0x0a, 0x02, 0x08, 0x01]),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }

    /// Scenario: recursive AnyValue arrays exceed the validation nesting limit.
    /// Guarantees: validation rejects excessive nesting before stack use can grow without bound.
    #[test]
    fn validation_rejects_excessive_message_nesting() {
        let mut any_value = Vec::new();
        for _ in 0..=MAX_NESTED_MESSAGE_DEPTH {
            let array_value = length_delimited_field(0x0a, &any_value);
            any_value = length_delimited_field(0x2a, &array_value);
        }

        assert!(matches!(
            validate_message(&any_value, MessageKind::AnyValue, 0),
            Err(Error::InvalidProtobufWireFormat)
        ));
    }
}
