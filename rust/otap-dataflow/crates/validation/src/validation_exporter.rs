// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter used in validation pipelines that compares control and
//! system-under-validation outputs and records pass/fail metrics.

use crate::ValidationInstructions;
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::NodeId as NodeName;
use otel_arrow_dfe_config::context::transport_headers::TransportHeaders;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ExporterFactory;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::pdata_context::{
    ConfigNodeContextDeclaration, ContextConsumerSelector, ContextDeclaration,
    ContextDeclarationProvider, ContextDomain, ContextEntrySelector, ContextEntrySelectorForm,
    ContextEntryTarget, ContextNameMatching, NodeContextDeclarations,
};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::pdata::{OtapPdata, PdataEffectHandlerExtension};
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata_codec::{EncodePolicy, PdataEncoding};
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry::otel_error;
use otel_arrow_dfe_telemetry_macros::metric_set;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// Default number of seconds the exporter waits without receiving any messages
/// before declaring the data stream settled and performing the final validation.
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 3;

/// URN that identifies the validation exporter within OTAP pipelines.
pub const VALIDATION_EXPORTER_URN: &str = "urn:otel:exporter:validation";

#[derive(Debug, Deserialize)]
struct ValidationExporterConfig {
    suv_input: NodeName,
    #[serde(default)]
    control_inputs: Vec<NodeName>,
    /// Validation rules to run.
    #[serde(default)]
    validations: Vec<ValidationInstructions>,
    /// Seconds to wait with no incoming messages before declaring the stream
    /// settled and performing the final validation check.
    #[serde(default = "default_idle_timeout_secs")]
    idle_timeout_secs: u64,
}

fn default_idle_timeout_secs() -> u64 {
    DEFAULT_IDLE_TIMEOUT_SECS
}

#[metric_set(name = "exporter.validation")]
#[derive(Debug, Default, Clone)]
struct ValidationExporterMetrics {
    /// Number of validation checks that did not match expectation
    #[metric(name = "check.failed", unit = "{check}")]
    failed_checks: otel_arrow_dfe_telemetry::instrument::Counter<u64>,
    /// Number of validation checks that did match expectation
    #[metric(name = "check.passed", unit = "{check}")]
    passed_checks: otel_arrow_dfe_telemetry::instrument::Counter<u64>,
    /// The value of the last comparison result
    /// 0 -> not valid
    /// 1 -> valid
    #[metric(unit = "{input}")]
    valid: otel_arrow_dfe_telemetry::instrument::Gauge<u64>,
    /// Whether the exporter has finished processing
    /// 0 -> still receiving / processing
    /// 1 -> idle timeout reached, final validation performed
    #[metric(unit = "{state}")]
    finished: otel_arrow_dfe_telemetry::instrument::Gauge<u64>,
}

/// Exporter that compares control and suv pipeline outputs and reports equivalence metrics.
pub struct ValidationExporter {
    suv_index: usize,
    control_indices: HashSet<usize>,
    validations: Vec<ValidationInstructions>,
    control_msgs: Vec<OtlpProtoMessage>,
    suv_msgs: Vec<(OtlpProtoMessage, Duration)>,
    /// Transport headers extracted from each SUV message's pipeline context.
    /// Stored separately from signal data since header validation is
    /// independent of the OTLP payload.
    suv_transport_headers: Vec<Option<TransportHeaders>>,
    metrics: MetricSet<ValidationExporterMetrics>,
    /// Duration to wait with no incoming messages before declaring the stream
    /// settled and performing the final validation.
    idle_timeout: Duration,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
/// Distributed-slice factory that registers the validation exporter with the engine.
pub static VALIDATION_EXPORTER_FACTORY: ExporterFactory<OtapPdata> = ExporterFactory {
    name: VALIDATION_EXPORTER_URN,
    create:
        |pipeline_ctx: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            Ok(ExporterWrapper::local(
                ValidationExporter::from_config(pipeline_ctx, &node_config.config)?,
                node,
                node_config,
                exporter_config,
            ))
        },
    context_declarations: Some(ContextDeclarationProvider::from_typed_config::<
        ValidationExporterConfig,
    >()),
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<
        ValidationExporterConfig,
    >,
};

impl ConfigNodeContextDeclaration for ValidationExporterConfig {
    fn context_declarations(&self) -> NodeContextDeclarations {
        let mut require_key_names = std::collections::BTreeSet::new();
        let mut require_key_value_names = std::collections::BTreeSet::new();
        let mut deny_names = std::collections::BTreeSet::new();

        for instruction in &self.validations {
            match instruction {
                ValidationInstructions::TransportHeaderRequireKey { keys } => {
                    require_key_names.extend(keys.iter().cloned());
                }
                ValidationInstructions::TransportHeaderRequireKeyValue { pairs } => {
                    require_key_value_names.extend(pairs.iter().map(|pair| pair.key.clone()));
                }
                ValidationInstructions::TransportHeaderDeny { keys } => {
                    deny_names.extend(keys.iter().cloned());
                }
                ValidationInstructions::Equivalence
                | ValidationInstructions::SignalDrop { .. }
                | ValidationInstructions::BatchItems { .. }
                | ValidationInstructions::BatchBytes { .. }
                | ValidationInstructions::AttributeDeny { .. }
                | ValidationInstructions::AttributeRequireKey { .. }
                | ValidationInstructions::AttributeRequireKeyValue { .. }
                | ValidationInstructions::AttributeNoDuplicate => {}
            }
        }

        [require_key_names, require_key_value_names, deny_names]
            .into_iter()
            .filter(|names| !names.is_empty())
            .map(|names| ContextDeclaration::Consumes {
                selector: ContextConsumerSelector::Entries {
                    entries: names
                        .into_iter()
                        .map(|name| ContextEntrySelector {
                            target: ContextEntryTarget::Primitive {
                                domain: ContextDomain::TransportHeader,
                                name,
                                matching: ContextNameMatching::Exact,
                            },
                            form: ContextEntrySelectorForm::Value,
                        })
                        .collect(),
                },
            })
            .collect()
    }
}

impl ValidationExporter {
    /// Run the configured validations and update metrics.
    fn validate_and_record(&mut self) {
        // The `OtlpProtoMessage` projection is built once here so that
        // multiple [`ValidationInstructions`] can share it without
        // redundant cloning.
        let suv_msgs: Vec<OtlpProtoMessage> =
            self.suv_msgs.iter().map(|(msg, _)| msg.clone()).collect();

        let mut valid = true;
        for instruction in &self.validations {
            valid &= instruction.validate(
                &self.control_msgs,
                &suv_msgs,
                &self.suv_msgs,
                &self.suv_transport_headers,
            );
        }

        if valid {
            self.metrics.passed_checks.add(1);
        } else {
            self.metrics.failed_checks.add(1);
        }
        self.metrics.valid.set(valid as u64);
    }

    /// Build a new exporter instance from user configuration embedded in the pipeline.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let metrics = ValidationExporterMetrics::register(&pipeline_ctx);
        let config: ValidationExporterConfig =
            serde_json::from_value(config.clone()).map_err(|e| {
                otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?;
        config.validate_context_declarations(&pipeline_ctx)?;
        let suv_node = pipeline_ctx
            .node_by_name(&config.suv_input)
            .ok_or_else(|| ConfigError::InvalidUserConfig {
                error: format!("unknown node name for suv_input: {}", config.suv_input),
            })?;
        let mut control_indices = HashSet::new();
        for ctrl in config.control_inputs.iter() {
            let ctrl_node =
                pipeline_ctx
                    .node_by_name(ctrl)
                    .ok_or_else(|| ConfigError::InvalidUserConfig {
                        error: format!("unknown node name for control_input: {ctrl}"),
                    })?;
            let _ = control_indices.insert(ctrl_node.index);
        }
        Ok(Self {
            suv_index: suv_node.index,
            control_indices,
            validations: config.validations,
            metrics,
            control_msgs: Vec::new(),
            suv_msgs: Vec::new(),
            suv_transport_headers: Vec::new(),
            idle_timeout: Duration::from_secs(config.idle_timeout_secs),
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ValidationExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let encoding_plan =
            effect_handler.resolve_encoding_plan(&PdataEncoding::OTLP, EncodePolicy::default())?;
        let mut time = Instant::now();
        let mut last_message_time = Instant::now();
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    // Check idle timeout: if enough time has passed since the
                    // last message and we have received at least one SUV
                    // message, perform the final validation and signal
                    // finished.
                    if last_message_time.elapsed() >= self.idle_timeout
                        && self.metrics.finished.get() != 1
                        && !self.suv_msgs.is_empty()
                    {
                        self.validate_and_record();
                        self.metrics.finished.set(1);
                    }
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    return Ok(TerminalState::new(deadline, [self.metrics]));
                }
                Message::PData(pdata) => {
                    last_message_time = Instant::now();
                    self.metrics.finished.set(0);
                    let time_elapsed = time.elapsed();
                    let (context, mut payload) = pdata.into_parts();
                    let source_node = context.source_node();
                    let transport_headers = context
                        .transport_headers()
                        .map(|headers| headers.to_transport_headers());
                    let signal = payload.signal_type();
                    let msg = effect_handler
                        .encode_owned(&mut payload, &encoding_plan)
                        .await
                        .ok()
                        .and_then(|bytes| {
                            OtlpProtoMessage::try_from(OtlpProtoBytes::new_from_bytes(
                                signal, bytes,
                            ))
                            .ok()
                        });

                    if let Some(msg) = msg {
                        if let Some(node_index) = source_node {
                            if node_index == self.suv_index {
                                self.suv_msgs.push((msg, time_elapsed));
                                self.suv_transport_headers.push(transport_headers);
                                time = Instant::now();
                            } else if self.control_indices.contains(&node_index) {
                                self.control_msgs.push(msg);
                            }
                        } else if self.control_indices.is_empty() {
                            self.suv_msgs.push((msg, time_elapsed));
                            self.suv_transport_headers.push(transport_headers);
                            time = Instant::now();
                        } else {
                            otel_error!("validation.missing.source");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::ContextEntryName;
    use otel_arrow_dfe_engine::pdata_context::{
        ContextConsumerSelector, ContextDeclaration, ContextEntrySelector,
    };

    fn context_name(raw: &str) -> ContextEntryName {
        ContextEntryName::try_from(raw).expect("valid test context entry name")
    }

    fn entries(names: &[&str]) -> Box<[ContextEntrySelector]> {
        names
            .iter()
            .map(|name| ContextEntrySelector {
                target: ContextEntryTarget::Primitive {
                    domain: ContextDomain::TransportHeader,
                    name: context_name(name),
                    matching: ContextNameMatching::Exact,
                },
                form: ContextEntrySelectorForm::Value,
            })
            .collect()
    }

    fn consumes(names: &[&str]) -> ContextDeclaration {
        ContextDeclaration::Consumes {
            selector: ContextConsumerSelector::Entries {
                entries: entries(names),
            },
        }
    }

    /// Scenario: validation config declares required keys, matching values, and forbidden headers.
    /// Guarantees: compiled reads preserve exact-case lookup, duplicate-value matching, and missing-header results.
    #[test]
    fn compiled_reads_match_validation_header_checks() {
        use crate::validation_types::transport_headers::{
            TransportHeaderKeyValue, validate_transport_header_deny_keys,
            validate_transport_header_require_key_values, validate_transport_header_require_keys,
        };
        use otel_arrow_dfe_config::PipelineKey;
        use otel_arrow_dfe_config::context::transport_headers::TransportHeader;
        use otel_arrow_dfe_config::engine::OtelDataflowSpec;
        use otel_arrow_dfe_engine::PipelineFactory;
        use otel_arrow_dfe_engine::pdata_context::Metadata;

        let spec: OtelDataflowSpec = serde_json::from_value(serde_json::json!({
            "version": "otel_dataflow/v1",
            "engine": {},
            "groups": {"group": {"pipelines": {"pipeline": {
                "nodes": {"exporter": {
                    "type": "urn:otel:exporter:validation",
                    "config": {
                        "suv_input": "suv",
                        "validations": [
                            {"type": "transport_header_require_key", "keys": ["Tenant", "tenant"]},
                            {"type": "transport_header_require_key_value", "pairs": [{"key": "Tenant", "value": "acme"}]},
                            {"type": "transport_header_deny", "keys": ["missing"]}
                        ]
                    }
                }},
                "connections": []
            }}}}
        })).expect("pipeline config");
        let mut resolved = spec.resolve();
        // This fixture does not register the implicit telemetry pipeline's nodes.
        resolved
            .pipelines
            .retain(|pipeline| pipeline.pipeline_group_id.as_ref() == "group");
        let prepared = PipelineFactory::new(
            &[],
            &[],
            std::slice::from_ref(&VALIDATION_EXPORTER_FACTORY),
            &[],
        )
        .compile_initial_context(&resolved)
        .expect("compiled validation config");
        let bindings = prepared
            .bindings
            .node_bindings(
                &PipelineKey::new("group".into(), "pipeline".into()),
                &"exporter".into(),
            )
            .expect("validation bindings");
        let mut headers = TransportHeaders::new();
        headers.push(TransportHeader::text(context_name("Tenant"), b"other"));
        headers.push(TransportHeader::text(context_name("Tenant"), b"acme"));
        for stored in [None, Some(headers)] {
            let mut metadata = Metadata::default();
            if let Some(headers) = &stored {
                metadata.set_transport_headers(headers.clone());
            }
            for read in bindings.values() {
                let name = read.reference().name().clone();
                let selected = read.read(&metadata);
                let present = selected.is_some();
                let mut matches_value = false;
                if let Some(selected) = selected {
                    selected.visit_members(|member| {
                        matches_value |= member.values().any(|value| value.bytes == b"acme");
                    });
                }
                assert_eq!(
                    present,
                    validate_transport_header_require_keys(
                        std::slice::from_ref(&stored),
                        std::slice::from_ref(&name)
                    ),
                );
                assert_eq!(
                    !present,
                    validate_transport_header_deny_keys(
                        std::slice::from_ref(&stored),
                        std::slice::from_ref(&name)
                    ),
                );
                assert_eq!(
                    matches_value,
                    validate_transport_header_require_key_values(
                        std::slice::from_ref(&stored),
                        &[TransportHeaderKeyValue::new(name, "acme")],
                    ),
                );
            }
        }
    }

    /// Scenario: validation checks header names and values.
    /// Guarantees: context reads are sorted and deduplicated.
    #[test]
    fn validation_requires_produce_consumer_declarations() {
        let config = serde_json::json!({
            "suv_input": "suv",
            "validations": [
                {
                    "type": "transport_header_require_key",
                    "keys": ["x-tenant-id", "x-request-id"]
                },
                {
                    "type": "transport_header_require_key_value",
                    "pairs": [{"key": "x-tenant-id", "value": "acme"}]
                },
                {
                    "type": "transport_header_deny",
                    "keys": ["x-secret"]
                }
            ]
        });

        let decls = (VALIDATION_EXPORTER_FACTORY
            .context_declarations
            .expect("validation exporter should declare context")
            .declarations)(&config)
        .unwrap();
        assert_eq!(
            decls,
            [
                consumes(&["x-request-id", "x-tenant-id"]),
                consumes(&["x-tenant-id"]),
                consumes(&["x-secret"]),
            ]
            .into_iter()
            .collect()
        );
    }

    /// Scenario: validation only checks for forbidden headers.
    /// Guarantees: forbidden names are declared as context reads.
    #[test]
    fn validation_deny_only_declares_context_entries() {
        let config = serde_json::json!({
            "suv_input": "suv",
            "validations": [
                {
                    "type": "transport_header_deny",
                    "keys": ["X-Secret"]
                }
            ]
        });

        assert_eq!(
            (VALIDATION_EXPORTER_FACTORY
                .context_declarations
                .expect("validation exporter should declare context")
                .declarations)(&config)
            .unwrap(),
            [consumes(&["X-Secret"])].into_iter().collect()
        );
    }

    /// Scenario: validation does not inspect transport headers.
    /// Guarantees: no context consumers are declared.
    #[test]
    fn validation_no_header_instructions_empty() {
        let config = serde_json::json!({
            "suv_input": "suv",
            "validations": [
                {"type": "equivalence"}
            ]
        });

        let decls = (VALIDATION_EXPORTER_FACTORY
            .context_declarations
            .expect("validation exporter should declare context")
            .declarations)(&config)
        .unwrap();
        assert!(decls.is_empty());
    }
}
