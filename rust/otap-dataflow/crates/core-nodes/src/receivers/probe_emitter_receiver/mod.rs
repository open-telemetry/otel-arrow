// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A receiver that periodically emits a synthetic OTLP log record stamped
//! with probe metadata, used for end-to-end pipeline latency measurement.
//!
//! Each emitted [`LogRecord`] carries three reserved attributes:
//! - `_otap_internal.probe.id` — a UUIDv4 string uniquely identifying the probe
//! - `_otap_internal.probe.emitted_at_unix_nanos` — emission timestamp in Unix
//!   nanoseconds (the wall-clock used for latency math at the sink)
//! - `_otap_internal.probe.source` — caller-configured source identifier
//!
//! Pair this receiver with the `probe_sink_processor`, which detects probe
//! records, records an end-to-end latency histogram, and drops them so they
//! do not reach the real backend.
//!
//! # Configuration
//!
//! ```yaml
//! probe:
//!   type: urn:otel:receiver:probe_emitter
//!   config:
//!     interval: 1s        # humantime duration, default: 1s
//!     source: ingest-pod  # logical source name, default: "default"
//! ```

use async_trait::async_trait;
use bytes::BytesMut;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use otap_df_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use prost::Message;
use serde_json::Value;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{MissedTickBehavior, interval};
use uuid::Uuid;

use self::config::{Config, PROBE_EMITTED_AT_ATTR, PROBE_ID_ATTR, PROBE_SOURCE_ATTR};
use self::metrics::ProbeEmitterMetrics;

pub mod config;
pub mod metrics;

/// URN identifying the probe-emitter receiver.
pub const PROBE_EMITTER_RECEIVER_URN: &str = "urn:otel:receiver:probe_emitter";

/// Registers the probe-emitter receiver as an OTAP receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static PROBE_EMITTER_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: PROBE_EMITTER_RECEIVER_URN,
    create: |pipeline_ctx: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        let config = ProbeEmitterReceiver::parse_config(&node_config.config)?;
        Ok(ReceiverWrapper::local(
            ProbeEmitterReceiver::new(pipeline_ctx, config),
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

/// Receiver that periodically emits probe log records.
pub struct ProbeEmitterReceiver {
    config: Config,
    metrics: MetricSet<ProbeEmitterMetrics>,
}

impl ProbeEmitterReceiver {
    /// Create a new receiver with the given configuration.
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = pipeline_ctx.register_metrics::<ProbeEmitterMetrics>();
        Self { config, metrics }
    }

    /// Parse configuration from a JSON value.
    pub fn parse_config(config: &Value) -> Result<Config, otap_df_config::error::Error> {
        serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })
    }

    /// Build an `OtapPdata` carrying a single probe log record stamped with
    /// the current Unix-nanos timestamp and a freshly minted UUID.
    fn build_probe_pdata(source: &str) -> OtapPdata {
        let emitted_at_unix_nanos: i64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let probe_id = Uuid::new_v4().to_string();

        let log = LogRecord {
            time_unix_nano: emitted_at_unix_nanos as u64,
            observed_time_unix_nano: emitted_at_unix_nanos as u64,
            attributes: vec![
                KeyValue::new(PROBE_ID_ATTR, AnyValue::new_string(probe_id)),
                KeyValue::new(
                    PROBE_EMITTED_AT_ATTR,
                    AnyValue::new_int(emitted_at_unix_nanos),
                ),
                KeyValue::new(PROBE_SOURCE_ATTR, AnyValue::new_string(source.to_owned())),
            ],
            ..Default::default()
        };

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![log],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let mut buf = BytesMut::with_capacity(request.encoded_len());
        request
            .encode(&mut buf)
            .expect("encode ExportLogsServiceRequest");

        OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.freeze()).into(),
        )
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for ProbeEmitterReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let mut metrics = self.metrics;
        let source = self.config.source.clone();
        let mut ticker = interval(self.config.interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        // First tick fires immediately; consume it so the first probe is
        // delayed by `interval` rather than fired at startup.
        let _ = ticker.tick().await;

        loop {
            tokio::select! {
                biased;

                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(
                                deadline,
                                [],
                            ));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(
                                deadline,
                                [],
                            ));
                        }
                        Ok(NodeControlMsg::CollectTelemetry {
                            mut metrics_reporter,
                        }) => {
                            let _ = metrics_reporter.report(&mut metrics);
                        }
                        Ok(_) => { /* ignore other control messages */ }
                        Err(e) => return Err(Error::ChannelRecvError(e)),
                    }
                }

                _ = ticker.tick() => {
                    let pdata = Self::build_probe_pdata(&source);
                    match effect_handler.send_message(pdata).await {
                        Ok(()) => metrics.probes_emitted.inc(),
                        Err(_) => metrics.probes_send_failed.inc(),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use self::config::DEFAULT_INTERVAL;
    use super::*;
    use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value as AnyValueOneof;

    #[test]
    fn build_probe_pdata_encodes_required_attributes() {
        let pdata = ProbeEmitterReceiver::build_probe_pdata("unit-test");
        assert_eq!(pdata.signal_type(), otap_df_config::SignalType::Logs);

        let (_ctx, payload) = pdata.into_parts();
        let bytes = match payload {
            otap_df_pdata::OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(b)) => b,
            other => panic!("expected ExportLogsRequest payload, got {other:?}"),
        };

        let req =
            ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode ExportLogsRequest");
        assert_eq!(req.resource_logs.len(), 1);
        assert_eq!(req.resource_logs[0].scope_logs.len(), 1);
        assert_eq!(req.resource_logs[0].scope_logs[0].log_records.len(), 1);

        let log = &req.resource_logs[0].scope_logs[0].log_records[0];
        let mut saw_id = false;
        let mut saw_emitted = false;
        let mut saw_source = false;
        for kv in &log.attributes {
            match kv.key.as_str() {
                PROBE_ID_ATTR => {
                    saw_id = true;
                    let inner = kv.value.as_ref().and_then(|v| v.value.as_ref());
                    assert!(matches!(inner, Some(AnyValueOneof::StringValue(s)) if !s.is_empty()));
                }
                PROBE_EMITTED_AT_ATTR => {
                    saw_emitted = true;
                    let inner = kv.value.as_ref().and_then(|v| v.value.as_ref());
                    assert!(matches!(inner, Some(AnyValueOneof::IntValue(ns)) if *ns > 0));
                }
                PROBE_SOURCE_ATTR => {
                    saw_source = true;
                    let inner = kv.value.as_ref().and_then(|v| v.value.as_ref());
                    assert!(
                        matches!(inner, Some(AnyValueOneof::StringValue(s)) if s == "unit-test")
                    );
                }
                _ => {}
            }
        }
        assert!(
            saw_id && saw_emitted && saw_source,
            "probe must carry id, emitted_at, and source attributes"
        );
    }

    #[test]
    fn config_uses_defaults_when_unspecified() {
        let cfg: Config = serde_json::from_value(serde_json::json!({})).expect("default config");
        assert_eq!(cfg.interval, DEFAULT_INTERVAL);
        assert_eq!(cfg.source, "default");
    }

    #[test]
    fn config_parses_humantime_interval() {
        let cfg: Config =
            serde_json::from_value(serde_json::json!({"interval": "250ms"})).expect("parse cfg");
        assert_eq!(cfg.interval, std::time::Duration::from_millis(250));
    }
}
