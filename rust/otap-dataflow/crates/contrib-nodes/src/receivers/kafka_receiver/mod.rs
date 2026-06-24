// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Implementation of the config settings for the kafka receiver
pub mod config;
/// Error types for the Kafka Receiver.
mod errors;
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
