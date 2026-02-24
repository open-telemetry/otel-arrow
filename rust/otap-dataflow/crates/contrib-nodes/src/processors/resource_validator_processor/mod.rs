// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resource Validator Processor
//!
//! This processor validates that a required resource attribute exists and its value
//! is in an allowed list. Requests that fail validation are permanently NACKed,
//! enabling clients to detect misconfiguration immediately rather than having data
//! silently dropped.
//!
//! # Example Use Case
//!
//! In multi-tenant cloud environments, telemetry includes a resource attribute
//! (e.g., `cloud.resource_id`) containing an identifier for the resource.
//! This processor validates:
//! 1. The attribute exists on the Resource
//! 2. The value is in the allowed list
//! 3. Rejects with permanent NACK on failure
//!
//! # Configuration
//!
//! ```yaml
//! processors:
//!   resource_validator:
//!     required_attribute_key: "cloud.resource_id"  # required, no default
//!     allowed_values:
//!       - "/subscriptions/xxx/resourceGroups/yyy/..."
//!     case_sensitive: false  # optional, defaults to true
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
use otap_df_telemetry::otel_warn;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;

/// URN identifier for the Resource Validator processor
pub const RESOURCE_VALIDATOR_PROCESSOR_URN: &str = "urn:otel:resource_validator:processor";

/// Source of allowed values for validation.
///
/// This enum enables extensibility for future dynamic auth context support.
/// Currently only `Static` is used, but `Dynamic` provides the extension
/// point for SAT auth integration.
#[derive(Debug, Clone)]
pub enum AllowedValuesSource {
    /// Use only the static config values.
    Static,

    /// Check auth context first, fall back to config values (future).
    /// When SAT auth extension is ready, this variant will be used to
    /// indicate that allowed values should be read from the request context.
    #[allow(dead_code)]
    Dynamic,
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
    /// Internal error during format conversion (Arrow to OTLP)
    ConversionError,
}

impl std::fmt::Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationFailure::MissingAttribute => write!(f, "missing"),
            ValidationFailure::InvalidAttributeType => write!(f, "invalid_type"),
            ValidationFailure::NotInAllowedList => write!(f, "not_allowed"),
            ValidationFailure::ConversionError => write!(f, "conversion_error(internal)"),
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
/// - `source_mode`: Determines where allowed values come from
/// - `get_allowed_values()`: Extension point for per-request allowed values
pub struct ResourceValidatorProcessor {
    /// The attribute key to validate
    required_attribute_key: String,
    /// Pre-normalized allowed values (used as-is for Static, as fallback for Dynamic)
    allowed_values: HashSet<String>,
    /// Where to get allowed values from
    #[allow(dead_code)]
    source_mode: AllowedValuesSource,
    /// Whether to perform case-sensitive comparison
    case_sensitive: bool,
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
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
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
            required_attribute_key: config.required_attribute_key.clone(),
            allowed_values: config.allowed_values_set(),
            source_mode: AllowedValuesSource::Static,
            case_sensitive: config.case_sensitive,
            metrics,
        })
    }

    /// Creates a new ResourceValidatorProcessor with explicit configuration
    #[must_use]
    #[cfg(test)]
    pub fn new(
        required_attribute_key: String,
        allowed_values: HashSet<String>,
        case_sensitive: bool,
        pipeline_ctx: PipelineContext,
    ) -> Self {
        let metrics = pipeline_ctx.register_metrics::<ResourceValidatorMetrics>();
        Self {
            required_attribute_key,
            allowed_values,
            source_mode: AllowedValuesSource::Static,
            case_sensitive,
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
    ///     match &self.source_mode {
    ///         AllowedValuesSource::Dynamic => {
    ///             // Try to get from auth context
    ///             if let Some(auth) = pdata.context().auth() {
    ///                 if let Some(resource_ids) = auth.get_resource_ids() {
    ///                     return Cow::Owned(self.normalize_values(resource_ids));
    ///                 }
    ///             }
    ///             // Fall back to config
    ///             Cow::Borrowed(&self.allowed_values)
    ///         }
    ///         AllowedValuesSource::Static => Cow::Borrowed(&self.allowed_values),
    ///     }
    /// }
    /// ```
    const fn get_allowed_values(&self, _pdata: &OtapPdata) -> Cow<'_, HashSet<String>> {
        // Currently just returns the static/fallback values.
        // When auth context is available, this will check pdata.context().auth() first.
        Cow::Borrowed(&self.allowed_values)
    }

    /// Validates a single resource's attributes against the provided allowed values.
    fn validate_resource_with_allowed<R: ResourceView>(
        &self,
        resource: &R,
        allowed_values: &HashSet<String>,
    ) -> Result<(), ValidationFailure> {
        let required_key = self.required_attribute_key.as_bytes();

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

                // Check if value is in allowed list.
                // Empty allowed_values rejects all values.
                // Case-sensitive: zero allocation with Cow::Borrowed.
                // Case-insensitive: allocates via to_lowercase() for O(1) HashSet lookup.
                let lookup_value: Cow<'_, str> = if self.case_sensitive {
                    Cow::Borrowed(str_value)
                } else {
                    Cow::Owned(str_value.to_lowercase())
                };

                if allowed_values.contains(lookup_value.as_ref()) {
                    return Ok(());
                } else {
                    return Err(ValidationFailure::NotInAllowedList);
                }
            }
        }

        // Attribute not found
        Err(ValidationFailure::MissingAttribute)
    }

    // Note: validate_logs, validate_metrics, validate_traces, and validate_arrow_logs
    // share identical loop bodies. A generic helper is impractical because the view traits
    // (ResourceLogsView, ResourceMetricsView, ResourceSpansView) use GATs with different
    // associated types and have no common super-trait for `.resource()`.

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
            let failure = ValidationFailure::ConversionError;
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
                    self.required_attribute_key
                )
            }
            ValidationFailure::InvalidAttributeType => {
                format!(
                    "resource attribute '{}' must be a string",
                    self.required_attribute_key
                )
            }
            ValidationFailure::NotInAllowedList => {
                format!(
                    "resource attribute '{}' value is not in the allowed list",
                    self.required_attribute_key
                )
            }
            ValidationFailure::ConversionError => {
                "internal error: failed to convert telemetry format for validation".to_string()
            }
        }
    }

    /// Updates metrics and logs warnings based on validation result
    fn update_metrics(&mut self, result: &Result<(), (ValidationFailure, String)>, num_items: u64) {
        match result {
            Ok(()) => {
                self.metrics.batches_accepted.add(1);
                self.metrics.items_accepted.add(num_items);
            }
            Err((failure, msg)) => {
                otel_warn!(
                    "resource_validator_processor.validation.fail",
                    failure_reason = %failure,
                    message = msg.as_str()
                );
                self.metrics.items_rejected.add(num_items);
                match failure {
                    ValidationFailure::MissingAttribute => {
                        self.metrics.batches_rejected_missing.add(1);
                    }
                    ValidationFailure::InvalidAttributeType => {
                        self.metrics.batches_rejected_invalid_type.add(1);
                    }
                    ValidationFailure::ConversionError => {
                        self.metrics.batches_rejected_conversion_error.add(1);
                    }
                    ValidationFailure::NotInAllowedList => {
                        self.metrics.batches_rejected_not_allowed.add(1);
                    }
                }
            }
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
                        // TODO: Implement OtapMetricsView/OtapTracesView to avoid clone + conversion
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
                                    let failure = ValidationFailure::ConversionError;
                                    Err((failure, self.format_error_message(failure)))
                                }
                            }
                        }
                    },
                };

                // Update metrics
                let num_items = pdata.num_items() as u64;
                self.update_metrics(&validation_result, num_items);

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

    // TODO: Refactor tests to use the actual `ResourceValidatorProcessor` instead of `TestValidator`.
    // Currently `TestValidator` reimplements validation logic, which means tests don't verify the
    // production code path. This requires test infrastructure for `PipelineContext` (metrics registration).

    /// Test helper struct for validation testing without metrics
    struct TestValidator {
        required_attribute_key: String,
        allowed_values: HashSet<String>,
        case_sensitive: bool,
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
            let required_key = self.required_attribute_key.as_bytes();

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

                    // Empty allowed_values rejects all values
                    let lookup_value: Cow<'_, str> = if self.case_sensitive {
                        Cow::Borrowed(str_value)
                    } else {
                        Cow::Owned(str_value.to_lowercase())
                    };

                    if self.allowed_values.contains(lookup_value.as_ref()) {
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
                let failure = ValidationFailure::ConversionError;
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

        fn validate_metrics(
            &self,
            data: &RawMetricsData<'_>,
        ) -> Result<(), (ValidationFailure, String)> {
            for resource_metrics in data.resources() {
                if let Some(resource) = resource_metrics.resource() {
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

        fn validate_traces(
            &self,
            data: &RawTraceData<'_>,
        ) -> Result<(), (ValidationFailure, String)> {
            for resource_spans in data.resources() {
                if let Some(resource) = resource_spans.resource() {
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
                        self.required_attribute_key
                    )
                }
                ValidationFailure::InvalidAttributeType => {
                    format!(
                        "resource attribute '{}' must be a string",
                        self.required_attribute_key
                    )
                }
                ValidationFailure::NotInAllowedList => {
                    format!(
                        "resource attribute '{}' value is not in the allowed list",
                        self.required_attribute_key
                    )
                }
                ValidationFailure::ConversionError => {
                    "internal error: failed to convert telemetry format for validation".to_string()
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: false,
        };

        let result = validator.validate_logs(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_logs_empty_allowed_list_rejects_all() {
        // Empty allowed_values rejects all values
        let logs_bytes = create_logs_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("any-value"),
        )]);

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: HashSet::new(),
            case_sensitive: true,
        };

        let result = validator.validate_logs(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::NotInAllowedList, _))
        ));
    }

    #[test]
    fn test_validate_logs_empty_resource_attributes() {
        // Resource with no attributes at all
        let logs_bytes = create_logs_request_with_resource(vec![]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawLogsData::new(&logs_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
        };

        let result = validator.validate_logs(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::MissingAttribute, _))
        ));
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            "required_attribute_key": "my.custom.attribute",
            "allowed_values": ["value1", "Value2"],
            "case_sensitive": false
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.required_attribute_key, "my.custom.attribute");
        assert_eq!(config.allowed_values, vec!["value1", "Value2"]);
        assert!(!config.case_sensitive);

        // allowed_values_set should lowercase values when case_sensitive is false
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
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
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: false,
        };

        let result = validator.validate_arrow_logs(&arrow_records);
        assert!(result.is_ok());
    }

    // ==================== Metrics/Traces OTLP Validation Tests ====================

    fn create_metrics_request_with_resource(attrs: Vec<KeyValue>) -> Bytes {
        use otap_df_pdata::proto::opentelemetry::{
            collector::metrics::v1::ExportMetricsServiceRequest,
            metrics::v1::{Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics},
        };

        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: attrs,
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope::default()),
                    metrics: vec![Metric {
                        name: "test_metric".to_string(),
                        data: Some(otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                value: Some(otap_df_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value::AsInt(42)),
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        Bytes::from(buf)
    }

    fn create_traces_request_with_resource(attrs: Vec<KeyValue>) -> Bytes {
        use otap_df_pdata::proto::opentelemetry::{
            collector::trace::v1::ExportTraceServiceRequest,
            trace::v1::{ResourceSpans, ScopeSpans, Span},
        };

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: attrs,
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope::default()),
                    spans: vec![Span {
                        name: "test_span".to_string(),
                        trace_id: vec![1u8; 16],
                        span_id: vec![2u8; 8],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        Bytes::from(buf)
    }

    #[test]
    fn test_validate_metrics_with_valid_attribute() {
        let metrics_bytes = create_metrics_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/123/resourceGroups/test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawMetricsData::new(&metrics_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
        };

        let result = validator.validate_metrics(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_metrics_missing_attribute() {
        let metrics_bytes = create_metrics_request_with_resource(vec![KeyValue::new(
            "other.attribute",
            AnyValue::new_string("value"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawMetricsData::new(&metrics_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
        };

        let result = validator.validate_metrics(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::MissingAttribute, _))
        ));
    }

    #[test]
    fn test_validate_traces_with_valid_attribute() {
        let traces_bytes = create_traces_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/123/resourceGroups/test"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawTraceData::new(&traces_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
        };

        let result = validator.validate_traces(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_traces_not_in_allowed_list() {
        let traces_bytes = create_traces_request_with_resource(vec![KeyValue::new(
            "microsoft.resourceId",
            AnyValue::new_string("/subscriptions/456/other"),
        )]);

        let mut allowed = HashSet::new();
        let _ = allowed.insert("/subscriptions/123/resourceGroups/test".to_string());

        let data = RawTraceData::new(&traces_bytes);
        let validator = TestValidator {
            required_attribute_key: "microsoft.resourceId".to_string(),
            allowed_values: allowed,
            case_sensitive: true,
        };

        let result = validator.validate_traces(&data);
        assert!(matches!(
            result,
            Err((ValidationFailure::NotInAllowedList, _))
        ));
    }
}
