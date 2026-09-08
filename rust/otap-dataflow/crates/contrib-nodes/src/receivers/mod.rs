// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// ETW (Event Tracing for Windows) receiver.
#[cfg(all(feature = "etw-receiver", target_os = "windows"))]
pub mod etw_receiver;

/// Shared, database-neutral receiver building blocks.
#[cfg(feature = "database-receiver")]
pub mod database;

/// Kafka receiver.
#[cfg(feature = "kafka-receiver")]
pub mod kafka_receiver;

/// Oracle database receiver.
#[cfg(feature = "oracle-receiver")]
pub mod oracle_receiver;

/// Linux user_events receiver.
#[cfg(all(feature = "user_events-receiver", target_os = "linux"))]
pub mod user_events_receiver;
