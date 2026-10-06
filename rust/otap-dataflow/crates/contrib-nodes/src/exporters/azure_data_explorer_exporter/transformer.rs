// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Transforms OTel logs, metrics, and traces into ADX rows. The column layouts
//! are derived from the Go `azuredataexplorerexporter` schemas in
//! `logsdata_to_adx.go`, `metricsdata_to_adx.go`, and `tracesdata_to_adx.go`.
//!
//! By default, logs extend the Go schema with one extra column, `EventName`:
//! `(Timestamp, ObservedTimestamp, TraceID, SpanID, SeverityText,
//!   SeverityNumber, Body: dynamic, EventName, ResourceAttributes, LogsAttributes)`
//! The top-level event name and `event.name` attribute fallback are independently
//! configurable.
//!
//! Metrics use the Go exporter column layout:
//! `(Timestamp, MetricName, MetricType, MetricUnit, MetricDescription,
//!   MetricValue, MetricAttributes, Host, ResourceAttributes)`
//! Histogram, exponential histogram, and summary data points produce multiple
//! rows for sum, count, buckets, and quantiles.
//!
//! Traces use the Go exporter column layout:
//! `(TraceID, SpanID, ParentID, SpanName, SpanStatus, SpanStatusMessage,
//!   SpanKind, StartTime, EndTime, ResourceAttributes, TraceAttributes,
//!   Events, Links)`

use bytes::Bytes;
use otel_arrow_dfe_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otel_arrow_dfe_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otel_arrow_dfe_pdata_views::views::metrics::{
    BucketsView, DataType, DataView, ExponentialHistogramDataPointView, ExponentialHistogramView,
    GaugeView, HistogramDataPointView, HistogramView, MetricView, MetricsView, NumberDataPointView,
    ResourceMetricsView, ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value,
    ValueAtQuantileView,
};
use otel_arrow_dfe_pdata_views::views::resource::ResourceView;
use otel_arrow_dfe_pdata_views::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

use super::encoding;
use super::error::Error;

/// Resource attribute key used to override the exporter host's own hostname
/// in the `Host` metrics column, matching the Go exporter's `hostkey`.
const HOST_KEY: &[u8] = b"host.name";

// Empty composite values can consume traversal work without consuming the
// byte budget, so bounded conversion also caps recursive depth and visits.
const MAX_ANY_VALUE_NESTING_DEPTH: usize = 64;
const MAX_ANY_VALUE_TRAVERSAL_WORK: usize = 65_536;

struct ScopeEntries {
    json: Vec<u8>,
    has_event_name: bool,
}

struct JsonWriter {
    bytes: Vec<u8>,
    max_row_bytes: usize,
    request_prefix_bytes: usize,
    max_request_bytes: usize,
}

impl JsonWriter {
    fn new(
        capacity: usize,
        max_row_bytes: usize,
        request_prefix_bytes: usize,
        max_request_bytes: usize,
    ) -> Self {
        let row_capacity = max_row_bytes.saturating_sub(1);
        let request_capacity = max_request_bytes.saturating_sub(request_prefix_bytes);
        Self {
            bytes: Vec::with_capacity(capacity.min(row_capacity).min(request_capacity)),
            max_row_bytes,
            request_prefix_bytes,
            max_request_bytes,
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), Error> {
        self.extend_from_slice(&[byte])
    }

    fn ensure_additional(&self, additional: usize) -> Result<(), Error> {
        let next_len = self.bytes.len().saturating_add(additional);
        let row_size = next_len.saturating_add(1);
        let request_size = self.request_prefix_bytes.saturating_add(next_len);
        let row_payload_limit = self.max_row_bytes.saturating_sub(1);
        let request_payload_limit = self
            .max_request_bytes
            .saturating_sub(self.request_prefix_bytes);
        if request_size > self.max_request_bytes && request_payload_limit < row_payload_limit {
            return Err(Error::RequestTooLarge {
                actual: request_size,
                limit: self.max_request_bytes,
            });
        }
        if row_size > self.max_row_bytes {
            return Err(Error::RowTooLarge {
                actual: row_size,
                limit: self.max_row_bytes,
            });
        }

        if request_size > self.max_request_bytes {
            return Err(Error::RequestTooLarge {
                actual: request_size,
                limit: self.max_request_bytes,
            });
        }
        Ok(())
    }

    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.ensure_additional(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn work_limit_error(&self) -> Error {
        Error::RowTooLarge {
            actual: self.max_row_bytes.saturating_add(1),
            limit: self.max_row_bytes,
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct TraversalBudget<'a> {
    work: &'a Cell<usize>,
    bounded: bool,
}

impl<'a> TraversalBudget<'a> {
    fn new(work: &'a Cell<usize>, bounded: bool) -> Self {
        Self { work, bounded }
    }

    fn visit(&mut self, depth: usize, writer: &JsonWriter) -> Result<(), Error> {
        if !self.bounded {
            return Ok(());
        }
        self.check_depth(depth, writer)?;
        let work = self.work.get().saturating_add(1);
        self.work.set(work);
        if work > MAX_ANY_VALUE_TRAVERSAL_WORK {
            return Err(writer.work_limit_error());
        }
        Ok(())
    }

    fn check_depth(&self, depth: usize, writer: &JsonWriter) -> Result<(), Error> {
        if self.bounded && depth > MAX_ANY_VALUE_NESTING_DEPTH {
            return Err(writer.work_limit_error());
        }
        Ok(())
    }
}

struct RowOutput {
    rows: Vec<Bytes>,
    bytes: usize,
    max_rows: usize,
    max_row_bytes: usize,
    max_bytes: usize,
    bounded_traversal: bool,
    traversal_work: Cell<usize>,
}

impl RowOutput {
    fn new(
        capacity: usize,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
        bounded_traversal: bool,
    ) -> Self {
        Self {
            rows: Vec::with_capacity(capacity.min(max_rows)),
            bytes: 0,
            max_rows,
            max_row_bytes,
            max_bytes,
            bounded_traversal,
            traversal_work: Cell::new(0),
        }
    }

    fn ensure_additional_rows(&self, additional: usize) -> Result<(), Error> {
        let next_rows = self.rows.len().saturating_add(additional);
        if next_rows > self.max_rows {
            return Err(Error::TooManyRows {
                actual: next_rows,
                limit: self.max_rows,
            });
        }
        Ok(())
    }

    fn writer(&self, capacity: usize) -> Result<JsonWriter, Error> {
        self.ensure_additional_rows(1)?;
        Ok(self.fragment_writer(capacity))
    }

    fn fragment_writer(&self, capacity: usize) -> JsonWriter {
        JsonWriter::new(
            capacity,
            self.max_row_bytes,
            self.bytes
                .saturating_add(usize::from(!self.rows.is_empty())),
            self.max_bytes,
        )
    }

    fn traversal_budget(&self) -> TraversalBudget<'_> {
        TraversalBudget::new(&self.traversal_work, self.bounded_traversal)
    }

    fn reject_invalid_metric(&self, reason: &'static str) -> Result<(), Error> {
        if self.bounded_traversal {
            Err(Error::InvalidMetricData { reason })
        } else {
            Ok(())
        }
    }

    fn push(&mut self, row: Vec<u8>) -> Result<(), Error> {
        let next_rows = self.rows.len().saturating_add(1);
        if next_rows > self.max_rows {
            return Err(Error::TooManyRows {
                actual: next_rows,
                limit: self.max_rows,
            });
        }
        let next_bytes = self
            .bytes
            .saturating_add(usize::from(!self.rows.is_empty()))
            .saturating_add(row.len());
        if next_bytes > self.max_bytes {
            return Err(Error::RequestTooLarge {
                actual: next_bytes,
                limit: self.max_bytes,
            });
        }
        self.bytes = next_bytes;
        self.rows.push(Bytes::from(row));
        Ok(())
    }

    fn finish(self) -> Vec<Bytes> {
        self.rows
    }
}

struct ValidatedHistogram {
    buckets: Vec<(f64, u64)>,
    total: u64,
}

struct ValidatedExponentialHistogram {
    negative: Vec<(f64, u64)>,
    positive: Vec<(f64, u64)>,
    zero_total: u64,
    zero_threshold: f64,
    total: u64,
}

/// Converts OTLP logs, metrics, and traces into JSON records matching the
/// ADX table schemas.
#[derive(Debug)]
pub struct Transformer {
    /// Default `Host` column value, used when a resource has no `host.name`
    /// attribute. Matches the Go exporter's `os.Hostname()` fallback.
    default_host: String,
    legacy_logs_body_string: bool,
    export_event_name: bool,
    add_event_name_to_log_attributes: bool,
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer {
    /// Create a new transformer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_host: sysinfo::System::host_name().unwrap_or_default(),
            legacy_logs_body_string: false,
            export_event_name: true,
            add_event_name_to_log_attributes: true,
        }
    }

    /// Select legacy string encoding for the log `Body` column.
    #[must_use]
    pub fn with_legacy_logs_body_string(mut self, enabled: bool) -> Self {
        self.legacy_logs_body_string = enabled;
        self
    }

    /// Configure top-level and attribute-level log event-name output.
    #[must_use]
    pub fn with_log_event_name_options(
        mut self,
        export_event_name: bool,
        add_event_name_to_log_attributes: bool,
    ) -> Self {
        self.export_event_name = export_event_name;
        self.add_event_name_to_log_attributes = add_event_name_to_log_attributes;
        self
    }

    /// Convert an OTLP logs view into a vector of JSON-encoded log records
    /// matching the ADX OTELLogs table schema.
    #[must_use]
    pub fn convert_logs<T: LogsDataView>(&self, logs_view: &T) -> Vec<Bytes> {
        self.convert_logs_with_limits(logs_view, usize::MAX, usize::MAX, usize::MAX, false)
            .expect("unbounded log transformation cannot exceed its limits")
    }

    pub(crate) fn convert_logs_bounded<T: LogsDataView>(
        &self,
        logs_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
    ) -> Result<Vec<Bytes>, Error> {
        self.convert_logs_with_limits(logs_view, max_rows, max_row_bytes, max_bytes, true)
    }

    fn convert_logs_with_limits<T: LogsDataView>(
        &self,
        logs_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
        bounded_traversal: bool,
    ) -> Result<Vec<Bytes>, Error> {
        let mut results =
            RowOutput::new(1024, max_rows, max_row_bytes, max_bytes, bounded_traversal);

        for resource_logs in logs_view.resources() {
            let resource = resource_logs.resource();
            // Pre-serialize resource attributes once per resource
            let resource_attrs_json = Self::serialize_attributes_map(
                resource
                    .as_ref()
                    .map(|r| r.attributes())
                    .into_iter()
                    .flatten(),
                &results,
            )?;

            for scope_logs in resource_logs.scopes() {
                let scope = scope_logs.scope();
                let scope_entries = Self::build_scope_entries(&scope, &results)?;

                for log_record in scope_logs.log_records() {
                    let mut buf = results.writer(2048)?;
                    buf.push(b'{')?;

                    buf.extend_from_slice(b"\"Timestamp\":")?;
                    Self::write_rfc3339_nano(log_record.time_unix_nano().unwrap_or(0), &mut buf)?;

                    buf.extend_from_slice(b",\"ObservedTimestamp\":")?;
                    Self::write_rfc3339_nano(
                        log_record.observed_time_unix_nano().unwrap_or(0),
                        &mut buf,
                    )?;

                    buf.extend_from_slice(b",\"TraceID\":")?;
                    match log_record.trace_id() {
                        Some(id) => Self::write_hex_string(id, &mut buf)?,
                        None => buf.extend_from_slice(b"\"\"")?,
                    }

                    buf.extend_from_slice(b",\"SpanID\":")?;
                    match log_record.span_id() {
                        Some(id) => Self::write_hex_string(id, &mut buf)?,
                        None => buf.extend_from_slice(b"\"\"")?,
                    }

                    buf.extend_from_slice(b",\"SeverityText\":")?;
                    Self::write_json_string(log_record.severity_text().unwrap_or(b""), &mut buf)?;

                    buf.extend_from_slice(b",\"SeverityNumber\":")?;
                    Self::write_i32(log_record.severity_number().unwrap_or(0), &mut buf)?;

                    // Body (dynamic - native JSON value; strings stay strings
                    // for backward compat with existing `tostring(Body)` queries)
                    buf.extend_from_slice(b",\"Body\":")?;
                    match log_record.body() {
                        Some(body) if self.legacy_logs_body_string => {
                            if body.value_type() == ValueType::String {
                                Self::write_json_string(
                                    body.as_string().unwrap_or_default(),
                                    &mut buf,
                                )?;
                            } else {
                                let mut encoded = results.fragment_writer(256);
                                let mut traversal = results.traversal_budget();
                                Self::write_any_value(&body, &mut encoded, &mut traversal, 0)?;
                                Self::write_json_string(&encoded.finish(), &mut buf)?;
                            }
                        }
                        Some(body) => {
                            let mut traversal = results.traversal_budget();
                            Self::write_any_value(&body, &mut buf, &mut traversal, 0)?;
                        }
                        None => buf.extend_from_slice(b"null")?,
                    }

                    if self.export_event_name {
                        buf.extend_from_slice(b",\"EventName\":")?;
                        Self::write_json_string(log_record.event_name().unwrap_or(b""), &mut buf)?;
                    }

                    buf.extend_from_slice(b",\"ResourceAttributes\":")?;
                    buf.extend_from_slice(&resource_attrs_json)?;

                    buf.extend_from_slice(b",\"LogsAttributes\":")?;
                    let event_name_bytes = log_record.event_name().unwrap_or(b"");
                    let mut traversal = results.traversal_budget();
                    Self::write_logs_attributes(
                        log_record.attributes(),
                        &scope_entries,
                        event_name_bytes,
                        self.add_event_name_to_log_attributes,
                        &mut buf,
                        &mut traversal,
                    )?;

                    buf.push(b'}')?;
                    results.push(buf.finish())?;
                }
            }
        }

        Ok(results.finish())
    }

    /// Build scope entries as the JSON-encoded interior of an object.
    fn build_scope_entries<S: InstrumentationScopeView>(
        scope: &Option<S>,
        results: &RowOutput,
    ) -> Result<ScopeEntries, Error> {
        let mut writer = results.fragment_writer(256);
        let mut traversal = results.traversal_budget();
        let mut first = true;
        let mut has_event_name = false;
        if let Some(s) = scope {
            if let Some(name) = s.name()
                && !name.is_empty()
            {
                Self::write_object_separator(&mut writer, &mut first)?;
                Self::write_json_string(b"scope.name", &mut writer)?;
                writer.push(b':')?;
                Self::write_json_string(name, &mut writer)?;
            }
            if let Some(version) = s.version()
                && !version.is_empty()
            {
                Self::write_object_separator(&mut writer, &mut first)?;
                Self::write_json_string(b"scope.version", &mut writer)?;
                writer.push(b':')?;
                Self::write_json_string(version, &mut writer)?;
            }
            for attr in s.attributes() {
                if let Some(attr_value) = attr.value() {
                    has_event_name |= attr.key() == b"event.name";
                    Self::write_object_separator(&mut writer, &mut first)?;
                    Self::write_json_string(attr.key(), &mut writer)?;
                    writer.push(b':')?;
                    Self::write_any_value(&attr_value, &mut writer, &mut traversal, 0)?;
                }
            }
        }
        Ok(ScopeEntries {
            json: writer.finish(),
            has_event_name,
        })
    }

    /// Serialize attributes from an iterator into a JSON object bytes.
    fn serialize_attributes_map<I, A>(attrs: I, results: &RowOutput) -> Result<Vec<u8>, Error>
    where
        I: Iterator<Item = A>,
        A: AttributeView,
    {
        let mut writer = results.fragment_writer(256);
        let mut traversal = results.traversal_budget();
        writer.push(b'{')?;
        let mut first = true;
        for attr in attrs {
            if let Some(val) = attr.value() {
                Self::write_object_separator(&mut writer, &mut first)?;
                Self::write_json_string(attr.key(), &mut writer)?;
                writer.push(b':')?;
                Self::write_any_value(&val, &mut writer, &mut traversal, 0)?;
            }
        }
        writer.push(b'}')?;
        Ok(writer.finish())
    }

    /// Serialize log attributes merged with scope entries and event name.
    fn write_logs_attributes<I, A>(
        attrs: I,
        scope_entries: &ScopeEntries,
        event_name: &[u8],
        add_event_name: bool,
        writer: &mut JsonWriter,
        traversal: &mut TraversalBudget<'_>,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = A>,
        A: AttributeView,
    {
        writer.push(b'{')?;
        let mut first = scope_entries.json.is_empty();
        let mut has_event_name = scope_entries.has_event_name;
        if !scope_entries.json.is_empty() {
            writer.extend_from_slice(&scope_entries.json)?;
        }

        for attr in attrs {
            let key = attr.key();
            if let Some(val) = attr.value() {
                has_event_name |= key == b"event.name";
                Self::write_object_separator(writer, &mut first)?;
                Self::write_json_string(key, writer)?;
                writer.push(b':')?;
                Self::write_any_value(&val, writer, traversal, 0)?;
            }
        }

        if add_event_name && !event_name.is_empty() && !has_event_name {
            Self::write_object_separator(writer, &mut first)?;
            Self::write_json_string(b"event.name", writer)?;
            writer.push(b':')?;
            Self::write_json_string(event_name, writer)?;
        }

        writer.push(b'}')
    }

    fn write_object_separator(writer: &mut JsonWriter, first: &mut bool) -> Result<(), Error> {
        if *first {
            *first = false;
            Ok(())
        } else {
            writer.push(b',')
        }
    }

    fn write_any_value<'a, V: AnyValueView<'a>>(
        value: &V,
        writer: &mut JsonWriter,
        traversal: &mut TraversalBudget<'_>,
        depth: usize,
    ) -> Result<(), Error> {
        traversal.visit(depth, writer)?;
        match value.value_type() {
            ValueType::String => match value.as_string() {
                Some(s) => Self::write_json_string(s, writer),
                None => writer.extend_from_slice(b"null"),
            },
            ValueType::Int64 => match value.as_int64() {
                Some(n) => {
                    let mut buf = itoa::Buffer::new();
                    writer.extend_from_slice(buf.format(n).as_bytes())
                }
                None => writer.extend_from_slice(b"null"),
            },
            ValueType::Double => match value.as_double() {
                Some(d) if d.is_finite() => {
                    let mut ryu_buf = ryu::Buffer::new();
                    writer.extend_from_slice(ryu_buf.format_finite(d).as_bytes())
                }
                _ => writer.extend_from_slice(b"null"),
            },
            ValueType::Bool => match value.as_bool() {
                Some(true) => writer.extend_from_slice(b"true"),
                Some(false) => writer.extend_from_slice(b"false"),
                None => writer.extend_from_slice(b"null"),
            },
            ValueType::Bytes => match value.as_bytes() {
                Some(bytes) => Self::write_hex_string(bytes, writer),
                None => writer.extend_from_slice(b"null"),
            },
            ValueType::Array => {
                writer.push(b'[')?;
                if let Some(values) = value.as_array() {
                    for (index, item) in values.enumerate() {
                        let child_depth = depth.saturating_add(1);
                        traversal.check_depth(child_depth, writer)?;
                        if index > 0 {
                            writer.push(b',')?;
                        }
                        Self::write_any_value(&item, writer, traversal, child_depth)?;
                    }
                }
                writer.push(b']')
            }
            ValueType::KeyValueList => {
                writer.push(b'{')?;
                if let Some(entries) = value.as_kvlist() {
                    for (index, entry) in entries.enumerate() {
                        let child_depth = depth.saturating_add(1);
                        traversal.check_depth(child_depth, writer)?;
                        if index > 0 {
                            writer.push(b',')?;
                        }
                        Self::write_json_string(entry.key(), writer)?;
                        writer.push(b':')?;
                        match entry.value() {
                            Some(entry_value) => {
                                Self::write_any_value(
                                    &entry_value,
                                    writer,
                                    traversal,
                                    child_depth,
                                )?;
                            }
                            None => {
                                traversal.visit(child_depth, writer)?;
                                writer.extend_from_slice(b"null")?;
                            }
                        }
                    }
                }
                writer.push(b'}')
            }
            ValueType::Empty => writer.extend_from_slice(b"null"),
        }
    }

    fn write_rfc3339_nano(nanos: u64, writer: &mut JsonWriter) -> Result<(), Error> {
        if nanos == 0 {
            return writer.extend_from_slice(b"\"0001-01-01T00:00:00Z\"");
        }
        let secs = (nanos / 1_000_000_000) as i64;
        let subsec = (nanos % 1_000_000_000) as u32;

        if let Some(dt) = chrono::DateTime::from_timestamp(secs, subsec) {
            writer.push(b'"')?;
            let formatted = dt.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string();
            writer.extend_from_slice(formatted.as_bytes())?;
            writer.push(b'"')
        } else {
            writer.extend_from_slice(b"\"0001-01-01T00:00:00Z\"")
        }
    }

    fn write_hex_string(data: &[u8], writer: &mut JsonWriter) -> Result<(), Error> {
        let encoded_len = data
            .len()
            .checked_mul(2)
            .and_then(|length| length.checked_add(2))
            .unwrap_or(usize::MAX);
        writer.ensure_additional(encoded_len)?;
        encoding::write_json_hex(data, &mut writer.bytes);
        Ok(())
    }

    fn write_json_string(input: &[u8], writer: &mut JsonWriter) -> Result<(), Error> {
        if std::str::from_utf8(input).is_ok() {
            let encoded_len = Self::json_string_content_len(input).saturating_add(2);
            writer.ensure_additional(encoded_len)?;
            encoding::write_json_string(input, &mut writer.bytes);
            return Ok(());
        }

        writer.push(b'"')?;
        let mut remaining = input;
        while !remaining.is_empty() {
            match std::str::from_utf8(remaining) {
                Ok(valid) => {
                    Self::write_json_string_content(valid.as_bytes(), writer)?;
                    break;
                }
                Err(error) => {
                    let valid_length = error.valid_up_to();
                    Self::write_json_string_content(&remaining[..valid_length], writer)?;
                    writer.extend_from_slice("\u{fffd}".as_bytes())?;
                    let invalid_length = error
                        .error_len()
                        .unwrap_or(remaining.len().saturating_sub(valid_length));
                    remaining = &remaining[valid_length.saturating_add(invalid_length)..];
                }
            }
        }
        writer.push(b'"')
    }

    fn json_string_content_len(input: &[u8]) -> usize {
        input.iter().fold(0_usize, |length, byte| {
            length.saturating_add(match byte {
                b'"' | b'\\' | b'\n' | b'\r' | b'\t' => 2,
                0..=0x1f => 6,
                _ => 1,
            })
        })
    }

    fn write_json_string_content(input: &[u8], writer: &mut JsonWriter) -> Result<(), Error> {
        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
        for byte in input {
            match byte {
                b'"' => writer.extend_from_slice(b"\\\"")?,
                b'\\' => writer.extend_from_slice(b"\\\\")?,
                b'\n' => writer.extend_from_slice(b"\\n")?,
                b'\r' => writer.extend_from_slice(b"\\r")?,
                b'\t' => writer.extend_from_slice(b"\\t")?,
                0..=0x1f => writer.extend_from_slice(&[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX_CHARS[(byte >> 4) as usize],
                    HEX_CHARS[(byte & 0x0f) as usize],
                ])?,
                _ => writer.push(*byte)?,
            }
        }
        Ok(())
    }

    fn write_i32(n: i32, writer: &mut JsonWriter) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        writer.extend_from_slice(buf.format(n).as_bytes())
    }

    /// Convert an OTLP metrics view into JSON records matching the ADX
    /// OTELMetrics table schema, exploding Histogram/Summary data points
    /// into sum/count/bucket/quantile rows the same way the Go exporter does.
    #[must_use]
    pub fn convert_metrics<T: MetricsView>(&self, metrics_view: &T) -> Vec<Bytes> {
        self.convert_metrics_with_limits(metrics_view, usize::MAX, usize::MAX, usize::MAX, false)
            .expect("unbounded metric transformation cannot exceed its limits")
    }

    pub(crate) fn convert_metrics_bounded<T: MetricsView>(
        &self,
        metrics_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
    ) -> Result<Vec<Bytes>, Error> {
        self.convert_metrics_with_limits(metrics_view, max_rows, max_row_bytes, max_bytes, true)
    }

    fn convert_metrics_with_limits<T: MetricsView>(
        &self,
        metrics_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
        bounded_traversal: bool,
    ) -> Result<Vec<Bytes>, Error> {
        let mut results =
            RowOutput::new(1024, max_rows, max_row_bytes, max_bytes, bounded_traversal);

        for resource_metrics in metrics_view.resources() {
            let resource = resource_metrics.resource();
            let resource_attrs_json = Self::serialize_attributes_map(
                resource
                    .as_ref()
                    .map(|r| r.attributes())
                    .into_iter()
                    .flatten(),
                &results,
            )?;
            let host =
                Self::serialize_host(resource.as_ref(), self.default_host.as_bytes(), &results)?;

            for scope_metrics in resource_metrics.scopes() {
                let scope = scope_metrics.scope();
                let scope_entries = Self::build_scope_entries(&scope, &results)?;

                for metric in scope_metrics.metrics() {
                    self.convert_metric(
                        &metric,
                        &scope_entries,
                        &resource_attrs_json,
                        &host,
                        &mut results,
                    )?;
                }
            }
        }

        Ok(results.finish())
    }

    fn convert_metric<M: MetricView>(
        &self,
        metric: &M,
        scope_entries: &ScopeEntries,
        resource_attrs_json: &[u8],
        host: &[u8],
        results: &mut RowOutput,
    ) -> Result<(), Error> {
        let Some(data) = metric.data() else {
            return Ok(());
        };
        let name = metric.name();
        let unit = metric.unit();
        let description = metric.description();

        match data.value_type() {
            DataType::Gauge => {
                if let Some(gauge) = data.as_gauge() {
                    for dp in gauge.data_points() {
                        let Some(value) = Self::number_value(dp.value()) else {
                            results.reject_invalid_metric(
                                "number data point does not contain a value",
                            )?;
                            continue;
                        };
                        let attrs_json = Self::serialize_attributes_map(dp.attributes(), results)?;
                        Self::write_metric_row(
                            results,
                            dp.time_unix_nano(),
                            name,
                            b"Gauge",
                            unit,
                            description,
                            value,
                            &attrs_json,
                            scope_entries,
                            resource_attrs_json,
                            host,
                        )?;
                    }
                }
            }
            DataType::Sum => {
                if let Some(sum) = data.as_sum() {
                    for dp in sum.data_points() {
                        let Some(value) = Self::number_value(dp.value()) else {
                            results.reject_invalid_metric(
                                "number data point does not contain a value",
                            )?;
                            continue;
                        };
                        let attrs_json = Self::serialize_attributes_map(dp.attributes(), results)?;
                        Self::write_metric_row(
                            results,
                            dp.time_unix_nano(),
                            name,
                            b"Sum",
                            unit,
                            description,
                            value,
                            &attrs_json,
                            scope_entries,
                            resource_attrs_json,
                            host,
                        )?;
                    }
                }
            }
            DataType::Histogram => {
                if let Some(histogram) = data.as_histogram() {
                    for dp in histogram.data_points() {
                        self.write_histogram_rows(
                            &dp,
                            name,
                            unit,
                            description,
                            scope_entries,
                            resource_attrs_json,
                            host,
                            results,
                        )?;
                    }
                }
            }
            DataType::Summary => {
                if let Some(summary) = data.as_summary() {
                    for dp in summary.data_points() {
                        self.write_summary_rows(
                            &dp,
                            name,
                            unit,
                            description,
                            scope_entries,
                            resource_attrs_json,
                            host,
                            results,
                        )?;
                    }
                }
            }
            DataType::ExponentialHistogram => {
                if let Some(histogram) = data.as_exponential_histogram() {
                    for dp in histogram.data_points() {
                        self.write_exponential_histogram_rows(
                            &dp,
                            name,
                            unit,
                            description,
                            scope_entries,
                            resource_attrs_json,
                            host,
                            results,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn write_histogram_rows<D: HistogramDataPointView>(
        &self,
        dp: &D,
        name: &[u8],
        unit: &[u8],
        description: &[u8],
        scope_entries: &ScopeEntries,
        resource_attrs_json: &[u8],
        host: &[u8],
        results: &mut RowOutput,
    ) -> Result<(), Error> {
        let Some(validated) = Self::validate_histogram(dp) else {
            return results.reject_invalid_metric("explicit histogram is malformed");
        };
        let additional_rows = usize::from(dp.sum().is_some_and(f64::is_finite))
            .saturating_add(1)
            .saturating_add(validated.buckets.len());
        results.ensure_additional_rows(additional_rows)?;

        let ts = Self::normalize_metric_timestamp(dp.time_unix_nano());
        let attrs_json = Self::serialize_attributes_map(dp.attributes(), results)?;
        let sum_name = Self::concatenate(&[name, b"_sum"], results)?;
        let count_name = Self::concatenate(&[name, b"_count"], results)?;
        let bucket_name = Self::concatenate(&[name, b"_bucket"], results)?;
        let sum_description =
            Self::concatenate(&[description, b"(Sum total of samples)"], results)?;
        let count_description = Self::concatenate(&[description, b"(Count of samples)"], results)?;

        if let Some(sum) = dp.sum() {
            Self::write_metric_row(
                results,
                ts,
                &sum_name,
                b"Histogram",
                unit,
                &sum_description,
                sum,
                &attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }
        Self::write_metric_row(
            results,
            ts,
            &count_name,
            b"Histogram",
            unit,
            &count_description,
            validated.total as f64,
            &attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;

        for (bound, cumulative_count) in validated.buckets {
            let le_attrs_json = Self::merge_extra_attr(&attrs_json, "le", bound, results)?;
            Self::write_metric_row(
                results,
                ts,
                &bucket_name,
                b"Histogram",
                unit,
                b"",
                cumulative_count as f64,
                &le_attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }
        Ok(())
    }

    fn validate_histogram<D: HistogramDataPointView>(dp: &D) -> Option<ValidatedHistogram> {
        let bounds = dp.explicit_bounds().collect::<Vec<_>>();
        let counts = dp.bucket_counts().collect::<Vec<_>>();
        if counts.is_empty() && bounds.is_empty() {
            return Some(ValidatedHistogram {
                buckets: Vec::new(),
                total: dp.count(),
            });
        }
        if counts.len() != bounds.len().checked_add(1)? {
            return None;
        }
        if bounds.iter().any(|bound| !bound.is_finite())
            || bounds.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return None;
        }

        let mut running = 0_u64;
        let mut buckets = Vec::with_capacity(counts.len());
        for (bound, count) in bounds.into_iter().zip(counts.iter().copied()) {
            running = running.checked_add(count)?;
            buckets.push((bound, running));
        }
        let total = running.checked_add(*counts.last()?)?;
        if total != dp.count() {
            return None;
        }
        buckets.push((f64::INFINITY, total));

        Some(ValidatedHistogram { buckets, total })
    }

    #[allow(clippy::too_many_arguments)]
    fn write_exponential_histogram_rows<D: ExponentialHistogramDataPointView>(
        &self,
        dp: &D,
        name: &[u8],
        unit: &[u8],
        description: &[u8],
        scope_entries: &ScopeEntries,
        resource_attrs_json: &[u8],
        host: &[u8],
        results: &mut RowOutput,
    ) -> Result<(), Error> {
        let Some(validated) = Self::validate_exponential_histogram(dp) else {
            return results.reject_invalid_metric("exponential histogram is malformed");
        };
        let additional_rows = usize::from(dp.sum().is_some_and(f64::is_finite))
            .saturating_add(3)
            .saturating_add(validated.negative.len())
            .saturating_add(validated.positive.len());
        results.ensure_additional_rows(additional_rows)?;

        let ts = Self::normalize_metric_timestamp(dp.time_unix_nano());
        let attrs_json = Self::serialize_attributes_map(dp.attributes(), results)?;
        let sum_name = Self::concatenate(&[name, b"_sum"], results)?;
        let count_name = Self::concatenate(&[name, b"_count"], results)?;
        let bucket_name = Self::concatenate(&[name, b"_bucket"], results)?;
        let sum_description =
            Self::concatenate(&[description, b"(Sum total of samples)"], results)?;
        let count_description = Self::concatenate(&[description, b"(Count of samples)"], results)?;

        if let Some(sum) = dp.sum() {
            Self::write_metric_row(
                results,
                ts,
                &sum_name,
                b"Histogram",
                unit,
                &sum_description,
                sum,
                &attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }
        Self::write_metric_row(
            results,
            ts,
            &count_name,
            b"Histogram",
            unit,
            &count_description,
            validated.total as f64,
            &attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;

        // Negative buckets are ordered by decreasing index. Their upper
        // boundary is -base^index after reversing the OTLP count array.
        for (bound, cumulative_count) in validated.negative {
            let le_attrs_json = Self::merge_extra_attr(&attrs_json, "le", bound, results)?;
            Self::write_metric_row(
                results,
                ts,
                &bucket_name,
                b"Histogram",
                unit,
                b"",
                cumulative_count as f64,
                &le_attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }

        let le_attrs_json =
            Self::merge_extra_attr(&attrs_json, "le", validated.zero_threshold, results)?;
        Self::write_metric_row(
            results,
            ts,
            &bucket_name,
            b"Histogram",
            unit,
            b"",
            validated.zero_total as f64,
            &le_attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;

        for (bound, cumulative_count) in validated.positive {
            let le_attrs_json = Self::merge_extra_attr(&attrs_json, "le", bound, results)?;
            Self::write_metric_row(
                results,
                ts,
                &bucket_name,
                b"Histogram",
                unit,
                b"",
                cumulative_count as f64,
                &le_attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }

        let le_attrs_json = Self::merge_extra_attr(&attrs_json, "le", f64::INFINITY, results)?;
        Self::write_metric_row(
            results,
            ts,
            &bucket_name,
            b"Histogram",
            unit,
            b"",
            validated.total as f64,
            &le_attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;
        Ok(())
    }

    fn validate_exponential_histogram<D: ExponentialHistogramDataPointView>(
        dp: &D,
    ) -> Option<ValidatedExponentialHistogram> {
        let scale = dp.scale();
        if !(-10..=20).contains(&scale) {
            return None;
        }
        let zero_threshold = dp.zero_threshold();
        if !zero_threshold.is_finite() || zero_threshold < 0.0 {
            return None;
        }

        let (negative_offset, negative_counts) = dp.negative().map_or_else(
            || (0, Vec::new()),
            |buckets| {
                (
                    buckets.offset(),
                    buckets.bucket_counts().collect::<Vec<_>>(),
                )
            },
        );
        let (positive_offset, positive_counts) = dp.positive().map_or_else(
            || (0, Vec::new()),
            |buckets| {
                (
                    buckets.offset(),
                    buckets.bucket_counts().collect::<Vec<_>>(),
                )
            },
        );

        let zero_count = dp.zero_count();
        let total = negative_counts
            .iter()
            .chain(std::iter::once(&zero_count))
            .chain(positive_counts.iter())
            .try_fold(0_u64, |total, count| total.checked_add(*count))?;
        if total != dp.count() {
            return None;
        }

        let mut running = 0_u64;
        let mut negative = Vec::with_capacity(negative_counts.len());
        let mut previous_bound = f64::NEG_INFINITY;
        for position in (0..negative_counts.len()).rev() {
            running = running.checked_add(negative_counts[position])?;
            let position = i64::try_from(position).ok()?;
            let index = i64::from(negative_offset).checked_add(position)?;
            let bound = -Self::exponential_histogram_boundary(scale, index)?;
            if bound <= previous_bound || bound > -zero_threshold {
                return None;
            }
            previous_bound = bound;
            negative.push((bound, running));
        }

        running = running.checked_add(zero_count)?;
        let zero_total = running;
        let mut positive = Vec::with_capacity(positive_counts.len());
        previous_bound = zero_threshold;
        for (position, count) in positive_counts.into_iter().enumerate() {
            running = running.checked_add(count)?;
            let position = i64::try_from(position).ok()?;
            let index = i64::from(positive_offset)
                .checked_add(position)?
                .checked_add(1)?;
            let bound = Self::exponential_histogram_boundary(scale, index)?;
            if bound <= previous_bound {
                return None;
            }
            previous_bound = bound;
            positive.push((bound, running));
        }
        if running != total {
            return None;
        }

        Some(ValidatedExponentialHistogram {
            negative,
            positive,
            zero_total,
            zero_threshold,
            total,
        })
    }

    /// Return `base^boundary_index` for the OTLP base `2^(2^-scale)`.
    fn exponential_histogram_boundary(scale: i32, boundary_index: i64) -> Option<f64> {
        let bucket_width = 2.0_f64.powf(-f64::from(scale));
        let exponent = (boundary_index as f64) * bucket_width;
        let boundary = 2.0_f64.powf(exponent);
        (boundary.is_finite() && boundary > 0.0).then_some(boundary)
    }

    #[allow(clippy::too_many_arguments)]
    fn write_summary_rows<D: SummaryDataPointView>(
        &self,
        dp: &D,
        name: &[u8],
        unit: &[u8],
        description: &[u8],
        scope_entries: &ScopeEntries,
        resource_attrs_json: &[u8],
        host: &[u8],
        results: &mut RowOutput,
    ) -> Result<(), Error> {
        let ts = Self::normalize_metric_timestamp(dp.time_unix_nano());
        let attrs_json = Self::serialize_attributes_map(dp.attributes(), results)?;
        let sum_name = Self::concatenate(&[name, b"_sum"], results)?;
        let count_name = Self::concatenate(&[name, b"_count"], results)?;
        let sum_description =
            Self::concatenate(&[description, b"(Sum total of samples)"], results)?;
        let count_description = Self::concatenate(&[description, b"(Count of samples)"], results)?;

        Self::write_metric_row(
            results,
            ts,
            &sum_name,
            b"Summary",
            unit,
            &sum_description,
            dp.sum(),
            &attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;
        Self::write_metric_row(
            results,
            ts,
            &count_name,
            b"Summary",
            unit,
            &count_description,
            dp.count() as f64,
            &attrs_json,
            scope_entries,
            resource_attrs_json,
            host,
        )?;

        for q in dp.quantile_values() {
            let quantile = Self::format_quantile(q.quantile());
            let quantile_name = Self::concatenate(&[name, b"_", quantile.as_bytes()], results)?;
            let qt_attrs_json = Self::merge_extra_attr(&attrs_json, "qt", q.quantile(), results)?;
            Self::write_metric_row(
                results,
                ts,
                &quantile_name,
                b"Summary",
                unit,
                &count_description,
                q.value(),
                &qt_attrs_json,
                scope_entries,
                resource_attrs_json,
                host,
            )?;
        }
        Ok(())
    }

    /// Format a quantile the same way Go's `strconv.FormatFloat(q, 'f', -1, 64)` does:
    /// the shortest decimal representation with no trailing zeros.
    fn format_quantile(q: f64) -> String {
        if !q.is_finite() {
            return Self::format_dimension(q);
        }
        let (negative, digits, decimal_point) = Self::shortest_decimal_parts(q);
        Self::format_fixed_decimal(negative, &digits, decimal_point)
    }

    /// Format a generated dimension like Go's `strconv.FormatFloat(value, 'g', -1, 64)`.
    fn format_dimension(value: f64) -> String {
        if value.is_nan() {
            return "NaN".to_owned();
        }
        if value == f64::INFINITY {
            return "+Inf".to_owned();
        }
        if value == f64::NEG_INFINITY {
            return "-Inf".to_owned();
        }

        let (negative, digits, decimal_point) = Self::shortest_decimal_parts(value);
        let exponent = decimal_point - 1;
        if !(-4..6).contains(&exponent) {
            let mut formatted = String::with_capacity(digits.len().saturating_add(7));
            if negative {
                formatted.push('-');
            }
            formatted.push(char::from(digits.as_bytes()[0]));
            if digits.len() > 1 {
                formatted.push('.');
                formatted.push_str(&digits[1..]);
            }
            formatted.push('e');
            if exponent < 0 {
                formatted.push('-');
            } else {
                formatted.push('+');
            }
            let exponent = exponent.unsigned_abs();
            if exponent < 10 {
                formatted.push('0');
            }
            formatted.push_str(&exponent.to_string());
            formatted
        } else {
            Self::format_fixed_decimal(negative, &digits, decimal_point)
        }
    }

    /// Decompose a finite float's shortest round-trippable decimal form.
    fn shortest_decimal_parts(value: f64) -> (bool, String, i32) {
        let negative = value.is_sign_negative();
        if value == 0.0 {
            return (negative, "0".to_owned(), 1);
        }

        let mut buf = ryu::Buffer::new();
        let shortest = buf.format(value.abs());
        let exponent_index = shortest
            .bytes()
            .position(|byte| byte == b'e' || byte == b'E');
        let (mantissa, exponent) = exponent_index.map_or((shortest, 0), |index| {
            (
                &shortest[..index],
                shortest[index + 1..]
                    .parse::<i32>()
                    .expect("ryu emits a valid decimal exponent"),
            )
        });
        let mut decimal_point =
            i32::try_from(mantissa.find('.').unwrap_or(mantissa.len())).unwrap_or(i32::MAX);
        decimal_point = decimal_point.saturating_add(exponent);

        let mut digits = mantissa
            .bytes()
            .filter(|byte| *byte != b'.')
            .map(char::from)
            .collect::<String>();
        let leading_zeros = digits.bytes().take_while(|byte| *byte == b'0').count();
        digits.replace_range(..leading_zeros, "");
        decimal_point =
            decimal_point.saturating_sub(i32::try_from(leading_zeros).unwrap_or(i32::MAX));
        while digits.ends_with('0') {
            digits.truncate(digits.len() - 1);
        }

        (negative, digits, decimal_point)
    }

    fn format_fixed_decimal(negative: bool, digits: &str, decimal_point: i32) -> String {
        let mut formatted = String::with_capacity(digits.len().saturating_add(16));
        if negative {
            formatted.push('-');
        }

        if decimal_point <= 0 {
            formatted.push_str("0.");
            for _ in 0..decimal_point.unsigned_abs() {
                formatted.push('0');
            }
            formatted.push_str(digits);
        } else {
            let decimal_point = usize::try_from(decimal_point).unwrap_or(usize::MAX);
            if decimal_point >= digits.len() {
                formatted.push_str(digits);
                for _ in digits.len()..decimal_point {
                    formatted.push('0');
                }
            } else {
                formatted.push_str(&digits[..decimal_point]);
                formatted.push('.');
                formatted.push_str(&digits[decimal_point..]);
            }
        }
        formatted
    }

    /// Resolve the metric `value()` (int or double) to an `f64`.
    fn number_value(value: Option<Value>) -> Option<f64> {
        match value {
            Some(Value::Double(d)) => Some(d),
            Some(Value::Integer(i)) => Some(i as f64),
            None => None,
        }
    }

    /// Serialize the `Host` column from resource `host.name`, falling back to
    /// the exporter process's own hostname.
    fn serialize_host<R: ResourceView>(
        resource: Option<&R>,
        default_host: &[u8],
        results: &RowOutput,
    ) -> Result<Vec<u8>, Error> {
        let mut writer = results.fragment_writer(64);
        if let Some(r) = resource {
            for attr in r.attributes() {
                if attr.key() == HOST_KEY
                    && let Some(val) = attr.value()
                    && let Some(s) = val.as_string()
                {
                    Self::write_json_string(s, &mut writer)?;
                    return Ok(writer.finish());
                }
            }
        }
        Self::write_json_string(default_host, &mut writer)?;
        Ok(writer.finish())
    }

    fn concatenate(parts: &[&[u8]], results: &RowOutput) -> Result<Vec<u8>, Error> {
        let capacity = parts
            .iter()
            .fold(0_usize, |total, part| total.saturating_add(part.len()));
        let mut writer = results.fragment_writer(capacity);
        for part in parts {
            writer.extend_from_slice(part)?;
        }
        Ok(writer.finish())
    }

    /// Merge a synthetic string dimension (e.g. `le`, `qt`) into an
    /// already-serialized attributes JSON object.
    fn merge_extra_attr(
        attrs_json: &[u8],
        key: &str,
        value: f64,
        results: &RowOutput,
    ) -> Result<Vec<u8>, Error> {
        let mut writer = results.fragment_writer(attrs_json.len().saturating_add(32));
        writer.push(b'{')?;
        Self::write_json_string(key.as_bytes(), &mut writer)?;
        writer.push(b':')?;
        let value = Self::format_dimension(value);
        Self::write_json_string(value.as_bytes(), &mut writer)?;
        // attrs_json is always at least "{}"; splice out its leading brace.
        if attrs_json.len() > 2 {
            writer.push(b',')?;
            writer.extend_from_slice(&attrs_json[1..])?;
        } else {
            writer.push(b'}')?;
        }
        Ok(writer.finish())
    }

    #[allow(clippy::too_many_arguments)]
    fn write_metric_row(
        results: &mut RowOutput,
        time_unix_nano: u64,
        name: &[u8],
        metric_type: &[u8],
        unit: &[u8],
        description: &[u8],
        value: f64,
        attrs_json: &[u8],
        scope_entries: &ScopeEntries,
        resource_attrs_json: &[u8],
        host: &[u8],
    ) -> Result<(), Error> {
        if !value.is_finite() {
            return results.reject_invalid_metric("metric value is not finite");
        }

        // Internal telemetry can omit the OTLP datapoint timestamp. ADX needs
        // a valid datetime, so use exporter observation time for that case.
        let time_unix_nano = Self::normalize_metric_timestamp(time_unix_nano);
        let mut buf = results.writer(512)?;
        buf.push(b'{')?;

        buf.extend_from_slice(b"\"Timestamp\":")?;
        Self::write_rfc3339_nano(time_unix_nano, &mut buf)?;

        buf.extend_from_slice(b",\"MetricName\":")?;
        Self::write_json_string(name, &mut buf)?;

        buf.extend_from_slice(b",\"MetricType\":")?;
        Self::write_json_string(metric_type, &mut buf)?;

        buf.extend_from_slice(b",\"MetricUnit\":")?;
        Self::write_json_string(unit, &mut buf)?;

        buf.extend_from_slice(b",\"MetricDescription\":")?;
        Self::write_json_string(description, &mut buf)?;

        buf.extend_from_slice(b",\"MetricValue\":")?;
        let mut ryu_buf = ryu::Buffer::new();
        buf.extend_from_slice(ryu_buf.format(value).as_bytes())?;

        buf.extend_from_slice(b",\"MetricAttributes\":")?;
        Self::write_scope_entries(attrs_json, scope_entries, &mut buf)?;

        buf.extend_from_slice(b",\"Host\":")?;
        buf.extend_from_slice(host)?;

        buf.extend_from_slice(b",\"ResourceAttributes\":")?;
        buf.extend_from_slice(resource_attrs_json)?;

        buf.push(b'}')?;
        results.push(buf.finish())
    }

    fn current_time_unix_nano() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| {
                duration.as_nanos().min(u64::MAX as u128) as u64
            })
    }

    fn normalize_metric_timestamp(time_unix_nano: u64) -> u64 {
        if time_unix_nano == 0 {
            Self::current_time_unix_nano().max(1)
        } else {
            time_unix_nano
        }
    }

    /// Merge scope-level entries (`scope.name`/`scope.version`) into an
    /// already-serialized attributes JSON object, giving datapoint/span
    /// attributes precedence when keys collide (they're appended last, and
    /// Kusto's `dynamic` JSON parsing keeps the last occurrence of a key).
    fn write_scope_entries(
        attrs_json: &[u8],
        scope_entries: &ScopeEntries,
        writer: &mut JsonWriter,
    ) -> Result<(), Error> {
        if scope_entries.json.is_empty() {
            return writer.extend_from_slice(attrs_json);
        }
        writer.push(b'{')?;
        writer.extend_from_slice(&scope_entries.json)?;
        if attrs_json.len() > 2 {
            writer.push(b',')?;
            writer.extend_from_slice(&attrs_json[1..])
        } else {
            writer.push(b'}')
        }
    }

    /// Convert an OTLP traces view into JSON records matching the ADX
    /// OTELTraces table schema.
    #[must_use]
    pub fn convert_traces<T: TracesView>(&self, traces_view: &T) -> Vec<Bytes> {
        self.convert_traces_with_limits(traces_view, usize::MAX, usize::MAX, usize::MAX, false)
            .expect("unbounded trace transformation cannot exceed its limits")
    }

    pub(crate) fn convert_traces_bounded<T: TracesView>(
        &self,
        traces_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
    ) -> Result<Vec<Bytes>, Error> {
        self.convert_traces_with_limits(traces_view, max_rows, max_row_bytes, max_bytes, true)
    }

    fn convert_traces_with_limits<T: TracesView>(
        &self,
        traces_view: &T,
        max_rows: usize,
        max_row_bytes: usize,
        max_bytes: usize,
        bounded_traversal: bool,
    ) -> Result<Vec<Bytes>, Error> {
        let mut results =
            RowOutput::new(256, max_rows, max_row_bytes, max_bytes, bounded_traversal);

        for resource_spans in traces_view.resources() {
            let resource = resource_spans.resource();
            let resource_attrs_json = Self::serialize_attributes_map(
                resource
                    .as_ref()
                    .map(|r| r.attributes())
                    .into_iter()
                    .flatten(),
                &results,
            )?;

            for scope_spans in resource_spans.scopes() {
                let scope = scope_spans.scope();
                let scope_entries = Self::build_scope_entries(&scope, &results)?;

                for span in scope_spans.spans() {
                    let mut buf = results.writer(1024)?;
                    buf.push(b'{')?;

                    buf.extend_from_slice(b"\"TraceID\":")?;
                    match span.trace_id() {
                        Some(id) => Self::write_hex_string(id, &mut buf)?,
                        None => buf.extend_from_slice(b"\"\"")?,
                    }

                    buf.extend_from_slice(b",\"SpanID\":")?;
                    match span.span_id() {
                        Some(id) => Self::write_hex_string(id, &mut buf)?,
                        None => buf.extend_from_slice(b"\"\"")?,
                    }

                    buf.extend_from_slice(b",\"ParentID\":")?;
                    match span.parent_span_id() {
                        Some(id) => Self::write_hex_string(id, &mut buf)?,
                        None => buf.extend_from_slice(b"\"\"")?,
                    }

                    buf.extend_from_slice(b",\"SpanName\":")?;
                    Self::write_json_string(span.name().unwrap_or(b""), &mut buf)?;

                    let status = span.status();
                    buf.extend_from_slice(b",\"SpanStatus\":")?;
                    Self::write_json_string(
                        Self::status_code_str(status.as_ref().map_or(0, |s| s.status_code()))
                            .as_bytes(),
                        &mut buf,
                    )?;

                    buf.extend_from_slice(b",\"SpanStatusMessage\":")?;
                    Self::write_json_string(
                        status.as_ref().and_then(|s| s.message()).unwrap_or(b""),
                        &mut buf,
                    )?;

                    buf.extend_from_slice(b",\"SpanKind\":")?;
                    Self::write_json_string(Self::span_kind_str(span.kind()).as_bytes(), &mut buf)?;

                    buf.extend_from_slice(b",\"StartTime\":")?;
                    Self::write_rfc3339_nano(span.start_time_unix_nano().unwrap_or(0), &mut buf)?;

                    buf.extend_from_slice(b",\"EndTime\":")?;
                    Self::write_rfc3339_nano(span.end_time_unix_nano().unwrap_or(0), &mut buf)?;

                    buf.extend_from_slice(b",\"ResourceAttributes\":")?;
                    buf.extend_from_slice(&resource_attrs_json)?;

                    buf.extend_from_slice(b",\"TraceAttributes\":")?;
                    let trace_attrs_json =
                        Self::serialize_attributes_map(span.attributes(), &results)?;
                    Self::write_scope_entries(&trace_attrs_json, &scope_entries, &mut buf)?;

                    buf.extend_from_slice(b",\"Events\":[")?;
                    for (i, event) in span.events().enumerate() {
                        if i > 0 {
                            buf.push(b',')?;
                        }
                        buf.push(b'{')?;
                        buf.extend_from_slice(b"\"EventName\":")?;
                        Self::write_json_string(event.name().unwrap_or(b""), &mut buf)?;
                        buf.extend_from_slice(b",\"Timestamp\":")?;
                        Self::write_rfc3339_nano(event.time_unix_nano().unwrap_or(0), &mut buf)?;
                        buf.extend_from_slice(b",\"EventAttributes\":")?;
                        let event_attrs =
                            Self::serialize_attributes_map(event.attributes(), &results)?;
                        buf.extend_from_slice(&event_attrs)?;
                        buf.push(b'}')?;
                    }
                    buf.push(b']')?;

                    buf.extend_from_slice(b",\"Links\":[")?;
                    for (i, link) in span.links().enumerate() {
                        if i > 0 {
                            buf.push(b',')?;
                        }
                        buf.push(b'{')?;
                        buf.extend_from_slice(b"\"TraceID\":")?;
                        match link.trace_id() {
                            Some(id) => Self::write_hex_string(id, &mut buf)?,
                            None => buf.extend_from_slice(b"\"\"")?,
                        }
                        buf.extend_from_slice(b",\"SpanID\":")?;
                        match link.span_id() {
                            Some(id) => Self::write_hex_string(id, &mut buf)?,
                            None => buf.extend_from_slice(b"\"\"")?,
                        }
                        buf.extend_from_slice(b",\"TraceState\":")?;
                        Self::write_json_string(link.trace_state().unwrap_or(b""), &mut buf)?;
                        buf.extend_from_slice(b",\"SpanLinkAttributes\":")?;
                        let link_attrs =
                            Self::serialize_attributes_map(link.attributes(), &results)?;
                        buf.extend_from_slice(&link_attrs)?;
                        buf.push(b'}')?;
                    }
                    buf.push(b']')?;

                    buf.push(b'}')?;
                    results.push(buf.finish())?;
                }
            }
        }

        Ok(results.finish())
    }

    /// Map a numeric OTLP span status code to the Go exporter's string names.
    fn status_code_str(code: i32) -> &'static str {
        match code {
            1 => "Ok",
            2 => "Error",
            _ => "Unset",
        }
    }

    /// Map a numeric OTLP span kind to the Go exporter's string names.
    fn span_kind_str(kind: i32) -> &'static str {
        match kind {
            1 => "Internal",
            2 => "Server",
            3 => "Client",
            4 => "Producer",
            5 => "Consumer",
            _ => "Unspecified",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::MAX_SAFE_ROW_BYTES;
    use super::{
        Error, JsonWriter, MAX_ANY_VALUE_NESTING_DEPTH, MAX_ANY_VALUE_TRAVERSAL_WORK, RowOutput,
        ScopeEntries, Transformer,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList, any_value,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
        Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
        exponential_histogram_data_point, metric, number_data_point, summary_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_dfe_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_dfe_pdata::views::otlp::bytes::metrics::RawMetricsData;
    use prost::Message;

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_owned(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_owned())),
            }),
        }
    }

    fn int_attribute(key: &str, value: i64) -> KeyValue {
        KeyValue {
            key: key.to_owned(),
            value: Some(AnyValue {
                value: Some(any_value::Value::IntValue(value)),
            }),
        }
    }

    fn bool_attribute(key: &str, value: bool) -> KeyValue {
        KeyValue {
            key: key.to_owned(),
            value: Some(AnyValue {
                value: Some(any_value::Value::BoolValue(value)),
            }),
        }
    }

    fn composite_scope_attribute() -> KeyValue {
        KeyValue {
            key: "worker.details".to_owned(),
            value: Some(AnyValue {
                value: Some(any_value::Value::ArrayValue(ArrayValue {
                    values: vec![
                        AnyValue {
                            value: Some(any_value::Value::StringValue("primary".to_owned())),
                        },
                        AnyValue {
                            value: Some(any_value::Value::KvlistValue(KeyValueList {
                                values: vec![
                                    bool_attribute("active", true),
                                    KeyValue {
                                        key: "levels".to_owned(),
                                        value: Some(AnyValue {
                                            value: Some(any_value::Value::ArrayValue(ArrayValue {
                                                values: vec![
                                                    AnyValue {
                                                        value: Some(any_value::Value::IntValue(1)),
                                                    },
                                                    AnyValue { value: None },
                                                ],
                                            })),
                                        }),
                                    },
                                ],
                            })),
                        },
                    ],
                })),
            }),
        }
    }

    fn log_request(event_name: &str, attributes: Vec<KeyValue>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        event_name: event_name.to_owned(),
                        attributes,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn exponential_histogram_request(
        point: ExponentialHistogramDataPoint,
    ) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "test.scope".to_owned(),
                        ..Default::default()
                    }),
                    metrics: vec![Metric {
                        name: "request.duration".to_owned(),
                        description: "Request duration".to_owned(),
                        unit: "ms".to_owned(),
                        data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                            data_points: vec![point],
                            aggregation_temporality: 1,
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn explicit_histogram_request(point: HistogramDataPoint) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "test.scope".to_owned(),
                        ..Default::default()
                    }),
                    metrics: vec![Metric {
                        name: "request.size".to_owned(),
                        description: "Request size".to_owned(),
                        unit: "By".to_owned(),
                        data: Some(metric::Data::Histogram(Histogram {
                            data_points: vec![point],
                            aggregation_temporality: 1,
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn summary_request(point: SummaryDataPoint) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "test.scope".to_owned(),
                        ..Default::default()
                    }),
                    metrics: vec![Metric {
                        name: "request.latency".to_owned(),
                        description: "Request latency".to_owned(),
                        unit: "ms".to_owned(),
                        data: Some(metric::Data::Summary(Summary {
                            data_points: vec![point],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn gauge_request(value: Option<number_data_point::Value>) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "temperature".to_owned(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                value,
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn metric_rows(request: ExportMetricsServiceRequest) -> Vec<serde_json::Value> {
        let bytes = request.encode_to_vec();
        Transformer::new()
            .convert_metrics(&RawMetricsData::new(&bytes))
            .iter()
            .map(|row| serde_json::from_slice(row).expect("metric row should be valid JSON"))
            .collect()
    }

    fn bounded_metric_error(request: ExportMetricsServiceRequest) -> Error {
        let bytes = request.encode_to_vec();
        Transformer::new()
            .convert_metrics_bounded(
                &RawMetricsData::new(&bytes),
                usize::MAX,
                usize::MAX,
                usize::MAX,
            )
            .expect_err("bounded conversion must refuse malformed metric data")
    }

    /// Scenario: bounded conversion receives missing, NaN, or infinite number data point values.
    /// Guarantees: values ADX cannot represent are refused instead of being substituted or dropped.
    #[test]
    fn invalid_number_data_points_are_refused() {
        for (value, reason) in [
            (None, "number data point does not contain a value"),
            (
                Some(number_data_point::Value::AsDouble(f64::NAN)),
                "metric value is not finite",
            ),
            (
                Some(number_data_point::Value::AsDouble(f64::INFINITY)),
                "metric value is not finite",
            ),
        ] {
            assert!(matches!(
                bounded_metric_error(gauge_request(value)),
                Error::InvalidMetricData { reason: actual } if actual == reason
            ));
        }
    }

    /// Scenario: a numeric log body is transformed in evolved and legacy schema modes.
    /// Guarantees: evolved mode emits a dynamic number while legacy mode emits its string representation.
    #[test]
    fn log_body_encoding_respects_legacy_toggle() {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(any_value::Value::IntValue(42)),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = request.encode_to_vec();
        let view = RawLogsData::new(&bytes);

        let evolved = Transformer::new().convert_logs(&view);
        let legacy = Transformer::new()
            .with_legacy_logs_body_string(true)
            .convert_logs(&view);

        assert!(String::from_utf8_lossy(&evolved[0]).contains("\"Body\":42"));
        assert!(String::from_utf8_lossy(&legacy[0]).contains("\"Body\":\"42\""));
    }

    /// Scenario: a log body contains nested OTLP arrays and key-value lists.
    /// Guarantees: evolved mode preserves the complete nested body as native JSON.
    #[test]
    fn composite_log_body_is_encoded_as_dynamic_json() {
        let body = AnyValue {
            value: Some(any_value::Value::ArrayValue(ArrayValue {
                values: vec![
                    AnyValue {
                        value: Some(any_value::Value::StringValue("primary".to_owned())),
                    },
                    AnyValue {
                        value: Some(any_value::Value::KvlistValue(KeyValueList {
                            values: vec![
                                bool_attribute("active", true),
                                KeyValue {
                                    key: "levels".to_owned(),
                                    value: Some(AnyValue {
                                        value: Some(any_value::Value::ArrayValue(ArrayValue {
                                            values: vec![
                                                AnyValue {
                                                    value: Some(any_value::Value::IntValue(1)),
                                                },
                                                AnyValue { value: None },
                                            ],
                                        })),
                                    }),
                                },
                            ],
                        })),
                    },
                ],
            })),
        };
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        body: Some(body),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = request.encode_to_vec();
        let view = RawLogsData::new(&bytes);

        let rows = Transformer::new().convert_logs(&view);
        let row: serde_json::Value = serde_json::from_slice(&rows[0]).expect("valid JSON row");

        assert_eq!(
            row["Body"],
            serde_json::json!(["primary", {"active": true, "levels": [1, null]}])
        );
    }

    /// Scenario: JSON escaping expands a log body beyond the configured per-row byte limit.
    /// Guarantees: bounded conversion returns RowTooLarge while emitting the string, before an oversized row is built.
    #[test]
    fn bounded_log_conversion_rejects_large_escaped_string_during_emission() {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(
                                "\n".repeat(MAX_SAFE_ROW_BYTES / 2),
                            )),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_logs_bounded(
                &RawLogsData::new(&bytes),
                usize::MAX,
                MAX_SAFE_ROW_BYTES,
                usize::MAX,
            )
            .expect_err("escaped output must honor the row budget during serialization");

        assert!(matches!(
            error,
            Error::RowTooLarge {
                actual,
                limit: MAX_SAFE_ROW_BYTES
            } if actual > MAX_SAFE_ROW_BYTES
        ));
    }

    /// Scenario: an operator configures a row limit below the 900 KiB safety ceiling.
    /// Guarantees: bounded conversion enforces the configured limit during row emission.
    #[test]
    fn bounded_log_conversion_honors_configured_row_limit() {
        let request = log_request("", Vec::new());
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_logs_bounded(&RawLogsData::new(&bytes), usize::MAX, 64, usize::MAX)
            .expect_err("configured row limit must be enforced");

        assert!(matches!(
            error,
            Error::RowTooLarge { actual, limit: 64 } if actual > 64
        ));
    }

    /// Scenario: one log row consumes nearly all of the configured request byte budget.
    /// Guarantees: the second row is rejected with RequestTooLarge while it is being emitted.
    #[test]
    fn bounded_log_conversion_honors_remaining_request_budget() {
        let mut request = log_request("", Vec::new());
        let one_record_bytes = request.encode_to_vec();
        let one_row = Transformer::new().convert_logs(&RawLogsData::new(&one_record_bytes));
        let max_bytes = one_row[0].len().saturating_add(1);
        request.resource_logs[0].scope_logs[0]
            .log_records
            .push(LogRecord::default());
        let two_record_bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_logs_bounded(
                &RawLogsData::new(&two_record_bytes),
                usize::MAX,
                MAX_SAFE_ROW_BYTES,
                max_bytes,
            )
            .expect_err("the second row must honor the remaining request budget");

        assert!(matches!(
            error,
            Error::RequestTooLarge {
                actual,
                limit
            } if actual > limit && limit == max_bytes
        ));
    }

    /// Scenario: a log body contains AnyValue arrays nested beyond the bounded traversal depth.
    /// Guarantees: bounded conversion rejects the body with RowTooLarge instead of recursing without a work limit.
    #[test]
    fn bounded_log_conversion_rejects_deep_any_value_nesting() {
        let mut body = AnyValue {
            value: Some(any_value::Value::IntValue(1)),
        };
        for _ in 0..=MAX_ANY_VALUE_NESTING_DEPTH {
            body = AnyValue {
                value: Some(any_value::Value::ArrayValue(ArrayValue {
                    values: vec![body],
                })),
            };
        }
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        body: Some(body),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_logs_bounded(
                &RawLogsData::new(&bytes),
                usize::MAX,
                MAX_SAFE_ROW_BYTES,
                usize::MAX,
            )
            .expect_err("deep AnyValue traversal must be bounded");

        assert!(matches!(
            error,
            Error::RowTooLarge {
                actual,
                limit: MAX_SAFE_ROW_BYTES
            } if actual > MAX_SAFE_ROW_BYTES
        ));
    }

    /// Scenario: many separate attributes collectively exceed the traversal-work budget.
    /// Guarantees: the work limit applies to the complete source conversion, not each attribute independently.
    #[test]
    fn bounded_log_conversion_shares_traversal_budget_across_attributes() {
        let attributes = (0..=MAX_ANY_VALUE_TRAVERSAL_WORK)
            .map(|_| int_attribute("a", 1))
            .collect();
        let request = log_request("", attributes);
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_logs_bounded(
                &RawLogsData::new(&bytes),
                usize::MAX,
                MAX_SAFE_ROW_BYTES,
                usize::MAX,
            )
            .expect_err("source-wide traversal work must be bounded");

        assert!(matches!(
            error,
            Error::RowTooLarge {
                limit: MAX_SAFE_ROW_BYTES,
                ..
            }
        ));
    }

    /// Scenario: top-level and attribute-level event-name output are configured independently.
    /// Guarantees: each option controls only its documented representation of the OTLP event name.
    #[test]
    fn log_event_name_options_are_independent() {
        let bytes = log_request("checkout.completed", Vec::new()).encode_to_vec();
        let view = RawLogsData::new(&bytes);

        let attribute_only = Transformer::new()
            .with_log_event_name_options(false, true)
            .convert_logs(&view);
        let attribute_only: serde_json::Value =
            serde_json::from_slice(&attribute_only[0]).expect("log row should be valid JSON");
        assert!(attribute_only.get("EventName").is_none());
        assert_eq!(
            attribute_only["LogsAttributes"]["event.name"],
            "checkout.completed"
        );

        let column_only = Transformer::new()
            .with_log_event_name_options(true, false)
            .convert_logs(&view);
        let column_only: serde_json::Value =
            serde_json::from_slice(&column_only[0]).expect("log row should be valid JSON");
        assert_eq!(column_only["EventName"], "checkout.completed");
        assert!(column_only["LogsAttributes"].get("event.name").is_none());
    }

    /// Scenario: a log record already has an `event.name` attribute and also has an OTLP event name.
    /// Guarantees: fallback enrichment preserves the existing attribute and emits the key only once.
    #[test]
    fn existing_event_name_attribute_is_not_overwritten() {
        let bytes = log_request(
            "record-event",
            vec![string_attribute("event.name", "attribute-event")],
        )
        .encode_to_vec();
        let view = RawLogsData::new(&bytes);

        let rows = Transformer::new().convert_logs(&view);
        let row = String::from_utf8_lossy(&rows[0]);
        let parsed: serde_json::Value =
            serde_json::from_slice(&rows[0]).expect("log row should be valid JSON");

        assert_eq!(parsed["EventName"], "record-event");
        assert_eq!(parsed["LogsAttributes"]["event.name"], "attribute-event");
        assert_eq!(row.matches("\"event.name\"").count(), 1);
    }

    /// Scenario: browser-shaped OTLP protobuf has metric metadata before populated data oneofs.
    /// Guarantees: ADX rows contain metric values and recursively serialized scope attributes.
    #[test]
    fn converts_browser_shaped_raw_protobuf_metrics() {
        let point = NumberDataPoint {
            time_unix_nano: 1_700_000_000_000_000_000,
            value: Some(number_data_point::Value::AsDouble(42.5)),
            attributes: vec![string_attribute("node.id", "point-node")],
            ..Default::default()
        };
        let metadata = vec![string_attribute("test.instrument.id", "browser-test")];
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![string_attribute("service.name", "browser-test")],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "browser.scope".to_owned(),
                        version: "1.0.0".to_owned(),
                        attributes: vec![
                            string_attribute("node.id", "scope-node"),
                            int_attribute("core.id", 3),
                            bool_attribute("shared", true),
                            composite_scope_attribute(),
                        ],
                        ..Default::default()
                    }),
                    metrics: vec![
                        Metric {
                            name: "browser.memory.usage".to_owned(),
                            unit: "By".to_owned(),
                            metadata: metadata.clone(),
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![point.clone()],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "browser.http.requests".to_owned(),
                            unit: "{request}".to_owned(),
                            metadata: metadata.clone(),
                            data: Some(metric::Data::Sum(Sum {
                                data_points: vec![NumberDataPoint {
                                    value: Some(number_data_point::Value::AsInt(7)),
                                    ..point.clone()
                                }],
                                aggregation_temporality: 1,
                                is_monotonic: true,
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "browser.http.server.duration".to_owned(),
                            unit: "ms".to_owned(),
                            metadata,
                            data: Some(metric::Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    time_unix_nano: point.time_unix_nano,
                                    count: 3,
                                    sum: Some(325.0),
                                    bucket_counts: vec![1, 1, 1],
                                    explicit_bounds: vec![50.0, 250.0],
                                    ..Default::default()
                                }],
                                aggregation_temporality: 1,
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let bytes = request.encode_to_vec();
        let rows = Transformer::new().convert_metrics(&RawMetricsData::new(&bytes));
        let rows = rows
            .iter()
            .map(|row| std::str::from_utf8(row).expect("metric row should be UTF-8"))
            .collect::<Vec<_>>();

        assert_eq!(rows.len(), 7);
        assert!(rows.iter().any(|row| {
            row.contains(r#""MetricName":"browser.memory.usage""#)
                && row.contains(r#""MetricValue":42.5"#)
                && row.contains(
                    r#""MetricAttributes":{"scope.name":"browser.scope","scope.version":"1.0.0","node.id":"scope-node","core.id":3,"shared":true,"worker.details":["primary",{"active":true,"levels":[1,null]}],"node.id":"point-node"}"#
                )
        }));
        assert!(rows.iter().any(|row| {
            row.contains(r#""MetricName":"browser.http.requests""#)
                && row.contains(r#""MetricValue":7.0"#)
        }));
        assert!(rows.iter().any(|row| {
            row.contains(r#""MetricName":"browser.http.server.duration_count""#)
                && row.contains(r#""MetricValue":3.0"#)
        }));
    }

    /// Scenario: an explicit histogram provides count and sum but omits both distribution arrays.
    /// Guarantees: the protocol-valid point emits exactly its sum and count rows without bucket rows.
    #[test]
    fn explicit_histogram_without_distribution_emits_sum_and_count_only() {
        let rows = metric_rows(explicit_histogram_request(HistogramDataPoint {
            count: 7,
            sum: Some(12.5),
            ..Default::default()
        }));

        let values = rows
            .iter()
            .map(|row| {
                (
                    row["MetricName"].as_str().expect("metric name"),
                    row["MetricValue"].as_f64().expect("metric value"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![("request.size_sum", 12.5), ("request.size_count", 7.0)]
        );
        assert!(
            rows.iter()
                .all(|row| row["MetricName"] != "request.size_bucket")
        );
    }

    /// Scenario: an explicit histogram contains a finite bound and an implicit positive-infinity bound.
    /// Guarantees: generated `le` dimensions are exact Go-compatible JSON strings.
    #[test]
    fn explicit_histogram_bounds_are_string_dimensions() {
        let rows = metric_rows(explicit_histogram_request(HistogramDataPoint {
            count: 3,
            bucket_counts: vec![1, 2],
            explicit_bounds: vec![0.5],
            ..Default::default()
        }));

        let buckets = rows
            .iter()
            .filter(|row| row["MetricName"] == "request.size_bucket")
            .map(|row| {
                (
                    row["MetricAttributes"]["le"]
                        .as_str()
                        .expect("le must be a string"),
                    row["MetricValue"].as_f64().expect("bucket value"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(buckets, vec![("0.5", 1.0), ("+Inf", 3.0)]);
    }

    /// Scenario: summary quantiles use a small scientific value and an ordinary finite value.
    /// Guarantees: names match Go fixed formatting and generated `qt` dimensions are JSON strings.
    #[test]
    fn summary_quantile_names_and_dimensions_match_go_formatting() {
        let rows = metric_rows(summary_request(SummaryDataPoint {
            count: 2,
            sum: 3.0,
            quantile_values: vec![
                summary_data_point::ValueAtQuantile {
                    quantile: 1e-10,
                    value: 1.0,
                },
                summary_data_point::ValueAtQuantile {
                    quantile: 0.5,
                    value: 2.0,
                },
            ],
            ..Default::default()
        }));

        let quantiles = rows
            .iter()
            .filter(|row| {
                row["MetricName"] != "request.latency_sum"
                    && row["MetricName"] != "request.latency_count"
            })
            .map(|row| {
                (
                    row["MetricName"].as_str().expect("quantile name"),
                    row["MetricAttributes"]["qt"]
                        .as_str()
                        .expect("qt must be a string"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            quantiles,
            vec![
                ("request.latency_0.0000000001", "1e-10"),
                ("request.latency_0.5", "0.5"),
            ]
        );
    }

    /// Scenario: generated dimensions contain positive and negative infinity.
    /// Guarantees: both values use Go spellings and are encoded as JSON strings.
    #[test]
    fn generated_infinite_dimensions_use_go_string_spellings() {
        let results = RowOutput::new(0, usize::MAX, usize::MAX, usize::MAX, false);

        let positive =
            Transformer::merge_extra_attr(b"{}", "le", f64::INFINITY, &results).expect("le");
        let negative =
            Transformer::merge_extra_attr(b"{}", "qt", f64::NEG_INFINITY, &results).expect("qt");

        assert_eq!(positive.as_slice(), br#"{"le":"+Inf"}"#);
        assert_eq!(negative.as_slice(), br#"{"qt":"-Inf"}"#);
    }

    /// Scenario: an explicit histogram omits its optional sum and source timestamp.
    /// Guarantees: no sum is fabricated and all derived rows reuse one fallback timestamp.
    #[test]
    fn histogram_omits_absent_sum_and_reuses_fallback_timestamp() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource::default()),
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "request.size".to_owned(),
                        data: Some(metric::Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                count: 2,
                                sum: None,
                                bucket_counts: vec![1, 1],
                                explicit_bounds: vec![10.0],
                                time_unix_nano: 0,
                                ..Default::default()
                            }],
                            aggregation_temporality: 1,
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let rows = metric_rows(request);
        assert!(
            rows.iter()
                .all(|row| row["MetricName"] != "request.size_sum")
        );
        let timestamps = rows
            .iter()
            .map(|row| row["Timestamp"].clone())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(timestamps.len(), 1);
    }

    /// Scenario: bounded conversion receives explicit histograms with invalid structure or totals.
    /// Guarantees: the complete source message is refused instead of being ACKed after silent row loss.
    #[test]
    fn malformed_explicit_histograms_are_refused() {
        let malformed_points = [
            (
                "overflow",
                HistogramDataPoint {
                    count: u64::MAX,
                    sum: Some(1.0),
                    bucket_counts: vec![u64::MAX, 1],
                    explicit_bounds: vec![1.0],
                    ..Default::default()
                },
            ),
            (
                "cardinality",
                HistogramDataPoint {
                    count: 1,
                    sum: Some(1.0),
                    bucket_counts: vec![1],
                    explicit_bounds: vec![1.0],
                    ..Default::default()
                },
            ),
            (
                "bounds without counts",
                HistogramDataPoint {
                    count: 0,
                    sum: Some(1.0),
                    explicit_bounds: vec![1.0],
                    ..Default::default()
                },
            ),
            (
                "order",
                HistogramDataPoint {
                    count: 0,
                    sum: Some(1.0),
                    bucket_counts: vec![0, 0, 0],
                    explicit_bounds: vec![2.0, 1.0],
                    ..Default::default()
                },
            ),
            (
                "non-finite bound",
                HistogramDataPoint {
                    count: 0,
                    sum: Some(1.0),
                    bucket_counts: vec![0, 0],
                    explicit_bounds: vec![f64::NAN],
                    ..Default::default()
                },
            ),
            (
                "count mismatch",
                HistogramDataPoint {
                    count: 3,
                    sum: Some(1.0),
                    bucket_counts: vec![1, 1],
                    explicit_bounds: vec![1.0],
                    ..Default::default()
                },
            ),
        ];

        for (case, point) in malformed_points {
            assert!(
                matches!(
                    bounded_metric_error(explicit_histogram_request(point)),
                    Error::InvalidMetricData {
                        reason: "explicit histogram is malformed"
                    }
                ),
                "{case} must be refused"
            );
        }
    }

    /// Scenario: an exponential histogram has negative, zero-region, and positive observations.
    /// Guarantees: ADX bucket rows use ordered OTLP upper bounds and cumulative counts across all ranges.
    #[test]
    fn converts_exponential_histogram_buckets_to_cumulative_rows() {
        let rows = metric_rows(exponential_histogram_request(
            ExponentialHistogramDataPoint {
                attributes: vec![string_attribute("route", "/checkout")],
                time_unix_nano: 1_700_000_000_000_000_000,
                count: 15,
                sum: Some(8.5),
                scale: -1,
                zero_count: 4,
                zero_threshold: 0.25,
                negative: Some(exponential_histogram_data_point::Buckets {
                    offset: -1,
                    bucket_counts: vec![2, 1],
                }),
                positive: Some(exponential_histogram_data_point::Buckets {
                    offset: -1,
                    bucket_counts: vec![3, 5],
                }),
                ..Default::default()
            },
        ));

        assert_eq!(rows.len(), 8);
        let buckets = rows
            .iter()
            .filter(|row| row["MetricName"] == "request.duration_bucket")
            .map(|row| {
                (
                    row["MetricAttributes"]["le"].clone(),
                    row["MetricValue"].as_f64().expect("finite metric value"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            buckets,
            vec![
                (serde_json::json!("-1"), 1.0),
                (serde_json::json!("-0.25"), 3.0),
                (serde_json::json!("0.25"), 7.0),
                (serde_json::json!("1"), 10.0),
                (serde_json::json!("4"), 15.0),
                (serde_json::json!("+Inf"), 15.0),
            ]
        );
        assert!(rows.iter().all(|row| {
            row["Timestamp"] == serde_json::json!("2023-11-14T22:13:20.000000000Z")
        }));
        assert_eq!(
            rows.iter()
                .find(|row| row["MetricName"] == "request.duration_sum")
                .expect("sum row")["MetricValue"],
            8.5
        );
        assert_eq!(
            rows.iter()
                .find(|row| row["MetricName"] == "request.duration_count")
                .expect("count row")["MetricAttributes"]["route"],
            "/checkout"
        );
        assert!(
            rows.iter()
                .filter(|row| row["MetricName"] == "request.duration_bucket")
                .all(|row| row["MetricAttributes"]["route"] == "/checkout")
        );
    }

    /// Scenario: exponential histogram expansion would exceed the request row budget.
    /// Guarantees: transformation preflights the complete point and refuses it before row emission.
    #[test]
    fn bounded_metric_conversion_rejects_expanded_row_count() {
        let request = exponential_histogram_request(ExponentialHistogramDataPoint {
            count: 3,
            sum: Some(3.0),
            scale: 0,
            positive: Some(exponential_histogram_data_point::Buckets {
                offset: 0,
                bucket_counts: vec![1, 2],
            }),
            ..Default::default()
        });
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_metrics_bounded(
                &RawMetricsData::new(&bytes),
                2,
                MAX_SAFE_ROW_BYTES,
                usize::MAX,
            )
            .expect_err("expanded rows must honor the configured limit");

        assert!(matches!(
            error,
            Error::TooManyRows {
                actual: 6,
                limit: 2
            }
        ));
    }

    /// Scenario: the first transformed metric row exceeds the JSON Lines byte budget.
    /// Guarantees: transformation refuses the row without returning an oversized collection.
    #[test]
    fn bounded_metric_conversion_rejects_encoded_bytes() {
        let request = exponential_histogram_request(ExponentialHistogramDataPoint {
            count: 1,
            sum: Some(1.0),
            scale: 0,
            positive: Some(exponential_histogram_data_point::Buckets {
                offset: 0,
                bucket_counts: vec![1],
            }),
            ..Default::default()
        });
        let bytes = request.encode_to_vec();

        let error = Transformer::new()
            .convert_metrics_bounded(
                &RawMetricsData::new(&bytes),
                usize::MAX,
                MAX_SAFE_ROW_BYTES,
                1,
            )
            .expect_err("encoded bytes must honor the configured limit");

        assert!(matches!(
            error,
            Error::RequestTooLarge { actual, limit: 1 } if actual > 1
        ));
    }

    /// Scenario: an exponential histogram datapoint already has an attribute named `le`.
    /// Guarantees: generated bucket rows retain the existing explicit-histogram attribute precedence.
    #[test]
    fn exponential_histogram_preserves_existing_le_attribute_precedence() {
        let rows = metric_rows(exponential_histogram_request(
            ExponentialHistogramDataPoint {
                attributes: vec![string_attribute("le", "point-value")],
                count: 1,
                zero_count: 1,
                ..Default::default()
            },
        ));

        assert_eq!(
            rows.iter()
                .filter(|row| row["MetricName"] == "request.duration_bucket")
                .count(),
            2
        );
        assert!(
            rows.iter()
                .filter(|row| row["MetricName"] == "request.duration_bucket")
                .all(|row| row["MetricAttributes"]["le"] == "point-value")
        );
    }

    /// Scenario: an exponential histogram omits its optional sum and source timestamp.
    /// Guarantees: no sum is fabricated and every emitted row reuses one normalized timestamp.
    #[test]
    fn exponential_histogram_omits_absent_sum_and_reuses_fallback_timestamp() {
        let rows = metric_rows(exponential_histogram_request(
            ExponentialHistogramDataPoint {
                count: 1,
                sum: None,
                zero_count: 1,
                zero_threshold: 0.0,
                ..Default::default()
            },
        ));

        assert_eq!(rows.len(), 3);
        assert!(
            rows.iter()
                .all(|row| row["MetricName"] != "request.duration_sum")
        );
        assert_eq!(rows[0]["Timestamp"], rows[1]["Timestamp"]);
        assert_eq!(rows[1]["Timestamp"], rows[2]["Timestamp"]);
    }

    /// Scenario: a zero-only exponential histogram uses a scale outside the OTLP range.
    /// Guarantees: bounded conversion refuses the point even when it has no positive or negative buckets.
    #[test]
    fn zero_only_exponential_histogram_rejects_scale_21() {
        let error = bounded_metric_error(exponential_histogram_request(
            ExponentialHistogramDataPoint {
                count: 3,
                sum: Some(4.5),
                scale: 21,
                zero_count: 3,
                zero_threshold: 0.125,
                ..Default::default()
            },
        ));

        assert!(matches!(
            error,
            Error::InvalidMetricData {
                reason: "exponential histogram is malformed"
            }
        ));
    }

    /// Scenario: bounded conversion receives exponential histograms with invalid structure or totals.
    /// Guarantees: the complete source message is refused instead of being ACKed after silent row loss.
    #[test]
    fn malformed_exponential_histograms_are_refused() {
        let malformed_points = [
            (
                "invalid threshold",
                ExponentialHistogramDataPoint {
                    count: 1,
                    sum: Some(1.0),
                    zero_count: 1,
                    zero_threshold: f64::NAN,
                    ..Default::default()
                },
            ),
            (
                "count overflow",
                ExponentialHistogramDataPoint {
                    count: u64::MAX,
                    sum: Some(1.0),
                    zero_count: 1,
                    positive: Some(exponential_histogram_data_point::Buckets {
                        offset: 0,
                        bucket_counts: vec![u64::MAX],
                    }),
                    ..Default::default()
                },
            ),
            (
                "count mismatch",
                ExponentialHistogramDataPoint {
                    count: 2,
                    sum: Some(1.0),
                    positive: Some(exponential_histogram_data_point::Buckets {
                        offset: 0,
                        bucket_counts: vec![1],
                    }),
                    ..Default::default()
                },
            ),
            (
                "boundary overflow",
                ExponentialHistogramDataPoint {
                    count: 2,
                    sum: Some(1.0),
                    scale: -10,
                    negative: Some(exponential_histogram_data_point::Buckets {
                        offset: 1,
                        bucket_counts: vec![1],
                    }),
                    positive: Some(exponential_histogram_data_point::Buckets {
                        offset: 1,
                        bucket_counts: vec![1],
                    }),
                    ..Default::default()
                },
            ),
        ];

        for (case, point) in malformed_points {
            assert!(
                matches!(
                    bounded_metric_error(exponential_histogram_request(point)),
                    Error::InvalidMetricData {
                        reason: "exponential histogram is malformed"
                    }
                ),
                "{case} must be refused"
            );
        }
    }

    /// Scenario: scope metadata is merged into an empty datapoint attribute object.
    /// Guarantees: the resulting JSON object has no leading or trailing comma.
    #[test]
    fn merge_scope_entries_does_not_add_trailing_comma_to_empty_attributes() {
        let scope_entries = ScopeEntries {
            json: br#""scope.name":"pipeline""#.to_vec(),
            has_event_name: false,
        };
        let mut writer = JsonWriter::new(64, usize::MAX, 0, usize::MAX);
        Transformer::write_scope_entries(b"{}", &scope_entries, &mut writer)
            .expect("unbounded writer");

        assert_eq!(writer.finish(), br#"{"scope.name":"pipeline"}"#);
    }

    /// Scenario: scope metadata is merged with existing datapoint attributes.
    /// Guarantees: scope and datapoint entries remain separated by valid JSON punctuation.
    #[test]
    fn merge_scope_entries_separates_scope_and_datapoint_attributes() {
        let scope_entries = ScopeEntries {
            json: br#""scope.name":"pipeline""#.to_vec(),
            has_event_name: false,
        };
        let mut writer = JsonWriter::new(64, usize::MAX, 0, usize::MAX);
        Transformer::write_scope_entries(br#"{"unit":"By"}"#, &scope_entries, &mut writer)
            .expect("unbounded writer");

        assert_eq!(writer.finish(), br#"{"scope.name":"pipeline","unit":"By"}"#);
    }
}
