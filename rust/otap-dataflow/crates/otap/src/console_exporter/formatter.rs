// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Hierarchical formatter for OTLP data with tree-style output.
//!
//! This module uses the shared `format_log_line` from `otap_df_telemetry::self_tracing`
//! to render OTLP log data in a hierarchical tree format. The tree structure is
//! inserted via the level-formatting callback, keeping timestamp left-aligned:
//!
//! ```text
//! 2026-01-14T18:29:09.645Z  RESOURCE  v1.Resource: [service.name=my-service]
//! 2026-01-14T18:29:09.645Z  │ SCOPE   v1.InstrumentationScope: [name=my-lib]
//! 2026-01-14T18:29:09.645Z  │ └─ DEBUG  event_name: body [attr=value]
//! ```

use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::schema::{SpanId, TraceId};
use otap_df_pdata::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType,
};
use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::resource::ResourceView;
use otap_df_telemetry::self_tracing::{AnsiCode, BufWriter, ConsoleWriter, LOG_BUFFER_SIZE};
use std::io::Write;

/// Tree drawing characters for Unicode mode.
mod unicode_tree {
    pub const VERTICAL: &str = "│";
    pub const TEE: &str = "├─";
    pub const CORNER: &str = "└─";
}

/// Tree drawing characters for ASCII mode.
mod ascii_tree {
    pub const VERTICAL: &str = "|";
    pub const TEE: &str = "+-";
    pub const CORNER: &str = "\\-";
}

/// Tree drawing characters.
#[derive(Clone, Copy)]
struct TreeChars {
    vertical: &'static str,
    tee: &'static str,
    corner: &'static str,
}

impl TreeChars {
    fn unicode() -> Self {
        Self {
            vertical: unicode_tree::VERTICAL,
            tee: unicode_tree::TEE,
            corner: unicode_tree::CORNER,
        }
    }

    fn ascii() -> Self {
        Self {
            vertical: ascii_tree::VERTICAL,
            tee: ascii_tree::TEE,
            corner: ascii_tree::CORNER,
        }
    }
}

/// Hierarchical formatter for OTLP data.
pub struct HierarchicalFormatter {
    writer: ConsoleWriter,
    tree: TreeChars,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub fn new(use_color: bool, use_unicode: bool) -> Self {
        let writer = if use_color {
            ConsoleWriter::color()
        } else {
            ConsoleWriter::no_color()
        };
        let tree = if use_unicode {
            TreeChars::unicode()
        } else {
            TreeChars::ascii()
        };
        Self { writer, tree }
    }

    /// Format logs from OTLP bytes.
    pub fn format_logs_bytes(&self, bytes: &OtlpProtoBytes) {
        if let OtlpProtoBytes::ExportLogsRequest(data) = bytes {
            let logs_data = RawLogsData::new(data.as_ref());
            self.format_logs_data(&logs_data);
        }
    }

    /// Format logs from a LogsDataView.
    fn format_logs_data<L: LogsDataView>(&self, logs_data: &L) {
        for resource_logs in logs_data.resources() {
            self.format_resource_logs(&resource_logs);
        }
    }

    /// Format a ResourceLogs with its nested scopes.
    fn format_resource_logs<R: ResourceLogsView>(&self, resource_logs: &R) {
        // Get first timestamp from nested log records
        let first_ts = self.get_first_log_timestamp(resource_logs);

        // Always format resource line (even if empty) for consistent tree structure
        match resource_logs.resource() {
            Some(resource) => {
                let view = ResourceLogView::new(&resource);
                self.print_line(first_ts, "v1.Resource", &view, |w, cw| {
                    cw.write_ansi(w, AnsiCode::Cyan);
                    cw.write_ansi(w, AnsiCode::Bold);
                    let _ = w.write_all(b"RESOURCE");
                    cw.write_ansi(w, AnsiCode::Reset);
                    let _ = w.write_all(b"  ");
                });
            }
            None => {
                self.print_resource_line(first_ts);
            }
        }

        // Format each scope
        let scopes: Vec<_> = resource_logs.scopes().collect();
        let scope_count = scopes.len();
        for (i, scope_logs) in scopes.into_iter().enumerate() {
            let is_last_scope = i == scope_count - 1;
            self.format_scope_logs(&scope_logs, is_last_scope);
        }
    }

    /// Print a resource line with no attributes (used when resource is None).
    fn print_resource_line(&self, timestamp_ns: u64) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        self.writer.write_ansi(&mut w, AnsiCode::Dim);
        ConsoleWriter::write_timestamp(&mut w, timestamp_ns);
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b"  ");
        self.writer.write_ansi(&mut w, AnsiCode::Cyan);
        self.writer.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"RESOURCE");
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b"    v1.Resource:\n");

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
    }

    /// Get the first timestamp from log records in a ResourceLogs.
    fn get_first_log_timestamp<R: ResourceLogsView>(&self, resource_logs: &R) -> u64 {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                if let Some(ts) = log_record.time_unix_nano() {
                    return ts;
                }
                if let Some(ts) = log_record.observed_time_unix_nano() {
                    return ts;
                }
            }
        }
        0
    }

    /// Format a ScopeLogs with its nested log records.
    fn format_scope_logs<S: ScopeLogsView>(&self, scope_logs: &S, is_last_scope: bool) {
        // Get first timestamp from log records
        let first_ts = scope_logs
            .log_records()
            .find_map(|lr| lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano()))
            .unwrap_or(0);

        // Always format scope line (even if empty) for consistent tree structure
        match scope_logs.scope() {
            Some(scope) => {
                let view = ScopeLogView::new(&scope);
                let tree = self.tree;
                self.print_line(first_ts, "v1.InstrumentationScope", &view, |w, cw| {
                    let _ = w.write_all(tree.vertical.as_bytes());
                    let _ = w.write_all(b" ");
                    cw.write_ansi(w, AnsiCode::Magenta);
                    cw.write_ansi(w, AnsiCode::Bold);
                    let _ = w.write_all(b"SCOPE");
                    cw.write_ansi(w, AnsiCode::Reset);
                    let _ = w.write_all(b"    ");
                });
            }
            None => {
                self.print_scope_line(first_ts);
            }
        }

        // Format each log record
        let records: Vec<_> = scope_logs.log_records().collect();
        let record_count = records.len();
        for (i, log_record) in records.into_iter().enumerate() {
            let is_last_record = i == record_count - 1;
            self.format_log_record(&log_record, is_last_scope, is_last_record);
        }
    }

    /// Print a scope line with no attributes (used when scope is None).
    fn print_scope_line(&self, timestamp_ns: u64) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        self.writer.write_ansi(&mut w, AnsiCode::Dim);
        ConsoleWriter::write_timestamp(&mut w, timestamp_ns);
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b"  ");
        let _ = w.write_all(self.tree.vertical.as_bytes());
        let _ = w.write_all(b" ");
        self.writer.write_ansi(&mut w, AnsiCode::Magenta);
        self.writer.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"SCOPE");
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b"    v1.InstrumentationScope:\n");

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
    }

    /// Format a single log record using format_log_line.
    fn format_log_record<L: LogRecordView>(
        &self,
        log_record: &L,
        is_last_scope: bool,
        is_last_record: bool,
    ) {
        let ts = log_record
            .time_unix_nano()
            .or_else(|| log_record.observed_time_unix_nano())
            .unwrap_or(0);

        let event_name = log_record
            .event_name()
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .unwrap_or_else(|| "event".to_string());

        let severity = log_record.severity_number();
        let tree = self.tree;

        self.print_line(ts, &event_name, log_record, |w, cw| {
            // Tree prefix
            let _ = w.write_all(tree.vertical.as_bytes());
            let _ = w.write_all(b" ");
            if is_last_record && is_last_scope {
                let _ = w.write_all(tree.corner.as_bytes());
            } else {
                let _ = w.write_all(tree.tee.as_bytes());
            }
            let _ = w.write_all(b" ");
            // Severity with color
            cw.write_severity(w, severity);
        });
    }

    /// Print a line using the shared format_log_line.
    fn print_line<V, F>(&self, timestamp_ns: u64, event_name: &str, record: &V, format_level: F)
    where
        V: LogRecordView,
        F: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
    {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        self.writer
            .format_log_line(&mut w, timestamp_ns, event_name, record, format_level);

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
    }
}

// ============================================================================
// View adapters that present Resource and Scope as LogRecordView
// ============================================================================

/// A view adapter that presents a Resource as a LogRecordView.
///
/// The resource attributes become the log record's attributes.
/// Body is empty, severity is ignored.
struct ResourceLogView<'a, R: ResourceView> {
    resource: &'a R,
}

impl<'a, R: ResourceView> ResourceLogView<'a, R> {
    fn new(resource: &'a R) -> Self {
        Self { resource }
    }
}

impl<R: ResourceView> LogRecordView for ResourceLogView<'_, R> {
    type Attribute<'att>
        = R::Attribute<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = R::AttributesIter<'att>
    where
        Self: 'att;
    type Body<'bod>
        = EmptyAnyValue
    where
        Self: 'bod;

    fn time_unix_nano(&self) -> Option<u64> {
        None
    }
    fn observed_time_unix_nano(&self) -> Option<u64> {
        None
    }
    fn severity_number(&self) -> Option<i32> {
        None
    }
    fn severity_text(&self) -> Option<Str<'_>> {
        None
    }
    fn body(&self) -> Option<Self::Body<'_>> {
        None
    }
    fn attributes(&self) -> Self::AttributeIter<'_> {
        self.resource.attributes()
    }
    fn dropped_attributes_count(&self) -> u32 {
        0
    }
    fn flags(&self) -> Option<u32> {
        None
    }
    fn trace_id(&self) -> Option<&TraceId> {
        None
    }
    fn span_id(&self) -> Option<&SpanId> {
        None
    }
    fn event_name(&self) -> Option<Str<'_>> {
        None
    }
}

/// A view adapter that presents an InstrumentationScope as a LogRecordView.
///
/// The scope's name, version, and attributes are merged into the attributes iterator.
struct ScopeLogView<'a, S: InstrumentationScopeView> {
    scope: &'a S,
}

impl<'a, S: InstrumentationScopeView> ScopeLogView<'a, S> {
    fn new(scope: &'a S) -> Self {
        Self { scope }
    }
}

impl<S: InstrumentationScopeView> LogRecordView for ScopeLogView<'_, S> {
    type Attribute<'att>
        = ScopeAttribute<'att, S>
    where
        Self: 'att;
    type AttributeIter<'att>
        = ScopeAttributeIter<'att, S>
    where
        Self: 'att;
    type Body<'bod>
        = EmptyAnyValue
    where
        Self: 'bod;

    fn time_unix_nano(&self) -> Option<u64> {
        None
    }
    fn observed_time_unix_nano(&self) -> Option<u64> {
        None
    }
    fn severity_number(&self) -> Option<i32> {
        None
    }
    fn severity_text(&self) -> Option<Str<'_>> {
        None
    }
    fn body(&self) -> Option<Self::Body<'_>> {
        None
    }
    fn attributes(&self) -> Self::AttributeIter<'_> {
        ScopeAttributeIter::new(self.scope)
    }
    fn dropped_attributes_count(&self) -> u32 {
        0
    }
    fn flags(&self) -> Option<u32> {
        None
    }
    fn trace_id(&self) -> Option<&TraceId> {
        None
    }
    fn span_id(&self) -> Option<&SpanId> {
        None
    }
    fn event_name(&self) -> Option<Str<'_>> {
        None
    }
}

/// Iterator that yields scope name, version, and attributes as a unified attribute stream.
struct ScopeAttributeIter<'a, S: InstrumentationScopeView> {
    scope: &'a S,
    phase: ScopeIterPhase<'a, S>,
}

enum ScopeIterPhase<'a, S: InstrumentationScopeView + 'a> {
    Name,
    Version,
    Attributes(S::AttributeIter<'a>),
    Done,
}

impl<'a, S: InstrumentationScopeView + 'a> ScopeAttributeIter<'a, S> {
    fn new(scope: &'a S) -> Self {
        Self {
            scope,
            phase: ScopeIterPhase::Name,
        }
    }
}

impl<'a, S: InstrumentationScopeView + 'a> Iterator for ScopeAttributeIter<'a, S> {
    type Item = ScopeAttribute<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.phase {
                ScopeIterPhase::Name => {
                    if let Some(name) = self.scope.name() {
                        self.phase = ScopeIterPhase::Version;
                        return Some(ScopeAttribute::Name(name));
                    }
                    self.phase = ScopeIterPhase::Version;
                }
                ScopeIterPhase::Version => {
                    if let Some(version) = self.scope.version() {
                        self.phase = ScopeIterPhase::Attributes(self.scope.attributes());
                        return Some(ScopeAttribute::Version(version));
                    }
                    self.phase = ScopeIterPhase::Attributes(self.scope.attributes());
                }
                ScopeIterPhase::Attributes(iter) => {
                    if let Some(attr) = iter.next() {
                        return Some(ScopeAttribute::Attr(attr));
                    }
                    self.phase = ScopeIterPhase::Done;
                    return None;
                }
                ScopeIterPhase::Done => return None,
            }
        }
    }
}

/// A synthetic attribute for scope name/version or a real scope attribute.
enum ScopeAttribute<'a, S: InstrumentationScopeView + 'a> {
    Name(Str<'a>),
    Version(Str<'a>),
    Attr(S::Attribute<'a>),
}

impl<'a, S: InstrumentationScopeView + 'a> AttributeView for ScopeAttribute<'a, S> {
    type Val<'val>
        = ScopeAttributeValue<'a, 'val, S>
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        match self {
            ScopeAttribute::Name(_) => b"name",
            ScopeAttribute::Version(_) => b"version",
            ScopeAttribute::Attr(a) => a.key(),
        }
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        match self {
            ScopeAttribute::Name(s) => Some(ScopeAttributeValue::String(s)),
            ScopeAttribute::Version(s) => Some(ScopeAttributeValue::String(s)),
            ScopeAttribute::Attr(a) => a.value().map(ScopeAttributeValue::Delegated),
        }
    }
}

/// Value type for scope attributes - either a string (name/version) or delegated.
/// 'a is the lifetime of the underlying data
/// 'val is the borrow lifetime for the value call
enum ScopeAttributeValue<'a, 'val, S: InstrumentationScopeView + 'a>
where
    S::Attribute<'a>: 'val,
{
    String(Str<'a>),
    Delegated(<S::Attribute<'a> as AttributeView>::Val<'val>),
}

impl<'a, 'val, S: InstrumentationScopeView + 'a> AnyValueView<'val>
    for ScopeAttributeValue<'a, 'val, S>
where
    'a: 'val,
{
    type KeyValue = EmptyAttribute;
    type ArrayIter<'arr>
        = std::iter::Empty<Self>
    where
        Self: 'arr;
    type KeyValueIter<'kv>
        = std::iter::Empty<EmptyAttribute>
    where
        Self: 'kv;

    fn value_type(&self) -> ValueType {
        match self {
            ScopeAttributeValue::String(_) => ValueType::String,
            ScopeAttributeValue::Delegated(v) => v.value_type(),
        }
    }

    fn as_string(&self) -> Option<Str<'_>> {
        match self {
            ScopeAttributeValue::String(s) => Some(s),
            ScopeAttributeValue::Delegated(v) => v.as_string(),
        }
    }

    fn as_int64(&self) -> Option<i64> {
        match self {
            ScopeAttributeValue::String(_) => None,
            ScopeAttributeValue::Delegated(v) => v.as_int64(),
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ScopeAttributeValue::String(_) => None,
            ScopeAttributeValue::Delegated(v) => v.as_bool(),
        }
    }

    fn as_double(&self) -> Option<f64> {
        match self {
            ScopeAttributeValue::String(_) => None,
            ScopeAttributeValue::Delegated(v) => v.as_double(),
        }
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            ScopeAttributeValue::String(_) => None,
            ScopeAttributeValue::Delegated(v) => v.as_bytes(),
        }
    }

    fn as_array(&self) -> Option<Self::ArrayIter<'_>> {
        None
    }

    fn as_kvlist(&self) -> Option<Self::KeyValueIter<'_>> {
        None
    }
}

/// An empty AnyValue (used as placeholder for body).
struct EmptyAnyValue;

impl<'a> AnyValueView<'a> for EmptyAnyValue {
    type KeyValue = EmptyAttribute;
    type ArrayIter<'arr>
        = std::iter::Empty<EmptyAnyValue>
    where
        Self: 'arr;
    type KeyValueIter<'kv>
        = std::iter::Empty<EmptyAttribute>
    where
        Self: 'kv;

    fn value_type(&self) -> ValueType {
        ValueType::Empty
    }
    fn as_string(&self) -> Option<Str<'_>> {
        None
    }
    fn as_int64(&self) -> Option<i64> {
        None
    }
    fn as_bool(&self) -> Option<bool> {
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

/// An empty attribute (used as placeholder).
struct EmptyAttribute;

impl AttributeView for EmptyAttribute {
    type Val<'val>
        = EmptyAnyValue
    where
        Self: 'val;

    fn key(&self) -> Str<'_> {
        b""
    }

    fn value(&self) -> Option<Self::Val<'_>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_chars() {
        let unicode = TreeChars::unicode();
        let ascii = TreeChars::ascii();

        assert_eq!(unicode.vertical, "│");
        assert_eq!(unicode.tee, "├─");

        assert_eq!(ascii.vertical, "|");
        assert_eq!(ascii.tee, "+-");
    }
}
