// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and drains
//! internal metrics from the telemetry registry. It emits both signals as
//! OTLP export requests into the observability pipeline.

use async_trait::async_trait;
use bytes::Bytes;
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
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_telemetry::event::{LogEvent, ObservedEvent};
use otap_df_telemetry::metrics::MetricSetSnapshot;
use otap_df_telemetry::metrics::otlp::MetricsOtlpEncoder;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

/// The URN for the internal telemetry receiver.
pub use otap_df_telemetry::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {}

/// A receiver that emits internal logs and metrics as OTLP data.
pub struct InternalTelemetryReceiver {
    #[allow(dead_code)]
    config: Config,
    /// Internal telemetry settings obtained from the pipeline context during construction.
    /// Contains the logs receiver channel, pre-encoded resource bytes, and registry handle.
    internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
}

/// Declares the internal telemetry receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create: |mut pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        // Get internal telemetry settings from the pipeline context
        let internal_telemetry = pipeline.take_internal_telemetry().ok_or_else(|| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: "InternalTelemetryReceiver requires internal telemetry settings in pipeline context".to_owned(),
            }
        })?;

        Ok(ReceiverWrapper::local(
            InternalTelemetryReceiver::new_with_telemetry(
                InternalTelemetryReceiver::parse_config(&node_config.config)?,
                internal_telemetry,
            ),
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl InternalTelemetryReceiver {
    /// Create a new receiver with the given configuration and internal telemetry settings.
    #[must_use]
    pub const fn new_with_telemetry(
        config: Config,
        internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
    ) -> Self {
        Self {
            config,
            internal_telemetry,
        }
    }

    /// Parse configuration from a JSON value.
    pub fn parse_config(config: &Value) -> Result<Config, otap_df_config::error::Error> {
        serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let internal = self.internal_telemetry.clone();
        let logs_receiver = internal.logs_receiver;
        let resource_bytes = internal.resource_bytes;
        let log_tap = internal.log_tap;
        let registry = internal.registry;
        let mut scope_cache = ScopeToBytesMap::new(registry.clone());
        let metrics_encoder = internal
            .metrics_interval
            .map(|_| MetricsOtlpEncoder::new(&resource_bytes))
            .transpose()
            .map_err(|error| Error::PdataConversionError {
                error: error.to_string(),
            })?;
        let mut metrics_interval = internal.metrics_interval.map(|period| {
            let mut interval = interval_at(Instant::now() + period, period);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval
        });
        let mut logs_channel_open = true;

        loop {
            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    if let Some(log_tap) = log_tap.as_ref() {
                                        log_tap.record(log_event.clone());
                                    }
                                    Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                                }
                            }
                            Self::send_metric_batch(&effect_handler, &registry, metrics_encoder.as_ref()).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            // Drain any remaining logs from channel before shutdown
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    if let Some(log_tap) = log_tap.as_ref() {
                                        log_tap.record(log_event.clone());
                                    }
                                    Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                                }
                            }
                            Self::send_metric_batch(&effect_handler, &registry, metrics_encoder.as_ref()).await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                            // No metrics to report for now
                        }
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                             // Ignore other control messages
                        }
                    }
                }

                // Drain and emit registry metrics at the configured cold-path interval.
                _ = Self::next_metrics_tick(&mut metrics_interval) => {
                    Self::send_metric_batch(&effect_handler, &registry, metrics_encoder.as_ref()).await?;
                }

                // Receive logs from the channel
                result = logs_receiver.recv_async(), if logs_channel_open => {
                    match result {
                        Ok(ObservedEvent::Log(log_event)) => {
                            if let Some(log_tap) = log_tap.as_ref() {
                                log_tap.record(log_event.clone());
                            }
                            Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                        }
                        Ok(ObservedEvent::Engine(_)) => {
                            // Engine events are not yet processed
                        }
                        Err(_) => {
                            logs_channel_open = false;
                            if metrics_encoder.is_none() {
                                return Ok(TerminalState::default());
                            }
                        }
                    }
                }
            }
        }
    }
}

impl InternalTelemetryReceiver {
    async fn next_metrics_tick(interval: &mut Option<Interval>) {
        match interval {
            Some(interval) => {
                let _ = interval.tick().await;
            }
            None => std::future::pending().await,
        }
    }

    /// Drain accumulated registry metrics and send one direct OTLP request.
    async fn send_metric_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
    ) -> Result<(), Error> {
        let Some(encoder) = encoder else {
            return Ok(());
        };

        let batch = registry.drain_metric_export_batch();
        let Some(metrics) =
            encoder
                .encode(&batch)
                .map_err(|error| Error::PdataConversionError {
                    error: error.to_string(),
                })?
        else {
            return Ok(());
        };

        effect_handler
            .send_message(OtapPdata::new(Context::default(), metrics.into()))
            .await?;
        Ok(())
    }

    /// Send a log event as OTLP logs with scope attributes from entity context.
    async fn send_log_event(
        effect_handler: &local::EffectHandler<OtapPdata>,
        log_event: LogEvent,
        resource_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
    ) -> Result<(), Error> {
        let mut buf = ProtoBuffer::with_capacity(512);

        encode_export_logs_request(&mut buf, &log_event, resource_bytes, scope_cache);

        let pdata = OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into(),
        );
        effect_handler.send_message(pdata).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::control::{NodeControlMsg, runtime_ctrl_msg_channel};
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::receiver::Receiver as _;
    use otap_df_engine::message::{Receiver as EngineReceiver, Sender as EngineSender};
    use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otap_df_pdata::OtapPayload;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otap_df_telemetry::instrument::Counter;
    use otap_df_telemetry::metrics::MetricValue;
    use otap_df_telemetry::reporter::MetricsReporter;
    use otap_df_telemetry::testing::EmptyAttributes;
    use otap_df_telemetry_macros::metric_set;
    use prost::Message as _;
    use std::collections::HashMap;
    use std::time::{Duration, Instant as StdInstant};

    #[metric_set(name = "receiver.internal_telemetry.test")]
    #[derive(Debug, Default)]
    struct TestMetrics {
        /// Number of test events emitted.
        #[metric(unit = "{event}")]
        emitted: Counter<u64>,
    }

    fn decode_metric_value(pdata: OtapPdata) -> i64 {
        let OtapPayload::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) = pdata.payload()
        else {
            panic!("internal telemetry receiver emitted a non-metrics payload")
        };
        let request =
            ExportMetricsServiceRequest::decode(bytes).expect("valid OTLP metrics request");
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        let [scope_metrics] = resource_metrics.scope_metrics.as_slice() else {
            panic!("expected one scope metrics message")
        };
        assert_eq!(
            scope_metrics.scope.as_ref().expect("scope").name,
            "receiver.internal_telemetry.test"
        );
        let [metric] = scope_metrics.metrics.as_slice() else {
            panic!("expected one metric")
        };
        assert_eq!(metric.name, "emitted");
        let Some(metric::Data::Sum(sum)) = metric.data.as_ref() else {
            panic!("expected a sum metric")
        };
        let [point] = sum.data_points.as_slice() else {
            panic!("expected one metric data point")
        };
        let Some(number_data_point::Value::AsInt(value)) = point.value else {
            panic!("expected an integer metric data point")
        };
        value
    }

    #[test]
    fn emits_metrics_on_interval_and_drains_pending_metrics_on_shutdown() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set = registry.register_metric_set::<TestMetrics>(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                &[MetricValue::U64(3)],
            );

            let (_logs_sender, logs_receiver) = flume::unbounded();
            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {},
                otap_df_telemetry::InternalTelemetrySettings {
                    logs_receiver,
                    resource_bytes: ResourceLogs::default().encode_to_vec().into(),
                    registry: registry.clone(),
                    metrics_interval: Some(Duration::from_millis(100)),
                    log_tap: None,
                },
            );

            let (output_tx, output_rx) = create_not_send_channel(4);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(4);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(4);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let (ctrl_tx, ctrl_rx) = create_not_send_channel::<NodeControlMsg<OtapPdata>>(4);
            let ctrl_channel =
                local::ControlChannel::new(EngineReceiver::Local(LocalReceiver::mpsc(ctrl_rx)));
            let receiver_task = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_channel, effect_handler).await
            });

            let interval_output = tokio::time::timeout(Duration::from_secs(2), output_rx.recv())
                .await
                .expect("timed out waiting for periodic metrics")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(interval_output), 3);

            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                &[MetricValue::U64(5)],
            );
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .expect("shutdown control should be sent");

            let shutdown_output = tokio::time::timeout(Duration::from_secs(2), output_rx.recv())
                .await
                .expect("timed out waiting for final metrics drain")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(shutdown_output), 5);

            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should stop cleanly");
        }));
    }
}
