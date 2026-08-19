// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// ETW (Event Tracing for Windows) receiver.
#[cfg(all(feature = "etw-receiver", target_os = "windows"))]
pub mod etw_receiver;

/// Kafka receiver.
#[cfg(feature = "kafka-receiver")]
pub mod kafka_receiver;

/// Oracle OCI polling receiver.
#[cfg(feature = "oracle-receiver")]
pub mod oracle_receiver;

/// Shared polling lifecycle for scraper-style receivers.
#[cfg(feature = "oracle-receiver")]
mod scraper;

/// Narrow database boundary for SQL polling receivers.
#[cfg(feature = "oracle-receiver")]
mod sql_polling;

/// Linux user_events receiver.
#[cfg(all(feature = "user_events-receiver", target_os = "linux"))]
pub mod user_events_receiver;
