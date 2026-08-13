// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public benchmark support for the Kafka exporter.
//!
//! Benchmarks are compiled as separate crates and therefore cannot see this
//! crate's `#[cfg(test)]` mock-broker suite or exporter harness. This module,
//! compiled only under the `kafka_bench` feature, re-exports the pieces a
//! benchmark needs behind a small `pub` facade so the throughput benchmark can
//! drive a
//! fully-wired exporter against the in-process `rdkafka::mocking::MockCluster`
//! without duplicating the engine wiring.
//!
//! # Threading
//!
//! [`BenchCluster`] wraps a `!Send` mock broker that must live on its creation
//! thread. Run the benchmark body on a current-thread runtime + `LocalSet` (see
//! [`run_on_local_set`]).

use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

use otap_df_config::SignalType;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use prost::Message as _;

use crate::common::kafka::MessageFormat;
use crate::common::kafka::node_harness::KafkaExporterHarness;
use crate::common::kafka::test::cluster::{KafkaTestCluster, KafkaTestClusterBuilder};
use crate::common::kafka::test::consumer::TestConsumer;
use crate::common::kafka::test::with_cluster as suite_with_cluster;
use crate::exporters::kafka_exporter::config::{KafkaExporterConfigBuilder, SignalConfig};

pub use crate::common::kafka::MessageFormat as Encoding;
pub use crate::exporters::kafka_exporter::config::KafkaExporterConfig;

/// A running mock broker for benchmarks. Thin re-export wrapper over the
/// internal test cluster so benches can create topics and read the bootstrap
/// address.
pub struct BenchCluster {
    inner: Rc<KafkaTestCluster>,
}

impl BenchCluster {
    /// The mock broker's `bootstrap.servers` string.
    #[must_use]
    pub fn bootstrap_servers(&self) -> String {
        self.inner.bootstrap_servers().to_string()
    }

    /// Number of records the broker has persisted for `topic` / `partition`.
    #[must_use]
    pub fn message_count(&self, topic: &str, partition: i32) -> i64 {
        self.inner.inspect().message_count(topic, partition)
    }

    /// Subscribes a consumer to `topics`.
    ///
    /// Subscribing before driving the exporter establishes topic leadership on
    /// the mock broker so that produce deliveries (and therefore the
    /// shutdown-time `flush`) complete promptly instead of waiting out the
    /// flush deadline.
    #[must_use]
    pub fn subscribe(&self, topics: &[&str]) -> BenchConsumer {
        BenchConsumer {
            inner: self.inner.consumer().subscribe(topics),
        }
    }

    /// Injects a per-request broker round-trip delay on all brokers.
    ///
    /// The in-process mock broker otherwise acknowledges produce requests with
    /// near-zero latency, which hides the whole point of delivery-future
    /// pipelining (overlapping the wait for broker acknowledgements). Injecting
    /// a small round-trip delay makes the `max_in_flight` benefit observable:
    /// a serial (`max_in_flight = 1`) run pays the delay once per batch, while a
    /// pipelined run overlaps up to `max_in_flight` of these waits.
    pub fn inject_round_trip_latency(&self, d: Duration) {
        // Broker id -1 broadcasts the setting to all brokers.
        self.inner.faults().round_trip_time(-1, d);
    }

    /// Starts a fully-wired Kafka exporter against this cluster.
    #[must_use]
    pub fn start_exporter(&self, cfg: KafkaExporterConfig) -> BenchExporter {
        BenchExporter {
            harness: KafkaExporterHarness::start(&self.inner, cfg),
        }
    }
}

/// A consumer subscribed to one or more topics on the bench cluster.
pub struct BenchConsumer {
    inner: TestConsumer,
}

impl BenchConsumer {
    /// Consumes `n` records (bounded internally by the suite's recv timeout).
    pub async fn drain_n(&self, n: usize) {
        let _ = self.inner.recv_n(n).await;
    }
}

/// A running exporter under benchmark, wrapping the internal harness.
pub struct BenchExporter {
    harness: KafkaExporterHarness,
}

impl BenchExporter {
    /// Sends one pdata batch to the exporter.
    pub async fn send(&self, pdata: OtapPdata) {
        self.harness
            .send_pdata(pdata)
            .await
            .expect("bench: send pdata to exporter");
    }

    /// Requests a graceful shutdown with `deadline` from now, then awaits the
    /// node's terminal state (draining all in-flight deliveries).
    pub async fn shutdown_and_wait(self, deadline: Duration) {
        self.harness.shutdown(deadline).await;
        let _ = self.harness.await_terminal_state().await;
    }
}

/// Runs `f` on a current-thread `LocalSet` with a live cluster whose topics are
/// pre-created from `topics` (each `(name, partitions)`).
///
/// The mock broker only auto-creates single-partition topics on produce, so
/// multi-partition topics must be pre-created here.
pub async fn run_on_local_set<F, Fut, T>(topics: &[(&str, i32)], f: F) -> T
where
    F: FnOnce(BenchCluster) -> Fut,
    Fut: Future<Output = T>,
{
    let mut builder: KafkaTestClusterBuilder = KafkaTestCluster::builder();
    for &(name, partitions) in topics {
        builder = builder.topic_with(name, partitions, 1);
    }
    suite_with_cluster(builder, |cluster| f(BenchCluster { inner: cluster })).await
}

/// Builds a validated single-signal exporter config for the given topic,
/// encoding, and in-flight concurrency bound.
#[must_use]
pub fn exporter_config(
    brokers: &str,
    signal: SignalType,
    topic: &str,
    encoding: Encoding,
    max_in_flight: usize,
) -> KafkaExporterConfig {
    let signal_cfg = SignalConfig::new(topic.to_string(), encoding);
    let mut builder = KafkaExporterConfigBuilder::new(brokers, "kafka-bench-exporter");
    builder = match signal {
        SignalType::Traces => builder.with_traces(signal_cfg),
        SignalType::Metrics => builder.with_metrics(signal_cfg),
        SignalType::Logs => builder.with_logs(signal_cfg),
    };
    builder
        .with_max_in_flight(max_in_flight)
        .try_into()
        .expect("bench: exporter config should be valid")
}

/// Builds a logs-only OTLP config pointed at `brokers` with an explicit short
/// `timeout_ms`, used by the unavailable-broker benchmark so each delivery
/// fails within the bound instead of hanging.
#[must_use]
pub fn unavailable_broker_config(
    brokers: &str,
    topic: &str,
    max_in_flight: usize,
    timeout_ms: u64,
) -> KafkaExporterConfig {
    KafkaExporterConfigBuilder::new(brokers, "kafka-bench-unavailable")
        .with_logs(SignalConfig::new(
            topic.to_string(),
            MessageFormat::OtlpProto,
        ))
        .with_max_in_flight(max_in_flight)
        .with_timeout_ms(timeout_ms)
        .try_into()
        .expect("bench: unavailable-broker config should be valid")
}

/// Re-export so benches can spell out signal types.
pub use otap_df_config::SignalType as BenchSignalType;

/// Builds a small but well-formed OTLP proto pdata for the requested signal.
///
/// The payload is a real single-record OTLP request so it round-trips through
/// both the OTLP passthrough and the OTAP encoding paths (which require a
/// decodable OTLP request).
#[must_use]
pub fn sample_pdata(signal: SignalType) -> OtapPdata {
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{Metric, ResourceMetrics, ScopeMetrics};
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};

    let proto = match signal {
        SignalType::Logs => {
            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            time_unix_nano: 1,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            OtlpProtoBytes::ExportLogsRequest(req.encode_to_vec().into())
        }
        SignalType::Traces => {
            let req = ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans {
                    scope_spans: vec![ScopeSpans {
                        spans: vec![Span {
                            name: "bench-span".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            OtlpProtoBytes::ExportTracesRequest(req.encode_to_vec().into())
        }
        SignalType::Metrics => {
            let req = ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    scope_metrics: vec![ScopeMetrics {
                        metrics: vec![Metric {
                            name: "bench-metric".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            OtlpProtoBytes::ExportMetricsRequest(req.encode_to_vec().into())
        }
    };
    OtapPdata::new(Context::default(), proto.into())
}

/// The OTLP-proto encoding.
#[must_use]
pub const fn encoding_otlp() -> Encoding {
    MessageFormat::OtlpProto
}

/// The OTAP-proto (Arrow) encoding.
#[must_use]
pub const fn encoding_otap() -> Encoding {
    MessageFormat::OtapProto
}
