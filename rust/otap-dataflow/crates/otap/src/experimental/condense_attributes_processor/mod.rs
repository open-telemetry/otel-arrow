// Copyright The OpenTelemetry Authors SPDX-License-Identifier: Apache-2.0

//! Condense Attributes Processor
//!
//! This module provides a processor performing a unique kind of attribute transformation.
//! It condenses multiple attributes into a single entry based on specified rules.
//!
//! This functionality may be useful for scenarios where attribute data needs to be simplified to match a specific output schema.
//!

use arrow::array::{
    Array, BooleanArray, DictionaryArray, Float64Array, Int64Array, StringArray, UInt8Array,
    UInt16Array,
};
use arrow::datatypes::{UInt8Type, UInt16Type};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::{Error, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;

/// URN identifier for the Condense Attributes processor
pub const CONDENSE_ATTRIBUTES_PROCESSOR_URN: &str = "urn:otel:condense_attributes:processor";

/// Configuration for the Condense Attributes processor
///
/// `destination_key`: The key under which the condensed attributes will be stored.
/// `delimiter`: The character used to separate individual attribute values in the condensed string.
/// `source_keys`: Optional set of specific keys to condense. Cannot be specified with `exclude_keys`.
/// `exclude_keys`: Optional set of keys to exclude from condensing. Cannot be specified with `source_keys`.
///
/// If neither `source_keys` nor `exclude_keys` is specified, all attributes will be condensed.
pub struct Config {
    destination_key: String,
    delimiter: char,
    source_keys: Option<HashSet<String>>,
    exclude_keys: Option<HashSet<String>>,
}

/// Processor that condenses multiple attributes into a single attribute based on predefined rules.
pub struct CondenseAttributesProcessor {
    config: Config,
}

fn engine_err(msg: &str) -> Error {
    Error::PdataConversionError {
        error: msg.to_string(),
    }
}

/// Factory function to create a Condense Attributes processor
pub fn create_condense_attributes_processor(
    _: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        CondenseAttributesProcessor::from_config(&node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register CondenseAttributesProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static CONDENSE_ATTRIBUTES_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: CONDENSE_ATTRIBUTES_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_condense_attributes_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

impl CondenseAttributesProcessor {
    /// Creates a new CondenseAttributesProcessor instance
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Creates a new CondenseAttributesProcessor instance from configuration
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let destination_key = config
            .get("destination_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: "destination_key is required and must be a string".to_string(),
            })?
            .to_string();

        let delimiter = config
            .get("delimiter")
            .and_then(|v| v.as_str())
            .and_then(|s| s.chars().next())
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: "delimiter is required and must be a single character string".to_string(),
            })?;

        let source_keys = if let Some(source_keys_val) = config.get("source_keys") {
            let keys: HashSet<String> = source_keys_val
                .as_array()
                .ok_or_else(|| ConfigError::InvalidUserConfig {
                    error: "source_keys must be an array".to_string(),
                })?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Some(keys)
        } else {
            None
        };

        let exclude_keys = if let Some(exclude_keys_val) = config.get("exclude_keys") {
            let keys: HashSet<String> = exclude_keys_val
                .as_array()
                .ok_or_else(|| ConfigError::InvalidUserConfig {
                    error: "exclude_keys must be an array".to_string(),
                })?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Some(keys)
        } else {
            None
        };

        // Validate that both source_keys and exclude_keys are not specified
        if source_keys.is_some() && exclude_keys.is_some() {
            return Err(ConfigError::InvalidUserConfig {
                error: "cannot specify both 'source_keys' and 'exclude_keys'".to_string(),
            });
        }

        Ok(Self::new(Config {
            destination_key,
            delimiter,
            source_keys,
            exclude_keys,
        }))
    }

    fn condense(&self, records: &mut OtapArrowRecords) -> Result<u64, Error> {
        let rb = match records.get(ArrowPayloadType::LogAttrs) {
            Some(rb) => rb,
            None => return Ok(0),
        };

        let num_rows = rb.num_rows();
        if num_rows == 0 {
            return Ok(0);
        }

        // Get parent_id column
        let parent_id_col = rb
            .column_by_name(consts::PARENT_ID)
            .ok_or_else(|| engine_err("parent_id column not found"))?;
        // Assume parent_id is UInt16 since it currently only support LogAttrs
        let parent_id_arr = parent_id_col
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| engine_err("parent_id column is not UInt16"))?;

        // Get attribute key column
        let key_col = rb
            .column_by_name(consts::ATTRIBUTE_KEY)
            .ok_or_else(|| engine_err("key column not found"))?;

        // Get attribute type column
        let type_col = rb
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .ok_or_else(|| engine_err("type column not found"))?;
        let type_arr = type_col
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| engine_err("type column is not UInt8"))?;

        let str_col = rb.column_by_name(consts::ATTRIBUTE_STR);
        let int_col = rb.column_by_name(consts::ATTRIBUTE_INT);
        let double_col = rb.column_by_name(consts::ATTRIBUTE_DOUBLE);
        let bool_col = rb.column_by_name(consts::ATTRIBUTE_BOOL);
        let delimiter_str = self.config.delimiter.to_string();

        // Helper function to get key at index i
        // Keys may be stored as plain StringArray or DictionaryArray
        let get_key = |i: usize| -> Option<&str> {
            match key_col.data_type() {
                arrow::datatypes::DataType::Utf8 => {
                    let arr = key_col.as_any().downcast_ref::<StringArray>()?;
                    if arr.is_null(i) {
                        None
                    } else {
                        Some(arr.value(i))
                    }
                }
                arrow::datatypes::DataType::Dictionary(_, _) => {
                    if let Some(dict_arr) = key_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt8Type>>()
                    {
                        if dict_arr.is_null(i) {
                            return None;
                        }
                        let dict_key = dict_arr.keys().value(i);
                        let values = dict_arr.values().as_any().downcast_ref::<StringArray>()?;
                        return Some(values.value(dict_key as usize));
                    }
                    if let Some(dict_arr) = key_col
                        .as_any()
                        .downcast_ref::<DictionaryArray<UInt16Type>>()
                    {
                        if dict_arr.is_null(i) {
                            return None;
                        }
                        let dict_key = dict_arr.keys().value(i);
                        let values = dict_arr.values().as_any().downcast_ref::<StringArray>()?;
                        return Some(values.value(dict_key as usize));
                    }
                    None
                }
                _ => None,
            }
        };

        // Helper function to extract value as string based on type
        let get_value_str = |value_type: AttributeValueType, i: usize| -> Option<String> {
            match value_type {
                AttributeValueType::Str => str_col.and_then(|col| {
                    Self::extract_value_from_column(col, i, |arr: &StringArray, index| {
                        arr.value(index).to_string()
                    })
                }),
                AttributeValueType::Int => int_col.and_then(|col| {
                    Self::extract_value_from_column(col, i, |arr: &Int64Array, index| {
                        arr.value(index)
                    })
                    .map(|v| v.to_string())
                }),
                AttributeValueType::Double => double_col.and_then(|col| {
                    Self::extract_value_from_column(col, i, |arr: &Float64Array, index| {
                        arr.value(index)
                    })
                    .map(|v| v.to_string())
                }),
                AttributeValueType::Bool => bool_col.and_then(|col| {
                    Self::extract_value_from_column(col, i, |arr: &BooleanArray, index| {
                        arr.value(index)
                    })
                    .map(|v| v.to_string())
                }),
                // If needed, add handling for Map, Slice, and Bytes?
                _ => None,
            }
        };

        let mut parent_to_attrs: HashMap<u16, Vec<(&str, String)>> = HashMap::new();
        let mut rows_to_preserve: Vec<usize> = Vec::new();

        for i in 0..num_rows {
            if parent_id_arr.is_null(i) {
                continue;
            }
            let parent_id = parent_id_arr.value(i);

            let key = match get_key(i) {
                Some(k) => k,
                None => continue,
            };

            // Check if we should include this key
            let should_condense = match (&self.config.source_keys, &self.config.exclude_keys) {
                (Some(source), _) => source.contains(key),
                (None, Some(exclude)) => !exclude.contains(key),
                (None, None) => true,
            };

            if !should_condense {
                rows_to_preserve.push(i);
                continue;
            }

            if type_arr.is_null(i) {
                continue;
            }
            let value_type = type_arr.value(i);

            if let Ok(value_type_enum) = AttributeValueType::try_from(value_type) {
                if let Some(val) = get_value_str(value_type_enum, i) {
                    parent_to_attrs
                        .entry(parent_id)
                        .or_insert_with(Vec::new)
                        .push((key, val));
                }
            }
        }

        // Build new record batch
        let mut builder = StrKeysAttributesRecordBatchBuilder::<u16>::new();
        let mut condensed_count = 0u64;

        // Add condensed attributes
        for (parent_id, attrs) in parent_to_attrs.iter() {
            if !attrs.is_empty() {
                // Allocate capacity for all key=val strings + n-1 delimiters
                let total_len = attrs
                    .iter()
                    .map(|(k, v)| k.len() + 1 + v.len())
                    .sum::<usize>()
                    + (attrs.len() - 1) * delimiter_str.len();
                let mut condensed_value = String::with_capacity(total_len);

                for (index, (key, val)) in attrs.iter().enumerate() {
                    if index > 0 {
                        condensed_value.push(self.config.delimiter);
                    }
                    condensed_value.push_str(key);
                    condensed_value.push('=');
                    condensed_value.push_str(val);
                    condensed_count += 1;
                }

                builder.append_parent_id(parent_id);
                builder.append_key(&self.config.destination_key);
                builder
                    .any_values_builder
                    .append_str(condensed_value.as_bytes());
            }
        }

        // Add preserved attributes
        for &i in &rows_to_preserve {
            let parent_id = parent_id_arr.value(i);
            let key = get_key(i).expect("key was validated as non-null");
            let value_type = type_arr.value(i);

            builder.append_parent_id(&parent_id);
            builder.append_key(key);

            if let Ok(value_type) = AttributeValueType::try_from(value_type) {
                match value_type {
                    AttributeValueType::Str => {
                        if let Some(val) = str_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &StringArray, index| {
                                arr.value(index).to_string()
                            })
                        }) {
                            builder.any_values_builder.append_str(val.as_bytes());
                        }
                    }
                    AttributeValueType::Int => {
                        if let Some(val) = int_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &Int64Array, index| {
                                arr.value(index)
                            })
                        }) {
                            builder.any_values_builder.append_int(val);
                        }
                    }
                    AttributeValueType::Double => {
                        if let Some(val) = double_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &Float64Array, index| {
                                arr.value(index)
                            })
                        }) {
                            builder.any_values_builder.append_double(val);
                        }
                    }
                    AttributeValueType::Bool => {
                        if let Some(val) = bool_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &BooleanArray, index| {
                                arr.value(index)
                            })
                        }) {
                            builder.any_values_builder.append_bool(val);
                        }
                    }
                    // If needed, add handling for Map, Slice, and Bytes?
                    _ => {}
                }
            }
        }

        // TODO: It seems wasteful to build a new batch and copy preserved attributes.
        // Once https://github.com/open-telemetry/otel-arrow/issues/1035 is resolved, should instead modify the original batch in-place.
        // - Inserting the new condensed attribute
        // - Removing the original attributes that were condensed
        // This would cut down on a lot of the copying currently happening.
        let new_batch = builder.finish().map_err(|e| {
            engine_err(&format!(
                "Failed to build condensed attributes batch: {}",
                e
            ))
        })?;

        records.set(ArrowPayloadType::LogAttrs, new_batch);

        Ok(condensed_count)
    }

    fn extract_value_from_column<T, PlainArr, F>(
        col: &Arc<dyn Array>,
        i: usize,
        extract_plain: F,
    ) -> Option<T>
    where
        PlainArr: Array + 'static,
        F: Fn(&PlainArr, usize) -> T,
    {
        if let Some(arr) = col.as_any().downcast_ref::<PlainArr>() {
            if arr.is_null(i) {
                return None;
            }
            return Some(extract_plain(arr, i));
        }
        if let Some(dict_arr) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
            if dict_arr.is_null(i) {
                return None;
            }
            let dict_key = dict_arr.keys().value(i);
            let values = dict_arr.values().as_any().downcast_ref::<PlainArr>()?;
            return Some(extract_plain(values, dict_key as usize));
        }
        if let Some(dict_arr) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
            if dict_arr.is_null(i) {
                return None;
            }
            let dict_key = dict_arr.keys().value(i);
            let values = dict_arr.values().as_any().downcast_ref::<PlainArr>()?;
            return Some(extract_plain(values, dict_key as usize));
        }
        None
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for CondenseAttributesProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(_control) => Ok(()),
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                let (context, payload) = pdata.into_parts();

                let mut records: OtapArrowRecords = payload.try_into()?;

                let condensed_records: OtapArrowRecords =
                    match signal {
                        SignalType::Logs => match self.condense(&mut records) {
                            Ok(_condensed) => records,
                            Err(e) => return Err(e),
                        },
                        _ => return Err(Error::ProcessorError {
                            processor: effect_handler.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error:
                                "CondenseAttributesProcessor only supported for SignalType 'Logs'"
                                    .to_string(),
                            source_detail: String::new(),
                        }),
                    };

                effect_handler
                    .send_message(OtapPdata::new(context, condensed_records.into()))
                    .await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod condense_tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use bytes::BytesMut;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message as _;
    use serde_json::json;

    fn build_log_with_attrs(log_attrs: Vec<KeyValue>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                ..Default::default()
            },
            vec![ScopeLogs::new(
                InstrumentationScope {
                    ..Default::default()
                },
                vec![
                    LogRecord::build()
                        .time_unix_nano(1u64)
                        .severity_number(SeverityNumber::Info)
                        .event_name("")
                        .attributes(log_attrs)
                        .finish(),
                ],
            )],
        )])
    }

    fn test_condense_helper(
        input: ExportLogsServiceRequest,
        cfg: Value,
        expected_attrs: Vec<KeyValue>,
    ) {
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let node = test_node("condense-attributes-processor-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config =
            NodeUserConfig::new_processor_config(CONDENSE_ATTRIBUTES_PROCESSOR_URN);
        node_config.config = cfg;
        let proc = create_condense_attributes_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;

                assert_eq!(log_attrs, &expected_attrs);
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_condense_all() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });

        let expected_attrs = vec![KeyValue::new(
            "condensed",
            AnyValue::new_string("attr1=value1;attr2=42;attr3=true"),
        )];

        test_condense_helper(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_with_source_keys() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["attr1", "attr2"]
        });

        let expected_attrs = vec![
            KeyValue::new("condensed", AnyValue::new_string("attr1=value1;attr2=42")),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ];

        test_condense_helper(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_with_exclude_keys() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "exclude_keys": ["attr1"]
        });

        let expected_attrs = vec![
            KeyValue::new("condensed", AnyValue::new_string("attr2=42;attr3=true")),
            KeyValue::new("attr1", AnyValue::new_string("value1")),
        ];

        test_condense_helper(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_no_attributes() {
        let input = build_log_with_attrs(vec![]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });

        let expected_attrs = vec![];

        test_condense_helper(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_no_attributes_matching_source_keys() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["nonexistent1", "nonexistent2"]
        });

        let expected_attrs = vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ];

        test_condense_helper(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_no_attributes_matching_exclude_keys() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "exclude_keys": ["attr1", "attr2", "attr3"]
        });

        let expected_attrs = vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
            KeyValue::new("attr3", AnyValue::new_bool(true)),
        ];

        test_condense_helper(input, cfg, expected_attrs);
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_parsing_with_source_keys() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": "|",
            "source_keys": ["key1", "key2", "key3"]
        });

        let processor = CondenseAttributesProcessor::from_config(&cfg).expect("valid config");
        assert_eq!(processor.config.destination_key, "condensed");
        assert_eq!(processor.config.delimiter, '|');
        assert!(processor.config.exclude_keys.is_none());

        let source_keys = processor
            .config
            .source_keys
            .as_ref()
            .expect("source_keys should be present");
        assert_eq!(source_keys.len(), 3);
        assert!(source_keys.contains("key1"));
        assert!(source_keys.contains("key2"));
        assert!(source_keys.contains("key3"));
    }

    #[test]
    fn test_config_parsing_with_exclude_keys() {
        let cfg = json!({
            "destination_key": "condensed_attr",
            "delimiter": ",",
            "exclude_keys": ["id", "timestamp"]
        });

        let processor = CondenseAttributesProcessor::from_config(&cfg).expect("valid config");
        assert_eq!(processor.config.destination_key, "condensed_attr");
        assert_eq!(processor.config.delimiter, ',');
        assert!(processor.config.source_keys.is_none());

        let exclude_keys = processor
            .config
            .exclude_keys
            .as_ref()
            .expect("exclude_keys should be present");
        assert_eq!(exclude_keys.len(), 2);
        assert!(exclude_keys.contains("id"));
        assert!(exclude_keys.contains("timestamp"));
    }

    #[test]
    fn test_config_parsing_with_condense_all() {
        let cfg = json!({
            "destination_key": "all_condensed",
            "delimiter": ";"
        });

        let processor = CondenseAttributesProcessor::from_config(&cfg).expect("valid config");
        assert_eq!(processor.config.destination_key, "all_condensed");
        assert_eq!(processor.config.delimiter, ';');
        assert!(processor.config.source_keys.is_none());
        assert!(processor.config.exclude_keys.is_none());
    }

    #[test]
    fn test_config_parsing_missing_destination_key() {
        let cfg = json!({
            "delimiter": "|",
            "source_keys": ["key1"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("destination_key"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }

    #[test]
    fn test_config_parsing_missing_delimiter() {
        let cfg = json!({
            "destination_key": "condensed",
            "source_keys": ["key1"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("delimiter"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }

    #[test]
    fn test_config_parsing_both_source_and_exclude_keys() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": "|",
            "source_keys": ["key1"],
            "exclude_keys": ["key2"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("cannot specify both"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }

    #[test]
    fn test_config_parsing_invalid_source_keys_not_array() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": "|",
            "source_keys": "not_an_array"
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("source_keys must be an array"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }

    #[test]
    fn test_config_parsing_invalid_exclude_keys_not_array() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": "|",
            "exclude_keys": "not_an_array"
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("exclude_keys must be an array"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }
}
