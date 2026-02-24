// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Content-based routing processor for OTAP pipelines.
//!
//! Routes telemetry signals to different output ports based on a resource
//! attribute value. Uses zero-copy protobuf views to extract the routing key
//! without full deserialization, making it efficient for OTLP-in/OTLP-out
//! pipelines.
//!
//! # Example Use Case
//!
//! Multi-tenant routing where each tenant's data goes to a dedicated exporter:
//!
//! ```yaml
//! processors:
//!   content_router:
//!     routing_key:
//!       resource_attribute: "service.namespace"
//!     case_sensitive: false
//!     routes:
//!       "frontend": "frontend_pipeline"
//!       "backend": "backend_pipeline"
//!     default_output: "fallback"
//! ```
//!
//! Each route value corresponds to a named output port that must be wired
//! in the pipeline configuration.
//!
//! # Unmatched and Mixed-Batch Behaviour
//!
//! - **Unmatched:** When no route matches (missing key or no matching value),
//!   the message is sent to `default_output` if configured, otherwise it is
//!   permanently NACKed.
//! - **Mixed batch:** If a single batch contains resources that would route to
//!   different destinations, the entire batch is permanently NACKed. Batches
//!   where all resources are consistently unmatched (missing key / no match)
//!   are **not** considered mixed and are routed to the default output.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::PortName;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, MessageSourceLocalEffectHandlerExtension, ProcessorFactory,
};
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
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;

use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// URN for the ContentRouter processor
pub const CONTENT_ROUTER_URN: &str = "urn:otel:content_router:processor";

/// Specifies where and how the routing key value is extracted from a telemetry message.
///
/// Using an explicit source type makes the configuration unambiguous and allows
/// future variants (e.g. scope attributes, metric names) to be added without
/// breaking existing configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingKeyExpr {
    /// Extract the routing value from a resource attribute with the given key.
    ResourceAttribute(String),
}

impl std::fmt::Display for RoutingKeyExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResourceAttribute(key) => write!(f, "resource_attribute({})", key),
        }
    }
}

/// Metrics for the ContentRouter processor.
#[metric_set(name = "content_router.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct ContentRouterMetrics {
    /// Number of messages received by the router.
    #[metric(unit = "{msg}")]
    pub signals_received: Counter<u64>,
    /// Number of messages routed to a named port.
    #[metric(unit = "{msg}")]
    pub signals_routed: Counter<u64>,
    /// Number of messages routed to the default output.
    #[metric(unit = "{msg}")]
    pub signals_routed_default: Counter<u64>,
    /// Number of messages NACKed (no route match, missing key, mixed batch,
    /// conversion error, or send failure).
    #[metric(unit = "{msg}")]
    pub signals_nacked: Counter<u64>,
    /// Number of messages where the routing key was missing.
    #[metric(unit = "{msg}")]
    pub signals_no_routing_key: Counter<u64>,
    /// Number of messages that failed due to internal conversion errors.
    #[metric(unit = "{msg}")]
    pub signals_conversion_error: Counter<u64>,
}

/// Configuration for the ContentRouter processor.
///
/// ```yaml
/// processors:
///   content_router:
///     routing_key:
///       resource_attribute: "service.namespace"
///     case_sensitive: false
///     routes:
///       "frontend": "frontend_pipeline"
///       "backend": "backend_pipeline"
///     default_output: "fallback"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRouterConfig {
    /// The source and key used to extract the routing value from a telemetry message.
    pub routing_key: RoutingKeyExpr,
    /// Map of attribute values to output port names.
    pub routes: HashMap<String, String>,
    /// Output port for messages that don't match any route.
    /// If not set, unmatched messages are permanently NACKed.
    #[serde(default)]
    pub default_output: Option<String>,
    /// Whether to perform case-sensitive matching on attribute values.
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
}

const fn default_case_sensitive() -> bool {
    true
}

impl ContentRouterConfig {
    /// Validates the configuration.
    ///
    /// If `declared_outputs` is non-empty, also checks that every route
    /// destination and `default_output` refers to a declared node output port.
    fn validate(&self, declared_outputs: &[PortName]) -> Result<(), ConfigError> {
        match &self.routing_key {
            RoutingKeyExpr::ResourceAttribute(key) => {
                if key.trim().is_empty() {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "routing_key.resource_attribute must not be empty".to_string(),
                    });
                }
            }
        }
        if self.routes.is_empty() {
            return Err(ConfigError::InvalidUserConfig {
                error: "routes must not be empty".to_string(),
            });
        }
        for (value, port) in &self.routes {
            if value.trim().is_empty() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "route key (attribute value) must not be empty".to_string(),
                });
            }
            if port.trim().is_empty() {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("route for value '{}' has an empty port name", value),
                });
            }
        }
        if let Some(ref default) = self.default_output {
            if default.trim().is_empty() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "default_output must not be empty when specified".to_string(),
                });
            }
        }
        // Detect case-insensitive key collisions
        if !self.case_sensitive {
            let normalized = self.normalized_routes();
            if normalized.len() < self.routes.len() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "routes contain duplicate keys after case-insensitive normalization"
                        .to_string(),
                });
            }
        }
        // Validate that every route destination and default_output refer to a
        // declared node output port. Skip this check when no outputs are declared
        // (e.g. pipeline-level wiring without explicit port declarations).
        if !declared_outputs.is_empty() {
            for (value, port) in &self.routes {
                if !declared_outputs.iter().any(|o| o.as_ref() == port.as_str()) {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "route for value '{}' references undeclared output port '{}'",
                            value, port
                        ),
                    });
                }
            }
            if let Some(ref default) = self.default_output {
                if !declared_outputs
                    .iter()
                    .any(|o| o.as_ref() == default.as_str())
                {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "default_output '{}' references undeclared output port",
                            default
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    /// Returns a normalized routes map (lowercased keys if case-insensitive).
    fn normalized_routes(&self) -> HashMap<String, PortName> {
        if self.case_sensitive {
            self.routes
                .iter()
                .map(|(k, v)| (k.clone(), PortName::from(v.clone())))
                .collect()
        } else {
            self.routes
                .iter()
                .map(|(k, v)| (k.to_lowercase(), PortName::from(v.clone())))
                .collect()
        }
    }
}

/// The result of attempting to resolve a routing destination from a message.
enum RouteResolution {
    /// Route to this named output port.
    Matched(String),
    /// No matching route found; use default if available.
    NoMatch,
    /// The routing key attribute was not found on the resource.
    MissingKey,
    /// Multiple resources in the batch have different routing keys.
    MixedBatch,
    /// Internal error during format conversion (e.g., Arrow to OTLP).
    ConversionError,
}

/// The ContentRouter processor routes messages to output ports based on
/// a resource attribute value.
pub struct ContentRouter {
    /// The source and key used to extract the routing value.
    routing_key: RoutingKeyExpr,
    /// Normalized routes: attribute value -> output port name.
    routes: HashMap<String, PortName>,
    /// Default output port for unmatched messages.
    default_output: Option<String>,
    /// Whether matching is case-sensitive.
    case_sensitive: bool,
    /// Telemetry metrics.
    metrics: Option<MetricSet<ContentRouterMetrics>>,
}

impl ContentRouter {
    /// Creates a new ContentRouter from config.
    #[must_use]
    pub fn new(config: ContentRouterConfig) -> Self {
        let routes = config.normalized_routes();
        Self {
            routing_key: config.routing_key,
            routes,
            default_output: config.default_output,
            case_sensitive: config.case_sensitive,
            metrics: None,
        }
    }

    /// Creates a new ContentRouter with metrics registered via PipelineContext.
    #[must_use]
    pub fn with_pipeline_ctx(pipeline_ctx: PipelineContext, config: ContentRouterConfig) -> Self {
        let metrics = pipeline_ctx.register_metrics::<ContentRouterMetrics>();
        let mut router = Self::new(config);
        router.metrics = Some(metrics);
        router
    }

    /// Extracts the routing key value from a resource's attributes using zero-copy views.
    /// Returns the resolved port name or None if the key is missing/not a route match.
    fn extract_route_from_resource<R: ResourceView>(&self, resource: &R) -> RouteResolution {
        let key_bytes = match &self.routing_key {
            RoutingKeyExpr::ResourceAttribute(key) => key.as_bytes(),
        };

        for attr in resource.attributes() {
            if attr.key() == key_bytes {
                let Some(value) = attr.value() else {
                    return RouteResolution::MissingKey;
                };

                // Key exists but value is not a usable string type — treat as NoMatch,
                // not MissingKey, since the attribute is present.
                if value.value_type() != ValueType::String {
                    return RouteResolution::NoMatch;
                };

                let Some(str_bytes) = value.as_string() else {
                    return RouteResolution::NoMatch;
                };

                let Ok(str_value) = std::str::from_utf8(str_bytes) else {
                    return RouteResolution::NoMatch;
                };

                // Note: to_lowercase() allocates per-resource when case-insensitive.
                // Do NOT replace with eq_ignore_ascii_case — that would break Unicode
                // case folding (e.g., Turkish İ). Accept the allocation cost here;
                // profile before optimizing.
                let lookup: Cow<'_, str> = if self.case_sensitive {
                    Cow::Borrowed(str_value)
                } else {
                    Cow::Owned(str_value.to_lowercase())
                };

                if let Some(port) = self.routes.get(lookup.as_ref()) {
                    return RouteResolution::Matched(port.to_string());
                }
                return RouteResolution::NoMatch;
            }
        }
        RouteResolution::MissingKey
    }

    /// Folds a new resource resolution into the running accumulator.
    /// Returns MixedBatch only when resources would route to different destinations.
    /// NoMatch and MissingKey are treated as equivalent (both go to default_output or NACK),
    /// so a batch mixing them is NOT considered mixed.
    fn fold_resolution(acc: Option<RouteResolution>, next: RouteResolution) -> RouteResolution {
        match acc {
            None => next,
            Some(RouteResolution::MixedBatch) => RouteResolution::MixedBatch,
            Some(ref prev) => {
                let consistent = match (prev, &next) {
                    (RouteResolution::Matched(a), RouteResolution::Matched(b)) => a == b,
                    // NoMatch and MissingKey both route to default_output (or NACK),
                    // so any combination of them is destination-consistent.
                    (
                        RouteResolution::NoMatch | RouteResolution::MissingKey,
                        RouteResolution::NoMatch | RouteResolution::MissingKey,
                    ) => true,
                    _ => false,
                };
                if consistent {
                    // Prefer MissingKey over NoMatch to preserve metric accuracy
                    match (&next, prev) {
                        (RouteResolution::NoMatch, RouteResolution::MissingKey)
                        | (RouteResolution::MissingKey, _) => RouteResolution::MissingKey,
                        _ => next,
                    }
                } else {
                    RouteResolution::MixedBatch
                }
            }
        }
    }

    /// Resolves the route for logs data (OTLP bytes). Validates all resources agree.
    fn resolve_logs_route(&self, data: &RawLogsData<'_>) -> RouteResolution {
        let mut acc: Option<RouteResolution> = None;
        for resource_logs in data.resources() {
            let res = match resource_logs.resource() {
                Some(resource) => self.extract_route_from_resource(&resource),
                None => RouteResolution::MissingKey,
            };
            acc = Some(Self::fold_resolution(acc, res));
            if matches!(acc, Some(RouteResolution::MixedBatch)) {
                return RouteResolution::MixedBatch;
            }
        }
        acc.unwrap_or(RouteResolution::MissingKey)
    }

    /// Resolves the route for Arrow logs data using native OTAP view (no OTLP conversion).
    fn resolve_arrow_logs_route(
        &self,
        arrow_records: &otap_df_pdata::OtapArrowRecords,
    ) -> RouteResolution {
        let logs_view = match OtapLogsView::try_from(arrow_records) {
            Ok(view) => view,
            Err(_) => return RouteResolution::ConversionError,
        };
        let mut acc: Option<RouteResolution> = None;
        for resource_logs in logs_view.resources() {
            let res = match resource_logs.resource() {
                Some(resource) => self.extract_route_from_resource(&resource),
                None => RouteResolution::MissingKey,
            };
            acc = Some(Self::fold_resolution(acc, res));
            if matches!(acc, Some(RouteResolution::MixedBatch)) {
                return RouteResolution::MixedBatch;
            }
        }
        acc.unwrap_or(RouteResolution::MissingKey)
    }

    /// Resolves the route for metrics data. Validates all resources agree.
    fn resolve_metrics_route(&self, data: &RawMetricsData<'_>) -> RouteResolution {
        let mut acc: Option<RouteResolution> = None;
        for resource_metrics in data.resources() {
            let res = match resource_metrics.resource() {
                Some(resource) => self.extract_route_from_resource(&resource),
                None => RouteResolution::MissingKey,
            };
            acc = Some(Self::fold_resolution(acc, res));
            if matches!(acc, Some(RouteResolution::MixedBatch)) {
                return RouteResolution::MixedBatch;
            }
        }
        acc.unwrap_or(RouteResolution::MissingKey)
    }

    /// Resolves the route for traces data. Validates all resources agree.
    fn resolve_traces_route(&self, data: &RawTraceData<'_>) -> RouteResolution {
        let mut acc: Option<RouteResolution> = None;
        for resource_spans in data.resources() {
            let res = match resource_spans.resource() {
                Some(resource) => self.extract_route_from_resource(&resource),
                None => RouteResolution::MissingKey,
            };
            acc = Some(Self::fold_resolution(acc, res));
            if matches!(acc, Some(RouteResolution::MixedBatch)) {
                return RouteResolution::MixedBatch;
            }
        }
        acc.unwrap_or(RouteResolution::MissingKey)
    }

    /// Resolves the output port for a given message payload.
    fn resolve_route(&self, pdata: &OtapPdata) -> RouteResolution {
        let signal_type = pdata.signal_type();

        match pdata.payload_ref() {
            OtapPayload::OtlpBytes(otlp_bytes) => match (signal_type, otlp_bytes) {
                (SignalType::Logs, OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                    let data = RawLogsData::new(bytes.as_ref());
                    self.resolve_logs_route(&data)
                }
                (SignalType::Metrics, OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
                    let data = RawMetricsData::new(bytes.as_ref());
                    self.resolve_metrics_route(&data)
                }
                (SignalType::Traces, OtlpProtoBytes::ExportTracesRequest(bytes)) => {
                    let data = RawTraceData::new(bytes.as_ref());
                    self.resolve_traces_route(&data)
                }
                // Defensive: signal_type/payload mismatch cannot occur for OtlpBytes
                // since signal_type() is derived from the OtlpProtoBytes variant itself.
                _ => RouteResolution::ConversionError,
            },
            OtapPayload::OtapArrowRecords(arrow_records) => {
                match signal_type {
                    // Use native OTAP Arrow view for logs (avoids clone + OTLP round-trip)
                    SignalType::Logs => self.resolve_arrow_logs_route(arrow_records),
                    // Metrics/Traces Arrow views not yet available — convert to OTLP.
                    // TODO: Use OtapMetricsView/OtapTracesView when available.
                    _ => match OtlpProtoBytes::try_from(arrow_records.clone()) {
                        Ok(OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
                            let data = RawMetricsData::new(bytes.as_ref());
                            self.resolve_metrics_route(&data)
                        }
                        Ok(OtlpProtoBytes::ExportTracesRequest(bytes)) => {
                            let data = RawTraceData::new(bytes.as_ref());
                            self.resolve_traces_route(&data)
                        }
                        _ => RouteResolution::ConversionError,
                    },
                }
            }
        }
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for ContentRouter {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(ctrl) => {
                if let NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } = ctrl
                {
                    if let Some(m) = self.metrics.as_mut() {
                        let _ = metrics_reporter.report(m);
                    }
                }
                Ok(())
            }
            Message::PData(data) => {
                if let Some(m) = self.metrics.as_mut() {
                    m.signals_received.inc();
                }

                let resolution = self.resolve_route(&data);

                match resolution {
                    RouteResolution::Matched(port) => {
                        match effect_handler
                            .send_message_with_source_node_to(port, data)
                            .await
                        {
                            Ok(()) => {
                                if let Some(m) = self.metrics.as_mut() {
                                    m.signals_routed.inc();
                                }
                                Ok(())
                            }
                            Err(e) => {
                                if let Some(m) = self.metrics.as_mut() {
                                    m.signals_nacked.inc();
                                }
                                Err(e.into())
                            }
                        }
                    }
                    RouteResolution::NoMatch | RouteResolution::MissingKey => {
                        if matches!(resolution, RouteResolution::MissingKey) {
                            if let Some(m) = self.metrics.as_mut() {
                                m.signals_no_routing_key.inc();
                            }
                        }
                        // Try default output if configured
                        if let Some(ref default_port) = self.default_output {
                            match effect_handler
                                .send_message_with_source_node_to(default_port.clone(), data)
                                .await
                            {
                                Ok(()) => {
                                    if let Some(m) = self.metrics.as_mut() {
                                        m.signals_routed_default.inc();
                                    }
                                    Ok(())
                                }
                                Err(e) => {
                                    if let Some(m) = self.metrics.as_mut() {
                                        m.signals_nacked.inc();
                                    }
                                    Err(e.into())
                                }
                            }
                        } else {
                            // No default output - NACK to inform upstream
                            if let Some(m) = self.metrics.as_mut() {
                                m.signals_nacked.inc();
                            }
                            let reason = if matches!(resolution, RouteResolution::MissingKey) {
                                format!(
                                    "routing key '{}' not found on resource and no default output configured",
                                    self.routing_key // Display: e.g. resource_attribute(service.namespace)
                                )
                            } else {
                                format!(
                                    "no matching route for routing key '{}' and no default output configured",
                                    self.routing_key
                                )
                            };
                            effect_handler
                                .notify_nack(NackMsg::new_permanent(reason, data))
                                .await?;
                            Ok(())
                        }
                    }
                    RouteResolution::MixedBatch => {
                        if let Some(m) = self.metrics.as_mut() {
                            m.signals_nacked.inc();
                        }
                        let reason = format!(
                            "batch contains resources with inconsistent routing for key '{}'; \
                             all resources must resolve to the same destination",
                            self.routing_key // Display: e.g. resource_attribute(service.namespace)
                        );
                        effect_handler
                            .notify_nack(NackMsg::new_permanent(reason, data))
                            .await?;
                        Ok(())
                    }
                    RouteResolution::ConversionError => {
                        if let Some(m) = self.metrics.as_mut() {
                            m.signals_conversion_error.inc();
                            m.signals_nacked.inc();
                        }
                        let reason =
                            "internal error: failed to convert telemetry format for routing"
                                .to_string();
                        effect_handler
                            .notify_nack(NackMsg::new_permanent(reason, data))
                            .await?;
                        Ok(())
                    }
                }
            }
        }
    }
}

/// Factory function to create a ContentRouter processor
pub fn create_content_router(
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let router_config: ContentRouterConfig = serde_json::from_value(node_config.config.clone())
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse ContentRouter configuration: {e}"),
        })?;
    router_config.validate(&node_config.outputs)?;

    let router = ContentRouter::new(router_config);

    Ok(ProcessorWrapper::local(
        router,
        node,
        node_config,
        processor_config,
    ))
}

/// Register ContentRouter as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static CONTENT_ROUTER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: CONTENT_ROUTER_URN,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<ContentRouterConfig>,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig| {
        let router_config: ContentRouterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse ContentRouter configuration: {e}"),
            })?;
        router_config.validate(&node_config.outputs)?;

        let router = ContentRouter::with_pipeline_ctx(pipeline, router_config);

        Ok(ProcessorWrapper::local(router, node, node_config, proc_cfg))
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use otap_df_engine::testing::{processor::TestRuntime, test_node};
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use prost::Message as ProstMessage;
    use serde_json::json;

    fn create_logs_with_resource_attr(key: &str, value: &str) -> Bytes {
        let request = ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                attributes: vec![KeyValue::new(key, AnyValue::new_string(value))],
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

    fn create_logs_without_resource_attr() -> Bytes {
        let request = ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                attributes: vec![],
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

    fn create_multi_resource_logs(resources: Vec<Vec<KeyValue>>) -> Bytes {
        let resource_logs: Vec<ResourceLogs> = resources
            .into_iter()
            .map(|attrs| {
                ResourceLogs::new(
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
                )
            })
            .collect();
        let request = ExportLogsServiceRequest::new(resource_logs);
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        Bytes::from(buf)
    }

    fn create_metrics_with_resource_attr(key: &str, value: &str) -> Bytes {
        use otap_df_pdata::proto::opentelemetry::{
            collector::metrics::v1::ExportMetricsServiceRequest,
            metrics::v1::{Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics},
        };

        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new(key, AnyValue::new_string(value))],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope::default()),
                    metrics: vec![Metric {
                        name: "test_metric".to_string(),
                        data: Some(
                            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Gauge(
                                Gauge {
                                    data_points: vec![NumberDataPoint {
                                        value: Some(otap_df_pdata::proto::opentelemetry::metrics::v1::number_data_point::Value::AsInt(42)),
                                        ..Default::default()
                                    }],
                                },
                            ),
                        ),
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

    fn create_traces_with_resource_attr(key: &str, value: &str) -> Bytes {
        use otap_df_pdata::proto::opentelemetry::{
            collector::trace::v1::ExportTraceServiceRequest,
            trace::v1::{ResourceSpans, ScopeSpans, Span},
        };

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new(key, AnyValue::new_string(value))],
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

    fn make_config(
        routes: HashMap<String, String>,
        default: Option<String>,
    ) -> ContentRouterConfig {
        ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
            routes,
            default_output: default,
            case_sensitive: true,
        }
    }

    // -------------------------------------------------------
    // Config deserialization tests
    // -------------------------------------------------------

    #[test]
    fn test_config_deserialization() {
        let config_json = json!({
            "routing_key": { "resource_attribute": "service.namespace" },
            "routes": {
                "/subscriptions/aaa": "tenant_a",
                "/subscriptions/bbb": "tenant_b"
            },
            "default_output": "fallback",
            "case_sensitive": false
        });
        let cfg: ContentRouterConfig = serde_json::from_value(config_json).unwrap();
        assert!(matches!(
            cfg.routing_key,
            RoutingKeyExpr::ResourceAttribute(ref k) if k == "service.namespace"
        ));
        assert_eq!(cfg.routes.len(), 2);
        assert_eq!(cfg.default_output, Some("fallback".to_string()));
        assert!(!cfg.case_sensitive);
    }

    #[test]
    fn test_config_deserialization_defaults() {
        let config_json = json!({
            "routing_key": { "resource_attribute": "tenant.id" },
            "routes": { "a": "port_a" }
        });
        let cfg: ContentRouterConfig = serde_json::from_value(config_json).unwrap();
        assert!(cfg.case_sensitive);
        assert!(cfg.default_output.is_none());
    }

    #[test]
    fn test_config_validation_empty_key() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("".to_string()),
            routes: HashMap::from([("a".into(), "b".into())]),
            default_output: None,
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_whitespace_key() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("  ".to_string()),
            routes: HashMap::from([("a".into(), "b".into())]),
            default_output: None,
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_empty_routes() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::new(),
            default_output: None,
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_empty_port_name() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("value".into(), "".into())]),
            default_output: None,
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_empty_route_key() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("".into(), "port_a".into())]),
            default_output: None,
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_empty_default_output() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("a".into(), "b".into())]),
            default_output: Some("".to_string()),
            case_sensitive: true,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_config_validation_route_undeclared_output() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("a".into(), "port_a".into())]),
            default_output: None,
            case_sensitive: true,
        };
        // "port_a" is not in the declared outputs
        let declared: Vec<PortName> = vec!["other_port".into()];
        assert!(cfg.validate(&declared).is_err());
    }

    #[test]
    fn test_config_validation_default_output_undeclared() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("a".into(), "port_a".into())]),
            default_output: Some("fallback".to_string()),
            case_sensitive: true,
        };
        // "port_a" is declared but "fallback" is not
        let declared: Vec<PortName> = vec!["port_a".into()];
        assert!(cfg.validate(&declared).is_err());
    }

    #[test]
    fn test_config_validation_all_outputs_declared_ok() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([("a".into(), "port_a".into())]),
            default_output: Some("fallback".to_string()),
            case_sensitive: true,
        };
        let declared: Vec<PortName> = vec!["port_a".into(), "fallback".into()];
        assert!(cfg.validate(&declared).is_ok());
    }

    #[test]
    fn test_config_validation_case_insensitive_collision() {
        let cfg = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("key".to_string()),
            routes: HashMap::from([
                ("Tenant_A".into(), "port_a".into()),
                ("tenant_a".into(), "port_b".into()),
            ]),
            default_output: None,
            case_sensitive: false,
        };
        assert!(cfg.validate(&[]).is_err());
    }

    #[test]
    fn test_factory_creation_ok() {
        let config = json!({
            "routing_key": { "resource_attribute": "service.namespace" },
            "routes": { "/sub/a": "tenant_a" }
        });
        let processor_config = ProcessorConfig::new("test_content_router");
        let mut node_config = NodeUserConfig::new_processor_config(CONTENT_ROUTER_URN);
        node_config.config = config;
        node_config.add_output("tenant_a");
        let result = create_content_router(
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
            &processor_config,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_creation_undeclared_output() {
        let config = json!({
            "routing_key": { "resource_attribute": "service.namespace" },
            "routes": { "/sub/a": "tenant_a" }
        });
        let processor_config = ProcessorConfig::new("test_content_router");
        let mut node_config = NodeUserConfig::new_processor_config(CONTENT_ROUTER_URN);
        node_config.config = config;
        // Declare an unrelated output — "tenant_a" is not in the list.
        node_config.add_output("other_port");
        let result = create_content_router(
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
            &processor_config,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_creation_bad_config() {
        let config = json!(42);
        let processor_config = ProcessorConfig::new("test_content_router");
        let mut node_config = NodeUserConfig::new_processor_config(CONTENT_ROUTER_URN);
        node_config.config = config;
        let result = create_content_router(
            test_node(processor_config.name.clone()),
            Arc::new(node_config),
            &processor_config,
        );
        assert!(result.is_err());
    }

    // -------------------------------------------------------
    // Zero-copy route resolution tests
    // -------------------------------------------------------

    #[test]
    fn test_resolve_logs_route_matched() {
        let routes = HashMap::from([
            ("/subscriptions/aaa".to_string(), "tenant_a".to_string()),
            ("/subscriptions/bbb".to_string(), "tenant_b".to_string()),
        ]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_logs_with_resource_attr("service.namespace", "/subscriptions/aaa");
        let data = RawLogsData::new(&bytes);
        match router.resolve_logs_route(&data) {
            RouteResolution::Matched(port) => assert_eq!(port, "tenant_a"),
            _ => panic!("expected Matched"),
        }
    }

    #[test]
    fn test_resolve_logs_route_no_match() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_logs_with_resource_attr("service.namespace", "/subscriptions/unknown");
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::NoMatch
        ));
    }

    #[test]
    fn test_resolve_logs_route_missing_key() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_logs_without_resource_attr();
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MissingKey
        ));
    }

    #[test]
    fn test_resolve_case_insensitive() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let config = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
            routes,
            default_output: None,
            case_sensitive: false,
        };
        let router = ContentRouter::new(config);

        let bytes = create_logs_with_resource_attr("service.namespace", "/Subscriptions/AAA");
        let data = RawLogsData::new(&bytes);
        match router.resolve_logs_route(&data) {
            RouteResolution::Matched(port) => assert_eq!(port, "tenant_a"),
            _ => panic!("expected case-insensitive match"),
        }
    }

    // -------------------------------------------------------
    // Metrics and traces route resolution tests
    // -------------------------------------------------------

    #[test]
    fn test_resolve_metrics_route_matched() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_metrics_with_resource_attr("service.namespace", "/subscriptions/aaa");
        let data = RawMetricsData::new(&bytes);
        match router.resolve_metrics_route(&data) {
            RouteResolution::Matched(port) => assert_eq!(port, "tenant_a"),
            _ => panic!("expected Matched for metrics"),
        }
    }

    #[test]
    fn test_resolve_traces_route_matched() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_traces_with_resource_attr("service.namespace", "/subscriptions/aaa");
        let data = RawTraceData::new(&bytes);
        match router.resolve_traces_route(&data) {
            RouteResolution::Matched(port) => assert_eq!(port, "tenant_a"),
            _ => panic!("expected Matched for traces"),
        }
    }

    // -------------------------------------------------------
    // Mixed-batch detection tests
    // -------------------------------------------------------

    #[test]
    fn test_resolve_mixed_batch_detected() {
        let routes = HashMap::from([
            ("/subscriptions/aaa".to_string(), "tenant_a".to_string()),
            ("/subscriptions/bbb".to_string(), "tenant_b".to_string()),
        ]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_multi_resource_logs(vec![
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/bbb"),
            )],
        ]);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MixedBatch
        ));
    }

    #[test]
    fn test_resolve_same_tenant_multi_resource_ok() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        let bytes = create_multi_resource_logs(vec![
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
        ]);
        let data = RawLogsData::new(&bytes);
        match router.resolve_logs_route(&data) {
            RouteResolution::Matched(port) => assert_eq!(port, "tenant_a"),
            _ => panic!("expected Matched for same-tenant multi-resource"),
        }
    }

    #[test]
    fn test_resolve_matched_plus_missing_key_is_mixed() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // First resource has routing key, second doesn't
        let bytes = create_multi_resource_logs(vec![
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
            vec![], // no routing key
        ]);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MixedBatch
        ));
    }

    #[test]
    fn test_resolve_missing_key_plus_matched_is_mixed() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // Reversed order: missing key first, then matched
        let bytes = create_multi_resource_logs(vec![
            vec![], // no routing key
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
        ]);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MixedBatch
        ));
    }

    #[test]
    fn test_resolve_no_match_plus_missing_key_is_consistent() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // One resource has unrecognized value, other has no key at all.
        // Both are "unroutable" (go to default_output or NACK), so NOT mixed.
        let bytes = create_multi_resource_logs(vec![
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/unknown"),
            )],
            vec![], // no routing key
        ]);
        let data = RawLogsData::new(&bytes);
        // Should resolve to MissingKey (preserved for metric accuracy)
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MissingKey
        ));
    }

    #[test]
    fn test_resolve_empty_batch() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // Empty batch (0 resources) should resolve to MissingKey
        let request = ExportLogsServiceRequest::new(vec![]);
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        let bytes = Bytes::from(buf);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MissingKey
        ));
    }

    #[test]
    fn test_resolve_non_string_attribute_returns_no_match() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // Routing key exists but has an integer value — should be NoMatch, not MissingKey
        let bytes = create_multi_resource_logs(vec![vec![KeyValue::new(
            "service.namespace",
            AnyValue::new_int(42),
        )]]);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::NoMatch
        ));
    }

    #[test]
    fn test_resolve_matched_plus_no_match_is_mixed() {
        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // First resource matches, second has unrecognized value → MixedBatch
        let bytes = create_multi_resource_logs(vec![
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/aaa"),
            )],
            vec![KeyValue::new(
                "service.namespace",
                AnyValue::new_string("/subscriptions/unknown"),
            )],
        ]);
        let data = RawLogsData::new(&bytes);
        assert!(matches!(
            router.resolve_logs_route(&data),
            RouteResolution::MixedBatch
        ));
    }

    // -------------------------------------------------------
    // Arrow ConversionError test
    // -------------------------------------------------------

    #[test]
    fn test_resolve_arrow_logs_conversion_error() {
        use otap_df_pdata::otap::{Logs, OtapArrowRecords};

        let routes = HashMap::from([("/subscriptions/aaa".to_string(), "tenant_a".to_string())]);
        let router = ContentRouter::new(make_config(routes, None));

        // Default Logs has no record batches, so OtapLogsView::try_from fails
        let arrow = OtapArrowRecords::Logs(Logs::default());
        assert!(matches!(
            router.resolve_arrow_logs_route(&arrow),
            RouteResolution::ConversionError
        ));
    }

    // -------------------------------------------------------
    // Pipeline integration test
    // -------------------------------------------------------

    #[test]
    fn test_process_control_message() {
        let test_runtime = TestRuntime::new();
        let config = ContentRouterConfig {
            routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
            routes: HashMap::from([("/sub/a".to_string(), "tenant_a".to_string())]),
            default_output: None,
            case_sensitive: true,
        };
        let user_cfg = Arc::new(NodeUserConfig::new_processor_config(CONTENT_ROUTER_URN));
        let wrapper = ProcessorWrapper::local(
            ContentRouter::new(config),
            test_node(test_runtime.config().name.clone()),
            user_cfg,
            test_runtime.config(),
        );

        let validation = test_runtime.set_processor(wrapper).run_test(|mut ctx| {
            Box::pin(async move {
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("control processing failed");
                assert!(ctx.drain_pdata().await.is_empty());
            })
        });

        validation.validate(|_| async {});
    }

    // -------------------------------------------------------
    // Telemetry metrics tests
    // -------------------------------------------------------

    mod telemetry {
        use super::*;
        use crate::pdata::OtapPdata;
        use otap_df_channel::mpsc;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::NodeControlMsg;
        use otap_df_engine::local::message::LocalSender;
        use otap_df_engine::local::processor::{
            EffectHandler as LocalEffectHandler, Processor as _,
        };
        use otap_df_engine::message::{Message, Sender};
        use otap_df_engine::testing::setup_test_runtime;
        use otap_df_telemetry::InternalTelemetrySystem;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;
        use std::collections::HashMap;
        use std::time::Duration;
        use tokio::task::JoinHandle;

        fn collect_metrics_map(
            telemetry_registry: &TelemetryRegistryHandle,
        ) -> HashMap<String, u64> {
            let mut out = HashMap::new();
            telemetry_registry.visit_current_metrics(|_desc, _attrs, iter| {
                for (field, value) in iter {
                    let _ = out.insert(field.name.to_string(), value.to_u64_lossy());
                }
            });
            out
        }

        fn start_telemetry() -> (TelemetryRegistryHandle, MetricsReporter, JoinHandle<()>) {
            let telemetry = InternalTelemetrySystem::default();
            let telemetry_registry = telemetry.registry();
            let reporter = telemetry.reporter();
            let collector_task = tokio::task::spawn_local(async move {
                let collector = telemetry.collector();
                let _ = collector.run_collection_loop().await;
            });
            (telemetry_registry, reporter, collector_task)
        }

        fn stop_telemetry(reporter: MetricsReporter, collector_task: JoinHandle<()>) {
            drop(reporter);
            collector_task.abort();
        }

        #[test]
        fn test_metrics_routed_success() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("content_router_test");

                let config = ContentRouterConfig {
                    routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
                    routes: HashMap::from([("/sub/a".to_string(), "tenant_a".to_string())]),
                    default_output: None,
                    case_sensitive: true,
                };
                let mut router = ContentRouter::with_pipeline_ctx(pipeline, config);

                let (tx, rx) = mpsc::Channel::new(4);
                let mut senders = HashMap::new();
                let _ = senders.insert("tenant_a".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let bytes = create_logs_with_resource_attr("service.namespace", "/sub/a");
                let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed");

                let _received = rx.recv().await.expect("no message on tenant_a port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");

                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(metrics.get("signals.received").copied().unwrap_or(0), 1);
                assert_eq!(metrics.get("signals.routed").copied().unwrap_or(0), 1);
                assert_eq!(metrics.get("signals.nacked").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_no_match_nacked() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("content_router_nack_test");

                let config = ContentRouterConfig {
                    routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
                    routes: HashMap::from([("/sub/a".to_string(), "tenant_a".to_string())]),
                    default_output: None,
                    case_sensitive: true,
                };
                let mut router = ContentRouter::with_pipeline_ctx(pipeline, config);

                let senders = HashMap::new();
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                let bytes = create_logs_with_resource_attr("service.namespace", "/sub/unknown");
                let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router should NACK gracefully");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");

                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(metrics.get("signals.received").copied().unwrap_or(0), 1);
                assert_eq!(metrics.get("signals.nacked").copied().unwrap_or(0), 1);
                assert_eq!(metrics.get("signals.routed").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_routed_to_default() {
            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("content_router_default_test");

                let config = ContentRouterConfig {
                    routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
                    routes: HashMap::from([("/sub/a".to_string(), "tenant_a".to_string())]),
                    default_output: Some("fallback".to_string()),
                    case_sensitive: true,
                };
                let mut router = ContentRouter::with_pipeline_ctx(pipeline, config);

                let (tx, rx) = mpsc::Channel::new(4);
                let mut senders = HashMap::new();
                let _ = senders.insert("fallback".into(), Sender::Local(LocalSender::mpsc(tx)));
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                // Send with non-matching route - should go to default
                let bytes = create_logs_with_resource_attr("service.namespace", "/sub/unknown");
                let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router failed");

                let _received = rx.recv().await.expect("no message on fallback port");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");

                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(metrics.get("signals.received").copied().unwrap_or(0), 1);
                assert_eq!(
                    metrics.get("signals.routed.default").copied().unwrap_or(0),
                    1
                );
                assert_eq!(metrics.get("signals.nacked").copied().unwrap_or(0), 0);

                stop_telemetry(reporter, collector_task);
            }));
        }

        #[test]
        fn test_metrics_conversion_error_nacked() {
            use otap_df_pdata::otap::{Logs, OtapArrowRecords};

            let (rt, local) = setup_test_runtime();
            rt.block_on(local.run_until(async move {
                let (telemetry_registry, reporter, collector_task) = start_telemetry();

                let controller = ControllerContext::new(telemetry_registry.clone());
                let pipeline =
                    controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
                let node_id = test_node("content_router_conversion_error_test");

                let config = ContentRouterConfig {
                    routing_key: RoutingKeyExpr::ResourceAttribute("service.namespace".to_string()),
                    routes: HashMap::from([("/sub/a".to_string(), "tenant_a".to_string())]),
                    default_output: None,
                    case_sensitive: true,
                };
                let mut router = ContentRouter::with_pipeline_ctx(pipeline, config);

                let senders = HashMap::new();
                let mut eh =
                    LocalEffectHandler::new(node_id.clone(), senders, None, reporter.clone());

                // Default Logs Arrow records have no batches -> ConversionError
                let arrow = OtapArrowRecords::Logs(Logs::default());
                let pdata = OtapPdata::new_default(arrow.into());
                router
                    .process(Message::PData(pdata), &mut eh)
                    .await
                    .expect("router should NACK conversion error gracefully");

                router
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter.clone(),
                        }),
                        &mut eh,
                    )
                    .await
                    .expect("collect telemetry failed");

                tokio::time::sleep(Duration::from_millis(50)).await;

                let metrics = collect_metrics_map(&telemetry_registry);
                assert_eq!(metrics.get("signals.received").copied().unwrap_or(0), 1);
                assert_eq!(metrics.get("signals.nacked").copied().unwrap_or(0), 1);
                assert_eq!(
                    metrics
                        .get("signals.conversion.error")
                        .copied()
                        .unwrap_or(0),
                    1
                );

                stop_telemetry(reporter, collector_task);
            }));
        }
    }
}
