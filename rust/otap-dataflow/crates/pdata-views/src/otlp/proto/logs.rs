// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for proto message structs
//! from otlp logs.proto.

use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs,
};

use crate::otlp::proto::common::{KeyValueIter, ObjAny, ObjInstrumentationScope, ObjKeyValue};
use crate::otlp::proto::resource::ObjResource;
use crate::otlp::proto::wrappers::{GenericIterator, GenericObj, Wraps};
use crate::views::common::Str;
use crate::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// Lightweight wrapper around `ResourceLogs` that implements `ResourceLogsView`
pub type ObjResourceLogs<'a> = GenericObj<'a, ResourceLogs>;

/// Lightweight wrapper around `ScopeLogs` that implements `ScopeLogsView`
pub type ObjScope<'a> = GenericObj<'a, ScopeLogs>;

/// Lightweight wrapper around `LogRecord` that implements `LogRecordView`
pub type ObjLogRecord<'a> = GenericObj<'a, LogRecord>;

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of `ObjResourceLogs`. Used in the implementation of `LogsDataView` to get an iterator
/// of the resources contained in the log data.
pub type ResourceIter<'a> = GenericIterator<'a, ResourceLogs, ObjResourceLogs<'a>>;

/// Iterator of `ObjScope`. Used in the implementation of `ResourceLogsView` ot get an iterator
/// of the instrumentation scopes for some resource.
pub type ScopeIter<'a> = GenericIterator<'a, ScopeLogs, ObjScope<'a>>;

/// Iterator of `ObjLogRecord`. Used in the implementation of `ScopeLogsView` to get an iterator
/// of the logs for some scope.
pub type LogRecordIter<'a> = GenericIterator<'a, LogRecord, ObjLogRecord<'a>>;

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl LogsDataView for LogsData {
    type ResourceLogs<'a>
        = ObjResourceLogs<'a>
    where
        Self: 'a;

    type ResourcesIter<'a>
        = ResourceIter<'a>
    where
        Self: 'a;

    fn resources(&self) -> Self::ResourcesIter<'_> {
        ResourceIter::new(self.resource_logs.iter())
    }
}

impl ResourceLogsView for ObjResourceLogs<'_> {
    type Resource<'a>
        = ObjResource<'a>
    where
        Self: 'a;
    type ScopeLogs<'b>
        = ObjScope<'b>
    where
        Self: 'b;
    type ScopesIter<'b>
        = ScopeIter<'b>
    where
        Self: 'b;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        self.inner.resource.as_ref().map(ObjResource::new)
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeIter::new(self.inner.scope_logs.iter())
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        let schema_url = self.inner.schema_url.as_str();
        if schema_url.is_empty() {
            None
        } else {
            Some(schema_url)
        }
    }
}

impl ScopeLogsView for ObjScope<'_> {
    type Scope<'a>
        = ObjInstrumentationScope<'a>
    where
        Self: 'a;

    type LogRecord<'b>
        = ObjLogRecord<'b>
    where
        Self: 'b;

    type LogRecordsIter<'b>
        = LogRecordIter<'b>
    where
        Self: 'b;

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        self.inner.scope.as_ref().map(ObjInstrumentationScope::new)
    }

    #[inline]
    fn log_records(&self) -> Self::LogRecordsIter<'_> {
        LogRecordIter::new(self.inner.log_records.iter())
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        let schema_url = self.inner.schema_url.as_str();
        if schema_url.is_empty() {
            None
        } else {
            Some(schema_url)
        }
    }
}

impl LogRecordView for ObjLogRecord<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type Body<'bod>
        = ObjAny<'bod>
    where
        Self: 'bod;

    #[inline]
    fn time_unix_nano(&self) -> Option<u64> {
        if self.inner.time_unix_nano != 0 {
            Some(self.inner.time_unix_nano)
        } else {
            None
        }
    }

    #[inline]
    fn observed_time_unix_nano(&self) -> Option<u64> {
        if self.inner.observed_time_unix_nano != 0 {
            Some(self.inner.observed_time_unix_nano)
        } else {
            None
        }
    }

    #[inline]
    fn severity_number(&self) -> Option<i32> {
        if self.inner.severity_number != 0 {
            Some(self.inner.severity_number)
        } else {
            None
        }
    }

    #[inline]
    fn severity_text(&self) -> Option<Str<'_>> {
        if !self.inner.severity_text.is_empty() {
            Some(self.inner.severity_text.as_str())
        } else {
            None
        }
    }

    #[inline]
    fn body(&self) -> Option<Self::Body<'_>> {
        self.inner.body.as_ref().map(ObjAny::new)
    }

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        if self.inner.flags != 0 {
            Some(self.inner.flags)
        } else {
            None
        }
    }

    #[inline]
    fn trace_id(&self) -> Option<&[u8]> {
        if is_valid_trace_id(&self.inner.trace_id) {
            Some(self.inner.trace_id.as_slice())
        } else {
            None
        }
    }

    #[inline]
    fn span_id(&self) -> Option<&[u8]> {
        if is_valid_span_id(&self.inner.span_id) {
            Some(self.inner.span_id.as_slice())
        } else {
            None
        }
    }
}

fn is_valid_trace_id(buf: &[u8]) -> bool {
    buf.len() == 16 && buf != [0; 16]
}

fn is_valid_span_id(buf: &[u8]) -> bool {
    buf.len() == 8 && buf != [0; 8]
}
