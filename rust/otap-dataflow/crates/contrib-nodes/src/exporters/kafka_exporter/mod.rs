// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka exporter for OpenTelemetry data.
//!
//! This module provides a Kafka exporter that sends telemetry data (traces, metrics, logs)
//! to Apache Kafka topics. It integrates with the OTAP dataflow engine and uses the
//! `rdkafka` client library for Kafka communication.
//!
//! # Features
//!
//! - Per-signal topic and encoding configuration (traces, metrics, logs)
//! - Optional signals -- only configure the signals you need
//! - Per-signal encoding: `otlp_proto` or `otap_proto`
//! - Per-signal dynamic topic routing from transport headers
//! - Authentication: SASL with AWS MSK IAM OAUTHBEARER
//! - Producer tuning: `required_acks`, `max_message_bytes`, `linger_ms`
//! - Escape hatch: `producer_config` for arbitrary librdkafka settings
//! - (Planned) resource attribute-based partitioning for stateful processing
//! - Async-first using `rdkafka::FutureProducer`
//! - Per-signal telemetry metrics
//!
//! # Live reconfiguration
//!
//! The exporter accepts live configuration changes via `NodeControlMsg::Config`
//! as a generation cutover: it builds a new producer, swaps it in for pdata
//! processed after the change, and retires the old producer off the event loop.
//!
//! - Each configuration is a generation. A batch is routed and enqueued under
//!   the generation active when the exporter processes it; once enqueued, its
//!   destination is fixed. A batch already in flight on the retiring
//!   generation's producer is therefore committed to the old topic,
//!   credentials, and tenant and is never rerouted across the change.
//! - The cutover is non-blocking: the retiring producer's flush and drop run on
//!   a blocking thread, so a slow or unavailable broker cannot stall normal
//!   processing or backpressure. At most one generation is retiring at a time.
//!
//! See [`exporter::KafkaExporter::reconfigure`] for details.
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
//!         topic_from_transport_header: "x-traces-topic"  # optional dynamic routing
//!       metrics:
//!         topic: "otlp_metrics"
//!         encoding: "otlp_proto"
//!       logs:
//!         topic: "otlp_logs"
//!         encoding: "otlp_proto"
//!         topic_from_transport_header: "x-logs-topic"    # optional dynamic routing
//!       timeout_ms: 5000
//!       compression: "zstd"
//!       required_acks: "one"
//!       max_message_bytes: 1000000
//!       linger_ms: 5
//! ```

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = exporter::KAFKA_EXPORTER_URN,
    target = "otel.exporter.kafka",
);

pub mod config;
pub mod encoder;
pub mod error;
pub mod exporter;
pub mod metrics;
pub mod partitioner;
mod producer;
mod topic_regex;
pub mod topic_router;

pub use config::{KafkaExporterConfig, KafkaExporterConfigBuilder};
pub use error::KafkaExporterError;
