// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP JSON serialization for logs pdata views.
//!
//! The encoder walks the resource, scope, and log-record hierarchy through `LogsDataView` and
//! streams protobuf-compatible JSON through serde. Trace context and other OTLP-specific fields
//! are encoded in place, so owned protobuf, raw protobuf, and OTAP Arrow views share one path.

use super::common::{
    AnyValueJson, AttributeIterJson, HexId, ProtoU64, ResourceJson, ScopeJson, Utf8,
};
use super::{JsonEncodeError, write_json};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::io::Write;

/// Writes one logs pdata view as a compact OTLP JSON document.
///
/// This function does not add a delimiter, bound output size, or roll back bytes already
/// accepted by the writer when serialization fails. Callers own those policies.
pub fn write_logs_json<L: LogsDataView, W: Write>(
    logs: &L,
    output: &mut W,
) -> Result<(), JsonEncodeError> {
    write_json(&LogsJson(logs), output)
}

struct LogsJson<'a, L: LogsDataView>(&'a L);

impl<L: LogsDataView> Serialize for LogsJson<'_, L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.resources().next().is_some() {
            map.serialize_entry("resourceLogs", &ResourceLogsList(self.0))?;
        }
        map.end()
    }
}

struct ResourceLogsList<'a, L: LogsDataView>(&'a L);

impl<L: LogsDataView> Serialize for ResourceLogsList<'_, L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for resource in self.0.resources() {
            sequence.serialize_element(&ResourceLogsJson(resource))?;
        }
        sequence.end()
    }
}

struct ResourceLogsJson<R: ResourceLogsView>(R);

impl<R: ResourceLogsView> Serialize for ResourceLogsJson<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(resource) = self.0.resource() {
            map.serialize_entry("resource", &ResourceJson(&resource))?;
        }
        if self.0.scopes().next().is_some() {
            map.serialize_entry("scopeLogs", &ScopeLogsList(&self.0))?;
        }
        if let Some(schema_url) = self.0.schema_url().filter(|value| !value.is_empty()) {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct ScopeLogsList<'a, R: ResourceLogsView>(&'a R);

impl<R: ResourceLogsView> Serialize for ScopeLogsList<'_, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for scope in self.0.scopes() {
            sequence.serialize_element(&ScopeLogsJson(scope))?;
        }
        sequence.end()
    }
}

struct ScopeLogsJson<L: ScopeLogsView>(L);

impl<L: ScopeLogsView> Serialize for ScopeLogsJson<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(scope) = self.0.scope() {
            map.serialize_entry("scope", &ScopeJson(&scope))?;
        }
        if self.0.log_records().next().is_some() {
            map.serialize_entry("logRecords", &LogRecordList(&self.0))?;
        }
        if let Some(schema_url) = self.0.schema_url().filter(|value| !value.is_empty()) {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct LogRecordList<'a, L: ScopeLogsView>(&'a L);

impl<L: ScopeLogsView> Serialize for LogRecordList<'_, L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for record in self.0.log_records() {
            sequence.serialize_element(&LogRecordJson(record))?;
        }
        sequence.end()
    }
}

struct LogRecordJson<L: LogRecordView>(L);

impl<L: LogRecordView> Serialize for LogRecordJson<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = self.0.time_unix_nano().filter(|value| *value != 0) {
            map.serialize_entry("timeUnixNano", &ProtoU64(value))?;
        }
        if let Some(value) = self.0.observed_time_unix_nano().filter(|value| *value != 0) {
            map.serialize_entry("observedTimeUnixNano", &ProtoU64(value))?;
        }
        if let Some(value) = self.0.severity_number().filter(|value| *value != 0) {
            map.serialize_entry("severityNumber", &value)?;
        }
        if let Some(value) = self.0.severity_text().filter(|value| !value.is_empty()) {
            map.serialize_entry("severityText", &Utf8(value))?;
        }
        if let Some(value) = self.0.body() {
            map.serialize_entry("body", &AnyValueJson::new(value))?;
        }
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        let dropped = self.0.dropped_attributes_count();
        if dropped != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped)?;
        }
        if let Some(flags) = self.0.flags().filter(|value| *value != 0) {
            map.serialize_entry("flags", &flags)?;
        }
        if let Some(trace_id) = self.0.trace_id() {
            map.serialize_entry("traceId", &HexId(trace_id))?;
        }
        if let Some(span_id) = self.0.span_id() {
            map.serialize_entry("spanId", &HexId(span_id))?;
        }
        if let Some(value) = self.0.event_name().filter(|value| !value.is_empty()) {
            map.serialize_entry("eventName", &Utf8(value))?;
        }
        map.end()
    }
}
