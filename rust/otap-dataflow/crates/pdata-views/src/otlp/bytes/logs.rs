// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for messages defined in logs.proto

use std::borrow::Cow;
use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::otlp::bytes::common::{KeyValueIter, RawAnyValue, RawInstrumentationScope, RawKeyValue};
use crate::otlp::bytes::consts::field_num::logs::{
    LOG_RECORD_ATTRIBUTES, LOG_RECORD_BODY, LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, LOG_RECORD_FLAGS,
    LOG_RECORD_OBSERVED_TIME_UNIX_NANO, LOG_RECORD_SEVERITY_NUMBER, LOG_RECORD_SEVERITY_TEXT,
    LOG_RECORD_SPAN_ID, LOG_RECORD_TIME_UNIX_NANO, LOG_RECORD_TRACE_ID, LOGS_DATA_RESOURCE,
    RESOURCE_LOGS_RESOURCE, RESOURCE_LOGS_SCHEMA_URL, RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOG_SCOPE,
    SCOPE_LOGS_LOG_RECORDS, SCOPE_LOGS_SCHEMA_URL,
};
use crate::otlp::bytes::consts::wire_types;
use crate::otlp::bytes::decode::{read_len_delim, read_varint, FieldOffsets, ProtoBytesParser, RepeatedFieldProtoBytesParser};
use crate::otlp::bytes::resource::RawResource;
use crate::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};

/// Implementation of `LogsDataView` backed by protobuf serialized `LogsData` message
pub struct RawLogsData<'a> {
    /// bytes of the serialized message
    buf: &'a [u8],
}

impl<'a> RawLogsData<'a> {
    /// Create a new instance of `RawLogsData`
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

/// Implementation of `ResourceLogsView` backed by protobuf serialized `ResourceLogs` message
pub struct RawResourceLogs<'a> {
    byte_parser: ProtoBytesParser<'a, ResourceLogsFieldOffsets>,
}

/// Known field offsets within byte buffer for fields in ResourceLogs message
pub struct ResourceLogsFieldOffsets {
    resource: Cell<Option<NonZeroUsize>>,
    schema_url: Cell<Option<NonZeroUsize>>,
    first_scope_logs: Cell<Option<NonZeroUsize>>,
}

impl FieldOffsets for ResourceLogsFieldOffsets {
    fn new() -> Self {
        Self {
            resource: Cell::new(None),
            schema_url: Cell::new(None),
            first_scope_logs: Cell::new(None),
        }
    }

    fn get_field_offset(&self, field_num: u64) -> Option<usize> {
        match field_num {
            RESOURCE_LOGS_RESOURCE => self.resource.get().map(NonZeroUsize::get),
            RESOURCE_LOGS_SCHEMA_URL => self.schema_url.get().map(NonZeroUsize::get),
            RESOURCE_LOGS_SCOPE_LOGS => self.first_scope_logs.get().map(NonZeroUsize::get),
            _ => None,
        }
    }

    fn set_field_offset(&self, field_num: u64, wire_type: u64, offset: usize) {
        if wire_type == wire_types::LEN {
            let nz = NonZeroUsize::new(offset);
            match field_num {
                RESOURCE_LOGS_RESOURCE => self.resource.set(nz),
                RESOURCE_LOGS_SCHEMA_URL => self.schema_url.set(nz),
                RESOURCE_LOGS_SCOPE_LOGS => {
                    if self.first_scope_logs.get().is_none() {
                        self.first_scope_logs.set(nz);
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of `ScopeLogsView` backed by protobuf serialized `ScopeLogs` message
pub struct RawScopeLogs<'a> {
    byte_parser: ProtoBytesParser<'a, ScopeLogsFieldOffsets>,
}

/// Known field offsets within byte buffer for fields in ResourceLogs message
pub struct ScopeLogsFieldOffsets {
    scope: Cell<Option<NonZeroUsize>>,
    schema_url: Cell<Option<NonZeroUsize>>,
    first_log_record: Cell<Option<NonZeroUsize>>,
}

impl FieldOffsets for ScopeLogsFieldOffsets {
    fn new() -> Self {
        Self {
            scope: Cell::new(None),
            schema_url: Cell::new(None),
            first_log_record: Cell::new(None),
        }
    }

    fn get_field_offset(&self, field_num: u64) -> Option<usize> {
        match field_num {
            SCOPE_LOG_SCOPE => self.scope.get().map(NonZeroUsize::get),
            SCOPE_LOGS_SCHEMA_URL => self.schema_url.get().map(NonZeroUsize::get),
            SCOPE_LOGS_LOG_RECORDS => self.first_log_record.get().map(NonZeroUsize::get),
            _ => None,
        }
    }

    fn set_field_offset(&self, field_num: u64, wire_type: u64, offset: usize) {
        if wire_type == wire_types::LEN {
            let nz = NonZeroUsize::new(offset);
            match field_num {
                SCOPE_LOG_SCOPE => self.scope.set(nz),
                SCOPE_LOGS_SCHEMA_URL => self.schema_url.set(nz),
                SCOPE_LOGS_LOG_RECORDS => {
                    if self.first_log_record.get().is_none() {
                        self.first_log_record.set(nz)
                    }
                }
                _ => { /* ignore unknown field */ }
            }
        }
    }
}

/// Implementation of `LogRecordView` backed by protobuf serialized `LogRecord` message
pub struct RawLogRecord<'a> {
    bytes_parser: ProtoBytesParser<'a, LogFieldOffsets>,
}

/// Known field offsets within byte buffer for fields in ResourceLogs message
pub struct LogFieldOffsets {
    scalar_fields: [Cell<Option<NonZeroUsize>>; 13],
    first_attribute: Cell<Option<NonZeroUsize>>,
}

impl FieldOffsets for LogFieldOffsets {
    fn new() -> Self {
        Self {
            scalar_fields: std::array::from_fn(|_| Cell::new(None)),
            first_attribute: Cell::new(None),
        }
    }

    fn set_field_offset(&self, field_num: u64, wire_type: u64, offset: usize) {
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

        let nz = match NonZeroUsize::new(offset) {
            Some(nz) => nz,
            None => return,
        };

        if field_num == LOG_RECORD_ATTRIBUTES {
            if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                self.first_attribute.set(Some(nz));
            }
        } else if field_num < 13 {
            let idx = field_num as usize;
            if wire_type == WIRE_TYPES[idx] {
                self.scalar_fields[idx].set(Some(nz));
            }
        }
    }

    fn get_field_offset(&self, field_num: u64) -> Option<usize> {
        if field_num == LOG_RECORD_ATTRIBUTES {
            self.first_attribute.get().map(NonZeroUsize::get)
        } else {
            self.scalar_fields
                .get(field_num as usize)
                .and_then(|c| c.get().map(NonZeroUsize::get))
        }
    }
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of ResourceLogs - produces implementation of `ResourceLogs` view from byte array
/// containing a serialized LogsData message
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

/// Iterator of ScopeLogs - produces implementation of `ResourceLogs` view from byte array
/// containing a serialized LogsData message
pub struct ScopeLogsIter<'a> {
    // field_index: usize,
    byte_parser: RepeatedFieldProtoBytesParser<'a, ResourceLogsFieldOffsets>,
}

impl<'a> Iterator for ScopeLogsIter<'a> {
    type Item = RawScopeLogs<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // let slice = self.byte_parser.advance_to_find_next_repeated(
        //     RESOURCE_LOGS_SCOPE_LOGS,
        //     self.field_index,
        //     wire_types::LEN,
        // )?;
        // self.field_index += 1;
        let slice = self.byte_parser.next()?;

        Some(RawScopeLogs {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of LogsRecord - produces implementation of `ResourceLogs` view from byte array
/// containing a serialized LogsData message
pub struct LogRecordsIter<'a> {
    // field_index: usize,
    byte_parser: RepeatedFieldProtoBytesParser<'a, ScopeLogsFieldOffsets>,
}

impl<'a> Iterator for LogRecordsIter<'a> {
    type Item = RawLogRecord<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        // self.field_index += 1;

        Some(RawLogRecord {
            bytes_parser: ProtoBytesParser::new(slice),
        })
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl LogsDataView for RawLogsData<'_> {
    type ResourceLogs<'a>
        = RawResourceLogs<'a>
    where
        Self: 'a;

    type ResourcesIter<'a>
        = ResourceLogsIter<'a>
    where
        Self: 'a;

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

    fn resource(&self) -> Option<Self::Resource<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_LOGS_RESOURCE, wire_types::LEN)?;

        Some(RawResource::new(ProtoBytesParser::new(slice)))
    }

    fn schema_url(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_LOGS_SCHEMA_URL, wire_types::LEN)?;
        std::str::from_utf8(slice).ok().map(Cow::Borrowed)
    }

    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeLogsIter {
            // field_index: 0,
            // byte_parser: self.byte_parser.clone(),
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser, 
                RESOURCE_LOGS_SCOPE_LOGS, 
                wire_types::LEN
            )
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

    fn log_records(&self) -> Self::LogRecordsIter<'_> {
        LogRecordsIter {
            // field_index: 0,
            // byte_parser: self.byte_parser.clone(),
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SCOPE_LOGS_LOG_RECORDS,
                wire_types::LEN
            )
        }
    }

    fn schema_url(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(SCOPE_LOGS_SCHEMA_URL, wire_types::LEN)?;
        std::str::from_utf8(slice).ok().map(Cow::Borrowed)
    }

    fn scope(&self) -> Option<Self::Scope<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(SCOPE_LOG_SCOPE, wire_types::LEN)?;
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

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(
            RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.bytes_parser, 
                LOG_RECORD_ATTRIBUTES,
                wire_types::LEN
            )
            // self.bytes_parser.clone(), LOG_RECORD_ATTRIBUTES
        )
    }

    fn body(&self) -> Option<Self::Body<'_>> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_BODY, wire_types::LEN)
            .map(RawAnyValue::new)
    }

    fn dropped_attributes_count(&self) -> u32 {
        match self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT)
        {
            Some(slice) => match read_varint(slice, 0) {
                Some((val, _)) => val as u32,
                None => 0,
            },
            None => 0,
        }
    }

    fn flags(&self) -> Option<u32> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_FLAGS, wire_types::FIXED32)?;
        let byte_arr: [u8; 4] = slice.try_into().ok()?;
        Some(u32::from_le_bytes(byte_arr))
    }

    fn observed_time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_OBSERVED_TIME_UNIX_NANO, wire_types::FIXED64)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    fn severity_number(&self) -> Option<i32> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT)?;
        let (val, _) = read_varint(slice, 0)?;
        Some(val as i32)
    }

    fn severity_text(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_SEVERITY_TEXT, wire_types::LEN)?;
        std::str::from_utf8(slice).ok().map(Cow::Borrowed)
    }

    fn span_id(&self) -> Option<&[u8]> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_SPAN_ID, wire_types::LEN)
    }

    fn time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    fn trace_id(&self) -> Option<&[u8]> {
        self.bytes_parser
            .advance_to_find_field(LOG_RECORD_TRACE_ID, wire_types::LEN)
    }
}
