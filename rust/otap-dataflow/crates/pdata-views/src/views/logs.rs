// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! **Backend-agnostic, zero-copy view traits for OTLP Logs.**
//! ```text
//! LogsView
//! └─ ResourceLogsView
//!    │  resource::ResourceView
//!    └─ ScopeLogsView
//!       │  InstrumentationScopeView
//!       └─ LogRecordView
//!          └─ common::AttributeView
//!             └─ common::AnyValueView
//! ```

use crate::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, Str},
    resource::ResourceView,
};

/// View for top level LogsData
pub trait LogsDataView {
    /// The `ResourceLogsView` trait associated with this impl of LogsDataView trait
    type ResourceLogs<'res>: ResourceLogsView
    where
        Self: 'res;

    /// The associated iterator type for this implementation of the trait. The iterator will yield
    /// borrowed references that must live as long as the input lifetime 'res
    type ResourcesIter<'res>: Iterator<Item = Self::ResourceLogs<'res>>
    where
        Self: 'res;

    /// Iterator yielding borrowed references for resources associated with this `LogsData`
    fn resources(&self) -> Self::ResourcesIter<'_>;
}

/// View for ResourceLogs
pub trait ResourceLogsView {
    /// The `ResourceView` trait associated with this impl of the `ResourceLogsView` trait.
    type Resource<'res>: ResourceView
    where
        Self: 'res;

    /// The `ScopeLogsView` trait associated with this impl of the `ResourceLogsView`. This allows
    type ScopeLogs<'scp>: ScopeLogsView
    where
        Self: 'scp;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'scp
    type ScopesIter<'scp>: Iterator<Item = Self::ScopeLogs<'scp>>
    where
        Self: 'scp;

    /// Access the resource for the logs contained in the `LogsData`. IF this returns `None` it
    /// means the resource info is unknown.
    fn resource(&self) -> Option<Self::Resource<'_>>;

    /// Iterator yielding the `ScopeLogs` that originate from this Resource.
    fn scopes(&self) -> Self::ScopesIter<'_>;

    /// The schema URL for the resource. If the schema is not known, this returns `None`
    fn schema_url(&self) -> Option<Str<'_>>;
}

/// View for ScopeLogs
pub trait ScopeLogsView {
    /// The `InstrumentationScopeView` trait associated with this impl of the `ScopeLogsView` trait.
    type Scope<'scp>: InstrumentationScopeView
    where
        Self: 'scp;

    /// The `LogRecordView` trait associated with this impl of the `ScopeLogsView` trait.
    type LogRecord<'rec>: LogRecordView
    where
        Self: 'rec;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'scp
    type LogRecordsIter<'rec>: Iterator<Item = Self::LogRecord<'rec>>
    where
        Self: 'rec;

    /// Access the instrumentation scope for the logs contained in this scope. If this returns
    /// `None` it means the scope is unknown
    fn scope(&self) -> Option<Self::Scope<'_>>;

    /// Iterator yielding `LogRecord`s
    fn log_records(&self) -> Self::LogRecordsIter<'_>;

    /// The schema URL. This schema URL applies to all logs returned by the `log_records` iterator.
    /// This method returns `None` if the schema URL is not known.
    fn schema_url(&self) -> Option<Str<'_>>;
}

/// View for LogRecords
pub trait LogRecordView {
    /// The `AttributeView` type associated with this impl of the `LogRecordView` trait for
    /// accessing this LogRecord's attributes
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated attribute iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// The associated `AnyValueView` type associated with this impl of `LogRecordView` trait for
    /// accessing this LogRecord's Body
    type Body<'bod>: AnyValueView<'bod>
    where
        Self: 'bod;

    /// access  the time when the event occurred.
    /// returns `None` if the timestamp is unknown or missing
    fn time_unix_nano(&self) -> Option<u64>;

    /// access the time when the event was observed by the collection system.
    /// returns `None` if the timestamp is unknown or missing
    fn observed_time_unix_nano(&self) -> Option<u64>;

    /// Access the numerical value of severity, normalized ot the Log Data Model
    /// https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber
    fn severity_number(&self) -> Option<i32>;

    /// Access the severity text (also known as log level)
    fn severity_text(&self) -> Option<Str<'_>>;

    /// Access the value containing the body of the log record.
    fn body(&self) -> Option<Self::Body<'_>>;

    /// Access the log record's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access this log record's dropped attributes.The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;

    /// Access the log record's flags
    fn flags(&self) -> Option<u32>;

    /// Access the trace ID. Should return None if the underlying trace ID is missing, or if the
    /// backend representation of the trace ID is invalid. Invalid trace IDs include all-0s or a
    /// trace ID of incorrect length (length != 16)
    fn trace_id(&self) -> Option<&[u8]>;

    /// Access the span ID. Should return None if the underlying span ID is missing or if the
    /// backend representation of the span ID is invalid. Invalid span IDs include all-0s or a
    /// span ID or incorrect length (length != 8)
    fn span_id(&self) -> Option<&[u8]>;

    // TODO event_name https://github.com/open-telemetry/otel-arrow/issues/422
}
