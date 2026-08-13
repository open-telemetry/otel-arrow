// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct transformation of serialized OTLP logs into ClickHouse Arrow columns.
//!
//! # Purpose
//!
//! The legacy raw-OTLP path first converts an `ExportLogsServiceRequest` into the canonical OTAP
//! Arrow representation and then runs the generic ClickHouse transformation plan. This module
//! avoids both intermediate stages. It reads the serialized protobuf through borrowed
//! [`RawLogsData`] views and builds the final ClickHouse-compatible [`RecordBatch`] directly.
//! Protobuf messages are not decoded into an owned Rust object tree and no intermediate OTAP
//! batches are created.
//!
//! Unlike the specialized OTAP transformer in `logs_fast`, this path cannot share input arrays
//! with its output because its input is a byte buffer rather than Arrow data. Its main savings
//! come from traversing the nested protobuf hierarchy once, materializing only the final arrays,
//! and reusing allocation and schema state between requests.
//!
//! # Applicability and routing
//!
//! The ClickHouse exporter attempts this transformer only for raw
//! `OtlpProtoBytes::ExportLogsRequest` payloads. It is not used for OTAP records, OTLP traces,
//! OTLP metrics, or already transformed Arrow batches. The routing and fallback policy lives in
//! the parent `clickhouse_exporter` module:
//!
//! - a successful non-empty transformation is written as the logs payload
//! - a valid request with no log records returns `None`, so no empty insert is issued
//! - invalid top-level protobuf wire framing is rejected and is not sent through the legacy path
//! - malformed or wrongly typed nested fields follow the raw views' lazy best-effort behavior
//! - other conversion or Arrow errors are returned to the caller, which may attempt the legacy
//!   OTLP-to-OTAP transformation
//!
//! This module does not perform fallback itself. Keeping that decision at the exporter boundary
//! makes direct-path and fallback telemetry account for the complete request.
//!
//! # Processing model
//!
//! `OtlpLogsTransformer::transform` follows the OTLP ownership hierarchy. Resource and scope data
//! is collected once and replayed for each child log, while log-local fields are appended directly
//! to the Arrow builders:
//!
//! ```text
//! ExportLogsServiceRequest bytes
//!                 |
//!                 v
//!       RawLogsData::try_new
//!       - validate top-level framing
//!       - expose borrowed views
//!                 |
//!                 v
//!       ResourceLogs (0..n)
//!       - resource schema URL
//!       - resource attributes + service.name
//!                 |
//!                 v
//!         ScopeLogs (0..n)
//!         - scope name/version/attributes
//!                 |
//!                 v
//!         LogRecord (0..n) --------> append one output row
//!                                     - scalar fields
//!                                     - body and attributes
//!                                     - trace/span IDs
//!                                               |
//!                                               v
//!                                   sort columns by name
//!                                   build/reuse Arrow schema
//!                                               |
//!                                               v
//!                                   ClickHouse RecordBatch
//! ```
//!
//! Traversal order is resource, then scope, then log order. That order is the output row order and
//! must remain stable. Resource and scope attributes are copied into every child log row because
//! the ClickHouse logs table stores flattened rows rather than the nested OTLP hierarchy.
//!
//! # Column mapping
//!
//! One output row represents one OTLP `LogRecord`. The transformer currently builds these
//! columns, then sorts them lexically to preserve the generic transformer's stable output order:
//!
//! ```text
//! +--------------------+--------------------------------------+-----------------------+
//! | Output column      | Raw OTLP source / operation          | Arrow representation  |
//! +--------------------+--------------------------------------+-----------------------+
//! | Timestamp          | LogRecord.time_unix_nano             | TimestampNanosecond   |
//! | ResourceSchemaUrl  | ResourceLogs.schema_url              | Utf8                  |
//! | ResourceAttributes | Resource.attributes -> string map    | Map(Utf8, Utf8)       |
//! | ServiceName        | first resource attr `service.name`   | Utf8                  |
//! | ScopeName          | InstrumentationScope.name            | Utf8                  |
//! | ScopeVersion       | InstrumentationScope.version         | Utf8                  |
//! | ScopeAttributes    | InstrumentationScope.attributes      | Map(Utf8, Utf8)       |
//! | LogAttributes      | LogRecord.attributes                 | Map(Utf8, Utf8)       |
//! | Body               | LogRecord.body -> string             | Utf8                  |
//! | EventName          | LogRecord.event_name                 | Utf8                  |
//! | SeverityNumber     | LogRecord.severity_number -> UInt8   | UInt8                 |
//! | SeverityText       | LogRecord.severity_text              | Utf8                  |
//! | TraceId            | LogRecord.trace_id -> lowercase hex  | Utf8                  |
//! | SpanId             | LogRecord.span_id -> lowercase hex   | Utf8                  |
//! +--------------------+--------------------------------------+-----------------------+
//! ```
//!
//! Fields not listed in this table are not emitted by this transformer. In particular, the
//! current implementation does not emit `ScopeSchemaUrl` or `TraceFlags`; extending the mapping
//! requires updating the builders, presence tracking, parity tests, and live ClickHouse binding
//! test together.
//!
//! # Value conversion
//!
//! ClickHouse attribute columns are maps of strings, so OTLP `AnyValue` values use the same
//! logical conversion as the legacy transformer:
//!
//! - strings remain strings; booleans, integers, and doubles use their textual form
//! - a top-level byte value is standard base64 text
//! - arrays and key-value lists are JSON; byte values nested in JSON use the serializer's byte
//!   array representation
//! - a missing attribute value becomes an empty string
//! - an empty or missing body becomes null
//! - trace and span IDs become lowercase hexadecimal strings
//!
//! String-bearing OTLP fields must contain valid UTF-8, and a severity number must fit in `UInt8`.
//! Violations are conversion errors rather than silent truncation or replacement.
//!
//! # Optional columns and defaults
//!
//! [`ColumnPresence`] records whether an optional field appears anywhere in the request. A column
//! is omitted from the batch when it is absent from every row, matching the generic transformer's
//! use of ClickHouse defaults. When a column is present for only some rows, its builders retain
//! nulls or empty maps at the other row positions. `Timestamp` is always emitted and a missing
//! OTLP timestamp becomes zero. `ResourceAttributes` and `ServiceName` are emitted together when
//! at least one resource attribute exists; an absent `service.name` becomes an empty string.
//!
//! # Reused state and allocation bounds
//!
//! [`OtlpLogsTransformer`] is owned by one exporter task and is reused sequentially. It retains:
//!
//! - the previous request's row count as the next request's builder-capacity hint, capped at 16K
//!   rows to prevent one unusually large request from causing persistent over-allocation
//! - a scratch byte buffer used for `AnyValue` text/JSON conversion and ID encoding
//! - a one-entry schema cache keyed by the sorted output column names and Arrow data types
//!
//! The Arrow builders themselves are request-local and are finalized into the returned batch.
//! Changes to conversion, column presence, or sorting must preserve logical parity with the
//! legacy path; the tests below compare names, row order, null placement, and values.

use std::sync::Arc;

use arrow::array::{
    ArrayRef, MapBuilder, RecordBatch, StringBuilder, TimestampNanosecondBuilder, UInt8Builder,
};
use arrow::datatypes::{DataType, Field, Schema};
use base64::Engine;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde::Serialize;
use serde::ser::{Error as SerError, SerializeMap, SerializeSeq, Serializer};

use crate::exporters::clickhouse_exporter::consts as ch_consts;
use crate::exporters::clickhouse_exporter::error::ClickhouseExporterError;

/// Raw resource-attribute key whose first value populates the dedicated `ServiceName` column.
const SERVICE_NAME_KEY: &[u8] = b"service.name";

/// Initial payload-byte estimate used for each row in an Arrow string builder.
///
/// Builders grow automatically, so this affects allocation behavior but never truncates values.
const INITIAL_STRING_BYTES_PER_ROW: usize = 16;

/// Maximum number of rows for which a new request speculatively reserves builder capacity.
///
/// The output itself is not capped. Requests larger than this value continue growing their
/// builders normally; only the capacity hint learned for the next request is bounded.
const MAX_PREALLOCATED_ROWS: usize = 16 * 1024;

/// Clamp a previous request's row count before using it as the next capacity hint.
fn bounded_row_capacity(row_count: usize) -> usize {
    row_count.min(MAX_PREALLOCATED_ROWS)
}

/// Stateful converter from serialized OTLP log requests to ClickHouse Arrow batches.
///
/// The exporter keeps one instance per task and calls it sequentially. Only allocation hints,
/// serialization scratch space, and the most recently used schema survive between calls; no
/// payload data is retained after [`Self::transform`] returns.
#[derive(Default)]
pub(crate) struct OtlpLogsTransformer {
    /// Last schema and the ordered `(name, data_type)` key that uniquely describes it.
    cached_schema: Option<(Vec<(&'static str, DataType)>, Arc<Schema>)>,
    /// Reusable buffer for scalar text, JSON, base64, and hexadecimal representations.
    json_scratch: Vec<u8>,
    /// Completed row count from the previous request, used only as a capacity estimate.
    previous_row_count: usize,
}

impl OtlpLogsTransformer {
    /// Transform serialized `ExportLogsServiceRequest` bytes directly to a ClickHouse batch.
    ///
    /// The method validates the protobuf framing, walks resources/scopes/logs in wire order, and
    /// appends one Arrow row for each log record. Parent resource and scope values are flattened
    /// into every child row. Output columns are selected from request-wide presence information
    /// and sorted by name before schema construction.
    ///
    /// Returns `Ok(None)` when the request is valid but contains no log records. The caller treats
    /// that as a successful no-op rather than issuing an empty ClickHouse insert.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid top-level protobuf framing, invalid UTF-8, unsupported value
    /// coercions, or Arrow builder/schema failures. The exporter, not this method, decides whether
    /// an error is rejected or offered to the legacy transformation path.
    pub(crate) fn transform(
        &mut self,
        request: &[u8],
    ) -> Result<Option<RecordBatch>, ClickhouseExporterError> {
        let logs = RawLogsData::try_new(request)?;
        let mut builders = LogsBuilders::new(self.previous_row_count);
        let mut presence = ColumnPresence::default();
        let mut row_count = 0;

        for resource_logs in logs.resources() {
            let resource_schema_url = optional_utf8(resource_logs.schema_url())?;
            let (resource_attributes, service_name) = match resource_logs.resource() {
                Some(resource) => collect_resource_attributes(resource, &mut self.json_scratch)?,
                None => (Vec::new(), String::new()),
            };
            presence.resource_schema_url |= resource_schema_url.is_some();
            presence.resource_attributes |= !resource_attributes.is_empty();

            for scope_logs in resource_logs.scopes() {
                let (scope_name, scope_version, scope_attributes) = match scope_logs.scope() {
                    Some(scope) => {
                        let name = optional_utf8(scope.name())?;
                        let version = optional_utf8(scope.version())?;
                        let attributes =
                            collect_attributes(scope.attributes(), &mut self.json_scratch)?;
                        (name, version, attributes)
                    }
                    None => (None, None, Vec::new()),
                };
                presence.scope_name |= scope_name.is_some();
                presence.scope_version |= scope_version.is_some();
                presence.scope_attributes |= !scope_attributes.is_empty();

                for log in scope_logs.log_records() {
                    row_count += 1;
                    builders
                        .timestamp
                        .append_value(log.time_unix_nano().unwrap_or(0) as i64);
                    append_optional_string(
                        &mut builders.resource_schema_url,
                        resource_schema_url.as_deref(),
                    );
                    append_attribute_map(&mut builders.resource_attributes, &resource_attributes)?;
                    builders.service_name.append_value(&service_name);
                    append_optional_string(&mut builders.scope_name, scope_name.as_deref());
                    append_optional_string(&mut builders.scope_version, scope_version.as_deref());
                    append_attribute_map(&mut builders.scope_attributes, &scope_attributes)?;

                    let mut has_log_attributes = false;
                    for attribute in log.attributes() {
                        append_attribute(
                            &mut builders.log_attributes,
                            attribute,
                            &mut self.json_scratch,
                        )?;
                        has_log_attributes = true;
                    }
                    builders.log_attributes.append(true)?;
                    presence.log_attributes |= has_log_attributes;

                    match log.body() {
                        Some(body) if body.value_type() != ValueType::Empty => {
                            stringify_any_value(body, &mut self.json_scratch)?;
                            builders.body.append_value(utf8(&self.json_scratch)?);
                            presence.body = true;
                        }
                        _ => builders.body.append_null(),
                    }

                    append_optional_bytes_as_string(
                        &mut builders.event_name,
                        log.event_name(),
                        &mut presence.event_name,
                    )?;
                    append_optional_bytes_as_string(
                        &mut builders.severity_text,
                        log.severity_text(),
                        &mut presence.severity_text,
                    )?;
                    match log.severity_number() {
                        Some(value) => {
                            let value = u8::try_from(value).map_err(|_| {
                                ClickhouseExporterError::CoercionError {
                                    error: format!(
                                        "OTLP log severity number {value} cannot be represented as UInt8"
                                    ),
                                }
                            })?;
                            builders.severity_number.append_value(value);
                            presence.severity_number = true;
                        }
                        None => builders.severity_number.append_null(),
                    }

                    append_optional_hex(
                        &mut builders.span_id,
                        log.span_id().map(|id| id.as_slice()),
                        &mut presence.span_id,
                        &mut self.json_scratch,
                    );
                    append_optional_hex(
                        &mut builders.trace_id,
                        log.trace_id().map(|id| id.as_slice()),
                        &mut presence.trace_id,
                        &mut self.json_scratch,
                    );
                }
            }
        }

        self.previous_row_count = row_count;
        if row_count == 0 {
            return Ok(None);
        }

        let mut columns = builders.finish(presence);
        columns.sort_unstable_by_key(|(name, _)| *name);
        let schema = self.schema_for(&columns);
        let arrays = columns.into_iter().map(|(_, array)| array).collect();
        Ok(Some(RecordBatch::try_new(schema, arrays)?))
    }

    /// Return a cached schema when column names and types match, or build and cache a new one.
    ///
    /// Column order is part of the cache key. Callers must sort columns before invoking this
    /// method so equivalent request shapes produce the same schema identity.
    fn schema_for(&mut self, columns: &[(&'static str, ArrayRef)]) -> Arc<Schema> {
        let key = columns
            .iter()
            .map(|(name, array)| (*name, array.data_type().clone()))
            .collect::<Vec<_>>();
        if let Some((cached_key, schema)) = &self.cached_schema
            && cached_key == &key
        {
            return schema.clone();
        }

        let fields = key
            .iter()
            .map(|(name, data_type)| Field::new(*name, data_type.clone(), true))
            .collect::<Vec<_>>();
        let schema = Arc::new(Schema::new(fields));
        self.cached_schema = Some((key, schema.clone()));
        schema
    }
}

/// Request-wide presence flags controlling which optional arrays enter the output batch.
///
/// Builders append a value or null for every row even before a field is known to occur. At
/// finalization, these flags discard arrays that were absent from all rows, preserving the legacy
/// transformer's conditional-column behavior and allowing ClickHouse defaults to apply.
#[derive(Default)]
struct ColumnPresence {
    body: bool,
    event_name: bool,
    log_attributes: bool,
    resource_attributes: bool,
    resource_schema_url: bool,
    scope_attributes: bool,
    scope_name: bool,
    scope_version: bool,
    severity_number: bool,
    severity_text: bool,
    span_id: bool,
    trace_id: bool,
}

/// Request-local Arrow builders kept in row lockstep while the protobuf hierarchy is flattened.
///
/// Every builder receives exactly one logical entry per output row. [`Self::finish`] may omit an
/// optional array, but any array it returns must have the same length as `timestamp`.
struct LogsBuilders {
    body: StringBuilder,
    event_name: StringBuilder,
    log_attributes: MapBuilder<StringBuilder, StringBuilder>,
    resource_attributes: MapBuilder<StringBuilder, StringBuilder>,
    resource_schema_url: StringBuilder,
    scope_attributes: MapBuilder<StringBuilder, StringBuilder>,
    scope_name: StringBuilder,
    scope_version: StringBuilder,
    service_name: StringBuilder,
    severity_number: UInt8Builder,
    severity_text: StringBuilder,
    span_id: StringBuilder,
    timestamp: TimestampNanosecondBuilder,
    trace_id: StringBuilder,
}

impl LogsBuilders {
    /// Create all builders using a bounded estimate of the request's eventual row count.
    ///
    /// String payload capacity uses a small per-row estimate. Map child builders intentionally
    /// start without a value-count estimate because attribute cardinality is independent of log
    /// row count.
    fn new(row_capacity: usize) -> Self {
        let row_capacity = bounded_row_capacity(row_capacity);
        let string_capacity = row_capacity.saturating_mul(INITIAL_STRING_BYTES_PER_ROW);
        let string_builder = || StringBuilder::with_capacity(row_capacity, string_capacity);
        let map_builder = || {
            MapBuilder::with_capacity(
                None,
                StringBuilder::new(),
                StringBuilder::new(),
                row_capacity,
            )
        };
        Self {
            body: string_builder(),
            event_name: string_builder(),
            log_attributes: map_builder(),
            resource_attributes: map_builder(),
            resource_schema_url: string_builder(),
            scope_attributes: map_builder(),
            scope_name: string_builder(),
            scope_version: string_builder(),
            service_name: string_builder(),
            severity_number: UInt8Builder::with_capacity(row_capacity),
            severity_text: string_builder(),
            span_id: string_builder(),
            timestamp: TimestampNanosecondBuilder::with_capacity(row_capacity),
            trace_id: string_builder(),
        }
    }

    /// Finalize required arrays and only those optional arrays observed in the request.
    ///
    /// `Timestamp` is unconditional. `ResourceAttributes` and `ServiceName` are a pair because
    /// the service name is derived while scanning the resource map. Sorting is deliberately left
    /// to the caller so this method can focus on presence policy and builder finalization.
    fn finish(mut self, presence: ColumnPresence) -> Vec<(&'static str, ArrayRef)> {
        let mut columns = Vec::with_capacity(14);
        if presence.body {
            columns.push((ch_consts::CH_BODY, Arc::new(self.body.finish()) as ArrayRef));
        }
        if presence.event_name {
            columns.push((
                ch_consts::CH_EVENT_NAME,
                Arc::new(self.event_name.finish()) as ArrayRef,
            ));
        }
        if presence.log_attributes {
            columns.push((
                ch_consts::CH_LOG_ATTRIBUTES,
                Arc::new(self.log_attributes.finish()) as ArrayRef,
            ));
        }
        if presence.resource_attributes {
            columns.push((
                ch_consts::CH_RESOURCE_ATTRIBUTES,
                Arc::new(self.resource_attributes.finish()) as ArrayRef,
            ));
            columns.push((
                ch_consts::CH_SERVICE_NAME,
                Arc::new(self.service_name.finish()) as ArrayRef,
            ));
        }
        if presence.resource_schema_url {
            columns.push((
                ch_consts::CH_RESOURCE_SCHEMA_URL,
                Arc::new(self.resource_schema_url.finish()) as ArrayRef,
            ));
        }
        if presence.scope_attributes {
            columns.push((
                ch_consts::CH_SCOPE_ATTRIBUTES,
                Arc::new(self.scope_attributes.finish()) as ArrayRef,
            ));
        }
        if presence.scope_name {
            columns.push((
                ch_consts::CH_SCOPE_NAME,
                Arc::new(self.scope_name.finish()) as ArrayRef,
            ));
        }
        if presence.scope_version {
            columns.push((
                ch_consts::CH_SCOPE_VERSION,
                Arc::new(self.scope_version.finish()) as ArrayRef,
            ));
        }
        if presence.severity_number {
            columns.push((
                ch_consts::CH_SEVERITY_NUMBER,
                Arc::new(self.severity_number.finish()) as ArrayRef,
            ));
        }
        if presence.severity_text {
            columns.push((
                ch_consts::CH_SEVERITY_TEXT,
                Arc::new(self.severity_text.finish()) as ArrayRef,
            ));
        }
        if presence.span_id {
            columns.push((
                ch_consts::CH_SPAN_ID,
                Arc::new(self.span_id.finish()) as ArrayRef,
            ));
        }
        columns.push((
            ch_consts::CH_TIMESTAMP,
            Arc::new(self.timestamp.finish()) as ArrayRef,
        ));
        if presence.trace_id {
            columns.push((
                ch_consts::CH_TRACE_ID,
                Arc::new(self.trace_id.finish()) as ArrayRef,
            ));
        }
        columns
    }
}

/// Materialize one resource's string map and extract its first `service.name` value.
///
/// Resource attributes must be replayed for every descendant log, so they are owned here rather
/// than repeatedly traversed through borrowed protobuf views. Attribute order and duplicate keys
/// are preserved. The first `service.name` wins to match the generic ClickHouse transformation.
fn collect_resource_attributes<R: ResourceView>(
    resource: R,
    scratch: &mut Vec<u8>,
) -> Result<(Vec<(String, String)>, String), ClickhouseExporterError> {
    let mut attributes = Vec::new();
    let mut service_name = None;
    for attribute in resource.attributes() {
        let key_bytes = attribute.key();
        let key = utf8(key_bytes)?.to_owned();
        stringify_optional_any_value(attribute.value(), scratch)?;
        let value = utf8(scratch)?.to_owned();
        if service_name.is_none() && key_bytes == SERVICE_NAME_KEY {
            service_name = Some(value.clone());
        }
        attributes.push((key, value));
    }
    Ok((attributes, service_name.unwrap_or_default()))
}

/// Materialize a reusable string-map representation of parent-level attributes.
///
/// This is used for scope attributes, which are converted once per scope and copied into every
/// descendant log row. Attribute order and duplicate keys are preserved.
fn collect_attributes<I, A>(
    attributes: I,
    scratch: &mut Vec<u8>,
) -> Result<Vec<(String, String)>, ClickhouseExporterError>
where
    I: Iterator<Item = A>,
    A: AttributeView,
{
    let mut collected = Vec::new();
    for attribute in attributes {
        let key = utf8(attribute.key())?.to_owned();
        stringify_optional_any_value(attribute.value(), scratch)?;
        collected.push((key, utf8(scratch)?.to_owned()));
    }
    Ok(collected)
}

/// Append one borrowed attribute to the current map row without an intermediate owned pair.
///
/// Callers are responsible for closing the map row with `MapBuilder::append` after all of that
/// log record's attributes have been added.
fn append_attribute<A: AttributeView>(
    builder: &mut MapBuilder<StringBuilder, StringBuilder>,
    attribute: A,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    builder.keys().append_value(utf8(attribute.key())?);
    stringify_optional_any_value(attribute.value(), scratch)?;
    builder.values().append_value(utf8(scratch)?);
    Ok(())
}

/// Replay a materialized parent attribute map and close exactly one Arrow map row.
///
/// Calling this with an empty slice appends a valid empty map, not a null map. That distinction
/// keeps all builders aligned while request-wide presence tracking decides whether the complete
/// column is emitted.
fn append_attribute_map(
    builder: &mut MapBuilder<StringBuilder, StringBuilder>,
    attributes: &[(String, String)],
) -> Result<(), ClickhouseExporterError> {
    for (key, value) in attributes {
        builder.keys().append_value(key);
        builder.values().append_value(value);
    }
    builder.append(true)?;
    Ok(())
}

/// Replace `scratch` with the string-map representation of an optional OTLP value.
///
/// A missing value intentionally produces an empty string because ClickHouse attribute maps use
/// string values and this matches the legacy transformer's coercion.
fn stringify_optional_any_value<'a, V: AnyValueView<'a> + 'a>(
    value: Option<V>,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    scratch.clear();
    if let Some(value) = value {
        stringify_any_value(value, scratch)?;
    }
    Ok(())
}

/// Replace `scratch` with the ClickHouse string representation of an OTLP `AnyValue`.
///
/// Scalar values avoid general-purpose JSON serialization. Arrays and key-value lists delegate
/// to serde so nesting is preserved. A top-level byte value is base64, whereas a byte value
/// nested inside JSON follows serde's byte-sequence representation.
fn stringify_any_value<'a, V: AnyValueView<'a> + 'a>(
    value: V,
    scratch: &mut Vec<u8>,
) -> Result<(), ClickhouseExporterError> {
    scratch.clear();
    match value.value_type() {
        ValueType::Empty => {}
        ValueType::String => scratch.extend_from_slice(
            value
                .as_string()
                .ok_or_else(|| invalid_any_value("string"))?,
        ),
        ValueType::Bool => scratch.extend_from_slice(
            if value.as_bool().ok_or_else(|| invalid_any_value("bool"))? {
                b"true"
            } else {
                b"false"
            },
        ),
        ValueType::Int64 => {
            let mut buffer = itoa::Buffer::new();
            scratch.extend_from_slice(
                buffer
                    .format(value.as_int64().ok_or_else(|| invalid_any_value("int64"))?)
                    .as_bytes(),
            );
        }
        ValueType::Double => {
            let mut buffer = ryu::Buffer::new();
            scratch.extend_from_slice(
                buffer
                    .format(
                        value
                            .as_double()
                            .ok_or_else(|| invalid_any_value("double"))?,
                    )
                    .as_bytes(),
            );
        }
        ValueType::Bytes => {
            let encoded = base64::engine::general_purpose::STANDARD
                .encode(value.as_bytes().ok_or_else(|| invalid_any_value("bytes"))?);
            scratch.extend_from_slice(encoded.as_bytes());
        }
        ValueType::Array | ValueType::KeyValueList => {
            serde_json::to_writer(&mut *scratch, &AnyValueSerializerWrapper(value)).map_err(
                |error| ClickhouseExporterError::CoercionError {
                    error: format!("failed to serialize OTLP AnyValue as JSON: {error}"),
                },
            )?;
        }
    }
    Ok(())
}

/// Describe an inconsistent view whose declared `ValueType` has no matching payload accessor.
fn invalid_any_value(expected: &str) -> ClickhouseExporterError {
    ClickhouseExporterError::CoercionError {
        error: format!("OTLP AnyValue declared {expected} but did not contain a valid value"),
    }
}

/// Adapter that lets a borrowed [`AnyValueView`] participate in recursive serde serialization.
///
/// Keeping the view borrowed avoids building a second owned tree solely for nested JSON values.
struct AnyValueSerializerWrapper<T>(T);

impl<'a, T> Serialize for AnyValueSerializerWrapper<T>
where
    T: AnyValueView<'a> + 'a,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_any_value(&self.0, serializer)
    }
}

/// Recursively serialize an OTLP value with JSON-compatible scalar, sequence, and map semantics.
///
/// This function is serializer-generic so recursion can use normal serde APIs; production calls
/// it through `serde_json::to_writer` in [`stringify_any_value`]. Missing nested attribute values
/// become JSON nulls, unlike missing top-level string-map values, which become empty strings.
fn serialize_any_value<'a, T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AnyValueView<'a> + 'a,
    S: Serializer,
{
    match value.value_type() {
        ValueType::Empty => serializer.serialize_none(),
        ValueType::String => serializer.serialize_str(
            std::str::from_utf8(
                value
                    .as_string()
                    .ok_or_else(|| S::Error::custom("missing OTLP string value"))?,
            )
            .map_err(S::Error::custom)?,
        ),
        ValueType::Bool => serializer.serialize_bool(
            value
                .as_bool()
                .ok_or_else(|| S::Error::custom("missing OTLP bool value"))?,
        ),
        ValueType::Int64 => serializer.serialize_i64(
            value
                .as_int64()
                .ok_or_else(|| S::Error::custom("missing OTLP int64 value"))?,
        ),
        ValueType::Double => serializer.serialize_f64(
            value
                .as_double()
                .ok_or_else(|| S::Error::custom("missing OTLP double value"))?,
        ),
        ValueType::Bytes => serializer.serialize_bytes(
            value
                .as_bytes()
                .ok_or_else(|| S::Error::custom("missing OTLP bytes value"))?,
        ),
        ValueType::Array => {
            let mut sequence = serializer.serialize_seq(None)?;
            for child in value
                .as_array()
                .ok_or_else(|| S::Error::custom("missing OTLP array value"))?
            {
                sequence.serialize_element(&AnyValueSerializerWrapper(child))?;
            }
            sequence.end()
        }
        ValueType::KeyValueList => {
            let mut map = serializer.serialize_map(None)?;
            for attribute in value
                .as_kvlist()
                .ok_or_else(|| S::Error::custom("missing OTLP map value"))?
            {
                let key = std::str::from_utf8(attribute.key()).map_err(S::Error::custom)?;
                match attribute.value() {
                    Some(child) => {
                        map.serialize_entry(key, &AnyValueSerializerWrapper(child))?;
                    }
                    None => map.serialize_entry(key, &Option::<()>::None)?,
                }
            }
            map.end()
        }
    }
}

/// Validate and own an optional protobuf string that must outlive its borrowed parent view.
fn optional_utf8(value: Option<&[u8]>) -> Result<Option<String>, ClickhouseExporterError> {
    value
        .map(|value| utf8(value).map(ToOwned::to_owned))
        .transpose()
}

/// Interpret protobuf string bytes as UTF-8 and normalize failures to an exporter coercion error.
fn utf8(value: &[u8]) -> Result<&str, ClickhouseExporterError> {
    std::str::from_utf8(value).map_err(|error| ClickhouseExporterError::CoercionError {
        error: format!("invalid UTF-8 in OTLP string: {error}"),
    })
}

/// Append an optional validated string while preserving a null for an absent row value.
fn append_optional_string(builder: &mut StringBuilder, value: Option<&str>) {
    match value {
        Some(value) => builder.append_value(value),
        None => builder.append_null(),
    }
}

/// Validate and append an optional protobuf string, recording request-wide column presence.
///
/// The presence flag changes only for `Some`, even when the present string is empty. This lets an
/// explicitly empty OTLP value remain distinguishable from a field omitted in every row.
fn append_optional_bytes_as_string(
    builder: &mut StringBuilder,
    value: Option<&[u8]>,
    present: &mut bool,
) -> Result<(), ClickhouseExporterError> {
    match value {
        Some(value) => {
            builder.append_value(utf8(value)?);
            *present = true;
        }
        None => builder.append_null(),
    }
    Ok(())
}

/// Append optional binary identifier bytes as lowercase hexadecimal text.
///
/// Hex output is written into the shared scratch buffer without allocating a temporary `String`.
/// As with other optional helpers, `present` records field presence across the entire request.
fn append_optional_hex(
    builder: &mut StringBuilder,
    value: Option<&[u8]>,
    present: &mut bool,
    scratch: &mut Vec<u8>,
) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    match value {
        Some(value) => {
            scratch.clear();
            scratch.reserve(value.len() * 2);
            for byte in value {
                scratch.push(HEX[(byte >> 4) as usize]);
                scratch.push(HEX[(byte & 0x0f) as usize]);
            }
            builder.append_value(
                std::str::from_utf8(scratch).expect("lowercase hexadecimal is valid UTF-8"),
            );
            *present = true;
        }
        None => builder.append_null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use arrow::array::Array;
    use arrow::util::display::array_value_to_string;
    use bytes::Bytes;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::{fixtures, round_trip::encode_logs};
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtlpProtoBytes, TryIntoWithOptions};
    use prost::Message;

    use crate::exporters::clickhouse_exporter::transform::logs_fast::{
        LogsFastTransform, LogsFastTransformer,
    };
    use crate::exporters::clickhouse_exporter::transform::transform_batch::BatchTransformer;
    use crate::exporters::clickhouse_exporter::{
        config::Config,
        writer::{ClickHouseWriter, build_client},
    };

    fn request_bytes(logs: LogsData) -> Bytes {
        Bytes::from(
            ExportLogsServiceRequest {
                resource_logs: logs.resource_logs,
            }
            .encode_to_vec(),
        )
    }

    fn legacy_batch(bytes: &Bytes) -> Option<RecordBatch> {
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(bytes.clone()).into();
        let mut records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("convert raw OTLP logs through OTAP Arrow");
        records
            .decode_transport_optimized_ids()
            .expect("decode transport optimized IDs");
        BatchTransformer::new()
            .apply_plan(records)
            .expect("apply legacy ClickHouse transform")
            .remove(&ArrowPayloadType::Logs)
    }

    fn otap_fast_batch(logs: &LogsData) -> RecordBatch {
        let mut records = encode_logs(logs);
        records
            .decode_transport_optimized_ids()
            .expect("decode transport optimized IDs");
        match LogsFastTransformer::default()
            .try_apply(&records)
            .expect("transform canonical OTAP logs through the fast path")
        {
            LogsFastTransform::Applied(batch) => batch,
            LogsFastTransform::NotApplicable(reason) => {
                panic!("nested canonical logs unexpectedly declined: {reason}")
            }
        }
    }

    fn logical_values(array: &dyn Array) -> Vec<Option<String>> {
        if matches!(array.data_type(), DataType::Binary)
            || matches!(
                array.data_type(),
                DataType::Dictionary(_, value) if **value == DataType::Binary
            )
        {
            let binary = arrow::compute::cast(array, &DataType::Binary)
                .expect("cast dictionary-encoded binary values")
                .as_any()
                .downcast_ref::<arrow::array::BinaryArray>()
                .expect("binary array")
                .clone();
            return (0..binary.len())
                .map(|row| {
                    (!binary.is_null(row)).then(|| {
                        std::str::from_utf8(binary.value(row))
                            .expect("ClickHouse string bytes are UTF-8")
                            .to_owned()
                    })
                })
                .collect();
        }

        (0..array.len())
            .map(|row| {
                (!array.is_null(row)).then(|| {
                    array_value_to_string(array, row).expect("format Arrow value for comparison")
                })
            })
            .collect()
    }

    fn assert_matches_legacy(logs: LogsData) {
        let bytes = request_bytes(logs);
        let expected = legacy_batch(&bytes);
        let actual = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform raw OTLP logs directly");

        match (actual, expected) {
            (Some(actual), Some(expected)) => {
                let actual_names = actual
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().clone())
                    .collect::<Vec<_>>();
                let expected_names = expected
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().clone())
                    .collect::<Vec<_>>();
                assert_eq!(actual_names, expected_names);
                assert_eq!(actual.num_rows(), expected.num_rows());
                for (actual, expected) in actual.columns().iter().zip(expected.columns()) {
                    assert_eq!(
                        logical_values(actual.as_ref()),
                        logical_values(expected.as_ref())
                    );
                }
            }
            (None, None) => {}
            (actual, expected) => panic!(
                "direct and legacy transforms disagree about output presence: direct={}, legacy={}",
                actual.is_some(),
                expected.is_some()
            ),
        }
    }

    fn logs_with_all_value_types() -> LogsData {
        let nested = AnyValue::new_kvlist(vec![
            KeyValue::new("nested.string", AnyValue::new_string("value")),
            KeyValue::new(
                "nested.array",
                AnyValue::new_array(vec![
                    AnyValue::new_bool(false),
                    AnyValue::new_int(-7),
                    AnyValue::new_bytes([1_u8, 2, 3]),
                ]),
            ),
        ]);
        LogsData::new(vec![ResourceLogs {
            resource: Some(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("service.name", AnyValue::new_string("first")),
                        KeyValue::new("service.name", AnyValue::new_string("second")),
                        KeyValue::new("resource.map", nested.clone()),
                    ])
                    .finish(),
            ),
            scope_logs: vec![ScopeLogs {
                scope: Some(
                    InstrumentationScope::build()
                        .name("raw-scope")
                        .version("1.0")
                        .attributes(vec![KeyValue::new(
                            "scope.array",
                            AnyValue::new_array(vec![AnyValue::new_double(1.5)]),
                        )])
                        .finish(),
                ),
                log_records: vec![
                    LogRecord::build()
                        .time_unix_nano(123_u64)
                        .severity_number(SeverityNumber::Error)
                        .severity_text("ERROR")
                        .body(nested.clone())
                        .attributes(vec![
                            KeyValue::new("empty", AnyValue::default()),
                            KeyValue::new("string", AnyValue::new_string("text")),
                            KeyValue::new("int", AnyValue::new_int(-42)),
                            KeyValue::new("double", AnyValue::new_double(3.25)),
                            KeyValue::new("bool", AnyValue::new_bool(true)),
                            KeyValue::new("nested", nested),
                        ])
                        .trace_id([0x11; 16])
                        .span_id([0x22; 8])
                        .event_name("raw-event")
                        .finish(),
                ],
                schema_url: "https://scope.example/v1".to_string(),
            }],
            schema_url: "https://resource.example/v1".to_string(),
        }])
    }

    /// Scenario: raw OTLP logs contain resource, scope, and log attributes plus scalar fields.
    /// Guarantees: the direct path preserves the legacy transform's columns, row order, and values.
    #[test]
    fn full_logs_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_with_full_resource_and_scope());
    }

    /// Scenario: raw OTLP logs omit every resource, scope, and log attribute.
    /// Guarantees: the direct path omits the same attribute columns as the legacy transform.
    #[test]
    fn logs_without_attributes_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_with_no_attributes());
    }

    /// Scenario: raw OTLP logs span resources and scopes with mixed optional content.
    /// Guarantees: direct traversal preserves legacy row order, nulls, and conditional columns.
    #[test]
    fn mixed_resources_and_scopes_match_legacy_transform() {
        assert_matches_legacy(fixtures::logs_multiple_resources_mixed_content());
    }

    /// Scenario: raw OTLP logs exercise scalar, nested, ID, and schema fields.
    /// Guarantees: direct JSON and scalar conversion exactly preserve legacy logical values.
    #[test]
    fn nested_value_types_match_legacy_transform() {
        assert_matches_legacy(logs_with_all_value_types());
    }

    /// Scenario: identical nested OTLP values take the direct and canonical OTAP fast paths.
    /// Guarantees: every direct-path ClickHouse column has the same rows and logical values.
    #[test]
    fn nested_values_match_otap_fast_path() {
        let logs = logs_with_all_value_types();
        let bytes = request_bytes(logs.clone());
        let direct = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform raw OTLP logs directly")
            .expect("direct logs batch");
        let fast = otap_fast_batch(&logs);

        assert_eq!(direct.num_rows(), fast.num_rows());
        for field in direct.schema().fields() {
            let name = field.name();
            let direct_column = direct.column_by_name(name).expect("direct column");
            let fast_column = fast
                .column_by_name(name)
                .unwrap_or_else(|| panic!("OTAP fast-path batch is missing {name}"));
            assert_eq!(
                logical_values(direct_column.as_ref()),
                logical_values(fast_column.as_ref()),
                "direct and OTAP fast-path values differ for {name}",
            );
        }
    }

    /// Scenario: a raw OTLP log attribute contains an arbitrary byte sequence.
    /// Guarantees: the direct path stores the attribute using standard base64 encoding.
    #[test]
    fn bytes_attribute_is_base64_encoded() {
        let logs = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![
                    LogRecord::build()
                        .attributes(vec![KeyValue::new(
                            "bytes",
                            AnyValue::new_bytes([0_u8, 1, 2, 255]),
                        )])
                        .finish(),
                ],
            )],
        )]);
        let bytes = request_bytes(logs);
        let batch = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform byte attribute")
            .expect("logs batch");
        let attributes = batch
            .column_by_name(ch_consts::CH_LOG_ATTRIBUTES)
            .expect("log attributes")
            .as_any()
            .downcast_ref::<arrow::array::MapArray>()
            .expect("map column");
        let keys = attributes
            .keys()
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .expect("map keys");
        let values = attributes
            .values()
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .expect("map values");

        assert_eq!(keys.value(0), "bytes");
        assert_eq!(values.value(0), "AAEC/w==");
    }

    /// Scenario: an OTLP export request contains no log records.
    /// Guarantees: the direct path produces no ClickHouse write batch, matching legacy behavior.
    #[test]
    fn empty_logs_produce_no_batch() {
        assert_matches_legacy(fixtures::empty_logs());
        assert_matches_legacy(fixtures::logs_with_empty_log_records());
    }

    /// Scenario: a raw OTLP request has invalid top-level protobuf framing.
    /// Guarantees: the direct path returns an error so exporter routing can reject the request.
    #[test]
    fn malformed_request_is_rejected() {
        let error = OtlpLogsTransformer::default()
            .transform(b"\xff")
            .expect_err("malformed protobuf must be rejected");

        assert!(matches!(error, ClickhouseExporterError::Child(_)));
    }

    /// Scenario: a nested OTLP field declares a length beyond its enclosing message.
    /// Guarantees: lazy traversal ignores the malformed field instead of rejecting the request.
    #[test]
    fn malformed_nested_request_uses_best_effort_view() {
        let batch = OtlpLogsTransformer::default()
            .transform(&[0x0a, 0x03, 0x1a, 0x05, 0x00])
            .expect("top-level framing is valid");

        assert!(batch.is_none());
    }

    /// Scenario: the previous OTLP request reports an arbitrarily large row count.
    /// Guarantees: speculative Arrow builder capacity remains capped at a bounded row count.
    #[test]
    fn reusable_row_preallocation_is_bounded() {
        assert_eq!(bounded_row_capacity(8_192), 8_192);
        assert_eq!(
            bounded_row_capacity(MAX_PREALLOCATED_ROWS),
            MAX_PREALLOCATED_ROWS
        );
        assert_eq!(bounded_row_capacity(usize::MAX), MAX_PREALLOCATED_ROWS);
    }

    #[derive(clickhouse::Row, serde::Deserialize)]
    struct RawLogSummary {
        row_count: u64,
        service_name: String,
        body: String,
    }

    /// Scenario: a direct raw OTLP batch is inserted into a live ClickHouse logs table.
    /// Guarantees: emitted Arrow types bind by name and preserve rows, service extraction, and body JSON.
    #[tokio::test]
    #[ignore = "requires a live ClickHouse; run with --ignored e2e"]
    async fn e2e_raw_otlp_logs_insert_roundtrips_through_clickhouse_schema() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let endpoint =
            std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".into());
        let username = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".into());
        let password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "test".into());
        let patch = serde_json::from_value(serde_json::json!({
            "endpoint": endpoint,
            "database": "otap_e2e_raw_otlp_logs",
            "username": username,
            "password": password,
            "async_insert": false,
        }))
        .expect("valid ClickHouse config");
        let config = Config::from_patch(patch);
        build_client(&config, "default")
            .query("DROP DATABASE IF EXISTS otap_e2e_raw_otlp_logs")
            .execute()
            .await
            .expect("drop pre-existing test database");

        let bytes = request_bytes(logs_with_all_value_types());
        let batch = OtlpLogsTransformer::default()
            .transform(&bytes)
            .expect("transform raw OTLP logs")
            .expect("logs batch");
        let batches = HashMap::from([(ArrowPayloadType::Logs, batch)]);
        let writer = ClickHouseWriter::new(&config)
            .await
            .expect("create ClickHouse schema");
        _ = writer
            .write_batches(&batches)
            .await
            .expect("insert raw OTLP logs batch");

        let summary = build_client(&config, &config.database)
            .query(
                "SELECT count() AS row_count, CAST(any(ServiceName) AS String) AS service_name, \
                 any(Body) AS body FROM otap_e2e_raw_otlp_logs.otel_logs",
            )
            .fetch_one::<RawLogSummary>()
            .await
            .expect("read raw OTLP logs back");
        assert_eq!(summary.row_count, 1);
        assert_eq!(summary.service_name, "first");
        assert_eq!(
            summary.body,
            "{\"nested.string\":\"value\",\"nested.array\":[false,-7,[1,2,3]]}"
        );
    }
}
