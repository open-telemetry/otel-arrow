// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::{BufMut, Bytes, BytesMut};
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

use super::config::{ATTRIBUTES_PASSTHROUGH_MARKER, Config, SchemaConfig};
use super::error::Error;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// The mandatory Log Analytics time column. When no mapping targets it, the
/// exporter injects it from the log record's event time so a record carries its
/// true event time rather than the ingestion arrival time Azure stamps on a
/// missing `TimeGenerated`.
const TIME_GENERATED_COLUMN: &str = "TimeGenerated";

/// Pre-serialized JSON key for [`TIME_GENERATED_COLUMN`] (`"TimeGenerated":`).
/// Fixed, so it is a const rather than a per-schema field.
const TIME_GENERATED_KEY_JSON: &[u8] = b"\"TimeGenerated\":";

/// Pre-parsed field mapping for a log record field
#[derive(Debug, Clone)]
struct FieldMapping {
    /// The source field name (e.g., "time_unix_nano", "body")
    source: LogRecordField,
    /// Destination column name (e.g. `TimeGenerated`).
    dest_name: String,
    /// Pre-serialized JSON key with quotes and colon, e.g. `"TimeGenerated":`
    dest_key_json: Vec<u8>,
}

/// Enum representing known log record fields for fast matching
#[derive(Debug, Clone, Copy)]
enum LogRecordField {
    TimeUnixNano,
    ObservedTimeUnixNano,
    TraceId,
    SpanId,
    Flags,
    SeverityNumber,
    SeverityText,
    Body,
    EventName,
}

impl LogRecordField {
    /// Parse a field name string into a LogRecordField enum
    fn from_str(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("time_unix_nano") {
            Some(Self::TimeUnixNano)
        } else if s.eq_ignore_ascii_case("observed_time_unix_nano") {
            Some(Self::ObservedTimeUnixNano)
        } else if s.eq_ignore_ascii_case("trace_id") {
            Some(Self::TraceId)
        } else if s.eq_ignore_ascii_case("span_id") {
            Some(Self::SpanId)
        } else if s.eq_ignore_ascii_case("flags") {
            Some(Self::Flags)
        } else if s.eq_ignore_ascii_case("severity_number") {
            Some(Self::SeverityNumber)
        } else if s.eq_ignore_ascii_case("severity_text") {
            Some(Self::SeverityText)
        } else if s.eq_ignore_ascii_case("body") {
            Some(Self::Body)
        } else if s.eq_ignore_ascii_case("event_name") {
            // Add this
            Some(Self::EventName)
        } else {
            None
        }
    }
}

/// Pre-parsed schema configuration for faster runtime processing
#[derive(Debug, Clone)]
struct ParsedSchema {
    /// Resource attribute mappings (source -> dest)
    resource_mapping: HashMap<String, String>,
    /// Scope attribute mappings (source -> dest)
    scope_mapping: HashMap<String, String>,
    /// Pre-parsed log record field mappings
    field_mappings: Vec<FieldMapping>,
    /// Pre-parsed attribute mappings (source attr -> pre-serialized JSON key)
    attribute_mapping: HashMap<String, Vec<u8>>,
    /// When true (`attributes: passthrough`), every log record attribute is
    /// emitted as-is as a top-level `"<key>": <value>` pair (the attribute key
    /// becomes the column name).
    attribute_passthrough: bool,
    /// Whether to auto-inject the mandatory `TimeGenerated` column.
    ///
    /// Rule: **always inject `TimeGenerated` unless it is already mapped or
    /// present.** This flag captures the config half of that rule -- it is
    /// `true` when no configured mapping (resource, scope, field, or attribute)
    /// targets the `TimeGenerated` column, computed once here at parse time so
    /// the hot path is a single branch. The "present" half -- a runtime
    /// passthrough attribute literally named `TimeGenerated` -- is detected at
    /// emit time and also suppresses injection (innermost-wins).
    ///
    /// When injecting, the value is the record's event time (`time_unix_nano`,
    /// falling back to `observed_time_unix_nano`, then now); see
    /// [`Self::write_default_time_generated_value`].
    default_time_generated: bool,
    /// All destination column names produced by explicit mappings (resource,
    /// scope, and log-record fields). Used only in attribute-passthrough mode to
    /// detect when an attribute key collides with a mapped column, compared
    /// against `attr.key()` bytes via `as_bytes()` (no allocation). Empty when no
    /// explicit mappings are configured (e.g. pure passthrough). Kept as a small
    /// `Vec` and gated by [`Self::reserved_len_mask`] so most attributes are
    /// rejected on length alone, without a byte comparison.
    reserved_columns: Vec<String>,
    /// Bitmask of the byte-lengths present in `reserved_columns` (bit `n` set if
    /// some reserved name has length `n`; lengths >= 63 map to bit 63). An
    /// attribute whose key length is absent here cannot collide, so the byte
    /// comparison is skipped entirely.
    reserved_len_mask: u64,
}

impl ParsedSchema {
    fn from_config(schema: &SchemaConfig) -> Result<Self, Error> {
        let mut field_mappings = Vec::new();
        let mut attribute_mapping = HashMap::new();
        let mut attribute_passthrough = false;

        for (key, value) in &schema.log_record_mapping {
            if key.eq_ignore_ascii_case("attributes") {
                match value {
                    // `attributes: passthrough` emits every attribute as-is as a
                    // top-level key/value pair.
                    Value::String(s) if s == ATTRIBUTES_PASSTHROUGH_MARKER => {
                        attribute_passthrough = true;
                    }
                    // `attributes: { key: Column, ... }` maps specific attributes.
                    Value::Object(attr_map) => {
                        for (attr_key, attr_dest) in attr_map {
                            let dest = attr_dest
                                .as_str()
                                .map(String::from)
                                .unwrap_or_else(|| attr_dest.to_string());
                            _ = attribute_mapping
                                .insert(attr_key.clone(), serialize_json_key(&dest));
                        }
                    }
                    _ => {
                        return Err(Error::InvalidFieldMapping { field: key.clone() });
                    }
                }
            } else {
                // Parse field mapping
                let source = LogRecordField::from_str(key)
                    .ok_or_else(|| Error::UnknownLogRecordField { field: key.clone() })?;
                let dest = value
                    .as_str()
                    .ok_or_else(|| Error::InvalidFieldMapping { field: key.clone() })?;
                field_mappings.push(FieldMapping {
                    source,
                    dest_name: dest.to_string(),
                    dest_key_json: serialize_json_key(dest),
                });
            }
        }

        let mut reserved_columns: Vec<String> = Vec::new();
        let mut reserved_len_mask: u64 = 0;

        // Inject the mandatory `TimeGenerated` column from the record's event
        // time unless a mapping already targets it. Detection is done once here
        // (cold path); an explicit resource/scope/field/attribute mapping to
        // `TimeGenerated` (a runtime passthrough attribute named `TimeGenerated`
        // is handled at emit time, innermost-wins) disables injection.
        let time_generated_explicitly_mapped = schema
            .resource_mapping
            .values()
            .any(|d| d == TIME_GENERATED_COLUMN)
            || schema
                .scope_mapping
                .values()
                .any(|d| d == TIME_GENERATED_COLUMN)
            || field_mappings
                .iter()
                .any(|fm| fm.dest_name == TIME_GENERATED_COLUMN)
            || attribute_mapping
                .values()
                .any(|k| k.as_slice() == TIME_GENERATED_KEY_JSON);
        let default_time_generated = !time_generated_explicitly_mapped;

        if attribute_passthrough {
            let mut push = |name: &str| {
                reserved_len_mask |= 1u64 << (name.len().min(63) as u64);
                reserved_columns.push(name.to_string());
            };
            for dest in schema.resource_mapping.values() {
                push(dest);
            }
            for dest in schema.scope_mapping.values() {
                push(dest);
            }
            for fm in &field_mappings {
                push(&fm.dest_name);
            }
            // So a runtime passthrough attribute named `TimeGenerated` is
            // detected as a collision and wins over the injected default.
            if default_time_generated {
                push(TIME_GENERATED_COLUMN);
            }
        }

        Ok(Self {
            resource_mapping: schema.resource_mapping.clone(),
            scope_mapping: schema.scope_mapping.clone(),
            field_mappings,
            attribute_mapping,
            attribute_passthrough,
            default_time_generated,
            reserved_columns,
            reserved_len_mask,
        })
    }

    /// Returns true if `key` matches a reserved (mapped) column name. Rejects most
    /// keys on length alone via `reserved_len_mask` before any byte comparison.
    #[inline]
    fn is_reserved_column(&self, key: &[u8]) -> bool {
        let bit = 1u64 << (key.len().min(63) as u64);
        if self.reserved_len_mask & bit == 0 {
            return false;
        }
        self.reserved_columns.iter().any(|c| c.as_bytes() == key)
    }
}

/// Pre-serialize a JSON key with quotes and trailing colon: `"key":`
/// Called once at config parse time so the hot path can use `extend_from_slice`.
fn serialize_json_key(key: &str) -> Vec<u8> {
    // JSON keys from config are simple ASCII identifiers; no escaping needed in practice,
    // but we go through write_json_string for correctness.
    let mut buf = Vec::with_capacity(key.len() + 3); // '"' + key + '"' + ':'
    Transformer::write_json_string_public(key.as_bytes(), &mut buf);
    buf.push(b':');
    buf
}

/// Converts OTLP logs to Azure Log Analytics format
#[derive(Debug)]
pub struct Transformer {
    schema: ParsedSchema,
}

impl Transformer {
    /// Create a new Transformer with the given configuration
    ///
    /// # Panics
    /// Panics if the schema configuration contains invalid field names
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self::try_new(config).expect("Invalid schema configuration")
    }

    /// Create a new Transformer, returning an error if configuration is invalid
    pub fn try_new(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            schema: ParsedSchema::from_config(&config.api.schema)?,
        })
    }

    /// High-perf, single-threaded: pre-serializes resource+scope fields once per ScopeLogs,
    /// then writes per-record fields directly to the buffer — no Map cloning or re-serialization.
    /// Assumes non-overlapping mappings across resource, scope, and log record levels.
    #[must_use]
    // TODO: When Rust generators stabilize (rust-lang/rust#117078), replace
    // Vec<Bytes> return with a generator that yields &[u8] per record,
    // allowing the caller to feed records directly into GzipBatcher
    // without the intermediate Vec allocation and second iteration pass.
    pub fn convert_to_log_analytics<T: LogsDataView>(&self, logs_view: &T) -> Vec<Bytes> {
        let schema = &self.schema;
        let mut results = Vec::with_capacity(1024);
        let mut buf = BytesMut::with_capacity(2048);
        let mut base_map = serde_json::Map::new();
        let mut record_buf = Vec::with_capacity(512);
        // Reused scratch for the combined (passthrough + mappings) path.
        let mut overridden: Vec<String> = Vec::new();
        let mut scratch = Vec::new();

        for resource_logs in logs_view.resources() {
            base_map.clear();
            if let Some(r) = resource_logs.resource() {
                Self::apply_resource_mapping(schema, &r, &mut base_map);
            }

            for scope_logs in resource_logs.scopes() {
                // Clone resource base once per ScopeLogs, add scope mappings
                let mut scope_map = base_map.clone();
                if let Some(s) = scope_logs.scope() {
                    Self::apply_scope_mapping(schema, &s, &mut scope_map);
                }

                // Pre-serialize resource+scope as JSON bytes (once per ScopeLogs)
                // Safety: base_map only contains valid JSON values from convert_any_value
                let base_json = serde_json::to_vec(&scope_map).unwrap_or_default();
                let has_base = base_json.len() > 2; // more than just "{}"
                // Strip trailing '}' to allow appending record fields
                let base_prefix = &base_json[..base_json.len() - 1];

                // Passthrough combined with explicit mappings needs collision
                // handling (innermost/attributes win); pure passthrough and pure
                // mapped records stay on the fast raw path. Records are built in a
                // reused Vec and bulk-copied into `buf` — benchmarks show one
                // amortized memcpy beats many small writes straight into BytesMut.
                let combined =
                    schema.attribute_passthrough && (has_base || !schema.field_mappings.is_empty());

                for log_record in scope_logs.log_records() {
                    buf.clear();

                    if combined {
                        record_buf.clear();
                        overridden.clear();
                        // Emit innermost-first: attributes, then fields; record
                        // which mapped columns an attribute overrides.
                        let has_record = Self::write_passthrough_record_inner(
                            schema,
                            &log_record,
                            &mut record_buf,
                            &mut overridden,
                        );

                        buf.put_u8(b'{');
                        let mut has = has_record;
                        if has_record {
                            buf.extend_from_slice(&record_buf);
                        }
                        // Append resource/scope columns, dropping any overridden
                        // by an attribute so keys stay unique.
                        if overridden.is_empty() {
                            if has_base {
                                if has {
                                    buf.put_u8(b',');
                                }
                                // base_json inner content (strip both braces)
                                buf.extend_from_slice(&base_json[1..base_json.len() - 1]);
                            }
                        } else {
                            for (k, v) in &scope_map {
                                if overridden.contains(k) {
                                    continue;
                                }
                                if has {
                                    buf.put_u8(b',');
                                }
                                has = true;
                                scratch.clear();
                                Self::write_json_string(k.as_bytes(), &mut scratch);
                                scratch.push(b':');
                                let _ = serde_json::to_writer(&mut scratch, v);
                                buf.extend_from_slice(&scratch);
                            }
                        }
                        buf.put_u8(b'}');
                    } else {
                        record_buf.clear();
                        let has_record =
                            Self::write_record_fields_json(schema, &log_record, &mut record_buf);

                        match (has_base, has_record) {
                            (true, true) => {
                                buf.extend_from_slice(base_prefix);
                                buf.put_u8(b',');
                                buf.extend_from_slice(&record_buf);
                                buf.put_u8(b'}');
                            }
                            (true, false) => {
                                buf.extend_from_slice(&base_json);
                            }
                            (false, true) => {
                                buf.put_u8(b'{');
                                buf.extend_from_slice(&record_buf);
                                buf.put_u8(b'}');
                            }
                            (false, false) => {
                                buf.extend_from_slice(b"{}");
                            }
                        }
                    }

                    results.push(buf.split().freeze());
                }
            }
        }

        results
    }

    /// Combined passthrough emit: write log attributes (innermost) first, then the
    /// mapped log-record fields, skipping any field whose column an attribute
    /// already emitted. Records attribute keys that collide with a mapped column
    /// (resource/scope/field) in `overridden` so the caller can drop those base
    /// columns. Returns true if anything was written.
    fn write_passthrough_record_inner<R: LogRecordView>(
        schema: &ParsedSchema,
        log_record: &R,
        out: &mut Vec<u8>,
        overridden: &mut Vec<String>,
    ) -> bool {
        let mut has = false;
        let check_reserved = !schema.reserved_columns.is_empty();

        for attr in log_record.attributes() {
            if let Some(val) = attr.value() {
                if has {
                    out.push(b',');
                }
                has = true;
                Self::write_json_string(attr.key(), out);
                out.push(b':');
                Self::write_any_value_json(&val, out);
                // Reject most attribute keys on length alone (no hashing); only
                // an actual collision (rare) materializes the key as a String.
                if check_reserved && schema.is_reserved_column(attr.key()) {
                    overridden.push(String::from_utf8_lossy(attr.key()).into_owned());
                }
            }
        }

        for fm in &schema.field_mappings {
            if !overridden.is_empty() && overridden.contains(&fm.dest_name) {
                continue;
            }
            if has {
                out.push(b',');
            }
            has = true;
            out.extend_from_slice(&fm.dest_key_json);
            Self::write_field_value_json(fm.source, log_record, out);
        }

        // Inject TimeGenerated from event time, unless a passthrough attribute
        // named `TimeGenerated` already emitted it (recorded in `overridden`,
        // innermost-wins).
        if schema.default_time_generated && !overridden.iter().any(|c| c == TIME_GENERATED_COLUMN) {
            if has {
                out.push(b',');
            }
            has = true;
            out.extend_from_slice(TIME_GENERATED_KEY_JSON);
            Self::write_default_time_generated_value(log_record, out);
        }

        has
    }

    /// Write log record fields directly as JSON key:value pairs (no braces) to a byte buffer.
    /// Returns true if any fields were written.
    fn write_record_fields_json<R: LogRecordView>(
        schema: &ParsedSchema,
        log_record: &R,
        out: &mut Vec<u8>,
    ) -> bool {
        let mut has_field = false;
        // Track whether a passthrough attribute named `TimeGenerated` was
        // emitted, so the injected default below is skipped (attribute wins).
        let track_tg = schema.default_time_generated && schema.attribute_passthrough;
        let mut tg_from_attr = false;

        for fm in &schema.field_mappings {
            if has_field {
                out.push(b',');
            }
            has_field = true;
            // Write pre-serialized key (e.g. `"TimeGenerated":`)
            out.extend_from_slice(&fm.dest_key_json);
            // Write value directly — avoids Value allocation for simple types
            Self::write_field_value_json(fm.source, log_record, out);
        }

        if !schema.attribute_mapping.is_empty() {
            for attr in log_record.attributes() {
                let attr_key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
                if let Some(dest) = schema.attribute_mapping.get(attr_key.as_ref()) {
                    if let Some(val) = attr.value() {
                        if has_field {
                            out.push(b',');
                        }
                        has_field = true;
                        out.extend_from_slice(dest);
                        Self::write_any_value_json(&val, out);
                    }
                }
            }
        }

        // Attribute passthrough: emit every log attribute as-is as a top-level
        // `"<key>": <value>` pair (the attribute key becomes the column name).
        // Mutually exclusive with `attribute_mapping` (the `attributes` config is
        // either the `passthrough` marker or an explicit mapping object).
        if schema.attribute_passthrough {
            for attr in log_record.attributes() {
                if let Some(val) = attr.value() {
                    if has_field {
                        out.push(b',');
                    }
                    has_field = true;
                    Self::write_json_string(attr.key(), out);
                    out.push(b':');
                    Self::write_any_value_json(&val, out);
                    if track_tg && attr.key() == TIME_GENERATED_COLUMN.as_bytes() {
                        tg_from_attr = true;
                    }
                }
            }
        }

        // Inject TimeGenerated from event time, unless a passthrough attribute
        // named `TimeGenerated` already emitted it (innermost-wins).
        if schema.default_time_generated && !tg_from_attr {
            if has_field {
                out.push(b',');
            }
            has_field = true;
            out.extend_from_slice(TIME_GENERATED_KEY_JSON);
            Self::write_default_time_generated_value(log_record, out);
        }

        has_field
    }

    /// Write the injected `TimeGenerated` value from the record's event time:
    /// `time_unix_nano`, falling back to `observed_time_unix_nano`, then (both
    /// unset) to the current time via [`Self::write_timestamp_json`]. This is the
    /// last point in the pipeline that still holds the record's real event time;
    /// omitting it would make Azure stamp `TimeGenerated` with the ingestion
    /// arrival time instead.
    #[inline]
    fn write_default_time_generated_value<R: LogRecordView>(log_record: &R, out: &mut Vec<u8>) {
        let ts = match log_record.time_unix_nano() {
            Some(t) if t != 0 => t,
            _ => log_record.observed_time_unix_nano().unwrap_or(0),
        };
        Self::write_timestamp_json(ts, out);
    }

    /// Write a field value directly to JSON bytes, avoiding intermediate Value allocation.
    #[inline]
    fn write_field_value_json<R: LogRecordView>(
        field: LogRecordField,
        log_record: &R,
        out: &mut Vec<u8>,
    ) {
        match field {
            LogRecordField::TimeUnixNano => {
                Self::write_timestamp_json(log_record.time_unix_nano().unwrap_or(0), out);
            }
            LogRecordField::ObservedTimeUnixNano => {
                Self::write_timestamp_json(log_record.observed_time_unix_nano().unwrap_or(0), out);
            }
            LogRecordField::TraceId => match log_record.trace_id() {
                Some(id) => Self::write_json_hex(id, out),
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::SpanId => match log_record.span_id() {
                Some(id) => Self::write_json_hex(id, out),
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::Flags => {
                Self::write_u64(log_record.flags().unwrap_or(0).into(), out);
            }
            LogRecordField::SeverityNumber => {
                Self::write_u64(log_record.severity_number().unwrap_or(0) as u64, out);
            }
            LogRecordField::SeverityText => {
                Self::write_json_string(log_record.severity_text().unwrap_or(b""), out);
            }
            LogRecordField::Body => match log_record.body() {
                Some(b) => Self::write_body_value_json(&b, out),
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::EventName => {
                Self::write_json_string(log_record.event_name().unwrap_or(b""), out);
            }
        }
    }

    /// Write an AnyValueView directly as JSON bytes.
    fn write_any_value_json<'a, V: AnyValueView<'a>>(value: &V, out: &mut Vec<u8>) {
        match value.value_type() {
            ValueType::String => match value.as_string() {
                Some(s) => Self::write_json_string(s, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Int64 => match value.as_int64() {
                Some(n) => Self::write_i64(n, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Double => match value.as_double() {
                Some(d) if d.is_finite() => {
                    let mut ryu_buf = ryu::Buffer::new();
                    out.extend_from_slice(ryu_buf.format_finite(d).as_bytes());
                }
                _ => out.extend_from_slice(b"null"),
            },
            ValueType::Bool => match value.as_bool() {
                Some(true) => out.extend_from_slice(b"true"),
                Some(false) => out.extend_from_slice(b"false"),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Bytes => match value.as_bytes() {
                Some(b) => Self::write_json_hex(b, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Array | ValueType::KeyValueList => {
                let v = Self::convert_any_value(value);
                _ = serde_json::to_writer(out, &v);
            }
            ValueType::Empty => out.extend_from_slice(b"null"),
        }
    }

    /// Write a body AnyValueView as a JSON string directly.
    /// For the common string case with valid UTF-8, writes raw bytes directly
    /// to avoid the intermediate String allocation from `extract_string_value`.
    /// Falls back to `from_utf8_lossy` for malformed input to guarantee valid JSON.
    #[inline]
    fn write_body_value_json<'a, V: AnyValueView<'a>>(value: &V, out: &mut Vec<u8>) {
        match value.value_type() {
            ValueType::String => match value.as_string() {
                Some(s) => {
                    let lossy = String::from_utf8_lossy(s);
                    Self::write_json_string(lossy.as_bytes(), out);
                }
                None => out.extend_from_slice(b"null"),
            },
            _ => {
                let s = Self::extract_string_value(value);
                Self::write_json_string(s.as_bytes(), out);
            }
        }
    }

    /// Public wrapper for `write_json_string`, used by `serialize_json_key` at config time.
    #[doc(hidden)]
    pub fn write_json_string_public(s: &[u8], out: &mut Vec<u8>) {
        Self::write_json_string(s, out);
    }

    /// Write a JSON-escaped string with quotes directly to output bytes.
    /// Uses bulk-copy for runs of safe bytes to minimize per-byte overhead.
    #[inline]
    fn write_json_string(s: &[u8], out: &mut Vec<u8>) {
        out.push(b'"');
        let mut start = 0;
        for (i, &b) in s.iter().enumerate() {
            // `\n`, `\r`, `\t` are all `< 0x20`, so `b < 0x20` already covers them;
            // only `"` and `\` need explicit checks. The escape arms below still
            // emit the short `\n`/`\r`/`\t` forms, so output is unchanged.
            let needs_escape = b == b'"' || b == b'\\' || b < 0x20;
            if needs_escape {
                if start < i {
                    out.extend_from_slice(&s[start..i]);
                }
                match b {
                    b'"' => out.extend_from_slice(b"\\\""),
                    b'\\' => out.extend_from_slice(b"\\\\"),
                    b'\n' => out.extend_from_slice(b"\\n"),
                    b'\r' => out.extend_from_slice(b"\\r"),
                    b'\t' => out.extend_from_slice(b"\\t"),
                    _ => {
                        // Control characters: \u00XX
                        out.extend_from_slice(b"\\u00");
                        out.push(HEX_CHARS[(b >> 4) as usize]);
                        out.push(HEX_CHARS[(b & 0x0f) as usize]);
                    }
                }
                start = i + 1;
            }
        }
        if start < s.len() {
            out.extend_from_slice(&s[start..]);
        }
        out.push(b'"');
    }

    /// Write hex-encoded bytes as a JSON string directly.
    #[inline]
    fn write_json_hex(bytes: &[u8], out: &mut Vec<u8>) {
        out.push(b'"');
        for &byte in bytes {
            out.push(HEX_CHARS[(byte >> 4) as usize]);
            out.push(HEX_CHARS[(byte & 0x0f) as usize]);
        }
        out.push(b'"');
    }

    /// Write a u64 as decimal to output bytes.
    #[inline]
    fn write_u64(n: u64, out: &mut Vec<u8>) {
        let mut itoa_buf = itoa::Buffer::new();
        out.extend_from_slice(itoa_buf.format(n).as_bytes());
    }

    /// Write an i64 as decimal to output bytes.
    #[inline]
    fn write_i64(n: i64, out: &mut Vec<u8>) {
        let mut itoa_buf = itoa::Buffer::new();
        out.extend_from_slice(itoa_buf.format(n).as_bytes());
    }

    /// Write a 2-digit zero-padded number directly to output bytes.
    #[inline]
    fn write_2_digits(n: u32, out: &mut Vec<u8>) {
        out.push(b'0' + (n / 10) as u8);
        out.push(b'0' + (n % 10) as u8);
    }

    /// Write a 4-digit zero-padded year directly to output bytes.
    #[inline]
    fn write_4_digits(n: u32, out: &mut Vec<u8>) {
        out.push(b'0' + (n / 1000) as u8);
        out.push(b'0' + ((n / 100) % 10) as u8);
        out.push(b'0' + ((n / 10) % 10) as u8);
        out.push(b'0' + (n % 10) as u8);
    }

    /// Write nanoseconds with dot prefix, trimming trailing zero groups of 3
    /// to match chrono's `to_rfc3339()` output (0, 3, 6, or 9 fractional digits).
    #[inline]
    fn write_nanos_trimmed(nanos: u32, out: &mut Vec<u8>) {
        out.push(b'.');
        let mut digits = [0u8; 9];
        let mut n = nanos;
        for d in digits.iter_mut().rev() {
            *d = b'0' + (n % 10) as u8;
            n /= 10;
        }
        // Trim in groups of 3: 9 -> 6 -> 3 (never 0, caller checks nanos > 0)
        let mut len = 9;
        while len > 3
            && digits[len - 1] == b'0'
            && digits[len - 2] == b'0'
            && digits[len - 3] == b'0'
        {
            len -= 3;
        }
        out.extend_from_slice(&digits[..len]);
    }

    /// Write a JSON-quoted RFC3339 timestamp directly to the output buffer.
    /// Avoids String allocation since RFC3339 contains no JSON-special characters.
    #[inline]
    fn write_timestamp_json(time_unix_nano: u64, out: &mut Vec<u8>) {
        if time_unix_nano == 0 {
            let ts = chrono::Utc::now().to_rfc3339();
            Self::write_json_string(ts.as_bytes(), out);
            return;
        }
        let secs = (time_unix_nano / 1_000_000_000) as i64;
        let nanos = (time_unix_nano % 1_000_000_000) as u32;
        match chrono::DateTime::from_timestamp(secs, nanos) {
            Some(dt) => {
                use chrono::{Datelike, Timelike};
                let year = dt.year();
                // Safety: fall back to allocating path for unusual years
                if !(0..=9999).contains(&year) {
                    let ts = dt.to_rfc3339();
                    Self::write_json_string(ts.as_bytes(), out);
                    return;
                }
                out.push(b'"');
                Self::write_4_digits(year as u32, out);
                out.push(b'-');
                Self::write_2_digits(dt.month(), out);
                out.push(b'-');
                Self::write_2_digits(dt.day(), out);
                out.push(b'T');
                Self::write_2_digits(dt.hour(), out);
                out.push(b':');
                Self::write_2_digits(dt.minute(), out);
                out.push(b':');
                Self::write_2_digits(dt.second(), out);
                if nanos > 0 {
                    Self::write_nanos_trimmed(nanos, out);
                }
                out.extend_from_slice(b"+00:00");
                out.push(b'"');
            }
            None => {
                let ts = chrono::Utc::now().to_rfc3339();
                Self::write_json_string(ts.as_bytes(), out);
            }
        }
    }

    /// Apply resource mapping based on configuration
    fn apply_resource_mapping<R: ResourceView>(
        schema: &ParsedSchema,
        resource: &R,
        map: &mut serde_json::Map<String, Value>,
    ) {
        for attr in resource.attributes() {
            let key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
            if let Some(mapped_name) = schema.resource_mapping.get(key.as_ref()) {
                if let Some(value) = attr.value() {
                    _ = map.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }
    }

    /// Apply scope mapping based on configuration
    fn apply_scope_mapping<S: InstrumentationScopeView>(
        schema: &ParsedSchema,
        scope: &S,
        map: &mut serde_json::Map<String, Value>,
    ) {
        for attr in scope.attributes() {
            let key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
            if let Some(mapped_name) = schema.scope_mapping.get(key.as_ref()) {
                if let Some(value) = attr.value() {
                    _ = map.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }
    }

    /// Convert Str<'_> (&[u8]) to String
    #[inline]
    fn str_to_string(s: Str<'_>) -> String {
        String::from_utf8_lossy(s).into_owned()
    }

    /// Hot path - hex encoding for trace_id/span_id
    #[inline]
    fn bytes_to_hex(bytes: &[u8]) -> String {
        let mut hex = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            hex.push(HEX_CHARS[(byte >> 4) as usize] as char);
            hex.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
        }
        hex
    }

    /// Convert AnyValueView to serde_json::Value
    fn convert_any_value<'a, V: AnyValueView<'a>>(value: &V) -> Value {
        match value.value_type() {
            ValueType::String => {
                Value::String(Self::str_to_string(value.as_string().unwrap_or(b"")))
            }
            ValueType::Int64 => Value::Number(value.as_int64().unwrap_or(0).into()),
            ValueType::Double => serde_json::Number::from_f64(value.as_double().unwrap_or(0.0))
                .map(Value::Number)
                .unwrap_or(Value::Null),
            ValueType::Bool => Value::Bool(value.as_bool().unwrap_or(false)),
            ValueType::Bytes => Value::String(Self::bytes_to_hex(value.as_bytes().unwrap_or(&[]))),
            ValueType::Array => value
                .as_array()
                .map(|iter| Value::Array(iter.map(|v| Self::convert_any_value(&v)).collect()))
                .unwrap_or_else(|| Value::Array(Vec::new())),
            ValueType::KeyValueList => value
                .as_kvlist()
                .map(|iter| {
                    let map: serde_json::Map<String, Value> = iter
                        .filter_map(|kv| {
                            kv.value().map(|v| {
                                (Self::str_to_string(kv.key()), Self::convert_any_value(&v))
                            })
                        })
                        .collect();
                    Value::Object(map)
                })
                .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
            ValueType::Empty => Value::Null,
        }
    }

    /// Extract string representation from AnyValueView (for body)
    fn extract_string_value<'a, V: AnyValueView<'a>>(value: &V) -> String {
        match value.value_type() {
            ValueType::String => Self::str_to_string(value.as_string().unwrap_or(b"")),
            ValueType::Int64 => value.as_int64().unwrap_or(0).to_string(),
            ValueType::Double => value.as_double().unwrap_or(0.0).to_string(),
            ValueType::Bool => value.as_bool().unwrap_or(false).to_string(),
            ValueType::Bytes => Self::bytes_to_hex(value.as_bytes().unwrap_or(&[])),
            ValueType::Array => value
                .as_array()
                .map(|iter| {
                    let vals: Vec<String> = iter.map(|v| Self::extract_string_value(&v)).collect();
                    format!("[{}]", vals.join(", "))
                })
                .unwrap_or_else(|| "[]".into()),
            ValueType::KeyValueList => Self::convert_any_value(value).to_string(),
            ValueType::Empty => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{
            AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList,
            any_value::Value as OtelAnyValueEnum,
        },
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use prost::Message;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        use super::super::config::{ApiConfig, HeartbeatConfig, SchemaConfig};

        Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".into(),
                stream_name: "test-stream".into(),
                dcr: "test-dcr".into(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::from([(
                        "service.name".into(),
                        "ServiceName".into(),
                    )]),
                    scope_mapping: HashMap::from([("scope.name".into(), "ScopeName".into())]),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!("Body")),
                        ("severity_text".into(), json!("Severity")),
                        ("attributes".into(), json!({"test.attr": "TestAttr"})),
                    ]),
                },
                azure_monitor_source_resourceid: None,
                gzip_compression_level: 6,
                user_agent: None,
            },
            heartbeat: HeartbeatConfig::default(),
        }
    }

    #[test]
    fn test_schema_mapping() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test-scope".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("my-scope".into())),
                            }),
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::IntValue(42)),
                        }),
                        severity_text: "INFO".into(),
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(true)),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["ServiceName"], "my-service");
        assert_eq!(json["ScopeName"], "my-scope");
        assert_eq!(json["Body"], "42");
        assert_eq!(json["Severity"], "INFO");
        assert_eq!(json["TestAttr"], true);
    }

    #[test]
    fn test_passthrough_mode() {
        let mut config = create_test_config();
        // `attributes: passthrough` emits all log attributes into one dynamic
        // column, with no resource/scope/field mappings.
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("hello".into())),
                        }),
                        severity_text: "INFO".into(),
                        attributes: vec![
                            KeyValue {
                                key: "user.id".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("abc".into())),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "http.status".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::IntValue(200)),
                                }),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // Each log attribute is emitted as its own top-level column, key verbatim.
        assert_eq!(json["user.id"], "abc");
        assert_eq!(json["http.status"], 200);
        // Nothing else is emitted: no field/resource/scope mappings.
        assert!(json.get("Attributes").is_none());
        assert!(json.get("Body").is_none());
        assert!(json.get("ServiceName").is_none());
        // The mandatory TimeGenerated is injected from the record's event time.
        assert!(json.get("TimeGenerated").is_some());
        // The two attribute columns plus the injected TimeGenerated.
        assert_eq!(json.as_object().unwrap().len(), 3);
    }

    #[test]
    fn test_passthrough_no_attributes_emits_only_injected_time_generated() {
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("hi".into())),
                        }),
                        time_unix_nano: 1_700_000_000_000_000_000,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // No attributes and no mappings -> the row carries only the injected
        // TimeGenerated (from the record's event time).
        assert_eq!(json.as_object().unwrap().len(), 1);
        assert_eq!(json["TimeGenerated"], "2023-11-14T22:13:20+00:00");
    }

    #[test]
    fn test_passthrough_composed_with_mappings() {
        // `attributes: passthrough` composes with resource, scope, and top-level
        // field mappings: the mapped columns plus each attribute as its own
        // top-level column are all emitted.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::from([("service.name".into(), "ServiceName".into())]),
            scope_mapping: HashMap::from([("scope.name".into(), "ScopeName".into())]),
            log_record_mapping: HashMap::from([
                ("body".into(), json!("Body")),
                ("attributes".into(), json!("passthrough")),
            ]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("svc".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "s".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("scp".into())),
                            }),
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("hello".into())),
                        }),
                        attributes: vec![KeyValue {
                            key: "user.id".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("abc".into())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // Mapped columns.
        assert_eq!(json["ServiceName"], "svc");
        assert_eq!(json["ScopeName"], "scp");
        assert_eq!(json["Body"], "hello");
        // Log attribute emitted as its own top-level column.
        assert_eq!(json["user.id"], "abc");
        // Mapped columns + attribute + injected TimeGenerated.
        assert!(json.get("TimeGenerated").is_some());
        assert_eq!(json.as_object().unwrap().len(), 5);
    }

    #[test]
    fn test_passthrough_composed_multiple_records_reset_overridden() {
        // Two log records share one scope in the combined (passthrough + base
        // mapping) path. The first has an attribute that overrides the mapped
        // resource column; the second does not. This exercises the per-record
        // reset of the `overridden` scratch buffer: the base column must be
        // suppressed for the first record and reappear for the second, with no
        // state leaking between records.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::from([("service.name".into(), "Shared".into())]),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("from-resource".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        // Record 1: attribute "Shared" collides with the mapped
                        // resource column -> attribute wins, base dropped.
                        LogRecord {
                            attributes: vec![KeyValue {
                                key: "Shared".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("from-attr".into())),
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                        // Record 2: no collision -> base "Shared" reappears
                        // alongside the passthrough attribute.
                        LogRecord {
                            attributes: vec![KeyValue {
                                key: "other".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("val".into())),
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 2);

        // Record 1: attribute overrides the base column; the injected
        // TimeGenerated is the only other key.
        let json0: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json0["Shared"], "from-attr");
        assert!(json0.get("TimeGenerated").is_some());
        assert_eq!(json0.as_object().unwrap().len(), 2);

        // Record 2: overridden reset -> base column reappears next to the attr.
        let json1: Value = serde_json::from_slice(&result[1]).unwrap();
        assert_eq!(json1["Shared"], "from-resource");
        assert_eq!(json1["other"], "val");
        assert!(json1.get("TimeGenerated").is_some());
        assert_eq!(json1.as_object().unwrap().len(), 3);
    }

    #[test]
    fn test_passthrough_attribute_overrides_resource_column() {
        // Innermost wins: a log attribute whose key equals a mapped resource
        // column overrides it, and no duplicate key is emitted.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::from([("service.name".into(), "Shared".into())]),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("from-resource".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "Shared".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("from-attr".into())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Shared"], "from-attr");
        // Injected TimeGenerated is the only other column.
        assert!(json.get("TimeGenerated").is_some());
        assert_eq!(json.as_object().unwrap().len(), 2);
    }

    #[test]
    fn test_passthrough_attribute_overrides_scope_column() {
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::from([("scope.name".into(), "Shared".into())]),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "s".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("from-scope".into())),
                            }),
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "Shared".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("from-attr".into())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Shared"], "from-attr");
        // Injected TimeGenerated is the only other column.
        assert!(json.get("TimeGenerated").is_some());
        assert_eq!(json.as_object().unwrap().len(), 2);
    }

    #[test]
    fn test_passthrough_attribute_overrides_field_column() {
        // A log attribute whose key equals a mapped top-level field column wins.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([
                ("body".into(), json!("Body")),
                ("attributes".into(), json!("passthrough")),
            ]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("from-field".into())),
                        }),
                        attributes: vec![KeyValue {
                            key: "Body".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("from-attr".into())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Body"], "from-attr");
        // Injected TimeGenerated is the only other column.
        assert!(json.get("TimeGenerated").is_some());
        assert_eq!(json.as_object().unwrap().len(), 2);
    }

    #[test]
    fn test_passthrough_multiple_collisions_and_survivor() {
        // Multiple attributes override mapped columns at once, a mapped field is
        // overridden, a non-colliding attribute is emitted, and a non-overridden
        // resource column survives.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::from([
                ("service.name".into(), "Svc".into()),
                ("host.name".into(), "Keep".into()),
            ]),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([
                ("body".into(), json!("Body")),
                ("attributes".into(), json!("passthrough")),
            ]),
        };
        let transformer = Transformer::new(&config);

        let str_kv = |k: &str, v: &str| KeyValue {
            key: k.into(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue(v.into())),
            }),
            ..Default::default()
        };
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        str_kv("service.name", "res-svc"),
                        str_kv("host.name", "res-host"),
                    ],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("res-body".into())),
                        }),
                        attributes: vec![
                            str_kv("Svc", "attr-svc"),   // overrides resource column
                            str_kv("Body", "attr-body"), // overrides field column
                            str_kv("other", "z"),        // no collision
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Svc"], "attr-svc"); // attribute wins over resource
        assert_eq!(json["Body"], "attr-body"); // attribute wins over field
        assert_eq!(json["other"], "z"); // non-colliding attribute
        assert_eq!(json["Keep"], "res-host"); // surviving resource column
        assert!(json.get("TimeGenerated").is_some()); // injected
        assert_eq!(json.as_object().unwrap().len(), 5);
    }

    #[test]
    fn test_passthrough_same_length_key_no_false_collision() {
        // An attribute key with the same length as a mapped column but different
        // bytes must not be treated as a collision (length bitmask passes, byte
        // comparison rejects). Both columns are emitted.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([
                ("body".into(), json!("Data")), // 4-byte column name
                ("attributes".into(), json!("passthrough")),
            ]),
        };
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("body-val".into())),
                        }),
                        attributes: vec![KeyValue {
                            key: "meta".into(), // same length (4) as "Data", different bytes
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("meta-val".into())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Data"], "body-val"); // mapped field survives
        assert_eq!(json["meta"], "meta-val"); // attribute emitted
        assert!(json.get("TimeGenerated").is_some()); // injected
        assert_eq!(json.as_object().unwrap().len(), 3);
    }

    #[test]
    fn test_passthrough_value_types() {
        // Passthrough preserves value types (string, int, double, bool).
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);

        let kv = |k: &str, v: OtelAnyValueEnum| KeyValue {
            key: k.into(),
            value: Some(AnyValue { value: Some(v) }),
            ..Default::default()
        };
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            kv("s", OtelAnyValueEnum::StringValue("txt".into())),
                            kv("i", OtelAnyValueEnum::IntValue(-7)),
                            kv("d", OtelAnyValueEnum::DoubleValue(1.5)),
                            kv("b", OtelAnyValueEnum::BoolValue(true)),
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["s"], "txt");
        assert_eq!(json["i"], -7);
        assert_eq!(json["d"], 1.5);
        assert_eq!(json["b"], true);
    }

    #[test]
    fn test_all_log_record_fields() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("time_unix_nano".to_string(), json!("Time")),
            ("observed_time_unix_nano".to_string(), json!("ObservedTime")),
            ("trace_id".to_string(), json!("TraceId")),
            ("span_id".to_string(), json!("SpanId")),
            ("flags".to_string(), json!("Flags")),
            ("severity_number".to_string(), json!("SeverityNum")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000_000_000,
                        observed_time_unix_nano: 2_000_000_000,
                        // trace_id must be 16 bytes
                        trace_id: vec![
                            0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x01,
                        ],
                        // span_id must be 8 bytes
                        span_id: vec![0xAB, 0xCD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
                        flags: 1,
                        severity_number: 9,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);

        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();

        assert!(json["Time"].as_str().unwrap().contains("1970"));
        assert!(json["ObservedTime"].as_str().unwrap().contains("1970"));
        assert_eq!(json["TraceId"], "ff000000000000000000000000000001");
        assert_eq!(json["SpanId"], "abcd000000000001");
        assert_eq!(json["Flags"], 1);
        assert_eq!(json["SeverityNum"], 9);
    }

    #[test]
    fn test_any_value_types() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::ArrayValue(ArrayValue {
                                values: vec![
                                    AnyValue {
                                        value: Some(OtelAnyValueEnum::DoubleValue(4.14)),
                                    },
                                    AnyValue {
                                        value: Some(OtelAnyValueEnum::BytesValue(vec![0xDE, 0xAD])),
                                    },
                                ],
                            })),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Body"], "[4.14, dead]");
    }

    #[test]
    fn test_kvlist_value() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::KvlistValue(KeyValueList {
                                values: vec![KeyValue {
                                    key: "nested".into(),
                                    value: Some(AnyValue {
                                        value: Some(OtelAnyValueEnum::StringValue("value".into())),
                                    }),
                                    ..Default::default()
                                }],
                            })),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert!(json["Body"].as_str().unwrap().contains("nested"));
    }

    #[test]
    fn test_empty_values() {
        let mut config = create_test_config();
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("trace_id".into(), json!("TraceId"));
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("span_id".into(), json!("SpanId"));
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("body".into(), json!("Body"));

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        trace_id: vec![],
                        span_id: vec![],
                        body: None,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["TraceId"], json!(null));
        assert_eq!(json["SpanId"], json!(null));
        assert_eq!(json["Body"], json!(null));
    }

    #[test]
    fn test_zero_timestamp() {
        let mut out = Vec::new();
        Transformer::write_timestamp_json(0, &mut out);
        let s = String::from_utf8(out).unwrap();
        // Should produce a quoted RFC3339 timestamp with 'T' separator
        assert!(s.starts_with('"'));
        assert!(s.ends_with('"'));
        assert!(s.contains('T'));
    }

    #[test]
    fn test_try_new_success() {
        let config = create_test_config();
        let result = Transformer::try_new(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_new_invalid_field() {
        let mut config = create_test_config();
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("invalid_field".into(), json!("Invalid"));

        let result = Transformer::try_new(&config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown log record field")
        );
    }

    #[test]
    fn test_try_new_invalid_mapping_type() {
        let mut config = create_test_config();
        // Field mapping value must be a string, not an object
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("body".into(), json!({"nested": "object"}));

        let result = Transformer::try_new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a string"));
    }

    #[test]
    fn test_empty_schema_mappings() {
        use super::super::config::{ApiConfig, HeartbeatConfig, SchemaConfig};

        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".into(),
                stream_name: "test-stream".into(),
                dcr: "test-dcr".into(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::new(),
                    scope_mapping: HashMap::new(),
                    log_record_mapping: HashMap::new(),
                },
                azure_monitor_source_resourceid: None,
                gzip_compression_level: 6,
                user_agent: None,
            },
            heartbeat: HeartbeatConfig::default(),
        };

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "unmapped.attr".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("value".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_700_000_000_000_000_000,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // No mappings configured, but the mandatory TimeGenerated is always
        // injected from the record's event time.
        assert_eq!(
            json,
            json!({ "TimeGenerated": "2023-11-14T22:13:20+00:00" })
        );
    }

    // ── TimeGenerated auto-injection ───────────────────────────

    /// Builds a single-record passthrough request with the given timestamps and
    /// attributes, and returns the emitted JSON object.
    fn passthrough_row(
        time_unix_nano: u64,
        observed_time_unix_nano: u64,
        attrs: Vec<KeyValue>,
    ) -> Value {
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("attributes".into(), json!("passthrough"))]),
        };
        let transformer = Transformer::new(&config);
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano,
                        observed_time_unix_nano,
                        attributes: attrs,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        assert_eq!(result.len(), 1);
        serde_json::from_slice(&result[0]).unwrap()
    }

    #[test]
    fn test_time_generated_injected_from_event_time() {
        // Unmapped TimeGenerated is injected from time_unix_nano (event time).
        let json = passthrough_row(1_700_000_000_000_000_000, 0, vec![]);
        assert_eq!(json["TimeGenerated"], "2023-11-14T22:13:20+00:00");
    }

    #[test]
    fn test_time_generated_falls_back_to_observed_time() {
        // time_unix_nano == 0 -> fall back to observed_time_unix_nano.
        let json = passthrough_row(0, 1_600_000_000_000_000_000, vec![]);
        assert_eq!(json["TimeGenerated"], "2020-09-13T12:26:40+00:00");
    }

    #[test]
    fn test_passthrough_attribute_named_time_generated_wins() {
        // A passthrough attribute literally named `TimeGenerated` wins over the
        // injected default (innermost-wins), and is emitted exactly once.
        let attrs = vec![KeyValue {
            key: "TimeGenerated".into(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("2001-01-01T00:00:00Z".into())),
            }),
            ..Default::default()
        }];
        let json = passthrough_row(1_700_000_000_000_000_000, 0, attrs);
        assert_eq!(json["TimeGenerated"], "2001-01-01T00:00:00Z");
        assert_eq!(json.as_object().unwrap().len(), 1);
    }

    #[test]
    fn test_explicit_time_generated_mapping_not_double_injected() {
        // Mapping a field to TimeGenerated disables injection: exactly one
        // TimeGenerated, carrying the mapped field's value.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::new(),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::from([("time_unix_nano".into(), json!("TimeGenerated"))]),
        };
        let transformer = Transformer::new(&config);
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_700_000_000_000_000_000,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["TimeGenerated"], "2023-11-14T22:13:20+00:00");
        assert_eq!(json.as_object().unwrap().len(), 1);
    }

    #[test]
    fn test_resource_mapping_to_time_generated_disables_injection() {
        // A resource attribute mapped to TimeGenerated also disables injection.
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping: HashMap::from([("host.time".into(), "TimeGenerated".into())]),
            scope_mapping: HashMap::new(),
            log_record_mapping: HashMap::new(),
        };
        let transformer = Transformer::new(&config);
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "host.time".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue(
                                "2005-05-05T05:05:05Z".into(),
                            )),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        // Event time present, but must be ignored: TimeGenerated
                        // is explicitly mapped from the resource attribute.
                        time_unix_nano: 1_700_000_000_000_000_000,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["TimeGenerated"], "2005-05-05T05:05:05Z");
        assert_eq!(json.as_object().unwrap().len(), 1);
    }

    #[test]
    fn test_attribute_with_no_value() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: None, // No value
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: None, // No value
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: None, // No value
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // Attributes with no value should be skipped
        assert!(json.get("ServiceName").is_none());
        assert!(json.get("ScopeName").is_none());
        assert!(json.get("TestAttr").is_none());
    }

    #[test]
    fn test_multiple_log_records() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("first".into())),
                            }),
                            severity_text: "INFO".into(),
                            ..Default::default()
                        },
                        LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("second".into())),
                            }),
                            severity_text: "ERROR".into(),
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 2);

        let json1: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json1["Body"], "first");
        assert_eq!(json1["Severity"], "INFO");

        let json2: Value = serde_json::from_slice(&result[1]).unwrap();
        assert_eq!(json2["Body"], "second");
        assert_eq!(json2["Severity"], "ERROR");
    }

    #[test]
    fn test_empty_body_string() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([("body".into(), json!("Body"))]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue(String::new())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Body"], "");
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(Transformer::bytes_to_hex(&[]), "");
        assert_eq!(Transformer::bytes_to_hex(&[0x00]), "00");
        assert_eq!(Transformer::bytes_to_hex(&[0xff]), "ff");
        assert_eq!(
            Transformer::bytes_to_hex(&[0x12, 0x34, 0xab, 0xcd]),
            "1234abcd"
        );
    }

    #[test]
    fn test_case_insensitive_field_names() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("TIME_UNIX_NANO".into(), json!("Time")),
            ("Body".into(), json!("Body")),
            ("SEVERITY_TEXT".into(), json!("Severity")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000_000_000,
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("test".into())),
                        }),
                        severity_text: "WARN".into(),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert!(json["Time"].as_str().is_some());
        assert_eq!(json["Body"], "test");
        assert_eq!(json["Severity"], "WARN");
    }

    #[test]
    fn test_double_nan_becomes_null() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::DoubleValue(f64::NAN)),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord::default()],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // NaN cannot be represented in JSON, so it becomes null
        assert_eq!(json["ServiceName"], json!(null));
    }

    #[test]
    fn test_event_name_field() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("event_name".into(), json!("EventName")),
            ("severity_text".into(), json!("Severity")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            event_name: "user.login".into(),
                            severity_text: "INFO".into(),
                            ..Default::default()
                        },
                        LogRecord {
                            event_name: String::new(),
                            severity_text: String::new(),
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 2);

        let json1: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json1["EventName"], "user.login");
        assert_eq!(json1["Severity"], "INFO");

        let json2: Value = serde_json::from_slice(&result[1]).unwrap();
        assert_eq!(json2["EventName"], "");
        assert_eq!(json2["Severity"], "");
    }

    /// Round-trip test: every record produced by the transformer must be valid JSON
    /// that can be deserialized back into a serde_json::Value and re-serialized
    /// to produce identical output.
    #[test]
    fn test_roundtrip_json_validity() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("my-scope".into())),
                            }),
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("hello world".into())),
                        }),
                        severity_text: "INFO".into(),
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(true)),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);

        for (i, result) in results.iter().enumerate() {
            // Must parse as valid JSON
            let parsed: Value = serde_json::from_slice(result).unwrap_or_else(|e| {
                panic!(
                    "record {i} is not valid JSON: {e}\nraw: {}",
                    String::from_utf8_lossy(result)
                )
            });

            // Re-serialize and re-parse must produce identical value
            let re_serialized = serde_json::to_vec(&parsed).unwrap();
            let re_parsed: Value = serde_json::from_slice(&re_serialized).unwrap();
            assert_eq!(parsed, re_parsed, "round-trip mismatch for record {i}");
        }
    }

    /// Test that UTF-8 multi-byte strings (emoji, CJK, etc.) survive the
    /// direct JSON serialization without corruption.
    #[test]
    fn test_utf8_multibyte_strings() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("body".into(), json!("Body")),
            ("severity_text".into(), json!("Sev")),
        ]);

        let transformer = Transformer::new(&config);

        let test_strings = vec![
            "hello world",                  // ASCII
            "héllo wörld",                  // Latin diacritics
            "日本語テスト",                 // CJK
            "🚀🔥💯",                       // Emoji
            "mixed: café ☕ naïve 日本 🎉", // Mixed scripts
            "line1\nline2\ttab",            // Escape sequences
            "quote\"and\\backslash",        // JSON-special chars
            "\x01\x02\x1f",                 // Control characters
            "",                             // Empty string
        ];

        for test_str in &test_strings {
            let request = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource: None,
                    scope_logs: vec![ScopeLogs {
                        scope: None,
                        log_records: vec![LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue(test_str.to_string())),
                            }),
                            severity_text: test_str.to_string(),
                            ..Default::default()
                        }],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            };

            let bytes = request.encode_to_vec();
            let logs_view = RawLogsData::new(&bytes);
            let results = transformer.convert_to_log_analytics(&logs_view);

            assert_eq!(
                results.len(),
                1,
                "expected 1 result for input: {test_str:?}"
            );

            let json: Value = serde_json::from_slice(&results[0]).unwrap_or_else(|e| {
                panic!(
                    "invalid JSON for input {test_str:?}: {e}\nraw bytes: {:?}",
                    results[0]
                )
            });

            // Verify the body value round-trips correctly
            let body = json["Body"].as_str().unwrap_or_else(|| {
                panic!("Body field missing or not a string for input: {test_str:?}")
            });
            assert_eq!(body, *test_str, "body mismatch for input: {test_str:?}");

            // Verify severity_text also round-trips
            let sev = json["Sev"].as_str().unwrap();
            assert_eq!(sev, *test_str, "severity mismatch for input: {test_str:?}");
        }
    }

    /// Test that write_json_string produces output identical to serde_json
    /// for various edge-case strings.
    #[test]
    fn test_write_json_string_matches_serde() {
        let cases = vec![
            "",
            "simple",
            "with \"quotes\"",
            "back\\slash",
            "new\nline",
            "tab\there",
            "carriage\rreturn",
            "\x00\x01\x1f", // control chars
            "emoji 🎉 and ñ",
            "日本語",
            "mixed\t\"escape\"\n🚀",
        ];

        for input in &cases {
            let mut our_output = Vec::new();
            Transformer::write_json_string(input.as_bytes(), &mut our_output);

            let serde_output = serde_json::to_vec(input).unwrap();

            assert_eq!(
                our_output,
                serde_output,
                "mismatch for input {input:?}:\n  ours:  {}\n  serde: {}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }
    }

    /// Test numeric formatting matches serde_json output.
    #[test]
    fn test_numeric_formatting_matches_serde() {
        // Integers via itoa
        let int_cases: Vec<i64> = vec![0, 1, -1, 42, -42, i64::MAX, i64::MIN];
        for n in &int_cases {
            let mut our_output = Vec::new();
            Transformer::write_i64(*n, &mut our_output);
            let serde_output = serde_json::to_vec(n).unwrap();
            assert_eq!(
                our_output,
                serde_output,
                "i64 mismatch for {n}: ours={}, serde={}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }

        // Unsigned via itoa
        let uint_cases: Vec<u64> = vec![0, 1, 42, u64::MAX];
        for n in &uint_cases {
            let mut our_output = Vec::new();
            Transformer::write_u64(*n, &mut our_output);
            let serde_output = serde_json::to_vec(n).unwrap();
            assert_eq!(
                our_output,
                serde_output,
                "u64 mismatch for {n}: ours={}, serde={}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }
    }

    /// Test that attribute values with type mismatch (e.g. value_type says Int64
    /// but as_int64() returns None) produce null instead of silently defaulting
    /// to 0, false, or empty string. This covers the null-safe extraction in
    /// write_any_value_json.
    #[test]
    fn test_null_safe_attribute_value_extraction() {
        // Test each value type mapped as a log record attribute
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([(
            "attributes".into(),
            json!({
                "str_attr": "StrCol",
                "int_attr": "IntCol",
                "dbl_attr": "DblCol",
                "bool_attr": "BoolCol",
                "bytes_attr": "BytesCol",
            }),
        )]);

        let transformer = Transformer::new(&config);

        // Normal case: all attributes present with valid values
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            KeyValue {
                                key: "str_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("hello".into())),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "int_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::IntValue(42)),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "dbl_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(2.72)),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "bool_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::BoolValue(true)),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "bytes_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::BytesValue(vec![0xCA, 0xFE])),
                                }),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(json["StrCol"], "hello");
        assert_eq!(json["IntCol"], 42);
        assert_eq!(json["DblCol"], 2.72);
        assert_eq!(json["BoolCol"], true);
        assert_eq!(json["BytesCol"], "cafe");

        // NaN/Infinity doubles become null
        let request_nan = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            KeyValue {
                                key: "dbl_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(f64::NAN)),
                                }),
                                ..Default::default()
                            },
                            KeyValue {
                                key: "int_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(f64::INFINITY)),
                                }),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request_nan.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(json["DblCol"], json!(null), "NaN must become null");
        assert_eq!(json["IntCol"], json!(null), "Infinity must become null");

        // Boolean false is a real value, not null
        let request_false = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "bool_attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(false)),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request_false.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(
            json["BoolCol"], false,
            "false must be serialized as false, not null"
        );
    }

    /// Test that invalid UTF-8 bytes in a body string produce valid JSON
    /// (using lossy replacement) instead of invalid JSON output.
    #[test]
    fn test_invalid_utf8_body_produces_valid_json() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([("body".into(), json!("Body"))]);

        let transformer = Transformer::new(&config);

        // Construct a protobuf request with invalid UTF-8 in the body.
        // Protobuf string fields are nominally UTF-8, but in practice
        // the bytes can be anything when transported via byte slices.
        let mut request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue(String::new())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        // Encode, then patch the body bytes with invalid UTF-8.
        // We'll set a known placeholder first, then swap the raw bytes.
        request.resource_logs[0].scope_logs[0].log_records[0].body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("PLACEHOLDER".into())),
        });
        let mut encoded = request.encode_to_vec();

        // Find the placeholder bytes and replace with invalid UTF-8 sequence.
        // \x80\xFF are not valid UTF-8 start bytes.
        let placeholder = b"PLACEHOLDER";
        if let Some(pos) = encoded
            .windows(placeholder.len())
            .position(|w| w == placeholder)
        {
            // Replace first 3 bytes of PLACEHOLDER with invalid UTF-8
            encoded[pos] = 0x80;
            encoded[pos + 1] = 0xFF;
            encoded[pos + 2] = 0xFE;
            // Keep remaining bytes as valid ASCII filler (same length = no protobuf reframing)
            encoded[pos + 3] = b'o';
            encoded[pos + 4] = b'k';
            encoded[pos + 5] = b'_';
            encoded[pos + 6] = b't';
            encoded[pos + 7] = b'a';
            encoded[pos + 8] = b'i';
            encoded[pos + 9] = b'l';
            encoded[pos + 10] = b'!';
        }

        let logs_view = RawLogsData::new(&encoded);
        let results = transformer.convert_to_log_analytics(&logs_view);
        assert_eq!(results.len(), 1, "expected one log record");

        // The output MUST be valid JSON — serde_json::from_slice must not fail.
        let json: Value = serde_json::from_slice(&results[0]).unwrap_or_else(|e| {
            panic!(
                "invalid JSON from body with invalid UTF-8: {e}\nraw: {:?}",
                String::from_utf8_lossy(&results[0])
            )
        });

        // The body value must be a string (not null) and must contain the
        // Unicode replacement character U+FFFD for the invalid bytes.
        let body = json["Body"].as_str().expect("Body should be a JSON string");
        assert!(
            body.contains('\u{FFFD}'),
            "expected replacement character in body, got: {body:?}"
        );
    }
}
