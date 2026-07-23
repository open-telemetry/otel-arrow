// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component wrappers that spin up the Kafka exporter/receiver against the
//! test suite's mock broker.
//!
//! The wrappers *consume* the test suite ([`super::test::cluster::KafkaTestCluster`]):
//! they take `&KafkaTestCluster`, auto-set `bootstrap.servers` from it, own all
//! engine wiring + `LocalSet` spawn + lifecycle, keep channel receiver-ends
//! alive internally, and expose intention-revealing handles. The test suite core
//! stays broker-only and node-agnostic; only these wrappers reference node
//! types.
//!
//! Each wrapper is gated by its node feature so it only compiles when that node
//! (and its `rdkafka`) is present.

#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
use otap_df_engine::context::ControllerContext;
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
use otap_df_engine::context::PipelineContext;
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
use otap_df_telemetry::registry::TelemetryRegistryHandle;

/// Builds a deterministic single-core pipeline context for the wrappers.
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
fn test_pipeline_context() -> PipelineContext {
    let registry = TelemetryRegistryHandle::new();
    let controller_ctx = ControllerContext::new(registry);
    controller_ctx.pipeline_context_with("test-group".into(), "test-pipeline".into(), 0, 1, 0)
}

/// Metric-observation helpers shared by the exporter and receiver harnesses.
///
/// Both harnesses read a node's final counters from the
/// [`otap_df_engine::terminal_state::TerminalState`] it returns at graceful
/// shutdown (via `await_terminal_state`). These helpers read individual counter
/// values out of the resulting [`MetricSetSnapshot`]s.
///
/// Metric field names follow the `#[metric_set]` convention where the Rust
/// field identifier's underscores become dots (e.g. `offset_commit_errors` ->
/// `offset.commit.errors`). Both helpers accept either spelling by normalizing
/// `_` to `.` before lookup.
// Consumed by the Kafka validation test branch; helpers may be unused on the
// branch that only finalizes the test suite.
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
#[allow(dead_code)]
pub(crate) mod node_metrics {
    use std::collections::HashMap;

    use otap_df_telemetry::metrics::MetricSetSnapshot;

    /// Normalizes a metric field name to the runtime dotted form so callers may
    /// pass either the Rust identifier (`offset_commit_errors`) or the emitted
    /// name (`offset.commit.errors`).
    fn normalize(name: &str) -> String {
        name.replace('_', ".")
    }

    /// Returns the `u64` value of the metric field `name` in `snapshot`, or
    /// `None` if the snapshot's descriptor has no such field.
    ///
    /// Values are read via `MetricValue::to_u64_lossy`, so float/MMSC metrics
    /// are coerced; the Kafka node counters are all `Counter<u64>`.
    #[must_use]
    pub(crate) fn metric_value(snapshot: &MetricSetSnapshot, name: &str) -> Option<u64> {
        let wanted = normalize(name);
        let fields = snapshot.descriptor().metrics;
        let values = snapshot.get_metrics();
        fields
            .iter()
            .zip(values.iter())
            .find(|(field, _)| field.name == wanted)
            .map(|(_, value)| value.to_u64_lossy())
    }

    /// Accumulates per-field metric values folded across several snapshots.
    ///
    /// A [`otap_df_engine::terminal_state::TerminalState`] can carry more than
    /// one [`MetricSetSnapshot`]; fold them together to read a single
    /// cumulative value per field.
    #[derive(Debug, Default, Clone)]
    pub(crate) struct FoldedMetrics {
        /// Cumulative value per dotted metric field name.
        totals: HashMap<String, u64>,
    }

    impl FoldedMetrics {
        /// Creates an empty accumulator.
        #[must_use]
        pub(crate) fn new() -> Self {
            Self::default()
        }

        /// Folds one snapshot's field values into the running totals.
        pub(crate) fn fold(&mut self, snapshot: &MetricSetSnapshot) {
            let fields = snapshot.descriptor().metrics;
            let values = snapshot.get_metrics();
            for (field, value) in fields.iter().zip(values.iter()) {
                *self.totals.entry(field.name.to_string()).or_insert(0) += value.to_u64_lossy();
            }
        }

        /// Folds an iterator of snapshots into the running totals.
        pub(crate) fn fold_all<'a, I>(&mut self, snapshots: I)
        where
            I: IntoIterator<Item = &'a MetricSetSnapshot>,
        {
            for snapshot in snapshots {
                self.fold(snapshot);
            }
        }

        /// Returns the cumulative value of `name` (accepts identifier or dotted
        /// form), or `0` if never observed.
        #[must_use]
        pub(crate) fn value(&self, name: &str) -> u64 {
            self.totals.get(&normalize(name)).copied().unwrap_or(0)
        }

        /// Returns `true` if any value has been folded for `name`.
        #[must_use]
        pub(crate) fn contains(&self, name: &str) -> bool {
            self.totals.contains_key(&normalize(name))
        }
    }
}

// ---------------------------------------------------------------------------
// Exporter wrapper
// ---------------------------------------------------------------------------

#[cfg(feature = "kafka-exporter")]
mod exporter_harness {
    use super::test_pipeline_context;
    use crate::common::kafka::test::cluster::KafkaTestCluster;
    use crate::common::kafka::test::error::TestError;
    use crate::exporters::kafka_exporter::config::{KafkaExporterConfig, SignalConfig};
    use crate::exporters::kafka_exporter::exporter::KAFKA_EXPORTER_URN;
    use crate::exporters::kafka_exporter::exporter::KafkaExporter;

    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::Interests;
    use otap_df_engine::config::ExporterConfig;
    use otap_df_engine::control::{
        Controllable, NodeControlMsg, pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
    };
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::terminal_state::TerminalState;
    use otap_df_engine::testing::{create_not_send_channel, test_node};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_telemetry::reporter::MetricsReporter;
    use tokio::task::JoinHandle;

    /// Opaque holder for channel read-ends kept alive for the harness lifetime.
    #[allow(dead_code)]
    struct KeepAlive(Vec<Box<dyn std::any::Any>>);

    /// A running Kafka exporter (sink) wired to the mock broker. Output lands in
    /// the broker and is observed via the test-suite consumer/inspector.
    pub(crate) struct KafkaExporterHarness {
        pdata_tx: Sender<OtapPdata>,
        control_tx: Sender<NodeControlMsg<OtapPdata>>,
        join: JoinHandle<Option<TerminalState>>,
        _keep_alive: KeepAlive,
    }

    impl KafkaExporterHarness {
        /// Starts the exporter with an explicit `cfg` (its `bootstrap.servers`
        /// must already point at `cluster`). Spawns onto the current `LocalSet`.
        pub(crate) fn start(_cluster: &KafkaTestCluster, cfg: KafkaExporterConfig) -> Self {
            let pipeline_ctx = test_pipeline_context();
            let node_config = Arc::new(NodeUserConfig::new_exporter_config(KAFKA_EXPORTER_URN));
            let exporter_config = ExporterConfig::new("test-kafka-exporter");
            let node_id = test_node(exporter_config.name.clone());

            let mut exporter = ExporterWrapper::local(
                KafkaExporter::new(pipeline_ctx, cfg).expect("kafka exporter config is valid"),
                node_id.clone(),
                node_config,
                &exporter_config,
            );

            let control_tx = exporter.control_sender();

            let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(32);
            let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
            let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
            exporter
                .set_pdata_receiver(node_id, pdata_rx)
                .expect("failed to set pdata receiver");

            let (runtime_ctrl_tx, runtime_ctrl_rx) = runtime_ctrl_msg_channel(16);
            let (completion_tx, completion_rx) = pipeline_completion_msg_channel(16);
            let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

            let keep_alive = KeepAlive(vec![
                Box::new(runtime_ctrl_rx),
                Box::new(completion_rx),
                Box::new(metrics_rx),
            ]);

            let join = tokio::task::spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_tx,
                        completion_tx,
                        metrics_reporter,
                        Interests::empty(),
                    )
                    .await
                    .ok()
            });

            Self {
                pdata_tx,
                control_tx,
                join,
                _keep_alive: keep_alive,
            }
        }

        /// Starts the exporter with a default config for `topics`.
        pub(crate) fn start_for(cluster: &KafkaTestCluster, topics: super::KafkaTopics) -> Self {
            use crate::exporters::kafka_exporter::config::KafkaExporterConfigBuilder;

            let mut builder =
                KafkaExporterConfigBuilder::new(cluster.bootstrap_servers(), "kafka-test-exporter");
            if let Some((topic, fmt)) = topics.traces {
                builder = builder.with_traces(SignalConfig::new(topic, fmt));
            }
            if let Some((topic, fmt)) = topics.metrics {
                builder = builder.with_metrics(SignalConfig::new(topic, fmt));
            }
            if let Some((topic, fmt)) = topics.logs {
                builder = builder.with_logs(SignalConfig::new(topic, fmt));
            }
            let cfg = builder
                .try_into()
                .expect("default exporter config is valid");
            Self::start(cluster, cfg)
        }

        /// Sends a pdata batch to the exporter.
        ///
        /// # Errors
        ///
        /// Returns [`TestError::Produce`] if the pdata channel is closed.
        pub(crate) async fn send_pdata(&self, pdata: OtapPdata) -> Result<(), TestError> {
            self.pdata_tx
                .send(pdata)
                .await
                .map_err(|e| TestError::Produce(format!("send pdata to exporter: {e}")))
        }

        /// Requests a graceful shutdown with the given `deadline` from now.
        pub(crate) async fn shutdown(&self, deadline: Duration) {
            self.control_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + deadline,
                    reason: "kafka-test shutdown".into(),
                })
                .await
                .expect("send shutdown to exporter");
        }

        /// Awaits the spawned exporter task's completion.
        pub(crate) async fn await_stopped(self) {
            let _ = self.join.await;
        }

        /// Awaits the spawned exporter task and returns the [`TerminalState`] it
        /// produced at graceful shutdown (the node's final metric snapshots).
        ///
        /// Returns `None` if the task panicked or the node returned an error
        /// instead of a terminal state. Read individual counters from the
        /// snapshots with [`super::node_metrics::metric_value`].
        pub(crate) async fn await_terminal_state(self) -> Option<TerminalState> {
            self.join.await.ok().flatten()
        }
    }
}

#[cfg(feature = "kafka-exporter")]
pub(crate) use exporter_harness::KafkaExporterHarness;

// ---------------------------------------------------------------------------
// Receiver wrapper
// ---------------------------------------------------------------------------

#[cfg(feature = "kafka-receiver")]
mod receiver_harness {
    use super::test_pipeline_context;
    use crate::common::kafka::test::cluster::KafkaTestCluster;
    use crate::receivers::kafka_receiver::config::{KafkaReceiverConfig, SignalConfig};
    use crate::receivers::kafka_receiver::receiver::KAFKA_RECEIVER_URN;
    use crate::receivers::kafka_receiver::receiver::KafkaReceiver;

    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use otap_df_channel::mpsc;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_config::transport_headers_policy::HeaderCapturePolicy;
    use otap_df_engine::control::{
        AckMsg, NodeControlMsg, RuntimeControlMsg, RuntimeCtrlMsgReceiver, runtime_ctrl_msg_channel,
    };
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::receiver as local;
    use otap_df_engine::local::receiver::Receiver as _;
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::terminal_state::TerminalState;
    use otap_df_engine::testing::test_node;
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::next_ack;
    use otap_df_telemetry::reporter::MetricsReporter;
    use tokio::task::JoinHandle;

    /// Default timeout for [`KafkaReceiverHarness::recv_pdata`].
    const DEFAULT_RECV_TIMEOUT: Duration = Duration::from_secs(30);

    /// Opaque holder for channel handles kept alive for the harness lifetime.
    #[allow(dead_code)]
    struct KeepAlive(Vec<Box<dyn std::any::Any>>);

    /// A running Kafka receiver (source) wired to the mock broker. Input comes
    /// from the broker (fed via the test-suite producer); decoded pdata is read via
    /// [`KafkaReceiverHarness::recv_pdata`] and acked via
    /// [`KafkaReceiverHarness::ack`].
    pub(crate) struct KafkaReceiverHarness {
        pdata_rx: Receiver<OtapPdata>,
        control_tx: mpsc::Sender<NodeControlMsg<OtapPdata>>,
        runtime_rx: RuntimeCtrlMsgReceiver<OtapPdata>,
        join: JoinHandle<Option<TerminalState>>,
        _keep_alive: KeepAlive,
    }

    impl KafkaReceiverHarness {
        /// Starts the receiver with an explicit `cfg` (its `bootstrap.servers`
        /// must already point at `cluster`). An optional `capture_policy` is
        /// installed on the effect handler before start. Spawns onto the
        /// current `LocalSet`.
        pub(crate) fn start_with_capture(
            _cluster: &KafkaTestCluster,
            cfg: KafkaReceiverConfig,
            capture_policy: Option<HeaderCapturePolicy>,
        ) -> Self {
            let pipeline_ctx = test_pipeline_context();
            let node_config = Arc::new(NodeUserConfig::new_receiver_config(KAFKA_RECEIVER_URN));
            let receiver = Box::new(
                KafkaReceiver::new(pipeline_ctx, cfg).expect("kafka receiver config is valid"),
            );

            let (control_sender, control_receiver) = mpsc::Channel::new(32);
            let control_receiver = LocalReceiver::mpsc(control_receiver);
            let ctrl_msg_chan = local::ControlChannel::new(Receiver::Local(control_receiver));

            let mut pdata_senders = HashMap::new();
            let (sender, recv) = mpsc::Channel::new(256);
            let pdata_sender = Sender::Local(LocalSender::mpsc(sender));
            let pdata_rx = Receiver::Local(LocalReceiver::mpsc(recv));
            let _ = pdata_senders.insert(Cow::Borrowed("test_receiver"), pdata_sender);

            let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = runtime_ctrl_msg_channel(10);
            let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let mut effect_handler = local::EffectHandler::new(
                test_node("test_receiver"),
                pdata_senders,
                node_config.default_output.clone(),
                pipeline_ctrl_msg_tx,
                metrics_reporter,
            );
            effect_handler.set_capture_policy(capture_policy);

            let keep_alive =
                KeepAlive(vec![Box::new(control_sender.clone()), Box::new(metrics_rx)]);

            let join = tokio::task::spawn_local(async move {
                receiver.start(ctrl_msg_chan, effect_handler).await.ok()
            });

            Self {
                pdata_rx,
                control_tx: control_sender,
                runtime_rx: pipeline_ctrl_msg_rx,
                join,
                _keep_alive: keep_alive,
            }
        }

        /// Starts the receiver with an explicit `cfg` and no capture policy.
        pub(crate) fn start(cluster: &KafkaTestCluster, cfg: KafkaReceiverConfig) -> Self {
            Self::start_with_capture(cluster, cfg, None)
        }

        /// Starts the receiver with a default OTLP-proto config for `topics`.
        pub(crate) fn start_for(cluster: &KafkaTestCluster, topics: super::KafkaTopics) -> Self {
            use crate::receivers::kafka_receiver::config::{
                AutoOffsetReset, KafkaReceiverConfigBuilder,
            };

            let mut builder = KafkaReceiverConfigBuilder::new(
                cluster.bootstrap_servers(),
                "kafka-test-group",
                "kafka-test-client",
            )
            .with_auto_offset_reset(AutoOffsetReset::Earliest);
            if let Some((topic, fmt)) = topics.traces {
                builder = builder.with_traces(SignalConfig::new(vec![topic]).with_encoding(fmt));
            }
            if let Some((topic, fmt)) = topics.metrics {
                builder = builder.with_metrics(SignalConfig::new(vec![topic]).with_encoding(fmt));
            }
            if let Some((topic, fmt)) = topics.logs {
                builder = builder.with_logs(SignalConfig::new(vec![topic]).with_encoding(fmt));
            }
            let cfg =
                KafkaReceiverConfig::try_from(builder).expect("default receiver config is valid");
            Self::start(cluster, cfg)
        }

        /// Receives one decoded pdata batch within the default 30s timeout.
        ///
        /// # Panics
        ///
        /// Panics if no pdata arrives before the timeout or the channel closes.
        pub(crate) async fn recv_pdata(&mut self) -> OtapPdata {
            self.try_recv_pdata(DEFAULT_RECV_TIMEOUT)
                .await
                .expect("kafka-test: timed out waiting for pdata from receiver")
        }

        /// Receives one decoded pdata batch within `timeout`.
        pub(crate) async fn try_recv_pdata(&mut self, timeout: Duration) -> Option<OtapPdata> {
            match tokio::time::timeout(timeout, self.pdata_rx.recv()).await {
                Ok(Ok(pdata)) => Some(pdata),
                Ok(Err(_)) | Err(_) => None,
            }
        }

        /// Receives one runtime-control message within `timeout` (e.g.
        /// `RuntimeControlMsg::ReceiverDrained`). Returns `None` on timeout or a
        /// closed channel.
        pub(crate) async fn try_recv_runtime(
            &mut self,
            timeout: Duration,
        ) -> Option<RuntimeControlMsg<OtapPdata>> {
            match tokio::time::timeout(timeout, self.runtime_rx.recv()).await {
                Ok(Ok(msg)) => Some(msg),
                Ok(Err(_)) | Err(_) => None,
            }
        }

        /// Acknowledges a consumed `pdata`, folding `next_ack` + `AckMsg` +
        /// control-channel send so manual-commit offsets advance.
        pub(crate) fn ack(&self, pdata: OtapPdata) {
            if let Some((_node_id, ack)) = next_ack(AckMsg::new(pdata)) {
                self.control_tx
                    .send(NodeControlMsg::Ack(ack))
                    .expect("send ack to receiver");
            }
        }

        /// Requests a graceful shutdown with the given `deadline` from now.
        pub(crate) fn shutdown(&self, deadline: Duration) {
            self.control_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: tokio::time::Instant::now().into_std() + deadline,
                    reason: "kafka-test shutdown".to_string(),
                })
                .expect("send shutdown to receiver");
        }

        /// Requests a receiver-first ingress drain with the given `deadline`
        /// from now. Mirrors [`shutdown`] but sends
        /// [`NodeControlMsg::DrainIngress`] so the receiver stops admitting new
        /// records and performs a bounded final commit.
        pub(crate) fn drain(&self, deadline: Duration) {
            self.control_tx
                .send(NodeControlMsg::DrainIngress {
                    deadline: tokio::time::Instant::now().into_std() + deadline,
                    reason: "kafka-test drain".to_string(),
                })
                .expect("send drain ingress to receiver");
        }

        /// Awaits the spawned receiver task's completion.
        pub(crate) async fn await_stopped(self) {
            let _ = self.join.await;
        }

        /// Awaits the spawned receiver task and returns the [`TerminalState`] it
        /// produced at graceful shutdown (the node's final metric snapshots).
        ///
        /// Returns `None` if the task panicked or the node returned an error
        /// instead of a terminal state. Read individual counters from the
        /// snapshots with [`super::node_metrics::metric_value`].
        pub(crate) async fn await_terminal_state(self) -> Option<TerminalState> {
            self.join.await.ok().flatten()
        }
    }
}

#[cfg(feature = "kafka-receiver")]
pub(crate) use receiver_harness::KafkaReceiverHarness;

// ---------------------------------------------------------------------------
// Shared topic layout helper
// ---------------------------------------------------------------------------

/// Per-signal topic/format layout used by the `start_for` wrapper variants.
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
#[derive(Debug, Clone, Default)]
pub(crate) struct KafkaTopics {
    /// Optional traces topic + encoding.
    pub(crate) traces: Option<(String, crate::common::kafka::MessageFormat)>,
    /// Optional metrics topic + encoding.
    pub(crate) metrics: Option<(String, crate::common::kafka::MessageFormat)>,
    /// Optional logs topic + encoding.
    pub(crate) logs: Option<(String, crate::common::kafka::MessageFormat)>,
}

#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
impl KafkaTopics {
    /// A logs-only layout.
    pub(crate) fn logs(topic: impl Into<String>, fmt: crate::common::kafka::MessageFormat) -> Self {
        Self {
            logs: Some((topic.into(), fmt)),
            ..Default::default()
        }
    }

    /// A traces-only layout.
    pub(crate) fn traces(
        topic: impl Into<String>,
        fmt: crate::common::kafka::MessageFormat,
    ) -> Self {
        Self {
            traces: Some((topic.into(), fmt)),
            ..Default::default()
        }
    }
}
