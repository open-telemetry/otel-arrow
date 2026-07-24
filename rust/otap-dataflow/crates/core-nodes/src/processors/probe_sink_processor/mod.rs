// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor that detects probe log records emitted by `probe_emitter`,
//! records an end-to-end pipeline latency histogram, and (by default) drops
//! the probes so they do not reach downstream exporters.
//!
//! A "probe" is any [`LogRecord`] carrying the reserved attribute
//! [`PROBE_ID_ATTR`] together with [`PROBE_EMITTED_AT_ATTR`] holding the
//! emitter-side Unix-nanos timestamp. Real (non-probe) logs are forwarded
//! downstream unchanged.
//!
//! # Configuration
//!
//! ```yaml
//! probe-sink:
//!   type: urn:otel:processor:probe_sink
//!   config:
//!     drop_probes: true  # default; set to false to forward probes for debugging
//! ```
//!
//! The PoC accepts only `OtlpProtoBytes::ExportLogsRequest` log payloads. Non-log
//! payloads and `OtapArrowRecords` log payloads pass through untouched.

use async_trait::async_trait;
use bytes::BytesMut;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::ProcessorFactory;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value as AnyValueOneof;
use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
use otap_df_telemetry::metrics::MetricSet;
use prost::Message as _;
use std::sync::Arc;
use std::time::SystemTime;

use crate::receivers::probe_emitter_receiver::config::{PROBE_EMITTED_AT_ATTR, PROBE_ID_ATTR};

use self::config::Config;
use self::metrics::ProbeSinkMetrics;

pub mod config;
pub mod metrics;

/// URN identifying the probe-sink processor.
pub const PROBE_SINK_PROCESSOR_URN: &str = "urn:otel:processor:probe_sink";

/// Registers the probe-sink processor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static PROBE_SINK_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: PROBE_SINK_PROCESSOR_URN,
    create: create_probe_sink_processor,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

/// Factory function building a `ProbeSinkProcessor` wrapper.
pub fn create_probe_sink_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
    _capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse probe_sink configuration: {e}"),
        }
    })?;
    let metrics = pipeline_ctx.register_metrics::<ProbeSinkMetrics>();
    Ok(ProcessorWrapper::local(
        ProbeSinkProcessor { config, metrics },
        node,
        node_config,
        processor_config,
    ))
}

/// Per-batch processing outcome used by [`ProbeSinkProcessor::process_logs`].
struct ProbeScanOutcome {
    /// Encoded `ExportLogsServiceRequest` to forward downstream, if any.
    /// `None` means every log was a dropped probe and the entire batch can be skipped.
    forward_bytes: Option<BytesMut>,
    /// Number of probes observed in this batch.
    probes_observed: u64,
    /// Number of probes dropped (zero when `drop_probes` is false).
    probes_dropped: u64,
    /// Number of non-probe log records forwarded.
    non_probe_forwarded: u64,
    /// Number of probe records that were ill-formed.
    probes_invalid: u64,
    /// Latency samples (ns) recorded for valid probes, in observation order.
    latency_samples_ns: Vec<u64>,
}

/// Processor that records end-to-end probe latency and optionally drops probes.
pub struct ProbeSinkProcessor {
    config: Config,
    metrics: MetricSet<ProbeSinkMetrics>,
}

impl ProbeSinkProcessor {
    /// Scan a decoded `ExportLogsServiceRequest`, record latency for any probes,
    /// and (when configured) drop them in-place. Returns counters and a
    /// re-encoded payload ready to forward, or `None` if every log was a
    /// dropped probe.
    fn process_logs(
        &self,
        mut request: ExportLogsServiceRequest,
        now_unix_nanos: i64,
    ) -> Result<ProbeScanOutcome, EncodeError> {
        let mut probes_observed: u64 = 0;
        let mut probes_dropped: u64 = 0;
        let mut non_probe_forwarded: u64 = 0;
        let mut probes_invalid: u64 = 0;
        let mut latency_samples_ns: Vec<u64> = Vec::new();

        for resource in request.resource_logs.iter_mut() {
            for scope in resource.scope_logs.iter_mut() {
                let mut kept: Vec<LogRecord> = Vec::with_capacity(scope.log_records.len());
                for log in std::mem::take(&mut scope.log_records) {
                    match classify_log(&log) {
                        ProbeClassification::NotProbe => {
                            non_probe_forwarded += 1;
                            kept.push(log);
                        }
                        ProbeClassification::ProbeMalformed => {
                            probes_observed += 1;
                            probes_invalid += 1;
                            if !self.config.drop_probes {
                                kept.push(log);
                            } else {
                                probes_dropped += 1;
                            }
                        }
                        ProbeClassification::Probe {
                            emitted_at_unix_nanos,
                        } => {
                            probes_observed += 1;
                            let latency_ns = (now_unix_nanos - emitted_at_unix_nanos).max(0) as u64;
                            latency_samples_ns.push(latency_ns);
                            if !self.config.drop_probes {
                                kept.push(log);
                            } else {
                                probes_dropped += 1;
                            }
                        }
                    }
                }
                scope.log_records = kept;
            }
            // Prune scopes that became empty so we don't emit empty wrappers.
            resource.scope_logs.retain(|s| !s.log_records.is_empty());
        }
        request.resource_logs.retain(|r| !r.scope_logs.is_empty());

        let forward_bytes = if request.resource_logs.is_empty() {
            None
        } else {
            let mut buf = BytesMut::with_capacity(request.encoded_len());
            request
                .encode(&mut buf)
                .map_err(|e| EncodeError(format!("failed to re-encode logs: {e}")))?;
            Some(buf)
        };

        Ok(ProbeScanOutcome {
            forward_bytes,
            probes_observed,
            probes_dropped,
            non_probe_forwarded,
            probes_invalid,
            latency_samples_ns,
        })
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for ProbeSinkProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            }) => {
                let _ = metrics_reporter.report(&mut self.metrics);
                Ok(())
            }
            Message::Control(_) => Ok(()),
            Message::PData(pdata) => {
                if pdata.signal_type() != SignalType::Logs {
                    effect_handler.send_message_with_source_node(pdata).await?;
                    return Ok(());
                }
                let (context, payload) = pdata.into_parts();
                let OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) = payload
                else {
                    // Non-OTLP-bytes log payloads (e.g. OtapArrowRecords) are
                    // out of scope for the PoC; pass them through untouched.
                    effect_handler
                        .send_message_with_source_node(OtapPdata::new(context, payload))
                        .await?;
                    return Ok(());
                };

                let request = ExportLogsServiceRequest::decode(bytes.as_ref()).map_err(|e| {
                    Error::ProcessorError {
                        processor: effect_handler.processor_id(),
                        kind: ProcessorErrorKind::Other,
                        error: format!("failed to decode ExportLogsRequest: {e}"),
                        source_detail: String::new(),
                    }
                })?;

                let now_unix_nanos: i64 = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
                    .unwrap_or(0);

                let outcome =
                    self.process_logs(request, now_unix_nanos)
                        .map_err(|EncodeError(msg)| Error::ProcessorError {
                            processor: effect_handler.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error: msg,
                            source_detail: String::new(),
                        })?;

                self.metrics.probes_observed.add(outcome.probes_observed);
                self.metrics.probes_dropped.add(outcome.probes_dropped);
                self.metrics
                    .non_probe_logs_forwarded
                    .add(outcome.non_probe_forwarded);
                self.metrics.probes_invalid.add(outcome.probes_invalid);
                for sample in outcome.latency_samples_ns {
                    self.metrics.pipeline_e2e_latency_ns.record(sample as f64);
                }

                if let Some(buf) = outcome.forward_bytes {
                    let forwarded = OtapPdata::new(
                        context,
                        OtlpProtoBytes::ExportLogsRequest(buf.freeze()).into(),
                    );
                    effect_handler
                        .send_message_with_source_node(forwarded)
                        .await?;
                }
                Ok(())
            }
        }
    }
}

/// Local error type so `process_logs` can stay decoupled from the engine's
/// `Error` enum (we only have the processor id available at the call site).
#[derive(Debug)]
struct EncodeError(String);

/// Classification of a single [`LogRecord`] relative to probe attributes.
enum ProbeClassification {
    /// The record carries no probe id attribute.
    NotProbe,
    /// The record has a probe id but is missing or has malformed
    /// `emitted_at_unix_nanos`.
    ProbeMalformed,
    /// A well-formed probe with the extracted emit timestamp.
    Probe { emitted_at_unix_nanos: i64 },
}

fn classify_log(log: &LogRecord) -> ProbeClassification {
    let mut has_probe_id = false;
    let mut emitted_at: Option<i64> = None;
    for kv in &log.attributes {
        if kv.key == PROBE_ID_ATTR {
            has_probe_id = true;
        } else if kv.key == PROBE_EMITTED_AT_ATTR {
            if let Some(AnyValueOneof::IntValue(ns)) =
                kv.value.as_ref().and_then(|v| v.value.as_ref())
            {
                emitted_at = Some(*ns);
            }
        }
    }
    if !has_probe_id {
        return ProbeClassification::NotProbe;
    }
    match emitted_at {
        Some(ns) => ProbeClassification::Probe {
            emitted_at_unix_nanos: ns,
        },
        None => ProbeClassification::ProbeMalformed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn fresh_processor(drop_probes: bool) -> ProbeSinkProcessor {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx = controller.pipeline_context_with("g".into(), "p".into(), 0, 1, 0);
        ProbeSinkProcessor {
            config: Config { drop_probes },
            metrics: pipeline_ctx.register_metrics::<ProbeSinkMetrics>(),
        }
    }

    fn probe_log(emitted_at_ns: i64) -> LogRecord {
        LogRecord {
            attributes: vec![
                KeyValue::new(PROBE_ID_ATTR, AnyValue::new_string("abc")),
                KeyValue::new(PROBE_EMITTED_AT_ATTR, AnyValue::new_int(emitted_at_ns)),
            ],
            ..Default::default()
        }
    }

    fn non_probe_log() -> LogRecord {
        LogRecord {
            attributes: vec![KeyValue::new("k", AnyValue::new_string("v"))],
            ..Default::default()
        }
    }

    fn wrap(logs: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: logs,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    #[test]
    fn probe_only_batch_dropped_when_drop_probes_true() {
        let proc = fresh_processor(true);
        let now_ns: i64 = 1_000_000_000;
        let req = wrap(vec![probe_log(now_ns - 5_000_000)]); // emitted 5ms ago
        let outcome = proc.process_logs(req, now_ns).expect("process");
        assert!(
            outcome.forward_bytes.is_none(),
            "batch must be skipped entirely"
        );
        assert_eq!(outcome.probes_observed, 1);
        assert_eq!(outcome.probes_dropped, 1);
        assert_eq!(outcome.non_probe_forwarded, 0);
        assert_eq!(outcome.latency_samples_ns, vec![5_000_000]);
    }

    #[test]
    fn mixed_batch_keeps_only_non_probes_when_dropping() {
        let proc = fresh_processor(true);
        let now_ns: i64 = 2_000_000_000;
        let req = wrap(vec![
            non_probe_log(),
            probe_log(now_ns - 1_000_000),
            non_probe_log(),
        ]);
        let outcome = proc.process_logs(req, now_ns).expect("process");
        let buf = outcome.forward_bytes.expect("must forward remaining logs");
        let forwarded = ExportLogsServiceRequest::decode(buf.as_ref()).expect("decode");
        assert_eq!(
            forwarded.resource_logs[0].scope_logs[0].log_records.len(),
            2
        );
        assert_eq!(outcome.probes_observed, 1);
        assert_eq!(outcome.probes_dropped, 1);
        assert_eq!(outcome.non_probe_forwarded, 2);
        assert_eq!(outcome.latency_samples_ns, vec![1_000_000]);
    }

    #[test]
    fn drop_probes_false_passes_probes_through() {
        let proc = fresh_processor(false);
        let now_ns: i64 = 5_000;
        let req = wrap(vec![probe_log(now_ns - 1_000), non_probe_log()]);
        let outcome = proc.process_logs(req, now_ns).expect("process");
        let buf = outcome.forward_bytes.expect("must forward");
        let forwarded = ExportLogsServiceRequest::decode(buf.as_ref()).expect("decode");
        assert_eq!(
            forwarded.resource_logs[0].scope_logs[0].log_records.len(),
            2
        );
        assert_eq!(outcome.probes_dropped, 0);
        assert_eq!(outcome.probes_observed, 1);
    }

    #[test]
    fn malformed_probe_counted_as_invalid() {
        let proc = fresh_processor(true);
        // probe id present but no emitted_at attribute
        let log = LogRecord {
            attributes: vec![KeyValue::new(PROBE_ID_ATTR, AnyValue::new_string("x"))],
            ..Default::default()
        };
        let req = wrap(vec![log]);
        let outcome = proc.process_logs(req, 1_000).expect("process");
        assert_eq!(outcome.probes_observed, 1);
        assert_eq!(outcome.probes_invalid, 1);
        assert!(outcome.latency_samples_ns.is_empty());
    }

    #[test]
    fn classify_negative_skew_clamps_to_zero() {
        // If the receiver's clock is slightly ahead of the sink's, the
        // subtraction can go negative; we should clamp to 0 rather than
        // wrap or panic.
        let proc = fresh_processor(true);
        let now_ns: i64 = 100;
        let req = wrap(vec![probe_log(now_ns + 1_000)]); // emitted "in the future"
        let outcome = proc.process_logs(req, now_ns).expect("process");
        assert_eq!(outcome.latency_samples_ns, vec![0]);
    }
}
