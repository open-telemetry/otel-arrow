// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared functionality for attributes processors.
//!
//! This module contains common code used by both the standard attributes processor
//! and the KQL-based attributes processor to avoid duplication.

use crate::attributes_processor::metrics::AttributesProcessorMetrics;
use crate::pdata::OtapPdata;
use otap_df_config::experimental::SignalType;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::otap::{
    OtapArrowRecords,
    transform::{AttributesTransform, transform_attributes_with_stats},
};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplyDomain {
    Signal,
    Resource,
    Scope,
}

/// Parse the `apply_to` configuration field into a set of domains.
pub fn parse_apply_to(apply_to: Option<&Vec<String>>) -> HashSet<ApplyDomain> {
    let mut set = HashSet::new();
    match apply_to {
        None => {
            // Default to signal only
            let _ = set.insert(ApplyDomain::Signal);
        }
        Some(list) => {
            for domain_str in list {
                match domain_str.to_lowercase().as_str() {
                    "signal" => {
                        let _ = set.insert(ApplyDomain::Signal);
                    }
                    "resource" => {
                        let _ = set.insert(ApplyDomain::Resource);
                    }
                    "scope" => {
                        let _ = set.insert(ApplyDomain::Scope);
                    }
                    _ => {
                        // For backward compatibility, silently ignore unknown domain strings
                        // and default to signal
                        let _ = set.insert(ApplyDomain::Signal);
                    }
                }
            }
        }
    }
    set
}

/// Get the appropriate payload types for attribute transformation based on signal type and domains.
pub fn attrs_payloads(
    signal: SignalType,
    has_resource_domain: bool,
    has_scope_domain: bool,
    has_signal_domain: bool,
) -> &'static [ArrowPayloadType] {
    use payload_sets::*;

    match (
        signal,
        has_resource_domain,
        has_scope_domain,
        has_signal_domain,
    ) {
        // No domains enabled - nothing to transform
        (_, false, false, false) => EMPTY,

        // Single domains
        (_, true, false, false) => RESOURCE_ONLY,
        (_, false, true, false) => SCOPE_ONLY,
        (SignalType::Logs, false, false, true) => LOGS_SIGNAL,
        (SignalType::Metrics, false, false, true) => METRICS_SIGNAL,
        (SignalType::Traces, false, false, true) => TRACES_SIGNAL,

        // Pairs of domains
        (_, true, true, false) => RESOURCE_SCOPE,
        (SignalType::Logs, true, false, true) => LOGS_RESOURCE_SIGNAL,
        (SignalType::Metrics, true, false, true) => METRICS_RESOURCE_SIGNAL,
        (SignalType::Traces, true, false, true) => TRACES_RESOURCE_SIGNAL,
        (SignalType::Logs, false, true, true) => LOGS_SCOPE_SIGNAL,
        (SignalType::Metrics, false, true, true) => METRICS_SCOPE_SIGNAL,
        (SignalType::Traces, false, true, true) => TRACES_SCOPE_SIGNAL,

        // All three domains
        (SignalType::Logs, true, true, true) => LOGS_ALL,
        (SignalType::Metrics, true, true, true) => METRICS_ALL,
        (SignalType::Traces, true, true, true) => TRACES_ALL,
    }
}

/// Apply attribute transformations with statistics tracking.
pub fn apply_transform_with_stats(
    transform: &AttributesTransform,
    records: &mut OtapArrowRecords,
    signal: SignalType,
    has_resource_domain: bool,
    has_scope_domain: bool,
    has_signal_domain: bool,
) -> Result<(u64, u64), EngineError> {
    let mut deleted_total: u64 = 0;
    let mut renamed_total: u64 = 0;

    // Only apply if we have transforms to apply
    if transform.rename.is_some() || transform.delete.is_some() {
        let payloads = attrs_payloads(
            signal,
            has_resource_domain,
            has_scope_domain,
            has_signal_domain,
        );
        for &payload_ty in payloads {
            if let Some(rb) = records.get(payload_ty) {
                let (rb, stats) = transform_attributes_with_stats(rb, transform)
                    .map_err(|e| engine_err(&format!("transform_attributes failed: {e}")))?;
                deleted_total += stats.deleted_entries;
                renamed_total += stats.renamed_entries;
                records.set(payload_ty, rb);
            }
        }
    }

    Ok((deleted_total, renamed_total))
}

/// Helper function to create an EngineError with a message.
pub fn engine_err(msg: &str) -> EngineError {
    EngineError::PdataConversionError {
        error: msg.to_string(),
    }
}

/// Check if the transform is a no-op (has no transformations).
pub const fn is_noop(transform: &AttributesTransform) -> bool {
    transform.rename.is_none() && transform.delete.is_none()
}

/// Trait for common attributes processor functionality.
pub trait AttributesProcessorTrait {
    /// Get the transform.
    fn transform(&self) -> &AttributesTransform;

    /// Get domain flags.
    fn domain_flags(&self) -> (bool, bool, bool); // (resource, scope, signal)

    /// Get mutable metrics reference.
    fn metrics_mut(&mut self) -> &mut Option<MetricSet<AttributesProcessorMetrics>>;

    /// Apply transform with stats tracking.
    fn apply_transform_with_stats(
        &self,
        records: &mut OtapArrowRecords,
        signal: SignalType,
    ) -> Result<(u64, u64), EngineError> {
        let (has_resource_domain, has_scope_domain, has_signal_domain) = self.domain_flags();
        apply_transform_with_stats(
            self.transform(),
            records,
            signal,
            has_resource_domain,
            has_scope_domain,
            has_signal_domain,
        )
    }
}

/// Shared process implementation for attributes processors.
pub async fn process_attributes<T: AttributesProcessorTrait>(
    processor: &mut T,
    msg: Message<OtapPdata>,
    effect_handler: &mut local::EffectHandler<OtapPdata>,
) -> Result<(), EngineError> {
    match msg {
        Message::Control(control_msg) => match control_msg {
            otap_df_engine::control::NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            } => {
                if let Some(metrics) = processor.metrics_mut() {
                    let _ = metrics_reporter.report(metrics);
                }
                Ok(())
            }
            _ => Ok(()),
        },
        Message::PData(pdata) => {
            if let Some(m) = processor.metrics_mut() {
                m.msgs_consumed.inc();
            }

            // Fast path: no actions to apply
            if is_noop(processor.transform()) {
                let res = effect_handler
                    .send_message(pdata)
                    .await
                    .map_err(|e| e.into());
                if res.is_ok() {
                    if let Some(m) = processor.metrics_mut() {
                        m.msgs_forwarded.inc();
                    }
                }
                return res;
            }

            let signal = pdata.signal_type();
            let (context, payload) = pdata.into_parts();

            let mut records: OtapArrowRecords = payload.try_into()?;

            // Update domain counters (count once per message when domains are enabled)
            let (has_resource_domain, has_scope_domain, has_signal_domain) =
                processor.domain_flags();
            if let Some(m) = processor.metrics_mut() {
                if has_resource_domain {
                    m.domains_resource.inc();
                }
                if has_scope_domain {
                    m.domains_scope.inc();
                }
                if has_signal_domain {
                    m.domains_signal.inc();
                }
            }

            // Apply transform across selected domains and collect exact stats
            match processor.apply_transform_with_stats(&mut records, signal) {
                Ok((deleted_total, renamed_total)) => {
                    if let Some(m) = processor.metrics_mut() {
                        if deleted_total > 0 {
                            m.deleted_entries.add(deleted_total);
                        }
                        if renamed_total > 0 {
                            m.renamed_entries.add(renamed_total);
                        }
                    }
                }
                Err(e) => {
                    if let Some(m) = processor.metrics_mut() {
                        m.transform_failed.inc();
                    }
                    return Err(e);
                }
            }

            let res = effect_handler
                .send_message(OtapPdata::new(context, records.into()))
                .await
                .map_err(|e| e.into());
            if res.is_ok() {
                if let Some(m) = processor.metrics_mut() {
                    m.msgs_forwarded.inc();
                }
            }
            res
        }
    }
}

// Pre-computed arrays for all domain combinations
pub mod payload_sets {
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType as A;

    pub const EMPTY: &[A] = &[];

    // Signal only
    pub const LOGS_SIGNAL: &[A] = &[A::LogAttrs];
    pub const METRICS_SIGNAL: &[A] = &[
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub const TRACES_SIGNAL: &[A] = &[A::SpanAttrs, A::SpanEventAttrs, A::SpanLinkAttrs];

    // Resource only
    pub const RESOURCE_ONLY: &[A] = &[A::ResourceAttrs];

    // Scope only
    pub const SCOPE_ONLY: &[A] = &[A::ScopeAttrs];

    // Resource + Signal
    pub const LOGS_RESOURCE_SIGNAL: &[A] = &[A::ResourceAttrs, A::LogAttrs];
    pub const METRICS_RESOURCE_SIGNAL: &[A] = &[
        A::ResourceAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub const TRACES_RESOURCE_SIGNAL: &[A] = &[
        A::ResourceAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];

    // Scope + Signal
    pub const LOGS_SCOPE_SIGNAL: &[A] = &[A::ScopeAttrs, A::LogAttrs];
    pub const METRICS_SCOPE_SIGNAL: &[A] = &[
        A::ScopeAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub const TRACES_SCOPE_SIGNAL: &[A] = &[
        A::ScopeAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];

    // Resource + Scope
    pub const RESOURCE_SCOPE: &[A] = &[A::ResourceAttrs, A::ScopeAttrs];

    // All three: Resource + Scope + Signal
    pub const LOGS_ALL: &[A] = &[A::ResourceAttrs, A::ScopeAttrs, A::LogAttrs];
    pub const METRICS_ALL: &[A] = &[
        A::ResourceAttrs,
        A::ScopeAttrs,
        A::MetricAttrs,
        A::NumberDpAttrs,
        A::HistogramDpAttrs,
        A::SummaryDpAttrs,
        A::NumberDpExemplarAttrs,
        A::HistogramDpExemplarAttrs,
    ];
    pub const TRACES_ALL: &[A] = &[
        A::ResourceAttrs,
        A::ScopeAttrs,
        A::SpanAttrs,
        A::SpanEventAttrs,
        A::SpanLinkAttrs,
    ];
}

#[cfg(test)]
pub mod test {
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::{
        config::ProcessorConfig,
        context::ControllerContext,
        context::PipelineContext,
        message::Message,
        node::NodeId,
        processor::ProcessorWrapper,
        testing::{node::test_node, processor::TestRuntime},
    };
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use prost::Message as _;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Type alias matching the factory functions for processors under test.
    pub type TestProcessorFactory =
        fn(
            PipelineContext,
            NodeId,
            Arc<NodeUserConfig>,
            &ProcessorConfig,
        ) -> Result<ProcessorWrapper<OtapPdata>, otap_df_config::error::Error>;

    /// Test helper function to build logs with attributes.
    pub fn build_logs_with_attrs(
        res_attrs: Vec<otel_arrow_rust::proto::opentelemetry::common::v1::KeyValue>,
        scope_attrs: Vec<otel_arrow_rust::proto::opentelemetry::common::v1::KeyValue>,
        log_attrs: Vec<otel_arrow_rust::proto::opentelemetry::common::v1::KeyValue>,
    ) -> ExportLogsServiceRequest {
        use otel_arrow_rust::proto::opentelemetry::{
            collector::logs::v1::ExportLogsServiceRequest,
            common::v1::{AnyValue, InstrumentationScope},
            logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
            resource::v1::Resource,
        };

        ExportLogsServiceRequest::new(vec![
            ResourceLogs {
                resource: Some(Resource {
                    attributes: res_attrs,
                    ..Default::default()
                }),
                scope_logs: vec![
                    ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "test_scope".to_string(),
                            version: "1.0.0".to_string(),
                            attributes: scope_attrs,
                            dropped_attributes_count: 0,
                        }),
                        log_records: vec![
                            LogRecord {
                                time_unix_nano: 1234567890,
                                observed_time_unix_nano: 1234567890,
                                severity_number: SeverityNumber::Info as i32,
                                severity_text: "INFO".to_string(),
                                body: Some(AnyValue {
                                    value: Some(otel_arrow_rust::proto::opentelemetry::common::v1::any_value::Value::StringValue("test log".to_string()))
                                }),
                                attributes: log_attrs,
                                dropped_attributes_count: 0,
                                flags: 0,
                                trace_id: vec![],
                                span_id: vec![],
                                event_name: "".to_string(),
                            }
                        ],
                        schema_url: "".to_string(),
                    }
                ],
                schema_url: "".to_string(),
            }
        ])
    }

    pub fn run_test_processor(
        urn: &'static str,
        cfg: Value,
        input: ExportLogsServiceRequest,
        factory: TestProcessorFactory,
    ) -> ExportLogsServiceRequest {
        use otap_df_telemetry::registry::MetricsRegistryHandle;

        let decoded_out: Arc<Mutex<Option<ExportLogsServiceRequest>>> = Arc::new(Mutex::new(None));

        // Pipeline & metrics context
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        // Runtime + processor
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let node = test_node("attributes-shared-test");
        let mut node_config = NodeUserConfig::new_processor_config(urn);
        node_config.config = cfg;
        let proc = factory(pipeline_ctx, node, Arc::new(node_config), rt.config())
            .expect("create test processor");
        let phase = rt.set_processor(proc);

        let decoded_ref = decoded_out.clone();
        let input_clone = input.clone();
        phase
            .run_test(move |mut ctx| {
                let input = input_clone.clone();
                async move {
                    let mut bytes = Vec::new();
                    input.encode(&mut bytes).expect("encode");
                    let pdata_in = OtapPdata::new(
                        Default::default(),
                        OtlpProtoBytes::ExportLogsRequest(bytes).into(),
                    );
                    ctx.process(Message::PData(pdata_in))
                        .await
                        .expect("process");

                    let out = ctx.drain_pdata().await;
                    let first = out.into_iter().next().expect("one output");
                    let (_, payload) = first.into_parts();
                    let otlp_bytes: OtlpProtoBytes = payload.try_into().expect("convert to otlp");
                    let bytes = match otlp_bytes {
                        OtlpProtoBytes::ExportLogsRequest(b) => b,
                        _ => panic!("unexpected otlp variant"),
                    };
                    let decoded =
                        ExportLogsServiceRequest::decode(bytes.as_slice()).expect("decode");
                    *decoded_ref.lock().unwrap() = Some(decoded);
                }
            })
            .validate(|_| async move {});

        Arc::try_unwrap(decoded_out)
            .unwrap()
            .into_inner()
            .unwrap()
            .expect("processor produced no output")
    }
}
