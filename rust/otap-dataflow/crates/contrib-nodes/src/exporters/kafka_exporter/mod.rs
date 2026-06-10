//! Kafka exporter for OpenTelemetry data.
//!
//! This module provides a Kafka exporter that sends telemetry data (traces, metrics, logs)
//! to Apache Kafka topics. It integrates with the OTAP dataflow engine and uses the
//! `rdkafka` client library for Kafka communication.
//!
//! # Features
//!
//! - Per-signal topic and encoding configuration (traces, metrics, logs)
//! - Optional signals — only configure the signals you need
//! - Per-signal encoding: `otlp_proto` or `otap_proto`
//! - Per-signal dynamic topic routing from transport headers
//! - Authentication: SASL with AWS MSK IAM OAUTHBEARER
//! - Producer tuning: `required_acks`, `max_message_bytes`, `linger_ms`
//! - Escape hatch: `producer_config` for arbitrary librdkafka settings
//! - (Planned) resource attribute-based partitioning for stateful processing
//!   (config knob present but not yet implemented; enabling it will currently
//!   trigger a config error to fail fast)
//! - Async-first using `rdkafka::FutureProducer`
//! - Per-signal telemetry metrics
//!
//! # Example Configuration
//!
//! ```yaml
//! nodes:
//!   kafka_exporter:
//!     type: "urn:otel:exporter:kafka"
//!     config:
//!       brokers: "kafka1:9092,kafka2:9092"
//!       client_id: "observability-gateway"
//!       traces:
//!         topic: "otlp_spans"
//!         encoding: "otlp_proto"
//!         topic_from_transport_header: "x_traces_topic"  # optional dynamic routing
//!       metrics:
//!         topic: "otlp_metrics"
//!         encoding: "otlp_proto"
//!       logs:
//!         topic: "otlp_logs"
//!         encoding: "otlp_proto"
//!         topic_from_transport_header: "x_logs_topic"    # optional dynamic routing
//!       timeout_ms: 5000
//!       compression: "zstd"
//!       required_acks: "one"
//!       max_message_bytes: 1000000
//!       linger_ms: 5
//! ```

pub mod config;
pub mod encoder;
pub mod exporter;
pub mod metrics;
pub mod partitioner;
mod producer;
pub mod topic_router;

pub use config::{KafkaExporterConfig, KafkaExporterConfigBuilder};
