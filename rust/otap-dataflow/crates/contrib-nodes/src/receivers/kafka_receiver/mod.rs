// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

otap_df_telemetry::otel_component_scope!(
    urn = receiver::KAFKA_RECEIVER_URN,
    target = "otel.receiver.kafka",
);

/// Implementation of the config settings for the kafka receiver
pub mod config;
/// Error types for the Kafka Receiver.
pub mod error;
/// Kafka header extraction and injection into telemetry payloads.
pub mod headers;
/// Implementation of the metrics to collect from the kafka receiver
pub mod metrics;
/// Per-offset tracking for Kafka consumer offset management.
pub mod offset_tracker;
/// Consumer-group rebalance handling (partition assign/revoke callbacks).
pub mod rebalance;
/// Implementation of the main kafka receiver
pub mod receiver;
