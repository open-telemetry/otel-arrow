// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resource Validator Processor
//!
//! This processor validates that a required resource attribute exists and its value
//! is in an allowed list. Requests that fail validation are permanently NACKed,
//! enabling clients to detect misconfiguration immediately rather than having data
//! silently dropped.
//!
//! # Use Case
//!
//! In multi-tenant Azure environments, telemetry includes a `microsoft.resourceId`
//! resource attribute containing the Azure Resource Manager (ARM) resource ID.
//! This processor validates:
//! 1. The attribute exists on the Resource
//! 2. The value is in the allowed list
//! 3. Rejects with permanent NACK on failure (HTTP 400 / gRPC INVALID_ARGUMENT)
//!
//! # Configuration
//!
//! ```yaml
//! processors:
//!   resource_validator:
//!     required_attribute: "microsoft.resourceId"
//!     allowed_values:
//!       - "/subscriptions/xxx/resourceGroups/yyy/..."
//!     case_insensitive: true
//! ```
//!
//! # Extensibility
//!
//! The processor is designed to be extensible for future dynamic validation:
//!
//! - **Static config (current)**: Uses `allowed_values` from configuration
//! - **Dynamic auth context (future)**: When SAT auth extension is ready, the
//!   processor can read allowed values from request context via `AllowedValuesSource::Dynamic`
//!
//! The `get_allowed_values` method provides the extension point - it currently
//! returns static config values, but can be extended to check auth context first.

mod config;
mod metrics;

pub use config::Config;
pub use metrics::ResourceValidatorMetrics;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata::views::logs::{LogsDataView, ResourceLogsView};
use otap_df_pdata::views::metrics::{MetricsView, ResourceMetricsView};
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otap_df_pdata::views::otlp::bytes::traces::RawTraceData;
use otap_df_pdata::views::resource::ResourceView;
use otap_df_pdata::views::trace::{ResourceSpansView, TracesView};
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;

/// URN identifier for the Resource Validator processor
pub const RESOURCE_VALIDATOR_PROCESSOR_URN: &str = "urn:otel:resource_validator:processor";

/// Source of allowed values for validation.
///
/// This enum enables extensibility for future dynamic auth context support.
/// Currently only `Static` is implemented, but `Dynamic` provides the extension
/// point for SAT auth integration.
#[derive(Debug, Clone)]
pub enum AllowedValuesSource {
    /// Static allowed values from configuration.
    /// These are pre-normalized (lowercased if case_insensitive).
    Static(HashSet<String>),

    /// Dynamic allowed values from auth context (future).
    /// When SAT auth extension is ready, this variant will be used to
    /// indicate that allowed values should be read from the request context.
    ///
    /// The optional HashSet provides fallback values from config if auth
    /// context is not available for a particular request.
    #[allow(dead_code)]
    Dynamic {
        /// Fallback values from config (used when auth context is unavailable)
        fallback: HashSet<String>,
    },
}

impl AllowedValuesSource {
    /// Returns true if validation should only check presence (no allowed list).
    /// This is useful for future dynamic mode where we might want to check
    /// if any allowed values are configured before attempting validation.
    #[allow(dead_code)]
    fn is_presence_only(&self) -> bool {
        match self {
            AllowedValuesSource::Static(values) => values.is_empty(),
            AllowedValuesSource::Dynamic { fallback } => fallback.is_empty(),
        }
    }

    /// Gets the static/fallback allowed values set.
    fn get_static_values(&self) -> &HashSet<String> {
        match self {
            AllowedValuesSource::Static(values) => values,
            AllowedValuesSource::Dynamic { fallback } => fallback,
        }
    }
}

/// Validation result indicating why validation failed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationFailure {
    /// The required attribute is missing from the resource
    MissingAttribute,
    /// The attribute value is not a string type
    InvalidAttributeType,
    /// The attribute value is not in the allowed list
    NotInAllowedList,
}

impl std::fmt::Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationFailure::MissingAttribute => write!(f, "missing"),
            ValidationFailure::InvalidAttributeType => write!(f, "invalid_type"),
            ValidationFailure::NotInAllowedList => write!(f, "not_allowed"),
        }
    }
}

/// Resource Validator Processor
///
/// Validates that telemetry data contains the required resource attribute
/// with a value from the allowed list.
///
/// # Extensibility
///
/// The processor is designed to support both static configuration and future
/// dynamic auth context validation:
///
/// - `allowed_values_source`: Determines where allowed values come from
/// - `get_allowed_values()`: Extension point for per-request allowed values
pub struct ResourceValidatorProcessor {
    /// The attribute key to validate
    required_attribute: String,
    /// Source of allowed values (static config or dynamic auth)
    allowed_values_source: AllowedValuesSource,
    /// Whether to perform case-insensitive comparison
    case_insensitive: bool,
    /// Telemetry metrics
    metrics: MetricSet<ResourceValidatorMetrics>,
}

/// Factory function to create a Resource Validator processor
pub fn create_resource_validator_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        ResourceValidatorProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register ResourceValidatorProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static RESOURCE_VALIDATOR_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: RESOURCE_VALIDATOR_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_resource_validator_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

impl ResourceValidatorProcessor {
    /// Creates a new ResourceValidatorProcessor from configuration
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<ResourceValidatorMetrics>();
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        config.validate()?;

        Ok(Self {
            required_attribute: config.required_attribute.clone(),
            allowed_values_source: AllowedValuesSource::Static(config.allowed_values_set()),
            case_insensitive: config.case_insensitive,
            metrics,
        })
    }

    /// Creates a new ResourceValidatorProcessor with explicit configuration
    #[must_use]
    #[cfg(test)]
    pub fn new(
        required_attribute: String,
        allowed_values: HashSet<String>,
        case_insensitive: bool,
        pipeline_ctx: PipelineContext,
    ) -> Self {
        let metrics = pipeline_ctx.register_metrics::<ResourceValidatorMetrics>();
        Self {
            required_attribute,
            allowed_values_source: AllowedValuesSource::Static(allowed_values),
            case_insensitive,
            metrics,
        }
    }

    /// Gets the allowed values for validation.
    ///
    /// This is the extension point for future dynamic auth context support.
    /// Currently returns values from the configured source (static config).
    ///
    /// # Future Enhancement
    ///
    /// When SAT auth extension is ready, this method can be extended to:
    /// 1. Check if `pdata` context contains auth information
    /// 2. Extract allowed resource IDs from auth claims
    /// 3. Fall back to static config if auth context is unavailable
    ///
    /// Example future implementation:
    /// ```ignore
    /// fn get_allowed_values<'a>(&'a self, pdata: &'a OtapPdata) -> Cow<'a, HashSet<String>> {
    ///     match &self.allowed_values_source {
    ///         AllowedValuesSource::Dynamic { fallback } => {
    ///             // Try to get from auth context
    ///             if let Some(auth) = pdata.context().auth() {
    ///                 if let Some(resource_ids) = auth.get_resource_ids() {
    ///                     return Cow::Owned(self.normalize_values(resource_ids));
    ///                 }
    ///             }
    ///             // Fall back to config
    ///             Cow::Borrowed(fallback)
    ///         }
    ///         AllowedValuesSource::Static(values) => Cow::Borrowed(values),
    ///     }
    /// }
    /// ```
    fn get_allowed_values(&self, _pdata: &OtapPdata) -> Cow<'_, HashSet<String>> {
        // Currently just returns the static/fallback values.
        // When auth context is available, this will check pdata.context().auth() first.
        Cow::Borrowed(self.allowed_values_source.get_static_values())
    }

    /// Validates a single resource's attributes against the provided allowed values.
    fn validate_resource_with_allowed<R: ResourceView>(
        &self,
        resource: &R,
        allowed_values: &HashSet<String>,
    ) -> Result<(), ValidationFailure> {
        let required_key = self.required_attribute.as_bytes();

        // Find the required attribute
        for attr in resource.attributes() {
            if attr.key() == required_key {
                // Found the attribute, validate it's a string
                let Some(value) = attr.value() else {
                    return Err(ValidationFailure::InvalidAttributeType);
                };

                if value.value_type() != ValueType::String {
                    return Err(ValidationFailure::InvalidAttributeType);
                }

                let Some(str_bytes) = value.as_string() else {
                    return Err(ValidationFailure::InvalidAttributeType);
                };

                // Convert bytes to string (may fail for invalid UTF-8)
                let str_value = std::str::from_utf8(str_bytes)
                    .map_err(|_| ValidationFailure::InvalidAttributeType)?;

                // Skip allowed list check if no values are configured (presence-only validation)
                if allowed_values.is_empty() {
                    return Ok(());
                }

                // Check if value is in allowed list
                let lookup_value = if self.case_insensitive {
                    str_value.to_lowercase()
                } else {
                    str_value.to_string()
                };

                if allowed_values.contains(&lookup_value) {
                    return Ok(());
                } else {
                    return Err(ValidationFailure::NotInAllowedList);
                }
            }
        }

        // Attribute not found
        Err(ValidationFailure::MissingAttribute)
    }

    /// Validates all resources in logs data
    fn validate_logs(
        &self,
        data: &RawLogsData<'_>,
        allowed_values: &HashSet<String>,
    ) -> Result<(), (ValidationFailure, String)> {
        for resource_logs in data.resources() {
            if let Some(resource) = resource_logs.resource() {
                if let Err(failure) = self.validate_resource_with_allowed(&resource, allowed_values)
                {
                    return Err((failure, self.format_error_message(failure)));
                }
            } else {
                // No resource means the attribute is missing
                let failure = ValidationFailure::MissingAttribute;
                return Err((failure, self.format_error_message(failure)));
            }
        }
        Ok(())
    }

    /// Validates all resources in metrics data
    fn validate_metrics(
        &self,
        data: &RawMetricsData<'_>,
        allowed_values: &HashSet<String>,
    ) -> Result<(), (ValidationFailure, String)> {
        for resource_metrics in data.resources() {
            if let Some(resource) = resource_metrics.resource() {
                if let Err(failure) = self.validate_resource_with_allowed(&resource, allowed_values)
                {
                    return Err((failure, self.format_error_message(failure)));
                }
            } else {
                let failure = ValidationFailure::MissingAttribute;
                return Err((failure, self.format_error_message(failure)));
            }
        }
        Ok(())
    }

    /// Validates all resources in traces data
    fn validate_traces(
        &self,
        data: &RawTraceData<'_>,
        allowed_values: &HashSet<String>,
    ) -> Result<(), (ValidationFailure, String)> {
        for resource_spans in data.resources() {
            if let Some(resource) = resource_spans.resource() {
                if let Err(failure) = self.validate_resource_with_allowed(&resource, allowed_values)
                {
                    return Err((failure, self.format_error_message(failure)));
                }
            } else {
                let failure = ValidationFailure::MissingAttribute;
                return Err((failure, self.format_error_message(failure)));
            }
        }
        Ok(())
    }

    /// Validates all resources in Arrow logs data
    fn validate_arrow_logs(
        &self,
        arrow_records: &OtapArrowRecords,
        allowed_values: &HashSet<String>,
    ) -> Result<(), (ValidationFailure, String)> {
        let logs_view = OtapLogsView::try_from(arrow_records).map_err(|_| {
            let failure = ValidationFailure::MissingAttribute;
            (failure, self.format_error_message(failure))
        })?;

        for resource_logs in logs_view.resources() {
            if let Some(resource) = resource_logs.resource() {
                if let Err(failure) = self.validate_resource_with_allowed(&resource, allowed_values)
                {
                    return Err((failure, self.format_error_message(failure)));
                }
            } else {
                let failure = ValidationFailure::MissingAttribute;
                return Err((failure, self.format_error_message(failure)));
            }
        }
        Ok(())
    }

    /// Formats an error message for the NACK response
    fn format_error_message(&self, failure: ValidationFailure) -> String {
        match failure {
            ValidationFailure::MissingAttribute => {
                format!(
                    "required resource attribute '{}' is missing from telemetry data",
                    self.required_attribute
                )
            }
            ValidationFailure::InvalidAttributeType => {
                format!(
                    "resource attribute '{}' must be a string",
                    self.required_attribute
                )
            }
            ValidationFailure::NotInAllowedList => {
                format!(
                    "resource attribute '{}' value is not in the allowed list",
                    self.required_attribute
                )
            }
        }
    }

    /// Updates metrics based on validation result
    fn update_metrics(&mut self, result: &Result<(), (ValidationFailure, String)>) {
        match result {
            Ok(()) => {
                self.metrics.batches_accepted.add(1);
            }
            Err((failure, _)) => match failure {
                ValidationFailure::MissingAttribute | ValidationFailure::InvalidAttributeType => {
                    self.metrics.batches_rejected_missing.add(1);
                }
                ValidationFailure::NotInAllowedList => {
                    self.metrics.batches_rejected_not_allowed.add(1);
                }
            },
        }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for ResourceValidatorProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(control) => {
                if let NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } = control
                {
                    let _ = metrics_reporter.report(&mut self.metrics);
                }
                Ok(())
            }
            Message::PData(pdata) => {
                let signal_type = pdata.signal_type();

                // Get allowed values (extension point for future dynamic auth)
                let allowed_values = self.get_allowed_values(&pdata);

                // Validate based on payload type
                let validation_result = match pdata.payload_ref() {
                    OtapPayload::OtlpBytes(otlp_bytes) => match (signal_type, otlp_bytes) {
                        (SignalType::Logs, OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                            let logs_data = RawLogsData::new(bytes.as_ref());
                            self.validate_logs(&logs_data, &allowed_values)
                        }
                        (SignalType::Metrics, OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
                            let metrics_data = RawMetricsData::new(bytes.as_ref());
                            self.validate_metrics(&metrics_data, &allowed_values)
                        }
                        (SignalType::Traces, OtlpProtoBytes::ExportTracesRequest(bytes)) => {
                            let trace_data = RawTraceData::new(bytes.as_ref());
                            self.validate_traces(&trace_data, &allowed_values)
                        }
                        _ => {
                            // Signal type doesn't match payload type - this shouldn't happen
                            // but pass through rather than fail
                            Ok(())
                        }
                    },
                    OtapPayload::OtapArrowRecords(arrow_records) => match signal_type {
                        SignalType::Logs => {
                            self.validate_arrow_logs(arrow_records, &allowed_values)
                        }
                        // Metrics/Traces Arrow views not yet available - convert to OTLP
                        SignalType::Metrics | SignalType::Traces => {
                            match OtlpProtoBytes::try_from(arrow_records.clone()) {
                                Ok(OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
                                    let data = RawMetricsData::new(bytes.as_ref());
                                    self.validate_metrics(&data, &allowed_values)
                                }
                                Ok(OtlpProtoBytes::ExportTracesRequest(bytes)) => {
                                    let data = RawTraceData::new(bytes.as_ref());
                                    self.validate_traces(&data, &allowed_values)
                                }
                                Ok(_) => Ok(()),
                                Err(_) => {
                                    let failure = ValidationFailure::MissingAttribute;
                                    Err((failure, self.format_error_message(failure)))
                                }
                            }
                        }
                    },
                };

                // Update metrics
                self.update_metrics(&validation_result);

                match validation_result {
                    Ok(()) => {
                        // Validation passed, forward the data
                        effect_handler.send_message(pdata).await?;
                        Ok(())
                    }
                    Err((_, error_msg)) => {
                        // Validation failed, send permanent NACK
                        effect_handler
                            .notify_nack(NackMsg::new_permanent(&error_msg, pdata))
                            .await?;
                        Ok(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use prost::Message as ProstMessage;

    /// Test helper struct for validation testing without metrics
    struct TestValidator {
        required_attribute: String,
        allowed_values: HashSet<String>,
        case_insensitive: bool,
    }

    impl TestValidator {
        fn validate_logs(&self, data: &RawLogsData<'_>) -> Result<(), (ValidationFailure, String)> {
            for resource_logs in data.resources() {
                if let Some(resource) = resource_logs.resource() {
                    if let Err(failure) = self.validate_resource(&resource) {
                        return Err((failure, self.format_error_message(failure)));
                    }
                } else {
                    let failure = ValidationFailure::MissingAttribute;
                    return Err((failure, self.format_error_message(failure)));
                }
            }
            Ok(())
        }

        fn validate_resource<R: ResourceView>(
            &self,
            resource: &R,
        ) -> Result<(), ValidationFailure> {
            let required_key = self.required_attribute.as_bytes();

            for attr in resource.attributes() {
                if attr.key() == required_key {
                    let Some(value) = attr.value() else {
                        return Err(ValidationFailure::InvalidAttributeType);
                    };

                    if value.value_type() != ValueType::String {
                        return Err(ValidationFailure::InvalidAttributeType);
                    }

                    let Some(str_bytes) = value.as_string() else {
                        return Err(ValidationFailure::InvalidAttributeType);
                    };

                    let str_value = std::str::from_utf8(str_bytes)
                        .map_err(|_| ValidationFailure::InvalidAttributeType)?;

                    if self.allowed_values.is_empty() {
                        return Ok(());
                    }

                    let lookup_value = if self.case_insensitive {
                        str_value.to_lowercase()
                    } else {
                        str_value.to_string()
                    };

                    if self.allowed_values.contains(&lookup_value) {
                        return Ok(());
                    } else {
                        return Err(ValidationFailure::NotInAllowedList);
                    }
                }
            }
            Err(ValidationFailure::MissingAttribute)
        }

        fn validate_arrow_logs(
            &self,
            arrow_records: &OtapArrowRecords,
        ) -> Result<(), (ValidationFailure, String)> {
            let logs_view = OtapLogsView::try_from(arrow_records).map_err(|_| {
                let failure = ValidationFailure::MissingAttribute;
                (failure, self.format_error_message(failure))
            })?;

            for resource_logs in logs_view.resources() {
                if let Some(resource) = resource_logs.resource() {
                    if let Err(failure) = self.validate_resource(&resource) {
                        return Err((failure, self.format_error_message(failure)));
                    }
                } else {
                    let failure = ValidationFailure::MissingAttribute;
                    return Err((failure, self.format_error_message(failure)));
                }
            }
            Ok(())
        }

        fn format_error_message(&self, failure: ValidationFailure) -> String {
            match failure {
                ValidationFailure::MissingAttribute => {
                    format!(
                        "required resource attribute '{}' is missing from telemetry data",
                        self.required_attribute
                    )
                }
                ValidationFailure::InvalidAttributeType => {
                    format!(
                        "resource attribute '{}' must be a string",
                        self.required_attribute
                    )
                }
                ValidationFailure::NotInAllowedList => {
                    format!(
                        "resource attribute '{}' value is not in the allowed list",
                        self.required_attribute
                    )
                }
            }
        }
    }

    fn create_logs_request_with_resource(attrs: Vec<KeyValue>) -> Bytes {
        let request = ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                attributes: attrs,
                dropped_attributes_count: 0,
                entity_refs: vec![],
            },
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(1u64)
                        .severity_number(SeverityNumber::Info)
                        .finish(),
                ],
            )],
        )]);
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        Bytes::from(buf)
    }

    #[test]
    fn test_validate_logs_with_valid_attribute() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/123/resourceGroups/test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logs_missing_attribute() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "other.attribute",
            AnyValue::new_string("value"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::MissingAttribute, _))
        ));
    }

    #[test]
    fn test_validate_logs_value_not_in_allowed_list() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/456/resourceGroups/other"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::NotInAllowedList, _))
        ));
    }

    #[test]
    fn test_validate_logs_case_insensitive() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/Subscriptions/123/ResourceGroups/Test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourcegroups/test".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: true,
        };

        let result = validator.validate_logs(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logs_empty_allowed_list_presence_only() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("any-value"),
        )]);

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: HashSet::new(),
            case_insensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logs_invalid_attribute_type() {
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_int(12345),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("12345".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::InvalidAttributeType, _))
        ));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "required_attribute": "my.custom.attribute",
            "allowed_values": ["value1", "Value2"],
            "case_insensitive": true
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.required_attribute, "my.custom.attribute");
        assert_eq!(config.allowed_values, vec!["value1", "Value2"]);
        assert!(config.case_insensitive);

        // allowed_values_set should lowercase values when case_insensitive
        let set = config.allowed_values_set();
        assert!(set.contains("value1"));
        assert!(set.contains("value2"));
        assert!(!set.contains("Value2"));
    }

    // Helper to create Arrow records from OTLP logs
    fn create_arrow_logs_with_resource(attrs: Vec<KeyValue>) -> OtapArrowRecords {
        let logs_bytes = create_logs_request_with_resource(attrs);
        let otlp_bytes = OtlpProtoBytes::ExportLogsRequest(logs_bytes);
        otlp_bytes.try_into().expect("Failed to convert to Arrow")
    }

    #[test]
    fn test_validate_arrow_logs_with_valid_attribute() {
        let arrow_records = create_arrow_logs_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/123/resourceGroups/test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_arrow_logs(&arrow_records);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_arrow_logs_missing_attribute() {
        let arrow_records = create_arrow_logs_with_resource(vec![KeyValue::new(
            "other.attribute",
            AnyValue::new_string("value"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_arrow_logs(&arrow_records);
        assert!(matches!(
            result,
            Err((ValidationFailure::MissingAttribute, _))
        ));
    }

    #[test]
    fn test_validate_arrow_logs_not_in_allowed_list() {
        let arrow_records = create_arrow_logs_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/456/resourceGroups/other"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: false,
        };

        let result = validator.validate_arrow_logs(&arrow_records);
        assert!(matches!(
            result,
            Err((ValidationFailure::NotInAllowedList, _))
        ));
    }

    #[test]
    fn test_validate_arrow_logs_case_insensitive() {
        let arrow_records = create_arrow_logs_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/Subscriptions/123/ResourceGroups/Test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourcegroups/test".to_string());

        let validator = TestValidator {
            required_attribute: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_insensitive: true,
        };

        let result = validator.validate_arrow_logs(&arrow_records);
        assert!(result.is_ok());
    }
}
