// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NackMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;

/// URN identifier for the Condense Attributes processor
pub const CONDENSE_ATTRIBUTES_PROCESSOR_URN: &str = "urn:otel:condense_attributes:processor";

/// Configuration for the Condense Attributes processor
///
/// `destination_key`: The key under which the condensed attributes will be stored.
/// `delimiter`: The character used to separate individual attribute values in the condensed string.
///                Note: Cannot be '=' as it would create ambiguous output (e.g., k1=v1=k2=v2).
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

impl Config {
    /// Creates a new Config instance from configuration
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

        // Validate that delimiter is not '=' as it would create ambiguous output
        if delimiter == '=' {
            return Err(ConfigError::InvalidUserConfig {
                error: "delimiter cannot be '=' as it would create ambiguous output (e.g., k1=v1=k2=v2)".to_string(),
            });
        }

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

        // Validate that destination_key is not in source_keys
        if let Some(ref keys) = source_keys {
            if keys.contains(&destination_key) {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "destination_key '{}' cannot be included in source_keys",
                        destination_key
                    ),
                });
            }
        }

        // Validate that destination_key is not in exclude_keys
        if let Some(ref keys) = exclude_keys {
            if keys.contains(&destination_key) {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "destination_key '{}' cannot be included in exclude_keys",
                        destination_key
                    ),
                });
            }
        }

        Ok(Self {
            destination_key,
            delimiter,
            source_keys,
            exclude_keys,
        })
    }
}

/// Processor that condenses multiple attributes into a single attribute based on predefined rules.
pub struct CondenseAttributesProcessor {
    config: Config,
}

enum CachedAttributeValue {
    Str(String),
    Int(i64),
    Double(f64),
    Bool(bool),
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
    let processor = CondenseAttributesProcessor::from_config(&node_config.config)?;

    otel_info!("condense_attributes_processor.ready");

    Ok(ProcessorWrapper::local(
        processor,
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
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: |config| Config::from_config(config).map(|_| ()),
    };

impl CondenseAttributesProcessor {
    /// Creates a new CondenseAttributesProcessor instance
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Creates a new CondenseAttributesProcessor instance from configuration
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        Ok(Self::new(Config::from_config(config)?))
    }

    /// Condenses attributes in the given record batch according to the processor configuration.
    /// Returns the number of individual attributes that were condensed.
    fn condense(&mut self, records: &mut OtapArrowRecords) -> Result<u64, Error> {
        let rb = match records.get(ArrowPayloadType::LogAttrs) {
            Some(rb) => rb,
            None => {
                otel_debug!("condense_attributes_processor.no_log_attrs_payload");
                return Ok(0);
            }
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

        // Pre-extract key arrays
        let key_str_arr = key_col.as_any().downcast_ref::<StringArray>();
        let (key_dict_u8_keys, key_dict_u8_vals) = if let Some(dict) = key_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>(
        ) {
            let vals = dict
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    engine_err("dictionary values are not StringArray for UInt8 key type")
                })?;
            (Some(dict.keys()), Some(vals))
        } else {
            (None, None)
        };
        let (key_dict_u16_keys, key_dict_u16_vals) = if let Some(dict) = key_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>(
        ) {
            let vals = dict
                .values()
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    engine_err("dictionary values are not StringArray for UInt16 key type")
                })?;
            (Some(dict.keys()), Some(vals))
        } else {
            (None, None)
        };

        // Helper function to get key at index i
        let get_key = |i: usize| -> Option<&str> {
            if let Some(arr) = key_str_arr {
                return if arr.is_null(i) {
                    None
                } else {
                    Some(arr.value(i))
                };
            }
            if let (Some(keys), Some(vals)) = (key_dict_u8_keys, key_dict_u8_vals) {
                return if keys.is_null(i) {
                    None
                } else {
                    Some(vals.value(keys.value(i) as usize))
                };
            }
            if let (Some(keys), Some(vals)) = (key_dict_u16_keys, key_dict_u16_vals) {
                return if keys.is_null(i) {
                    None
                } else {
                    Some(vals.value(keys.value(i) as usize))
                };
            }
            None
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

        // parent_to_attrs uses borrowed &str keys from Arrow arrays, so cannot be easily reused across calls.
        // TODO: is reusing a HashMap<u16, Vec<(String, String)>> worth it?
        let mut parent_to_attrs: HashMap<u16, Vec<(&str, String)>> = HashMap::new();
        let mut preserved_attrs: Vec<(u16, &str, CachedAttributeValue)> =
            Vec::with_capacity(num_rows);
        let mut removed_existing_destination = false;
        let mut removed_existing_destination_count = 0u64;

        for i in 0..num_rows {
            if parent_id_arr.is_null(i) {
                continue;
            }
            let parent_id = parent_id_arr.value(i);

            let key = match get_key(i) {
                Some(k) => k,
                None => continue,
            };

            // Always skip attributes that match the destination_key to prevent circular references
            if key == self.config.destination_key {
                removed_existing_destination = true;
                removed_existing_destination_count += 1;
                continue;
            }

            // Check if we should include this key
            let should_condense = match (&self.config.source_keys, &self.config.exclude_keys) {
                (Some(source), _) => source.contains(key),
                (None, Some(exclude)) => !exclude.contains(key),
                (None, None) => true,
            };

            if !should_condense {
                if type_arr.is_null(i) {
                    continue;
                }

                let value_type = type_arr.value(i);
                if let Ok(value_type_enum) = AttributeValueType::try_from(value_type) {
                    let cached_value = match value_type_enum {
                        AttributeValueType::Str => str_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &StringArray, index| {
                                arr.value(index).to_string()
                            })
                            .map(CachedAttributeValue::Str)
                        }),
                        AttributeValueType::Int => int_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &Int64Array, index| {
                                arr.value(index)
                            })
                            .map(CachedAttributeValue::Int)
                        }),
                        AttributeValueType::Double => double_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &Float64Array, index| {
                                arr.value(index)
                            })
                            .map(CachedAttributeValue::Double)
                        }),
                        AttributeValueType::Bool => bool_col.and_then(|col| {
                            Self::extract_value_from_column(col, i, |arr: &BooleanArray, index| {
                                arr.value(index)
                            })
                            .map(CachedAttributeValue::Bool)
                        }),
                        _ => None,
                    };

                    if let Some(cached_value) = cached_value {
                        preserved_attrs.push((parent_id, key, cached_value));
                    }
                }

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
                        .or_default()
                        .push((key, val));
                }
            }
        }

        // No-op fast path: no rows condensed and no destination rows removed.
        if parent_to_attrs.is_empty() && !removed_existing_destination {
            otel_debug!(
                "condense_attributes_processor.no_condensing_needed",
                num_rows
            );
            return Ok(0);
        }

        if removed_existing_destination_count > 0 {
            otel_debug!(
                "condense_attributes_processor.destination_key_replaced",
                dropped_existing_destination_items = removed_existing_destination_count
            );
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
        for (parent_id, key, value) in preserved_attrs {
            builder.append_parent_id(&parent_id);
            builder.append_key(key);

            match value {
                CachedAttributeValue::Str(val) => {
                    builder.any_values_builder.append_str(val.as_bytes());
                }
                CachedAttributeValue::Int(val) => {
                    builder.any_values_builder.append_int(val);
                }
                CachedAttributeValue::Double(val) => {
                    builder.any_values_builder.append_double(val);
                }
                CachedAttributeValue::Bool(val) => {
                    builder.any_values_builder.append_bool(val);
                }
            }
        }

        // TODO: This rebuild path is copy-heavy.
        // `RecordBatch` is immutable, so true in-place mutation is not possible today.
        // A more efficient approach could be:
        // - filter/delete condensed source rows from the existing batch,
        // - append computed condensed rows,
        // - concatenate/reconcile schemas as needed.
        // Note: `transform.rs` insert only supports predefined literal values; condense requires
        // per-parent computed values, so we cannot reuse that insert path directly.
        // Follow-up optimization tracked in https://github.com/open-telemetry/otel-arrow/issues/1694
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
            Message::Control(control_msg) => {
                use otap_df_engine::control::NodeControlMsg;
                match control_msg {
                    NodeControlMsg::Config { config } => {
                        match Config::from_config(&config) {
                            Ok(new_config) => {
                                otel_info!("condense_attributes_processor.reconfigured");
                                self.config = new_config;
                            }
                            Err(e) => {
                                otel_warn!(
                                    "condense_attributes_processor.reconfigure_error",
                                    message = %e
                                );
                            }
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                let (context, payload) = pdata.into_parts();
                let saved_payload = if context.may_return_payload() {
                    payload.clone()
                } else {
                    OtapPayload::empty(signal)
                };

                let mut records: OtapArrowRecords = payload.try_into()?;

                let input_items = records.num_items() as u64;

                otel_debug!("condense_attributes_processor.processing", input_items);

                let result = match signal {
                    SignalType::Logs => self.condense(&mut records),
                    _ => Err(Error::InternalError {
                        message: "CondenseAttributesProcessor only supported for SignalType 'Logs'"
                            .to_string(),
                    }),
                };

                match result {
                    Ok(condensed) => {
                        let output_items = records.num_items() as u64;
                        otel_debug!(
                            "condense_attributes_processor.success",
                            input_items,
                            output_items,
                            condensed_items = condensed
                        );
                        effect_handler
                            .send_message(OtapPdata::new(context, records.into()))
                            .await?;
                        Ok(())
                    }
                    Err(e) => {
                        let message = e.to_string();
                        otel_error!(
                            "condense_attributes_processor.failure",
                            input_items,
                            signal = ?signal,
                            message,
                        );

                        effect_handler
                            .notify_nack(NackMsg::new(
                                message,
                                OtapPdata::new(context, saved_payload),
                            ))
                            .await?;
                        Err(e)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod condense_tests {
    use super::*;
    use bytes::BytesMut;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::control::PipelineControlMsg;
    use otap_df_engine::control::pipeline_ctrl_msg_channel;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::TestCallData;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        collector::metrics::v1::ExportMetricsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
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

    fn run_condense_test<F>(input: ExportLogsServiceRequest, cfg: Value, validate: F)
    where
        F: FnOnce(ExportLogsServiceRequest) + 'static,
    {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

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
                let (_, first) = out.into_iter().next().expect("one output").into_parts();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                validate(decoded);
            })
            .validate(|_| async move {});
    }

    fn test_condense_single_log(
        input: ExportLogsServiceRequest,
        cfg: Value,
        expected_attrs: Vec<KeyValue>,
    ) {
        run_condense_test(input, cfg, move |decoded| {
            let log_attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
            assert_eq!(log_attrs, &expected_attrs);
        });
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

        test_condense_single_log(input, cfg, expected_attrs);
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

        test_condense_single_log(input, cfg, expected_attrs);
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

        test_condense_single_log(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_no_attributes() {
        let input = build_log_with_attrs(vec![]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });

        let expected_attrs = vec![];

        test_condense_single_log(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_no_op_fast_path_keeps_log_attrs_batch() {
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

        let mut processor = CondenseAttributesProcessor::from_config(&cfg).expect("valid config");

        let mut bytes = BytesMut::new();
        input.encode(&mut bytes).expect("encode input");
        let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into();
        let mut records: OtapArrowRecords = payload.try_into().expect("convert to records");

        let before_batch = records
            .get(ArrowPayloadType::LogAttrs)
            .expect("log attrs batch present");
        let before_num_rows = before_batch.num_rows();
        let before_num_items = records.num_items();

        let condensed_count = processor
            .condense(&mut records)
            .expect("no-op condense should succeed");

        assert_eq!(condensed_count, 0);
        assert_eq!(records.num_items(), before_num_items);

        let after_batch = records
            .get(ArrowPayloadType::LogAttrs)
            .expect("log attrs batch present after condense");
        assert_eq!(after_batch.num_rows(), before_num_rows);
    }

    #[test]
    fn test_condense_no_log_attrs_payload_returns_zero() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });
        let mut processor = CondenseAttributesProcessor::from_config(&cfg).expect("valid config");
        let mut records = OtapArrowRecords::from(Logs::default());

        let condensed_count = processor
            .condense(&mut records)
            .expect("condense should succeed without log attrs payload");

        assert_eq!(condensed_count, 0);
        assert!(records.get(ArrowPayloadType::LogAttrs).is_none());
        assert_eq!(records.num_items(), 0);
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

        test_condense_single_log(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_destination_key_already_exists_ignores() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("condensed", AnyValue::new_string("old_value")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });

        let expected_attrs = vec![KeyValue::new(
            "condensed",
            AnyValue::new_string("attr1=value1;attr2=42"),
        )];

        test_condense_single_log(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_destination_key_not_in_source_keys_ignores() {
        let input = build_log_with_attrs(vec![
            KeyValue::new("attr1", AnyValue::new_string("value1")),
            KeyValue::new("condensed", AnyValue::new_string("old_value")),
            KeyValue::new("attr2", AnyValue::new_int(42)),
        ]);

        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["attr1", "attr2"]
        });

        let expected_attrs = vec![KeyValue::new(
            "condensed",
            AnyValue::new_string("attr1=value1;attr2=42"),
        )];

        test_condense_single_log(input, cfg, expected_attrs);
    }

    #[test]
    fn test_condense_multiple_logs() {
        // Build input with multiple log records, each with different attributes
        let input = ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                ..Default::default()
            },
            vec![ScopeLogs::new(
                InstrumentationScope {
                    ..Default::default()
                },
                vec![
                    // Log 1: Has all source_keys (user_id, session_id) plus extra attributes
                    LogRecord::build()
                        .time_unix_nano(1u64)
                        .severity_number(SeverityNumber::Info)
                        .event_name("login")
                        .attributes(vec![
                            KeyValue::new("user_id", AnyValue::new_string("user123")),
                            KeyValue::new("session_id", AnyValue::new_string("sess456")),
                            KeyValue::new("ip_address", AnyValue::new_string("192.168.1.1")),
                            KeyValue::new("timestamp", AnyValue::new_int(1234567890)),
                        ])
                        .finish(),
                    // Log 2: Has only one source_key (user_id) plus different extra attributes
                    LogRecord::build()
                        .time_unix_nano(2u64)
                        .severity_number(SeverityNumber::Warn)
                        .event_name("api_call")
                        .attributes(vec![
                            KeyValue::new("user_id", AnyValue::new_string("user789")),
                            KeyValue::new("endpoint", AnyValue::new_string("/api/v1/data")),
                            KeyValue::new("method", AnyValue::new_string("POST")),
                            KeyValue::new("status_code", AnyValue::new_int(200)),
                            KeyValue::new("duration_ms", AnyValue::new_double(45.3)),
                        ])
                        .finish(),
                    // Log 3: Has session_id but not user_id, plus other attributes
                    LogRecord::build()
                        .time_unix_nano(3u64)
                        .severity_number(SeverityNumber::Error)
                        .event_name("error")
                        .attributes(vec![
                            KeyValue::new("session_id", AnyValue::new_string("sess999")),
                            KeyValue::new("error_code", AnyValue::new_int(500)),
                            KeyValue::new(
                                "error_msg",
                                AnyValue::new_string("Internal Server Error"),
                            ),
                            KeyValue::new("retry", AnyValue::new_bool(true)),
                        ])
                        .finish(),
                    // Log 4: Has neither source_key, only non-source attributes
                    LogRecord::build()
                        .time_unix_nano(4u64)
                        .severity_number(SeverityNumber::Debug)
                        .event_name("cache_hit")
                        .attributes(vec![
                            KeyValue::new("cache_key", AnyValue::new_string("key_abc")),
                            KeyValue::new("hit", AnyValue::new_bool(true)),
                        ])
                        .finish(),
                ],
            )],
        )]);

        let cfg = json!({
            "destination_key": "user_session",
            "delimiter": "|",
            "source_keys": ["user_id", "session_id"]
        });

        run_condense_test(input, cfg, |decoded| {
            let has_attr =
                |attrs: &Vec<KeyValue>, key: &str| -> bool { attrs.iter().any(|kv| kv.key == key) };
            let get_string_val = |attrs: &Vec<KeyValue>, key: &str| -> String {
                attrs
                    .iter()
                    .find(|kv| kv.key == key)
                    .and_then(|kv| kv.value.as_ref())
                    .and_then(|v| {
                        if let Some(any_value::Value::StringValue(s)) = &v.value {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| panic!("Expected string value for key {}", key))
            };

            let log_records = &decoded.resource_logs[0].scope_logs[0].log_records;
            assert_eq!(log_records.len(), 4, "Should have 4 log records");

            // Log 1: Both user_id and session_id condensed, other attrs preserved
            let log1_attrs = &log_records[0].attributes;
            assert_eq!(log1_attrs.len(), 3, "Log 1 should have 3 attributes");
            assert!(
                has_attr(log1_attrs, "user_session"),
                "Log 1 should have user_session"
            );
            let condensed_str = get_string_val(log1_attrs, "user_session");
            assert!(
                condensed_str.contains("user_id=user123"),
                "Log 1 condensed should contain user_id=user123, got: {}",
                condensed_str
            );
            assert!(
                condensed_str.contains("session_id=sess456"),
                "Log 1 condensed should contain session_id=sess456, got: {}",
                condensed_str
            );
            assert!(
                has_attr(log1_attrs, "ip_address"),
                "Log 1 should preserve ip_address"
            );
            assert!(
                has_attr(log1_attrs, "timestamp"),
                "Log 1 should preserve timestamp"
            );

            // Log 2: Only user_id condensed (session_id not present), other attrs preserved
            let log2_attrs = &log_records[1].attributes;
            assert_eq!(log2_attrs.len(), 5, "Log 2 should have 5 attributes");
            assert!(
                has_attr(log2_attrs, "user_session"),
                "Log 2 should have user_session"
            );
            let condensed_str = get_string_val(log2_attrs, "user_session");
            assert_eq!(
                condensed_str, "user_id=user789",
                "Log 2 should only condense user_id, got: {}",
                condensed_str
            );
            assert!(
                has_attr(log2_attrs, "endpoint"),
                "Log 2 should preserve endpoint"
            );
            assert!(
                has_attr(log2_attrs, "method"),
                "Log 2 should preserve method"
            );
            assert!(
                has_attr(log2_attrs, "status_code"),
                "Log 2 should preserve status_code"
            );
            assert!(
                has_attr(log2_attrs, "duration_ms"),
                "Log 2 should preserve duration_ms"
            );

            // Log 3: Only session_id condensed (user_id not present), other attrs preserved
            let log3_attrs = &log_records[2].attributes;
            assert_eq!(log3_attrs.len(), 4, "Log 3 should have 4 attributes");
            assert!(
                has_attr(log3_attrs, "user_session"),
                "Log 3 should have user_session"
            );
            let condensed_str = get_string_val(log3_attrs, "user_session");
            assert_eq!(
                condensed_str, "session_id=sess999",
                "Log 3 should only condense session_id, got: {}",
                condensed_str
            );
            assert!(
                has_attr(log3_attrs, "error_code"),
                "Log 3 should preserve error_code"
            );
            assert!(
                has_attr(log3_attrs, "error_msg"),
                "Log 3 should preserve error_msg"
            );
            assert!(has_attr(log3_attrs, "retry"), "Log 3 should preserve retry");

            // Log 4: No source_keys present, all attributes preserved, no condensed attr
            let log4_attrs = &log_records[3].attributes;
            assert_eq!(log4_attrs.len(), 2, "Log 4 should have 2 attributes");
            assert!(
                !has_attr(log4_attrs, "user_session"),
                "Log 4 should not have user_session"
            );
            assert!(
                has_attr(log4_attrs, "cache_key"),
                "Log 4 should preserve cache_key"
            );
            assert!(has_attr(log4_attrs, "hit"), "Log 4 should preserve hit");
        });
    }

    #[test]
    fn test_nack_preserves_original_payload_for_unsupported_signal() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("condense-attributes-processor-nack-preserve-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config =
            NodeUserConfig::new_processor_config(CONDENSE_ATTRIBUTES_PROCESSOR_URN);
        node_config.config = json!({
            "destination_key": "condensed",
            "delimiter": ";"
        });

        let proc = create_condense_attributes_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .expect("create processor");

        rt.set_processor(proc)
            .run_test(|mut ctx| async move {
                let (pipeline_ctrl_tx, mut pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_ctrl_tx);

                let mut bytes = BytesMut::new();
                ExportMetricsServiceRequest::default()
                    .encode(&mut bytes)
                    .expect("encode metrics request");

                let pdata_in = OtapPdata::new_default(
                    OtlpProtoBytes::ExportMetricsRequest(bytes.freeze()).into(),
                )
                .test_subscribe_to(
                    Interests::NACKS,
                    TestCallData::default().into(),
                    777,
                );

                let result = ctx.process(Message::PData(pdata_in)).await;
                assert!(result.is_err(), "unsupported signal should return error");

                match pipeline_ctrl_rx.recv().await.expect("pipeline msg") {
                    PipelineControlMsg::DeliverNack { nack, node_id } => {
                        assert_eq!(node_id, 777);
                        assert_eq!(nack.refused.signal_type(), SignalType::Metrics);
                    }
                    other => panic!("expected DeliverNack, got: {other:?}"),
                }
            })
            .validate(|_| async move {});
    }

    #[test]
    fn test_control_reconfigure_applies_valid_and_ignores_invalid_config() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let node = test_node("condense-attributes-processor-reconfigure-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config =
            NodeUserConfig::new_processor_config(CONDENSE_ATTRIBUTES_PROCESSOR_URN);
        node_config.config = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["attr1", "attr2"]
        });

        let proc = create_condense_attributes_processor(
            pipeline_ctx,
            node,
            Arc::new(node_config),
            rt.config(),
        )
        .expect("create processor");

        rt.set_processor(proc)
            .run_test(|mut ctx| async move {
                let input = build_log_with_attrs(vec![
                    KeyValue::new("attr1", AnyValue::new_string("value1")),
                    KeyValue::new("attr2", AnyValue::new_int(42)),
                    KeyValue::new("attr3", AnyValue::new_bool(true)),
                ]);

                // Valid reconfiguration should update destination key and delimiter.
                ctx.process(Message::Control(NodeControlMsg::Config {
                    config: json!({
                        "destination_key": "merged",
                        "delimiter": "|",
                        "source_keys": ["attr1", "attr2"]
                    }),
                }))
                .await
                .expect("valid reconfig control message should succeed");

                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode first input");
                let pdata_in = OtapPdata::new_default(
                    OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into(),
                );
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process after valid reconfig");

                let out = ctx.drain_pdata().await;
                let (_, first_payload) = out.into_iter().next().expect("one output").into_parts();
                let otlp_bytes: OtlpProtoBytes = first_payload.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");
                let attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                let merged = attrs
                    .iter()
                    .find(|kv| kv.key == "merged")
                    .expect("merged attribute should exist after valid reconfig");
                let merged_val = merged
                    .value
                    .as_ref()
                    .and_then(|v| match &v.value {
                        Some(any_value::Value::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .expect("merged should be a string value");
                assert_eq!(merged_val, "attr1=value1|attr2=42");

                // Invalid reconfiguration should be ignored, keeping previous valid config.
                ctx.process(Message::Control(NodeControlMsg::Config {
                    config: json!({
                        "destination_key": "ignored",
                        "source_keys": ["attr1", "attr2"]
                    }),
                }))
                .await
                .expect("invalid reconfig control message should not fail processing");

                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode second input");
                let pdata_in = OtapPdata::new_default(
                    OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into(),
                );
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process after invalid reconfig");

                let out = ctx.drain_pdata().await;
                let (_, second_payload) = out.into_iter().next().expect("one output").into_parts();
                let otlp_bytes: OtlpProtoBytes =
                    second_payload.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");
                let attrs = &decoded.resource_logs[0].scope_logs[0].log_records[0].attributes;
                let merged_after_invalid = attrs
                    .iter()
                    .find(|kv| kv.key == "merged")
                    .expect("previous valid config should remain active after invalid reconfig");
                let merged_after_invalid_val = merged_after_invalid
                    .value
                    .as_ref()
                    .and_then(|v| match &v.value {
                        Some(any_value::Value::StringValue(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .expect("merged should be a string value");
                assert_eq!(merged_after_invalid_val, "attr1=value1|attr2=42");
                assert!(attrs.iter().all(|kv| kv.key != "ignored"));
            })
            .validate(|_| async move {});
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
    fn test_config_parsing_delimiter_cannot_be_equals() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": "=",
            "source_keys": ["key1", "key2"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("delimiter cannot be '='"));
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

    #[test]
    fn test_config_parsing_destination_key_in_source_keys() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "source_keys": ["attr1", "condensed"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("destination_key"));
                assert!(error.contains("source_keys"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }

    #[test]
    fn test_config_parsing_destination_key_in_exclude_keys() {
        let cfg = json!({
            "destination_key": "condensed",
            "delimiter": ";",
            "exclude_keys": ["attr1", "condensed"]
        });

        let result = CondenseAttributesProcessor::from_config(&cfg);
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidUserConfig { error }) => {
                assert!(error.contains("destination_key"));
                assert!(error.contains("exclude_keys"));
            }
            _ => panic!("expected InvalidUserConfig error"),
        }
    }
}
