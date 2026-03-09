// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Payload type definitions from the OTAP spec.
//!
//! Each payload type has a definition that describes the columns it contains,
//! their native Arrow types, and dictionary encoding constraints.
//!
//! See docs/otap-spec.md sections 5.1-5.4 for the full specification.

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Native Arrow types from the OTAP spec.
///
/// This is a const-constructible alternative to `arrow::datatypes::DataType`
/// which uses Arc/Box for complex types like Struct and List.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeType {
    Boolean,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float64,
    Utf8,
    Binary,
    FixedSizeBinary(i32),
    /// Timestamp(Nanosecond, None)
    TimestampNs,
    /// Duration(Nanosecond)
    DurationNs,
    /// Struct (sub-fields defined via dotted paths in the same definition)
    Struct,
    /// List(Struct { ... })
    ListStruct,
    /// List(UInt64)
    ListUInt64,
    /// List(Float64)
    ListFloat64,
}

/// Minimum dictionary key size constraint from the OTAP spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinDictKeySize {
    U8,
    U16,
}

/// Column definition from the OTAP spec.
#[derive(Debug, Clone, Copy)]
pub struct ColumnDef {
    /// The native (non-dictionary) Arrow type for this column.
    pub native_type: NativeType,
    /// Minimum dictionary key size if dictionary encoding is allowed.
    /// None if the column does not support dictionary encoding.
    pub min_dict_key_size: Option<MinDictKeySize>,
}

/// Schema definition for a payload type.
///
/// Columns are stored as a sorted flat list of `(dotted_path, column_def)`
/// pairs where dotted_path uses the spec convention: `"body"` for a top-level
/// column, `"body.int"` for a sub-field of the `body` struct.
pub struct PayloadDefinition {
    pub columns: &'static [(&'static str, ColumnDef)],
}

impl PayloadDefinition {
    /// An empty definition with no columns. When used with `select_dictionary_type`,
    /// all dictionary columns will be converted to their native type since no column
    /// will be found in the definition.
    pub const EMPTY: PayloadDefinition = PayloadDefinition { columns: &[] };

    /// Look up a column by name. Returns `None` if the column is not present
    /// in the definition.
    pub fn get(&self, name: &str) -> Option<&ColumnDef> {
        self.columns
            .binary_search_by_key(&name, |(n, _)| n)
            .ok()
            .map(|idx| &self.columns[idx].1)
    }

    /// Look up a nested column without allocating. E.g., `get_nested("body",
    /// "int")` looks up `"body.int"` in the definition.
    pub fn get_nested(&self, parent: &str, child: &str) -> Option<&ColumnDef> {
        self.columns
            .binary_search_by(|(name, _)| cmp_dotted(name, parent, child))
            .ok()
            .map(|idx| &self.columns[idx].1)
    }

    /// Returns the full sorted list of (path, column_def) pairs.
    pub fn columns(&self) -> &'static [(&'static str, ColumnDef)] {
        self.columns
    }
}

/// Compare `name` against the virtual string `"{parent}.{child}"` without
/// allocating.
fn cmp_dotted(name: &str, parent: &str, child: &str) -> std::cmp::Ordering {
    let name_bytes = name.as_bytes();
    let expected_len = parent.len() + 1 + child.len();

    // Compare parent prefix
    let parent_bytes = parent.as_bytes();
    let cmp_len = name_bytes.len().min(parent_bytes.len());
    let ord = name_bytes[..cmp_len].cmp(parent_bytes);
    if ord != std::cmp::Ordering::Equal {
        return ord;
    }
    if name_bytes.len() <= parent_bytes.len() {
        // name is shorter than or equal to parent, so name < "parent.child"
        return if name_bytes.len() == parent_bytes.len() {
            // name == parent exactly, but we need "parent.child" which is longer
            std::cmp::Ordering::Less
        } else {
            name_bytes.len().cmp(&expected_len)
        };
    }

    // Compare the dot separator
    let dot_ord = name_bytes[parent.len()].cmp(&b'.');
    if dot_ord != std::cmp::Ordering::Equal {
        return dot_ord;
    }

    // Compare child suffix
    let name_suffix = &name_bytes[parent.len() + 1..];
    let child_bytes = child.as_bytes();
    let suffix_cmp_len = name_suffix.len().min(child_bytes.len());
    let ord = name_suffix[..suffix_cmp_len].cmp(&child_bytes[..suffix_cmp_len]);
    if ord != std::cmp::Ordering::Equal {
        return ord;
    }

    name_bytes.len().cmp(&expected_len)
}

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/// Get the payload definition for a given payload type.
pub fn get_definition(typ: ArrowPayloadType) -> &'static PayloadDefinition {
    match typ {
        ArrowPayloadType::Unknown => &EMPTY_DEFINITION,

        // Logs
        ArrowPayloadType::Logs => &LOGS_DEFINITION,
        ArrowPayloadType::LogAttrs => &U16_ATTRS_DEFINITION,

        // Traces
        ArrowPayloadType::Spans => &SPANS_DEFINITION,
        ArrowPayloadType::SpanAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::SpanEvents => &SPAN_EVENTS_DEFINITION,
        ArrowPayloadType::SpanLinks => &SPAN_LINKS_DEFINITION,
        ArrowPayloadType::SpanEventAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::SpanLinkAttrs => &U32_ATTRS_DEFINITION,

        // Metrics
        ArrowPayloadType::UnivariateMetrics => &UNIVARIATE_METRICS_DEFINITION,
        ArrowPayloadType::MultivariateMetrics => &EMPTY_DEFINITION,
        ArrowPayloadType::NumberDataPoints => &NUMBER_DATA_POINTS_DEFINITION,
        ArrowPayloadType::SummaryDataPoints => &SUMMARY_DATA_POINTS_DEFINITION,
        ArrowPayloadType::HistogramDataPoints => &HISTOGRAM_DATA_POINTS_DEFINITION,
        ArrowPayloadType::ExpHistogramDataPoints => &EXP_HISTOGRAM_DATA_POINTS_DEFINITION,
        ArrowPayloadType::NumberDpExemplars => &EXEMPLARS_DEFINITION,
        ArrowPayloadType::HistogramDpExemplars => &EXEMPLARS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplars => &EXEMPLARS_DEFINITION,

        // Metric attributes
        ArrowPayloadType::MetricAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::NumberDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::SummaryDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::HistogramDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::NumberDpExemplarAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::HistogramDpExemplarAttrs => &U32_ATTRS_DEFINITION,
        ArrowPayloadType::ExpHistogramDpExemplarAttrs => &U32_ATTRS_DEFINITION,

        // Common
        ArrowPayloadType::ResourceAttrs => &U16_ATTRS_DEFINITION,
        ArrowPayloadType::ScopeAttrs => &U16_ATTRS_DEFINITION,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const fn col(native_type: NativeType) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: None,
    }
}

const fn dict(native_type: NativeType, min: MinDictKeySize) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: Some(min),
    }
}

const U8: MinDictKeySize = MinDictKeySize::U8;
const U16: MinDictKeySize = MinDictKeySize::U16;

// ---------------------------------------------------------------------------
// Empty definition
// ---------------------------------------------------------------------------

static EMPTY_DEFINITION: PayloadDefinition = PayloadDefinition { columns: &[] };

// ---------------------------------------------------------------------------
// U16 Attributes (RESOURCE_ATTRS, SCOPE_ATTRS, LOG_ATTRS, METRIC_ATTRS,
//                 SPAN_ATTRS)
// Spec: 5.4.2
// ---------------------------------------------------------------------------

static U16_ATTRS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("bool", col(NativeType::Boolean)),
        ("bytes", dict(NativeType::Binary, U16)),
        ("double", col(NativeType::Float64)),
        ("int", dict(NativeType::Int64, U16)),
        ("key", dict(NativeType::Utf8, U8)),
        ("parent_id", col(NativeType::UInt16)),
        ("ser", dict(NativeType::Binary, U16)),
        ("str", dict(NativeType::Utf8, U16)),
        ("type", col(NativeType::UInt8)),
    ],
};

// ---------------------------------------------------------------------------
// U32 Attributes (SPAN_EVENT_ATTRS, SPAN_LINK_ATTRS, all DP attrs,
//                 all exemplar attrs)
// Spec: 5.4.1
// ---------------------------------------------------------------------------

static U32_ATTRS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("bool", col(NativeType::Boolean)),
        ("bytes", dict(NativeType::Binary, U16)),
        ("double", col(NativeType::Float64)),
        ("int", dict(NativeType::Int64, U16)),
        ("key", dict(NativeType::Utf8, U8)),
        ("parent_id", dict(NativeType::UInt32, U8)),
        ("ser", dict(NativeType::Binary, U16)),
        ("str", dict(NativeType::Utf8, U16)),
        ("type", col(NativeType::UInt8)),
    ],
};

// ---------------------------------------------------------------------------
// LOGS (ROOT) - Spec: 5.1.1
// ---------------------------------------------------------------------------

static LOGS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically by dotted path
        ("body", col(NativeType::Struct)),
        ("body.bool", col(NativeType::Boolean)),
        ("body.bytes", dict(NativeType::Binary, U16)),
        ("body.double", col(NativeType::Float64)),
        ("body.int", dict(NativeType::Int64, U16)),
        ("body.str", dict(NativeType::Utf8, U16)),
        ("body.type", col(NativeType::UInt8)),
        ("body_ser", dict(NativeType::Binary, U16)),
        ("dropped_attributes_count", col(NativeType::UInt32)),
        ("event_name", dict(NativeType::Utf8, U8)),
        ("flags", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt16)),
        ("observed_time_unix_nano", col(NativeType::TimestampNs)),
        ("resource", col(NativeType::Struct)),
        ("resource.dropped_attributes_count", col(NativeType::UInt32)),
        ("resource.id", col(NativeType::UInt16)),
        ("resource.schema_url", dict(NativeType::Utf8, U8)),
        ("schema_url", dict(NativeType::Utf8, U8)),
        ("scope", col(NativeType::Struct)),
        ("scope.dropped_attributes_count", col(NativeType::UInt32)),
        ("scope.id", col(NativeType::UInt16)),
        ("scope.name", dict(NativeType::Utf8, U8)),
        ("scope.version", dict(NativeType::Utf8, U8)),
        ("severity_number", dict(NativeType::Int32, U8)),
        ("severity_text", dict(NativeType::Utf8, U8)),
        ("span_id", dict(NativeType::FixedSizeBinary(8), U8)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
        ("trace_id", dict(NativeType::FixedSizeBinary(16), U8)),
    ],
};

// ---------------------------------------------------------------------------
// SPANS (ROOT) - Spec: 5.2.1
// ---------------------------------------------------------------------------

static SPANS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically by dotted path
        ("dropped_attributes_count", col(NativeType::UInt32)),
        ("dropped_events_count", col(NativeType::UInt32)),
        ("dropped_links_count", col(NativeType::UInt32)),
        ("duration_time_unix_nano", col(NativeType::DurationNs)),
        ("id", col(NativeType::UInt16)),
        ("kind", col(NativeType::Int32)),
        ("name", col(NativeType::Utf8)),
        ("parent_span_id", col(NativeType::FixedSizeBinary(8))),
        ("resource", col(NativeType::Struct)),
        ("resource.dropped_attributes_count", col(NativeType::UInt32)),
        ("resource.id", col(NativeType::UInt16)),
        ("resource.schema_url", dict(NativeType::Utf8, U8)),
        ("schema_url", col(NativeType::Utf8)),
        ("scope", col(NativeType::Struct)),
        ("scope.dropped_attributes_count", col(NativeType::UInt32)),
        ("scope.id", col(NativeType::UInt16)),
        ("scope.name", col(NativeType::Utf8)),
        ("scope.version", col(NativeType::Utf8)),
        ("span_id", col(NativeType::FixedSizeBinary(8))),
        ("start_time_unix_nano", col(NativeType::TimestampNs)),
        ("status", col(NativeType::Struct)),
        ("status.code", dict(NativeType::Int32, U8)),
        ("status.status_message", dict(NativeType::Utf8, U8)),
        ("trace_id", col(NativeType::FixedSizeBinary(16))),
        ("trace_state", col(NativeType::Utf8)),
    ],
};

// ---------------------------------------------------------------------------
// SPAN_EVENTS - Spec: 5.2.2
// ---------------------------------------------------------------------------

static SPAN_EVENTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("dropped_attributes_count", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("name", col(NativeType::Utf8)),
        ("parent_id", col(NativeType::UInt16)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
    ],
};

// ---------------------------------------------------------------------------
// SPAN_LINKS - Spec: 5.2.3
// ---------------------------------------------------------------------------

static SPAN_LINKS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("dropped_attributes_count", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("parent_id", col(NativeType::UInt16)),
        ("span_id", col(NativeType::FixedSizeBinary(8))),
        ("trace_id", col(NativeType::FixedSizeBinary(16))),
        ("trace_state", col(NativeType::Utf8)),
    ],
};

// ---------------------------------------------------------------------------
// UNIVARIATE_METRICS (ROOT) - Spec: 5.3.1
// ---------------------------------------------------------------------------

static UNIVARIATE_METRICS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically by dotted path
        ("aggregation_temporality", col(NativeType::Int32)),
        ("description", col(NativeType::Utf8)),
        ("id", col(NativeType::UInt16)),
        ("is_monotonic", col(NativeType::Boolean)),
        ("metric_type", col(NativeType::UInt8)),
        ("name", col(NativeType::Utf8)),
        ("resource", col(NativeType::Struct)),
        ("resource.dropped_attributes_count", col(NativeType::UInt32)),
        ("resource.id", col(NativeType::UInt16)),
        ("resource.schema_url", col(NativeType::Utf8)),
        ("schema_url", col(NativeType::Utf8)),
        ("scope", col(NativeType::Struct)),
        ("scope.dropped_attributes_count", col(NativeType::UInt32)),
        ("scope.id", col(NativeType::UInt16)),
        ("scope.name", col(NativeType::Utf8)),
        ("scope.version", col(NativeType::Utf8)),
        ("unit", col(NativeType::Utf8)),
    ],
};

// ---------------------------------------------------------------------------
// NUMBER_DATA_POINTS - Spec: 5.3.2
// ---------------------------------------------------------------------------

static NUMBER_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("double_value", col(NativeType::Float64)),
        ("flags", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("int_value", col(NativeType::Int64)),
        ("parent_id", col(NativeType::UInt16)),
        ("start_time_unix_nano", col(NativeType::TimestampNs)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
    ],
};

// ---------------------------------------------------------------------------
// SUMMARY_DATA_POINTS - Spec: 5.3.3
// ---------------------------------------------------------------------------

static SUMMARY_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("count", col(NativeType::UInt64)),
        ("flags", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("parent_id", col(NativeType::UInt16)),
        ("quantile", col(NativeType::ListStruct)),
        ("quantile.quantile", col(NativeType::Float64)),
        ("quantile.value", col(NativeType::Float64)),
        ("start_time_unix_nano", col(NativeType::TimestampNs)),
        ("sum", col(NativeType::Float64)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
        ("value", col(NativeType::ListFloat64)),
    ],
};

// ---------------------------------------------------------------------------
// HISTOGRAM_DATA_POINTS - Spec: 5.3.4
// ---------------------------------------------------------------------------

static HISTOGRAM_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("bucket_counts", col(NativeType::ListUInt64)),
        ("count", col(NativeType::UInt64)),
        ("explicit_bounds", col(NativeType::ListFloat64)),
        ("flags", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("max", col(NativeType::Float64)),
        ("min", col(NativeType::Float64)),
        ("parent_id", col(NativeType::UInt16)),
        ("start_time_unix_nano", col(NativeType::TimestampNs)),
        ("sum", col(NativeType::Float64)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
    ],
};

// ---------------------------------------------------------------------------
// EXP_HISTOGRAM_DATA_POINTS - Spec: 5.3.5
// ---------------------------------------------------------------------------

static EXP_HISTOGRAM_DATA_POINTS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("count", col(NativeType::UInt64)),
        ("flags", col(NativeType::UInt32)),
        ("id", col(NativeType::UInt32)),
        ("max", col(NativeType::Float64)),
        ("min", col(NativeType::Float64)),
        ("negative_bucket_counts", col(NativeType::ListUInt64)),
        ("negative_offset", col(NativeType::Int32)),
        ("parent_id", col(NativeType::UInt16)),
        ("positive_bucket_counts", col(NativeType::ListUInt64)),
        ("positive_offset", col(NativeType::Int32)),
        ("scale", col(NativeType::Int32)),
        ("start_time_unix_nano", col(NativeType::TimestampNs)),
        ("sum", col(NativeType::Float64)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
        ("zero_count", col(NativeType::UInt64)),
    ],
};

// ---------------------------------------------------------------------------
// EXEMPLARS (NUMBER_DP_EXEMPLARS, HISTOGRAM_DP_EXEMPLARS,
//            EXP_HISTOGRAM_DP_EXEMPLARS) - Spec: 5.3.6
// ---------------------------------------------------------------------------

static EXEMPLARS_DEFINITION: PayloadDefinition = PayloadDefinition {
    columns: &[
        // SORTED alphabetically
        ("double_value", col(NativeType::Float64)),
        ("id", col(NativeType::UInt32)),
        ("int_value", dict(NativeType::Int64, U8)),
        ("parent_id", dict(NativeType::UInt32, U8)),
        ("span_id", dict(NativeType::FixedSizeBinary(8), U8)),
        ("time_unix_nano", col(NativeType::TimestampNs)),
        ("trace_id", dict(NativeType::FixedSizeBinary(16), U8)),
    ],
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definitions_are_sorted() {
        // Verify all definitions have their columns sorted, which is required
        // for binary search to work correctly.
        let all_types = [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            ArrowPayloadType::MetricAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ];

        for typ in all_types {
            let def = get_definition(typ);
            for window in def.columns.windows(2) {
                assert!(
                    window[0].0 < window[1].0,
                    "Definition for {:?} is not sorted: {:?} >= {:?}",
                    typ,
                    window[0].0,
                    window[1].0,
                );
            }
        }
    }

    #[test]
    fn test_get_top_level_column() {
        let def = get_definition(ArrowPayloadType::LogAttrs);

        let str_col = def.get("str").unwrap();
        assert_eq!(str_col.native_type, NativeType::Utf8);
        assert_eq!(str_col.min_dict_key_size, Some(MinDictKeySize::U16));

        let key_col = def.get("key").unwrap();
        assert_eq!(key_col.native_type, NativeType::Utf8);
        assert_eq!(key_col.min_dict_key_size, Some(MinDictKeySize::U8));

        let type_col = def.get("type").unwrap();
        assert_eq!(type_col.native_type, NativeType::UInt8);
        assert_eq!(type_col.min_dict_key_size, None);

        assert!(def.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_nested_column() {
        let def = get_definition(ArrowPayloadType::Logs);

        let body_str = def.get_nested("body", "str").unwrap();
        assert_eq!(body_str.native_type, NativeType::Utf8);
        assert_eq!(body_str.min_dict_key_size, Some(MinDictKeySize::U16));

        let body_int = def.get_nested("body", "int").unwrap();
        assert_eq!(body_int.native_type, NativeType::Int64);
        assert_eq!(body_int.min_dict_key_size, Some(MinDictKeySize::U16));

        let resource_schema = def.get_nested("resource", "schema_url").unwrap();
        assert_eq!(resource_schema.native_type, NativeType::Utf8);
        assert_eq!(resource_schema.min_dict_key_size, Some(MinDictKeySize::U8));

        assert!(def.get_nested("body", "nonexistent").is_none());
        assert!(def.get_nested("nonexistent", "str").is_none());
    }

    #[test]
    fn test_u16_attrs_value_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::ResourceAttrs);
        for col_name in &["str", "int", "bytes", "ser"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_name,
            );
        }
    }

    #[test]
    fn test_u32_attrs_value_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::SpanEventAttrs);
        for col_name in &["str", "int", "bytes", "ser"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_name,
            );
        }
    }

    #[test]
    fn test_logs_body_columns_require_u16() {
        let def = get_definition(ArrowPayloadType::Logs);
        for col_path in &["body.str", "body.int", "body.bytes"] {
            let parts: Vec<&str> = col_path.split('.').collect();
            let col = def
                .get_nested(parts[0], parts[1])
                .unwrap_or_else(|| panic!("missing column: {}", col_path));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U16),
                "column {} should require Dict(u16)",
                col_path,
            );
        }
        // body_ser is a top-level column
        let body_ser = def.get("body_ser").unwrap();
        assert_eq!(body_ser.min_dict_key_size, Some(MinDictKeySize::U16));
    }

    #[test]
    fn test_exemplars_dict_columns() {
        let def = get_definition(ArrowPayloadType::NumberDpExemplars);
        for col_name in &["parent_id", "int_value", "span_id", "trace_id"] {
            let col = def
                .get(col_name)
                .unwrap_or_else(|| panic!("missing column: {}", col_name));
            assert_eq!(
                col.min_dict_key_size,
                Some(MinDictKeySize::U8),
                "exemplar column {} should allow Dict(u8)",
                col_name,
            );
        }
    }

    #[test]
    fn test_cmp_dotted() {
        use std::cmp::Ordering;

        // Exact match
        assert_eq!(cmp_dotted("body.str", "body", "str"), Ordering::Equal);
        assert_eq!(cmp_dotted("body.int", "body", "int"), Ordering::Equal);
        assert_eq!(
            cmp_dotted("resource.schema_url", "resource", "schema_url"),
            Ordering::Equal
        );

        // name < "parent.child"
        assert_eq!(cmp_dotted("body", "body", "str"), Ordering::Less);
        assert_eq!(cmp_dotted("aaa.zzz", "body", "str"), Ordering::Less);

        // name > "parent.child"
        assert_eq!(cmp_dotted("zzz", "body", "str"), Ordering::Greater);
        assert_eq!(cmp_dotted("body.zzz", "body", "str"), Ordering::Greater);

        // Prefix but different separator
        assert_eq!(cmp_dotted("body_ser", "body", "str"), Ordering::Greater);
    }
}
