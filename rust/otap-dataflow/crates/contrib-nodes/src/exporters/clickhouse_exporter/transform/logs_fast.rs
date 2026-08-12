// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Specialized transformation of canonical OTAP log batches into ClickHouse columns.
//!
//! # Applicability
//!
//! The exporter attempts this fast path only for log signals whose original format is
//! `SignalFormat::OtapRecords`. It applies when a `Logs` batch is present and its optional
//! `resource`, `scope`, and `body` columns use the canonical struct layout. Parent IDs must be
//! `UInt16` when the corresponding resource, scope, or log attribute payload is present. Missing
//! optional columns and attribute payloads do not prevent fast-path use.
//!
//! The fast path is not attempted for raw OTLP protobuf (`SignalFormat::OtlpBytes`), traces, or
//! metrics; those inputs use the generic transformation plan directly. If an OTAP logs batch is
//! eligible for an attempt but has an unsupported layout, this transformer returns
//! [`LogsFastTransform::NotApplicable`] and the caller falls back to the generic plan.
//!
//! # Approach
//!
//! The generic transformer interprets a reusable transformation plan that supports several
//! signals and input layouts. This transformer recognizes the canonical OTAP logs layout and
//! constructs the equivalent ClickHouse `RecordBatch` directly:
//!
//! - compatible Arrow arrays are shared with the output; only columns that require conversion or
//!   joining are materialized
//! - compact resource, scope, and log attribute payloads are joined to their parent log rows, with
//!   resource and scope joins using a dense ID-to-row lookup
//! - `service.name` is extracted while resource attributes are copied, avoiding another map scan
//! - output columns are sorted to match the generic transformer's stable lexical ordering
//! - the most recently used output schema is cached and reused for batches with the same layout
//!
//! # ClickHouse-compatible Arrow RecordBatch
//!
//! The output contains one row per input log row. Columns are assembled as follows (`LC` means
//! `LowCardinality`):
//!
//! ```text
//! +--------------------+--------------------------------------+--------------------------+
//! | Output column      | OTAP source / operation              | ClickHouse type          |
//! +--------------------+--------------------------------------+--------------------------+
//! | Body               | body -> stringify AnyValue           | String                   |
//! | EventName          | event_name -> reuse                  | String                   |
//! | LogAttributes      | LogAttrs joined on Logs.id           | Map(LC(String), String)  |
//! | ResourceAttributes | ResourceAttrs joined on resource.id  | Map(LC(String), String)  |
//! | ResourceSchemaUrl  | resource.schema_url -> reuse         | LC(String)               |
//! | ScopeAttributes    | ScopeAttrs joined on scope.id        | Map(LC(String), String)  |
//! | ScopeName          | scope.name -> reuse                  | String                   |
//! | ScopeSchemaUrl     | scope.schema_url -> reuse            | LC(String)               |
//! | ScopeVersion       | scope.version -> reuse               | LC(String)               |
//! | ServiceName        | extract service.name from resource   | LC(String)               |
//! | SeverityNumber     | severity_number -> cast to UInt8     | UInt8                    |
//! | SeverityText       | severity_text -> reuse               | LC(String)               |
//! | SpanId             | span_id -> lowercase hex string      | String                   |
//! | Timestamp          | time_unix_nano -> reuse              | DateTime64(9)            |
//! | TraceFlags         | span_flags -> reuse                  | UInt8                    |
//! | TraceId            | trace_id -> lowercase hex string     | String                   |
//! +--------------------+--------------------------------------+--------------------------+
//! ```
//!
//! ```text
//!                      +--------------------------------+
//! all output arrays -->| sort columns lexically         |
//!                      | build/reuse cached Arrow schema |
//!                      +---------------+----------------+
//!                                      |
//!                                      v
//!                      +--------------------------------+
//!                      | RecordBatch                    |
//!                      +---------------+----------------+
//!                                      |
//!                                      | FORMAT ArrowStream
//!                                      v
//!                      +--------------------------------+
//!                      | ClickHouse otel_logs table      |
//!                      +--------------------------------+
//! ```
//!
//! An unsupported input shape returns [`LogsFastTransform::NotApplicable`] without modifying the
//! input, allowing the caller to use the generic transformer. Conversion and Arrow errors are
//! propagated instead of falling back, so malformed data is not silently handled differently.
//! Equivalence tests compare the specialized output with the generic transformation plan.

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, MapArray, MapBuilder, RecordBatch, StringArray, StringBuilder, StructArray,
    UInt16Array, UInt32Array,
};
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::{OtapArrowRecords, schema::consts};

use crate::exporters::clickhouse_exporter::arrays::{StructColumnAccessor, get_u16_array_opt};
use crate::exporters::clickhouse_exporter::consts as ch_consts;
use crate::exporters::clickhouse_exporter::error::ClickhouseExporterError;
use crate::exporters::clickhouse_exporter::transform::transform_attributes::{
    group_attributes_to_map_str, inline_attributes_to_parent,
};
use crate::exporters::clickhouse_exporter::transform::transform_column::{
    fixed_binary_to_hex_string, struct_column_to_string,
};

const OTAP_SPAN_FLAGS: &str = "span_flags";
const SERVICE_NAME_KEY: &str = "service.name";
const MISSING_INDEX: u32 = u32::MAX;

/// Result of attempting the specialized transformation.
pub(crate) enum LogsFastTransform {
    /// The canonical logs layout was transformed successfully.
    Applied(RecordBatch),
    /// The input layout is valid but not supported by this specialized path.
    ///
    /// The reason is retained for benchmark and test diagnostics; production records a counter.
    NotApplicable(#[allow(dead_code)] &'static str),
}

/// Specialized transformer with a one-entry output schema cache.
#[derive(Default)]
pub(crate) struct LogsFastTransformer {
    cached_schema: Option<(Vec<(&'static str, DataType)>, Arc<Schema>)>,
}

impl LogsFastTransformer {
    /// Transform a decoded canonical OTAP logs batch or decline it without modifying the input.
    pub(crate) fn try_apply(
        &mut self,
        records: &OtapArrowRecords,
    ) -> Result<LogsFastTransform, ClickhouseExporterError> {
        let Some(logs) = records.get(ArrowPayloadType::Logs) else {
            return Ok(LogsFastTransform::NotApplicable("missing logs payload"));
        };

        let resource = match logs.column_by_name(consts::RESOURCE) {
            Some(array) => match array.as_any().downcast_ref::<StructArray>() {
                Some(array) => Some(array),
                None => {
                    return Ok(LogsFastTransform::NotApplicable(
                        "resource column is not a struct",
                    ));
                }
            },
            None => None,
        };
        let scope = match logs.column_by_name(consts::SCOPE) {
            Some(array) => match array.as_any().downcast_ref::<StructArray>() {
                Some(array) => Some(array),
                None => {
                    return Ok(LogsFastTransform::NotApplicable(
                        "scope column is not a struct",
                    ));
                }
            },
            None => None,
        };
        let body = match logs.column_by_name(consts::BODY) {
            Some(array) => match array.as_any().downcast_ref::<StructArray>() {
                Some(array) => Some(array),
                None => {
                    return Ok(LogsFastTransform::NotApplicable(
                        "body column is not a struct",
                    ));
                }
            },
            None => None,
        };

        let resource_attrs = grouped_attributes(records, ArrowPayloadType::ResourceAttrs)?;
        let scope_attrs = grouped_attributes(records, ArrowPayloadType::ScopeAttrs)?;
        let log_attrs = records.get(ArrowPayloadType::LogAttrs);

        let mut columns: Vec<(&'static str, ArrayRef)> = Vec::with_capacity(16);
        if let Some(body) = body {
            let Some(body_types) = StructColumnAccessor::new(body)
                .primitive_column_op::<UInt8Type>(consts::ATTRIBUTE_TYPE)?
            else {
                return Ok(LogsFastTransform::NotApplicable(
                    "body type column is not UInt8",
                ));
            };
            columns.push((
                ch_consts::CH_BODY,
                struct_column_to_string(body_types, &StructColumnAccessor::new(body))?,
            ));
        }
        if let Some(column) = logs.column_by_name(consts::EVENT_NAME) {
            columns.push((ch_consts::CH_EVENT_NAME, column.clone()));
        }

        if let Some(attributes_batch) = log_attrs {
            let Some(log_ids) = fast_path_u16_column(logs, consts::ID) else {
                return Ok(LogsFastTransform::NotApplicable(
                    "log id column is not UInt16",
                ));
            };
            if let Some(attributes) = inline_attributes_to_parent(attributes_batch, log_ids)? {
                columns.push((ch_consts::CH_LOG_ATTRIBUTES, Arc::new(attributes)));
            }
        }

        if let Some(compact) = resource_attrs.as_ref() {
            let Some(resource_ids) =
                resource.and_then(|array| fast_path_struct_u16_column(array, consts::ID))
            else {
                return Ok(LogsFastTransform::NotApplicable(
                    "resource id column is not UInt16",
                ));
            };
            let (attributes, service_name) = inline_attributes(resource_ids, compact, true)?;
            columns.push((ch_consts::CH_RESOURCE_ATTRIBUTES, attributes));
            columns.push((
                ch_consts::CH_SERVICE_NAME,
                service_name.expect("service name requested"),
            ));
        }

        if let Some(resource_schema_url) =
            resource.and_then(|array| array.column_by_name(consts::SCHEMA_URL))
        {
            columns.push((
                ch_consts::CH_RESOURCE_SCHEMA_URL,
                resource_schema_url.clone(),
            ));
        }

        if let Some(compact) = scope_attrs.as_ref() {
            let Some(scope_ids) =
                scope.and_then(|array| fast_path_struct_u16_column(array, consts::ID))
            else {
                return Ok(LogsFastTransform::NotApplicable(
                    "scope id column is not UInt16",
                ));
            };
            let (attributes, _) = inline_attributes(scope_ids, compact, false)?;
            columns.push((ch_consts::CH_SCOPE_ATTRIBUTES, attributes));
        }

        if let Some(scope) = scope {
            for (source, destination) in [
                (consts::NAME, ch_consts::CH_SCOPE_NAME),
                (consts::SCHEMA_URL, ch_consts::CH_SCOPE_SCHEMA_URL),
                (consts::VERSION, ch_consts::CH_SCOPE_VERSION),
            ] {
                if let Some(column) = scope.column_by_name(source) {
                    columns.push((destination, column.clone()));
                }
            }
        }

        if let Some(column) = logs.column_by_name(consts::SEVERITY_NUMBER) {
            columns.push((
                ch_consts::CH_SEVERITY_NUMBER,
                cast(column, &DataType::UInt8)?,
            ));
        }
        if let Some(column) = logs.column_by_name(consts::SEVERITY_TEXT) {
            columns.push((ch_consts::CH_SEVERITY_TEXT, column.clone()));
        }
        if let Some(column) = logs.column_by_name(consts::SPAN_ID) {
            columns.push((ch_consts::CH_SPAN_ID, fixed_binary_to_hex_string(column)?));
        }
        if let Some(column) = logs.column_by_name(consts::TIME_UNIX_NANO) {
            columns.push((ch_consts::CH_TIMESTAMP, column.clone()));
        }
        if let Some(column) = logs.column_by_name(OTAP_SPAN_FLAGS) {
            columns.push((ch_consts::CH_TRACE_FLAGS, column.clone()));
        }
        if let Some(column) = logs.column_by_name(consts::TRACE_ID) {
            columns.push((ch_consts::CH_TRACE_ID, fixed_binary_to_hex_string(column)?));
        }

        if columns.is_empty() {
            return Ok(LogsFastTransform::NotApplicable(
                "logs payload has no ClickHouse columns",
            ));
        }

        // The generic transformer emits columns in lexical order. Matching that order makes the
        // specialized output directly comparable and keeps ArrowStream schemas stable.
        columns.sort_unstable_by_key(|(name, _)| *name);
        let schema = self.schema_for(&columns);
        let arrays = columns.into_iter().map(|(_, array)| array).collect();
        let batch = RecordBatch::try_new(schema, arrays)?;

        Ok(LogsFastTransform::Applied(batch))
    }

    fn schema_for(&mut self, columns: &[(&'static str, ArrayRef)]) -> Arc<Schema> {
        let key: Vec<_> = columns
            .iter()
            .map(|(name, array)| (*name, array.data_type().clone()))
            .collect();
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

fn fast_path_u16_column<'a>(batch: &'a RecordBatch, name: &str) -> Option<&'a UInt16Array> {
    // Missing and mistyped columns both mean that this specialized layout does not apply. The
    // generic transformer remains responsible for deciding whether the input itself is invalid.
    get_u16_array_opt(batch, name).ok().flatten()
}

fn fast_path_struct_u16_column<'a>(array: &'a StructArray, name: &str) -> Option<&'a UInt16Array> {
    // Preserve the same applicability policy while delegating Arrow access and type checking to
    // the exporter's shared accessor.
    StructColumnAccessor::new(array)
        .primitive_column_op::<UInt16Type>(name)
        .ok()
        .flatten()
}

fn grouped_attributes(
    records: &OtapArrowRecords,
    payload_type: ArrowPayloadType,
) -> Result<Option<(UInt32Array, MapArray)>, ClickhouseExporterError> {
    let Some(batch) = records.get(payload_type) else {
        return Ok(None);
    };
    group_attributes_to_map_str(batch)
}

fn inline_attributes(
    parent_ids: &UInt16Array,
    compact: &(UInt32Array, MapArray),
    extract_service_name: bool,
) -> Result<(ArrayRef, Option<ArrayRef>), ClickhouseExporterError> {
    let (compact_ids, compact_maps) = compact;
    let keys = compact_maps
        .keys()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "attribute map keys are not strings".into(),
        })?;
    let values = compact_maps
        .values()
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ClickhouseExporterError::CoercionError {
            error: "attribute map values are not strings".into(),
        })?;

    let max_id = compact_ids.iter().flatten().max().unwrap_or(0) as usize;
    let mut dense_remap = vec![MISSING_INDEX; max_id.saturating_add(1)];
    for (index, id) in compact_ids.iter().enumerate() {
        if let Some(id) = id
            && let Some(slot) = dense_remap.get_mut(id as usize)
        {
            *slot = index as u32;
        }
    }

    // Bulk range copies with MutableArrayData did not improve typical batches in benchmarks.
    // Revisit that approach for high-cardinality attributes, where the setup cost may amortize.
    let mut map_builder = MapBuilder::with_capacity(
        None,
        StringBuilder::new(),
        StringBuilder::new(),
        parent_ids.len(),
    );
    let mut service_builder = extract_service_name.then(|| {
        StringBuilder::with_capacity(parent_ids.len(), parent_ids.len().saturating_mul(16))
    });
    let offsets = compact_maps.offsets();

    for row in 0..parent_ids.len() {
        let compact_index = if parent_ids.is_valid(row) {
            dense_remap
                .get(parent_ids.value(row) as usize)
                .copied()
                .filter(|index| *index != MISSING_INDEX)
                .map(|index| index as usize)
        } else {
            None
        };
        let mut service_name = None;

        if let Some(compact_index) = compact_index {
            let start = offsets[compact_index] as usize;
            let end = offsets[compact_index + 1] as usize;
            for index in start..end {
                let key = keys.value(index);
                map_builder.keys().append_value(key);
                if values.is_null(index) {
                    map_builder.values().append_null();
                } else {
                    let value = values.value(index);
                    map_builder.values().append_value(value);
                    if service_name.is_none() && key == SERVICE_NAME_KEY {
                        service_name = Some(value);
                    }
                }
            }
        }

        map_builder.append(true)?;
        if let Some(builder) = &mut service_builder {
            builder.append_value(service_name.unwrap_or(""));
        }
    }

    Ok((
        Arc::new(map_builder.finish()),
        service_builder.map(|mut builder| Arc::new(builder.finish()) as ArrayRef),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::UInt32Builder;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::{fixtures, round_trip::encode_logs};

    use crate::exporters::clickhouse_exporter::transform::transform_batch::BatchTransformer;

    fn decoded_logs(logs: LogsData) -> OtapArrowRecords {
        let mut records = encode_logs(&logs);
        records
            .decode_transport_optimized_ids()
            .expect("decode IDs");
        records
    }

    fn assert_matches_generic(records: OtapArrowRecords) {
        let mut generic = BatchTransformer::new();
        let expected = generic
            .apply_plan(records.clone())
            .expect("generic transform")
            .remove(&ArrowPayloadType::Logs)
            .expect("generic logs output");
        let actual = match LogsFastTransformer::default()
            .try_apply(&records)
            .expect("fast transform")
        {
            LogsFastTransform::Applied(batch) => batch,
            LogsFastTransform::NotApplicable(reason) => {
                panic!("canonical logs unexpectedly declined: {reason}")
            }
        };
        assert_eq!(actual, expected);
    }

    fn rich_logs() -> LogsData {
        LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("service.name", AnyValue::new_string("fast-path-service")),
                    KeyValue::new("resource.int", AnyValue::new_int(42)),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("fast-path-scope".to_string())
                    .version("1.2.3".to_string())
                    .attributes(vec![KeyValue::new("scope.bool", AnyValue::new_bool(true))])
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(1_000u64)
                        .severity_number(SeverityNumber::Info)
                        .severity_text("INFO")
                        .body(AnyValue::new_string("message"))
                        .attributes(vec![KeyValue::new("log.double", AnyValue::new_double(1.5))])
                        .flags(LogRecordFlags::TraceFlagsMask)
                        .trace_id(vec![0x11; 16])
                        .span_id(vec![0x22; 8])
                        .event_name("rich-event")
                        .finish(),
                ],
            )],
        )])
    }

    /// Scenario: canonical logs contain resource, scope, and log attributes.
    /// Guarantees: the fast path produces the exact same ClickHouse batch as the generic path.
    #[test]
    fn full_logs_match_generic_transform() {
        assert_matches_generic(decoded_logs(fixtures::logs_with_full_resource_and_scope()));
    }

    /// Scenario: canonical logs omit all resource, scope, and log attributes.
    /// Guarantees: the fast path preserves the generic path's omitted attribute columns.
    #[test]
    fn logs_without_attributes_match_generic_transform() {
        assert_matches_generic(decoded_logs(fixtures::logs_with_no_attributes()));
    }

    /// Scenario: canonical logs contain multiple resources and scopes with mixed content.
    /// Guarantees: fixed-layout joins preserve row ordering and values from the generic path.
    #[test]
    fn mixed_resources_and_scopes_match_generic_transform() {
        assert_matches_generic(decoded_logs(
            fixtures::logs_multiple_resources_mixed_content(),
        ));
    }

    /// Scenario: canonical logs exercise every transformed ClickHouse field and attribute level.
    /// Guarantees: service extraction, ID hex encoding, body conversion, and maps match generic output.
    #[test]
    fn rich_logs_match_generic_transform() {
        assert_matches_generic(decoded_logs(rich_logs()));
    }

    /// Scenario: the same canonical logs layout is transformed more than once.
    /// Guarantees: the specialized transformer reuses its cached output schema.
    #[test]
    fn repeated_layout_reuses_cached_schema() {
        let records = decoded_logs(rich_logs());
        let mut transformer = LogsFastTransformer::default();
        let first = match transformer.try_apply(&records).expect("first transform") {
            LogsFastTransform::Applied(batch) => batch,
            LogsFastTransform::NotApplicable(reason) => {
                panic!("canonical logs unexpectedly declined: {reason}")
            }
        };
        let second = match transformer.try_apply(&records).expect("second transform") {
            LogsFastTransform::Applied(batch) => batch,
            LogsFastTransform::NotApplicable(reason) => {
                panic!("canonical logs unexpectedly declined: {reason}")
            }
        };

        assert!(Arc::ptr_eq(first.schema_ref(), second.schema_ref()));
    }

    /// Scenario: a root log ID is present as UInt16, absent, or encoded with another type.
    /// Guarantees: the shared accessor is reused while unsupported layouts remain fallback cases.
    #[test]
    fn root_log_id_accessor_preserves_fast_path_fallback_policy() {
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("valid_id", DataType::UInt16, false),
                Field::new("wrong_id", DataType::UInt32, false),
            ])),
            vec![
                Arc::new(UInt16Array::from(vec![7_u16])),
                Arc::new(UInt32Array::from(vec![7_u32])),
            ],
        )
        .expect("ID test batch");

        assert_eq!(
            fast_path_u16_column(&batch, "valid_id").map(|ids| ids.value(0)),
            Some(7)
        );
        assert!(fast_path_u16_column(&batch, "missing_id").is_none());
        assert!(fast_path_u16_column(&batch, "wrong_id").is_none());
    }

    /// Scenario: a resource or scope ID is present as UInt16, absent, or has another type.
    /// Guarantees: nested ID access reuses StructColumnAccessor and preserves generic fallback.
    #[test]
    fn nested_parent_id_accessor_preserves_fast_path_fallback_policy() {
        let array = StructArray::from(vec![
            (
                Arc::new(Field::new("valid_id", DataType::UInt16, false)),
                Arc::new(UInt16Array::from(vec![11_u16])) as ArrayRef,
            ),
            (
                Arc::new(Field::new("wrong_id", DataType::UInt32, false)),
                Arc::new(UInt32Array::from(vec![11_u32])) as ArrayRef,
            ),
        ]);

        assert_eq!(
            fast_path_struct_u16_column(&array, "valid_id").map(|ids| ids.value(0)),
            Some(11)
        );
        assert!(fast_path_struct_u16_column(&array, "missing_id").is_none());
        assert!(fast_path_struct_u16_column(&array, "wrong_id").is_none());
    }

    /// Scenario: parent rows contain sparse, null, known, and unknown attribute IDs.
    /// Guarantees: maps preserve parent order and service names default when no match exists.
    #[test]
    fn inline_attributes_handles_sparse_and_null_parent_ids() {
        let mut map_builder = MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        map_builder.keys().append_value(SERVICE_NAME_KEY);
        map_builder.values().append_value("checkout");
        map_builder.keys().append_value("nullable");
        map_builder.values().append_null();
        map_builder.append(true).expect("first compact map");
        map_builder.keys().append_value("other");
        map_builder.values().append_value("value");
        map_builder.append(true).expect("second compact map");
        let compact = (UInt32Array::from(vec![2_u32, 7_u32]), map_builder.finish());
        let parent_ids = UInt16Array::from(vec![Some(7_u16), None, Some(2_u16), Some(9_u16)]);

        let (attributes, service_names) =
            inline_attributes(&parent_ids, &compact, true).expect("inline compact attributes");
        let attributes = attributes
            .as_any()
            .downcast_ref::<MapArray>()
            .expect("attribute map output");
        let keys = attributes
            .keys()
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("string keys");
        let values = attributes
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("string values");
        let service_names = service_names
            .expect("service names requested")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("service name strings")
            .clone();

        assert_eq!(attributes.value_offsets(), &[0, 1, 1, 3, 3]);
        assert_eq!(keys.value(0), "other");
        assert_eq!(keys.value(1), SERVICE_NAME_KEY);
        assert_eq!(keys.value(2), "nullable");
        assert_eq!(values.value(0), "value");
        assert_eq!(values.value(1), "checkout");
        assert!(values.is_null(2));
        assert_eq!(
            service_names.iter().collect::<Vec<_>>(),
            vec![Some(""), Some(""), Some("checkout"), Some("")]
        );
    }

    /// Scenario: a compact attribute map uses a non-string key or value type.
    /// Guarantees: the specialized inliner reports coercion errors instead of reading invalid data.
    #[test]
    fn inline_attributes_rejects_non_string_map_entries() {
        let mut map_builder = MapBuilder::new(None, UInt32Builder::new(), StringBuilder::new());
        map_builder.keys().append_value(1);
        map_builder.values().append_value("value");
        map_builder.append(true).expect("compact map");
        let compact = (UInt32Array::from(vec![1_u32]), map_builder.finish());
        let parent_ids = UInt16Array::from(vec![1_u16]);

        let error = inline_attributes(&parent_ids, &compact, false)
            .expect_err("non-string keys must be rejected");

        assert!(error.to_string().contains("keys are not strings"));

        let mut map_builder = MapBuilder::new(None, StringBuilder::new(), UInt32Builder::new());
        map_builder.keys().append_value("key");
        map_builder.values().append_value(1);
        map_builder.append(true).expect("compact map");
        let compact = (UInt32Array::from(vec![1_u32]), map_builder.finish());
        let error = inline_attributes(&parent_ids, &compact, false)
            .expect_err("non-string values must be rejected");

        assert!(error.to_string().contains("values are not strings"));
    }

    /// Scenario: an OTAP logs envelope contains no root logs record batch.
    /// Guarantees: the fast path declines the input so normal exporter handling remains available.
    #[test]
    fn missing_logs_payload_is_declined_for_generic_fallback() {
        let records = encode_logs(&fixtures::empty_logs());
        let result = LogsFastTransformer::default()
            .try_apply(&records)
            .expect("applicability check");
        assert!(matches!(result, LogsFastTransform::NotApplicable(_)));
    }
}
