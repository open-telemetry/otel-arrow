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
//! by building a new producer and swapping it in for the old one. This support
//! does NOT yet provide two important guarantees:
//!
//! - It does not guarantee that pdata accepted before the config change is
//!   flushed before the swap, so in-flight records can be sent using the new
//!   topic, credentials, or tenant configuration.
//! - It does not guarantee a non-blocking cutover: the bounded flush and
//!   old-producer retirement run synchronously and can stall the pipeline for
//!   up to the flush timeout.
//!
//! See [`exporter::KafkaExporter::reconfigure`] and the live-reconfiguration
//! issue (<https://github.com/open-telemetry/otel-arrow/issues/3768>) for
//! details.
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

otap_df_telemetry::otel_component_scope!(
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
